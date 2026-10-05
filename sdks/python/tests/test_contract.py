"""Offline contract tests for the Python SDK over a real local HTTP socket.

Unlike ``test_client.py`` (which mocks ``httpx``), these tests run the real
``httpx`` client against a stdlib HTTP server bound to 127.0.0.1, so the request
path, headers, JSON-RPC body, status handling, timeouts and connection failures
are exercised end to end. The responses are copies of the shapes the real
``engram-server`` produces (see ``tests/canonical_journey/contract.rs``).

This is the OFFLINE acceptance lane. It proves serialization and error decoding;
it does NOT prove compatibility with a running server. That is the live lane:
``scripts/test-python-sdk-live.sh`` against an installed wheel.
"""

from __future__ import annotations

import asyncio
import json
import socket
import subprocess
import sys
import threading
import time
from dataclasses import dataclass, field
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Callable

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from engram_client import EngramClient, EngramError  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[3]


def tool_result(payload: object, *, is_error: bool = False) -> dict:
    """The MCP ``CallToolResult`` envelope ``engram-server`` returns for a tool call."""
    result: dict = {
        "content": [{"type": "text", "text": json.dumps(payload, indent=2)}]
    }
    if is_error:
        result["isError"] = True
    return result


def rpc_result(result: object, request_id: int = 1) -> dict:
    return {"jsonrpc": "2.0", "id": request_id, "result": result}


@dataclass
class Recorded:
    path: str
    headers: dict[str, str]
    body: dict


@dataclass
class Reply:
    status: int = 200
    body: bytes | dict = field(default_factory=dict)
    content_type: str = "application/json"
    delay: float = 0.0


class StubServer:
    """A real HTTP server whose reply is chosen per request by ``responder``."""

    def __init__(self) -> None:
        self.requests: list[Recorded] = []
        self.responder: Callable[[Recorded], Reply] = lambda _r: Reply(
            body=rpc_result(tool_result({}))
        )
        stub = self

        class Handler(BaseHTTPRequestHandler):
            def do_POST(self) -> None:  # noqa: N802 - stdlib naming
                length = int(self.headers.get("Content-Length", "0"))
                body = json.loads(self.rfile.read(length) or b"{}")
                recorded = Recorded(
                    self.path,
                    {key.lower(): value for key, value in self.headers.items()},
                    body,
                )
                stub.requests.append(recorded)
                reply = stub.responder(recorded)
                if reply.delay:
                    time.sleep(reply.delay)
                payload = (
                    reply.body
                    if isinstance(reply.body, bytes)
                    else json.dumps(reply.body).encode()
                )
                try:
                    self.send_response(reply.status)
                    self.send_header("Content-Type", reply.content_type)
                    self.send_header("Content-Length", str(len(payload)))
                    self.end_headers()
                    self.wfile.write(payload)
                except (BrokenPipeError, ConnectionResetError):
                    pass  # the client gave up (timeout test)

            def log_message(self, *_args: object) -> None:
                return

        class Server(ThreadingHTTPServer):
            daemon_threads = True
            request_queue_size = 128  # the default backlog of 5 drops bursts of connections

        self._server = Server(("127.0.0.1", 0), Handler)
        self.url = f"http://127.0.0.1:{self._server.server_address[1]}"
        self._thread = threading.Thread(
            target=lambda: self._server.serve_forever(poll_interval=0.01), daemon=True
        )

    def __enter__(self) -> "StubServer":
        self._thread.start()
        return self

    def __exit__(self, *_exc: object) -> None:
        self._server.shutdown()
        self._server.server_close()
        self._thread.join(timeout=5)


@pytest.fixture
def stub():
    with StubServer() as server:
        yield server


def make_client(stub: StubServer, timeout: float = 5.0) -> EngramClient:
    return EngramClient(stub.url, "contract-key", "contract-tenant", timeout=timeout)


# ---------------------------------------------------------------------------
# Wire format
# ---------------------------------------------------------------------------


async def test_request_uses_v1_mcp_bearer_tenant_and_tools_call(stub: StubServer) -> None:
    stub.responder = lambda _r: Reply(body=rpc_result(tool_result({"id": 7})))
    async with make_client(stub) as client:
        assert await client.get(7) == {"id": 7}

    request = stub.requests[0]
    assert request.path == "/v1/mcp"
    assert request.headers["authorization"] == "Bearer contract-key"
    assert request.headers["x-tenant-slug"] == "contract-tenant"
    assert request.headers["content-type"].startswith("application/json")
    assert request.body["jsonrpc"] == "2.0"
    assert isinstance(request.body["id"], int)
    assert request.body["method"] == "tools/call"
    assert request.body["params"] == {"name": "memory_get", "arguments": {"id": 7}}


async def test_arguments_are_snake_case_on_the_wire(stub: StubServer) -> None:
    async with make_client(stub) as client:
        await client.create(
            "content",
            memory_type="issue",
            workspace="ws",
            media_url="https://example.invalid/a.png",
        )
        await client.list(workspace="ws", memory_type="issue", sort_by="created_at")

    for request in stub.requests:
        arguments = request.body["params"]["arguments"]
        camel = [key for key in arguments if key != key.lower() or "-" in key]
        assert not camel, f"camelCase argument keys on the wire: {camel}"
    created = stub.requests[0].body["params"]["arguments"]
    assert created["memory_type"] == "issue"
    assert created["media_url"] == "https://example.invalid/a.png"


# ---------------------------------------------------------------------------
# Pagination
# ---------------------------------------------------------------------------


async def test_list_pagination_parameters_reach_the_wire(stub: StubServer) -> None:
    stub.responder = lambda _r: Reply(body=rpc_result(tool_result([])))
    async with make_client(stub) as client:
        assert await client.list() == []
        assert await client.list(limit=2, offset=4, workspace="ws") == []

    default, paged = (r.body["params"]["arguments"] for r in stub.requests)
    assert (default["limit"], default["offset"]) == (50, 0)
    assert (paged["limit"], paged["offset"], paged["workspace"]) == (2, 4, "ws")
    assert stub.requests[0].body["params"]["name"] == "memory_list"


async def test_search_limit_reaches_the_wire(stub: StubServer) -> None:
    stub.responder = lambda _r: Reply(body=rpc_result(tool_result([])))
    async with make_client(stub) as client:
        await client.search("query", limit=3, workspace="ws")

    call = stub.requests[0].body["params"]
    assert call["name"] == "memory_search"
    assert call["arguments"]["limit"] == 3


# ---------------------------------------------------------------------------
# Envelopes and errors, decoded from real-shaped server responses
# ---------------------------------------------------------------------------

NORMALIZED_ERRORS = {
    "tool_not_found": {
        "error": {
            "code": "tool_not_found",
            "message": "Unknown tool: q4_missing",
            "tool": "q4_missing",
        }
    },
    "permission_denied": {
        "error": {
            "code": "permission_denied",
            "message": "memory_delete requires admin mode",
            "tool": "memory_delete",
            "current_mode": "read_only",
            "required_mode": "admin",
        }
    },
    "invalid_params": {
        "error": {
            "code": "invalid_params",
            "message": "invalid type: integer `123`, expected a string",
        }
    },
    "missing_argument": {
        "error": {
            "code": "missing_argument",
            "message": "Missing required argument: 'id'",
            "details": {"argument": "id"},
        }
    },
    "not_found": {
        "error": {
            "code": "not_found",
            "message": "memory not found: '9'",
            "details": {"entity": "memory", "id": "9"},
        }
    },
}


@pytest.mark.parametrize("code", sorted(NORMALIZED_ERRORS))
async def test_normalized_tool_errors_become_engram_error_with_the_server_message(
    stub: StubServer, code: str
) -> None:
    payload = NORMALIZED_ERRORS[code]
    stub.responder = lambda _r: Reply(
        body=rpc_result(tool_result(payload, is_error=True))
    )
    async with make_client(stub) as client:
        with pytest.raises(EngramError) as raised:
            await client.get(9)

    assert str(raised.value) == payload["error"]["message"]


async def test_legacy_string_errors_become_engram_error(stub: StubServer) -> None:
    stub.responder = lambda _r: Reply(
        body=rpc_result(tool_result({"error": "name is required"}, is_error=True))
    )
    async with make_client(stub) as client:
        with pytest.raises(EngramError, match="name is required"):
            await client.get(1)


async def test_error_payload_without_is_error_flag_still_raises(stub: StubServer) -> None:
    stub.responder = lambda _r: Reply(
        body=rpc_result(tool_result(NORMALIZED_ERRORS["not_found"]))
    )
    async with make_client(stub) as client:
        with pytest.raises(EngramError, match="memory not found"):
            await client.get(9)


async def test_jsonrpc_level_error_becomes_engram_error(stub: StubServer) -> None:
    stub.responder = lambda _r: Reply(
        body={"jsonrpc": "2.0", "id": 1, "error": {"code": -32601, "message": "Method not found: x"}}
    )
    async with make_client(stub) as client:
        with pytest.raises(EngramError, match="Method not found: x"):
            await client.get(1)


@pytest.mark.parametrize(
    ("reply", "expected"),
    [
        (
            Reply(
                status=401,
                body={"jsonrpc": "2.0", "error": {"code": -32001, "message": "Unauthorized"}},
            ),
            "HTTP 401",
        ),
        (
            Reply(status=400, body=b"Failed to parse the request body as JSON", content_type="text/plain"),
            "HTTP 400",
        ),
        (Reply(status=500, body=b"boom", content_type="text/plain"), "HTTP 500"),
        (Reply(status=200, body=b"<html>not json</html>", content_type="text/html"), "invalid JSON"),
        (Reply(status=200, body=b"[1, 2]"), "invalid JSON-RPC response"),
    ],
)
async def test_transport_failures_are_typed(
    stub: StubServer, reply: Reply, expected: str
) -> None:
    stub.responder = lambda _r: reply
    async with make_client(stub) as client:
        with pytest.raises(EngramError, match=expected):
            await client.get(1)


# ---------------------------------------------------------------------------
# Timeout and connection failure
# ---------------------------------------------------------------------------


async def test_timeout_is_typed_bounded_and_the_client_stays_usable(stub: StubServer) -> None:
    slow = Reply(body=rpc_result(tool_result({"id": 1})), delay=1.5)
    fast = Reply(body=rpc_result(tool_result({"id": 2})))
    stub.responder = lambda r: slow if len(stub.requests) == 1 else fast

    async with make_client(stub, timeout=0.2) as client:
        started = time.monotonic()
        with pytest.raises(EngramError, match="Engram request failed"):
            await client.get(1)
        assert time.monotonic() - started < 1.2, "timeout must bound the call"
        assert await client.get(2) == {"id": 2}


async def test_connection_refused_is_typed() -> None:
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        port = probe.getsockname()[1]
    # The port is closed again: nothing listens on it.
    async with EngramClient(f"http://127.0.0.1:{port}", "k", "t", timeout=1.0) as client:
        with pytest.raises(EngramError, match="Engram request failed"):
            await client.get(1)


# ---------------------------------------------------------------------------
# Async lifecycle
# ---------------------------------------------------------------------------


async def test_concurrent_calls_use_unique_request_ids(stub: StubServer) -> None:
    def echo(request: Recorded) -> Reply:
        identifier = request.body["id"]
        return Reply(body=rpc_result(tool_result({"echo": identifier}), identifier))

    stub.responder = echo
    async with make_client(stub) as client:
        results = await asyncio.gather(*(client.get(i) for i in range(25)))

    sent = [request.body["id"] for request in stub.requests]
    assert len(set(sent)) == 25, "JSON-RPC ids must be unique across concurrent calls"
    assert sorted(result["echo"] for result in results) == sorted(sent)


async def test_close_is_idempotent_and_use_after_close_is_typed(stub: StubServer) -> None:
    client = make_client(stub)
    await client.close()
    await client.close()
    with pytest.raises(EngramError, match="closed"):
        await client.get(1)
    assert stub.requests == [], "a closed client must not touch the network"


async def test_context_manager_closes_even_when_the_body_raises(stub: StubServer) -> None:
    client = make_client(stub)
    with pytest.raises(RuntimeError, match="boom"):
        async with client:
            raise RuntimeError("boom")
    with pytest.raises(EngramError, match="closed"):
        await client.get(1)


async def test_cancelled_call_leaves_the_client_usable(stub: StubServer) -> None:
    slow = Reply(body=rpc_result(tool_result({"id": 1})), delay=1.0)
    fast = Reply(body=rpc_result(tool_result({"id": 2})))
    stub.responder = lambda r: slow if len(stub.requests) == 1 else fast

    async with make_client(stub, timeout=5.0) as client:
        with pytest.raises(asyncio.TimeoutError):
            await asyncio.wait_for(client.get(1), timeout=0.1)
        assert await client.get(2) == {"id": 2}


# ---------------------------------------------------------------------------
# Alignment with the MCP registry
# ---------------------------------------------------------------------------


def test_sdk_calls_match_the_registry_baseline() -> None:
    """New SDK/registry drift fails; so does drift fixed but still baselined."""
    completed = subprocess.run(
        [
            sys.executable,
            str(REPO_ROOT / "scripts/check-sdk-contract-alignment.py"),
            "--only",
            "python",
        ],
        capture_output=True,
        text=True,
        check=False,
        timeout=120,
    )
    assert completed.returncode == 0, completed.stdout + completed.stderr

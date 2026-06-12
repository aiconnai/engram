# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "fastmcp",
# ]
# ///
# --- How to run ---
# Dry-run without dependencies:
#   python examples/fastmcp-server/server.py
# Live FastMCP server:
#   uv run examples/fastmcp-server/server.py --live

from __future__ import annotations

import json
import os
import sys
from typing import Final, Literal, TypedDict
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

DEFAULT_ENGRAM_URL: Final = "http://localhost:8080/mcp"
DEFAULT_TOKEN: Final = "dev-engram-token"
DEFAULT_WORKSPACE: Final = "fastmcp-example"


class MemoryCreateArguments(TypedDict):
    content: str
    memory_type: str
    workspace: str
    tags: list[str]


class MemorySearchArguments(TypedDict):
    query: str
    workspace: str
    limit: int


class ToolParams(TypedDict):
    name: str
    arguments: MemoryCreateArguments | MemorySearchArguments


class JsonRpcToolRequest(TypedDict):
    jsonrpc: Literal["2.0"]
    id: int
    method: Literal["tools/call"]
    params: ToolParams


class EngramCallError(RuntimeError):
    operation: str
    detail: str

    def __init__(self, operation: str, detail: str) -> None:
        self.operation = operation
        self.detail = detail
        super().__init__(f"{operation}: {detail}")


def tool_request(
    name: str,
    arguments: MemoryCreateArguments | MemorySearchArguments,
    request_id: int,
) -> JsonRpcToolRequest:
    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "method": "tools/call",
        "params": {"name": name, "arguments": arguments},
    }


def call_engram(request_body: JsonRpcToolRequest) -> str:
    token = os.environ.get("ENGRAM_HTTP_API_KEY", DEFAULT_TOKEN)
    request = Request(
        os.environ.get("ENGRAM_URL", DEFAULT_ENGRAM_URL),
        data=json.dumps(request_body).encode("utf-8"),
        method="POST",
        headers={
            "Authorization": f"Bearer {token}",
            "Content-Type": "application/json",
        },
    )
    try:
        with urlopen(request, timeout=10) as response:
            return response.read().decode("utf-8")
    except HTTPError as error:
        detail = error.read().decode("utf-8")
        raise EngramCallError("Engram HTTP error", detail) from error
    except URLError as error:
        raise EngramCallError("Engram connection error", str(error.reason)) from error


def remember_decision(content: str) -> str:
    return call_engram(
        tool_request(
            "memory_create",
            {
                "content": content,
                "memory_type": "decision",
                "workspace": DEFAULT_WORKSPACE,
                "tags": ["fastmcp", "example"],
            },
            1,
        )
    )


def search_memory(query: str) -> str:
    return call_engram(
        tool_request(
            "memory_search",
            {"query": query, "workspace": DEFAULT_WORKSPACE, "limit": 5},
            2,
        )
    )


def print_dry_run() -> None:
    print("FastMCP tools this server exposes:")
    print("- remember_project_decision(content: str) -> str")
    print("- search_project_memory(query: str) -> str")
    print("\nSample memory_create payload:")
    print(
        json.dumps(
            tool_request(
                "memory_create",
                {
                    "content": "FastMCP example uses Engram as external memory.",
                    "memory_type": "decision",
                    "workspace": DEFAULT_WORKSPACE,
                    "tags": ["fastmcp", "example"],
                },
                1,
            ),
            indent=2,
        )
    )


def run_server() -> None:
    from fastmcp import FastMCP

    mcp = FastMCP("Engram Memory Bridge")

    @mcp.tool
    def remember_project_decision(content: str) -> str:
        """Store a durable project decision in Engram."""
        return remember_decision(content)

    @mcp.tool
    def search_project_memory(query: str) -> str:
        """Search project memory stored in Engram."""
        return search_memory(query)

    mcp.run()


def main() -> int:
    if "--live" not in sys.argv[1:]:
        print_dry_run()
        return 0
    run_server()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

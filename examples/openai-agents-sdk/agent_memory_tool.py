# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "openai-agents",
# ]
# ///
# --- How to run ---
# Dry-run without API keys or network:
#   python examples/openai-agents-sdk/agent_memory_tool.py
# Live OpenAI Agents SDK run:
#   uv run examples/openai-agents-sdk/agent_memory_tool.py --live

from __future__ import annotations

import json
import os
import sys
from typing import Final, Literal, TypedDict
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

DEFAULT_ENGRAM_URL: Final = "http://localhost:8080/mcp"
DEFAULT_TOKEN: Final = "dev-engram-token"
DEFAULT_WORKSPACE: Final = "openai-agents"


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


def create_memory_request(content: str, request_id: int = 1) -> JsonRpcToolRequest:
    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "method": "tools/call",
        "params": {
            "name": "memory_create",
            "arguments": {
                "content": content,
                "memory_type": "decision",
                "workspace": DEFAULT_WORKSPACE,
                "tags": ["openai-agents", "example"],
            },
        },
    }


def search_memory_request(query: str, request_id: int = 2) -> JsonRpcToolRequest:
    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "method": "tools/call",
        "params": {
            "name": "memory_search",
            "arguments": {
                "query": query,
                "workspace": DEFAULT_WORKSPACE,
                "limit": 5,
            },
        },
    }


def call_engram(request_body: JsonRpcToolRequest) -> str:
    body = json.dumps(request_body).encode("utf-8")
    token = os.environ.get("ENGRAM_HTTP_API_KEY", DEFAULT_TOKEN)
    request = Request(
        os.environ.get("ENGRAM_URL", DEFAULT_ENGRAM_URL),
        data=body,
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
    return call_engram(create_memory_request(content))


def search_memory(query: str) -> str:
    return call_engram(search_memory_request(query))


def print_dry_run() -> None:
    print("memory_create payload:")
    print(json.dumps(create_memory_request("Use Engram for durable agent memory."), indent=2))
    print("\nmemory_search payload:")
    print(json.dumps(search_memory_request("durable agent memory"), indent=2))


def run_agent() -> None:
    from agents import Agent, Runner, function_tool

    @function_tool
    def remember_project_decision(content: str) -> str:
        """Store a durable project decision in Engram."""
        return remember_decision(content)

    @function_tool
    def search_project_memory(query: str) -> str:
        """Search durable project memory in Engram."""
        return search_memory(query)

    agent = Agent(
        name="Engram memory demo",
        instructions=(
            "Use the Engram tools for durable decisions. "
            "Do not store secrets, credentials, or raw logs."
        ),
        tools=[remember_project_decision, search_project_memory],
    )
    result = Runner.run_sync(
        agent,
        "Remember that this project uses Engram as durable agent memory, "
        "then search for durable agent memory.",
    )
    print(result.final_output)


def main() -> int:
    if "--live" not in sys.argv[1:]:
        print_dry_run()
        return 0
    run_agent()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

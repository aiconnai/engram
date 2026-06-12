# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "langgraph",
# ]
# ///
# --- How to run ---
# Dry-run without dependencies:
#   python examples/langgraph-tool/memory_graph.py
# Live LangGraph run:
#   uv run examples/langgraph-tool/memory_graph.py --live

from __future__ import annotations

import json
import os
import sys
from typing import Final, Literal, NotRequired, TypedDict
from urllib.error import HTTPError, URLError
from urllib.request import Request, urlopen

DEFAULT_ENGRAM_URL: Final = "http://localhost:8080/mcp"
DEFAULT_TOKEN: Final = "dev-engram-token"
DEFAULT_WORKSPACE: Final = "langgraph-example"


class MemoryGraphState(TypedDict):
    query: str
    decision: str
    workspace: str
    search_result: NotRequired[str]
    create_result: NotRequired[str]


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


def search_memory_node(state: MemoryGraphState) -> MemoryGraphState:
    result = call_engram(
        tool_request(
            "memory_search",
            {
                "query": state["query"],
                "workspace": state["workspace"],
                "limit": 5,
            },
            1,
        )
    )
    return {**state, "search_result": result}


def remember_decision_node(state: MemoryGraphState) -> MemoryGraphState:
    result = call_engram(
        tool_request(
            "memory_create",
            {
                "content": state["decision"],
                "memory_type": "decision",
                "workspace": state["workspace"],
                "tags": ["langgraph", "example"],
            },
            2,
        )
    )
    return {**state, "create_result": result}


def print_dry_run() -> None:
    initial_state: MemoryGraphState = {
        "query": "durable graph memory",
        "decision": "LangGraph example uses Engram as external durable memory.",
        "workspace": DEFAULT_WORKSPACE,
    }
    print("Initial LangGraph state:")
    print(json.dumps(initial_state, indent=2))
    print("\nThe live graph runs search_memory_node, then remember_decision_node.")


def run_graph() -> None:
    from langgraph.graph import END, START, StateGraph

    builder = StateGraph(MemoryGraphState)
    builder.add_node("search_memory", search_memory_node)
    builder.add_node("remember_decision", remember_decision_node)
    builder.add_edge(START, "search_memory")
    builder.add_edge("search_memory", "remember_decision")
    builder.add_edge("remember_decision", END)
    graph = builder.compile()
    result = graph.invoke(
        {
            "query": "durable graph memory",
            "decision": "LangGraph example uses Engram as external durable memory.",
            "workspace": DEFAULT_WORKSPACE,
        }
    )
    print(json.dumps(result, indent=2))


def main() -> int:
    if "--live" not in sys.argv[1:]:
        print_dry_run()
        return 0
    run_graph()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

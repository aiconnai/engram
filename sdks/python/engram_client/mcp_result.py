"""Decode MCP CallToolResult payloads for the Engram HTTP client."""

from __future__ import annotations

import json
from typing import Any

import httpx

from .errors import EngramError


async def post_tool_call(
    client: httpx.AsyncClient,
    *,
    request_id: int,
    method: str,
    params: dict[str, Any] | None,
) -> Any:
    """POST a tools/call JSON-RPC request and decode the tool result."""
    payload = {
        "jsonrpc": "2.0",
        "id": request_id,
        "method": "tools/call",
        "params": {
            "name": method,
            "arguments": params or {},
        },
    }
    try:
        resp = await client.post("/v1/mcp", json=payload)
    except httpx.RequestError as exc:
        raise EngramError(f"Engram request failed: {exc}") from exc
    try:
        resp.raise_for_status()
    except httpx.HTTPStatusError as exc:
        raise EngramError(
            f"HTTP {exc.response.status_code}: {exc.response.text}"
        ) from exc
    try:
        result = resp.json()
    except ValueError as exc:
        raise EngramError("Engram returned invalid JSON") from exc
    if not isinstance(result, dict):
        raise EngramError("Engram returned an invalid JSON-RPC response")
    if "error" in result:
        error = result["error"]
        message = (
            error.get("message", "Unknown error")
            if isinstance(error, dict)
            else error
        )
        raise EngramError(str(message))
    return decode_tool_result(result.get("result", {}))


def error_message(error: Any) -> str:
    """Extract a human-readable message from an MCP/JSON-RPC error payload."""
    if isinstance(error, dict):
        message = error.get("message")
        if message is not None:
            return str(message)
    return str(error)


def decode_tool_result(result: Any) -> Any:
    """Decode an MCP ``CallToolResult`` while preserving legacy responses."""
    if not isinstance(result, dict) or not isinstance(result.get("content"), list):
        return result

    text = next(
        (
            block.get("text")
            for block in result["content"]
            if isinstance(block, dict)
            and block.get("type") == "text"
            and isinstance(block.get("text"), str)
        ),
        None,
    )
    if text is None:
        raise EngramError("Engram MCP response did not contain text content")
    try:
        decoded = json.loads(text)
    except json.JSONDecodeError as exc:
        if result.get("isError") is True:
            raise EngramError(text) from exc
        raise EngramError("Engram MCP response contained invalid JSON") from exc

    if result.get("isError") is True:
        error = (
            decoded.get("error", decoded) if isinstance(decoded, dict) else decoded
        )
        raise EngramError(error_message(error))
    if isinstance(decoded, dict) and "error" in decoded:
        raise EngramError(error_message(decoded["error"]))
    return decoded

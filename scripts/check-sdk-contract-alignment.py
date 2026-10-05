#!/usr/bin/env python3
"""Check that the Python and TypeScript SDKs only call MCP tools and arguments that exist.

Source of truth is the tool registry (``src/mcp/tools``), parsed with the same code
that generates ``docs/MCP_TOOLS.md`` (whose drift is checked in CI), so the chain is
registry -> reference -> SDK calls.

* Python: every public ``EngramClient`` coroutine is executed against a recording
  ``_mcp_call`` twice (required parameters only, then every parameter), so the
  tool name and argument keys come from the real code path.
* TypeScript: ``mcpCall("tool", ...)`` call sites in ``sdks/typescript/src`` are
  parsed statically (object literals and ``params.key = ...`` assignments).

Each drift entry names the SDK call site (Python method, or TypeScript file and
method), so a new method that calls an already-baselined tool, or a new call site
that omits a required argument, is new drift.

The result is compared with ``docs/quality/sdk-contract-drift-baseline.json``. The
baseline is a ratchet: new drift fails, and so does drift that was fixed but is
still listed. ``--update-baseline`` is shrink-only: it refuses to add an entry
unless ``--allow-new-drift --reason TEXT`` is given, and the reason is recorded in
the baseline changelog. The baseline records existing divergences; it never
blesses them.

Exit status: 0 when the computed drift equals the baseline, 1 otherwise, 2 on usage
or environment errors.
"""

from __future__ import annotations

import argparse
import asyncio
import inspect
import json
import re
import sys
import types
import typing
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(Path(__file__).resolve().parent))

BASELINE_PATH = ROOT / "docs/quality/sdk-contract-drift-baseline.json"
PYTHON_SDK = ROOT / "sdks/python"
TYPESCRIPT_SRC = ROOT / "sdks/typescript/src"
BASELINE_VERSION = 1


@dataclass(frozen=True)
class ToolSpec:
    properties: frozenset[str]
    required: frozenset[str]


@dataclass(frozen=True)
class Call:
    """One SDK call site: the tool it invokes and the argument keys it can send."""

    sdk: str
    tool: str
    keys: frozenset[str]
    source: str


# ---------------------------------------------------------------------------
# Registry
# ---------------------------------------------------------------------------


def load_registry() -> dict[str, ToolSpec]:
    import generate_mcp_reference as reference

    registry: dict[str, ToolSpec] = {}
    for tool in reference.parse_tools(reference.DEFAULT_SOURCE):
        schema = tool.schema if isinstance(tool.schema, dict) else {}
        registry[tool.name] = ToolSpec(
            properties=frozenset(schema.get("properties", {})),
            required=frozenset(schema.get("required", [])),
        )
    return registry


# ---------------------------------------------------------------------------
# Python SDK: execute the real methods against a recording transport
# ---------------------------------------------------------------------------


def _sample(annotation: Any) -> Any:
    """A plausible value for a type annotation (only the keys it produces matter)."""
    origin = typing.get_origin(annotation)
    args = [arg for arg in typing.get_args(annotation) if arg is not type(None)]
    if origin is typing.Literal:
        return args[0]
    if origin in (typing.Union, getattr(types, "UnionType", None)):
        return _sample(args[0]) if args else None
    if origin in (list, set, tuple, typing.Sequence):
        return [_sample(args[0])] if args else ["x"]
    if origin is dict or annotation is dict:
        return {"k": "v"}
    if annotation is bool:
        return True
    if annotation is int:
        return 1
    if annotation is float:
        return 0.5
    if annotation is str:
        return "x"
    if annotation is list:
        return ["x"]
    return "x"


def _invoke_arguments(function: Any, include_optional: bool) -> dict[str, Any]:
    try:
        hints = typing.get_type_hints(function)
    except Exception:  # unresolved forward references: fall back to a string value
        hints = {}
    arguments: dict[str, Any] = {}
    for name, parameter in inspect.signature(function).parameters.items():
        if parameter.kind in (parameter.VAR_POSITIONAL, parameter.VAR_KEYWORD):
            continue
        if parameter.default is not inspect.Parameter.empty and not include_optional:
            continue
        arguments[name] = _sample(hints.get(name, str))
    return arguments


def python_calls() -> tuple[list[Call], list[str]]:
    sys.path.insert(0, str(PYTHON_SDK))
    try:
        from engram_client import EngramClient
    except ImportError as exc:
        raise SystemExit(f"Python SDK import failed (is httpx installed?): {exc}") from exc

    recorded: list[tuple[str, str, dict[str, Any]]] = []

    class Recording(EngramClient):
        current = ""

        async def _mcp_call(self, method: str, params: dict[str, Any] | None = None) -> Any:
            recorded.append((self.current, method, dict(params or {})))
            return {}

    unexercised: list[str] = []

    async def sweep() -> None:
        client = Recording("http://127.0.0.1:1", "key", "tenant")
        try:
            for name in sorted(dir(client)):
                function = getattr(client, name)
                if name.startswith("_") or not inspect.iscoroutinefunction(function):
                    continue
                for include_optional in (False, True):
                    client.current = name
                    try:
                        await function(**_invoke_arguments(function, include_optional))
                    except Exception as exc:  # report, never hide: the method is unchecked
                        unexercised.append(f"{name}: {type(exc).__name__}")
                        break
        finally:
            await client.close()

    asyncio.run(sweep())
    calls = [
        Call("python", tool, frozenset(params), method)
        for method, tool, params in recorded
    ]
    return calls, sorted(set(unexercised))


# ---------------------------------------------------------------------------
# TypeScript SDK: static extraction of mcpCall sites
# ---------------------------------------------------------------------------

_CLOSERS = {"{": "}", "[": "]", "(": ")"}


def _matching_close(text: str, open_index: int) -> int:
    """Index of the bracket closing ``text[open_index]``; string and comment aware."""
    stack = [_CLOSERS[text[open_index]]]
    index = open_index + 1
    while index < len(text) and stack:
        char = text[index]
        if char in "\"'`":
            index = _skip_string(text, index)
            continue
        if text.startswith("//", index):
            index = text.find("\n", index)
            index = len(text) if index < 0 else index
            continue
        if text.startswith("/*", index):
            index = text.find("*/", index) + 2
            continue
        if char in _CLOSERS:
            stack.append(_CLOSERS[char])
        elif char == stack[-1]:
            stack.pop()
        index += 1
    if stack:
        raise ValueError("unbalanced brackets in TypeScript source")
    return index - 1


def _skip_string(text: str, start: int) -> int:
    quote = text[start]
    index = start + 1
    while index < len(text):
        if text[index] == "\\":
            index += 2
            continue
        if text[index] == quote:
            return index + 1
        index += 1
    raise ValueError("unterminated string in TypeScript source")


def _split_top_level(text: str) -> Iterable[str]:
    parts, depth, start, index = [], 0, 0, 0
    while index < len(text):
        char = text[index]
        if char in "\"'`":
            index = _skip_string(text, index)
            continue
        if char in "{[(":
            depth += 1
        elif char in "}])":
            depth -= 1
        elif char == "," and depth == 0:
            parts.append(text[start:index])
            start = index + 1
        index += 1
    parts.append(text[start:])
    return parts


def literal_keys_anywhere(expression: str) -> tuple[set[str], bool]:
    """Keys of every top-level ``{ ... }`` literal inside an expression.

    Handles ``cond && { key: value }`` and ``cond ? { a } : {}``. The flag is True
    when a literal cannot be read (computed keys, nested spreads of variables).
    """
    keys: set[str] = set()
    unresolved = False
    index = 0
    while index < len(expression):
        char = expression[index]
        if char in "\"'`":
            index = _skip_string(expression, index)
            continue
        if char == "{":
            end = _matching_close(expression, index)
            literal_keys, literal_unresolved = object_literal_keys(expression[index : end + 1])
            keys |= literal_keys
            unresolved |= literal_unresolved
            index = end + 1
            continue
        index += 1
    return keys, unresolved


def object_literal_keys(literal: str) -> tuple[set[str], bool]:
    """Top-level keys of ``{ ... }`` plus whether something hides further keys."""
    inner = literal.strip()[1:-1]
    keys: set[str] = set()
    unresolved = False
    for item in _split_top_level(inner):
        item = item.strip()
        if not item:
            continue
        if item.startswith("..."):
            spread_keys, spread_unresolved = literal_keys_anywhere(item[3:])
            keys |= spread_keys
            # A spread that contributes no literal at all (e.g. `...options`) is opaque.
            unresolved |= spread_unresolved or not spread_keys
            continue
        match = re.match(r"""^(?:"([^"]+)"|'([^']+)'|([A-Za-z_$][\w$]*))\s*(?::|$)""", item)
        if match is None:
            unresolved = True
            continue
        keys.add(next(group for group in match.groups() if group))
    return keys, unresolved


_CALL = re.compile(r"""mcpCall\(\s*(?:"([^"]+)"|'([^']+)')\s*(?:,\s*)?""")
_ASSIGN = re.compile(r"""\bparams(?:\.([A-Za-z_]\w*)|\[["']([^"']+)["']\])\s*=(?!=)""")
_DECLARE = re.compile(r"""\b(?:const|let)\s+params\b[^=;]*=\s*(?=\{)""")


def typescript_calls() -> tuple[list[Call], list[str]]:
    calls: list[Call] = []
    unresolved: list[str] = []
    for path in sorted(TYPESCRIPT_SRC.rglob("*.ts")):
        if path.name.endswith(".test.ts"):
            continue
        text = path.read_text(encoding="utf-8")
        relative = path.relative_to(ROOT).as_posix()
        previous_end = 0
        for match in _CALL.finditer(text):
            tool = match.group(1) or match.group(2)
            argument_start = match.end()
            window = text[previous_end : match.start()]
            keys, resolved = _keys_for_call(text, argument_start, window)
            previous_end = match.end()
            site = f"{path.relative_to(TYPESCRIPT_SRC).as_posix()}:{_enclosing_method(text, match.start())}"
            if not resolved:
                unresolved.append(f"{relative}:{tool}")
            calls.append(Call("typescript", tool, frozenset(keys), site))
    return calls, sorted(set(unresolved))


_METHOD = re.compile(r"^  (?:public |private |protected |static )*(?:async )?([A-Za-z_$][\w$]*)\s*[(<]", re.M)


def _enclosing_method(text: str, position: int) -> str:
    """Name of the two-space-indented class member that contains ``position``."""
    name = "?"
    for match in _METHOD.finditer(text, 0, position):
        if match.group(1) not in ("if", "for", "while", "switch", "return", "constructor"):
            name = match.group(1)
    return name


def _keys_for_call(text: str, argument_start: int, window: str) -> tuple[set[str], bool]:
    """Keys passed as the second argument of one ``mcpCall(...)``; flag False if unreadable."""
    call_open = text.rfind("(", 0, argument_start)
    argument = text[argument_start : _matching_close(text, call_open)].strip()
    if not argument:
        return set(), True
    if not re.match(r"params\b", argument):
        keys, unresolved = literal_keys_anywhere(argument)
        return keys, not unresolved and bool(keys or argument in ("{}",))
    keys: set[str] = set()
    resolved = True
    declaration = None
    for declaration in _DECLARE.finditer(window):
        pass
    if declaration is not None:
        start = declaration.end()
        literal_keys, unresolved = object_literal_keys(
            window[start : _matching_close(window, start) + 1]
        )
        keys |= literal_keys
        resolved = not unresolved
    for assignment in _ASSIGN.finditer(window):
        keys.add(assignment.group(1) or assignment.group(2))
    return keys, resolved


# ---------------------------------------------------------------------------
# Drift
# ---------------------------------------------------------------------------


def compute_drift(
    registry: dict[str, ToolSpec],
    calls: Iterable[Call],
    unresolved: Iterable[str] = (),
    unexercised: Iterable[str] = (),
) -> set[str]:
    """Drift entries, one string each, keyed by SDK call site (``...@site``).

    Keys are unioned only within one call site (a Python method is swept twice, with
    required and with all parameters), never across sites of the same tool.
    """
    drift: set[str] = set()
    sent: dict[tuple[str, str, str], set[str]] = {}
    for call in calls:
        sent.setdefault((call.sdk, call.tool, call.source), set()).update(call.keys)
    for (sdk, tool, site), keys in sorted(sent.items()):
        spec = registry.get(tool)
        if spec is None:
            drift.add(f"{sdk}:unknown_tool:{tool}@{site}")
            continue
        for key in sorted(keys - spec.properties):
            drift.add(f"{sdk}:unknown_argument:{tool}.{key}@{site}")
        for key in sorted(spec.required - keys):
            drift.add(f"{sdk}:required_never_sent:{tool}.{key}@{site}")
    drift.update(f"typescript:unresolved_call:{entry}" for entry in unresolved)
    drift.update(f"python:unexercised_method:{entry}" for entry in unexercised)
    return drift


SDKS = ("python", "typescript")


def collect_drift(only: str | None = None) -> set[str]:
    """Drift for both SDKs, or for one when ``only`` names it (the other is not run)."""
    registry = load_registry()
    calls: list[Call] = []
    python_unexercised: list[str] = []
    typescript_unresolved: list[str] = []
    if only in (None, "python"):
        python, python_unexercised = python_calls()
        calls.extend(python)
    if only in (None, "typescript"):
        typescript, typescript_unresolved = typescript_calls()
        calls.extend(typescript)
    return compute_drift(registry, calls, typescript_unresolved, python_unexercised)


def load_baseline(path: Path = BASELINE_PATH) -> set[str]:
    document = json.loads(path.read_text(encoding="utf-8"))
    if document.get("version") != BASELINE_VERSION:
        raise SystemExit(f"{path}: unsupported baseline version {document.get('version')!r}")
    entries = document.get("known_drift")
    if not isinstance(entries, list) or not all(isinstance(entry, str) for entry in entries):
        raise SystemExit(f"{path}: known_drift must be a list of strings")
    return set(entries)


def _load_changelog(path: Path) -> list[dict[str, Any]]:
    if not path.exists():
        return []
    changelog = json.loads(path.read_text(encoding="utf-8")).get("changelog", [])
    return changelog if isinstance(changelog, list) else []


def write_baseline(
    drift: set[str],
    path: Path = BASELINE_PATH,
    changelog: list[dict[str, Any]] | None = None,
) -> None:
    document = {
        "version": BASELINE_VERSION,
        "description": (
            "SDK calls that do not match the MCP tool registry, one entry per SDK call "
            "site. A ratchet: it records existing divergence, it does not approve it. "
            "Shrink with scripts/check-sdk-contract-alignment.py --update-baseline; "
            "adding entries needs --allow-new-drift --reason."
        ),
        "known_drift": sorted(drift),
        "changelog": changelog if changelog is not None else _load_changelog(path),
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(document, indent=2) + "\n", encoding="utf-8")


def update_baseline(
    drift: set[str],
    path: Path = BASELINE_PATH,
    allow_new_drift: bool = False,
    reason: str | None = None,
) -> int:
    """Shrink-only rewrite. Returns the exit status; nothing is written on refusal."""
    existing = load_baseline(path) if path.exists() else set()
    added, removed = sorted(drift - existing), sorted(existing - drift)
    for entry in added:
        print(f"ADD    {entry}")
    for entry in removed:
        print(f"REMOVE {entry}")
    if added and not allow_new_drift:
        print(
            f"refusing to bless {len(added)} new drift entr{'y' if len(added) == 1 else 'ies'}: "
            "fix the SDK or the registry, or pass --allow-new-drift --reason TEXT",
            file=sys.stderr,
        )
        return 1
    if added and not (reason and reason.strip()):
        print("--allow-new-drift requires a non-empty --reason", file=sys.stderr)
        return 1
    changelog = _load_changelog(path)
    if added or removed:
        changelog.append(
            {"reason": (reason or "shrink: drift fixed").strip(), "added": len(added), "removed": len(removed)}
        )
    write_baseline(drift, path, changelog)
    print(f"baseline updated: {len(added)} added, {len(removed)} removed, {len(drift)} total")
    return 0


def compare(
    drift: set[str], baseline: set[str], only: str | None = None
) -> tuple[list[str], list[str]]:
    if only is not None:
        baseline = {entry for entry in baseline if entry.startswith(f"{only}:")}
    return sorted(drift - baseline), sorted(baseline - drift)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument(
        "--update-baseline",
        action="store_true",
        help="rewrite the baseline; shrink-only unless --allow-new-drift",
    )
    parser.add_argument(
        "--allow-new-drift",
        action="store_true",
        help="with --update-baseline: allow ADDING entries (requires --reason)",
    )
    parser.add_argument("--reason", help="why new drift is being recorded (kept in the changelog)")
    parser.add_argument("--list", action="store_true", help="print the computed drift and exit")
    parser.add_argument(
        "--only",
        choices=SDKS,
        help="check one SDK only (the TypeScript check needs no Python dependencies)",
    )
    args = parser.parse_args(argv)
    if args.update_baseline and args.only:
        parser.error("--update-baseline rewrites the whole baseline; drop --only")
    if (args.allow_new_drift or args.reason) and not args.update_baseline:
        parser.error("--allow-new-drift/--reason only apply to --update-baseline")

    drift = collect_drift(args.only)
    if args.list:
        print("\n".join(sorted(drift)))
        return 0
    if args.update_baseline:
        return update_baseline(drift, BASELINE_PATH, args.allow_new_drift, args.reason)

    new, fixed = compare(drift, load_baseline(), args.only)
    for entry in new:
        print(f"NEW DRIFT (fix the SDK or the registry): {entry}")
    for entry in fixed:
        print(f"STALE BASELINE (drift is gone, remove it): {entry}")
    if new or fixed:
        return 1
    print(f"sdk contract alignment: OK ({len(drift)} known drift entries, none new)")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Generate the MCP tools reference from src/mcp/tools/registry.rs."""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = ROOT / "src/mcp/tools/registry.rs"
DEFAULT_OUTPUT = ROOT / "docs/MCP_TOOLS.md"


@dataclass(frozen=True)
class Tool:
    name: str
    description: str
    schema: dict
    annotations: str
    tier: str
    group: str
    required_feature: str | None


NAME_RE = re.compile(r'name:\s*"(?P<value>(?:\\.|[^"\\])*)"')
DESCRIPTION_RE = re.compile(r'description:\s*"(?P<value>(?:\\.|[^"\\])*)"')
SCHEMA_RE = re.compile(
    r'schema:\s*r(?P<hashes>\#*)"(?P<value>.*?)"(?P=hashes)\s*,',
    re.DOTALL,
)
TIER_RE = re.compile(r"tier:\s*ToolTier::(?P<value>Essential|Standard|Advanced)")
INCLUDE_RE = re.compile(r"=\s*include!\(\s*\"(?P<path>[^\"]+)\"\s*\)")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate or check docs/MCP_TOOLS.md from MCP tool definitions"
    )
    parser.add_argument("--source", type=Path, default=DEFAULT_SOURCE)
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail if the output file differs from generated content",
    )
    parser.add_argument(
        "--stdout",
        action="store_true",
        help="print generated content instead of writing the output file",
    )
    args = parser.parse_args()

    tools = parse_tools(args.source)
    markdown = render_reference(tools, args.source)

    if args.stdout:
        print(markdown, end="")
        return 0

    if args.check:
        existing = args.output.read_text()
        if existing != markdown:
            print(
                f"{args.output.relative_to(ROOT)} is stale; "
                "run `./scripts/generate-mcp-reference.sh`",
                file=sys.stderr,
            )
            return 1
        print(f"{args.output.relative_to(ROOT)} is up to date")
        return 0

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(markdown)
    print(f"wrote {args.output.relative_to(ROOT)}")
    return 0


def parse_tools(source: Path) -> list[Tool]:
    text = extract_tool_definition_source(source)
    tools: list[Tool] = []
    for block in tool_blocks(text):
        name = decode_rust_string(required_match(NAME_RE, block, "name"))
        schema_text = required_match(SCHEMA_RE, block, f"{name} schema")
        try:
            schema = json.loads(schema_text)
        except json.JSONDecodeError as exc:
            raise ValueError(f"tool {name!r} has invalid JSON schema: {exc}") from exc
        tools.append(
            Tool(
                name=name,
                description=decode_rust_string(
                    required_match(DESCRIPTION_RE, block, f"{name} description")
                ),
                schema=schema,
                annotations=annotation_summary(block),
                tier=required_match(TIER_RE, block, f"{name} tier").lower(),
                group=tool_group(name),
                required_feature=required_feature(name),
            )
        )

    if not tools:
        raise ValueError(f"no ToolDef entries found in {source}")
    return tools


def required_match(pattern: re.Pattern[str], text: str, label: str) -> str:
    match = pattern.search(text)
    if not match:
        raise ValueError(f"missing {label}")
    return match.group("value")


def extract_tool_definition_source(source: Path) -> str:
    text = source.read_text()
    definitions = text.find("pub const TOOL_DEFINITIONS")
    if definitions != -1:
        text = text[definitions:]
        include = INCLUDE_RE.search(text)
        if include:
            include_path = (source.parent / include.group("path")).resolve()
            return extract_tool_definition_source(include_path)
        return text

    text = text.lstrip()
    if text.startswith("&["):
        return text

    raise ValueError("missing TOOL_DEFINITIONS")


def tool_blocks(text: str) -> list[str]:
    definitions = text.find("pub const TOOL_DEFINITIONS")
    if definitions == -1:
        # For sources that are included directly as a slice literal (`&[ ... ]`).
        if not text.startswith("&["):
            raise ValueError("missing TOOL_DEFINITIONS")
        # Keep whole text.
    else:
        text = text[definitions:]
    blocks: list[str] = []
    cursor = 0
    while True:
        start = text.find("ToolDef {", cursor)
        if start == -1:
            return blocks
        brace = text.find("{", start)
        end = matching_brace(text, brace)
        blocks.append(text[brace + 1 : end])
        cursor = end + 1


def matching_brace(text: str, start: int) -> int:
    depth = 0
    index = start
    while index < len(text):
        if text.startswith('r#"', index):
            index = skip_raw_string(text, index + 1)
            continue
        char = text[index]
        if char == '"':
            index = skip_string(text, index)
            continue
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return index
        index += 1
    raise ValueError("unterminated ToolDef block")


def skip_string(text: str, start: int) -> int:
    index = start + 1
    while index < len(text):
        if text[index] == "\\":
            index += 2
            continue
        if text[index] == '"':
            return index + 1
        index += 1
    raise ValueError("unterminated string literal")


def skip_raw_string(text: str, start: int) -> int:
    hashes = 0
    while text[start + hashes] == "#":
        hashes += 1
    quote = start + hashes
    if quote >= len(text) or text[quote] != '"':
        return start
    terminator = '"' + ("#" * hashes)
    end = text.find(terminator, quote + 1)
    if end == -1:
        raise ValueError("unterminated raw string literal")
    return end + len(terminator)


def decode_rust_string(value: str) -> str:
    return json.loads(f'"{value}"')


def render_reference(tools: list[Tool], source: Path) -> str:
    relative_source = source.relative_to(ROOT)
    lines = [
        "# MCP Tools Reference",
        "",
        "<!-- GENERATED: do not edit manually. Run `./scripts/generate-mcp-reference.sh`. -->",
        "",
        "This reference documents the MCP surface that turns Engram into a shared source of truth for team memory.",
        "",
        f"It is generated from `{relative_source}`.",
        "",
        f"Total tools: **{len(tools)}**",
        "",
        "## Summary",
        "",
        "| Tool | Tier | Group | Feature | Annotations | Required Inputs |",
        "|------|------|-------|---------|-------------|-----------------|",
    ]

    for tool in tools:
        required = required_fields(tool.schema)
        lines.append(
            f"| `{escape_table(tool.name)}` | {tool.tier} | {escape_table(tool.group)} | "
            f"{escape_table(tool.required_feature or 'always')} | "
            f"{escape_table(tool.annotations)} | {escape_table(required_summary(required))} |"
        )

    lines.extend(["", "## Tools", ""])
    for tool in tools:
        required = required_fields(tool.schema)
        properties = schema_properties(tool.schema)
        lines.extend(
            [
                f"### `{tool.name}`",
                "",
                tool.description,
                "",
                f"- Tier: `{tool.tier}`",
                f"- Group: `{tool.group}`",
                f"- Required feature: `{tool.required_feature or 'always'}`",
                f"- Annotations: {tool.annotations}",
                f"- Required inputs: {required_summary(required)}",
                "",
                "| Input | Type | Required | Summary |",
                "|-------|------|----------|---------|",
            ]
        )
        if properties:
            for prop in properties:
                required_label = "yes" if prop["name"] in required else "no"
                lines.append(
                    f"| `{escape_table(prop['name'])}` | `{escape_table(prop['type'])}` | "
                    f"{required_label} | {escape_table(prop['summary'])} |"
                )
        else:
            lines.append("| _(none)_ |  | no | No input properties declared. |")
        lines.append("")

    while lines and lines[-1] == "":
        lines.pop()
    return "\n".join(lines) + "\n"


def required_feature(name: str) -> str | None:
    match name:
        case (
            "langfuse_connect"
            | "langfuse_sync"
            | "langfuse_sync_status"
            | "langfuse_extract_patterns"
            | "memory_from_trace"
        ):
            return "langfuse"
        case (
            "meilisearch_search"
            | "meilisearch_reindex"
            | "meilisearch_status"
            | "meilisearch_config"
        ):
            return "meilisearch"
        case (
            "memory_auto_link"
            | "memory_list_auto_links"
            | "memory_auto_link_stats"
            | "memory_cluster"
            | "memory_get_cluster"
            | "memory_list_clusters"
        ):
            return "emergent-graph"
        case (
            "memory_sync_media"
            | "memory_describe_image"
            | "memory_transcribe_audio"
            | "memory_capture_screenshot"
            | "memory_process_video"
            | "memory_list_media"
            | "memory_search_by_image"
        ):
            return "multimodal"
        case "memory_graph_path" | "memory_temporal_snapshot" | "memory_scope_snapshot":
            return "duckdb-graph"
        case (
            "dream_run_now"
            | "dream_create"
            | "dream_get"
            | "dream_list"
            | "dream_cancel"
            | "dream_archive"
            | "dream_candidates_list"
            | "dream_candidate_get"
            | "dream_candidate_review"
            | "dream_candidate_apply"
            | "memory_agent_writeback"
            | "dream_eval_run"
        ):
            return "dream-phase"
        case (
            "attestation_log"
            | "attestation_verify"
            | "attestation_chain_verify"
            | "attestation_list"
        ):
            return "attestation"
        case "snapshot_create" | "snapshot_load" | "snapshot_inspect":
            return "snapshot"
        case _:
            return None


def tool_group(name: str) -> str:
    feature = required_feature(name)
    if feature is not None:
        match feature:
            case "langfuse":
                return "feature.langfuse"
            case "meilisearch":
                return "feature.meilisearch"
            case "emergent-graph":
                return "feature.emergent_graph"
            case "multimodal":
                return "feature.multimodal"
            case "duckdb-graph":
                return "feature.duckdb_graph"
            case "dream-phase":
                return "feature.dream"
            case "attestation":
                return "feature.attestation"
            case "snapshot":
                return "feature.snapshot"
            case _:
                return "feature.other"

    match name:
        case "discover_tools" | "recent_activity" | "memory_agent_contract":
            return "core"
        case (
            "context_seed"
            | "context_record"
            | "context_record_artifact"
            | "context_get_artifact"
            | "context_search"
            | "context_build_bundle"
            | "context_budget_check"
        ):
            return "context"
        case _:
            pass

    prefix = name.split("_", maxsplit=1)[0]
    match prefix:
        case "identity":
            return "identity"
        case "session":
            return "session"
        case "workspace":
            return "workspace"
        case "quality" | "salience":
            return "quality"
        case "scope":
            return "scope"
        case "temporal":
            return "temporal"
        case "sync":
            return "sync"
        case "agent":
            return "agent"
        case "harness":
            return "harness"
        case "lifecycle" | "retention":
            return "lifecycle"
        case "attestation" | "snapshot":
            return "portability"
        case "embedding":
            return "embedding"
        case "search":
            return "search"
        case "pending":
            return "admin"
        case "memory":
            return memory_subgroup(name)
        case _:
            return "misc"


def memory_subgroup(name: str) -> str:
    if any(
        needle in name
        for needle in (
            "search",
            "retrieve",
            "digest",
            "expand",
            "related",
            "traverse",
            "find_path",
            "smart",
            "injection",
        )
    ):
        return "memory.search"
    if "identity" in name:
        return "identity"
    if "block" in name:
        return "memory.block"
    if any(needle in name for needle in ("quality", "conflict", "duplicate", "reconcile")):
        return "memory.quality"
    if any(
        needle in name
        for needle in (
            "lifecycle",
            "archive",
            "decay",
            "promote",
            "cleanup",
            "expir",
            "consolidat",
            "garden",
            "score",
            "policy",
        )
    ):
        return "memory.lifecycle"
    if any(
        needle in name
        for needle in (
            "entity",
            "link",
            "cluster",
            "coactivation",
            "fact",
            "triplet",
            "knowledge",
            "reflect",
        )
    ):
        return "memory.graph"
    if any(
        needle in name
        for needle in (
            "session",
            "working_memory",
            "checkpoint",
            "observe_tool",
            "archived_output",
        )
    ):
        return "memory.session"
    if any(
        needle in name
        for needle in (
            "enrichment",
            "replay",
            "events",
            "stats",
            "versions",
            "cache",
            "embedding",
            "share",
            "import",
            "export",
            "migrate",
            "rebuild",
            "tag",
            "validate",
            "upload",
            "compress",
            "sentiment",
            "feedback",
            "utility",
            "synthesis",
            "detect",
            "suggest",
        )
    ):
        return "memory.admin"
    return "memory.core"


def required_fields(schema: dict) -> set[str]:
    required = schema.get("required", [])
    return {field for field in required if isinstance(field, str)}


def required_summary(required: set[str]) -> str:
    if not required:
        return "none"
    return ", ".join(f"`{field}`" for field in sorted(required))


def schema_properties(schema: dict) -> list[dict[str, str]]:
    properties = schema.get("properties")
    if not isinstance(properties, dict):
        return []
    return [
        {
            "name": name,
            "type": schema_type(value),
            "summary": property_summary(value),
        }
        for name, value in properties.items()
        if isinstance(value, dict)
    ]


def schema_type(value: dict) -> str:
    type_value = value.get("type")
    if isinstance(type_value, str):
        return type_value
    if isinstance(type_value, list):
        return " | ".join(str(item) for item in type_value)
    if "properties" in value:
        return "object"
    if "items" in value:
        return "array"
    return "any"


def property_summary(value: dict) -> str:
    parts: list[str] = []
    description = value.get("description")
    if isinstance(description, str):
        parts.append(description)
    if "default" in value:
        parts.append(f"Default: `{inline_json(value['default'])}`.")
    enum = value.get("enum")
    if isinstance(enum, list):
        allowed = ", ".join(f"`{inline_json(item)}`" for item in enum)
        parts.append(f"Allowed: {allowed}.")
    json_format = value.get("format")
    if isinstance(json_format, str):
        parts.append(f"Format: `{json_format}`.")
    items = value.get("items")
    if isinstance(items, dict):
        parts.append(f"Items: `{schema_type(items)}`.")
    for key, label in (
        ("minimum", "Minimum"),
        ("maximum", "Maximum"),
        ("minItems", "Min items"),
        ("maxLength", "Max length"),
    ):
        if key in value:
            parts.append(f"{label}: `{inline_json(value[key])}`.")
    return " ".join(parts) if parts else "No description."


def inline_json(value: object) -> str:
    if isinstance(value, str):
        return value
    return json.dumps(value, separators=(",", ":"))


def annotation_summary(value: str) -> str:
    if "read_only()" in value:
        return "readOnlyHint"
    if "destructive()" in value:
        return "destructiveHint"
    if "idempotent()" in value:
        return "idempotentHint"

    hints = []
    if "read_only_hint: Some(true)" in value:
        hints.append("readOnlyHint")
    if "destructive_hint: Some(true)" in value:
        hints.append("destructiveHint")
    if "idempotent_hint: Some(true)" in value:
        hints.append("idempotentHint")
    if "open_world_hint: Some(true)" in value:
        hints.append("openWorldHint")
    return ", ".join(hints) if hints else "mutating (no MCP hints)"


def escape_table(value: str) -> str:
    return value.replace("|", "\\|").replace("\n", " ")


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Validate the static MCP contract and emit a canonical harness JSON report."""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import generate_mcp_reference as reference


ROOT = Path(__file__).resolve().parents[1]
SCHEMA_VERSION = "v1"
TOOL_NAME = "mcp-contract-validator"

CRITICAL_TOOLS = [
    "memory_create",
    "memory_get",
    "memory_search",
    "memory_list",
    "context_search",
    "context_build_bundle",
    "harness_status",
]

CRITICAL_REQUIRED_FIELDS = {
    "memory_create": ["content"],
    "memory_get": ["id"],
    "context_search": ["query"],
}

CONTEXT_BUNDLE_SELECTORS = {
    "query",
    "repo_id",
    "workspace_path_hash",
    "workspace",
    "session_id",
    "task_id",
}

READ_ONLY_TOOLS = {
    "memory_get",
    "memory_list",
    "memory_search",
    "context_search",
    "context_build_bundle",
    "harness_status",
}

DESTRUCTIVE_TOOLS = {
    "memory_delete",
    "memory_cleanup_expired",
    "embedding_cache_clear",
}

IDEMPOTENT_TOOLS = {
    "lifecycle_run",
    "retention_policy_apply",
    "memory_rebuild_embeddings",
}

ADMIN_TOOLS = {
    "agent_deregister",
    "agent_register",
    "embedding_cache_clear",
    "identity_delete",
    "memory_cache_clear",
    "memory_delete",
    "memory_delete_batch",
    "memory_events_clear",
    "memory_grant_access",
    "memory_revoke_access",
    "retention_policy_delete",
    "search_cache_clear",
    "session_delete",
    "workspace_delete",
}

VALID_TIERS = {"essential", "standard", "advanced"}

DISPATCH_ALIASES = {
    "memory_seed",
    "graph_cluster_concepts",
    "graph_predict_links",
}

DISPATCH_ARM_RE = re.compile(
    r'^\s*((?:"(?:\\.|[^"\\])*"\s*\|\s*)*"(?:\\.|[^"\\])*")\s*=>',
    re.MULTILINE,
)
DISPATCH_NAME_RE = re.compile(r'"(?P<name>(?:\\.|[^"\\])*)"')
DOC_HEADING_RE = re.compile(r"^### `(?P<name>[^`]+)`\s*$", re.MULTILINE)
GENERATED_MARKER = "<!-- GENERATED: do not edit manually. Run `./scripts/generate-mcp-reference.sh`. -->"


@dataclass(frozen=True)
class ToolDefParsed:
    name: str
    description: str
    schema: Any
    schema_error: str | None
    tier: str | None
    annotations: str
    raw_block: str


def repo_path(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(ROOT))
    except ValueError:
        return str(path)


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def has_annotation(tool_or_ann: Any, annotation: str) -> bool:
    if hasattr(tool_or_ann, "annotations"):
        ann_str = tool_or_ann.annotations
    else:
        ann_str = str(tool_or_ann)
    return annotation in {part.strip() for part in ann_str.split(",")}


def add_check(
    checks: list[dict[str, Any]],
    check_id: str,
    status: str,
    message: str,
    path: Path | str | None = None,
) -> None:
    item: dict[str, Any] = {
        "id": check_id,
        "status": status,
        "message": message,
    }
    if path is not None:
        item["path"] = repo_path(path) if isinstance(path, Path) else path
    checks.append(item)


def add_failure(failures: list[str], message: str) -> None:
    failures.append(message)


def add_warning(warnings: list[str], message: str) -> None:
    warnings.append(message)


def parse_tools_tolerant(source: Path) -> list[ToolDefParsed]:
    text = reference.extract_tool_definition_source(source)
    blocks = reference.tool_blocks(text)
    tools: list[ToolDefParsed] = []
    tier_pattern = re.compile(r"tier:\s*(?:ToolTier::)?(?P<value>[A-Za-z0-9_]+)")

    for block in blocks:
        name_m = reference.NAME_RE.search(block)
        name = reference.decode_rust_string(name_m.group("value")) if name_m else ""

        desc_m = reference.DESCRIPTION_RE.search(block)
        description = reference.decode_rust_string(desc_m.group("value")) if desc_m else ""

        schema_m = reference.SCHEMA_RE.search(block)
        schema = None
        schema_error = None
        if schema_m:
            try:
                schema = json.loads(schema_m.group("value"))
            except Exception as exc:  # noqa: BLE001
                schema_error = str(exc)
        else:
            schema_error = "missing schema"

        rust_fields = reference.SCHEMA_RE.sub("", block) if schema_m else block
        tier_m = tier_pattern.search(rust_fields)
        tier = tier_m.group("value").lower() if tier_m else None
        annotations = reference.annotation_summary(rust_fields)

        tools.append(
            ToolDefParsed(
                name=name,
                description=description,
                schema=schema,
                schema_error=schema_error,
                tier=tier,
                annotations=annotations,
                raw_block=block,
            )
        )
    return tools


def to_reference_tool(tool: ToolDefParsed) -> reference.Tool:
    return reference.Tool(
        name=tool.name,
        description=tool.description,
        schema=tool.schema if isinstance(tool.schema, dict) else {},
        annotations=tool.annotations,
        tier=tool.tier or "standard",
        group=reference.tool_group(tool.name),
        required_features=reference.required_features(tool.name),
    )


def parse_dispatch_names(path: Path) -> set[str]:
    text = path.read_text(encoding="utf-8")
    names: set[str] = set()
    for match in DISPATCH_ARM_RE.finditer(text):
        arm = match.group(1)
        for n_match in DISPATCH_NAME_RE.finditer(arm):
            names.add(json.loads(f'"{n_match.group("name")}"'))
    return names


def parse_doc_names(path: Path) -> set[str]:
    text = path.read_text(encoding="utf-8")
    return {match.group("name") for match in DOC_HEADING_RE.finditer(text)}


def default_docs_comparison_allowed(registry_path: Path) -> bool:
    try:
        registry_path.resolve().relative_to(ROOT)
        return True
    except ValueError:
        return False


def validate_tool_schema(
    tool: ToolDefParsed,
    checks: list[dict[str, Any]],
    failures: list[str],
    registry_path: Path,
) -> None:
    # 1. Non-empty name
    if not tool.name or not tool.name.strip():
        add_failure(failures, "Tool definition has empty name")
        add_check(checks, "mcp_schema:name:empty", "fail", "tool name is empty", registry_path)
    else:
        add_check(checks, f"mcp_schema:name:{tool.name}", "pass", "tool name is non-empty", registry_path)

    tool_label = tool.name if tool.name else "unnamed"

    # 2. Non-empty description
    if not tool.description or not tool.description.strip():
        add_failure(failures, f"{tool_label} description is empty")
        add_check(checks, f"mcp_schema:description:{tool_label}", "fail", "tool description is empty", registry_path)
    else:
        add_check(checks, f"mcp_schema:description:{tool.name}", "pass", "tool description is non-empty", registry_path)

    # 3. Valid JSON schema object
    if tool.schema_error is not None:
        add_failure(failures, f"{tool_label} schema is invalid JSON: {tool.schema_error}")
        add_check(checks, f"mcp_schema:object:{tool_label}", "fail", f"invalid JSON schema: {tool.schema_error}", registry_path)
        return

    schema = tool.schema
    if not isinstance(schema, dict):
        add_failure(failures, f"{tool_label} schema is not an object")
        add_check(checks, f"mcp_schema:object:{tool_label}", "fail", "schema is not an object", registry_path)
        return

    if schema.get("type") != "object":
        add_failure(failures, f"{tool_label} schema type must be 'object'")
        add_check(checks, f"mcp_schema:object:{tool_label}", "fail", "schema type is not 'object'", registry_path)
        return

    add_check(checks, f"mcp_schema:object:{tool.name}", "pass", "schema is a valid JSON object", registry_path)

    # 4. ToolTier correctness
    if not tool.tier or tool.tier.lower() not in VALID_TIERS:
        add_failure(failures, f"{tool_label} has invalid or missing ToolTier: {tool.tier!r}")
        add_check(checks, f"mcp_schema:tier:{tool_label}", "fail", "invalid ToolTier", registry_path)
    else:
        add_check(checks, f"mcp_schema:tier:{tool.name}", "pass", "valid ToolTier", registry_path)

    # 5. ToolAnnotations presence
    if not tool.annotations or not tool.annotations.strip():
        add_failure(failures, f"{tool_label} has missing ToolAnnotations")
        add_check(checks, f"mcp_schema:annotations:{tool_label}", "fail", "missing ToolAnnotations", registry_path)
    else:
        add_check(checks, f"mcp_schema:annotations:{tool.name}", "pass", "valid ToolAnnotations", registry_path)

    # 6. Required array
    required = schema.get("required", [])
    if required is None:
        required = []
    if not isinstance(required, list) or any(not isinstance(field, str) for field in required):
        add_failure(failures, f"{tool.name} required fields must be a string array")
        add_check(checks, f"mcp_schema:required_array:{tool.name}", "fail", "required is not a string array", registry_path)
        return
    add_check(checks, f"mcp_schema:required_array:{tool.name}", "pass", "required is a string array", registry_path)

    # 7. Properties object
    properties = schema.get("properties", {})
    if properties is None:
        properties = {}
    if not isinstance(properties, dict):
        add_failure(failures, f"{tool.name} properties must be an object")
        add_check(checks, f"mcp_schema:properties_object:{tool.name}", "fail", "properties is not an object", registry_path)
        return
    add_check(checks, f"mcp_schema:properties_object:{tool.name}", "pass", "properties is an object", registry_path)

    # 8. Required fields present in properties
    missing_required = sorted(set(required) - set(properties))
    if missing_required:
        add_failure(failures, f"{tool.name} required fields missing from properties: {', '.join(missing_required)}")
        add_check(
            checks,
            f"mcp_schema:required_properties:{tool.name}",
            "fail",
            "required fields are missing from properties",
            registry_path,
        )
    else:
        add_check(
            checks,
            f"mcp_schema:required_properties:{tool.name}",
            "pass",
            "required fields are present in properties",
            registry_path,
        )


def validate_critical_required_fields(
    tool_by_name: dict[str, ToolDefParsed],
    checks: list[dict[str, Any]],
    failures: list[str],
    registry_path: Path,
) -> None:
    for name, required_fields in CRITICAL_REQUIRED_FIELDS.items():
        tool = tool_by_name.get(name)
        if tool is None or not isinstance(tool.schema, dict):
            continue
        required = set(tool.schema.get("required") or [])
        missing = sorted(set(required_fields) - required)
        if missing:
            add_failure(failures, f"{name} missing critical required fields: {', '.join(missing)}")
            add_check(checks, f"mcp_schema:critical_required:{name}", "fail", "critical required fields missing", registry_path)
        else:
            add_check(checks, f"mcp_schema:critical_required:{name}", "pass", "critical required fields exist", registry_path)

    bundle = tool_by_name.get("context_build_bundle")
    if bundle is None or not isinstance(bundle.schema, dict):
        return
    properties = bundle.schema.get("properties") or {}
    selectors = CONTEXT_BUNDLE_SELECTORS.intersection(properties)
    if selectors:
        add_check(
            checks,
            "mcp_schema:critical_selectors:context_build_bundle",
            "pass",
            "context_build_bundle exposes explicit selector fields",
            registry_path,
        )
    else:
        add_failure(failures, "context_build_bundle lacks explicit selector fields")
        add_check(
            checks,
            "mcp_schema:critical_selectors:context_build_bundle",
            "fail",
            "context_build_bundle lacks explicit selector fields",
            registry_path,
        )


def validate_annotations(
    tool_by_name: dict[str, ToolDefParsed],
    checks: list[dict[str, Any]],
    failures: list[str],
    registry_path: Path,
) -> None:
    for name in sorted(READ_ONLY_TOOLS):
        tool = tool_by_name.get(name)
        if tool is None:
            continue
        if has_annotation(tool, "readOnlyHint"):
            add_check(checks, f"mcp_annotation:read_only:{name}", "pass", "read-only annotation exists", registry_path)
        else:
            add_failure(failures, f"{name} missing readOnlyHint")
            add_check(checks, f"mcp_annotation:read_only:{name}", "fail", "readOnlyHint missing", registry_path)

    for name in sorted(DESTRUCTIVE_TOOLS):
        tool = tool_by_name.get(name)
        if tool is None:
            continue
        if has_annotation(tool, "destructiveHint"):
            add_check(checks, f"mcp_annotation:destructive:{name}", "pass", "destructive annotation exists", registry_path)
        else:
            add_failure(failures, f"{name} missing destructiveHint")
            add_check(checks, f"mcp_annotation:destructive:{name}", "fail", "destructiveHint missing", registry_path)

    for name in sorted(IDEMPOTENT_TOOLS):
        tool = tool_by_name.get(name)
        if tool is None:
            continue
        if has_annotation(tool, "idempotentHint"):
            add_check(checks, f"mcp_annotation:idempotent:{name}", "pass", "idempotent annotation exists", registry_path)
        else:
            add_failure(failures, f"{name} missing idempotentHint")
            add_check(checks, f"mcp_annotation:idempotent:{name}", "fail", "idempotentHint missing", registry_path)

    # 100% of registered tools: annotation consistency validation
    mutating_verbs = {"create", "delete", "update", "mutate", "insert", "drop", "destroy", "wipe"}
    for name, tool in sorted(tool_by_name.items()):
        is_ro = has_annotation(tool, "readOnlyHint")
        is_dest = has_annotation(tool, "destructiveHint")

        inconsistencies: list[str] = []

        if is_ro and is_dest:
            inconsistencies.append("cannot be both readOnly and destructive")

        name_parts = set(name.split("_"))
        if is_ro and (name_parts & mutating_verbs):
            inconsistencies.append("mutating action cannot possess readOnlyHint")

        if is_ro and name in ADMIN_TOOLS:
            inconsistencies.append("administrative tool cannot possess readOnlyHint")

        if (name.endswith("_delete") or name.endswith("_delete_batch")) and not is_dest:
            inconsistencies.append("destructive delete tool must possess destructiveHint")

        if inconsistencies:
            msg = f"{name} annotation inconsistency: {'; '.join(inconsistencies)}"
            add_failure(failures, msg)
            add_check(checks, f"mcp_annotation:consistency:{name}", "fail", msg, registry_path)
        else:
            add_check(checks, f"mcp_annotation:consistency:{name}", "pass", "annotation consistency verified", registry_path)


def degraded_mode_for(
    checks: list[dict[str, Any]],
    failures: list[str],
    warnings: list[str],
) -> str:
    if failures:
        if any(check["id"].endswith(":available") and check["status"] == "fail" for check in checks):
            return "unavailable"
        return "invalid"
    if any(check["status"] == "warn" and "optional" in check["id"] for check in checks):
        return "missing_optional"
    if warnings or any(check["status"] == "warn" for check in checks):
        return "degraded"
    return "ok"


def summary_for(status: str, tool_count: int, failures: list[str], warnings: list[str]) -> str:
    if status == "fail":
        return f"MCP contract validation failed with {len(failures)} failure(s)"
    if status == "warn":
        return f"MCP contract validation passed with {len(warnings)} warning(s)"
    return f"MCP contract validation passed for {tool_count} tool(s)"


def validate_contract(
    registry_path: Path = reference.DEFAULT_SOURCE,
    handlers_path: Path = ROOT / "src/mcp/handlers/mod.rs",
    docs_path: Path = reference.DEFAULT_OUTPUT,
) -> dict[str, Any]:
    warnings: list[str] = []
    failures: list[str] = []
    checks: list[dict[str, Any]] = []
    artifacts: list[dict[str, str]] = []

    tools: list[ToolDefParsed] = []
    tool_by_name: dict[str, ToolDefParsed] = {}
    dispatch_names: set[str] = set()
    doc_names: set[str] = set()

    if registry_path.exists():
        artifacts.append({"path": repo_path(registry_path), "kind": "mcp_registry", "format": "rust"})
        try:
            tools = parse_tools_tolerant(registry_path)
            tool_by_name = {tool.name: tool for tool in tools if tool.name}
            if not tools:
                add_failure(failures, f"no ToolDef entries found in {repo_path(registry_path)}")
                add_check(checks, "mcp_registry:parse", "fail", "no ToolDef entries found", registry_path)
            else:
                add_check(checks, "mcp_registry:parse", "pass", f"tool registry parsed ({len(tools)} tools)", registry_path)
        except Exception as exc:  # noqa: BLE001
            add_failure(failures, f"tool registry could not be parsed: {exc}")
            add_check(checks, "mcp_registry:parse", "fail", str(exc), registry_path)
    else:
        add_failure(failures, f"tool registry missing: {repo_path(registry_path)}")
        add_check(checks, "mcp_registry:available", "fail", "tool registry missing", registry_path)

    names = [tool.name for tool in tools if tool.name]
    duplicate_names = sorted({name for name in names if names.count(name) > 1})
    if duplicate_names:
        add_failure(failures, f"duplicate MCP tool names: {', '.join(duplicate_names)}")
        add_check(checks, "mcp_registry:unique_names", "fail", f"duplicate tool names found: {', '.join(duplicate_names)}", registry_path)
    elif tools:
        add_check(checks, "mcp_registry:unique_names", "pass", "all tool names are unique", registry_path)

    # Validate 100% of tools for schema correctness
    for tool in tools:
        validate_tool_schema(tool, checks, failures, registry_path)

    # Validate critical tools existence
    for name in CRITICAL_TOOLS:
        if name in tool_by_name:
            add_check(checks, f"mcp_tools_list:critical:{name}", "pass", "critical tool is registered", registry_path)
        else:
            add_failure(failures, f"critical tool missing from registry: {name}")
            add_check(checks, f"mcp_tools_list:critical:{name}", "fail", "critical tool missing", registry_path)

    validate_critical_required_fields(tool_by_name, checks, failures, registry_path)
    validate_annotations(tool_by_name, checks, failures, registry_path)

    if handlers_path.exists():
        artifacts.append({"path": repo_path(handlers_path), "kind": "mcp_dispatch", "format": "rust"})
        try:
            dispatch_names = parse_dispatch_names(handlers_path)
            add_check(checks, "mcp_dispatch:parse", "pass", f"handler dispatch parsed ({len(dispatch_names)} arms)", handlers_path)
        except Exception as exc:  # noqa: BLE001
            add_failure(failures, f"handler dispatch could not be parsed: {exc}")
            add_check(checks, "mcp_dispatch:parse", "fail", str(exc), handlers_path)
    else:
        add_failure(failures, f"handler dispatch missing: {repo_path(handlers_path)}")
        add_check(checks, "mcp_dispatch:available", "fail", "handler dispatch file missing", handlers_path)

    if tools and dispatch_names:
        registry_names = set(tool_by_name)
        missing_dispatch = sorted(registry_names - dispatch_names)
        unknown_dispatch = sorted(dispatch_names - registry_names - DISPATCH_ALIASES)

        # 100% dispatch alignment: check every registered tool
        for name in sorted(registry_names):
            if name in dispatch_names or name in DISPATCH_ALIASES:
                add_check(checks, f"mcp_dispatch:tool:{name}", "pass", "tool has dispatch arm", handlers_path)
            else:
                add_check(checks, f"mcp_dispatch:tool:{name}", "fail", "tool missing dispatch arm", handlers_path)

        if missing_dispatch:
            add_failure(failures, f"registered tools missing dispatch: {', '.join(missing_dispatch[:20])}")
            add_check(checks, "mcp_dispatch:registry_coverage", "fail", "some registered tools lack dispatch arms", handlers_path)
        else:
            add_check(checks, "mcp_dispatch:registry_coverage", "pass", "all registered tools have dispatch arms", handlers_path)

        if unknown_dispatch:
            add_failure(failures, f"dispatch arms missing registry entries: {', '.join(unknown_dispatch[:20])}")
            add_check(checks, "mcp_dispatch:unknown_arms", "fail", "unknown dispatch arms found", handlers_path)
        else:
            add_check(checks, "mcp_dispatch:unknown_arms", "pass", "dispatch arms match registry or aliases", handlers_path)

    if docs_path.exists():
        artifacts.append({"path": repo_path(docs_path), "kind": "mcp_reference", "format": "markdown"})
        docs_text = docs_path.read_text(encoding="utf-8")
        if GENERATED_MARKER in docs_text:
            add_check(checks, "mcp_docs:generated_marker", "pass", "generated marker exists", docs_path)
        else:
            add_failure(failures, "docs/MCP_TOOLS.md is missing generated marker")
            add_check(checks, "mcp_docs:generated_marker", "fail", "generated marker missing", docs_path)

        doc_names = parse_doc_names(docs_path)
        if tools:
            # 100% documentation alignment: check every registered tool
            for name in sorted(tool_by_name):
                if name in doc_names:
                    add_check(checks, f"mcp_docs:tool:{name}", "pass", "tool documented in reference", docs_path)
                else:
                    add_check(checks, f"mcp_docs:tool:{name}", "fail", "tool missing docs entry", docs_path)

            missing_docs = sorted(set(tool_by_name) - doc_names)
            if missing_docs:
                add_failure(failures, f"tools missing docs entries: {', '.join(missing_docs[:20])}")
                add_check(checks, "mcp_docs:tool_coverage", "fail", "some tools lack docs entries", docs_path)
            else:
                add_check(checks, "mcp_docs:tool_coverage", "pass", "all registered tools have docs entries", docs_path)

        if tools and default_docs_comparison_allowed(registry_path):
            valid_tools = [to_reference_tool(t) for t in tools if t.schema_error is None]
            generated = reference.render_reference(valid_tools, registry_path)
            if docs_text == generated:
                add_check(checks, "mcp_docs:generated_check", "pass", "generated reference is up to date", docs_path)
            else:
                add_failure(failures, "docs/MCP_TOOLS.md is stale; run ./scripts/generate-mcp-reference.sh")
                add_check(checks, "mcp_docs:generated_check", "fail", "generated reference is stale", docs_path)
    else:
        add_failure(failures, f"MCP docs missing: {repo_path(docs_path)}")
        add_check(checks, "mcp_docs:available", "fail", "MCP docs missing", docs_path)

    degraded_mode = degraded_mode_for(checks, failures, warnings)
    status = "fail" if failures else "warn" if warnings else "pass"
    exit_code = 1 if failures else 0

    return {
        "schema_version": SCHEMA_VERSION,
        "tool": TOOL_NAME,
        "mode": "offline",
        "status": status,
        "exit_code": exit_code,
        "timestamp": utc_now(),
        "summary": summary_for(status, len(tools), failures, warnings),
        "warnings": warnings,
        "failures": failures,
        "checks": checks,
        "artifacts": artifacts,
        "degraded_mode": degraded_mode,
        "counts": {
            "tools": len(tools),
            "dispatch_arms": len(dispatch_names),
            "docs_entries": len(doc_names),
            "checks": len(checks),
            "warnings": len(warnings),
            "failures": len(failures),
        },
    }


def print_human(result: dict[str, Any]) -> None:
    print(result["summary"])
    for failure in result["failures"]:
        print(f"FAIL: {failure}")
    for warning in result["warnings"]:
        print(f"WARN: {warning}")


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate static MCP tool contracts")
    parser.add_argument("--json", action="store_true", help="emit canonical harness JSON envelope")
    parser.add_argument("--registry", type=Path, default=reference.DEFAULT_SOURCE)
    parser.add_argument("--handlers", type=Path, default=ROOT / "src/mcp/handlers/mod.rs")
    parser.add_argument("--docs", type=Path, default=reference.DEFAULT_OUTPUT)
    args = parser.parse_args()

    result = validate_contract(args.registry, args.handlers, args.docs)
    if args.json:
        print(json.dumps(result, ensure_ascii=False))
    else:
        print_human(result)
    return int(result["exit_code"])


if __name__ == "__main__":
    raise SystemExit(main())

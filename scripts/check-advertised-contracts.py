#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# ///
# ─── How to run ───
# rtk python3 scripts/check-advertised-contracts.py --inventory docs/contracts/advertised-surfaces.toml
from __future__ import annotations

import argparse
import ast
import hashlib
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import generate_mcp_reference as mcp_reference
import validate_mcp_contract

PUBLIC_DOCS = "README.md docs/MCP_TOOLS.md docs/MCP_AUTH.md docs/AI_GUIDE.md docs/USER_GUIDE.md docs/GETTING_STARTED.md docs/REFERENCE.md examples/README.md".split()
CHANNELS = {
    "MCP stdio": r"MCP stdio|transport stdio|stdio transport",
    "HTTP MCP": r"HTTP MCP|HTTP JSON-RPC|POST /mcp|POST /v1/mcp",
    "WebSocket events": r"WebSocket|ENGRAM_WS_PORT",
    "gRPC": r"gRPC|--transport grpc|ENGRAM_GRPC_API_KEY",
    "CLI": r"engram-cli|CLI / SDKs",
    "Python SDK": r"Python SDK|Python and TypeScript SDKs",
    "TypeScript SDK": r"TypeScript SDK|Python and TypeScript SDKs",
}


def main() -> int:
    parser = argparse.ArgumentParser(description="Check Engram advertised contract inventory.")
    parser.add_argument("--inventory", type=Path, required=True)
    for name in "missing-surface tier-drift cli-drift doc-drift image-get-query-drift image-delete-query-drift".split():
        parser.add_argument(f"--self-test-{name}", action="store_true")
    args = parser.parse_args()
    try:
        inventory = load_inventory(args.inventory)
        current = current_inventory()
    except (FileNotFoundError, tomllib.TOMLDecodeError, ValueError) as exc:
        print(f"advertised-contracts: invalid input: {exc}", file=sys.stderr)
        return 2
    for flag, mutator in ((args.self_test_missing_surface, missing_surface), (args.self_test_tier_drift, tier_drift), (args.self_test_cli_drift, cli_drift), (args.self_test_doc_drift, doc_drift), (args.self_test_image_get_query_drift, lambda item: image_query_drift(item, "GET")), (args.self_test_image_delete_query_drift, lambda item: image_query_drift(item, "DELETE"))):
        if flag:
            current = mutator(current)
    failures = compare_inventory(inventory, current)
    if failures:
        print("advertised-contracts: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print("advertised-contracts: PASS")
    print(
        "checked "
        f"{inventory['mcp']['total_tool_count']} MCP tools, "
        f"{inventory['mcp']['tools_list_default_tool_count']} default tools/list tools, "
        f"{len(inventory['sdk_python']['engram_client_methods'])} Python SDK methods, "
        f"{len(inventory['sdk_typescript']['engram_client_methods'])} TypeScript SDK methods, "
        f"{inventory['rust_public_api']['line_count']} Rust API baseline lines"
    )
    return 0


def load_inventory(path: Path):
    data = tomllib.loads(path.read_text(encoding="utf-8"))
    if data.get("schema_version") != "advertised-surfaces-v1":
        raise ValueError("schema_version must be advertised-surfaces-v1")
    for section in "sources mcp cli cargo sdk_python sdk_typescript docs_contract rust_public_api".split():
        if section not in data:
            raise ValueError(f"missing [{section}] section")
    return data


def current_inventory():
    tools = mcp_reference.parse_tools(ROOT / "src/mcp/tools/registry.rs")
    cargo = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    rust_api = ROOT / "docs/api/types-baseline.txt"
    default_transports, gated_transports = server_transports()
    validate_generated_mcp_reference(tools)
    validate_mcp_runtime_contract()
    return {
        "sources": {"public_docs": PUBLIC_DOCS, "examples": examples()},
        "mcp": mcp_inventory(tools),
        "cli": {
            "binaries": [item["name"] for item in cargo["bin"]],
            "engram_cli_commands": cli_commands(False),
            "feature_gated_commands": cli_commands(True),
            "feature_gated_command_features": cli_command_features(),
            "engram_server_default_transports": default_transports,
            "engram_server_feature_transports": gated_transports,
            "http_auth_env": "ENGRAM_HTTP_API_KEY",
        },
        "cargo": {"features": list(cargo["features"].keys())},
        "sdk_python": {"exports": python_exports(), "engram_client_methods": python_methods()},
        "sdk_typescript": {
            "exported_classes": ts_exports("class"),
            "exported_interfaces": ts_exports("interface"),
            "engram_client_methods": ts_methods("EngramClient"),
            "council_skill_methods": ts_methods("CouncilSkill"),
        },
        "docs_contract": docs_contract(),
        "rust_public_api": {
            "line_count": len(rust_api.read_text(encoding="utf-8").splitlines()),
            "sha256": hashlib.sha256(rust_api.read_bytes()).hexdigest(),
        },
    }


def mcp_inventory(tools: list[mcp_reference.Tool]):
    available = [tool for tool in tools if not tool.required_features]
    default = [tool.name for tool in available if tool.tier == "essential" or tool.name == "discover_tools"]
    standard = [tool.name for tool in available if tool.tier in {"essential", "standard"} or tool.name == "discover_tools"]
    all_available = [tool.name for tool in available]
    feature_names = sorted({feature for tool in tools for feature in tool.required_features})
    return {
        "total_tool_count": len(tools),
        "feature_available_tool_count": len(all_available),
        "tools_list_default_tool_count": len(default),
        "tools_list_standard_tool_count": len(standard),
        "tools_list_all_tool_count": len(all_available),
        "tools": [tool.name for tool in tools],
        "feature_available_tools": all_available,
        "feature_gated_tools": [tool.name for tool in tools if tool.required_features],
        "required_features": feature_names,
        "required_feature_tools": [f"{feature}=" + ",".join(tool.name for tool in tools if feature in tool.required_features) for feature in feature_names],
        "tools_list_default_tools": default,
        "tools_list_standard_tools": standard,
        "tools_list_all_tools": all_available,
    }


def docs_contract():
    text = "\n".join((ROOT / path).read_text(encoding="utf-8") for path in PUBLIC_DOCS)
    features = set(re.findall(r"--features\s+([a-z0-9_, -]+)", text)); cargo_features = set(tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))["features"])
    split_features = {part for item in features for part in re.split(r"[ ,]+", item) if re.fullmatch(r"[a-z][a-z0-9-]*", part) and part not in {"cargo", "release"}}
    table_features = {feature for feature in cargo_features if re.search(rf"(?<![A-Za-z0-9_-]){re.escape(feature)}(?![A-Za-z0-9_-])", text)}
    endpoints = sorted({f"{verb} {path}" for verb, path in re.findall(r"\b(GET|POST|PATCH|DELETE)\s+(/[-A-Za-z0-9_/:]+(?:\?[A-Za-z0-9_]+=\.\.\.)?)", text)}); env_vars = sorted(set(re.findall(r"\b(ENGRAM_[A-Z0-9_]+|OPENAI_API_KEY|MEILISEARCH_[A-Z0-9_]+|R2_[A-Z0-9_]+|AWS_[A-Z0-9_]+|LANGFUSE_[A-Z0-9_]+|VISION_PROVIDER)\b", text)))
    cli_examples = sorted(set(re.findall(r"\bengram-cli [^`\n]+", text))); tier_values = sorted(set(re.findall(r"ENGRAM_TOOL_TIER=(essential|standard|advanced|all)", text)))
    return {
        "channels": [name for name, pattern in CHANNELS.items() if re.search(pattern, text, re.I)],
        "http_endpoints": endpoints,
        "env_vars": env_vars,
        "feature_promises": sorted((split_features | table_features) & cargo_features),
        "tool_tier_values": tier_values,
        "cli_examples": cli_examples,
    }


def examples() -> list[str]:
    return [str(path.relative_to(ROOT)) for path in sorted((ROOT / "examples").iterdir()) if path.is_dir()]


def validate_generated_mcp_reference(tools: list[mcp_reference.Tool]) -> None:
    docs = ROOT / "docs/MCP_TOOLS.md"
    generated = mcp_reference.render_reference(tools, ROOT / "src/mcp/tools/registry.rs")
    if docs.read_text(encoding="utf-8") != generated:
        raise ValueError("docs/MCP_TOOLS.md is stale; run ./scripts/generate-mcp-reference.sh")


def validate_mcp_runtime_contract() -> None:
    report = validate_mcp_contract.validate_contract()
    if report["exit_code"] != 0:
        raise ValueError("MCP runtime contract drift: " + "; ".join(report.get("failures", [])))


def python_exports() -> list[str]:
    text = (ROOT / "sdks/python/engram_client/__init__.py").read_text(encoding="utf-8")
    match = re.search(r"__all__\s*=\s*\[(?P<items>[^\]]+)\]", text)
    if not match:
        raise ValueError("Python SDK __all__ is missing")
    return re.findall(r'"([^"]+)"', match.group("items"))


def python_methods() -> list[str]:
    tree = ast.parse((ROOT / "sdks/python/engram_client/client.py").read_text(encoding="utf-8"))
    for node in tree.body:
        if isinstance(node, ast.ClassDef) and node.name == "EngramClient":
            return [child.name for child in node.body if isinstance(child, (ast.FunctionDef, ast.AsyncFunctionDef)) and not child.name.startswith("_")]
    raise ValueError("Python EngramClient class is missing")


def ts_exports(kind: str) -> list[str]:
    items: list[str] = []
    for path in (ROOT / "sdks/typescript/src").glob("*.ts"):
        text = path.read_text(encoding="utf-8")
        items.extend(re.findall(rf"^export\s+{kind}\s+([A-Za-z_$][\w$]*)", text, re.MULTILINE))
    return sorted(items)


def find_ts_class_file(class_name: str) -> tuple[Path, str]:
    for path in (ROOT / "sdks/typescript/src").glob("*.ts"):
        text = path.read_text(encoding="utf-8")
        if re.search(rf"\bclass\s+{class_name}\b", text):
            return path, text
    raise ValueError(f"TypeScript class {class_name} is missing")


def ts_methods(class_name: str) -> list[str]:
    _, text = find_ts_class_file(class_name)
    body = ts_class_body(text, class_name)
    methods: list[str] = []
    depth = 0
    for line in body.splitlines():
        match = re.match(r"^(?:async\s+)?([A-Za-z_$][\w$]*)\s*\(", line.strip()) if depth == 0 else None
        if match and match.group(1) != "constructor" and not match.group(1).startswith("_"):
            methods.append(match.group(1))
        depth = max(0, depth + line.count("{") - line.count("}"))
    return methods


def ts_class_body(text: str, class_name: str) -> str:
    start = text.index(f"class {class_name}")
    brace = text.index("{", start)
    depth = 0
    quote = ""
    escaped = False
    for index in range(brace, len(text)):
        char = text[index]
        if quote:
            escaped, quote = (False, quote) if escaped else (char == "\\", "" if char == quote else quote)
            continue
        if char in {"'", '"', "`"}:
            quote = char
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return text[brace + 1 : index]
    raise ValueError(f"TypeScript class {class_name} is unterminated")



def cli_command_entries():
    body = (ROOT / "src/bin/cli/args.rs").read_text(encoding="utf-8").split("pub(crate) enum Commands {", 1)[1].rsplit("\n}", 1)[0]
    return [(feature, kebab(name)) for feature, name in re.findall(r'(?:#\[cfg\(feature = "([^"]+)"\)\]\s*)?(?:\s*///[^\n]+\n)*\s{4}([A-Z][A-Za-z0-9_]*)\b', body)]


def cli_commands(gated: bool) -> list[str]:
    names = [name for feature, name in cli_command_entries() if bool(feature) is gated]
    return names if gated else names + ["help"]


def cli_command_features() -> list[str]:
    return [f"{name}={feature}" for feature, name in cli_command_entries() if feature]


def server_transports() -> tuple[list[str], list[str]]:
    body = (ROOT / "src/bin/server.rs").read_text(encoding="utf-8").split("enum TransportMode {", 1)[1].split("\n}", 1)[0]
    values = re.findall(r'^\s{4}(?:#\[cfg\(feature = "([^"]+)"\)\]\s*)?([A-Z][A-Za-z0-9_]*)\b', body, re.MULTILINE)
    default = [kebab(name) for feature, name in values if not feature]
    return default, [kebab(name) for feature, name in values if feature]


def kebab(name: str) -> str:
    return re.sub(r"(?<!^)([A-Z])", r"-\1", name).lower()


def compare_inventory(expected, current) -> list[str]:
    failures: list[str] = []
    for path in "mcp.total_tool_count mcp.feature_available_tool_count mcp.tools_list_default_tool_count mcp.tools_list_standard_tool_count mcp.tools_list_all_tool_count cli.http_auth_env rust_public_api.line_count rust_public_api.sha256".split():
        section, key = path.split(".")
        if expected[section][key] != current[section][key]:
            failures.append(f"{path} expected {expected[section][key]!r}, got {current[section][key]!r}")
    list_paths = "sources.public_docs|public docs;sources.examples|examples;mcp.tools|MCP tools;mcp.feature_available_tools|feature-available MCP tools;mcp.feature_gated_tools|feature-gated MCP tools;mcp.required_features|MCP required features;mcp.required_feature_tools|MCP required feature tools;mcp.tools_list_default_tools|default tools/list MCP tools;mcp.tools_list_standard_tools|standard tools/list MCP tools;mcp.tools_list_all_tools|all tools/list MCP tools;cli.binaries|binaries;cli.engram_cli_commands|engram-cli commands;cli.feature_gated_commands|feature-gated CLI commands;cli.feature_gated_command_features|feature-gated CLI command features;cli.engram_server_default_transports|server default transports;cli.engram_server_feature_transports|server feature transports;cargo.features|Cargo features;sdk_python.exports|Python SDK exports;sdk_python.engram_client_methods|Python SDK methods;sdk_typescript.exported_classes|TypeScript exported classes;sdk_typescript.exported_interfaces|TypeScript exported interfaces;sdk_typescript.engram_client_methods|TypeScript SDK methods;sdk_typescript.council_skill_methods|CouncilSkill methods;docs_contract.channels|doc channels;docs_contract.http_endpoints|doc endpoints;docs_contract.env_vars|doc env vars;docs_contract.feature_promises|doc feature promises;docs_contract.tool_tier_values|doc tool-tier values;docs_contract.cli_examples|doc CLI examples".split(";")
    for item in list_paths:
        path, label = item.split("|", 1)
        section, key = path.split(".")
        failures.extend(diff_messages(label, expected[section][key], current[section][key]))
    return failures


def diff_messages(label: str, expected: list[str], current: list[str]) -> list[str]:
    missing = sorted(set(expected) - set(current))
    extra = sorted(set(current) - set(expected))
    return ([f"{label} missing at runtime: {', '.join(missing[:20])}"] if missing else []) + ([f"{label} unrecorded additions: {', '.join(extra[:20])}"] if extra else [])


def clone(current):
    return {section: dict(values) for section, values in current.items()}


def missing_surface(current):
    mutated = clone(current)
    mutated["mcp"]["tools"] = [name for name in current["mcp"]["tools"] if name != "memory_create"]
    mutated["sdk_python"]["engram_client_methods"] = [name for name in current["sdk_python"]["engram_client_methods"] if name != "create"]
    mutated["cargo"]["features"] = [name for name in current["cargo"]["features"] if name != "default"]
    return mutated


def tier_drift(current):
    mutated = clone(current)
    mutated["mcp"]["tools_list_default_tools"] = current["mcp"]["feature_available_tools"]
    mutated["mcp"]["tools_list_default_tool_count"] = len(mutated["mcp"]["tools_list_default_tools"])
    return mutated


def cli_drift(current):
    mutated = clone(current)
    mutated["cli"]["feature_gated_commands"] = [name for name in current["cli"]["feature_gated_commands"] if name != "model"]
    mutated["cli"]["feature_gated_command_features"] = [item for item in current["cli"]["feature_gated_command_features"] if not item.startswith("model=")]
    return mutated


def doc_drift(current):
    mutated = clone(current)
    mutated["mcp"]["required_features"] = [item for item in current["mcp"]["required_features"] if item not in {"langfuse", "duckdb-graph", "emergent-graph"}]
    mutated["mcp"]["required_feature_tools"] = [item for item in current["mcp"]["required_feature_tools"] if not item.startswith(("langfuse=", "duckdb-graph=", "emergent-graph="))]
    mutated["docs_contract"]["channels"] = [item for item in current["docs_contract"]["channels"] if item != "HTTP MCP"]; mutated["docs_contract"]["http_endpoints"] = [item for item in current["docs_contract"]["http_endpoints"] if item != "POST /mcp"]
    mutated["docs_contract"]["env_vars"] = [item for item in current["docs_contract"]["env_vars"] if item not in {"ENGRAM_TOOL_TIER", "R2_ACCESS_KEY_ID", "R2_SECRET_ACCESS_KEY", "AWS_ENDPOINT_URL", "AWS_PROFILE", "LANGFUSE_PUBLIC_KEY", "LANGFUSE_SECRET_KEY", "VISION_PROVIDER"}]
    mutated["docs_contract"]["feature_promises"] = [item for item in current["docs_contract"]["feature_promises"] if item not in {"grpc", "langfuse", "duckdb-graph", "emergent-graph"}]
    return mutated


def image_query_drift(current, method: str):
    mutated = clone(current); old = f"{method} /v1/images?memory_id=..."; new = f"{method} /v1/images?id=..."
    mutated["docs_contract"]["http_endpoints"] = [new if item == old else item for item in current["docs_contract"]["http_endpoints"]]
    return mutated


if __name__ == "__main__":
    raise SystemExit(main())

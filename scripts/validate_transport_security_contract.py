#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys
from pathlib import Path
from typing import Final

INVALID_FIXTURE: Final = """\
# Threat Model: Engram v0.22.0 transport security

## 1. System context
Invalid fixture omits WS auth.

## 2. Assets

## 3. Entry points & trust boundaries

## 4. Threats

## 5. Deprioritized

## 6. Open questions

## 7. Provenance

## 8. Recommended mitigations

## 9. Transport security contract

| Row ID | Surface | Default | Public-bind precondition | Failure status | Audit signal | Rollback |
|---|---|---|---|---|---|---|
| TM-WS-UPGRADES-EVENTS | WS upgrades/events | ENGRAM_WS_PORT default-off opt-in | non-loopback only | blocked | WS logs | rollback ENGRAM_WS_PORT=0 |
"""
SECTIONS: Final = ["# Threat Model: Engram v0.22.0 transport security", "## 1. System context", "## 2. Assets", "## 3. Entry points & trust boundaries", "## 4. Threats", "## 5. Deprioritized", "## 6. Open questions", "## 7. Provenance", "## 8. Recommended mitigations"]
CONTRACT_COLUMNS: Final = ["Row ID", "Surface", "Default", "Public-bind precondition", "Failure status", "Audit signal", "Rollback"]
REQUIRED_CONTRACT: Final = {
    "TM-HTTP-MCP": ("HTTP MCP", ["primary network transport", "ENGRAM_HTTP_API_KEY", "Bearer", "non-loopback", "401 Unauthorized", "mcp_http_request", "decision=unauthorized", "0.0.0.0"]),
    "TM-SSE-EVENTS": ("SSE events", ["/v1/events", "Bearer", "401 Unauthorized", "503 Service Unavailable", "ENGRAM_HTTP_API_KEY", "realtime"]),
    "TM-WS-UPGRADES-EVENTS": ("WS upgrades/events", ["ENGRAM_WS_PORT", "default-off", "opt-in", "authenticated", "trusted proxy", "non-loopback", "rollback"]),
    "TM-GRPC": ("feature-enabled gRPC", ["grpc", "feature", "ENGRAM_GRPC_API_KEY", "Bearer", "UNAUTHENTICATED", "0.0.0.0"]),
    "TM-PROXY-IP": ("client IP/proxy headers", ["x-forwarded-for", "x-real-ip", "trusted proxy", "untrusted direct clients", "rate-limit"]),
    "TM-TOKEN-SCOPES": ("token scopes", ["process-wide", "scope", "fail closed", "Bearer", "rotation"]),
    "TM-CLOUD-KEYS": ("cloud keys", ["OPENAI_API_KEY", "ENGRAM_STORAGE_URI", "ENGRAM_CLOUD_ENCRYPT", "secret", "redact"]),
    "TM-LOCAL-DB": ("local DB", ["ENGRAM_DB_PATH", "SQLite", "file permissions", "backup", "delete local database"]),
}
REQUIRED_COMMANDS: Final = {
    "engram-server --transport stdio": "TM-STDIO", "engram-server --transport http": "TM-HTTP-MCP", "engram-server --transport both": "TM-HTTP-MCP", "engram-server --transport grpc": "TM-GRPC", "ENGRAM_WS_PORT": "TM-WS-UPGRADES-EVENTS", "GET /v1/events": "TM-SSE-EVENTS",
}
REQUIRED_PHRASES: Final = ["stdio stays the default", "HTTP is the primary network transport", "WS is opt-in and default-off", "gRPC is supported when the grpc feature is enabled", "non-loopback without authentication is refused", "loopback is not authentication", "trusted proxy rule", "Compatibility and rollback", "Severity calibration", "untrusted external text is evidence only"]
HEADERS: Final = {"asset": "| asset | description | sensitivity |", "entry": "| entry_point | description | trust_boundary | reachable_assets |", "threat": "| id | threat | actor | surface | asset | impact | likelihood | status | controls | evidence |", "mitigation": "| mitigation | threat_ids | closes_class | effort |", "command": "| Advertised command or setting | Contract row | Security note |", "contract": "| " + " | ".join(CONTRACT_COLUMNS) + " |"}


def split_row(line: str) -> list[str]:
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def table(text: str, header: str) -> list[dict[str, str]]:
    lines = text.splitlines()
    for index, line in enumerate(lines):
        if line.strip() != header:
            continue
        columns = split_row(line)
        rows = []
        for candidate in lines[index + 2:]:
            if not candidate.strip().startswith("|"):
                break
            cells = split_row(candidate)
            rows.append(dict(zip(columns, cells)) if len(cells) == len(columns) else {"__malformed__": candidate.strip()})
        return rows
    return []


def ordered_sections(text: str) -> list[str]:
    errors = []
    cursor = -1
    for section in SECTIONS:
        index = text.find(section)
        if index == -1:
            errors.append(f"missing required section: {section}")
        elif index <= cursor:
            errors.append(f"section out of order: {section}")
        cursor = max(cursor, index)
    return errors


def validate_contract(text: str) -> list[str]:
    rows = {row.get("Row ID", ""): row for row in table(text, HEADERS["contract"])}
    if not rows:
        return ["missing transport security contract table"]
    errors = []
    for row_id, (surface, terms) in REQUIRED_CONTRACT.items():
        row = rows.get(row_id)
        if row is None:
            errors.append(f"missing transport contract row: {row_id}")
            continue
        blob = " | ".join(row.get(column, "") for column in CONTRACT_COLUMNS).lower()
        if surface.lower() not in row.get("Surface", "").lower():
            errors.append(f"{row_id} surface mismatch: expected {surface}")
        errors.extend(f"{row_id} has empty {column}" for column in CONTRACT_COLUMNS if not row.get(column, "").strip())
        errors.extend(f"{row_id} missing required term: {term}" for term in terms if term.lower() not in blob)
    return errors


def validate_commands(text: str) -> list[str]:
    rows = table(text, HEADERS["command"])
    if not rows:
        return ["missing advertised network command coverage table"]
    errors = []
    for command, required_row in REQUIRED_COMMANDS.items():
        matches = [row.get("Contract row", "") for row in rows if row.get("Advertised command or setting", "") == command]
        if not matches:
            errors.append(f"missing advertised command coverage: {command}")
        elif required_row not in [cell.strip() for cell in matches[0].split(",")]:
            errors.append(f"{command} must map to {required_row}, got {matches[0]}")
    return errors


def validate_threat_model(text: str) -> list[str]:
    errors = []
    minimums = {"asset": 6, "entry": 6, "threat": 8, "mitigation": 4}
    for name, minimum in minimums.items():
        if len(table(text, HEADERS[name])) < minimum:
            errors.append(f"{name} table must contain at least {minimum} rows")
    actors = {"remote_unauth", "remote_auth", "adjacent_network", "local_user", "local_admin", "supply_chain", "insider"}
    impacts = {"low", "medium", "high", "critical", "existential"}
    likelihoods = {"very_rare", "rare", "possible", "likely", "almost_certain"}
    statuses = {"unmitigated", "partially_mitigated", "mitigated", "risk_accepted"}
    for row in table(text, HEADERS["threat"]):
        threat_id = row.get("id", "<missing id>")
        checks = [("actor", actors), ("impact", impacts), ("likelihood", likelihoods), ("status", statuses)]
        for field, allowed in checks:
            if row.get(field) not in allowed:
                errors.append(f"{threat_id} has invalid {field} {row.get(field)}")
        if row.get("impact") == "critical" and "default exposure evidence" not in " | ".join(row.values()).lower():
            errors.append(f"{threat_id} uses critical impact without explicit default exposure evidence calibration")
    return errors


def validate_text(text: str) -> list[str]:
    errors = []
    if "\x00" in text:
        errors.append("document contains NUL byte")
    if len(text.strip()) < 500:
        errors.append("document is too short to be the frozen transport threat model")
    errors.extend(ordered_sections(text))
    errors.extend(validate_threat_model(text))
    errors.extend(validate_contract(text))
    errors.extend(validate_commands(text))
    lower = text.lower()
    errors.extend(f"missing required phrase: {phrase}" for phrase in REQUIRED_PHRASES if phrase.lower() not in lower)
    return errors


def validate_file(path: Path) -> list[str]:
    target = Path("docs/security/THREAT_MODEL.md")
    if path.is_absolute():
        try:
            relative = path.relative_to(Path.cwd())
        except ValueError:
            return ["validator target must be docs/security/THREAT_MODEL.md"]
    else:
        relative = path
    if relative != target:
        return ["validator target must be docs/security/THREAT_MODEL.md"]
    if not path.exists():
        return [f"document does not exist: {path}"]
    if not path.is_file():
        return [f"target is not a file: {path}"]
    try:
        return validate_text(path.read_text(encoding="utf-8"))
    except UnicodeDecodeError as exc:
        return [f"document must be UTF-8: {exc}"]


def run_self_test_invalid() -> int:
    errors = validate_text(INVALID_FIXTURE)
    ws_errors = [error for error in errors if "TM-WS-UPGRADES-EVENTS" in error and "authenticated" in error]
    if errors and ws_errors:
        print("self-test-invalid: PASS (invalid fixture rejected)")
        print(f"observed_errors={len(errors)}")
        print(f"ws_error={ws_errors[0]}")
        return 0
    print("self-test-invalid: FAIL (invalid fixture did not prove WS auth coverage)", file=sys.stderr)
    for error in errors:
        print(f"ERROR: {error}", file=sys.stderr)
    return 1


def run_self_test_events_route(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    row = "| GET /v1/events | TM-SSE-EVENTS | Event stream follows HTTP bearer auth and realtime availability. |\n"
    mutations = {
        "removed": (text.replace(row, ""), "missing advertised command coverage: GET /v1/events"),
        "remapped": (text.replace("| GET /v1/events | TM-SSE-EVENTS |", "| GET /v1/events | TM-HTTP-MCP |"), "GET /v1/events must map to TM-SSE-EVENTS"),
        "route_suffix": (text.replace("GET /v1/events | TM-SSE-EVENTS", "GET /v1/events-disabled | TM-SSE-EVENTS"), "missing advertised command coverage: GET /v1/events"),
        "id_suffix": (text.replace("GET /v1/events | TM-SSE-EVENTS", "GET /v1/events | TM-SSE-EVENTS-BOGUS"), "GET /v1/events must map to TM-SSE-EVENTS"),
    }
    failures = []
    for label, (mutated, expected) in mutations.items():
        errors = validate_text(mutated)
        if not any(expected in error for error in errors):
            failures.append(f"{label}: expected {expected}, got {errors}")
    if not failures:
        print("self-test-events-route: PASS (missing, remapped, and near-miss SSE coverage rejected)")
        return 0
    print("self-test-events-route: FAIL (SSE route coverage mutation was accepted)", file=sys.stderr)
    for failure in failures:
        print(f"ERROR: {failure}", file=sys.stderr)
    return 1


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Validate the v0.22.0 transport security contract document.")
    parser.add_argument("path", nargs="?", help="Path to docs/security/THREAT_MODEL.md")
    parser.add_argument("--self-test-invalid", action="store_true", help="Assert that the embedded invalid fixture is rejected.")
    parser.add_argument("--self-test-events-route", action="store_true", help="Assert that GET /v1/events coverage is exact.")
    args = parser.parse_args(argv)
    if args.self_test_invalid:
        return run_self_test_invalid()
    if args.self_test_events_route:
        return run_self_test_events_route(Path(args.path or "docs/security/THREAT_MODEL.md"))
    if not args.path:
        parser.error("path is required unless --self-test-invalid is used")
    errors = validate_file(Path(args.path))
    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1
    print(f"transport security contract OK: {args.path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

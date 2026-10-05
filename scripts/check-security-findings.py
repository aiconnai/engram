#!/usr/bin/env python3
"""Mechanical, fail-closed findings policy for one security scanner report (task Q5).

Three things are kept separate on purpose:

* scanner execution   - did the scanner run and exit cleanly (``--scanner-exit``)?
* report evidence     - is the SARIF present, well formed, from the expected tool and for the
                        expected commit (``--sarif``, ``--tool``, ``--expected-sha``)?
* findings policy     - is any high finding left after governed exceptions?

Identity (scanner name, tool name, expected commit SHA, evaluation date, whether a skip was
authorized) is supplied by the supervisor on the command line. Nothing inside the SARIF payload
can grant itself authority: embedded suppressions are ignored and the embedded revision is only
compared against the supervisor-provided SHA. A successful SARIF *upload* or a scanner exit code
of 0 never proves the absence of findings.

Verdicts: ``pass`` (exit 0), ``neutral`` (exit 0, explicit allowed skip only, never reported as
PASS) and ``block`` (exit 1). Usage errors exit 2.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from datetime import date, timedelta
from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - python < 3.11
    import tomli as tomllib  # type: ignore[no-redef]

SHA_RE = re.compile(r"^[0-9a-f]{40}$")
HIGH_SECURITY_SEVERITY = 7.0
MAX_EXCEPTION_HORIZON_DAYS = 90
NOT_RUN_STATES = ("not-run", "skipped")
EXCEPTION_FIELDS = ("scanner", "rule_id", "owner", "approved_by", "rationale")


class UsageError(RuntimeError):
    """Invalid supervisor-provided arguments (exit 2)."""


@dataclass
class Decision:
    verdict: str
    reasons: list[str] = field(default_factory=list)
    high_findings: int = 0
    excepted_findings: int = 0

    def block(self, reason: str) -> "Decision":
        self.verdict = "block"
        self.reasons.append(reason)
        return self


@dataclass(frozen=True)
class FindingException:
    scanner: str
    rule_id: str
    path: str | None
    owner: str
    approved_by: str
    expires: date
    rationale: str


def load_exceptions(path: Path | None, today: date) -> list[FindingException]:
    """Parse governed finding exceptions; any invalid record invalidates the whole file."""

    if path is None:
        return []
    try:
        with path.open("rb") as handle:
            data = tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise ValueError(f"exceptions file unreadable: {exc}") from exc
    records = data.get("exceptions", [])
    if not isinstance(records, list):
        raise ValueError("exceptions must be an array of tables")
    parsed: list[FindingException] = []
    for index, record in enumerate(records):
        if not isinstance(record, dict):
            raise ValueError(f"exception #{index + 1} is not a table")
        for key in EXCEPTION_FIELDS:
            value = record.get(key)
            if not isinstance(value, str) or not value.strip():
                raise ValueError(f"exception #{index + 1} lacks non-empty {key}")
        expires = record.get("expires")
        if not isinstance(expires, date):
            raise ValueError(f"exception #{index + 1} lacks an expires date")
        if expires > today + timedelta(days=MAX_EXCEPTION_HORIZON_DAYS):
            raise ValueError(
                f"exception #{index + 1} expires more than {MAX_EXCEPTION_HORIZON_DAYS} days ahead"
            )
        path_scope = record.get("path")
        if path_scope is not None and (not isinstance(path_scope, str) or not path_scope.strip()):
            raise ValueError(f"exception #{index + 1} has an empty path")
        parsed.append(
            FindingException(
                scanner=record["scanner"].strip().lower(),
                rule_id=record["rule_id"].strip(),
                path=path_scope,
                owner=record["owner"].strip(),
                approved_by=record["approved_by"].strip(),
                expires=expires,
                rationale=record["rationale"].strip(),
            )
        )
    return parsed


def parse_sarif(path: Path) -> dict[str, Any]:
    """Read and structurally validate SARIF 2.1.0; raise ValueError when untrustworthy."""

    try:
        text = path.read_text()
    except OSError as exc:
        raise FileNotFoundError(f"SARIF missing or unreadable: {path}") from exc
    try:
        document = json.loads(text)
    except json.JSONDecodeError as exc:
        raise ValueError(f"SARIF is not valid JSON: {exc}") from exc
    if not isinstance(document, dict):
        raise ValueError("SARIF root must be an object")
    if document.get("version") != "2.1.0":
        raise ValueError(f"unsupported SARIF version: {document.get('version')!r}")
    runs = document.get("runs")
    if not isinstance(runs, list) or not runs:
        raise ValueError("SARIF has no runs: no evidence that a scan happened")
    for index, run in enumerate(runs):
        label = f"run #{index + 1}"
        if not isinstance(run, dict):
            raise ValueError(f"{label} is not an object")
        driver = (run.get("tool") or {}).get("driver") if isinstance(run.get("tool"), dict) else None
        if not isinstance(driver, dict) or not isinstance(driver.get("name"), str):
            raise ValueError(f"{label} lacks tool.driver.name")
        if not isinstance(run.get("results"), list):
            raise ValueError(f"{label} lacks a results array")
        if not all(isinstance(item, dict) for item in run["results"]):
            raise ValueError(f"{label} has a non-object result")
        rules = driver.get("rules", [])
        if not isinstance(rules, list):
            raise ValueError(f"{label} rules must be an array")
    return document


def run_revisions(run: dict[str, Any]) -> list[str]:
    provenance = run.get("versionControlProvenance")
    if not isinstance(provenance, list):
        return []
    return [
        item["revisionId"]
        for item in provenance
        if isinstance(item, dict) and isinstance(item.get("revisionId"), str)
    ]


def check_identity(document: dict[str, Any], tool: str, expected_sha: str, decision: Decision) -> None:
    for index, run in enumerate(document["runs"], start=1):
        name = run["tool"]["driver"]["name"]
        if tool.lower() not in name.lower():
            decision.block(f"run #{index} is from tool {name!r}, expected {tool!r}")
        revisions = run_revisions(run)
        if not revisions:
            decision.block(
                f"run #{index} carries no revision provenance: report identity cannot be verified "
                "(fix hint: inspect the retained artifact, e.g. codeql-security-sarif, and check "
                "runs[].versionControlProvenance[].revisionId; the scanner must record the commit)"
            )
        elif any(revision != expected_sha for revision in revisions):
            decision.block(
                f"run #{index} is stale: revision {revisions[0][:12]} != expected {expected_sha[:12]}"
            )


def rule_for(run: dict[str, Any], result: dict[str, Any]) -> dict[str, Any]:
    rules = run["tool"]["driver"].get("rules", [])
    index = result.get("ruleIndex")
    if isinstance(index, int) and 0 <= index < len(rules) and isinstance(rules[index], dict):
        return rules[index]
    rule_id = result.get("ruleId")
    for rule in rules:
        if isinstance(rule, dict) and rule.get("id") == rule_id:
            return rule
    return {}


def security_severity(*sources: dict[str, Any]) -> float | None:
    for source in sources:
        raw = (source.get("properties") or {}).get("security-severity") if isinstance(source, dict) else None
        try:
            return float(raw)
        except (TypeError, ValueError):
            continue
    return None


def is_high(run: dict[str, Any], result: dict[str, Any]) -> bool:
    rule = rule_for(run, result)
    severity = security_severity(result, rule)
    if severity is not None and severity >= HIGH_SECURITY_SEVERITY:
        return True
    level = result.get("level") or (rule.get("defaultConfiguration") or {}).get("level")
    return level == "error"


def result_path(result: dict[str, Any]) -> str | None:
    for location in result.get("locations", []) or []:
        uri = ((location.get("physicalLocation") or {}).get("artifactLocation") or {}).get("uri")
        if isinstance(uri, str):
            return uri
    return None


def covered(
    scanner: str, rule_id: str | None, path: str | None, exceptions: list[FindingException], today: date
) -> tuple[bool, str | None]:
    """Return (covered, problem). An expired matching exception is a problem, not coverage."""

    expired_match: str | None = None
    for exception in exceptions:
        if exception.scanner != scanner or exception.rule_id != rule_id:
            continue
        if exception.path is not None and exception.path != path:
            continue
        if exception.expires < today:
            expired_match = f"exception for {rule_id} expired on {exception.expires.isoformat()}"
            continue
        return True, None
    return False, expired_match


def decide(
    *,
    scanner: str,
    tool: str,
    sarif: Path,
    expected_sha: str,
    scanner_exit: str,
    exceptions_path: Path | None,
    allowed_skip: str | None,
    today: date,
) -> Decision:
    decision = Decision("pass")
    if scanner_exit in NOT_RUN_STATES:
        if allowed_skip is not None and allowed_skip.strip():
            decision.verdict = "neutral"
            decision.reasons.append(f"skip explicitly allowed by supervisor: {allowed_skip.strip()}")
            return decision
        return decision.block(f"scanner {scanner} did not run and no skip was authorized")
    if scanner_exit != "0":
        return decision.block(f"scanner {scanner} exited with {scanner_exit}")
    try:
        document = parse_sarif(sarif)
    except (FileNotFoundError, ValueError) as exc:
        return decision.block(str(exc))
    check_identity(document, tool, expected_sha, decision)
    try:
        exceptions = load_exceptions(exceptions_path, today)
    except ValueError as exc:
        decision.block(str(exc))
        exceptions = []
    for run in document["runs"]:
        for result in run["results"]:
            if not is_high(run, result):
                continue
            decision.high_findings += 1
            rule_id = result.get("ruleId") if isinstance(result.get("ruleId"), str) else None
            ok, problem = covered(scanner, rule_id, result_path(result), exceptions, today)
            if ok:
                decision.excepted_findings += 1
                continue
            reason = f"high finding {rule_id or '<no rule id>'} at {result_path(result) or '<no path>'}"
            decision.block(f"{reason} ({problem})" if problem else reason)
    if decision.verdict == "pass" and decision.excepted_findings:
        decision.reasons.append(f"{decision.excepted_findings} high finding(s) covered by approved exceptions")
    return decision


def parse_scanner_exit(value: str) -> str:
    if value in NOT_RUN_STATES or re.fullmatch(r"[0-9]{1,3}", value):
        return str(int(value)) if value.isdigit() else value
    raise argparse.ArgumentTypeError("must be an exit code, 'not-run' or 'skipped'")


def parse_expected_sha(value: str) -> str:
    if not SHA_RE.fullmatch(value):
        raise argparse.ArgumentTypeError("must be a full 40-character lowercase hex commit SHA")
    return value


def parse_args(argv: list[str] | None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--scanner", required=True, help="scanner id used by exceptions, e.g. codeql")
    parser.add_argument("--tool", required=True, help="substring of SARIF tool.driver.name")
    parser.add_argument("--sarif", required=True, type=Path)
    parser.add_argument("--expected-sha", required=True, type=parse_expected_sha)
    parser.add_argument("--scanner-exit", required=True, type=parse_scanner_exit)
    parser.add_argument("--exceptions", type=Path, help="governed finding exceptions (TOML)")
    parser.add_argument("--allowed-skip", help="supervisor-provided reason that authorizes a skip")
    parser.add_argument("--today", type=date.fromisoformat, default=None)
    parser.add_argument("--json", action="store_true", help="print one JSON object")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    try:
        args = parse_args(argv)
    except SystemExit as exc:  # argparse already printed the usage error
        return int(exc.code) if isinstance(exc.code, int) else 2
    decision = decide(
        scanner=args.scanner.strip().lower(),
        tool=args.tool,
        sarif=args.sarif,
        expected_sha=args.expected_sha,
        scanner_exit=args.scanner_exit,
        exceptions_path=args.exceptions,
        allowed_skip=args.allowed_skip,
        today=args.today or date.today(),
    )
    if args.json:
        print(
            json.dumps(
                {
                    "scanner": args.scanner,
                    "verdict": decision.verdict,
                    "reasons": decision.reasons,
                    "high_findings": decision.high_findings,
                    "excepted_findings": decision.excepted_findings,
                }
            )
        )
    else:
        label = decision.verdict.upper()
        detail = "; ".join(decision.reasons) if decision.reasons else "no high findings"
        stream = sys.stderr if decision.verdict == "block" else sys.stdout
        print(f"security-findings {args.scanner}: {label} - {detail}", file=stream)
    return 1 if decision.verdict == "block" else 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
# /// script
# requires-python = ">=3.9"
# dependencies = [
#   "tomli>=1.1.0; python_version < '3.11'",
# ]
# ///
# ─── How to run ───
# rtk python3 scripts/check-security-exceptions.py \
#   --config docs/security/advisory-exceptions.toml \
#   --audit-config .cargo/audit.toml \
#   --deny-config deny.toml
"""Validate governed RustSec advisory exceptions."""

from __future__ import annotations

import argparse
import dataclasses
import sys
from dataclasses import dataclass
from datetime import date, timedelta
from pathlib import Path
from typing import Final

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

AUDIT_IGNORE: Final = "cargo-audit:ignore"
AUDIT_WARNING: Final = "cargo-audit:allowed-warning"
DENY_IGNORE: Final = "cargo-deny:ignore"
ALLOWED_TOOLS: Final = frozenset({AUDIT_IGNORE, AUDIT_WARNING, DENY_IGNORE})
PDF_ADVISORY: Final = "RUSTSEC-2026-0192"


@dataclass(frozen=True)  # noqa: SLOTS_OK
class ExceptionRecord:
    """One governed advisory exception."""

    __slots__ = (
        "advisory",
        "crate",
        "versions",
        "tools",
        "dependency_path",
        "feature",
        "exposure",
        "owner",
        "expires",
        "remediation",
        "feature_gated",
        "default_graph",
    )

    advisory: str
    crate: str
    versions: tuple[str, ...]
    tools: frozenset[str]
    dependency_path: tuple[str, ...]
    feature: str
    exposure: str
    owner: str
    expires: date
    remediation: str
    feature_gated: bool
    default_graph: bool


class CheckError(RuntimeError):
    """Raised when advisory exception governance is invalid."""


def read_toml(path: Path):
    """Read TOML with path-specific diagnostics."""

    try:
        with path.open("rb") as handle:
            return tomllib.load(handle)
    except FileNotFoundError as exc:
        raise CheckError(f"missing TOML file: {path}") from exc
    except tomllib.TOMLDecodeError as exc:
        raise CheckError(f"invalid TOML in {path}: {exc}") from exc
    except OSError as exc:
        raise CheckError(f"could not read {path}: {exc}") from exc


def string(table, key: str, advisory: str) -> str:
    """Read a required non-empty string."""

    value = table.get(key)
    if not isinstance(value, str) or not value.strip():
        raise CheckError(f"{advisory}: missing non-empty {key}")
    return value


def strings(table, key: str, advisory: str) -> tuple[str, ...]:
    """Read a required non-empty list of strings."""

    value = table.get(key)
    if not isinstance(value, list) or not value:
        raise CheckError(f"{advisory}: missing non-empty list {key}")
    parsed: list[str] = []
    for item in value:
        if not isinstance(item, str) or not item.strip():
            raise CheckError(f"{advisory}: {key} contains a non-string/empty item")
        parsed.append(item)
    return tuple(parsed)


def boolean(table, key: str, advisory: str) -> bool:
    """Read a required boolean."""

    value = table.get(key)
    if not isinstance(value, bool):
        raise CheckError(f"{advisory}: missing boolean {key}")
    return value


def local_date(table, key: str, advisory: str) -> date:
    """Read a required TOML local-date."""

    value = table.get(key)
    if not isinstance(value, date):
        raise CheckError(f"{advisory}: missing TOML date {key}")
    return value


def parse_record(table) -> ExceptionRecord:
    """Parse and validate one exception record."""

    advisory = string(table, "advisory", "<unknown>")
    tools = frozenset(strings(table, "tools", advisory))
    unknown = tools - ALLOWED_TOOLS
    if unknown:
        raise CheckError(f"{advisory}: unknown tool mapping(s): {', '.join(sorted(unknown))}")
    if not tools:
        raise CheckError(f"{advisory}: missing tool mapping")
    return ExceptionRecord(
        advisory=advisory,
        crate=string(table, "crate", advisory),
        versions=strings(table, "versions", advisory),
        tools=tools,
        dependency_path=strings(table, "dependency_path", advisory),
        feature=string(table, "feature", advisory),
        exposure=string(table, "exposure", advisory),
        owner=string(table, "owner", advisory),
        expires=local_date(table, "expires", advisory),
        remediation=string(table, "remediation", advisory),
        feature_gated=boolean(table, "feature_gated", advisory),
        default_graph=boolean(table, "default_graph", advisory),
    )


def parse_config(path: Path) -> tuple[ExceptionRecord, ...]:
    """Parse governed exception records."""

    exceptions = read_toml(path).get("exceptions")
    if not isinstance(exceptions, list) or not exceptions:
        raise CheckError("config must contain at least one [[exceptions]] record")
    records = tuple(parse_record(entry) for entry in exceptions)
    ids = [record.advisory for record in records]
    duplicates = {advisory for advisory in ids if ids.count(advisory) > 1}
    if duplicates:
        raise CheckError(f"duplicate advisory records: {', '.join(sorted(duplicates))}")
    return records


def advisory_ignores(path: Path) -> frozenset[str]:
    """Read [advisories].ignore from cargo-audit or cargo-deny config."""

    advisories = read_toml(path).get("advisories", {})
    if not isinstance(advisories, dict):
        raise CheckError(f"{path}: [advisories] must be a table")
    ignore = advisories.get("ignore", [])
    if not isinstance(ignore, list):
        raise CheckError(f"{path}: [advisories].ignore must be a list")
    for item in ignore:
        if not isinstance(item, str) or not item.strip():
            raise CheckError(f"{path}: [advisories].ignore contains a non-string/empty item")
    return frozenset(ignore)


def cargo_deny_ban_skips(path: Path) -> frozenset[str]:
    """Read cargo-deny ban skips; advisory governance keeps them absent."""

    bans = read_toml(path).get("bans", {})
    if not isinstance(bans, dict):
        raise CheckError(f"{path}: [bans] must be a table")
    skipped: set[str] = set()
    for key in ("skip", "skip-tree"):
        values = bans.get(key, [])
        if not isinstance(values, list):
            raise CheckError(f"{path}: [bans].{key} must be a list when present")
        for value in values:
            if isinstance(value, str):
                skipped.add(value)
                continue
            if isinstance(value, dict) and isinstance(value.get("crate"), str):
                skipped.add(value["crate"])
                continue
            raise CheckError(f"{path}: [bans].{key} contains an unsupported entry")
    return frozenset(skipped)


def mapped(records: tuple[ExceptionRecord, ...], tool: str) -> frozenset[str]:
    """Return advisory IDs mapped to a tool."""

    return frozenset(record.advisory for record in records if tool in record.tools)


def show(values: frozenset[str]) -> str:
    """Format a diagnostic set."""

    return ", ".join(sorted(values)) if values else "<none>"


def validate_records(
    records: tuple[ExceptionRecord, ...],
    audit_ignores: frozenset[str],
    deny_ignores: frozenset[str],
    deny_skips: frozenset[str],
    today: date,
) -> None:
    """Validate ownership, expiry, parity, and scoped PDF classification."""

    errors: list[str] = []
    if deny_skips:
        errors.append(f"cargo-deny bans skip/skip-tree entries are not allowed: {show(deny_skips)}")
    for record in records:
        if record.expires < today:
            errors.append(f"{record.advisory}: expired on {record.expires.isoformat()}")
        if not record.owner.strip():
            errors.append(f"{record.advisory}: missing owner")
        if AUDIT_WARNING in record.tools and AUDIT_IGNORE in record.tools:
            errors.append(f"{record.advisory}: cannot be both audit ignore and allowed warning")
    audit_mapped = mapped(records, AUDIT_IGNORE)
    deny_mapped = mapped(records, DENY_IGNORE)
    if audit_mapped != audit_ignores:
        errors.append(
            "cargo-audit ignore parity mismatch; "
            f"config-only={show(audit_mapped - audit_ignores)} "
            f"audit-only={show(audit_ignores - audit_mapped)}"
        )
    if deny_mapped != deny_ignores:
        errors.append(
            "cargo-deny ignore parity mismatch; "
            f"config-only={show(deny_mapped - deny_ignores)} "
            f"deny-only={show(deny_ignores - deny_mapped)}"
        )
    pdf_records = [record for record in records if record.advisory == PDF_ADVISORY]
    if len(pdf_records) != 1:
        errors.append(f"{PDF_ADVISORY}: exactly one governed PDF record is required")
    else:
        pdf = pdf_records[0]
        if AUDIT_IGNORE in pdf.tools or DENY_IGNORE in pdf.tools:
            errors.append(f"{PDF_ADVISORY}: must not be globally ignored")
        if AUDIT_WARNING not in pdf.tools:
            errors.append(f"{PDF_ADVISORY}: must be classified as cargo-audit allowed warning")
        if not pdf.feature_gated or pdf.default_graph or "pdf" not in pdf.feature.lower():
            errors.append(f"{PDF_ADVISORY}: must be pdf feature-gated and absent from default graph")
    if errors:
        raise CheckError("\n".join(errors))


def expect_blocker(records: tuple[ExceptionRecord, ...], mutated: ExceptionRecord, text: str) -> None:
    """Run an in-memory negative self-test and require the expected blocker."""

    changed = (mutated, *records[1:])
    try:
        validate_records(changed, mapped(changed, AUDIT_IGNORE), mapped(changed, DENY_IGNORE), frozenset(), date.today())
    except CheckError as exc:
        if text in str(exc):
            print(f"self-test-{text.replace(' ', '-')}: PASS")
            return
        raise CheckError(f"self-test for {text} failed for the wrong reason") from exc
    raise CheckError(f"self-test for {text} failed: invalid record was accepted")


def parse_args() -> argparse.Namespace:
    """Parse CLI flags."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", required=True, type=Path)
    parser.add_argument("--audit-config", default=Path(".cargo/audit.toml"), type=Path)
    parser.add_argument("--deny-config", default=Path("deny.toml"), type=Path)
    parser.add_argument("--self-test-expired", action="store_true")
    parser.add_argument("--self-test-missing-owner", action="store_true")
    return parser.parse_args()


def main() -> int:
    """Run the configured validation or one negative self-test."""

    args = parse_args()
    try:
        records = parse_config(args.config)
        if args.self_test_expired:
            expect_blocker(records, dataclasses.replace(records[0], expires=date.today() - timedelta(days=1)), "expired")
        elif args.self_test_missing_owner:
            expect_blocker(records, dataclasses.replace(records[0], owner=""), "missing owner")
        else:
            validate_records(
                records,
                advisory_ignores(args.audit_config),
                advisory_ignores(args.deny_config),
                cargo_deny_ban_skips(args.deny_config),
                date.today(),
            )
            print(f"security exception check: PASS ({len(records)} governed records)")
    except CheckError as exc:
        print(f"security exception check failed:\n{exc}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
"""Unified Version Management & Consistency Validator for Engram.

Manages semantic versioning across Rust crates, Python SDK, TypeScript SDK,
WASM crate, documentation badges, and release channel matrices.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import UTC, datetime
from pathlib import Path
from typing import NamedTuple

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib  # type: ignore

REPO_ROOT = Path(__file__).resolve().parent.parent

CARGO_TOML = REPO_ROOT / "Cargo.toml"
WASM_CARGO_TOML = REPO_ROOT / "engram-wasm" / "Cargo.toml"
PYPROJECT_TOML = REPO_ROOT / "sdks" / "python" / "pyproject.toml"
PACKAGE_JSON = REPO_ROOT / "sdks" / "typescript" / "package.json"
CHANNEL_MATRIX = REPO_ROOT / "docs" / "releases" / "channel-matrix.toml"
README_MD = REPO_ROOT / "README.md"
DOCS_INDEX_HTML = REPO_ROOT / "docs" / "index.html"
CHANGELOG_MD = REPO_ROOT / "CHANGELOG.md"

SEMVER_REGEX = re.compile(r"^v?([0-9]+)\.([0-9]+)\.([0-9]+)(?:-([0-9A-Za-z.-]+))?$")


class VersionReport(NamedTuple):
    core: str
    wasm: str
    python: str
    typescript: str
    matrix_core: str
    matrix_observed_utc: str


def validate_semver(version: str) -> str:
    cleaned = version.removeprefix("v")
    if not SEMVER_REGEX.match(cleaned):
        raise ValueError(f"Invalid semver version: '{version}'. Expected format: X.Y.Z (e.g. 0.22.0)")
    return cleaned


def get_current_versions() -> VersionReport:
    # 1. Root Cargo.toml
    with CARGO_TOML.open("rb") as f:
        cargo_data = tomllib.load(f)
    core_ver = cargo_data["package"]["version"]

    # 2. WASM Cargo.toml
    with WASM_CARGO_TOML.open("rb") as f:
        wasm_data = tomllib.load(f)
    wasm_ver = wasm_data["package"]["version"]

    # 3. Python pyproject.toml
    with PYPROJECT_TOML.open("rb") as f:
        py_data = tomllib.load(f)
    py_ver = py_data["project"]["version"]

    # 4. TypeScript package.json
    with PACKAGE_JSON.open("r", encoding="utf-8") as f:
        ts_data = json.load(f)
    ts_ver = ts_data["version"]

    # 5. Channel Matrix
    with CHANNEL_MATRIX.open("rb") as f:
        matrix_data = tomllib.load(f)
    matrix_core = matrix_data["matrix"]["core_version"]
    matrix_utc = matrix_data["matrix"]["observed_utc"]

    return VersionReport(
        core=core_ver,
        wasm=wasm_ver,
        python=py_ver,
        typescript=ts_ver,
        matrix_core=matrix_core,
        matrix_observed_utc=matrix_utc,
    )


def check_consistency(quiet: bool = False) -> bool:
    report = get_current_versions()
    issues: list[str] = []

    if not quiet:
        print("=== Engram Version Consistency Check ===")
        print(f"  • engram-core (Rust):     v{report.core}")
        print(f"  • engram-wasm (WASM):     v{report.wasm}")
        print(f"  • engram-client (Python): v{report.python}")
        print(f"  • engram-client (TS):     v{report.typescript}")
        print(f"  • Channel Matrix Core:    v{report.matrix_core}")
        print(f"  • Matrix Observed UTC:    {report.matrix_observed_utc}")
        print("-----------------------------------------")

    # Check 1: Matrix core version matches Cargo.toml
    if report.core != report.matrix_core:
        issues.append(f"Cargo.toml ({report.core}) != channel-matrix.toml ({report.matrix_core})")

    # Check 2: Matrix freshness (< 30 days)
    try:
        dt = datetime.fromisoformat(report.matrix_observed_utc.replace("Z", "+00:00"))
        age_days = (datetime.now(UTC) - dt).days
        if age_days > 30:
            issues.append(f"channel-matrix.toml is stale ({age_days} days old > 30 days limit). Run --refresh-matrix to update.")
    except Exception as e:
        issues.append(f"Invalid timestamp in channel-matrix.toml: {e}")

    # Check 3: Changelog section exists for core version
    if CHANGELOG_MD.exists():
        changelog_content = CHANGELOG_MD.read_text(encoding="utf-8")
        if f"[{report.core}]" not in changelog_content and f"v{report.core}" not in changelog_content:
            issues.append(f"CHANGELOG.md missing entry for core version [{report.core}]")

    if issues:
        print("❌ Version consistency issues found:")
        for issue in issues:
            print(f"   - {issue}")
        return False

    print("✅ All package versions and matrix contracts are consistent.")
    return True


def refresh_matrix_timestamp() -> None:
    now_utc = datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ")
    content = CHANNEL_MATRIX.read_text(encoding="utf-8")
    updated = re.sub(
        r'observed_utc\s*=\s*"[^"]*"',
        f'observed_utc = "{now_utc}"',
        content,
        count=1,
    )
    CHANNEL_MATRIX.write_text(updated, encoding="utf-8")
    print(f"✅ Updated channel-matrix.toml observed_utc to: {now_utc}")


def bump_core_version(new_version: str) -> None:
    ver = validate_semver(new_version)
    print(f"==> Bumping engram-core to {ver}...")

    # 1. Cargo.toml
    cargo_content = CARGO_TOML.read_text(encoding="utf-8")
    cargo_updated = re.sub(
        r'(\[package\]\s*\nname\s*=\s*"engram-core"\s*\nversion\s*=\s*)"[^"]+"',
        rf'\g<1>"{ver}"',
        cargo_content,
        count=1,
    )
    CARGO_TOML.write_text(cargo_updated, encoding="utf-8")

    # 2. channel-matrix.toml
    matrix_content = CHANNEL_MATRIX.read_text(encoding="utf-8")
    matrix_updated = re.sub(
        r'core_version\s*=\s*"[^"]*"',
        f'core_version = "{ver}"',
        matrix_content,
        count=1,
    )
    matrix_updated = re.sub(
        r'core_tag\s*=\s*"[^"]*"',
        f'core_tag = "v{ver}"',
        matrix_updated,
        count=1,
    )
    now_utc = datetime.now(UTC).strftime("%Y-%m-%dT%H:%M:%SZ")
    matrix_updated = re.sub(
        r'observed_utc\s*=\s*"[^"]*"',
        f'observed_utc = "{now_utc}"',
        matrix_updated,
        count=1,
    )
    # Update local core channel
    matrix_updated = re.sub(
        r'(id\s*=\s*"core"[\s\S]*?local_version\s*=\s*)"[^"]*"',
        rf'\g<1>"{ver}"',
        matrix_updated,
        count=1,
    )
    matrix_updated = re.sub(
        r'(id\s*=\s*"core"[\s\S]*?observed_version\s*=\s*)"[^"]*"',
        rf'\g<1>"{ver}"',
        matrix_updated,
        count=1,
    )
    # Update git_tag target
    matrix_updated = re.sub(
        r'(id\s*=\s*"git_tag"[\s\S]*?target\s*=\s*)"[^"]*"',
        rf'\g<1>"v{ver}"',
        matrix_updated,
        count=1,
    )
    # Update github_release target
    matrix_updated = re.sub(
        r'(id\s*=\s*"github_release"[\s\S]*?target\s*=\s*)"[^"]*"',
        rf'\g<1>"v{ver}"',
        matrix_updated,
        count=1,
    )
    # Update crates_io local_version
    matrix_updated = re.sub(
        r'(id\s*=\s*"crates_io"[\s\S]*?local_version\s*=\s*)"[^"]*"',
        rf'\g<1>"{ver}"',
        matrix_updated,
        count=1,
    )
    CHANNEL_MATRIX.write_text(matrix_updated, encoding="utf-8")

    # 3. Update README & docs badges if applicable
    if README_MD.exists():
        readme_content = README_MD.read_text(encoding="utf-8")
        readme_updated = re.sub(
            r'Release-v[0-9.]+%20GA-success\.svg',
            f'Release-v{ver}%20GA-success.svg',
            readme_content,
        )
        README_MD.write_text(readme_updated, encoding="utf-8")

    if DOCS_INDEX_HTML.exists():
        html_content = DOCS_INDEX_HTML.read_text(encoding="utf-8")
        html_updated = re.sub(
            r'v[0-9.]+\s*GA',
            f'v{ver} GA',
            html_content,
        )
        html_updated = re.sub(
            r'"softwareVersion":\s*"[^"]*"',
            f'"softwareVersion": "{ver}"',
            html_updated,
        )
        DOCS_INDEX_HTML.write_text(html_updated, encoding="utf-8")

    print(f"✅ Core successfully bumped to {ver}.")


def bump_python_version(new_version: str) -> None:
    ver = validate_semver(new_version)
    print(f"==> Bumping engram-client (Python SDK) to {ver}...")

    # 1. pyproject.toml
    py_content = PYPROJECT_TOML.read_text(encoding="utf-8")
    py_updated = re.sub(
        r'(\[project\]\s*\nname\s*=\s*"engram-client"\s*\nversion\s*=\s*)"[^"]+"',
        rf'\g<1>"{ver}"',
        py_content,
        count=1,
    )
    PYPROJECT_TOML.write_text(py_updated, encoding="utf-8")

    # 2. channel-matrix.toml
    matrix_content = CHANNEL_MATRIX.read_text(encoding="utf-8")
    matrix_updated = re.sub(
        r'(id\s*=\s*"pypi"[\s\S]*?local_version\s*=\s*)"[^"]*"',
        rf'\g<1>"{ver}"',
        matrix_content,
        count=1,
    )
    matrix_updated = re.sub(
        r'(sdk\s*=\s*"python"[\s\S]*?local_version\s*=\s*)"[^"]*"',
        rf'\g<1>"{ver}"',
        matrix_updated,
        count=1,
    )
    CHANNEL_MATRIX.write_text(matrix_updated, encoding="utf-8")
    print(f"✅ Python SDK successfully bumped to {ver}.")


def bump_typescript_version(new_version: str) -> None:
    ver = validate_semver(new_version)
    print(f"==> Bumping engram-client (TypeScript SDK) to {ver}...")

    # 1. package.json
    with PACKAGE_JSON.open("r", encoding="utf-8") as f:
        ts_data = json.load(f)
    ts_data["version"] = ver
    with PACKAGE_JSON.open("w", encoding="utf-8") as f:
        json.dump(ts_data, f, indent=2)
        f.write("\n")

    # 2. channel-matrix.toml
    matrix_content = CHANNEL_MATRIX.read_text(encoding="utf-8")
    matrix_updated = re.sub(
        r'(id\s*=\s*"npm"[\s\S]*?local_version\s*=\s*)"[^"]*"',
        rf'\g<1>"{ver}"',
        matrix_content,
        count=1,
    )
    matrix_updated = re.sub(
        r'(sdk\s*=\s*"typescript"[\s\S]*?local_version\s*=\s*)"[^"]*"',
        rf'\g<1>"{ver}"',
        matrix_updated,
        count=1,
    )
    CHANNEL_MATRIX.write_text(matrix_updated, encoding="utf-8")
    print(f"✅ TypeScript SDK successfully bumped to {ver}.")


def bump_wasm_version(new_version: str) -> None:
    ver = validate_semver(new_version)
    print(f"==> Bumping engram-wasm to {ver}...")

    wasm_content = WASM_CARGO_TOML.read_text(encoding="utf-8")
    wasm_updated = re.sub(
        r'(\[package\]\s*\nname\s*=\s*"engram-wasm"\s*\nversion\s*=\s*)"[^"]+"',
        rf'\g<1>"{ver}"',
        wasm_content,
        count=1,
    )
    WASM_CARGO_TOML.write_text(wasm_updated, encoding="utf-8")
    print(f"✅ WASM crate successfully bumped to {ver}.")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Engram Unified Version Management and Consistency Tool",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""Examples:
  python3 scripts/bump-version.py --check
  python3 scripts/bump-version.py --refresh-matrix
  python3 scripts/bump-version.py --core 0.23.0
  python3 scripts/bump-version.py --python 0.5.1
  python3 scripts/bump-version.py --typescript 0.5.1
  python3 scripts/bump-version.py --wasm 0.1.1
""",
    )
    parser.add_argument("--check", action="store_true", help="Validate version consistency across the repository")
    parser.add_argument("--refresh-matrix", action="store_true", help="Update observed_utc timestamp in channel matrix to now")
    parser.add_argument("--core", metavar="VERSION", help="Bump engram-core (Rust) version")
    parser.add_argument("--python", metavar="VERSION", help="Bump engram-client (Python SDK) version")
    parser.add_argument("--typescript", metavar="VERSION", help="Bump engram-client (TypeScript SDK) version")
    parser.add_argument("--wasm", metavar="VERSION", help="Bump engram-wasm version")

    args = parser.parse_args()

    if not any([args.check, args.refresh_matrix, args.core, args.python, args.typescript, args.wasm]):
        parser.print_help()
        return 0

    has_mutation = any([args.refresh_matrix, args.core, args.python, args.typescript, args.wasm])

    if args.refresh_matrix:
        refresh_matrix_timestamp()

    if args.core:
        bump_core_version(args.core)

    if args.python:
        bump_python_version(args.python)

    if args.typescript:
        bump_typescript_version(args.typescript)

    if args.wasm:
        bump_wasm_version(args.wasm)

    if args.check or has_mutation:
        success = check_consistency(quiet=False)
        return 0 if success else 1

    return 0


if __name__ == "__main__":
    sys.exit(main())

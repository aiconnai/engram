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
CARGO_LOCK = REPO_ROOT / "Cargo.lock"
TYPES_CARGO_TOML = REPO_ROOT / "crates" / "engram-types" / "Cargo.toml"
WASM_CARGO_TOML = REPO_ROOT / "engram-wasm" / "Cargo.toml"
PYPROJECT_TOML = REPO_ROOT / "sdks" / "python" / "pyproject.toml"
PYTHON_INIT_PY = REPO_ROOT / "sdks" / "python" / "engram_client" / "__init__.py"
PACKAGE_JSON = REPO_ROOT / "sdks" / "typescript" / "package.json"
PACKAGE_LOCK_JSON = REPO_ROOT / "sdks" / "typescript" / "package-lock.json"
CHANNEL_MATRIX = REPO_ROOT / "docs" / "releases" / "channel-matrix.toml"
README_MD = REPO_ROOT / "README.md"
DOCS_INDEX_HTML = REPO_ROOT / "docs" / "index.html"
CHANGELOG_MD = REPO_ROOT / "CHANGELOG.md"

SEMVER_REGEX = re.compile(r"^v?([0-9]+)\.([0-9]+)\.([0-9]+)(?:-([0-9A-Za-z.-]+))?$")
STABLE_TAG_REGEX = re.compile(r"^v([0-9]+)\.([0-9]+)\.([0-9]+)$")
DUNDER_VERSION_REGEX = re.compile(r"""^__version__\s*=\s*["']([^"']+)["']\s*$""", re.MULTILINE)


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


def read_cargo_lock_versions(path: Path) -> dict[str, list[str]]:
    """Return versions of path/workspace packages (entries without `source`) in a Cargo.lock."""
    with path.open("rb") as f:
        lock_data = tomllib.load(f)
    versions: dict[str, list[str]] = {}
    for package in lock_data.get("package", []):
        if "source" in package:
            continue
        versions.setdefault(package["name"], []).append(package["version"])
    return versions


def read_package_lock_versions(path: Path) -> tuple[str | None, str | None]:
    """Return (root `version`, `packages[""].version`) from an npm package-lock.json."""
    with path.open("r", encoding="utf-8") as f:
        lock_data = json.load(f)
    root_version = lock_data.get("version")
    package_version = lock_data.get("packages", {}).get("", {}).get("version")
    return root_version, package_version


def read_dunder_version(path: Path) -> str | None:
    match = DUNDER_VERSION_REGEX.search(path.read_text(encoding="utf-8"))
    return match.group(1) if match else None


def workspace_manifests() -> dict[str, Path]:
    return {
        "engram-core": CARGO_TOML,
        "engram-types": TYPES_CARGO_TOML,
        "engram-wasm": WASM_CARGO_TOML,
    }


def collect_cargo_lock_issues() -> list[str]:
    if not CARGO_LOCK.exists():
        return [f"{CARGO_LOCK.name} is missing"]
    lock_versions = read_cargo_lock_versions(CARGO_LOCK)
    issues: list[str] = []
    for name, manifest in workspace_manifests().items():
        with manifest.open("rb") as f:
            manifest_version = tomllib.load(f)["package"]["version"]
        locked = lock_versions.get(name, [])
        if locked != [manifest_version]:
            issues.append(
                f"Cargo.lock entry for {name} ({', '.join(locked) or 'missing'}) != "
                f"{manifest.name} ({manifest_version})"
            )
    return issues


def collect_artifact_issues(report: VersionReport) -> list[str]:
    """Check lockfiles and in-source version constants against the manifests."""
    issues = collect_cargo_lock_issues()

    dunder = read_dunder_version(PYTHON_INIT_PY) if PYTHON_INIT_PY.exists() else None
    if dunder != report.python:
        issues.append(
            f"sdks/python/engram_client/__init__.py __version__ ({dunder}) != pyproject.toml ({report.python})"
        )

    if not PACKAGE_LOCK_JSON.exists():
        issues.append("sdks/typescript/package-lock.json is missing")
    else:
        lock_root, lock_package = read_package_lock_versions(PACKAGE_LOCK_JSON)
        if lock_root != report.typescript:
            issues.append(
                f"package-lock.json version ({lock_root}) != package.json ({report.typescript})"
            )
        if lock_package != report.typescript:
            issues.append(
                f'package-lock.json packages[""].version ({lock_package}) != package.json ({report.typescript})'
            )
    return issues


def collect_issues(report: VersionReport, *, check_staleness: bool = True) -> list[str]:
    issues: list[str] = []

    # Check 1: Matrix core version matches Cargo.toml
    if report.core != report.matrix_core:
        issues.append(f"Cargo.toml ({report.core}) != channel-matrix.toml ({report.matrix_core})")

    # Check 2: Matrix freshness (< 30 days)
    try:
        dt = datetime.fromisoformat(report.matrix_observed_utc.replace("Z", "+00:00"))
        age_days = (datetime.now(UTC) - dt).days
        if check_staleness and age_days > 30:
            issues.append(f"channel-matrix.toml is stale ({age_days} days old > 30 days limit). Run --refresh-matrix to update.")
    except Exception as e:
        issues.append(f"Invalid timestamp in channel-matrix.toml: {e}")

    # Check 3: Changelog section exists for core version
    if CHANGELOG_MD.exists():
        changelog_content = CHANGELOG_MD.read_text(encoding="utf-8")
        if f"[{report.core}]" not in changelog_content and f"v{report.core}" not in changelog_content:
            issues.append(f"CHANGELOG.md missing entry for core version [{report.core}]")

    # Check 4: Lockfiles and in-source version constants match the manifests
    issues.extend(collect_artifact_issues(report))
    return issues


def check_consistency(quiet: bool = False) -> bool:
    report = get_current_versions()

    if not quiet:
        print("=== Engram Version Consistency Check ===")
        print(f"  • engram-core (Rust):     v{report.core}")
        print(f"  • engram-wasm (WASM):     v{report.wasm}")
        print(f"  • engram-client (Python): v{report.python}")
        print(f"  • engram-client (TS):     v{report.typescript}")
        print(f"  • Channel Matrix Core:    v{report.matrix_core}")
        print(f"  • Matrix Observed UTC:    {report.matrix_observed_utc}")
        print("-----------------------------------------")

    issues = collect_issues(report)

    if issues:
        print("❌ Version consistency issues found:")
        for issue in issues:
            print(f"   - {issue}")
        return False

    print("✅ All package versions and matrix contracts are consistent.")
    return True


class ReleasePlan(NamedTuple):
    core_version: str
    core_tag: str
    python_version: str
    typescript_version: str


class ReleaseMismatchError(Exception):
    """Raised when the release tag, Cargo version, and SDK manifests disagree."""

    def __init__(self, issues: list[str]) -> None:
        super().__init__("; ".join(issues))
        self.issues = issues


def core_in_range(core_version: str, minimum: str, maximum: str) -> bool:
    """True when core is within [minimum, maximum]; maximum may be `X.Y.x`."""

    def triple(version: str) -> tuple[int, int, int] | None:
        match = STABLE_TAG_REGEX.match(f"v{version.removeprefix('v')}")
        return tuple(int(part) for part in match.groups()) if match else None  # type: ignore[return-value]

    core, low = triple(core_version), triple(minimum)
    if core is None or low is None or core < low:
        return False
    if maximum.endswith(".x"):
        major_minor = maximum[: -len(".x")].split(".")
        return len(major_minor) == 2 and all(p.isdigit() for p in major_minor) and core[:2] == (
            int(major_minor[0]),
            int(major_minor[1]),
        )
    high = triple(maximum)
    return high is not None and core <= high


def _matrix_row(rows: list[dict], key: str, value: str) -> dict | None:
    return next((row for row in rows if row.get(key) == value), None)


def _check_matrix_release_rows(
    report: VersionReport, tag: str | None, matrix: dict
) -> list[str]:
    issues: list[str] = []
    channels = matrix.get("channels", [])
    sdk_rows = matrix.get("sdk_compatibility", [])

    if tag is not None:
        declared = {"core_tag": matrix["matrix"].get("core_tag")}
        for channel_id in ("git_tag", "github_release"):
            row = _matrix_row(channels, "id", channel_id)
            declared[f"{channel_id}.target"] = row.get("target") if row else None
        for label, value in declared.items():
            if value != tag:
                issues.append(f"channel-matrix.toml {label} ({value}) != release tag ({tag})")

    for sdk, channel_id, local in (
        ("python", "pypi", report.python),
        ("typescript", "npm", report.typescript),
    ):
        channel = _matrix_row(channels, "id", channel_id)
        if channel is None or channel.get("local_version") != local:
            found = channel.get("local_version") if channel else "missing"
            issues.append(f"channel-matrix.toml {channel_id} local_version ({found}) != {sdk} manifest ({local})")
        row = _matrix_row(sdk_rows, "sdk", sdk)
        if row is None:
            issues.append(f"channel-matrix.toml has no sdk_compatibility row for {sdk}")
            continue
        if row.get("local_version") != local:
            issues.append(
                f"channel-matrix.toml {sdk} compatibility local_version ({row.get('local_version')}) != manifest ({local})"
            )
        if not core_in_range(report.core, row.get("compatible_core_min", ""), row.get("compatible_core_max", "")):
            issues.append(
                f"core {report.core} is outside the {sdk} SDK tested range "
                f"{row.get('compatible_core_min')}..{row.get('compatible_core_max')}"
            )
    return issues


def resolve_release_plan(tag: str | None) -> ReleasePlan:
    """Derive the expected core tag and SDK versions from the manifests; fail closed on drift.

    The core tag is always `v<Cargo.toml version>`. SDK versions are independent of the core
    tag and come from their own manifests. When `tag` is given (publish runs) it must equal
    the derived core tag and be declared by the reviewed channel matrix. Matrix staleness is a
    commit-time gate (`--check`); it is intentionally not re-evaluated here so that a publish
    run on an already-tagged commit is not invalidated by the passage of time.
    """
    report = get_current_versions()
    issues = collect_issues(report, check_staleness=False)

    expected_tag = f"v{report.core}"
    if not STABLE_TAG_REGEX.match(expected_tag):
        issues.append(f"Cargo.toml version ({report.core}) is not a stable X.Y.Z release version")
    if tag is not None:
        if not STABLE_TAG_REGEX.match(tag):
            issues.append(f"release tag ({tag!r}) must be an exact stable vX.Y.Z tag")
        elif tag != expected_tag:
            issues.append(f"release tag ({tag}) != v<Cargo.toml version> ({expected_tag})")

    for label, version in (("Python SDK", report.python), ("TypeScript SDK", report.typescript)):
        try:
            validate_semver(version)
        except ValueError as error:
            issues.append(f"{label} version is invalid: {error}")

    with CHANNEL_MATRIX.open("rb") as f:
        matrix = tomllib.load(f)
    issues.extend(_check_matrix_release_rows(report, tag, matrix))

    if issues:
        raise ReleaseMismatchError(issues)
    return ReleasePlan(report.core, expected_tag, report.python, report.typescript)


def run_release_plan(tag: str | None) -> int:
    """Print machine-readable key=value lines on stdout; diagnostics go to stderr."""
    try:
        plan = resolve_release_plan(tag)
    except ReleaseMismatchError as error:
        print("release-plan: version mapping check FAILED:", file=sys.stderr)
        for issue in error.issues:
            print(f"  - {issue}", file=sys.stderr)
        return 1
    print(f"core_version={plan.core_version}")
    print(f"core_tag={plan.core_tag}")
    print(f"python_version={plan.python_version}")
    print(f"npm_version={plan.typescript_version}")
    return 0


def sync_cargo_lock(package_name: str, version: str) -> None:
    """Keep the workspace package entry in Cargo.lock aligned with its manifest."""
    if not CARGO_LOCK.exists():
        return
    updated, count = re.subn(
        rf'(\[\[package\]\]\nname = "{re.escape(package_name)}"\nversion = )"[^"]+"(?=\n(?!source = ))',
        rf'\g<1>"{version}"',
        CARGO_LOCK.read_text(encoding="utf-8"),
        count=1,
    )
    if count == 0:
        print(f"   ⚠️ Cargo.lock has no workspace entry for {package_name}; run cargo to refresh it.", file=sys.stderr)
        return
    CARGO_LOCK.write_text(updated, encoding="utf-8")


def sync_package_lock(version: str) -> None:
    """Update only the two version fields npm records for the root package."""
    if not PACKAGE_LOCK_JSON.exists():
        return
    text = PACKAGE_LOCK_JSON.read_text(encoding="utf-8")
    text = re.sub(r'^(  "version":\s*)"[^"]*"', rf'\g<1>"{version}"', text, count=1, flags=re.MULTILINE)
    text = re.sub(
        r'(^    "":\s*\{[^}]*?^      "version":\s*)"[^"]*"',
        rf'\g<1>"{version}"',
        text,
        count=1,
        flags=re.MULTILINE,
    )
    PACKAGE_LOCK_JSON.write_text(text, encoding="utf-8")


def sync_python_dunder_version(version: str) -> None:
    if not PYTHON_INIT_PY.exists():
        return
    text = PYTHON_INIT_PY.read_text(encoding="utf-8")
    PYTHON_INIT_PY.write_text(
        DUNDER_VERSION_REGEX.sub(f'__version__ = "{version}"', text, count=1), encoding="utf-8"
    )


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
    sync_cargo_lock("engram-core", ver)

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
    print("   ℹ️ Note: Local targets and channel-matrix.toml local_version staged.")
    print("   Registry observed_version fields will remain at published baseline until release is completed.")



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
    sync_python_dunder_version(ver)

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
    sync_package_lock(ver)

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
    sync_cargo_lock("engram-wasm", ver)
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
  python3 scripts/bump-version.py --release-plan --release-tag v0.24.0
""",
    )
    parser.add_argument("--check", action="store_true", help="Validate version consistency across the repository")
    parser.add_argument("--refresh-matrix", action="store_true", help="Update observed_utc timestamp in channel matrix to now")
    parser.add_argument("--core", metavar="VERSION", help="Bump engram-core (Rust) version")
    parser.add_argument("--python", metavar="VERSION", help="Bump engram-client (Python SDK) version")
    parser.add_argument("--typescript", metavar="VERSION", help="Bump engram-client (TypeScript SDK) version")
    parser.add_argument("--wasm", metavar="VERSION", help="Bump engram-wasm version")
    parser.add_argument(
        "--release-plan",
        action="store_true",
        help="Derive and verify the release version mapping; print key=value lines (read-only, for CI)",
    )
    parser.add_argument(
        "--release-tag",
        metavar="TAG",
        help="With --release-plan: exact core tag that must equal v<Cargo.toml version>",
    )

    args = parser.parse_args()

    if args.release_tag is not None and not args.release_plan:
        parser.error("--release-tag requires --release-plan")
    if args.release_plan:
        if any([args.check, args.refresh_matrix, args.core, args.python, args.typescript, args.wasm]):
            parser.error("--release-plan is read-only and cannot be combined with other actions")
        return run_release_plan(args.release_tag)

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

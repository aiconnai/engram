"""Release-channel static/live checks and regression self-tests."""

from __future__ import annotations

import sys
import shutil
import tempfile
from datetime import UTC, datetime
from pathlib import Path

from release_channels.model import (
    DEFAULT_MATRIX, MISSING_SELF_TEST_TAG, REQUIRED_CHANNELS, REQUIRED_SDKS,
    Channel, CheckResult, CommandResult, MatrixPolicy, SdkCompatibility,
    channel_map, fetch_pypi_package, local_version_from_cargo,
    local_version_from_package_json, local_version_from_pyproject,
    parse_exact_version_line, parse_homebrew_version, parse_matrix,
    parse_npm_version, parse_pypi_json_version, read_text, run_command,
    tag_exists, version_matches,
)


def semver_tuple(version: str) -> tuple[int, int, int] | None:
    parts = version.removeprefix("v").split(".")
    if len(parts) != 3 or not all(part.isdigit() for part in parts):
        return None
    return int(parts[0]), int(parts[1]), int(parts[2])


def core_in_range(core_version: str, minimum: str, maximum: str) -> bool:
    core = semver_tuple(core_version)
    min_version = semver_tuple(minimum)
    if core is None or min_version is None or core < min_version:
        return False
    if maximum.endswith(".x"):
        max_parts = maximum.split(".")
        return len(max_parts) == 3 and max_parts[0].isdigit() and max_parts[1].isdigit() and core[:2] == (int(max_parts[0]), int(max_parts[1]))
    max_version = semver_tuple(maximum)
    return max_version is not None and core <= max_version


def static_checks(policy: MatrixPolicy, channels: list[Channel], compatibility: list[SdkCompatibility]) -> list[CheckResult]:
    by_id = channel_map(channels)
    observed = datetime.fromisoformat(policy.observed_utc.replace("Z", "+00:00"))
    age_days = (datetime.now(UTC) - observed).days
    results = [
        CheckResult(0 <= age_days <= policy.max_staleness_days, "matrix freshness", f"age_days={age_days}"),
        CheckResult(set(by_id) == set(REQUIRED_CHANNELS) and len(by_id) == len(channels), "channel IDs", "required and unique"),
        CheckResult(all((c.owner and c.dry_run_command and c.publish_command and c.rollback and c.human_gate) for c in channels), "channel governance", "owner/dry-run/publish/rollback/human_gate present"),
        CheckResult(bool(policy.versioning_policy and policy.rollback_limitation and policy.human_gate), "matrix governance", "present"),
        CheckResult(by_id["core"].local_version == local_version_from_cargo() == policy.core_version, "local core version", f"matrix={by_id['core'].local_version}"),
        CheckResult(by_id["pypi"].local_version == local_version_from_pyproject(), "local python sdk version", f"matrix={by_id['pypi'].local_version}"),
        CheckResult(by_id["npm"].local_version == local_version_from_package_json(), "local typescript sdk version", f"matrix={by_id['npm'].local_version}"),
        CheckResult(f"## [{by_id['changelog'].observed_version}]" in read_text("CHANGELOG.md"), "changelog section", f"matrix={by_id['changelog'].observed_version}"),
        CheckResult(str(by_id["docs"].observed_version) in read_text("docs/GETTING_STARTED.md"), "docs version example", f"matrix={by_id['docs'].observed_version}"),
    ]
    compat_by_sdk = {row.sdk: row for row in compatibility}
    results.append(CheckResult(set(compat_by_sdk) == set(REQUIRED_SDKS) and len(compatibility) == len(REQUIRED_SDKS), "SDK compatibility rows", "python+typescript required and unique"))
    for sdk, channel_id in (("python", "pypi"), ("typescript", "npm")):
        row = compat_by_sdk.get(sdk)
        channel = by_id[channel_id]
        ok = row is not None and row.package == channel.package and row.local_version == channel.local_version and row.observed_registry_version == channel.observed_version and not row.publish_with_core and core_in_range(policy.core_version, row.compatible_core_min, row.compatible_core_max)
        detail = "missing" if row is None else f"package={row.package} local={row.local_version} registry={row.observed_registry_version} core={row.compatible_core_min}..{row.compatible_core_max}"
        results.append(CheckResult(ok, f"{sdk} independent versioning", detail))
    return results


def live_checks(policy: MatrixPolicy, channels: list[Channel], timeout_seconds: int) -> list[CheckResult]:
    by_id = channel_map(channels)
    tag = by_id["git_tag"]
    release = by_id["github_release"]
    cargo = by_id["crates_io"]
    pypi = by_id["pypi"]
    npm = by_id["npm"]
    brew = by_id["homebrew"]
    target_tag = tag.target or policy.core_tag
    latest_tag = tag.observed_latest or policy.latest_released_tag
    results = [
        CheckResult(tag_exists(target_tag, timeout_seconds) is tag.expected_present, f"git tag {target_tag}", f"expected_present={tag.expected_present}"),
        CheckResult(tag_exists(latest_tag, timeout_seconds), f"git tag {latest_tag}", "expected present"),
    ]
    gh_target = run_command(["gh", "release", "view", release.target or policy.core_tag, "--repo", policy.repository, "--json", "tagName"], timeout_seconds)
    gh_latest = run_command(["gh", "release", "view", release.observed_latest or policy.latest_released_tag, "--repo", policy.repository, "--json", "tagName"], timeout_seconds)
    results.extend([
        CheckResult((gh_target.returncode == 0) is release.expected_present, f"GitHub release {release.target}", f"expected_present={release.expected_present}"),
        CheckResult(gh_latest.returncode == 0 and f'"{release.observed_latest}"' in gh_latest.stdout, f"GitHub release {release.observed_latest}", "expected present"),
        CheckResult(version_matches(run_command(["cargo", "search", cargo.package or "", "--limit", "5"], timeout_seconds), lambda text: parse_exact_version_line(text, cargo.package or ""), cargo.observed_version or ""), f"crates.io {cargo.package}", f"matrix={cargo.observed_version}"),
        CheckResult(version_matches(fetch_pypi_package(pypi.package or "", timeout_seconds), lambda text: parse_pypi_json_version(text, pypi.package or ""), pypi.observed_version or ""), f"PyPI {pypi.package}", f"matrix={pypi.observed_version}"),
        CheckResult(version_matches(run_command(["npm", "view", npm.package or "", "version", "--json"], timeout_seconds), parse_npm_version, npm.observed_version or ""), f"npm {npm.package}", f"matrix={npm.observed_version}"),
        CheckResult(version_matches(run_command(["brew", "info", "--json=v2", brew.package or ""], timeout_seconds), lambda text: parse_homebrew_version(text, brew.package or ""), brew.observed_version or ""), f"Homebrew {brew.package}", f"matrix={brew.observed_version}"),
    ])
    return results


def print_results(results: list[CheckResult]) -> int:
    failed = [result for result in results if not result.ok]
    for result in results:
        print(f"{'OK' if result.ok else 'FAIL'}: {result.label} — {result.detail}")
    return 1 if failed else 0


def self_test_nonexistent_tag(timeout_seconds: int) -> int:
    if tag_exists(MISSING_SELF_TEST_TAG, timeout_seconds):
        print(f"UNEXPECTED PASS: nonexistent tag {MISSING_SELF_TEST_TAG} exists", file=sys.stderr)
        return 2
    print(f"EXPECTED FAIL: nonexistent tag {MISSING_SELF_TEST_TAG} is absent", file=sys.stderr)
    return 1


def self_test_parser_hardening() -> int:
    cargo = "warning: mirror says engram-core = \"9.9.9\"\nnot-engram-core = \"0.21.1\""
    pypi = '{"info":{"name":"not-engram-client","version":"0.4.0"}}'
    npm = '{"version":"9.9.9"}'
    return print_results([CheckResult(parse_exact_version_line(cargo, "engram-core") is None and parse_pypi_json_version(pypi, "engram-client") is None and parse_npm_version(npm) is None, "untrusted registry parser", "misleading success output rejected")])


def self_test_timeout() -> int:
    result = run_command([sys.executable, "-c", "import time; time.sleep(2)"], 1)
    pypi_result = fetch_pypi_package("engram-client", 0)
    return print_results([
        CheckResult(result.returncode == 124, "bounded timeout", "sleep command timed out"),
        CheckResult(pypi_result.returncode == 124, "bounded PyPI probe timeout", "worker process timed out"),
    ])


def self_test_wrong_matrix() -> int:
    policy, channels, compatibility = parse_matrix(DEFAULT_MATRIX)
    wrong = MatrixPolicy(policy.observed_utc, policy.max_staleness_days, policy.repository, "9.9.9", policy.core_tag, policy.latest_released_tag, policy.versioning_policy, policy.rollback_limitation, policy.human_gate)
    return 0 if print_results(static_checks(wrong, channels, compatibility)) != 0 else 1


def self_test_future_timestamp() -> int:
    policy, channels, compatibility = parse_matrix(DEFAULT_MATRIX)
    future = MatrixPolicy("2099-01-01T00:00:00Z", policy.max_staleness_days, policy.repository, policy.core_version, policy.core_tag, policy.latest_released_tag, policy.versioning_policy, policy.rollback_limitation, policy.human_gate)
    return 0 if print_results(static_checks(future, channels, compatibility)) != 0 else 1


def self_test_failed_registry_command() -> int:
    checks = [
        CheckResult(not version_matches(CommandResult(1, 'engram-core = "0.21.1"\n', "fail"), lambda text: parse_exact_version_line(text, "engram-core"), "0.21.1"), "fake cargo failure", "rejected"),
        CheckResult(not version_matches(CommandResult(1, '{"info":{"name":"engram-client","version":"0.4.0"}}', "fail"), lambda text: parse_pypi_json_version(text, "engram-client"), "0.4.0"), "fake PyPI failure", "rejected"),
        CheckResult(not version_matches(CommandResult(1, '"0.3.0"', "fail"), parse_npm_version, "0.3.0"), "fake npm failure", "rejected"),
        CheckResult(not version_matches(CommandResult(1, '{"formulae":[{"full_name":"aiconnai/engram/engram","versions":{"stable":"0.21.2"}}]}', "fail"), lambda text: parse_homebrew_version(text, "aiconnai/engram/engram"), "0.21.2"), "fake brew failure", "rejected"),
    ]
    return print_results(checks)


def self_test_sdk_compatibility_required() -> int:
    policy, channels, compatibility = parse_matrix(DEFAULT_MATRIX)
    wrong = [[], [row for row in compatibility if row.sdk == "python"], [SdkCompatibility("ruby", "engram-client", "0.5.0", "0.1.0", "0.20.0", "0.22.x", "wrong sdk", False)], [compatibility[0], compatibility[0], compatibility[1]]]
    checks = [CheckResult(print_results(static_checks(policy, channels, rows)) != 0, "sdk compatibility required", f"case={index}") for index, rows in enumerate(wrong, start=1)]
    return print_results(checks)

def self_test_sdk_package_and_range() -> int:
    policy, channels, compatibility = parse_matrix(DEFAULT_MATRIX)
    wrong_package = [SdkCompatibility("python", "wrong-client", "0.5.0", "0.4.0", "0.20.0", "0.22.x", "wrong package", False), compatibility[1]]
    high_min = [SdkCompatibility("python", "engram-client", "0.5.0", "0.4.0", "0.23.0", "0.24.x", "bad min", False), compatibility[1]]
    low_max = [compatibility[0], SdkCompatibility("typescript", "engram-client", "0.5.0", "0.3.0", "0.20.0", "0.21.x", "bad max", False)]
    rows = (("wrong package", wrong_package), ("high min", high_min), ("low max", low_max))
    checks = [CheckResult(print_results(static_checks(policy, channels, rowset)) != 0, "sdk package/range required", name) for name, rowset in rows]
    return print_results(checks)


def self_test_missing_publish_with_core() -> int:
    matrix_text = DEFAULT_MATRIX.read_text(encoding="utf-8").replace("\npublish_with_core = false", "", 1)
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".toml", delete=False) as handle:
        handle.write(matrix_text)
        matrix_path = Path(handle.name)
    try:
        try:
            parse_matrix(matrix_path)
        except KeyError as error:
            ok = error.args == ("publish_with_core",)
            detail = "missing field rejected"
        else:
            ok = False
            detail = "missing field accepted"
        return print_results([CheckResult(ok, "publish_with_core required", detail)])
    finally:
        matrix_path.unlink(missing_ok=True)


def self_test_read_only_no_pycache(timeout_seconds: int) -> int:
    pycache = Path(__file__).resolve().parent / "__pycache__"
    if pycache.exists():
        shutil.rmtree(pycache)
    script = Path(__file__).resolve().parents[1] / "check-release-channels.py"
    result = run_command([sys.executable, str(script), "--matrix", str(DEFAULT_MATRIX), "--read-only"], timeout_seconds)
    created = pycache.exists()
    if created:
        shutil.rmtree(pycache)
    ran_checker = "matrix freshness" in result.stdout
    detail = f"child_exit={result.returncode} ran_checker={ran_checker} pycache_created={created}"
    return print_results([CheckResult(ran_checker and not created and not pycache.exists(), "read-only leaves no pycache", detail)])

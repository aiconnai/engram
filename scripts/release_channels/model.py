"""Parsed release-channel matrix model and read-only probes."""

from __future__ import annotations

import json
import re
import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path
from typing import Callable

DEFAULT_MATRIX = Path("docs/releases/channel-matrix.toml")
REMOTE_TAG_URL = "https://github.com/aiconnai/engram.git"
MISSING_SELF_TEST_TAG = "v0.0.0-engram-nonexistent-self-test"
REQUIRED_CHANNELS = ("core", "git_tag", "github_release", "crates_io", "pypi", "npm", "homebrew", "docs", "changelog")
REQUIRED_SDKS = ("python", "typescript")


@dataclass(frozen=True, slots=True)
class CommandResult:
    returncode: int
    stdout: str
    stderr: str


@dataclass(frozen=True, slots=True)
class CheckResult:
    ok: bool
    label: str
    detail: str


@dataclass(frozen=True, slots=True)
class MatrixPolicy:
    observed_utc: str
    max_staleness_days: int
    repository: str
    core_version: str
    core_tag: str
    latest_released_tag: str
    versioning_policy: str
    rollback_limitation: str
    human_gate: str


@dataclass(frozen=True, slots=True)
class Channel:
    id: str
    kind: str
    owner: str
    dry_run_command: str
    publish_command: str
    rollback: str
    human_gate: bool
    package: str | None
    target: str | None
    local_source: str | None
    local_version: str | None
    observed_version: str | None
    observed_latest: str | None
    expected_present: bool | None


@dataclass(frozen=True, slots=True)
class SdkCompatibility:
    sdk: str
    package: str
    local_version: str
    observed_registry_version: str
    compatible_core_min: str
    compatible_core_max: str
    compatibility_basis: str
    publish_with_core: bool


def run_command(command: list[str], timeout_seconds: int) -> CommandResult:
    try:
        completed = subprocess.run(command, check=False, capture_output=True, text=True, timeout=timeout_seconds)
    except subprocess.TimeoutExpired as error:
        return CommandResult(124, error.stdout or "", error.stderr or "timeout")
    except OSError as error:
        return CommandResult(127, "", str(error))
    return CommandResult(completed.returncode, completed.stdout, completed.stderr)


def parse_exact_version_line(text: str, package: str) -> str | None:
    match = re.search(rf"^{re.escape(package)}\s*=\s*\"([^\"]+)\"", text, re.MULTILINE)
    return None if match is None else match.group(1)


def parse_pip_index_version(text: str, package: str) -> str | None:
    match = re.search(rf"^{re.escape(package)} \(([^)]+)\)$", text, re.MULTILINE)
    return None if match is None else match.group(1)


def parse_npm_version(text: str) -> str | None:
    try:
        value = json.loads(text)
    except json.JSONDecodeError:
        return None
    return value if isinstance(value, str) else None


def parse_homebrew_version(text: str, formula: str) -> str | None:
    try:
        payload = json.loads(text)
    except json.JSONDecodeError:
        return None
    formulae = payload.get("formulae") if isinstance(payload, dict) else None
    if not isinstance(formulae, list):
        return None
    for item in formulae:
        versions = item.get("versions") if isinstance(item, dict) and item.get("full_name") == formula else None
        if isinstance(versions, dict) and isinstance(versions.get("stable"), str):
            return versions["stable"]
    return None


def version_matches(result: CommandResult, parser: Callable[[str], str | None], expected: str) -> bool:
    return result.returncode == 0 and parser(result.stdout) == expected


def read_text(path: str) -> str:
    return Path(path).read_text(encoding="utf-8")


def local_version_from_cargo() -> str | None:
    match = re.search(r'^version = "([^"]+)"$', read_text("Cargo.toml"), re.MULTILINE)
    return None if match is None else match.group(1)


def local_version_from_pyproject() -> str | None:
    match = re.search(r'^version = "([^"]+)"$', read_text("sdks/python/pyproject.toml"), re.MULTILINE)
    return None if match is None else match.group(1)


def local_version_from_package_json() -> str | None:
    value = json.loads(read_text("sdks/typescript/package.json")).get("version")
    return value if isinstance(value, str) else None


def text_field(table, key: str) -> str:
    value = table.get(key)
    if isinstance(value, str) and value:
        return value
    raise KeyError(key)


def optional_text(table, key: str) -> str | None:
    value = table.get(key)
    return value if isinstance(value, str) else None


def optional_bool(table, key: str) -> bool | None:
    value = table.get(key)
    return value if isinstance(value, bool) else None


def bool_field(table, key: str) -> bool:
    value = table.get(key)
    if isinstance(value, bool):
        return value
    raise KeyError(key)


def parse_matrix(matrix_path: Path) -> tuple[MatrixPolicy, list[Channel], list[SdkCompatibility]]:
    with matrix_path.open("rb") as handle:
        payload = tomllib.load(handle)
    matrix = payload.get("matrix")
    raw_channels = payload.get("channels")
    raw_compat = payload.get("sdk_compatibility")
    if not isinstance(matrix, dict) or not isinstance(raw_channels, list) or not isinstance(raw_compat, list):
        missing_tables = "matrix/channels/sdk_compatibility"
        raise KeyError(missing_tables)
    max_age = matrix.get("max_staleness_days")
    if not isinstance(max_age, int):
        missing_max_age = "max_staleness_days"
        raise KeyError(missing_max_age)
    policy = MatrixPolicy(*(text_field(matrix, key) for key in ("observed_utc",)), max_age, *(text_field(matrix, key) for key in ("repository", "core_version", "core_tag", "latest_released_tag", "versioning_policy", "rollback_limitation", "human_gate")))
    channels = [Channel(text_field(row, "id"), text_field(row, "kind"), text_field(row, "owner"), text_field(row, "dry_run_command"), text_field(row, "publish_command"), text_field(row, "rollback"), row.get("human_gate") is True, optional_text(row, "package"), optional_text(row, "target"), optional_text(row, "local_source"), optional_text(row, "local_version"), optional_text(row, "observed_version"), optional_text(row, "observed_latest"), optional_bool(row, "expected_present")) for row in raw_channels if isinstance(row, dict)]
    compat = [SdkCompatibility(text_field(row, "sdk"), text_field(row, "package"), text_field(row, "local_version"), text_field(row, "observed_registry_version"), text_field(row, "compatible_core_min"), text_field(row, "compatible_core_max"), text_field(row, "compatibility_basis"), bool_field(row, "publish_with_core")) for row in raw_compat if isinstance(row, dict)]
    return policy, channels, compat


def channel_map(channels: list[Channel]) -> dict[str, Channel]:
    return {channel.id: channel for channel in channels}


def tag_exists(tag: str, timeout_seconds: int = 20) -> bool:
    result = run_command(["git", "ls-remote", "--tags", REMOTE_TAG_URL, f"refs/tags/{tag}"], timeout_seconds)
    return result.returncode == 0 and f"refs/tags/{tag}" in result.stdout

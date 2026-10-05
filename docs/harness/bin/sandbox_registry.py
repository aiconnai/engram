#!/usr/bin/env python3
"""Check registry and argv policy of the sandbox adapter (task H3, split out of sandbox-adapter.py).

Loads docs/harness/checks/registry.json strictly (duplicate keys, extra keys, unknown types and
unpinned images are refused), cross-checks every check ID against the H2 check catalog, and enforces
the argv policy: a registrable argv is a list of plain strings that is not a shell string, an
inline-code invocation or a wrapper (env, timeout, xargs, ...). Pure policy: no process is spawned here.
"""

from __future__ import annotations

import importlib.util
import re
import sys
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Dict, List, Mapping, Optional, Sequence, Tuple

HARNESS_DIR = Path(__file__).resolve().parent.parent

REGISTRY_VERSION = "check-registry-v1"
POLICY_VERSION_RE = re.compile(r"[a-z0-9][a-z0-9._-]{0,63}")
IMAGE_RE = re.compile(r"[a-z0-9][a-z0-9._/-]*@sha256:[0-9a-f]{64}")
CHECK_ID_RE = re.compile(r"[a-zA-Z0-9_:.-]{1,128}")
SHELLS = {"sh", "bash", "zsh", "dash", "ksh", "csh", "tcsh", "fish", "ash", "busybox"}
INLINE_INTERPRETERS = {"perl", "ruby", "node", "nodejs", "php", "lua"}
# Wrappers re-exec an arbitrary program (so `env bash -c ...` would hide inline code) and programs whose
# argument is itself code (awk, sed): never registrable. Register the final script instead.
FORBIDDEN_ARGV0 = {
    "eval", "exec", "sudo", "su", "doas", "env", "xargs", "nsenter", "docker", "podman", "chroot", "timeout", "nice",
    "setsid", "stdbuf", "nohup", "time", "command", "builtin", "ionice", "taskset", "unshare", "strace", "flock",
    "runuser", "script", "watch", "awk", "gawk", "sed",
}
# Prefix matches: the code may be attached to the flag (-cCODE, -e'puts 1', -pe...). Long flags may carry =value.
SHELL_INLINE_RE = re.compile(r"-[A-Za-z]*c")
INTERPRETER_INLINE_RE = re.compile(r"-[A-Za-z]*[ceEpr]")
LONG_INLINE_FLAGS = {"--command", "--eval", "--print", "--run"}
_VALIDATOR: Any = None


class Refusal(Exception):
    """Input or policy problem: the run must not start. Maps to status `refused`."""


class RegistryError(Refusal):
    """The check registry is malformed or violates policy."""


# --- Reuse of the H2 validator (strict JSON loader, task-v2 validation, check catalog) ---

def validator() -> Any:
    global _VALIDATOR
    if _VALIDATOR is None:
        path = HARNESS_DIR / "bin" / "validate-evidence.py"
        spec = importlib.util.spec_from_file_location("validate_evidence", path)
        if spec is None or spec.loader is None:
            raise Refusal(f"validator not loadable: {path}")
        module = importlib.util.module_from_spec(spec)
        sys.modules["validate_evidence"] = module
        spec.loader.exec_module(module)
        _VALIDATOR = module
    return _VALIDATOR


# --- Registry ---

@dataclass(frozen=True)
class SandboxConfig:
    image: str
    memory_mb: int
    pids_limit: int
    cpus: float
    tmpfs_mb: int
    log_cap_bytes: int
    max_total_timeout_seconds: int


@dataclass(frozen=True)
class CheckSpec:
    id: str
    argv: Tuple[str, ...]
    timeout_seconds: int
    note: str = ""


@dataclass(frozen=True)
class Registry:
    policy_version: str
    sandbox: SandboxConfig
    checks: Mapping[str, CheckSpec]


def is_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def _bounded_int(data: Mapping[str, Any], key: str, low: int, high: int, where: str) -> int:
    value = data.get(key)
    if not is_int(value) or not low <= value <= high:
        raise RegistryError(f"{where}.{key} must be an integer in [{low}, {high}]")
    return value


def validate_argv(check_id: str, argv: Any) -> Tuple[str, ...]:
    """A registrable argv is a non-empty list of plain strings that is not a shell/eval wrapper."""
    where = f"checks.{check_id}.argv"
    if not isinstance(argv, list) or not argv or len(argv) > 64:
        raise RegistryError(f"{where} must be a non-empty list (shell strings are not allowed)")
    for arg in argv:
        if not isinstance(arg, str) or "\0" in arg or len(arg) > 4096:
            raise RegistryError(f"{where} must contain only strings without NUL (max 4096 chars)")
    if not argv[0]:
        raise RegistryError(f"{where}[0] must not be empty")
    base = PurePosixPath(argv[0]).name
    if base in FORBIDDEN_ARGV0:
        raise RegistryError(f"{where}[0] '{base}' is a wrapper that is never registrable")
    is_python = re.fullmatch(r"python[0-9.]*", base) is not None
    if base in SHELLS or is_python:
        inline_re = SHELL_INLINE_RE
    elif base in INLINE_INTERPRETERS:
        inline_re = INTERPRETER_INLINE_RE
    else:
        inline_re = None
    if inline_re is not None:
        # Every argument is scanned (not only the leading options): fail closed over a false positive.
        for arg in argv[1:]:
            if arg.split("=", 1)[0] in LONG_INLINE_FLAGS or (not arg.startswith("--") and inline_re.match(arg)):
                raise RegistryError(f"{where} uses inline code ('{arg}'); register a script file instead")
    return tuple(argv)


def _parse_sandbox(raw: Any) -> SandboxConfig:
    expected = {"image", "memory_mb", "pids_limit", "cpus", "tmpfs_mb", "log_cap_bytes", "max_total_timeout_seconds"}
    if not isinstance(raw, dict) or set(raw) != expected:
        raise RegistryError(f"sandbox must be an object with exactly: {sorted(expected)}")
    image = raw["image"]
    if not isinstance(image, str) or not IMAGE_RE.fullmatch(image):
        raise RegistryError("sandbox.image must be pinned by digest (name@sha256:<64 hex>); tags are not accepted")
    cpus = raw["cpus"]
    if isinstance(cpus, bool) or not isinstance(cpus, (int, float)) or not 0.1 <= cpus <= 4:
        raise RegistryError("sandbox.cpus must be a number in [0.1, 4]")
    return SandboxConfig(
        image=image,
        memory_mb=_bounded_int(raw, "memory_mb", 16, 4096, "sandbox"),
        pids_limit=_bounded_int(raw, "pids_limit", 8, 512, "sandbox"),
        cpus=float(cpus),
        tmpfs_mb=_bounded_int(raw, "tmpfs_mb", 1, 512, "sandbox"),
        log_cap_bytes=_bounded_int(raw, "log_cap_bytes", 1024, 16 * 1024 * 1024, "sandbox"),
        max_total_timeout_seconds=_bounded_int(raw, "max_total_timeout_seconds", 1, 3600, "sandbox"),
    )


def parse_registry(data: Any, catalog_ids: Sequence[str]) -> Registry:
    if not isinstance(data, dict) or set(data) != {"registry_version", "policy_version", "sandbox", "checks"}:
        raise RegistryError("registry must be an object with exactly registry_version, policy_version, sandbox, checks")
    if data["registry_version"] != REGISTRY_VERSION:
        raise RegistryError(f"registry_version must be '{REGISTRY_VERSION}'")
    policy = data["policy_version"]
    if not isinstance(policy, str) or not POLICY_VERSION_RE.fullmatch(policy):
        raise RegistryError("policy_version is invalid")
    sandbox = _parse_sandbox(data["sandbox"])
    raw_checks = data["checks"]
    if not isinstance(raw_checks, dict) or not raw_checks:
        raise RegistryError("checks must be a non-empty object")
    checks: Dict[str, CheckSpec] = {}
    for check_id, entry in raw_checks.items():
        if not CHECK_ID_RE.fullmatch(check_id):
            raise RegistryError(f"invalid check id {check_id!r}")
        if check_id not in catalog_ids:
            raise RegistryError(f"check id '{check_id}' is not in the check catalog")
        if not isinstance(entry, dict) or not {"argv", "timeout_seconds"} <= set(entry) <= {"argv", "timeout_seconds", "note"}:
            raise RegistryError(f"checks.{check_id} must contain argv and timeout_seconds (and optionally note) only")
        note = entry.get("note", "")
        if not isinstance(note, str):
            raise RegistryError(f"checks.{check_id}.note must be a string")
        checks[check_id] = CheckSpec(
            id=check_id,
            argv=validate_argv(check_id, entry["argv"]),
            timeout_seconds=_bounded_int(entry, "timeout_seconds", 1, 3600, f"checks.{check_id}"),
            note=note,
        )
    return Registry(policy_version=policy, sandbox=sandbox, checks=checks)


def load_registry(registry_path: Path, catalog_path: Path) -> Registry:
    """Strictly load a registry and cross-check its check IDs against the check catalog."""
    catalog_ids, errors = validator().load_check_catalog(Path(catalog_path))
    if errors or catalog_ids is None:
        raise RegistryError("check catalog invalid: " + "; ".join(str(e) for e in errors[:2]))
    data, errors = validator().read_json_file(Path(registry_path))
    if errors:
        raise RegistryError("registry unreadable: " + "; ".join(str(e) for e in errors[:2]))
    return parse_registry(data, sorted(catalog_ids))

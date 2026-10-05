#!/usr/bin/env python3
"""validate-evidence.py

Standalone, offline validator for Engram agent harness task specifications, evidence receipts
and review reports.

TRUST BOUNDARY (read this first)
    `validate_file` / `validate_file_detailed` check STRUCTURE and SEMANTICS only. They never
    prove authenticity: a document that validates was not thereby produced by a trusted runner,
    reviewer or recorder, and nothing here upgrades a historical v1 artifact into trusted
    evidence. Facts that decide whether an artifact matches the candidate under evaluation
    (expected candidate / base / tree SHA, policy version, catalog hash, logs directory, task
    specification) are supplied by the CALLER through `Expectations` (CLI flags) and are never
    read from the payload as trusted. Comparing two fields written by the same author is a
    consistency check, not an origin check.

Fail-closed behaviour
    * Strict JSON: duplicate keys, NaN/Infinity/overflowing numbers, invalid UTF-8, BOM, files
      over MAX_FILE_BYTES, excessive nesting, symlinks and non-regular files are rejected.
    * The schema is chosen by `schema_version` (or an explicit, matching `--schema`), never by
      key markers or file names.
    * The built-in pure-Python validator supports a fixed keyword set. A schema using any other
      keyword (or a malformed one) is rejected, in both jsonschema modes.
    * `jsonschema`, when importable, is an independent cross-check using the same keyword set
      and the same strict integer rule. It can only turn a pass into a failure
      (`validator_divergence`); it never relaxes the built-in result. With or without it the
      verdict and the reported errors are identical.
    * bool is never an integer or number, 1.0 is not an integer, enum/const/uniqueItems do not
      conflate True with 1, and `$` in a pattern does not accept a trailing newline.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import math
import os
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional, Sequence, Set, Tuple

# jsonschema is optional. Tests simulate its absence with an import blocker
# (sys.modules["jsonschema"] = None), which lands in the ImportError branch below.
try:
    import jsonschema  # type: ignore

    HAS_JSONSCHEMA = True
except ImportError:
    jsonschema = None  # type: ignore
    HAS_JSONSCHEMA = False

VALIDATION_SCOPE = "structure_only"
MAX_FILE_BYTES = 1024 * 1024
MAX_JSON_DEPTH = 32
MAX_LOG_BYTES = 64 * 1024 * 1024
FUTURE_SKEW = datetime.timedelta(minutes=5)
MIN_YEAR = 2000

HEX_40_RE = re.compile(r"[0-9a-f]{40}")
HEX_64_RE = re.compile(r"[0-9a-f]{64}")
POLICY_RE = re.compile(r"[a-z0-9][a-z0-9._-]{0,63}")
TIMESTAMP_RE = re.compile(
    r"([0-9]{4})-([0-9]{2})-([0-9]{2})T([0-9]{2}):([0-9]{2}):([0-9]{2})(?:\.([0-9]{1,9}))?Z"
)
CHECK_ID_RE = re.compile(r"[a-zA-Z0-9_:.-]+")
DRIVE_RE = re.compile(r"[A-Za-z]:")
GLOB_CHARS = set("*?[]{}")
SHA_KEYS = ("commit_sha", "target_sha", "base_sha", "head_sha", "tree_sha")

KNOWN_SCHEMAS = ("task-v1", "evidence-v1", "review-v1", "task-v2", "evidence-v2", "review-v2")
SCHEMA_KIND = {name: name.rsplit("-", 1)[0] for name in KNOWN_SCHEMAS}
SHORT_SCHEMA_NAMES = ("task", "evidence", "review")
CATALOG_FILE = "check-catalog-v1.json"
CATALOG_VERSION = "check-catalog-v1"

CANDIDATE_FIELD = {"evidence": "commit_sha", "review": "head_sha", "task": "target_sha"}
# Expectations a caller must provide with --require-expectations, per schema.
REQUIRED_EXPECTATIONS = {
    "task-v1": ("candidate_sha",),
    "evidence-v1": ("candidate_sha",),
    "review-v1": ("candidate_sha",),
    "task-v2": ("candidate_sha", "policy_version"),
    "review-v2": ("candidate_sha", "policy_version"),
    "evidence-v2": ("candidate_sha", "policy_version", "catalog_sha256", "logs", "task"),
}
# Expectations that are meaningful for a schema; unapplied ones are listed in `notes`.
RECOMMENDED_EXPECTATIONS = {
    "task-v1": ("candidate_sha", "base_sha"),
    "evidence-v1": ("candidate_sha", "base_sha", "tree_sha"),
    "review-v1": ("candidate_sha", "base_sha"),
    "task-v2": ("candidate_sha", "base_sha", "policy_version"),
    "review-v2": ("candidate_sha", "base_sha", "policy_version"),
    "evidence-v2": ("candidate_sha", "base_sha", "tree_sha", "policy_version", "catalog_sha256", "logs", "task"),
}
EXPECTATION_KEYS = ("candidate_sha", "base_sha", "tree_sha", "policy_version", "catalog_sha256", "logs", "task")
EXPECTATION_FLAGS = {
    "candidate_sha": "--expect-candidate-sha",
    "base_sha": "--expect-base-sha",
    "tree_sha": "--expect-tree-sha",
    "policy_version": "--expect-policy-version",
    "catalog_sha256": "--expect-catalog-sha256",
    "logs": "--logs-dir",
    "task": "--task",
}

# Capabilities that are never granted to a writer.
DISALLOWED_CAPABILITIES = {
    "arbitrary_host_exec",
    "host_exec",
    "host_execution",
    "credential_mount",
    "credential_access",
    "unrestricted_network",
    "network_all",
    "root",
    "sudo",
    "production_mutation",
    "production_access",
}
APPROVED_CAPABILITIES = {
    "workspace_read",
    "workspace_write",
    "test_exec",
    "lint_exec",
    "format_exec",
    "doc_exec",
    "git_read",
    "sandbox_container",
    "network_none",
}
# v2 tasks that write or execute must run in a sandbox with networking disabled (ADR: network is
# off by default; there is no egress capability in the approved list).
EXEC_CAPABILITIES = {"workspace_write", "test_exec", "lint_exec", "format_exec", "doc_exec"}
ISOLATION_CAPABILITIES = {"sandbox_container", "network_none"}
BLOCKING_SEVERITIES = {"blocker", "critical", "high"}

SUPPORTED_DIALECTS = {
    "https://json-schema.org/draft/2020-12/schema",
    "http://json-schema.org/draft-07/schema#",
}
SUPPORTED_KEYWORDS = {
    "$schema", "$id", "title", "description",
    "type", "const", "enum",
    "pattern", "minLength", "maxLength",
    "minimum", "maximum",
    "minItems", "maxItems", "uniqueItems", "items",
    "required", "properties", "additionalProperties", "propertyNames",
    "minProperties", "maxProperties",
}
SCHEMA_TYPES = {"object", "array", "string", "integer", "number", "boolean", "null"}


# ---------------------------------------------------------------------------------------------
# Diagnostics
# ---------------------------------------------------------------------------------------------

class ValidationErrorItem:
    def __init__(self, code: str, field: str, message: str):
        self.code = code
        self.field = field
        self.message = message

    def to_dict(self) -> Dict[str, str]:
        return {"code": self.code, "field": self.field, "message": self.message}

    def __str__(self) -> str:
        if self.field:
            return f"[{self.code}] {self.field}: {self.message}"
        return f"[{self.code}] {self.message}"


def _err(code: str, field_name: str, message: str) -> ValidationErrorItem:
    return ValidationErrorItem(code, field_name, message)


def _join(path: str, key: Any) -> str:
    return f"{path}.{key}" if path else str(key)


def _is_int(value: Any) -> bool:
    return type(value) is int


# ---------------------------------------------------------------------------------------------
# Strict JSON
# ---------------------------------------------------------------------------------------------

class StrictJsonError(Exception):
    def __init__(self, code: str, message: str):
        super().__init__(message)
        self.code = code
        self.message = message


def _reject_constant(name: str) -> Any:
    raise StrictJsonError("non_finite_number", f"non-finite number literal '{name}' is not allowed")


def _parse_float(text: str) -> float:
    value = float(text)
    if not math.isfinite(value):
        raise StrictJsonError("non_finite_number", f"number '{text}' overflows to a non-finite value")
    return value


def _pairs_hook(pairs: List[Tuple[str, Any]]) -> Dict[str, Any]:
    result: Dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise StrictJsonError("duplicate_key", f"duplicate object key '{key}'")
        result[key] = value
    return result


def _depth(node: Any) -> int:
    depth = 1
    stack: List[Tuple[Any, int]] = [(node, 1)]
    while stack:
        current, level = stack.pop()
        depth = max(depth, level)
        if isinstance(current, dict):
            stack.extend((v, level + 1) for v in current.values() if isinstance(v, (dict, list)))
        elif isinstance(current, list):
            stack.extend((v, level + 1) for v in current if isinstance(v, (dict, list)))
    return depth


def parse_json_strict(text: str) -> Any:
    """Parse JSON text, rejecting duplicate keys, non-finite numbers and excessive nesting."""
    try:
        data = json.loads(
            text,
            object_pairs_hook=_pairs_hook,
            parse_constant=_reject_constant,
            parse_float=_parse_float,
        )
    except StrictJsonError:
        raise
    except RecursionError as exc:
        raise StrictJsonError("nesting_too_deep", "JSON nesting exceeds the parser limit") from exc
    except (ValueError, TypeError) as exc:
        raise StrictJsonError("json_decode_error", f"JSON parse error: {exc}") from exc
    if _depth(data) > MAX_JSON_DEPTH:
        raise StrictJsonError("nesting_too_deep", f"JSON nesting exceeds {MAX_JSON_DEPTH} levels")
    return data


def read_json_file(path: Path, max_bytes: int = MAX_FILE_BYTES) -> Tuple[Any, List[ValidationErrorItem]]:
    """Read one regular, non-symlink JSON file strictly. Returns (data, errors)."""
    label = str(path)
    try:
        if not os.path.lexists(path):
            return None, [_err("file_not_found", label, "file does not exist")]
        if path.is_symlink() or not path.is_file():
            return None, [_err("unsafe_input_file", label, "input must be a regular file, not a symlink or directory")]
        size = path.stat().st_size
        if size > max_bytes:
            return None, [_err("file_too_large", label, f"file is {size} bytes; limit is {max_bytes}")]
        raw = path.read_bytes()
        if len(raw) > max_bytes:
            return None, [_err("file_too_large", label, f"file exceeds {max_bytes} bytes")]
    except OSError as exc:
        return None, [_err("file_unreadable", label, f"cannot read file: {exc.strerror or exc}")]
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError as exc:
        return None, [_err("invalid_utf8", label, f"file is not valid UTF-8: {exc}")]
    try:
        return parse_json_strict(text), []
    except StrictJsonError as exc:
        return None, [_err(exc.code, label, exc.message)]


# ---------------------------------------------------------------------------------------------
# Built-in schema validator (draft 2020-12 subset used by the harness schemas)
# ---------------------------------------------------------------------------------------------

def json_equal(a: Any, b: Any) -> bool:
    """JSON equality where booleans are never numbers (True != 1)."""
    if isinstance(a, bool) or isinstance(b, bool):
        return isinstance(a, bool) and isinstance(b, bool) and a == b
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return a == b
    if isinstance(a, str) and isinstance(b, str):
        return a == b
    if a is None or b is None:
        return a is None and b is None
    if isinstance(a, list) and isinstance(b, list):
        return len(a) == len(b) and all(json_equal(x, y) for x, y in zip(a, b))
    if isinstance(a, dict) and isinstance(b, dict):
        return a.keys() == b.keys() and all(json_equal(a[k], b[k]) for k in a)
    return False


def _has_control_chars(value: str) -> bool:
    return any(ord(c) < 0x20 or ord(c) == 0x7F for c in value)


def find_unsupported_keywords(schema: Any, path: str = "") -> List[ValidationErrorItem]:
    """Static walk of a schema: any keyword outside SUPPORTED_KEYWORDS, or a malformed value, is an error."""
    errors: List[ValidationErrorItem] = []
    where = path or "<schema>"
    if not isinstance(schema, dict):
        return [_err("unsupported_schema_keyword", where, "schema nodes must be objects")]
    for key in schema:
        if key not in SUPPORTED_KEYWORDS:
            errors.append(_err("unsupported_schema_keyword", where, f"keyword '{key}' is not supported; failing closed"))
    if "$schema" in schema and schema["$schema"] not in SUPPORTED_DIALECTS:
        errors.append(_err("unsupported_schema_dialect", where, f"unsupported $schema dialect '{schema['$schema']}'"))
    if "type" in schema:
        t = schema["type"]
        if not isinstance(t, str) or t not in SCHEMA_TYPES:
            errors.append(_err("unsupported_schema_keyword", where, f"unsupported type '{t}'"))
    if "pattern" in schema:
        try:
            re.compile(schema["pattern"])
        except (re.error, TypeError) as exc:
            errors.append(_err("invalid_schema", where, f"invalid pattern: {exc}"))
    for key in ("minLength", "maxLength", "minItems", "maxItems", "minProperties", "maxProperties"):
        if key in schema and not _is_int(schema[key]):
            errors.append(_err("invalid_schema", where, f"{key} must be an integer"))
    for key in ("minimum", "maximum"):
        if key in schema and (isinstance(schema[key], bool) or not isinstance(schema[key], (int, float))):
            errors.append(_err("invalid_schema", where, f"{key} must be a number"))
    if "required" in schema and not (isinstance(schema["required"], list) and all(isinstance(r, str) for r in schema["required"])):
        errors.append(_err("invalid_schema", where, "required must be a list of strings"))
    if "enum" in schema and not isinstance(schema["enum"], list):
        errors.append(_err("invalid_schema", where, "enum must be a list"))
    if "uniqueItems" in schema and not isinstance(schema["uniqueItems"], bool):
        errors.append(_err("invalid_schema", where, "uniqueItems must be a boolean"))
    props = schema.get("properties")
    if "properties" in schema:
        if not isinstance(props, dict):
            errors.append(_err("invalid_schema", where, "properties must be an object"))
        else:
            for name, sub in props.items():
                errors.extend(find_unsupported_keywords(sub, _join(path, f"properties.{name}")))
    if "items" in schema:
        errors.extend(find_unsupported_keywords(schema["items"], _join(path, "items")))
    if "propertyNames" in schema:
        errors.extend(find_unsupported_keywords(schema["propertyNames"], _join(path, "propertyNames")))
    if "additionalProperties" in schema:
        ap = schema["additionalProperties"]
        if isinstance(ap, dict):
            errors.extend(find_unsupported_keywords(ap, _join(path, "additionalProperties")))
        elif not isinstance(ap, bool):
            errors.append(_err("invalid_schema", where, "additionalProperties must be a boolean or a schema"))
    return errors


class PureSchemaValidator:
    """Zero-dependency JSON Schema validator for SUPPORTED_KEYWORDS. Unknown keywords fail closed."""

    def __init__(self, schema: Dict[str, Any]):
        self.schema = schema

    def validate(self, instance: Any) -> List[ValidationErrorItem]:
        static_errors = find_unsupported_keywords(self.schema)
        if static_errors:
            return static_errors
        errors: List[ValidationErrorItem] = []
        self._node(instance, self.schema, "", errors)
        return errors

    def _node(self, instance: Any, schema: Dict[str, Any], path: str, errors: List[ValidationErrorItem]) -> None:
        where = path or "<root>"
        expected_type = schema.get("type")
        if expected_type is not None and not self._type_ok(instance, expected_type):
            errors.append(_err("type_mismatch", where, f"expected type '{expected_type}', got '{type(instance).__name__}'"))
            return
        if "const" in schema and not json_equal(instance, schema["const"]):
            errors.append(_err("const_mismatch", where, f"expected constant value '{schema['const']}', got '{instance}'"))
        if "enum" in schema and not any(json_equal(instance, option) for option in schema["enum"]):
            errors.append(_err("invalid_enum_value", where, f"value '{instance}' is not one of allowed enum values: {schema['enum']}"))
        if isinstance(instance, str):
            self._string(instance, schema, where, errors)
        elif isinstance(instance, (int, float)) and not isinstance(instance, bool):
            if "minimum" in schema and instance < schema["minimum"]:
                errors.append(_err("number_too_small", where, f"value {instance} is less than minimum {schema['minimum']}"))
            if "maximum" in schema and instance > schema["maximum"]:
                errors.append(_err("number_too_large", where, f"value {instance} exceeds maximum {schema['maximum']}"))
        elif isinstance(instance, list):
            self._array(instance, schema, path, where, errors)
        elif isinstance(instance, dict):
            self._object(instance, schema, path, where, errors)

    def _string(self, instance: str, schema: Dict[str, Any], where: str, errors: List[ValidationErrorItem]) -> None:
        if "pattern" in schema:
            if _has_control_chars(instance) or not re.search(schema["pattern"], instance):
                errors.append(_err("invalid_pattern", where, f"value {instance!r} does not match pattern '{schema['pattern']}'"))
        if "minLength" in schema and len(instance) < schema["minLength"]:
            errors.append(_err("string_too_short", where, f"string length {len(instance)} is shorter than minLength {schema['minLength']}"))
        if "maxLength" in schema and len(instance) > schema["maxLength"]:
            errors.append(_err("string_too_long", where, f"string length {len(instance)} exceeds maxLength {schema['maxLength']}"))

    def _array(self, instance: List[Any], schema: Dict[str, Any], path: str, where: str, errors: List[ValidationErrorItem]) -> None:
        if "minItems" in schema and len(instance) < schema["minItems"]:
            errors.append(_err("array_too_short", where, f"array has {len(instance)} items, expected at least {schema['minItems']}"))
        if "maxItems" in schema and len(instance) > schema["maxItems"]:
            errors.append(_err("array_too_long", where, f"array has {len(instance)} items, expected at most {schema['maxItems']}"))
        if schema.get("uniqueItems") and len(instance) <= 4096:
            for i, item in enumerate(instance):
                if any(json_equal(item, earlier) for earlier in instance[:i]):
                    errors.append(_err("duplicate_array_item", where, f"array contains duplicate item: {item}"))
                    break
        item_schema = schema.get("items")
        if isinstance(item_schema, dict):
            for idx, item in enumerate(instance):
                self._node(item, item_schema, f"{path}[{idx}]", errors)

    def _object(self, instance: Dict[str, Any], schema: Dict[str, Any], path: str, where: str, errors: List[ValidationErrorItem]) -> None:
        if "minProperties" in schema and len(instance) < schema["minProperties"]:
            errors.append(_err("object_too_small", where, f"object has {len(instance)} properties, expected at least {schema['minProperties']}"))
        if "maxProperties" in schema and len(instance) > schema["maxProperties"]:
            errors.append(_err("object_too_large", where, f"object has {len(instance)} properties, expected at most {schema['maxProperties']}"))
        for req in schema.get("required", []):
            if req not in instance:
                errors.append(_err("missing_required", _join(path, req), f"required field '{req}' is missing"))
        properties = schema.get("properties", {})
        additional = schema.get("additionalProperties", True)
        names_schema = schema.get("propertyNames")
        for key, value in instance.items():
            if isinstance(names_schema, dict):
                name_errors: List[ValidationErrorItem] = []
                self._node(key, names_schema, "", name_errors)
                if name_errors:
                    errors.append(_err("invalid_property_name", f"{path}[{key}]" if path else f"[{key}]", f"property name '{key}' violates its schema: {name_errors[0].message}"))
            if key in properties:
                self._node(value, properties[key], _join(path, key), errors)
            elif additional is False:
                errors.append(_err("unexpected_property", _join(path, key), f"unexpected additional property '{key}' not permitted by schema"))
            elif isinstance(additional, dict):
                self._node(value, additional, _join(path, key), errors)

    @staticmethod
    def _type_ok(value: Any, expected: str) -> bool:
        if expected == "object":
            return isinstance(value, dict)
        if expected == "array":
            return isinstance(value, list)
        if expected == "string":
            return isinstance(value, str)
        if expected == "integer":
            return _is_int(value)
        if expected == "number":
            return isinstance(value, (int, float)) and not isinstance(value, bool)
        if expected == "boolean":
            return isinstance(value, bool)
        if expected == "null":
            return value is None
        return False


def _jsonschema_errors(js_module: Any, schema: Dict[str, Any], data: Any) -> List[Any]:
    """Run jsonschema with the same strict integer rule as the built-in validator (1.0 is not an integer)."""
    base = js_module.validators.validator_for(schema)
    checker = base.TYPE_CHECKER.redefine("integer", lambda _checker, instance: _is_int(instance))
    strict = js_module.validators.extend(base, type_checker=checker)
    return list(strict(schema).iter_errors(data))


# ---------------------------------------------------------------------------------------------
# Paths, timestamps, SHAs
# ---------------------------------------------------------------------------------------------

def check_path(path_str: Any, context: str, allow_dir: bool = False) -> List[ValidationErrorItem]:
    """Repo-relative, normalized, glob-free path. Wildcards are never authorized."""
    if not isinstance(path_str, str):
        return []
    errors: List[ValidationErrorItem] = []
    if _has_control_chars(path_str):
        errors.append(_err("invalid_path_char", context, "path contains control characters"))
    if "\\" in path_str:
        errors.append(_err("invalid_path_char", context, "path contains a backslash"))
    if path_str == "" or path_str in (".", "./"):
        errors.append(_err("broad_path", context, f"path {path_str!r} would cover the whole repository"))
        return errors
    normalized = path_str.replace("\\", "/")
    if normalized.startswith("/") or DRIVE_RE.match(normalized):
        errors.append(_err("scope_violation", context, f"absolute path '{path_str}' is forbidden; must be repo-relative"))
    parts = normalized.split("/")
    if ".." in parts:
        errors.append(_err("scope_violation", context, f"path traversal '..' in '{path_str}' is forbidden"))
    if any(ch in GLOB_CHARS for ch in path_str):
        errors.append(_err("unauthorized_glob", context, f"glob pattern '{path_str}' is not authorized; list exact paths"))
    if normalized.strip("/") == "":
        errors.append(_err("broad_path", context, f"path {path_str!r} would cover the whole repository"))
        return errors
    body = parts[:-1] if (allow_dir and normalized.endswith("/")) else parts
    if normalized.startswith("/"):
        body = body[1:]
    if any(p in ("", ".") for p in body) or (not allow_dir and normalized.endswith("/")):
        errors.append(_err("unnormalized_path", context, f"path '{path_str}' is not normalized (empty or '.' component)"))
    return errors


def check_timestamp(value: Any, field_name: str, now: datetime.datetime) -> List[ValidationErrorItem]:
    if not isinstance(value, str):
        return [_err("invalid_timestamp", field_name, f"expected a string timestamp, got {type(value).__name__}")]
    match = TIMESTAMP_RE.fullmatch(value)
    if not match:
        return [_err("invalid_timestamp", field_name, f"expected canonical RFC 3339 UTC timestamp ending in 'Z', got {value!r}")]
    year, month, day, hour, minute, second = (int(match.group(i)) for i in range(1, 7))
    try:
        moment = datetime.datetime(year, month, day, hour, minute, second, tzinfo=datetime.timezone.utc)
    except ValueError as exc:
        return [_err("invalid_timestamp", field_name, f"impossible calendar date or time in {value!r}: {exc}")]
    if year < MIN_YEAR:
        return [_err("invalid_timestamp", field_name, f"year {year} is before {MIN_YEAR}")]
    if moment > now + FUTURE_SKEW:
        return [_err("timestamp_in_future", field_name, f"timestamp {value} is later than the validator clock plus {int(FUTURE_SKEW.total_seconds())}s")]
    return []


def check_shas(data: Mapping[str, Any]) -> List[ValidationErrorItem]:
    errors: List[ValidationErrorItem] = []
    for key in SHA_KEYS:
        if key not in data:
            continue
        val = data[key]
        if not isinstance(val, str) or not HEX_40_RE.fullmatch(val):
            errors.append(_err("invalid_sha", key, f"expected 40-character lowercase hexadecimal SHA, got {val!r}"))
        elif val == "0" * 40:
            errors.append(_err("null_sha", key, "the all-zero SHA is the null revision, not a candidate"))
    return errors


# ---------------------------------------------------------------------------------------------
# Semantic checks (keyed on the resolved schema, never on payload key markers)
# ---------------------------------------------------------------------------------------------

def _semantic_task(data: Mapping[str, Any], v2: bool, catalog: Optional[Set[str]]) -> List[ValidationErrorItem]:
    errors: List[ValidationErrorItem] = []
    caps = data.get("allowed_capabilities", [])
    cap_names: Set[str] = set()
    if isinstance(caps, list):
        for idx, cap in enumerate(caps):
            where = f"allowed_capabilities[{idx}]"
            if not isinstance(cap, str):
                continue
            cap_names.add(cap)
            if cap in DISALLOWED_CAPABILITIES:
                errors.append(_err("scope_violation", where, f"disallowed capability '{cap}' is strictly prohibited"))
            elif cap not in APPROVED_CAPABILITIES:
                errors.append(_err("unapproved_capability", where, f"capability '{cap}' is not in approved list: {sorted(APPROVED_CAPABILITIES)}"))
        if v2 and cap_names & EXEC_CAPABILITIES and not ISOLATION_CAPABILITIES <= cap_names:
            errors.append(_err("missing_isolation_capability", "allowed_capabilities", "write/exec capabilities require sandbox_container and network_none"))
    allowed = data.get("allowed_paths", [])
    protected = data.get("protected_paths", [])
    for key, paths in (("allowed_paths", allowed), ("protected_paths", protected)):
        if isinstance(paths, list):
            for idx, p in enumerate(paths):
                errors.extend(check_path(p, f"{key}[{idx}]", allow_dir=True))
    if v2:
        if isinstance(allowed, list) and isinstance(protected, list):
            for ai, a in enumerate(allowed):
                for p in protected:
                    if isinstance(a, str) and isinstance(p, str):
                        a_n, p_n = a.rstrip("/"), p.rstrip("/")
                        if a_n and p_n and (a_n == p_n or a_n.startswith(p_n + "/")):
                            errors.append(_err("path_conflict", f"allowed_paths[{ai}]", f"'{a}' is inside protected path '{p}'"))
        errors.extend(_unknown_check_ids(data.get("required_checks"), "required_checks", catalog, item_is_id=True))
    return errors


def _unknown_check_ids(items: Any, field_name: str, catalog: Optional[Set[str]], item_is_id: bool) -> List[ValidationErrorItem]:
    if catalog is None or not isinstance(items, list):
        return []
    errors: List[ValidationErrorItem] = []
    for idx, item in enumerate(items):
        cid = item if item_is_id else (item.get("id") if isinstance(item, dict) else None)
        if isinstance(cid, str) and cid not in catalog:
            suffix = "" if item_is_id else ".id"
            errors.append(_err("unknown_check_id", f"{field_name}[{idx}]{suffix}", f"check id '{cid}' is not in the check catalog"))
    return errors


def _semantic_evidence(data: Mapping[str, Any], v2: bool, catalog: Optional[Set[str]]) -> List[ValidationErrorItem]:
    errors: List[ValidationErrorItem] = []
    manifest = data.get("sha256_manifest", {})
    if isinstance(manifest, dict):
        for path_key, digest in manifest.items():
            errors.extend(check_path(path_key, f"sha256_manifest['{path_key}']"))
            if not isinstance(digest, str) or not HEX_64_RE.fullmatch(digest):
                errors.append(_err("invalid_digest", f"sha256_manifest['{path_key}']", f"expected 64-character lowercase hex SHA-256 digest, got {digest!r}"))
    checks = data.get("checks")
    verdict = data.get("verdict")
    if isinstance(checks, list):
        seen_ids: Set[str] = set()
        seen_logs: Set[str] = set()
        statuses: List[str] = []
        for idx, chk in enumerate(checks):
            if not isinstance(chk, dict):
                continue
            cid, status, exit_code = chk.get("id"), chk.get("status"), chk.get("exit_code")
            if isinstance(cid, str):
                if cid in seen_ids:
                    errors.append(_err("duplicate_check_id", f"checks[{idx}].id", f"check id '{cid}' appears more than once"))
                seen_ids.add(cid)
            if isinstance(status, str):
                statuses.append(status)
                if _is_int(exit_code):
                    if status == "pass" and exit_code != 0:
                        errors.append(_err("status_exit_mismatch", f"checks[{idx}]", f"status 'pass' with exit_code {exit_code}"))
                    if status in ("fail", "timeout") and exit_code == 0:
                        errors.append(_err("status_exit_mismatch", f"checks[{idx}]", f"status '{status}' with exit_code 0"))
            if v2:
                lp = chk.get("log_path")
                if isinstance(lp, str):
                    errors.extend(check_path(lp, f"checks[{idx}].log_path"))
                    if lp in seen_logs:
                        errors.append(_err("duplicate_log_path", f"checks[{idx}].log_path", f"log_path '{lp}' is shared by several checks"))
                    seen_logs.add(lp)
        if v2:
            errors.extend(_unknown_check_ids(checks, "checks", catalog, item_is_id=False))
        errors.extend(_verdict_vs_checks(verdict, statuses))
    return errors


def _verdict_vs_checks(verdict: Any, statuses: Sequence[str]) -> List[ValidationErrorItem]:
    if not isinstance(verdict, str) or not statuses:
        return []
    v = verdict.lower()
    bad = [s for s in statuses if s in ("fail", "timeout")]
    errors: List[ValidationErrorItem] = []
    if v == "pass":
        if any(s in ("fail", "timeout", "warn") for s in statuses):
            errors.append(_err("verdict_check_mismatch", "verdict", "verdict 'pass' but a check failed, timed out or warned"))
        if "skipped" in statuses:
            errors.append(_err("skipped_check_in_pass", "verdict", "verdict 'pass' is not allowed while a check was skipped"))
    elif v == "warn":
        if bad or not any(s in ("warn", "skipped") for s in statuses):
            errors.append(_err("verdict_check_mismatch", "verdict", "verdict 'warn' needs a warned/skipped check and no failed check"))
    elif v == "fail":
        if not bad:
            errors.append(_err("verdict_check_mismatch", "verdict", "verdict 'fail' but no check failed or timed out"))
    return errors


def _semantic_review(data: Mapping[str, Any]) -> List[ValidationErrorItem]:
    errors: List[ValidationErrorItem] = []
    findings = data.get("findings")
    verdict = data.get("verdict")
    verdict_l = verdict.lower() if isinstance(verdict, str) else None
    blocking = 0
    if isinstance(findings, list):
        seen: Set[str] = set()
        for idx, f in enumerate(findings):
            if not isinstance(f, dict):
                continue
            errors.extend(check_path(f.get("path"), f"findings[{idx}].path"))
            fid = f.get("finding_id")
            if isinstance(fid, str):
                if fid in seen:
                    errors.append(_err("duplicate_finding_id", f"findings[{idx}].finding_id", f"finding id '{fid}' appears more than once"))
                seen.add(fid)
            if f.get("severity") in BLOCKING_SEVERITIES:
                blocking += 1
        if verdict_l == "pass" and blocking:
            errors.append(_err("pass_with_blocking_finding", "verdict", f"verdict 'pass' but {blocking} blocking finding(s) (blocker/critical/high)"))
        if verdict_l == "comment" and blocking:
            errors.append(_err("comment_with_blocking_finding", "verdict", f"verdict 'comment' but {blocking} blocking finding(s)"))
        if verdict_l in ("fail", "request_changes") and not findings:
            errors.append(_err("fail_without_findings", "verdict", f"verdict '{verdict_l}' without any finding to act on"))
    return errors


def run_semantic_checks(
    data: Mapping[str, Any],
    schema_name: str,
    now: Optional[datetime.datetime] = None,
    catalog: Optional[Set[str]] = None,
) -> List[ValidationErrorItem]:
    """Fail-closed semantic rules beyond JSON Schema typing; keyed on the resolved schema."""
    now = now or datetime.datetime.now(datetime.timezone.utc)
    kind = SCHEMA_KIND.get(schema_name)
    v2 = schema_name.endswith("-v2")
    errors: List[ValidationErrorItem] = list(check_shas(data))
    if "timestamp" in data:
        errors.extend(check_timestamp(data["timestamp"], "timestamp", now))
    if kind == "task":
        errors.extend(_semantic_task(data, v2, catalog))
    elif kind == "evidence":
        errors.extend(_semantic_evidence(data, v2, catalog))
    elif kind == "review":
        errors.extend(_semantic_review(data))
    return errors


# ---------------------------------------------------------------------------------------------
# External expectations (supplied by the caller, never by the payload)
# ---------------------------------------------------------------------------------------------

@dataclass(frozen=True)
class Expectations:
    candidate_sha: Optional[str] = None
    base_sha: Optional[str] = None
    tree_sha: Optional[str] = None
    policy_version: Optional[str] = None
    catalog_sha256: Optional[str] = None
    catalog_path: Optional[Path] = None
    logs_dir: Optional[Path] = None
    task: Optional[Mapping[str, Any]] = None
    now: Optional[datetime.datetime] = None

    def __post_init__(self) -> None:
        for name in ("candidate_sha", "base_sha", "tree_sha"):
            val = getattr(self, name)
            if val is not None and not (isinstance(val, str) and HEX_40_RE.fullmatch(val)):
                raise ValueError(f"{name} must be a 40-character lowercase hex SHA, got {val!r}")
        if self.catalog_sha256 is not None and not (isinstance(self.catalog_sha256, str) and HEX_64_RE.fullmatch(self.catalog_sha256)):
            raise ValueError("catalog_sha256 must be a 64-character lowercase hex SHA-256")
        if self.policy_version is not None and not (isinstance(self.policy_version, str) and POLICY_RE.fullmatch(self.policy_version)):
            raise ValueError(f"policy_version must match {POLICY_RE.pattern}, got {self.policy_version!r}")
        if self.now is not None and self.now.tzinfo is None:
            raise ValueError("now must be timezone-aware")

    def provided(self, key: str) -> bool:
        if key == "logs":
            return self.logs_dir is not None
        if key == "task":
            return self.task is not None
        return getattr(self, key) is not None


@dataclass
class ValidationResult:
    ok: bool
    schema: str
    errors: List[ValidationErrorItem]
    notes: List[str] = field(default_factory=list)
    applied: Dict[str, bool] = field(default_factory=lambda: {k: False for k in EXPECTATION_KEYS})
    trust: str = VALIDATION_SCOPE

    @property
    def expectations_complete(self) -> bool:
        required = REQUIRED_EXPECTATIONS.get(self.schema)
        return bool(required) and all(self.applied.get(k) for k in required)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "ok": self.ok,
            "schema": self.schema,
            "trust": self.trust,
            "errors": [e.to_dict() for e in self.errors],
            "notes": list(self.notes),
            "expectations_applied": dict(self.applied),
            "expectations_complete": self.expectations_complete,
        }


def load_check_catalog(path: Path) -> Tuple[Optional[Set[str]], List[ValidationErrorItem]]:
    """Strictly load a check catalog: {"catalog_version": "check-catalog-v1", "checks": {id: {"description": str}}}."""
    data, errors = read_json_file(path)
    if errors:
        return None, [_err("catalog_invalid", str(path), str(e)) for e in errors]
    problems: List[str] = []
    if not isinstance(data, dict) or set(data) != {"catalog_version", "checks"}:
        problems.append("catalog must be an object with exactly 'catalog_version' and 'checks'")
    else:
        if data["catalog_version"] != CATALOG_VERSION:
            problems.append(f"catalog_version must be '{CATALOG_VERSION}'")
        checks = data["checks"]
        if not isinstance(checks, dict) or not checks:
            problems.append("'checks' must be a non-empty object")
        else:
            for cid, entry in checks.items():
                if not CHECK_ID_RE.fullmatch(cid) or len(cid) > 128:
                    problems.append(f"invalid check id {cid!r}")
                if not (isinstance(entry, dict) and set(entry) == {"description"} and isinstance(entry["description"], str) and entry["description"]):
                    problems.append(f"entry for '{cid}' must be {{\"description\": non-empty string}}")
    if problems:
        return None, [_err("catalog_invalid", str(path), "; ".join(problems))]
    return set(data["checks"]), []


def _sha256_file(path: Path, limit: int) -> Optional[str]:
    digest = hashlib.sha256()
    total = 0
    with open(path, "rb") as handle:
        while True:
            chunk = handle.read(1024 * 1024)
            if not chunk:
                break
            total += len(chunk)
            if total > limit:
                return None
            digest.update(chunk)
    return digest.hexdigest()


def _verify_logs(data: Mapping[str, Any], logs_dir: Path) -> Tuple[List[ValidationErrorItem], bool]:
    """Recompute each check's log hash from files under logs_dir. Returns (errors, verified_at_least_one)."""
    errors: List[ValidationErrorItem] = []
    verified = 0
    try:
        root = logs_dir.resolve(strict=True)
    except OSError:
        return [_err("logs_dir_invalid", str(logs_dir), "logs directory does not exist")], False
    if not root.is_dir():
        return [_err("logs_dir_invalid", str(logs_dir), "logs directory is not a directory")], False
    checks = data.get("checks")
    if not isinstance(checks, list):
        return errors, False
    for idx, chk in enumerate(checks):
        if not isinstance(chk, dict):
            continue
        lp, lh = chk.get("log_path"), chk.get("log_hash")
        where = f"checks[{idx}].log_path"
        if not (isinstance(lp, str) and isinstance(lh, str) and HEX_64_RE.fullmatch(lh)):
            continue
        if check_path(lp, where):
            continue  # already reported as a path error; never touch the filesystem with it
        current = root
        unsafe = False
        for part in lp.split("/"):
            current = current / part
            if current.is_symlink():
                unsafe = True
                break
        if unsafe:
            errors.append(_err("log_path_unsafe", where, f"'{lp}' passes through a symlink"))
            continue
        if not os.path.lexists(current):
            errors.append(_err("log_file_missing", where, f"log file '{lp}' not found under the logs directory"))
            continue
        if not current.is_file():
            errors.append(_err("log_path_unsafe", where, f"'{lp}' is not a regular file"))
            continue
        actual = _sha256_file(current, MAX_LOG_BYTES)
        if actual is None:
            errors.append(_err("log_file_too_large", where, f"log '{lp}' exceeds {MAX_LOG_BYTES} bytes"))
        elif actual != lh:
            errors.append(_err("log_hash_mismatch", f"checks[{idx}].log_hash", f"recorded hash does not match the bytes of '{lp}'"))
        else:
            verified += 1
    return errors, verified > 0 and not errors


def _task_shape_errors(task: Any) -> List[str]:
    problems: List[str] = []
    if not isinstance(task, Mapping):
        return ["task must be an object"]
    if not (isinstance(task.get("task_id"), str) and task["task_id"]):
        problems.append("task_id")
    if not (isinstance(task.get("target_sha"), str) and HEX_40_RE.fullmatch(task["target_sha"])):
        problems.append("target_sha")
    rc = task.get("required_checks")
    if not (isinstance(rc, list) and rc and all(isinstance(x, str) for x in rc)):
        problems.append("required_checks")
    if not (_is_int(task.get("timeout_seconds")) and task["timeout_seconds"] > 0):
        problems.append("timeout_seconds")
    return [f"task.{p} is missing or malformed" for p in problems]


def _check_task_coverage(data: Mapping[str, Any], task: Mapping[str, Any]) -> List[ValidationErrorItem]:
    shape = _task_shape_errors(task)
    if shape:
        return [_err("task_invalid", "task", "; ".join(shape))]
    errors: List[ValidationErrorItem] = []
    if data.get("task_id") != task["task_id"]:
        errors.append(_err("task_id_mismatch", "task_id", f"evidence task_id {data.get('task_id')!r} differs from the supplied task"))
    if isinstance(data.get("commit_sha"), str) and data["commit_sha"] != task["target_sha"]:
        errors.append(_err("task_sha_mismatch", "commit_sha", "evidence commit_sha differs from the supplied task target_sha"))
    checks = data.get("checks") if isinstance(data.get("checks"), list) else []
    by_id: Dict[str, Mapping[str, Any]] = {c["id"]: c for c in checks if isinstance(c, dict) and isinstance(c.get("id"), str)}
    for required in task["required_checks"]:
        chk = by_id.get(required)
        if chk is None:
            errors.append(_err("missing_required_check", "checks", f"required check '{required}' has no result"))
        elif chk.get("status") != "pass" or (_is_int(chk.get("exit_code")) and chk["exit_code"] != 0):
            errors.append(_err("required_check_not_passed", "checks", f"required check '{required}' did not pass"))
    limit_ms = task["timeout_seconds"] * 1000
    for idx, chk in enumerate(checks):
        dur = chk.get("duration_ms") if isinstance(chk, dict) else None
        if _is_int(dur) and dur > limit_ms:
            errors.append(_err("check_exceeds_timeout", f"checks[{idx}].duration_ms", f"{dur} ms exceeds the task timeout of {limit_ms} ms"))
    return errors


def _check_expectations(
    data: Mapping[str, Any],
    schema_name: str,
    exp: Expectations,
    schemas_dir: Path,
    applied: Dict[str, bool],
) -> Tuple[List[ValidationErrorItem], Optional[Set[str]]]:
    """Compare the payload with caller-supplied expectations. Returns (errors, catalog ids for v2 semantics)."""
    errors: List[ValidationErrorItem] = []
    kind = SCHEMA_KIND[schema_name]
    v2 = schema_name.endswith("-v2")

    def compare(key: str, field_name: str, expected: Optional[str], code: str) -> None:
        if expected is None:
            return
        if field_name not in data:
            errors.append(_err("missing_expected_field", field_name, f"caller expects {field_name} but the artifact has none"))
        elif data[field_name] != expected:
            errors.append(_err(code, field_name, f"{field_name} {data[field_name]!r} differs from the externally supplied expectation"))
        else:
            applied[key] = True

    compare("candidate_sha", CANDIDATE_FIELD[kind], exp.candidate_sha, "expected_sha_mismatch")
    compare("base_sha", "base_sha", exp.base_sha, "expected_sha_mismatch")
    if kind == "evidence":
        compare("tree_sha", "tree_sha", exp.tree_sha, "expected_sha_mismatch")
    compare("policy_version", "policy_version", exp.policy_version, "expected_policy_mismatch")

    catalog_ids: Optional[Set[str]] = None
    if v2 and kind in ("task", "evidence"):
        cat_path = exp.catalog_path or (schemas_dir / CATALOG_FILE)
        catalog_ids, cat_errors = load_check_catalog(cat_path)
        errors.extend(cat_errors)
        if exp.catalog_sha256 is not None and not cat_errors:
            try:
                actual = hashlib.sha256(cat_path.read_bytes()).hexdigest()
            except OSError as exc:
                errors.append(_err("catalog_invalid", str(cat_path), f"cannot read catalog: {exc.strerror or exc}"))
            else:
                if actual != exp.catalog_sha256:
                    errors.append(_err("catalog_sha256_mismatch", str(cat_path), "check catalog hash differs from the externally supplied expectation"))
                else:
                    applied["catalog_sha256"] = True

    if kind == "evidence":
        if exp.task is not None:
            task_errors = _check_task_coverage(data, exp.task)
            errors.extend(task_errors)
            if not task_errors:
                applied["task"] = True
        if v2 and exp.logs_dir is not None:
            log_errors, verified = _verify_logs(data, exp.logs_dir)
            errors.extend(log_errors)
            if verified:
                applied["logs"] = True
    return errors, catalog_ids


# ---------------------------------------------------------------------------------------------
# Schema resolution and validation entry points
# ---------------------------------------------------------------------------------------------

def normalize_explicit_schema(explicit: str) -> str:
    name = explicit.strip().lower()
    for suffix in (".schema.json", ".json"):
        if name.endswith(suffix):
            name = name[: -len(suffix)]
    return name


def resolve_schema(data: Any, explicit: Optional[str], label: str) -> Tuple[Optional[str], List[ValidationErrorItem]]:
    """Pick the schema from `schema_version` (or a matching explicit name). Never from key markers or file names."""
    if not isinstance(data, dict):
        return None, [_err("unknown_schema", label, "document root must be a JSON object")]
    payload = data.get("schema_version")
    if payload is not None and not (isinstance(payload, str) and payload in KNOWN_SCHEMAS):
        return None, [_err("unknown_schema", "schema_version", f"unknown schema_version {payload!r}")]
    if explicit:
        name = normalize_explicit_schema(explicit)
        if name in KNOWN_SCHEMAS:
            if payload is not None and payload != name:
                return None, [_err("schema_mismatch", "schema_version", f"payload declares '{payload}' but '{name}' was requested")]
            return name, []
        if name in SHORT_SCHEMA_NAMES:
            if payload is None:
                return None, [_err("missing_schema_version", "schema_version", f"short schema '{name}' needs a payload schema_version; pass the full name or add it")]
            if SCHEMA_KIND[payload] != name:
                return None, [_err("schema_mismatch", "schema_version", f"payload declares '{payload}' but kind '{name}' was requested")]
            return payload, []
        return None, [_err("unknown_schema", label, f"unknown schema '{explicit}'")]
    if payload is None:
        return None, [_err("missing_schema_version", "schema_version", "no schema_version in the payload and no --schema given; schemas are never guessed")]
    return payload, []


def load_schema(schemas_dir: Path, schema_name: str) -> Tuple[Optional[Dict[str, Any]], Optional[str]]:
    schema_file = schemas_dir / f"{schema_name}.schema.json"
    data, errors = read_json_file(schema_file, max_bytes=MAX_FILE_BYTES)
    if errors:
        return None, f"cannot load schema {schema_file}: {errors[0]}"
    if not isinstance(data, dict):
        return None, f"schema {schema_file} is not an object"
    return data, None


def validate_file_detailed(
    file_path: Path,
    explicit_schema: Optional[str],
    schemas_dir: Path,
    use_jsonschema: bool = True,
    expectations: Optional[Expectations] = None,
    jsonschema_module: Any = None,
) -> ValidationResult:
    """Validate one artifact. Structure and semantics only; authenticity is NOT established.

    `expectations` carries caller-supplied facts (candidate SHA, policy version, ...). Anything not
    supplied is reported in `notes` as unapplied; it is never inferred from the payload.
    """
    exp = expectations or Expectations()
    now = exp.now or datetime.datetime.now(datetime.timezone.utc)
    label = str(file_path)

    data, parse_errors = read_json_file(Path(file_path))
    if parse_errors:
        return ValidationResult(False, "unknown", parse_errors)

    schema_name, resolve_errors = resolve_schema(data, explicit_schema, label)
    if schema_name is None:
        return ValidationResult(False, "unknown", resolve_errors)

    schema_dict, load_err = load_schema(schemas_dir, schema_name)
    if load_err or schema_dict is None:
        return ValidationResult(False, schema_name, [_err("schema_load_error", label, load_err or "schema empty")])

    result = ValidationResult(True, schema_name, [])
    errors: List[ValidationErrorItem] = result.errors

    # 1. Built-in validator (canonical result). Unsupported keywords stop here, in both modes.
    structural = PureSchemaValidator(schema_dict).validate(data)
    errors.extend(structural)
    schema_unusable = any(e.code in ("unsupported_schema_keyword", "unsupported_schema_dialect", "invalid_schema") for e in structural)

    # 2. jsonschema cross-check: may only add a divergence failure, never relax the verdict.
    js = jsonschema_module if jsonschema_module is not None else (jsonschema if HAS_JSONSCHEMA else None)
    if use_jsonschema and js is not None and not schema_unusable:
        try:
            js_errors = _jsonschema_errors(js, schema_dict, data)
        except Exception as exc:  # noqa: BLE001 - any validator crash must fail closed, visibly
            errors.append(_err("jsonschema_exception", label, f"{type(exc).__name__}: {exc}"))
        else:
            if js_errors and not structural:
                first = js_errors[0]
                where = ".".join(str(p) for p in getattr(first, "path", [])) or "<root>"
                errors.append(_err("validator_divergence", where, f"jsonschema rejects what the built-in validator accepted: {first.message}"))

    # 3. External expectations, check catalog and log hashes (caller-supplied facts only).
    if schema_unusable:
        return _finish(result, schema_name, exp)
    exp_errors, catalog_ids = _check_expectations(data, schema_name, exp, schemas_dir, result.applied)
    for e in exp_errors:
        _add_unique(errors, e)

    # 4. Semantic and security rules, keyed on the resolved schema.
    for s_err in run_semantic_checks(data, schema_name, now, catalog_ids):
        _add_unique(errors, s_err)

    return _finish(result, schema_name, exp)


def _add_unique(errors: List[ValidationErrorItem], item: ValidationErrorItem) -> None:
    if not any(e.field == item.field and e.code == item.code for e in errors):
        errors.append(item)


def _finish(result: ValidationResult, schema_name: str, exp: Expectations) -> ValidationResult:
    result.ok = not result.errors
    if schema_name.endswith("-v1"):
        result.notes.append(
            "historical_v1: v1 artifacts are historical, validated for structure only, and never become trusted evidence"
        )
    missing = [k for k in RECOMMENDED_EXPECTATIONS.get(schema_name, ()) if not result.applied.get(k)]
    if missing:
        result.notes.append(
            "external_expectations_missing: not verified against caller-supplied facts: " + ", ".join(missing)
        )
    return result


def validate_file(
    file_path: Path,
    explicit_schema: Optional[str],
    schemas_dir: Path,
    use_jsonschema: bool = True,
    expectations: Optional[Expectations] = None,
) -> Tuple[bool, str, List[ValidationErrorItem]]:
    """Structure-only validation; returns (ok, schema name, errors). Does not establish authenticity."""
    r = validate_file_detailed(file_path, explicit_schema, schemas_dir, use_jsonschema, expectations)
    return r.ok, r.schema, r.errors


def load_task_expectation(
    task_path: Path, schemas_dir: Path, use_jsonschema: bool, now: Optional[datetime.datetime]
) -> Tuple[Optional[Dict[str, Any]], List[ValidationErrorItem]]:
    """Validate a caller-supplied task file and return its data for required-check coverage."""
    result = validate_file_detailed(task_path, None, schemas_dir, use_jsonschema, Expectations(now=now))
    if not result.ok or SCHEMA_KIND.get(result.schema) != "task":
        detail = "; ".join(str(e) for e in result.errors[:3]) or f"not a task specification (schema {result.schema})"
        return None, [_err("task_invalid", str(task_path), detail)]
    data, errors = read_json_file(task_path)
    return (data, []) if not errors else (None, [_err("task_invalid", str(task_path), str(errors[0]))])


# ---------------------------------------------------------------------------------------------
# Self-test over the shipped fixtures
# ---------------------------------------------------------------------------------------------

SELF_TEST_VALID = (
    "valid_task.json", "valid_evidence.json", "valid_review.json",
    "valid_task_v2.json", "valid_evidence_v2.json", "valid_review_v2.json",
)
SELF_TEST_INVALID = (
    ("invalid_wrong_sha.json", "invalid_sha"),
    ("invalid_scope_violation.json", "scope_violation"),
    ("invalid_missing_required.json", "missing_required"),
    ("invalid_duplicate_key.json", "duplicate_key"),
    ("invalid_nan.json", "non_finite_number"),
    ("invalid_bool_integer.json", "type_mismatch"),
    ("invalid_impossible_date.json", "invalid_timestamp"),
    ("invalid_pass_with_blocking.json", "pass_with_blocking_finding"),
    ("invalid_broad_glob.json", "unauthorized_glob"),
    ("invalid_unknown_field.json", "unexpected_property"),
    ("invalid_zero_checks.json", "array_too_short"),
    ("invalid_unknown_check_id.json", "unknown_check_id"),
)


def run_fixtures_self_test(schemas_dir: Path, use_jsonschema: bool = True) -> int:
    fixtures_dir = schemas_dir.parent / "fixtures"
    if not fixtures_dir.is_dir():
        print(f"ERROR: fixtures directory not found at {fixtures_dir}", file=sys.stderr)
        return 2
    now = datetime.datetime(2026, 8, 1, tzinfo=datetime.timezone.utc)
    exp = Expectations(now=now)
    passed = failed = 0
    print("=== Running Harness Fixtures Self-Test ===\n")
    print("-- Valid fixtures (expect structural PASS) --")
    for fname in SELF_TEST_VALID:
        r = validate_file_detailed(fixtures_dir / fname, None, schemas_dir, use_jsonschema, exp)
        if r.ok:
            passed += 1
            print(f"  PASS [expected PASS]: {fname} (schema: {r.schema})")
        else:
            failed += 1
            print(f"  FAIL [expected PASS]: {fname} (schema: {r.schema})", file=sys.stderr)
            for err in r.errors:
                print(f"       {err}", file=sys.stderr)
    print("\n-- Adversarial fixtures (expect FAIL-CLOSED with the named diagnostic) --")
    for fname, expected_code in SELF_TEST_INVALID:
        r = validate_file_detailed(fixtures_dir / fname, None, schemas_dir, use_jsonschema, exp)
        got = {e.code for e in r.errors}
        if not r.ok and expected_code in got:
            passed += 1
            print(f"  PASS [expected FAIL]: {fname} -> {expected_code} ({len(r.errors)} diagnostic(s))")
        else:
            failed += 1
            print(f"  FAIL [expected FAIL:{expected_code}]: {fname} ok={r.ok} codes={sorted(got)}", file=sys.stderr)
    print()
    print(f"SELF_TEST_RESULT: passed={passed} failed={failed}")
    return 0 if failed == 0 and passed > 0 else 1


# ---------------------------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------------------------

def _parse_now(value: str) -> datetime.datetime:
    match = TIMESTAMP_RE.fullmatch(value)
    if not match:
        raise argparse.ArgumentTypeError("--now must be an RFC 3339 UTC timestamp ending in Z")
    parts = [int(match.group(i)) for i in range(1, 7)]
    try:
        return datetime.datetime(*parts, tzinfo=datetime.timezone.utc)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(f"--now is not a real calendar time: {exc}") from exc


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Validate Engram harness task, evidence and review JSON artifacts (structure only; never proves authenticity).",
    )
    parser.add_argument("files", nargs="*", help="JSON file(s) to validate")
    parser.add_argument("--schema", choices=list(KNOWN_SCHEMAS) + list(SHORT_SCHEMA_NAMES),
                        help="Explicit schema; must match the payload schema_version (schemas are never guessed)")
    parser.add_argument("--schemas-dir", type=Path, default=None, help="Directory containing *.schema.json files")
    parser.add_argument("--json", action="store_true", help="Emit the harness-json-v1 envelope (docs/harness/JSON_OUTPUTS.md)")
    parser.add_argument("--self-test", action="store_true", help="Run the self-test over the shipped fixtures")
    parser.add_argument("--no-jsonschema", action="store_true", help="Do not use the optional jsonschema cross-check")
    parser.add_argument("--now", type=_parse_now, default=None, help="Override the validator clock (tests); RFC 3339 UTC")
    group = parser.add_argument_group("external expectations (caller-supplied; never read from the payload)")
    group.add_argument("--expect-candidate-sha", help="Expected candidate commit SHA (evidence commit_sha / review head_sha / task target_sha)")
    group.add_argument("--expect-base-sha", help="Expected base commit SHA")
    group.add_argument("--expect-tree-sha", help="Expected tree SHA (evidence)")
    group.add_argument("--expect-policy-version", help="Expected policy version (v2 artifacts)")
    group.add_argument("--catalog", type=Path, help="Check catalog file (default: <schemas-dir>/check-catalog-v1.json)")
    group.add_argument("--expect-catalog-sha256", help="Expected SHA-256 of the check catalog file")
    group.add_argument("--task", type=Path, help="Trusted task specification: required-check coverage, task id, timeout")
    group.add_argument("--logs-dir", type=Path, help="Directory holding the check logs; verifies each log_hash against its bytes")
    group.add_argument("--require-expectations", action="store_true",
                       help="Exit 2 unless the expectations required for each artifact kind were supplied")
    return parser


def _emit_json(payload: Dict[str, Any]) -> None:
    print(json.dumps(payload, indent=2))


def main(argv: Optional[List[str]] = None) -> int:
    # Payload text (e.g. a lone surrogate in a key) must never crash the diagnostics printer.
    for stream in (sys.stdout, sys.stderr):
        if hasattr(stream, "reconfigure"):
            stream.reconfigure(errors="backslashreplace")
    parser = _build_parser()
    args = parser.parse_args(argv)

    schemas_dir = args.schemas_dir or (Path(__file__).resolve().parent.parent / "schemas")
    use_js = not args.no_jsonschema

    if args.self_test:
        return run_fixtures_self_test(schemas_dir, use_js)
    if not args.files:
        parser.print_help(sys.stderr)
        return 2

    try:
        base = Expectations(
            candidate_sha=args.expect_candidate_sha,
            base_sha=args.expect_base_sha,
            tree_sha=args.expect_tree_sha,
            policy_version=args.expect_policy_version,
            catalog_sha256=args.expect_catalog_sha256,
            catalog_path=args.catalog,
            logs_dir=args.logs_dir,
            now=args.now,
        )
    except ValueError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 2

    task_errors: List[ValidationErrorItem] = []
    task_data: Optional[Dict[str, Any]] = None
    if args.task is not None:
        task_data, task_errors = load_task_expectation(args.task, schemas_dir, use_js, args.now)
    exp = Expectations(
        candidate_sha=base.candidate_sha, base_sha=base.base_sha, tree_sha=base.tree_sha,
        policy_version=base.policy_version, catalog_sha256=base.catalog_sha256, catalog_path=base.catalog_path,
        logs_dir=base.logs_dir, task=task_data, now=base.now,
    )

    timestamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    failed_files = 0
    failures_json: List[Dict[str, Any]] = []
    checks_json: List[Dict[str, Any]] = []
    notes_json: List[str] = []
    files_json: List[Dict[str, Any]] = []
    missing_flags: List[str] = []
    all_applied = {k: True for k in EXPECTATION_KEYS}
    all_complete = True

    for file_str in args.files:
        result = validate_file_detailed(Path(file_str), args.schema, schemas_dir, use_js, exp)
        errors = list(result.errors)
        if args.task is not None and task_errors:
            errors.extend(task_errors)
        ok = not errors
        for key in EXPECTATION_KEYS:
            all_applied[key] = all_applied[key] and result.applied[key]
        all_complete = all_complete and result.expectations_complete
        if args.require_expectations:
            for key in REQUIRED_EXPECTATIONS.get(result.schema, ()):
                if not exp.provided(key) and EXPECTATION_FLAGS[key] not in missing_flags:
                    missing_flags.append(EXPECTATION_FLAGS[key])
        files_json.append({"path": file_str, "schema": result.schema, "trust": result.trust,
                           "expectations_applied": dict(result.applied), "notes": list(result.notes)})
        for note in result.notes:
            notes_json.append(note)
            if not args.json:
                print(f"NOTE[{note.split(':', 1)[0].upper()}]: {file_str}: {note.split(':', 1)[1].strip()}")
        if ok:
            checks_json.append({"id": f"validate:{file_str}", "status": "pass",
                                "message": f"structurally valid against {result.schema} (not authenticated)", "path": file_str})
            if not args.json:
                print(f"VALIDATION_OK: {file_str} (schema: {result.schema}; structure only, not authenticated)")
        else:
            failed_files += 1
            for err in errors:
                failures_json.append({"id": f"validation_error:{file_str}:{err.code}", "message": str(err),
                                      "path": file_str, "code": err.code, "field": err.field})
                if not args.json:
                    print(f"VALIDATION_ERROR: {file_str}: {err}", file=sys.stderr)
            checks_json.append({"id": f"validate:{file_str}", "status": "fail",
                                "message": f"failed validation with {len(errors)} error(s)", "path": file_str})

    if args.require_expectations and missing_flags:
        message = "--require-expectations: missing " + ", ".join(missing_flags)
        if args.json:
            _emit_json({
                "schema_version": "harness-json-v1", "tool": "validate-evidence", "mode": "json",
                "status": "usage_error", "exit_code": 2, "timestamp": timestamp, "summary": message,
                "warnings": [], "failures": [{"id": "usage:missing_expectations", "message": message}],
                "checks": checks_json, "artifacts": [],
            })
        else:
            print(f"ERROR: {message}", file=sys.stderr)
        return 2

    exit_code = 0 if failed_files == 0 else 1
    if args.json:
        _emit_json({
            "schema_version": "harness-json-v1",
            "tool": "validate-evidence",
            "mode": "json",
            "status": "pass" if exit_code == 0 else "fail",
            "exit_code": exit_code,
            "timestamp": timestamp,
            "summary": f"validated {len(args.files)} file(s), {failed_files} failure(s); structure only, not authenticated",
            "warnings": [],
            "failures": failures_json,
            "checks": checks_json,
            "artifacts": [],
            "trust": VALIDATION_SCOPE,
            "jsonschema": "used" if (use_js and HAS_JSONSCHEMA) else ("disabled" if not use_js else "absent"),
            "expectations_applied": all_applied,
            "expectations_complete": all_complete,
            "notes": notes_json,
            "files": files_json,
        })
    return exit_code


if __name__ == "__main__":
    sys.exit(main())

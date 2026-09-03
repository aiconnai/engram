#!/usr/bin/env python3
"""validate-evidence.py

Standalone validator for Engram agent harness task specifications,
revision-bound evidence receipts, and structured review reports.

Supports:
- Pure Python stdlib validation (zero external dependencies required)
- Optional jsonschema library validation when available
- Strict fail-closed semantic checks:
  - 40-char lowercase hex Git commit SHAs
  - 64-char lowercase hex SHA-256 digests
  - Canonical RFC 3339 UTC timestamps
  - Scope and path-traversal prevention (no absolute paths, no '..')
  - Capability allowlisting (blocks unauthorized execution / credential mounts)
  - Missing required fields
  - Unexpected additional properties
- Human-readable and JSON output modes (aligned with docs/harness/JSON_OUTPUTS.md)
"""

from __future__ import annotations

import argparse
import datetime
import json
import os
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple, Union

# Try to import jsonschema if available; fallback cleanly to stdlib if absent
try:
    import jsonschema  # type: ignore
    HAS_JSONSCHEMA = True
except ImportError:
    jsonschema = None  # type: ignore
    HAS_JSONSCHEMA = False

HEX_40_RE = re.compile(r"^[0-9a-f]{40}$")
HEX_64_RE = re.compile(r"^[0-9a-f]{64}$")
RFC3339_Z_RE = re.compile(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}(?:\.[0-9]+)?Z$")

# Capabilities considered dangerous and prohibited in sandbox task specifications
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


class ValidationErrorItem:
    def __init__(self, code: str, field: str, message: str):
        self.code = code
        self.field = field
        self.message = message

    def to_dict(self) -> Dict[str, str]:
        return {
            "code": self.code,
            "field": self.field,
            "message": self.message,
        }

    def __str__(self) -> str:
        if self.field:
            return f"[{self.code}] {self.field}: {self.message}"
        return f"[{self.code}] {self.message}"


class PureSchemaValidator:
    """Zero-dependency JSON Schema validator covering draft 2020-12 / draft-7 subsets used in harness schemas."""

    def __init__(self, schema: Dict[str, Any]):
        self.schema = schema

    def validate(self, instance: Any) -> List[ValidationErrorItem]:
        errors: List[ValidationErrorItem] = []
        self._validate_node(instance, self.schema, "", errors)
        return errors

    def _validate_node(
        self, instance: Any, schema: Dict[str, Any], path: str, errors: List[ValidationErrorItem]
    ) -> None:
        # Type check
        expected_type = schema.get("type")
        if expected_type:
            if not self._check_type(instance, expected_type):
                actual_type = type(instance).__name__
                errors.append(
                    ValidationErrorItem(
                        "type_mismatch",
                        path or "<root>",
                        f"expected type '{expected_type}', got '{actual_type}'",
                    )
                )
                return

        # const
        if "const" in schema:
            if instance != schema["const"]:
                errors.append(
                    ValidationErrorItem(
                        "const_mismatch",
                        path or "<root>",
                        f"expected constant value '{schema['const']}', got '{instance}'",
                    )
                )

        # enum
        if "enum" in schema:
            if instance not in schema["enum"]:
                errors.append(
                    ValidationErrorItem(
                        "invalid_enum_value",
                        path or "<root>",
                        f"value '{instance}' is not one of allowed enum values: {schema['enum']}",
                    )
                )

        # String validations
        if isinstance(instance, str):
            if "pattern" in schema:
                pattern = schema["pattern"]
                if not re.search(pattern, instance):
                    errors.append(
                        ValidationErrorItem(
                            "invalid_pattern",
                            path or "<root>",
                            f"value '{instance}' does not match pattern '{pattern}'",
                        )
                    )
            if "minLength" in schema and len(instance) < schema["minLength"]:
                errors.append(
                    ValidationErrorItem(
                        "string_too_short",
                        path or "<root>",
                        f"string length {len(instance)} is shorter than minLength {schema['minLength']}",
                    )
                )
            if "maxLength" in schema and len(instance) > schema["maxLength"]:
                errors.append(
                    ValidationErrorItem(
                        "string_too_long",
                        path or "<root>",
                        f"string length {len(instance)} exceeds maxLength {schema['maxLength']}",
                    )
                )

        # Number / integer validations
        if isinstance(instance, (int, float)) and not isinstance(instance, bool):
            if "minimum" in schema and instance < schema["minimum"]:
                errors.append(
                    ValidationErrorItem(
                        "number_too_small",
                        path or "<root>",
                        f"value {instance} is less than minimum {schema['minimum']}",
                    )
                )
            if "maximum" in schema and instance > schema["maximum"]:
                errors.append(
                    ValidationErrorItem(
                        "number_too_large",
                        path or "<root>",
                        f"value {instance} exceeds maximum {schema['maximum']}",
                    )
                )

        # Array validations
        if isinstance(instance, list):
            if "minItems" in schema and len(instance) < schema["minItems"]:
                errors.append(
                    ValidationErrorItem(
                        "array_too_short",
                        path or "<root>",
                        f"array has {len(instance)} items, expected at least {schema['minItems']}",
                    )
                )
            if schema.get("uniqueItems", False):
                try:
                    seen = set()
                    for item in instance:
                        key = json.dumps(item, sort_keys=True)
                        if key in seen:
                            errors.append(
                                ValidationErrorItem(
                                    "duplicate_array_item",
                                    path or "<root>",
                                    f"array contains duplicate item: {item}",
                                )
                            )
                            break
                        seen.add(key)
                except Exception:
                    pass

            item_schema = schema.get("items")
            if item_schema and isinstance(item_schema, dict):
                for idx, item in enumerate(instance):
                    item_path = f"{path}[{idx}]" if path else f"[{idx}]"
                    self._validate_node(item, item_schema, item_path, errors)

        # Object validations
        if isinstance(instance, dict):
            # required properties
            for req in schema.get("required", []):
                if req not in instance:
                    field_name = f"{path}.{req}" if path else req
                    errors.append(
                        ValidationErrorItem(
                            "missing_required",
                            field_name,
                            f"required field '{req}' is missing",
                        )
                    )

            # additionalProperties
            properties = schema.get("properties", {})
            additional_properties = schema.get("additionalProperties", True)
            if additional_properties is False:
                for key in instance:
                    if key not in properties:
                        field_name = f"{path}.{key}" if path else key
                        errors.append(
                            ValidationErrorItem(
                                "unexpected_property",
                                field_name,
                                f"unexpected additional property '{key}' not permitted by schema",
                            )
                        )

            # propertyNames
            prop_names_schema = schema.get("propertyNames")
            if prop_names_schema and isinstance(prop_names_schema, dict):
                pattern = prop_names_schema.get("pattern")
                if pattern:
                    for key in instance:
                        if not re.search(pattern, key):
                            field_name = f"{path}[{key}]" if path else f"[{key}]"
                            errors.append(
                                ValidationErrorItem(
                                    "invalid_property_name",
                                    field_name,
                                    f"property name '{key}' violates pattern '{pattern}'",
                                )
                            )

            # properties
            for prop_name, prop_value in instance.items():
                if prop_name in properties:
                    prop_schema = properties[prop_name]
                    prop_path = f"{path}.{prop_name}" if path else prop_name
                    self._validate_node(prop_value, prop_schema, prop_path, errors)
                elif isinstance(additional_properties, dict):
                    prop_path = f"{path}.{prop_name}" if path else prop_name
                    self._validate_node(prop_value, additional_properties, prop_path, errors)

    def _check_type(self, value: Any, expected_type: str) -> bool:
        if expected_type == "object":
            return isinstance(value, dict)
        if expected_type == "array":
            return isinstance(value, list)
        if expected_type == "string":
            return isinstance(value, str)
        if expected_type == "integer":
            return type(value) is int
        if expected_type == "number":
            return isinstance(value, (int, float)) and not isinstance(value, bool)
        if expected_type == "boolean":
            return isinstance(value, bool)
        if expected_type == "null":
            return value is None
        return True


def check_scope_and_paths(path_str: str, context: str) -> Optional[ValidationErrorItem]:
    """Ensures file paths are repo-relative, without traversal (..) or leading slashes."""
    if not isinstance(path_str, str):
        return None
    if path_str.startswith("/") or path_str.startswith("\\"):
        return ValidationErrorItem(
            "scope_violation",
            context,
            f"absolute path '{path_str}' is forbidden; must be repo-relative",
        )
    parts = path_str.replace("\\", "/").split("/")
    if ".." in parts:
        return ValidationErrorItem(
            "scope_violation",
            context,
            f"path traversal '..' in '{path_str}' is forbidden",
        )
    return None


def run_semantic_checks(data: Dict[str, Any], schema_name: str) -> List[ValidationErrorItem]:
    """Enforces fail-closed security and semantic rules beyond basic schema typing."""
    errors: List[ValidationErrorItem] = []

    # SHA checks
    for sha_key in ("commit_sha", "target_sha", "base_sha", "head_sha", "tree_sha"):
        if sha_key in data:
            val = data[sha_key]
            if not isinstance(val, str) or not HEX_40_RE.match(val):
                errors.append(
                    ValidationErrorItem(
                        "invalid_sha",
                        sha_key,
                        f"expected 40-character lowercase hexadecimal SHA, got '{val}'",
                    )
                )

    # Timestamp checks
    if "timestamp" in data:
        ts = data["timestamp"]
        if not isinstance(ts, str) or not RFC3339_Z_RE.match(ts):
            errors.append(
                ValidationErrorItem(
                    "invalid_timestamp",
                    "timestamp",
                    f"expected canonical RFC 3339 UTC timestamp ending in 'Z', got '{ts}'",
                )
            )

    # Task specification semantic checks
    if schema_name == "task-v1" or "task_id" in data:
        # Capabilities
        caps = data.get("allowed_capabilities", [])
        if isinstance(caps, list):
            for idx, cap in enumerate(caps):
                field_path = f"allowed_capabilities[{idx}]"
                if cap in DISALLOWED_CAPABILITIES:
                    errors.append(
                        ValidationErrorItem(
                            "scope_violation",
                            field_path,
                            f"disallowed capability '{cap}' is strictly prohibited",
                        )
                    )
                elif cap not in APPROVED_CAPABILITIES:
                    errors.append(
                        ValidationErrorItem(
                            "unapproved_capability",
                            field_path,
                            f"capability '{cap}' is not in approved list: {sorted(APPROVED_CAPABILITIES)}",
                        )
                    )
        # Paths
        for p_key in ("allowed_paths", "protected_paths"):
            paths = data.get(p_key, [])
            if isinstance(paths, list):
                for idx, p in enumerate(paths):
                    err = check_scope_and_paths(p, f"{p_key}[{idx}]")
                    if err:
                        errors.append(err)

    # Evidence receipt semantic checks
    if schema_name == "evidence-v1" or "sha256_manifest" in data:
        manifest = data.get("sha256_manifest", {})
        if isinstance(manifest, dict):
            for path_key, digest in manifest.items():
                err = check_scope_and_paths(path_key, f"sha256_manifest['{path_key}']")
                if err:
                    errors.append(err)
                if not isinstance(digest, str) or not HEX_64_RE.match(digest):
                    errors.append(
                        ValidationErrorItem(
                            "invalid_digest",
                            f"sha256_manifest['{path_key}']",
                            f"expected 64-character lowercase hex SHA-256 digest, got '{digest}'",
                        )
                    )

    # Review report semantic checks
    if schema_name == "review-v1" or "findings" in data:
        findings = data.get("findings", [])
        if isinstance(findings, list):
            for idx, f in enumerate(findings):
                if isinstance(f, dict):
                    f_path = f.get("path")
                    if f_path:
                        err = check_scope_and_paths(f_path, f"findings[{idx}].path")
                        if err:
                            errors.append(err)

    return errors


def resolve_schema_for_data(
    data: Any, explicit_schema: Optional[str], file_path: Path
) -> str:
    if explicit_schema:
        # Normalize e.g. "task-v1.schema.json" -> "task-v1"
        s = explicit_schema.lower()
        if "task" in s:
            return "task-v1"
        if "evidence" in s:
            return "evidence-v1"
        if "review" in s:
            return "review-v1"
        return explicit_schema

    if isinstance(data, dict):
        # 1. Check explicit schema_version
        version = data.get("schema_version")
        if version in ("task-v1", "evidence-v1", "review-v1"):
            return version

        # 2. Check key markers
        if "task_id" in data or "allowed_capabilities" in data or "required_checks" in data:
            return "task-v1"
        if "sha256_manifest" in data or "commit_sha" in data:
            return "evidence-v1"
        if "review_id" in data or "findings" in data or "head_sha" in data:
            return "review-v1"

    # 3. Fallback to filename heuristic
    fname = file_path.name.lower()
    if "task" in fname:
        return "task-v1"
    if "evidence" in fname:
        return "evidence-v1"
    if "review" in fname:
        return "review-v1"

    return "unknown"


def load_schema(schemas_dir: Path, schema_name: str) -> Tuple[Optional[Dict[str, Any]], Optional[str]]:
    schema_file = schemas_dir / f"{schema_name}.schema.json"
    if not schema_file.exists():
        # Try direct path
        direct = Path(schema_name)
        if direct.exists():
            schema_file = direct
        else:
            return None, f"Schema file not found: {schema_file}"

    try:
        with open(schema_file, "r", encoding="utf-8") as f:
            return json.load(f), None
    except Exception as e:
        return None, f"Failed to parse schema file {schema_file}: {e}"


def validate_file(
    file_path: Path,
    explicit_schema: Optional[str],
    schemas_dir: Path,
    use_jsonschema: bool = True,
) -> Tuple[bool, str, List[ValidationErrorItem]]:
    if not file_path.exists():
        return False, "unknown", [
            ValidationErrorItem("file_not_found", str(file_path), "file does not exist")
        ]

    try:
        with open(file_path, "r", encoding="utf-8") as f:
            data = json.load(f)
    except Exception as e:
        return False, "unknown", [
            ValidationErrorItem("json_decode_error", str(file_path), f"JSON parse error: {e}")
        ]

    schema_name = resolve_schema_for_data(data, explicit_schema, file_path)
    if schema_name == "unknown":
        return False, "unknown", [
            ValidationErrorItem(
                "unknown_schema",
                str(file_path),
                "cannot determine schema (specify with --schema)",
            )
        ]

    schema_dict, load_err = load_schema(schemas_dir, schema_name)
    if load_err or schema_dict is None:
        return False, schema_name, [
            ValidationErrorItem("schema_load_error", str(file_path), load_err or "schema empty")
        ]

    errors: List[ValidationErrorItem] = []

    # 1. Pure stdlib validation
    pure_validator = PureSchemaValidator(schema_dict)
    pure_errors = pure_validator.validate(data)
    errors.extend(pure_errors)

    # 2. jsonschema validation (if available and requested)
    if HAS_JSONSCHEMA and use_jsonschema:
        try:
            validator_cls = jsonschema.validators.validator_for(schema_dict)
            validator = validator_cls(schema_dict)
            for err in validator.iter_errors(data):
                field_path = ".".join(str(p) for p in err.path) or "<root>"
                # Deduplicate if pure validator caught same required field
                already_reported = any(e.field == field_path for e in errors)
                if not already_reported:
                    errors.append(
                        ValidationErrorItem(
                            "schema_violation",
                            field_path,
                            err.message,
                        )
                    )
        except Exception as e:
            errors.append(
                ValidationErrorItem("jsonschema_exception", str(file_path), str(e))
            )

    # 3. Fail-closed semantic and security checks
    semantic_errors = run_semantic_checks(data, schema_name)
    for s_err in semantic_errors:
        # Check if already recorded with same field
        if not any(e.field == s_err.field and e.code == s_err.code for e in errors):
            errors.append(s_err)

    is_valid = len(errors) == 0
    return is_valid, schema_name, errors


def run_fixtures_self_test(schemas_dir: Path) -> int:
    fixtures_dir = schemas_dir.parent / "fixtures"
    if not fixtures_dir.exists():
        print(f"ERROR: fixtures directory not found at {fixtures_dir}", file=sys.stderr)
        return 2

    valid_fixtures = ["valid_task.json", "valid_evidence.json", "valid_review.json"]
    invalid_fixtures = [
        "invalid_wrong_sha.json",
        "invalid_scope_violation.json",
        "invalid_missing_required.json",
    ]

    all_passed = True
    print("=== Running Harness Fixtures Self-Test ===\n")

    print("-- Testing Valid Fixtures (expect PASS) --")
    for fname in valid_fixtures:
        fpath = fixtures_dir / fname
        ok, schema, errors = validate_file(fpath, None, schemas_dir)
        if ok:
            print(f"  PASS [expected PASS]: {fname} (schema: {schema})")
        else:
            all_passed = False
            print(f"  FAIL [expected PASS]: {fname} (schema: {schema})", file=sys.stderr)
            for err in errors:
                print(f"       {err}", file=sys.stderr)

    print("\n-- Testing Adversarial Fixtures (expect FAIL-CLOSED) --")
    for fname in invalid_fixtures:
        fpath = fixtures_dir / fname
        ok, schema, errors = validate_file(fpath, None, schemas_dir)
        if not ok:
            print(f"  PASS [expected FAIL]: {fname} (rejected with {len(errors)} diagnostic error(s))")
            for err in errors:
                print(f"       -> {err}")
        else:
            all_passed = False
            print(f"  FAIL [expected FAIL]: {fname} was unexpectedly ACCEPTED", file=sys.stderr)

    print("\n==========================================")
    if all_passed:
        print("ALL FIXTURES BEHAVED AS EXPECTED (6/6 tests passed)")
        return 0
    else:
        print("SELF-TEST FAILED", file=sys.stderr)
        return 1


def main(argv: Optional[List[str]] = None) -> int:
    parser = argparse.ArgumentParser(
        description="Validate Engram harness task, evidence, and review JSON artifacts against schemas.",
    )
    parser.add_argument(
        "files",
        nargs="*",
        help="JSON file(s) to validate",
    )
    parser.add_argument(
        "--schema",
        choices=["task-v1", "evidence-v1", "review-v1", "task", "evidence", "review"],
        help="Explicit schema to validate against (defaults to auto-detection)",
    )
    parser.add_argument(
        "--schemas-dir",
        type=Path,
        default=None,
        help="Directory containing *.schema.json files",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Emit structured JSON output (conforming to docs/harness/JSON_OUTPUTS.md)",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="Run self-test on standard harness fixtures",
    )

    args = parser.parse_args(argv)

    # Locate schemas directory
    script_dir = Path(__file__).resolve().parent
    repo_root = script_dir.parent.parent.parent
    schemas_dir = args.schemas_dir or (repo_root / "docs" / "harness" / "schemas")

    if args.self_test:
        return run_fixtures_self_test(schemas_dir)

    if not args.files:
        parser.print_help(sys.stderr)
        return 2

    timestamp = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    total_files = len(args.files)
    failed_files = 0
    failures_json: List[Dict[str, Any]] = []
    checks_json: List[Dict[str, Any]] = []

    for file_str in args.files:
        file_path = Path(file_str)
        ok, schema_name, errors = validate_file(file_path, args.schema, schemas_dir)

        if ok:
            checks_json.append({
                "id": f"validate:{file_str}",
                "status": "pass",
                "message": f"validated successfully against {schema_name}",
                "path": file_str,
            })
            if not args.json:
                print(f"VALIDATION_OK: {file_str} (schema: {schema_name})")
        else:
            failed_files += 1
            for err in errors:
                failures_json.append({
                    "id": f"validation_error:{file_str}:{err.code}",
                    "message": str(err),
                    "path": file_str,
                    "code": err.code,
                    "field": err.field,
                })
                if not args.json:
                    print(f"VALIDATION_ERROR: {file_str}: {err}", file=sys.stderr)

            checks_json.append({
                "id": f"validate:{file_str}",
                "status": "fail",
                "message": f"failed validation with {len(errors)} error(s)",
                "path": file_str,
            })

    status_str = "pass" if failed_files == 0 else "fail"
    exit_code = 0 if failed_files == 0 else 1

    if args.json:
        output_payload = {
            "schema_version": "harness-json-v1",
            "tool": "validate-evidence",
            "mode": "json",
            "status": status_str,
            "exit_code": exit_code,
            "timestamp": timestamp,
            "summary": f"validated {total_files} file(s), {failed_files} failure(s)",
            "warnings": [],
            "failures": failures_json,
            "checks": checks_json,
            "artifacts": [],
        }
        print(json.dumps(output_payload, indent=2))

    return exit_code


if __name__ == "__main__":
    sys.exit(main())

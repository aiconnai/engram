#!/usr/bin/env python3
"""Regression suite for docs/harness/bin/validate-evidence.py (task H2).

Offline and deterministic: only synthetic fixtures and temporary directories, an injected
clock, no network. It runs in the mandatory offline lane (docs/harness/bin/run-offline-lane.sh).

The validator checks structure and semantics, never authenticity. These tests therefore also pin
the trust boundary: expected candidate SHA / policy version / catalog hash / logs directory come
from the caller (CLI flags or ``Expectations``), never from the payload.

The jsonschema-present path is tested with jsonschema installed; the absent path is simulated
with an import blocker (``sys.modules['jsonschema'] = None``), not by uninstalling anything. A
missing jsonschema fails the suite instead of skipping it.
"""

from __future__ import annotations

import copy
import datetime
import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO_ROOT = HERE.parent.parent.parent
BIN = REPO_ROOT / "docs" / "harness" / "bin" / "validate-evidence.py"
SCHEMAS = REPO_ROOT / "docs" / "harness" / "schemas"
FIXTURES = REPO_ROOT / "docs" / "harness" / "fixtures"
FIXED_NOW = datetime.datetime(2026, 8, 1, tzinfo=datetime.timezone.utc)

CAND = "0123456789abcdef0123456789abcdef01234567"
BASE = "abcdef0123456789abcdef0123456789abcdef01"
TREE = "fedcba9876543210fedcba9876543210fedcba98"
OTHER = "1111111111111111111111111111111111111111"
POLICY = "harness-hardening-v1"

_spec = importlib.util.spec_from_file_location("validate_evidence", BIN)
V = importlib.util.module_from_spec(_spec)
sys.modules["validate_evidence"] = V
_spec.loader.exec_module(V)


def fixture(name: str) -> dict:
    return json.loads((FIXTURES / name).read_text(encoding="utf-8"))


def mutated(name: str, mutate) -> str:
    data = fixture(name)
    mutate(data)
    return json.dumps(data)


def codes(result) -> set:
    return {e.code for e in result.errors}


# ---------------------------------------------------------------------------------------------
# Table of adversarial cases. Each entry: (case name, fixture, mutator or raw-text builder,
# codes that must ALL be reported). Raw cases return text instead of mutating the object.
# ---------------------------------------------------------------------------------------------

TASK, EVID, REV = "valid_task_v2.json", "valid_evidence_v2.json", "valid_review_v2.json"


def setk(key, value):
    def apply(d):
        d[key] = value
    return apply


def delk(key):
    def apply(d):
        d.pop(key, None)
    return apply


def check0(**kw):
    def apply(d):
        d["checks"][0].update(kw)
    return apply


def finding0(**kw):
    def apply(d):
        d["findings"][0].update(kw)
    return apply


def raw_dup_key(d_text: str) -> str:
    return d_text.replace('"schema_version": "task-v2",', '"schema_version": "task-v2", "schema_version": "task-v2",', 1)


def raw_replace(old, new):
    def build(text):
        assert old in text, old
        return text.replace(old, new, 1)
    return build


def comb(*fns):
    def apply(d):
        for fn in fns:
            fn(d)
    return apply


def sha_case(kind, name, key, value):
    return (name, kind, setk(key, value), {"invalid_pattern", "invalid_sha"})


CASES = [
    # --- parser: strict JSON -------------------------------------------------------------
    ("dup_key_top", TASK, ("raw", raw_dup_key), {"duplicate_key"}),
    ("dup_key_nested", TASK, ("raw", raw_replace('"author": "synthetic-fixture",', '"author": "a", "author": "b",')), {"duplicate_key"}),
    ("nan_value", TASK, ("raw", raw_replace('"timeout_seconds": 300', '"timeout_seconds": NaN')), {"non_finite_number"}),
    ("infinity_value", TASK, ("raw", raw_replace('"timeout_seconds": 300', '"timeout_seconds": Infinity')), {"non_finite_number"}),
    ("neg_infinity_value", TASK, ("raw", raw_replace('"timeout_seconds": 300', '"timeout_seconds": -Infinity')), {"non_finite_number"}),
    ("float_overflow", EVID, ("raw", raw_replace('"duration_ms": 150', '"duration_ms": 1e999')), {"non_finite_number"}),
    ("trailing_comma", TASK, ("raw", raw_replace('"timeout_seconds": 300,', '"timeout_seconds": 300,,')), {"json_decode_error"}),
    ("deep_nesting", TASK, ("raw", lambda t: "[" * 200 + "]" * 200), {"nesting_too_deep"}),
    ("empty_file", TASK, ("raw", lambda t: ""), {"json_decode_error"}),
    ("root_is_list", TASK, ("raw", lambda t: "[]"), {"unknown_schema"}),
    # --- types: bool as integer, float as integer ---------------------------------------
    ("bool_timeout", TASK, setk("timeout_seconds", True), {"type_mismatch"}),
    ("float_timeout", TASK, setk("timeout_seconds", 300.0), {"type_mismatch"}),
    ("string_timeout", TASK, setk("timeout_seconds", "300"), {"type_mismatch"}),
    ("bool_exit_code", EVID, check0(exit_code=False), {"type_mismatch"}),
    ("bool_duration", EVID, check0(duration_ms=True), {"type_mismatch"}),
    ("bool_line", REV, finding0(line=True), {"type_mismatch"}),
    ("line_zero", REV, finding0(line=0), {"number_too_small"}),
    # --- timeouts, zero checks, caps ----------------------------------------------------
    ("timeout_missing", TASK, delk("timeout_seconds"), {"missing_required"}),
    ("timeout_zero", TASK, setk("timeout_seconds", 0), {"number_too_small"}),
    ("timeout_negative", TASK, setk("timeout_seconds", -5), {"number_too_small"}),
    ("timeout_huge", TASK, setk("timeout_seconds", 86401), {"number_too_large"}),
    ("zero_checks", EVID, setk("checks", []), {"array_too_short"}),
    ("checks_missing", EVID, delk("checks"), {"missing_required"}),
    ("required_checks_empty", TASK, setk("required_checks", []), {"array_too_short"}),
    ("caps_missing", TASK, delk("allowed_capabilities"), {"missing_required"}),
    ("caps_empty", TASK, setk("allowed_capabilities", []), {"array_too_short"}),
    ("caps_duplicate", TASK, setk("allowed_capabilities", ["workspace_read", "workspace_read"]), {"duplicate_array_item"}),
    ("caps_disallowed", TASK, setk("allowed_capabilities", ["workspace_read", "arbitrary_host_exec", "credential_mount"]), {"scope_violation", "invalid_enum_value"}),
    ("caps_unapproved", TASK, setk("allowed_capabilities", ["workspace_read", "web_access"]), {"unapproved_capability", "invalid_enum_value"}),
    ("caps_exec_without_isolation", TASK, setk("allowed_capabilities", ["workspace_write", "test_exec"]), {"missing_isolation_capability"}),
    ("allowed_paths_missing", TASK, delk("allowed_paths"), {"missing_required"}),
    ("allowed_paths_too_many", TASK, setk("allowed_paths", [f"src/p{i}/" for i in range(300)]), {"array_too_long"}),
    ("string_too_long", EVID, check0(message="x" * 3000), {"string_too_long"}),
    ("manifest_empty", EVID, setk("sha256_manifest", {}), {"object_too_small"}),
    ("policy_missing", TASK, delk("policy_version"), {"missing_required"}),
    ("policy_bad_pattern", TASK, setk("policy_version", "Bad Policy!"), {"invalid_pattern"}),
    # --- unknown fields ------------------------------------------------------------------
    ("unknown_task_field", TASK, setk("expected_commit_sha", CAND), {"unexpected_property"}),
    ("unknown_check_field", EVID, check0(trusted=True), {"unexpected_property"}),
    ("unknown_finding_field", REV, finding0(approved=True), {"unexpected_property"}),
    ("unknown_evidence_field", EVID, setk("authenticated", True), {"unexpected_property"}),
    # --- SHAs ----------------------------------------------------------------------------
    sha_case(TASK, "sha_short", "target_sha", "0123456789abcdef"),
    sha_case(TASK, "sha_upper", "target_sha", CAND.upper()),
    sha_case(TASK, "sha_trailing_newline", "target_sha", CAND + "\n"),
    sha_case(TASK, "sha_non_hex", "base_sha", "z" * 40),
    sha_case(EVID, "sha_commit_malformed", "commit_sha", "not-a-valid-sha-xyz-0123456789abcdef"),
    ("sha_int", TASK, setk("target_sha", 12345), {"type_mismatch", "invalid_sha"}),
    ("sha_all_zero", EVID, setk("commit_sha", "0" * 40), {"null_sha"}),
    ("digest_malformed", EVID, comb(lambda d: d["sha256_manifest"].update({"docs/x.md": "abc"})), {"invalid_digest"}),
    ("log_hash_malformed", EVID, check0(log_hash="abc"), {"invalid_pattern"}),
    # --- check semantics -----------------------------------------------------------------
    ("pass_with_failed_check", EVID, check0(status="fail", exit_code=1), {"verdict_check_mismatch"}),
    ("pass_with_timeout_check", EVID, check0(status="timeout", exit_code=124), {"verdict_check_mismatch"}),
    ("pass_status_nonzero_exit", EVID, check0(exit_code=2), {"status_exit_mismatch"}),
    ("fail_status_zero_exit", EVID, comb(setk("verdict", "fail"), check0(status="fail", exit_code=0)), {"status_exit_mismatch"}),
    ("skipped_in_pass", EVID, check0(status="skipped", exit_code=0), {"skipped_check_in_pass"}),
    ("fail_verdict_all_pass", EVID, setk("verdict", "fail"), {"verdict_check_mismatch"}),
    ("duplicate_check_id", EVID, comb(lambda d: d["checks"].append(copy.deepcopy(d["checks"][0]))), {"duplicate_check_id"}),
    ("verdict_uppercase_v2", EVID, setk("verdict", "PASS"), {"invalid_enum_value"}),
    ("unknown_check_id_evidence", EVID, check0(id="not_in_catalog"), {"unknown_check_id"}),
    ("unknown_check_id_task", TASK, setk("required_checks", ["fmt", "invented_check"]), {"unknown_check_id"}),
    ("duplicate_log_path", EVID, comb(lambda d: d["checks"][1].update({"log_path": d["checks"][0]["log_path"]})), {"duplicate_log_path"}),
    ("duration_negative", EVID, check0(duration_ms=-1), {"number_too_small"}),
    ("recorder_missing", EVID, delk("recorder"), {"missing_required"}),
    # --- paths: traversal, globs, normalization -----------------------------------------
    ("path_dotdot", TASK, setk("allowed_paths", ["../../etc/shadow"]), {"scope_violation"}),
    ("path_inner_dotdot", TASK, setk("allowed_paths", ["src/../secrets/"]), {"scope_violation"}),
    ("path_absolute", TASK, setk("allowed_paths", ["/etc/passwd"]), {"scope_violation"}),
    ("path_backslash_dotdot", TASK, setk("allowed_paths", ["src\\..\\x"]), {"scope_violation"}),
    ("path_double_slash", TASK, setk("allowed_paths", ["src//lib"]), {"unnormalized_path"}),
    ("path_leading_dot", TASK, setk("allowed_paths", ["./src/"]), {"unnormalized_path"}),
    ("path_dot_component", TASK, setk("allowed_paths", ["src/./lib"]), {"unnormalized_path"}),
    ("path_glob_star", TASK, setk("allowed_paths", ["*"]), {"unauthorized_glob"}),
    ("path_glob_recursive", TASK, setk("allowed_paths", ["src/**"]), {"unauthorized_glob"}),
    ("path_glob_brace", TASK, setk("allowed_paths", ["src/{a,b}"]), {"unauthorized_glob"}),
    ("path_whole_repo_dot", TASK, setk("allowed_paths", ["."]), {"broad_path"}),
    ("path_whole_repo_slash", TASK, setk("allowed_paths", ["/"]), {"broad_path", "scope_violation"}),
    ("path_empty", TASK, setk("allowed_paths", [""]), {"broad_path", "string_too_short"}),
    ("path_nul", TASK, setk("allowed_paths", ["src/a\u0000b"]), {"invalid_path_char"}),
    ("path_newline", TASK, setk("allowed_paths", ["src/a\n"]), {"invalid_path_char", "invalid_pattern"}),
    ("path_percent_encoded", TASK, setk("allowed_paths", ["%2e%2e/etc"]), {"invalid_pattern"}),
    ("path_allowed_under_protected", TASK, comb(setk("allowed_paths", ["src/secret/x.rs"]), setk("protected_paths", ["src/secret/"])), {"path_conflict"}),
    ("path_duplicate", TASK, setk("allowed_paths", ["src/", "src/"]), {"duplicate_array_item"}),
    ("manifest_traversal", EVID, comb(lambda d: d["sha256_manifest"].update({"../etc/passwd": "0" * 64})), {"scope_violation"}),
    ("manifest_absolute", EVID, comb(lambda d: d["sha256_manifest"].update({"/etc/passwd": "0" * 64})), {"scope_violation"}),
    ("log_path_traversal", EVID, check0(log_path="../outside.txt"), {"scope_violation"}),
    ("finding_path_traversal", REV, finding0(path="../../etc/passwd"), {"scope_violation"}),
    # --- calendar and timestamps ---------------------------------------------------------
    ("ts_feb30", EVID, setk("timestamp", "2026-02-30T10:00:00Z"), {"invalid_timestamp"}),
    ("ts_month13", EVID, setk("timestamp", "2026-13-01T10:00:00Z"), {"invalid_timestamp"}),
    ("ts_apr31", EVID, setk("timestamp", "2026-04-31T10:00:00Z"), {"invalid_timestamp"}),
    ("ts_feb29_non_leap", EVID, setk("timestamp", "2025-02-29T10:00:00Z"), {"invalid_timestamp"}),
    ("ts_hour24", EVID, setk("timestamp", "2026-07-21T24:00:00Z"), {"invalid_timestamp"}),
    ("ts_minute60", EVID, setk("timestamp", "2026-07-21T23:60:00Z"), {"invalid_timestamp"}),
    ("ts_second60", EVID, setk("timestamp", "2026-07-21T23:59:60Z"), {"invalid_timestamp"}),
    ("ts_offset", EVID, setk("timestamp", "2026-07-21T18:00:00+00:00"), {"invalid_timestamp", "invalid_pattern"}),
    ("ts_lowercase_t", EVID, setk("timestamp", "2026-07-21t18:00:00Z"), {"invalid_timestamp", "invalid_pattern"}),
    ("ts_fraction_too_long", EVID, setk("timestamp", "2026-07-21T18:00:00.1234567890Z"), {"invalid_timestamp", "invalid_pattern"}),
    ("ts_trailing_newline", EVID, setk("timestamp", "2026-07-21T18:00:00Z\n"), {"invalid_timestamp", "invalid_pattern"}),
    ("ts_in_future", EVID, setk("timestamp", "2026-12-31T00:00:00Z"), {"timestamp_in_future"}),
    ("ts_review_feb30", REV, setk("timestamp", "2026-02-30T10:00:00Z"), {"invalid_timestamp"}),
    # --- review semantics ----------------------------------------------------------------
    ("review_pass_blocker", REV, finding0(severity="blocker"), {"pass_with_blocking_finding"}),
    ("review_pass_critical", REV, finding0(severity="critical"), {"pass_with_blocking_finding"}),
    ("review_pass_high", REV, finding0(severity="high"), {"pass_with_blocking_finding"}),
    ("review_comment_blocker", REV, comb(setk("verdict", "comment"), finding0(severity="blocker")), {"comment_with_blocking_finding"}),
    ("review_fail_no_findings", REV, comb(setk("verdict", "fail"), setk("findings", [])), {"fail_without_findings"}),
    ("review_duplicate_finding_id", REV, comb(lambda d: d["findings"].append(copy.deepcopy(d["findings"][0]))), {"duplicate_finding_id"}),
    ("review_duplicate_lane", REV, setk("lanes", ["security", "security"]), {"duplicate_array_item"}),
    ("review_intensity_alias", REV, setk("intensity", "ultra"), {"invalid_enum_value"}),
    ("review_verdict_uppercase", REV, setk("verdict", "PASS"), {"invalid_enum_value"}),
    ("review_severity_warn", REV, finding0(severity="warn"), {"invalid_enum_value"}),
    ("review_reviewer_missing", REV, delk("reviewer"), {"missing_required"}),
]

NEGATIVE_NAMES = [c[0] for c in CASES]


class CaseFiles:
    """Materializes every table case as a file in a temp dir (shared by several tests)."""

    def __init__(self, root: Path):
        self.root = root
        self.entries = []
        for name, fx, build, _codes in CASES:
            text = (FIXTURES / fx).read_text(encoding="utf-8")
            if isinstance(build, tuple):
                text = build[1](text)
            else:
                text = mutated(fx, build)
            path = root / f"{name}.json"
            path.write_text(text, encoding="utf-8")
            self.entries.append((name, path))
        # v1 review: PASS with a blocking finding must be rejected by the old schema too
        v1 = fixture("valid_review.json")
        v1["findings"][0]["severity"] = "high"
        p = root / "v1_review_pass_high.json"
        p.write_text(json.dumps(v1), encoding="utf-8")
        self.entries.append(("v1_review_pass_high", p))
        v1e = fixture("valid_evidence.json")
        v1e["checks"][0].update({"status": "fail", "exit_code": 1})
        p = root / "v1_evidence_pass_failed_check.json"
        p.write_text(json.dumps(v1e), encoding="utf-8")
        self.entries.append(("v1_evidence_pass_failed_check", p))


def run(path, *, schema=None, exp=None, use_jsonschema=True, schemas_dir=SCHEMAS):
    exp = exp or V.Expectations(now=FIXED_NOW)
    return V.validate_file_detailed(Path(path), schema, schemas_dir, use_jsonschema=use_jsonschema, expectations=exp)


class TmpDirCase(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="h2-validate-"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)

    def write(self, name, obj_or_text):
        p = self.tmp / name
        p.write_text(obj_or_text if isinstance(obj_or_text, str) else json.dumps(obj_or_text), encoding="utf-8")
        return p

    def assertCodes(self, result, expected, msg=""):
        missing = set(expected) - codes(result)
        self.assertFalse(result.ok, f"expected rejection {msg}")
        self.assertFalse(missing, f"missing codes {missing}; got {[str(e) for e in result.errors]} {msg}")


# ---------------------------------------------------------------------------------------------
# 1. Valid fixtures and fail-closed table
# ---------------------------------------------------------------------------------------------

class ValidFixtureTests(TmpDirCase):
    def test_valid_v2_fixtures_pass_structurally(self):
        for name in (TASK, EVID, REV):
            with self.subTest(name):
                r = run(FIXTURES / name)
                self.assertTrue(r.ok, [str(e) for e in r.errors])

    def test_valid_v1_fixtures_pass_but_are_marked_historical(self):
        for name in ("valid_task.json", "valid_evidence.json", "valid_review.json"):
            with self.subTest(name):
                r = run(FIXTURES / name)
                self.assertTrue(r.ok, [str(e) for e in r.errors])
                self.assertIn("historical_v1", {w.split(":")[0] for w in r.notes})

    def test_leap_day_and_fractional_seconds_are_accepted(self):
        d = fixture(EVID)
        d["timestamp"] = "2024-02-29T23:59:59.123456789Z"
        self.assertTrue(run(self.write("leap.json", d)).ok)

    def test_validation_never_claims_trust(self):
        for name in (TASK, EVID, REV, "valid_evidence.json"):
            r = run(FIXTURES / name)
            self.assertEqual(r.trust, "structure_only")
            payload = r.to_dict()
            self.assertEqual(payload["trust"], "structure_only")
            self.assertNotIn("trusted", payload)


class FailClosedTableTests(TmpDirCase):
    @classmethod
    def setUpClass(cls):
        cls._tmp = Path(tempfile.mkdtemp(prefix="h2-cases-"))
        cls.files = CaseFiles(cls._tmp)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls._tmp, ignore_errors=True)

    def test_every_adversarial_case_is_rejected_with_expected_codes(self):
        by_name = {c[0]: c[3] for c in CASES}
        by_name["v1_review_pass_high"] = {"pass_with_blocking_finding"}
        by_name["v1_evidence_pass_failed_check"] = {"verdict_check_mismatch"}
        self.assertGreaterEqual(len(self.files.entries), 100)
        for name, path in self.files.entries:
            with self.subTest(name):
                r = run(path, schema="review-v1" if name == "v1_review_pass_high" else (
                    "evidence-v1" if name == "v1_evidence_pass_failed_check" else None))
                self.assertCodes(r, by_name[name], name)

    def test_parse_failures_stop_before_schema_validation(self):
        r = run(self._tmp / "dup_key_top.json")
        self.assertEqual(codes(r), {"duplicate_key"})


# ---------------------------------------------------------------------------------------------
# 2. Strict parser details
# ---------------------------------------------------------------------------------------------

class StrictParserTests(TmpDirCase):
    def test_parse_json_strict_rejects_duplicates_and_constants(self):
        for text, code in (
            ('{"a":1,"a":2}', "duplicate_key"),
            ('{"a":{"b":1,"b":1}}', "duplicate_key"),
            ('[NaN]', "non_finite_number"),
            ('[Infinity]', "non_finite_number"),
            ('[-Infinity]', "non_finite_number"),
            ('[1e999]', "non_finite_number"),
            ('[-1e999]', "non_finite_number"),
        ):
            with self.subTest(text):
                with self.assertRaises(V.StrictJsonError) as cm:
                    V.parse_json_strict(text)
                self.assertEqual(cm.exception.code, code)

    def test_parse_json_strict_keeps_valid_documents(self):
        self.assertEqual(V.parse_json_strict('{"a":[1,2.5,true,null,"x"]}'), {"a": [1, 2.5, True, None, "x"]})

    def test_invalid_utf8_and_bom_are_rejected(self):
        p = self.tmp / "bad.json"
        p.write_bytes(b'{"a": "\xff"}')
        self.assertIn("invalid_utf8", codes(run(p)))
        p.write_bytes(b"\xef\xbb\xbf" + json.dumps(fixture(TASK)).encode())
        self.assertFalse(run(p).ok)

    def test_oversized_file_is_rejected_without_parsing(self):
        p = self.tmp / "big.json"
        p.write_text('{"x": "' + "a" * (V.MAX_FILE_BYTES + 10) + '"}', encoding="utf-8")
        self.assertIn("file_too_large", codes(run(p)))

    def test_symlink_and_directory_inputs_are_rejected(self):
        target = self.write("real.json", fixture(TASK))
        link = self.tmp / "link.json"
        link.symlink_to(target)
        self.assertIn("unsafe_input_file", codes(run(link)))
        self.assertIn("unsafe_input_file", codes(run(self.tmp)))

    def test_missing_file(self):
        self.assertIn("file_not_found", codes(run(self.tmp / "nope.json")))


# ---------------------------------------------------------------------------------------------
# 3. Pure validator semantics (strict equality, unknown keywords)
# ---------------------------------------------------------------------------------------------

class PureValidatorTests(unittest.TestCase):
    def validate(self, schema, instance):
        return V.PureSchemaValidator(schema).validate(instance)

    def test_bool_is_not_an_integer_or_number(self):
        self.assertTrue(self.validate({"type": "integer"}, True))
        self.assertTrue(self.validate({"type": "number"}, False))
        self.assertTrue(self.validate({"type": "integer"}, 1.0))
        self.assertFalse(self.validate({"type": "integer"}, 1))

    def test_enum_and_const_do_not_conflate_bool_and_int(self):
        self.assertTrue(self.validate({"enum": [1]}, True))
        self.assertTrue(self.validate({"enum": [0]}, False))
        self.assertTrue(self.validate({"const": 1}, True))
        self.assertTrue(self.validate({"const": True}, 1))
        self.assertFalse(self.validate({"enum": [1]}, 1))
        self.assertFalse(self.validate({"const": True}, True))

    def test_unique_items_distinguishes_bool_from_int(self):
        self.assertFalse(self.validate({"type": "array", "uniqueItems": True}, [1, True]))
        self.assertTrue(self.validate({"type": "array", "uniqueItems": True}, [1, 1]))

    def test_pattern_rejects_trailing_newline_and_control_characters(self):
        schema = {"type": "string", "pattern": "^[0-9a-f]{4}$"}
        self.assertTrue(self.validate(schema, "abcd\n"))
        self.assertFalse(self.validate(schema, "abcd"))

    def test_unknown_type_name_fails_closed(self):
        errs = self.validate({"type": "strng"}, "x")
        self.assertIn("unsupported_schema_keyword", {e.code for e in errs})

    def test_unsupported_keywords_are_reported_not_ignored(self):
        for schema in (
            {"oneOf": [{"type": "string"}]},
            {"anyOf": [{"type": "string"}]},
            {"allOf": [{"type": "string"}]},
            {"not": {"type": "string"}},
            {"$ref": "#/definitions/x"},
            {"if": {"type": "string"}, "then": {"minLength": 1}},
            {"format": "date-time"},
            {"patternProperties": {"^a": {"type": "string"}}},
            {"dependentRequired": {"a": ["b"]}},
            {"contains": {"type": "string"}},
            {"multipleOf": 2},
            {"exclusiveMinimum": 0},
            {"properties": {"a": {"oneOf": []}}},
            {"items": {"$ref": "#/x"}},
            {"additionalProperties": {"format": "uri"}},
            {"propertyNames": {"format": "uri"}},
        ):
            with self.subTest(schema=schema):
                errs = self.validate(schema, "anything")
                self.assertIn("unsupported_schema_keyword", {e.code for e in errs})

    def test_caps_keywords_are_enforced(self):
        self.assertIn("array_too_long", {e.code for e in self.validate({"type": "array", "maxItems": 1}, [1, 2])})
        self.assertIn("object_too_large", {e.code for e in self.validate({"type": "object", "maxProperties": 1}, {"a": 1, "b": 2})})
        self.assertIn("object_too_small", {e.code for e in self.validate({"type": "object", "minProperties": 1}, {})})

    def test_property_names_apply_full_schema_to_keys(self):
        schema = {"type": "object", "propertyNames": {"type": "string", "maxLength": 3}}
        self.assertTrue(self.validate(schema, {"toolong": 1}))
        self.assertFalse(self.validate(schema, {"ok": 1}))


# ---------------------------------------------------------------------------------------------
# 4. Schema lint: every v2 schema carries explicit caps
# ---------------------------------------------------------------------------------------------

class SchemaCapsLintTests(unittest.TestCase):
    def walk(self, node, path, problems):
        if not isinstance(node, dict):
            return
        t = node.get("type")
        if t == "array" and "maxItems" not in node:
            problems.append(f"{path}: array without maxItems")
        if t == "string" and not ({"maxLength", "const", "enum"} & set(node)):
            problems.append(f"{path}: string without maxLength")
        if t == "integer" and not ("minimum" in node and "maximum" in node):
            problems.append(f"{path}: integer without minimum/maximum")
        if t == "object":
            if "additionalProperties" not in node:
                problems.append(f"{path}: object without explicit additionalProperties")
            if node.get("additionalProperties") is not False and "maxProperties" not in node:
                problems.append(f"{path}: open object without maxProperties")
        for key, sub in node.get("properties", {}).items():
            self.walk(sub, f"{path}.{key}", problems)
        for key in ("items", "propertyNames"):
            self.walk(node.get(key), f"{path}.{key}", problems)
        if isinstance(node.get("additionalProperties"), dict):
            self.walk(node["additionalProperties"], f"{path}.*", problems)

    def test_v2_schemas_are_fully_capped(self):
        for name in ("task-v2", "evidence-v2", "review-v2"):
            with self.subTest(name):
                schema = json.loads((SCHEMAS / f"{name}.schema.json").read_text(encoding="utf-8"))
                problems = []
                self.walk(schema, name, problems)
                self.assertEqual(problems, [])
                self.assertEqual(V.find_unsupported_keywords(schema), [])
                self.assertEqual(schema["properties"]["schema_version"]["const"], name)

    def test_all_schemas_use_only_supported_keywords(self):
        for p in sorted(SCHEMAS.glob("*.schema.json")):
            with self.subTest(p.name):
                self.assertEqual(V.find_unsupported_keywords(json.loads(p.read_text(encoding="utf-8"))), [])


# ---------------------------------------------------------------------------------------------
# 5. Schema resolution: no payload-driven schema selection
# ---------------------------------------------------------------------------------------------

class SchemaResolutionTests(TmpDirCase):
    def test_missing_schema_version_without_explicit_schema_fails_closed(self):
        d = fixture(TASK)
        del d["schema_version"]
        r = run(self.write("task.json", d))
        self.assertCodes(r, {"missing_schema_version"})

    def test_filename_and_key_markers_do_not_select_a_schema(self):
        d = fixture(EVID)
        del d["schema_version"]
        r = run(self.write("evidence-receipt.json", d))
        self.assertCodes(r, {"missing_schema_version"})

    def test_explicit_schema_must_match_payload_schema_version(self):
        r = run(FIXTURES / TASK, schema="evidence-v2")
        self.assertCodes(r, {"schema_mismatch"})
        r = run(FIXTURES / "valid_task.json", schema="task-v2")
        self.assertCodes(r, {"schema_mismatch"})

    def test_short_schema_name_requires_payload_version(self):
        d = fixture(TASK)
        del d["schema_version"]
        r = run(self.write("t.json", d), schema="task")
        self.assertCodes(r, {"missing_schema_version"})
        self.assertTrue(run(FIXTURES / TASK, schema="task").ok)

    def test_unknown_schema_version(self):
        d = fixture(TASK)
        d["schema_version"] = "task-v9"
        self.assertCodes(run(self.write("t.json", d)), {"unknown_schema"})

    def test_semantics_are_keyed_on_schema_not_on_payload_keys(self):
        # An evidence payload that carries task-like keys must not trigger (or dodge) task rules.
        d = fixture(EVID)
        d["allowed_capabilities"] = ["arbitrary_host_exec"]
        r = run(self.write("e.json", d))
        self.assertCodes(r, {"unexpected_property"})
        self.assertNotIn("scope_violation", codes(r))

    def test_historical_v1_without_schema_version_needs_explicit_schema(self):
        d = fixture("valid_task.json")
        del d["schema_version"]
        p = self.write("t.json", d)
        self.assertCodes(run(p), {"missing_schema_version"})
        r = run(p, schema="task-v1")
        self.assertTrue(r.ok)
        self.assertIn("historical_v1", {w.split(":")[0] for w in r.notes})


# ---------------------------------------------------------------------------------------------
# 6. External expectations (caller-supplied, never read from the payload)
# ---------------------------------------------------------------------------------------------

class ExpectationTests(TmpDirCase):
    def exp(self, **kw):
        kw.setdefault("now", FIXED_NOW)
        return V.Expectations(**kw)

    def test_matching_expectations_are_applied_and_reported(self):
        r = run(FIXTURES / EVID, exp=self.exp(candidate_sha=CAND, base_sha=BASE, tree_sha=TREE, policy_version=POLICY))
        self.assertTrue(r.ok, [str(e) for e in r.errors])
        for key in ("candidate_sha", "base_sha", "tree_sha", "policy_version"):
            self.assertTrue(r.applied[key], key)

    def test_unapplied_expectations_are_reported_as_a_warning_not_trust(self):
        r = run(FIXTURES / EVID)
        self.assertTrue(r.ok)
        self.assertFalse(any(r.applied.values()))
        self.assertIn("external_expectations_missing", {w.split(":")[0] for w in r.notes})

    def test_candidate_sha_mismatch_per_kind(self):
        cases = (
            (EVID, "commit_sha"),
            (REV, "head_sha"),
            (TASK, "target_sha"),
            ("valid_evidence.json", "commit_sha"),
            ("valid_review.json", "head_sha"),
            ("valid_task.json", "target_sha"),
        )
        for name, field in cases:
            with self.subTest(name):
                r = run(FIXTURES / name, exp=self.exp(candidate_sha=OTHER))
                self.assertCodes(r, {"expected_sha_mismatch"})
                self.assertIn(field, {e.field for e in r.errors if e.code == "expected_sha_mismatch"})

    def test_base_and_tree_mismatch(self):
        r = run(FIXTURES / EVID, exp=self.exp(base_sha=OTHER, tree_sha=OTHER))
        self.assertEqual(sorted(e.field for e in r.errors if e.code == "expected_sha_mismatch"), ["base_sha", "tree_sha"])

    def test_payload_cannot_supply_its_own_expectation(self):
        d = fixture(EVID)
        d["commit_sha"] = OTHER  # payload self-consistent, but not the candidate the caller expects
        r = run(self.write("e.json", d), exp=self.exp(candidate_sha=CAND))
        self.assertCodes(r, {"expected_sha_mismatch"})

    def test_expected_policy_version_mismatch_and_missing(self):
        r = run(FIXTURES / EVID, exp=self.exp(policy_version="harness-other-v9"))
        self.assertCodes(r, {"expected_policy_mismatch"})
        r = run(FIXTURES / "valid_evidence.json", exp=self.exp(policy_version=POLICY))
        self.assertCodes(r, {"missing_expected_field"})  # v1 carries no policy_version

    def test_expected_tree_sha_on_v1_evidence_without_tree(self):
        d = fixture("valid_evidence.json")
        del d["tree_sha"]
        r = run(self.write("e.json", d), exp=self.exp(tree_sha=TREE))
        self.assertCodes(r, {"missing_expected_field"})

    def test_malformed_expectations_are_rejected_at_construction(self):
        for kw in ({"candidate_sha": "abc"}, {"base_sha": CAND.upper()}, {"tree_sha": ""}, {"policy_version": "Bad!"},
                   {"catalog_sha256": "xyz"}):
            with self.subTest(kw):
                with self.assertRaises(ValueError):
                    V.Expectations(**kw)

    def test_future_timestamp_uses_the_injected_clock(self):
        d = fixture(EVID)
        d["timestamp"] = "2026-07-31T23:59:00Z"
        self.assertTrue(run(self.write("e.json", d), exp=self.exp()).ok)
        d["timestamp"] = "2026-08-01T01:00:00Z"
        self.assertCodes(run(self.write("e2.json", d), exp=self.exp()), {"timestamp_in_future"})


class TaskCoverageTests(TmpDirCase):
    def exp(self, task=None, **kw):
        return V.Expectations(task=task or fixture(TASK), now=FIXED_NOW, **kw)

    def test_all_required_checks_passing_is_accepted(self):
        r = run(FIXTURES / EVID, exp=self.exp())
        self.assertTrue(r.ok, [str(e) for e in r.errors])
        self.assertTrue(r.applied["task"])

    def test_missing_required_check(self):
        d = fixture(EVID)
        d["checks"] = [c for c in d["checks"] if c["id"] != "doctor"]
        r = run(self.write("e.json", d), exp=self.exp())
        self.assertCodes(r, {"missing_required_check"})

    def test_required_check_not_passed(self):
        d = fixture(EVID)
        d["verdict"] = "warn"
        d["checks"][3].update({"status": "warn"})
        r = run(self.write("e.json", d), exp=self.exp())
        self.assertCodes(r, {"required_check_not_passed"})

    def test_task_id_mismatch(self):
        t = fixture(TASK)
        t["task_id"] = "other-task"
        self.assertCodes(run(FIXTURES / EVID, exp=self.exp(task=t)), {"task_id_mismatch"})

    def test_check_duration_over_task_timeout(self):
        t = fixture(TASK)
        t["timeout_seconds"] = 1
        r = run(FIXTURES / EVID, exp=self.exp(task=t))
        self.assertCodes(r, {"check_exceeds_timeout"})

    def test_task_target_must_match_evidence_commit(self):
        t = fixture(TASK)
        t["target_sha"] = OTHER
        self.assertCodes(run(FIXTURES / EVID, exp=self.exp(task=t)), {"task_sha_mismatch"})

    def test_task_coverage_does_not_apply_to_other_kinds(self):
        r = run(FIXTURES / REV, exp=self.exp())
        self.assertTrue(r.ok)


class CatalogTests(TmpDirCase):
    def test_default_catalog_has_every_fixture_check_id(self):
        cat, errs = V.load_check_catalog(SCHEMAS / "check-catalog-v1.json")
        self.assertEqual(errs, [])
        self.assertTrue({"fmt", "clippy", "test_lib", "doctor"} <= cat)

    def test_catalog_hash_pin(self):
        digest = hashlib.sha256((SCHEMAS / "check-catalog-v1.json").read_bytes()).hexdigest()
        exp = V.Expectations(catalog_sha256=digest, now=FIXED_NOW)
        r = run(FIXTURES / EVID, exp=exp)
        self.assertTrue(r.ok, [str(e) for e in r.errors])
        self.assertTrue(r.applied["catalog_sha256"])
        bad = V.Expectations(catalog_sha256="0" * 64, now=FIXED_NOW)
        self.assertCodes(run(FIXTURES / EVID, exp=bad), {"catalog_sha256_mismatch"})

    def test_alternative_catalog_without_the_check_rejects(self):
        cat = {"catalog_version": "check-catalog-v1", "checks": {"fmt": {"description": "x"}}}
        p = self.write("cat.json", cat)
        exp = V.Expectations(catalog_path=p, now=FIXED_NOW)
        self.assertCodes(run(FIXTURES / EVID, exp=exp), {"unknown_check_id"})

    def test_malformed_catalogs_fail_closed(self):
        bad = {
            "dup": '{"catalog_version":"check-catalog-v1","checks":{"fmt":{"description":"x"},"fmt":{"description":"y"}}}',
            "version": json.dumps({"catalog_version": "other", "checks": {"fmt": {"description": "x"}}}),
            "empty": json.dumps({"catalog_version": "check-catalog-v1", "checks": {}}),
            "badid": json.dumps({"catalog_version": "check-catalog-v1", "checks": {"../x": {"description": "x"}}}),
            "extra": json.dumps({"catalog_version": "check-catalog-v1", "checks": {"fmt": {"description": "x"}}, "extra": 1}),
            "entry": json.dumps({"catalog_version": "check-catalog-v1", "checks": {"fmt": "x"}}),
            "nan": '{"catalog_version":"check-catalog-v1","checks":{"fmt":{"description":NaN}}}',
        }
        for name, text in bad.items():
            with self.subTest(name):
                cat, errs = V.load_check_catalog(self.write(f"{name}.json", text))
                self.assertIsNone(cat)
                self.assertTrue(errs)
        exp = V.Expectations(catalog_path=self.write("dup.json", bad["dup"]), now=FIXED_NOW)
        self.assertCodes(run(FIXTURES / EVID, exp=exp), {"catalog_invalid"})


class LogHashTests(TmpDirCase):
    def logs(self):
        d = self.tmp / "logs"
        shutil.copytree(FIXTURES / "check-logs", d)
        return d

    def exp(self, d):
        return V.Expectations(logs_dir=d, now=FIXED_NOW)

    def test_matching_logs_are_verified(self):
        r = run(FIXTURES / EVID, exp=self.exp(self.logs()))
        self.assertTrue(r.ok, [str(e) for e in r.errors])
        self.assertTrue(r.applied["logs"])

    def test_without_logs_dir_hashes_stay_unverified(self):
        r = run(FIXTURES / EVID)
        self.assertFalse(r.applied["logs"])
        self.assertIn("external_expectations_missing", {w.split(":")[0] for w in r.notes})

    def test_tampered_log(self):
        d = self.logs()
        (d / "fmt.txt").write_text("tampered\n", encoding="utf-8")
        self.assertCodes(run(FIXTURES / EVID, exp=self.exp(d)), {"log_hash_mismatch"})

    def test_missing_log(self):
        d = self.logs()
        (d / "clippy.txt").unlink()
        self.assertCodes(run(FIXTURES / EVID, exp=self.exp(d)), {"log_file_missing"})

    def test_symlinked_log_is_rejected_even_with_matching_bytes(self):
        d = self.logs()
        real = self.tmp / "elsewhere.txt"
        real.write_bytes((d / "fmt.txt").read_bytes())
        (d / "fmt.txt").unlink()
        (d / "fmt.txt").symlink_to(real)
        self.assertCodes(run(FIXTURES / EVID, exp=self.exp(d)), {"log_path_unsafe"})

    def test_log_that_is_a_directory(self):
        d = self.logs()
        (d / "fmt.txt").unlink()
        (d / "fmt.txt").mkdir()
        self.assertCodes(run(FIXTURES / EVID, exp=self.exp(d)), {"log_path_unsafe"})

    def test_log_path_cannot_escape_logs_dir(self):
        d = self.logs()
        outside = self.tmp / "outside.txt"
        outside.write_bytes((d / "fmt.txt").read_bytes())
        e = fixture(EVID)
        e["checks"][0]["log_path"] = "../outside.txt"
        r = run(self.write("e.json", e), exp=self.exp(d))
        self.assertCodes(r, {"scope_violation"})
        self.assertNotIn("log_hash_mismatch", codes(r))

    def test_log_hash_is_computed_over_bytes(self):
        d = self.logs()
        data = (d / "doctor.txt").read_bytes()
        (d / "doctor.txt").write_bytes(data + b"\n")
        self.assertCodes(run(FIXTURES / EVID, exp=self.exp(d)), {"log_hash_mismatch"})


# ---------------------------------------------------------------------------------------------
# 7. jsonschema present vs absent
# ---------------------------------------------------------------------------------------------

RUNNER = r'''
import importlib.util, json, sys, datetime
mode, script, manifest = sys.argv[1:4]
if mode == "absent":
    sys.modules["jsonschema"] = None          # import blocker: `import jsonschema` raises ImportError
spec = importlib.util.spec_from_file_location("validate_evidence", script)
V = importlib.util.module_from_spec(spec); sys.modules["validate_evidence"] = V
spec.loader.exec_module(V)
now = datetime.datetime(2026, 8, 1, tzinfo=datetime.timezone.utc)
out = {}
for case in json.load(open(manifest)):
    exp = V.Expectations(now=now)
    r = V.validate_file_detailed(__import__("pathlib").Path(case["path"]), case.get("schema"),
                                 __import__("pathlib").Path(case["schemas_dir"]), expectations=exp)
    out[case["name"]] = [r.ok, r.schema, sorted([e.code, e.field] for e in r.errors)]
print(json.dumps({"has_jsonschema": V.HAS_JSONSCHEMA, "results": out}))
'''


class JsonschemaModeTests(TmpDirCase):
    @classmethod
    def setUpClass(cls):
        cls._tmp = Path(tempfile.mkdtemp(prefix="h2-equiv-"))
        cls.files = CaseFiles(cls._tmp)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls._tmp, ignore_errors=True)

    def corpus(self):
        items = [{"name": n, "path": str(p), "schemas_dir": str(SCHEMAS),
                  "schema": "review-v1" if n == "v1_review_pass_high" else ("evidence-v1" if n == "v1_evidence_pass_failed_check" else None)}
                 for n, p in self.files.entries]
        for fx in ("valid_task.json", "valid_evidence.json", "valid_review.json", TASK, EVID, REV):
            items.append({"name": f"fixture:{fx}", "path": str(FIXTURES / fx), "schemas_dir": str(SCHEMAS), "schema": None})
        return items

    def run_mode(self, mode):
        manifest = self.tmp / f"manifest-{mode}.json"
        manifest.write_text(json.dumps(self.corpus()), encoding="utf-8")
        script = self.tmp / "runner.py"
        script.write_text(RUNNER, encoding="utf-8")
        proc = subprocess.run([sys.executable, str(script), mode, str(BIN), str(manifest)],
                              capture_output=True, text=True, timeout=300, cwd=str(REPO_ROOT))
        self.assertEqual(proc.returncode, 0, proc.stderr)
        return json.loads(proc.stdout)

    def test_jsonschema_is_installed_for_the_present_mode(self):
        # Never skipped: the present-mode half of the equivalence claim needs the real library.
        self.assertTrue(V.HAS_JSONSCHEMA, "jsonschema must be installed to exercise the present mode "
                        "(pip install jsonschema, or apt install python3-jsonschema)")

    def test_import_blocker_really_hides_jsonschema(self):
        absent = self.run_mode("absent")
        present = self.run_mode("present")
        self.assertFalse(absent["has_jsonschema"])
        self.assertTrue(present["has_jsonschema"])

    def test_results_are_identical_with_and_without_jsonschema(self):
        present = self.run_mode("present")
        absent = self.run_mode("absent")
        self.assertEqual(present["results"].keys(), absent["results"].keys())
        self.assertGreaterEqual(len(present["results"]), 100)
        for name in present["results"]:
            with self.subTest(name):
                self.assertEqual(present["results"][name], absent["results"][name])

    def test_in_process_toggle_matches_too(self):
        for name, path in self.files.entries:
            if name.startswith("v1_"):
                continue
            with self.subTest(name):
                a = run(path, use_jsonschema=True)
                b = run(path, use_jsonschema=False)
                self.assertEqual((a.ok, sorted((e.code, e.field) for e in a.errors)),
                                 (b.ok, sorted((e.code, e.field) for e in b.errors)))

    def schema_dir_with(self, mutate):
        d = self.tmp / "schemas"
        shutil.copytree(SCHEMAS, d, dirs_exist_ok=True)
        p = d / "evidence-v2.schema.json"
        schema = json.loads(p.read_text(encoding="utf-8"))
        mutate(schema)
        p.write_text(json.dumps(schema), encoding="utf-8")
        return d

    def test_unsupported_keyword_fails_closed_in_both_modes(self):
        def add_one_of(s):
            s["properties"]["verdict"] = {"oneOf": [{"type": "string"}, {"type": "integer"}]}
        d = self.schema_dir_with(add_one_of)
        for use in (True, False):
            with self.subTest(use_jsonschema=use):
                r = run(FIXTURES / EVID, use_jsonschema=use, schemas_dir=d)
                self.assertCodes(r, {"unsupported_schema_keyword"})
                self.assertEqual(codes(r), {"unsupported_schema_keyword"})

    def test_nested_unsupported_keyword_and_unknown_dialect(self):
        d = self.schema_dir_with(lambda s: s["properties"]["checks"]["items"]["properties"]["id"].update({"format": "uri"}))
        self.assertCodes(run(FIXTURES / EVID, schemas_dir=d), {"unsupported_schema_keyword"})
        d = self.schema_dir_with(lambda s: s.update({"$schema": "https://example.test/custom-dialect"}))
        self.assertCodes(run(FIXTURES / EVID, schemas_dir=d), {"unsupported_schema_dialect"})

    def test_unsupported_keyword_also_blocks_the_absent_mode_subprocess(self):
        d = self.schema_dir_with(lambda s: s.update({"not": {"type": "null"}}))
        manifest = self.tmp / "m.json"
        manifest.write_text(json.dumps([{"name": "x", "path": str(FIXTURES / EVID), "schemas_dir": str(d), "schema": None}]), encoding="utf-8")
        script = self.tmp / "runner2.py"
        script.write_text(RUNNER, encoding="utf-8")
        outs = []
        for mode in ("present", "absent"):
            proc = subprocess.run([sys.executable, str(script), mode, str(BIN), str(manifest)], capture_output=True, text=True, timeout=120)
            self.assertEqual(proc.returncode, 0, proc.stderr)
            outs.append(json.loads(proc.stdout)["results"]["x"])
        self.assertEqual(outs[0], outs[1])
        self.assertFalse(outs[0][0])
        self.assertEqual(outs[0][2][0][0], "unsupported_schema_keyword")

    def test_jsonschema_exception_is_fail_closed_not_ignored(self):
        class Boom:
            class validators:
                @staticmethod
                def validator_for(_schema):
                    raise RuntimeError("boom")
        r = V.validate_file_detailed(FIXTURES / TASK, None, SCHEMAS, use_jsonschema=True,
                                     expectations=V.Expectations(now=FIXED_NOW), jsonschema_module=Boom)
        self.assertCodes(r, {"jsonschema_exception"})

    def test_divergence_between_validators_fails_closed(self):
        class FakeErr:
            path = ["timeout_seconds"]
            message = "stub says no"

        class FakeTypeChecker:
            def redefine(self, *a, **k):
                return self

        class FakeValidator:
            TYPE_CHECKER = FakeTypeChecker()

            def __init__(self, schema, *a, **k):
                pass

            def iter_errors(self, instance):
                return iter([FakeErr()])

        class Fake:
            class validators:
                @staticmethod
                def validator_for(_schema):
                    return FakeValidator

                @staticmethod
                def extend(cls, **kw):
                    return cls
        r = V.validate_file_detailed(FIXTURES / TASK, None, SCHEMAS, use_jsonschema=True,
                                     expectations=V.Expectations(now=FIXED_NOW), jsonschema_module=Fake)
        self.assertCodes(r, {"validator_divergence"})


# ---------------------------------------------------------------------------------------------
# 8. CLI contract
# ---------------------------------------------------------------------------------------------

def cli(*args, env=None):
    proc = subprocess.run([sys.executable, str(BIN), *args], capture_output=True, text=True, timeout=120,
                          cwd=str(REPO_ROOT), env={**os.environ, **(env or {})})
    return proc.returncode, proc.stdout, proc.stderr


class CliTests(TmpDirCase):
    def test_exit_codes(self):
        self.assertEqual(cli(str(FIXTURES / EVID), "--now", "2026-08-01T00:00:00Z")[0], 0)
        self.assertEqual(cli(str(FIXTURES / "invalid_wrong_sha.json"))[0], 1)
        self.assertEqual(cli()[0], 2)
        self.assertEqual(cli(str(FIXTURES / EVID), "--expect-candidate-sha", "nothex")[0], 2)
        self.assertEqual(cli(str(FIXTURES / EVID), "--bogus")[0], 2)

    def test_expected_candidate_sha_flag(self):
        rc, out, err = cli(str(FIXTURES / EVID), "--now", "2026-08-01T00:00:00Z", "--expect-candidate-sha", CAND)
        self.assertEqual(rc, 0, err)
        rc, out, err = cli(str(FIXTURES / EVID), "--now", "2026-08-01T00:00:00Z", "--expect-candidate-sha", OTHER)
        self.assertEqual(rc, 1)
        self.assertIn("expected_sha_mismatch", err)

    def test_json_envelope_carries_trust_and_applied_expectations(self):
        rc, out, _ = cli(str(FIXTURES / EVID), "--json", "--now", "2026-08-01T00:00:00Z",
                         "--expect-candidate-sha", CAND, "--expect-policy-version", POLICY)
        self.assertEqual(rc, 0)
        payload = json.loads(out)
        self.assertEqual(payload["schema_version"], "harness-json-v1")
        self.assertEqual(payload["status"], "pass")  # structure is valid; trust stays explicit below
        self.assertEqual(payload["trust"], "structure_only")
        self.assertFalse(payload["expectations_complete"])
        self.assertTrue(payload["expectations_applied"]["candidate_sha"])
        self.assertTrue(payload["expectations_applied"]["policy_version"])
        self.assertFalse(payload["expectations_applied"]["logs"])
        self.assertEqual(payload["warnings"], [])
        self.assertTrue(any(n.startswith("external_expectations_missing") for n in payload["notes"]))

    def test_json_reports_complete_expectations(self):
        logs = self.tmp / "logs"
        shutil.copytree(FIXTURES / "check-logs", logs)
        digest = hashlib.sha256((SCHEMAS / "check-catalog-v1.json").read_bytes()).hexdigest()
        rc, out, err = cli(str(FIXTURES / EVID), "--json", "--now", "2026-08-01T00:00:00Z",
                           "--expect-candidate-sha", CAND, "--expect-base-sha", BASE, "--expect-tree-sha", TREE,
                           "--expect-policy-version", POLICY, "--expect-catalog-sha256", digest,
                           "--task", str(FIXTURES / TASK), "--logs-dir", str(logs), "--require-expectations")
        self.assertEqual(rc, 0, err)
        payload = json.loads(out)
        self.assertEqual(payload["status"], "pass")
        self.assertTrue(payload["expectations_complete"])
        self.assertTrue(all(payload["expectations_applied"].values()))

    def test_require_expectations_is_a_usage_error_when_incomplete(self):
        rc, _, err = cli(str(FIXTURES / EVID), "--require-expectations", "--expect-candidate-sha", CAND)
        self.assertEqual(rc, 2)
        self.assertIn("--expect-policy-version", err)
        rc, _, err = cli(str(FIXTURES / REV), "--require-expectations", "--expect-candidate-sha", CAND, "--expect-policy-version", POLICY,
                         "--now", "2026-08-01T00:00:00Z")
        self.assertEqual(rc, 0, err)

    def test_invalid_task_flag_is_reported(self):
        bad = self.write("bad-task.json", {"schema_version": "task-v2"})
        rc, _, err = cli(str(FIXTURES / EVID), "--task", str(bad), "--now", "2026-08-01T00:00:00Z")
        self.assertEqual(rc, 1)
        self.assertIn("task_invalid", err)

    def test_no_jsonschema_flag_and_blocker_give_same_json(self):
        a = cli(str(FIXTURES / "invalid_wrong_sha.json"), "--json")
        b = cli(str(FIXTURES / "invalid_wrong_sha.json"), "--json", "--no-jsonschema")
        pa, pb = json.loads(a[1]), json.loads(b[1])
        self.assertEqual(a[0], b[0])
        key = lambda p: sorted((f["code"], f["field"]) for f in p["failures"])
        self.assertEqual(key(pa), key(pb))

    def test_lone_surrogate_in_a_payload_cannot_crash_the_diagnostics(self):
        p = self.write("surrogate.json", '{"k\\ud800": 1, "k\\ud800": 2}')
        rc, out, err = cli(str(p), "--now", "2026-08-01T00:00:00Z")
        self.assertEqual(rc, 1, err)
        self.assertIn("duplicate_key", err)
        self.assertNotIn("Traceback", err)

    def test_self_test_reports_a_positive_count(self):
        rc, out, err = cli("--self-test")
        self.assertEqual(rc, 0, err)
        line = [l for l in out.splitlines() if l.startswith("SELF_TEST_RESULT:")]
        self.assertEqual(len(line), 1, out)
        self.assertRegex(line[0], r"passed=[1-9][0-9]* failed=0")

    def test_historical_v1_is_flagged_in_cli_output(self):
        rc, out, err = cli(str(FIXTURES / "valid_task.json"))
        self.assertEqual(rc, 0, err)
        self.assertIn("NOTE[HISTORICAL_V1]", out + err)


if __name__ == "__main__":
    unittest.main(verbosity=2)

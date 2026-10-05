#!/usr/bin/env python3
"""Tests for the SDK/registry alignment checker (synthetic inputs, no network)."""

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

SCRIPT = Path(__file__).resolve().with_name("check-sdk-contract-alignment.py")
spec = importlib.util.spec_from_file_location("check_sdk_contract_alignment", SCRIPT)
checker = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = checker  # dataclasses resolves annotations through sys.modules
assert spec.loader is not None
spec.loader.exec_module(checker)

REGISTRY = {
    "memory_get": checker.ToolSpec(frozenset({"id"}), frozenset({"id"})),
    "memory_create": checker.ToolSpec(
        frozenset({"content", "memory_type", "tags"}), frozenset({"content"})
    ),
}


def call(tool: str, *keys: str, sdk: str = "python", site: str = "test") -> "checker.Call":
    return checker.Call(sdk, tool, frozenset(keys), site)


class ComputeDriftTests(unittest.TestCase):
    def test_aligned_calls_have_no_drift(self) -> None:
        calls = [call("memory_get", "id"), call("memory_create", "content", "tags")]
        self.assertEqual(checker.compute_drift(REGISTRY, calls), set())

    def test_unknown_tool_is_reported_per_sdk(self) -> None:
        drift = checker.compute_drift(
            REGISTRY, [call("memory_nope", "x"), call("memory_nope", sdk="typescript")]
        )
        self.assertEqual(
            drift,
            {"python:unknown_tool:memory_nope@test", "typescript:unknown_tool:memory_nope@test"},
        )

    def test_camel_case_key_is_an_unknown_argument(self) -> None:
        drift = checker.compute_drift(REGISTRY, [call("memory_create", "content", "memoryType")])
        self.assertEqual(drift, {"python:unknown_argument:memory_create.memoryType@test"})

    def test_required_argument_never_sent_is_reported(self) -> None:
        drift = checker.compute_drift(REGISTRY, [call("memory_get", "memory_id")])
        self.assertEqual(
            drift,
            {
                "python:unknown_argument:memory_get.memory_id@test",
                "python:required_never_sent:memory_get.id@test",
            },
        )

    def test_keys_are_unioned_within_one_call_site_only(self) -> None:
        # One Python method is swept twice (required-only, then all parameters).
        same_site = [call("memory_create", "content"), call("memory_create", "tags")]
        self.assertEqual(checker.compute_drift(REGISTRY, same_site), set())

    def test_new_call_site_of_a_baselined_unknown_tool_is_new_drift(self) -> None:
        baseline = checker.compute_drift(REGISTRY, [call("memory_nope", site="old_method")])
        grown = checker.compute_drift(
            REGISTRY,
            [call("memory_nope", site="old_method"), call("memory_nope", site="new_method")],
        )
        self.assertEqual(checker.compare(grown, baseline), (["python:unknown_tool:memory_nope@new_method"], []))

    def test_new_call_site_omitting_a_required_key_is_new_drift(self) -> None:
        good = [call("memory_create", "content", site="create")]
        baseline = checker.compute_drift(REGISTRY, good)
        grown = checker.compute_drift(REGISTRY, [*good, call("memory_create", "tags", site="create_tagged")])
        self.assertEqual(
            checker.compare(grown, baseline),
            (["python:required_never_sent:memory_create.content@create_tagged"], []),
        )

    def test_unresolved_and_unexercised_entries_are_drift(self) -> None:
        drift = checker.compute_drift(
            REGISTRY, [], unresolved=["a.ts:memory_get"], unexercised=["thing: TypeError"]
        )
        self.assertEqual(
            drift,
            {
                "typescript:unresolved_call:a.ts:memory_get",
                "python:unexercised_method:thing: TypeError",
            },
        )


class RatchetTests(unittest.TestCase):
    def test_new_drift_and_stale_baseline_both_fail(self) -> None:
        new, stale = checker.compare({"a", "b"}, {"b", "c"})
        self.assertEqual((new, stale), (["a"], ["c"]))

    def test_identical_sets_pass(self) -> None:
        self.assertEqual(checker.compare({"a"}, {"a"}), ([], []))

    def test_baseline_round_trip_and_version_gate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "baseline.json"
            checker.write_baseline({"z", "a"}, path)
            self.assertEqual(checker.load_baseline(path), {"a", "z"})
            document = json.loads(path.read_text())
            self.assertEqual(document["known_drift"], ["a", "z"])
            document["version"] = 99
            path.write_text(json.dumps(document))
            with self.assertRaises(SystemExit):
                checker.load_baseline(path)

    def test_main_exit_status_follows_the_ratchet(self) -> None:
        drift = {"python:unknown_tool:new"}
        with mock.patch.object(checker, "collect_drift", lambda only=None: drift), \
                contextlib.redirect_stdout(io.StringIO()) as output:
            with mock.patch.object(checker, "load_baseline", lambda: {"python:unknown_tool:old"}):
                self.assertEqual(checker.main([]), 1)
            self.assertIn("NEW DRIFT", output.getvalue())
            self.assertIn("STALE BASELINE", output.getvalue())
            with mock.patch.object(checker, "load_baseline", lambda: {"python:unknown_tool:new"}):
                self.assertEqual(checker.main([]), 0)

    def test_only_filters_the_baseline_to_one_sdk(self) -> None:
        baseline = {"python:unknown_tool:a", "typescript:unknown_tool:b"}
        self.assertEqual(
            checker.compare({"typescript:unknown_tool:b"}, baseline, only="typescript"), ([], [])
        )
        self.assertEqual(
            checker.compare(set(), baseline, only="typescript"),
            ([], ["typescript:unknown_tool:b"]),
        )


class UpdateBaselineTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.path = Path(self.directory.name) / "baseline.json"
        checker.write_baseline({"python:unknown_tool:a@x", "python:unknown_tool:b@y"}, self.path)

    def update(self, drift: set[str], **kwargs: object) -> tuple[int, str, str]:
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            status = checker.update_baseline(drift, self.path, **kwargs)
        return status, out.getvalue(), err.getvalue()

    def test_shrink_is_accepted_and_logged(self) -> None:
        status, out, _ = self.update({"python:unknown_tool:a@x"})
        self.assertEqual(status, 0)
        self.assertIn("REMOVE python:unknown_tool:b@y", out)
        self.assertEqual(checker.load_baseline(self.path), {"python:unknown_tool:a@x"})
        entry = json.loads(self.path.read_text())["changelog"][-1]
        self.assertEqual((entry["added"], entry["removed"]), (0, 1))

    def test_growth_is_refused_and_nothing_is_written(self) -> None:
        before = self.path.read_text()
        status, out, err = self.update(
            {"python:unknown_tool:a@x", "python:unknown_tool:b@y", "python:unknown_tool:c@z"}
        )
        self.assertEqual(status, 1)
        self.assertIn("ADD    python:unknown_tool:c@z", out)
        self.assertIn("refusing to bless", err)
        self.assertEqual(self.path.read_text(), before)

    def test_growth_needs_a_reason_even_with_the_flag(self) -> None:
        before = self.path.read_text()
        status, _, err = self.update(
            {"python:unknown_tool:c@z"}, allow_new_drift=True, reason="  "
        )
        self.assertEqual(status, 1)
        self.assertIn("--reason", err)
        self.assertEqual(self.path.read_text(), before)

    def test_growth_with_flag_and_reason_is_accepted_and_recorded(self) -> None:
        status, out, _ = self.update(
            {"python:unknown_tool:a@x", "python:unknown_tool:c@z"},
            allow_new_drift=True,
            reason="registry removed tool c upstream",
        )
        self.assertEqual(status, 0)
        self.assertIn("ADD    python:unknown_tool:c@z", out)
        self.assertEqual(
            checker.load_baseline(self.path), {"python:unknown_tool:a@x", "python:unknown_tool:c@z"}
        )
        entry = json.loads(self.path.read_text())["changelog"][-1]
        self.assertEqual(entry["reason"], "registry removed tool c upstream")
        self.assertEqual((entry["added"], entry["removed"]), (1, 1))

    def test_main_update_baseline_path_is_shrink_only(self) -> None:
        shrunk = {"python:unknown_tool:a@x"}
        grown = {"python:unknown_tool:a@x", "python:unknown_tool:new@n"}
        with mock.patch.object(checker, "BASELINE_PATH", self.path), mock.patch.object(
            checker, "collect_drift", lambda only=None: shrunk
        ), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(checker.main(["--update-baseline"]), 0)
        with mock.patch.object(checker, "BASELINE_PATH", self.path), mock.patch.object(
            checker, "collect_drift", lambda only=None: grown
        ), contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(checker.main(["--update-baseline"]), 1)
            self.assertEqual(
                checker.main(["--update-baseline", "--allow-new-drift", "--reason", "test"]), 0
            )
        self.assertEqual(checker.load_baseline(self.path), grown)

    def test_flags_are_rejected_outside_update_baseline(self) -> None:
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
            checker.main(["--allow-new-drift"])


class TypeScriptExtractionTests(unittest.TestCase):
    def extract(self, source: str):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "resource.ts").write_text(source)
            with mock.patch.object(checker, "TYPESCRIPT_SRC", root), mock.patch.object(
                checker, "ROOT", root
            ):
                return checker.typescript_calls()

    def test_inline_literal_with_shorthand_and_snake_case_keys(self) -> None:
        calls, unresolved = self.extract(
            'class R {\n  async f(a: number) {\n    return this.caller.mcpCall("memory_get", { id: a, tag_list });\n  }\n}\n'
        )
        self.assertEqual(unresolved, [])
        self.assertEqual(calls[0].tool, "memory_get")
        self.assertEqual(calls[0].keys, frozenset({"id", "tag_list"}))
        self.assertEqual(calls[0].source, "resource.ts:f")

    def test_params_object_literal_and_assignments(self) -> None:
        source = """
class R {
  async f(options?: O) {
    const params: Record<string, unknown> = { content, memory_type: options?.t ?? "note" };
    if (options?.tags) params.tags = options.tags;
    if (options?.m) params["metadata"] = options.m;
    return this.caller.mcpCall("memory_create", params);
  }
  async g() { return this.caller.mcpCall("memory_get", { id: 1 }); }
}
"""
        calls, unresolved = self.extract(source)
        self.assertEqual(unresolved, [])
        by_tool = {c.tool: c.keys for c in calls}
        self.assertEqual({c.tool: c.source for c in calls}, {"memory_create": "resource.ts:f", "memory_get": "resource.ts:g"})
        self.assertEqual(
            by_tool["memory_create"], frozenset({"content", "memory_type", "tags", "metadata"})
        )
        self.assertEqual(by_tool["memory_get"], frozenset({"id"}))

    def test_conditional_spread_literals_are_read(self) -> None:
        source = (
            'x.mcpCall("t", { a, ...(o?.b && { b_key: o.b }), ...(o?.c !== undefined && { c_key: 1 }) });'
        )
        calls, unresolved = self.extract(source)
        self.assertEqual(unresolved, [])
        self.assertEqual(calls[0].keys, frozenset({"a", "b_key", "c_key"}))

    def test_opaque_spread_is_reported_unresolved(self) -> None:
        calls, unresolved = self.extract('x.mcpCall("t", { a, ...options });')
        self.assertEqual(unresolved, ["resource.ts:t"])

    def test_conditional_argument_expression(self) -> None:
        calls, unresolved = self.extract('x.mcpCall("t", tool ? { tool } : {});')
        self.assertEqual(unresolved, [])
        self.assertEqual(calls[0].keys, frozenset({"tool"}))

    def test_strings_with_braces_do_not_confuse_matching(self) -> None:
        calls, unresolved = self.extract('x.mcpCall("t", { a: "}{", b: `x${1}` });')
        self.assertEqual(unresolved, [])
        self.assertEqual(calls[0].keys, frozenset({"a", "b"}))


class RealRepositoryTests(unittest.TestCase):
    def test_registry_loads_and_python_sweep_runs(self) -> None:
        registry = checker.load_registry()
        self.assertIn("memory_create", registry)
        self.assertIn("content", registry["memory_create"].required)
        calls, _ = checker.python_calls()
        self.assertTrue(any(c.tool == "memory_create" for c in calls))

    def test_checked_in_baseline_matches_current_drift(self) -> None:
        new, stale = checker.compare(checker.collect_drift(), checker.load_baseline())
        self.assertEqual((new, stale), ([], []))

    def test_typescript_only_run_matches_its_share_of_the_baseline(self) -> None:
        drift = checker.collect_drift("typescript")
        self.assertTrue(all(entry.startswith("typescript:") for entry in drift))
        self.assertEqual(
            checker.compare(drift, checker.load_baseline(), "typescript"), ([], [])
        )


if __name__ == "__main__":
    unittest.main()

#!/usr/bin/env python3
"""Fixture tests for scripts/rust_risk_inventory.py (Q2).

Run: python3 -m unittest scripts/test_rust_risk_inventory.py
Each test builds a tiny synthetic crate in a temp dir (no repository access).
"""

from __future__ import annotations

import importlib.util
import io
import json
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path
from typing import Dict, List, Tuple

_SPEC = importlib.util.spec_from_file_location("rust_risk_inventory", Path(__file__).parent / "rust_risk_inventory.py")
rri = importlib.util.module_from_spec(_SPEC)
assert _SPEC.loader is not None
sys.modules["rust_risk_inventory"] = rri  # dataclasses resolve annotations via sys.modules
_SPEC.loader.exec_module(rri)

CARGO = '[package]\nname = "fx"\nversion = "0.0.0"\nedition = "2021"\n'


def build(files: Dict[str, str]):
    """Write a fixture crate to a temp dir and return its Inventory."""
    tmp = tempfile.TemporaryDirectory()
    root = Path(tmp.name)
    (root / "Cargo.toml").write_text(CARGO)
    for rel, text in files.items():
        p = root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)
    inv = rri.build_inventory(root, "walk")
    tmp.cleanup()
    return inv


def occ(inv, kind: str) -> List[Tuple[str, int, str]]:
    return [(o["path"], o["line"], o["scope"]) for o in inv.occurrences if o["kind"] == kind]


def scopes(inv, kind: str, path: str = "src/lib.rs") -> List[Tuple[int, str]]:
    return [(line, scope) for p, line, scope in occ(inv, kind) if p == path]


class LexerTests(unittest.TestCase):
    def test_raw_strings_hide_code_like_text(self):
        src = (
            "pub fn f() {\n"
            '    let a = r#"x.unwrap() // not a comment"#;\n'
            '    let b = br##"y".expect("z")"##;\n'
            '    let c = r"multi\n'
            "line .unwrap()\n"
            '";\n'
            "    real.unwrap();\n"
            "}\n"
        )
        inv = build({"src/lib.rs": src})
        self.assertEqual(scopes(inv, "unwrap"), [(7, "prod-lib")])
        self.assertEqual(occ(inv, "expect"), [])

    def test_comments_nested_and_doc(self):
        src = (
            "/// doc x.unwrap()\n"
            "//! inner doc y.unwrap()\n"
            "// line z.unwrap()\n"
            "/* a /* nested b.unwrap() */ still comment c.unwrap() */\n"
            "pub fn f() {\n"
            "    /** doc block q.unwrap() */\n"
            "    real.unwrap(); // trailing .unwrap()\n"
            "}\n"
        )
        inv = build({"src/lib.rs": src})
        self.assertEqual(scopes(inv, "unwrap"), [(7, "prod-lib")])

    def test_strings_escapes_chars_and_lifetimes(self):
        src = (
            "pub fn f<'a>(s: &'a str) -> &'static str {\n"
            '    let _a = "http://x \\" .unwrap() \\\\";\n'
            "    let _b = '\"';\n"
            "    let _c = '\\'';\n"
            "    let _d = b'\"';\n"
            '    let _e = b"bytes .expect(1)";\n'
            "    'outer: loop { break 'outer; }\n"
            "    s.parse::<u32>().unwrap();\n"
            '    ""\n'
            "}\n"
        )
        inv = build({"src/lib.rs": src})
        self.assertEqual(scopes(inv, "unwrap"), [(8, "prod-lib")])
        self.assertEqual(occ(inv, "expect"), [])
        self.assertEqual(inv.stats["lex_errors"], 0)

    def test_unwrap_variants_not_confused(self):
        src = (
            "pub fn f(o: Option<u8>, r: Result<u8, ()>) {\n"
            "    o.unwrap_or(1);\n"
            "    o.unwrap_or_else(|| 1);\n"
            "    o.unwrap_or_default();\n"
            "    r.clone().unwrap_err();\n"
            '    r.clone().expect_err("e");\n'
            "    o.unwrap();\n"
            "}\n"
        )
        inv = build({"src/lib.rs": src})
        self.assertEqual([x[1] for x in occ(inv, "unwrap")], [7])
        self.assertEqual([x[1] for x in occ(inv, "unwrap_err")], [5])
        self.assertEqual([x[1] for x in occ(inv, "expect_err")], [6])

    def test_receiver_method_detail(self):
        src = (
            "use std::sync::Mutex;\n"
            "pub fn f(m: &Mutex<u8>, s: &str) {\n"
            "    m.lock().unwrap();\n"
            "    s.parse::<u8>().unwrap();\n"
            "    s.strip_prefix(\"a\").unwrap();\n"
            "}\n"
        )
        inv = build({"src/lib.rs": src})
        details = [o["detail"] for o in inv.occurrences if o["kind"] == "unwrap"]
        self.assertEqual(details, ["lock", "parse", "strip_prefix"])

    def test_custom_unwrap_definition_is_reported_not_hidden(self):
        src = "pub struct P;\nimpl P { pub fn expect(&self) {} }\npub fn f(p: P) { p.expect(); }\n"
        inv = build({"src/lib.rs": src})
        self.assertEqual(len(inv.stats["custom_unwrap_expect_defs"]), 1)


class CfgScopeTests(unittest.TestCase):
    def test_nested_cfg_test_modules(self):
        src = (
            "pub fn prod() { a.unwrap(); }\n"
            "#[cfg(test)]\n"
            "mod outer {\n"
            "    mod inner {\n"
            "        fn t() { b.unwrap(); }\n"
            "    }\n"
            "    fn u() { c.unwrap(); }\n"
            "}\n"
            "pub fn prod2() { d.unwrap(); }\n"
        )
        inv = build({"src/lib.rs": src})
        self.assertEqual(
            scopes(inv, "unwrap"),
            [(1, "prod-lib"), (5, "test-unit"), (7, "test-unit"), (9, "prod-lib")],
        )

    def test_cfg_test_items_inside_production_file(self):
        src = (
            "pub fn prod() { a.unwrap(); }\n"
            "#[cfg(test)]\n"
            "fn helper() { b.unwrap(); }\n"
            "#[cfg(test)]\n"
            "const X: u8 = foo().unwrap();\n"
            "#[cfg(test)]\n"
            "use std::fmt;\n"
            "#[cfg(test)]\n"
            "impl<T, U> Foo<T, U> where T: Clone {\n"
            "    fn a() { c.unwrap(); }\n"
            "}\n"
            "#[cfg(test)]\n"
            "fn generic<'a, T: Into<String>>(x: &'a str) -> Result<(), ()> where T: Clone { d.unwrap(); Ok(()) }\n"
            "pub fn after() { e.unwrap(); }\n"
        )
        inv = build({"src/lib.rs": src})
        self.assertEqual(
            scopes(inv, "unwrap"),
            [(1, "prod-lib"), (3, "test-unit"), (5, "test-unit"), (10, "test-unit"), (13, "test-unit"), (14, "prod-lib")],
        )

    def test_test_attributes_without_cfg(self):
        src = (
            "pub fn prod() { a.unwrap(); }\n"
            "#[test]\n"
            "fn t1() { b.unwrap(); }\n"
            "#[tokio::test]\n"
            "async fn t2() { c.unwrap(); }\n"
            "pub fn prod2() { d.unwrap(); }\n"
        )
        inv = build({"src/lib.rs": src})
        self.assertEqual(
            scopes(inv, "unwrap"),
            [(1, "prod-lib"), (3, "test-unit"), (5, "test-unit"), (6, "prod-lib")],
        )

    def test_three_valued_cfg_predicates(self):
        src = (
            '#[cfg(not(test))]\nfn a() { x.unwrap(); }\n'
            '#[cfg(any(test, feature = "f"))]\nfn b() { x.unwrap(); }\n'
            '#[cfg(all(test, feature = "f"))]\nfn c() { x.unwrap(); }\n'
            '#[cfg(feature = "g")]\nfn d() { x.unwrap(); }\n'
            "#[cfg(false)]\nfn e() { x.unwrap(); }\n"
            "#[cfg_attr(test, derive(Debug))]\nstruct S;\nfn f() { x.unwrap(); }\n"
        )
        inv = build({"src/lib.rs": src})
        got = {o["line"]: (o["scope"], o["conditional"], tuple(o["features"])) for o in inv.occurrences if o["kind"] == "unwrap"}
        self.assertEqual(got[2], ("prod-lib", False, ()))
        self.assertEqual(got[4], ("prod-lib", True, ("f",)))
        self.assertEqual(got[6], ("test-unit", False, ("f",)))  # never in a prod build
        self.assertEqual(got[8], ("prod-lib", True, ("g",)))
        self.assertEqual(got[10][0], "disabled")
        self.assertEqual(got[13][0], "prod-lib")

    def test_unparsed_cfg_stays_unknown(self):
        src = "#[cfg(weird 123)]\nfn a() { x.unwrap(); }\n"
        inv = build({"src/lib.rs": src})
        self.assertEqual(scopes(inv, "unwrap"), [(2, "unknown")])
        self.assertEqual(inv.stats["unparsed_cfg_attrs"], 1)

    def test_inner_cfg_test_attribute_on_file_and_inline_mod(self):
        files = {
            "src/lib.rs": "pub mod a;\npub mod m { #![cfg(test)]\n fn t() { y.unwrap(); } }\npub fn p() { z.unwrap(); }\n",
            "src/a.rs": "#![cfg(test)]\nfn t() { x.unwrap(); }\n",
        }
        inv = build(files)
        self.assertEqual(scopes(inv, "unwrap", "src/a.rs"), [(2, "test-unit")])
        self.assertEqual(scopes(inv, "unwrap"), [(3, "test-unit"), (4, "prod-lib")])

    def test_cfg_on_match_arm_does_not_swallow_next_arm(self):
        src = (
            "pub fn f(k: u8) {\n"
            "    match k {\n"
            "        #[cfg(test)]\n"
            "        0 => a.unwrap(),\n"
            "        _ => b.unwrap(),\n"
            "    }\n"
            "}\n"
        )
        inv = build({"src/lib.rs": src})
        self.assertEqual(scopes(inv, "unwrap"), [(4, "test-unit"), (5, "prod-lib")])
        self.assertEqual(inv.stats["nonitem_cfg_units_approximated"], 1)
        self.assertEqual(inv.stats["nonitem_test_gated_units_approximated"], 1)


class MacroTests(unittest.TestCase):
    def test_macro_rules_body_is_macro_def_and_invocations_are_code(self):
        src = (
            "macro_rules! m {\n"
            "    ($e:expr) => { $e.unwrap() };\n"
            "}\n"
            "pub fn f() {\n"
            "    let _v = vec![a.unwrap()];\n"
            '    println!("{}", b.unwrap());\n'
            "    assert_eq!(1, 1);\n"
            '    unreachable!("x");\n'
            "}\n"
        )
        inv = build({"src/lib.rs": src})
        self.assertEqual(scopes(inv, "unwrap"), [(2, "macro-def"), (5, "prod-lib"), (6, "prod-lib")])
        self.assertEqual(scopes(inv, "print"), [(6, "prod-lib")])
        self.assertEqual(scopes(inv, "assert_macro"), [(7, "prod-lib")])
        self.assertEqual(scopes(inv, "panic_macro"), [(8, "prod-lib")])

    def test_macro_names_inside_strings_and_idents_ignored(self):
        src = 'pub fn f() { let _s = "println!(1) panic!()"; let println = 1; let _ = println; }\n'
        inv = build({"src/lib.rs": src})
        self.assertEqual(occ(inv, "print"), [])
        self.assertEqual(occ(inv, "panic_macro"), [])


class ModuleTreeTests(unittest.TestCase):
    def test_out_of_line_test_module_and_nested_dirs(self):
        files = {
            "src/lib.rs": "pub mod a;\n#[cfg(test)]\nmod tests;\npub fn p() { x.unwrap(); }\n",
            "src/a/mod.rs": "pub mod b;\npub fn a() { y.unwrap(); }\n",
            "src/a/b.rs": "pub fn b() { z.unwrap(); }\n",
            "src/tests.rs": "fn t() { w.unwrap(); }\n",
        }
        inv = build(files)
        self.assertEqual(scopes(inv, "unwrap", "src/a/mod.rs"), [(2, "prod-lib")])
        self.assertEqual(scopes(inv, "unwrap", "src/a/b.rs"), [(1, "prod-lib")])
        self.assertEqual(scopes(inv, "unwrap", "src/tests.rs"), [(1, "test-unit")])
        self.assertEqual(inv.stats["orphan_files"], 0)

    def test_cfg_test_on_inline_parent_propagates_to_child_file(self):
        files = {
            "src/lib.rs": "#[cfg(test)]\nmod outer {\n    mod inner;\n}\n",
            "src/outer/inner.rs": "fn t() { x.unwrap(); }\n",
        }
        inv = build(files)
        self.assertEqual(scopes(inv, "unwrap", "src/outer/inner.rs"), [(1, "test-unit")])

    def test_path_attribute_and_unresolved_mod(self):
        files = {
            "src/lib.rs": '#[path = "weird/place.rs"]\nmod m;\nmod missing;\n',
            "src/weird/place.rs": "pub fn f() { x.unwrap(); }\n",
        }
        inv = build(files)
        self.assertEqual(scopes(inv, "unwrap", "src/weird/place.rs"), [(1, "prod-lib")])
        self.assertEqual(len(inv.unresolved), 1)
        self.assertIn("missing", inv.unresolved[0])

    def test_include_inherits_gate_and_non_literal_is_unresolved(self):
        files = {
            "src/lib.rs": '#[cfg(test)]\ninclude!("gen.rs");\ninclude!(concat!(env!("OUT_DIR"), "/x.rs"));\n',
            "src/gen.rs": "fn g() { x.unwrap(); }\n",
        }
        inv = build(files)
        self.assertEqual(scopes(inv, "unwrap", "src/gen.rs"), [(1, "test-unit")])
        self.assertEqual(len(inv.unresolved), 1)

    def test_orphan_file_is_unknown(self):
        files = {"src/lib.rs": "", "src/orphan.rs": "pub fn f() { x.unwrap(); }\n"}
        inv = build(files)
        self.assertEqual(scopes(inv, "unwrap", "src/orphan.rs"), [(1, "unknown")])
        self.assertEqual(inv.orphans, ["src/orphan.rs"])


class TargetKindTests(unittest.TestCase):
    def test_tests_benches_examples_build_bin_and_lib(self):
        files = {
            "src/lib.rs": 'pub fn l() { println!("lib"); x.unwrap(); }\n',
            "src/main.rs": 'fn main() { println!("main"); }\n',
            "src/bin/tool.rs": 'fn main() { eprintln!("tool"); }\n',
            "src/bin/multi/main.rs": "mod helper;\nfn main() {}\n",
            "src/bin/multi/helper.rs": 'pub fn h() { println!("h"); }\n',
            "tests/it.rs": "mod common;\n#[test]\nfn t() { x.unwrap(); }\n",
            "tests/common/mod.rs": "pub fn c() { y.unwrap(); }\n",
            "benches/b.rs": "fn main() { z.unwrap(); }\n",
            "examples/e.rs": 'fn main() { println!("e"); }\n',
            "build.rs": 'fn main() { println!("cargo:rerun-if-changed=x"); }\n',
        }
        inv = build(files)
        by_path = {(o["path"], o["kind"]): (o["scope"], o["target"]) for o in inv.occurrences}
        self.assertEqual(by_path[("src/lib.rs", "print")], ("prod-lib", "fx:lib"))
        self.assertEqual(by_path[("src/main.rs", "print")], ("prod-bin", "fx:bin:fx"))
        self.assertEqual(by_path[("src/bin/tool.rs", "print")], ("prod-bin", "fx:bin:tool"))
        self.assertEqual(by_path[("src/bin/multi/helper.rs", "print")], ("prod-bin", "fx:bin:multi"))
        self.assertEqual(by_path[("tests/it.rs", "unwrap")][0], "test-integration")
        self.assertEqual(by_path[("tests/common/mod.rs", "unwrap")][0], "test-integration")
        self.assertEqual(by_path[("benches/b.rs", "unwrap")][0], "bench")
        self.assertEqual(by_path[("examples/e.rs", "print")][0], "example")
        self.assertEqual(by_path[("build.rs", "print")][0], "build-script")

    def test_all_reachability_stays_unknown(self):
        inv = build({"src/lib.rs": "pub fn f() { x.unwrap(); }\n"})
        self.assertTrue(all(o["reachability"] == "unknown" for o in inv.occurrences))


class UnsafeAndSwallowTests(unittest.TestCase):
    def test_unsafe_kinds_and_safety_comments(self):
        src = (
            "// SAFETY: fd is valid for the call\n"
            "pub fn a() { let _r = unsafe { libc::getpid() }; }\n"
            "pub fn b() { let _r = unsafe { libc::getppid() }; }\n"
            "pub unsafe fn c() {}\n"
            "unsafe impl Send for S {}\n"
            "#[unsafe(no_mangle)]\n"
            "pub fn d() {}\n"
            'pub fn e() { let _s = "unsafe { }"; let is_unsafe_raw = 1; let _ = is_unsafe_raw; } // unsafe\n'
        )
        inv = build({"src/lib.rs": src})
        got = [(o["line"], o["unsafe_kind"], o["safety"]) for o in inv.occurrences if o["kind"] == "unsafe"]
        self.assertEqual(got, [(2, "block", "yes"), (3, "block", "no"), (4, "fn", "no"), (5, "impl", "no"), (6, "attr", "no")])

    def test_safety_comment_placement_rules(self):
        src = (
            "pub fn a() {\n"
            "    // SAFETY: registered once, fn-pointer shape matches\n"
            "    ONCE.call_once(|| unsafe {\n"
            "        reg();\n"
            "    });\n"
            "    unsafe {\n"
            "        // SAFETY: inside the block\n"
            "        reg();\n"
            "    }\n"
            "    // SAFETY: covers only the next statement\n"
            "    let x = 1;\n"
            "    unsafe { reg(); }\n"
            "}\n"
            "/// Does a thing.\n"
            "///\n"
            "/// # Safety\n"
            "/// Caller must hold the lock.\n"
            "pub unsafe fn c() {}\n"
        )
        inv = build({"src/lib.rs": src})
        got = [(o["line"], o["unsafe_kind"], o["safety"]) for o in inv.occurrences if o["kind"] == "unsafe"]
        self.assertEqual(got, [(3, "block", "yes"), (6, "block", "inside"), (12, "block", "no"), (18, "fn", "yes")])

    def test_unsafe_code_lint_is_recorded(self):
        inv = build({"src/lib.rs": "#![forbid(unsafe_code)]\n"})
        self.assertEqual(len(inv.stats["unsafe_code_lints"]), 1)

    def test_swallow_signals(self):
        src = (
            "pub fn f(tx: Tx, r: Result<u8, E>) -> Option<u8> {\n"
            "    let _ = tx.send(1);\n"
            "    let _guard = 1;\n"
            "    r.clone().ok();\n"
            "    let w = r.clone().ok();\n"
            "    let mut z = None; z = r.clone().ok();\n"
            "    let v = r.clone().ok()?;\n"
            "    match r { Err(_) => {}, Ok(_) => {} }\n"
            "    match r { Err(_) => (), Ok(_) => () }\n"
            "    let _ = (tx, z, w);\n"
            "    let _ = std::fs::remove_file(\"x\");\n"
            "    let _ = conn.execute(\"x\", []);\n"
            "    let _n: Vec<u8> = rows.filter_map(|row| row.ok()).collect();\n"
            "    let _m: Vec<u8> = rows.filter_map(Result::ok).collect();\n"
            "    let _o: Vec<u8> = rows.filter_map(|row| row.ok().map(|x| x)).collect();\n"
            "    let _p = r.unwrap_or_else(|_| 0);\n"
            "    let _q = r.unwrap_or_else(|e| e.len());\n"
            "    Some(v)\n"
            "}\n"
        )
        inv = build({"src/lib.rs": src})
        self.assertEqual([x[1] for x in occ(inv, "let_underscore")], [2, 10, 11, 12])
        self.assertEqual([x[1] for x in occ(inv, "ok_discard")], [4])  # bound `.ok();` is not a discard
        self.assertEqual([x[1] for x in occ(inv, "ok_bound")], [5, 6])
        self.assertEqual([x[1] for x in occ(inv, "err_arm_ignored")], [8, 9])
        self.assertEqual([x[1] for x in occ(inv, "filter_map_ok")], [13, 14])
        self.assertEqual([x[1] for x in occ(inv, "err_default_closure")], [16])
        details = [(o["detail"], o["rhs_class"]) for o in inv.occurrences if o["kind"] == "let_underscore"]
        self.assertEqual(
            details,
            [("tx.send(1)", "channel-send"), ("(tx , z , w)", "unused-binding"), ('std::fs::remove_file("x")', "teardown-io"), ('conn.execute("x" , [ ])', "db-or-state-write")],
        )

    def test_ok_discard_ignores_bound_multiline_chains_with_closure_bodies(self):
        src = (
            "pub fn f(conn: &C) {\n"
            "    let found = conn\n"
            "        .query_row(\"select 1\", [], |row| {\n"
            "            Ok(row.get(0)?)\n"
            "        })\n"
            "        .ok();\n"
            "    if found.is_some() {\n"
            "        touch();\n"
            "    }\n"
            "    conn.execute(\"x\", []).ok();\n"
            "}\n"
        )
        inv = build({"src/lib.rs": src})
        self.assertEqual([x[1] for x in occ(inv, "ok_discard")], [10])
        self.assertEqual([x[1] for x in occ(inv, "ok_bound")], [6])  # error -> None conversion is still reported

    def test_safety_comment_before_previous_block_does_not_leak(self):
        src = (
            "pub fn f() {\n"
            "    // SAFETY: only for the first block\n"
            "    unsafe { a(); }\n"
            "    if c {\n"
            "        b();\n"
            "    }\n"
            "    unsafe { d(); }\n"
            "}\n"
        )
        inv = build({"src/lib.rs": src})
        got = [(o["line"], o["safety"]) for o in inv.occurrences if o["kind"] == "unsafe"]
        self.assertEqual(got, [(3, "yes"), (7, "no")])

    def test_stdout_handle_and_process_exit(self):
        src = "use std::io;\npub fn f() { let _o = io::stdout(); std::process::exit(2); }\n"
        inv = build({"src/lib.rs": src})
        self.assertEqual([x[1] for x in occ(inv, "io_handle")], [2])
        self.assertEqual([x[1] for x in occ(inv, "process_exit")], [2])


class CliTests(unittest.TestCase):
    def test_json_output_is_deterministic_and_filterable(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "Cargo.toml").write_text(CARGO)
            (root / "src").mkdir()
            (root / "src/lib.rs").write_text("pub fn f() { a.unwrap(); b.expect(\"x\"); }\n#[test]\nfn t() { c.unwrap(); }\n")

            def run(*extra: str) -> str:
                buf = io.StringIO()
                with redirect_stdout(buf):
                    rc = rri.main(["--root", tmp, "--files", "walk", *extra])
                self.assertEqual(rc, 0)
                return buf.getvalue()

            first = run("--format", "json")
            self.assertEqual(first, run("--format", "json"))
            data = json.loads(first)
            self.assertEqual(len(data["occurrences"]), 3)
            listing = run("--format", "list", "--kind", "unwrap", "--scope", "prod-lib")
            self.assertEqual(listing.strip().splitlines(), ["src/lib.rs:1: unwrap [<ident>] scope=prod-lib fn=f"])
            self.assertIn("| unwrap |", run())


if __name__ == "__main__":
    unittest.main()

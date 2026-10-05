#!/usr/bin/env python3
"""Reproducible Rust risk inventory (Q2): unwrap/expect/panic/unsafe/print/swallow.

Standard library only. The tool lexes every tracked ``*.rs`` file (comments,
nested block comments, strings, raw strings, char literals and lifetimes are
handled), resolves the ``mod`` / ``include!`` tree from each Cargo target root,
evaluates ``#[cfg(...)]`` gates three-valued (``test`` known, features and
targets unknown) and classifies every occurrence into a scope.

Principles
- Unresolvable context stays ``unknown`` (never guessed): files not reachable
  from any target root, ``macro_rules!`` bodies, unparsed cfg predicates.
- Reachability from user input is NOT inferred; it is always ``unknown`` here and
  is established by hand for the backlog candidates in the inventory document.
- The tool never edits code and has no network or git side effects except an
  optional ``git ls-files`` / ``git rev-parse`` call.

Usage
  scripts/rust_risk_inventory.py                       # markdown summary
  scripts/rust_risk_inventory.py --format list --kind unwrap --scope prod-lib
  scripts/rust_risk_inventory.py --format json > inventory.json
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
import tomllib
from collections import Counter
from dataclasses import asdict, dataclass, field
from pathlib import Path, PurePosixPath
from typing import Dict, Iterable, List, Optional, Sequence, Tuple

TOOL_VERSION = 1

# --------------------------------------------------------------------------
# Lexer
# --------------------------------------------------------------------------

RAW_STR_RE = re.compile(r'(?:br|cr|r)(#*)"')


@dataclass(frozen=True)
class Tok:
    kind: str  # ident | punct | str | char | lifetime | num
    text: str
    line: int


@dataclass(frozen=True)
class Comment:
    line: int
    end_line: int
    text: str
    doc: bool


def _ident_start(c: str) -> bool:
    return c == "_" or c.isalpha()


def _ident_cont(c: str) -> bool:
    return c == "_" or c.isalnum()


def lex(src: str) -> Tuple[List[Tok], List[Comment], List[str]]:
    """Tokenize Rust source. Returns (tokens, comments, lex_errors)."""
    toks: List[Tok] = []
    comments: List[Comment] = []
    errors: List[str] = []
    n = len(src)
    i = 0
    line = 1

    def scan_string(q: int) -> int:
        j = q + 1
        while j < n:
            ch = src[j]
            if ch == "\\":
                j += 2
                continue
            if ch == '"':
                return j + 1
            j += 1
        errors.append(f"unterminated string at line {line}")
        return n

    def emit(kind: str, start: int, end: int) -> None:
        nonlocal line
        toks.append(Tok(kind, src[start:end], line))
        line += src.count("\n", start, end)

    while i < n:
        c = src[i]
        if c == "\n":
            line += 1
            i += 1
            continue
        if c.isspace():
            i += 1
            continue
        if c == "/" and i + 1 < n and src[i + 1] == "/":
            j = src.find("\n", i)
            j = n if j < 0 else j
            text = src[i:j]
            doc = (text.startswith("///") and not text.startswith("////")) or text.startswith("//!")
            comments.append(Comment(line, line, text, doc))
            i = j
            continue
        if c == "/" and i + 1 < n and src[i + 1] == "*":
            depth, j, end_line = 1, i + 2, line
            while j < n and depth:
                if src.startswith("/*", j):
                    depth += 1
                    j += 2
                elif src.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    if src[j] == "\n":
                        end_line += 1
                    j += 1
            if depth:
                errors.append(f"unterminated block comment at line {line}")
            text = src[i:j]
            doc = (text.startswith("/**") and not text.startswith("/***") and text != "/**/") or text.startswith("/*!")
            comments.append(Comment(line, end_line, text, doc))
            line = end_line
            i = j
            continue
        if c in "rbc":
            m = RAW_STR_RE.match(src, i)
            if m:
                closer = '"' + m.group(1)
                end = src.find(closer, m.end())
                if end < 0:
                    errors.append(f"unterminated raw string at line {line}")
                    j = n
                else:
                    j = end + len(closer)
                emit("str", i, j)
                i = j
                continue
            if c in "bc" and i + 1 < n and src[i + 1] == '"':
                j = scan_string(i + 1)
                emit("str", i, j)
                i = j
                continue
            if c == "b" and i + 1 < n and src[i + 1] == "'":
                j = i + 2
                if j < n and src[j] == "\\":
                    j += 2
                while j < n and src[j] != "'":
                    j += 1
                j += 1
                emit("char", i, min(j, n))
                i = min(j, n)
                continue
            if c == "r" and src.startswith("r#", i) and i + 2 < n and _ident_start(src[i + 2]):
                j = i + 2
                while j < n and _ident_cont(src[j]):
                    j += 1
                toks.append(Tok("ident", src[i + 2 : j], line))
                i = j
                continue
        if c == '"':
            j = scan_string(i)
            emit("str", i, j)
            i = j
            continue
        if c == "'":
            if i + 1 < n and src[i + 1] == "\\":
                j = i + 3
                while j < n and src[j] != "'":
                    j += 1
                j = min(j + 1, n)
                emit("char", i, j)
                i = j
            elif i + 2 < n and src[i + 2] == "'" and src[i + 1] != "'":
                emit("char", i, i + 3)
                i += 3
            elif i + 1 < n and _ident_start(src[i + 1]):
                j = i + 1
                while j < n and _ident_cont(src[j]):
                    j += 1
                toks.append(Tok("lifetime", src[i:j], line))
                i = j
            else:
                toks.append(Tok("punct", "'", line))
                i += 1
            continue
        if _ident_start(c):
            j = i + 1
            while j < n and _ident_cont(src[j]):
                j += 1
            toks.append(Tok("ident", src[i:j], line))
            i = j
            continue
        if c.isdigit():
            j = i + 1
            while j < n and (src[j].isalnum() or src[j] == "_"):
                j += 1
            toks.append(Tok("num", src[i:j], line))
            i = j
            continue
        toks.append(Tok("punct", c, line))
        i += 1
    return toks, comments, errors


OPEN = {"(": ")", "[": "]", "{": "}"}
CLOSE = {v: k for k, v in OPEN.items()}


def match_brackets(toks: Sequence[Tok]) -> Tuple[Dict[int, int], Dict[int, int], int]:
    """Return (open->close, close->open, mismatch_count)."""
    fwd: Dict[int, int] = {}
    rev: Dict[int, int] = {}
    stack: List[int] = []
    bad = 0
    for idx, t in enumerate(toks):
        if t.kind != "punct":
            continue
        if t.text in OPEN:
            stack.append(idx)
        elif t.text in CLOSE:
            if stack and OPEN[toks[stack[-1]].text] == t.text:
                o = stack.pop()
                fwd[o] = idx
                rev[idx] = o
            else:
                bad += 1
    return fwd, rev, bad + len(stack)


# --------------------------------------------------------------------------
# cfg predicates (three-valued)
# --------------------------------------------------------------------------


@dataclass(frozen=True)
class Gate:
    pred: str
    e_prod: Optional[bool]  # evaluation with cfg(test)=false; None = depends on features/targets
    e_test: Optional[bool]  # evaluation with cfg(test)=true
    features: Tuple[str, ...]
    parsed: bool


def _parse_cfg(toks: Sequence[Tok]):
    pos = 0

    def expr():
        nonlocal pos
        t = toks[pos]
        if t.kind != "ident":
            raise ValueError("expected ident")
        name = t.text
        pos += 1
        if pos < len(toks) and toks[pos].text == "(":
            pos += 1
            args = []
            while toks[pos].text != ")":
                args.append(expr())
                if toks[pos].text == ",":
                    pos += 1
            pos += 1
            return (name, args)
        if pos < len(toks) and toks[pos].text == "=":
            pos += 1
            val = toks[pos].text.strip('"')
            pos += 1
            return ("kv", name, val)
        return ("atom", name)

    node = expr()
    if pos != len(toks):
        raise ValueError("trailing tokens")
    return node


def _eval_cfg(node, test: bool) -> Optional[bool]:
    kind = node[0]
    if kind == "atom":
        if node[1] == "test":
            return test
        if node[1] == "true":
            return True
        if node[1] == "false":
            return False
        return None
    if kind == "kv":
        return None
    args = [_eval_cfg(a, test) for a in node[1]]
    if kind == "not" and len(args) == 1:
        return None if args[0] is None else (not args[0])
    if kind == "all":
        if any(a is False for a in args):
            return False
        return None if any(a is None for a in args) else True
    if kind == "any":
        if any(a is True for a in args):
            return True
        return None if any(a is None for a in args) else False
    return None


def _cfg_features(node) -> List[str]:
    if node[0] == "kv":
        return [node[2]] if node[1] == "feature" else []
    if node[0] == "atom":
        return []
    out: List[str] = []
    for a in node[1]:
        out.extend(_cfg_features(a))
    return out


def _tok_text(toks: Sequence[Tok]) -> str:
    return " ".join(t.text for t in toks)


def make_gate(pred_toks: Sequence[Tok]) -> Gate:
    text = _tok_text(pred_toks)
    try:
        node = _parse_cfg(pred_toks)
    except (ValueError, IndexError):
        return Gate(text, None, None, (), False)
    return Gate(text, _eval_cfg(node, False), _eval_cfg(node, True), tuple(sorted(set(_cfg_features(node)))), True)


# --------------------------------------------------------------------------
# Per-file analysis
# --------------------------------------------------------------------------

UNWRAP_METHODS = {
    "unwrap": "unwrap",
    "expect": "expect",
    "unwrap_err": "unwrap_err",
    "expect_err": "expect_err",
    "unwrap_unchecked": "unwrap_unchecked",
}
PANIC_MACROS = {"panic", "unreachable", "todo", "unimplemented"}
ASSERT_MACROS = {"assert", "assert_eq", "assert_ne"}
PRINT_MACROS = {
    "print": "stdout",
    "println": "stdout",
    "eprint": "stderr",
    "eprintln": "stderr",
    "dbg": "stderr",
}
BLOCK_ITEMS = {"fn", "mod", "impl", "trait", "struct", "enum", "union", "extern"}
MODIFIERS = {"pub", "async", "unsafe", "default", "safe"}


@dataclass
class Interval:
    start: int
    end: int
    kind: str  # gate | test_attr | macro_def | fn | mod
    data: object = None


@dataclass
class ModDecl:
    name: str
    tok: int
    line: int
    path_attr: Optional[str]
    include: bool = False
    include_path: Optional[str] = None


@dataclass
class RawOcc:
    kind: str
    line: int
    tok: int
    detail: str = ""
    extra: Dict[str, object] = field(default_factory=dict)


@dataclass
class FileAnalysis:
    toks: List[Tok]
    comments: List[Comment]
    lex_errors: List[str]
    bracket_errors: int
    intervals: List[Interval]
    mods: List[ModDecl]
    occs: List[RawOcc]
    nonitem_attr_units: int = 0
    nonitem_test_units: int = 0
    unparsed_cfgs: int = 0
    lint_unsafe_code: List[str] = field(default_factory=list)
    custom_defs: List[str] = field(default_factory=list)


def _attr_path(attr: Sequence[Tok]) -> str:
    parts: List[str] = []
    for t in attr:
        if t.kind == "ident" or (t.kind == "punct" and t.text == ":"):
            parts.append(t.text)
        else:
            break
    return "".join(parts)


def _skip_modifiers(toks: Sequence[Tok], match: Dict[int, int], k: int, limit: int) -> int:
    while k < limit:
        t = toks[k]
        if t.kind == "ident" and t.text in MODIFIERS:
            k += 1
            if t.text == "pub" and k < limit and toks[k].text == "(" and k in match:
                k = match[k] + 1
            continue
        if t.kind == "ident" and t.text == "const" and k + 1 < limit and toks[k + 1].text in {"fn", "unsafe", "async", "extern"}:
            k += 1
            continue
        break
    return k


def _unit_end(toks: Sequence[Tok], match: Dict[int, int], s: int, parent_close: int) -> Tuple[int, bool]:
    """Find the last token index of the unit an attribute group applies to.

    Returns (end_index, is_item). Statement/field/arm units are approximated.
    """
    n = len(toks)
    limit = min(parent_close, n)
    k = _skip_modifiers(toks, match, s, limit)
    head = toks[k] if k < limit else None
    block_item = False
    if head is not None:
        if head.kind == "ident" and head.text in BLOCK_ITEMS:
            block_item = True
        elif head.kind == "ident" and head.text == "macro_rules":
            block_item = True
        elif head.kind == "punct" and head.text == "{":
            block_item = True
    t = s
    while t < limit:
        tok = toks[t]
        if tok.kind == "punct" and tok.text in OPEN:
            close = match.get(t, limit - 1)
            if tok.text == "{" and (block_item or (t > s and toks[t - 1].text == "!" and t - 2 >= s)):
                return close, True
            t = close + 1
            continue
        if tok.kind == "punct" and tok.text == ";":
            return t, block_item or (head is not None and head.kind == "ident")
        if tok.kind == "punct" and tok.text == "," and not block_item:
            return t, False
        t += 1
    return max(limit - 1, s), False


def _is_unsafe_kind(toks: Sequence[Tok], i: int) -> str:
    nxt = toks[i + 1] if i + 1 < len(toks) else None
    if nxt is None:
        return "other"
    if nxt.text == "{":
        return "block"
    if nxt.text == "impl":
        return "impl"
    if nxt.text == "trait":
        return "trait"
    if nxt.text in {"fn", "async", "const", "extern"}:
        for k in range(i + 1, min(i + 6, len(toks))):
            if toks[k].text == "fn":
                return "fn"
            if toks[k].text == "{":
                return "extern_block"
        return "extern_block" if nxt.text == "extern" else "other"
    return "other"


CONTINUATION_AFTER_BLOCK = {".", "?", ")", "]", ",", "else", "as"}


def _stmt_start(toks: Sequence[Tok], rev: Dict[int, int], i: int) -> int:
    """Index of the first token of the statement/item containing token ``i``.

    Walks backwards: `;` and an unmatched `{` end the statement; a closing
    bracket is jumped over (closure bodies, call arguments) unless it is a `}`
    followed by something that starts a new statement.
    """
    k = i
    while k > 0:
        prev = k - 1
        t = toks[prev]
        if t.kind == "punct" and t.text in {";", "{"}:
            break
        if t.kind == "punct" and t.text in CLOSE:
            if t.text == "}" and toks[k].text not in CONTINUATION_AFTER_BLOCK and toks[k].kind != "punct":
                break
            k = rev.get(prev, prev)
            continue
        k = prev
    return k


def _stmt_start_line(toks: Sequence[Tok], rev: Dict[int, int], i: int) -> int:
    return toks[_stmt_start(toks, rev, i)].line


def _is_bound_statement(toks: Sequence[Tok], match: Dict[int, int], rev: Dict[int, int], i: int) -> bool:
    """True when the statement containing ``i`` binds/assigns its value
    (``let x = ...``, ``x = ...``) instead of discarding it."""
    k = _stmt_start(toks, rev, i)
    if toks[k].text in {"let", "return", "break"}:
        return True
    t = k
    while t < i:
        tok = toks[t]
        if tok.kind == "punct" and tok.text in OPEN and t in match:
            t = match[t] + 1
            continue
        if tok.kind == "punct" and tok.text == "=":
            prev = toks[t - 1].text if t > 0 else ""
            nxt = toks[t + 1].text if t + 1 < len(toks) else ""
            if prev not in {"=", "!", "<", ">", "+", "-", "*", "/", "%", "&", "|", "^"} and nxt not in {"=", ">"}:
                return True
        t += 1
    return False


def _safety_status(comments: Sequence[Comment], toks: Sequence[Tok], rev: Dict[int, int], i: int) -> str:
    """``yes``: a contiguous comment block directly above (or on the line of) the
    statement mentions "safety"; ``inside``: such a comment opens the block within
    two lines after the keyword; ``no`` otherwise."""
    line = toks[i].line
    start = _stmt_start_line(toks, rev, i)
    by_end = {c.end_line: c for c in comments}
    cur = start - 1
    while cur in by_end:
        c = by_end[cur]
        if "safety" in c.text.lower():
            return "yes"
        cur = c.line - 1
    for c in comments:
        if "safety" in c.text.lower() and (start <= c.line <= line or line < c.line <= line + 2):
            return "yes" if c.line <= line else "inside"
    return "no"


def _recv_method(toks: Sequence[Tok], rev: Dict[int, int], dot: int) -> str:
    """Method name feeding ``.unwrap()``: ``x.lock().unwrap()`` -> ``lock``."""
    p = dot - 1
    if p >= 0 and toks[p].text == ")" and p in rev:
        k = rev[p] - 1
        if k >= 0 and toks[k].text == ">":  # turbofish: parse::<T>()
            depth = 0
            while k >= 0:
                if toks[k].text == ">":
                    depth += 1
                elif toks[k].text == "<":
                    depth -= 1
                    if depth == 0:
                        break
                k -= 1
            k -= 3  # skip `::`
        return toks[k].text if 0 <= k < len(toks) and toks[k].kind == "ident" else ""
    if p >= 0 and toks[p].kind == "ident":
        return "<ident>"
    if p >= 0 and toks[p].text == "]":
        return "<index>"
    if p >= 0 and toks[p].kind == "str":
        return "<str>"
    return ""


def _seq(toks: Sequence[Tok], i: int, texts: Sequence[str]) -> bool:
    if i + len(texts) > len(toks):
        return False
    return all(toks[i + k].text == texts[k] for k in range(len(texts)))


def _rhs_snippet(toks: Sequence[Tok], start: int) -> str:
    out: List[str] = []
    t = start
    while t < len(toks) and len(out) < 14:
        if toks[t].kind == "punct" and toks[t].text == ";":
            break
        out.append(toks[t].text)
        t += 1
    return re.sub(r" ?([.:()!?&]) ?", r"\1", " ".join(out)).strip()


BARE_PATH_RE = re.compile(r"^[&*]?[A-Za-z_][A-Za-z0-9_.]*$")
TEARDOWN_RE = re.compile(r"remove_file|remove_dir|\.kill\(|\.wait\(|\.join\(|join_io|shutdown|\.close|ctrl_c|signal|recv\(|set_len|flush|create_dir_all|timeout\(|write!|writeln!")


def rhs_class(snippet: str) -> str:
    """Heuristic triage label for ``let _ = <rhs>``; always re-checked by hand."""
    if BARE_PATH_RE.match(snippet) or (snippet.startswith("(") and "(" not in snippet[1:]):
        return "unused-binding"
    if ".send(" in snippet:
        return "channel-send"
    if re.search(r"with_connection|with_transaction|\.execute\(|persist_|log_audit|record_|prune_|upsert_|checkpoint|transition_|reassert", snippet):
        return "db-or-state-write"
    if TEARDOWN_RE.search(snippet):
        return "teardown-io"
    return "other"


def analyze(src: str) -> FileAnalysis:
    toks, comments, lex_errors = lex(src)
    fwd, rev, bracket_errors = match_brackets(toks)
    n = len(toks)
    fa = FileAnalysis(toks, comments, lex_errors, bracket_errors, [], [], [])
    stack: List[int] = []
    i = 0

    def enclosing_close() -> int:
        return fwd.get(stack[-1], n) if stack else n

    def handle_attr_group(i: int) -> Optional[int]:
        pending: List[Tuple[str, object, int]] = []
        j = i
        while j < n and toks[j].text == "#":
            k = j + 1
            inner = k < n and toks[k].text == "!"
            if inner:
                k += 1
            if k >= n or toks[k].text != "[" or k not in fwd:
                break
            close = fwd[k]
            attr = toks[k + 1 : close]
            path = _attr_path(attr)
            effects: List[Tuple[str, object]] = []
            if path == "cfg" and len(attr) >= 3 and attr[1].text == "(":
                gate = make_gate(attr[2:-1])
                if not gate.parsed:
                    fa.unparsed_cfgs += 1
                effects.append(("gate", gate))
            elif path.split("::")[-1] in {"test", "bench"}:
                effects.append(("test_attr", path))
            elif path == "path" and len(attr) >= 3 and attr[2].kind == "str":
                effects.append(("path", attr[2].text.strip('"')))
            elif path == "unsafe":
                fa.occs.append(RawOcc("unsafe", toks[j].line, j, "attr", {"unsafe_kind": "attr", "safety": _safety_status(comments, toks, rev, j)}))
            elif path in {"forbid", "deny", "warn", "allow"} and any(t.text == "unsafe_code" for t in attr):
                fa.lint_unsafe_code.append(f"{'#!' if inner else '#'}[{path}(unsafe_code)] line {toks[j].line}")
            for kind, data in effects:
                if inner:
                    if stack:
                        fa.intervals.append(Interval(stack[-1], fwd.get(stack[-1], n - 1), kind, data))
                    else:
                        fa.intervals.append(Interval(0, max(n - 1, 0), kind, data))
                else:
                    pending.append((kind, data, close))
            j = close + 1
        if j == i:
            return None
        if pending:
            end, is_item = _unit_end(toks, fwd, j, enclosing_close())
            if not is_item:
                fa.nonitem_attr_units += 1
                if any(k == "test_attr" or (k == "gate" and d.e_prod is False) for k, d, _ in pending):
                    fa.nonitem_test_units += 1
            pa = None
            for kind, data, _ in pending:
                if kind == "path":
                    pa = data
                else:
                    fa.intervals.append(Interval(j, end, kind, data))
            if pa is not None:
                fa.intervals.append(Interval(j, end, "path", pa))
        return j

    while i < n:
        t = toks[i]
        if t.kind == "punct":
            if t.text in OPEN:
                stack.append(i)
            elif t.text in CLOSE:
                if stack:
                    stack.pop()
            elif t.text == "#":
                j = handle_attr_group(i)
                if j is not None:
                    i = j
                    continue
            elif t.text == "." and i + 2 < n and toks[i + 1].kind == "ident" and toks[i + 2].text == "(":
                name = toks[i + 1].text
                if name in UNWRAP_METHODS:
                    fa.occs.append(RawOcc(UNWRAP_METHODS[name], toks[i + 1].line, i + 1, _recv_method(toks, rev, i)))
                elif name == "ok" and _seq(toks, i + 3, [")", ";"]):
                    bound = _is_bound_statement(toks, fwd, rev, i)
                    fa.occs.append(RawOcc("ok_bound" if bound else "ok_discard", toks[i + 1].line, i + 1, _recv_method(toks, rev, i)))
                elif name in {"filter_map", "flat_map"} and (
                    _seq(toks, i + 3, ["|"]) and i + 8 < n and toks[i + 4].kind == "ident" and _seq(toks, i + 5, ["|", toks[i + 4].text, ".", "ok", "(", ")", ")"])
                    or _seq(toks, i + 3, ["Result", ":", ":", "ok", ")"])
                ):
                    fa.occs.append(RawOcc("filter_map_ok", toks[i + 1].line, i + 1))
                elif name == "unwrap_or_else" and _seq(toks, i + 3, ["|", "_", "|"]):
                    fa.occs.append(RawOcc("err_default_closure", toks[i + 1].line, i + 1))
            i += 1
            continue
        if t.kind == "ident":
            nxt = toks[i + 1] if i + 1 < n else None
            nxt2 = toks[i + 2] if i + 2 < n else None
            is_macro = nxt is not None and nxt.text == "!" and nxt2 is not None and nxt2.text in OPEN
            if is_macro and t.text in PANIC_MACROS:
                fa.occs.append(RawOcc("panic_macro", t.line, i, t.text))
            elif is_macro and t.text in ASSERT_MACROS:
                fa.occs.append(RawOcc("assert_macro", t.line, i, t.text))
            elif is_macro and t.text in PRINT_MACROS:
                fa.occs.append(RawOcc("print", t.line, i, t.text, {"stream": PRINT_MACROS[t.text]}))
            elif t.text in {"stdout", "stderr"} and _seq(toks, i + 1, ["(", ")"]) and i > 0 and toks[i - 1].text != ".":
                fa.occs.append(RawOcc("io_handle", t.line, i, t.text, {"stream": t.text}))
            elif t.text == "exit" and i >= 2 and toks[i - 1].text == ":" and toks[i - 3].text == "process" and nxt is not None and nxt.text == "(":
                fa.occs.append(RawOcc("process_exit", t.line, i))
            elif t.text == "let" and _seq(toks, i + 1, ["_", "="]):
                snippet = _rhs_snippet(toks, i + 3)
                fa.occs.append(RawOcc("let_underscore", t.line, i, snippet, {"rhs_class": rhs_class(snippet)}))
            elif t.text == "Err" and (_seq(toks, i + 1, ["(", "_", ")", "=", ">", "{", "}"]) or _seq(toks, i + 1, ["(", "_", ")", "=", ">", "(", ")"])):
                fa.occs.append(RawOcc("err_arm_ignored", t.line, i))
            elif t.text == "unsafe":
                kind = _is_unsafe_kind(toks, i)
                fa.occs.append(RawOcc("unsafe", t.line, i, kind, {"unsafe_kind": kind, "safety": _safety_status(comments, toks, rev, i)}))
            elif t.text == "macro_rules" and nxt is not None and nxt.text == "!" and i + 3 < n and toks[i + 2].kind == "ident" and toks[i + 3].text in OPEN and (i + 3) in fwd:
                fa.intervals.append(Interval(i + 3, fwd[i + 3], "macro_def", toks[i + 2].text))
            elif t.text == "include" and is_macro and nxt2.text == "(" and i + 3 < n and toks[i + 3].kind == "str" and (i + 4) < n and toks[i + 4].text == ")":
                fa.mods.append(ModDecl("", i, t.line, None, True, toks[i + 3].text.strip('"')))
            elif t.text == "include" and is_macro:
                fa.mods.append(ModDecl("", i, t.line, None, True, None))
            elif t.text == "fn" and nxt is not None and nxt.kind == "ident" and (i == 0 or toks[i - 1].text != "."):
                name = nxt.text
                if name in {"unwrap", "expect"}:
                    fa.custom_defs.append(f"fn {name} line {t.line}")
                k = i + 2
                while k < n:
                    tk = toks[k]
                    if tk.kind == "punct" and tk.text in OPEN:
                        if tk.text == "{":
                            if k in fwd:
                                fa.intervals.append(Interval(k, fwd[k], "fn", name))
                            break
                        k = fwd.get(k, k) + 1
                        continue
                    if tk.kind == "punct" and tk.text == ";":
                        break
                    k += 1
            elif t.text == "mod" and nxt is not None and nxt.kind == "ident" and (i == 0 or toks[i - 1].text != "."):
                if nxt2 is not None and nxt2.text == "{" and (i + 2) in fwd:
                    fa.intervals.append(Interval(i + 2, fwd[i + 2], "mod", nxt.text))
                elif nxt2 is not None and nxt2.text == ";":
                    fa.mods.append(ModDecl(nxt.text, i, t.line, None))
        i += 1
    return fa


# --------------------------------------------------------------------------
# Context at a token
# --------------------------------------------------------------------------


@dataclass
class TokCtx:
    gates: List[Gate]
    test_attr: bool
    macro_def: bool
    fn_name: str
    mods: Tuple[str, ...]
    path_attr: Optional[str]


def ctx_at(fa: FileAnalysis, idx: int) -> TokCtx:
    gates: List[Gate] = []
    test_attr = False
    macro_def = False
    best_fn: Optional[Interval] = None
    mods: List[Interval] = []
    path_attr = None
    for iv in fa.intervals:
        if not (iv.start <= idx <= iv.end):
            continue
        if iv.kind == "gate":
            gates.append(iv.data)  # type: ignore[arg-type]
        elif iv.kind == "test_attr":
            test_attr = True
        elif iv.kind == "macro_def":
            macro_def = True
        elif iv.kind == "fn":
            if best_fn is None or (iv.end - iv.start) < (best_fn.end - best_fn.start):
                best_fn = iv
        elif iv.kind == "mod":
            mods.append(iv)
        elif iv.kind == "path":
            path_attr = iv.data  # type: ignore[assignment]
    mods.sort(key=lambda m: m.start)
    return TokCtx(gates, test_attr, macro_def, str(best_fn.data) if best_fn else "", tuple(str(m.data) for m in mods), path_attr)


# --------------------------------------------------------------------------
# Crate / target discovery and module-tree resolution
# --------------------------------------------------------------------------


@dataclass
class Root:
    path: str
    kind: str  # lib | bin | integration-test | bench | example | build-script
    crate: str
    label: str


@dataclass
class FileCtx:
    root_kind: str
    root_label: str
    crate: str
    gates: Tuple[Gate, ...]
    test_attr: bool
    chain: Tuple[str, ...]
    also_from: List[str] = field(default_factory=list)
    conflict: bool = False


def list_rust_files(root: Path, mode: str) -> List[str]:
    if mode == "git":
        try:
            out = subprocess.run(
                ["git", "ls-files", "-z", "--", "*.rs"], cwd=root, capture_output=True, check=True
            ).stdout.decode("utf-8")
            files = [f for f in out.split("\0") if f]
            return sorted(f for f in files if (root / f).is_file())
        except (OSError, subprocess.CalledProcessError):
            pass
    skip = {"target", ".git", ".claude", ".worktrees", "node_modules"}
    files = []
    for p in root.rglob("*.rs"):
        rel = p.relative_to(root)
        if any(part in skip for part in rel.parts):
            continue
        files.append(rel.as_posix())
    return sorted(files)


def find_crate_dirs(root: Path, files: Sequence[str]) -> List[str]:
    dirs = set()
    for f in files:
        p = PurePosixPath(f).parent
        while True:
            if (root / p / "Cargo.toml").is_file():
                dirs.add(p.as_posix() if p.as_posix() != "." else "")
                break
            if p.as_posix() in {".", ""}:
                break
            p = p.parent
    return sorted(dirs)


def discover_roots(root: Path, files: Sequence[str]) -> List[Root]:
    fileset = set(files)
    roots: List[Root] = []
    seen = set()

    def add(path: str, kind: str, crate: str, label: str) -> None:
        if path in fileset and path not in seen:
            seen.add(path)
            roots.append(Root(path, kind, crate, label))

    for cd in find_crate_dirs(root, files):
        base = (cd + "/") if cd else ""
        try:
            manifest = tomllib.loads((root / cd / "Cargo.toml").read_text())
        except (OSError, tomllib.TOMLDecodeError):
            manifest = {}
        crate = manifest.get("package", {}).get("name") or (cd or "root")
        lib = manifest.get("lib", {})
        add(base + lib.get("path", "src/lib.rs"), "lib", crate, f"{crate}:lib")
        for b in manifest.get("bin", []):
            if "path" in b:
                add(base + b["path"], "bin", crate, f"{crate}:bin:{b.get('name', Path(b['path']).stem)}")
        for kind, section, folder in (("integration-test", "test", "tests"), ("bench", "bench", "benches"), ("example", "example", "examples")):
            for b in manifest.get(section, []):
                if "path" in b:
                    add(base + b["path"], kind, crate, f"{crate}:{kind}:{b.get('name', Path(b['path']).stem)}")
        add(base + "src/main.rs", "bin", crate, f"{crate}:bin:{crate}")
        build = manifest.get("package", {}).get("build", "build.rs")
        if isinstance(build, str):
            add(base + build, "build-script", crate, f"{crate}:build-script")
        for f in sorted(fileset):
            if not f.startswith(base):
                continue
            rel = f[len(base):]
            parts = rel.split("/")
            if parts[0] == "src" and len(parts) >= 3 and parts[1] == "bin":
                if len(parts) == 3:
                    add(f, "bin", crate, f"{crate}:bin:{Path(parts[2]).stem}")
                elif len(parts) == 4 and parts[3] == "main.rs":
                    add(f, "bin", crate, f"{crate}:bin:{parts[2]}")
            for kind, folder in (("integration-test", "tests"), ("bench", "benches"), ("example", "examples")):
                if parts[0] == folder:
                    if len(parts) == 2:
                        add(f, kind, crate, f"{crate}:{kind}:{Path(parts[1]).stem}")
                    elif len(parts) == 3 and parts[2] == "main.rs":
                        add(f, kind, crate, f"{crate}:{kind}:{parts[1]}")
    order = {"lib": 0, "bin": 1, "build-script": 2, "integration-test": 3, "bench": 4, "example": 5}
    return sorted(roots, key=lambda r: (order[r.kind], r.path))


def _child_dir(path: str, is_root: bool) -> PurePosixPath:
    p = PurePosixPath(path)
    if is_root or p.name in {"lib.rs", "main.rs", "mod.rs"}:
        return p.parent
    return p.parent / p.stem


@dataclass
class Inventory:
    root: str
    files: List[str]
    analyses: Dict[str, FileAnalysis]
    ctxs: Dict[str, FileCtx]
    roots: List[Root]
    unresolved: List[str]
    orphans: List[str]
    occurrences: List[dict]
    stats: Dict[str, object]


def resolve_tree(root: Path, files: Sequence[str], roots: Sequence[Root], analyses: Dict[str, FileAnalysis]) -> Tuple[Dict[str, FileCtx], List[str]]:
    fileset = set(files)
    ctxs: Dict[str, FileCtx] = {}
    unresolved: List[str] = []
    for r in roots:
        queue: List[Tuple[str, Tuple[Gate, ...], bool, Tuple[str, ...], bool, bool]] = [(r.path, (), False, (), True, False)]
        visited_this_root = set()
        while queue:
            path, gates, tattr, chain, is_root, is_fragment = queue.pop(0)
            if path in visited_this_root:
                continue
            visited_this_root.add(path)
            existing = ctxs.get(path)
            if existing is None:
                ctxs[path] = FileCtx(r.kind, r.label, r.crate, gates, tattr, chain)
            else:
                if existing.root_label != r.label:
                    existing.also_from.append(r.label)
                if existing.root_kind != r.kind:
                    existing.conflict = True
                continue
            fa = analyses[path]
            for md in fa.mods:
                c = ctx_at(fa, md.tok)
                g = gates + tuple(c.gates)
                ta = tattr or c.test_attr
                if md.include:
                    if md.include_path is None:
                        unresolved.append(f"{path}:{md.line} include! with non-literal path")
                        continue
                    cand = (PurePosixPath(path).parent / md.include_path).as_posix()
                    cand = _normalize(cand)
                    if cand in fileset:
                        queue.append((cand, g, ta, chain, False, True))
                    else:
                        unresolved.append(f"{path}:{md.line} include!({md.include_path}) not found")
                    continue
                base = _child_dir(path, is_root) if not is_fragment else PurePosixPath(path).parent
                for m in c.mods:
                    base = base / m
                cands: List[str] = []
                if c.path_attr:
                    cands.append(_normalize((PurePosixPath(path).parent / c.path_attr).as_posix()))
                else:
                    cands.append((base / f"{md.name}.rs").as_posix())
                    cands.append((base / md.name / "mod.rs").as_posix())
                hit = next((x for x in cands if x in fileset), None)
                if hit is None:
                    unresolved.append(f"{path}:{md.line} mod {md.name} not found")
                    continue
                queue.append((hit, g, ta, chain + (md.name,), False, False))
    return ctxs, unresolved


def _normalize(p: str) -> str:
    parts: List[str] = []
    for seg in p.split("/"):
        if seg == "..":
            if parts:
                parts.pop()
        elif seg not in {".", ""}:
            parts.append(seg)
    return "/".join(parts)


# --------------------------------------------------------------------------
# Classification
# --------------------------------------------------------------------------

SCOPE_BY_ROOT = {
    "lib": "prod-lib",
    "bin": "prod-bin",
    "integration-test": "test-integration",
    "bench": "bench",
    "example": "example",
    "build-script": "build-script",
}


def classify(fctx: Optional[FileCtx], tctx: TokCtx) -> Tuple[str, bool, Tuple[str, ...], str]:
    """Return (scope, conditional, features, note)."""
    if fctx is None:
        return "unknown", False, (), "file not reachable from any Cargo target root"
    gates = list(fctx.gates) + list(tctx.gates)
    if any(not g.parsed for g in gates):
        return "unknown", False, (), "unparsed cfg predicate"
    if any(g.e_prod is False and g.e_test is False for g in gates):
        return "disabled", False, (), "cfg is never true"
    if tctx.macro_def:
        return "macro-def", False, (), "inside macro_rules! body; expansion context unknown"
    is_test = tctx.test_attr or fctx.test_attr or any(g.e_prod is False for g in gates)
    features = tuple(sorted({f for g in gates for f in g.features}))
    conditional = any(g.e_prod is None for g in gates)
    if fctx.conflict:
        return "unknown", conditional, features, "file reached from conflicting target kinds"
    if is_test:
        if fctx.root_kind in {"lib", "bin", "build-script"}:
            return "test-unit", conditional, features, ""
        return SCOPE_BY_ROOT[fctx.root_kind] if fctx.root_kind != "integration-test" else "test-integration", conditional, features, ""
    return SCOPE_BY_ROOT[fctx.root_kind], conditional, features, ""


def build_inventory(root: Path, file_mode: str = "git") -> Inventory:
    files = list_rust_files(root, file_mode)
    analyses: Dict[str, FileAnalysis] = {}
    for f in files:
        analyses[f] = analyze((root / f).read_text(encoding="utf-8", errors="replace"))
    roots = discover_roots(root, files)
    ctxs, unresolved = resolve_tree(root, files, roots, analyses)
    orphans = [f for f in files if f not in ctxs]
    occs: List[dict] = []
    for f in files:
        fa = analyses[f]
        fctx = ctxs.get(f)
        lines_cache: Optional[List[str]] = None
        for o in fa.occs:
            tctx = ctx_at(fa, o.tok)
            scope, conditional, features, note = classify(fctx, tctx)
            if lines_cache is None:
                lines_cache = (root / f).read_text(encoding="utf-8", errors="replace").splitlines()
            snippet = lines_cache[o.line - 1].strip()[:140] if 0 < o.line <= len(lines_cache) else ""
            rec = {
                "path": f,
                "line": o.line,
                "kind": o.kind,
                "detail": o.detail,
                "scope": scope,
                "target": fctx.root_label if fctx else "unknown",
                "crate": fctx.crate if fctx else "unknown",
                "fn": tctx.fn_name,
                "conditional": conditional,
                "features": list(features),
                "reachability": "unknown",
                "note": note,
                "snippet": snippet,
            }
            rec.update(o.extra)
            occs.append(rec)
    occs.sort(key=lambda r: (r["path"], r["line"], r["kind"]))
    tok_total = sum(len(a.toks) for a in analyses.values())
    stats = {
        "files": len(files),
        "tokens": tok_total,
        "roots": len(roots),
        "files_reached": len(ctxs),
        "orphan_files": len(orphans),
        "unresolved_mods": len(unresolved),
        "lex_errors": sum(len(a.lex_errors) for a in analyses.values()),
        "bracket_errors_files": sum(1 for a in analyses.values() if a.bracket_errors),
        "unparsed_cfg_attrs": sum(a.unparsed_cfgs for a in analyses.values()),
        "nonitem_cfg_units_approximated": sum(a.nonitem_attr_units for a in analyses.values()),
        "nonitem_test_gated_units_approximated": sum(a.nonitem_test_units for a in analyses.values()),
        "files_multi_root": sum(1 for c in ctxs.values() if c.also_from),
        "files_conflicting_root_kind": sum(1 for c in ctxs.values() if c.conflict),
        "custom_unwrap_expect_defs": sorted(f"{f}: {d}" for f, a in analyses.items() for d in a.custom_defs),
        "unsafe_code_lints": sorted(f"{f}: {d}" for f, a in analyses.items() for d in a.lint_unsafe_code),
    }
    return Inventory(root.as_posix(), files, analyses, ctxs, roots, unresolved, orphans, occs, stats)


# --------------------------------------------------------------------------
# Reporting
# --------------------------------------------------------------------------

KIND_ORDER = ["unwrap", "expect", "unwrap_err", "expect_err", "unwrap_unchecked", "panic_macro", "assert_macro", "unsafe", "print", "io_handle", "process_exit", "let_underscore", "ok_discard", "ok_bound", "filter_map_ok", "err_default_closure", "err_arm_ignored"]
SCOPE_ORDER = ["prod-lib", "prod-bin", "test-unit", "test-integration", "bench", "example", "build-script", "macro-def", "disabled", "unknown"]


def head_sha(root: Path) -> str:
    try:
        return subprocess.run(["git", "rev-parse", "HEAD"], cwd=root, capture_output=True, check=True).stdout.decode().strip()
    except (OSError, subprocess.CalledProcessError):
        return "unknown"


def count_table(occs: Iterable[dict], kinds: Sequence[str]) -> List[str]:
    c: Counter = Counter((o["kind"], o["scope"]) for o in occs)
    scopes = [s for s in SCOPE_ORDER if any(c[(k, s)] for k in kinds)]
    lines = ["| kind | " + " | ".join(scopes) + " | total |", "|---|" + "---:|" * (len(scopes) + 1)]
    for k in kinds:
        row = [c[(k, s)] for s in scopes]
        if sum(row) == 0:
            continue
        lines.append(f"| {k} | " + " | ".join(str(x) for x in row) + f" | {sum(row)} |")
    return lines


def render_summary(inv: Inventory) -> str:
    out = ["# Rust risk inventory (tool v%d)" % TOOL_VERSION, ""]
    out += ["## Stats", ""]
    for k, v in inv.stats.items():
        if isinstance(v, list):
            out.append(f"- {k}: {len(v)}")
            out.extend(f"  - {x}" for x in v)
        else:
            out.append(f"- {k}: {v}")
    out += ["", "## Counts by kind x scope", ""] + count_table(inv.occurrences, KIND_ORDER)
    prod = [o for o in inv.occurrences if o["scope"] in {"prod-lib", "prod-bin"}]
    out += ["", "## Production counts by crate/target", ""]
    by: Counter = Counter((o["target"], o["kind"]) for o in prod)
    targets = sorted({o["target"] for o in prod})
    kinds = [k for k in KIND_ORDER if any(by[(t, k)] for t in targets)]
    out += ["| target | " + " | ".join(kinds) + " |", "|---|" + "---:|" * len(kinds)]
    for t in targets:
        out.append(f"| {t} | " + " | ".join(str(by[(t, k)]) for k in kinds) + " |")
    if inv.unresolved:
        out += ["", "## Unresolved module declarations", ""] + [f"- {u}" for u in inv.unresolved]
    if inv.orphans:
        out += ["", "## Orphan files (not reachable from any target root; scope unknown)", ""] + [f"- {o}" for o in inv.orphans]
    return "\n".join(out) + "\n"


def filter_occs(occs: Iterable[dict], kinds: Sequence[str], scopes: Sequence[str]) -> List[dict]:
    return [o for o in occs if (not kinds or o["kind"] in kinds) and (not scopes or o["scope"] in scopes)]


def parse_args(argv: Optional[Sequence[str]] = None) -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--root", default=".", help="repository root (default: .)")
    p.add_argument("--format", choices=["summary", "list", "json"], default="summary")
    p.add_argument("--kind", action="append", default=[], help="filter by kind (repeatable)")
    p.add_argument("--scope", action="append", default=[], help="filter by scope (repeatable)")
    p.add_argument("--files", choices=["git", "walk"], default="git", help="file discovery mode")
    return p.parse_args(argv)


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = parse_args(argv)
    root = Path(args.root).resolve()
    inv = build_inventory(root, args.files)
    occs = filter_occs(inv.occurrences, args.kind, args.scope)
    if args.format == "json":
        meta = {"tool_version": TOOL_VERSION, "head": head_sha(root), "file_mode": args.files, "stats": inv.stats,
                "unresolved": inv.unresolved, "orphans": inv.orphans, "roots": [asdict(r) for r in inv.roots]}
        json.dump({"meta": meta, "occurrences": occs}, sys.stdout, indent=1, sort_keys=True)
        sys.stdout.write("\n")
    elif args.format == "list":
        for o in occs:
            extra = f" [{o['detail']}]" if o["detail"] else ""
            print(f"{o['path']}:{o['line']}: {o['kind']}{extra} scope={o['scope']} fn={o['fn'] or '-'}")
    else:
        sys.stdout.write(render_summary(inv))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

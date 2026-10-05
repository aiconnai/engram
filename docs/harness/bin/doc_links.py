#!/usr/bin/env python3
"""Offline Markdown link and anchor checker for the harness live-state documents (task H6).

Usage:
  python3 docs/harness/bin/check-doc-links.py [--root DIR] [--allow-missing PATH ...] FILE.md [FILE.md ...]

For every inline link or reference definition in each FILE.md that is not an external URL
(scheme, `//`, mailto) it requires that
  * the target path (relative to the file that contains the link; `/x` is relative to --root)
    exists and, when --root is inside a git work tree, is tracked or at least not git-ignored
    (a link to an ignored file resolves locally but breaks in a clean clone);
  * a `#fragment` names an existing heading (GitHub slug rules, `-1`/`-2` suffixes for
    duplicates), or an explicit `<a id|name="...">` anchor, in the target Markdown file
    (or in the file itself for `#fragment` links).

Links inside fenced code blocks and inline code spans are ignored. `--allow-missing` names
targets (paths relative to --root) that are knowingly pending; each use is reported as ALLOWED so
the exception stays visible. Exit 0 when every link resolves, 1 otherwise, 2 on usage errors.
No network, no dependencies beyond the standard library.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
import unicodedata
from pathlib import Path
from urllib.parse import unquote

FENCE_RE = re.compile(r"^\s{0,3}(`{3,}|~{3,})")
INLINE_CODE_RE = re.compile(r"(`+)(?:.+?)\1")
INLINE_LINK_RE = re.compile(r"!?\[(?:[^\[\]]|\[[^\]]*\])*\]\(\s*(<[^>]*>|[^()\s]*(?:\([^()]*\)[^()\s]*)*)(?:\s+(?:\"[^\"]*\"|'[^']*'))?\s*\)")
REF_DEF_RE = re.compile(r"^\s{0,3}\[[^\]]+\]:\s*(<[^>]*>|\S+)")
HEADING_RE = re.compile(r"^\s{0,3}(#{1,6})\s+(.*?)\s*#*\s*$")
HTML_ANCHOR_RE = re.compile(r"<a\s+[^>]*?(?:id|name)=[\"']([^\"']+)[\"']", re.IGNORECASE)
EXTERNAL_RE = re.compile(r"^(?:[a-zA-Z][a-zA-Z0-9+.-]*:|//)")


def strip_inline_markdown(text: str) -> str:
    text = re.sub(r"!?\[([^\]]*)\]\([^)]*\)", r"\1", text)
    text = re.sub(r"<[^>]+>", "", text)
    text = text.replace("`", "")
    text = re.sub(r"[*_~]{1,3}(\S(?:.*?\S)?)[*_~]{1,3}", r"\1", text)
    return text


def github_slug(heading_text: str) -> str:
    text = unicodedata.normalize("NFC", strip_inline_markdown(heading_text)).strip().lower()
    out = []
    for ch in text:
        if ch in " -":
            out.append("-" if ch == " " else ch)
        elif ch == "_" or ch.isalnum():
            out.append(ch)
        elif unicodedata.category(ch).startswith(("L", "N", "M")):
            out.append(ch)
    return "".join(out)


def iter_unfenced_lines(text: str):
    fence = None
    for line in text.splitlines():
        m = FENCE_RE.match(line)
        if m:
            marker = m.group(1)
            if fence is None:
                fence = marker[0]
            elif marker[0] == fence:
                fence = None
            continue
        if fence is None:
            yield line


def anchors_of(path: Path, cache: dict[Path, set[str]]) -> set[str]:
    if path in cache:
        return cache[path]
    anchors: set[str] = set()
    seen: dict[str, int] = {}
    for line in iter_unfenced_lines(path.read_text(encoding="utf-8", errors="replace")):
        m = HEADING_RE.match(line)
        if m:
            slug = github_slug(m.group(2))
            n = seen.get(slug, 0)
            anchors.add(slug if n == 0 else f"{slug}-{n}")
            seen[slug] = n + 1
        anchors.update(HTML_ANCHOR_RE.findall(line))
    cache[path] = anchors
    return anchors


def extract_targets(path: Path) -> list[tuple[int, str]]:
    targets: list[tuple[int, str]] = []
    fence = None
    for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
        m = FENCE_RE.match(line)
        if m:
            marker = m.group(1)
            if fence is None:
                fence = marker[0]
            elif marker[0] == fence:
                fence = None
            continue
        if fence is not None:
            continue
        scrubbed = INLINE_CODE_RE.sub(lambda mm: " " * len(mm.group(0)), line)
        ref = REF_DEF_RE.match(scrubbed)
        if ref:
            targets.append((lineno, ref.group(1)))
        for mm in INLINE_LINK_RE.finditer(scrubbed):
            targets.append((lineno, mm.group(1)))
    return targets


def git_visible(root: Path, target: Path) -> bool:
    """True unless the path is git-ignored (a clean clone would not contain it)."""
    try:
        rel = target.relative_to(root)
    except ValueError:
        return True
    probe = subprocess.run(
        ["git", "-C", str(root), "check-ignore", "-q", "--", str(rel)],
        capture_output=True,
        check=False,
    )
    return probe.returncode != 0  # 0 = ignored, 1 = not ignored, 128 = not a repo


def check_file(path: Path, root: Path, allow_missing: set[str], cache: dict[Path, set[str]]):
    problems: list[str] = []
    allowed: list[str] = []
    for lineno, raw in extract_targets(path):
        target = raw.strip()
        if target.startswith("<") and target.endswith(">"):
            target = target[1:-1]
        if not target or EXTERNAL_RE.match(target):
            continue
        link_path, _, fragment = target.partition("#")
        link_path = unquote(link_path.split("?", 1)[0])
        fragment = unquote(fragment)
        if not link_path:
            resolved = path
        elif link_path.startswith("/"):
            resolved = (root / link_path.lstrip("/")).resolve()
        else:
            resolved = (path.parent / link_path).resolve()
        where = f"{path.relative_to(root) if path.is_relative_to(root) else path}:{lineno}"
        try:
            rel_resolved = str(resolved.relative_to(root))
        except ValueError:
            rel_resolved = str(resolved)
        if not resolved.exists():
            if rel_resolved in allow_missing:
                allowed.append(f"ALLOWED {where}: pending target {rel_resolved}")
            else:
                problems.append(f"BROKEN {where}: target does not exist: {raw}")
            continue
        if link_path and not git_visible(root, resolved):
            problems.append(f"BROKEN {where}: target is git-ignored (absent in a clean clone): {raw}")
            continue
        if fragment:
            if resolved.is_dir() or resolved.suffix.lower() not in (".md", ".markdown"):
                problems.append(f"BROKEN {where}: anchor on a non-Markdown target: {raw}")
            elif fragment not in anchors_of(resolved, cache):
                problems.append(f"BROKEN {where}: anchor #{fragment} not found in {rel_resolved}")
    return problems, allowed


def section_text(path: Path, anchor: str) -> str | None:
    """Whole Markdown section for `anchor`: its heading through the line before the next heading
    of the same or a higher level (fenced code is never mistaken for a heading), so a reader
    never gets a cut-off paragraph. None when the anchor names no heading."""
    lines = path.read_text(encoding="utf-8", errors="replace").splitlines(keepends=True)
    seen: dict[str, int] = {}
    fence = None
    start = None
    level = 0
    for i, line in enumerate(lines):
        fm = FENCE_RE.match(line)
        if fm:
            marker = fm.group(1)
            if fence is None:
                fence = marker[0]
            elif marker[0] == fence:
                fence = None
            continue
        if fence is not None:
            continue
        hm = HEADING_RE.match(line.rstrip("\n"))
        if not hm:
            continue
        this_level = len(hm.group(1))
        if start is not None and this_level <= level:
            return "".join(lines[start:i])
        slug = github_slug(hm.group(2))
        n = seen.get(slug, 0)
        seen[slug] = n + 1
        if start is None and (slug if n == 0 else f"{slug}-{n}") == anchor:
            start, level = i, this_level
    return "".join(lines[start:]) if start is not None else None


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Offline Markdown link/anchor checker")
    parser.add_argument("files", nargs="+")
    parser.add_argument("--root", default=".", help="repository root (default: cwd)")
    parser.add_argument("--allow-missing", action="append", default=[], metavar="PATH",
                        help="knowingly pending target, relative to --root (repeatable)")
    args = parser.parse_args(argv)
    root = Path(args.root).resolve()
    allow = set(args.allow_missing)
    cache: dict[Path, set[str]] = {}
    all_problems: list[str] = []
    checked = 0
    for name in args.files:
        path = (root / name).resolve() if not Path(name).is_absolute() else Path(name)
        if not path.is_file():
            all_problems.append(f"BROKEN {name}: file to check does not exist")
            continue
        problems, allowed = check_file(path, root, allow, cache)
        checked += 1
        all_problems.extend(problems)
        for line in allowed:
            print(line)
    for line in all_problems:
        print(line)
    if all_problems:
        print(f"LINK_CHECK: FAIL files={checked} broken={len(all_problems)}")
        return 1
    print(f"LINK_CHECK: PASS files={checked}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

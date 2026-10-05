#!/usr/bin/env python3
"""Retention manifest, backup, restore and read-only inventory for harness artifact classes (task O2).

Stdlib only, offline. Policy: docs/OPERATIONS_GIT_RETENTION.md.

Subcommands (all take --root, default: the repository this script lives in):

  inventory      read-only snapshot with timestamps: tracked artifact classes, git object store,
                 refs/worktrees, ignored files on disk, largest blobs, sensitivity flags (names only)
  generate       write a manifest (path, size, sha256, git blob id, introducing commit, sensitivity
                 flag names) for one artifact class; refuses files that differ from the index
  verify         compare the working tree with a manifest (exit 0 ok, 1 mismatch, 4 missing)
  backup         copy the class into a content-addressed store outside the repository
  verify-backup  check every manifest entry exists in the backup with the recorded hash
  restore        recreate missing files from a backup (--backup DIR) or from git history (--from-git),
                 hash-checked before anything is written; never overwrites a different file
                 unless --force-overwrite
  untrack        `git rm --cached` the class and add an ignore rule. Dry run by default; with --apply
                 it requires a verified backup. It deletes no file, creates no commit and does not
                 touch history, refs or the object store.

Exit codes: 0 ok, 1 failure (mismatch, corrupt source, conflict), 2 usage/unsafe input, 4 missing.
Sensitive-looking content is reported by flag name and path only; it is never printed or stored.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional, Sequence, Tuple

SCHEMA_VERSION = 1
DEFAULT_ROOT = Path(__file__).resolve().parent.parent.parent.parent

# Artifact classes. `pattern` selects tracked paths; `ignore_rule` is what `untrack` appends to .gitignore.
CLASSES: Dict[str, Dict[str, object]] = {
    "review-raw": {
        "pattern": re.compile(r"^docs/harness/reviews/[^/]+\.raw$"),
        "ignore_rule": "docs/harness/reviews/*.raw",
        "description": "review-gate.sh prompt/diff dumps (.raw) next to each review verdict artifact",
    },
}

# Classes shown by `inventory` (a superset of the migratable ones above; the rest are measured only).
INVENTORY_CLASSES: List[Tuple[str, "re.Pattern[str]"]] = [
    ("review-raw", CLASSES["review-raw"]["pattern"]),  # type: ignore[list-item]
    ("review-verdicts", re.compile(r"^docs/harness/reviews/[^/]+\.md$")),
    ("progress-logs", re.compile(r"^docs/harness/(progress/[^/]+\.md|progress-history\.md|progress\.md)$")),
    ("harness-audits", re.compile(r"^docs/harness/audits/")),
    ("harness-canvas", re.compile(r"^docs/harness/canvas/")),
    ("benchmark-results", re.compile(r"^benches/results/")),
    ("benchmark-history-dev-bench", re.compile(r"^dev/bench")),  # gh-pages benchmark JSON, history only
    ("tracked-logs-data", re.compile(r"\.(log|jsonl|csv)$")),
]

SENSITIVE_PATTERNS: List[Tuple[str, "re.Pattern[bytes]"]] = [
    ("github-token", re.compile(rb"gh[pousr]_[A-Za-z0-9]{30,}")),
    ("aws-access-key", re.compile(rb"AKIA[0-9A-Z]{16}")),
    ("api-key-sk", re.compile(rb"\bsk-[A-Za-z0-9_-]{20,}")),
    ("slack-token", re.compile(rb"xox[baprs]-[A-Za-z0-9-]{10,}")),
    ("private-key-block", re.compile(rb"-----BEGIN [A-Z ]*PRIVATE KEY-----")),
    ("bearer-token", re.compile(rb"(?i)bearer\s+[A-Za-z0-9._~+/-]{24,}")),
    ("credential-assignment",
     re.compile(rb"(?i)\b(password|passwd|secret|api[_-]?key|access[_-]?token|auth[_-]?token)\b\s*[:=]\s*['\"]?[A-Za-z0-9/+_.-]{12,}")),
    ("email-address", re.compile(rb"[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+\.[A-Za-z0-9.-]+")),
    ("abs-home-path", re.compile(rb"/Users/[A-Za-z0-9._-]+/|/home/[a-z_][a-z0-9_-]*/")),
]
LOW_SEVERITY_FLAGS = {"abs-home-path", "email-address"}

EXIT_FAIL, EXIT_USAGE, EXIT_MISSING = 1, 2, 4
MAX_LISTED = 12  # verify prints at most this many MISSING/MISMATCH/UNLISTED lines unless --all


class Usage(Exception):
    """Unsafe or malformed input (exit 2)."""


class Failure(Exception):
    """Operation failed (exit 1)."""


# --------------------------------------------------------------------------------------- helpers


def utc_now() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def git_env() -> Dict[str, str]:
    env = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}
    env.update({"LC_ALL": "C", "GIT_OPTIONAL_LOCKS": "0", "GIT_TERMINAL_PROMPT": "0"})
    return env


def git(root: Path, *args: str, check: bool = True, input_bytes: Optional[bytes] = None) -> bytes:
    proc = subprocess.run(
        ["git", "-c", "core.quotepath=false", "-C", str(root), *args],
        env=git_env(), capture_output=True, input=input_bytes, timeout=300,
    )
    if check and proc.returncode != 0:
        raise Failure(f"git {' '.join(args[:3])} failed: {proc.stderr.decode(errors='replace').strip()[:200]}")
    return proc.stdout if proc.returncode == 0 else b""


def git_blob_id(data: bytes) -> str:
    return hashlib.sha1(b"blob %d\0" % len(data) + data).hexdigest()  # noqa: S324 (git object id, not security)


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def class_def(name: str) -> Dict[str, object]:
    if name not in CLASSES:
        raise Usage(f"unknown class {name!r}; known: {', '.join(sorted(CLASSES))}")
    return CLASSES[name]


def tracked_index(root: Path) -> Dict[str, str]:
    """path -> blob id for every tracked file (index, stage 0)."""
    out = git(root, "ls-files", "-s", "-z")
    index: Dict[str, str] = {}
    for rec in out.split(b"\0"):
        if not rec:
            continue
        meta, _, path = rec.partition(b"\t")
        fields = meta.split()
        if len(fields) == 3 and fields[2] == b"0":
            index[path.decode("utf-8", "surrogateescape")] = fields[1].decode()
    return index


def safe_target(root: Path, rel: str, pattern: "re.Pattern[str]") -> Path:
    """Resolve a manifest path inside root or raise Usage. No absolute paths, no '..', no symlinked parents."""
    parts = Path(rel).parts
    if not rel or os.path.isabs(rel) or ".." in parts or "\0" in rel or not pattern.match(rel):
        raise Usage(f"unsafe path in manifest: {rel!r}")
    cur = root
    for part in parts[:-1]:
        cur = cur / part
        if cur.is_symlink():
            raise Usage(f"unsafe path in manifest (symlinked parent): {rel!r}")
    target = root / rel
    if target.is_symlink():
        raise Usage(f"unsafe path in manifest (symlink target): {rel!r}")
    return target


def load_manifest(path: Path) -> dict:
    try:
        m = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        raise Usage(f"cannot read manifest {path}: {exc}") from exc
    if not isinstance(m, dict) or m.get("schema_version") != SCHEMA_VERSION or m.get("class") not in CLASSES:
        raise Usage("manifest has an unsupported schema_version or class")
    entries = m.get("entries")
    if not isinstance(entries, list):
        raise Usage("manifest has no entries list")
    for e in entries:
        if not isinstance(e, dict) or not all(k in e for k in ("path", "size", "sha256", "git_blob")):
            raise Usage("manifest entry missing path/size/sha256/git_blob")
    return m


def scan_sensitivity(data: bytes) -> List[str]:
    return sorted(name for name, rx in SENSITIVE_PATTERNS if rx.search(data))


def atomic_write(target: Path, data: bytes) -> None:
    target.parent.mkdir(parents=True, exist_ok=True)
    fd, tmp = tempfile.mkstemp(prefix=".o2-", dir=str(target.parent))
    try:
        with os.fdopen(fd, "wb") as fh:
            fh.write(data)
        os.chmod(tmp, 0o644)
        os.replace(tmp, target)
    except BaseException:
        if os.path.exists(tmp):
            os.unlink(tmp)
        raise


def backup_object(backup: Path, digest: str) -> Path:
    return backup / "objects" / digest[:2] / digest


def read_entry_source(root: Path, entry: dict, backup: Optional[Path], from_git: bool) -> bytes:
    """Bytes for an entry from the chosen source, verified against the manifest hash and size."""
    if backup is not None:
        obj = backup_object(backup, entry["sha256"])
        try:
            data = obj.read_bytes()
        except OSError as exc:
            raise Failure(f"{entry['path']}: backup object absent ({exc.strerror})") from exc
    elif from_git:
        proc = subprocess.run(["git", "-C", str(root), "cat-file", "blob", entry["git_blob"]],
                              env=git_env(), capture_output=True, timeout=120)
        if proc.returncode != 0:
            raise Failure(f"{entry['path']}: blob {entry['git_blob'][:12]} not in the git object store")
        data = proc.stdout
    else:  # pragma: no cover - guarded by callers
        raise Usage("no source")
    if len(data) != entry["size"] or sha256_hex(data) != entry["sha256"]:
        raise Failure(f"{entry['path']}: hash mismatch against the manifest (source corrupt)")
    return data


# ------------------------------------------------------------------------------------- generate


def cmd_generate(args: argparse.Namespace) -> int:
    root = Path(args.root)
    cdef = class_def(args.cls)
    pattern = cdef["pattern"]
    index = tracked_index(root)
    entries = []
    for rel in sorted(p for p in index if pattern.match(p)):  # type: ignore[union-attr]
        target = safe_target(root, rel, pattern)  # type: ignore[arg-type]
        try:
            data = target.read_bytes()
        except OSError as exc:
            raise Failure(f"{rel}: tracked but unreadable in the working tree ({exc.strerror})") from exc
        if git_blob_id(data) != index[rel]:
            raise Failure(f"{rel}: working-tree content differs from the index; commit or restore it first")
        introduced = git(root, "log", "--diff-filter=A", "--format=%H", "-1", "--", rel, check=False).decode().strip()
        flags = scan_sensitivity(data)
        entries.append({
            "path": rel, "size": len(data), "sha256": sha256_hex(data), "git_blob": index[rel],
            "introduced_commit": introduced or None, "sensitive": bool(flags), "sensitive_flags": flags,
        })
    head = git(root, "rev-parse", "HEAD", check=False).decode().strip() or None
    manifest = {
        "schema_version": SCHEMA_VERSION, "class": args.cls, "description": cdef["description"],
        "generated_at": utc_now(), "source_commit": head,
        "totals": {"files": len(entries), "bytes": sum(e["size"] for e in entries)},
        "entries": entries,
    }
    out = Path(args.out)
    atomic_write(out, (json.dumps(manifest, indent=2, ensure_ascii=False) + "\n").encode("utf-8"))
    print(f"GENERATE: OK class={args.cls} files={len(entries)} bytes={manifest['totals']['bytes']} out={out}")
    return 0


# --------------------------------------------------------------------------------------- verify


def cmd_verify(args: argparse.Namespace) -> int:
    root = Path(args.root)
    manifest = load_manifest(Path(args.manifest))
    pattern = CLASSES[manifest["class"]]["pattern"]
    mismatched, missing, ok = [], [], 0
    shown = 0
    cap = None if args.all else MAX_LISTED

    def report(label: str, path: str) -> None:
        nonlocal shown
        if cap is None or shown < cap:
            print(f"{label} {path}")
        shown += 1

    for e in manifest["entries"]:
        target = safe_target(root, e["path"], pattern)  # type: ignore[arg-type]
        if not target.exists():
            missing.append(e["path"])
            report("MISSING", e["path"])
            continue
        data = target.read_bytes() if target.is_file() else b""
        if not target.is_file() or len(data) != e["size"] or sha256_hex(data) != e["sha256"]:
            mismatched.append(e["path"])
            report("MISMATCH", e["path"])
        else:
            ok += 1
    listed = {e["path"] for e in manifest["entries"]}
    unlisted = sorted(p for p in tracked_index(root) if pattern.match(p) and p not in listed)  # type: ignore[union-attr]
    for p in unlisted:
        report("UNLISTED", p)
    if cap is not None and shown > cap:
        print(f"... {shown - cap} more line(s) not shown (use --all)")
    if mismatched or (args.strict and unlisted):
        print(f"VERIFY: FAIL ok={ok} mismatched={len(mismatched)} missing={len(missing)} unlisted={len(unlisted)}")
        return EXIT_FAIL
    if missing and not args.allow_missing:
        print(f"VERIFY: FAIL ok={ok} missing={len(missing)} (use restore, or --allow-missing for a migrated tree)")
        return EXIT_MISSING
    tail = f" missing={len(missing)} (allowed)" if missing else ""
    print(f"VERIFY: OK files={len(manifest['entries'])} ok={ok}{tail} unlisted={len(unlisted)}")
    return 0


# ------------------------------------------------------------------------------ backup / restore


def cmd_backup(args: argparse.Namespace) -> int:
    root, dest = Path(args.root), Path(args.dest)
    manifest = load_manifest(Path(args.manifest))
    pattern = CLASSES[manifest["class"]]["pattern"]
    written = kept = 0
    for e in manifest["entries"]:
        target = safe_target(root, e["path"], pattern)  # type: ignore[arg-type]
        data: Optional[bytes] = None
        if target.is_file():
            cand = target.read_bytes()
            if len(cand) == e["size"] and sha256_hex(cand) == e["sha256"]:
                data = cand
        if data is None:  # working copy absent or different: fall back to the recorded git blob
            data = read_entry_source(root, e, None, True)
        obj = backup_object(dest, e["sha256"])
        if obj.is_file() and obj.read_bytes() == data:
            kept += 1
            continue
        atomic_write(obj, data)
        written += 1
    meta_path = dest / "BACKUP.json"
    meta = json.loads(meta_path.read_text()) if meta_path.is_file() else {"schema_version": SCHEMA_VERSION, "classes": {}}
    meta["classes"][manifest["class"]] = {
        "updated_at": utc_now(), "files": len(manifest["entries"]),
        "bytes": manifest["totals"]["bytes"] if "totals" in manifest else None,
        "manifest_sha256": sha256_hex(Path(args.manifest).read_bytes()),
    }
    atomic_write(meta_path, (json.dumps(meta, indent=2) + "\n").encode())
    print(f"BACKUP: OK class={manifest['class']} files={len(manifest['entries'])} written={written} unchanged={kept} dest={dest}")
    return 0


def check_backup(manifest: dict, backup: Path) -> Tuple[int, int]:
    bad = good = 0
    for e in manifest["entries"]:
        obj = backup_object(backup, e["sha256"])
        if not obj.is_file():
            print(f"ABSENT {e['path']}")
            bad += 1
        elif obj.stat().st_size != e["size"] or sha256_hex(obj.read_bytes()) != e["sha256"]:
            print(f"CORRUPT {e['path']}")
            bad += 1
        else:
            good += 1
    return good, bad


def cmd_verify_backup(args: argparse.Namespace) -> int:
    manifest = load_manifest(Path(args.manifest))
    good, bad = check_backup(manifest, Path(args.backup))
    print(f"VERIFY-BACKUP: {'FAIL' if bad else 'OK'} files={len(manifest['entries'])} ok={good} bad={bad}")
    return EXIT_FAIL if bad else 0


def cmd_restore(args: argparse.Namespace) -> int:
    if bool(args.backup) == bool(args.from_git):
        raise Usage("restore needs exactly one source: --backup DIR or --from-git")
    root = Path(args.root)
    manifest = load_manifest(Path(args.manifest))
    pattern = CLASSES[manifest["class"]]["pattern"]
    backup = Path(args.backup) if args.backup else None
    restored = present = 0
    failures: List[str] = []
    for e in manifest["entries"]:
        target = safe_target(root, e["path"], pattern)  # type: ignore[arg-type]
        if target.is_file():
            cur = target.read_bytes()
            if len(cur) == e["size"] and sha256_hex(cur) == e["sha256"]:
                present += 1
                continue
            if not args.force_overwrite:
                failures.append(e["path"])
                print(f"CONFLICT {e['path']}: a different file exists; refusing to overwrite (use --force-overwrite)")
                continue
        elif target.exists():
            failures.append(e["path"])
            print(f"CONFLICT {e['path']}: not a regular file")
            continue
        try:
            data = read_entry_source(root, e, backup, args.from_git)
        except Failure as exc:
            failures.append(e["path"])
            print(f"FAILED {exc}")
            continue
        atomic_write(target, data)
        restored += 1
    if failures:
        print(f"RESTORE: FAIL restored={restored} already present={present} failed={len(failures)}")
        return EXIT_FAIL
    print(f"RESTORE: OK restored={restored} already present={present}")
    return 0


# ---------------------------------------------------------------------------------------- untrack


def cmd_untrack(args: argparse.Namespace) -> int:
    root = Path(args.root)
    manifest = load_manifest(Path(args.manifest))
    cdef = CLASSES[manifest["class"]]
    pattern = cdef["pattern"]
    good, bad = check_backup(manifest, Path(args.backup))
    if bad:
        raise Failure(f"backup is not complete and verified (bad={bad}); refusing to untrack")
    index = tracked_index(root)
    todo = []
    for e in manifest["entries"]:
        if e["path"] not in index:
            continue
        target = safe_target(root, e["path"], pattern)  # type: ignore[arg-type]
        data = target.read_bytes() if target.is_file() else None
        if data is None or sha256_hex(data) != e["sha256"] or git_blob_id(data) != index[e["path"]]:
            raise Failure(f"{e['path']}: working copy does not match the manifest and index; refusing to untrack")
        todo.append(e["path"])
    rule = str(cdef["ignore_rule"])
    gi = root / ".gitignore"
    has_rule = gi.is_file() and rule in gi.read_text(encoding="utf-8").splitlines()
    if not args.apply:
        print(f"DRY RUN: would `git rm --cached` {len(todo)} file(s) of class {manifest['class']} "
              f"and {'keep' if has_rule else 'append'} the ignore rule {rule!r}; backup verified ({good} objects). "
              "Re-run with --apply. No file is deleted and no commit is created either way.")
        return 0
    for i in range(0, len(todo), 50):
        git(root, "rm", "--cached", "-q", "--", *todo[i:i + 50])
    if not has_rule:
        text = gi.read_text(encoding="utf-8") if gi.is_file() else ""
        sep = "" if (not text or text.endswith("\n")) else "\n"
        atomic_write(gi, (text + sep + f"\n# O2: {cdef['description']} (restore with retention-manifest.py)\n{rule}\n").encode())
    print(f"UNTRACK: OK class={manifest['class']} untracked={len(todo)} ignore_rule={rule!r} "
          "(files kept on disk; nothing committed; history and object store untouched)")
    return 0


# -------------------------------------------------------------------------------------- inventory


def count_objects(root: Path) -> Tuple[str, Dict[str, str]]:
    human = git(root, "count-objects", "-vH", check=False).decode()
    raw: Dict[str, str] = {}
    for line in git(root, "count-objects", "-v", check=False).decode().splitlines():
        k, _, v = line.partition(": ")
        raw[k] = v
    return human, raw


def history_blobs(root: Path) -> List[Tuple[str, int, int, str]]:
    """(oid, size, size_on_disk, first path) for every blob reachable from any ref, deduplicated."""
    rev = subprocess.run(["git", "-C", str(root), "rev-list", "--objects", "--all"], env=git_env(),
                         capture_output=True, timeout=300)
    if rev.returncode != 0:
        return []
    inp = rev.stdout
    chk = subprocess.run(
        ["git", "-C", str(root), "cat-file", "--batch-check=%(objecttype) %(objectname) %(objectsize) %(objectsize:disk) %(rest)"],
        env=git_env(), capture_output=True, input=inp, timeout=300)
    blobs: Dict[str, Tuple[str, int, int, str]] = {}
    for line in chk.stdout.decode("utf-8", "replace").splitlines():
        parts = line.split(" ", 4)
        if len(parts) >= 4 and parts[0] == "blob" and parts[1] not in blobs:
            blobs[parts[1]] = (parts[1], int(parts[2]), int(parts[3]), parts[4] if len(parts) > 4 else "")
    return list(blobs.values())


def du_kib(path: Path) -> Optional[int]:
    proc = subprocess.run(["du", "-sk", str(path)], capture_output=True, text=True)
    try:
        return int(proc.stdout.split()[0])
    except (IndexError, ValueError):
        return None


def build_inventory(root: Path, ignored_sizes: bool) -> dict:
    now = utc_now()
    head = git(root, "rev-parse", "HEAD", check=False).decode().strip()
    branch = git(root, "rev-parse", "--abbrev-ref", "HEAD", check=False).decode().strip()
    tree = git(root, "ls-tree", "-r", "-l", "-z", "HEAD", check=False)
    tracked: List[Tuple[str, int]] = []
    for rec in tree.split(b"\0"):
        if not rec:
            continue
        meta, _, path = rec.partition(b"\t")
        size = meta.split()[-1]
        tracked.append((path.decode("utf-8", "surrogateescape"), int(size) if size.isdigit() else 0))
    classes = []
    for name, rx in INVENTORY_CLASSES:
        sel = [(p, s) for p, s in tracked if rx.search(p)]
        classes.append({"class": name, "files": len(sel), "bytes_at_head": sum(s for _, s in sel)})
    blobs = history_blobs(root)
    for c, (_, rx) in zip(classes, INVENTORY_CLASSES):
        sel_b = [b for b in blobs if rx.search(b[3])]
        c["history_distinct_blobs"] = len(sel_b)
        c["history_bytes_uncompressed"] = sum(b[1] for b in sel_b)
        c["history_bytes_on_disk"] = sum(b[2] for b in sel_b)
    human, raw = count_objects(root)
    refs = git(root, "for-each-ref", "--format=%(refname)", check=False).decode().split()
    ref_counts = {
        "heads": sum(r.startswith("refs/heads/") for r in refs),
        "remotes": sum(r.startswith("refs/remotes/") for r in refs),
        "tags": sum(r.startswith("refs/tags/") for r in refs),
        "other": sum(not r.startswith(("refs/heads/", "refs/remotes/", "refs/tags/")) for r in refs),
    }
    wts = []
    cur: Dict[str, str] = {}
    for line in git(root, "worktree", "list", "--porcelain", check=False).decode().splitlines() + [""]:
        if not line:
            if cur:
                wts.append({"path": cur.get("worktree", ""), "head": cur.get("HEAD", "")[:12],
                            "branch": cur.get("branch", "(detached)").replace("refs/heads/", "")})
            cur = {}
            continue
        k, _, v = line.partition(" ")
        cur[k] = v
    reflog = len(git(root, "reflog", "show", "--all", check=False).decode().splitlines())
    stash = len(git(root, "stash", "list", check=False).decode().splitlines())
    ignored = []
    for rel in git(root, "ls-files", "--others", "--ignored", "--exclude-standard", "--directory", check=False).decode().splitlines():
        item: Dict[str, object] = {"path": rel}
        if ignored_sizes:
            item["kib"] = du_kib(root / rel)
        item["log_like"] = bool(re.search(r"(\.log$|(^|/)logs?/)", rel))
        ignored.append(item)
    largest_head = sorted(tracked, key=lambda t: -t[1])[:8]
    largest_hist = sorted(blobs, key=lambda b: -b[1])[:8]
    raw_cls = CLASSES["review-raw"]["pattern"]
    flagged = []
    mtimes = []
    for p, _ in tracked:
        if raw_cls.match(p):  # type: ignore[union-attr]
            f = root / p
            if f.is_file():
                mtimes.append(f.stat().st_mtime)
                flags = scan_sensitivity(f.read_bytes())
                if flags:
                    flagged.append({"path": p, "flags": flags})
    return {
        "generated_at": now, "head": head, "branch": branch,
        "tracked_total": {"files": len(tracked), "bytes_at_head": sum(s for _, s in tracked)},
        "classes": classes, "count_objects_human": human, "count_objects": raw,
        "refs": ref_counts, "worktrees": wts, "reflog_entries_all": reflog, "stash_entries": stash,
        "ignored_on_disk": ignored, "largest_tracked_at_head": [{"path": p, "bytes": s} for p, s in largest_head],
        "largest_blobs_in_history": [{"path": b[3], "bytes": b[1], "bytes_on_disk": b[2]} for b in largest_hist],
        "review_raw_sensitivity": flagged,
        "review_raw_mtime": {
            "oldest": datetime.fromtimestamp(min(mtimes), timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ") if mtimes else None,
            "newest": datetime.fromtimestamp(max(mtimes), timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ") if mtimes else None,
        },
    }


def render_inventory(inv: dict) -> str:
    L: List[str] = []
    L.append(f"INVENTORY (read-only) generated_at={inv['generated_at']} head={inv['head'][:12]} branch={inv['branch']}")
    t = inv["tracked_total"]
    L.append(f"\n## Tracked at HEAD: {t['files']} files, {t['bytes_at_head']} bytes (uncompressed)")
    L.append("class | files | bytes at HEAD | history: distinct blobs | uncompressed bytes | bytes on disk (packed)")
    for c in inv["classes"]:
        L.append(f"{c['class']} | {c['files']} | {c['bytes_at_head']} | {c['history_distinct_blobs']} | "
                 f"{c['history_bytes_uncompressed']} | {c['history_bytes_on_disk']}")
    L.append("\n## Git object store (git count-objects -vH)")
    L.append(inv["count_objects_human"].rstrip())
    r = inv["refs"]
    L.append(f"\n## Refs (shared across worktrees) heads={r['heads']} remotes={r['remotes']} tags={r['tags']} other={r['other']} "
             f"reflog_entries={inv['reflog_entries_all']} stash_entries={inv['stash_entries']}")
    for w in inv["worktrees"]:
        L.append(f"worktree {w['path']} head={w['head']} branch={w['branch']}")
    L.append("\n## Ignored files on disk (logs and build output; never tracked)")
    for i in inv["ignored_on_disk"]:
        kib = f" {i['kib']} KiB" if i.get("kib") is not None else ""
        L.append(f"{i['path']}{kib}{' [log-like]' if i['log_like'] else ''}")
    L.append("\n## Largest tracked files at HEAD")
    L.extend(f"{x['bytes']} {x['path']}" for x in inv["largest_tracked_at_head"])
    L.append("\n## Largest blobs anywhere in history (path as first seen; bytes uncompressed / on disk)")
    L.extend(f"{x['bytes']} / {x['bytes_on_disk']} {x['path']}" for x in inv["largest_blobs_in_history"])
    L.append(f"\n## review-raw working-copy mtimes oldest={inv['review_raw_mtime']['oldest']} newest={inv['review_raw_mtime']['newest']}")
    L.append("## review-raw sensitivity (flag names only; content is never printed)")
    if not inv["review_raw_sensitivity"]:
        L.append("none flagged")
    for f in inv["review_raw_sensitivity"]:
        L.append(f"sensitive: yes, path only: {f['path']} flags={','.join(f['flags'])}")
    return "\n".join(L) + "\n"


def cmd_inventory(args: argparse.Namespace) -> int:
    inv = build_inventory(Path(args.root), ignored_sizes=not args.no_ignored_sizes)
    sys.stdout.write(json.dumps(inv, indent=2) + "\n" if args.format == "json" else render_inventory(inv))
    return 0


# ------------------------------------------------------------------------------------------ main


def build_parser() -> argparse.ArgumentParser:
    ap = argparse.ArgumentParser(prog="retention-manifest.py", description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--root", default=str(DEFAULT_ROOT), help="repository root (default: this repository)")
    sub = ap.add_subparsers(dest="cmd", required=True)
    p = sub.add_parser("inventory", help="read-only snapshot")
    p.add_argument("--format", choices=("text", "json"), default="text")
    p.add_argument("--no-ignored-sizes", action="store_true", help="skip du on ignored paths (fast)")
    p.set_defaults(fn=cmd_inventory)
    p = sub.add_parser("generate", help="write a manifest for one class")
    p.add_argument("--class", dest="cls", required=True)
    p.add_argument("--out", required=True)
    p.set_defaults(fn=cmd_generate)
    p = sub.add_parser("verify", help="compare the tree with a manifest")
    p.add_argument("--manifest", required=True)
    p.add_argument("--allow-missing", action="store_true", help="treat absent files as OK (migrated tree); still listed")
    p.add_argument("--strict", action="store_true", help="also fail on tracked class files absent from the manifest")
    p.add_argument("--all", action="store_true", help="list every problem path instead of the first few")
    p.set_defaults(fn=cmd_verify)
    p = sub.add_parser("backup", help="copy the class into a content-addressed store")
    p.add_argument("--manifest", required=True)
    p.add_argument("--dest", required=True)
    p.set_defaults(fn=cmd_backup)
    p = sub.add_parser("verify-backup", help="check a backup against a manifest")
    p.add_argument("--manifest", required=True)
    p.add_argument("--backup", required=True)
    p.set_defaults(fn=cmd_verify_backup)
    p = sub.add_parser("restore", help="recreate missing files from a backup or from git history")
    p.add_argument("--manifest", required=True)
    p.add_argument("--backup", help="content-addressed backup directory")
    p.add_argument("--from-git", action="store_true", help="read the recorded blob ids from the object store")
    p.add_argument("--force-overwrite", action="store_true", help="replace a differing file (default: refuse)")
    p.set_defaults(fn=cmd_restore)
    p = sub.add_parser("untrack", help="stop tracking the class (dry run unless --apply; needs a verified backup)")
    p.add_argument("--manifest", required=True)
    p.add_argument("--backup", required=True)
    p.add_argument("--apply", action="store_true")
    p.set_defaults(fn=cmd_untrack)
    return ap


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        return args.fn(args)
    except Usage as exc:
        print(f"usage error: {exc}", file=sys.stderr)
        return EXIT_USAGE
    except Failure as exc:
        print(f"FAILED: {exc}", file=sys.stderr)
        return EXIT_FAIL


if __name__ == "__main__":
    sys.exit(main())

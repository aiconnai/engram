#!/usr/bin/env python3
"""Neutralized git access for the trusted runner (task H4). Stdlib only; no shell.

Shared by check-scope.py, run-task.py and record-evidence.py. It follows the review-gate.sh
(task H1) approach: writer-reachable git state must not change what the supervisor sees.

  * Every call is `git -C <repo> ...` with an argv list, never a shell string.
  * The environment is rebuilt from scratch: no inherited GIT_* variable (GIT_DIR, GIT_INDEX_FILE,
    GIT_OBJECT_DIRECTORY, ...) can redirect git; system and global config are disabled
    (GIT_CONFIG_NOSYSTEM, GIT_CONFIG_GLOBAL=/dev/null); replace refs and grafts are ignored
    (--no-replace-objects, GIT_NO_REPLACE_OBJECTS, GIT_GRAFT_FILE=/dev/null); system attributes and
    the attributes file are disabled; hooks are pointed at /dev/null (update-ref runs the
    reference-transaction hook); pathspecs are literal.
  * Workspaces are exported and re-hashed with plumbing only (ls-tree + cat-file --batch,
    hash-object --no-filters + update-index --index-info + write-tree), so in-tree .gitattributes,
    .gitignore, eol conversion and filter drivers never alter the bytes that are judged.
"""

from __future__ import annotations

import os
import stat
import subprocess
import time
from pathlib import Path
from typing import Dict, Iterable, List, Optional, Sequence, Tuple

ZERO_OID = "0" * 40
GIT_CONFIG_ARGS = (
    "--no-replace-objects",
    "-c", "core.attributesFile=/dev/null",
    "-c", "core.hooksPath=/dev/null",
    "-c", "core.fsmonitor=false",
    "-c", "core.quotepath=true",
    "-c", "core.autocrlf=false",
    "-c", "core.safecrlf=false",
    "-c", "core.symlinks=true",
    "-c", "advice.graftFileDeprecated=false",
    "-c", "commit.gpgSign=false",
)
KEEP_ENV = ("PATH", "HOME", "TMPDIR")
MODE_FILE, MODE_EXEC, MODE_LINK, MODE_GITLINK = b"100644", b"100755", b"120000", b"160000"
# Host-side bounds of export / re-hash so the wall budget also bounds supervisor work on writer output.
MAX_WORKSPACE_FILES = 100_000
MAX_WORKSPACE_BYTES = 2 * 1024 ** 3


class GitError(Exception):
    """A git command failed (non-zero exit, timeout or unusable output)."""


class WorkspaceError(Exception):
    """A workspace or tree cannot be represented safely. `code` is a stable reason string."""

    def __init__(self, code: str, path: bytes, detail: str = ""):
        super().__init__(f"{code}: {path!r} {detail}".strip())
        self.code, self.path, self.detail = code, path, detail


def _check_deadline(deadline: Optional[float]) -> None:
    if deadline is not None and time.monotonic() >= deadline:
        raise WorkspaceError("deadline_exceeded", b"", "wall budget exhausted during host-side export / hashing")


def _timeout(deadline: Optional[float], default: float = 300) -> float:
    return default if deadline is None else max(1.0, min(default, deadline - time.monotonic()))


def git_env(index_file: Optional[Path] = None, extra: Optional[Dict[str, str]] = None) -> Dict[str, str]:
    env = {k: os.environ[k] for k in KEEP_ENV if k in os.environ}
    env.update({
        "LC_ALL": "C",
        "GIT_CONFIG_NOSYSTEM": "1",
        "GIT_CONFIG_GLOBAL": os.devnull,
        "GIT_NO_REPLACE_OBJECTS": "1",
        "GIT_GRAFT_FILE": os.devnull,
        "GIT_ATTR_NOSYSTEM": "1",
        "GIT_TERMINAL_PROMPT": "0",
        "GIT_OPTIONAL_LOCKS": "0",
        "GIT_LITERAL_PATHSPECS": "1",
    })
    if index_file is not None:
        env["GIT_INDEX_FILE"] = str(index_file)
    env.update(extra or {})
    return env


def git(repo: Path, args: Sequence[str], *, input_bytes: Optional[bytes] = None, index_file: Optional[Path] = None,
        extra_env: Optional[Dict[str, str]] = None, timeout: float = 300) -> bytes:
    """Run one git command; return stdout bytes. Raises GitError on any failure."""
    argv = ["git", "-C", str(repo), *GIT_CONFIG_ARGS, *args]
    try:
        proc = subprocess.run(argv, input=input_bytes, capture_output=True, timeout=timeout, check=False,
                              env=git_env(index_file, extra_env), stdin=None if input_bytes is not None else subprocess.DEVNULL)
    except (OSError, subprocess.TimeoutExpired) as exc:
        raise GitError(f"git {args[0] if args else ''} failed: {type(exc).__name__}") from exc
    if proc.returncode != 0:
        tail = " ".join(proc.stderr.decode("utf-8", "replace").split())[-300:]
        raise GitError(f"git {args[0] if args else ''} exited {proc.returncode}: {tail}")
    return proc.stdout


def resolve_commit(repo: Path, rev: str) -> Optional[str]:
    """Full commit SHA of `rev`, or None when it does not name a commit in this repository."""
    if not rev or rev.startswith("-") or "\0" in rev:
        return None
    try:
        out = git(repo, ["rev-parse", "--verify", "--quiet", "--end-of-options", f"{rev}^{{commit}}"])
    except GitError:
        return None
    sha = out.decode("ascii", "replace").strip()
    return sha if len(sha) == 40 else None


def tree_of(repo: Path, commit: str) -> str:
    return git(repo, ["rev-parse", "--verify", "--end-of-options", f"{commit}^{{tree}}"]).decode().strip()


def parents_of(repo: Path, commit: str) -> List[str]:
    line = git(repo, ["rev-list", "--parents", "-n", "1", "--end-of-options", commit]).decode().split()
    return line[1:]


def git_common_dir(repo: Path) -> Path:
    out = git(repo, ["rev-parse", "--path-format=absolute", "--git-common-dir"]).decode().strip()
    return Path(out).resolve()


def toplevel(repo: Path) -> Optional[Path]:
    try:
        out = git(repo, ["rev-parse", "--show-toplevel"]).decode().strip()
    except GitError:
        return None
    return Path(out).resolve() if out else None


def is_within(child: Path, parent: Path) -> bool:
    child, parent = Path(os.path.realpath(child)), Path(os.path.realpath(parent))
    return child == parent or parent in child.parents


def untrusted_roots(repo: Path) -> List[Path]:
    """Every worktree of the repository plus its git dirs: writer-reachable or writer-adjacent places."""
    roots: List[Path] = []
    top = toplevel(repo)
    if top is not None:
        roots.append(top)
    for flag in ("--git-common-dir", "--git-dir"):
        try:
            out = git(repo, ["rev-parse", flag]).decode().strip()
        except GitError:
            continue
        path = Path(out)
        roots.append((Path(repo) / path).resolve() if not path.is_absolute() else path.resolve())
    try:
        listing = git(repo, ["worktree", "list", "--porcelain", "-z"])
    except GitError:
        listing = b""
    for record in listing.split(b"\0"):
        if record.startswith(b"worktree "):
            roots.append(Path(os.fsdecode(record[len(b"worktree "):])).resolve())
    return roots


def ref_target(repo: Path, ref: str) -> Optional[str]:
    try:
        out = git(repo, ["rev-parse", "--verify", "--quiet", "--end-of-options", f"{ref}^{{commit}}"])
    except GitError:
        return None
    return out.decode().strip() or None


# --- byte-exact export of a tree into a plain directory -------------------------------------------

def _safe_components(path: bytes) -> List[bytes]:
    parts = path.split(b"/")
    for part in parts:
        if part in (b"", b".", b"..") or part.lower() == b".git":
            raise WorkspaceError("unsafe_tree_path", path)
    return parts


def ls_tree(repo: Path, treeish: str) -> List[Tuple[bytes, str, bytes]]:
    """(mode, object sha, path) for every blob/link/gitlink in the tree, recursively."""
    out = git(repo, ["ls-tree", "-r", "-z", "--full-tree", "--end-of-options", treeish])
    entries = []
    for record in out.split(b"\0"):
        if not record:
            continue
        meta, _, path = record.partition(b"\t")
        mode, _kind, sha = meta.split(b" ")
        entries.append((mode, sha.decode("ascii"), path))
    return entries


class _CatFile:
    """One `git cat-file --batch` process; request/response one object at a time (no pipe deadlock)."""

    def __init__(self, repo: Path):
        self.proc = subprocess.Popen(["git", "-C", str(repo), *GIT_CONFIG_ARGS, "cat-file", "--batch"],
                                     stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, env=git_env())

    def read(self, sha: str) -> bytes:
        assert self.proc.stdin is not None and self.proc.stdout is not None
        self.proc.stdin.write(sha.encode("ascii") + b"\n")
        self.proc.stdin.flush()
        header = self.proc.stdout.readline().split()
        if len(header) != 3 or header[1] != b"blob":
            raise GitError(f"cat-file returned {header!r} for {sha}")
        size = int(header[2])
        data = self.proc.stdout.read(size)
        self.proc.stdout.read(1)
        if len(data) != size:
            raise GitError(f"short read for {sha}")
        return data

    def close(self) -> None:
        for stream in (self.proc.stdin, self.proc.stdout):
            if stream:
                stream.close()
        try:
            self.proc.wait(timeout=30)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            self.proc.wait(timeout=30)


def export_tree(repo: Path, treeish: str, dest: Path, deadline: Optional[float] = None) -> int:
    """Write the exact bytes of `treeish` into the empty directory `dest`. Returns the entry count.

    Gitlinks (submodules) are refused: a plain directory cannot carry them faithfully. `deadline`
    (time.monotonic()) bounds the export: past it -> WorkspaceError("deadline_exceeded").
    """
    _check_deadline(deadline)
    dest_b = os.fsencode(str(dest))
    entries = ls_tree(repo, treeish)
    for mode, _sha, path in entries:
        if mode == MODE_GITLINK:
            raise WorkspaceError("submodule_in_tree", path, "trees with submodules are not supported by the runner")
        _safe_components(path)
    cat = _CatFile(repo)
    try:
        for mode, sha, path in entries:
            _check_deadline(deadline)
            data = cat.read(sha)
            target = os.path.join(dest_b, path)
            os.makedirs(os.path.dirname(target), mode=0o755, exist_ok=True)
            if mode == MODE_LINK:
                os.symlink(data, target)
                continue
            fd = os.open(target, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o755 if mode == MODE_EXEC else 0o644)
            with os.fdopen(fd, "wb") as handle:
                handle.write(data)
    finally:
        cat.close()
    return len(entries)


# --- re-hash a plain directory into a tree object (no filters, no ignore rules) -------------------

def scan_workspace(root: Path, max_files: int = MAX_WORKSPACE_FILES, max_bytes: int = MAX_WORKSPACE_BYTES,
                   deadline: Optional[float] = None) -> List[Tuple[bytes, bytes, bytes]]:
    """(mode, relative path, absolute path) of every file and symlink under root, sorted by path.

    Directories named .git (any case) and special files (fifo, socket, device) are refused, and so is
    a workspace above max_files entries or max_bytes of regular-file content ("workspace_too_large").
    """
    root_b = os.fsencode(str(root))
    found: List[Tuple[bytes, bytes, bytes]] = []
    total = 0
    for dirpath, dirnames, filenames in os.walk(root_b, followlinks=False):
        _check_deadline(deadline)
        for name in list(dirnames):
            full = os.path.join(dirpath, name)
            rel = os.path.relpath(full, root_b)
            if name.lower() == b".git":
                raise WorkspaceError("git_metadata_in_workspace", rel, "a .git entry cannot be part of a candidate")
            if stat.S_ISLNK(os.lstat(full).st_mode):
                dirnames.remove(name)
                found.append((MODE_LINK, rel, full))
        for name in filenames:
            full = os.path.join(dirpath, name)
            rel = os.path.relpath(full, root_b)
            if name.lower() == b".git":
                raise WorkspaceError("git_metadata_in_workspace", rel, "a .git entry cannot be part of a candidate")
            info = os.lstat(full)
            if stat.S_ISLNK(info.st_mode):
                found.append((MODE_LINK, rel, full))
            elif stat.S_ISREG(info.st_mode):
                found.append((MODE_EXEC if info.st_mode & stat.S_IXUSR else MODE_FILE, rel, full))
                total += info.st_size
            else:
                raise WorkspaceError("special_file_in_workspace", rel, "only regular files and symlinks are allowed")
            if len(found) > max_files or total > max_bytes:
                raise WorkspaceError("workspace_too_large", rel,
                                     f"more than {max_files} files or {max_bytes} bytes in the workspace")
    return sorted(found, key=lambda e: e[1])


def _hash_blobs(repo: Path, entries: Iterable[Tuple[bytes, bytes, bytes]], deadline: Optional[float] = None) -> Dict[bytes, str]:
    shas: Dict[bytes, str] = {}
    batch = [e for e in entries if e[0] != MODE_LINK and b"\n" not in e[2]]
    if batch:
        out = git(repo, ["hash-object", "-w", "--no-filters", "--stdin-paths"], timeout=_timeout(deadline),
                  input_bytes=b"".join(e[2] + b"\n" for e in batch)).decode().split()
        if len(out) != len(batch):
            raise GitError("hash-object returned an unexpected number of object ids")
        shas.update({e[1]: sha for e, sha in zip(batch, out)})
    for mode, rel, full in entries:
        if rel in shas:
            continue
        _check_deadline(deadline)
        data = os.readlink(full) if mode == MODE_LINK else Path(os.fsdecode(full)).read_bytes()
        shas[rel] = git(repo, ["hash-object", "-w", "--no-filters", "--stdin"], input_bytes=data).decode().strip()
    return shas


def build_tree(repo: Path, root: Path, scratch: Path, *, max_files: int = MAX_WORKSPACE_FILES,
               max_bytes: int = MAX_WORKSPACE_BYTES, deadline: Optional[float] = None) -> str:
    """Hash every file of `root` into the object store and return the resulting tree SHA (bounded)."""
    entries = scan_workspace(root, max_files, max_bytes, deadline)
    shas = _hash_blobs(repo, entries, deadline)
    _check_deadline(deadline)
    index_file = Path(scratch) / f"index-{os.getpid()}-{os.urandom(4).hex()}"
    try:
        info = b"".join(mode + b" " + shas[rel].encode() + b"\t" + rel + b"\0" for mode, rel, _ in entries)
        if info:
            git(repo, ["update-index", "-z", "--add", "--index-info"], input_bytes=info, index_file=index_file)
        else:
            git(repo, ["read-tree", "--empty"], index_file=index_file)
        return git(repo, ["write-tree"], index_file=index_file).decode().strip()
    finally:
        for leftover in (index_file, Path(str(index_file) + ".lock")):
            try:
                leftover.unlink()
            except FileNotFoundError:
                pass

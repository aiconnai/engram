"""Checkout verification: the evidence is only valid for the exact candidate."""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Dict

from .common import SHA_RE, CandidateError


def _git(repo: Path, *args: str) -> str:
    try:
        completed = subprocess.run(
            ["git", "-C", str(repo), *args],
            capture_output=True,
            text=True,
            check=True,
            shell=False,
        )
    except (OSError, subprocess.CalledProcessError) as exc:
        detail = getattr(exc, "stderr", "") or str(exc)
        raise CandidateError(f"git {' '.join(args)} failed: {detail.strip()}") from exc
    return completed.stdout


def verify_checkout(repo: Path, candidate_sha: str) -> Dict[str, object]:
    """Require HEAD == candidate_sha and a clean tree; return sha, tree, clean."""
    if not SHA_RE.fullmatch(candidate_sha):
        raise CandidateError("--candidate-sha must be a full 40-character lowercase Git SHA")
    head = _git(repo, "rev-parse", "HEAD").strip()
    if head != candidate_sha:
        raise CandidateError(
            f"candidate mismatch: checkout HEAD is {head}, requested {candidate_sha}"
        )
    tree = _git(repo, "rev-parse", "HEAD^{tree}").strip()
    dirty = _git(repo, "status", "--porcelain=v1", "--untracked-files=all").strip()
    if dirty:
        first = dirty.splitlines()[0]
        raise CandidateError(
            f"checkout is not clean ({len(dirty.splitlines())} change(s), first: {first!r}); "
            "evidence must describe the committed candidate"
        )
    return {"sha": head, "tree": tree, "clean": True}


def is_tracked(repo: Path, relative: str) -> bool:
    try:
        subprocess.run(
            ["git", "-C", str(repo), "ls-files", "--error-unmatch", "--", relative],
            capture_output=True,
            check=True,
            shell=False,
        )
    except subprocess.CalledProcessError:
        return False
    except OSError as exc:
        raise CandidateError(f"git ls-files failed: {exc}") from exc
    return True


def show_at(repo: Path, revision: str, relative: str) -> str:
    if not SHA_RE.fullmatch(revision):
        raise CandidateError(f"anchor revision must be a 40-character SHA, got {revision!r}")
    return _git(repo, "show", f"{revision}:{relative}")


def relative_to_repo(repo: Path, path: Path, label: str) -> str:
    try:
        return path.resolve().relative_to(repo.resolve()).as_posix()
    except ValueError as exc:
        raise CandidateError(f"{label}: {path} is outside the candidate checkout") from exc


def is_ancestor(repo: Path, ancestor: str, descendant: str) -> bool:
    """True when ``ancestor`` is a strict-or-equal ancestor of ``descendant``.

    Unknown commits (shallow clones, wrong repository) count as "not an ancestor".
    """
    completed = subprocess.run(
        ["git", "-C", str(repo), "merge-base", "--is-ancestor", ancestor, descendant],
        capture_output=True,
        shell=False,
        check=False,
    )
    return completed.returncode == 0

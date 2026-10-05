#!/usr/bin/env python3
"""Retention manifest, backup and restore tests (task O2). Offline, stdlib only, hermetic.

Every destructive-looking step (untracking a class, absent working copies) runs in disposable
repositories/clones created under a temp directory; nothing here touches the real repository
except read-only checks of the committed manifest and policy document.

Covered:
  * manifest generation (sorted, sha256/size/git blob, deterministic, refuses dirty files);
  * sensitivity is reported by flag name only: the sentinel content is never printed or stored;
  * verify (ok / mismatch / missing / unlisted) and its exit codes;
  * content-addressed backup, tamper detection, restore from backup or from git history;
  * untrack is a dry run by default, requires a verified backup, never deletes files and never
    rewrites history;
  * the disposable-clone proof: after untracking in repo A and cloning it fresh (clone B has no
    versioned copy of the class), links resolve, the class restores byte-identically from the
    backup and from git history;
  * the committed real manifest verifies against the real tree and the policy document is
    complete (per-class owner/duration/storage/lookup/restore; destructive commands prohibited).
"""

from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent.parent
BIN = REPO_ROOT / "docs" / "harness" / "bin"
TOOL = BIN / "retention-manifest.py"
POLICY = REPO_ROOT / "docs" / "OPERATIONS_GIT_RETENTION.md"
REAL_MANIFEST = REPO_ROOT / "docs" / "harness" / "retention" / "review-raw.manifest.json"
sys.path.insert(0, str(BIN))

import doc_links  # noqa: E402

CLASS = "review-raw"
RAW_DIR = "docs/harness/reviews"
# Built from parts so this file itself never contains a secret-looking literal.
SENTINEL_SECRET = "gh" + "p_" + "A1b2C3d4E5f6G7h8I9j0K1l2M3n4O5p6Q7r8"
SENTINEL_HOME = "/Us" + "ers/someone/private/project"

GIT_ENV = {
    **{k: v for k, v in os.environ.items() if not k.startswith("GIT_")},
    "GIT_CONFIG_NOSYSTEM": "1",
    "GIT_CONFIG_GLOBAL": os.devnull,
    "GIT_AUTHOR_NAME": "t",
    "GIT_AUTHOR_EMAIL": "t@example.invalid",
    "GIT_COMMITTER_NAME": "t",
    "GIT_COMMITTER_EMAIL": "t@example.invalid",
}


def git(root: Path, *args: str, check: bool = True) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["git", "-c", "core.hooksPath=/dev/null", "-c", "commit.gpgSign=false", "-C", str(root), *args],
        env=GIT_ENV, capture_output=True, check=check,
    )


def tool(root: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        [sys.executable, str(TOOL), "--root", str(root), *args],
        env=GIT_ENV, capture_output=True, text=True, timeout=120,
    )


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def make_repo(parent: Path, name: str = "origin") -> tuple[Path, dict[str, bytes]]:
    """A synthetic repo with a few tracked .raw review prompts, two .md files and one doc."""
    root = parent / name
    root.mkdir()
    git(root, "init", "-q", "-b", "main")
    files = {
        f"{RAW_DIR}/2026-01-01-alpha-pre.md.raw": b"prompt alpha\nline 2\n",
        f"{RAW_DIR}/2026-01-02-beta-post.md.raw": b"prompt beta\n" + bytes(range(256)),
        f"{RAW_DIR}/2026-01-03-gamma.raw": b"",  # empty file is a legal member
    }
    for rel, data in files.items():
        p = root / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(data)
    (root / RAW_DIR / "2026-01-01-alpha-pre.md").write_text("# alpha verdict\nREVIEW_VERDICT: PASS\n")
    (root / "docs").mkdir(exist_ok=True)
    (root / "docs" / "index.md").write_text(
        "# Index\n\nSee [alpha](harness/reviews/2026-01-01-alpha-pre.md).\n"
    )
    (root / ".gitignore").write_text("target/\n")
    git(root, "add", "-A")
    git(root, "commit", "-q", "-m", "chore: seed")
    return root, files


class ToolCase(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="o2-"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.root, self.files = make_repo(self.tmp)

    def generate(self, root: Path | None = None, out: str = "m.json") -> Path:
        root = root or self.root
        outp = self.tmp / out
        r = tool(root, "generate", "--class", CLASS, "--out", str(outp))
        self.assertEqual(r.returncode, 0, r.stderr)
        return outp

    def backup(self, manifest: Path, dest: str = "bk") -> Path:
        d = self.tmp / dest
        r = tool(self.root, "backup", "--manifest", str(manifest), "--dest", str(d))
        self.assertEqual(r.returncode, 0, r.stderr)
        return d


class GenerateTests(ToolCase):
    def test_manifest_lists_exactly_the_tracked_class_sorted_with_hashes(self):
        m = json.loads(self.generate().read_text())
        self.assertEqual(m["class"], CLASS)
        self.assertEqual(m["schema_version"], 1)
        paths = [e["path"] for e in m["entries"]]
        self.assertEqual(paths, sorted(self.files))
        for e in m["entries"]:
            data = self.files[e["path"]]
            self.assertEqual(e["size"], len(data))
            self.assertEqual(e["sha256"], sha256(data))
            blob = hashlib.sha1(b"blob %d\0" % len(data) + data).hexdigest()
            self.assertEqual(e["git_blob"], blob)
            self.assertEqual(e["introduced_commit"], git(self.root, "rev-parse", "HEAD").stdout.decode().strip())
        self.assertNotIn("alpha-pre.md\"", json.dumps(paths))  # the verdict .md is not part of the class
        self.assertEqual(m["totals"], {"files": 3, "bytes": sum(len(d) for d in self.files.values())})

    def test_generation_is_deterministic_apart_from_the_timestamp(self):
        a = json.loads(self.generate(out="a.json").read_text())
        b = json.loads(self.generate(out="b.json").read_text())
        for m in (a, b):
            m.pop("generated_at")
        self.assertEqual(a, b)

    def test_refuses_a_file_that_differs_from_the_index(self):
        (self.root / next(iter(self.files))).write_bytes(b"edited after commit")
        r = tool(self.root, "generate", "--class", CLASS, "--out", str(self.tmp / "m.json"))
        self.assertEqual(r.returncode, 1)
        self.assertIn("differs from the index", r.stderr)
        self.assertFalse((self.tmp / "m.json").exists())

    def test_unknown_class_is_a_usage_error(self):
        r = tool(self.root, "generate", "--class", "nope", "--out", str(self.tmp / "m.json"))
        self.assertEqual(r.returncode, 2)


class SensitivityTests(ToolCase):
    def test_flags_by_name_only_and_never_prints_content(self):
        rel = f"{RAW_DIR}/2026-01-04-secret-post.md.raw"
        (self.root / rel).write_text(f"value {SENTINEL_SECRET}\ncwd {SENTINEL_HOME}\n")
        git(self.root, "add", rel)
        git(self.root, "commit", "-q", "-m", "chore: add sensitive-looking raw")
        out = self.generate()
        m = json.loads(out.read_text())
        entry = next(e for e in m["entries"] if e["path"] == rel)
        self.assertTrue(entry["sensitive"])
        self.assertEqual(sorted(entry["sensitive_flags"]), ["abs-home-path", "github-token"])
        clean = next(e for e in m["entries"] if e["path"].endswith("alpha-pre.md.raw"))
        self.assertFalse(clean["sensitive"])
        self.assertEqual(clean["sensitive_flags"], [])
        inv = tool(self.root, "inventory", "--no-ignored-sizes")
        self.assertEqual(inv.returncode, 0, inv.stderr)
        for blob in (out.read_text(), inv.stdout, inv.stderr):
            self.assertNotIn(SENTINEL_SECRET, blob)
            self.assertNotIn(SENTINEL_HOME, blob)
        self.assertIn(rel, inv.stdout)  # path only
        self.assertIn("sensitive: yes, path only", inv.stdout)


class VerifyTests(ToolCase):
    def test_ok_mismatch_missing_unlisted_and_exit_codes(self):
        mf = self.generate()
        r = tool(self.root, "verify", "--manifest", str(mf))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("VERIFY: OK", r.stdout)

        victim = self.root / sorted(self.files)[0]
        victim.write_bytes(victim.read_bytes() + b"x")
        r = tool(self.root, "verify", "--manifest", str(mf))
        self.assertEqual(r.returncode, 1)
        self.assertIn("MISMATCH", r.stdout)
        victim.write_bytes(self.files[sorted(self.files)[0]])

        gone = self.root / sorted(self.files)[1]
        gone.unlink()
        r = tool(self.root, "verify", "--manifest", str(mf))
        self.assertEqual(r.returncode, 4)
        self.assertIn("MISSING", r.stdout)
        r = tool(self.root, "verify", "--manifest", str(mf), "--allow-missing")
        self.assertEqual(r.returncode, 0)
        self.assertIn("MISSING", r.stdout)  # allowed, never silent

        extra = self.root / RAW_DIR / "2026-02-01-new.md.raw"
        extra.write_bytes(b"new")
        git(self.root, "add", str(extra.relative_to(self.root)))
        r = tool(self.root, "verify", "--manifest", str(mf), "--allow-missing")
        self.assertEqual(r.returncode, 0)
        self.assertIn("UNLISTED", r.stdout)
        r = tool(self.root, "verify", "--manifest", str(mf), "--allow-missing", "--strict")
        self.assertEqual(r.returncode, 1)

    def test_rejects_path_traversal_and_absolute_paths_in_a_manifest(self):
        mf = self.generate()
        for bad in ("../outside.raw", "/etc/passwd", f"{RAW_DIR}/../../x.raw"):
            m = json.loads(mf.read_text())
            m["entries"][0]["path"] = bad
            bad_mf = self.tmp / "bad.json"
            bad_mf.write_text(json.dumps(m))
            r = tool(self.root, "verify", "--manifest", str(bad_mf))
            self.assertEqual(r.returncode, 2, bad)
            self.assertIn("unsafe path", r.stderr)


class BackupRestoreTests(ToolCase):
    def test_backup_is_content_addressed_idempotent_and_verifiable(self):
        mf = self.generate()
        bk = self.backup(mf)
        for data in self.files.values():
            h = sha256(data)
            self.assertEqual((bk / "objects" / h[:2] / h).read_bytes(), data)
        self.assertTrue((bk / "BACKUP.json").is_file())
        again = tool(self.root, "backup", "--manifest", str(mf), "--dest", str(bk))
        self.assertEqual(again.returncode, 0, again.stderr)
        r = tool(self.root, "verify-backup", "--manifest", str(mf), "--backup", str(bk))
        self.assertEqual(r.returncode, 0, r.stderr)
        h = sha256(self.files[sorted(self.files)[1]])
        (bk / "objects" / h[:2] / h).write_bytes(b"corrupt")
        r = tool(self.root, "verify-backup", "--manifest", str(mf), "--backup", str(bk))
        self.assertEqual(r.returncode, 1)
        self.assertIn("CORRUPT", r.stdout)

    def test_restore_from_backup_is_byte_identical_and_skips_present_files(self):
        mf = self.generate()
        bk = self.backup(mf)
        for rel in self.files:
            (self.root / rel).unlink()
        r = tool(self.root, "restore", "--manifest", str(mf), "--backup", str(bk))
        self.assertEqual(r.returncode, 0, r.stderr)
        for rel, data in self.files.items():
            self.assertEqual((self.root / rel).read_bytes(), data)
        r = tool(self.root, "restore", "--manifest", str(mf), "--backup", str(bk))
        self.assertEqual(r.returncode, 0)
        self.assertIn("already present", r.stdout)

    def test_restore_never_writes_a_file_from_a_corrupt_source(self):
        mf = self.generate()
        bk = self.backup(mf)
        rel = sorted(self.files)[1]
        h = sha256(self.files[rel])
        (bk / "objects" / h[:2] / h).write_bytes(b"corrupt")
        (self.root / rel).unlink()
        r = tool(self.root, "restore", "--manifest", str(mf), "--backup", str(bk))
        self.assertEqual(r.returncode, 1)
        self.assertFalse((self.root / rel).exists())
        self.assertIn("hash mismatch", r.stdout + r.stderr)

    def test_restore_refuses_to_overwrite_a_different_file_without_force(self):
        mf = self.generate()
        bk = self.backup(mf)
        rel = sorted(self.files)[0]
        (self.root / rel).write_bytes(b"local edit")
        r = tool(self.root, "restore", "--manifest", str(mf), "--backup", str(bk))
        self.assertEqual(r.returncode, 1)
        self.assertEqual((self.root / rel).read_bytes(), b"local edit")
        r = tool(self.root, "restore", "--manifest", str(mf), "--backup", str(bk), "--force-overwrite")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual((self.root / rel).read_bytes(), self.files[rel])

    def test_restore_from_git_history_uses_the_recorded_blob(self):
        mf = self.generate()
        for rel in self.files:
            (self.root / rel).unlink()
        r = tool(self.root, "restore", "--manifest", str(mf), "--from-git")
        self.assertEqual(r.returncode, 0, r.stderr)
        for rel, data in self.files.items():
            self.assertEqual((self.root / rel).read_bytes(), data)

    def test_restore_needs_exactly_one_source(self):
        mf = self.generate()
        r = tool(self.root, "restore", "--manifest", str(mf))
        self.assertEqual(r.returncode, 2)


class UntrackTests(ToolCase):
    def tracked(self) -> set[str]:
        return set(git(self.root, "ls-files").stdout.decode().split("\n")) - {""}

    def test_dry_run_changes_nothing(self):
        mf = self.generate()
        bk = self.backup(mf)
        before = self.tracked()
        r = tool(self.root, "untrack", "--manifest", str(mf), "--backup", str(bk))
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("DRY RUN", r.stdout)
        self.assertEqual(self.tracked(), before)

    def test_requires_a_verified_backup(self):
        mf = self.generate()
        bk = self.tmp / "empty-backup"
        bk.mkdir()
        r = tool(self.root, "untrack", "--manifest", str(mf), "--backup", str(bk), "--apply")
        self.assertEqual(r.returncode, 1)
        self.assertEqual(self.tracked() & set(self.files), set(self.files))

    def test_apply_untracks_the_class_only_keeps_files_and_ignores_future_dumps(self):
        mf = self.generate()
        bk = self.backup(mf)
        head_before = git(self.root, "rev-parse", "HEAD").stdout
        r = tool(self.root, "untrack", "--manifest", str(mf), "--backup", str(bk), "--apply")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertEqual(self.tracked() & set(self.files), set())
        self.assertIn(f"{RAW_DIR}/2026-01-01-alpha-pre.md", self.tracked())  # verdicts stay versioned
        for rel, data in self.files.items():
            self.assertEqual((self.root / rel).read_bytes(), data)  # nothing deleted from disk
        self.assertEqual(git(self.root, "rev-parse", "HEAD").stdout, head_before)  # no history rewrite, no commit
        gi = (self.root / ".gitignore").read_text()
        self.assertEqual(gi.count(f"{RAW_DIR}/*.raw"), 1)
        ign = git(self.root, "check-ignore", "-q", f"{RAW_DIR}/2030-01-01-future-pre.md.raw", check=False)
        self.assertEqual(ign.returncode, 0)
        tool(self.root, "untrack", "--manifest", str(mf), "--backup", str(bk), "--apply")
        self.assertEqual((self.root / ".gitignore").read_text().count(f"{RAW_DIR}/*.raw"), 1)


class DisposableCloneProof(ToolCase):
    """Untrack in repo A, clone it fresh (B has no versioned copies), then prove the recovery."""

    def test_untrack_clone_links_restore_byte_identical_from_backup_and_git(self):
        mf = self.generate()
        bk = self.backup(mf)
        r = tool(self.root, "untrack", "--manifest", str(mf), "--backup", str(bk), "--apply")
        self.assertEqual(r.returncode, 0, r.stderr)
        git(self.root, "add", ".gitignore")
        git(self.root, "commit", "-q", "-m", "chore: untrack review raw prompts")

        for variant, flag in (("clone-backup", "--backup"), ("clone-git", "--from-git")):
            clone = self.tmp / variant
            subprocess.run(["git", "clone", "-q", "--no-hardlinks", str(self.root), str(clone)],
                           env=GIT_ENV, check=True, capture_output=True)
            for rel in self.files:
                self.assertFalse((clone / rel).exists(), "fresh clone must not carry the class")
            # Links and verdict artifacts do not depend on the raw class.
            self.assertEqual(doc_links.main(["--root", str(clone), "docs/index.md"]), 0)
            self.assertTrue((clone / RAW_DIR / "2026-01-01-alpha-pre.md").is_file())
            # Without the class verify reports MISSING (4), never a silent pass.
            self.assertEqual(tool(clone, "verify", "--manifest", str(mf)).returncode, 4)
            args = ["restore", "--manifest", str(mf), flag] + ([str(bk)] if flag == "--backup" else [])
            r = tool(clone, *args)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertEqual(tool(clone, "verify", "--manifest", str(mf)).returncode, 0)
            for rel, data in self.files.items():
                self.assertEqual((clone / rel).read_bytes(), data)
            # Restored files are ignored, so the working tree stays clean.
            self.assertEqual(git(clone, "status", "--porcelain").stdout, b"")


class RealRepositoryContract(unittest.TestCase):
    def test_committed_manifest_verifies_against_the_real_tree(self):
        self.assertTrue(REAL_MANIFEST.is_file(), "manifest must be committed")
        m = json.loads(REAL_MANIFEST.read_text())
        self.assertEqual(m["class"], CLASS)
        self.assertGreater(len(m["entries"]), 0)
        r = tool(REPO_ROOT, "verify", "--manifest", str(REAL_MANIFEST), "--allow-missing")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertNotIn("MISMATCH", r.stdout)

    def test_every_manifest_blob_is_recoverable_from_git_history_or_present(self):
        m = json.loads(REAL_MANIFEST.read_text())
        shallow = subprocess.run(["git", "-C", str(REPO_ROOT), "rev-parse", "--is-shallow-repository"],
                                 capture_output=True, text=True).stdout.strip() == "true"
        for e in m["entries"]:
            present = (REPO_ROOT / e["path"]).is_file()
            in_git = subprocess.run(["git", "-C", str(REPO_ROOT), "cat-file", "-e", e["git_blob"]],
                                    capture_output=True).returncode == 0
            if shallow:
                self.assertTrue(present or in_git, e["path"])
            else:
                self.assertTrue(in_git, f"blob {e['git_blob']} for {e['path']} not in the object store")

    def test_policy_document_defines_every_class_and_forbids_destructive_cleanup(self):
        text = POLICY.read_text(encoding="utf-8")
        for needle in ("review-raw", "Owner", "Duração", "Armazenamento", "Consulta", "Restauração",
                       "pending O1", "retention-manifest.py", "git count-objects"):
            self.assertIn(needle, text, needle)
        for forbidden in ("gc --prune=now", "reflog expire", "filter-repo", "git push --force"):
            self.assertIn(forbidden, text)
        low = text.lower()
        self.assertIn("proibid", low)
        self.assertIn("não promet", low)  # no MB saving promised without measurement
        self.assertEqual(doc_links.main(["--root", str(REPO_ROOT), "docs/OPERATIONS_GIT_RETENTION.md"]), 0)

    def test_the_tool_exposes_no_history_or_object_deletion_command(self):
        r = subprocess.run([sys.executable, str(TOOL), "--help"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 0)
        for word in ("prune", "expire", "gc", "rewrite", "filter"):
            self.assertNotIn(word, r.stdout.lower().replace("get-", ""))


if __name__ == "__main__":
    unittest.main()

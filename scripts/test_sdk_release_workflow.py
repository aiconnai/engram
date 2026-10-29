#!/usr/bin/env python3
"""Static and behavioural tests for .github/workflows/sdk-release.yml.

Guards the release-pipeline invariants: no hardcoded release tag or SDK version,
every safety gate still present, and the publish jobs re-derive the version mapping.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import tempfile
import unittest
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
WORKFLOW = REPO_ROOT / ".github" / "workflows" / "sdk-release.yml"

# Safety gates that must survive every edit of the workflow.
REQUIRED_GATES = (
    "supplied SHA does not match github.sha",
    "checked-out commit does not match supplied SHA",
    "dry-run cannot publish",
    "dry-run does not accept a release tag",
    "non-dry-run must explicitly request publish",
    "publish dispatch must select exactly one registry",
    "publish requires the exact tag ref",
    "publish requires an annotated tag object",
    "publish requires a cryptographically signed tag",
    "tag, github.sha, and supplied SHA do not agree",
    "approved immutable GitHub Release is missing",
    "Dry-run complete: no registry request was attempted.",
    "scripts/verify-release-tag.sh",
    "--trusted-ref origin/main",
    "ENGRAM_RELEASE_ALLOWED_SIGNERS",
    "environment: release-pypi",
    "environment: release-npm",
    "id-token: write",
)


def read_workflow() -> str:
    return WORKFLOW.read_text(encoding="utf-8")


def job_block(text: str, job: str) -> str:
    start = text.index(f"\n  {job}:\n")
    match = re.search(r"\n  [a-z][a-z0-9-]*:\n", text[start + 3 :])
    return text[start:] if match is None else text[start : start + 3 + match.start()]


def guard_script(text: str) -> str:
    """Extract the `run:` body of the preflight step with `id: guard`."""
    lines = text.splitlines()
    guard = lines.index("        id: guard")
    run = next(i for i in range(guard, len(lines)) if lines[i] == "        run: |")
    body: list[str] = []
    for line in lines[run + 1 :]:
        if line.strip() and not line.startswith("          "):
            break
        body.append(line[10:])
    return "\n".join(body) + "\n"


class TestWorkflowStatic(unittest.TestCase):
    def test_no_hardcoded_release_tag_or_sdk_version(self):
        offenders = []
        for number, line in enumerate(read_workflow().splitlines(), start=1):
            if "uses:" in line:  # pinned action refs carry their own version comments
                continue
            if re.search(r"\bv?\d+\.\d+\.\d+\b", line):
                offenders.append(f"{number}: {line.strip()}")
        self.assertEqual(offenders, [], "hardcoded versions found in sdk-release.yml")

    def test_required_safety_gates_are_present(self):
        text = read_workflow()
        missing = [gate for gate in REQUIRED_GATES if gate not in text]
        self.assertEqual(missing, [])

    def test_preflight_derives_versions_from_manifests(self):
        script = guard_script(read_workflow())
        self.assertIn("--release-plan", script)
        self.assertIn('--release-tag "${INPUT_TAG}"', script)
        self.assertIn("python_version=", script)
        self.assertIn("npm_version=", script)
        self.assertIn('jq -e --arg tag "${INPUT_TAG}"', script)

    def test_registry_guards_use_derived_versions(self):
        text = read_workflow()
        self.assertIn("engram-client/${PYTHON_VERSION}/json", text)
        self.assertIn('"engram-client@${NPM_VERSION}"', text)

    def test_publish_jobs_revalidate_mapping_after_approval(self):
        text = read_workflow()
        for job, key in (("publish-python", "python_version"), ("publish-npm", "npm_version")):
            with self.subTest(job=job):
                block = job_block(text, job)
                self.assertIn('--release-plan --release-tag "${RELEASE_TAG}"', block)
                self.assertIn(f"s/^{key}=//p", block)
                self.assertIn('jq -e --arg tag "${RELEASE_TAG}"', block)
                self.assertIn('git show origin/main:scripts/verify-release-tag.sh > "${trusted_verifier}"', block)
                self.assertIn(
                    'bash "${trusted_verifier}" --repo . --tag "${RELEASE_TAG}" --expected-sha "${RELEASE_SHA}" --trusted-ref origin/main',
                    block,
                )

    def test_workflow_never_executes_untrusted_worktree_tag_verifier(self):
        text = read_workflow()
        self.assertNotIn("bash scripts/verify-release-tag.sh", text)
        self.assertNotIn("./scripts/verify-release-tag.sh", text)
        for job in ("preflight", "publish-python", "publish-npm"):
            with self.subTest(job=job):
                block = job_block(text, job)
                self.assertIn("git show origin/main:scripts/verify-release-tag.sh", block)

    def test_post_publish_verification_uses_sdk_version_not_core_tag(self):
        text = read_workflow()
        self.assertNotIn("RELEASE_TAG#v", text)
        self.assertIn('--version "${PYTHON_VERSION}"', job_block(text, "publish-python"))
        self.assertIn('--version "${NPM_VERSION}"', job_block(text, "publish-npm"))

    def test_dry_run_remains_the_default(self):
        text = read_workflow()
        self.assertRegex(text, r"dry_run:\n(?:.*\n)*?\s+default: true")
        self.assertRegex(text, r"publish:\n(?:.*\n)*?\s+default: false")


@unittest.skipUnless(shutil.which("bash") and shutil.which("git"), "bash and git are required")
class TestPreflightGuardBehaviour(unittest.TestCase):
    """Run the real guard script with a faked GitHub environment (no network is reachable)."""

    def run_guard(self, **env_overrides: str) -> tuple[int, str, str]:
        head = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=REPO_ROOT, capture_output=True, text=True, check=True
        ).stdout.strip()
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "github_output"
            output.write_text("", encoding="utf-8")
            env = {
                **os.environ,
                "GITHUB_EVENT_NAME": "workflow_dispatch",
                "GITHUB_SHA": head,
                "GITHUB_REF_TYPE": "branch",
                "GITHUB_REF_NAME": "main",
                "GITHUB_REPOSITORY": "aiconnai/engram",
                "GITHUB_OUTPUT": str(output),
                "GH_TOKEN": "unused",
                "INPUT_CHANNEL": "all",
                "INPUT_DRY_RUN": "true",
                "INPUT_PUBLISH": "false",
                "INPUT_SHA": head,
                "INPUT_TAG": "",
            }
            env.update(env_overrides)
            result = subprocess.run(
                ["bash", "-c", guard_script(read_workflow())],
                cwd=REPO_ROOT, env=env, capture_output=True, text=True, timeout=60,
            )
            return result.returncode, output.read_text(encoding="utf-8"), result.stdout + result.stderr

    def test_dry_run_derives_sdk_versions_from_manifests(self):
        code, outputs, log = self.run_guard()
        self.assertEqual(code, 0, log)
        lines = outputs.splitlines()
        self.assertIn("dry_run=true", lines)
        self.assertIn("release_tag=", lines)
        version = re.compile(r"^(python|npm)_version=\d+\.\d+\.\d+$")
        self.assertEqual(len([line for line in lines if version.match(line)]), 2, outputs)

    def test_publish_rejects_malformed_tag_before_any_network_call(self):
        for tag in ("", "v0.24", "0.24.0", "v0.24.0-rc1"):
            with self.subTest(tag=tag):
                code, outputs, log = self.run_guard(
                    INPUT_DRY_RUN="false", INPUT_PUBLISH="true", INPUT_CHANNEL="python", INPUT_TAG=tag
                )
                self.assertNotEqual(code, 0)
                self.assertEqual(outputs, "")
                self.assertIn("exact stable vX.Y.Z core release tag", log)

    def test_publish_no_longer_special_cases_one_tag(self):
        # A well-formed tag passes the format gate and is then rejected by the tag-ref gate,
        # not by a hardcoded "approved tag" comparison.
        code, outputs, log = self.run_guard(
            INPUT_DRY_RUN="false", INPUT_PUBLISH="true", INPUT_CHANNEL="python", INPUT_TAG="v9.9.9"
        )
        self.assertNotEqual(code, 0)
        self.assertEqual(outputs, "")
        self.assertIn("publish requires the exact tag ref", log)

    def test_dry_run_rejects_a_release_tag(self):
        code, _, log = self.run_guard(INPUT_TAG="v0.24.0")
        self.assertNotEqual(code, 0)
        self.assertIn("dry-run does not accept a release tag", log)


@unittest.skipUnless(shutil.which("bash"), "bash is required")
class TestArtifactMetadata(unittest.TestCase):
    def test_registry_channels_map_to_sdk_compatibility_and_can_be_repeated(self):
        matrix = tomllib.loads((REPO_ROOT / "docs/releases/channel-matrix.toml").read_text())
        for channel, sdk, registry, filenames in (
            ("python", "python", "pypi", ("fixture.whl", "fixture.tar.gz")),
            ("npm", "typescript", "npm", ("fixture.tgz",)),
        ):
            with self.subTest(channel=channel), tempfile.TemporaryDirectory() as tmp:
                artifacts = Path(tmp)
                for name in filenames:
                    (artifacts / name).write_bytes(b"synthetic archive bytes for metadata hashing")
                command = [
                    "bash", str(REPO_ROOT / "scripts/verify-sdk-artifacts.sh"),
                    "--write-metadata", channel, "--artifact-dir", tmp,
                    "--expected-sha", "1" * 40,
                ]
                metadata_path = artifacts / f"{channel}-sdk-metadata.json"
                previous = None
                for _ in range(2):
                    result = subprocess.run(command, cwd=REPO_ROOT, capture_output=True, text=True, timeout=30)
                    self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                    metadata = json.loads(metadata_path.read_text())
                    policy = next(row for row in matrix["channels"] if row["id"] == registry)
                    compatibility = next(row for row in matrix["sdk_compatibility"] if row["sdk"] == sdk)
                    self.assertEqual(metadata["channel"], registry)
                    self.assertEqual(metadata["version"], policy["local_version"])
                    self.assertEqual(metadata["sha"], "1" * 40)
                    for key in ("compatible_core_min", "compatible_core_max"):
                        self.assertEqual(metadata[key], compatibility[key])
                    self.assertEqual(metadata["files"], [
                        {"name": name, "sha256": hashlib.sha256((artifacts / name).read_bytes()).hexdigest()}
                        for name in sorted(filenames)
                    ])
                    if previous is not None:
                        self.assertEqual(metadata, previous, "metadata must not hash itself on retry")
                    previous = metadata


@unittest.skipUnless(
    shutil.which("bash") and shutil.which("git") and shutil.which("ssh-keygen"),
    "bash, git, and ssh-keygen are required",
)
class TestVerifyReleaseTagScript(unittest.TestCase):
    """End-to-end cryptographic tests for scripts/verify-release-tag.sh."""

    SCRIPT = REPO_ROOT / "scripts" / "verify-release-tag.sh"

    def _git(self, repo: Path, *args: str, env: dict[str, str] | None = None) -> str:
        run_env = {**os.environ, **(env or {})}
        result = subprocess.run(
            ["git", *args],
            cwd=repo,
            env=run_env,
            capture_output=True,
            text=True,
            check=True,
        )
        return result.stdout.strip()

    def _keygen(self, directory: Path, name: str, comment: str) -> tuple[Path, str]:
        key_path = directory / name
        subprocess.run(
            ["ssh-keygen", "-q", "-t", "ed25519", "-N", "", "-C", comment, "-f", str(key_path)],
            check=True,
        )
        pub = key_path.with_suffix(".pub").read_text(encoding="utf-8").strip()
        parts = pub.split()
        signer_line = f"{comment} namespaces=\"git\" {parts[0]} {parts[1]} {comment}"
        return key_path, signer_line

    def _verify(self, *args: str, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
        base_env = {
            k: v
            for k, v in os.environ.items()
            if not k.startswith("ENGRAM_RELEASE_ALLOWED_SIGNERS")
        }
        if env:
            base_env.update(env)
        return subprocess.run(
            ["bash", str(self.SCRIPT), *args],
            cwd=REPO_ROOT,
            env=base_env,
            capture_output=True,
            text=True,
            timeout=30,
        )

    def test_verify_release_tag_cryptographic_and_anti_self_authorization_matrix(self):
        with tempfile.TemporaryDirectory() as tmp_str:
            tmp = Path(tmp_str)
            keys = tmp / "keys"
            keys.mkdir()
            trusted_key, trusted_line = self._keygen(keys, "trusted", "release@aiconnai.com")
            rogue_key, rogue_line = self._keygen(keys, "rogue", "attacker@example.invalid")

            repo = tmp / "repo"
            repo.mkdir()
            self._git(repo, "init", "-b", "main")
            self._git(repo, "config", "user.name", "Release Tester")
            self._git(repo, "config", "user.email", "release@aiconnai.com")

            (repo / "docs" / "releases").mkdir(parents=True)
            (repo / "docs" / "releases" / "allowed-signers").write_text(
                f"# Trusted signers on main\n{trusted_line}\n",
                encoding="utf-8",
            )
            (repo / "README.md").write_text("release candidate\n", encoding="utf-8")
            self._git(repo, "add", "docs/releases/allowed-signers", "README.md")
            self._git(repo, "commit", "-m", "chore: initial main commit")
            main_sha = self._git(repo, "rev-parse", "HEAD")
            self._git(repo, "update-ref", "refs/remotes/origin/main", main_sha)

            # 1. Valid SSH-signed annotated tag verified against origin/main trust store -> PASS
            self._git(
                repo,
                "-c",
                "gpg.format=ssh",
                "-c",
                f"user.signingkey={trusted_key}",
                "tag",
                "-s",
                "-m",
                "Release v1.0.0",
                "v1.0.0",
                main_sha,
            )
            ok = self._verify(
                "--repo",
                str(repo),
                "--tag",
                "v1.0.0",
                "--expected-sha",
                main_sha,
                "--trusted-ref",
                "origin/main",
            )
            self.assertEqual(ok.returncode, 0, ok.stdout + ok.stderr)

            # 2. Inline ENGRAM_RELEASE_ALLOWED_SIGNERS + SHA256 pin -> PASS on match, FAIL on mismatch
            inline_content = f"{trusted_line}\n"
            inline_sha = hashlib.sha256(inline_content.encode("utf-8")).hexdigest()
            ok_inline = self._verify(
                "--repo",
                str(repo),
                "--tag",
                "v1.0.0",
                "--expected-sha",
                main_sha,
                "--trusted-ref",
                "origin/main",
                env={
                    "ENGRAM_RELEASE_ALLOWED_SIGNERS": trusted_line,
                    "ENGRAM_RELEASE_ALLOWED_SIGNERS_SHA256": inline_sha,
                },
            )
            self.assertEqual(ok_inline.returncode, 0, ok_inline.stdout + ok_inline.stderr)

            bad_pin = self._verify(
                "--repo",
                str(repo),
                "--tag",
                "v1.0.0",
                "--expected-sha",
                main_sha,
                env={
                    "ENGRAM_RELEASE_ALLOWED_SIGNERS": trusted_line,
                    "ENGRAM_RELEASE_ALLOWED_SIGNERS_SHA256": "0" * 64,
                },
            )
            self.assertNotEqual(bad_pin.returncode, 0)
            self.assertIn("SHA-256 mismatch", bad_pin.stderr)

            # 3. Wrong expected SHA -> FAIL
            wrong_sha = self._verify(
                "--repo",
                str(repo),
                "--tag",
                "v1.0.0",
                "--expected-sha",
                "f" * 40,
                "--trusted-ref",
                "origin/main",
            )
            self.assertNotEqual(wrong_sha.returncode, 0)
            self.assertIn("tag, github.sha, and supplied SHA do not agree", wrong_sha.stderr)

            # 4. Signed by unknown/rogue SSH key -> FAIL
            self._git(
                repo,
                "-c",
                "gpg.format=ssh",
                "-c",
                f"user.signingkey={rogue_key}",
                "tag",
                "-s",
                "-m",
                "Rogue v1.0.1",
                "v1.0.1",
                main_sha,
            )
            rogue_res = self._verify(
                "--repo",
                str(repo),
                "--tag",
                "v1.0.1",
                "--expected-sha",
                main_sha,
                "--trusted-ref",
                "origin/main",
            )
            self.assertNotEqual(rogue_res.returncode, 0)
            self.assertIn("publish requires a cryptographically signed tag", rogue_res.stderr)

            # 5. Anti-self-authorization: tag commit alters BOTH docs/releases/allowed-signers
            # AND scripts/verify-release-tag.sh in its own tree while origin/main still only
            # trusts trusted_line -> materializing verifier from origin/main MUST FAIL!
            (repo / "scripts").mkdir(parents=True, exist_ok=True)
            (repo / "scripts" / "verify-release-tag.sh").write_text(
                self.SCRIPT.read_text(encoding="utf-8"),
                encoding="utf-8",
            )
            self._git(repo, "add", "scripts/verify-release-tag.sh")
            self._git(repo, "commit", "-m", "chore: commit trusted verifier on main")
            main_with_verifier_sha = self._git(repo, "rev-parse", "HEAD")
            self._git(repo, "update-ref", "refs/remotes/origin/main", main_with_verifier_sha)

            self._git(repo, "checkout", "-b", "tampered-tag-branch")
            (repo / "docs" / "releases" / "allowed-signers").write_text(
                f"{rogue_line}\n",
                encoding="utf-8",
            )
            (repo / "scripts" / "verify-release-tag.sh").write_text(
                "#!/usr/bin/env bash\nexit 0\n",
                encoding="utf-8",
            )
            self._git(repo, "add", "docs/releases/allowed-signers", "scripts/verify-release-tag.sh")
            self._git(repo, "commit", "-m", "tamper: self-authorize rogue signer and neuter worktree verifier")
            tampered_sha = self._git(repo, "rev-parse", "HEAD")
            self._git(
                repo,
                "-c",
                "gpg.format=ssh",
                "-c",
                f"user.signingkey={rogue_key}",
                "tag",
                "-s",
                "-m",
                "Tampered v1.0.2",
                "v1.0.2",
                tampered_sha,
            )
            trusted_verifier_copy = tmp / "trusted-verify-release-tag.sh"
            trusted_verifier_copy.write_text(
                self._git(repo, "show", "origin/main:scripts/verify-release-tag.sh") + "\n",
                encoding="utf-8",
            )
            clean_env = {
                k: v
                for k, v in os.environ.items()
                if not k.startswith("ENGRAM_RELEASE_ALLOWED_SIGNERS")
            }
            # Exercise materialized verifier outside repo BOTH without --repo (cwd fallback)
            # and with --repo . against valid v1.0.0 (PASS) and tampered v1.0.2 (FAIL).
            valid_from_temp = subprocess.run(
                [
                    "bash",
                    str(trusted_verifier_copy),
                    "--tag",
                    "v1.0.0",
                    "--expected-sha",
                    main_sha,
                    "--trusted-ref",
                    "origin/main",
                ],
                cwd=repo,
                env=clean_env,
                capture_output=True,
                text=True,
                timeout=30,
            )
            self.assertEqual(
                valid_from_temp.returncode,
                0,
                valid_from_temp.stdout + valid_from_temp.stderr,
            )
            self_auth = subprocess.run(
                [
                    "bash",
                    str(trusted_verifier_copy),
                    "--tag",
                    "v1.0.2",
                    "--expected-sha",
                    tampered_sha,
                    "--trusted-ref",
                    "origin/main",
                ],
                cwd=repo,
                env=clean_env,
                capture_output=True,
                text=True,
                timeout=30,
            )
            self.assertNotEqual(self_auth.returncode, 0)
            self.assertNotIn("cannot read", self_auth.stderr)
            self.assertIn("publish requires a cryptographically signed tag", self_auth.stderr)

            # 6. Unsigned annotated tag -> FAIL
            self._git(repo, "tag", "-a", "-m", "Unsigned v1.0.3", "v1.0.3", main_sha)
            unsigned = self._verify(
                "--repo",
                str(repo),
                "--tag",
                "v1.0.3",
                "--expected-sha",
                main_sha,
                "--trusted-ref",
                "origin/main",
            )
            self.assertNotEqual(unsigned.returncode, 0)
            self.assertIn("publish requires a cryptographically signed tag", unsigned.stderr)

            # 7. Lightweight tag -> FAIL
            self._git(repo, "tag", "v1.0.4", main_sha)
            lightweight = self._verify(
                "--repo",
                str(repo),
                "--tag",
                "v1.0.4",
                "--expected-sha",
                main_sha,
                "--trusted-ref",
                "origin/main",
            )
            self.assertNotEqual(lightweight.returncode, 0)
            self.assertIn("publish requires an annotated tag object", lightweight.stderr)

            # 8. Default committed docs/releases/allowed-signers template (comment-only) -> FAIL CLOSED
            empty_signers = REPO_ROOT / "docs" / "releases" / "allowed-signers"
            unconfigured = self._verify(
                "--repo",
                str(repo),
                "--tag",
                "v1.0.0",
                "--expected-sha",
                main_sha,
                "--allowed-signers",
                str(empty_signers),
            )
            self.assertNotEqual(unconfigured.returncode, 0)
            self.assertIn("no trusted release signers configured", unconfigured.stderr)


if __name__ == "__main__":
    unittest.main()

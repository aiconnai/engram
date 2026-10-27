#!/usr/bin/env python3
"""Static and behavioural tests for .github/workflows/sdk-release.yml.

Guards the release-pipeline invariants: no hardcoded release tag or SDK version,
every safety gate still present, and the publish jobs re-derive the version mapping.
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import tempfile
import unittest
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
    "git verify-tag",
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
                self.assertIn("git verify-tag", block)

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


if __name__ == "__main__":
    unittest.main()

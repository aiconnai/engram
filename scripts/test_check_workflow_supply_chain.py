#!/usr/bin/env python3
"""Tests for the workflow supply-chain checker (task Q5).

Pins the policy for ``.github/workflows`` and the Dockerfile base images:

* every third-party ``uses:`` is pinned by a full commit SHA;
* every container image is pinned by digest, or sits in the governed, expiring ledger;
* a job that can run on a pull_request gets no write permission and no secret other than the
  automatic GITHUB_TOKEN (measurement jobs are read-only; publication runs on trusted events);
* ``pull_request_target`` is never used.

Synthetic workflows are written to temporary directories; the repository itself must pass.
"""

from __future__ import annotations

import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CHECKER = ROOT / "scripts" / "check-workflow-supply-chain.py"
LEDGER = ROOT / "docs" / "security" / "supply-chain-pins.toml"
SHA = "93cb6efe18208431cddfb8368fd83d5badbf9bfd"
DIGEST = "sha256:" + "ab" * 32

PR_WORKFLOW_HEADER = """\
name: Synthetic
on:
  pull_request:
    branches: [main]
permissions:
  contents: read
jobs:
"""

EMPTY_LEDGER = 'schema_version = 1\n'


def run_checker(
    workflows: dict[str, str],
    ledger: str = EMPTY_LEDGER,
    dockerfile: str | None = None,
    today: str = "2026-10-05",
) -> subprocess.CompletedProcess:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        wf_dir = root / "workflows"
        wf_dir.mkdir()
        for name, text in workflows.items():
            (wf_dir / name).write_text(textwrap.dedent(text))
        ledger_path = root / "ledger.toml"
        ledger_path.write_text(textwrap.dedent(ledger))
        args = [
            sys.executable,
            str(CHECKER),
            "--workflows",
            str(wf_dir),
            "--ledger",
            str(ledger_path),
            "--today",
            today,
        ]
        if dockerfile is not None:
            docker_path = root / "Dockerfile"
            docker_path.write_text(textwrap.dedent(dockerfile))
            args += ["--dockerfile", str(docker_path)]
        return subprocess.run(args, capture_output=True, text=True, timeout=60)


def job(body: str) -> str:
    return textwrap.indent(textwrap.dedent(body), "  ")


class RepositoryTests(unittest.TestCase):
    def test_repository_workflows_pass(self) -> None:
        proc = subprocess.run(
            [sys.executable, str(CHECKER)],
            cwd=ROOT,
            capture_output=True,
            text=True,
            timeout=60,
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("workflow supply-chain: PASS", proc.stdout)

    def test_ledger_entries_are_owned_and_expire(self) -> None:
        import datetime
        import tomllib

        data = tomllib.loads(LEDGER.read_text())
        self.assertIsInstance(data.get("owner"), str)
        entries = data.get("unpinned_images")
        self.assertIsInstance(entries, list, "ledger must declare unpinned_images (may be empty)")
        as_of = data["as_of"]
        for entry in entries:
            with self.subTest(image=entry.get("image")):
                for key in ("image", "reason", "owner"):
                    self.assertTrue(str(entry.get(key, "")).strip(), key)
                self.assertIsInstance(entry.get("expires"), datetime.date)
                self.assertLessEqual((entry["expires"] - as_of).days, 90)


class UsesPinTests(unittest.TestCase):
    def test_sha_pinned_action_passes(self) -> None:
        text = PR_WORKFLOW_HEADER + job(f"""\
            a:
              runs-on: ubuntu-latest
              steps:
                - uses: actions/checkout@{SHA} # v5
            """)
        self.assertEqual(run_checker({"a.yml": text}).returncode, 0)

    def test_tag_pinned_action_fails(self) -> None:
        text = PR_WORKFLOW_HEADER + job("""\
            a:
              runs-on: ubuntu-latest
              steps:
                - uses: actions/checkout@v5
            """)
        proc = run_checker({"a.yml": text})
        self.assertEqual(proc.returncode, 1)
        self.assertIn("actions/checkout@v5", proc.stderr)

    def test_branch_pinned_action_fails(self) -> None:
        text = PR_WORKFLOW_HEADER + job("""\
            a:
              runs-on: ubuntu-latest
              steps:
                - uses: some/action@main
            """)
        self.assertEqual(run_checker({"a.yml": text}).returncode, 1)

    def test_local_action_is_allowed(self) -> None:
        text = PR_WORKFLOW_HEADER + job("""\
            a:
              runs-on: ubuntu-latest
              steps:
                - uses: ./.github/actions/local
            """)
        self.assertEqual(run_checker({"a.yml": text}).returncode, 0)

    def test_docker_uses_without_digest_fails(self) -> None:
        text = PR_WORKFLOW_HEADER + job("""\
            a:
              runs-on: ubuntu-latest
              steps:
                - uses: docker://alpine:3.20
            """)
        self.assertEqual(run_checker({"a.yml": text}).returncode, 1)


class ImageDigestTests(unittest.TestCase):
    def docker_job(self, image: str) -> str:
        return PR_WORKFLOW_HEADER + job(f"""\
            a:
              runs-on: ubuntu-latest
              steps:
                - run: |
                    docker run --rm \\
                      -v "$PWD:/src" \\
                      {image} \\
                      scan /src
            """)

    def test_digest_pinned_image_passes(self) -> None:
        proc = run_checker({"a.yml": self.docker_job(f"vendor/tool:1.2.3@{DIGEST}")})
        self.assertEqual(proc.returncode, 0, proc.stderr)

    def test_tag_only_image_fails(self) -> None:
        proc = run_checker({"a.yml": self.docker_job("vendor/tool:1.2.3")})
        self.assertEqual(proc.returncode, 1)
        self.assertIn("vendor/tool:1.2.3", proc.stderr)

    def test_tag_only_image_in_ledger_is_tolerated_until_expiry(self) -> None:
        ledger = """\
            schema_version = 1
            [[unpinned_images]]
            image = "vendor/tool:1.2.3"
            reason = "digest requires a registry lookup"
            owner = "Ronaldo"
            expires = 2026-11-05
            """
        workflows = {"a.yml": self.docker_job("vendor/tool:1.2.3")}
        self.assertEqual(run_checker(workflows, ledger).returncode, 0)
        self.assertEqual(run_checker(workflows, ledger, today="2026-11-06").returncode, 1)

    def test_ledger_entry_without_owner_is_rejected(self) -> None:
        ledger = """\
            schema_version = 1
            [[unpinned_images]]
            image = "vendor/tool:1.2.3"
            reason = "no owner"
            expires = 2026-11-05
            """
        workflows = {"a.yml": self.docker_job("vendor/tool:1.2.3")}
        self.assertEqual(run_checker(workflows, ledger).returncode, 1)

    def test_unused_ledger_entry_is_rejected(self) -> None:
        ledger = """\
            schema_version = 1
            [[unpinned_images]]
            image = "vendor/gone:9"
            reason = "stale"
            owner = "Ronaldo"
            expires = 2026-11-05
            """
        workflows = {"a.yml": self.docker_job(f"vendor/tool:1.2.3@{DIGEST}")}
        self.assertEqual(run_checker(workflows, ledger).returncode, 1)

    def test_dockerfile_base_without_digest_fails_unless_ledgered(self) -> None:
        workflows = {"a.yml": PR_WORKFLOW_HEADER + job("""\
            a:
              runs-on: ubuntu-latest
              steps:
                - run: echo hi
            """)}
        docker = """\
            FROM rust:1-bookworm AS builder
            RUN true
            FROM builder AS test
            FROM debian:bookworm-slim
            """
        self.assertEqual(run_checker(workflows, dockerfile=docker).returncode, 1)
        ledger = """\
            schema_version = 1
            [[unpinned_images]]
            image = "rust:1-bookworm"
            reason = "base image"
            owner = "Ronaldo"
            expires = 2026-11-05
            [[unpinned_images]]
            image = "debian:bookworm-slim"
            reason = "base image"
            owner = "Ronaldo"
            expires = 2026-11-05
            """
        self.assertEqual(run_checker(workflows, ledger, docker).returncode, 0)


class PullRequestExposureTests(unittest.TestCase):
    def test_job_write_permission_on_pull_request_fails(self) -> None:
        text = PR_WORKFLOW_HEADER + job(f"""\
            publish:
              runs-on: ubuntu-latest
              permissions:
                contents: write
              steps:
                - uses: actions/checkout@{SHA}
            """)
        proc = run_checker({"a.yml": text})
        self.assertEqual(proc.returncode, 1)
        self.assertIn("publish", proc.stderr)

    def test_pull_requests_write_fails(self) -> None:
        text = PR_WORKFLOW_HEADER + job(f"""\
            comment:
              runs-on: ubuntu-latest
              permissions:
                contents: read
                pull-requests: write
              steps:
                - uses: actions/checkout@{SHA}
            """)
        self.assertEqual(run_checker({"a.yml": text}).returncode, 1)

    def test_security_events_write_on_pull_request_fails(self) -> None:
        text = PR_WORKFLOW_HEADER + job(f"""\
            scan:
              runs-on: ubuntu-latest
              permissions:
                security-events: write
              steps:
                - uses: actions/checkout@{SHA}
            """)
        self.assertEqual(run_checker({"a.yml": text}).returncode, 1)

    def test_write_job_excluded_from_pull_requests_passes(self) -> None:
        text = PR_WORKFLOW_HEADER + job(f"""\
            publish:
              runs-on: ubuntu-latest
              if: github.event_name != 'pull_request'
              permissions:
                security-events: write
              steps:
                - uses: actions/checkout@{SHA}
            """)
        proc = run_checker({"a.yml": text})
        self.assertEqual(proc.returncode, 0, proc.stderr)

    def test_write_job_scheduled_only_passes(self) -> None:
        text = PR_WORKFLOW_HEADER + job(f"""\
            publish:
              runs-on: ubuntu-latest
              if: github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'
              permissions:
                contents: write
              steps:
                - uses: actions/checkout@{SHA}
            """)
        self.assertEqual(run_checker({"a.yml": text}).returncode, 0)

    def test_always_condition_does_not_exclude_pull_requests(self) -> None:
        text = PR_WORKFLOW_HEADER + job(f"""\
            publish:
              runs-on: ubuntu-latest
              if: always()
              permissions:
                contents: write
              steps:
                - uses: actions/checkout@{SHA}
            """)
        self.assertEqual(run_checker({"a.yml": text}).returncode, 1)

    def test_workflow_level_write_with_pull_request_trigger_fails(self) -> None:
        text = f"""\
            name: Synthetic
            on:
              pull_request:
            permissions:
              contents: write
            jobs:
              a:
                runs-on: ubuntu-latest
                steps:
                  - uses: actions/checkout@{SHA}
            """
        self.assertEqual(run_checker({"a.yml": text}).returncode, 1)

    def test_write_all_shorthand_fails(self) -> None:
        text = PR_WORKFLOW_HEADER + job(f"""\
            a:
              runs-on: ubuntu-latest
              permissions: write-all
              steps:
                - uses: actions/checkout@{SHA}
            """)
        self.assertEqual(run_checker({"a.yml": text}).returncode, 1)

    def test_secret_in_pull_request_job_fails(self) -> None:
        text = PR_WORKFLOW_HEADER + job(f"""\
            a:
              runs-on: ubuntu-latest
              steps:
                - uses: actions/checkout@{SHA}
                - run: ./deploy
                  env:
                    TOKEN: ${{{{ secrets.DEPLOY_TOKEN }}}}
            """)
        proc = run_checker({"a.yml": text})
        self.assertEqual(proc.returncode, 1)
        self.assertIn("DEPLOY_TOKEN", proc.stderr)

    def test_github_token_in_pull_request_job_is_allowed(self) -> None:
        text = PR_WORKFLOW_HEADER + job(f"""\
            a:
              runs-on: ubuntu-latest
              steps:
                - uses: actions/checkout@{SHA}
                  with:
                    token: ${{{{ secrets.GITHUB_TOKEN }}}}
            """)
        self.assertEqual(run_checker({"a.yml": text}).returncode, 0)

    def test_secret_in_job_excluded_from_pull_requests_is_allowed(self) -> None:
        text = PR_WORKFLOW_HEADER + job(f"""\
            cov:
              runs-on: ubuntu-latest
              if: github.event_name == 'schedule'
              steps:
                - uses: actions/checkout@{SHA}
                - run: upload
                  env:
                    CODECOV_TOKEN: ${{{{ secrets.CODECOV_TOKEN }}}}
            """)
        self.assertEqual(run_checker({"a.yml": text}).returncode, 0)

    def test_pull_request_target_is_forbidden(self) -> None:
        text = f"""\
            name: Synthetic
            on:
              pull_request_target:
            permissions:
              contents: read
            jobs:
              a:
                runs-on: ubuntu-latest
                steps:
                  - uses: actions/checkout@{SHA}
            """
        self.assertEqual(run_checker({"a.yml": text}).returncode, 1)

    def test_write_in_workflow_without_pull_request_trigger_passes(self) -> None:
        text = f"""\
            name: Release
            on:
              push:
                tags: ["v*"]
            permissions:
              contents: write
            jobs:
              a:
                runs-on: ubuntu-latest
                steps:
                  - uses: actions/checkout@{SHA}
            """
        self.assertEqual(run_checker({"a.yml": text}).returncode, 0)


class RefinementTests(unittest.TestCase):
    """Review-fix round: flow-style permissions, missing top-level permissions, mixed conditions."""

    def pr_job(self, condition: str, perms: str = "contents: write") -> str:
        return PR_WORKFLOW_HEADER + job(f"""\
            publish:
              runs-on: ubuntu-latest
              if: {condition}
              permissions:
                {perms}
              steps:
                - uses: actions/checkout@{SHA}
            """)

    def test_flow_style_job_permissions_are_parsed(self) -> None:
        for flow in ("{contents: write}", "{ contents: read, pull-requests: write }"):
            with self.subTest(flow):
                text = PR_WORKFLOW_HEADER + job(f"""\
                    a:
                      runs-on: ubuntu-latest
                      permissions: {flow}
                      steps:
                        - uses: actions/checkout@{SHA}
                    """)
                self.assertEqual(run_checker({"a.yml": text}).returncode, 1)

    def test_flow_style_read_only_permissions_pass(self) -> None:
        text = PR_WORKFLOW_HEADER + job(f"""\
            a:
              runs-on: ubuntu-latest
              permissions: {{contents: read}}
              steps:
                - uses: actions/checkout@{SHA}
            """)
        self.assertEqual(run_checker({"a.yml": text}).returncode, 0)

    def test_flow_style_workflow_permissions_are_parsed(self) -> None:
        text = f"""\
            name: Synthetic
            on:
              pull_request:
            permissions: {{contents: write}}
            jobs:
              a:
                runs-on: ubuntu-latest
                steps:
                  - uses: actions/checkout@{SHA}
            """
        self.assertEqual(run_checker({"a.yml": text}).returncode, 1)

    def test_pull_request_workflow_without_top_level_permissions_fails(self) -> None:
        text = f"""\
            name: Synthetic
            on:
              pull_request:
            jobs:
              a:
                runs-on: ubuntu-latest
                steps:
                  - uses: actions/checkout@{SHA}
            """
        proc = run_checker({"a.yml": text})
        self.assertEqual(proc.returncode, 1)
        self.assertIn("top-level permissions", proc.stderr)

    def test_non_pull_request_workflow_without_permissions_is_not_flagged(self) -> None:
        text = f"""\
            name: Synthetic
            on:
              push:
            jobs:
              a:
                runs-on: ubuntu-latest
                steps:
                  - uses: actions/checkout@{SHA}
            """
        self.assertEqual(run_checker({"a.yml": text}).returncode, 0)

    def test_mixed_not_equal_and_equal_conditions_stay_exposed(self) -> None:
        for condition in (
            "github.event_name != 'pull_request' || github.event_name == 'pull_request'",
            "github.event_name != 'pull_request' || github.ref == 'refs/heads/main'",
            "github.event_name == 'pull_request' || github.event_name == 'push'",
            "(github.event_name != 'pull_request')",
        ):
            with self.subTest(condition):
                self.assertEqual(run_checker({"a.yml": self.pr_job(condition)}).returncode, 1)

    def test_conjunctions_and_event_disjunctions_exclude_pull_requests(self) -> None:
        for condition in (
            "always() && github.event_name != 'pull_request'",
            "github.event_name == 'push' && github.ref == 'refs/heads/main'",
            "github.event_name == 'schedule' || github.event_name == 'workflow_dispatch'",
            "github.event_name != 'pull_request' && github.ref == 'refs/heads/main'",
        ):
            with self.subTest(condition):
                proc = run_checker({"a.yml": self.pr_job(condition)})
                self.assertEqual(proc.returncode, 0, proc.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)

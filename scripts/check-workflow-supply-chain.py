#!/usr/bin/env python3
"""Check GitHub workflow supply-chain and pull-request exposure policy (task Q5).

Rules, all fail-closed:

1. Every non-local ``uses:`` is pinned by a full 40-character commit SHA.
2. Every container image (``docker run`` in a workflow step, ``docker://`` actions, Dockerfile
   ``FROM``) is pinned by ``@sha256:`` digest, or is listed in the governed ledger
   (``docs/security/supply-chain-pins.toml``) with an owner, a reason and an unexpired date.
   Ledger entries that no longer match anything are errors, so the ledger cannot rot.
3. A job that can run on a ``pull_request`` gets no write permission and uses no secret other
   than the automatic ``GITHUB_TOKEN``. Measurement jobs are read-only; publication/comment jobs
   must be excluded from pull requests with an ``if:`` on ``github.event_name``.
4. ``pull_request_target`` is never used.

The parser is deliberately line based (no YAML dependency) and supports the block style used in
this repository.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from datetime import date
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - python < 3.11
    import tomli as tomllib  # type: ignore[no-redef]

SHA_REF = re.compile(r"^[^@\s]+@[0-9a-f]{40}$")
DIGEST = re.compile(r"@sha256:[0-9a-f]{64}$")
SECRET = re.compile(r"secrets\.([A-Za-z0-9_]+)")
AUTO_SECRETS = frozenset({"GITHUB_TOKEN"})
WRITE_VALUES = frozenset({"write", "write-all"})
DOCKER_VALUE_OPTIONS = frozenset(
    {"-v", "--volume", "-e", "--env", "-w", "--workdir", "-u", "--user", "--name",
     "--entrypoint", "--platform", "-p", "--publish", "--network", "-l", "--label",
     "--env-file", "--mount", "--add-host", "--cap-add", "--cap-drop", "--security-opt"}
)


@dataclass
class Job:
    name: str
    lines: list[str]
    condition: str = ""
    write_permissions: list[str] = field(default_factory=list)

    @property
    def text(self) -> str:
        return "\n".join(self.lines)


@dataclass
class Workflow:
    path: Path
    triggers: set[str]
    workflow_writes: list[str]
    jobs: list[Job]
    lines: list[str]
    has_top_permissions: bool = True


def strip_comment(line: str) -> str:
    return re.sub(r"\s+#.*$", "", line).rstrip()


def parse_permissions(lines: list[str], indent: int) -> list[str]:
    """Return write-level permission entries declared at ``indent`` in ``lines``."""

    writes: list[str] = []
    for position, raw in enumerate(lines):
        line = strip_comment(raw)
        match = re.match(rf"^ {{{indent}}}permissions:\s*(.*)$", line)
        if not match:
            continue
        inline = match.group(1).strip()
        if inline.startswith("{"):
            for pair in inline.strip("{} ").split(","):
                key, _, value = pair.partition(":")
                if value.strip().strip("'\"") in WRITE_VALUES:
                    writes.append(f"{key.strip()}: {value.strip()}")
            continue
        if inline:
            if inline in WRITE_VALUES:
                writes.append(f"permissions: {inline}")
            continue
        for follower in lines[position + 1 :]:
            body = strip_comment(follower)
            if not body.strip():
                continue
            if len(body) - len(body.lstrip()) <= indent:
                break
            entry = re.match(r"^\s+([A-Za-z-]+):\s*(\S+)$", body)
            if entry and entry.group(2) in WRITE_VALUES:
                writes.append(f"{entry.group(1)}: {entry.group(2)}")
    return writes


def parse_workflow(path: Path) -> Workflow:
    lines = path.read_text().splitlines()
    triggers: set[str] = set()
    in_on = False
    jobs_at: int | None = None
    for index, raw in enumerate(lines):
        line = strip_comment(raw)
        if re.match(r"^(on|'on'|\"on\"):\s*(.*)$", line):
            inline = re.match(r"^(?:on|'on'|\"on\"):\s*(.+)$", line)
            if inline:
                triggers |= {t.strip(" []'\"") for t in inline.group(1).split(",")}
                in_on = False
            else:
                in_on = True
            continue
        if re.match(r"^[A-Za-z_-]+:", line):
            in_on = False
            if line.startswith("jobs:"):
                jobs_at = index
        elif in_on:
            key = re.match(r"^  ([A-Za-z_]+):", line)
            if key:
                triggers.add(key.group(1))
    top_level = lines[:jobs_at] if jobs_at is not None else lines
    workflow_writes = parse_permissions(top_level, 0)
    has_top_permissions = any(
        re.match(r"^permissions:", strip_comment(line)) for line in top_level
    )
    jobs: list[Job] = []
    if jobs_at is not None:
        current: Job | None = None
        for raw in lines[jobs_at + 1 :]:
            header = re.match(r"^  ([A-Za-z0-9_-]+):\s*$", raw)
            if header:
                current = Job(header.group(1), [])
                jobs.append(current)
                continue
            if current is not None:
                current.lines.append(raw)
        for job in jobs:
            condition = next(
                (
                    re.match(r"^    if:\s*(.*)$", strip_comment(line)).group(1)  # type: ignore[union-attr]
                    for line in job.lines
                    if re.match(r"^    if:\s*", strip_comment(line))
                ),
                "",
            )
            job.condition = condition
            job.write_permissions = parse_permissions(job.lines, 4)
    return Workflow(path, triggers, workflow_writes, jobs, lines, has_top_permissions)


def excludes_pull_request(condition: str) -> bool:
    """True only when the job `if:` provably cannot run on a pull_request event.

    The condition is split on `||`; every alternative must itself exclude pull_request, either with
    `!= 'pull_request'` or by requiring `github.event_name == '<other event>'`. An alternative that
    mentions `== 'pull_request'` is exposed. Parentheses (beyond status functions such as
    `always()`) are not analysed, so they are treated as exposed.
    """

    if not condition:
        return False
    flat = re.sub(r"\b[A-Za-z]+\(\)", "", condition)
    if "(" in flat or ")" in flat:
        return False
    for alternative in flat.split("||"):
        if re.search(r"==\s*['\"]pull_request['\"]", alternative):
            return False
        negated = re.search(r"!=\s*['\"]pull_request['\"]", alternative)
        other_event = re.search(r"github\.event_name\s*==\s*['\"](?!pull_request)", alternative)
        if not (negated or other_event):
            return False
    return True


def join_command(lines: list[str], start: int) -> str:
    parts: list[str] = []
    for raw in lines[start:]:
        text = raw.strip()
        continued = text.endswith("\\")
        parts.append(text.rstrip("\\").strip())
        if not continued:
            break
    return " ".join(parts)


def docker_run_image(command: str) -> str | None:
    expression_free = re.sub(r"\$\{\{.*?\}\}", "EXPR", command)
    tokens = expression_free.replace('"', " ").split()
    try:
        position = tokens.index("run", tokens.index("docker")) + 1
    except ValueError:
        return None
    while position < len(tokens):
        token = tokens[position]
        if token.startswith("-"):
            position += 2 if token in DOCKER_VALUE_OPTIONS else 1
            continue
        return token
    return None


def workflow_images(workflow: Workflow) -> list[str]:
    images: list[str] = []
    for index, raw in enumerate(workflow.lines):
        if re.search(r"\bdocker\s+run\b", raw) and not raw.strip().startswith("#"):
            image = docker_run_image(join_command(workflow.lines, index))
            if image:
                images.append(image)
        uses = re.match(r"^\s*-?\s*uses:\s*docker://(\S+)", strip_comment(raw))
        if uses:
            images.append(uses.group(1))
    return images


def dockerfile_images(path: Path) -> list[str]:
    images: list[str] = []
    aliases: set[str] = set()
    for raw in path.read_text().splitlines():
        match = re.match(r"^FROM\s+(?:--\S+\s+)*(\S+)(?:\s+AS\s+(\S+))?", raw.strip(), re.IGNORECASE)
        if not match:
            continue
        image, alias = match.group(1), match.group(2)
        if image.lower() != "scratch" and image not in aliases:
            images.append(image)
        if alias:
            aliases.add(alias)
    return images


def load_ledger(path: Path, today: date, errors: list[str]) -> dict[str, date]:
    try:
        with path.open("rb") as handle:
            data = tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError) as exc:
        errors.append(f"ledger {path} unreadable: {exc}")
        return {}
    entries: dict[str, date] = {}
    for index, record in enumerate(data.get("unpinned_images", []), start=1):
        image = record.get("image") if isinstance(record, dict) else None
        problems = [
            key
            for key in ("image", "reason", "owner")
            if not (isinstance(record, dict) and isinstance(record.get(key), str) and record[key].strip())
        ]
        expires = record.get("expires") if isinstance(record, dict) else None
        if not isinstance(expires, date):
            problems.append("expires")
        if problems:
            errors.append(f"ledger entry #{index} ({image}) is missing: {', '.join(problems)}")
            continue
        entries[image] = expires
    return entries


def check_pins(workflow: Workflow, errors: list[str]) -> None:
    for raw in workflow.lines:
        match = re.match(r"^\s*-?\s*uses:\s*(\S+)", strip_comment(raw))
        if not match:
            continue
        ref = match.group(1).strip("'\"")
        if ref.startswith("./") or ref.startswith("docker://"):
            continue
        if not SHA_REF.match(ref):
            errors.append(f"{workflow.path.name}: action not pinned by commit SHA: {ref}")


def check_exposure(workflow: Workflow, errors: list[str]) -> None:
    name = workflow.path.name
    if "pull_request_target" in workflow.triggers:
        errors.append(f"{name}: pull_request_target is forbidden")
    if "pull_request" not in workflow.triggers:
        return
    if not workflow.has_top_permissions:
        errors.append(
            f"{name}: pull_request workflow has no top-level permissions (token defaults may be write)"
        )
    for entry in workflow.workflow_writes:
        errors.append(f"{name}: workflow-level write permission with a pull_request trigger ({entry})")
    for job in workflow.jobs:
        if excludes_pull_request(job.condition):
            continue
        for entry in job.write_permissions:
            errors.append(f"{name}: job {job.name} can run on pull_request with write permission ({entry})")
        for secret in sorted(set(SECRET.findall(job.text)) - AUTO_SECRETS):
            errors.append(f"{name}: job {job.name} can run on pull_request and uses secret {secret}")


def check_images(
    images: list[tuple[str, str]], ledger: dict[str, date], today: date, errors: list[str]
) -> None:
    used: set[str] = set()
    for source, image in images:
        if DIGEST.search(image):
            continue
        if image in ledger:
            used.add(image)
            if ledger[image] < today:
                errors.append(f"{source}: image {image} is unpinned and its ledger entry expired {ledger[image]}")
            continue
        errors.append(f"{source}: image not pinned by digest and not in the ledger: {image}")
    for image in sorted(set(ledger) - used):
        errors.append(f"ledger entry matches nothing (remove it or restore the image): {image}")


def run_checks(workflows_dir: Path, ledger_path: Path, dockerfile: Path | None, today: date) -> list[str]:
    errors: list[str] = []
    ledger = load_ledger(ledger_path, today, errors)
    paths = sorted([*workflows_dir.glob("*.yml"), *workflows_dir.glob("*.yaml")])
    if not paths:
        return [f"no workflows found in {workflows_dir}"]
    images: list[tuple[str, str]] = []
    for path in paths:
        workflow = parse_workflow(path)
        check_pins(workflow, errors)
        check_exposure(workflow, errors)
        images.extend((path.name, image) for image in workflow_images(workflow))
    if dockerfile is not None:
        images.extend((dockerfile.name, image) for image in dockerfile_images(dockerfile))
    check_images(images, ledger, today, errors)
    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--workflows", type=Path, default=Path(".github/workflows"))
    parser.add_argument("--ledger", type=Path, default=Path("docs/security/supply-chain-pins.toml"))
    parser.add_argument("--dockerfile", type=Path, default=None)
    parser.add_argument("--today", type=date.fromisoformat, default=None)
    args = parser.parse_args(argv)
    dockerfile = args.dockerfile
    if dockerfile is None and args.workflows == Path(".github/workflows") and Path("Dockerfile").is_file():
        dockerfile = Path("Dockerfile")
    errors = run_checks(args.workflows, args.ledger, dockerfile, args.today or date.today())
    if errors:
        print("workflow supply-chain check failed:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1
    print("workflow supply-chain: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

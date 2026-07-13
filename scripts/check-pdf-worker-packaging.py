#!/usr/bin/env python3
from __future__ import annotations

import argparse
import ast
from pathlib import Path


# CI still builds without --locked; the release workflow requires a locked graph.
CI_RELEASE_BUILD = (
    "cargo build --release --target ${{ matrix.target }} --features pdf "
    "--bin engram-server --bin engram-cli --bin engram-pdf-worker"
)
RELEASE_BUILD = (
    "cargo build --locked --release --target ${{ matrix.target }} --features pdf "
    "--bin engram-server --bin engram-cli --bin engram-pdf-worker"
)
# Back-compat alias used by unit fixtures that still refer to RELEASE_BUILD.
# Prefer the explicit CI_RELEASE_BUILD / RELEASE_BUILD names for new checks.
RELEASE_ARCHIVE = 'tar -czf "../../../${ARCHIVE}" engram-server engram-cli engram-pdf-worker'
CI_WORKER_PATH = "target/${{ matrix.target }}/release/engram-pdf-worker"
HOMEBREW_UPDATE_COMMAND = "python3 ../engram-source/.github/scripts/ensure-pdf-worker-homebrew.py Formula/engram.rb"
VERSION_GUARD = 'if [[ ! "${INPUT_VERSION}" =~ ^v[0-9]+\\.[0-9]+\\.[0-9]+$ ]]; then'
CREATE_BUNDLE = "scripts/verify-release-artifacts.sh \\"
SMOKE_BINARIES = "scripts/test-release-binary.sh \\"
LINUX_ONLY_WORKER_SHELL = '[[ "$target" == *-linux-* ]] && binaries+=(engram-pdf-worker)'
LINUX_ONLY_WORKER_SMOKE = "[[ \"$target\" == *-linux-* ]] && expected+=(engram-pdf-worker)"


def read_contract(root: Path, relative: str, missing: list[str]) -> str | None:
    try:
        return (root / relative).read_text(encoding="utf-8")
    except (OSError, UnicodeError) as error:
        missing.append(f"{relative}: unreadable: {error}")
        return None


def workflow_step(text: str, name: str) -> str | None:
    lines = text.splitlines()
    marker = f"- name: {name}"
    for index, line in enumerate(lines):
        if line.strip() != marker:
            continue
        indent = len(line) - len(line.lstrip())
        body: list[str] = []
        for candidate in lines[index + 1 :]:
            candidate_indent = len(candidate) - len(candidate.lstrip())
            if candidate_indent == indent and candidate.lstrip().startswith("-"):
                break
            if not candidate.lstrip().startswith("#"):
                body.append(candidate)
        return "\n".join(body)
    return None


def assignment(section: str, key: str):
    prefix = f"{key} ="
    for line in section.splitlines():
        if line.strip().startswith(prefix):
            return ast.literal_eval(line.split("=", 1)[1].strip())
    return None


def toml_section(text: str, header: str) -> str | None:
    marker = f"[{header}]"
    lines = text.splitlines()
    for index, line in enumerate(lines):
        if line.strip() != marker:
            continue
        body: list[str] = []
        for candidate in lines[index + 1 :]:
            if candidate.strip().startswith("["):
                break
            if not candidate.lstrip().startswith("#"):
                body.append(candidate)
        return "\n".join(body)
    return None


def cargo_worker_block(text: str) -> str | None:
    for block in text.split("[[bin]]")[1:]:
        if assignment(block, "name") == "engram-pdf-worker":
            return block.split("[[", 1)[0]
    return None


def require_step_lines(
    missing: list[str],
    relative: str,
    text: str,
    name: str,
    expected: tuple[str, ...],
    expected_condition: str | None = None,
) -> None:
    step = workflow_step(text, name)
    if step is None:
        missing.append(f"{relative}: missing step {name}")
        return
    lines = {line.rstrip() for line in step.splitlines() if line.strip()}
    direct_conditions = {line for line in lines if line.startswith("        if:")}
    if expected_condition is None and direct_conditions:
        missing.append(f"{relative}: step {name}: must not be conditional")
    elif expected_condition is not None and direct_conditions != {expected_condition}:
        missing.append(f"{relative}: step {name}: condition must be {expected_condition}")
    if "        continue-on-error: true" in lines:
        missing.append(f"{relative}: step {name}: must fail closed")
    for required in expected:
        if required not in lines:
            missing.append(f"{relative}: step {name}: {required}")


def require_script_contains(missing: list[str], relative: str, text: str | None, needle: str) -> None:
    if text is None:
        return
    if needle not in text:
        missing.append(f"{relative}: missing required contract line: {needle}")


def missing_contracts(root: Path) -> list[str]:
    missing: list[str] = []
    release_path = ".github/workflows/release.yml"
    release = read_contract(root, release_path, missing)
    if release is not None:
        require_step_lines(
            missing,
            release_path,
            release,
            "Validate event, ref, and SHA before any write",
            (f"          {VERSION_GUARD}",),
        )
        require_step_lines(
            missing,
            release_path,
            release,
            "Build release binaries",
            (f"        run: {RELEASE_BUILD}",),
        )
        require_step_lines(
            missing,
            release_path,
            release,
            "Package deterministic archive and supply-chain evidence",
            (
                f"          {CREATE_BUNDLE}",
                f"          {SMOKE_BINARIES}",
            ),
        )
        require_step_lines(
            missing,
            release_path,
            release,
            "Update the formula from verified checksums",
            (f"          {HOMEBREW_UPDATE_COMMAND}",),
        )

    verifier_path = "scripts/verify-release-artifacts.sh"
    verifier = read_contract(root, verifier_path, missing)
    require_script_contains(missing, verifier_path, verifier, LINUX_ONLY_WORKER_SHELL)
    require_script_contains(missing, verifier_path, verifier, "engram-pdf-worker")

    smoke_path = "scripts/test-release-binary.sh"
    smoke = read_contract(root, smoke_path, missing)
    require_script_contains(missing, smoke_path, smoke, LINUX_ONLY_WORKER_SMOKE)
    require_script_contains(missing, smoke_path, smoke, "engram-pdf-worker)")

    ci_path = ".github/workflows/ci.yml"
    ci = read_contract(root, ci_path, missing)
    if ci is not None:
        require_step_lines(missing, ci_path, ci, "Build release", (f"        run: {CI_RELEASE_BUILD}",))
        require_step_lines(
            missing,
            ci_path,
            ci,
            "Upload PDF worker artifact",
            (f"            {CI_WORKER_PATH}",),
            "        if: contains(matrix.target, '-linux-')",
        )

    cargo_path = "Cargo.toml"
    cargo = read_contract(root, cargo_path, missing)
    if cargo is not None:
        try:
            block = cargo_worker_block(cargo)
            if block is None:
                missing.append(f"{cargo_path}: missing engram-pdf-worker binary")
            else:
                if assignment(block, "path") != "src/bin/pdf_worker.rs":
                    missing.append(f"{cargo_path}: worker path must be src/bin/pdf_worker.rs")
                if assignment(block, "required-features") != ["pdf"]:
                    missing.append(f"{cargo_path}: worker required-features must be [pdf]")
        except (SyntaxError, ValueError) as error:
            missing.append(f"{cargo_path}: invalid contract value: {error}")

    inventory_path = "docs/contracts/advertised-surfaces.toml"
    inventory = read_contract(root, inventory_path, missing)
    if inventory is not None:
        try:
            cli = toml_section(inventory, "cli")
            if cli is None:
                missing.append(f"{inventory_path}: missing [cli]")
            else:
                binaries = assignment(cli, "binaries")
                if not isinstance(binaries, list) or "engram-pdf-worker" not in binaries:
                    missing.append(f"{inventory_path}: [cli].binaries missing engram-pdf-worker")
                expected = {
                    "pdf_worker_distribution": "sibling-binary",
                    "pdf_worker_supported_platforms": ["linux"],
                    "pdf_worker_unsupported_platform_behavior": "fail-closed",
                    "pdf_worker_docker_support": "not-advertised",
                }
                for key, value in expected.items():
                    if assignment(cli, key) != value:
                        missing.append(f"{inventory_path}: [cli].{key} must be {value!r}")
        except (SyntaxError, ValueError) as error:
            missing.append(f"{inventory_path}: invalid contract value: {error}")
    return missing


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    args = parser.parse_args()
    missing = missing_contracts(args.root)
    if missing:
        print("PDF worker packaging contract violations:")
        for entry in missing:
            print(f"- {entry}")
        return 1
    print("OK PDF worker packaging contract")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

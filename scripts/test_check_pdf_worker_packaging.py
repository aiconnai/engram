#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("check-pdf-worker-packaging.py")
SPEC = importlib.util.spec_from_file_location("pdf_worker_packaging", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


RELEASE = "\n".join(
    (
        "jobs:",
        "  build:",
        "    steps:",
        "      - name: Validate event, ref, and SHA before any write",
        "        run: |",
        f"          {MODULE.VERSION_GUARD}",
        "            exit 1",
        "          fi",
        "      - name: Build release binaries",
        f"        run: {MODULE.RELEASE_BUILD}",
        "      - name: Package deterministic archive and supply-chain evidence",
        "        run: |",
        f"          {MODULE.CREATE_BUNDLE}",
        "            --create-bundle \\",
        f"          {MODULE.SMOKE_BINARIES}",
        "            --artifact-dir release-bundle \\",
        "      - name: Update the formula from verified checksums",
        "        run: |",
        f"          {MODULE.HOMEBREW_UPDATE_COMMAND}",
    )
)
CI = "\n".join(
    (
        "jobs:",
        "  release:",
        "    steps:",
        "      - name: Build release",
        f"        run: {MODULE.CI_RELEASE_BUILD}",
        "      - name: Upload PDF worker artifact",
        "        if: contains(matrix.target, '-linux-')",
        "        with:",
        "          path: |",
        f"            {MODULE.CI_WORKER_PATH}",
    )
)
CARGO = "\n".join(
    (
        "[[bin]]",
        'name = "engram-pdf-worker"',
        'path = "src/bin/pdf_worker.rs"',
        'required-features = ["pdf"]',
    )
)
INVENTORY = "\n".join(
    (
        "[cli]",
        'binaries = ["engram-server", "engram-cli", "engram-pdf-worker"]',
        'pdf_worker_distribution = "sibling-binary"',
        'pdf_worker_supported_platforms = ["linux"]',
        'pdf_worker_unsupported_platform_behavior = "fail-closed"',
        'pdf_worker_docker_support = "not-advertised"',
    )
)
VERIFY_SCRIPT = "\n".join(
    (
        "binaries=(engram-server engram-cli)",
        f"{MODULE.LINUX_ONLY_WORKER_SHELL}",
        "archive_members+=(engram-pdf-worker)",
    )
)
SMOKE_SCRIPT = "\n".join(
    (
        "expected=(engram-server engram-cli)",
        f"{MODULE.LINUX_ONLY_WORKER_SMOKE}",
        "    engram-pdf-worker)",
        "      # protocol smoke",
    )
)


class PdfWorkerPackagingContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp_dir = tempfile.TemporaryDirectory()
        self.root = Path(self.temp_dir.name)
        for relative, text in {
            ".github/workflows/release.yml": RELEASE,
            ".github/workflows/ci.yml": CI,
            "Cargo.toml": CARGO,
            "docs/contracts/advertised-surfaces.toml": INVENTORY,
            "scripts/verify-release-artifacts.sh": VERIFY_SCRIPT,
            "scripts/test-release-binary.sh": SMOKE_SCRIPT,
        }.items():
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8")

    def tearDown(self) -> None:
        self.temp_dir.cleanup()

    def test_complete_distribution_contract_passes(self) -> None:
        self.assertEqual(MODULE.missing_contracts(self.root), [])

    def test_commented_worker_build_does_not_satisfy_release_step(self) -> None:
        release = self.root / ".github/workflows/release.yml"
        release.write_text(
            release.read_text(encoding="utf-8").replace(
                f"run: {MODULE.RELEASE_BUILD}",
                f"run: cargo build --release\n        # {MODULE.RELEASE_BUILD}",
                1,
            ),
            encoding="utf-8",
        )
        self.assertTrue(any("Build release binaries" in item for item in MODULE.missing_contracts(self.root)))

    def test_worker_build_in_unrelated_step_does_not_satisfy_release_step(self) -> None:
        release = self.root / ".github/workflows/release.yml"
        release.write_text(
            release.read_text(encoding="utf-8").replace(
                f"run: {MODULE.RELEASE_BUILD}",
                f"run: cargo build --release\n      - name: Unrelated\n        run: {MODULE.RELEASE_BUILD}",
                1,
            ),
            encoding="utf-8",
        )
        self.assertTrue(any("Build release binaries" in item for item in MODULE.missing_contracts(self.root)))

    def test_echoed_worker_build_does_not_satisfy_release_step(self) -> None:
        release = self.root / ".github/workflows/release.yml"
        release.write_text(
            release.read_text(encoding="utf-8").replace(
                f"run: {MODULE.RELEASE_BUILD}",
                f"run: echo '{MODULE.RELEASE_BUILD}'",
                1,
            ),
            encoding="utf-8",
        )
        self.assertTrue(any("Build release binaries" in item for item in MODULE.missing_contracts(self.root)))

    def test_disabled_worker_build_fails(self) -> None:
        release = self.root / ".github/workflows/release.yml"
        release.write_text(
            release.read_text(encoding="utf-8").replace(
                "      - name: Build release binaries\n",
                "      - name: Build release binaries\n        if: false\n",
                1,
            ),
            encoding="utf-8",
        )
        self.assertTrue(any("must not be conditional" in item for item in MODULE.missing_contracts(self.root)))

    def test_continue_on_error_worker_build_fails(self) -> None:
        release = self.root / ".github/workflows/release.yml"
        release.write_text(
            release.read_text(encoding="utf-8").replace(
                "      - name: Build release binaries\n",
                "      - name: Build release binaries\n        continue-on-error: true\n",
                1,
            ),
            encoding="utf-8",
        )
        self.assertTrue(any("must fail closed" in item for item in MODULE.missing_contracts(self.root)))

    def test_permissive_release_version_guard_fails(self) -> None:
        release = self.root / ".github/workflows/release.yml"
        release.write_text(
            release.read_text(encoding="utf-8").replace(
                MODULE.VERSION_GUARD,
                'if [[ ! "${INPUT_VERSION}" =~ ^v.*$ ]]; then',
            ),
            encoding="utf-8",
        )
        self.assertTrue(
            any("Validate event, ref, and SHA before any write" in item for item in MODULE.missing_contracts(self.root))
        )

    def test_macos_worker_upload_fails(self) -> None:
        ci = self.root / ".github/workflows/ci.yml"
        ci.write_text(
            ci.read_text(encoding="utf-8").replace("        if: contains(matrix.target, '-linux-')\n", ""),
            encoding="utf-8",
        )
        self.assertTrue(any("Upload PDF worker artifact" in item for item in MODULE.missing_contracts(self.root)))

    def test_worker_inventory_in_wrong_table_fails(self) -> None:
        inventory = self.root / "docs/contracts/advertised-surfaces.toml"
        inventory.write_text(
            INVENTORY.replace(
                'binaries = ["engram-server", "engram-cli", "engram-pdf-worker"]',
                'binaries = ["engram-server", "engram-cli"]',
            )
            + '\n[other]\nbinaries = ["engram-pdf-worker"]\n',
            encoding="utf-8",
        )
        self.assertTrue(any("[cli].binaries" in item for item in MODULE.missing_contracts(self.root)))

    def test_missing_file_reports_contract_error(self) -> None:
        (self.root / "Cargo.toml").unlink()
        self.assertTrue(any("Cargo.toml: unreadable" in item for item in MODULE.missing_contracts(self.root)))

    def test_malformed_cargo_value_reports_contract_error(self) -> None:
        (self.root / "Cargo.toml").write_text("[[bin]]\nname = [\n", encoding="utf-8")
        self.assertTrue(any("Cargo.toml: invalid contract value" in item for item in MODULE.missing_contracts(self.root)))

    def test_malformed_inventory_value_reports_contract_error(self) -> None:
        (self.root / "docs/contracts/advertised-surfaces.toml").write_text("[cli]\nbinaries = [\n", encoding="utf-8")
        self.assertTrue(
            any("advertised-surfaces.toml: invalid contract value" in item for item in MODULE.missing_contracts(self.root))
        )

    def test_missing_linux_only_packaging_rule_fails(self) -> None:
        verifier = self.root / "scripts/verify-release-artifacts.sh"
        verifier.write_text("binaries=(engram-server engram-cli engram-pdf-worker)\n", encoding="utf-8")
        self.assertTrue(any("verify-release-artifacts.sh" in item for item in MODULE.missing_contracts(self.root)))

    def test_homebrew_updater_removes_worker(self) -> None:
        formula = self.root / "Formula/engram.rb"
        formula.parent.mkdir(parents=True, exist_ok=True)
        formula.write_text(
            'def install\n  bin.install "engram-server"\n  bin.install "engram-cli"\n  bin.install "engram-pdf-worker"\nend\n',
            encoding="utf-8",
        )
        updater = SCRIPT.parent.parent / ".github/scripts/ensure-pdf-worker-homebrew.py"
        subprocess.run(["python3", str(updater), str(formula)], check=True)
        self.assertNotIn('bin.install "engram-pdf-worker"', formula.read_text(encoding="utf-8"))

    def test_homebrew_updater_rejects_unknown_formula_shape(self) -> None:
        formula = self.root / "Formula/engram.rb"
        formula.parent.mkdir(parents=True, exist_ok=True)
        formula.write_text('def install\n  bin.install "engram-server"\nend\n', encoding="utf-8")
        updater = SCRIPT.parent.parent / ".github/scripts/ensure-pdf-worker-homebrew.py"
        result = subprocess.run(["python3", str(updater), str(formula)], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("engram-cli install entry was not found", result.stderr)


if __name__ == "__main__":
    unittest.main()

#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import os
import shutil
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


@unittest.skipUnless(shutil.which("bash") and shutil.which("openssl"), "bash and openssl required")
class ReleaseBundleRoundTripTests(unittest.TestCase):
    def test_actual_bundle_creation_and_verification_for_each_release_target(self) -> None:
        """Exercise the workflow's real packaging entry point and its local helpers."""
        source_root = SCRIPT.parent.parent
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scripts = root / "scripts"
            scripts.mkdir()
            for name in ("verify-release-artifacts.sh", "sign-candidate-manifest.sh"):
                source = source_root / "scripts" / name
                if source.is_file():
                    shutil.copy2(source, scripts / name)
            shutil.copy2(source_root / "Cargo.lock", root / "Cargo.lock")
            binaries = root / "binaries"
            binaries.mkdir()
            for name in ("engram-server", "engram-cli", "engram-pdf-worker"):
                (binaries / name).write_text(f"fixture bytes for {name}\n", encoding="utf-8")
            scratch = root / "tmp"
            scratch.mkdir()
            env = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "TMPDIR": str(scratch)}
            sha = "1" * 40
            for target in (
                "aarch64-apple-darwin", "x86_64-apple-darwin",
                "x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu",
            ):
                with self.subTest(target=target):
                    output = root / target
                    command = ["bash", str(scripts / "verify-release-artifacts.sh")]
                    created = subprocess.run(
                        [*command, "--create-bundle", "--binary-dir", str(binaries),
                         "--output-dir", str(output), "--target", target,
                         "--version", "v0.24.0", "--sha", sha, "--ref", "refs/heads/test"],
                        cwd=root, env=env, text=True, capture_output=True, timeout=45,
                    )
                    self.assertEqual(created.returncode, 0, created.stdout + created.stderr)
                    verified = subprocess.run(
                        [*command, "--artifact-dir", str(output), "--expected-sha", sha,
                         "--target", target, "--allow-untrusted-local"],
                        cwd=root, env=env, text=True, capture_output=True, timeout=15,
                    )
                    self.assertEqual(verified.returncode, 0, verified.stdout + verified.stderr)
                    trusted = subprocess.run(
                        [*command, "--artifact-dir", str(output), "--expected-sha", sha,
                         "--target", target],
                        cwd=root, env=env, text=True, capture_output=True, timeout=15,
                    )
                    self.assertNotEqual(trusted.returncode, 0)
                    self.assertIn("missing identity-bound GitHub attestation", trusted.stderr)
                    self.assertEqual(list(scratch.iterdir()), [], "temporary signing keys leaked")

    def test_manifest_or_signature_tampering_is_rejected_and_scratch_is_cleaned(self) -> None:
        helper = SCRIPT.parent / "sign-candidate-manifest.sh"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scratch = root / "tmp"
            scratch.mkdir()
            env = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "TMPDIR": str(scratch)}
            manifest, signature = root / "manifest.json", root / "manifest.sig"
            manifest.write_text('{"commit":"test-fixture"}\n', encoding="utf-8")
            command = ["bash", str(helper)]
            signed = subprocess.run(
                [*command, "--manifest", str(manifest), "--signature", str(signature)],
                env=env, text=True, capture_output=True, timeout=45,
            )
            self.assertEqual(signed.returncode, 0, signed.stdout + signed.stderr)
            original_manifest = manifest.read_bytes()
            original_signature = signature.read_bytes()

            def verify() -> subprocess.CompletedProcess[str]:
                return subprocess.run(
                    [*command, "--verify", str(manifest), "--signature", str(signature)],
                    env=env, text=True, capture_output=True, timeout=15,
                )

            self.assertEqual(verify().returncode, 0)
            manifest.write_bytes(original_manifest + b"tampered")
            self.assertNotEqual(verify().returncode, 0)
            manifest.write_bytes(original_manifest)
            envelope = json.loads(original_signature)
            invalid = [
                {**envelope, "algorithm": "unsigned"},
                {**envelope, "manifest_sha256": "not-a-digest"},
                {**envelope, "signature_base64": "not!base64"},
                {**envelope, "signature_base64": "AA=="},
                {**envelope, "public_key_pem": 42},
                {key: value for key, value in envelope.items() if key != "public_key_pem"},
                [],
            ]
            for value in invalid:
                with self.subTest(value=value if isinstance(value, list) else list(value)):
                    signature.write_text(json.dumps(value), encoding="utf-8")
                    self.assertNotEqual(verify().returncode, 0)
                    self.assertEqual(list(scratch.iterdir()), [], "failed verification leaked scratch")
            signature.write_bytes(original_signature)
            self.assertEqual(verify().returncode, 0)
            self.assertEqual(list(scratch.iterdir()), [])

    def test_sign_candidate_manifest_key_cli_and_leading_dash_edge_cases(self) -> None:
        helper = SCRIPT.parent / "sign-candidate-manifest.sh"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scratch = root / "tmp"
            scratch.mkdir()
            env = {"PATH": os.environ.get("PATH", "/usr/bin:/bin"), "TMPDIR": str(scratch)}
            manifest = root / "manifest.json"
            signature = root / "manifest.sig"
            manifest.write_text('{"commit":"edge-cases"}\n', encoding="utf-8")
            command = ["bash", str(helper)]

            corrupt_key = root / "corrupt.pem"
            corrupt_key.write_text("not a pem key\n", encoding="utf-8")
            corrupt = subprocess.run(
                [*command, "--manifest", str(manifest), "--key", str(corrupt_key), "--signature", str(signature)],
                env=env, stdin=subprocess.DEVNULL, text=True, capture_output=True, timeout=10,
            )
            self.assertNotEqual(corrupt.returncode, 0)
            self.assertIn("failed to extract public key from signing key", corrupt.stderr)

            enc_key = root / "encrypted.pem"
            subprocess.run(
                [
                    "openssl", "genpkey", "-algorithm", "RSA", "-pkeyopt", "rsa_keygen_bits:2048",
                    "-aes-256-cbc", "-pass", "pass:secret-passphrase", "-out", str(enc_key),
                ],
                check=True, capture_output=True, text=True, timeout=15,
            )
            encrypted = subprocess.run(
                [*command, "--manifest", str(manifest), "--key", str(enc_key), "--signature", str(signature)],
                env=env, stdin=subprocess.DEVNULL, text=True, capture_output=True, timeout=10,
            )
            self.assertNotEqual(encrypted.returncode, 0)
            self.assertIn("failed to extract public key from signing key", encrypted.stderr)

            ec_key = root / "ec.pem"
            subprocess.run(
                ["openssl", "genpkey", "-algorithm", "EC", "-pkeyopt", "ec_paramgen_curve:P-256", "-out", str(ec_key)],
                check=True, capture_output=True, text=True, timeout=15,
            )
            non_rsa = subprocess.run(
                [*command, "--manifest", str(manifest), "--key", str(ec_key), "--signature", str(signature)],
                env=env, stdin=subprocess.DEVNULL, text=True, capture_output=True, timeout=10,
            )
            self.assertNotEqual(non_rsa.returncode, 0)
            self.assertIn("signing key must be RSA", non_rsa.stderr)

            for bad_args, expected_msg in (
                (["--manifest", str(manifest), "--signature"], "--signature requires a value"),
                (["--manifest", str(manifest), "--key", "--signature", str(signature)], "--key requires a value"),
            ):
                missing = subprocess.run(
                    [*command, *bad_args],
                    env=env, stdin=subprocess.DEVNULL, text=True, capture_output=True, timeout=10,
                )
                self.assertNotEqual(missing.returncode, 0)
                self.assertIn(expected_msg, missing.stderr)

            dash_dir = root / "dash"
            dash_dir.mkdir()
            (dash_dir / "-manifest.json").write_text('{"commit":"leading-dash"}\n', encoding="utf-8")
            rsa_key = dash_dir / "-key.pem"
            subprocess.run(
                ["openssl", "genpkey", "-algorithm", "RSA", "-pkeyopt", "rsa_keygen_bits:2048", "-out", str(rsa_key)],
                check=True, capture_output=True, text=True, timeout=15,
            )
            dash_signed = subprocess.run(
                [*command, "--manifest", "-manifest.json", "--key", "-key.pem", "--signature", "-manifest.sig"],
                cwd=dash_dir, env=env, stdin=subprocess.DEVNULL, text=True, capture_output=True, timeout=15,
            )
            self.assertEqual(dash_signed.returncode, 0, dash_signed.stdout + dash_signed.stderr)
            dash_verified = subprocess.run(
                [*command, "--verify", "-manifest.json", "--signature", "-manifest.sig"],
                cwd=dash_dir, env=env, stdin=subprocess.DEVNULL, text=True, capture_output=True, timeout=15,
            )
            self.assertEqual(dash_verified.returncode, 0, dash_verified.stdout + dash_verified.stderr)
            self.assertEqual(list(scratch.iterdir()), [])


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

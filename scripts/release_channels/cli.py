"""CLI entry point for the release-channel checker."""

from __future__ import annotations

import argparse
import json
import tomllib
from pathlib import Path

from release_channels.checks import (
    live_checks, print_results, self_test_failed_registry_command,
    self_test_future_timestamp, self_test_nonexistent_tag,
    self_test_parser_hardening, self_test_sdk_compatibility_required,
    self_test_sdk_package_and_range, self_test_missing_publish_with_core,
    self_test_read_only_no_pycache, self_test_timeout, self_test_wrong_matrix, static_checks,
)
from release_channels.model import DEFAULT_MATRIX, parse_matrix


def main() -> int:
    parser = argparse.ArgumentParser(description="Read-only release channel policy checker for Engram.")
    parser.add_argument("--matrix", type=Path, default=DEFAULT_MATRIX)
    parser.add_argument("--read-only", action="store_true")
    parser.add_argument("--command-timeout-seconds", type=int, default=30)
    parser.add_argument("--self-test-nonexistent-tag", action="store_true")
    parser.add_argument("--self-test-parser-hardening", action="store_true")
    parser.add_argument("--self-test-timeout", action="store_true")
    parser.add_argument("--self-test-wrong-matrix", action="store_true")
    parser.add_argument("--self-test-future-timestamp", action="store_true")
    parser.add_argument("--self-test-failed-registry-command", action="store_true")
    parser.add_argument("--self-test-sdk-compatibility-required", action="store_true")
    parser.add_argument("--self-test-sdk-package-and-range", action="store_true")
    parser.add_argument("--self-test-missing-publish-with-core", action="store_true")
    parser.add_argument("--self-test-read-only-no-pycache", action="store_true")
    args = parser.parse_args()
    if args.self_test_nonexistent_tag:
        return self_test_nonexistent_tag(args.command_timeout_seconds)
    if args.self_test_parser_hardening:
        return self_test_parser_hardening()
    if args.self_test_timeout:
        return self_test_timeout()
    if args.self_test_wrong_matrix:
        return self_test_wrong_matrix()
    if args.self_test_future_timestamp:
        return self_test_future_timestamp()
    if args.self_test_failed_registry_command:
        return self_test_failed_registry_command()
    if args.self_test_sdk_compatibility_required:
        return self_test_sdk_compatibility_required()
    if args.self_test_sdk_package_and_range:
        return self_test_sdk_package_and_range()
    if args.self_test_missing_publish_with_core:
        return self_test_missing_publish_with_core()
    if args.self_test_read_only_no_pycache:
        return self_test_read_only_no_pycache(args.command_timeout_seconds)
    try:
        policy, channels, compatibility = parse_matrix(args.matrix)
        results = static_checks(policy, channels, compatibility)
    except (KeyError, TypeError, ValueError, tomllib.TOMLDecodeError, json.JSONDecodeError) as error:
        print(f"ERROR: invalid release channel matrix: {error}")
        return 2
    if args.read_only:
        results.extend(live_checks(policy, channels, args.command_timeout_seconds))
    return print_results(results)

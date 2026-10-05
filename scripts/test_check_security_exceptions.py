#!/usr/bin/env python3
"""Tests for the governed advisory-exception horizon rule (task Q5).

An advisory exception may not outlive 90 days from the day it is evaluated: renewals need fresh
review, and an expiry far in the future is how an exception becomes permanent by accident.
"""

from __future__ import annotations

import dataclasses
import importlib.util
import sys
import unittest
from datetime import date, timedelta
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CHECKER = ROOT / "scripts" / "check-security-exceptions.py"
CONFIG = ROOT / "docs" / "security" / "advisory-exceptions.toml"
AUDIT = ROOT / ".cargo" / "audit.toml"
DENY = ROOT / "deny.toml"

SPEC = importlib.util.spec_from_file_location("check_security_exceptions", CHECKER)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)

TODAY = date(2026, 10, 5)


def validate(records, today: date = TODAY) -> None:
    MODULE.validate_records(
        records,
        MODULE.advisory_ignores(AUDIT),
        MODULE.advisory_ignores(DENY),
        MODULE.cargo_deny_ban_skips(DENY),
        today,
    )


class HorizonTests(unittest.TestCase):
    def setUp(self) -> None:
        self.records = MODULE.parse_config(CONFIG)

    def test_repository_records_are_valid_on_the_review_date(self) -> None:
        validate(self.records)

    def test_every_record_expires_within_ninety_days(self) -> None:
        for record in self.records:
            with self.subTest(record.advisory):
                self.assertLessEqual(record.expires, TODAY + timedelta(days=90))
                self.assertGreaterEqual(record.expires, TODAY)

    def test_expiry_beyond_ninety_days_is_rejected(self) -> None:
        far = dataclasses.replace(self.records[0], expires=TODAY + timedelta(days=91))
        with self.assertRaises(MODULE.CheckError) as ctx:
            validate((far, *self.records[1:]))
        self.assertIn("90 days", str(ctx.exception))

    def test_expiry_at_exactly_ninety_days_is_allowed(self) -> None:
        edge = dataclasses.replace(self.records[0], expires=TODAY + timedelta(days=90))
        validate((edge, *self.records[1:]))

    def test_expired_record_is_still_rejected(self) -> None:
        old = dataclasses.replace(self.records[0], expires=TODAY - timedelta(days=1))
        with self.assertRaises(MODULE.CheckError) as ctx:
            validate((old, *self.records[1:]))
        self.assertIn("expired", str(ctx.exception))

    def test_records_expire_after_the_review_window_closes(self) -> None:
        with self.assertRaises(MODULE.CheckError):
            validate(self.records, today=max(r.expires for r in self.records) + timedelta(days=1))


if __name__ == "__main__":
    unittest.main(verbosity=2)

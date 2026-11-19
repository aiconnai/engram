#!/usr/bin/env python3
"""Tests for the governed advisory-exception horizon rule (task Q5).

An advisory exception may not outlive 90 days from the day it is evaluated: renewals need fresh
review, and an expiry far in the future is how an exception becomes permanent by accident.
"""

from __future__ import annotations

import dataclasses
import importlib.util
import sys
import tempfile
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


def validate(records, today: date, *, as_of: date | None = None) -> None:
    MODULE.validate_records(
        records,
        MODULE.advisory_ignores(AUDIT),
        MODULE.advisory_ignores(DENY),
        MODULE.cargo_deny_ban_skips(DENY),
        today,
        as_of=as_of,
    )


class HorizonTests(unittest.TestCase):
    def setUp(self) -> None:
        self.review_date = MODULE.parse_as_of(CONFIG)
        self.records = MODULE.parse_config(CONFIG)

    def test_repository_records_are_valid_on_the_review_date(self) -> None:
        validate(self.records, self.review_date, as_of=self.review_date)

    def test_every_record_expires_within_ninety_days(self) -> None:
        for record in self.records:
            with self.subTest(record.advisory):
                self.assertLessEqual(record.expires, self.review_date + timedelta(days=90))
                self.assertGreaterEqual(record.expires, self.review_date)

    def test_expiry_beyond_ninety_days_is_rejected(self) -> None:
        far = dataclasses.replace(self.records[0], expires=self.review_date + timedelta(days=91))
        with self.assertRaises(MODULE.CheckError) as ctx:
            validate((far, *self.records[1:]), self.review_date, as_of=self.review_date)
        self.assertIn("90 days", str(ctx.exception))

    def test_expiry_at_exactly_ninety_days_is_allowed(self) -> None:
        edge = dataclasses.replace(self.records[0], expires=self.review_date + timedelta(days=90))
        validate((edge, *self.records[1:]), self.review_date, as_of=self.review_date)

    def test_expired_record_is_still_rejected(self) -> None:
        old = dataclasses.replace(self.records[0], expires=self.review_date - timedelta(days=1))
        with self.assertRaises(MODULE.CheckError) as ctx:
            validate((old, *self.records[1:]), self.review_date, as_of=self.review_date)
        self.assertIn("expired", str(ctx.exception))

    def test_records_expire_after_the_review_window_closes(self) -> None:
        with self.assertRaises(MODULE.CheckError):
            validate(
                self.records,
                max(r.expires for r in self.records) + timedelta(days=1),
                as_of=self.review_date,
            )

    def test_future_metadata_as_of_is_rejected(self) -> None:
        with self.assertRaises(MODULE.CheckError) as ctx:
            validate(self.records, self.review_date, as_of=self.review_date + timedelta(days=1))
        self.assertIn("as_of", str(ctx.exception))

    def test_missing_metadata_as_of_is_rejected(self) -> None:
        with tempfile.NamedTemporaryFile("w", encoding="utf-8", suffix=".toml", delete=False) as tmp:
            tmp.write("[metadata]\nschema_version = 1\n")
            tmp_path = Path(tmp.name)
        try:
            with self.assertRaises(MODULE.CheckError) as ctx:
                MODULE.parse_as_of(tmp_path)
            self.assertIn("as_of", str(ctx.exception))
        finally:
            tmp_path.unlink(missing_ok=True)


if __name__ == "__main__":
    unittest.main(verbosity=2)

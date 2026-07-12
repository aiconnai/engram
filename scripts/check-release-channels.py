#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# ///
# ─── How to run ───
# python3 scripts/check-release-channels.py --matrix docs/releases/channel-matrix.toml --read-only
"""Read-only release channel policy checker for Engram."""

from __future__ import annotations

import sys

sys.dont_write_bytecode = True

from release_channels.cli import main


if __name__ == "__main__":
    raise SystemExit(main())

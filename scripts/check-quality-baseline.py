#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.9"
# dependencies = []
# ///

# ─── How to run ───
# 1. Install uv (if not installed):
#      curl -LsSf https://astral.sh/uv/install.sh | sh
# 2. Run directly (no venv, no pip install needed):
#      uv run scripts/check-quality-baseline.py benches/results/benchmark_baseline.txt
# 3. Or with the repo RTK Python route:
#      rtk python3 scripts/check-quality-baseline.py benches/results/benchmark_baseline.txt
# ──────────────────

from __future__ import annotations

from quality_baseline.checker import main


if __name__ == "__main__":
    raise SystemExit(main())

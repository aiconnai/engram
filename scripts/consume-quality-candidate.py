#!/usr/bin/env python3
"""Apply the Q1 consumption rule to a candidate quality report.

    python3 scripts/consume-quality-candidate.py --report <report.json> \
        --candidate-sha <SHA> --supervisor "$ENGRAM_QUALITY_SUPERVISOR" \
        [--summary "$GITHUB_STEP_SUMMARY"]

Exit 0 accepted, 3 intact but floors not accepted yet, 1 rejected, 2 usage.
See scripts/quality_candidate/consume.py and
docs/quality/retrieval-performance-policy.md ("Consuming a report").
"""

from __future__ import annotations

import sys
from pathlib import Path

# Runs from a trusted checkout next to the candidate; never write bytecode into it.
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))

from quality_candidate.consume import main  # noqa: E402

if __name__ == "__main__":
    raise SystemExit(main())

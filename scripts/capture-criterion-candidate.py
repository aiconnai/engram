#!/usr/bin/env python3
"""Capture Criterion output marked for one candidate (input of run-quality-candidate.py).

    python3 scripts/capture-criterion-candidate.py --candidate-sha <SHA> \
        --features "$CI_REQUIRED_FEATURES" --output <captured-criterion.txt>
"""

from __future__ import annotations

import sys
from pathlib import Path

# The runner requires a clean checkout; never write bytecode into it.
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))

from quality_candidate.capture import run  # noqa: E402

if __name__ == "__main__":
    raise SystemExit(run(sys.argv[1:]))

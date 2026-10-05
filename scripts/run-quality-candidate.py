#!/usr/bin/env python3
"""Run the approved retrieval evaluation for one candidate and write a report.

    python3 scripts/run-quality-candidate.py --candidate-sha <SHA> \
        --corpus tests/fixtures/retrieval_quality/candidate_corpus.json \
        --features "$CI_REQUIRED_FEATURES" --criterion <captured-criterion.txt> \
        --output <report.json>

Produce the Criterion input with scripts/capture-criterion-candidate.py first.
In CI run both from a trusted ref with --candidate-dir <candidate checkout>,
--require-supervisor and --floors-anchor <trusted sha>; see
docs/quality/retrieval-performance-policy.md ("Trust model").
Exit 0 only when every check passed; see scripts/quality_candidate/runner.py.
"""

from __future__ import annotations

import sys
from pathlib import Path

# The runner requires a clean checkout; never write bytecode into it.
sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))

from quality_candidate.runner import run  # noqa: E402

if __name__ == "__main__":
    raise SystemExit(run(sys.argv[1:]))

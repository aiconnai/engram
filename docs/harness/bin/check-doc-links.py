#!/usr/bin/env python3
"""CLI shim for doc_links.py (offline Markdown link/anchor checker, task H6)."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from doc_links import main  # noqa: E402

if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))

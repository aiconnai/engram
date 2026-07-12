#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path


WORKER_INSTALL = 'bin.install "engram-pdf-worker"'
CLI_INSTALL = 'bin.install "engram-cli"'


def update_formula(path: Path) -> None:
    lines = path.read_text(encoding="utf-8").splitlines()
    if not any(line.strip() == CLI_INSTALL for line in lines):
        raise ValueError("engram-cli install entry was not found")
    filtered = [line for line in lines if line.strip() != WORKER_INSTALL]
    path.write_text("\n".join(filtered) + "\n", encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("formula", type=Path)
    args = parser.parse_args()
    try:
        update_formula(args.formula)
    except (OSError, UnicodeError, ValueError) as error:
        parser.error(str(error))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

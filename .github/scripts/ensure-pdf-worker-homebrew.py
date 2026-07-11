#!/usr/bin/env python3
from __future__ import annotations

import argparse
from pathlib import Path


WORKER_INSTALL = 'bin.install "engram-pdf-worker"'
CLI_INSTALL = 'bin.install "engram-cli"'


def update_formula(path: Path) -> None:
    lines = path.read_text(encoding="utf-8").splitlines()
    if any(line.strip() == WORKER_INSTALL for line in lines):
        return
    for index, line in enumerate(lines):
        if line.strip() == CLI_INSTALL:
            indent = line[: len(line) - len(line.lstrip())]
            lines.insert(index + 1, f"{indent}{WORKER_INSTALL}")
            path.write_text("\n".join(lines) + "\n", encoding="utf-8")
            return
    raise ValueError("engram-cli install entry was not found")


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

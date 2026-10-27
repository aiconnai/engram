#!/usr/bin/env python3
"""Check public-facing docs against the MCP catalog (Public Claims Gate).

Fails when a public page states a tool count that is not derived from the
catalog, repeats a banned unverified claim, or lists a tool that does not
exist (or has the wrong tier) on the featured-tools page.

Counts come from the same parser as docs/MCP_TOOLS.md
(``scripts/generate_mcp_reference.py``):

* ``total``   - every tool in the catalog (``Total tools`` in MCP_TOOLS.md);
* ``default`` - tools callable in a default ``cargo build`` (all required
  Cargo features are enabled by the ``default`` feature set);
* ``essential`` - tools in the default ``tools/list`` (Essential tier plus
  ``discover_tools``).

Run: ``python3 scripts/check-public-claims.py``
"""

from __future__ import annotations

import re
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

import generate_mcp_reference as mcp_reference  # noqa: E402

PUBLIC_FILES = (
    "README.md",
    "docs/README.md",
    "docs/index.html",
    "docs/mcp-tools.html",
    "docs/site.js",
)
FEATURED_TOOLS_PAGE = "docs/mcp-tools.html"

# A claim "N tools" where N has 2-3 digits. A leading "~" marks an approximate
# third-party figure (competitor table), which is not ours to verify here.
TOOL_COUNT_PATTERNS = (
    re.compile(r"(?<![~\d.])(\d{2,3})\+?\s+(?:active\s+|registered\s+)?(?:MCP\s+)?tools?\b", re.I),
    re.compile(
        r"(?<![~\d.])(\d{2,3})\s+(?:essential tools|in the default build|default build|of them are callable)",
        re.I,
    ),
    re.compile(r"\btools?\s*\((\d{2,3})\)", re.I),
    re.compile(r"MCP%20Server-(\d{2,3})%20Tools"),
    re.compile(r"Catalog \((\d{2,3}) Tools\)"),
    re.compile(r'stat-number">(\d{2,3})\+?</div>\s*<div class="stat-label">Active MCP Tools'),
)

# phrase -> why it cannot be published as-is
BANNED_PHRASES = {
    "sub-millisecond": "unmeasured; FTS5 p50 3.2 ms and hybrid p50 250 ms at 100K memories",
    "submillisecond": "unmeasured; FTS5 p50 3.2 ms and hybrid p50 250 ms at 100K memories",
    "native adapters": "no model-specific adapters exist in this repository",
}

CARD_RE = re.compile(
    r'<div class="tool-card" data-tier="(?P<tier>\w+)"[^>]*>.*?'
    r'<span class="tool-name">(?P<name>\w+)</span>',
    re.S,
)


@dataclass(frozen=True)
class CatalogCounts:
    total: int
    default: int
    essential: int

    @property
    def allowed(self) -> set[int]:
        return {self.total, self.default, self.essential}


def default_feature_closure(cargo: dict) -> set[str]:
    features = cargo.get("features", {})
    seen: set[str] = set()
    pending = list(features.get("default", []))
    while pending:
        feature = pending.pop()
        if feature in seen:
            continue
        seen.add(feature)
        for dependency in features.get(feature, []):
            if not dependency.startswith("dep:") and "/" not in dependency:
                pending.append(dependency)
    return seen


def catalog_counts(tools: list[mcp_reference.Tool], enabled: set[str]) -> CatalogCounts:
    return CatalogCounts(
        total=len(tools),
        default=sum(1 for t in tools if all(f in enabled for f in t.required_features)),
        essential=sum(1 for t in tools if t.tier == "essential" or t.name == "discover_tools"),
    )


def check_text(path: str, text: str, counts: CatalogCounts) -> list[str]:
    failures: list[str] = []
    stale = sorted(
        {
            int(match.group(1))
            for pattern in TOOL_COUNT_PATTERNS
            for match in pattern.finditer(text)
            if int(match.group(1)) not in counts.allowed
        }
    )
    for number in stale:
        failures.append(
            f"{path}: states {number} tools; catalog says total={counts.total}, "
            f"default build={counts.default}, default tools/list={counts.essential}"
        )
    lowered = text.lower()
    for phrase, reason in BANNED_PHRASES.items():
        if phrase in lowered:
            failures.append(f"{path}: contains banned claim {phrase!r} ({reason})")
    return failures


def check_featured_tools(
    path: str, text: str, tools: list[mcp_reference.Tool]
) -> list[str]:
    by_name = {tool.name: tool for tool in tools}
    failures: list[str] = []
    for card in CARD_RE.finditer(text):
        name, tier = card.group("name"), card.group("tier")
        tool = by_name.get(name)
        if tool is None:
            failures.append(f"{path}: lists tool {name!r}, which is not in the catalog")
        elif tool.tier != tier:
            failures.append(f"{path}: {name} is tier {tier!r} but the catalog says {tool.tier!r}")
    return failures


def check_repo(root: Path = ROOT) -> list[str]:
    tools = mcp_reference.parse_tools(mcp_reference.DEFAULT_SOURCE)
    cargo = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    counts = catalog_counts(tools, default_feature_closure(cargo))

    failures: list[str] = []
    for relative in PUBLIC_FILES:
        text = (root / relative).read_text(encoding="utf-8")
        failures.extend(check_text(relative, text, counts))
        if relative == FEATURED_TOOLS_PAGE:
            failures.extend(check_featured_tools(relative, text, tools))
    return failures


def main() -> int:
    try:
        failures = check_repo()
    except (OSError, ValueError, tomllib.TOMLDecodeError) as exc:
        print(f"public-claims: invalid input: {exc}", file=sys.stderr)
        return 2
    if failures:
        print("public-claims: FAIL")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print("public-claims: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

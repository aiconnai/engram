"""Reviewed candidate floors, keyed by corpus SHA256, tamper-evident and shrink-only.

A floors entry binds a corpus hash to per-mode metric floors and a review record.
Guards against silent floor edits:

1. ``entry_digest`` covers every other field of the entry. Editing a number
   without ``seal`` (which a reviewer sees in the diff) fails. This is
   tamper-evidence for accidents, not authentication: the candidate controls the
   file, so it proves nothing about review.
2. The trusted anchor comes from the **supervisor** (``--floors-anchor <sha>``),
   never from the candidate's own file. The anchor must be a strict ancestor of
   the candidate, hold an entry for the same corpus hash, and current floors must
   be >= the anchored ones. Only then are floors ``anchored``; they are
   ``accepted`` only when the anchored entry's review status is
   ``accepted-independent-review``. A forged ``anchor_revision`` inside the
   candidate's floors file is ignored.
3. The v1 smoke corpus entry must not be lower than ``docs/quality/budgets.json``
   retrieval floors (cross-lane reconciliation, never an automatic relaxation).

Seal after an intentional, reviewed edit:
``cd scripts && python3 -m quality_candidate.floors seal ../docs/quality/candidate-floors.json``
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Mapping, Optional

from .common import (
    FLOORS_SCHEMA,
    METRICS,
    MODES,
    SHA256_RE,
    SHA_RE,
    CandidateError,
    canonical_json,
    loads_strict,
    require_metric_map,
    sha256_bytes,
)
from .gitstate import is_ancestor, show_at

ENTRY_FIELDS = (
    "name",
    "version",
    "corpus_path",
    "corpus_sha256",
    "deterministic_seed",
    "memory_count",
    "query_count",
    "floors",
    "review",
    "entry_digest",
)
REVIEW_FIELDS = ("status", "label_review", "justification")
ACCEPTED_REVIEW_STATUS = "accepted-independent-review"


def entry_digest(entry: Mapping[str, Any]) -> str:
    body = {key: value for key, value in entry.items() if key != "entry_digest"}
    return sha256_bytes(canonical_json(body).encode("utf-8"))


def parse_floors(text: str) -> Dict[str, Any]:
    doc = loads_strict(text, "candidate floors")
    if not isinstance(doc, dict) or doc.get("schema_version") != FLOORS_SCHEMA:
        raise CandidateError(f"candidate floors: schema_version must be {FLOORS_SCHEMA}")
    corpora = doc.get("corpora")
    if not isinstance(corpora, list) or not corpora:
        raise CandidateError("candidate floors: corpora must be a non-empty list")
    seen = set()
    for index, entry in enumerate(corpora):
        label = f"candidate floors.corpora[{index}]"
        if not isinstance(entry, dict):
            raise CandidateError(f"{label}: must be an object")
        missing = [field for field in ENTRY_FIELDS if field not in entry]
        if missing:
            raise CandidateError(f"{label}: missing field(s) {', '.join(missing)}")
        sha = entry["corpus_sha256"]
        if not isinstance(sha, str) or not SHA256_RE.fullmatch(sha):
            raise CandidateError(f"{label}.corpus_sha256: must be 64 lowercase hex characters")
        if sha in seen:
            raise CandidateError(f"{label}: duplicate corpus_sha256 {sha}")
        seen.add(sha)
        floors = entry["floors"]
        if not isinstance(floors, dict) or set(floors) != set(MODES):
            raise CandidateError(f"{label}.floors: must contain exactly modes {', '.join(MODES)}")
        for mode in MODES:
            require_metric_map(floors[mode], f"{label}.floors.{mode}")
        review = entry["review"]
        if not isinstance(review, dict) or any(
            not isinstance(review.get(f), str) or not review[f].strip() for f in REVIEW_FIELDS
        ):
            raise CandidateError(f"{label}.review: needs non-empty {', '.join(REVIEW_FIELDS)}")
        for count in ("memory_count", "query_count", "deterministic_seed"):
            value = entry[count]
            if isinstance(value, bool) or not isinstance(value, int) or value < 0:
                raise CandidateError(f"{label}.{count}: must be a non-negative integer")
        if entry["entry_digest"] != entry_digest(entry):
            raise CandidateError(
                f"{label}: entry_digest mismatch (floor, corpus binding or review edited "
                "without reviewed re-seal)"
            )
    return doc


def select_entry(doc: Mapping[str, Any], corpus_sha256: str) -> Dict[str, Any]:
    for entry in doc["corpora"]:
        if entry["corpus_sha256"] == corpus_sha256:
            return entry
    known = ", ".join(entry["corpus_sha256"][:12] for entry in doc["corpora"])
    raise CandidateError(
        f"corpus hash divergence: corpus sha256 {corpus_sha256} has no reviewed floors entry "
        f"(known: {known}); a changed corpus needs a new reviewed entry, not a reused floor"
    )


def assess_anchor(
    repo: Path,
    candidate_sha: str,
    entry: Mapping[str, Any],
    path: str,
    anchor: Optional[str],
) -> Dict[str, Any]:
    """Evaluate the supervisor-supplied anchor; raise only on a relaxed floor.

    Returns ``anchored``, ``accepted``, ``anchor_revision`` and a ``reason`` that
    explains every not-anchored outcome. A missing/untrusted anchor is not an
    error (the report simply says so); a floor below a valid anchor is.
    """
    result: Dict[str, Any] = {
        "anchored": False,
        "accepted": False,
        "anchor_revision": anchor,
        "reason": None,
    }
    if anchor is None:
        result["reason"] = "no supervisor --floors-anchor supplied"
        return result
    if not SHA_RE.fullmatch(anchor):
        raise CandidateError("--floors-anchor must be a full 40-character lowercase Git SHA")
    if anchor == candidate_sha:
        result["reason"] = "anchor is the candidate's own commit"
        return result
    if not is_ancestor(repo, anchor, candidate_sha):
        result["reason"] = "anchor is not an ancestor of the candidate (or is unknown here)"
        return result
    try:
        anchored_doc = parse_floors(show_at(repo, anchor, path))
    except CandidateError as exc:
        result["reason"] = f"anchor has no valid floors file: {exc}"
        return result
    for old in anchored_doc["corpora"]:
        if old["corpus_sha256"] != entry["corpus_sha256"]:
            continue
        for mode in MODES:
            for metric in METRICS:
                before = float(old["floors"][mode][metric])
                after = float(entry["floors"][mode][metric])
                if after < before:
                    raise CandidateError(
                        f"floor relaxed vs anchor {anchor[:12]}: {mode}.{metric} {before} -> {after}"
                    )
        result["anchored"] = True
        result["accepted"] = old["review"]["status"] == ACCEPTED_REVIEW_STATUS
        if not result["accepted"]:
            result["reason"] = (
                f"anchored entry review status is {old['review']['status']!r}, "
                f"not {ACCEPTED_REVIEW_STATUS!r}"
            )
        return result
    result["reason"] = "anchor holds no entry for this corpus hash"
    return result


def reconcile_with_budgets(
    entry: Mapping[str, Any], budgets: Mapping[str, Any], baseline: Mapping[str, Any]
) -> bool:
    """The v1 smoke corpus must keep floors at least as strict as budgets.json."""
    fixture = baseline.get("corpus", {}).get("fixture_path")
    if entry["corpus_path"] != fixture:
        return False
    budget_floors = require_metric_map(
        budgets.get("retrieval", {}).get("floors"), "budgets.retrieval.floors"
    )
    for metric in METRICS:
        have = float(entry["floors"]["lexical_fts5"][metric])
        if have < budget_floors[metric]:
            raise CandidateError(
                f"lane threshold relaxed: lexical_fts5.{metric} floor {have} is below "
                f"budgets.json retrieval floor {budget_floors[metric]}"
            )
    return True


def seal(path: Path) -> List[str]:
    """Recompute every entry_digest in place (reviewed edits only)."""
    doc = json.loads(path.read_text(encoding="utf-8"))
    changed = []
    for entry in doc["corpora"]:
        digest = entry_digest(entry)
        if entry.get("entry_digest") != digest:
            entry["entry_digest"] = digest
            changed.append(entry["name"])
    path.write_text(json.dumps(doc, indent=2) + "\n", encoding="utf-8")
    return changed


def main(argv: Optional[List[str]] = None) -> int:
    args = sys.argv[1:] if argv is None else argv
    if len(args) != 2 or args[0] != "seal":
        print("usage: python3 -m quality_candidate.floors seal <candidate-floors.json>", file=sys.stderr)
        return 2
    changed = seal(Path(args[1]))
    print(f"sealed {len(changed)} entr{'y' if len(changed) == 1 else 'ies'}: {', '.join(changed) or 'none changed'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

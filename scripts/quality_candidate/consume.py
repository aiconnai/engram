"""Q1 consumption rule for candidate quality reports (task Q1b).

A report from ``run-quality-candidate.py`` is accepted evidence only when all of
docs/quality/retrieval-performance-policy.md ("Consuming a report") hold:
``status == "pass"``, ``candidate.sha`` equals the SHA the supervisor checked
out, ``supervisor`` equals the job's own id and the Criterion marker's
supervisor, ``supervisor_required`` is true and ``floors.accepted`` is true.

Three outcomes keep integrity failures distinct from the expected pending state:

* ``rejected`` (exit 1): the evidence itself is failed, mismatched or malformed;
* ``not-accepted`` (exit 3): the evidence is intact but the floors are not
  accepted by independent review yet (report-only phase);
* ``accepted`` (exit 0).
"""

from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, List, Mapping, Optional, Sequence

from .common import LOCAL_SUPERVISORS, SHA_RE, CandidateError, load_json_file

ACCEPTED = "accepted"
NOT_ACCEPTED = "not-accepted"
REJECTED = "rejected"
EXIT_ACCEPTED = 0
EXIT_REJECTED = 1
EXIT_NOT_ACCEPTED = 3
EXIT_CODES = {ACCEPTED: EXIT_ACCEPTED, NOT_ACCEPTED: EXIT_NOT_ACCEPTED, REJECTED: EXIT_REJECTED}


@dataclass(frozen=True)
class Verdict:
    decision: str
    reasons: List[str] = field(default_factory=list)


def _section(report: Mapping[str, Any], key: str) -> Mapping[str, Any]:
    value = report.get(key)
    return value if isinstance(value, Mapping) else {}


def integrity_problems(
    report: Mapping[str, Any], candidate_sha: str, supervisor: str
) -> List[str]:
    """Every reason the evidence is not trustworthy for this job and candidate."""
    problems: List[str] = []
    if not SHA_RE.fullmatch(candidate_sha):
        problems.append(f"expected candidate SHA {candidate_sha!r} is not a 40-hex commit id")
    if supervisor.strip().lower() in LOCAL_SUPERVISORS:
        problems.append(f"expected supervisor {supervisor!r} is not a supervisor-owned id")
    status = report.get("status")
    if status != "pass":
        errors = report.get("errors")
        detail = "; ".join(str(e) for e in errors) if isinstance(errors, list) and errors else "none"
        problems.append(f"status is {status!r}, not 'pass' (errors: {detail})")
    if _section(report, "candidate").get("sha") != candidate_sha:
        problems.append("candidate.sha does not equal the checked-out candidate SHA")
    if report.get("supervisor") != supervisor:
        problems.append("supervisor does not equal this job's supervisor id")
    marker = _section(_section(report, "criterion"), "marker")
    if marker.get("supervisor") != supervisor:
        problems.append("criterion.marker.supervisor does not equal this job's supervisor id")
    if marker.get("candidate_sha") != candidate_sha:
        problems.append("criterion.marker.candidate_sha does not equal the candidate SHA")
    if report.get("supervisor_required") is not True:
        problems.append("supervisor_required is not true (run without --require-supervisor)")
    if not isinstance(report.get("floors"), Mapping):
        problems.append("floors section is missing")
    return problems


def assess(report: Any, *, candidate_sha: str, supervisor: str) -> Verdict:
    if not isinstance(report, Mapping):
        return Verdict(REJECTED, ["report root is not an object"])
    problems = integrity_problems(report, candidate_sha, supervisor)
    if problems:
        return Verdict(REJECTED, problems)
    floors = _section(report, "floors")
    if floors.get("accepted") is not True:
        return Verdict(
            NOT_ACCEPTED,
            [
                "floors.accepted is not true: floors are pending independent review "
                f"(anchored={floors.get('anchored')!r}, anchor_reason={floors.get('anchor_reason')!r})"
            ],
        )
    return Verdict(ACCEPTED, [])


def render_summary(verdict: Verdict, candidate_sha: str, report_path: Path) -> str:
    title = {ACCEPTED: "ACCEPTED", NOT_ACCEPTED: "NOT ACCEPTED", REJECTED: "REJECTED"}[verdict.decision]
    lines = [
        f"### Candidate quality evidence: {title}",
        "",
        f"- Candidate: `{candidate_sha}`",
        f"- Report: `{report_path.name}` (uploaded as a workflow artifact)",
        "- Phase: report-only, not a required check, until the floors are accepted by"
        " independent human review and anchored on the protected base.",
    ]
    lines += [f"- {reason}" for reason in verdict.reasons]
    return "\n".join(lines) + "\n"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--report", type=Path, required=True)
    parser.add_argument("--candidate-sha", required=True)
    parser.add_argument("--supervisor", required=True)
    parser.add_argument("--summary", type=Path, default=None, help="Markdown file to append to")
    return parser


def main(argv: Optional[Sequence[str]] = None) -> int:
    args = build_parser().parse_args(list(sys.argv[1:] if argv is None else argv))
    try:
        report: Any = load_json_file(args.report, "candidate report")
    except CandidateError as exc:
        verdict = Verdict(REJECTED, [str(exc)])
    else:
        verdict = assess(report, candidate_sha=args.candidate_sha, supervisor=args.supervisor)
    summary = render_summary(verdict, args.candidate_sha, args.report)
    print(summary, end="")
    if args.summary is not None:
        with args.summary.open("a", encoding="utf-8") as handle:
            handle.write(summary)
    return EXIT_CODES[verdict.decision]

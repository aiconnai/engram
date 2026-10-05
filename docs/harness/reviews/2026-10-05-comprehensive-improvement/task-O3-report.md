# Task O3 report — probabilistic evaluators as advisory, never gates

Status: DONE_WITH_CONCERNS (provenance unavailable; docs-only, no TDD applicable)
Worktree/branch: engram-improvement-lane-p / claude/improvement-lane-p (base df9e9c8)

## Commits (local, no push)
- b135c87 docs(quality): add advisory-only probabilistic evaluator policy
- fc5a688 docs(harness): refresh progress last commit after O3

## Files
New: docs/quality/probabilistic-evaluation-policy.md;
docs/quality/evaluations/2026-10-02-probability-review/PROVENANCE.md;
docs/quality/evaluations/holdout-case.schema.json;
docs/quality/evaluations/holdout-cases.example.jsonl (2 synthetic rows).
Edited: docs/harness/progress/2026-10-05-improvement-lane-p.md (PT-BR O3 entry),
docs/harness/progress.md (Last commit only). No H2-owned files touched.

## Policy contents
Advisory-only/never merge-enabling/never in default CI, sensors, offline lane, pre-commit; fail-closed
(defer/unknown/abstain/parse error/unavailable = no signal); metric separation (p(true), action
distribution, ordinal, confidence; no averaging); correlated/alias models not independent votes;
tests: explicit abstention, permutation, two predefined phrasings, context/truncation, model id;
per-class Brier with base rate, N, limits; no fit/eval on same sample; holdout human-labeled spec
separated from policy author; reference intake/licence before Laya/Kev/JEV; independent evaluation
for policy changes; rollback = remove advisor.
Prior experiment recorded verbatim as diagnostic history (Laya 6/8 Brier 0.21588291625; Kev 7/8
0.06903502125; 8 curated cases not holdout; Laya P(defer) 62.13-82.70% by order; JEV not executed, no
credential; jev-latest alias not independent). Hosted JEV: UNAVAILABLE (stated in section 3).

## Provenance
/tmp/engram-probability-review-2026-10-02/ does not exist (also no similar dir in /tmp). Recorded
"provenance unavailable" in PROVENANCE.md; nothing archived/hashed/reconstructed. The experiment
figures are carried from the plan text, unverified against source files (stated in policy and
PROVENANCE.md).

## Verification
- jsonschema Draft202012 check_schema on holdout-case.schema.json: ok; both example rows validate: ok.
- check-live-state.sh --progress docs/harness/progress.md: PASS (head=b135c87 before final commit;
  PASS again after fc5a688, worktree clean).
- doctor.sh: "OK harness doctor", one pre-existing WARN (no review artifact for active task).
- Arithmetic note: 82.70-62.13 = 20.57 points (stated in policy).
- No Rust touched; cargo/pre-commit hooks passed on both commits.

## NOT RUN / concerns
- No evaluator (Laya/Kev/JEV) was executed; no holdout benchmark was authored (policy specifies it; humans
  must label it). Independent evaluation/review of the policy is for the controller.
- Policy was drafted with no knowledge of Laya/Kev/JEV internals beyond the brief; it names no
  implementation details, licences or model versions.
- Schema/example fixtures are not wired to any test, CI, sensors or doctor (intentional); drift is
  not machine-checked.
- sensors.sh was not run (docs-only change; last sensors receipt untouched).

## Fix report — review round 1
Commits: 9dd07df fix(quality): require two phrasings in holdout schema; 17c6497 docs(harness): refresh progress last commit after O3 fix.
Changes:
- holdout-case.schema.json: `phrasings` added to `required` (already minItems=maxItems=2).
- Policy §7: phrasings listed as required case field; note that labeler exclusion (not policy author /
  evaluator configurer) is a process check the schema cannot enforce (schema only requires >=2 distinct
  labeler ids).
- Policy §8: `unavailable` (infrastructure/credential missing) distinct from `NOT RUN` (skipped / not
  attempted though it could have); neither is imputed or counted as pass/zero.
- Lane P log appended; progress.md Last commit -> 9dd07df.
Validation (python3, jsonschema Draft202012Validator over the schema + example jsonl):
  pass example-supported-001
  pass example-unknown-002
  no phrasings errors: ["'phrasings' is a required property"]
  1 phrasing errors: ["[...] is too short"]
  3 phrasings errors: ["[...] is too long"-style maxItems failure]
Also: check-live-state.sh PASS; doctor.sh "OK harness doctor" (same pre-existing WARN); worktree clean.

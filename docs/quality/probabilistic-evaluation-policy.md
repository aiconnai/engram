# Probabilistic Evaluator Policy (Advisory Only)

Status: policy, task O3 of the Engram comprehensive improvement plan (Lane P).
Scope: how probabilistic or model-based evaluators ("advisors") may be used
around Engram work, and what evidence is required before anyone relies on one.

## 1. Decision

Probabilistic evaluators are **advisory diagnostics, never gates**.

- An advisor result never enables, approves, or substitutes for a merge. Merge
  stays human (`docs/harness/GATES.md`, `docs/harness/CODE_REVIEW_POLICY.md`).
- No advisor runs in default CI, `sensors.sh`, `scripts/ci.sh`, the offline
  lane, or the pre-commit hook. Nothing here may become a required check or an
  offline-flaky one. Wiring an advisor into any gate is a policy change and
  needs its own task plus independent evaluation (section 9).
- Deterministic verifiers (tests, clippy, schema validation, review gate) keep
  full authority. If an advisor and a verifier disagree, the verifier wins; if
  the advisor is unavailable, work proceeds exactly as if it did not exist.
- Fail-closed: an advisor outcome of "defer", "unknown", an abstention, a parse
  failure, or an unavailable runtime means "no signal", never "pass".
- Rollback: remove the advisor invocation. Nothing else depends on it, so the
  verifier path is unaffected.

## 2. Prior experiment (diagnostic history only)

An earlier, informal experiment (2026-10-02) compared evaluators identified as
Laya, Kev and JEV on a small hand-curated set. It is preserved here as
diagnostic history. It is **not** evidence that any evaluator is calibrated,
and it must not be cited as a benchmark result.

| Evaluator | Correct | Brier score | Notes |
|-----------|---------|-------------|-------|
| Laya | 6/8 | 0.21588291625 | P(defer) ranged 62.13%-82.70% depending on option order |
| Kev | 7/8 | 0.06903502125 | no order-sensitivity figure carried forward |
| JEV | not executed | n/a | no credential was available |

Limits that apply to every row above:

- The 8 cases were curated by the experimenter. They are not a holdout, were
  not human-labeled by an independent party, and N=8 cannot separate skill from
  luck (one case moves accuracy by 12.5 points).
- Per-class breakdown, base rate, abstention handling, the exact prompts, and
  the model/runtime versions were not preserved. They cannot be reconstructed
  from this record.
- Laya's P(defer) swing of 62.13%-82.70% across option orders is a measured
  position-bias signal: ordering alone moved the answer by 20.57 points.
  Under this policy that makes Laya's raw scores unusable for any decision
  until permutation stability is demonstrated (section 6).
- JEV was never run. The local alias `jev-latest` points at a model that is not
  independent of the others and must not be counted as an independent JEV
  opinion. JEV is recorded as **unavailable**, not as a result.
- Kev's 0.06903502125 is the lowest Brier in the table, but it comes from the
  same 8 curated cases and an unrecorded prompt, so it is not a ranking of
  Kev over Laya.

Provenance status: the local working directory named by the experiment
(`/tmp/engram-probability-review-2026-10-02/`) no longer exists, so no raw
artifacts could be sanitized or archived. See
`docs/quality/evaluations/2026-10-02-probability-review/PROVENANCE.md`.
The figures above are therefore carried forward from the plan text, unverified
against source files.

## 3. Hosted JEV: unavailable

Hosted JEV is **unavailable** in this program. There is no existing credential
that was obtained through a secure channel, no authorization to send repository
or private content to a hosted service, no budget approval, and no official
runtime selected. Under this policy nobody may provision a key, install a
proxy or shim, or send private content to complete a comparison table. Absence
of the credential is recorded as `unavailable`, never as a pass, a zero, or an
imputed score. This status changes only if the owner supplies all four of:
an existing credential obtained securely, content authorization, a budget, and
an official runtime.

## 4. Metric separation

Four quantities are different things and are reported separately. They are
never summed, averaged, or ranked together.

| Quantity | What it is | Proper treatment |
|----------|------------|------------------|
| p(true) | probability that a stated claim is true | scored with Brier / log loss against a binary label |
| Distribution over actions | probabilities across discrete actions (e.g. accept / revise / defer) | scored per distribution with multi-class Brier or log loss; never collapsed to p(true) |
| Ordinal score | rank on an ordered scale (e.g. 1-5 severity) | scored with rank-aware measures (e.g. weighted kappa); never treated as a probability |
| Confidence | the evaluator's stated certainty in its own answer | evaluated only through calibration against correctness; never a score of the claim itself |

Rules:

- Do not average a p(true) with an ordinal score, a confidence, or an action
  probability, and do not average Brier scores across incompatible quantities.
- Report one table per quantity. A single "overall score" is not permitted.
- Aggregating several evaluators is allowed only within one quantity, and the
  aggregation method must be pre-registered (section 5).

## 5. Correlated evaluators are not independent votes

Models from the same family, the same provider, a shared fine-tune, or reached
through an alias (such as `jev-latest`) share training data and failure modes.

- Count evaluators as independent only with evidence of independence (distinct
  vendor/lineage and demonstrably different error patterns on the holdout).
- Without that evidence, treat N agreeing evaluators as one opinion; agreement
  does not raise confidence and majority voting is not allowed.
- Record the exact model identifier and version string the runtime reports for
  every run. An alias that cannot be resolved to a concrete model is recorded
  as such and is not eligible as an independent evaluator.

## 6. Required tests before any advisor output is trusted

Each test is reported with N and its limits. A result without N is invalid.

1. **Explicit abstention.** The evaluator must be able to answer "unknown" or
   abstain. Report the abstention rate and score abstentions separately from
   answered cases; abstaining is not a wrong answer and not a right answer.
2. **Permutation stability.** Re-run each case with the option / evidence order
   permuted (at least all cyclic rotations; all permutations when options <= 4).
   Report the min-max range of every probability per case, as the Laya
   P(defer) 62.13%-82.70% figure above. A spread larger than the pre-registered
   tolerance disqualifies the evaluator for that quantity.
3. **Two predefined phrasings.** Each case is posed in two wordings that were
   written and frozen before any scoring. Report agreement between them. The
   wordings are not tuned after seeing results.
4. **Context and truncation.** Re-run with the evidence truncated and with
   distractor context added; report the change in score. Record the context
   window and whether truncation occurred silently.
5. **Model identification.** Record the reported model identifier per run and
   verify it matches the declared evaluator (section 5).
6. **Fail-closed overrides.** Where the policy fails closed (defer, unknown,
   parse error), the fail-closed outcome prevails over any numeric score,
   including a high-probability one.

## 7. Holdout benchmark specification

The benchmark that scores advisors is a **separate, human-labeled holdout**.

- Written and labeled by people other than the author of this policy and other
  than whoever configures the evaluators. The policy author must not see
  holdout labels before the evaluators are frozen.
- Each case records: a claim, the evidence offered, a label of `supported`,
  `refuted` or `unknown` (unknown is a first-class label, not missing data), the
  task, the risk class (what a wrong answer costs), the independent labelers,
  the two predefined phrasings (required), and the split (`dev` or `holdout`).
- Labeler exclusion (labelers are neither the policy author nor whoever
  configures the evaluators) is a **process check** that the schema cannot
  enforce; the schema only requires at least two distinct labeler ids. A
  reviewer must verify exclusion before a holdout is accepted.
- Cases include refutations and unknowns in meaningful proportion; a
  benchmark that is mostly supported claims rewards always-agree behavior.
- Fixtures are synthetic or sanitized: no production data, no customer data, no
  secrets, no model weights. The format is
  `docs/quality/evaluations/holdout-case.schema.json`, with two synthetic
  illustration rows in `docs/quality/evaluations/holdout-cases.example.jsonl`.
  The example rows are format illustrations, not benchmark data, and are not
  run by any tool or gate.
- **No fit and eval on the same sample.** Anything tuned (prompts, thresholds,
  calibration maps, aggregation weights) is tuned on `dev` only. The `holdout`
  split is scored once per frozen configuration; re-scoring after changes
  burns that holdout and a fresh one is required.
- The two predefined phrasings (section 6) and tolerances are frozen and
  committed before the first holdout scoring run.

## 8. Reporting requirements

Every advisory report states, at minimum:

- N overall and N per class (supported / refuted / unknown, and per risk class).
- The **base rate** of each class and a baseline score (always predicting the
  base rate, and the uninformative 0.5 reference) so a Brier number is
  interpretable. For binary p(true), a constant 0.5 scores 0.25; a score near
  that carries no information.
- **Brier per class**, not only pooled, with the per-class N, plus abstention
  count, error list (which cases failed and how), and stability under section 6.
- Confidence intervals or an explicit statement that N is too small for one.
  Small N (for example under 30 per class) is reported as "indicative only".
- Evaluator model identifiers, run date, prompt/wording ids, and whether the
  evaluator is independent of the others (section 5).
- What could not or did not run, with the two states kept distinct and never
  imputed: `unavailable` means infrastructure or a credential is missing (for
  example hosted JEV, section 3); `NOT RUN` means the run was skipped or not
  attempted although it could have been. Neither counts as a pass or a zero.

## 9. Governance

- **Reference intake before adoption.** Laya, Kev and JEV implementations, or
  any other evaluator source, may be used only after reference intake records
  the source, its licence, what is adapted, and what is excluded. Until then
  nothing is copied or vendored. No model weights, prompts that embed secrets,
  or credentials are versioned in this repository.
- **Independent evaluation of policy changes.** Any change to this policy,
  tolerances, the holdout, or an advisor's role (especially toward gating)
  requires an evaluation by someone other than its author, on the holdout.
- **Isolation.** Evaluation fixtures live under `docs/quality/evaluations/`.
  They are not referenced by tests, `sensors.sh`, `ci.yml`, or `doctor.sh`.
  Running an advisor is a manual, opt-in, offline-or-owner-authorized activity.
- **Budget and content.** Any hosted run requires explicit owner authorization
  for the content sent and the spend, per run.

## 10. Acceptance statement for O3

Results from these evaluators do not enable merge, are not required in CI, and
cannot make CI offline-flaky because none of them is invoked by CI. Removing the
advisor (deleting its invocation) leaves the verifier untouched.

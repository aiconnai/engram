FAIL progress records B2 as passing even though the referenced review evidence is failing/missing from HEAD

- [BLOCKER] `docs/harness/progress.md:9` sets `Last review` to `pass: docs/harness/reviews/2026-06-21-b2-loop-skills-policy-post.md`, but that artifact begins with `FAIL` at `docs/harness/reviews/2026-06-21-b2-loop-skills-policy-post.md:1` locally and no `docs/harness/reviews/2026-06-21-b2-loop-skills-policy*.md` file is tracked in `HEAD`. `docs/harness/progress.md:677` also claims the review evidence was recorded. This is a fake-success audit trail for a process-critical harness change; commit the final PASS artifact or correct progress to the real pending/FAIL state.

REVIEW_VERDICT: FAIL progress points at failing or untracked B2 review evidence

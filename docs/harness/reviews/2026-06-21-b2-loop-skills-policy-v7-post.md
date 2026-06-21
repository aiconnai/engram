FAIL B2 still has a hard-gate evidence gap and the new skill frontmatter check is not fail-closed.

- [HIGH] `docs/harness/progress.md:671` records `doctor.sh` checks and `docs/harness/progress.md:674` records only `bash docs/harness/bin/sensors.sh quick` for a change that touched `docs/harness/bin/doctor.sh`. `docs/harness/GATES.md:154` and `docs/harness/GATES.md:166` say the no-argument full `sensors.sh` is the canonical gate and optional lanes do not replace it for handoff/completion; `sensors.sh status --json` currently reports `last_mode=quick`. Fix by running and recording full `bash docs/harness/bin/sensors.sh` for this process-critical harness change.

- [MED] `docs/harness/bin/doctor.sh:368` says the parser extracts frontmatter between the leading `---` and next `---`, but `docs/harness/bin/doctor.sh:372`-`377` never requires the closing fence. A file with only an opening `---` plus `name:`/`description:` lines passes the same checks at `docs/harness/bin/doctor.sh:388`-`392`; I verified this with a temp-file probe where both `name_check_passed` and `description_check_passed` were emitted. This weakens the new `docs/harness/SKILLS.md:38`-`40` contract. Fix by making `skill_frontmatter` fail unless it observes the second fence.

REVIEW_VERDICT: FAIL full sensor evidence is missing for the harness script change and frontmatter parsing accepts malformed fences.

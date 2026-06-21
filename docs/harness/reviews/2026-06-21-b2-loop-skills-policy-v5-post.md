FAIL doctor skill validation can pass an explicitly not-ported follow-up skill as inventoried

- [HIGH] `docs/harness/bin/doctor.sh:405` checks disk-to-inventory membership with a broad grep over all of `docs/harness/SKILLS.md`. Because `docs/harness/SKILLS.md:20` marks “Available for Follow-Up” as not ported in B2, and `docs/harness/SKILLS.md:31` already mentions `loop-triage`, a tracked `skills/loop-triage/SKILL.md` passes doctor even though it is not in the Current Skills table. I verified this in a temp archive by adding a tracked valid `skills/loop-triage/SKILL.md`; `bash docs/harness/bin/doctor.sh` exited 0. This weakens the new gate against hidden scope creep. Fix by validating disk skills against only the parsed Current Skills set.

REVIEW_VERDICT: FAIL doctor accepts follow-up-only skills as inventoried

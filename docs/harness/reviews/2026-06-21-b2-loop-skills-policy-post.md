FAIL Missing required Review Canvas and progress traceability for a process-critical harness skill-policy change

REVIEW_VERDICT: FAIL Missing required Review Canvas and progress traceability for b2-loop-skills-policy

- [BLOCKER] Missing Review Canvas for `b2-loop-skills-policy`. The diff changes process-critical `docs/harness/bin/doctor.sh:255` and adds the 358-line `skills/loop-engineering/SKILL.md:1`; `docs/harness/GATES.md:138`, `docs/harness/GATES.md:142`, and `docs/harness/GATES.md:148` require a canvas before post-review for this shape of change, and `docs/harness/CODE_REVIEW_POLICY.md:162` makes missing canvas a blocker when process-critical scripts are touched. No matching `docs/harness/canvas/2026-06-21-b2-loop-skills-policy.md` exists.

- [HIGH] Canonical harness progress was not updated for this task. `docs/harness/INVARIANTS.md:26` requires harness-process commits to update `progress.md` and the active sprint log, but `docs/harness/progress.md:7` still points at `harness-bootstrap`, `docs/harness/progress.md:11` still records old commit `f2b1799`, and grep found no `b2-loop-skills-policy`, `SKILLS.md`, or `loop-engineering` entry in the live progress files.

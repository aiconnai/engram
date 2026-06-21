Now I have enough evidence to write the review.

---

FAIL Two overlapping PR-title scripts with divergent exit codes and doctor.sh gap on the new policy script

- **[BLOCKER] Dual PR-title scripts with incompatible exit contracts.**
  The diff introduces two separate scripts for the same gate: `docs/harness/bin/check-pr-title.sh` (exits **1** on `[codex]` rejection) and `docs/harness/bin/pr-title-policy.sh` (exits **4** on `[codex]` rejection). `sensors.sh` calls only `run_pr_title_policy` (the `exit 4` variant) while `doctor.sh` self-tests only `check-pr-title.sh` (the `exit 1` variant). GATES.md documents exit code 4 under "PR Title Policy" but lists no exit codes under "PR Title Gate". The two scripts are therefore not interchangeable, yet both are presented as implementing the same invariant (INVARIANTS.md §6). This is a fake-parity pattern: `doctor.sh` will pass while `sensors.sh` exercises a different script — any divergence between the two implementations (e.g. the spaced/mixed-case `[ CoDeX ]` pattern, which `check-pr-title.sh` strips differently than `pr-title-policy.sh` via `tr` vs `grep -Eiq`) will go undetected by the gate that is supposed to be the source of truth (Invariant 15).

- **[HIGH] `doctor.sh` does not require or test `pr-title-policy.sh`.**
  `doctor.sh` adds `require_file` / `require_exec` for `check-pr-title.sh` and runs its two self-tests against that script. `pr-title-policy.sh` — the script wired into every `sensors.sh` mode including `full` — has no `require_file`, no `require_exec`, and no self-test in `doctor.sh`. Per INVARIANTS.md §15, "Doctor.sh é a fonte de verdade para integridade do harness." A missing or corrupt `pr-title-policy.sh` would silently pass doctor while causing every sensors mode to fail. Canvas `2026-06-21-b1-pr-title-policy.md` under "Breakage Risk" lists "`doctor.sh` requires a script that is not executable" as a risk but mitigates it with "`Commit the script executable`" — it does not address the missing `require_file`/`require_exec` entry.

- **[MED] Canvas owner recorded as "Codex" across all four canvas files.**
  `docs/harness/canvas/2026-06-20-hooks-contracts.md`, `-pr-title-guard.md`, `-storage-extension-semantics.md`, and `2026-06-21-b1-pr-title-policy.md` all list `Owner: Codex`. INVARIANTS.md §9 requires cross-CLI/cross-model review for non-trivial tasks and the review prompt explicitly calls out fake-success pattern #10: "Reviewer is being shown a self-referential or incomplete prompt." Having the implementing agent also author its own canvas owner field does not violate policy text directly, but it is notable that the harness being reviewed was built by the same entity authoring the canvases presented as independent design records. This is a process hygiene flag, not a blocker, but reviewers should be aware.

REVIEW_VERDICT: FAIL Dual PR-title scripts with incompatible exit codes (1 vs 4) and doctor.sh gap on pr-title-policy.sh violate harness self-consistency guarantees (Invariants 15, 25)

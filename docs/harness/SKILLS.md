# Repository Skills

Engram keeps repository-specific agent skills under `skills/<name>/SKILL.md`.
These are part of the local harness surface when they describe repeatable
project work, review policy, loop behavior, or external-system operation.

Skills are explicit operational context, while diagnostics and doctor checks
keep the harness observable. Repo-local skills must therefore be tracked,
reviewed, and validated instead of left as accidental untracked files.

## Current Skills

| Skill | Purpose | Default level |
|---|---|---|
| `agentshield-scan` | Weekly/manual AgentShield security triage loop with a hard iteration cap | L1 report-only |
| `engram-council` | Structured consensus via `memory_council` / `CouncilSkill` for design, policy, security, and tradeoff decisions | Read-only decision support |
| `engram-onboarding` | Bootstrap an agent's engram memory session in a new repository | Policy/reference |
| `loop-engineering` | Shared safety model for repeatable agent loops | Policy/reference |

## Available for Follow-Up (Not Ported in B2)

The following skills exist in AgentShield's repo-local harness and are candidates
for a future port to Engram. They are **intentionally not ported in Task B2** —
B2 carries only `loop-engineering` (the shared safety base) plus this policy, so
the gate and review stay scoped to one new operational surface. Each of these
introduces its own operating surface (CI parsing, dependency advisories, PR
triage) and warrants its own task, port, and review canvas.

| Skill | Purpose | Suggested level on port |
|---|---|---|
| `loop-triage` | Daily signal triage for CI, issues, PRs, commits, and chat | L1 report-only |
| `loop-triage-ci` | CI failure grouping and bounded minimal-fix handoff | L1 report, L2 only with verifier |
| `dependency-triage` | Dependency advisory and patch-candidate triage | L1 report, patch-only candidates |
| `pr-review-triage` | PR aging, CI block, and reviewer-thread triage | L1 report, L2 only with verifier |

## Policy

- Every repo-local skill must live at `skills/<name>/SKILL.md`.
- Every skill must have YAML frontmatter with `name` and `description`.
- `name` must match the directory name.
- New skills require a harness review canvas when they change loop behavior,
  gate behavior, external-system operations, or automation level.
- Skills that only support a single operator and should not affect the repo
  belong in that operator's personal CLI skills directory (for example
  `~/.codex/skills` or `~/.claude/skills`), not this repository.
- Local run artifacts, sub-agent evidence, and review transcripts belong in
  `docs/harness/reviews/` or the loop's `docs/loops/<name>/` directory according
  to their purpose.

## Promotion Checklist

Before adding a new skill:

1. Decide whether it is repo policy or a personal operator shortcut.
2. Confirm the skill is report-only by default unless the user explicitly
   requests write-path work.
3. Name the verification commands or manual evidence the skill requires.
4. Add or update harness docs if the skill changes process.
5. Run `bash docs/harness/bin/doctor.sh`.

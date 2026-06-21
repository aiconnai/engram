# Review Canvas: b2-loop-skills-policy

Date: 2026-06-21
Owner: Claude (Opus); reviewed cross-model by Codex
Scope: Port AgentShield's `loop-engineering` base skill plus the `SKILLS.md`
repository-skill promotion policy into Engram, and wire skill-frontmatter
validation into the deterministic harness doctor. The other four AgentShield
loop skills (`loop-triage`, `loop-triage-ci`, `dependency-triage`,
`pr-review-triage`) are intentionally out of scope and documented as follow-ups.

## Trigger

| Trigger | Evidence |
|---|---|
| Harness gate/policy change | Adds inventory + frontmatter validation to `docs/harness/bin/doctor.sh` and adds `docs/harness/SKILLS.md` policy. |
| More than 200 non-generated lines | `skills/loop-engineering/SKILL.md` is ~358 lines of new content. |
| Process-critical script touched | `doctor.sh` is in `docs/harness/bin/*`; required `[BLOCKER]`-level canvas per `CODE_REVIEW_POLICY.md`. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Port only `loop-engineering` + `SKILLS.md` policy this task | Accepted | Keeps the gate and review scoped to one new operational surface; the safety base is the prerequisite for the other four skills. |
| Port the whole 5-skill family at once | Rejected | Mixes skill policy with several new operating surfaces (CI parsing, dependency advisories, PR triage), making the gate/review harder and broadening blast radius. Documented as follow-ups in `SKILLS.md`. |
| Read sources by checking out `chore/repo-local-harness-skills` in AgentShield | Rejected | Mutating that branch is unnecessary; `git show <branch>:<path>` reads the sources without touching any working tree. |
| Copy `SKILL.md` verbatim from AgentShield | Rejected | Source is AgentShield-specific (`cargo run -- scan`, `release.yml`, Homebrew/crates.io). Adapted product references to Engram (MCP/memory loops, `just ci`, `agentshield-scan` worked example) while preserving the L1/L2/L3 model and frontmatter. |
| Port the AgentShield `.omo/`-ignored-evidence doctor check | Rejected | Engram's `.gitignore` has no `.omo/` convention; porting it would assert a convention the repo does not have. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| `doctor.sh` skill loop | O(k) in number of `skills/*/SKILL.md` (currently 4) | O(1) | One `git ls-files` + a bounded `find -mindepth 2 -maxdepth 2`; three `grep` per skill. |
| `doctor.sh` cross-references | O(1) | O(1) | Three additional `require_grep` calls (README + SKILLS anchors). |
| `sensors.sh quick` | Constant extra work | O(1) | Doctor already runs in the quick lane; skill checks add bounded greps. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| New skill file left untracked | `git ls-files --others` check fails doctor until the skill is tracked. Verified: doctor FAILed on untracked `skills/loop-engineering/SKILL.md`, passed after `git add`. |
| Skill `name:` does not match its directory | `require_frontmatter_field name "$skill_dir"` fails for that skill. Verified in a temp clone: a `name: wrong-name` frontmatter FAILs doctor (exit 1). |
| Skill not listed in `SKILLS.md` inventory | `require_grep` for `` `$skill_dir` `` against `SKILLS.md` fails if a skill is missing from the table. |
| Inventoried skill deleted from disk (inventory→disk gap) | The "Current Skills" table is parsed and each named skill must have `skills/<name>/SKILL.md`. Verified: deleting `loop-engineering/SKILL.md` in a temp clone FAILs doctor (`inventoried skill missing on disk`), where the disk→inventory `find` loop alone passed. Found by cross-model (Codex) review v2. |
| Frontmatter keys present only in the markdown body | `name:`/`description:` are validated inside the leading `---`…`---` YAML block via `skill_frontmatter`, not anywhere in the file. Verified: a body-only `name:`/`description:` (no fence) FAILs doctor (`no YAML frontmatter block`). Found by cross-model (Codex) review v2. |
| Follow-up-only skill added to disk without promotion | Inventory membership is computed only from the parsed "Current Skills" table (`current_skills_set` + `grep -qxF`), so a `skills/loop-triage/SKILL.md` matching the "Available for Follow-Up" table is rejected. Verified: a tracked `loop-triage` skill FAILs doctor (`not in SKILLS.md Current Skills table`). Found by cross-model (Codex) review v5. |
| Frontmatter opening fence with no closing fence | `skill_frontmatter` is fail-closed: it emits the block only when the closing `---` is observed (buffered + `END { if (closed) ... }`), so an unterminated block yields no frontmatter and the field checks FAIL. Verified: an open-`---`-only file FAILs doctor (`no YAML frontmatter block`). Found by cross-model (Codex) review v7. |
| `SKILLS.md` or its required anchors removed | `require_file docs/harness/SKILLS.md` + `require_grep` for `` `loop-engineering` `` and `~/.codex/skills` fail. |
| README loses the SKILLS.md reference | `require_grep docs/harness/README.md 'SKILLS\.md'` fails. |

## Breakage Risk

| Risk | Impact | Mitigation | Rollback | Verification |
|---|---|---|---|---|
| Untracked-skills check blocks unrelated WIP skill dirs | Local doctor fails until tracked/ignored | Check matches only `skills/*/SKILL.md`; `--exclude-standard` honors `.gitignore` | Revert this commit to drop the skill loop | `doctor.sh` exits 0 with the 4 tracked skills. |
| Backtick quoting in the `inventoried` grep mis-parses | False FAIL/PASS on inventory check | Pattern uses `\`$skill_dir\`` (literal backtick in ERE) and was exercised against all 4 skills | Revert this commit | `doctor.sh` exits 0; removing a skill from `SKILLS.md` would FAIL. |
| `require_grep` arg order differs from AgentShield source | Wrong file/pattern checked | Engram contract is `require_grep <path> <pattern> <label>`; all new calls follow it | Revert this commit | `doctor.sh` exits 0 and FAILs when an anchor is removed. |
| Adapted skill content drifts from real Engram gates | Skill points at non-existent commands/paths | Worked examples (`agentshield-scan`, `just ci`, `sensors.sh quick`) and human-gated paths (`release.yml`, `proto/`, attestation, auth) were checked against the live repo | Revert the `SKILL.md` add | `doctor.sh` validates frontmatter + inventory; manual read confirms commands exist. |

## Decision

Proceed.

Reason: This ports an accepted AgentShield safety base plus a small deterministic
inventory/frontmatter gate, scoped to a single new skill. Sources were read
without mutating any branch, product references were adapted to Engram, and the
out-of-scope skills are documented as explicit follow-ups. Cheap local
verification (`doctor.sh`, `sensors.sh quick`) and a direct per-commit rollback
path exist.

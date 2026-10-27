# Review record — Engram comprehensive improvement program (waves 0-4)

| Field | Value |
|-------|-------|
| Program | `engram-comprehensive-improvement` (plan `docs/harness/plans/2026-10-02-engram-comprehensive-improvement-plan.md`) |
| Branch | `claude/engram-improvement-plan-edb43d` (lane R; lane P merged in `ee8c4ea`) |
| Execution baseline | `1952f3b` (14 local commits ahead of `origin/main` `949c963`) |
| Final review scope | `1952f3b..4868d49` (whole branch after integration) |
| Fix wave after the final review | FINALFIX (section below) |
| Delivery | Local commits only; no push, PR or merge |

## What kind of review this is

These are **subagent-driven-development (SDD) independent reviews**: for every task the
controller dispatched a reviewer subagent with a fresh context (no access to the implementer's
reasoning), against a frozen diff package, using the reviewer model named per row below. Fix
rounds re-dispatched the implementer and then a re-reviewer on the fix diff. The ledger ruling
(see `ledger.md`, "Ruling: SDD task reviews ...") treats these as the cross-model review required
by harness invariant 10.

**`review-gate.sh post` with an operator receipt was NOT run** for any task of this program
(the H1 gate only landed mid-program, and no operator receipt was ever produced). The ledger
ruling expected review-gate artifacts per wave; none were created. Nothing in this directory is a
receipt-verified PASS, and nothing here authorizes a merge. Merge remains a human decision.

## Archived evidence (this directory)

- `ledger.md`: the SDD ledger verbatim (dispatches, verdicts, fix rounds, deferred minors,
  controller rulings, integration steps). Copied from the git-ignored `.superpowers/sdd/` run dir.
- `task-<ID>-brief.md`: the requirements given to each implementer.
- `task-<ID>-report.md`: each implementer's report with TDD evidence, verification commands and
  appended fix-round reports.
- `final-review.md`: the final whole-branch review (Opus, 2026-10-05).
- Not archived: the `review-*.diff` packages (re-derivable from git with the commit ranges below)
  and the Q4 repro script. Absolute home and scratch paths in the copies were redacted to `~/`
  and `<session-tmp>`; trailing whitespace was stripped. No other content was changed.

## Per-task record

Reviewer = model of the first review; later rounds name the re-reviewer when the ledger records
it ("n/r" = not recorded in the ledger). Every task ended "review clean" in the ledger.

| Task | Lane | Implementer | Reviewer → verdict | Fix rounds | Commits |
|---|---|---|---|---|---|
| E0 state/ADR/baseline | P | sonnet | sonnet → Approved | 0 | `630dd26..0c6ea2b` |
| Q1a CI/local quality contract | P | sonnet | n/r → Approved w/ 1 Important | 1 | `0c6ea2b..cf6d969` |
| H1 fail-closed review gate | P | sonnet | opus → Needs fixes | 1 | `cf6d969..60343b3` |
| H2 schemas/evidence + offline lane | P | sonnet | sonnet → Approved | 0 | `60343b3..df9e9c8` |
| O3 probabilistic evaluators (advisory) | P | sonnet | haiku → Approved w/ 1 Important | 1 | `df9e9c8..17c6497` |
| Q5 security gate + supply chain | P | sonnet | sonnet → Needs fixes | 1 | `f4f547c 945a6cf cae024c 62101f8` |
| H3 check registry + sandbox | P | sonnet | opus → Needs fixes; sonnet, haiku re-reviews | 2 | `60cf3db e4b0812 e2a2930 5d9c04c 93ff80b 4d341a0` |
| H6 short live context + retention | P | sonnet | sonnet → Needs fixes | 1 | `c7a8d71 a0f3dea 2be03c1 96c82c5` |
| H4 runner + external evidence | P | opus | opus → Needs fixes; sonnet, haiku re-reviews | 2 | `2bdf942 1501600 e0021c9 240d0a4 26d43ea f96b200` |
| O2 git hygiene + retention | P | sonnet | sonnet → Approved | 0 | `a40430d e6807e3 980737f 695b553 19a6218` |
| H5 SHA-bound review + merge policy | P | opus | opus → Needs fixes (1 Critical) | 2 | `8950f47 7e4a2c9 bcbb568 0d8c7c1 5b2fa0f 280698c 403bf3e cd5e7f3 29d4f7e 3a373e7` |
| O4 standing checks | P | sonnet | sonnet → Approved | 0 | `a9809d1 1f5879d c8581cc` |
| C2 WAL replay/recovery + storage concurrency | R | opus | opus → Needs fixes | 1 | `630dd26..46fc3e2` |
| Q2 Rust risk inventory | R | sonnet | sonnet → Approved | 0 | `a87a3ce` |
| C1 workspace authorization | R | opus | opus → Needs fixes | 1 | `44d42f8 6750b9b` |
| C3 Unicode/hex panics | R | sonnet | sonnet → Approved | 0 | `a6f55ac` |
| Q4 MCP/SDK contract tests | R | sonnet | sonnet → Needs fixes | 1 | `98bd050 0d36d3d` |
| G1 SQLite lock-drop data loss (new P0) | R | opus | opus → Needs fixes; opus re-review (round 2: n/r) | 2 | `974b7a6 9712ade bbe9203` |
| Q3 fuzz/Miri/mutants lanes | R | sonnet | sonnet → Needs fixes | 1 | `4e8ba9a 730ede4` |
| C7 defer_embedding enqueue | R | sonnet | opus → Needs fixes; sonnet re-review (round 2: n/r) | 2 | `f560a6c 081c73f ec3093c` |
| C5 create path vocabulary | R | sonnet | sonnet → Approved | 0 | `c22e11e` |
| C6 hooks/multimodal/sync failures | R | sonnet | opus → Needs fixes; sonnet re-review | 1 | `c601d69 437d0e0 909cdea df2f97c 7abedef b3fb284` (+ `src/hooks/failure_tests.rs` in `ec3093c`) |
| Q7 candidate quality runner | R | sonnet | sonnet → Approved with fixes (3 Important) | 1 | `12d2c3b 8e7f76f 4d067f1` |
| P1 perf regression fix (new) | R | opus | opus → Approved | 0 | `772e84a 0a46e9b` |
| Q2F Q2 backlog fixes (new) | R | sonnet | sonnet → Needs fixes | 1 | `ab22911 62c266b 98103b0` + `9e0b306..a1a19c4` merged in `77741c3` |
| O1 observability + redaction | R | sonnet | opus → Needs fixes | 1 | O1 files in `35a1592`, `8cef307` (content of orphaned `9a6ed82`), `96d41bb` |
| Q6 dependency advisories | R | sonnet | sonnet → Approved | 0 | `6c4ab81 6c27b57 35a1592` (lock hunk) `572495b 8cef307` |
| C4 SQLite fd ADR + spike | R | opus | sonnet → Needs fixes; haiku re-review | 1 | `1c246c8 c099ca2` |
| INT lane integration + Q1b | — | opus | covered by the final whole-branch review | — | `ee8c4ea a2fd341 b8e693a a231cdb e121894 33cf8a1 11a48a1 4868d49` |

Known commit-hygiene facts (owner decision, not changed here): `35a1592` mixes Q6
(`Cargo.lock` event-listener) with 51 O1 files, and `ec3093c` (C7) also carries C6's
`src/hooks/failure_tests.rs`. Both are acceptable only with a squash merge; otherwise they need
an owner-authorized split.

## Final whole-branch review

- Reviewer: Opus, fresh context, scope `1952f3b..4868d49`. Verdict: **Ready with fixes**,
  0 Critical, 5 Important, 11 Minor. Full text: `final-review.md`.

## FINALFIX wave (after the final review)

Single fix agent (Opus), base `4868d49`. Its own report and verification evidence live in the
run dir as `task-FINALFIX-report.md`; the commit list and the verification summary are recorded
in the lane-P log entry "FINALFIX" (`docs/harness/progress/2026-10-05-improvement-lane-p.md`).
A scoped independent re-review (Sonnet) of `4868d49..9d068fb` then verdicted every finding
ADDRESSED with no new Critical/Important breakage. It recommended removing the
`REVIEW_VERDICT` pointer file that had been added to green `doctor.sh`; the owner agreed and the
pointer was removed. There is deliberately **no** `REVIEW_VERDICT` marker for this program:
no `review-gate.sh post` was run with an operator receipt, so `doctor.sh` keeps its honest
"no review artifact found for active task" warning until that post-gate happens.

- Important 1: `engram-cli mcp install` temp file and backups take the config's mode (0600 when
  new); a reused backup is narrowed; a symlinked config stays a symlink (TDD).
- Important 2: `memory_ingest_media` catalog text states the cross-workspace `file_hash` limit;
  `docs/MCP_TOOLS.md` regenerated.
- Important 3: `CHANGELOG.md` [Unreleased] lists the client-visible changes.
- Important 4: this record, the archive, and the reconciled `progress.md`/`SPEC.md`.
- Important 5: offline-lane floors set to the exact counts, with a contract test per component.
- Minors folded: `.gitattributes -whitespace` for byte-exact artifacts, budgets anchoring
  prerequisite, PR candidate SHA note, gRPC join-error redaction (TDD), standing-checks
  upload-artifact v7, review-gate absolute `GATE_SCRIPT` (TDD), `ttl_days` min/max in the
  catalog, CODEOWNERS coverage, repo-relative plan paths, supply-chain pin expiry in
  `progress.md`, qualified hooks panic claim.

<a id="owner-decisions-surfaced-not-implemented"></a>
## Owner decisions & post-merge waves 5–6 resolution (2026-10-14)

All owner decisions, follow-up issues, and deferred items surfaced during the waves 0–4 review
were resolved, implemented, verified, and closed in post-merge Waves 5 and 6:

- **Squash-merge PR #234**: PR #234 was squash-merged (`26b40d3`), resolving the mixed commits
  `35a1592` and `ec3093c` without requiring manual branch rebase.
- **Follow-up issues #235–#241 implemented and verified** in commits `661b140`, `1ad6586`, `336ef07`,
  `dd3b6ae`, and `e17fb5e`:
  - `media_assets` `UNIQUE(workspace, file_hash)` + migration `v49` (#235, `1ad6586`)
  - Bounded 50 MiB media reads in `multimodal.rs` and `image_storage.rs` (#236, `1ad6586`, `336ef07`)
  - `SessionStartHandler`, workspace-scoped `PostToolUse`, and `SyncWorker` wired in `src/bin/server.rs` (#237, `dd3b6ae`)
  - macOS `SQLITE_READONLY` `-shm` open probe + retry under `DATABASE_OPEN_LOCK` (#238, `dd3b6ae`)
  - `aws-sdk-s3` + `BehaviorVersion::v2026_01_12()` pin removing `lru 0.16.4` (#239, `336ef07`)
  - `budgets.json` anchoring against `--floors-anchor` + owner `Ronaldo` acceptance and cryptographic sealing of `docs/quality/candidate-floors.json` (#240, `dd3b6ae`, `e17fb5e`)
  - `docs/security/supply-chain-pins.toml` digests pinned (`unpinned_images = []`, #241) and O4 standing-checks owner set to `Ronaldo` (`336ef07`)
- **Wave 6 resolution**:
  - SDK/server drift (`G-2`, 161 entries -> 0)
  - TypeScript error/timeout parity (`G-4`, `G-5`)
  - MCP parameter/error normalization (`G-6`, `G-7`)
  - SQLite read-path permission check optimization
  - `readers_readonly_N` benchmark and Criterion rebaseline
  - Owner sign-off on `D7`, `C1`, `#152`, and RFCs 0009–0011

See [`wave5-6-summary.md`](./wave5-6-summary.md)
for the full post-merge verification record.

## Nota de proveniência (2026-10-26)

O veredito `REVIEW_VERDICT: PASS` de 2026-10-14
([`../2026-10-14-engram-comprehensive-improvement-post.md`](../2026-10-14-engram-comprehensive-improvement-post.md))
**contradiz** as linhas 21-24 e 91-95 acima (nenhum `review-gate.sh post` com receipt; nenhum marcador
de propósito) e não registra reviewer, intervalo nem receipt. O que está e o que não está evidenciado
fica em [`../2026-10-14-engram-comprehensive-improvement-post-provenance.md`](../2026-10-14-engram-comprehensive-improvement-post-provenance.md).
Este README permanece como foi escrito em 2026-10-05; a nota não altera o veredito nem cria receipt.

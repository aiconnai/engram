# AI Operating Guide

Use this guide to decide when and how an AI team should trigger `lazycodex-ai`.

## Precedence

This guide does not override system, developer, user, `AGENTS.md`, `CLAUDE.md`,
or harness instructions. In this repository, run the required harness bootstrap
and read order before delegating work. Use `lazycodex-ai` only for explicit
execution tasks, and never as a bypass around local validation, review, security
boundaries, or user approval requirements.

Command examples below use the Engram shell form with `rtk`. Outside this
repository, the plain command is the same without the `rtk` prefix.

## Quick Decision Matrix

- **Use `run`** when the request is for action: implement, fix, refactor,
  generate files, migrate, or modify repo behavior.
- **Do not use `run`** when the request is explanatory: definitions, opinions,
  comparisons, or strategy discussion.
- **Run `doctor` first** when environment health is uncertain.

### Quick Checks

- Execution request + repo files involved -> `rtk npx lazycodex-ai run "..."`
- No state change requested -> direct response
- After `run` -> verify via tests/build/lint and report evidence

## 1. Why Use `lazycodex-ai`

Use `lazycodex-ai` for repository tasks that require operational execution with
multiple edits, validation steps, or persistent state changes.

Prefer using it when:

- you are implementing a feature, fixing a bug, refactoring, or doing
  structural changes;
- the task spans multiple steps, for example: edit files, run checks, and
  consolidate results;
- completion needs explicit behavioral verification;
- the user requested concrete delivery, not just an explanation.

Avoid using it for:

- theoretical questions, such as "how does X work?", where a text answer is
  enough;
- low-risk docs-only edits that can be done directly;
- trivial, single-line changes that do not need the toolchain.

## 2. When To Use It

1. If the user says "do X", "fix X", "implement X", or "generate X", use
   `run`.
2. If the user asks "explain", "what are the options", or "what do you think"
   with no state change, do not use `run`.
3. If you are unsure about environment health, such as missing installations or
   dependencies, run `doctor` first.

### Quick Heuristic

- If repository files are involved, use `run`.
- If this is a decision or recommendation request, `run` usually is not needed.
- If it is a long repetitive workflow, use `run` and `--on-complete` if helpful.

## 3. How To Use It

### 3.1 Pre-Check

1. Run:

   ```bash
   rtk npx lazycodex-ai doctor
   ```

2. Fix any blocking issues before delegating.
3. Define a measurable goal:
   success = file X changed + test Y passing + validation Z completed.

### 3.2 Standard Execution Command

```bash
rtk npx lazycodex-ai run "Task summary with acceptance criteria"
```

Examples:

```bash
rtk npx lazycodex-ai run "Fix the /health bug in app.py and add return validation"
rtk npx lazycodex-ai run "Generate an organization/project catalog index.html for repositories"
```

### 3.3 Useful Options

- `--agent <name>`: choose agent, such as `Sisyphus` or `Hephaestus`.
- `--model <provider/model>`: override model when needed.
- `--directory <path>`: set the working directory.
- `--json`: output structured JSON.
- `--session-id <id>`: resume a prior session.
- `--on-complete "<cmd>"`: run a follow-up command after completion, such as
  validation, CI, push, or another local check.

### 3.4 Standard AI Prompt Format

Always include:

- Objective
- Acceptance criteria
- Constraints
- Validation steps

Prompt template:

```text
Implement X. Keep existing behavior compatible.
Acceptance criteria:
1) change files A/B;
2) validation command `cmd` passes;
3) preserve existing contracts.
Return: changed files, commands run, and test evidence.
```

## 4. Minimum Validation Flow

After every `run`:

1. Re-check local health if needed:

   ```bash
   rtk npx lazycodex-ai doctor
   ```

2. Run the relevant verification commands, such as tests, build, or lint.
3. Confirm evidence in output:
   - touched files;
   - test results;
   - relevant observations.

## 5. Maintenance Commands

```bash
rtk npx lazycodex-ai install
rtk npx lazycodex-ai update --dry-run
rtk npx lazycodex-ai doctor
rtk npx lazycodex-ai run --help
```

- `install` bootstraps setup.
- `update --dry-run` previews an update without applying it.
- `doctor` runs a quick health check.
- `run --help` shows current options.

## 6. Common Issues

- "OpenCode binary not found" -> install OpenCode and ensure it is on `PATH`.
- "oh-my-openagent is not registered" -> install the plugin in OpenCode
  configuration.
- Unclear task phrasing -> rewrite with explicit acceptance criteria before
  running.

## 7. Team Memory Note

Operational consistency matters more than stylistic variation.

If execution is required, use `lazycodex-ai run` with a verifiable objective and
explicit end-of-run evidence.

# Task H3 report: check registry and sandbox adapter with FAKE writer

Status: DONE_WITH_CONCERNS. Worktree: engram-improvement-lane-p, branch claude/improvement-lane-p.
Commits: 60cf3db feat(harness): add check registry and sandbox adapter with fake writer;
e4b0812 docs(harness): refresh progress last commit after H3.

## Implemented
- docs/harness/checks/registry.json (check-registry-v1): image pinned by digest
  python@sha256:02108f5d322dd89f1c9e552442c25acb0543dfdbc455693a5599624f20d9155d (multi-arch index digest of
  python:3.12-slim; one authorized `docker pull python:3.12-slim` was done locally), sandbox limits, fixed argv per
  check ID. IDs reuse docs/harness/schemas/check-catalog-v1.json (registry must be a subset; unknown ID refused).
  Only `pr_title_policy` is runnable in this image (accept case only; reject cases need an expected non-zero exit
  and stay in sensors.sh). Cargo checks need a toolchain image (later task).
- docs/harness/bin/sandbox-adapter.py: run_isolated(manifest_path, worktree_path, run_dir) -> RunOutcome with
  status, exit_code, argv, started_at, finished_at, log_paths, limits_enforced (+ checks, reason, supervisor_limits,
  manifest_sha256, executed_on_host=false). CLI: `run`, `validate-registry`; exit 0 passed / 1 failed,timeout,error /
  2 refused / 3 unavailable. Keyword-only injection for tests: registry_path, catalog_path, docker_bin, tcb_files.
  See module docstring for the contract. Key points: only the docker CLI is spawned on the host (argv lists, never
  a shell; check argv goes to Docker as entrypoint+command); --pull never; --network none; --cap-drop ALL;
  no-new-privileges; non-root uid (host uid, 65534 if root); --read-only; tmpfs /tmp noexec; memory=swap, pids, cpus;
  --init; constant env allowlist; mounts /work (rw worktree) and /tcb (ro supervisor-staged snapshot of
  manifest/registry/catalog/tools); run_dir and live TCB never mounted; limits_enforced is read back from
  `docker inspect` after create and the run refuses to start (status error) if Docker did not apply a policy
  value; timeout = docker kill + rm -f + label-based verification that nothing survives (survivor = error);
  outcome.json and logs (capped, sha256) written exclusively by the supervisor, O_EXCL, never overwritten;
  manifest validated as task-v2 with validate-evidence.py (expected policy_version from registry) and must grant
  sandbox_container + network_none; worktree containing the live adapter or manifest/registry is refused.
- docs/harness/tests/fake_writer.py (runs in the container; 13 hostile/benign behaviors),
  sandbox_test_support.py (fixtures + recording fake docker CLI), test_sandbox_adapter.py (37 OFFLINE tests),
  test_sandbox_smoke.py (14 real-Docker tests), bin/run-sandbox-smoke.sh (PASS | FAIL | UNAVAILABLE exit 3).
- Lane wiring: run-offline-lane.sh gained component sandbox_unit (floor 30, now 7 components);
  test_offline_lane.py updated (counts, components=7, new contract test for the component); doctor.sh requires the
  new files/exec bits and the lane grep. The docker smoke is NOT in the lane/sensors/CI (a unit test asserts the
  lane's active lines never mention docker or the smoke runner).
- Lane-p log entry appended (PT-BR); progress.md Last commit refreshed to 60cf3db.

## TDD evidence
- RED: `python3 -m unittest discover -s docs/harness/tests -p 'test_sandbox_adapter.py'` before the adapter existed:
  `FileNotFoundError: .../docs/harness/bin/sandbox-adapter.py` ... `Ran 1 test in 0.000s FAILED (errors=1)`.
  Expected: the module under test did not exist (this is a missing-module RED; behavioral REDs were not captured
  individually because tests and implementation were written in the same sitting, then mutation-checked below).
- GREEN: same command: `Ran 37 tests in ~23s OK` (output pristine; run with -W error::ResourceWarning too).
- Test sensitivity (mutation): with the adapter's read-back verification disabled, removing each of: tcb readonly,
  --network none, --cap-drop ALL, --pids-limit, no-new-privileges, memory limits makes the real-Docker smoke FAIL
  (1-2 failing tests each); removing --read-only initially did NOT fail (non-root cannot write /etc anyway) so
  `/var/tmp/engram-escape` was added to the write-outside attempts, after which that mutation fails too. With the
  read-back enabled every mutation is already stopped before start (status error, 12 failures). Adapter restored
  (cmp against backup).

## Verification (all exit 0)
- `python3 -m unittest discover -s docs/harness/tests -p 'test_sandbox_adapter.py'`: 37 tests OK.
- `bash docs/harness/bin/run-sandbox-smoke.sh`: `SANDBOX_SMOKE: PASS tests=14` (Docker 29.4.0, arm64, macOS).
  Without docker on PATH: `SANDBOX_SMOKE: UNAVAILABLE reason=docker CLI not found...` exit 3 (also unit-tested).
- `bash docs/harness/bin/run-offline-lane.sh`: `OFFLINE_LANE: PASS components=7 checks=237`
  (sandbox_unit 37, lane_contract 12, validator_unit 76, self 18, fixtures 39, live_state 19, review_gate 36).
- `bash docs/harness/bin/doctor.sh`: OK (one pre-existing WARN: no review artifact for active task).
- check-live-state --progress: PASS. Post-run: no containers with label engram.sandbox.run left; no temp TCB dirs.
- The pre-commit hook ran on the commits (no Rust changes) and passed.

## Deviations / notes
- Extra files beyond the brief list (justified): bin/run-sandbox-smoke.sh (distinct UNAVAILABLE state, keeps Docker
  out of the lane), tests/test_sandbox_smoke.py (real-Docker split from offline unit tests),
  tests/sandbox_test_support.py (shared fixtures). Modified unlisted files: run-offline-lane.sh, test_offline_lane.py,
  doctor.sh (required by the lane instruction).
- Catalog not modified (no new IDs): the registry holds only IDs already in the catalog.
- sandbox-adapter.py is exactly 800 lines (the file-size ceiling); it was trimmed to fit. Consider splitting
  (registry vs runner) in a later task if it grows.

## Concerns / NOT RUN
- Smoke executed only on macOS Docker Desktop (arm64). Linux/x86 CI behaviour (uid mapping of the bind-mounted
  worktree, cgroup swap accounting) is unverified; the read-back check would refuse rather than run if limits
  differ. The CI lane does not run the smoke (and I did not touch .github/workflows).
- A remote Docker context selected via DOCKER_CONTEXT/config is not detected (only a non-unix DOCKER_HOST is refused).
- Manifest target_sha is recorded but not bound to the worktree HEAD (belongs to the runner task).
- Registry runs the checks' code from the (untrusted) worktree by design; that is what the sandbox is for.
- `pr_title_policy` registry entry covers the accept case only.
- No independent review performed (controller's job).

---
# Fix report, round 1 (commit e2a2930 fix(harness): pin docker endpoint and tighten sandbox read-back; refresh commit 5d9c04c)

Platform correction: all earlier "Docker Desktop" wording is wrong. The smoke ran on **OrbStack** (docker context
`orbstack`, socket unix://~/.orbstack/run/docker.sock, Docker 29.4.0, arm64).

## Changes
1. Remote engine detection (important). `connect_docker` resolves the endpoint once: DOCKER_HOST, else
   `docker context inspect --format '{{.Endpoints.docker.Host}}'` (honors DOCKER_CONTEXT and the config
   currentContext). Anything not `unix:///...` -> `unavailable` (ssh, tcp, npipe, relative). All later calls run with
   `DOCKER_HOST=<resolved socket>` and DOCKER_CONTEXT removed (verified empirically that DOCKER_HOST wins over
   DOCKER_CONTEXT); the endpoint is recorded as `docker_endpoint` in the outcome. A docker socket inside the worktree is refused.
   Tests (EndpointTests, stub docker): DOCKER_CONTEXT ssh/tcp, config currentContext ssh/tcp/npipe, DOCKER_HOST ssh/tcp/
   relative -> unavailable and no create/version call; pinned endpoint (exactly one `context` call, every other call has
   DOCKER_HOST and no DOCKER_CONTEXT); DOCKER_HOST wins; unresolvable context; socket inside worktree.
2. Vacuous total-timeout test replaced: manifest budget 3 s, stub start sleeps 2 s, per-check limit 30 s; asserts
   status timeout, exactly 1 check record (passed), start count 1, reason names "manifest timeout of 3s" and the skipped
   check, supervisor_limits.total_timeout_seconds == 3. Mutation `if remaining < 1:` -> `if False:` fails it (checked).
   Also fixed a latent truncation: per-check timeout now uses ceil(budget) instead of int(budget).
3. Minor items:
   - Read-back: SecurityOpt exactly no-new-privileges (seccomp/apparmor/label overrides rejected), Pid/Ipc/UTS not
     host/foreign, Devices empty, RestartPolicy none, /tmp tmpfs must carry noexec+nosuid+nodev and the configured
     size (bytes form accepted), Entrypoint/Cmd must equal the registry argv. ReadBackTests use crafted inspect
     documents (good document = no problems; 26 single-fault variants each detected) plus stub-level modes.
     Also confirmed against real OrbStack inspect output (smoke passes with the stricter rules).
   - validate_argv: scans every arg (not only leading) for inline-code flags (shells/python: -c family; perl/ruby/
     node/php/lua: -c/-e/-E/-p/-r family, --eval/--print/--run/--command); forbids wrapper argv0 (env, timeout, nice,
     setsid, stdbuf, xargs, nohup, time, command, ionice, taskset, unshare, strace, flock, runuser, script, watch,
     sudo, su, doas, nsenter, docker, podman, chroot, eval, exec) and awk/gawk/sed (program-as-argument).
     Plain script argv remain registrable (test). Lane-p log wording corrected to describe exactly this rule.
   - Outcome: `target_sha` -> `manifest_target_sha` plus `target_sha_bound: false`.
   - run_dir: lstat must be a directory owned by the euid with no group/other write bit (checked before use and
     after creation, chmod 0700); TCB staging dir inside the worktree refused (TMPDIR case); worktree that is or
     contains $HOME refused; docker socket inside worktree refused.
   - LogPump: I/O errors (create/write/close) are captured while still draining the pipe, and make the check status
     `error` ("evidence capture failed"); test patches os.open to refuse the stdout log.
   - Per-check `limits_enforced` stored in each `checks[]` record (also for refused-to-start containers).
   - Smoke assertions: `assertTrue(status, "passed")` replaced by assertEqual plus endpoint/limits checks; the
     credential path probe now asserts each probe exists in the fake writer output (/tmp/.ssh, /root/.ssh,
     /var/run/docker.sock, /run/secrets) and is not visible, instead of an all() over a possibly empty list.
   - Split: registry loading and argv policy moved to docs/harness/bin/sandbox_registry.py (pure, no subprocess,
     asserted by a test); sandbox-adapter.py keeps the CLI entry point and re-exports the names and is now 759 lines
     (registry module 192). doctor.sh requires the new file.

## Evidence (all exit 0)
- `python3 -W error::ResourceWarning -m unittest discover -s docs/harness/tests -p 'test_sandbox_adapter.py'`: Ran 48 OK.
- Mutations against the unit suite (each restored byte-identical afterwards): budget guard removed -> 1 failure;
  security_opt rule removed -> 7 failures; DOCKER_CONTEXT left in the CLI env -> 2 failures; log errors ignored -> 1;
  run_dir owner/mode check removed -> 3; inline-flag scan limited to the first arg -> 2.
- `bash docs/harness/bin/run-sandbox-smoke.sh`: `SANDBOX_SMOKE: PASS tests=14` (OrbStack).
- `bash docs/harness/bin/run-offline-lane.sh`: `OFFLINE_LANE: PASS components=7 checks=248` (sandbox_unit 48).
- `bash docs/harness/bin/doctor.sh`: OK (same pre-existing review-artifact WARN). check-live-state: PASS.

## Remaining concerns
- Smoke only on macOS/OrbStack arm64; Linux/x86 CI unverified (read-back refuses rather than runs if limits differ).
- target_sha is recorded but still not bound to the worktree HEAD (target_sha_bound=false is explicit).
- OrbStack/other engines whose socket is symlinked are accepted by path only; the engine itself is not authenticated.
- Docker contexts that use `unix://` to a socket forwarded to a remote machine (e.g. ssh -L) cannot be detected.

---
# Fix report, round 2 (commit 93ff80b fix(harness): restore shadowed sandbox tests and match attached inline flags; refresh 4d341a0)

1. Shadowed class (important). My round-1 edit had turned the `CreateArgsTests` header into a second `class CliTests`
   (first at line 385, second at line 735), so the 5 create-args tests never ran. Renamed the first back to
   `CreateArgsTests`. Count: 48 -> 54 (+5 restored, +1 new guard), all pass.
   Guards: `ModuleHygieneTests.test_no_duplicate_test_case_classes_or_methods_in_the_sandbox_test_modules` (ast over
   test_sandbox_adapter.py and test_sandbox_smoke.py; checked to fail when the duplicate is reintroduced), and the
   offline-lane floor for `sandbox_unit` is now the exact count 54 (test_offline_lane.py stub count updated to 54), so a
   silently shadowed/dropped test makes the lane fail.
2. Attached inline flags (minor). `SHELL_INLINE_RE` / `INTERPRETER_INLINE_RE` now match by prefix (`re.match`) for short
   flags, and long flags are compared before `=` (`--eval=1`, `--command=id`). New rejected cases: `python3 -cprint(1)`,
   `python3 -c'print(1)'`, `bash -cid`, `perl -eprint 1`, `perl -e'print 1'`, `ruby -eputs 1`, `ruby -e'puts 1'`,
   `node -p1`, `node --eval=1`, `bash --command=id`. Reverting to fullmatch makes 8 registry assertions fail (checked).
   Known over-match (fail-closed): interpreter flags such as `perl -Mstrict` are rejected because the letter run
   contains an inline letter; register a script that does not need them.

## Evidence
- `python3 -W error::ResourceWarning -m unittest discover -s docs/harness/tests -p 'test_sandbox_adapter.py'`: Ran 54 tests OK.
- `bash docs/harness/bin/run-offline-lane.sh`: `OFFLINE_LANE: PASS components=7 checks=254` (sandbox_unit 54, floor 54).
- `bash docs/harness/bin/doctor.sh`: OK. check-live-state: PASS. Real-Docker smoke not re-run (no adapter runtime code
  changed this round; only the argv policy regexes and tests).

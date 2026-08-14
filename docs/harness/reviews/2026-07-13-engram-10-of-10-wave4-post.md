REVIEW_VERDICT: PASS Wave 4 integration meets the security, SDK live-contract, quality-budget, and dry-run publication guards without fake success or publication claims

## Scope

- **Reviewer role**: independent; did not implement Wave 4.
- **Range**: `origin/main..HEAD` on `feat/engram-10-of-10-wave4`
- **Base main**: `962655a90f9bb05497fcc54fd78952c87ce19712`
- **Evidence consulted**:
  - `.omo/start-work/SESSION_HANDOFF_2026-07-13_WAVE4.md`
  - `.omo/evidence/task-{22,23,24,26,27,37,38,39}-engram-10-of-10.md`
  - `docs/harness/progress.md` Wave 4 section
  - `docs/harness/progress/2026-06-27-harness-live-state-closeout.md`
  - `docs/harness/CODE_REVIEW_POLICY.md`, `docs/harness/GATES.md`
- **Spot-checked surfaces**: realtime WS server/config/events, HTTP transport router/security, quality-budget checker, release/SDK workflows and verify scripts, Python/TS live drivers, progress truthfulness.

## Checklist

### 1. WebSocket workspace / Origin isolation and filter-before-serialize — PASS

- Handshake binds one normalized workspace (`/ws?workspace=…`, default `default`) and rejects unauthorized workspace before upgrade (`403`).
- Browser Origin: exact allowlist via `ENGRAM_WS_ALLOWED_ORIGINS`; wildcard/`*`, malformed, non-HTTP(S), userinfo, path/query, and multi-Origin fail closed; missing Origin remains non-browser path.
- Send path gates on `client_matches_event` **before** `serde_json::to_string` / write (`event.workspace() == Some(client.workspace)` plus subscription filter).
- Memory create/update/delete producers attach authoritative `data.workspace`.
- Regression: `real_client_receives_only_its_authorized_workspace` proves private sentinel never serializes to a default-workspace peer.

### 2. WebSocket caps, idle timeout, bounded writes, cleanup, no-PII metrics — PASS

- Env bounds: connections default 128 / hard max 10k; message bytes default 64 KiB / hard max 16 MiB; idle default 60s / hard max 3600s; zero/malformed/over-max fail closed at startup (`RealtimeConfig::from_env` → `Result`).
- Cap+1: semaphore `try_acquire_owned` → HTTP `503` before upgrade.
- Oversized frames: transport limit + app check → close `1009`; idle → close `1008`.
- Writes/close timeout-bounded (`WS_WRITE_TIMEOUT` 10s, `WS_CLOSE_TIMEOUT` 1s).
- Drop guards release registration, semaphore permit, and active gauges on disconnect/cancel.
- `/health` resources snapshot is aggregate counters only (no principal, token, workspace, or message dimensions).

### 3. HTTP timeout / auth / body ordering; no detached timed-out mutations — PASS

- Tower order (outer → inner): `enforce_request_timeout` → `enforce_mcp_auth` → `DefaultBodyLimit` → handler.
- Auth runs before body collection; unauthenticated oversized body returns `401` (not `413`).
- Authenticated `limit+1` → `413` before JSON parse; slow partial body → `408` / JSON-RPC `-32008`.
- No `spawn_blocking` / detached handler work under `src/mcp/http_transport`.
- Real-binary regressions in `tests/http_transport_security.rs` cover body, auth-before-body, timeout cleanup, notification `202`, and SSE setup-vs-established stream semantics.
- Invalid HTTP limit env values fall back to documented 1 MiB / 30 s defaults (cannot select unbounded mode).

### 4. Installed-wheel Python and packed-npm TypeScript live behavior and negatives — PASS

- Python: build/install wheel into isolated venv; assert package path under venv not `sdks/python`; live CRUD/search/close/context-manager against real loopback authenticated server; wrong-bearer and killed-server negative modes in `scripts/test-python-sdk-live.sh` + `test_live_client.py`.
- TypeScript: `npm pack` → install tarball in temp consumer; import only from packed package; happy + wrong-bearer + missing-endpoint; CI workflow runs all three modes.
- Receipts and local logs record green happy and negative paths; no registry publish in these lanes.

### 5. Quality-budget resistance to editing all apparent baselines together — PASS (with residual)

- Checker binds Criterion `before_criterion` to `git show <source_revision>:<baseline_source_path>`.
- Self-test multiplies before/after/hot_path baseline copies and requires fail with immutable-source error.
- Retrieval floors require after == floors == observed and after ≥ before; CI required job runs checker with `fetch-depth: 0`.
- Residual (non-blocking): retrieval floor reduction is not git-object-bound the way Criterion is; simultaneous edit of floors + retrieval artifact + before/after metrics could still lower floors under process review. Criterion all-copy attack (review-2 FAIL root cause) is closed.

### 6. Release / SDK workflow SHA / ref / category guards — PASS

- Release preflight: full 40-char SHA must match `github.sha` and checked-out HEAD; live path requires exact version tag; dry-run rejects homebrew_only/overwrite write modes.
- SDK preflight: dispatch-only; dry-run forbids publish/tag; publish requires single channel, signed annotated `v0.22.0` tag, tag/SHA agreement, and existing immutable GitHub Release; publish jobs revalidate after environment approval.

### 7. Artifact digest, SBOM, Sigstore/OIDC, provenance binding — PASS

- Bundle creates SHA-256, CycloneDX 1.5, SPDX 2.3, SLSA provenance, internal signed attestation, and GitHub `attest-build-provenance` (OIDC) bundle.
- Offline verify binds archive digest, SBOM digests, provenance subject/SHA/target, attestation schema, and (when trusted) `gh attestation verify` against repo `aiconnai/engram`, signer workflow `.github/workflows/release.yml`, and `--source-digest`.

### 8. Dry-run paths cannot reach publish jobs — PASS

- Release: `release` and `update-homebrew` require `dry_run == false`; dry-run summary asserts no publication.
- SDK: `publish-python` / `publish-npm` require `dry_run == false && publish == true && channel match`; dry-run forbids publish in preflight.

### 9. No fake-success skip / ignore / allowlist / continue-on-error weakening — PASS

- Wave 4 release/SDK/live workflows do not introduce PR-blocking `continue-on-error` or ignore allowlists for required gates.
- Existing benchmark-action `continue-on-error` remains non-PR-only; PR path still has `fail-on-alert` and the separate required quality-budget step.
- Quality checker and release verify self-tests fail closed on mismatch/tamper.

### 10. Truthful progress and publication claims — PASS

- Progress and live-state closeout:
  - dry-run `29286930522` bound to exact SHA `2cfab4563d5da43932c1cc3aa6741eeea6b487ea`;
  - Release/Homebrew skipped; no tag/registry/release/Homebrew/deploy write;
  - **no** claim that v0.22.0 or any SDK is published;
  - SQLite descriptor-bound opening still deferred;
  - Todo 24 human `autorizado` scoped only to integrating remediated SHA `6a42f00…`, not publication;
  - integration HEAD advanced to closeout `e17544a…` while dry-run remains historically bound to `2cfab45…`.

## Findings

- [LOW] No blocking issues found in the Wave 4 integration range for the required checklist. Security isolation, resource bounds, HTTP ordering, live SDK package contracts, quality-budget Criterion immutability, release/SDK guards, artifact attestation, dry-run non-publication, and progress truthfulness hold under code and receipt review.

## Residual risks (non-blocking)

1. **Retrieval floor multi-edit** — Criterion baselines are git-revision-bound; retrieval floors still rely on co-edited JSON floors + observed metrics + update metadata. Mitigated by required reviewer/rationale/evidence fields and human review, not by immutable object binding.
2. **Python live CI negatives** — happy path is CI-wired; wrong-bearer/killed-server modes exist in the driver and receipts but are not separate required workflow steps (TypeScript CI does wire negatives).
3. **SSE omit-workspace semantics** — WebSocket always binds one workspace; SSE still allows omit-workspace for principals that authorize `None` (including anonymous loopback → effectively unfiltered event types). Adjacent surface; not the Todo 37 WS contract. Consider fail-closed default workspace for anonymous SSE if multi-tenant SSE hardens later.
4. **SQLite descriptor-bound open** — still deferred; Wave 4 does not claim Wave 2 SQLite race closure.
5. **Publication** — dry-run success is not channel approval; Todos 36 / 52–55 still need fresh explicit write approvals.

## Verdict rationale

Wave 4 delivers the stated foundations (WS isolation + bounds, HTTP lifetime/body bounds, live SDK package contracts, quality floors in required CI, attested dry-runnable release and gated SDK lanes) without weakening gates or asserting publication. Independent review of code paths and durable receipts supports merge-readiness of the integration branch subject to normal CI, not release/publish authorization.

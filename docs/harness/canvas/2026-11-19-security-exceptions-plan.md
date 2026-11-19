# Review Canvas: security-exceptions-plan-2026-12-31

Date: 2026-11-19
Owner: owner-selected Frente 3 (`chore/security-exceptions-plan-2026-12-31` over `origin/main` `531a8012`)
Scope: Anchor `scripts/test_check_security_exceptions.py` and `scripts/check-security-exceptions.py` on `[metadata].as_of` in `docs/security/advisory-exceptions.toml` (replacing the hardcoded `TODAY = date(2026, 10, 5)` constant that blocked post-`2027-01-03` horizon reviews), record the empirically verified remediation plan for the three `2026-12-31` RustSec advisories (`RUSTSEC-2026-0192` `ttf-parser`, `RUSTSEC-2024-0436` `paste`, and `RUSTSEC-2026-0543` `chrono`) in `docs/harness/plans/2026-11-19-rustsec-2026-12-31-remediation-plan.md`, link it from `RISK-0008` in `docs/harness/risk-register.yaml`, and sync harness live-state.

## Trigger

| Trigger | Evidence |
|---|---|
| Hardcoded `TODAY = date(2026, 10, 5)` in `scripts/test_check_security_exceptions.py:29` enforces `expires <= 2027-01-03` in the `Security Gate` CI job (`.github/workflows/ci.yml:527`, transitively gating required `Test (ubuntu-latest)`), diverging from `[metadata].as_of` in `docs/security/advisory-exceptions.toml` | `scripts/test_check_security_exceptions.py`, `scripts/check-security-exceptions.py`, `docs/security/advisory-exceptions.toml` |
| All 3 governed RustSec exceptions (`RUSTSEC-2026-0192`, `RUSTSEC-2024-0436`, `RUSTSEC-2026-0543`) expire on `2026-12-31` (`42` days) | `docs/security/advisory-exceptions.toml`, `docs/harness/risk-register.yaml` (`RISK-0008`) |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Bump hardcoded `TODAY = date(2026, 11, 19)` in `test_check_security_exceptions.py` vs derive the review anchor from `[metadata].as_of` in `advisory-exceptions.toml` and enforce `as_of <= today` in `check-security-exceptions.py` | Accepted (`[metadata].as_of` anchor) | Keeps unit tests 100% deterministic offline (no `date.today()` time-bomb in `HorizonTests`) while eliminating manual two-file date drift and rejecting future/missing `as_of` values in CI |
| Big-bang rewrite of `chrono` before touching `ttf-parser` / `paste` vs sequenced Waves `W1-PDF` -> `W2-ONNX` -> `W3A/W3B/W3C` | Accepted (sequenced waves) | Live crates.io inspection proved `ttf-parser` (`lopdf 0.47`) and `paste` (`tokenizers 1.0`) can be eliminated in small, non-breaking `v0.24.x` PRs, whereas `arrow-array 60.0.0` (used by `duckdb`) still hard-depends on `chrono ^0.4.40` and `engram-types` exposes `DateTime<Utc>` in its public Rust API (`v0.25.0` scope) |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| Runtime Rust / MCP / SDK code | `O(0)` (zero runtime code changed) | `O(0)` | CI Python validator (`O(R)` over `R = 3` exception records), unit tests, and governance/planning docs only |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| TOML datetime (`2026-11-19T00:00:00Z`) passed where a TOML local-date (`2026-11-19`) is required (`isinstance(datetime, date)` is `True` in Python) | `local_date` in `scripts/check-security-exceptions.py` rejects `isinstance(value, datetime)` |
| `[metadata]` table or `as_of` key missing from `advisory-exceptions.toml` | `test_missing_metadata_as_of_is_rejected` in `scripts/test_check_security_exceptions.py` |
| `[metadata].as_of` set in the future (`as_of > today`) to game the 90-day horizon window | `validate_records` rejects `as_of > today`; verified by `test_future_metadata_as_of_is_rejected` |
| Record `expires` updated without updating `as_of` (`expires > as_of + 90 days`) | `validate_records` enforces both `today` and `as_of` 90-day horizons |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| Existing callers of `MODULE.validate_records(records, ..., today)` break if `as_of` is positional | Medium | Make `as_of: date \| None = None` keyword-only with backwards-compatible default `None` | `python3 -B scripts/test_check_security_exceptions.py` (`8/8 OK`) + `--self-test-expired` + `--self-test-missing-owner` |
| Prematurely extending `expires = 2026-12-31` without executing Wave 1/2 fixes | High (violates `GATES.md` no-renewal-of-fixable-advisories rule) | Keep all 3 `expires = 2026-12-31` dates unchanged; update only `[metadata].as_of = 2026-11-19` | `git diff docs/security/advisory-exceptions.toml` (`+1/-1` line only) |
| Moving `origin/main` past `531a8012` if the owner plans to tag `v0.24.1` on `531a8012` first | Medium | Open PR without merging automatically so `origin/main` remains at `531a8012` until the owner chooses merge order | `git rev-parse origin/main` remains `531a80121d13621dff44a7899b1af6360e60b4eb` |

## Decision

Proceed / split / block: **Proceed** (run full `sensors.sh`, `run-offline-lane.sh`, and independent post-review, and open PR against `main` while leaving `origin/main` at `531a8012` for owner merge/tag ordering).

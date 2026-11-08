# Release closeout evidence — 2026-11-07

Task: release-closeout-2026-11-07. Base main7ca7a9b8; working candidate starts at
SDK fix2c7544db on isolated branch `fix/release-closeout-2026-11-07`.
This is execution evidence, not a human receipt or publication authority.

## Current remote source

- PR264 merged on04/11 into7ca7a9b8. CI42371841522 and Standing42366783284 are green
  on07/11. Both live SDK push workflows passed on the same merged source.
- Full manual Nightly41966511833: seven successful jobs, including Miri and mutation.
  Scheduled07/11 Nightly42359394025 passed with mutation skipped by its lane; the
  manual run supplies the complete scope. No workflow was triggered for this observation.
- SDK dry-run41966515107: preflight PASS, Python check failure and npm install-path
  failure; publication jobs skipped. Fix2c7544db was local, not merged or remotely green.
- Exact tag0.24.0 query returned404; no open issues/PRs in the07/11 assessment.
  Registry versions/settings require their own current observations before operation.

## Bounded CDPATH follow-up

Independent Terra/xhigh static inspection identified that the original canonicalizing
`cd` inherits CDPATH. The selected directory can be printed twice into command
substitution or resolved under a competing search root. The supported relative-path
consumer scenario then fails despite valid artifacts.

Reproduction on2c7544db: the existing relative-path unittest with CDPATH=. failed
with `expected exactly one *.tgz, found 0` (1 test, exit1). The extended test
created an empty competing directory of the same basename and exercised empty,
dot and alternate CDPATH. Dot/alternate subcases failed before the fix (exit1).
Clearing CDPATH only for the canonicalizing cd made all15 workflow tests pass
(2.474s, exit0). This is real npm/node in offline mode, not a mock of installation.
The current build/check tool floor and all publication guards remain unchanged.

## Documentation integration

- Integrated the useful step6 architecture/template cleanup without overwriting
  the SDK branch's progress or borrowing its sensor receipt. CLAUDE points to the
  actual shared Markdown core rather than duplicating volatile feature/tool counts.
- Reconciled main integration/Nightly/signer/Cargo facts in readiness, original-plan
  entry point, known-issues and risk mitigation. RISK0010/0011 remain open pending
  owner closure; main integration is recorded without claiming publication.
- Added self-contained evolution v4 with EV- IDs. PH0/P0 code is integrated; managed
  projection, durable continuity, evidence/setup and model experiments remain distinct.
  Optional experiments have their own terminal decisions rather than blocking the
  first essential integrated result. RFC0004a remains a proposal.
- Consumer preparation covers cache-clear and Markdown import/export notes. None sent.

## Verification status

The first combined full sensor run failed at2026-11-07T15:34:14Z (545s): the
new Last evidence bullet lacked its required link to the active log entry.
All other deterministic stages and doctor passed. The link was restored without
test changes; all50 context-budget tests passed when local Git fixture clones
were allowed. A full retry follows. The failed sensor history/log is retained.

Package checks were repeated: Python3.10/setuptools84.0.0 built wheel+sdist;
Twine6.2.0/packaging26.3 accepted both. Node22.22.3 type-check/build and104 tests
passed (CI uses Node24). The initial TS test run was blocked by sandbox listen
EPERM; the identical authorized socket-enabled run passed. Npm pack succeeded
after allowing its cache write; no cache ownership or global configuration changed.

Artifact-backed live verification returned PASS: Python happy scenario1 passed,
2 other scenario variants unselected/skipped; npm CRUD/search and explicit
wrong-auth/cross-workspace negatives passed. Source identity is2c7544db; SDK product
bytes match main. Final candidate metadata/review binding is recorded after freeze.
Channel observations were repeated outside the network-restricted sandbox and all
checks passed; no observed version changed. Matrix freshness was refreshed only
after that successful read-only observation.

| Check | State |
|---|---|
| Initial bootstrap/doctor | PASS; doctor retains the preexisting active-review warning |
| SDK regression RED/GREEN | Two new CDPATH subcases failed before fix; full15 tests PASS after fix |
| Syntax/shellcheck, links/doctor | Shellcheck PASS;11 document link checks PASS; doctor PASS with preexisting review warning; final refresh follows |
| Full sensors | Retry PASS at2026-11-07T15:46:02Z,417s, no exclusions; first545s context-pointer failure retained |
| Real packages/installed checks | Wheel/sdist/Twine and npm tarball/104 tests PASS; both clean installs/live checks PASS with the selected Python scenario and explicit negative checks |
| Independent final-candidate opinion | Static PASS on the frozen28-path snapshot; final identity/metadata reconciliation stays separate from human H1 |
| Human H1 / upstream / corrected remote dry-run | Pending; no receipt issued by this agent |

Command logs and frozen completion bundle are kept outside the worktree in
`/private/tmp/engram-release-closeout-run-20261107/`. Final results are appended
below; pending entries above are the preparation snapshot, not completion claims.

## AI usage

| Native participant | Work | Requested model/effort → reported | Input/output/cache/reasoning tokens |
|---|---|---|---|
| SDK reviewer | Initial SDK delta; CDPATH finding | gpt-6.1-terra/xhigh → not exposed | Not observed |
| Final reviewer | Combined frozen candidate and identity reconciliation | gpt-6.1-terra/xhigh → not exposed | Not observed |
| Plan researcher | Current dispositions/evolution boundaries | gpt-6-luna/max → not exposed | Not observed |
| Integrator | Edits, independent reproduction and serial checks | Host configuration not exposed | Not observed |

These are three distinct read-only delegates plus the integrator. Delegates ran
no builds, children or external inference. Calls/retries and receipts are recorded
in the completion registry. Effective model/effort and financial cost are not
exposed; requested settings are not authenticated runtime telemetry.

## Combined deterministic completion

The full retry completed with exit0 at2026-11-07T15:46:02Z in417 seconds, without
exclusions: required-feature format/Clippy/tests, binaries, WASM, rustdoc, MCP
reference, offline harness, title policy and doctor. Negative title fixtures
intentionally print FAIL while the aggregate is PASS. The known active-review
warning remains. Both full runs are in the tracked append-only sensor history;
the first raw log is sensors-full-first.log and the retry is sensors-full.log.

Post-run updates below change documentation/evidence fields only; executable
workflow/verifier/test bytes retain their tested hashes. Final immutable review,
commit identity and human-only H1 procedure are kept in the completion bundle.
The corrected remote SDK run has not been dispatched and publication remains blocked.

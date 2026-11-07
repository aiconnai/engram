# Review canvas — release-closeout-2026-11-07

Base: main7ca7a9b8 plus existing SDK patch2c7544db. Scope: verifier CDPATH
follow-up, integration documentation and release preparation; no core/API/schema change.

| Alternative | Decision |
|---|---|
| Leave relative tarball path untouched | Rejected: real consumer changes cwd and fails ENOENT |
| Canonicalize with inherited CDPATH | Rejected by regression: output contains an extra path or searches an alternate root |
| Clear CDPATH for the canonicalizing cd | Selected: preserve the validated caller-relative directory and spaces |
| Remove License-File from package | Rejected: keep license/metadata; use packaging>=24.2 in build/check environment |
| Reopen completed release/Miri/Markdown work | Rejected: reconcile exact merged evidence and work only on remaining blockers |

Hot path: one directory resolution per artifact verification; no package-content,
runtime or registry behavior change. Existing checksum/archive/install guards remain.

| Edge / breakage risk | Evidence / containment |
|---|---|
| Relative path with spaces and CDPATH=. | Existing real-npm regression extended; RED then GREEN |
| Competing CDPATH directory with the same basename | Empty decoy directory; verifier must use the validated caller directory |
| New docs reuse old PASS/receipt | Freeze new candidate and generate a separate operator bundle; historical opinions immutable |
| Source from step6 overwrites SDK-fix progress/sensors | Integrate only intended architecture/template cleanup; recompute current evidence fields |
| Optional evolution blocks essential delivery | Namespaced v4 outcomes; contracts and operational decisions retain their own gates |

Required checks: SDK workflow suite, shellcheck/syntax, links/doctor, full sensors,
package verification where available, and independent candidate review. Human H1
and any upstream operation remain governed by the unchanged prior policy.

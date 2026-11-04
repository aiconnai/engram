# Cloud download cancellation — review canvas

Task: cloud-download-cancellation; base d9915e99. S1 requires cancellation without
residual partial files. The old future owns its guard while Tokio owns the pending
blocking open: cancellation can unlink before the open creates the temporary file.

## Approaches

1. Keep a guard outside each async filesystem operation: rejected because an
   abandoned blocking operation can outlive the guard, including on Windows.
2. Put preparation and commit in one blocking task: rejected because cancellation
   of queued preparation would also replace the target.
3. Use two blocking ownership transfers: prepare all file bytes and permissions,
   returning the cleanup guard; then commit by rename with the guard owned by that
   operation. Selected. Discarding either queued/in-flight result cannot strand a
   temporary file. A commit already queued may complete after cancellation.

## Complexity and boundaries

Preparation is O(N) time for N downloaded bytes; move the existing Vec into the
blocking task rather than copying it. Two blocking dispatches, one atomic rename,
file fsync and best-effort directory fsync. No network or database transaction is
inside these tasks. The guard starts owning the path only after exclusive creation.

## Edge cases and risk

| Case | Expected behavior / verification |
|---|---|
| Cancel with file preparation queued | Original destination unchanged; no temporary remains after the queue drains; deterministic channel-barrier regression |
| Cancel while preparation runs or its result awaits delivery | The blocking operation/result owns cleanup until the closed file can be removed |
| Preparation or rename fails | Original error propagates and the closed temporary is removed; existing failure suite |
| Cancel after commit is queued | Commit may finish; destination has one complete generation and no temporary remains |
| New/existing destination | New file remains owner-only; existing mode retained; existing tests |

Public behavior risk is limited to cancellation cleanup and internal scheduling.
No live databases or external services are used for verification.

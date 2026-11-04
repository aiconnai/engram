# Cloud download cancellation — PR 264 follow-up

Task: cloud-download-cancellation. Base: d9915e992c375751c3b88c5f5577f113b188a4bb.
Owner requested the fix after the xhigh two-axis review found S1 partially met.

## Scope and acceptance

- Reproduce cancellation while the real download's file creation is queued in a
  one-worker blocking pool. Use channel barriers and real temporary files, not sleeps.
- Keep creation, writing, permission handling and cleanup ownership together in a
  blocking preparation operation. A discarded preparation result cleans its file.
- Transfer the prepared file's cleanup ownership into a blocking rename/directory
  sync operation. Once commit is queued it may finish despite caller cancellation;
  the destination is atomically replaced in full or remains intact.
- Preserve public signatures, encryption validation, owner-only new files,
  existing destination modes, SQLite protections and directory-fsync warnings.
- Run RED/GREEN regression, cloud failure/encryption suites, format, cloud Clippy,
  full local sensors, independent review and the PR's remote checks. Keep the old
  receipt bound to its original HEAD; a new receipt is a human-only step.

No schema, dependency, SDK, MCP catalog, release or deployment changes.

Evidence: active [log](../progress/2026-10-26-remediation.md) and
[canvas](../canvas/2026-11-04-cloud-download-cancellation.md).

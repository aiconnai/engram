# Engram 10/10 Wave 2 — independent review v2

## Findings

- **Critical/High/Medium/Low:** no blocking findings.
- The earlier `/dev/fd` compatibility finding is resolved without weakening the
  storage boundary: only strict numeric aliases receive special handling,
  descriptor identity is checked by device/inode, normal paths retain `lstat`,
  and sidecars retain `openat` plus `O_NOFOLLOW` and metadata validation.
- HTTP, WebSocket, and gRPC remain fail-closed on non-loopback listeners without
  a key. Cloud keys remain durable with controlled rotation. PDF parsing remains
  isolated and resource-bounded.

## Independent verification

- Storage Unix: **7/7**
- HTTP security: **5/5**
- Listener configuration: **9/9**
- gRPC: **25/25**
- PDF worker: **1/1**

Non-blocking limitation: FreeBSD/OpenBSD runtime behavior was not executed on
this host; the shared fallback logic is covered by a unit test for strict alias
recognition.

REVIEW_VERDICT: PASS Wave 2 closes the Unix descriptor fallback finding without weakening symlink or descriptor-swap defenses.

FAIL

- [BLOCKER] `doctor.sh` now fails in this strict read-only Codex sandbox because the new default `.sensors-log` validation uses a Bash heredoc. Evidence: `bash docs/harness/bin/doctor.sh` exits 1 with `line 747: cannot create temp file for here document: Operation not permitted`; scoped code is at `docs/harness/bin/doctor.sh:706`. This makes the mandatory harness doctor gate fail in a static/read-only review environment.

REVIEW_VERDICT: FAIL

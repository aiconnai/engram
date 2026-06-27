FAIL shellcheck validation will hard-fail in shellcheck-enabled environments, and the review diff is incomplete.

- [BLOCKER] `doctor.sh` makes `shellcheck` blocking when installed, but changed scripts use dynamic `source "$BIN_DIR/lib.sh"` without a ShellCheck source directive or `-x`. Evidence: `doctor.sh:237` and `doctor.sh:243`; dynamic sources at `bootstrap.sh:23` and `sensors.sh:20`. This will produce SC1090-style failures where `shellcheck` exists.

- [HIGH] Review input is incomplete / scope-mixed. The diff adds links to `docs/USER_GUIDE.md` at `README.md:60` and `docs/README.md:10`, but that new file is not included in the supplied diff; live status also shows additional omitted files/changes such as `docs/harness/progress/2026-05-30-harness-bootstrap.md`, `.sensors-log`, and review/canvas artifacts. This blocks a trustworthy post-review.

REVIEW_VERDICT: FAIL shellcheck blocking is not portable and the supplied review diff is incomplete.

FAIL README still has unsupported CLI examples in review scope
REVIEW_VERDICT: FAIL README still has unsupported CLI examples in review scope

- [MED] README.md:290 still documents `engram-cli create ... --workspace`, README.md:293 documents `engram-cli workspace list`, README.md:305 documents `--tier daily`, and README.md:314/README.md:369 document unsupported `session`/`quality` subcommands. Current `engram-cli create --help` exposes only `--type`, `--tags`, and `--importance`; `engram-cli workspace|session|quality --help` returns unrecognized subcommand. This leaves user-facing docs with the same class of unsupported CLI examples fixed in `docs/USER_GUIDE.md`.

Other required checks passed: `engram-server --help` exposes `--transport`/stdio default and not `--mcp`; targeted docs have no remaining `--mcp` or old `--http`/`--port` examples except harness progress evidence; `docs/ieee-12207.md` is ignored and untracked; checked active Markdown links and JSON snippets are sane; no standards conformance claim, copied standard wording, hidden scope creep, or gate weakening found; latest inspected full sensors evidence is pass at `2026-06-26T14:52:36Z`.

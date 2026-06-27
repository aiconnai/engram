FAIL bootstrap does not satisfy the scoped read-only observable count requirement.

- [HIGH] [docs/harness/bin/bootstrap.sh](/Users/ronaldo/Projects/_aiconnai/engram/docs/harness/bin/bootstrap.sh:124): the new Python heredoc fails in this strict read-only review sandbox with `cannot create temp file for here document: Operation not permitted`, then falls back to `MCP tools (source total): 278`. That means the current observable bootstrap output is not `238 active / 278 total` for read-only agents, and it reintroduces the same heredoc/read-only compatibility class the harness recently fixed in other scripts. Use a heredoc-free invocation, a checked-in small helper, or another read-only-safe parser path.

Other scoped checks looked sound: `registry.rs` remains canonical via `include!("registry.rs")`, no live `tools/discovery` references were found outside historical evidence, and the new tests cover duplicate tool names plus absence of the orphan file. I did not run Rust tests because the environment is read-only.

REVIEW_VERDICT: FAIL bootstrap active/total count path is not read-only-sandbox compatible.

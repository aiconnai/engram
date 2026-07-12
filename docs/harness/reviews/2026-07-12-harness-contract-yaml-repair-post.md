PASS Single YAML quoting fix that prevents zero-job CI failure on merge-group events

- [LOW] Sensors ran in `quick` mode (fmt + check + pr-title-policy + doctor) rather than `full` mode (clippy + tests + integration + wasm + doc + ref_check). For a one-line YAML quoting change this is proportionate and low-risk, but it means clippy and the full test suite were not exercised against the current tree. Acceptable given scope; note for auditors.

REVIEW_VERDICT: PASS Single-character YAML quoting fix is correct, minimal, and safe; no gate weakening or scope creep detected.

# Test Fixtures

`mcp_mock_parity_scenarios.json` is the seed for the deterministic MCP parity
harness. Rust protocol tests execute these scenarios through the real MCP
`tools/call` path and compare normalized output, excluding volatile database IDs,
timestamps, scores, and generated UUIDs.

Future Python and TypeScript SDK parity should reuse the same scenario names and
fixture inputs, then compare each SDK's normalized public response shape against
the same `expected_normalized` block.

## retrieval_quality

`retrieval_quality/corpus.json` (v1) is the frozen smoke corpus compared byte for
byte with `baseline.json`. `retrieval_quality/candidate_corpus.json` (v2) is the
candidate-evaluation corpus (PT/EN, typos, negation, ambiguous entities,
duplicates, daily/transcript filtering, workspace isolation). Its relevance
labels were authored by an LLM and are not human reviewed (see its `labeling`
block and `docs/quality/retrieval-performance-policy.md`). Changing either file
changes its SHA256, which has no reviewed floors entry in
`docs/quality/candidate-floors.json` until a reviewed entry is added.

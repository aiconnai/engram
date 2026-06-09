# Test Fixtures

`mcp_mock_parity_scenarios.json` is the seed for the deterministic MCP parity
harness. Rust protocol tests execute these scenarios through the real MCP
`tools/call` path and compare normalized output, excluding volatile database IDs,
timestamps, scores, and generated UUIDs.

Future Python and TypeScript SDK parity should reuse the same scenario names and
fixture inputs, then compare each SDK's normalized public response shape against
the same `expected_normalized` block.

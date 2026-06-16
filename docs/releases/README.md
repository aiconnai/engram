# Release Notes and Versioning

This directory documents release hygiene for Engram.

## Release Notes Source

GitHub Release bodies are generated from the matching section in
[`../../CHANGELOG.md`](../../CHANGELOG.md) by
[`../../.github/scripts/extract-release-notes.sh`](../../.github/scripts/extract-release-notes.sh).

Do not re-enable GitHub-generated release notes for normal releases. Generated
notes can re-expand old PR lists on reruns and make a release body drift from
the reviewed changelog.

## Next Version Decision

Use the next patch version, currently `0.21.2`, for release automation,
documentation hygiene, claim hygiene, token/secret rotation, and other changes
that do not alter public runtime behavior.

Use `0.22.0` when the next release includes product-facing changes to MCP
tools, SDK APIs, CLI behavior, storage behavior, or agent workflows.

## Public Claims Gate

Before copying repository text into launch posts, docs homepages, package
descriptions, sales copy, or public comparison material, verify:

- current crate/package versions;
- generated MCP tool count from `docs/MCP_TOOLS.md`;
- shipped transports, SDKs, adapters, and examples;
- benchmark numbers against committed commands and raw artifacts;
- competitor features, licenses, hosted/local requirements, and activity from
  dated sources;
- community metrics such as stars, downloads, contributors, and adoption.

If a claim cannot be re-verified close to publication, keep it as internal
strategy input or phrase it as a product direction rather than a public fact.

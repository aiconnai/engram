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

Use the next patch version after the latest released tag (`latest_released_tag`
in [`channel-matrix.toml`](channel-matrix.toml)) for release automation,
documentation hygiene, claim hygiene, token/secret rotation, and other changes
that do not alter public runtime behavior.

Use the next minor version when the release includes product-facing changes to
MCP tools, SDK APIs, CLI behavior, storage behavior, or agent workflows. The
version in `Cargo.toml` must already equal the tag you intend to push; the
release and SDK workflows fail closed when they differ.

## Prepared 0.24.0 candidate

See the [readiness decision](v0.24.0-readiness.md),
[SDK 0.7.0 notes](v0.7.0-sdk-notes.md) and
[cache-clear migration/consumer notice](../migrations/v0.24.0-cache-clear.md).
These documents prepare the release; publication remains pending.

## Before Tagging

Run these from the exact commit that will be tagged:

1. `python3 scripts/bump-version.py --check` (versions, lockfiles, changelog).
2. `python3 scripts/bump-version.py --release-plan --release-tag vX.Y.Z`
   (the mapping the SDK workflow re-derives; see
   [`SDK_PUBLISHING.md`](SDK_PUBLISHING.md#version-mapping)).
3. Re-observe every registry and GitHub release and correct any stale
   `observed_*` value in [`channel-matrix.toml`](channel-matrix.toml)
   (`python3 scripts/check-release-channels.py --matrix docs/releases/channel-matrix.toml --read-only`),
   then `python3 scripts/bump-version.py --refresh-matrix`. The matrix records
   observations only; never edit them without observing the registry.

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

# SDK Publishing

The Python and TypeScript SDKs version independently from Engram core. The
authoritative versions and tested core ranges are recorded in
[`channel-matrix.toml`](channel-matrix.toml).

## Safe default

`.github/workflows/sdk-release.yml` is dispatch-only and defaults to
`dry_run=true`. A dry-run builds and verifies the selected Python wheel/sdist
and/or npm tarball, uploads them as workflow artifacts, and has no registry
credentials or registry-write job in its execution graph.

Use the exact candidate SHA when dispatching:

```bash
RUN_ID="$(bash scripts/dispatch-and-wait-workflow.sh \
  --workflow sdk-release.yml \
  --ref "$(git branch --show-current)" \
  --field channel=all \
  --field dry_run=true \
  --field sha="$(git rev-parse HEAD)")"
bash scripts/verify-sdk-artifacts.sh --run-id "$RUN_ID" --live
```

The verifier checks package names, independently versioned package metadata,
core compatibility ranges, SHA binding, checksums, compiled npm entry points,
and clean installation of both packages. `--live` additionally runs the real
server SDK contracts.

## Protected publication

Publication is not implied by a core release or by a successful dry-run. It
requires all of the following immediately before a registry request:

1. The immutable final `v0.22.0` GitHub Release exists and its annotated tag
   passes cryptographic `git verify-tag` validation.
2. `tag`, `github.sha`, the supplied full SHA, and checked-out HEAD agree.
3. Exactly one channel (`python` or `npm`) is selected with `dry_run=false`
   and `publish=true`.
4. A release owner supplies fresh channel-specific approval through the
   protected `release-pypi` or `release-npm` GitHub environment.
5. The registry has been verified read-only for package-name ownership and
   version availability by the release owner.

PyPI uses GitHub OIDC trusted publishing. npm uses npm trusted publishing with
OIDC provenance. No long-lived registry token is accepted by the workflow.

After publication, verify from a clean consumer:

```bash
bash scripts/verify-pypi-release.sh --package engram-client --version 0.5.0
bash scripts/verify-npm-release.sh --package engram-client --version 0.5.0
```

PyPI files and npm versions cannot be safely replaced. On an incident, yank or
deprecate the affected version as appropriate and publish a forward fix; never
silently rebuild an existing version.

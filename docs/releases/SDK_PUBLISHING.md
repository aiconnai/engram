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
SHA="$(git rev-parse HEAD)"; REF="$(git branch --show-current)"   # the pushed branch whose head is $SHA
gh workflow run sdk-release.yml --ref "$REF" \
  -f channel=all -f dry_run=true -f publish=false -f sha="$SHA"
sleep 10   # allow the run to register, then pick the newest dispatch on that ref
RUN_ID="$(gh run list --workflow sdk-release.yml --branch "$REF" --event workflow_dispatch --limit 1 --json databaseId --jq '.[0].databaseId')"
gh run watch "$RUN_ID" --exit-status
bash scripts/verify-sdk-artifacts.sh --run-id "$RUN_ID" --expected-sha "$SHA" --live
```

The verifier checks package names, independently versioned package metadata,
core compatibility ranges, SHA binding, checksums, compiled npm entry points,
and clean installation of both packages. `--live` additionally runs the real
server SDK contracts.

## Version mapping

Nothing in the workflow hardcodes a release tag or an SDK version. The
preflight job (and again each publish job, after environment approval) runs the
read-only mapping check at the exact release SHA:

```bash
python3 scripts/bump-version.py --release-plan --release-tag vX.Y.Z
# prints: core_version, core_tag, python_version, npm_version
```

The mapping is:

- **Core tag** is always `v<Cargo.toml version>` (stable `vX.Y.Z` only). The
  `tag` dispatch input must equal it, and the reviewed
  [`channel-matrix.toml`](channel-matrix.toml) must declare the same tag as
  `core_tag`, `git_tag.target`, and `github_release.target`.
- **SDK versions** are independent of the core tag. The Python version comes
  from `sdks/python/pyproject.toml` and the npm version from
  `sdks/typescript/package.json`; they must match the matrix `pypi`/`npm`
  `local_version`, the SDK `__version__`, and `package-lock.json`.
- **Core compatibility**: the core version must fall inside each SDK's
  `compatible_core_min .. compatible_core_max` range in the matrix.
- `Cargo.lock` workspace entries must match their manifests, and `CHANGELOG.md`
  must contain the core version.

Any disagreement fails the run before a registry is contacted. The registry
"already published" guards and the post-publish installability checks use the
derived SDK version (for example `0.7.0`), never the core tag.

## Protected publication

Publication is not implied by a core release or by a successful dry-run. It
requires all of the following immediately before a registry request:

1. The immutable final GitHub Release for the exact core tag
   (`v<Cargo.toml version>`) exists and its annotated tag passes cryptographic
   signature validation via `origin/main:scripts/verify-release-tag.sh`
   (materialized from `origin/main` into `${RUNNER_TEMP}` and using an explicit
   OpenSSH `allowed_signers` trust anchor from protected repository/environment
   variable `ENGRAM_RELEASE_ALLOWED_SIGNERS` — optionally pinned by
   `ENGRAM_RELEASE_ALLOWED_SIGNERS_SHA256` — or from
   `origin/main:docs/releases/allowed-signers`, never trusting a verifier script
   or signers file from the tag tree under verification; fails closed when no
   trusted signer is configured).
2. `tag`, `github.sha`, the supplied full SHA, and checked-out HEAD agree, and
   the version mapping above passes.
3. Exactly one channel (`python` or `npm`) is selected with `dry_run=false`
   and `publish=true`.
4. A release owner supplies fresh channel-specific approval through the
   protected `release-pypi` or `release-npm` GitHub environment.
5. The registry has been verified read-only for package-name ownership and
   version availability by the release owner.

PyPI uses GitHub OIDC trusted publishing. npm uses npm trusted publishing with
OIDC provenance. No long-lived registry token is accepted by the workflow.

After publication, verify from a clean consumer, using the SDK versions the
mapping check derives (not the core tag):

```bash
plan="$(python3 scripts/bump-version.py --release-plan)"
bash scripts/verify-pypi-release.sh --package engram-client --version "$(sed -n 's/^python_version=//p' <<<"$plan")"
bash scripts/verify-npm-release.sh --package engram-client --version "$(sed -n 's/^npm_version=//p' <<<"$plan")"
```

PyPI files and npm versions cannot be safely replaced. On an incident, yank or
deprecate the affected version as appropriate and publish a forward fix; never
silently rebuild an existing version.

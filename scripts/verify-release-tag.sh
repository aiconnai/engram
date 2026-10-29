#!/usr/bin/env bash
# Verify that a Git release tag is an annotated, cryptographically SSH-signed tag
# pointing to the expected commit SHA, using an explicit pinned trust store.
#
# Fail-closed properties:
# 1. Ignores host/system/user Git and GPG configuration (GIT_CONFIG_NOSYSTEM=1,
#    GIT_CONFIG_GLOBAL=/dev/null, isolated ephemeral HOME).
# 2. Anti-self-authorization: when --trusted-ref <ref> is supplied (for example
#    origin/main) and ENGRAM_RELEASE_ALLOWED_SIGNERS / --allowed-signers is not
#    set, reads <ref>:docs/releases/allowed-signers from Git object storage
#    instead of the checked-out tag worktree.
# 3. Fails closed when zero active signer entries are configured.

set -euo pipefail

script_repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
repo_dir="$(git rev-parse --show-toplevel 2>/dev/null || printf '%s' "${script_repo_root}")"
tag=""
expected_sha=""
allowed_signers_arg=""
trusted_ref=""

usage() {
  cat <<'EOF' >&2
Usage: scripts/verify-release-tag.sh --tag <tag> [options]

Options:
  --tag <tag>                 Exact release tag name (for example v0.24.0)
  --expected-sha <40-hex>     Verify tag peels to this exact 40-char commit SHA
  --allowed-signers <path>    Explicit OpenSSH allowed_signers file path
  --trusted-ref <git-ref>     Read docs/releases/allowed-signers from <git-ref>
                              (for example origin/main) instead of the worktree
  --repo <dir>                Repository directory (defaults to repo root)
EOF
}

fail() {
  echo "::error::$*" >&2
  exit 1
}

sha256_file() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${path}" | awk '{print tolower($1)}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${path}" | awk '{print tolower($1)}'
  else
    python3 -c 'import hashlib, pathlib, sys; print(hashlib.sha256(pathlib.Path(sys.argv[1]).read_bytes()).hexdigest())' "${path}"
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      tag="${2:-}"
      shift 2
      ;;
    --expected-sha)
      expected_sha="${2:-}"
      shift 2
      ;;
    --allowed-signers)
      allowed_signers_arg="${2:-}"
      shift 2
      ;;
    --trusted-ref)
      trusted_ref="${2:-}"
      shift 2
      ;;
    --repo)
      repo_dir="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage
      fail "unknown argument to verify-release-tag.sh: $1"
      ;;
  esac
done

[[ -n "${tag}" ]] || fail "verify-release-tag.sh requires --tag"
[[ -d "${repo_dir}" ]] || fail "repository directory does not exist: ${repo_dir}"

tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/engram-verify-tag.XXXXXX")"
chmod 700 "${tmp_dir}"
cleanup() {
  rm -rf "${tmp_dir}"
}
trap cleanup EXIT

isolated_home="${tmp_dir}/home"
mkdir -p "${isolated_home}"
chmod 700 "${isolated_home}"

resolved_signers="${tmp_dir}/allowed_signers"

if [[ -n "${ENGRAM_RELEASE_ALLOWED_SIGNERS:-}" ]]; then
  printf '%s\n' "${ENGRAM_RELEASE_ALLOWED_SIGNERS}" > "${resolved_signers}"
elif [[ -n "${allowed_signers_arg}" ]]; then
  [[ -f "${allowed_signers_arg}" ]] || fail "publish requires a cryptographically signed tag (allowed-signers file not found: ${allowed_signers_arg})"
  cp "${allowed_signers_arg}" "${resolved_signers}"
elif [[ -n "${ENGRAM_RELEASE_ALLOWED_SIGNERS_FILE:-}" ]]; then
  [[ -f "${ENGRAM_RELEASE_ALLOWED_SIGNERS_FILE}" ]] || fail "publish requires a cryptographically signed tag (ENGRAM_RELEASE_ALLOWED_SIGNERS_FILE not found)"
  cp "${ENGRAM_RELEASE_ALLOWED_SIGNERS_FILE}" "${resolved_signers}"
elif [[ -n "${trusted_ref}" ]]; then
  if ! git -C "${repo_dir}" cat-file -e "${trusted_ref}:docs/releases/allowed-signers" 2>/dev/null; then
    fail "publish requires a cryptographically signed tag (cannot read ${trusted_ref}:docs/releases/allowed-signers)"
  fi
  git -C "${repo_dir}" show "${trusted_ref}:docs/releases/allowed-signers" > "${resolved_signers}"
else
  default_signers="${repo_dir}/docs/releases/allowed-signers"
  [[ -f "${default_signers}" ]] || fail "publish requires a cryptographically signed tag (missing docs/releases/allowed-signers)"
  cp "${default_signers}" "${resolved_signers}"
fi
chmod 600 "${resolved_signers}"

if [[ -n "${ENGRAM_RELEASE_ALLOWED_SIGNERS_SHA256:-}" ]]; then
  expected_signers_sha="$(printf '%s' "${ENGRAM_RELEASE_ALLOWED_SIGNERS_SHA256}" | tr '[:upper:]' '[:lower:]')"
  [[ "${expected_signers_sha}" =~ ^[0-9a-f]{64}$ ]] || fail "ENGRAM_RELEASE_ALLOWED_SIGNERS_SHA256 must be a 64-character hex SHA-256 digest"
  actual_signers_sha="$(sha256_file "${resolved_signers}")"
  [[ "${actual_signers_sha}" == "${expected_signers_sha}" ]] || fail "publish requires a cryptographically signed tag (allowed-signers SHA-256 mismatch)"
fi

active_signers_file="${tmp_dir}/active_signers"
awk '
  {
    line = $0
    sub(/^[[:space:]]+/, "", line)
    sub(/[[:space:]]+$/, "", line)
    if (line == "" || substr(line, 1, 1) == "#") {
      next
    }
    print line
  }
' "${resolved_signers}" > "${active_signers_file}"
chmod 600 "${active_signers_file}"

if [[ ! -s "${active_signers_file}" ]]; then
  fail "publish requires a cryptographically signed tag (no trusted release signers configured)"
fi

git -C "${repo_dir}" show-ref --verify --quiet "refs/tags/${tag}" || fail "publish requires the exact tag ref"

[[ "$(git -C "${repo_dir}" cat-file -t "refs/tags/${tag}")" == "tag" ]] || fail "publish requires an annotated tag object"

if [[ -n "${expected_sha}" ]]; then
  [[ "${expected_sha}" =~ ^[0-9a-f]{40}$ ]] || fail "expected-sha must be a full lowercase commit SHA"
  peeled_sha="$(git -C "${repo_dir}" rev-list -n 1 "refs/tags/${tag}^{commit}")"
  [[ "${peeled_sha}" == "${expected_sha}" ]] || fail "tag, github.sha, and supplied SHA do not agree"
fi

if ! GIT_CONFIG_NOSYSTEM=1 \
     GIT_CONFIG_GLOBAL=/dev/null \
     HOME="${isolated_home}" \
     git -C "${repo_dir}" \
       -c gpg.format=ssh \
       -c gpg.ssh.program=ssh-keygen \
       -c "gpg.ssh.allowedSignersFile=${active_signers_file}" \
       -c gpg.ssh.revocationFile=/dev/null \
       verify-tag "refs/tags/${tag}" >/dev/null 2>&1; then
  fail "publish requires a cryptographically signed tag"
fi

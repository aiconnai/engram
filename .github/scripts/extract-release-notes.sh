#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'USAGE'
Usage: extract-release-notes.sh <version> [changelog] [output]

Extracts a single Keep a Changelog release section into a GitHub Release notes
file. <version> may be passed as either 0.21.1 or v0.21.1.
USAGE
}

if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ] || [ "$#" -lt 1 ]; then
  usage
  exit 2
fi

version="${1#v}"
tag="v${version}"
changelog="${2:-CHANGELOG.md}"
output="${3:-release/RELEASE_NOTES.md}"

if [ ! -f "${changelog}" ]; then
  echo "Changelog not found: ${changelog}" >&2
  exit 1
fi

mkdir -p "$(dirname "${output}")"
tmp="$(mktemp)"
trap 'rm -f "${tmp}"' EXIT

awk -v version="${version}" -v tag="${tag}" '
  BEGIN {
    in_section = 0
    found = 0
  }

  $0 ~ "^## \\[" version "\\]" {
    found = 1
    in_section = 1
    print "## Engram " tag
    if (match($0, / - [0-9]{4}-[0-9]{2}-[0-9]{2}$/)) {
      print ""
      print "_Release date:" substr($0, RSTART + 2) "_"
    }
    next
  }

  in_section && ($0 ~ "^---$" || $0 ~ "^## \\[") {
    in_section = 0
    next
  }

  in_section {
    print
  }

  END {
    if (!found) {
      print "Release section not found for " version > "/dev/stderr"
      exit 1
    }
  }
' "${changelog}" > "${tmp}"

compare_url="$(
  awk -v version="${version}" '$0 ~ "^\\[" version "\\]: " { print $2; exit }' "${changelog}"
)"

if [ -n "${compare_url}" ]; then
  {
    echo ""
    echo "---"
    echo ""
    echo "Full changelog: ${compare_url}"
  } >> "${tmp}"
fi

mv "${tmp}" "${output}"
trap - EXIT

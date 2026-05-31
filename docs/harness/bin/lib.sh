#!/usr/bin/env bash

field_value() {
  local file="$1"
  local key="$2"
  awk -F'|' -v key="$key" '
    $2 ~ "^[[:space:]]*" key "[[:space:]]*$" {
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", $3)
      gsub(/^`|`$/, "", $3)
      print $3
      exit
    }
  ' "$file" 2>/dev/null || true
}


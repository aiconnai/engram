#!/usr/bin/env bash
# docs/harness/bin/harness-decision-log.sh
#
# Structured decision registry helper for the Engram harness.
# Keeps deterministic IDs (DEC-0001, DEC-0002, ...) and a YAML append-only record.

set -euo pipefail

DEFAULT_DECISION_FILE="docs/harness/decisions/harness-decision-log.yaml"
DECISION_FILE="${HARNESS_DECISION_FILE:-$DEFAULT_DECISION_FILE}"

usage() {
  cat <<'USAGE'
Usage:
  harness-decision-log.sh new --title "..." --context "..." --decision "..." --rationale "..." [--option "name|pro1;pro2|con1;con2"] ...
  harness-decision-log.sh list [--file path]
  harness-decision-log.sh validate [--file path]
  harness-decision-log.sh help

Commands:
  new       Add a structured decision entry (ID auto-generated as DEC-XXXX).
  list      Show recorded decisions.
  validate  Validate file shape (best-effort sanity checks).

Environment:
  HARNESS_DECISION_FILE path    Override the target registry file.

Option format:
  --option "option name|pros (;) separated|cons (;) separated"
  Example:
    --option "ONNX local|sem dependência externa;rápido para dev|precisão menor"
USAGE
}

ensure_file() {
  local file="$1"

  if [ -f "$file" ]; then
    return
  fi

  mkdir -p "$(dirname "$file")"
  {
    printf 'schema_version: harness-decision-log-v1\n'
    printf 'updated_at: %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    printf 'entries:\n'
  } > "$file"
}

update_file_timestamp() {
  local file="$1"
  local now
  now="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  if [ -f "$file" ]; then
    sed -i.bak "s/^updated_at: .*/updated_at: ${now}/" "$file"
    rm -f "${file}.bak"
  fi
}

next_decision_id() {
  local file="$1"
  local max_id=0
  local current_id

  if [ -f "$file" ]; then
    while IFS= read -r line; do
      if [[ "$line" =~ ^[[:space:]]*-[[:space:]]id:[[:space:]]DEC-([0-9]{4})$ ]]; then
        current_id="${BASH_REMATCH[1]}"
        if [ "$current_id" -gt "$max_id" ]; then
          max_id="$current_id"
        fi
      fi
    done < "$file"
  fi

  printf 'DEC-%04d' "$((max_id + 1))"
}

escape_yaml_scalar() {
  local value="$1"
  value="${value//$'\r'/}"
  value="${value//$'\n'/\\n}"
  value="${value//\\/\\\\}"
  value="${value//\"/\\\"}"
  printf '%s' "$value"
}

validate_yaml() {
  local file="$1"

  if ! command -v ruby >/dev/null 2>&1; then
    echo "WARN: ruby not installed; skipping YAML parse validation for $file"
    return 0
  fi

  ruby -e 'require "yaml";
doc = YAML.load_file(ARGV[0]);
raise "invalid root type" unless doc.is_a?(Hash)
raise "schema_version missing" unless doc["schema_version"] == "harness-decision-log-v1"
raise "entries missing" unless doc.key?("entries")
raise "entries must be array" unless doc["entries"].is_a?(Array)
doc["entries"].each do |entry|
  raise "entry must be mapping" unless entry.is_a?(Hash)
  %w[id date title context decision rationale].each do |field|
    raise "missing #{field}" unless entry.key?(field)
    raise "#{field} empty" if entry[field].to_s.empty?
  end
  raise "invalid id format" unless entry["id"].to_s =~ /^DEC-[0-9]{4}$/
end
' "$file"
}

append_yaml_list_items() {
  local values_raw="$1"
  local indent="$2"

  if [ -z "$values_raw" ]; then
    printf '%s[]\n' "$indent"
    return
  fi

  local value
  IFS=';' read -ra values <<< "$values_raw"
  for value in "${values[@]}"; do
    if [ -n "$value" ]; then
      printf '%s- "%s"\n' "$indent" "$(escape_yaml_scalar "$(printf '%s' "$value" | sed 's/^ *//;s/ *$//')")"
    fi
  done
}

append_option_block() {
  local option_payload="$1"
  local value_name
  local pros_raw
  local cons_raw

  value_name="${option_payload%%|*}"
  pros_raw="${option_payload#*|}"
  pros_raw="${pros_raw%%|*}"
  cons_raw="${option_payload##*|}"

  printf '    - option: "%s"\n' "$(escape_yaml_scalar "$value_name")"
  printf '      pros:\n'
  if [ -n "$pros_raw" ] && [ "$pros_raw" != "$cons_raw" ]; then
    append_yaml_list_items "$pros_raw" "      "
  else
    printf '      []\n'
  fi
  printf '      cons:\n'
  if [ -n "$cons_raw" ] && [ "$cons_raw" != "$value_name" ]; then
    append_yaml_list_items "$cons_raw" "      "
  else
    printf '      []\n'
  fi
}

run_new() {
  local title=""
  local context=""
  local decision=""
  local rationale=""
  local id=""
  local date
  date="$(date -u +%Y-%m-%d)"
  local option_count=0

  local -a options
  shift
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --file)
        DECISION_FILE="$2"
        shift 2
        ;;
      --title)
        title="$2"
        shift 2
        ;;
      --context)
        context="$2"
        shift 2
        ;;
      --decision)
        decision="$2"
        shift 2
        ;;
      --rationale)
        rationale="$2"
        shift 2
        ;;
      --option)
        options+=("$2")
        option_count=$((option_count + 1))
        shift 2
        ;;
      --id)
        id="$2"
        shift 2
        ;;
      --date)
        date="$2"
        shift 2
        ;;
      --)
        shift
        break
        ;;
      *)
        echo "ERROR: unknown argument '$1'" >&2
        usage
        exit 2
        ;;
    esac
  done

  if [ -z "$title" ] || [ -z "$context" ] || [ -z "$decision" ] || [ -z "$rationale" ]; then
    echo "ERROR: missing required fields (--title, --context, --decision, --rationale)" >&2
    usage
    exit 2
  fi

  ensure_file "$DECISION_FILE"

  if [ -z "$id" ]; then
    id="$(next_decision_id "$DECISION_FILE")"
  fi

  {
    printf '\n  - id: %s\n' "$id"
    printf '    date: %s\n' "$date"
    printf '    title: "%s"\n' "$(escape_yaml_scalar "$title")"
    printf '    context: "%s"\n' "$(escape_yaml_scalar "$context")"
    if [ "$option_count" -eq 0 ]; then
      printf '    options: []\n'
    else
      printf '    options:\n'
      for option in "${options[@]}"; do
        append_option_block "$option"
      done
    fi
  printf '    decision: "%s"\n' "$(escape_yaml_scalar "$decision")"
  printf '    rationale: "%s"\n' "$(escape_yaml_scalar "$rationale")"
  } >> "$DECISION_FILE"

  update_file_timestamp "$DECISION_FILE"
  echo "Added decision $id to $DECISION_FILE"
}

run_list() {
  shift
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --file)
        DECISION_FILE="$2"
        shift 2
        ;;
      *)
        echo "ERROR: unknown argument '$1'" >&2
        usage
        exit 2
        ;;
    esac
  done

  if [ ! -f "$DECISION_FILE" ]; then
    echo "(no decision file yet: $DECISION_FILE)"
    exit 0
  fi

  awk '
    $0 ~ /^  - id:/ {
      if (id != "") {
        if (title == "") {
          title = "?"
        }
        printf "%s  %s  %s\n", id, date, title
      }
      id = $3
      date = ""
      title = ""
      next
    }
    $0 ~ /^    title:/ {
      title = $0
      sub(/^[[:space:]]*title: /, "", title)
      gsub(/^"/, "", title)
      gsub(/"$/, "", title)
      next
    }
    $0 ~ /^    date:/ {
      date = $0
      sub(/^[[:space:]]*date: /, "", date)
      next
    }
    END {
      if (id != "") {
        if (title == "") {
          title = "?"
        }
        printf "%s  %s  %s\n", id, date, title
      }
    }
  ' "$DECISION_FILE" | sed -n '1,200p'
}

run_validate() {
  shift
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --file)
        DECISION_FILE="$2"
        shift 2
        ;;
      *)
        echo "ERROR: unknown argument '$1'" >&2
        usage
        exit 2
        ;;
    esac
  done

  if [ ! -f "$DECISION_FILE" ]; then
    echo "FAIL: decision file missing: $DECISION_FILE" >&2
    exit 1
  fi

  if ! grep -q '^schema_version:' "$DECISION_FILE"; then
    echo "FAIL: schema_version missing in $DECISION_FILE" >&2
    exit 1
  fi
  if ! grep -q '^entries:' "$DECISION_FILE"; then
    echo "FAIL: entries block missing in $DECISION_FILE" >&2
    exit 1
  fi

  if ! validate_yaml "$DECISION_FILE" >/tmp/harness-decision-validate.out 2>&1; then
    echo "FAIL: invalid YAML content in $DECISION_FILE" >&2
    cat /tmp/harness-decision-validate.out >&2
    rm -f /tmp/harness-decision-validate.out
    exit 1
  fi
  rm -f /tmp/harness-decision-validate.out

  if ! grep -q '^  - id:' "$DECISION_FILE"; then
    echo "WARN: no entries in decision file" >&2
  fi

  local dup
  dup="$(grep '^  - id:' "$DECISION_FILE" | awk '{print $3}' | tr -d '"' | sort | uniq -d)"
  if [ -n "$dup" ]; then
    echo "FAIL: duplicated decision ids detected:" >&2
    echo "$dup" >&2
    exit 1
  fi

  echo "OK: $DECISION_FILE appears structurally valid"
}

COMMAND="${1:-}"
if [ -z "$COMMAND" ]; then
  usage
  exit 2
fi

case "$COMMAND" in
  help|-h|--help)
    usage
    ;;
  new)
    run_new "$@"
    ;;
  list)
    run_list "$@"
    ;;
  validate)
    run_validate "$@"
    ;;
  *)
    echo "ERROR: unknown command: $COMMAND" >&2
    usage
    exit 2
    ;;
esac

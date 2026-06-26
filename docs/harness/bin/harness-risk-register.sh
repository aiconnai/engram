#!/usr/bin/env bash
# docs/harness/bin/harness-risk-register.sh
#
# Structured risk register helper for the Engram harness.

set -euo pipefail

DEFAULT_RISK_REGISTER_FILE="docs/harness/risk-register.yaml"
RISK_FILE="${HARNESS_RISK_FILE:-$DEFAULT_RISK_REGISTER_FILE}"

usage() {
  cat <<'USAGE'
Usage:
  harness-risk-register.sh add --description "..." --probability 1-5 --impact 1-5 --mitigation "..." [--contingency "..."] [--owner "name"] [--status open|closed|accepted|deferred|monitoring] [--monitor "event 1;event 2"]
  harness-risk-register.sh list [--file path]
  harness-risk-register.sh validate [--file path]

Commands:
  add       Add a new risk with computed score = probability × impact.
  list      Show risks ordered by id.
  validate  Validate file shape (best-effort sanity checks).

Environment:
  HARNESS_RISK_FILE path    Override the target register file.
USAGE
}

ensure_file() {
  local file="$1"

  if [ -f "$file" ]; then
    return
  fi

  mkdir -p "$(dirname "$file")"
  {
    printf 'schema_version: harness-risk-register-v1\n'
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

next_risk_id() {
  local file="$1"
  local max_id=0
  local current_id

  if [ -f "$file" ]; then
    while IFS= read -r line; do
      if [[ "$line" =~ ^[[:space:]]*-[[:space:]]id:[[:space:]]RISK-([0-9]{4})$ ]]; then
        current_id="${BASH_REMATCH[1]}"
        if [ "$current_id" -gt "$max_id" ]; then
          max_id="$current_id"
        fi
      fi
    done < "$file"
  fi

  printf 'RISK-%04d' "$((max_id + 1))"
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
raise "schema_version missing" unless doc["schema_version"] == "harness-risk-register-v1"
raise "entries missing" unless doc.key?("entries")
raise "entries must be array" unless doc["entries"].is_a?(Array)
doc["entries"].each do |entry|
  raise "entry must be mapping" unless entry.is_a?(Hash)
  %w[id date status description probability impact score owner mitigation].each do |field|
    raise "missing #{field}" unless entry.key?(field)
    raise "#{field} empty" if entry[field].to_s.empty?
  end
  raise "invalid id format" unless entry["id"].to_s =~ /^RISK-[0-9]{4}$/
  raise "invalid status" unless ["open", "closed", "accepted", "deferred", "monitoring"].include?(entry["status"].to_s)
  raise "invalid probability" unless entry["probability"].to_i.between?(1, 5)
  raise "invalid impact" unless entry["impact"].to_i.between?(1, 5)
  raise "invalid score" unless entry["score"].to_i == entry["probability"].to_i * entry["impact"].to_i
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

run_add() {
  local id=""
  local description=""
  local probability=""
  local impact=""
  local mitigation=""
  local contingency=""
  local owner="unknown"
  local status="open"
  local monitoring=""
  local date=""

  shift
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --file)
        RISK_FILE="$2"
        shift 2
        ;;
      --id)
        id="$2"
        shift 2
        ;;
      --description)
        description="$2"
        shift 2
        ;;
      --probability)
        probability="$2"
        shift 2
        ;;
      --impact)
        impact="$2"
        shift 2
        ;;
      --mitigation)
        mitigation="$2"
        shift 2
        ;;
      --contingency)
        contingency="$2"
        shift 2
        ;;
      --owner)
        owner="$2"
        shift 2
        ;;
      --status)
        status="$2"
        shift 2
        ;;
      --monitor)
        monitoring="$2"
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

  if [ -z "$description" ] || [ -z "$probability" ] || [ -z "$impact" ] || [ -z "$mitigation" ]; then
    echo "ERROR: missing required fields (--description, --probability, --impact, --mitigation)" >&2
    usage
    exit 2
  fi

  if ! [[ "$probability" =~ ^[1-5]$ ]] || ! [[ "$impact" =~ ^[1-5]$ ]]; then
    echo "ERROR: probability and impact must be integers 1-5" >&2
    exit 2
  fi

  ensure_file "$RISK_FILE"
  if [ -z "$id" ]; then
    id="$(next_risk_id "$RISK_FILE")"
  fi
  if [ -z "$date" ]; then
    date="$(date -u +%Y-%m-%d)"
  fi

  local score=$((probability * impact))
  if ! [[ "$status" =~ ^(open|closed|accepted|deferred|monitoring)$ ]]; then
    echo "ERROR: invalid status: $status" >&2
    exit 2
  fi

  {
    printf '\n  - id: %s\n' "$id"
    printf '    date: %s\n' "$date"
    printf '    status: %s\n' "$status"
    printf '    description: "%s"\n' "$(escape_yaml_scalar "$description")"
    printf '    probability: %s\n' "$probability"
    printf '    impact: %s\n' "$impact"
    printf '    score: %s\n' "$score"
    printf '    owner: "%s"\n' "$(escape_yaml_scalar "$owner")"
    printf '    mitigation: "%s"\n' "$(escape_yaml_scalar "$mitigation")"
    if [ -n "$contingency" ]; then
      printf '    contingency: "%s"\n' "$(escape_yaml_scalar "$contingency")"
    else
      printf '    contingency: ""\n'
    fi
    if [ -n "$monitoring" ]; then
      printf '    monitoring:\n'
      append_yaml_list_items "$monitoring" "      "
    else
      printf '    monitoring: []\n'
    fi
  } >> "$RISK_FILE"

  update_file_timestamp "$RISK_FILE"
  echo "Added risk $id to $RISK_FILE (score=$score)"
}

run_list() {
  shift
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --file)
        RISK_FILE="$2"
        shift 2
        ;;
      *)
        echo "ERROR: unknown argument '$1'" >&2
        usage
        exit 2
        ;;
    esac
  done

  if [ ! -f "$RISK_FILE" ]; then
    echo "(no risk file yet: $RISK_FILE)"
    exit 0
  fi

  awk '
    $0 ~ /^  - id:/ {
      if (id != "") {
        if (description == "") {
          description = "?"
        }
        printf "%s  %s  %s  %s  %s\n", id, score, status, owner, description
      }
      id = $3
      score = ""
      status = ""
      owner = ""
      description = ""
      next
    }
    $0 ~ /^    description:/ {
      description = $0
      sub(/^[[:space:]]*description: /, "", description)
      gsub(/^"/, "", description)
      gsub(/"$/, "", description)
      next
    }
    $0 ~ /^    score:/ {score = $2; next}
    $0 ~ /^    status:/ {status = $2; next}
    $0 ~ /^    owner:/ {owner = $2; next}
    END {
      if (id != "") {
        if (description == "") {
          description = "?"
        }
        printf "%s  %s  %s  %s  %s\n", id, score, status, owner, description
      }
    }
  ' "$RISK_FILE" | sed -n '1,200p'
}

run_validate() {
  shift
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --file)
        RISK_FILE="$2"
        shift 2
        ;;
      *)
        echo "ERROR: unknown argument '$1'" >&2
        usage
        exit 2
        ;;
    esac
  done

  if [ ! -f "$RISK_FILE" ]; then
    echo "FAIL: risk file missing: $RISK_FILE" >&2
    exit 1
  fi

  if ! grep -q '^schema_version:' "$RISK_FILE"; then
    echo "FAIL: schema_version missing in $RISK_FILE" >&2
    exit 1
  fi
  if ! grep -q '^entries:' "$RISK_FILE"; then
    echo "FAIL: entries block missing in $RISK_FILE" >&2
    exit 1
  fi

  if ! validate_yaml "$RISK_FILE" >/tmp/harness-risk-validate.out 2>&1; then
    echo "FAIL: invalid YAML content in $RISK_FILE" >&2
    cat /tmp/harness-risk-validate.out >&2
    rm -f /tmp/harness-risk-validate.out
    exit 1
  fi
  rm -f /tmp/harness-risk-validate.out

  local dup
  dup="$(grep '^  - id:' "$RISK_FILE" | awk '{print $3}' | tr -d '"' | sort | uniq -d)"
  if [ -n "$dup" ]; then
    echo "FAIL: duplicated risk ids detected:" >&2
    echo "$dup" >&2
    exit 1
  fi

  if grep '^  - id:' "$RISK_FILE" | grep -v '^  - id: RISK-[0-9]\{4\}$' >/dev/null 2>&1; then
    echo "WARN: found malformed risk id(s); expected RISK-XXXX" >&2
  fi

  echo "OK: $RISK_FILE appears structurally valid"
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
  add)
    run_add "$@"
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

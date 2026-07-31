#!/bin/zsh
set -u
set -o pipefail

# Self-contained replay of a retained Apple artifact-compatibility record.
#
# The record binds its producer set by digest, so replaying it means running the
# producer revision it names and not whatever the current tree happens to hold.
# Each retained result therefore keeps those exact bytes under `producers/`,
# laid out by the repository-relative paths its own `input-manifest.tsv` names.
#
# This script verifies every retained producer byte against both the manifest
# and the record's own producer fields *before* it executes any of them, so an
# edited retained validator cannot certify its own changed identity, and then
# validates the record with the retained validator rather than the current-tree
# one. It reads nothing outside the named result directory and needs no Python
# project environment: the retained validators are standard-library-only.

result_root=${1:-}
python_bin=${TILER_REPLAY_PYTHON:-python3}

fail() {
  print -u2 -r -- "compatibility replay failed: $1"
  exit 4
}

[[ -n "$result_root" ]] \
  || fail "usage: replay_retained_compatibility_record.sh <retained-result-directory>"
result_root="${result_root:A}"
[[ -d "$result_root" ]] || fail "not a directory: $result_root"

record="$result_root/record.tsv"
manifest="$result_root/input-manifest.tsv"
producers="$result_root/producers"
[[ -f "$record" ]] || fail "no record.tsv under $result_root"
[[ -f "$manifest" ]] || fail "no input-manifest.tsv under $result_root"
[[ -d "$producers" ]] || fail "no retained producer set under $result_root"

for command_name in shasum find awk "$python_bin"; do
  command -v "$command_name" >/dev/null 2>&1 || fail "required command unavailable: $command_name"
done

# These three answer through globals rather than standard output, because `fail`
# inside a command substitution exits only that subshell: the caller would carry
# on with an empty value and report the *next* mismatch instead of the real
# reason. A check that says no for the wrong reason is barely better than one
# that cannot say no at all.
typeset -g row_value=""
typeset -g file_sha256=""

# One row, or none: a duplicated key is as unusable as a missing one, and the
# retained validator's own duplicate check runs only after this point.
row() {
  local key="$1"
  row_value=$(awk -F'\t' -v key="$key" '
    $1 == key { count++; value = $2 }
    END { if (count != 1) exit 1; print value }
  ' "$record") || fail "record does not carry exactly one $key row"
  [[ -n "$row_value" ]] || fail "record field is empty: $key"
}

digest_row() {
  row "$1"
  [[ "$row_value" =~ '^[0-9a-f]{64}$' ]] || fail "malformed SHA-256 in $1: $row_value"
}

file_digest() {
  local output
  output=$(shasum -a 256 "$1" 2>&1) || fail "could not hash $1: $output"
  file_sha256=${output%% *}
  [[ "$file_sha256" =~ '^[0-9a-f]{64}$' ]] \
    || fail "malformed SHA-256 for $1: $file_sha256"
}

# The producer population this replay accepts, stated here rather than derived
# from the manifest, so a manifest that lost or gained a row fails instead of
# redefining what the replay checks. Schema v2 records bind five inputs; the
# current tree's post-Python-tooling harness binds three and its validator
# expects three, which is why this replay selects the retained validator.
typeset -A producer_field
producer_field=(
  'spikes/apple-targets/compatibility_probe.sh'           probe.harness_sha256
  'spikes/apple-targets/copy.metal'                       probe.source_sha256
  'spikes/apple-targets/validate_compatibility_record.py' probe.validator_sha256
  'pyproject.toml'                                        probe.project_sha256
  'uv.lock'                                               probe.lock_sha256
)
retained_validator="$producers/spikes/apple-targets/validate_compatibility_record.py"

row schema
[[ "$row_value" == "tiler.apple-target-compatibility/v2" ]] \
  || fail "record does not carry the schema this replay validates: $row_value"
row probe.input_manifest_file
[[ "$row_value" == "input-manifest.tsv" ]] \
  || fail "record names an input manifest this replay does not retain: $row_value"

digest_row probe.input_manifest_sha256
recorded_manifest="$row_value"
file_digest "$manifest"
[[ "$file_sha256" == "$recorded_manifest" ]] \
  || fail "retained input manifest digest mismatch: recorded $recorded_manifest, found $file_sha256"
print -r -- "verified input-manifest.tsv $file_sha256"

# The manifest is read row by row rather than trusted as a set, so a row naming
# an unexpected producer, a repeated row, and a short population are three
# separate refusals instead of one silent pass.
typeset -A seen
verified=0
while IFS=$'\t' read -r relative recorded; do
  [[ -n "$relative" && -n "$recorded" ]] \
    || fail "input manifest row is not a nonempty path/digest pair"
  [[ -n "${producer_field[$relative]:-}" ]] \
    || fail "input manifest names a producer this replay does not expect: $relative"
  [[ -z "${seen[$relative]:-}" ]] || fail "input manifest names $relative twice"
  seen[$relative]=1
  retained="$producers/$relative"
  [[ -f "$retained" ]] || fail "retained producer is missing: producers/$relative"
  file_digest "$retained"
  actual="$file_sha256"
  [[ "$actual" == "$recorded" ]] \
    || fail "retained producer does not match the manifest: producers/$relative, manifest $recorded, found $actual"
  digest_row "${producer_field[$relative]}"
  [[ "$actual" == "$row_value" ]] \
    || fail "retained producer does not match ${producer_field[$relative]}: producers/$relative, record $row_value, found $actual"
  verified=$(( verified + 1 ))
  print -r -- "verified producers/$relative $actual"
done <"$manifest"

(( verified == ${#producer_field} )) \
  || fail "input manifest named $verified producers, expected ${#producer_field}"

# An unlisted file under `producers/` would be retained bytes no digest covers,
# so count the population rather than only checking the rows that exist.
retained_count=$(find "$producers" -type f | wc -l | tr -d ' ')
(( retained_count == verified )) \
  || fail "producers/ holds $retained_count files but the manifest names $verified"

[[ -f "$retained_validator" ]] || fail "retained validator is missing after verification"
"$python_bin" "$retained_validator" "$record" \
  || fail "the retained validator rejected the record"

print -r -- "replayed_record=$record"
print -r -- "replayed_with=producers/spikes/apple-targets/validate_compatibility_record.py"
print -r -- "verified_producers=$verified"

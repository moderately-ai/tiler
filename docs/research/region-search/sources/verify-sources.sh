#!/usr/bin/env bash
#
# Verifies the preserved source population against expected-sources.tsv. That population covers the
# optimizer literature the rewrite-search formalism survey cites as primary evidence: the database
# optimizer lineage, the equality-saturation lineage and its tensor-graph applications, the
# tensor-compiler schedule-search lineage, and the phase-ordering literature.
#
# The check counts its population before it inspects it. An empty or truncated manifest, a deleted
# vendored file, an unreferenced stray file, or a mutated digest each fail with a named reason, so
# silence can never be mistaken for success.
#
# Run from anywhere:  docs/research/region-search/sources/verify-sources.sh

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
manifest="$here/expected-sources.tsv"

# The expected population, declared here rather than derived from the manifest, so that a manifest
# that lost rows fails instead of agreeing with itself.
expect_total=30
expect_vendored=10
expect_metadata_only=20
# Zero since 2026-08-06, and declared rather than dropped: the pending class is the record's own
# work-item channel, so asserting it is empty is what makes a re-opened request fail this check
# instead of passing unnoticed.
expect_pending=0

# Files that are the record itself rather than preserved upstream bytes.
self_files=("README.md" "expected-sources.tsv" "verify-sources.sh" ".gitattributes")

failures=0
fail() {
	printf 'FAIL: %s\n' "$1" >&2
	failures=$((failures + 1))
}

[ -r "$manifest" ] || {
	printf 'FAIL: manifest %s is missing or unreadable\n' "$manifest" >&2
	exit 2
}

ids=()
paths=()
classes=()
digests=()
while IFS=$'\t' read -r id class path digest; do
	case "$id" in '#'* | '') continue ;; esac
	ids+=("$id")
	classes+=("$class")
	paths+=("$path")
	digests+=("$digest")
done <"$manifest"

total="${#ids[@]}"
[ "$total" -eq "$expect_total" ] ||
	fail "manifest holds $total records, expected $expect_total"

if [ "$total" -gt 0 ]; then
	unique="$(printf '%s\n' "${ids[@]}" | sort -u | wc -l | tr -d ' ')"
	[ "$unique" -eq "$total" ] ||
		fail "manifest ids are not unique: $unique distinct of $total records"
fi

count_vendored=0
count_metadata=0
count_pending=0
declared_paths=()

for i in "${!ids[@]}"; do
	id="${ids[$i]}"
	class="${classes[$i]}"
	path="${paths[$i]}"
	digest="${digests[$i]}"

	case "$class" in
	vendored)
		count_vendored=$((count_vendored + 1))
		[ "$path" != "-" ] || {
			fail "$id: vendored record declares no path"
			continue
		}
		declared_paths+=("$path")
		[ -f "$here/$path" ] || {
			fail "$id: vendored file $path is missing"
			continue
		}
		[[ "$digest" =~ ^[0-9a-f]{64}$ ]] || {
			fail "$id: vendored record has no SHA-256 digest"
			continue
		}
		actual="$(shasum -a 256 "$here/$path" | cut -d' ' -f1)"
		[ "$actual" = "$digest" ] ||
			fail "$id: digest mismatch for $path (recorded $digest, found $actual)"
		;;
	metadata-only | pending-acquisition)
		if [ "$class" = "metadata-only" ]; then
			count_metadata=$((count_metadata + 1))
		else
			count_pending=$((count_pending + 1))
		fi
		[ "$path" = "-" ] ||
			fail "$id: $class record must not retain local bytes, but declares $path"
		[ "$digest" = "-" ] || [[ "$digest" =~ ^[0-9a-f]{64}$ ]] ||
			fail "$id: digest field is neither '-' nor a SHA-256 digest"
		;;
	*)
		fail "$id: unknown classification '$class'"
		;;
	esac
done

[ "$count_vendored" -eq "$expect_vendored" ] ||
	fail "vendored records: $count_vendored, expected $expect_vendored"
[ "$count_metadata" -eq "$expect_metadata_only" ] ||
	fail "metadata-only records: $count_metadata, expected $expect_metadata_only"
[ "$count_pending" -eq "$expect_pending" ] ||
	fail "pending-acquisition records: $count_pending, expected $expect_pending"

# Every byte on disk must be claimed by a vendored record, so an unrecorded file cannot masquerade
# as preserved evidence.
while IFS= read -r found; do
	rel="${found#"$here"/}"
	skip=0
	for self in "${self_files[@]}"; do
		[ "$rel" = "$self" ] && skip=1
	done
	[ "$skip" -eq 1 ] && continue
	claimed=0
	for path in "${declared_paths[@]}"; do
		[ "$rel" = "$path" ] && claimed=1
	done
	[ "$claimed" -eq 1 ] || fail "$rel is present on disk but absent from the manifest"
done < <(find "$here" -type f | sort)

if [ "$failures" -ne 0 ]; then
	printf '%d check(s) failed over %d declared records.\n' "$failures" "$total" >&2
	exit 1
fi

printf 'OK: %d records verified (%d vendored, %d metadata-only, %d pending-acquisition).\n' \
	"$total" "$count_vendored" "$count_metadata" "$count_pending"

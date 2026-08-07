#!/usr/bin/env bash
#
# Verifies the preserved source population behind the measured-feedback tuning-loop design
# against expected-sources.tsv: the tensor-program autotuning literature, the general-autotuning
# precedent, the adaptive-execution and parametric-plan line, and the benchmarking-statistics
# source.
#
# The check counts its population before it inspects it. An empty or truncated manifest, a
# mutated digest, an unknown classification, or a stray file under local/ each fail with a named
# reason, so silence can never be mistaken for success.
#
# The `local-only` classification is why this script differs from its numerics sibling. Those
# bytes are retained on one machine at a gitignored path and are deliberately absent from every
# clone, so their absence is not a failure — but it is never silent either. The summary always
# names how many local-only files were found and how many were absent, because a run that
# checked no bytes at all must not read like a run that checked them and found them good.
#
# Run from anywhere:  docs/research/cost-model/sources/verify-sources.sh

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
manifest="$here/expected-sources.tsv"

# The expected population, declared here rather than derived from the manifest, so that a
# manifest that lost rows fails instead of agreeing with itself.
expect_total=14
expect_vendored=0
expect_local_only=4
expect_metadata_only=10
expect_pending=0

# Files that are the record itself rather than preserved upstream bytes.
self_files=("README.md" "expected-sources.tsv" "verify-sources.sh")

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
count_local=0
count_metadata=0
count_pending=0
local_present=0
local_absent=0
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
	local-only)
		count_local=$((count_local + 1))
		# A local-only row must point into local/ and must carry a real digest. The digest is
		# the entire value of the row on a machine that does not hold the bytes: it is what a
		# re-acquisition is checked against, so "-" would leave the row asserting nothing.
		case "$path" in
		local/*) ;;
		*)
			fail "$id: local-only record must declare a path under local/, found '$path'"
			continue
			;;
		esac
		declared_paths+=("$path")
		[[ "$digest" =~ ^[0-9a-f]{64}$ ]] || {
			fail "$id: local-only record has no SHA-256 digest"
			continue
		}
		if [ -f "$here/$path" ]; then
			local_present=$((local_present + 1))
			actual="$(shasum -a 256 "$here/$path" | cut -d' ' -f1)"
			[ "$actual" = "$digest" ] ||
				fail "$id: digest mismatch for $path (recorded $digest, found $actual)"
		else
			# Expected on every machine but the one holding the bytes. Reported, never fatal.
			local_absent=$((local_absent + 1))
		fi
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
[ "$count_local" -eq "$expect_local_only" ] ||
	fail "local-only records: $count_local, expected $expect_local_only"
[ "$count_metadata" -eq "$expect_metadata_only" ] ||
	fail "metadata-only records: $count_metadata, expected $expect_metadata_only"
[ "$count_pending" -eq "$expect_pending" ] ||
	fail "pending-acquisition records: $count_pending, expected $expect_pending"

# Every byte on disk must be claimed by a record, so an unrecorded file cannot masquerade as
# preserved evidence. This sweep is the reason the manifest exists at all: local/ is gitignored,
# so `git status` reports nothing there and a stray licence-restricted PDF would otherwise sit
# unnoticed and unattributed. The sweep descends into local/ precisely because git does not.
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

printf 'OK: %d records verified (%d vendored, %d local-only, %d metadata-only, %d pending-acquisition).\n' \
	"$total" "$count_vendored" "$count_local" "$count_metadata" "$count_pending"
printf 'local-only bytes: %d present and digest-verified, %d absent from this machine.\n' \
	"$local_present" "$local_absent"
if [ "$local_present" -eq 0 ] && [ "$count_local" -gt 0 ]; then
	printf 'Note: no local-only bytes were checked on this machine, so this run verified digests for none of them.\n'
fi

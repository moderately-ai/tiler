#!/usr/bin/env bash
#
# Ties this spike's copied items back to the sources they were copied from.
#
# `crates/tiler-ir` does not compile under Kani's bundled rustc, so `src/lib.rs`
# holds *copies* of the encoders and of the types they range over. A proof over a
# copy proves the copy. This script is the only thing that says the copy is still
# the same code: for every `/// @source: <path> :: <item>` marker in `src/lib.rs`
# it re-extracts `<item>` from `<path>` and compares it against the copy.
#
# WHAT IT COMPARES. Token content, not text. Both sides are normalized by
# dropping whole-line comments and attribute lines, collapsing whitespace runs,
# and stripping the visibility prefix. So reformatting, doc-comment edits, and a
# `pub(super)` to `pub` change do not trip it, while a renamed field, an added
# enum variant, a changed tag literal, a dropped `bytes.push`, or a reordered
# write does. Those are the drifts that would invalidate a proof.
#
# WHAT IT DOES NOT DO. It does not run unless someone runs it — no `make` target
# reaches this directory, by the standing spikes discipline. It is a staleness
# detector for a human to run alongside `cargo kani`, not a gate.
#
# Usage, from this directory:
#     ./guard.sh
# Exits 0 when every copy matches. Otherwise reports *every* divergence with both
# sides printed, then exits 1 — all of them, because a source refactor typically
# moves several copies at once and stopping at the first would hide the rest.

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "${here}/../../.." && pwd)"
shim="${here}/src/lib.rs"

# Extracts one top-level item by name from a rustfmt-formatted Rust file.
#
# Relies on rustfmt's layout: a top-level item starts at column 0 and ends either
# at a `}` at column 0 or, for a tuple struct, at the `;` on its own first line.
extract() {
    local file="$1" name="$2"
    awk -v name="${name}" '
        BEGIN { started = 0 }
        started == 0 {
            # Match `fn NAME(`, `enum NAME {`, `struct NAME(` / `{`, allowing any
            # visibility and `const`, anchored at column 0.
            if ($0 ~ "^(pub(\\([a-z]+\\))? )?(const )?(fn|enum|struct) " name "[ (<{;]") {
                started = 1
                print
                # A tuple struct declared and closed on one line.
                if ($0 ~ ";[ \t]*$") { exit }
            }
            next
        }
        { print }
        /^}/ { exit }
    ' "${file}"
}

# Drops whole-line comments and attribute lines, collapses whitespace, strips
# the visibility prefix.
#
# Whole-line only: a trailing comment is left in place rather than parsed out,
# because stripping one needs to know whether the `//` sits inside a string
# literal. Leaving it can only produce a false *positive* — a reported drift that
# is really a comment edit — which is the safe direction for a staleness check.
#
# Visibility is stripped everywhere rather than only on the item line, and a
# trailing comma before a closing brace is dropped, because neither changes the
# set of values a type has or the bytes a function writes — and both differ
# routinely between a `pub(crate)` field in a crate and its copy here, or between
# a struct variant rustfmt broke across lines and one it kept inline.
normalize() {
    grep -v -e '^[[:space:]]*//' -e '^[[:space:]]*#\[' \
        | tr '\n' ' ' \
        | tr -s ' \t' ' ' \
        | sed -e 's/pub(crate) //g' -e 's/pub(super) //g' -e 's/pub //g' \
              -e 's/, }/ }/g' \
              -e 's/^ *//' -e 's/ *$//'
}

checked=0
failed=0

while IFS= read -r marker; do
    rest="${marker#*@source: }"
    path="${rest%% :: *}"
    item="${rest##* :: }"

    source_file="${repo}/${path}"
    if [[ ! -f "${source_file}" ]]; then
        printf 'MISSING SOURCE FILE for %s: %s\n' "${item}" "${source_file}" >&2
        failed=$((failed + 1))
        continue
    fi

    from_source="$(extract "${source_file}" "${item}" | normalize)"
    from_shim="$(extract "${shim}" "${item}" | normalize)"

    if [[ -z "${from_source}" ]]; then
        printf 'NOT FOUND in source: %s in %s\n' "${item}" "${path}" >&2
        failed=$((failed + 1))
        continue
    fi
    if [[ -z "${from_shim}" ]]; then
        printf 'NOT FOUND in shim: %s\n' "${item}" >&2
        failed=$((failed + 1))
        continue
    fi

    checked=$((checked + 1))
    if [[ "${from_source}" != "${from_shim}" ]]; then
        failed=$((failed + 1))
        printf 'DRIFT: %s (%s)\n  source: %s\n  shim:   %s\n\n' \
            "${item}" "${path}" "${from_source}" "${from_shim}" >&2
    fi
done < <(grep '@source:' "${shim}")

# A guard that matched nothing must not look clean. The count is asserted rather
# than trusted, for the same reason the exhaustive-injectivity tests assert their
# populations: a marker syntax that silently stopped matching would compare
# nothing and report no divergence, and "nothing ran" would then be
# indistinguishable from "nothing drifted".
expected=30
if [[ "${checked}" -ne "${expected}" ]]; then
    printf 'GUARD POPULATION CHANGED: compared %d items, expected %d.\n' \
        "${checked}" "${expected}" >&2
    printf 'Update the expected count in guard.sh deliberately, in the same commit.\n' >&2
    exit 1
fi

if [[ "${failed}" -ne 0 ]]; then
    printf '\n%d of %d copied items have drifted from their sources.\n' \
        "${failed}" "${checked}" >&2
    exit 1
fi

printf '%d copied items match their sources.\n' "${checked}"

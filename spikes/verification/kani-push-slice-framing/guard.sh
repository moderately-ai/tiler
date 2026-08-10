#!/usr/bin/env bash
# Independently ties this spike's two copied framing primitives to live source.

set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "${here}/../../.." && pwd)"
shim="${here}/src/lib.rs"
source_file="${repo}/crates/tiler-ir/src/identity.rs"

extract() {
    local file="$1" name="$2"
    awk -v name="${name}" '
        BEGIN { started = 0 }
        started == 0 {
            if ($0 ~ "^(pub(\\([a-z]+\\))? )?(const )?fn " name "[ (]") {
                started = 1
                print
            }
            next
        }
        { print }
        /^}/ { exit }
    ' "${file}"
}

normalize() {
    grep -v -e '^[[:space:]]*//' -e '^[[:space:]]*#\[' \
        | tr '\n' ' ' \
        | tr -s ' \t' ' ' \
        | sed -e 's/pub(crate) //g' -e 's/pub(super) //g' -e 's/pub //g' \
              -e 's/^ *//' -e 's/ *$//'
}

checked=0
failed=0
while IFS= read -r marker; do
    item="${marker##* :: }"
    from_source="$(extract "${source_file}" "${item}" | normalize)"
    from_shim="$(extract "${shim}" "${item}" | normalize)"
    if [[ -z "${from_source}" || -z "${from_shim}" ]]; then
        printf 'NOT FOUND: %s\n' "${item}" >&2
        failed=$((failed + 1))
        continue
    fi
    checked=$((checked + 1))
    if [[ "${from_source}" != "${from_shim}" ]]; then
        printf 'DRIFT: %s\n  source: %s\n  shim:   %s\n' \
            "${item}" "${from_source}" "${from_shim}" >&2
        failed=$((failed + 1))
    fi
done < <(grep '@source: crates/tiler-ir/src/identity.rs ::' "${shim}")

expected=2
if [[ "${checked}" -ne "${expected}" ]]; then
    printf 'FRAMING GUARD POPULATION CHANGED: compared %d items, expected %d.\n' \
        "${checked}" "${expected}" >&2
    exit 1
fi
if [[ "${failed}" -ne 0 ]]; then
    printf '%d of %d framing copies have drifted.\n' "${failed}" "${checked}" >&2
    exit 1
fi
printf '%d framing copies match live source.\n' "${checked}"

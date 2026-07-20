#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/tiler-family-cfg.XXXXXX")

rustc --edition 2021 -D warnings \
    "$root/family_cfg_fallback.rs" -o "$scratch/fallback"
"$scratch/fallback"

if rustc --edition 2021 \
    "$root/family_cfg_required_fail.rs" -o "$scratch/required" \
    2>"$scratch/required.stderr"; then
    echo "matching-family compile unexpectedly succeeded" >&2
    exit 1
fi
grep -F "selected macOS artifact family could not be built" \
    "$scratch/required.stderr" >/dev/null

assert_cfg() {
    target=$1
    expected=$2
    rustc --print cfg --target "$target" | grep -Fx "$expected" >/dev/null
}

assert_cfg aarch64-apple-darwin 'target_os="macos"'
assert_cfg aarch64-apple-ios 'target_abi=""'
assert_cfg aarch64-apple-ios-sim 'target_abi="sim"'
assert_cfg aarch64-apple-ios-macabi 'target_abi="macabi"'

echo "family cfg delivery probe passed; local output retained at $scratch"

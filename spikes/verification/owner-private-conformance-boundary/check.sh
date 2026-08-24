#!/bin/sh
set -eu

fixture_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
scratch_dir=$(mktemp -d "${TMPDIR:-/tmp}/tiler-owner-boundary.XXXXXX")
trap 'rm -rf "$scratch_dir"' EXIT HUP INT TERM

run_check() {
    CARGO_TARGET_DIR="$scratch_dir/target" cargo check --locked --manifest-path "$fixture_dir/Cargo.toml" "$@"
}

if run_check -p consumer --features probe-test-only >"$scratch_dir/test-only.log" 2>&1; then
    echo "dependency unexpectedly observed an owner cfg(test) item" >&2
    exit 1
fi
grep -F "cannot find function \`test_only_inventory\` in crate \`owner\`" "$scratch_dir/test-only.log"

if run_check -p consumer --features probe-private >"$scratch_dir/private.log" 2>&1; then
    echo "dependency unexpectedly observed an owner private item" >&2
    exit 1
fi
grep -F "function \`private_inventory\` is private" "$scratch_dir/private.log"

run_check -p consumer --features probe-feature

OWNER_MANIFEST_OUT="$scratch_dir/owner-manifest.txt" \
    CARGO_TARGET_DIR="$scratch_dir/target" \
    cargo test --locked --manifest-path "$fixture_dir/Cargo.toml" -p owner \
    tests::emit_private_inventory -- --exact

expected='owner.subject.alpha@1
owner.subject.beta@1'
actual=$(sed -n '1,2p' "$scratch_dir/owner-manifest.txt")
test "$actual" = "$expected"
test "$(wc -l < "$scratch_dir/owner-manifest.txt" | tr -d ' ')" = 2

OWNER_MANIFEST_OUT="$scratch_dir/owner-feature-manifest.txt" \
    CARGO_TARGET_DIR="$scratch_dir/target" \
    cargo test --locked --manifest-path "$fixture_dir/Cargo.toml" -p owner \
    --features conditional-subject tests::emit_private_inventory -- --exact

grep -Fx "owner.subject.conditional@1" "$scratch_dir/owner-feature-manifest.txt"
test "$(wc -l < "$scratch_dir/owner-feature-manifest.txt" | tr -d ' ')" = 3

echo "owner-private boundary fixture: 2 refusals, 1 conditional-public success, 1 private emitter success, 1 configuration-dependent population"

#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/tiler-macro-env.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
cp -R "$root/fixture" "$scratch/fixture"

manifest="$scratch/fixture/Cargo.toml"
consumer="$scratch/fixture/consumer/src/lib.rs"
macro_source="$scratch/fixture/probe-macro/src/lib.rs"
trace="$scratch/trace.log"
cache="$scratch/cache"
export TILER_TRACE_PATH="$trace"
export TILER_PROBE_CACHE="$cache"

line_count() {
    if test -f "$trace"; then wc -l < "$trace" | tr -d ' '; else echo 0; fi
}

export TILER_TOOLCHAIN_FINGERPRINT=xcode-a
cargo check --manifest-path "$manifest" --quiet
test "$(line_count)" = 1

cargo check --manifest-path "$manifest" --quiet
test "$(line_count)" = 1

export TILER_TOOLCHAIN_FINGERPRINT=xcode-b
cargo check --manifest-path "$manifest" --quiet
test "$(line_count)" = 1

printf '\n// unrelated consumer edit\n' >> "$consumer"
cargo check --manifest-path "$manifest" --quiet
test "$(line_count)" = 2

rm -rf "$cache"
cargo check --manifest-path "$manifest" --quiet
test "$(line_count)" = 2

printf '\n// second unrelated consumer edit\n' >> "$consumer"
cargo check --manifest-path "$manifest" --quiet
test "$(line_count)" = 3

printf '\n// macro crate edit\n' >> "$macro_source"
cargo check --manifest-path "$manifest" --quiet
test "$(line_count)" = 4

cargo test --manifest-path "$manifest" --quiet

echo "macro environment probe: native freshness witnesses passed"
cat "$trace"

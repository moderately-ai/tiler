#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/tiler-macro-target.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
cp -R "$root/fixture" "$scratch/fixture"

host_target=$(rustc -vV | sed -n 's/^host: //p')
requested_target=${1:-$host_target}
trace="$scratch/trace.log"
cache="$scratch/cache"
export TILER_TRACE_PATH="$trace"
export TILER_PROBE_CACHE="$cache"
export TILER_TOOLCHAIN_FINGERPRINT=target-probe

if ! rustup target list --installed | grep -Fx "$requested_target" >/dev/null; then
    echo "target is not installed: $requested_target" >&2
    echo "installed targets:" >&2
    rustup target list --installed >&2
    exit 2
fi

cargo check \
    --manifest-path "$scratch/fixture/Cargo.toml" \
    --target "$requested_target" \
    --quiet

echo "host=$host_target requested_target=$requested_target"
cat "$trace"

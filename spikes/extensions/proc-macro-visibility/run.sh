#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

cargo test --manifest-path "$root/Cargo.toml"

cycle_log="$root/target/cycle-error.log"
if cargo metadata --manifest-path "$root/cycle/consumer/Cargo.toml" >"$cycle_log" 2>&1; then
    echo "cycle fixture unexpectedly succeeded" >&2
    exit 1
fi

grep -q "cyclic package dependency" "$cycle_log"
echo "proc-macro visibility probe: all witnesses passed"

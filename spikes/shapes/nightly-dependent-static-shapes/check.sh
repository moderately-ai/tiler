#!/usr/bin/env bash
set -euo pipefail

spike_dir="$(cd "$(dirname "$0")" && pwd)"
readonly spike_dir
if [[ $# -ne 2 || ! -x "$1" || "$2" != nightly-* ]]; then
    echo "usage: $0 <absolute-rustup> <exact-dated-nightly>" >&2
    exit 2
fi
readonly rustup="$1"
readonly toolchain="$2"

cd "$spike_dir"
"$rustup" run "$toolchain" cargo fmt --all --check
"$rustup" run "$toolchain" cargo check --workspace --all-targets --locked
"$rustup" run "$toolchain" cargo clippy --workspace --all-targets --locked -- -D warnings
"$rustup" run "$toolchain" cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" \
    "$rustup" run "$toolchain" cargo doc --workspace --no-deps --locked

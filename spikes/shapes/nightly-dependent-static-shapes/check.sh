#!/usr/bin/env bash
set -euo pipefail

spike_dir="$(cd "$(dirname "$0")" && pwd)"
readonly spike_dir
readonly toolchain="${1:-nightly-2026-07-19}"

cd "$spike_dir"
cargo "+${toolchain}" fmt --all --check
cargo "+${toolchain}" check --workspace --all-targets
cargo "+${toolchain}" clippy --workspace --all-targets -- -D warnings
cargo "+${toolchain}" test --workspace
RUSTDOCFLAGS="${RUSTDOCFLAGS:-} -D warnings" \
    cargo "+${toolchain}" doc --workspace --no-deps

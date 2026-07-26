#!/usr/bin/env bash
# Re-verify this spike's retained `trybuild` diagnostics.
#
# Run it when you are working on this spike. Nothing else runs it: the
# repository's `make` targets cover `crates/` only, and a spike is a recorded
# measurement rather than something to re-execute on every change.
#
# The goldens are byte-compared, so they are only meaningful under the toolchain
# they were captured on. Plain `cargo` gets that here: this directory sits under
# the repository root, so rustup walks up and resolves the same
# `rust-toolchain.toml` pin -- including `rust-src`, without which a const-eval
# panic renders differently and every golden mismatches.

set -euo pipefail

cd "$(dirname "$0")"

cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

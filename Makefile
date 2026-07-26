# Tiler's verification targets.
#
# Every recipe below is one command you can also just type. There is no
# orchestration layer: `rust-toolchain.toml` selects the compiler, `--locked`
# rejects a lockfile edit, and `set -e` is make's default per-recipe behaviour.
#
# Spikes deliberately have no target. A spike is a recorded measurement, and it
# is re-run from its own directory by whoever is working on it.

.PHONY: check fmt build lint test doc full

# The working loop.
check: fmt build lint test

fmt:
	cargo fmt --all --check

build:
	cargo check --workspace --all-targets --locked

lint:
	cargo clippy --workspace --all-targets --locked -- -D warnings

test:
	cargo test --workspace --locked

doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

# What CI runs, and what to run before pushing to main.
full: check doc
	cargo test --release --locked -p tiler-reference -p tiler-compiler
	ticketsplease lint
	shellcheck --severity style deps.sh

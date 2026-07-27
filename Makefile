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

# Prototypes are excluded deliberately. They are non-published, experimental,
# and deleted or rewritten as the slice they prove moves; holding them to the
# same style bar as the crates costs edits that teach nobody anything. They are
# still built and still tested by the targets above — only the style pass skips
# them, so a prototype that stops compiling or stops passing still fails.
lint:
	cargo clippy --workspace --all-targets --locked \
		--exclude tiler-prototype-run --exclude tiler-prototype-compile -- -D warnings

# Two commands because nextest does not run doc-tests, at all. Dropping the
# second would silently stop running the compile-fail doc-tests on
# `Preflight::commit`, which are the compiler-checked evidence for ADR 0051's
# one-way routing commit — they would pass by never being compiled.
# `.config/nextest.toml` is what makes the first quiet on a green run.
test:
	cargo nextest run --workspace --locked
	cargo test --workspace --doc --locked --quiet

doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

# What CI runs, and what to run before pushing to main.
full: check doc
	cargo nextest run --release --locked -p tiler-reference -p tiler-compiler
	ticketsplease lint
	shellcheck --severity style deps.sh

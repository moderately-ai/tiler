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

# The second and third commands exist because `cargo fmt` reaches cargo targets
# and the facade's `trybuild` fixtures are not targets: they are source files a
# test compiles as a separate crate at run time, so the first command never sees
# them. The count is asserted rather than trusted — a glob that has stopped
# matching produces no complaints, which is indistinguishable from a population
# that is clean. Adding a fixture updates the number in the same commit; that is
# the intended failure, not an obstacle.
#
# `pass/` only. Every `fail/` fixture is paired with a `.stderr` golden that
# `trybuild` compares byte for byte, and all nine goldens quote the fixture's own
# source under `--> tests/facade/fail/<name>.rs:LINE:COL` headers with caret
# columns beneath. Reformatting one moves a line number or a caret and breaks the
# golden, which costs more than the blind spot it would close. The narrower
# exclusions — spans inside a `pass/` fixture that are verbatim macro-emitter
# output, which the macro crate's tests assert the file still contains — carry
# `#[rustfmt::skip]` at the item itself, so each is stated where it applies
# instead of turning into a second list here.
fmt:
	cargo fmt --all --check
	test $$(ls crates/tiler/tests/facade/pass/*.rs | wc -l) -eq 10
	rustfmt --check crates/tiler/tests/facade/pass/*.rs

build:
	cargo check --workspace --all-targets --locked

# Prototypes are excluded deliberately. They are non-published, experimental,
# and deleted or rewritten as the slice they prove moves; holding them to the
# same style bar as the crates costs edits that teach nobody anything. They are
# still built and still tested by the targets above — only the style pass skips
# them, so a prototype that stops compiling or stops passing still fails.
lint:
	cargo clippy --workspace --all-targets --locked \
		--exclude tiler-prototype-run --exclude tiler-prototype-compile \
		--exclude tiler-prototype-candle -- -D warnings

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

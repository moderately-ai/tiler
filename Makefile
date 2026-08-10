# Tiler's verification targets.
#
# Every recipe below is one command you can also just type. There is no
# orchestration layer: `rust-toolchain.toml` selects the compiler, `--locked`
# rejects a lockfile edit, and `set -e` is make's default per-recipe behaviour.
#
# Spikes deliberately have no target. A spike is a recorded measurement, and it
# is re-run from its own directory by whoever is working on it.

.PHONY: check citations fmt build lint test doc full

# The working loop. `citations` runs first because it is the cheapest and
# needs no toolchain, so a stale ticket citation is reported in half a second
# rather than after a build.
check: citations fmt build lint test

# Resolve the pinned source citations and the local markdown links in open
# tickets and live documents against the working tree. This is in `check`
# rather than only in `full` on purpose. `tickets/` is not in the delta rule's
# gated set (see AGENTS.md, "Verify and ship"), so a ticket-only change carries
# the previous green gate and reruns `tkt lint` alone -- a check reachable only
# from `full` would never see the edits it exists to catch. Run this target
# beside `tkt lint` for a ticket-only delta.
#
# A green run means the citations and the links point somewhere. It does not
# mean the tickets are true, and it does not mean a link that resolves reaches
# the document it claims to; the script's own header is explicit about both
# differences, and AGENTS.md carries the reading obligation that governs them.
citations:
	./check-citations.sh

# The remaining commands exist because `cargo fmt` reaches cargo targets and
# the `trybuild` fixtures are not targets: they are source files a test compiles
# as a separate crate at run time, so the first command never sees them. The
# count is asserted rather than trusted — a glob that has stopped matching
# produces no complaints, which is indistinguishable from a population that is
# clean. Adding a fixture updates the number in the same commit; that is the
# intended failure, not an obstacle.
#
# `pass/` only. Every `fail/` fixture is paired with a `.stderr` golden that
# `trybuild` compares byte for byte. The nine facade goldens and all seventeen
# `tiler-ir` goldens quote their fixture source under `--> tests/...:LINE:COL`
# headers with caret columns beneath; reformatting one moves a line number or a
# caret and breaks its golden, which costs more than the blind spot it would
# close. The narrower exclusions — spans inside a facade `pass/` fixture that
# are verbatim macro-emitter output, which the macro crate's tests assert the
# file still contains — carry `#[rustfmt::skip]` at the item itself, so each is
# stated where it applies instead of turning into a second list here.
fmt:
	cargo fmt --all --check
	test $$(ls crates/tiler/tests/facade/pass/*.rs | wc -l) -eq 10
	rustfmt --check crates/tiler/tests/facade/pass/*.rs
	test $$(ls crates/tiler-ir/tests/index-region/pass/*.rs | wc -l) -eq 1
	rustfmt --check crates/tiler-ir/tests/index-region/pass/*.rs
	test $$(ls crates/tiler-ir/tests/shape-evidence/pass/*.rs | wc -l) -eq 2
	rustfmt --check crates/tiler-ir/tests/shape-evidence/pass/*.rs
	test $$(ls crates/tiler-ir/tests/typed-handles/pass/*.rs | wc -l) -eq 1
	rustfmt --check crates/tiler-ir/tests/typed-handles/pass/*.rs

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

# The floors below extend the `fmt` target's reasoning — "a glob that has
# stopped matching produces no complaints, which is indistinguishable from a
# population that is clean" — to the globs `trybuild` itself expands. A
# zero-match glob is silent at every layer: `expand_globs` collects an empty
# match list without an error, a case that never expanded is not something
# `Runner::run` can count as a failure, and the "no trybuild tests enabled"
# notice is both a bare `println!` and unreachable when one half collapses,
# because each harness registers a `pass` glob and a `fail` glob against one
# `TestCases` and the surviving half keeps the list non-empty. Rename a fixture
# directory and the harness reports `ok` while testing nothing.
#
# That silence costs more here than an ordinary missing assertion. These globs
# carry compile-fail evidence, which is the kind this gate already goes out of
# its way to keep — the `--doc` command below is retained for a *different*
# population of it, the `Preflight::commit` doc-tests, and nothing was doing the
# equivalent job for these. Two of the nine facade fixtures are also read by
# name from `tiler-macros`; the other seven and all seventeen `tiler-ir`
# compile-fail fixtures are held up by nothing but the count on their line.
#
# One floor per glob, stated next to the command that consumes it. The four pass
# populations — facade plus the three `tiler-ir` directories — are floored in
# `fmt` above because `rustfmt --check` reads them there; duplicating their
# counts here would only give each count two places to be wrong.
#
# Two commands because nextest does not run doc-tests, at all. Dropping the
# second would silently stop running the compile-fail doc-tests on
# `Preflight::commit`, which are the compiler-checked evidence for ADR 0051's
# one-way routing commit — they would pass by never being compiled.
# `.config/nextest.toml` is what makes the first quiet on a green run.
test:
	test $$(ls crates/tiler/tests/facade/fail/*.rs | wc -l) -eq 9
	test $$(ls crates/tiler-ir/tests/index-region/fail/*.rs | wc -l) -eq 4
	test $$(ls crates/tiler-ir/tests/shape-evidence/fail/*.rs | wc -l) -eq 7
	test $$(ls crates/tiler-ir/tests/typed-handles/fail/*.rs | wc -l) -eq 6
	cargo nextest run --workspace --locked
	cargo test --workspace --doc --locked --quiet

doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

# What CI runs, and what to run before pushing to main.
full: check doc
	cargo nextest run --release --locked -p tiler-reference -p tiler-compiler
	ticketsplease lint
	shellcheck --severity style deps.sh check-citations.sh

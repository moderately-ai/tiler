---
schema: "tiler-doc/v1"
id: "tiler.spike.extensions"
kind: "experiment"
title: "Operation-extension experiments"
topics: ["extensions", "proc-macro", "rust"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.extensions.operation-extension-surface", "tiler.research.extensions.operation-extension-api", "tiler.research.extensions.proc-macro-extension-visibility"]
entrypoints: ["spikes/extensions/run.py", "spikes/extensions/non-exhaustive-visibility/Cargo.toml"]
last_verified: "2026-07-25"
ticket: "operation-extension-surface"
---

# Operation-extension experiments

The `operation-api` crate compile-checks the proposed capability boundary. The
`proc-macro-visibility` workspace demonstrates which providers a stable proc
macro can observe across host and consumer crate boundaries. The
`non-exhaustive-visibility` workspace measures what `#[non_exhaustive]` does and does not constrain across a crate boundary, which is the evidence behind ADR 0074's amended convention 5.

Run from the repository root:

```sh
python3 spikes/extensions/run.py
```

The runner gives the complete suite a five-minute deadline, runs the
proc-macro observation twice, rejects missing success markers or the wrong
cycle diagnostic, bounds each command's combined output to four MiB, and
records source/toolchain provenance plus command output
in the ignored `spikes/extensions/proc-macro-visibility/target/` directory.
Source provenance includes full tracked/untracked status and bounded digests of
every Rust, Python, shell, and Cargo fixture input.
Run `python3 spikes/extensions/run.py --self-test` to exercise malformed-output
and timeout handling without invoking Cargo.
It requires Python 3.11 or newer and POSIX process-group behavior on the
repository's supported macOS and Debian-family development hosts.

The API names remain experimental. The visibility result is bounded to the
recorded Rust/Cargo compilation model and does not establish a plugin ABI.

## The `non-exhaustive-visibility` workspace

Two crates: `defining` exports a two-variant `#[non_exhaustive] enum Growing` and maps it totally with no wildcard arm; `consuming` depends on it and holds every cross-crate form. Each compile-fail case is paired with the compiling contrast it is evidence against, because a diagnostic on its own states what the compiler rejects and not what it accepts. The forms that must *not* compile are `trybuild` fixtures under [`consuming/tests/ui/fail/`](non-exhaustive-visibility/consuming/tests/ui/fail), where the retained `.stderr` beside each case is compared byte for byte and is the measurement itself. The compiling contrasts are under [`consuming/tests/ui/pass/`](non-exhaustive-visibility/consuming/tests/ui/pass).

**Measurement — same crate versus consuming crate.** [`cross_crate_total_map.rs`](non-exhaustive-visibility/consuming/tests/ui/fail/cross_crate_total_map.rs) is the body of `defining`'s `same_crate_total_map` moved across the crate boundary and otherwise unchanged. It fails `error[E0004]: non-exhaustive patterns: '&_' not covered`, noting that the enum "is marked as non-exhaustive, so a wildcard `_` is necessary to match exhaustively". The same body compiles inside `defining` and in the same-file [`same_crate_total_map.rs`](non-exhaustive-visibility/consuming/tests/ui/pass/same_crate_total_map.rs). This is the asymmetry ADR 0074 relies on when it says a same-crate encoder keeps its compile-time guard and a cross-crate one cannot.

**Measurement — the `non_exhaustive_omitted_patterns` alternative.** With `#![feature(non_exhaustive_omitted_patterns_lint)]`, [`omitted_patterns_denied.rs`](non-exhaustive-visibility/consuming/tests/ui/fail/omitted_patterns_denied.rs) keeps its wildcard, omits one known variant, and fails with "some variants are not matched explicitly"/"pattern `&Growing::B` not covered". Listing every variant alongside the wildcard compiles. Without the feature gate the lint name is unknown and the attribute is inert: [`omitted_patterns_warns_without_feature.rs`](non-exhaustive-visibility/consuming/tests/ui/pass/omitted_patterns_warns_without_feature.rs) compiles with the omission intact, and [`omitted_patterns_inert_without_feature.rs`](non-exhaustive-visibility/consuming/tests/ui/fail/omitted_patterns_inert_without_feature.rs) shows what the repository gate's warning-free requirement turns that warning into. Its retained diagnostic is also the negative evidence: the recorded claim forbids "some variants are not matched explicitly" from appearing in it, so an inertness that quietly became a real check would fail.

**Measurement — the attribute does not reach a discriminant cast.** [`cross_crate_discriminant_cast.rs`](non-exhaustive-visibility/consuming/tests/ui/pass/cross_crate_discriminant_cast.rs) writes `value as u8` on `Growing` from outside the defining crate and compiles. [`cross_crate_discriminant_tag_match.rs`](non-exhaustive-visibility/consuming/tests/ui/fail/cross_crate_discriminant_tag_match.rs) is the same total map written as the convention-3 match, deriving the same tag for the same variant across the same boundary, and it fails `error[E0004]: non-exhaustive patterns: '_' not covered`. The pair is the measurement rather than either half: `#[non_exhaustive]` reaches the written match and not the cast, so marking a vocabulary would not have prevented a discriminant-cast encoder from silently re-encoding every subject when the vocabulary grew. That is why ADR 0074 puts the discriminant rule in convention 3 — replace the cast with a match — rather than in convention 5. The recorded claim forbids the sibling case's `` `&_` not covered `` from appearing here, since the two share `E0004` and differ only in whether the scrutinee is a reference.

**Measurement — and neither does the strictest available lint.** [`cast_ignores_denied_omitted_patterns.rs`](non-exhaustive-visibility/consuming/tests/ui/pass/cast_ignores_denied_omitted_patterns.rs) enables `non_exhaustive_omitted_patterns_lint` and denies the lint crate-wide, and the cast still compiles. A compiling fixture alone cannot tell that apart from a crate-level lint level the compiler accepts and never consults, and the `omitted_patterns_denied.rs` case above only rules that out for a `#[deny]` written on the match itself — a granularity no cast site has. [`omitted_patterns_denied_at_crate_level.rs`](non-exhaustive-visibility/consuming/tests/ui/fail/omitted_patterns_denied_at_crate_level.rs) is therefore the control: it denies the lint at crate level and contains both constructs, and its retained diagnostic reports the omitting match, names the crate-level attribute as the level that fired, and forbids the cast's own source text from appearing. So the hole convention 3 closes is not one a stricter lint level closes.

**Measurement boundary.** These are facts about one compiler, recorded in `results/` with its exact commit hash; `#[non_exhaustive]` interacting with `as` is not a stability guarantee and a later compiler could add a diagnostic there. The same-crate cast is deliberately not measured: `#[non_exhaustive]` is defined to be inert inside its defining crate, nothing in ADR 0074 rests on that cell, and convention 3 forbids reading a discriminant on either side of the boundary regardless.

Run the workspace directly, or through the suite runner:

```sh
cargo test --locked --manifest-path spikes/extensions/non-exhaustive-visibility/Cargo.toml
python3 spikes/extensions/run.py --suite non-exhaustive-visibility
```

[`results/`](non-exhaustive-visibility/results) records the exact toolchain each retained diagnostic was captured on, and the runner refuses to reuse a measurement whose channel is no longer the one `rust-toolchain.toml` pins. `non_exhaustive_omitted_patterns` is an unstable lint ([rust-lang/rust#89554](https://github.com/rust-lang/rust/issues/89554)) whose behaviour may change, so the pin comparison is deliberately fail-closed: bumping the toolchain must force a fresh run rather than let the old conclusion carry forward. Refresh a diagnostic with `TRYBUILD=overwrite` **only** after deciding the claim still holds, and re-record the toolchain in the same commit.

`run.py --self-test` verifies the retained evidence against that record without invoking Cargo, and `scripts/tests/test_research_harnesses.py` runs `--self-test` inside the repository gate. That predicate checks the record against the retained diagnostic; it cannot see whether the fixture beside it still *produces* that diagnostic, because only a Cargo run compares the two. **`compile-extension-spike-fixtures-in-the-gate` closed that half.** `scripts/check_rust.py` now compiles this workspace under the pinned toolchain on every gate invocation and requires the run to name each `trybuild` case, so a fixture edited until it no longer fails for its recorded reason fails the gate. The three sibling workspaces here retain no compiler-produced golden artifact and are still built only by hand: their conclusion is about whatever code is present, so nothing checked in can go stale against it. `AGENTS.md` states the admission rule the split follows.

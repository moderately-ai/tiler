---
id: strengthen-the-family-cfg-evidence-with-the-installed-cross-targets
title: Strengthen the family-cfg evidence with the installed cross-targets
status: done
priority: p2
dependencies: []
related: []
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

The five-target family-`cfg` matrix is proved by real cross-target compilation rather than by `rustc --print cfg` inference, so the claim "a nonmatching target compiles the semantic fallback" rests on a build that ran.

## Why

**Fact.** `generate-cfg-gated-artifact-family-delivery` recorded this as out of reach: "`rustup target list --installed` reports `aarch64-apple-darwin` alone, and installing a target is a host-toolchain change reserved to Tom." Tom authorized the change on 2026-07-31 and the coordinator installed exactly `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-apple-ios-macabi`, and `x86_64-unknown-linux-gnu` (rust-std components only, via rustup, removable with `rustup target remove`). The blocked evidence should now be re-run per AGENTS.md's rule that once authorized, the exact resulting component is recorded and the blocked measurement re-runs.

## Work

Compile the delivery emitter's matching, nonmatching, and retained-diagnostic fixture shapes with `cargo check --target <t>` (or the trybuild equivalent if it admits a target override) for each installed target, assert the expected pass/fail per the recorded five-target matrix, and record the evidence beside the existing `rustc --print cfg` derivation in the family_cfg tests or the delivery record — whichever the existing evidence idiom favours. Note the boundary: `cargo check`-level evidence needs no SDK or linker; a full link is out of scope.

## Closes when

Each of the five targets has a real compilation outcome recorded agreeing with the matrix, and the evidence states its check-only boundary.

## Outcome

**Done.** `every_emitted_shape_compiles_as_the_five_target_matrix_says` in `crates/tiler-macros/src/delivery/tests.rs` compiles the delivery emitter's own `DeliveryPlan::items_source` output for each of the five targets, and the measured matrix, its boundary, and its toolchain are recorded in that module's docs.

**Trybuild admits no target override.** `trybuild-1.0.118/src/cargo.rs:203-222` hardcodes `target_triple::TARGET` — the triple the *test binary* was built for — and gates it only on a `trybuild_no_target` cfg that removes `--target` entirely. Its whole public API is `TestCases::{new, pass, compile_fail}` (`grep -n 'pub fn ' src/lib.rs src/run.rs` in that crate returns exactly those three). So the harness shape the ticket describes was the only option.

**Shape: a harness inside the crate, not a script beside it.** The fixtures must be the emitter's own bytes or the evidence is about text someone typed twice, and `DeliveryPlan` is `pub(crate)` — unreachable from `tests/`, which is why an integration test or a standalone script cannot produce them. The harness is therefore a unit test in the module that already holds the three plan shapes, and it writes each fixture to a reconstructed temporary directory.

**Compilation is `rustc --edition 2024 --crate-type lib --emit=metadata --target <triple>`, not `cargo check --target`.** `--emit=metadata` is what `cargo check` runs; the fixtures are dependency-free, so a synthesized manifest would add a target directory and a lockfile resolution and nothing else to the verdict, and the toolchain resolves identically either way by directory ancestry. It is the same reason `family_cfg::tests::target_cfg` already spells `rustc` directly. The edition is stated because rustc's command-line default is 2015, where the emitter's `::core::` paths do not resolve at all — a fact the first probe run surfaced.

**Measurement — `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, 2026-08-01.** Fifteen compilations, three emitted shapes on five targets, every one agreeing with the matrix `the_emitted_arms_select_exactly_one_payload_per_consumer_target` and `a_retained_diagnostic_fires_only_on_the_family_it_names` derive from `rustc --print cfg`:

| emitted shape | `aarch64-apple-darwin` | `aarch64-apple-ios` | `aarch64-apple-ios-sim` | `aarch64-apple-ios-macabi` | `x86_64-unknown-linux-gnu` |
| --- | --- | --- | --- | --- | --- |
| macOS and iOS device both built | payload 1 | payload 0 | fallback | fallback | fallback |
| iOS device built, simulator retained | fallback | payload 0 | build fails on the retained diagnostic | fallback | fallback |
| macOS retained, nothing built | build fails on the retained diagnostic | compiles, no item survives `#[cfg]` | compiles, no item survives `#[cfg]` | compiles, no item survives `#[cfg]` | compiles, no item survives `#[cfg]` |

**Boundary — check level, no SDK, no link.** `#[cfg]` selection, selector totality, the 256-byte-value byte-string literal, and the `const` assertions are decided by rustc for the named target; no linker runs, no Apple SDK is consulted, and nothing is linked or executed. The evidence says the delivered *source* is correct per target. It says nothing about whether a `metallib` carried in it would load there.

**Why the compilation adds something the `cfg` sweep could not.** `rustc --print cfg` establishes which predicate holds where. It cannot establish that the emitted *items* are well-formed on a target, that exactly one selector arm survives `#[cfg]` there (a second is a duplicate definition, a gap is an undefined name), or that the byte-string literal lexes identically for a non-Apple target. Compiling for the target decides all three at once.

**`#[ignore]`d, for a host reason and not a cost one.** Seven runs reported 0.88 s to 1.42 s for the whole test — twenty compilations including five installed-target probes — against a suite that already carries a 13 s test, so cost does not decide it. What decides it is that `rust-toolchain.toml` declares `channel`, `profile`, and `components` and `deps.sh` verifies only that `components` array; neither declares a *target*. A gate-resident test would fail `make check` on a host bootstrapped exactly as this repository documents, and the files that would fix that are `implementation/workspace`, outside this ticket's scopes — and a host-toolchain policy change besides. A test that instead skipped the targets it could not find would report a clean pass over a population it never counted. Promotion is `declare-the-cross-compilation-targets-in-the-toolchain-manifest`, which records the 555 MB of `rust-std` the policy would cost every checkout.

**Proven able to fail.** Five perturbations, each restored:

1. `x86_64-unknown-linux-gnu` mapped to payload 0 in the both-built shape — failed on rustc's own `E0080: evaluation panicked` for that target.
2. The simulator row's expected fatal dropped — failed on the shape-consistency assertion, before any compile ran.
3. The retained diagnostic claimed to fire on `x86_64-unknown-linux-gnu` as well — failed on "must fail … and it compiled".
4. `require_installed_target` pointed at `aarch64-apple-tvos`, which this host lacks — failed with `E0463: can't find crate for std`, which is exactly the confusion the probe exists to prevent: without it that non-zero exit would have been counted as a retained diagnostic firing.
5. The fatal target's selector expectation also made wrong — failed on the compound-failure guard rather than passing on the retained text alone.

**Stale text corrected.** `family_cfg/tests.rs`'s claim that the checked targets have "none of whose standard libraries are installed on this host" named the five it now has (the host target was always among them, and the other four arrived on 2026-07-31); it and `delivery/tests.rs`'s "the only way to check the five-target matrix … without installing five standard libraries" are rewritten to say what each kind of evidence covers and why the `--print cfg` sweep is still the wider one. `family_cfg.rs`'s measurement section points at the compile evidence.

**Commands.** `cargo fmt --all --check`; `cargo clippy -p tiler-macros -p tiler --all-targets --locked -- -D warnings`; `cargo nextest run -p tiler-macros -p tiler --locked` (140 passed, 1 skipped); `cargo nextest run -p tiler-macros --locked --run-ignored all -E 'test(every_emitted_shape_compiles_as_the_five_target_matrix_says)'`; `tkt lint`; `git diff --check`; `tkt guard --base 0162efc tkt/strengthen-the-family-cfg-evidence-with-the-installed-cross-targets` (verdict ok; the five reported collisions are shared `project/tickets` claims); `make full` green — 1817 workspace tests, 626 release tests, all doc-tests, `ticketsplease lint`, and `shellcheck`.

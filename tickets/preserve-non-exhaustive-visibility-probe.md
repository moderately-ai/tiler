---
id: preserve-non-exhaustive-visibility-probe
title: Check in the two-crate probe behind ADR 0074's non-exhaustive measurements
status: in-progress
priority: p2
dependencies: []
related: [resolve-non-exhaustive-recognizer-hole, extend-canonical-identity-encodings-for-reserved-variants]
scopes: [research/extensions, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [research, spike, verification]
claimed_from: todo
assignee: agent-preserve-non-exhaustive-visibility-probe
lease_expires_at: 1784919376
---
The amendment to ADR 0074 rests on measurements of Rust's `#[non_exhaustive]`
behaviour, and those measurements have **no checked-in harness**. The two-crate
probe that produced them was written in an agent scratchpad and is gone.

`AGENTS.md` is explicit that this is not acceptable as a resting state:
reproducible experiments and *referenced measurements* belong in a dedicated
directory under `spikes/`, and a document should link to the checked-in harness
supporting its claim. The agent flagged the gap itself rather than quietly
leaving it, and correctly declined to check the probe in — `spikes/` was outside
its declared scopes.

The claims that currently have no reproduction:

- A same-crate exhaustive `match` over a `#[non_exhaustive]` enum compiles with no
  wildcard arm; the cross-crate form fails `E0004`. **This is load-bearing**: it
  is the entire reason `extend-canonical-identity-encodings-for-reserved-variants`
  is safe for the schedule encoders (same crate) but blocked for
  `fusion_legality::effect_tag` (cross crate). The amended convention 5 rests on
  it too.
- `#![feature(non_exhaustive_omitted_patterns_lint)]` with
  `#[deny(non_exhaustive_omitted_patterns)]` restores a hard compile error for an
  omitted variant across a crate boundary; without the feature gate the attribute
  is inert and yields only `unknown_lints`. This one decides whether a recorded
  alternative to the amendment is even available.

Build a small two-crate probe under `spikes/` that asserts both, in the style of
the existing `spikes/shapes/` and `spikes/macro-environment/` probes: a defining
crate exporting a `#[non_exhaustive]` enum, a consuming crate, and checks that the
expected forms compile and the expected forms fail with the exact diagnostic
codes. Record the exact toolchain — the measurement was taken on
`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, edition 2024 — and keep the
compiler-version dependence visible, since `non_exhaustive_omitted_patterns` is an
unstable lint whose behaviour may change.

Then link the probe from ADR 0074's amendment so the claim and its evidence are
connected, which is the point of the rule.

Pick a home deliberately: `research/extensions` is this ticket's scope because the
probe concerns public-API extension behaviour, but if a better area exists, use it
and say why. Follow the existing convention of a narrow `.gitignore` for
regenerable build output while keeping the source and any result fixture.

## Outcome

Both measurements reproduced exactly; neither disagrees with ADR 0074.

The probe lives at `spikes/extensions/non-exhaustive-visibility/`, a two-crate Cargo workspace of its own. `research/extensions` was the right home: the claim is about what a public vocabulary constrains across a crate boundary, which is the same question `proc-macro-visibility` and `operation-api` ask, and the directory already had a suite runner to extend. No other spike area was closer.

**Measurement — macOS arm64, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)` and `cargo 1.99.0-nightly (3efb1f477 2026-07-17)` selected by `rust-toolchain.toml`'s `nightly-2026-07-19`, edition 2024, trybuild 1.0.118.** `defining` exports a two-variant `#[non_exhaustive] enum Growing` and maps it totally with no wildcard arm; that compiles. The identical body in `consuming` fails `error[E0004]: non-exhaustive patterns: '&_' not covered`, noting that `Growing` "is marked as non-exhaustive, so a wildcard `_` is necessary to match exhaustively". With `#![feature(non_exhaustive_omitted_patterns_lint)]`, `#[deny(non_exhaustive_omitted_patterns)]` on a cross-crate match that keeps its wildcard and omits `Growing::B` fails with "some variants are not matched explicitly"/"pattern `&Growing::B` not covered"; listing every variant alongside the wildcard compiles. Without the gate the lint is unknown, the omission compiles with only `warning: unknown lint: 'non_exhaustive_omitted_patterns'`, and nothing about the omitted variant is reported at all.

Seven claims, six trybuild fixtures. The three compile-fail cases retain their diagnostics byte for byte as trybuild `.stderr` expectations, so a compiler that changes the code, the note, or which patterns it reports fails the probe rather than being absorbed. `results/2026-07-24-macos-arm64.json` records the exact toolchain and, per fixture, the expected first line, the diagnostic code, the fragments that must appear, and — for the inertness case — the fragments that must *not*, which is the only way "the attribute is inert" is falsifiable.

`spikes/extensions/run.py` gained a `non-exhaustive-visibility` suite and a retained-evidence predicate. The predicate needs no Cargo, so it runs inside the repository gate: `scripts/tests/test_research_harnesses.py` already invokes `run.py --self-test`, which now verifies the checked-in diagnostics against the record and then proves nine tampering paths are still rejected, including a `TRYBUILD=overwrite`-shaped refresh and a fixture added without a record. The Cargo fixtures themselves are compiled only when the suite is run by hand; the gate builds no Cargo fixture under `spikes/extensions/`.

The version dependence is fail-closed in both directions. The predicate refuses a measurement whose channel is no longer the pinned one, and the suite compares the running compiler's commit hash against the recorded one, so bumping `rust-toolchain.toml` forces a fresh run instead of letting an unstable-lint conclusion carry forward.

ADR 0074's convention 5 amendment now links the harness and each retained diagnostic. `contracts/decisions` was added to this ticket before that edit. Body text only: no frontmatter field that feeds a generated catalog was touched, and `scripts/docs.py render` produced no navigation diff.

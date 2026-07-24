---
id: preserve-non-exhaustive-visibility-probe
title: Check in the two-crate probe behind ADR 0074's non-exhaustive measurements
status: todo
priority: p2
dependencies: []
related: [resolve-non-exhaustive-recognizer-hole, extend-canonical-identity-encodings-for-reserved-variants]
scopes: [research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [research, spike, verification]
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

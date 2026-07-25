---
id: probe-the-non-exhaustive-discriminant-cast-hole
title: "Probe whether #[non_exhaustive] constrains a cross-crate discriminant cast"
status: in-progress
priority: p2
dependencies: []
related: [reconcile-adr-records-with-the-widened-numerical-vocabulary, preserve-non-exhaustive-visibility-probe]
scopes: [research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [research, api-hardening, identity]
claimed_from: todo
assignee: agent-extensions
lease_expires_at: 1784996299
---
ADR 0074 convention 5b now carries an **Inference** that `#[non_exhaustive]` does not constrain an `as` cast, so marking a vocabulary would not have prevented the four discriminant casts convention 3 now forbids. Every neighbouring claim in that record is a **Measurement** with a retained `trybuild` fixture; this one is not, and the asymmetry is the gap.

**Why it matters rather than being pedantry.** The inference is what makes convention 3 rather than convention 5 the home of the discriminant rule. If the attribute did constrain casting, 5b would already cover the site and convention 3 would not have needed amending. The record's reasoning therefore rests on it.

**What exists to extend.** `spikes/extensions/non-exhaustive-visibility/` already holds the harness that produced the `E0004` and `non_exhaustive_omitted_patterns` diagnostics ADR 0074 cites, with `consuming/tests/ui/fail/*.stderr` as retained expectations and a recorded environment row under `results/`. `scripts/check_rust.py` compiles it as a gated spike workspace, and `spikes/extensions/run.py --self-test` checks the retained diagnostics against the record.

**What closes this.** Add the case in the shape the existing fixtures use, on the toolchain `rust-toolchain.toml` pins. Note that a *passing* compile has no `.stderr` to retain, so decide how the harness records a positive result — the existing cases are all compile-failures, and a case whose evidence is "this compiled" needs a different retention form than a golden diagnostic, or it needs to be paired with a failing companion (for example, the same cast written as a wildcard-free `match`, which must fail with `E0004`) so that the pair is what the fixture pins. That pairing is probably the right answer because it pins the *contrast* the inference asserts, but decide it rather than assuming.

Then replace the Inference in ADR 0074 convention 5b with a Measurement citing the fixture, and remove the sentence naming this ticket. Do not change `decision_status`.

**Unverified reference measurement, recorded so it can be refuted rather than repeated.** A two-crate reproduction outside this repository, on an ambient `rustc 1.97.0` and not the repository pin, compiled `pub fn tag(value: Growing) -> u8 { value as u8 }` against a `#[non_exhaustive] pub enum Growing { A, B }` in a separate crate with exit 0. That is consistent with the inference and establishes nothing about the pinned toolchain; it is not evidence this ticket may cite.

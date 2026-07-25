---
id: probe-the-non-exhaustive-discriminant-cast-hole
title: "Probe whether #[non_exhaustive] constrains a cross-crate discriminant cast"
status: done
priority: p2
dependencies: []
related: [reconcile-adr-records-with-the-widened-numerical-vocabulary, preserve-non-exhaustive-visibility-probe]
scopes: [research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [research, api-hardening, identity]
---
ADR 0074 convention 5b now carries an **Inference** that `#[non_exhaustive]` does not constrain an `as` cast, so marking a vocabulary would not have prevented the four discriminant casts convention 3 now forbids. Every neighbouring claim in that record is a **Measurement** with a retained `trybuild` fixture; this one is not, and the asymmetry is the gap.

**Why it matters rather than being pedantry.** The inference is what makes convention 3 rather than convention 5 the home of the discriminant rule. If the attribute did constrain casting, 5b would already cover the site and convention 3 would not have needed amending. The record's reasoning therefore rests on it.

**What exists to extend.** `spikes/extensions/non-exhaustive-visibility/` already holds the harness that produced the `E0004` and `non_exhaustive_omitted_patterns` diagnostics ADR 0074 cites, with `consuming/tests/ui/fail/*.stderr` as retained expectations and a recorded environment row under `results/`. `scripts/check_rust.py` compiles it as a gated spike workspace, and `spikes/extensions/run.py --self-test` checks the retained diagnostics against the record.

**What closes this.** Add the case in the shape the existing fixtures use, on the toolchain `rust-toolchain.toml` pins. Note that a *passing* compile has no `.stderr` to retain, so decide how the harness records a positive result — the existing cases are all compile-failures, and a case whose evidence is "this compiled" needs a different retention form than a golden diagnostic, or it needs to be paired with a failing companion (for example, the same cast written as a wildcard-free `match`, which must fail with `E0004`) so that the pair is what the fixture pins. That pairing is probably the right answer because it pins the *contrast* the inference asserts, but decide it rather than assuming.

Then replace the Inference in ADR 0074 convention 5b with a Measurement citing the fixture, and remove the sentence naming this ticket. Do not change `decision_status`.

**Unverified reference measurement, recorded so it can be refuted rather than repeated.** A two-crate reproduction outside this repository, on an ambient `rustc 1.97.0` and not the repository pin, compiled `pub fn tag(value: Growing) -> u8 { value as u8 }` against a `#[non_exhaustive] pub enum Growing { A, B }` in a separate crate with exit 0. That is consistent with the inference and establishes nothing about the pinned toolchain; it is not evidence this ticket may cite.

## Outcome

**Measurement — the inference is confirmed, on the pin.** Environment: `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, commit `eff8269f797067c30555e77f160ec84c0ed15cd9`; `cargo 1.99.0-nightly (3efb1f477 2026-07-17)`; `aarch64-apple-darwin`; macOS 27.0 (26A5388g). Verified byte-identical to the toolchain row already recorded in `spikes/extensions/non-exhaustive-visibility/results/2026-07-24-macos-arm64.json` before the run, so the new claims extend that record rather than opening a second one. Procedure: `uv run --locked python spikes/extensions/run.py --suite non-exhaustive-visibility`, which is what the repository gate reaches through `scripts/check_rust.py`'s gated-spike compilation and `scripts/tests/test_research_harnesses.py`'s `--self-test`.

`#[non_exhaustive]` does not constrain a cross-crate `as` cast. Four fixtures, recorded as two contrast pairs, because a compiling fixture on its own states only that the compiler did not object and not why:

- `pass/cross_crate_discriminant_cast.rs` writes `value as u8` on the marked enum from the consuming crate and compiles. Its companion `fail/cross_crate_discriminant_tag_match.rs` is the same total map written as the convention-3 match, deriving the same tag for the same variant across the same boundary, and fails with `E0004`, "non-exhaustive patterns: `_` not covered". The recorded claim forbids the sibling `cross_crate_total_map` case's `` `&_` not covered ``, since the two share `E0004`.
- `pass/cast_ignores_denied_omitted_patterns.rs` enables `non_exhaustive_omitted_patterns_lint` and denies the lint crate-wide, and the cast still compiles — so the escape hatch ADR 0074 records as the alternative to dropping the attribute does not close the hole either. Its companion `fail/omitted_patterns_denied_at_crate_level.rs` is the control that makes that a measurement: it denies the lint at the same crate granularity and contains both constructs, and its retained diagnostic reports the omitting match, names `#![deny(non_exhaustive_omitted_patterns)]` as the level that fired, and forbids the cast's own source text `value as u8` from appearing. Without it, the compiling sibling's silence had a second explanation — a crate-level lint level the compiler accepts and never consults — that the pre-existing `omitted_patterns_denied.rs` rules out only for a `#[deny]` on the match itself, a granularity no cast site has.

**Decision on retention form, which the ticket left open.** The pairing is what is retained, as the ticket suspected. A positive result gets no separate retention mechanism: the record's `outcome: "compiles"` claim already requires the fixture to exist and to retain no `.stderr`, and the paired compile-failure is what pins the contrast the inference asserts. Adding a new retention form for "this compiled" would have restated on the side what the gate's compilation already re-derives.

**Harness change.** `run.py`'s `require_output` list of compile-fail cases was hand-maintained and named three of the then-three cases. It now derives every case from disk via `trybuild_case_names`, because the list decayed in the one direction that matters — a case added and not listed is never asserted and nothing reports it. The assertion is now over all eleven cases, six passing and five failing. This is not circular: the record already requires the fixture set on disk to equal the recorded set in both directions, so the derived list asserts that the *recorded* set reached the compiler. `scripts/check_rust.py`'s `verify_fixture_coverage` needed no change — it already globs `tests/ui/*/*.rs` — so no `implementation/workspace` edit was required.

**Measurement boundary.** These are bounded facts about one compiler. `#[non_exhaustive]` not interacting with `as` is not a stability guarantee, and a later compiler could add a diagnostic there; the record's fail-closed channel comparison forces a fresh run at the next pin migration. The same-crate cast is deliberately unmeasured: the attribute is defined to be inert inside its defining crate, nothing in ADR 0074 rests on that cell, and convention 3 forbids reading a discriminant on either side of the boundary regardless.

**Split out — the ADR edit did not land here.** This ticket declares `research/extensions`; ADR 0074 is `contracts/decisions`, held by a concurrent in-progress claim during this wave. Replacing convention 5b's **Inference** with a **Measurement** citing these fixtures, and removing the sentence naming this ticket, is owned by `record-the-cast-hole-measurement-in-adr-0074`, which carries the exact text. Nothing about the measurement is pending; only its transcription into the decision record is.

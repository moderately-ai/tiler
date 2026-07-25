---
id: record-the-cast-hole-measurement-in-adr-0074
title: Record the cast-hole measurement in ADR 0074
status: todo
priority: p2
dependencies: [probe-the-non-exhaustive-discriminant-cast-hole]
related: [reconcile-adr-records-with-the-widened-numerical-vocabulary, preserve-non-exhaustive-visibility-probe]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [research, api-hardening, identity]
---
`probe-the-non-exhaustive-discriminant-cast-hole` measured the thing ADR 0074 currently carries as an **Inference** in two places, and the measurement is checked in. This ticket is only the transcription: nothing about the evidence is pending. It exists as its own ticket because the probe declared `research/extensions` and `docs/decisions/[0-9]*.md` is `contracts/decisions`, which a concurrent claim held during that wave.

**What was measured, so this can be transcribed without re-reading the spike.** On `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, commit `eff8269f797067c30555e77f160ec84c0ed15cd9`, `aarch64-apple-darwin`, macOS 27.0 (26A5388g) — the pin, verified byte-identical to the toolchain row already in `spikes/extensions/non-exhaustive-visibility/results/2026-07-24-macos-arm64.json`, which now carries the four new claims. Two contrast pairs under `spikes/extensions/non-exhaustive-visibility/consuming/tests/ui/`:

- `pass/cross_crate_discriminant_cast.rs` writes `value as u8` on the marked enum from outside its defining crate and compiles; `fail/cross_crate_discriminant_tag_match.rs` is the same total map written as the convention-3 match, deriving the same tag for the same variant across the same boundary, and fails `E0004`.
- `pass/cast_ignores_denied_omitted_patterns.rs` denies `non_exhaustive_omitted_patterns` crate-wide under its feature gate and still compiles the cast; `fail/omitted_patterns_denied_at_crate_level.rs` is the control, denying the lint at that same crate granularity and containing both constructs, whose retained diagnostic reports the omitting match, names `#![deny(non_exhaustive_omitted_patterns)]` as the level that fired, and forbids the cast's own source text from appearing.

**Edit 1 — convention 5b.** Replace the paragraph beginning "**Inference — at such a site the attribute is inert, so 5b's tie-break against convention 3 does not arise.**" with:

> **Measurement — at such a site the attribute is inert, so 5b's tie-break against convention 3 does not arise.** `#[non_exhaustive]` constrains construction and pattern matching; it does not constrain `as`. `spikes/extensions/non-exhaustive-visibility/consuming/tests/ui/pass/cross_crate_discriminant_cast.rs` writes `value as u8` on a marked enum from outside its defining crate and compiles, while `fail/cross_crate_discriminant_tag_match.rs` — the same total map written as the convention-3 match, deriving the same tag for the same variant across the same boundary — fails `E0004`. The pair is the evidence, not either half: a cross-crate cast compiles cleanly, the wildcard the attribute is otherwise famous for forcing never appears, and marking the enum would not have prevented the defect recorded under convention 3 above. The compile error that closes an `as` cast is the one convention 3 now requires — replace the cast with a match — and it is available whatever the attribute says. A second pair measures the escape hatch recorded below: `pass/cast_ignores_denied_omitted_patterns.rs` denies `non_exhaustive_omitted_patterns` crate-wide under its feature gate and still compiles the cast, and `fail/omitted_patterns_denied_at_crate_level.rs` is the control showing that same crate-level level firing on a match and reporting nothing about the cast beside it, so a stricter lint level does not close the hole either. Bounded to the recorded compiler; the record's fail-closed channel comparison forces a fresh run at the next pin migration.

**Edit 2 — the convention 3 discussion.** Replace the paragraph beginning "**Inference — `#[non_exhaustive]` would not have closed it, which is why the rule belongs to convention 3 rather than to convention 5.**" with the same text up to "…and nothing else in convention 5 changes.", then replace its closing sentence — "This inference rests on the language rule that a cast is not a match together with the `E0004` measurement recorded above; it is not itself measured on the pinned toolchain, and `probe-the-non-exhaustive-discriminant-cast-hole` owns adding the retained fixture." — with:

> Measured on the pinned toolchain by the `cross_crate_discriminant_cast.rs` / `cross_crate_discriminant_tag_match.rs` pair cited under convention 5b above.

Change the paragraph's leading label from **Inference** to **Measurement** to match.

**Constraints.** Do not change `decision_status`. Both edits remove the last sentence naming `probe-the-non-exhaustive-discriminant-cast-hole` as owner, so check no other ADR or index still names it as open work — `grep -rn 'probe-the-non-exhaustive-discriminant-cast-hole' docs/` must return nothing after the edit. Run `uv run --locked python scripts/docs.py render` and `uv run --locked python scripts/check_repository.py`.

**Trigger for reconsideration if declined:** none — declining leaves an accepted decision resting on an inference whose measurement is already checked in beside it, which is the exact asymmetry the probe ticket was filed to remove.

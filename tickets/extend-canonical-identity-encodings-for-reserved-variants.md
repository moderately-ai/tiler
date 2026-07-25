---
id: extend-canonical-identity-encodings-for-reserved-variants
title: Make canonical identity encodings fail closed when reserved enum variants grow
status: done
priority: p1
dependencies: [resolve-non-exhaustive-recognizer-hole]
related: [prototype-scheduled-region-ir, prototype-fusion-legality-and-numerical-proof, harden-public-enums-non-exhaustive]
scopes: [implementation/ir, implementation/compiler, implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, identity, correctness]
---
Two canonical identity encodings currently discard information that is
unobservable only because the enum in question has exactly one inhabited variant
today. When the bounded profile widens — which the operation-family breadth work
will do — each becomes a silent identity **collision**: two structurally distinct
subjects would share identity bytes, and identity is what dedup, caching, and
receipt verification rest on. Silent wrongness, not a rejection.

Both sites were found during review and are documented in code comments, but a
comment does not fail closed:

- `crates/tiler-ir/src/schedule/model.rs` — `push_numerical` encodes the profile
  key, canonical NaN bits, and the two derived permission booleans, but does
  **not** encode `input_subnormals` / `result_subnormals`. `SubnormalMode` has
  only `Preserve` today, so no information is lost yet. Add a second
  `SubnormalMode` variant (for example flush-to-zero) and two regions differing
  only in subnormal handling collide on `CanonicalScheduledRegionIdentity`.
- `crates/tiler-compiler/src/fusion_legality.rs` — `effect_tag` maps
  `OperationEffect::Pure` to `1` and every other effect to `u8::MAX` via a
  wildcard arm. Two distinct non-pure effects would therefore share a tag inside
  `FusionLegalityProof`'s occurrence identity.

## Correction 2026-07-24: the `effect_tag` half was unachievable as first written

This ticket originally said to replace both wildcards with exhaustive matches.
That is correct for the schedule encoder but **was impossible for `effect_tag`**,
and the reason is a genuine contradiction between two accepted conventions rather
than an oversight here.

`tiler_ir::semantic::OperationEffect` is **already `#[non_exhaustive]`**
(`crates/tiler-ir/src/semantic/operation.rs`), and `effect_tag` encodes it from
`tiler-compiler` — a *different* crate. `#[non_exhaustive]` forbids a wildcard-free
match across a crate boundary, so ADR 0074's convention 3 (encoders match
exhaustively so growth is a compile error) and its convention 5 (mark growing
enums `#[non_exhaustive]`) have been in direct contradiction at that exact site
since both were accepted. No amount of care in this ticket could satisfy both.

Resolution: `resolve-non-exhaustive-recognizer-hole` amends convention 5 so that
an enum an out-of-crate consumer maps **totally** — which is precisely what an
identity encoder does — must not carry the attribute, with convention 3 winning.
So the fix for `effect_tag` is **two-step**: first remove `#[non_exhaustive]` from
`OperationEffect` under the amended rule, then make the match exhaustive and give
each effect its own tag. Doing the second without the first will not compile.

The schedule encoder is unaffected: its enums are encoded from within their own
defining crate, where the attribute has no effect. That same-crate exemption was
measured, not assumed — a same-crate exhaustive match over a `#[non_exhaustive]`
enum compiles, while the cross-crate form fails `E0004`.

Make both fail closed structurally rather than by convention: replace the
omission and the wildcard with **exhaustive** matches over the enums, so adding a
variant is a compile error at the encoding site instead of a silent collision.
Encode the subnormal modes into the scheduled-region identity, and give each
`OperationEffect` its own distinct tag. Both are identity-changing edits, so
rebaseline the affected identity fixtures deliberately and state in the Outcome
that the change is an intentional identity re-baseline, not a drift.

Add a regression test per site proving two subjects that differ only in the
previously-discarded facet now have distinct identity bytes. Because the second
variants do not exist yet, the test may need a temporary local enum or a
documented equivalent; if no honest test is possible before the variants land,
say so explicitly and rely on the exhaustive match as the structural guard rather
than claiming coverage that does not exist.

While in these encoders, also settle two tag-form deviations that proposed ADR
0074 recorded as Fact while verifying the encoding convention. Every domain tag
in the workspace is the NUL-terminated form `b"tiler.<subject>.v<N>\0"` except
`b"tiler.schedule.v1"` in `schedule/model.rs`'s `encode_identity`, and
`push_numerical` writes `profile_key` NUL-terminated rather than length-prefixed
like every other variable-length run. **Neither is ambiguous today** — the tag is
a fixed constant followed by fixed-width fields, and `profile_key` is a
crate-chosen `&'static str` — so this is uniformity, not a defect: adopting the
one form means the "is this ambiguous?" reasoning does not have to be redone at
each site. Both edits change identity bytes, so fold them into this ticket's
deliberate re-baseline rather than making a second one.

Related: `harden-public-enums-non-exhaustive` covers the separate API-stability
concern (marking these same enums `#[non_exhaustive]`); this ticket covers
identity completeness. They touch the same types and are best sequenced together.
`disambiguate-presentation-label-from-semantic-key-accessors` owns the adjacent
naming hazard ADR 0074 also left open.

## Correction: the schedule-encoder half is closed

`widen-numerical-vocabulary-and-complete-identity` implemented ADR 0076 items 1 and 6 and closed the first bullet of this ticket. `crates/tiler-ir/src/schedule/model.rs`'s `push_numerical` now encodes both subnormal dimensions and both permissions, each through an exhaustive `match` (`push_subnormal`, `push_permission`) over enums that are deliberately not `#[non_exhaustive]`, and the derived `permits_*` booleans it used to encode are gone from the encoding. The variants that made the omission observable exist: `SubnormalMode::FlushToZero { zero_sign }` and `NumericalPermission::Permitted`. `crates/tiler-ir/src/schedule/builder.rs` carries the regression test this ticket asked for, over every dimension including the flushed zero's sign, plus a pinned 194-byte identity for the strict-`f32` fixture.

**What remains here.** The `fusion_legality::effect_tag` bullet is untouched and still two-step behind `resolve-non-exhaustive-recognizer-hole`. The two tag-form deviations also remain: `b"tiler.schedule.v1"` is still the one domain tag in the workspace that is not NUL-terminated, and `push_numerical` still writes `profile_key` NUL-terminated rather than length-prefixed. Both were deliberately left out of that change so it would not absorb this ticket's scope.

## Measurement 2026-07-25: the remaining `tiler-ir` half is not landable inside `implementation/ir`

An `implementation/ir` worker claimed this, implemented both remaining tag-form edits in `crates/tiler-ir/src/schedule/model.rs` — `b"tiler.schedule.v1"` to `b"tiler.schedule.v1\0"`, and `profile_key` from NUL-terminated to `crate::identity::push_slice` — measured the blast radius, and **reverted**. The ticket now records what it costs so the next worker does not rediscover it.

**Measurement (macOS arm64, pinned nightly, base `9608997`).** `cargo nextest run --workspace --no-fail-fast` goes from 769 passing to 764 passing and 5 failing:

- `tiler-ir schedule::builder::tests::the_strict_f32_region_has_its_recorded_canonical_identity` — the pinned scheduled-region identity. In scope.
- `tiler-metal tests::{pointwise,single_axis_reduction,multi_axis_reduction,fused_reduction}_matches_its_golden_source` — **out of scope.**

**Why the Metal goldens are not incidental.** The generated MSL embeds the identities in its own text: the entry point is named `tiler_kernel_<kernel identity digest>`, and the header comment carries both `kernel identity digest:` and `scheduled region identity digest:`. For the pointwise golden the failure is `5eb771d4f02610db`/`a82a4d1c67a8aa44` becoming `56c4136874313b48`/`8747500aa18bd2fb`. Re-baselining therefore means rewriting all four `crates/tiler-metal/goldens/*.metal` files, which is `implementation/metal`.

**Why that was refused rather than done.** This ticket declares `implementation/ir` and `implementation/compiler`, not `implementation/metal`. `tkt guard` shows six open tickets holding `implementation/metal` — `prototype-metal-bundle-assembly`, `prototype-metal-kir-lowering`, `prototype-metal-numerical-realization`, `declare-metal-numerical-honourability`, `compile-golden-msl-through-the-aot-driver-in-the-gate`, and `choose-one-owner-for-apple-target-vocabulary` — so re-baselining goldens underneath them would land an identity shift in the middle of live work in a scope this ticket never claimed.

**Consequence for scheduling.** Nothing that remains here is landable within `implementation/ir` alone. The `effect_tag` half is `tiler-compiler` and still two-step behind `resolve-non-exhaustive-recognizer-hole`; both tag-form edits ripple into `implementation/metal`. **Add `implementation/metal` to this ticket's scopes before dispatching it**, and dispatch it to a worker who holds all three, or sequence it after the open Metal tickets close. The first re-baseline (`widen-numerical-vocabulary-and-complete-identity`) declared `implementation/metal` for exactly this reason; this ticket does not, and that is the gap.

**One re-baseline has already happened**, so this ticket's remaining edits are a *second* deliberate re-baseline rather than the first. Recorded shifts, so the next one can be told apart from drift: scheduled-region identity for the strict-`f32` pointwise fixture went 192 -> 194 bytes (`sha256 d900fe4a…` -> `d221e1a3…`, pinned as exact hex in `builder.rs`), kernel identity 607 -> 612 bytes (`sha256 39804fc0…` -> `75181a5c…`), artifact-program identity 12833 -> 12866 bytes (`sha256 3a622133…` -> `271e9e35…`), and the four `crates/tiler-metal/goldens/*.metal` digests moved with them.

## Retraction 2026-07-25: the recorded golden digests are right and their *direction* is reversed

The measurement above reproduces exactly on base `6fae4f3` — same five tests, same digests — but it names the pointwise pair the wrong way round. It says the failure is "`5eb771d4f02610db`/`a82a4d1c67a8aa44` becoming `56c4136874313b48`/`8747500aa18bd2fb`". It is the reverse: `56c4136874313b48`/`8747500aa18bd2fb` is what `crates/tiler-metal/goldens/pointwise_scale_bias.metal` held at `6fae4f3` (checkable in one line: `grep -n "identity digest" crates/tiler-metal/goldens/pointwise_scale_bias.metal`), and `5eb771d4f02610db`/`a82a4d1c67a8aa44` is what the tag-form edits make the emitter produce.

The cause is worth recording because it is a reading error anyone repeats. `assert_golden(name, expected, actual)` calls `assert_eq!(actual, expected, …)`, so in the panic message **`left` is the freshly emitted source and `right` is the checked-in fixture** — the opposite of the "left is what you had, right is what you want" that a diff reads like. A worker who takes `left` as the old value inverts every recorded shift.

Nothing else in the measurement changes: the five failing tests, the four affected goldens, and the scope consequence are all confirmed.

## Outcome

Both remaining halves landed on base `6fae4f3`, and `implementation/metal` was added to this ticket's scopes first, which is what the measurement above said had to happen before it was dispatchable.

**The `effect_tag` half — and a second site the ticket never named.** `tiler_ir::semantic::OperationEffect` lost `#[non_exhaustive]` under ADR 0074's amended convention 5b, which is the first of the three resolutions convention 3's note records; its doc comment now states the clause and the asymmetry that decides it, so the absence is a stated decision rather than an omission a later worker restores. Both cross-crate encoders are now exhaustive with no wildcard arm.

The ticket named one of them. There are two: `crates/tiler-compiler/src/fusion_legality.rs::effect_tag` feeding `FusionLegalityContentIdentity`, and `crates/tiler-compiler/src/legality.rs::effect_tag` feeding `RefinementContentIdentity`. Both carried the identical `_ => u8::MAX` arm. Found by sweeping every `OperationEffect` reference in `crates/` rather than by trusting the ticket's enumeration — the exact check is `grep -rn "OperationEffect" crates/`, which also confirms the third encoder, `tiler_ir::semantic::registry.rs:2382`, was already exhaustive because it is same-crate and the attribute never bound it.

**Deliberately not required to agree.** The two `effect_tag`s are separate encoders over separate domains, and a future effect may legitimately take a different tag in each. No test asserts they agree, because that invariant does not exist and asserting it would invent one.

**The `effect_tag` half changes no bytes, which is checked and not assumed.** `Pure` encoded as `1` before and after; only the unreachable `u8::MAX` arm is gone. The workspace test run confirms it: the five failures below are exactly the tag-form half's, with none attributable to the effect encoders.

**The two tag-form deviations.** `crates/tiler-ir/src/schedule/model.rs` now writes `b"tiler.schedule.v1\0"` and length-prefixes `profile_key` through `crate::identity::push_slice`. Both were the workspace's only sites deviating from the one form ADR 0074 convention 3 states.

**Measurement — the second deliberate identity re-baseline, not drift** (macOS arm64, pinned nightly, base `6fae4f3`). `cargo nextest run --workspace --no-fail-fast` went 787 passing to 782 passing and 5 failing, then 787 passing after the re-baseline. The five are exactly the ones the earlier measurement predicted:

- `tiler-ir schedule::builder::tests::the_strict_f32_region_has_its_recorded_canonical_identity` — the pinned scheduled-region identity, **194 -> 202 bytes**. The eight bytes are attributable rather than opaque and `builder.rs`'s constant documentation now says where each goes: one for the domain tag's NUL terminator, seven for the 21-byte `tiler.test.strict-f32` moving from a one-byte terminator to an eight-byte length prefix.
- The four `crates/tiler-metal/goldens/*.metal` fixtures, each differing from its regenerated form in exactly three lines — the entry-point name, the kernel identity digest, and the scheduled region identity digest — and in nothing else. That was verified by diffing each regenerated file against its predecessor rather than by editing the digests by hand, so a change to any other line would have been visible.

Recorded shifts, so a third re-baseline can be told apart from drift: pointwise `56c4136874313b48`/`8747500aa18bd2fb` -> `5eb771d4f02610db`/`a82a4d1c67a8aa44`; single-axis reduction `00634cefd2e0d8df`/`30a8c423c1663849` -> `9691e534b2336fd8`/`0a5ecbb7a29e3eac`; multi-axis reduction `cc845f33d21e62b1`/`c5420224b5719911` -> `955495a109931ca6`/`1f3099a86985211a`; fused multiply-add reduction `b7b499964dd388f1`/`39820b13aedee425` -> `1898bf58a4d1c9be`/`3abbb999f847ae08`.

**No regression test exists for the `effect_tag` half, and this states that rather than claiming coverage.** The ticket asked for a test proving two subjects differing only in the previously-discarded facet now have distinct identity bytes. `OperationEffect` has exactly one variant, so no second subject is constructible and no honest test is possible before a second effect lands. The guard is structural: the match is exhaustive over an enum that no longer carries `#[non_exhaustive]`, so a second variant is a compile error at both encoding sites. That is a type-system reservation plus implemented support, and deliberately not a tested guarantee. The tag-form half *is* covered, by the pinned 202-byte identity and the four goldens.

`uv run --locked python scripts/check_repository.py` passes.

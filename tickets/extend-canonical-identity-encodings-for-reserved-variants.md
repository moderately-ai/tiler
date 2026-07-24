---
id: extend-canonical-identity-encodings-for-reserved-variants
title: Make canonical identity encodings fail closed when reserved enum variants grow
status: todo
priority: p1
dependencies: [resolve-non-exhaustive-recognizer-hole]
related: [prototype-scheduled-region-ir, prototype-fusion-legality-and-numerical-proof, harden-public-enums-non-exhaustive]
scopes: [implementation/ir, implementation/compiler]
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

**One re-baseline has already happened**, so this ticket's remaining edits are a *second* deliberate re-baseline rather than the first. Recorded shifts, so the next one can be told apart from drift: scheduled-region identity for the strict-`f32` pointwise fixture went 192 -> 194 bytes (`sha256 d900fe4a…` -> `d221e1a3…`, pinned as exact hex in `builder.rs`), kernel identity 607 -> 612 bytes (`sha256 39804fc0…` -> `75181a5c…`), artifact-program identity 12833 -> 12866 bytes (`sha256 3a622133…` -> `271e9e35…`), and the four `crates/tiler-metal/goldens/*.metal` digests moved with them.

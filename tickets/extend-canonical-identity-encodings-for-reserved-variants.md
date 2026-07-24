---
id: extend-canonical-identity-encodings-for-reserved-variants
title: Make canonical identity encodings fail closed when reserved enum variants grow
status: todo
priority: p1
dependencies: []
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

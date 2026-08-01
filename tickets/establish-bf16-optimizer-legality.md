---
id: establish-bf16-optimizer-legality
title: Establish BF16 optimizer legality rather than inheriting the F32 permissions
status: todo
priority: p2
dependencies: [admit-bf16-into-the-schedule-and-kernel-vocabulary]
related: [spike-bf16-through-the-second-dtype-seams, design-the-bf16-computation-and-accumulator-contract, widen-the-f16-operation-vocabulary-to-contraction-and-reassociation]
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, optimizer, fusion, legality]
---
## User-visible outcome

A BF16 region derives its own fusion and rewrite legality instead of failing closed or, worse, borrowing F32's. A rewrite that is meaning-preserving at binary32 and not at BF16 is refused with a named rule.

## Why legality does not transfer across widths

**Fact, at `ef3c051`.** Every legality authority in `crates/tiler-compiler` is F32-named and F32-keyed: `FusionNumericalCapabilities::governed` maps the six `*_f32_op` keys to roles, `fusion_legality.rs` uses `F32::resolved_type().canonical_encoding()` as its governed dtype byte string, the fusion proof rule is `strict-f32-proof`, and the rewrite identities are `ordered-reassociate-add-f32.v1` and `ordered-reassociate-multiply-f32.v1`.

**Fact.** The numerical permissions a rewrite consults — contraction, reassociation, permutation, signed zero — are per-arithmetic-type on the target profile, because `docs/numerical-semantics.md` keys honourability by `(subject, dimension)` for measured reasons.

**Inference.** Reassociation error grows with the gap between the exact result and the representable neighbours, and BF16 has an 8-bit significand against binary32's 24. A reassociation permitted under an F32 error budget is not the same permission at BF16, and the rewrite identity must say which dtype it was proved for or two different rewrites will share one identity.

**Measurement.** Finding 28 of the [Apple numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md) records a per-dtype difference in the *target's* contraction behaviour: under `safe` with `-ffp-contract=fast`, `f16` fuses and `bf16` does not. So even the target side does not agree across widths, and an inherited conclusion would be wrong in a measurable way.

## Implementation keys

- The legality authority becomes keyed by the region's arithmetic type rather than by the F32 constant, and a region whose dtype has no registered capability fails closed with a named rule rather than defaulting to the F32 one.
- BF16 rewrite-rule identities are distinct from the F32 ones. A rewrite proved at one width must not be citable at the other; the identity is what enforces that.
- The fusion proof's rule name stops being `strict-f32-proof` for a BF16 region, or the proof carries its dtype — either is acceptable, and the explain output must let a reader tell which dtype a proof was about.
- The element-width derivation in `component_cost.rs`, which today matches the four F32 contract keys and returns `4`, derives from the dtype instead. An unknown key must stay `Unknown` rather than falling back to four.
- No new rewrite is admitted by this ticket. It makes the existing ones dtype-honest.

## Required evidence

- A BF16 region derives legality and fuses, rather than failing closed for want of a capability.
- A BF16 rewrite carries an identity distinct from its F32 counterpart, asserted directly.
- A region whose dtype has no registered legality capability is refused by name, observed failing.
- A reassociation forbidden by the BF16 contract is refused on a BF16 region while the same rewrite is admitted on an F32 region under an F32 contract permitting it — the pair is what shows the keying works.
- The F32 rewrite identities and the F32 fusion proof are byte-identical, pinned by the existing explain goldens.
- `component_cost` returns 2 for a BF16 element and 4 for F32, and `Unknown` for an unregistered key.

## Closes when

BF16 regions derive their own legality, BF16 and F32 rewrites carry distinct identities, the fail-closed path is observed failing, F32 identities are unchanged, the element-width derivation is dtype-derived, and the `Optimizer legality` cell for BF16 moves.

## Graph maintenance

- Depends on the kernel and schedule vocabulary; there is no BF16 region to prove legality about otherwise.
- Deliberately not a dependency of `lower-bf16-to-metal` or the runtime child: a BF16 program that does not fuse is still correct, so the vertical can reach a device without this and this can land after it.
- Contraction and FMA permissions stay with `design-the-bf16-computation-and-accumulator-contract`. This ticket keys the *existing* permissions correctly; it does not decide new ones.
- The `Optimizer legality` cell for the U4/F32 family is `absent/unsupported` while its physical cells are tested, and that non-monotone shape is deliberate. Do not read this ticket as a template for closing that one.

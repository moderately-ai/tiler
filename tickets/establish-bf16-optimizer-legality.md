---
id: establish-bf16-optimizer-legality
title: Establish BF16 optimizer legality rather than inheriting the F32 permissions
status: done
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

## Outcome — delivered 2026-08-07 at `e6a34bb8`

**Legality is derived, not inherited, and no capability is `Measured`.** Every obligation is discharged as `Derived` with its basis named: referential transparency from `OperationEffect::Pure` on all three registrations; conversion-boundary preservation from the fact records resolving computation, accumulator, intermediate-materialization and result all to `tiler::bf16@1`, with mixed-precision and implicit-promotion refused by name at application time; contraction two ways, from the closed same-family proof extended *by key* and from the contract's own `Forbidden` guarantee, with MSL having no `bfloat` overload of `fma` so there is no fused primitive at this width to contract into; exceptional values from the declared NaN behaviour and canonical bits.

**Finding 28 was decided rather than deferred**, which is the reason this ticket existed. It measures that under `safe` with `-ffp-contract=fast`, `f16` fuses and `bf16` does not — a fact about **a target's compiler under a flag row**, which is the target profile's authority, keyed by subject and resolved before any region reaches this derivation. The `ArithmeticContraction` obligation asks the different question of whether *fusing* changes what the contract authorizes, and fusing introduces no multiply-plus-add adjacency the unfused form lacks. So it is not a counterexample — and it is precisely why the rows are derived rather than transferred from `f32`.

**The four reduction obligations are discharged vacuously and classed `SoundProof` rather than `NormativeGuarantee`.** The BF16 vocabulary is exactly three families — constant, multiply, add — with no reduction, no contraction-capable family and no coordinate relation. That is what made the legality establishable without a measurement, and it is what bounds the claim: the significand argument that would bite here has nothing to bite on. Labelling it as a sound proof over an empty population rather than a guarantee is the maturity ladder used correctly.

**BF16 reassociation stays `Unknown` and is explicitly withheld**, at the operation vocabulary rather than here.

**The wall was re-founded, not relaxed.** A multi-occurrence BF16 program now fuses into one selected region, with the trace asserted to name `tiler.contract.bf16.v1` and **not** the f32 contract. Beside it, a contraction-permitting BF16 contract on a profile declaring `Permitted` still refuses `NoFeasiblePlan` with `unrealized-contraction` — the same wall an `f32` region meets under a relaxed contract, which is the point: BF16 legality is now decided by the same authority under the same rules.

**Seven perturbations watched failing and restored**, including one that restores the *exact prior wall* and one that makes the new refusal disappear — so both directions are pinned.

**No pin moved.** The fusion-legality proof and the explain trace are compilation-local, so neither the rows nor the provider rename reaches artifact identity. `GOVERNED_PROVIDER_REVISION` was deliberately not bumped, on the precedent that the softmax, concatenate and contraction rows were added without stepping it — and the first perturbation confirmed the widening moves no `f32` answer.

**No public surface.** One observable-output change worth noting: rendered explain traces now attribute fusion legality to `tiler.fusion-numerical-capabilities@1` rather than `tiler.fusion-strict-f32@1`, because a provider called `strict-f32` attributing a BF16 region's proof states the opposite of what happened — which is the defect this ticket's problem statement named.

**Two required-evidence items were not satisfiable as written, and the worker said so rather than forcing them.** A BF16 rewrite carrying a distinct identity, and a BF16 reassociation refused where its f32 counterpart is admitted, both presuppose a BF16 rewrite and a BF16 fold — neither exists, and the ticket's own Implementation keys forbid admitting a new rewrite. The rules already bind `add_f32_op`/`multiply_f32_op` **by key**, so they are dtype-honest and can never match a BF16 key. What was substituted is the analogous keyed pair on the obligation that does exist. That is the right resolution of a ticket contradicting itself.

**Released:** [`move-the-bf16-optimizer-legality-ledger-cell`](move-the-bf16-optimizer-legality-ledger-cell.md) and [`correct-the-recognizer-era-sentences-in-the-optimizer-contract`](correct-the-recognizer-era-sentences-in-the-optimizer-contract.md) — the second being another ticket's debt the worker declined to absorb silently.

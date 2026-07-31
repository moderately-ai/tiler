---
id: admit-a-reassociating-contract-without-contraction
title: Admit a reassociating contract without contraction
status: in-progress
priority: p1
dependencies: []
related: [enumerate-the-split-reduction-on-the-planning-frontier, calibrate-and-activate-parallel-reduction-selection]
scopes: [implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, numerics, reductions]
claimed_from: todo
assignee: loop-admit-a-reas
lease_expires_at: 1785533981
---
## User-visible outcome

A compilation whose caller wants reassociation — and only reassociation — can state it, so a legal multi-member region survives fusion legality and a reduction split becomes selectable end to end.

## Why this is its own ticket

**Fact.** `enumerate-the-split-reduction-on-the-planning-frontier` landed the split as an enumerated frontier alternative and a verified three-stage program, and found it unreachable through `compile`. A split consumes reassociation. The only registered contract permitting reassociation is `StrictF32NumericalContract::governed_relaxed`, which also permits contraction, reciprocal replacement, and approximate intrinsics. For the recognized serial-sum program — whose members mix `tiler.multiply-f32` and `tiler.add-f32` — `derive_fusion_legality` cannot discharge `FusionObligation::ArithmeticContraction` and returns `Unknown("unrealized-contraction")` for every multi-member candidate, so every cover containing one is dropped and `select_physical_plans` retains nothing.

**Fact.** This is pre-existing and already pinned: `crates/tiler-compiler/src/fusion_legality.rs` `a_relaxed_mixed_arithmetic_region_still_needs_contraction_evidence` asserts exactly this outcome for the whole-program candidate.

**Inference.** The consequence is broader than reductions: under the relaxed contract the bounded profile can compile a pointwise chain (`relaxed_reassociation_reaches_verified_global_physical_selection` does) but not the recognized serial-sum program at all. Every reassociation-consuming strategy is therefore enumerable and unreachable, which is why `calibrate-and-activate-parallel-reduction-selection` has nothing to measure.

## Implementation keys

Two candidate resolutions survive an obvious elimination and the choice between them is the work, not a detail: register a contract preset that permits reassociation while forbidding contraction (a new versioned key, a new request-subject identity, and a decision about what else it resolves), or discharge `ArithmeticContraction` for a mixed multiply/add region by proving the emitted realization performs no contraction (the `is_exact_governed_same_family_pointwise` `SoundProof` widened to a case it currently cannot state). The second removes an unknown rather than adding a knob and is the stronger outcome if the proof is actually available; the first is reachable without one. Decide with evidence, record the elimination, and do not add a preset whose only justification is making a test compile.

Whichever lands, the closing evidence is the same: the recognized serial-sum program compiles under a reassociation-permitting contract, the split alternative reaches the retained portfolio beside the serial one, and the selected plan is still the serial one because the structural cost model prices the split higher — preference remains `calibrate-and-activate-parallel-reduction-selection`'s.

## Closes when

A registered contract admits a legal multi-member region for the recognized serial-sum program while permitting reassociation; `compile` retains the three-stage split alternative beside the two-stage serial one under it; the existing contraction-evidence test either still holds or is superseded with its reasoning recorded; every new check is perturbation-proved; and targeted tests plus the batch gate pass.

## Graph maintenance

- Unblocks the end-to-end half of `calibrate-and-activate-parallel-reduction-selection`, which cannot measure a plan no compilation selects.
- Contract-preset changes move the canonical request subject and therefore every artifact identity derived from it; recompute pinned digests on the merged tree rather than taking either side's.
- If the resolution is a new preset, `StrictF32NumericalContract::governed_profile` grows and `crate::policy::NumericalPolicyPreset` must state why the new resolution is a different meaning rather than a relaxation.

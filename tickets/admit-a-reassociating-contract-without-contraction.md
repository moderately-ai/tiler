---
id: admit-a-reassociating-contract-without-contraction
title: Admit a reassociating contract without contraction
status: done
priority: p1
dependencies: []
related: [enumerate-the-split-reduction-on-the-planning-frontier, calibrate-and-activate-parallel-reduction-selection]
scopes: [implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [compiler, numerics, reductions]
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

## Outcome

**Resolution A, and B was eliminated rather than deferred.** A fourth registered preset, `NumericalPolicyPreset::PermitReassociation` — `reassociation: Permitted`, every other dimension at the strict resolution, key `tiler.reassociate-f32.v1`, public spelling `NumericalContract::ReassociateF32`.

**Fact — B founders on a claim that is false, not merely unproved.** `crates/tiler-metal/src/emit.rs::realization_requirements` inserts `MetalNumericalRequirement::NoFloatingPointContraction` only in the `NumericalPermission::Forbidden` arm, and says why in its own doc comment: "a granted freedom is tolerated under every selection". Under a contraction-permitting realization the emitted artifact therefore carries no `-ffp-contract=off` obligation for `crates/tiler-build/src/metal_assembly.rs` or `crates/tiler-metal/src/golden_compilation.rs` to check, and finding 6 of the Apple numerical record measures the written multiply/add pair returning the fused `0x3fc58f9d` at `-ffp-contract=fast` against the separately rounded `0x3fc58f9e` at `off`/`on`. A `SoundProof` stating "the emitted realization performs no contraction" would be refuted by measurement.

**Fact — B also founders on phase and dependency direction, independently.** `derive_fusion_legality` is handed `(program, budgets, contract, capabilities, formation, candidate)`. None of them names a target profile, a scheduled region, a scalar program, or an emitter: the realization is not chosen until `physical.rs` builds the `ScheduledRegion`, and the emitter is a per-target crate the compiler core is contractually independent of. A machine-checked proof about emission would have to import a backend into legality.

**Fact — B would have moved the failure rather than removed it.** `crates/tiler-ir/src/schedule/builder.rs` requires `!contraction` on both `ScalarProgram::FusedMultiplyAddSerialSum` arms, so a contraction-permitting contract already makes the fused single-dispatch strategy fail the schedule verifier. `physical.rs:469` derives that flag from `request.numerical_contract().contraction`.

**Inference — B would not have delivered this ticket's stated user-visible outcome.** "A compilation whose caller wants reassociation — and only reassociation — can state it." Under B the sole reassociating contract would still be `governed_relaxed`, which also permits contraction, reciprocal replacement, and `BackendElementary` approximate intrinsics; ADR 0015 makes contraction independent of reassociation, so pricing one at the cost of the other is exactly what the elimination refuses.

**Fact — the existing contraction-evidence test still holds and is not superseded.** `a_relaxed_mixed_arithmetic_region_still_needs_contraction_evidence` is a statement about `governed_relaxed` and is unchanged. `a_reassociating_contract_discharges_the_mixed_region_by_forbidding_contraction` is added beside it.

**Fact — closing evidence, measured on this branch.** `the_reassociating_contract_reaches_the_split_through_compile` in `crates/tiler-compiler/src/pipeline/tests.rs`: the recognized serial-sum program over `[1, 4]` compiles under the new contract; the retained stage counts are `[2, 3]` — the two-stage materialized plan and the three-stage split beside it; the selected alternative is the two-stage one, because the structural cost model prices two dispatches and a staged partial tensor above one dispatch and no temporary. Perturbation: the same program under the strict contract retains `[1, 2]` and no split at all.

**Fact — a second, separate limit found and recorded rather than absorbed.** The *whole-program* fused candidate is still `Unknown` under any reassociation-permitting contract, on `FusionObligation::ReductionReassociation` with reason `unproven-reassociation`: `push_reduction_obligations` fails closed for any region containing a reduction when reassociation is permitted. So the reassociating portfolio holds the materialized and split plans and not the fused one. That is a different obligation from this ticket's and is left as it stands.

**Fact — the identity movement is narrower than this ticket predicted, and the check is one line.** `VerifiedRequestSubject::canonical_explain_subject_bytes` writes the resolved contract, the stated preference list, and every budget; it does **not** write `MAX_NUMERICAL_CONTRACT_PREFERENCES` or the preset count. So no existing request subject moves: a request under the new preset is a *new* subject (new key, new reassociation byte), and every artifact identity derived from it is new rather than moved. Reproduce by reading that function — no pinned digest in the corpus encodes a preset count, and the 494-test `-p tiler-compiler` run needed no rebaselining.

## Graph maintenance

- Unblocks the end-to-end half of `calibrate-and-activate-parallel-reduction-selection`, which cannot measure a plan no compilation selects.
- Contract-preset changes move the canonical request subject and therefore every artifact identity derived from it; recompute pinned digests on the merged tree rather than taking either side's. **Corrected by the Outcome above:** registering a preset moves no existing subject, because the preset count is not encoded; only a request that *states* the new contract has a new subject. The instruction still stands for any change to an existing preset's dimensions.
- If the resolution is a new preset, `StrictF32NumericalContract::governed_profile` grows and `crate::policy::NumericalPolicyPreset` must state why the new resolution is a different meaning rather than a relaxation. **Done.** `governed_profile` now returns `[Self; NumericalPolicyPreset::ALL.len()]`, so a preset registered in the table and omitted from the admitted set is a build error.
- **Public boundary, not self-accepted.** `NumericalContract::ReassociateF32` (a `#[non_exhaustive]` public enum, so additive) and `MAX_NUMERICAL_CONTRACT_PREFERENCES` moving `3 → 4` are both session-visible and go to Tom under ADR 0075 before acceptance.
- Filed: [`correct-the-optimizer-contract-registered-preset-count`](correct-the-optimizer-contract-registered-preset-count.md) — `docs/compiler/optimizer.md` still names three registered contracts, and `contracts/optimizer` was outside this ticket's scopes.

**Boundary acceptance (2026-07-31).** Tom accepted the public boundary as reviewed: `NumericalContract::ReassociateF32` with key `tiler.reassociate-f32.v1`, and the table-derived `MAX_NUMERICAL_CONTRACT_PREFERENCES` moving 3 to 4.

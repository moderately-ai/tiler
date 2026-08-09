---
id: record-the-contraction-choice-a-fused-fold-actually-made
title: Record the contraction choice a fused fold actually made
status: done
priority: p3
dependencies: [implement-the-realization-witness-vocabulary]
related: [enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle, measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order]
scopes: [implementation/ir, implementation/compiler, research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, scheduling, conformance]
---
## User-visible outcome

The two plan-side contraction sites either record which choice the emitted body made, or say in a durable place that they cannot — so a contraction-permitting contract stops being a freedom no plan reports.

## Why this exists

**Fact — one site carries a field that is a mirror rather than a witness.** `ScalarProgram::FusedMultiplyAddSerialSum.contraction` is set from the contract under the source anchor `contraction: request.numerical_contract().contraction`, so it answers "was I allowed to fuse", never "did I fuse". Kernel lowering destructures the variant while ignoring that field. The later witness audit found a stronger current boundary: the intrinsic schedule verifier admits this scalar program only when the field is `false`, so the verified `true` population is empty and the field does not reliably carry even the contract resolution.

**Fact — the other site has no field at all.** `ScalarProgram::StrictTensorContraction` carries exactly `contracted_shape`, `order`, and `canonical_nan_bits`. Its per-contributor step is `accumulator + a * b`, and the `TENSOR_CONTRACTION` policy row lists `Contraction` precisely because that adjacency is real.

**Inference — under a contraction-permitting contract these are the refutation the enumeration predicts.** Two plans agreeing on the whole field set can differ in whether the step fuses, and a fused step rounds once where a separate one rounds twice.

**Fact — the third contraction site is already filed elsewhere and is not this ticket.** Whether the *backend compiler* preserves the emitted order is `measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order`; the Metal realization requirements omit `NoFloatingPointContraction` exactly when the contract permits contraction.

## What this ticket must produce

For each of the two sites: either the field that records the emitted choice, with the lowering that reads it and the verifier check that keeps it honest; or the derivation that no such choice exists to record, landed where a reader looking at the field will find it. A field added is a schedule-identity change and states its identity consequence.

## Explicit non-goals

The backend-compiler measurement; admitting a contraction-permitting contract for any new operation; changing `ELEMENTARY_UNCARRIED_DIMENSIONS`.

## Closes when

Both sites are resolved either way, and the freedom-sites enumeration's rows 3.1 and 3.2 are corrected to match.

## Graph maintenance

Filed by the freedom-sites enumeration as an out-of-scope defect the enumeration revealed.

## Outcome audit — 2026-08-09

Delivered by the accepted realization-witness vocabulary; no new schedule field or identity change is owed. `RealizationWitness::unpinned_freedom_site` returns `ContractionUnrecorded` whenever a contraction-permitting realization reaches a fold adjacency whose plan records no choice. The exhaustive helper `unrecorded_fold_contraction` maps `FusedMultiplyAddSerialSum` to `UnrecordedFoldContraction::ScaleBiasContributor` and `StrictTensorContraction` to `UnrecordedFoldContraction::ContractedProduct`. `every_unrecorded_fold_contraction_is_named_by_its_adjacency` pins the complete four-program/three-adjacency population and the permission gate.

The freedom-sites record's source anchors `site 3.1's field is a mirror` and `site 3.2 has no field at all`, together with its dated correction 3, now state the derivation in the durable place this ticket required. The backend-compiler choice remains the separate third site. Both plan-side sites are therefore resolved by an explicit typed “unrecorded” outcome rather than by inventing a choice, satisfying this ticket's allowed closure.

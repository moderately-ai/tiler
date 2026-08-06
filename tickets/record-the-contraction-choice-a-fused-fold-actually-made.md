---
id: record-the-contraction-choice-a-fused-fold-actually-made
title: Record the contraction choice a fused fold actually made
status: todo
priority: p3
dependencies: []
related: [enumerate-the-freedom-sites-a-physical-plan-must-pin-for-a-permissive-conformance-oracle, measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order]
scopes: [implementation/ir, implementation/compiler, research/reference]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, scheduling, conformance]
---
## User-visible outcome

The two plan-side contraction sites either record which choice the emitted body made, or say in a durable place that they cannot — so a contraction-permitting contract stops being a freedom no plan reports.

## Why this exists

**Fact — one site carries a field that is a mirror rather than a witness.** `ScalarProgram::FusedMultiplyAddSerialSum.contraction` is set from the contract at `crates/tiler-compiler/src/physical.rs:1415-1416` (`request.numerical_contract().contraction != NumericalPermission::Forbidden`), so it answers "was I allowed to fuse", never "did I fuse". Two plans that disagree about whether the emitted body fused carry the same value. **And nothing reads it**: the variant is destructured at `crates/tiler-ir/src/kernel/lower.rs:880-884` and `:1457-1460`, and both bindings take `scale_bits`, `bias_bits`, and `..`.

**Fact — the other site has no field at all.** `ScalarProgram::StrictTensorContraction` (`crates/tiler-ir/src/schedule/model.rs:565-573`) carries exactly `contracted_shape`, `order`, and `canonical_nan_bits`. Its per-contributor step is `accumulator + a * b`, and `TENSOR_CONTRACTION` (`crates/tiler-compiler/src/policy.rs:258-267`) lists `Contraction` precisely because that adjacency is real — the row's own comment calls it "the single point where the two senses of 'contraction' meet".

**Inference — under a contraction-permitting contract these are the refutation the enumeration predicts.** Two plans agreeing on the whole field set can differ in whether the step fuses, and a fused step rounds once where a separate one rounds twice.

**Fact — the third contraction site is already filed elsewhere and is not this ticket.** Whether the *backend compiler* preserves the emitted order is `measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order`; `crates/tiler-metal/src/emit.rs:1027-1032` drops `-ffp-contract=off` exactly when the contract permits.

## What this ticket must produce

For each of the two sites: either the field that records the emitted choice, with the lowering that reads it and the verifier check that keeps it honest; or the derivation that no such choice exists to record, landed where a reader looking at the field will find it. A field added is a schedule-identity change and states its identity consequence.

## Explicit non-goals

The backend-compiler measurement; admitting a contraction-permitting contract for any new operation; changing `ELEMENTARY_UNCARRIED_DIMENSIONS`.

## Closes when

Both sites are resolved either way, and the freedom-sites enumeration's rows 3.1 and 3.2 are corrected to match.

## Graph maintenance

Filed by the freedom-sites enumeration as an out-of-scope defect the enumeration revealed.

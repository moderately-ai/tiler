---
id: admit-a-bf16-index-realization-law-and-refinement-contract
title: Admit a BF16 index-realization law and refinement contract
status: in-progress
priority: p1
dependencies: []
related: [carry-bf16-through-the-artifact-encoding-and-identity, conform-the-bf16-vertical-end-to-end, lower-bf16-to-metal, state-and-check-a-bf16-numerical-contract]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, indexing, identity]
claimed_from: todo
assignee: agent-bf16-lawful
lease_expires_at: 1785985712
---
## User-visible outcome

A pure-BF16 semantic program can be compiled into a verified kernel program and packaged into an artifact by a producer, rather than only hand-assembled at the artifact envelope. Today every BF16 layer below the program is implemented and tested and the composition is unreachable: a BF16 occurrence cannot obtain executable coverage, so no BF16 kernel program verifies, so no BF16 artifact can be built.

## Why the wall is here and not elsewhere

**Fact, at `55652b2b`.** `NumericalContractIdentity` (`crates/tiler-ir/src/index/refinement.rs`) is a newtype over `F32NumericalContractKey`. Its two constructors are `try_from_key`, which parses the `f32` grammar, and `From<F32NumericalContractKey>`. `Bf16NumericalContractKey` exists beside it under `tiler.contract.bf16.v1` and has no route into this type. `IndexRefinementSubject::derive` takes a `NumericalContractIdentity`, so a BF16 occurrence's refinement subject is not expressible.

**Fact.** `crates/tiler-ir/src/index/` contains no BF16 reference at all. The exact check is `grep -rln 'bf16\|Bf16' crates/tiler-ir/src/index/`, which returns nothing. There is therefore no BF16 scalar operation key a candidate index region could apply, where `constant_f32_scalar_op`, `multiply_f32_scalar_op`, and `add_f32_scalar_op` exist for the `f32` family.

**Fact.** The standard semantic provider registers index-realization laws for exactly nine operations (`crates/tiler-ir/src/semantic/registry.rs`, the `register_index_realization_law` loop): the four `f32` arithmetic and reduction families, SiLU, reindex, broadcast, strict tensor contraction, and strict-affine dequantization. None is BF16, and the comment beside the loop states the absence is deliberate and fails closed later.

**Inference.** The three compose into one wall. Without a contract identity a subject cannot be derived; without a registered law `FrozenIndexRealizationLawRegistry::resolve` cannot resolve one; without a scalar operation a candidate region cannot be built. So `CoveredOccurrence` is unobtainable for a BF16 operation, `KernelProgramBuilder::push_stage` accepts empty coverage but whole-program verification refuses the result as `KernelProgramDiagnostic::UncoveringStage`, and nothing above the program layer can be reached with a BF16 program in hand.

**Fact.** This is what `carry-bf16-through-the-artifact-encoding-and-identity` hit. That ticket closed on the evidence its own layer owns — the encoding carries the carrier losslessly, the carrier reaches artifact identity, and both width refusals are observed — by assembling its BF16 envelope directly rather than through a producer, and recorded the boundary rather than fabricating a semantically incoherent fixture.

## Implementation keys

- Decide how a BF16 contract reaches refinement identity. `NumericalContractIdentity` wrapping one arithmetic type's key is the current shape; the sibling-domain pattern `state-and-check-a-bf16-numerical-contract` established for the contract keys themselves is the obvious candidate, and the choice must keep every existing `f32` refinement receipt byte-identical, because a receipt's numerical contract is folded into stage coverage and therefore into kernel-program and artifact identity.
- Register the BF16 scalar operations and index-realization laws for `tiler::constant-bf16@1`, `tiler::multiply-bf16@1`, and `tiler::add-bf16@1`. The law is what makes the verifier a check rather than a rubber stamp, so it is derived independently of any candidate region a caller supplies.
- The registration adds rows to the semantic provider's law sidecar, which is folded into `FrozenIndexRealizationLawRegistry`'s identity. Establish whether that moves any retained identity before assuming it is appends-only, and enumerate every moved pin if it does.
- No new arithmetic behaviour is decided here. The BF16 family's semantics, its canonical NaN, and its numerical contract are all already accepted; this ticket makes them reachable through the refinement layer.

## Required evidence

- A pure-BF16 constant/multiply/add semantic program obtains verified coverage for every one of its occurrences, and a candidate region that does *not* realize its operation is refused — the rubber-stamp perturbation, observed failing.
- That program builds a `VerifiedKernelProgram` over a `PointwiseBf16` scheduled region and packages into a `VerifiedArtifactProgram` that encodes, decodes, and re-derives its identity.
- Every existing `f32` refinement receipt, stage key, kernel-program identity, and artifact identity is unchanged, or every moved pin is enumerated and recomputed on the tree the step lands into.
- A BF16 program under an `f32` contract identity, and an `f32` program under a BF16 one, are each refused by name rather than verified.

## Closes when

A pure-BF16 program reaches a verified artifact through the ordinary producer path, the rubber-stamp and cross-contract refusals are observed failing, and either no retained identity moved or every moved pin is enumerated.

## Graph maintenance

- Blocks `conform-the-bf16-vertical-end-to-end`, whose end-to-end run needs exactly this composition, and unblocks the producer half of `carry-bf16-through-the-artifact-encoding-and-identity`'s outcome. Related to `lower-bf16-to-metal`, which needs a BF16 kernel program to lower.
- `admit-bf16-into-the-schedule-and-kernel-vocabulary` and `state-and-check-a-bf16-numerical-contract` are both `done` and are the layers this one sits between; neither is reopened.
- Touching the law sidecar is an identity-domain risk. Survey the pin population before editing, as the artifact ticket's own report did.

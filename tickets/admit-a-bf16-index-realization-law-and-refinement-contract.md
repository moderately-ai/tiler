---
id: admit-a-bf16-index-realization-law-and-refinement-contract
title: Admit a BF16 index-realization law and refinement contract
status: done
priority: p1
dependencies: []
related: [carry-bf16-through-the-artifact-encoding-and-identity, conform-the-bf16-vertical-end-to-end, lower-bf16-to-metal, state-and-check-a-bf16-numerical-contract]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, indexing, identity]
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

## Outcome

**The wall is gone because the refinement layer now derives the width instead of assuming it.** All three of the ticket's Facts held at the current tree, and the fix is one design decision plus two registrations.

**Fact — how a BF16 contract reaches refinement identity.** `NumericalContractIdentity` is now an opaque newtype over a *private* `enum NumericalContractKey { F32(F32NumericalContractKey), Bf16(Bf16NumericalContractKey) }` (`crates/tiler-ir/src/index/refinement.rs`). The sum is private deliberately: every consumer discriminates through `arithmetic()`, whose `ArithmeticType` is not `#[non_exhaustive]`, so a third admitted width is a build error at each such site rather than a match written before it existed falling through. A public enum was the rejected alternative for exactly that reason. The public delta is additive only — one `From<Bf16NumericalContractKey>` impl, and `try_from_key` now dispatching on the rendered domain.

**Fact — no f32 receipt moved.** The identity bytes are the key spelling with no width tag: the two governed domains render mutually closed preimages, so the spelling already determines the width, and `push_slice(subject.numerical_contract.as_bytes())` produces byte-identical output for every `f32` contract. `try_from_key` routes everything not rendered under `tiler.contract.bf16.v1` to the `f32` parser unchanged, so no previously admitted key and no previously reported refusal moved.

**Fact — the law derives its arithmetic rather than declaring it.** `accepts_numerical_contract` now takes the `IndexRefinementSubject` and, for the seven general templates, requires the contract's `canonical_type_key()` to equal the subject's single result type's nominal key (`governs_result_arithmetic`, `crates/tiler-ir/src/index/law.rs`). Every one of those templates builds its output tensor from the subject's own result type, so the result type *is* the arithmetic the region will emit; declaring it on the law would be a second authority over one fact. The check is strictly tighter than the `f32`-only test it replaced — a non-nominal result type now names no arithmetic and is refused — and a non-single-result subject is left to the law's own arity rule so no existing diagnostic changed.

**Fact — registration.** Three `bf16` scalar operations (`tiler.scalar::constant-bf16@1`, `multiply-bf16@1`, `add-bf16@1`) in `crates/tiler-ir/src/index/scalar.rs`, and three law rows reusing the existing `ConstantFromFloatBits` and `PointwiseBinary` templates. **No law encoding tag was added**, so every existing row's payload is untouched. `realize_constant` now keys the scalar attribute record by the law's own `AttributeFieldId` rather than `F32_CONSTANT_BITS_ATTRIBUTE`; both spell field 1, so the bytes are identical, but the `bf16` path is now correct by derivation rather than by that coincidence. The `bf16` payload width is read from `registered_bf16_payload_bytes`, the same function the semantic constant uses, so the two cannot drift.

**Measurement — evidence.** `crates/tiler-ir/src/program/tests.rs`: a pure-BF16 constant/multiply/add program obtains verified coverage for all four occurrences and builds a `VerifiedKernelProgram` over a `PointwiseBf16` region; a candidate region realizing a *different* occurrence is refused `SemanticRealizationMismatch`; and both cross-width pairings are refused `NumericalContractNotGoverned`. Each refusal test carries a positive control — the honest candidate and the native contract each verify — so a refusal is caused by the perturbation rather than by a broken fixture.

**Measurement — the checks can say no.** Three deliberate perturbations, each watched failing and reverted. Neutering the contract gate to `true`: a BF16 program **verified under an `f32` contract**. Restoring the previous `f32`-only gate: three of the four new tests fail `NumericalContractNotGoverned`. Removing the three law rows: both reachability tests fail on the missing registered law.

**Measurement — pin survey.** `cargo nextest run --workspace` → `2707 tests run: 2706 passed, 1 failed, 7 skipped`, twice, with the identical failure. Exactly one pinned identity moved: `crates/tiler-compiler/src/explain.rs:4090`, `8e06e11fdc3a2889` → `b2d55d5a36e0159b`. The request subject binds `FrozenIndexRealizationLawRegistry::identity()`, which folds the scalar snapshot and the whole law sidecar, so both halves of this change land in that one field. The *semantic* snapshot did not move — it is computed over definitions and operations only, and the three `bf16` operation families were already registered.

**Fact — artifact identity did not move.** `tiler-build`'s `the_standard_metal_path_publishes_its_recorded_identities` passed unchanged, pinning the standard Metal artifact identity and cache subject by SHA-256. That is the empirical form of the design argument: `CoveredOccurrence` retains the *reached-only* executable-coverage identity, which restates no whole-registry snapshot, so growing the registries beneath an `f32` occurrence leaves its kernel-program and artifact identity byte-identical.

**Boundary — two halves of the required evidence are out of this ticket's scope.** The branch holds `implementation/ir` only.

- The moved pin is `implementation/compiler`. Filed as `recompute-the-explain-request-qualifier-for-the-bf16-realization-rows` with the new value, the derivation, and the reproduction command. **This branch leaves the workspace gate red at that one assertion and nowhere else.**
- `VerifiedArtifactProgram` lives in `tiler-artifact`, and `tiler-ir` declares no workspace crate dependencies, so no `tiler-ir` test can reach it. Filed as `carry-the-pure-bf16-producer-path-into-artifact-packaging-evidence`, which also carries the now-false justification comment on `bf16_input_envelope`.
- `docs/artifact-abi.md`'s producer-wall paragraph is falsified and is `contracts/artifacts`. Filed as `correct-the-artifact-abi-contracts-bf16-producer-wall-paragraph`.

The stale "nine laws" claim in `lower-the-concatenate-occurrence-through-partitioned-writes` was corrected in place under this ticket's shared `project/tickets` scope.

## Refinement-identity surface — accepted

Accepted by Tom on 2026-08-06 at the morning decision review, witnessed first-hand by the coordinator: the private width enum inside `NumericalContractIdentity`, the additive `From<Bf16NumericalContractKey>` route, the no-width-tag identity resting on the domains' mutual closure, and the subject-derived coverage gate. Acceptance is not stabilization.

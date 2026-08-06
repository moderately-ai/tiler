---
id: accept-the-multi-region-index-realization-surface
title: Accept the multi-region index realization surface
status: awaiting-decision
priority: p1
dependencies: [admit-a-multi-region-index-realization-law]
related: [lower-a-two-region-occurrence-through-one-index-access-capability, admit-the-rms-normalization-family, admit-the-softmax-family]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [decision, api, ir, indexing]
---
## What is being accepted

The public surface [`admit-a-multi-region-index-realization-law`](admit-a-multi-region-index-realization-law.md) landed as a **draft**, exactly as that ticket's own Decision boundary section required. It is implemented and tested; a tested implementation is a concrete draft, not implicit approval of its interface, so this node parks until Tom closes it.

## The exact surface

New in `tiler_ir::index`:

- **`VerifiedIndexRegionSequence`** — an ordered chain of verified regions. `single`, `try_new`, `stages` (an `Iterator`, not a slice), `leading_stages`, `stage`, `stage_count`, `is_single_stage`, `stage_sources`, `intermediates`, `final_stage` (a `const fn`), `identity`. `PartialEq`/`Eq` compare canonical identity only.
- **`StagedInputSource`** — `Occurrence(usize) | Intermediate(usize)`, naming where each region input boundary's value comes from. Deliberately *not* `#[non_exhaustive]`: a source a reader could not match exhaustively is one the identity encoder would have to guess at.
- **`StagedIntermediate`** — the checked record of one handed value: `producer`, `producer_output`, `consumer`, `consumer_input`, `value_type`, `shape`.
- **`CanonicalIndexRegionSequenceIdentity`** — opaque bytes, `as_bytes` only.
- **`IndexRegionSequenceError`** — `#[non_exhaustive]`, eight named chain refusals.
- **`MAX_INDEX_REGION_SEQUENCE_STAGES`** — `64`.
- **`IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32 { axes_attribute, scalar }`** and its constructor `staged_strict_serial_sum_then_multiply_f32`.
- **`ResolvedIndexRealization::verify_sequence`** — the general path; `verify` is its one-region spelling.
- **`IndexRefinementVerificationError::SemanticRealizationSequenceMismatch { expected, actual }`** — additive under the enum's existing `#[non_exhaustive]`.

Changed in `tiler_ir::index`:

- **`IndexRefinementReceipt`** gains `regions() -> Vec<..>`, `realization()`, `scalar_authorities() -> Vec<..>`. `region()` and `scalar_authority()` are retained and now answer *the final stage*, which for every one-stage realization is the only stage.
- **`PendingIndexRefinementReceipt`** gains `realization()`, `scalar_authorities()`, `staged_obligations()`. `region()` and `scalar_authority()` are retained, still `const fn`, and answer the final stage.
- **`OperandBinding`** gains `stage()`; **`IndexRefinementDomainProof`** gains `stage()`. A region-local handle needs the region it resolves against.
- **`MAX_INDEX_REFINEMENT_RESIDUAL_OBLIGATIONS`** moves from `3 * MAX_TENSOR_RANK * 2` to `6 * MAX_TENSOR_RANK * 2`, re-derived over the staged template's five rank-wide accesses.

## The choices worth objecting to

- **The intermediate is declared per input boundary and checked, not inferred.** Inference has no answer for a normalization, whose value and weight operands agree on element type and shape; the cost is that `try_new` takes a source list beside the regions.
- **A one-stage sequence's identity is its region's identity, byte for byte**, and staged receipts are written under separate domain tags. This is what makes the change identity-neutral for every existing law, at the cost of two encodings per identity rather than one uniform one.
- **`region()` and `scalar_authority()` retained rather than removed.** They are `const fn` and called from `const fn` in `tiler-compiler`, which this ticket's scope could not edit. The alternative — removing them — is a compiler-side change belonging to [`lower-a-two-region-occurrence-through-one-index-access-capability`](lower-a-two-region-occurrence-through-one-index-access-capability.md), and it is the honest question here: should a staged receipt expose a single-region accessor at all?
- **`stages()` returns an `Iterator` rather than a slice**, because the final stage is stored beside the earlier ones so that non-emptiness is a type invariant rather than an `expect` at every read.
- **One law form, admitted for a family nothing registers yet.** The form is real and tested; it is not the normalization's law, which needs a governed reciprocal square root that does not exist. See the deriving ticket's Outcome for that derivation.

## Evidence

The deriving ticket's Outcome section carries: the per-site injectivity reasoning for law tag 9 and for both receipt domain tags, the pin survey (nothing moved; `cargo nextest run --workspace` → 2757 passed before and after), the chain-well-formedness argument mirrored from `derive_subprogram_boundary_contract`, and the watched-failing perturbations — including two that showed an assertion which looked right and was not.

## Closes when

Tom accepts, accepts with a named exclusion, or rejects. Nothing releases on this node meanwhile; the surface is in use inside `tiler-ir` and labelled a draft.

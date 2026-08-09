---
id: accept-the-multi-region-index-realization-surface
title: Accept the multi-region index realization surface
status: done
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

## Delta — the compiler-side consumer, 2026-08-06

Appended rather than filed as a second node, because it is one small addition to the decision already open: the surface above has no consumer outside `tiler-ir` without it, and accepting the two separately would leave a ruling on a vocabulary with nothing that speaks it. [`lower-a-two-region-occurrence-through-one-index-access-capability`](lower-a-two-region-occurrence-through-one-index-access-capability.md) landed it as a draft; its Outcome carries the derivations and the watched-failing perturbations.

**New in `tiler_compiler::capability`** — every item here is already inside that module's stated draft boundary:

- **`IndexAccessLoweringProvider::lower_sequence`** — a *defaulted* method taking the realization context. The default opens one stage sourced positionally, so all twenty existing providers are untouched; `lower` stays required and unchanged. A staged provider overrides `lower_sequence` and implements `lower` as an explicit refusal, which mirrors `IndexRealizationLaw::realize` refusing for the same law rather than being a stub.
- **`IndexAccessSequenceContext`** — `occurrence`, `stage_count`, `stage(sources, build)`, `single_stage(build)`. Each stage gets its own canonical builder and is verified before the next opens.
- **`IndexAccessStageFailure`** — `#[non_exhaustive]`; `Emit { stage, source }`, `Build { stage, diagnostics }`, `Chain(IndexRegionSequenceError)`.

**Changed in `tiler_compiler::legality`:**

- `IndexRefinement` and `PendingIndexRefinement` gain `realization()` and `single_region() -> Option<&VerifiedIndexRegion>`; `region()` is **removed** from both. `PendingIndexRefinement` also gains `scalar_authorities()`.
- `RefinementContent::region_identity` becomes `realization_identity() -> &CanonicalIndexRegionSequenceIdentity`, beside new `stage_count()` and `scalar_authorities()`.
- `RefinementError::Emit(..)` becomes `Emit { stage, source }`, `Build` gains `stage`, and `Realization { source }` is added; `impl From<LoweringEmitError> for RefinementError` is removed because a stage ordinal cannot be recovered from the source alone.

**The choices worth objecting to here:**

- **A defaulted sequence method rather than one required realization method.** The unified alternative writes the same three-line adapter into twenty providers, and the host can generate it. The alternative of a second provider trait plus a second `register_*` was eliminated for forking `GovernedIndexAccess` and the resolution surface over a property of the emission, not of the registration.
- **`single_region()` returns `None` for a chain rather than the final stage.** This is the answer to the question paragraph five of "The choices worth objecting to" left open. A chain's final stage reads a value no occurrence operand carries, so the documented "feed each input boundary the operand tensor named by `operand_bindings`" does not compose for it; returning it would let an evaluator run a third of a realization and report the result as the occurrence's. Both misreads were live in the tree before this branch.
- **The IR-side half of that answer is a rename, not a removal, and is not taken here.** Retaining a final-stage accessor survives elimination — its writes are the occurrence's results and `bind_results` derives from it alone — but `region()` is the wrong name for it. That is `implementation/ir` and is filed at [`name-the-index-receipt-final-stage-accessors-for-what-they-return`](name-the-index-receipt-final-stage-accessors-for-what-they-return.md), deferred behind this node. The `const fn` constraint this node recorded no longer binds `region()`: after the compiler-side change nothing in `tiler-compiler` calls it (`grep -rn 'receipt\.region()' crates/tiler-compiler` returns nothing), and only `scalar_authority()` is still reached, from a `pub const fn`.
- **Two more compiler identity domains.** `tiler.compiler.index-refinement-content.staged.v1\0` and `tiler.compiler.index-refinement-occurrence.staged.v1\0`, carrying every leading stage's reached authority and admission. A one-stage binding keeps the existing tags and encodes exactly the bytes it always did, because a one-stage sequence identity is its region's identity and a one-stage realization retains no leading stage. Surveyed rather than argued: 26 distinct 16-hex and 6 distinct 64-hex pins over `crates/tiler-compiler`, identical on the base commit and the branch.

## Closes when

Tom accepts, accepts with a named exclusion, or rejects — the compiler-side delta above included, since it is part of the same surface. Nothing releases on this node meanwhile; the surface is in use inside `tiler-ir` and `tiler-compiler` and labelled a draft in both.

## Decided — accepted

Accepted by Tom on 2026-08-06 at the morning decision review in the coordination session, witnessed first-hand by the coordinator, with the evidence packet this node carries. Acceptance is not stabilization; the surface is accepted pre-alpha vocabulary.

## Current follow-on correction — 2026-08-09

The IR-side accessor rename filed in the compiler delta is complete. [`name-the-index-receipt-final-stage-accessors-for-what-they-return`](name-the-index-receipt-final-stage-accessors-for-what-they-return.md) renamed the four receipt readers to `final_stage()` and `final_scalar_authority()` without changing signatures, return values, identity bytes, or verification behaviour; the compiler's `single_region()` refusal remains the explicit route for consumers that cannot evaluate a chain. The independent multi-value handoff remains `deferred` on its own trigger and was not absorbed by this rename.

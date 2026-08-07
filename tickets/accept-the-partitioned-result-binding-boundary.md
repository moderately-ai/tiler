---
id: accept-the-partitioned-result-binding-boundary
title: Accept the partitioned result-binding boundary
status: awaiting-decision
priority: p1
dependencies: []
related: [bind-a-partitioned-output-through-index-refinement, accept-the-partitioned-write-ownership-proof-boundary]
scopes: [contracts/decisions]
shared_scopes: []
paths: []
tags: [public-boundary]
---
## The decision

**Only Tom closes this ticket**; it parks at `awaiting-decision` carrying the exact surface. [`bind-a-partitioned-output-through-index-refinement`](bind-a-partitioned-output-through-index-refinement.md) landed a public boundary as a labelled draft under ADR 0075, not self-accepted. **No public type, field, method, or signature changed** — what moved is the *contract* two public items state, which is a public boundary exactly as a signature is.

### What changed, exactly

- **`ResultBinding` is now one binding per output *root*, not one per semantic result.** The struct, its four fields, and its four accessors are byte-for-byte unchanged. A result whose output is written whole still has exactly one binding, which is every realization the closed law vocabulary produces today. A result whose output is *partitioned* — several roots, each total over its own declared partition under `WriteOwnershipProofView::PartitionMember`, the partitions jointly disjoint and covering — has one binding per member, all sharing `result()` and `output_tensor()` and each carrying its own `write_access()` and `written_value()`. A consumer that needs one answer per result groups by `ResultBinding::result`.
- **`ResultBinding::write_access` is no longer always the whole output's write.** It is total over the whole output when the result has one binding and total over that member's declared partition when it has several; the region's own `WriteOwnershipProofView` says which, and refinement admits no root carrying neither. The accessor doc previously read "the complete unique write".
- **`IndexRefinementVerificationError::ResultArity::region_outputs` counts distinct output *tensors*, not output roots.** Its value for every region that ever produced it is unchanged, because a region with one root per output tensor counts the same either way. `crates/tiler-compiler/src/legality.rs`'s `RefinementError::ResultArity` mirror is unchanged in value and still documents the old count; realigning that doc comment is [`realign-the-compiler-refinement-error-mirror-with-the-grouped-result-arity`](realign-the-compiler-refinement-error-mirror-with-the-grouped-result-arity.md), serialized behind a live `implementation/compiler` claim.

### The choices worth objecting to

**One binding per root, rather than one binding carrying a set of accesses.** The set shape is more faithful to what the region holds, and it is what the ticket named as the alternative. It was rejected on evidence: no consumer anywhere in the workspace reads a `ResultBinding` field — `tiler-compiler`'s two `result_bindings()` methods are pass-throughs and every other site is a test asserting `len() == 1` — so the set shape would change a public type to serve no reader, while the repeat shape is what `bind_operands` already does for the identical many-to-one question on the operand side (one binding per reading stage, not one per operand). Naming a single member instead was not available: a receipt naming one of several roots is a claim about a write the region does not make alone.

**Distinct output tensors as the arity population.** Two roots writing two genuinely different output tensors are still two outputs and still disagree with one result; only roots over *one* tensor group. Held by `two_distinct_output_tensors_still_disagree_with_one_result`, watched failing under a perturbation that collapsed the grouping key.

**No new receipt-side population limit.** The binding count is now the region's root count rather than the result count, bounded by the region's own `MAX_OUTPUT_ROOTS` (4,096). Operand bindings needed an independent `MAX_INDEX_REFINEMENT_OPERAND_BINDINGS` because alias expansion can exceed the region's own boundary ceiling; root-derived result bindings cannot exceed a ceiling the region already enforces.

### The evidence

No identity-domain step. `ResultBinding` is encoded into `IndexRefinementExecutableCoverageIdentity`, which reaches `CoveredOccurrence` → kernel-program identity → the artifact stage key, so the encoding is pinned. One record per root keeps that grammar self-delimiting under the existing record count and makes a one-root result write exactly the bytes it always wrote; the whole workspace suite (2,916 tests, including the 20 pinned explain digests, the six Metal shader golden identity pairs, and the artifact/kernel-program stage-key agreement test) is green with no pin recomputed. Three deliberate perturbations were each watched failing and reverted. A tested implementation is a concrete draft, not approval of its spelling.

## Decision

_Parked for Tom._

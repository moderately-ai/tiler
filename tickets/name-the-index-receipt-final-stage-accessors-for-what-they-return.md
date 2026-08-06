---
id: name-the-index-receipt-final-stage-accessors-for-what-they-return
title: Name the index receipt final-stage accessors for what they return
status: in-progress
priority: p2
dependencies: [accept-the-multi-region-index-realization-surface]
related: [lower-a-two-region-occurrence-through-one-index-access-capability, admit-a-multi-region-index-realization-law]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [api, ir, indexing, naming]
claimed_from: todo
assignee: agent-accessor-rename
lease_expires_at: 1786026692
---
## User-visible outcome

`IndexRefinementReceipt::region`/`scalar_authority` and `PendingIndexRefinementReceipt::region`/`scalar_authority` are named for the stage they answer, so a consumer reading one cannot mistake the final stage of a chain for the whole realization.

## Why this is filed

**Fact.** [`admit-a-multi-region-index-realization-law`](admit-a-multi-region-index-realization-law.md) made a realization an ordered `VerifiedIndexRegionSequence` and retained the two single-region accessors on both receipts, re-documented to answer *the final stage*. Its acceptance node names the open question directly: "should a staged receipt expose a single-region accessor at all?" and records that removal was not available to it, because `tiler-compiler` reached the accessors from `const fn`.

**Fact — the misread is observed, not hypothesized.** [`lower-a-two-region-occurrence-through-one-index-access-capability`](lower-a-two-region-occurrence-through-one-index-access-capability.md) answered the question from the consuming side and found two sites that had already read the accessor as the realization: `complete_pending_index_refinement` handed `PendingIndexRefinementReceipt::region()` to content assembly as the whole realization, and `IndexRefinement::region()`'s documentation claimed the `tiler-reference` oracle could evaluate it against `operand_bindings` — false for a chain, whose final stage reads a value no occurrence operand carries. Both were well-typed reads of the wrong thing, and both are fixed compiler-side.

**Inference — retention survives, the spelling does not.** The final stage is a real concept rather than a truncation: its writes are the occurrence's results, `bind_results` derives from it alone, and it is a separate field so that "a realization has a final stage" is a type invariant. Removing the accessor pushes that invariant back to every consumer as an `expect`. What does not survive is a name that says `region` while answering one stage of several.

## Proposed shape

Rename to `final_stage()` and `final_scalar_authority()` on both receipts, keeping `regions()`/`realization()`/`scalar_authorities()` beside them unchanged. Consider an `Option`-returning `single_region()` mirroring the compiler-side spelling `lower-a-two-region-occurrence-through-one-index-access-capability` landed, so a consumer that can evaluate exactly one region refuses a chain explicitly rather than truncating it.

## Non-goals

Changing what any accessor returns, any identity encoding, or any verification behaviour. This is a naming change over an accepted surface; if the acceptance ruling changes the surface itself, this ticket is superseded rather than layered on top.

## Closes when

Both receipts' accessors are renamed, every call site in the workspace is updated, and `cargo doc` describes the final-stage semantics at the new names.

## Trigger check log

- 2026-08-06 — not fired. The dependency is `awaiting-decision`: renaming a surface Tom has not yet accepted would rewrite the exact interface his ruling is about. Reproduce: `tkt show accept-the-multi-region-index-realization-surface --field status`.

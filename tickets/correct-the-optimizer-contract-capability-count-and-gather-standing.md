---
id: correct-the-optimizer-contract-capability-count-and-gather-standing
title: Correct the optimizer contract's capability count and gather standing
status: in-progress
priority: p2
dependencies: []
related: [lower-a-recognized-gather-through-a-governed-capability]
scopes: [contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [docs, compiler, gather]
claimed_from: todo
assignee: worker-optdoc
lease_expires_at: 1787471310
---
## User-visible outcome

`docs/compiler/optimizer.md` states the governed index-access capability population and the gather family's standing in terms that stopped being true when the governed `tiler::gather-f32@1` lowering row landed. A reader of the optimizer contract is told a count and a class that the tree contradicts.

## Why this exists

Filed 2026-08-23 by `worker-gathercap` while landing [`lower-a-recognized-gather-through-a-governed-capability`](lower-a-recognized-gather-through-a-governed-capability.md). That lane holds `implementation/compiler` and `project/tickets`; `docs/compiler/**` is `contracts/optimizer`, which it does not hold, so the repair could not land with the change that caused it.

## Facts, each read at `a0fd5af2` in the file it names

**Fact — the stated count is one short.** `docs/compiler/optimizer.md` says the two walled families "nonetheless hold a registered index-access lowering capability, among the twenty-one `governed_index_access_capabilities` returns", and glosses that as "fourteen fixed-signature families plus one per admitted concatenate arity". Anchor: `among the twenty-one`. At the landing commit `GOVERNED_INDEX_ACCESS_CAPABILITIES` is **22** — fifteen fixed rows plus seven concatenate arities — and `the_governed_registry_holds_one_capability_per_admitted_concatenate_arity` asserts the frozen registry against it.

**Fact — the gather sentence names the wrong wall.** The same paragraph's closing sentence about this family reads, in the source, `tiler::gather-f32@1` is the same class on the ordinary path: the governed target answers `DTypeNotDispatchable` for its index type before recognition, and a later `dtype-recognized` wall is still not this rule. Anchor: `is the same class on the ordinary path`. The first clause stays true — the governed target still refuses the U32 index before recognition — but the sentence is now the *only* thing the document says about a family that has a registered lowering row, refines to a verified index region, and stops at `RegionVocabularyWall::GatherProofUnavailable`. Under a U32-capable profile the refusal is `("planning", "region-vocabulary")`, pinned by `a_governed_gather_refuses_at_dispatch_then_at_the_region_vocabulary` in `crates/tiler-compiler/src/request/tests.rs`.

## Required work

- Re-audit both Facts at your own base before editing; the count in particular moves whenever a capability row lands.
- Correct the count and its gloss, and add the gather family to the paragraph's account of what holds a registered lowering and what wall it stops at, under the document's own dated-correction convention.
- Do not restate the wall's reason for existing; `crates/tiler-compiler/src/physical.rs`'s `spell_output` gather arm owns it and [`thread-resolved-lowering-into-the-governed-spelling-path`](thread-resolved-lowering-into-the-governed-spelling-path.md) owns retiring it.

## Non-goals

Any `crates/` change. Retiring `RegionVocabularyWall::GatherProofUnavailable`. Re-opening the accepted data-dependent index surface.

## Closes when

The document's capability count and gather standing agree with the tree at the base the repair lands on, `make citations` passes, and no other document repeats the superseded count.

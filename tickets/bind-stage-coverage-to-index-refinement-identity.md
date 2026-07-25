---
id: bind-stage-coverage-to-index-refinement-identity
title: Bind kernel-program stage coverage to the index refinements it rests on
status: todo
priority: p1
dependencies: [correct-adr-0071-retained-lower-layer-identity-cardinality]
related: [bind-the-scheduled-region-to-the-verified-index-region-identity]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, identity]
---
**Fact — a verified index region's identity reaches no verified product.** `crates/tiler-compiler/src/legality.rs::emit_region` builds and verifies a `VerifiedIndexRegion` per semantic occurrence and `IndexRefinement` carries its `CanonicalIndexRegionIdentity`. The only consumer is `crates/tiler-compiler/src/pipeline.rs::refinement_label`, which slices `identity().as_bytes()` into an `EXPLAIN` string. The index layer's verifier, compaction, and identity derivation therefore contribute to explain output and to nothing else.

**Fact — the program layer is where the cardinality already fits.** `crates/tiler-ir/src/program/model.rs` gives each stage a `coverage: Vec<SemanticOccurrence>`, folded into program identity at its encoder. Coverage is already one-stage-to-many-occurrences, which is the same shape as one scheduled region to many refined index regions. A stage that named the *refinement identity* alongside the occurrence would state which verified index region proves it implements that occurrence, rather than only which occurrence it claims.

**Why not on the schedule.** `bind-the-scheduled-region-to-the-verified-index-region-identity` records the evidence: a scheduled region stands over several refined regions, so a single retained identity on it could name only one of them. That ticket's outcome has the full argument.

## Scope

Carry the refinement identity from `IndexRefinement` into the verified kernel program's stage coverage and into program identity, so a program names the exact verified index regions its stages rest on. Two decisions this ticket owns: whether coverage becomes a pair type or gains a parallel vector — the former keeps an occurrence and its evidence inseparable — and whether a stage with a recorded proof gap rather than a refinement is representable, since `pipeline.rs` already distinguishes `OccurrenceEvidence::Refined` from a gap and collapsing them would let an unproved stage look proved.

Changing the stage coverage type is a public-boundary change in `tiler_ir::program`, so the exact signature is Tom's to accept.

## Closes when

A verified kernel program names the refinement identity behind each covered occurrence, program identity separates two programs that differ only in which verified index region proves a stage, a recorded proof gap stays distinguishable from a refinement, and `uv run --locked python scripts/check_repository.py` passes.

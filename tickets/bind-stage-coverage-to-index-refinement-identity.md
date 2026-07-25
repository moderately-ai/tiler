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
**Fact — a verified index region's identity reaches no verified product.** `crates/tiler-compiler/src/legality.rs::emit_region` builds and verifies a `VerifiedIndexRegion` per semantic occurrence and `IndexRefinement` carries its `CanonicalIndexRegionIdentity`. The exact check is `grep -rn 'CanonicalIndexRegionIdentity' crates/tiler-ir/src/schedule crates/tiler-ir/src/kernel crates/tiler-ir/src/program crates/tiler-artifact/src crates/tiler-metal/src`, which returns nothing.

**Sharpened 2026-07-25 while landing `correct-adr-0071-retained-lower-layer-identity-cardinality`, because the original wording understates what already exists and a worker who inherits it will build the wrong thing.** This ticket first said the identity's "only consumer" is `pipeline.rs::refinement_label`. That is not accurate inside `legality.rs`, where the identity is already folded into two complete compiler-owned identities: `encode_content_identity` folds it into `RefinementContentIdentity`, `encode_occurrence_identity` folds that into `IndexRefinementIdentity`, and `tiler_ir::index::ScalarAuthorityEvidence` binds its region-bound receipt to the same bytes. The accurate statement is that the chain *terminates* there. `pipeline.rs` retains the `IndexRefinement` in `CompletePlans` and consumes it in exactly two places, both explain — `record_refinement`, which renders eight trailing bytes through `refinement_label` into a presentation handle, and `record_numerical_equivalence`, which reads only the resolved provider. The artifact plan records `ResolvedLowering::providers()`, deduplicated lowering provenance, and no region or refinement identity. So the work here is to carry an identity that already exists and is already complete into a verified product, not to derive one.

**Fact — the program layer is where the cardinality already fits.** `crates/tiler-ir/src/program/model.rs` gives each stage a `coverage: Vec<SemanticOccurrence>`, folded into program identity at its encoder. Coverage is already one-stage-to-many-occurrences, which is the same shape as one scheduled region to many refined index regions. A stage that named the *refinement identity* alongside the occurrence would state which verified index region proves it implements that occurrence, rather than only which occurrence it claims.

**Why not on the schedule.** `bind-the-scheduled-region-to-the-verified-index-region-identity` records the evidence: a scheduled region stands over several refined regions, so a single retained identity on it could name only one of them. That ticket's outcome has the full argument.

## Scope

Carry the refinement identity from `IndexRefinement` into the verified kernel program's stage coverage and into program identity, so a program names the exact verified index regions its stages rest on. Two decisions this ticket owns: whether coverage becomes a pair type or gains a parallel vector — the former keeps an occurrence and its evidence inseparable — and whether a stage with a recorded proof gap rather than a refinement is representable, since `pipeline.rs` already distinguishes `OccurrenceEvidence::Refined` from a gap and collapsing them would let an unproved stage look proved.

Changing the stage coverage type is a public-boundary change in `tiler_ir::program`, so the exact signature is Tom's to accept.

## Closes when

A verified kernel program names the refinement identity behind each covered occurrence, program identity separates two programs that differ only in which verified index region proves a stage, a recorded proof gap stays distinguishable from a refinement, and `uv run --locked python scripts/check_repository.py` passes.

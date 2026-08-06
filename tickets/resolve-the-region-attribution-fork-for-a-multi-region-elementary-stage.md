---
id: resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage
title: Resolve the region-attribution fork for a multi-region elementary stage
status: done
priority: p1
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages, widen-the-staged-realization-law-to-the-registered-elementary-families]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The fork, and why it is Tom's rather than a worker's

A registered elementary family whose realization is a region *sequence* -- the normalization's fold-then-normalize, the softmax's three passes -- has no representation in the compiler's region-attribution model. Two resolutions are available, they encode different priorities, and neither is dominated, so this parks per AGENTS.md rather than being chosen inside an implementation ticket.

## The derivation: the atom of the cover is a semantic occurrence

**Fact.** Region attribution is keyed on the exact set of semantic occurrences a region covers, and nothing finer exists:

- `NormalizedOutput::owns_region_members(&[SemanticMemberId])` (`crates/tiler-compiler/src/request.rs:1660-1680`) distinguishes an output's parts by *disjoint member sets*: a `SerialSum`'s prologue part, its reduction part, and their union; an `Epilogue`'s own part versus its producer's.
- `physical::spell_output` (`crates/tiler-compiler/src/physical.rs:442-518`) resolves a placed region by `members ==` comparisons in that same order, and answers with the first arm that matches.
- `cover::derive_duplication` (`crates/tiler-compiler/src/cover.rs:1999-2018`) treats a member appearing in two regions as *deliberate duplication of that occurrence*, never as a split of it.

**Inference.** A single elementary occurrence realizing as a fold region plus a normalizing region gives both regions the *same* member set -- the one occurrence. `spell_output`'s first matching arm therefore answers for both, the second region is unreachable, and `owns_region_members` cannot tell the cover search which part a candidate is. The existing epilogue chain works only because its two parts come from *different* occurrences: `sum(x * x) * scale` is three occurrences the walk splits, and `rms_norm(x, w)` is one.

**Fact.** `RegionWrite` (`physical.rs:222-252`) does not disambiguate them either: `spell_output` matches on members first and passes `write` through into the spelling. The publishing-copy second dispatch (`RegionWrite::MaterializedAndPublished`) is a *copy* of a computed value, not a second computed stage, so it is not the seam.

## Option A -- make the cover's atom a stage of an occurrence

Replace `SemanticMemberId` as the attribution key with a `(member, stage)` pair throughout the region graph, cover search, duplication accounting, spelling, and program assembly.

- **Enables.** Every recognized family's stages become ordinary cover atoms, so fusion across a family's internal boundary, and a stage participating in a neighbour's region, are both statable -- which is what an attention chain eventually wants.
- **Prevents / costs.** It moves an identity domain: cover identity, region-occurrence identity, and explain records all encode member positions. It is the wider change and the one with the larger blast radius.

## Option B -- let one placed region carry a multi-stage spelling

Keep the occurrence as the cover atom and add a `RegionSpellingKind` arm whose scheduled realization is an ordered stage sequence, the way a split reduction already spells a partial pass and a final pass from one recognized output.

- **Enables.** No change to the cover's atom, its identity, or its duplication accounting; the widening stays local to `NormalizedOutput`, `RegionSpellingKind`, the `physical` builders, `NormalizedOutputSubject`, and `verify_region_output_binding`.
- **Prevents.** The family's internal boundary is opaque to the cover search, so nothing can ever fuse across it -- a later ticket wanting the softmax's exponential pass fused into a neighbouring contraction would have to take Option A after all.

## What a worker must not do

Pick one and implement it. Both are coherent, and the choice is about whether a family's internal stage boundary is a first-class planning object. Per AGENTS.md this is an architecture fork: draft both, park, and let Tom decide.

## Closes when

Tom names the option. The chosen one is then filed as its own implementation ticket with the surface it touches enumerated.

## Decided 2026-08-06 — Option A, stage-level atoms

**Tom decided at the live session (coordinator's question round, witnessed and executed by the coordinator):** the attribution atom becomes a *(member, stage)* pair. The elimination was run under Tom's stated priority order (correctness, performance, long-term maintainability, code quality): correctness equal; performance — stage atoms enable fusing a family's internal pass into a neighbouring region, the flash-shaped plan; maintainability — one identity migration rather than B-then-A; quality — stages are real domain objects. Option B is rejected rather than deferred. Tom additionally confirmed the DX ground: explain records gain per-stage attribution. Implementation is [`implement-stage-level-cover-atoms-for-multi-region-occurrences`](implement-stage-level-cover-atoms-for-multi-region-occurrences.md).

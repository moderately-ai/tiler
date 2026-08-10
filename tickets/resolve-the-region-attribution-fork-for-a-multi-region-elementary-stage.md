---
id: resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage
title: Resolve the region-attribution fork for a multi-region elementary stage
status: done
priority: p1
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages, implement-stage-level-cover-atoms-for-multi-region-occurrences, widen-the-staged-realization-law-to-the-registered-elementary-families]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The fork, and why it is Tom's rather than a worker's

**Correction — 2026-08-10.** The opening problem statement and the derivation Facts below are the **pre-decision** model this ticket parked. They are not live claims about the tree. Tom chose Option A on 2026-08-06; [`implement-stage-level-cover-atoms-for-multi-region-occurrences`](implement-stage-level-cover-atoms-for-multi-region-occurrences.md) is `status: done` and the attribution atom is now `SemanticStage { member, stage }` (`crates/tiler-compiler/src/region.rs`). Multi-region elementary realizations are represented as separate stage atoms and spelled as `StagedFold` / `StagedPass` regions. Rotten line ranges that once pointed at `owns_region_members`, `spell_output`, `derive_duplication`, and `RegionWrite` are struck below; cite those symbols by name. A later, separate fork on *which authority mints* multi-stage candidates (`resolve-which-authority-mints-a-multi-stage-region-candidate`) is not remainder of this close condition.

~~A registered elementary family whose realization is a region *sequence* -- the normalization's fold-then-normalize, the softmax's three passes -- has no representation in the compiler's region-attribution model.~~ **Historical at filing; obsolete after Option A.** Two resolutions were available, they encoded different priorities, and neither was dominated, so this parked per AGENTS.md rather than being chosen inside an implementation ticket.

## The derivation: the atom of the cover is a semantic occurrence

**~~Fact~~ — historical pre-Option-A keying.** Region attribution was keyed on the exact set of semantic occurrences a region covers, and nothing finer existed:

- `NormalizedOutput::owns_region_members` (`crates/tiler-compiler/src/request.rs`) distinguished an output's parts by *disjoint member sets*: a `SerialSum`'s prologue part, its reduction part, and their union; an `Epilogue`'s own part versus its producer's. **Correction — 2026-08-10.** The live signature is `owns_region_members(&self, members: &[SemanticStage]) -> bool` (stage atoms, not bare `SemanticMemberId`). Do not cite the retired `request.rs:1660-1680` range.
- `physical::spell_output` (`crates/tiler-compiler/src/physical.rs`) resolved a placed region by `members ==` comparisons in that same order, and answered with the first arm that matched. **Correction — 2026-08-10.** First-match on members remains, plus a `NormalizedOutput::Staged` arm that defers which-stage to `spell_staged`. Do not cite the retired `physical.rs:442-518` range.
- `cover::derive_duplication` (`crates/tiler-compiler/src/cover.rs`) treated a member appearing in two regions as *deliberate duplication of that occurrence*, never as a split of it. **Correction — 2026-08-10.** Live keying is `BTreeSet<SemanticStage>`; two different stages of one occurrence are a split, not a duplication. Do not cite the retired `cover.rs:1999-2018` range (that span was `assemble_resolved_cover` at audit base).

**~~Inference~~ — historical collision under the bare-occurrence key.** A single elementary occurrence realizing as a fold region plus a normalizing region gave both regions the *same* member set -- the one occurrence. `spell_output`'s first matching arm therefore answered for both, the second region was unreachable, and `owns_region_members` could not tell the cover search which part a candidate was. The existing epilogue chain worked only because its two parts came from *different* occurrences: `sum(x * x) * scale` is three occurrences the walk splits, and `rms_norm(x, w)` is one. **Correction — 2026-08-10.** That collision is the documented rationale for `SemanticStage`; live multi-stage elementary realizations no longer share one bare-member atom set.

**~~Fact~~ — historical: write role was not the stage seam.** `RegionWrite` (`crates/tiler-compiler/src/physical.rs`) did not disambiguate stages either: `spell_output` matched on members first and passed `write` through into the spelling. The publishing-copy second dispatch (`RegionWrite::MaterializedAndPublished`) is a *copy* of a computed value, not a second computed stage, so it is not the seam. **Correction — 2026-08-10.** That conceptual claim still holds under Option A (stages are not encoded as write roles); do not cite the retired `physical.rs:222-252` range.

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

**Correction — 2026-08-10.** Option A landed via that implementation ticket (`status: done`). Bare-occurrence keying and the retired line citations in the derivation section are pre-decision only. Current atom is `SemanticStage`; `owns_region_members`, `spell_output`, and `derive_duplication` all key or compare on stage atoms; staged families spell as separate regions (`StagedFold` / `StagedPass`), not Option B's multi-stage single-region spelling. Close condition remains met: Tom named the option and the implementation vehicle was filed and delivered. Do not re-open this ticket for Option A′ minting-authority, identity-fold, or stage-enumeration work — those are separate graph nodes.

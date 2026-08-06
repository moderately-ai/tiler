---
id: resolve-which-authority-mints-a-multi-stage-region-candidate
title: Resolve which authority mints a multi-stage region candidate
status: awaiting-decision
priority: p1
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages, fold-the-attribution-stage-into-region-and-request-subject-identity, implement-stage-level-cover-atoms-for-multi-region-occurrences, resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, planner, identity-domain]
---
## The fork, and why it parks rather than being chosen

Tom decided on 2026-08-06 that the planner's attribution atom is a *(member, stage)* pair — Option A of [`resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage`](resolve-the-region-attribution-fork-for-a-multi-region-elementary-stage.md) — and [`implement-stage-level-cover-atoms-for-multi-region-occurrences`](implement-stage-level-cover-atoms-for-multi-region-occurrences.md) landed the atom at every attribution site. What that decision did **not** settle, and what nothing in the graph settles, is *which authority mints a candidate carrying a non-first stage*. Two answers are coherent, they touch different surfaces, and they disagree about whether an identity domain moves at all — so the encoding ticket [`fold-the-attribution-stage-into-region-and-request-subject-identity`](fold-the-attribution-stage-into-region-and-request-subject-identity.md) either fires or does not fire depending on which is taken. That is the definition of a fork this repository parks.

## The derivation, verified at its cited sites on `c22d4b24`

**Fact — nothing mints a multi-stage claim except one physical pass.** `region::assemble` (`crates/tiler-compiler/src/region.rs:1592`–`:1614`) is the sole constructor of a `RegionCandidate` and mints `SemanticStage::first` for every member, stating why at the site: it observes the semantic DAG alone, "where an occurrence is one operation and its realization has not been chosen, so it has nothing to number stages from". The only non-first atom in the tree is `physical::final_reduction_region`'s combine claim (`physical.rs:1709`–`:1714`), which is a *dispatch* of one placed cover region rather than a candidate.

**Fact — the recognizer's parts are compared for exact equality against a cover region's atoms.** `NormalizedOutput::owns_region_members` (`request.rs:1660`–`:1680`) and `physical::spell_output` (`physical.rs:442`–`:518`) both decide by `members ==`. So a recognized part naming a non-first stage is unusable unless some cover region carries that atom, and a cover region's atoms come from a candidate. A staged *partition* therefore implies staged *candidates*; there is no third arrangement.

**Fact — region formation is not in a position to mint them today, and the obstacle is a value rather than a loop.** `RegionShape` (`region.rs:1540`–`:1579`) derives `boundary_inputs` and `retained_outputs` from `SemanticValueId`s, which are positions in the verified program's own value list (`region::value_ordinal`, `:284`). A staged occurrence's published intermediate — the normalization's `r`, the softmax's row maximum — is *no program value*: it exists only inside the realization the law builds. Splitting one occurrence into two candidates therefore requires the region graph to carry synthetic values, which propagates to `RetainedOutput`, `MaterializationEdge::value` (`cover.rs:454`–`:459`), program assembly's internal values, and the assembled program's ABI.

**Fact — the cover search would refuse such a cover even if it existed.** `verify_cover` counts per *operation* (`cover.rs:1158`–`:1180`) and reads a count above one as `IllegalDuplication` under the governed policy; `Partitioner::refused_duplication` (`:1592`–`:1607`) prunes it during the search; the completeness test (`:1446`) calls an occurrence covered when only its second stage is present. `cover::member_index`'s own doc (`:2379`–`:2387`) records this as the fail-closed direction and names the obligation: "the mask has to require every *stage* covered once, which is a different obligation from every operation covered once."

## Option A′ — region formation enumerates stages as graph nodes

`form_region_candidates` is handed each occurrence's stage count (from the registered `IndexRealizationLaw`), the region graph gains one node per stage with an intra-occurrence producer/consumer edge and a synthetic value on it, and connectivity, convexity, canonical member order, boundary derivation, and the content/occurrence encodings all range over stages.

- **Enables.** Exactly what Tom's Option A rationale names: a stage of one family fusing into a neighbouring region, which is the flash-shaped plan. The cover search sees the family's internal boundary.
- **Costs.** Region formation stops being a pure function of the semantic DAG — it needs the law registry, which means the scalar authority, which means `form_region_candidates` takes what today only `VerifiedTargetRequest` holds. Synthetic values reach the cover's materialization edges and the assembled program. The stage folds into `encode_content` and `encode_occurrence`, so `fold-the-attribution-stage-into-region-and-request-subject-identity` fires whole and moves region, cover, and explain identity.

## Option B′ — the recognizer supplies the split and region formation stays DAG-only

Recognition classifies a staged family into a partition of *k* parts and hands region formation the split as an explicit input rather than deriving it; region formation emits one candidate per part for those occurrences alone and leaves every other program's enumeration byte-identical.

- **Enables.** The same stage atoms in covers and spellings, with region formation's DAG walk untouched for every program containing no staged family — so no existing candidate's content identity can move for a reason unrelated to the change.
- **Costs.** Two authorities describe one split (the law and the recognizer) and must agree; the repository's stated preference is one authority per fact, and `recognize_elementwise_output`'s own doc argues exactly that against a second classifier. The synthetic-value problem is unchanged. Whether `encode_content` needs the stage depends on whether two parts of one occurrence can ever produce equal content bytes, which has to be derived rather than assumed.

## What a worker must not do

Pick one and implement it. The difference is whether the region graph is a view of the semantic DAG or a view of the *chosen realizations* of it, which is an architectural commitment, and it decides whether an identity domain moves.

## Closes when

Tom names the option. The chosen one is filed as its own implementation ticket with the surface it touches enumerated, and `fold-the-attribution-stage-into-region-and-request-subject-identity` records the resulting trigger evaluation.

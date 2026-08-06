---
id: admit-a-scheduled-region-for-a-staged-elementary-family
title: Admit a scheduled region for a staged elementary family
status: todo
priority: p1
dependencies: []
related: [admit-the-registered-elementary-families-as-recognizable-program-stages, accept-the-root-mean-square-scale-realization-law]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, scheduling, identity-domain]
---
## User-visible outcome

A program whose middle stage is a registered elementary family compiles end to end with reference bit-agreement, instead of stopping at `RegionVocabularyWall::StagedFamilyUnspellable`. This is the **physical half** [`admit-the-registered-elementary-families-as-recognizable-program-stages`](admit-the-registered-elementary-families-as-recognizable-program-stages.md) stopped at and filed; that ticket's recognizer half has landed, so every layer above this one already works.

## Where the wall is, verified on `b3d5a9ed` plus the recognizer landing

**Fact — everything above the scheduled region already runs.** For `rms_norm(value, weight) * value` over `[2, 2]`, `compile()` now: recognizes the normalization as `NormalizedOutput::Staged`; resolves `tiler.governed-index-access.rms-norm-f32@1`; refines it — `realization-stages: count=2`, one handed value, producer stage 0, consumer stage 1; enumerates one region candidate per stage; and enumerates four legal covers, including the one that fuses the normalization's pass beside its downstream consumer. `pipeline::tests::a_staged_family_program_reaches_its_lowering_and_names_the_vocabulary_wall` is the measurement.

**Fact — the missing thing is a `ScalarProgram`.** `tiler_ir::schedule::ScalarProgram` has eight variants (`crates/tiler-ir/src/schedule/model.rs:473`). Stage zero of `IndexRealizationLaw::StagedRootMeanSquareScaleF32` folds each contributor's *square* and then applies `/N`, `+eps`, and `Rsqrt` to the fold's value **inside the producing region**. `ScalarProgram::SquaredSerialSum` is the closest variant and carries no epilogue — deliberately, and its doc-comment says so: "the division by the extent, the `eps` addition, the reciprocal square root, and the two multiplies belong to the pointwise pass that consumes this reduction's result".

**Fact — that doc-comment and the accepted law disagree about where the epilogue lives**, and the disagreement is the substance of this ticket. The law's Outcome records why the epilogue is in stage zero: `r` is computed once per folded row and read once per point, so publishing `a` and putting `/N`, `+eps`, `Rsqrt` in the pointwise pass evaluates each `N` times per row — a different scalar program, not a different schedule. Refinement compares a provider's emission against the law byte for byte, so a scheduled region built the other way would not be the realization the compile path proved.

## The surface this touches

- **`tiler-ir` schedule vocabulary.** A `ScalarProgram` variant for a squared serial fold with a scalar epilogue on the fold's value, or a decision that the law moves instead. This is a public `#[non_exhaustive]` enum: land as a labelled draft with its own acceptance node, and derive the canonical region-identity encoding's per-tag injectivity at the encoding site.
- **`tiler-compiler` physical.** A `RegionSpellingKind` variant, a region builder, the `frontier::GovernedPhysicalProvider::propose` dispatch, `physical::spell_output`'s staged arm (which today returns the wall), and `physical::verify_region_output_binding`'s `(NormalizedOutputSubject::Staged, _)` arm (which today answers `false`). The recognized shape it binds against is already in place.
- **`tiler-compiler` program assembly.** A staged occurrence's handed value reaches `derive_materializations` as an ordinary `MaterializationEdge` today, but `CoverAssembly::from_plan` shapes each internal from the producing stage's `iteration_shape` and cross-checks it against the edge's `element_count` (`program.rs`, `materialized-extent-disagreement`). The normalization's handed value is one element per folded row while its fold stage iterates the *published* domain, so this is the first check to derive against rather than assume.
- **`tiler-metal` emission and `tiler-reference` evaluation** for the new scalar program, and the cost model.

## Non-goals

The softmax's law (blocked on its own two tickets); a staged family that *reads* a materialized intermediate ([`admit-a-staged-family-that-reads-a-materialized-intermediate`](admit-a-staged-family-that-reads-a-materialized-intermediate.md)); the failure-class question ([`classify-a-vocabulary-gap-refusal-as-an-unsupported-capability`](classify-a-vocabulary-gap-refusal-as-an-unsupported-capability.md)).

## Closes when

A program with a registered elementary family as a middle stage compiles through the ordinary path and agrees with `tiler-reference` bit for bit, the scheduled-region vocabulary addition is a labelled draft with an acceptance node, every moved identity pin is recomputed on the landing tree and enumerated, and the staged arm of `spell_output` returns a spelling instead of a wall.

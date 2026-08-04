---
id: admit-an-indirect-gather-family-for-tied-embedding-lookup
title: Admit an indirect gather access family
status: todo
priority: p1
dependencies: [derive-transformer-operation-and-shape-surface, reclassify-language-model-work-as-a-conformance-track]
related: [own-operation-family-support-matrix, design-model-ingestion-and-complete-execution, implement-index-domain-predicates]
scopes: [contracts/foundation, implementation/ir, implementation/reference, implementation/compiler, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, indexing, gather, language-model, breadth, class-generic-capability]
---
## User-visible outcome

A program can use one tensor's values as coordinates into another — an indirect, tensor-data-derived access class that the admitted index vocabulary rejects by construction and that no composition of admitted families can express.

**Retitled 2026-08-04 under [`reclassify-language-model-work-as-a-conformance-track`](reclassify-language-model-work-as-a-conformance-track.md).** The outcome above read "A language-model program can read its own input: token IDs select rows of the embedding matrix". The access class is generic; a tied embedding lookup is the occurrence that found it and is the workload evidence below, never the thing that names or owns it. **The ticket id is deliberately unchanged**: five records outside this ticket's editable scopes link to it by filename — `docs/research/shapes/transformer-operation-and-shape-surface.md:166`, `docs/research/numerics/first-quantized-lm-profile.md:182`, `docs/research/program-planning/first-attention-program-vertical.md:164`, `docs/research/program-planning/model-level-qualification.md:356`, and `docs/research/program-planning/complete-model-ingestion-and-execution.md:105`/`:305` — so renaming the file would trade a workload-flavoured identifier for broken links a reader hits and no gate reports.

## Evidence prerequisite

**Fact — the admitted access language rejects it by construction.** [`docs/ir.md`](../docs/ir.md) Layer 2 bounds the initial index vocabulary to addition and negation, multiplication by a parameter-only expression, and Euclidean floor division or modulo by a proven-positive parameter-only expression, and states that "iteration-by-iteration multiplication and tensor-data-derived indices are rejected". A token ID read from an input tensor and used as a row coordinate is exactly a tensor-data-derived index. This is not a missing key over an existing access class; it is a missing access class.

**Fact — the corpus already tracks it as absent, with no owner.** [Q-SHAPE-007](../docs/open-questions.md#q-shape-007--indirect-gatherscatter-relations) states the trigger as "gather/scatter enters an active product profile" and records that closure "needs bounds, duplicate-write, determinism, and validation rules". The [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) mentions gather only inside the structural row's trigger — "Gather and scatter stay out until Q-SHAPE-007 triggers" — and gives it no row of its own, so the family has no recorded rung at all.

**Fact — the workload evidence, from the L2 derivation.** [The transformer operation and shape surface derivation](../docs/research/shapes/transformer-operation-and-shape-surface.md) records one gather occurrence per forward pass of the pinned `Qwen/Qwen3-0.6B-Base` profile: `[T]` token IDs index a `[151936, 1024]` F32 matrix to produce `[T, 1024]`. The bounded rows fix `T` at 10 then 1 for the conformance row and up to 8192 for the benchmark matrix, and the pinned `vocab_size` is 151,936. **Fact — the same matrix is also a contraction operand.** The checkpoint carries `tie_word_embeddings: true` and no `lm_head.weight`, so one tensor serves the gather on the input side and the vocabulary projection on the output side; the semantic graph admits one value with two consumers, and a plan that allocates two copies doubles the model's largest single allocation.

**Inference — the ratio is why this is a boundary question and not a cost question.** One gather sits against 253 contractions in the same forward pass, so its execution cost is negligible. What it decides is whether the program has a boundary at all: with no admitted access class the model's first operation is not expressible, and the alternative is to move the lookup outside Tiler and hand the compiler a materialized `[T, 1024]` input, which is a different product boundary rather than a different implementation.

## Required delivery

One vertical carrying:

- **The access class.** An indirect relation in the index layer, with the bounds, duplicate-write, determinism, and validation rules Q-SHAPE-007 names as its closure condition. A read-only gather does not need the duplicate-write rule to be *implemented*, but it does need it *stated*, so that admitting scatter later is additive rather than a reinterpretation.
- **Semantic identity and validation.** A governed `OpKey` with an index-tensor operand of an admitted integer value type, a gathered-axis attribute, and validation that the result shape composes the index shape with the source's surviving axes.
- **The bounds obligation.** An index value outside `0..151936` must refuse or be validated at a named enforcement boundary. It may not clamp, wrap, or read out of bounds. `docs/ir.md` already fixes the shape of this: a semantic precondition is proved statically or the physical plan names a supported enforcement and publication boundary, and a semantic validation failure is never a plan miss.
- **Normative reference, lowering capability, target realization, and runtime binding**, on the same terms every other family owes them.
- **Bounded conformance evidence.** Token IDs at 0, at 151935, repeated, and out of range; an empty `T`; and the tied case where the same value feeds both a gather and a contraction, verified not to duplicate the allocation.
- **A matrix row** for the family, with its rung and its trigger, since it currently has none.

## Non-goals

Scatter, and any data-dependent output shape. The workload needs neither, and `docs/ir.md` separately holds data-dependent output shapes and device-produced launch dimensions as unsupported.

## Reconsideration trigger

Active now: the selected workload's first operation requires it. If the product boundary moves so that embedding lookup happens on the consumer side and Tiler receives materialized activations, this family stops being required by this workload — and that is a decision for [`design-model-ingestion-and-complete-execution`](design-model-ingestion-and-complete-execution.md) to make explicitly rather than a gap to leave open.

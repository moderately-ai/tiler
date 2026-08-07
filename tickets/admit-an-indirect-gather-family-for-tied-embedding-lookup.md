---
id: admit-an-indirect-gather-family-for-tied-embedding-lookup
title: Admit an indirect gather access family
status: todo
priority: p1
dependencies: [derive-transformer-operation-and-shape-surface, reclassify-language-model-work-as-a-conformance-track]
related: [own-operation-family-support-matrix, design-model-ingestion-and-complete-execution, implement-index-domain-predicates]
scopes: [contracts/foundation, implementation/ir, implementation/reference, implementation/compiler, implementation/metal, contracts/navigation, contracts/decisions, research/semantic-graph]
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

## Repaired before dispatch, 2026-08-07 — the "unowned and untracked" framing was false in four ways

Verified by the coordinator against the corpus. The workload evidence, the index vocabulary quote, the bounds-obligation shape, and the non-goals all **verify unchanged**; what follows corrects the tracking claims built on top of them.

### Struck: "the corpus already tracks it as absent, with no owner"

**Four records name this ticket as the owner.** `docs/open-questions.md:280` — "The gather half's trigger **has fired**, and it is re-owned rather than left standing as a deferral (2026-08-04)"; `docs/roadmap.md:417`, `:479`, `:481`; and `docs/research/semantic-graph/operation-family-delivery-graph.md:187`. The correction is therefore not "record an absence" but "advance a tracked, owned row" — a different and smaller job.

### Struck: the Q-SHAPE-007 trigger quote

The ticket quotes the trigger as "gather/scatter enters an active product profile". **That string exists nowhere in the tree except inside this ticket.** The current Q-SHAPE-007 (`docs/open-questions.md:281`) reads "Trigger, for the half that has **not** fired: scatter." The gather trigger is spent, not pending.

### Struck: the roadmap quote and the "no recorded rung" claim

"Gather and scatter stay out until Q-SHAPE-007 triggers" **no longer exists** in `docs/roadmap.md` (`grep -c` returns 0), and the structural row now says the opposite at `:479`: "Scatter likewise stays out under Q-SHAPE-007, but **gather no longer does** … it is a family this row does not carry rather than a class held behind an unfired trigger." Gather appears at `:353`, `:408`, `:417`, `:479` and `:481` — five sites, not one.

Two rungs are recorded: `docs/research/shapes/transformer-operation-and-shape-surface.md:67` and `:87` (R1), and `operation-family-delivery-graph.md:97` carries an explicit row — `| **O-08** Indirect gather | F-34 | owed; live ticket | owed | owed | owed |`. **The surviving true clause is narrower:** no row in the *roadmap matrix* specifically.

### Link line numbers refreshed, and a sixth record added

`transformer-operation-and-shape-surface.md` is `:188` (not `:166`); `model-level-qualification.md` is `:358` (not `:356`); `complete-model-ingestion-and-execution.md` is `:105` ✓ and `:322` (not `:305`). `first-quantized-lm-profile.md:182` and `first-attention-program-vertical.md:164` are correct. The unlisted sixth is **`docs/research/semantic-graph/operation-family-delivery-graph.md:187` and `:308`** — so six records, not five.

### Two scopes added, one of them an ADR obligation this ticket did not name

- **`contracts/decisions`.** ADR 0046 is `accepted`, and its `:48-49` carries the normative rejection this work must lift — "It rejects iteration-by-iteration multiplication and **tensor-data-derived indices**" — with `:73-74` adding that "Data-dependent gather, scatter, sparse iteration, and data-dependent cardinality **require later explicit IR contracts**." Admitting this access class supersedes or extends an accepted ADR, and the ticket named no ADR obligation at all. Per AGENTS.md, if the branch cannot land in `docs/decisions/`, preserve a **verbatim-landable ADR body** and file a carrier ticket — editing during transfer creates a fork.
- **`research/semantic-graph`**, for the O-08 row at `:97` and the F-34 join row at `:308`. `docs/roadmap.md:437` makes that file the authority for which ticket holds each family's next step, so it moves when the family lands.

### "Runtime binding" is not this ticket's to deliver

Required Delivery demands "runtime binding, on the same terms every other family owes them". `crates/tiler-runtime/**` is `implementation/runtime`, **not declared here and deliberately not added** — that work is owned by `admit-a-storage-carrier-for-integer-program-inputs`, whose own `dependencies` list contains *this* ticket. So the item is both out of scope and assigned downstream. **Drop it from Required Delivery** and note the discharge. The semantic side needs no work: `tiler::u32@1`, `i32` and `i64` are already registered in `crates/tiler-ir/src/semantic/catalog.rs`.

`implementation/conformance` is **not** added: the four landed peer family tickets all kept their evidence inside `implementation/ir` + `implementation/reference` and declared no conformance scope. Follow the peers.

### Struck: the unverifiable conformance item

"the tied case where the same value feeds both a gather and a contraction, **verified not to duplicate the allocation**" is degenerate in both readings. In the workload's real shape it can never say *yes* — `complete-model-ingestion-and-execution.md:68` records that under W-C those are two programs, so the matrix is bound twice, "binding two copies costs **622,329,856 bytes** and **nothing below the consumer can tell that the two bindings name one tensor**". Non-duplication is a consumer obligation with no enforcer here. In a synthetic single-program shape it can never say *no*, because inputs are bound rather than allocated, so one value with two consumers cannot duplicate. **Replace it with the two-program binding fact**, and hand the tied case to `assemble-the-embedding-and-vocabulary-projection-programs`, which already records that cost and is the real owner.

### Struck: the Reconsideration trigger

It cites `design-model-ingestion-and-complete-execution` as still owing the product-boundary decision. That ticket is `done` and **made the decision**, under the heading "The input boundary: the gather stays inside" — `complete-model-ingestion-and-execution.md:103`, `:105` ("It is made here."), with IN-A surviving at `:109` and IN-B eliminated at `:110`, confirmed by `docs/roadmap.md:408`. As written the trigger can fire only by superseding an accepted decision. Restate it as settled.

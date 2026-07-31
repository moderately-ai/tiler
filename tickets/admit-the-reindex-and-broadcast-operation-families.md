---
id: admit-the-reindex-and-broadcast-operation-families
title: Admit the Reindex and Broadcast operation families
status: todo
priority: p1
dependencies: [derive-transformer-operation-and-shape-surface]
related: [own-operation-family-support-matrix, scope-transformer-nonlinear-normalization-and-reductions, design-attention-program-vertical]
scopes: [implementation/ir, implementation/reference, implementation/compiler, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, structural, language-model, breadth]
---
## User-visible outcome

A program can state that a `[1024]` weight vector multiplies a `[T, 1024]` activation, and that a `[T, 2048]` projection is sixteen heads of width 128 — the two structural families every non-scalar tensor program needs before any of its arithmetic can be written down.

## Evidence prerequisite

**Fact — the families have normative semantics and no identity.** [`docs/ir.md`](../docs/ir.md) gives `Reindex` a total output-to-input coordinate function whose initial admitted forms are bijective permutations, splits, merges, and legal unit-axis insertion or removal, and gives `Broadcast` an explicit axis mapping for every many-to-one relation. The [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) places the row at R2 and records that no `Reindex` or `Broadcast` key exists, with a reproducible absence check. `StandardSemantics::register` in `crates/tiler-ir/src/semantic/registry.rs` registers four F32 operation keys and neither of these.

**Fact — the rank-zero admission does not substitute for either.** `docs/ir.md` states that the narrow scalar admission `tiler::add-f32@1` and `tiler::multiply-f32@1` declare is a shape rule for a rank-zero operand alone, and that "rank padding, extent-one stretching, and every other many-to-one mapping still require an explicit `Broadcast` with an axis mapping, in every signature and at every rank". `BinaryF32::infer` implements exactly that: one operand of rank zero, or shapes equal, or a typed rejection.

**Fact — the workload evidence, from the L2 derivation.** [The transformer operation and shape surface derivation](../docs/research/shapes/transformer-operation-and-shape-surface.md) counts, for one forward pass of the pinned `Qwen/Qwen3-0.6B-Base` profile at F32, at least 197 `Broadcast` occurrences and at least 280 `Reindex` occurrences. The broadcasts are the RMS-normalization weight multiply at 113 sites (`[1024]` against `[T, 1024]`, and `[128]` against `[T, 16, 128]`), the rotary `cos`/`sin` tables at 56 sites (`[T, 128]` against `[T, 16, 128]`), and the causal mask at 28 sites (`[T, S]` against `[8, 2, T, S]`). The reindexes are ten per layer: three Q/K/V head splits, one attention-output merge, and three each for the rotary half-split of Q and of K — all of which the derivation shows fall inside the admitted bijective forms. Every extent above is F32 and, except for `T` and `S`, static.

**Inference — this is a delivery gap and not a scoping gap.** The support matrix's own trigger cell names this row "Milestone 2's largest unstated prerequisite", and Milestone 2's exit criterion already requires an einops-derived chain with reindex plus pointwise fusion. No ticket in the graph delivers either family; the matrix that names them is a visibility ledger whose own work record states that it "does not itself authorize implementing any operation".

## Required delivery

One vertical, not one ticket per crate. It must carry:

- **Semantic identity and validation.** Governed `OpKey`s with canonical attribute schemas. `Reindex` validates totality of the output-to-input coordinate function over the declared output domain and rejects any mapping outside the admitted initial forms — in particular a non-bijective one, which is a slice and is a different family. `Broadcast` validates that its axis mapping accounts for every output axis and that each many-to-one relation is explicit. Both reject rather than normalizing a malformed mapping.
- **Normative reference.** A `NormativeDefinitionRef` for each, stating the admitted mapping forms and, for `Reindex`, that it makes no claim that storage was transposed or copied.
- **Compiler and lowering capability.** An index-access lowering capability for each occurrence. Both are pure coordinate relations inside the bounded index vocabulary — `docs/ir.md` already describes a broadcast as a map that omits an iteration coordinate or maps it to zero, and a transpose as a coordinate permutation — so neither needs a new access class. A fusion role is required, because an operation family with no registered role yields no fusion legality at all.
- **Target realization.** Neither emits a structured-kernel operation; both compose into an access map. The deliverable is that a program containing them reaches a verified kernel, not that a new `BinaryOp` variant exists.
- **Bounded conformance evidence.** Equivalence against a materialized reference for each admitted mapping form, plus negative cases for a non-total mapping, a non-bijective `Reindex`, an implicit rank pad, and an extent-one stretch presented without an axis mapping. State exactly which forms and ranks the evidence covers.
- **The matrix row.** Update the structural row's rung and evidence in the same change, and correct absence check 2, which will stop returning what it currently returns.

## Non-goals

Slice, concatenate, gather, and scatter. The L2 derivation establishes that the selected workload needs none of the first two inside a layer, and gather has its own ticket. Widening `Reindex` to a non-bijective mapping to absorb a slice would silently change the family's normative semantics.

## Reconsideration trigger

Active now: the selected workload cannot express its normalization weights or its head layout without both families, and no alternative spelling exists. If the workload is superseded before this lands, re-derive the occurrence counts from the replacement rather than carrying these forward.

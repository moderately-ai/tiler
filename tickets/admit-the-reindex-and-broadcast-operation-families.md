---
id: admit-the-reindex-and-broadcast-operation-families
title: Admit the Reindex and Broadcast operation families
status: todo
priority: p1
dependencies: [derive-transformer-operation-and-shape-surface]
related: [own-operation-family-support-matrix, scope-transformer-nonlinear-normalization-and-reductions, design-attention-program-vertical, compose-rotary-position-embedding-from-reindex-and-broadcast, admit-the-grouped-query-head-layout-reindex-profile, reach-a-verified-kernel-through-the-structural-families]
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
- **Settle decision D-10 — whether "bijective permutation" covers a coordinate permutation *within* an axis, or only a permutation *of* axes.** Added 2026-07-31 by [the L4 attention design](../docs/research/program-planning/first-attention-program-vertical.md), which found the one structural requirement in the workload that this reading decides. `rotate_half` needs the coordinate map `(…, i, j) -> (…, 1 − i, j)` on a size-2 axis; every other structural need in the workload is a split, a merge, or an axis permutation. `docs/ir.md`'s wording reads most naturally as axis permutation, and the index-expression vocabulary separately admits `1 + (−i)`, so the access map is expressible and the open question is this family's admission rule. **Measurement — the composition itself is correct**: it reproduces the reference's `rotate_half` at 0 of 20,480 elements, with the swap removed and the sign reversed each differing at all 20,480, retained by [the attention-block probe](../spikes/program-planning/attention-block-reference/README.md). Answer it in the normative reference either way; a "no" means the workload needs one structural form beyond the four admitted today, and [`compose-rotary-position-embedding-from-reindex-and-broadcast`](compose-rotary-position-embedding-from-reindex-and-broadcast.md) is the consumer that fails closed without it.
- **Compiler and lowering capability.** An index-access lowering capability for each occurrence. Both are pure coordinate relations inside the bounded index vocabulary — `docs/ir.md` already describes a broadcast as a map that omits an iteration coordinate or maps it to zero, and a transpose as a coordinate permutation — so neither needs a new access class. A fusion role is required, because an operation family with no registered role yields no fusion legality at all.
- **Target realization.** Neither emits a structured-kernel operation; both compose into an access map. The deliverable is that a program containing them reaches a verified kernel, not that a new `BinaryOp` variant exists.
- **Bounded conformance evidence.** Equivalence against a materialized reference for each admitted mapping form, plus negative cases for a non-total mapping, a non-bijective `Reindex`, an implicit rank pad, and an extent-one stretch presented without an axis mapping. State exactly which forms and ranks the evidence covers.
- **The matrix row.** Update the structural row's rung and evidence in the same change, and correct absence check 2, which will stop returning what it currently returns.

## Non-goals

Slice, concatenate, gather, and scatter. The L2 derivation establishes that the selected workload needs none of the first two inside a layer, and gather has its own ticket. Widening `Reindex` to a non-bijective mapping to absorb a slice would silently change the family's normative semantics.

## Reconsideration trigger

Active now: the selected workload cannot express its normalization weights or its head layout without both families, and no alternative spelling exists. If the workload is superseded before this lands, re-derive the occurrence counts from the replacement rather than carrying these forward.

## Outcome

Both families are admitted through R5 — registered identity, validated mapping attributes, normative references, reference evaluators, a fusion role, and an index-access lowering capability each. Decision D-10 is settled. The R6 half of the target-realization deliverable is not delivered and is not deferrable inside this ticket; the reason and the remainder ticket are below.

### Decision D-10, settled in the normative reference

**Fact — the resolution.** `tiler::reindex-f32@1` admits a within-axis coordinate permutation in exactly one named form, `reverse-axis`, the map `i -> extent − 1 − i`. At extent two that is `i -> 1 − i`, the swap `rotate_half` performs. No other within-axis permutation is admitted, and one presented under any other name is refused as `reindex.form.unadmitted-kind` naming the boundary. The answer lives in the registered `NormativeDefinitionRef`, so a reader gets it from the definition rather than from a research record.

**Inference — the derivation, stated so it can be refuted rather than only disagreed with.** Four steps. *One*, refusing outright buys no invariant: the composition is measured correct and its access map is expressible in the accepted bounded index vocabulary, so a refusal would only send the workload to a slice plus a concatenate, two families with no normative contract anywhere in this corpus. *Two*, admitting the general reading admits what cannot lower: an arbitrary within-axis permutation is a permutation table whose size is the axis extent, undefined for a symbolic extent, and applying it is a tensor-data-derived index the index vocabulary rejects — so the family would admit at construction a mapping no lowering can produce. *Three*, the named form is not a narrowing of the affine class, it *is* that class: an affine within-axis map `i -> a·i + b` carries `{0, …, n−1}` onto itself only when the image is `n` consecutive integers, so `|a| = 1`; `a = 1` forces `b = 0` and is the identity, `a = −1` forces `b = n − 1` and is the reversal. *Four*, a within-axis rotation `i -> (i + k) mod n` is separately expressible, quasi-affinely, and is deliberately unadmitted, with a reachable named refusal recording that admitting it would need a modulus in canonical identity, a positivity proof, and a conformance row of its own.

**Fact — what this changes elsewhere.** [`compose-rotary-position-embedding-from-reindex-and-broadcast`](compose-rotary-position-embedding-from-reindex-and-broadcast.md) no longer fails closed; the slice row's second reconsideration trigger did not fire; and L2's "no slice and no concatenate is required inside a layer" stands without narrowing. `docs/ir.md`'s own sentence still spells the initial forms without the reversal, and that file is `contracts/foundation` — outside this ticket's scopes. Propagating the resolution into `docs/ir.md`, into L4's D-10 entry, and into the sequence-extending record's qualification is [`propagate-the-d10-resolution-into-the-contract-corpus`](propagate-the-d10-resolution-into-the-contract-corpus.md).

### The mapping vocabularies

**Fact — `Reindex`.** One named form per occurrence, carried as a strongly typed attribute: `permute-axes`, `split-axis`, `merge-axes`, `insert-unit-axis`, `remove-unit-axis`, `reverse-axis`. Totality and bijectivity are *proved* rather than asserted, because the result shape is derived from the operand's extents and a result exists only when the map is total over it and bijective onto the operand's domain. The two failure directions of a split are separate named rules: factors overshooting the axis are non-total, factors falling short are a **slice** and the diagnostic says so. A composition is a chain of occurrences, so a merge over non-adjacent axes is refused rather than folded into one attribute.

**Fact — `Broadcast`.** An explicit axis mapping with exactly one entry per result axis, over three relations: `from-operand`, `stretch-unit`, `replicate`. The two many-to-one relations are deliberately distinct — a rank pad consumes no operand axis and a unit stretch consumes one — and the pinned workload needs both, the normalization weights being rank pads and the rotary sign operand the one unit stretch. The mapping consumes every operand axis exactly once in ascending order, so a reordering is a reindex and a dropped axis is a reduction or a slice, each refused by name. A mapping of only one-to-one correspondences denotes no broadcast and is refused.

**Inference — nothing is normalized.** Every malformed presentation is refused under its own rule: an implicit rank pad is a source-count disagreement, an unstated extent-one stretch is an extent disagreement naming the relation that should have been written, and a relation that does not widen is refused so one relation has one spelling.

### Conformance coverage

**Fact — forms × ranks.** `crates/tiler-reference/tests/structural_conformance.rs` covers `permute-axes` at ranks two and four, `split-axis` at ranks one and two, `merge-axes` at ranks two and three, `insert-unit-axis` and `remove-unit-axis` at ranks one and two, and `reverse-axis` at ranks one and three; and for `Broadcast`, `from-operand`, `replicate` leading, interior, and repeated, and `stretch-unit`, at ranks one through four. Every operand is `tiler::f32@1` and every extent is static. Each expectation is a literal permutation derived by hand from the mapping's definition rather than by a second implementation of it.

**Fact — the four required negative cases fire, by diagnostic code.** Non-total mapping (`reindex.split.not-total`), non-bijective reindex (`reindex.split.not-surjective`), implicit rank pad (`broadcast.mapping.source-count`, and `broadcast.mapping.operand-axes-unconsumed` at an occurrence), extent-one stretch without an axis mapping (`broadcast.mapping.extent-disagreement`). Each sits beside its admitted neighbour, so the checks are demonstrated discriminating rather than uniformly refusing.

**Measurement — five deliberate perturbations, each watched failing.** A reference stretch reading the result coordinate, a reference reversal reduced to the identity, a compiler split with strides computed major-first, a compiler merge decode with strides reversed, and the semantic split's two classifications swapped. Every one produced a failing test; the reverted tree is green.

**Boundary.** No symbolic extent, no rank above four, no dtype but F32, and no executed or compiled realization. A pass is evidence about the semantic contract, the reference evaluator, and the emitted index region — not about a plan, a kernel, or a target.

### What is not delivered, and why it is not a descope

**Fact — the target-realization deliverable's R6 half is blocked upstream of these families.** `select_supported_strategy` in `crates/tiler-compiler/src/request.rs` recognizes exactly two whole-program shapes; `NormalizedProgram::serial_sum` panics on any other variant, and the `pointwise()`-else-`serial_sum()` shape recurs across `physical.rs`, `frontier.rs`, `selection.rs`, and `program.rs`. The compiled-program profile additionally declares one external input, while the workload's broadcast occurrence — `Multiply(activation, Broadcast(weight))` — is inherently two-input. Neither limit is a property of these families: the contraction family is blocked by the same recognizer at the same boundary, and the support matrix already assigns that limit to [`prototype-optimizer-conformance-gate`](prototype-optimizer-conformance-gate.md).

**Inference — the alternative would violate this ticket's own non-goals.** Reaching a kernel through a *standalone* structural program needs a `ScalarProgram` copy variant and a materializing copy kernel, which is the "new `BinaryOp` variant" outcome the deliverable explicitly rules out. The correct shape is a fused region where the structural occurrence contributes an access map and an arithmetic neighbour contributes the scalar program, and no recognized program shape contains one. Filed as [`reach-a-verified-kernel-through-the-structural-families`](reach-a-verified-kernel-through-the-structural-families.md), which states the exact call sites.

**Fact — what stands in its place.** Both lowering capabilities are registered and *refine*, and `governed::tests` executes each emitted region on the independent index-region oracle against hand-derived expected permutations — including the exceptional-payload case proving neither family rewrites a NaN it only transports. `fusion_legality::tests::a_region_containing_both_structural_families_derives_legality` derives legality for a region containing both plus a multiply, with the role withdrawn as the perturbation. The access-map claim is therefore verified; the kernel claim is not made.

### Identity movements

**Fact.** The explain request digest was rebaselined from `e1e95ea1d50a918f` to `bddeaf899938ede4`, with a comment recording that both halves of the subject moved this time — the semantic snapshot admits two further families, and unlike the contraction the lowering registry admits a capability for each, so the lowering-registry identity moved too. `FusionRegionStructure` gained a `coordinate_relations` count, which changes fusion-legality content identity for every region; it is counted separately so the four role counts sum to `members`. The `tiler.standard-semantics` and `tiler.standard-reference` provider revisions were deliberately **not** bumped, following the precedent this ticket's base commit records for the dtype catalog and the contraction.

---
id: admit-the-attention-contraction-structures
title: Admit the attention score and value contraction index structures
status: todo
priority: p1
dependencies: [admit-the-contraction-normative-reference]
related: [design-attention-program-vertical, admit-the-contraction-semantic-profile, realize-the-attention-contractions-on-metal, implement-parallel-reduction-strategies, own-operation-family-support-matrix]
scopes: [implementation/ir, implementation/reference, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, contraction, attention, language-model, identity]
---
## User-visible outcome

A program can state the two contractions that make attention attention: `grtd,gsd->grts`, which scores sixteen query heads against eight key heads without materializing the repetition, and `grts,gsd->grtd`, which composes the values over a contracted extent that grows during decode. Together with the projection structure already owned by [`admit-the-contraction-semantic-profile`](admit-the-contraction-semantic-profile.md), this completes all three of the index structures the pinned workload contains.

## Why it is separate from the projection structure

**Inference — from the [L4 design](../docs/research/program-planning/first-attention-program-vertical.md).** The projection profile deliberately stopped at structure 1, because L3 could not schedule an operand whose production was undefined. The L4 prefill block defines that production: at `S = T` the block computes its own `K` and `V`, so both structures have ordinary program-internal operands and neither waits on the KV-state model. Three obligations arrive with them that structure 1 never exercised, and each is why this is a ticket rather than a wider extent range on an existing one:

- **A free index appearing in one operand and the output.** `r` — the grouped-query repetition — is in the query operand and the result and never in the key operand. That is the first index in this workload whose access map drops it from one operand, and it is exactly what makes the 8→16 repetition free rather than a `[16, S, 128]` broadcast.
- **A five-index structure.** Structure 1 has three indices; structures 2 and 3 have four and five. The renaming-invariant canonical encoding and the five structural rules ADR 0087 fixes must be exercised at that width, including the mutation proof.
- **A symbolic contracted extent.** Structure 3 contracts over `S`, the workload's only growing extent. Structure 1 contracts over a static `D_in` at every occurrence.

## Evidence prerequisite

**Fact — the canonical index structures, from the [L2 derivation](../docs/research/shapes/transformer-operation-and-shape-surface.md).** Structure 2 is `[8, 2, T, 128] × [8, S, 128] -> [8, 2, T, S]` at 28 occurrences per forward pass; structure 3 is `[8, 2, T, S] × [8, S, 128] -> [8, 2, T, 128]` at 28. Both pass the five structural admission rules: no output index absent from every operand, no summed index in only one operand, no index repeated within one operand, each output order a duplicate-free permutation of the free indices, and no index in more than two operands.

**Measurement — the index structure denotes the reference's computation, and the F32 disagreement is reduction order rather than structure.** At the C1 prefill shape the `grtd,gsd->grts` spelling and the reference's repeat-then-matmul differ at 943 of 1,600 F32 elements with a maximum absolute gap of 1.72 × 10⁻⁵, and agree at **0 of 1,600** when both are evaluated in float64 and rounded once. The [attention-block probe](../spikes/program-planning/attention-block-reference/README.md) retains the counts. **Inference.** So two spellings of one structure that no permission distinguishes still return different bits, which is why the order contract must be stated on the structure rather than left to a realization.

**Fact — the contracted extents.** Structure 2 contracts over the static 128. Structure 3 contracts over `S`, which is 10 at the C1 prefill row, up to 18 across C1's decode, and up to 8,320 at B1-d. Structure 3's fold is therefore the longest accumulation in the whole workload — longer than the 1,024-to-3,072 contributor counts the [L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md) recorded as the longest under its own profile — and it accumulates probabilities in `[0, 1]` summing to approximately one, which is a different conditioning problem from a weight-activation dot product. That evidence belongs to decision D-6 and to [`implement-parallel-reduction-strategies`](implement-parallel-reduction-strategies.md).

## Required delivery

- **Two structure values under the one keyed family**, never two keys: ADR 0087 accepts a single family whose node carries the structure as a strongly typed attribute. The canonical encoding is renaming-invariant and must carry the mutation proof at four and five indices — a perturbation that makes two distinct structures encode equally, or one structure encode two ways, demonstrated failing before the encoder is trusted.
- **The five structural refusals at this width**, each under its own named rule, with a malformed structure never reaching identity, planning, explain output, or a cache subject.
- **Extent agreement through the accepted three-outcome path**: `128` between the operands of structure 2, and `S` between the operands of structure 3, with both bindings surviving so a failure reports both observed sources.
- **The reduction signature per structure**, parameterized by the structure as ADR 0087 item 5 requires: `tiler::f32@1` operands, accumulator, and result; the contributor sequence ascending over the contracted index; **no seed**, so the accumulator starts at the first product; `FlushSubnormalsToZeroF32`; the canonical arithmetic NaN; reassociation, permutation, and ADR 0015 contraction all Forbidden under the governed contract.
- **A `tiler-reference` evaluator for both structures**, bit-compared against the strict fold, with the signed-zero seed case among its tests — for structure 3 that case is reachable from ordinary data, because a masked position contributes `+0.0 × v`, which is `-0.0` wherever `v` is negative.
- **The empty-domain declaration.** Structure 3's contracted extent is a symbol; `S = 0` is statically unreachable in this workload and the family still owes a declared behaviour, because the extent is an attribute and not a proof.
- **The matrix row.** The contraction row of the [support matrix](../docs/roadmap.md#operation-family-support-matrix) moves only as far as the delivered layers actually reach, and this ticket delivers semantics and reference and nothing below them.

## Non-goals

Any schedule, any realization, any cost, and the KV cache. Operands arrive as ordinary program values at `S = T`; the cached-operand form is [`design-autoregressive-state-and-kv-cache`](design-autoregressive-state-and-kv-cache.md)'s and needs no change to the structures admitted here. A batched or multi-operand contraction form is also out: every occurrence in this block is binary.

## Closes when

Both structures verify, refuse malformedness under named rules, and reference-evaluate bit-identically to a strict ascending fold at the C1 prefill extents, with the canonical encoding's mutation proof retained.

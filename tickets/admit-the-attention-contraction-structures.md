---
id: admit-the-attention-contraction-structures
title: Admit the attention score and value contraction index structures
status: in-progress
priority: p1
dependencies: [admit-the-contraction-normative-reference]
related: [design-attention-program-vertical, admit-the-contraction-semantic-profile, realize-the-attention-contractions-on-metal, implement-parallel-reduction-strategies, own-operation-family-support-matrix]
scopes: [implementation/ir, implementation/reference, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, contraction, attention, language-model, identity]
claimed_from: todo
assignee: worker-attn-structs
lease_expires_at: 1785563718
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

## Outcome

**Fact — nothing was widened; admission was a matter of evidence.** Derived before writing any code, by reading both foundations in full. `ContractionIndexStructure` and `StrictTensorContractionF32::infer` in `crates/tiler-ir/src/semantic/contraction.rs` are already generic over an arbitrary admitted binary structure: the five rules are decided on the supplied tuples with no arity or width special-casing, the canonical numbering scans every operand tuple then the output, and extent agreement zips the structure's tuples against the operands. `contract_operands` in `crates/tiler-reference/src/contraction.rs` is likewise generic, as its landing report claimed — verified rather than trusted. So no inference change, no evaluator change, and no registration were required or made, and the two structures are structure *values* under the one key ADR 0087 accepts. What this ticket delivers is the corpus that proves it at the new width, plus the support-matrix row.

**Fact — a correction to this ticket's own text.** "Structures 2 and 3 have four and five" indices is not what the structures derive. Both have **five distinct indices** (`g, r, t, d, s`) and **four-wide** operand-0 and output tuples, against structure 1's three indices and two-wide tuples. The delivery is stated and tested as four-wide tuples at five indices.

### The canonical derivation, and the collision it makes reachable

| structure | operand 0 | operand 1 | output | contracted |
| --- | --- | --- | --- | --- |
| 2 — `grtd,gsd->grts` | `(0, 1, 2, 3)` | `(0, 4, 3)` | `(0, 1, 2, 4)` | `{3}` (`d`) |
| 3 — `grts,gsd->grtd` | `(0, 1, 2, 3)` | `(0, 3, 4)` | `(0, 1, 2, 4)` | `{3}` (`s`) |

The two agree on operand 0, on the output, and on the contracted set, and differ **only** in whether operand 1 reads `(g, s, d)` as `(0, 4, 3)` or `(0, 3, 4)` — because `s` is new at its first appearance in structure 2 and already numbered in structure 3. That is the sharpest identity hazard in the workload and it does not exist at two-wide tuples.

### Mutation proof at four-wide tuples and five indices

Three perturbations in `the_canonical_encoder_is_mutation_proved_at_four_and_five_indices`, each a property of a deliberately broken twin rather than of the shipped encoder.

1. **Operand-tuple sorting collides the two attention structures.** With each tuple sorted before encoding, structures 2 and 3 encode to identical bytes. Watched failing against the shipped encoder by sorting inside `encode_structure`: 12 of 32 contraction tests failed, including `both_attention_structures_canonicalize_by_first_appearance` reporting the two encodings byte-identical.
2. **Flattening the operand framing collides two admitted five-index structures.** `abc,deac->abde` and `abcd,eac->abde` flatten to one index run `0,1,2,3,4,0,2` with the same output and contracted set, and are separated only by the framing.
3. **Dropping the renumbering makes one five-index structure encode two ways** — structure 2 spelled with workload labels versus dense labels.

### The five refusals at this width, each watched failing

Each rule was disabled in `derive_contracted` in turn; `every_structural_rule_refuses_at_four_wide_tuples_too` failed every time, at that rule's own assertion.

| rule | width-4 malformed structure | code | with the rule disabled |
| --- | --- | --- | --- |
| 1 output index in no operand | `grtd,gsd->grtx` | `contraction.rule.output-index-in-no-operand` | falls through to rule two |
| 2 summed index in one operand | `grtd,gs->grts` | `contraction.rule.summed-index-in-one-operand` | admitted, result inferred |
| 3 index repeated within an operand | `grtt,gsd->grts` | `contraction.rule.index-repeated-within-operand` | falls through to rule two |
| 4 duplicated output index | `grtd,gsd->grtt` | `contraction.rule.duplicate-output-index` | falls through to rule two |
| 5 index in more than two operands | `grtd,gsd,gsd->grts` | `contraction.rule.index-in-more-than-two-operands` | falls through to `contraction.structure.operand-count` |

Rule five's fall-through confirms the module's own claim that it is decided on the structure's operand count *before* the exact-arity schema, or it could never fire.

**The free index is asserted explicitly.** `the_grouped_query_repetition_index_is_free_in_one_operand_and_the_output` states that `r` is in the query operand and the result, in neither the key operand nor the contracted set — the first such index in this workload, and the whole of what makes the eight-to-sixteen repetition free rather than a `[16, S, 128]` materialization (10,240 key elements against the repeated 20,480).

### Extent agreement, and the limitation restated

Structure 2's `d` = 128 and structure 3's `S` both resolve through the accepted three-outcome path, and every disproof names both observed operand axes — including the head-dimension case where the declared 128 meets the divided `hidden_size / num_attention_heads` reading of 64. **The unresolved third outcome remains unreachable, exactly as the projection profile's landing recorded:** a semantic `ValueFact` carries a static `Extent`, so structure 3's growing extent is exercised at the static values C1 takes — `S` ∈ {10, 16, 18} — and never as a symbol. Stated in the test, in the new reference file's boundary section, and in the matrix row.

**Empty domain.** `the_value_structure_refuses_a_zero_context_length` shows `S = 0` refused under `contraction.extent.empty-contracted-domain`, with a zero *free* extent admitted as the contrasting case; watched failing by disabling the zero-extent check.

### Reference evidence

`crates/tiler-reference/tests/attention_contraction_structures.rs`, five tests, all through the public builder and evaluator.

**Measurement — the repeat-then-matmul comparison, at the C1 prefill shapes.** Structure 2 against per-query-head `td,od->to` on repeat-interleaved keys: **0 of 1,600** differing. Structure 3 against the same oracle with the value head transposed: **0 of 20,480** differing, at each of `S` = 10, 16, 18.

**Fact — the retained 943 is not reproducible here, and that is the finding rather than a gap.** The probe's `score_structure_einsum_differing_elements = 943` of 1,600 (max absolute gap 1.72e-5) compares `torch`'s einsum kernel against `torch`'s matmul kernel — two *undeclared* reduction orders — on operands from a `torch` seed this crate cannot reproduce, and the probe's own README states only the zero-and-nonzero counts generalize past that seed. Its `score_structure_f64_differing_elements = 0` is the structural claim: one computation, two spellings. Under this family's **declared** contributor order both spellings fold the identical contributor sequence, so the F32 count is 0 — the contract removing the disagreement the probe measured, which is precisely why the order is stated on the structure rather than left to a realization. Reproducing 943 would require an evaluator with no declared order.

**Measurement — the comparison discriminates, watched failing three ways.**

- Perturbing the evaluator so operand 1 always reads group 0: **1,400 of 1,600** score elements and **17,920 of 20,480** value elements differ — the latter matching the probe's own `gqa_repeat_kv_matches_modulo_differing_elements = 17920`.
- The live in-test perturbation reads the key head as `h % 8` instead of `h / 2`: exactly **14 of 16** query heads differ, reproducing the probe's `gqa_heads_whose_source_differs_between_the_two_readings = 14`.
- Reversing the evaluator's contributor order: caught by `both_structures_reproduce_a_fold_written_from_the_index_letters` (19 elements) and **not** by the repeat-then-matmul comparison, since both sides reverse identically. That asymmetry is why both oracles exist, and it is recorded in the file.

**Signed zero, reachable from ordinary data.** `a_masked_position_contributes_a_signed_zero_to_the_value_contraction` drives structure 3 with an attention row of `1.0` followed by exact `+0.0`, against a value whose lane-0 entry is `-0.0`. Both outcomes are asserted, so the sign is discriminated rather than observed once: all-negative contributors give `0x80000000`, and a single positive value at a masked position rewrites it to `0x00000000`, because `fl(-0.0 + +0.0)` is `+0.0`. This reproduces the probe's mechanism (`first_product = 0x80000000`, `masked_contributor_signs = 0x00000000 0x80000000 0x80000000`, `strict_ascending_fold = 0x00000000`) with designed operands. Watched failing by seeding the accumulator at `+0.0`: it failed alongside the two pre-existing seed regressions.

**Independence boundary, stated rather than implied.** The repeat-then-matmul oracle is genuinely independent — a different structure, a different program, an explicit materialization. The hand-written fold restates the *access relation* independently and runs the same binary32 arithmetic in the same declared order, so it discriminates a wrong index binding and is silent about the arithmetic; the transcribed exceptional-value corpus in `contraction_conformance.rs` covers that instead. Operands are a local SplitMix64 generator, not the checkpoint and not `torch`; the asserted counts are the data-robust ones.

### Support matrix

The contraction row moved **R3 → R4** and no further: semantics and reference, nothing below. The row was **already stale at R3** — it read "No reference evaluator" although `admit-the-contraction-normative-reference` registered one, because that ticket held `implementation/reference` and `implementation/ir` but not `contracts/navigation` and so could not move its own row. Corrected here along with three assertions the change made inconsistent: the ladder paragraph's "it sits at R3 … with no evaluator", the section preamble's reference-admission list (which omitted the contraction), and `docs/open-questions.md`'s "records this family at R1", which was stale by two rungs before this ticket.

**Out of scope, flagged not fixed:** `docs/research/scheduling/first-metal-contraction-realizations.md:19` also says "the row sits at R3". That file is `contracts/scheduling-research`, not `contracts/navigation`.

### Identity, public surface, and verification

**Fact — no identity moved.** No key, schema, signature field, or provider revision changed; `cargo nextest run -p tiler-compiler` is green unchanged at 507 tests and the explain request digest needed no rebaseline, which is what a ticket that registers nothing should produce.

**Public surface: none.** No public item was added, removed, or changed in either crate. No workload-named constructor was added, following the grouped-query head-layout precedent deliberately: `grtd` is a consumer's reading of an index tuple, and the compiler's semantic model does not learn it. The two constructors are private to the tests that use them.

**Commands.** `cargo fmt`; `cargo nextest run -p tiler-ir -p tiler-reference -p tiler-compiler`; `cargo test --doc`; `cargo clippy -p tiler-ir -p tiler-reference -- -D warnings`; `tkt lint`; `git diff --check`; `tkt guard --base 448ecd6`; `make full`.

**Base note.** The dispatched worktree was checked out at `59d232f`, one commit *behind* the named base `448ecd6` (it missed only this ticket's own claim commit) with zero unique commits and a clean tree; fast-forwarded to `448ecd6` before any edit.

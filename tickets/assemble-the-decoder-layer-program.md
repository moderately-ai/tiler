---
id: assemble-the-decoder-layer-program
title: Assemble the complete decoder-layer program
status: done
priority: p1
dependencies: [assemble-the-causal-self-attention-block-program, admit-the-silu-activation-family, admit-the-sequence-extension-concatenate-family, reclassify-language-model-work-as-a-conformance-track]
related: [design-model-ingestion-and-complete-execution, design-attention-program-vertical, design-autoregressive-state-and-kv-cache, widen-the-deterministic-budgets-to-the-decoder-layer-program, decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode, cut-the-decoder-layer-reference-evaluation-s-suite-wall-clock]
scopes: [implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, transformer, program, language-model, class-conformance-fixture]
---
## User-visible outcome

One complete decoder layer of the pinned checkpoint — attention, MLP, both residuals, and the two cache extensions — is a single verified semantic program that reference-evaluates against the pinned reference.

## Required content

The program [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md) calls P2:

- **Eighteen ordered inputs.** [L4's twelve](../docs/research/program-planning/first-attention-program-vertical.md) — `x`, `w_input_layernorm`, `W_q`, `W_k`, `W_v`, `w_q_norm`, `w_k_norm`, `cos`, `sin`, `rope_sign`, `mask`, `W_o` — plus [L5's two](../docs/research/runtime/autoregressive-state-and-kv-cache.md) `k_cache` and `v_cache`, plus the MLP's `w_post_attention_layernorm`, `W_gate`, `W_up`, and `W_down`.
- **Three ordered named outputs.** `h_out [T, 1024]`, `k_rope [8, S, 128]`, `v_heads [8, S, 128]`.
- **The MLP half is `down(silu(gate(x)) * up(x))`** over `[T, 3072]` intermediates, which L2 derived introduces no family its constituents do not, plus the second residual add.
- **The two concatenations are at the block boundary**, exactly where L5 places them: L4's steps 13 and 14 each feed one, and the concatenation's result is the retained output the score and value contractions read.

## The counts this ticket must produce

They are an input to [`widen-the-deterministic-budgets-to-the-decoder-layer-program`](widen-the-deterministic-budgets-to-the-decoder-layer-program.md) and must be measured rather than estimated: the program's exact `value_count()`, `operation_count()`, and `input_count()`. The L6 record's derived floors are at least fifty-one occurrences and at least twenty-one boundary values; a smaller number is a real result and a larger one is too, and either replaces the floor.

## Closes when

The program verifies, reference-evaluates against the pinned reference at the C1 prefill shape and at one decode shape, the three outputs are ordered and named, and the exact counts above are recorded in this ticket's outcome.

## Do not

Do not compile it. This ticket assembles and reference-evaluates; the deterministic budgets refuse the compilation and widening them is a separate, identity-moving decision that belongs to Tom.

## Outcome

Landed as `crates/tiler-reference/tests/decoder_layer.rs`: the complete decoder layer, twenty-nine steps over **ten** already-registered keys, built through the public builder, with eighteen ordered inputs and three ordered named outputs (`h_out`, `k_rope`, `v_heads`). Nothing was registered, admitted, or widened, and nothing was compiled — the layer is a shape over `tiler::rms-norm-f32@1`, `tiler::strict-tensor-contraction-f32@1`, `tiler::reindex-f32@1`, `tiler::broadcast-f32@1`, `tiler::multiply-f32@1`, `tiler::add-f32@1`, `tiler::constant-f32@1`, `tiler::softmax-f32@1`, `tiler::concatenate-f32@1`, and `tiler::silu-f32@1`.

### The counts, measured

**Measurement — `the_layer_verifies_at_the_c1_prefill_row` and `the_layer_verifies_at_the_c1_decode_row`, 2026-08-05.**

| Row | `input_count()` | `operation_count()` | `value_count()` |
| --- | --- | --- | --- |
| C1 prefill, `T = 10, C = 0, S = 10` | 18 | **58** | **76** |
| C1 decode step 8, `T = 1, C = 17, S = 18` | 18 | **62** | **80** |

Both replace the L6 record's derived floors of "at least fifty-one occurrences over at least twenty-one boundary values", and both are larger. The value count is the eighteen inputs plus one result per occurrence, because no occurrence in this layer produces more than one value; the test asserts that identity rather than only the total. Occurrences by key at the prefill row: 5 `add`, 11 `broadcast`, 2 `concatenate`, 1 `constant`, 8 `multiply`, 16 `reindex`, 4 `rms-norm`, 1 `silu`, 1 `softmax`, 9 `strict-tensor-contraction`.

**These are the numbers [`widen-the-deterministic-budgets-to-the-decoder-layer-program`](widen-the-deterministic-budgets-to-the-decoder-layer-program.md) needs, and the decode row's are the binding ones**: `semantic_values` 16 against 80, `semantic_operations` 8 against 62, and `buffers` 4 against a `4.max(input_count + 1)` actual of 19. `DeterministicBudgets::governed` was not touched and no compilation was attempted, per the "Do not".

**Program identity is not the constraint.** The [identity-growth measurement](../spikes/program-planning/identity-growth/README.md) fits `program_bytes(n) = 134n² + 3650n + 710` and puts 51 occurrences at 535,394 bytes, 0.80% of `MAX_PROGRAM_IDENTITY_BYTES`; 62 is still far inside the same curve. Cited rather than re-derived.

### Delivered as stated

**Both rows verify and both reference-evaluate.** The prefill row at the C1 conformance row's own extents with nothing reduced — ten new positions, an empty `[8, 0, 128]` cache, a 1,024-wide model dimension, a 3,072-wide intermediate, sixteen query heads over eight groups, head dimension 128, the causal mask, the scale, the rotary composition, all three contraction index structures, and both residuals — at **0 differing elements on all three outputs** against an independent recomputation. The decode row at `T = 1, C = 17, S = 18`, likewise at 0 differing on all three, and additionally with the first seventeen context positions of `k_rope` compared against the bound cache tensor itself, which is the concatenation's bit-preservation stated against the operand rather than against the recomputation.

**The decode row is the row the attention block could not reach**, and it needed no widening: the prefill block computes its own key, so a mask asserting a wider context is refused at the mask add, while here the extension supplies the wider context. The difference is two program inputs and two occurrences.

**`S == C + T` is a retained relation, not a convention.** `T`, `C` and `S` are three separate `ShapeEnv` symbols, each bound to an input dimension, each bounded to the checkpoint's `max_position_embeddings` — `C` from zero, the other two from one — and joined by `ExtentRelation::additive_equality`. An inconsistent row is refused twice, independently: at `ShapeEnvBuilder::build` under `shape-env.unsupported-relation` with `FragmentViolation::UnderdeterminedAdditiveEquality { undetermined: 3 }` (all three terms are runtime-bound, so the relation is decided against their proved lower bounds), and again at the mask add under `binary.shape` for a caller that assembled the extents by hand.

**Every new check was watched failing under a deliberate perturbation, each beside an admitted neighbour.** The inconsistent row at both refusal sites; a `[16, C, 128]` cache against the eight heads the key path produces, under `concatenate.operands.extent-disagreement`; a position-axis rank pad at one new position, under `broadcast.mapping.relation-does-not-widen`, at all three of the layer's widenings; the grouped-query head reading `h % 8` for `h / 2`, which moves the residual and leaves the two retained outputs at 0 differing; the MLP's gating `silu(up) * gate` for `silu(gate) * up`, same shape and same occurrence count, which moves the residual and not the KV seam; the extension's operand order, which moves every one of eighteen context positions against a nonempty cache and **provably moves nothing against an empty one** — so the prefill row is exactly the row that cannot discriminate the two, which is why the decode comparison exists; and the reference work bound, refused at the default and at one step below the layer's own largest fold, quoting both step counts.

**Pinned-reference comparison, with its boundary.** The [attention-block probe]'s retained `row_h0_t2_scores_raw` is driven through this layer's own operations 16, 17 and 18 under this layer's own `mask_mapping` at the workload's eight groups and two repetitions, and reproduces `row_h0_t2_scores_scaled`, `row_h0_t2_scores_masked` and `row_h0_t2_probs` **bit for bit**; the generated mask's row 2 is compared against the record's `mask_row_t2`. **The record holds no MLP, cache, or decode observable** — its keys are the rotary composition, the grouped-query reading, the mask, the score chain, the softmax rows and the eager attention output — and the C1 conformance fixture retains per-layer hidden states as regenerable local data rather than in-tree bits. So the MLP half and both rows' end-to-end results are compared against the independent recomputation, whose own boundary is that the three non-linear families' scalar arithmetic is the crate's certified `rms_norm_f32`, `softmax_f32` and `silu_f32` rather than a second implementation.

[attention-block probe]: ../spikes/program-planning/attention-block-reference/README.md

### Two findings the records did not have

**The decode row is a different graph, and the cache is not why.** At a fixed `T` the layer built against `C = 0` and `C = 8` has an identical occurrence signature — same families in the same order, same forms, same structures, same broadcast relations — so the sequence extension is exactly the binding change L5 says it is. But at `T = 1` six position-axis rank pads duplicate nothing, `tiler::broadcast-f32@1` refuses a many-to-one relation onto an extent-one result axis, and the layer carries 62 occurrences instead of 58. That contradicts the reading of L5's P2 and L6's "270 executions over exactly three artifact identities" under which one graph serves both phases. Filed as [`decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode`](decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode.md) with the mechanism, the four candidates, and the exact sentences in both records that would move; nothing in either record was edited from here, and the scopes to do so are not this ticket's.

**The prefill evaluation is the workspace's new critical path.** 71.0s for one test, against the 132 ms [`audit-the-suite-s-slowest-tests`](audit-the-suite-s-slowest-tests.md) left the previous dominant one at. Filed as [`cut-the-decoder-layer-reference-evaluation-s-suite-wall-clock`](cut-the-decoder-layer-reference-evaluation-s-suite-wall-clock.md) with the per-test timing table and four unmeasured candidates. The row's extents are not on the table there: a cheaper test at a narrower model dimension checks a different program. Within this ticket the three graph-local perturbations were moved to the decode row, which exercises the identical head geometry, intermediate width, families and forms at one new position instead of ten, cutting the file from 343s to 71s with no check removed.

### Measurement boundary

Everything here is the semantic layer and the reference evaluator on one host, at two rows of one conformance workload, over synthetic operands at a recorded seed. Nothing is established about a plan, a schedule, a cover, a kernel, a device, artifact identity bytes, or any layer-level numeric tolerance — the last is deliberately not composable from per-operation tolerances. The pinned-reference comparison covers the three operations the retained record covers and no more.

### Commands run

`cargo fmt --all --check`; `cargo check --workspace --all-targets`; `cargo clippy -p tiler-ir -p tiler-reference --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-ir -p tiler-reference --no-deps`; `cargo test -p tiler-ir -p tiler-reference --doc`; `cargo nextest run -p tiler-ir -p tiler-reference` — 1032 passed, 2 skipped, 78.9s; `cargo nextest run -p tiler-reference --test decoder_layer --profile timing` — 18 passed, 71.0s.

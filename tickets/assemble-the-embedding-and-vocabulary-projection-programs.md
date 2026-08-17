---
id: assemble-the-embedding-and-vocabulary-projection-programs
title: Assemble the embedding and vocabulary-projection programs
status: done
priority: p1
dependencies: [admit-a-storage-carrier-for-integer-program-inputs, admit-the-rms-normalization-family, reclassify-language-model-work-as-a-conformance-track, admit-an-indirect-gather-family-for-tied-embedding-lookup, admit-the-contraction-semantic-profile]
related: [design-model-ingestion-and-complete-execution, project-only-the-final-position-logits]
scopes: [implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, semantics, gather, logits, language-model, class-conformance-fixture]
---
## User-visible outcome

The two ends of the model exist as programs: token IDs become a residual stream, and a residual stream becomes logits — so what the consumer hands over is token IDs and what it receives is logits.

## Required content

The two programs [the L6 record](../docs/research/program-planning/complete-model-ingestion-and-execution.md) calls P1 and P3.

**Graph correction — 2026-08-09.** The ticket originally left the gather and contraction semantic families under `related` even though P1 and P3 cannot be constructed without them. Both have landed and are now dependencies: [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](admit-an-indirect-gather-family-for-tied-embedding-lookup.md) owns P1's operation and bounds refusal, and [`admit-the-contraction-semantic-profile`](admit-the-contraction-semantic-profile.md) owns P3's projection operation. The outstanding integer-storage decision remains the only non-terminal family/carrier dependency.

**Graph repair — 2026-08-10.** Gather and contraction remain in `dependencies` only; they were dropped from `related` so the same ids are not double-listed once required.

- **P1 — embedding gather.** Inputs `token_ids [T]` at the admitted integer identity and `W_embed [151936, 1024]` F32; output `x0 [T, 1024]`; one operation.
- **P3 — final norm and vocabulary projection.** Inputs `h [T, 1024]`, `w_norm [1024]`, and the same `W_embed`; output `logits [T, 151936]` F32. L6's abstract cut is two operations, `model.norm` then `Contract td,od->to`. A constructible program must count the explicit `w_norm` widening (broadcast or reindex by `T`) before `rms_norm`, because `tiler::rms-norm-f32@1` refuses a `[1024]` weight against `[T, 1024]` with no implicit broadcast — so occurrence counts at C1 prefill and decode will be greater than two and must be recorded as measured.
- **The tied matrix is one consumer-owned tensor bound to both programs.** The checkpoint carries no `lm_head.weight`, so the same `[151936, 1024]` tensor is the gather's table and the projection's weight. P1 and P3 necessarily carry distinct program-local values; the evidence must instead bind one exact consumer-owned tensor subject at both input interfaces. Both bindings are read-only, so no aliasing question arises; what must be recorded is that binding two copies costs 622,329,856 bytes and nothing below the consumer can detect it.
- **The token-ID bounds obligation has a named enforcement boundary.** At the reference boundary delivered by [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](admit-an-indirect-gather-family-for-tied-embedding-lookup.md), an index outside the gathered extent refuses as `ReferenceOperationError::GatherIndexOutOfBounds`, naming its position, value, and extent. It never clamps, never wraps, and never reads out of bounds. This ticket claims no physical enforcement boundary.
- **The logits contract is what the output means.** Unnormalized, every position, no softmax, no scale, no vocabulary mask, no argmax, no token. The final-position mode is a different program shape and belongs to [`project-only-the-final-position-logits`](project-only-the-final-position-logits.md).

## Both programs fit today's budgets, and that is worth checking rather than assuming

P1 declares two inputs and P3 three. Governed `buffers` is **30**, and `check_program_budgets` derives the buffers actual as `input_count + output_count * 4`, refusing only when the actual **exceeds** the limit (so P1's actual is 6 and P3's is 7 if each has one output). Value and operation budgets are similarly far inside today's governed limits. Verify the measured `input_count` / `operation_count` / `value_count` (and buffers actual if `verify_program` is exercised) rather than inherit them — if either program fails a real budget, the budget widening ticket's sizing evidence changes.

## Fact audit — 2026-08-10 at base `c99ac54950f2`

- The prior buffers claim `check_budget("buffers", 4, 4.max(input_count + 1))` never matched source (same defect class L6 already corrected under output-derived budgets). Implemented path: `check_program_budgets` / `DeterministicBudgets::governed` in `crates/tiler-compiler/src/request.rs`.
- P3 "two operations" is the L6 abstract cut only; constructible programs insert the explicit weight widening before RMS (see decoder-layer `widen_hidden_weight` pattern).

## Exact-base Fact repair — 2026-08-17 at base `b3f0ada383b6a6ac332dd0b3e115b89aa522e901`

- **False — literal C1 reference evaluation.** `crates/tiler-reference/src/lib.rs`, anchor `MAX_REFERENCE_TENSOR_ELEMENTS`, bounds one tensor at `16 * 1024 * 1024` elements. The `[151936, 1024]` matrix contains 155,582,464 elements, and `Tensor::dense`, anchor `if expected > MAX_REFERENCE_TENSOR_ELEMENTS`, refuses it. `crates/tiler-reference/tests/gather_conformance.rs`, anchor `The pinned model's own [151936, 1024] extents are exercised only as a shape`, already states this boundary. P3's literal contraction also asks 155,582,464 multiply-accumulate steps at `T = 1` and 1,555,824,640 at `T = 10`, beyond the default per-occurrence allowance. The exact C1 programs must therefore be constructed and inspected without materializing their inputs; reference evaluation uses an explicitly extent-independent structural analogue and makes no claim that it materializes C1.
- **Imprecise — one value across two programs.** `SemanticProgramBuilder` gives every program its own graph owner, so a `ValueId` cannot cross from P1 to P3. `InputBinding::new`, however, borrows a consumer-owned `Tensor`; the bounded fixture can and must bind the same tensor subject at both programs while proving their program-local values remain distinct.
- **Imprecise — the bounds boundary.** The gather dependency's current outcome delivers `GatherF32Reference` as the named reference enforcement boundary and explicitly delivers no physical enforcement boundary. This ticket tests the exact `GatherIndexOutOfBounds { position, value, extent }` reference refusal and claims nothing about lowering or device execution.
- **Verified — current contraction reference authority exists.** The contraction dependency's statement that no reference evaluator existed is historical. `crates/tiler-reference/src/standard.rs`, anchors `ContractionContract::governed` and `strict_tensor_contraction_f32_op`, registers `StrictTensorContractionF32Reference` today.
- **Verified — exact construction needs no compiler scope.** `check_program_budgets` in `crates/tiler-compiler/src/request.rs` is private and derives buffers as `input_count + output_count * 4`; P1's actual is 6 and P3's is 7. Semantic construction supplies the requested exact input, operation, value, output, and result-shape evidence without claiming the recognizer, lowering, or runtime accepts either program.

## Implementation evidence — 2026-08-17

`crates/tiler-reference/tests/language_model_boundaries.rs`, anchor `exact_c1_prefill_and_decode_programs_have_the_measured_shapes_and_counts`, constructs and semantically validates literal C1 programs at both requested rows. P1 is two inputs, one operation, three values, one output, derived buffers actual 6, and result `[T, 1024]`. P3 is three inputs, three operations, six values, one output, derived buffers actual 7, and result `[T, 151936]`. At `T = 10` its exact key sequence is broadcast, RMS normalization, strict tensor contraction; at `T = 1` the explicit widening is a reindex unit-axis insertion followed by the same normalization and contraction. The contraction's second operand is asserted to be P3's own `W_embed` input. The exact operation list contains no softmax, scale, mask, argmax, or token-producing operation, and the single output is named `logits`, so it denotes every unnormalized position.

The same file's anchor `bounded_prefill_and_decode_analogues_use_one_tied_tensor_and_match_literal_results` evaluates P1 and P3 at `V = 3`, `H = 2`, for both `T = 2` and `T = 1`. One consumer-owned `Tensor` is borrowed at P1's gather-table and P3's projection-weight bindings; pointer identity proves the binding subject is the same, while unequal graph-owned `ValueId`s prove no value identity crosses programs. P1's expected rows are literal. P3's independent oracle starts from the retained RMS worked-example result `[0x3f593923, 0x4010d0c2]` for `[3, 4]` and weight `[1, 2]`, then projects through the hand-stated rows `e0`, `e1`, and `e0 + e1`, yielding the two values and the literal strict-add result `0x40471f0b`; a second all-zero row stays positive zero. This is extent-independent structural evidence and does not materialize C1.

The anchor `an_out_of_range_token_id_reaches_the_exact_named_gather_refusal` extracts `ReferenceOperationError::GatherIndexOutOfBounds { position: 1, value: 3, extent: 3 }` from the gather operation and pins its message: `gather index element 1 holds 3 and the gathered axis has extent 3, so it names no coordinate; an out-of-range index is refused rather than clamped to the axis or wrapped modulo its extent`.

Four independent subject perturbations were run with `cargo nextest run -p tiler-reference -E 'test(<name>)' --no-capture`, each restored before the next and followed by a green run of all three tests:

- **Gather index/OOB path:** replacing the out-of-range `3` with the last valid index `2` made the refusal check fail with `an out-of-range token ID must refuse: [Tensor(... shape: Shape([Extent(2), Extent(2)]) ...)]`.
- **Explicit RMS weight widening:** bypassing `widen_norm_weight` made construction refuse with `RMS normalization receives the explicitly widened weight` and provider code `rms-norm.f32.weight-shape`, whose message names the absent explicit `tiler::broadcast-f32@2` occurrence.
- **Contraction weight association:** swapping `normalized` and `embedding_value` at `F32TensorContraction::apply` made the structural check fail with `assertion left == right failed: td,od->to reads W_embed as its od operand`; the observed second operand was value index 4 rather than the `W_embed` input at index 2.
- **Tied tensor binding:** independently materializing equal embedding bytes for P3 made the fixture fail with `assertion failed: std::ptr::eq(p1_bindings[1].tensor(), p3_bindings[2].tensor())`.

Focused and affected-package evidence after restoration: `cargo fmt --all -- --check`; `cargo check -p tiler-reference --all-targets`; `cargo clippy -p tiler-reference --all-targets -- -D warnings`; three of three boundary tests; all 320 `tiler-reference` tests passed with two skipped; package rustdoc with `RUSTDOCFLAGS='-D warnings'`; and package doctests with warnings denied. No evaluator limit, public API, identity/schema/domain, compiler, lowering, artifact, runtime, Metal, model/checkpoint type, or execution support changed.

## Integrated outcome — 2026-08-17

The reviewed implementation commit `e5e1e66294dc8c4303647c34b660ac8c6a36736c` was integrated on `main` as `eb1d89febbd11518a1b7cb28b0dba23589776e9a`. Independent exact-commit review reported no findings at any severity and reproduced the RMS/projection bits plus all four subject perturbations. On the integrated tree, `make full` exited zero: citations, formatting, workspace all-target check, warnings-denied workspace Clippy, workspace nextest (3,670 tests with eight configured skips), doctests, warnings-denied rustdoc, the release numerical gate (1,262 tests with three configured skips), ticket lint, and shellcheck all passed. The outcome is therefore the bounded semantic/reference conformance fixture described above; compiler, lowering, artifact, runtime, Metal, and full C1 materialization remain explicitly unsupported here.

## Closes when

Both programs build as exact-shape semantic programs at C1 prefill `T = 10` and decode `T = 1`, with exact input, operation, value, output, result-shape, and derived-buffer counts recorded. Extent-independent bounded analogues reference-evaluate both programs with independently expected values; the same consumer-owned tensor is bound at P1's gather-table and P3's projection-weight interfaces without conflating their program-local values; and an out-of-range token ID reaches the exact named reference refusal. The evidence does not materialize the C1 embedding matrix, widen evaluator limits, or claim compiler, lowering, runtime, or device support.

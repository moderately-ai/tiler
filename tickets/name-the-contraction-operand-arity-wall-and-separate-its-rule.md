---
id: name-the-contraction-operand-arity-wall-and-separate-its-rule
title: Name the contraction declared-input arity wall and separate its rule
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler, research/program-planning, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: sol-contraction-arity
lease_expires_at: 1786245873
---
## User-visible outcome

The contraction recognizer's exactly-two-declared-input refusal carries its own stable public diagnostic key and its complete current reason, instead of sharing `input-arity` with the program-wide zero-input check. Admission, planning, numerics, and identity do not move.

## Fact audit — 2026-08-08 at `4be35e12`

**Verified:** `request.rs` has two live `return mismatch("input-arity");` sites: `select_supported_strategy` refuses a program with no declared input, while `normalize_contraction` refuses any declared-input count other than two. The latter is not an operand-arity check: ADR 0087 already fixes the semantic family as binary, and `structure.operand_count() != 2` has its own `contraction-operand-count` rule.

**False:** the normalizer says a third declaration has no ordinal. Program input ordinals can be gapped, but the pointwise subset-read landing did not remove this wall. The current exactly-two guard makes several dense assumptions coincide: `NormalizedContraction`'s fixed `input_keys`, `input_shapes`, `inputs`, `input_elements`, and `operand_positions`; `NormalizedOutput::input_elements_at` and `reads_declared_input`; `contraction_region`; `contraction_accesses_match`; the `contraction-f32.v1` request-subject arm; and `tiler_ir::schedule::verify_contraction`, which independently requires reads at input ordinals zero and one. `input_keys` is both the complete declaration list and the operand list only because the guard proves those sets equal.

**False in the original ticket:** this is not behavior-neutral in the public-observation sense. `CompileFailureClass::UnsupportedCapability { rule }` documents `rule` as a stable diagnostic key, so the returned value intentionally changes. The accepted-program population, plans, numerical behavior, request-subject bytes, artifact identity, and every public type and signature remain unchanged.

**False corpus claims:** [the program-planning record](../docs/research/program-planning/complete-model-ingestion-and-execution.md), at `the input-arity rule now fires only for`, and [the correctness contract](../docs/correctness-and-testing.md), at `neither input nor output cardinality is part of that bound any more`, both overlook the contraction recognizer's independent declared-input wall. Preserve their prior corrections and append dated forward corrections.

**Trigger already fired:** admitted multi-output programs can retain a third declaration beside a binary contraction, and `rms_norm(matmul(a, b), w)` is already a named required chain. [`admit-a-contraction-over-a-subset-of-the-declared-inputs`](admit-a-contraction-over-a-subset-of-the-declared-inputs.md) is therefore `todo`, depends on this naming repair, and owns the IR verifier, explicit ordinal map, physical consumers, and identity determination. The sibling `sum-contributor-ordinal` remains the precedent for naming a lower-layer vocabulary wall before widening it.

## The work — classification only, no admission

- Rename only `normalize_contraction`'s rule to `contraction-input-arity`; keep the program-wide zero-input rule as `input-arity`.
- Rewrite the normalizer and `NormalizedContraction::input_keys` documentation around the guard-enforced coincidence and complete widening owner above. Correct the live `rms_norm(matmul(a, b), w)` comments in `request.rs`, `recognized_chain_depth_boundary.rs`, and `staged_family_over_a_materialized_intermediate.rs`.
- In `contraction_direct_path.rs`, retain the one-input refusal, add a retained-three-input/multi-output contraction subject, assert the new key under all five contracts, and repair both stale `all four` comments to `all five`.
- Add dated corrections to the research record and correctness contract. Leave accurate historical and zero-input uses of `input-arity` alone.
- Change no encoder, identity pin, public item or signature, admission, plan, or numerical result.

## Closes when

The rule is separated, both sides of the declared-input wall are pinned, the program-wide zero-input row remains distinct, the complete widening owner is `todo`, both corpus claims carry dated corrections, and all identity pins remain unchanged. Restore only the contraction rule string to `input-arity`; require both contraction tests to fail with the actual/expected keys while the zero-input row stays green, then restore.

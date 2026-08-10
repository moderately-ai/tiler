---
id: admit-a-contraction-over-a-subset-of-the-declared-inputs
title: Admit a contraction over a subset of the declared inputs
status: in-progress
priority: p2
dependencies: [name-the-contraction-operand-arity-wall-and-separate-its-rule]
related: [admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs, admit-a-materialized-intermediate-read-in-the-scheduled-region-vocabulary, admit-a-staged-family-that-reads-a-materialized-intermediate]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, ir, schedule, compiler-api]
claimed_from: todo
assignee: sol-contraction-subset
lease_expires_at: 1786404386
---
## Trigger — fired 2026-08-08

Ordered multi-output programs are admitted, so a binary contraction can coexist with an independent output that retains a skipped declared input. Separately, the already required single-output `rms_norm(matmul(a, b), w)` chain retains a third declaration via the staged consumer rather than multi-output coexistence. Both subjects hit this wall under `contraction-input-arity`. This is current implementation work, not a parked capability.

## Boundary

This ticket does **not** admit a multi-operand contraction. ADR 0087's binary family and exactly two distinct contraction reads remain unchanged. It admits those two reads from a subset of a larger declared interface while preserving their program input ordinals.

Current construction assumes dense ordinals in more than one place: `tiler_ir::schedule::verify_contraction`; `NormalizedContraction`'s fixed declaration/operand arrays; `NormalizedOutput::input_elements_at` and `reads_declared_input`; `contraction_region`; `contraction_accesses_match`; and the `contraction-f32.v1` request-subject encoding. The parent ticket names the wall; this ticket owns its complete removal.

## What closes this

- `verify_contraction` admits two distinct program inputs in strictly ascending program-ordinal order, including gaps. Repeats, descent, non-input roles, malformed maps, and proof mismatches still refuse.
- `NormalizedContraction` retains the complete declared-input interface and carries an explicit two-read map naming each operand's program ordinal, shape, element count, semantic value, and structure operand position.
- `contraction_region`, `contraction_accesses_match`, `input_elements_at`, and `reads_declared_input` all consume that map rather than dense position or declaration-list length.
- The request subject distinguishes all three two-input subsets of a three-input declaration without relying on the enclosing semantic identity. Determine whether a conditional encoding over the previously impossible wider-interface branch preserves `contraction-f32.v1`; otherwise step the sub-tag and enumerate every moved pin. Do not assume the answer.
- A contraction reading non-contiguous ordinals beside an independent output retaining the skipped input compiles and bit-agrees with the reference evaluator. Dense renumbering must produce a different binding and be named in the assertion.
- The one-declared-input repeated-operand case remains refused by its actual distinct-operand/binding rule. Schedule identity already encodes input ordinals; verify that widening the old-impossible population moves no old region bytes.

## Required independent perturbations

Restore the IR verifier's exact `0,1` predicate; densely renumber only `contraction_region`; densely renumber only `contraction_accesses_match`; perturb both physical derivations together; reintroduce dense indexing in `input_elements_at`; reintroduce the length predicate in `reads_declared_input`; and remove the request-subject operand-ordinal encoding. Each must fail its own targeted subject with quoted diagnostics before restoration. Test repeat and descent independently.

Stop for any public signature or semantic-family change, or if the identity determination cannot preserve injectivity without a separately reviewed domain step and pin sweep.

## Fact audit — 2026-08-10

**Correction — 2026-08-10.** The Trigger originally said the `rms_norm(matmul(a, b), w)` chain "needs exactly that shape," equating it with multi-output coexistence of a binary contraction beside an independent retained-third-input output. Those are two distinct subjects; both still refuse under `contraction-input-arity` at this tree: (a) multi-output binary contraction beside an independent output that retains a skipped declaration; (b) single-output staged chain that retains the third declaration in the staged consumer. Only (a) is multi-output coexistence; the chain does not need multi-output. Fixtures still spell `rms_norm(matmul(a, b), a)` over two declared inputs to dodge this wall (`staged_family_over_a_materialized_intermediate`, `recognized_chain_depth_boundary`, `request.rs` contraction-fed-normalization comments). Boundary prose "ADR 0087's binary family" means the reserved multi-operand question / fifth structural rule (`no index in more than two operands`), not a second family key.

## Worker evidence — 2026-08-10

The implementation keeps the complete declared interface and records exactly two normalized reads, each with its program ordinal, shape, element count, semantic value, and structure operand position. Normalization orders the pair by strictly ascending program ordinal; scheduled-region construction, request binding, read predicates, and element lookup consume that map without local renumbering. The IR verifier now accepts a gap while refusing repeat and descent independently with `[NumericalOrAccessRefinement]`.

The `contraction-f32.v1` identity is unchanged for the admitted population. Before this widening, a contraction could have exactly two declarations and its verified distinct ascending ordinal pair was therefore recoverably `0, 1`. The encoder writes the two ordinals only when the framed declaration count is wider than the two-read run, a branch that the retired `contraction-input-arity` guard made impossible. The declaration count selects the branch, and the fixed two-`u32` run makes every pair in a wider interface injective before the existing shape fields. `a_two_declaration_contraction_keeps_its_v1_subject_bytes` reconstructs the old arm and compares every byte; `contraction_subjects_separate_all_two_input_subsets_of_three_declarations` drives the contraction arm directly and distinguishes `0,1`, `0,2`, and `1,2` without enclosing semantic identity.

The multi-output `a,c` contraction plus independent `b+b` output compiles, exposes physical reads `[Input(0), Input(2)]` and `[Input(1)]`, and bit-agrees with the reference evaluator. The staged fixtures now spell `rms_norm(matmul(a,b),w)` over three declarations: recognition reaches the existing scheduled-region two-edge vocabulary wall and retains its typed `NoFeasiblePlan` refusal. Multi-operand contraction and a repeated one-input contraction remain unsupported; the latter refuses under `contraction-operands`.

Each required perturbation was applied to the subject and restored:

- restoring the verifier's exact `0,1` predicate failed the `0,2` positive case with `diagnostics: [NumericalOrAccessRefinement]`;
- dense renumbering only in `contraction_region` failed with `Intrinsic { rule: "request-binding", region: RegionId(0) }`;
- dense renumbering only in `contraction_accesses_match` failed with `Intrinsic { rule: "request-binding", region: RegionId(0) }`;
- dense renumbering in both physical derivations failed with `InvalidCompilerOutput(Program(CoreVerification(UnusedValue)))` because the falsely substituted declaration left `c` unused;
- dense indexing in `input_elements_at` failed with `assertion left == right failed; left: None; right: Some(4)`;
- the declaration-length predicate in `reads_declared_input` failed with `assertion failed: !recognized.outputs()[0].reads_declared_input(skipped)`;
- removing the wider-interface ordinal run failed with `two declared-input subsets collided`.

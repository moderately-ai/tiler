---
id: correct-the-contraction-schedule-comment-s-operand-count-authority
title: Correct the contraction schedule comment's operand-count authority
status: done
priority: p3
dependencies: []
related: [admit-the-contraction-semantic-profile, refresh-multi-output-correctness-row-after-access-ordinal-reconciliation]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, contraction, schedule]
---
## User-visible outcome

The schedule verifier documents the actual authority that keeps the currently admitted strict contraction binary. It no longer attributes the two-read schedule shape to ADR 0087's fifth structural rule, which governs how many operands may share one index rather than how many operands the operation has.

## Exact-base Fact audit — 2026-08-16, `0f04651e8a0f2c132ce37cab6b86552edc46044c`

The audit was first drafted at ancestor `791e9356926c916e172dec570b4634ef7224f34c`; the claim-only commits to this implementation base do not change any cited IR, semantic, test, or ADR source.

**Fact — verified stale source claim.** `crates/tiler-ir/src/schedule/builder.rs`, anchor `A contraction reads exactly two operands`, says ADR 0087's fifth rule makes any third read a semantic-registry refusal. That implication is false: the rule refuses an *index* present in more than two operands; it does not require the structure to have only two operands.

**Fact — verified semantic authority.** `register_standard_contraction` in `crates/tiler-ir/src/semantic/contraction.rs` installs `OperationArity::exact(2)` for `tiler::strict-tensor-contraction-f32@1`. `a_structure_whose_operand_count_is_not_the_signature_is_refused` in the same module's tests constructs four operands in two independent pairs. No index appears in more than two operands, so rule five passes and `contraction.structure.operand-count` is the refusal. The separate three-operand fixture sharing one index proves rule five remains independently reachable before the occurrence's exact-arity check.

**Fact — verified schedule authority.** The `ScalarProgram::StrictTensorContraction` arm in `verify_intrinsic` currently destructures exactly `[left, right, write]`, and `verify_contraction` is explicitly the verifier for a two-operand strict contraction region. This fixed scheduled form is consistent with the registered operation's exact-two signature; it is not a derivation from rule five.

**Fact — verified accepted decision.** ADR 0087, anchor `no index in more than two operands`, deliberately leaves the multi-operand question reserved. The completed admission ticket records that the five structural rules run before occurrence operand count, specifically so rule five remains reachable under an exact-arity-two schema. Correcting the comment preserves both authorities rather than widening either one.

## Required work

- Rewrite only the stale `verify_intrinsic` contraction comment to name the registered operation's exact-two semantic signature and the current fixed two-read scheduled form.
- Preserve the distinction between exact-two *operation operands* and ADR 0087 rule five's per-index participation limit.
- Do not widen semantic arity, schedule access count, lowering, identity, schema, public diagnostics, or the reserved multi-operand population.
- Add or retain a source-level check whose subject is the false implication, and demonstrate it fails when that implication is restored. The check must also confirm the corrected authority remains present so deleting the comment cannot appear green.

## Closes when

The live Rust comment agrees with `OperationArity::exact(2)`, the existing rule-five and operand-count tests remain green, the source-subject perturbation has been watched failing and restored, and proportional IR documentation/test gates plus `tkt lint`, citations, diff check, and exact-base scope guard pass.

## Implementation record — 2026-08-16

The `verify_intrinsic` comment now names the registered operation's exact-two semantic signature and the scheduled form that preserves it. It separately states ADR 0087 rule five's actual authority: limiting one index's participation across operands. No code, public surface, diagnostic, identity, schema, or admitted population changed.

The final source check is:

```sh
test "$(rg -c 'exact-two semantic signature' crates/tiler-ir/src/schedule/builder.rs || true)" -eq 1 && test "$(rg -c 'third read would be an occurrence' crates/tiler-ir/src/schedule/builder.rs || true)" -eq 0
```

It exited 1 on the exact base because the corrected authority was absent and the false implication was present. After the repair it exits 0. Restoring only the retired implication after the repair made the second limb exit 1; the subject was restored and the check returned to green, so deleting or regressing it cannot appear green.

Both independent semantic authorities remained green under their fully qualified exact tests:

```sh
cargo test -p tiler-ir --lib semantic::contraction::tests::rule_five_refuses_an_index_present_in_more_than_two_operands -- --exact --nocapture
cargo test -p tiler-ir --lib semantic::contraction::tests::a_structure_whose_operand_count_is_not_the_signature_is_refused -- --exact --nocapture
```

Each ran one test and passed: the first pins the per-index rule, and the second pins the distinct exact-arity authority with a four-operand structure that rule five admits.

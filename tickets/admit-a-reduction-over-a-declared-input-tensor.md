---
id: admit-a-reduction-over-a-declared-input-tensor
title: Admit a reduction whose contributor tensor is a declared input
status: in-progress
priority: p2
dependencies: [admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary]
related: []
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, ir]
claimed_from: todo
assignee: agent-input-reduction
lease_expires_at: 1786003957
---
## User-visible outcome

`sum(x)` — a strict serial reduction whose operand is a declared program input, with no elementwise prologue at all — compiles, instead of refusing at the request boundary under `reduction-prologue`.

## Why this exists

**Fact — the wall is in the schedule IR, not the recognizer.** `crates/tiler-ir/src/schedule/builder.rs`'s `verify_access_and_semantics` admits a `ScalarProgram::StrictSerialSum` region only when `read.tensor == TensorRole::Intermediate`; the `FusedMultiplyAddSerialSum` and `SquaredSerialSum` arms beside it admit `FIRST_INPUT` because their prologues read the original input. A region folding a declared input directly under a plain `StrictSerialSum` is therefore rejected by the intrinsic verifier as malformed compiler output.

**Fact — the general recognizer reaches it and refuses.** `admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary` generalized the elementwise prologue to the whole `PointwiseF32Expression` vocabulary, and `sum(x)` is the one recognized shape it then had to refuse *at the boundary* rather than admit and fail mid-pipeline. `recognize_reduction` states the rule and names this ticket; `a_reduction_over_a_declared_input_refuses_under_the_prologue_rule` in `crates/tiler-compiler/tests/composed_family_recognition.rs` drives it against an accepted neighbour.

**Inference — synthesizing an identity prologue is not the fix.** Staging a copy of a tensor the fold could read directly adds an observable materialization boundary, and its rounding, that the caller's program never asked for.

## Boundaries

- The schedule-IR arm is the change, and it is a *widening of an admitted access*, not a relaxation: a `StrictSerialSum` region reading `FIRST_INPUT` must still prove its contributor relation, bounds, and ownership exactly as the intermediate-reading one does.
- The split and single-workgroup-tree strategies read the same contributor tensor. Either they widen with it, or the frontier declines them for a prologue-less fold with a stated reason — silently losing an alternative is not acceptable.
- The program assembly must bind one buffer per declared input with no temporary, which `build_fused_core` already does for the single-region shapes.

## Closes when

`sum(x)` compiles through `tiler_compiler::session` to an emitted region under every registered contract; the `reduction-prologue` refusal is removed together with the boundary that raised it; and the schedule verifier still refuses a `StrictSerialSum` region whose contributor access does not match its declared reduction, observed failing.

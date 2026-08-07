---
id: correct-the-stale-dtype-f32-recognizer-claims-in-the-conformance-crate
title: Correct the stale dtype-f32 recognizer claims in the conformance crate
status: todo
priority: p2
dependencies: []
related: [widen-the-strategy-recognizer-past-the-f32-wall, conform-the-bf16-vertical-end-to-end]
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, bf16, dtype, correction]
---
## What is false

**Fact, at the merge of `widen-the-strategy-recognizer-past-the-f32-wall`.** Two module comments in `crates/tiler-conformance` state a rule that no longer exists:

- `crates/tiler-conformance/src/lib.rs:17` — "non-`f32` program under the rule `dtype-f32` before a subject is normalized".
- `crates/tiler-conformance/src/bf16_vertical.rs:20` — "refuses every program carrying a non-`f32` value under the rule `dtype-f32`".

`select_supported_strategy` no longer carries a `dtype-f32` rule at all. It derives the program's one arithmetic type and admits the two widths this build spells a per-point body in, refusing a width it cannot spell under `dtype-recognized` and a mixed-width program under `dtype-uniform`.

**Fact.** `crates/tiler-conformance/src/bf16_vertical.rs:24` cites the compiler test `a_flush_accepting_bf16_contract_reaches_the_recognizer_dtype_wall`, which was renamed to `a_flush_accepting_bf16_contract_reaches_a_selected_plan` and now asserts the opposite outcome.

## What is true now

A single-occurrence BF16 program is recognized, planned, and reaches a selected `PlanAlternative`. A BF16 region covering *several* occurrences stops one layer further on, at `fusion_legality`, whose capability table is keyed by the `f32` operation set — `crates/tiler-compiler/tests/bf16_numerical_contract.rs`'s `a_multi_occurrence_bf16_program_stops_at_the_fusion_legality_wall` asserts that boundary, and `establish-bf16-optimizer-legality` owns widening it.

That distinction is what the corrected comments have to carry: the vertical's own `(x * 3.0) + (-0.0)` fixture is a four-occurrence chain, so it is still assembled through `tiler-ir`'s public builders — for the *fusion* reason rather than the dtype one.

## Why it is filed rather than fixed

`crates/tiler-conformance/**` is `implementation/conformance`, which the recognizer branch did not hold and which was live-claimed by another worker at the time.

## Required evidence

- Both comments state the boundary that exists, naming the rule that refuses and its owner.
- The cited test name resolves, or the citation is replaced by one that does.
- `cargo doc --no-deps` with warnings denied still passes for the crate.

## Closes when

No comment in `crates/tiler-conformance` claims a `dtype-f32` rule, and each cited test name exists.

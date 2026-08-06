---
id: test-or-correct-the-structural-read-of-a-staged-operand
title: Test or correct the structural read of a staged operand
status: todo
priority: p3
dependencies: []
related: [admit-elementwise-epilogues-over-a-materialized-intermediate, move-the-structural-row-to-r6-and-retire-its-backend-residual]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, correctness, doc-claim, structural, tests]
---

## The claim, and why it is currently unearned

**Fact.** The doc comment on `recognize_structural_read` (`crates/tiler-compiler/src/request.rs:4775-4779`) makes a positive admission claim: "An epilogue's staged operand is a different case and is admitted: another region already materialized it, so the rounding boundary is the cover's rather than one this occurrence introduced, and the read binds the materialization edge the cover hands the region." The check backing it is `leaves.is_leaf(*operand)` at `:4803`, which a staged operand plausibly satisfies — but whether `ElementwiseLeaves` actually marks a staged value as a leaf on the epilogue walk has not been read to construction, and **no test exercises the combination**: `rg -n 'fn .*epilogue' crates/tiler-compiler/src/pipeline/tests.rs crates/tiler-compiler/tests/materialized_intermediate_epilogue_wall.rs` lists nine epilogue tests and none contains a structural occurrence, and `rg -ln 'structural.*epilogue|epilogue.*structural' crates/tiler-compiler/` returns nothing. Found by the 2026-08-06 navigation batch, coordinator-verified by direct read the same day.

A doc comment is a claim the next worker acts on (AGENTS.md), and this one makes unreached work look reachable: a reader planning `reverse(matmul(a, b))` would conclude the region vocabulary admits it today.

## The work

Read `ElementwiseLeaves` to construction on the epilogue path and determine which of the two worlds is real:

- **The admission is real.** Then it needs a test before the doc may say so: a structural occurrence (reindex or broadcast) over a staged value — e.g. an epilogue reversing a materialized contraction result — compiled through the ordinary `compile()` entry point and bit-compared against `tiler-reference`, beside the refusal test for a structural occurrence over a value the *same* region computes (`a_structural_occurrence_over_a_computed_value_refuses_by_name`), so the boundary between the two cases is pinned from both sides.
- **The admission is not real** (the walk refuses or misclassifies a staged structural operand). Then the doc comment is the defect: narrow it to what the tests carry, and file or widen the admission as its own owned wall if the capability is wanted.

Do not decide which world by the doc — decide by tracing the staged operand through `ElementwiseLeaves` and, if ambiguous, by writing the program and observing the verdict.

## Closes when

Either a passing bit-compared test exercises the structural-read-of-a-staged-operand admission and the doc's claim is test-backed, or the doc comment states only the tested behaviour and the wanted widening (if any) has a named owner.

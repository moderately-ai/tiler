---
id: admit-ordered-multi-output-programs-at-the-compiler-request-boundary
title: Admit ordered multi-output programs at the compiler request boundary
status: in-progress
priority: p1
dependencies: [admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary]
related: [admit-multi-input-elementwise-programs-at-the-compiler-boundary, accept-the-public-compiler-facade-boundary]
scopes: [implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, conformance]
claimed_from: todo
assignee: agent-multi-output
lease_expires_at: 1785688837
---
## User-visible outcome

A semantic program declaring several ordered named outputs compiles, so the conformance gate's multi-output requirement is discharged by a program that compiles rather than by a test asserting the refusal.

## Why this exists

**Fact — two guards make single-output a boundary invariant.** `crates/tiler-compiler/src/request.rs:2228` opens `select_supported_strategy` with `if program.output_count() != 1`, and `crates/tiler-compiler/src/program.rs:1234` carries the same condition on the program-assembly path. A program with two ordered outputs is refused before a strategy is selected.

*Corrected by [`admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary`](admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary.md).* This paragraph read "four guards" at `request.rs:2184`, `:2377`, and `:2581` — one per whole-program template. The general recognizer replaced those three templates with one occurrence walk, so the three per-template guards became the single program-wide one above, checked once before the output's producing occurrence is classified. The count changed; the invariant did not, and neither did this ticket's obligation.

**Fact — the conformance gate's multi-output row is therefore a permanent negative test.** `docs/correctness-and-testing.md:106-111` requires the optimizer conformance owner to exercise "non-isomorphic and fan-out or multi-output graphs" *before the public compiler facade is accepted*; `:117` records the consequence in the gate's own words — "Ordered multi-output programs are rejected explicitly rather than compiled, so the multi-output row of the requirement above is a negative test." The test is `ordered_multi_output_programs_reject_explicitly` at `crates/tiler-compiler/src/pipeline/conformance.rs:389`.

**Inference — the gap contradicts the architectural contract directly.** AGENTS.md requires modelling programs "as typed operations and values with ordered named outputs and multi-result support — not one SQL-like root or a single output tensor". A boundary refusing every multi-output program is the single-output root that clause forbids, and no other node owns lifting it. [`admit-multi-input-elementwise-programs-at-the-compiler-boundary`](admit-multi-input-elementwise-programs-at-the-compiler-boundary.md) (`done`) is the precedent for how such a limit is located, measured, and lifted — and its outcome shows the wall is usually one layer below the guard, so this ticket's first obligation is to locate the multi-output wall in `tiler-ir` before editing any guard.

## Boundaries

- Recognizer generalization is [`admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary`](admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary.md)'s and is this ticket's dependency; do not widen templates here.
- Output *order* is identity, not presentation. A widening that admits two outputs but leaves them interchangeable would satisfy the guard and violate the contract. Ordering enters the schedule and artifact identity encodings exhaustively, with no previously encodable program's bytes moving.
- If the wall proves to be in the scheduled-region or artifact vocabulary rather than in `tiler-compiler`, file that widening as its own ticket and depend on it — do not widen a compiler guard onto a physical layer that cannot express the result.

## Required failure-path evidence

Each run against a case that must fail and observed failing, against an accepted neighbour: two declared outputs colliding on one semantic value; a program declaring more outputs than the plan names; a plan naming fewer outputs than the program declares (the same rule [`implement-general-dag-partitioning`](implement-general-dag-partitioning.md) states as its closing condition 2); and two programs differing only in output order yielding distinct identities.

## Closes when

`ordered_multi_output_programs_reject_explicitly` is flipped from a refusal expectation to a compilation, with the transition demonstrated failing at the unwidened base; `docs/correctness-and-testing.md:117`'s sentence naming the multi-output row a negative test is corrected in the same change; and every ordering obligation above is checked by a check observed failing.

---
id: refuse-nothing-legal-on-the-explain-detail-ceiling
title: Refuse nothing legal on the explain detail ceiling
status: in-progress
priority: p2
dependencies: []
related: [widen-the-identity-growth-ladder-to-the-governed-operation-budget, carry-the-exhausted-resource-through-the-budget-refusal]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [defect, explain, compiler, limits]
claimed_from: todo
assignee: agent-explain-ceiling
lease_expires_at: 1786073446
---
## User-visible outcome

A semantic program that satisfies every governed budget compiles, or is refused with a class that names the caller's problem. It is not refused with `CompileFailureClass::InvalidCompilerOutput`, which the public documentation defines as "always a defect in Tiler rather than in the caller's program".

## The measurement

**Measurement, 2026-08-06, `spikes/program-planning/identity-growth`, Apple M4 Max, macOS 27.0 `26A5388g`, toolchain `nightly-2026-07-19`.** An eleven-operation chain — one `tiler::constant-f32@1` and ten `tiler::multiply-f32@1` over a rank-1 extent-4 `f32` tensor, compiled through `tiler_compiler::session::compile` under `FLUSH_SUBNORMALS_TO_ZERO_F32` against the authoritative macOS Apple9 declaration — refuses:

```
CompileFailure { class: InvalidCompilerOutput, explain: "3478 records" }
```

whose terminal record is `compiler-failure:explain-detail-capacity`. The same generator at ten operations verifies, and at twelve refuses `NoFeasiblePlan` for an unrelated reason (`region-expansion-exhaustion-loses-the-only-feasible-plan`).

**Fact — nothing about the program is out of budget.** Region formation succeeds at eleven operations with `budget-stops:count=0`, `candidate-count:count=66`, `rejected-non-convex:count=1013`. `semantic_operations` is 62, `semantic_values` 80, `regions` 12. The refusal is downstream of all of them.

**Fact — the ceiling.** `MAX_RECORDS = 4_096` and `MAX_CANONICAL_BYTES = 1024 * 1024` (`crates/tiler-compiler/src/explain.rs:114-115`). `ExplainWriter::push_detail` returns `ExplainError::DetailCapacity` when a detail record would exceed either, and `pipeline.rs:2092` maps that to `"detail-capacity"`. The bound is deliberate and its reason is stated in the source: "A trace is complete or it is refused. Exceeding a bound is a typed failure, never a silent drop: a reader who cannot tell which records are missing cannot rely on the ones that remain."

**Inference — what exhausts it is cover enumeration, and it grows with the operation count.** 2,300 of the 3,478 records are `candidate-enumeration rejected-intrinsic rule=selection.region-coverage.v1@1 … event=check:selection.region-implemented:disproved:region-unimplemented`, one per rejected region cover against a single unimplemented singleton region. That population grows combinatorially in the operation count while the ceiling is a constant, so the ceiling is a program-size limit that no budget declares and no caller can read.

## Why the refusal discipline is not the thing to change

The complete-or-refused property is correct and this ticket does not propose weakening it. What it proposes deciding is one of:

- the detail ceiling is sized against the program sizes the governed budgets admit, rather than against a constant chosen before the budgets moved;
- the rejection population is summarized at its source (one record per rejection *class* with a count, rather than one per rejected cover), which is a change to what the trace says and therefore an explainability decision rather than a capacity one; or
- the ceiling refuses under a class that is not `InvalidCompilerOutput`, because a caller who reads that class is told Tiler is broken when what happened is that Tiler declined to explain itself at length.

The third is the smallest and the least satisfying: it renames a refusal without moving the wall. The first and second both move the wall and both touch `docs/compiler/optimizer.md`'s requirement that explain output "never collapses these into 'not fused'".

## Why it matters

The governed `semantic_operations` budget was raised to 62 on 2026-08-05, sized "to the complete decoder-layer program, which is the largest program shape this profile may be asked to admit". This ceiling refuses at eleven. Any work that assumes the budget describes the reachable domain is wrong by a factor of about six, and the identity-growth ladder is the first thing to have measured it.

## Evidence

- `spikes/program-planning/identity-growth/results/2026-08-06-apple-m4-max-macos27.0-26A5388g/growth.tsv`, wall table, first row.
- `cd spikes/program-planning/identity-growth && cargo run --release` reproduces it; the wall is probed on every run and fails loudly if it moves.

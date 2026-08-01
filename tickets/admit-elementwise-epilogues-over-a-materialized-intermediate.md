---
id: admit-elementwise-epilogues-over-a-materialized-intermediate
title: Admit an elementwise epilogue over a materialized intermediate
status: todo
priority: p2
dependencies: [admit-a-general-program-shape-recognizer-at-the-compiler-request-boundary]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-api, optimizer]
---
## User-visible outcome

A program whose output is an elementwise expression over a *contraction's* or a *reduction's* result — `matmul(a, b) * 2.0`, or `sum(x * x) * scale` — compiles, instead of refusing at the request boundary under `operation-set`.

## Why this exists

**Fact — the recognizer refuses it deliberately.** `normalize_contraction` requires the contraction occurrence to cover the program exactly (`CONTRACTION_OPERATIONS`), and `recognize_elementwise` classifies every operand as a declared input, a constant, or another elementwise occurrence — so an operand produced by a fold or a contraction refuses. `select_supported_strategy`'s documentation names this ticket for the widening.

**Fact — the wall is the physical layer's, not the schedule IR's.** `TensorRole::Intermediate` is a *per-region* role, so nothing in `tiler-ir` forbids a chain that stages a second temporary; `partial_reduction_region` already writes an intermediate that `final_reduction_region` reads. What is missing in `tiler-compiler` is: an elementwise region that reads an intermediate (`pointwise_region` builds every read as `TensorRole::Input { ordinal }`), a contraction region that writes one (`contraction_region` hard-codes `TensorRole::Output`), the program assembly for the resulting chain, and the request-subject binding arms for both.

**Inference — this is what makes the recognizer's generality reach the families it already knows.** The elementwise walk is general over its own vocabulary; what it cannot yet do is compose *across* a materialization boundary, which is the composition every fused-epilogue workload needs.

## Boundaries

- The recognized program must still be a bounded chain: the `regions` and `buffers` budgets bound it, and `verify_program` derives both from the declared arity, so a chain needing more must refuse by name rather than be assembled.
- Every stage's request-subject binding must re-derive its accesses from the recognized program, as the existing arms do. A stage admitted on its scalar program alone would let a provider bind the wrong tensor.
- `ProgramAlternativeKind::of` classifies a plan by its cover, and `build_plan_program` matches on `(kind, region count)`. A three-stage non-split chain is a shape neither currently expresses; widening them is part of this ticket, not a follow-on.

## Closes when

A contraction feeding an elementwise epilogue compiles through `tiler_compiler::session` to an emitted region; a chain the budgets cannot admit refuses by name, observed failing; and the request-subject binding refuses a forged stage for each new region kind, observed failing.

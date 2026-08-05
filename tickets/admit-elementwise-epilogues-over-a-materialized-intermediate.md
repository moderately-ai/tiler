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

## Corrected 2026-08-05 by the coordinator's pre-resume sweep — the wall this ticket owns doubled, and the recognition layer it cites was rebuilt

The Fact above describes the single-output recognition as it stood before `recognize-several-ordered-named-outputs-at-the-compiler-request-boundary` and `admit-ordered-multi-output-programs-at-the-compiler-request-boundary` landed (both 2026-08-05). Recognition is now one walk per declared output (`recognize_program_outputs`), and `select_supported_strategy`'s documentation moved with it — re-read it at dispatch rather than trusting the paragraph above. What is unchanged: an operand produced by a fold or contraction still refuses inside a single output's elementwise walk, which is this ticket's original wall.

**What is new: this ticket now owns a second, measured wall.** A program that publishes an intermediate *and* consumes it — the conformance suite's own multi-output fixture, publishing `scaled` and reducing it into `reduced` — refuses at `phase: "strategy", rule: "output-partition-overlap"`, because the published value's occurrence sits inside the reducing output's walk and two outputs may not claim one occurrence. The gate row's paragraph in `docs/correctness-and-testing.md` records the derivation: one region's owning write would have to serve both a materialization edge and a publication, `ValueRole` is exclusive (`fills` refuses an `Output` value for an `Intermediate` buffer), so the shape needs a copy stage reading `Intermediate` and writing `Output` — and building that copy stage is exactly this ticket's outcome. The pinned test is `a_published_and_consumed_intermediate_refuses_by_name`. Discharging this ticket therefore flips one of the compiler-facade gate's two named open bounds (the other is `admit-an-elementwise-region-reading-a-subset-of-the-declared-inputs`), which is worth stating in the dispatch brief because it raises the ticket's effective priority above its filing-time framing.

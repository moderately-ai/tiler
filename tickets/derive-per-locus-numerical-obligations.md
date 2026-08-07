---
id: derive-per-locus-numerical-obligations
title: Derive per-locus numerical obligations in the compiler
status: todo
priority: p2
dependencies: []
related: [redesign-the-delivered-realization-record-from-typed-evidence, accept-the-delivered-realization-artifact-surface]
scopes: [implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, compiler]
---
## Why

`redesign-the-delivered-realization-record-from-typed-evidence` shapes the delivered-realization record around `(subject, dimension, locus)` obligation rows, because ADR 0011's per-operation restrictions attach to a *position*: one `f32` operation's accumulator and its observable materialization boundary can carry different legal requirements, and a dtype-wide ceiling alone keeps whichever was written last.

> **The Fact below is stale and was corrected 2026-08-07 by the coordinator at `879dec67`. Do not brief from the struck version.** It read: "the compiler cannot produce such a row today. Exact check: `grep -rni "locus" --include="*.rs" crates/` returns nothing." That check now returns **166 matches**. The locus vocabulary landed, and so did a producer — just a single-locus one. What remains open is narrower than what this ticket was filed for, and a worker briefed on the old text would re-derive vocabulary that already exists.

**Fact, 2026-08-07 — the vocabulary exists and the compiler already emits per-occurrence obligations at one locus.** `PolicyLocus` is declared at `crates/tiler-ir/src/numerics.rs:1024` with `NumericalObligationKey` beside it, and `crates/tiler-compiler/src/session/realization.rs:377` constructs `NumericalObligationKey::new(*occurrence, PolicyLocus::Computation)` — its module header (`:39`) states the shape in terms: one row "per honoured dimension at `PolicyLocus::Computation` of **every** occurrence". So the occurrence half of the key is derived and the locus half is pinned to a constant.

**Fact — `dimension_requirements` is no longer the eight-whole-program projection this ticket described either.** It now derives its subject from the caller's contract through `arithmetic_subject` rather than hard-coding `F32::resolved_type()`; that change landed under `6207fba4` and retired the sibling ticket [`key-numerical-requirements-by-the-contract-s-own-resolved-type`](key-numerical-requirements-by-the-contract-s-own-resolved-type.md), closed `obsolete` on 2026-08-07. The dtype-wide ceiling and the locus obligations remain separate statements, which is the rule below that survives unchanged.

**What is actually open.** Exactly the locus half: an obligation whose locus is drawn from the full set — input, computation, accumulator, result, component, and materialization — rather than fixed at `Computation`. This is the "single-locus producer" this ticket's own Graph maintenance anticipated the record would admit; it landed, and widening it is the remaining work. Recheck the boundary with `grep -rn "PolicyLocus::" crates/tiler-compiler/src/` — while every constructed key reads `PolicyLocus::Computation`, this ticket is open.

## What closes this

The compiler derives, per selected plan, one obligation per `(policy subject, dimension, program occurrence, policy locus)` that a packaged route relies on, with the locus drawn from input, computation, accumulator, result, component, and materialization. The occurrence is `tiler_ir::program::SemanticOccurrence`, so an obligation and the stage coverage implementing it name the position the same way.

Two rules survive unchanged: the dtype-wide ceiling and the locus obligations are separate statements, neither derived from the other; and a locus requirement is at least as strict as the ceiling, never weaker.

## Graph maintenance

- The review packet at `spikes/numerics/delivered-realization-record/` records the shape this must fill; do not re-derive it.
- This does not block `accept-the-delivered-realization-artifact-surface`: the packet is reviewable, and the record admits a single-locus producer.
- `wire-the-delivered-realization-record-into-the-artifact` may land against the single-locus producer; this widens what that path carries.

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

**Fact — the compiler cannot produce such a row today.** Exact check: `grep -rni "locus" --include="*.rs" crates/` returns nothing. `StrictF32NumericalContract` (`crates/tiler-compiler/src/request.rs:179-214`) is one flat record for one arithmetic type, and `crate::policy::dimension_requirements` (`policy.rs:636-654`) projects it into exactly eight **whole-program** `NumericalRequirement`s. There is no per-locus, per-occurrence, or per-accumulator numerical requirement anywhere in the compiler.

**Inference.** The record's shape is derived from the contract and is not blocked by this; what is blocked is a producer that can fill it with more than one row per dimension. Until this lands, a conforming producer emits one obligation per consumable dimension at the computation locus of the occurrence that consumes it, which is exactly as much as the compiler can honestly say.

## What closes this

The compiler derives, per selected plan, one obligation per `(policy subject, dimension, program occurrence, policy locus)` that a packaged route relies on, with the locus drawn from input, computation, accumulator, result, component, and materialization. The occurrence is `tiler_ir::program::SemanticOccurrence`, so an obligation and the stage coverage implementing it name the position the same way.

Two rules survive unchanged: the dtype-wide ceiling and the locus obligations are separate statements, neither derived from the other; and a locus requirement is at least as strict as the ceiling, never weaker.

## Graph maintenance

- The review packet at `spikes/numerics/delivered-realization-record/` records the shape this must fill; do not re-derive it.
- This does not block `accept-the-delivered-realization-artifact-surface`: the packet is reviewable, and the record admits a single-locus producer.
- `wire-the-delivered-realization-record-into-the-artifact` may land against the single-locus producer; this widens what that path carries.

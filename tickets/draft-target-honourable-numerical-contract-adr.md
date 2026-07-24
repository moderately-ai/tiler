---
id: draft-target-honourable-numerical-contract-adr
title: Draft a proposed ADR for target-honourable numerical contracts
status: todo
priority: p1
dependencies: []
related: [prototype-metal-numerical-realization, prototype-artifact-program-model, own-operation-family-support-matrix]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [documentation, decisions, numerics, feasibility]
---
Record, as a **proposed** ADR, how a numerical contract expresses what a target
can actually honour — so a real target can be conformant rather than permanently
refused.

## What forced this (measured, not theorised)

Apple GPU `f32` arithmetic **flushes subnormals to zero unconditionally**.
Compiling with `-fmetal-math-mode=safe` — the strictest mode, which explicitly
disables fast math — still emits `air.compile.denorms_disable` alongside
`air.compile.fast_math_disable` in the AIR. No offline flag and no runtime
`MTLCompileOptions` setting clears it. Materialization is unaffected: load/store
round-trips preserve every subnormal. So the limit is specific to arithmetic.

Our strict profile declares `SubnormalMode::Preserve` for inputs and results,
because the profile's correctness claim is bitwise equality with the CPU
reference evaluator, and the CPU preserves subnormals. On Apple that claim is
therefore unsatisfiable for any computation that produces a subnormal.

## The reframe this ticket exists to record

This is **not** a restriction on what Tiler can do. It is a specification input:
it tells us which knobs the numerical contract must expose. Today
`tiler_ir::schedule::SubnormalMode` has exactly one variant, `Preserve`, and
`StrictF32NumericalContract::governed()` is a hardcoded constant no caller
chooses. So there is no flush-tolerant contract to select and no way for a caller
to ask for one — which is why refusal was the only available answer. A vocabulary
that cannot describe real hardware forces every real target to be non-conformant.

The architectural line to preserve while fixing it: a numerical contract is a
**target-neutral semantic declaration** of what the program means; whether a given
target can deliver it is a **feasibility** question. A target's limitation must
never silently redefine what the program means. Under that line, refusing
strict-preserve on Apple is the feasibility authority working correctly — the gap
is only that no feasible alternative is expressible.

## What the ADR should settle

- **Vocabulary.** What `SubnormalMode` must offer beyond `Preserve` (at minimum a
  flush-to-zero mode), and whether input and result subnormals need independent
  settings, since hardware can differ on each.
- **Selection.** How a caller states the contract it needs, replacing a hardcoded
  `governed()` constant — and what happens when a caller states nothing.
- **Target capability declaration.** How a target profile declares which
  numerical realizations it can honour, so feasibility can *select* a conformant
  contract rather than only reject an unsatisfiable one. This is the piece that
  turns a refusal into a choice.
- **Artifact record.** How the delivered realization is recorded so a consumer
  knows what it actually got, rather than inferring it from the request. A
  consumer comparing GPU output against a CPU oracle must be able to tell which
  contract the artifact honours.
- **The honesty rule.** What must happen when no available contract is honourable
  on a target: an explainable rejection, never a silently downgraded one.

Note the same shape applies beyond subnormals — contraction, reassociation, and
NaN payload behaviour are all places where a target may not honour the strict
reading. Decide whether this ADR states a general model or only the subnormal
case, and say which.

## Boundaries

Proposed only: `decision_status: "proposed"`, `ticket` pointing here, open
questions left explicit. Change no code and no other contract; implementation
follows as its own tickets across `tiler-ir` (vocabulary), `tiler-compiler`
(selection and feasibility), `tiler-metal` (capability declaration), and
`tiler-artifact` (delivered-realization record). Prose is not hard-wrapped. Run
`scripts/docs.py render` and the documentation gate before completion.

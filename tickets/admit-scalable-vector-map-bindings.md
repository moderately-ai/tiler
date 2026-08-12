---
id: admit-scalable-vector-map-bindings
title: Admit scalable-vector map bindings
status: deferred
priority: p2
dependencies: [admit-vector-lane-bindings-into-the-schedule-vocabulary, admit-lane-typed-values-and-masked-memory-into-the-kernel-ir, declare-cpu-vector-realization-facts-in-the-target-profile]
related: []
scopes: [implementation/ir, implementation/compiler, implementation/cpu, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [cpu, vector, scheduling, scalable-vector, public-boundary]
---
## User-visible outcome

A width-agnostic CPU vector approach may map independent outputs only when its KIR, target declaration, host qualification, and native runtime all preserve one identity without pretending a runtime vector length is a fixed schedule width.

## Required delivery

- Add a scalable map binding as a distinct execution form with no literal lane count and no fixed-width fallback.
- Define width-agnostic KIR for lane indices, predicates, arithmetic, and memory. Predication is explicit on every iteration; `Exact` and scalar-epilogue policies remain unstatable without a compile-time width.
- Keep contributor partitions unavailable until a separately accepted symbolic coverage relation can be intrinsically proved.
- Treat concrete scalable vector length as cost/runtime evidence only unless a separately named legality predicate proves otherwise. It never changes schedule identity or selects an approach implicitly.
- Require an exact native scalable execution approach and host-earned ISA qualification in `tiler-cpu` / `tiler-cpu-runtime`. No architecture name, target triple, mock probe, simulator, or reference evaluator supplies the evidence.
- Record identity, artifact, and cache consequences without enumerating one fixed schedule per observed runtime length.

## Trigger check log

- 2026-08-12 — **not fired.** No production native scalable CPU representation or width-agnostic KIR exists. Recheck when `tiler-cpu` has a selected SVE- or RVV-class approach that can execute one retained payload without a fixed lane literal.

## Closes when

The trigger fires, Tom accepts the exact scalable carrier, and one real scalable CPU artifact executes under host-earned qualification with fixed-width and contributor-partition substitutions independently refused.

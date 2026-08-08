---
id: admit-a-recognized-chain-more-than-one-materialization-boundary-deep
title: Admit a recognized chain more than one materialization boundary deep
status: in-progress
priority: p3
dependencies: []
related: [admit-a-staged-family-that-reads-a-materialized-intermediate, admit-elementwise-epilogues-over-a-materialized-intermediate]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, planner]
claimed_from: todo
assignee: coord
lease_expires_at: 1786176329
---
## User-visible outcome

`rms_norm(matmul(a, b), a) * a` is recognized instead of refused under `staged-operand-depth` — a recognized chain whose regions are separated by *two* materialization edges rather than one.

## Where the wall is, and why it is a rule rather than a gap

**Fact.** Recognition admits at most one materialization edge per recognized shape, and a shape reached *across* an edge admits none. `crates/tiler-compiler/src/request.rs` states it as `StagedOperandAdmission`: `recognize_output` hands its declared output's occurrence `OneEdge`, and `recognize_epilogue_producer` — the one function reached across an edge — hands `NoEdge`. A staged occurrence at the far side that reads its own edge refuses under `staged-operand-depth`.

**Fact.** The same rule is stated a second time, for the elementwise walk, by `plan_elementwise`'s `leaves.staged.is_none()` guard: a walk that already reads one staged value and reaches a folding family reports the ordinary refusal instead of naming a second boundary. `crates/tiler-compiler/src/request.rs`'s `every_refusal_names_its_unrecognized_property` drives one instance of it (`sum(sum(x) * 2.0)`, reported as `operation-set`), and [`name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set`](name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set.md) owns separating that rule name from the vocabulary refusal it currently shares.

**Inference.** The two guards are one rule about chain depth and they are not the unordinalled-`TensorRole::Intermediate` rule that [`admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region`](admit-a-second-read-of-one-materialized-intermediate-in-an-elementwise-region.md) owns: a two-boundary chain gives each region *one* intermediate read, and what it needs is a recognized shape that can carry a producer at every level and a cover that can place two edges through one output's partition.

## What lifting it costs, read before promising it

`NormalizedStaged::producer` and `NormalizedEpilogue::producer` already nest, so the recognized shape may already express the depth; what is unread is whether the recursion stays bounded (a chain's depth is the caller's program, and the recognizer's producer walk is recursive rather than worklist-driven), whether cover enumeration places two edges through one partition, and whether the `staged-family.v2` and `epilogue-f32.v1` subject arms stay self-delimiting under nesting. Read those three before deciding the shape; a depth counter is the wrong answer if the real bound is host stack.

## Closes when

Either a two-boundary chain is recognized with the recursion bound stated and a cover placing both edges observed, or the depth rule keeps its two guards with one shared statement and this ticket records the measured reason it stays.

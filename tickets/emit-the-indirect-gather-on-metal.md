---
id: emit-the-indirect-gather-on-metal
title: Emit the indirect gather on Metal
status: blocked
priority: p3
dependencies: [admit-the-indirect-access-class-into-the-index-layer]
related: [admit-an-indirect-gather-family-for-tied-embedding-lookup, admit-a-storage-carrier-for-integer-program-inputs]
scopes: [implementation/metal, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, gather, language-model]
---
## User-visible outcome

A region containing an indirect gather reaches a `VerifiedKernel` and a backend emits it, so the support matrix's indirect-gather row can carry an R6 claim.

## Why this is blocked rather than todo

**Fact.** No index region can express the access today: `AccessData` carries one tensor ordinal and `IndexNode` has no variant reading tensor data. There is nothing for a backend to emit, and the question of whether there ever will be is a decision `admit-the-indirect-access-class-into-the-index-layer` holds. Filing this as `todo` would make it dispatchable to a worker who could only park it by hand.

**Fact — a second prerequisite is separately owned.** `StorageScalar` at `crates/tiler-ir/src/program/model.rs` has two variants, `U8` and `F32`, so a `tiler::u32@1` index operand has no runtime carrier. That is `admit-a-storage-carrier-for-integer-program-inputs`, whose own dependencies include the gather family.

## What this ticket delivers when unblocked

An emitted construct for whatever access class the decision above admits, a golden, and a device comparison — the three things R7 additionally needs beyond R6. The unsigned-index arithmetic will need the same *attribution* treatment `emit-the-structural-region-on-metal` gave the mirror's subtraction: name the bound the IR proved rather than implying a check happened at emission.

## Non-goals

Scatter. The access-class decision itself.

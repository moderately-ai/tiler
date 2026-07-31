---
id: implement-the-target-neutral-multi-pass-reduction-strategy
title: Implement the target-neutral multi-pass reduction strategy
status: todo
priority: p1
dependencies: []
related: [implement-parallel-reduction-strategies]
scopes: [implementation/ir, implementation/compiler, implementation/reference, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

A reduction can produce and consume explicit partial tensors across multiple kernel-program stages, retaining a legal serial alternative and without requiring an intra-workgroup barrier.

## Implementation keys

Define each pass's reduction order, accumulation dtype, partial shape/storage, materialization, dispatch dependency, visibility transition, and empty-domain identity. Preserve reassociation and contributor permutation as independent permissions. The program verifier must prove each partial is initialized before use and that the final pass covers every contributor exactly under the selected order.

Retain serial and multi-pass alternatives together. Hard feasibility rejects unsupported storage, dispatch, arithmetic, or numerical permissions; cost remains separate and this ticket does not make the multi-pass plan win by preference.

## Required evidence

One program retains serial and multi-pass alternatives with distinct identities and explain records. Empty, one-element, uneven-tail, and multi-pass extents match the reference under each admitted numerical contract. Missing partial initialization, wrong dependency order, narrowed accumulation, and independently denied reassociation/permutation each reject. If the boundary-enforcer test reaches a real mismatch, activate its owner rather than widening constants.

## Closes when

The target-neutral multi-pass alternative is verified and artifact-replayable, every new check is mutation-proved, public schedule/program boundary changes are reviewed by Tom, and targeted tests/Clippy plus the batch gate pass. Metal realization and calibrated selection remain downstream.

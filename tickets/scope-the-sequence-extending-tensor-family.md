---
id: scope-the-sequence-extending-tensor-family
title: Scope the sequence-extending tensor family the KV cache needs
status: todo
priority: p2
dependencies: [derive-transformer-operation-and-shape-surface]
related: [design-autoregressive-state-and-kv-cache, own-operation-family-support-matrix]
scopes: [contracts/foundation, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, semantics, structural, language-model, breadth]
---
## User-visible outcome

The corpus says what it means to extend a tensor along one axis — the operation every autoregressive decode step performs twice per layer — instead of leaving it in the one position no ledger records: absent from the support matrix, absent from the normative contracts, and absent from the ticket graph.

## Evidence prerequisite

**Fact — neither candidate mechanism exists, and the absence is unrecorded.** A tensor `Concatenate` has no row on the [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix), no normative semantics in [`docs/ir.md`](../docs/ir.md), and no registered key; it is the only family this workload touches that is not even enumerated as absent. The alternative — an in-place windowed write into a preallocated cache buffer — is excluded by the implemented profile: `docs/ir.md` states that "input boundaries may be read but not written, output boundaries may be written but not read, and every declared output boundary requires exactly one complete ordinary write root", and [Q-PLAN-015](../docs/open-questions.md#q-plan-015--advanced-buffer-reuse-and-in-place-execution) defers in-place execution.

**Fact — `Reindex` does not reach it.** `docs/ir.md` admits `Reindex` as bijective permutations, splits, merges, and unit-axis insertion or removal. A concatenate is multi-operand with an output partitioned by operand, which is outside those forms and outside `Broadcast`.

**Fact — the workload evidence, from the L2 derivation.** [The transformer operation and shape surface derivation](../docs/research/shapes/transformer-operation-and-shape-surface.md) counts 56 sequence-extending state writes per forward pass of the pinned `Qwen/Qwen3-0.6B-Base` profile — one for `K` and one for `V` in each of 28 layers. Each appends `T` positions of shape `[T, 8, 128]` F32 to a cache holding `S - T` positions, with `S` bounded at 18 for the conformance row and 8,320 for the benchmark matrix. L1 records the arithmetic: 229,376 F32 bytes per cached token across the whole model.

**Inference — the two mechanisms are not implementations of one design.** A `Concatenate` produces a new value of a larger extent and leaves the physical planner to decide whether that is a copy; a windowed write mutates a buffer whose valid range is state. They differ in semantic identity, in whether the operation is pure, in what the index verifier must prove about write ownership, and in whether the growing extent is a shape symbol or a runtime-tracked capacity. Choosing between them by whichever is easier to schedule would settle a semantic question with a physical argument.

## Required analysis

- State what each mechanism would owe: identity, validation, purity or effect declaration, access relation, write-ownership proof, and the extent-symbol treatment of a growing axis.
- Decide between them, or state exactly what evidence would decide it, running the elimination explicitly rather than presenting two options.
- Record whether a general `Concatenate` and a general `Slice` are wanted at all, since the same derivation shows the workload needs neither inside a layer: the rotary half-split reduces to a bijective split, a permutation, a broadcast multiply, and a merge.
- Add the resulting rows to the support matrix with their rungs and triggers, so the absence is tracked whichever way the decision goes.

## Non-goals

The KV-state model itself — capacity, valid range, growth policy, placement, aliasing, retention, and lifetime — which [`design-autoregressive-state-and-kv-cache`](design-autoregressive-state-and-kv-cache.md) owns at rung L5. This ticket settles the *semantic family* that state model will invoke, and hands it a named mechanism instead of an open one. It implements nothing.

## Reconsideration trigger

Active now for the matrix rows, which record an absence that exists today regardless of the decision. The mechanism decision may reasonably wait for L5 to state its state model, in which case this ticket narrows to the enumeration and L5 inherits the choice with the analysis already done — but it may not stay unrecorded, because an unenumerated family is invisible to every reader who checks the matrix.

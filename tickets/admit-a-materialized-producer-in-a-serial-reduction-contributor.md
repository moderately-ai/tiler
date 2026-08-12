---
id: admit-a-materialized-producer-in-a-serial-reduction-contributor
title: Admit a materialized producer in a serial-reduction contributor
status: todo
priority: p3
dependencies: [name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set]
related: [name-the-fold-prologue-chain-boundary-instead-of-reporting-operation-set, admit-a-recognized-chain-more-than-one-materialization-boundary-deep]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [research, compiler, planner, identity]
---
## User-visible outcome

A strict serial reduction can consume contributors computed across a materialization boundary, such as `sum(sum(x) * 2)` or `sum(contract(a, b) * 2)`, without flattening away the producer or changing the program's numerical meaning.

## Facts at filing — 2026-08-12, base `0a67f558`

**Fact — recognition finds the boundary and discards it.** At the cited base, `plan_elementwise` returns `ElementwiseRefusal::Folded(ValueId)` when a serial reduction's contributor walk reaches a strict reduction, contraction, or registered staged family, and `recognize_reduction` reaches that result only through `recognize_elementwise`. The paired accepted diagnostic ticket changes the conversion from the stale `operation-set` classification to `reduction-contributor-materialization`; it does not retain the producer.

**Fact — this is not the staged-family depth guard.** `StagedOperandAdmission::NoEdge` governs a staged family reached across a materialization edge. The serial-reduction path never consults that guard; it fails earlier because `NormalizedSerialSum` carries an optional pointwise expression but no producer relation. `admit-a-recognized-chain-more-than-one-materialization-boundary-deep` remains a distinct deferred boundary.

**Fact — several producer families expose one missing relation.** `materializes_its_result` recognizes strict serial reduction, strict tensor contraction, and every registered region-sequence family. Admission must model one serial-reduction contributor supplied by a materialized producer rather than specialize the normal form around a producer key.

**Fact — the existing accepted neighbor is shallower.** A reduction over a pointwise expression of declared inputs already compiles because its contributor expression can be retained directly as the optional prologue. An elementwise epilogue over a materialized producer also compiles because `NormalizedEpilogue` carries a producer. Neither shape supplies the missing serial-reduction producer field.

## Research and decision required

Before implementation, derive and compare the smallest exact normal form that retains the producing `NormalizedOutput`, the elementwise contributor continuation, their materialization edge, occurrence partition, numerical materialization boundary, subject encoding, cover formation, and physical/KIR consequences. Audit recursion and deterministic work bounds: the current producer forms are recursive, and accepting caller-proportional nesting without an iterative or explicitly bounded representation would turn program depth into host-stack risk.

The design must preserve the producer's own semantic and numerical identity, prove producer-before-consumer ordering, and refuse unsupported producer/continuation combinations by name. It must never synthesize a declared-input baseline, flatten the boundary, reuse a nearby pointwise prologue, or fall back after a failed admission.

## Non-goals

Producer-specific diagnostic keys, widening a staged family to read two materialization edges, arbitrary chain depth, backend emission, or performance selection.

## Activation and closure

Move this ticket to `awaiting-decision` only after the complete construction and consumption census identifies a bounded, injective carrier and its identity/schema consequences. Close only when at least the nested-reduction, contraction, and staged-family subjects are either admitted through that shared carrier or each refused under a narrower named prerequisite, with the declared-input neighbor unchanged.

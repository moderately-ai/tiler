---
id: scope-the-effect-signature-opening
title: Scope the effect-signature opening for implicitly stateful operations
status: deferred
priority: p2
dependencies: []
related: [multi-device-and-sharding-scope-gate, scope-the-non-tensor-value-kinds-and-control-constructs, scope-the-counter-based-random-generation-family, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, effects, deferred]
---
## User-visible outcome

[Q-SEM-011](../docs/open-questions.md#q-sem-011--semantic-effects-and-resource-tokens) acquires a ticket: when the first genuinely effectful operation is proposed, the ordering, liveness, verification, ABI, and partial-execution rules an effect signature owes are scoped as one thing rather than discovered by whoever widens the enum.

## Why this is deferred rather than open

**Fact — non-representability here is a chosen mechanism, not an absence.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-44 is "Neither, today, and deliberately so". `OperationEffect` has exactly one variant, `Pure`, and is deliberately not `#[non_exhaustive]`: three total identity encoders map the vocabulary exhaustively onto an identity tag — `tiler_compiler::legality::effect_tag`, `tiler_compiler::fusion_legality::effect_tag` (both outside `tiler-ir`), and this crate's registry definition encoder (inside `tiler-ir`) — so adding a variant is a build error at each of them rather than a silent re-encoding. A fourth total match sits in the `tiler-ir` index refinement subject encoder. That is "the difference between 'we have not built this' and 'the system will tell you when someone tries'".

**Correction — 2026-08-10.** Earlier wording said "three encoders outside `tiler-ir`"; the fail-closed exhaustive-match mechanism is correct, but the census of locations was imprecise — only two of the three declaration-named total mappers live outside `tiler-ir`, and a fourth total match exists in index refinement.

**Fact — the boundary is explicitly temporary and the opening mechanism is named.** [Operation extensions](../docs/operation-extensions.md) states this "is a capability boundary rather than a permanent exclusion" and that "the durable operation and value model reserves a separately versioned effect signature and resource/effect-token value kinds". So the row is a reservation with a known opening mechanism, not a closed door.

**Fact — floating-point environment observation is excluded on independent grounds.** [ADR 0020](../docs/decisions/0020-value-only-floating-point-exceptions.md)'s value-only exception model is precisely what makes environment observation inexpressible, and F-16's classification predicates are the admitted way to *observe* exceptional values without an environment.

**Inference — three reservations are commonly confused and this ticket carries exactly one.** The taxonomy's conclusion 7 separates them: "Effect, region, and non-tensor-value support are three separate reservations, not one 'advanced features' bucket. Sorting needs a region and no effect; scatter needs neither; collectives need effects and tokens; control flow needs regions and changes what determinism means." This ticket is the effect reservation only.

## Activation trigger

Q-SEM-011's own trigger: the first stateful, mutating, or hidden-random operation proposal. Three near misses do **not** fire it, and naming them is the point — a scatter is pure, a counter-based generator is pure, and an in-place append is a physical buffer-reuse question under [Q-PLAN-015](../docs/open-questions.md) rather than a semantic effect.

## What the work would be, when it starts

Scope the effect signature as one versioned thing: which effect classes exist, the ordering relation between effectful occurrences, liveness and the lifetime of any effect token, what the verifier must prove, what the ABI carries, and — the one most easily skipped — the partial-execution rule, because an effectful program that fails midway has observably done something. Then state the encoder consequence explicitly: the declaration-named total mappers (`tiler_compiler::legality::effect_tag`, `tiler_compiler::fusion_legality::effect_tag`, and the `tiler-ir` registry definition encoder) plus the index refinement subject encoder all map the vocabulary totally without a wildcard, so the widening is a coordinated identity-domain step rather than an enum edit.

## Explicit non-goals

- Widening `OperationEffect` ahead of the scoping. The compile error at every total encoder of the vocabulary is the mechanism working, not an obstacle to route around.
- Regions and control flow, which are [`scope-the-non-tensor-value-kinds-and-control-constructs`](scope-the-non-tensor-value-kinds-and-control-constructs.md)'s.
- Collectives, which need effects *and* tokens and are [`multi-device-and-sharding-scope-gate`](multi-device-and-sharding-scope-gate.md)'s.
- In-place execution, which is Q-PLAN-015's and is physical.

## Closes when

Q-SEM-011's five closure items — ordering, liveness, verification, ABI, and failure rules — are stated together against a named first effectful proposal, and the identity-domain consequence of widening the vocabulary is enumerated before anything is widened.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-37** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-44 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No stateful, mutating, or hidden-random operation is proposed. Recheck: `rg -n 'enum OperationEffect' -A6 crates/tiler-ir/src/semantic/operation.rs` — the vocabulary still has exactly one variant, `Pure`, and is deliberately not `#[non_exhaustive]`.
- 2026-08-09 — **not fired.** `OperationEffect` remains the closed one-variant `Pure` vocabulary. The live caller-retained append question is still explicitly physical buffer reuse with a recovery contract, and counter-based randomness remains pure explicit state; neither is the first semantic effectful operation proposal.
- **Recheck restored — 2026-08-22; no verdict re-decided here.** The entry above states its verdict in prose and names no command, so AGENTS.md's per-entry obligation — a verdict *plus a reproducing command* — was carried forward unmet. Restored from this log's own history rather than invented: the most recent command this log names is `rg -n 'enum OperationEffect' -A6 crates/tiler-ir/src/semantic/operation.rs`, and run at this base it returns **7** lines. A result other than the 7 recorded here is the changed answer. Whether the trigger has fired is deliberately not re-decided here; that reading belongs to [`refresh-the-deferred-triggers-whose-stated-reason-is-now-false`](refresh-the-deferred-triggers-whose-stated-reason-is-now-false.md).

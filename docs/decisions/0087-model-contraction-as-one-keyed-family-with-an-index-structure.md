---
schema: "tiler-doc/v1"
id: "ADR-0087"
kind: "decision"
title: "Model contraction as one keyed family with an index structure"
topics: ["semantics", "operation-families", "contraction", "identity", "validation", "language-model"]
catalog_group: "foundation-semantics-extensions"
decision_status: "accepted"
implementation_status: "partial"
applies_to: ["tiler.contract.ir"]
evidence: ["tiler.research.shapes.transformer-operation-and-shape-surface", "tiler.research.program-planning.first-metal-lm-workload"]
ticket: "decide-whether-a-contraction-is-one-keyed-family-or-fixed-arity-keys"
---

# 0087: Model contraction as one keyed family with an index structure

**Status:** accepted. Tom decided this on 2026-07-31, in session, after a two-sided implications walkthrough, with an explicit rider that the implementation take no shortcuts and fail nowhere silently: the safety this decision buys must be delivered at build time. It settles the first of the three choices [Q-SEM-015](../open-questions.md#q-sem-015--tensor-contraction-matmul-batched-matmul-and-einsum) reserves; the multi-operand and distributivity choices remain open and are deliberately not bundled here.

## Context

**Fact — the question was reserved with the costs stated both ways.** The [Milestone 6 framing](../roadmap.md#framing-what-a-tensor-contraction-family-would-impose) reserved whether a tensor contraction is one keyed family carrying an index-structure attribute — one validation authority, one family for normalization and region formation, at the cost of a large attribute schema and an inference routine that must reject every malformed structure — or a set of fixed-arity keys per shape class, each small and exactly checkable, at the cost of registry growth and of normalization having to relate keys that denote one computation.

**Fact — the demand trigger fired and the workload evidence is exact.** The [L2 derivation](../research/shapes/transformer-operation-and-shape-surface.md) resolves all 253 contraction occurrences per forward pass of the pinned `Qwen/Qwen3-0.6B-Base` workload into exactly three index structures: `td,od->to` at 197 occurrences spanning all six weight shape classes, `grtd,gsd->grts` at 28, and `grts,gsd->grtd` at 28. None is `[M, K] x [K, N]`, because the checkpoint stores every projection weight `[out_features, in_features]`, so the contracted index is the last axis of both operands; fixed matmul keys would force a transposing `Reindex` on all 197 weighted occurrences or a pre-transposed copy of 2.22 GiB of F32 weights. The grouped-query 8→16 repetition is free under a structure-carrying contraction — the repetition index simply does not appear in the key operand — and must be materialized as a `Broadcast` plus `Reindex` with a `[16, S, 128]` intermediate under fixed-arity keys.

**Fact — the transpose and materialization costs are partially recoverable under fixed keys, and were therefore not the deciding axis.** A transposing `Reindex` composes into an access map that a planner can erase into kernel addressing, and a broadcast can in principle fuse into an operand's access relation — provided that fusion machinery handles every occurrence, which is itself unbuilt machinery the fixed-key option would newly depend on. The decision was taken on the axes that do not recover.

## Decision

A tensor contraction is **one keyed semantic operation family** whose node carries its index structure — per-operand index tuples, the output tuple, and the contracted set — as a **strongly typed attribute** that participates in canonical identity.

1. **The canonical encoding of the structure is renaming-invariant.** Two spellings of one structure that differ only by index renaming produce identical canonical bytes; the canonicalization (index numbering by canonical first appearance) is part of the identity authority, is domain-separated and exhaustively encoded like every other identity in this corpus, and is mutation-proved: a perturbation that makes two distinct structures encode equally, or one structure encode two ways, must be demonstrated failing before the encoder is trusted. An identity collision here is the silent-wrongness class this repository exists to prevent, and the test obligation is stated now, with the decision, rather than discovered at implementation.
2. **The five structural admission rules reject at construction, each under its own named rule.** No output index absent from every operand; no summed index in only one operand; no index repeated within one operand; no duplicated output index, with each output order a permutation of the structure's free indices; no index in more than two operands (the multi-operand question stays reserved, and this rule is where its future answer lands). A malformed structure is a typed refusal naming the violated rule, never a generic invalidity, and never a value that reaches identity, planning, explain output, or a cache subject.
3. **Frontends never choose among contraction keys, because there is only one.** The structure analysis lives in the validator, once, governed; a frontend states the structure and the validator says yes or no. This is the build-time-safety half of the decision: under fixed-arity keys the key choice would be per-frontend, ungoverned analysis whose mistakes the compiler cannot see.
4. **An unsupported structure fails closed at lowering-capability resolution.** Admitting the family does not claim every structure is realizable: a structure no installed capability covers is a typed resolution refusal, exactly as contraction occurrences fail closed today. Growth is a validator widening plus a capability entry; the key, registry identity, ABI, and artifact identity domains do not move.
5. **The embedded reduction's numerical signature is stated once, generically.** Computation and input precision, accumulator dtype, result dtype, conversion behaviour, and an order contract — a strict lexicographic fold over the canonically ordered contracted index space unless a registered permission authorizes otherwise — parameterized by the structure rather than restated per shape class. Reassociation, permutation, and the absent distributivity dimension keep their existing meanings and owners.

## Consequences

- The semantic half of an attention einsum needs **one governed key**, not a key per shape class; the workload's three structures are three attribute values under it.
- Rung L3's activation trigger — "L2 lists the contraction shapes, and milestone 6 settles the keyed-family question" — is now fully satisfied. L3 remains gated by the planning half exactly as [Q-SEM-015](../open-questions.md#q-sem-015--tensor-contraction-matmul-batched-matmul-and-einsum) states: no contraction planning before the optimizer conformance gate closes.
- The admission itself — registering the key, the inference routine, the access-relation emission, reference semantics, and conformance evidence — is future implementation work this record authorizes the *shape* of, not the start of; the research/implementation boundary is unchanged.
- Einsum-shaped growth is a structure widening under one identity, with no migration horizon; the fixed-key option's later generalization would have rebaselined every artifact identity, cache subject, and golden that named a fixed key.

## Alternatives considered

**Fixed-arity keys per shape class.** Genuinely simpler per key: positional shape checks, trivial reference evaluators, concrete per-key numerical contracts, and no canonicalization problem at all. Rejected on the three costs that do not recover: the structure analysis moves into every frontend as ungoverned key choice; the key set grows without bound, each key a full vertical of registry, reference, conformance, ABI, and identity obligations — this one workload already needed three; and einsum generality, which Q-SEM-015 explicitly contemplates, would migrate every fixed key's identity later, which is the temporary-becomes-permanent-then-expensive pattern the decision criteria name. The recoverable runtime costs were disclosed as recoverable and did not carry the decision.

**Defer with a trigger.** Rejected because the demand trigger had already fired — the pinned workload requires 253 contractions per forward pass — and deferral would have left rung L3 blocked on a question whose evidence was complete.

## Traceability

This record settles the first reserved choice of [Q-SEM-015](../open-questions.md#q-sem-015--tensor-contraction-matmul-batched-matmul-and-einsum) and item 1 of the [Milestone 6 framing's reserved decisions](../roadmap.md#decisions-reserved-for-tom); the multi-operand choice (item 2) and the distributivity choice ([`decide-whether-to-admit-a-distributivity-permission`](../../tickets/decide-whether-to-admit-a-distributivity-permission.md)) remain reserved. The workload evidence is the [transformer operation and shape surface derivation](../research/shapes/transformer-operation-and-shape-surface.md) over the [first Metal language-model workload profile](../research/program-planning/first-metal-lm-workload.md). The [support matrix](../roadmap.md#operation-family-support-matrix) contraction row stayed at R1 while this decision was only accepted — identity decided, nothing admitted. Correction, 2026-07-31: [`admit-the-contraction-semantic-profile`](../../tickets/admit-the-contraction-semantic-profile.md) registered `tiler::strict-tensor-contraction-f32@1` with the canonical structure encoding, the five structural refusals, and the item-5 numerical signature this decision requires, and the row moved to R3. The decision itself is unchanged; what moved is the implementation state it reports, and everything above R3 — evaluator, fusion role, lowering, backend — remains absent and fails closed.

---
id: integrate-the-contraction-vertical-into-the-runtime
title: Run one profile contraction end to end through the AOT and runtime route
status: todo
priority: p1
dependencies: []
related: [design-attention-program-vertical, prototype-metal-runtime-proof, prototype-metal-aot-slice, realize-the-tiled-contraction-schedule-and-its-metal-emission]
scopes: [implementation/runtime, implementation/metal-aot, implementation/artifact, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, artifacts, contraction, language-model]
---
## User-visible outcome

Rung L3's stated capability — "one contraction runs end to end on Metal" — becomes true through the accepted AOT and runtime route rather than through a spike's own dispatch host. This is the remainder the L3 record deliberately did not claim.

## What is already true, and what is not

**Fact.** The [L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md) measured six realizations under a hand-written Objective-C host that loads a metallib, dispatches, checks `MTLCommandBufferStatusCompleted`, and reads back. That is a spike, not the route: it produces no artifact, has no identity, resolves no capability, and answers no applicability predicate.

**Fact — the route it must use instead.** An offline-produced metallib loaded through the accepted AOT path, with artifact identity carrying the offline compiler's provenance and exact native translator identity remaining `Unknown` per [ADR 0086](../docs/decisions/0086-require-attributable-or-attested-native-translation.md). The source-JIT compiler build measured elsewhere is not an input to this route and must not be substituted into its identity.

## Required delivery

- Artifact planning, ABI derivation, and buffer planning for a two-input one-output contraction — the first program in the project with two tensor inputs, so every place that assumed one is a place to check rather than to trust.
- Preflight before routing commit, and no fallback after allocation, partial encoding, submission, or semantic validation failure.
- Exact command-buffer terminal success before host validation readback.
- Bit-comparison of the executed result against the reference evaluator, with the spike's retained `result_sha256` values at the profile's cells available as an independent cross-check on a matching host row.
- Retention of asynchronous resources through their final device use.

## Non-goals

A transformer block, an attention program, the KV cache, batching, or more than one contraction in one program. L4 owns the block; this ticket owns making one contraction reach the device through the real route.

## Closes when

One contraction of the L3 profile executes through the accepted route with a terminal-success check before readback, its result is bit-identical to the reference, and a deliberately corrupted artifact is refused rather than executed.

## Dependency corrected at the third tiled stop (2026-08-01)

The supersede recipe re-pointed this ticket from `realize-the-strict-contraction-on-metal` onto the deferred tiled chain, and the coordinator reversed that edge on reading this ticket's own outcome: "one contraction runs end to end on Metal" is a claim about the accepted AOT and runtime route, not about which realization rides it, and the `direct` realization — compiled through the ordinary entry point, bit-compared at the profile cells — is a complete vehicle for it. The tiled realization arrives later as the performance-selected alternative behind `realize-the-tiled-contraction-schedule-and-its-metal-emission` (kept as related), and integrating on `direct` first is exactly the multi-kernel-may-be-correct-and-faster posture the architectural contract states.

---
id: define-the-model-execution-state-boundary
title: Define the model execution state boundary
status: closed
priority: p1
dependencies: [assemble-the-decoder-layer-program, define-the-runtime-kv-state-boundary, reclassify-language-model-work-as-a-conformance-track, supersede-the-runtime-owned-kv-state-design]
related: [design-model-ingestion-and-complete-execution, design-autoregressive-state-and-kv-cache, drive-the-complete-forward-pass-over-three-artifacts, scope-an-in-place-append-into-a-caller-retained-allocation]
scopes: [contracts/integrations, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [design, runtime, state, kv-cache, lifetime, public-boundary, language-model, supersession, class-obsolete]
closed_reason: superseded
closed_note: Model-level instantiation of the withdrawn runtime KV state; the consumer owns the cursor and retained tensors.
---
## Superseded — 2026-08-04

**This ticket is closed as superseded by
[`supersede-the-runtime-owned-kv-state-design`](supersede-the-runtime-owned-kv-state-design.md).
It satisfies no dependent.** It was the model-level instantiation of the
runtime-owned KV state, and it cannot outlive it: every one of its sections
began "instantiate the generic `KvStateSet`", and that type is withdrawn.

### What it asked for

One named Tiler object holding 28 ordered K/V pairs (56 logical members), one
model cursor, a generation per member, a token-granular transaction publishing
all 56 replacements after one observed terminal success, a model-level poisoned
status, and typed refusals for a missing, extra, duplicated, or mispaired layer
member. It carried L6's **D-16** — whether the transaction boundary ever moves
from the token to the layer — as the question for Tom.

### Why it is superseded

Tiler retains nothing between invocations, so there is no Tiler object to hold a
cursor over 28 layers, no set to poison, and no membership for Tiler to check.
The *property* the boundary protected is real and is unchanged: no reader may
observe a model advanced for some layers and not others. It is now discharged
where the tensors actually live — a consumer that advances all 56 of its own
retained tensors and its own cursor together, on one reported terminal success,
makes the partially-advanced state unrepresentable on its side rather than
refused on Tiler's. That is a stronger position than a refusal, because it
removes the state rather than checking it.

### Where its content went

- **The granularity rule and the token transaction** are recorded as consumer
  obligations in
  [the L6 ownership table](../docs/research/program-planning/complete-model-ingestion-and-execution.md),
  corrected in place on 2026-08-04.
- **The 30-execution ordering, the single completion observation, and the
  no-fallback-after-commit rule** were never state properties and are owned by
  [`drive-the-complete-forward-pass-over-three-artifacts`](drive-the-complete-forward-pass-over-three-artifacts.md),
  which is unaffected and now depends on this ticket no longer.
- **The typed model-level failure report** — execution ordinal, phase, token in
  flight, and the operations a numerical claim covers — is
  [`name-the-execution-ordinal-in-model-level-failures`](name-the-execution-ordinal-in-model-level-failures.md)'s
  and is generic: an invocation reports which invocation failed, without holding
  state.
- **D-16** stays open as a research question in the L6 record. Its subject
  becomes the consumer's own allocation policy — whether a consumer ever
  publishes per layer instead of per token — and it needs the same two halves it
  always did: a measured binding cost at a B1 row *and* a recovery contract.
  No Tiler boundary gates it, so no ticket carries it as deliverable work; it is
  recorded as L6's D-16 with its trigger intact.

### What would reopen the question

A consumer-neutral reason for the runtime to retain typed state across
invocations. Nothing model-shaped qualifies.

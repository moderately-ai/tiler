---
id: select-the-first-general-elementary-function-keys
title: Select the first general elementary-function keys
status: deferred
priority: p2
dependencies: []
related: [derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, numerics, transcendentals, deferred]
---
## User-visible outcome

Q-SEM-004's remaining half is discharged: a named set of general `Exp`, `Log`, `Sin`, or `Gelu` keys enters the first profile with its dtype and accuracy tuples chosen, instead of three composite families each declaring the accuracy of a subordinate elementary function under its own key and no general one existing.

## Why this is deferred rather than open

**Fact — the machinery is delivered and the selection is not.** [Q-SEM-004](../docs/open-questions.md#q-sem-004--first-profile-transcendental-tuples) was restated on 2026-08-04: both reasons it gave for staying open were discharged, the cross-metric implication is registered once with its derivation attached and reused by a second operation, `crates/tiler-reference/src/accuracy.rs` supplies the certified enclosures and the three-way conformance decision, and "what this question still owns is the *selection* it always named — which general operation, dtype, and accuracy tuples enter the first profile, and the exceptional-value contract each owes."

**Fact — three landings each minted no general key, deliberately, and the checks that hold that line are named.** `no_general_exponential_or_sigmoid_key_is_registered`, `no_layer_normalization_rsqrt_mean_or_bias_key_is_registered`, and `no_general_exponential_maximum_reduction_or_log_softmax_key_is_registered`. [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s transcendental row records the same from the delivery side and holds it at R2.

**Fact — the accuracy and exceptional-value obligations are independent, and this is where that rule bites hardest.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-11 requires "an accuracy contract under [ADR 0016](../docs/decisions/0016-transcendental-accuracy-contracts.md) and [ADR 0042](../docs/decisions/0042-use-typed-transcendental-accuracy-contracts.md), plus an **independent** exceptional-value policy; an accuracy bound never implies special-value behaviour", and records that "a target with no published bound yields `Unknown`, which fails closed rather than defaulting to the vendor's behaviour".

**Inference — the deferral rests on an elimination rather than on silence.** Three families that each needed an exponential or a reciprocal square root were delivered without one, and the third proved the machinery reusable — one registered implication now serves two operations, and `the_softmax_needs_no_second_registered_implication` proves it by stripping the registry and watching the declaration stop refining. What could *not* be reused was the contract instance, for a structural reason: an `AccuracyContract` carries the `OpKey` it speaks about. So a general key is not a refactor of what exists; it is a new selection.

## Activation trigger

A named consumer requires a general elementary function as an operation rather than as a subordinate step inside a composite family. A fourth composite family needing its own subordinate exponential does **not** fire it — that is the case the three delivered families already cover, and covering it again is evidence that the general key is still not needed.

## What the work would be, when it starts

Choose the tuples — operation, dtype, accuracy contract form, and admitted domain — and for each state the exceptional-value contract separately from the bound, because ADR 0042's independence rule means an entry with a stated ULP envelope still owes NaN, infinity, and subnormal behaviour. Then check the two derivations the delivered families supplied and reuse or refuse them explicitly: the cross-metric factor registered as `RegisteredImplication::ScaledMetric`, and the observation that a metric-free faithful contract does not bind Metal §8.2's rounding-mode question at all. A general key whose domain is not confined the way the softmax's is will re-enter the overflow band the activation's contract had to close above, so the domain is part of the selection rather than a detail of it.

## Explicit non-goals

- A second cross-metric implication registered by copying the first. One row serves two operations today and a third must be shown to need its own before it gets one.
- Any decomposition of a composite family into general keys. The three delivered families pin numerical decisions a composition would leave open, and replacing them would be a different graph identity.
- A vendor bound adopted without a stated derivation. Apple states its bound under its own ULP definition, which is a different metric key.

## Closes when

One or more general keys exist with their tuples selected, their exceptional-value contracts stated independently of their bounds, a reference evaluator, and a backend realization whose declared guarantee refines the contract — and Q-SEM-004 is closed or restated to name only what remains.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-20** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-11 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No named consumer requires a general elementary function as an operation; every occurrence in the pinned workload is subordinate to `tiler::silu-f32@1`, `tiler::rms-norm-f32@1`, or `tiler::softmax-f32@1`, and each of those carries a check asserting it minted no general key. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
- 2026-08-09 — **not fired.** The governed semantic vocabulary has grown, but no named consumer requires `Exp`, `Log`, `Sin`, or `Gelu` as a standalone operation. The delivered SiLU, RMS-normalization, and softmax families still own their subordinate elementary-function requirements under their composite keys, so another vocabulary census is not evidence that the selection trigger fired.

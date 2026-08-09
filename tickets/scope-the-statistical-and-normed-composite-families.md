---
id: scope-the-statistical-and-normed-composite-families
title: Scope the statistical and normed composite families
status: deferred
priority: p3
dependencies: []
related: [derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, reductions, numerics, deferred]
---
## User-visible outcome

`RQ-OP-08` is answered for mean, variance, and log-sum-exp: each is either an atomic family pinning a stable formulation, or a composition whose naive form the corpus accepts — decided by exhibiting an input where the two differ, not by taste.

## Why this is deferred rather than open

**Fact.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-30 classifies the group as "composition over governed keys where the accumulation is exposed; atomic where it is not — `RQ-OP-08`", records that its accumulator type "usually differs from the input type, which is why the composition can be numerically wrong", and closes with the sentence that names the whole difficulty: "This is the group where 'it decomposes' and 'it is correct' most often disagree."

**Fact — `RQ-OP-08`'s closure test is concrete and reusable.** "Closes by exhibiting, for each, an input on which the naive composition and the stable formulation differ in the result type at a magnitude the conformance corpus would catch. If they differ, the family is atomic — the reasoning is F-12's, applied to a second group."

**Fact — F-12's reasoning is delivered three times over and is the precedent this track inherits.** [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) records `tiler::softmax-f32@1` pinning "the four decisions a composition would leave open", `tiler::rms-norm-f32@1` pinning that nothing is subtracted and where `eps` sits, and `tiler::silu-f32@1` pinning its formula. The general rule the taxonomy derives from them is that "a family is atomic when a composition would leave a *numerical* decision to whichever pass happened to run".

**Inference — the answer is likely atomic and that is exactly why it must not be assumed.** A track that pre-decided atomicity would register three families on an analogy; the closure test costs one worked input per family and produces evidence instead.

## Activation trigger

A named workload requires a general mean, variance, standard deviation, norm, or log-sum-exp as an operation. The matrix's transcendental row already records the nearest near miss and its exclusion: `tiler::rms-norm-f32@1` mints "**no** general `Rsqrt` or `Sqrt` key, and a `LayerNorm` and a general mean with it", all of which were that ticket's stated non-goals.

## What the work would be, when it starts

Run `RQ-OP-08`'s test per member: construct the input where the naive composition and the stable formulation differ in the result type, at a magnitude a conformance corpus would catch, and record it. Where they differ, state the family's pinned formula, its accumulator type as an operation fact under the accepted rule rather than a schedule choice, its reduced-axis and correction-or-order parameters, and its possibly two-pass physical route. Where they do not, record that the composition is admitted and say which governed keys it is a composition *over*, since several of them do not exist yet.

## Explicit non-goals

- `LayerNorm` as a fourth normalization key. It is a composite whose atomicity this test decides; naming it before the test would answer the question by filing it.
- Any reassociation permission. A stable formulation is a different expression, not a regrouped one.
- Widening the delivered normalization family. `tiler::rms-norm-f32@1`'s `eps` is part of its identity and a general mean is a different operation.

## Closes when

Each of mean, variance, and log-sum-exp is classified atomic or composition against a worked discriminating input, and the atomic ones have a pinned formula with an accumulator declared as an operation fact — or the group is recorded as unneeded with the consumer that would have needed it named.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-26** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-30 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No workload requires a general mean, variance, norm, or log-sum-exp; the three delivered composite families each mint no general key and name that as an explicit non-goal. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.
- 2026-08-09 — **not fired.** RMS normalization and softmax now have deeper staged realization support, but their mean/squared-sum/log-sum-exp pieces remain internal to those exact composite keys. No general mean, variance, norm, or log-sum-exp consumer is named.

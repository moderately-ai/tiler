---
id: scope-the-standalone-extrema-and-clamp-families
title: Scope the standalone extrema and clamp families
status: deferred
priority: p2
dependencies: []
related: [admit-a-parallel-topology-for-the-identity-less-extrema-fold, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, numerics, extrema, deferred]
---
## User-visible outcome

`Minimum`, `Maximum`, `MinimumNumber`, `MaximumNumber`, and `Clamp` exist as operations a program can name, instead of as an embedded fold inside one composite family and four accepted ADR paragraphs with no key.

## Why this is deferred rather than open

**Fact.** [ADR 0023](../docs/decisions/0023-floating-point-extrema-semantics.md) accepts the propagating and number-preferring families as *separate semantic operations* with a deterministic `-0.0 < +0.0` ordering, rather than one mode-selected operation. [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-10 adds clamp as the same family at arity three and records that "a clamp whose bounds are not ordered is a construction refusal, not a defined empty interval", and that vendor `fmin`/`fmax` "are number-preferring with order-dependent NaN behaviour" — so neither family lowers to the obvious intrinsic.

**Fact — one form of one of the four is delivered, embedded, and the matrix row says exactly what that did and did not move.** `tiler::softmax-f32@1` carries the propagating `Maximum` as its first fold, with `ScalarProgram::StrictSerialMaximum` and `BinaryOp::F32Maximum`, lowered by an exact fixup built from ordered comparisons and a bitwise `and` rather than by `air.fmax.f32`. [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s extrema row records that "**No key exists for any of the four families and the rung does not move**", on its own rule: "the elementwise and reduction forms name one scalar family but retain separate identity, seed, and order contracts, so admitting one does not admit the other".

**Inference — the delivered fixup is the expensive half and it is reusable, which is an argument for filing this track and against starting it early.** The emission that implements the total order exists and is tested; what does not exist is any of the five identities, their seeds, or their order contracts. Admitting them now would fix five schemas around one embedded occurrence.

## Activation trigger

Clamp or ReLU recognition, or a *standalone* extrema reduction, enters a profile — the matrix row's own trigger, unchanged. A parallel topology for the existing embedded fold does **not** fire it: that is [`admit-a-parallel-topology-for-the-identity-less-extrema-fold`](admit-a-parallel-topology-for-the-identity-less-extrema-fold.md)'s and it admits no key.

## What the work would be, when it starts

Per family: the key, the total order stated as a semantic field rather than a target detail, the NaN rule that distinguishes the propagating family from the number-preferring one, the seed and empty-domain rule for the reduction form (no binary32 value is an identity for `Maximum`, which is why the delivered scalar program carries no `empty_identity_bits` field at all), the clamp's unordered-bounds refusal, and the emission — which must reuse the delivered exact fixup rather than mint a second one, and must be shown to reuse it rather than asserted to.

## Explicit non-goals

- A mode attribute selecting between the two families. ADR 0023 accepted them as separate operations and this ticket does not reopen that.
- The parallel topology for the embedded fold, which has its own live owner.
- A `ReLU` key. A clamp at zero is an occurrence of clamp, not a sixth family.

## Closes when

The five identities exist with their own seeds and order contracts, the emission reuses the delivered fixup, and a conformance corpus distinguishes the two NaN families on an input where they differ — or the standalone forms are recorded as permanently subsumed by composite families, which would need the subsumption argument the matrix row currently refuses to make.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-19** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-10 alone, in both its elementwise and its standalone-reduction forms and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No clamp or ReLU recognition and no standalone extrema reduction has entered a profile; the only extrema in the corpus is the softmax's embedded fold, which the matrix row records as not moving this rung. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.

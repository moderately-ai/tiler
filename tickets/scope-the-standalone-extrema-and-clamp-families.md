---
id: scope-the-standalone-extrema-and-clamp-families
title: Scope the standalone extrema and clamp families
status: deferred
priority: p2
dependencies: []
related: [admit-a-parallel-topology-for-the-identity-less-extrema-fold, derive-the-operation-family-and-signature-delivery-graph, scope-the-monoid-reducers-beyond-the-strict-sum]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, numerics, extrema, deferred]
---
## User-visible outcome

`Minimum`, `Maximum`, `MinimumNumber`, `MaximumNumber`, and `Clamp` exist as *elementwise* (and clamp arity-three) operations a program can name, instead of as an embedded fold inside one composite family and four accepted ADR paragraphs with no key. Standalone extrema *reduction* under F-28 is not this ticket's outcome; that form is owned by [`scope-the-monoid-reducers-beyond-the-strict-sum`](scope-the-monoid-reducers-beyond-the-strict-sum.md) (O-39).

## Why this is deferred rather than open

**Fact.** [ADR 0023](../docs/decisions/0023-floating-point-extrema-semantics.md) accepts the propagating and number-preferring families as *separate semantic operations* with a deterministic `-0.0 < +0.0` ordering, rather than one mode-selected operation. [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-10 adds clamp as the same family at arity three and records that "a clamp whose bounds are not ordered is a construction refusal, not a defined empty interval", and that vendor `fmin`/`fmax` "are number-preferring with order-dependent NaN behaviour" — so neither family lowers to the obvious intrinsic.

**Fact — one form of one of the four is delivered, embedded, and the matrix row says exactly what that did and did not move.** `tiler::softmax-f32@1` carries the propagating `Maximum` as its first fold, with `ScalarProgram::StrictSerialMaximum` and `BinaryOp::F32Maximum`, lowered by an exact fixup built from ordered comparisons and a bitwise `and` rather than by `air.fmax.f32`. [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s extrema row records that "**No key exists for any of the four families and the rung does not move**", on its own rule: "the elementwise and reduction forms name one scalar family but retain separate identity, seed, and order contracts, so admitting one does not admit the other".

**Inference — the delivered fixup is the expensive half and it is reusable, which is an argument for filing this track and against starting it early.** The emission that implements the total order exists and is tested; what does not exist is any of the five F-10 identities, their elementwise order contracts, or the clamp unordered-bounds refusal. Admitting them now would fix five schemas around one embedded occurrence.

## Activation trigger

Clamp or ReLU recognition enters a profile — the elementwise / clamp half of the matrix row's trigger. A *standalone* extrema *reduction* entering a profile is **not** this ticket's activation; that fires [`scope-the-monoid-reducers-beyond-the-strict-sum`](scope-the-monoid-reducers-beyond-the-strict-sum.md) (O-39) under F-28. A parallel topology for the existing embedded fold does **not** fire either ticket: that work is done on [`admit-a-parallel-topology-for-the-identity-less-extrema-fold`](admit-a-parallel-topology-for-the-identity-less-extrema-fold.md) and admits no key.

## What the work would be, when it starts

Per F-10 identity: the semantic key, the total order stated as a semantic field rather than a target detail, the NaN rule that distinguishes the propagating family from the number-preferring one, the clamp's unordered-bounds refusal, and the emission — which must reuse the delivered exact fixup rather than mint a second one, and must be shown to reuse it rather than asserted to. Standalone reduction seeds, empty-domain rules, and F-28 reducer instantiation are O-39's, not this track's.

**Correction — 2026-08-10.** Earlier body text claimed "no binary32 value is an identity for `Maximum`, which is why the delivered scalar program carries no `empty_identity_bits` field at all." That causal claim is false. `StrictSerialMaximum` omits `empty_identity_bits` because the empty-domain *result* is undeclared for the embedded fold (empty-result vs padding-identity remain separate under ADRs 0022 and 0025); `-inf` (`0xff80_0000`) *is* a two-sided identity / observably neutral *padding* value for the pinned combiner, but no schedule in the vocabulary pads with it and no registered embedding declares an empty-domain result. Anchor: schedule model docs on `StrictSerialMaximum` ("`-inf` is a two-sided identity of the pinned family").

## Explicit non-goals

- A mode attribute selecting between the two families. ADR 0023 accepted them as separate operations and this ticket does not reopen that.
- The parallel topology for the embedded fold; that was owned by [`admit-a-parallel-topology-for-the-identity-less-extrema-fold`](admit-a-parallel-topology-for-the-identity-less-extrema-fold.md) and is `status: done`.
- A `ReLU` key. A clamp at zero is an occurrence of clamp, not a sixth family.
- Standalone extrema *reduction* under F-28, including product-style monoid reducer schema instantiation for those reducers; that is [`scope-the-monoid-reducers-beyond-the-strict-sum`](scope-the-monoid-reducers-beyond-the-strict-sum.md)'s (O-39). Admitting an elementwise identity does not admit the reduction form, and the reverse does not admit this track.

## Closes when

The five F-10 identities exist as elementwise (and clamp arity-three) operations with their order contracts and NaN rules stated, the emission reuses the delivered fixup, and a conformance corpus distinguishes the two NaN families on an input where they differ — or the elementwise forms are recorded as permanently subsumed by composite families, which would need the subsumption argument the matrix row currently refuses to make. Standalone reduction closure is O-39's.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-19** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md). O-19 covers F-10's five identities in their elementwise and clamp forms. The standalone-reduction form of extrema is F-28 material owned by **O-39** ([`scope-the-monoid-reducers-beyond-the-strict-sum`](scope-the-monoid-reducers-beyond-the-strict-sum.md)); the matrix rule that elementwise and reduction forms keep separate identity/seed/order contracts is why they are two tracks rather than one.
- **Correction — 2026-08-10.** Filing-time Graph maintenance claimed O-19 "covers F-10 alone, in both its elementwise and its standalone-reduction forms". That dual-form claim conflicted with O-39's non-goal (elementwise is O-19's; O-39 carries only the reduction form) and with the delivery-graph F-28 → O-39 mapping for "standalone extrema". This ticket now aligns with that split; the delivery-graph O-19 prose that still reads as owning "both forms" is residual docs debt, not live ticket ownership.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No clamp or ReLU recognition and no standalone extrema reduction has entered a profile; the only extrema in the corpus is the softmax's embedded fold, which the matrix row records as not moving this rung. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys *on that day*, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys then counted; the family's key is absent from that list. **Correction — 2026-08-10:** treat the "46 / eighteen" census as dated-true for 2026-08-05 only; later rechecks use the current unique-key count.
- 2026-08-09 — **not fired.** The exact extrema fixup remains exercised only inside the softmax family; no clamp/ReLU key or standalone extrema reduction has entered a selected profile. Parallelizing the embedded fold does not create a standalone semantic family.
- 2026-08-10 — **not fired.** Same activation reading: no clamp/ReLU recognition and no standalone extrema reduction in a profile. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 47 unique governed keys at this tree (dtype identities, `tiler::ulp-reference-gap@1`, and the registered operation keys including `gather-f32`); none of `minimum`, `maximum`, `minimum-number`, `maximum-number`, `clamp`, or `relu` appears as a `tiler::…@N` semantic family key. Ownership note recorded the same day: clamp/ReLU activates O-19; standalone extrema reduction activates O-39.

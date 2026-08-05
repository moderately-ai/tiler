---
id: route-the-reserved-numeric-families-through-the-extension-boundary
title: Route the reserved numeric families through the extension boundary
status: deferred
priority: p3
dependencies: [govern-external-dtype-namespace-registration-and-equivalence]
related: [derive-dtype-family-research-tracks-from-the-mature-taxonomy, define-dtype-namespace-admission-policy, own-the-dtype-support-maturity-matrix]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, deferred, extensions]
---
## User-visible outcome

Wide and arbitrary-width integers, fixed-point, decimal fixed-point, UNORM and SNORM, and posit have one owner and one route, so none of them is mistaken for a built-in admission question or for a family it merely resembles.

## Why they are one track

**Fact.** [The admission policy](../docs/research/numerics/dtype-identity-admission-policy.md)'s `### Initial extension-only families` lists `i128`/`u128` and arbitrary-width integers; posit, quire, logarithmic, unum, rational, and arbitrary-precision families; fixed-point, decimal fixed-point, and UNORM/SNORM; and project codecs without an admitted canonical descriptor. They share exactly one route — a registered provider through the extension boundary — and no built-in admission gate. "Extension-only means a registered provider may make the identity fully verifiable and executable. It does not mean opaque or permanently unsupported."

**Fact — each is separately at risk of a false equivalence, and the taxonomy names the risk.** Fixed-point and normalized integers "are not equivalent to affine ML quantization merely because each can be implemented with integer storage and a scale". Older `posit<n, es>` research notation "is not automatically the same contract as the ratified standard", whose `quireN` exact-accumulation companion has no analogue in any admitted family. UNORM and SNORM are defined by their endpoint behaviour, which is the whole of their contract.

**Fact.** [The dtype support ledger](../docs/dtype-support.md) carries all of them in one row — "Wide or bounded integer extensions, fixed-point, UNORM/SNORM, posits, and other reserved numeric families" — as type-system reservations with no semantic, reference, or numerical authority.

## Activation trigger, per member

An exact producer and consumer for that member. Because the route is the extension boundary, the member additionally needs the registration governance [`govern-external-dtype-namespace-registration-and-equivalence`](govern-external-dtype-namespace-registration-and-equivalence.md) owes, which is why that ticket is a dependency rather than a relation.

## Closes when

Each member has either an extension provider carrying it end to end, or a recorded decision excluding it from the intended product surface. Members close independently; this ticket splits rather than waits for the last one.

## Graph maintenance

- Filed by [`derive-dtype-family-research-tracks-from-the-mature-taxonomy`](derive-dtype-family-research-tracks-from-the-mature-taxonomy.md) as track D-12 of [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md).

## Trigger check log

- 2026-08-04 — **not fired**, per member. Track D-12's trigger is checked in [Dtype-family research tracks](../docs/research/numerics/dtype-family-research-tracks.md):225: no member has an exact producer and consumer, and the route additionally needs the registration governance [`govern-external-dtype-namespace-registration-and-equivalence`](govern-external-dtype-namespace-registration-and-equivalence.md) owes, which this sweep also found unfired.

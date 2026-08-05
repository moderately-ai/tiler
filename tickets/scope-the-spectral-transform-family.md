---
id: scope-the-spectral-transform-family
title: Scope the spectral transform family
status: deferred
priority: p3
dependencies: [scope-the-complex-arithmetic-vertical]
related: [derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, spectral, complex, deferred]
---
## User-visible outcome

A discrete Fourier or related transform is scoped once, with its normalization convention canonical and its accuracy bounded, instead of being admitted as whichever convention the first consumer's library happened to use.

## Why this is deferred rather than open

**Fact — the family is gated on an identity that is recognized and admits no operation.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-40 records that "a real transform's result is complex, which makes this the first family that *requires* the complex identity to be more than recognized", and closes: "Complex is registered as an identity with no operation admitting it, so this family is gated on the complex arithmetic that does not exist."

**Fact — the gate has an owner on the dtype axis.** [`scope-the-complex-arithmetic-vertical`](scope-the-complex-arithmetic-vertical.md) is deferred and owns `tiler::complex@1<ComponentTypeKey>`'s operation admission, its branch cuts, its exceptional values, and its planar-versus-interleaved storage. This ticket depends on it rather than restating it.

**Fact — two obligations here are unusual and must not be lost.** The normalization convention "differs across ecosystems and must be canonical", and the accuracy obligation is a *difference bound between two algorithms*: "an exact reference and a realized transform differ by an error the contract must bound", so the family is "deterministic given a fixed algorithm; **not** deterministic across algorithms", with the direct O(n²) transform as the reference route and FFT as "a faster and numerically different realization".

**Inference — that last point makes this family a test of an accepted rule rather than a new one.** [ADR 0076](../docs/decisions/0076-declare-target-honourable-numerical-realizations.md) forbids substituting a caller's stated contract to make a target feasible, and an FFT substituted for a direct transform is exactly such a substitution. Admitting the family without an accuracy contract that distinguishes them would make the substitution invisible.

## Activation trigger

The complex arithmetic vertical delivers an operation admitting `tiler::complex@1`, **and** a named workload requires a spectral transform. Both halves are required: a complex arithmetic vertical with no transform consumer admits nothing here, and a transform consumer with no complex arithmetic has no result type.

## What the work would be, when it starts

Per transform type: the real and complex variants as separate signatures, the transformed axes and transform length, the normalization convention fixed canonically with the ecosystem divergence recorded, the exact discrete transform at high precision as the oracle, and the accuracy contract that bounds an FFT realization's difference from it — stated as a bound rather than as an equivalence, because they are not equal.

## Explicit non-goals

- Complex arithmetic itself, which the dependency owns.
- Admitting an FFT as a realization of a direct transform without a bound distinguishing them.
- Real-only transforms as a way around the complex gate; a real transform's *result* is complex, which is the gate.

## Closes when

The family has signatures for its real and complex variants, a canonical normalization convention, an exact oracle, and an accuracy contract that bounds the realized transform against it — or is recorded as consumer-owned with the derivation.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-33** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-40 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired, on either half.** The complex vertical's trigger is unfired — no operation admits `tiler::complex@1` and its constructor refuses every component but f16, f32, and f64 — and no workload names a spectral transform. Recheck: read the `Trigger check log` or activation section of [`scope-the-complex-arithmetic-vertical`](scope-the-complex-arithmetic-vertical.md) and re-run the command it names.

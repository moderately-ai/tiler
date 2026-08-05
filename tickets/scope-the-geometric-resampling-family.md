---
id: scope-the-geometric-resampling-family
title: Scope the geometric resampling family
status: deferred
priority: p3
dependencies: [admit-an-indirect-gather-family-for-tied-embedding-lookup]
related: [scope-the-padding-and-cropping-family, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/indexing]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, resampling, deferred]
---
## User-visible outcome

A resize, a grid sample, or an affine warp is one family carrying all four of the attributes that decide its result, so that two callers asking for "bilinear" cannot get two different tensors.

## Why this is deferred rather than open

**Fact — the four attributes are the family.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-41 is atomic per interpolation mode, with "output extents or scale factors, interpolation mode, coordinate transformation mode, and a boundary mode — four attributes that are commonly conflated and produce different tensors", and it says so explicitly: "The four attributes are the reason this is one family rather than one per mode, and omitting any of them makes the family underspecified."

**Fact — the numerical difficulty is in the coordinates, not the interpolation.** "Interpolation arithmetic is ordinary float arithmetic; the coordinate computation's rounding is where implementations diverge."

**Fact — its physical route is a gather.** D7 records "physical fallback is a gather with computed coordinates", and [the minimum correct physical realization profile](../docs/research/program-planning/minimum-correct-physical-realization-profile.md) places F-41 in its *covered only under a stated precondition* class, the precondition being "the gather its coordinates are read through". That gather is live work: [`admit-an-indirect-gather-family-for-tied-embedding-lookup`](admit-an-indirect-gather-family-for-tied-embedding-lookup.md) owns the tensor-data-derived index class under Q-SHAPE-007.

**Inference — the dependency is real rather than topical.** Without an admitted indirect access class the family has no lowering at all, so scoping it first would produce a signature with no route.

## Activation trigger

A named image, signal, or vision workload requires resampling. The pinned language-model track does not reach it, and the roadmap's own candidate-track table records image and signal pipelines as "Not filed".

## What the work would be, when it starts

State all four attributes as canonical fields with their admissible values, and — the part that decides correctness — pin the coordinate computation's rounding rather than leaving it to the realization, since that is where implementations diverge. Then the exact-coordinate-then-interpolate oracle, and the gather-with-computed-coordinates lowering expressed over whatever access class the gather work admits, rather than a second indirection.

## Explicit non-goals

- The indirect access class, which the gather ticket owns.
- One family per interpolation mode, which the taxonomy's own reasoning rejects.
- A boundary mode implemented as a padding family. Reflect, edge, and wrap need the piecewise access class, and borrowing a pad here would hide that.

## Closes when

The family has all four attributes canonical, a pinned coordinate rounding, an exact oracle, and a lowering expressed over the admitted indirect access class — or is recorded as consumer-owned with the derivation.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-34** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-41 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No image, signal, or vision workload is filed; the roadmap's candidate-track table records that class as "Not filed", and the only live conformance track is language-model inference. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.

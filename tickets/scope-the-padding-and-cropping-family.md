---
id: scope-the-padding-and-cropping-family
title: Scope the padding and cropping family
status: deferred
priority: p3
dependencies: []
related: [scope-the-windowed-reduction-and-convolution-family, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, structural, numerics, deferred]
---
## User-visible outcome

A padded or cropped tensor is one governed operation whose pad value is a stated numerical participant, so that a pass eliding the materialization owes a neutrality proof instead of assuming zero is neutral.

## Why this is deferred rather than open

**Fact.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s F-25 is atomic, with interior, edge, and negative padding as attribute values of one family, "low, high, and interior pad amounts per axis, signed so that a negative amount crops", and one D5 sentence that is the whole reason this is its own track: "The pad value participates in downstream numerics and is **not** neutral by virtue of being an identity element."

**Fact — the counterexample is already in the corpus and is exact.** [Numerical semantics](../docs/numerical-semantics.md) keeps empty result, algebraic identity, and safe physical padding as three separate facts: a strict floating sum may return `+0.0` for an empty domain, yet adding `+0.0` to a singleton `-0.0` under round-to-nearest produces `+0.0`, so `+0.0` is not bitwise-neutral padding for that reduction even though it is its empty result. The same obligation is what a tiled contraction owes for a ragged contracted extent, and [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s contraction row states it there.

**Inference — this is why padding is not grouped with the other structural families.** F-22, F-23, F-24, F-26, and F-27 have no numerical content at all; padding introduces a value into the result that downstream arithmetic reads. Grouping it with them would give one track two numerical contracts, which is exactly the split this record's partition rule requires.

**Fact — reflect, edge, and wrap modes are out, and the reason is an access class rather than a preference.** The taxonomy records them as "separate coordinate maps and are unsupported until the piecewise map class exists" — [Q-SHAPE-006](../docs/open-questions.md#q-shape-006--finite-piecewise-access-maps), whose trigger is unfired.

## Activation trigger

A named workload needs explicit padding or cropping — most plausibly a convolution or pooling occurrence, whose family is [`scope-the-windowed-reduction-and-convolution-family`](scope-the-windowed-reduction-and-convolution-family.md)'s and which carries the same pad-value obligation inside its own window attributes. A *physical* tile pad does not fire it: that is a schedule choice owing a neutrality proof, not a semantic operation.

## What the work would be, when it starts

The per-axis signed low/high/interior attributes with the cropping sign convention stated; the constant-pad form's rank-zero pad value operand of the same type; the refusal of reflect, edge, and wrap until the piecewise class exists; the materializing oracle; the guarded-read lowering; and — the part that is not bookkeeping — the neutrality obligation an eliding pass must discharge, written as a proof requirement under the selected numerical contract rather than as a note.

## Explicit non-goals

- The physical tile pad a tiled schedule performs, which is a schedule obligation under the same neutrality rule and not this family.
- Reflect, edge, and wrap modes, which need the piecewise access class.
- The windowed family's own padding attributes, which belong to that family's signature even though they carry this obligation.

## Closes when

The family has a signed per-axis attribute schema, a materializing oracle, a guarded-read lowering, and a written neutrality obligation for elision — with the obligation exercised by at least one case where a plausible pad value is not neutral for the consuming reduction.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-24** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which covers F-25 alone and states why they are one track rather than several.
- [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity. This ticket moves no rung, and a scoping record delivers nothing.

## Trigger check log

- 2026-08-05 — **not fired.** No named workload needs explicit padding or cropping; the pinned workload's only padding-shaped concern is the additive causal mask, which is a bound `f32` program input rather than a pad. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys today, comprising the dtype identities, the ULP metric key, and the eighteen registered operation keys; the family's key is absent from that list.

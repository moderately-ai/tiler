---
id: attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget
title: Attribute the canonical-manifest growth and decide whether the encoding owes a budget
status: todo
priority: p2
dependencies: []
related: [re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps, re-price-the-envelope-band-consumers-against-the-re-derived-band]
scopes: [research/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [research, artifacts, measurement, encoding]
---
## User-visible outcome

The 4× growth of the artifact envelope's fixed content between 2026-08-04 and 2026-08-06 is attributed to the changes that caused it, and the repository states whether the canonical encoding owes itself a size budget — or records why unbounded manifest growth is acceptable while a 1 MiB per-invocation embedding ceiling stands.

## The measurement this starts from, taken rather than owed

**Measurement** ([the hot-path note §9.1](../docs/research/cache/hot-path-efficiency.md), 2026-08-06). One unchanging fixture's zero-object envelope was built at two commits and both framings parsed: fixed content is 28,527 bytes at `194744e6` (2026-08-04) and 114,043 at `8bd720b8` — +65,363 bytes of canonical manifest, +20,153 of `KernelProgramSubject` section, `BackendPayloadMetadata` byte-identical as the control. On real producer output the effect is a 4.4× envelope band move (32,136–47,803 → 141,532–159,037) with every `metallib` byte-identical; on the largest member the canonical manifest is 76.3% of the envelope and the carried compiled object under a twentieth. The `MANIFEST_SCHEMA` steps in the interval (12.0 → 14.0) change no lengths, so the growth is in what the manifest describes, not the framing.

**Why it matters now, not later.** The macro embedding ceiling headroom fell from "more than an order of magnitude" to 15.17% consumed ([the embedding note](../docs/research/embedding/self-contained-embedding.md) §5): roughly two-thirds of one more threefold growth exhausts the 1 MiB per-invocation gate. The cache-hit cost is now ~90% envelope validation, and [the hot-path note §9.7](../docs/research/cache/hot-path-efficiency.md) closes with "if the hit path is ever worth attacking again, the lever is the encoding, not this crate". Three consumers price against envelope size (macro embedding, expansion-cache steady state, cache-hit latency), and none of them owns the number that drives all three.

## What this must produce

1. **The attribution, as a bounded experiment.** Rebuild the hot-path fixture (or an equivalent unchanging fixture) at the intermediate commits between `194744e6` and `8bd720b8` and attribute the +85,516 fixed bytes to the changes that added them, separating what each buys (identity coverage, delivered-realization evidence, staged-coverage encoding, …) from what it costs. The re-derivation deliberately did not do this; its fixture-rebuild method is recorded in [its ticket](re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps.md) and reuses directly.
2. **The decision surface, drafted not decided.** Whether the encoding owes a budget (a tracked fixed-overhead number with a check that fails on unexplained growth, per the make-new-checks-fail discipline), owes compression or elision for derivable content, or deliberately owes nothing while the ceiling stands — compare on correctness, maintainability, and the three consumers' costs, give the strongest counterpoint, recommend one. Anything touching the canonical encoding is an identity-domain change and a public boundary: draft and park for Tom.

## Non-goals

Changing the encoding; re-deciding the 1 MiB ceiling or the 30-day cache window (owned elsewhere); re-running the consumer re-pricing ([`re-price-the-envelope-band-consumers-against-the-re-derived-band`](re-price-the-envelope-band-consumers-against-the-re-derived-band.md) owns it).

## Closes when

The growth is attributed with per-change sizes on the unchanging fixture, each attributed change names what the bytes buy, and the budget question is answered or parked for Tom with the evidence and a recommendation.

---
id: spike-first-metal-contraction-vertical
title: Spike the first workload-derived Metal contraction vertical
status: todo
priority: p1
dependencies: [derive-transformer-operation-and-shape-surface, prototype-metal-runtime-proof]
related: [scope-einsum-contraction-support, implement-opaque-physical-call-providers, implement-parallel-reduction-strategies, implement-analytical-component-cost-model]
scopes: [research/scheduling, research/apple-targets, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [spike, research, contraction, matmul, metal, language-model]
---
Use the selected workload to bound and measure the first tensor-contraction
profile rather than attempting general einsum support.

## Questions the spike must answer

- Which fixed matmul or batched-matmul shapes and dtypes constitute the first
  useful profile?
- Which semantic identity, structural validation, access relation, reduction
  order, accumulation dtype, and exceptional-value rules does it require?
- Which direct, tiled, simdgroup, or opaque/library realization candidates
  survive correctness and Metal feasibility checks?
- What padding, layout, synchronization, resource, and numerical obligations
  eliminate a candidate?
- What can be measured on the selected Apple target, and what remains unknown?

Preserve a reproducible harness, exact environment, raw or summarized results,
unsupported cases, and stop conditions under `spikes/`. A candidate with
unknown numerical behavior is not a viable implementation merely because it is
fast.

## Ticket-producing outcome

File separate dependency-ordered delivery tickets for the surviving semantic
profile, normative reference, direct Metal realization, optimized schedule
portfolio, qualified opaque alternative if one survives, runtime integration,
and conformance evidence. Do not file work for eliminated candidates.

## Closes when

At least one bounded contraction path is shown feasible or every tested path is
rejected with reproducible reasons; the architecture and measurement boundary
are recorded; and the surviving work is represented by scoped vertical
tickets with explicit user-visible outcomes.

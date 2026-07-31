---
id: spike-first-metal-contraction-vertical
title: Spike the first workload-derived Metal contraction vertical
status: in-progress
priority: p1
dependencies: [derive-transformer-operation-and-shape-surface, prototype-metal-runtime-proof]
related: [scope-einsum-contraction-support, implement-opaque-physical-call-providers, implement-parallel-reduction-strategies, implement-analytical-component-cost-model]
scopes: [research/scheduling, research/apple-targets, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [spike, research, contraction, matmul, metal, language-model]
claimed_from: todo
assignee: loop-l3-spike
lease_expires_at: 1785526607
---
## User-visible outcome

The first contraction profile is *bounded and measured* — which matmul/batched-matmul shapes and dtypes, under which realization (direct, tiled, simdgroup, or library call), at what measured cost on the bench host — instead of an attempt at general einsum. The measurements are the evidence `scope-first-quantized-lm-profile` and the cost calibration consume.

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

## Activation trigger — added 2026-07-27 by `scope-optimized-metal-lm-inference`

**Rung L3** of the language-model inference ladder in [`docs/roadmap.md`](../docs/roadmap.md).

**Active when:** L2 lists the contraction shapes **and** milestone 6 settles whether a contraction is one keyed family or a set of per-shape keys.

**Rests on:** L2, plus the milestone 6 open question.

Do not start this before its trigger fires. Each rung's scope is derived from the rung below it, so beginning early means deriving a surface from an assumption rather than from delivered evidence — which is how a discovery ticket turns into a rewrite.

## Graph maintenance

- **This is a spike**: it lives under `spikes/`, runs from its own directory with the invocation its README records, and no `make` target reaches it. Keep the harness, inputs, and result fixtures checked in; `.gitignore` only regenerable outputs.
- **An opaque/library realization candidate is an opaque physical call** — if the spike shows the library route wins, the admission machinery already exists (declaration, registration, frontier admission) and the gap is caller-supplied providers plus lowering; record that on `exercise-opaque-admissions-downstream-of-the-frontier` and the enforcers ticket rather than inventing a separate integration path.
- **Measurements happen on the M3 bench host, serially** — never in parallel agents; interleave A/B; record exact environment per row.
- **On close, update the roadmap ladder rung** and hand the shape/dtype profile to `derive-transformer-operation-and-shape-surface` if it is still open.

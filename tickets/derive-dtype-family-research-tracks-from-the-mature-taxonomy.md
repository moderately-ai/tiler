---
id: derive-dtype-family-research-tracks-from-the-mature-taxonomy
title: Derive dtype-family research tracks from the mature taxonomy
status: in-progress
priority: p1
dependencies: [enumerate-the-mature-tensor-dtype-taxonomy, own-the-dtype-support-maturity-matrix]
related: [own-operation-family-support-matrix, declare-the-bf16-rows-on-the-authoritative-metal-profile]
scopes: [research/numerics, contracts/navigation, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, dtypes, roadmap, ticket-graph]
claimed_from: todo
assignee: agent-dtype-tracks
lease_expires_at: 1785880009
---
## User-visible outcome

Every dtype family in the mature taxonomy has a bounded research owner and a route
to explicit support or explicit deferral; no family disappears merely because no
current workload produces it.

Read the taxonomy and support ledger row by row. Partition work by genuinely shared
representation and numerical obligations: booleans; signed and unsigned integers;
IEEE and reduced-precision floats; FP8/FP6/FP4 families; complex; quantized compound
values and scales/zero-points; and any opaque/extension carriers. For each partition
record semantic identity, host/reference carrier, conversion behavior, exceptional
values, constant encoding, artifact ABI, scalar/KIR support, backend capability,
and conformance requirements.

Create research/design/spike tickets for every missing family. Reuse existing BF16,
quantization, and numerical-policy nodes where they are exact owners. Implementation
remains signature-driven: a family track does not claim every operation supports the
dtype. Deferred tracks require measurable activation triggers.

## Closes when

Every dtype taxonomy row maps to an existing exact owner or a newly filed bounded
track; dependency order preserves numerical/reference authority before optimizer and
backend claims; and the support ledger links to those owners without overstating
implemented maturity.

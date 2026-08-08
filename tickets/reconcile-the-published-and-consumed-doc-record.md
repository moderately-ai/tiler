---
id: reconcile-the-published-and-consumed-doc-record
title: Reconcile the published-and-consumed doc record
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

One in-file account of the published-and-consumed capability, with the dead test name retired everywhere it propagated.

## Why this exists (drift audit 2026-08-06, coordinator-verified: the dead name survives at request.rs:3857, six ticket bodies, and the flash record)

`select_supported_strategy`'s doc (request.rs:3831-3858) says the shape is refused under `output-partition-overlap`; `check_output_cover` (4001-4020, 4058) admits it and the in-file test asserts admission. The refusing doc cites `a_published_and_consumed_intermediate_refuses_by_name`, which exists in no crate — the real test is `pipeline/conformance.rs`'s `a_published_and_consumed_intermediate_compiles_and_agrees`. Graph maintenance, not prose: correct the stale doc, then sweep the six ticket bodies and `flash-class-capability-set.md:69` with dated notes (done tickets get superseded-paragraph appends, never rewrites).

## Closes when

The two in-file docs agree, the dead name is gone from code, and every propagated citation carries its dated correction.

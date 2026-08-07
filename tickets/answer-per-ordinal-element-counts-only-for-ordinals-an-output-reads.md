---
id: answer-per-ordinal-element-counts-only-for-ordinals-an-output-reads
title: Answer per-ordinal element counts only for ordinals an output reads
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-ordinal
lease_expires_at: 1786121747
---
## User-visible outcome

`resolve_work_items` stops spuriously refusing an opaque call's `PerElementOf` scaling in a multi-output program whose outputs iterate different domains — a false negative today, because a non-reading output volunteers an element count for an ordinal it never loads and breaks the agreement fold.

## Why this exists (audited 2026-08-06, coordinator-verified: input_keys = program.inputs() at request.rs:5154; agreed fold at :1768; sole caller frontier.rs:2676)

`NormalizedOutput::input_elements_at` (`crates/tiler-compiler/src/request.rs:1514`) answers for every ordinal below the program's declared arity in its `SerialSum` and `Pointwise` arms, because `input_keys` is the whole program's declared list. Its two siblings on the same type — `max_input_elements` and `reads_declared_input` — were corrected to read the recognized read lists when subset reads landed; this one was not, and its comment ("Every declared input of a reduced program is read at the contributor domain") is the stale premise. The `Epilogue` arm guards its own half then recurses into the unguarded producer arm.

## The work

Both arms gate on `self.reads_declared_input(ordinal)` — the authority already on the type — before answering; the comment restates per-read truth. Failure perturbation: two outputs over disjoint inputs at different extents with an opaque call bound to ordinal 0 and `PerElementOf` — before: `UnknownParameter`; after: the reading output's count — and the genuine one-input-two-domains disagreement case still refusing, or the fix removed the check.

## Closes when

Both arms answer only for read ordinals, both perturbations are observed, and the stale comment is corrected.

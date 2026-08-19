---
id: decide-how-a-dynamic-bounds-witness-enters-the-schedule-vocabulary
title: Decide how a dynamic bounds witness enters the schedule vocabulary
status: blocked
priority: p2
dependencies: [package-the-admitted-live-schedule-into-a-symbolic-kernel-program]
related: [replace-zero-live-bounds-sentinels-with-abi-derived-accessible-ranges, carry-live-extent-operands-through-the-artifact-envelope]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, identity, public-boundary, schedule]
---
## User-visible outcome

A live-extent access carries a bounds proof that states its reach, instead of the zero-length `LinearRange` the verifier emits today — under an accepted public spelling and an accepted identity consequence, rather than a worker's improvisation.

## Why this exists — filed 2026-08-19 from the p0 live-bounds audit

[`replace-zero-live-bounds-sentinels-with-abi-derived-accessible-ranges`](replace-zero-live-bounds-sentinels-with-abi-derived-accessible-ranges.md) (p0) cannot proceed without this decision. Its worker's audit at `441f3215`, merged with that ticket, establishes the two authority steps the p0 does not carry:

**Fact — a dynamic range witness is a public-vocabulary change whose accepted record states the opposite intent.** It requires a new `BoundsProofKind` variant in `tiler_ir::schedule`. That vocabulary is accepted public surface, and the accepted `LiveRowMajorSource` record says the live extent is `crates/tiler-ir/src/schedule/model.rs "consumed in the payload"` rather than specialized into the schedule. Admitting a schedule-level witness reverses a stated design intent and is therefore Tom's, not a carrier's.

**Fact — the tag moves identity.** `BoundsProofKind` is written into the canonical scheduled-region identity encoding beside `TAG_LINEAR_RANGE`. A new tag moves every live region's schedule identity and cascades through kernel, kernel-program, and artifact identity. The neighbouring tag comments show these assignments are reconciled *across* accepted decision packets, not assigned by a worker.

**Fact — the p0 ticket's stated direction inverts the layer order.** It says the witness is "derived from the artifact's existing `AbiRoot::InputExtent` authority", but the schedule sits below the program and artifact and cannot read them. The workable direction is the reverse, which is a materially different design from the one the p0 specifies.

**Fact — the static agreement rule blocks the obvious spelling.** `KernelProgramBuilder::push_stage` requires `evaluate_static_abi(accessible_bytes)` to equal the view's window length, and `static_facts()` binds only declared *static* extents — so on a symbolic subject that check cannot evaluate at all. Publishing a live reach needs a symbolic element count, a symbolic `ByteWindow` length, and a symbolic ABI agreement rule together, not one at a time.

## Why this is blocked rather than ready

The readiness gate's first step: a local API shape is not decision-ready while its consumer or prerequisite is unresolved. No live-extent artifact is constructible or decodable at this base, so the packet cannot state what the witness must serve, and a frontier derived now would be about a population that does not yet exist. [`package-the-admitted-live-schedule-into-a-symbolic-kernel-program`](package-the-admitted-live-schedule-into-a-symbolic-kernel-program.md) — accepted by Tom on 2026-08-19 as the complete-subject fold, and the ticket that makes a symbolic program packageable — is the release trigger.

## Required work when released

Author a Pareto-complete decision packet under the AGENTS.md readiness gate: enumerate every materially distinct spelling (a new `BoundsProofKind` variant; a payload-side witness that leaves the schedule vocabulary alone, consistent with the `LiveRowMajorSource` record's stated intent; a symbolic-extent widening of the existing `LinearRange`; the status quo with the population left refused); eliminate anything that can silently return a wrong reach, default a bound, or let an adapter reconstruct a second meaning of the live extent; compare survivors on correctness, fail-closed strictness, maintainability, and the exact identity cascade each implies with its tag reconciliation; state the strongest counterargument and reversal evidence for each; and present the nondominated frontier to Tom as one concrete question. Independent review before queueing.

## Closes when

Tom accepts one exact spelling with its identity consequence, or records that the population stays refused — and the p0 carrier's dependency on this ticket resolves either way.

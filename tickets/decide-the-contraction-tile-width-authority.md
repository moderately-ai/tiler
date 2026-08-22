---
id: decide-the-contraction-tile-width-authority
title: Decide the contraction tile-width authority
status: in-progress
priority: p1
dependencies: []
related: [realize-the-tiled-contraction-schedule-and-its-metal-emission, offer-the-tiled-contraction-alternative-in-physical-planning]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, target-profiles, scheduling]
claimed_from: todo
assignee: worker-tilewidth
lease_expires_at: 1787428751
---
## User-visible outcome

Tom decides where a tiled contraction's tile width comes from, so the physical planner can offer the tiled alternative without either hard-coding a measured constant or inventing a target-profile row on a worker's authority.

## Why this exists

Filed 2026-08-22 by the coordinator, from the tiled-contraction lane's enumerated remainder. That lane landed the schedule, the lowering, and the Metal emission, and **stopped deliberately** before the compiler alternative because its first prerequisite is a decision rather than code.

**Fact — the repository already refused this exact shortcut once, and said why.** `crates/tiler-compiler/src/physical.rs` declines to offer the workgroup tree unless the target profile declares a closed width policy. Anchor: `Silence is` — which resolves exactly once in that file, at a `///` comment reading *"The tree is offered only under an explicit closed policy. Silence is / not a default, not a clamp onto the internal `256`, and not a / substitution of [`governed_partition`]."* Verified by the coordinator at `b3c07259`. **Note the anchor is deliberately short**: the rendered sentence "Silence is not a default" spans a line break in the source and greps to zero, which is the failure mode AGENTS.md records.

**Fact — the measured width is 16 and it is one host's measurement.** The tiled kernel the first-Metal-contraction record measures uses a 16-wide tile. A measured value is not a portable authority.

**Inference — there are two honest shapes and they are not equivalent.** Either the target profile grows a contraction-tile-width policy row — a target-profile public boundary, and so Tom's — or a named measured constant is accepted with the standing `MEASURED_TREE_PARTICIPANT_CAP` already has. The first makes the width a declared target property that a profile can refuse to state; the second makes it a repository constant that every profile inherits.

## Required work

- Re-audit both Facts and the Inference at your own base and report a per-Fact verdict before writing any packet prose.
- Apply AGENTS.md's decision-packet readiness gate in full. In particular, enumerate the status quo (no tiled alternative is offered at all, which is the current state and is honest) alongside both shapes above.
- For each survivor, state what it enables and prevents, its identity and public-surface consequence, its strongest counterargument, and the evidence that would reverse it.
- Do not present until the gate is satisfied. If one option dominates, recommend it rather than manufacturing a choice.

## Non-goals

Implementing the alternative — that is [`offer-the-tiled-contraction-alternative-in-physical-planning`](offer-the-tiled-contraction-alternative-in-physical-planning.md); changing the landed schedule, lowering, or emission; and declaring a width on any profile before the authority is decided.

## Closes when

Tom has accepted one authority for the tile width with provenance recorded, or has redirected the question, and the implementation ticket can name the accepted source without inventing one.

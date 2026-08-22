---
id: reconsider-the-square-tiles-only-lowering-restriction
title: Reconsider the square-tiles-only lowering restriction
status: todo
priority: p2
dependencies: []
related: [realize-the-tiled-contraction-schedule-and-its-metal-emission, calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [scheduling, contraction, lowering, performance]
---
## User-visible outcome

The contraction lowering admits the tile shapes the schedule layer already models, so a workload whose output is one row wide does not pay for fifteen masked rows it never uses.

## Why this exists

Filed 2026-08-22 by the coordinator from the tiled-contraction lane's remainder, now with a measured motivation rather than a stylistic one.

**Fact — the restriction is at the lowering layer only.** The tile is one `u64` because of a `Square tiles only` restriction in `crates/tiler-ir/src/kernel/lower.rs`; `crates/tiler-ir/src/schedule/blocked.rs` admits general shapes. So the schedule vocabulary is already wider than what lowering accepts. Reported by the tile-width lane at `f2c974a8`; **re-derive both halves at your base** — the coordinator has verified neither anchor.

**Fact (reported, unverified by the coordinator) — the measured regression is attributed to exactly this.** The first-Metal-contraction record attributes a 2.28x regression to a square block wasting fifteen of sixteen rows at M = 1, calling it *"a schedule mismatch, not a bandwidth result"*. Read the record before relying on that.

## Required work

- Re-audit both Facts at your base and report a per-Fact verdict; this ticket rests entirely on another lane's reading and the coordinator has confirmed none of it.
- Determine whether lifting the restriction is a lowering change alone or reaches the schedule identity. **If any identity moves, stop and report** — that is a separate decision.
- If lifted, the admissible shape set must be stated and refused outside, not inferred.
- Perturb the subject with quoted failure text, and add a fixture at a non-square shape, since every existing fixture is square by construction.

## Non-goals

Choosing a tile width; offering the alternative in planning; and any performance claim not backed by a measurement on its claimed hardware.

## Closes when

Either the restriction is lifted with its admissible set stated and refused outside, or it is recorded as deliberate with the reason and a reconsideration trigger — and in both cases the measured attribution above is confirmed or refuted rather than repeated.

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

**Fact — the measured regression is attributed to exactly this, and the attribution is an Inference wearing a Measurement label.** `docs/research/scheduling/first-metal-contraction-realizations.md` says the `16×16` output tile *"computes one useful row and fifteen masked ones when `M = 1` — a schedule mismatch, not a bandwidth result"*, at `t_vocab_full` where `tiled` is 9,669 µs against a 4,247 µs best. **That sentence sits under a `**Measurement —**` heading, but no width was swept and no masked-thread count was instrumented.** So the attribution is a cause the record inferred, not one it measured, and this ticket must treat it as a hypothesis.

*(Corrected 2026-08-22. This Fact previously read "a 2.28x regression … a square block wasting fifteen of sixteen rows". Neither figure exists in any record: **`2.28` is a ratio I derived without saying so** — 2.2764 against `ksplit_contiguous`, 2.0327 against `direct`, 2.1883 against MPS — and **"fifteen of sixteen" is my rewording** of "one useful row and fifteen masked ones". Verified absent by the coordinator at `97e7fef1`. Restating a source claim in new words is what AGENTS.md forbids, and both restatements greped to zero.)*

**Consequence for this ticket's ordering.** [`calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol`](calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol.md) has frozen a two-arm sweep whose **rectangular arm exists precisely to refute or confirm this attribution**, along with a registered null control at `M = 128`/`512` where the mechanism predicts no width effect. **Wait for it.** Lifting the restriction on an unmeasured cause is how a performance claim becomes folklore.

## Required work

- Re-audit both Facts at your base and report a per-Fact verdict; this ticket rests entirely on another lane's reading and the coordinator has confirmed none of it.
- Determine whether lifting the restriction is a lowering change alone or reaches the schedule identity. **If any identity moves, stop and report** — that is a separate decision.
- If lifted, the admissible shape set must be stated and refused outside, not inferred.
- Perturb the subject with quoted failure text, and add a fixture at a non-square shape, since every existing fixture is square by construction.

## Non-goals

Choosing a tile width; offering the alternative in planning; and any performance claim not backed by a measurement on its claimed hardware.

## Closes when

Either the restriction is lifted with its admissible set stated and refused outside, or it is recorded as deliberate with the reason and a reconsideration trigger — and in both cases the measured attribution above is confirmed or refuted rather than repeated.

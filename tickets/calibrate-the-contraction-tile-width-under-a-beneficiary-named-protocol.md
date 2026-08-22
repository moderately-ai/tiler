---
id: calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol
title: Calibrate the contraction tile width under a beneficiary-named protocol
status: in-progress
priority: p2
dependencies: []
related: [decide-the-contraction-tile-width-authority, carry-the-contraction-tile-width-policy-as-a-target-profile-row]
scopes: [research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [measurement, scheduling, contraction, target-profiles]
claimed_from: todo
assignee: worker-tileprotocol
lease_expires_at: 1787430110
---
## User-visible outcome

The contraction tile width has a swept, protocol-registered measurement naming its beneficiary profile key before the run — so a width can be *chosen* on evidence rather than inherited from the single value that happened to be compiled.

## Why this exists

Filed 2026-08-22 by the coordinator from the tile-width authority packet, which found the existing evidence cannot support any declared width and cannot be repaired.

**Fact — the 16 was never swept.** `spikes/scheduling/metal_contraction_vertical/kernels.metal` declares `constant uint TILE = 16;` — one compile-time constant, with every other `TILE` reference derived from it. The spike swept *realization families*, not widths. **No width other than 16 has ever been executed.** Verified by the coordinator at `f2c974a8`; `grep -n "TILE" ` on that file returns the constant plus five derived uses and one precondition comment.

**Fact — the retained record measures the tiled kernel losing.** The packet reports `tiled` at 523.5 µs against `direct`'s 251.4 and `ksplit_contiguous`'s 234.0 at one workload, and a 2.28x regression at another, with the record attributing the cause to a square block wasting fifteen of sixteen rows at M = 1 — *"a schedule mismatch, not a bandwidth result"*. Re-read the record before relying on these figures; they are the packet's, re-derived once.

**Fact — ADR 0113 bars the existing record from ever populating a profile row.** Component 3(a) admits a measured row into a family-keyed profile only when the producing measurement's frozen protocol named that exact profile key as beneficiary *before* the run. The contraction spike named four realization families and no profile key, and the ADR is explicit that this is unrepairable. **A new measurement is the only route.**

## Required work

- Re-audit all three Facts at your base and report a per-Fact verdict.
- Write the frozen protocol **first**, naming the exact beneficiary profile key, and **commit it before the harness runs** so pre-registration is provable from history rather than asserted. The thread-execution-width lane did exactly this; follow it.
- Sweep widths, not one width. State the admissible set and why, and include the shapes where the square block is known to waste rows — the regression above is a hypothesis this sweep should confirm or refute, not assume.
- **Run on the idle M3 Pro, not a coordination host during an agent wave.** Record workload, target, metric, warm-up, repetitions, variance, exact toolchain, source commit, and load controls. Before recording any toolchain fact, state which invocation produced it — a bare `xcrun … metal --version` answers for `xcode-select`, not for what the repository compiles with.
- State the measurement boundary explicitly: which contracted extents and shapes the claim covers, and which it does not.

## Non-goals

Declaring any profile row — that is [`carry-the-contraction-tile-width-policy-as-a-target-profile-row`](carry-the-contraction-tile-width-policy-as-a-target-profile-row.md); changing the landed schedule, lowering, or emission; and repairing the existing record, which ADR 0113 forbids.

## Closes when

A protocol naming its beneficiary key is committed before its harness runs, a width sweep is measured on the idle M3 Pro with its boundary stated, and the result either supports a width choice or records that none is supportable.

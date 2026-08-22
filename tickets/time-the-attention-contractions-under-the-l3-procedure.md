---
id: time-the-attention-contractions-under-the-l3-procedure
title: Time the attention contractions under the L3 procedure
status: blocked
priority: p1
dependencies: []
related: [realize-the-attention-contractions-on-metal, plan-the-recomputing-attention-decomposition, calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol]
scopes: [research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [measurement, scheduling, contraction, attention]
---
## User-visible outcome

The two attention contractions have timings at the C1 prefill row and at least two B1 rows, taken under the L3 record's own procedure, so the D-A-versus-D-B comparison in [`plan-the-recomputing-attention-decomposition`](plan-the-recomputing-attention-decomposition.md) has a baseline that exists.

## Why this is separate from the realization lane

Filed 2026-08-22 by `worker-attention`, splitting the unmeasurable half off [`realize-the-attention-contractions-on-metal`](realize-the-attention-contractions-on-metal.md) so that lane could close its correctness half.

**Fact — no cell of either structure has been timed at any shape.** The [L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md) scopes its realizations, attribution corpus, and timing rows to structure 1 and names the attention lane as owning the measurements that close it.

**Fact — the bench host was above its own load gate when this was filed.** The `m3` host reported load averages `2.44 2.39 2.33`, against the 0.5 gate the contraction tile-width protocol freezes. Measured directly by the filing lane, not inherited.

**Inference — a measurement taken only at C1 would rank these kernels on the row where they barely matter.** The block's four projections perform `T * 6,291,456` multiply-accumulates and its two attention contractions `4,096 * T^2` at `S = T`; they are equal at `T = 1,536`. At C1 the projections dominate 154x and at B1-d the attention contractions dominate 5.3x. So the B1 rows are the point of this ticket, not an extension of it.

## Required work

- Re-audit both Facts at your base, including re-probing the bench host's load rather than trusting the figure above.
- **Run on the idle M3 Pro, not a coordination host during an agent wave.** Reuse the L3 procedure exactly: settled minimum over interleaved A/B rounds, round 0 reported separately, spread stated.
- Before recording any toolchain fact, state which invocation produced it. A bare `xcrun --sdk macosx metal --version` answers for `xcode-select`, not for what the repository compiles with.
- State the measurement boundary: which structures, rows, and extents the claim covers, and which it does not.
- Note that the bench host's macOS build has moved from the retained record's `26A5378n`, so retained microsecond figures are not a baseline; write predictions as ratios internal to the new sweep.

## Non-goals

Changing any schedule, lowering, or emission; choosing a realization; and declaring any target-profile row.

## Activation triggers

Bench-host load below the gate, and a dispatch route for the attention structures. Note that no contraction equivalent of `crates/tiler-conformance/src/dispatch.rs`'s device harness exists yet — the projection structure reaches a device only through `prototypes/serial-sum-compile` and `prototypes/serial-sum-run`.

## Trigger check log

- 2026-08-22 — **not fired.** Bench host `m3` at load `2.44 2.39 2.33`, above the 0.5 gate. Reproduce: `ssh m3 uptime`.

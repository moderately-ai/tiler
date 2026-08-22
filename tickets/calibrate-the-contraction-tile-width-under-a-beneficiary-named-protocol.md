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

## Fact audit at base `1bcf6d59` — worker `worker-tileprotocol`, 2026-08-22

Every Fact above was re-read at this base against the source it names. Verdicts follow; the Facts are left standing and these notes govern where they disagree.

**Fact 1 — "the 16 was never swept": verified in substance, false in its count, and materially incomplete.**

Verified: `spikes/scheduling/metal_contraction_vertical/kernels.metal "constant uint TILE = 16;"` is present, every reference in that file derives from it, and no width other than 16 has ever been executed.

False: the stated enumeration of *the constant plus five derived uses and one precondition comment* does not match the file. `grep -c "TILE"` returns 17 lines, `grep -o "TILE" | wc -l` counts 23 occurrences, and the coordinator's own `grep -n "TILE "` form returns 12 lines. This is the `grep -c` counts lines, not occurrences hazard `AGENTS.md` records, reaching a ticket Fact.

Materially incomplete, and this is the part that would have misled a worker: **the width 16 is bound at four independent sites across two files, not one.** Besides the kernel constant, `spikes/scheduling/metal_contraction_vertical/host.m` binds it three more times, none deriving from it — the precondition `tiled-requires-k-multiple-of-16` and its `k_extent % 16u` test, the grid divisor `(n_extent + 15) / 16`, and the threadgroup size `MTLSizeMake(16, 16, 1)`. A worker who believed the width lived in one constant would have changed it and dispatched the old shape.

**Fact 2 — "the retained record measures the tiled kernel losing": figures verified exactly, ratio unsourced, attribution correctly quoted but mislabelled at its source.**

Verified exactly: at `w_vocab_slice` (M=1, N=8192, K=1024) the retained `timing-summary.tsv` gives `tiled` 523.500 µs, `direct` 251.417 µs, `ksplit_contiguous` 234.000 µs, reproduced in the research record's table.

Imprecise: the string `2.28` appears **nowhere** in the record — `grep -rn "2\.28" docs/research/scheduling/ spikes/scheduling/` returns nothing. It is a ratio derived from `t_vocab_full`, and which ratio depends on the rival: `tiled`/`ksplit_contiguous` is 2.2764, `tiled`/`direct` at the same cell is 2.0327, and `tiled`/`opaque_mps` is 2.1883. A Fact citing "2.28x" without naming the denominator is not re-derivable.

Attribution: the quoted fragment *a schedule mismatch, not a bandwidth result* is verbatim, but it lives in `docs/research/scheduling/first-metal-contraction-realizations.md`, not in the spike record's own README, and the phrase "fifteen of sixteen" appears in no record — the source reads that its 16x16 output tile computes one useful row and fifteen masked ones at M = 1, which the ticket paraphrases faithfully.

**New, and it is why this ticket's framing is right:** that attribution sits inside a paragraph opened by a **Measurement —** label, yet no width was ever swept and no masked-thread count was ever instrumented. It is an **Inference** presented with measurement authority. Correcting the label belongs to that record's owner; this lane declines to inherit it as a premise and pre-registers it as a refutable hypothesis instead.

**Fact 3 — "ADR 0113 bars the existing record from ever populating a profile row": verified.**

Component 3(a) reads as the ticket states, and the record adds *A record whose protocol scoped it elsewhere composes into nothing else, ever.* The spike's own README confirms the disqualifying scope: *Four realization families were named ahead of the measurement*, and no profile key. A new measurement is the only route.

**Addition the ticket should carry:** 3(a) is a **necessary condition, not a sufficient one**. Naming the beneficiary does not make the eventual row's admission automatic, because a contraction tile-width policy is a fact family the flagship profile does not currently state, and ADR 0113 keeps a move of the profile's stated content, descriptor, and pins a Tom-facing packet. The frozen protocol states this before the run so the carrier ticket cannot inherit it as settled.

**Coordinator brief claim, checked:** the toolchain invocation caveat is exact. On the coordination host `xcrun --sdk macosx metal --version` answers `32023.921` because `xcode-select -p` is `/Applications/Xcode-beta.app/Contents/Developer`, and `DEVELOPER_DIR=/Applications/Xcode.app xcrun --sdk macosx metal --version` answers `32023.883`. On the bench host `m3` both forms answer `32023.883`, because its `xcode-select -p` is already `/Applications/Xcode.app/Contents/Developer`.

## Progress — protocol frozen and harness validated, timing not run

**Pre-registration is provable from history.** `a25f0a2ffdd4e8d546b42b3595895a84ef5398f3` adds `spikes/scheduling/metal_contraction_tile_width/PROTOCOL-2026-08-22-contraction-tile-width.md` and nothing else; at that commit the directory holds exactly that one file, with no harness and no result. `431722dd262ee4e55933c5259ce66d2347dbac6a` corrects two defects in the protocol found while building the harness, still before any dispatch. `7ed93ab9ebbb297eed0e464c315aee24f70c5df1` adds the harness.

**The beneficiary is named:** `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`, and no other.

**Harness validated on the coordination host, with no wall clock read** — 21 of 21 pipelines prepared, the parameterized `(16, 16)` variant byte-identical to the retained `contract_tiled` at all seven validation cells, 126 of 126 cross-variant oracle comparisons matching, zero unwritten outputs, reproduced at load 4.03 and 7.34 with identical digests.

**Not done: the timing sweep.** It has not been run and no width is recommended. The bench host was at load averages `2.13 2.17 2.16` when probed, above the 0.5 gate the protocol freezes, and its macOS build has moved to `26A5388g` from the retained record's `26A5378n`.

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

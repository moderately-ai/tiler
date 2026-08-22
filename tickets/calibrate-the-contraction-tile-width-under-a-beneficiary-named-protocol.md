---
id: calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol
title: Calibrate the contraction tile width under a beneficiary-named protocol
status: blocked
priority: p2
dependencies: []
related: [decide-the-contraction-tile-width-authority, carry-the-contraction-tile-width-policy-as-a-target-profile-row]
scopes: [research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [measurement, scheduling, contraction, target-profiles]
---
## User-visible outcome

The contraction tile width has a swept, protocol-registered measurement naming its beneficiary profile key before the run — so a width can be *chosen* on evidence rather than inherited from the single value that happened to be compiled.

## Why this exists

Filed 2026-08-22 by the coordinator from the tile-width authority packet, which found the existing evidence cannot support any declared width and cannot be repaired.

**Fact — the 16 was never swept.** `spikes/scheduling/metal_contraction_vertical/kernels.metal` declares `constant uint TILE = 16;`. The spike swept *realization families*, not widths. **No width other than 16 has ever been executed.**

**Correction — 2026-08-22 by `worker-tileprotocol`, and the coordinator's version of this Fact would have produced a silently invalid sweep.** I wrote that the constant is the single binding, "with every other `TILE` reference derived from it", and gave the count as "the constant plus five derived uses and one precondition comment". Both halves are wrong. The count is **17 lines / 23 occurrences** in that file (`grep -c` versus `grep -o | wc -l`; the two units differ and I quoted neither). More seriously, **the width is bound at four independent sites across two files**, and the three in `host.m` derive from nothing: the precondition `k_extent % 16u`, the grid divisor `(n_extent + 15) / 16` paired with `(m_extent + 15) / 16`, and `MTLSizeMake(16, 16, 1)`. Verified by the coordinator at `97e7fef1`. **A worker who believed the width lived in one constant would have changed it and dispatched the old shape**, producing timings for a kernel that is not the one it thought it was measuring.

**Fact — the retained record measures the tiled kernel losing.** At `w_vocab_slice`, `tiled` is 523.500 µs against `direct`'s 251.417 and `ksplit_contiguous`'s 234.000 — verified against `timing-summary.tsv` exactly. At `t_vocab_full` it is 9,669 µs against a 4,247 µs best. The record's stated cause, verbatim, is that the `16×16` output tile *"computes one useful row and fifteen masked ones when `M = 1` — a schedule mismatch, not a bandwidth result"*, in `docs/research/scheduling/first-metal-contraction-realizations.md`.

**Correction — 2026-08-22 by `worker-tileprotocol`, two of the coordinator's figures were unsourced.** I wrote "a 2.28x regression" and "a square block wasting fifteen of sixteen rows". **`2.28` appears nowhere in any record** — it is a ratio I derived without saying so, and *which* ratio depends on the denominator: 2.2764 against `ksplit_contiguous`, 2.0327 against `direct`, 2.1883 against MPS. **"fifteen of sixteen" appears in no record either**; the record says "one useful row and fifteen masked ones". Both were carried from a worker report and restated in my own words, which is exactly what AGENTS.md forbids. Verified absent by the coordinator at `97e7fef1`.

**Finding — the record labels an Inference as a Measurement, and this ticket must not inherit that.** The row-waste attribution above sits under a `**Measurement —**` heading in that record, but **no width was swept and no masked-thread count was instrumented**. It is an inference about a cause, carrying measurement authority. Treat it as the hypothesis this sweep tests, never as an established result — and see the null control the frozen protocol registers for exactly this reason.

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

## Coordinator disposition — 2026-08-22, merged at `97e7fef1`; protocol landed, timing held

**Pre-registration is provable from history, not asserted.** The protocol is `a25f0a2f`, and `git ls-tree -r a25f0a2f -- spikes/scheduling/` shows the `metal_contraction_tile_width/` directory containing **only** `PROTOCOL-2026-08-22-contraction-tile-width.md` — no harness, no results. Verified by the coordinator. Sole named beneficiary: `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`.

**The worker did not inherit the sweep design, and that was the right call.** `TILE` is not a width: it is one constant serving as M-block height, N-block width, *and* K-chunk depth simultaneously. Sweeping it alone yields a compound and can only confirm the row-waste hypothesis, never refute it. The frozen protocol therefore carries **two arms** — square over six widths, and rectangular over twelve pairs decoupling block height from chunk depth — plus a registered **null control** at `M = 128`/`512`, where the waste factor is 1 for every admissible width and the mechanism predicts no effect. A protocol that can only confirm is not a protocol.

**Harness validated without reading a clock.** The parameterized `(16,16)` variant collapses statement-for-statement to the retained kernel and matched the reference at all seven validation cells including the masking path; 126/126 cross-variant comparisons matched `direct`; identical digests at load 4.03 and 7.34, which itself demonstrates the correctness checks are load-independent. Two perturbations with their text: a mis-declared threadgroup height left 4,096 elements unwritten and was rejected, and a `+0.0`-seeded twin proved **invisible under PRNG operands** and was caught only under signed zeros — a random-operand corpus alone would have passed a semantically wrong kernel and reported clean.

**HELD: the timing sweep. Release trigger — bench-host load below the 0.5 gate the protocol freezes.** At delivery the M3 Pro reported load `2.13 2.17 2.16` with no process above 3.2% CPU, so the cause is unidentified rather than transient; `--mode timing` refuses outright above the gate, which is the protocol working. The bench host is on build `26A5388g`, **not** the retained record's `26A5378n`, so the retained µs figures are not a baseline and every prediction in the protocol is written as a ratio internal to the new sweep. Command to run when the host is quiet is recorded in the delivery.

**Gate blind spot found by perturbation, filed separately.** `make citations` returns exit 0 with a deliberately broken link under `spikes/`; the checker's population is `tickets/**`, `docs/**`, and root markdown. All sixteen links in this lane's documents were resolved by hand. See [`decide-whether-the-citation-checker-should-reach-spike-records`](decide-whether-the-citation-checker-should-reach-spike-records.md).

**Correction — 2026-08-22 by [`re-run-a-merged-document-s-own-evidence-commands-against-the-merged-tree`](re-run-a-merged-document-s-own-evidence-commands-against-the-merged-tree.md), read at `6f3c2594`: the blind spot this entry filed is closed, and the entry's own reproduction now fails.** Retired wording, preserved: *"`make citations` returns exit 0 with a deliberately broken link under `spikes/`; the checker's population is `tickets/**`, `docs/**`, and root markdown"*. The ticket this entry filed was accepted and landed at `04d5eae9`, in the same batch and ahead of this record, so the sentence was already false at the commit it merged into. Re-run at `6f3c2594`, a broken link planted under `spikes/` makes `make citations` **fail with exit 2**, naming the carrying file and the unresolvable target; the populations are now `tickets/**`, `docs/**`, `spikes/**`, and root markdown, recorded at `check-citations.sh "THE FOUR POPULATIONS, AND WHAT TERMINAL MEANS IN EACH"`, and the run reports a nonzero spike link population over 68 live spike record files, 596 of them at `6f3c2594`. Only the link half closed: pinned source citations under `spikes/` are still declined by decision and printed as a number, `54 pinned citation(s) DECLINED`, because a spike record pins the base its own record names rather than the tip.

**Consequence for this lane, which is the operative half.** A `spikes/` + `docs/` + `tickets/` delta no longer carries the last green gate for free on its `spikes/` half. `make citations` must be rerun for such a delta, as `AGENTS.md "Record the carry reasoning and rerun"` now requires. The sixteen links this lane resolved by hand are nineteen at `6f3c2594` and are now gated, so the hand pass is no longer the only signal. The count grew rather than shrank partly because the dated corrections repairing this drift add references of their own; expect that, and do not read a larger number as a regression.

## Release-trigger correction — 2026-08-22, by `worker-quietgate`

The disposition above is left standing rather than rewritten; this note governs the trigger. Filed from [`re-derive-the-quiet-host-gate-the-bench-host-cannot-satisfy`](re-derive-the-quiet-host-gate-the-bench-host-cannot-satisfy.md), whose whole purpose is to release this hold.

**The stated release trigger could never fire.** It reads *bench-host load below the 0.5 gate the protocol freezes*, and the bench M3 Pro's **idle** one-minute load average is **1.86–2.47** — a floor the OS configuration imposes, not a queue that drains. The same floor is retained in-tree on the same host nine days earlier, at `spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-13-request-wide-macos-27.0-m3-pro.stdout.txt` reporting `loadavg={ 2.22 2.39 2.26 }` and its 2026-08-14 sibling reporting `loadavg={ 2.18 2.23 2.24 }`. So this hold was not waiting, it was foreclosed, and it would have read as *not yet* indefinitely.

**Two supporting observations in the disposition above are also wrong, and are corrected rather than restated in new words.** *No process above 3.2% CPU* does not hold: the bench host carries a live console session running Google Chrome across seven processes, observed with a renderer at **88.8% CPU** and the main process at 21.7%, producing episodic bursts of about 1.1 of its 11 cores. And *the cause is unidentified* is no longer true — the cause is that the load average reads 2.2 on a host measured at 98.8–99.4% CPU idle and 0–1% GPU device utilization, so it cannot discriminate competing work from baseline at any threshold.

**The trigger is replaced.** Read: **release when the protocol's quiet-host gate admits the bench host**, which is now a condition with a reachable satisfying case. Confirm it directly, dispatching nothing and reading no wall clock:

```sh
ssh m3 'cd ~/tiler-tile-width-spike && python3 tile_width_sweep.py --mode gate'
```

Exit 0 admits and exit 1 refuses naming the component and its reason. **Quiesce the interactive console session — quit foreground applications — before the timing leg; do not re-read the gate until it happens to pass**, because that selects for the quiet phase of a host that is not quiet. The gate, the evidence for each threshold, and both perturbation directions are in [the protocol's pre-run amendment](../spikes/scheduling/metal_contraction_tile_width/PROTOCOL-2026-08-22-contraction-tile-width.md) and [the spike README](../spikes/scheduling/metal_contraction_tile_width/README.md).

**Pre-registration is intact.** The amendment was committed before any dispatch was timed: no `results/` directory exists in the spike, no timing artefact is tracked, and the protocol's `Result` section still reads *Not yet run*. Nothing else in the protocol changed — not a cell, variant, prediction, beneficiary key, or measurement boundary.

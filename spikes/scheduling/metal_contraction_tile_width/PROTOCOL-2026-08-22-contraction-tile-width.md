---
schema: "tiler-doc/v1"
id: "tiler.spike.scheduling.metal-contraction-tile-width.protocol-2026-08-22"
kind: "experiment"
title: "Frozen protocol: contraction tile width for the standard macOS Apple9 profile"
topics: ["scheduling", "contraction", "matmul", "metal", "target-profiles", "provenance"]
experiment_status: "frozen"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
ticket: "calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol"
---

# Frozen protocol — contraction tile width for the standard macOS Apple9 profile

**This document is written and committed before the harness exists and before a single time is read.** It follows the shape of [the thread-execution-width lane's protocol](../../target-profiles/metal-thread-execution-width/PROTOCOL-2026-08-22-standard-profile.md), which was committed alone at `f5a274bafe938b3c3a8df6143db0183b4405d135` and run afterwards, so that pre-registration is provable from git history rather than asserted. The same two-step is used here.

The measurement this protocol governs does not yet exist. The [Metal contraction realization probe](../metal_contraction_vertical/README.md) beside it is a *different* record with a *different* frozen scope, and nothing here rescopes it, repairs it, or reads a row out of it into a profile.

## Pre-registered beneficiary

The profile this measurement may inform is named here, before the run:

> `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`

That key, and no other.

[ADR 0113](../../../docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md) component 3 is the clause this section exists to satisfy, quoted from the record:

> **Single-host-to-family rule: pre-registered beneficiary, stated value, measured validity.** A measured row enters a family-keyed profile only when (a) the producing measurement's frozen protocol named that exact profile key as beneficiary **before the run**; (b) the declaring module transcribes only what the record states and refuses every other value by name (the `UnevidencedWidth` pattern); and (c) the row's validity is `MeasuredEnvironment` — the constructor already makes a wider claim unbuildable. A record whose protocol scoped it elsewhere composes into nothing else, ever.

This document discharges **(a)** and nothing else. (b) and (c) belong to the declaring module in `crates/tiler-build`, which is outside this ticket's scope and outside this lane's authority.

`tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` is the only public compile-profile key minted in the tree, at `crates/tiler-build/src/metal_declaration.rs "This is a new content key"` — the comment block on `FIRST_MACOS_APPLE9`. The crate-private M3 Pro fixture key `tiler.metal.macos-m3pro-apple9.msl4-0.subgroup-f32.v1` is deliberately **not** the beneficiary: it is a demoted evidence fixture with no public re-export, and naming it would produce a second record that composes into nothing.

### A boundary this protocol states rather than assumes

Naming the beneficiary discharges component 3(a). It does **not** make the eventual row's admission automatic, and this protocol says so before the run so that the carrier ticket cannot inherit an unstated assumption. A contraction tile-width policy is a **fact family the standard profile does not currently state**. Under ADR 0113 component 1 a key names "backend, artifact/platform family, GPU family, language standard, stated content families, revision", and the record's own consequence for the neighbouring subgroup lane reads:

> Adding the subgroup fact family to the flagship public profile remains a Tom-facing packet under existing rules, because it moves the profile's stated content, descriptor, and every pin.

The same sentence's reasoning reaches a contraction tile-width family for the same reason. So the sequence is: this protocol pre-names the beneficiary (done here), the run produces the evidence, and admitting a new fact family to the flagship profile is a Tom-facing decision packet owned by [`carry-the-contraction-tile-width-policy-as-a-target-profile-row`](../../../tickets/carry-the-contraction-tile-width-policy-as-a-target-profile-row.md). Component 3(a) is a necessary condition, never a sufficient one.

## Pre-named execution environment — the only row this measurement is valid for

The measuring host is the bench M3 Pro reached as `m3` in `~/.ssh/config`. Fields below were read from that host on 2026-08-22 while this protocol was being written; the harness re-derives every one of them at the run and the retained record is authoritative over this table.

| Component | Exact value | Command |
| --- | --- | --- |
| Device | `Apple M3 Pro` | `sysctl -n machdep.cpu.brand_string` |
| OS | macOS 27.0, build `26A5388g` | `sw_vers -productVersion`, `sw_vers -buildVersion` |
| Architecture | `arm64` | `uname -m` |
| Apple GPU family | `apple9`, asserted by the harness before any pipeline is built | `MTLDevice.supportsFamily` |
| Max threads per threadgroup | asserted `>= 1024` by the harness | `MTLDevice.maxThreadsPerThreadgroup` |

**This is not the environment row the retained contraction record was measured on.** That record's timing leg reports `host_os 27.0 26A5378n` in its `environment.tsv`; the bench host has since moved to `26A5388g`. Under ADR 0113 component 2 these are two execution environments and may not be folded. The consequence is stated here before the run so no reader is tempted later: **absolute microsecond figures from the 2026-07-31 timing record are not a baseline for this sweep**, and every refutable prediction below is therefore written as a ratio *internal to this sweep* rather than as a comparison against those figures.

## Pre-named offline compilation environment

The run pins `DEVELOPER_DIR=/Applications/Xcode.app` for every compiler invocation. **A toolchain claim without its invocation is not a fact**, so both invocations were run and both are recorded:

| Host | Invocation | Answer |
| --- | --- | --- |
| Bench `m3` | `xcrun --sdk macosx metal --version` | `Apple metal version 32023.883 (metalfe-32023.883)` |
| Bench `m3` | `DEVELOPER_DIR=/Applications/Xcode.app xcrun --sdk macosx metal --version` | `Apple metal version 32023.883 (metalfe-32023.883)` |
| Coordination host | `xcrun --sdk macosx metal --version` | `Apple metal version 32023.921 (metalfe-32023.921)` |
| Coordination host | `DEVELOPER_DIR=/Applications/Xcode.app xcrun --sdk macosx metal --version` | `Apple metal version 32023.883 (metalfe-32023.883)` |

On the bench host the two agree because its `xcode-select -p` is already `/Applications/Xcode.app/Contents/Developer`; on the coordination host `xcode-select -p` is `/Applications/Xcode-beta.app/Contents/Developer` and the bare form answers for a downloaded Metal toolchain instead. The pin is kept on both regardless, so the recorded invocation is the same string wherever it runs.

| Component | Exact value on `m3` |
| --- | --- |
| Offline compiler | `Apple metal version 32023.883 (metalfe-32023.883)` |
| Xcode | `Xcode 26.6`, build version `17F113` |
| macOS SDK | `macosx` 26.5, build `25F70` |
| Language standard | `-std=metal4.0`, requested target `air64-apple-macos26.0` |
| Governed numerical flags | `-fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off` |

These four offline fields reproduce the retained record's `environment.tsv` offline table field for field, which is deliberate: it removes the offline toolchain as an axis of difference, leaving the execution build as the only environment axis that has moved.

## The subject, and why "width" needed defining before it could be swept

The kernel under study is the threadgroup-memory tiled realization of the index structure `td,od->to` — operands `A[M, K]` and `B[N, K]`, contracted index last on both. In the existing spike its tile is governed by one compile-time constant, `spikes/scheduling/metal_contraction_vertical/kernels.metal "constant uint TILE = 16;"`, and that constant is simultaneously three different quantities:

1. the **M-block height** — how many output rows a threadgroup covers;
2. the **N-block width** — how many output columns a threadgroup covers; and
3. the **K-chunk depth** — how many contracted elements are staged per barrier round.

They are welded together by the load pattern, not by intent: a `TILE × TILE` thread block loading one element each covers a `TILE × TILE` patch, which forces the staged k-extent to equal the block's other dimension. **Sweeping that one constant therefore cannot separate the three effects**, and a sweep that only varied it would produce a compound number and no mechanism. This protocol therefore freezes **two arms**, and the second exists specifically so the hypothesis below can be refuted rather than only confirmed.

- **Square arm.** `TILE_M == TILE_W == W`. Reproduces the existing kernel's structure exactly. At `W = 16` it must be byte-identical to the retained `contract_tiled`, and that identity is a required pre-run check (below).
- **Rectangular arm.** `TILE_M < TILE_W = W`, with `TILE_M` dividing `W`. The M-block height is decoupled from the N-block width and the K-chunk depth, which stay at `W`. Each thread then performs `W / TILE_M` B-operand loads instead of one; the trip count is compile-time constant, so every barrier stays threadgroup-uniform.

The rectangular arm is measured here as evidence, and decides nothing about the compiler's `Square tiles only` lowering restriction, which is [`reconsider-the-square-tiles-only-lowering-restriction`](../../../tickets/reconsider-the-square-tiles-only-lowering-restriction.md)'s question and a `crates/` change this lane may not make.

## Admissible width set, and why each member is in it

Frozen before the run. Constraints, in the order they bind:

1. **Structural precondition** — `K` must be a positive multiple of `W`. The frozen contracted extents are `{1024, 2048, 3072}`; every power of two up to 1024 divides all three, so powers of two are the set that keeps one width admissible at every cell.
2. **Thread limit** — `TILE_M × W <= maxThreadsPerThreadgroup`, which is 1024 on this device (the retained record's `environment.tsv` reports `device_max_threads_per_threadgroup 1024` for this device, and the harness re-asserts it). This binds `W <= 32` in the square arm.
3. **Threadgroup memory** — `(TILE_M × W + W × W) × 4` bytes must fit; at `W = 32` that is 8 KiB, well inside the limit, so it never binds before (2).

**Square arm: `W ∈ {1, 2, 4, 8, 16, 32}`.**

| `W` | Threads/group | Why it is in the set |
| --- | --- | --- |
| 1 | 1 | **Negative control.** One thread per threadgroup, a k-chunk of one, and a barrier per element. It must be strictly slower than the untiled `direct` kernel at every cell; if it is not, the harness is not measuring the kernel it claims to. Not a delivery candidate. |
| 2 | 4 | Degenerate lower end; fills the interval so a monotone trend can be distinguished from a two-point line. |
| 4 | 16 | Below the 32-wide SIMD group, so at least half of every SIMD group idles by construction. |
| 8 | 64 | Two SIMD groups. |
| 16 | 256 | **The incumbent** — the only width ever executed. |
| 32 | 1024 | The maximum constraint (2) admits; one tile row is exactly one SIMD group. |

**Rectangular arm: `(TILE_M, W)` with `TILE_M | W`, `TILE_M < W`, `W ∈ {8, 16, 32}`** — twelve pairs: `(1,8) (2,8) (4,8)`, `(1,16) (2,16) (4,16) (8,16)`, `(1,32) (2,32) (4,32) (8,32) (16,32)`.

Eighteen tiled variants in total, plus the untiled `direct` kernel carried as an in-sweep reference so that every comparison is against a kernel measured in the same process, in the same interleaving, on the same environment row — never against a figure transcribed from the 2026-07-31 record.

## The hypothesis this sweep must be able to refute

The retained research record attributes the tiled kernel's loss at `M = 1` to wasted rows, at `docs/research/scheduling/first-metal-contraction-realizations.md "a schedule mismatch, not a bandwidth result"`. The full clause is: *its 16×16 output tile computes one useful row and fifteen masked ones when `M = 1`*.

**That attribution is an inference, not a measurement, and it is presented in the record under a `Measurement —` label.** No width was ever swept, and no masked-thread count was ever instrumented. Correcting the authority label on that sentence belongs to the record's owner; what this protocol does is decline to inherit it as a premise.

Re-deriving the mechanism from the retained record's own numbers gives it a sharp, falsifiable form. Define the **waste factor** `ω(M, W) = ceil(M / W) × W / M`, the ratio of output rows a threadgroup grid covers to output rows that exist. Multiplying each retained tiled row's useful throughput by its waste factor yields the arithmetic the kernel actually issued:

| Cell | `M` | Useful GFLOP/s | `ω(M, 16)` | Issued GFLOP/s |
| --- | --- | --- | --- | --- |
| `w_prefill_mlp_in` | 128 | 502.7 | 1.00 | 502.7 |
| `w_prefill_mlp_out` | 128 | 503.5 | 1.00 | 503.5 |
| `w_prefill_o` | 128 | 510.6 | 1.00 | 510.6 |
| `t_prefill_mlp_512` | 512 | 503.7 | 1.00 | 503.7 |
| `w_prefill_q` | 10 | 281.6 | 1.60 | 450.5 |
| `w_vocab_slice` | 1 | 32.0 | 16.00 | 512.8 |
| `t_vocab_full` | 1 | 32.2 | 16.00 | 514.9 |
| `w_decode_kv` | 1 | 34.7 | 16.00 | 554.6 |

The three `M = 1` cells issue 512.8, 514.9 and 554.6 GFLOP/s against an ALU-bound 502.7–510.6 GFLOP/s measured at `M >= 128`. **The tiled kernel at `M = 1` is running at its own arithmetic peak, and fifteen sixteenths of that arithmetic is discarded.** That is a stronger and more testable statement than "a schedule mismatch", and it makes four predictions that the sweep can contradict.

`H1 — waste-linear in the ALU-bound regime.` At `M = 1`, square-arm time falls roughly in proportion to `W` while the kernel stays arithmetic-bound. Predicted `t(W=16) / t(W=8) ≈ 2`. **Refuted if that ratio is below 1.5.**

`H2 — decoupling recovers the loss.` At `M = 1`, the rectangular `(TILE_M = 1, W = 16)` variant issues `1/16` of the square `(16, 16)` variant's arithmetic while staging the identical k-chunk depth and reading the identical B-operand bytes. Predicted at least a 3× speedup over `(16, 16)`, landing within 25% of `direct` at the same cell. **Refuted if `(1, 16)` is not at least 2× faster than `(16, 16)`** — which would mean row waste is not the dominant cost and the record's attribution is wrong.

`H3 — the null control, a shape where the mechanism predicts nothing.` At `M = 128` and `M = 512`, `ω(M, W) = 1` for every admissible `W`, so the row-waste mechanism predicts **no width effect at all** at those cells. Any spread observed there is attributable to k-chunk depth, occupancy, or threadgroup-memory pressure — *not* to row waste. **The mechanism is undermined if the width spread at `M = 128` is comparable in magnitude to the spread at `M = 1`**, because that would identify a `W`-dependence that has nothing to do with masked rows.

`H4 — the bandwidth floor is a floor.` No variant at any cell may beat the cell's B-operand traffic divided by the host's achievable bandwidth. A variant that appears to is reading a cache-resident operand, not achieving bandwidth, and must be reported as such rather than as a result.

`H2` and `H3` are the load-bearing pair: `H2` can confirm the mechanism, `H3` can undermine it, and a sweep of the square constant alone could have done neither.

## Shapes, extents, and the measurement boundary — stated before the run

Two cell groups, frozen here.

**Group A — the M-sweep**, at fixed `N = 8192`, `K = 1024`: `M ∈ {1, 2, 4, 8, 16, 32, 128}`. This group exists to resolve the `M`/`W` interaction, which no existing cell can: the retained record has only `M ∈ {1, 10, 128, 512}` and nothing between 1 and 10.

**Group B — the workload cells**, carried so the sweep speaks to the pinned language-model workload rather than to a synthetic shape: `w_decode_kv` (1×1024×1024), `w_prefill_q` (10×2048×1024), `w_prefill_mlp_in` (128×3072×1024), `w_prefill_mlp_out` (128×1024×3072), `w_prefill_o` (128×1024×2048), `w_vocab_slice` (1×8192×1024), `t_vocab_full` (1×151936×1024), `t_prefill_mlp_512` (512×3072×1024).

**A frozen size-based restriction, fixed in advance and not result-dependent.** The two degenerate negative controls `W ∈ {1, 2}` run only on cells whose B operand is at most 32 MiB — all of group A, plus `w_decode_kv`. The reason is runtime: at `t_vocab_full` a one-thread threadgroup over a 622 MB operand is projected in the tens of seconds per dispatch. This restriction is written here, before any time is read, precisely so it can never be mistaken for dropping a variant that measured badly. Every other variant runs at every cell.

### Pre-run correction, 2026-08-22 — the size-based restriction is withdrawn, and the population counts are restated

Written while building the harness, **before any dispatch was executed and before any wall clock was read**. The frozen text above is left standing rather than rewritten, per this repository's correction convention; this note is what governs.

**Defect 1 — the restriction contradicted itself.** The paragraph states a criterion (*B operand at most 32 MiB*) and then an enumeration (*all of group A, plus `w_decode_kv`*), and the two do not select the same cells. Computing `N × K × 4` for every frozen cell: group A is 32 MiB exactly, `w_decode_kv` 4 MiB, `w_prefill_q` 8 MiB, `w_prefill_o` 8 MiB, `w_prefill_mlp_in` 12 MiB, `w_prefill_mlp_out` 12 MiB, `t_prefill_mlp_512` 12 MiB, and `t_vocab_full` 594 MiB. The criterion admits every cell except `t_vocab_full`; the enumeration admits eight of fourteen. A worker following the criterion and a worker following the enumeration would have run different sweeps.

**Defect 2 — the projection the restriction rested on was arithmetically wrong.** The stated reason was that *at `t_vocab_full` a one-thread threadgroup over a 622 MB operand is projected in the tens of seconds per dispatch*. That projection ignored the waste factor. `ω(M, 1) = ceil(M/1) × 1 / M = 1` for every `M`, so `W = 1` issues **no** wasted arithmetic at all — its cost is an occupancy cost, roughly `1/32` of the device's SIMD width, not a `W`-fold arithmetic cost. Re-projecting `t_vocab_full` at `W = 1`: useful work is `2 × 151936 × 1024 = 311` MFLOP, which at an occupancy-limited `505 / 32 ≈ 16` GFLOP/s is about **20 ms** per dispatch, not tens of seconds. The whole sweep re-projects to single-digit minutes.

**Disposition: the restriction is withdrawn entirely. Every variant runs at every cell.** This is the conservative direction on purpose — it *removes* an exclusion rather than adding one, so it cannot bias the sweep toward any result, and it removes the possibility that a variant excluded on a mistaken projection is later mistaken for a variant dropped because it measured badly. No cell, variant, or prediction is otherwise changed.

**Population counts, restated exactly.** The frozen sweep population is **18 tiled variants** (six square, twelve rectangular) plus the untiled `direct` reference, run over **14 distinct cells** — group A's seven, plus group B's eight, less the one cell they share, `(M=1, N=8192, K=1024)`. The metallib additionally exports **three non-sweep functions** that the driver never times as variants: `contract_direct` (the reference, also a sweep row), `contract_tiled_reference` (a verbatim copy of the retained probe's kernel, present only for the byte-identity check), and `contract_tiled_m16_w16_zero_seed` (a deliberately wrong twin the cross-variant oracle must reject). The prepared-pipeline count the harness asserts is therefore **21**, and check 3's frozen counts are the 18, the 14, and the 21. The original text's "1 reference kernel" undercounted the non-sweep functions and is superseded by this sentence.

### Measurement boundary

Stated now so the eventual claim cannot inherit an unstated one, and written to be narrower than the retained record's own boundary rather than to borrow it.

The claim will cover: one device, `Apple M3 Pro`; one execution environment row, macOS 27.0 build `26A5388g`; one offline toolchain, `metalfe-32023.883` under `-std=metal4.0`; F32 only; the index structure `td,od->to` with the contracted index last on both operands and no operand transposed; contracted extents `K ∈ {1024, 2048, 3072}` **and no other**; free extents exactly the `M` and `N` values the two cell groups enumerate; and the threadgroup-memory tiled realization together with the untiled `direct` reference.

The claim will **not** cover: any other Apple GPU family or device; any other macOS build; F16, BF16, or any quantized format; the runtime-compilation path, since every kernel here is offline-compiled and loaded from a metallib; `K = 16`, which the retained record's semantic corpus uses and which `W = 32` cannot divide; any contracted extent outside `{1024, 2048, 3072}`; the `simdgroup` and `MPSMatrixMultiplication` realizations, which are not tiled kernels and are not in this sweep; and multi-dispatch or fused settings, since every cell is one dispatch over freshly seeded output.

The retained record states its own reach as no contracted extent outside `{16, 1024, 2048, 3072}`. This sweep's `{1024, 2048, 3072}` is a strict subset, chosen rather than inherited.

## Metric, baseline, warm-up, repetitions, oracle, noise controls

- **Metric.** `MTLCommandBuffer.GPUEndTime - GPUStartTime` for a single dispatch, in microseconds. Reported as the **settled minimum** over rounds 1–4, with round 0 reported separately, exactly as the neighbouring record defines those terms. Derived GFLOP/s figures use *useful* flops `2·M·N·K`; issued arithmetic is reported as a separate column, never folded into the same number.
- **Baseline.** The in-sweep `direct` kernel and the in-sweep square `W = 16` variant. Both are measured in this run, on this environment row. The 2026-07-31 figures are **not** a baseline, for the environment reason given above.
- **Warm-up.** One untimed dispatch per case per round, discarded.
- **Repetitions.** 5 rounds × 7 timed dispatches, A/B interleaved by manifest order so that no realization is measured as a contiguous block.
- **Oracle.** Every tiled variant must return a result bit-identical to the `direct` kernel at the same cell, because the tiling changes the memory schedule and nothing about the reduction — each thread still folds its own output's contributors in ascending `d`. A variant whose `result_sha256` differs from `direct`'s at any cell is **refused, not timed**, and reported as a correctness failure. This is the oracle that makes a timing meaningful: a faster kernel computing a different reduction is not a faster kernel.
- **Noise controls, and this one is gated rather than recorded.** This is a wall-clock measurement, so AGENTS.md's idle-host discipline applies in full and the neighbouring protocol's record-don't-gate choice would be wrong here. The run aborts unless the bench host's one-minute load average is **below 0.5** at start and at end, and the harness records both. Observed on 2026-08-22 while writing this protocol: `load averages: 2.13 2.17 2.16` with no process above 3.2% CPU — the host is not currently inside its own gate, and the cause must be identified rather than waited out.

### Pre-run amendment, 2026-08-22 — the load gate is replaced by a quiet-host gate this host can satisfy

Written and committed **before any dispatch was timed and before any wall clock was read**, which is the only window in which this correction is admissible: a host precondition amended before the first timing run leaves pre-registration intact, and one amended after a run destroys it. The frozen bullet above is left standing rather than rewritten, per this repository's correction convention; this note is what governs.

**The defect — the gate's satisfying case was unreachable.** The frozen bullet requires the bench host's one-minute load average to be **below 0.5** at start and at end, and `--mode timing` refused outright above it. That threshold is one this machine never reaches. Its idle one-minute load average was observed at **1.86–2.47** across more than twenty observations on 2026-08-22, and the floor is not a transient of that day: this repository already retains it on the same host nine days earlier, at `spikes/program-planning/physical-frontier-budget-calibration/results/2026-08-13-request-wide-macos-27.0-m3-pro.stdout.txt` reporting `loadavg={ 2.22 2.39 2.26 }` and its 2026-08-14 sibling reporting `loadavg={ 2.18 2.23 2.24 }`. So the gate did not delay the run, it **foreclosed** it, and it would have gone on reading as *not yet* indefinitely — the same defect class the deferred-trigger audit found in seven checks whose satisfying case could not occur.

**Why the load average was the wrong instrument, not merely the wrong number.** The figure is a floor the OS configuration imposes rather than a queue that drains. Sustained sampling on 2026-08-22 recorded the host at **98.8–99.4% CPU idle and 0–1% GPU device utilization while its one-minute load average sat at 2.2**, with no process in uninterruptible or disk-wait state. A quantity that reads 2.2 on a machine doing nothing cannot discriminate competing work from baseline at any threshold, because raising the threshold to admit the baseline also admits roughly a core of real work.

**What replaces it.** Four components, each of which must pass, read at start and again at end, recorded either way. The harness implements them in `tile_width_sweep.py "def quiet_host_gate"` and they can be read alone, dispatching nothing, with `--mode gate`.

| Component | Threshold | What it admits | What it refuses |
| --- | --- | --- | --- |
| Mean CPU idle over ten one-second samples | `>= 95%` | The quiet host, measured at 97.2–99.7% idle | Anything consuming about a core or more, immediately and without lag |
| GPU device utilization, peak over `IOAccelerator` nodes | `<= 5%` | The quiet device, measured at 0–1% | Another workload holding the device this measurement times |
| One-minute load average | `<= 3.5` | This host's recorded baseline of 1.86–2.47 | Sustained work, including work blocked rather than running, that the CPU window may sit between |
| Exclusive advisory lock on `/tmp/tiler-contraction-tile-width-sweep.lock` | held | A single measurement session | A second sweep sharing the device |

The CPU idle mean is the **primary discriminator** and the load average is demoted to a lagging cross-check, because the load average is a lagging indicator by construction. That is not a theoretical objection: measured here, twenty-five seconds into four cores of deliberate competing work, CPU idle had already fallen to **61.17%** while the one-minute load average was still **2.89** — inside any ceiling that also admits the 2.2 baseline. A gate resting on load alone would have admitted that run.

**The threshold is placed in a measured empty band, not chosen round.** Sixty consecutive one-second idle samples on 2026-08-22 are bimodal: a quiet mode at 97.2–99.7% and an episodic desktop-session burst at 84.2–89.8%, about 1.1 of the 11 cores. Nothing was observed between 90% and 95%. The floor sits inside that gap, so it separates the two modes rather than cutting through either.

**A sampling detail that is load-bearing, because getting it wrong reads a different quantity.** `top`'s **first** sample is a since-boot average, not an instantaneous reading. The harness therefore requests one extra sample and discards it. Before that was understood, `top -l 1` on this host read 80.40% and 69.18% idle within minutes of sustained sampling reading 98.8–99.4% — so a gate written naively against `top -l 1` would refuse a host that was in fact quiet, and its author would have concluded the host was busy.

**Every component fails closed, and that is the point of the amendment rather than a detail of it.** A component whose input cannot be read **refuses**; it does not pass. An unreadable probe and a quiet host produce the same silence, and "quiet" is the direction that admits a contaminated measurement. This is written in response to the deferred-trigger audit's finding of a check that polled a `system_profiler` data type macOS had renamed, returned empty, and therefore read as *not fired* forever. The discipline is not novel here: the one other executable host-quietness gate in this repository already applies it, at `spikes/program-planning/reduction-partition-calibration/src/main.rs "fn concurrent_build_processes"`, whose `map_or(usize::MAX, ...)` makes an unreadable `ps` fail the `the timed tree-width study requires no concurrent Cargo, rustc, or make process` assertion rather than satisfy it. That gate is also the closest working precedent for what this amendment does — it discriminates competing work by counting it, and so admits a host whose load average never falls.

**What it would take for this gate to say no, and confirmation that each case is reachable.** Stated because a gate whose refusal is unreachable is the defect being removed, reintroduced in the other direction. All four were observed on the bench host on 2026-08-22 and their failure text is quoted in `README.md`: competing CPU work drives the idle mean below the floor; a renamed `Device Utilization %` key makes the GPU probe unreadable and refuses instead of passing; sustained competing work drives the one-minute load average above the ceiling; and a second process holding the lock refuses. The admitting case was observed on the same host in the same minutes.

**An operational consequence, stated rather than left for the operator to discover.** The bench host carries an interactive console session, and that session produces episodic bursts of roughly one core. The gate correctly refuses during them, so a read taken at an arbitrary moment currently refuses a meaningful fraction of the time. **The remedy is to quiesce the interactive session — quit foreground applications — before the run, not to re-read the gate until it happens to pass.** Re-reading until green selects for the quiet phase of a host that is not quiet, which is the contamination this gate exists to exclude. Quiescing a user session is an operator action; it is not a change to the evidence environment, and nothing here authorizes disabling the system extensions that carry the load-average floor, which are the machine's networking and are reserved to Tom.

**The frozen stop-conditions bullet below is superseded in wording only.** Where it reads *the load gate is not met at start or at end*, read: **the quiet-host gate refuses at start or at end**. The run aborts in the first case having timed nothing; in the second the tables are written so the run stays inspectable, and no timing claim may rest on them. Nothing else in this protocol changes — not a cell, not a variant, not a prediction, not the beneficiary key, and not the measurement boundary.

## Stop conditions, frozen before the run

The run aborts with no timing claim and no retained record if:

- there is no default Metal device, or it does not report `supportsFamily(Apple9)`;
- the device reports `maxThreadsPerThreadgroup.width < 1024`;
- the pre-run byte-identity check below fails;
- any command buffer returns other than `MTLCommandBufferStatusCompleted`, or a non-nil `error`;
- any cell reports a non-zero unwritten-element count;
- the driver's independent reconstruction of the operand stream disagrees with the host's reported operand digest;
- the load gate is not met at start or at end.

A structural refusal — `K` not a multiple of `W`, or a threadgroup exceeding the device limit — is **recorded as a refused row, not an abort**. Refusals are part of the result.

If no width dominates, that is a complete and reportable outcome. The ticket's own closing condition admits it: *the result either supports a width choice or records that none is supportable*. A per-shape frontier with no single winner is the outcome this protocol most expects, given that `H3` predicts a different optimum at `M = 128` than at `M = 1`.

## Required pre-run checks, and what each would have to do to say no

Frozen here so that a green run cannot be assembled after the fact.

1. **Byte-identity against the retained kernel.** The square `W = 16` variant must produce a `result_sha256` identical to the retained `contract_tiled` at a shared cell. This is the check that proves the new harness measures the *same subject* as the record it is calibrating, rather than a kernel that merely resembles it. It says no if the parameterized rewrite changed the reduction: a differing digest fails the run.
2. **Cross-variant oracle.** Every one of the eighteen variants must match `direct` bit for bit at every cell it runs. It says no if a variant's tiling perturbs contributor order.
3. **Population cardinality, printed not assumed.** The driver prints the count of variants, cells, and case-rows it is about to run and the counts are asserted against the frozen numbers in this document — 18 tiled variants, 1 reference kernel, 14 distinct cells. It says no if a variant silently fails to compile and the sweep quietly shrinks: a compile failure removes a pipeline, and a population floor is the only thing that distinguishes that from a clean run.
4. **A deliberate perturbation of the subject, with its failure text quoted in the record.** At minimum: change one variant's accumulator seed from the first product to `+0.0` and confirm the cross-variant oracle rejects it, and mis-declare one variant's threadgroup size and confirm the identity check rejects it. Perturbing the assertions would show only that they run.

## What this protocol does not authorize

- **It declares nothing.** No profile row, no fact family, no key. See the boundary section above: admitting a contraction tile-width family to the flagship profile moves its stated content and every dependent pin, and stays a Tom-facing packet.
- **It changes no compiled code.** Not the landed schedule, not lowering, not emission, and not the `Square tiles only` restriction — the rectangular arm is spike evidence for that separate ticket, not a decision on it.
- **It repairs nothing.** ADR 0113 forbids retroactively rescoping the 2026-07-31 record, and this protocol reads that record for hypothesis formation only. Reading a record is not composing a row out of it.
- **It makes no family, portability, or cross-build claim.** A width measured on one M3 Pro under build `26A5388g` is not an Apple9-family fact and is not a fact about build `26A5378n`, on which the record it is calibrating against was taken.

## The standing hazard this spike sits inside

`spikes/` is outside every repository gate. `make check` and `make full` do not compile, run, or lint anything in this directory, and the delta that carries this work is a `spikes/` + `docs/` + `tickets/` delta that reuses the last green gate for exactly that reason.

So this harness breaks **silently**. Concretely, it would break if: the Metal language or the pinned toolchain stopped accepting the token-pasting macro that instantiates the eighteen variants; `MTLDevice.maxThreadsPerThreadgroup` fell below 1024 on a future bench host, making the `W = 32` arm structurally inadmissible while the driver still listed it; or the retained `metal_contraction_vertical` sources moved, which would break the byte-identity check's reference. None of those is a compile error in any gated crate, and nothing in CI exists to notice, because there is no CI.

A future reader finds out in exactly one way: by running the frozen commands below and reading the cardinality line, which prints the population it is about to measure and fails against the frozen counts rather than proceeding with a shrunken one. That is the only tripwire this directory has, and it is the reason check 3 above is a required pre-run check rather than a convenience.

## Commands, frozen

From this directory. Every compiler invocation carries the `DEVELOPER_DIR` pin.

```sh
# 1. Ship the harness to the bench host.
tar -cf - kernels.metal host.m tile_width_sweep.py \
  | ssh m3 'mkdir -p ~/tiler-tile-width-spike && tar -xf - -C ~/tiler-tile-width-spike'

# 2. Read the quiet-host gate alone. Dispatches nothing, reads no wall clock.
#    Exit 0 admits, exit 1 refuses and prints which component refused and why.
ssh m3 'cd ~/tiler-tile-width-spike && python3 tile_width_sweep.py --mode gate'

# 3. Cardinality and compile-only validation. No wall clock is read.
ssh m3 'cd ~/tiler-tile-width-spike && DEVELOPER_DIR=/Applications/Xcode.app \
  python3 tile_width_sweep.py --mode validate --work-dir work'

# 4. The sweep. Serial, one process, A/B interleaved by manifest order.
ssh m3 'cd ~/tiler-tile-width-spike && DEVELOPER_DIR=/Applications/Xcode.app \
  python3 tile_width_sweep.py --mode timing --rounds 5 --reps 7 \
  --out results-timing --work-dir work'

# 5. Retrieve.
ssh m3 'cd ~/tiler-tile-width-spike/results-timing && tar -cf - .' \
  | tar -xf - -C results/2026-08-22-timing-apple9-f32-msl4-macos27-m3pro-26A5388g-metal32023.883
```

## Result

*Not yet run. This section is appended by the run, in a later commit, exactly as the neighbouring lane's protocol appends its own.*

The commit that carries this document without a `Result` section, and without the harness beside it, is the pre-registration. Its hash is recorded in the ticket.

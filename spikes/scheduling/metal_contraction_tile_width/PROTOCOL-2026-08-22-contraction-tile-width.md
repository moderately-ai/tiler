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

# 2. Cardinality and compile-only validation. No wall clock is read.
ssh m3 'cd ~/tiler-tile-width-spike && DEVELOPER_DIR=/Applications/Xcode.app \
  python3 tile_width_sweep.py --mode validate --work-dir work'

# 3. The sweep. Serial, one process, A/B interleaved by manifest order.
ssh m3 'cd ~/tiler-tile-width-spike && DEVELOPER_DIR=/Applications/Xcode.app \
  python3 tile_width_sweep.py --mode timing --rounds 5 --reps 7 \
  --out results-timing --work-dir work'

# 4. Retrieve.
ssh m3 'cd ~/tiler-tile-width-spike/results-timing && tar -cf - .' \
  | tar -xf - -C results/2026-08-22-timing-apple9-f32-msl4-macos27-m3pro-26A5388g-metal32023.883
```

## Result

*Not yet run. This section is appended by the run, in a later commit, exactly as the neighbouring lane's protocol appends its own.*

The commit that carries this document without a `Result` section, and without the harness beside it, is the pre-registration. Its hash is recorded in the ticket.

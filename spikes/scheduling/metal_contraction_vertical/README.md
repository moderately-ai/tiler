---
schema: "tiler-doc/v1"
id: "tiler.spike.scheduling.metal-contraction-vertical"
kind: "experiment"
title: "Metal contraction realization probe"
topics: ["scheduling", "contraction", "matmul", "metal", "numerics", "reductions", "language-model"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement", "executable-model"]
supports: ["tiler.research.scheduling.first-metal-contraction-realizations"]
entrypoints: ["spikes/scheduling/metal_contraction_vertical/contraction_probe.py", "spikes/scheduling/metal_contraction_vertical/kernels.metal", "spikes/scheduling/metal_contraction_vertical/host.m"]
last_verified: "2026-07-31"
ticket: "spike-first-metal-contraction-vertical"
---

# Metal contraction realization probe

## The named question

For the index structure `td,od->to` — which the [L2 derivation](../../../docs/research/shapes/transformer-operation-and-shape-surface.md) resolves 197 of the pinned workload's 253 contraction occurrences into — **which realization on an Apple9 GPU actually computes the reduction the governed F32 contract requires, and at what cost?**

Four realization families were named ahead of the measurement: a direct kernel with one thread per output, a threadgroup-memory tiled kernel, a `simdgroup_float8x8` matrix-multiply-accumulate kernel, and an opaque `MPSMatrixMultiplication` library call. Two more exist here because the reduction contract distinguishes them and nothing else in the corpus measures the distinction: a contracted-axis split into contiguous intervals, which consumes reassociation, and one into strided subsets, which consumes reassociation *and* permutation.

The question is not "which is fastest". A realization whose reduction topology cannot be stated is not a slower correct option; it is an option with no numerical contract, and this probe exists to say which is which before any cost is compared.

## What it does and does not establish

**It establishes**, on the exact host rows recorded below and for the eight designed cases and six workload cells it runs:

- which named reduction topology each realization's returned bits are consistent with, and which topologies are *refuted*, with the refuting case named per topology;
- whether each realization reproduces a host-computed binary32 strict left fold bit for bit at the workload's own extents;
- which structural preconditions each realization refuses rather than approximating;
- the settled GPU time of each realization at each cell on one bench host.

**It does not establish** a topology *guarantee* for any realization it did not author. `simdgroup_multiply_accumulate` and `MPSMatrixMultiplication` publish no accumulation order and no internal precision; twenty-two named topologies over eight cases is exhaustive elimination over a finite candidate set, not a proof that the surviving one is what the hardware does. It says nothing about another Apple GPU family, another dtype, another toolchain, or a contracted extent this profile does not contain. And it compiles no Tiler program, registers no operation, and plans nothing: the kernels are hand-written, and the reduction topologies are modelled here rather than derived from a schedule.

**It installs and mutates nothing.** It reads the compiler already on the host, writes into a scratch directory it creates, and — for the timing leg — copies three source files to a bench host over an existing SSH route.

## Reproduce

No `make` target reaches `spikes/`. From **this directory**:

```sh
# Correctness. `numpy` is needed only for the workload leg's host oracle.
uv run --with numpy python contraction_probe.py \
  --mode correctness \
  --out results/2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883

# The semantics leg alone needs no third-party package at all.
python3 contraction_probe.py --mode semantics --out /tmp/tiler-contraction-semantics

# The same run under an optimized interpreter, which must produce byte-identical
# TSVs. No verdict here may rest on a statement `-O` deletes.
uv run --with numpy python -O contraction_probe.py \
  --mode correctness --out /tmp/tiler-contraction-optimized
```

The timing leg runs on the bench host, serially, in one process, with the A/B interleaving carried by the manifest order the driver writes. `m3` is the `~/.ssh/config` entry for that host:

```sh
tar -cf - kernels.metal host.m contraction_probe.py \
  | ssh m3 'mkdir -p ~/tiler-contraction-spike && tar -xf - -C ~/tiler-contraction-spike'
ssh m3 'cd ~/tiler-contraction-spike && python3 contraction_probe.py \
  --mode timing --rounds 5 --reps 7 --out results-timing --work-dir work'
ssh m3 'cd ~/tiler-contraction-spike/results-timing && tar -cf - .' \
  | tar -xf - -C results/2026-07-31-timing-apple9-f32-msl4-macos26-m3pro-metal32023.883
```

Every compilation uses the governed Apple9/F32 baseline the [workload profile](../../../docs/research/program-planning/first-metal-lm-workload.md) records — `-fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`, at `-std=metal4.0` for `air64-apple-macos26.0` — and `environment.tsv` carries the exact flag list beside the result.

## How one run works, in order

1. Compile `kernels.metal` under the governed flags, link a metallib, and build `host.m`.
2. Write a manifest of cases. The host runs them in manifest order and never reorders, which is what makes the timing leg interleaved rather than blocked per realization.
3. For each case the host seeds the output allocation with `-3.0e38f` — a *finite* pattern no admitted case can produce, since every result here is bounded by 768 in magnitude — dispatches, requires `MTLCommandBufferStatusCompleted` and a nil `commandBuffer.error` before touching the allocation, then reports how many elements still hold the seed. The seed is finite rather than NaN because `MPSMatrixMultiplication` computes `alpha * A*B + beta * C` and a NaN seed would poison the result through `0 * NaN`.
4. Operands come from a SplitMix64 stream generated in the host, so a 622 MB weight matrix never reaches the filesystem. The host prints the SHA-256 of the exact operand bytes; the driver reconstructs the same stream independently and **stops** if the digests disagree. Without that check the host oracle would be a comparison against operands the driver merely believes were used.
5. The driver computes each named topology's exact result in rational arithmetic, rounds once to binary32, and classifies. Each topology is evaluated twice, once preserving subnormals and once flushing them sign-preservingly, because flush-to-zero is the *declared* realization of this target row and not a deviation from it.

## Retained records

Two, because correctness and performance were measured on different hosts and the two claims must not be read as one row.

- [`results/2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/`](results/2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883) — Apple M4 Max, macOS 27.0 build `26A5388g`. Holds `semantics-candidates.tsv` (every named topology's exact bits per case), `semantics-observations.tsv` (what each realization returned), `semantics-attribution.tsv` (the corpus-level elimination, with the refuting case per refuted topology), and `workload.tsv`.
- [`results/2026-07-31-timing-apple9-f32-msl4-macos26-m3pro-metal32023.883/`](results/2026-07-31-timing-apple9-f32-msl4-macos26-m3pro-metal32023.883) — Apple M3 Pro, macOS 27.0 build `26A5378n`. Holds `timing.tsv` (every round) and `timing-summary.tsv`.

Both carry `environment.tsv` and a `manifest.tsv` of SHA-256 digests over the retained files and over all three producer sources. The two hosts share Xcode 26.6 build `17F113`, macOS SDK 26.5 build `25F70`, and offline compiler `metalfe-32023.883`, and differ in GPU and in macOS build; both report `MTLGPUFamilyApple9`.

## Findings

**Measurement — attribution is unique for five realizations and empty for the sixth.** Over the eight-case corpus, each realization's returned bits are consistent with exactly one of the twenty-two named topologies, and every other topology is refuted with a named case. `direct` and `tiled` are both `strict_fold+ftz`; `ksplit_contiguous` is `contiguous_split+ftz`; `ksplit_strided` is `strided_split+ftz`; `simdgroup` is `fma_zero_seed_fold+ftz` — a *fused* left fold over an accumulator seeded at `+0.0`. `opaque_mps` is consistent with **none of the twenty-two**.

**Measurement — `-ffp-contract=off` does not reach a matrix-multiply-accumulate instruction.** On `contraction_pair`, the four hand-written scalar kernels return the separately rounded `0x3fc58f9e` while `contract_simdgroup` — compiled in the same module under the same `-ffp-contract=off` — returns the fused `0x3fc58f9d`, as does MPS. This is [finding 16 of the Apple numerical-behaviour record](../../../docs/research/apple-targets/numerical-behaviour.md) at a new construct: the flag is a defence against the compiler contracting a written multiply and add, and is no defence against a fused operation the source asked for. `simdgroup_multiply_accumulate` is such an operation.

**Measurement — the accumulator seed is observable and two realizations get it wrong for an unseeded reduction.** On `negative_zero_seed`, where every product is `-0.0`, `direct`, `tiled`, and both split kernels return `0x80000000` and `simdgroup` and `opaque_mps` return `0x00000000`. A `+0.0` seed is not a defect by itself — it is a reduction carrying an explicit `initial`, which `docs/numerical-semantics.md` admits as a different semantic operation — but it is a defect for a contraction that does not declare one.

**Measurement — the declared subnormal flush is delivered by every realization, including the opaque one.** On `subnormal_product`, whose exact product is `2^-127`, all six return `0x00000000`, matching only the flush-to-zero candidates.

**Measurement — the canonical arithmetic NaN pattern is what comes back.** On `nan_payload` the contributor `0x7fc0dead` yields `0x7fc00000` from every realization, and `payload_propagating_fold` is refuted for all six. `infinity_times_zero` returns `0x7fc00000` too. That is the pattern `tiler::canonical-arithmetic-nan-f32@1` names; it is one corpus on one device and not evidence that the per-combine canonicalization obligation is discharged in general.

**Measurement — the tiled schedule preserves the fold exactly.** `direct` and `tiled` produce byte-identical results at all six workload cells (identical `result_sha256`), which is the design claim: the tiling is over the free indices and over contiguous chunks of the contracted index, and each thread still folds its own output in ascending order.

**Measurement — MPS coincides with the simdgroup kernel at some shapes and not others.** At the three cells with `M = 128` the two are byte-identical. At `M = 1` and `M = 10` the simdgroup kernel refuses on its own precondition while MPS runs and returns something different again, and at the semantic corpus's `16x16x16` MPS returns `0xbb1d0600` where the simdgroup kernel returns `0xbb1d047f`. So the opaque call's reduction topology is *shape-dependent* on one device, one dtype, one driver.

**Measurement — the cost separation is shape-dependent, and it inverts.** On the M3 Pro, at the `B1-b` prefill cell (`M=512, N=3072, K=1024`) MPS runs in 839 µs against the strict-fold `tiled` kernel's 6,395 µs — 7.6× — and the simdgroup kernel's 3,734 µs. At the complete decode vocabulary projection (`M=1, N=151936, K=1024`) the ordering collapses and inverts: `ksplit_contiguous` 4,247 µs, MPS 4,418 µs, `direct` 4,757 µs, all within 12% of each other, because 622 MB of weights at 131–146 GB/s is the whole cost. Settled spread across rounds is under 2% on every row but one, which is 5.0%.

**Measurement — the small decode cell is cache-resident and does not extrapolate.** At `w_decode_kv` (`M=1, N=1024, K=1024`) MPS runs in 15.5 µs against `tiled`'s 60.5 µs. That weight is 4.19 MB, so 15.5 µs implies 270 GB/s — above this host's DRAM bandwidth — because the harness reuses one operand buffer across dispatches. A real decode step walks 28 layers of distinct weights totalling about 1.76 GiB and cannot be cache-resident, so this row bounds a cache-warm kernel and is not an estimate of the decode step's aggregate.

## The checks can say no

Four perturbations were run on 2026-07-31 against a scratch copy of the three sources, and each produced the failure it was supposed to. The sources were restored between perturbations and the retained records were produced from the unperturbed tree.

1. **The binary32 rounding model is checked before anything rests on it.** Replacing `round_to_f32`'s ties-to-even rule with ties-away failed the self-check on two vectors — `round_to_f32(16777217) expected 4b800000 observed 4b800001` and the subnormal quantum tie `expected 00000000 observed 00000001` — and the run exited 1 with `binary32 rounding self-check failed; no classification is admissible` before dispatching anything.
2. **The oracle compares against the operands the device consumed.** Changing the driver's SplitMix64 increment from `0x2545F4914F6CDD1D` to `…1F` produced `w_decode_kv: host operand digests disagree with the reconstruction; no comparison against this cell is admissible` and exit 1, at the first cell.
3. **An unwritten output is not read as a result.** Making `contract_direct` return without writing `C[0, 0]` turned every one of that realization's eight rows into `inadmissible-unwritten-1` with the observed bits reported as the seed pattern `ff61b1e6`, while the other six realizations were unaffected.
4. **The governed contraction flag is what holds the fusion back.** Recompiling with `-ffp-contract=fast` and nothing else changed flipped `direct`, `tiled`, and `ksplit_contiguous` on `contraction_pair` from `0x3fc58f9e` to the fused `0x3fc58f9d`, moved all three to `disagrees-with-declared`, and re-attributed `direct` from `strict_fold+ftz` to `fma_fold+ftz`. So the `strict_fold` attribution under the governed flags is a property of those flags and not of the kernel text.

A fifth demonstration is permanent rather than a perturbation: `contract_direct_zero_seed` is a deliberately wrong twin of the direct kernel, differing only in seeding its accumulator at `+0.0`, and the corpus attributes it to `zero_seed_fold+ftz` and refutes `strict_fold+ftz` for it. A classification that could not separate the two would separate nothing.

A sixth is structural: the harness carries no executable `assert`. The whole correctness run was repeated under `python -O` on 2026-07-31 and all four TSVs compared byte-identical with `cmp -s`, so no verdict here can be deleted by an optimized interpreter.

## Measurement boundary

One GPU per claim, and the claims are not interchangeable. Correctness is measured on an Apple M4 Max under macOS build `26A5388g`; performance on an Apple M3 Pro under macOS build `26A5378n`. Both are Apple9 and share one offline toolchain, and neither result is transferred to the other host. Nothing here reaches another Apple GPU family, an iOS device or simulator, F16, BF16, any quantized format, or a contracted extent outside `{16, 1024, 2048, 3072}`. The `simdgroup` and `opaque_mps` topology results are empirical eliminations over a finite named set; under the repository's evidence classes they qualify a bounded profile and establish no worst-case or universal behaviour. The runtime-compilation path is not exercised at all — every kernel here is offline-compiled and loaded from a metallib — so the [runtime compiler's separate contraction behaviour](../../../docs/research/apple-targets/numerical-behaviour.md) is neither reproduced nor contradicted.

## Traceability

- **Supported claim:** [First Metal contraction realizations](../../../docs/research/scheduling/first-metal-contraction-realizations.md).
- **Semantic identity this composes with, and does not reopen:** [ADR 0087](../../../docs/decisions/0087-model-contraction-as-one-keyed-family-with-an-index-structure.md).
- **Shapes and extents:** the [L2 derivation](../../../docs/research/shapes/transformer-operation-and-shape-surface.md) and the [L1 workload profile](../../../docs/research/program-planning/first-metal-lm-workload.md).
- **Neighbouring measurement this extends to a new construct:** [Apple GPU numerical behaviour](../../../docs/research/apple-targets/numerical-behaviour.md), findings 6 and 16.
- **Reduction vocabulary:** [Reduction semantics and legality](../../../docs/research/numerics/reduction-semantics-and-legality.md).
- **Work record:** [`spike-first-metal-contraction-vertical`](../../../tickets/spike-first-metal-contraction-vertical.md).

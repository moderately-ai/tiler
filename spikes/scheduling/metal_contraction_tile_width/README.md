---
schema: "tiler-doc/v1"
id: "tiler.spike.scheduling.metal-contraction-tile-width"
kind: "experiment"
title: "Contraction tile-width sweep"
topics: ["scheduling", "contraction", "matmul", "metal", "target-profiles"]
experiment_status: "harness-validated"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
entrypoints: ["spikes/scheduling/metal_contraction_tile_width/tile_width_sweep.py", "spikes/scheduling/metal_contraction_tile_width/kernels.metal", "spikes/scheduling/metal_contraction_tile_width/host.m"]
last_verified: "2026-08-22"
ticket: "calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol"
---

# Contraction tile-width sweep

## The named question

For the threadgroup-memory tiled realization of `td,od->to` on an Apple9 GPU, **what tile shape should a target profile carry, and is the retained record's attribution of the `M = 1` loss to masked rows correct?**

The pre-registration that governs every answer is [`PROTOCOL-2026-08-22-contraction-tile-width.md`](PROTOCOL-2026-08-22-contraction-tile-width.md). Read it first. It names the beneficiary profile key before the run, freezes the admissible width set, and states four predictions the sweep can contradict.

**The timing sweep has not been run.** This directory currently holds a *validated harness* and nothing else: there is no `results/` directory, no timing table, and no width recommendation. What has been established is that the harness measures the subject it claims to.

## Why this is a second spike rather than an edit to the first

The [Metal contraction realization probe](../metal_contraction_vertical/README.md) beside it cannot be extended in place. Its retained records pin SHA-256 digests over their own producer sources — `producer.sha256.host.m` and `producer.sha256.kernels.metal` in each `manifest.tsv` — so editing either file to accept a tile-width parameter would break the custody of a record this lane has no authority to touch, and which [ADR 0113](../../../docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md) forbids repairing anyway. A new directory leaves that record byte-identical.

## What it does

One metallib exports 21 functions:

- **18 sweep variants** — `contract_tiled_m<TILE_M>_w<TILE_W>`, six square and twelve rectangular, all instantiated from one template body so that no variant can drift from another.
- **`contract_direct`** — the untiled reference, copied verbatim from the probe, carried as an in-sweep row so every comparison is against a kernel measured in the same process on the same environment row.
- **`contract_tiled_reference`** — the probe's `contract_tiled` verbatim at its compiled-in `TILE = 16`, present only so the parameterized `(16, 16)` variant can be held against it bit for bit.
- **`contract_tiled_m16_w16_zero_seed`** — a deliberately wrong twin differing only in seeding its accumulator at `+0.0`, which the oracle must reject.

## Harness validation — 2026-08-22, and it is not a timing result

Run on the **coordination host** (Apple M4 Max, macOS 27.0 build `26A5416b`), **not** the bench host, while three compiling worker lanes were live. The one-minute load average was 4.03. That is why this leg reads no wall clock: every check below is a compile-success, cardinality, or bit-identity check, none of which a busy machine can move. The driver prints `NO WALL CLOCK WAS READ` on success for exactly this reason, and `--mode timing` refuses outright above a load of 0.5.

Toolchain, with the invocation that produced it — `DEVELOPER_DIR=/Applications/Xcode.app xcrun --sdk macosx metal --version` answered `Apple metal version 32023.883 (metalfe-32023.883)`, under `Xcode 26.6`, SDK 26.5 build `25F70`.

| Check | Result |
| --- | --- |
| Population floor — prepared pipelines | 21 of a frozen 21; all 18 variants compiled |
| Population floor — variants, cells | 18 and 14, both against their frozen counts |
| **Byte-identity** against the retained kernel | `(16, 16)` matched `contract_tiled_reference` at **all 7** validation cells |
| **Cross-variant oracle** | **126** variant-cell comparisons against `direct`, **0** mismatched |
| Unwritten outputs | 0 across every case |
| Oracle rejects the wrong twin | rejected under signed-zero operands |
| Operand custody | host digests agreed with an independent pure-Python reconstruction |

The validation cells include `M = 3` and `M = 17`, deliberately indivisible by every tile height above 1, so the masking path is exercised rather than assumed.

### What proves the harness measures its subject

Byte-identity is the load-bearing one. The parameterized body is written so that at `TILE_M == TILE_W` its B-load loop runs exactly one iteration and every index expression collapses to the retained kernel's statement for statement. The check confirms that the rewrite did not perturb the reduction: the `(16, 16)` variant is the retained `contract_tiled`, not a kernel resembling it. Without it, a width sweep would be calibrating something other than the thing the record measured.

### The checks can say no

Two perturbations of the **subject**, each run and each quoted:

1. **Mis-declared threadgroup shape.** Dispatching `contract_tiled_m16_w16` with a threadgroup height of 8 rather than the 16 its template was instantiated with left **4096 output elements unwritten** and produced a digest the oracle **rejected**, against a correct dispatch at the same cell that was accepted with 0 unwritten. Both the unwritten floor and the oracle catch it independently.

2. **The `+0.0` accumulator seed, and the corpus that cannot see it.** This one is worth stating carefully, because it shows a check failing to reach its subject. Under PRNG operands the wrong twin returned a digest **identical** to `direct` — `fl(+0.0 + x) == x` for every `x` but `-0.0`, so the defect is *invisible*. Only under `const:80000000,00000000`, where every product is `-0.0`, did the oracle reject it. **A validation corpus of random operands alone would have passed a semantically wrong kernel and reported a clean run.** The signed-zero case exists because of that, and a future reader who deletes it deletes the only thing separating a strict fold from a seeded one.

## A structural property found by reading, which bounds how the sweep should be read

**Inference, not measurement.** The B-operand load is not coalesced, in the retained kernel and therefore in every variant here, because the variants faithfully generalize it. Thread `(local_m, local_n)` stages `b[(n0 + local_n) * K + k0 + local_m + r * TILE_M]`, and `local_n` is the fastest-varying index, so lanes adjacent in a SIMD group read addresses `4 × K` bytes apart — 4096 bytes at `K = 1024`. The A-operand load is contiguous across `local_n` and is coalesced.

This was noticed while writing the generalization and has not been measured. It matters for reading the eventual result in two ways: prediction `H4`'s bandwidth floor may not be reachable by any tiled variant at all, and a rectangular variant that wins may be winning partly on load-count reduction rather than only on discarded arithmetic. A coalesced B-stage is a *different kernel*, not a different width, so it is out of scope here; it is a candidate follow-up rather than a change to this sweep.

## The standing hazard

`spikes/` is outside every gate. `make check` and `make full` compile, run, and lint nothing here, and there is no CI. **This harness breaks silently.** The protocol's own hazard section enumerates the concrete ways; the single tripwire is that `--mode validate` prints the population it is about to measure and fails against the frozen counts rather than proceeding with a shrunken one. A variant that stops compiling turns into a prepared-pipeline count of 20 against a floor of 21, and the run stops.

## Reproduce

No `make` target reaches `spikes/`. From **this directory**. No third-party package is needed in any mode.

```sh
# Compile, prepare, and check. Reads no wall clock; valid on a loaded host.
DEVELOPER_DIR=/Applications/Xcode.app python3 tile_width_sweep.py \
  --mode validate --work-dir work

# Show that the checks can fail, by breaking the subject.
DEVELOPER_DIR=/Applications/Xcode.app python3 tile_width_sweep.py \
  --mode perturb --perturbation wrong-threadgroup --work-dir work
DEVELOPER_DIR=/Applications/Xcode.app python3 tile_width_sweep.py \
  --mode perturb --perturbation zero-seed-under-oracle --work-dir work
```

The timing leg runs on the bench host under the frozen commands in the protocol, and only there. It refuses to start above a one-minute load average of 0.5.

## Traceability

- **Governing pre-registration:** [`PROTOCOL-2026-08-22-contraction-tile-width.md`](PROTOCOL-2026-08-22-contraction-tile-width.md).
- **The record being calibrated, and not repaired:** [First Metal contraction realizations](../../../docs/research/scheduling/first-metal-contraction-realizations.md) and the [realization probe](../metal_contraction_vertical/README.md).
- **The composition rule that made a new measurement the only route:** [ADR 0113](../../../docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md), component 3.
- **The pre-registration pattern this follows:** [the thread-execution-width protocol](../../target-profiles/metal-thread-execution-width/PROTOCOL-2026-08-22-standard-profile.md).
- **Work record:** [`calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol`](../../../tickets/calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol.md).

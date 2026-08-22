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

Run on the **coordination host** (Apple M4 Max, macOS 27.0 build `26A5416b`), **not** the bench host, while three compiling worker lanes were live. The one-minute load average was 4.03. That is why this leg reads no wall clock: every check below is a compile-success, cardinality, or bit-identity check, none of which a busy machine can move. The driver prints `NO WALL CLOCK WAS READ` on success for exactly this reason, and `--mode timing` refuses outright unless the quiet-host gate admits.

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

**`make citations` does not reach this directory either, and that was confirmed by perturbation rather than assumed.** Appending a link to a nonexistent file to this README and re-running `make citations` left it **green, exit 0**, with every population count unchanged. The cause is in the checker's own populations: `check-citations.sh` builds its document set with `find docs -type f -name '*.md'` and documents its three populations as `tickets/**`, `docs/**`, and the tracked markdown at the repository root. `spikes/**` is in none of them. So every markdown link in this directory — and in the realization probe's README beside it — is unchecked by any gate, and a link that rots here fails nobody. The 16 local links across this README, the protocol, and the parent `spikes/scheduling/README.md` were therefore resolved by hand on 2026-08-22; a future editor of these files has to do the same, because nothing will do it for them.

**Correction — 2026-08-22 by [`re-run-a-merged-document-s-own-evidence-commands-against-the-merged-tree`](../../../tickets/re-run-a-merged-document-s-own-evidence-commands-against-the-merged-tree.md), read at `6f3c2594`: `make citations` reaches this directory, and the two paragraphs above fail in the direction that matters — they tell a reader a check is absent when it exists.** Retired wording, preserved: *"`spikes/` is outside every gate"*; *"`make citations` does not reach this directory either, and that was confirmed by perturbation rather than assumed"*; and the conclusion drawn from them, *"every markdown link in this directory — and in the realization probe's README beside it — is unchecked by any gate, and a link that rots here fails nobody"*. All three were true when written, at `7fd2f927`. All three were **already false at the commit this README merged into**, because [`decide-whether-the-citation-checker-should-reach-spike-records`](../../../tickets/decide-whether-the-citation-checker-should-reach-spike-records.md) put `spikes/**` into the gate's populations at `04d5eae9`, ahead of this record and in the same batch.

**Re-confirmed by the same perturbation, in the opposite direction.** Appending a markdown link whose target is `./no-such-file-perturbation.md` to this README and running `make citations` at `6f3c2594` now **fails, exit 2**. It prints `FAIL  spikes/scheduling/metal_contraction_tile_width/README.md` followed by `no tracked file or directory at spikes/scheduling/metal_contraction_tile_width/no-such-file-perturbation.md`, and the spike link population rises by exactly one rather than staying unchanged. The perturbation was reverted. The retired paragraph's stated *cause* was accurate and has been superseded by addition rather than by correction: the document set is still built at `check-citations.sh "find docs -type f -name"`, and what changed is that a fourth population was added beside it at `check-citations.sh "find spikes -type f -name"`.

**What is still true, stated so this note does not overcorrect in the other direction.** The coverage is a deliberate split and only one half of it closed. Local markdown links are checked; pinned source citations under `spikes/` are deliberately **not**, for the reason recorded at `check-citations.sh "SPIKE RECORDS: LINKS CHECKED, PINNED CITATIONS DELIBERATELY NOT"` — a spike record pins the base its own record names rather than the tip, so demanding its citations resolve at the tip is unsatisfiable. The exclusion is printed as a number rather than a silence, as `54 pinned citation(s) DECLINED` over `68 live file(s)`. The hazard paragraph's first claim also survives on its narrow reading: `make check` and `make full` still compile, run, and lint nothing here, so a harness that stops compiling still breaks silently and `--mode validate`'s frozen counts remain the only tripwire for *that*. What no longer holds is the link half alone — the local links across this README, the protocol, and the parent `spikes/scheduling/README.md` are gated now, and there are **nineteen** of them at `6f3c2594` rather than the sixteen counted by hand on 2026-08-22. Two of those nineteen are the ticket links this correction itself adds, which is the mechanism worth naming: a dated correction preserves retired wording and adds its own references, so the count **cannot shrink across a repair**. A later reader who finds fewer should suspect the count, not the gate.

## Reproduce

No `make` target reaches `spikes/`. From **this directory**. No third-party package is needed in any mode.

**Correction — 2026-08-22, by the ticket and at the base named in the hazard note above.** Retired wording, preserved: *"No `make` target reaches `spikes/`"*. `make citations` reaches it and resolves every local markdown link in this directory, so a rotted link here now fails the gate. No `make` target **compiles or runs** anything here, which is the half the instruction below depends on and is unchanged.

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

The timing leg runs on the bench host under the frozen commands in the protocol, and only there. It refuses to start unless the quiet-host gate admits, and the gate can be read alone — dispatching nothing, reading no wall clock — with `--mode gate`.

## The quiet-host gate, and both directions of it watched

The gate this lane originally froze required a one-minute load average **below 0.5**, and the bench host's idle one-minute load average is **1.86–2.47**. That gate had no reachable satisfying case: it did not delay the timing run, it foreclosed it. It was replaced before any dispatch was timed, under [the protocol's pre-run amendment](PROTOCOL-2026-08-22-contraction-tile-width.md), by four components that fail closed — mean CPU idle over ten one-second samples at or above 95%, GPU device utilization at or below 5%, one-minute load average at or below a baseline-relative 3.5, and an exclusive measurement lock.

A gate that only ever admits is the defect being removed, reintroduced in the other direction, so each component was broken on the bench host on 2026-08-22 and its own failure text is quoted below. **The subject was perturbed in every case; no assertion was edited.**

**It admits the quiet host.** Read on the bench host with nothing dispatched, quoted in full and unedited:

```text
== quiet-host gate ==
   host Thomass-MacBook-Pro, 11 cores, Apple M3 Pro
  CPU idle, mean of 10 x 1 s    98.65%  (floor 95.0%, min sample 98.10%)  pass
  GPU device utilization              1%  (ceiling 5%)  pass
  load average, one minute         2.33   (ceiling 3.5, recorded idle baseline one-minute 1.86-2.47 over 20+ observations 2026-08-22; retained in-tree at spikes/program-planning/physical-frontier-budget-calibration/results/ as { 2.22 2.39 2.26 } on 2026-08-13 and { 2.18 2.23 2.24 } on 2026-08-14, same host)  pass
  exclusive measurement lock     /tmp/tiler-contraction-tile-width-sweep.lock  held

verdict: ADMIT -- a timing run may start now.
NO WALL CLOCK WAS READ. This mode makes no timing claim of any kind.
```

The refusal blocks below are excerpts from their own runs; where a long constant is elided it is marked `...`, and no line is combined from a different read.

**It refuses competing CPU work.** Four bounded busy loops were started on the bench host and the gate re-read twenty-five seconds later:

```text
  CPU idle, mean of 10 x 1 s    61.17%  (floor 95.0%, min sample 55.50%)  REFUSE
  load average, one minute         2.89   (ceiling 3.5, ...)  pass
REFUSED: CPU idle averaged 61.17% over 10 s, below the 95.0% floor. Work is competing
for CPU, and on a unified-memory device that competes for bandwidth with the dispatch
being timed.
```

**That reading is also the evidence for demoting the load average.** Twenty-five seconds into four cores of competing work the load average was still `2.89`, inside any ceiling that also admits this host's 2.2 baseline — so a gate resting on load alone would have admitted that run. Seventy seconds further into the same load it caught up and the second component fired too:

```text
  CPU idle, mean of 10 x 1 s    51.92%  (floor 95.0%, min sample 49.62%)  REFUSE
  load average, one minute         4.24   (ceiling 3.5, ...)  REFUSE
REFUSED: One-minute load average is 4.24, above the 3.5 ceiling. This ceiling is
relative to this host's recorded idle baseline of ..., not an absolute quiet figure.
```

**It refuses an unreadable probe rather than passing it.** The `Device Utilization %` key was renamed on its way out of `ioreg`, reproducing the failure that left a deferred trigger polling a `system_profiler` data type macOS had renamed and reading *not fired* forever. A pass-through stub on the same `PATH` is the negative control, and it passes — so the refusal is the rename, not the stub:

```text
=== CONTROL: pass-through ioreg stub on PATH ===
  GPU device utilization              0%  (ceiling 5%)  pass

=== PERTURBED: the Device Utilization % key is renamed by the OS ===
  GPU device utilization         UNREADABLE  REFUSE
REFUSED: GPU utilization is unreadable: no `Device Utilization %` field was found on
any IOAccelerator node. The field may have been renamed. An unreadable probe refuses,
because a renamed key and an idle GPU look identical.
```

**It refuses a second measurement session.** A separate process took the lock, and the gate was read while it was held and again after it exited:

```text
  exclusive measurement lock     /tmp/tiler-contraction-tile-width-sweep.lock  REFUSE
REFUSED: Another measurement session holds /tmp/tiler-contraction-tile-width-sweep.lock.
Two sweeps sharing one device measure each other.
...
  exclusive measurement lock     /tmp/tiler-contraction-tile-width-sweep.lock  held
```

**Before the run, quiesce the interactive session.** This host carries a console login whose foreground applications produce episodic bursts of about one core, and the gate refuses during them — correctly. Quit them before the timing leg. Re-reading the gate until it happens to pass selects for the quiet phase of a host that is not quiet, which is exactly the contamination the gate exists to exclude. Nothing here authorizes disabling the system extensions that carry the load-average floor: they are the machine's networking, and that is Tom's decision.

## Traceability

- **Governing pre-registration:** [`PROTOCOL-2026-08-22-contraction-tile-width.md`](PROTOCOL-2026-08-22-contraction-tile-width.md).
- **The record being calibrated, and not repaired:** [First Metal contraction realizations](../../../docs/research/scheduling/first-metal-contraction-realizations.md) and the [realization probe](../metal_contraction_vertical/README.md).
- **The composition rule that made a new measurement the only route:** [ADR 0113](../../../docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md), component 3.
- **The pre-registration pattern this follows:** [the thread-execution-width protocol](../../target-profiles/metal-thread-execution-width/PROTOCOL-2026-08-22-standard-profile.md).
- **Work record:** [`calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol`](../../../tickets/calibrate-the-contraction-tile-width-under-a-beneficiary-named-protocol.md).

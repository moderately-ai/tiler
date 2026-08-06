---
id: measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order
title: Measure whether the Metal compiler preserves the emitted evaluation order
status: in-progress
priority: p2
dependencies: []
related: [derive-the-oracle-for-a-permitted-divergence-candidate, admit-a-refutation-only-derived-bound-conformance-oracle]
scopes: [research/apple-targets]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, apple-targets, conformance, measurement]
claimed_from: todo
assignee: agent-eval-order
lease_expires_at: 1786042476
---
## User-visible outcome

A measured answer to whether a Metal kernel's emitted floating-point evaluation order survives the backend compiler, and a target-profile fact that carries the answer — because today the property is asserted by a flag and declared by nothing.

## Why this exists

**Fact — [the oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) makes the plan's pinned evaluation order the whole basis of qualifying a permitted-divergence candidate.** That basis holds only if the order the artifact emits is the order the device executes.

**Fact — Tiler pins it today by asserting flags, not by consulting a fact.** `MetalNumericalRequirement::NoFloatingPointContraction` renders `-ffp-contract=off` and `SafeMathMode` renders `-fmetal-math-mode=safe` (`crates/tiler-metal/src/record.rs:76-132`); `crates/tiler-metal/src/tests.rs:1311` records the reason — "`-ffp-contract=off` is a defence against the *compiler* contracting a written multiply and add".

**Fact — no target profile declares the property.** `MetalTargetFacts` (`crates/tiler-metal/src/target.rs:755`) has five fields: language, platform, deployment minimum, per-type subnormal arithmetic, buffer binding limit. `CapabilityAxis` (`crates/tiler-compiler/src/target/feasibility.rs:211`) has seven, none about compiler-preserved evaluation order.

**Inference — so under a contract that permits contraction, Tiler would have no ground to keep asserting the flag that supplies its own pin**, and the executed order would become a property of a compiler nothing declares. `NumericalContract::RELAXED_F32` permits contraction and is registered, so the case is reachable rather than hypothetical.

## The bounded experiment

- **Inputs:** a kernel whose written order and a legal alternative order differ in bits — the four-operand set at `0x3f400000, 0x3e800000, 0x33400000, 0x33000000` already separates a serial fold from a two-by-two split by one ULP and is the natural seed. Compiled at each combination of `-fmetal-math-mode` and `-ffp-contract` the toolchain accepts, including the combinations Tiler does *not* assert.
- **Outputs and metric:** the executed result bits per combination, against the reference value of the written order. The metric is agreement or disagreement, not a magnitude.
- **What it must separate:** whether a disagreement is contraction (the existing golden-compilation work already probes this at `crates/tiler-metal/src/golden_compilation.rs:584`) or reassociation, which is the new question. Design the case so the two are distinguishable, and record it if they cannot be.
- **Unsupported cases and stop condition:** if no flag combination the toolchain accepts reorders the written sequence, the honest result is that this toolchain on this row does not, which is a bounded observation and not a portable guarantee — record it as such and stop rather than searching for a stronger claim.

## What it decides

A `Preserved` answer makes the pinned-order oracle sound on this row and supplies the fact a target profile would declare. A `NotPreserved` answer makes refusal class 3 of the derivation permanent for the affected contracts and fires [`admit-a-refutation-only-derived-bound-conformance-oracle`](admit-a-refutation-only-derived-bound-conformance-oracle.md)'s first clause. **Neither outcome is presumed here.**

## Explicit non-goals

Adding a target-profile field, which is a public boundary and a separate ticket; any performance claim; any change to which flags Tiler asserts.

## Closes when

Every accepted flag combination is measured on a named host with its exact toolchain version, the contraction and reassociation causes are separated or the inseparability is recorded, and the boundary states what the result does not generalize to.

## Graph maintenance

Filed by [the permitted-divergence oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) as the one refusal class with no closer in the graph.

## Outcome

**Measured 2026-08-06. The answer is `NotPreserved` under `relaxed` and `fast`, and `Preserved` under `safe` in every cell measured.** Neither outcome was presumed and the stop condition was not reached: a flag combination the toolchain accepts does reorder the written sequence, so this is a positive result rather than the bounded no-reorder observation the ticket also admitted.

**The measurement.** [`spikes/apple-targets/evaluation-order-probe/`](../spikes/apple-targets/evaluation-order-probe/README.md), retained record [`results/2026-08-06-evaluation-order-macos27-msl4-metalfe-32023.921/record.tsv`](../spikes/apple-targets/evaluation-order-probe/results/2026-08-06-evaluation-order-macos27-msl4-metalfe-32023.921/record.tsv), schema `tiler.apple-evaluation-order/v1`, 502 rows from repository revision `2c0c8501`. Three kernels over one twelve-element operand buffer: `serial_fold4` computing `((a+b)+c)+d`, `split_fold4` computing `(a+b)+(c+d)`, and `contraction_control` computing `a*b+c`. The discriminating quad is the ticket's own seed `3f400000 3e800000 33400000 33000000`, where the serial fold gives `3f800000` and the split gives `3f800001`. Seventy-two cases in one host invocation: 54 offline (3 `-fmetal-math-mode` × 3 `-ffp-contract` × `-O0`/`-O2`, `-std=metal4.0`, `air64-apple-macos26.0`) and 18 runtime (3 `mathMode` × `Default`/`Size`, `MTLLanguageVersion4_0`), which is every combination either compiler accepts.

**Result.** `serial_fold4` returned the order its source names in all 24 cases. `split_fold4` returned a different legal order in 6: offline `-O2` under `relaxed` and under `fast` with `-ffp-contract=fast`, and every runtime cell under `Relaxed` and `Fast` at both optimization levels. In each it returned `3f800000` where its source names `3f800001`. Every `safe` cell on both paths, every `-O0` cell, and `relaxed`/`fast` at `-ffp-contract=off` and `=on` returned the source's own value.

**Attribution, from the emitted module rather than from the bits.** In the two diverged offline cells the written `t0=a0+a1;t1=a2+a3;t2=t0+t1` is emitted as `t0=a1+a0;t1=t0+a2;t2=t1+a3` — the front end re-serializes the split and swaps the first pair — so the rewrite is in the LLVM IR and not in the AIR-to-ISA stage below it. `serial_fold4` and `split_fold4` emit the **identical** tree there: two programs pinning different orders compile to one program. An opcode count could not have seen it; `float_operations` is `fadd;fadd;fadd` in all 36 offline fold cases.

**Contraction and reassociation are separated, by construction and by measurement.** The fold kernels contain adds and nothing else, so `-ffp-contract` has no multiply/add pair to act on — measured, not asserted: no `fmul`, no `llvm.fma`, no `air.fma.f32` in any of the 36 offline fold cases, and a case whose emitted list is not the one its kernel declares makes the producer refuse. `contraction_control` fuses in 10 of its 24 cases in the same run (every offline `-ffp-contract=fast` cell, reproducing finding 6; every runtime `Relaxed`/`Fast` cell, reproducing finding 30), so the contraction axis is live beside them. The separation is established rather than recorded as unavailable.

**One cell that must not be read as a defence.** Offline, the reordering fires only where the emitted licences carry both `reassoc` and `contract`, and not under `reassoc` without `contract`. The authorizing licence is `reassoc`, which `-ffp-contract=off` does not withdraw, so the coincidence is a pass-pipeline observation on one build and not a mechanism a contract may lean on. On the runtime path there is no contraction property at all and only `mathMode = Safe` keeps the order.

**Named host and toolchain.** Apple M4 Max, `MTLGPUFamilyApple9` supported, registry ID `4294968656`; macOS 27.0 build 26A5388g, arm64; Xcode 27.0 build 27A5228h at `/Applications/Xcode-beta.app/Contents/Developer`; SDK `macosx` 27.0 build 26A5388f; offline `Apple metal version 32023.921 (metalfe-32023.921)`; runtime `GPUCompiler.framework` images recorded by path with no build string recovered. **This is not the toolchain row findings 1 to 33 of the numerical-behaviour record share** (Xcode 26.6 build 17F113, offline `metalfe-32023.883`) — `xcode-select` moved on this host before this ticket, as the elementary-identity probe recorded on 2026-08-05. Changing it back would change the evidence environment and needs Tom's authorization, so it was not touched; the boundary is recorded instead.

**Commands and verdicts**, all from `spikes/apple-targets/evaluation-order-probe/`:

```text
python3 order_probe.py                               # exit 0, 502 rows
python3 order_probe.py --retain                      # exit 0, wrote the retained record
python3 order_probe.py --perturb written-order       # exit 0: serial_fold4 diverged in 18 cases where the unperturbed run reports 0
python3 order_probe.py --perturb dead-contraction-axis  # exit 1: "the contraction control never fused, so the contraction axis is not live in this run"
python3 order_probe.py --perturb deleted-arithmetic  # exit 1: "emitted none, where this kernel declares fadd;fadd;fadd"
```

Two consecutive unperturbed runs differ in exactly one row of 502, `environment.date_utc`; the three retained sources are byte-identical. The unperturbed run returned 0 before and after each perturbation.

**What it does not generalize to.** One offline compiler build, one runtime compiler, one OS build, one SDK, one language standard, one deployment target, and not the row the rest of the numerical record shares. `MacOs` and one Apple9 GPU only. `f32` only — `f16` and `bf16` evaluation order is `Unknown`, not inherited. Four contributors within one thread over a device buffer: not a reduction over a threadgroup, a subgroup, or a multi-round cooperative tile, and no longer chain. `-O1`, `-O3`, `-Os` unswept. The bit metric separates the written serial order from two of the seed's four legal alternatives and not from the other two, which is the right metric for an oracle that compares bitwise against the pinned order but is not a tree-identity claim; only the offline `fold_shape` reading makes one. No timing.

**What this decides, for the coordinator to route.** Refusal class 3 of [the oracle derivation](../docs/research/reference/permitted-divergence-oracle.md) is reachable with a measured population, which fires [`admit-a-refutation-only-derived-bound-conformance-oracle`](admit-a-refutation-only-derived-bound-conformance-oracle.md)'s first clause. Three edits follow that this ticket's scope excludes and that are reported rather than made:

1. `docs/research/reference/permitted-divergence-oracle.md` (`research/reference`) still describes this experiment as filed rather than measured, in Part 7 item 5 and in the four-outcome roll-up.
2. A target profile field carrying the property — an explicit non-goal here, a public boundary, and Tom's under ADR 0075. `MetalTargetFacts` and `CapabilityAxis` still declare nothing about compiler-preserved evaluation order, so a profile admitting `relaxed` or `fast` cannot record that it does not honour a pinned order.
3. `spikes/README.md` and `docs/research/README.md` (`contracts/navigation`) need catalog rows for the new spike; both are outside this ticket's scopes.

**Scope confirmation.** The diff touches `docs/research/apple-targets/numerical-behaviour.md`, `spikes/apple-targets/README.md`, `spikes/apple-targets/evaluation-order-probe/**`, and this ticket — `research/apple-targets` and `project/tickets` only. No `crates/` file changed.

---
schema: "tiler-doc/v1"
id: "tiler.spike.apple-targets.evaluation-order"
kind: "experiment"
title: "Metal emitted-evaluation-order probe"
topics: ["apple-targets", "metal", "numerics", "reassociation", "contraction", "math-modes", "runtime-compilation", "conformance"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.apple-targets.numerical-behaviour", "tiler.research.reference.permitted-divergence-oracle"]
entrypoints: ["spikes/apple-targets/evaluation-order-probe/order_probe.py"]
last_verified: "2026-08-06"
ticket: "measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order"
---

# Metal emitted-evaluation-order probe

## The named question

**Is the floating-point evaluation order a Metal kernel emits the order the device executes?** [The permitted-divergence oracle derivation](../../../docs/research/reference/permitted-divergence-oracle.md) makes the plan's pinned evaluation order the whole basis of qualifying a candidate under a contract that permits reassociation: the oracle evaluates the *one* realization the physical plan declared and compares bitwise. That basis holds only if the emitted order survives the backend compiler, and the derivation names the gap as refusal class 3 — Tiler pins the order today by asserting `-fmetal-math-mode=safe` and `-ffp-contract=off`, and no target profile declares the property those flags are standing in for.

## The answer, in one line

**The order does not survive `relaxed` or `fast`, and it does survive `safe` in every cell measured on this row.** A written two-by-two split `(a+b)+(c+d)` is re-emitted as the left-deep chain `((b+a)+c)+d` and the device returns the serial value, one ULP from the value the written order names. The written left-deep chain is returned as written everywhere, which is a fact about that chain being the form this compiler canonicalizes *to* rather than a fact about order preservation.

## What it measures

Three kernels in the Metal emitter's per-statement output shape, over one twelve-element operand buffer of three quads. A lane folds the quad it belongs to, so every lane of every case is written and the dispatch host's unwritten-element sentinel stays meaningful.

| kernel | body | why |
| --- | --- | --- |
| `serial_fold4` | `((a+b)+c)+d` | the written order a serial reduction's plan would pin |
| `split_fold4` | `(a+b)+(c+d)` | the legal alternative a two-by-two partition would pin, and the perturbation that shows the operand set discriminates |
| `contraction_control` | `a*b+c` | the positive control that keeps the contraction axis live in the same run |

| quad | operands | role |
| --- | --- | --- |
| `seed` | `3f400000 3e800000 33400000 33000000` | `0.75`, `0.25`, `3·2⁻²⁶`, `2⁻²⁵` — the serial fold gives `3f800000` and the two-by-two split gives `3f800001` |
| `witness` | `3f800000 40000000 40800000 41000000` | powers of two: every grouping gives `41700000`, which differs from all four operands, so it reports that the adds ran without reporting anything about their order |
| `contraction` | `3eb97ef9 3fc00000 3f800000 3f800000` | the operand, scale, and bias whose separately rounded `3fc58f9e` and fused `3fc58f9d` differ |

The seed is the set [the oracle derivation's Part 6](../../../docs/research/reference/permitted-divergence-oracle.md) works through, and its arithmetic reproduces by hand: `0.75 + 0.25 = 1.0` exactly, `ulp(1.0) = 2⁻²³`, so the two tail contributors are `0.375` and `0.25` ulp. A serial fold adds them one at a time and each rounds back to `1.0`; a two-by-two split adds them to each other first — `0.625` ulp, exact — and the last add rounds up.

The matrix is every combination the two compilers accept. Offline: three `-fmetal-math-mode` values × three `-ffp-contract` values × `-O0` and `-O2`, at `-std=metal4.0` for `air64-apple-macos26.0`, which is **54 cases**. Runtime: three `mathMode` values × both `MTLLibraryOptimizationLevel` values at `MTLLanguageVersion4_0`, which is **18 cases**. Every case is dispatched in one host invocation from the byte-identical generated source.

## The result

`summary.*` rows in the retained record, and every one reproduces from `case.*`:

| kernel | path | as written | diverged |
| --- | ---: | ---: | ---: |
| `serial_fold4` | offline | 18 | 0 |
| `serial_fold4` | runtime | 6 | 0 |
| `split_fold4` | offline | 16 | **2** |
| `split_fold4` | runtime | 2 | **4** |
| `contraction_control` | offline | 12 unfused | 6 fused |
| `contraction_control` | runtime | 2 unfused | 4 fused |

The six diverged fold cases are exactly the ones where the math mode is `relaxed` or `fast` and the contraction licence is granted:

| path | selection | `split_fold4` returned | emitted add tree |
| --- | --- | --- | --- |
| offline `-O2` | `relaxed`, `-ffp-contract=fast` | `3f800000` where the source names `3f800001` | `t0=a1+a0;t1=t0+a2;t2=t1+a3` |
| offline `-O2` | `fast`, `-ffp-contract=fast` | `3f800000` | `t0=a1+a0;t1=t0+a2;t2=t1+a3` |
| runtime | `Relaxed`, `Default` and `Size` | `3f800000` | not readable on this path |
| runtime | `Fast`, `Default` and `Size` | `3f800000` | not readable on this path |

Every other cell — all nine `safe` offline cells, both `safe` runtime cells, every `-O0` cell, and `relaxed`/`fast` at `-ffp-contract=off` and `=on` — returned the value its own source names.

## How the offline half attributes the change, rather than inferring it

The bits say the device returned a different value; they do not say which stage produced it. The offline half reads the emitted module and says. `case.*.fold_shape` records the add tree with its leaves labelled by the operand slot they were loaded from, so a preserved split reads `t0=a0+a1;t1=a2+a3;t2=t0+t1` and a re-serialization reads `t0=a1+a0;t1=t0+a2;t2=t1+a3`. **The rewrite is in the LLVM IR the front end emits**, so it is the offline compiler and not the AIR-to-ISA stage below it — which matters because that lower stage is the one finding 7 of [the numerical-behaviour record](../../../docs/research/apple-targets/numerical-behaviour.md) measured deleting operations invisibly.

An opcode count could not have found this. Rearranging three adds leaves three adds, and `case.*.float_operations` is `fadd;fadd;fadd` in all 36 offline fold cases. The tree is a separate reading for that reason.

The two kernels **converge on one module** in the two diverged offline cells: `serial_fold4` and `split_fold4` emit the byte-identical tree there. Two source programs that pin different evaluation orders compile to one program, so the pin is not weakened in those cells — it is erased.

`case.*.fast_math_licences` records what the operations actually carry, because the driver flag and the granted licence are different things. The reordering fires only where the set contains **both** `reassoc` and `contract` — `reassoc nsz arcp contract afn` under `relaxed`, and the umbrella `fast` — and does not fire in the cells carrying `reassoc nsz arcp afn` or `reassoc nnan ninf nsz arcp afn`, which are `reassoc` without `contract`. **This does not make `-ffp-contract=off` a defence against reassociation.** The licence that authorizes the rewrite is `reassoc`, `-ffp-contract=off` does not withdraw it, and a pass pipeline that happened not to spend it here can spend it in the next compiler build. It is a measured cell, not a mechanism.

## Where reassociation is separated from contraction

The ticket requires the two causes distinguished, and this is where.

- **Structurally.** Both fold kernels contain adds and nothing else, so contraction — which fuses a multiply into an add — has no pair to act on in them. That is not asserted: `case.*.float_operations` is `fadd;fadd;fadd` for every one of the 36 offline fold cases, with no `fmul`, no `llvm.fma`, and no `air.fma.f32`, and a case whose emitted list is not the one its kernel declares makes the producer refuse before any verdict is read.
- **By a live control.** `contraction_control` is dispatched in the same run and the same matrix, and it fuses in 10 of its 24 cases — every offline `-ffp-contract=fast` cell including under `safe`, reproducing finding 6 of the numerical-behaviour record on a newer compiler build, and every runtime `Relaxed` and `Fast` cell, reproducing finding 30. If it fused nowhere the contraction axis would not be live, the fold kernels' behaviour under that axis would attribute to nothing, and the producer publishes no record at all.

So the separation is established rather than recorded as unavailable: a fold divergence cannot be contraction, and the axis that could have confounded it is measured working beside it.

The runtime half carries the structural argument and not the module reading — `newLibraryWithSource:options:` returns an opaque library, so `float_operations` and `fold_shape` are **absent** rather than empty for a runtime case, the distinction the numerical-behaviour record encodes as `None` versus `()`. That the runtime divergence is a reassociation is an Inference from the source containing no multiply plus the offline compiler exhibiting exactly this rewrite; it is not a runtime module reading, because there is none to take.

## The guards, and the runs in which each was watched refusing

Three guards, three perturbations, each run on 2026-08-06 on the row below with the unperturbed run returning 0 before and after.

| guard | perturbation | what happened |
| --- | --- | --- |
| the order metric can report a reordering | `--perturb written-order` emits the split chain under `serial_fold4`'s name, leaving its reference the serial value | `serial_fold4` diverged in **18** cases where the unperturbed run reports 0, exit 0 |
| the separation rests on a live contraction axis | `--perturb dead-contraction-axis` compiles the control under `-ffp-contract=off` offline and `mathMode=Safe` at runtime, the only selections either compiler exposes that suppress fusion | `the contraction control never fused, so the contraction axis is not live in this run and the fold kernels' invariance under it attributes to nothing`, exit 1 |
| a returned pattern is inadmissible unless the arithmetic ran | `--perturb deleted-arithmetic` replaces both fold kernels with pass-throughs that still declare three adds | `offline.serial_fold4.fast.O0.contract-fast: emitted none, where this kernel declares fadd;fadd;fadd`, exit 1 |

The second and third are refusals, so a run that publishes under them is the perturbation failing to fire and the producer says so and exits nonzero. Two further refusals are in the producer and were not reachable by a perturbation this probe runs: a quad whose four lanes disagree, and an observed value that is neither the written order's nor any value the freedom under test admits.

The witness quad is held to its order-independent value in every case, and the host seeds its output with a pattern the producer proves no reference value equals, so an unwritten lane cannot read as a measured zero.

## Determinism

Two consecutive runs from the same tree differ in exactly one row of 502, `environment.date_utc`. The three retained sources are byte-identical between them.

## The environment this bounds

| | |
| --- | --- |
| host | Apple M4 Max, `MTLGPUFamilyApple9` supported, registry ID `4294968656` |
| OS | macOS 27.0 build 26A5388g, arm64 |
| Xcode | 27.0 build 27A5228h at `/Applications/Xcode-beta.app/Contents/Developer` |
| SDK | `macosx` 27.0 build 26A5388f |
| offline compiler | `Apple metal version 32023.921 (metalfe-32023.921)`, from the `MetalToolchain` asset the record names in full |
| runtime compiler | the `GPUCompiler.framework` images loaded into the dispatch process, recorded by path; no build string was recovered |
| language and target | `-std=metal4.0`, `-target air64-apple-macos26.0`; runtime `MTLLanguageVersion4_0` |

**This is not the toolchain row findings 1 to 33 of the numerical-behaviour record are measured on.** That row is Xcode 26.6 build 17F113 with an offline `metalfe-32023.883`; this host now resolves Xcode 27.0 and an offline `metalfe-32023.921`. So this probe's rows and that record's rows are **not rows of one table**, and a difference between them is not evidence of drift until one is re-run on the other's toolchain. The offline build string here coincides with the build that record names as the macOS *runtime* compiler, and a coincident build string is not the same artifact reached by the same route.

The registry ID `4294968656` is a third value for the same named M4 Max, after the `4294968621` and `4294968452` the numerical-behaviour record retains. It is a within-run correlation handle and identifies no hardware across records, exactly as that record's environment section establishes from the SDK header.

## What this does not establish

- **Nothing about another toolchain.** One offline compiler build, one runtime compiler, one OS build, one SDK, one language standard, one deployment target. Finding 8's standing reason to keep re-measuring applies with full force: the runtime compiler ships with the OS and moves independently of the offline one.
- **Nothing about another GPU, family, or artifact family.** `MacOs` only, one Apple9 device. No iOS device, no iOS Simulator, no non-Apple Metal implementation.
- **Nothing about another dtype.** `f32` only. Findings 21 and 24 of the numerical-behaviour record established that a licence measured at one width predicts nothing at another, and findings 28 to 30 had to measure contraction at `f16` and `bf16` separately for exactly that reason. `f16` and `bf16` evaluation order is `Unknown` here.
- **Nothing about a reduction.** This is reassociation exposed within one thread over four contributors in a device buffer, which is the licence a reduction would be subject to. It is not a reduction over a threadgroup, a subgroup, or a multi-round cooperative tile, and it is not a chain longer than four.
- **Nothing about `-O1`, `-O3`, or `-Os`.** Finding 19 measures those behaving like `-O2` for its own kernels; this probe did not re-establish it for these.
- **Not an exhaustive statement about the four alternatives.** The seed's five order-preserving groupings produce two values, so the returned bits separate the written serial order from two of its four alternatives and not from the other two. That is the right metric for the question the oracle asks — it compares bitwise against the pinned order's value, so an alternative that agrees bitwise is not a divergence the oracle can or should catch — but it is not a claim that the tree was identical. The offline `fold_shape` reading is what makes a tree claim, and only on the offline path.
- **Nothing about a `#pragma` defence.** Finding 33 measures a source-level contraction pragma surviving to the runtime compiler; whether `#pragma clang fp reassociate(off)` or any spelling exists and survives for *reassociation* was not asked here.
- **No timing.** No performance claim of any kind is made or measured.

## Reproduce

From **this directory**, on a host with an Apple toolchain and a Metal device (no `make` target reaches `spikes/`, and nothing here is wired into the repository gate):

```sh
python3 order_probe.py                              # print the record to stdout
python3 order_probe.py --retain                     # write results/<date>-<identity>/
python3 order_probe.py --perturb written-order      # the metric's failure proof
python3 order_probe.py --perturb dead-contraction-axis
python3 order_probe.py --perturb deleted-arithmetic
```

Standard library only; no dependency to pin and none claimed. It reads the compiler and device already on the host, installs nothing, and writes only inside this directory.

## Retained record

[`results/2026-08-06-evaluation-order-macos27-msl4-metalfe-32023.921/`](results/2026-08-06-evaluation-order-macos27-msl4-metalfe-32023.921) holds `record.tsv` — schema `tiler.apple-evaluation-order/v1` — and the three exact generated kernel sources. The record pins its producer and the dispatch host by SHA-256 and names the repository revision it ran from, so a row is attributable to the exact source that produced it. It is a positive claim that outlives its producer: only re-running detects drift from it.

**`probe.repository_base_revision` is `2c0c8501`, which is where the working tree came from and not a tree containing this producer** — the producer and the record land together in the commit after it. That is the same producer-generation gap the numerical-behaviour record documents for its own retained results, and the digests are what close it: `probe.producer_sha256` and `probe.host_source_sha256` identify the exact bytes that ran, and both match the files checked in beside this record.

## Why this reuses the numerical probe's dispatch host

`numerical_probe_host.m` is taken **unmodified** and built from this producer, for the reason the [contraction-pragma runtime probe](../contraction-pragma-runtime-probe/README.md) states: modifying it would move `probe.host_source_sha256` in every retained numerical record beside it. This probe shares that host, the host row's retention conventions, and nothing else — its kernels, operand vector, schema, verdict vocabulary, and result directory are its own.

## Traceability

- **The question and what it decides:** [the permitted-divergence oracle](../../../docs/research/reference/permitted-divergence-oracle.md), refusal class 3, and its Part 6 worked example which supplies the operand set.
- **The record this finding joins:** [Apple GPU numerical behaviour](../../../docs/research/apple-targets/numerical-behaviour.md), finding 34. Findings 17 and 31 there are its nearest neighbours and neither answers this: finding 17 measures a *written serial* chain over immediates being reassociated under `relaxed` and `fast`, and finding 31 measures contributor permutation. Neither asks whether a written *split* survives, which is the grouping a parallel reduction's plan pins.
- **The permission vocabulary:** [ADR 0014](../../../docs/decisions/0014-reassociation-vs-permutation.md) keeps reassociation and contributor permutation separate, and [ADR 0015](../../../docs/decisions/0015-fma-vs-contraction.md) keeps contraction separate from both. The leaf swap this probe records in the two re-serialized cells is a permutation observation in the emitted IR; over a two-operand add it is unobservable in bits, and nothing here measures a permutation with a value consequence.
- **Work record:** [`measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order`](../../../tickets/measure-whether-the-metal-compiler-preserves-the-emitted-evaluation-order.md).

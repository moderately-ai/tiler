---
id: measure-whether-a-targets-exponential-is-exact-at-zero
title: Measure whether a target's exponential is exact at zero
status: in-progress
priority: p3
dependencies: []
related: [derive-the-tree-fold-form-of-the-online-softmax-rescaling-bound, expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate]
scopes: [research/numerics, research/apple-targets, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, accuracy, transcendentals, measurement, trigger-fired]
claimed_from: todo
assignee: sol-exp-zero
lease_expires_at: 1786423431
---
## User-visible outcome

Whether a target's `exp` returns exactly `1.0` at a zero argument is a measured target fact rather than an assumption, so the online-softmax rescaling bound can drop the elementary term it currently charges on the winning side of every merge.

## Why this exists

**Fact.** The merge operator `(m1,d1) + (m2,d2) = (max, d1*exp(m1-max) + d2*exp(m2-max))` evaluates `exp(0)` on whichever side holds the maximum. [The tree-fold record](../docs/research/numerics/tree-fold-online-softmax-bound.md) charges `eps_exp` and a rounding for it, because nothing in the target vocabulary says the result is exact.

**Fact.** Metal's Table 8.1 states `exp <= 4 ulp` under Apple's own definition, relayed in [the Metal elementary-function accuracy record](../docs/research/numerics/metal-elementary-function-accuracy.md). A 4-ulp bound at `1.0` does **not** imply exactness at zero, so the conservative reading is currently required rather than merely chosen.

**Inference.** If the result is exact, every path step whose rescale argument is zero (winning side of a merge, or a leaf already at the running max) drops its `eps_exp` charge and its rescale multiply is exact. That does not license halving `E`: the published price is path-based with `E = 1 + D`, and under a strict max jump at every merge the deepest always-losing path may still pay a non-zero rescale at every level, so exactness of `exp(0)` can drop far less than half of that path's charges. The conservative unsharpened first-order price remains `D * (u + eps_exp)`; any sharpened form must be re-derived from the path counting rather than obtained by slogan. The saving is therefore a bounded constant improvement on an already-small quantity — which is why this is a bounded measurement rather than a research wave.

## What this ticket must produce

- One measurement per honoured target row: the exact bits `exp(+0.0)` and `exp(-0.0)` return on the device through an explicitly runtime-compiled source kernel. This is bounded evidence about the OS runtime compiler, not about the offline compiler the production AOT profile names; the kernel and compile options mirror the production `precise::exp` spelling and precise/safe selection so the observation reaches the governed elementary-function realization without attributing it to the offline toolchain.
- The result recorded against the target row that claims it, per the discipline that a device measurement belongs to the host whose hardware row it claims.
- The sharpened bound stated in the tree-fold record if and only if a target answers exactly, with the unmeasured case staying conservative rather than assumed.

## Non-goals

Widening the transcendental accuracy contract; measuring `exp` anywhere but at zero; changing the bound's form.

## Trigger

The first target profile that declares a numeric elementary accuracy a parametric bound can instantiate from — because until a bound is instantiated on a real target, sharpening its constant changes no answer.

## Trigger check log

- 2026-08-06 — not fired. No target authority yields a numeric `eps_exp`; [`expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate`](expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate.md) is the ticket that would change that and is open. Reproduce with `tkt show expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate`.
- 2026-08-09 — **fired.** The related ticket `expose-the-numeric-elementary-accuracy-a-parametric-bound-can-instantiate` is `done`. `elementary_relative_accuracy` now obtains an exact rational from the governed target/requirement authority, including `24u` for the registered softmax and SiLU requirements and a typed magnitude domain. A real target row can therefore instantiate the parametric quantity this zero-point measurement sharpens. The measurement is no longer speculative and moves to `todo`; `research/apple-targets` is added because the required device result and environment row belong in the Apple target record, not only in the generic numerical derivation.

## Outcome

**Fact — source audit and repair.** Both stated Facts verify: the tree merge evaluates `exp(0)` on a side whose subtree maximum wins, and Metal Shading Language 4.1 Table 8.1 guarantees only `exp <= 4 ulp`, which does not imply exactness at `1.0`. The `24u` trigger verifies in `elementary_relative_accuracy`. The original deliverable rationale was imprecise in calling runtime compilation the production fold route: Tiler's production profile is AOT and supplies no source at runtime. The deliverable above is repaired to classify this as bounded OS runtime-compiler evidence using the production spelling and selection, not as evidence about the offline AOT compiler; the ticket's purpose and authority do not change.

**Fact — measured population.** There is one current authoritative Apple hardware row, `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1`, constructed by `BoundMetalCompileDeclaration::first_macos_apple9`. The governed elementary `exp` requirement is its F32 realization, not a second Apple target row. The result is recorded once against that execution environment while keeping the target-neutral governed profile and the runtime compiler as separate authorities.

**Measurement — exact device bits.** The retained [runtime probe](../spikes/apple-targets/exp-at-zero-runtime-probe/README.md) compiled `precise::exp(input[tid])` through `MTLDevice.newLibraryWithSource` with applied `math=safe,fpfun=precise,lang=4.0,opt=default`, dispatched buffer-supplied input bits `00000000` and `80000000`, required terminal command-buffer success, and returned `3f800000` for both. The row is Apple M4 Max / Apple9 / arm64 macOS 27.0 build 26A5388g; the host build row is Xcode 27.0 build 27A5228h, SDK 27.0 build 26A5388f, Apple clang 21.0.0; the source runtime compiler is `metalfe-32023.921`, recovered from the serialized binary archive. The production profile's offline Xcode 26.6 / `metalfe-32023.883` row is retained separately and remains unmeasured at these inputs.

**Inference — sharpened count.** On that measured runtime row a zero-argument leaf drops its elementary factor, and a zero-argument merge rescale drops both its elementary factor and the multiply rounding because the factor is exact `1.0`. This does not halve `E`: a non-maximum contributor may lose at every merge. The tree record now derives `E0(j) = q_b(j) + L(j)` and `N0(j) = h_intra(j) + D_j + L(j)`, sharpens the matched baseline too, and retains the published conservative form for deepest non-singleton always-losing paths and every unmeasured target/compiler. A singleton-leaf schedule guarantees one elementary-factor saving; realized winning steps may save more.

**Fact — failure reachability and retained custody.** The unperturbed record validates. Perturbing the kernel bytes is rejected by `probe.kernel_sha256 mismatch`; changing the negative-zero input row is rejected by `measurement.input.1 mismatch`; changing a result to `3f800001` is rejected by `measurement.result.0 mismatch`. The retained record binds the kernel, shared host, harness, manifest, invocation, raw host output, exact environment, and compiler route by digest. No CPU path or timing was run.

**Review correction — 2026-08-11, exact candidate `9fd427e9741f31668471ca6626d1698f10183b7e`.** Independent review found that the validator treated `environment.runtime_compiler.version_source` and the retained host build fields as presence-only: relabelling the source `offline-metal` or changing the Xcode version to `0.0` still passed. The validator now requires the exact `serialized-MTLBinaryArchive` source label and exact retained Xcode/SDK/clang/Metal version/build row; host tool paths remain nonempty provenance rather than fixed-row authority. Both new subject-only perturbations are included alongside the kernel, input, and result demonstrations. The producer still scans the runtime archive before validating and atomically publishing the staged result, but retains only the recovered compiler text, not the raw archive; this record therefore makes no archive-replay claim.

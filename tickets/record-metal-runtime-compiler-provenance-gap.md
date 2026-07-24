---
id: record-metal-runtime-compiler-provenance-gap
title: Record that Metal artifact toolchain provenance names only the offline compiler
status: done
priority: p2
dependencies: []
related: [probe-metal-runtime-compilation-numerics, declare-metal-numerical-honourability, repoint-adr-0076-evidence-at-the-numerical-record, prototype-metal-bundle-assembly]
scopes: [contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, metal, numerics, provenance]
---
[probe-metal-runtime-compilation-numerics](probe-metal-runtime-compilation-numerics.md) measured something the Metal contracts do not yet say, and it is about identity rather than about numbers.

**Measurement — a Metal host resolves two compilers, not one.** On the recorded row (macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113, Apple M4 Max), `xcrun --sdk macosx metal --version` reports `Apple metal version 32023.883 (metalfe-32023.883)`, resolved from the Xcode MetalToolchain asset. A library compiled in process by `newLibraryWithSource:options:`, serialized through an `MTLBinaryArchive`, embeds `Apple metal version 32023.921 (metalfe-32023.921)`; that compiler is `/System/Library/PrivateFrameworks/MTLCompiler.framework`, `CFBundleVersion` 382.5, which ships with the OS rather than with Xcode. Both are recorded in [`results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv`](../spikes/apple-targets/results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv) as `environment.metal_version` and `environment.runtime_compiler`, and the gate compares against both.

**Inference — an artifact's toolchain provenance is therefore not the provenance of everything that runs.** Tiler's Metal artifact identity records the offline `metal` and `metallib` builds. That identifies the compiler for an AOT-compiled kernel and identifies nothing about a kernel compiled through `newLibraryWithSource:`, whose compiler moves with the OS and can change without the artifact changing at all. The [artifact-compatibility record](../docs/research/apple-targets/artifact-compatibility.md) already establishes that "Xcode 26.6" is an insufficient toolchain identity; this is a second and independent insufficiency on a different axis.

**Fact — the two paths currently agree, which is why this is a contract-wording ticket and not a correctness one.** All 40 measured runtime cases return bit patterns identical to their offline counterparts, so nothing is broken today. The contracts should say that this is a bounded measurement on one host row that happens to hold between two specific compiler builds, not a property of Metal, and that a delivered numerical realization read off an offline build is not thereby true of a runtime-compiled kernel.

## The work

Record in `docs/backends/metal.md` — and in `docs/artifact-abi.md` if that is where toolchain provenance is actually sited, which is worth checking rather than assuming — that:

- the recorded toolchain provenance identifies the offline compiler only;
- a runtime-compiled kernel is compiled by a separate, separately versioned compiler that is not part of that provenance, and on the measured host is a different build;
- the measured agreement between the two is a bounded host-row measurement, with the pointer to `tiler.research.apple-targets.numerical-behaviour` findings 8 and 9.

Decide and state whether Tiler's Metal story admits a runtime-compiled kernel at all. If the AOT boundary is meant to exclude it entirely, that is a stronger and simpler answer than a provenance caveat, and the contract should say so — but say it explicitly rather than leaving the case unaddressed, because Candle's own command stream reaches `newLibraryWithSource:` and `prototype-metal-bundle-assembly` will have to answer the same question.

Do **not** widen ADR 0076's conclusion here. Finding 9 supports it and finding 8 strengthens its central argument. If the ADR gains anything it is one sentence — that a versioned target numerical fact must identify which compiler the realization was measured on — and `repoint-adr-0076-evidence-at-the-numerical-record` holds `contracts/decisions`.

## Scope note

`contracts/artifacts` covers both `docs/backends/**` and `docs/artifact-abi.md`. `declare-metal-numerical-honourability` also holds `contracts/artifacts` and owns the *subnormal* half of the `docs/backends/metal.md` numerics text; this ticket owns the *provenance* half only. Sequence them rather than editing the same paragraph twice.

## Outcome

`docs/backends/metal.md` gained a new section, "Compiler provenance and the runtime compiler", between "Numerical compiler realization" and "Expansion-time offline compilation", and `tiler.research.apple-targets.numerical-behaviour` was added to its `evidence` frontmatter, which the research record already reciprocated through `informs`. `docs/artifact-abi.md` gained one paragraph in "Artifact identity". Nothing else was touched.

### The question the ticket asked me to settle: Tiler does not admit a runtime-compiled kernel, and that was not mine to decide

The ticket asks whether Tiler's Metal story admits a runtime-compiled kernel at all, and says the stronger answer is to exclude it. It is excluded, and it was excluded before this finding existed, by two accepted decisions and a stated non-goal. [ADR 0002](../docs/decisions/0002-aot-metal-artifacts.md) decides that the runtime "creates and caches pipeline objects from compiled artifacts but does not compile MSL source". [ADR 0043](../docs/decisions/0043-use-typed-phased-target-feasibility.md) restates it as a standing prohibition: "this does not authorize runtime source compilation: the initial product still forbids it, while a backend may declare required device translation of an AOT target-IR artifact such as a metallib." `docs/vision.md` lists runtime source compilation among the first implementation's non-goals, and `AGENTS.md` forbids runtime source JIT in the macro frontend contract. So this is not a live product judgement about the supported deployment surface and was not presented to Tom as one. What was genuinely missing is that `docs/backends/metal.md` recorded the exclusion in one clause — "Runtime pipeline creation remains necessary, but runtime source compilation does not" — as a statement about what is *unnecessary*, which is a weaker thing than a prohibition and left the numerical case unaddressed.

What the new section adds is the second, independent justification the finding supplies, which ADR 0002's latency-and-deployment argument does not carry: within the exclusion an artifact's recorded provenance is *complete* for everything Tiler compiles, and outside it an artifact would carry a declared numerical realization attributable to no compiler its own provenance identifies. The section also states why this is not fixable by widening the provenance record — an identity dimension must be fixed at expansion time, and the runtime compiler is not selected until the process that runs the kernel exists — so admitting runtime compilation would require a new provenance mechanism keyed by the execution environment rather than a caveat.

### The boundary that was left unaddressed and is now stated

The exclusion scopes Tiler's kernels, not the host process, and the first consumer is a live counterexample to the loose reading. Verified by reading the source, not inferred from a grep: in the local `huggingface/candle` working checkout at revision `4bb954d`, `Kernels::load_library` in `candle-metal-kernels/src/kernel.rs` compiles each built-in kernel source through `new_library_with_source`, and `MetalDevice::compile` in `candle-core/src/metal_backend/device.rs` does the same for a `ug`-generated kernel. So a Tiler kernel and a Candle kernel can sit in one command buffer with different numerical provenance, and Tiler's claims cover only the first. `scope-tiler-numerical-claims-across-the-candle-kernel-boundary` owns what `docs/integration/candle.md` should conclude from that; `contracts/integrations` is not held here.

### Both halves of the agreement, carried together

A reader who takes only the provenance gap away over-corrects, so the section states the agreement at full strength and its boundary in the same breath. All 40 macOS runtime cases returned bit patterns identical to their offline `-O2` counterparts for every operand, and the iOS Simulator's 40 agreed the same way against its own offline path; across the three artifact families all 42 compiled cases emit byte-identical `air.compile_options` and floating-point operation lists. The counts were checked against the retained record rather than copied: `comparison.macos.*` and `comparison.ios-simulator.*` are 40 rows each with zero `differ`, and each family carries 42 distinct offline case keys. The record's own gloss of the 40 ("eight kernels across `MTLMathModeSafe`, `Relaxed`, and `Fast`, at both `MTLLibraryOptimizationLevelDefault` and `Size`") reads as a 48-element cross product; it is not one, because the two contraction kernels are swept under `safe` only — 6 × 3 × 2 plus 2 × 1 × 2 — so the section states the sweep without implying the product.

Against that, three sentences of boundary: it is agreement between separately built compilers rather than one compiler invoked twice, which makes it *stronger* than a self-comparison and still one host row; it does not make a realization read off an offline build transferable to a runtime-compiled one, only coincident here; and nothing in it licenses relaxing the exclusion. The cross-family half carries finding 13's boundary explicitly, because the iOS Simulator dispatches on the host Mac GPU with the same `registryID` and `IOsDevice` dispatches nowhere.

### Sequencing against `declare-metal-numerical-honourability`

The new section is placed immediately after "Numerical compiler realization" and does not touch a word of it — in particular not the sentence that the compatibility probe "did not observe the numerical behavior these flags request", which is the sentence that ticket's contract half will replace. The seam is stated in the section's last paragraph rather than left for a later editor to discover: this section identifies *which* compiler delivered a realization and how far that identification reaches, does not state what the realization is, and names `declare-metal-numerical-honourability` as the owner of the subnormal verdict. The two are disjoint by construction — the provenance text reads the same whatever the subnormal verdict turns out to be — so that ticket can amend the section above without re-editing this one.

### `docs/artifact-abi.md`, which the ticket asked to be checked rather than assumed

Toolchain provenance is sited in both documents, and the artifact ABI's is the one that could be misread. "Artifact identity" lists, for Metal, "the resolved `metal` and `metallib` component versions or executable digests" — which is the offline compiler and reads like *the* compiler. One paragraph was added there stating that those versions identify the offline compiler alone, that the runtime source compiler belongs to the execution environment rather than to the artifact so no artifact identity can name it, and that widening the list would not change that. The measurement itself is deliberately not restated there; the paragraph points at `docs/backends/metal.md`, which keeps one authority over the measured row and avoids a one-way `evidence` edge from a contract the research record does not name in `informs`.

### What ADR 0076 should gain

One sentence, and it is not this ticket's to write: ADR 0076 is accepted and `docs/decisions/**` is `contracts/decisions`. Its conclusion is unchanged — findings 9, 11, and 12 support it and finding 8 strengthens its central argument, since honourability must be a stated versioned target fact precisely because the compiler that delivers it is not always the one the artifact names. What it should add is to item 3's provenance discipline, which already requires an availability phase, a validity scope, an authority, and the declaring profile's identity: that the validity scope must identify which compiler build and which execution environment the behaviour was measured on, cross-referenced from item 4, whose delivered-realization record inherits the same requirement. `name-the-compiler-and-environment-in-adr-0076-target-facts` carries the proposed wording. `repoint-adr-0076-evidence-at-the-numerical-record` is already `done`, so this needed a new ticket rather than an addition to that one.

### One correction to the ticket's own text

This ticket cites `results/2026-07-24-numerics-xcode26.6-metal32023.883/record.tsv` and attributes the macOS runtime compiler to `/System/Library/PrivateFrameworks/MTLCompiler.framework`, `CFBundleVersion` 382.5. Both are superseded by `measure-numerics-across-apple-artifact-families`, which landed after this ticket was written. The current retained record is `results/2026-07-24-numerics-families-xcode26.6-metal32023.883/record.tsv`, and finding 12 recovers the loaded image from `dyld` rather than assuming it: no image whose path contains `MTLCompiler` is loaded into either process at all, and `environment.family.macos.runtime_compiler_images` names `GPUCompiler.framework` instead. The contract text therefore says `GPUCompiler.framework`, and says three compiler builds rather than two.

### Follow-ups

- `name-the-compiler-and-environment-in-adr-0076-target-facts` (p2, `contracts/decisions`) — the one sentence ADR 0076 should gain, with proposed wording.
- `scope-tiler-numerical-claims-across-the-candle-kernel-boundary` (p2, `contracts/integrations`) — what a consumer may conclude when a Tiler kernel and a runtime-compiled Candle kernel share a command buffer, and re-pinning the Candle source claim against a real dependency revision.
- `give-the-apple-open-questions-a-runtime-compiler-drift-axis` (p3, `contracts/navigation`) — Q-ART-007 closes on a matrix over machines and toolchain patch versions, and the runtime compilers move with the OS build and the simulator runtime instead.

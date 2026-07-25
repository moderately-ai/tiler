---
schema: "tiler-doc/v1"
id: "tiler.contract.metal-backend"
kind: "contract"
title: "Metal AOT backend"
topics: ["backends", "metal", "aot", "apple-targets"]
contract_status: "accepted"
implementation_status: "not-started"
evidence: ["tiler.research.apple-targets.compatibility", "tiler.research.apple-targets.numerical-behaviour", "tiler.research.artifacts.target-neutral-envelope", "tiler.research.macro-environment.build-environment"]
ticket: "synthesize-artifact-contracts"
---

# Metal AOT backend

**Status:** accepted backend contract; runtime compatibility matrix remains bounded

The Metal backend translates an already scheduled program into deterministic
MSL. The frontend proc macro invokes Apple's offline tools during expansion,
then embeds the completed bytes. Runtime pipeline creation remains necessary,
but runtime source compilation does not.

## Pipeline

```text
verified scheduled iteration IR
    -> structured typed kernel IR
    -> deterministic MSL translation unit
    -> xcrun metal -> AIR
    -> xcrun metallib -> metallib
    -> macro-local bundle with versioned manifest
    -> manifest/metallib byte-string literals in generated Rust
```

The pure emitter owns syntax translation and helper emission. It receives a
structured kernel already verified as a refinement of its schedule, together
with target requirements, providers, resources, and ABI. It does not create a
Metal device, invoke `xcrun`, inspect Candle layouts, decide fusion, repair
missing synchronization, or change reduction/numerical behavior.

## Scheduled lowering

Before MSL emission, target lowering resolves:

- grid and threadgroup built-ins;
- vector types and operations;
- address spaces and access modes;
- tail masking;
- explicit subgroup or threadgroup reduction algorithms;
- barrier scopes;
- static and dynamic threadgroup storage;
- numerical-mode-specific intrinsics.

It first checks support for every governed kernel type, operation, memory
space, builtin coordinate, collective, fence, conversion, and required MSL
feature. A gap is a typed backend rejection. MSL compiler acceptance is an
additional validation layer, not a substitute for the kernel verifier.

There is no final generic `BlockReduce`. A scheduled reduction is an explicit
algorithm with convergence, lane-visibility, extent, dtype, and capability
requirements.

Native MSL spelling does not establish semantic compatibility. MSL `fmin` and
`fmax` prefer a numeric operand over NaN and select an equal operand in a way
that can make opposite-signed-zero results operand-order-dependent. Strict
Tiler `Minimum`/`Maximum` propagate NaN, while strict `MinimumNumber`/
`MaximumNumber` still order `-0.0 < +0.0`. Metal lowering therefore emits a
semantic fixup or rejects the native alternative unless the operation's
resolved numerical permissions explicitly admit the native behavior.

## Target profiles

Offline scheduling uses a named conservative target profile, not assumptions
about an unspecified Apple GPU. A profile records platform/deployment range,
supported dtypes/features, SIMD-group assumptions, maximum threads and
threadgroup memory, binding limits, supported address/index widths, and
bootstrap cost parameters.

This is the Metal instance of ADR 0043's generic schema. Family/platform facts
are compile guarantees; `MTLDevice` facts are live-device facts; and
`threadExecutionWidth`, `maxTotalThreadsPerThreadgroup`, and
`staticThreadgroupMemoryLength` are prepared-pipeline facts keyed by device,
bundle, entry point, function constants, canonical pipeline descriptor, and
archive/runtime mode. A metallib load is not a pipeline feasibility proof.

Numerical capabilities are keyed by operation, dtype, effective accuracy,
special-value and subnormal contracts, implementation/helper revision, and
toolchain profile. A generic claim that a target supports `fast` or `precise`
math is not a feasibility fact.

Some limits are known only after pipeline creation, such as execution width or
maximum threads for a compiled function. The manifest records corresponding
deferred preflight assertions. A bundle may contain a conservative generic
portfolio plus device-family variants with explicit compatibility guards.
Profile and cost-model version are compilation provenance and scheduled
identity.

Metal does not expose stable planning facts for exact register use, spills,
active threadgroups, or occupancy. These remain estimates or measurements;
Metal feasibility uses pipeline creation plus documented launch/resource caps,
not a generic nonzero-occupancy rule. Pipeline maximum threads is a hard launch
limit, not an occupancy estimate.
Recommended working-set size is likewise performance guidance, not an
allocation ceiling.

Metal may also implement a versioned provider for a semantic target-property
binding declared by the backend-neutral program interface. Compile-profile and
live-device-preflight properties may bind initial semantic extents when their
contracts are deterministic and available before allocation. Pipeline-derived
properties remain physical assertions in the initial model; they cannot feed
semantic output shapes merely because the backend can query them after
pipeline creation.

For a concrete launch, preflight checks each threadgroup axis against the live
device, the product against the pipeline's
`maxTotalThreadsPerThreadgroup`, and pipeline static plus every aligned dynamic
threadgroup-memory allocation against the live device limit. It also validates
the selected uniform/nonuniform dispatch mode, checked launch-index
representation, actual input binding presence/access/base-plus-offset alignment
and range, and output/temporary allocation specifications plus allocator
alignment/capacity guarantees before `RoutingCommit`. After allocation, the
returned output/temporary bindings are validated against those guarantees as
post-commit invariants; a mismatch fails closed.

## MSL emission

A macro-local translation unit should:

- include and deduplicate every entry point/helper required by that invocation's
  complete plan portfolio;
- use deterministic content-derived symbols;
- emit each helper once;
- assign explicit `[[buffer(N)]]` attributes;
- emit explicit built-in parameters;
- realize each operation's effective accuracy and independent NaN, infinity,
  signed-zero, contraction, and subnormal contracts; a translation-unit-wide
  flag is legal only when it stays within every affected operation contract;
- include comments mapping symbols to semantic hashes and optional origins;
- state the required Metal language and platform version.

Source emission should be snapshot-testable and should never depend on global
counters or hash-map iteration order.

Buffer-offset alignment, MSL pointer alignment, vector-load alignment, and
metadata-struct alignment are distinct rules. Host packers and MSL declarations
are generated from the artifact binding table. Scalar metadata passed through
`set_bytes` is still represented in Metal's buffer namespace; maximum inline
payload and constant-buffer fallback are target/runtime capabilities.

## Numerical compiler realization

Tiler never inherits the Metal compiler's math defaults. The strict baseline
spellings for the local Metal 32023.883 toolchain row are:

```text
-fmetal-math-mode=safe
-fmetal-math-fp32-functions=precise
-ffp-contract=off
```

The [Apple artifact-compatibility probe](../research/apple-targets/artifact-compatibility.md)
measured only that this toolchain accepted these exact spellings and compiled
every tested macOS, iOS-device, and iOS-simulator tuple with them. That probe
qualifies the row for bounded compile and same-host reproducibility alone; it
explicitly does not qualify it for Tiler's runtime support matrix or numerical
conformance, and it did not itself observe the numerical behavior these flags
request.

**Measurement — the strict row does not deliver subnormal preservation.** A
separate measurement recorded in
[ADR 0076](../decisions/0076-declare-target-honourable-numerical-realizations.md) did
observe it, and the result is negative: `-fmetal-math-mode=safe` emits
`air.compile.denorms_disable` alongside `air.compile.fast_math_disable`, and it
does so under `safe`, `relaxed`, and `fast` alike. No offline flag and no
runtime `MTLCompileOptions` setting clears it. The strict spellings above
therefore request preservation and do not obtain it.

**The limit is arithmetic specifically, not the target generally.** A
load-then-store round trip preserves every subnormal bit pattern, so
materialization is unaffected; only `f32` *arithmetic* flushes. Stating this as
a blanket property of the target would be wrong in the direction that matters,
because a program that only moves subnormals is unaffected by it.

**The flush is sign-preserving.** On an Apple M4 Max under macOS 27.0 with
Metal 32023.883, an emitted `x * 2.0f` returns `0x80000000` for the operand
`0x80400000`, not `0x00000000`. That is why a flush-accepting contract can be a
positive conformance claim on this row rather than merely a weaker one: the
zero a contract names can be compared against the zero the target produces, and
only a genuine mismatch is a gap.

**Inference — honourability is a stated fact here, never a probed one.** Under
`-fmetal-math-mode=relaxed` a `scale 1.0, bias +0.0` kernel returns subnormal
operands unchanged, which looks like preservation and is not: `x * 1.0` folds to
a copy under every math mode, the kernel retains exactly one floating-point
operation under `safe` (the `+0.0` fadd, unremovable without `nsz`) and zero
under `relaxed`, and the surviving `fadd` is what flushes. The same licence that
breaks signed zero deletes the operation that would have flushed. So preserved
subnormals observed from a compiled kernel are not evidence that this target
preserves them, and the modes where that inference misleads are exactly the
least trustworthy ones. `MetalTargetFacts::subnormal_arithmetic` is
correspondingly a required caller-stated fact with its measurement recorded on
the type, not a value inferred from a probe kernel.

These spellings are a governed realization for that toolchain row, not a
portable promise that future Metal compilers use the same flags or definitions.
Each supported toolchain row maps the canonical per-operation numerical
contract to explicit compiler flags, intrinsics, helpers, and fixups and carries
conformance evidence for that mapping. An unavailable realization rejects the
candidate or toolchain; it does not fall back to compiler defaults.

Relaxation is not one `fast` bit. Reassociation, operand permutation,
reciprocal transforms, approximate elementary functions, NaN/Inf assumptions,
signed-zero behavior, contraction, subnormal handling, and observable
intermediate-rounding removal remain independent permissions in IR, explain
output, compiler realization, and artifact identity.

## Compiler provenance and the runtime compiler

**Fact — Tiler compiles no MSL at runtime, and that is already decided.** [ADR 0002](../decisions/0002-aot-metal-artifacts.md) decides that the runtime "creates and caches pipeline objects from compiled artifacts but does not compile MSL source". [ADR 0043](../decisions/0043-use-typed-phased-target-feasibility.md) restates it as a standing prohibition — "this does not authorize runtime source compilation: the initial product still forbids it, while a backend may declare required device translation of an AOT target-IR artifact such as a metallib" — and [Vision](../vision.md) lists runtime source compilation among the first implementation's non-goals. `newLibraryWithSource:options:` is therefore on no Tiler path, and this section does not reopen that question. It states what the exclusion is worth, because the measurement below is a second and independent justification for it that ADR 0002's latency-and-deployment argument does not carry.

**Measurement — one Apple host resolves three Metal compiler builds, and an artifact's toolchain provenance names one of them.** On the qualified row of the [Apple GPU `f32` numerical behaviour](../research/apple-targets/numerical-behaviour.md) record — Apple M4 Max, macOS 27.0 build 26A5388g, Xcode 26.6 build 17F113, with SDKs `macosx` 26.5 build 25F70, `iphoneos` 26.5 build 23F81a, and `iphonesimulator` 26.5 build 23F81a — the offline `xcrun metal` driver shared by all three SDKs is `metalfe-32023.883`, resolved from the Xcode MetalToolchain asset. A macOS host process compiling the byte-identical source in process through `newLibraryWithSource:options:` loads `metalfe-32023.921` from `/System/Library/PrivateFrameworks/GPUCompiler.framework`, which ships with the OS rather than with Xcode. A booted iOS 26.0 Simulator, build 23A8464, doing the same loads `metalfe-32023.830.1` from the simulator runtime's own bundled copy of that framework. Three distinct builds, one machine, one instant (findings 8 and 12).

**Inference — the runtime compiler belongs to the execution environment, so no artifact identity can name it.** The Metal identity dimensions this contract and the [artifact ABI](../artifact-abi.md) record — normalized platform family, requested deployment minimum, MSL standard, SDK version and build, the flag set, and the resolved `metal` and `metallib` versions or digests — identify the offline compiler exactly and identify no runtime compiler at all. A runtime compiler moves with the OS build or with the simulator runtime rather than with the artifact, so it can change under byte-identical artifact bytes, and one machine resolves two of them at once for two different artifact families. This is not a missing identity dimension that a wider provenance record would supply: the dimension would have to be fixed at expansion time, and the runtime compiler is not selected until the process that runs the kernel exists.

**Inference — the AOT exclusion is therefore load-bearing rather than merely conservative.** Within it the recorded provenance is complete for everything Tiler compiles, because every Tiler kernel is produced by the compiler its artifact names. Outside it an artifact would carry a declared numerical realization attributable to no compiler its own provenance identifies, and [ADR 0076](../decisions/0076-declare-target-honourable-numerical-realizations.md)'s requirement that a delivered realization be recorded and never inferred would have nothing to anchor that record to. Admitting runtime compilation is consequently not a caveat to add to the provenance record. It would first require a distinct provenance mechanism keyed by the execution environment and resolved after the artifact is built, and no such mechanism is proposed here.

**Fact — the exclusion scopes Tiler's kernels, not the host process.** A process running a Tiler artifact may also run Metal kernels Tiler did not compile, and the OS-resident compiler above is what compiles them. The intended Candle consumer is a concrete instance rather than a hypothetical one, though Tiler declares no Candle dependency today: at `huggingface/candle` `31f35b147389700ed2a178ee66a91c3cc25cc80d` (0.11.0), `Kernels::load_library` at `candle-metal-kernels/src/kernel.rs:109` compiles each built-in kernel source through `new_library_with_source` at line 122, and `MetalDevice::compile` at `candle-core/src/metal_backend/device.rs:101` does the same at line 111 for a `ug`-generated kernel. Tiler's recorded provenance and declared numerical realization cover the kernels Tiler emitted and compiled; they say nothing about a neighbouring kernel in the same command buffer. What a consumer may conclude across that boundary belongs to [Candle integration](../integration/candle.md).

**Measurement — where the two compilation paths were compared, every case agreed.** All 40 macOS runtime cases returned bit patterns identical to their offline `-O2` counterparts for every operand in the probe's vector, across `MTLMathModeSafe`, `Relaxed`, and `Fast` at both `MTLLibraryOptimizationLevelDefault` and `Size`; the iOS Simulator's 40 agreed the same way against its own offline path (findings 9 and 12). The compile side is byte-identical across all three artifact families: for all 42 compiled cases the emitted `air.compile_options` string set and the emitted floating-point operation list — opcode and fast-math flags — match exactly between `MacOs`, `IOsDevice`, and `IOsSimulator` (finding 11). Because each family's runtime path is compared against its own offline path, the simulator's agreement holds between `metalfe-32023.883` and `metalfe-32023.830.1`, a pair differing in both directions from the macOS one.

**Fact — that agreement is a bounded measurement and not a guarantee, and both halves travel together.** It is agreement between separately built compilers rather than one compiler invoked twice, which makes it a stronger observation than a self-comparison. It is also one host, one OS build, one Xcode build, one offline toolchain build, one Mac GPU, one simulator runtime, and one MSL version. It is not evidence that any other pair of builds agrees, and it does not make a realization read off an offline build *transferable* to a runtime-compiled kernel — it makes the two coincide here. Nothing in it licenses relaxing the exclusion above. The independence of the two builds is the reason to keep re-measuring, not the reason to stop.

**Fact — the boundary on the cross-family half.** The iOS Simulator dispatches on the host Mac GPU: `MTLCreateSystemDefaultDevice` inside the booted runtime reports device name `Apple iOS simulator GPU` and the *same* `registryID` as `Apple M4 Max` on the host. A simulator result is admissible as a measurement of what the `IOsSimulator` family delivers on this host and is not evidence about iPhone or iPad hardware. `IOsDevice` is compiled for all 42 cases and dispatched for none, because no physical device is attached; closing that gap requires dispatching the `air64-apple-ios16.0` metallib on an Apple-silicon iPhone or iPad's own GPU. Loading that module on this Mac is not a substitute and is refused even though it succeeds (findings 13 and 14).

**Fact — what this section does not state.** It identifies *which* compiler delivered a numerical realization and how far that identification reaches. It does not state what the realization is. What the strict flag row above is measured to deliver is recorded in that section rather than here; its subnormal verdict has since been measured and is negative for arithmetic and positive for materialization. The two sections are deliberately disjoint: this one reads the same whatever the subnormal verdict is, which is why recording that verdict did not touch it.

## Expansion-time offline compilation

On a supported macOS host, the proc-macro AOT layer:

1. resolves the explicit artifact family and qualified toolchain row, then
   computes full identity from canonical plans, MSL, target, SDK, compiler,
   linker, flags, and numerical realization;
2. reads a validated global cache hit when available;
3. otherwise acquires a per-hash cross-process lock and rechecks;
4. writes canonical MSL to a temporary cache entry;
5. runs `xcrun metal` with explicit platform target, deployment minimum,
   language, optimization, debug, and math flags and then runs `xcrun metallib`
   through the same selected SDK/toolchain;
6. validates and atomically publishes the cache entry;
7. converts manifest and metallib bytes into proc-macro byte-string literals;
8. returns self-contained Rust tokens with source-spanned diagnostics on error.

No consumer `build.rs`, descriptor registry, or custom Cargo command is used.
Identical invocations avoid external compilation through content identity;
crate-wide metallib aggregation is not required. One invocation still
aggregates all entry points needed by its own one- or multi-kernel plans.

## Expansion compiler cache

The cache is content-addressed and concurrency-safe:

- full artifact identity is the key;
- writers use stable per-key OS advisory locks, recheck after locking, and
  create-new same-filesystem temporary files;
- completed temporary bundles are independently validated before one atomic
  rename publishes them;
- readers validate hashes and completeness;
- toolchain or flag changes invalidate entries;
- resolved accuracy contracts, selected helpers/intrinsics, and conformance-
  provider revisions invalidate entries;
- cache hit/miss reasons can be inspected.

Locks suppress duplicate work; immutable self-validating entries and atomic
publication provide correctness. Corruption is a miss followed by locked
recheck/rebuild. Internal GC retains lock files and takes the per-key lock before
eviction. Cache I/O failure uses validated uncached compilation; Metal compiler
or artifact validation failure remains a hard macro error. The default promises
process-crash safety, not power-loss durability.

The cache is an internal accelerator. Generated Rust embeds completed bytes and
does not refer to cache paths. The default lives in an OS-appropriate user cache
with a CI/sandbox override, rather than consumer `OUT_DIR`. Stable Cargo cannot
be assumed to track arbitrary proc-macro filesystem/environment side effects,
so complete identity and explicit invalidation live in Tiler.

The cache key includes the Metal compiler fingerprint. Tool failures retain MSL
and diagnostics when requested and become macro compilation errors rather than
runtime fallback.

## Runtime cache

Per Metal device, cache:

1. `MTLLibrary` by bundle hash;
2. `MTLFunction` by bundle, symbol, and function constants;
3. `MTLComputePipelineState` by the same identity plus canonical descriptor,
   archive, and relevant runtime-mode fields.

Device-bound objects are never stored in a device-agnostic global singleton.
Initialization is concurrency-safe and fallible. Pipelines are not recreated
for each dispatch.

## Platform concerns

- The Metal AOT toolchain requires an eligible macOS compilation host.
- macOS, iOS device, and iOS simulator are distinct measured artifact families.
  Platform family, requested deployment minimum, SDK identity, MSL standard,
  compiler/linker identity, and flags are explicit payload/cache dimensions.
- Proc-macro host identity is never treated as the consumer target. A canonical
  `ArtifactFamilySelection` explicitly enables one or several families and
  generated Rust selects only among compatible embedded payloads.
- Mac Catalyst is a fourth `ios` + `macabi` family and is deferred. It is not
  relabeled as macOS or iOS-device compatible.
- The checked-in probe compiled all measured macOS/iOS-device/simulator tuples
  with Metal 32023.883. Final metallibs differed across platform families and
  were byte-identical across two source directories for the trivial kernel;
  AIR retained path-dependent bytes. This is same-host build evidence, not
  old-OS/device runtime qualification.
- BF16 and other features are guarded by target capabilities.
- Generated variant and macro-local bundle size are bounded to avoid metallib
  and embedded-binary bloat.
- Expansion-time compiler rejection may choose the next retained candidate before
  publication. Runtime pipeline-limit rejection may choose another published
  plan only during pre-encoding preparation; the emitter itself never performs
  an unbounded optimizer search.

An offline metallib is GPU-independent Metal IR and may still compile to
device-specific machine code during pipeline creation. Library load, function
lookup, and pipeline construction are separate failure stages. Tiler describes
this as source-level AOT, not zero first-dispatch compilation.

`MTLBinaryArchive` and offline pipeline binaries are a separate optional runtime
cache/distribution problem with device-family and OS compatibility. Dynamic
Metal libraries likewise introduce runtime assets and dependencies. Both remain
deferred until measured startup or size costs justify changing the initial
self-contained payload contract.

## Traceability

This document owns structured-kernel-to-Metal lowering and Apple AOT target
requirements. Artifact framing and consumer execution are owned by the
[artifact](../artifact-abi.md) and [Candle integration](../integration/candle.md)
contracts. Frontmatter links the accepted decisions and bounded evidence.

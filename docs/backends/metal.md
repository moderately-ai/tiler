---
schema: "tiler-doc/v1"
id: "tiler.contract.metal-backend"
kind: "contract"
title: "Metal AOT backend"
topics: ["backends", "metal", "aot", "apple-targets"]
contract_status: "accepted"
implementation_status: "not-started"
governed_by: ["ADR-0002", "ADR-0049", "ADR-0050", "ADR-0053"]
evidence: ["tiler.research.apple-targets.compatibility", "tiler.research.artifacts.target-neutral-envelope", "tiler.research.macro-environment.build-environment"]
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

Tiler never inherits the Metal compiler's math defaults. On the qualified local
Metal 32023.883 toolchain, the measured strict baseline is:

```text
-fmetal-math-mode=safe
-fmetal-math-fp32-functions=precise
-ffp-contract=off
```

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

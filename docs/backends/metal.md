# Metal AOT backend

**Status:** proposed

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

The pure emitter owns syntax translation and helper emission. It does not
create a Metal device, invoke `xcrun`, inspect Candle layouts, or decide fusion.

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

There is no final generic `BlockReduce`. A scheduled reduction is an explicit
algorithm with convergence, lane-visibility, extent, dtype, and capability
requirements.

## Target profiles

Offline scheduling uses a named conservative target profile, not assumptions
about an unspecified Apple GPU. A profile records platform/deployment range,
supported dtypes/features, SIMD-group assumptions, maximum threads and
threadgroup memory, binding limits, supported address/index widths, and
bootstrap cost parameters.

Some limits are known only after pipeline creation, such as execution width or
maximum threads for a compiled function. The manifest records corresponding
runtime assertions. A bundle may contain a conservative generic portfolio plus
device-family variants with explicit compatibility guards. Profile and
cost-model version are compilation provenance and scheduled identity.

Metal may also implement a versioned provider for a semantic target-property
binding declared by the backend-neutral program interface. Compile-profile and
live-device-preflight properties may bind initial semantic extents when their
contracts are deterministic and available before allocation. Pipeline-derived
properties remain physical assertions in the initial model; they cannot feed
semantic output shapes merely because the backend can query them after
pipeline creation.

## MSL emission

A macro-local translation unit should:

- include and deduplicate every entry point/helper required by that invocation's
  complete plan portfolio;
- use deterministic content-derived symbols;
- emit each helper once;
- assign explicit `[[buffer(N)]]` attributes;
- emit explicit built-in parameters;
- preserve precise versus fast numerical policy;
- include comments mapping symbols to semantic hashes and optional origins;
- state the required Metal language and platform version.

Source emission should be snapshot-testable and should never depend on global
counters or hash-map iteration order.

Buffer-offset alignment, MSL pointer alignment, vector-load alignment, and
metadata-struct alignment are distinct rules. Host packers and MSL declarations
are generated from the artifact binding table. Scalar metadata passed through
`set_bytes` is still represented in Metal's buffer namespace; maximum inline
payload and constant-buffer fallback are target/runtime capabilities.

## Expansion-time offline compilation

On a supported macOS host, the proc-macro AOT layer:

1. computes full identity from canonical plans, MSL, target, and toolchain;
2. reads a validated global cache hit when available;
3. otherwise acquires a per-hash cross-process lock and rechecks;
4. writes canonical MSL to a temporary cache entry;
5. runs `xcrun metal` with explicit SDK/language flags and then `xcrun
   metallib`;
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
- writers use locks and temporary files;
- publication uses atomic rename;
- readers validate hashes and completeness;
- toolchain or flag changes invalidate entries;
- cache hit/miss reasons can be inspected.

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
3. `MTLComputePipelineState` by the same identity plus descriptor fields.

Device-bound objects are never stored in a device-agnostic global singleton.
Initialization is concurrency-safe and fallible. Pipelines are not recreated
for each dispatch.

## Platform concerns

- The Metal AOT toolchain requires an eligible macOS compilation host.
- macOS and iOS platform, minimum deployment version, SDK, and language version
  are explicit artifact dimensions.
- Initial support is native macOS AOT. Non-Apple hosts emit non-Apple fallback;
  cross-Apple compilation from a non-macOS host is rejected explicitly.
- Proc-macro host identity is not silently treated as the Rust target. Future
  macOS/iOS/simulator bundles can be generated as an enabled family and selected
  in emitted Rust with target `cfg` attributes.
- BF16 and other features are guarded by target capabilities.
- Generated variant and macro-local bundle size are bounded to avoid metallib
  and embedded-binary bloat.
- Expansion-time compiler rejection may choose the next retained candidate before
  publication. Runtime pipeline-limit rejection may choose another published
  plan only during pre-encoding preparation; the emitter itself never performs
  an unbounded optimizer search.

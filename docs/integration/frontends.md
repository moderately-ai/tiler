---
schema: "tiler-doc/v1"
id: "tiler.contract.frontend-integration"
kind: "contract"
title: "Frontend and proc-macro integration"
topics: ["integrations", "frontends", "proc-macros", "aot"]
contract_status: "accepted"
implementation_status: "not-started"
governed_by: ["ADR-0004", "ADR-0049", "ADR-0050", "ADR-0053"]
evidence: ["tiler.research.macro-environment.build-environment", "tiler.research.embedding.artifact-costs", "tiler.research.cache.crash-race-protocol"]
ticket: "synthesize-artifact-contracts"
---

# Frontend and proc-macro integration

**Status:** accepted inline AOT contract; rust-analyzer performance remains unmeasured

Frontends translate user-facing tensor languages into Tiler's public semantic
tensor graph. `candle-einops` is the first proposed frontend. For that Rust
integration, ordinary inline invocation is a developer-experience constraint:
users do not declare kernels separately, add a build script, run a Cargo
subcommand, or accept runtime JIT compilation. This does not define the
frontend-neutral compiler API or constrain other integrations to use macros.

For an inline proc-macro frontend, the operation-provider snapshot is limited
to providers in the macro's host dependency graph plus complete canonical
semantic declarations present in invocation tokens. A consumer-local Rust
trait implementation is target-crate code and is not executable by the
already-compiled proc macro. ADR 0045 records this boundary; it does not narrow
the provider set accepted by the ordinary compiler API.

## Invocation is the compilation unit

Given:

```rust
let y = einops!("b h w c -> b c", x);
```

the proc macro sees the complete frontend operation represented by that
invocation. It parses and verifies the expression, invokes Tiler optimization,
generates every entry point needed by the selected plan portfolio, compiles one
macro-local metallib, and embeds the artifact in its returned Rust tokens.

```text
macro tokens
  -> frontend plan
  -> semantic IR
  -> logical/physical optimization
  -> program-plan portfolio
  -> MSL translation unit containing all required entry points
  -> content-addressed AOT compilation
  -> embedded artifact + runtime/fallback expression
```

One invocation may contain one fused kernel, multiple guarded schedule variants,
or a multi-step plan such as a two-pass reduction. “Macro-local bundle” does
not mean “one GPU kernel.”

## Frontend responsibilities

A frontend owns:

- parsing and source spans;
- axis names, ellipses, and grouping syntax;
- frontend-specific validation and diagnostics;
- rank, shape, and axis constraints;
- translation into explicit atomic semantic operations such as `Reindex`,
  `Broadcast`, named pointwise operations, and `Reduce`;
- resolution of source-library transcendental behavior or ergonomic accuracy
  presets into complete canonical per-operation contracts; an underspecified
  source intrinsic selects an explicit import profile or is rejected rather
  than inheriting the eventual backend;
- construction of a semantically compatible fallback expression;
- invocation of the compiler/AOT pipeline from its proc-macro crate.

It does not own Candle storage strides, runtime variant selection, Metal device
objects, or command encoding.

## Compile-time knowledge

An einops macro commonly knows the expression graph, ranks and axis
relationships, permutations, split/merge relationships, reduction axes, scalar
expression structure, and statically supplied axis sizes. Runtime extents,
Candle storage strides/start offsets, buffer alignment, and device capabilities
remain typed artifact metadata and guards.

## Expansion-time AOT flow

The proc macro synchronously performs:

1. Parse input tokens and retain diagnostic spans.
2. Construct, verify, normalize, optimize, and schedule a semantic program.
3. Emit deterministic MSL and a canonical artifact manifest.
4. Compute a complete artifact identity.
5. Look up the identity in a global content-addressed compiler cache.
6. On a miss, acquire a cross-process hash lock, check again, invoke `xcrun
   metal` and `xcrun metallib`, validate the result, and publish atomically.
7. Read the manifest and metallib bytes.
8. Emit them as byte-string literals together with runtime selection and
   fallback code.

Target-neutral parse, semantic, optimizer, verifier, and envelope failures
become unconditional `compile_error!` diagnostics associated with the invocation
span. A selected artifact family's unavailable toolchain or external compiler
failure is retained as a family-scoped diagnostic and emitted under that
family's governed consumer `#[cfg]`; it is fatal when the consumer target
matches that requested family but does not break an unrelated fallback-only
target. Debug configuration may retain canonical MSL and tool diagnostics under
the cache entry.

Rust procedural macros execute during compilation with the compiler's file and
process resources, so host tool invocation is within their execution model; it
also carries the same security responsibility as build scripts. See the
[Rust procedural macro reference](https://doc.rust-lang.org/reference/procedural-macros.html).

## Direct byte embedding

The generated code conceptually contains:

```rust
{
    static MANIFEST: &[u8] = b"...";
    static METALLIB: &[u8] = b"...";

    ::tiler_candle::execute_or_fallback(
        ::tiler_artifact::EmbeddedBundle::new(MANIFEST, METALLIB),
        /* tensors and fallback */,
    )
}
```

The actual proc-macro implementation should construct byte-string literal
tokens directly rather than emit millions of integer tokens. No generated path
or `include_bytes!` dependency is required. The completed Rust expansion is
self-contained; the compiler cache can be deleted without affecting an already
compiled binary.

The stable proc-macro API provides `Literal::byte_string` for constructing one
byte-string token from artifact bytes; see
[`proc_macro::Literal`](https://doc.rust-lang.org/proc_macro/struct.Literal.html).

Measured 10 KiB, 100 KiB, and 1 MiB fixtures confirm byte-string literals as
the required representation. At 1 MiB, one numeric token per byte used about
3.5 times the wall time and peak RSS while linking identical output. Linker
merging is not assumed: default release retained all eight identical 100 KiB
copies in the measured fixture, and folding varied with crate boundaries,
codegen units, and LTO.

The initial gate is 1 MiB per invocation and 32 invocations or 3.2 MiB of
logical embedded bytes per consumer package. Crossing it requires an explicit
override and remeasurement. Macro diagnostics report logical bytes and payload
counts; CI owns the crate-wide aggregate because independent invocations cannot
reliably coordinate it.

## Compiler cache

The cache avoids repeated external compilation; it is not an output contract.
Its key includes:

- canonical semantic and scheduled IR;
- complete program-plan portfolio, ABI, guards, and numerical contract;
- MSL and helper-library identity;
- Tiler schema/compiler/codegen versions;
- Metal platform/profile/language version;
- `xcrun`/Metal compiler fingerprint and flags.

A default macOS user cache is used rather than consumer `OUT_DIR`. A documented
override supports CI and sandboxed builds. One immutable self-validating bundle
is stored per complete key. A miss uses a stable per-key OS advisory lock,
locked recheck, create-new same-filesystem temporary file, full temporary
validation, and atomic rename. Readers validate every hit without taking the
lock. Identical invocations share external compiler work even when expanded in
different rustc processes.

Locking suppresses duplicate work; complete identity, immutable bytes,
validation, and atomic publication provide correctness. Corruption is a miss.
Cache I/O failure compiles and validates without publication; compiler or
artifact failure remains a hard error. The default durability contract covers
process crashes, not power loss.

Deleting the cache may cause the next macro expansion to rebuild it; generated
runtime code never opens cache files. Cache cleanup and compiler incremental
state are tested together.

Explicit proc-macro environment/path dependency tracking is currently an
experimental Rust API, so correctness must not depend on Cargo discovering
cache side effects. See
[`proc_macro::tracked`](https://doc.rust-lang.org/proc_macro/tracked/index.html).

## Target policy

Proc macros execute for the host and do not receive the same guaranteed target
metadata as Cargo build scripts. Each invocation therefore resolves a typed,
canonical `ArtifactFamilySelection`; it does not infer the consumer family from
the proc-macro host. A selection may name one or several governed families such
as macOS, iOS device, and iOS simulator. Each family remains a distinct artifact
with its own target manifest and content identity.

A frontend may offer an ergonomic literal default profile, but the resolved
selection is still explicit compiler input. Generated Rust may use `#[cfg]` to
choose among compatible embedded families. An unselected or unavailable family
uses the integration's semantic fallback where allowed, or produces an explicit
unsupported-AOT diagnostic; it never receives a host-family artifact.

The selection also carries a delivery policy:

```text
ArtifactDeliveryPolicy =
    SelectedFamilies([AppleArtifactFamily], RequiredWhenTargetMatches)
  | FallbackOnly
```

For each selected family, successful expansion embeds its payload under the
family's governed consumer-target `#[cfg]`. If that family cannot be built on
the macro host, expansion emits the retained toolchain/compiler diagnostic as a
`#[cfg]`-gated `compile_error!` item and emits the semantic fallback for
nonmatching targets. Thus a Linux host building Linux can use the same portable
source without Metal, while a Linux host cross-building a selected macOS family
gets a deterministic unsupported-cross-AOT error. The proc macro does not need
to observe the consumer target to make either decision.

An unselected family intentionally uses fallback. `FallbackOnly` is a valid
explicit profile and performs no backend compiler work. A frontend may expose a
separate explicit “acceleration required” policy, but it cannot silently turn a
selected-family build failure into fallback on the matching target. The mapping
from family to consumer `cfg` predicate is versioned Tiler data and covered by
generated-code tests.

Platform policy, SDK, deployment target, and Metal language version participate
in artifact identity. No target is silently inferred from the proc-macro host
when that would produce an incompatible artifact.

Cargo documents `TARGET` and `CARGO_CFG_*` as build-script inputs rather than
ordinary crate-compilation variables. Local measurement also found them absent
from native and explicitly targeted proc-macro expansion; see the
[proc-macro environment research](../research/macro-environment/proc-macro-build-environment.md)
and
[Cargo environment variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html).

Changing Xcode, the selected developer directory, SDK contents, or explicit
Tiler toolchain configuration is a rebuild boundary. On an actual expansion,
the resolved compiler fingerprint changes the cache key. Stable Cargo does not
track those external changes, so users and CI must force the affected consumer
crate to rebuild after a toolchain change. Cache deletion alone does not affect
already generated Rust or compiled binaries.

## Rust-analyzer and `cargo check`

The macro may be expanded by rust-analyzer and by non-codegen Cargo commands.
The architecture does not depend on undocumented IDE environment variables.
Instead:

- content hashing and cache hits must be cheap;
- one unique cold expansion may compile once;
- warm IDE and `cargo check` expansion must avoid `xcrun`;
- emitted types and fallback behavior remain identical across analysis/codegen;
- an optional analysis stub is considered only if measurements demonstrate a
  material problem and it can preserve type/diagnostic behavior.

Cold/warm IDE behavior remains a useful performance measurement. Correctness
does not depend on it: expansion has identical types, diagnostics, artifact
selection, and fallback semantics in every compiler process.

## Fusion visibility boundary

A proc macro can optimize only semantics visible inside its invocation or
generated internally by that frontend. One einops expression can fuse the
multiple Candle operations it would otherwise lower into. It cannot see later
independent Rust method calls:

```rust
let a = einops!("...", x);
let b = a.gelu(); // not visible to the previous invocation
```

Wider fusion therefore requires an inline region frontend, for example:

```rust
let y = tiler! {
    let a = einops("b h w c -> b c h w", x);
    reduce_sum(gelu(a + bias), [h, w])
};
```

This preserves inline DX while making the whole fusion region explicit. Cross-
invocation whole-program fusion would require a compiler plugin or runtime
graph/JIT and is not claimed.

## Fallback contract

Generated execution is an optimization of an available semantic computation.
The expansion includes a fallback that runs when the target backend is absent
or no compiled plan applies. It preserves output shape, dtype, numerical
contract, and autograd behavior. Existing Candle operations are valid only
where those contracts match.

For an explicitly environment-parameterized program, generated compiled and
fallback expressions share one typed semantic root-binding environment. A
frontend may not substitute a conventional value when an admitted target
property is unavailable or let fallback observe a different value. If neither
path can bind the declared semantic interface, execution returns a typed
interface/binding error rather than silently changing the computation.

Artifact compilation errors for a selected family are compile-time errors when
the consumer target matches that family rather than silent fallback; otherwise
broken generated code could ship unnoticed. Family-scoped `cfg` delivery keeps
the same invocation portable to unrelated fallback targets. Runtime
applicability misses may use fallback before custom-op application as described
in [Candle integration](candle.md).

## Feasibility evidence and remaining vertical checks

Completed bounded measurements establish:

1. the immutable cache protocol survives concurrent writers, nine killed-writer
   phases, corruption, deletion, unavailable roots, and reader/eviction races;
2. direct byte literals have measured initial size/count gates and repeated
   identical bundles cannot rely on linker deduplication;
3. Cargo no-op builds skip expansion, consumer or macro edits rerun it, and
   cache/toolchain changes alone do not invalidate an otherwise fresh expansion;
4. the qualified Metal toolchain compiles distinct macOS, iOS-device, and
   iOS-simulator payload families; and
5. Metal library load, function lookup, and pipeline creation are distinct
   runtime failure stages.

The first vertical implementation slice must still demonstrate an actual Tiler
macro compiling, embedding, loading, and dispatching a one- and multi-entry
bundle; a production warm cache hit invoking no `xcrun`; source-spanned retained
MSL diagnostics; and the non-Apple semantic fallback path without consumer
setup. rust-analyzer cold/warm performance also remains unmeasured because the
component was unavailable. None of these gaps changes the accepted contract,
but they must not be reported as completed feasibility.

## Traceability

This document owns frontend translation and the inline proc-macro delivery
profile, not consumer runtime execution. Its accepted decisions and measured
macro, cache, and embedding boundaries are linked in frontmatter.

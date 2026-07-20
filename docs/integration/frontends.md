# Frontend and proc-macro integration

**Status:** proposed integration direction; feasibility measurements remain

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

External compiler failures become `compile_error!` diagnostics associated with
the invocation span. Debug configuration may retain canonical MSL and tool
diagnostics under the cache entry.

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

Embedding cost must be measured at representative 10 KiB, 100 KiB, and 1 MiB
bundle sizes. Linker merging of identical byte literals is not assumed. Binary
deduplication may later use platform/linker mechanisms without changing macro
syntax or artifact semantics.

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
override supports CI and sandboxed builds. Cache entries use cross-process
locking, temporary files, validation, and atomic rename. Identical invocations
share external compiler work even when expanded in different rustc processes.

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

Artifact compilation errors on a supported native Metal build are compile-time
errors rather than silent fallback; otherwise broken generated code could ship
unnoticed. Runtime applicability misses may use fallback before custom-op
application as described in [Candle integration](candle.md).

## Initial feasibility gates

Before broad IR implementation, prove:

1. cold macro expansion compiles and embeds a loadable metallib;
2. warm equivalent expansions do not invoke `xcrun`;
3. concurrent identical expansions compile once and never observe partial data;
4. direct byte literals have acceptable rustc time/memory at representative
   sizes;
5. repeated identical bundles have measured, understood binary-size behavior;
6. rust-analyzer and `cargo check` cold/warm behavior is usable;
7. cache deletion, `cargo clean`, and compiler/toolchain changes behave safely;
8. Metal diagnostics point to the macro invocation and preserve retained MSL;
9. native macOS and non-Apple fallback paths work without consumer setup;
10. one bundle can contain several entry points and a multi-step program plan.

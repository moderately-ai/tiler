---
schema: "tiler-doc/v1"
id: "tiler.contract.frontend-integration"
kind: "contract"
title: "Frontend and proc-macro integration"
topics: ["integrations", "frontends", "proc-macros", "aot"]
contract_status: "accepted"
implementation_status: "partial"
evidence: ["tiler.research.macro-environment.build-environment", "tiler.research.embedding.artifact-costs", "tiler.research.cache.crash-race-protocol", "tiler.research.cache.root-policy", "tiler.research.shapes.nightly-const-shape-parameters"]
ticket: "synthesize-artifact-contracts"
---

# Frontend and proc-macro integration

**Status:** accepted inline AOT contract; rust-analyzer performance remains unmeasured

`implementation_status` moved from `not-started` to `partial` on 2026-07-31, and the boundary of that word is narrow. What exists is the crate pair this contract's inline delivery routes through — `tiler` and `tiler-macros`, admitted by [ADR 0088](../decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md) — the approved region grammar `prototype-inline-proc-macro-frontend` delivered that same day, the symbol-binding and runtime-value boundary the section below describes, and the target policy's stating half: an expansion parses a region from real tokens, resolves it against the governed semantic operation registry, states a delivery policy — including the `deliver` statement a consumer writes — and validates it through the single canonical `ArtifactFamilySelection` constructor. Since `prototype-inline-aot-integration-proof` landed later the same day, a selected buildable family is *delivered*, not refused: the expansion compiles the region through `tiler_compiler::session`, runs the offline Metal driver through `tiler-build`'s cache-validated `accept_or_publish_metal_plan` — the crate-private cache-root resolver is now that path's live input — and embeds the produced envelope as one byte-string literal under the family's governed `#[cfg]`, with guarded pre-commit selection in the consumer binary stopping at the first question only a device can answer (`route-an-embedded-artifact-through-a-consumer-storage-seam` owns the dispatch seam). A family the one authoritative declaration cannot build, a symbolic-extent region, and a disabled cache each refuse with a spanned error rather than silently falling back. What is *not* implemented: the device dispatch itself, multi-family delivery from one expansion, and every capability the compiler refuses — a refused compilation is still an unconditional `compile_error!`. Examples elsewhere in this document describe the contract; the delivered path is the one this paragraph names.

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

## Rust toolchain contract

ADR 0067 requires the exact governed nightly for Tiler crates and Rust
consumers because generated types may contain dependent array const parameters.
The initial pin is `nightly-2026-07-19`; a rolling `nightly` channel is not
supported. An inline macro must emit the same canonical `StaticShape<RANK,
EXTENTS>` expansion as an ordinary Rust frontend.

Using nightly for that public type does not authorize unstable proc-macro APIs.
Expansion, artifact embedding, diagnostics, and external-input behavior retain
their accepted stable proc-macro contracts. A compiler-pin migration must pass
the retained shape-evidence and proc-macro call-site conformance cases before
the governed pin changes.

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

## Symbol binding and the runtime-value boundary

Step 2 above needs two things the region text alone does not supply: what a symbolic extent's value *is* at runtime, and what kind of value the invocation was handed. Both are decided, and Tom ratified the shape of each on 2026-07-30 under `define-inline-symbol-binding-and-runtime-value-adaptation`. The exact items below are a reviewed draft under [ADR 0075](../decisions/0075-scope-public-boundary-approval-by-change-category.md) and ADR 0074 convention 7 until Tom accepts them.

**`sym n;` means operand unification.** One `sym` statement declares one logical extent variable. Its runtime value is unified from every operand dimension that names it: at least one occurrence must source it, and every additional occurrence owes an equality against the first checked value. Expansion picks the canonical source by the canonical order of interface keys and axes rather than by which occurrence was written first, so reordering the `in` list moves nothing the graph identifies. Unbound and undeclared symbols, a repeated declaration, and a result count past the bounded profile are typed refusals carrying the span of the token that caused them.

The binding itself is stated in the promoted `ShapeEnv` vocabulary and nowhere else: a symbol is a `ShapeSymbol`, its value is a `RootBinding` over `BindingSource::InputDimension { input, axis }` at `LiveDevicePreflight` with `RuntimeValidated` provenance, and that binding names an *interface key*, which is what makes graph identity independent of declaration order. An additional occurrence is deliberately not a second binding — [ADR 0008](../decisions/0008-typed-root-bindings.md) gives each symbol exactly one root binding and the environment rejects a second — so it is carried beside the environment as a runtime equality obligation.

**A runtime value is an opaque wrapper over a consumer-supplied adapter.** The facade owns `tiler::value`, and it names no Candle, Metal, or other consumer type, lifetime, storage layout, allocation policy, or device object. An integration implements `TensorAdapter`, naming its own value, context, and error as associated types, and supplies three capabilities: read-only metadata (stored scalar and extents), a support predicate over the capabilities a region requires, and construction of one result. The adapter travels in the wrapper's type parameter, so there is no global registry and no adapter argument at the call site; the integration owns the conversion in, and gets its own value back out, because a region's result is the adapter's value rather than a wrapper.

Element type is `tiler_ir::program::StorageScalar`, re-exported rather than restated: the scalar a storage position holds already has one authority, and a facade-local copy would leave the correspondence between what an expansion decides and what the facade means held only by the text of the emitted tokens. That re-export is why the facade depends on `tiler-ir`.

**What an invocation refuses at runtime,** each typed and each naming the operand and axis a consumer wrote: an operand count the region did not declare, a capability the adapter does not offer, a rank or stored scalar the region did not declare, and two axes naming one symbol that report different extents. The adapter's own error is carried rather than flattened. Emitted facts that disagree with themselves are a typed refusal too, reported as a defect in the expansion rather than a panic in the consumer's process.

**Deliberately absent.** No storage access — no pointer, buffer, byte slice, or device object — because nothing dispatches yet and a storage surface with no caller would be an unreviewed boundary. The dense row-major storage property a first dispatch profile requires is stated as an adapter *capability* instead, so an adapter that cannot offer it is refused now rather than read wrongly later. Per-value storage properties, more than one result, and operands on different contexts are outside the bounded profile and reject explicitly.

## Direct byte embedding

The generated code conceptually contains the shape below. **It is illustrative and not delivered:** no expansion emits any of it today, and the item names under `__private` are placeholders for a surface that does not exist.

```rust
{
    static MANIFEST: &[u8] = b"...";
    static METALLIB: &[u8] = b"...";

    ::tiler::__private::execute_or_fallback(
        ::tiler::__private::EmbeddedBundle::new(MANIFEST, METALLIB),
        /* tensors and fallback */,
    )
}
```

**Every path a generated token spells resolves through `tiler`.** That is the settled property, decided by Tom on 2026-07-31; the exact items are not. A procedural macro has no `$crate`, so its expansion must spell an absolute path, and an earlier revision of this example spelled `::tiler_candle::` and `::tiler_artifact::` — which would hand a consumer a dependency it never declared and would break its build on crates it cannot see. Routing through the facade's `#[doc(hidden)] pub mod __private` is what makes "generate only paths reachable through the consumer's declared `tiler` dependency" true rather than intended.

The facade's generated paths today are `::tiler::__private::RegionFacts` and `::tiler::__private::bind_and_build` — the block one region expands to declares its facts as a block-local constant and calls that one entry point, which Tom accepted on 2026-07-31 when `prototype-inline-proc-macro-frontend` delivered the grammar (the earlier inert `expansion_anchor` existed only so the re-export was compiler-checked before a grammar existed, and was removed with it). The region-facts vocabulary behind them — `RegionFacts`, `OperandFacts` (whose per-axis `extents: &[OperandExtent]` of `Literal(u64) | Symbolic` replaced the original bare `rank` on 2026-08-01, so a declared literal extent is checked against the supplied value's rather than trusted, refusing as `tiler.bind.literal-extent-mismatch`), `OperandExtent`, `SymbolFacts`, `AxisRef`, `ResultFacts`, `ResultAxis`, `bind_region`, and `build_result` — was delivered by `define-inline-symbol-binding-and-runtime-value-adaptation` and is described in the symbol-binding section above. `promote-artifact-family-selection-for-the-frontend` reviewed its own question and answered *none*, because no generated token names a selection type.

The facts are data, not a rebuilt environment. An expansion constructs a real `ShapeEnv` in the proc-macro process and decides everything decidable there; what survives into tokens is the residue of that decision, so a consumer's runtime re-derives nothing the host already concluded. The compile-pass fixtures under `crates/tiler/tests/facade/pass/` compile the emitter's exact output as separate out-of-tree crates, each supplying its own adapter, and the macro crate's tests read those files so the two ends cannot drift apart.

The actual proc-macro implementation should construct byte-string literal
tokens directly rather than emit millions of integer tokens. No generated path
or `include_bytes!` dependency is required. The completed Rust expansion is
self-contained; the compiler cache can be deleted without affecting an already
compiled binary.

**Fact — what "self-contained" was measured to be, on one recorded host.** [The self-contained embedding note](../research/embedding/self-contained-embedding.md) demonstrates the property this contract asserts, rather than arguing it: a consumer built and ran with every Tiler-produced artifact and the entire expansion-cache root deleted — sixteen files, each deletion proved against a path that held files beforehand — from a crate declaring no dependency at all, with the payload travelling as exactly one byte-string literal. The distinction the evidence turns on is that the artifact and the cache are inputs to the *expansion* and never to the *expanded code*: the artifact must exist the first time a build expands an invocation, and is needed by nothing afterwards. That is a bounded measurement on the note's exact host, toolchain, and artifact population, describing the invariant ADR 0004 already accepts — it is not a portable guarantee, and the note's section 7 states what it does not cover.

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

A default macOS user cache is used rather than consumer `OUT_DIR`. A documented override supports CI and sandboxed builds. [ADR 0089](../decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md) fixes the exact derivation, accepted on 2026-07-31: an expansion reads `TILER_EXPANSION_CACHE_DIR` and, only when that is unset, `$HOME`, and reads nothing else. A stated override is the root **verbatim** — Tiler appends nothing to it — except for the exact value `off`, which expands with no cache at all; otherwise the root is `$HOME/Library/Caches/ai.moderately.tiler/expansion`. Precedence is override-first and total, so a stated override decides the root whether or not the default would have worked, and an empty override is a refusal rather than an absence. A root that is relative, that lies at or under `/tmp`, `/private/tmp`, `/var/tmp`, `/private/var/tmp`, or `/Users/Shared` — the trees macOS makes writable by every user of the machine, which the cache's privacy requirement forbids — or that cannot be derived because `HOME` is unset or empty is a typed refusal at the invocation naming the offending input and both remedies, never a silent miss and never a second location. `$TMPDIR` is per-user on macOS and stays usable, which is what makes that refusal affordable in CI. The root is deliberately not part of the key above: moving it changes where entries live and never what they mean.

One immutable self-validating bundle is stored per complete key. A miss uses a
stable per-key OS advisory lock, locked recheck, create-new same-filesystem
temporary file, full temporary validation, and atomic rename. Readers validate
every hit without taking the lock. Identical invocations share external compiler
work even when expanded in different rustc processes.

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

That grammar has exactly one canonical encoder, `tiler_metal_aot::family`, and a
frontend states a policy by validating it through
`ArtifactFamilySelection::new`. Canonical family ordering, duplicate and empty
refusal, the per-family deployment minimum and Metal language standard, and the
selection's identity bytes are that module's; a frontend that restated any of
them would be a second authority over one subject. Tom accepted the surface and
its placement on 2026-07-31 under `promote-artifact-family-selection-for-the-frontend`.

The frontend edge to that module belongs to the proc-macro crate, not to the
consumer-facing facade. A `proc-macro` crate and its dependencies are built for
the host and never enter a consumer's target build graph, so the macro crate can
hold an edge to a process-spawning Apple toolchain driver at no cost to a
consumer; the same edge on the facade would compile that driver into every
consumer on every platform and would publish Apple backend policy on a
consumer-neutral boundary. Nothing a consumer writes needs the type: a policy is
stated in region syntax, and generated tokens name `#[cfg]` predicates and byte
literals.

### The accepted spelling

Tom accepted the consumer-visible spelling on 2026-07-31 under [`accept-the-inline-artifact-family-profile-syntax`](../../tickets/accept-the-inline-artifact-family-profile-syntax.md), which closed Q-ART-008. An inline region states its delivery policy with a `deliver` statement in the declaration block beside `sym` and `in`, at most once, in either of two productions:

```text
sym n;
in a: f32[n], b: f32[n];
deliver macos-and-ios;        // a named profile
out a * b
```

```text
sym n;
in a: f32[n], b: f32[n];
deliver macos 14.0, ios 17.0; // a family list, when a floor must be stated
out a * b
```

The profile vocabulary is `fallback-only`, `macos`, `ios`, and `macos-and-ios`; a profile fixes every family it names to that family's governed floor for the Metal language standard Tiler compiles with, which is why it publishes no version. The family-list vocabulary is `macos` and `ios`, each with a `<major>.<minor>` deployment minimum, and `ios` covers the iOS device and the iOS simulator together — a name covering only the device would leave every simulator build silently on the fallback path. The two productions share one vocabulary rather than defining two: a list stating the governed floors resolves to the identical selection as the profile that names the same families. A minimum below the governed floor is the driver's own typed refusal, reported at the version token that stated it.

**Stating nothing is `fallback-only`.** The statement's absence resolves to the same explicit policy, so a region written without it is unchanged token-for-token, and no consumer is required to state a policy to get one.

The statement is where the "ergonomic literal default profile" this contract permits actually lives, and the list is what keeps that affordable: the profile names publish no Apple deployment-minimum vocabulary on the mandatory path, and a consumer whose own floor is higher can still state it without waiting for a second profile to be minted. The second axis this surface reserves — a separate explicit "acceleration required" policy — remains statable as its own statement in the same block. It is not `#[tiler::deliver(macos)]`: a `#[proc_macro]` cannot see attributes outside its own token stream, so an attribute form would need a second macro entry point and would break the accepted "each invocation is a self-contained AOT and embedding unit".

**A statement selecting a family is refused today, and the refusal is the contract working.** No expansion runs the offline driver yet — the compiler boundary admits none of the frontend's multi-input elementwise programs, which [`admit-multi-input-elementwise-programs-at-the-compiler-boundary`](../../tickets/admit-multi-input-elementwise-programs-at-the-compiler-boundary.md) owns — so there is no compiled payload for a selected family to deliver. Expansion therefore fails closed with a spanned `compile_error!` at the `deliver` keyword naming the selected families, rather than emitting the semantic fallback: a selected family is *required* when the consumer target matches it, and a quiet fallback there is exactly what this contract forbids. `deliver fallback-only;` and the statement's absence are consequently the only spellings an expansion completes, and the compile-pass and compile-fail fixtures under `crates/tiler/tests/facade/` pin both halves.

**One envelope, N payloads** (Tom, 2026-07-25). A selection naming several families produces one artifact carrying one payload per built family, so the whole selection has one identity and a partial delivery is impossible by construction. The bytes are therefore embedded **once and unconditionally**, and what the family's governed consumer-target `#[cfg]` gates is the *position* of that family's payload within the envelope the consumer already holds — not the bytes. An earlier revision of this sentence said expansion "embeds its payload under the family's `#[cfg]`", which describes one artifact per family and is superseded by that decision; the accepted cost of the current shape is that a consumer needing one family carries the bytes for all of them. The selector is total by construction — one arm per built family plus a `not(any(…))` arm — so an overlapping or missing predicate is a build error in the consumer's own compilation rather than a wrong payload.

If a family cannot be built on the macro host, expansion emits the retained
toolchain/compiler diagnostic as a `#[cfg]`-gated `compile_error!` item and emits
the semantic fallback for nonmatching targets. Thus a Linux host building Linux
can use the same portable source without Metal, while a Linux host
cross-building a selected macOS family gets a deterministic unsupported-cross-AOT
error. The proc macro does not need to observe the consumer target to make
either decision.

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
let y = tiler::tensor! {
    let a = einops("b h w c -> b c h w", x);
    reduce_sum(gelu(a + bias), [h, w])
};
```

`tiler::tensor!` is the ratified public path, fixed by Tom on 2026-07-30 and recorded in [ADR 0088](../decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md); an earlier revision of this example spelled the macro `tiler!`, which was an illustrative spelling rather than a second decision. The region body above is still illustrative, but for a narrower reason than it once was: `tensor!` now has a grammar — a declaration block of `sym`, `in`, and `deliver` statements followed by one `out` expression — and that grammar admits neither a `let` binding, nor a named operation call such as `einops(…)` or `gelu(…)`, nor any operator beyond `*` and `+`. Each of those is refused at the token that spells it, with the named-call form reserved rather than filled, because the governed semantic profile registers no operation without an operator spelling.

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

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

**Status:** accepted inline AOT contract; one macOS family delivers end to end on one measured host, multi-family delivery is parked on a second measured Apple family, and no non-macOS host has been measured

`implementation_status` moved from `not-started` to `partial` on 2026-07-31, and the boundary of that word is narrow. What exists is the crate pair this contract's inline delivery routes through — `tiler` and `tiler-macros`, admitted by [ADR 0088](../decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md) — the approved region grammar `prototype-inline-proc-macro-frontend` delivered that same day, the symbol-binding and runtime-value boundary the section below describes, and the target policy's stating half: an expansion parses a region from real tokens, resolves it against the governed semantic operation registry, states a delivery policy — including the `deliver` statement a consumer writes — and validates it through the single canonical `ArtifactFamilySelection` constructor. Since `prototype-inline-aot-integration-proof` landed later the same day, a selected buildable family is *delivered*, not refused: the expansion compiles the region through `tiler_compiler::session`, runs the offline Metal driver through `tiler-build`'s cache-validated `accept_or_publish_metal_plan` — the crate-private cache-root resolver is now that path's live input — and embeds the produced envelope as one byte-string literal under the family's governed `#[cfg]`. Since `route-an-embedded-artifact-through-a-consumer-storage-seam` landed on 2026-08-01, the consumer binary no longer stops at the first device question: an integration implementing `tiler::value::DispatchAdapter` hands the region's storage to its own device authority, the one-way commit stays inside `tiler_runtime::adapter::route_with_adapter`, and a post-commit failure surfaces as `BindError::DispatchFailed` rather than the fallback's value; a consumer without a dispatch adapter still takes the semantic fallback before the commit, exactly as before. The composed path executed on hardware the same day: `spikes/runtime/inline-dispatch` reaches a completed dispatch on one Apple M4 Max under the labelled producer-declared-equality diagnostic, with the result checked bit-for-bit against the consumer's own arithmetic — a measurement bound to that host, not a portable guarantee. Since 2026-08-04 that path carries a *bundle*: a region whose selected plan needs two executable entries packages both into the one embedded artifact and dispatches them in the declared order on the same host, with the entry count asserted from the consumer's side and a back-to-front reordering watched returning a wrong answer rather than a refusal — the **Landed** list below carries the citation and the one shape it is measured at. A family the one authoritative declaration cannot build, a symbolic-extent region, and a disabled cache each refuse with a spanned error rather than silently falling back. What is *not* implemented: multi-family delivery from one expansion, a facade-reachable answer to live-device route requirements (a consumer cannot name `tiler-metal`'s GPU-family vocabulary, so any region declaring that row is refused fail-closed), and every capability the compiler refuses — a refused compilation is still an unconditional `compile_error!`. Examples elsewhere in this document describe the contract; the delivered path is the one this paragraph names.

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

**A runtime value is an opaque wrapper over a consumer-supplied adapter.** The facade owns `tiler::value`, and it names no Candle, Metal, or other consumer type, lifetime, storage layout, allocation policy, or device object. An integration implements `TensorAdapter`, naming its own value, context, and error as associated types, and supplies three capabilities: read-only metadata (stored scalar and extents), a support predicate over the capabilities a region requires, and construction of one result. Those three are the whole of the obligation for a region that dispatches nothing; a delivering region asks for the storage itself, and asks through a second trait rather than through more methods here — see **Storage access** below. The adapter travels in the wrapper's type parameter, so there is no global registry and no adapter argument at the call site; the integration owns the conversion in, and gets its own value back out, because a region's result is the adapter's value rather than a wrapper.

Element type is `tiler_ir::program::StorageScalar`, re-exported rather than restated: the scalar a storage position holds already has one authority, and a facade-local copy would leave the correspondence between what an expansion decides and what the facade means held only by the text of the emitted tokens. That re-export is why the facade depends on `tiler-ir`.

**What an invocation refuses at runtime,** each typed and each naming the operand and axis a consumer wrote: an operand count the region did not declare, a capability the adapter does not offer, a rank or stored scalar the region did not declare, an extent the region fixed literally that the supplied value does not report, and two axes naming one symbol that report different extents. A delivering region owes two more, both of which arrived with the storage seam below: a value whose byte run is not the length its own reported extents describe, and a committed route that did not complete — the second reported rather than replaced by the fallback's value, because ADR 0051 permits no fallback after the commit. The list is exactly the variants of `BindError` in `crates/tiler/src/value.rs`. The adapter's own error is carried rather than flattened. Emitted facts that disagree with themselves are a typed refusal too, reported as a defect in the expansion rather than a panic in the consumer's process.

**Storage access, and why it is owed only where it is used.** This paragraph read "No storage access — no pointer, buffer, byte slice, or device object — because nothing dispatches yet and a storage surface with no caller would be an unreviewed boundary" until this commit, and the reason expired before the sentence did: `route-an-embedded-artifact-through-a-consumer-storage-seam` landed the caller on 2026-08-01, which the status paragraph at the top of this document already records. What exists is a *second* trait rather than three more methods on `TensorAdapter` — `DispatchAdapter: TensorAdapter` in `crates/tiler/src/value.rs` adds `storage` and `storage_mut`, which borrow one value's dense row-major byte run, and `dispatcher`, which turns a `RegionRequest` into the integration's own `RuntimeAdapter`. A region's storage travels in that request: `RegionRequest::operand` answers a byte run by interface key, and `RegionRequest::result_mut` hands over the run a dispatch writes into. The split is the point of the shape rather than an accident of it — only `bind_route_and_build` in `crates/tiler/src/route.rs` is bounded by `DispatchAdapter`, so a consumer whose regions are all `fallback-only` calls `bind_and_build`, writes no byte accessor, and names no device authority. `AdapterCapability::DenseRowMajorStorage` deliberately did not move onto the new trait, because it is a claim an adapter may state and a region may check without dispatching; what the seam added is the surface that *reads* the storage that claim describes, together with the check that the claim holds of each value — `BindError::StorageLengthMismatch` refuses a value whose reported extents and reported bytes disagree, applied by `route::checked_length` before any byte reaches a kernel.

**Still absent at this commit, each stated as a check a reader can run.** No raw pointer: every accessor above hands over a bounds-carrying slice, and `grep -n '\*const\|\*mut\|NonNull\|as_ptr\|from_raw' crates/tiler/src/value.rs crates/tiler/src/route.rs crates/tiler/src/expansion.rs` matches nothing. No device object, memory domain, allocation, or consumer type: `TensorAdapter` names the integration's value, context, and error as associated types and nothing else, `ResultRequest` carries a stored scalar and extents while naming no allocation policy, memory domain, or device, a result is constructed by the integration's own `TensorAdapter::build`, and the facade holds no device at all — `dispatch_embedded_route` reaches one only through the adapter the integration built. No per-value storage property: `ValueMetadata` carries the stored scalar and the extents, so density remains a claim about the *adapter* rather than about one value; moving it onto the metadata is the widening path, and nothing yet consumes the distinction.

**What the bounded profile still refuses, and one thing it does not.** More than one result is a typed refusal at the declaration that crosses the bound — `RegionBindError::UnsupportedResultCardinality` in `crates/tiler-macros/src/binding.rs` — and a region requiring a capability the adapter declines is `BindError::UnsupportedCapability`, raised by `bind_region` before any shape is compared. Operands on different contexts are outside the profile too, but the sentence replaced above said they "reject explicitly" and they do not: `bind_and_build` and `dispatch_embedded_route` both construct through the context of the *first* declared operand and compare no other, which `build_result`'s own documentation states as its contract. Nor is it a check this boundary could make as it stands — `TensorAdapter::Context` carries no bound under which two contexts could be compared — so the accurate word is unhandled, and narrowing it would take either a bound on that associated type or an explicit placement contract.

**All of this storage surface belongs to the reviewed draft this section opened with, not to an accepted boundary.** `DispatchAdapter`, `RegionRequest`, and `RegionOperand` are `pub` because the seam only works if a crate outside `tiler` can implement it, which is the same reason the rest of `tiler::value` is; `crates/tiler/src/value.rs`'s own module documentation says so, and none of it is an accepted public facade until Tom accepts the exact interface.

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

**A delivering region emits a second record, and every field of it is a producer declaration rather than a host observation.** `::tiler::__private::RouteFacts` carries the artifact bytes and the `#[cfg]`-resolved payload position alongside the canonical artifact identity, the governed target profile key and exact descriptor, the backend family, the executable representation, and — since `declare-host-dtype-dispatchability-at-the-consumer-boundary` landed on 2026-08-06 — the dtype-dispatchability rows the selected target profile declares. The first four are read off the verified artifact program the expansion just assembled. The dtype rows have no counterpart in the envelope, because an artifact declares no dispatchability, so they are read from `tiler_build::BoundMetalCompileDeclaration::dtype_dispatchability_rows` — the same `TargetProfile` the compile gate consulted, answered at `AvailabilityPhase::CompileProfile`. Only exact declarations are emitted: a dtype the profile resolves `Unknown` or `Deferred` produces no row, and a row a host never receives is a dtype it never claims.

**The inline-region path cannot earn a host-earned dtype row at all, and that is structural rather than pending.** `tiler::route::execution_environment` builds the `ExecutionEnvironment` it hands to `DispatchAdapter::dispatcher`, so the environment must exist *before* the integration's adapter does — there is no point on this path at which a device could be consulted, and the facade holds none of its own in any case. Emitting the rows therefore removed a call-site literal standing in for a declaration; it did not make the loader's dtype comparison non-tautological on this path, any more than `ExecutionEnvironment::classify` is when a host restates the artifact's own profile. The one place a host-earned row can arise is the integration's `RuntimeAdapter::bind_execution_context`, which answers with the environment the route is actually settled against and holds the device the facade does not: an adapter returning `RegionRequest::declared_environment` verbatim has chosen producer-declared equality for the dtype rows along with everything else, and `tiler::__private::PRODUCER_DECLARED_EQUALITY` is the label it must report beside the result. Under [ADR 0086](../decisions/0086-require-attributable-or-attested-native-translation.md) no macOS host can earn the applicability receipt that would let it offer the profile at all, so no adapter in this workspace states one today. What emission does buy is that the fact is load-bearing at the boundary rather than assumed there: `crates/tiler/tests/facade/pass/inline_region_refuses_an_undispatchable_dtype.rs` routes one region twice through one adapter, differing only in whether the region's own dtype row is stated, and the withheld run is refused before the payload is looked at.

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

The cache trims itself, and a consumer who configures nothing gets that. Tom decided on 2026-08-04 that eviction is automatic, configured by environment variables, with no maintenance command shipped. An expansion reads one further variable, `TILER_EXPANSION_CACHE_MAX_ENTRY_AGE`, and it names the *entry age* rather than the cache for the reason ADR 0089 names the directory precisely: the typed collection bound carries a byte and an entry ceiling too, and a generic spelling would have to change meaning the day one of those becomes configurable. Unset states `tiler_cache::expansion::MaxEntryAge::DEFAULT` — thirty days, a documented product choice under that decision and explicitly not a measurement, whose ground is stated on the constant itself. The exact value `off` — the same word `TILER_EXPANSION_CACHE_DIR` uses to disable, deliberately, so one environment surface has one word for "do not" — is the opt-out and removes nothing ever. Any other value is a whole number and exactly one lowercase unit suffix from `s`, `m`, `h`, `d`: `45s`, `90m`, `12h`, `30d`. There is no compound form and no unsuffixed count; `30` alone is refused rather than assigned a unit, because that is the ambiguity that removes thirty days of compiled artifacts from someone who believed they had written thirty days. Only the age is configurable: the two aggregate ceilings select by publication recency, which can evict a working set a build is still using, and no working-set measurement exists to justify offering them.

A value that cannot be read is a typed refusal **of the eviction**, never of the build. The expansion compiles, publishes, and embeds exactly as it would have, nothing is removed, and no bound is guessed — an age too large to represent is refused rather than saturated, because a saturated age would silently mean `off`. The refusal is attributable: one line on the expanding process's standard error, naming the variable, the offending value, the accepted spellings, the opt-out, the default, and the fact that nothing was removed, at most once per process. Cargo forwards that line to the terminal for a build that expands; a fully warm build runs no macro and prints nothing.

The resolved root is probed in that same shape, and for that same reason it never refuses. Before an expansion uses the cache, and at most once per build process, the frontend asks the root whether it can do what the publication protocol below rests on: one filesystem under the whole root, so a publication's rename does not cross devices; a create-new that refuses a path already there, which is what makes a temporary file one expansion's own; an advisory lock that excludes a second holder on this host; a rename that publishes an entry over whatever was there; and a modification time on a written file, which is what the eviction orders entries by. A root that answers for all five is silent — a healthy cache is not announced on every crate. One that does not produces one line on the expanding process's standard error, attributed to the macro that wrote it, naming the root and every property that did not answer, what it costs, and what to change. A property the filesystem **refuted** and one the probe **could not run** are reported as different things, because they have different remedies: an unrunnable probe most often means the root is not writable rather than that the filesystem is unsuitable. The expansion is never refused over any of it — every cache operation already fails closed on its own, so an unsuitable root costs repeated compiler work and never a wrong artifact, and failing a build over it would turn an optional accelerator into a correctness dependency. `TILER_EXPANSION_CACHE_DIR=off` has no root, probes nothing, and does not spend the process's one probe, so a process that resolves `off` for one crate and a real root for the next still reports the real one. The amortization is the eviction's rule with the eviction's per-driver meaning — at most once per `rustc` process under Cargo, once per proc-macro server session under rust-analyzer — and its bound is the process rather than the root: a process expanding under two different real roots probes the first and says nothing about the second. The reach of the line is the eviction refusal's.

The eviction runs **after a successful publication and nowhere else** — never on a hit, never inside the cache's own lookup, never on a `fallback-only` region, and never under `TILER_EXPANSION_CACHE_DIR=off`. Reaching a publication means the expansion has just run `metal` and `metallib` as external processes, so a directory scan rides on work far larger than itself. It is amortized further, and the rule is stated rather than probabilistic: **at most one pass per process**. Under Cargo the expanding process is `rustc`, one per crate compilation, so a build trims at most once per crate that published anything; under rust-analyzer it is the proc-macro server, one process for the editor session, so a session of thousands of expansions trims once. No clock decides that it is time, nothing is persisted in the cache root to record when the last pass ran, and no thread is spawned.

The collection's report is deliberately not surfaced. Automatic hygiene is silent, in the shape `cargo` and `sccache` set: no build-log line per eviction, no marker file, and never a compile error. What stands in for it is that the policy is readable back — an entry leaves only for the age the consumer's own environment states, the zero-configuration value is a documented constant, and `ExpansionCache::collect` remains public and returns the same report, naming every removed entry, its bytes, and which ceiling selected it, to any caller that wants the detail. A scan that fails is likewise not a build failure: the artifact is already published and correct.

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

Tom accepted the consumer-visible spelling on 2026-07-31 under [`accept-the-inline-artifact-family-profile-syntax`](../../tickets/accept-the-inline-artifact-family-profile-syntax.md), which closed Q-ART-008. An inline region states its delivery policy with a `deliver` statement in the declaration block beside `sym`, `in`, and `contract`, at most once, in either of two productions:

```text
sym n;
in a: f32[n], b: f32[n];
contract flush_subnormals_to_zero_f32;
deliver macos-and-ios;        // a named profile
out a * b
```

```text
sym n;
in a: f32[n], b: f32[n];
contract flush_subnormals_to_zero_f32;
deliver macos 26.0, ios 26.0; // a family list, when a floor must be stated
out a * b
```

The profile vocabulary is `fallback-only`, `macos`, `ios`, and `macos-and-ios`; a profile fixes every family it names to that family's governed floor for the Metal language standard Tiler compiles with, which is why it publishes no version. The family-list vocabulary is `macos` and `ios`, each with a `<major>.<minor>` deployment minimum, and `ios` covers the iOS device and the iOS simulator together — a name covering only the device would leave every simulator build silently on the fallback path. The two productions share one vocabulary rather than defining two: a list stating the governed floors resolves to the identical selection as the profile that names the same families.

**The numbers above are the driver's, not this surface's, and that is the part a reader must not mistake for a constant.** A profile publishes no version because it fixes every family it names to that family's governed floor for the Metal language standard Tiler compiles with; a family list lets a consumer state a higher floor of its own, and the driver checks it against the same governed table. **Fact — at this commit both families sit at 26.0, because the profile standard is MSL 4.0.** `DeliveredFamily::governed_minimum` in `crates/tiler-macros/src/delivery.rs` returns 26.0 for `macos` and `ios` alike, and `every_profile_family_sits_on_its_governed_language_floor` in `crates/tiler-macros/src/delivery/tests.rs` holds that restatement against the driver by requiring one minor version lower to be refused. Under MSL 3.1 the same two rows read 14.0 and 17.0, so a standard change moves those numbers without touching one line of the accepted spelling. **Any example anywhere in this corpus that spells a floor — including the family list above — is therefore dated evidence about a standard, never part of the spelling.** [ADR 0098](../decisions/0098-state-an-inline-regions-delivery-policy-with-a-named-profile-or-a-family-list.md) states this rule as its decision 2 and was accepted by Tom on 2026-08-05, so the rule above now carries the record's authority as well as the driver's governed table and the code that restates it.

**Stating nothing is `fallback-only`.** The statement's absence resolves to the same explicit policy, so a region written without it is unchanged token-for-token, and no consumer is required to state a policy to get one.

The statement is where the "ergonomic literal default profile" this contract permits actually lives, and the list is what keeps that affordable: the profile names publish no Apple deployment-minimum vocabulary on the mandatory path, and a consumer whose own floor is higher can still state it without waiting for a second profile to be minted. The second axis this surface reserves — a separate explicit "acceleration required" policy — remains statable as its own statement in the same block. It is not `#[tiler::deliver(macos)]`: a `#[proc_macro]` cannot see attributes outside its own token stream, so an attribute form would need a second macro entry point and would break the accepted "each invocation is a self-contained AOT and embedding unit".

**A statement selecting a buildable family delivers; four things still refuse, at three layers.** A `deliver` statement is checked by the region grammar, then by the delivery policy's one canonical constructor, then by the expansion's AOT stage — and each refusal lands on the token responsible rather than on the invocation, so a consumer with several regions is told which one it was:

- **Vocabulary and syntax, at the grammar.** An unknown profile name, an underscored near miss of an accepted one, a family name the consumer surface does not publish, an unsuffixed count where a `<major>.<minor>` belongs, a missing minimum, one family stated twice, and a second `deliver` statement. [`crates/tiler/tests/facade/fail/deliver_statement_diagnostics.rs`](../../crates/tiler/tests/facade/fail/deliver_statement_diagnostics.rs) states eight regions, each differing from an accepted one in exactly one token — those seven, plus a `deliver macos 13.0;` that belongs to the layer below — and its byte-compared golden records the caret column each refusal lands on.
- **A deployment minimum below the governed floor, at the driver.** `delivery::stated_delivery` validates every stated policy through `ArtifactFamilySelection::new`, so a minimum the governed table does not admit is `tiler_metal_aot::family`'s own typed refusal, reported at the version token — `deliver macos 14.0;` is refused because MSL 4.0 on macOS requires 26.0.
- **A symbolic-extent region under a selected family, at the AOT stage.** Selecting a family compiles the region ahead of time, which needs every extent known at expansion time; the refusal says so and names declaring literal extents or stating `fallback-only` as the two ways out. [`carry-symbolic-extents-into-the-semantic-program`](../../tickets/carry-symbolic-extents-into-the-semantic-program.md) is the work that removes the restriction.
- **An iOS family, at the AOT stage.** No measured Metal compile-time declaration exists for one, and a declaration is assembled from measured rows rather than widened by argument, so the refusal names macOS as the only family that has one. [`first-authoritative-ios-metal-compile-declaration`](../../tickets/first-authoritative-ios-metal-compile-declaration.md) is the work that measures a second; see **Parked** below.

The last three are checkable in one place. [`crates/tiler/tests/facade/fail/deliver_selects_an_artifact_family.rs`](../../crates/tiler/tests/facade/fail/deliver_selects_an_artifact_family.rs) states one region for each — `deliver macos;` over a `sym`-declared extent, `deliver ios;`, `deliver macos-and-ios;`, and `deliver macos 14.0;` — and its byte-compared golden carries all four refusals with the token each lands on, including the evidence that `macos` is *absent* from the list of what could not be built. Every one of them is a spanned `compile_error!` rather than the semantic fallback, which is the point: a selected family is *required* when the consumer target matches it, and a quiet fallback there is exactly what this contract forbids. The delivering half is pinned by [`crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs`](../../crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs), and the equivalence of `deliver fallback-only;` with the statement's absence by [`crates/tiler/tests/facade/pass/deliver_states_fallback_only.rs`](../../crates/tiler/tests/facade/pass/deliver_states_fallback_only.rs).

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

## Numerical contract statement

Every region states the numerical contract it computes under, with a `contract` statement in the declaration block beside `sym`, `in`, and `deliver`, at most once:

```text
sym n;
in a: f32[n], b: f32[n];
contract flush_subnormals_to_zero_f32;
out a * b
```

**There is no default, by decision.** Tom decided on 2026-08-01, at the live session, relayed through [`decide-the-inline-frontend-numerical-contract`](../../tickets/decide-the-inline-frontend-numerical-contract.md): a region states its contract in its own text rather than inheriting one. The reason is that the contracts are not settings on one meaning. Under the flushing-and-reassociating contract a reduction may be split or folded as a workgroup tree, so its result may differ from the flush-only reading in the last bits, and neither contract is stricter than the other in every respect — a frontend that picked one would be choosing what a consumer's program computes.

The vocabulary is `strict_f32`, `flush_subnormals_to_zero_f32`, `relaxed_f32`, `reassociate_f32`, and `flush_and_reassociate_f32`. Each names the `tiler_compiler::session::NumericalContract` constant of the same name, in lowercase, following the `strict_serial_sum` precedent. This frontend names contracts and does not compose them: a composed contract would be a meaning statable by a consumer and unknown to the artifact identity, the explain trace, and the cache key, all of which name the compiler's own contract key. The vocabulary widens exactly when the compiler publishes another constant.

Three refusals, at three different tokens, because they are three different mistakes:

- A region stating no `contract` is refused **at the invocation**, since no token is responsible for an absence, with a diagnostic naming the statement to add and listing the vocabulary.
- A statement naming something outside the vocabulary is refused **at the name**, with the vocabulary listed. Matching is exact — no case folding and no prefixes — so `FLUSH_SUBNORMALS_TO_ZERO_F32` is refused rather than folded onto the name beside it, because a name that is nearly a contract decides which results a program may return.
- A second `contract` statement is refused **at the second keyword**, by the grammar rather than by the vocabulary, because two statements would be two meanings for one region.

**Resolving a name is not admitting it, and the split is deliberate.** Every name above resolves; whether the delivered target can honour the contract is the compiler's own target feasibility question, reported at the `deliver` keyword with a typed reason naming the dimension. The measured Apple `f32` row flushes subnormals in every math mode, so a region stating `strict_f32` parses, means exactly what it says, and is refused later. Pre-answering that in the grammar would put a target fact where a second measured declaration would have to contradict it.

## Rust-analyzer and `cargo check`

The macro may be expanded by rust-analyzer and by non-codegen Cargo commands.
The architecture does not depend on undocumented IDE environment variables.
Instead:

- content hashing and cache hits must be cheap;
- one unique cold expansion may compile once;
- a warm expansion resolves the Apple toolchain and compiles nothing, in the IDE and under `cargo check` alike;
- emitted types and fallback behavior remain identical across analysis/codegen;
- an optional analysis stub is considered only if measurements demonstrate a
  material problem and it can preserve type/diagnostic behavior.

Cold/warm IDE behavior remains a useful performance measurement. Correctness
does not depend on it: expansion has identical types, diagnostics, artifact
selection, and fallback semantics in every compiler process.

### Why a warm expansion resolves the toolchain

The third bullet above read "warm IDE and `cargo check` expansion must avoid `xcrun`" until 2026-08-01. It was corrected rather than implemented, for a structural reason and a measured one; `avoid-toolchain-resolution-on-a-warm-expansion-cache-hit` carries the derivation and the numbers.

**The structural reason.** The compiler fingerprint is an *input* to the compilation identity, and that identity is a facet of the key deciding hit or miss. `Toolchain::prepare` must therefore observe the toolchain before a lookup exists to skip it. Reaching a cache entry without observing the toolchain would mean keying on something other than the compiler that would build a miss — which is the incomplete-key failure ADR 0050 exists to exclude.

**What the observation is worth, stated exactly.** Every `xcrun` invocation a resolution makes is answered from `xcrun`'s own on-disk cache. `xcrun(1)` documents it through `-n/--no-cache` and `-k/--kill-cache`, and `xcrun --verbose` names both the store (`$TMPDIR/xcrun_db`) and the keys: a tool lookup is keyed `<tool>|<SDK path>|<TOOLCHAINS>|<DEVELOPER_DIR>|`, an SDK field `<selector>|<DEVELOPER_DIR>|<sdklookup>|<field>`, and each resolved tool path additionally carries a `<toolchain-signature>` entry. Re-running these each expansion re-reads Apple's cache rather than observing the installed toolchain, and its invalidation rule is Apple's and is not documented. The part that *does* observe the toolchain that will run is the two direct executions of the resolved `metal` and `metallib` binaries to read their reported versions — not `xcrun` invocations at all.

So the invariant is narrower and stronger than "resolve every time": **identity must fold a fingerprint read by executing the binaries the same prepared token will execute.** `PreparedCompilation` already guarantees that structurally — it owns the resolved absolute paths and is consumed by the compilation. The consequence is what makes a stale observation harmless between processes: a stale fingerprint yields a stale key, and a stale key cannot collide with a fresh one, so no fresh build is ever served an entry keyed on a toolchain it did not resolve.

**Measurement — macOS 27.0 arm64, Apple M4 Max, `nightly-2026-07-19`, `rust-analyzer 1.97.0-nightly (8b03437a 2026-05-12)`, 2026-08-01.** One out-of-tree consumer crate declaring only `tiler`, holding one `deliver macos;` region, against a private per-user cache root, with an `xcrun` shim first on `PATH` logging every invocation:

| what | cost |
| --- | ---: |
| `Toolchain::resolve()` whole, warm | 44–97 ms (median 52–63, n=40×3) |
| — each of five `xcrun` calls | ~6 ms |
| — each of two direct `--version` executions | 10–16 ms |
| bare process-spawn floor | ~1.2 ms |
| one `xcrun --find` with `--no-cache` | ~3.33 s |
| warm `cargo check`, whole crate | 170–190 ms |
| live in-region edit, `semanticTokens` round trip | 137–217 ms (delivering) vs 10–16 ms (fallback-only) |

The table measures the code as it stood on that date, when a resolution made five `xcrun` calls. **A resolution now makes four.** `drop-the-unread-sdk-path-from-the-resolved-toolchain` removed the fifth — `--show-sdk-path` — on 2026-08-01: it populated an `SdkIdentity::path` that compilation identity excluded, the artifact payload did not carry, and no compiler or linker flag read, so it bought nothing and cost ~6 ms of every expansion. The removal moved no artifact identity and no cache subject, because the field was already outside both. Note that a warm `cargo build` still makes one `--show-sdk-path` call at link time — that one is rustc's own, and no Tiler change removes it. In a live rust-analyzer session every settled edit *inside* a region costs exactly one expansion, so the resolution is roughly 30–45% of that round trip — the largest single component, though Tiler's own optimize, emit, assemble, and cache-validation work is the remainder.

**Two designs were eliminated rather than deferred.** A cross-process cached fingerprint would be a second cache layered on `xcrun_db`, witnessing the same selection inputs Apple's cache already witnesses, and would need the full cache obligations — complete identity, validation on every hit, immutable entries, atomic publication, defined crash and race behavior — inside a crate whose dependency closure is pinned empty by ADR 0077 item 2 and which therefore owns no digest to key one with. A process-lifetime memo cannot help `cargo check` at all, because each check is a fresh process expanding once, and in a long-lived proc-macro server it would widen the window between observing a toolchain and executing it from one expansion to one session.

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

`tiler::tensor!` is the ratified public path, fixed by Tom on 2026-07-30 and recorded in [ADR 0088](../decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md); an earlier revision of this example spelled the macro `tiler!`, which was an illustrative spelling rather than a second decision. The region body above is still illustrative, but for a narrower reason than it once was: `tensor!` now has a grammar — a declaration block of `sym`, `in`, `contract`, and `deliver` statements followed by one `out` expression — and that grammar admits no `let` binding and no operator beyond `*` and `+`, with each refused at the token that spells it. The region shown above also states no `contract`, which is a second reason it does not compile: that statement is mandatory, and the section below states why. The named-call form is filled by exactly one call: `strict_serial_sum(<expression>, [<axis>, …])`, resolving to `tiler::strict-serial-sum-f32@1` and accepted by Tom on 2026-08-01 with the two forms that make it reachable — a binary32-exact scalar real literal in the body, rounded exactly as the equivalent Rust `f32` literal with a leading `-` signing the literal, and an optional axis name in an operand's declared shape (`f32[cols: 8]`), which is what a reduction names its axis by. `einops(…)`, `gelu(…)`, and every other name are still refused at the name, because the governed semantic profile registers no other operation without an operator spelling; the spelling is deliberately `strict_serial_sum` rather than `sum`, because the strict ascending fold is a numerical guarantee the name carries rather than a default the consumer discovers.

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

Completed bounded measurements — taken before an expansion existed, against spikes and prototypes rather than against the delivered path — establish:

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

### What the first vertical slice has and has not demonstrated

The checklist this section carried was written before any expansion delivered an artifact, and it is swept here as of 2026-08-01 rather than patched: several of its items were discharged by work that landed on 2026-07-31 and 2026-08-01, one was withdrawn as unreachable by construction, and the remainder split into work with an owner and work parked behind a measurement. Each item below states which of the four it is, because a reader acts differently on each — a landed item needs a citation to be checkable, an outstanding one needs an owner to be reachable, and a parked one needs the trigger that would restart it. The lists are maintained rather than re-swept: an item moves in the change that discharges it and says on which date it moved, so a reader can tell a 2026-08-01 classification from a later one without diffing.

**Landed.**

- **An expansion compiles, identifies, caches, embeds, and routes a one-entry bundle, with no consumer setup.** [`crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs`](../../crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs) is an out-of-tree consumer crate of one source file and a manifest — no `build.rs`, no registry, no source scan, no Cargo subcommand, no prepare step, no runtime JIT, each absence checkable by reading it — whose compilation runs the eight expansion-time steps above inside `rustc`, and whose binary then routes the embedded bytes through the loader. It costs a real `xcrun metal` and `xcrun metallib` run on a cold cache, deliberately: a fixture that avoided the driver would prove something about a fixture.
- **Loading, function lookup, pipeline creation, and dispatch, on hardware.** [`spikes/runtime/inline-dispatch`](../../spikes/runtime/inline-dispatch) takes one `deliver macos;` region to a completed dispatch through the `tiler::value::DispatchAdapter` seam and checks the returned bytes against the consumer's own `f32` arithmetic bit for bit. **Measurement — Apple M4 Max, macOS 27.0 build 26A5388g, `nightly-2026-07-19`, Apple metal 32023.883, 2026-08-01**, run by hand from that directory. It is bound to that host and is not evidence that another device loads the same library.
- **A multi-entry bundle, produced by an expansion and dispatched on hardware.** This section's own sentence that "one invocation may contain one fused kernel, multiple guarded schedule variants, or a multi-step plan such as a two-pass reduction" is now exercised end to end, and it moved here from **Still outstanding** on 2026-08-04. The producer half landed under [`package-a-multi-entry-bundle-from-one-expansion`](../../tickets/package-a-multi-entry-bundle-from-one-expansion.md): `tiler_macros::aot::tests::a_split_selection_packages_every_entry_in_the_one_embedded_artifact` asserts that the artifact one expansion embeds carries 1 payload, 2 entries, and the edge `[(0, 1)]`, with the flush-only contract's fused 1-entry plan as its watched perturbation, and `crates/tiler/tests/facade/pass/deliver_compiles_embeds_and_routes.rs` states the region in an out-of-tree consumer crate. The consumer half landed under [`dispatch-a-multi-entry-bundle-on-hardware`](../../tickets/dispatch-a-multi-entry-bundle-on-hardware.md): [`spikes/runtime/inline-dispatch`](../../spikes/runtime/inline-dispatch)'s second binary takes `in x: f32[rows: 1, cols: 4]; deliver macos; contract flush_and_reassociate_f32; out strict_serial_sum(x * 2.0 + 1.0, [cols])` to a completed dispatch reporting `2/2 entry(ies) encoded` over one shared allocation, counts the entries from the consumer's side, and compares the result against the consumer's own `f32` bit for bit on operands chosen so that no association can round. **Nothing asks for two kernels**: the region states a computation and what its arithmetic may do, and the selection policy answers with a split — under `flush_subnormals_to_zero_f32` the same text selects one fused kernel, which the spike runs as the perturbation that refuses the count. A deliberate back-to-front encoding of the same committed route is watched returning a **wrong answer rather than a refusal**, which is what makes the ordered run evidence about ordering. **Measurement — Apple M4 Max, macOS 27.0 build 26A5388g, `nightly-2026-07-19`, Xcode 27.0 build 27A5228h, Apple metal 32023.921, 2026-08-04**, run by hand from that directory. **Boundary — one measured shape.** The dispatch recorded here ran at `[rows: 1, cols: 4]`, the smallest window whose selected plan splits — `governed_partition` needs two partitions of at least two — and no other shape has been dispatched. When the measurement was taken it was also the *only* window selecting a split on the bound declaration, with `[rows: 1, cols: 8]` and `[rows: 2, cols: 4]` refused `NoFeasiblePlan` and `[rows: 1, cols: 5]` refused `InvalidCompilerOutput`; the grid-axis row has since widened to a retained measurement and the declined-strategy record was corrected, so those three refusals are dated observations rather than the current boundary, and the wider shapes' behaviour is unmeasured — `calibrate-and-activate-parallel-reduction-selection` owns the sweep. Nothing here is evidence about a three-entry bundle or a wider reduction, and no plan carrying *guarded schedule variants* has been produced at all.
- **The consumer storage seam the dispatch travels through.** `route-an-embedded-artifact-through-a-consumer-storage-seam` landed the `tiler::value::DispatchAdapter` boundary on 2026-08-01; the one-way commit stays inside `tiler_runtime::adapter::route_with_adapter`, and a consumer that implements no dispatch adapter still takes the semantic fallback before the commit.
- **One envelope carrying one payload per built family.** `carry-one-payload-per-artifact-family-in-one-envelope` moved artifact identity to `tiler.artifact-program.v13` and the neutral manifest to 11.0 so that an entry names one payload per *delivery position*; `tiler_build`'s `one_envelope_carries_one_payload_per_artifact_family` drives the production seam over two families end to end. The exercise is a `#[cfg(test)]` fixture, because its second family's rows were measured on a macOS host — the machinery is delivered, the second *measurement* is not (see **Parked** below).
- **A retained external-compiler diagnostic reaching the consumer at the invocation's span.** `DriverError::ToolFailure` retains the failing tool's own bytes, bounded at `MAX_RETAINED_OUTPUT_BYTES` (16 KiB) with truncation recorded rather than hidden, and `tiler_macros::aot::retained` emits them as the family-scoped `#[cfg]`-gated `compile_error!` this contract requires. [`crates/tiler/tests/facade/fail/family_cfg_matching_family_retains_its_diagnostic.rs`](../../crates/tiler/tests/facade/fail/family_cfg_matching_family_retains_its_diagnostic.rs) and its byte-compared golden pin the fatal half, and [`crates/tiler/tests/facade/pass/family_cfg_nonmatching_targets_fall_back.rs`](../../crates/tiler/tests/facade/pass/family_cfg_nonmatching_targets_fall_back.rs) pins that the same item does not break an unrelated target.
- **A nonmatching target compiling the semantic fallback, proved by compilation rather than by `cfg` inference.** `every_emitted_shape_compiles_as_the_five_target_matrix_says` in `crates/tiler-macros/src/delivery/tests.rs` compiles the delivery emitter's own output for `aarch64-apple-darwin`, `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `aarch64-apple-ios-macabi`, and `x86_64-unknown-linux-gnu`. **Measurement — `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, 2026-08-01**, fifteen compilations, every one agreeing with the recorded matrix. **Boundary — check level.** It is `rustc --emit=metadata`: no linker runs and no SDK is consulted, so it decides that the delivered *source* is correct for each target and says nothing about whether a `metallib` carried in it would load there.
- **rust-analyzer's warm interactive cost.** Measured in the table above on `rust-analyzer 1.97.0-nightly (8b03437a 2026-05-12)`, macOS 27.0 arm64 / Apple M4 Max, 2026-08-01: a settled in-region edit costs one expansion and a 137–217 ms `semanticTokens` round trip, against 10–16 ms for the same region with no `deliver` statement. The clause that stood here — that the measurement was blocked because the component was unavailable — is void: the analyzer binary and the proc-macro server were both already present on the measured host.
- **A real `metal` refusal retained through an expansion, with the compiler's own text.** `tiler_macros::aot::tests::a_real_metal_front_end_rejection_is_retained_under_its_family` drives `deliver` end to end against the host's real `metal` binary for both routes a `CompileStage::Metal` nonzero exit is reachable by, and [`crates/tiler/tests/facade/fail/family_cfg_matching_family_retains_a_metal_front_end_diagnostic.rs`](../../crates/tiler/tests/facade/fail/family_cfg_matching_family_retains_a_metal_front_end_diagnostic.rs) and its byte-compared golden pin what a consumer reads. Establishing reachability first is what the work turned on, and it found two unequal routes: a rejection of the emitted *source* is unreachable from any invocation — `tiler-metal` names entry points, helpers, and staging from identity digests, buffers `b<ordinal>`, and constants as hexadecimal bit patterns, so no region token reaches the translation unit — and is therefore a defect in Tiler's emitter reached here by injection; while a build host whose `metal` predates the bound declaration's measured MSL 4.0 reaches one with nothing wrong on either side, because nothing compares the requested language standard against the resolved tool. `crates/tiler-macros/src/aot.rs`'s "Reaching the `metal` stage's own refusal" carries the derivation. **Measurement — macOS 27.0 / Apple M4 Max, Metal Toolchain 27A5228f, 2026-08-04**; the fixture's retained text is a verbatim capture from that run, whose two absolute paths, line, and column are that run's and are recorded as such rather than as reproducible.

- **Canonical MSL and tool diagnostics readable from the cache entry, and the debug section that carries what the envelope cannot.** Moved here from **Still outstanding** on 2026-08-05 under [`retain-canonical-msl-under-a-debug-expansion-cache-entry`](../../tickets/retain-canonical-msl-under-a-debug-expansion-cache-entry.md), and it split in two on evidence. The **canonical MSL was already under the entry** and needed no new storage: `tiler_build::metal_compile_request` puts the emitted translation unit's exact source in `PayloadMetadata::source`, which is part of the preimage the payload digest is taken over, so it travels inside the artifact envelope the bundle carries and is readable from any validated hit — `payload.metadata().source == unit.source().as_bytes()` in `crates/tiler-build/src/metal_assembly.rs` pins the producer half and `a_retained_diagnostic_survives_publication_and_returns_from_the_hit` in `crates/tiler-build/tests/custom_backend` pins that it survives publication and comes back from the hit. Retaining a second copy under a non-keyed section was rejected for the reason ADR 0082 rejects a second digest authority: an unkeyed copy can disagree with the keyed original and nothing could refuse the disagreement. What has no other home is the **tool run's own output**, and that is what the new `BundleSection::DebugRetention` carries — a caller-stated `DebugRetention` of bounded, labelled runs (16 KiB each, truncation recorded, `MAX_RETAINED_RUNS` of them), framed only when a producer states one. The three identity answers are implemented rather than intended: it does **not** reach the key, so one compilation is one entry with or without it; it **does** carry its own section digest inside the declared total length and the contiguity chain, so an entry edited to alter retained text is refused; and an **absent** section is a hit with nothing to show. `crates/tiler-cache/src/expansion/retention.rs` carries the derivation, and the present, absent, and damaged cases are each exercised — including a forgery that recomputes the section digest and is still refused by the retention's own parser. **A failing compilation is out of scope by construction**: nothing is published for a miss that failed, so there is no entry to attach diagnostics to, and a failed compilation's text reaches a consumer through the family-scoped `compile_error!` in the bullet above.

**Withdrawn as unreachable by construction.** "A production warm cache hit invoking no `xcrun`" was never a property this design can have, for the reason [Why a warm expansion resolves the toolchain](#why-a-warm-expansion-resolves-the-toolchain) derives: the compiler fingerprint is an input to the compilation identity, so `Toolchain::prepare` must observe it before a lookup exists to skip. What survives of the requirement is the part that is about compilation rather than resolution, and it is demonstrated — `tiler_macros::aot::tests::the_second_expansion_of_one_subject_compiles_nothing`.

**Still outstanding, each with an owner.**

- **A succeeding Metal compilation's own diagnostics.** The storage above exists and the Metal producer has nothing to put in it: `tiler_metal_aot::driver::Toolchain::run_stage` retains a stage's captured output only when the stage *fails*, in `DriverError::ToolFailure`, and drops both streams on success, so a `metal` warning on a compilation that produced an artifact reaches nobody. Until that changes, `accept_or_publish_delivered_metal_artifact` states `DebugRetention::none()` and says so where it does it, rather than framing an empty section that would read as a delivered capability. Owner: [`retain-succeeding-metal-stage-tool-output`](../../tickets/retain-succeeding-metal-stage-tool-output.md), with [`state-a-debug-retention-from-the-inline-frontend`](../../tickets/state-a-debug-retention-from-the-inline-frontend.md) behind it for the invocation that would ask for one.

**Parked, each with a trigger.**

- **Multi-family end-to-end delivery.** Not a machinery gap — the envelope, the positional emitter, and the total `#[cfg]` selector are all delivered — but a *measurement* gap: `deliver ios;` and `deliver macos-and-ios;` are refused because no bound compile declaration exists for an iOS family, and one cannot be inherited from the macOS rows. [`first-authoritative-ios-metal-compile-declaration`](../../tickets/first-authoritative-ios-metal-compile-declaration.md) was deferred on 2026-08-01 when Tom deprioritized iOS — the target devices are Metal on macOS and CPU — and reactivates on the first consumer asking for an iOS artifact; its device half is separately hardware-blocked on an attached iOS device. [`deliver-several-artifact-families-from-one-expansion`](../../tickets/deliver-several-artifact-families-from-one-expansion.md) depends on it.
- **Any expansion on a non-macOS host.** The fallback evidence above is check-level compilation of emitted source *for* a non-Apple target, produced on a macOS host. Tiler develops on macOS only and runs no CI, so no expansion has ever run where `xcrun` is absent; the `ToolchainUnavailable` path that case would take is exercised by pointing the driver at a launcher that is not there. Trigger: a supported non-macOS development host, which is a project-scope change rather than a task.
- **The cold interactive round trip.** The rust-analyzer table measures settled edits in a warm session; no wall-clock number exists for the first expansion of a region against an empty cache inside the IDE. Trigger: this section's own analysis-stub bullet, which admits a stub only if measurements demonstrate a material problem — the warm numbers already show in-region editing 10–15× slower than fallback-only, so a cold number is what a stub proposal would have to carry.
- **The five-target compile matrix in the gate.** `every_emitted_shape_compiles_as_the_five_target_matrix_says` is `#[ignore]`d because `rust-toolchain.toml` declares no cross-targets, so a gate-resident form would fail on a host bootstrapped exactly as this repository documents. Promotion is [`declare-the-cross-compilation-targets-in-the-toolchain-manifest`](../../tickets/declare-the-cross-compilation-targets-in-the-toolchain-manifest.md), deferred against the 555 MB of `rust-std` the policy would cost every checkout.
- **MSL text attributed to region source.** Not built, and the reason is that it would name the wrong construct rather than that it is hard: no correspondence from an MSL position back to an `out` sub-expression exists at two independent points — `tiler-ir`'s semantic program holds no frontend spans and must not, and the emitter attaches no per-statement provenance — and in both reachable routes above the diagnostic's subject is not the region. The source-rejection route is a Tiler defect whose reader is a Tiler developer; the host route is about the machine. Trigger: the first invocation-controlled text that reaches the emitted MSL — a consumer-chosen identifier, an inline MSL escape hatch, or a consumer-supplied literal — makes an MSL diagnostic attributable to something a consumer wrote. [`carry-a-source-correspondence-from-region-text-to-emitted-msl`](../../tickets/carry-a-source-correspondence-from-region-text-to-emitted-msl.md) holds the design and the two questions reactivation must answer first.

None of these gaps changes the accepted contract, but they must not be reported as completed feasibility.

## Traceability

This document owns frontend translation and the inline proc-macro delivery
profile, not consumer runtime execution. Its accepted decisions and measured
macro, cache, and embedding boundaries are linked in frontmatter.

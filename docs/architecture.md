---
schema: "tiler-doc/v1"
id: "tiler.contract.architecture"
kind: "contract"
title: "System architecture"
topics: ["architecture", "compiler"]
contract_status: "mixed"
implementation_status: "partial"
evidence: ["tiler.research.program-planning.kernel-program-buffer-plan", "tiler.research.semantic-graph.rust-construction-lifecycle", "tiler.research.shapes.nightly-const-shape-parameters"]
---

# System architecture

**Status:** mixed — accepted boundaries and proposed field-level detail

Accepted ADRs govern the layer separation and dependency boundaries cited by
this document. Unless a section says otherwise, concrete component names,
schemas, and API shapes below are proposed.

## Ownership boundary

This document owns component boundaries, dependency direction, and the
compiler-to-runtime lifecycle. The IR contract owns representation fields and
verifiers; optimizer contracts own search; artifact and integration contracts
own serialization and execution adapters.

## Overview

Tiler separates tensor meaning, global optimization, local scheduling, target
emission, artifact construction, and runtime execution. Each stage consumes a
verified representation and produces another verified representation.

```text
frontend syntax or API
        │
        ▼
SemanticTensorGraph
        │ normalization + logical alternatives
        ▼
CandidateRegionSet
        ├── independent legal complete covers ──────────┐
        └── checked per-region schedules                │
              + local ImplementationFrontiers ─────────┤
        │ select and verify compatible complete plan    │
        ▼
CheckedSelectedPhysicalPlan / guarded plan portfolio
        │ verified structured lowering
        ▼
structured kernel IR per selected scheduled implementation
        │ assemble verified stages, buffers, and routing
        ▼
KernelProgram / guarded ProgramPortfolio
        │ target emission
        ▼
target source/binary + ABI manifest
        │ integration-specific packaging
        ▼
runtime adapter or embedded artifact
```

The proposed inline Metal/Candle path is one integration of this general
pipeline, not part of the compiler's defining abstraction.

### Where the post-compiler half is general today

**Fact — audited 2026-08-04 by reading the four stages below the compiler.** The interesting result is that the multi-kernel and multi-output questions have different answers, and that neither answer is uniform across the four stages, so a reader must not carry one stage's maturity to its neighbour. [The optimizer contract](compiler/optimizer.md#what-each-stage-is-general-over-today) states the same per stage for the eleven compiler stages.

**Multi-kernel and multi-entry programs are general end to end.** Metal emission takes a slice of verified kernels, orders and deduplicates them by canonical kernel identity, and emits one entry point per kernel with no cardinality check at all — an empty slice is accepted and exercised. Artifact assembly declares one entry per program stage, keyed by that stage's own kernel identity, under a variant-entry bound of 4,096 and a negotiated multi-stage-program feature the reader supports. The device-free loader routes per entry in the variant's own execution order, and the two adapters that exist encode one command encoder per entry with an explicit fence between them. Nothing here is a bounded-profile approximation of a single-kernel program.

**Ordered named outputs are general in the middle two stages and not at the ends.** The artifact layer binds storage to a *list* of named output keys per binding, derived per program value rather than per schedule role, and folds it into identity; the loader publishes outputs in semantic interface order and inspects no target class, so neither imposes a count. The two ends do. At emission, `TensorRole` distinguishes inputs by ordinal and carries none on `Output`, so *one kernel* cannot write two distinct named outputs — several stages each writing one can, because the program layer binds a stage's buffers to values positionally. At dispatch, the inline frontend's region facts carry one result key and one result byte run, and the Candle adapter matches a binding target of exactly one output key against its own fixed `"result"` key. So the runtime-side widening a multi-output program needs is at the consumer seam, not in the artifact encoding or the loader.

**One restriction the loader does enforce is about pairing, not counting, and it is worth not misreading as the multi-output wall.** Deriving a shared allocation from a typed data dependency requires the producing entry to have exactly one internal write binding and the consuming entry exactly one internal read binding; either failing is a typed refusal naming which. Its own site records that the uniqueness half is unreachable from any current fixture and is retained so that widening the kernel profile becomes a refusal rather than a silent mis-bind — which is a claim about the code rather than a measured one.

**Live execution is a role no crate is admitted to own, and since 2026-08-07 it is no longer absent from `crates/`.** `grep -rln 'wait_until_completed' --include='*.rs' .` returned only `spikes/runtime/` and `prototypes/` files when this section was audited on 2026-08-04; at `3e0074d5` it also returns `crates/tiler-conformance/src/dispatch.rs` and `crates/tiler-conformance/src/device_buffer.rs`. The loader is still device-free by decision, and the submit, terminal-status, and readback path now exists three times — twice as proof executables and once inside the conformance member. All three check the command buffer's terminal status after waiting and before any readback, and all three classify an unnamed status as non-terminal rather than as success. **The distinction the grep no longer draws is worth drawing by hand.** Both new files are `#[cfg(all(test, target_os = "macos"))]` test machinery inside the member the component table records as owning cross-layer executed *evidence*, which is a different responsibility from the runtime-adapter role: a conformance run reaches a device to refute a claim, and it publishes no execution boundary any component may reach for. So the table's reading is unchanged — live execution is still a role with no admitted crate — but nothing mechanical confirms it any more, and a future crate that began executing for a consumer rather than for a comparison would not be caught by this grep.

## Semantic invocations and consumer-owned composition

One compiler request describes one pure, finite semantic program with explicit
inputs and ordered named outputs. Sharing and multi-result operations live
inside that program; recurrence across program invocations does not. A caller
that retains an output tensor and binds it as a later input is composing two
ordinary invocations.

The generic runtime owns artifact validation, explicit binding, reversible
preflight, one-way routing commit, execution, and returned outputs. It does not
own models, transformers, KV caches, decode cursors, samplers, training or
inference loops, serving, or application/session state. A runtime adapter may
retain device-scoped pipelines and asynchronous resources because those are
physical execution obligations, not semantic workload state.

Workload programs remain useful as conformance evidence and optimization
motivation. A workload-specific observation becomes a core feature only after
it is restated as a consumer-neutral semantic operation, physical property,
feasibility rule, cost-model input, or execution contract that applies beyond
that workload.

## Core-contract evidence

The core boundary is backed by completed research tracks rather than frontend
or backend assumptions:

| Contract | Evidence | Durable decision |
|---|---|---|
| Pure operation/value DAG, sharing, multi-result operations, and ordered named results | [Semantic graph contract memo](research/semantic-graph/contract-memo.md) | ADRs 0005 and 0006 |
| Transactional drafts, recoverable consuming build, immutable shared programs, and graph-owned handles | [Rust semantic-program construction lifecycle](research/semantic-graph/rust-construction-lifecycle.md) | ADR 0058 |
| Scoped extent symbols, typed root bindings, admitted constraint language, and sourceability | [Shape environment contract](research/shapes/shape-environment-contract.md) | ADR 0008 |
| One semantic authority plus separately versioned optional capabilities in an explicit frozen registry | [Operation-extension research](research/extensions/operation-extension-surface.md) and [API spike](research/extensions/operation-extension-api.md) | ADR 0044 |
| Proc-macro provider visibility bounded by the host dependency graph | [Proc-macro visibility experiment](research/extensions/proc-macro-extension-visibility.md) | ADR 0045 |

These decisions constrain the compiler core. They do not make serialized IR a
public compatibility promise, add effectful operations, or require a proc
macro for non-Rust consumers.

## Consumer-independent compilation request

One initial compiler invocation borrows one immutable, verified
`SemanticProgram` through a conceptual `CompilationRequest`:

```text
CompilationRequest {
    semantic_program: &SemanticProgram,
    numerical_contract,
    shape_environment,
    target_profiles,
    frozen_operation_registry_and_provider_fingerprints,
    installed_lowering_capabilities,
    deterministic_search_and_artifact_budgets,
    compilation_options,
}
```

`installed_lowering_capabilities` is a request input rather than a compiler constant, and the compile path resolves every recognized occurrence through it. It carries a frozen lowering-capability registry together with the exact frozen scalar authority that registry was registered against; the two are checked to agree at the request boundary, because every resolved provider emits against that authority. Tiler's own governed lowerings are registered through the same builder any other provider uses, so the bounded profile is one composed registry rather than a privileged path.

Frontends obtain that program through the ADR 0058 commitment boundary:

```text
SemanticProgramBuilder -- build(self) --> SemanticProgram
```

The mutable builder is not compiler input. Its edits are transactional, its
borrowed validation is diagnostic, and a failed consuming build returns the
original builder with structured diagnostics. A successful build moves the
graph storage into private immutable `Arc`-backed data. Compiler, optimizer,
and evaluator APIs borrow the result, so sharing a completed program is cheap
without making unfinished-graph snapshots implicit.

The semantic program remains backend-neutral. `shape_environment` contains the
typed root-binding environment for its extent symbols, including explicitly
admitted target-property bindings. It contains stable declarations and values
available at their declared binding phase, never live backend objects or
implicit callbacks. Physical-only target facts remain target-profile or ABI
inputs rather than semantic bindings.

The result is one or more target-specific `KernelProgram`/`ProgramPortfolio`
values expressed in target-independent compiler schemas, plus diagnostics and
provenance. Backend compilation, packaging, cache publication,
embedding, and runtime loading are later integration/backend steps. A proc-macro
invocation may aggregate or package compiler results, but it is not the
consumer-independent compilation unit.

The compiler entry point remains general even when implemented support is
narrow. Capability resolution distinguishes an invalid semantic request from a
valid program lacking access, scheduling, target, or lowering support, and
from a candidate that is intrinsically or target-infeasible. Initial vertical
slices remain private strategy and conformance identities; they do not create
graph-specific compiler entry points or public support-profile namespaces.
Fixed region, stage, entry, and buffer cardinalities in a slice are not
`CompilationRequest` or compiler-product invariants. See ADR 0069.

Lowering-capability resolution is an implemented stage of that entry point rather than a description of one. It runs unconditionally, resolves exactly one index/access capability per recognized occurrence, and fails closed on an absent or a contended capability with a typed, occurrence-attributed cause. An out-of-crate caller can compose a lowering registry through the public capability surface, bind it to the exact frozen scalar authority through `session::InstalledCapabilities::installed`, install it with `session::CompileRequest::with_capabilities`, and compile through `session::compile`. The semantic provider transaction owns each operation's immutable typed realization law; request verification derives and retains that law snapshot from the exact semantic registry carried by the program, independently binds its identity into the request subject, and checks the installed lowering/scalar pair against the program's semantic snapshot. Equal semantic snapshot bytes therefore cannot let an installer's different law sidecar replace the program's law. `session::compile_governed` is a single-target convenience wrapper over that same path rather than a second pipeline. The session module remains a reviewed experimental draft as a complete facade; shape-environment choice and deterministic budgets remain governed because each still has only one admitted public value. Separately, Tom accepted the experimental caller-authored `TargetProfileBuilder`/immutable `TargetProfile` boundary and bounded `TargetRequest`, so target selection is no longer confined to the governed profile. [The optimizer contract](compiler/optimizer.md#lowering-capability-resolution-and-index-region-refinement) owns the stage's behaviour and maturity boundary.

## Hierarchical planning with feedback

The design deliberately separates global tensor planning from local kernel
scheduling without pretending they are independent sequential phases.

### Global tensor optimizer

The program planner decides:

- equivalent logical formulations;
- fusion regions and materialization boundaries;
- whether shared work is recomputed or stored;
- useful intermediate layouts;
- opaque library-call boundaries;
- which boundary requirements and guarantees region implementations must
  satisfy.

Its natural unit is `SemanticTensorGraph`: an operation/value DAG with sharing,
multi-result operations, and several named program results. A hypergraph may be
used internally to index overlapping region candidates, but it is not the
durable graph or physical-program representation. Region identity includes
boundary values, retained results, and allowed duplication, not only a set of
member operations. Canonical region semantic content is separate from its
occurrence identity and exact graph-value bindings, so equivalent content may
occur more than once without losing coverage or sharing information. Actual
materialization is selected by region implementations and the complete kernel
program.

### Local kernel scheduler

For a proposed fusion region, the local scheduler decides:

- iteration order and dimension coalescing;
- mappings to governed target execution scopes/coordinates, such as GPU lanes,
  subgroups, and threadgroups or CPU tasks, threads, and vector lanes;
- tile and vector widths;
- tail predication;
- reduction strategy;
- local-memory staging and synchronization;
- launch geometry and capability requirements.

Its natural unit is one `RegionCandidate` with iteration domains and access
maps. It returns a bounded `ImplementationFrontier`, not one unconditional
winner. Every retained `RegionImplementation` contains boundary
requirements/guarantees, applicability predicates, target requirements,
consumed compile guarantees, deferred target predicates with evaluation phases,
exact/proven resource requirements, estimates, calibration identity, and a cost
estimate. The mature implementation body is one of:

```text
ScheduledKernel(ScheduledRegion)
KernelSubprogram(stages, internal temporaries, dependencies)
OpaqueCall(call contract)
View(alias/metadata result)
```

The bounded physical frontier admits checked `ScheduledKernel` and
`KernelSubprogram` values and registered `OpaqueCall` proposals, and rejects
`View` explicitly while retaining this additive sum-type seam. A subprogram is
an ordered chain of dispatches realizing one region subject, verified stage by
stage through the same checked path a single kernel takes.

Every executable body also carries the selected numerical realization,
machine-checkable guarantee, and scoped evidence identity. These must refine
the region's effective operation contracts before costing.

Index, schedule, and structured-kernel identities describe canonical
structural content. An IR-owned sealed receipt binds index structure to a
particular semantic-program occurrence, exact boundary/access mappings, reached
semantic definitions, and proof evidence; the compiler layers selected-provider
provenance around that receipt. Complete
program identity—not a nested whole-graph digest inside every structural
object—proves occurrence coverage, executable composition, and the executable
contract a program commits to: its buffers, its entry ABI, its applicability
guard, and its routing-commit lifecycle. ADR 0072 owns this identity layering.

Selected provider provenance is derived, not declared. An artifact plan's lowering providers are re-derived from the request's own installed registry and compared against what the plan recorded, so a receipt naming an authority the registry never resolved fails closed rather than being carried into a compilation product.

A locally slower implementation may provide a layout that removes a downstream
conversion. Multi-pass reductions are `KernelSubprogram` bodies rather than one
oversized `KernelSchedule`; opaque library calls need not invent a schedule.

Lowering-capability resolution and index-region refinement precede cover
enumeration, because grouping occurrences the installed authority cannot lower
would enumerate plans nothing could realize.
Complete-cover enumeration independently proves legal coverage using candidate
regions. Per-region schedule verification and target-aware frontier formation
independently prove local implementations; they do not depend on a globally
selected cover. The program planner then joins one complete cover with
compatible implementations and emits a checked selected-physical-plan or
portfolio receipt. Structured KIR refinement follows that selection.

An implementation may interleave these searches, schedule only regions still
present in viable covers, and feed boundary, materialization, or cost bounds in
both directions. This feedback is why the architecture is hierarchical
planning rather than a rigid batch pipeline. It does not invert authority: a
cover is not schedule evidence, a frontier is not whole-program coverage, and
neither substitutes for checked complete-plan selection.

The selected `KernelProgram` is an executable dependency DAG of kernel stages,
materializations, temporaries, and opaque calls, together with the entry ABI
each stage dispatches under, the guard deciding whether it may be routed to at
all, and the routing-commit lifecycle stating where fallback stops being legal.
A `ProgramPortfolio` may retain several complete programs for different runtime
applicability regions, ordered by the priority in which their guards are tried;
that priority is the portfolio's and never a program's.

Whole-program verification checks semantic-result coverage, dependency
acyclicity, producer completeness, deliberate duplication of pure work,
boundary-contract satisfaction, temporary initialization and lifetimes,
aliasing, ordered opaque effects, ABI/launch references, and routing among
complete programs. Region-local and schedule-local verification do not imply
these whole-program invariants.

## Initial placement, execution, and buffer model

The semantic graph is device-agnostic. Each initial `KernelProgram` targets one
device/target domain: all bound inputs are accessible there, every scheduled or
opaque stage executes there, all temporaries reside there, and results are
produced there. Cross-device transfers, placement search, sharding, distributed
collectives, and queue affinity are not represented initially; future support
requires explicit placement and transfer stages rather than hidden schedule
annotations. That one-device profile is [ADR 0047](decisions/0047-model-placement-as-physical-properties.md)'s accepted initial execution profile — one symbolic affinity, one live device, one ordered command stream — and it is unchanged here: whether multiple devices become expressible at all is [`multi-device-and-sharding-scope-gate`](../tickets/multi-device-and-sharding-scope-gate.md)'s deferred product decision, not something a vocabulary or a component boundary settles.

The initial execution contract uses one ordered command stream and a canonical
topological step order. Dependency edges verify producer/consumer order and
program structure but do not authorize concurrent execution of incomparable
steps. Multi-queue execution, explicit events/timepoints, and asynchronous
cross-device programs require a later execution model and participate in
program identity.

`KernelProgram` owns a canonical `BufferPlan` distinct from semantic tensor
values and views. The conservative initial policy assigns one distinct,
non-aliasing allocation to each output and declared cross-kernel temporary;
inputs may alias one another, but there is no output/input aliasing, temporary
reuse, suballocation, or in-place assignment. Every temporary use is after
initialization and before lifetime end. A later buffer-assignment pass may reuse
storage only with explicit liveness, size, alignment, memory-space, and alias
proofs.

## Component ownership

ADR 0070 accepts these dependency and verifier-ownership boundaries. They are not necessarily the final published-crate layout, and individual field sets remain experimental until their dedicated implementation tickets land. The `tiler-metal-aot`, `tiler-runtime`, `tiler-cache`, `tiler-build`, `tiler-macros`, `tiler`, `tiler-digest`, and `tiler-conformance` rows postdate ADR 0070 and each carries its own accepted admission — ADR 0077, ADR 0081, ADR 0082, ADR 0085, [ADR 0088](decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md) for the frontend pair, [ADR 0104](decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) for the digest, and [ADR 0106](decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md) for the conformance member — rather than being covered by 0070's block; the packaging profile below states what each of those records decides. `Frontend core` and `tiler-candle` are the two rows that remain roles with no admitted crate behind them.

The `tiler-metal-aot` row previously read "Expansion-time Apple tool invocation, cross-process content cache, atomic publication, byte embedding, …" beside forbidden dependencies of "Every workspace and third-party dependency". [ADR 0082](decisions/0082-admit-tiler-cache-as-the-expansion-cache-owner.md) is the accepted record that those two halves could not both stand: ADR 0050 requires every cache hit to be validated against the governed digest `tiler.digest.sha-256.v1`, so the assigned owner could not implement what it was assigned without spending the closure the same cell decided. Tom decided on 2026-07-25 that the cache is a dedicated crate and the driver keeps its closure. Byte embedding moves with it to the `tiler-macros` row, which already emits the artifact tokens.

The `tiler-digest` row is the twelfth library and the second time that same constraint decided a boundary, in the one direction ADR 0082's answer was unavailable. [ADR 0104](decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) folds each executable-coverage record's bound graph identity as a digest, which turns kernel-program identity from quadratic in operation count into linear; the crate that mints that record is `tiler-ir`, and `tiler-ir` is the crate every other member depends on. ADR 0082 resolved a reachability problem by moving the *consumer*; here the consumer is the bottom crate and cannot be moved, and a local hash function would make the shared IR a second identity authority over one subject — the thing that record's own argument refuses. Tom decided on 2026-08-06 that the governed digest is its own crate below both, on the grounds that hashing is a separate responsibility from tensor IR, that the one-authority rule deserves a structural home rather than riding in whichever crate needed it first, and that a future layered-identity consumer should reach it without reopening the boundary. `tiler-artifact` re-exports the three promoted names from `tiler_artifact::program`, so ADR 0082 item 3's surface is unchanged and `tiler-cache` reaches it exactly where that record says.

| Component | Responsibility | Forbidden dependencies |
| --- | --- | --- |
| `tiler-digest` | The one governed content-digest algorithm every identity authority in this workspace hashes under: `DigestAlgorithm`, the opaque `Digest`, `DIGEST_BYTES`, the wire tag table, and the two admitted pre-image shapes | Every workspace crate, and every third-party dependency but the measured hash implementation. It owns no *domain*, deliberately: a domain belongs to the authority that decides what it names |
| `tiler-ir` | Public semantic graph and operation-extension contracts; experimental index, schedule, kernel, executable-program, `BufferPlan`, and `AbiExpr` representations, including a program's entry ABI, applicability guard, and routing-commit lifecycle; authoritative IR verifiers and pure checked expression semantics | Frontend syntax, reference execution, artifact encoding, runtime fact binding, Candle, and Metal runtime APIs |
| `tiler-reference` | Host reference values, executable semantic-operation capabilities, interpreter traversal, and conformance utilities | Optimizer, scheduler, backend, and live device APIs |
| `tiler-compiler` | Normalization, rule engine, fusion planning, index lowering, schedule search, costing, typed explain infrastructure | Candle |
| `tiler-artifact` | Versioned target-neutral artifact/ABI encoding, compatibility, runtime fact binding, failure classification, and backend-payload mappings | Candle, optimizer, and Metal device APIs |
| `tiler-metal` | Pure structured-kernel-to-MSL translation; target facts for language standard, artifact family, deployment minimum, subnormal arithmetic, and buffer binding capacity; and the selected emission realization that fixes backend ABI choices such as the launch-index declaration | Candle, Metal device APIs, and Apple tool discovery in its normal dependency graph |
| `tiler-metal-aot` | Expansion-time Apple tool invocation, the canonical compilation-key subject its own inputs determine, and the target facts a compiler invocation selects: Apple SDK, language standard, artifact family, and deployment minimum | Every workspace and third-party dependency, Candle included: its empty closure is decided, not incidental |
| `tiler-cache` | The cross-process expansion cache: content-addressed namespace, per-key advisory locking, immutable self-validating bundles, complete validation on every hit, atomic publication, corruption replacement, and typed miss and refusal reporting | Optimizer, semantic IR construction, backend internals, Apple tool discovery, every platform device API, and any second content-digest authority |
| `tiler-build` | Build-time sequencing from checked compiler plan through backend emission, prepared AOT compilation, artifact assembly, cache-subject composition, provenance correspondence, publication, and hit acceptance | Runtime device APIs, frontend syntax, consumer runtime objects, and reimplementations of identities or subject encodings owned by its dependencies |
| `tiler-runtime` | Device-free artifact decoding, declared-target-profile compatibility classification, program binding by canonical identity, carried-object resolution, and the one-way routing commit | Optimizer, semantic IR construction, backend internals, every platform device API, and any dependency that would make a load undecidable without hardware |
| Frontend core | Translate source syntax into semantic IR and map diagnostics back to users | Backend-specific scheduling |
| `tiler-macros` | Token parsing, span mapping, inline region expansion, the expansion's stated canonical artifact-family delivery policy, and invocation of the frontend/compiler/AOT pipeline to emit artifact plus runtime/fallback tokens | A second semantic operation vocabulary or canonical selection encoder, consumer source scanning, a required consumer `build.rs`, runtime source JIT, and Candle runtime internals beyond its public adapter |
| `tiler` | The inline Rust frontend's import path: the `tensor!` re-export, stable absolute paths generated tokens name, the doc-hidden compiler-diagnostic builtin re-export those tokens use, runtime-value adaptation, and re-exported neutral artifact/loader vocabulary needed by generated routes | Compiler and build orchestration, the offline Apple driver in particular, backend-specific scheduling, and consumer-specific runtime objects |
| `tiler-candle` | Layout validation, output allocation, pipeline cache, ABI binding, dispatch, fallback | Optimizer internals |
| `tiler-conformance` | Cross-layer executed conformance evidence: one run that builds a program in the shared IR, plans and lowers it, compiles and packages the artifact, executes it, and compares the result against the reference oracle — reporting the measured half as unavailable and naming what was missing on a host that cannot reach it, never a silent skip and never a claimed pass. It is the one member nothing depends on and nothing may, which is the inverse of the facade's position, and why its declared edges are normal rather than development | The frontend pair `tiler` and `tiler-macros`, which `crates/tiler/tests/dependency_direction.rs` refuses against the merged `Cargo.lock` edge list. Three further exclusions are ownership refusals rather than Cargo edges, and they are the row's real boundary: a second semantic authority — `tiler-reference` remains the oracle and this crate only uses it — a benchmark harness, and a home for layer-local tests, which stay in the crate whose behaviour they name |

Neither Metal row subsumes the other, and their overlap is owned twice deliberately. `tiler-metal` records what emitted source declares it was written against; `tiler-metal-aot` records what a compilation is invoked with. They overlap in exactly the language standard, the artifact family, and the deployment minimum. The emitter separately owns subnormal arithmetic and binding capacity as target facts and owns the selected launch-index declaration as an emission realization; the latter is a choice among source forms the language permits, not a fixed target delivery fact. The driver separately owns the Apple SDK, which selects `xcrun --sdk` and builds the target triple, and which has no emitter counterpart at all. Collapsing the overlap into a shared type or a dependency edge would spend the driver's empty dependency closure on three enumerations; `crates/tiler-metal/src/target.rs` and `crates/tiler-metal-aot/src/input.rs` own that reasoning and the alternatives it rejects. What keeps the two vocabularies from drifting is not a shared type but a total map: `crates/tiler-metal/src/target_correspondence.rs` pairs every variant of each vocabulary with its counterpart through matches exhaustive over both, so a language standard or artifact family added to either crate fails `tiler-metal`'s build until the other gains it.

That split is one topology and not a naming rule. A *backend* is the responsibility of translating verified physical work into one target representation and declaring the target facts that translation depends on: `tiler-metal` owns exactly that, and the rows around it separately own expansion-time AOT invocation, artifact assembly and cache publication, device-free loading, and — as a role with no admitted crate — live execution. A `tiler-<backend>` crate name is therefore correct only where the named role really is backend-owned, and a second target may divide the same responsibilities differently: the bounded scalar CPU vertical declares its own governed backend-family and representation keys from a spike rather than from a crate at all. The [glossary](glossary.md#backend-device-and-execution-context-vocabulary) separates backend, backend family, provider, target profile, artifact family, representation, live device, execution context, the device-free execution environment, and runtime adapter; this table allocates responsibility and dependency direction and does not redefine any of them.

**Fact — an accepted record states how these responsibilities compose across a *set* of backends.** [ADR 0090](decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), accepted 2026-07-31, decides that a component claims individual responsibilities rather than a whole backend, on the evidence of two spikes that each wanted a strict subset of one, and that no `BackendProvider`, provider bundle, emitter registration, or runtime-adapter registration exists or is needed. This table and the split above are the responsibility map that decision composes over; the concrete public surfaces it names remain individually Tom's under ADR 0075 at implementation time.

Initially, semantic, index, schedule, and kernel IRs may be modules in one
crate. Splitting every representation into a crate before its API stabilizes
would add ceremony without improving the dependency graph.

Shared compiler IR uses checked public builders with private storage. Local
insertion failures are reported immediately; consuming `build()` performs the
whole-object verifier and returns an opaque immutable verified product or a
typed failure retaining builder ownership. Compiler passes, third-party plan
producers, artifact decoders, and backends use this same verifier authority.
Only verified products cross those boundaries. See ADR 0071.

ADR 0074 fixes when a landed authority may join a crate's public surface. A
component that is implemented but not yet reachable from its crate's entry
point stays a private module whose items are crate-visible, carrying a
module-level `#![allow(dead_code, reason = "…")]` whose stated reason names
what the surface reserves and, where it is known, which slice will consume it.
It becomes public only when Tom accepts the exact facade, and a module that is
already public while its boundary is still under review says so in its own
module documentation, so a consumer cannot mistake a reviewed draft for a
settled interface. That staging rule keeps the ownership table above checkable
while implementation is incomplete: a row records which component owns a
responsibility, not that its crate has already published a public surface for
it.

ADR 0074's remaining conventions constrain the shape of a public API rather
than the placement of a component: typed non-erasing errors, opaque identities
that expose canonical bytes, domain-separated and exhaustively matched
canonical encodings, the transactional builder with a consuming terminal, and
verified products that expose no public fields. They bind any workspace crate
that exposes such a surface, and the IR contract states them normatively for
the representations it owns. This document does not restate them: a component
boundary in the table above allocates responsibility and dependency direction,
and it does not license a differently shaped public API.

### Permanently internal authorities

[ADR 0078](decisions/0078-name-the-intended-public-extension-seams.md) accepts that these responsibilities are Tiler's outright, free to change shape without a participation story: region formation, cover enumeration, fusion-legality *derivation*, plan selection, feasibility assessment, normalization, request verification, the pipeline itself, and the governed provider set. Each decides something about a program that no provider is positioned to know, and none of them takes a proposal. Giving one of them a registration path contradicts an accepted decision and needs a superseding one. [The operation-extension contract](operation-extensions.md#public-extension-seams) states the complementary half, which surfaces are intended as public extension seams, and neither list may be extended by reading the other.

Two qualifications keep that from over-claiming, and both belong here because this document is what allocates the responsibilities they name. **Explain** is internal as an authority and public as an obligation: nobody registers an explain provider, and ADR 0073 nonetheless makes typed explain output a contract every stage speaks, the seams included. **Feasibility** is internal as a decision *procedure* only. The compiler owns the target-neutral checked profile vocabulary and verifier; it does not own backend facts. A backend owns the facts it declares. When those sibling crates cannot depend on one another, `tiler-build` owns the checked projection because it already sees both authorities.

**Fact — which half of the feasibility boundary an out-of-crate provider may pre-run is decided.** [ADR 0090](decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), accepted 2026-07-31, exposes a compilation's resolved numerical realization to a provider while keeping hard feasibility a rejection the provider reads back, precisely because feasibility is the decision procedure this section holds internal. Both halves stay implemented internally until the seam ships; the asymmetric split is the accepted position rather than a proposal.

That ownership is implemented first for a bounded F32 Metal subnormal seam. `tiler-metal` totally projects its target-side behaviour into the shared numerical vocabulary; the compiler exposes independent measured input/result declarations; and `tiler-build` applies both transactionally without freezing the profile builder. The supplied measurement provenance is caller-vouched and independent from `MetalTargetFacts`. This seam therefore does not own or construct a production Metal profile, source quantitative or dispatchability facts, bind a plan/artifact/runtime environment, or generalize to F16/BF16. The production binding remains downstream work.

## Accepted prototype packaging profile

ADR 0065 refines ADR 0056 after the evaluator implementation exposed a real consumer boundary. The workspace carries twelve reusable libraries, one conformance member counted apart from them, and three non-published proof/integration executables, whose intra-workspace edges — normal, plus development where marked — are:

```text
tiler-digest    -> []
tiler-ir        -> [tiler-digest]
tiler-reference -> [tiler-ir]
tiler-artifact  -> [tiler-digest, tiler-ir]
tiler-compiler  -> [tiler-ir]                  + development [tiler-reference]
tiler-metal     -> [tiler-ir, tiler-artifact]  + development [tiler-metal-aot]
tiler-metal-aot -> []
tiler-runtime   -> [tiler-artifact]                  + development [tiler-ir, tiler-reference]
tiler-cache     -> [tiler-artifact]
tiler-build     -> [tiler-artifact, tiler-cache, tiler-compiler, tiler-ir, tiler-metal, tiler-metal-aot]
tiler-macros    -> [tiler-build, tiler-cache, tiler-compiler, tiler-ir, tiler-metal-aot]
tiler           -> [tiler-artifact, tiler-ir, tiler-macros, tiler-runtime]

tiler-conformance -> [tiler-artifact, tiler-build, tiler-compiler, tiler-ir, tiler-metal, tiler-metal-aot, tiler-reference, tiler-runtime] + macOS-gated metal

tiler-prototype-compile -> [tiler-ir, tiler-reference, tiler-artifact, tiler-build, tiler-cache, tiler-compiler, tiler-metal, tiler-metal-aot]
tiler-prototype-run     -> [tiler-ir, tiler-reference, tiler-artifact, tiler-build, tiler-compiler, tiler-metal, tiler-metal-aot, tiler-runtime] + metal
tiler-prototype-candle  -> [tiler-artifact, tiler-build, tiler-ir, tiler-metal, tiler-metal-aot, tiler-runtime] + candle-core
```

These edges are a description maintained by reading rather than a checked contract, with two checked slices. Nothing pins any package's dependency list — the *member set* is pinned, by `crates/tiler/tests/workspace_population.rs` as the paragraph on admission records below states, and that is a different property — so a manifest that gains an edge crossing a boundary an ADR decided is caught in review of that manifest diff. The first slice is the frontier around the frontend: `crates/tiler/tests/dependency_direction.rs` reads `Cargo.lock` — what Cargo actually resolved, merging normal, build, and development edges into one list per package — and fails if any non-frontend package holds a direct edge to `tiler` or `tiler-macros`, or if `tiler` holds one to `tiler-metal-aot`. The second is `crates/tiler-runtime/tests/identity_join/main.rs`'s `the_consumer_links_no_compiler_emitter_or_build_provider`, which walks `tiler-runtime`'s dev-inclusive transitive `Cargo.lock` closure, requires `tiler-artifact`, `tiler-ir`, and `tiler-reference`, and refuses `tiler-build`, `tiler-compiler`, `tiler-cache`, `tiler-metal`, and `tiler-metal-aot`. Each checks one property of this block rather than pinning the block: the frontend frontier is the first part of the table a test could say no to since `scripts/check_workspace.py` was deleted by `e197176`, and the runtime check is a negative-reachability refusal rather than an exact-closure assertion.

The block lists intra-workspace edges, and several rows carry third-party edges it does not show: `tiler` and `tiler-ir` each hold a development dependency on `trybuild` for their compile-fail evidence, and `tiler-digest` holds a normal one on the measured hash implementation. A `-> []` row therefore means "no intra-workspace edges" and not "an empty complete closure", which today is decided for `tiler-metal-aot` alone — the two `-> []` rows look alike and claim different things. The runner's `metal` binding is shown because it is a landed edge rather than a planned one: `prototypes/serial-sum-run` executes the value proof on a real device. `tiler-conformance` declares the same binding behind `cfg(target_os = "macos")`, and since 2026-08-07 it holds content behind it — `b7c01815` landed the BF16 vertical and `0f948637` the migrated device-executed value proof — so both rows record an executed path rather than one of them recording a declared edge alone, and the prototype is no longer the one member whose *code* reaches a device. That is what moved the live-execution grep recorded earlier in this document off "no file under `crates/`", and that paragraph now names the two files it returns there.

`tiler-metal-aot` is the offline Apple Metal compiler driver. Its empty dependency closure is a decided property rather than an accident of ordering: the crate spawns `xcrun metal` and `xcrun metallib`, and its whole value is that the exact compiler invocation can be read and audited without the lowering stack behind it. `crates/tiler-metal-aot/src/input.rs` records that property and what follows from it.

The `tiler-metal` → `tiler-metal-aot` edge is a development dependency only, and promoting it would cost both reasons it exists. `tiler-metal` is pure source emission owning no Apple tool discovery, so a normal edge would put a process-spawning toolchain driver into every consumer's build graph to serve tests alone. And Cargo permits a cycle through a development dependency while rejecting one through normal dependencies, so keeping this edge out of the normal graph preserves the eventual `tiler-metal-aot` → `tiler-metal` production direction that the driver's consumption of emitted source implies.

[ADR 0077](decisions/0077-admit-tiler-metal-aot-as-a-dependency-free-driver.md) is the accepted record of that admission: it admits the crate, decides the empty closure and the development-only edge as properties rather than accidents, restates the block above with six libraries and both development edges, and supersedes ADR 0056's retained AOT-invocation clause. That supersession is now in force, so ADR 0056's retained packaging text no longer places AOT invocation inside `tiler-metal`. ADR 0065 is correct exactly as accepted — its count is an ordinal about the crate it adds, `tiler-reference`, not a cap on the profile — and is not superseded by either. ADR 0077's own six-library restatement is likewise an ordinal about the crate it admits and is not a cap; [ADR 0081](decisions/0081-admit-tiler-runtime-as-a-device-free-artifact-loader.md) adds the seventh, [ADR 0082](decisions/0082-admit-tiler-cache-as-the-expansion-cache-owner.md) the eighth, [ADR 0085](decisions/0085-admit-tiler-build-as-the-build-time-orchestrator.md) the ninth, [ADR 0088](decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md) the tenth and eleventh, and [ADR 0104](decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md) the twelfth, `tiler-digest`. [ADR 0106](decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md) admits `tiler-conformance` and continues that chain without extending its count: the member is counted *apart from* the twelve rather than as a thirteenth, because it expresses no component and no dependency edge points at it. None of those records edits ADR 0077's block: an admission record restates the profile as of its own acceptance, and this document is what holds the live one. `crates/tiler/tests/workspace_population.rs` is what makes the count fail rather than stale — it names the expected member set and derives the actual one from `cargo metadata`.

`tiler-cache` is the cross-process expansion cache. Its single edge is a decided property in the same sense as the driver's empty closure and the loader's single edge: it reaches `tiler-artifact` for exactly the two things a storage protocol cannot supply itself and ADR 0050 requires on every hit — the governed digest `tiler.digest.sha-256.v1`, which validates a stored bundle's section digests, and `decode_artifact`, which re-proves the carried envelope's manifest, section digests, and canonical identity. Anything wider would let a cache decide something about a program; a *local* hash function, the alternative that needed no edge at all, would make it a second identity authority over one subject. `crates/tiler-cache/src/expansion.rs` records the five correctness properties it implements and, separately, the two it does not test in-crate. The digest half of that edge is now a re-export: ADR 0104 moved the algorithm itself to `tiler-digest`, and the cache's declared edge and reached path are both unchanged, which is what the re-export exists to preserve.

`tiler-digest` is the governed content-digest algorithm and the workspace's bottom crate. Its near-empty closure is a decided property in the same sense as the driver's empty one: it depends on the measured hash implementation and on nothing else, because it owns one algorithm, one opaque digest, and the discipline that separates their subjects — a second dependency would mean it had acquired a second responsibility. What it deliberately does *not* own is any governed domain. A domain names a subject, and the authority that decides what a subject is owes the no-prefix check over its own admitted set; siting the domains here would gather that obligation into a crate that knows none of the subjects it is discharged for. `docs/artifact-abi.md` records the obligation normatively, including the cross-crate argument that stands where no single crate can hold the check.

`tiler-runtime` is the device-free artifact loader. It decodes artifact bytes, classifies the declared target profile against a host's stated execution environment — the [device-free host declaration](glossary.md#backend-device-and-execution-context-vocabulary) `ExecutionEnvironment`, which carries one target-profile reference, one backend family, and one executable representation, and which is not the live device or execution context a route later binds — binds a loaded artifact to the program a caller expects by canonical identity, resolves the carried object, and commits routing one way. Its ordinary `preflight` refuses unanswered deferred predicates; its staged `prepare` returns the exact routed entries and entry-bound target-property requests so an outside device host can reversibly prepare pipelines and resolve those requirements before obtaining the same committable preflight. The loader itself creates no device object, no pipeline state, and no command encoder. Its single-edge closure is a decided property in the same sense as the driver's empty one: a loader that acquired `tiler-compiler` could rebuild a plan instead of validating one, and a loader that acquired a platform binding would stop being decidable without hardware. The device half of a runtime stays outside it, in `prototypes/serial-sum-run` today and in a backend runner later.

`tiler-build` is the downstream build-time orchestrator admitted by [ADR 0085](decisions/0085-admit-tiler-build-as-the-build-time-orchestrator.md). Its implemented checked-plan path consumes an owner-linked compiler alternative and the compiler's complete offered-provider environment — the compilation-request environment of ADR 0072, which is provenance rather than an execution environment of any kind — emits one Metal translation unit, prepares one AOT operation, assembles the descriptor-only and carried forms of one target-neutral artifact through one recipe, composes the cache subject, compiles only on a miss, and re-proves correspondence before publication or hit acceptance. The initial support profile is deliberately singular: one checked plan, one Metal payload, compiler-minted prepared-entry target requirements preserved whole with their exact program-entry ordinals, and no launch-time preconditions. Artifact construction mints each executable predicate from its checked requirement rather than accepting an assembler-authored formula. The crate consumes typed facts without becoming another identity, digest, or subject-encoding authority, and its existence does not spend `tiler-metal-aot`'s empty closure.

**Fact — that promotion has landed, and it changed what a second backend must author rather than what this block says.** [ADR 0090](decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), accepted 2026-07-31, decided to promote `tiler-build`'s existing backend-neutral assembly closure rather than add an edge or a backend trait. Both halves are now public: `assemble_plan_artifact` is the assembly seam and `accept_or_publish_delivered_payload_artifact` the cache-and-correspondence half, exported at `crates/tiler-build/src/lib.rs:80-84`, and two of the crate's eight modules import nothing from `tiler-metal` or `tiler-metal-aot`. A second backend therefore needs neither a new `*_plan.rs` module inside `tiler-build` nor a sibling orchestrator: `crates/tiler-build/tests/custom_backend` is a non-Metal producer that shares no code with the Metal path, and `crates/tiler-build/Cargo.toml` declares no `[dev-dependencies]`, so it reaches that path through the crate's public API and ordinary dependency closure alone.

**The block above is unchanged by it, and the reason is worth stating because the promotion is easy to over-read.** `tiler-build` still names `tiler-metal` and `tiler-metal-aot` unconditionally, and no member of this workspace declares a `[features]` section, so a non-Metal consumer of `tiler-build` still compiles the Metal emitter and a process-spawning Apple toolchain driver. What the promotion removed is the *authoring* choice between a second in-crate module and a sibling orchestrator; the packaging cost survives it, and narrowing that cost is separate work no accepted record has yet decided. The promoted functions and their parameter types remain a public boundary requiring Tom's acceptance under ADR 0075, which has not happened: the surface carries a coordinator's provisional acceptance recorded on 2026-08-01, under names it has since outgrown, and [`accept-the-neutral-build-orchestration-boundary`](../tickets/accept-the-neutral-build-orchestration-boundary.md) is where that decision belongs.

`tiler` and `tiler-macros` are the inline Rust frontend pair admitted by [ADR 0088](decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md). They are two crates because Rust permits a `proc-macro` crate to export nothing but macros: `tiler-macros` implements the expansion, and `tiler` is the one crate an inline-frontend consumer names, re-exporting `tensor!` and owning the absolute paths generated tokens spell. This does not make `tiler` the accepted facade for consumers that construct and compile arbitrary semantic programs; the general graph and compiler surfaces currently remain in `tiler-ir` and `tiler-compiler`, and their eventual coherent facade is a separate public-boundary decision.

The `tiler-macros` → `tiler-metal-aot` edge is where the pair's reasoning concentrates, because the placement is the decision rather than the edge. ADR 0049 requires every inline invocation to carry a canonical `ArtifactFamilySelection`, whose sole encoder is `tiler_metal_aot::family`; copying it would create a second authority over one identity subject, and moving it beneath the driver would spend the empty closure ADR 0077 item 2 decides, so the frontend must depend on the driver. It is the macro crate that pays for it, because a `proc-macro` crate and its dependencies are built for the host and never enter a consumer's target build graph. The same edge on `tiler` would compile a process-spawning Apple toolchain driver into every consumer on every platform and publish Apple backend policy on a consumer-neutral boundary — the cost ADR 0077 item 4 already refused for `tiler-metal`. The driver's empty closure is untouched: this edge points at it, not out of it.

Nothing in the workspace may depend on the frontend, which is why `tiler` is deliberately absent from `[workspace.dependencies]` with the reason written at the point of absence. An inward edge would put a frontend's macro, grammar, and expansion machinery inside the compiler's dependency closure, which is the coupling the crate split exists to prevent.

`tiler-conformance` is the cross-layer evidence member admitted by [ADR 0106](decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md), and its position is the frontend's inverted. Nothing depends on it and nothing may, so it is the top of the graph on the evidence side as `tiler` is on the consumer side, and that is what makes its wide row cost nothing: its closure reaches no consumer, so the edges are **normal rather than development** — a decided inversion of what every other member does, because a normal edge states what the crate is for where a development edge would record only what its tests happen to reach. `tiler-cache` and `tiler-digest` are reached transitively and deliberately not named, on the same discipline that keeps `tiler-ir` off the loader's row. A reverse dependent would mean the crate had begun exporting a harness as a library, which is a public-boundary decision under ADR 0075 rather than an edge a manifest may acquire.

Two consequences of that position are worth reading off it rather than rediscovering. The frontend pair is unreachable from here — `crates/tiler/tests/dependency_direction.rs` refuses it against the merged `Cargo.lock` edge list, in the development spelling as much as the normal one — so a program under conformance is built through `tiler_ir` and handed to the compiler directly, and the inline macro path stays covered where it already is, in `crates/tiler/tests/facade/`. And it is one of two members that do not inherit the workspace lint table, by exactly one lint; the other is `prototypes/serial-sum-run`. `unsafe_code = "forbid"` cannot reach `MTLBuffer` storage — the raw pointer `metal::Buffer::contents` returns is the only route to it — and no inner attribute relaxes `forbid` at any scope, so while it stood the member could hold no device-reaching items at all. Tom decided on 2026-08-07, on [`decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access`](../tickets/decide-the-conformance-crate-s-unsafe-lint-level-for-device-buffer-access.md), that this crate restates the table with `unsafe_code = "deny"` instead: named `#[allow(unsafe_code)]` at individual sites and **never at the crate**, with FFI memory management against Metal as the only admitted justification, and isolation in one module as a design constraint rather than a preference. This crate's population is two sites, both in `crates/tiler-conformance/src/device_buffer.rs`, each carrying what [ADR 0079](decisions/0079-permit-unsafe-code-case-by-case-at-named-sites.md) requires. `crates/tiler/tests/workspace_unsafe_sites.rs` cross-checks the explicit member list against Cargo's actual package set, enumerates every target root and supported local source-loading edge, and lexically pins all four workspace permissions by path, complete item signature, and exact reason, so an implicit member, inactive hidden source, addition, move, removal, or re-justification is a red test rather than an absorbed edit. Rustc suppresses unsafe-code diagnostics from external macro expansions even under a forced warning, including under inherited `forbid`; the compiler-backed half therefore stays deliberately scoped to the two `deny` packages, where it checks every ordinary/test target and pins the six visible source operations by package, target, target root, operation source, and multiplicity. A separate all-member source-language inventory closes workspace-authored generators and invocations: fifteen private local `macro_rules!` definitions, the guarded `tensor!` exporter and facade re-export, 73 fixture invocations plus one rustdoc invocation, and the exact compiler/std macro, attribute, and derive vocabularies. Custom/path-qualified invocations, workspace-owned local declarative macro exports or re-exports, additional procedural exporters, namespace aliases and shadowing imports, dynamic names, and unsupported rustdoc source forms fail closed. Cargo metadata derives thirteen doctest-enabled library/proc-macro roots; each injects `forbid(unsafe_code)` into rustdoc's extracted test crates, whose macro language is scanned separately. Compiler-builtin generated implementation details and dependency internals are outside ADR 0079's workspace-authored inventory. The other fourteen members keep `forbid`; `crates/tiler/tests/workspace_lint_inheritance.rs` holds that partition and the two diverging members run the same exact table reader.

This is an unstable prototype packaging profile, not the final published crate set. It deliberately omits Candle and reusable Metal-*runtime* crates until the proof reaches those boundaries. `tiler-runtime` is not one of those: ADR 0077 states the test that clause applies — a component that "never touches a live device, an `MTLDevice`, or a pipeline state" is not the reusable Metal-runtime crate the clause withholds — and the loader meets it, so it is admitted on the clause's own terms rather than as an exception to it. The withheld crate remains withheld; what is admitted is the backend-independent half, which is not Metal-specific at all.

The clause previously withheld a "generalized cache" as well, and [ADR 0082](decisions/0082-admit-tiler-cache-as-the-expansion-cache-owner.md) **amends** it rather than reading `tiler-cache` out of it. That distinction is deliberate: the loader was admitted by applying a stated test, and no equivalent test admits the cache, because ADR 0050's expansion cache *is* the thing the clause named. What changed is that the clause and the ownership table were found to be jointly unsatisfiable — the cache was assigned to a crate whose decided empty closure cannot reach the governed digest ADR 0050 requires it to validate against — so the clause is superseded on the point rather than reinterpreted. It continues to withhold every other cache: a runtime pipeline-state cache, a compiler plan cache, and a general-purpose content-addressed store are each still outside the profile.

The clause also withheld "frontend" and "proc-macro" crates, and [ADR 0088](decisions/0088-admit-tiler-and-tiler-macros-as-the-frontend-pair.md) **amends** it on the same terms ADR 0082 used rather than the terms ADR 0081 used. The distinction is the one that matters when reading the clause later: the loader was admitted because ADR 0077 stated a test — no live device, no `MTLDevice`, no pipeline state — that the loader passes, so the clause admitted it on its own wording. No such test is available here, because `tiler` and `tiler-macros` *are* the frontend and proc-macro crates the clause named. The clause is therefore superseded on this point, and it continues to withhold what it still names: `tiler-candle` is unadmitted, and the reusable Metal-runtime crate stays outside the profile.

[ADR 0106](decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md) needs neither move, and recording *that* is the point of this paragraph, because the three admissions above used two different mechanisms and a reader must be able to see which one each admission spent. `tiler-conformance` is none of the five crates the clause names, so the clause is neither amended nor applied — it is untouched. The nearby reading to reject is that the crate passes ADR 0077's stated test. It created no device object when it was admitted only because it contained no code, and its whole purpose is a run that executes on one — which has since happened: `crates/tiler-conformance/src/dispatch.rs` and `src/envelope/apple.rs` obtain an `MTLDevice`, build libraries and compute pipeline states, and submit command buffers, so the test is no longer passed. ADR 0106 said in its own item 4 that it would stop being, and the admission never rested on it. What keeps the member outside the withheld clause now that its device half exists is that it is not *reusable* — nothing may depend on it — so it publishes no runtime boundary and offers no component a device capability. The clause withholds a library other components would reach for; this is a test host nothing may reach, and it is not precedent for admitting the crate the clause still withholds.

ADR 0067 supersedes ADR 0057's stable Rust 1.89 floor. The prototype retains
Rust 2024 but uses the exact `nightly-2026-07-19` toolchain so its optional exact
shape evidence can use dependent array const parameters. `rust-toolchain.toml`
is authoritative; the workspace does not claim stable-compiler compatibility
while those features are required. Cache locking remains behind an internal
adapter even though the selected nightly includes the Rust 1.89 standard-
library locking API; `crates/tiler-cache/src/expansion/lock.rs` is that adapter,
and it names the reason — Rust documents that the mapping of `File::lock` to a
platform primitive may change and that the lock may be advisory, so the
primitive is named in one place rather than at each call site.

Nightly upgrades are deliberate migrations, not rolling-channel updates. The
candidate pin must pass the shape-evidence conformance harness alongside the
governed pin before replacement. This toolchain choice does not authorize
unstable proc-macro APIs or make Rust evidence part of semantic or artifact
identity.

## Dependency direction

The pipeline below is drawn as **production and consumption**. Every arrow means
that the value named beside it flows from the component that produces it to the
component that consumes it, and no arrow is a Cargo dependency edge.

```text
frontend integrations      public operation definitions
          │                              │
          │ semantic tensor graph        │ registered capabilities
          └──────────────┬───────────────┘
                         ▼
                      tiler-ir
                         │
                         │ compilation request
                         ▼
                   tiler-compiler
                         │
                         │ verified IR products
              ┌──────────┴──────────┐
              ▼                     ▼
        tiler-artifact       backend emitters
              │                     │
              │ artifact            │ emitted target source
              ▼                     ▼
       runtime adapters      target AOT tools
```

**Flow direction and dependency direction are frequently opposite, which is why
this section draws only one of them.** A runtime adapter consumes an artifact
but *depends on* `tiler-artifact`; a backend emitter produces source that target
AOT tooling consumes, while the only Cargo edge between those two crates today
runs from the emitter to the tooling and exists solely as a development
dependency. Reading a flow arrow as a dependency claim inverts the first case and
asserts, in the second, exactly the normal edge the packaging profile forbids.

Intra-workspace Cargo edges belong to the accepted packaging profile above, and
this section is deliberately not a second copy of it. The emitter/AOT pair in particular must not
be read here as a dependency claim in either direction: the `tiler-metal` →
`tiler-metal-aot` edge is development-only for the two reasons the profile
states, and the eventual `tiler-metal-aot` → `tiler-metal` production direction
is reserved and unbuilt.

What this section does constrain is which components may know about which, including roles that no workspace crate has yet. The runtime adapter must not link the optimizer merely to execute a compiled artifact. Backend emitters do not own frontend syntax or runtime storage objects. Target AOT tooling owns external compiler invocation. The build-time orchestrator owns the sequence that assembles the artifact and accepts or publishes it through the expansion cache. The compiler core must not know about Candle storage objects, einops syntax, or a particular artifact-delivery workflow.

**Fact — an accepted record adds one rule to that list for a role no crate holds yet.** [ADR 0090](decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), accepted 2026-07-31, decides that a physical-provider crate depends *inward* — on `tiler-ir` for the proposal body and on `tiler-compiler` for the seam — and that nothing in the workspace depends on a provider crate, the compiler holding installed providers as values rather than as a dependency. It restates the three rules above rather than replacing them.

The optimizer-specific instance of the compiler-core rule — the four surfaces physical-plan enumeration and selection may consult, so that no selection logic is rewritten when an execution tier or device family arrives, and the review obligation that is the only thing checking it — is stated once in [the optimizer contract](compiler/optimizer.md#the-four-surfaces-the-optimizer-may-consult).

## Proposed initial Rust/Metal integration composition

One macro invocation can produce multiple region candidates and complete one-
or multi-kernel `KernelProgram`s. All entry points required by the selected
`ProgramPortfolio` are compiled into one invocation-local metallib:

```text
SemanticTensorGraph
  ├─ region A ─┬─ schedule A1
  │            └─ schedule A2 (guarded fast path)
  └─ region B ──── schedule B1

selected ProgramPortfolio
  ─► macro-local metallib + manifest + routing policy
  ─► embedded byte-string literals in returned Rust tokens
```

Program variants specialize high-value choices such as vector width, alignment,
reduction strategy, and region boundaries. A program may contain one kernel or an
ordered/dependent set with temporary buffers. Runtime dimensions, strides, and
offsets should remain ABI parameters unless specialization is deliberately
selected. A portfolio carries a deterministic, versioned routing policy because
several compatible plans may have different extent-dependent costs.

Equivalent invocations share compilation work through their content hash. The
architecture does not initially require crate-wide collection or metallib
aggregation. Binary-level deduplication of identical embedded bundles is a
measured optimization, not a correctness dependency.

## Expansion-time composition

The proc macro synchronously invokes the build-time orchestrator, which performs:

```text
inline tokens
  -> SemanticTensorGraph
  -> verified optimization and scheduling
  -> deterministic MSL + manifest
  -> artifact identity
  -> cache hit: load bytes
     cache miss: lock, xcrun metal/metallib, validate, atomic publish
  -> emit embedded manifest/metallib byte literals and fallback expression
```

The compiler cache is disposable and is never referenced by runtime code. The
generated Rust artifact is self-contained. External-tool failure becomes a
source-spanned macro compilation error.

## Runtime composition

At runtime the adapter:

1. computes output shape metadata;
2. validates device, rank, dtype, dimensions, strides, and offsets;
3. selects a compatible precompiled program variant using the routing policy;
4. prepares every required per-device pipeline before encoding;
5. allocates output and declared temporary storage;
6. evaluates and encodes each dependency-ordered kernel step with its ABI and
   launch formula into Candle's active command stream;
7. retains temporary lifetimes through their final GPU use;
8. returns the output without synchronously waiting.

If no variant's preflight guards hold, the Tensor-level integration invokes a
defined fallback rather than entering an unsafe custom operation. Launch-time
artifact or encoder failures normally return errors, because retrying a graph
after device side effects may not be safe.

Pure view results are a separate physical result mode, not a zero-work kernel
artifact. The initial custom-op path produces one newly allocated output; view
return plans are deferred until the runtime integration can return aliased
storage and layout explicitly.

## Future opaque implementations

Not every semantic operation should eventually be implemented as primitive
scalar work. After optimizer conformance and mature boundary-property and
analytical-cost authorities, the physical planner and `KernelProgram` may admit
reviewed `OpaqueCall` implementations with explicit boundary contracts, target
requirements, exact function/accuracy/special-value behavior, and three
separate typed evidence classes: exact or proven `ResourceRequirements` for
hard feasibility; uncertain resource-pressure estimates with provenance and
`Unknown` (such as registers, occupancy, and source size); and analytical cost
estimates with model provenance and `Unknown`. Unknown pressure estimates never
prove feasibility, and unknown cost never silently wins. Examples include an
optimized matrix multiplication or a
handwritten reduction. These form deliberate fusion boundaries unless a
backend-specific implementation rule can legally absorb adjacent operations.
Opaque execution effects order physical stages; they do not introduce hidden
effects into the initial pure semantic graph.

The implementation owner is
[`implement-opaque-physical-call-providers`](../tickets/implement-opaque-physical-call-providers.md).

## Architectural constraints

- Every durable representation is deterministic and schema-versioned.
- Every lowering boundary has a verifier.
- Artifact identity includes semantics, schedule, ABI, guards, target, and
  compiler configuration.
- Launch policy travels with the artifact and is never reconstructed from
  tensor element count alone.
- Source spans survive long enough to explain invalid frontend expressions and
  failed specialization assumptions.
- Numerical transformations are conditioned on an explicit numerical contract.
- Runtime layout metadata is never assumed from logical shape alone.

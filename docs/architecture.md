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
    deterministic_search_and_artifact_budgets,
    compilation_options,
}
```

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

The bounded P0 physical frontier admits only checked `ScheduledKernel` values
and rejects the other variants explicitly while retaining this additive
sum-type seam. Opaque physical calls are a later reviewed extension, not part of
the first frontier proof.

Every executable body also carries the selected numerical realization,
machine-checkable guarantee, and scoped evidence identity. These must refine
the region's effective operation contracts before costing.

Index, schedule, and structured-kernel identities describe canonical
structural content. A compiler-owned checked refinement binds index structure
to a particular region occurrence, exact boundary/access mappings, reached
semantic definitions, selected provider provenance, and evidence. Complete
program identity—not a nested whole-graph digest inside every structural
object—proves occurrence coverage and executable composition. ADR 0072 owns
this identity layering.

A locally slower implementation may provide a layout that removes a downstream
conversion. Multi-pass reductions are `KernelSubprogram` bodies rather than one
oversized `KernelSchedule`; opaque library calls need not invent a schedule.

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
materializations, temporaries, and opaque calls. A guarded `ProgramPortfolio`
may retain several complete programs for different runtime applicability
regions.

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
annotations.

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

ADR 0070 accepts these dependency and verifier-ownership boundaries. They are
not necessarily the final published-crate layout, and individual field sets
remain experimental until their dedicated implementation tickets land.

| Component | Responsibility | Forbidden dependencies |
| --- | --- | --- |
| `tiler-ir` | Public semantic graph and operation-extension contracts; experimental index, schedule, kernel, executable-program, `BufferPlan`, and `AbiExpr` representations; authoritative IR verifiers and pure checked expression semantics | Frontend syntax, reference execution, artifact encoding, runtime fact binding, Candle, and Metal runtime APIs |
| `tiler-reference` | Host reference values, executable semantic-operation capabilities, interpreter traversal, and conformance utilities | Optimizer, scheduler, backend, and live device APIs |
| `tiler-compiler` | Normalization, rule engine, fusion planning, index lowering, schedule search, costing | Candle |
| `tiler-artifact` | Versioned target-neutral artifact/ABI encoding, compatibility, runtime fact binding, failure classification, and backend-payload mappings | Candle, optimizer, and Metal device APIs |
| `tiler-metal` | Pure structured-kernel-to-MSL translation and Metal target metadata | Candle and Metal device APIs |
| `tiler-metal-aot` | Expansion-time Apple tool invocation, cross-process content cache, atomic publication, byte embedding | Candle tensor APIs |
| Frontend core | Translate source syntax into semantic IR and map diagnostics back to users | Backend-specific scheduling |
| Frontend proc-macro crate | Invoke frontend/compiler/AOT pipeline and emit artifact plus runtime/fallback tokens | Candle runtime internals beyond its public adapter |
| `tiler-candle` | Layout validation, output allocation, pipeline cache, ABI binding, dispatch, fallback | Optimizer internals |

Initially, semantic, index, schedule, and kernel IRs may be modules in one
crate. Splitting every representation into a crate before its API stabilizes
would add ceremony without improving the dependency graph.

Shared compiler IR uses checked public builders with private storage. Local
insertion failures are reported immediately; consuming `build()` performs the
whole-object verifier and returns an opaque immutable verified product or a
typed failure retaining builder ownership. Compiler passes, third-party plan
producers, artifact decoders, and backends use this same verifier authority.
Only verified products cross those boundaries. See ADR 0071.

## Accepted prototype packaging profile

ADR 0065 refines ADR 0056 after the evaluator implementation exposed a real
consumer boundary. The prototype uses five reusable libraries and two
non-published proof executables:

```text
tiler-ir       -> []
tiler-reference -> [tiler-ir]
tiler-artifact -> [tiler-ir]
tiler-compiler -> [tiler-ir]
tiler-metal    -> [tiler-ir, tiler-artifact]

prototype-compile -> [tiler-ir, tiler-reference, tiler-artifact, tiler-compiler, tiler-metal]
prototype-run     -> [tiler-artifact, platform Metal bindings]
```

This is an unstable prototype packaging profile, not the final published crate
set. It deliberately omits frontend, proc-macro, Candle, generalized cache, and
reusable Metal-runtime crates until the proof reaches those boundaries.

ADR 0067 supersedes ADR 0057's stable Rust 1.89 floor. The prototype retains
Rust 2024 but uses the exact `nightly-2026-07-19` toolchain so its optional exact
shape evidence can use dependent array const parameters. `rust-toolchain.toml`
is authoritative; the workspace does not claim stable-compiler compatibility
while those features are required. Cache locking remains behind an internal
adapter even though the selected nightly includes the Rust 1.89 standard-
library locking API.

Nightly upgrades are deliberate migrations, not rolling-channel updates. The
candidate pin must pass the shape-evidence conformance harness alongside the
governed pin before replacement. This toolchain choice does not authorize
unstable proc-macro APIs or make Rust evidence part of semantic or artifact
identity.

## Dependency direction

```text
frontend integrations ─► tiler-ir ◄─ tiler-compiler
                              ▲              │
                              │              ▼
                        public op       verified IR products
                        definitions       │             │
                                          ▼             ▼
                                   tiler-artifact   backend emitters
                                          ▲             │
                                          │             ▼
                                  runtime adapters  target AOT tools
```

The runtime adapter must not link the optimizer merely to execute a compiled
artifact. Backend emitters do not own frontend syntax or runtime storage
objects. Target AOT tooling owns external compiler invocation and caching. The
compiler core must not know about Candle storage objects, einops syntax, or a
particular artifact-delivery workflow.

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

The proc macro synchronously performs:

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

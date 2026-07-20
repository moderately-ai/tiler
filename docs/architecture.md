# System architecture

**Status:** proposed

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
        │
        ├── derive iteration/access representation ─────┐
        │                                               │
        └◄─ ImplementationFrontier(region, target) ◄────┘
        │ select compatible region implementations
        ▼
KernelProgram / guarded ProgramPortfolio
        │ structured lowering
        ▼
structured kernel IR
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
| Scoped extent symbols, typed root bindings, admitted constraint language, and sourceability | [Shape environment contract](research/shapes/shape-environment-contract.md) | ADR 0008 |
| One semantic authority plus separately versioned optional capabilities in an explicit frozen registry | [Operation-extension research](research/extensions/operation-extension-surface.md) and [API spike](research/extensions/operation-extension-api.md) | ADR 0044 |
| Proc-macro provider visibility bounded by the host dependency graph | [Proc-macro visibility experiment](research/extensions/proc-macro-extension-visibility.md) | ADR 0045 |

These decisions constrain the compiler core. They do not make serialized IR a
public compatibility promise, add effectful operations, or require a proc
macro for non-Rust consumers.

## Consumer-independent compilation request

One initial compiler invocation consumes one verified semantic graph through a
conceptual `CompilationRequest`:

```text
CompilationRequest {
    semantic_graph,
    numerical_contract,
    shape_environment,
    target_profiles,
    frozen_operation_registry_and_provider_fingerprints,
    deterministic_search_and_artifact_budgets,
    compilation_options,
}
```

The semantic graph remains backend-neutral. `shape_environment` contains the
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
member operations. Actual materialization is selected by region
implementations and the complete kernel program.

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
estimate. Its implementation
body is one of:

```text
ScheduledKernel(ScheduledRegion)
KernelSubprogram(stages, internal temporaries, dependencies)
OpaqueCall(call contract)
View(alias/metadata result)
```

Every executable body also carries the selected numerical realization,
machine-checkable guarantee, and scoped evidence identity. These must refine
the region's effective operation contracts before costing.

A locally slower implementation may provide a layout that removes a downstream
conversion. Multi-pass reductions are `KernelSubprogram` bodies rather than one
oversized `KernelSchedule`; opaque library calls need not invent a schedule.

The program planner selects a compatible covering `RegionPartition` and one
implementation per selected region only after candidate regions have legal
schedules and costs. Conversely, boundary and materialization choices determine
which schedules the local scheduler can consider. This feedback is why the
architecture is hierarchical planning rather than “choose fusion, then
schedule.”

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

## Proposed components

These are dependency boundaries, not necessarily the initial published-crate
layout.

| Component | Responsibility | Forbidden dependencies |
| --- | --- | --- |
| `tiler-ir` | Public semantic graph and operation-extension contracts; internal/experimental index, schedule, and kernel representations; verifiers and reference evaluator | Frontend syntax, Candle, and Metal runtime APIs |
| `tiler-compiler` | Normalization, rule engine, fusion planning, index lowering, schedule search, costing | Candle |
| `tiler-artifact` | Versioned target-neutral artifact/ABI schema and checked expression evaluator | Candle, optimizer, and Metal device APIs |
| `tiler-metal` | Pure structured-kernel-to-MSL translation and Metal target metadata | Candle and Metal device APIs |
| `tiler-metal-aot` | Expansion-time Apple tool invocation, cross-process content cache, atomic publication, byte embedding | Candle tensor APIs |
| Frontend core | Translate source syntax into semantic IR and map diagnostics back to users | Backend-specific scheduling |
| Frontend proc-macro crate | Invoke frontend/compiler/AOT pipeline and emit artifact plus runtime/fallback tokens | Candle runtime internals beyond its public adapter |
| `tiler-candle` | Layout validation, output allocation, pipeline cache, ABI binding, dispatch, fallback | Optimizer internals |

Initially, semantic, index, schedule, and kernel IRs may be modules in one
crate. Splitting every representation into a crate before its API stabilizes
would add ceremony without improving the dependency graph.

## Dependency direction

```text
frontend integrations ─► tiler-ir ◄─ tiler-compiler ─► backend integrations
                              ▲              │                  │
                              │              ▼                  ▼
                        public op       tiler-artifact     target AOT tools
                        definitions           ▲
                                              │
                                      runtime integrations
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

## Core opaque implementations

Not every semantic operation should be implemented as primitive scalar work.
The physical planner and `KernelProgram` admit `OpaqueCall` implementations
with explicit boundary contracts, target requirements, resource/hazard
metadata, exact function/accuracy/special-value behavior, and costs, for
example an optimized matrix multiplication or a
handwritten reduction. These form deliberate fusion boundaries unless a
backend-specific implementation rule can legally absorb adjacent operations.
Opaque execution effects order physical stages; they do not introduce hidden
effects into the initial pure semantic graph.

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

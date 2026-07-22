---
schema: "tiler-doc/v1"
id: "tiler.research.target-profiles.physical-feasibility-model"
kind: "research"
title: "Target profiles and phased physical feasibility"
topics: ["targets", "feasibility", "gpu"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "adopted"
implementation_status: "partial"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.architecture", "tiler.contract.fusion-and-scheduling", "tiler.contract.cost-model"]
adopted_by: ["ADR-0043"]
ticket: "target-profile-feasibility-model"
---

# Target profiles and phased physical feasibility

**Status:** research basis for ADR 0043
**Ticket:** `target-profile-feasibility-model`

**Evidence boundary:** the phased model is primary-source synthesis. The
private compiler slice exercises one conservative prototype profile, but this
report has no retained executable spanning declared, compile-refined,
pipeline-refined, and live-device feasibility. It therefore does not claim
`executable-model` evidence for that full contract.

## Conclusion

Device mapping does not turn the semantic tensor DAG into a hypergraph. A
schedule is a normal physical candidate with exact resource use, target
requirements, and estimates. Cross-dimensional restrictions are typed
feasibility predicates over that candidate and a target capability environment.
For example, a workgroup must satisfy each dimension limit, a product limit,
and a static-plus-dynamic local-memory limit simultaneously. A relation among
several facts is not evidence that graph edges need hyperedge semantics.

No one static profile can decide every constraint. Metal function-specific
thread limits exist only after pipeline creation; CUDA register and kernel
limits depend on the selected or JIT-compiled image; scalable CPU vector length
may remain runtime-parametric. Feasibility therefore refines monotonically
through named phases.

## Target-neutral records

```text
DeclaredTargetProfile {
  profile_key,
  descriptor_digest,
  compatibility_contract,
  compile_guarantees: TypedCapabilitySet,
  data_layout,
  execution_model,
  memory_space_model,
  vector_model,
  phase_schemas: [(CapabilityPhase, QueryOrEvidenceSchema)],
  artifact_execution_contract,
  feasibility_rule_set_key,
  tuning_model_key,
}

KernelRequirements {
  capability_predicates,
  exact_or_proven_resource_bounds,
  deferred_predicates: [(availability_phase, predicate)],
  runtime_applicability_guards,
  resource_estimates,
}

CapabilityFact<T> {
  capability_key,
  value,
  availability_phase,
  validity_scope,
  authority,
  provenance,
}

Feasibility =
  Proven
| Deferred { checks_grouped_by_phase: NonEmptySet<DeferredCheck> }
| Rejected { rule, reason }
| Unknown { missing_fact_or_proof }
```

Capability keys are governed, typed, versioned, canonically encoded, and
bounded. Each defines its value type, meaning, earliest availability, validity
scope, and accepted authority. A free-form property map cannot prove
correctness. Multivariate rules also have governed identities; a collection of
scalar maxima alone cannot express every launch constraint.

Facts that prove feasibility use normative guarantees, compiler/prepared-
kernel facts, live queries, or proven derivations. Estimated and measured facts
remain in the resource/cost model and cannot be inserted under a hard authority
label.

The artifact execution contract names representation (`NativeImage`,
`PortableTargetIR`, or another governed form), compatibility rules, compiler or
driver minima, and whether a named runtime translation/JIT provider is required
and permitted. Metal's normal metallib-to-device pipeline preparation and a
CUDA cubin are therefore distinguishable from PTX driver JIT. A backend cannot
silently introduce a runtime compiler path that the product profile forbids.

The phases are:

1. **`CompileProfile`:** conservative guarantees used to generate an artifact;
2. **`ArtifactEvidence`:** facts established by offline compilation or
   reflection;
3. **`LiveDevicePreflight`:** facts queried for one device/context;
4. **`PreparedKernelPreflight`:** facts for the selected entry point,
   specialization, descriptor/configuration, and live device after
   library/module/function/pipeline preparation; and
5. **`LaunchPreflight`:** evaluated grid, group, dynamic-memory, ABI, input
   binding, required-allocation specification, and alignment facts before
   resource acquisition or encoding.

`RoutingCommit` is the named no-alternate-plan boundary after all route-sensitive
`LaunchPreflight` checks and final plan selection but before output/scratch
resource acquisition or encoding. `Deferred` is legal only when every named
query can run before that commit. Intrinsic candidate assessment checks only
that each deferred query is admissible; the later portfolio/integration
verifier proves equivalent coverage for every deferred-rejection region through
another variant or declared external fallback. Checks requiring actual plan-
specific allocations are guaranteed by
the allocator contract or become post-commit invariants whose failure closes;
transient allocation failure is not target incompatibility. A launch or
asynchronous execution error cannot trigger graph replay.

Aggregate assessment is deterministic. Any authoritatively disproved hard
predicate yields `Rejected`. Otherwise, any unresolved predicate lacking an
admissible proof/query path yields `Unknown`. Otherwise, unresolved predicates
produce one nonempty canonical `Deferred` set grouped by phase. With none
remaining the result is `Proven`.

`Unknown` may be retained for search diagnostics but never enters an executable
frontier or manifest. With no covering proven/deferred candidate, compilation
reports the region unsupported.

Target predicates range over a typed environment of capability facts,
candidate resource requirements, evaluated launch values, ABI/layout facts,
and binding/access facts. This expresses products, sums, divisibility, and
two-sided alignment checks; it is deliberately broader than a predicate over
capability keys alone.

## Capability families

### Execution and synchronization

Execution levels are typed scopes, not universal aliases. GPU grid,
workgroup/block, subgroup/warp, and lane scopes coexist with CPU task/thread and
vector-lane scopes. A CPU worker is not modeled as a Metal threadgroup merely
to reuse a name.

A barrier or collective capability identifies participants, execution scope,
fenced memory spaces/order, convergence obligations, and supported operation.
“Barrier supported” is insufficient. Ordinary GPU dispatches provide no
implicit grid-wide barrier; cross-workgroup communication needs a supported
atomic/cooperative protocol or a dispatch boundary.

### Vectors

The vector model distinguishes `Fixed(lanes)` from
`Scalable(min_lanes * vscale)`. Legality is contextual on operation, dtype,
shape, masks/tails, address space, width, and alignment. Preferred widths and
legal widths are separate facts. GPU shader vector types are also distinct from
the hardware subgroup width.

### Memory

Addressable memory spaces have explicit scope, access, coherence, alignment,
and hard managed-capacity rules. Transparent caches are cost-model levels, not
scratch spaces into which a schedule can explicitly stage data. Threadgroup or
shared memory capacity is a hard bound; cache capacity, bandwidth, bank
conflicts, and working-set fit are normally estimates.

Storage mode is not a portable disk/RAM/VRAM placement identity. For example,
Metal private storage may use unified system memory or discrete VRAM while
remaining GPU-only through the API. Cross-device and external-storage placement
remain separate program-planning concerns.

### Resources and ABI

Hard quantities include supported instructions/dtypes/atomics, address widths,
binding or parameter ABI, group/grid dimensional limits, explicit local-memory
bytes, instruction-required alignment, checked allocation-size/address bounds,
and buffer limits. Actual allocation success is a runtime execution condition,
not a compile-profile guarantee. Register pressure, spills, occupancy above
zero, coalescing, throughput, and cache
behavior begin as estimates. A compiler or prepared-kernel query may promote a
specific resource fact into a later hard check; an analytical estimate cannot.

ABI and alignment checks are two-sided: the target supplies instruction and
ABI requirements; generated layout and byte totals are artifact/resource facts;
actual input base-plus-offset, range, accessibility, and alignment are launch
facts. Preferred coalescing alignment remains cost-only. A user-selected peak
temporary-memory budget may hard-prune search, while volatile free memory and a
recommended working set are policy/cost evidence rather than portable target
capabilities.

## Classification examples

| Dimension | Hard feasibility | Deferred fact | Cost or estimate |
|---|---|---|---|
| Threads | Per-axis/product limits; algorithmic subgroup-width assertion | Metal PSO max total threads; CUDA loaded-kernel max threads | Preferred width multiple; underfill |
| Local memory | Static + dynamic bytes, alignment, slot/count rules | Specialized pipeline/kernel static bytes and opt-in maximum | Bank conflicts; occupancy effect |
| Registers | Compiler-established launch/resource limit when exposed | CUDA registers/thread and occupancy-zero check | Pre-lowering pressure, spills, occupancy tier; Metal register use is not exposed |
| Vectors | Operation/type/address-space/alignment legality | CPU scalable width or live subgroup assertion | Preferred width and legalization cost |
| Synchronization | Scope, participants, convergence, memory fence, cooperative protocol | Live cooperative/cluster support | Barrier latency |
| ABI | Binding/parameter layout, count/bytes, access and alignment | Reflection used to validate the manifest | Packing overhead |
| Memory capacity | Explicit local capacity; buffer maximum; checked allocation size/address range; explicit user policy budget | Live buffer limit and allocator-contract facts | Recommended working set, volatile free/current allocation, cache capacity, bandwidth |

## Metal findings

Apple's [Metal feature tables](https://developer.apple.com/metal/Metal-Feature-Set-Tables.pdf)
provide family-scoped floors and ceilings, while live `MTLDevice` queries refine
device limits. A metallib contains Metal IR and does not prove live pipeline
feasibility. A specialized `MTLComputePipelineState` supplies authoritative
`threadExecutionWidth`, `maxTotalThreadsPerThreadgroup`, and
`staticThreadgroupMemoryLength`. These may differ between pipelines on one
device, so preparation must occur before `RoutingCommit` when fallback is
possible. Apple documents this runtime specialization in
[Metal libraries](https://developer.apple.com/documentation/metal/metal-libraries)
and [threadgroup sizing](https://developer.apple.com/documentation/metal/calculating-threadgroup-and-grid-sizes).

The device's recommended working-set size is a performance recommendation, not
an allocation guarantee. Metal exposes no stable planning API for exact
registers, spills, active groups, or occupancy; those remain estimates or
post-execution measurements. `threadExecutionWidth` is not an MSL vector width,
and merely choosing its multiple is generally a performance recommendation
unless the generated algorithm requires that subgroup width.

The inspected Candle checkout reinforces the phase boundary. Its Metal
pipeline wrapper (`candle-metal-kernels/src/metal/compute_pipeline.rs`) exposes
function-specific maximum threads but not execution width or static
threadgroup bytes; `src/utils.rs` launch helpers either choose that maximum or
use a fixed 1024-thread product cap. `src/kernel.rs` applies function constants
before pipeline creation and cache lookup, correctly making prepared facts
specialization-specific. `src/metal/encoder.rs` forwards dynamic threadgroup
bytes and dispatch modes, so Tiler's manifest/preflight—not those helpers—must
validate alignment, total local bytes, and the concrete launch. These are
implementation observations, not normative Metal capabilities.

## CUDA findings

CUDA similarly separates [device attributes](https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__DEVICE.html),
[loaded-function attributes and launch](https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__EXEC.html),
and [occupancy queries](https://docs.nvidia.com/cuda/cuda-driver-api/group__CUDA__OCCUPANCY.html).
Grid/block dimension limits come from the device and the selected function.
Static shared memory, registers, local bytes, maximum dynamic shared memory,
and kernel maximum threads are compiled-kernel facts. Cooperative grid sync and
clusters add multivariate live launch constraints.

A warp multiple is normally a cost heuristic, but warp width becomes hard when
the algorithm uses masks, shuffles, votes, or warp-synchronous collectives.
Occupancy above zero for the exact loaded kernel and launch is feasibility;
higher occupancy is not necessarily faster. PTX/cubin compatibility and JIT
policy belong to artifact/target requirements, and PTX JIT makes final resource
facts driver/device outputs.

## CPU and compiler precedents

LLVM [TargetTransformInfo](https://llvm.org/doxygen/classllvm_1_1TargetTransformInfo.html)
provides target legality and cost queries without changing program semantics;
[VPlan](https://llvm.org/docs/VectorizationPlan.html) retains scalar and vector
candidates through legality and costing. LLVM and MLIR explicitly represent
[scalable vectors](https://llvm.org/docs/LangRef.html#scalable-vector-type)
rather than pretending a fixed compile-time lane count.

MLIR [DLTI](https://mlir.llvm.org/docs/Dialects/DLTIDialect/) and IREE
[HAL](https://iree.dev/reference/mlir-dialects/HAL/) support scoped target and
device descriptions outside tensor operation semantics. Their flexible maps
are useful precedent for extensibility, but Tiler requires governed typed keys
for facts that establish correctness.

## Identity and testing consequences

The declared profile, capability/rule schemas, kernel requirements, compiler
target/options, artifact execution/translation policy, feasibility descriptor,
and output-affecting artifact facts participate in plan and artifact identity.
The tuning-model key is selection provenance unless it changes the emitted
portfolio or embedded manifest. Live-device and prepared-kernel facts scope
runtime caches and routing records; they are not portable semantic identity.
Calibration and measurements carry model/provenance identity but never become
capability truth without a separately accepted proof/query contract.

Tests cover every hard boundary at minus/equal/plus one, missing and dishonest
providers, fixed/scalable vector legality matrices, barrier scope and
convergence, deferred-query timing, specialization-specific facts, generic
fallback retention, and the invariant that an estimate never proves legality.

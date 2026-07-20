# 0043: Use typed phased target feasibility

**Status:** accepted

## Context

GPU feasibility depends on relations among schedule topology, memory spaces,
barriers, ABI, device limits, compiled-kernel resources, and launch values.
Some facts are conservative compile-profile guarantees; others exist only for
a live device or prepared specialized kernel. CPU fixed/scalable SIMD presents
the same static/runtime split without sharing the GPU execution hierarchy.

Putting these dimensions on semantic tensor nodes would mix meaning with
implementation and still would not express multivariate constraints. Treating
one target profile as omniscient would reject valid deferred plans or accept
plans based on estimates.

## Decision

Target feasibility is a typed, monotonically refined physical contract outside
the semantic tensor graph. It consists of:

- a governed, versioned declared target profile containing compatibility,
  conservative compile guarantees, data layout, execution/memory/vector
  models, per-phase query/evidence schemas, artifact representation and runtime-
  translation policy, feasibility-rule identity, and tuning-model identity;
- per-implementation target predicates, exact/proven resource requirements,
  deferred predicates, applicability guards, and estimates;
- typed capability facts with explicit availability phase, validity scope,
  authority, and provenance; and
- a launch instance that evaluates runtime formulas and bindings against the
  accumulated facts before work is committed.

Capability and feasibility-rule keys are governed, typed, canonically encoded,
and bounded. A free-form backend property bag cannot prove correctness.
Predicates range over typed capability, candidate-resource, launch, ABI/layout,
and binding/access facts. Relations such as per-axis plus product limits,
static-plus-dynamic local memory, divisibility, and two-sided alignment are
ordinary typed predicates; they do not require a hypergraph IR.

Feasibility has four outcomes: `Proven`, `Deferred`, `Rejected`, and `Unknown`.
Any disproved hard predicate rejects; otherwise a predicate with no admissible
proof/query path is unknown; otherwise all unresolved checks form one nonempty
canonical deferred set grouped by phase; no remaining checks means proven.
An `Unknown` candidate may remain in search/explain state but cannot enter an
executable frontier or manifest. If no legal candidate covers a semantic
region, compilation reports unsupported rather than emitting uncertainty.

`RoutingCommit` is the named no-alternate-plan boundary after route-sensitive
launch preflight and final plan selection but before output/scratch acquisition
or encoding. `Deferred` is retained only when every query runs before that
commit. Intrinsic candidate assessment need not know the future portfolio.
Before packaging, the portfolio/integration verifier proves semantically and
numerically equivalent coverage for every deferred-rejection region through
another variant or an explicitly available external fallback. Allocation-
dependent conditions are guaranteed by the allocator contract or are post-
commit invariants whose failure closes. Transient allocation or execution
failure is not target incompatibility and never becomes a fallback signal.

The ordered availability phases are `CompileProfile`, `ArtifactEvidence`,
`LiveDevicePreflight`, `PreparedKernelPreflight`, and `LaunchPreflight`.
Pipeline-
derived facts may constrain physical routing but cannot affect semantic shapes
or values. A physical target capability becomes semantic only when a graph
explicitly declares it as a versioned `TargetProperty` root binding under ADR
0008. Such a semantic property is admitted and bound exactly once from
`CompileProfile` or `LiveDevicePreflight` before semantic shape evaluation and
plan routing; later physical facts cannot overwrite or refine it.

Hard requirements remain separate from estimates. Supported operations,
address spaces, synchronization scope/convergence, ABI, instruction alignment,
launch dimensions, explicit local memory, and proven resource ceilings are
hard. A backend-specific authoritative prepared/launch rule may prove a resource
configuration impossible, such as zero resident CUDA blocks. Register pressure,
spills, occupancy above feasibility, cache behavior, bandwidth, throughput, and
preferred vector widths are otherwise cost evidence.

Execution and vector models are explicit. GPU workgroup/subgroup scopes are not
aliases for CPU worker/vector scopes. Fixed and scalable vector shapes are
distinct, and vector legality depends on operation, dtype, masks, address
space, width, and alignment. Transparent caches are cost-model levels, not
explicitly stageable memory spaces.

Target requirements and the feasibility descriptor/rule identity participate
in plan and artifact identity. Artifact representation, compatibility, and
runtime-translation policy are explicit. This does not authorize runtime source
compilation: the initial product still forbids it, while a backend may declare
required device translation of an AOT target-IR artifact such as a metallib.
Live-device and prepared-kernel facts
scope runtime caches and routing provenance, not portable semantic identity.
The tuning-model key is selection provenance unless it changes emitted content;
cost calibration and measurements never serve as capability truth themselves.

## Consequences

- One physical planning interface can support Metal, CUDA, and CPU/SIMD
  without forcing them into one execution topology.
- Function-specific Metal/CUDA limits can be checked safely at preflight rather
  than guessed offline.
- Generic/scalar plans remain available when specialized facts are absent.
- Explain output distinguishes static proof, deferred query, rejection,
  unknown fact, and cost disadvantage.
- Target extensions require governed schemas/providers rather than arbitrary
  maps used as proof.

## Alternatives considered

Annotating semantic nodes with device properties makes target choice part of
tensor meaning and cannot model candidate-specific resource relations. One
flat target struct cannot represent specialization- and phase-dependent facts.
Treating unknown as false loses valid deferred plans; treating it as true is
unsound. Treating occupancy, cache fit, or register estimates as hard limits
confuses performance heuristics with correctness.

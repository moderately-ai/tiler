---
schema: "tiler-doc/v1"
id: "tiler.contract.optimizer"
kind: "contract"
title: "Optimizer model"
topics: ["optimizer", "search", "planning"]
contract_status: "accepted"
implementation_status: "partial"
evidence: ["tiler.research.region-search.exhaustive-region-oracle", "tiler.research.reference.normative-reference-slice", "tiler.research.cost-model.bootstrap-cost-model", "tiler.research.program-planning.general-compilation-boundary"]
---

# Optimizer model

**Status:** accepted research contract; bounded prototype implementation

The first private compiler slice now retains complete materialized and fused
program alternatives, carries exact structural metrics, and selects the fused
program only when it strictly Pareto-dominates the baseline. Its stable policy
key makes no latency claim. Missing fusion capability, candidate-budget
exhaustion, or fused target infeasibility rejects only the fused alternative;
failure of a compiler-produced verifier remains a hard compiler error. General
memo search, partitioning, and calibrated cost estimation remain unimplemented.

The bounded slice rederives alternative identity, structural cost, KIR,
program, artifact receipt, and selection from the verified semantic/request
subject before returning a portfolio. Selection authority is the verified
alternative identity under a named cost model, not a caller-editable vector
index or stored cost. Explain records retain typed subjects, evidence class,
budget actual/limit pairs, feasibility facts, and provenanced cost values.

## Ownership boundary

This document owns planning phases, rule contracts, alternative retention,
search bounds, costing inputs, and explainability. It consumes verified IR
schemas from the IR contract and does not redefine their fields or backend
resource limits.

Tiler borrows selected techniques from property-aware database optimizers while
using a tensor operation/value DAG, access-aware fusion regions, and explicit
GPU schedules. DataFusion is useful vocabulary for semantic/executable
separation and boundary enforcement, but it is not the structural template for
Tiler's graph or search algorithm.

The contract synthesizes the [region oracle](../research/region-search/exhaustive-region-oracle.md),
[index/access model](../research/indexing/index-access-model.md),
[scheduled-region model](../research/scheduling/scheduled-region-model.md),
[whole-program plan](../research/program-planning/kernel-program-buffer-plan.md),
and [structured-kernel verifier](../research/kernel-ir/structured-kernel-ir-verifier.md).

## Compilation boundary and failure classes

Everything below is reached through one general, consumer-independent
compilation boundary over a verified `SemanticProgram` and explicit request
inputs. Under ADR 0069 there is no graph-specific entry point, `experimental`
namespace, or serial-Sum support profile. A bounded vertical slice remains a
private strategy, conformance fixture, and explain identity; its fixed region,
stage, entry, and buffer cardinalities are not request or result invariants.

The boundary returns either general target-neutral program products or a typed
outcome drawn from five distinct failure classes:

- **invalid request:** the semantic program, resolved numerical contracts,
  shapes, frozen registry, or request inputs are malformed;
- **missing compilation capability:** the program is valid, but no installed
  access, scheduling, lowering, or provider capability covers it;
- **infeasible plan:** every candidate is intrinsically invalid or rejected by
  typed target feasibility;
- **exhausted bounded search:** a declared candidate or expansion budget
  stopped exploration before a complete plan was selected; and
- **compiler IR verification failure:** a compiler-produced index, schedule,
  kernel, or program value failed its authoritative verifier.

These classes are not interchangeable. A valid program that lacks coverage is
never reported as malformed, and an unsupported case fails closed with an
explainable reason rather than being approximated to retain a fast path. A
budget that stops one growth path while complete coverage survives is an
explain reason on the selected plan, not this failure class. A verifier failure
is invalid compiler output and remains a hard error rather than a costed
rejection.

## Planning model

```text
SemanticTensorGraph
  -> deterministic normalization
  -> bounded logical exploration
  -> overlapping RegionCandidates
     |-> independent complete-cover enumeration ---------|
     `-> checked schedules + ImplementationFrontier -----|
  -> compatible complete physical-plan selection
  -> structured KIR refinement
  -> KernelProgram or guarded ProgramPortfolio
```

The optimizer must distinguish:

- **logical equivalence:** expressions compute the same tensor under a stated
  numerical policy;
- **fusion legality:** a region can be implemented correctly as one kernel;
- **physical feasibility:** a schedule fits target capabilities and resources;
- **profitability:** the complete plan is preferable to legal alternatives.

## Named stages and verifier boundaries

The initial optimizer pipeline has explicit stage names and cannot skip their
verification boundaries:

1. `VerifySemanticRequest` checks the graph, resolved numerical contracts,
   shapes, and frozen operation registry.
2. `NormalizeSemantics` produces one deterministic canonical graph.
3. `ExploreLogicalAlternatives` adds only proved contract-preserving forms.
4. `EnumerateRegionCandidates` forms connected convex semantic regions and
   retains complete singleton coverage.
5. `LowerIndexRegions` derives width-independent domains/access maps and proves
   read bounds plus exact unique ordinary writes.
6. `EnumerateCompleteCovers` independently enumerates legal whole-graph covers;
   it does not select schedules or implementations.
7. `ExploreScheduledRegions` intrinsically verifies normalized schedules for
   individual legal regions. Typed target-feasibility assessment then admits
   bounded per-region physical frontiers. This authority does not require a
   previously selected global cover.
8. `SelectCompletePhysicalPlans` joins complete covers with compatible local
   implementations, boundary contracts, proposed materializations,
   dependencies, and guards. It emits a checked selected-plan or portfolio
   receipt for cover/implementation compatibility, not final executable-program
   authority. Buffer requirements remain provisional at this stage.
9. `RefineStructuredKernels` lowers each selected scheduled kernel and proves typed,
   effect-safe refinement of exactly that schedule before backend emission.
10. `AssembleKernelPrograms` constructs verified executable programs from the
    checked physical-plan receipt and verified KIR. Only this post-KIR verifier
    authoritatively checks executable stage coverage, buffers, initialization,
    lifetimes, aliasing, storage handoffs, ABI/launch references, and routing.

Semantic, index, schedule, program/buffer, and structured-kernel verifiers have
separate authority. Target feasibility cannot repair intrinsic invalidity;
costing observes only candidates that have passed the applicable gates.
`Intrinsic` and refinement failures therefore remain invalid compiler output;
only a checked target/resource rejection can contribute to a valid empty
physical frontier.
Search implementations may interleave cover and local-frontier exploration,
feed pruning information in either direction, and lazily schedule only regions
retained by viable covers. Such feedback is implementation freedom: it cannot
make a cover receipt prove schedule feasibility, or a local frontier prove
whole-program coverage.

## Bounded hierarchical search

A Cascades-style memo is one possible implementation technique, not a committed
architecture. The durable concepts are contract-conforming semantic
alternatives, explicit region candidates, bounded implementation frontiers,
and deterministic complete-program selection. The term `memo` is reserved for
an implementation that actually groups equivalence classes and performs
goal-directed property search.

Examples of equivalent expressions include:

- consecutive reindexes versus one composed access map;
- a pointwise operation before or after a reindex when domains permit;
- alternative contraction associations for future multi-input einsum.

Recomputation, materialization, fusion, and register residency are physical
implementations of one logical DAG. They do not create new logical equivalence
groups.

The first implementation should use bounded exploration: canonical operation
and value keys, deterministic rule order, small alternative sets, dominance
pruning, and explicit search budgets. Tiny graphs should have an exhaustive
oracle in tests so heuristic completeness and plan quality can be measured
before a memo architecture is chosen.

The first deterministic safety budgets are 32 semantic occurrences per region,
8 boundary outputs, 64 live boundary/internal values, 32 candidates per seed,
8 nondominated implementations per region, and 10,000 candidate expansions per
compilation request. Producer duplication is disabled outside oracle tests in
the initial implementation. Hitting a budget stops only that growth path,
emits an explain reason, and never removes singleton/unfused coverage. These
defaults are calibration inputs, not correctness constants.

## Rule classes

### Semantic normalization

Normalization chooses a canonical form and must terminate deterministically:

- resolve axis names and ellipses;
- canonicalize reductions and output-axis policy;
- compose permutations and legal split/merge chains;
- canonicalize explicit broadcast/repeat axis mappings;
- eliminate identity reindexes and no-op casts;
- normalize constants and dtypes;
- remove dead values.

Normalization must not silently change floating-point evaluation order.

### Logical exploration

These rules add alternatives:

- push a view through a pointwise expression;
- add contract-conforming alternatives over named pointwise operations;
- choose alternative contraction associations;
- reassociate arithmetic or reductions only when numerical policy permits.

### Region-candidate formation

Region rules propose, but do not automatically select, candidates with explicit
member operations, boundary values, retained results, materialized edges, and
duplication policy:

- pointwise plus pointwise;
- reindex plus pointwise;
- pointwise prologue into a reduction;
- pointwise epilogue after a reduction;
- compatible sibling consumers as a future multi-output kernel;
- supported prologue/epilogue around a semantic operation with an opaque
  library implementation;
- an explicit split/materialize alternative at eligible edges.

Each initial candidate is nonempty, connected, and convex in the operation DAG:
a path between included operations may not leave and re-enter the region.
Explicit duplication creates separately accounted occurrences; it never
silently waives convexity. Values consumed outside the region and graph results
are retained boundary outputs, so one fused region may correctly produce
several ordered values.

Producer duplication, region boundaries, and materialization belong to this
physical exploration phase rather than logical rewrite identity. A hypergraph
may index overlapping candidates internally, but membership alone is not a
complete region identity.

### Physical implementation

Implementation rules produce schedules such as:

- scalar or vectorized flat loops;
- rank-aware strided loops;
- direct or tiled rearrangement;
- serial, subgroup, threadgroup, or multi-pass reduction;
- direct or GEMM-backed contraction.

The bounded P0 frontier admits only checked `ScheduledKernel` proposals and
rejects opaque-call proposals explicitly. Its provider/body representation
must retain an additive sum-type seam so the later reviewed
[`implement-opaque-physical-call-providers`](../../tickets/implement-opaque-physical-call-providers.md)
ticket can add opaque implementations without weakening scheduled-kernel
verification.

Each implementation candidate advertises a machine-checkable numerical
guarantee, realization/provider identity, and scoped evidence. It is admitted
only when that guarantee refines every effective operation contract. A stronger
implementation may satisfy a weaker requested result set, but it does not
rewrite semantic identity.

### Enforcers

An enforcer supplies a missing required property at a cost:

- contiguous materialization;
- layout conversion;
- dtype cast;

Scalar alignment-safe execution and bounds masking are schedule alternatives or
proof obligations, not enforcers. A partial buffer plus second pass is a
multi-kernel reduction implementation.

### Cleanup

After program selection, local passes perform index-expression CSE,
loop-invariant motion, strength reduction, constant folding, bounds-check
elimination, and dead-code elimination. Schedule-affecting normalization
finishes before `ScheduledRegion` identity is formed. Later structured-kernel
cleanup is independently canonicalized and committed through codegen/artifact
identity; it must not silently mutate the already-hashed schedule.

## Boundary requirements and guarantees

A downstream region implementation requests boundary properties and each
producer implementation advertises what it guarantees. Initial boundary
contracts include:

- storage layout class and contiguous axes;
- alignment and vectorizable width;
- materialized buffer, alias/view, or opaque runtime value;
- device and address space.

Logical shape, accumulation semantics, and numerical policy are semantic traits
or optimization-context constraints, not properties supplied by a schedule.
Target capabilities, runtime guards, resource use, schedule invariants, and
cost estimates are also distinct concepts rather than entries in one universal
property bag. Iteration order and register residency are region-internal unless
they affect a boundary value.

For example, a vectorized reduction may require a unit-stride reduction axis,
16-byte alignment, and an extent divisible by four. The optimizer compares a
contiguous-materialization enforcer followed by that reduction against a
generic strided reduction.

The boundary-contract system defines canonical keys, satisfaction and
subsumption (for
example, 16-byte alignment satisfies a 4-byte requirement), child requirement
derivation, and dominance. Enforcer insertion is cycle-checked. Interesting
boundary properties such as useful unit-stride axes are retained on a bounded
Pareto frontier even when they are not locally cheapest.

One implementation dominates another only within the same semantic and
constraint region when its applicability covers the other's, its target and
boundary requirements are no stronger, its guarantees are at least as strong,
its hard resources are no worse where relevant, and its symbolic cost is no
worse throughout the compared constraint cell and strictly better somewhere.
Otherwise both remain or the constraint space is partitioned. Cost alone may
not prune the only implementation valid for a runtime region.

Target-requirement implication and evaluation phase participate in dominance.
A candidate needing a stronger or later runtime predicate does not dominate a
generic candidate merely because its estimated cost is lower. Scalar/generic
coverage is retained whenever specialized feasibility is deferred or narrower.

Numerical conformance is checked before this dominance relation. Accuracy is a
hard semantic dimension, not a Pareto cost; incomparable or unknown evidence
cannot be made legal by a lower estimated runtime.

## Possible memo contract

If a bounded memo is adopted, its conceptual key is:

```text
semantic group key = canonical semantic expression
optimization key = (group, boundary requirements, target profile,
                    numerical policy, constraint region)
candidate = region implementation + child boundary requirements
            + boundary guarantees
```

It would store a bounded Pareto set, track shared DAG cost without charging a
materialized producer once per parent, detect cycles, and retain structured
rule/candidate provenance. Search-budget exhaustion returns the best complete
plan found under deterministic fallback heuristics.

Before global DAG planning is implemented, the same interfaces may be backed by
a trivial region builder for a narrow semantic graph; this staged shortcut is
explicit rather than a second optimizer architecture.

## Symbolic parameters and routing

The optimizer consumes a constraint environment describing exact/ranged
extents, divisibility, equalities, and optionally common profiled values. Costs
may be symbolic or piecewise over this environment. The selected result can be
a portfolio of AOT variants plus a deterministic routing decision tree or
crossover formula. Guards establish validity; routing chooses profitability
when several variants are valid. Routing policy participates in `EXPLAIN` and
artifact identity.

## Rule interface

Semantic rules conceptually provide:

```text
match(expression) -> bindings
check(bindings, semantic_context) -> proof or rejection
apply(bindings) -> equivalent expression(s)
```

Implementation rules conceptually provide:

```text
implement(group, boundary_requirements) -> candidate {
    implementation,
    child_requirements,
    boundary_guarantees,
    legality_constraints,
    estimated_resources
}
```

Every rule needs a stable name, declared numerical preconditions, positive and
negative tests, deterministic search behavior, and explain-trace output.

## Explainability

An `EXPLAIN` report should show:

```text
logical input
normalization rules fired
equivalent alternatives retained
fusion regions considered
boundary requirements/guarantees
enforcers inserted
schedules considered and rejected
per-operation reference and effective accuracy envelope
candidate numerical guarantee, realization, and evidence class
selected cost and assumptions
runtime guards and fallback
```

Structured rejection reasons are important: “threadgroup reduction rejected:
shared memory exceeds target limit” is actionable; a later MSL compiler error
is not. Numerical reasons are equally concrete, such as “claimed 3 ULP exceeds
required 1 ULP,” “domain uncovered,” or “toolchain evidence unknown,” and are
reported separately from cost rejection.

Every rejection records its stage, stable reason code, rule/provider identity,
affected operation/value or candidate, failed predicate/evidence, and whether
the result is a hard rejection, safe deferral, budget stop, dominance pruning,
or cost disadvantage. Explain output never collapses these into “not fused.”

### Explain authority

Under ADR 0073 the typed explain vocabulary — records, subjects, stages,
dispositions, reason and rule keys, evidence classes, and retention bounds — is a
module of `tiler-compiler`, not a separate `tiler-explain` crate. The compiler
owns record construction, canonical identity, causal integrity, and the versioned
renderer. Emission is compiler-owned: sibling compiler modules obtain record
handles from a writer, and no provider-facing emission trait is published. Module
visibility is a public-facade question rather than a packaging one; the module is
private while the compiler boundary is private.

If a second crate must ever read canonical traces, the record, subject, and
disposition vocabulary moves into `tiler-ir` following the `AbiExpr` co-location
precedent of ADRs 0068 and 0070, with emission staying compiler-owned. A new
crate is not the expansion path. Until that trigger fires, a component that
cannot depend on `tiler-compiler` has no explain contract; it is an explicit
unsupported case rather than a licence to copy the vocabulary.

Canonical trace content is data and the renderer is presentation. Nothing in this
contract requires an explain trace to be serialized into an artifact envelope,
and the artifact contract does not carry one.

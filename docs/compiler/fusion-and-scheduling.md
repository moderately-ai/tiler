---
schema: "tiler-doc/v1"
id: "tiler.contract.fusion-and-scheduling"
kind: "contract"
title: "Fusion and scheduling"
topics: ["fusion", "scheduling", "optimizer"]
contract_status: "accepted"
implementation_status: "partial"
evidence: ["tiler.research.region-search.exhaustive-region-oracle", "tiler.research.scheduling.scheduled-region-model", "tiler.research.target-profiles.physical-feasibility-model", "tiler.research.program-planning.kernel-program-buffer-plan"]
---

# Fusion and scheduling

**Status:** accepted research contract; bounded prototype implementation

The private strict-`f32` serial-`Sum` slice now enumerates five singleton
candidates, the four-operation pointwise candidate, and the complete fused
candidate from canonical semantic occurrence roles. It checks membership,
boundaries, connectivity, convexity, numerical permissions, deterministic
candidate budget, schedule feasibility, and structured-kernel refinement. This
is evidence for the architecture, not a general region enumerator or public
fusion API.

## Ownership boundary

This document owns fusion-region formation and schedule candidate generation,
legality queries, and split-plan retention. The IR contract owns the normalized
schedule fields and verifier; target backends own realization of accepted
requirements on concrete devices.

## Fusion is a plan choice

Fusion removes intermediate storage and dispatches by evaluating producer work
inside a consumer kernel. It can also increase live ranges, indexing work,
source size, synchronization, and register pressure. Therefore:

```text
can fuse != should fuse
```

The optimizer must retain a split implementation wherever a fused
implementation is considered.

Logical operation boundaries do not imply materialization. Named `Multiply`,
`Add`, `Gelu`, `Broadcast`, and `Reduce` nodes remain visible in the semantic
graph until region exploration chooses which operations to compose into an
iteration/scalar expression.

## Region representation

A candidate is more than a set of operation IDs:

```text
RegionCandidate {
    member_operations,
    boundary_inputs,
    retained_outputs,
    allowed_duplication,
    semantic_region_id,
    numerical_contract_id,
}
```

The initial region is also connected and convex. If `a -> b -> d` and
`a -> c -> d`, the set `{a, b, d}` is illegal because another path between its
members leaves through `c` and re-enters at `d`. Contracting such a set would
hide required interleaving. Duplication creates distinct, explicitly costed
occurrences; it is not an exception to convexity.

Overlapping candidates may be indexed as hyperedges during search. That
hypergraph is an optimizer data structure, not the semantic graph or selected
physical program. Two candidates with the same member operations can differ in
retained outputs or allowed producer duplication and therefore have different
feasible implementations. Boundary contracts and actual materializations
belong to region implementations and complete kernel programs.

For each candidate and target profile, iteration/access lowering plus local
scheduling returns a bounded `ImplementationFrontier`. Each implementation
contains a sum-typed body (`ScheduledKernel`, `KernelSubprogram`, `OpaqueCall`,
or `View`), boundary requirements/guarantees, applicability predicates, target
requirements, exact/proven resource requirements, resource estimates, and a
cost estimate. Program selection chooses a compatible covering set only after
these frontiers are available.

For a shared producer `p` with consumers `left` and `right`, legal alternatives
include one materialized `p`, a multi-output region `{p,left,right}`, or—only
with explicit duplication capability—two occurrences in `{p,left}` and
`{p,right}`. The first implementation keeps duplication disabled while the
exhaustive tiny-DAG oracle retains it as a completeness witness.

## Legality

A proposed fusion region is legal only when all of the following hold.

### Iteration and indexing

- Every output coordinate maps to valid input coordinates.
- Reindex composition is representable and in bounds.
- Broadcast reads may alias but output writes do not overlap.
- Rank-changing view operations remain metadata unless a physical reorder is
  deliberately chosen.
- Zero-sized domains cause no memory access or illegal dispatch.

### Dependencies and effects

- The region is acyclic.
- Internal values have a defined ownership and lifetime.
- Reduction dependencies are properly nested.
- No cross-threadgroup dependency exists without an explicitly supported
  atomic or multi-pass protocol.
- Barriers are reached uniformly by all participating threads.
- Alias and in-place behavior are explicit; the initial design is out-of-place.

### Target capabilities

- Required dtypes, operations, execution scopes, memory spaces, barriers, and
  collectives are supported by typed capability facts.
- Index arithmetic cannot overflow under guards.
- Selected execution-scope/group dimensions, local memory, bindings, and
  generated resources fit target limits.
- Vector access satisfies alignment and tail requirements.
- Any unresolved hard fact is deferred to a named safe preflight phase with an
  equivalent packaged alternative; estimates never establish legality.

### Numerical semantics

- Reduction identity and accumulator type are defined.
- Operation and reduction order satisfy the selected policy.
- A concrete reduction topology is a physical-plan decision proven to satisfy
  the semantic reduction's allowed evaluation-order or result class.
- Reduction scheduling proves reassociation and operand-permutation legality
  independently; a permission or algebraic capability for one is not evidence
  for the other.
- NaN, signed-zero, empty-domain, cast, and overflow semantics are preserved.
- Every fused scalar realization and opaque intrinsic refines each
  transcendental operation's effective reference, domain, accuracy, special-
  value, and subnormal contract.

Legality failure produces a structured split or fallback reason. An accuracy
failure is hard infeasibility, never a cost penalty.

## Profitability

Benefits of fusion include:

- avoided launches and intermediate allocation;
- eliminated global-memory writes and reads;
- register-local producer/consumer reuse;
- joint index simplification and layout planning.

Costs include:

- duplicated work at fan-out;
- larger live ranges and reduced occupancy;
- additional local memory and barriers;
- loss of parallelism around reductions;
- worse memory coalescing;
- index div/mod overhead;
- divergence and masked lanes;
- loss of tuned library kernels;
- integration-supplied compilation, artifact, and delivery-size costs; for the
  proposed Rust/Metal path these include cold macro expansion and embedded
  metallib/binary growth.

Fan-out greater than one is evidence for materialization, not a categorical
boundary. Cheap reindex or scalar work may be worth duplicating; a large
reduction usually is not.

## Pointwise and reindex schedules

Candidate schedules include:

- one thread per logical output;
- grid-stride loops;
- collapsed contiguous iteration;
- rank-aware dynamic-stride indexing;
- fixed vector widths such as 1, 2, 4, or 8 as backend/profile-specific search
  candidates, plus scalable vector shapes where the target model admits them;
- alternate axis orders for coalescing;
- masked vector or scalar tails.

Vector legality is queried for the complete operation, dtype, fixed/scalable
shape, mask/tail, address space, access width, and alignment contract. A
preferred width is a cost fact, not proof that the operation is legal.

A logical transpose need not be materialized. It becomes a different access
map in the fused consumer. Materialization remains a candidate when it improves
several downstream consumers enough to repay its write/read cost.

## Schedule representation

A `ScheduledRegion` is one canonical `IndexRegion` plus one normalized
`KernelSchedule`. The schedule owns execution axes, work assignment, output
ownership, loop/vector/tail organization, staging lifetimes, reduction
topology/result visibility, synchronization phases, launch formulas, and
specialization bindings. The index region continues to own scalar meaning and
logical access maps. Derived hard resource requirements are checked from the
schedule; target facts and cost estimates remain separate inputs.

A selected schedule explicitly represents loop/tile hierarchy, coordinate
mapping to grid/threadgroup/SIMD/lane/vector axes, vectorization, memory
placement, staging, synchronization, reduction topology, tails, and launch
formulas. It is canonical physical IR rather than a bag of node annotations.

Scheduling transformations and their rejection reasons are recorded in a
separate trace for explanation and replay. The normalized result is the
identity-bearing executable intent and is independently verified; successful
application of a transform sequence is not a legality proof.

## Reduction implementations and schedules

Reduction remains semantic until region implementation and scheduling select an
explicit strategy:

1. **Serial per output:** one thread loops over the reduction domain.
2. **Multiple outputs per thread:** amortizes indexing for small reductions.
3. **SIMD-group reduction:** lanes cooperatively reduce one or more outputs.
4. **Threadgroup reduction:** several SIMD groups combine through local memory.
5. **Multi-pass reduction:** a `KernelSubprogram` materializes partial outputs
   for a later scheduled kernel.

Each implementation declares valid extents, lane-result visibility, tail masking,
barrier requirements, accumulator type, and target capabilities. There is no
underspecified portable “block reduce” operation in final scheduled IR.

A multi-pass reduction is a `KernelSubprogram`: an initial scheduled kernel
writes typed partials to declared scratch, a typed dependency and
`StorageHandoff` makes those bits visible, and a later scheduled kernel produces
the result. Scratch preserves accumulator bits unless the semantic contract
explicitly admits a conversion. Canonical stream/list order alone is not a
durable storage-handoff proof.

## Rearrangement schedules

Alternatives include:

- direct loads/stores from a composed logical tensor-access map;
- collapsed contiguous copy;
- tiled threadgroup-memory transpose;
- materialize once for multiple consumers.

A no-kernel alias/view result is another global physical alternative, not a
kernel schedule, and is deferred from the initial Candle custom-op path.

## Future contraction schedules

Einsum adds global contraction-order choices and local implementations:

- direct scalar or tiled contraction;
- GEMM canonicalization;
- optimized library matmul;
- layout conversion enforcers;
- batching and split-reduction strategies;
- fusible pointwise prologues and epilogues.

Contraction planning should follow, not precede, the boundary-contract and cost
infrastructure.

## Search control

Schedule exploration must be bounded using target limits, ranked candidate
sizes, dominance pruning, and explicit compile-time budgets. Deterministic
heuristics are preferred initially. Offline empirical calibration may improve
candidate ranking later without introducing runtime JIT compilation.

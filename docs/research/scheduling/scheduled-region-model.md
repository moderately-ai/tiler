# First-class scheduled-region model

**Status:** research basis for ADR 0007
**Ticket:** `scheduled-region-model`

## Conclusion

`ScheduledRegion = IndexRegion + normalized KernelSchedule` is the right
physical boundary before structured kernel lowering. The `IndexRegion` says
which logical coordinates are computed, which scalar expressions are evaluated,
and how buffers are accessed. The schedule says which execution instances own
those coordinates, in what loop/vector/reduction organization, with which
explicit staging, synchronization, tails, and launch formulas.

The schedule is neither a tensor-graph annotation nor a history of scheduling
commands. It is a small declarative execution plan whose verifier can establish
intrinsic correctness and derive hard target requirements. A later lowering
expands that plan into typed control flow and memory operations. A separate
trace explains how the scheduler reached or rejected the plan.

The worked examples and the compile-checking spike support the existing
architecture without requiring a hypergraph or one universal target topology.
They also expose one important boundary: a multi-kernel reduction is a
`KernelSubprogram`, not an oversized single-kernel schedule.

## Evidence from primary systems

The systems differ in where their final executable representation lives, but
their common lessons are consistent:

- Halide specifies the algorithm separately from scheduling and shows `split`,
  `fuse`, `reorder`, tiling, vectorization, unrolling, and parallelization as
  changes to the generated loop nest. Its tutorial also demonstrates that tail
  handling may deliberately repeat or clamp work, so a transform name alone is
  not a coverage proof. See the official
  [scheduling tutorial](https://halide-lang.org/tutorials/tutorial_lesson_05_scheduling_1.html).
- TVM defines a schedule as semantics-preserving transformations over TensorIR,
  has distinct concrete and traced schedules, and can enable checks after
  transformations. This supports retaining a trace separately from the
  transformed executable payload. See the official
  [`Schedule` API](https://tvm.apache.org/docs/reference/api/doxygen/classtvm_1_1s__tir_1_1Schedule.html)
  and [TensorIR architecture](https://tvm.apache.org/docs/deep_dive/tensor_ir/index.html).
- MLIR Linalg keeps iteration and access semantics in structured operations and
  uses them for tiling, promotion, fusion, vectorization, and hardware mapping.
  The Transform dialect explicitly distinguishes transform IR from payload IR,
  models transform-side effects and failures, and provides normal-form types.
  This is strong precedent for a trace/control representation not being the
  final scheduled identity. See the
  [Linalg dialect](https://mlir.llvm.org/docs/Dialects/Linalg/),
  [Linalg rationale](https://mlir.llvm.org/docs/Rationale/RationaleLinalgDialect/),
  and [Transform dialect](https://mlir.llvm.org/docs/Dialects/Transform/).
- MLIR's GPU dialect gives launch dimensions, thread/block coordinates,
  address spaces, workgroup-lifetime memory, and barriers explicit IR
  representation. Its rationale for function-level memory attribution is that
  ownership and lifetime should remain structurally visible. See the
  [GPU dialect](https://mlir.llvm.org/docs/Dialects/GPU/).
- CUDA makes block independence a correctness condition: blocks may run in any
  order, while communication and synchronization are naturally block-scoped.
  It separately identifies grid, block, warp, shared-memory, and lane behavior.
  A schedule therefore cannot imply an ordinary grid-wide barrier or hide a
  cross-block dependency. See the official
  [CUDA programming model](https://docs.nvidia.com/cuda/cuda-programming-guide/01-introduction/programming-model.html).
- Metal distinguishes grid size, threadgroup size, uniform versus nonuniform
  edge groups, and pipeline-specific `threadExecutionWidth` and
  `maxTotalThreadsPerThreadgroup`. The latter can differ for two pipelines on
  one device. Launch formulas belong to the schedule, while final legality may
  be deferred to prepared-pipeline preflight. See Apple's
  [threadgroup and grid guidance](https://developer.apple.com/documentation/metal/calculating-threadgroup-and-grid-sizes)
  and [`maxThreadsPerThreadgroup`](https://developer.apple.com/documentation/metal/mtldevice/maxthreadsperthreadgroup).

These are precedents, not claims that Tiler should copy any one IR. In
particular, Tiler needs a versioned target-neutral schema, exact numerical
contracts, and complete artifact identity that schedule APIs used only within a
compiler do not necessarily provide.

## Normalized model

The proposed conceptual schema is:

```text
ScheduledRegion {
  schema_version,
  index_region,
  schedule: KernelSchedule,
}

KernelSchedule {
  execution_axes,
  work_assignment,
  loop_and_vector_plan,
  output_ownership,
  local_staging,
  reduction_plans,
  synchronization,
  phase_order,
  launch_plan,
  specialization_bindings,
}
```

All collections use canonical order and stable newtyped IDs. All expressions
use canonical typed arenas. There are no implicit defaults once identity is
formed.

### Execution axes and work assignment

An execution axis records:

```text
ExecutionAxis {
  id,
  extent,
  binding: Serial
         | Grid(axis)
         | Workgroup(axis)
         | Subgroup
         | Lane
         | FixedVectorLane
         | ScalableVectorLane,
  loop_order_and_parent,
  unroll_or_pipeline_policy,
  tail_policy,
}
```

`work_assignment` is the total mapping from execution-axis coordinates to each
logical spatial and reduction coordinate in the `IndexRegion`, plus an active
predicate. It is the normalized result of split, fuse, reorder, bind, and
grid-stride transforms. The schedule does not copy the `IndexRegion` access
maps or scalar program.

Subgroup lanes and per-thread vector lanes are different bindings. GPU
threadgroups and CPU worker/vector scopes are likewise different governed
scope kinds. A backend profile declares which scope topology and coordinate
roots it supports; the common schema does not pretend they are synonyms.

### Output ownership and tails

Each output write records the owning execution instance and whether redundant
evaluation is allowed. Tail policy is explicit:

- `Exact` derives a divisibility/applicability obligation;
- `Predicated` carries an in-domain active predicate;
- `IdentityPadded` is legal only with a proved reduction identity and padding
  safety;
- a deliberately duplicated pure computation states that duplication and
  still assigns each externally visible output write unambiguously.

This is necessary because split histories can lead to clamping, overlap,
masking, or exact division. The verifier checks the resulting ownership and
coverage rather than trusting the transform name.

### Vectors

A vector plan names fixed or scalable shape, the vector-lane axis, operation
and dtype context, mask/tail behavior, address space, access width, and required
alignment. It does not equate vector lanes with subgroup lanes. Legal width and
preferred width remain distinct: the former produces hard requirements, the
latter belongs to cost.

### Local staging and synchronization

Local staging records the source access, local allocation shape/layout/address
space, cooperative copy assignment, active predicate, lifetime, and producer/
consumer phases. Synchronization records participants, execution scope, fenced
memory spaces/order, convergence, and its position between phases.

The initial phase model can be a canonical total sequence of abstract
`StageLoad`, `Compute`, `ReduceStep`, `Barrier`, and `Store` phases. These are
not imperative statements: structured-kernel lowering still chooses lexical
loops and emits actual loads, stores, conditionals, and barriers. A later
software-pipeline extension may introduce explicit dependence tokens or a
partial order without changing the existing phase meanings.

Transparent caches are absent from staging. They are cost-model levels, not
addressable storage. A private temporary is schedule intent; whether it becomes
a register or spills is a lowering/target fact unless the target exposes a
stronger contract.

### Reductions

Each reduction domain has exactly one topology:

```text
Serial(axis, order)
SubgroupTree(contributors, combine_steps, result_lane, masked_identity)
WorkgroupTree(contributors, local_partials, combine_steps,
              barriers, result_owner, masked_identity)
```

The plan names contributor coverage, per-contributor serial order, combine
tree/order, accumulator dtype, identity/tail behavior, result visibility, and
owner. Reassociation and operand permutation are checked independently. An
opaque `block_reduce` is insufficient because it does not determine numerical
order, visibility, barriers, or resource use.

Several launches with a partial-result buffer are a `KernelSubprogram` with
several individually scheduled regions. Cross-kernel allocations and
dependencies do not move into `KernelSchedule`.

### Launch and specialization

The launch plan owns canonical host-evaluable formulas for grid/group extents,
uniform or nonuniform dispatch, dynamic local-memory sizes, zero-work behavior,
and referenced specialization values. The verifier checks that launch plus
work assignment covers the declared domain. Artifact launch expressions are
checked derivations, not a second authority.

## Verification boundary

Verification is deliberately layered.

### 1. Structural and canonical verification

- IDs and references are valid, unique, and canonically ordered.
- Every automatic choice and default has been resolved.
- Expressions are typed, canonical, bounded, and use admitted roots.
- Every spatial and reduction domain has one coordinate reconstruction.
- Phase, local-allocation, result, and specialization references are valid.

### 2. Intrinsic schedule verification

- Active work covers every required logical coordinate.
- Output writes have unique ownership; redundant work is explicitly pure and
  permitted.
- Composed index/access maps remain in range under the active predicates.
- Local values are initialized before reads and remain live through consumers.
- Races are absent or use an explicit valid atomic/reduction protocol.
- Barriers and collectives have the required participants, fence, and
  convergence.
- Vector lanes and tails cover the scalar domain without forbidden overlap.
- Reduction contributors appear exactly once unless proved identity padding is
  used; the combine tree satisfies numerical permissions.
- Zero extents and empty reductions follow the semantic contracts.

Failure here is a hard schedule rejection independent of target performance.

### 3. Derived requirements and target feasibility

The verified schedule deterministically derives exact/proven quantities such as
threads per group, static/dynamic local bytes, bindings, barrier/collective
requirements, vector legality predicates, and subgroup assumptions. These feed
the phased feasibility model from ADR 0043:

- a disproved hard predicate is `Rejected`;
- a permitted preflight query is `Deferred`;
- a missing proof/query path is `Unknown` and cannot be executable;
- all established predicates are `Proven`.

Resource requirements should be derived from the identity-bearing schedule and
checked against any stored manifest copy. They should not be an independently
editable schedule field.

### 4. Costing

Coalescing, bank conflicts, redundant indexing, expected divergence, register
pressure, occupancy above feasibility, source size, and throughput are cost
evidence. A schedule with poor coalescing can still be legal. An estimate may
rank or prune under an explicit search budget, but it cannot prove a barrier,
vector access, launch, or memory allocation legal.

## Worked schedules

The notation below uses `g*` for grid/workgroup coordinates, `t*` for local
threads, `q` for serial loops, and `v` for vector lanes.

### 1. Fused elementwise

```text
z[i] = max(x[i] + y[i], 0),  0 <= i < N
```

Scalar schedule:

```text
workgroup = [256, 1, 1]
groups.x = ceil_div(N, 256)
i = group.x * 256 + thread.x
active = i < N
owner(z[i]) = this thread
```

Vector-four alternative:

```text
workgroup = [64, 1, 1]
groups.x = ceil_div(N, 256)
i = (group.x * 64 + thread.x) * 4 + vector_lane
vector shape = Fixed(4), masked tail
```

The vector schedule derives operation/dtype/mask/address-space legality and
alignment requirements. A failed alignment guard is an applicability miss; an
unsupported vector operation is hard target infeasibility; a low estimated
benefit is only a cost rejection. Both schedules implement the same fused
scalar `IndexRegion`.

### 2. Broadcast plus elementwise

```text
z[b,m,n] = x[b,m,n] + bias[n]
```

One schedule collapses `B*M*N`, binds the innermost contiguous `n` progression
to adjacent threads, and reconstructs:

```text
linear = group.x * 256 + thread.x
b = linear / (M*N)
m = (linear / N) % M
n = linear % N
active = linear < B*M*N
```

The `bias[n]` broadcast is already an `IndexRegion` access map; it is not a
schedule annotation on the broadcast node. Mapping `m` rather than `n` to
adjacent lanes may remain intrinsically legal but cost worse because `x` is
strided. A local bias tile is another schedule only when it explicitly records
the cooperative copy, local allocation, lifetime, and barrier.

### 3. Tiled transpose-like materialization

```text
y[m,n] = x[n,m]
```

A direct gather schedule is legal but has strided input reads or output writes.
The cooperative alternative uses a 32 by 32 tile padded to 33 columns:

```text
workgroup = [32, 8, 1]
groups = [ceil_div(M,32), ceil_div(N,32), 1]
q in 0..4 serially

load:
  m_in = group.x*32 + thread.x
  n_in = group.y*32 + thread.y + q*8
  if in bounds: tile[n_local][m_local] = x[n_in,m_in]

barrier(all workgroup threads, fence tile)

store:
  m_out = group.x*32 + thread.y + q*8
  n_out = group.y*32 + thread.x
  if in bounds: y[m_out,n_out] = tile[thread.x][thread.y+q*8]
```

The local allocation is `32*33*sizeof(f32)` bytes with an explicit lifetime.
The padding is a bank-conflict cost choice, not a correctness requirement. A
barrier placed inside the load bounds predicate is rejected for nonconvergence.
Excess local bytes or threads are target-feasibility failures. The direct
schedule remains an alternative rather than being called invalid.

### 4. Row reduction

```text
y[m] = reduce_add(n, x[m,n])
```

The baseline assigns one thread per row and iterates `n` serially in semantic
order. It is the schedule available when the contract forbids reordering.

A workgroup-tree alternative assigns one row per group:

```text
workgroup = [256,1,1]
n = thread.x + q*256
each thread accumulates its in-range contributors
partials[thread.x] = accumulator or proved identity
barrier
combine strides = [128,64,32,16,8,4,2,1], barrier between shared steps
thread.x == 0 stores y[m]
```

This schedule is rejected unless the numerical contract permits both the
resulting reassociation and operand permutation. Identity padding must be
proved for the effective operation contract. Local storage and the 256-thread
shape are hard target requirements; expected occupancy is cost. A multi-pass
alternative belongs to a `KernelSubprogram` because it has multiple launches
and a cross-kernel partial buffer.

## Why schedules are not node properties

The examples rule out node annotations as the authoritative representation:

- scalar and vector elementwise schedules implement the same operations;
- the broadcast access map participates in one joint flattened work assignment
  across multiple fused nodes;
- transpose staging is a cooperative relation among loads, a local tile,
  barriers, and stores, not a property of the logical transpose operation;
- reduction topology assigns contributors and result visibility across many
  execution instances and may turn one region into a multi-kernel subprogram;
- the same node can occur in overlapping region candidates with different
  retained outputs, duplication, boundaries, and downstream layout value.

Hardware dimensions are coordinates and resource constraints of an
implementation, not additional dependency edges. Multivariate feasibility
predicates therefore do not make the tensor graph or schedule a hypergraph.

## Rejection and explanation contract

Every attempted transform and candidate assessment records a stable phase and
rule, for example:

```text
Rejected {
  phase: IntrinsicSchedule | TargetFeasibility | Numerical | SearchBudget,
  rule_id,
  subject_path,
  derived_facts,
  required_facts,
  source_candidate,
}
```

Examples include:

- `schedule.barrier.convergent` — hard intrinsic rejection;
- `schedule.reduction.numerical_order` — hard numerical rejection;
- `target.threads_per_workgroup` — proven/deferred target result;
- `applicability.input_alignment` — guarded variant miss;
- `cost.memory_coalescing` — legal candidate ranked lower;
- `budget.max_candidates` — legal candidate was not explored/retained.

The trace contains transformation names, parameters, preconditions, and these
outcomes. It is useful for `EXPLAIN` and replay, but only the independently
verified normalized schedule is executable identity.

## Spike

[`scheduled_region_model.rs`](../../../spikes/scheduling/scheduled_region_model.rs)
is a dependency-free Rust model that compile-checks the proposed ownership
boundaries. It constructs the four workloads plus scalar/vector alternatives,
derives resource requirements, separates intrinsic verification from target
assessment and cost, and tests reduction-order, barrier, and target-limit
rejections.

The spike intentionally does not implement a general symbolic solver or prove
arbitrary coordinate-map bijectivity. Those belong to the index/access and
solver work. Its purpose is to test that the records and verifier boundaries can
express the required schedules without target objects or node annotations.

Run it with:

```sh
rustc --edition 2021 --test \
  spikes/scheduling/scheduled_region_model.rs \
  -o /tmp/tiler-schedule-spike
/tmp/tiler-schedule-spike
```

## Resolved choices and deferred refinements

The evidence resolves these decisions:

- normalized scheduled intent, not transform history, is authoritative;
- coordinate mapping, output ownership, tails, vectors, staging, reduction
  topology, synchronization, phases, and launch belong in `KernelSchedule`;
- scalar/access semantics remain in `IndexRegion`;
- exact/proven resource requirements are derived and target feasibility is a
  separate assessment;
- cost estimates cannot establish legality;
- cross-kernel materialization belongs to `KernelSubprogram`/`KernelProgram`.

Two representation details can remain experimental without changing those
contracts:

1. The initial abstract phase order can be a total sequence. Asynchronous copy
   pipelines may later justify explicit dependence tokens or a partial order.
2. The schedule-coordinate expression arena can reuse the index-expression
   implementation, but it needs a distinct admitted-root type so hardware and
   serial coordinates cannot be confused with logical iteration symbols.

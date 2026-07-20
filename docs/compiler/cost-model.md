---
schema: "tiler-doc/v1"
id: "tiler.contract.cost-model"
kind: "contract"
title: "Cost model"
topics: ["cost-model", "optimizer", "measurement"]
contract_status: "accepted"
implementation_status: "spike-only"
governed_by: ["ADR-0043"]
evidence: ["tiler.research.cost-model.bootstrap-cost-model"]
---

# Cost model

**Status:** accepted bootstrap contract; target calibration pending

The cost model ranks complete legal plans. Its first implementation should be
simple and inspectable rather than claiming hardware accuracy it does not have.

## Objectives

Execution time is the primary objective, subject to correctness, target
resources, compile-time search budget, and artifact-size budget. Compilation
cost and runtime cost should remain separate dimensions so the optimizer can
retain a Pareto frontier such as “lowest estimated runtime below this artifact
budget.” Peak temporary device memory is also a hard budget or Pareto dimension.

An initial analytical form is:

```text
estimated time =
    launch count * launch latency
  + max(bytes transferred / effective bandwidth,
        weighted operations / effective throughput)
  + synchronization penalty
  + occupancy penalty
```

Recomputed operations and bytes are included in work and memory counts rather
than added again as a generic penalty. Occupancy modifies effective rates or is
modeled as a discrete tier; it is not double-counted in both places.

Every estimate carries a point value, lower/upper interval, calibration-profile
identity, feature-domain coverage, and unsupported interaction list. Plans
whose intervals overlap are cost-indistinguishable; deterministic identity may
break the packaging tie, but explain output must not claim a measured winner.
An infeasible or intrinsically invalid candidate has no cost estimate rather
than a large penalty.

## Work features

- scalar arithmetic count by dtype and operation class;
- transcendental and special-function count by function, dtype, and legal
  realization class such as native bounded, fixup, emulated correctly rounded,
  or opaque library call;
- integer index arithmetic, especially division and modulo;
- reduction operations and serial loop length;
- duplicated producer work caused by fan-out;
- loop-invariant work that can be hoisted.

## Memory features

- bytes read and written at each modeled memory level;
- intermediate allocation size;
- expected transaction efficiency and coalescing;
- unit-stride and alignment properties;
- vectorized width;
- broadcast or repeated-read reuse;
- threadgroup-memory traffic and reuse;
- materialized layout-conversion traffic.
- peak live intermediate bytes and allocation lifetime.

## Parallel-execution features

- grid and threadgroup dimensions;
- useful versus masked threads;
- serial work per thread;
- divergence and tail policy;
- barrier and collective steps;
- local-memory footprint;
- live-value/register-pressure proxy;
- compiler/preflight-proven resource feasibility where a backend exposes an
  authoritative rule, including nonzero residency when required for launch;
- estimated occupancy tier, register pressure, and spill risk;
- number of dispatches and multi-pass dependencies.

## Compilation and deployment features

- generated IR, target payload, and delivery-representation size;
- number of specialized variants and target entry points;
- compiler work and expected artifact-cache reuse under the selected delivery
  policy;
- packaged artifact and final-binary contribution;
- expected execution frequency and amortization;

An integration supplies these deployment features. For the proposed Rust/Metal
path they include MSL and expanded-token size, macro-local entry points, cold
proc-macro/AOT work, compiler-cache hit rate, and embedded metallib size. Actual
cache state must not make canonical program selection nondeterministic; any
amortization assumptions are explicit planner inputs and provenance.

These features should usually constrain or rank a Pareto set, rather than being
converted blindly into runtime nanoseconds.

## Non-additive behavior

GPU resource costs are discontinuous. Crossing a register, local-memory, or
threadgroup-size threshold can reduce occupancy or invalidate a plan. The cost
model therefore combines estimates with hard feasibility constraints.

An analytical register or occupancy estimate cannot invalidate a candidate.
Only an authoritative compiled/prepared-kernel fact and target rule may prove
zero feasible residency or another hard launch failure. Occupancy above zero is
a performance variable and higher occupancy is not inherently faster.

CPU profiles add analogous costs for vector legalization/splitting, register
spill pressure, cache working set, memory-level bandwidth, thread-pool width,
oversubscription, and task/barrier overhead. Cache capacity remains a cost fact
unless the schedule explicitly manages that memory space.

Fusion is evaluated as a complete alternative:

```text
fused producer + consumer
```

versus:

```text
producer launch + intermediate write
+ consumer launch + intermediate read
```

For shared work, compare recomputation in each consumer against one compute,
one materialization, and all later reads. Use count alone is insufficient.
Global graph costing accounts for shared subplans once, peak live memory, and
command dependencies; a future model may distinguish critical-path time from
work that can overlap.

## Symbolic and piecewise costs

Runtime extents often decide the winner. Candidate cost can therefore be an
expression over the constraint environment:

```text
serial_cost(outputs, reduction_extent)
threadgroup_cost(outputs, reduction_extent) when reduction_extent >= threshold
```

The optimizer partitions parameter regions or emits a bounded routing policy.
A compatibility guard alone does not imply that a variant is the best choice.

## Calibration

Milestone 3 requires a bootstrap target-profile schema with conservative device
limits and approximate constants. Later calibration refines it with
device-family microbenchmark data:

- warm launch latency;
- effective bandwidth by access pattern and vector width;
- arithmetic throughput by dtype and operation class;
- subgroup and threadgroup reduction regimes;
- synchronization cost;
- occupancy cliffs from local-memory and live-value pressure.

Calibration is offline. Runtime JIT or online autotuning is not required.
Observed versus predicted performance should be retained in benchmark reports
so coefficients can evolve without obscuring plan changes.

The bootstrap suite isolates dispatch, contiguous/strided traffic, arithmetic
classes, index div/rem and wide arithmetic, barriers, threadgroup/live-value
pressure, allocation, compilation/artifact size, and fused-versus-materialized
pairs. Each measurement records target profile, live device, OS/driver,
toolchain, thermal/power state, harness version, sample count, median/tail, and
residual error. The detailed controlled matrix is in the
[bootstrap cost-model research](../research/cost-model/bootstrap-cost-model.md).

For every tiny graph, all oracle-legal plans are costed and measured. Reports
include top-1 accuracy, rank correlation, absolute/relative error, and regret
of the selected plan versus the measured best under the same legal
implementation set. This distinguishes search loss, model error, and illegal
enumeration.

## Explain output

Every selected plan should report the dominant estimated terms and rejected
alternatives. For example:

```text
selected: fused threadgroup reduction
  launches: 1
  global bytes: input + output only
  threadgroup bytes: 1024
  estimated occupancy tier: 2

rejected: materialized transpose + vector reduction
  reason: 64 MiB additional write/read

rejected: float4 fused reduction
  reason: alignment property not provided
```

Cost-model version and target profile participate in program-selection and artifact
reproducibility metadata.

Resource estimates have phases: cheap pre-lowering estimates guide search;
schedule and kernel verifiers apply exact known hard limits; target compilation
failure may reject a candidate before artifact packaging. In the proposed Metal
profile this includes source, AIR, and metallib compilation.
At runtime, pipeline preparation may try a retained semantically identical plan
only before allocation or encoding. Such fallback is bounded and recorded rather
than recursively searching inside the backend emitter.

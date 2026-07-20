---
schema: "tiler-doc/v1"
id: "tiler.research.cost-model.bootstrap-cost-model"
kind: "research"
title: "Initial cost model and calibration plan"
topics: ["cost-model", "optimizer", "measurement"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "adopted"
implementation_status: "spike-only"
evidence_classes: ["executable-model"]
informs: ["tiler.contract.optimizer", "tiler.contract.cost-model"]
ticket: "cost-model-bootstrap"
---

# Initial cost model and calibration plan

## Contract

Costing begins only after semantic, intrinsic schedule, and target-feasibility
checks. An infeasible or unknown implementation is excluded from the executable
frontier; it never receives a large synthetic cost. Deferred feasibility is
retained only under ADR 0043's preflight and coverage rules.

The initial model produces a component vector, an estimated latency interval,
and provenance:

```text
CostEstimate {
  global_traffic,
  allocation,
  dispatch,
  scalar_compute,
  index_compute,
  synchronization,
  occupancy_pressure,
  redundant_compute,
  compiler_work,
  artifact_size,
  latency { point, lower, upper },
  calibration_profile,
  unsupported_features,
}
```

Components remain visible in explain output even when a calibrated scalar
latency ranks candidates. Compilation time and artifact size are separate
objectives with configurable budgets; they are not silently converted into GPU
nanoseconds.

## Precedent findings

- XLA documents analytical GPU fusion estimates alongside measured performance
  tables. Its architecture description identifies avoided intermediate HBM
  traffic as a principal fusion benefit.
- Halide's 2019 autoscheduler uses derived program/schedule features, beam
  search, large random-program training corpora, and optional ground-truth
  benchmarking. This demonstrates both the value of measured calibration and
  the danger of pretending a small hand formula generalizes indefinitely.
- Apple documents that occupancy depends on threads and internal resources,
  including register and threadgroup memory, while warning that high occupancy
  alone does not imply good performance. Metal limiter/utilization counters are
  correlation evidence, not semantic truth.
- NVIDIA likewise treats block threads, shared memory, and registers as a
  coupled occupancy problem and recommends measuring effective bandwidth.

The first Tiler model should therefore be transparent and calibratable rather
than learned. Its errors become training evidence for a later model.

## Feature extraction

Features are derived from verified program and scheduled-region data:

| Component | Initial features |
|---|---|
| Global traffic | unique and total bytes read/written, transaction/coalescing class, reuse distance class, materialized intermediate bytes |
| Allocation | allocation count, requested bytes, storage mode, zero/fill requirement |
| Dispatch | kernel count, grid size, empty/small-grid class, preflight/pipeline changes |
| Scalar compute | operation counts by target throughput class and dtype |
| Redundant compute | duplicated semantic occurrence counts and operation classes |
| Index compute | add/mul/compare/select plus separately counted division, remainder, and wide-integer operations |
| Synchronization | barrier count/scope, participating threads, atomic class, multi-stage dependencies |
| Occupancy pressure | threads per group, static/dynamic threadgroup bytes, estimated live values/register class, subgroup topology |
| Compiler work | structured-KIR nodes, generated source bytes, entry points, variants, specialization count |
| Artifact size | emitted source and measured payload bytes |

Unknown cache hit rates, spills, bank conflicts, compiler register allocation,
and device scheduling are reported as uncertainty drivers rather than guessed
facts.

## Bootstrap latency equation

For each kernel:

```text
memory_time  = traffic_class_bytes / calibrated_effective_bandwidth
compute_time = sum(op_class_count / calibrated_throughput)
body_time    = max(memory_time, compute_time) + index_time + sync_time
kernel_time  = dispatch_intercept(grid_class)
             + body_time * calibrated_pressure_multiplier(resource_bucket)
```

Program latency adds ordered kernel estimates and host allocation/coordination
terms. The `max` is a roofline-style overlap hypothesis, not a claim that all
memory and arithmetic overlap. Calibration residuals widen the interval for
mixed or unsupported feature combinations.

Redundant work contributes to the same traffic/compute counts and is also
reported separately for explanation. Occupancy pressure is a measured bucketed
multiplier; it is not monotonic and never a feasibility test.

## Calibration experiments

Every result is keyed by target-profile identity, live device identity,
OS/driver, compiler/toolchain, power/thermal state, and benchmark harness
version. Use warmup, randomized case order, synchronized completion, repeated
samples, median and tail statistics, and counter collection where available.

1. **Dispatch:** no-op and one-store kernels over empty, tiny, and saturated
   grids; 1/2/4/8 ordered encodes.
2. **Traffic:** contiguous read, write, copy, and read-modify-write over 4 KiB
   through saturation; vary element width, vector width, stride, alignment, and
   reuse.
3. **Arithmetic:** dependency chains and independent lanes for integer, f16,
   f32, conversion, FMA, division, and supported transcendental classes.
4. **Indexing:** equivalent payload with increasingly complex affine/semi-
   affine address formulas; isolate wide arithmetic and div/rem.
5. **Synchronization:** 0/1/2/4 barriers, subgroup versus workgroup scope,
   varying participating threads and staged bytes.
6. **Pressure:** cross product of threadgroup size, explicit threadgroup memory,
   and generated live-value counts; collect compiled resource evidence and
   occupancy/limiter counters.
7. **Allocation:** buffer count and size, reused versus fresh, storage modes,
   with GPU execution excluded.
8. **Compilation/artifact:** KIR/source size, helper count, entry points,
   specialization variants, cold/warm cache, resulting AIR/metallib bytes.
9. **Fusion pairs:** producer/consumer chains comparing materialization versus
   fusion across bytes, compute intensity, fan-out, and index complexity.

Each microbenchmark changes one primary dimension while recording known
secondary changes. Cases where the compiler optimizes away the intended work
must be rejected by reflection, disassembly/report evidence, or output
dependency checks.

## Oracle comparison

For every graph accepted by the exhaustive region oracle:

- enumerate all legal plans under the same physical implementation set;
- calculate model rank and uncertainty for each plan;
- measure every plan on the calibration device;
- report top-1 accuracy, rank correlation, absolute/relative error, and regret
  `(selected_latency / best_latency) - 1`;
- categorize errors by traffic, dispatch, compute, indexing, synchronization,
  pressure, or interaction;
- verify that no illegal plan was made selectable by cost.

Plans whose intervals overlap remain tied. A deterministic identity tie-breaker
may choose one for AOT output, but explain must state that the model could not
distinguish them.

## Initial uncertainty policy

Every calibrated coefficient records sample count, domain, median, dispersion,
and residual error. Prediction intervals combine component residuals
conservatively and widen for extrapolation or unsupported interactions. A model
profile is valid only for its declared target/toolchain domain. Missing
calibration falls back to a coarse target-generic interval, never to invented
precision.

## Deliberate limitations

The first model does not reliably predict caches, spills, bank conflicts,
instruction issue interactions, asynchronous overlap, concurrent queues,
thermal throttling, unified-memory contention, library internals, or multi-
device transfers. It also does not perform runtime autotuning. Persistent
residuals justify new features or a learned model only after the transparent
baseline and oracle comparisons exist.

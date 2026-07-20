---
schema: "tiler-doc/v1"
id: "tiler.prior-art.logical-graphs-and-schedules"
kind: "prior-art"
title: "Logical-graph and schedule-IR precedents"
topics: ["semantic-graph", "scheduling", "prior-art"]
informs: ["tiler.contract.architecture", "tiler.contract.ir", "tiler.contract.optimizer", "tiler.contract.fusion-and-scheduling"]
---

# Logical-graph and schedule-IR precedents

**Status:** supporting research

This note records evidence used to distinguish Tiler's semantic graph, region
search, physical schedules, and executable kernel programs. It does not make
those systems architectural authorities for Tiler.

## Operation/value graph precedents

MLIR's base model separates operations from individually typed SSA values. An
operation has ordered operands, attributes/properties, and zero or more ordered
results. Values have one definition and may have several uses. Extensible
dialects expose generic behavior through traits and interfaces rather than
requiring every pass to know every operation.

Sources:

- [MLIR language reference](https://mlir.llvm.org/docs/LangRef/)
- [MLIR interfaces](https://mlir.llvm.org/docs/Interfaces/)
- [MLIR operation definitions](https://mlir.llvm.org/docs/DefiningDialects/Operations/)

StableHLO is a graph of named tensor operations. Broadcasting is explicit
through `broadcast_in_dim`, operations may have variadic results, and
higher-level `composite` operations carry a namespaced/versioned identity plus
a semantics-preserving decomposition. Irreducible `custom_call` operations are
an opaque alternative rather than automatically fusible semantics.

Source: [StableHLO specification](https://openxla.org/stablehlo/spec).

TVM Relax is a graph-level tensor operator IR. It legalizes operations into
TensorIR functions with buffers, loop blocks, access regions, and scalar
expressions. Relax allows implicit NumPy broadcasting at its API boundary, but
legalized TensorIR must make the selected accesses explicit. Relax also limits
free fusion/reordering to pure dataflow regions, illustrating why a boolean
side-effect annotation is not a sufficient long-term effect system.

Sources:

- [TVM Relax overview](https://tvm.apache.org/docs/deep_dive/relax/index.html)
- [TVM TensorIR](https://tvm.apache.org/docs/deep_dive/tensor_ir/abstraction.html)
- [TVM operator fusion](https://tvm.apache.org/docs/arch/fusion.html)

## Tensor operations and fused scalar expressions

The reviewed systems generally retain named tensor operations before forming a
fused loop/scalar representation:

- StableHLO preserves named tensor operations through graph optimization.
- TVM legalizes Relax calls into TensorIR and fuses legalized functions later.
- `ug` starts with separate unary, binary, layout, and reduction nodes and
  recursively forms a fused shape-aware expression during scheduling.
- Burn records separate tensor operations and builds a fused trace afterward.

MLIR Linalg demonstrates the useful post-region representation:
`linalg.generic` and `linalg.map` combine explicit iterator/access mappings with
a scalar payload region. This supports tiling, fusion, and lowering to loops,
loads, scalar operations, and stores.

Source: [MLIR Linalg](https://mlir.llvm.org/docs/Dialects/Linalg/).

The resulting lesson for Tiler is not “every scalar instruction is a tensor
node” or “every pointwise chain is one logical Map.” Atomic named tensor
operations belong in the semantic graph; a typed scalar-expression DAG is
formed when region lowering chooses to compose them.

## Explicit broadcasting and access maps

StableHLO and MLIR Linalg require explicit broadcast semantics or indexing
maps. `ug` represents broadcast as a layout operation and reverse-maps a
broadcast coordinate to zero. TVM Relax accepts implicit broadcast syntax but
makes the accesses explicit during legalization.

Tiler therefore permits frontend shorthand but canonicalizes it into explicit
semantic axis mappings before optimization. Region lowering then composes those
mappings into access maps.

## Schedule representation precedents

Halide separates algorithm from schedule and explicitly represents loop
dimensions, splits, placement, storage dimensions, and tail strategies. Shape
estimates are autoscheduler inputs rather than executable semantics.

Sources:

- [Halide schedule representation](https://halide-lang.org/docs/_schedule_8h.html)
- [Halide `Func`](https://halide-lang.org/docs/class_halide_1_1_func.html)

TVM TensorIR exposes both the transformed module and a procedural schedule
trace. The module is current executable structure; the trace records and can
replay transformations and sampled decisions.

Sources:

- [TensorIR scheduling tutorial](https://tvm.apache.org/docs/deep_dive/tensor_ir/tutorials/tir_transformation.html)
- [TVM schedule trace](https://tvm.apache.org/docs/reference/api/doxygen/classtvm_1_1s__tir_1_1Trace.html)

MLIR's Transform dialect similarly separates transform/control IR from payload
IR. Transform application may fail and does not substitute for independently
verifying the resulting program.

Source: [MLIR Transform dialect](https://mlir.llvm.org/docs/Dialects/Transform/).

XLA GPU indexing analysis uses bounded maps from thread, block, and vector
coordinates to tensor coordinates. Fusion partitioning considers access-map
compatibility; identical operation membership is not sufficient when a shared
producer is reached through incompatible indices.

Sources:

- [XLA GPU emitters](https://openxla.org/xla/emitters)
- [XLA indexing analysis](https://openxla.org/xla/indexing)

The common lesson is to retain both result and recipe:

- normalized schedule IR is authoritative, canonical, verifiable, and
  identity-bearing;
- a transformation trace is explanatory and replayable;
- target profiles, selected capability requirements, resource requirements and
  estimates, and cost estimates remain separate typed concepts.

## Shape, bufferization, and execution scope

JAX export scopes symbolic dimensions and requires variables to be solvable
from input shapes. PyTorch similarly maintains symbolic shape environments and
emits guards. MLIR's Shape dialect makes shape computation and constraint
witnesses explicit. These precedents support scoped Tiler symbols and the
initial requirement that every output/temporary/guard/launch expression be
host-evaluable from declared inputs. They also show that data-dependent shapes
would require a separate shape/discovery program rather than another ordinary
extent expression.

Sources:

- [JAX symbolic shapes](https://docs.jax.dev/en/latest/export/shape_poly.html)
- [PyTorch dynamic shapes](https://docs.pytorch.org/docs/stable/user_guide/torch_compiler/compile/dynamic_shapes_core_concepts.html)
- [MLIR Shape dialect](https://mlir.llvm.org/docs/Dialects/ShapeDialect/)

MLIR One-Shot Bufferize performs whole-function use-def, alias, and equivalence
analysis before choosing in-place buffers or copies; deallocation is a separate
ownership problem. This supports keeping semantic tensor values distinct from
`KernelProgram` allocation identity and beginning with a conservative 1:1
buffer plan.

Sources:

- [MLIR Bufferization](https://mlir.llvm.org/docs/Bufferization/)
- [Ownership-based buffer deallocation](https://mlir.llvm.org/docs/OwnershipBasedBufferDeallocation/)

IREE Flow explicitly represents dispatch workloads and dynamic dimensions;
Stream/HAL add resource lifetimes, timepoints, affinities, queues, and device
placement. Tiler's `IndexRegion`, `ScheduledRegion`, kernel entry, and
`KernelProgram` already cover the narrower single-device dispatch path. IREE's
additional layers are evidence that multi-queue concurrency, cross-device
placement, and async resource availability cannot be inferred from a dependency
DAG and should be deferred explicitly rather than represented implicitly.

Sources:

- [IREE Flow dialect](https://iree.dev/reference/mlir-dialects/Flow/)
- [IREE Stream dialect](https://iree.dev/reference/mlir-dialects/Stream/)
- [IREE HAL dialect](https://iree.dev/reference/mlir-dialects/HAL/)
- [IREE Stream passes](https://iree.dev/reference/mlir-passes/Stream/)

PyTorch PrimTorch decomposes broadcasting and type promotion into explicit
primitives, reinforcing Tiler's explicit `Broadcast` and `Cast` semantics. FX
also demonstrates that data-dependent control flow is not captured by an
ordinary straight-line traced graph.

Sources:

- [PyTorch compiler IRs](https://docs.pytorch.org/docs/main/user_guide/torch_compiler/torch.compiler_ir.html)
- [PyTorch FX](https://docs.pytorch.org/docs/stable/fx.html)

## Database optimizer analogy

DataFusion strongly supports semantic/executable separation, per-child
requirements, output guarantees, enforcers, and invariant checking. It does
not provide a direct structural model for Tiler's shared tensor DAG or nested
GPU schedule search, and DataFusion explicitly does not claim a sophisticated
memoized cost-based optimizer.

Sources:

- [DataFusion crate architecture](https://docs.rs/datafusion/latest/datafusion/)
- [DataFusion execution plans](https://docs.rs/datafusion/latest/datafusion/physical_plan/trait.ExecutionPlan.html)
- [DataFusion invariants](https://datafusion.apache.org/contributor-guide/specification/invariants.html)
- [DataFusion optimizer discussion](https://datafusion.apache.org/blog/2025/06/15/optimizing-sql-dataframes-part-two/)

Cascades remains the stronger database precedent for memoized alternatives,
required properties, enforcers, and guided cost search. Tiler should use that
terminology only if its implementation actually supplies the corresponding
equivalence groups and goal-directed search.

Source: [The Cascades Framework for Query Optimization](https://www.sigmod.org/publications/dblp/db/journals/debu/Graefe95a.html).

The durable conclusion is a qualified analogy:

```text
SemanticTensorGraph
    -> CandidateRegionSet
    -> ImplementationFrontier(region, target)
    -> selected RegionPartition and implementations
    -> KernelProgram
    -> guarded ProgramPortfolio
```

Boundary requirements and guarantees are the closest analogue to database
physical properties. Numerical contracts, target requirements, applicability
predicates, resource envelopes, schedule invariants, and costs are separate
categories.

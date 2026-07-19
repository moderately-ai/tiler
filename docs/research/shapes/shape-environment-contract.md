# Shape environment contract research memo

**Status:** research in progress; candidate contract, not an ADR  
**Ticket:** `shape-environment-contract`  
**Scope:** ranked tensors with static or symbolic axis extents

## Purpose

This memo defines which tensor-shape facts exist at semantic-graph time, where
runtime dimension values originate, which constraints are semantic, and which
shape expressions the host must be able to evaluate before dispatch.

Facts, inferences, proposals, accepted decisions, and measurements are labeled
separately. The existing contract documents remain unchanged until synthesis.

## Evidence: ranked versus unranked compilation

### Primary-source facts

- StableHLO permits dynamic dimension sizes but explicitly prohibits a dynamic
  number of dimensions. Its axes are numbered from `0` through fixed rank
  `R - 1`. [StableHLO specification](https://openxla.org/stablehlo/spec).
- TensorRT permits runtime dimensions and optimization profiles but requires
  every tensor's rank at engine-build time. It rejects a reshape whose runtime
  shape-tensor length would make the output rank unknown.
  [TensorRT dynamic-shape restrictions](https://docs.nvidia.com/deeplearning/tensorrt/10.x.x/inference-library/dynamic-shapes-advanced.html).
- ONNX can represent an internal tensor of unknown rank, but top-level graph
  inputs and outputs must carry a shape that establishes rank. Its portable IR
  does not imply that an optimizing backend can compile every unranked value.
  [ONNX IR specification](https://onnx.ai/onnx/repo-docs/IR.html).
- TVM Relax and MLIR can retain unknown-rank tensors at a high abstraction
  level. MLIR's code-generation guidance nevertheless discourages unranked
  buffers: efficient code generation needs the number of enclosing loops. It
  recommends specializing/casting to ranked form, dispatching through a rank
  switch, or accepting expensive generic linearization and delinearization.
  [TVM Relax tensor type](https://tvm.apache.org/docs/reference/api/doxygen/classtvm_1_1relax_1_1TensorTypeNode.html),
  [MLIR builtin types](https://mlir.llvm.org/docs/Dialects/Builtin/).
- JAX export supports symbolic dimension expressions within a fixed-rank
  argument specification. Its ellipsis is resolved from the argument
  specification rather than becoming an arbitrary runtime-rank value.
  [JAX shape polymorphism](https://docs.jax.dev/en/latest/export/shape_poly.html).

### Inference

Unknown rank is representable, but it is a distinct specialization problem.
It makes the number of axis identities, result dimensions, strides, access-map
arguments, loop variables, and ABI fields runtime-dependent. Tiler's index and
schedule representations instead require a fixed collection of axes whose
extent values may remain symbolic.

The database analogy is schema arity versus cardinality: a logical relational
operator normally has fixed output fields and types even though row counts are
unknown. A runtime-dependent output schema requires a boundary before ordinary
relational optimization; it is not merely another cardinality estimate.

## Accepted decision: rank boundary

**Accepted by Tom on 2026-07-18:**

```text
rank-polymorphic frontend plan
    -> rank resolution or finite rank specialization
    -> ranked SemanticTensorGraph
    -> semantic optimization and index/access lowering
```

Every tensor value in a `SemanticTensorGraph` submitted to Tiler's optimizer
has statically known rank. Each axis extent may be a static integer or a scoped
symbolic expression evaluated later.

Rank-polymorphic frontend syntax remains permitted. A frontend may resolve it,
construct a finite guarded portfolio of ranked graphs, or fall back. A future
`RankPolymorphicProgram` or rank-specialization layer is not precluded, but
`Unranked` is not part of the initial semantic tensor type and every operation
capability is not required to handle it.

This decision does not permit a frontend to silently choose an arbitrary rank:
the selected ranked graph or guarded portfolio is part of compilation and
interface identity and must be visible in explanation output.

## Consequences for later shape decisions

With fixed rank, `ShapeEnv` reasons over a finite set of declared extent
symbols and expressions. Remaining questions are:

1. which static and runtime sources may bind a root extent symbol;
2. which integer expression fragment is canonical and host-evaluable;
3. how semantic input constraints differ from inferred facts and physical
   variant guards;
4. whether zero extents are valid and how intervals represent them;
5. whether data-dependent extents are rejected or require a separate shape
   program;
6. which values are specialized into artifacts versus passed through the ABI;
7. how overflow and index-width proofs are represented.

These decisions are intentionally not inferred from the fixed-rank decision.

## Accepted decision: root extent sources

**Accepted by Tom on 2026-07-19:** a runtime root extent may be bound from an
input tensor dimension or from an explicitly declared host integer shape
parameter.

```text
ExtentSource =
    Static(u64)
  | InputDimension { input: InputIndex, axis: AxisIndex }
  | ShapeParameter { parameter: ShapeParameterIndex }
```

A `ShapeParameter` is part of the program interface. It is an immutable,
nonnegative host integer available before allocation, routing, or dispatch. It
has a stable declaration, type, optional diagnostic name, and semantic
constraints. It is not:

- a rank-zero tensor value whose contents may reside on a device;
- an operation attribute fixed during compilation;
- an ambient callback, environment variable, or consumer-global value;
- a derived solver fact without an interface binding.

JAX export currently requires dimension variables to be solvable from input
tensor shapes; its documented workaround for a runtime `top_k` parameter is a
dummy tensor with shape `(0, k)`. Tiler will model that input explicitly rather
than encode metadata as a tensor shape.
[JAX sourceability restriction](https://docs.jax.dev/en/latest/export/shape_poly.html#dimension-variables-must-be-solvable-from-the-input-shapes).

The same integer may participate in tensor semantics as runtime metadata when
an operation definition declares that use. Its shape-source role remains
typed, and the host and generated kernel ABI derive their encoding from one
declaration.

Root bindings participate in semantic computation and interface identity.
Optional diagnostic names follow the accepted split identity rule: they are
excluded from computation identity unless external binding is name-based, but
included in an artifact/interface identity whenever the ABI exposes them.

Every non-root extent is a canonical expression over declared roots and
constants. Free symbols, ambiguous bindings, multiple incompatible bindings,
and references to tensor element data are invalid in the initial `ShapeEnv`.

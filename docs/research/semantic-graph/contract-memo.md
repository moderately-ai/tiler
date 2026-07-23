---
schema: "tiler-doc/v1"
id: "tiler.research.semantic-graph.contract-memo"
kind: "research"
title: "Semantic tensor graph contract research memo"
topics: ["semantics", "graph", "ir"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "adopted"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.vision", "tiler.contract.ir"]
adopted_by: ["ADR-0005", "ADR-0006"]
ticket: "semantic-graph-contract"
---

# Semantic tensor graph contract research memo

**Status:** completed research adopted by ADRs 0005 and 0006
**Ticket:** `semantic-graph-contract`  
**Scope:** target-independent tensor semantics only

## Question

What is the smallest public graph model that can faithfully describe a pure
tensor computation with sharing, multi-result operations, and several external
results, while leaving fusion, materialization, layout, and device scheduling
to later compiler stages?

## Executive conclusion

The strongest initial model is a **pure, immutable, acyclic operation/value
graph with a separate program interface**:

- an **operation invocation** records one application of versioned operation
  semantics to ordered operand values and canonical semantic attributes;
- each operation exposes one or more ordered **result ports**;
- each result port defines a separately typed **value**;
- a value has exactly one definition and any number of uses;
- ordered program inputs and ordered, named program results form the graph's
  external interface;
- program results reference values and are not synthetic operations;
- all initial values are tensors, including rank-zero tensors;
- the reachable graph is pure and acyclic, with no mutation, hidden state,
  randomness, I/O, control-flow regions, or data-dependent execution order.

This is an ordinary directed dataflow graph when represented as explicit
operation and value entities. A value with several uses is mathematically a
multi-edge, but calling the public IR a hypergraph does not improve its use-def
model. Hyperedges remain useful as an analysis view for overlapping fusion
candidates, not as the semantic graph's primary representation.

## Evidence

### Facts from primary precedents

1. MLIR operations have ordered operands, attributes/properties, and zero or
   more ordered SSA results. Results are independently typed values, and an SSA
   value can have multiple uses. MLIR explicitly describes a value in a graph
   region as a multi-edge from one source operation to multiple destinations.
   MLIR is broader than Tiler: graph regions can contain cycles, operations can
   have effects and regions, and zero-result operations are valid.
   [MLIR language reference](https://mlir.llvm.org/docs/LangRef/),
   [MLIR `Operation`](https://mlir.llvm.org/doxygen/classmlir_1_1Operation.html).

2. ONNX specifies a side-effect-free acyclic computation graph whose nodes
   invoke operators and have zero or more inputs and one or more outputs. Node
   outputs obey SSA naming, edges arise when another node references an output,
   and graph inputs and outputs are separate interface lists. ONNX graph
   outputs are uses of values, not output nodes. ONNX also separates propagated
   values from static operation attributes and requires operator signatures to
   validate arity, types, and attributes.
   [ONNX IR specification](https://onnx.ai/onnx/repo-docs/IR.html),
   [normative source](https://github.com/onnx/onnx/blob/main/docs/IR.md).

3. StableHLO uses named, versioned tensor operations on MLIR's operation/value
   substrate. Its specification includes tuple values and result-producing
   operations, explicit broadcasting, decomposable composite operations, and
   opaque custom calls. This shows both that multi-result semantics are normal
   and that an unknown operation is not automatically safe to optimize.
   [StableHLO specification](https://openxla.org/stablehlo/spec),
   [StableHLO repository](https://github.com/openxla/stablehlo).

4. DataFusion's logical plan cleanly separates logical operations from physical
   implementations and exposes extension nodes, but its relational tree and
   single tabular result schema are not a sufficient structural model for a
   shared, multi-output tensor graph.
   [DataFusion logical plans](https://datafusion.apache.org/library-user-guide/building-logical-plans.html).

### Inferences for Tiler

1. **Operation and value must be distinct concepts.** A node-only model works
   only while every operation has exactly one result. Adding tuples to repair
   that model would make tuple construction an IR encoding artifact rather than
   tensor semantics.

2. **A result port is neither another node nor an arbitrary node property.** It
   is an ordered position in an operation's semantic result signature. The
   value defined at that port owns the use-def identity and verified tensor
   type.

3. **Graph output is an interface binding, not computation.** Making output a
   synthetic operation would falsely create semantics, complicate returning the
   same value twice, and conflate computation identity with ABI naming.

4. **Separate atomic operations do not imply materialization.** Operation
   boundaries preserve named semantics and rewrite opportunities. Buffer
   allocation, rounding at an explicit materialization, fusion, recomputation,
   and kernel partitioning are later choices.

5. **Purity must be structural, not conventional.** A boolean `has_effects`
   field cannot define ordering, aliasing, randomness, or resource behavior.
   Until an effect/resource-token model exists, such operations should be
   rejected from this graph rather than accepted as opaque ordered nodes.

6. **The public graph should be narrower than MLIR or ONNX.** General sequences,
   maps, optional values, graph-valued attributes, nested regions, calls, and
   control flow are not required to prove the tensor optimizer architecture.
   Supporting them early would force effect, reachability, and interprocedural
   contracts before the first vertical slice.

## Candidate data model

The model below is conceptual rather than a Rust API commitment:

```text
SemanticTensorGraph {
    inputs:       Vec<ProgramInput>,
    operations:   Arena<OperationInvocation>,
    values:       Arena<Value>,
    results:      Vec<ProgramResult>,
    shape_env:    ShapeEnvRef,
}

ProgramInput {
    name: Option<InterfaceName>,
    contract: TensorInputContract,
}

OperationInvocation {
    op_key: OpKey,                    // dialect + operation + semantic version
    operands: Vec<ValueId>,           // ordered by the operation signature
    attributes: CanonicalAttributes,  // semantic constants, never cached facts
    results: Vec<ValueId>,            // ordered result ports
    source: Option<SourceOrigin>,      // diagnostic only
}

Value {
    definition: Input(InputIndex)
              | OpResult(OperationId, ResultIndex),
    verified_type: TensorType,
}

ProgramResult {
    name: Option<InterfaceName>,
    value: ValueId,
    contract: TensorResultContract,
}
```

`OperationId` and `ValueId` are graph-local arena handles. They are not durable
semantic identifiers and cannot be referenced by another graph.

### What is a node?

In user-facing graph language, a **node** is one `OperationInvocation`: one
application of an operation definition to operands and attributes. Internally,
using the precise term `operation invocation` avoids confusing:

- the reusable operation definition (`OpKey` plus registered capabilities);
- this occurrence of that operation in a graph (`OperationInvocation`);
- its independently addressable result values (`Value`);
- physical kernels or dispatches chosen later.

### What is a node property?

Meaning-defining, invocation-local information is stored on the operation:

- operation identity and semantic version;
- ordered operands;
- canonical attributes such as reduction axes, comparison direction, or an
  explicit broadcast axis map;
- ordered result-port identities;
- optional source origin excluded from semantic identity.

The following are **derived facts**, not trusted semantic attributes:

- result shapes and dtypes;
- inferred equality or divisibility facts;
- use lists and use counts;
- reachability and topological position.

The following are **not semantic node properties**:

- storage layout or strides;
- allocation or alias decisions;
- fusion membership and materialization;
- index-width fast paths;
- tile, grid, threadgroup, warp/SIMD-group, or vector mapping;
- memory-space placement, barriers, occupancy, register estimates, or cost.

Those belong to index/access, region, schedule, kernel, target-profile, or
whole-program representations.

## Candidate invariants

### Ownership and references

1. The graph owns every input declaration, operation invocation, and value.
2. Every ID is local to exactly one graph and is bounds-checked on use.
3. Every value is defined exactly once by an input position or operation result
   port.
4. Every operation result port defines exactly one value, and the reverse
   `Value::OpResult` reference agrees with it.
5. A value may have zero or more operation uses and zero or more program-result
   uses before reachability pruning.
6. Every operand and program result references an existing value in the same
   graph.

### Structure

7. The operation dependency relation is acyclic.
8. Serialized operation order may be topological, but insertion order and arena
   numbering are not semantic.
9. The canonical graph contains exactly the transitive closure reachable from
   ordered program results.
10. The initial graph contains at least one program result. Empty programs can
    be reconsidered with modules or effectful execution.
11. Every initial operation has at least one result. A pure zero-result
    operation is unobservable; an effectful one is out of scope.
12. Several program results may reference one value. Several values may be
    defined by one operation.

### Types and operation semantics

13. Every value is a tensor with statically known rank and element dtype;
    extents may be symbolic under the separate ShapeEnv contract.
14. Rank-zero tensors represent scalar data. Shape/index metadata is not
    smuggled in as scalar tensor data.
15. An operation definition validates operand/result arity, attributes, shape
    constraints, and dtype rules.
16. Stored result types are verified products of inference, not frontend
    assertions. Deserialization reruns or checks inference against them.
17. Binary broadcasting, casts, quantization, accumulation dtype, reduction
    axes, and other observable behavior are explicit semantics.
18. Missing extension knowledge is conservative: it may block optimization or
    executability, but never licenses a rewrite.

### Purity and determinism

19. Evaluation depends only on operand tensor values, canonical attributes, and
    declared semantic shape sources.
20. Mutation, I/O, time, hidden randomness, global state, and implicit resource
    access are rejected.
21. Pure operations are referentially transparent under their declared
    numerical contract.
22. Canonical attribute encoding is total, deterministic, and independent of
    Rust allocation addresses, map iteration order, registry addresses, source
    spans, and caches.

### Program interface

23. Inputs and results are ordered interface lists; their arity and contracts
    are validated.
24. Result names, when present, are unique within the interface.
25. A program result's verified tensor type satisfies its result contract.
26. An immutable weight supplied by the caller is a program input. An embedded
    tensor literal is a zero-operand `Constant` operation whose canonical typed
    payload participates in computation identity.

## Minimal end-to-end example

The example deliberately contains sharing, a multi-result operation, and
multiple program results:

```text
inputs:
  x:     tensor<B,H,W,C x f16>
  scale: tensor<C x f16>
  bias:  tensor<C x f16>

v3 = Broadcast(scale, map=[C -> C])
v4 = Multiply(x, v3)
v5 = Broadcast(bias, map=[C -> C])
v6 = Add(v4, v5)
v7 = Gelu(v6)
(v8, v9) = ReduceWithArgMax(v7, axes=[H,W], accumulator=f32)

results:
  activations = v7
  totals      = v8
  maxima_at   = v9
```

This graph says:

- `v7` has two uses: one program result and one reduction operand;
- `ReduceWithArgMax` is one operation with two independently typed results;
- three external tensors are returned;
- no tensor is necessarily materialized merely because it is a value;
- no kernel count, allocation, layout, reduction topology, or device mapping is
  implied.

A first executable slice does not need to implement `ReduceWithArgMax`; it is
included to validate that the graph architecture does not assume one result per
operation. A smaller implemented slice may use separate `Reduce` operations.

## Counterexamples and rejection cases

### Node-only result

```text
node = ReduceWithArgMax(x)
consumer(node) // Which result?
```

Rejected as an IR design because the use cannot identify a result without
inventing tuple projection or result-port identity later.

### Synthetic output operation

```text
Output(name="a", v)
Output(name="b", v)
```

Rejected because external naming is an interface concern, and the two nodes
falsely suggest computation or ordering. `results = [("a", v), ("b", v)]`
expresses the interface directly.

### Implicit materialization

```text
v1 = Cast<f16>(x)
v2 = Add(v1, y)
```

The cast is semantic; a store/reload boundary is not. If the operation's
normative semantics require f16 quantization, lowering must preserve that
rounding explicitly even when no intermediate buffer exists.

### Hidden random state

```text
v = Dropout(x, probability=0.5) // no seed/state operand or result
```

Rejected from the initial graph because identical operands and attributes do
not determine the result. A future design could pass and return explicit random
state or effect tokens.

### In-place update

```text
UpdateSlice(x, indices, update) // implicitly mutates x
```

Rejected from the initial graph. A pure `UpdatedSlice` returning a new tensor
is admissible if its complete value semantics are specified; choosing in-place
storage remains physical.

### Shape from tensor data

```text
indices = NonZero(x)
```

The values may be semantically expressible, but their data-dependent output
extent cannot enter the initial host-evaluable ShapeEnv without a separate
shape/discovery program. The operation is therefore outside the first dynamic
shape contract unless its result is bounded and represented by an explicit
future mechanism.

### Cross-graph value

Using a `ValueId` created by another graph is rejected even if its numeric arena
index happens to exist. Cross-program composition requires explicit import,
inlining, or a future module/call model.

## Identity questions exposed by the model

### Computation identity versus interface identity

Input and result names are useful for diagnostics and host binding, but changing
`"totals"` to `"sum"` does not change tensor mathematics. Conversely, ordered
input/result positions, result value references, tensor contracts, and aliasing
the same value into two result positions affect the callable interface.

**Proposal:** maintain two explicit identities:

1. **semantic computation identity** excludes optional interface names and
   source origins but includes the reachable operation/value structure,
   constants, semantic attributes, numerical contract, types, and shape
   constraints;
2. **compilation/interface identity** additionally includes ordered input and
   result contracts, binding policy, names where the generated ABI exposes
   them, guards, and later physical/artifact choices.

This permits computation-cache reuse across harmless renaming without letting
an ABI manifest reuse the wrong external bindings.

### Sharing versus duplicated pure computation

These graphs are extensionally equivalent for a deterministic pure operation:

```text
a = Gelu(x); outputs = [a, a]

a = Gelu(x); b = Gelu(x); outputs = [a, b]
```

They are structurally different and present different recomputation/sharing
hints to a planner. Saying that insertion order and arena IDs are excluded from
identity does not decide whether these graphs hash alike.

**Proposal:** make a verified, deterministic semantic normalization step
authoritative before computation identity. It performs dead-code elimination
and may perform common-subexpression elimination only for operation definitions
that explicitly permit it under the numerical contract. The durable identity
commits to the normalized graph. Until that rule is accepted, structural
sharing must participate in identity to avoid claiming equivalence without a
defined normalization proof.

Canonical numbering can then be assigned by deterministic traversal from the
ordered result list through ordered operands and ordered result ports. Any
commutative operand normalization must be an operation-specific semantic
rewrite, not a serializer trick.

## Validation boundaries

Validation should be staged so each failure identifies its owner:

1. **Structural verifier:** local IDs, reciprocal operation/result references,
   arity containers, acyclicity, reachability, unique result names, and
   graph-local ownership.
2. **Operation verifier:** resolve `OpKey`; validate canonical attributes,
   operand/result schemas, semantic constraints, and declared purity.
3. **Type/shape inference verifier:** infer result tensor types and constraints;
   compare against stored verified types; reject contradictions and free shape
   symbols.
4. **Semantic normalization verifier:** ensure every rewrite declares its
   numerical equivalence contract and returns another valid graph.
5. **Executability analysis:** separately report whether each operation can
   decompose, lower to index/access form, call an opaque implementation, or must
   remain an unsupported boundary. Semantic validity alone does not promise an
   executable backend plan.

## Explicitly deferred cases

- effect and resource tokens;
- mutation and alias-aware semantic operations;
- seeded or state-threaded randomness;
- semantic functions, calls, modules, recursion, and interprocedural identity;
- graph-level branches, loops, and region-bearing operations;
- data-dependent result rank or extent and shape/discovery programs;
- non-tensor semantic values, sparse tensors, sequences, maps, and optionals;
- cross-device placement, transfers, collectives, and sharding;
- autodiff semantics and backward graph generation;
- zero-result operations;
- public ABI naming and language-specific calling conventions.

## Decisions requested

The evidence supports the operation/value graph strongly. Two identity choices
are material:

1. **Accepted by Tom on 2026-07-18:** maintain separate computation and
   interface/artifact identities. Optional input and result names are excluded
   from computation identity. They participate in interface/artifact identity
   only where the binding or ABI exposes them.
2. **Accepted by Tom on 2026-07-18:** normalize identical referentially
   transparent operation invocations to one semantic value before computation
   identity. Equality requires the same operation key, operands, canonical
   attributes, numerical contract, and inferred result types. Source origins
   are preserved for explanation but excluded from equality. Physical planning
   may still recompute the shared value independently when that is cheaper than
   reuse or materialization.

   **Relocated, not superseded (confirmed by Tom on 2026-07-23):** ADR 0064
   places common-subexpression elimination outside commitment compaction and in
   "their existing later layers". That moved this merge obligation's home; it
   did not reject the 2026-07-18 decision. Its home is the deterministic
   semantic normalization stage proposed above under "Sharing versus duplicated
   pure computation", which runs before computation identity is committed. The
   implementation obligation is recorded on
   [`prototype-semantic-normalization`](../../../tickets/prototype-semantic-normalization.md);
   no contract or ADR restates it, and ADR 0064 is not superseded.

These choices do not change the operation/value shape of the graph, but they do
define cache identity, deterministic serialization, and explain output.

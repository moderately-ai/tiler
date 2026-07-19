# IR stack and invariants

**Status:** proposed

Tiler uses several representations because tensor semantics, symbolic indexing,
hardware scheduling, and imperative kernel code have different invariants.
Collapsing them into one universal IR would make target choices semantic and
make malformed programs difficult to reject early.

## Common invariants

Every durable Tiler representation must satisfy these rules:

1. IDs are local to one representation/program, never process-global.
2. Construction, canonical serialization, and hashing are deterministic.
3. Serialized forms carry a schema version.
4. Values have a statically known kind and dtype.
5. Extents and indices have an explicit integer type.
6. Narrowing conversions are checked and represented explicitly.
7. Runtime assumptions appear as guards, not comments.
8. Malformed IR is rejected before source generation.
9. Passes state the numerical equivalence relation they preserve.
10. Floating constants have a defined bit-level equality and hash policy.
11. Source origins survive lowering sufficiently for diagnostics and `EXPLAIN`.
12. Artifact identity uses canonical content, never allocation identity.

## Layer 0: frontend plan

The frontend plan retains syntax-level information such as axis names,
grouping, ellipses, and source spans. It validates operation-specific axis
rules and translates them into generic tensor semantics.

It must not contain storage strides, thread IDs, materialization decisions, or
Metal details. This layer normally remains owned by the frontend crate.

Required properties include:

- every input has a resolved logical rank or rank constraint;
- introduced axes have known extent expressions;
- removed axes name an explicit reduction;
- composed axes have factorization constraints;
- output axis order is complete and unambiguous.

## Layer 1: public semantic tensor graph

`SemanticTensorGraph` is the public, frontend-neutral semantic representation.
It is a pure, backend-neutral operation/value DAG describing what tensor values
mean. Frontends construct this graph; no frontend syntax, consumer runtime
object, storage layout, kernel boundary, target schedule, or live device object
belongs in it. Its extent expressions reference scoped symbols. A separate
typed semantic interface binds those symbols from static values, input
metadata, caller parameters, or admitted versioned target properties.

This permits explicitly target-parameterized semantics without making target
queries into tensor operations or shape-expression primitives. The graph is a
function over its unbound symbols; the graph plus binding environment is the
closed semantic program interface used for validation and compilation.

The initial compilation unit is one straight-line graph with ordered inputs and
results. It has no semantic functions/calls, recursion, region-bearing control
flow, data-dependent branches, or semantic loops. Frontends inline such work or
submit separate graphs. Scalar-expression `select` is elementwise computation,
not graph control flow. A future `SemanticModule` and structured control-flow
model require separate decisions about effects, reachability, shape constraints,
and interprocedural identity.

The durable graph is an operation/value model rather than a node-only tree:

ADR 0005 accepts the public graph and extension boundary. The concrete
operation/value model below remains proposed in ADR 0006.

```text
ProgramInput {
    name,
    tensor_type_or_constraints,
}

Operation {
    key: OpKey,                 // dialect + name + semantic version
    operands: Vec<ValueId>,
    canonical_attributes,
    results: Vec<ValueId>,
}

Value {
    definition: Input(i) | OpResult(OperationId, result_index),
    tensor_type,
}

ProgramResult {
    name,
    value: ValueId,
    result_contract,
}
```

`OperationId` and `ValueId` are arena-local handles, not semantic identity.
Every non-input value has exactly one defining operation result. Operations may
have several results, and values may have several consumers. Program results
are a separate ordered, named list of value references rather than synthetic
`Output` operations. A program may return several independently shaped and
typed tensors, and two result declarations may intentionally reference the
same value.
Whether result names participate in semantic identity remains open; ordered
result arity, value references, and result contracts do participate.

All initial semantic values are tensors; rank-zero tensors represent scalar
data. This initial restriction is not a claim that every future graph value
must be a tensor. A later effect model may add explicitly kinded resource or
effect-token values without reinterpreting existing tensor values. Unsupported
value kinds are rejected at schema and capability boundaries. `ProgramInput`
covers runtime tensor parameters and immutable weights.
`Constant` owns a shape plus canonical typed bit payload included in semantic
identity. Shape/index metadata scalars are not tensor values and instead enter
through declared symbolic sources. Externalizing a large constant is an
artifact-packaging policy and must not silently change semantic identity.
Whether input names participate in identity follows the same unresolved policy
as result names.

Element-type representability is intentionally broader than executable
operation support. A tensor may carry a recognized exact element type through
operations whose declared semantics support it, such as a bit-preserving view,
without implying that arithmetic, the reference evaluator, every optimizer
pass, or any backend supports that type. Verification checks each operation's
complete typed signature and required capabilities.

A representable type is still known, versioned, and canonical; this is not an
unknown-type escape hatch. Initial verified graphs reject unregistered nominal
type identities. Backend compilation separately proves the selected storage
encoding, ABI, and realization for every operation/type combination in the
physical plan.

Built-in and extension element types share one durable nominal identity model.
Conceptually, a type key contains a namespace, name, and semantic version:
`tiler::f32@1` and `acme::fp8_special@1` differ by identity even if some
structural facts coincide. Built-ins may have ergonomic Rust spellings such as
`DType::F32`, but canonical hashing, serialization, registry lookup, and
capability diagnostics use the durable key rather than a Rust enum
discriminant, `TypeId`, or address. A canonical type descriptor supplies the
format's structural and value-semantic facts; those facts do not replace its
nominal identity.

The graph initially contains atomic named tensor operations. Representative
built-ins include:

```text
Constant    Cast           Reindex     Broadcast
Add         Multiply       Gelu
Reduce
```

Program inputs are declarations rather than operation invocations. The
operation list is illustrative rather than a closed Rust enum. An operation
invocation is a graph node; its axes, reduction kind, accumulator dtype, and
other meaning-defining parameters are canonical semantic attributes. Shape,
result dtype, and constraints are inferred semantic facts. Layout,
alignment, materialization, tiling, and thread mapping are not logical
properties.

Separate semantic operation nodes do not imply intermediate allocation or
additional rounding merely because they are separate nodes. Explicit casts,
quantization, and each operation's normative dtype semantics remain observable
and must be preserved across fusion. Fusion, recomputation, and materialization
are physical choices. For
example, the semantic chain

```text
Broadcast(scale) -> Multiply -> Add -> Gelu -> Reduce
```

may become one fused scalar/reduction expression, two materialized kernels, or
another contract-conforming physical implementation. Keeping named operations
until physical exploration preserves sharing, operation-specific rewrites,
extension identity, and explainability.

`Reindex` represents a total output-to-input coordinate function plus its shape
constraints. Initial reindexes are bijective permutations/split/merge mappings
or legal removal/insertion of unit axes. Many-to-one broadcast/repeat behavior
is represented separately by an explicit `Broadcast` with an axis mapping. It
does not claim that storage was transposed or copied. Frontends may accept
implicit broadcasting syntax, but the canonical semantic graph makes the
mapping explicit before optimization.

### Proposed public experimental operation extension contract

Built-in and third-party operations use the same public experimental operation
definition path. Durable IR stores an `OpKey`, canonical attributes, operands,
and results; it never serializes Rust trait objects or registry addresses. A
registry resolves `OpKey` to versioned operation capabilities.

Mandatory capabilities define:

- operand/result schema and arity;
- shape, dtype, axis, and semantic-constraint inference and verification;
- canonical attribute encoding and deterministic identity;
- purity/effect declaration;
- normative/reference semantics and conformance behavior;
- explain and diagnostic formatting.

Optional capabilities may provide:

- decomposition into other semantic operations;
- canonicalization and contract-preserving rewrite rules;
- iteration-domain and access-map lowering;
- region-fusion participation;
- physical implementations, boundary requirements/guarantees, and costing;
- structured-kernel lowering.

Registration alone does not make an operation optimizable. A pass may transform
an extension only when the operation decomposes into understood semantics or
supplies every interface and proof that the pass requires. Missing optional
knowledge is conservative. Missing rewrite or fusion support makes the
operation an optimization boundary. If no decomposition, iteration/access
lowering, physical implementation, or explicit opaque implementation exists,
the operation remains valid semantic IR but Tiler cannot construct an
executable program for it and must diagnose or delegate it. Compiler/artifact
identity must include the registered dialect's semantic and lowering
fingerprint.

The initial extension execution model is trusted, statically linked compiler
code supplied explicitly to one compiler session. It does not promise native
dynamic plugin loading, sandboxing, or automatic discovery of consumer-local
registrations by a separately compiled proc macro. Registry, canonical-data,
provider-identity, threading, panic, and rewrite-transaction invariants are
specified in [Operation extensions](operation-extensions.md).

### Graph and semantic verifier

- The initial graph is pure, immutable, acyclic tensor SSA with statically known
  rank and optionally dynamic extents. Stateful effects,
  mutation, hidden randomness, and I/O are rejected until explicit effect or
  resource tokens are designed. Floating-point exception cases initially have
  explicit value-only, no-observable-flag semantics rather than hidden effects.
- Every operand references an existing, type-compatible value, and every
  non-input value has exactly one definition.
- Every initial semantic operation produces one or more ordered, individually
  typed tensor results. A future effect model may add non-tensor token results
  and, if justified, zero-result operations through a new versioned capability;
  it cannot silently broaden the meaning of an existing pure operation.
- Operation results and program results are ordered and individually typed.
- Result names are unique; result values exist and match their contracts.
- Output shapes and dtypes are derived rather than trusted assertions.
- Every tensor value has a resolved value dtype, and every operation has a
  resolved numerical signature. Canonical semantic IR contains no ambient
  frontend promotion, weak-scalar, default-dtype, or autocast decision.
- A resolved dtype need only be representable at the value boundary. Every
  operation separately proves that its full typed signature is semantically
  admitted; representability alone grants no evaluator, optimizer, or backend
  capability.
- Ordinary elementwise mixed-dtype inputs use explicit semantic conversions.
  Operations with intrinsic mixed precision, such as reductions and
  contractions, declare computation precision, accumulator/result types, and
  relevant order or algorithm contracts through their specialized semantics.
- Every numeric conversion carries a resolved, typed conversion contract for
  its conversion family. Source and destination dtype alone are not a complete
  conversion, and canonical IR does not inherit ambient rounding or exceptional
  value behavior.
- Every operation's effective numerical optimization permissions are resolved
  and no more permissive than the program policy ceiling. Optimizer and
  scheduling rules must name the effective permission they consume.
- Required single-rounding fused multiply-add is a dedicated semantic
  operation. Separate multiply and add operations remain separate rounding
  boundaries unless their resolved contraction permission authorizes fusion.
- Every transcendental operation carries a resolved accuracy contract. No
  canonical operation inherits transcendental accuracy from backend defaults
  or ambient compiler flags.
- The initial optimizer enforces local numerical contracts and does not
  redistribute a graph-level error budget. Reference provenance, input/shape
  assumptions, casts, materialization boundaries, and reduction topology remain
  available to a future explicit region-accuracy analysis.
- Every root extent symbol has exactly one typed binding whose source class and
  availability phase are supported by every semantic factor that consumes it.
- Target-property bindings use stable versioned keys and cannot depend on a
  selected or prepared physical pipeline in the initial execution model.
- Binary operations use explicit broadcasting.
- Reindex mappings are total over their output domain.
- Reductions name valid axes and explicit accumulation/output dtypes.
- Every reduction declares a typed empty-domain result or rejects empty input.
  Empty result, algebraic identity, and replicable physical padding are
  separate capabilities. An explicit initial value is one logical contributor
  for every reduction domain, not an empty-only fallback; schedules may inject
  only padding proven neutral under the selected conformance contract and may
  never replicate an arbitrary seed.
- Reduction semantic nodes constrain the legal evaluation-order or result
  class, while concrete reduction trees, partitioning, and multi-pass topology
  belong to selected physical plans and artifact identity.
- Reduction contracts distinguish regrouping from operand permutation. Neither
  permission implies the other, and each requires the corresponding operation
  capability before a schedule may consume it.
- Determinism guarantees name their stability scope. Canonical contracts do
  not contain an unqualified deterministic boolean.
- Portable-bitwise arithmetic uses a versioned canonical quiet-NaN result per
  dtype. Bit-preserving operations retain source bits, and other NaN behaviors
  must be explicit operation contracts.
- Subnormal input treatment and subnormal result treatment are independently
  resolved. Portable-bitwise contracts preserve both; a backend's coupled
  flush mode cannot widen operation permissions.
- Every initial floating-point operation uses the explicit value-only
  exception-observation contract. Unknown future effect signatures or
  exception-observation modes are rejected rather than treated as pure.
- The canonical graph contains only the transitive closure reachable from all
  program results; dead pure operations are removed before identity is formed.
- Stable serialization and hashing do not depend on arena IDs, insertion order,
  source spans, cached use lists, or registry addresses.
- Shared values remain graph sharing; use count is not a materialization rule.

## Constraint and proof context

Semantic and index lowering share a typed `ShapeEnv` containing scoped symbol
declarations, source bindings, and a constraint environment containing
extent equalities, divisibility, nonnegativity, intervals, and factorization
relationships. Facts record provenance: statically proven, frontend-required,
or runtime-validated.

Value-domain facts use the same provenance discipline but are not shape facts.
The initial optimizer may consume compiler-proven or runtime-validated value
facts for correctness-sensitive transformations. It records caller-declared,
unvalidated value assumptions for diagnostics and future policy evolution but
does not trust them for legality. A tensor-content validation may be a costed
preflight computation rather than a scalar dispatch predicate.

Every extent symbol has one declaration and one typed static or runtime root
binding; equal spelling in different scopes never implies equality, and free
symbols are invalid. Contradictory semantic constraints reject the graph.
Inferred or proven facts may not silently become additional frontend-required
semantics. Canonical identity includes symbol declarations, root-binding
provenance, and semantic constraints but excludes derived solver caches. The
solver algorithm and exact supported arithmetic fragment remain implementation
choices.

A **semantic input constraint** is required for the expression to be defined,
such as a split-axis factorization. A **variant guard** is required only for a
particular optimization, such as 16-byte alignment. They are not
interchangeable. Later guards also record provenance as storage-applicability,
schedule-applicability, target-compatibility, or dispatch-safety predicates.
Failure of a semantic input constraint is an invalid-input diagnostic. Failure
of a variant guard selects another valid plan or fallback before dependent work
begins.

## Layer 2: index and iteration IR

This layer converts a proposed semantic region into a canonical `IndexRegion`
containing symbolic iteration domains, scalar computation, and access maps.
Atomic semantic operations may be composed
here into a fused scalar-expression DAG after a region candidate has been
formed. An access map answers:

```text
(output coordinates, reduction coordinates, runtime metadata)
    -> buffer element offset
```

Core concepts:

```text
ExtentExpr        IndexExpr          ScalarExpr
IterationVar      IterationDomain    ReductionDomain
AccessMap         StorageLayout      ProvenFact         BufferView
```

The typed `ScalarExpr` vocabulary contains argument references, constants,
booleans, comparisons, select, arithmetic, min/max, casts, and an explicit set
of elementary functions. Integer and floating-point division/modulo are
distinct operations with documented semantics. Forming this expression is a
lowering or physical-region decision; it is not evidence that the logical graph
originally contained one composite `Map` node.

`IndexRegion` identity commits to its semantic-region reference, iteration and
reduction domains, scalar expressions, access maps, and constraints. It is the
input to scheduling rather than data independently restated by the schedule.

Index expressions should be stored in an interned arena/DAG so repeated
division, modulo, and stride arithmetic can be shared and simplified.

For a contiguous NHWC tensor:

```text
x[b,h,w,c]
  -> x_offset + b*(H*W*C) + h*(W*C) + w*C + c
```

For a runtime-strided view:

```text
x_offset + b*stride_b + h*stride_h + w*stride_w + c*stride_c
```

Logical transformations lower by reverse coordinate composition. A flattened
coordinate may be split with division/modulo; a transpose permutes coordinates;
a broadcast maps its input coordinate to zero.

### Index verifier

- Access-map rank matches the logical buffer rank.
- Every expression is integer typed.
- Divisors are nonzero statically or under a guard.
- Symbolic shape products and maximum relative offsets cannot overflow the
  selected type under declared constraints and guards.
- The compiler derives a required accessible element/byte range; runtime
  binding separately validates that the actual allocation provides it.
- Broadcast reads may alias. Physical output-store ownership is separately
  verified and cannot be inferred from logical reduction contributors.
- Every declared output produced by the compiled region is fully initialized
  according to its result contract. Narrow integration profiles may separately
  restrict execution to one out-of-place output.
- Zero-sized domains issue no accesses.
- Every runtime scalar has one ABI source.
- Every dynamic output extent, temporary size, applicability predicate, and
  launch expression is host-evaluable from declared input metadata or scalar
  ABI sources. Data-dependent output shapes and device-produced/indirect launch
  dimensions are initially unsupported.
- Bounds are proven or represented by explicit predicates and guards.

## Layer 3: scheduled iteration IR

A `ScheduledRegion` pairs one canonical `IndexRegion` with a normalized
`KernelSchedule` that maps its domains onto a target machine without introducing
new tensor semantics:

```text
ScheduledRegion {
    index_region,
    normalized_schedule,
}
```

It is a first-class, serializable, and verifiable physical representation, not
an opaque backend configuration and not merely a history of scheduling API
calls.

Representative scheduling operations:

```text
Split       FuseAxes       Reorder
BindGrid    BindThread     Vectorize
Unroll      StageLocal     ChooseReduction
```

The authoritative normalized schedule owns:

- loop, tile, and vector hierarchy;
- mappings from grid, threadgroup/block, SIMD-group/warp, lane, and vector
  coordinates into logical iteration coordinates, including bounded domains;
- intra-kernel memory placement, staging, reuse scopes, and local lifetimes;
- reduction topology, combination order, and result visibility;
- synchronization points, scopes, and convergence requirements;
- tail, predication, and padding policy;
- unrolling and software-pipeline choices;
- symbolic launch expressions and specialization constants.

All automatic/default choices are resolved before identity is formed. Two
transformation histories that produce the same normalized physical intent over
the same `IndexRegion` should have the same `ScheduledRegion` identity. A
mapping structure alone is not executable identity when paired with a different
scalar/access program.

The scheduling transformation trace is retained separately for `EXPLAIN`,
replay tests, and search provenance. A trace records stable transform names,
parameters, decisions, preconditions, and rejection reasons, but it is not the
executable truth and does not prove legality. The normalized schedule is
verified independently after transformation.

Several adjacent concepts remain deliberately separate:

- `TargetProfile` is planner input containing capabilities, hard ceilings, and
  calibrated cost-model identity.
- `TargetRequirement` is the selected implementation's machine-checkable
  capability predicate.
- `ResourceRequirements` records exact quantities or proven upper bounds used
  for feasibility, such as bindings, threads, and local-memory bytes.
- `ResourceEstimate` records quantities that cannot yet prove feasibility, such
  as register pressure, occupancy, and source/code-size estimates.
- `ApplicabilityPredicate` is a runtime-checkable condition over shapes,
  layouts, and alignment. Live-device capabilities belong to
  `TargetRequirement`.
- `CostEstimate` and its model version are search/explain metadata, never
  execution semantics.
- Boundary requirements and guarantees describe values crossing regions;
  they do not encode a region's internal thread mapping.

### Intrinsic schedule verifier

- Coordinate mappings cover the required iteration domain without missing or
  forbidden duplicate work.
- The schedule is observationally equivalent: every logical result receives
  the required value, redundant/masked work has no forbidden effects, and
  cooperative contributors combine according to the selected algorithm.
- Reads and writes are race-free or use an explicitly valid reduction/atomic
  protocol; output ownership is unique where required.
- Tail elements are guarded correctly.
- Vector access satisfies alignment and divisibility requirements.
- Barriers and collectives are convergent.
- Index ranges and coordinate maps cannot overflow under the declared guards.
- The chosen schedule preserves the declared numerical contract.

### Target feasibility assessment

`assess_feasibility(ScheduledRegion, TargetProfile)` computes exact/proven
resource requirements and target requirements or rejects the candidate. It
checks launch limits, bindings, supported operations/dtypes/collectives, local
memory, and every other target-dependent hard constraint. Estimates may guide
search and dominance but cannot prove feasibility.

Cross-kernel materialized buffers, dependencies, and lifetime intervals belong
to `KernelSubprogram` or `KernelProgram`, not an individual kernel schedule.
The schedule owns the canonical launch expression for its kernel; artifact
launch fields are checked derivations rather than a second editable authority.

## Layer 4: structured kernel IR

After scheduling, Tiler lowers into typed imperative code with lexical control
flow. It is not described as SSA if it contains mutation.

Representative constructs:

```text
BufferParameter    ScalarParameter
ImmutableValue     MutableVariable
For                If
Load               Store
Unary              Binary             Cast
Barrier            SubgroupCollective ThreadgroupCollective
```

Pointers state element type, address space, access mode, alignment, and
accessible range. The initial alias contract permits input/input aliasing but
requires a newly allocated output that aliases no input. Richer alias classes
are deferred until an optimization consumes them. Immutable values and mutable
accumulators are distinct constructs.

### Kernel verifier

- Definitions dominate uses and lexical scopes are valid.
- Only mutable variables may be assigned.
- Operation signatures agree with operand and result types.
- Buffer element types match loads and stores.
- Read-only buffers cannot be written and write-only buffers cannot be read.
- Address spaces are explicit and valid.
- Barriers and collectives satisfy convergence requirements.
- Every store is in bounds or predicated.
- Local-memory allocation and launch assumptions satisfy target constraints.

## Layer 5: artifact representation

The artifact is the versioned unit consumed by a runtime. It contains a target
payload together with semantic, implementation, and program identities, ABI,
guards, dispatch formulas, target requirements, and compiler fingerprints.

Identity is layered:

1. `IndexRegion` commits to canonical iteration/scalar/access content.
2. `ScheduledRegion` commits to its `IndexRegion` plus normalized schedule.
3. `RegionImplementation` commits to its body, boundary contracts,
   applicability predicates, target requirements, and exact/proven resource
   requirements. Cost estimates, target-profile calibration, and schedule
   traces are provenance rather than executable identity.
4. `KernelProgram` and `ProgramPortfolio` commit to the stage DAG,
   materializations, temporaries, ABI, routing, guards, and referenced
   implementation identities.

Output-affecting backend/compiler configuration and selected target identity
also participate in artifact identity. The artifact verifier checks that the
manifest agrees exactly with generated bindings and target payload. A target
profile such as Metal additionally verifies entry-point existence and binary
compatibility. See [Artifact and kernel ABI](artifact-abi.md), whose current
concrete schema is the proposed Metal profile.

## Numerical policy

Numerical behavior is part of IR meaning. At minimum the policy must address:

- integer and index overflow;
- division and modulo behavior;
- cast behavior;
- F16/BF16 accumulation;
- reduction order and determinism;
- NaN and signed-zero behavior for min/max;
- empty-reduction identities;
- fast math, reassociation, and fused multiply-add permission.

An optimization that changes reduction order is not an exact rewrite merely
because it is algebraically valid over real numbers.

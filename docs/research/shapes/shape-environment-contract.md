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

## Accepted decision: pre-dispatch host evaluability

**Accepted by Tom on 2026-07-19:** every initial output shape, temporary
allocation size, applicability guard, routing expression, and launch expression
must be evaluable on the host before any device work begins.

The allowed inputs are static constants, input tensor metadata, explicit host
shape parameters, and admitted host-visible target properties. Tensor element
data and values produced by a device dispatch are not initial extent sources.

Data-dependent shapes such as `NonZero`, `Unique`, and variable-length
selection require a future explicit shape/discovery program and two-phase
execution contract. They are rejected from the initial compilable graph with a
specific sourceability diagnostic rather than treated as an ordinary dynamic
extent. This preserves complete allocation and pipeline preflight before
partial execution and keeps fallback transactional.

## Accepted decision: zero extents

**Accepted by Tom on 2026-07-19:** zero is a valid axis extent in `ShapeEnv`.
The graph does not impose a global strictly-positive extent invariant.

```text
tensor<4 x 0 x 8 x f32>       // valid tensor shape, zero elements
Gelu(x)                       // valid; produces an empty tensor
ReduceSum(x, axis = 1)        // valid only if the op defines its empty domain
```

Each operation definition must specify its empty-domain semantics or reject an
empty domain with a semantic precondition. Physical plans must handle zero
work explicitly; they may not rely on launching a zero-sized grid. Shape and
index expressions must also avoid evaluating otherwise irrelevant division or
modulo operations whose divisor becomes zero.

The main complications are therefore operation semantics, launch legality,
and checked shape arithmetic—not a reason to reject zero-sized tensors. A
zero extent is not an unknown extent or a sentinel value.

## Accepted decision: typed constraint provenance

**Accepted by Tom on 2026-07-19:** shape predicates are strongly typed by why
they exist and what failure means. They are not stored as an undifferentiated
list of boolean expressions.

```text
SemanticRequirement(N % 4 == 0) // false => program/input is invalid
DerivedFact(A == B)              // proved by validation or inference
PhysicalGuard(C % 4 == 0)        // false => this plan is inapplicable
```

At minimum, each constraint carries its category, canonical predicate, origin
(such as an operation, input declaration, inference rule, or physical
alternative), and structured diagnostic context. Operation extensions may
define precise invalid-shape diagnostics without weakening the common
classification or requiring the compiler to parse human-readable messages.

The categories remain behaviorally distinct:

- a failed semantic requirement rejects the graph or runtime invocation;
- a derived fact is evidence available to later reasoning and must retain its
  proof/source provenance;
- a failed physical guard rejects only that physical alternative, allowing
  another plan or fallback.

An inferred fact must never silently become a new user-facing semantic
requirement, and an optimization guard must never redefine program validity.
Explanation output reports both the predicate and its provenance.

## Accepted decision: closed shape-expression language

**Accepted by Tom on 2026-07-19:** the initial `ShapeEnv` uses a closed,
canonical shape-expression language. Extensions construct expressions from
the admitted primitives; they cannot inject arbitrary Rust callbacks, opaque
functions, or custom evaluation behavior.

This strict boundary is part of the prototype contract. It ensures the same
expression can be validated, normalized, compared, reasoned about, serialized,
hashed, evaluated on the host, and rendered in explanation output without
extension-specific code. New primitives may be added later only when a concrete
operation cannot be represented cleanly and the primitive receives defined
typing, overflow, canonicalization, serialization, and evaluation semantics.

The exact initial primitive set remains to be decided. Candidate needs include
constants and roots; checked arithmetic; exact, floor, and ceiling division;
remainder and divisibility; comparisons; min/max; and a typed conditional.

## Accepted decision: signed intermediate arithmetic

**Accepted by Tom on 2026-07-19:** shape expressions may contain checked signed
intermediate values even though a tensor extent is always nonnegative.

```text
conv_output = floor_div(N + 2P - D * (K - 1) - 1, S) + 1
```

This permits tensor formulas such as convolution, cropping, padding, and
slicing to retain their natural mathematical structure instead of requiring
unsigned-only rewrites. Signed division and rounding behavior must be explicit
for each relevant primitive.

Every expression exported as a tensor extent must evaluate to a nonnegative,
representable extent. Checked-arithmetic overflow, division by zero, or a
negative final extent is a typed shape-evaluation failure with constraint and
origin context. The concrete bounded integer representation and overflow
width remain a separate decision.

## Accepted decision: explicit division modes

**Accepted by Tom on 2026-07-19:** the shape language has semantically distinct
exact, floor, and ceiling division operations. It has no ambiguous generic
division operation.

```text
ExactDiv(12, 4) = 3
ExactDiv(10, 4) = error
FloorDiv(-3, 2) = -2
CeilDiv(-3, 2) = -1
```

All modes reject a zero divisor. `ExactDiv` additionally requires zero
remainder. `FloorDiv` rounds toward negative infinity and `CeilDiv` toward
positive infinity, including for negative operands. These distinctions are
part of expression semantics, canonical identity, diagnostics, and
serialization.

The operations may share implementation modules or internal helpers. A future
unified representation is compatible only if it retains the division mode
explicitly and preserves these observable semantics; public constructors and
serialized formats would require ordinary compatibility/versioning treatment.

## Provisional decision: specialization boundary

**Accepted provisionally by Tom on 2026-07-19:** runtime extents remain symbolic
in the logical plan by default. Specializing an extent to a concrete value is a
physical-planning decision.

```text
logical extent: batch

physical alternatives:
  batch == 1       -> shape-specialized kernel
  batch % 8 == 0   -> guarded vectorized kernel
  otherwise        -> generic plan or fallback
```

This preserves one logical computation across shape profiles and allows the
physical optimizer to build a bounded, costed portfolio. It does not prevent
constant folding or specialization: a frontend-declared static extent is
already constant, and a physical alternative may introduce an explicit guard
that makes a symbolic extent constant within that alternative.

Specializing in the logical plan would expose constants earlier and can
simplify shape-specific reasoning, but it discards generality, splits logical
identity by shape, and risks compile-time and artifact proliferation. The
symbolic-first direction is therefore the prototype default, not yet a durable
policy: experiments must determine which specializations pay for their guard,
compile-time, and artifact-size costs and how the portfolio is bounded.

## Accepted decision: scoped symbol identity

**Accepted by Tom on 2026-07-19:** frontend identifiers such as `M`, `K`, and
`N` use ordinary lexical binding. A frontend resolves them to scoped internal
symbol IDs; the IDs, not diagnostic strings, establish equality in the
semantic IR.

```text
graph_a: [B, 128]  -> [symbol_a, 128]
graph_b: [B, 128]  -> [symbol_b, 128]
```

Composing these graphs keeps `symbol_a` and `symbol_b` independent unless the
composition explicitly connects them or adds an equality constraint. Within
one scope, repeated use of `B` naturally resolves to the same ID and asserts
equality.

This makes graph composition capture-avoiding and permits diagnostic renaming
without changing computation identity. Serialization and cloning must preserve
or consistently remap IDs. Explanation output should retain optional readable
names and disambiguate collisions when necessary.

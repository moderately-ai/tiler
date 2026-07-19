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

## Accepted decision: closed solver exchange and durable derived facts

**Accepted by Tom on 2026-07-19:** the closed expression and predicate
vocabulary is the public language exchanged with the constraint solver, not
merely a format produced by it. Operations contribute typed expressions,
semantic requirements, and known facts in that language. The solver returns
typed proof outcomes, contradictions, residual host checks, structured
`Unknown` reasons, and any derived facts that it can establish. It does not
expose opaque callbacks or solver-specific formulas to the optimizer.

Derived facts become durable, canonical members of the `ShapeEnv`. Each keeps
its proof/source provenance and is deduplicated by canonical identity, so
later validation and optimization can reuse it and explanation output can
reconstruct why a decision was legal. This does not require incidental solver
state, search traces, or memoization caches to become part of the semantic
program or its content identity.

Solver implementations may use richer private representations internally,
but optimizer and artifact layers consume only the stable typed result
contract. Expression arity is an implementation property rather than the
public semantic classification: for example, `Not` is unary, `Equal` is
binary, `Select` is ternary, and canonical `Add`, `All`, or `Any` may be
n-ary.

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

**Accepted by Tom on 2026-07-19:** shape expressions may contain signed
intermediate values even though a tensor extent is always nonnegative.

```text
conv_output = floor_div(N + 2P - D * (K - 1) - 1, S) + 1
```

This permits tensor formulas such as convolution, cropping, padding, and
slicing to retain their natural mathematical structure instead of requiring
unsigned-only rewrites. Signed division and rounding behavior must be explicit
for each relevant primitive.

Every expression exported as a tensor extent must evaluate to a nonnegative,
representable extent. Division by zero or a negative final extent is a typed
shape-evaluation failure with constraint and origin context. The arithmetic
domain and bounded conversion contract are defined below.

## Accepted decision: mathematical shape arithmetic

**Accepted by Tom on 2026-07-19:** semantic `ShapeExpr` arithmetic has
mathematical-integer semantics. It does not wrap, saturate, or expose overflow
from an arbitrary compiler intermediate width.

```text
ExactDiv(A * B, B) == A  // when B != 0, even if A*B exceeds u64/i128
```

This makes algebraic equivalence independent of an implementation integer
width. Conversion of a final nonnegative result to `Extent(u64)` is explicit
and checked; an unrepresentable result is rejected before allocation or device
work. Physical and ABI expressions may instead define bounded-width arithmetic
because machine representation is part of those domains' contracts.

The exact evaluator and prover use deterministic expression-size,
integer-magnitude, and reasoning budgets. Exhausting a budget produces a typed
resource-limit diagnostic or `Unknown(ResourceLimit)`, as appropriate; it
never produces an approximate, wrapped, or saturated value. Concrete evaluator
strategy and arbitrary-precision representation are implementation choices.

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

## Accepted decision: extent width and domain newtypes

**Accepted by Tom on 2026-07-19:** root extent parameters and final tensor
extents have a portable `u64` value domain. Rust `usize` is not part of the
semantic shape contract. Signed shape intermediates use the mathematical
semantics above rather than exposing a bounded primitive representation.

Index width is a physical-plan decision. A `u32` indexing alternative is legal
only when proofs or pre-dispatch guards cover every relevant extent, product,
stride, and maximum reachable storage offset; otherwise the planner must use a
wider legal alternative or fall back.

These domains use semantic newtypes rather than exposing interchangeable
primitive integers. At minimum, extent values, signed shape intermediates,
symbol IDs, axis indices, input indices, host shape-parameter indices, and
physical index widths must not be accidentally mixed merely because their
representations are integer types. Conversion across domains is explicit and
checked. This is a correctness and readability invariant, not only an API-style
preference.

## Accepted decision: three-outcome semantic validation

**Accepted by Tom on 2026-07-19:** validation classifies every semantic shape
requirement as proved, disproved, or unresolved under the facts available at
compile time.

```text
MatMul([M, K1], [K2, N]) requires K1 == K2

proved      -> accept and retain the proof as a derived fact
disproved   -> reject during compilation with a typed diagnostic
unresolved  -> emit a typed host-side pre-dispatch requirement
```

At invocation time an unresolved semantic requirement is evaluated after root
extent binding but before allocation or device work. Failure means the runtime
input is invalid for the logical program. It must not be reclassified as a
physical guard, select another kernel as though semantics were still valid, or
permit fallback to execute an equally invalid operation.

The artifact/interface contract includes the remaining runtime requirements
and enough provenance to produce the same structured diagnostic category as
compile-time validation. This permits independently supplied dynamic inputs
without requiring every valid relationship to be statically provable.

## Research finding: proof scope is not expression scope

The closed shape-expression language guarantees deterministic representation
and concrete evaluation, but not complete static proof. Affine/Presburger
integer constraints provide a substantial decidable core; products and
division between symbolic values introduce nonlinear obligations required by
ordinary tensor operations such as reshape.

The detailed precedent, tractability boundary, initial-versus-later capability
matrix, and proposed staged proof architecture are recorded in
[Shape constraint prover boundary](constraint-prover-boundary.md).

**Accepted by Tom on 2026-07-19:** the minimally viable prototype includes a
working native prover for canonical equality, substitution, intervals,
constants, and common divisibility consequences. It must produce evidence and
structured `Unknown` reasons and exercise the validation/guard paths; a
prover-shaped interface that returns `Unknown` universally is insufficient.
Fuller Presburger and nonlinear capabilities remain measurement-driven
extensions behind the same proof contract.

## Accepted decision: domain-specific expression IRs over shared components

**Accepted by Tom on 2026-07-19:** `ShapeExpr` and runtime/artifact `AbiExpr`
are distinct, newtyped domain IRs. They do not become one universal expression
language merely because their implementations need many of the same arithmetic
operations.

```text
ShapeExpr sources: extent symbols, input dimensions, shape parameters
AbiExpr sources:   lowered extents, strides, buffer sizes, target properties
```

The implementations should be composed from atomic shared components for
checked arithmetic, division modes, evaluation, canonicalization,
serialization support, and testing where their semantics truly coincide. Each
domain still defines its own admitted sources, result types, validation rules,
identity, and versioning. Lowering from `ShapeExpr` to `AbiExpr` is explicit,
typed, and checked.

Code organization is separate from IR architecture: shared Rust modules or
generic internals do not imply shared semantic identity, and distinct IR types
do not require duplicated code.

## Accepted decision: typed lazy shape selection

**Accepted by Tom on 2026-07-19:** the initial shape language includes a typed
conditional expression whose condition is a host-evaluable shape predicate and
whose branches have the same shape-expression type.

```text
Select(A == 1, B, A)                    // dynamic broadcast extent
Select(N == 0, 0, CeilDiv(N, chunk))    // zero-aware shape formula
```

`Select` is shape-metadata computation. It is not tensor `where`, general
logical-graph control flow, or a device branch. Concrete evaluation is lazy:
only the selected branch is evaluated, so an invalid operation in an
unselected branch does not fail evaluation.

Canonicalization eliminates a statically selected branch. The prover may split
on an unresolved condition only within a deterministic case budget; otherwise
it returns a structured `Unknown(ResourceLimit)`. This represents common
piecewise shape semantics without requiring specialized primitives for every
operation or separate logical graphs for each metadata case.

## Accepted decision: specialized min and max expressions

**Accepted by Tom on 2026-07-19:** `Min` and `Max` are explicit shape-expression
primitives even though either can be represented using `Select` and a
comparison.

```text
Min(A, B)  // not canonicalized into Select(A <= B, A, B)
Max(A, B)  // not canonicalized into Select(A >= B, A, B)
```

The specialized nodes preserve common mathematical intent, provide canonical
forms for padding/slicing/clamping formulas, and allow bounds reasoning without
introducing an artificial proof case split. Their evaluator or lowering may
reuse the same atomic implementation components as `Select`; that code reuse
does not erase their distinct IR identity.

## Accepted decision: typed Boolean shape predicates

**Accepted by Tom on 2026-07-19:** shape constraints and guards use a closed,
typed, extensible `ShapePredicate` language rather than only an implicit
conjunction of comparisons.

```text
Compare(...)
All([...])
Any([...])
Not(...)
```

This permits alternative-validity rules such as dynamic broadcasting:

```text
Any([A == B, A == 1, B == 1])
```

`All` and `Any` use short-circuit concrete evaluation and canonicalize by
flattening nested instances, removing identities and duplicates, and assigning
a deterministic operand order. `Not` is typed and normalized where doing so
does not cause uncontrolled expansion. Proving disjunctions or negations may
case-split only within deterministic budgets and otherwise returns structured
`Unknown`.

The enclosing typed constraint retains operation-specific provenance and error
context, so a generic predicate can still report a specialized diagnostic such
as incompatible broadcast dimensions. New predicate primitives require the
same versioned semantic treatment as new shape-expression primitives; arbitrary
callbacks remain excluded.

## Accepted decision: binding-kind capabilities

**Accepted by Tom on 2026-07-19:** `ShapeEnv` is generic over the admitted root
binding kinds already defined by this contract, while each operation or
semantic factor position explicitly declares which binding classes it
supports.

```text
global root kinds:
  Static | InputDimension | ShapeParameter

example operation capability:
  split factor: StaticOnly

later operation capability:
  split factor: HostEvaluable
```

The common IR can therefore represent a runtime semantic factor such as
`N == A * B` when `A` and `B` are host-sourceable, without requiring every
initial operation implementation to support every binding kind. Validation
checks an expression's transitive sources against the declared capability and
rejects unsupported combinations with a typed explanation naming the
operation, factor position, observed binding kinds, and supported set.

This separates architectural expressiveness from vertical operation support.
Adding runtime support to a factor can extend an operation capability without
inventing a new shape representation; it still requires the operation's
validation, lowering, ABI, and tests to support that case.

## Accepted decision: explicit divisibility predicate

**Accepted by Tom on 2026-07-19:** the shape language provides both a numeric
`Remainder` expression and a specialized `Divisible` predicate.

```text
Remainder(N, 8)  // produces a numeric value
Divisible(N, 8)  // states a proof/validation condition
```

Canonicalization maps `Remainder(x, d) == 0` to `Divisible(x, d)` so equivalent
spellings have one canonical identity. The specialized predicate preserves
intent for proof evidence and produces direct diagnostics such as "extent N
must be divisible by 8." It also names the precondition required by
`ExactDiv`.

Concrete evaluation supports a well-typed nonzero symbolic divisor. Constant
divisors belong to the initial prover's supported congruence fragment; a
symbolic divisor crosses the affine boundary and may produce a structured
`Unknown` during static proof. A zero divisor is a typed evaluation or
statically detected construction/validation error, as applicable.

## Accepted decision: inferred extents require a unique solution

**Accepted by Tom on 2026-07-19:** extent inference succeeds only when the
available semantic constraints determine exactly one nonnegative extent.
Underdetermined or ambiguous inference is rejected with a typed diagnostic;
the solver must not choose an arbitrary conventional value.

```text
input shape [0], requested [inferred]    -> uniquely inferred as [0]
input shape [0], requested [0, inferred] -> ambiguous because 0 == 0 * x
input shape [0], requested [0, 7]        -> explicit and valid
```

An inference sentinel is frontend or construction syntax, not a semantic
extent value in the logical IR. A frontend may deliberately implement a
documented convention for an ambiguous source language, but it must resolve
that convention to an explicit extent before producing Tiler's semantic
graph. The selected value and its frontend provenance therefore become
visible to validation, hashing, and explanation rather than being hidden
inside the constraint solver.

## Accepted decision: shape is upstream of access, not physical planning

**Accepted by Tom on 2026-07-19:** under Tiler's pure tensor-value semantics,
`ShapeEnv` describes logical rank and extents independently of storage layout.
Iteration and access representations may reference shape expressions and
proved shape facts; storage layout must not redefine logical tensor meaning or
make an otherwise valid logical computation invalid.

This is an architectural dependency direction, not a claim that physical
planning is a one-way pipeline. Producer and consumer layout requirements and
guarantees may propagate in both directions while the physical planner jointly
chooses fusion, schedules, layouts, views, and materializations. A conflict
rejects a physical alternative or introduces a conversion/materialization; it
does not change semantic shape facts.

Runtime tensor descriptors bind related but distinct facts: dimensions bind
logical extents, while strides, offsets, allocation ranges, and alignment bind
access/layout properties. Access validity is checked using the dimensions,
but allocation metadata is not used to invent or revise logical extents.

This initial separation must not prevent richer future contracts. Sparse or
encoded storage, returned views, required aliasing, and mutation/in-place
behavior may make representation observable. Such features require explicit
encoding, alias, or effect semantics and corresponding physical interfaces;
they do not enter the system by silently turning layout observations into
ordinary `ShapeEnv` facts.

## Accepted decision: guaranteed bounds are not profile hints

**Accepted by Tom on 2026-07-19:** a guaranteed shape bound and a statistical
optimization profile are different contracts even when they contain similar
numbers.

```text
SemanticRequirement(1 <= N && N <= 4096) // violation is invalid input
ProfileHint(N usually lies in 128..=512)  // cost guidance only
```

Guaranteed bounds are typed `ShapeEnv` constraints with provenance. The
solver may use them to prove semantic legality, arithmetic safety, and the
applicability of a physical alternative. Profile hints belong to a separate
optimization-profile domain: they may influence estimated frequency, expected
cost, portfolio ordering, or specialization choices, but they must never
prove a requirement, discharge a guard, or remove a correct general path.

This separation leaves room for later histograms, observed distributions,
feedback-directed profiles, and confidence or freshness metadata without
changing logical program validity or contaminating canonical semantic facts.

## Deferred decision: construction and commitment lifecycle

The discussion established that local, environment-relative, and graph-wide
invariants can all be maintained incrementally; "whole graph" describes the
scope of an invariant, not necessarily a nonincremental validation algorithm.
It also established requirements for multiple frontend/DX surfaces, reusable
intermediate representations, consumer-side caching, and a stable point at
which a graph may be treated as a complete program.

No mandatory `GraphBuilder`, `.build()`, `.seal()`, immutable snapshot type, or
canonicalization transition is accepted yet. Those are possible
implementations of the requirements, but choosing among them is deferred until
the remaining semantic shape contracts are settled. "Sealing" may be used as
informal terminology for a completeness/commitment boundary without implying
a particular public API or Rust type-state design.

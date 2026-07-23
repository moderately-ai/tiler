---
schema: "tiler-doc/v1"
id: "tiler.contract.ir"
kind: "contract"
title: "IR stack and invariants"
topics: ["ir", "semantics", "scheduling"]
contract_status: "mixed"
implementation_status: "partial"
evidence: ["tiler.research.semantic-graph.contract-memo", "tiler.research.semantic-graph.rust-construction-lifecycle", "tiler.research.indexing.index-access-model", "tiler.research.scheduling.scheduled-region-model", "tiler.research.kernel-ir.structured-kernel-ir-verifier", "tiler.research.shapes.nightly-const-shape-parameters"]
---

# IR stack and invariants

**Status:** mixed — accepted representation boundaries and proposed schemas

Accepted ADRs govern the representation separation and semantic invariants they
name. Unless a section says otherwise, concrete field sets, enum variants, and
API shapes below remain proposed.

## Ownership boundary

This document owns field-level semantic, index/access, schedule,
`KernelProgram`, `BufferPlan`, `AbiExpr`, target-requirement, and
structured-kernel models, including their canonical identity and verifiers.
The IR boundary also owns the authoritative pure checked `AbiExpr` evaluation
semantics. Optimizer documents own how candidates are found and ranked; the
artifact contract owns versioned encoding, runtime fact binding, compatibility,
failure classification, and the serialized envelope.

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
6. Narrowing conversions have explicit typed behavior and are represented
   explicitly; rejecting families are proven or runtime-validated.
7. Runtime assumptions appear as guards, not comments.
8. Malformed IR is rejected before source generation.
9. Passes state the numerical equivalence relation they preserve.
10. Floating constants have a defined bit-level equality and hash policy.
11. Source origins survive lowering sufficiently for diagnostics and `EXPLAIN`.
12. Artifact identity uses canonical content, never allocation identity.

## Shared IR construction lifecycle

ADR 0070 assigns the experimental target-neutral layers to public modules in
`tiler-ir`: `index`, `schedule`, `kernel`, and `program`. Compiler-owned region
candidates, search alternatives, costs, and explain records are not shared IR
merely because they refer to semantic operations.

ADR 0071 establishes one construction lifecycle for the shared layers:

```text
LayerBuilder -- build(self) --> VerifiedLayer
             -- failure -----> { builder, typed diagnostics }
```

Builders own private mutable storage and perform local admission checks.
Whole-object verification occurs at consuming build. Verified products are
immutable and expose read-only meaning rather than arena storage. Backends and
artifact codecs accept only the verified wrappers; artifact decoding rebuilds
through the same checked path. Layer-specific opaque `u32` newtypes live with
their domains and cannot be forged from public numeric constructors.

The implementation order is index region, scheduled region, structured
kernel, kernel program, then portfolio. A public module declaration or a
private proof struct is not implemented support for that layer.

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

ADRs 0005 and 0006 accept the public graph/extension boundary and the concrete
operation/value model below.

```text
ProgramInput {
    key: ProgramInputKey,
    display_name?,
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
    key: ProgramOutputKey,
    display_name?,
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
`ProgramInputKey` and `ProgramOutputKey` are stable newtyped interface keys and
participate in semantic identity together with ordered position, referenced
value, type/constraints, and result contract. They are not diagnostic names. A
frontend that does not expose authored keys deterministically assigns
`input/<ordinal>` and `output/<ordinal>`. Optional display names and source spans
do not participate in identity and may change without invalidating a program.
Two interface entries cannot share a key even when two outputs intentionally
reference the same value.

### Accepted Rust construction and ownership boundary

ADRs 0058 and 0059 fix the public lifecycle and typed authoring boundary without
making storage layout public. The
conceptual namespaces are `tiler_ir::shape` and `tiler_ir::semantic`; reference
execution is a downstream `tiler-reference` consumer under ADR 0065. Internal
files are organized beneath their semantic concepts rather than exposed as a
generic collection of newtypes. `Axis`, `Extent`, and
`Shape` belong to the shape vocabulary. `ValueId`, `OperationId`, input keys,
and output keys belong to the semantic graph. Physical schedules and kernel IR
must define different handles even if they use the same integer representation.

`SemanticProgramBuilder` is append-only and non-`Clone`. Fallible insertions
are transactional: validation and capacity checks occur before mutation, and
an error leaves the draft unchanged. Borrowed `validate(&self)` supports
diagnostics and tooling but does not turn a draft into compiler input. It runs
the same structural checks and reachable semantic-authority projection used by
commitment, reporting authority failures as typed diagnostics with the
underlying registry error preserved as an error source. The commitment boundary
is:

```text
build(self) -> Result<SemanticProgram, ProgramBuildError>
```

`build` runs that combined validation/projection pass once, retains its checked
identity subjects, and consumes the arenas without cloning the draft. Under
ADR 0064 it compacts the output-reachable
closure into dense completed-program storage and assigns a new graph-owner
identity; draft handles do not survive successful commitment. A failed build
returns structured diagnostics together with ownership of the original builder. The caller may
inspect it through borrowed accessors, use `into_builder` or `into_parts` to
recover ownership, correct it, and retry without reconstructing the graph.

Commitment computes the deterministic old-to-new mapping needed to rewrite
live edges, interfaces, constraints, witnesses, and provenance, but ordinary
`build` need not retain or expose it. Declared results cross the boundary
through typed stable interface selectors such as conceptual `Output<T>`, which
resolve to new completed-program-owned handles after validating the output key
or position and resolved value type. A future additive `build_with_report` may
expose governed retained/rewritten/coalesced/removed correlation without
changing ordinary `build`; draft arena indices never become durable identity.

`SemanticProgram` is immutable and cheaply cloneable through private
`Arc<ProgramData>` storage. Compiler, optimizer, and evaluator entry points
borrow `&SemanticProgram`. A shared lazy cache may memoize canonical identity,
using `OnceLock` across every clone. The `Arc`, owner token, arena numbering,
and insertion history do not enter that identity.

The primary Rust authoring capability is an exact nominal `Value<T>`, where `T`
denotes the complete semantic tensor type rather than a coarse numerical family.
The canonical heterogeneous graph stores an authoritative complete
`ResolvedValueType`, shape, and definition under an opaque graph-owned
`ValueId`; it does not store Rust `T`, `TypeId`, or type names. The resolved
type may be nominal, parameterized, or an encoded-numeric scheme contract under
ADR 0062. `ValueId` means the type is unknown at the
current Rust call site, not that the value may be used as any type. It grants
identity and lookup only. `ValueRef` exposes the authoritative runtime metadata.

Only the owning builder or program constructs `Value<T>` after checking an exact
resolved-value-type match. There is no `AnyValue`, unchecked public constructor, implicit
retyping, or unvalidated general insertion API. Erasure to `ValueId` is explicit
and checked reification is fallible. All handles have no cross-graph validity,
serialization contract, or durable semantic identity. Public operations reject
foreign handles. Internal edges store private compact typed `u32` indices so
the ownership guard does not inflate every edge.

Under ADR 0063, graph ownership is an opaque runtime-checked safety property,
not a mandatory Rust lifetime or generative brand. Every handle-consuming
public API verifies exact ownership before indexing storage or mutating a draft.
Foreign values, refined values, and witnesses produce a typed argument-specific
error and leave an append-only builder unchanged. Owner tokens never enter
durable identity or internal verified edges, and exhaustion cannot alias a live
graph.

Under ADR 0060, a Rust marker does not declare or own its semantic key. The
explicit frozen registry binds one local `'static` marker to one complete
registered `ResolvedValueType`; duplicate marker or resolved-identity bindings
fail before construction.
Only a builder/program using that frozen binding may create or checked-reify the
corresponding `Value<T>`. A process-local `TypeId<T>` may implement lookup but
never enters semantic or artifact identity.

The implemented ownership boundary distinguishes semantic authority from later
compiler capabilities. `tiler-ir` owns an immutable, cheap-clone
`FrozenSemanticRegistry` containing portable type definitions, provider
provenance, and process-local marker bindings. Semantic builders and completed
programs own that snapshot rather than borrowing a context. Registration begins
from an empty or mutable standard `SemanticRegistryBuilder`, applies built-in
and statically linked external providers transactionally, validates referenced
type closure, and consumes the builder at freeze. Provider callbacks are not
retained. Optimizer, evaluator, scheduler, and backend capabilities belong to
later layer-specific registries; a higher-level compilation session may compose
them without making `SemanticProgram` own executable provider machinery.

Registration is fail-sticky within each provider batch: an ignored duplicate or
partial marked-registration error still prevents the entire batch from
committing. Registry counts and aggregate canonical bytes are checked before
retention, and freeze diagnostics are deterministic. The frozen snapshot offers
only borrowed definition lookup and canonical-key-order iteration. Definitions,
schemas, and bounded arities are read-only; executable validators and
inferencers do not become mutable public authority.

Frozen-registry snapshot identity includes the sorted semantic definitions and
stable provider revisions but excludes marker `TypeId`s and Rust names.
`SemanticGraphIdentity` includes the complete resolved type of every retained
value and the canonical operation/value graph and interface, but no provider
revision or unrelated registry entry. Reached provider-independent definitions
have a separate `SemanticDefinitionProjectionIdentity`; the providers whose
mandatory capabilities admitted those definitions have a separate
`SemanticAdmissionProvenanceIdentity`. `SemanticRegistrySnapshotIdentity`
identifies the complete frozen authority environment.

Incremental program construction has one private aggregate canonical-work byte
budget. Each input, operation and inferred result set, and named output is
charged before any arena mutation; rejection reports the first aggregate value
and active limit without poisoning the builder. This is a conservative work
budget over the staged draft, so dead values remain charged until commitment.
It is not exact heap accounting. Commitment compacts the reachable graph,
computes its exact canonical encoded length without constructing per-operation
byte buffers, and records that proven length. Lazy identity construction checks
the proof before making one exactly sized final allocation.

`SemanticProgram::semantic_identity()` returns one borrowed, non-forgeable
`SemanticIdentity` owner for all four subjects:

```rust,ignore
let identity = program.semantic_identity();
let graph = identity.graph();
let definitions = identity.reached_definitions();
let admission = identity.admission_provenance();
let snapshot = identity.registry_snapshot();
```

The bundle has private fields and no public constructor. Individual subject
newtypes remain public so consumers can inspect or compare the exact equality
they need, but compiler requests, target requests, and artifact-construction
plans retain the bundle atomically. This prevents component-wise assembly from
different programs while preserving the distinctions owned by ADR 0072.

The immutable `SemanticProgram` computes and owns both reached subjects during
checked build. Their authority closure starts from every retained value type,
operation key, and occurrence attribute value, then transitively follows
parameterized and encoded type components, canonical `Type` and `FloatBits`
references, type-definition facts, and operation defaults, facts, and
conformance requirements. Closure is deterministic, iterative, cycle-safe,
and governed by separate bounds for roots consumed and unique authority
subjects discovered. Both are enforced while ingesting or enqueuing, before an
unbounded worklist can form. Their numeric limits are private implementation
policy rather than public API constants; typed resource errors still report the
resource, active limit, and first rejected count. A caller cannot manufacture
program-complete evidence by supplying an incomplete root list to a registry
projection API; consumers obtain the authoritative bundle from the completed
program.

ADR 0061 adds optional, checked Rust-side shape evidence without making it
canonical graph authority. Conceptually, `ShapedValue<T, E>` refines a
`Value<T>` with evidence such as fixed rank or an exact static shape. Only the
owning builder or completed program may construct it after checking `E` against
the value's authoritative ranked shape-expression vector and `ShapeEnv`.
Absence of such evidence means only that the Rust caller does not possess it;
the semantic value never becomes unranked.

ADR 0067 fixes the initial exact-static evidence spelling to one dependent
array family on the governed nightly:

```rust,ignore
pub struct StaticShape<
    const RANK: usize,
    const EXTENTS: [u64; RANK],
>;

type Matrix = ShapedValue<F32, StaticShape<2, { [2, 3] }>>;
```

`RANK` is `usize` only because Rust array lengths require it; each extent is
`u64` and is checked into Tiler's canonical extent newtype at refinement. This
is one arbitrary-rank family, not a finite `StaticShapeN` vocabulary. The
explicit type is canonical; a future type-position macro may abbreviate it but
cannot define a second evidence identity.

Weakening a refined handle to `Value<T>` is explicit and zero-cost. Refinement
is checked and fallible unless the producing operation established the evidence
directly. User-implemented marker traits cannot forge evidence, and Rust shape
markers, const parameters, names, and `TypeId` values never enter durable
identity. Multi-value solver proofs use graph-owned typed witnesses such as a
same-shape or broadcast-compatibility witness rather than an untyped boolean.

Refined and unrefined authoring APIs share one semantic admission path. Shape
evidence may improve arguments, results, and diagnostics, but it neither owns a
second shape inference system nor directs physical specialization. An operation
propagates evidence only for a relationship that it can establish
unambiguously and revalidate against the canonical result shape. The initial
surface remains builder-centered; an independent fluent shaped-value API is
reserved until completeness and nonduplication are demonstrated.

There is no implicit snapshot, builder `Clone`, mutable thaw, or hidden
copy-on-write arena. Adding unfinished-graph branching requires a separately
reviewed `snapshot` or `fork` contract backed by measurements. Completed
immutable programs already branch cheaply.

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
Input interface keys participate in identity; optional display names do not.

Canonical operation attributes use this bounded host-owned data model:

```text
CanonicalValue =
    Bool
  | SignedInt { width: 8 | 16 | 32 | 64, bits }
  | UnsignedInt { width: 8 | 16 | 32 | 64, bits }
  | FloatBits { format: TypeKey, bits }
  | Bytes
  | Utf8String
  | Type(ResolvedValueType)
  | Sequence([CanonicalValue])
  | Record([(AttributeFieldId, CanonicalValue)])
```

`AttributeFieldId(u32)` is stable within one versioned operation attribute
schema. Record fields are sorted by ID and unique; sequence order is semantic.
Strings are exact valid UTF-8 bytes with no implicit Unicode normalization.
Integers never use host `usize`/`isize`, and floats are raw governed-format bits
so signed zero and NaN payloads are not host-normalized. Recursion, bytes,
items, string length, and collection sizes are checked against host limits.

The schema validates and normalizes attributes before storing or hashing them:
a field equal to its declared default has one canonical representation, which
is omission unless the schema marks presence itself semantic. It resolves the
default again only for checked inference. Unknown fields are rejected in the
initial lockstep schema. The v1 identity encoder uses explicit one-byte kind and
integer-width tags, big-endian integer payloads, big-endian `u64` byte/item
lengths, big-endian `u32` field IDs, and exact payload bytes; records use sorted
field IDs. This identity encoding is Tiler-owned and is not ordinary provider
serialization or the still-unselected public artifact codec.

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

Built-in and extension nominal element types share one durable identity model,
and ADR 0062 composes them into the larger tagged `ResolvedValueType` domain.
Conceptually, a nominal type key contains a namespace, name, and semantic version:
`tiler::f32@1` and `acme::fp8_special@1` differ by identity even if some
structural facts coincide. Built-ins may have ergonomic Rust spellings such as
`DType::F32`, but canonical hashing, serialization, registry lookup, and
capability diagnostics use the durable key rather than a Rust enum
discriminant, `TypeId`, or address. A canonical type descriptor supplies the
format's structural and value-semantic facts; those facts do not replace its
nominal identity.

Formats admitted into Tiler's built-in vocabulary use Tiler-governed keys whose
immutable descriptors carry mandatory normative-definition references. A new
standards document revision does not automatically change type identity:
semantically identical revisions may add provenance, while an incompatible
meaning requires a new key version. Already-published external project/vendor
identities remain external when Tiler adds support and are never silently
rekeyed.

The built-in recognized integer catalog includes two-valued `bool` and the
signed and unsigned widths 2, 4, 8, 16, 32, and 64. Recognition of `i2`, `u2`,
`i4`, and `u4` does not imply unpacked storage or general arithmetic support;
packing, literals, operations, evaluation, and target lowering remain explicit
capabilities. Quantized tensors are not identified by their integer storage
type alone: scale, zero point, axis/block structure, expressed type, and their
operation semantics require an explicit quantization contract.

Affine quantization maps each data coordinate to a coordinate in its scale and
optional zero-point parameter tensors through a bounded canonical parameter
index map. Per-tensor, per-axis, and per-block quantization are built-in forms
of that mapping. The mapping is semantic and shape-verified; physical packing
and addressing remain storage decisions. Representability and verification of
a mapping do not imply reference, optimizer, or backend support for it.

A quantized tensor is one first-class semantic tensor value even when its
runtime representation has several components. Its static type contract names
the versioned scheme, code and expressed dtypes, component roles, coordinate
maps, and resolved numerical behavior. Concrete code, scale, zero-point,
codebook, or other component tensors are ordered operands to a dedicated
assembly or conversion operation; graph-local value handles and parameter
payloads never enter the static type.

`AssembleQuantized` associates existing components without numeric conversion.
`Quantize`, `Dequantize`, and `Requantize` are observable numeric conversions.
Component extraction is explicit, and physical packing remains a later storage
decision. Canonical program identity includes the producing operation and its
canonical operand identities, not incidental arena IDs. Artifact lowering may
expand one logical quantized argument or result into several verified physical
bindings.

The graph initially contains atomic named tensor operations. Representative
built-ins include:

```text
Constant    Cast           Reindex     Broadcast
FloatAdd    WrappingAdd    CheckedAdd  Multiply
SaturatingAdd              WideningAdd Gelu
Reduce
```

Program inputs are declarations rather than operation invocations. The
operation list is illustrative rather than a closed Rust enum. In particular,
an admitted `Gelu` key pins its exact formula or decomposition and every
subordinate transcendental contract; erf-GELU and a tanh approximation are not
interchangeable implementations of an unspecified node. An operation
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

One narrow admission is operation semantics rather than an implicit graph rule.
A binary elementwise signature may declare that it accepts a rank-zero operand,
in which case that operand contributes its single value at every output
coordinate and the result takes the other operand's shape. The built-in
`tiler::add-f32@1` and `tiler::multiply-f32@1` signatures declare exactly that
admission. [ADR 0061](decisions/0061-layer-checked-shape-evidence-over-values.md)
accepts the `F32Add` and `F32Multiply` authoring facades over it and names their
scalar broadcast, while this document owns the admission itself. A declaring
signature checks the rule in its own inference and states it in its normative
definition. No node is synthesized: canonical identity records the binary
operation and its two operand identities, never an implicit `Broadcast`.

Nothing else broadcasts implicitly. Operands of nonzero rank must agree in
shape, and rank padding, extent-one stretching, and every other many-to-one
mapping still require an explicit `Broadcast` with an axis mapping, in every
signature and at every rank. A signature that does not declare scalar admission
rejects a rank-zero operand exactly as it rejects any other shape disagreement.
The admission is a shape rule alone: a declaring signature still requires
matching resolved operand value types, so it grants no promotion, weak-scalar,
or other dtype permission.

### Proposed public experimental operation extension contract

Built-in and third-party operations use the same public experimental operation
definition path. Durable IR stores an `OpKey`, canonical attributes, operands,
and results; it never serializes Rust trait objects or registry addresses. A
registry resolves `OpKey` to versioned operation capabilities.

The mandatory semantic definition supplies:

- operand/result schema and arity;
- shape, dtype, axis, and semantic-constraint inference and verification;
- canonical attribute encoding and deterministic identity;
- purity/effect declaration;
- normative semantic specification identity and conformance behavior;
- for transcendental operations, immutable reference semantics, admitted
  accuracy envelopes/domains, independent special-value policies, and
  reference-evaluator support;
- stable host-readable names and documentation for explain output.

Optional capabilities may provide:

- executable reference evaluation;
- decomposition into other semantic operations;
- canonicalization and contract-preserving rewrite rules;
- iteration-domain and access-map lowering;
- region-fusion participation;
- physical implementations, boundary requirements/guarantees, and costing;
- accuracy realizations and scoped conformance evidence;
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

Semantic graph identity excludes provider revisions. Compilation-request
provenance records the complete frozen registry, reached provider-independent
definitions, and admission-provider revisions as distinct subjects. A selected
plan and artifact include only reached definitions plus admission and optional
capability providers required by that plan. Registering an unused provider
therefore does not change graph meaning or invalidate an otherwise identical
artifact.

The initial extension execution model is trusted, statically linked compiler
code supplied explicitly to one compiler session. It does not promise native
dynamic plugin loading, sandboxing, or automatic discovery of consumer-local
registrations by a separately compiled proc macro. Registry, canonical-data,
provider-identity, threading, panic, and rewrite-transaction invariants are
specified in [Operation extensions](operation-extensions.md).

The implemented semantic callback receives host-validated operands and resolved
canonical attributes through an immutable request. It writes ordered result
facts through a host-owned bounded writer rather than returning an unrestricted
`Vec`. Maximum arity and aggregate canonical fact-byte limits are checked before
retention; a failed push poisons finalization, callback failure discards staged
facts, and minimum arity plus complete registry admission are rechecked before
graph mutation. These canonical-byte limits govern accepted identity work, not
exact allocator memory. Trusted provider code can still allocate, loop, panic,
or use unsafe code outside the host-owned data boundary.

Frozen-registry application first checks host-owned schema arity and attribute
field/kind rules. Only a structurally admissible application may invoke type
family validators, attribute authority validation, or the operation inferencer.
The complete schema checks are deliberately repeated at inference rather than
replaced by this ordering preflight.

Provider diagnostic codes are validated bounded newtypes and clone shared
storage cheaply. Operation-inference and type-instance errors remain distinct;
each accepts a bounded dynamic message. Invalid message construction is exposed
as a typed provider-contract cause under a reserved host diagnostic, without
truncation. Independent later failures remain explicit secondary evidence and
are not reported through Rust's causal `Error::source` chain.

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
- Optional Rust shape evidence is checked against derived graph shapes; it is
  never trusted as an alternative source of shape truth.
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
  or ambient compiler flags. Correctly rounded, faithful, typed piecewise
  bounded, and named-elementary contracts are discriminated; references,
  domains, exact tolerances, and metric versions are canonical identity.
- The initial optimizer enforces local numerical contracts and does not
  redistribute a graph-level error budget. Reference provenance, input/shape
  assumptions, casts, materialization boundaries, and reduction topology remain
  available to a future explicit region-accuracy analysis.
- Every root extent symbol has exactly one typed binding whose source class and
  availability phase are supported by every semantic factor that consumes it.
- Target-property bindings use stable versioned keys and cannot depend on a
  selected or prepared physical pipeline in the initial execution model.
- Binary operations use explicit broadcasting, except where a signature
  declares scalar admission as part of its own semantics: such a signature
  accepts a rank-zero operand and gives the result the other operand's shape.
  The built-in `tiler::add-f32@1` and `tiler::multiply-f32@1` signatures
  declare it, and ADR 0061 accepts the authoring facades over it. Operands of
  nonzero rank must agree in shape, every other many-to-one mapping still
  requires an explicit `Broadcast`, and the admission grants no dtype
  permission.
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
- The completed canonical graph contains only the transitive closure reachable
  from all program results; dead pure draft operations are removed and live
  storage is compacted during commitment, before identity is formed.
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

Semantic operation preconditions use a proof/witness contract independent of
their physical enforcement. Static proof erases the obligation; otherwise the
physical plan must name a supported enforcement and publication boundary, such
as host validation, device pre-scan, or transactional fused validation. A
semantic validation failure is never a plan miss. An explicitly trusted
assumption is a separate future policy, not another enforcement of strict
semantics.

A residual witness dependency names the stable predicate and obligation,
logical subject and component roles, exact logical view, value version or
immutability proof, and producer/coherence prerequisites. Consumers depend on
that witness, not on an untyped boolean or storage pointer. The physical
mechanism may erase or realize the dependency, but it cannot change the
predicate or subject. Witness reuse requires exact dependency equality or an
explicit proof of refinement.

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

This layer represents a canonical `IndexRegion` containing symbolic iteration
domains, scalar computation, and access maps. Operation compilation
capabilities may compose atomic semantic operations into this representation
after a region candidate has been formed. The structural region neither names
nor authenticates a semantic source by itself. An access map answers:

```text
(output coordinates, reduction coordinates, shape/interface parameters)
    -> logical tensor coordinates
```

It deliberately does not answer where those coordinates live in an allocation.
A selected physical implementation composes the logical `TensorAccessMap` with
a verified `BufferView` to derive allocation-relative element offsets. Storage
encoding and target lowering perform later checked element-to-byte or packed
address conversion. See ADR 0046.

Core concepts:

```text
ExtentExpr        IndexExpr          ScalarOperation / ScalarValue
IterationVar      IterationDomain    ReductionDomain
TensorAccessMap   ProvenFact
```

`StorageLayout` and `BufferView` are adjacent physical-plan concepts used only
when realizing these logical accesses.

The proposed scalar representation is a typed operation/value SSA graph, not a
closed Rust enum with one variant per dtype and operation. Each scalar
operation has a distinct namespaced and versioned `ScalarOpKey`, bounded
host-canonical attributes, ordered operands, and one or more ordered,
individually typed results. Each scalar value has one complete
`ResolvedValueType` and is either an access read or one result of exactly one
scalar operation. `ScalarOpKey` is deliberately distinct from semantic
`OpKey`: one semantic tensor operation may lower into several scalar
operations, and one fused scalar graph may implement several semantic
operations.

A frozen scalar-definition registry supplies the checked schema and semantic
authority for each `ScalarOpKey`. The schema owns operand and result arities,
canonical attributes, normative identity, and deterministic result inference.
Only ordinary scalar applications use these definitions; reduction is a
separate structural region whose body contains such applications. The host
exclusively derives and revalidates ordered result types; providers cannot
inject an asserted result type, untyped payload, `Any`, downcast value, or
unchecked node. Constants are zero-operand scalar operations with
schema-validated canonical attributes. Built-in and provider-defined dtypes
use the same `ResolvedValueType` path.

Canonical scalar attributes use the same `CanonicalValue` representation as
semantic operation attributes. Integer values retain their declared 8/16/32/64
bit width, floating values retain a registered format key plus exact bytes,
and field IDs are the `AttributeFieldId` newtype. A schema may own a typed
default. Inference observes the resolved default, while stored structural IR
and canonical identity omit an explicit value equal to that default. This
keeps construction spelling out of identity without delegating normalization
to a provider serializer.

Reduction is a structural region form rather than one enum variant per
reduction or dtype. It owns ordered bound dimensions, ordered initial state,
ordered contributor values, a checked nested scalar operation/value body, and
ordered results. The body receives typed state and contributor parameters and
yields the next state, so an N-state reducer may contain several generic
`ScalarOpKey` applications. The first supported traversal is an exact
lexicographic left fold whose empty result is its initial state; alternative
ordering contracts remain explicit rather than being inferred from a combiner.
This admits strict sum initially without freezing the representation around a
binary combiner, and preserves the structure needed for value/index pairs,
checked arithmetic, and other multi-operation, multi-result reductions.

`IndexRegion` identity commits only to the canonical structural program:
iteration and reduction domains, typed tensor boundaries, access maps, scalar
operations and values, constraints, and ordered outputs. Ordinary scalar
operation identity includes the key, normalized attributes, ordered operand
identities, and ordered resolved result types. Reduction identity additionally
includes its traversal, bound-dimension order, init/contributor identities,
nested body, and yields. Multi-result sharing is preserved by identifying one
operation occurrence and deriving each result identity from its result position.
Ownership tokens, arena indices, insertion order, provider addresses,
executable callbacks, proof caches, targets, and any semantic-region identity
are excluded.

The structural index verifier does not establish that an `IndexRegion`
implements any semantic operation or region. Compiler-owned legality evidence
separately binds a generated region to its selected semantic source and records
the reached scalar-definition and lowering-provider provenance required by
compilation and artifact identity. Matching shapes, dtypes, or operation names
cannot substitute for that evidence.

Before that semantic binding, a selected frozen scalar registry revalidates
every ordinary and reducer-body scalar application in a verified structural
region. It checks canonical attributes, operand/result arity, inferred result
types, and referenced type authority, then returns a receipt bound to the exact
`IndexRegion` identity. The receipt keeps the reached provider-independent
definition projection separate from provider-attributed admission provenance.
It is scalar-authority evidence only: it does not authenticate access maps or
prove semantic lowering equivalence.

Index expressions should be stored in an interned arena/DAG so repeated
division, modulo, and coordinate arithmetic can be shared and simplified.
They use exact signed mathematical-integer semantics for canonicalization.
The bounded initial vocabulary admits addition/negation, multiplication by a
parameter-only expression, and Euclidean floor division/modulo by a proven-
positive parameter-only expression. Iteration-by-iteration multiplication and
tensor-data-derived indices are rejected. Passes classify maps as affine,
constant-divisor quasi-affine, semi-affine, or data-dependent and may
conservatively decline classes they cannot analyze.

For a contiguous NHWC physical view, address derivation after logical access is:

```text
x[b,h,w,c]
  -> x_offset + b*(H*W*C) + h*(W*C) + w*C + c
```

For a runtime-strided physical view:

```text
x_offset + b*stride_b + h*stride_h + w*stride_w + c*stride_c
```

Logical transformations lower by reverse coordinate composition before that
address derivation. A flattened coordinate may be split with division/modulo;
a transpose permutes coordinates; a broadcast omits an iteration coordinate or
maps it to zero. These maps do not themselves promise a no-copy view.

Semantic constraints, index-domain predicates, physical variant guards, and
per-point schedule predicates are distinct. A `TensorAccessMap` is total over
its declared domain. Tail masks belong to scheduled IR rather than weakening
logical totality.

### Index verifier

- Access-map result rank matches the logical tensor rank.
- Every expression is integer typed.
- Divisors are proven positive and use Euclidean floor/mod semantics.
- Every logical coordinate is in bounds over the complete iteration domain.
- Canonical arithmetic does not overflow because it is width-independent.
- Broadcast reads may alias. Ordinary writes prove exact output coverage and
  unique ownership; reductions and atomics use explicit contracts.
- Every declared output produced by the compiled region is fully initialized
  according to its result contract. Narrow integration profiles may separately
  restrict execution to one out-of-place output.
- Zero-sized domains issue no accesses.
- Every runtime scalar has one ABI source.
- Every dynamic output extent, temporary size, applicability predicate, and
  launch expression is host-evaluable from declared input metadata or scalar
  ABI sources. Data-dependent output shapes and device-produced/indirect launch
  dimensions are initially unsupported.
- Semantic/index-domain bounds are proved or retained as semantic obligations.

### Proposed first static index profile

The in-progress first experimental `tiler_ir::index` slice is intended to
implement a deliberately smaller, fail-closed subset of this contract. This is
a required implementation profile, not a claim that the corrected generic
scalar model is complete:

- public owner-checked draft handles, a recoverable checked builder, borrowed
  structural views, and an opaque immutable `VerifiedIndexRegion`;
- exact mathematical-integer index constants backed by arbitrary-precision
  arithmetic, static parallel/reduction dimensions, canonical addition,
  constant scaling, and Euclidean floor division/modulo by positive constants;
- ordered typed input/output tensor boundaries and logical accesses with
  explicit lexical evaluation domains that end at tensor coordinates and
  retain no allocation, stride, byte-address, target-width, or physical
  execution-scope state;
- a generic checked scalar operation/value SSA representation with distinct
  `ScalarOpKey` authority, host-canonical attributes, registry-derived
  `ResolvedValueType` results, ordered multi-result values, and structural
  N-state reduction regions with lexical reduction dimensions;
- registry fixtures proving zero-operand constants, ordinary applications,
  multi-result operations, and exact serial reduction without dtype branches;
  the downstream initial executable profile remains strict `f32`, which is a
  capability limit rather than an intrinsic limit of scalar IR;
- interval bounds proofs, resource-bounded finite fallback when a conservative
  interval overlaps a boundary, structural permutation proofs for large
  ordinary writes, resource-bounded exhaustive ownership fallback,
  zero/rank-zero behavior, and access-owned bounds/write-ownership evidence
  with inspectable proof kinds; and
- reachable compaction plus canonical structural identity that excludes draft
  ownership, raw semantic handles, dead builder history, semantic-region
  identity, proof caches, provider addresses, and target choices.

Static dimensions and tensor boundaries expose optional `static_extent()` and
`static_shape()` facts rather than unconditional universal extents/shapes.
They return `Some` throughout this bounded profile. A future symbolic profile
can return `None` and expose its `ShapeEnv` expression through an additive
borrowed view instead of changing the meaning of an existing accessor.

The structural verifier proves only structural well-formedness, bounds,
lexical reduction closure, and ordinary write ownership. It does not claim
semantic sourceability or operation equivalence. A relation such as
`y[i] = x[0]` can be structurally valid and in bounds while being an incorrect
lowering of semantic `y[i] = x[i]`; later legality evidence must reject that
mismatch.

The first access profile remains out-of-place: input boundaries may be read but
not written, output boundaries may be written but not read, and every declared
output boundary requires exactly one complete ordinary write root.
In-place/read-modify-write relations, output partitions, atomics, and other
reduction organizations require later specialized contracts rather than
implicit relaxation. The first registered executable scalar capability set is
strict `f32`; other resolved dtypes reject through missing checked capability,
not through a closed scalar representation.

Completing this bounded static-extent profile will not complete the symbolic
contract above. `ShapeEnv`-backed root bindings,
semi-affine symbolic coefficients/divisors, typed index-domain predicates, and
durable solver evidence are tracked by
[`implement-shapeenv-index-bindings`](../tickets/implement-shapeenv-index-bindings.md)
and
[`implement-index-domain-predicates`](../tickets/implement-index-domain-predicates.md).
Unsupported dynamic cases must reject rather than entering an index-local
symbol or untyped predicate escape hatch.

### Physical view and address verifier

- Logical accesses compose with exactly one selected view/address convention.
- The derived accessible element/byte range fits the actual view and allocation.
- Layout and alignment requirements are proved or explicit variant guards.
- Coordinate, element-offset, byte/packed-offset, and dispatch widths are
  separately proved under the emitter's fixed evaluation order.
- A guarded `u32` path covers every relevant intermediate and retains a target-
  supported wide correctness path.
- Alias/view results refine the semantic coordinate relation and program alias
  contract; layout compatibility alone does not establish semantic equivalence.

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

- loop, tile, and fixed/scalable vector hierarchy;
- mappings from governed typed execution-scope coordinates into logical
  iteration coordinates, including bounded domains; GPU grid/workgroup/
  subgroup/lane and CPU task/thread/vector scopes are target-model examples;
- intra-kernel placement in governed addressable memory spaces, staging, reuse
  scopes, and local lifetimes; transparent caches remain cost facts;
- reduction topology, combination order, and result visibility;
- synchronization points, participant/execution scopes, fenced memory spaces,
  and convergence requirements;
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

- `TargetProfile` is governed planner input containing typed conservative
  compile guarantees, compatibility, data layout, execution/memory/vector
  models, phase-specific query/evidence schemas, artifact representation and
  runtime-translation policy, feasibility-rule identity, and a separate
  calibrated cost-model identity.
- `TargetRequirement` is the selected implementation's canonical bounded
  predicate over typed capability facts, candidate resources, evaluated launch
  values, ABI/layout, and binding/access facts, including any named deferred
  phase.
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
resource requirements and target predicates. Aggregate feasibility is
`Rejected` if any hard predicate is disproved; otherwise `Unknown` if any lacks
an admissible proof/query path; otherwise `Deferred` with a nonempty canonical
set of checks grouped by phase; otherwise `Proven`. It
checks launch limits, bindings, supported operations/dtypes/collectives, local
memory, and every other target-dependent hard constraint. A deferred candidate
survives intrinsic assessment only when the fact has an admissible query path
before `RoutingCommit`. The later portfolio or
integration verifier proves equivalent coverage for every deferred-rejection
region. That boundary
follows route-sensitive `LaunchPreflight` and final selection but precedes
output/scratch acquisition or encoding. Later allocation and launch invariants
fail closed. Estimates may guide search and dominance but cannot prove
feasibility.

`Unknown` candidates remain explain/search state only and cannot enter an
executable `ImplementationFrontier` or manifest.

Cross-kernel materialized buffers, dependencies, and lifetime intervals belong
to `KernelSubprogram` or `KernelProgram`, not an individual kernel schedule.
The schedule owns the canonical launch expression for its kernel; artifact
launch fields are checked derivations rather than a second editable authority.

## Layer 4: structured kernel IR

After scheduling, Tiler lowers into typed imperative code with lexical control
flow. The initial form uses immutable SSA-style values and typed loop-carried
values rather than general mutation. It is a verified refinement of exactly one
`ScheduledRegion`, not a second scheduler or target IR.

Representative constructs:

```text
BufferParameter    ScalarParameter    SpecializationParameter
ImmutableValue     For                If                 Yield
Load               Store              AtomicUpdate
Unary              Binary             Convert            Bitcast
CheckedNarrow      Barrier             Collective         Builtin
```

The initial form uses typed buffer references plus checked allocation-relative
element/storage offsets instead of unrestricted pointers. Buffers state element
or storage type, governed address space, access mode, alignment, accessible
range, and alias class. The initial alias contract permits input/input aliasing
but requires a newly allocated output that aliases no input. Richer alias
classes are deferred until an optimization consumes them.

Loads and stores carry dominating schedule-derived bounds evidence. Ordinary
stores also carry output-ownership evidence; atomics and reductions name their
selected protocols. Barriers separately state execution scope, memory scope,
fenced spaces, ordering, convergence, and the schedule synchronization point
they realize. Serial reductions use explicit loops; collectives retain the
selected participant set, combine order, identity/tail, owner/visibility, and
numerical realization. Conversions distinguish semantic value conversion,
representation conversion, checked index/address narrowing, and bitcast.

Invocation coordinates are governed builtins admitted by the kernel signature
and mapped to schedule execution axes, never backend source names. The schedule
owns launch formulas; the kernel and artifact contain checked references or
derivations rather than editable copies. General CFGs, recursion, unbounded
loops, unrestricted pointers, and calls with unknown effects are outside the
initial form.

### Kernel verifier

- Definitions dominate uses and lexical scopes are valid.
- Region arguments, loop-carried values, and yields have exact arity and types.
- Operation signatures agree with operand and result types.
- Buffer element types match loads and stores.
- Read-only buffers cannot be written and write-only buffers cannot be read.
- Address spaces are explicit and valid.
- Every memory effect has dominating bounds evidence; every ordinary store
  matches its scheduled ownership witness.
- Barriers and collectives match scheduled participant, scope, fence, phase,
  convergence, visibility, and order requirements.
- Builtins, loops, tails, accesses, conversions, and reductions refine the
  referenced schedule and numerical contracts.
- Derived local-memory and launch requirements match the schedule. Target
  support is established separately by target feasibility, then checked as a
  backend precondition rather than inferred from source acceptance.

See the [structured kernel IR research](research/kernel-ir/structured-kernel-ir-verifier.md)
for the proposed schema, worked lowerings, and verifier split.

## Layer 5: executable program and artifact-facing IR

`KernelProgram` and `ProgramPortfolio` are verified target-neutral executable
IR owned by `tiler-ir`. A kernel-program stage references the verified
structured kernel selected for that stage rather than only a schedule or a
compiler-private candidate. The artifact is the separately encoded versioned
unit consumed by a runtime; it carries this meaning with target payloads,
compatibility metadata, and compiler fingerprints without becoming a second
editable program authority.

Identity is layered:

1. `IndexRegion` commits to canonical iteration/scalar/access content.
2. `ScheduledRegion` commits to its `IndexRegion` plus normalized schedule.
3. `RegionImplementation` commits to its body, boundary contracts,
   applicability predicates, target requirements, and exact/proven resource
   requirements, including the selected numerical realization/provider.
   Conformance evidence identity, cost estimates, target-profile calibration,
   and schedule traces are provenance rather than semantic identity. The
   selected realization/provider and every output-affecting helper and flag
   remain physical-plan and artifact identity.
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

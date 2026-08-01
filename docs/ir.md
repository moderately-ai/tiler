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

### Accepted public API conventions

ADR 0074 accepts explicit conformance conventions for public Tiler APIs. That
record owns the decision and its evidence; the rules below are what those
conventions make normative for the shared IR surfaces this document owns. A
deviation is an argued decision, not an oversight.

**Errors are typed and non-erasing.** A public fallible IR entry point returns
a concrete failure enum, never `Box<dyn Error>`, another erased trait object,
or a string. Distinct failure kinds are distinct variants, and a variant
carries the structured data a caller reacts to — the rejected entity, the
exhausted resource with its attempted and permitted quantities, the expected
and actual arity — rather than a preformatted message. A failure that wraps a
lower-layer error returns it from `Error::source()` with its own type intact,
as the borrowed validation and consuming build paths below do for registry
authority failures. A convenience shared by two layers stays generic over both
concrete error types instead of unifying them, so an insertion-time admission
rejection and a whole-object verification failure remain structurally distinct.

**Identities are opaque and expose canonical bytes.** Every canonical identity
named in this document is a newtype whose byte storage is private and whose
only public reader is `as_bytes()`. Equality, ordering, hashing, dedup, and
cache keying use those bytes. An identity a layer derives has no public
constructor: only the encoder that establishes what the identity means may
produce one, so no caller can assemble an identity naming a subject that no
verifier examined. An identity received at a boundary may have an explicit
wrapping constructor when the constructor and the type both document that the
bytes are opaque and are never re-derived locally; such a constructor states
that this layer is not the authority for that subject, and it must not be used
to shortcut an identity the layer is the authority for. A surface may also
expose a short bounded label for explain output and diagnostics. That label is
presentation only and is never an equality or dedup input; the rule is about
the role of the value rather than the spelling of the accessor returning it.

**Canonical encodings are domain-separated, length-prefixed, ordinal-free, and
exhaustively matched.** An encoder writes a versioned NUL-terminated domain tag
of the form `tiler.<subject>.v<N>` before any content, so bytes produced for one
subject cannot be read as another subject's. It writes a fixed-width length
before every variable-length run, so no concatenation of fields is ambiguous.
It excludes transient identifiers — arena indices, builder insertion order,
graph-local ordinals, planning identifiers — wherever the represented semantics
are equivalent without them. It matches every encoded enum exhaustively, with
no wildcard arm and no silently omitted field, destructuring a single-variant
enum irrefutably so that widening the enum is a compile error at the encoding
site rather than two structurally distinct subjects that share identity bytes.
This is the encoding-level form of the common invariants that canonical
serialization is deterministic and that identity uses canonical content rather
than allocation identity. ADR 0074 names the encoders that do not yet satisfy
it; a landed encoder is not evidence that the rule already holds.

**Construction yields an unforgeable verified product.** The lifecycle above is
also a conformance item. The shared IR terminal is spelled `build`, it consumes
the builder, and its product cannot be forged: a verified product has private
fields, so struct-literal construction fails to compile, and it offers no
mutation, thawing, unchecked constructor, or mutable access to its draft. A
closure convenience delegates to the same builder and the same consuming
verifier rather than re-implementing verification, and it scopes the draft by
mutable borrow so the closure body cannot reach the consuming step. The frozen
authority snapshots described below consume their builder at `freeze` and are
immutable and unforgeable in the same sense, but a failed freeze does not
return builder ownership; whether ADR 0074's terminal rule reaches that
registry family is a question for that record and is not settled here.

**Verified products expose no public fields; leaf value-data descriptors may.**
A type whose invariants a verifier established exposes borrowed accessors,
iterators, and view types that yield meaning without yielding storage. A leaf
value-data descriptor — a plain record with no cross-field invariant that a
producer legitimately assembles or reads field by field, and that becomes
trustworthy only once a verifier binds it into a verified product — may expose
public fields, because opacity is enforced at the verified boundary rather than
at the descriptor. Which of the two forms `tiler-ir` should use for its own
descriptors remains open and is owned by
[`unify-schedule-index-region-with-verified-index-region`](../tickets/unify-schedule-index-region-with-verified-index-region.md).

**Two of ADR 0074's conventions are deliberately not stated here.** Its rule
for marking growing public enums and output records `#[non_exhaustive]` is
under amendment: for an enum a consumer exhaustively recognizes, the attribute
makes a later variant compile at every cross-crate consumer while silently
routing it into a reject-unknown arm, which is a silent loss of a supported
capability rather than the intended fail-closed compile break. That amendment
is owned by
[`resolve-non-exhaustive-recognizer-hole`](../tickets/resolve-non-exhaustive-recognizer-hole.md),
and this contract states no growth-marking rule until the record distinguishes
recognized enums from produced or read ones. Its staging rule for when a crate
module may be `pub` constrains component boundaries rather than representation
shape, and is normative in the architecture contract instead.

## Layer 0: frontend plan

The frontend plan retains syntax-level information such as axis names,
grouping, ellipses, and source spans. It validates operation-specific axis
rules and translates them into generic tensor semantics.

It must not contain storage strides, thread IDs, materialization decisions, or
Metal details. This layer normally remains owned by the frontend crate.

Required properties include:

- every input has a resolved logical rank or rank constraint;
- introduced axes have known shape expressions;
- removed axes name an explicit reduction;
- composed axes have factorization constraints;
- output axis order is complete and unambiguous.

## Layer 1: public semantic tensor graph

`SemanticTensorGraph` is the public, frontend-neutral semantic representation.
It is a pure, backend-neutral operation/value DAG describing what tensor values
mean. Frontends construct this graph; no frontend syntax, consumer runtime
object, storage layout, kernel boundary, target schedule, or live device object
belongs in it. Its shape expressions reference scoped extent symbols. A separate
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
operation list is illustrative rather than a closed Rust enum.

**An illustrative name is prose, and a governed key is an identity; the spelling tells you which.** The names in that list denote operation *families* in running text and none of them is an operation key. A governed key is always written `namespace::name@version` — the rendering `OpKey`'s `Display` produces — so the two never have to be distinguished by context: `tiler::add-f32@1` later in this section is a key, and `FloatAdd` above is not. Several placeholders sit deliberately close to a real identifier without being one. `FloatAdd` and `Multiply` name families that `tiler::add-f32@1` and `tiler::multiply-f32@1` currently realize for a single dtype, and `F32Add` and `F32Multiply` are the typed authoring facades over those two keys rather than third names for the families. Renaming a placeholder to match a key would assert a correspondence that does not hold, since a family is broader than any one key that realizes part of it.

In particular, an admitted `Gelu` key pins its exact formula or decomposition and every subordinate transcendental contract; erf-GELU and a tanh approximation are not interchangeable implementations of an unspecified node. The placeholder pins none of that, which is the difference the spelling rule above exists to keep visible.

**This document states what an operation means, never which operations exist.** The registered inventory is whatever a compilation request's frozen operation registry resolves; per-family status, evidence, and remaining work belong to the [operation-family support matrix](roadmap.md#operation-family-support-matrix). A placeholder appearing here is not a support claim, not a reservation, and not a commitment to that spelling — the matrix records, for example, that no `Cast` key exists, while `Reindex` and `Broadcast`, placeholders in this list exactly as `Cast` is, are now registered as `tiler::reindex-f32@1` and `tiler::broadcast-f32@1`. A name's presence in this list asserted neither state and asserts neither now.

An operation
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

`Reindex` represents a total output-to-input coordinate function plus its shape constraints, and that function is one named form drawn from a closed vocabulary rather than an arbitrary map — which is what lets totality and bijectivity be *proved* per form against the operand's shape instead of asserted by a caller. `tiler::reindex-f32@1`'s registered normative definition states the complete set and says "and no others": `permute-axes`, a reordering of whole axes; `split-axis`, a row-major factorization of one axis with the major factor first; `merge-axes`, the row-major product of a strictly ascending adjacent axis run; `insert-unit-axis` and `remove-unit-axis`, over an extent-one axis alone; and `reverse-axis`, the within-axis coordinate map `i -> extent - 1 - i`. A form outside that set is refused by name at construction, under `reindex.form.unadmitted-kind`, rather than approximated by a nearest admitted one.

`reverse-axis` is the whole of the within-axis admission, and it is one named form rather than a class for the reason the same definition records: "the affine within-axis bijections of an axis are exactly the identity and the reversal, while a general within-axis permutation is a tensor-data-derived index the accepted index vocabulary rejects" — which is exactly the rejection the bounded initial index-expression vocabulary states below. So naming the reversal admits every affine within-axis bijection that does anything, while admitting the general reading would admit at construction a permutation *table* that no lowering can produce. A within-axis rotation `i -> (i + k) mod n` is expressible in that vocabulary and deliberately unadmitted. This is where [the L4 attention design](research/program-planning/first-attention-program-vertical.md)'s decision D-10 is resolved: in the registered normative reference rather than in a research record.

Many-to-one broadcast/repeat behavior is represented separately by an explicit `Broadcast` with an axis mapping, and a non-surjective mapping is a slice — a different family, refused rather than admitted as a narrow reindex. A `Reindex` does not claim that storage was transposed or copied. Frontends may accept implicit broadcasting syntax, but the canonical semantic graph makes the mapping explicit before optimization.

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

**Implemented algebraic declaration.** `OperationDefinition` carries `OperationAlgebraicCapabilities`, whose first closed declaration is ordered associativity. The declaration is operation-owned semantic authority and participates in the frozen definition's canonical encoding; an optimizer rule may consume it only after matching the exact operation key, attributes, operand/result arity, and registry-inferred reassociated result facts. It is not a global property of `f32`, a capability inferred from a Rust facade, or numerical permission. The effective numerical contract independently decides whether a declared regrouping may be used.

**Fact — the implemented physical projection is bounded rather than scalar-generic.** The schedule layer can represent and verify one-input, one-output, three-leaf same-family `f32` add or multiply chains as `ScalarProgram::PointwiseF32(PointwiseF32Expression)`, allowing an algebraically reassociated candidate under an admissible numerical contract to reach exact structured-KIR lowering and verified program assembly. The expression has one implicit `f32` result per node, exact `u32` constant bits, ordered add and multiply operands, deterministic topology, DAG sharing, and an explicit root. It is a closed physical projection distinct from the index layer's registry-governed scalar SSA and introduces no generic dtype or operation authority. Other dtypes and operations, conversions, predicates, mixed-precision or multi-result programs, and compound encoded or quantized values remain explicit unsupported physical verticals until separately verified schedule, KIR, backend, reference, identity, and ABI contracts exist.

"Conservative" is the *floor* on that diagnosis, not a description of every family's behaviour. The iteration/access lowering entry is the one family the compiler resolves today, and it is stricter: an occurrence for which no installed capability resolves stops the compilation with a typed, occurrence-attributed cause rather than degrading to a narrower result, because a program with an unlowerable occurrence has no valid plan to degrade to. [The optimizer contract](compiler/optimizer.md#resolution-is-unconditional-and-fails-closed) owns that stage.

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
- Reassociation of any operation holds one same-operation leaf sequence and its order fixed while changing only grouping. The operation definition must declare ordered associativity and the effective numerical contract must permit reassociation; neither fact supplies the other.
- Reduction contracts apply that general reassociation rule to their canonical contributor sequence and distinguish it from contributor permutation. Neither permission implies the other, and each requires the corresponding operation capability before a schedule may consume it.
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

`ShapeExpr` is the expression language over that environment, and it is the
one this contract names at every layer that computes an extent.
[ADR 0008](decisions/0008-typed-root-bindings.md) fixes that a shape
expression references scoped extent symbols while `ShapeEnv` separately
declares how each root symbol is bound, so value algebra and value provenance
never become one concept. Its arithmetic is mathematical-integer rather than
machine-width, and it is a newtyped domain distinct from the bounded-width
`AbiExpr` a lowered program carries; [the shape environment
contract](research/shapes/shape-environment-contract.md) owns both accepted
decisions and the explicit checked lowering between the two domains.

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

**Fact — the semantic producer boundary is implemented.** `OperationDefinition` owns a bounded ordered set of typed declarations: stable predicate identity, host-derived declaration ordinal, operand selector, exact logical view, and stable invalid-input code. `SemanticProgramBuilder` instantiates them against the exact occurrence after schema, inference, and typed-result checks. A known proof retains its closed host-owned proof basis, a residual retains one cached canonical obligation identity, and a static disproof returns a typed owned build error before committing the operation or any result. Public provider identity cannot mint proof authority; only the sealed standard constant-f32 recognizer supplies the first proof basis. The host preflights aggregate residual-identity bytes against a governed 16 MiB bound before allocating or encoding those identities.

**Fact — residual identity is canonical occurrence identity, not an arena handle.** The identity is minted after reachable compaction from the semantic graph identity, reached definition projection, canonical operation coordinate, declaration encoding, canonical subject coordinate, complete resolved type, and shape. It excludes provider revision, registry snapshot, storage, pointer, checker identity, and mutable runtime state. The first assessment authority recognizes only exact governed f32 constants; every other producer remains residual rather than being inferred from type, shape, or descriptive facts.

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

**Fact — a semantic value's shape is fixed-extent in the implemented profile, and the symbolic vocabulary reaches only the index layer.** `crates/tiler-ir/src/shape.rs` is the "fixed shape vocabulary", its `Extent` wraps a `u64`, and `SemanticProgramBuilder::input` takes a `Shape`; the sourced constant-or-symbol vocabulary that `ShapeEnv` backs is exported from `tiler_ir::index`. So an extent symbol reaches an iteration domain, a tensor boundary axis, and a division divisor at layer 2, and reaches no semantic value at layer 1. That is a gap in this contract's own symbolic profile rather than a decided restriction: the accepted rule above is that each axis extent "may be a static integer or a scoped symbolic expression evaluated later". [The symbolic-semantic-extents record](research/shapes/symbolic-semantic-extents.md) runs the eliminations for how it is spelled, where the environment's identity enters, and what a frontend does with an extent unknown until dispatch, and files the delivery chain that closes it.

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
ShapeExpr         IndexExpr          ScalarOperation / ScalarValue
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

The two keys separate by namespace rather than by name component: `tiler::add-f32@1` is the semantic tensor family and `tiler.scalar::add-f32@1` is the scalar operation it lowers into. That is the general convention for operation names across layers of this contract, several of which are spelled identically in two layers while denoting different constructs. The [glossary](glossary.md#operation-names-shared-across-expression-layers) indexes which layer each shared name belongs to.

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
operations and values, constraints, ordered outputs, and the region's declared
numerical realization. Ordinary scalar operation identity includes the key,
normalized attributes, ordered operand identities, and ordered resolved result
types. Reduction identity additionally includes its traversal, bound-dimension
order, init/contributor identities, nested body, and yields. Multi-result
sharing is preserved by identifying one operation occurrence and deriving each
result identity from its result position. Ownership tokens, arena indices,
insertion order, provider addresses, executable callbacks, proof caches,
targets, and any semantic-region identity are excluded.

The declared numerical realization is inside that structural program, not
attached beside it, because it says what the region's scalar operations *mean*
rather than how a device executes them: two regions with identical domains,
accesses, and scalar content but different subnormal, contraction, or
reassociation resolutions compute different values, so they must not share
identity. Its encoding is **complete over every dimension and exhaustive per
dimension** — every field is encoded, each through a total match over a
vocabulary that is deliberately not `#[non_exhaustive]`, so widening the
vocabulary is a build error at the encoder rather than a silent identity
collision. No layer may encode the realization's contract key in place of the
field values that key names, and no layer may substitute a derived predicate
for the field it was derived from; a key and a predicate are both projections,
and a projection cannot fail closed when its source grows.

The region-level realization and the numerical fields a scalar operation
carries in its own right — a canonical NaN bit pattern, a contraction flag —
are both encoded, and they are not two authorities. The region-level
declaration is the contract; a scalar operation's fields are a refinement the
structural verifier requires to agree with it, so the agreement is checked
rather than assumed and encoding both cannot admit a disagreement.

The structural index verifier does not establish that an `IndexRegion`
implements any semantic operation or region. Compiler-owned legality evidence
separately binds a generated region to its selected semantic source and records
the reached scalar-definition and lowering-provider provenance required by
compilation and artifact identity. Matching shapes, dtypes, or operation names
cannot substitute for that evidence.

That evidence relates the reached scalar authority to the emitting capability's declaration by **containment, not equality**: a region must reach nothing beyond the scalar operations its capability declared it may emit, and it need not reach all of them. The declaration is a bound on what a lowering can compute, and a bound is exactly what containment checks. Equality would additionally require every declared operation to be exercised, which is a claim about one occurrence rather than about the capability and which no shape-general provider can satisfy — one capability lowers every occurrence of its family and signature, while which declared operations a given occurrence needs depends on that occurrence's shapes and attributes. [The optimizer contract](compiler/optimizer.md#scalar-authority-conformance-is-containment-not-equality) owns the reasoning and the worked reduction case; this layer states only which relation the evidence carries.

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

ADR 0084 fixes the accepted index-domain predicate language. A predicate is one atom in a closed exhaustive vocabulary, and a verified region carries atoms as an implicit conjunction. `NonNegative { expression }` and `LessThanExtent { expression, extent }` reference canonical verified index-expression nodes and region-owned dimension or tensor-axis extents; they do not embed a second expression tree. The two atoms state `0 <= e` and `e < extent` for every admitted affine, quasi-affine, or semi-affine expression. The vocabulary contains no Boolean escape hatch, physical guard, runtime check, or proof result.

Proof exchange keeps the accepted `Proved`, `Disproved`, and `Unknown(reason)` outcomes. Insufficient facts, an unsupported fragment, and a resource limit are structured reasons for `Unknown`, not evidence and not additional outcomes. An unproved index predicate enters one named compiler semantic-discharge stage before cover enumeration. A trusted rule assesses the exact borrowed region-owned obligation once; only an all-`Proved` result seals durable receipts and completes refinement, while `Disproved` and unsupported `Unknown` refuse atomically before program work. It never silently becomes a physical variant guard.

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
- interval bounds proofs, a structural proved-extent-equality bounds proof for a coordinate that *is* a domain dimension whose extent the environment proves equal to its axis, resource-bounded finite fallback when neither closes, structural permutation proofs for large ordinary writes, resource-bounded exhaustive ownership fallback, zero/rank-zero behavior, and access-owned discharged or residual predicate records with inspectable proof kinds and `Unknown` reasons; and
- reachable compaction plus canonical structural identity that excludes draft
  ownership, raw semantic handles, dead builder history, semantic-region
  identity, proof caches, provider addresses, and target choices.

Static dimensions and tensor boundaries expose optional `static_extent()` and
`static_shape()` facts rather than unconditional universal extents/shapes.
They return `Some` throughout this bounded profile. A future symbolic profile
can return `None` and expose its `ShapeEnv` expression through an additive
borrowed view instead of changing the meaning of an existing accessor.

The structural verifier proves structural well-formedness, lexical reduction closure, and ordinary write ownership, and it classifies each logical read-bounds atom as discharged or residual. It does not claim semantic sourceability or operation equivalence. A relation such as `y[i] = x[0]` can be structurally valid and in bounds while being an incorrect lowering of semantic `y[i] = x[i]`; later legality evidence must reject that mismatch.

Each access contributes exact logical-coordinate atoms—nonnegativity and strict upper bounds sourced from its tensor axes—independently of the tensor's `ResolvedValueType`. The same predicate vocabulary therefore applies to nominal booleans and integers, parameterized complex values, and encoded-numeric or quantized values. The compiler's finite discharge authority preserves that separation: it enumerates the logical dimension domain and evaluates index expressions with exact integers without reading tensor payloads or representation metadata. This establishes no physical bit, byte, component-buffer, packing, alignment, or masked-execution safety: those remain separate storage, ABI, schedule, and KIR obligations.

The finite fallbacks are resource-bounded. When a read predicate cannot be decided because facts are insufficient, the admitted expression fragment is unsupported, or a governed proof resource is exhausted, a structurally verified region retains the exact access-owned predicate with `IndexDomainUnknownReason::{InsufficientFacts, UnsupportedFragment, ResourceLimit}`. A resource-limit reason names the exact resource, required amount, and governed limit. It is neither a proof nor a physical guard. Genuine disproval, malformed structure, unresolved ordinary-write ownership, or any other hard diagnostic still rejects; a proof-resource limit beside such a diagnostic cannot upgrade the rejection into a verified region.

Discharged and residual predicates share one canonical assessment sequence in region identity, including exact subject, predicate, outcome, proof basis or unknown reason, exhausted resource, required amount, and limit. A residual also exposes an opaque canonical region-local key over that exact tuple; it must be paired with the owning region or refinement occurrence rather than treated as a global identity. A bounds-proof view is absent when an access retains residual bounds. The checked record lifecycle validates region ownership, handle existence, the expression's membership in the subject access, the tensor-axis association, and dimension-domain membership before minting either outcome.

Only a `SoundProof` or an exact `ExhaustiveFinite` result can mint a discharged predicate. `Empirical` remains a distinct reserved evidence class for measurements, but it is non-discharging and grants no execution permission. Compiler refinement returns a typed pending state for an otherwise-conforming region with residuals. That state owns the exact region that resolves its local handles, the full semantic occurrence, frozen scalar and capability authorities, and the already-checked operand/result bindings; it mints no refinement identity and cannot be consumed as an executable `IndexRefinement`. Compiler discharge does not mutate or rebuild that verified region: sealed proof receipts overlay it and join reusable refinement content, binding the exact region identity, local obligation key, proof authority and revision, and proof basis into refinement identity. This preserves region-owner handle validity and keeps later evidence distinct from the IR verifier's original assessment.

The first access profile remains out-of-place: input boundaries may be read but
not written, output boundaries may be written but not read, and every declared
output boundary requires exactly one complete ordinary write root.
In-place/read-modify-write relations, output partitions, atomics, and other
reduction organizations require later specialized contracts rather than
implicit relaxation. The first registered executable scalar capability set is
strict `f32`; other resolved dtypes reject through missing checked capability,
not through a closed scalar representation.

Completing this bounded static-extent profile will not complete the symbolic
contract above. `ShapeEnv`-backed root bindings landed with
[`implement-shapeenv-index-bindings`](../tickets/implement-shapeenv-index-bindings.md),
which split the rest. Semi-affine symbolic coefficients and divisors are tracked
by
[`admit-semi-affine-index-expression-class`](../tickets/admit-semi-affine-index-expression-class.md);
typed index-domain predicates and durable solver evidence by
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
- `ResourceRequirements` records exact quantities or proven upper bounds used for feasibility, such as bindings, threads, and local-memory bytes. Local-memory bytes are derived from the workgroup staging a cooperative tile allocates and are zero for every topology that stages nothing. It does not encode synchronization as a barrier-operation count.
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
- A cooperative workgroup tile's participants are the whole launched workgroup, every phase is uniformly reachable, staged writes are disjoint and total over their allocation *within one round* — the occupancy map spans the phase sequence once, which is one round, so a loop-carried tile may rewrite its slots on the next round while two writers inside a single round remain a refused race — staged reads are in range, within the declared per-round lifetime, and produced by a strictly earlier phase, and exactly one participant commits the owning write.
- Index ranges and coordinate maps cannot overflow under the declared guards.
- The chosen schedule preserves the declared numerical contract.

**Fact — the implemented schedule profile declares and verifies its synchronization, and the dataflow and its ordering are one checked structure.** `ReductionTopology::CooperativeWorkgroup` carries a `CooperativeTile` stating the participant set, the local coordinate space, the workgroup staging allocations with their shapes and declared per-round lifetimes, the ordered phases and the staged writes and reads each performs, how many rounds the phase sequence executes, which single participant commits the owning write, and the tile's own `SynchronizationPoint` list — each point an identity-bearing declaration of operation kind, placement, participant set, arrival and publication scopes, fenced spaces, ordering, and convergence evidence class. The intrinsic verifier proves the dataflow — participants are exactly the launched workgroup, every phase is reachable by all of them, staged writes are a bijection onto the allocation's slots within one round, every staged read is in range and in lifetime and has a strictly earlier producing phase, and the tile performs at least one cross-invocation handoff — and then proves the ordering against it.

**Two derived evidence classes, one discharge rule each.** `CooperativeTile::visibility_edges` derives, per (allocation, producing phase, consuming phase) triple, the producer-to-consumer dependency a point must discharge; on a tile with more than one round, `CooperativeTile::anti_dependency_edges` derives the cross-round anti-dependency — round `r + 1`'s rewrite must not overtake round `r`'s read — per (allocation, reading phase, rewriting phase) triple over every phase pair regardless of order, because the rewrite is in the following round. A point discharges a visibility edge by the conjunction "placed at or after the producer and at or before the consumer"; it discharges an anti-dependency by the disjunction "at or after the read, or at or before the rewrite", and a `SynchronizationPlacement::RoundBoundary` — which names no phase ordinals, since the phases it separates are the tile's own last and first — discharges every anti-dependency and no visibility edge. The verifier requires exactly one discharging point per edge in each class, refuses an undischarged edge by class (`UndischargedVisibility`, `UndischargedAntiDependency`), refuses a point that discharges nothing in either class as redundant, and refuses convergence evidence that does not match the tile's round structure: a single-round tile's points rest on every participant reaching the point, a repeating tile's on every participant executing every round — a derivation the declared literal round count supports and a loaded or per-invocation count could not. A caller's bare assertion of convergence is refused whatever the tile looks like. A schedule with no cooperative tile has no edge and is admitted vacuously, consuming no target synchronization fact. Whether any *target* can perform the realization a point requires remains a feasibility question composed against the target's own declaration — a schedule proving its own ordering claims nothing about any device.

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

Synchronization is not a scalar capacity axis. A future synchronized schedule must carry one typed obligation joining the schedule synchronization point and phase, operation kind, participants and execution scope, visibility, fenced memory spaces, ordering, and convergence. Target support must be one atomic realization fact with provenance over that same subject; independently true component facts cannot be composed into permission for a combination no authority established.

An `Unknown` *feasibility* verdict keeps its candidate in explain and search state only; such a candidate cannot enter an executable `ImplementationFrontier` or manifest. This rule is about target feasibility and does not erase `Unknown` elsewhere: a structurally verified index region may retain an exact unresolved logical predicate. That region remains valid analysis state, but it is not executable refinement evidence. The current compiler runs [semantic discharge before cover or frontier construction](compiler/optimizer.md#refinement-requires-discharged-index-domain-evidence), completes refinement only when every residual is proved, and otherwise fails closed with a typed `Disproved` or `Unknown` assessment.

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

These names are local to this layer. `Unary`, `Binary`, and `Constant` are also operation names in the ABI expression language of Layer 5, in the index and scalar expressions of Layer 2, or in both, and they denote different constructs there; `Constant` additionally names a governed address space within this layer rather than an operation. The [glossary](glossary.md#operation-names-shared-across-expression-layers) indexes every such name against its layer, construct, implementation spelling, and maturity.

The initial form uses typed buffer references plus checked allocation-relative
element/storage offsets instead of unrestricted pointers. Buffers state element
or storage type, governed address space, access mode, alignment, accessible
range, and alias class. The initial alias contract permits input/input aliasing
but requires a newly allocated output that aliases no input. Richer alias
classes are deferred until an optimization consumes them.

Loads and stores carry dominating schedule-derived bounds evidence. Ordinary stores also carry output-ownership evidence; atomics and reductions name their selected protocols. Staged loads and stores are authorized differently, and deliberately so: a staged access names the cooperative-tile *phase* whose declared staged write or read admits it — which is what a bounds witness is for a boundary access — and that phase also fixes when the effect happens relative to the tile's visibility edges. A `BarrierSpec` names the schedule synchronization point it realizes together with the execution scope, memory scope, fenced spaces, and ordering a backend emits. Those are not one authority: the point states the obligation and the spec declares how it is spelled, and the verifier projects the spec onto the point's subject through one total mapping and requires equality, exactly as a kernel's declared resource requirements are proven equal to the derived record rather than trusted beside it. A barrier is admitted only when its point is one the region's cooperative tile declares, when it sits where its tile's round structure makes it convergent, and when it lexically separates each staged write from the staged read consuming it. Convergence is two facts: no predicate may enclose it, because a predicate admits a dynamic subset of the participants; and its loop nesting must be the one the tile authorizes, which is the round loop for a tile whose phases repeat and nothing at all for a tile that runs them once. A loop is admissible where a predicate is not because a serial loop's bounds are `u64` literals, so every invocation runs an identical trip count and reaches the same dynamic instance — and the round count a tile declares is a literal for the same reason. A barrier in a region whose schedule owns no point remains `UnexpectedSynchronization`, which is where the redundant barrier in a pointwise, global-linear program is eliminated. Serial reductions use explicit loops; collectives retain the selected participant set, combine order, identity/tail, owner/visibility, and numerical realization. Conversions distinguish semantic value conversion, representation conversion, checked index/address narrowing, and bitcast.

Workgroup staging is declared separately from the buffer parameters, not as one of them. A buffer parameter's position is its argument-table ordinal, and a workgroup allocation is not an argument, so placing one in that list would re-base every later ordinal and change what an existing signature position means; a staging declaration instead names the scheduled staging ordinal it realizes, and the verifier proves the declared element type, address space, and slot count against the region's cooperative tile.

Invocation coordinates are governed builtins admitted by the kernel signature
and mapped to schedule execution axes, never backend source names. A region
carrying a cooperative tile additionally admits the local invocation
coordinate, because its participants are named by their position within the
workgroup. The schedule
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
- Barriers match scheduled participant, scope, fence, phase, convergence, visibility, and order requirements: every emitted barrier names a declared point of the region's cooperative tile, sits outside every predicate and no deeper than the tile's own round loop, is realized as many times as its placement requires — `rounds` for a phase boundary and `rounds - 1` for a round boundary — and projects onto that point's subject through one total mapping. Collectives remain unadmitted.
- Declared workgroup staging realizes the region's cooperative tile exactly, and a region that stages nothing declares none.
- A visibility edge no declared point discharges is rejected as `UndischargedVisibility`, and a cross-round anti-dependency no point discharges as `UndischargedAntiDependency`, before any body is derived. For a single-round tile whose edges are each discharged, `emit_cooperative` is the canonical lowering: it stages, barriers, and re-reads in the order the points declare. A multi-round tile lowers to `emit_loop_carried_cooperative`, which peels round zero — every fold seeds at its first contributor, because `+0.0 + x` is not `x` at `x = -0.0` — and carries the accumulator through a `1..rounds` loop whose body opens with the round boundary. The staged fold sits outside every predicate, because a predicated region produces no value that could cross the loop's back edge and staged accesses are not the boundary effects predicate dominance governs. The round boundary's position is checked rather than assumed: a barrier at position `b` of the round body separates round `r`'s read at `c` from round `r + 1`'s write at `w` exactly when `b > c` or `b < w`, and only the second arm also orders the peeled round's reads against the loop's first rewrite (`UnorderedStagedRewrite`).
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

1. `IndexRegion` commits to canonical iteration/scalar/access content and to
   its declared numerical realization, complete over every dimension.
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

The compiler may construct one target product from independently readmitted semantic candidates, but this does not weaken the identity layers above. Each retained `ProgramAlternative` has an owning semantic-candidate key and an identity re-derived from the rule origin, the owner's exact semantic program, the owner's exact verified target request, and the verified physical plan. The global verifier requires the flattened alternative set to equal the union attributed to those owners and refuses an alternative whose owner key or re-derived identity disagrees. Global selection is then re-derived from that verified set.

The composite explain object is a compiler-owned companion rather than executable IR. Its v3 top-level semantic-selection records bind each candidate key to the exact canonical `CompilationSubject`; each nested candidate trace carries its own subject. Composite verification requires exact equality between the two and rejects duplicate subjects, duplicate keys, missing candidates, and key-preserving subject swaps. A rendered qualifier remains diagnostic and cannot substitute for this full-subject binding.

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
- subnormal input and result handling as independent dimensions, each stating
  which zero a flush produces;
- fast math, reassociation, and fused multiply-add permission.

An optimization that changes reduction order is not an exact rewrite merely
because it is algebraically valid over real numbers.

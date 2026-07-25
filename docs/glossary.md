---
schema: "tiler-doc/v1"
id: "tiler.portal.glossary"
kind: "portal"
title: "Glossary"
topics: ["terminology"]
---

# Glossary

Use these terms consistently in documentation, diagnostics, and code.

| Term | Definition |
| --- | --- |
| Access mode | Whether a binding may be read, written, or both. |
| Accumulation dtype | Type specified independently for reduction accumulation; it may equal the input or output dtype. |
| Accuracy contract | Canonical per-operation allowed result set relative to immutable reference semantics, including domains and versioned metrics where applicable. |
| Accuracy guarantee | Machine-checkable result-set claim made by one candidate implementation; it must refine the requested accuracy contract. |
| Add (shape expression) | The canonical, possibly n-ary addition of the shape-metadata expression language. Its arithmetic is mathematical-integer by the accepted decision of 2026-07-19: it does not wrap, saturate, or expose overflow from an arbitrary compiler intermediate width, so `ExactDiv(A * B, B) == A` holds even where `A * B` exceeds any machine width. Distinct from *Add (tensor)*, which is floating-point and carries a rounding contract instead. Adopted and unimplemented: `crates/tiler-ir/src/shape/` implements only the crate-private `ExtentTerm`/`ExtentRelation` constraint fragment, which is deliberately closed under relations over symbols and literals and has no arithmetic node at all. |
| Add (tensor) | The semantic elementwise addition family whose strict boundary is a resolved homogeneous computation and result type, round-to-nearest-ties-even, and a rounding boundary distinct from every other operation's. Registered as `tiler::add-f32@1` and authored through the `F32Add` facade; the scalar operation it lowers into is separately identified as `tiler.scalar::add-f32@1`. Distinct from *Add (shape expression)*: naming a shape formula's addition `Add` grants it no rounding contract, and naming a tensor addition `Add` grants it no exact mathematical-integer arithmetic. A `Multiply` followed by an `Add` has two rounding boundaries, and whether they fuse into one is *contraction (numerical)* rather than a property of `Add`. |
| Artifact | A versioned bundle or kernel-entry record consumed across the compiler/runtime boundary. |
| Axis symbol | Stable frontend identity for a logical axis such as `b`, `h`, or `w`. |
| Bundle | Self-contained target artifact and manifest containing a complete program portfolio; an integration may scope one bundle to one macro invocation. |
| Capability fact | Typed value for a governed target key with explicit availability phase, validity scope, authority, and provenance. |
| Capability phase | One of `CompileProfile`, `ArtifactEvidence`, `LiveDevicePreflight`, `PreparedKernelPreflight`, or `LaunchPreflight`, stating when a physical fact becomes available. |
| Canonical attribute | Host-owned bounded typed value attached to a semantic operation; its normalized encoding, not provider serialization, participates in identity. |
| Canonical identity | Opaque newtype over the deterministic canonical bytes identifying one subject; the bytes are read only through `as_bytes()` and are the sole input to equality, ordering, hashing, dedup, and cache keying. |
| Byte offset | Offset used by a buffer-binding API, measured in bytes. |
| Candidate region set | Overlapping region candidates considered by program planning; a hypergraph may be used as its internal index. |
| Boundary enforcer | Explicit materialization, layout conversion, or copy that satisfies a boundary requirement. Value-preserving by construction: a dtype cast is a semantic operation, not an enforcer. |
| Boundary guarantee | Layout/materialization/storage contract a region implementation provides for an outgoing value. |
| Boundary requirement | Layout/materialization/storage contract a region implementation requires of an incoming value. |
| Buffer plan | Kernel-program mapping from logical values/views to allocation identities and verified lifetime intervals. |
| Collective | Operation involving several hardware lanes or threads, such as a reduction. |
| Compilation request | One semantic graph plus numerical/shape context, frozen operation registry, installed lowering capabilities, targets, budgets, and deterministic compiler options. |
| Compile guarantee | Conservative typed capability fact promised by a declared target profile before artifact generation. |
| Contraction (numerical) | ADR 0015's permission for a separately rounded multiply and add to fuse into one rounding, as in a fused multiply-add. Distinct from *contraction (tensor)*, with which it shares only a name. Spelled `NumericalRealization::contraction`, `StrictF32NumericalContract::contraction`, and `MetalNumericalRequirement::NoFloatingPointContraction`. |
| Contraction (tensor) | Summation over indices shared by two or more operands — matrix multiply, batched matmul, einsum. Distinct from *contraction (numerical)*: naming an operation a tensor contraction grants it no rounding permission. The two meet only in that a tensor contraction's per-contributor `accumulator + a * b` step is where the numerical permission would apply, and under the registered strict `f32` contract that permission is `Forbidden`. Its *association* is separately governed: regrouping `(AB)C` to `A(BC)` consumes distributivity, which no expressible contract grants, rather than reassociation. |
| Element offset | Typed index into elements of a buffer view, distinct from a byte offset. |
| Evidence class | Scope and strength of support for an implementation guarantee: proof, exhaustive, normative guarantee, empirical qualification, or unknown. |
| Expansion compiler cache | Disposable global content-addressed cache used by proc macros to avoid repeated external AOT compilation. |
| Expansion-time AOT | Offline target compilation performed synchronously while a proc macro expands, with completed bytes embedded in returned Rust. |
| Extent expression | Static extent or expression over runtime scalar parameters. |
| `F32Add` / `F32Multiply` (semantic authoring facade) | The public typed facades over the registered tensor families `tiler::add-f32@1` and `tiler::multiply-f32@1`. Each is a unit struct whose `apply`, `apply_shaped`, `apply_scalar_left`, and `apply_scalar_right` constructors append one semantic operation to a `SemanticProgramBuilder`; they are defined in `crates/tiler-ir/src/semantic/standard_operations.rs` and re-exported from `crates/tiler-ir/src/semantic.rs`. Distinct from *`F32Add` / `F32Multiply` (structured kernel operation)*, which is the same spelling for a device-level operation in a lowered kernel body. The two meet at exactly one point — lowering a semantic graph is what produces the kernel operations — and share nothing else: one is how a caller authors a graph, the other is an instruction inside the result. |
| `F32Add` / `F32Multiply` (structured kernel operation) | Two variants of `BinaryOp`, the pure binary operation of the structured kernel IR, documented "IEEE-754 binary32 addition" and "IEEE-754 binary32 multiplication", defined in `crates/tiler-ir/src/kernel/model.rs` and consumed by the Metal emitter in `crates/tiler-metal/src/emit.rs`. Their four sibling variants are spelled `IndexAdd`, `IndexMultiply`, `IndexDivide`, and `IndexModulo`, whose prefixes carry the operand role where these two carry only the element type. Distinct from *`F32Add` / `F32Multiply` (semantic authoring facade)*. This is the only pair in the shared-name table whose two senses are both implemented, both public, and both defined in `tiler-ir`; every other shared name separates by maturity, by crate, or by both. |
| Fallback | Semantically compatible alternative execution path used when no compiled variant applies. |
| Fusion visibility boundary | Limit that a frontend can optimize only semantics submitted in its semantic graph; a proc-macro invocation is one such boundary. |
| Applicability predicate | Runtime-checkable condition under which a program or region implementation may execute. |
| Guard | Checked predicate with provenance: semantic input, storage, schedule, target, or dispatch safety. |
| Implementation frontier | Bounded non-dominated region implementations returned for one candidate and target profile. |
| Index-domain predicate | Constraint restricting the mathematical points of an index region; it is part of map truth rather than a physical fallback guard. |
| Index-region refinement | Compiler-owned checked evidence that one emitted `IndexRegion` realizes one semantic occurrence: agreeing ordered value interface, reached scalar authority contained in what the capability declared it may emit, agreeing semantic type authority, and complete unique-write ownership. A successful structural build is not this evidence. |
| Iteration domain | Cartesian coordinate space over which outputs are computed. |
| Kernel ABI | Ordered buffer and scalar parameters with types, roles, access modes, and binding locations. |
| Launch geometry | Grid and threadgroup dimensions derived from a schedule. |
| Leaf value-data descriptor | Plain record with no cross-field invariant that a producer assembles or reads field by field and that becomes trustworthy only once a verifier binds it into a verified product; it may expose public fields. |
| Logical coordinate | One index for each logical tensor axis. |
| Logical shape | Ordered axis extents independent of physical storage. |
| Manifest | Canonical metadata describing a bundle or kernel entry without being executable shader code. |
| Macro-local bundle | Bundle whose collection scope is one inline invocation, though it may contain many kernel entries and steps. |
| Kernel program | Executable dependency DAG of kernel stages, materializations, opaque calls, buffers, and launches. |
| Kernel schedule | Normalized mapping from one region's iteration/access representation onto target execution and memory hierarchy. |
| Live-device capability | Typed fact queried for one runtime device/context and used for preflight or routing, not portable semantic identity. |
| Map/scalar expression | Typed pointwise computation formed while lowering or fusing semantic operations into a region implementation. |
| Materialization | Allocating and storing an intermediate tensor rather than retaining it in a fused expression. |
| Materialization boundary | Kernel-program edge at which an intermediate tensor is stored. |
| `Minimum` / `Maximum` (ABI expression) | Two variants of `AbiBinaryOp`, documented "Unsigned minimum" and "Unsigned maximum" and evaluated as `u64::min` and `u64::max` over the checked 64-bit unsigned domain of the host-side ABI expression language, in `crates/tiler-ir/src/program/abi.rs`. That domain has neither NaN nor a signed zero, so the exceptional-value contract defining *`Minimum` / `Maximum` (tensor)* is not merely unimplemented here — it is inexpressible. The invariant a reader needs: every `Minimum` or `Maximum` in `crates/` that is an identifier rather than English prose in a comment is this construct, reproducible with `grep -rnwE 'Minimum\|Maximum' crates/ \| grep -vE ':\s*(///\|//\|\*)'`, which returns lines only from `program/abi.rs`. The shape-expression language deliberately spells its own extrema `Min` and `Max`. |
| `Minimum` / `Maximum` (tensor) | The semantic extrema family whose strict boundary is NaN-propagating with a deterministic `-0.0 < +0.0` ordering, so the minimum of opposite-signed zeros is `-0.0` and the maximum is `+0.0`; NaN-absence and signed-zero relaxations remain independent permissions. It is a separate family from `MinimumNumber`/`MaximumNumber` rather than one operation with a backend-selected mode, and Metal `fmin`/`fmax` are number-preferring and therefore not an exact implementation of it without a fixup or a matching authorized relaxation. No operation key registers it. Distinct from *`Minimum` / `Maximum` (ABI expression)*: an implemented `Minimum` found in `crates/` is never evidence that this family is supported. |
| Numerical contract | Operation semantics, optimization permissions, and execution guarantees taken together. |
| Numerical policy | Granular optimization permissions such as reassociation, contraction, and approximate intrinsics. |
| Numerical mode | Optional user-facing preset that expands into a complete numerical contract. |
| Opaque call | Physical kernel-program stage implementing a normatively defined semantic operation through an external/library implementation with explicit boundary, target, hazard, and cost contracts. |
| Operation | Atomic named semantic tensor computation consuming values and initially producing one or more individually typed values. |
| Operation definition | Registered versioned capabilities that define an operation's schema, semantics, verification, decomposition, optimization, and lowering support. |
| Operation key | Durable dialect, name, and semantic-version identity stored in semantic IR. |
| Operation registry snapshot | Immutable deterministic per-compilation mapping from semantic operation keys and capability-provider IDs to trusted implementations and revisions. |
| Presentation label | Short bounded digest of canonical identity bytes used in explain output and diagnostics; it is never an equality, ordering, or dedup input. |
| Provider revision | Stable provider-declared fingerprint for output-affecting implementation behavior, distinct from an operation's semantic version. |
| Program portfolio | Guarded alternatives containing complete kernel programs for one semantic graph. |
| Program result | Ordered named reference to a semantic value returned by the graph; it is not an `Output` operation. |
| Prepared-kernel capability | Fact available only after selecting and preparing one entry point/specialization for a live device. |
| Region candidate | Proposed semantic subgraph plus explicit boundary values, retained outputs, materialized edges, duplication policy, and contract. |
| Region partition | Compatible covering set of selected region candidates. |
| Resource estimate | Non-authoritative estimate such as register pressure, occupancy, or code size used for pruning and costing. |
| Resource requirements | Exact quantities or proven upper bounds such as threads, bindings, and local-memory bytes used for feasibility. |
| Reindex | Logical output-to-input coordinate transformation; it does not imply a storage copy. |
| Reduction domain | Coordinates combined to produce one logical output value. |
| Schedule trace | Non-authoritative history of scheduling transforms, parameters, preconditions, and rejections retained for explain/replay. |
| Root binding | Typed declaration mapping a semantic extent symbol to a static value, input dimension, interface parameter, or admitted target property. |
| Routing commit | Boundary after all route-sensitive launch preflight and final variant selection, before output/scratch acquisition or encoding; no later failure selects another plan or semantic fallback. |
| Select (ABI expression) | The ternary conditional of the host-side ABI expression language: `ExprNode::Select`, defined in `crates/tiler-ir/src/program/abi.rs` and re-exported through `tiler-artifact`'s `program::expr` shim, projected as `AbiExprView::Select` and verified in that crate's `program/verify.rs`. Condition and branches are bounded-width ABI values; evaluation is lazy, so only the selected branch is evaluated. Distinct from the three `Select` entries below, and the only one of the four that exists in compiled code — every hit of `grep -rnw Select crates/` is this construct. It meets *Select (shape expression)* at exactly one point, the explicit typed checked lowering from `ShapeExpr` to `AbiExpr`, and shares no identity with it. |
| Select (shape expression) | The ternary conditional of the closed shape-expression language, whose condition is a host-evaluable shape predicate and whose branches share one shape-expression type. It computes shape metadata: the shape-environment contract states it "is not tensor `where`, general logical-graph control flow, or a device branch", which is what separates it from *Select (tensor)* and from *Select (structured kernel operation)*. Distinct from *Select (ABI expression)* by the accepted decision that `ShapeExpr` and `AbiExpr` are separate newtyped domain IRs, which may share arithmetic components without sharing identity. Adopted and unimplemented: `crates/tiler-ir/src/shape/` defines no `ShapeExpr`. |
| Select (structured kernel operation) | A device-level operation in the bounded initial operation set of the structured kernel IR — the kind of branch that *Select (shape expression)* explicitly is not. Its research is `disposition: adopted` with `implementation_status: spike-only`, and the implemented structured-kernel vocabulary in `crates/tiler-ir/src/kernel/model.rs` has no `Select`. It is a proposal, not support. |
| Select (tensor) | The semantic tensor operation family named by the adopted conformance matrix row "`Select` and bit-selecting operations": preserve the selected operand's bits, with explicit predicate semantics. That row is the only place the corpus mentions the family — no ADR, no normative contract section, and no operation key defines it, and the roadmap's operation-family support matrix places it at the lowest rung. A `Select` found in `crates/` is therefore never evidence that this family is supported; that hit is *Select (ABI expression)*. |
| Semantic tensor graph | Public backend-neutral operation/value DAG describing tensor values and named program results as a function over explicit inputs and extent symbols. |
| Semantic authority | The single registered definition owning an operation key's meaning, schema, normative specification, and deterministic inference/validation contract. |
| Shape constraint | Equality, divisibility, interval, or factorization fact required by tensor semantics. |
| Shape environment | Scoped symbolic extent declarations, static/runtime source bindings, semantic constraints, and derived facts. |
| SIMD group | Hardware subgroup of lanes that execute and cooperate; Metal uses this term where CUDA commonly uses warp. |
| Scalable vector shape | Vector lane count expressed as a fixed minimum multiplied by a runtime-stable scale rather than one compile-time width. |
| Source origin | Diagnostic mapping from canonical IR back to frontend source. |
| Storage layout | Base offset and physical strides associated with a logical shape. |
| Target profile | Governed versioned compile guarantees, compatibility, execution/data-layout models, phased query schemas, feasibility rules, and separately identified tuning model used for physical planning. |
| Target property binding | Explicit semantic root binding to a stable, versioned property admitted initially from `CompileProfile` or `LiveDevicePreflight`; later physical phases cannot overwrite it. |
| Target requirement | Canonical bounded predicate over typed capability, candidate-resource, launch, ABI/layout, and binding/access facts required by a selected implementation, possibly deferred to a named safe preflight phase. |
| Tensor access map | Width-independent map from iteration coordinates and admitted parameters to logical tensor coordinates; storage addressing is derived separately. |
| Tensor expression | Pure semantic operation that produces a logical tensor value. |
| Tensor view | Logical shape, strides, and start position over an allocation. |
| Threadgroup | Threads dispatched together with shared synchronization and memory; Metal uses this term where CUDA commonly uses block. |
| Value | Individually typed semantic result with exactly one definition and zero or more consumers. |
| Variant | One complete kernel program plus applicability/routing contract for a semantic graph. |
| Verified product | Immutable value whose invariants a consuming `build` established; it has private storage, no public fields, no unchecked constructor, and no path back to a mutable draft. |

Avoid using **layout** for both logical axis transformations and physical
strides. Use **reindex** or **axis transform** for the former and **storage
layout** for the latter.

Avoid **mega-kernel** in normative interfaces. It is useful conversationally,
but **fused kernel** or **fusion region** states the compiler concept without
implying that larger is always better.

Avoid unqualified **property** and **physical plan** where a more precise term
exists. Use boundary requirement/guarantee, target requirement, applicability
predicate, resource requirements/estimate, schedule invariant, cost estimate,
`RegionPartition`, `KernelSchedule`, or `KernelProgram` as appropriate.

Never write an unqualified **`Select`** in normative text or diagnostics; name
which of the four constructs above is meant. A substring search cannot make the
distinction for you, and this is the case AGENTS.md's warning against concluding
support from a search is about: every `Select` in `crates/` belongs to the ABI
expression language, and the same search over `docs/` additionally returns ADR
0049, whose title uses the ordinary English verb and defines no `Select`
construct at all.

## Operation names shared across expression layers

`Select` is not the only name that denotes several unrelated constructs. Six vocabularies in this repository name operations, and each is the naming authority only inside its own layer. The same spelling appearing in two of them is two constructs, not one construct seen twice.

| Layer | Naming authority | Maturity of that vocabulary |
|---|---|---|
| Semantic tensor operations | Governed `OpKey`s in the `tiler` namespace, plus the family names of the adopted [operation conformance matrix](research/numerics/operation-conformance-matrix.md) | Four families have a governed key and a lowering capability outside tests — `constant-f32`, `multiply-f32`, `add-f32`, `strict-serial-sum-f32`; most matrix families have no key |
| Shape expressions and predicates | The accepted 2026-07-19 decisions in the [shape environment contract](research/shapes/shape-environment-contract.md), which also record that the exact initial primitive set remains to be chosen | Adopted; the only implemented part is the crate-private `ExtentTerm`/`ExtentRelation` constraint fragment |
| Index and scalar expressions | `IndexExprView`, and governed `ScalarOpKey`s in the `tiler.scalar` namespace | Implemented and public |
| Structured kernel operations | The bounded initial set of the adopted [structured kernel IR research](research/kernel-ir/structured-kernel-ir-verifier.md); [Layer 4 of the IR contract](ir.md#layer-4-structured-kernel-ir) names a representative subset | Partially implemented; `OperationView` is the implemented subset |
| ABI expressions | `ExprNode`, `AbiUnaryOp`, and `AbiBinaryOp` | Implemented and public |
| Reference oracle | Independent scalar reference implementations in `tiler-reference` | Implemented; a checking authority rather than a program vocabulary |

**The convention, now recorded.** An operation name is layer-qualified by its identity, not by its bare spelling, and two mechanisms already do this. A governed key carries a namespace: `tiler::add-f32@1` is the semantic tensor family and `tiler.scalar::add-f32@1` is the scalar operation it lowers into, identical in name component and distinct in authority, which is the same separation [the IR contract](ir.md#layer-2-index-and-iteration-ir) states when it makes `ScalarOpKey` deliberately distinct from `OpKey`. A Rust variant carries its enum path, so `AbiBinaryOp::Minimum` and `BinaryOp::F32Add` are unambiguous at a use site. Where neither mechanism is present — a bare word in prose, a diagnostic string, a search result — the layer is unrecoverable from the name and the reader must look it up here.

A new operation name entering a layer should be distinguishable from an existing name in another layer by its key namespace or by a qualified spelling. Several already are, unevenly and without anything having recorded the rule: the ABI language spells its addition `CheckedAdd` and its divisibility predicate `IsMultipleOf` where the shape language says `Add` and `Divisible`; the shape language spells its extrema `Min` and `Max` where the tensor family says `Minimum` and `Maximum`; the structured-kernel research spells its narrowing `CheckedNarrow`; and four of `BinaryOp`'s six variants carry an `Index` prefix. `AbiBinaryOp::Minimum`, `AbiBinaryOp::Maximum`, `AbiBinaryOp::Equal`, and `BinaryOp::F32Add` did not, which is why the table below exists.

The table indexes which layer a name belongs to. Where two senses also differ semantically in a way that produces a wrong answer rather than only a wrong location, the term table above carries the full definitions: *`Add`*, *`Minimum` / `Maximum`*, *`F32Add` / `F32Multiply`*, and the four *`Select`* entries.

| Name | Layer and construct | Implementation spelling | Maturity |
|---|---|---|---|
| `Add` | Semantic tensor elementwise addition | `tiler::add-f32@1`, facade `F32Add` | Registered and implemented |
| `Add` | Shape-expression canonical addition | None | Adopted, unimplemented |
| `Add` | Scalar operation a tensor `Add` lowers into | `tiler.scalar::add-f32@1`, `PointwiseScalar::Add` | Implemented |
| `Add` | Reference-oracle addition used to check a result | `F32BinaryReference::Add`, `StandardScalarBinaryF32::Add` | Implemented |
| `Binary` | ABI expression: one checked binary application over two earlier arena nodes | `ExprNode::Binary` | Implemented, public |
| `Binary` | Structured kernel operation applying a pure `BinaryOp` | `OperationView::Binary` | Implemented, public |
| `Binary` | Metal emission diagnostic family | `MetalOperationFamily::Binary` | Implemented |
| `Compare` | Shape predicate primitive of the typed `ShapePredicate` language | None | Adopted, unimplemented |
| `Compare` | Structured kernel operation producing a predicate | `OperationView::Compare` over `CompareOp::IndexLessThan` | Implemented, public |
| `Constant` | Semantic tensor bit-preserving constant family | `tiler::constant-f32@1`, facade `F32Constant` | Registered and implemented |
| `Constant` | Shape constraint literal extent term | `ExtentTerm::Constant` | Implemented, crate-private |
| `Constant` | Index-expression exact integer constant | `IndexExprView::Constant` | Implemented, public |
| `Constant` | Scalar operation a tensor constant lowers into | `tiler.scalar::constant-f32@1` | Implemented |
| `Constant` | Structured kernel operation defining a typed immediate | `OperationView::Constant` over `KernelConstant` | Implemented, public |
| `Constant` | Structured kernel *address space* — read-only memory constant for the whole dispatch, not an operation | `AddressSpace::Constant` | Implemented, public |
| `Equal` | Shape constraint asserting equality of two extent terms | `ExtentRelation::Equal` | Implemented, crate-private |
| `Equal` | ABI expression unsigned equality | `AbiBinaryOp::Equal` | Implemented, public |
| `FloorDiv` | Shape-expression floor division, rounding toward negative infinity including for negative operands | None | Adopted, unimplemented |
| `FloorDiv` | Index-expression Euclidean floor division by a positive constant | `IndexExprView::FloorDiv` | Implemented, public |
| `F32Add` / `F32Multiply` | Semantic authoring facades — see the term rows above | `standard_operations.rs` | Implemented, public |
| `F32Add` / `F32Multiply` | Structured kernel binary operations — see the term rows above | `BinaryOp` | Implemented, public |
| `Minimum` / `Maximum` | Semantic tensor extrema family — see the term rows above | None | Named by an adopted matrix row only |
| `Minimum` / `Maximum` | ABI expression unsigned extrema — see the term rows above | `AbiBinaryOp` | Implemented, public |
| `Not` | Shape predicate negation, typed and normalized where that does not cause uncontrolled expansion | None | Adopted, unimplemented |
| `Not` | ABI expression predicate negation | `AbiUnaryOp::Not` | Implemented, public |
| `Select` | Four constructs — see the four *`Select`* term rows above | `ExprNode::Select` only | One of four implemented |
| `Unary` | ABI expression: one checked unary application over an earlier arena node | `ExprNode::Unary` | Implemented, public |
| `Unary` | Structured kernel operation in the proposed bounded initial set | None | Proposed, unimplemented |

Three invariants hold over `crates/` and are more durable than a hit count, which the relocation of the ABI domain has already made stale once. Every `Unary` is the ABI construct; every `Not` is the ABI construct; and every `Minimum` or `Maximum` that is an identifier rather than English prose in a comment is the ABI construct. No such invariant holds for `Constant`, `Binary`, `Equal`, or `Add`, each of which has at least two implemented senses in `crates/`, and `AddressSpace::Constant` and `OperationView::Constant` are declared in one file.

Never write any of these names unqualified in normative text or diagnostics. Name the layer, or use the spelling that already carries it.

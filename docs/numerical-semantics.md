---
schema: "tiler-doc/v1"
id: "tiler.contract.numerical-semantics"
kind: "contract"
title: "Numerical semantics"
topics: ["numerics", "semantics", "dtypes", "accuracy"]
contract_status: "mixed"
implementation_status: "partial"
evidence: ["tiler.research.numerics.affine-quantization-semantics","tiler.research.numerics.bf16-computation-accumulator-and-conversion","tiler.research.numerics.dtype-identity-admission-policy","tiler.research.numerics.dtype-resolution-precedents","tiler.research.numerics.float-to-integer-conversion-precedents","tiler.research.numerics.floating-point-extrema-precedents","tiler.research.numerics.integer-division-precedents","tiler.research.numerics.integer-overflow-precedents","tiler.research.numerics.mature-dtype-taxonomy","tiler.research.numerics.operation-conformance-matrix","tiler.research.numerics.quantization-ir-precedents","tiler.research.numerics.quantized-value-and-transform-contract","tiler.research.numerics.reduction-semantics-and-legality","tiler.research.numerics.region-accuracy-contract","tiler.research.numerics.sound-region-analyzer-spike","tiler.research.numerics.transcendental-accuracy-precedents","tiler.research.numerics.transformer-nonlinear-normalization-and-reductions"]
ticket: "numerical-policy-contract"
---

# Numerical semantics

**Status:** accepted framework; initial product-profile operation tuples remain open

Tiler optimizes floating-point and integer programs whose algebraic identities
do not automatically imply machine-level equivalence. Numerical policy is part
of semantic meaning, legality, plan identity, artifact identity, and testing.

## Ownership boundary and traceability

This document owns Tiler's target-independent numerical meaning and legality.
It does not claim that every recognized dtype or semantic tuple is implemented.
The accepted decisions are [ADRs 0009–0042](decisions/README.md) together with
ADRs 0055, 0059, 0060, 0062, and 0066, with primary support in the
[numerical research corpus](research/numerics/). Implementation support remains
capability-gated and unmeasured unless a linked experiment says otherwise.

## Three parts of the contract

Numerical meaning is divided into three machine-checkable parts: operation
semantics, optimization permissions, and execution guarantees. Each part below
is a subsection of this section; later sections elaborate individual contracts
rather than adding a fourth part.

### Operation semantics

Each scalar or reduction operation defines its dtype signature, casts,
identity and empty-domain behavior, division/modulo behavior, and min/max NaN
and signed-zero contract. These are properties of the operation, not global
optimizer switches.

#### Resolved numerical typing

Every compilable semantic tensor value has a resolved value dtype. Every
operation has a resolved numerical signature sufficient to define its
observable computation. Tiler does not apply an ambient global promotion table
after semantic admission.

Ordinary elementwise operations are homogeneous by default. Frontends may
offer PyTorch-like, JAX-like, strict, or custom promotion, weak-scalar, and
autocast policies, but they lower the result to explicitly typed constants,
conversions, operands, and results before optimization.

For Rust construction, ADRs 0059 and 0062 expose exact `Value<T>` handles, where
`T` binds through the frozen registry to a complete shape-independent resolved
value type rather than necessarily to one primitive `TypeKey`. This uniformly
covers nominal, parameterized, and encoded-numeric tensor values. The contract
requires result-affecting promotion, accumulator, conversion, rounding, and
output choices as explicit operation signature/contract inputs. A typed builder
returns the statically resolved result handle. Runtime-parsed frontends resolve
the same semantic signature through the operation registry; they do not gain a
second ambient promotion system or authority to invent result types.

Operations with intrinsic mixed-precision behavior use specialized typed
signatures. Depending on the operation, these may distinguish:

- tensor value dtype;
- per-operand computation or input precision;
- accumulator dtype;
- result value dtype;
- conversion and rounding behavior;
- reduction-order or contraction permissions;
- a required numerical algorithm.

These are semantic roles rather than one universal `dtype` field or a bag of
optional attributes attached to every operation. Physical storage encoding is
separate again: a fused implementation may avoid materializing a typed edge,
but it must still reproduce every semantic conversion on that edge.

The evidence and cross-system differences behind this boundary are recorded in
[Dtype resolution and mixed-precision precedent](research/numerics/dtype-resolution-precedents.md).
The broader inventory of scalar types, computation formats, numeric
interpretations, and storage encodings is maintained separately in the
[mature tensor dtype taxonomy](research/numerics/mature-dtype-taxonomy.md); that
inventory is not an implementation support promise.

The focused [quantization IR precedent review](research/numerics/quantization-ir-precedents.md)
and [quantized value contract](research/numerics/quantized-value-and-transform-contract.md)
define a hybrid of first-class quantized tensor values and explicit
assembly/conversion operations. Static scheme structure belongs to the type
contract; concrete static or runtime parameters remain graph operands.

**Fact — encoded logical values now carry generic ordered component declarations.** `EncodedNumericContract` records each component's stable scheme-local role, complete resolved value type, and typed relationship between the component shape and the logical value shape. Component order is semantic and remains distinct from role identity; neither graph operand position nor kernel ABI slot position can substitute for the role. The generic constructor rejects duplicate roles, while each scheme authority remains responsible for admitting its exact role set, component types, and shape relations.

**Fact — the first implemented parameter-map form is deliberately only per-tensor.** `ParameterIndexMap::per_tensor` maps every logical data coordinate to one rank-zero parameter and is the sole producer-backed map form. The type is an architectural seam for future workload-selected per-axis, block, group, hierarchical, codebook, mask, or outlier mappings, not a claim that those mappings exist. No placeholder block size or generic grouping field is carried before a producer can define and validate it.

**Fact — the standard semantic registry admits two complete strict-affine proof instances.** `tiler::u4@1` and `tiler::u8@1` are logical unsigned integer code types, distinct from `tiler::strict-affine@1` encoded values whose expressed and computation type is `tiler::f32@1`. Each encoded value has the ordered roles codes, scale, and zero point; codes have the logical value shape, while scale and zero point use the per-tensor rank-zero map. `AssembleStrictAffine`, `QuantizeStrictAffine`, and `DequantizeStrictAffine` are separate pure operations, so association, conversion, and materialization cannot collapse into one implicit dtype convention.

Representability and operation support are separate. A recognized dtype may
appear on tensor values and participate in explicitly compatible operations
without implying that arbitrary arithmetic, reference evaluation, optimization,
or backend lowering exists for it. Each operation admits a complete typed
signature; unsupported combinations are rejected with the missing capability.

All canonical dtypes, including Tiler built-ins and third-party extensions,
use the same namespaced, versioned nominal identity mechanism. An ergonomic
API name such as `DType::F32` denotes the durable identity `tiler::f32@1`; it is
not a distinct identity system. Structural descriptions such as bit width,
exponent width, and fraction width are descriptor facts and are not sufficient
identity because formats can differ in bias, special values, and encoding.
IEEE decimal32, decimal64, and decimal128 are built-in recognized logical
dtypes. Their DPD and BID representations are explicit storage encodings, not
separate dtype identities. Recognition does not imply arithmetic or backend
support; see ADR 0035.

IEEE binary16/32/64/128, BF16, OCP OFP8 E4M3/E5M2, and the OCP MX
FP6/FP4/E8M0 constituents are also built-in recognized logical formats under
ADR 0036. Compound MX tensors remain scheme-typed quantized values rather than
scalar dtypes, and TF32 remains an execution-precision contract.

Complex uses the nominal parameterized identity
`tiler::complex@1<ComponentTypeKey>`. The initial admitted component types are
f16, f32, and f64. Width-based frontend names are aliases, storage layout is a
physical contract, and operation support remains specific to the complete
complex instance and signature; see ADR 0037.

OCP MXFP8, MXFP6, MXFP4, and MXINT8 are built-in compound scheme identities,
not scalar dtypes. Their element codes and shared E8M0 scales are ordinary
graph operands of one first-class encoded value; block membership and
scale-selection semantics are part of the scheme contract. See ADR 0038.

Fixed-width integer add, subtract, and multiply use explicit overflow-
specialized semantic families. Initial recognized contracts include wrapping,
saturating, checked, and widening forms. Required-no-overflow is a discharged
proof or runtime-validation obligation, never ambient undefined behavior or
poison. See ADR 0039 and the
[precedent review](research/numerics/integer-overflow-precedents.md).

### Optimization permissions

The program carries a granular policy ceiling: the maximum numerical freedoms
the user authorizes anywhere in the program. Every operation also carries its
resolved effective permissions for the dimensions applicable to that
operation. An operation's permissions may be stricter than the program ceiling
but can never exceed it.

Conceptually, resolution combines the program ceiling, any tighter per-operation
request, and the operation's declared capabilities:

```text
effective_permissions(op)
  = program_ceiling
  ∩ per_operation_restrictions(op)
  ∩ operation_capabilities(op)
```

The resulting canonical contract is granular. The implemented scalar-arithmetic
contract resolves **eleven** governed dimensions, in this canonical order:

```rust
struct ScalarNumericPolicy {
    // The contract speaks for exactly one arithmetic type; see below.
    arithmetic: ArithmeticType,
    input_subnormals: SubnormalMode,          // Preserve | FlushToZero { zero_sign }
    result_subnormals: SubnormalMode,
    contraction: NumericalPermission,         // Forbidden | Permitted
    reassociation: NumericalPermission,
    permutation: NumericalPermission,
    signed_zero: NumericalPermission,
    reciprocal_transform: NumericalPermission,
    approximate_intrinsics: ApproximationEnvelope,
    nan_assumptions: ExceptionalValueAssumption,
    infinity_assumptions: ExceptionalValueAssumption,
    materialization_rounding: MaterializationRounding,
}
```

Operand permutation, signed zero, and the two exceptional-value assumptions are
first-class dimensions of the resolved contract rather than only prose elsewhere
in this document; the [Reductions](#reductions) section defines what separates
permutation from reassociation, and both are resolved here. The two subnormal
dimensions stay independent even where a target couples them. Distributivity is
deliberately absent — the [Distributivity](#distributivity-is-outside-the-order-contract)
subsection states why, and adding a field here would contradict a decision rather
than answer a reserved question: [ADR 0095](decisions/0095-decline-a-distributivity-permission.md)
declines the permission outright.

Two things the sketch names but does not resolve are properties of the contract
rather than dimensions a target declares honourability over: its own versioned
governed key, and the canonical arithmetic-NaN bits. A key names the governing
contract and the bits are a produced value, so neither is a behaviour a profile
can be asked whether it honours.

**The dense contract is `f32` and is not a general policy shape.** Every
resolution above is stated for exactly one `ArithmeticType`, because subnormal
behaviour is measurably per-dtype: one Apple row flushes in `f32`, preserves in
`f16`, and flushes in `bf16`, so a dtype-free contract would state something
already known to be false for one of them. Nothing here generalizes to integer,
boolean, quantized-compound, or any other future policy family — those have their
own semantics elsewhere in this document, and this dimension set is not a
template for them.

**Fact — a caller resolves the dimensions, and does not choose from a list.** The implemented boundary was a four-value preset enumeration (strict, flush-to-zero, relaxed, permit-reassociation), and it is now `tiler_compiler::session::NumericalContract`, composed one dimension at a time from a strict base. The elimination is on the record: every axis the enumeration spanned had already been decided independent — [ADR 0011](decisions/0011-per-operation-numerical-permissions.md) holds that one permission never implies another, [ADR 0014](decisions/0014-reassociation-vs-permutation.md) split ordered regrouping from contributor permutation on evidence, [ADR 0080](decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) added a third independent dimension — and the target side already declares honourability and refuses per dimension. The enumeration was the one point-shaped surface left, and it produced its predictable failure the first time real hardware needed a corner no preset named: Apple `f32` arithmetic flushes subnormals in every measured math mode, both reassociating presets required them preserved, and so no parallel reduction was statable on the one measured Apple row — for want of a contract a caller could name, not for want of a target fact. Tom decided the direction on 2026-08-01 in the live session, relayed by the coordinator who witnessed it.

**Fact — omission never widens.** A composition starts at the strict resolution of every dimension and a caller resolves the ones it means to move, so an unstated dimension is forbidden rather than unconstrained, and a dimension added to the vocabulary later arrives forbidden in every contract written before it existed. This is the same fail-closed direction the [honesty rule](#the-honesty-rule-in-both-directions) states for a target's declaration, read from the caller's side.

**Fact — the contract key is derived, not chosen.** A contract's key is the canonical, injective encoding of its dimension vector under the versioned domain `tiler.contract.f32.v2`: the arithmetic type's tag, the canonical arithmetic-NaN bits, and then each dimension's tag and behaviour in the canonical order above, rendered as lowercase hex. Injectivity is what the key has to carry, because several authorities identify a contract by its key *alone* with no dimension beside it — the scheduled region's `profile_key`, the compiler's fusion-legality content identity, and its semantic-occurrence contract identity — so two contracts sharing a key would give two stated meanings one artifact and one cache entry. The predecessor scheme was four hand-written names (`tiler.strict-f32.v1` and siblings); those named exactly the four presets and could not name the rest of the space, which is why the domain stepped rather than the names growing. Injectivity over the whole statable space is checked exhaustively rather than sampled, in `crates/tiler-compiler/src/request.rs`.

**Fact — five named contracts are retained, and they are points rather than the space.** Strict, flush-to-zero, relaxed, permit-reassociation, and flush-and-reassociate. They keep their documentation value as named constants; nothing selects between them and they are not ordered by strength.

**Fact — why permit-reassociation is a point rather than a setting of relaxed.** Ordered regrouping and fused-multiply-add contraction are independent dimensions by [ADR 0015](decisions/0015-fma-vs-contraction.md), and until `admit-a-reassociating-contract-without-contraction` the only registered contract permitting the first also permitted the second. A caller wanting a split reduction over exactly the strict reading's rounding boundaries therefore could not state its program at all, and the compiler's own fusion-legality authority refused every mixed multiply/add region under the relaxed contract because a permitted-but-unrealized contraction leaves the delivered arithmetic undetermined. Permit-reassociation resolves `reassociation` to `Permitted` and every other dimension — contraction included — at the strict resolution, so the delivered realization stays pinned. It is a different meaning rather than a weaker one, and a program under it may return different bits from both the strict and the relaxed reading.

#### Coherence, enumerated rather than discovered

Composition lets a caller state combinations a four-value enumeration could not, so the combinations that are *not* contracts are named here rather than found in the field. The list is deliberately small, and the eliminations matter as much as the survivor: a reader must be able to refute the list rather than only read it.

**What survives — exactly one.** A contract may not assert a value-domain absence on evidence it is not the author of. The [value-assumption provenance classes](#value-assumptions-and-validation) define compiler-proven as derived soundly by the compiler from verified producers, constants, or analysis, and runtime-validated as established by a guard that runs before any plan relying on it executes. Neither is a claim a caller is in a position to make — the first is a conclusion this compiler reaches, the second names a guard this build neither emits nor checks — so a caller-stated `AssumeAbsent` carries caller-declared-unvalidated provenance, and any other class is refused by name at construction. This is the same rule the `x / x` example above states: the required domain must be proven or validated, and a caller's unchecked claim is not either.

**What was eliminated, with its derivation.**

- *Assumed-absent NaNs against the canonical arithmetic NaN pattern.* Not contradictory: the pattern governs a NaN the build *produces*, the assumption governs a NaN an *operand* may carry, and this document already keeps the two apart.
- *One exceptional value assumed absent and the other not.* Independent by ADR 0011; refusing the pair would re-couple two dimensions a decision separated.
- *Permitted signed-zero elimination beside a sign-preserving flush.* The flush's zero sign is carried on the flush behaviour precisely so no permission can leave it unspecified, so the two are independent by construction rather than in tension.
- *Forbidden signed-zero elimination beside a flush to always-positive zero.* A declared flush behaviour is a stated, checkable result rather than a rewrite; forbidding the *elimination* of a distinction does not forbid an operation whose defined result produces one zero.
- *Permitted contraction with forbidden reassociation, and permitted permutation with forbidden reassociation.* ADR 0015 separates fusing a multiply into an add from regrouping an operand sequence, and ADR 0014 separates permuting a reduction's contributors from regrouping them: a permuted sequence folded strictly left to right is a well-defined sum consuming no regrouping at all.

Coherence is about the statement alone. A contract this build cannot *realize* is a separate refusal naming the dimension, the behaviour the build realizes, and the operation that would consume it; a contract a *target* cannot honour is a third, reported per dimension by feasibility.

#### Statable exceeds tested, permanently

The number of statable contracts is the size of the dimension product, and this build's conformance evidence covers a handful of points in it. **That gap is permanent, and it is not closed by narrowing what a caller may say.** Narrowing was the previous design, and what it bought was an unstatable corner on real hardware rather than a smaller tested surface.

It is closed per dimension instead, by three gates in order, each of which fails closed:

1. **Coherence**, at construction: a self-contradictory vector is refused by name, per the enumeration above.
2. **Representability**, at the request boundary before any target is consulted: a dimension whose stated resolution no scheduled region can record is refused with the dimension, the behaviour this build realizes, and the first admitted operation that would consume it. This is a property of the build rather than of a profile, which is why it is assessed before a target.
3. **Feasibility**, per dimension, against the target's measured [honourability declaration](#per-dimension-honourability-and-how-it-composes-with-feasibility): an unmeasured resolution is `Unknown` rather than assumed, and an unhonourable one is a typed rejection naming the dimension, the arithmetic type, the required behaviour, the behaviour the target declares, and the declaring profile.

So an untested combination reaches a device only through a target that has *declared* every dimension it resolves. What a caller may state and what this build has measured are different sets by design, and the second never silently stands in for the first.

The sketch is descriptive rather than a committed API. A user-facing named
mode may initialize the program ceiling, but an overlapping `fast_math`
boolean is avoided. Frontends may expose per-region or per-operation controls;
those controls resolve to the same canonical per-operation representation.

Every rewrite and physical alternative declares the effective permission it
requires. Explain output identifies the program ceiling, the operation's
resolved permission, and the restriction that rejected an otherwise applicable
alternative. Backend flags are derived from the resolved operations and must
not silently enable additional transformations.

NaN-result semantics are distinct from permission to assume NaNs absent.
Infinity assumptions, signed-zero distinctions, reciprocal replacement,
approximate elementary functions, reassociation, and contraction are likewise
independent. One permission never implies another. A backend compiler switch
that couples several freedoms is usable only when every freedom it enables is
already authorized for the affected operations.

#### Value assumptions and validation

Every value-domain fact used for correctness has explicit provenance:

- **compiler-proven:** derived soundly from verified producers, constants, or
  analysis and usable without a runtime check;
- **runtime-validated:** established by a guard or validation computation before
  any plan that relies on it executes;
- **caller-declared but unvalidated:** recorded and explainable, but initially
  ineligible to justify a correctness-sensitive rewrite.

For example, replacing `x / x` with `1` requires more than a caller's unchecked
claim: the required nonzero, finite, non-NaN domain must be proven or validated.
Validation of tensor contents may require a full scan and is itself a costed
operation, not a free scalar guard.

An optimization guard changes only physical eligibility. If it fails, dispatch
selects another valid plan or the general fallback before dependent work begins.
It does not make a semantically valid input invalid. A semantic input
precondition is different: it defines the program's admitted domain, and its
failure produces a precise invalid-input diagnostic. The two kinds of predicate
remain distinct in IR, explanation, artifact routing, and testing.

### Execution guarantees

Execution guarantees state reduction-order constraints, determinism, index
overflow safety, and a conformance level. They constrain which physical plans
are valid even when two plans are algebraically equivalent.

## Exact and relaxed transformations

Exact logical normalization may compose index maps, remove identity operations,
and fold constants where bit-level semantics are preserved. It must not reorder
floating-point operations merely because they are associative over real numbers.

Relaxed policies may permit:

- arithmetic reassociation;
- tree rather than serial reduction order;
- fused multiply-add contraction;
- approximate transcendental intrinsics;
- reciprocal-based division;
- elimination of signed-zero or NaN distinctions.

Every rule declares which permission it requires. The optimizer cannot infer a
relaxed policy from a backend's default compiler flags.

Each name in that list means the dimension this document defines, not a broader vendor reading of the same word. `reassociation` is ordered regrouping of one same-operation operand sequence: `(a op b) op c` may become `a op (b op c)` while retaining the leaves and their order. A reduction's contributor sequence is one instance of that general rule. Reassociation does not extend to rewrites that change which products or other leaf values are formed, which consume the separate [distributivity](#distributivity-is-outside-the-order-contract) dimension and are not admitted.

### Implemented ordered-associativity rules and oracle boundary

**Fact — ordered associativity is declared by the operation and authorized by the contract.** The frozen definitions of `tiler::add-f32@1` and `tiler::multiply-f32@1` declare ordered associativity. That declaration says a rule may consider regrouping while preserving the ordered leaf sequence; it does not authorize the transformation by itself. The compiler additionally requires the operation's effective reassociation dimension to resolve to `Permitted`, and the governed strict and flush-to-zero contracts therefore decline these rules after semantic applicability while the governed relaxed and permit-reassociation contracts may admit them.

**Fact — the exact result-set oracle is bounded test evidence, not a compile-time authority.** For three through six scalar `f32` leaves, the conformance fixture enumerates every full binary grouping that preserves leaf order, evaluates each grouping through the independent semantic reference evaluator, deduplicates the exact output bit patterns, and requires the rewritten result to be a member of that set. It separately requires the rewrite to retain the original ordered leaf sequence. The fixture refuses an unreviewed leaf count rather than extrapolating beyond its exhaustive boundary. Ordinary compilation does not materialize tensor inputs or invoke this oracle; it relies on the operation-owned algebraic declaration, the checked semantic and numerical guards, structural rebuilding, and the rule's bounded conformance evidence.

**Fact — the implemented physical payoff is exact and deliberately narrow.** When an admissible effective numerical contract accepts one of the implemented reassociations, a one-input, one-output, three-leaf same-family `f32` add or multiply chain can compile through the verified bounded `PointwiseF32Expression` physical projection. That projection preserves every semantic arithmetic boundary and lowers each add or multiply followed by the required arithmetic-NaN canonicalization; it does not infer permission from the dtype or relax the operation contract. Separately, the strict-affine work described below carries one u4 dequantization through a role-addressed target-neutral schedule, structured kernel, verified kernel program, and neutral artifact round trip; that bounded structural proof neither makes `PointwiseF32Expression` generic nor supplies a safely runnable quantized backend. Other executable dtype and operation verticals must still reject until their complete numerical, lowering, runtime-validation, and target-realization contracts exist.

## Initial arithmetic rounding

Initial ordinary floating-point `Add`, `Subtract`, `Multiply`, and `Divide`
contracts use round-to-nearest, ties-to-even for each semantic operation.
Required `Fma` uses the correctly rounded fused result under the same direction.
Separate multiply and add operations retain two such rounding boundaries unless
their resolved contraction permission authorizes fusion.

This is an explicit operation contract, not an ambient hardware rounding mode.
Numeric conversions and transcendental operations continue to use their own
specialized rounding or accuracy contracts. Future directed-rounding arithmetic
can be added as new typed operation contracts; existing operations retain their
round-to-nearest, ties-to-even meaning, and older consumers reject unsupported
rounding contracts.

## Fused multiply-add and contraction

Tiler distinguishes a required fused multiply-add from optional contraction:

```text
Fma(a, b, c)       // one semantic rounding after a*b+c
Add(Mul(a, b), c)  // separate semantic multiply and add roundings
```

`Fma` is a dedicated semantic operation. A backend implements its
single-rounding contract natively, emulates it exactly, uses an already
permitted relaxation, or rejects the plan. It cannot lower required FMA to
separately rounded multiply and add merely because that is cheaper.

`Mul` followed by `Add` remains two semantic operations. Its resolved
contraction permission may authorize a rewrite or physical implementation using
FMA. Contraction is independent of reassociation: permission to contract the
existing pattern does not authorize algebraic regrouping to manufacture a new
pattern.

"Contraction" in this section is only ADR 0015's fused-multiply-add permission. A *tensor* contraction — summation over indices shared by two or more operands — is a reduction, and this permission governs exactly one thing about it: whether its per-contributor `accumulator + a * b` step may round once instead of twice. It says nothing about that reduction's order, and regrouping a chain of tensor contractions consumes the separate [distributivity](#distributivity-is-outside-the-order-contract) dimension, which no permission in this document grants.

## Transcendental accuracy

Every transcendental operation carries a resolved, operation-specific accuracy
contract. `Exp`, `Log`, `Sin`, and similar operations do not inherit an
accuracy choice from ambient compiler flags or a backend's default math
library.

ADR 0042 defines four discriminated forms: correctly rounded, faithful,
piecewise bounded, and immutable versioned named-elementary behavior. Bounded
clauses use exact rational tolerances and versioned absolute, relative,
absolute-plus-relative, or ULP metrics over explicit domains. The initial ULP
definition is `tiler::ulp-reference-gap@1`, matching the spacing definition
used by OpenCL. Exceptional values, signed zero, and input/result subnormals
remain orthogonal rather than being inferred from an error metric.

A frontend may expose named accuracy presets, but it resolves them before
canonical semantic admission. A rewrite, fusion choice, or backend intrinsic
is legal only when its implementation guarantee refines the resolved effective
operation contract. Every authorized relaxation has already resolved into that
canonical contract before optimization. Backend feasibility may report exact
native support, exact emulation, relaxed-only support, or rejection.

Approximate-intrinsic permission resolves to a maximum accuracy envelope before
semantic optimization, not a boolean or a later license to weaken meaning.
Proof, exhaustive finite-domain testing, or an applicable normative
guarantee can establish hard feasibility. Empirical qualification remains
labeled empirical and cannot establish an unmeasured worst-case bound.

**Fact — the implemented envelope vocabulary is governed and closed, not a free-form key.** `tiler_ir::schedule::ApproximationEnvelope` has exactly two resolutions today: `Forbidden`, keyed `approximation.forbidden`, under which every elementary function follows its own resolved accuracy contract; and `BackendElementary`, keyed `tiler::backend-elementary@1`, which is the backend-elementary conformance level named under [Conformance levels](#conformance-levels) and bounds the approximation by the backend's own *stated* accuracy, so a backend that states none cannot honour it. The two are not two settings of one field: `Forbidden` authorizes no envelope at all, which is a different claim from authorizing an empty one, and the type reports that difference by returning no envelope key for it rather than a sentinel. Closedness is what makes the dimension identity-safe — a named envelope enters the contract's canonical identity, where a tolerance spelled inline could be widened without changing it. The named relaxed contract authorizes `BackendElementary`; nothing in this build emits an approximate intrinsic, so that authorization is expressible and unconsumed. A third envelope is a new named resolution with its own key, never a re-reading of one of these two.

**A composite operation's own formula is part of its contract, not a choice left to whoever spells it.** [Transformer non-linear, normalization, and reduction contracts](research/numerics/transformer-nonlinear-normalization-and-reductions.md) derives the first worked instance: an admitted `Softmax` must pin whether the row maximum is subtracted before exponentiation, and whether the result divides by the denominator or multiplies by its reciprocal, because the alternatives differ observably in F32 — the first as a finite value against NaN, the second at measured discriminating elements. The reciprocal choice in particular is *not* an exercise of the `reciprocal_math` permission when it is the pinned formula; the permission governs replacing a stated division, and a contract that stated the wrong one would diverge while consuming no permission at all. The same rule reaches an admitted `RmsNorm`'s `eps` placement and its choice of a reciprocal square root, and an admitted activation's choice between two conventional spellings that measurement separates by one ULP.

Local operation contracts are mandatory and authoritative. The initial
optimizer does not redistribute an end-to-end error budget across operations.

A future optional region/output accuracy layer is additive rather than a
replacement for local semantics. A region goal must identify an observable
output, explicit reference semantics, an input and shape domain, an error metric
and tolerance, and its evidence class. It is a hard feasibility constraint:
cost optimization occurs only among plans demonstrated or explicitly accepted
to meet it.

No region goal silently overrides a local operation contract. Any future
delegation of internal accuracy to a region goal must be explicit and scoped.
Proof, empirical validation under a named test definition, and unknown status
remain distinct; empirical evidence cannot satisfy a sound-proof contract.

Tiler preserves the information a future analysis needs: semantic casts and
materialization boundaries, reduction topology, input/shape assumptions,
reference provenance, and resolved local numerical permissions. General graph
budget analysis remains out of initial scope because local ULP or relative
bounds do not compose safely through cancellation, sensitivity, correlation,
branches, or unbounded reductions.

## Reductions

A reduction definition includes:

- input dtype;
- accumulator dtype;
- output dtype;
- identity and empty-domain behavior;
- operation order guarantee;
- NaN and signed-zero behavior;
- deterministic or implementation-dependent result policy.

Canonical reduced axes are a nonempty, unique, sorted set. For an ordered fold,
contributors appear in ascending lexicographic order of reduced coordinates
using original logical axis order; `keepdims` affects result-shape lowering,
not contributor order. Input-to-accumulator conversion, optional seed
conversion, accumulator dtype, result conversion, empty behavior, and order
permissions all participate in semantic identity.

Accumulator dtype does not determine reduction semantics by itself. The order
contract independently states which serial or tree evaluations are permitted.
It represents reassociation and operand permutation as independent dimensions:

- **reassociation** changes grouping while preserving logical operand order;
- **permutation** changes logical operand order.

Granting reassociation does not grant permutation. Reassociation requires both
an operation capability supporting regrouping and an effective numerical
permission to use it. Permutation independently requires a commutative
operation capability and an effective numerical permission to reorder. A
physical schedule proves both properties separately.

The semantic order contract constrains the legal evaluation orders or result
set; it does not encode a concrete SIMD, threadgroup, or multi-pass reduction
tree. It must be able to distinguish concepts such as an ordered fold, a
deterministically selected legal order, and a reassociation-permitted result
set. Those names are illustrative rather than a frozen public enum.

The selected physical plan records the actual reduction topology, including
partitioning, tree shape, synchronization, and intermediate passes. That
topology participates in physical-plan and artifact identity. A scheduler may
choose it only when it satisfies the semantic order contract.

Changing from a serial reduction to a SIMD or threadgroup tree is a physical
alternative only when the numerical policy permits its evaluation order. F16
or BF16 inputs do not imply low-precision accumulation; promotion is explicit.

**Fact — the accumulator's width is observable, so it is an operation property and never a schedule or contract one.** [The BF16 computation and accumulator derivation](research/numerics/bf16-computation-accumulator-and-conversion.md) exhibits two folds of one contributor sequence, in one order, differing only in the type each partial is held at: `0x3f80` (BF16 `1.0`) followed by four copies of `0x3b00` (`2^-9`) returns `0x3f80` at a BF16 accumulator and `0x3f81` at a binary32 one, because one BF16 ulp at `1.0` is `2^-7` and each addend alone rounds away. With one contributor both return `0x3f80`, so the difference is accumulated rounding rather than a disagreement about the seed. **Inference — two consequences that are not restatements of each other.** A planner may not choose the accumulator, because two plans returning different bits are not two plans of one program and cost-based selection may never price meaning. And the resolved scalar-arithmetic contract may not carry it either: that contract speaks for exactly one `ArithmeticType`, and per-dimension honourability is keyed by an arithmetic subject, so an operation that stores at one width and accumulates at another names two subjects and no single contract can speak for it. The accumulator therefore lives where `CONTRACTION_F32_FACT_ACCUMULATOR_TYPE` already puts it — in the operation's registered definition facts, and hence in its identity.

### Distributivity is outside the order contract

Reassociation acts on one ordered sequence of leaves combined by the same operation: it holds those leaf values and their order fixed and varies only their grouping. Permutation holds the leaves fixed and changes their order. A reduction's contributor sequence is one instance of this distinction, and its reduction order contract supplies the canonical sequence. A rewrite that changes *which* products or other leaves are formed is outside both dimensions because it leaves no single same-operation operand sequence for either permission to govern.

Rewriting a tensor-contraction chain is the motivating case. For output `[i, l]`, `(AB)C` forms the rounded partials `T[i, k] = sum over j of A[i, j] * B[j, k]` and then sums the contributors `T[i, k] * C[k, l]` over `k`; `A(BC)` forms `U[j, l] = sum over k of B[j, k] * C[k, l]` and then sums the contributors `A[i, j] * U[j, l]` over `j`. The two contributor sequences share no value, are indexed by different axes, and neither is a grouping of the other; no common sequence exists of which both are groupings. The identity relating them is distributivity of multiplication over addition, `(x + y) * c = x * c + y * c`, which round-to-nearest floating-point multiplication does not satisfy. The conclusion does not depend on whether a contraction chain is a chain of binary operations or one multi-operand node: were such a node defined as the flat sum over `(j, k)` of `A[i, j] * B[j, k] * C[k, l]`, its contributors would be triple products that neither association ever computes, and factoring `C[k, l]` out of the `j`-sum would again be distributivity.

**Distributivity** is therefore a third numerical dimension, independent of reassociation and permutation. It authorizes exchanging a product of a sum for a sum of products in either direction, so it changes which values are multiplied and where roundings fall. It is additional to reassociation and permutation rather than a substitute for either: routed through the flat contraction form, a chain regroup also changes the nesting order over the flat reduction domain, and grouping the canonical lexicographic contributor order by the outer axis combines non-contiguous intervals — so the reduction-specific rule that reassociation without permutation may combine only contiguous contributor intervals in order makes permutation necessary too. Consuming distributivity would require both an operation capability declaring the algebraic property and an effective numerical permission to use it, as ADR 0014 requires of the other two. Granting reassociation does not grant it, and granting it does not grant reassociation or permutation; ADR 0011 already holds that one permission never implies another, and ADR 0015 settles the same shape of question for fused multiply-add by ruling that a permission over an existing pattern does not authorize manufacturing a new one. [ADR 0080](decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md) is the accepted decision this subsection derives from; it records the dimension, its independence, the requirement that a rejection name the missing dimension, and the reservation it deliberately left open. [ADR 0095](decisions/0095-decline-a-distributivity-permission.md) is the accepted decision that closed that reservation, and it closed it by declining.

No distributivity permission is admitted. The canonical policy sketched above has no such field, and `NumericalPermission` in `crates/tiler-ir/src/schedule/numerics.rs` supplies only the two general resolutions `Forbidden` and `Permitted` that each *declared* dimension takes — there is no distributivity dimension for either resolution to apply to. A contract composed in `crates/tiler-compiler/src/session.rs` resolves reassociation independently, which is why the implemented add and multiply rules may regroup their unchanged ordered leaves under a contract that permits it. It still cannot authorize a tensor-contraction association rewrite that forms different products. A rewrite that consumes distributivity is therefore rejected under every statable contract, and its rejection names the missing distributivity dimension rather than reporting a forbidden reassociation — the two are different explanations, and only the first avoids implying that a registered reassociation-permitting contract admits the rewrite. Whether to admit the dimension at all was a product choice that does not follow from these definitions, and it is no longer open: [ADR 0095](decisions/0095-decline-a-distributivity-permission.md) **declines it**, so no distributivity permission is admitted, the rewrite's rejection is a decided position rather than a pending one, and contraction ordering remains a planning question within one semantic contraction rather than a logical rewrite over a chain. The grounds are that no chain in the pinned workload asks for the regroup, that a permission nobody can spend would widen every contract to authorize nothing, and that admitting one later is purely additive — this derivation, this definition, and this rejection wording are exactly what an admission would build on. Its reopening trigger is the first workload whose *natural spelling* is a directly regroupable contraction chain, distinguished from one an optimizer might speculatively want. The dependent question — whether one permission covers both directions of the identity or the factoring and expanding directions are cut apart — does not arise under a decline and stays parked with [`decide-whether-distributivity-directions-share-one-permission`](../tickets/decide-whether-distributivity-directions-share-one-permission.md), live only if ADR 0095 is reopened. [Q-SEM-015](open-questions.md) indexed this as the third of its reserved choices and it is now closed.

### Empty domains and initial values

Each reduction operation declares its empty-domain result or declares an empty
domain invalid. Representative empty results include additive zero,
multiplicative one, `true` for `all`, and `false` for `any`; the exact typed
value is operation semantics, not a backend default.

Empty result, algebraic identity, and safe physical padding are separate facts.
A schedule may inject or replicate a padding value only when the operation
contract proves that doing so preserves the required observable semantics. For
example, strict floating sum may return `+0.0` for an empty domain, but adding
`+0.0` to the singleton value `-0.0` under round-to-nearest produces `+0.0`.
Therefore `+0.0` is not bitwise-neutral padding for that strict reduction even
though it is its empty result. Such a schedule tracks nonempty partials or uses
another proven construction; a signed-zero relaxation may admit more choices.

An optional explicit `initial` is a true reduction seed, not an empty-only
fallback. It is converted according to the resolved reduction signature and is
one logical contributor for every output reduction domain, including non-empty
domains. Thus `minimum([20], initial=10)` produces `10`, and a sum with
`initial=10` adds ten exactly once.

This distinction constrains physical scheduling. A non-identity seed cannot be
copied into every SIMD lane, threadgroup, or partial reduction. A proven
replicable padding value may be copied only under the conformance contract for
which neutrality was established; an arbitrary initial value remains exactly
one logical contributor even when the permitted topology reassociates work.

Parallel partials carry `has_value` unless nonemptiness or observably neutral
padding is proved. Reassociation without permutation may combine only
contiguous contributor intervals in order; lane-strided partials generally
permute contributors and require the independent permission. Cross-kernel
scratch preserves accumulator bits, including contracted subnormal and NaN
behavior. Narrowing, flushing, or NaN rewriting in scratch is an explicit
semantic conversion, never a cost-only storage choice.

An identity-less reduction such as the initial `minimum`/`maximum` contract is
valid only with an explicit initial value or a proven/runtime-validated
non-empty domain. Otherwise a statically empty graph is rejected during
verification and a dynamically empty semantic precondition produces a precise
invalid-input error before dependent work begins. Empty-only fallback behavior,
if later needed, is a separate explicitly named operation or conditional rather
than an alternate meaning of `initial`.

An unqualified `deterministic` boolean is not a complete contract. The initial
scoped guarantee is **plan deterministic**: identical input bits and runtime
bindings, executed through the same artifact digest and selected plan variant
in the same declared target environment, produce identical output bits. The
physical plan must reject timing-dependent atomics or other execution choices
that can violate this promise.

**Portable bitwise** is a separate, stronger conformance level: identical
inputs produce identical output bits across every target conforming to that
contract. It may substantially restrict legal operations, elementary
functions, and physical schedules. Recompilation may select a different
deterministic topology, so plan determinism does not promise equal results
across different artifact identities.

## NaN result bit patterns

Portable-bitwise arithmetic canonicalizes NaN results to one dtype-specific,
versioned quiet-NaN bit pattern. Exact payload propagation is not implicitly
part of that conformance level. This makes arithmetic that produces NaN
portable and bitwise testable rather than allowing a backend to select any NaN
payload.

The bounded first `f32` prototype profile names
`tiler::canonical-arithmetic-nan-f32@1` with exact quiet-NaN bits
`0x7fc00000`. Ordinary `f32` Multiply and Add replace any NaN they produce with
that pattern. Strict `f32` Sum uses the same Add rule after every combine and
applies the canonicalization at its result boundary even when the contributor
sequence is a singleton. The redundant result-boundary rule prevents an
uncombined input payload from leaking through an arithmetic reduction.

Canonicalization applies according to each operation's semantic family; it is
not a blanket rewrite of stored tensor bits. Operations defined to preserve or
select existing bits, including views and bit-preserving copies, preserve an
input NaN payload. Numeric conversions use their resolved conversion contract.
Constants retain their declared bit pattern until an operation's semantics
produce a new value. The named profile and exact canonical bits participate in
semantic, plan, artifact, and cache identity.

Other conformance modes may explicitly request operand-payload propagation or
permit any quiet NaN. Those choices are typed operation contracts and affect
plan feasibility, reference evaluation, determinism, and artifact identity.
No mode inherits NaN payload behavior from a backend default.

## Subnormal inputs and results

Subnormal handling has two independent dimensions:

```text
SubnormalContract {
  inputs:  Preserve | FlushToZero { zero_sign },
  results: Preserve | FlushToZero { zero_sign },
}
```

Input flushing treats an existing subnormal operand as zero before arithmetic.
Result flushing replaces a newly produced subnormal result with zero. A flush
states which zero it produces, on the behaviour itself: binary32 has two zeros
and they are observably different values, so a flush that leaves the sign to be
resolved elsewhere is under-specified (ADR 0076 item 1, ADR 0019 as amended).
The sign never follows an ambient target mode. Should the resolved contract
later gain a signed-zero dimension, that dimension constrains the stated sign
and does not supply it.

Portable-bitwise execution preserves both input and result subnormals. Other
contracts may permit either or both forms of flushing. Some targets expose only
a coupled mode or cannot realize every combination; that is reported as native,
emulated, relaxed-only, or unsupported backend feasibility rather than
collapsing the semantic dimensions.

A declared resolution is compared against a target's behaviour only where it is
observable. When a program's value domain keeps every operand and result of its
arithmetic in one type out of the subnormal range, both resolutions of that
type's two dimensions return identical bits, so a target resolving them
differently is not a gap. That is a *discharged* obligation, not a weakened
contract: the declaration is unchanged, and the same declaration on a program
whose domain nothing bounds is still refused. The discharge is
`tiler_ir::schedule::SubnormalFreedom`, derived from a verified scheduled
program and never declared by a producer, and it is keyed by arithmetic type
because the derivations behind it are. The only one implemented is the
strict-affine decode's, recorded with the rest of that profile below.

## Floating-point exception observation

The initial numerical contract is explicitly value-only: floating-point
exception cases produce the operation's resolved result value and do not expose
an ambient status flag or synchronous trap. This is a `RaiseNoFlag`-style
contract, not an omission whose meaning may be inherited from a host language,
compiler, or device. Division by zero, invalid operations, overflow, and similar
cases still have defined value semantics through the operation's NaN, infinity,
signed-zero, conversion, and conformance contracts.

Diagnostics that are ordinary data can remain pure. For example, a future
`DivideWithStatus` operation could return `(result_tensor, exception_mask)` as
two explicit tensor results. Because the status is a value, ordinary use-def,
fusion, and dead-code rules remain sufficient.

True observation or mutation of a floating-point environment is different. A
sticky flag, trap, or ordered clear/read operation is an effect: it introduces
ordering, liveness, and partial-execution obligations that tensor dataflow alone
cannot represent. Supporting it later requires an explicit versioned
effect/resource-token value kind and effect signature, plus corresponding
optimizer, verifier, runtime, ABI, and artifact rules.

The initial pure graph does not implement those rules, but its compatibility
contract reserves them as additive extensions. Existing tensor values and pure
operations retain their current meaning. Serialized programs and artifacts
identify the exception-observation/effect model they use; an older compiler or
runtime rejects an unsupported future model rather than interpreting it as
`RaiseNoFlag`.

## Min and max

Tiler represents two distinct floating-point operation families:

```text
Minimum / Maximum
    if either operand is NaN: NaN

MinimumNumber / MaximumNumber
    if exactly one operand is NaN: the numeric operand
    if both operands are NaN: NaN
```

Both families deterministically order `-0.0 < +0.0`. Therefore minimum of
opposite-signed zeros is `-0.0`, and maximum is `+0.0`. Under portable-bitwise
conformance, a produced NaN follows the canonical arithmetic-NaN contract.

These are separate semantic operations, not one `Min`/`Max` operation with a
backend-selected mode. Number preference changes observable results and is not
merely permission to assume NaNs absent. A separate signed-zero relaxation may
permit either zero where authorized, but it does not change the operation's
canonical strict semantics.

Elementwise and reduction forms name the same scalar semantic family while
retaining their separate reduction identity, seed, and order contracts. Rewrite
rules declare the exact family they preserve. Operand commutation, tree
selection, clamp formation, and ReLU recognition may proceed only when NaN and
zero-tie behavior remain valid.

A backend intrinsic is selected only when its full behavior agrees. In
particular, Metal `fmin`/`fmax` are number-preferring and their signed-zero
result can depend on operand order; they are not an exact implementation of
strict `Minimum`/`Maximum` or deterministic-zero `MinimumNumber`/
`MaximumNumber` without a fixup or a matching authorized relaxation.

## Constants

Constants are represented by typed bit patterns or by a documented canonical
form. Hash and equality behavior must agree for positive/negative zero and NaN
payloads. Text source round-tripping must not silently alter a constant's value.

## Integer and index arithmetic

Data-operation overflow and address/index overflow are separate policies.
Fixed-width data arithmetic names an explicit wrapping, saturating, checked,
widening, or future versioned operation family. Required-no-overflow is a
proven or runtime-validated precondition, not poison or undefined behavior.
The operation-extension mechanism is not sealed to the initial family set.
Index expressions must not wrap into a valid-looking address. Shape products,
stride products, offset additions, and narrowing conversions are checked
statically or protected by runtime guards.

Division and remainder use explicit signed truncating, floor, Euclidean,
ceiling, or canonical unsigned families. Their quotient rounding, matched
remainder sign/range, zero-divisor behavior, and signed quotient overflow are
semantic contracts. Exact division adds a validated divisibility precondition.
Standalone signed `MIN rem -1` is valid zero even when a target's combined
divide/remainder instruction cannot implement it directly. Simplification
passes preserve these semantics. See ADR 0040 and the
[precedent review](research/numerics/integer-division-precedents.md).

## Casts

Casts are semantic operations with resolved, typed conversion contracts. Source
and destination dtype alone are insufficient to define the result. A contract
contains only the dimensions relevant to its conversion family; it is not one
universal bag of optional fields.

Initial conversion families include:

- floating-point widening and narrowing;
- floating-point to integer and integer to floating-point;
- integer widening and narrowing;
- quantization and dequantization;
- bit reinterpretation, as an operation distinct from numeric conversion.

Float-to-integer conversion distinguishes strict rounded, exact, ordered
saturating, and explicit total saturating NaN-to-zero families. Rounding is
named independently. Saturation determines endpoint behavior for ordered
values and infinities but does not by itself determine a NaN mapping. Rejecting
families use semantic preconditions and ADR 0033 enforcement; NaN-to-zero is a
separate compatibility contract. See ADR 0041 and the
[precedent review](research/numerics/float-to-integer-conversion-precedents.md).

As applicable, their contracts define:

- out-of-range float-to-integer conversion;
- NaN to integer;
- narrowing integer conversion;
- floating-point rounding;
- overflow behavior, signed-zero preservation, NaN handling, and subnormal
  handling;
- backend feature-dependent formats.

Named presets may provide concise frontend ergonomics, but canonicalization
resolves them to versioned typed contracts before semantic optimization. No
conversion inherits ambient frontend, compiler, or device defaults.

### Floating-point widening and narrowing, derived at the BF16/binary32 pair

The first float-to-float pair a workload reaches is BF16 and binary32, and its two directions are asymmetric enough to fix the family's shape. The complete derivation, its eliminations, and its worked examples are in [BF16 computation, accumulator, and conversion](research/numerics/bf16-computation-accumulator-and-conversion.md), and [ADR 0091](decisions/0091-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md) is the accepted decision the facts below now rest on; the results this contract depends on are below. Nothing here is registered: no conversion key exists, and the `Cast and convert` family remains at its stated rung.

**Fact — the two directions carry disjoint field sets, so they are two families rather than one with a direction field.** BF16-to-binary32 widening is exact and total: the two formats share an exponent width of eight and therefore a bias, and BF16's seven trailing significand bits are a prefix of binary32's twenty-three, so the conversion is a sixteen-bit shift under which a normal stays normal, a **subnormal stays subnormal**, each zero and infinity maps to itself, and a NaN payload zero-extends with its quiet bit in place. All 65,536 encodings are checked against an independent field decode with zero disagreements. Under the rule ADR 0041 already applies to exact float-to-integer conversion, an exact conversion carries no rounding rule — so the widening contract has no rounding, overflow, or NaN-mapping field, and one that carried a field is malformed rather than redundant.

**Fact — this exactness is a property of BF16's parameters and its derivation does not transfer.** Binary16's widening to binary32 is also exact, but by a *different* argument: its exponent range is strictly inside binary32's rather than equal to it, so it renormalizes and every binary16 subnormal becomes a binary32 **normal**. That difference has a measured consequence — findings 24 and 25 of [the Apple numerical behaviour record](research/apple-targets/numerical-behaviour.md) record `bf16` arithmetic flushing subnormals on the qualified Apple9 row where `f16` arithmetic preserves them, and attribute the split to exactly this exponent field. F64 and F128 have no wider carrier at all. A dtype-addition recipe must therefore not assume a lossless widening exists, and must not reuse this derivation for the format that arrives next.

**Fact — the narrowing direction has four decisions, each of which changes an answer, and one of them is forced.** Over 65,536 binary32 patterns covering all of `[1, 2)`: round-to-nearest-ties-to-even differs from truncation to the high sixteen bits in 32,704 of them, and from ties-away in 64. Overflow is not a formality — binary32's largest finite magnitude exceeds BF16's overflow threshold, so `0x7f7fffff` narrows to a positive infinity, and a narrowing contract stating no overflow rule says nothing about the whole top binade of its source. The forced decision is NaN: preserving a payload prefix is **not total**, because the signalling binary32 NaN `0x7f800001` carries its payload only in the low sixteen bits and truncates to `0x7f80`, the infinity encoding. Canonicalizing to the dtype's versioned quiet-NaN pattern is uniform and is what the [NaN result bit patterns](#nan-result-bit-patterns) rule already requires of portable-bitwise arithmetic; a payload-preserving narrowing remains a separate named family, exactly as ADR 0041 keeps NaN-to-zero separate from ordered saturation. Subnormal results underflow gradually and signed zeros are preserved; whether a *target* flushes is a numerical-honourability fact of that target's profile row and never a field of the conversion.

**Fact — an internal accumulator is an instance of the no-implicit-promotion rule rather than an exception to it, and for this pair the two spellings agree bit for bit.** ADR 0009's own alternatives-considered rejects "requiring graph-level casts for every scalar step inside a reduction" as encoding an operation's internal scalar iteration in the public tensor graph, so a reduction or contraction may state a wider accumulator in its own signature. At a *pointwise* boundary the explicit spelling is not merely permitted but exactly equivalent here: widening is exact, and an exact product of two BF16 values needs at most sixteen significand bits where binary32 holds twenty-four, so a widen/evaluate/narrow route and one exact BF16 rounding agree on all 524,288 checked multiply and add cases. The conversion family is therefore the prerequisite and a BF16 accumulating key is not; admitting the key first would add an identity whose behaviour the conversion family already reproduces.

**Fact — a fused BF16 multiply-add is the exception, and its promoted realization is not its contract.** The same route is *not* exact for a fused multiply-add, because the double-rounding bound that covers `+ − × ÷` and square root does not cover it: on operands `0x3fc0`, `0x3fb2`, `0xb300` one correctly rounded BF16 fused result is `0x4005` and the widen/fma/narrow route gives `0x4006`, and a bounded sweep finds 21,546 such triples in 262,144. **Measurement — the promoted route is the only one available on the measured row.** Finding 29 records `metal` rejecting `bfloat v6 = fma(v3, v4, v5)` on offline `metalfe-32023.883` under Xcode 26.6 and macOS 27.0 build 26A5388g, because MSL has no `bfloat` overload. **Inference.** A semantic operation naming itself a fused BF16 multiply-add would state a single-rounding BF16 contract that nothing can deliver, which is the erasure of observable rounding ADR 0015 exists to prevent. What is admissible, when a workload asks for it, is a mixed-precision operation whose facts state binary32 computation over exactly widened operands, one binary32 rounding, and one narrowing — under a name that does not imply the other.

**Measurement — an unfused BF16 guarantee is currently unhonourable on the measured Apple runtime path, and that is an explicit refusal.** Finding 30 records the runtime compiler contracting a written multiply/add pair under `relaxed` and `fast` at all three widths whatever the offline selection says, and finding 10 records that `MTLCompileOptions` exposes no contraction property at all. So a BF16 contract resolving contraction to `Forbidden` contributes a **disproved** predicate on those profiles under the composition rule in [Per-dimension honourability](#per-dimension-honourability-and-how-it-composes-with-feasibility), not a satisfied or an unknown one. Whether a source-level `#pragma METAL fp contract(off)` is a defence on the runtime path is unmeasured; finding 10 records the pragma as an available mechanism the probe deliberately did not adopt.

A cast or quantization boundary is observable even if fusion removes a
physical store/reload that would otherwise have realized it. A backend must
implement the resolved contract natively, emulate it exactly, use an already
permitted relaxation, or reject the plan.

The researched [affine quantization numerical baseline](research/numerics/affine-quantization-semantics.md)
requires positive finite scales, in-range codes and zero points, widened
subtraction, explicit evaluation dtype/order, nearest-even encoding, endpoint
saturation, and distinct logical `Requantize` and integer `Rescale` families.
Strict affine `Quantize` rejects NaN input. Alternative NaN mappings are
separate typed conversion families and never backend-selected behavior.
Strict affine evaluation widens code-minus-zero-point before subtraction, uses
an explicit computation dtype and operation order, preserves subnormals at
named boundaries, clamps before nearest-even integer conversion, saturates
infinities, and makes logical requantization an observable decode followed by
encode.

### Implemented strict-affine proof profile and maturity

**Fact — conversion semantics are exact for the two admitted profiles.** A valid value uses a positive **normal** rank-zero f32 scale, a rank-zero zero point in the complete u4 or u8 code domain, and codes in that same complete domain. The scale domain is a field of the governed static contract (`positive-normal-f32`) rather than a fact about one producing operation, because the decode consumes the type and cannot re-derive where an already-assembled value's scale came from. Quantization evaluates f32 `value / scale`, then f32 addition of the exactly represented zero point, clamps the result inclusively to `[0, 15]` or `[0, 255]`, and applies round-to-nearest, ties-to-even. NaN input is invalid; negative and positive infinity saturate to the lower and upper endpoint. Dequantization widens both code and zero point to i32 before subtraction, converts their difference exactly to f32, and multiplies by scale. Equal code and zero point produce canonical positive zero, and the profile requests no subnormal flushing.

**Fact — association and observable materialization introduce no hidden numerical conversion.** `AssembleStrictAffine` preserves the supplied code, scale, and zero-point tensors exactly as the three components of one logical value. Materializing that encoded value preserves the exact codes and associated parameter payloads; it does not decode, requantize, round, or select a backend-native approximation. The reference comparison for this profile is therefore exact component bytes and exact dequantized f32 result bits, with zero tolerance rather than an epsilon chosen after observing a backend.

**Fact — identity separates static meaning from runtime data.** The namespaced scheme key, complete static contract — including the scale-domain field — ordered component roles, component resolved types, and parameter-map forms participate in the canonical resolved-value-type identity; narrowing the scale domain therefore moved the frozen registry snapshot's digest, and with it the compiler explain request qualifier, without advancing any encoding version; adding the component declarations advanced that encoding to `tiler.resolved-value-type.v3`. Adding operation-owned semantic-precondition declarations and the host-sealed static-evidence authority tag advanced the definition projection to v5, the registry snapshot to v7, and the standard semantic provider to revision 7. The seal is registry provenance rather than predicate meaning, so it does not enter provider-independent definition projection or residual obligation identity. An exact governed constant scale producer participates through the semantic graph and therefore changes program identity when its bits change; no governed constant producer for u4/u8 zero points exists yet. Runtime-bound scale or zero-point payloads intentionally do not become static type or artifact identity; correctness instead requires their bindings to retain the declared logical value, component role, and validation contract. The implemented neutral artifact profile preserves that logical-to-physical association and folds its component roles, storage scalars, access types, and encodings into artifact identity; it remains structural ABI evidence rather than proof that runtime payloads satisfy the scheme.

**Fact — both strict-affine producers declare their exact operation-owned scale preconditions.** `QuantizeStrictAffine` declares `NoNaN` over the whole expressed-value operand; `QuantizeStrictAffine` and `AssembleStrictAffine` each declare `PositiveFiniteScalar` *and* `PositiveNormalScalar` over their whole scale operand. Assembly declares them because a stored quantized weight reaches a decode by being assembled, never by being quantized on device, so an obligation only `Quantize` carried would leave that path unconstrained. `DequantizeStrictAffine` declares none: its sole operand is the assembled compound value, whose scale is no longer a scalar a static proof basis or a residual obligation could name.

These are logical-value predicates: ±infinity in the expressed value remains valid and saturates, and both signed zeros remain valid. `PositiveNormalScalar` is logically strictly stronger than `PositiveFiniteScalar`, and both are declared rather than merged so the two causes report two codes — `…-scale-not-positive-finite` for a zero, negative, ±infinite, qNaN, or sNaN scale, and `…-scale-subnormal` for a positive finite scale below `f32::MIN_POSITIVE`. The two are different callers with different fixes. Static disproof priority is `(invalid-input code, declaration ordinal)`, and the names are chosen so a value failing both — a negative subnormal — reports the general cause. Exact standard constant-f32 bits can prove or disprove them, and every retained proof names the sealed host-owned proof basis rather than trusting provider self-identification. Code and zero-point payload domains are encoded-value conformance rather than duplicate producer predicates, and packed-tail canonicality remains physical representation validity.

**Fact — static assessment and residual identity are implemented, while enforcement is not.** Exact governed f32 constants can prove or disprove all three predicates during transactional graph construction. Unknown values retain ordered proved/residual assessment records, and each residual receives canonical identity only after output compaction from the reached semantic definition, canonical operation and subject coordinates, declaration meaning, full resolved type, and shape. Provider revision, storage, pointer, runtime checker, and transient arena handles are excluded. Two `Assemble` occurrences over one shared scale value therefore keep four distinct obligations, and a `Quantize` occurrence over that same scale adds three more that collide with none of them; a runtime discharge is a fact about one occurrence's operand, never about a predicate or a subject shape. Occurrences whose results reach no output have their whole assessment compacted away with them. No physical plan, artifact, or runtime currently enforces these residuals.

**Measurement — the implemented reference boundary is exercised over both code widths.** The retained tests cover zero, negative, infinite, and NaN scales; subnormal scales at both ends of the subnormal range and the smallest normal scale one bit above it; the complete u8 code × zero-point grid at that smallest admitted scale, showing every decode result is `+0.0` or normal; an out-of-domain u4 code; u8 domain endpoints including 255; component count, role, and shape validation; nearest-even half cases; both input infinities; NaN input rejection; a widened negative difference; positive zero at the zero point; u4 quantize/dequantize composition; and a typed missing-capability refusal for an unknown scheme. These are bounded executable checks of the named profiles, not evidence for an unimplemented scheme or target.

**Measurement — the scale domain is checked class by class, on both producers and through both layers.** One exhaustive table of `f32` classes — both signed zeros, negative finite normals including the most negative one, both signed subnormals with the subnormal range's two endpoints and its interior, both infinities, a quiet and a signalling NaN, and the smallest, an interior, and the largest positive normal — is applied to `Assemble` occurrences over u4 and u8. Each class takes its exact typed outcome: the admitted ones prove both declarations through the standard constant-f32-bits basis with no obligation, and each refused one reports its exact predicate, invalid-input code, and declaration ordinal while committing no canonical work, so the builder that refused it is still usable. The same class table is replayed as runtime payloads through the reference evaluator's `Assemble` route, which reaches its scale validator on its own path rather than through the compound validator, and the three admitted scales assemble and preserve their exact bits. `Assemble`'s ordinals are 0 and 1 where `Quantize`'s are 1 and 2, which is why the two tables are separate rather than shared.

**Measurement — the target-neutral strict-affine u4 path is structural, and Metal now honours its declared contract.** The retained schedule, kernel, program, and artifact fixtures prove role-addressed packed-code access, scalar parameter broadcast, exact-tail storage, widened subtraction, conversion and multiplication order, component-aware bindings, neutral encode/decode, identity sensitivity, and exact evaluation for one five-element dequantization. The carried payload is descriptor-only and requires device translation, so it is not runnable evidence. Mechanical Metal emission reaches the expected packed extraction and conversions, and `require_declared_realization` now succeeds for the decode where it previously refused with `MetalEmitError::UnrealizableNumericalObligation { gap: MetalNumericalGap::SubnormalFlushInArithmetic }`.

**Inference — the refusal was removed by narrowing the value domain, not by weakening the contract.** The decode still declares `Preserve` on both subnormal dimensions, and a pointwise kernel declaring exactly the same thing is still refused on the same target row. What changed is that the declaration became *unobservable* for this program: exhaustively over the finite code domain, the i32 subtraction of two codes is exact and cannot overflow, the conversion of a value of magnitude at most 255 to f32 is exact so the converted operand is `+0.0` or at least `1.0`, and the product with the scale is `+0.0` for equal codes and otherwise at least the scale in magnitude — so no operand and no result is subnormal when the scale is normal, and a flushing and a preserving f32 return identical bits. `tiler-ir` derives this as `SubnormalFreedom` from the *verified* scheduled program and refuses to let a producer declare one; the freedom is keyed by arithmetic type, because the derivation rests on f32's exponent range and on integers up to 255 being exactly representable in f32, and neither premise transfers to a narrower format. Finding 32 of `docs/research/apple-targets/numerical-behaviour.md` measured the chain on the qualified Apple row on 2026-07-31: 1,310,720 normal-scale cells bit-identical to the exact rational reference, `+0.0` in every diagonal cell, and at a subnormal scale a flush acting on the *operand*, where the derivation places it. That measurement's boundary is one GPU family, one toolchain and flag row, u8 codes, and no packed extraction.

**Inference — the reusable result is the separation, not a universal quantization implementation.** Generic ordered component declarations and the typed parameter-map seam can represent a future scheme without adding affine fields to a core enum, but semantic recognition, structural validation, reference evaluation, operations, storage, lowering, dispatchability, runtime validation, and native execution remain separately admitted capabilities. Passing a generic structure test proves only that the mechanism is dtype-neutral; it does not prove numerical or backend support for bool, complex, codebook, MX, NVFP, GGML, sparse, ragged, or another encoded family.

**Proposal — broader parameter maps and executable profiles remain consumer-triggered work.** Per-axis, block, group, hierarchical, codebook, mask, and outlier mappings require a named workload to define their coordinate projection, component shapes, view behavior, validation, identity, and ABI consequences before implementation. The target-neutral u4 dequantization lowering is implemented and Metal's honourability boundary now admits it, but runtime parameter/tail preflight remains unsupported and nothing yet enforces the normal-scale residual against a bound payload, so the decode is not executable. Metal's typed subnormal-flush refusal must not be bypassed by selecting weaker arithmetic, and the discharge that admits the decode is not such a bypass: it narrows the value domain the program is valid over, leaves the declared contract intact, and still refuses every kernel whose domain nothing bounds. Internally produced compound values require producer-derived logical grouping before their components may be scheduled independently; until then they must fail closed rather than be grouped by role, shape, or slot resemblance.

## Backend numerical feasibility

For a resolved operation signature, a backend reports one of these semantic
outcomes rather than silently choosing a nearby instruction:

```text
SupportedExactly
SupportedWithExactEmulation
SupportedOnlyUnderDeclaredRelaxation
Unsupported
```

Target defaults such as TF32 input precision, reduced-precision accumulation,
floating-point contraction, flush-to-zero, or conversion rounding cannot
expand the program's permissions.

**Measurement — a backend default that would expand them, at one pinned compiler.** The [transformer non-linear derivation](research/numerics/transformer-nonlinear-normalization-and-reductions.md) records that at offline compiler `metalfe-32023.883` an unqualified MSL `exp`, `rsqrt`, or `fmax` selects `air.exp.f32`, `air.rsqrt.f32`, and `air.fmax.f32` under the governed `-fmetal-math-mode=safe -fmetal-math-fp32-functions=precise` flag set, and selects `air.fast_exp.f32`, `air.fast_rsqrt.f32`, and `air.fast_fmax.f32` — with LLVM `fast` flags on each call — when the flags are omitted, because the compiler predefines the fast-function selector by default. Intrinsic selection and call-site fast-math licence move independently: `-fmetal-math-mode=relaxed` changes the first without the second. A profile that recorded one "fast math" bit would conflate two freedoms, which the rule above already forbids; the operational consequence is that emitting a transcendental for a precise-family contract requires stating the flag rather than inheriting it. The observation is compile-side and bounded to that compiler; it establishes which intrinsic is selected and nothing about what any of them returns.

### The contract is a required input, stated before planning

The resolved numerical contract is a required, typed input at the compilation
request boundary. It has no default, no ambient fallback, and no implicit
strictest reading; a request that states none does not compile, and the
diagnostic says the contract is unstated rather than naming a dimension the
caller never chose. A strict default is the safe direction for results and the
wrong direction here: on a target whose arithmetic cannot preserve subnormals it
would make every compilation fail with a rejection the caller never asked for,
teaching callers about the contract through refusals.

A caller states one resolved contract, or an explicitly ordered preference list of contracts it declares equally acceptable. Resolution of one semantic candidate follows the caller's stated order and the first honourable entry wins. It is deterministic, it is recorded — the stated list participates in request identity alongside the entry that won, so two requests that resolve alike but declare different fallbacks stay distinct — and it is **never cost-ranked**. A single-entry list and a bare contract behave identically. One contract governs one complete semantic candidate: it does not become a per-region choice, and two regions of one candidate never honour different contracts. A baseline-preserving algebraic portfolio independently readmits each whole-program candidate, so its candidates may resolve differently; the portfolio considers those contract groups only in the same caller-stated order and falls through a group only when it has no feasible complete plan. This is not planner authority to compare contracts on cost, and no contract absent from the caller's list can enter. See [ADR 0076](decisions/0076-declare-target-honourable-numerical-realizations.md).

### Per-dimension honourability, and how it composes with feasibility

A target profile declares, for each **scalar-arithmetic policy subject** and each
dimension of the contract it can be asked about in that subject, which behaviour
it honours and by which of the four means above. The declaration is a stated,
versioned profile fact carrying the same provenance a capability bound does — an
availability phase, a fact authority, a validity scope, and the declaring
profile's identity — so a rejection can name where the claim came from, and it
participates in the profile's canonical descriptor, so two profiles that honour
different behaviours cannot share an identity.

**The key is `(subject, dimension)`, and the subject is load-bearing rather than symmetric.** A subject is an arithmetic type paired with the complete resolved semantic value type it computes in, spelled `tiler_compiler::target::ScalarArithmetic`. A pair reaches that type only through a validated route whose authority is the governed built-in scalar catalog rather than either argument's spelling: the value identity must be one that catalog registers, and the format class and width its registered descriptor states must be the ones stated by the descriptor of the identity the arithmetic type itself names. Each of the four governed floating-point identities — `f16`, `bf16`, `f32`, and `f64` — is therefore constructible over its own value type and over no other, and every remaining pair is refused with `UnvalidatedScalarArithmetic`: an unregistered or foreign identity, a width disagreeing with the arithmetic type's (`f32` arithmetic over `tiler::f16@1`), a class disagreeing with it (`f32` arithmetic over `tiler::u32@1`, which states exactly `f32`'s width), and a logical identity whose descriptor states a value cardinality and no width at all. That the semantic registry recognizes an identity was never evidence that an arithmetic subject had been calibrated for it, and a similar-sounding name was never evidence of either, which is why the route proves the association from the registry instead of accepting the pair a caller asserts. **Constructing a subject is not declaring a fact about it.** A profile that declares no row for a subject leaves every dimension `Unknown` for it, on exactly the terms the next paragraph states. The reason for the key is measured, not structural: on one Apple row — same GPU, same math modes, modules declaring denormals disabled identically — `f32` arithmetic flushes subnormals, `f16` preserves them, and `bf16` flushes. So on that one profile input-subnormal handling is honoured exactly for `f16` and unsupported for `f32`, and a declaration keyed by dimension alone would have to state one of the two wrongly.

**Silence about an arithmetic type is silence, on exactly the terms silence about a dimension is.** The fail-closed clause below applies to all three coordinates of a query — subject, dimension, and required behaviour — and a profile that has spoken about a neighbouring one has said nothing about the one asked for. A profile declaring `f16` preservation says nothing about `f32`; resolution matches the arithmetic type rather than filtering after the fact, and the alternative behaviour a rejection may report is likewise matched on the subject, so a behaviour honoured in a neighbouring width is never offered as an alternative for the width the caller asked about.

**This is the scalar-arithmetic contract and does not generalize.** The subject vocabulary covers floating-point scalar arithmetic. Integer overflow families, boolean semantics, quantized compound schemes, and any future policy family have their own contracts elsewhere in this document and acquire no honourability declaration by standing beside this one.

**Fact — a refusal retains the exact refusing fact rather than a summary of it.** `UnhonouredDimension` in `crates/tiler-compiler/src/target/honourability.rs` holds the checked `NumericalHonourabilityFact` that refused, by shared immutable ownership, beside the behaviour the *caller* required and any behaviour the profile does honour unconditionally. The required value is kept separate from the fact because the two answer different questions: the fact states what the target declares, and the required value states what was asked for. Every rejection that carries the refusal onward — the request boundary's `ContractRejection`, the feasibility authority's `RejectionCause`, the frontier's `FrontierRejection` and `OpaqueCallRejectionCause` — carries the same instance rather than rebuilding one, and every canonical encoder and explain record spells the whole of it, so two profiles refusing the same behaviour on different measured compiler builds share neither a rejection identity nor a rendered explanation. There is no provenance-free way to construct a refusal: a fact exists only once a declaration has been attributed to a declaring profile, and a profile whose declaration source is malformed is refused at construction under the `declaration-source` rule.

Honourability is a **distinct authority** from the quantitative capability axes
of [ADR 0043](decisions/0043-use-typed-phased-target-feasibility.md),
and it composes into that record's outcomes rather than joining its space.
`SupportedWithExactEmulation` has no representation as a bound comparison —
emulation is honoured by *emitting different operations*, so it changes the
program rather than the verdict — and encoding it as a satisfied boolean
predicate would discard the one outcome that carries work. The composition is:

- a dimension honoured exactly or by exact emulation contributes a satisfied hard
  predicate, and the means is retained rather than collapsed into the verdict;
- a dimension honourable only under a relaxation the caller's stated contract
  does not authorize contributes a **disproved** predicate, not a deferred or
  unknown one, because that authorization is known when the contract is resolved
  and cannot arrive at a later phase;
- a dimension the profile declares unhonourable contributes a disproved
  predicate; and
- a dimension the profile does not speak to at all, in the arithmetic type asked
  about, contributes `Unknown` in ADR 0043's exact sense — no admissible proof
  path — so it may appear in search and explain state and never in an executable
  frontier.

That last clause is what makes an unenumerated declaration fail closed instead of
defaulting to honoured, and it applies equally to a profile that enumerates a
dimension but not the behaviour required, and to one that enumerates a dimension
in one arithmetic type but not in the type the caller's contract is stated for:
silence about a behaviour, a dimension, or an arithmetic type is silence, not a
refusal, and nothing may be inferred from the profile having spoken about a
neighbouring one.

**Fact — one bounded F32 projection path now implements this separation.** `tiler-build` owns the adapter because it is the first crate that can depend on both the target-neutral compiler profile vocabulary and Metal's independently defined numerical facts. A caller supplies a `TargetCompileProfileMeasurementSource` separately from the profile under construction. Independent input-subnormal and result-subnormal declaration operations each consume one exact scalar subject, mode, and source and insert either a complete exact three-row declaration or no profile change. The convenience operation `declare_metal_f32_subnormal_behaviour` reads the one stated F32 Metal fact, uses Metal's owner-side total conversion including the zero sign of any flush, stages both dimensions on a clone, and publishes only if both succeed. None of these operations finalize the profile.

**Fact — the checked measured source is admitted across every current profile fact family without becoming an unrestricted source.** Explicit `TargetProfileBuilder::declare_measured_*` operations accept the fixed `CompileProfile`/`MeasuredProfile`/`MeasuredEnvironment` source for all quantitative axes, exact resolved-type dispatchability, and the nine non-subnormal numerical dimensions. The two subnormal operations remain distinct because each atomically publishes a complete exclusive three-row table; there is no row-level measured source path that could publish a partial table. All operations share private insertion sinks, preserve conflict atomicity and canonical identity, and leave omitted facts `Unknown`. The source cannot be converted into `TargetFactSource`, its tuple field remains private, and adding a future fact family does not silently admit measured evidence there.

**Inference — the caller vouches for provenance association, while each owner retains semantic authority.** The compiler owns profile validity, conflict handling, identity, and honourability composition. Metal owns the meaning of the fact it reports and the backend-context recheck. `tiler-build` owns only the dependency-direction-safe projection between those siblings. Accepting a caller-supplied source therefore records a structurally valid assertion that the supplied context and fact are associated; it neither proves that association nor authenticates how the caller obtained them. The retained Metal emission check independently compares each emitted arithmetic obligation against `MetalTargetFacts`; it does not authenticate the caller's provenance claim either.

**Proposal — production use requires an authoritative bound profile.** The follow-up [`construct-and-bind-the-first-authoritative-metal-compile-profile`](../tickets/construct-and-bind-the-first-authoritative-metal-compile-profile.md) must source the quantitative limits, exact F32 dispatchability, and F32 numerical facts for one named and versioned macOS Metal compile profile; bind the compiler, emitter, plan, artifact, and runtime applicability identities; and reject unknown or mismatched contexts. Until that work lands, the bounded projection is not production evidence, does not authorize the serial-sum prototype's unsourced constants, and says nothing about F16, BF16, F64, or iOS devices.

### The honesty rule, in both directions

The rule above states one direction: target defaults cannot expand the program's
permissions. The converse holds too: **no authority may narrow, weaken, or
substitute the caller's stated numerical contract in order to make a target
feasible.** When no contract the caller stated is honourable, compilation rejects
with a typed, explainable error naming the dimension, the required behaviour, the
behaviour the target declares, the means the profile offers if any, the declaring
profile's identity, and the complete provenance of the refusing fact — its
availability phase, authority class, validity scope, versioned authority
identity, and either the governed or external guarantee it cites or the exact
compiler builds and execution environments it was measured on. That last part is
what makes the refusal actionable rather than merely typed: every flushing target
refuses preserved subnormals, and only the measurement boundary tells a caller
whether the refusal was established anywhere near its own deployment. A pre-trace
contract-resolution refusal exposes it through the same typed session facade a
traced one does; a traced refusal additionally retains it in explain identity and
rendering. It never emits a program under a different
contract, never falls back to a target default, and never reports the difference
as a cost. A rejection may report which behaviour the target *would* honour, so a
caller can see what contract this target accepts; only the caller may act on it.

**The numerical contract is therefore not a search dimension.** Cost-based
selection ranks implementations of one contract and may never rank contracts
against each other, because doing so prices meaning. The neighbouring temptation
this forbids by name is treating a flush-tolerant plan as a cheaper alternative
to a preserving one.

**Fact — semantic-alternative readmission preserves this rule.** Every baseline or algebraic candidate re-resolves from the caller's same ordered preference list rather than inheriting another candidate's answer. Candidates are partitioned by the contract they resolve, groups are considered in caller order, and cost comparison is confined to one group. The first group with a feasible complete plan is selected; later groups are recorded as preference-pruned without planning. This is defensive handling of independently verified candidates, not permission for the optimizer to synthesize, weaken, or cost-rank a contract.

## Conformance levels

“Exact” is not synonymous with portable bitwise equality. A kernel declares a
conformance level such as:

- **portable bitwise:** same bits across conforming targets;
- **plan deterministic:** same bits for identical inputs and bindings under
  the same artifact, selected variant, and declared target environment;
- **toolchain bitwise:** same bits for a pinned target/toolchain contract;
- **backend elementary:** operation graph is preserved but elementary function
  results follow the backend contract;
- **bounded error:** result satisfies operation- and dtype-specific bounds;
- **permitted result set:** nondeterministic reductions may return any result
  satisfying a documented model.

The exact set of public levels remains open, but every test oracle chooses one
explicitly. Relaxed and nondeterministic reductions may require repeated runs,
an interval/result-set model, and invariants rather than comparison to one
reference number.

## Testing authority

Normative operation specifications are authoritative. The reference evaluator
implements those specifications and is tested with hand-authored conformance
vectors and independent higher-precision checks where appropriate. A consumer
runtime is a compatibility oracle only when its documented behavior matches the
selected contract. The proposed first integration checks Candle and generated
Metal against the declared conformance level without making either the semantic
authority.

**Fact — strict affine uses an exact oracle rather than a tolerance fit.** The evaluator retains code and parameter payloads as exact bytes and emits dequantized f32 values by their exact result bits after validating the complete role/type/shape contract. The retained fixtures assert exact u4 code bytes and dequantized values plus exact u8 boundary results; invalid scale, code, parameter, component, and unsupported-scheme cases must reject by their typed boundary rather than produce a nearby value.

Tests include NaN, infinities, subnormals, signed zero, extreme integers, empty
domains, and schedule changes.

The selected numerical contract, implementation realization, evidence
provenance, and backend compiler flags appear in `EXPLAIN`, cache keys, and
artifact manifests.

# Numerical semantics

**Status:** proposed framework; concrete operation policies remain open

Tiler optimizes floating-point and integer programs whose algebraic identities
do not automatically imply machine-level equivalence. Numerical policy is part
of semantic meaning, legality, plan identity, artifact identity, and testing.

## Three parts of the contract

Numerical meaning is divided into three machine-checkable parts.

### Operation semantics

Each scalar or reduction operation defines its dtype signature, casts,
identity and empty-domain behavior, division/modulo behavior, and min/max NaN
and signed-zero contract. These are properties of the operation, not global
optimizer switches.

## Resolved numerical typing

Every compilable semantic tensor value has a resolved value dtype. Every
operation has a resolved numerical signature sufficient to define its
observable computation. Tiler does not apply an ambient global promotion table
after semantic admission.

Ordinary elementwise operations are homogeneous by default. Frontends may
offer PyTorch-like, JAX-like, strict, or custom promotion, weak-scalar, and
autocast policies, but they lower the result to explicitly typed constants,
conversions, operands, and results before optimization.

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

The resulting canonical contract is granular, for example:

```rust
struct NumericPolicy {
    reassociation: Reassociation,
    contraction: Contraction,
    approximate_intrinsics: ApproximateIntrinsics,
    reciprocal_math: ReciprocalMath,
    nan_assumptions: NaNAssumptions,
    infinity_assumptions: InfinityAssumptions,
    signed_zero: SignedZeroPolicy,
    subnormals: SubnormalContract,
}
```

The example is descriptive rather than a committed API. A user-facing named
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

## Value assumptions and validation

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

## Transcendental accuracy

Every transcendental operation carries a resolved, operation-specific accuracy
contract. `Exp`, `Log`, `Sin`, and similar operations do not inherit an
accuracy choice from ambient compiler flags or a backend's default math
library.

Illustrative contract forms include correctly rounded results, a bounded ULP
or absolute/relative error model over a stated input domain, and a versioned
backend-elementary contract. The actual metric vocabulary remains to be
decided; an opaque `approximate` boolean is insufficient.

A frontend may expose named accuracy presets, but it resolves them before
canonical semantic admission. A rewrite, fusion choice, or backend intrinsic
is legal only when it proves that its implementation satisfies the resolved
operation contract or consumes a separately authorized relaxation. Backend
feasibility may report exact native support, exact emulation, relaxed-only
support, or rejection.

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

### Empty domains and initial values

Each reduction operation declares whether it has an intrinsic identity for its
resolved accumulator dtype and numerical contract. An empty reduction returns
that identity when one exists. Representative contracts include additive zero,
multiplicative one, `true` for `all`, and `false` for `any`; the exact typed
identity is operation semantics, not a backend default.

An optional explicit `initial` is a true reduction seed, not an empty-only
fallback. It is converted according to the resolved reduction signature and is
one logical contributor for every output reduction domain, including non-empty
domains. Thus `minimum([20], initial=10)` produces `10`, and a sum with
`initial=10` adds ten exactly once.

This distinction constrains physical scheduling. A non-identity seed cannot be
copied into every SIMD lane, threadgroup, or partial reduction. A true intrinsic
identity may be replicated where the operation contract proves it neutral; an
arbitrary initial value remains exactly one logical contributor even when the
permitted topology reassociates work.

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

Canonicalization applies according to each operation's semantic family; it is
not a blanket rewrite of stored tensor bits. Operations defined to preserve or
select existing bits, including views and bit-preserving copies, preserve an
input NaN payload. Numeric conversions use their resolved conversion contract.
Constants retain their declared bit pattern until an operation's semantics
produce a new value.

Other conformance modes may explicitly request operand-payload propagation or
permit any quiet NaN. Those choices are typed operation contracts and affect
plan feasibility, reference evaluation, determinism, and artifact identity.
No mode inherits NaN payload behavior from a backend default.

## Subnormal inputs and results

Subnormal handling has two independent dimensions:

```text
SubnormalContract {
  inputs:  Preserve | FlushToZero,
  results: Preserve | FlushToZero,
}
```

Input flushing treats an existing subnormal operand as zero before arithmetic.
Result flushing replaces a newly produced subnormal result with zero. The zero
sign follows the resolved signed-zero and subnormal contract rather than an
ambient target mode.

Portable-bitwise execution preserves both input and result subnormals. Other
contracts may permit either or both forms of flushing. Some targets expose only
a coupled mode or cannot realize every combination; that is reported as native,
emulated, relaxed-only, or unsupported backend feasibility rather than
collapsing the semantic dimensions.

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
Index expressions must not wrap into a valid-looking address. Shape products,
stride products, offset additions, and narrowing conversions are checked
statically or protected by runtime guards.

Division and modulo define signedness, rounding direction, and zero-divisor
behavior. Simplification passes preserve these semantics.

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

A cast or quantization boundary is observable even if fusion removes a
physical store/reload that would otherwise have realized it. A backend must
implement the resolved contract natively, emulate it exactly, use an already
permitted relaxation, or reject the plan.

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

Tests include NaN, infinities, subnormals, signed zero, extreme integers, empty
domains, and schedule changes.

The selected numerical contract and backend compiler flags appear in `EXPLAIN`,
cache keys, and artifact manifests.

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
    preserve_nan: bool,
    preserve_signed_zero: bool,
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

## Min and max

Backends differ in their treatment of NaN and signed zero. Tiler must define
whether min/max are propagating, number-preferring, or follow another named
contract, and then emit or synthesize matching behavior. A backend intrinsic is
not selected until its semantics are known to agree.

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

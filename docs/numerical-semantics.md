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

### Optimization permissions

The program carries granular permissions such as:

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
mode may expand into a complete permission set, but an overlapping `fast_math`
boolean is avoided. Backend flags are derived from the permissions and must not
silently enable additional transformations.

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

Changing from a serial reduction to a SIMD or threadgroup tree is a physical
alternative only when the numerical policy permits its evaluation order. F16
or BF16 inputs do not imply low-precision accumulation; promotion is explicit.

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

Casts state source and destination dtype and define behavior for:

- out-of-range float-to-integer conversion;
- NaN to integer;
- narrowing integer conversion;
- floating-point rounding;
- backend feature-dependent formats.

## Conformance levels

“Exact” is not synonymous with portable bitwise equality. A kernel declares a
conformance level such as:

- **portable bitwise:** same bits across conforming targets;
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

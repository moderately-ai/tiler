# 0023: Separate propagating and number-preferring extrema

**Status:** accepted

## Context

Floating-point minimum and maximum do not have one portable meaning. IEEE
754-2019 distinguishes NaN-propagating `minimum`/`maximum` from
number-preferring `minimumNumber`/`maximumNumber`. LLVM exposes corresponding
distinct intrinsic families. Both strict IEEE families order `-0.0 < +0.0`.

GPU source languages do not necessarily provide exact native equivalents.
Metal `fmin`/`fmax` prefer numbers over NaNs and can select opposite-signed zero
according to operand order. Treating backend spelling as semantic authority
would make operand swapping, fusion, and target selection observable.

## Decision

Tiler exposes distinct semantic operation families:

- `Minimum` and `Maximum` propagate NaN;
- `MinimumNumber` and `MaximumNumber` return the numeric operand when exactly
  one operand is NaN and return NaN when both are NaN.

Both families deterministically order `-0.0 < +0.0`. Portable-bitwise NaN
results use the existing canonical arithmetic-NaN contract.

The families are different operations, not backend modes or optimizer flags.
Any relaxation of NaN assumptions or signed-zero observability remains a
separate resolved numerical permission. Elementwise and reduction operations
name their scalar family explicitly, and reduction identity/seed/order behavior
remains separately resolved.

A backend lowers an operation natively only when the complete NaN and zero-tie
behavior matches. Otherwise it emits an exact fixup, uses an already authorized
relaxation, or rejects the physical alternative.

## Consequences

- Frontends can lower their own min/max conventions without ambiguity.
- Rewrite rules and costed plans name the exact extrema family they require.
- Strict ReLU, clamp, operand commutation, and reductions cannot silently adopt
  Metal's native `fmin`/`fmax` behavior.
- Exact Metal lowering may require additional comparisons or bit-level zero
  handling; that cost belongs in physical planning.
- Conformance tests cover both operand orders of qNaN, sNaN, opposite-signed
  zeros, infinities, and finite values, plus reduction tree permutations.

## Alternatives considered

One generic operation with a NaN attribute is mechanically possible but weakens
operation identity and repeats a distinction already standardized as separate
operations. Choosing one global behavior prevents faithful lowering of
frontends with the other behavior. Inheriting target intrinsics makes semantics
backend-dependent.

## Primary precedents

- [StableHLO `minimum`](https://openxla.org/stablehlo/spec#minimum) and
  [`maximum`](https://openxla.org/stablehlo/spec#maximum)
- [LLVM floating-point min/max comparison](https://llvm.org/docs/LangRef.html#floating-point-min-max-intrinsics-comparison)
- [Metal Shading Language specification](https://developer.apple.com/metal/Metal-Shading-Language-Specification.pdf)

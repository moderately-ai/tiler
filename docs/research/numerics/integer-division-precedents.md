---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.integer-division-precedents"
kind: "research"
title: "Integer division and remainder precedents"
topics: ["numerics","integers","division"]
research_status: "complete"
disposition: "adopted"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.numerical-semantics"]
adopted_by: ["ADR-0040"]
reproduced_by: []
ticket: "numerical-policy-contract"
---

# Integer division and remainder precedents

**Status:** adopted decision research supporting ADR 0040

## Traceability

- **Current disposition:** adopted; historical status text below records the report's state when written.
- **Normative destination:** [Numerical semantics](../../numerical-semantics.md).
- **Adoption:** [ADR 0040](../../decisions/0040-specialize-integer-division-families.md).
- **Work record:** [numerical-policy-contract](../../../tickets/numerical-policy-contract.md).


## Distinct quotient families

Negative operands make quotient rounding observable. For `a = -7, b = 4`:

| Family | Quotient | Remainder rule | Remainder |
|---|---:|---|---:|
| truncating | toward zero: `-1` | zero or dividend sign | `-3` |
| floor | toward negative infinity: `-2` | zero or divisor sign | `1` |
| Euclidean | chosen so remainder is nonnegative | `0 <= r < abs(b)` | `1` |

Floor and Euclidean division differ when the divisor is negative. Ceiling
division is another useful quotient family, especially for shape and tiling
calculations. Exact division is not a rounding family: it adds a divisibility
precondition under which all quotient roundings coincide.

Rust exposes truncating `/` and `%` plus Euclidean methods. Python uses floor
division and divisor-sign remainder. NumPy and PyTorch expose both floor/modulo
and truncating/fmod families. MLIR explicitly includes truncating, floor, and
ceiling operations. This is strong precedent for preserving families rather
than assigning one universal `%` meaning.

Primary sources:

- [Rust arithmetic operators](https://doc.rust-lang.org/reference/expressions/operator-expr.html#arithmetic-and-logical-binary-operators)
- [Rust Euclidean division](https://doc.rust-lang.org/std/primitive.i32.html#method.div_euclid)
- [Python binary arithmetic](https://docs.python.org/3/reference/expressions.html#binary-arithmetic-operations)
- [NumPy floor division](https://numpy.org/doc/stable/reference/generated/numpy.floor_divide.html)
- [PyTorch division](https://docs.pytorch.org/docs/stable/generated/torch.div.html)
- [MLIR arithmetic operations](https://mlir.llvm.org/docs/Dialects/ArithOps/)

## Exceptional inputs

LLVM defines signed and unsigned truncating division/remainder, but division by
zero is undefined behavior. Signed `MIN / -1` is also undefined because the
quotient is unrepresentable. LLVM additionally makes `MIN % -1` undefined even
though its mathematical remainder is zero, allowing targets to use combined
division/remainder instructions. An `exact` division whose remainder is not
zero produces poison.

StableHLO leaves integer runtime errors implementation-defined. PyTorch
documents that integer division by zero raises on CPU but may return any value
on GPU. These are precisely the target divergences a portable Tiler graph must
exclude.

Primary sources:

- [LLVM signed division](https://llvm.org/docs/LangRef.html#sdiv-instruction)
- [LLVM signed remainder](https://llvm.org/docs/LangRef.html#srem-instruction)
- [StableHLO execution errors](https://openxla.org/stablehlo/spec#errors)
- [JAX integer division](https://docs.jax.dev/en/latest/_autosummary/jax.lax.div.html)

## Rewrite consequences

- A quotient and remainder identity `a = q*b + r` only connects matching
  families.
- Signed truncating division by a positive power of two is not generally an
  arithmetic shift for negative nonmultiples; floor division is.
- `a / -1 -> -a` requires excluding signed `MIN` or preserving the failure.
- Euclidean remainder provides nonnegative range facts useful for indexing;
  truncating remainder does not.
- Fused `DivRem` and target strength reductions must preserve quotient
  rounding and every exceptional-input contract.

## Tiler implication

Canonical operations name signedness, quotient rounding, remainder convention,
and exceptional-input policy. Zero divisors, unrepresentable quotients, and
exact-divisibility requirements are statically proven or runtime validated.
They never become silent UB or poison. Standalone remainder semantics should
follow mathematics rather than inherit an implementation restriction of a
combined target instruction.

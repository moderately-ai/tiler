---
schema: "tiler-doc/v1"
id: "ADR-0040"
kind: "decision"
title: "Specialize integer division and remainder families"
topics: ["numerics","integers","division"]
decision_status: "accepted"
implementation_status: "not-started"
applies_to: ["tiler.contract.numerical-semantics"]
evidence: ["tiler.research.numerics.integer-division-precedents"]
ticket: "numerical-policy-contract"
---

# 0040: Specialize integer division and remainder families

**Status:** accepted

## Traceability

- **Normative owner:** [Numerical semantics](../numerical-semantics.md).
- **Evidence:** [integer division precedents](../research/numerics/integer-division-precedents.md).
- **Work record:** [numerical-policy-contract](../../tickets/numerical-policy-contract.md).


## Context

Signed integer quotient and remainder differ under truncating, floor, and
Euclidean conventions. Ceiling division is independently useful for shape and
tiling calculations. Source ecosystems expose several of these meanings, while
some tensor IRs leave exceptional inputs implementation-defined.

Division by zero, signed `MIN / -1`, and non-divisible exact division cannot be
allowed to inherit target undefined behavior. They affect correctness,
rewrites, runtime validation, and whether a GPU plan can fail without partial
publication.

## Decision

Canonical integer division uses specialized, versioned semantic operations.
The recognized families include:

- signed truncating division and dividend-sign remainder;
- signed floor division and divisor-sign remainder;
- signed Euclidean division and nonnegative Euclidean remainder;
- signed ceiling division;
- unsigned division/remainder, whose truncating, floor, and Euclidean meanings
  coincide for a nonzero divisor; and
- unsigned ceiling division.

Division descriptors state quotient rounding and signedness. Remainder and
paired `DivRem` descriptors additionally state the matched identity
`a = q*b + r` and remainder sign/range. Frontend spellings such as
`/`, `%`, `//`, `fmod`, `remainder`, and `div_euclid` are resolved by their
source-language contract and never determine semantics by spelling alone.

Strict division requires a nonzero divisor. Signed division additionally
requires that the quotient be representable, excluding `MIN / -1`. These are
semantic preconditions discharged statically or through runtime validation
under ADRs 0021 and 0033. Invalid inputs produce the explicit semantic failure;
they never yield poison, undefined behavior, or a target-dependent value.

Standalone remainder requires only a nonzero divisor. In particular, signed
`MIN rem -1` is mathematically zero and remains valid. A lowering through a
combined target divide/remainder instruction must fix up or reject that
physical plan rather than weakening logical semantics. A paired `DivRem`
operation inherits the quotient's representability precondition.

Exact division is a contract-bearing specialization or refinement with the
additional precondition `remainder == 0`; it is not LLVM-style poison. Future
total families may return validity masks, fill values, wrapping results, or
other explicitly versioned outcomes without changing existing meanings.

True division that converts integers to floating point is a separate typed
conversion-and-arithmetic contract. It is not an integer division mode.

## Consequences

- Python-, Rust-, NumPy-, and framework-originated operations can be imported
  without flattening distinct negative-operand behavior.
- Rewrites and range inference key on the exact family.
- Unsigned equivalent forms canonicalize to one division/remainder identity
  rather than retaining redundant aliases.
- Runtime validation may make a plan more expensive or infeasible, but cannot
  change the semantic result.
- Physical fused division/remainder and strength-reduction choices must
  preserve exceptional behavior as well as ordinary values.

## Alternatives considered

One integer `Div`/`Rem` pair would either be ambiguous or force every frontend
into one language's convention. Copying LLVM UB/poison makes invalid inputs an
optimizer promise rather than a diagnosable program contract. Treating
`MIN rem -1` as invalid merely because some hardware computes quotient and
remainder together leaks a physical limitation into the logical IR. Returning
a validity mask for every division makes the operation total but imposes an
additional output and handling burden even when validity is already provable;
it remains an extensible future family.

---
schema: "tiler-doc/v1"
id: "tiler.research.numerics.integer-overflow-precedents"
kind: "research"
title: "Integer arithmetic overflow precedents"
topics: ["numerics","integers","overflow"]
catalog_group: "numerical-operations"
research_status: "complete"
disposition: "adopted"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.numerical-semantics"]
adopted_by: ["ADR-0039"]
ticket: "numerical-policy-contract"
---

# Integer arithmetic overflow precedents

**Status:** adopted decision research supporting ADR 0039

## Traceability

- **Current disposition:** adopted; historical status text below records the report's state when written.
- **Normative destination:** [Numerical semantics](../../numerical-semantics.md).
- **Adoption:** [ADR 0039](../../decisions/0039-explicit-integer-overflow-operations.md).
- **Work record:** [numerical-policy-contract](../../../tickets/numerical-policy-contract.md).


## Finding

Integer overflow is operation semantics, not a property determined by the
integer dtype. Existing systems expose several incompatible contracts, and
some widely used tensor IRs leave generic overflow underspecified.

## Compiler IRs

MLIR `arith` and LLVM IR define plain fixed-width addition, subtraction, and
multiplication modulo `2^N`. Their `nuw` and `nsw` annotations instead promise
that unsigned or signed overflow does not occur; a violated promise produces
poison. LLVM represents observable overflow with separate `with.overflow`
intrinsics returning the modular result and an overflow bit, and represents
saturation with separate intrinsics. MLIR similarly provides extended-result
operations.

StableHLO specifies generic integer add, subtract, and multiply but its global
error contract currently leaves integer overflow implementation-defined. ONNX
generic Add, Sub, and Mul likewise do not provide a portable overflow contract.
These are negative precedents for a toolkit whose logical graph must mean the
same thing across targets.

Primary sources:

- [MLIR arithmetic operations](https://mlir.llvm.org/docs/Dialects/ArithOps/)
- [LLVM integer addition](https://llvm.org/docs/LangRef.html#add-instruction)
- [LLVM arithmetic-with-overflow intrinsics](https://llvm.org/docs/LangRef.html#arithmetic-with-overflow-intrinsics)
- [LLVM saturation intrinsics](https://llvm.org/docs/LangRef.html#saturation-arithmetic-intrinsics)
- [StableHLO execution errors](https://openxla.org/stablehlo/spec#errors)
- [ONNX Add](https://onnx.ai/onnx/operators/onnx__Add.html)

## User and target systems

Rust ordinary integer operators can check and panic or wrap depending on
compilation configuration, while its `checked_*`, `wrapping_*`, `saturating_*`,
`overflowing_*`, and `strict_*` families expose explicit behavior. Tiler cannot
inherit a consumer's Rust overflow-check setting because artifact semantics
must not change between debug and release builds.

PTX exposes separate physical forms: ordinary fixed-width arithmetic, selected
saturating operations, low/high/wide multiplication, and carry/borrow variants.
This demonstrates target feasibility for several contracts but does not choose
the logical default.

Primary sources:

- [Rust overflow expressions](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow)
- [Rust integer overflow behavior](https://doc.rust-lang.org/reference/behavior-not-considered-unsafe.html#integer-overflow)
- [PTX integer arithmetic instructions](https://docs.nvidia.com/cuda/parallel-thread-execution/#integer-arithmetic-instructions)

## Rewrite consequences

- Wrapping add and multiply form arithmetic modulo `2^N`; associativity,
  commutativity, and distributivity hold, but ordinary signed/unsigned
  monotonicity and range reasoning do not survive wrap.
- Saturating arithmetic is generally not associative, distributive, or
  cancellative.
- Checked or required-no-overflow arithmetic makes intermediate overflow
  observable, so reassociation requires range proofs and must preserve failure
  behavior.
- Widening arithmetic changes the result signature rather than merely changing
  an overflow flag.

For signed i8, `(100 sat+ 100) sat+ -100` is `27`, while
`100 sat+ (100 sat+ -100)` is `100`. The same regrouping changes whether an
intermediate checked addition overflows.

## Tiler implication

Canonical integer arithmetic must name its overflow family. Backend poison,
undefined behavior, host compilation flags, or framework implementation habits
cannot supply missing semantics. An overflow-free fact is a proof obligation
or runtime-validated precondition, not silent poison. The operation mechanism
must remain extensible beyond the initially recognized wrapping, saturating,
checked, and widening families.

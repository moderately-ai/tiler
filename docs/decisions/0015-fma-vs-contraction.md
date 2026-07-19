# 0015: Distinguish required FMA from optional contraction

**Status:** accepted

## Context

Fused multiply-add computes a multiply and add with one final rounding. A
separate multiply followed by add normally has a rounding boundary between the
operations, so the results can differ. This is observable semantics, not only
an instruction-selection preference.

There are genuine public and compiler precedents for both meanings. Rust
guarantees the rounded infinite-precision result for `f32::mul_add`. LLVM uses
`llvm.fma` when fusion is required and separately exposes contraction
permission for eligible multiply/add instructions.

## Decision

Tiler supports a dedicated semantic `Fma` operation for required
single-rounding multiply-add behavior. A backend must implement it natively,
emulate it exactly, use an already permitted relaxation, or reject the plan.

Separate semantic `Mul` and `Add` operations retain their separate rounding
boundaries. Their resolved contraction permission may authorize replacing the
existing pattern with an FMA implementation. Contraction permission is
independent of reassociation and does not authorize regrouping an expression to
create additional contraction opportunities.

## Consequences

- Frontends can preserve APIs and algorithms that explicitly require FMA.
- Exact evaluation can distinguish one-rounding and two-rounding expressions.
- Optimizers can still exploit hardware FMA for ordinary multiply/add graphs
  when the user permits contraction.
- A backend without suitable native FMA support may need exact emulation or may
  reject a required-FMA plan.
- Rewrite explanations distinguish required FMA lowering from optional
  contraction.

## Alternatives considered

Representing every multiply-add as separate operations cannot express required
single-rounding semantics. Representing every eligible pair as `Fma` erases
observable rounding unless contraction was explicitly permitted. Treating FMA
as an affine/index operation conflates numeric value computation with affine
access maps; `x * y + z` is also not affine jointly in varying `x` and `y`.

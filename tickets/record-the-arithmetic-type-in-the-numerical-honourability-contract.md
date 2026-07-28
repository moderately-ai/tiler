---
id: record-the-arithmetic-type-in-the-numerical-honourability-contract
title: Record the arithmetic type in the numerical honourability contract
status: todo
priority: p2
dependencies: []
related: [implement-first-profile-numerical-policies, carry-the-dtype-on-the-metal-subnormal-flush-fact]
scopes: [contracts/numerics]
shared_scopes: []
paths: []
tags: [documentation, numerics]
---
`docs/numerical-semantics.md`'s "Per-dimension honourability, and how it composes with feasibility" section says a target profile declares, "for each dimension of the contract it can be asked about, which behaviour it honours and by which of the four means above". It names no arithmetic type, and the implementation now keys every declaration by one.

**Fact — the measurement.** ADR 0076 boundary item 3's inherited note of 2026-07-25 records that one Apple profile flushes subnormals in `f32` and preserves them in `f16`, so `InputSubnormals` is `SupportedExactly` for one arithmetic type and `Unsupported` for the other on the same profile.

**Fact — the implementation.** `implement-first-profile-numerical-policies` keyed `DeclaredBehaviour`, `NumericalRequirement`, `RelaxationRequirement`, `HonouredDimension`, `UnhonouredDimension`, `UndeclaredDimension`, and `DeferredDimension` by `(dimension, tiler_ir::schedule::ArithmeticType)`. `CheckedTargetProfile::resolve_dimension` matches the arithmetic type rather than filtering after the fact, and `honoured_alternative` matches it too, so a behaviour honoured in a neighbouring dtype is never reported as an alternative for the type the caller asked about.

**Inference — the contract sentence is now the weaker claim.** A reader following that document would write a per-dimension declaration and find it rejected as incomplete, or worse, read the current sentence as licence to declare one behaviour per dimension for a profile whose measured behaviour differs by width. The section's own fail-closed clause — "a dimension the profile does not speak to at all contributes `Unknown`" — needs the same widening: silence about an *arithmetic type* is silence, and nothing may be inferred from the profile having spoken about a neighbouring width.

**Also owed by the same change.** The document's canonical policy sketch lists eight fields; the implemented dimension vocabulary is eleven, adding operand permutation, signed zero, and the two exceptional-value assumptions as first-class dimensions of the resolved contract rather than only as prose in the Reductions and optimization-permissions sections. And the approximate-intrinsic dimension resolves to a governed *named envelope* vocabulary rather than a free-form key, which the "Transcendental accuracy" section should say if it is to remain the authority over that spelling.

## Closes when

The honourability section states that a declaration is keyed by dimension *and* arithmetic type, states that an unenumerated arithmetic type fails closed as `Unknown` on the same terms as an unenumerated dimension and an unenumerated behaviour, and the canonical policy sketch agrees with the implemented dimension vocabulary. `make full` passes. (Citation corrected at landing: both Python tools this ticket originally named were retired by `e197176`; nothing renders or validates the docs corpus now, so the docs half of this ticket is checked only by reading — write accordingly.)

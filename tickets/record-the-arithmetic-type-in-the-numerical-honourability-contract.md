---
id: record-the-arithmetic-type-in-the-numerical-honourability-contract
title: Record the arithmetic type in the numerical honourability contract
status: closed
priority: p2
dependencies: []
related: [implement-first-profile-numerical-policies, carry-the-dtype-on-the-metal-subnormal-flush-fact]
scopes: [contracts/numerics]
shared_scopes: []
paths: []
tags: [documentation, numerics]
closed_reason: superseded
closed_note: Requirements merged into the broader stale public compiler and ADR 0076 authority correction.
---
`docs/numerical-semantics.md`'s "Per-dimension honourability, and how it composes with feasibility" section says a target profile declares, "for each dimension of the contract it can be asked about, which behaviour it honours and by which of the four means above". It names no arithmetic type, and the implementation now keys every declaration by one.

**Fact — the measurement.** ADR 0076 boundary item 3's inherited note of 2026-07-25 records that one Apple profile flushes subnormals in `f32` and preserves them in `f16`, so `InputSubnormals` is `SupportedExactly` for one arithmetic type and `Unsupported` for the other on the same profile.

**Fact — the implementation.** `implement-first-profile-numerical-policies` keyed `DeclaredBehaviour`, `NumericalRequirement`, `RelaxationRequirement`, `HonouredDimension`, `UnhonouredDimension`, `UndeclaredDimension`, and `DeferredDimension` by `(dimension, tiler_ir::schedule::ArithmeticType)`. `CheckedTargetProfile::resolve_dimension` matches the arithmetic type rather than filtering after the fact, and `honoured_alternative` matches it too, so a behaviour honoured in a neighbouring dtype is never reported as an alternative for the type the caller asked about.

**Inference — the contract sentence is now the weaker claim.** A reader following that document would write a per-dimension declaration and find it rejected as incomplete, or worse, read the current sentence as licence to declare one behaviour per dimension for a profile whose measured behaviour differs by width. The section's own fail-closed clause — "a dimension the profile does not speak to at all contributes `Unknown`" — needs the same widening: silence about an *arithmetic type* is silence, and nothing may be inferred from the profile having spoken about a neighbouring width.

**Also owed by the same change.** The document's canonical policy sketch lists eight fields; the implemented dimension vocabulary is eleven, adding operand permutation, signed zero, and the two exceptional-value assumptions as first-class dimensions of the resolved contract rather than only as prose in the Reductions and optimization-permissions sections. And the approximate-intrinsic dimension resolves to a governed *named envelope* vocabulary rather than a free-form key, which the "Transcendental accuracy" section should say if it is to remain the authority over that spelling.

## Closes when

The honourability section states that a declaration is keyed by dimension *and* arithmetic type, states that an unenumerated arithmetic type fails closed as `Unknown` on the same terms as an unenumerated dimension and an unenumerated behaviour, and the canonical policy sketch agrees with the implemented dimension vocabulary. `make full` passes. (Citation corrected at landing: both Python tools this ticket originally named were retired by `e197176`; nothing renders or validates the docs corpus now, so the docs half of this ticket is checked only by reading — write accordingly.)

## Superseded, and where each requirement landed (2026-07-31)

[`correct-stale-public-compiler-boundary-authorities`](correct-stale-public-compiler-boundary-authorities.md) carried every requirement above rather than absorbing the ticket into a general sweep. Recorded per requirement so the supersession is checkable rather than asserted:

- **The declaration is keyed by a scalar-arithmetic policy subject as well as by dimension.** `docs/numerical-semantics.md`, "Per-dimension honourability, and how it composes with feasibility" — the section's opening sentence now names the subject, and a following paragraph states that the subject is an arithmetic type paired with the complete resolved semantic value type, spelled `tiler_compiler::target::ScalarArithmetic`, with the measured `f32`/`f16`/`bf16` divergence as the reason the key exists.
- **Silence about an arithmetic type is `Unknown`.** The same section's fail-closed clause now applies to all three coordinates of a query — subject, dimension, and required behaviour — and states that a declaration about a neighbouring width is not evidence about the width asked for, matching `resolve_dimension` and `honoured_alternative` both matching on the subject.
- **The canonical policy sketch agrees with the implemented vocabulary.** `docs/numerical-semantics.md`, "Optimization permissions" — the sketch lists all eleven governed dimensions in `CANONICAL_DIMENSIONS` order, adds operand permutation and materialization rounding, splits the two subnormal dimensions, and states that the contract key and canonical NaN bits are properties rather than honourability dimensions. It also states explicitly that the dense scalar contract is `f32` and generalizes to no integer, boolean, or quantized-compound policy family.
- **The approximate-intrinsic envelope vocabulary is named.** `docs/numerical-semantics.md`, "Transcendental accuracy" — `ApproximationEnvelope`'s two governed resolutions and their keys, why `Forbidden` is not an empty envelope, and why closedness is what makes the dimension identity-safe.
- **ADR 0076 records the same correction against its own text.** A dated correction beside item 2's "As landed" paragraph names the module move, the widening from four dimensions to eleven, and the subject key, without editing the accepted decision.

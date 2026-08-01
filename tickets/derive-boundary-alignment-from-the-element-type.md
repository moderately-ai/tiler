---
id: derive-boundary-alignment-from-the-element-type
title: Derive boundary alignment from the element type rather than the profile
status: todo
priority: p1
dependencies: []
related: [spike-bf16-through-the-second-dtype-seams, admit-bf16-into-the-schedule-and-kernel-vocabulary]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, abi, boundary]
---
## User-visible outcome

A boundary value's alignment requirement comes from its own element type, so a two-byte dtype states two-byte alignment instead of inheriting `f32`'s four. Today every boundary in the compiler is aligned as though it were `f32`, and for BF16 that is silently wrong in the permissive direction.

## Why this is a prerequisite rather than a cleanup

**Fact, at `ef3c051`.** `ByteAlignment::F32_NATURAL` is a constant `4` in `crates/tiler-compiler/src/boundary.rs`, and its own doc comment names the gap:

> The bounded profile's boundary values are strict `f32` throughout under `StrictF32NumericalContract`, and `ScheduledRegion` carries no resolved element type of its own. A widened dtype vocabulary must derive this from the boundary value's element type rather than from the profile, and that derivation needs a field the scheduled-region IR does not have today.

Reproduce in one line:

```sh
rg -n -B6 'F32_NATURAL: Self' crates/tiler-compiler/src/boundary.rs
```

**Fact.** It is consumed at roughly twenty sites across `call_registry.rs`, `call_abi.rs`, `call_declaration.rs`, `selection.rs`, `frontier.rs`, and `boundary.rs` itself.

**Inference.** [The BF16 spike](../spikes/numerics/bf16-second-dtype/README.md) classified this as a *missing typed extension point* rather than an F32-specific fact: alignment is a property of the element type in every ABI, and the constant is standing in for a derivation the IR cannot yet express. A four-byte alignment applied to a two-byte element over-aligns, which is not a wrong answer today only because there is one dtype — the moment there are two, one of them is being told a requirement that is not its own, and an ABI check that passes for the wrong reason is the kind of thing that stops passing when a real allocator gets involved.

## Implementation keys

- The scheduled-region IR needs the resolved element type at the boundary. Decide where it lives and state the elimination: a field on the boundary value, a derivation from the region's scalar program, or a lookup through the semantic value — these differ in whether a region with no scalar program can answer, and in whether the answer is part of canonical identity.
- Alignment then derives from the element type's byte width. `StorageScalar::byte_width` is the existing exhaustive derivation and is the natural authority; do not add a second width table.
- `ByteAlignment::new` already refuses non-powers-of-two and must keep doing so, for the reason its doc gives: divisibility is a partial order over powers of two and not over arbitrary integers.
- Alignment subsumption stays divisibility. A widened dtype must not weaken the relation.
- If the derivation turns out to belong in `tiler-ir` rather than `tiler-compiler`, that is a scope change to report, not to absorb silently.

## Required evidence

- An `f32` boundary still derives four-byte alignment, and every existing ABI fixture is unchanged.
- A two-byte element derives two-byte alignment, exercised through a real boundary rather than a unit call on the constant.
- A boundary whose declared alignment does not satisfy its derived requirement is refused, and the refusal is observed.
- Whether the derivation enters canonical identity is stated explicitly, and if it does, the moved identity is recorded.

## Closes when

Boundary alignment derives from the element type at every site that consumes `F32_NATURAL`, `f32` behaviour and fixtures are unchanged, a narrower element derives a narrower alignment through a real boundary, the refusal path is observed failing, and the doc comment naming this gap is replaced by one describing what the code now does.

## Graph maintenance

- Independent of the semantic and target children; it can land in parallel with either.
- Gates `admit-bf16-into-the-schedule-and-kernel-vocabulary`, which introduces the first element type whose width is not four.
- The comment quoted above is the specification for this ticket. When the derivation lands, that comment is stale and must be corrected in the same change — a doc comment describing an absent mechanism is a defect once the mechanism exists.

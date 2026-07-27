---
id: widen-shapeenv-factorization-fragment
title: Decide support for factorizations with multiple runtime-unknown terms
status: awaiting-decision
priority: p2
dependencies: []
related: [implement-shapeenv-constraints, implement-shapeenv-index-bindings]
scopes: [implementation/ir, contracts/foundation]
shared_scopes: []
paths: []
tags: [shapes, indexing, mature-product, needs-tom]
---
The current ShapeEnv deliberately accepts a decidable fragment. It rejects a
split relation such as `parent == outer * tile` when more than one factor
remains unknown at runtime.

## Facts

- Non-static `BindingSource` variants name where a future value comes from; they
  do not carry that value.
- `AvailabilityPhase` says when a binding may become available. It cannot make
  an unknown value determined.
- Only `BindingSource::Static(Extent)` currently contributes a constant to
  fragment checking.
- General nonlinear integer constraints over multiple 64-bit unknowns do not
  have a bounded complete procedure suitable for this compile path without a
  budget and an explicit undecided result.

The former proposed resolution—treat a compile-phase caller parameter as known
from its phase—cannot be implemented in the current model because no supplied
value exists to substitute.

## Atomic product decision

Must one artifact support a split whose factors remain unknown until launch?

- **Refuse it.** Every built ShapeEnv remains decided; a frontend specializes
  on a static factor and that value participates in artifact identity. This is
  the current fail-closed behavior.
- **Support it.** First introduce either an explicit specialization input that
  supplies and identities a compile-time value before ShapeEnv construction,
  or an `Undecided` constraint state propagated through every consumer. An
  availability phase alone is not a third option.

## Recommendation

Keep fully launch-dynamic factorizations unsupported until a frontend or product
requirement needs one artifact to serve arbitrary runtime factors. Explicit
refusal preserves the accepted meaning that a built environment is decided and
does not preclude a later specialization-input design.

## Counterpoint

If one artifact must serve arbitrary runtime tile sizes without recompilation,
specialization is insufficient; Tiler must accept the broader undecided-state
cost and define how every optimizer, verifier, artifact, and runtime consumer
handles it.

## Closes when

Tom confirms the product requirement. The shape contract records either the
explicit refusal and reopening trigger or the admitted value/undecided model,
and no implementation infers a known value from availability alone.

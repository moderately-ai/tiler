---
id: name-the-unprovable-symbolic-extent-diagnostic
title: Give an unprovable symbolic extent its own region diagnostic
status: todo
priority: p2
dependencies: []
related: [implement-shapeenv-index-bindings]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, indexing, diagnostics]
---
**Fact — what landed.** `implement-shapeenv-index-bindings` makes the region verifier refuse an access over a domain whose symbolic extent the `ShapeEnv` neither bounds nor determines. It reports that refusal as the existing `IndexRegionDiagnostic::BoundsNotProven` or `WriteOwnershipNotProven`, chosen by access mode.

**Fact — that is sound but imprecise.** Both are refusals in the taxonomy `docs/ir.md` establishes, so nothing is misclassified: "a result carrying only proof-resource diagnostics leaves its predicates open, while one carrying any other diagnostic is a rejection whatever else accompanies it." The landed code is deliberately *not* `ProofResourceLimit`, because no enumeration stopped. It has the precedent of `verify_access_exhaustively`, which reports the same pair when a tensor's element count is unrepresentable.

**What is missing.** The diagnostic does not say *why*. A consumer reading `BoundsNotProven` cannot distinguish "interval propagation overlapped a boundary and the finite fallback disproved it" from "the extent is symbolic and the environment bounds it nowhere". Only the second is fixable by adding a constraint, and that is the action a frontend would need to be told to take.

`IndexRegionDiagnostic` is already `#[non_exhaustive]`, so adding this
diagnostic follows the repository's additive-growth convention. The public
meaning still needs to be documented and tested rather than hidden under the
older generic refusal.

## Closes when

A missing symbolic bound produces a distinct diagnostic naming the affected
access or dimension and extent symbol. Genuine interval failure continues to
produce `BoundsNotProven` or `WriteOwnershipNotProven`; positive and negative
neighbors prove the distinction, and `make full` passes.

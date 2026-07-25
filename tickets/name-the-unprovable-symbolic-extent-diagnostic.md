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

**Why it was not done in that ticket.** Adding a variant to the public `IndexRegionDiagnostic` is a public API addition, which that ticket's worker was not authorized to make. It is filed rather than implied.

## Closes when

An unprovable symbolic extent reports a distinct diagnostic naming the dimension and the symbol whose bound is missing, the existing refusal taxonomy is preserved, a test pairs it with the interval-proved neighbour, and `uv run --locked python scripts/check_repository.py` passes.

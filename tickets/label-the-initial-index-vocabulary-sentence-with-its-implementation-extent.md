---
id: label-the-initial-index-vocabulary-sentence-with-its-implementation-extent
title: Label the initial-index vocabulary sentence with its implementation extent
status: todo
priority: p3
dependencies: []
related: [refresh-the-l2-derivation-s-symbolic-index-profile-source-claims]
scopes: [contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
---
## User-visible outcome

A reader of `docs/ir.md`'s bounded initial index vocabulary can tell the admitted contract from the implemented subset, so the L2-refresh class of staleness — a contract sentence read as a source fact — has one fewer site to arise at.

## The finding, from the L2 symbolic-profile refresh

**Fact.** `docs/ir.md:1037` states the bounded initial index vocabulary admits "addition/negation, multiplication by a parameter-only expression, and Euclidean floor division/modulo by a proven-positive parameter-only expression". The implemented index layer admits a `ShapeSymbol` only as a floor-division or modulo divisor (`IndexRegionBuilder::linear_combination` takes `IndexInteger` coefficients; no constructor accepts a symbolic addend or coefficient), so two of the three admitted forms are contract-only today. No ticket owns that divergence directly; the nearest is `admit-live-extent-operands-to-payload-indexing` (todo), which owns the payload-consumable half.

**The question, which is smaller than a correction.** The sentence states an *admitted* vocabulary, which is the corpus's normal ordering — contracts may lead implementation. What it lacks is a maturity label separating "admitted" from "implemented" per AGENTS.md's reserved-type/seam/implemented/tested discipline. Decide whether the sentence gains an explicit extent label (with the implemented subset named and the gap's owner cited), or whether the surrounding section already carries the distinction and only a cross-reference is owed. Read the whole section before deciding; do not weaken the contract to match the implementation.

## Closes when

The sentence and its section state the contract/implementation split explicitly, verified by a full section read, with the gap's owning ticket cited.

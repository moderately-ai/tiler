---
id: decide-domain-dimension-symbolic-view
title: Decide the domain dimension's symbolic view
status: awaiting-decision
priority: p2
dependencies: []
related: [promote-the-symbolic-index-profile-to-a-public-boundary]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [indexing, public-boundary, decision]
---
## Decision needed (2026-07-28)

**is `DomainDimensionRef::sourced_extent` the right additive view, or a narrower `symbol()` accessor?**

Touches `crate::index::model` (`DomainDimensionRef`) and `crate::index::sourced` (`SourcedExtent`).

```rust
match dimension.sourced_extent() { SourcedExtent::Static(e) => .., SourcedExtent::Symbol(s) => .. }
// versus
if let Some(symbol) = dimension.symbol() { .. } else { let extent = dimension.static_extent().expect("total with symbol()"); }
```

| | `sourced_extent()` (as drafted) | narrower `symbol()` |
| --- | --- | --- |
| **Enables** | One total view: a consumer matches two cases and the compiler tells it when a third arrives. `docs/ir.md` reserved exactly this additive view beside `static_extent()`. | The public surface grows by one accessor returning `Option<&ShapeSymbol>` and **`SourcedExtent` itself stays private**, which is one fewer type in the compatibility commitment. |
| **Prevents** | Publishing it publishes `SourcedExtent` — a two-case enum whose cases are the profile's own vocabulary, and whose `#[non_exhaustive]` decision then has to be made too. | Totality by construction. `static_extent()` returns `Some` exactly when the dimension is not symbolic — pinned by `crates/tiler-ir/src/index/sourced.rs::static_extent_is_absent_exactly_for_a_symbolic_dimension` — so the pair is total *today*, but nothing in the types says so, and a third source kind would make both accessors return `None` with no consumer forced to notice. |

**Recommendation: `sourced_extent()`.** The totality argument decides it: the narrower pair is total only by a test, and the exhaustive match is total by the type. A third source kind is not hypothetical vocabulary-widening in the abstract — `SourcedExtent`'s own doc says it is "deliberately two cases and not an expression tree", which is a boundary that could move.

**Counterpoint:** publishing `SourcedExtent` publishes the shape of the profile's extent vocabulary, and that is the part most likely to move. `symbol()` keeps it private at the cost of a test-held invariant, and a `#[non_exhaustive] SourcedExtent` recovers most of the safety while leaving the match non-total for out-of-crate consumers — which is the third option and is worth stating as one.

Split from `promote-the-symbolic-index-profile-to-a-public-boundary`, whose recorded owner approval to promote was granted and never applied; the four shape decisions are atomic so one can be settled without re-opening the others. The parent keeps the promotion history and the approval record.

## Closes when

Tom records a decision; the parent applies it as part of the promotion.

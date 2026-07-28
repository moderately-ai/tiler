---
id: promote-the-symbolic-index-profile-to-a-public-boundary
title: Promote the sourced-extent index profile to a reviewed public boundary
status: awaiting-decision
priority: p2
dependencies: []
related: [implement-shapeenv-index-bindings, implement-shapeenv-core]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, shapes, indexing, api]
---
## Decision needed (2026-07-28)

**Split performed 2026-07-28:** the four decisions below were split into atomic `awaiting-decision` tickets — `decide-shapeenv-builder-attachment`, `decide-symbolic-extent-error-siting`, `decide-domain-dimension-symbolic-view`, `decide-shapeenv-module-path` — each carrying its full options/recommendation/counterpoint shape. They remain recorded here as the promotion's context; answer them on the splits.


**Promotion is approved in principle and blocked in practice.** Tom approved the promotion on 2026-07-25 (recorded below). It has not happened, and it cannot happen as a mechanical visibility change, because four of the shapes inside the boundary are ADR 0075 always-ask categories the moment they are `pub`. Each is stated below as its own decision with its own options; **this ticket closes when all four are answered and the accepted subset is `pub`.**

The four are independent — answering one does not constrain the others — so they can be answered in any order or in one pass. Each names which of the three drafts it touches, so the blast radius of each answer is visible before it is given.

### Decision 1 — does `with_shape_environment` stay a consuming builder step, or become a `new`-time argument?

Touches `crate::index::builder` (`IndexRegionBuilder`).

```rust
let mut builder = IndexRegionBuilder::new(registry)?.with_shape_environment(env);
// versus
let mut builder = IndexRegionBuilder::new(registry, Some(env))?;
```

| | Consuming builder step (as drafted) | `new`-time argument |
| --- | --- | --- |
| **Enables** | A caller authoring a purely static region never names shapes at all; `new` keeps one argument and the symbolic profile stays opt-in at the call site that wants it. | "This region has exactly one environment, fixed at construction" becomes a property of the type rather than of a doc comment. |
| **Prevents** | Nothing structurally — and that is the problem below. | Every existing `IndexRegionBuilder::new` call site changes, including the static ones that have no environment to pass. |

**Recommendation: `new`-time argument.** The drafted method's own documentation states the invariant — "A region has exactly one environment for the whole of its life, so this is a constructor-shaped step rather than a setter — a second environment would silently reinterpret extents already authored against the first" — and the body does not hold it: `self.sources = Some(ExtentSources::new(environment))` assigns unconditionally, with no guard, no error return, and no test that a second call is refused (`sed -n '753,756p' crates/tiler-ir/src/index/builder.rs`). A second call after `symbolic_dimension` therefore replaces the environment the symbols were resolved against, silently. While the surface is private that is a latent defect; once public it is a defect a consumer can reach.

**Counterpoint:** the same invariant can be held by making the setter fallible or by refusing a second call, which keeps `new` unchanged and costs one error variant. That is a smaller edit, and it is the right answer if the churn at the static call sites is judged worse than an invariant enforced by a runtime refusal rather than by arity.

### Decision 2 — does `SymbolicExtentError` stay a separate error type, or fold into `IndexBuildError`?

Touches `crate::index::sourced` (`SymbolicExtentError`) and `crate::index::builder` (`IndexBuildError`).

```rust
fn symbolic_dimension(&mut self, role: DomainRole, symbol: ShapeSymbol) -> Result<DimensionId, SymbolicExtentError>;
// versus
fn symbolic_dimension(&mut self, role: DomainRole, symbol: ShapeSymbol) -> Result<DimensionId, IndexBuildError>;
```

| | Separate type (as drafted) | Folded into `IndexBuildError` |
| --- | --- | --- |
| **Enables** | Three authorities stay distinguishable in the type: `Source(ExtentSourceError)` is the environment's refusal, `Structural(IndexBuildError)` is the index layer's own limit, and `Shape(ShapeError)` is the shape vocabulary's bound. The variant documentation already argues that collapsing the last two "would report one limit's rejection under the other's name". | One error type for the whole builder; a caller matches once. |
| **Prevents** | A caller handling both static and symbolic dimensions matches two error types and converts between them. | The layering: `SymbolicExtentError::Structural` **wraps** `IndexBuildError` today (`crates/tiler-ir/src/index/sourced.rs:368-381`), so folding is not flattening one type into another — it is inverting the containment and adding `Source` and `Shape` variants to the type that is currently the inner one. |

**Recommendation: keep it separate.** The fold is not the cheap edit it looks like, for the containment reason above, and the distinction it would erase is one the source already argues for in prose.

**Counterpoint:** two error types on one builder is a real ergonomic cost, and `From<SymbolicExtentError> for IndexBuildError` cannot exist in the direction a caller wants without the same inversion. If the profile's dominant caller turns out to mix static and symbolic dimensions freely, that cost is paid on every call.

### Decision 3 — is `DomainDimensionRef::sourced_extent` the right additive view, or a narrower `symbol()` accessor?

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

### Decision 4 — is `ShapeEnv` exported from `tiler_ir::shape`, or does it get its own module?

Touches `crate::shape` (module layout) and every use path in the other two drafts.

```rust
use tiler_ir::shape::{ShapeEnv, ShapeEnvBuilder, ShapeSymbol};
// versus
use tiler_ir::shape::env::{ShapeEnv, ShapeEnvBuilder, ShapeSymbol};
```

| | Flat re-export from `tiler_ir::shape` | `pub mod env` under `shape` |
| --- | --- | --- |
| **Enables** | Matches the precedent already inside this module: `shape` keeps `mod evidence;` private and re-exports its accepted items flat (`pub use evidence::{Rank, ShapeEvidence, ShapeExpectation, StaticShape};`, `crates/tiler-ir/src/shape.rs:13`). The accepted subset is then visibly a *subset* — what is not re-exported is not public. | The environment's items stay grouped, and `shape::env::constraint` has an obvious home if it is ever published too. |
| **Prevents** | `shape`'s flat namespace grows by the environment's whole accepted vocabulary, and a reader loses the grouping. | Publishing the module publishes the *module path* as a commitment, and the granular choice of which items are public gets made file by file rather than at one `pub use` list. |

**Recommendation: flat re-export.** The precedent inside the same file is the strongest evidence available, and it makes the accepted subset explicit at one reviewable line rather than distributed across item visibilities.

**Counterpoint:** `env` is larger than `evidence` and carries a submodule (`constraint`), so the precedent is being extended from a small vocabulary to a whole authority. If `constraint` is ever promoted, the flat namespace holds two vocabularies that a reader would have to know are separate.

## Recorded history

### Decision — Tom, 2026-07-25

**Approved: promote.** ADR 0075 reserves public-surface promotions to the owner; this one is granted.

Covers all three ShapeEnv drafts — `shape::env`, `env::constraint`, and `index::sourced`. The authority, its constraint environment, and its first consumer are complete and tested, and were unreachable outside `tiler-ir`. Note the fragment boundary is still being widened by `bind-shapeenv-sources-into-tensor-boundaries-and-coefficients`, so promote the surface that is settled and say plainly which parts are still moving.

**The approval was never applied, and the one-line proof is:** `grep -n 'mod env;' crates/tiler-ir/src/shape.rs` prints `11:pub(crate) mod env;` and `grep -n 'mod sourced;' crates/tiler-ir/src/index/mod.rs` prints `12:pub(crate) mod sourced;` (2026-07-28). Nothing is reachable outside `tiler-ir`.

### Parked 2026-07-27 — awaiting Tom

Each of the four shapes above was chosen to be cheap to revise *while private*, so promoting without deciding them would spend that option for nothing. That is why the approval alone did not close this ticket.

## What is currently crate-internal

`crate::shape::env` (scoped symbols, typed root bindings, the constraint environment, identity); `crate::index::sourced` (`SourcedExtent`, the phase ceiling, `ExtentSources`); `IndexRegionBuilder::with_shape_environment` and `symbolic_dimension`; `VerifiedIndexRegion::extent_sources`; and `DomainDimensionRef::sourced_extent`, which is the additive borrowed view `docs/ir.md` reserved beside `static_extent()`.

`implement-shapeenv-core`, `implement-shapeenv-constraints`, and `implement-shapeenv-index-bindings` each landed under ADR 0074 convention 7: implemented, tested, and `pub(crate)`, because each ticket states that "any consequential public or cross-crate boundary remains a draft until Tom reviews and accepts the exact implementation commit".

**Why this is a ticket and not a mechanical change.** Promotion is the point at which the boundary becomes a compatibility commitment, and the four shapes above were chosen to be cheap to revise while private.

Every module involved carries a `dead_code` allow whose stated reason is exactly this draft status — file-scope at `crates/tiler-ir/src/shape/env.rs:1-4` and `crates/tiler-ir/src/index/sourced.rs:1-4`, item-scope at `crates/tiler-ir/src/index/builder.rs:750` and `:772`. Promotion removes those allows; it must not be done by adding a caller that exists only to satisfy the lint.

## Closes when

All four decisions above are answered; Tom has reviewed the exact boundary, the accepted subset is `pub` with its documentation and `#[non_exhaustive]` decisions made, and draft `dead_code` allows are removed or narrowed for every promoted item. A still-private reservation may keep an item- or submodule-level allow whose reason names the unavailable producer or consumer and reopening trigger; a whole-file draft allow must not survive merely because some reserved work remains. `make full` passes.

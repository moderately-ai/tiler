---
id: decide-symbolic-extent-error-siting
title: Decide where symbolic-extent errors are sited
status: closed
priority: p2
dependencies: []
related: [promote-the-symbolic-index-profile-to-a-public-boundary]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [indexing, public-boundary, decision]
closed_reason: superseded
closed_note: Requirements consolidated into the coherent sourced-extent and semi-affine public-boundary draft.
---
## Decision needed (2026-07-28)

**does `SymbolicExtentError` stay a separate error type, or fold into `IndexBuildError`?**

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

Split from `promote-the-symbolic-index-profile-to-a-public-boundary`, whose recorded owner approval to promote was granted and never applied; the four shape decisions are atomic so one can be settled without re-opening the others. The parent keeps the promotion history and the approval record.

## Closes when

Tom records a decision; the parent applies it as part of the promotion.

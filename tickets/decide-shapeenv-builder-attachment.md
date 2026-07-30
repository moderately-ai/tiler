---
id: decide-shapeenv-builder-attachment
title: Decide how a shape environment attaches to the index builder
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

**does `with_shape_environment` stay a consuming builder step, or become a `new`-time argument?**

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

Split from `promote-the-symbolic-index-profile-to-a-public-boundary`, whose recorded owner approval to promote was granted and never applied; the four shape decisions are atomic so one can be settled without re-opening the others. The parent keeps the promotion history and the approval record.

## Closes when

Tom records a decision; the parent applies it as part of the promotion.

---
id: decide-shapeenv-module-path
title: Decide the ShapeEnv public module path
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

**is `ShapeEnv` exported from `tiler_ir::shape`, or does it get its own module?**

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

Split from `promote-the-symbolic-index-profile-to-a-public-boundary`, whose recorded owner approval to promote was granted and never applied; the four shape decisions are atomic so one can be settled without re-opening the others. The parent keeps the promotion history and the approval record.

## Closes when

Tom records a decision; the parent applies it as part of the promotion.

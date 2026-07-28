---
id: split-the-index-builder-by-construction-phase
title: Split the index builder by construction phase
status: done
priority: p2
dependencies: []
related: [promote-the-symbolic-index-profile-to-a-public-boundary, admit-semi-affine-index-expression-class]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [refactor, ir, indexing, progressive-disclosure]
---
Make the index-construction path readable without changing its public meaning or
adding another crate.

`tiler-ir/src/index/builder.rs` currently combines public construction,
access formation, scalar reduction, proof-related validation, canonical
compaction, identity formation, and a large test module. Keep
`tiler_ir::index::builder` as the disclosure point and separate those phases
beneath it.

## Outcome

The module root explains the construction lifecycle and owns stable re-exports.
Deeper files each own a coherent invariant: access construction, reduction,
proof checks, compaction, and identity. Extract the large test module first so
the production split can be reviewed independently.

Preserve public paths and error behavior. Do not use the refactor to widen the
index-expression vocabulary or settle the symbolic-index public boundary.

## Closes when

The common construction path is visible from the module root, each phase has
one clear owner, public paths and canonical identities are unchanged, and the
full gate passes.

## Outcome — 4,665-line builder split by construction phase (2026-07-27)

`crates/tiler-ir/src/index/builder.rs` went from **4,665 lines to 2,220**, and the root now reads as the construction lifecycle: declare tensors and dimensions, build expressions, record accesses, apply operations, name outputs, `build`.

| file | lines | invariant it owns |
| --- | --- | --- |
| `builder.rs` | 2,220 | the lifecycle — public constructors, draft state, handle resolvers |
| `builder/identity.rs` | 732 | canonical encoding; `encode_region` and `encoded_region_len` are one asserted pair |
| `builder/proof.rs` | 691 | the obligations a draft discharges — **collects every diagnostic, never the first** |
| `builder/compact.rs` | 611 | alpha-equivalence — every ordering derives from a content key, never draft position |
| `builder/reduction.rs` | 232 | a reducer body canonicalizes independently of its containing region |
| `builder/tests.rs` | 284 | resource-ordering tests |

**Splitting the 2,650-line `impl IndexRegionBuilder` was the substance of this, not the free-function moves.** `proof` and `compact` each carry their own `impl IndexRegionBuilder` block; methods reached across the sibling boundary widened from private to `pub(super)`, and nothing crossed the crate boundary. That is what let the phases separate by invariant rather than by which functions happened to be free.

**`reduction` is beside the sequence rather than in it**, and the root says so: a reducer body is canonicalized independently of the region containing it, so it is not a step between proof and compaction.

**Public paths and error behaviour are unchanged.** `tiler_ir::index::builder` remains the single disclosure point, no re-export moved, and 273 `tiler-ir` tests pass — including the diagnostic-set assertions that would catch `verify` returning early instead of accumulating.

**The refactor widened nothing.** No index-expression vocabulary changed and the symbolic-index public boundary is untouched, per this ticket's instruction; `promote-the-symbolic-index-profile-to-a-public-boundary` still owns that question.

**One deliberate lint exception.** The phase modules glob-import their parent; `clippy::wildcard_imports` is denied workspace-wide, so each carries an `#![allow]` with a `reason` — they are private children of one module, every name they use is defined in that parent, and enumerating them would restate the parent's imports on every change.

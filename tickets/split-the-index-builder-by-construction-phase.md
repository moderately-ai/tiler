---
id: split-the-index-builder-by-construction-phase
title: Split the index builder by construction phase
status: todo
priority: p2
dependencies: []
related: [promote-the-symbolic-index-profile-to-a-public-boundary, admit-semi-affine-index-expression-class]
scopes: [implementation/ir]
shared_scopes: []
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

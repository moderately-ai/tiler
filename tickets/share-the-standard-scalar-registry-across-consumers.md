---
id: share-the-standard-scalar-registry-across-consumers
title: Move ad-hoc scalar registries onto the standard scalar profile
status: todo
priority: p2
dependencies: [wire-capability-and-refinement-into-compile-path]
related: []
scopes: [implementation/ir, implementation/reference]
shared_scopes: []
paths: []
tags: [implementation, testing, milestone-0b]
---
`FrozenScalarRegistry::standard()` now exists, but the consumers the originating ticket named still compose ad-hoc scalar registries: `crates/tiler-ir/tests/index_region.rs` (eight sites), `crates/tiler-ir/src/index/builder.rs`, `crates/tiler-compiler/src/capability.rs`, `crates/tiler-compiler/src/legality.rs`, and `crates/tiler-reference/tests/index_region_oracle.rs`.

Not all of them should move. Several exist precisely to exercise an *externally registered* scalar vocabulary through the public builder, which is coverage the standard profile would remove rather than duplicate. Decide per site which of the two it is, and move only the ones whose subject is the governed vocabulary.

**Closing evidence.** Each remaining ad-hoc registry carries a one-line reason naming what it tests that the standard profile does not, and the sites whose subject is the governed vocabulary compose `FrozenScalarRegistry::standard()`.

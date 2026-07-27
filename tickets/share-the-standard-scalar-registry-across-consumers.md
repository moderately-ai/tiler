---
id: share-the-standard-scalar-registry-across-consumers
title: Move ad-hoc scalar registries onto the standard scalar profile
status: todo
priority: p2
dependencies: [wire-capability-and-refinement-into-compile-path]
related: []
scopes: [implementation/ir, implementation/compiler, implementation/reference]
shared_scopes: []
paths: []
tags: [implementation, testing, milestone-0b]
---
`FrozenScalarRegistry::standard()` now exists and governed compiler and
reference fixtures already use it. Ad-hoc registries remain in index-region,
capability, legality, and reference-oracle tests. Several deliberately vary a
provider revision or exercise externally registered scalar vocabulary, while
others may still be historical setup.

Not all of them should move. Classify the remaining sites by the behavior under
test, move only fixtures whose subject is the governed vocabulary, and document
why each intentional custom registry cannot use the standard profile.

**Closing evidence.** Each remaining ad-hoc registry carries a one-line reason
naming what it tests that the standard profile does not, and every site whose
subject is the governed vocabulary composes
`FrozenScalarRegistry::standard()`.

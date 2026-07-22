---
id: prototype-semantic-index-refinement
title: Verify semantic-to-index refinement
status: todo
priority: p0
dependencies: [prototype-operation-capability-registry, harden-compiler-verifier-subject-binding-and-totality]
related: [prototype-canonical-index-region-slice]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-foundation, refinement]
---
Verify capability output against exact semantic occurrences and canonical index
regions. Bind ordered values and accesses, numerical/effect evidence, scalar
authority, reached definitions, selected-provider provenance, and reusable
content separately from occurrence identity. Registration or successful
builder construction alone is not refinement evidence.

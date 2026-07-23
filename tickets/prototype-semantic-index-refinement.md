---
id: prototype-semantic-index-refinement
title: Verify semantic-to-index refinement
status: in-progress
priority: p0
dependencies: [prototype-operation-capability-registry, prototype-generic-region-formation]
related: [prototype-canonical-index-region-slice]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, compiler-foundation, refinement]
claimed_from: todo
assignee: agent-prototype-semantic-index-refinement
lease_expires_at: 1784834740
---
Verify capability output against exact semantic occurrences and canonical index
regions. Bind ordered values and accesses, numerical/effect evidence, scalar
authority, reached definitions, selected-provider provenance, and reusable
content separately from occurrence identity. Registration or successful
builder construction alone is not refinement evidence.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

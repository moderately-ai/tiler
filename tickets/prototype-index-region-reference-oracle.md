---
id: prototype-index-region-reference-oracle
title: Implement the generic IndexRegion reference oracle
status: todo
priority: p0
dependencies: [prototype-canonical-index-region-slice, correct-reference-value-and-authority-contracts]
related: []
scopes: [implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, reference, indexing, oracle]
---
Implement a slow generic checked IndexRegion oracle in tiler-reference. Resolve
registered scalar evaluators without downcasting, execute ordered multi-result
SSA and N-state lexical reductions, preserve exact dtype bits and empty-domain
semantics, and fail closed for missing authority. Fusion legality must compare
against this independent path rather than a graph-specific host expression.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

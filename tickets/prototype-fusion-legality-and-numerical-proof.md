---
id: prototype-fusion-legality-and-numerical-proof
title: Derive fusion legality and numerical evidence
status: todo
priority: p0
dependencies: [prototype-operation-compilation-capabilities, prototype-canonical-index-region-slice, prototype-generic-region-formation]
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference]
shared_scopes: []
paths: []
tags: [implementation, optimizer, fusion, numerics]
---
Derive legality from operation capabilities, access/effect contracts, materialization boundaries, conversions, and numerical policy instead of graph-specific rule tables or asserted proof labels. Produce replayable evidence or typed Unknown/rejection and cover exceptional values, conversion rounding, contraction, empty domains, and reduction order.

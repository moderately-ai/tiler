---
id: prototype-fusion-legality-and-numerical-proof
title: Derive fusion legality and numerical evidence
status: todo
priority: p0
dependencies: [prototype-operation-compilation-capabilities, prototype-canonical-index-region-slice, prototype-generic-region-formation, correct-reference-value-and-authority-contracts, harden-compiler-verifier-subject-binding-and-totality, repair-numerical-witness-integrity]
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference]
shared_scopes: []
paths: []
tags: [implementation, optimizer, fusion, numerics]
---
Derive legality from operation capabilities, access/effect contracts, materialization boundaries, conversions, and numerical policy instead of graph-specific rule tables or asserted proof labels. Produce replayable evidence or typed Unknown/rejection and cover exceptional values, conversion rounding, contraction, empty domains, and reduction order.

The proof output must distinguish reusable refinement content from its checked
binding to one exact region occurrence, value/access mapping, reached semantic
definitions, selected providers, and evidence. It must not place provider or
whole-program identity into pure index structure.

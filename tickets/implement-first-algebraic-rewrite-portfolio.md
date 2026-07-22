---
id: implement-first-algebraic-rewrite-portfolio
title: Implement the first algebraic rewrite portfolio
status: todo
priority: p1
dependencies: [implement-transactional-rewrite-engine, implement-first-profile-numerical-policies]
related: []
scopes: [implementation/compiler, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, rewrites, numerics]
---
Add the first separately reviewed algebraic alternatives with named rules,
explicit semantic and numerical preconditions, reference-oracle comparison,
positive/negative tests, stable explain, and bounded search. Do not fold this
portfolio into canonical normalization or fusion-region formation.

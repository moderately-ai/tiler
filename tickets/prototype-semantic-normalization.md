---
id: prototype-semantic-normalization
title: Implement bounded semantic normalization
status: todo
priority: p0
dependencies: [prototype-typed-explain-infrastructure, correct-reference-value-and-authority-contracts]
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, normalization]
---
Introduce the deterministic normalization stage before region formation. The
first profile may be identity-only or contain a deliberately tiny proved rule
set, but it must establish termination, traversal order, budgets, semantic and
reference revalidation, transactional failure, canonical identity, and typed
explain records. Normalization must not imply the later alternative-producing
rewrite engine.

---
id: prototype-runtime-routing-commit
title: Implement one-way runtime routing commit
status: todo
priority: p0
dependencies: [prototype-runtime-artifact-validation]
related: []
scopes: [implementation/runtime]
shared_scopes: []
paths: []
tags: [implementation, runtime, routing, correctness]
---
Implement a state boundary preserving fallback authority only before one-way commit and consuming it before allocation, encoding or submission. Demonstrate fallback is uncallable afterward; semantic invalidity and corrupt artifacts fail closed rather than becoming route misses.

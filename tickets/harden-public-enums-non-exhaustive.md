---
id: harden-public-enums-non-exhaustive
title: Harden public growth seams without weakening total maps
status: todo
priority: p2
dependencies: [resolve-non-exhaustive-recognizer-hole]
related: [prototype-apple-aot-driver, prototype-scheduled-region-ir, resolve-non-exhaustive-recognizer-hole, harden-kernel-vocabulary-recognizer-completeness, admit-semi-affine-index-expression-class]
scopes: [implementation/ir, implementation/metal-aot, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, api-hardening]
---
Classify selected public enums and output records from their current
construction and match sites so future growth is either compatible by design or
deliberately compile-time breaking.

## Rules

- Mark a type `#[non_exhaustive]` only when every out-of-crate consumer is a
  partial or forwarding consumer that can handle an unknown future value.
- Keep total identity maps, support recognizers, and other closed vocabularies
  exhaustive so a new variant produces a compile error at every authority that
  must classify it.
- For each verdict, record the current construction site and out-of-crate
  consumer that justifies it. Declarations and historical inventories are not
  evidence.

## Search seed

Re-derive the six schedule growth seams, the relevant Metal AOT input/output
types (`AppleSdk`, `OptimizationLevel`, `ArtifactProvenance`,
`CompiledArtifact`), and `IndexExprClass`. The list is a search seed, not a
closed authority; `admit-semi-affine-index-expression-class` depends on this
ticket because `IndexExprClass` currently lacks an additive-growth boundary.

Do not bundle unrelated doctest cleanup into this work.

## Closes when

Each selected type is classified from current call sites, compatible growth
seams are non-exhaustive, total maps and recognizers remain deliberately
exhaustive, negative compile coverage protects both directions, and `make full`
passes.

---
id: reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission
title: Reconcile the roadmap and public facades with the consumer-neutral mission
status: todo
priority: p0
dependencies: []
related: [accept-the-public-compiler-facade-boundary, own-operation-family-support-matrix]
scopes: [contracts/foundation, contracts/navigation, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [architecture, documentation, consumer-neutral, audit]
---
## User-visible outcome

A reader can identify Tiler's product boundary without learning a workload-specific
story first: Tiler accepts verified typed logical MIMO tensor programs, performs
target-independent logical optimization and target-aware physical planning, lowers
them to structured kernels and artifacts, and generically binds and executes those
artifacts. Consumers own models, training and inference loops, application/session
state, sampling, serving, and workload vocabulary.

## Evidence to reconcile

- **Fact:** `README.md`, `docs/vision.md`, `docs/architecture.md`, ADR 0005, ADR
  0006, and ADR 0069 describe a consumer-neutral tensor compiler with a pure MIMO
  semantic graph and one general compile boundary.
- **Fact:** `docs/roadmap.md` currently presents language-model inference as the
  goal rather than one conformance workload.
- **Fact:** the public crate facades do not contain transformer or KV-specific
  types, but the general graph/compiler entrypoints are not yet presented as one
  coherent consumer-facing surface.
- **Inference:** the accepted architecture is sound; navigation, packaging prose,
  and workload framing have drifted from it.

## Work

Read the complete documentation entry path and all crate manifests and `lib.rs`
facades. Correct mission, roadmap, glossary, architecture packaging, and integration
language that assigns consumer concerns to the compiler or runtime. State explicitly
that workloads may supply conformance fixtures and optimization motivation without
becoming semantic/runtime abstractions. Record the current facade maturity without
inventing or accepting a new public API; any consequential public boundary remains
Tom's.

## Closes when

The entry path, roadmap, architecture, glossary, and integration overview agree on
the boundary; stale packaging/facade claims are corrected; searches for workload
ownership language have been reviewed in context; and every remaining exception is
either justified as conformance evidence or represented by a linked ticket.

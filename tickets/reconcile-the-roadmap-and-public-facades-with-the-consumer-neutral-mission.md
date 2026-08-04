---
id: reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission
title: Reconcile the roadmap and public facades with the consumer-neutral mission
status: done
priority: p0
dependencies: []
related: [accept-the-public-compiler-facade-boundary, own-operation-family-support-matrix, clarify-the-inline-frontend-facades-consumer-scope]
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

## Outcome

The repository entry path now defines Tiler as a consumer-neutral AOT compiler
and execution toolkit over pure explicit MIMO programs. `docs/vision.md` and
`docs/architecture.md` assign invocation composition, models, training and
inference loops, KV retention, sampling, serving, and application/session state
to consumers while keeping device pipelines and asynchronous resource lifetimes
as physical runtime obligations.

The roadmap now puts atomic building blocks and a minimum correct general
physical route before advanced optimization, and reclassifies its
language-model ladder as a conformance track. The glossary no longer publishes
the proposed `KV state` and `Live state scope` as repository-wide terms. The
status and design map expose the general-DAG, multi-output, operation/dtype,
physical-baseline, and backend-authoring gaps.

The packaging block was recomputed from `cargo metadata --no-deps
--format-version 1`: it now includes the three proof/integration executables and
the current `tiler`, `tiler-macros`, and `tiler-runtime` edges. The Markdown
contract identifies `tiler` as the inline Rust frontend facade rather than a
universal compiler facade.

Two bounded remainders stay explicit. The workload research corpus and its
ticket graph are owned by `reclassify-language-model-work-as-a-conformance-track`
and `supersede-the-runtime-owned-kv-state-design`. Rust source documentation
still carries the unqualified universal-consumer wording; it cannot be edited
under the current code freeze, so
`clarify-the-inline-frontend-facades-consumer-scope` is deferred until Tom lifts
that freeze and preserves the public-boundary review obligation.

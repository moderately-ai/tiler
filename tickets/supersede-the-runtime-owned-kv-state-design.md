---
id: supersede-the-runtime-owned-kv-state-design
title: Supersede the runtime-owned KV-state design with generic invocation contracts
status: todo
priority: p0
dependencies: [reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission]
related: [design-autoregressive-state-and-kv-cache, define-the-runtime-kv-state-boundary, establish-a-dynamic-kv-physical-layout-authority]
scopes: [contracts/foundation, contracts/integrations, contracts/navigation, research/runtime, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [architecture, runtime, consumer-neutral, supersession]
---
## User-visible outcome

The runtime executes an artifact invocation from explicit bindings and returns
explicit outputs. It does not own a KV cache, model cursor, layer ordinal, decode
generation, poisoned model state, or any other transformer-specific session object.
Consumers express repeated invocations and retained tensors by composing ordinary
program inputs and outputs.

## Evidence and disposition

- **Fact:** the accepted semantic graph is pure, finite, acyclic MIMO dataflow.
- **Fact:** no KV-specific public type is present in the merged crate facades at
  the ticket's creation base.
- **Fact:** `docs/research/runtime/autoregressive-state-and-kv-cache.md`,
  `docs/research/runtime/dynamic-kv-physical-layout.md`, and the L5 ticket chain
  assign workload-specific state ownership to the generic runtime.
- **Inference:** generic facts found by that work remain useful: explicit bindings,
  extent validation before routing commit, placement/context compatibility,
  asynchronous resource retention, atomic publication, and fail-closed execution.
  Their KV-named ownership conclusion does not.

Audit the complete research records, glossary entries, roadmap dependencies, and
ticket chain. Preserve measurements and generic contracts while clearly superseding
the runtime-owned KV abstraction. Recast valid workload cases as integration or
conformance tests over ordinary tensors. Close, supersede, split, or rewrite each
affected ticket so no scheduler path can recreate the rejected abstraction. Preserve
the unmerged `tkt/define-the-runtime-kv-state-boundary` draft as review evidence; do
not merge it and do not silently delete its rationale.

This ticket is documentation and graph maintenance only. It does not authorize code
changes or a new public runtime boundary.

## Closes when

Every KV/model-state ownership claim has an explicit disposition; retained generic
requirements have consumer-neutral owners and correct dependencies; workload tests
bind explicit tensors through the ordinary invocation boundary; and no ready ticket
can introduce a KV-, transformer-, model-, or decode-specific core/runtime type.

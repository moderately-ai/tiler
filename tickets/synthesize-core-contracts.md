---
id: synthesize-core-contracts
title: Synthesize semantic graph, shape, and extension contracts
status: done
priority: p1
dependencies: [semantic-graph-contract, shape-environment-contract, operation-extension-surface, proc-macro-extension-visibility]
related: []
scopes: [contracts/core]
shared_scopes: []
paths: []
tags: [tiler-research, synthesis, decision]
---
Reconcile the completed foundation evidence into the core architecture, IR, extension, glossary, open-question, roadmap, and ADR documents. Preserve proposed alternatives where evidence is insufficient and add durable ADRs only for decisions actually made with Tom.

Acceptance requires cross-document terminology consistency, explicit graph invariants, a documented extension trust boundary, and traceability from each changed contract to its research evidence.

## Outcome

- Contracts: [architecture](../docs/architecture.md), [IR](../docs/ir.md), and [operation extensions](../docs/operation-extensions.md)
- Decisions: [ADRs 0005–0008](../docs/decisions/0005-public-semantic-tensor-graph.md) and [ADRs 0044–0045](../docs/decisions/0044-use-explicit-capability-operation-registry.md)
- Result: reconciled graph, shape, identity, binding, registry, and proc-macro visibility evidence into the proposed core contracts and accepted decisions.

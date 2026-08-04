---
id: audit-the-general-compiler-pipeline-against-the-semantic-program-model
title: Audit the general compiler pipeline against the semantic program model
status: todo
priority: p1
dependencies: []
related: [implement-general-dag-partitioning, admit-ordered-multi-output-programs-at-the-compiler-request-boundary, accept-the-public-compiler-facade-boundary]
scopes: [research/program-planning, research/region-search, research/scheduling, contracts/optimizer, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [architecture, compiler-pipeline, audit, mimo]
---
## User-visible outcome

The documented and planned compiler pipeline demonstrably accepts the public typed
MIMO semantic program rather than only the three shapes recognized by the first
prototype.

Trace the complete construction and consumption path for the eleven intended stages:
semantic verification, normalization, logical exploration, region enumeration,
lowering-capability resolution, index-region lowering, complete-cover enumeration,
scheduled-region exploration, complete physical-plan selection, structured-kernel
refinement, and kernel-program assembly. Then trace backend emission, artifact build,
runtime preflight/routing commit, and execution.

For each boundary record its typed input/output, governing identity, validation and
explain obligations, unsupported cases, and whether merged code, a contract, only a
proposal, or no owner exists. Pay special attention to general DAGs, ordered named
outputs, multi-result operations, symbolic extents, materialization, transfers,
memory lifetimes, and the current premature `select_supported_strategy` collapse.

This is a read/design audit. It may create and repair tickets and documentation but
does not authorize implementation or acceptance of a public compiler facade.

## Closes when

Every stage has a maturity classification and exact owner; missing bridges are filed
with dependency-correct edges; no physical choice has leaked into semantic identity;
and the critical path to a naive but general compiled MIMO program is explicit.

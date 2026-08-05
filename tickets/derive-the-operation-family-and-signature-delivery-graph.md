---
id: derive-the-operation-family-and-signature-delivery-graph
title: Derive the operation-family and signature delivery graph
status: in-progress
priority: p1
dependencies: [enumerate-the-mature-tensor-operation-and-signature-taxonomy]
related: [own-operation-family-support-matrix, admit-the-registered-unary-families-at-the-compiler-request-boundary]
scopes: [contracts/navigation, research/semantic-graph, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, roadmap, ticket-graph]
claimed_from: todo
assignee: agent-delivery-graph
lease_expires_at: 1785943730
---
## User-visible outcome

The operation taxonomy becomes an executable research and delivery plan rather than
a long aspirational list.

For every taxonomy family/signature partition, map the maturity rungs separately:
semantic identity, validation and shape inference, reference semantics, logical
rewrite participation, index/access lowering, minimum physical realization, backend
realization, and bounded conformance evidence. Group signatures only where one
correctness argument and one implementation really cover them; split when numerical
contracts, compound storage, effects, or backend feasibility differ.

Create at least one design/spike/audit owner for every unsupported family. Create
implementation tickets only when their prerequisite contracts and acceptance
boundaries are resolved. Give deferred families explicit activation triggers and
file them as `deferred`, not dispatchable `todo`. Connect existing tickets instead of
duplicating them, and correct the operation-family support matrix where the taxonomy
shows that a row was too broad.

## Closes when

Every taxonomy row reaches a live owner or a justified deferred node, dependencies
run from semantics and reference correctness toward lowering/backends, exact dtype
signatures are visible, and no umbrella "support all operations" ticket hides an
unbounded implementation scope.

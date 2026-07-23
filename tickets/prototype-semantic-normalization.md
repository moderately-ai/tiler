---
id: prototype-semantic-normalization
title: Implement bounded semantic normalization
status: in-progress
priority: p0
dependencies: [prototype-typed-explain-infrastructure, correct-reference-value-and-authority-contracts]
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, normalization]
claimed_from: todo
assignee: agent-prototype-semantic-normalization
lease_expires_at: 1784827223
---
Introduce the deterministic normalization stage before region formation. The
first profile may be identity-only or contain a deliberately tiny proved rule
set, but it must establish termination, traversal order, budgets, semantic and
reference revalidation, transactional failure, canonical identity, and typed
explain records. Normalization must not imply the later alternative-producing
rewrite engine.

This stage owns one relocated obligation. Tom accepted on 2026-07-18 that
identical referentially transparent operation invocations normalize to one
semantic value before computation identity — equality requiring the same
operation key, operands, canonical attributes, numerical contract, and inferred
result types, with source origins preserved for explanation but excluded from
equality. ADR 0064 later placed common-subexpression elimination outside
commitment compaction and in "existing later layers"; Tom confirmed on
2026-07-23 that this relocated the obligation rather than cancelling it, and
that this normalization stage is its home. Implementing it here is in scope
whenever the first profile's proved rule set admits it; deferring it is
acceptable only with an explicit note recording that the obligation remains
open and unowned elsewhere. Physical planning may still recompute a shared
value independently when that is cheaper than reuse or materialization.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

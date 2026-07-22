---
id: prototype-typed-explain-infrastructure
title: Implement typed optimizer explain infrastructure
status: in-progress
priority: p0
dependencies: [reconcile-implementation-work-graph-after-authority-audit, harden-compiler-verifier-subject-binding-and-totality]
related: []
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, explain, authority]
claimed_from: todo
assignee: gpt-sol-explain
lease_expires_at: 1784743250
---
Implement one bounded typed explain authority shared by normalization, region,
feasibility, costing, selection, and refinement stages. Stable stage,
disposition, reason/rule/provider keys, subject references, evidence classes,
predicates, and exact budget stops are data; rendered strings are presentation.
Require deterministic ordering, bounded retention, causal errors, and stable
positive and negative conformance fixtures.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

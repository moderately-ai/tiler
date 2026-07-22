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

## Review handoff

The implementation draft deliberately keeps `tiler_compiler::explain` private.
It adds a sealed, request-qualified `VerifiedExplainTrace` to each successful
target compilation product and migrates the current normalization, fusion,
feasibility, costing, selection, kernel, program, and artifact-plan flow to
typed records. Canonical identity uses an explicit schema encoding; the
versioned text renderer is presentation only. Retention is bounded across the
complete canonical trace, preserves the current portfolio's terminal selection
records, and emits a typed truncation record.

Tom still needs to review these public-boundary choices before this ticket can
be completed:

- compiler-owned public module versus a future `tiler-explain` crate;
- how successful and failed compilations return partial or complete reports;
- whether canonical traces are serialized or embedded in artifacts;
- which renderer guarantees, retention controls, and provider-detail/redaction
  policy form part of the public contract.
- whether public enums are non-exhaustive, versioned schema views, or both;
- which component may mint trusted evidence receipts for external providers;
- whether the public identity is canonical bytes, a specified digest, or both;
- how much of the request-qualified renderer header is stable versus redacted.

No provider emission trait is proposed yet. The draft keeps emission
compiler-owned and exposes only private read accessors, avoiding a premature
extension contract while preserving typed provider and rule identities.

The request-qualified trace boundary begins only after a
`VerifiedTargetRequest` exists. Malformed requests, request-budget failures,
unsupported semantic signatures, and semantic-output preflight failures retain
their existing typed errors and do not forge a target-qualified
`VerifiedExplainTrace`. A future public facade may carry separate unverified
attempt evidence, but that is a distinct authority and API decision.

The correction pass after immutable review also keeps terminal decisions in a
typed ledger: successful traces require one selected alternative and one
selection disposition per feasible or target-rejected considered alternative;
failed target compilations retain their original typed source plus one terminal
failure record. Configured limits govern optional detail only. Mandatory
decisions and the typed truncation summary are retained independently under the
compiler's hard aggregate bound, so detail retention cannot change planning.

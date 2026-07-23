---
id: prototype-generic-region-formation
title: Implement generic fusion-region formation
status: todo
priority: p0
dependencies: [prototype-semantic-normalization]
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, fusion, milestone-0b]
---
Replace proof-graph recognition and hard-coded occurrences with deterministic
bounded enumeration of connected convex regions from arbitrary supported DAGs.
Include singleton coverage, boundaries, retained named/multi-result outputs,
fan-out handling, stable identity and budgets; compare small graphs with an
exhaustive oracle. Define separate canonical region-content and graph-
occurrence identities so identical content at distinct occurrences remains
shareable without losing exact coverage or boundary bindings.

## Outcome

`crates/tiler-compiler/src/region.rs` implements `EnumerateRegionCandidates` as
a deterministic bounded enumerator over an arbitrary verified semantic DAG,
running immediately after `NormalizeSemantics`. The prior hand-written
serial-Sum recognizer in `fusion.rs` (fixed `SemanticOccurrence`/`CandidateKind`
roles) is deleted; `fusion.rs` now only proves strict-`f32` numerical legality
for one generic region candidate, and `physical.rs` carries graph-local
`SemanticMemberId` coverage instead of the fixed role vocabulary.

**What the first profile enumerates.** For any verified program it seeds from
every operation in ascending order and grows connected sets across
producer/consumer value edges, emitting exactly the connected, convex candidates
within the deterministic budgets. Complete singleton coverage is emitted
unconditionally before any growth budget can fire, so the unfused plan is never
lost. Each candidate carries member operations, boundary inputs, retained
multi-result outputs (a value consumed outside the region or named as a program
result is a retained boundary output, so one region may produce several ordered
values), and a `DuplicationPolicy`. Region-content identity (members renumbered
to canonical local positions) is kept separate from graph-occurrence identity
(the exact site in canonical coordinates), so reusable content at a different
site shares one content identity. On the governed strict-`f32` serial-Sum
program this is 17 candidates (5 singletons + 12 grown); the shared-constant
normalized spelling is 10. Enumeration is proven equal to an independent
exhaustive subset oracle on four fixtures (chain, diamond, shared producer,
shared constant), every emitted candidate is oracle-legal, and singleton
coverage is complete.

**What the first profile does not do.** Producer duplication is DISABLED
(`DuplicationPolicy::Disabled` on every candidate); the exhaustive oracle retains
duplicated covers only as a completeness witness. It proposes only — it selects
no cover, chooses no implementation, lowers no index region, plans nothing
physical, and costs nothing. Non-singleton candidate identity depends on the
canonical member order being complete; a residual tie falls back to graph order,
which can split two genuinely interchangeable occurrences into distinct content
identities (a lost reuse opportunity, never a conflation). Budgets
(`region_members`, `region_boundary_outputs`, `region_live_values`,
`region_candidates_per_seed`, `region_expansions`, defaulting to the
optimizer.md safety numbers) stop a growth path with a typed `BudgetStop`
explain record and never remove singleton coverage.

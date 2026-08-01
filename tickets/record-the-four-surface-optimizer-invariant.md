---
id: record-the-four-surface-optimizer-invariant
title: Record the four-surface optimizer invariant in the contracts
status: in-progress
priority: p1
dependencies: []
related: [implement-transactional-rewrite-engine, route-the-compile-path-through-the-rewrite-engine, emit-analytical-costs-through-the-typed-cost-vocabulary, drive-an-external-physical-implementation-provider-through-compilation]
scopes: [contracts/optimizer, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, optimizer, architecture, backend-providers]
claimed_from: todo
assignee: worker-invariant
lease_expires_at: 1785598277
---
## User-visible outcome

The property that keeps the optimizer rewrite-proof across execution tiers and backends is a contract sentence a worker inherits, instead of a synthesis a reader must assemble from five documents — so the first landing that violates it is caught by review against a named rule rather than by someone noticing the drift.

## The invariant, as decided

**Fact — Tom set the direction on 2026-08-01:** physical-plan optimization must operate generically over every execution tier and backend, so that optimizer and selection logic is never rewritten when a device family arrives. The derivation he reviewed states the enforcing invariant:

The optimizer sees exactly four surfaces and nothing else —

1. **Neutral alternatives**: schedules in the execution-axis / tile / synchronization vocabulary of `crates/tiler-ir/src/schedule/`, never a backend construct.
2. **Typed permissions**: reassociation, contributor permutation, and FMA-contraction consumed from the operation's own registered numerical contract — legality is target-independent.
3. **Feasibility queries**: whether a target realizes an alternative is answered from typed profile data (atomic realization facts in the ADR 0043/0076 shape), never by calling backend code; a target lacking a tier starves those alternatives at feasibility with an explainable reason rather than forking enumeration.
4. **Typed costs**: the analytical cost vocabulary, with hard feasibility never expressed as a cost.

Backends contribute *data* (facts, realizations) and *alternative generators* (providers whose output re-enters the neutral vocabulary and passes the same verifier and feasibility), never search logic. ADR 0090 items 1 and 2 are the accepted authorities for the two halves; this ticket records the composed consequence where optimizer workers read.

## What to write, and where

- The invariant stated once in the optimizer contract (`docs/compiler/` — `fusion-and-scheduling.md` or the contract document the corpus treats as the optimizer's home; read the existing structure and place it where enumeration and selection are described), with each of the four surfaces citing the implementation that carries it today.
- One sentence in `docs/architecture.md`'s separation-of-concerns text linking to it, since the invariant is the optimizer-specific instance of the compiler-core-independence rule already there.
- The review obligation stated explicitly: nothing mechanical checks this — an execution-tier or backend landing that touches selection machinery is the signal a reviewer must treat as a violation until justified. Cite the evidence that the discipline holds: the cooperative-workgroup tier (2026-08-01) landed without touching selection, and the tree-reduction strategy is required to do the same.
- Do not restate the schedule vocabulary or the provider seam; cite them. A restatement is a second authority that drifts.

## Closes when

The invariant is stated in the optimizer contract with its four surfaces cited to living code, the architecture contract links to it, the review obligation is explicit, and no sentence restates what a cited authority already owns.

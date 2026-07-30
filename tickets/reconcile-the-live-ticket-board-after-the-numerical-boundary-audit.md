---
id: reconcile-the-live-ticket-board-after-the-numerical-boundary-audit
title: Reconcile the live ticket board after the numerical boundary audit
status: done
priority: p1
dependencies: []
related: []
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: []
---
# Reconcile the live ticket board after the numerical boundary audit

## Goal

The active board states the implementation and documentation work that remains after the numerical boundary audit without stale decision gates, backwards dependencies, duplicated outcomes, or universal dtype assumptions.

## Gap

Several active tickets still describe an earlier two-variant numerical vocabulary, treat correctness-derived internal designs as product decisions, or make producers depend on the artifacts that consume them. Two decision tickets ask whether to retain behavior that is already the tested fail-closed contract. Related dtype and reduction tickets also carry stale dependency edges that either overconstrain an unselected quantized workload or deadlock work behind the follow-up it is meant to trigger.

## Work

- Derive every status and edge change from the current source, accepted contracts, and full ticket bodies.
- Replace false decision gates with implementation-ready tickets that preserve a later exact public-boundary review.
- Reorder target-fact provenance, caller profile declaration, Metal adaptation, and delivered-record design from producer to consumer.
- Merge or close outcomes only after preserving their remaining requirements in a live ticket.
- Keep conditional dtype, quantization, and enforcement work conditional until a selected workload names an exact consumer.
- Treat orphan Git branches as separate reconciliation evidence; do not delete them as ticket cleanup.

## Acceptance

Every changed ticket has a current user-visible outcome, source-backed implementation keys, and graph maintenance instructions; the nonterminal dependency graph is acyclic and lint-clean; reproduced stale claims no longer appear in active tickets; no live outcome is lost; each edited check is perturbed once and observed failing; `tkt guard`, `git diff --check`, and `make full` pass.

## Outcome

The awaiting-decision queue fell from twenty tickets to the two genuine product choices: the first Metal LM workload and inline macro syntax. Five ShapeEnv/index splits, dtype dispatchability, per-target outcomes, and one numerical-contract documentation ticket were consolidated without losing their requirements. The caller-profile/provenance/Metal/delivered-record chain now runs producer to consumer. Premature unsafe and dynamic-factorization work is deferred behind explicit triggers, the cache boundary waits on its two concrete signature drafts, and conditional quantized dependencies stay conditional.

No ticket was deleted because every candidate retained useful rationale or history. Forty-three stale `tkt/*` branches were separately proven to have zero unique commits and no attached worktree; they were not deleted because branch administration is outside this ticket cleanup.

The lint check's failure path was demonstrated with a temporary self-cycle and then restored. Detached review at `3fad568e76d48c0cdfb0dd1709e519308d7a03ed` found no blocking issue or lost outcome. `tkt guard` found no under-declared scope, `git diff --check` passed, and `make full` passed with 1,296 workspace tests, the doc-tests, and 487 release numerical tests.

## Refs

- `AGENTS.md`
- `docs/numerical-semantics.md`
- `docs/decisions/0076-record-numerical-honourability-as-target-profile-facts.md`
- `crates/tiler-compiler/src/honourability.rs`
- `crates/tiler-compiler/src/request.rs`

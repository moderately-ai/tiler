---
id: reconcile-the-live-ticket-board-after-the-numerical-boundary-audit
title: Reconcile the live ticket board after the numerical boundary audit
status: in-progress
priority: p1
dependencies: []
related: []
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: codex-root
lease_expires_at: 1785427894
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

## Refs

- `AGENTS.md`
- `docs/numerical-semantics.md`
- `docs/decisions/0076-record-numerical-honourability-as-target-profile-facts.md`
- `crates/tiler-compiler/src/honourability.rs`
- `crates/tiler-compiler/src/request.rs`

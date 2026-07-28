---
id: rebase-and-land-the-stranded-numerical-policies-worktree
title: Rebase and land the stranded numerical-policies worktree
status: todo
priority: p1
dependencies: []
related: [implement-first-profile-numerical-policies]
scopes: [implementation/compiler, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, integration]
---
A functionally complete implementation of `implement-first-profile-numerical-policies` exists ONLY as uncommitted work in an abandoned worktree, and none of it is on main.

## Facts (verified 2026-07-28)

- Worktree: `.claude/worktrees/agent-ad2893b1fba4d7f5b`, branch `tkt/implement-first-profile-numerical-policies`, HEAD `06af0c6` (2026-07-25), 313 commits behind main at audit time.
- Uncommitted there: `crates/tiler-compiler/src/policy.rs` (722 lines, `NumericalPolicyP…` vocabulary), `crates/tiler-reference/src/conformance.rs` (367 lines), modifications across 11 further files, a 58-line Outcome section on the ticket, and THREE untracked follow-up tickets (including `record-the-arithmetic-type-in-the-num…`).
- The lease was stripped by `fdf68a2` while status stayed `in-progress`; nothing on disk or in the graph claims the work.

## Why a dedicated ticket rather than a status flip

Reverting the parent to todo without this record would invite a fresh worker to re-derive 1,100+ lines from a 13-line stub. Landing is a real rebase across 313 commits of drift — the compiler grew the rewrite engine, the opaque-call stack, the analytical cost model, and `NormalizationOutcome` changed shape — so the diff will not apply mechanically and its Outcome claims must be re-verified against current source, not trusted.

## Approach

1. From the worktree, commit the work onto its branch as-is first (preserve it before touching anything).
2. Rebase onto main, resolving against the current normalize/rewrite shape; the follow-up tickets ride along.
3. Run the ticket's own conformance tests plus the full gate; land via the standard integration flow; close the parent on its actual outcome.

## Closes when

- The work (or its deliberate replacement) is on main, the parent ticket's status reflects reality, the three follow-up tickets are filed, and the worktree is removed by the coordinator after verifying nothing uncommitted remains.

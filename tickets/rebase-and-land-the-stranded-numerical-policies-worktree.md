---
id: rebase-and-land-the-stranded-numerical-policies-worktree
title: Rebase and land the stranded numerical-policies worktree
status: done
priority: p1
dependencies: []
related: [implement-first-profile-numerical-policies]
scopes: [implementation/compiler, implementation/reference]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, integration]
---
## User-visible outcome

A worker picking up numerical-policy work finds the first-profile implementation **on main** — policy vocabulary, conformance oracle, and tests — instead of a 13-line stub whose real implementation sits invisible in an abandoned worktree. Until this lands, any numerics ticket risks re-deriving 1,100+ lines that already exist.

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

## Graph maintenance (do these as outcomes arrive, not at the end)

- **Before touching anything**: commit the worktree's staged work onto its own branch exactly as found. If you skip this and the rebase goes wrong, the only copy of the work is gone.
- **When the rebase lands**: close `implement-first-profile-numerical-policies` as done *citing the landing commit*, and paste its worktree Outcome section into the ticket only after re-verifying each claim against the merged tree — the Outcome was written against a base 300+ commits stale and is labelled "Claimed, not verified against main" for that reason.
- **File the three follow-up tickets** found untracked in the worktree (one is `record-the-arithmetic-type-in-the-num…`). Before filing, fix any `Closes when` that cites the retired Python gate — the gate is `make full` now.
- **If the rebase conflicts on `NormalizationOutcome`, the rewrite engine, or `component_cost`**: those subsystems landed after the branch base. Resolve toward main's shape and re-run the branch's own conformance tests; do not resolve toward the branch on any file main's audit commits touched.
- **If you conclude the work should be re-implemented rather than rebased**: say so on this ticket with the specific conflict that decided it, keep the worktree until the replacement lands, and only then ask the coordinator to remove it (`git worktree remove`, never `rm`).
- **When done**: the coordinator (not you) removes the worktree after verifying nothing uncommitted remains.

## Landed (2026-07-28)

Executed exactly per the protocol above. Preservation commit `1e449d2` on the branch before anything was touched; squash-merge onto main; every conflict resolved toward main's structure (pipeline split, reference facade, feasibility split) with the branch's intent re-applied in the new layout; both moved identity pins (explain trace digest, governed descriptor) recomputed on the merged tree with reasons at the sites; the three follow-up tickets landed with their retired-gate citations corrected to `make full`; `component_cost`'s width arm taught the third contract key so relaxed plans keep their memory-traffic bound; the opaque-call admission threads the contract's arithmetic type into `assess_resources` alongside the scheduled path. Parent closed as done citing the landing commit. Worktree removal is the final step after the landing commit exists.


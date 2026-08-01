---
id: re-point-the-boundary-property-enforcer-edges-after-the-provider-seam-landed
title: Re-point the boundary-property enforcer edges after the provider seam landed
status: todo
priority: p1
dependencies: []
related: [implement-general-dag-partitioning, implement-boundary-property-enforcers, drive-an-external-physical-implementation-provider-through-compilation]
scopes: [project/tickets]
shared_scopes: []
paths: []
tags: [graph-repair, dependencies]
---
## User-visible outcome

A p1 ticket that owns two open questions stops being permanently unreachable behind a parked dependency its own text says is probably backwards, and the enforcer ticket's restart condition gains the graph edge it names in prose.

## Why this exists

**Fact — a p1 `todo` ticket can never reach `ready`.** [`implement-general-dag-partitioning`](implement-general-dag-partitioning.md) is `todo` at p1 and depends on [`implement-boundary-property-enforcers`](implement-boundary-property-enforcers.md), which is `deferred` — a parked state that never satisfies a dependent. It is the only p1 among the todo tickets with a parked dependency.

**Fact — the ticket's own text says the edge is probably backwards and asks for a re-read the frontier makes impossible.** Its Dependency note (2026-07-28, `implement-general-dag-partitioning.md:19`) concludes: "So this work is likely to be what unblocks the enforcers rather than something blocked behind them, and the dependency should be re-read — **not merely re-checked** — when this ticket is picked up." A ticket that can never surface in `ready` is never picked up, so the re-read it schedules for itself cannot happen.

**Fact — it owns two open questions.** `docs/open-questions.md:144` (Q-PLAN-002) and `:160` (Q-PLAN-005) both name `implement-general-dag-partitioning` as owner. Their owner being unreachable is ownership in name and orphanhood in fact — the failure mode [`re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal`](re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal.md) exists for, in the one variant that sweep does not detect because the owner is not terminal.

**Fact — the enforcer ticket's restart condition names a ticket it holds no edge to.** [`implement-boundary-property-enforcers`](implement-boundary-property-enforcers.md):58 records that the startable condition is no longer the constant test failing: it is "**a compile-path provider proposes an opaque call whose contract the composing consumer refuses**", which "arrives with caller-supplied physical providers". [`drive-an-external-physical-implementation-provider-through-compilation`](drive-an-external-physical-implementation-provider-through-compilation.md) is exactly that work, is p1 and dispatchable, and no edge connects them.

## Work

Read the two tickets against each other in full — not their summaries — and set the edges the evidence supports. Two facts bear on the re-read and neither is settled by this ticket's framing:

- **ADR 0078's correction keeps opaque declaration and registration crate-private.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):125 restates it: "Out-of-crate opaque-call registration stays compiler-owned and crate-private per ADR 0078's correction; this record adds only that no caller of any kind registers one on the compile path, which [`register-opaque-calls-on-the-compile-path`](register-opaque-calls-on-the-compile-path.md) owns as an internal wiring gap." So whether an *out-of-crate* provider can produce the refused handoff at all is part of the re-read, not an assumption it may start from.
- **`PhysicalAuthorities::composed` still has no production caller.** Verify before relying on it either way (`grep -rn "PhysicalAuthorities" crates/`), and record what the check showed.

Then add the missing edge from `drive-an-external-physical-implementation-provider-through-compilation` to the enforcers ticket in whichever direction the re-read supports, so the restart condition is a graph fact rather than a sentence.

## Boundaries

- Scope is `project/tickets` alone: this ticket edits ticket files. It implements no partitioning, no enforcer, and no provider.
- Do not resolve the direction by deleting the dependency. If the enforcers genuinely precede partitioning, say so with the evidence; if partitioning unblocks the enforcers, invert the edge and say why. Either is an outcome; "removed the edge to unblock the frontier" is not.
- This ticket also carries one one-line prose repair too small for its own brief: `draft-the-backend-provider-composition-adr.md:49` links `route-a-custom-backend-through-a-registered-runtime-adapter.md`, renamed to `route-a-custom-backend-through-an-independently-selected-adapter.md` at `622cf62`. It is the only ticket-to-ticket link break among 823 resolved targets, so it is a repair, not evidence of a class needing a sweep.

## Closes when

`implement-general-dag-partitioning` can reach `ready` or its blocked state is a stated finding rather than an unexamined inversion; the re-read is recorded with the two facts above checked rather than assumed; the enforcer restart condition has a graph edge; Q-PLAN-002 and Q-PLAN-005 have a reachable owner; and `tkt lint` passes with every edge resolving.

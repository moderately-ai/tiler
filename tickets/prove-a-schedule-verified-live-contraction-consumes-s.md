---
id: prove-a-schedule-verified-live-contraction-consumes-s
title: Prove a schedule-verified live contraction consumes S
status: in-progress
priority: p1
dependencies: [accept-the-live-extent-operand-public-surface]
related: [admit-live-extent-operands-to-payload-indexing]
scopes: [implementation/ir, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, contraction, extents, identity]
claimed_from: todo
assignee: worker-prove-live-contraction
lease_expires_at: 1786662228
---
## User-visible outcome

A bounded direct contraction consumes the live input-axis extent `S` as its contributor-loop bound and performs exactly `S` loads, without baking `S` into the schedule or kernel identity.

## Exact gap

**Fact on draft `9a8f53c937dc9b9f777a1d4b361cadc1a0b0316e`.** `ReductionTopology::LiveContraction` and `LogicalAccess::LiveRowMajor` exist as labelled draft variants. The parent ticket's second required evidence — a schedule-verified contraction that consumes `S` and changes the oracle when the bound is replaced — is not on that commit. The working construction path is `ScheduledRegionBuilder` + `lower_scheduled_region`, not `compile()`.

Continue from the preserved parent branch after Tom accepts the surface. Rebase onto current `main` rather than merging `9a8f53c9` unchanged if main has moved.

## Required work

- After [`accept-the-live-extent-operand-public-surface`](accept-the-live-extent-operand-public-surface.md) closes, use the accepted `LiveContraction` / `LiveRowMajor` spelling.
- A bounded direct contraction consumes `S` as its contributor-loop bound and performs exactly `S` loads.
- Replacing the bound value by the neighbouring extent changes the oracle. Baking either value changes identity and fails the no-specialization assertion.
- Omitted, swapped-symbol, wrong-axis, late-phase, overflowing, and unused live operands fail at the named layer. Remove each new check and watch its negative fail.

## Required evidence

- Schedule verification plus lowered kernel for at least two neighbouring `S` values, with load-count oracles that move and identities that do not.
- Subject perturbations for the named refusal classes, each with quoted failure text.
- Targeted IR and compiler tests, identity blast radius, `tkt lint`, `git diff --check`, exact-base guard, and the required repository gate.

## Non-goals

Artifact envelope, payload, and pipeline execution. `compile()` through strategy selection. Widening past bounded unsigned input-axis extents available by `LiveDevicePreflight`.

## Closes when

The accepted contraction spelling is schedule-verified, the neighbouring-extent oracle moves, baked neighbours change identity, and every named negative is fail-capable.

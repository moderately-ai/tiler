---
id: scope-the-sub-tensor-selection-fusion-role
title: Scope the sub-tensor selection fusion role
status: in-progress
priority: p3
dependencies: []
related: [scope-the-concatenate-fusion-role-and-lowering]
scopes: [research/indexing]
shared_scopes: [project/tickets]
paths: []
tags: []
claimed_from: todo
assignee: agent-slice-role
lease_expires_at: 1785963860
---
## User-visible outcome

The sub-tensor selection family's R5 gap gets the same treatment the concatenate got: a scoped fusion-role conclusion (existing role, new role, or refusal with grounds) so the support matrix's Slice row can name an owner instead of a comparison that went stale when the concatenate's role landed its scoping.

## Why this exists

**Fact, found by `carry-the-concatenate-scoping-conclusions-into-the-navigation-corpus` on 2026-08-05.** `docs/roadmap.md`'s Slice row reads "R5 needs a fusion role, which this family has no more of than the concatenation does" — refuted the day the concatenate scoping concluded `CoordinateRelation` covers it (`docs/research/indexing/concatenate-fusion-role-and-lowering.md`). No fusion-role ticket exists for this family, so the carrier correctly left the row alone rather than naming an owner that did not exist. This ticket is that owner.

## What this ticket owes

The concatenate scoping is the working precedent and likely a short path: sub-tensor selection is also a pure coordinate relation (a windowed read rather than an extended write), so the first question is whether the same four-candidate elimination lands on the same role with the obligations discharging on the same or stronger premises — derived against `derive_obligations` at the then-current base, not inherited from the concatenate's answer. The matrix row correction lands in the same change (the row is `contracts/navigation`; add the scope with a recorded reason at execution, checking live claims first).

## Closes when

The role conclusion is recorded with its elimination, the Slice row names this work's outcome instead of the stale comparison, and any follow-on implementation tickets are filed with correct edges.

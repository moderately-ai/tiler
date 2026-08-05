---
id: scope-the-sub-tensor-selection-fusion-role
title: Scope the sub-tensor selection fusion role
status: done
priority: p3
dependencies: []
related: [scope-the-concatenate-fusion-role-and-lowering, admit-a-fusion-role-for-the-sub-tensor-selection-slice, lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability, admit-the-sub-tensor-selection-family]
scopes: [research/indexing, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, fusion, slice]
---
## User-visible outcome

The sub-tensor selection family's R5 gap gets the same treatment the concatenate got: a scoped fusion-role conclusion (existing role, new role, or refusal with grounds) so the support matrix's Slice row can name an owner instead of a comparison that went stale when the concatenate's role landed its scoping.

## Why this exists

**Fact, found by `carry-the-concatenate-scoping-conclusions-into-the-navigation-corpus` on 2026-08-05.** `docs/roadmap.md`'s Slice row reads "R5 needs a fusion role, which this family has no more of than the concatenation does" — refuted the day the concatenate scoping concluded `CoordinateRelation` covers it (`docs/research/indexing/concatenate-fusion-role-and-lowering.md`). No fusion-role ticket exists for this family, so the carrier correctly left the row alone rather than naming an owner that did not exist. This ticket is that owner.

## What this ticket owes

The concatenate scoping is the working precedent and likely a short path: sub-tensor selection is also a pure coordinate relation (a windowed read rather than an extended write), so the first question is whether the same four-candidate elimination lands on the same role with the obligations discharging on the same or stronger premises — derived against `derive_obligations` at the then-current base, not inherited from the concatenate's answer. The matrix row correction lands in the same change (the row is `contracts/navigation`; add the scope with a recorded reason at execution, checking live claims first).

## Closes when

The role conclusion is recorded with its elimination, the Slice row names this work's outcome instead of the stale comparison, and any follow-on implementation tickets are filed with correct edges.

## Graph maintenance

- **`contracts/navigation` was added at execution on 2026-08-05, with the reason this section records.** Two files in that scope had to move in the same change as the record: `docs/roadmap.md`'s `Sub-tensor selection` trigger column, whose stale comparison is this ticket's stated outcome, and `docs/research/README.md`'s catalog block, because a new governed record's catalog row is edited in the same change that adds the metadata behind it. `tkt claims` was read first and no live holder declared the scope at that moment. One of the three concurrent claims, `survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature`, later added `contracts/navigation` and reached `done` on `main` while this ticket was in flight, so the guard reports a direct collision with it. **File-level disjointness was verified against what that work actually landed rather than assumed from topic:** `git diff --name-only 3cca2a3f origin/main` shows its `contracts/navigation` edit is `docs/research/README.md` alone and does not touch `docs/roadmap.md`, and `git merge-tree --write-tree HEAD origin/main` returns a single tree with no conflict, whose `docs/research/README.md` carries both catalog rows. This is declaration and scheduling metadata; it authorizes no product outcome.
- **The conclusion landed as a sibling record rather than as an extension of the concatenate's**, because that record states its own boundary — "this one answers only O-07's two cells" — and carries one `ticket` field and one title. Widening it to O-06 would have made its id, its title, and its ticket edge describe less than its contents.
- The two follow-on tickets are [`admit-a-fusion-role-for-the-sub-tensor-selection-slice`](admit-a-fusion-role-for-the-sub-tensor-selection-slice.md) (M4 / R5) and [`lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability`](lower-the-sub-tensor-selection-occurrence-through-one-index-access-capability.md) (M5, which the delivery graph left owed and unowned). Neither depends on the other, and the record states the reading of `derive_fusion_legality` and `derive_obligations` that makes that true.
- A factual error was corrected in two places outside this ticket's own outcome: the concatenate record and its role ticket both say the fusion-role table holds eight keys, and it holds nine at both commits involved. The correction is one word in each and is recorded in the new record so it is not read as a silent edit.

---
id: record-that-schedule-rule-8-cannot-check-the-proofs-region-identity
title: Record that schedule rule 8 cannot check the proof's region identity
status: todo
priority: p3
dependencies: []
related: [bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, gather, layering]
---
## User-visible outcome

The schedule layer says in its own source why it cannot check a gather proof's region identity, so a later reader does not add the check at the layer that structurally cannot make it.

## Why this exists

Found 2026-08-22 by the refinement-seam packet while establishing where the occupancy check belongs. This is a one-paragraph source note, filed rather than folded in because it guards against a specific future mistake.

**Fact — rule 8 has nothing to compare against.** `tiler_ir::schedule::IndexRegion` carries no `CanonicalIndexRegionIdentity` counterpart, so the comparison is not merely absent from rule 8 — it is unavailable there. The check therefore belongs in `tiler-compiler`, where the occurrence and its region identity are both in scope.

**Why this earns a note rather than silence.** Rule 8 compares four accessors and looks like the natural home for a fifth. A reader who notices the missing region comparison will reach for that spot first, find it cannot be done, and either force it or conclude no check is needed. Both outcomes are worse than a sentence saying where it lives.

## Required work

- Re-audit the Fact at your base with a verdict; confirm by reading that `IndexRegion` has no identity counterpart rather than by a failed search — **a failed search does not prove absence**.
- Add the note at `GatherAddressReadRule::ProofMismatch`, stating what rule 8 checks, what it structurally cannot, and which layer owns the occupancy comparison.
- Do **not** add a check here.

## Non-goals

Any behavioural change; the occupancy check itself, which is [`bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence`](bind-a-scheduled-gathers-retained-proof-to-its-own-occurrence.md); and any identity movement.

## Closes when

The source states where the region-identity comparison lives and why it cannot live at the schedule layer, and no behaviour has changed.

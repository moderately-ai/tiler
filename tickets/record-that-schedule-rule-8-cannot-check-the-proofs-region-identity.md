---
id: record-that-schedule-rule-8-cannot-check-the-proofs-region-identity
title: Record that schedule rule 8 cannot check the proof's region identity
status: done
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

## Coordinator verification — 2026-08-23: the corroboration is real, and one reason is worth carrying over

**The note is comment-only and correctly placed** — the diff carries no non-comment line, and it sits directly after the `Rule 8: proof mismatch` block a reader editing the check would land on.

**The lane's self-flag is accurate and worth keeping.** `CanonicalIndexRegionIdentity` under `crates/tiler-ir/src/schedule/` goes **0 → 1** across this change, because the new comment names the type in prose. That is not a counterpart appearing; the structural Fact is unchanged. Flagging it pre-empts a future re-run reading the count as a contradiction, which is the same class of false signal as a grep count that shrinks across a repair.

**The independent corroboration exists, though not as quoted.** The lane reported `gather_accesses_match`'s doc as already carrying this explanation. It does — `crates/tiler-compiler/src/physical.rs` reads *"rule 8 structurally cannot make this comparison, because `IndexRegion` carries no region identity to compare against … which is why the check is here"*. But the sentence it quoted back greps to **0**, because a `///` wrap falls between `cannot` and `make`; the fragments `rule 8 structurally`, `carries no region identity`, and `which is why the check is here` each return **1**. So the substance is confirmed and the quotation was reconstructed across the wrap — the same anchor hazard, appearing in a report rather than in a committed citation. The lane's committed anchors were each verified individually and are sound.

**One reason the compiler-side doc carries and this note does not:** *"(ADR 0070 keeps semantic correlation out of the schedule)"*. That is the architectural *why* behind the absence — the schedule layer has no identity counterpart by decision, not by omission. A future reader of the new note learns the check is unavailable and where it lives, but not that its absence is deliberate. Recorded rather than ticketed: the note is accurate and achieves the ticket's stated outcome, and adding the ADR reference is a one-line improvement any lane touching this block should make. **Reconsideration trigger:** the next change to rule 8 or to `gather_accesses_match` should carry the ADR 0070 reason into whichever note lacks it.

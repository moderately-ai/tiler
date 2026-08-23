---
id: retire-the-l4-zero-synchronization-ground-where-other-records-restate-it
title: Retire the L4 zero-synchronization ground where other records restate it
status: in-progress
priority: p2
dependencies: [reconcile-the-l4-records-self-contradicting-softmax-elimination-row]
related: []
scopes: [research/scheduling, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift, scheduling]
claimed_from: todo
assignee: worker-zerosync
lease_expires_at: 1787450840
---
## User-visible outcome

No record still eliminates threadgroup-cooperative softmax on a zero-synchronization ground, and none attributes that ground to the L4 record, which has withdrawn it.

## Why this exists

Filed 2026-08-22 when `reconcile-the-l4-records-self-contradicting-softmax-elimination-row` **withdrew** that elimination rather than re-grounding it. Both halves of the ground fell, and the coordinator verified the decisive ones:

- **The barrier is landed and Metal-proven.** `crates/tiler-build/src/metal_declaration.rs` declares the workgroup `ControlBarrier` subject at `SynchronizationSupport::Realized` from a production call site.
- **The surviving half is not a discriminator.** `tiler::softmax-f32@1` is registered and recognized; what it lacks is an installed lowering, so a softmax refuses under `UnsupportedCapability` rule `accuracy.elementary.no-installed-realization` — reached *before* a schedule is chosen, so it falls on every candidate topology alike and eliminates none relative to the others.

**The problem this ticket fixes is attribution.** Three documents restate the retired ground and **one quotes the L4 record by name**, so repairing L4 alone leaves the claim alive elsewhere *citing an authority that has withdrawn it*. Reported by that lane, unverified by the coordinator: `two-level-subgroup-workgroup-reduction.md` (2 sites), `multi-round-two-level-reduction-composition.md` (1), `autoregressive-state-and-kv-cache.md` (1).

Also reported: three stale claims in `crates/`, and one in the **`done`** ticket `implement-the-single-workgroup-synchronized-reduction-strategy` saying `metal_declaration.rs` declares no synchronization row — which the production `Realized` row falsifies in both halves. `make citations` reads only open tickets, so the gate has never seen that one.

## Required work

- Re-audit every site at your base with a per-Fact verdict; **re-derive the census yourself** and say which spellings you searched for and why that set is complete. A census is only as complete as its vocabulary — that is how a closed ticket shut green over live sites this week.
- Withdraw the ground where it appears, following L4's own precedent: the in-convention move is **withdrawal**, as the 2026-08-10 lane did three rows up in the same table for a structurally identical reason. Do not invent a replacement ground.
- Repair the attribution first where a record cites L4 by name — a claim borrowing a withdrawn authority is worse than a claim standing alone.
- **Preserve retired wording in dated corrections**; counts cannot shrink.
- The `crates/` sites and the terminal ticket are **out of this ticket's scopes** — report them rather than widening.

## A hazard this lane hit, worth inheriting

Its first draft asserted two false things about softmax, taken from a **stale doc comment** in `crates/tiler-compiler/src/request/recognize.rs` reading `carries no law at all`. An independent source sweep caught it before commit. **Do not source a claim from a doc comment without checking the code it describes** — that comment is still there.

## Non-goals

Re-deciding the softmax schedule set, which the L4 text explicitly leaves open; editing `crates/`; and the flash-class citation repairs, which are their own ticket.

## Closes when

No live record eliminates threadgroup-cooperative softmax on a zero-synchronization ground, no record attributes that ground to L4, every withdrawal preserves what it replaced, and the out-of-scope sites are reported with their owners named.

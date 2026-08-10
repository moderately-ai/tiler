---
id: admit-the-concatenate-family-into-the-scheduled-region-vocabulary
title: Admit the concatenate family into the scheduled region vocabulary
status: awaiting-decision
priority: p2
dependencies: []
related: [scope-the-concatenate-fusion-role-and-lowering, admit-the-structural-families-into-the-scheduled-region-vocabulary, admit-a-fusion-role-for-the-sequence-extension-concatenate, lower-the-concatenate-occurrence-through-partitioned-writes, accept-the-partitioned-concatenate-realization-law, accept-the-partitioned-write-ownership-proof-boundary, accept-the-sub-domain-write-domain-surface]
scopes: [implementation/compiler, implementation/ir, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [semantics, regions, decision, needs-tom, public-boundary]
---
## The gap, and why it was unowned

**Fact.** `tiler::concatenate-f32@1` is a registered family with a landed `CoordinateRelation` fusion role and per-arity index-access lowerings. The scoping record at [`scope-the-concatenate-fusion-role-and-lowering`](scope-the-concatenate-fusion-role-and-lowering.md) selected the design; [`admit-a-fusion-role-for-the-sequence-extension-concatenate`](admit-a-fusion-role-for-the-sequence-extension-concatenate.md) landed the `CoordinateRelation` role; [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md) landed the per-arity index-access lowerings and the `PartitionedConcatenate` law at `a86fddc2` on 2026-08-07. It is nonetheless in `UNPLANNED_OPERATIONS` (`crates/tiler-compiler/src/policy.rs`, source anchor `const UNPLANNED_OPERATIONS`), which states its own reason: **the request boundary refuses the family under `operation-set` because no kernel construct writes a partitioned output.**

**Correction — 2026-08-10.** An earlier wording of the Fact above attributed delivery of the role and lowerings to `scope-the-concatenate-fusion-role-and-lowering` at `a86fddc2`. That is false: the scope ticket is research/scoping and files implementation tickets; the hash is the lowering ticket's Outcome, not the scope ticket's. The corrected ownership is the three-ticket chain in the Fact.

So the family is recognized at the semantic, fusion, and index layers (registration, `CoordinateRelation`, realization law, and per-arity lowerings) and still cannot appear in a scheduled region: the request boundary never owns a concatenate occurrence, so planning refuses under `operation-set`.

**No ticket owned closing that.** [`admit-the-structural-families-into-the-scheduled-region-vocabulary`](admit-the-structural-families-into-the-scheduled-region-vocabulary.md) covers reindex and broadcast only. Found by the worker on `correct-the-recognizer-era-sentences-in-the-optimizer-contract`, which noticed that `docs/compiler/optimizer.md`'s "two registered families the region vocabulary cannot spell" is now **three**, and corrected the count only because it framed a paragraph it was already rewriting — flagging the underlying gap rather than absorbing it. The optimizer contract names no owner for the concatenation's widening (it does not name this ticket by id).

## What this owes

- A kernel construct that writes a **partitioned output**, which is the stated obstacle. The partitioned write-ownership vocabulary and the sub-range write domains already landed and are separately accepted, and the partitioned concatenate realization law was accepted on 2026-08-07 — so the semantic and index-region halves exist. What is missing is the *scheduled region* and *kernel* half.
- The `UNPLANNED_OPERATIONS` entry retired **with its stated reason**, not merely deleted: the comment there is the record of why the refusal existed, and it should be superseded rather than dropped.
- The refusal re-founded on whatever survives. If some concatenate shape remains unplannable, that boundary is asserted rather than left implicit — the pattern `establish-bf16-optimizer-legality` and the recognizer widening both followed.
- `docs/compiler/optimizer.md`'s count restated once the population changes; it currently says three and names no owner for the concatenation's widening.

## What is not this ticket

`tiler::slice-f32@1` is also registered and unplanned, but it holds **no governed lowering at all** — an uninstalled-provider case rather than this one. Do not bundle it; the two have different obstacles and different evidence.

Do not re-open the partitioned write-ownership contract, the sub-range write domains, or the concatenate realization law. All three are landed and separately accepted; this consumes them.

## Decision packet — 2026-08-09

The accepted law proves several partitioned write roots, but no accepted schedule/kernel construct carries them. That missing construct is a consequential public IR boundary and cannot be selected inside an implementation ticket.

- **Option A — admit one multi-root scheduled/kernel copy construct for the whole concatenate occurrence (recommended).** It preserves the accepted occurrence/law as one semantic unit and carries the already-proved disjoint ownership directly.
- **Option B — split the occurrence into one existing single-root region/kernel per operand.** This avoids a multi-root construct but introduces stage ordering, shared-output ownership, occurrence-accounting, and coverage relations that do not exist today.

Tom must choose the schedule/kernel boundary. The accepted partitioned law is not reopened by either answer.

## Identity

Admitting a family into the region vocabulary is likely to move pinned identities — the lowering registry, the request subject, and anything downstream. Enumerate the population before editing and recompute **on the merged tree**, not on the branch: two branches moved shared pins from different bases on 2026-08-07 and neither's values survived.

## Closes when

A concatenate occurrence reaches a scheduled region and a kernel that writes its partitioned output; the `UNPLANNED_OPERATIONS` entry is superseded with its reason recorded; any surviving refusal is asserted and watched failing; and every moved pin is enumerated and recomputed on the merged tree.

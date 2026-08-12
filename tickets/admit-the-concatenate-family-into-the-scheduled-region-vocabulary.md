---
id: admit-the-concatenate-family-into-the-scheduled-region-vocabulary
title: Admit the concatenate family into the scheduled region vocabulary
status: done
priority: p2
dependencies: []
related: [scope-the-concatenate-fusion-role-and-lowering, admit-the-structural-families-into-the-scheduled-region-vocabulary, admit-a-fusion-role-for-the-sequence-extension-concatenate, lower-the-concatenate-occurrence-through-partitioned-writes, accept-the-partitioned-concatenate-realization-law, accept-the-partitioned-write-ownership-proof-boundary, accept-the-sub-domain-write-domain-surface, admit-an-explicit-non-arithmetic-region-and-delivery-state, admit-the-partitioned-copy-scheduled-region, lower-the-partitioned-copy-region-through-kernel-ir, derive-target-numerical-feasibility-from-reached-arithmetic-only, plan-concatenate-through-one-partitioned-copy-entry, repair-the-scheduled-vocabulary-census-and-concatenate-law-standing]
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

## Source-first corrections — 2026-08-12

- **The missing population was understated.** The gap is not only schedule and kernel. Current `IndexRegion` requires one `ScalarProgram` and one `NumericalRealization`; resource derivation copies numerical fields; KIR and artifact entry records also require numerical claims. Concatenate is a bit-preserving copy and deliberately has no arithmetic capability row, so an explicit non-arithmetic region, requirement, delivery, and request-feasibility state is a prerequisite rather than fixture churn.
- **The slice comparison is false at this tree.** Slice now has `IndexRealizationLaw::slice_f32()` and a governed `GovernedSliceF32` lowering. It remains a separate single-root scheduling outcome, but it is no longer an uninstalled-provider comparator.
- **The optimizer census is stale.** Its “three-family” paragraph still describes reindex and broadcast as lacking scheduled access maps even though both have landed, and it omits newly lowered slice from the current wall population. The whole census and reasons must be re-derived, not merely have its number decremented.
- **Option B was not a reuse option.** The accepted concatenate law is one region, current regions prove total ownership individually, kernel-program verification rejects multiple stage writers, and artifacts carry no joint cross-entry shared-output proof. Splitting one occurrence into two through eight kernels would require a new multi-kernel shared-writer architecture and reopen the accepted law.
- **Identity consequences are conditional, not blanket.** The concatenate law and lowering-registry identities already moved when those rows landed. Append-only new schedule/program tags may preserve every existing byte; fixed KIR/artifact numerical records may require domain or schema movement if they become explicit sums. Every version and pin must be re-derived from the exact selected encoding rather than assumed here.
- **The accepted law still carries stale source standing.** `IndexRealizationLaw::PartitionedConcatenate` says it awaits the already-completed acceptance ticket. That label and the stale optimizer census are owned by [the bounded repair](repair-the-scheduled-vocabulary-census-and-concatenate-law-standing.md).

## Accepted decision — 2026-08-12

Tom accepted the strengthened one-region option in this thread. One whole concatenate occurrence becomes one scheduled `PartitionedCopy` program, one verified KIR, one backend entry, and one dispatch. It preserves the accepted operand-ordered partition law, including zero-extent members and two members for `concat(x, x)` even when they share one input binding.

The public boundary is an exhaustive arithmetic-versus-copy sum, not `ScalarProgram::Copy`, optional fields, or a fabricated strict numerical realization. A copy entry explicitly states that arithmetic numerical requirements and delivery are not applicable; silence, a nearby profile, and a default are not interpretations. Construction derives checked prefix offsets, requires the member rectangles to be pairwise disjoint and jointly exhaustive, and refuses overflow, gaps, overlap, rank/type disagreement, or unsupported maps by typed cause. Boundary buffers may be deduplicated, while ordered operand members may not.

The canonical first implementation is one output-domain/root-partition kernel with specialized ownership verification. It may use several mutually exclusive predicated stores only under one proof that exactly one member supplies every output position. Generic one-store verification is not weakened. A flattened-output source selector and an N-kernel realization are later explicit physical alternatives only after proving equivalent ownership; neither is an implicit fallback.

The accepted dependency chain is:

1. [explicit non-arithmetic region and delivery state](admit-an-explicit-non-arithmetic-region-and-delivery-state.md);
2. [the partitioned-copy scheduled region](admit-the-partitioned-copy-scheduled-region.md) and [reached-arithmetic feasibility](derive-target-numerical-feasibility-from-reached-arithmetic-only.md);
3. [its exact KIR lowering and verifier](lower-the-partitioned-copy-region-through-kernel-ir.md); and
4. [one-entry compiler, Metal, build, artifact, and conformance integration](plan-concatenate-through-one-partitioned-copy-entry.md).

The decision is recorded here; it does not authorize silently implementing around an unfinished prerequisite and does not itself move production or identity bytes.

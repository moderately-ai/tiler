---
id: scope-the-concatenate-fusion-role-and-lowering
title: Scope the concatenate family's fusion role and lowering
status: done
priority: p1
dependencies: []
related: [scope-an-in-place-append-into-a-caller-retained-allocation, admit-a-fusion-role-for-the-tensor-contraction, reach-a-verified-kernel-through-the-structural-families, derive-the-operation-family-and-signature-delivery-graph, admit-a-fusion-role-for-the-sequence-extension-concatenate, admit-a-partitioned-write-ownership-contract, lower-the-concatenate-occurrence-through-partitioned-writes, carry-the-concatenate-scoping-conclusions-into-the-navigation-corpus]
scopes: [research/indexing]
shared_scopes: [project/tickets]
paths: []
tags: [research, operation-families, concatenate, fusion, lowering]
---
## User-visible outcome

A program containing `tiler::concatenate-f32@1` derives a fusion legality other than `Unknown` and reaches a lowering, so the sequence-extension family stops being a registered identity that no plan can consume.

## Why this exists

**Fact — the family is registered and reference-evaluated and stops there.** [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s `Sequence extension` row records `tiler::concatenate-f32@1` at R4 with the exact-sum extent derivation, the typed refusals, and a bit-preserving evaluator, and states in its own words that "R5 needs a fusion role" and "R6 needs a structured-kernel construct and backend emission". The family "performs no arithmetic, so it deliberately has no `OperationNumericalCapability` row and appears in `UNPLANNED_OPERATIONS`".

**Fact — no ticket owns either rung.** Every other family whose fusion role is missing names its owner in the same row — the contraction's is [`admit-a-fusion-role-for-the-tensor-contraction`](admit-a-fusion-role-for-the-tensor-contraction.md), the structural families' backend rung is [`reach-a-verified-kernel-through-the-structural-families`](reach-a-verified-kernel-through-the-structural-families.md). The concatenate row names an owner for the in-place append ([`scope-an-in-place-append-into-a-caller-retained-allocation`](scope-an-in-place-append-into-a-caller-retained-allocation.md), deferred) and for the extent relation, and none for R5 or R6.

**Correction — 2026-08-10.** The two Facts above are the filing-time matrix and board, not live claims. The matrix maturity cell is now **R5 for the F32 family**, with R5 evidence dated 2026-08-06 under [`admit-a-fusion-role-for-the-sequence-extension-concatenate`](admit-a-fusion-role-for-the-sequence-extension-concatenate.md) (`CoordinateRelation` role landed). R6's index-access lowering and write-ownership contract tickets [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md) and [`admit-a-partitioned-write-ownership-contract`](admit-a-partitioned-write-ownership-contract.md) landed and are `status: done`; the trigger cell still says R6 needs a structured-kernel construct and backend emission. All four tickets this Outcome filed exist and are `done`. Reproduce: matrix row anchor `Sequence extension: \`Concatenate\``; frontmatter of the four filed ids.

**Fact — the lowering is a genuine fork rather than a missing keystroke, which is why this is a scoping ticket and not an implementation one.** [Q-SHAPE-006](../docs/open-questions.md#q-shape-006--finite-piecewise-access-maps) records the one live piecewise pressure in the corpus: "lowering the sequence-extension concatenate family needs either a piecewise read or two write roots partitioning one output. The second alternative is available, so the trigger has not fired; it fires if that alternative is eliminated." Choosing between them decides whether Q-SHAPE-006 fires, which is a consequence larger than one family.

**Inference — the demand is live rather than hypothetical.** The decode path appends to two caches per layer per step, and [`execute-the-decode-step-path`](execute-the-decode-step-path.md) and [`integrate-the-autoregressive-decode-loop`](integrate-the-autoregressive-decode-loop.md) are `todo` p1 above it. A family at R4 cannot carry them.

## What the work is

Derive, and record with the elimination rather than only the choice: which fusion role the family takes given that it applies no scalar operation and its write map is a windowed partition; whether the lowering is one piecewise read or two write roots over one output, costed against what each does to Q-SHAPE-006; and whether an inner-axis concatenate's loss of the contiguous-window realization is an applicability predicate on a physical candidate rather than a second semantic identity, which the matrix row already asserts and this work must check rather than inherit.

## Explicit non-goals

- The in-place append into a caller-retained allocation, which is [`scope-an-in-place-append-into-a-caller-retained-allocation`](scope-an-in-place-append-into-a-caller-retained-allocation.md)'s under Q-PLAN-015.
- Any second semantic family. Stacking is a unit-axis insertion followed by a concatenation and is deliberately not a third key.
- Moving a matrix rung. A scoping record delivers nothing.

## Closes when

The fusion role is derived with its legality argument, the lowering alternative is selected with the eliminated one recorded and Q-SHAPE-006's firing condition restated against the choice, and the implementation work is filed as its own ticket with the acceptance boundary named.

## Outcome

Delivered by [Concatenate fusion role and lowering](../docs/research/indexing/concatenate-fusion-role-and-lowering.md), written at `d5960e81`. Two eliminations, one restated trigger, four filed tickets.

**The fusion role.** `CoordinateRelation`, the existing role. Four candidates were tested against what `derive_obligations` actually decides rather than against what a role's name suggests: no role is the present state the ticket exists to end; `ValueSource` fails the role doc's own distinction (anchor `a value source contributes a value the` on the `CoordinateRelation` variant in `crates/tiler-compiler/src/fusion_legality.rs`), because every element of the result is an element of an operand already at the region's boundary; a seventh role derives no obligation differently and would need a fifth `FusionRegionStructure` count, which moves the content identity of every region the vocabulary can already encode. All nine obligations discharge, with `ConversionBoundaryPreservation` resting on a stronger premise than the arithmetic families have — the inferencer refuses a non-`f32` operand at construction, so every admissible occurrence is homogeneous by construction. One decision is forced explicitly: `is_exact_governed_same_family_pointwise`'s coordinate-relation arm is closed over exact keys and must be extended, because its own soundness argument transfers verbatim and leaving it unextended silently defers every fused candidate containing a concatenate under a contraction-permitting contract.

**The most schedulable finding: M4 does not wait on M5.** Neither `derive_fusion_legality` nor `derive_obligations` resolves an index-access capability, consults a realization law, or reaches the request boundary. The role is landable now, independent of the lowering fork and of Q-SHAPE-006.

**The lowering.** The partitioned write survives; the piecewise read is eliminated. It is eliminated as *insufficient*, not merely expensive: the case selects a different operand **tensor** per coordinate, which `AccessData`'s single `tensor` field does not express and which ADR 0046's piecewise reservation — being over the map, with the tensor outside it — does not reserve; and the read-both-and-select spelling is refused by the bounds proof and additionally needs a predicate dtype `RQ-OP-03` owns. The surviving alternative asks the coordinate language for nothing (`t + offset` for a literal offset, carried by `IndexNode::LinearCombination`'s exact-integer constant, staying `Affine`) and asks the write-ownership contract for one new thing. The decisive argument is composition: the copy-free windowed realization the workload wants **is** a partitioned write, so the eliminated alternative would pay for a language widening and still owe the surviving one's contract.

**One corpus claim corrected.** Q-SHAPE-006 says the partitioned alternative "is available". It is not: it is refused at four sites, each read in full — `index/builder.rs:1899` `DuplicateOutputTensor`, `index/builder.rs:1308` `InvalidWriteDomain`, `index/builder/proof.rs:702` total-coverage ownership, `program/verify.rs:197` `MultipleWriters`. The accurate claim is the delivery graph's: it stays *inside the admitted access language*, which is a statement about what it does not widen. **Q-SHAPE-006 does not fire on this family**; its restated trigger is the first family whose *read* map is genuinely case-split over one tensor, with padding and cropping (O-24) the named candidate.

**Correction — 2026-08-10.** The four-site refusal inventory above is the research base at `d5960e81`, not the live tree. The research record's **Correction — 2026-08-06** discharges sites 1–2, splits site 3, and leaves site 4 for multi-stage multi-writer; partitioned write ownership (`WriteOwnershipProofView::PartitionMember`) and the governed concatenate lowering have since landed under the filed children. Prefer that dated correction over re-reading the line pins as live refusals.

**Checked rather than inherited.** The inner-axis assertion holds. The index region is axis-uniform — same `LinearCombination` coordinate, same interval bounds question, same contiguous-range partition on any axis — and the contiguous-window difference lives entirely in the storage half of the two-map boundary. It is an applicability predicate over a physical candidate, not a second semantic identity.

**One registration cost nobody had recorded.** `resolve_index_access` keys on the exact `(family, operation, signature)` triple and the family admits two through eight operands, so the lowering needs **seven** index-access capabilities, one per arity.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-07** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which maps every taxonomy family onto the eight delivery rungs and states why this partition is one track rather than several.
- The record owns the partition; [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity and this ticket moves no rung. Do not restate a rung here.
- `research/indexing` is declared because the lowering fork is an access-relation question the index model owns; `contracts/navigation` is **not** declared, because this ticket moves no matrix rung.
- **`contracts/navigation` was reconsidered and deliberately still not declared, for a reason the original line did not anticipate.** Three navigation edits fall out of the record — its own catalog row in `docs/research/README.md`, Q-SHAPE-006's live-pressure bullet, and the matrix row's missing owner links — and none of them moves a rung, so the original ground did not settle it. What settles it is that they are one step and cannot all be taken here: `docs/roadmap.md` was in the branch diff of the live claim `tkt/record-the-landed-bf16-carrier-in-the-dtype-ledger` at this base (`git diff --name-only d5960e81 tkt/record-the-landed-bf16-carrier-in-the-dtype-ledger` → `docs/dtype-support.md`, `docs/roadmap.md`, that ticket's own file), so file-level disjointness fails there. Taking the other two would leave the catalog, the question, and the matrix disagreeing with each other in different directions — half a navigation step, which is worse than none. The whole step is carried by [`carry-the-concatenate-scoping-conclusions-into-the-navigation-corpus`](carry-the-concatenate-scoping-conclusions-into-the-navigation-corpus.md), and the Q-SHAPE-006 replacement is drafted verbatim-landable inside the record for byte-identical transfer.
- **Filed rather than absorbed:** [`admit-a-fusion-role-for-the-sequence-extension-concatenate`](admit-a-fusion-role-for-the-sequence-extension-concatenate.md) (M4, no dependencies — it does not wait on the lowering); [`admit-a-partitioned-write-ownership-contract`](admit-a-partitioned-write-ownership-contract.md) (M5's `tiler-ir` half, whose `WriteOwnershipProofView` variant is a public boundary and stays Tom's); [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md) (M5's compiler half, depending on the contract); and the navigation carrier above. None is a deferral, so none carries a trigger log.

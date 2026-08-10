---
id: admit-a-fusion-role-for-the-sequence-extension-concatenate
title: Admit a fusion role for the sequence-extension concatenate
status: done
priority: p1
dependencies: []
related: [scope-the-concatenate-fusion-role-and-lowering, admit-a-fusion-role-for-the-tensor-contraction, reach-a-verified-kernel-through-the-structural-families]
scopes: [implementation/compiler, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, fusion, concatenate]
---
## User-visible outcome

A cover region holding `tiler::concatenate-f32@1` beside another operation derives its fusion legality from a declared role instead of failing closed to `Unknown`, so the support matrix's R5 criterion is met for the sequence-extension family rather than skipped.

## Why this exists

**Fact — before this ticket, the family resolved to no legality at all.** `FusionNumericalCapabilities::governed` in `crates/tiler-compiler/src/fusion_legality.rs` registered nine keys and the concatenate was not among them; `derive_member` returns `Ok(None)` for an unregistered family and `derive_fusion_legality` converts that into `FusionLegality::Unknown` with obligation `OperationCapabilitiesResolved` and reason `"unsupported-operation-capability"`. (Line-number citations that once sat here are retired; search those anchors and the reason string.)

**Fact — the elimination is done and one candidate survived.** [Concatenate fusion role and lowering](../docs/research/indexing/concatenate-fusion-role-and-lowering.md) tests four candidates — no role, `ValueSource`, a new seventh role, and `CoordinateRelation` — against what `derive_obligations` actually decides, and only `CoordinateRelation` survives. `ValueSource` fails on the role doc's own distinction (`a value source contributes a value the region did not otherwise have, while a coordinate relation contributes an access map`); a seventh role fails because it derives no obligation differently and a fifth `FusionRegionStructure` count would move the content identity of every region the vocabulary can already encode.

**Fact — M4 does not wait on M5.** Neither `derive_fusion_legality` nor `derive_obligations` resolves an index-access capability, consults a realization law, or reaches the request boundary. This ticket is independent of the concatenate lowering chain and of Q-SHAPE-006.

## What the work is

Register `concatenate_f32_op()` under `FusionOperationRole::CoordinateRelation` in `FusionNumericalCapabilities::governed`, with a comment stating the derivation rather than citing the record.

Extend the `CoordinateRelation` arm of `is_exact_governed_same_family_pointwise` (`fusion_legality.rs:1187-1189`) to the concatenate key. This arm is deliberately closed over exact keys so that each addition is decided rather than inherited, and the decision here is that the arm's own soundness argument — "inserting a pure data movement between two adds cannot introduce a product to fuse" — transfers verbatim to a join that introduces no multiply, no add, and no adjacency between them. Not extending it is not free: under a contraction-permitting contract a member falling through returns `Unknown` with reason `"unrealized-contraction"` (`:1113-1116`) and `first_unknown` makes the whole candidate unknown.

Prove each new path can fail. A deliberate perturbation must show a concatenate-bearing region reaching `Unknown` when the role is removed — `governed_without` (`:357-362`) exists for exactly this — and a second showing the contraction obligation's outcome under a contraction-permitting contract with and without the arm extension.

Confirm on the merged tree whether the pinned explain digest in `explain::tests::deterministic_trace_is_sealed_and_rendered_separately` (`crates/tiler-compiler/src/explain.rs` — cited by test name because its line number has already drifted once, when the softmax fact correction rebaselined it to `a95ad77532352d7f`) moves. The record's reading is that it does not — `ExplainWriter::new` folds only `FusionNumericalCapabilities::governed().provider()` (`explain.rs:1219-1235`), not the role table, and `GOVERNED_PROVIDER_REVISION` did not move when the reindex and broadcast roles were added — but that is an inference from a precedent and must be observed rather than inherited, because the ledger comments at `explain.rs:4008-4021` record two occasions on which a concatenate-related change moved it for a different reason.

## Explicit non-goals

- Any index-access lowering. That is [`admit-a-partitioned-write-ownership-contract`](admit-a-partitioned-write-ownership-contract.md) and [`lower-the-concatenate-occurrence-through-partitioned-writes`](lower-the-concatenate-occurrence-through-partitioned-writes.md), and this ticket must not wait on either.
- An `OperationNumericalCapability` row. The family performs no arithmetic, so there is no dimension a row could list; `UNPLANNED_OPERATIONS` (`crates/tiler-compiler/src/policy.rs:788-817`) records that reasoning and the entry stays until a physical realization exists.
- A seventh `FusionOperationRole` variant or a fifth `FusionRegionStructure` count.

## Closes when

The role is registered, the contraction arm is decided explicitly, a concatenate-bearing region derives `Legal` with the nine obligations discharged, both deliberate failure perturbations are shown to fail, and the matrix's `Sequence extension` row records R5 with its evidence.

## Graph maintenance

- `contracts/navigation` is declared because delivering R5 moves the [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix)'s `Sequence extension` rung and its next-column text, exactly as [`admit-a-fusion-role-for-the-tensor-contraction`](admit-a-fusion-role-for-the-tensor-contraction.md) declares it for the same reason.
- The scoping record owns the derivation and this ticket owns the rung. Do not restate the elimination here.

## Outcome

**Fact — the family no longer fails closed.** `FusionNumericalCapabilities::governed` registers `tiler::concatenate-f32@1` under the existing `FusionOperationRole::CoordinateRelation`, so a cover region holding a concatenate beside a multiply derives `FusionLegality::Legal` with all nine obligations discharged where it previously returned `Unknown` with obligation `OperationCapabilitiesResolved` and reason `"unsupported-operation-capability"`. The registration comment states the derivation in place — why a role at all, why not `ValueSource`, why not a seventh role, and why the two-through-eight arity is not an obstacle — rather than citing the scoping record.

**Fact — the contraction arm was decided, not inherited.** `is_exact_governed_same_family_pointwise`'s `CoordinateRelation` arm names the concatenate key by exact key (decision not role-inherited), on the transfer this ticket states: a join introduces no multiply, no add, and no adjacency between them. At landing the arm named concatenate beside `reindex` and `broadcast`; a later ticket also named `slice` on the same arm — census that arm in source rather than treating this paragraph as a live key list. No seventh role, no fifth `FusionRegionStructure` count, no `OperationNumericalCapability` row, and no index-access lowering were added by this ticket.

**Measurement — both perturbations were watched failing, on this branch's tree with one edit reverted at a time.** Withdrawing the registration (`roles.insert` removed) failed all three new tests, the region one reporting `Unknown(FusionUnknown { obligation: OperationCapabilitiesResolved, reason: "unsupported-operation-capability", ... })` where `Legal` is asserted. Unextending the arm (concatenate dropped from the match guard) failed exactly one test, at the contraction assertion: `left: Unknown { reason: "unrealized-contraction" }`, `right: Discharged`. Under the strict governed contract that perturbation is invisible — contraction is `Forbidden`, so the obligation discharges as a `NormativeGuarantee` regardless — which is why the perturbation is stated against `StrictF32NumericalContract::governed_relaxed()` and the test asserts that contract does not forbid contraction before relying on it.

**Measurement — the pinned explain digest did not move at landing, observed rather than inherited.** `cargo nextest run -p tiler-compiler -E 'test(deterministic_trace_is_sealed_and_rendered_separately)'` passed unchanged on the landing tree and the sealed-trace golden then held the pin value recorded in that observation. No identity-domain step was taken and no ledger comment was added, because none was owed. **Inference:** this matches the record's reading — `ExplainWriter::new` folds the fusion provider identity, not the role table, and `GOVERNED_PROVIDER_REVISION` is unmoved by role-table edits alone.

**Correction — 2026-08-10, on the explain pin absolute string.** The landing Measurement above must not be read as a live pin. The absolute request digest moves for unrelated reasons; the scoping record already says to read the current pin from `crates/tiler-compiler/src/explain.rs` in `deterministic_trace_is_sealed_and_rendered_separately` rather than from this ticket. At the 2026-08-10 audit base the golden pins `tiler-explain-v7 request=7ba3d77a66f04638`, not the landing-era `a95ad77532352d7f`. The mechanism Inference (provider identity folded, not the role table; `GOVERNED_PROVIDER_REVISION` unmoved by this role registration) still holds.

**Fact — one comment corrected in place.** `policy.rs`'s `UNPLANNED_OPERATIONS` doc said the concatenate has "no lowering, fusion role, or kernel construct". At this ticket's landing the first and third clauses still stood; the second was made false by this change, so the sentence then named the lowering and kernel construct as what was absent and recorded why holding a fusion role is not in tension with carrying no capability row. Later lowering work has since updated that same comment further — read `UNPLANNED_OPERATIONS` in `policy.rs` for the live wording. `FusionNumericalCapabilities::governed`'s own doc comment was also stale — it enumerated three roles the table outgrew several landings ago — and was restated as the invariant instead of a list.

**Fact — the matrix rung moved.** `Sequence extension` reads `R5 for the F32 family`, with the R5 evidence Fact and the trigger column repointed from the fusion role to R6's two lowering owners.

**Out of scope, flagged rather than absorbed (at landing):** the [operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md)'s O-07 M4 cell then still read **owed, and newly owned**. That file maps to `research/semantic-graph`, which this ticket does not declare, and the matrix is the sole maturity ledger, so the cell was a dated research snapshot rather than a maturity claim.

**Correction — 2026-08-10, on the delivery-graph O-07 M4 cell.** The live "still owed" claim above is no longer true of the graph file. The O-07 M4 cell now reads `delivered (`CoordinateRelation`), 2026-08-06`. Broader owner-table / M5 staleness elsewhere in that research snapshot is not this ticket's remainder and is not a maturity claim — the support matrix remains the sole ledger.

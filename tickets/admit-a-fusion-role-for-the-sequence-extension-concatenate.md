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

**Fact — the family resolves to no legality at all today.** `FusionNumericalCapabilities::governed` (`crates/tiler-compiler/src/fusion_legality.rs:268-335`) registers nine keys and the concatenate is not among them; `derive_member` returns `Ok(None)` for an unregistered family (`:1037-1039`) and `derive_fusion_legality` converts that into `FusionLegality::Unknown` with obligation `OperationCapabilitiesResolved` and reason `"unsupported-operation-capability"` (`:940-953`).

**Fact — the elimination is done and one candidate survived.** [Concatenate fusion role and lowering](../docs/research/indexing/concatenate-fusion-role-and-lowering.md) tests four candidates — no role, `ValueSource`, a new seventh role, and `CoordinateRelation` — against what `derive_obligations` actually decides, and only `CoordinateRelation` survives. `ValueSource` fails on the role doc's own distinction at `fusion_legality.rs:205-212`; a seventh role fails because it derives no obligation differently and a fifth `FusionRegionStructure` count would move the content identity of every region the vocabulary can already encode (`:511-538`).

**Fact — M4 does not wait on M5.** Neither `derive_fusion_legality` (`:922-967`) nor `derive_obligations` (`:1063-1163`) resolves an index-access capability, consults a realization law, or reaches the request boundary. This ticket is independent of the concatenate lowering chain and of Q-SHAPE-006.

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

**Fact — the contraction arm was decided, not inherited.** `is_exact_governed_same_family_pointwise`'s `CoordinateRelation` arm now names the concatenate key beside `reindex` and `broadcast`, on the transfer this ticket states: a join introduces no multiply, no add, and no adjacency between them. No seventh role, no fifth `FusionRegionStructure` count, no `OperationNumericalCapability` row, and no index-access lowering were added.

**Measurement — both perturbations were watched failing, on this branch's tree with one edit reverted at a time.** Withdrawing the registration (`roles.insert` removed) failed all three new tests, the region one reporting `Unknown(FusionUnknown { obligation: OperationCapabilitiesResolved, reason: "unsupported-operation-capability", ... })` where `Legal` is asserted. Unextending the arm (concatenate dropped from the match guard) failed exactly one test, at the contraction assertion: `left: Unknown { reason: "unrealized-contraction" }`, `right: Discharged`. Under the strict governed contract that perturbation is invisible — contraction is `Forbidden`, so the obligation discharges as a `NormativeGuarantee` regardless — which is why the perturbation is stated against `StrictF32NumericalContract::governed_relaxed()` and the test asserts that contract does not forbid contraction before relying on it.

**Measurement — the pinned explain digest did not move, observed rather than inherited.** `cargo nextest run -p tiler-compiler -E 'test(deterministic_trace_is_sealed_and_rendered_separately)'` passes unchanged and `explain.rs` still pins `"tiler-explain-v7 request=a95ad77532352d7f"`. No identity-domain step was taken and no ledger comment was added, because none was owed. **Inference:** this matches the record's reading — `ExplainWriter::new` folds the fusion provider identity, not the role table, and `GOVERNED_PROVIDER_REVISION` is unmoved.

**Fact — one comment corrected in place.** `policy.rs`'s `UNPLANNED_OPERATIONS` doc said the concatenate has "no lowering, fusion role, or kernel construct". The first and third clauses stand; the second was made false by this change, so the sentence now names the lowering and kernel construct as what is absent and records why holding a fusion role is not in tension with carrying no capability row. `FusionNumericalCapabilities::governed`'s own doc comment was also stale — it enumerated three roles the table outgrew several landings ago — and now states the invariant instead of a list.

**Fact — the matrix rung moved.** `Sequence extension` reads `R5 for the F32 family`, with the R5 evidence Fact and the trigger column repointed from the fusion role to R6's two lowering owners.

**Out of scope, flagged rather than absorbed:** the [operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md)'s O-07 row still reads **owed, and newly owned** in its M4 cell (line 96). That file maps to `research/semantic-graph`, which this ticket does not declare, and the matrix is the sole maturity ledger, so the cell is a dated research snapshot rather than a maturity claim — but it is now behind the board and a reader could take it for current.

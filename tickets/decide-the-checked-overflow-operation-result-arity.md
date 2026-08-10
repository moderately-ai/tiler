---
id: decide-the-checked-overflow-operation-result-arity
title: Decide the checked-overflow integer operation's result arity
status: deferred
priority: p2
dependencies: []
related: [derive-the-operation-family-and-signature-delivery-graph, define-the-integer-numerical-contract-and-honourability-subject]
scopes: [research/semantic-graph]
shared_scopes: [project/tickets]
paths: []
tags: [research, operations, integers, deferred, arity]
---
## User-visible outcome

When a named integer workload selects the checked-overflow form, the integer honourability track has a worked two-operand consumer program under the multi-result shape already fixed by [ADR 0039](../docs/decisions/0039-explicit-integer-overflow-operations.md) (wrapped low bits plus an overflow predicate as explicit results), and research records that still frame checked arity as open (`RQ-OP-01`, join tables) are reconciled to that decision rather than reopening it. Required-no-overflow remains a separate concept: one value plus an ADR 0021 proof-or-runtime-validation obligation, not a second candidate shape for the checked family.

## Why this is deferred rather than open

**Fact — the question was stated as open in research records.** [the mature operation and signature taxonomy](../docs/research/semantic-graph/mature-operation-and-signature-taxonomy.md)'s `RQ-OP-01` asks whether a checked-overflow integer operation "return[s] one result plus a validated precondition, or two results with an explicit overflow predicate", names F-08 as the family it blocks, and states a worked-program / ADR 0021 closure test. That row still reads as an open binary choice at filing and at the 2026-08-10 audit.

**Correction — 2026-08-10.** Accepted [ADR 0039](../docs/decisions/0039-explicit-integer-overflow-operations.md) already selects the multi-result answer for *checked*: checked add/sub/mul produce "the same wrapped low `N` bits as the wrapping family plus an overflow predicate as explicit results," and Consequences state that checked arithmetic "naturally exercises first-class multi-result operations." The one-result plus validated-precondition shape in `RQ-OP-01` is *required-no-overflow* in the same ADR — "a semantic precondition and proof or runtime-validation obligation under ADRs 0021 and 0033" — not a free alternative arity for the checked family. Taxonomy `RQ-OP-01` and this ticket's original "choose a shape" framing therefore mislabel two distinct ADR 0039 concepts as competing shapes of one family. Reopening checked arity as free choice would conflict with ADR 0039 and with [correctness-and-testing](../docs/correctness-and-testing.md)'s intended conformance shape ("checked integer arithmetic returning wrapped low bits plus the correct overflow predicate"). Superseding that multi-result decision is Tom-owned, not this node's unilateral call. Reproduce: ADR 0039 Decision bullets for checked vs Required-no-overflow; anchors `plus an overflow predicate as explicit results` and `Does a checked-overflow integer operation return one result`.

**Fact — the nearest honourability ticket excludes the arity question by name.** [`define-the-integer-numerical-contract-and-honourability-subject`](define-the-integer-numerical-contract-and-honourability-subject.md) lists under its explicit non-goals "`RQ-OP-01`'s arity question for a checked-overflow operation, which the operation axis owns and which this work must consume rather than re-decide." Nothing on the operation axis owned that question until this ticket was filed as O-10's carrier; the residual after the ADR 0039 correction is reconciliation and consumer-program evidence under the decided multi-result shape, not a fresh binary architecture choice.

**Inference — residual work still waits on a named checked workload.** A worked multi-result consumer program needs a consumer. Absent a named integer workload that selects the checked form there is no activation signal, and repairing taxonomy/join-table language is documentation residual (see residuals in the 2026-08-10 audit repair note), not a reason to open implementation.

## Activation trigger

[`define-the-integer-numerical-contract-and-honourability-subject`](define-the-integer-numerical-contract-and-honourability-subject.md)'s own trigger fires — a named tensor workload selects an exact width, an operation family, an overflow behaviour, a storage, a target, and a corpus — **and** that workload's family list includes the checked form. A wrapping, saturating, or widening workload does not fire this: those three are single-result families under ADR 0039 and do not need multi-result consumer evidence.

## What the work would be, when it starts

Write the worked two-operand program under the ADR 0039 multi-result checked shape (wrapped low bits plus overflow predicate), stating what the consumer must do with the overflow information and what the second result costs in allocation and in fusion legality when nothing reads it. Separately, if the honourability subject includes required-no-overflow, state how that form discharges under ADR 0021 (proved at construction or carried as a runtime validation obligation) — without treating it as a competing *checked* arity. Reconcile taxonomy `RQ-OP-01` / F-08 D2 and any dtype-track join row that still frames checked arity as open so they cite ADR 0039's multi-result decision. Hand the settled answer and the worked program to the integer track rather than implementing arithmetic registration here. Do not re-decide checked arity unless Tom supersedes ADR 0039.

## Explicit non-goals

- The honourability subject, which the integer track owns.
- Any other overflow family, and checked's result count. Wrapping, saturating, and widening are single-result under ADR 0039; checked is multi-result under the same ADR. None of those four result-shape decisions is reopened here.
- A public spelling, which is Tom's under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md).
- Superseding ADR 0039's multi-result checked decision, which is Tom-owned if ever proposed.

## Closes when

A worked multi-result consumer program under ADR 0039 is recorded against a named checked workload (or an explicit decision removes the checked family from the intended surface), taxonomy `RQ-OP-01` and dependent join-table language are reconciled to the multi-result decision rather than left open, and the integer track's ticket cites that answer rather than re-deriving it. Closing solely on "already decided by ADR 0039" without the taxonomy/join-table reconciliation leaves the false open-question framing live in research records.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) as track **O-10** of [Operation-family delivery graph](../docs/research/semantic-graph/operation-family-delivery-graph.md), which maps every taxonomy family onto the eight delivery rungs and states why this partition is one track rather than several.
- The record owns the partition; [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) owns delivered maturity and this ticket moves no rung. Do not restate a rung here.
- **Correction — 2026-08-10.** The honourability ticket is an *activation* peer (parent trigger plus checked form), not a completion dependency. Encoding it as `dependencies:` would block this node until the parent is *done*, while the parent non-goal requires it to *consume* this answer when checked is in scope — inverted completion order for the checked case. Both tickets remain related activation-coupled peers under `related:`; `dependencies:` is empty while deferred.

## Fact audit — 2026-08-10

Phase B repair against audit report `docs/research/documentation/ticket-audit-2026-08-10/reports/decide-the-checked-overflow-operation-result-arity/b7ec363b8f37_c99ac54950f2.md`.

- [VERIFIED] Taxonomy `RQ-OP-01` still states the binary question and F-08 / ADR 0021 closure test (research table; not registration).
- [VERIFIED] Parent non-goal excludes `RQ-OP-01` by name; parent `status: deferred`.
- [FALSE → repaired] Framing that checked arity is still an open binary choice for this ticket to *choose*. ADR 0039 already binds checked to multi-result; required-no-overflow is the separate one-result + ADR 0021 form.
- [VERIFIED] Wrapping, saturating, and widening are one-result under ADR 0039; activation still requires the checked form in the workload family list.
- [VERIFIED] No checked-overflow (or other general integer data-arithmetic) family is registered; `tiler::u32@1` remains gather's index-operand identity only (`Returns the one admitted index-operand identity` in `crates/tiler-ir/src/semantic/gather.rs`).
- [IMPRECISE → repaired] Parent listed under `dependencies:` as if completion-ordered; moved to `related:` as activation coupling.
- Residual product debt (not this ticket-only wave): reconcile taxonomy `RQ-OP-01` / F-08 D2 and dtype-track join rows to ADR 0039; optional worked multi-result program when a checked workload appears.

## Trigger check log

- 2026-08-05 — **not fired.** The integer track's own log records its trigger unmet on 2026-08-04 and nothing has changed it: no registered operation admits a general integer operand, and the only integer keys in the semantic layer are dtype identities. Recheck: `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 46 governed keys, of which the operation keys are the F32, BF16, activation, structural, contraction, and strict-affine quantization sets and no integer arithmetic key at all.
- 2026-08-09 — **not fired.** The parent integer-honourability ticket remains `deferred`, and the semantic operation registry still contains no checked-overflow integer family. `tiler::u32@1` is now a real gather index identity, but the gather contract names it as an index operand rather than an arithmetic operation, so it does not select the checked form or its result arity. Recheck the parent status and the anchors `Returns the one admitted index-operand identity` in `crates/tiler-ir/src/semantic/gather.rs` and `register_operation` in `crates/tiler-ir/src/semantic/registry.rs`.
- 2026-08-10 — **not fired.** Parent remains `deferred`; no checked-overflow integer family is registered; gather still only admits `tiler::u32@1` as an index operand (`Returns the one admitted index-operand identity`). ADR 0039 vs `RQ-OP-01` reconciliation is a prose defect independent of activation: checked multi-result is already decided, so the open binary-choice framing is false even while the trigger stays unmet. Recheck: parent `^status: deferred$`; anchors `plus an overflow predicate as explicit results` in ADR 0039 and `Returns the one admitted index-operand identity` in `gather.rs`.

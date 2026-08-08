---
id: accept-adr-0108-data-dependent-index-coordinate-siting
title: Accept or revise ADR 0108 on siting a data-dependent index coordinate
status: in-progress
priority: p1
dependencies: []
related: [admit-the-indirect-access-class-into-the-index-layer, admit-an-indirect-gather-family-for-tied-embedding-lookup, emit-the-indirect-gather-on-metal, accept-adr-0107-indirect-gather-semantic-family]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [decisions, indexing, ir, gather, verification, needs-tom]
assignee: sol-adr0108
lease_expires_at: 1786219272
---
**This ticket is Tom's decision, not an agent's work item.** It exists so a follow-on that admits the form has something to depend on rather than being schedulable while the record shaping it is still `proposed`.

`docs/decisions/0108-site-a-data-dependent-index-coordinate-on-the-expression.md` is `decision_status: proposed`, `implementation_status: none`. It extends [ADR 0107](../docs/decisions/0107-admit-an-indirect-gather-as-a-semantic-family-above-the-index-language.md), which admitted the gather family and named the index-layer question as a separate decision it deliberately did not answer.

## What the record decides

**One: the shape.** A data-dependent coordinate, if ever admitted, is an index-expression form and a fourth `IndexExprClass` member — never a second tensor ordinal or an indirection record on `AccessData`. Three reasons, each read from the implementation rather than argued from the framing:

- the canonical encoder dispatches on an explicit per-form tag (`encode_index_node`, `structural_index_key`), so a sixth form changes the bytes of no region that lacks one, while any field on the access moves every region identity ever derived and forces `tiler.index-region.v11` to `v12`;
- `IndexDomainPredicate` names a `VerifiedIndexExprId` in both variants and `validate_index_domain_predicate` requires that expression to be one of the access's coordinates, so an indirect *axis* that is not an expression has no handle to constrain and needs a second predicate subject; and
- the per-axis coordinate-to-extent correspondence is spelled as `zip` in seven functions of `proof.rs`, which an access-level indirection falsifies at all seven simultaneously and silently, because `zip` truncates rather than failing.

**Two: what ADR 0046's non-weakening condition costs.** The form is *sound* — an expression reading tensor data has no propagated interval, so it can neither prove a bound nor refute one, and it declines in every discharge mechanism without changing any direct-access coordinate's answer. What it weakens is the meaning of a retained unknown: all three `IndexDomainUnknownReason` members mean "dischargeable in principle by supplying more", and a data-dependent bound is closable by none of them in any environment. The form therefore requires a **fourth reason naming undecidability in principle**, and form and reason are one change rather than two.

**Three: not yet.** Nothing consumes an index region containing an indirect coordinate — no realization law, no lowering capability, `classify` returns `None`, and `LogicalAccess` has no relation for it. Admitting the form today would replace an early typed refusal at the request boundary with a region that *builds* carrying an obligation nothing can discharge.

## The trade-off, stated as it was decided

Accepting this record accepts that a **verifiable-but-undischargeable region is not the same kind of legitimate delivered state as a registered-but-unplannable family**. ADR 0107 accepted the latter; this record declines the former, on the ground that a family is a statement of meaning while a region is a carrier of proof.

**The counterpoint.** A shape decided and not taken can rot: the three costs above are measured against today's encoder, predicate vocabulary, and verifier, and a later refactor could move any of them without anyone re-reading this record. The mitigation is that the vocabulary counts are now pinned from their types in `crates/tiler-ir/src/index/builder/tests.rs`, so widening either enum is a build error naming the record — but nothing pins the *reasons*, only the outcome.

## What acceptance does not commit to

Acceptance is not a public-boundary acceptance. Under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) the four widenings the record shapes — an `IndexNode` form, an `IndexExprView` variant, an `IndexExprClass` member, and an `IndexDomainUnknownReason` member — are a **decided shape and an undrafted surface**. None is written, so none is yet a labelled draft.

## What closes this ticket

Either set `decision_status: accepted` with acceptance provenance, or record the requested revisions here and send the record back. Rejecting the *shape* half means the access-record route stays open and its three costs must be answered by whoever takes it. Rejecting the *timing* half means filing a ticket that admits the expression form together with the fourth unknown reason — never the form alone.

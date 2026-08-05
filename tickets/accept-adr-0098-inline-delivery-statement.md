---
id: accept-adr-0098-inline-delivery-statement
title: Accept or reject the inline delivery-statement ADR
status: done
priority: p2
dependencies: [draft-an-adr-for-the-inline-delivery-statement]
related: [accept-the-inline-artifact-family-profile-syntax, first-authoritative-ios-metal-compile-declaration]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, frontend, inline-dx, apple-targets, decision, needs-tom]
---
## User-visible outcome

[ADR 0098](../docs/decisions/0098-state-an-inline-regions-delivery-policy-with-a-named-profile-or-a-family-list.md) moves from `proposed` to `accepted`, or is rejected.

**Only Tom closes this ticket.** No agent may set it `done`, and no agent may do its work. Its permanent status is `awaiting-decision` — a parked state `tkt ready` excludes and that never satisfies a dependent; an agent that finds it in `todo` should set it back and do nothing else.

**This node carries the scopes its own acceptance sweep needs.** The sweep touches `docs/decisions/[0-9]*.md` (`contracts/decisions`) and both catalog views in `docs/decisions/README.md` (`contracts/navigation`), which is exactly what is declared above. It deliberately declares no research scope — see "What acceptance does and does not do" for why no research record's frontmatter moves.

## What is actually being decided, stated so Tom can act without re-deriving it

**The spelling is not in question and must not be reopened here.** Tom accepted it on 2026-07-31 under [`accept-the-inline-artifact-family-profile-syntax`](accept-the-inline-artifact-family-profile-syntax.md), and it is implemented, tested, and stated in [the frontend contract](../docs/integration/frontends.md). ADR 0098 changes nothing a consumer writes. **What is being decided is whether that already-taken decision is worth a catalogued record** — whether the `deliver` statement is consequential enough to outlive its ticket, the way the neighbouring consumer-visible spelling from the other half of the same contract was when it became [ADR 0089](../docs/decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md) on the same day.

So the question is narrow, and rejecting it costs nothing implemented: it would mean the delivery spelling stays discoverable only through a terminal ticket and the frontend contract's prose, which is the state the drafting ticket argued against.

**One thing in the record is more than a transcription, and it is flagged rather than buried.** ADR 0098's measurement boundary *corrects* the deciding ticket instead of repeating it. That ticket recorded a stated selected family as refused outright; `prototype-inline-aot-integration-proof` landed later the same day and made `deliver macos;` deliver end to end. The record states what is true at the commit that created it — two spellings have ever completed an expansion, two of the four profile names have never delivered anything for want of an iOS compile declaration, and the family-list production has never produced a payload — each with a one-line reproducible check. If Tom disagrees that a record may correct its own source ticket that way, that is the item to say so on.

## Before deciding, read this provenance note

**The 2026-07-31 acceptance is a relay as far as this record is concerned.** Nobody who wrote ADR 0098 witnessed Tom accept the spelling; the drafting worker took it from the deciding ticket's Decision section, which is the durable record of it. That is sound for a *transcription* — the record cites the ticket for every ground rather than asserting them — but it means an error in that section would propagate into the catalog with an ADR's authority behind it. Nothing has been released on this record and no contract has been rewritten under it, which is what keeps the rollback below cheap.

## What acceptance does and does not do

Acceptance flips `decision_status` to `accepted` on ADR 0098 and updates both catalog views in [the decisions index](../docs/decisions/README.md) — the theme row under "Artifacts, build, and toolchains" and the chronology row — from `proposed` to `accepted`.

**It moves no research record's frontmatter, and that is a derivation rather than an omission.** ADR 0098 cites [the proc-macro build environment research](../docs/research/macro-environment/proc-macro-build-environment.md) and [the Apple Metal artifact compatibility research](../docs/research/apple-targets/artifact-compatibility.md) as `evidence`, for the absent-target-variable measurements and the device-versus-simulator distinctness respectively. Neither record *proposed* the `deliver` spelling, and `docs/document-metadata.md` states that "`evidence`, `informs`, and `adopted_by` are independent predicates: evidence may support a decision without that decision adopting the report's proposal." So `adopted_by` does not gain `ADR-0098` on either — both already name ADR-0049 and ADR-0053, which did adopt them — and neither `disposition` moves.

**It moves no contract frontmatter either.** A contract's inbound-ADR link is the derived `governed_by`, which `docs/document-metadata.md` declares "invalid in stored v1 frontmatter"; the edge is stored on ADR 0098's own `applies_to` and nowhere else. The frontend contract is already `contract_status: accepted` and `implementation_status: partial`, and this record changes neither.

**It implements nothing and changes no consumer-visible surface.** It admits no new profile name, no new production, no vocabulary widening, and no change to what any existing region expands to. It moves no version string, no encoding, and no pinned identity.

**It releases nothing, and that is checked against the board rather than asserted.** No ticket declares a dependency on [`draft-an-adr-for-the-inline-delivery-statement`](draft-an-adr-for-the-inline-delivery-statement.md) — reproduce with `grep -rn "^dependencies:.*draft-an-adr-for-the-inline-delivery-statement" tickets/`, which reports no match. This node exists so that a *future* ticket conditional on the delivery spelling being a decided record depends on something that can distinguish "written" from "decided", which the drafting ticket cannot: drafting a proposed ADR is a completed outcome, so that ticket goes `done` the moment the file exists.

> **The reproducing command above does not report what the sentence says, corrected 2026-08-04 by the stale-claim sweep. The claim is right and the check was wrong from the day it was written.** That `grep` returns exactly one match — `tickets/accept-adr-0098-inline-delivery-statement.md:6`, this node's own `dependencies:` line, which has named the drafting ticket since the node was filed. So a reader running it sees a hit and cannot tell a genuine dependent from the self-reference; the check cannot say no, which is the failure mode AGENTS.md names. **What the sentence means is that no *other* ticket depends on it**, and that is true: `grep -rn "^dependencies:.*draft-an-adr-for-the-inline-delivery-statement" tickets/ | grep -v accept-adr-0098-inline-delivery-statement` reports no match, over a `tickets/` population of 821 files. Use the second form. **Nothing about the decision moves**: acceptance still releases nothing, the status stays `awaiting-decision`, and only Tom closes this node.

## Rollback, kept cheap on purpose

If the relayed acceptance behind ADR 0098 turns out to be wrong, or the record is rejected after being accepted, the repair is one field and two catalog rows: `decision_status` back to `proposed`, the theme and chronology rows back to `proposed`, and this node back to `awaiting-decision`. Nothing else moves, because acceptance released nothing and rewrote no contract.

Rejecting the record outright is a deletion of one file plus its two catalog rows and this node, and it leaves the accepted spelling exactly where it already is — in the deciding ticket and in the frontend contract.

## Closes when

Tom accepts or rejects it.

## Decided — accepted

Accepted by Tom on 2026-08-05 at the live decision review in the coordination session, witnessed first-hand by the coordinator. The acceptance sweep executed in the same change: `decision_status` flipped, both catalog views updated, and the frontends contract's proposed-status disclosure corrected to cite the record's authority.

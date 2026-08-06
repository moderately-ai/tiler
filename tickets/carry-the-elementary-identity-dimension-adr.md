---
id: carry-the-elementary-identity-dimension-adr
title: Carry the elementary-identity dimension ADR
status: review
priority: p2
dependencies: []
related: [name-the-elementary-identity-rewrite-dimension, connect-certified-rounding-error-bounds-to-rewrite-permissions, decide-whether-to-admit-an-elementary-identity-permission]
scopes: [contracts/decisions, contracts/navigation, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, decision, carrier]
claimed_from: todo
assignee: agent-identity-carrier
lease_expires_at: 1785978598
---
## User-visible outcome

The drafted ADR body inside [the elementary-identity rewrite dimension record](../docs/research/numerics/elementary-identity-rewrite-dimension.md) exists as a `proposed` ADR under `docs/decisions/`, catalogued, so that the dimension's definition has a decision record behind it rather than living only in a research record's Proposal labels.

## Why this exists

**Fact.** [ADR 0080](../docs/decisions/0080-treat-distributivity-as-a-third-numerical-dimension.md)'s own Context states why the gap matters rather than being a formality: under [documentation metadata](../docs/document-metadata.md)'s `mixed`-contract rule, "only accepted-ADR-derived invariants and sections explicitly labeled accepted are normative", and `docs/numerical-semantics.md` is `contract_status: mixed`. A dimension defined only in a research record is a well-argued paragraph that the metadata contract classifies as proposed.

**Fact.** The producing ticket held `research/numerics` and `contracts/navigation` and never `contracts/decisions`, so it structurally could not write the record — the same split [`settle-contraction-chain-distributivity-permission`](settle-contraction-chain-distributivity-permission.md) hit before [`record-distributivity-dimension-adr`](record-distributivity-dimension-adr.md) carried ADR 0080.

## What this ticket does

- **Transfer the drafted span byte-identically.** The span is between the two horizontal rules inside the record's "Drafted ADR body" section, beginning at `**Title:**` and ending at the last alternatives-considered paragraph. **Re-derive the line numbers** with `grep -n '^---$'` rather than trusting any stated in the record; every edit above the span moves them. Map `#### ` to `## ` and change nothing else. Diff the two ranges after that normalization to check the transfer, and **perturb one word and watch the check fail before believing it**.
- **Re-read the ADR number.** `0100` was the highest when the record was written and `0101` is drafted. Take the next free number; nothing in the span depends on it, because the span's H1 comes from the `**Title:**` line rather than from the body.
- **Write the traceability, normative-owner, and work-record sections fresh at the destination.** The span deliberately carries no relative links at all, which is what makes the transfer unconditional; re-run the record's own stated check (`grep -c ']('` over the span, against a nonzero count elsewhere in the same file) to confirm before transferring.
- **Add the catalog row** under [`docs/decisions/README.md`](../docs/decisions/README.md), which `ticketsplease.toml` routes to `contracts/navigation` rather than to `contracts/decisions` — which is why this ticket holds both.
- **File an acceptance node.** The ADR lands `proposed`, so it is a non-decision until Tom accepts it. Park an `accept-adr-NNNN-elementary-identity-dimension` ticket at `awaiting-decision`, per `ticketsplease.toml`'s own note that a ticket conditional on an ADR being accepted depends on that ADR's acceptance ticket and never on the ticket that drafted the record.

## Non-goals

Accepting the ADR; editing the span's content; editing `docs/numerical-semantics.md`, which is `contracts/numerics` and moves only at acceptance; admitting any permission.

## Closes when

The ADR file exists at `docs/decisions/` with `decision_status: proposed`, the transfer is confirmed byte-identical after the stated normalization with the perturbation watched failing, the decisions catalog carries its row, and the acceptance node is parked at `awaiting-decision`.

## Outcome

**Landed as [ADR 0101](../docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md), `decision_status: proposed`, on 2026-08-05.** `0100` was still the highest at the base `de377fb1` (`ls docs/decisions/01*.md` returned it alone), so the drafted number was free and was taken unchanged.

**The transfer is byte-identical and the check was proved able to fail first.** The span was re-derived rather than trusted — `grep -n '^---$'` put the section's rules at `287` and `339`, giving `289,337` at the base — and the record's three stated link counts all reproduced before the transfer: span `0`, lines `1,100` → `12`, whole file → `40`. Perturbing decision 1 from "it is a fourth one" to "it is a fifth one" made the normalized `diff` report that line; with the perturbation removed, `cmp` reported the two byte-identical and the normalized `diff` reported no differences, while the raw `diff` reported exactly eight changed lines forming the four heading pairs — the sole `#### ` → `## ` transform and nothing else. The `**Title:**` and `**Frontmatter:**` directive lines were consumed into the frontmatter and H1 rather than transferred, which is the convention every landed ADR follows (`grep -ln '^\*\*Title:\*\*' docs/decisions/*.md` returns nothing); the `**Status:**` paragraph transfers as body.

**Both catalog views carry the row as `proposed`**, following the ADR 0099 landing precedent at `affe281f`: the theme row under "Numerical operations" in title order beside ADR 0080, and the chronology row after ADR 0100. Reconciled by counting the population rather than by eye — 101 ADR files against 101 chronology rows and 101 theme rows, zero missing in either view, with the check's negative and positive controls both fired.

**The record's carrier note was updated per its siblings' convention** and its drafted-span disclosure was kept rather than replaced: the landed-ADR pointer sits beside it, the span's line range is restated as `293,341` after the added paragraphs moved it, and the `**Frontmatter:**` line's `decision_status: proposed` is recorded as true-as-landed with a note that acceptance makes it false and that it must then be flagged rather than edited. The record's `disposition` stays `pending` and no `adopted_by` edge was added, because a `proposed` ADR adopts nothing — every record carrying `adopted_by` names an accepted ADR.

**Not done here, and it is the one closing condition this branch does not satisfy: the acceptance node.** The dispatch brief assigned it to the coordinator at integration, against this ticket's own "Closes when" above. That is not a silent descope — it is a live conflict surfaced rather than resolved by the worker, and the brief's reading is corroborated in the tree by ADR 0100's own work record, which states that filing an acceptance node "is the coordinator's step under the convention every proposed record here follows". **The remainder is one ticket**: park `accept-adr-0101-elementary-identity-dimension` at `awaiting-decision`, depending on this one, so the graph blocks structurally on Tom rather than deadlocking this carrier in `review`. Until it exists, nothing in the graph is blocked on ADR 0101's status.

**`research/numerics` was added to this ticket's scopes autonomously, and here is why it belongs.** Updating the source record's carrier note is part of the transfer rather than adjacent to it — the convention the sibling records set is that a drafted span stops claiming to be undrafted the moment it lands, and a record still saying "the drafted ADR body has no carrier" after the carrier ran is precisely the stale assertion the docs process exists to prevent. `ticketsplease.toml` routes `docs/research/**` to `research/numerics`, which the original declaration missed because the ticket was written as if the transfer were write-only into `docs/decisions/`. `tkt guard` caught it as `under_declared` rather than a reviewer catching it later, which is the mechanism working.

**No live ticket holds `research/numerics`, checked by counting rather than by sampling.** Eighty-two tickets declare it in their `scopes:` line and **zero** are `in-progress` or `review`; the scan's control fired, correctly reporting this ticket as the sole live `contracts/decisions` holder. The one live neighbour, [`correct-the-online-single-pass-softmax-fold-legality-fact`](correct-the-online-single-pass-softmax-fold-legality-fact.md), declares `implementation/ir` alone and is scope-disjoint. Its branch has **no commits**, so the file-level check against it is **vacuous and is reported as such rather than as disjointness** — the substantive protection is not file separation but citation discipline: ADR 0101 contains zero `.rs:NNN` line citations, zero `SOFTMAX_` constant references, zero registered fact-string values, and zero references to the semantic-definition projection, so that ticket's identity step cannot fork this record's text. The one `crates/` path the ADR names is `crates/tiler-ir/src/schedule/numerics.rs` for the contract-key literals, which is a different file from the semantic registry that correction moves, and it is cited without a line number.

**Non-goals held.** `docs/numerical-semantics.md` is untouched (`contracts/numerics`, and it moves only at acceptance), no permission is admitted, no span content was edited, and nothing in `crates/` changed.

---
id: carry-the-elementary-identity-dimension-adr
title: Carry the elementary-identity dimension ADR
status: todo
priority: p2
dependencies: []
related: [name-the-elementary-identity-rewrite-dimension, connect-certified-rounding-error-bounds-to-rewrite-permissions, decide-whether-to-admit-an-elementary-identity-permission]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [tiler-research, numerics, decision, carrier]
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

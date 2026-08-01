---
id: land-the-backend-scoped-route-requirement-answer-adr
title: Land the backend-scoped route-requirement answer ADR as proposed
status: in-progress
priority: p2
dependencies: [design-the-adapter-owned-route-requirement-answer-channel]
related: [dispatch-a-tiler-region-on-metal-hardware, close-the-metal-gpu-family-out-of-crate-total-map]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, runtime, backends, metal, public-boundary]
claimed_from: todo
assignee: worker-answer-adr
lease_expires_at: 1785596471
---
## User-visible outcome

The backend-scoped route-requirement answer design exists as a `proposed` ADR under `docs/decisions/`, listed in the decision catalog, and the research record behind it is listed in the research catalog — so a reader arriving at either index finds it instead of a record nothing points at. Today the design is complete and lives only in `docs/research/runtime/` because the ticket that produced it could not reach either path.

## Why this is a separate ticket and not an omission

**Fact — the scope map, checkable in one line.** `ticketsplease.toml` maps `docs/decisions/[0-9]*.md` to `contracts/decisions`, and maps `docs/decisions/README.md` **and `docs/research/README.md`** to `contracts/navigation`:

```sh
rg -n 'contracts/decisions|contracts/navigation' -A 14 ticketsplease.toml
```

**Fact.** `design-the-adapter-owned-route-requirement-answer-channel` holds `research/runtime` and `research/extensions` exclusively and `project/tickets` shared, and holds neither of the two scopes above. Writing an ADR file or editing either catalog from that branch is a guard escape. This is the same split [`land-the-bf16-conversion-and-accumulator-adr`](land-the-bf16-conversion-and-accumulator-adr.md) records for the BF16 design, and the idiom is copied deliberately.

**Note — two catalog rows, not one.** The BF16 precedent carried only the decision catalog because its research record's row already existed. This design's research record is new, so `docs/research/README.md` needs a row too, and that file is `contracts/navigation` as well.

## What already exists and must be used rather than rewritten

[Backend-scoped route-requirement answers](../docs/research/runtime/backend-scoped-route-requirement-answers.md) carries a **Drafted ADR body** section written to be landed verbatim: context, nine numbered decisions, consequences, and six alternatives-considered entries each with the elimination stated. It also carries the b1/b2 derivation, both worked examples, the public-boundary list, the measurement boundary, and the deferrals. Do not re-derive any of it.

## Implementation keys

- Create `docs/decisions/00NN-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md` with the frontmatter the record's drafted body states — `decision_status: proposed`, `implementation_status: not-started`, `catalog_group: "runtime-integration-placement"`. Take the next free number by reading the directory, not by remembering one: `0090` was the highest at `6f7caf3` and a sibling may have landed since.
- Add the row to the **proposed** section of `docs/decisions/README.md`, and to the numeric index further down, in the same change. An ADR appears in both.
- Add the research record's row to `docs/research/README.md` under `### Runtime, integration, and placement`, in title order — it sorts after "Autoregressive state and KV-cache ownership" and before "Candle Metal post-wait error checking". Copy the format from the neighbouring rows rather than from here: a list item whose link text is the title and whose target is the path relative to `docs/research/`, then an em dash, the disposition, a semicolon, the comma-joined evidence classes, a semicolon, and `informs:` followed by links to each `informs` contract and each `adopted_by` ADR. Its disposition is `pending` and its evidence class is `primary-source-synthesis`; it cites no experiment, so it carries no `experiments:` clause.
- Add the traceability block the sibling ADRs carry, pointing at [System architecture](../docs/architecture.md), [Artifact envelope and Metal kernel ABI profile](../docs/artifact-abi.md), the research record, and this ticket.
- Set `adopted_by` on the research record only if and when the ADR is *accepted*. A proposed ADR is not an adoption, and the record's `disposition` stays `pending` until Tom accepts. **Do not add a `related` link to the research record's frontmatter** — `docs/document-metadata.md` does not license `related` for `kind: research`, and the BF16 carrier ticket's instruction to add one should not be copied.

## Explicit non-goals

- **Do not accept the ADR.** Acceptance of a public boundary is Tom's, and the record enumerates seven boundary items including reclassifying `tiler-metal` as a crate a consumer may name. Landing it `proposed` is the whole of this ticket.
- **Do not amend `docs/architecture.md`.** Decision item 6 restates "a consumer names `tiler` alone" as a property of the non-dispatching consumer, and that sentence lives in `contracts/foundation`, which this ticket does not hold. It is part of the acceptance sweep, not of landing a proposal.
- **Do not implement anything.** No crate gains an item, no test changes, and `spikes/runtime/inline-dispatch` stays fail-closed.

## Closes when

The ADR file exists with `decision_status: proposed`, both decision-catalog views list it, the research catalog lists the research record, the record links the ADR in its prose, and `make full` is green.

## Graph maintenance

- Depends on the design ticket, whose Outcome states exactly what the ADR must say.
- Gates nothing that exists. Implementation is a separate phase decision under the implementation boundary and has no ticket, deliberately: research completion does not authorize scaffolding.
- If Tom accepts the ADR in the same session, the acceptance sweep — catalog views, the architecture-contract sentence, and any released work — is that acceptance's own change and not this one.

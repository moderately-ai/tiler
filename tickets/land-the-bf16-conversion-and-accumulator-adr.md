---
id: land-the-bf16-conversion-and-accumulator-adr
title: Land the BF16 conversion and accumulator ADR as proposed
status: in-progress
priority: p2
dependencies: [design-the-bf16-computation-and-accumulator-contract]
related: [spike-bf16-through-the-second-dtype-seams, admit-a-bf16-scalar-arithmetic-subject, register-the-bf16-semantic-operation-signatures]
scopes: [contracts/decisions, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, dtype, bf16, numerics, conversion]
claimed_from: todo
assignee: worker-adr-carrier
lease_expires_at: 1785594086
---
## User-visible outcome

The BF16 computation, accumulator, and conversion design exists as a `proposed` ADR under `docs/decisions/`, listed in the decision catalog, so a reader arriving at the accepted-decision index finds it instead of a research record nothing points at. Today the design is complete and lives only in `docs/research/numerics/` because the ticket that produced it could not reach either path.

## Why this is a separate ticket and not an omission

**Fact — the scope map, checkable in one line.** `ticketsplease.toml` maps `docs/decisions/[0-9]*.md` to `contracts/decisions` and `docs/decisions/README.md`, `docs/dtype-support.md`, and `docs/roadmap.md` to `contracts/navigation`:

```sh
rg -n 'contracts/decisions|contracts/navigation' -A 14 ticketsplease.toml
```

**Fact.** `design-the-bf16-computation-and-accumulator-contract` holds `research/numerics` and `contracts/numerics` exclusively and `project/tickets` shared, and holds neither of the two scopes above. Writing an ADR file or editing the catalog from that branch would have been a guard escape of exactly the class [the BF16 spike's finding 8](../spikes/numerics/bf16-second-dtype/README.md) records, and `contracts/navigation` was concurrently held by an in-progress ticket, so the edit would also have collided with live work.

**Inference.** The dispatch brief for that ticket named a proposed ADR as the expected vehicle and named the catalog update as a deliverable. Both are correct as *outcomes* and neither was reachable from the scopes the ticket declares. This ticket carries them.

## What already exists and must be used rather than rewritten

[BF16 computation, accumulator, and conversion](../docs/research/numerics/bf16-computation-accumulator-and-conversion.md) carries a **Drafted ADR body** section written to be landed verbatim: context, five numbered decisions, consequences, and four alternatives-considered entries each with the elimination stated. It also carries the derivation, the worked examples with differing bits, the measurement boundaries, and the public-boundary list. Do not re-derive any of it.

## Implementation keys

- Create `docs/decisions/00NN-separate-bf16-float-conversion-families-and-keep-the-accumulator-an-operation-fact.md` with `decision_status: proposed`, `implementation_status: not-started`, `applies_to: ["tiler.contract.numerical-semantics"]`, and `evidence: ["tiler.research.numerics.bf16-computation-accumulator-and-conversion"]`. Take the next free number by reading the directory, not by remembering one.
- Add the traceability block the sibling ADRs carry — normative owner, evidence, work record — pointing at [Numerical semantics](../docs/numerical-semantics.md), the research record, and this ticket.
- Add the row to the **proposed** section of `docs/decisions/README.md` in the same change, per the docs-maintenance rule that a catalog is edited in the change that moves the metadata behind it.
- Set `adopted_by` on the research record only if and when the ADR is *accepted*. A proposed ADR is not an adoption, and the record's `disposition` stays `pending` until Tom accepts.
- Add a `related` link from the research record's frontmatter to the new ADR id.

## Explicit non-goals

- **Do not accept the ADR.** Acceptance of a public boundary is Tom's, and the drafted decisions include two new operation keys and a new conversion-contract vocabulary. Landing it `proposed` is the whole of this ticket.
- **Do not move any `docs/dtype-support.md` cell.** No conversion key is registered, no evaluator exists, and the ledger's own closing rule is that out-of-tree evidence never promotes an unregistered family. The `Cast and convert` row of the roadmap matrix likewise stays at R2.
- **Do not restate the derivation in the ADR.** An ADR records the decision and its eliminations; the evidence lives in the research record and the spike.

## Closes when

The ADR file exists with `decision_status: proposed`, the catalog's proposed section lists it, the research record links it, no cell of the dtype ledger or the roadmap matrix moved, and `make full` is green.

## Graph maintenance

- Depends on the design ticket, whose Outcome states exactly what the ADR must say.
- Gates nothing that exists. Implementation of the conversion family is a separate phase decision under the implementation boundary and has no ticket, deliberately: research completion does not authorize scaffolding.
- If Tom accepts the ADR in the same session, the acceptance sweep — catalog views, contract sentences whose truth depended on the proposed status, and any released work — is that acceptance's own change and not this one.

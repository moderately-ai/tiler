---
id: catalogue-adr-0100-and-the-multi-round-composition-record
title: Catalogue ADR 0100 and the multi-round composition record
status: todo
priority: p2
dependencies: []
related: [derive-the-multi-round-two-level-reduction-composition]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, navigation, adr, scheduling, catalog]
---
## User-visible outcome

[ADR 0100](../docs/decisions/0100-admit-the-multi-round-two-level-reduction-composition.md) and [the multi-round two-level reduction composition](../docs/research/scheduling/multi-round-two-level-reduction-composition.md) are reachable from the two catalogs a reader navigates by, instead of only from the records that link them.

## Why this is a separate ticket rather than part of the derivation

**Fact.** `ticketsplease.toml` routes `docs/decisions/[0-9]*.md` to `contracts/decisions` and both `docs/decisions/README.md` and `docs/research/README.md` to `contracts/navigation`. [`derive-the-multi-round-two-level-reduction-composition`](derive-the-multi-round-two-level-reduction-composition.md) holds `research/scheduling` and `contracts/decisions`, so it could land both records and neither catalog row. That is the same split [`land-the-two-level-reduction-adr`](land-the-two-level-reduction-adr.md) and [`land-the-two-dimensional-staging-relation-adr`](land-the-two-dimensional-staging-relation-adr.md) carried, narrowed to the rows alone because the records themselves are already landed.

**Fact — nothing validates a catalog.** [AGENTS.md](../AGENTS.md) records that the catalog blocks under `docs/` and `docs/decisions/README.md` are hand-maintained prose with no renderer and no gate behind them, so an uncatalogued record costs a reader rather than a check.

## Work

- Add ADR 0100's row to `docs/decisions/README.md`, in the `physical-planning-lowering` group and in the status view, matching the existing row shape: title link, `proposed`, contracts, evidence. ADR 0097's and ADR 0096's rows are the two to copy from, and the record appears in **both** views — the group listing and the status listing further down — which is the half a previous carrier had to be told about.
- Add the research record's row to `docs/research/README.md` under the same group, matching the shape of [the two-dimensional staging relation](../docs/research/scheduling/two-dimensional-cooperative-staging-relation.md)'s row: title link, disposition, evidence class, `informs:` targets. Its disposition is `pending`, not `adopted`, because ADR 0100 is `proposed` — a row claiming `adopted` would be the record asserting its own acceptance.
- Verify by reading, not by adding: check first whether either row already exists, because a second row for one record is worse than none.

## Closes when

Both catalogs carry a row for their record, each row's status field matches the record's own frontmatter, and every link in the two new rows resolves.

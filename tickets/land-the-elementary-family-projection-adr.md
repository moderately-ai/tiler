---
id: land-the-elementary-family-projection-adr
title: Land the elementary-family projection ADR
status: todo
priority: p1
dependencies: [admit-the-registered-unary-families-at-the-compiler-request-boundary]
related: []
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [research, decisions, carrier]
---
## User-visible outcome

The route the compiler took to make a registered elementary family reachable is an accepted decision in `docs/decisions/`, so the next reader of `crates/tiler-compiler/src/elementary.rs` finds the derivation behind it rather than a module comment asserting one.

## Why this is a carrier ticket

**Fact — the deriving ticket's scopes cannot reach the decision record.** `ticketsplease.toml` maps `docs/decisions/[0-9]*.md` to `contracts/decisions` and `docs/decisions/README.md` to `contracts/navigation`. [`admit-the-registered-unary-families-at-the-compiler-request-boundary`](admit-the-registered-unary-families-at-the-compiler-request-boundary.md) declares `implementation/compiler` exclusively and `project/tickets` shared, so writing the ADR or editing the decision catalog from that branch would be a guard escape. Its body therefore holds the ADR **drafted verbatim-landable**, in the "Drafted ADR body, to be landed byte-identically" section.

**The transfer is byte-identical.** A transfer that edits is a fork. The drafted body carries no traceability section and no `docs/decisions/`-relative links, so it has none of the link tension a drafted body with one would have; copy the fenced block, allocate the next number, and rename the file accordingly.

## Required delivery

- The drafted body landed byte-identically at the next free `docs/decisions/NNNN-project-an-elementary-family-s-per-point-body-from-one-shared-statement.md`, with only the numeric prefix and the `id` frontmatter field adjusted to the allocated number if the catalog's convention requires it — and if either adjustment is needed, it is recorded here as a stated exception rather than made silently.
- The catalog block in `docs/decisions/README.md` updated in the same commit as the metadata behind it. That file maps to `contracts/navigation`, so this ticket declares both scopes.
- `decision_status` left at `proposed`. **Acceptance is Tom's and nothing in the deriving work relayed one.** Moving it to accepted, updating the catalog views, and correcting every contract sentence whose truth depended on the old status is a separate step with its own acceptance provenance — who accepted, the date, and the venue.

## Non-goals

Re-deriving the decision. The elimination is recorded in the deriving ticket and the ADR states it; this ticket transfers, it does not re-argue.

## Closes when

The ADR file exists at an allocated number with the drafted body byte-identical, the catalog names it, and `decision_status` is `proposed` with no acceptance claimed.

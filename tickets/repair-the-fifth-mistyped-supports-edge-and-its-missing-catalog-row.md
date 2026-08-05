---
id: repair-the-fifth-mistyped-supports-edge-and-its-missing-catalog-row
title: Repair the fifth mistyped supports edge and its missing experiment-catalog row
status: todo
priority: p3
dependencies: []
related: [repair-the-four-mistyped-typed-frontmatter-edges, reconcile-the-research-and-experiment-catalogs-with-their-frontmatter, redesign-the-delivered-realization-record-from-typed-evidence, derive-the-operation-family-and-signature-delivery-graph]
scopes: [research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [docs, navigation, catalog, metadata]
---
## User-visible outcome

The experiment catalog renders a row for every governed experiment record, and the one record it renders no row for stops being invisible to a reader browsing `spikes/README.md`.

## Why this exists

**Measurement — found on 2026-08-05 while landing the operation-family delivery graph, by running the check [`reconcile-the-research-and-experiment-catalogs-with-their-frontmatter`](reconcile-the-research-and-experiment-catalogs-with-their-frontmatter.md) already owns.** Over a named population of 37 experiment records against 36 rendered rows, it reports exactly one discrepancy: `MISSING experiment rows for ['tiler.spike.numerics.delivered-realization-record']`. The same run reports zero discrepancies on the research side over 84 rows against 84 records, so the check reached both halves.

**Fact — the discrepancy pre-dates that landing and is reproducible at its base.** Re-running the experiment half against the tree at `b63dd5d0` reports the same single missing id, so it is not a consequence of the delivery-graph change; that change added a research record and a research row, and touched no experiment record and no experiment row.

**Fact — the missing row is the second half of a mistyped edge, not an omission on its own.** `spikes/numerics/delivered-realization-record/README.md` declares `supports: ["tiler.contract.artifact-abi", "tiler.contract.numerical-semantics"]`. [`docs/document-metadata.md`](../docs/document-metadata.md) types `supports` as **experiment to research**, so both targets are the wrong kind. This is a fifth instance of exactly the class [`repair-the-four-mistyped-typed-frontmatter-edges`](repair-the-four-mistyped-typed-frontmatter-edges.md) enumerates — its own check encodes the rule as `rules.append(("supports", ("research",)))` — and it appeared after that ticket's table was written, which is why it is filed rather than folded in silently.

**Inference — the two halves must be repaired together and in that order.** Rendering a row now would render a `supports:` clause derived from two contract ids, which the catalog's own derivation cannot express; the edge has to point at the research record that states the bounded universe and procedure behind the packet's claims first, and the row then renders from it. Rendering the row first would bake the mistyped edge into the view that is supposed to check it.

## Required work

- Decide the correct `supports` target by reading what the packet actually establishes, not by picking the nearest research record. If no existing research record states the packet's boundary, that absence is the finding and the repair is a research record rather than a re-pointed edge.
- Repoint the edge, then add the `spikes/README.md` row in the **same** change, since the catalog is a hand-maintained derived view with no generator.
- Re-run the reconciliation check and report the count, which must reach zero for this id, with its failing perturbation watched — drop the row again and see it reported.

## Explicit non-goals

- Widening `supports` to admit a contract target. The metadata contract records the measured reason it declined that relaxation, and reversing it is an ADR-level decision rather than a repair — the four-edge ticket says so in its own non-goals and this ticket inherits it.
- The other four edges, which that ticket owns. This is a fifth, filed separately because it was found separately and because its catalog half is a different symptom.

## Closes when

The edge points at a `research` target, the experiment catalog renders the row derived from it, and the reconciliation check reports no missing experiment row for this id over a named population.

## Graph maintenance

- Filed by [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md), which ran the reconciliation check as one of its own required checks and found this survivor. That ticket held `contracts/navigation` and could have added the row; it did not, because adding a row over a mistyped edge is the repair that hides the defect.
- `research/numerics` is declared because the spike record carrying the edge lives under `spikes/numerics/**`, which that scope maps. `contracts/navigation` is additionally required for the `spikes/README.md` row and must be added by whoever claims this.

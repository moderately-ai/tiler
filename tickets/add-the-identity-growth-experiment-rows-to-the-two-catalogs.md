---
id: add-the-identity-growth-experiment-rows-to-the-two-catalogs
title: Add the identity-growth experiment rows to the two catalogs
status: done
priority: p3
dependencies: []
related: [measure-executable-coverage-identity-growth-against-the-program-identity-bound, derive-the-operation-family-and-signature-delivery-graph]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, navigation, catalog]
---
## User-visible outcome

Both hand-maintained catalogs render the identity-growth experiment — the experiment catalog lists the record, and the research catalog's row for [Complete model ingestion and execution](../docs/research/program-planning/complete-model-ingestion-and-execution.md) names it under `experiments:` — so the reconciliation check returns to its pre-existing baseline.

## Why this exists

**Fact.** [`measure-executable-coverage-identity-growth-against-the-program-identity-bound`](measure-executable-coverage-identity-growth-against-the-program-identity-bound.md) landed `spikes/program-planning/identity-growth/README.md`, a governed experiment record whose `supports:` names `tiler.research.program-planning.complete-model-ingestion-and-execution`. Both catalogs are hand-maintained derived views of that frontmatter: `spikes/README.md` renders the record's own row, and the research catalog's `experiments:` clause is the inverse of the experiment records' `supports:` lists. Neither was written, so both are one entry short.

**Fact — the measuring ticket did not write either row because both files are in `contracts/navigation`, which it does not hold.** [`derive-the-operation-family-and-signature-delivery-graph`](derive-the-operation-family-and-signature-delivery-graph.md) holds that scope. File-level disjointness against that worker's actual branch diff was checked rather than assumed:

```sh
git diff --name-only $(git merge-base main tkt/derive-the-operation-family-and-signature-delivery-graph)..tkt/derive-the-operation-family-and-signature-delivery-graph
```

The diff **lists `docs/research/README.md`** and **does not list `spikes/README.md`**. So the research-catalog row was refuted outright, and the experiment-catalog row was admissible under the disjointness rule — but declaring the scope needed to write it made `tkt why` report a real batch conflict against a live p1 ticket, and one appended row is not worth holding a shared navigation scope open for. Both rows moved here instead, where one correctly-scoped ticket owns them together.

**Measurement — the exact gap this ticket closes.** At the measuring ticket's commit the reconciliation check reports **2 discrepancies against a population of 83 research rows / 83 research records / 38 experiments and 37 experiment rows**. One is `MISSING experiment rows for ['tiler.spike.numerics.delivered-realization-record']`, which is **pre-existing** — the same check at base `5f14cd11` reports it against 37 experiments / 36 rows and is not this ticket's. The other is the `complete-model-ingestion-and-execution` `experiments:` clause, which is.

## Required work

- Add the experiment row to `spikes/README.md`, in the *Physical planning and lowering* section between `Exhaustive fusion-region oracle experiment` and `Kernel-program planning experiment`:

  ```
  - [How kernel-program identity grows against its 64 MiB bound](program-planning/identity-growth/README.md) — reproducible; bounded-measurement, executable-model; supports: [Complete model ingestion and execution](../docs/research/program-planning/complete-model-ingestion-and-execution.md)
  ```

- Add `How kernel-program identity grows against its 64 MiB bound` to the `experiments:` clause of the `complete-model-ingestion-and-execution` row in `docs/research/README.md`, matching the record's title exactly.
- Re-run the reconciliation check from [`reconcile-the-research-and-experiment-catalogs-with-their-frontmatter`](reconcile-the-research-and-experiment-catalogs-with-their-frontmatter.md) and report the count against its named population. It must reach the pre-existing baseline — the `delivered-realization-record` row above, plus any `UNGOVERNED` rows [`govern-the-three-ungoverned-spike-records`](govern-the-three-ungoverned-spike-records.md) owns — and nothing else.
- Watch the check fail first. The measuring ticket already demonstrated it can: mistyping the row's title to `64 MB` and repointing its `supports` link made the check report both `TITLE` and `SUPPORTS` against that exact row, taking the count from 2 to 4.

## Explicit non-goals

No generator, no gate, no schema change. Not repairing the pre-existing `delivered-realization-record` row, which belongs to whoever owns that record.

## Closes when

Both rows render the experiment, and the reconciliation check reports only the pre-existing discrepancies over a named population.

## Outcome — delivered 2026-08-05

`55dda5c8` added both derived views. `spikes/README.md` now carries **How
kernel-program identity grows against its 64 MiB bound** in the physical
planning and lowering group with its `supports` link, and the **Complete model
ingestion and execution** row in `docs/research/README.md` carries the inverse
`experiments:` link. The commit restored the reconciliation baseline recorded
by the measuring ticket; it did not absorb the separately owned
delivered-realization row or introduce a generator, schema, or gate. Both rows
remain present at the current tree.

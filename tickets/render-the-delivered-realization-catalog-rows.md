---
id: render-the-delivered-realization-catalog-rows
title: Render the delivered-realization catalog rows
status: done
priority: p3
dependencies: [repair-the-fifth-mistyped-supports-edge-and-its-missing-catalog-row]
related: [reconcile-the-research-and-experiment-catalogs-with-their-frontmatter, repair-the-four-mistyped-typed-frontmatter-edges]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, navigation, catalog, metadata]
---
## User-visible outcome

The experiment catalog renders a row for the delivered-realization design packet and the research catalog renders a row for the record stating its boundary, so both stop being invisible to a reader browsing `spikes/README.md` and `docs/research/README.md`.

## Why this exists

**Fact — this is the second half of [`repair-the-fifth-mistyped-supports-edge-and-its-missing-catalog-row`](repair-the-fifth-mistyped-supports-edge-and-its-missing-catalog-row.md), split out because its scope was unavailable, not because the work is unclear.** That ticket repaired the mistyped `supports` edge and found, as its Required work anticipated, that no existing research record stated the packet's boundary — so the repair was a new research record, [`docs/research/numerics/delivered-numerical-realization-record.md`](../docs/research/numerics/delivered-numerical-realization-record.md), rather than a re-pointed edge. Both catalog files live in `contracts/navigation`, which that ticket did not hold.

**Measurement — the rows are already written and already proved to close the check.** Before landing them the reconciliation check embedded in [`reconcile-the-research-and-experiment-catalogs-with-their-frontmatter`](reconcile-the-research-and-experiment-catalogs-with-their-frontmatter.md) reports `DISCREPANCIES: 2` over 91 research rows against 92 research records and 40 experiment rows against 41 experiment records. Inserting exactly the two rows below and re-running reports `population: 92 research rows, 92 research records` and `41 experiment rows against 41 records`, `DISCREPANCIES: 0`. That dry run was performed at `fac629a7` plus the edge repair and then reverted, so the rows are transcribed from a run that reached zero rather than composed by hand here.

## Required work

Insert into `docs/research/README.md`, in `### Numerical operations`, immediately before the `The elementary-identity rewrite dimension` row:

```text
- [The delivered numerical realization record](numerics/delivered-numerical-realization-record.md) — adopted; executable-model, exhaustive-finite; informs: [Numerical semantics](../numerical-semantics.md), [Artifact envelope and Metal kernel ABI profile](../artifact-abi.md), [Declare which numerical realizations a target honours](../decisions/0076-declare-target-honourable-numerical-realizations.md); experiments: [The delivered-realization record, redesigned from typed evidence](../../spikes/numerics/delivered-realization-record/README.md)
```

Insert into `spikes/README.md`, in the numerics group, immediately before the `Elementary-identity folding probe` row:

```text
- [The delivered-realization record, redesigned from typed evidence](numerics/delivered-realization-record/README.md) — reproducible; executable-model, exhaustive-finite; supports: [The delivered numerical realization record](../docs/research/numerics/delivered-numerical-realization-record.md)
```

Re-run the reconciliation check and report `DISCREPANCIES: 0` over its named population, having first watched it fail — drop either row again and see it reported as `MISSING`. Do not trust the row text above without that run: the corpus moves, and a record retitled meanwhile would make these rows wrong in exactly the way the check exists to catch.

## Explicit non-goals

No generator and no gate. The four typed edges [`repair-the-four-mistyped-typed-frontmatter-edges`](repair-the-four-mistyped-typed-frontmatter-edges.md) owns are not this ticket's; the typed-edge check reports `MISTYPED: 4` after the fifth-edge repair and those four are all of it.

## Closes when

Both rows render, and the reconciliation check reports zero discrepancies over a named population with its failing perturbation watched.

## Graph maintenance

- `contracts/navigation` is the only scope required, because both files map to it and nothing else changes.
- Depends on the fifth-edge repair: the rows render frontmatter that ticket introduces, and inserting them first would render a row for a record that does not exist.

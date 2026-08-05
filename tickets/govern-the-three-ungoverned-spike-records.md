---
id: govern-the-three-ungoverned-spike-records
title: Govern the three spike records the experiment catalog renders from nothing
status: todo
priority: p2
dependencies: []
related: [reconcile-the-research-and-experiment-catalogs-with-their-frontmatter, list-the-corpus-reachability-spike-in-the-spike-index]
scopes: [research/target-profiles, research/program-planning, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, navigation, catalog, spikes]
---
## User-visible outcome

Every row of the experiment catalog is a rendering of frontmatter that exists, so the reconciliation check can evaluate the whole population instead of naming three rows it cannot reach — and the `supports` edges those three spikes actually carry become reachable from the research catalog, where two of them are missing today.

## Why this exists

**Measurement — three of thirty-six experiment-catalog rows point at READMEs that are not governed documents, at `5f810e9a`.** Found by [`reconcile-the-research-and-experiment-catalogs-with-their-frontmatter`](reconcile-the-research-and-experiment-catalogs-with-their-frontmatter.md), whose check aborted on the first of them with a `KeyError` before that ticket's worker made it report and count them instead. The rows are hand-written prose in [`spikes/README.md`](../spikes/README.md); nothing behind them derives from metadata, because there is no metadata.

| Spike | Frontmatter state |
| --- | --- |
| [`spikes/target-profiles/metal-grid-axis-extent/`](../spikes/target-profiles/metal-grid-axis-extent/README.md) | a five-key block with **unquoted** scalar values; no `schema`, `kind`, `title`, `topics`, `implementation_status`, or `supports` |
| [`spikes/program-planning/reduction-crossover/`](../spikes/program-planning/reduction-crossover/README.md) | the same five-key unquoted block, same six keys absent |
| [`spikes/program-planning/qwen3-corpus-reachability/`](../spikes/program-planning/qwen3-corpus-reachability/README.md) | no frontmatter block at all |

[`docs/document-metadata.md`](../docs/document-metadata.md) requires `schema`, `id`, `kind`, `title`, and `topics` on every governed document and `experiment_status`, `implementation_status`, `evidence_classes`, and `supports` on every experiment, and fixes the encoding as "every non-delimiter line is `key: <JSON value>`" — which the unquoted scalars are not.

**Inference — two rendered `supports` claims are already wrong, and only frontmatter can settle them.** The `qwen3-corpus-reachability` row names one supported record, while [`list-the-corpus-reachability-spike-in-the-spike-index`](list-the-corpus-reachability-spike-in-the-spike-index.md) — the ticket that authored the row, now `done` — specifies three: `model-level-qualification`, `first-metal-lm-workload`, and `complete-model-ingestion-and-execution`. Separately the `reduction-crossover` row renders `supports: [Fusion and scheduling](../docs/compiler/fusion-and-scheduling.md)`, which is a **contract**, and the metadata contract types `supports` as experiment-to-research only. Neither is a rendering slip that editing the row fixes; each needs a decision about what the spike establishes, recorded as an edge.

**Fact — the grid-axis spike is load-bearing for a record that currently shows no experiment for it.** [The authority ledger](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md) sources its grid-axis row as `Measurement` from `spikes/target-profiles/metal-grid-axis-extent`, run 2026-08-04, 6,294 dispatched rows. The reconciliation ticket added the ledger's *other* experiment edge (from `spikes/apple-targets`, which supplies its numerical and dispatchability rows); the grid-axis edge cannot exist until this spike is a governed record, so the research catalog under-reports the ledger's evidence until then.

## Required work

- Author complete, contract-conforming `tiler-doc/v1` frontmatter on each of the three READMEs, quoting every scalar, and confirm each `title` matches the file's first level-one heading.
- Derive each `supports` list by reading what the spike measured and what the research record claims — not by transcribing the existing catalog row, which is the artefact under repair. Resolve the `reduction-crossover` contract target explicitly: name the research record that carries the claim, or record that none does and why the spike's edge is elsewhere.
- Restore the three `supports` targets `list-the-corpus-reachability-spike-in-the-spike-index` specified for the corpus-reachability probe, or record why that ticket's list is now wrong.
- Update the rows in `spikes/README.md` and any newly implied `experiments:` clause in [`docs/research/README.md`](../docs/research/README.md) in the same change, since both are derived views and nothing regenerates them.

## Explicit non-goals

No generator, no gate, no schema change, and no re-running of any spike. The measurements stand; this ticket gives them governed identity.

## Closes when

The reconciliation check in [`reconcile-the-research-and-experiment-catalogs-with-their-frontmatter`](reconcile-the-research-and-experiment-catalogs-with-their-frontmatter.md) reports `0 ungoverned` and zero discrepancies over its counted population, and the two mis-rendered `supports` claims above are each resolved by a recorded reading rather than by matching the row to the frontmatter or the frontmatter to the row.

---
id: acquire-the-six-flagged-optimizer-literature-sources
title: Acquire the six flagged optimizer-literature sources
status: done
priority: p3
dependencies: []
related: [survey-and-select-the-rewrite-search-formalism-against-the-optimizer-literature]
scopes: [research/region-search]
shared_scopes: [project/tickets]
paths: []
tags: [research, sources, acquisition]
---
## User-visible outcome

The six `pending-acquisition` rows in [the optimizer-literature source record](../docs/research/region-search/sources/README.md) become read documents — or, where a document genuinely cannot be obtained, rows that say so with the route exhausted rather than untried.

## Why this exists

**Fact.** The rewrite-search formalism survey could not retrieve six sources it identified. Each is filed with the exact reference, what was tried, and what it would decide; none is cited anywhere in the record, and no conclusion depends on one. `docs/research/region-search/sources/verify-sources.sh` counts the class, so the outstanding set is one command.

**This ticket needs a human with a browser for most of it.** Four of the six failed to automated retrieval specifically — `dl.acm.org` returns HTTP 403 to non-browser clients across every route tried, which is bot protection rather than a paywall.

## The six, ordered by what they would change

1. **`sparse-extraction-oopsla-2024`** — Goharshady, Lam, Parreaux, "Fast and Optimal Extraction for Sparse Equality Graphs", PACMPL 8(OOPSLA2), DOI `10.1145/3689801`. Converts a *relayed* link in the record's elimination into a read one, and may weaken it: it carries a fast optimal extraction algorithm for sparse e-graphs.
2. **`conditional-eqsat-successor-unlocated`** — a peer-reviewed successor to colored e-graphs, if one exists. **Highest value for changing a conclusion.** This is a search request: the survey did not locate the reference and one guessed arXiv id resolved to an unrelated paper.
3. **`denali-pldi-2002`** — Joshi, Nelson, Randall, DOI `10.1145/512529.512566`, or SRC-RR-171 via a Wayback capture. Goal-directed rather than saturating e-graph use.
4. **`metaflow-mlsys-2019`** — Jia et al., "Optimizing DNN Computation with Relaxed Graph Substitutions", MLSys 2019. A *measured* instance of the record's own finding that a search must not prune on cost before a composition completes.
5. **`elevate-icfp-2020`** — Hagedorn et al., PACMPL 4(ICFP), gold open access. The hand-written baseline guided equality saturation compares rule counts against.
6. **`columbia-optimizer-thesis-1998`** — Xu, PSU. Host unreachable. Calibration only; changes no verdict.

## What landing looks like

For each acquired document: move its manifest row's class, fill its digest, update the counts at the top of `verify-sources.sh`, rewrite its README section from `pending-acquisition` into a real provenance record with a licence verdict read from the acquired text, and — the part that matters — state in [the record](../docs/research/region-search/rewrite-search-formalism.md) whether it confirmed, qualified, or refuted the claim it was flagged against. An acquisition that does not report back on its own question is half-done.

## Closes when

Every one of the six is either read and reported against its question, or has its row updated with an exhausted route and the reason it cannot be obtained. `verify-sources.sh` passes with counts that match.

## Outcome

Completed 2026-08-06. Tom manually acquired all six flagged documents on 2026-08-05. Five became read provenance records that day; the sixth search request remained open after its first real candidate proved to be an unevaluated ongoing-work survey, then closed on 2026-08-06 when the peer-reviewed FMCAD 2024 publication of colored e-graphs was located and read. The final source census is 30 records: 10 vendored, 20 metadata-only, and 0 pending acquisition.

Reading changed the pre-acquisition account rather than merely filling it in. Sparse extraction strengthened the inapproximability evidence and carried limitations that exclude Tiler's acyclic extraction case. Denali saturates to quiescence and puts goal direction in SAT-based selection, contrary to item 3 above. MetaFlow confirmed the cost-regressing enabling relation while also demonstrating the infeasibility-as-infinite-cost collapse Tiler forbids. Elevate supplied the hand-written comparison, and Columbia supplied both the memo calibration and a measured quality-loss curve for epsilon pruning. The final conditional-equality-saturation document was not a successor: it was the peer-reviewed edition of the existing colored-egraph work, raising its evidence class and narrowing its applicability without changing the selected rewrite-search formalism.

The complete per-source provenance, reading bounds, corrections, and reproducible population check live under the source-record anchors `Acquisition note for the six formerly-flagged documents` and `The search request that closed`; the synthesis and unchanged formalism verdict are recorded under `What the six turned out to say` in the rewrite-search record. `docs/research/region-search/sources/verify-sources.sh` currently reports `OK: 30 records verified (10 vendored, 20 metadata-only, 0 pending-acquisition).`

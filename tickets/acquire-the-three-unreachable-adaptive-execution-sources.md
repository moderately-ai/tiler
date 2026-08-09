---
id: acquire-the-three-unreachable-adaptive-execution-sources
title: Acquire the three unreachable adaptive-execution sources
status: done
priority: p3
dependencies: []
related: [design-the-measured-feedback-tuning-loop-against-the-autotuning-and-adaptive-execution-literature]
scopes: [research/cost-model]
shared_scopes: [project/tickets]
paths: []
tags: [research, cost-model]
---
## User-visible outcome

The two conclusions [the measured-feedback tuning loop](../docs/research/cost-model/measured-feedback-tuning-loop.md) records as thinner than the rest are re-founded on the documents that would settle them, or the record states that they could not be.

## The three

1. **Volker Markl, Vijayshankar Raman, David Simmen, Guy Lohman, Hamid Pirahesh, Miso Cilimdzic, "Robust Query Processing through Progressive Optimization", ACM SIGMOD 2004, pp. 659-670. DOI `10.1145/1007568.1007642`.** Attempted 2026-08-07 at `https://cs.uwaterloo.ca/~david/cs848/paper-progressive-optimization.pdf` (HTTP 404); no further route tried.
2. **Richard L. Cole, Goetz Graefe, "Optimization of Dynamic Query Evaluation Plans", ACM SIGMOD 1994, pp. 150-160. DOI `10.1145/191839.191872`.** Attempted 2026-08-07 at `https://www.vldb.org/conf/1994/P150.PDF` (HTTP 404 — the attempt was misdirected at a VLDB volume rather than the SIGMOD proceedings, so the document's availability is untested rather than disproved).
3. **Goetz Graefe, Karen Ward, "Dynamic Query Evaluation Plans", ACM SIGMOD 1989, pp. 358-366. DOI `10.1145/67544.66960`.** Not attempted.

## What each would decide

**Markl et al.** — the design record's shape-transfer rule 2 requires a stored per-subject result to carry a *declared* validity region rather than a derived one, and it takes the validity-range mechanism from the adaptive-query-processing survey's description of this paper rather than from the paper. The survey states that ranges exist and that falling outside one triggers re-optimization; it does not state how a range is computed or how conservative it is. Reading the paper either supplies a construction the record can adopt, or confirms that declaring is the only safe option.

**Cole and Graefe, and Graefe and Ward.** — whether a run-time `choose-plan` operator is a better shape for Tiler's deferred-measurement case than the `AvailabilityPhase` ladder [ADR 0043](../docs/decisions/0043-use-typed-phased-target-feasibility.md) already provides. The design record decides against it without this evidence and names that as the one place a reader should expect the argument to be thinner. **Note the dependency:** a run-time plan choice implies shipping several plans in one artifact, which the design record records as a product expansion and Tom's decision — so this reading informs that decision rather than pre-empting it.

## Non-goals

Re-opening any decision the design record made on evidence it did have. If a source contradicts one, that is a finding to record and a follow-up to file, not an edit to make in passing.

## Outcome

Completed 2026-08-07. Markl et al. and Cole/Graefe were retrieved by plain `curl` from corrected public routes; the original attempts had been incomplete or misdirected. Graefe/Ward remained inaccessible to this host because every legitimate route ended at the ACM Digital Library bot wall, so Tom supplied the paper from a normal browser session. All three byte streams were identified, digested, read, and entered into the 14-record source population; `docs/research/cost-model/sources/verify-sources.sh` now reports 0 pending acquisition.

Both conclusions identified above survived and gained direct evidence. Progressive Optimization derives a conservative validity range from intersections between the optimizer's own cost functions while pruning alternatives. That does not safely derive the validity region of a machine observation: it bounds the wrong authority, deliberately admits a range wider than true optimality because POP can re-optimize, and depends on alternatives Tiler discards. Tiler's declared-region rule therefore remains.

The 1994 `choose-plan` design re-evaluates analytic costs at start-up and requires every potentially optimal alternative in the shipped DAG; its measured ten-way-join plan had 14,090 operator nodes versus 21. The 1989 origin precomputes break-even decisions and is cheaper at start-up, but still ships the alternatives and carries no optimality guarantee. Neither is a measured-feedback mechanism or a cheaper substitute for the existing `AvailabilityPhase` ladder. No design verdict changed; the two formerly thin conclusions were strengthened. The complete provenance, reading bounds, and secondary-source corrections live under the source-record anchors `markl-pop-sigmod-2004`, `graefe-ward-sigmod-1989`, and `cole-graefe-sigmod-1994`, and the integrated conclusions are recorded under `Sources and evidence gaps` in the measured-feedback design.

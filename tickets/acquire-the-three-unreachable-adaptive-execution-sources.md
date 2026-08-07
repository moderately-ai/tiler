---
id: acquire-the-three-unreachable-adaptive-execution-sources
title: Acquire the three unreachable adaptive-execution sources
status: todo
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

---
schema: "tiler-doc/v1"
id: "tiler.research.cost-model.reading-note.pqo-vldb-1992"
kind: "research"
title: "Reading note: Ioannidis, Ng, Shim and Sellis, Parametric Query Optimization (1992)"
topics: ["cost-model", "optimizer", "parametric-plans", "prior-art"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "informational"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.optimizer"]
ticket: "close-the-four-licence-readings-tom-supplied-and-admit-graefe-and-ward"
---

# Reading note: Ioannidis, Ng, Shim and Sellis, Parametric Query Optimization (1992)

Distilled from the byte stream pinned as `pqo-vldb-1992` in [the source record](../sources/README.md#pqo-vldb-1992), read 2026-08-07. That row carries the citation, digest, provenance, and licence verdict. The copy is a scan with OCR damage; every passage quoted here was taken from text that extracts unambiguously.

**Why this note exists.** This paper states Tiler's shape-transfer question thirty-four years earlier, in a different vocabulary, and the repository had been citing it for that framing without having read it since the licence pass.

## What the paper proposes

**The problem statement, which is the citation that matters.** "Parametric query optimization attempts to identify several execution plans, each one of which is optimal for a subset of all possible values of the run-time parameters." That is the shape of Tiler's question — one artifact, several candidate implementations, a partition of an input space by which one wins — stated for buffer size rather than tensor shape.

The motivation is that compile-time assumptions are violated at run time: "the database contents and the physical schema change incessantly …, the multiprogramming level of the system and the resource needs of concurrently running queries cannot be predicted, and queries may be executed with different bindings for their constants."

**The method is randomized search run concurrently across parameter values.** The paper adopts Iterative Improvement, Simulated Annealing, and Two Phase Optimization, and adds **sideways information passing**: rather than optimizing independently at each buffer size, one co-routine per parameter value shares discovered plans with its neighbours. **Measurement (theirs).** "these enhanced algorithms optimize queries for large numbers of buffer sizes in the same time needed by their conventional versions for a single buffer size, without much sacrifice in the output quality", with `sipIIs(1)` identified as most effective "for a very broad spectrum of cases".

**The output is a plan function, and the run-time step is a lookup.** "it offers a complete query optimization algorithm that has a plan function as output and makes no assumptions about any properties of the plan costs. … It will take a simple table look-up with the parameter values to identify the appropriate plan for the execution."

## What this settles for Tiler

**It names the design Tiler refuses, and prices it honestly.** A plan function over a parameter space, resolved by table lookup at run time, is exactly the per-shape winner table [the measured-feedback tuning loop](../measured-feedback-tuning-loop.md) declines. This paper is the strongest statement of the case *for* it, so it is worth being precise about why Tiler still says no, and the reason is not that the technique fails on its own terms.

- **The lookup presumes the alternatives are shipped.** Same retention obligation as the choose-plan line; see [`graefe-ward-sigmod-1989`](graefe-ward-sigmod-1989.md).
- **The partition is the expensive part, and the [plan-diagram measurements](plan-diagrams-vldb-2005.md) show it is mostly noise** — two-thirds of a dense partition discardable within a 10% cost threshold, and none of convexity, uniqueness, or homogeneity holding.
- **Inference.** The paper's own scope is the qualifier the repository should keep attached to its framing citation. It studies "primarily the buffer size parameter" — a single, ordered, low-cardinality, machine-level scalar. Tiler's shape space is multi-dimensional, and the paper's own closing work item concedes the gap: "it would be important to experiment with large vectors of diverse parameters to understand the scalability of the proposed algorithms." **So this paper is authority for the problem statement and not for the tractability of the solution at Tiler's dimensionality.**

**One property worth noting in the paper's favour.** It "makes no assumptions about any properties of the plan costs", which is precisely the assumption-freedom the plan-diagram paper later showed the rest of the PQO literature lacked. Its randomized search does not need convexity or continuity. **That does not rescue the winner table for Tiler** — the objection is retention and partition cost, not the search method — but it means this paper is not among those refuted by the 2005 measurements, and the record should not lump it in with them.

**No conclusion in the design record changes. This note proposes no edit to it.**

## Cross-reference worth recording

The paper cites Graefe and Ward's dynamic plans as the neighbouring approach, describing an optimizer that works "by essentially introducing choose-plan operators [GW89]". **The two lines were contemporaneous and mutually aware**, which is context the repository's reading of the choose-plan line should carry: parametric plans and dynamic plans are two answers to one question posed around 1989–1992, not successive refinements of one idea.

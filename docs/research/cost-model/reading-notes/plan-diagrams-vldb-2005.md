---
schema: "tiler-doc/v1"
id: "tiler.research.cost-model.reading-note.plan-diagrams-vldb-2005"
kind: "research"
title: "Reading note: Reddy and Haritsa, Analyzing Plan Diagrams of Database Query Optimizers (2005)"
topics: ["cost-model", "optimizer", "parametric-plans", "prior-art"]
catalog_group: "physical-planning-lowering"
research_status: "complete"
disposition: "informational"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis"]
informs: ["tiler.contract.cost-model"]
ticket: "close-the-four-licence-readings-tom-supplied-and-admit-graefe-and-ward"
---

# Reading note: Reddy and Haritsa, Analyzing Plan Diagrams of Database Query Optimizers (2005)

Distilled from the byte stream pinned as `plan-diagrams-vldb-2005` in [the source record](../sources/README.md#plan-diagrams-vldb-2005), read 2026-08-07. That row carries the citation, digest, provenance, and licence verdict.

**Why this note exists.** This is the paper [the measured-feedback tuning loop](../measured-feedback-tuning-loop.md)'s refusal of a per-shape winner table rests on, and its bytes were unreadable by this host until Tom supplied them. Reading it in full turned up a second result the repository had not been citing, which matters more to Tiler than the one it had.

## What the paper measures

A **plan diagram** is "a color-coded pictorial enumeration of the execution plan choices of a database query optimizer over the relational selectivity space" — the optimizer is run over a grid of selectivity settings and the winning plan at each point is plotted. The study covers TPC-H queries against three commercial optimizers, at a 100 × 100 grid.

**Measurement (theirs).** Real optimizers partition the space far more finely than the cost differences justify.

- Query 8 under one optimizer produces "no less than 68 plans cover the space in a highly convoluted manner", and the count is grid-dependent: "a finer grid size of 300 x 300 resulted in the plan cardinality going up to 80 plans".
- The partition is heavily skewed: "80 percent of the space is usually covered by less than 20 percent of the plans, with many of the smaller plans occupying less than one percent of the selectivity space", with Gini indices "in excess of 0.5 for most queries, on occasion going even higher than 0.8".
- Most of it is discardable. The 68-plan diagram "can be 'reduced' to that shown in Figure 2(b) featuring as few as seven plans, without increasing the estimated cost of any individual query point by more than 10 percent." Generalized: "two-thirds of the plans in a dense plan diagram are liable to be eliminated through plan swallowing", and the *average* cost increase from swallowing is "less than 2%" against that 10% threshold.

**Inference (theirs, and labelled as such by them).** "current optimizers may perhaps be over-sophisticated in that they make extremely fine-grained plan choices" not merited by the coarseness of the cost space.

## The result the repository was not citing

The paper's §4, *Relationship to PQO*, tests the three assumptions the parametric-query-optimization literature rests on:

- **Plan convexity** — "If a plan P is optimal at point A and at point B, then it is optimal at all points on the line joining the two points";
- **Plan uniqueness** — "An optimal plan P appears at only one contiguous region in the entire space";
- **Plan homogeneity** — "An optimal plan P is optimal within the entire region enclosed by its plan boundaries".

**Measurement.** "we find that none of the three assumptions hold true, even approximately, in the plan diagrams produced by the commercial optimizers." Each failure is exhibited: convexity broken by two named plan regions; uniqueness by a plan "which appears in two non-contiguous locations in the top left quadrant" while another "appears in finely-chopped pieces"; homogeneity by a small rectangle of one plan sitting inside another's optimality region. The paper also refutes a prior estimate that dense regions occur only near the selectivity axes, finding them "elsewhere in the selectivity space also".

## What this settles for Tiler

**It supplies the missing empirical half of [section 3, rule 2](../measured-feedback-tuning-loop.md#3-the-transfer-model-across-shapes).** Rule 2 requires a stored per-subject measurement to carry a **declared** validity region rather than a derived one, and it was argued from the shape of Tiler's own guarantees and from what POP's derivation assumes. The convexity result is the argument from the other side: **if the region where one plan wins is not convex, not contiguous, and not homogeneous, then observing that plan A wins at shape X and at shape Y licenses no conclusion about anything between them.** Interpolating a validity region between two measured points is unsound in general, not merely unproven.

**It is also the argument against a per-shape winner table**, which was already recorded: a table keyed on every distinguishable shape stores a partition two-thirds of which is noise, at a cost the 10%-threshold reduction shows nobody would pay for.

**No conclusion in the design record changes.** Both readings support conclusions already reached on other grounds. **This note proposes no edit to the design record's conclusions.**

## Bounds on the claim

**Measurement, not a universal.** The authors state their own limits and this note keeps them: they were "not being privy to optimizer internals", "some of the conclusions drawn here are perforce speculative in nature and should therefore be treated as such", and the study covers three commercial relational optimizers on TPC-H at a fixed grid. **The result bounds a general belief that plan optimality regions are well behaved. It establishes nothing about Tiler's cost space**, where no comparable diagram has ever been produced — a Tiler plan diagram over a shape space is a bounded experiment nobody has run, and it is what would be needed before any *quantitative* claim about Tiler's own region geometry could be made.

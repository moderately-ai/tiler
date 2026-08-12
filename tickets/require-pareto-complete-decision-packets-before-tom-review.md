---
id: require-pareto-complete-decision-packets-before-tom-review
title: Require Pareto-complete decision packets before Tom review
status: done
priority: p1
dependencies: []
related: []
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [process, decision, correctness, review]
---
## User-visible outcome

Every future consequential decision is critically and skeptically audited before presentation, and Tom sees only complete, nondominated options that are top-tier on correctness and strictness rather than an early plausible answer padded with weaker alternatives.

## Fact audit — exact base `914264833d21cb290783a47df99e75793b3b5251`

- **Verified.** Root `AGENTS.md`, anchor `Before escalating, compare options on correctness, maintainability, and performance`, required comparison but did not require a complete material option census, explicit hard eliminations, prerequisite healing, or a nondominated frontier.
- **Verified.** Root priorities already order correctness, long-term maintainability, then performance, while the architecture and decision sections reject silently wrong fast paths. The new gate strengthens the presentation procedure without changing those authorities.
- **Verified.** Root coordination guidance already requires source-first Fact audits, independent high-risk review, subject perturbations, and explicit remainder tickets. The new gate composes those existing controls at the point before Tom is asked to decide rather than creating a second workflow.

## Decision — accepted 2026-08-12

**Decided by Tom on 2026-08-12 in the live decision round, relayed by the coordinator from this ticket:** before presenting the next decision, agents must apply the new `Decision-packet readiness gate` in root `AGENTS.md`.

The gate requires exact-base source and authority reading; a materially complete option census; elimination of incorrect, silently defaulting, identity-conflating, validation-incomplete, or prerequisite-dependent terminal answers; comparison across correctness, fail-closed strictness, long-term maintainability/compatibility, and Tiler host runtime/memory; and presentation of only the nondominated frontier. Kernel performance remains a separate dimension unless the decision is about kernels.

If one answer dominates, agents do not manufacture a choice. If a real trade-off remains, every presented candidate states its strongest counterargument, reversal evidence, unsupported population, identity/schema consequences, negative controls, and explicit healing/dependency graph.

## Delivery

The canonical root guide carries the rule under the searchable anchor `Decision-packet readiness gate`, where every consequential worker, reviewer, coordinator, and user directing or accepting work is already required to read it in full. This ticket records who accepted it, the date, venue, exact base, and why the previous guidance was insufficient.

## Non-goals

Replacing source-first Fact audits, mechanical checks, independent high-risk review, or Tom's retained public-boundary and architecture authority; turning a qualitative architectural judgment into a numeric score; presenting dominated options for symmetry.

## Outcome

The durable canonical instruction and acceptance provenance are recorded. No product source, identity, schema, runtime behavior, or support claim changed.

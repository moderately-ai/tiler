---
id: route-subagent-models-by-change-risk
title: Route subagent models by change risk
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [implementation/workspace]
shared_scopes: [project/tickets]
paths: []
tags: [coordination, agents]
claimed_from: todo
assignee: coord
lease_expires_at: 1786224856
---
Record the established subagent model-routing policy in AGENTS.md: use Sol for authority, identity, public-boundary, and broad cross-layer work; use Terra for tightly bounded work with clear invariants; select independent review by risk instead of running an ongoing model comparison.

## Outcome

The repository guide now routes author and reviewer models by the consequences of a wrong answer. It names escalation conditions for a bounded Terra lane and makes clear that model choice does not replace the source-first Fact audit, subject perturbation, exact-base review, or gates.

## Evidence

The 2026-08-08 comparison supplied the boundary: Terra completed the bounded conformance-status history repair without a review finding, while the artifact digest-ordinal lane needed Sol review to find a rendered-source anchor and a second live contract ordinal outside the first repaired paragraph. That is evidence for risk-based routing, not for continuing a standing comparison.

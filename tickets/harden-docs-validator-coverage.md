---
id: harden-docs-validator-coverage
title: Harden documentation validator coverage
status: in-progress
priority: p1
dependencies: []
related: []
scopes: [contracts/navigation, contracts/decisions, research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, tooling]
claimed_from: todo
assignee: agent-harden-docs-validator-coverage
lease_expires_at: 1784823821
---
Three verified gaps where docs.py enforces less than document-metadata.md promises, letting status-bearing prose drift silently:

- the hand-maintained "Chronological index" in docs/decisions/README.md is invisible to the validator and stale (ends at ADR 0059 while the corpus reaches 0072); fold it into the generated-catalog machinery (extend the renderer and markers) or delete it in favour of the generated thematic catalog, then regenerate;
- docs.py licenses `related` frontmatter on every kind while document-metadata.md's kind-field table licenses it on four kinds only, and live instances exist on unlicensed kinds (ADR-0056, ADR-0070, one research doc); reconcile contract and validator in one deliberate direction and migrate the instances; and
- entrypoints/last_verified well-formedness and date checks run only when experiment_status is "reproducible" while the metadata contract states the field rules unconditionally; validate them on every experiment record.

Also correct the stale status prose this ticket's scope owns: docs/status.md line 88 still names the completed verifier subject-binding correction as "the immediate compiler frontier" (that ticket is done and merged), and docs/roadmap.md still lists it as pending work. Point both at the current frontier.

Lock each closed gap with a scripts/tests case so the gate cannot regress. Run the full documentation gate before completion. This ticket exclusively holds `contracts/navigation`; `repair-research-evidence-residuals` also needs it for two catalog-regenerating frontmatter fixes, so merge this ticket first and let that one follow.

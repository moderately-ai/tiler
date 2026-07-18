---
id: research-readiness-gate
title: Run the research-to-implementation readiness gate
status: todo
priority: p1
dependencies: [synthesize-core-contracts, synthesize-optimizer-contracts, synthesize-artifact-contracts]
related: []
scopes: [contracts/core]
shared_scopes: [contracts/compiler, contracts/artifacts, contracts/integrations]
paths: []
tags: [tiler-research, gate, decision]
---
Audit the synthesized design for contradictions, missing invariants, unmeasured feasibility claims, and decisions that would force crate or IR boundaries to change. Rank remaining unknowns by architecture impact and experimental cost, then propose the smallest defensible implementation slice.

This ticket does not authorize implementation. It is complete only after Tom reviews the gate, unresolved blockers remain explicit, and the roadmap records whether the project is ready for scaffolding, needs another research wave, or must narrow scope.

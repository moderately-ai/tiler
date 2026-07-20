---
id: region-search-oracle
title: Build a tiny exhaustive oracle for fusion-region search
status: done
priority: p1
dependencies: [semantic-graph-contract, numerical-policy-contract]
related: []
scopes: [research/region-search]
shared_scopes: []
paths: []
tags: [tiler-research, spike, optimizer]
---
Define RegionCandidate and ImplementationFrontier semantics, then construct a tiny exhaustive enumerator for small DAGs to expose legal fusion regions, overlapping alternatives, duplicated work, and materialization choices. Use it as an oracle for future bounded heuristics, not as the production optimizer.

Deliver representative graphs, the enumerated alternatives, legality-rejection explanations, and proposed bounds for the first heuristic search. Include multiple graph outputs and shared producers.

---
id: prototype-optimizer-conformance-gate
title: Gate the target-neutral optimizer conformance profile
status: todo
priority: p0
dependencies: [prototype-neutral-program-and-artifact-types]
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference, implementation/artifact]
shared_scopes: []
paths: []
tags: [implementation, optimizer, conformance, milestone-0b]
---
Exercise the ordinary compiler path over at least two non-isomorphic graph shapes: generic occurrences, region enumeration, legality evidence, complete selection, verified KIR, neutral program construction, stable explain, deterministic identity, and correct failure taxonomy. Remove proof-only candidate lists and downstream cfg(test) isolation after interface review.

Include identity conformance for provider-only revision changes, identical
region/index/schedule structure at distinct occurrences, occurrence-specific
refinements, and complete-plan coverage. Each change must affect only the
identity and provenance subjects governed by ADR 0072.

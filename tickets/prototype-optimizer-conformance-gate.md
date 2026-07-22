---
id: prototype-optimizer-conformance-gate
title: Gate the target-neutral optimizer conformance profile
status: todo
priority: p0
dependencies: [enforce-repository-validation-gate-integrity, prototype-artifact-program-model]
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, conformance, milestone-0b]
---
Exercise an externally registered operation through the ordinary compiler path,
not a test-only shortcut. Cover at least two non-isomorphic graph shapes plus
fan-out or ordered multi-output behavior: generic occurrences, checked
refinement, region enumeration, legality evidence, complete selection, verified
KIR, neutral and artifact program construction, typed stable explain,
deterministic identity, and the correct failure taxonomy. Remove proof-only
candidate lists and downstream `cfg(test)` isolation after interface review.

Include identity conformance for provider-only revision changes, identical
region/index/schedule structure at distinct occurrences, occurrence-specific
refinements, and complete-plan coverage. Assert identity and selected-provider
provenance at every implemented layer. Each change must affect only the identity
and provenance subjects governed by ADR 0072.

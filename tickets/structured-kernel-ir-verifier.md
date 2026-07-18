---
id: structured-kernel-ir-verifier
title: Validate the structured kernel IR and verifier boundary
status: todo
priority: p1
dependencies: [scheduled-region-model, index-access-model, target-profile-feasibility-model]
related: []
scopes: [research/kernel-ir]
shared_scopes: []
paths: []
tags: [tiler-research, spike, kernel-ir, gpu]
---
Lower representative scheduled regions into a typed structured kernel form with explicit control flow, address spaces, conversions, barriers, loads, stores, and built-in invocation coordinates. Determine what must be proven before backend source emission.

Deliver well-formed and intentionally invalid examples, verifier responsibilities, backend assumptions, and evidence that the representation can express the first Metal schedules without embedding Metal syntax or runtime objects.

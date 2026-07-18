---
id: apple-artifact-compatibility
title: Measure Apple artifact target and deployment compatibility
status: todo
priority: p1
dependencies: [artifact-envelope-model, target-profile-feasibility-model]
related: []
scopes: [research/apple-targets]
shared_scopes: []
paths: []
tags: [tiler-research, spike, metal, measurement]
---
Create a bounded compatibility matrix for macOS, iOS device, iOS simulator, deployment minima, SDK identities, and supported toolchain versions. Determine whether incompatibility appears at metallib load, function lookup, or pipeline creation and evaluate Catalyst as support, rejection, or deferral.

Record exact commands, hosts and devices, compiler metadata, artifacts, and results. Separate local measurement from portable contract; do not generalize across untested OS or GPU families.

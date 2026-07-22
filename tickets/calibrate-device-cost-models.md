---
id: calibrate-device-cost-models
title: Calibrate analytical costs for selected device profiles
status: deferred
priority: p2
dependencies: [implement-analytical-component-cost-model]
related: []
scopes: [implementation/compiler, research/cost-model]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, measurement, cost-model, deferred]
---
Activate only after representative kernels, exact target profiles, devices,
and a reproducible benchmark protocol are selected. Fit and validate component
parameters with held-out measurements, provenance, uncertainty, drift policy,
and an explicit activation threshold. Until then the analytical model remains
uncalibrated and must not claim device-optimal latency.

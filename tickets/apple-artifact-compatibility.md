---
id: apple-artifact-compatibility
title: Measure Apple artifact target and deployment compatibility
status: done
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

## Outcome

Delivered the [bounded compatibility report](../docs/research/apple-targets/artifact-compatibility.md)
and [reproducible probes](../spikes/apple-targets/README.md). All six compile
tuples were exercised on the recorded host; old-device, old-OS, and
cross-machine runtime compatibility remain follow-up evidence.

## Evidence correction (2026-07-21)

The original record did not bind its producer inputs and did not retain every
historically described intermediate observation. The
[Apple experiment repair](repair-apple-target-experiment-integrity.md) and
[current report](../docs/research/apple-targets/artifact-compatibility.md)
replace it with a schema-v2 compile record bound to the retained harness,
validator, source, manifest, and toolchain metadata. Runtime compatibility and
the cause of AIR digest differences remain unmeasured.

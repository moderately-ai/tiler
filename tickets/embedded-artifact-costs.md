---
id: embedded-artifact-costs
title: Measure direct embedded-artifact costs across crates
status: done
priority: p2
dependencies: []
related: []
scopes: [research/embedding]
shared_scopes: []
paths: []
tags: [tiler-research, spike, measurement, artifact]
---
Extend the existing byte-literal observations into a reproducible matrix over artifact sizes, invocation counts, identical versus unique bytes, crate boundaries, profiles, codegen units, LTO, debug information, and incremental rebuilds.

Capture wall time, peak memory, intermediate and final sizes, and section duplication. Produce evidence-based thresholds and diagnostics; do not assume linker deduplication is guaranteed.

## Outcome

Delivered the [measurement report](../docs/research/embedding/embedded-artifact-costs.md)
and [reproducible harness](../spikes/embedding/README.md). The initial gates are
bounded host measurements; linker deduplication is explicitly not guaranteed.

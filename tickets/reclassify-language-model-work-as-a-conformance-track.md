---
id: reclassify-language-model-work-as-a-conformance-track
title: Reclassify language-model work as a consumer conformance track
status: in-progress
priority: p0
dependencies: [reconcile-the-roadmap-and-public-facades-with-the-consumer-neutral-mission]
related: [supersede-the-runtime-owned-kv-state-design, retain-the-c1-attention-block-conformance-evidence, retain-the-qwen-conformance-reference-logit-fixture]
scopes: [contracts/navigation, contracts/integrations, research/program-planning, research/runtime, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [architecture, conformance, language-model, roadmap]
claimed_from: todo
assignee: agent-reclassify-lm
lease_expires_at: 1785876199
---
## User-visible outcome

Language-model examples remain valuable end-to-end evidence, but they test the
generic compiler as consumer-owned tensor programs. They do not define Tiler's
product goal, semantic model, runtime state, or public API.

Inventory every language-model, attention, rotary, normalization, quantization,
prefill, decode, KV, and Candle-specific roadmap node. Classify each as one of:

1. a generic atomic operation or compiler/runtime capability, renamed and specified
   without workload ownership;
2. a consumer integration/conformance fixture that composes generic capabilities;
3. a performance study whose result may motivate a generic optimization; or
4. obsolete work whose premise is superseded.

Preserve exact numerical fixtures and measurements with their bounded profiles.
Ensure integration tests do not become alternate semantic authorities. Correct the
roadmap ladder and dependency graph, filing generic prerequisite tickets where a
workload currently hides a missing atomic building block.

## Closes when

Every affected node has one classification, core capabilities have consumer-neutral
names and contracts, application loops/state remain in consumer integration scope,
and the roadmap presents language models as one demanding conformance track among
many possible tensor workloads.

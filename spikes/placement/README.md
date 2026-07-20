---
schema: "tiler-doc/v1"
id: "tiler.spike.placement"
kind: "experiment"
title: "Placement and memory-domain model"
topics: ["placement", "memory-domains", "physical-properties"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model"]
supports: ["tiler.research.placement.device-memory-domains"]
entrypoints: ["spikes/placement/placement_domain_model.rs"]
last_verified: "2026-07-20"
ticket: "device-placement-and-memory-domain-contract"
---

# Placement and memory-domain model

The model tests symbolic affinities, capability-graph feasibility, and explicit
placement enforcers without calling a device API.

```sh
rustc --edition 2021 --test spikes/placement/placement_domain_model.rs -o /tmp/tiler-placement-tests
/tmp/tiler-placement-tests
```

It does not predict transfer costs or implement distributed scheduling. See the
[placement report](../../docs/research/placement/device-placement-and-memory-domains.md).

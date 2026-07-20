---
schema: "tiler-doc/v1"
id: "tiler.spike.numerics.region-accuracy"
kind: "experiment"
title: "Region accuracy observation probe"
topics: ["numerics", "accuracy", "measurement"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.numerics.region-accuracy-contract"]
ticket: "research-region-accuracy-contracts-and-analyzable-error-budgets"
---

# Region accuracy observation probe

This dependency-light `mpmath` probe demonstrates cancellation, reference
choice, materialization removal, relative error at zero, and reduction-topology
sensitivity. Its observations are empirical; they do not establish a sound
worst-case bound.

## Reproduce

From the repository root with `mpmath` available:

```sh
python3 spikes/numerics/region_accuracy_probe.py
```

## Traceability

- **Supported claim:** [Region accuracy contracts and analyzable error budgets](../../../docs/research/numerics/region-accuracy-contract.md).
- **Normative owner:** [Correctness and testing](../../../docs/correctness-and-testing.md).
- **Work record:** [region-accuracy research](../../../tickets/research-region-accuracy-contracts-and-analyzable-error-budgets.md).

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
entrypoints: ["spikes/numerics/region_accuracy_probe.py"]
last_verified: "2026-07-21"
ticket: "research-region-accuracy-contracts-and-analyzable-error-budgets"
---

# Region accuracy observation probe

This probe uses the repository-locked `mpmath==1.3.0` oracle to demonstrate cancellation, reference
choice, materialization removal, relative error at zero, and reduction-topology
sensitivity. Its observations are empirical; they do not establish a sound
worst-case bound.

## Reproduce

From the repository root:

```sh
uv run --locked python spikes/numerics/region_accuracy_probe.py
uv run --locked python -O spikes/numerics/region_accuracy_probe.py
```

The probe uses explicit verdict checks rather than Python `assert`, so optimized
Python cannot silently discard its witness validation. Either command exits
nonzero instead of publishing JSON when the dependency version, oracle
precision, or an expected adversarial result changes.

[`results.json`](results.json) is the byte-for-byte output retained from both
modes on the recorded host. It binds the exact probe source, algorithm,
interpreter, host, `mpmath` version, and 100-digit precision. Another
environment may produce a new bounded record; it must not silently overwrite
the provenance of this one.

## Traceability

- **Supported claim:** [Region accuracy contracts and analyzable error budgets](../../../docs/research/numerics/region-accuracy-contract.md).
- **Normative owner:** [Correctness and testing](../../../docs/correctness-and-testing.md).
- **Work record:** [region-accuracy research](../../../tickets/research-region-accuracy-contracts-and-analyzable-error-budgets.md).

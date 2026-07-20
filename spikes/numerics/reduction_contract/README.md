---
schema: "tiler-doc/v1"
id: "tiler.spike.numerics.reduction-contract"
kind: "experiment"
title: "Reduction contract probe"
topics: ["numerics", "reductions", "semantics"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "exhaustive-finite"]
supports: ["tiler.research.numerics.reduction-semantics-and-legality"]
entrypoints: ["spikes/numerics/reduction_contract_probe.py"]
last_verified: "2026-07-20"
ticket: "reduction-semantics-contract"
---

# Reduction contract probe

This dependency-free host model exercises strict serial reduction semantics,
empty domains, seed placement, typed accumulator boundaries, and adversarial
floating-point cases. It does not measure a GPU lowering or prove parallel
reduction topology.

The `exhaustive-finite` claim is limited to all 24 permutations of the single
four-value cancellation witness `[1e20, -1e20, 3.0, 4.0]`. Other seed, empty,
conversion, and exceptional-value cases are selected witnesses, not exhaustive
domains.

## Reproduce

From the repository root:

```sh
python3 spikes/numerics/reduction_contract_probe.py
```

The script exits nonzero on a failed assertion and otherwise prints
`reduction contract probe: all witnesses passed`.

## Traceability

- **Supported claim:** [Reduction semantics and legality](../../../docs/research/numerics/reduction-semantics-and-legality.md).
- **Normative owner:** [Numerical semantics](../../../docs/numerical-semantics.md).
- **Work record:** [reduction-semantics-contract](../../../tickets/reduction-semantics-contract.md).

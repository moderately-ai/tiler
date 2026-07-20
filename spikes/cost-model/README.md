---
schema: "tiler-doc/v1"
id: "tiler.spike.cost-model"
kind: "experiment"
title: "Bootstrap cost-model experiment"
topics: ["cost-model", "optimizer"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model"]
supports: ["tiler.research.cost-model.bootstrap-cost-model"]
entrypoints: ["spikes/cost-model/bootstrap_model.py"]
last_verified: "2026-07-20"
ticket: "cost-model-bootstrap"
---

# Bootstrap cost-model experiment

This dependency-free model checks that hard feasibility is evaluated before a
transparent interval-valued cost estimate and that the estimate exposes its
component terms.

Run from the repository root:

```sh
python3 spikes/cost-model/bootstrap_model.py
```

It is an executable contract model, not calibrated device-performance evidence.

---
schema: "tiler-doc/v1"
id: "tiler.spike.reference"
kind: "experiment"
title: "Normative reference-evaluator experiment"
topics: ["reference", "semantics", "correctness"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model"]
supports: ["tiler.research.reference.normative-reference-slice"]
entrypoints: ["spikes/reference/reference_evaluator.py"]
last_verified: "2026-07-20"
ticket: "reference-evaluator-slice"
---

# Normative reference-evaluator experiment

This bit-oriented Python slice preserves observable f32-to-f16 materialization,
broadcasting, reshape semantics, multiple outputs, and stable shape errors.

Run from the repository root:

```sh
python3 spikes/reference/reference_evaluator.py
```

It is a deliberately small semantic oracle, not a complete dtype or operation
implementation.

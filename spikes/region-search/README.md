---
schema: "tiler-doc/v1"
id: "tiler.spike.region-search"
kind: "experiment"
title: "Exhaustive fusion-region oracle experiment"
topics: ["fusion", "search", "optimizer"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["exhaustive-finite", "executable-model"]
supports: ["tiler.research.region-search.exhaustive-region-oracle"]
entrypoints: ["spikes/region-search/exhaustive_oracle.py"]
last_verified: "2026-07-20"
ticket: "region-search-oracle"
---

# Exhaustive fusion-region oracle experiment

The Python oracle enumerates legal connected fusion regions and complete
program covers for bounded tiny DAGs, retaining legality and rejection reasons.

Run from the repository root:

```sh
python3 spikes/region-search/exhaustive_oracle.py
```

Its exhaustive claim applies only to the finite graph bounds and legality
language implemented by the harness.

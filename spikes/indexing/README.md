---
schema: "tiler-doc/v1"
id: "tiler.spike.indexing"
kind: "experiment"
title: "Index and access-model experiment"
topics: ["indexing", "access", "storage"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model"]
supports: ["tiler.research.indexing.index-access-model"]
entrypoints: ["spikes/indexing/index-access-model/Cargo.toml"]
last_verified: "2026-07-20"
ticket: "index-access-model"
---

# Index and access-model experiment

This Rust model exercises canonical logical index expressions, access
verification, storage-view composition, and guarded narrow physical paths.

Run from the repository root:

```sh
cargo test --manifest-path spikes/indexing/index-access-model/Cargo.toml
```

The nested README documents the individual positive and negative cases. The
model does not implement a production symbolic solver or target backend.

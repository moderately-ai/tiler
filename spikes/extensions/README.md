---
schema: "tiler-doc/v1"
id: "tiler.spike.extensions"
kind: "experiment"
title: "Operation-extension experiments"
topics: ["extensions", "proc-macro", "rust"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.extensions.operation-extension-surface", "tiler.research.extensions.operation-extension-api", "tiler.research.extensions.proc-macro-extension-visibility"]
entrypoints: ["spikes/extensions/operation-api/Cargo.toml", "spikes/extensions/proc-macro-visibility/run.sh"]
last_verified: "2026-07-20"
ticket: "operation-extension-surface"
---

# Operation-extension experiments

The `operation-api` crate compile-checks the proposed capability boundary. The
`proc-macro-visibility` workspace demonstrates which providers a stable proc
macro can observe across host and consumer crate boundaries.

Run from the repository root:

```sh
cargo test --manifest-path spikes/extensions/operation-api/Cargo.toml
spikes/extensions/proc-macro-visibility/run.sh
```

The API names remain experimental. The visibility result is bounded to the
recorded Rust/Cargo compilation model and does not establish a plugin ABI.

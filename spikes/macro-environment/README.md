---
schema: "tiler-doc/v1"
id: "tiler.spike.macro-environment"
kind: "experiment"
title: "Proc-macro environment and artifact-family spikes"
topics: ["proc-macros", "cargo", "cross-compilation"]
experiment_status: "reproducible"
implementation_status: "spike-only"
supports: ["tiler.research.macro-environment.build-environment"]
entrypoints: ["run.sh", "run-target.sh", "run-family-cfg.sh"]
last_verified: "2026-07-20"
ticket: "macro-build-environment"
---

# Proc-macro environment and artifact-family spikes

Run the isolated fixture for native freshness, an installed explicit Rust
target, and generated consumer-family `cfg` behavior:

```sh
spikes/macro-environment/run.sh
spikes/macro-environment/run-target.sh
spikes/macro-environment/run-family-cfg.sh
```

The target probe rejects unavailable targets rather than installing them. The
harness does not measure rust-analyzer. See the [research report](../../docs/research/macro-environment/proc-macro-build-environment.md).

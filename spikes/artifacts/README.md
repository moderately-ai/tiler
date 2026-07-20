---
schema: "tiler-doc/v1"
id: "tiler.spike.artifacts"
kind: "experiment"
title: "Artifact envelope spike"
topics: ["artifacts", "abi", "validation"]
experiment_status: "reproducible"
implementation_status: "spike-only"
supports: ["tiler.research.artifacts.target-neutral-envelope"]
entrypoints: ["artifact_envelope.rs"]
last_verified: "2026-07-20"
ticket: "artifact-envelope-model"
---

# Artifact envelope spike

This dependency-free model checks bounded framing, canonical identity,
cross-references, section digests, and staged neutral/backend validation.

```sh
rustc --edition 2021 --test spikes/artifacts/artifact_envelope.rs -o /tmp/tiler-artifact-envelope-spike
/tmp/tiler-artifact-envelope-spike
```

It does not choose a production codec, digest algorithm, or compatibility
policy. See the [research result](../../docs/research/artifacts/target-neutral-artifact-envelope.md).

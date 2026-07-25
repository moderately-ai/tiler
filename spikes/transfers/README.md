---
schema: "tiler-doc/v1"
id: "tiler.spike.transfers"
kind: "experiment"
title: "Transfer synchronization and lifetime model"
topics: ["transfers", "synchronization", "resource-lifetime"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model"]
supports: ["tiler.research.transfers.synchronization-lifetime"]
entrypoints: ["spikes/transfers/transfer_contract.rs"]
last_verified: "2026-07-25"
ticket: "transfer-synchronization-and-resource-lifetime-contract"
---

# Transfer synchronization and lifetime model

The verifier tests endpoint identity, mechanism-specific feasibility,
dependency/completion distinctions, hazards, cancellation, and retention. Twenty-one tests pass.

`Recompute` is the one mechanism with no source endpoint, so it is checked
against its operands and against a stated four-condition value-preservation
record rather than against a version it moves. The record's conditions are
stated rather than derived here; the report's spike section says what that
bounds.

```sh
rustc --edition 2021 --test spikes/transfers/transfer_contract.rs -o /tmp/tiler-transfer-tests
/tmp/tiler-transfer-tests
```

It does not call Metal/CUDA or measure bandwidth. See the
[transfer report](../../docs/research/transfers/transfer-synchronization-and-resource-lifetime.md).

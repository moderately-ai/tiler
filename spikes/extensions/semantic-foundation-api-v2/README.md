---
schema: "tiler-doc/v1"
id: "tiler.spike.extensions.semantic-foundation-api-v2"
kind: "experiment"
title: "Semantic foundation API v2 compile-checking spike"
topics: ["extensions", "semantics", "rust", "reference"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model"]
supports: ["tiler.research.extensions.semantic-foundation-api-v2"]
entrypoints: ["spikes/extensions/semantic-foundation-api-v2/consumer/src/main.rs"]
last_verified: "2026-07-21"
ticket: "prototype-semantic-foundation-api-v2"
---

# Semantic foundation API v2 compile-checking spike

This isolated Rust workspace checks the corrected dependency and authority
shape before production APIs change:

```text
consumer -> external -> reference -> ir
                    \-------------> ir
```

It demonstrates semantic definitions without mandatory markers, optional
marker binding, on-demand parameterized and encoded type validation, generic
typed and checked-erased inputs, one checked operation-admission path, typed
facades, and reference capabilities outside the IR crate.

Run:

```sh
python3 spikes/extensions/run.py --suite semantic-foundation
```

The shared runner applies an overall timeout and records the exact source,
toolchain, commands, exit status, and output in its ignored local trace.

The code intentionally omits production canonical codecs, diagnostics, owner
checks, and numerical implementations. It proves dependency and API shape,
not completeness or compatibility.

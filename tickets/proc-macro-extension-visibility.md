---
id: proc-macro-extension-visibility
title: Spike proc-macro visibility of operation extensions
status: todo
priority: p1
dependencies: [operation-extension-surface]
related: []
scopes: [research/extensions]
shared_scopes: []
paths: []
tags: [tiler-research, spike, macro, extensions]
---
Test whether the selected public operation-extension model is actually visible and deterministic during stable procedural macro expansion across crate boundaries. Probe trait metadata, registration, linking, host-versus-target compilation, and reproducibility constraints.

Record executable experiments, toolchain versions, results, and the impact on the API. A negative result must identify the smallest viable restriction rather than silently moving work to runtime.

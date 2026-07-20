---
id: proc-macro-extension-visibility
title: Spike proc-macro visibility of operation extensions
status: done
priority: p1
dependencies: [operation-extension-surface]
related: []
scopes: [research/extensions, contracts/core, contracts/integrations]
shared_scopes: []
paths: []
tags: [tiler-research, spike, macro, extensions]
---
Test whether the selected public operation-extension model is actually visible and deterministic during stable procedural macro expansion across crate boundaries. Probe trait metadata, registration, linking, host-versus-target compilation, and reproducibility constraints.

Record executable experiments, toolchain versions, results, and the impact on the API. A negative result must identify the smallest viable restriction rather than silently moving work to runtime.

## Outcome

- Research: [proc-macro extension visibility](../docs/research/extensions/proc-macro-extension-visibility.md)
- Experiment: [operation-extension experiments](../spikes/extensions/README.md)
- Adopted decision: [ADR 0045](../docs/decisions/0045-bound-proc-macro-providers-to-host-dependencies.md)
- Result: confirmed that inline macros can use host-linked providers and canonical invocation data, but cannot discover arbitrary consumer-local Rust implementations.

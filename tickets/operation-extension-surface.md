---
id: operation-extension-surface
title: Design the public experimental operation extension surface
status: done
priority: p0
dependencies: [semantic-graph-contract]
related: []
scopes: [research/extensions, contracts/foundation]
shared_scopes: []
paths: []
tags: [tiler-research, foundation, research, spike]
---
Compare a closed built-in operation set, trusted explicit registration, and other Rust extension mechanisms for semantic operations. Define the minimum traits and invariants needed for shape inference, validation, canonical identity, reference evaluation, numerical policy, access lowering, and explain output.

Produce a small API sketch or throwaway compile-checking spike, plus failure cases for collisions, nondeterminism, opaque attributes, missing capabilities, and provider versioning. Public and experimental is acceptable; semantic ambiguity is not.

## Outcome

- Research: [extension surface](../docs/research/extensions/operation-extension-surface.md) and [API sketch](../docs/research/extensions/operation-extension-api.md)
- Experiment: [operation-extension experiments](../spikes/extensions/README.md)
- Adopted decision: [ADR 0044](../docs/decisions/0044-use-explicit-capability-operation-registry.md)
- Result: selected a frozen explicit registry with one semantic authority per operation key and separately versioned optional capability providers.

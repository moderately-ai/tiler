---
id: prototype-runtime-artifact-validation
title: Implement runtime artifact validation
status: todo
priority: p0
dependencies: [prototype-neutral-artifact-codec]
related: []
scopes: [implementation/runtime, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, artifact]
---
Implement runtime-owned device-free decoding, integrity/program/ABI validation, checked expression evaluation, and typed compatibility classification. The runtime path must not import semantic IR, optimizer state, backend internals, or proof-sidecar semantics.

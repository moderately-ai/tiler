---
id: prototype-runtime-artifact-validation
title: Implement runtime artifact validation
status: in-progress
priority: p0
dependencies: [prototype-neutral-artifact-codec]
related: []
scopes: [implementation/runtime, implementation/artifact, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, runtime, artifact]
claimed_from: todo
assignee: agent-runtime
lease_expires_at: 1784997110
---
Implement runtime-owned device-free decoding, integrity/program/ABI validation, checked expression evaluation, and typed compatibility classification. The runtime path must not import semantic IR, optimizer state, backend internals, or proof-sidecar semantics.

If the owning production crate is absent, this ticket owns its atomic workspace admission and lockfile update. After that crate exists, replace any temporary prototype entry in `[scope_crates]` with the real package owner; do not leave reverse-dependency expansion attached to the prototype.

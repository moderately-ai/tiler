---
id: prototype-workspace-scaffold
title: Scaffold the unstable prototype workspace
status: done
priority: p0
dependencies: [prototype-foundation-contract]
related: []
scopes: [implementation/workspace]
shared_scopes: [project/tickets, contracts/navigation, implementation/ir, implementation/artifact, implementation/compiler, implementation/metal, implementation/runtime]
paths: []
tags: [implementation, prototype, rust]
---
Create the Rust 2024 workspace at MSRV 1.89 with the four library packages and two publish=false proof packages from ADR 0056. Add dependency-direction checks, minimum-version CI, formatting/lint/test commands, and no functional compiler behavior beyond compile-checking package boundaries.

## Outcome

- Created the four library and two non-published proof packages with the exact
  accepted dependency DAG and no tensor/compiler behavior.
- Declared edition 2024 and `rust-version = "1.89"` uniformly.
- Added a standard-library-only workspace graph/manifest checker.
- Added CI on Rust 1.89.0 and stable for graph, formatting, check, and tests.
- Kept existing research spike workspaces explicitly outside the prototype
  workspace.

## Supersession note

ADR 0067 subsequently superseded the stable-only Rust 1.89 toolchain policy.
The scaffold outcome above remains historical evidence; the retained nightly
conformance spike owns migration of the workspace pin and CI.

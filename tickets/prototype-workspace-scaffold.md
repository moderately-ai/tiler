---
id: prototype-workspace-scaffold
title: Scaffold the unstable prototype workspace
status: todo
priority: p0
dependencies: [prototype-foundation-contract]
related: []
scopes: [implementation/workspace]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [implementation, prototype, rust]
---
Create the Rust 2024 workspace at MSRV 1.89 with the four library packages and two publish=false proof packages from ADR 0056. Add dependency-direction checks, minimum-version CI, formatting/lint/test commands, and no functional compiler behavior beyond compile-checking package boundaries.

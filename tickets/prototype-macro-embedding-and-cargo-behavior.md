---
id: prototype-macro-embedding-and-cargo-behavior
title: Measure macro embedding and Cargo behavior
status: todo
priority: p1
dependencies: [prototype-inline-proc-macro-frontend, implement-the-expansion-cache-protocol, compose-the-complete-expansion-cache-subject, prototype-artifact-family-delivery, prototype-metal-aot-slice]
related: [repair-macro-and-embedding-harness-integrity]
scopes: [implementation/frontend, research/embedding, research/macro-environment]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, measurement, proc-macro, inline-dx]
---
Prove direct byte-literal embedding is self-contained and cache deletion cannot break expanded code. Measure Cargo/rust-analyzer cold and warm behavior, edits and toolchain changes, repeated/unique artifacts across crates, and bounded sizes with exact environments and explicit diagnostic/size gates.

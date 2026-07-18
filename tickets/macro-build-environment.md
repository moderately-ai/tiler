---
id: macro-build-environment
title: Measure native and cross-target proc-macro environments
status: todo
priority: p1
dependencies: []
related: []
scopes: [research/macro-environment]
shared_scopes: []
paths: []
tags: [tiler-research, spike, macro, measurement]
---
Probe stable proc-macro expansion under native and cross-target Cargo builds, rust-analyzer cold and warm analysis, unrelated edits, macro-crate edits, cache deletion, and toolchain changes. Inventory only environment and target inputs that are actually observable.

Deliver reproducible fixtures and traces, an explicit contract for when rebuild is required after Xcode changes, and options for selecting Apple artifact families without a build script, source scan, registry, or prepare step.

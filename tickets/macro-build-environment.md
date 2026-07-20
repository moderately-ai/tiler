---
id: macro-build-environment
title: Measure native and cross-target proc-macro environments
status: done
priority: p1
dependencies: []
related: []
scopes: [research/macro-environment, contracts/core, contracts/integrations]
shared_scopes: []
paths: []
tags: [tiler-research, spike, macro, measurement]
---
Probe stable proc-macro expansion under native and cross-target Cargo builds, rust-analyzer cold and warm analysis, unrelated edits, macro-crate edits, cache deletion, and toolchain changes. Inventory only environment and target inputs that are actually observable.

Deliver reproducible fixtures and traces, an explicit contract for when rebuild is required after Xcode changes, and options for selecting Apple artifact families without a build script, source scan, registry, or prepare step.

Completed with reproducible native and explicit-target fixtures, accepted ADR
0049, and integration-contract updates. Native and explicit `--target`
measurement found no target/build-script variables in the proc-macro process;
freshness probes cover consumer edits, macro edits, cache deletion, toolchain
fingerprint changes, and test compilation contexts. rust-analyzer and a truly
different installed Rust target were unavailable without mutating the host, so
those performance measurements are explicitly retained as unmeasured while the
contract fails closed and does not depend on either environment.

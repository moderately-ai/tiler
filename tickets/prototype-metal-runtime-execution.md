---
id: prototype-metal-runtime-execution
title: Implement Metal runtime execution mechanics
status: todo
priority: p0
dependencies: [prototype-metal-runtime-preflight, prototype-runtime-routing-commit]
related: []
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, runtime, metal, execution]
---
Implement bounded allocation, ABI binding, checked dispatch, asynchronous resource retention through final device use, submission, exact terminal-status validation, and readback. Inject post-commit failures and prove no fallback occurs after commit.

## Blocked on a dependency decision, 2026-07-25

The offline half is complete: `cargo run -p tiler-prototype-compile` produces a real 3,667-byte `metallib` for `air64-apple-macos13.0` from a semantic program, through the public compiler boundary, MSL emission, and `xcrun`. Nothing more is needed before dispatch except the ability to talk to a device.

**Fact — the workspace has no Metal bindings and no path to one.** `[workspace.dependencies]` names exactly four external crates: `num-bigint`, `num-integer`, `num-traits`, and `trybuild`. `tiler-metal-aot` is deliberately dependency-free because it shells out to `xcrun`; that works for compilation and cannot work for execution. Metal exposes no C API — `MTLDevice`, `MTLCommandQueue`, `MTLLibrary`, and `MTLComputeCommandEncoder` are Objective-C only — so a Rust process cannot create a device without either an Objective-C binding crate or a compiled shim.

**The decision this ticket cannot avoid.** Which binding Tiler depends on is durable: it fixes the runtime crate's shape, its unsafe-code posture (the workspace currently forbids `unsafe`), its transitive dependency set, and its portability story. The candidates are an `objc2`-family binding, the older `metal-rs`, or a hand-written Objective-C shim built by a `cc`-driven `build.rs`. Each has a different answer for how `MTLCommandBuffer` completion and error checking are surfaced, which `docs/backends/metal.md` and the runtime contract already have opinions about.

**Do not resolve it by picking the most convenient crate.** The workspace's four-dependency discipline and its `unsafe`-forbidden lint are deliberate, and every Metal binding requires relaxing at least one. That relaxation is the decision, not an implementation detail of it.

`prototypes/serial-sum-run` already exists as a workspace member mapped to `implementation/runtime`, so no new crate admission is needed to host the first dispatch — only the binding.

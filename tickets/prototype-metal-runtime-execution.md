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

**Verified 2026-07-25 — the blocker is the `unsafe` prohibition, not the dependency.** Adding a crate to `prototypes/serial-sum-run` is not an ADR 0075 always-ask category: it is not a new publicly reachable namespace, not a new public trait, not a breaking signature change, and not a `pub(crate)` promotion. `cargo add --dry-run` resolves `metal v0.33.0` cleanly, so the dependency itself is available and admissible.

What blocks it is `[workspace.lints.rust] unsafe_code = "forbid"` in the root `Cargo.toml`. `forbid` cannot be relaxed by an `allow` in the crate, so it binds every workspace member including the prototypes. With `metal-rs`, device creation, library loading from `metallib` bytes, pipeline construction, encoder setup, and dispatch are all safe calls — but getting input bytes *into* an `MTLBuffer` is not: `Device::new_buffer_with_data` takes a `*const c_void` and `MTLBuffer::contents` returns a `*mut c_void`, so writing operands and reading results back both require an `unsafe` block in our code. There is no safe path to a buffer's contents in that binding.

`AGENTS.md` states the boundary directly: "unsafe code remains forbidden unless an accepted decision changes that boundary." So first dispatch requires an accepted decision, and that decision is Tom's.

**The shape of the decision, stated so it can be made once.** The narrow form is to permit `unsafe` only where a backend binding requires it — a per-crate `unsafe_code = "allow"` on the runtime prototype and, later, the runtime crate — leaving `forbid` workspace-wide everywhere else. The broad form relaxes the workspace default. The narrow form keeps the property the prohibition buys for the compiler, IR, artifact, and reference crates, where it is genuinely load-bearing, and confines the relaxation to the one layer that must talk to an Objective-C API. Whichever is chosen, the runtime contract's existing requirements — exact command-buffer terminal success before host validation readback, and no fallback after allocation or partial encoding — are what the `unsafe` region must be structured around, not an afterthought to it.

**The decision this ticket cannot avoid.** Which binding Tiler depends on is durable: it fixes the runtime crate's shape, its unsafe-code posture (the workspace currently forbids `unsafe`), its transitive dependency set, and its portability story. The candidates are an `objc2`-family binding, the older `metal-rs`, or a hand-written Objective-C shim built by a `cc`-driven `build.rs`. Each has a different answer for how `MTLCommandBuffer` completion and error checking are surfaced, which `docs/backends/metal.md` and the runtime contract already have opinions about.

**Do not resolve it by picking the most convenient crate.** The workspace's four-dependency discipline and its `unsafe`-forbidden lint are deliberate, and every Metal binding requires relaxing at least one. That relaxation is the decision, not an implementation detail of it.

`prototypes/serial-sum-run` already exists as a workspace member mapped to `implementation/runtime`, so no new crate admission is needed to host the first dispatch — only the binding.

---
id: distinguish-the-five-compile-failure-classes
title: Distinguish ADR 0069's five compile failure classes
status: todo
priority: p2
dependencies: []
related: []
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [compiler-api, diagnostics]
---
`CompileFailureClass` has four variants and ADR 0069 requires five distinguishable classes.

## Fact

ADR 0069 states the compiler's "failure classes distinguish at least invalid requests, valid programs lacking a required compilation capability, intrinsically or target-infeasible plans, exhausted bounded search, and failures of compiler-produced IR verification" — five.

`crates/tiler-compiler/src/session.rs` declares four: `Unsupported { rule }`, `NoFeasiblePlan`, `BudgetExhausted`, `InvalidCompilerOutput`. The first two of ADR 0069's list are folded into `Unsupported`, distinguished only by a `&'static str`.

## Fact — the collapse is deliberate and reasoned, not an oversight

`class_of` in the same file carries the argument: "Both are statements about the request rather than about Tiler, and both carry the refusing check's own key, so they classify the same way; the internal distinction between a malformed request and an unsupported capability is preserved in the explain trace." The internal `pipeline::CompileError` does keep `InvalidRequest` and `UnsupportedCapability` apart, so no information is lost — it is merged on the way out.

## Inference — the reasoning is sound about information and wrong about class

Two things pull the other way, and both come from this crate's own conventions.

`CompileFailureClass`'s own doc says the enum exists so "a caller branches on the boundary that refused instead of matching on text". Distinguishing a malformed request from an uncoverable program currently requires matching `rule` against strings, which is the thing the type exists to avoid. ADR 0074 convention 1 makes the same point generally: a variant carries the structured data a caller needs to react, not a preformatted discriminator.

The two also imply different actions. "Your request is malformed" says fix the request. "Your program is valid and no installed capability compiles it" says install a provider or wait for coverage — and that distinction became reachable in practice on 2026-07-27, when out-of-crate capability installation landed and a caller acquired something to *do* about the second.

## Scope

Split the two, preserving the rule key on both. Supersede the `class_of` comment explicitly rather than deleting it: its claim about the explain trace is true and is not what the class is for.

Check the remaining three against ADR 0069's list at the same time rather than assuming they line up.

## Closes when

`CompileFailureClass` distinguishes ADR 0069's five classes; each is reachable by a test that reaches it from the public surface, or is recorded as unreachable with the reason; the superseded reasoning is preserved at its site; and `make full` passes.

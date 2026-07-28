---
id: prototype-inline-proc-macro-frontend
title: Implement the inline proc-macro frontend proof
status: awaiting-decision
priority: p1
dependencies: [prototype-public-compiler-api, prototype-neutral-artifact-codec]
related: []
scopes: [implementation/frontend, implementation/compiler, implementation/workspace]
shared_scopes: [project/tickets, implementation/cargo-lock]
paths: []
tags: [implementation, frontend, proc-macro, inline-dx]
---
Implement a bounded inline Rust proc-macro frontend that parses one visible tensor region, constructs the public logical program, invokes the ordinary compiler boundary, reports span-aware typed errors, and emits generated Rust. Preserve no consumer build.rs, registry, source scan, prepare step, or runtime JIT. Tom reviews public syntax and ergonomics.

If the owning production crate is absent, this ticket owns its atomic workspace admission and lockfile update. After that crate exists, replace any temporary prototype entry in `[scope_crates]` with the real package owner; do not leave reverse-dependency expansion attached to the prototype.

## Parked 2026-07-27 — awaiting Tom

**The question, atomic:** what does the inline tensor region *look like* at a call site?

The ticket reserves it: "Tom reviews public syntax and ergonomics." That is not a boundary that can be drafted and swapped later in the usual way — the syntax **is** the product surface here, every example in the documentation corpus would be written against it, and a later change rewrites all of them rather than just a signature.

Everything else this ticket asks for is settled and implementable without the answer: parse one visible region, construct the public logical program, invoke the ordinary compiler boundary, report span-aware typed errors, emit generated Rust, and preserve the accepted inline developer experience — no consumer `build.rs`, no registry, no source scan, no prepare step, no runtime JIT, each invocation a self-contained AOT and embedding unit.

**What a decision needs to cover**, so the answer is usable in one pass rather than three: the region delimiter and whether it is expression- or item-position; how a tensor operand names its shape and dtype; whether operations are written as method chains, operators, or an einsum-like string; and how an output is bound back into surrounding Rust.

**No recommendation offered.** Unlike the other parked boundaries, this one is a product-taste decision rather than one the constraints narrow — the elimination that usually removes candidates does not apply, because every reasonable syntax is implementable and the choice is about what reads well to a Rust user.

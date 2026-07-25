---
id: carry-the-dtype-on-the-metal-subnormal-flush-fact
title: Carry the dtype on the Metal subnormal-flush fact
status: todo
priority: p1
dependencies: []
related: [widen-the-apple-numerical-probe-to-a-second-dtype, declare-metal-numerical-honourability, accept-adr-0076-numerical-realizations]
scopes: [implementation/metal, contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, metal, dtypes, correctness]
---
`MetalSubnormalArithmetic::FlushesToZero` is stated once, with no dtype, and `MetalTargetFacts::new` requires a caller to supply it. Every measurement behind it was `f32`, and finding 21 of [the Apple numerical-behaviour record](../docs/research/apple-targets/numerical-behaviour.md) now measures the same hardware **not** flushing in `f16`: a subnormal `half` operand comes back exactly doubled from a witnessed `fmul`, and the same holds for the result direction, for a bare add, and for a surviving `fdiv`, in all three math modes, on both compilation paths and both dispatchable families. The emitted modules are indistinguishable — `air.compile.denorms_disable` is declared for both dtypes — so nothing on the compile side would have caught this.

A single dtype-free declaration is therefore false for one of the two dtypes whichever way it is set. The type has to carry the dtype, or name the dtype its fact applies to and reject a query about any other; what it must not keep doing is answer a question it was not measured for.

**Which direction the error runs, so the fix is not over-scoped.** Reading the `f32` fact for an `f16` kernel says a subnormal is flushed when it is carried exactly. On the plan side that over-rejects — a spurious `MetalNumericalGap::SubnormalFlushInArithmetic`, or a feasibility rejection for a plan that is correct. It becomes a wrong tensor only where a *reference* evaluation flushes to match a device that does not, so the reference-side reading is the one to check first.

ADR 0076's re-verification and its subnormal-flush Measurement are stated without a dtype for the same reason and are consequently under-scoped. No conclusion of ADR 0076 moves: a module declaring `denorms_disable` while delivering preserved subnormals is a *stronger* case for its central argument that numerical honourability must be a stated versioned target fact rather than inferred from a compiled kernel.

**What closes this.** The declared fact distinguishes the dtypes it was measured for, an unmeasured dtype is rejected explicitly rather than defaulted to either behaviour, the measurement comment on the type cites finding 21 with its environment row, and ADR 0076's two dtype-free sentences say which dtype they hold for.

---
id: declare-metal-numerical-honourability
title: Declare Metal numerical honourability as a target profile fact
status: todo
priority: p1
dependencies: [select-numerical-contract-and-compose-feasibility]
related: [draft-target-honourable-numerical-contract-adr, prototype-metal-numerical-realization]
scopes: [implementation/metal, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, metal, numerics]
---
ADR 0076 item 3, on the one target that has a measured unhonourable dimension. This is the ticket that gives the Apple row a positive conformance story for the first time: a flush-tolerant `f32` contract compiles and conforms, a preserving one rejects with a named cause.

## What is implemented today, and why it is not enough

`MetalNumericalGap::SubnormalFlushInArithmetic` records the unhonourable obligation, is written into the generated MSL provenance header, and is enforced by `MetalTranslationUnit::require_declared_realization`, which fails closed with `MetalEmitError::UnrealizableNumericalObligation`. That is correct as far as it reaches and was the honest thing to build at the time. It is insufficient as a durable answer for four reasons, each independently sufficient:

- it is one gap variant that cannot distinguish input flushing from result flushing;
- it names no target-profile identity, so a rejection cannot say who declared the fact;
- emission still succeeds, so a caller that never asks for conformance never sees the rejection;
- nothing above `tiler-metal` can select a contract the target would honour instead, so the only reachable outcomes on Apple are a refused conformance claim or a caller that never asks.

## The work

Express `MetalSubnormalArithmetic` as a per-dimension honourability declaration in the shared form `select-numerical-contract-and-compose-feasibility` establishes, rather than a backend-local target fact — so the compiler can assess it *before* emission rather than discovering it during. Retire `MetalNumericalGap` and `require_declared_realization` in favour of the typed rejection, **or** state precisely why a backend-local conformance step survives alongside the profile declaration. Either is acceptable; leaving both without saying which is authoritative is not.

Keep the measurements recorded on the declaring types. `MetalTargetFacts` already documents its measured basis on the type itself, and that is the pattern to preserve.

## The inference that constrains how you may establish honourability

**Honourability is a stated target fact and must never be a value observed from a probe kernel.** The measurement behind this is worth reading before you design anything: under `-fmetal-math-mode=relaxed`, subnormal operands come back *unchanged* from a `scale 1.0, bias +0.0` kernel, which looks like preservation. It is not. `x * 1.0` folds to a copy under every math mode, the kernel retains exactly one floating-point operation under `safe` (the `+0.0` fadd, unremovable without `nsz`) and zero under `relaxed`, and the surviving `fadd` is what flushes. The same licence that breaks signed zero deletes the operation that would have flushed. So observing preserved subnormals from a compiled kernel is not evidence that the target preserves them — it may be evidence that no arithmetic executed, and the modes where this misleads are exactly the least trustworthy ones.

`MetalTargetFacts::subnormal_arithmetic` already takes the correct approach: a required caller-stated fact with the measurement recorded on the type. Generalize that; do not replace it with anything inferred.

## The contract half

`docs/backends/metal.md` records the strict flag row and states that the compatibility probe "did not observe the numerical behavior these flags request". The re-verified measurement in ADR 0076 closes that gap in one direction and the contract must record it: **the strict row does not deliver subnormal preservation.** `-fmetal-math-mode=safe` emits `air.compile.denorms_disable` alongside `air.compile.fast_math_disable`, under `safe`, `relaxed`, and `fast` alike; no offline flag and no runtime `MTLCompileOptions` setting clears it. Materialization is unaffected — a load-then-store round trip preserves every subnormal — so the limit is a property of arithmetic specifically, and the contract should say that rather than a blanket claim about the target.

ADR 0076 leaves as an open question whether the profile *declaration mechanism* belongs in `docs/backends/metal.md` or in the architecture contract. Recording the measured flag behaviour there is not in question; siting the mechanism is. If you conclude it belongs elsewhere, say so and add the scope rather than writing it where it does not belong.

## A knock-on you will hit immediately

`crates/tiler-metal/src/emit.rs` carries irrefutable `let SubnormalMode::Preserve = mode;` bindings in `realization_requirements` and `record_subnormal_obligation`. Those become compile errors the moment `widen-numerical-vocabulary-and-complete-identity` lands, which is the guard working as designed. Handling the new variant is part of this ticket.

The four golden fixtures can then carry a contract the hardware actually honours instead of one it cannot — check whether their compilation through `golden_compilation` should move to the flush-tolerant contract, and say which contract they are governed under either way.

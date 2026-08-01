---
id: decide-the-unnameable-gpu-enumerator-channel
title: Decide whether tiler-metal owns the unnameable-enumerator channel
status: todo
priority: p3
dependencies: []
related: [close-the-serial-sum-run-gpu-family-probe-table, close-the-metal-gpu-family-out-of-crate-total-map, widen-the-metal-gpu-family-vocabulary-to-apple10]
scopes: [implementation/metal]
shared_scopes: []
paths: []
tags: [api-conventions, metal, adr-0074, decision]
---
## The question, atomically

Should `tiler_metal::applicability::observe_highest_gpu_family` grow a channel for "the caller's binding cannot name this enumerator", or does each such binding keep refusing locally?

This is a public-boundary addition to an accepted-draft surface (ADR 0074 §7), so it is Tom's, and it was deliberately left open by `close-the-serial-sum-run-gpu-family-probe-table` rather than self-accepted.

## What is already true

**Fact.** `observe_highest_gpu_family` takes `impl FnMut(AppleGpuFamilyConstant) -> bool`. `bool` is total: a caller has no way to say "I could not put this question to the device". The vocabulary owns the population and the order; the caller owns the crossing into its own Metal binding, and that crossing can fail.

**Fact — it can fail in exactly one of the two bindings in this workspace.** `objc2-metal` 0.3.2 models `MTLGPUFamily` as `MTLGPUFamily(pub NSInteger)`, so `AppleGpuFamilyConstant::value()` crosses directly and no enumerator is unnameable; `prototypes/candle-metal-adapter/src/adapter.rs::observed_apple_family` is two lines with no failure case. `metal` 0.33.0 models it as a `#[repr(i64)]` Rust enum, `#[non_exhaustive]`, with no `TryFrom` and no constructor from a raw value (`metal-0.33.0/src/device.rs:70-89`), and it stops at `Apple9` while the macOS 26.5 SDK declares `MTLGPUFamilyApple10 = 1010`.

**Fact — the local refusal is implemented and green.** `prototypes/serial-sum-run/src/proof.rs` carries `ProbedGpuFamily`, a `binding_apple_enumerator` partial map joined on Apple's own enumerator value, a compile-time counted-population assertion, and two refusals: the host observation leaves the predicate unstated so the policy answers `MetalHostApplicabilityRefusal::Unobserved { predicate: GpuFamily }`, and the route-requirement adapter answers `LiveDeviceObservation::Unrecognized`. Both were watched failing.

## What the local shape costs, stated rather than dismissed

**Inference.** The local shape's correctness argument is not obvious, and it is the part a second binding would have to rediscover: because the walk is highest-first and stops at the first supported family, one unnameable enumerator invalidates the *whole* walk and not merely its own query — `Highest(lower)` would otherwise be an understatement wearing the shape of a most-specific claim. The runner implements that by capturing a flag in the closure and discarding the observation if it was ever set. A second `metal`-shaped binding that reasoned only about the one query would produce a plausible, green, wrong answer.

**Inference.** A channel in `tiler-metal` would also let the crate *name* the outcome, so explain output could tell "no device answered this" from "the device answered no" without each consumer inventing a type for it. The runner had to invent `ProbedGpuFamily` for exactly that distinction.

## The elimination, so it can be refuted rather than only the conclusion

- **Answer `false` on an unnameable enumerator.** Eliminated on correctness: it reports a question nobody asked as a device that answered no. Watched producing `GpuFamilyMismatch { required: Apple9, observed: NoneNamed }` — the closed defect in new clothes.
- **Make `AppleGpuFamilyConstant` convertible into `metal::MTLGPUFamily` inside `tiler-metal`.** Eliminated on the architectural contract: `tiler-metal` names no Metal runtime type, and this would make the compiler crate depend on one consumer's choice of binding.
- **`unsafe` transmute of the raw value into the binding's enum.** Eliminated: ADR 0079's first condition is unmet because a safe route exists, and the value may not be a valid enumerator at all, which is UB rather than a lint question.

Two candidates survive, which is why this is a question and not a research task:

- **Keep the local refusal (status quo).** No new public surface; the one binding that needs it has it; a `bool` closure stays the simplest thing that could work for the binding shape that cannot fail.
- **Add a fallible channel** — `observe_highest_gpu_family` returning a `Result`, or a closure returning `Option<bool>` with a third `MetalGpuFamilySupport`-adjacent outcome. Every binding of the `metal` shape gets the discard-the-walk semantics from the authority that owns the walk, rather than each reimplementing it.

## Recommendation

**Proposal.** Defer, with a trigger, rather than add the surface now. The generalization has exactly one instance today, and a `Result` return would force `prototypes/candle-metal-adapter` — which structurally cannot produce the failure — to handle an outcome it can never see, which is the kind of obligation ADR 0074's conventions exist to avoid minting speculatively. The trigger is concrete: **a second consumer that binds Metal through a raw-value-less enum**, or the moment `widen-the-metal-gpu-family-vocabulary-to-apple10` lands and the runner's compile-time assertion fires — at which point the discard-the-walk reasoning has to be re-derived by whoever repairs the binding, and that is the evidence that it belongs in the crate.

The counterpoint Tom should weigh against that: the reasoning is subtle *now*, and the cost of the channel is small, so "wait for a second instance" is also how one binding's correct-but-undiscoverable argument becomes two bindings' divergent ones.

## Closes when

Tom accepts one of the two surviving candidates. If the channel is accepted, `tiler-metal` grows it under `implementation/metal`, `prototypes/serial-sum-run` moves onto it and drops `ProbedGpuFamily`, and `prototypes/candle-metal-adapter` is updated to the new signature. If the deferral is accepted, the trigger above is recorded on `widen-the-metal-gpu-family-vocabulary-to-apple10` and this closes.

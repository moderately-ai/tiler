---
id: decide-the-unnameable-gpu-enumerator-channel
title: Add a fallible GPU-enumerator channel when a second binding needs it
status: todo
priority: p3
dependencies: []
related: [close-the-serial-sum-run-gpu-family-probe-table, close-the-metal-gpu-family-out-of-crate-total-map, widen-the-metal-gpu-family-vocabulary-to-apple10]
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [api-conventions, metal, adr-0074, trigger-fired]
---
## The question, atomically

Should `tiler_metal::applicability::observe_highest_gpu_family` grow a channel for "the caller's binding cannot name this enumerator", or does each such binding keep refusing locally?

This is a public-boundary addition to an accepted-draft surface (ADR 0074 §7), so it is Tom's, and it was deliberately left open by `close-the-serial-sum-run-gpu-family-probe-table` rather than self-accepted.

## What is already true

**Fact.** `observe_highest_gpu_family` takes `impl FnMut(AppleGpuFamilyConstant) -> bool`. `bool` is total: a caller has no way to say "I could not put this question to the device". The vocabulary owns the population and the order; the caller owns the crossing into its own Metal binding, and that crossing can fail.

**Fact — three workspace members bind Metal for this observation; two use the raw-value-less shape.** `objc2-metal` 0.3.2 models `MTLGPUFamily` as `MTLGPUFamily(pub NSInteger)`, so `AppleGpuFamilyConstant::value()` crosses directly and no enumerator is unnameable; `prototypes/candle-metal-adapter/src/adapter.rs::observed_apple_family` is two lines with no failure case. `metal` 0.33.0 models it as a `#[repr(i64)]` Rust enum, `#[non_exhaustive]`, with no `TryFrom` and no constructor from a raw value (`metal-0.33.0/src/device.rs` ends at `Apple9 = 1009`), and it stops at `Apple9` while the macOS 26.5 SDK declares `MTLGPUFamilyApple10 = 1010`. That raw-value-less shape is used by **two** independent consumers: `prototypes/serial-sum-run` and `crates/tiler-conformance` (the latter does not depend on the former; both re-derive `binding_apple_enumerator` / `ProbedGpuFamily` / discard-the-walk). **Correction — 2026-08-10.** Earlier text said the failure lived in "exactly one of the two bindings" and treated the generalization as having one instance; that census was accurate when filed and is false as present-tense description after `tiler-conformance` copied the metal probe.

**Fact — the local refusal is implemented and green in both metal-shaped consumers.** `prototypes/serial-sum-run/src/proof.rs` and `crates/tiler-conformance/src/{dispatch,applicability,envelope}.rs` each carry `ProbedGpuFamily`, a `binding_apple_enumerator` partial map joined on Apple's own enumerator value, a compile-time `MetalGpuFamily::COUNT == 5` assertion, and the two refusals: host observation leaves the predicate unstated (`Unobserved` / GpuFamily), and the route-requirement adapter answers `LiveDeviceObservation::Unrecognized`. serial-sum-run's pair was watched failing under `close-the-serial-sum-run-gpu-family-probe-table`; conformance's pair is tested under the same names (`an_unnameable_enumerator_leaves_the_family_predicate_unobserved`, dispatch COUNT / nameability pins).

## What the local shape costs, stated rather than dismissed

**Inference.** The local shape's correctness argument is not obvious, and it is the part a second binding would have to rediscover: because the walk is highest-first and stops at the first supported family, one unnameable enumerator invalidates the *whole* walk and not merely its own query — `Highest(lower)` would otherwise be an understatement wearing the shape of a most-specific claim. Both metal-shaped consumers implement that by capturing a flag in the closure and discarding the observation if it was ever set. A third `metal`-shaped binding that reasoned only about the one query would produce a plausible, green, wrong answer.

**Inference.** A channel in `tiler-metal` would also let the crate *name* the outcome, so explain output could tell "no device answered this" from "the device answered no" without each consumer inventing a type for it. Both metal-shaped consumers had to invent `ProbedGpuFamily` for exactly that distinction.

## The elimination, so it can be refuted rather than only the conclusion

- **Answer `false` on an unnameable enumerator.** Eliminated on correctness: it reports a question nobody asked as a device that answered no. Watched producing `GpuFamilyMismatch { required: Apple9, observed: NoneNamed }` — the closed defect in new clothes.
- **Make `AppleGpuFamilyConstant` convertible into `metal::MTLGPUFamily` inside `tiler-metal`.** Eliminated on the architectural contract: `tiler-metal` names no Metal runtime type, and this would make the compiler crate depend on one consumer's choice of binding.
- **`unsafe` transmute of the raw value into the binding's enum.** Eliminated: ADR 0079's first condition is unmet because a safe route exists, and the value may not be a valid enumerator at all, which is UB rather than a lint question.

Two candidates survive, which is why this is a question and not a research task:

- **Keep the local refusal (status quo).** No new public surface; each binding that needs it reimplements it; a `bool` closure stays the simplest thing that could work for the binding shape that cannot fail.
- **Add a fallible channel** — `observe_highest_gpu_family` returning a `Result`, or a closure returning `Option<bool>` with a third `MetalGpuFamilySupport`-adjacent outcome. Every binding of the `metal` shape gets the discard-the-walk semantics from the authority that owns the walk, rather than each reimplementing it.

## Recommendation

**Proposal (historical — 2026-08-01).** Defer, with a trigger, rather than add the surface then. At filing the generalization had one metal-shaped instance, and a `Result` return would force `prototypes/candle-metal-adapter` — which structurally cannot produce the failure — to handle an outcome it can never see, which is the kind of obligation ADR 0074's conventions exist to avoid minting speculatively. The trigger is concrete: **a second consumer that binds Metal through a raw-value-less enum**, or the moment `widen-the-metal-gpu-family-vocabulary-to-apple10` lands and the runner's compile-time assertion fires — at which point the discard-the-walk reasoning has to be re-derived by whoever repairs the binding, and that is the evidence that it belongs in the crate.

The counterpoint Tom should weigh against that: the reasoning is subtle *now*, and the cost of the channel is small, so "wait for a second instance" is also how one binding's correct-but-undiscoverable argument becomes two bindings' divergent ones. **Correction — 2026-08-10.** That second instance has arrived (`crates/tiler-conformance`); see Trigger check log. The deferral decision remains accepted; this node is the implementation carrier and is `todo` because trigger A has fired.

## Closes when

Tom accepts the exact public channel signature under ADR 0074 §7 (or reaffirms permanent local-only with a new trigger — that would be a new decision, not this deferral contract). If the channel lands, `tiler-metal` grows it under `implementation/metal`; `prototypes/serial-sum-run` and `crates/tiler-conformance` move onto it and drop their private `ProbedGpuFamily` / probe copies; `prototypes/candle-metal-adapter` is updated to the new signature even though it cannot produce Unnameable. Claim-time scopes must add `implementation/runtime`, `implementation/conformance`, and the candle adapter scope as consumer edits require — current board scopes list only `implementation/metal` for the channel land site.

~~If the deferral is accepted, the trigger above is recorded on `widen-the-metal-gpu-family-vocabulary-to-apple10` and this closes.~~ **Correction — 2026-08-10.** That branch is superseded by the 2026-08-09 carrier repair: deferral was accepted, this ticket stays open as the implementation carrier, and widen does not host this ticket's trigger.

## Deferral (2026-08-01)

Tom approved the recommendation: defer. The runner is fail-closed regardless, and the general fallible-probe channel activates on the recorded triggers — a second raw-value-less binding needing the same decision, or `widen-the-metal-gpu-family-vocabulary-to-apple10` firing the build error the counted-population assertion now guarantees.

The decision is therefore complete. This node remains the implementation carrier for the accepted trigger; its old “Decide whether” title and `decision` tag were stale and were repaired on 2026-08-09. **Correction — 2026-08-10.** Trigger A has fired (see log); board status is `todo` rather than pure deferred dormancy. Exact `Result` vs `Option<bool>` / third-outcome shape remains Tom's under ADR 0074 §7 when the channel is implemented.

## Trigger check log

- 2026-08-04 — **not fired.** Both recorded triggers were unmet at that recheck: the workspace then appeared to bind Metal through exactly two crates with only `prototypes/serial-sum-run` (`metal` 0.33.0) on the raw-value-less shape; and [`widen-the-metal-gpu-family-vocabulary-to-apple10`](widen-the-metal-gpu-family-vocabulary-to-apple10.md) recorded its own deferral the same day, so the counted-population assertion had not been made to fire. Recheck command used then: `grep -rn 'objc2-metal\|^metal = \|metal = "' prototypes/*/Cargo.toml` — **structurally blind** to `crates/*/Cargo.toml` (so it could not have seen a later `tiler-conformance` fire).
- 2026-08-10 — **fired** (trigger A). `crates/tiler-conformance` binds Metal through workspace `metal` 0.33.0 and reimplements `binding_apple_enumerator`, `ProbedGpuFamily`, discard-the-walk, `MetalGpuFamily::COUNT == 5`, and Unobserved/Unrecognized refusals independently of serial-sum-run — the second raw-value-less consumer the Recommendation named. Trigger B still **not fired**: [`widen-the-metal-gpu-family-vocabulary-to-apple10`](widen-the-metal-gpu-family-vocabulary-to-apple10.md) remains `deferred`; vocabulary still five variants Apple5–Apple9. Recheck: `rg -n 'binding_apple_enumerator|ProbedGpuFamily' prototypes crates/tiler-conformance`; `rg -n 'metal\.workspace|objc2-metal|^metal' prototypes/*/Cargo.toml crates/*/Cargo.toml`; `rg -n '^status:' tickets/widen-the-metal-gpu-family-vocabulary-to-apple10.md`.

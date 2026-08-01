---
id: close-the-serial-sum-run-gpu-family-probe-table
title: Close the serial-sum-run GPU family probe table
status: in-progress
priority: p2
dependencies: []
related: [close-the-metal-gpu-family-out-of-crate-total-map, widen-the-metal-gpu-family-vocabulary-to-apple10]
scopes: [implementation/runtime]
shared_scopes: []
paths: []
tags: [correctness, api-conventions, metal, adr-0074]
claimed_from: todo
assignee: worker-sum-probe
lease_expires_at: 1785598277
---
## User-visible outcome

`prototypes/serial-sum-run` probes a device for exactly the families `tiler_metal::applicability::MetalGpuFamily::ALL` names, so a family added to that vocabulary cannot leave the proof runner's applicability observation silently one family short.

## Why this is a separate ticket

**Fact.** `close-the-metal-gpu-family-out-of-crate-total-map` closed the same defect at `prototypes/candle-metal-adapter/src/adapter.rs` and named that site as the only one. It was not: `highest_apple_family` in `prototypes/serial-sum-run/src/proof.rs` (lines 703-716 at `cb5d86a`) carries the identical five-element pair table:

```rust
[
    (MTLGPUFamily::Apple9, MetalGpuFamily::Apple9),
    (MTLGPUFamily::Apple8, MetalGpuFamily::Apple8),
    (MTLGPUFamily::Apple7, MetalGpuFamily::Apple7),
    (MTLGPUFamily::Apple6, MetalGpuFamily::Apple6),
    (MTLGPUFamily::Apple5, MetalGpuFamily::Apple5),
]
```

**Fact — it is a different scope.** `prototypes/serial-sum-run/**` maps to `implementation/runtime` in `ticketsplease.toml`, and the closing ticket declared `implementation/metal` and `implementation/candle` only.

**Fact — and it is not the same fix, which is the substantive reason it was not absorbed.** The candle adapter binds Metal through `objc2-metal` 0.3.2, where `MTLGPUFamily(pub NSInteger)` is a public tuple newtype, so `tiler_metal::applicability::AppleGpuFamilyConstant::value()` crosses into it directly and the whole correspondence disappears. This runner binds Metal through `metal` 0.33.0, where `MTLGPUFamily` is a `#[repr(i64)]` **Rust enum** with no `TryFrom` and no safe constructor from a raw value (`metal-0.33.0/src/device.rs:70-89`, `supports_family` at `:1629`). A raw enumerator therefore has to be named back into that enum by hand here, and the crate forbids the transmute that would avoid it (`unsafe_code = "deny"`, one admitted site, and ADR 0079's first condition is not met because a safe route exists).

## Implementation keys

- Drive the probe from `tiler_metal::applicability::observe_highest_gpu_family` so the *population* is the vocabulary's, exactly as `prototypes/candle-metal-adapter/src/adapter.rs::observed_apple_family` now does.
- Decide what the residual `isize -> metal::MTLGPUFamily` step does with an enumerator this binding does not name, and make it loud. `metal` 0.33.0 stops at `Apple9` while the macOS 26.5 SDK declares `MTLGPUFamilyApple10 = 1010`, so the case is reachable rather than theoretical the moment `widen-the-metal-gpu-family-vocabulary-to-apple10` lands. Returning `false` is the defect being closed wearing different clothes: it answers "this device does not support that family" to a question that was never asked.
- Two shapes are worth weighing, and the choice is a public-boundary consequence to state rather than assume. Either the runner refuses locally — it stops calling `observing_gpu_family`, so the policy reports `MetalHostApplicabilityRefusal::Unobserved { predicate: GpuFamily }`, which is precisely the typed outcome that exists for "an adapter did not ask" — or `tiler-metal` grows a fallible probe (`observe_highest_gpu_family` returning `Result`, or a closure returning `Option<bool>`) so every binding of this shape gets the channel rather than each inventing one. The first needs no new public surface; the second is the general answer and is Tom's to accept.
- Whatever is chosen, add the check that can say no and watch it fail: the population this runner probes must be compared against `MetalGpuFamily::ALL` with a **literal** count, and verified by adding a sixth family locally and watching the count assertion reject it.

## Explicit non-goals

- **Do not widen `MetalGpuFamily`.** `widen-the-metal-gpu-family-vocabulary-to-apple10` owns that, and it is a measurement question rather than a transcription.
- **Do not implement the answer surface** proposed in `docs/research/runtime/backend-scoped-route-requirement-answers.md`.

## Closes when

No workspace consumer pairs a `MetalGpuFamily` variant with an Apple enumerator by hand, an enumerator this runner's binding cannot name is refused rather than answered `false`, the counted-population check has been watched failing, and `make full` is green.

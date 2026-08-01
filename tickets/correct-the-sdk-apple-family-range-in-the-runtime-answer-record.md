---
id: correct-the-sdk-apple-family-range-in-the-runtime-answer-record
title: Correct the SDK Apple family range in the runtime answer record
status: in-progress
priority: p2
dependencies: []
related: [close-the-metal-gpu-family-out-of-crate-total-map, widen-the-metal-gpu-family-vocabulary-to-apple10]
scopes: [research/runtime]
shared_scopes: []
paths: []
tags: [documentation, metal, adr-0074]
claimed_from: todo
assignee: worker-sdk-range
lease_expires_at: 1785598736
---
## User-visible outcome

`docs/research/runtime/backend-scoped-route-requirement-answers.md` states the SDK's Apple family range correctly, and its account of `MetalGpuFamily`'s attribute and consumers describes the tree as it is rather than as it was at `6f7caf3`.

## What is stale, found by reading

**Fact — the SDK range is wrong.** The record's b1 evidence paragraph and its measurement-boundary bullet both say `MTLDevice.h` "declares `MTLGPUFamilyApple1 = 1001` through `MTLGPUFamilyApple9 = 1009` (`...MTLDevice.h:233-241`)". The same header in the same installed macOS 26.5 SDK declares `MTLGPUFamilyApple10 = 1010` on line 242. Reproduce: `grep -n MTLGPUFamilyApple "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h"`. The record's conclusions do not depend on the omission — the `"Apple10" < "Apple9"` elimination is arithmetic on ASCII and is unaffected — but the record explicitly says "whether Apple ships an `MTLGPUFamilyApple10` is unknown here", and it is no longer unknown. `widen-the-metal-gpu-family-vocabulary-to-apple10` owns whether Tiler's vocabulary should follow; this ticket owns only the recorded fact.

**Fact — the attribute claim is closed.** The record states that "both its attribute and its stated reason are wrong at `6f7caf3`". `close-the-metal-gpu-family-out-of-crate-total-map` corrected the stated reason and removed `#[non_exhaustive]` under ADR 0074 convention 5b. The paragraph should record the closure and keep its reasoning rather than continue to assert a live defect.

**Fact — one of the two total maps it cites is gone and the other is not.** The record quotes `prototypes/candle-metal-adapter/src/adapter.rs:584-590`'s pair table; that site now calls `tiler_metal::applicability::observe_highest_gpu_family` and names no family. The record does not mention `prototypes/serial-sum-run/src/proof.rs:703-716`, which carries the identical table and still does; `close-the-serial-sum-run-gpu-family-probe-table` owns it. The "Fact — a working implementation of the surviving design already exists in-workspace" paragraph cites `adapter.rs:582-596` and `713-742` by line and both moved.

**Inference — the design's own conclusion is strengthened, not weakened.** The record derives that the observation should cross as a raw Apple constant supplied by `tiler-metal` rather than by publishing `MetalGpuFamily`, and item b1's proposed sketch (`AppleGpuFamilyConstant`, `observe_highest_gpu_family`) is what landed, half of it, for exactly the reason the record gives. What is now measured rather than proposed is worth marking as such: the constant crosses as `isize`, because `MTLDevice.h` declares the enumeration as `NS_ENUM(NSInteger, MTLGPUFamily)` and `objc2-metal` models `NSInteger` as `isize`; the record's sketch says `i64`, which compiles against `metal` 0.33.0 and forces a fallible conversion at the `objc2-metal` call site.

## Implementation keys

- Correct the SDK range and the measurement-boundary bullet, keeping the reproduction command so the next reader re-runs rather than trusts.
- Move the `MetalGpuFamily` attribute paragraph from a live defect to a recorded closure, citing the ticket that closed it, and preserve the 5b/5c reasoning that made it a defect.
- Re-point the line citations into `prototypes/candle-metal-adapter/src/adapter.rs`, and add `prototypes/serial-sum-run` as the remaining out-of-crate total map with its owning ticket.
- Mark the b1 sketch's `i64` as superseded by the landed `isize` and say why.

## Explicit non-goals

- **Do not accept or implement the answer surface.** The record stays a proposal; this is a factual correction to it.
- **Do not widen `MetalGpuFamily`.**

## Closes when

Every fact above is either corrected or shown to be already right by a reproduction a reader can run in one line, and no sentence in the record asserts a defect that has been closed.

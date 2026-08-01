---
id: close-the-metal-gpu-family-out-of-crate-total-map
title: Close the MetalGpuFamily out-of-crate total map
status: in-progress
priority: p2
dependencies: []
related: [design-the-adapter-owned-route-requirement-answer-channel]
scopes: [implementation/metal, implementation/candle]
shared_scopes: [project/tickets]
paths: []
tags: [correctness, api-conventions, metal, adr-0074]
claimed_from: todo
assignee: worker-gpu-map
lease_expires_at: 1785596471
---
## User-visible outcome

Adding a family to `tiler_metal::applicability::MetalGpuFamily` is a build error at every site that has to know about it, instead of compiling cleanly and leaving a device silently unprobed for the new family.

## The defect, found by reading

**Fact.** `MetalGpuFamily`'s declaration (`crates/tiler-metal/src/applicability.rs:116-120`) is `#[non_exhaustive]` and states the reason as "a later Apple family lands additively and **no consumer outside this crate classifies it by exhaustive match**".

**Fact — one does.** `prototypes/candle-metal-adapter/src/adapter.rs:584-590` maps every variant onto its Apple counterpart as a five-element table:

```rust
[
    (MTLGPUFamily::Apple9, MetalGpuFamily::Apple9),
    (MTLGPUFamily::Apple8, MetalGpuFamily::Apple8),
    (MTLGPUFamily::Apple7, MetalGpuFamily::Apple7),
    (MTLGPUFamily::Apple6, MetalGpuFamily::Apple6),
    (MTLGPUFamily::Apple5, MetalGpuFamily::Apple5),
]
```

**Inference — this is an [ADR 0074](../docs/decisions/0074-use-explicit-public-api-conventions.md) convention 5b site by that record's own test.** A total map is "a match in which every variant must contribute its own correct result and no wildcard value is derivable from the variant it would cover", and the 2026-07-24 amendment extends the clause to total maps "whose arms are all implied rather than written". There is no Apple constant a wildcard could return for an unrecognized Tiler family. 5b's rule is that such a vocabulary is not `#[non_exhaustive]`.

**Inference — and written as a table rather than a match, the attribute would not have helped anyway, which is the sharper half.** Adding `Apple10` to `MetalGpuFamily` and to `MetalGpuFamily::ALL` compiles cleanly at that site. The device is never probed for it, `observed_apple_family` returns a lower family or `NoneNamed`, `evaluate_metal_host_applicability` reports a `GpuFamilyMismatch` naming the wrong observed value, and any future route requiring Apple10 is refused on a device that satisfies it. That is convention 5c's named failure mode — "fail-closed but silently incomplete, which is the harder failure to notice" — reached without the attribute being involved.

**Note — the neighbouring codec is already correct and is the pattern to copy.** `gpu_family_from_payload` (`adapter.rs:603-607`) scans `MetalGpuFamily::ALL` rather than a second written table, so it is complete by construction. The device probe is the one site that is not.

## Implementation keys

- Decide and record which of the two closures applies, because they are different fixes and only one is needed: either the probe table becomes complete by construction the way `gpu_family_from_payload` already is — driven by `MetalGpuFamily::ALL` with the Apple constant obtained from the vocabulary rather than paired by hand — or `MetalGpuFamily` drops `#[non_exhaustive]` per 5b so the map must be written as a wildcard-free match. Prefer the first: it removes the out-of-crate map instead of making its incompleteness a compile error, and it is what [Backend-scoped route-requirement answers](../docs/research/runtime/backend-scoped-route-requirement-answers.md) derives independently for the answer surface.
- If the first is taken, the Apple constant must come from `tiler-metal`. `MTLDevice.h` in the macOS 26.5 SDK declares `MTLGPUFamilyApple1 = 1001` through `MTLGPUFamilyApple9 = 1009` (`$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h:233-241`); re-read it rather than trusting this line. `tiler-metal` must not name a Metal runtime type, so the value crosses as a raw constant the caller passes to its own binding — which also makes it work for both `metal` 0.33.0 (a Rust enum) and `objc2-metal` (a newtype over `NSInteger`).
- Correct the doc comment on `MetalGpuFamily` either way. Its current stated reason is false at `6f7caf3` and a reader takes it as fact.
- Add the check that can say no: a test that fails if the probe population and `MetalGpuFamily::ALL` disagree in length or membership. A test that merely passes on the current five proves nothing — verify it by adding a sixth variant locally, watching it fail, and reverting.

## Explicit non-goals

- **Do not implement the answer surface** the design record proposes. This ticket closes a live defect; that design is a separate, unaccepted proposal with its own public-boundary items.
- **Do not change `MetalGpuFamilySupport`.** It is correctly exhaustive and its documentation correctly states that out-of-crate consumers map both arms.

## Closes when

A family added to `MetalGpuFamily` cannot be silently unprobed by any workspace consumer, the check that establishes it has been watched failing, the type's doc comment describes what is true, and `make full` is green.

## Graph maintenance

- Independent of the design ticket that found it, and deliberately so: the defect is live at `6f7caf3` whether or not that design is ever accepted.
- If the answer-surface design lands first, this ticket's first closure is a subset of it and should be checked for redundancy rather than done twice.

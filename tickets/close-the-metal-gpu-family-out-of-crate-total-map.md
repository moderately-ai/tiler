---
id: close-the-metal-gpu-family-out-of-crate-total-map
title: Close the MetalGpuFamily out-of-crate total map
status: in-progress
priority: p2
dependencies: []
related: [design-the-adapter-owned-route-requirement-answer-channel, close-the-serial-sum-run-gpu-family-probe-table, widen-the-metal-gpu-family-vocabulary-to-apple10, correct-the-sdk-apple-family-range-in-the-runtime-answer-record]
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

## Outcome

**Both closures were taken, not one.** The first was preferred as this ticket says and is what removes the map; the second turned out to be *forced* by the first's own evidence rather than an alternative to it, because after the candle site was fixed a second out-of-crate total map was still standing and ADR 0074 convention 5b is not conditional on how many there are.

### The first closure: the probe population moved into the vocabulary's crate

`crates/tiler-metal/src/applicability.rs` gained three items. `MetalGpuFamily::apple_constant` is a wildcard-free exhaustive match from each family onto the enumerator `MTLDevice.h` declares for it — the Apple-side authority the ticket asked for, in the crate that defines the vocabulary. `AppleGpuFamilyConstant` carries that enumerator as an opaque value with a private field, so no caller can mint one for a family this vocabulary does not name. `observe_highest_gpu_family` walks `MetalGpuFamily::ALL` in reverse and asks the caller one yes-or-no question per family.

`prototypes/candle-metal-adapter/src/adapter.rs::observed_apple_family` is now two lines and names no family at all: `objc2-metal` models `MTLGPUFamily` as `MTLGPUFamily(pub NSInteger)`, so the enumerator crosses directly with no correspondence written at the call site.

**Placement elimination.** *In the consumer* (the status quo) — eliminated: `prototypes/**` is outside `make lint`, so no lint could reach it, and a pair table has no arm to be missing, so no compile error could either. *In `tiler-metal-aot`* — eliminated: it owns offline compilation provenance, which ADR 0086 excludes from the live-host decision, and `tiler-metal` only development-depends on it. *In `tiler-runtime`* — eliminated: backend-neutral by charter, and an Apple enumerator is a backend fact. *In a new crate* — eliminated: a second authority beside `as_str`'s. *In `tiler-metal`* — chosen: the enum, `ALL`, `as_str` and the policy already live there, both consumers already depend on it, and it is inside both the lint gate and the test gate. It does not make `tiler-metal` name a Metal runtime type; the enumerator crosses as a raw `isize`, which is what `NS_ENUM(NSInteger, MTLGPUFamily)` declares and which suits `objc2-metal`'s newtype and `metal` 0.33.0's `#[repr(i64)]` enum alike.

### The check that can say no, and a stronger one beside it

`MetalGpuFamily::ALL` is declared `[Self; core::mem::variant_count::<Self>()]`, so a family added to the enum and not to the list is an array-length error at the declaration rather than a silently short probe. `crates/tiler-metal/src/lib.rs` gained `#![feature(variant_count)]` for it, which the pinned nightly makes free; every other site that has to know about a family is already an exhaustive match that `rustc` closes on its own, and the list was the one that was not. A const assertion beside `ALL` also rejects a member inserted out of order, comparing Apple's enumerators rather than the derived `Ord` that would have agreed with a misordering for the reason that made it wrong.

`applicability_tests::a_probe_covers_every_named_family_highest_first` records what one probe asks and compares it against `ALL` in length and membership, with the count pinned as the **literal** 5 rather than `COUNT` — a count derived from the thing it counts cannot fail, which the sibling matrix test in the same file already learned.

### Watched failing

- **The baseline defect, reproduced.** At `cb5d86a`, adding `Apple10` to `MetalGpuFamily`, to `ALL`, and to `as_str` left `cargo check -p tiler-prototype-candle -p tiler-metal --all-targets` at exit 0 — the probe table untouched, covering five of six families.
- **The same perturbation after the fix.** Adding the `Apple10` variant alone produces three errors: `E0308` on `ALL`'s length ("expected an array with a size of 6, found one with a size of 5"), and `E0004` at `as_str` and at `apple_constant`.
- **Completing the addition.** With the arms and `ALL` filled in, the candle probe covered the new family with no call-site edit — the recording test observed `[1010, 1009, 1008, 1007]` — and the counted-population assertions failed loudly (`left: 6, right: 5`) so a widening is acknowledged rather than absorbed.
- **The table form, reintroduced.** Replacing the `ALL`-driven walk in `observe_highest_gpu_family` with a hand-written four-family table failed the same test at `left: 4, right: 5`. This is the failure the base tree had no check for at all.

### The `#[non_exhaustive]` verdict, and its boundary consequence

**Removed.** `MetalGpuFamily` is a convention 5b type by that record's own test: a wildcard would have to invent the Apple enumerator that the variant alone determines. The clause is unconditional, and one out-of-crate total map remains at `prototypes/serial-sum-run/src/proof.rs:703-716`. The attribute was also the *cause* of the table form — written as a match, that correspondence is `E0004` across the crate boundary, so its author wrote a table instead, and a table cannot fail closed. Removal is behaviour-neutral at the build, because nothing outside `tiler-metal` matches this type.

**Consequence to state rather than bury:** this changes the type's Rust compatibility contract. A future family is now a source-breaking change for any out-of-crate exhaustive match. ADR 0075 accepts that class of break while no crate is publishable and no external consumer exists, and ADR 0074 convention 5c records the compiler as the cheapest correct growth announcement, but it is a public-boundary change and is flagged for Tom rather than self-accepted — as are the two new public items, `AppleGpuFamilyConstant` and `observe_highest_gpu_family`. Both are inside the module that already declares every public item a reviewed draft (ADR 0074 §7).

### Corrections to this ticket's own premises

- **`MTLGPUFamilyApple10 = 1010` exists**, in the same macOS 26.5 SDK (build `25F70`) this ticket cites. The implementation key's range "`Apple1 = 1001` through `Apple9 = 1009` (`MTLDevice.h:233-241`)" is a bounded window that stops one line before it; line 242 is `MTLGPUFamilyApple10 = 1010`. Re-read as instructed, and reproducible with `grep -n MTLGPUFamilyApple "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h"`. Whether Tiler's vocabulary should follow is a measurement question owned by `widen-the-metal-gpu-family-vocabulary-to-apple10`; the identical claim in `docs/research/runtime/backend-scoped-route-requirement-answers.md` is owned by `correct-the-sdk-apple-family-range-in-the-runtime-answer-record`.
- **The candle adapter was not the only out-of-crate total map.** `prototypes/serial-sum-run/src/proof.rs:703-716` carries the identical five-element table. It is `implementation/runtime`, outside this ticket's declared scopes, and it is genuinely a different fix: `metal` 0.33.0 models `MTLGPUFamily` as a `#[repr(i64)]` Rust enum with no safe constructor from a raw value, so that consumer must name the enumerator back by hand and needs a decision about what to do with one its binding does not know. Filed as `close-the-serial-sum-run-gpu-family-probe-table` rather than absorbed, and this ticket's "any workspace consumer" is therefore supported only once that lands.

### The residual, stated

Nothing mechanically prevents a consumer from writing a family list beside its device call again — it would now have to reach for `apple_constant()` per family to do so, but it would compile. What the fix removes is the need for such a list and the pairing that made one wrong; what it adds is that a widened vocabulary stops the build in three places first. That is a real narrowing, not a closure, and it is why the population lives in the crate rather than being validated at each consumer.

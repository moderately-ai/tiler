---
id: widen-the-metal-gpu-family-vocabulary-to-apple10
title: Decide whether MetalGpuFamily names Apple10
status: deferred
priority: p3
dependencies: []
related: [close-the-metal-gpu-family-out-of-crate-total-map, close-the-serial-sum-run-gpu-family-probe-table, correct-the-sdk-apple-family-range-in-the-runtime-answer-record]
scopes: [implementation/metal]
shared_scopes: [project/tickets]
paths: []
tags: [research, metal, apple-targets]
---
## The question

Should `tiler_metal::applicability::MetalGpuFamily` name `Apple10`, and on what evidence?

## What was found, and where the prior claim came from

**Fact — Apple ships an `MTLGPUFamilyApple10`, in the SDK this project already reads.** `MTLDevice.h` in the installed macOS 26.5 SDK (build `25F70`, `/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.5.sdk`) declares:

```
233:    MTLGPUFamilyApple1  = 1001,
...
241:    MTLGPUFamilyApple9  = 1009,
242:    MTLGPUFamilyApple10 = 1010,
```

Reproduce in one line: `grep -n MTLGPUFamilyApple "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h"`.

**Fact — two records state the range as ending at `Apple9`, and both cite lines `233-241`.** `docs/research/runtime/backend-scoped-route-requirement-answers.md` says "`MTLDevice.h` in the installed macOS 26.5 SDK declares `MTLGPUFamilyApple1 = 1001` through `MTLGPUFamilyApple9 = 1009` (`...MTLDevice.h:233-241`)", and `tickets/close-the-metal-gpu-family-out-of-crate-total-map.md` repeats it. The citation is a bounded window that stops exactly one line before the constant that matters — the failure AGENTS.md names ("a bounded window ... can split the construct being searched for"). Nothing about the reasoning in either record depends on the omission, but the stated fact is wrong and a reader takes it as fact. `correct-the-sdk-apple-family-range-in-the-runtime-answer-record` owns the research-record half.

**Fact — the bindings disagree, which is itself part of the decision.** `objc2-metal` 0.3.2 names `MTLGPUFamilyApple10` (`objc2-metal-0.3.2/src/generated/MTLDevice.rs:238`). `metal` 0.33.0 does **not**: its `#[repr(i64)]` enum stops at `Apple9 = 1009` (`metal-0.33.0/src/device.rs:70-89`). So `prototypes/candle-metal-adapter` could already ask a device about Apple10 and `prototypes/serial-sum-run` could not without naming the raw value itself.

**Fact — that second binding gap is now announced rather than silent, and it will stop this ticket's build.** `close-the-serial-sum-run-gpu-family-probe-table` left a compile-time assertion in `prototypes/serial-sum-run/src/proof.rs` that fails when `MetalGpuFamily::COUNT` leaves `5`, and fails again on a nameability sweep if the literal is merely raised. Adding an `Apple10` variant therefore breaks `cargo check` until that runner's `metal` binding genuinely names the enumerator — the intended coupling, not an obstacle to route around. At runtime the same runner already refuses fail-closed on an enumerator it cannot name (`MetalHostApplicabilityRefusal::Unobserved { predicate: GpuFamily }` for host applicability, `LiveDeviceObservation::Unrecognized` for a family route requirement), so widening degrades nothing silently; it leaves that runner unable to *observe* the family until its binding catches up.

## Why this is not a transcription

`MetalGpuFamily`'s own documentation states that the set "is bounded by what the retained measurements needed". Widening it is therefore a *measurement* question, not a header-reading one, and it has consequences that outlive the edit:

- Every device this project has measured reports `supportsFamily:MTLGPUFamilyApple9` (Apple M4 Max, both retained 2026-07-31 records). Nothing here has observed an Apple10 device, so an `Apple10` variant would be a vocabulary member with no measurement behind it.
- `MetalGpuFamily`'s derived `Ord` is its declaration order and is what the route-requirement comparison uses (`highest >= required`). The lexicographic hazard the runtime answer record already registers — `"Apple10" < "Apple9"` byte-for-byte, because `'1'` precedes `'9'` at the sixth byte — becomes live for any consumer that compares the canonical payload spelling rather than the ordering, so widening turns a recorded future hazard into a present one.
- `MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9` compares the family for **exact** equality, deliberately. Widening the vocabulary does not widen the measured row and must not appear to.

## What would close this

Either an accepted decision to widen — with the ordering and payload-comparison consequences discharged, and `prototypes/serial-sum-run`'s binding gap from `close-the-serial-sum-run-gpu-family-probe-table` resolved first — or a recorded deferral with a stated trigger, the obvious one being the first device this project measures that reports `MTLGPUFamilyApple10`.

Whichever way it goes, the `MetalGpuFamily` doc comment currently points here by name and must end up pointing at the answer.

## Recorded deferral — 2026-08-04

**The elimination leaves one survivor, so this closes as a deferral with a trigger rather than a question.** Widening now fails on three independent grounds: no retained measurement observes any device reporting `MTLGPUFamilyApple10` (every measured device reports `Apple9`), so an `Apple10` variant would breach the vocabulary's own boundedness contract; widening turns the registered `"Apple10" < "Apple9"` lexicographic hazard from a recorded future risk into a live one for any payload-spelling comparator; and `prototypes/serial-sum-run`'s `metal` 0.33.0 binding cannot name the enumerator, so the compile-time coupling left by `close-the-serial-sum-run-gpu-family-probe-table` breaks the build by design. Deferral costs nothing operationally: an unnamed family is refused fail-closed (`MetalHostApplicabilityRefusal::Unobserved`, `LiveDeviceObservation::Unrecognized`), never silently renamed.

**Verification against the current environment, 2026-08-04.** The selected toolchain moved to the Xcode 27.0 beta since this ticket's facts were recorded; the 27.0 SDK's `MTLDevice.h` declares `MTLGPUFamilyApple1 = 1001` through `MTLGPUFamilyApple10 = 1010` at the same lines (`233–242`), and the cited 26.5 SDK header still names `Apple10`. Reproduce: `grep -n MTLGPUFamilyApple "$(xcrun --sdk macosx --show-sdk-path)/System/Library/Frameworks/Metal.framework/Headers/MTLDevice.h"`. The `MetalGpuFamily` doc comment now states the deferral and its trigger directly.

**Reactivation triggers:** a retained measurement of a device reporting `MTLGPUFamilyApple10`, with the `serial-sum-run` binding gap resolved first as a precondition. On reactivation, the ordering and payload-comparison consequences in "Why this is not a transcription" are the work plan.

## Trigger check log

- 2026-08-04 — **not fired**, re-confirmed by the deferred sweep hours after the deferral above was recorded. No retained measurement observes a device reporting `MTLGPUFamilyApple10`, and the `prototypes/serial-sum-run` binding gap is unresolved, so the stated precondition is also unmet.
- 2026-08-09 — **not fired.** `MetalGpuFamily` still ends at `Apple9`; the retained runtime and conformance measurements still report Apple9; and `prototypes/serial-sum-run` still records that `metal` 0.33.0 cannot name Apple10. The SDK constant's existence is already documented, but the measurement and binding prerequisites for widening remain absent.

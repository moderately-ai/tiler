---
id: declare-the-bf16-rows-on-the-authoritative-metal-profile
title: Declare the measured BF16 dispatchability and subnormal rows on the Metal profile
status: in-progress
priority: p1
dependencies: [admit-a-bf16-scalar-arithmetic-subject]
related: [spike-bf16-through-the-second-dtype-seams, construct-and-bind-the-first-authoritative-metal-compile-profile, measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes, decide-per-dtype-dispatchability-as-a-target-capability, measure-apple-numerics-on-physical-ios-device]
scopes: [implementation/build, implementation/metal, contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, dtype, bf16, target-profiles, metal, apple-targets]
claimed_from: todo
assignee: agent-bf16-rows
lease_expires_at: 1785689920
---
## User-visible outcome

The authoritative Metal compile profile carries the measured BF16 facts: dispatchable on the macOS row, explicitly unsupported on the iOS Simulator, and `Unknown` on the unmeasured iOS device. A BF16 program routed at a family that cannot run it is refused **before** the routing commit rather than failing at pipeline creation after it.

## Why both rows land together

**Fact.** `BoundMetalCompileDeclaration` (`crates/tiler-build/src/metal_declaration.rs`) declares `f32` alone: one measured dispatchability row and six honourability rows, all over `ScalarArithmetic::f32()`. Its own ticket states "Do not infer F16 or BF16 from F32, do not claim BF16 on either iOS family" and names this spike's successor as the first non-F32 use of the mechanism.

**Fact.** The profile descriptor is one identity. A dispatchability row and a numerical row both change its bytes, and the golden that pins them is one fixture. Landing them separately would rebaseline the same golden twice and leave an intermediate commit whose profile claims BF16 is dispatchable while saying nothing about its arithmetic — a profile that is worse than either endpoint. They are merged for that reason and no other.

**Measurement, and its exact boundary.** From the retained record `spikes/apple-targets/results/2026-07-31-numerics-covering-xcode26.6-metal32023.883/record.tsv` on an Apple M4 Max, macOS 27.0 build 26A5388g, Metal 32023.883, Xcode 26.6 — findings 24 and 26 of the [Apple numerical behaviour record](../docs/research/apple-targets/numerical-behaviour.md):

- macOS: `device_bfloat_support supported`; BF16 arithmetic **flushes** subnormals, sign-preserving (`8040 → 8000`), across all three math modes, at `-O0` and `-O2`, on both compilation paths, with an execution witness on every verdict. `materialize_bf16` returns all eight operands unchanged, so the flush is a property of arithmetic and not of the buffer round trip.
- iOS Simulator: compiles and links every `bfloat` module, then fails pipeline creation with `XPC_ERROR_CONNECTION_INTERRUPTED`. The arithmetic-free `materialize_bf16` is refused too, so the refusal is about the **format**, not one operation.
- iOS device: never asked. `Unknown`, and it stays `Unknown`.

## Implementation keys

- Extend the existing `BoundMetalCompileDeclaration` and its authority ledger. **Do not** add a parallel constructor or a second backend dtype list; the profile-construction ticket explicitly forbids it.
- The macOS BF16 dispatchability row is `Dispatchable` from a measured source. The iOS-Simulator row is `Unsupported` from a measured source carrying the exact diagnostic. The iOS-device row is **absent**, which is `Unknown` — not `Unsupported`, because nobody asked.
- The BF16 subnormal rows project the measured flush through the same `MetalSubnormalArithmeticFacts` path `f32` uses, which already carries the BF16 slot and already refuses to answer from a neighbouring dtype.
- The profile key must change, because the profile's content changed and the key names its content. Decide whether that is a new key or a version bump and state the reasoning; a descriptor change under an unchanged key is exactly the drift ADR 0043 draws its `ProfileKeyMismatch` against.
- No F16 or F64 row. No iOS-device row. No inference from `f32`.

## Required evidence

- BF16 resolves `Dispatchable` on the macOS profile, `Unsupported` on the simulator profile, and `Unknown` on the device profile, all at `AvailabilityPhase::CompileProfile` — three distinct answers, asserted as a matrix whose shape is checked rather than three independent facts.
- `f32` resolves `Dispatchable` on all three, so no refusal is a dead profile.
- `f16` still resolves `Unknown` on every profile, so a measured BF16 row did not fill a neighbour's omission. The existing test asserting this for `f16` against `f32` is the pattern.
- A strict-subnormal-preserving contract is refused for BF16 on macOS with a named numerical gap, since the measured behaviour flushes.
- The profile descriptor's byte length and identity are recorded before and after.

## Closes when

The three families carry their three distinct BF16 answers, the measured flush is declared and its host/toolchain/family boundary is stated in the ledger rather than generalized, every refusal above is observed failing, the profile key and descriptor movement are recorded, and `docs/dtype-support.md`'s BF16 `Target-family dispatchability` cell moves from `architectural seam` to a stated claim.

## Graph maintenance

- Depends on `admit-a-bf16-scalar-arithmetic-subject`: without a BF16 subject the honourability half is unstatable, and this ticket lands both halves at once.
- `measure-the-apple-subnormal-flush-for-the-remaining-mature-dtypes` owns the measurement and is `done`. **Do not re-measure**; transcribe, and cite the retained record and finding numbers.
- `measure-apple-numerics-on-physical-ios-device` is `deferred` and must not be a dependency — `deferred` never satisfies a dependent. It is the only route to closing the iOS-device `Unknown`, and it stays `related`.
- A differing physical-iOS result would reopen `declare-metal-numerical-honourability`; say so rather than assuming the family agrees.

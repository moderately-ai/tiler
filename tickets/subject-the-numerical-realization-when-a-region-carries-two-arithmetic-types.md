---
id: subject-the-numerical-realization-when-a-region-carries-two-arithmetic-types
title: Subject the numerical realization when a region carries two arithmetic types
status: deferred
priority: p3
dependencies: []
related: [accept-the-bf16-subnormal-resolution-carrier, land-the-bf16-conversion-and-accumulator-adr]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [numerics, identity, deferred]
---
## Deferred: this is arm B, closed against a trigger rather than parked as the eventual carrier

**Do not claim this until the trigger below has fired.** Tom decided on 2026-08-07, on [`accept-the-bf16-subnormal-resolution-carrier`](accept-the-bf16-subnormal-resolution-carrier.md), that `NumericalRealization`'s subnormal fields do **not** acquire a subject, and that the format is derived from the region instead. This ticket exists so the reasoning is recoverable if the premise changes — not as a scheduled migration.

## What would be done, if the trigger fires

Give `NumericalRealization`'s `input_subnormals` and `result_subnormals` a subject, the way `declare_metal_bf16_subnormal_behaviour` declares its rows against `ScalarArithmetic::new(ArithmeticType::Bf16, Bf16::resolved_type())` and `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_BF16` resolves its dimensions per format. **Not alone, though** — see the next section.

## Why it was declined, recorded so it is not re-litigated from the summary

- **The consumer does not exist.** `ReferenceNumericalConformance::from_realization` is the designed bridge from a region's realization to a reference conformance and has **no caller** anywhere in `crates/` or `prototypes/`; every construction site is `strict()` or a test's `new()`. A subject would be carried for nothing to read.
- **`NumericalRealization` is folded into artifact identity** (ADR 0076 item 4), so this is an irreversible identity-domain migration, and the repository's standing position is to keep changes cheap to reverse until verified.
- **It would not have inoculated against the hazard it was chosen for.** See below.

## Trigger for reconsideration

**A `ScalarProgram` variant that carries two arithmetic types in one region.** Today `region_arithmetic_type` (`crates/tiler-ir/src/schedule/model.rs:1333`) is a total function from `ScalarProgram` to exactly one `ArithmeticType`, so a region has one arithmetic type structurally. ADR 0091 keeps BF16/binary32 conversion as two separate typed families and has a mixed-width program spell its conversions as explicit operations, so registering a conversion does **not** by itself fire this.

**When it fires, this is not a two-field change.** `canonical_arithmetic_nan_bits` is one `u32` per region and carries the region's own arithmetic NaN pattern zero-extended; a two-arithmetic region breaks it in the same instant. That field's own doc records the 2026-08-06 decision, taken independently in the schedule and artifact-ABI layers, not to widen it — on the ground that the arithmetic type is already a total function of the region's scalar program, with agreement enforced at `crates/tiler-ir/src/schedule/builder.rs:664`. **All three fields must be decided together**, or the record ends up carrying its subject two different ways.

Recheck: `grep -n "fn region_arithmetic_type" -A 15 crates/tiler-ir/src/schedule/model.rs` — more than one `ArithmeticType` reachable from a single variant is the fired state.

## Trigger check log

- 2026-08-07 — **not fired, and filed in this state.** `region_arithmetic_type` maps every `ScalarProgram` variant to exactly one `ArithmeticType`; `ScalarProgram::PointwiseBf16` yields `Bf16` and every other variant yields `F32`. No fused mixed-arithmetic variant exists. ADR 0091 reads `decision_status: "accepted"` with `implementation_status: "not-started"`, and no conversion key is registered.

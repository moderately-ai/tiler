---
id: subject-the-numerical-realization-when-a-region-carries-two-arithmetic-types
title: Subject the numerical realization when a region carries two arithmetic types
status: deferred
priority: p3
dependencies: []
related: [accept-the-bf16-subnormal-resolution-carrier, land-the-bf16-conversion-and-accumulator-adr, give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject]
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

- **Subject lives at the bridge, not on the realization.** `ReferenceNumericalConformance::from_realization` has non-test callers in `tiler-conformance` (`bf16_vertical` `conformance_of`; publication `conformance_stated_for`). Those callers pass `ArithmeticType` from the region or `RealizationWitness` — the subject is deliberately an argument to the bridge, not a field of `NumericalRealization`. Storing a subject on the realization would restate `region_arithmetic_type` and pay an identity-domain migration for a multi-type region that is still unreachable. A subject field would still not serve a second arithmetic type until the trigger fires.
- **`NumericalRealization` is folded into artifact identity** (ADR 0076 item 4), so this is an irreversible identity-domain migration, and the repository's standing position is to keep changes cheap to reverse until verified.
- **It would not have inoculated against the hazard it was chosen for.** See below.

**Correction — 2026-08-10.** The 2026-08-07 decline ground "the consumer does not exist / `from_realization` has no caller" is **false** at this base. [`give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject`](give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject.md) landed non-test callers that pass subject at the point of use. The decision to keep format derived from the region (arm A) stands; the trigger below is unchanged. Arm B remains closed until a multi-type region forces the three-field identity migration.

## Trigger for reconsideration

**A `ScalarProgram` variant that carries two arithmetic types in one region.** Today `region_arithmetic_type` (`pub(super) const fn region_arithmetic_type` in `crates/tiler-ir/src/schedule/model.rs`) is a total function from `ScalarProgram` to exactly one `ArithmeticType`, so a region has one arithmetic type structurally. ADR 0091 keeps BF16/binary32 conversion as two separate typed families and has a mixed-width program spell its conversions as explicit operations, so registering a conversion does **not** by itself fire this.

**When it fires, this is not a two-field change.** `canonical_arithmetic_nan_bits` is one `u32` per region and carries the region's own arithmetic NaN pattern zero-extended; a two-arithmetic region breaks it in the same instant. That field's own doc records the 2026-08-06 decision, taken independently in the schedule and artifact-ABI layers, not to widen it — on the ground that the arithmetic type is already a total function of the region's scalar program. Agreement is checked rather than assumed: `verify_pointwise_bf16` gates the zero-extended bf16 NaN payload against `canonical_arithmetic_nan_bits`, and `verify_accumulation_width` gates accumulation against `region_arithmetic_type`. **All three fields must be decided together**, or the record ends up carrying its subject two different ways.

Recheck: `grep -n "fn region_arithmetic_type" -A 15 crates/tiler-ir/src/schedule/model.rs` — more than one `ArithmeticType` reachable from a single variant is the fired state.

## Trigger check log

- 2026-08-07 — **not fired, and filed in this state.** `region_arithmetic_type` maps every `ScalarProgram` variant to exactly one `ArithmeticType`; `ScalarProgram::PointwiseBf16` yields `Bf16` and every other variant yields `F32`. No fused mixed-arithmetic variant exists. ADR 0091 reads `decision_status: "accepted"` with `implementation_status: "not-started"`, and no conversion key is registered.
- 2026-08-09 — **not fired.** `region_arithmetic_type` remains a total one-type answer: `PointwiseBf16` yields `Bf16`, while the pointwise F32, strict-affine dequantize, fold, contraction, and maximum variants yield `F32`. The registered conversion work has not introduced a `ScalarProgram` variant carrying two arithmetic types in one region, so the three-field identity migration described above is still unnecessary.
- 2026-08-10 — **not fired.** `region_arithmetic_type` is still exhaustive one-type (`PointwiseBf16` → `Bf16`; remaining arms → `F32`). No fused mixed-arithmetic `ScalarProgram` variant. ADR 0091 remains accepted with `implementation_status: "not-started"`. Command: `grep -n "fn region_arithmetic_type" -A 15 crates/tiler-ir/src/schedule/model.rs`.

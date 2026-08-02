---
id: state-and-check-a-bf16-numerical-contract
title: State and check a BF16 numerical contract
status: todo
priority: p1
dependencies: [declare-the-bf16-rows-on-the-authoritative-metal-profile]
related: [admit-bf16-into-the-schedule-and-kernel-vocabulary, design-the-bf16-computation-and-accumulator-contract]
scopes: [implementation/compiler, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, bf16, numerics, target-profiles]
---
## User-visible outcome

A caller can state the pure-BF16 numerical contract that the registered
constant/multiply/add family already defines, and target feasibility checks that
contract against the exact BF16 arithmetic subject. On the measured macOS
Apple9 profile, a strict subnormal-preserving request is refused with a typed
numerical gap naming BF16 and the profile's measured sign-preserving flush; a
flush-accepting request reaches the next independently unsupported layer. No
F32 contract entry, profile row, or arithmetic behaviour is inherited.

## Why this is a separate boundary

**Fact, at `aa09b5e`.** `ScalarArithmetic::new` can now construct the validated
`(ArithmeticType::Bf16, Bf16::resolved_type())` subject, and
`TargetProfileBuilder::declare_measured_*_subnormal_behaviour` can state its
complete exclusive tables. `declare-the-bf16-rows-on-the-authoritative-metal-profile`
can therefore add the measured BF16 rows without changing this ticket's scope.

**Fact.** The public request contract cannot ask the corresponding question.
`NumericalContract` contains one dimension vector documented as "Every
resolution is stated for `f32`"; its only entry point is
`NumericalContractBuilder::strict_f32`; and `NumericalContract::resolve`
constructs `StrictF32NumericalContract`. A pure-BF16 semantic program is refused
at the request boundary with `dtype-f32` before target numerical feasibility,
so adding a BF16 profile row does not make a strict BF16 preservation refusal
observable through `compile`.

**Inference.** Treating the existing `STRICT_F32` contract as a BF16 contract
would silently transfer per-dtype behaviour across a boundary the retained
Apple measurements prove differs (`f16` preserves where `f32` and BF16 flush).
Adding a test-only target resolver in `tiler-build` would prove a path no caller
can use. The missing work is therefore a compiler numerical-contract boundary,
not part of transcribing the measured profile rows.

## Scope keys

- Run the AGENTS.md elimination over the durable shape: a per-arithmetic
  contract, a contract entry carrying its exact `ScalarArithmetic`, or another
  consumer-neutral representation. Reject any shape that defaults an omitted
  dtype, makes one contract silently apply to every float width, or lets the
  profile choose program meaning.
- Preserve every existing F32 contract key and meaning byte-for-byte. New BF16
  meaning receives a distinct, canonical identity; enumerate and recompute
  every moved pin on the tree the identity step lands into.
- Keep computation and accumulator types on the operation facts, as accepted by
  ADR 0091. This ticket states numerical permissions and required behaviours;
  it does not introduce implicit promotion or a mixed-precision operation.
- The request must check the BF16 contract before physical planning. A strict
  preservation requirement must report the profile's measured
  `DeclaredUnhonourable` fact, not degrade into `Unknown`, a generic dtype
  refusal, or a late Metal emission error.
- Consequential public constructor, builder, or call-site changes remain Tom's
  decision under ADR 0075. A tested implementation is a draft until accepted.

## Required evidence

- A pure-BF16 constant/multiply/add program states a strict preserving contract
  and the macOS Apple9 profile refuses it with `ArithmeticType::Bf16`,
  `Bf16::resolved_type()`, `SubnormalMode::Preserve`, `Unsupported`, and the
  honoured sign-preserving flush, all from the exact measured source.
- The same program under a sign-preserving-flush contract passes numerical
  feasibility and reaches the next named unsupported layer without implying
  schedule, kernel, lowering, or runtime support.
- An F32 contract does not answer for BF16 and a BF16 contract does not change
  any existing F32 compile outcome or key.
- A mutation deleting the BF16 profile declaration turns the refusal into
  `Unknown`; a mutation substituting F32 for the subject is detected.
- Every identity movement and public-boundary consequence is enumerated, with
  targeted fmt/check/clippy/nextest/doc-tests, `tkt lint`, `git diff --check`,
  and `tkt guard` green.

## Closes when

The pure-BF16 numerical contract is statable and checked before physical
planning; the measured macOS row produces the named preservation refusal and a
flush-accepting request passes that dimension; no neighbouring dtype or F32
identity moves silently; and Tom has accepted every consequential public
boundary.

## Graph maintenance

- Depends on `declare-the-bf16-rows-on-the-authoritative-metal-profile`, which
  owns the measured fact this request must consume. The provider does not depend
  on its first consumer; keeping the edge this direction avoids a cycle.
- Related to `admit-bf16-into-the-schedule-and-kernel-vocabulary`: this ticket's
  first positive request may still stop at that independently unsupported layer,
  and must name rather than absorb it.
- Related to `design-the-bf16-computation-and-accumulator-contract`, whose
  accepted outcome keeps accumulator width on operation identity and forbids a
  fused BF16 operation. Do not reopen either decision here.

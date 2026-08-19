---
id: define-host-applicability-for-profiles-whose-rows-span-environments
title: Define host applicability for profiles whose rows span environments
status: deferred
priority: p3
dependencies: []
related: [decide-the-host-evidence-to-profile-composition-model, reseat-the-grid-and-cost-profile-rows-on-the-re-measured-records]
scopes: [implementation/metal, research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [metal, target-profiles, applicability, provenance, deferred]
---
## User-visible outcome

When host admission becomes answerable at all, a host-applicability evaluation for a profile whose measured rows span more than one execution environment has a defined, fail-closed comparison shape: which rows a given host observation can stand inside, what a policy names, and what a partial match refuses — instead of a single-row policy silently mismatching a multi-environment profile.

## Why deferred rather than ready

**Fact — the mismatch is structural but not yet live.** `MetalHostApplicabilityPolicy` in `crates/tiler-metal/src/applicability.rs` is a closed single-row value (`FIRST_MACOS_APPLE9`, anchor `tiler.metal.host-applicability.macos-27.0-26A5388g-arm64-m4max-apple9.v1`), while the accepted (R, R) disposition on `resolve-the-retained-metal-profile-measurement-invocation-authority` (anchor `the profile's measured rows then span two execution environments, each scoped exactly`) commits the standard profile's measured rows to span `26A5388g` and `26A5406e` once `reseat-the-grid-and-cost-profile-rows-on-the-re-measured-records` lands. After the reseat, "the exact measured row" of that profile is a set of two, and a one-row policy can no longer describe it.

**Fact — nothing is wrong today, by construction.** ADR 0086 keeps the positive receipt uninhabited: a mismatching host is refused on its first failing environment predicate with the named mismatch, and even a host matching every field is refused by `evaluate_metal_host_applicability` with `UnknownNativeTranslationAuthority`, so no observation earns a receipt. *(Corrected 2026-08-18 per the composition-packet independent review at `427b2080`; this originally said every host is refused with the authority refusal "whatever that comparison would have said", which misnames the refusal for the mismatching population.)* No consumer reads cross-row environment equality, so the mapping question has no observable consequence until an ADR 0086 item-7 trigger fires or Tom decides to admit hosts.

**What the design must answer when it wakes.** Per the composition-model packet on `decide-the-host-evidence-to-profile-composition-model`: matching stays byte-exact per environment field against a measured context of a row's own population, and stays necessary-not-sufficient; the open questions are whether applicability is answered per row-population (a host may stand inside some populations and outside others), what a policy value names (one environment, a set, or a per-population map), and how a partial match is refused and explained. Averaging environments, matching on device model or family, or treating a match as authority are all already eliminated by ADR 0086 item 3 and its recorded reason.

## Closes when

A refusal-first design fixes the policy shape, the per-population matching rule, and the partial-match refusal vocabulary, consistent with ADR 0086 and the accepted composition model, or a recorded decision retires host admission in a way that makes the question moot.

## Trigger check log

- 2026-08-18 — **not fired.** No ADR 0086 item-7 trigger has fired (no Apple API naming the translating component, no accepted attributing observer, no adopted host-attestation mechanism), and the positive receipt remains structurally uninhabited. Reproduce: `grep -n "NoAdmissibleAuthority" crates/tiler-metal/src/applicability.rs` (the uninhabited authority enum is present) and `grep -c "UnknownNativeTranslationAuthority" crates/tiler-metal/src/applicability.rs` (nonzero; a fully matching observation still refuses on it — mismatching hosts refuse earlier on their named predicate).

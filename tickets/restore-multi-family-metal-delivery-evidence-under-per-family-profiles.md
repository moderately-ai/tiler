---
id: restore-multi-family-metal-delivery-evidence-under-per-family-profiles
title: Restore multi-family Metal delivery evidence under per-family profiles
status: in-progress
priority: p3
dependencies: [carry-required-compilation-selection-identity-on-compile-profile-contexts]
related: [reseat-the-grid-and-cost-profile-rows-on-the-re-measured-records]
scopes: [implementation/build, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [metal, artifacts, cache, fail-closed]
claimed_from: todo
assignee: worker-multifamily
lease_expires_at: 1787415365
---
## Why this exists

The required compilation-selection carrier made the `second_artifact_family_fixture` unrepresentable: its premise — that `MetalTargetFacts::platform` does not project into the compiler profile, so two artifact families can share one profile and one selected plan — is falsified by the accepted selection decision, because a second family's production selection can never equal the macOS-measured rows' recorded selection. The declaration now refuses it by name (`a_second_artifact_family_cannot_wear_this_profiles_measured_rows`, `CompilationSelectionMismatch { population: GridAxis }`), which is the ledger's prose inheritance refusal made structural.

Three `metal_plan` tests rode on that fixture and were removed with it: `one_envelope_carries_one_payload_per_artifact_family`, `every_multi_position_stage_is_retained_under_its_own_governed_label`, and `a_payload_at_another_familys_delivery_position_is_refused`. Their subject — the multi-position envelope, per-position retention labels, and swapped-position identity refusals of the delivered-payload cache seam — is real machinery that now has no multi-position exercise anywhere: the neutral `tests/custom_backend` caller declares exactly one delivery position.

## Facts at filing (2026-08-19)

- `accept_or_publish_metal_plan` still takes one plan plus N declarations and checks every declaration against the one compilation's profile (`require_compiled_under`), so multi-family delivery through it presupposes a shared profile — which per-family measured provenance now forbids for any two real families.
- A real second family (`first-authoritative-ios-metal-compile-declaration`) was always going to carry its own measured profile, so the multi-declaration path never worked for real distinct-profile families; only the fixture's shared-profile shortcut exercised it.

## Outcome

Either restore multi-position evidence at the neutral seam (a second delivery position in `tests/custom_backend`, covering envelope payload-per-position resolution, per-position retention labels, and the swapped-position identity refusal), or reshape the Metal multi-family delivery boundary for per-family profiles (per-profile plan compilation feeding one envelope) — the latter is a public-boundary question for Tom. Until one lands, the multi-position machinery is unsupported-by-evidence from the Metal path and this ticket is the record of that gap.

## Closes when

Multi-position envelope machinery has live evidence again (neutral or Metal), or Tom decides the multi-family Metal delivery boundary and the deciding ticket supersedes this one.

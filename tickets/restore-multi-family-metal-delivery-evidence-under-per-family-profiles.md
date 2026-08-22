---
id: restore-multi-family-metal-delivery-evidence-under-per-family-profiles
title: Restore multi-family Metal delivery evidence under per-family profiles
status: done
priority: p3
dependencies: [carry-required-compilation-selection-identity-on-compile-profile-contexts]
related: [reseat-the-grid-and-cost-profile-rows-on-the-re-measured-records]
scopes: [implementation/build, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [metal, artifacts, cache, fail-closed]
---
## Why this exists

The required compilation-selection carrier made the `second_artifact_family_fixture` unrepresentable: its premise — that `MetalTargetFacts::platform` does not project into the compiler profile, so two artifact families can share one profile and one selected plan — is falsified by the accepted selection decision, because a second family's production selection can never equal the macOS-measured rows' recorded selection. The declaration now refuses it by name (`a_second_artifact_family_cannot_wear_this_profiles_measured_rows`, `CompilationSelectionMismatch { population: GridAxis }`), which is the ledger's prose inheritance refusal made structural.

Three `metal_plan` tests rode on that fixture and were removed with it: `one_envelope_carries_one_payload_per_artifact_family`, `every_multi_position_stage_is_retained_under_its_own_governed_label`, and `a_payload_at_another_familys_delivery_position_is_refused`. Their subject — the multi-position envelope, per-position retention labels, and swapped-position identity refusals of the delivered-payload cache seam — is real machinery that now has no multi-position exercise anywhere: the neutral `tests/custom_backend` caller declares exactly one delivery position.

## Facts at filing (2026-08-19)

- `accept_or_publish_metal_plan` still takes one plan plus N declarations and checks every declaration against the one compilation's profile (`require_compiled_under`), so multi-family delivery through it presupposes a shared profile — which per-family measured provenance now forbids for any two real families.
- A real second family (`first-authoritative-ios-metal-compile-declaration`) was always going to carry its own measured profile, so the multi-declaration path never worked for real distinct-profile families; only the fixture's shared-profile shortcut exercised it.

## Fact audit at base `3cca5438` (2026-08-22)

Every Fact re-read at this base rather than carried from the filing. One is imprecise; the rest hold.

| Claim | Verdict | Evidence |
| --- | --- | --- |
| The declaration refuses a second family by name, `a_second_artifact_family_cannot_wear_this_profiles_measured_rows` with `CompilationSelectionMismatch { population: GridAxis }` | **Verified** | `crates/tiler-build/src/metal_declaration.rs`, anchor `macOS-measured rows must not source an iOS production profile`; the asserted error is `BoundMetalDeclarationError::CompilationSelectionMismatch` with `MetalProfileMeasurementPopulation::GridAxis`. The ticket's unqualified `GridAxis` is the enum variant, not a second name. |
| `second_artifact_family_fixture` no longer exists | **Verified** | `git log -S second_artifact_family_fixture -- crates/tiler-build` ends at `1f6ec214`; no source hit remains. |
| Three `metal_plan` tests were removed with it — `one_envelope_carries_one_payload_per_artifact_family`, `every_multi_position_stage_is_retained_under_its_own_governed_label`, `a_payload_at_another_familys_delivery_position_is_refused` | **Verified** | Each name's `git log -S` over `crates/tiler-build` ends at `1f6ec214`; none is present in `metal_plan.rs`'s test module. |
| `accept_or_publish_metal_plan` takes one plan plus N declarations and checks every declaration against the one compilation's profile | **Verified** | `crates/tiler-build/src/metal_plan.rs`, anchor `so a selection with one wrong family costs no compiler work`: a loop over `declarations` calling `require_compiled_under(compilation.target_profile_key(), compilation.target_profile_descriptor())` before any emission. |
| A real second family carries its own measured profile, so the multi-declaration path never worked for real distinct-profile families | **Verified** | `tickets/first-authoritative-ios-metal-compile-declaration.md` (`deferred`), anchor `No iOS family, physical or simulated, gains a row from this one`. |
| The multi-position machinery "has no multi-position exercise anywhere" | **Imprecise** | True of the **build orchestration seam** and false of the artifact model. No `DeclaredPayload` run longer than one existed anywhere in `crates/` or `prototypes/` — every caller of `accept_or_publish_delivered_payload_artifact` and `accept_or_publish_metal_plan` passed `std::slice::from_ref`, and `tests/custom_backend`'s `sole` helper said so in words (`this backend declares exactly one delivery position`). But `tiler-artifact`'s own tests do exercise several positions: `whole_artifact_rules.rs` asserts `EmptyDelivery`, `DeliveryCardinality`, and `AmbiguousPayloadDelivery`, and `forged_models.rs` the same two. What had no exercise is the seam that *orders* positions, not the model that bounds them. |
| The neutral `tests/custom_backend` caller declares exactly one delivery position | **Verified** at the base, and changed by this work. | `sole`'s own doc and every `std::slice::from_ref(&declaration.declared())` call site. |

## Outcome

**The neutral seam, not the Metal one, and the Metal option was eliminated by reading rather than deferred.** Restoring the removed tests at the Metal path needs two `MetalTargetFacts` differing in `platform`, and those facts carry `subnormal_arithmetic` — so minting an iOS one from the macOS rows is exactly the move `first-authoritative-ios-metal-compile-declaration` prohibits in its closing paragraph ("**Do not** … reus[e] the macOS measurement source under an iOS platform"). Reshaping the Metal multi-family boundary is the public-boundary question that ticket names and stays Tom's. What is left is the neutral seam, which the Outcome already offered and which is entirely inside `implementation/build`.

**The neutral backend can hold two families honestly, which is the whole reason this works there and not at Metal.** `tests/custom_backend`'s scalar-host backend runs no target compiler: its payload is an image its own in-process translator writes, so it has no measured rows to inherit and states no numerical claim about either target. Its profile key is `tiler.test.scalar-host-aarch64-darwin`, and `aarch64-apple-darwin` and `aarch64-apple-ios` are both aarch64 Darwin, so every axis the profile declares holds for both and the two families share one profile key and one byte-identical canonical descriptor — the same shape the ledger records for `MetalTargetFacts::platform`. The family reaches the artifact through the payload provenance and through target-mangled entry-point symbols, and nowhere else.

Landed, all in `implementation/build`:

- `tests/custom_backend/profile.rs` — `TARGET_TRIPLE` becomes `MACOS_TARGET_TRIPLE` and `IOS_TARGET_TRIPLE`, with the argument for why one profile covers both.
- `tests/custom_backend/backend.rs` — `ScalarHostFamily` (census sized by `variant_count`); `emit`, `payload_metadata`, `prepare`, and `symbol_for` take it; `assemble` and `assemble_pending` take **delivery-ordered runs** rather than one payload each.
- `tests/custom_backend/main.rs` — four new cases: `one_envelope_carries_one_payload_per_delivery_position`, `every_delivery_positions_retention_is_labelled_by_its_own_position`, `a_payload_at_another_familys_delivery_position_is_refused` (four sub-cases: two orders are two artifacts; a carried swap is `ArtifactIdentity`; a pending swap is `PayloadSubject { delivery: 0 }`; a correspondence closure reading the wrong position is `Correspondence { delivery: 0, cause: Target }`), and `another_familys_object_under_this_familys_metadata_is_caught_only_by_the_backend`.
- `src/metal_cache.rs` — a `#[cfg(test)]` module covering the per-position stage labelling `stage_retention` owns, from one fake-toolchain compilation's real `StageOutputs`: `every_delivery_positions_stage_is_retained_under_its_own_governed_label`, `a_stage_label_names_its_position_and_its_tool`, and `a_selection_wider_than_the_run_limit_states_the_elision` (the all-or-nothing arm, first reachable at nine positions).

**No pin moved.** `the_standard_metal_path_publishes_its_recorded_identities` and `the_authority_ledger_mirrors_the_live_standard_metal_pins` pass unchanged: the family plumbing is confined to the `custom_backend` test crate, whose identities nothing pins, and the `metal_cache` module is `#[cfg(test)]`.

**A neighbouring deferred boundary is now covered, and by a narrower route than it asked for.** [`cover-multi-position-stage-retention`](cover-multi-position-stage-retention.md) is `done` and left its all-or-nothing half open with this trigger: *"cover the all-or-nothing branch when an authorized product path can construct and pass nine declared artifact families (18 stage runs) to this seam"*. `a_selection_wider_than_the_run_limit_states_the_elision` covers the branch **at the function** — nine cloned `StageOutputs` into `stage_retention` — and not through a product path, because no product path can pass nine families and none is expected to. The coordinator may want to record that against that ticket rather than leaving its deferral note reading as uncovered; nothing was edited there, since it is terminal.

**What is still unsupported by evidence, named rather than implied.** A two-family **Metal** publication end to end — `accept_or_publish_metal_plan` with two declarations, two emitted units, two AOT triples — remains unreachable and belongs to `first-authoritative-ios-metal-compile-declaration`. The neutral cases cover the seam that orders positions; they do not cover Metal's own emission-per-family.

## Closes when

Multi-position envelope machinery has live evidence again (neutral or Metal), or Tom decides the multi-family Metal delivery boundary and the deciding ticket supersedes this one.

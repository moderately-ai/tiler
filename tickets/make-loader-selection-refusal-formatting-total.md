---
id: make-loader-selection-refusal-formatting-total
title: Make loader selection-refusal formatting total
status: in-progress
priority: p1
dependencies: [select-executable-variants-across-registered-backend-families]
related: [accept-the-loader-variant-eligibility-vocabulary]
scopes: [implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, correctness, diagnostics]
claimed_from: todo
assignee: worker-loader-refusal-fmt
lease_expires_at: 1786585709
---
## User-visible outcome

Formatting a publicly constructible loader rejection never panics, including malformed `packaged`/`filtered` counts that the loader itself would never produce.

## Facts to re-verify

**Fact — loader-produced counts are coherent.** `select_variant` reports `NoEligibleVariant` only when `filtered.len() == packaged`; otherwise `NoApplicableVariant` has fewer filtered members than packaged.

**Fact — the public enum is constructible outside the loader.** A caller can construct `LoadRejection::NoApplicableVariant { packaged, filtered }` with `filtered.len() > packaged`. Its `Display` path currently subtracts the two counts without a checked or saturating boundary and can panic in debug builds.

## Required outcome

- Make every `LoadRejection` display path total over every publicly constructible value.
- Preserve the exact current rendering for all loader-produced coherent refusals.
- Do not normalize, silently discard, or reinterpret malformed evidence as a valid loader-produced count; render the inconsistency explicitly or use a checked presentation that cannot claim a false eligible population.
- Add a direct malformed public-value regression and perturb the count subject with its assertion unchanged.

## Non-goals

No routing, selection, fallback, artifact, identity, or public-field decision changes here. The accepted leaf-data fields remain public.

## Required checks

Run focused formatting tests, the runtime package census and doctests, Clippy/rustdoc with warnings denied, citations, lint, exact-base guard, and the exact-tip publication gate required by the runtime crate change.

## Worker report — 2026-08-12, base `61246804`

**Fact audit.** `select_variant` still returns `NoEligibleVariant` only when `filtered.len() == packaged` (anchor `if filtered.len() == packaged`). `LoadRejection::NoApplicableVariant` fields remain public. `Display` previously did `packaged - filtered.len()` (now `checked_sub`).

**Change.** Display is total: a loader-legal count keeps the previous rendering; `filtered.len() > packaged` names the inconsistency and does not claim an eligible population.

**Evidence.** `a_malformed_public_applicable_count_does_not_invent_an_eligible_population` and `an_unfiltered_portfolio_reports_every_variant_as_eligible` pass. Subject perturbation is the packaged count 1 vs 2 in that test.

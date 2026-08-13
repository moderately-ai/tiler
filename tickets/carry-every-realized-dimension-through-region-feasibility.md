---
id: carry-every-realized-dimension-through-region-feasibility
title: Carry every realized dimension through region feasibility
status: done
priority: p1
dependencies: []
related: [carry-the-elementary-numerical-dimensions-in-the-region-realization, wire-the-delivered-realization-record-into-the-artifact]
scopes: [implementation/compiler, contracts/optimizer, contracts/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, numerics, feasibility, correctness, fail-closed]
---

# Carry every realized dimension through region feasibility

## User-visible outcome

A selected region carries target evidence for every numerical behaviour its verified `ResourceRequirements` states, rather than asking feasibility about only a prefix and later treating a consumed dimension as not required.

## Facts to re-audit before editing

Re-read in full at worker base `1913da6f8373cdb29a80a002ab2f69b1488c8e33` before any edit. Coordinator-supplied commands and the filing-base wording were re-run and re-read at this base; two coordinator path claims were wrong.

**Fact — verified, path repaired.** `ResourceRequirements` still carries eight numerical fields: `input_subnormals`, `result_subnormals`, `contraction`, `reassociation`, `permutation`, `signed_zero`, `nan_assumptions`, `infinity_assumptions`. The owning type is `pub struct ResourceRequirements` in `crates/tiler-ir/src/schedule/model.rs`, not `crates/tiler-ir/src/schedule/numerics.rs`. The coordinator command `rg -n "pub struct ResourceRequirements" -A 20 crates/tiler-ir/src/schedule/numerics.rs` matches nothing; the same search under `crates/tiler-ir/src/schedule/` hits `model.rs`. `NumericalRealization` in `numerics.rs` carries the same eight behaviour fields plus `profile_key` and `canonical_arithmetic_nan_bits`. `derive_requirements` copies every one of those eight onto the resource record (`input_subnormals: region.index.numerical.input_subnormals` through `infinity_assumptions: region.index.numerical.infinity_assumptions`).

**Fact — verified, module-path claim remains false.** `fn region_proposal` in `crates/tiler-compiler/src/physical.rs` still constructs `NumericalRequirement`s for only `InputSubnormals`, `ResultSubnormals`, `Contraction`, and `Reassociation`. There is no `region_proposal.rs`. The surrounding contract still says the region's declared realization is carried forward **per dimension**. `REALIZED_DIMENSIONS` in `crates/tiler-compiler/src/policy.rs` already names all eight, and the governed profile already declares honourability for permutation, signed zero, NaN assumptions, and infinity assumptions (`governed_target_honourability`).

**Fact — verified after re-reading the three paths the coordinator left unverified.**

- **Request-resolution already asks the consumable set.** `StrictF32NumericalContract::dimension_requirements` delegates to `policy::dimension_requirements`, which walks `CANONICAL_DIMENSIONS` filtered by `is_consumable`. Every entry of `REALIZED_DIMENSIONS` is consumable, so `assess_contract` places all eight on the target. The comment on `assess_contract` that the proposal carries the contract's *four* dimensions is stale: that path is already the consumable projection, not a four-dimension prefix.
- **Selected-plan evidence is the region prefix.** `aggregate_honoured` in `selection.rs` folds each region's `AdmissionEvidence::honoured()`, and that evidence is whatever `assess_resources` → `region_proposal` asked. `an_honoured_dimension_no_covered_occurrence_consumes_carries_no_row` asserts the honoured set is exactly `{InputSubnormals, ResultSubnormals, Contraction, Reassociation}` and says "the region proposal asks every candidate about the same four dimensions".
- **Delivered-realization then derives `NotRequired` from that prefix.** `DeliveredRealizationEvidence::materialize` iterates `plan.honoured()` and emits a locus row only for a honoured dimension a covered occurrence consumes. A dimension the proposal never asked is absent from `honoured`, so it contributes no row; the artifact builder treats a dimension with no obligations as `NotRequired`. That is the failure the user-visible outcome names: a consumed dimension the verified region already stated is later treated as not required because feasibility was only asked about a prefix. Whole-program contract resolution does not replace this projection.

## Required delivery

- Re-read the complete request-resolution, region-feasibility, selected-plan evidence, and delivered-realization construction paths and repair any stale Fact above before editing.
- Make the region proposal project all eight currently realized dimensions with their exact typed behaviour spaces and canonical order. Do not infer a strict value, omit a field because the current strategy does not transform it, or recover a value from the contract key.
- Prove the projection population from the owning type rather than a hand-written count that can remain green after widening.
- Add positive evidence showing all eight dimensions reach target assessment and selected delivered evidence, plus one-at-a-time subject perturbations for permutation, signed zero, NaN assumptions, and infinity assumptions with unchanged assertions and quoted failures.
- Align stale comments/contracts that claim either four or eight, without pre-implementing the two elementary dimensions owned by the dependent ticket.

## Boundaries

This heals the existing eight-dimension projection only. It does not add reciprocal transformation or approximate intrinsics, change a public vocabulary, choose target behaviour, alter a numerical contract, weaken feasibility, or provide any fallback when a profile is silent. The dependent elementary carrier re-audits this path and grows the complete population from eight to ten after this ticket is done.

## Closes when

Region feasibility and selected delivered evidence carry exactly every dimension in the current `NumericalRealization`/`ResourceRequirements` overlap; removing any one production projection fails its unchanged targeted test; target silence remains `Unknown`; and package plus repository gates are green.

## Outcome

This heals the existing eight-dimension projection only. It advances no support-matrix or dtype-maturity row.

`region_numerical_requirements` now exhaustively destructures `ResourceRequirements` and matches `NumericalDimension`, projecting the eight realized fields in `CANONICAL_DIMENSIONS` order. Reciprocal transform, approximate intrinsics, and materialization rounding stay off the proposal. A profile silent about permutation, signed zero, NaN assumptions, or infinity assumptions stays `Unknown`. Selected-plan honoured facts and delivered obligations now carry the same eight; a consumed dimension is no longer derived `NotRequired` because the region proposal never asked.

The projection population is type-derived: `variant_count::<NumericalDimension>()` pins `CANONICAL_DIMENSIONS`, and an exhaustive carry match — not a hand-written 8 — is the expected set. No public type, numerical vocabulary, or identity domain stepped. Selected-plan identity *content* grows with the four newly honoured facts under the existing `tiler.compiler.selected-physical-plan.v2` tag.

One-at-a-time production perturbations, assertions unchanged:

- Drop permutation: `left: [InputSubnormals, ResultSubnormals, Contraction, Reassociation, SignedZero, NanAssumptions, InfinityAssumptions]` vs `right: […, Permutation, SignedZero, …]`; selected plan: `region feasibility must ask the target about every realized dimension` with the same missing `Permutation`.
- Drop signed zero: left missing `SignedZero`; selected plan missing `SignedZero`.
- Drop NaN assumptions: left missing `NanAssumptions`; selected plan missing `NanAssumptions`.
- Drop infinity assumptions: left missing `InfinityAssumptions`; selected plan missing `InfinityAssumptions`.

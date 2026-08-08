---
id: accept-the-installed-physical-provider-public-surface
title: Accept or revise the installed physical-provider public surface
status: awaiting-decision
priority: p1
dependencies: [drive-an-external-physical-implementation-provider-through-compilation]
related: [accept-the-public-backend-provider-composition-boundary, disclose-offered-and-selected-physical-provider-sets-separately]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, public-boundary, decision, needs-tom]
---
## User-visible outcome

Tom accepts or revises the exact included and excluded public surface of `tiler_compiler::physical_provider`, so it stops being a labelled draft and the contracts that describe it can state an accepted boundary.

## Decision boundary

[ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):19 routes this to Tom in terms — "every concrete public surface named here — **the provider registry and its installation method**, the offered-versus-selected disclosure accessors, the promoted `assemble_artifact` boundary — still comes to Tom at implementation time under [ADR 0075]". [`accept-the-public-backend-provider-composition-boundary`](accept-the-public-backend-provider-composition-boundary.md) accepted the *model* and explicitly did not accept any surface.

This node is not research or implementation work. Only Tom closes it.

## The surface, as landed 2026-08-08

**Included — `tiler_compiler::physical_provider`.** `PhysicalImplementationProvider` (trait, two methods); `ImplementationContext` with `subject`, `target_profile`, `target_profile_key`, `numerical_realization`, `baseline`; `BaselineImplementation` with `region` and `cost`; `FrontierRegionSubject` with `role` and `covered_occurrences`; `ImplementationProposal::scheduled_kernel`; `TargetApplicability::{for_targets, target_profile_keys}`; `PhysicalCostEstimate::structural` with its four readers; `ProviderOffer::{proposing, decline, default}`; `DeclinedStrategy::new`; `StrategyDeclineCause` (`#[non_exhaustive]`); `PhysicalProviderProvenance` and its error; `InstalledPhysicalProviders::{governed, installed, identities}`; `PhysicalProviderInstallationError` (`#[non_exhaustive]`); `GOVERNED_PHYSICAL_COST_MODEL_KEY`.

**Included — `tiler_compiler::session`.** `CompileRequest::with_physical_providers`; `PlanAlternative::selected_physical_providers`; `SelectedImplementation` with `provider`, `provider_explain_subject`, `proposal_kind`.

**Excluded, each by a stated reason rather than by omission.** `ProposalBody` and its subprogram, opaque-call, and reserved-view variants; `KernelSubprogram` and `SubprogramStage`; `SemanticStage` and `SemanticMemberId`; `RegionWrite`; `VerifiedTargetRequest`; `PhysicalCostEstimate::new`; `FrontierRegionSubject`'s constructors and `semantic_members`; `enumerate_frontier`, `PhysicalAuthorities`, and `GovernedPhysicalProvider`; every removal or reordering of the governed provider. Four of these are pinned by `compile_fail` doctests carrying exact error codes in the module documentation.

## The three questions that are genuinely Tom's

1. **Is the additive rule right?** Installing adds to the governed provider and cannot displace it, deliberately unlike `with_capabilities`, which replaces the lowering registry. The ground is that two lowering claimants are a contradiction while two physical implementations are alternatives. What would argue the other way is a caller wanting a compilation the governed provider does not participate in at all.
2. **Is `baseline` the right shape for what a provider reads?** It hands back this host's own spelling for the provider to specialize, which is what makes the seam usable given that the request-subject binding is host-owned — and which also means the seam supports specializing a spelling and not contributing a new region shape. The alternative, exposing the five facts the binding compares, was rejected for creating a second derivation of one answer.
3. **Is `scheduled_kernel` the right restriction?** A caller may propose one body variant. The subprogram and opaque-call exclusions each have an independent reason, so accepting one does not accept the other.

## Closes when

Tom accepts or revises each of the three, the module documentation and the contracts describing the seam state an accepted boundary rather than a draft, and the acceptance provenance — who, date, venue, relay source — is recorded.

## Graph maintenance

- Only Tom approves or revises. After his answer, the implementing agent records it durably, applies every consequence, runs the checks, and closes this node.
- A revision that changes what may be proposed is an implementation change rather than a prose one; file it rather than editing the record to match a tree that has not moved.

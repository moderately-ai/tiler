---
id: prototype-optimizer-conformance-gate
title: Gate the target-neutral optimizer conformance profile
status: in-progress
priority: p0
dependencies: [enforce-repository-validation-gate-integrity, prototype-artifact-program-model]
related: []
scopes: [implementation/compiler, implementation/ir, implementation/reference, implementation/artifact]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, optimizer, conformance, milestone-0b]
claimed_from: todo
assignee: agent-prototype-optimizer-conformance-gate
lease_expires_at: 1784919751
---
Exercise an externally registered operation through the ordinary compiler path,
not a test-only shortcut. Cover at least two non-isomorphic graph shapes plus
fan-out or ordered multi-output behavior: generic occurrences, checked
refinement, region enumeration, legality evidence, complete selection, verified
KIR, neutral and artifact program construction, typed stable explain,
deterministic identity, and the correct failure taxonomy. Remove proof-only
candidate lists and downstream `cfg(test)` isolation after interface review.

Include identity conformance for provider-only revision changes, identical
region/index/schedule structure at distinct occurrences, occurrence-specific
refinements, and complete-plan coverage. Assert identity and selected-provider
provenance at every implemented layer. Each change must affect only the identity
and provenance subjects governed by ADR 0072.

The reviewed draft authorities this gate must wire into the ordinary path are now
concrete: `capability`, `legality`, `fusion_legality`, `frontier`, `cover`, and
`selection` (plus the pre-existing `explain`/`feasibility` drafts). Each carries a
module-level `#![allow(dead_code, reason = "reviewed draft authority; not yet
wired…")]` that must be removed as it is wired — a still-present allow after this
gate is a sign the authority is not actually on the compile path. Two concrete
deferrals recorded at their review to settle here: promote `cover`'s draft-local
`CoverBudgets` into the live `request::DeterministicBudgets` (it is local today
only to avoid fields read solely under `cfg(test)`), and emit the draft
authorities' typed events through the explain vocabulary rather than leaving them
explain-silent.

## Outcome

Four of the six reviewed draft authorities the ticket names — `cover`, `fusion_legality`, `frontier`, and `selection` — are now on the ordinary `compile()` path, and their module-level `#![allow(dead_code, …)]` markers are gone. The compile path no longer constructs two hard-coded alternatives; it enumerates legal complete covers, decides each multi-occurrence region's fusion legality, enumerates each cover region's local implementation frontier from a registered physical provider, joins them into complete physical plans, and assembles each retained plan into verified KIR, a kernel program, and a neutral artifact construction plan. A retained alternative *is* a selected plan: its stable identifier is the plan's content-derived identity label and its cost is the plan's exact aggregate structural cost, so the two hard-coded `alternative:*-serial-sum.v1` names and the pipeline-local `StructuralCost` are gone.

The two recorded deferrals are settled. `cover::CoverBudgets` and `selection::PlanBudgets` are deleted; their fields are live `request::DeterministicBudgets` fields (`region_covers`, `region_cover_expansions`, `physical_plan_combinations`) folded into the canonical request-subject encoding. Every wired authority now emits typed explain records — `cover.enumeration.v1`, `fusion.legality.v1`, `frontier.enumeration.v1`, `selection.complete-plan.v1`, plus per-plan `compile.plan.boundary`, `schedule.plan-regions`, `kernel.plan-refinement`, `program.plan-verified`, and `artifact.plan-construction` — with a fusion-legality verdict attributed to the capability provider that declared the member operations' roles, and typed budget stops and infeasibility reasons for cover enumeration and plan enumeration. `pipeline::tests::every_wired_authority_emits_its_typed_explain_records` pins the exhaustive rule-count snapshot, so a newly explain-silent authority fails the gate. The `#[cfg(test)]` isolation on `CompilationRequest::governed` and `DeterministicBudgets::governed` is removed: the governed request profile is an ordinary crate-internal constructor.

The `pipeline::conformance` module is the gate itself. It defines the whole operation set through an out-of-crate `SemanticRegistryProvider` written only against `tiler-ir`'s public surface and drives every case through `compile()`; nothing reaches past that entry point into a stage-local constructor. It covers two non-isomorphic graph shapes (rank-2 trailing reduction, rank-3 interior reduction) and producer fan-out through a shared constant, asserts complete-plan coverage and one implementation per region occurrence, rejects an ordered multi-output program explicitly rather than approximating it, and asserts ADR 0072 identity conformance: a provider-only revision change preserves graph meaning and the reached-definition projection, changes admission provenance and the registry snapshot, and leaves index/schedule identity, KIR, the plan receipt, the aggregate cost, and the selected lowering providers byte-identical; structurally identical constant regions at distinct graph occurrences share one `RegionContentIdentity` and keep distinct `RegionOccurrenceIdentity` values.

Two markers remain, both on authorities that *are* on the compile path, and both reasons were rewritten to state what is actually reserved rather than that the module is unwired. `explain` reserves evidence, quantity, disposition, and subject-kind variants the bounded profile does not yet produce, plus the presentation renderer that only a trace consumer calls. `feasibility` reserves the later-phase surface — artifact-evidence, device-runtime, prepared-kernel, and launch phases with their fact authorities and validity scopes, the deferred and unknown verdicts, and the feasible-set view — which no compile-profile assessment can reach.

`capability` and `legality` are not wired, and `request::CompilerCapabilitySnapshot` still carries two named `LoweringProviderIdentity` fields that the artifact plan reads directly. That is the last proof-only candidate list on the compile path. It is split into `wire-capability-and-refinement-into-compile-path` rather than hidden here, because the work is gated on evidence this slice measured: `tiler_ir::index` bounds exhaustive verification at `MAX_EXHAUSTIVE_PROOF_CELLS = 1_048_576`, so `legality::refine_index_region` cannot be an unconditional compile-path gate — the existing `[70_000, 2]` and `[70_000, 70_000]` conformance fixtures both exceed it, and making refinement mandatory would convert a target-feasibility verdict into a proof-budget rejection. The follow-up records the required split (unconditional capability resolution, refinement as exhaustive finite evidence with a typed `ProofResource::Cells` budget stop when it is unaffordable) and the governed scalar-registry and index-access-provider work it needs.

Three behaviours changed and their tests were updated rather than relaxed. A cover budget of one now loses the discovered two-region partition — the enumerator retains the all-singleton and whole-program covers unconditionally, and the bounded profile implements no singleton region — so the materialized alternative is lost and the typed stop makes that visible. A zero `region_candidates_per_seed` budget leaves only singleton candidates, so compilation fails closed with a typed no-complete-plan error instead of implementing a pointwise region that region formation never proposed. Target rejections are now attributed to region roles and deduplicated by role and axis, because a rejected region is not an alternative.

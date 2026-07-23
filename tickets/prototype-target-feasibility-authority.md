---
id: prototype-target-feasibility-authority
title: Implement checked target-profile feasibility authority
status: in-progress
priority: p0
dependencies: [prototype-typed-explain-infrastructure]
related: [target-profile-feasibility-model]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, target-profile, feasibility, authority]
claimed_from: todo
assignee: agent-prototype-target-feasibility-authority
lease_expires_at: 1784843428
---
Implement immutable checked target profiles and typed feasibility predicates,
facts, provenance, evaluation phases, resource limits, and Unknown outcomes.
Hard feasibility is not cost; malformed profiles/proposals are intrinsic errors
and a valid empty feasible set is a distinct result.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Outcome

Implemented the immutable checked target-profile feasibility authority per ADR 0043 as a new crate-internal module `crates/tiler-compiler/src/feasibility.rs`. The authority is entirely within `implementation/compiler`; `tiler-ir` was not needed and was not touched, because target profiles are a physical-planning contract that would pollute the semantic IR crate. No `pub` (crate-external) or cross-crate item was introduced; the whole surface is `pub(crate)` draft, following the sibling `explain.rs` reserved-draft convention.

**Typed model and four outcomes.** Predicates range over a governed, bounded `CapabilityAxis` vocabulary (grid-axis threads, workgroup threads, buffer bindings, index-width bits, device address space, strict-f32, local-memory bytes, barriers) with a typed comparison relation each (`AtMost`/`Exact`/`Implies`) — not a free-form property bag. A `CheckedTargetProfile` carries typed `CapabilityFact`s, each with an `AvailabilityPhase`, `FactAuthority`, `FactValidityScope`, and `FactProvenance` (the declaring profile's versioned identity). `CheckedTargetProfile::assess(&FeasibilityProposal, available_phase)` resolves each required axis against the most-refined fact available through the assessment phase and returns exactly one `FeasibilityOutcome` with fixed precedence: any disproved hard predicate ⇒ `Rejected`; else any axis with no admissible fact/query path ⇒ `Unknown`; else any axis whose fact is admissible only at a later phase ⇒ `Deferred` (one nonempty set, canonically sorted by `(phase, axis)` and exposing per-phase groups); else (no remaining checks, including the vacuous empty proposal) ⇒ `Proven`.

**Hard feasibility separate from cost.** The module has no cost type, term, or estimate anywhere; a disproved predicate is a `Rejected` outcome, never an expensive plan, and no admitted quantity is a cost. The two authorities cannot be confused because they share no type.

**Malformed vs empty-feasible vs Unknown.** Malformed profiles/proposals are intrinsic errors surfaced only at construction (`FeasibilityError::MalformedProfile`/`MalformedProposal`: empty/unversioned identity, inadmissible bound for an axis, provenance/authority/phase inconsistency, duplicate `(axis, phase)` fact or duplicate axis requirement) — never an outcome. A valid empty feasible set is `assess_set(...)` returning a `FeasibleSet` whose admitted partition is empty (`admitted_is_empty()`), a legitimate result distinct from an error and from `Unknown`. `Unknown` candidates are a per-candidate outcome that stays in explain/search state and never enters the admitted partition.

**Generalization of the prototype and bounded-path preservation.** `physical.rs::assess_target` now delegates to a new `assess_region`, which builds the checked profile and proposal from `PrototypeTargetProfile` + `ResourceRequirements` (local-memory and barrier ceilings modelled as conservative compile-time bounds of zero) and maps the outcome onto the existing `PhysicalError` contract: `Proven` yields the canonical resolved predicates, `Rejected` yields the first disproved axis as `PhysicalError::Target { rule, region, required, available }` with the identical rule strings and quantities, and a `Deferred`/`Unknown`/malformed verdict fails closed as an intrinsic error (the governed baseline declares only compile-profile predicates, so those cannot occur unless the checked contract drifts). `pipeline.rs::record_target_admissions` now derives its admitted feasibility events from the same `assess_region` authority instead of a parallel inline check list, removing the prior drift risk between the decision and the explain trace. The bounded serial-Sum path is unchanged observably: all 114 `tiler-compiler` tests pass, including every pre-existing physical and pipeline feasibility test.

**Determinism.** No `HashMap` participates in any identity, ordering, or diagnostic. Facts and requirements are stored sorted by their typed keys; deferred sets sort by `(phase, axis)`; the disproved/unknown/proven lists follow the derived canonical `CapabilityAxis` order; the profile identity is a `(key, version)` pair.

**Cross-crate/public boundaries needing Tom's review (all `pub(crate)`, drafts):** the module `feasibility` and its types `AvailabilityPhase`, `CapabilityAxis`, `FactAuthority`, `FactValidityScope`, `ProfileIdentity`, `FactProvenance`, `CapabilityFact`, `CheckedTargetProfile`, `AxisRequirement`, `FeasibilityProposal`, `ResolvedPredicate`, `DeferredPredicate`, `UnknownPredicate`, `Rejection`, `DeferredSet`, `UnknownSet`, `FeasibilityOutcome`, `FeasibleSet`, `FeasibilityError`; the new `physical::assess_region` entry consumed by `pipeline.rs`; and the visibility change making `explain::Quantity::value` `pub(crate)`.

**Deferred / not owned by this ticket:** wiring the versioned profile identity and feasibility-rule identity into artifact/plan hashing (owned by the artifact-identity work); consuming `FeasibilityOutcome::Deferred`/`Unknown` or `FeasibleSet` in the executable frontier / portfolio selection (owned by the scheduling and physical-frontier tickets); Metal/live-device fact providers (owned by Metal realization). These remain reserved typed seams validated by module tests, which is why `feasibility.rs` carries a reserved-draft `#![allow(dead_code, reason = …)]` mirroring `explain.rs`. Docs (`docs/ir.md`, `docs/compiler/optimizer.md`) already describe this exact `Rejected > Unknown > Deferred > Proven` model and are out of this ticket's scope; the implementation matches them, so no doc change was required.

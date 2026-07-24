---
id: prototype-physical-implementation-frontier
title: Implement the physical implementation frontier
status: in-progress
priority: p0
dependencies: [prototype-scheduled-region-ir, prototype-target-feasibility-authority, prototype-fusion-legality-and-numerical-proof]
related: [implement-opaque-physical-call-providers]
scopes: [implementation/compiler, implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, physical-planning, scheduling]
claimed_from: todo
assignee: agent-prototype-physical-implementation-frontier
lease_expires_at: 1784853895
---
Add the typed provider surface for checked scheduled-kernel implementations,
then enumerate their proposals with typed boundary
requirements/guarantees, target/applicability predicates, exact feasibility
resources, estimated costs, provider provenance, and a minimal serial schedule.
Multiple physical providers contribute additive alternatives rather than a
singular-capability ambiguity. Every proposal must re-enter ordinary checked IR
verification. Keep infeasibility distinct from cost and malformed compiler
output distinct from a valid no-plan result.

Frontiers are checked local authorities for individual legal regions. Their
enumeration does not depend on a complete cover and does not prove global
coverage; complete physical-plan selection joins the independent authorities.

The bounded P0 profile rejects opaque-call proposals explicitly while
preserving an additive sum-type/provider seam. The reviewed
`implement-opaque-physical-call-providers` ticket owns their later contracts
and implementation after optimizer conformance and mature boundary properties.

Any consequential public or cross-crate crate, module, trait, type, or call-site boundary remains a draft until Tom reviews and accepts the exact implementation commit. This ticket does not preselect that interface.

## Outcome

**Fact.** Added `crates/tiler-compiler/src/frontier.rs`: a checked, local implementation-frontier authority for one legal region on one target profile, wired as `mod frontier;` in `crates/tiler-compiler/src/lib.rs`. Every new item is crate-internal (`pub(crate)`, private `mod`), matching the reviewed-draft posture of the sibling `feasibility` and `fusion_legality` authorities. It is not wired into the private `compile()` facade; the `#![allow(dead_code, reason=…)]` mirrors those modules, and the complete-physical-plan-selection slice will consume it.

**Fact.** The typed provider surface is `trait PhysicalImplementationProvider { fn provenance(&self) -> PhysicalProviderProvenance; fn propose(&self, &ImplementationContext) -> Vec<ImplementationProposal>; }`. A provider is trusted but never believed: it declares a `ProposalBody`, a `TargetApplicability` predicate, and a `PhysicalCostEstimate`; it does not declare provider identity (the frontier stamps it from the calling provider), exact resource requirements (derived from the verified region), or the boundary contract (derived).

**Fact.** `ProposalBody` is the additive sum type — `ScheduledKernel(Box<ScheduledRegion>)`, `KernelSubprogram`, `OpaqueCall`, `View` — matching `docs/compiler/fusion-and-scheduling.md`. The bounded P0 `enumerate_frontier` admits only `ScheduledKernel` and rejects the three reserved variants as `FrontierRejection::UnsupportedVariant` without failing the enumeration, preserving the seam the reviewed `implement-opaque-physical-call-providers` ticket will fill.

**Fact.** Every `ScheduledKernel` body re-enters ordinary checked verification through the new `crate::physical::verify_schedule_with_feasibility`, which runs the exact path of `verify_schedule` (request-subject precondition, whole-region intrinsic verification, numerical-realization agreement, request-subject binding, single hard-feasibility decision) and additionally returns the resolved feasibility predicates. `verify_schedule` now delegates to it; the redundant `assess_target` helper was removed. A provider cannot smuggle unverified IR.

**Fact.** Hard infeasibility is kept distinct from cost. A `PhysicalError::Target` maps to `FrontierRejection::Infeasible` naming the disproved capability axis, required, and available amounts — never an expensive plan. `PhysicalCostEstimate` is retained only for the post-feasibility non-domination prune (`ImplementationFrontier::non_dominated`) and can neither prove nor disprove feasibility. Malformed compiler output is distinct from a valid no-plan result: a structurally invalid body or an ungoverned cost model fails the whole enumeration closed as `FrontierError`, whereas an enumeration that finds no feasible implementation returns `Ok` with an empty admitted set (`ImplementationFrontier::is_empty`).

**Fact.** Multiple providers contribute additive alternatives (not a singular-capability ambiguity as in the lowering-capability registry): two providers proposing the same region are both admitted and stay distinct through provider provenance in `ImplementationProposalIdentity`. Admitted implementations and rejections are emitted in canonical, provider-order-independent order.

**Measurement.** On base `0afed5d` (worktree `…/prototype-physical-implementation-frontier/edit`), `uv run --locked python scripts/check_repository.py` passes; `cargo nextest run -p tiler-compiler` runs 137 tests, 137 passed (13 new `frontier::tests`); `git diff --check` is clean; `ticketsplease guard tkt/prototype-physical-implementation-frontier` reports no scope escape.

**Proposal (draft public boundary — awaiting Tom's review of the exact commit).** New `pub(crate)` surface in `frontier.rs`: the `frontier` module; the `PhysicalImplementationProvider` trait; `ImplementationContext`; `enumerate_frontier`; the proposal types `ImplementationProposal`, `ProposalBody`, `ReservedProposalSeam`, `PhysicalProposalKind`, `TargetApplicability`, `PhysicalCostEstimate`, `PhysicalProviderProvenance`; the derived/outcome types `BoundaryContract`, `BoundaryRequirement`, `BoundaryGuarantee`, `BoundaryAvailability`, `BoundaryProduction`, `FrontierRegionSubject`, `AdmittedImplementation`, `ImplementationProvenance`, `ImplementationProposalIdentity`, `ImplementationFrontier`, `FrontierRejection`, `FrontierError`. New `pub(crate)` surface in `physical.rs`: `verify_schedule_with_feasibility`, `VerifiedScheduledRegion::canonical_identity`, and the now-`pub(crate)` `pointwise_region`. All are `pub(crate)` (never `pub`): the frontier is a compiler-internal optimizer authority with no cross-crate consumer yet, and the whole physical/scheduling boundary is a draft until Tom accepts the exact interface.

**Proposal (deferred).** Opaque-call, kernel-subprogram, and view bodies remain reserved variants; their typed ABI/effect/aliasing/evidence contracts belong to `implement-opaque-physical-call-providers`. Wiring the frontier into `compile()` and joining covers with per-region frontiers belongs to `prototype-complete-physical-plan-selection`. The cost estimate is a structural placeholder attributed to `tiler.cost.structural.v1`; an analytical/uncertain-estimate cost model with an explicit `Unknown` state is out of scope here.

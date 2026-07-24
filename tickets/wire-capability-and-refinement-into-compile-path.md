---
id: wire-capability-and-refinement-into-compile-path
title: Wire lowering-capability resolution and index-region refinement into the compile path
status: todo
priority: p0
dependencies: [prototype-optimizer-conformance-gate]
related: []
scopes: [implementation/compiler, implementation/ir]
shared_scopes: []
paths: []
tags: [implementation, optimizer, capability, milestone-0b]
---
Wire `capability` (lowering-capability resolution) and `legality` (checked index-region refinement) into the ordinary `compile()` path, so an externally registered *lowering* provider — not only an externally registered *semantic* provider — drives compilation end to end.

`prototype-optimizer-conformance-gate` wired `cover`, `frontier`, `selection`, and `fusion_legality` into `compile()` and proved an externally registered semantic operation set through the ordinary path. It did not wire `capability`/`legality`, and left `request::CompilerCapabilitySnapshot` as two named `LoweringProviderIdentity` fields (`materialized_serial_sum`, `fused_serial_sum`) that the artifact plan reads directly. That two-field snapshot is the last proof-only candidate list on the compile path: lowering provenance is a compile-time constant rather than a registry resolution.

**What this ticket must produce**

1. Replace `CompilerCapabilitySnapshot`'s two named fields with a frozen `capability::FrozenLoweringCapabilityRegistry` plus the `tiler_ir::index::FrozenScalarRegistry` it was registered against, carried on `CompilationRequest`. `VerifiedRequestSubject` retains the registry's `CanonicalLoweringRegistryIdentity` (it is `Eq + Ord`; the registry itself is not).
2. Resolve an index-access lowering capability per recognized semantic occurrence through `FrozenLoweringCapabilityRegistry::resolve_index_access`, failing closed on `MissingCapability`/`AmbiguousCapability` with a typed compile error and a `CapabilityResolution`-stage explain record.
3. Derive `ArtifactConstructionPlan::lowering_providers` from the resolved capabilities' `ProviderIdentity`/`LoweringCapabilityRevision` instead of the snapshot constants. `LoweringProviderIdentity { key: &'static str, revision: u32 }` must become an owned identity, and `explain::ProviderRef::lowering` must follow.
4. Run `legality::refine_index_region` per occurrence and retain the resulting `IndexRefinement` as occurrence-bound evidence, with a `KernelRefinement`-stage explain record carrying the refinement identity and the selected provider.
5. Ship a governed lowering-capability registry: a `tiler_ir::index::ScalarRegistryBuilder` populated with the scalar operations the four governed families emit, plus one `IndexAccessLoweringProvider` per family (`tiler.constant-f32`, `tiler.multiply-f32`, `tiler.add-f32`, `tiler.strict-serial-sum-f32`). No such governed scalar registry exists today: every consumer (`tiler-ir` tests, `tiler-compiler::capability` tests, `tiler-reference` tests) builds its own ad-hoc one.

**The measured blocker that shapes the design**

**Fact (inspected source, `crates/tiler-ir/src/index/mod.rs:76` at `b1c3e9b`).** `MAX_EXHAUSTIVE_PROOF_CELLS = 1_048_576`. `IndexRegionBuilder::build` proves bounds and write ownership by exhaustive enumeration over the domain, so a region whose iteration domain exceeds ~1M points fails closed with `IndexRegionDiagnostic::ProofResourceLimit { resource: ProofResource::Cells, .. }`.

**Inference.** Index-region refinement therefore cannot be an unconditional gate on the compile path. The existing conformance fixtures already cross the bound: `pipeline::tests::infeasible_baseline_does_not_suppress_a_feasible_fused_plan` compiles a `[70_000, 2]` program (140,000 points) and `no_feasible_plan_retains_a_typed_terminal_failure_trace` a `[70_000, 70_000]` one (4.9e9 points). Making refinement mandatory would convert a target-feasibility rejection — and, for the first fixture, a *successful* compilation — into a proof-budget rejection, which is exactly the confusion between hard feasibility and an exhausted analysis budget that `AGENTS.md` forbids.

**Required design consequence.** Capability *resolution* is unconditional and fails closed; index-region *refinement* is exhaustive finite evidence attached when the proof budget affords it, and a `BudgetStop` explain record at `ExplainStage::KernelRefinement` naming `ProofResource::Cells`, its limit, and the required cell count when it does not. The absence of refinement must be a recorded `Unknown`-class gap, never a silent pass and never a rejection of an otherwise valid plan. Preserve `SoundProof`, exhaustive finite evidence, and `Unknown` as distinct classes.

**Closing evidence.** The ticket closes when `compile()` resolves every recognized occurrence's lowering capability from a registry supplied on the request; a conformance test registers its *own* out-of-crate `IndexAccessLoweringProvider` and observes its `ProviderIdentity` in the artifact plan's `lowering_providers`; a missing and an ambiguous capability each produce a distinct typed error with an explain record; and a large-domain fixture records a typed proof-budget stop instead of failing.

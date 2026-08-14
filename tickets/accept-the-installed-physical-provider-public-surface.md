---
id: accept-the-installed-physical-provider-public-surface
title: Accept or revise the installed physical-provider public surface
status: in-progress
priority: p1
dependencies: [drive-an-external-physical-implementation-provider-through-compilation]
related: [accept-the-public-backend-provider-composition-boundary, disclose-offered-and-selected-physical-provider-sets-separately, publish-occurrence-bound-selected-physical-implementation-evidence]
scopes: [contracts/decisions, contracts/optimizer, contracts/foundation, implementation/compiler, implementation/build, research/extensions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [backend-providers, public-boundary, decision, needs-tom]
claimed_from: todo
assignee: worker-provider-surface-acceptance
lease_expires_at: 1786718048
---
## User-visible outcome

Tom accepts or revises the exact included and excluded public surface of `tiler_compiler::physical_provider`, so it stops being a labelled draft and the contracts that describe it can state an accepted boundary.

## Decision boundary

[ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), anchor `the provider registry and its installation method`, routes this to Tom in terms: every concrete public surface still comes to Tom at implementation time under ADR 0075. [`accept-the-public-backend-provider-composition-boundary`](accept-the-public-backend-provider-composition-boundary.md) accepted the *model* and explicitly did not accept any surface.

This node is not research or implementation work. Only Tom closes it.

## The surface, as landed 2026-08-08

**Included — `tiler_compiler::physical_provider`.** `PhysicalImplementationProvider` (trait, two methods); `ImplementationContext` with `subject`, `target_profile`, `target_profile_key`, `numerical_realization`, `baseline`; `BaselineImplementation` with `region` and `cost`; `FrontierRegionSubject` with `role` and `covered_occurrences`; `ImplementationProposal::scheduled_kernel`; `TargetApplicability::{for_targets, target_profile_keys}`; `PhysicalCostEstimate::structural` with its four readers; `ProviderOffer::{proposing, decline, default}`; `DeclinedStrategy::new`; `StrategyDeclineCause` (`#[non_exhaustive]`); `PhysicalProviderProvenance` and its error; `InstalledPhysicalProviders::{governed, installed, identities}`; `PhysicalProviderInstallationError` (`#[non_exhaustive]`); `GOVERNED_PHYSICAL_COST_MODEL_KEY`.

**Included — `tiler_compiler::session`.** `CompileRequest::with_physical_providers`; `PlanAlternative::selected_physical_providers`; `SelectedImplementation` with `provider`, `provider_explain_subject`, `proposal_kind`.

**Added 2026-08-08 by [`disclose-offered-and-selected-physical-provider-sets-separately`](disclose-offered-and-selected-physical-provider-sets-separately.md) — ADR 0090 item 5's *offered* accessor, which the routing sentence quoted above already names.** The included set gains exactly one item, `Compilation::offered_physical_providers(&self) -> &[ProviderIdentity]`, reporting the governed provider first then the caller's in installation order. **The change is additive**, in ADR 0075's sense that no existing item's signature, variant set, or behaviour moved: it is a new inherent method on an existing `pub struct` whose fields are private, so no caller can be broken by it. Nothing else on the boundary moved — `Compilation::offered_providers` keeps its name, its signature, and its lowering-only behaviour, and its doc comment was corrected to describe that behaviour rather than the whole environment it never reported.

**Excluded from the same landing, by reason.** `InstalledPhysicalProviders::offered_identities` stays `pub(crate)`: it and the already-included `identities` answer two different questions, and publishing both invites a caller to read the caller-installed set as the compilation's environment — which is the conflation item 5 exists to remove. A caller wanting the compilation's environment reads it off the compilation. Also excluded: any physical row in the artifact's `CompilationEnvironment`, which is built from `Compilation::offered_providers` alone and is a separate subject owned where the artifact type is defined.

**Excluded, each by a stated reason rather than by omission.** `ProposalBody` and its subprogram, opaque-call, and reserved-view variants; `KernelSubprogram` and `SubprogramStage`; `SemanticStage` and `SemanticMemberId`; `RegionWrite`; `VerifiedTargetRequest`; `PhysicalCostEstimate::new`; `FrontierRegionSubject`'s constructors and `semantic_members`; `enumerate_frontier`, `PhysicalAuthorities`, and `GovernedPhysicalProvider`; every removal or reordering of the governed provider. Four of these are pinned by `compile_fail` doctests carrying exact error codes in the module documentation.

## The questions that are genuinely Tom's

**The heading previously read "The three questions that are genuinely Tom's" and is quoted here so a grep for that string lands in this note rather than proving the count still stands.** A fourth was added 2026-08-08 with the offered accessor.

1. **Is the additive rule right?** Installing adds to the governed provider and cannot displace it, deliberately unlike `with_capabilities`, which replaces the lowering registry. The ground is that two lowering claimants are a contradiction while two physical implementations are alternatives. What would argue the other way is a caller wanting a compilation the governed provider does not participate in at all.
2. **Is `baseline` the right shape for what a provider reads?** It hands back this host's own spelling for the provider to specialize, which is what makes the seam usable given that the request-subject binding is host-owned — and which also means the seam supports specializing a spelling and not contributing a new region shape. The alternative, exposing the five facts the binding compares, was rejected for creating a second derivation of one answer.
3. **Is `scheduled_kernel` the right restriction?** A caller may propose one body variant. The subprogram and opaque-call exclusions each have an independent reason, so accepting one does not accept the other.
4. **Is the offered pair's naming asymmetry acceptable, or should `offered_providers` be renamed?** The pair reads `offered_providers` / `offered_physical_providers`, so the lowering half is the unqualified one and the physical half is qualified — a reader meeting only the first can still take it for the whole environment, which the doc comment now denies but the name does not. Renaming it `offered_lowering_providers` would make the pair symmetric and self-describing. It is a **breaking** change to a name Tom already accepted on 2026-07-28 under [ADR 0085](../docs/decisions/0085-admit-tiler-build-as-the-build-time-orchestrator.md), with `crates/tiler-build/src/plan_artifact.rs` as a live consumer, which is why the landing documented the asymmetry rather than removing it: an accepted name is not an implementing agent's to change. Answering "keep it" closes the question at no cost; answering "rename it" is a small mechanical change across one consumer.

## Recommendation

Accept the additive installation rule, the `baseline` subject, and the `scheduled_kernel` restriction as built. They preserve the governed provider, keep one host-owned derivation, and expose only the body variant the external-provider vertical actually exercised. Rename `offered_providers` to `offered_lowering_providers` in the same acceptance sweep: this pre-production tree has one production in-workspace consumer (`crates/tiler-build/src/plan_artifact.rs`), and the symmetric pair prevents the old completeness misreading from returning through the shorter name. **Strongest counterpoint:** that final rename reopens a name Tom already accepted under ADR 0085; keeping it would avoid churn and the corrected documentation is now truthful. Tom should answer the four questions independently even if he accepts the first three as a block.

## Accepted — 2026-08-11

**Decision.** Tom accepted all four recommended answers in the Codex coordination thread by replying `sounds good, accept` to the coordinator's decision packet. The relay source is Tom's direct response in that thread. This moves the ticket to `todo` for the mechanical rename and acceptance-label sweep; the accepted provider semantics do not wait on those edits.

1. **Installation is additive.** `InstalledPhysicalProviders` always enumerates the governed provider and then the caller-installed providers. Installation order remains reporting order and never precedence. No public spelling removes, replaces, or reorders the governed provider. This asymmetry with `with_capabilities` is accepted: competing physical implementations are alternatives, while competing lowering authorities for one occurrence are contradictory semantic claims.
2. **`ImplementationContext::baseline` is the accepted construction seam.** An installed provider specializes the host's own verified spelling rather than reconstructing the request-subject binding from separately exposed region identity, iteration shape, scalar program, semantic members, and access map. The present seam intentionally supports schedule specialization and does not authorize a provider to contribute a new region shape.
3. **`ImplementationProposal::scheduled_kernel` is the only accepted public body constructor.** `ProposalBody`, `KernelSubprogram`, `SubprogramStage`, opaque-call registration and proposal bodies, reserved views, `SemanticStage`, `SemanticMemberId`, `RegionWrite`, `VerifiedTargetRequest`, region-subject constructors and semantic members, frontier enumeration, `PhysicalAuthorities`, and `GovernedPhysicalProvider` remain private. A broader body family requires its own evidence and public-boundary decision.
4. **Rename `Compilation::offered_providers` to `Compilation::offered_lowering_providers`.** `Compilation::offered_physical_providers` remains unchanged. The symmetric pair names the two authority environments explicitly. Update the one production consumer in `crates/tiler-build/src/plan_artifact.rs`, every source/test/document reference, and the accepted ADR 0085 surface record in one sweep; do not retain a deprecated alias in this pre-production tree.

The remainder of the ticket's landed included surface is accepted exactly as listed: provider trait and provenance, bounded implementation context and baseline, scheduled-kernel proposal, target applicability, governed structural-cost estimate and key, proposals and typed declines, installed-provider environment and errors, compile-request installation, offered/selected physical-provider disclosure, and selected-implementation provenance.

The accepted exclusions remain exclusions. In particular, this decision does not publish physical rows into the artifact `CompilationEnvironment`, let providers choose a cost model, disclose graph-local semantic members, bypass host verification or hard feasibility, allow replacement of the governed provider, or broaden the seam from specialization to new region-shape authorship.

No identity, schema, planning meaning, artifact bytes, or provider-selection ordering changes with the rename and label sweep.

**Strongest counterpoints accepted with the decision.** Returning a baseline constrains the seam to specialization, and a future provider may need new region shapes; that is a separate, broader authority rather than a reason to expose the five binding facts prematurely. Renaming the lowering accessor reopens an ADR 0085 name whose documentation is now truthful, but the small in-workspace churn is preferred to retaining an objectively asymmetric pair that repeatedly reads as a complete environment.

## Fact audit — 2026-08-14 at `a660ed618446ade55234993b835e75e26d44921c`

The implementing worker re-read this ticket, its completed dependency, ADRs 0075, 0078, 0085, and 0090, the complete provider module and session construction, the build consumer, and the compiler integration fixtures before editing. Every premise below is verified; none is false or changes this ticket's purpose.

1. **Verified — included surface.** The public re-export module exposes the exact provider vocabulary listed above; the trait and its accepted constructors/readers are `pub`, and the session exposes installation plus offered/selected physical disclosure. Reproduce with `rg -n 'pub trait PhysicalImplementationProvider|pub fn (with_physical_providers|offered_physical_providers|selected_physical_providers)|pub fn (baseline|scheduled_kernel)' crates/tiler-compiler/src`.
2. **Verified — exclusions.** `InstalledPhysicalProviders::offered_identities` and the subject's `semantic_members` are `pub(crate)`; `FrontierError`, `PhysicalAuthorities`, and `GovernedPhysicalProvider` are crate-private. The module's four `compile_fail` fixtures pin exact E0624/E0603 failures. Reproduce with `rg -n 'pub\(crate\) fn (offered_identities|semantic_members)|pub\(crate\) (enum FrontierError|struct PhysicalAuthorities|struct GovernedPhysicalProvider)|compile_fail,E0(603|624)' crates/tiler-compiler/src`.
3. **Verified — distinct environments.** The old `Compilation::offered_providers` is populated only from `capabilities.0.lowering().providers()`, while the physical accessor is populated from `physical.offered_identities()`. Renaming the former qualifies its subject and does not widen either population.
4. **Verified — consumer population and break.** `crates/tiler-build/src/plan_artifact.rs` is the sole production consumer; the other Rust references are the defining session code and the `caller_target_profile` and `external_physical_provider` tests. Reproduce with `rg -n 'offered_providers' crates` at this base. Removing the old public method is deliberately source-breaking and no alias is authorized.
5. **Verified — consequence boundary.** The accessor rename changes no value construction, identity encoding, schema, artifact bytes, provider ordering, plan selection, provider sink, or calibration. The artifact environment remains lowering-only and adding a physical row remains outside this ticket.
6. **Verified — documentation population.** Live provider-specific draft language existed in the module, ADRs 0078 and 0090, the optimizer and operation-extension contracts, and the glossary. The backend-composition research record retains dated old spellings as historical evidence, so its new correction distinguishes those subjects from the live API rather than rewriting measurements made at older commits. Unrelated target and runtime-adapter draft boundaries remain untouched.
7. **Verified — evidence and dependency.** The dependency is `done`; its nine-test integration fixture is a separate compilation unit that installs a provider through the ordinary compile path and distinguishes offered from selected identities. The retained out-of-tree spike is a bounded separate-workspace Measurement, not a portability guarantee.
8. **Verified as recorded provenance.** Tom accepted all four recommended answers on 2026-08-11 in the Codex coordination thread. This ticket records his direct reply and the exact included and excluded sets it accepted.

## Implementation evidence — 2026-08-14

- `Compilation::offered_lowering_providers` replaces the old method and backing-field spelling with no alias. The compiler, its integration fixtures, and the sole production build consumer use the qualified name; `rg -n 'offered_providers' crates` returns no match.
- The module documentation and live governing contracts now state the accepted boundary. Dated ADR/research passages retain old spellings and pre-acceptance claims as historical evidence, with 2026-08-11 corrections making their present disposition explicit. Unrelated target-profile and runtime-adapter draft boundaries are unchanged.
- **Production-subject perturbation.** Temporarily restoring only `pub fn offered_providers` made `cargo check -p tiler-build` fail E0599 at `plan_artifact.rs` with `no method named 'offered_lowering_providers'` and a suggestion for the restored old name. Restoring the accepted method made the check green.
- Focused evidence passed: 22 tests across `caller_target_profile` and `external_physical_provider`, all 93 `tiler-build` tests, all-target checks for the two touched packages, Clippy with warnings denied, rustdoc with warnings denied, and all 13 `tiler-compiler` doctests including the four provider-boundary compile-fail cases.
- `make check` passed: citation and formatting populations, workspace all-target check and Clippy, 3,613 nextest tests with 8 skipped, and the workspace doctest population. `tkt lint`, `git diff --check`, and exact-base `tkt guard` are the final branch checks.

## Closes when

The accepted rename lands without a compatibility alias; the module documentation and governing contracts state the accepted included and excluded boundary rather than a draft; acceptance provenance remains recorded; and targeted compiler/build tests plus repository publication gates pass.

## Graph maintenance

- Only Tom approves or revises. After his answer, the implementing agent records it durably, applies every consequence, runs the checks, and closes this node.
- A revision that changes what may be proposed is an implementation change rather than a prose one; file it rather than editing the record to match a tree that has not moved.

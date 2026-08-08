---
id: drive-an-external-physical-implementation-provider-through-compilation
title: Drive an external physical implementation provider through compilation
status: in-progress
priority: p1
dependencies: [accept-the-public-backend-provider-composition-boundary]
related: [prototype-complete-physical-plan-selection, wire-capability-and-refinement-into-compile-path]
scopes: [implementation/compiler, implementation/ir, contracts/optimizer, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, compiler]
claimed_from: todo
assignee: coord
lease_expires_at: 1786178177
---
## User-visible outcome

An out-of-crate caller can install a physical implementation provider into the ordinary compiler session, have its candidates reverified and considered additively, and observe exact selected-provider provenance in the resulting plan and explain output.

## Implementation keys

- Promote only the exact physical-provider facade accepted by the composition ADR.
- Let providers propose bodies, applicability, and estimates through bounded writers; derive provider identity from registration and derive resource/boundary facts from verified output.
- Retain several valid providers' implementations side by side for cost-based selection.
- Preserve the asymmetry with lowering: two lowering authorities for one occurrence are ambiguous, while two physical implementations of one verified region are alternatives.
- Reject malformed provider output as a provider/compiler defect rather than silently treating it as an empty offer.
- Keep empty offer, hard rejection, unknown analysis, provider defect, and cost disadvantage distinct in explain output.
- Add an out-of-crate compile fixture and perturb installation, identity, region coverage, target applicability, and verifier bypass attempts.
- Review the exact public trait/module/session boundary with Tom before acceptance.

## Closes when

An external provider reaches `enumerate_frontier` through `session::compile`, the selected plan records its non-forgeable identity, every negative control fails for the intended reason, targeted nextest and Clippy pass, and one final `make full` passes for the batch.

## What blocks this today

Measured by `prototype-a-forkless-custom-metal-physical-provider` at commit `7b1e3a7e15b09dd3ea65c88759699655c462be4a`, with retained compile-fail evidence at [`spikes/extensions/forkless-physical-provider/`](../spikes/extensions/forkless-physical-provider/README.md). Two independent changes are needed, and the second is the one a reader is likely to miss.

Visibility: `mod frontier;` is declared private in `crates/tiler-compiler/src/lib.rs`, and the closure a provider needs reaches four more private modules — `request::VerifiedTargetRequest`, `region::{SemanticMemberId, SemanticStage}`, `physical::{pointwise_region, verify_schedule_with_feasibility, RegionWrite}`, and `pipeline::compile`. The governed cost-model key `tiler.cost.structural.v1` is a private constant (`const COST_MODEL_KEY` in `frontier.rs`) and the only admissible one, so it needs a public spelling too.

Installation: publishing the trait alone would still leave a provider uninstallable, and the internal `pub(crate) struct CompilationRequest` (`request.rs`) carries no provider field. The out-of-crate compile fixture this ticket asks for should keep the spike's pairing so it states an asymmetry rather than a bare absence.

**Anchors refreshed 2026-08-05 at base `51e9374a` by [`audit-backend-authoring-against-all-thirteen-responsibilities`](audit-backend-authoring-against-all-thirteen-responsibilities.md); the blockers themselves are unchanged.** This section previously cited a "hardcoded one-element literal" in `pipeline/planning.rs`. That literal no longer exists on the production path: the provider list and the opaque-call registry are composed into `pub(crate) struct PhysicalAuthorities` (`frontier.rs`), installed as `PhysicalAuthorities::governed()` from `pipeline::compile` and consumed at the `enumerate_frontier(verified, &subject, physical.providers(), physical.calls())` call in `pipeline/planning.rs`. That is an internal composition improvement and moves none of the three obligations this ticket exists to discharge — visibility, installation from outside the crate, and observability all still fail, the last where `let offered_providers: Arc<[ProviderIdentity]> = Arc::from(capabilities.0.lowering().providers());` in `session::compile` populates the offered set from the lowering registry alone. Read the change as narrowing what has to be built, not as partial completion.

The spike also establishes what does *not* need work: `tiler-metal` is reusable unchanged by an out-of-tree provider, the *payload* a proposal body carries is already public (`tiler_ir::schedule::ScheduledRegion`), and the schedule axis a specialization varies is free under the intrinsic verifier and folded into canonical identity.

## Fact audit at base `c81f9257`, 2026-08-08

**Every line number this section carried was stale, and they drifted in both directions.** The substance survived in every case but two, which are corrected above rather than restated. Re-read in full at `c81f9257`; the citations above are now searchable anchors rather than line numbers, per `AGENTS.md`.

| Claim as written | Verdict | Evidence at `c81f9257` |
| --- | --- | --- |
| `lib.rs:24` declares `mod frontier;` | **verified** | `mod frontier;` is line 24 exactly — the one citation in this section that had not moved |
| four more private modules at `lib.rs:34`, `:35`, `:38`, `:39` | **false line numbers, verified substance** | actual: `mod request;` 40 (+6), `mod region;` 39 (+4), `mod physical;` 35 (−3), `mod pipeline;` 36 (−3). All four are private; every named item is `pub(crate)` |
| the closure is `VerifiedTargetRequest`, `SemanticMemberId`, `pointwise_region`, `verify_schedule_with_feasibility`, `pipeline::compile` | **imprecise, and incomplete** | a body also needs `region::SemanticStage` (the planner's attribution atom, which `SemanticMemberId` is only half of) and `physical::RegionWrite`, both `pub(crate)`. A worker taking the list as complete would have promoted a set that does not compile |
| `tiler.cost.structural.v1` is a private constant at `frontier.rs:100` | **imprecise line, verified substance** | `const COST_MODEL_KEY` is line 103; line 100 is the first line of its doc comment. Private, and `enumerate_frontier` returns `FrontierError::MalformedCostProvenance` for any other key. **Not stated by the ticket:** the same literal is spelled a second time as `const STRUCTURAL_COST_MODEL_KEY` in `pipeline.rs`, also private |
| `CompilationRequest` at `request.rs:1024` carries no provider field | **false line, verified substance** | `pub(crate) struct CompilationRequest<'a>` is line 1217; its six fields are `program`, `shape_environment`, `numerical_contracts`, `budgets`, `target_profiles`, `capabilities` |
| `PhysicalAuthorities` at `frontier.rs:2893` | **false line, verified substance** | line 2955 |
| installed as `PhysicalAuthorities::governed()` at `pipeline.rs:604` | **false line, verified substance** | line 625, inside `pipeline::compile` at 620 |
| consumed at `pipeline/planning.rs:292` | **false line, verified substance, and the dangerous kind** | the consumption is line 310. Line 292 resolves to plausible neighbouring code — an argument of the `FrontierRegionSubject::reading_intermediates` call — so a reader checking the citation would find compiling code about the same subject and conclude it was right |
| observability fails at `session.rs:2092-2093` | **false line, verified substance** | lines 2208–2209 |
| "the proposal body type is already public" | **false** | `pub(crate) enum ProposalBody` (`frontier.rs`). What the spike measured is that its *payload*, `tiler_ir::schedule::ScheduledRegion`, is public. Following this as written would have concluded `ProposalBody` needed no work; the landing instead keeps it private on purpose and exposes `ImplementationProposal::scheduled_kernel` |
| the fixture should pair a compile-fail case for the absent installation method with a compiling `with_capabilities` case | **stale by construction** | the instruction describes the pre-landing tree: this ticket's own deliverable is that installation method, so a compile-fail case for its absence would have to be deleted in the same commit that satisfies it. The landed fixture keeps the *pairing discipline* and moves the negative half onto the bypasses that must stay closed — the verified request, an ungoverned cost model, the region subject's members, and `enumerate_frontier` itself |
| dependency `accept-the-public-backend-provider-composition-boundary` | **verified** | `status: done`, Tom accepted ADR 0090 in full on 2026-07-31 |

## Graph maintenance

- Unblock payload production and final provider composition only through the accepted public seam.
- Keep semantic-equivalence trust limitations explicit; structural verification cannot prove arbitrary replacement mathematics.
- Update ADR 0078's implementation status and governed seam inventory only after the path is genuinely external and exercised. **This ticket does not hold `contracts/decisions`**, so the ADR 0078 and ADR 0090 status updates are carried by [`record-the-landed-physical-provider-seam-in-adrs-0078-and-0090`](record-the-landed-physical-provider-seam-in-adrs-0078-and-0090.md).
- The offered-versus-selected split stays with [`disclose-offered-and-selected-physical-provider-sets-separately`](disclose-offered-and-selected-physical-provider-sets-separately.md), which depends on this node. This landing supplies the **selected** half only (`PlanAlternative::selected_physical_providers`); `Compilation::offered_providers` is still populated from the lowering registry alone and reading an installed provider's absence from it as "never installed" remains the conflation that ticket exists to prevent.

## Outcome (2026-08-08)

**An out-of-crate provider reaches `enumerate_frontier` through `session::compile` and the retained plan records its host-stamped identity.** `tiler_compiler::physical_provider` is the seam, `CompileRequest::with_physical_providers` installs it, `pipeline::compile_with_physical_providers` is the one production entry that varies the authorities, and `PlanAlternative::selected_physical_providers` discloses the selection. `crates/tiler-compiler/tests/external_physical_provider.rs` is the out-of-crate fixture: seven tests defining a provider against the public surface only.

**The design decision, and the alternative rejected.** An installed provider builds its body by specializing this host's own baseline spelling (`ImplementationContext::baseline`), which is derived by one shared authority the governed provider also calls (`govern_spelling` in `frontier.rs`). The rejected alternative was to expose the raw facts the request-subject binding compares — region identity, iteration shape, scalar program, semantic members, access map — and let a provider assemble a matching region itself. That is strictly worse: those facts *are* what the binding compares, so handing them over piecemeal is the same information in a form two derivations can drift apart in, and it would have required exporting `SemanticStage`, a graph-local authoring coordinate this boundary deliberately keeps out of a provider's decision and out of the trace.

**The limitation that decision makes visible, stated rather than absorbed.** Because the binding is host-owned, the only freedom an installed provider has today is the schedule axes the intrinsic verifier leaves free. Proposing a region over member sets the request-level recognizer did not pre-compute still fails the binding, so *specializing a spelling* is the operation the seam supports and *contributing a new region shape* is not. That is the same wall the optimizer contract's stage-8 paragraph names, not a second one, and it is ADR 0090's open "where the additivity line falls" question in concrete form.

---
id: prototype-a-forkless-custom-metal-physical-provider
title: Prototype a forkless custom Metal physical provider
status: todo
priority: p1
dependencies: []
related: [draft-public-extension-seam-ownership-adr, prototype-complete-physical-plan-selection, implement-opaque-physical-call-providers]
scopes: [research/extensions]
shared_scopes: [research/program-planning, project/tickets]
paths: []
tags: [backend-providers, pluggability, metal, spike]
---
## User-visible outcome

A retained executable spike demonstrates whether a separately authored, statically linked provider can contribute one specialized Metal physical implementation alongside Tiler's governed provider without forking or replacing `tiler-metal`.

## Why this slice exists

The internal physical frontier is additive, but `PhysicalImplementationProvider` is crate-private and no backend-defined provider reaches the ordinary compile path. A custom provider that must replace the entire backend would not meet the fork-avoidance goal.

## Implementation keys

- Keep the spike private and bounded; it is evidence for a later public boundary, not an implicit promotion.
- Choose one real supported region for which both the governed provider and the custom provider can offer correct implementations.
- Give the custom provider a distinct stable identity and output-affecting revision.
- Drive both alternatives through ordinary schedule, feasibility, structured-KIR, and selected-plan verification; the custom provider may propose but may not stamp provenance, resource requirements, or boundary guarantees.
- Demonstrate that registration order does not select the winner, lowering-authority contention remains an error, and physical alternatives remain additive.
- Reuse standard Metal emission and runtime behavior rather than copying it; record any interface that prevents this reuse.
- Perturb the provider output into a malformed schedule or mismatched region and observe the verifier fail.
- Preserve the spike harness, exact invocation, inputs, and result fixture under `spikes/`.
- Do not edit production crates in this spike. File any evidence-backed production blocker as a separate ticket with its own scope and public-boundary review where required.

## Closes when

The spike proves or falsifies partial Metal-provider composition, records the exact private surfaces it needed, distinguishes verified guarantees from trusted semantic-equivalence claims, and leaves no production API or crate admission behind.

## Graph maintenance

- Feed the measured interface and reuse requirements into `specify-the-consumer-neutral-backend-provider-composition-contract`.
- If reuse fails because Metal emission or runtime ownership is inseparable, file the smallest evidence-backed split ticket rather than widening this spike.
- Do not treat opaque physical-call registration as backend-provider registration; keep the tickets related but distinct.

## Outcome

The spike is [`spikes/extensions/forkless-physical-provider/`](../spikes/extensions/forkless-physical-provider/README.md), run by hand with `cargo nextest run --workspace` from its own directory. Seven tests; no Apple toolchain, device, or dispatch involved. Retained evidence and exact toolchain provenance are in [`results/2026-07-31-macos-arm64.json`](../spikes/extensions/forkless-physical-provider/results/2026-07-31-macos-arm64.json). Subject commit `7b1e3a7e15b09dd3ea65c88759699655c462be4a`; no production crate was edited.

**The question is falsified, and the falsification is localized.** A separately authored crate cannot contribute a physical implementation alongside the governed provider today. Nothing in `tiler-metal` is the reason.

**Fact — two independent blockers, both in `tiler-compiler`.** First, `crates/tiler-compiler/src/lib.rs:19` declares `mod frontier;` private, so the ten items a provider must name are unreachable as a set: `PhysicalImplementationProvider` (`frontier.rs:787`), `ImplementationContext` (`754`), `ImplementationProposal` + `::new` (`726`, `735`), `ProposalBody` (`168`), `TargetApplicability` + `::for_targets` (`210`, `224`), `PhysicalCostEstimate` + `::structural` (`260`, `288`), `PhysicalProviderProvenance` + `::new` and its error (`634`, `641`, `650`), `FrontierRegionSubject` (`807`), `enumerate_frontier` (`1429`), and the private cost-model constant `tiler.cost.structural.v1` (`80`) — which is the only key that is not a hard `FrontierError::MalformedCostProvenance`, and has no public spelling. Its transitive closure is private too: `request::VerifiedTargetRequest` (`request.rs:809`), `region::SemanticMemberId` (`region.rs:123`), `physical::{pointwise_region, reduction_region, fused_region}` (`physical.rs:227`, `309`, `397`), `physical::verify_schedule_with_feasibility` (`642`), and `pipeline::compile` (`pipeline.rs:580`). Not blocking, and easy to add to the list by mistake: `request::{TargetProfile, TargetProfileKey}` are `pub(crate) use` re-exports (`request.rs:34`) of types that are public in `tiler_compiler::target`.

**Fact — publishing the trait alone would not compose.** The provider array is a hardcoded one-element literal at `crates/tiler-compiler/src/pipeline/planning.rs:171`, and the internal `CompilationRequest` (`request.rs:542`) carries no provider field for a public method to write into. This is the same asymmetry ADR 0078 item 4 named for *lowering* providers — buildable but not installable — and closed with `CompileRequest::with_capabilities`; it is still open for physical providers. The compile-fail fixture is paired with a compiling contrast that installs a lowering authority through that exact method, so the finding reads as an asymmetry rather than as "nothing installs anything".

**Fact — nor is the failure observable.** `Compilation::offered_providers` (`session.rs:710`) is populated from the lowering registry (`session.rs:1443`); the governed physical provider's own identity `tiler/prototype-serial-sum-physical` does not appear in it, and no public type carries physical-provider provenance.

**Measurement — stock Metal emission needs no change.** `acme-provider` does not depend on `tiler-metal`. The probe lowers with public `tiler_ir::kernel::lower_scheduled_region` and emits with public `tiler_metal::emit::emit_translation_unit`, and the flushing realization the provider declares passes `require_declared_realization` against the measured Apple facts. No interface prevents reuse, so the graph-maintenance split ticket that clause anticipated is not needed.

**Measurement — the specialization is real and identity-bearing.** `threads_per_workgroup` is free under the intrinsic verifier (`tiler-ir/src/schedule/builder.rs:288`) and folded into `CanonicalScheduledRegionIdentity` (`model.rs:892`). A 32-thread and the governed 1-thread implementation of one index region have distinct identities and emit byte-identical kernel bodies under distinct entry-point symbols, so two alternatives would not collide in one translation unit; launch geometry is carried by the dispatch, not the emitted statements.

**Measurement — the perturbation fails closed.** `launch.grid_threads` one short of the iteration domain is rejected by `ScheduledRegionBuilder::build` with exactly `[LaunchCoverage]`, rule `launch-coverage` — the same call the frontier makes on a provider's body (`physical.rs:652`) and the diagnostic it maps onto `PhysicalError::Intrinsic` (`684`). A companion test pins the perturbation to that one field so the rejection is evidence about launch coverage and not a second illegal difference.

**Inference — the request-subject binding already permits a specialized alternative.** `verify_region_subject_binding` (`physical.rs:700`) compares region id, iteration shape, scalar program, semantic members, and access map, and says nothing about `KernelSchedule`. So it constrains what a region means rather than how it is scheduled. Read from source; the binding is private and was not exercised.

**Unreachable, stated rather than skipped.** Three implementation keys cannot be exercised from outside the crate. *Registration order does not select the winner* needs two providers in one `enumerate_frontier` call; the in-crate forward/reverse test at `frontier.rs:2511` is evidence about the algorithm, not about a cross-crate provider reaching it, and for the probe's program the retained non-dominated set is a single fused alternative. *Lowering-authority contention* belongs to the capability registry, a different seam — frontier alternatives are additive and have no contention to demonstrate, which is what this ticket's own third graph-maintenance clause warns against conflating. *A provider may propose but not stamp provenance, resources, or guarantees* is true by construction (`ImplementationProposal` carries only body, applicability, and cost, `frontier.rs:726`) but read rather than exercised.

**Measurement boundary.** One compiler, one host, one commit. The `.stderr` goldens render diagnostics and are not a stability guarantee; they are retained so that a public `frontier` or an added installation method turns the suite red rather than leaving a stale conclusion in the corpus. A red run reopens the question; it is not licence to bless a golden. Nothing here measures runtime behaviour, cost comparability between two providers' estimates, or what a second provider would do to selection.

**No new ticket filed.** The two blockers are already owned: `drive-an-external-physical-implementation-provider-through-compilation` is the production seam, gated behind `accept-the-public-backend-provider-composition-boundary`. Both now cite this spike. Adding a third would duplicate an existing outcome.

**Catalog note.** `spikes/README.md`'s generated experiment catalog is in the `contracts/navigation` scope, which this ticket does not hold, so its entry for the extensions experiment was not amended. The sub-spike is described and linked from `spikes/extensions/README.md`, which the catalog already points at, and that README's `entrypoints` now names the new manifest.

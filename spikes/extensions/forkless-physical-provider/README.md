# Forkless custom Metal physical provider

**Question.** Can a separately authored, statically linked provider contribute one specialized Metal physical implementation alongside Tiler's governed provider, without forking `tiler-compiler` and without replacing `tiler-metal`?

**Answer at `7b1e3a7`: no.** The result is a falsification, and the useful part is *where* it falsifies. Nothing in `tiler-metal` is in the way — a third party reuses stock Metal emission unchanged, and the implementation body it wants to propose is fully constructible from the public `tiler_ir::schedule` surface. Everything in the way is one crate's registration seam, and it fails in two independent places: the frontier vocabulary is behind a private module, and even if it were not, there is no method that installs a physical provider into a compilation.

## Run it

From this directory:

```sh
cargo nextest run --workspace
```

Seven tests, no host toolchain required beyond the repository's pinned Rust: nothing here invokes `xcrun`, allocates a device, or dispatches. `rustup` resolves the repository pin by directory ancestry, so no selector is needed and this spike deliberately carries no `rust-toolchain.toml` of its own.

`cargo nextest run -E 'test(forkless_provider_surface_diagnostics)'` runs only the compile-fail evidence, which is the slow half.

## What is here

`acme-provider/` is the separately authored crate. It depends on `tiler-compiler` and `tiler-ir` exactly as an out-of-tree crate would — by path, no feature flag, no `#[path]` include, no private access. It offers one specialized implementation of the bounded profile's pointwise region: the same index region, the same scalar program, the same numerical realization, and a 32-thread workgroup where the governed provider emits one thread per workgroup (`crates/tiler-compiler/src/physical.rs:495`). It also builds the governed-shaped contrast beside it, and a deliberately malformed variant.

`probe/` drives that provider as far as the public surface reaches. `tests/composition.rs` holds the runtime claims; `tests/ui/` holds the compile-fail evidence and its compiling contrasts.

`results/` records the exact toolchain, the blocking surfaces with their declaration sites, and the falsification runs.

## What blocks it

**Fact — the frontier vocabulary is private.** `crates/tiler-compiler/src/lib.rs:19` declares `mod frontier;`, so the module gate fires before item visibility is consulted, and the nine items a `PhysicalImplementationProvider` implementation must name are unreachable as a set: the trait (`frontier.rs:787`), `ImplementationContext` and `ImplementationProposal` (its two method signatures), `ProposalBody`, `TargetApplicability`, `PhysicalCostEstimate`, `PhysicalProviderProvenance`, `FrontierRegionSubject`, and `enumerate_frontier`. A tenth item is easy to miss: the governed cost-model key `tiler.cost.structural.v1` is a private constant (`frontier.rs:80`), and a proposal attributing its estimate to any other key is a hard `FrontierError::MalformedCostProvenance` — so the only admissible cost has no public spelling either. [`frontier_provider_vocabulary_is_private.rs`](probe/tests/ui/fail/frontier_provider_vocabulary_is_private.rs) is the diagnostic.

**Fact — the transitive closure is private too.** A provider must read its context and bind its body to a region subject, and those live behind four more private modules: `request::VerifiedTargetRequest`, `region::SemanticMemberId`, `physical::{pointwise_region, verify_schedule_with_feasibility}`, and `pipeline::compile`. [`provider_inputs_are_private.rs`](probe/tests/ui/fail/provider_inputs_are_private.rs). One near-miss is worth naming so a later reader does not add it to the list: `request::{TargetProfile, TargetProfileKey}` are `pub(crate) use` re-exports (`request.rs:34`) of types that *are* public in `tiler_compiler::target`, so they are reachable.

**Fact — and making all of that public would still not compose.** The provider array is a hardcoded one-element literal, `let providers: [&dyn PhysicalImplementationProvider; 1] = [&GovernedPhysicalProvider];` at `crates/tiler-compiler/src/pipeline/planning.rs:171`, and the internal `CompilationRequest` (`request.rs:542`) carries no provider field for a public method to write into. This is the second, independent blocker, and it is the same shape as the asymmetry ADR 0078 item 4 named for *lowering* providers — everything needed to build one was public and nothing could install one — which that item closed with `CompileRequest::with_capabilities`. [`no_physical_provider_installation_seam.rs`](probe/tests/ui/fail/no_physical_provider_installation_seam.rs) is the missing method; [`lowering_installation_seam_exists.rs`](probe/tests/ui/pass/lowering_installation_seam_exists.rs) is the one that exists, so the finding reads as the asymmetry it is rather than as "nothing installs anything".

**Fact — nor could a third party observe the failure.** `Compilation::offered_providers` is the only provider set the public boundary reports and it is populated from the lowering registry (`session.rs:1443`). The governed physical provider's own identity, `tiler/prototype-serial-sum-physical`, does not appear in it, and no public type carries physical-provider provenance.

## What is *not* in the way

**Measurement — stock Metal emission is reusable unchanged.** `acme-provider` does not depend on `tiler-metal` at all. The probe lowers its region with `tiler_ir::kernel::lower_scheduled_region` and emits with `tiler_metal::emit::emit_translation_unit`, both public, and the flushing realization the provider declares passes `require_declared_realization` against the measured Apple facts. The ticket asked which interface prevents reuse; the answer is none. Emission consumes verified kernels and knows nothing about who proposed them, which is exactly the separation that makes partial composition plausible once registration exists.

**Measurement — the specialization is real, and identity-bearing.** `threads_per_workgroup` is free under the intrinsic verifier — it must equal `launch.threads_per_workgroup` and be non-zero (`tiler-ir/src/schedule/builder.rs:288`) — and it is folded into `CanonicalScheduledRegionIdentity` (`tiler-ir/src/schedule/model.rs:892`). So the 32-thread and 1-thread implementations of one region are additive alternatives with distinct identities rather than one implementation twice. They emit byte-identical kernel bodies under *distinct entry-point symbols*, because the symbol is derived from that identity: two alternatives of one region would not collide in a translation unit holding both, and launch geometry is carried by the dispatch rather than by the emitted statements.

**Inference — the request-subject binding already permits it.** `verify_region_subject_binding` (`physical.rs:700`) compares region id, iteration shape, scalar program, semantic members, and access map against the compiler's own normalized expectation, and says nothing about `KernelSchedule`. So the gate constrains what a region *means* and not how it is *scheduled*, which is the separation a specialized provider needs. This is read from source rather than exercised — the binding is private — and it is labelled an inference for that reason.

**Measurement — a malformed body fails closed.** Perturbing one field, `launch.grid_threads` one short of the iteration domain, is rejected by `ScheduledRegionBuilder::build` with exactly `[LaunchCoverage]`, rule `launch-coverage`. That is the same call the frontier makes on a provider's body (`physical.rs:652`) and the diagnostic it maps onto `PhysicalError::Intrinsic` (`physical.rs:684`). A companion test pins the perturbation to that single field, so the rejection is evidence about launch coverage and not about some second difference that happened to be illegal.

## What this spike cannot answer

Three of the ticket's implementation keys are unreachable from outside the crate, and saying so precisely is part of the result.

*Registration order does not select the winner* needs two providers in one `enumerate_frontier` call. `crates/tiler-compiler/src/frontier.rs:2511` enumerates the same two providers forward and reversed and compares the outcome — but that is an in-crate unit test, and it is evidence about the algorithm, not about whether a cross-crate provider can reach it. For the program the probe compiles, the retained non-dominated set is a single fused alternative, so even the ordering the public surface *does* expose has nothing to order.

*Lowering-authority contention remains an error* belongs to the lowering capability registry, which is a different seam. Frontier alternatives are additive by construction and have no contention to demonstrate; treating the two as one mechanism is what this ticket's own graph-maintenance section warns against.

*A provider may propose but not stamp provenance, resources, or boundary guarantees* is true by construction — `ImplementationProposal` carries only a body, an applicability predicate, and a cost (`frontier.rs:726`), and the frontier stamps and derives the rest — but it is read, not exercised.

## Measurement boundary

These are facts about one compiler, one host, and one commit, recorded in [`results/`](results). The `.stderr` goldens are the *rendering* of a diagnostic, not a stability guarantee: a later rustc could reword `E0603` or `E0599` with nothing in Tiler having changed. They are retained anyway, because their job is to go red when the answer changes — a public `frontier`, or an added installation method, fails this suite instead of quietly leaving a stale conclusion in the corpus. **A red run means the question has been reopened, not that a golden should be blessed.** Refresh one with `TRYBUILD=overwrite` only after deciding the recorded claim still holds, and re-record the toolchain in the same commit.

Nothing here measures runtime behaviour, cost-model comparability between two providers' estimates, or what a second provider would do to plan selection.

## Proving the checks can say no

Both compile-fail claims were run against a case that must fail, on 2026-07-31, by copying the spike outside the repository and rewriting the three `tiler-*` path dependencies to absolute paths:

- replacing `.with_physical_providers(..)` with `.with_capabilities(InstalledCapabilities::governed())` — a method that does exist — turned the installation-seam case into `Expected test case to fail to compile, but it succeeded`;
- replacing the private `tiler_compiler::region` import with the public `tiler_compiler::capability::LoweringFamily` removed that one `E0603` from the actual output and the byte comparison reported `mismatch`.

The copy is reconstructible from the two sentences above and was not retained; the checked-in fixtures are the evidence.

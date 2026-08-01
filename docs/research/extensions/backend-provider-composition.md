---
schema: "tiler-doc/v1"
id: "tiler.research.extensions.backend-provider-composition"
kind: "research"
title: "Consumer-neutral backend-provider composition"
topics: ["extensions", "backends", "pluggability", "identity", "artifacts", "runtime"]
catalog_group: "foundation-semantics-extensions"
research_status: "complete"
disposition: "pending"
implementation_status: "not-started"
evidence_classes: ["primary-source-synthesis", "executable-model", "bounded-measurement"]
informs: ["tiler.contract.architecture", "tiler.contract.operation-extensions", "tiler.contract.artifact-abi", "tiler.contract.cpu-backend"]
ticket: "specify-the-consumer-neutral-backend-provider-composition-contract"
---

# Consumer-neutral backend-provider composition

## What this record is, and what it deliberately is not

This record specifies how statically linked backend components compose from compilation through execution, so that [`draft-the-backend-provider-composition-adr`](../../../tickets/draft-the-backend-provider-composition-adr.md) can consume its decisions one at a time. It accepts nothing. It changes no visibility, no signature, and no behaviour, and nothing in it may be cited as accepted — the [glossary](../../glossary.md#backend-device-and-execution-context-vocabulary)'s Provider row names this ticket as the owner of the open question and says so in the same words.

It is also not a proposal for a `BackendProvider` type. The single most important finding of the two spikes that constrain it is that **no such thing is needed**, because the pieces a backend is made of are already installed, identified, and joined by four different mechanisms at four different times, and the useful contract is the one that says which mechanism owns which subject. A monolithic trait would have to re-mediate edges that already work — and would break one of them, as the elimination in "Eliminated alternatives" below shows in detail.

Every claim carries a label. **Fact** is inspected source at base commit `e6a47d9` or a primary document; **Measurement** is an observation tied to an exact environment and procedure, and here it is always one of the two spikes; **Inference** is derived from stated facts; **Proposal** is a design that is neither accepted nor tested. The Rust in "Interface sketches" is **Proposal** throughout and is presented as a shape to argue with, never as an interface.

## The evidence this record is built on

Two spikes bound the design from opposite ends, and this record synthesizes only what they exercised or what an accepted contract forces.

The [forkless custom Metal physical provider](../../../spikes/extensions/forkless-physical-provider/README.md) asked whether a separately authored crate can contribute one specialized Metal implementation alongside the governed provider. Its answer at commit `7b1e3a7e15b09dd3ea65c88759699655c462be4a` is **no**, and the value is in where it falsifies: nothing in `tiler-metal` is in the way, and the blockers are two independent properties of one crate's registration seam.

The [bounded scalar CPU backend vertical](../../../spikes/target-profiles/scalar-cpu-vertical/README.md) asked the complementary question — whether the accepted target-profile, artifact, and runtime contracts silently encode Metal's execution hierarchy — and carried a materially different backend end to end through the public boundaries. Its answer is *almost nothing does, and the exceptions are nameable*. Its eleven findings are the second half of this record's input.

The vocabulary is the [glossary's backend, device, and execution-context section](../../glossary.md#backend-device-and-execution-context-vocabulary), and this record consumes its terms without redefining any of them. Where this record says *backend* it means the responsibility of translating verified physical work into one representation and declaring the target facts translation depends on; where it says *runtime adapter* it means the component that binds a validated artifact to one live device and execution context. The two are different roles, and a `tiler-<backend>` crate name is correct only where the named role really is backend-owned.

## The responsibility, identity, and lifecycle matrix

This is the record's primary deliverable, and its most load-bearing column is the last one. **Compared** means two instances of the subject are checked for agreement and a mismatch is a typed refusal. **Provenance** means the subject is retained, reported, and folded into a cache or artifact key, and is *never* compared against a counterpart — because it has no counterpart. **Independently selected** means the two sides of a boundary choose the subject separately and no identity relates their choices.

| # | Subject | Owning role | Fixed at | How it is installed today | Identity subject | Disposition |
|---|---|---|---|---|---|---|
| 1 | Semantic authority | frontend / caller | program construction | `SemanticProgramBuilder::try_new(registry.freeze())` | `SemanticAdmissionProvenanceIdentity`, `SemanticRegistrySnapshotIdentity` | Provenance ([ADR 0072](../../decisions/0072-separate-semantic-meaning-from-provider-provenance.md)) |
| 2 | Index/access lowering capability | third party or Tiler | compilation request | `InstalledCapabilities::installed` + `CompileRequest::with_capabilities` | `LoweringCapabilityKey {family, operation, signature, provider}` | **Compared** at resolution; retained as selected-plan provenance |
| 3 | Scalar lowering capability | third party or Tiler | compilation request | same registry pair | same key shape | Registrable and resolved by no compile-path caller |
| 4 | Physical implementation | third party or Tiler | frontier enumeration | **nothing installs one** | `ProviderIdentity` folded into `ImplementationProposalIdentity` | **Additive** — alternatives retained side by side, separated by folded provenance, selected on cost |
| 5 | Opaque call declaration | Tiler (crate-private) | before enumeration | **nothing registers one on the compile path** | `OpaqueCallIdentity` | Named by a proposal, checked against the registration |
| 6 | Target profile | caller (backend facts projected by `tiler-build`) | compilation request | `TargetProfileBuilder` → `TargetProfile` → `TargetRequest` | `TargetProfileKey` **plus exact `canonical_descriptor()` bytes** | **Compared** at load; key and descriptor are separate findings |
| 7 | Backend emitter | backend crate | build time | an ordinary Cargo edge from the producing crate | none of its own | **Independently selected**; the composition contract does not mediate this edge |
| 8 | Build-time orchestration | `tiler-build` | build time | **no indirection at all — statically Metal** | none | Would be **independently selected** if its one neutral seam were reachable |
| 9 | Backend family and representation | backend | payload construction | `BackendKey` / `RepresentationKey` on the payload | the two governed keys | **Compared as a pair** against the loading host's declaration |
| 10 | Payload provenance | backend | payload construction | `PayloadProvenance` on the payload | folded into artifact identity | Provenance |
| 11 | Compiler plan and entry mapping | compiler + `tiler-build` | artifact assembly | `ArtifactProgramBuilder` | `RecordedArtifactProgramIdentity`, entry mapping | **Compared** at bind |
| 12 | Runtime adapter | consumer/backend | run time | **nothing registers one** | none that travels | **Independently selected**; joined only through #6 and #9 |
| 13 | Live device and execution context | runtime adapter | run time | not installable by construction | runtime-cache keys only | **Independently selected**; never travels in an artifact |

**Ten of the thirteen rows already have a mechanism, and that is the record's headline.** The three that do not are row 4, which nothing installs; row 5, which nothing registers on the compile path; and row 8, which has no indirection at all. Row 12's absence of a registry is not a gap in that sense — independent selection *is* its mechanism, and giving it one would be the mistake eliminated below. A reader who takes one thing from this record should take that the question "what should a `BackendProvider` be" was the wrong question, and "what are rows 4, 5, and 8 missing" is the right one.

Eight consequences of that table are what the ADR has to preserve, and each one is a place where a plausible simplification would be wrong.

**A physical provider is the one subject where two claimants are an alternative rather than a contradiction.** *Fact.* The lowering registry resolves a `LoweringCapabilityKey` by filtering on family, operation, and signature only, so two providers matching one occurrence resolve as `AmbiguousCapability` and two revisions of one provider do the same; nothing is last-wins. The frontier is the opposite by construction: `enumerate_frontier` iterates a provider list and retains each admitted proposal, separating them by folded provenance in `ImplementationProposalIdentity`. [ADR 0078](../../decisions/0078-name-the-intended-public-extension-seams.md) item 3 states the asymmetry and warns against generalizing either rule to the other seam, and this record depends on it: exactly one authority may define what an occurrence *means*, and a region may legitimately have several correct *implementations*.

**Producer provenance is never compared against adapter identity, because the adapter has no identity that travels.** *Fact.* The only things a loading host states about itself are one `TargetProfileRef` of governed key plus exact descriptor identity, one `BackendKey`, and one `RepresentationKey` — the device-free `tiler_runtime::load::ExecutionEnvironment`. Nothing in it names which crate, which provider, or which process produced the payload. *Inference.* A composition contract that made the join "the adapter must have been registered by the same provider that produced the payload" would have to invent an identity subject that does not exist and could not survive the process boundary [`join-build-time-producers-to-runtime-adapters-through-artifact-identity`](../../../tickets/join-build-time-producers-to-runtime-adapters-through-artifact-identity.md) exists to prove. The join is governed keys and canonical identities, and the producer's provenance rides along as evidence about how the bytes came to be, never as a matching key.

**The backend-family/representation pair is compared as a pair, and the profile is classified rather than equated.** *Fact.* The glossary states this and gives the reason: a wrong artifact and a stale descriptor must stay different findings. *Inference for the contract.* A provider composition boundary must therefore not collapse "this host cannot execute these bytes" into "this artifact is for another target". They are two refusals with two remedies.

**A specialization is identity-bearing without needing a new identity authority.** *Measurement, forkless spike.* `threads_per_workgroup` is free under the intrinsic verifier — it must equal `launch.threads_per_workgroup` and be nonzero — and it is folded into `CanonicalScheduledRegionIdentity`, so a 32-thread and a 1-thread implementation of one region are distinct identities that emit byte-identical bodies under *distinct entry-point symbols*. *Inference.* "Several providers' implementations retained side by side" needs no addition to the identity model for this case; two alternatives in one translation unit do not collide. That is a narrow claim about one axis, and the record marks it as such under "What this could not establish".

**The emitter edge is a Cargo edge and must stay one.** *Measurement, forkless spike.* `acme-provider` does not depend on `tiler-metal` at all; the probe lowered its region with `tiler_ir::kernel::lower_scheduled_region` and emitted it with `tiler_metal::emit::emit_translation_unit`, both public, and the emitted unit passed `require_declared_realization` against the measured Apple facts. *Inference.* The question "how does a partial provider reuse another backend's pieces" has a concrete answer for the emission piece: it takes an ordinary dependency on `tiler-ir` and the emitter crate, and the composition contract mediates nothing. Routing that edge through a registry would add an indirection whose only effect is to make a compile-time-checkable reuse into a runtime-resolvable one.

**The runtime adapter answers a route requirement; it never decides one.** *Fact.* A backend-scoped route requirement is resolved in two separate places, and neither of them lets an adapter rule on itself. `DecodedProgram::prepare` refuses a `RouteRequirement::BackendFeature` whose `owner()` is not the host's own `BackendKey` before any adapter is consulted (`crates/tiler-runtime/src/load.rs:586-595`), and the loader's own comment gives the reason: that is decidable from the host's declaration, and asking an adapter about another backend's namespace would invite it to answer. Then `LiveDeviceQualification::resolve_live_device_requirements` takes a host closure returning a `LiveDeviceObservation` — a quantity, a boolean, or `Unrecognized` — and the *loader* performs the comparison, so `Unrecognized`, a wrong-shaped answer, and an unmet floor are three distinct rejections and an adapter cannot reverse a capacity comparison by answering cleverly. *Inference for the contract:* the adapter's conformance obligation is to *report facts*, never to adjudicate them, and a composition design must not move the comparison into the adapter to save a round trip.

**A partial provider is the normal case, not an exception.** *Inference from rows 4, 7, 9, and 12.* The forkless spike's provider wanted exactly one row of the table — row 4 — and reused Tiler's rows 7, 9, and 12 unchanged. The CPU vertical wanted rows 6, 7, 9, 10, 11, 12, and 13 and reused rows 1 through 5 unchanged. Neither wanted all thirteen. A contract whose unit of registration is "a backend" would force both to declare pieces they do not own.

**Row 8 is the one place where a second backend has no seam at all, and the neutral seam it needs already exists in private.** *Fact.* `tiler-build` is statically Metal: `crates/tiler-build/Cargo.toml` takes `tiler-metal` and `tiler-metal-aot` as unconditional dependencies with no feature gate, all six of its modules are `metal_*`, the backend and representation are hardcoded `&str` constants at `crates/tiler-build/src/metal_assembly.rs:27-28` (`"tiler.metal"`, `"metallib"`), and its public entry point `accept_or_publish_metal_plan` takes Metal-AOT types in its signature. **Fact — and there is no indirection anywhere in `crates/` to relax it.** Reproduce with `grep -rniE "(backend|emitter)[_ ]?(registry|register|factory|plugin|dispatcher|selector)" --include='*.rs' crates/`, which returns nothing; the positive control is the same grep with `lowering` in place of `backend|emitter`, which returns 69 lines. *Fact — the useful half.* `crates/tiler-build/src/metal_plan.rs:266`, the private `assemble_artifact`, is already backend-neutral: it takes `declare_payload: impl FnOnce(&mut ArtifactProgramBuilder, TargetProfileRef) -> Result<PayloadId, ArtifactBuildError>` and derives the target-profile reference, feasibility rules, compilation environment, selected lowering providers, deferred predicates, entry specs, and variant without naming Metal once. Its three callers are the three Metal payload declarations. *Inference.* A second backend does not need a new abstraction here; it needs that closure parameter promoted and two hardcoded literals moved into it. The residue that is genuinely not yet neutral is small and nameable: `BindingKind::Buffer` at `metal_plan.rs:302-304` (the enum has one variant), and `zero_work_skips_dispatch: true` with empty launch preconditions at `metal_plan.rs:306-309`.

## Installation, visibility, and observability are three separate obligations

The ticket names three, and there is a fourth that precedes them and is already met, so the table below carries four rows. *Fact.* A surface can satisfy any subset of them, and the physical-provider seam today satisfies exactly one — the one nobody had to work for.

| Obligation | Question it answers | Physical provider, at `e6a47d9` |
|---|---|---|
| Composability | Can the thing a provider returns be built outside the crate? | **Yes.** A `ProposalBody::ScheduledKernel` is a `tiler_ir::schedule::ScheduledRegion`, fully public and constructed for real by `acme-provider` |
| Visibility | Can the vocabulary be named? | **No.** `crates/tiler-compiler/src/lib.rs:23` declares `mod frontier;`, so the module gate fires before item visibility; the governed cost-model key `tiler.cost.structural.v1` is a private constant and the only admissible one |
| Installation | Can one be supplied to a compilation? | **No.** `crates/tiler-compiler/src/pipeline/planning.rs:171` is `let providers: [&dyn PhysicalImplementationProvider; 1] = [&GovernedPhysicalProvider];`, and the internal request carries no provider field |
| Observability | Can a caller see which providers were offered or selected? | **No.** `Compilation::offered_providers` is populated at `session.rs:1513` from `capabilities.0.lowering().providers()` — the lowering registry alone |

**Fact — the observability claim, stated so it can be refuted in one line.** `grep -n "offered_providers" crates/tiler-compiler/src/session.rs` returns the field, the accessor, the population site, and the assembly-side read; the population site is `Arc::from(capabilities.0.lowering().providers())`. The governed physical provider's own identity, `tiler/prototype-serial-sum-physical`, therefore cannot appear, and the spike's `public_surface_names_no_physical_provider` asserts exactly that against a compilation that really ran.

**Fact — the installation asymmetry has a fifth instance nobody has named, and it is in-crate.** The opaque-call seam that [`integrate-opaque-calls-into-the-physical-frontier`](../../../tickets/integrate-opaque-calls-into-the-physical-frontier.md) landed is reachable from `enumerate_frontier`, which takes an `&OpaqueCallRegistry`. The sole production call site constructs an empty one inline: `crates/tiler-compiler/src/pipeline/planning.rs:228` passes `&crate::call_registry::OpaqueCallRegistry::new()`. Reproduce with `grep -rn "OpaqueCallRegistry" crates/`; every other hit is the definition module, the `use`/parameter in `frontier.rs`, or a site inside a `#[cfg(test)]` module (`selection.rs` after its `#[cfg(test)]` at line 1689, `frontier.rs`'s own test module, and `pipeline/tests.rs`). The positive control for that check is `GovernedPhysicalProvider`, which the same style of grep finds at a production site, `planning.rs:171`. *Inference.* No opaque call can reach a compilation through `session::compile`, so the frontier's opaque-call admission is implemented support behind an authority that nothing populates. This is a different statement from ADR 0078's correction, which records that no *out-of-crate* provider registers a call; the gap here is that no caller of any kind does. It is filed as [`register-opaque-calls-on-the-compile-path`](../../../tickets/register-opaque-calls-on-the-compile-path.md) rather than absorbed into this design.

**Proposal — the obligation the ADR must state.** Publishing a trait discharges visibility and nothing else. A seam is complete when all four obligations hold, and a record that promotes one surface must say which of the four it moved. ADR 0078 item 4 is the precedent: the lowering seam's composability and visibility held for a checkpoint during which installation did not, and the record had to say so twice — once as a gap and once as its closure — because no mechanism distinguished them.

**Proposal — the disclosure rule the responsibility matrix owes.** A compilation should report, as separate sets: the complete frozen provider environment it was *offered* (already true for lowering, and ADR 0072's compilation-request environment is exactly this subject), and the providers its retained plan *selected* (already true for lowering, through `PlanAlternative::selected_capabilities`). Physical providers should join both sets, and an installed-but-never-selected physical provider must be visible in the first and absent from the second — because "this provider was available and lost on cost" and "this provider was never installed" are the two findings a composition failure most needs to tell apart, and today neither is answerable at all.

## The propose-then-reverify split: which gates an out-of-crate provider may pre-run

*Fact.* `crates/tiler-compiler/src/physical.rs:1016`, `verify_schedule_with_feasibility`, runs five gates in a fixed order. This table is what the ticket asks for, and the right-hand column is the design question: a provider that can pre-run none of these cannot report a typed local failure of its own, and must instead propose work it knows will be rejected and read the rejection back.

| Order | Gate | Where | Reachable out of crate today | Why |
|---|---|---|---|---|
| 1 | Request authority — `request.reconstructs_its_authority()` and `numerical_contract().is_governed()` | `physical.rs:1023` | **No** | `VerifiedTargetRequest` is in the private `request` module |
| 2 | Whole-region intrinsic verification — `ScheduledRegionBuilder::from_region(..).build()` | `tiler-ir` | **Yes** | `tiler_ir::schedule::ScheduledRegionBuilder` is public, and the spike ran it on both a good and a perturbed body |
| 3 | Numerical-realization agreement — the region's realization must equal the request's | `physical.rs:1029` | **Half** | `NumericalRealization::new` is public and `acme-provider` constructs one; the *request's* resolved realization is not readable — `NumericalContract::resolve` is private and `Compilation` exposes only `resolved_numerical_contract_key() -> &'static str` |
| 4 | Request-subject binding — `verify_region_subject_binding` | `physical.rs:1074` | **No** | It compares against `NormalizedProgramSubject`, a normalization output the compiler owns outright |
| 5 | Hard feasibility — `assess_region` against the target profile | `physical.rs:1033` | **Half** | A provider *can* derive the exact input: `VerifiedScheduledRegion::requirements()` is public in `tiler-ir`. It cannot run the comparison: every `CapabilityAxis`, `CapabilityQuery`, and `FeasibilityOutcome` in `crates/tiler-compiler/src/target/feasibility.rs` is `pub(crate)`, and `TargetProfile`'s public accessors are `governed`, `profile_key`, `canonical_descriptor`, and `dtype_dispatchability` alone |

*Inference.* Gates 1 and 4 are correctly host-only and should stay so: both are statements about a program the host normalized, and a provider positioned to re-derive them would be positioned to disagree with normalization, which is a permanently internal authority under ADR 0078 item 6. Gate 2 is already public and is the one a provider must run on itself — it is what makes a malformed body a provider defect rather than a compiler fault.

**Proposal — the two halves are the decision.** Gates 3 and 5 are each half-reachable in the same shape: the provider holds one operand and the host holds the comparison. Two spellings are available and they are not equivalent.

- *Expose the operands.* Give the provider the request's resolved `NumericalRealization` and a read-only feasibility query over the `TargetProfile`. It can then decline locally, with its own typed reason, before proposing. Cost: two more public surfaces, and a provider that re-implements the comparison may drift from the host's.
- *Expose only the rejection.* Keep both private and let the provider propose and read back `FrontierRejection::Infeasible` naming the disproved predicate. Cost: a provider cannot distinguish "this target refuses my strategy" from "I built the body wrong" without a round trip, and it cannot express `DeclinedStrategy` honestly — that channel exists precisely to say what a provider considered and withheld *before* constructing a region, and it is unusable for a reason the provider cannot evaluate.

The second is cheaper and the recommendation is against it, for the reason the `DeclinedStrategy` channel already encodes: an enumeration is complete only if it can say what was deliberately withheld, and a provider that must propose in order to learn why it should not have cannot supply that. The middle position worth considering is asymmetric — expose the resolved realization (a value, comparable by equality, that the provider must already construct to fill the body) and keep feasibility as a rejection only (a decision procedure ADR 0078 item 6 makes internal). That asymmetry is atomic decision **D6** below.

## Interface sketches

**Proposal throughout.** These sketches are grounded in what the two spikes actually wrote and in the shapes already in the tree; none is accepted, and every one of them is a public boundary that would be Tom's under [ADR 0075](../../decisions/0075-scope-public-boundary-approval-by-change-category.md).

### Installing a physical provider

The shape is deliberately the shape `with_capabilities` already has, because ADR 0078 item 4 closed the identical asymmetry for lowering and the second seam should not invent a second idiom.

```rust
// Proposal — not accepted, not implemented.
// A frozen, per-session, immutable registry. No global, no discovery.
pub struct InstalledPhysicalProviders(/* private */);

impl InstalledPhysicalProviders {
    /// The physical providers this build ships.
    pub fn governed() -> Self;

    /// The governed set plus the caller's own, in the caller's order.
    ///
    /// Order is retained for reporting and is *not* precedence: the frontier is
    /// additive and its output is canonical and provider-order-independent.
    /// Registering one `ProviderIdentity` twice is a typed collision, not a
    /// silent replacement.
    pub fn installed(
        additional: impl IntoIterator<Item = Box<dyn PhysicalImplementationProvider>>,
    ) -> Result<Self, PhysicalProviderInstallError>;
}

impl<'a> CompileRequest<'a> {
    pub fn with_physical_providers(self, providers: InstalledPhysicalProviders) -> Self;
}
```

Two properties of that sketch are load-bearing rather than stylistic. It takes owned boxed providers because a provider must outlive one `enumerate_frontier` call per distinct region subject and the pipeline memoizes frontiers across covers; and it returns a `Result` because a duplicate `ProviderIdentity` has to be a typed refusal at installation, which is the only point at which the collision is cheap to name.

### Observing what was offered and what was selected

```rust
// Proposal.
impl Compilation {
    /// The complete frozen lowering-provider environment. Unchanged.
    pub fn offered_providers(&self) -> &[ProviderIdentity];

    /// The complete frozen physical-provider environment, in installation order.
    /// Present whether or not any of them contributed to the retained plan.
    pub fn offered_physical_providers(&self) -> &[ProviderIdentity];
}

impl PlanAlternative<'_> {
    /// The lowering authorities this alternative selected. Unchanged.
    pub fn selected_capabilities(&self) -> impl ExactSizeIterator<Item = SelectedCapability<'_>>;

    /// The physical providers whose proposals this alternative retained, one
    /// entry per bound region implementation, in canonical region order.
    pub fn selected_implementations(
        &self,
    ) -> impl ExactSizeIterator<Item = SelectedImplementation<'_>>;
}
```

The two accessors are separate on purpose. An offered-but-unselected provider is exactly the case a composition failure needs to see, and merging the sets destroys it.

### Reusing another backend's emitter, with no seam at all

*Fact, from the spike.* This already works and needs no proposal. It is written out because the contract's answer to "how does a partial provider reuse another backend's pieces" is that for this piece it does nothing, and a reader who expects a registry will look for one.

```rust
// Fact — this is what the forkless spike's probe does today, unchanged.
let verified = ScheduledRegionBuilder::from_region(my_specialized_region).build()?;
let kernel = tiler_ir::kernel::lower_scheduled_region(&verified)?;
let unit = tiler_metal::emit::emit_translation_unit(&[&kernel], &target_facts, realization)?;
unit.require_declared_realization()?;
```

### Declaring a backend that is not Metal

*Fact, from the CPU vertical.* Also already works. A second backend declared its own governed profile key, backend family, and representation, packaged a real payload, and routed to it, against `crates/` unmodified.

```rust
// Fact — the shape the CPU vertical used, condensed.
let profile: TargetProfile = TargetProfileBuilder::new(TargetProfileKey::new(
    "tiler.target.cpu-scalar-host-aarch64-darwin",
)?) /* … declared axes … */ .build()?;

let compilation = tiler_compiler::session::compile(
    CompileRequest::new(&program, NumericalContract::StrictF32, TargetRequest::new([profile])?),
)?;

// The backend translates the verified kernels into bytes it alone understands,
// and states whose vocabulary owns them. Note what is NOT here: tiler-build.
let content: PayloadContent = my_backend::translate(alternative.kernels())?;
let mut builder = ArtifactProgramBuilder::new(&program, environment)?;
let payload = builder.push_carried_payload(
    BackendKey::new("tiler.cpu.scalar")?,
    RepresentationKey::new("tiler.cpu.scalar-image-v1")?,
    SchemaVersion::new(1, 0),
    compatibility,                       // TargetProfileRef: key + exact descriptor digest
    ArtifactExecutionPolicy::NativeImage, // the only policy the loader routes
    content,
)?;
builder.push_variant(alternative.abi().kernel_program(), variant_spec(payload))?;
let artifact = builder.build()?;
```

**Fact worth naming, because it is the row-8 finding seen from the other side.** The CPU vertical reached `tiler_artifact::program` directly and used `tiler-build` not at all. That is why it could run against `crates/` unmodified: the orchestrator is the one component with no seam, so the only way past it is around it. A production second backend cannot take that route, because `tiler-build` is also what composes the cache subject, compiles on a miss, and re-proves correspondence before publication — none of which the vertical did.

*Measurement — finding 2 of the CPU vertical.* `tiler.cpu.scalar` and `tiler.cpu.scalar-image-v1` were minted without touching a registry. That openness is what makes a new backend expressible; the cost is that nothing prevents two producers minting one key for two things, which is atomic decision **D9**.

**Correction — that finding overstates the validation, and the reason it does is a name collision the ADR should know about.** The spike records the keys as validating "length and alphabet only". **Fact, at `e6a47d9`.** `crates/tiler-artifact/src/program/keys.rs:73-85`, `validate_key`, is the entire validator behind `BackendKey`, `RepresentationKey`, `TargetProfileKey`, `FeasibilityRuleSetKey`, `CapabilityKey`, and `RouteFeatureKey`, and it checked two things: non-empty, and at most `MAX_GOVERNED_KEY_BYTES` (256). There was no alphabet, case, separator, or namespace-prefix check anywhere in the crate. Reproduce with `grep -rn "is_ascii\|InvalidByte" crates/tiler-artifact/src/`, which returned nothing at that commit; the positive control is the identical grep over `crates/tiler-compiler/src/target.rs`, which returned six lines including the admitted-byte closure then at line 226. **Fact — why the spike could honestly report otherwise.** Two different types are spelled `TargetProfileKey`. `tiler_compiler::target::TargetProfileKey` (`target.rs:210`) enforces non-empty, at most 128 bytes, and an ASCII-lowercase/digit/`.`/`-`/`_` alphabet with a typed `InvalidByte { index, value }`. `tiler_artifact::program::TargetProfileKey` enforced length alone. The spike used the first to declare its profile and the second inside its payload, and reported one sentence about both. `crates/tiler-build/src/metal_plan.rs:333` was the laundering site: `TargetProfileKey::new(compilation.target_profile_key())` converted the strict spelling into the permissive one, so the strict grammar was a producer-side property and not an artifact-level guarantee. *Inference.* An artifact arriving from anywhere but a governed producer may carry a profile key the compiler's own constructor would refuse, which is a real asymmetry rather than a naming nuisance; it is filed as [`reconcile-the-two-target-profile-key-grammars`](../../../tickets/reconcile-the-two-target-profile-key-grammars.md) rather than settled here.

**Corrected 2026-08-01 — the spike's sentence is now right, and the collision this correction exposed is what closed it.** That ticket made the six `governed_key!` types admit exactly `tiler_compiler::target::TargetProfileKey`'s alphabet, refusing every other byte as a typed `ArtifactBuildError::NoncanonicalKeyByte`, so the artifact layer validates length *and* alphabet and `metal_plan.rs` launders nothing. Re-run at `7ad2aca`, `grep -rn "is_ascii\|InvalidByte" crates/tiler-artifact/src/` returns one line — `keys.rs:121`, the admitted-byte closure — so the command that was this record's evidence of *absence* is now its positive control, and the identical grep over `crates/tiler-compiler/src/target.rs` still returns six lines. Two things this correction got right survive the fix and are worth keeping in view: the byte bounds deliberately stay apart, 128 minting into 256, so a reader must not "finish" the reconciliation by narrowing the artifact bound to one producer's number; and the two types stay two types, which [the glossary](../../glossary.md) now indexes by name so that a future sentence cannot describe both at once the way finding 2 did.

## Trust and linkage boundary

**Proposal, and it is a restatement rather than a new position.** The [operation-extension contract](../../operation-extensions.md#initial-trust-and-linkage-model) already fixes the trust and linkage model for extension providers, accepted by [ADR 0045](../../decisions/0045-bound-proc-macro-providers-to-host-dependencies.md): trusted native compiler code, statically linked into the process, supplied explicitly to a session, with compiler-process privileges and no sandbox. Backend composition adds no new linkage mode and inherits every clause, including the proc-macro limitation — a function-like macro supports providers already in its host dependency graph and cannot discover objects defined later in the consuming crate.

The following are **deferred**, explicitly and jointly, and no part of this design reserves a seam for any of them: native dynamic loading of a backend, a stable plugin ABI, `dlopen`-style discovery of a runtime adapter, hot reload, untrusted or sandboxed providers, cross-process provider callbacks, and runtime source compilation of a kernel. The one thing that crosses a process boundary is **artifact transport**: bytes, validated from bytes on arrival, with no Rust object and no callback travelling with them.

*Inference — why the deferral costs nothing to state now.* Every join in the matrix above is already a governed key or a canonical identity rather than a Rust object. `TypeId`, vtable addresses, function pointers, and registration order appear in none of them. So the artifact half of a dynamic-loading design is already built; what is deferred is only the mechanism by which a *process* acquires code, and deferring it does not shape any identity subject. A design that instead joined producer to adapter by a shared Rust object would have to be undone to defer anything, which is the argument for the current join and not merely a consequence of it.

## Minimum conformance obligations

**Proposal.** A component claiming a row of the matrix owes exactly the obligations of that row and no others. Stating them per row rather than per "backend" is what makes a partial provider expressible.

**Every provider-shaped component (rows 2, 3, 4).** Deterministic, side-effect-free, in-process. A versioned `ProviderIdentity` whose revision is output-affecting in the literal sense: bumping it must accompany a change to the bytes it produces, and not bumping it while the bytes change is the one defect the host cannot detect. Output re-enters the ordinary checked path and is never believed. No self-stamped provenance, resources, or boundary guarantees. Offering nothing is a legitimate local result and must be distinguishable from a rejection.

**A physical provider (row 4), additionally.** It must run gate 2 on its own body before proposing, because a body that fails intrinsic verification is its defect and fails the whole enumeration closed rather than degrading to an empty offer. It must attribute its cost estimate to a governed cost-model key. It must report withheld strategies through the decline channel when the reason is a fact about the request rather than about a region it did not build.

**A backend (rows 7, 9, and 10).** *Measurement — finding 11 of the CPU vertical.* A payload's `code` bytes are opaque to every check the artifact layer performs, and the six payload-level refusals in that run — a foreign domain separator, truncation, trailing bytes, an out-of-range slot, an access-mode violation, and the accepted neighbour they were measured against — are checks the backend owns. So: **a backend must validate its own payload from bytes, and must do so while the preflight is still held.** *Inference, and the CPU spike states it in these words:* the first backend that skips this discovers a malformed payload after the routing commit, where [ADR 0051](../../decisions/0051-make-runtime-routing-commit-one-way.md) forbids selecting another plan. It must also declare its representation such that a host consuming a different one is refused rather than translated — the CPU vertical's host consuming `tiler.cpu.scalar-image-v2` was refused as `runtime.unexecutable-payload`, and that refusal is the obligation working. **Corrected 2026-08-01:** that class was removed by [`select-executable-variants-across-registered-backend-families`](../../../tickets/select-executable-variants-across-registered-backend-families.md), which made host-relative ineligibility a filter applied before any applicability guard rather than a terminal mismatch after one. The same host now excludes the variant as `VariantIneligibility::UnsupportedRepresentation`, and the load is refused as `runtime.no-eligible-variant` because the spike's artifact packages one variant; on a portfolio packaging a runnable alternative the same exclusion would leave that alternative to be selected instead. Re-measured at `e2da98f`. The obligation is unchanged — this corrects the class a reader would go looking for, not the finding.

**A runtime adapter (row 12).** It **reports** facts and never adjudicates them: it answers a live-device request with an observation and lets the loader compare, and it never sees a backend-scoped requirement owned by a backend the host did not declare, because the loader refuses that before asking. It re-validates from bytes rather than trusting the producing process. It must complete every route-sensitive check before the routing commit, and it owns the second-stage facts no artifact can assert. *Measurement — finding 4 of the CPU vertical.* On a CPU that second stage is real but numerical rather than structural: the vertical's host preflight measured whether the running process preserves subnormals and refused a route whose image declared otherwise. *Inference.* "Device-bound facts" is the wrong name for this obligation; the general shape is *facts about the execution context that no artifact can assert*, and a contract must not require the stage to be a pipeline query.

**A target-profile authority (row 6).** It declares typed facts with explicit availability phase, authority, and provenance, and leaves undeclared axes `Unknown` rather than defaulting them. *Measurement — finding 8 of the CPU vertical.* `WorkgroupThreads` and `LocalMemoryBytes` are compared against every kernel's derived requirements, so a CPU profile has to declare `1` and `0`; omitting them leaves them `Unknown`, which is a different claim. That is the sparse-profile rule working as designed, and it is also evidence that the quantitative axis set is a GPU axis set with a neutral spelling. **Fact — and the axis set is closed as well as GPU-shaped.** Every variant of `CapabilityAxis` is `pub(crate)` in `crates/tiler-compiler/src/target/feasibility.rs:195`, and the seven admitted axes are reachable only through the `TargetProfileBuilder::declare_*` methods, each of which hard-codes its own axis internally. *Inference.* A backend cannot declare an axis this compiler build does not already have a method for, so the CPU vertical's missing vector width, mask and tail support, scalable-vector length, cache levels, and thread count are not merely undeclared — they are inexpressible, and closing that gap is a compiler change rather than a profile one.

**What no component may do.** Register itself globally. Depend on registration order for precedence. Replace an existing registration. Mutate a frozen registry. Declare a numerical permission the host cannot verify. Return a semantically incorrect result to preserve a fast path.

## Every unsupported case

Stated as refusals, because [ADR 0069](../../decisions/0069-use-a-general-compilation-boundary.md)'s general compilation boundary requires an unsupported case to reject explicitly rather than approximate.

- **More than one live device, sharding, collectives, cross-device transfers, queue affinity, and multiple command streams.** The accepted initial execution profile is one symbolic affinity, one live device, one ordered command stream ([ADR 0047](../../decisions/0047-model-placement-as-physical-properties.md)), and whether multiple devices become expressible at all is [`multi-device-and-sharding-scope-gate`](../../../tickets/multi-device-and-sharding-scope-gate.md)'s product decision. This record narrows nothing there.
- **Dynamic loading, plugin ABI, hot reload, untrusted providers, cross-process callbacks, runtime source compilation.** Deferred above.
- **A `ProposalBody::View`.** The one body variant still carrying a `ReservedProposalSeam` and rejecting as `FrontierRejection::UnsupportedVariant`.
- **Out-of-crate opaque-call registration.** ADR 0078's correction records it as compiler-owned and crate-private; this record adds only that no caller of any kind registers one on the compile path.
- **A backend contributing scheduling *knowledge* as data rather than code, or vice versa.** This is ADR 0078's open question and is decision **D1** below, not something this record settles.
- **A host-process availability phase.** *Measurement — finding 5 of the CPU vertical.* The five `AvailabilityPhase` variants are `CompileProfile`, `ArtifactEvidence`, `LiveDevicePreflight`, `PreparedKernelPreflight`, and `LaunchPreflight`; a fact known once a host *process* is bound must borrow `LiveDevicePreflight`. Filed as [`name-a-host-process-availability-phase`](../../../tickets/name-a-host-process-availability-phase.md).
- **A neutral execution policy vocabulary.** *Measurement — finding 6 of the CPU vertical.* `ArtifactExecutionPolicy` is a two-valued GPU dichotomy with no spelling for an interpreted image, a JIT input, or a dynamically linked object. **Fact — and one of its two values is unroutable.** `crates/tiler-runtime/src/load.rs:468-473` matches the policy exhaustively and returns `LoadRejection::UndeliverableExecutionPolicy` for `RequiresDeviceTranslation`, deliberately rather than by wildcard. *Inference.* A backend whose representation needs device-side translation cannot route through `tiler-runtime` at all today, so the two-valued vocabulary is effectively one-valued at the load boundary, and a contract must not read the second variant as supported.
- **A build-time orchestration seam.** *Fact.* `tiler-build` has no indirection between itself and `tiler-metal`; a second backend today needs either a sibling orchestrator crate or a new `*_plan.rs` module and a new dependency inside `tiler-build`. The neutral half already exists as the private `assemble_artifact` closure parameter, which is what makes this a promotion rather than a design.
- **A profile-key grammar guaranteed at the artifact boundary — supplied after this record, on 2026-08-01.** *Fact, at `e6a47d9`.* The artifact layer validated governed key length alone and the strict alphabet lived only in the compiler's separate same-named type. [`reconcile-the-two-target-profile-key-grammars`](../../../tickets/reconcile-the-two-target-profile-key-grammars.md) closed it for all six governed keys; the correction above records the re-run and what deliberately did not change.
- **Backend-neutral payload provenance.** *Measurement — finding 7 of the CPU vertical.* `PayloadProvenance` requires Apple-shaped fields with no CPU meaning.
- **A typed home for a target triple, ABI, or data layout.** *Measurement — finding 9 of the CPU vertical.* `CapabilityAxis` has no such axis, so both survive only inside the profile key string and the payload provenance.
- **A non-identity transport mapping assumed either way.** *Measurement — finding 10 of the CPU vertical.* A scalar entry's transports are its ABI slots because storage is bound by signature position; Metal's mapping is not the identity in general, and a contract assuming either is wrong for the other backend.
- **A mandatory prepare stage.** *Measurement — finding 3 of the CPU vertical.* Because the CPU profile declares its workgroup bound as a compile-time fact, the plan carried zero deferred prepared-entry requirements and `preflight` alone sufficed; Metal cannot do this, because only a built pipeline knows its own `maxTotalThreadsPerThreadgroup`. *Inference.* The prepare stage is correctly optional, and a provider contract must not make it universal.

## Two end-to-end examples

### Standard Metal plus a partial custom provider

The composition is *three* installed things, of which one does not exist yet. **Proposal** for the sequence; **Fact** for every step marked as such.

1. The caller freezes a lowering registry and installs it — `InstalledCapabilities::installed(lowering, scalars)` then `CompileRequest::with_capabilities`. *Fact: this path exists and an out-of-crate caller drives it in `prototypes/serial-sum-compile`.*
2. The caller declares or takes the Metal target profile and installs it through `TargetRequest`. *Fact.*
3. The caller installs `[GovernedPhysicalProvider, AcmeSimdgroupPointwise]`. **This is the missing step.** Acme implements one region — the bounded profile's pointwise region — with a 32-thread workgroup where the governed provider emits one thread.
4. `enumerate_frontier` asks both. Both propose. Both bodies pass gates 1–5. Two admitted implementations of one region are retained with distinct `ImplementationProposalIdentity` values separated by folded provenance. *Fact about the algorithm: `crates/tiler-compiler/src/frontier.rs`'s own tests enumerate two providers forward and reversed and compare the outcome. Inference that a cross-crate provider would behave identically, since `enumerate_frontier` does not distinguish them.*
5. Plan selection compares them on cost and retains the non-dominated set. Acme wins, loses, or ties on the structural cost model — the contract does not privilege it, and registration order does not decide.
6. `tiler-build` emits the selected kernels through **stock `tiler-metal`**, unchanged. *Measurement: the spike emitted Acme's specialized body through `emit_translation_unit` and it passed `require_declared_realization`.* The two alternatives' entry symbols differ because the symbol is identity-derived, so a unit holding both does not collide.
7. The artifact carries `BackendKey` `tiler.metal`, representation `metallib`, the target profile reference, and — **proposed** — the selected physical provider's identity as plan provenance.
8. The Metal runtime adapter binds the artifact to a live device. It has never heard of Acme, needs no registration relating it to Acme, and refuses only on the governed keys and canonical identities. *Fact about the loader's inputs.*

What this example demonstrates and what it does not: it demonstrates that a partial provider needs exactly one row of the matrix and inherits twelve; it does **not** demonstrate that two providers' cost estimates are comparable, which the spike explicitly could not measure.

### A CPU backend

*Measurement, base commit `488efac`, Apple M-series arm64 macOS.* This one is not a proposal — it ran.

1. Declare a bounded scalar CPU target profile through `TargetProfileBuilder`: governed key `tiler.target.cpu-scalar-host-aarch64-darwin`, one thread per workgroup, zero staged local memory, two buffer bindings per entry, subnormals preserved exactly, every reshaping freedom unsupported, and every vector, mask, tail, threading, and cache axis left `Unknown`.
2. Compile against that profile alone, under `NumericalContract::StrictF32`, through the same `session::compile` Metal uses. **No physical provider is installed and none is needed** — the governed provider's proposals are target-neutral scheduled regions, and the profile is what makes them feasible or not.
3. Translate each verified structured kernel into `tiler.cpu.scalar-image-v1`, a representation this backend minted. Observe the translator refusing four buffer parameters it cannot bind, against an accepted neighbour, before claiming the positive path.
4. Package a real artifact **by calling `tiler_artifact::program` directly and skipping `tiler-build` entirely**, encode and decode the envelope, and run the fail-closed probe set against those exact bytes. The skip is not an oversight; row 8 offers nothing to call.
5. Decode the payload through a decoder that knows nothing about `VerifiedKernel`, and run the payload-level probe set. **This is the backend's own validation obligation, and it runs while the preflight is still held.**
6. Bind a live host execution context by *measuring this process* — architecture, system, pointer width, byte order, and the subnormal behaviour of its actual arithmetic — and refuse a route whose declared realization the process does not deliver.
7. Commit the route one way, execute, and compare bits against `tiler-reference`.

*Measurement.* Twelve `f32` elements agreed bit for bit, including a negative zero, both least-magnitude subnormals through a multiply, a non-canonical NaN payload canonicalized to `0x7fc00000`, and both infinities. Deleting the `CanonicalizeF32Nan` arm from the interpreter made the run fail at the comparison naming exactly one differing element, which is the evidence the agreement is a result rather than a tautology.

The conclusion the composition contract takes from this: **a second backend needed no production edit and no provider interface at all.** Rows 6, 7, and 9 through 13 were all it wanted, and every one of them is already installable or already outside the compiler. It skipped row 8 rather than using it. The thing that is missing on the *compiler* side is row 4, and only row 4.

## Eliminated alternatives

Each is stated with the grounds that eliminate it, so a reader can refute the elimination rather than only the conclusion.

**A monolithic `Device` or `BackendProvider` trait bundling profile, physical proposals, emission, artifact production, and runtime adaptation.** Eliminated on three independent grounds. *Correctness:* the pieces are fixed at four different times — profile at compile time, emission at build time, artifact at publication, adapter at run time — and a single trait object cannot exist at all four, so the design would either force a live-device-capable object into the compiler or split itself back into the four pieces it claimed to unify. *Maintainability:* both spikes wanted a strict subset — the forkless provider wanted row 4 alone and the CPU vertical wanted rows 6 through 13 — so the unit of registration would never match the unit of contribution, and every partial provider would declare pieces it does not own. *And it breaks a working edge:* the emitter is reached today by an ordinary Cargo dependency that the compiler checks; routing it through a trait converts a compile-time-checked reuse into a runtime-resolved one and buys nothing, since emission consumes verified kernels and knows nothing about who proposed them. The one row that genuinely lacks a seam, row 8, already has a *narrower* neutral shape in private — a single closure parameter — so even there the trait would be the larger answer to the smaller question.

**Global registration or ambient discovery — a `static` registry, an inventory-style link-time collector, or an environment-variable search path.** Eliminated on correctness. Compilation identity must be a deterministic function of the request, and ADR 0072 already places the complete frozen registry environment inside compilation-request provenance; a registry that a linked-in crate or an environment variable can change makes two identical requests produce two identities for reasons no artifact records. It also makes the seam untestable in-process, since two tests in one binary would share it.

**Registration-order precedence — first-wins, last-wins, or a priority number.** Eliminated on correctness, and the tree already refuses it in both seams. In the lowering registry two matching capabilities are `AmbiguousCapability` and two revisions of one provider are the same, deliberately, so that no newer revision silently supersedes an older one. In the frontier the admitted set is returned in canonical, provider-order-independent order, so ordering has nothing to order. *Inference.* Adding precedence to either would convert a typed contradiction into a silent choice, which is precisely the erosion ADR 0078's consequences section names as the most likely.

**Last-wins replacement of a registration.** Eliminated with the above, and separately on identity grounds: a replaced registration would have to leave the compilation-request environment either stale or silently rewritten, and neither is a claim an artifact could carry.

**Ambient mutation of an installed registry after freezing.** Eliminated on correctness. Every registry in the design is a verified product with private storage and a consuming terminal build ([ADR 0071](../../decisions/0071-use-checked-builders-for-shared-compiler-ir.md)); a thaw would let a plan be selected under one authority and reported under another.

**Joining build-time producer to runtime adapter by a shared Rust object, a `TypeId`, or a registration handle.** Eliminated on correctness. The join must survive a process boundary — that is the whole point of an artifact — and no process-local value can. The mechanism that exists, governed keys plus canonical identities compared from bytes, already works and is what [`join-build-time-producers-to-runtime-adapters-through-artifact-identity`](../../../tickets/join-build-time-producers-to-runtime-adapters-through-artifact-identity.md) will prove.

**Requiring the runtime adapter's identity to match the producing provider's.** Eliminated on both correctness and design. There is no adapter identity in the artifact to match against; adding one would assert that bytes may only be executed by their producer's counterpart, which is false — the CPU vertical's payload is executable by any decoder of `tiler.cpu.scalar-image-v1`, and that substitutability is what a representation key *means*.

**Making the `prepare` stage mandatory for every backend.** Eliminated by measurement. The CPU vertical's plan carried zero deferred prepared-entry predicates and `preflight` alone was sufficient and correct; requiring `prepare` would oblige a backend to invent a pipeline object it does not have.

**Treating "installed" and "visible" as one obligation, closed by a `pub` keyword.** Eliminated by the forkless spike's compile-fail evidence, which is the direct measurement: `no_physical_provider_installation_seam.rs` fails on a missing *method*, not a missing *item*, and the pass fixture beside it installs a lowering registry through the method that does exist. Publishing `frontier` would leave the second fixture failing exactly as it does now.

**Deriving physical-provider precedence from the cost model by letting a provider declare its own cost model.** Eliminated on correctness. A proposal attributing its estimate to any key but the governed one is already a hard `FrontierError::MalformedCostProvenance`, and relaxing it would let two providers' estimates be incomparable while still being ranked — which is a silently wrong selection rather than a refusal.

**Letting a physical provider stamp its own provenance, resource requirements, or boundary contract.** Eliminated by ADR 0078 item 1, and by construction: `ImplementationProposal` carries a body, an applicability predicate, and a cost, and the frontier derives the rest. Any relaxation makes the artifact's claim about which authority produced a plan forgeable.

**Adding an unrestricted scoring callback so a consumer can rank providers.** Eliminated on correctness and determinism: a caller-supplied closure is neither deterministic across processes nor identity-bearing, so two identical requests could select different plans with nothing in the artifact recording why. Typed policy over governed cost identities is the shape that survives, and [`expose-explicit-backend-provider-and-selection-policy-composition`](../../../tickets/expose-explicit-backend-provider-and-selection-policy-composition.md) already names it.

## Proposed dependency direction

**Proposal.** The direction below adds no edge to the accepted packaging profile in the [architecture contract](../../architecture.md#accepted-prototype-packaging-profile) **for rows 1 through 7 and 9 through 13**, and needs one for row 8 alone. That split is the claim worth checking, because a composition design that needed new edges throughout would be a redesign of component ownership rather than a contract over it — and this one does not. The compiler in particular is untouched: `crates/tiler-compiler/Cargo.toml` depends on `tiler-ir` and two numeric crates and does not know Metal exists, so installing a physical provider adds nothing to its closure.

```text
    caller / frontend
        │  installs: semantic registry, lowering registry, physical providers,
        │            target profiles                    (all per-session, frozen)
        ▼
    tiler-ir  ◄──────────────────────────────  a provider crate depends on tiler-ir
        │                                       for the proposal body, and on
        │ compilation request                   tiler-compiler for the seam
        ▼
    tiler-compiler ─── enumerate_frontier ──►  installed physical providers
        │                                       (in-process, statically linked)
        │ verified kernels + plan + provenance
        ▼
    tiler-build ──────► backend emitter crate (tiler-metal, or another)
        │                    │ emitted representation
        │ artifact           ▼
        ▼               target AOT tooling
    tiler-artifact ── bytes ──►  ────────────────────────────────┐
        │                                                        │ process boundary
        ▼                                                        ▼
    tiler-runtime  (device-free: decode, classify, bind, commit)
        │  ExecutionEnvironment: one TargetProfileRef, one BackendKey,
        │                        one RepresentationKey — stated, not discovered
        ▼
    runtime adapter ──► live device + execution context
```

Four rules constrain it, and each is a restatement of an accepted boundary rather than a new one.

A **provider crate depends inward**: on `tiler-ir` for the body and on `tiler-compiler` for the seam. Nothing in the workspace depends on a provider crate, and the compiler holds providers as installed values rather than as a dependency.

A **backend emitter is reached by an ordinary Cargo edge from the producing crate**, never by a registry inside the compiler. `tiler-build` holds that edge today and the profile already records it. *Fact — and this is the one place where the accepted profile and a composed backend set genuinely disagree.* `tiler-build` names `tiler-metal` unconditionally, so a second backend either adds a second unconditional edge to the same crate or acquires a sibling orchestrator. Which of those the ADR chooses is decision **D10**, and it is the only decision in this record that changes the packaging profile rather than describing it.

A **runtime adapter depends on `tiler-artifact` and never on `tiler-compiler`.** The architecture contract states this as a rule about roles that no crate has yet — a runtime adapter must not link the optimizer merely to execute a compiled artifact — and it is the rule that keeps row 12 independently selected.

**No edge points at the frontend**, which the `dependency_direction` test in `crates/tiler` already enforces for the one edge class it covers.

## The atomic decisions a durable ADR must make

Enumerated so [`draft-the-backend-provider-composition-adr`](../../../tickets/draft-the-backend-provider-composition-adr.md) can take them one at a time. Each states what is at stake, what this record recommends, and what evidence would refute the recommendation. **D1 is prior to D2 and D3**; the rest are independent of each other.

**D1 — Is target-specific scheduling knowledge typed profile data, provider code, or a checked combination?** This is ADR 0078's own open question, marked as Tom's, and it decides `frontier::PhysicalImplementationProvider`'s participation model. *Evidence now available that ADR 0078 did not have:* the forkless spike shows a real specialization — a 32-thread workgroup — that is *code* in the sense that no profile axis expresses it, and that a profile could not express without acquiring a "preferred workgroup width" axis whose value is a schedule choice rather than a target fact. The CPU vertical shows the opposite direction: a whole second backend needed profile *data* and no provider code at all. *Recommendation:* a checked combination, with the split drawn at feasibility — profiles declare what a target can do, providers propose what to do with it, and the host compares. *What would refute it:* a specialization that is fully determined by declared target facts, which would make provider code redundant for that case.

**D2 — Does a physical provider become an installable public seam, and with what exact facade?** Depends on D1. *Recommendation:* yes, with the `with_physical_providers` shape above, because the alternative — keeping row 4 internal — makes "partial provider" mean "fork `tiler-compiler`", which the originating ticket exists to prevent. *This is a public trait and namespace and is Tom's under ADR 0075 regardless of the recommendation.*

**D3 — Does ADR 0078's item 5 trigger read as already fired?** The record itself flags this as unresolved and as Tom's: one sentence settles whether the sharpened trigger restates his intent or narrows it, and the answer decides whether D1 and D2 are ripe now or still waiting.

**D4 — What exactly does a compilation disclose about physical providers?** Three sub-answers that must be given together: the offered set, the selected set, and whether an installed-but-unselected provider is visible. *Recommendation:* offered and selected are separate accessors, and an unselected provider appears in the first only. *Grounds:* merging them destroys the distinction a composition failure most needs.

**D5 — Is an installed physical provider's identity part of artifact provenance, and if so which subject?** *Recommendation:* yes, in selected-plan provenance beside the lowering authority, and never in graph meaning — the direction ADR 0072 already fixes. *What would refute it:* a demonstration that two providers producing byte-identical plans should share one artifact identity, which would argue for excluding it.

**D6 — May an out-of-crate provider pre-run the numerical-realization comparison, the feasibility assessment, both, or neither?** The asymmetric middle position is the recommendation: expose the request's resolved `NumericalRealization`, keep feasibility as a rejection only. *Grounds:* the realization is a value the provider must construct anyway and compare by equality; feasibility is a decision procedure ADR 0078 item 6 makes permanently internal, and exposing a second implementation of it invites drift.

**D7 — Is the payload-validation obligation normative on every backend, and does it run before the routing commit?** *Recommendation:* yes to both, stated as an obligation rather than a mechanism. *Grounds:* the artifact layer provably cannot do it — `code` bytes are opaque to every check `DecodedProgram` performs — and ADR 0051 forbids a post-commit fallback, so an unvalidated payload has nowhere to fail safely.

**D8 — Is `prepare` optional or universal?** *Recommendation:* optional, and the contract must state the exact condition, because the natural reading of a Metal-derived pipeline is that it is universal. **Fact — the condition, read from the loader.** `preflight` is sufficient exactly when the selected variant has zero deferred predicates *and* zero route requirements: `crates/tiler-runtime/src/load.rs:309-310` calls `refuse_route_requirements` then `refuse_deferred`, in that order and with that reason — a host short of a bound device learns that before it learns it is also short of a prepared pipeline. And once a caller enters `prepare`, both device stages are mandatory even when their lists are empty, because a `RoutePreparation` can only be reached through a resolved qualification. *Grounds:* the CPU vertical carried zero of both and completed correctly on `preflight` alone; Metal cannot, because only a built pipeline knows its own `maxTotalThreadsPerThreadgroup`.

**D9 — Who governs the `BackendKey` and `RepresentationKey` namespaces?** At `e6a47d9` neither the grammar beyond a length bound nor the namespace was governed. *Recommendation:* keep minting open and make collision a producer-side responsibility with a stated convention, rather than adding a registry that would reintroduce global state. *What would refute it:* a case where two independently minted identical keys reach one loading host, which the current single-consumer profile cannot produce but a published crate set could. The grammar half was separate and more urgent, and it was decided on 2026-08-01: the artifact layer does enforce the compiler's alphabet, per the correction above and [`reconcile-the-two-target-profile-key-grammars`](../../../tickets/reconcile-the-two-target-profile-key-grammars.md). The namespace half is what remains open, and [ADR 0090](../../decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 10 accepted this recommendation for it.

**D10 — Does the build-time orchestration seam become a public boundary, and at what shape?** *Recommendation:* promote the existing `assemble_artifact` closure rather than introduce a backend trait, and move `BindingKind`, the zero-work dispatch policy, and the launch preconditions into what a backend supplies. *Grounds:* the neutral function already exists and is already correct; a trait would re-mediate an edge that a closure parameter already abstracts. *What would refute it:* a second backend needing to vary something `assemble_artifact` derives rather than something it delegates, which would show the split is in the wrong place.

**D11 — What is the initial device profile of a composed backend set?** *Recommendation:* restate the accepted one — one symbolic affinity, one live device, one ordered command stream — and defer to the existing gate rather than narrowing it here.

## Measurement boundary and what this record could not establish

- **One commit, one host.** Every `Fact` is inspected source at `e6a47d9`. Every `Measurement` is one of the two spikes at its own recorded commit and toolchain, on one macOS arm64 host.
- **Two providers' cost estimates were never compared.** The forkless spike could not reach `enumerate_frontier`, and the program it compiled retains a single fused alternative, so *even the ordering the public surface does expose had nothing to order*. Whether a third party's structural cost estimate is comparable to the governed provider's is unmeasured, and D2 does not settle it.
- **The additivity claim is narrow.** It rests on one axis, `threads_per_workgroup`, being free under the intrinsic verifier and folded into canonical identity. A specialization varying an axis the request-subject binding *does* compare would be rejected, and no case measures where that line falls.
- **Nothing here measures runtime behaviour of a composed set.** The CPU vertical ran in one process, so its artifact-identity check is a tautology by its own statement; the cross-process join is [`join-build-time-producers-to-runtime-adapters-through-artifact-identity`](../../../tickets/join-build-time-producers-to-runtime-adapters-through-artifact-identity.md)'s work.
- **No three-backend portfolio exists.** Standard Metal, a custom Metal specialization, and CPU have never been enumerated together; [`exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio`](../../../tickets/exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio.md) is where that evidence comes from, and every claim here about a mixed portfolio is inference.
- **The interface sketches are unexercised.** None compiles; none has an out-of-crate fixture. A tested implementation would be a concrete draft and still not an accepted boundary.

## Traceability

This record is evidence for the composition ADR and is not itself a contract. [ADR 0078](../../decisions/0078-name-the-intended-public-extension-seams.md) owns the seam classification and the propose-then-re-verify trust rule it depends on; item 5's deferral is what this record supplies evidence toward resolving, and it does not overturn it. [ADR 0072](../../decisions/0072-separate-semantic-meaning-from-provider-provenance.md) owns the separation of provider provenance from graph meaning, which the matrix's disposition column applies. The [architecture contract](../../architecture.md#component-ownership) owns component responsibility and dependency direction; the [operation-extension contract](../../operation-extensions.md) owns the provider trust, identity, registration, and diagnostic obligations this record inherits without restating; the [glossary](../../glossary.md#backend-device-and-execution-context-vocabulary) owns the vocabulary. The [proposed CPU/SIMD target profile](../../backends/cpu.md) stays proposed and Q-PLAN-011 stays open.

The reproductions are the [forkless custom Metal physical provider](../../../spikes/extensions/forkless-physical-provider/README.md), which is a workspace of the [operation-extension experiments](../../../spikes/extensions/README.md), and the [bounded scalar CPU backend vertical](../../../spikes/target-profiles/scalar-cpu-vertical/README.md).

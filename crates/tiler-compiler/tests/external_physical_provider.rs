//! Out-of-crate proof of the installable physical-implementation provider seam.
//!
//! An integration test is a separate crate, so everything below reaches
//! `tiler-compiler` through exactly the surface a third-party provider crate
//! would: no `#[path]` include, no feature flag, no private access. What compiles
//! here is what an out-of-tree provider can write, and what fails to compile here
//! is what it cannot.
//!
//! The provider is deliberately *partial*. It specializes one schedule axis of
//! whichever regions this compiler already spells and offers nothing for the
//! rest, which is the composition case ADR 0090 calls normal: it claims one row
//! of the responsibility matrix and reuses the other twelve.

use std::collections::BTreeSet;

use tiler_compiler::physical_provider::{
    DeclinedStrategy, GOVERNED_PHYSICAL_COST_MODEL_KEY, ImplementationContext,
    ImplementationProposal, InstalledPhysicalProviders, PhysicalImplementationProvider,
    PhysicalProviderInstallationError, PhysicalProviderProvenance, PhysicalProviderProvenanceError,
    ProviderOffer, StrategyDeclineCause, TargetApplicability,
};
use tiler_compiler::session::{
    Compilation, CompileFailureClass, CompileRequest, NumericalContract, compile,
};
use tiler_compiler::target::{
    DTypeDispatchability, DeviceAddressWidth, IndexArithmeticSupport, ScalarArithmetic,
    ScalarSupport, TargetFactProducerIdentity, TargetFactSource, TargetNormativeReferenceIdentity,
    TargetProfile, TargetProfileBuilder, TargetProfileKey, TargetRequest,
};
use tiler_ir::schedule::{
    ExceptionalValueAssumption, NumericalPermission, ScheduledRegion, SubnormalMode,
};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, ProviderIdentity, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

/// The stable name this file's providers decline under.
const WIDE_WORKGROUP_STRATEGY: &str = "acme.wide-workgroup";

/// The explain subject one provider identity renders to.
///
/// Derived from the identity rather than written out, so a change to how an
/// identity renders fails at the comparison with both spellings in hand rather
/// than leaving these assertions guarding a name nothing produces.
fn explain_subject(name: &str, revision: u32) -> String {
    ProviderIdentity::new("acme", name, revision)
        .expect("the acme provider identity is valid")
        .to_string()
}

/// How a specialization perturbs the host's own baseline region.
#[derive(Clone, Copy)]
enum Specialization {
    /// A wider workgroup, which the intrinsic verifier leaves free.
    Workgroup(u32),
    /// A zero-thread workgroup: structurally invalid IR, not an expensive plan.
    ZeroThreadWorkgroup,
    /// A grid one thread short of the iteration domain, which is the launch
    /// coverage rule rather than a target limit.
    UndercoveredGrid,
}

/// A separately authored provider that specializes one schedule axis.
///
/// It reads the host's baseline, perturbs exactly one field, and proposes the
/// result under the baseline's own cost. It stamps no provenance, declares no
/// resources, and states no boundary guarantee — there is no public spelling for
/// any of those, which is the seam working rather than an omission here.
struct AcmeProvider {
    identity: ProviderIdentity,
    specialization: Specialization,
}

impl AcmeProvider {
    fn new(name: &str, revision: u32, specialization: Specialization) -> Self {
        Self {
            identity: ProviderIdentity::new("acme", name, revision)
                .expect("the acme provider identity is valid"),
            specialization,
        }
    }

    fn specialize(&self, baseline: &ScheduledRegion) -> ScheduledRegion {
        let mut region = baseline.clone();
        match self.specialization {
            Specialization::Workgroup(threads) => {
                region.schedule.threads_per_workgroup = threads;
                region.schedule.launch.threads_per_workgroup = threads;
            }
            Specialization::ZeroThreadWorkgroup => {
                region.schedule.threads_per_workgroup = 0;
                region.schedule.launch.threads_per_workgroup = 0;
            }
            Specialization::UndercoveredGrid => {
                region.schedule.launch.grid_threads =
                    region.schedule.launch.grid_threads.saturating_sub(1);
            }
        }
        region
    }
}

impl PhysicalImplementationProvider for AcmeProvider {
    fn provenance(&self) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
        PhysicalProviderProvenance::new(self.identity.clone())
    }

    fn propose(&self, context: &ImplementationContext<'_>) -> ProviderOffer {
        let Some(baseline) = context.baseline() else {
            // Silence and a named decline are different answers, and this is the
            // second: the strategy applied and this subject admitted no shape
            // for it. A subject with no single-dispatch baseline is exactly the
            // published-and-consumed and unspellable cases the seam documents.
            return ProviderOffer::default().decline(DeclinedStrategy::new(
                WIDE_WORKGROUP_STRATEGY,
                StrategyDeclineCause::NoAdmissibleShape {
                    rule: "acme.no-single-dispatch-baseline",
                    extent: context.subject().covered_occurrences() as u64,
                },
            ));
        };
        ProviderOffer::proposing(vec![ImplementationProposal::scheduled_kernel(
            self.specialize(baseline.region()),
            TargetApplicability::for_targets([context.target_profile().profile_key().clone()]),
            // The host's own estimate for the region this specializes. A wider
            // workgroup changes no structural dimension, so inventing a lower
            // number would win a comparison that measured nothing.
            baseline.cost(),
        )])
    }
}

/// A provider that answers for nothing at all, with no decline either.
struct SilentProvider(ProviderIdentity);

/// A provider claiming Tiler's own governed physical-provider identity.
///
/// The identity is written out rather than read from the crate, because that is
/// the whole point: the governed identity is not a name this boundary exports,
/// so an impostor has to spell it — and installation refuses it anyway.
struct GovernedImpostor;

impl PhysicalImplementationProvider for GovernedImpostor {
    fn provenance(&self) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
        PhysicalProviderProvenance::new(
            ProviderIdentity::new("tiler", "prototype-serial-sum-physical", 1).unwrap(),
        )
    }

    fn propose(&self, _: &ImplementationContext<'_>) -> ProviderOffer {
        ProviderOffer::default()
    }
}

impl PhysicalImplementationProvider for SilentProvider {
    fn provenance(&self) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
        PhysicalProviderProvenance::new(self.0.clone())
    }

    fn propose(&self, _: &ImplementationContext<'_>) -> ProviderOffer {
        ProviderOffer::default()
    }
}

fn guarantee_source() -> TargetFactSource {
    TargetFactSource::external_guarantee(
        TargetFactProducerIdentity::new("test.acme-profile-producer.v1".to_owned(), 1).unwrap(),
        TargetNormativeReferenceIdentity::new("test.acme-profile-spec.v1".to_owned(), 1).unwrap(),
    )
}

/// A profile whose workgroup capacity is a *compile-time* declared fact.
///
/// Declared rather than deferred on purpose. The governed profile answers its
/// workgroup capacity through a prepared-entry query, so a specialization that
/// overran it would resolve as a deferred predicate and never produce the hard
/// rejection the negative control below needs to observe.
fn acme_profile(key: &str, max_threads_per_workgroup: u32) -> TargetProfile {
    let source = guarantee_source();
    let mut builder = TargetProfileBuilder::new(TargetProfileKey::new(key.to_owned()).unwrap());
    builder
        .declare_max_threads_per_grid_axis(65_535, source.clone())
        .unwrap();
    builder
        .declare_max_threads_per_workgroup(max_threads_per_workgroup, source.clone())
        .unwrap();
    builder
        .declare_max_buffer_bindings_per_entry(31, source.clone())
        .unwrap();
    builder
        .declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
        .unwrap();
    builder
        .declare_device_address_width(DeviceAddressWidth::Bits64, source.clone())
        .unwrap();
    builder.declare_device_memory(true, source.clone()).unwrap();
    builder
        .declare_local_memory_bytes(32_768, source.clone())
        .unwrap();
    let subject = ScalarArithmetic::f32();
    builder
        .declare_input_subnormals(
            subject.clone(),
            SubnormalMode::Preserve,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_result_subnormals(
            subject.clone(),
            SubnormalMode::Preserve,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_contraction(
            subject.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_reassociation(
            subject.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_permutation(
            subject.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_signed_zero(
            subject.clone(),
            NumericalPermission::Forbidden,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_nan_assumptions(
            subject.clone(),
            ExceptionalValueAssumption::MakeNoAssumption,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_infinity_assumptions(
            subject,
            ExceptionalValueAssumption::MakeNoAssumption,
            ScalarSupport::Exact,
            source.clone(),
        )
        .unwrap();
    builder
        .declare_dtype_dispatchability(
            F32::resolved_type(),
            DTypeDispatchability::Dispatchable,
            source,
        )
        .unwrap();
    builder.build().unwrap()
}

fn semantic_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([4, 1]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).unwrap();
    let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
    let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), sum)
        .unwrap();
    builder.build().unwrap()
}

/// Compiles the shared program against one profile and one provider environment.
fn compile_with(
    program: &SemanticProgram,
    profile: &TargetProfile,
    providers: &InstalledPhysicalProviders<'_>,
) -> Result<Compilation, CompileFailureClass> {
    let request = CompileRequest::new(
        program,
        NumericalContract::STRICT_F32,
        TargetRequest::new([profile.clone()]).unwrap(),
    )
    .with_physical_providers(providers.clone());
    let mut batch = match compile(request) {
        Ok(batch) => batch.into_targets(),
        Err(failure) => return Err(failure.class()),
    };
    let (_, outcome) = batch.pop().unwrap().into_parts();
    outcome.map_err(|failure| failure.class())
}

/// Every physical provider identity any retained alternative selected.
fn selected_provider_identities(compilation: &Compilation) -> BTreeSet<String> {
    compilation
        .alternatives()
        .flat_map(|alternative| {
            alternative
                .selected_physical_providers()
                .map(|selected| selected.provider_explain_subject().to_owned())
                .collect::<Vec<_>>()
        })
        .collect()
}

/// **An installed provider reaches the frontier, is re-verified, and its
/// implementations are retained beside the governed provider's.**
///
/// The three assertions are separable on purpose. The provider is *reachable*
/// (its identity appears at all), its bodies are *admitted* (they survived
/// whole-region verification, the request-subject binding, and feasibility), and
/// they are *additive* (the governed implementations are still there). Dropping
/// any one of the three would leave a seam that looked installed and was not.
#[test]
fn an_installed_provider_reaches_the_frontier_and_is_retained_additively() {
    let program = semantic_program();
    let profile = acme_profile("test.acme-retained.v1", 256);
    let specialized = AcmeProvider::new("wide-workgroup", 1, Specialization::Workgroup(32));

    let governed_only = compile_with(&program, &profile, &InstalledPhysicalProviders::governed())
        .expect("the governed environment compiles this program");
    let governed_providers = selected_provider_identities(&governed_only);
    let governed_alternatives = governed_only.alternatives().len();
    assert_eq!(
        governed_providers.len(),
        1,
        "the governed environment selects exactly one provider: {governed_providers:?}"
    );

    let composed = compile_with(
        &program,
        &profile,
        &InstalledPhysicalProviders::installed([
            &specialized as &dyn PhysicalImplementationProvider
        ])
        .expect("one identity installs"),
    )
    .expect("installing a partial provider does not refuse the compilation");
    let composed_providers = selected_provider_identities(&composed);

    assert!(
        composed_providers.contains(&explain_subject("wide-workgroup", 1)),
        "the installed provider never reached a retained plan: {composed_providers:?}"
    );
    assert!(
        composed_providers.is_superset(&governed_providers),
        "installing a provider displaced the governed one: {composed_providers:?}"
    );
    assert!(
        composed.alternatives().len() > governed_alternatives,
        "the specialization was not retained as an additional alternative"
    );

    // The selected plan names one provider per cover region, and every name is
    // one the host stamped rather than one a proposal carried.
    let selected = composed.selected().expect("a retained plan is selected");
    assert!(
        selected.selected_physical_providers().len() > 0,
        "the selected plan records no physical provenance at all"
    );
    for implementation in selected.selected_physical_providers() {
        assert!(
            composed_providers.contains(implementation.provider_explain_subject()),
            "the selected plan names a provider no enumeration offered"
        );
        assert_eq!(implementation.proposal_kind(), "scheduled-kernel");
    }
}

/// **Compilation is deterministic under an installed provider.**
///
/// The frontier's admitted set is canonical and provider-order-independent, so
/// two compilations of one request against one environment select the same plan.
/// Without this, the retention assertion above could be satisfied by a race.
#[test]
fn an_installed_provider_leaves_selection_deterministic() {
    let program = semantic_program();
    let profile = acme_profile("test.acme-deterministic.v1", 256);
    let specialized = AcmeProvider::new("wide-workgroup", 1, Specialization::Workgroup(32));
    let environment =
        InstalledPhysicalProviders::installed(
            [&specialized as &dyn PhysicalImplementationProvider],
        )
        .expect("one identity installs");

    let selected = |profile: &TargetProfile| {
        let compilation = compile_with(&program, profile, &environment).expect("it compiles");
        let plan = compilation.selected().expect("a plan is selected");
        (
            plan.stable_id().to_owned(),
            plan.selected_physical_providers()
                .map(|implementation| implementation.provider_explain_subject().to_owned())
                .collect::<Vec<_>>(),
        )
    };
    assert_eq!(selected(&profile), selected(&profile));
}

/// **A specialization the target cannot satisfy is a rejection, not a cost.**
///
/// The same provider, the same body, and the same program: only the profile's
/// declared workgroup capacity moves. Under a capacity that admits the
/// specialization the compilation retains it; under one that does not, the
/// proposal is refused as hard-infeasible and the governed implementation is
/// what remains — never an expensive plan that could win by being cheap enough.
#[test]
fn a_specialization_beyond_the_targets_capacity_is_refused_rather_than_costed() {
    let program = semantic_program();
    let specialized = AcmeProvider::new("wide-workgroup", 1, Specialization::Workgroup(512));
    let environment =
        InstalledPhysicalProviders::installed(
            [&specialized as &dyn PhysicalImplementationProvider],
        )
        .expect("one identity installs");

    let admitting = compile_with(
        &program,
        &acme_profile("test.acme-capacity-admits.v1", 512),
        &environment,
    )
    .expect("a target that admits the specialization compiles");
    assert!(
        selected_provider_identities(&admitting).contains(&explain_subject("wide-workgroup", 1)),
        "the positive control did not retain the specialization, so the negative one proves nothing"
    );

    let refusing = compile_with(
        &program,
        &acme_profile("test.acme-capacity-refuses.v1", 256),
        &environment,
    )
    .expect("a target that refuses the specialization still compiles the governed plan");
    let retained = selected_provider_identities(&refusing);
    assert!(
        !retained.contains(&explain_subject("wide-workgroup", 1)),
        "an infeasible specialization was retained as a plan: {retained:?}"
    );
    assert!(
        !retained.is_empty(),
        "refusing the specialization also lost the governed implementation"
    );
}

/// **Malformed provider output is a defect, never an empty offer.**
///
/// Two independent structural rules, perturbed separately so each shows which
/// assertion is load-bearing: a zero-thread workgroup and a grid one thread
/// short of its iteration domain. Both are bodies the intrinsic verifier refuses,
/// and both must fail the whole compilation closed as invalid compiler output —
/// a provider whose IR is wrong is a fault, and reporting it as "this provider
/// had nothing to offer" would make a defect indistinguishable from silence.
#[test]
fn a_structurally_invalid_body_fails_the_compilation_closed() {
    let program = semantic_program();
    let profile = acme_profile("test.acme-malformed.v1", 256);

    let silent = SilentProvider(ProviderIdentity::new("acme", "silent", 1).unwrap());
    let quiet = compile_with(
        &program,
        &profile,
        &InstalledPhysicalProviders::installed([&silent as &dyn PhysicalImplementationProvider])
            .expect("one identity installs"),
    )
    .expect("a provider that offers nothing is not an error");
    assert!(
        !selected_provider_identities(&quiet).contains(&explain_subject("silent", 1)),
        "a provider that proposed nothing appeared in a retained plan"
    );

    for (label, specialization) in [
        ("zero-thread workgroup", Specialization::ZeroThreadWorkgroup),
        ("undercovered grid", Specialization::UndercoveredGrid),
    ] {
        let malformed = AcmeProvider::new("malformed", 1, specialization);
        // Matched rather than `expect_err`, because the success value is a whole
        // compilation and printing it would bury the failure this test reports.
        let Err(class) = compile_with(
            &program,
            &profile,
            &InstalledPhysicalProviders::installed([
                &malformed as &dyn PhysicalImplementationProvider
            ])
            .expect("one identity installs"),
        ) else {
            panic!("{label}: a structurally invalid body compiled");
        };
        assert!(
            matches!(class, CompileFailureClass::InvalidCompilerOutput),
            "{label} was reported as {class:?} rather than invalid compiler output"
        );
    }
}

/// **A provider cannot forge the governed identity, and cannot install twice.**
///
/// Installation is where both are refused, before any compilation runs, so a
/// forged identity never reaches a plan's provenance to be compared against.
#[test]
fn installation_refuses_a_forged_or_repeated_identity() {
    let impostor = AcmeProvider::new("wide-workgroup", 1, Specialization::Workgroup(32));
    let same_identity = AcmeProvider::new("wide-workgroup", 1, Specialization::Workgroup(64));
    let error = InstalledPhysicalProviders::installed([
        &impostor as &dyn PhysicalImplementationProvider,
        &same_identity,
    ])
    .expect_err("one identity installed twice is refused");
    assert!(matches!(
        error,
        PhysicalProviderInstallationError::DuplicateIdentity { .. }
    ));
    assert!(error.to_string().contains("duplicate-identity"));

    let forged = InstalledPhysicalProviders::installed([
        &GovernedImpostor as &dyn PhysicalImplementationProvider
    ])
    .expect_err("claiming the governed identity is refused");
    assert!(matches!(
        forged,
        PhysicalProviderInstallationError::GovernedIdentity { .. }
    ));
}

/// **The one admissible cost model has a public spelling and only one.**
///
/// A provider that could attribute an estimate to a model of its own would let
/// two incomparable numbers be ranked. There is no public constructor that takes
/// a key, so the constant below is what a provider *reads* — and every estimate
/// it can build already carries it.
#[test]
fn every_constructible_estimate_carries_the_one_governed_cost_model() {
    use tiler_compiler::physical_provider::PhysicalCostEstimate;

    assert_eq!(GOVERNED_PHYSICAL_COST_MODEL_KEY, "tiler.cost.structural.v1");
    assert_eq!(
        PhysicalCostEstimate::structural(1, 4, 0).model_key(),
        GOVERNED_PHYSICAL_COST_MODEL_KEY
    );
}

/// **An installed provider is asked about every subject and can say nothing.**
///
/// The declining provider names a strategy it withheld for subjects with no
/// single-dispatch baseline; the compilation is unaffected either way. What this
/// pins is that the seam reaches the *whole* enumeration rather than one region:
/// the provider observes more subjects than the program has retained plans.
#[test]
fn an_installed_provider_is_offered_every_region_subject() {
    use std::cell::RefCell;

    struct CountingProvider {
        identity: ProviderIdentity,
        subjects: RefCell<BTreeSet<(String, usize, bool)>>,
    }

    impl PhysicalImplementationProvider for CountingProvider {
        fn provenance(
            &self,
        ) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
            PhysicalProviderProvenance::new(self.identity.clone())
        }

        fn propose(&self, context: &ImplementationContext<'_>) -> ProviderOffer {
            self.subjects.borrow_mut().insert((
                context.subject().role().to_owned(),
                context.subject().covered_occurrences(),
                context.baseline().is_some(),
            ));
            ProviderOffer::default()
        }
    }

    let counting = CountingProvider {
        identity: ProviderIdentity::new("acme", "counting", 1).unwrap(),
        subjects: RefCell::new(BTreeSet::new()),
    };
    let program = semantic_program();
    compile_with(
        &program,
        &acme_profile("test.acme-counting.v1", 256),
        &InstalledPhysicalProviders::installed([&counting as &dyn PhysicalImplementationProvider])
            .expect("one identity installs"),
    )
    .expect("a counting provider changes nothing about the compilation");

    let observed = counting.subjects.into_inner();
    assert!(
        observed.len() >= 3,
        "the provider was offered {} distinct subjects, which is fewer than this program's regions",
        observed.len()
    );
    assert!(
        observed.iter().any(|(_, _, has_baseline)| *has_baseline),
        "no subject offered a baseline, so the specialization path is unreachable"
    );
    assert!(
        observed.iter().all(|(_, covered, _)| *covered > 0),
        "a subject naming no occurrence reached an installed provider: {observed:?}"
    );
}

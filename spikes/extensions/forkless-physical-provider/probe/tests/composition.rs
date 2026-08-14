//! What a separately authored provider can and cannot do, from outside the
//! workspace that defines the seam.
//!
//! Each test is one claim. Together they bound the answer from both sides: an
//! out-of-tree provider is installable, is asked about every region subject, is
//! re-verified rather than believed, is retained *beside* the governed provider
//! rather than in place of it, and its body emits through stock `tiler-metal`
//! unchanged — while the five subjects the host reserves stay unreachable, which
//! is `ui.rs`'s half of the answer because an absent API is evidence only when
//! the compiler says it is absent.
//!
//! What this file deliberately does not restate is the in-tree integration
//! evidence at `crates/tiler-compiler/tests/external_physical_provider.rs`:
//! determinism under an installed provider, the every-subject enumeration
//! count, and the hard-feasibility refusal against a declared workgroup capacity
//! are measured there, against a fabricated profile that this probe has no
//! reason to duplicate. What only this workspace can say is that the same
//! provider source compiles and runs against `tiler-compiler` resolved as an
//! ordinary path dependency of a *different* workspace, with its own lockfile.

use acme_provider::{AcmeProvider, Specialization};
use tiler_compiler::physical_provider::{
    InstalledPhysicalProviders, PhysicalImplementationProvider, PhysicalProviderInstallationError,
};
use tiler_compiler::session::{
    Compilation, CompileFailureClass, CompileRequest, NumericalContract, compile,
};
use tiler_compiler::target::{TargetProfile, TargetRequest};
use tiler_ir::kernel::lower_scheduled_region;
use tiler_ir::schedule::ScheduledRegionBuilder;
use tiler_ir::semantic::{ProviderIdentity, SemanticProgram};
use tiler_metal::emit::emit_translation_unit;
use tiler_metal::target::{
    LaunchIndexRealization, MetalDeploymentMinimum, MetalEmissionRealization,
    MetalFloatArithmeticType, MetalFlushedZeroSign, MetalPlatform, MetalSubnormalArithmetic,
    MetalSubnormalArithmeticFacts, MetalTargetFacts, MslLanguageVersion,
};

use composition_probe::{COLUMNS, ROWS, serial_sum_program};

/// Tiler's own governed physical provider, spelled out.
///
/// The boundary does not export it — that is deliberate, and it is why the
/// forged-identity claim below has to write it down. Every other use of it here
/// is a *presence* check on a compilation's own output, so a rename of the
/// governed provider turns this constant into a failing assertion rather than
/// into a silently weaker one.
const GOVERNED_PHYSICAL_PROVIDER: (&str, &str, u32) = ("tiler", "prototype-serial-sum-physical", 1);

/// The Apple row's measured facts, restated here because a provider crate that
/// reuses stock emission must supply them; they are not Tiler's to imply.
fn target_facts() -> MetalTargetFacts {
    MetalTargetFacts::new(
        MslLanguageVersion::Metal3_1,
        MetalPlatform::MacOs,
        MetalDeploymentMinimum::new(14, 0),
        MetalSubnormalArithmeticFacts::unmeasured()
            .stating(
                MetalFloatArithmeticType::F32,
                MetalSubnormalArithmetic::FlushesToZero {
                    zero_sign: MetalFlushedZeroSign::PreservesSign,
                },
            )
            .stating(
                MetalFloatArithmeticType::F16,
                MetalSubnormalArithmetic::PreservesSubnormals,
            ),
        31,
    )
}

fn governed_identity() -> ProviderIdentity {
    let (namespace, name, revision) = GOVERNED_PHYSICAL_PROVIDER;
    ProviderIdentity::new(namespace, name, revision).expect("the governed identity is well formed")
}

/// Compiles the shared program against the governed profile and one environment.
fn compile_with(
    program: &SemanticProgram,
    providers: &InstalledPhysicalProviders<'_>,
) -> Result<Compilation, CompileFailureClass> {
    let request = CompileRequest::new(
        program,
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
        TargetRequest::new([TargetProfile::governed()]).expect("one profile is a valid request"),
    )
    .with_physical_providers(providers.clone());
    let mut batch = match compile(request) {
        Ok(batch) => batch.into_targets(),
        Err(failure) => return Err(failure.class()),
    };
    let (_, outcome) = batch
        .pop()
        .expect("one requested profile produces one result")
        .into_parts();
    outcome.map_err(|failure| failure.class())
}

/// Every physical provider identity any retained alternative selected.
fn selected_provider_identities(compilation: &Compilation) -> Vec<ProviderIdentity> {
    let mut identities: Vec<ProviderIdentity> = compilation
        .alternatives()
        .flat_map(|alternative| {
            alternative
                .selected_physical_providers()
                .map(|selected| selected.provider().clone())
                .collect::<Vec<_>>()
        })
        .collect();
    identities.sort_by_key(ToString::to_string);
    identities.dedup();
    identities
}

/// Claim 1 — the public compile path is reachable from outside the workspace.
///
/// The control every negative result below depends on. Without it, an
/// observation that the custom provider contributed nothing would be
/// indistinguishable from a probe that never compiled anything.
#[test]
fn the_governed_compile_path_is_reachable_from_an_out_of_tree_crate() {
    let program = serial_sum_program(ROWS, COLUMNS);
    let compilation = compile_with(&program, &InstalledPhysicalProviders::governed())
        .expect("the governed program compiles under the flushing contract");
    assert_eq!(
        compilation.target_profile_key(),
        "tiler.prototype-target-neutral-baseline.v1",
    );
    assert!(
        compilation.selected().is_some(),
        "a compiled program has a selected plan",
    );
    assert_eq!(
        selected_provider_identities(&compilation),
        [governed_identity()],
        "the governed environment selects exactly the governed physical provider",
    );
}

/// Claim 2 — an out-of-tree provider reaches a retained plan, additively.
///
/// This is the claim the operation-extension contract names as the rung above
/// an in-package integration test, and its three assertions are separable on
/// purpose. The provider is *reachable* — its identity appears at all; its
/// bodies are *admitted* — they survived whole-region verification, the
/// request-subject binding, and the feasibility decision; and they are
/// *additive* — the governed implementations are still there. Dropping any one
/// would leave a seam that looked installed and was not.
#[test]
fn an_out_of_tree_provider_reaches_a_retained_plan_beside_the_governed_one() {
    let program = serial_sum_program(ROWS, COLUMNS);
    let governed_only = compile_with(&program, &InstalledPhysicalProviders::governed())
        .expect("the governed environment compiles this program");
    let governed_alternatives = governed_only.alternatives().len();

    let specialized = AcmeProvider::new(Specialization::WideWorkgroup);
    let environment =
        InstalledPhysicalProviders::installed(
            [&specialized as &dyn PhysicalImplementationProvider],
        )
        .expect("one identity installs");
    assert_eq!(
        environment.identities(),
        [acme_provider::identity()],
        "the installed set reports the caller's provider and only it",
    );

    let composed = compile_with(&program, &environment)
        .expect("installing a partial provider does not refuse the compilation");
    // Set equality rather than two `contains` checks. `contains` would stay
    // green if a third physical provider appeared in this compilation's plans,
    // and a provider nobody installed reaching a retained plan is exactly the
    // kind of change this claim exists to notice. The identities sort
    // `acme::…` before `tiler::…`.
    let retained = selected_provider_identities(&composed);
    assert_eq!(
        retained,
        [acme_provider::identity(), governed_identity()],
        "the composed environment must retain the installed provider and the \
         governed one, and nothing else",
    );
    assert!(
        composed.alternatives().len() > governed_alternatives,
        "the specialization was not retained as an additional alternative: {} against {governed_alternatives}",
        composed.alternatives().len(),
    );

    // Every name a plan carries is one the host stamped. The provider proposed
    // a body, an applicability predicate, and a cost, and nothing else; there is
    // no identity field on a proposal for it to have filled in.
    let selected = composed.selected().expect("a retained plan is selected");
    assert!(
        selected.selected_physical_providers().len() > 0,
        "the selected plan records no physical provenance at all",
    );
    for implementation in selected.selected_physical_providers() {
        assert!(
            retained.contains(implementation.provider()),
            "the selected plan names a provider no enumeration offered",
        );
        assert_eq!(implementation.proposal_kind(), "scheduled-kernel");
    }
}

/// Claim 3 — the provider is offered the host's own spelling of each subject.
///
/// The specialization path exists only because the host hands back a baseline;
/// a seam that stated none of it would admit only providers that had
/// reimplemented this crate's normalization. So this pins both halves: a
/// baseline was offered, and the body proposed for it differs from it in the
/// one axis this provider specializes and in nothing else.
#[test]
fn the_provider_specializes_the_hosts_own_baseline_in_one_axis() {
    let program = serial_sum_program(ROWS, COLUMNS);
    let specialized = AcmeProvider::new(Specialization::WideWorkgroup);
    compile_with(
        &program,
        &InstalledPhysicalProviders::installed([
            &specialized as &dyn PhysicalImplementationProvider
        ])
        .expect("one identity installs"),
    )
    .expect("the composed environment compiles");

    let exchanged = specialized.exchanged();
    assert!(
        !exchanged.is_empty(),
        "no subject offered a baseline, so the specialization path is unreachable",
    );
    for exchange in exchanged {
        assert_eq!(
            exchange.baseline.index, exchange.proposed.index,
            "the specialization reached the index region the subject binding compares",
        );
        assert_eq!(
            exchange.proposed.schedule.threads_per_workgroup,
            acme_provider::SPECIALIZED_THREADS_PER_WORKGROUP,
        );
        // The whole difference, pinned to the two fields the intrinsic verifier
        // requires to agree. Without this the claim above would hold for a body
        // that differed in some second way nothing here names.
        let mut normalized = exchange.proposed.clone();
        normalized.schedule.threads_per_workgroup =
            exchange.baseline.schedule.threads_per_workgroup;
        normalized.schedule.launch.threads_per_workgroup =
            exchange.baseline.schedule.launch.threads_per_workgroup;
        assert_eq!(exchange.baseline, normalized);

        let baseline = ScheduledRegionBuilder::from_region(exchange.baseline.clone())
            .build()
            .expect("the host's own baseline verifies");
        let proposed = ScheduledRegionBuilder::from_region(exchange.proposed.clone())
            .build()
            .expect("the specialized body verifies");
        assert_ne!(
            baseline.canonical_identity(),
            proposed.canonical_identity(),
            "the workgroup width is folded into canonical identity, so the two are \
             additive alternatives rather than one implementation twice",
        );
    }
}

/// Claim 4 — the specialized body reuses stock Metal emission unchanged.
///
/// `acme-provider` does not depend on `tiler-metal` at all: the probe lowers the
/// body that actually reached the frontier with `tiler_ir::kernel::
/// lower_scheduled_region` and emits it with `tiler_metal::emit::
/// emit_translation_unit`, both public. So the reuse the spike asks about is
/// available — no interface prevents a custom physical provider from reusing
/// standard emission, because emission consumes verified kernels and knows
/// nothing about who proposed them.
///
/// It also records two boundaries of that reuse. Launch geometry is not part of
/// the emitted body, so the specialization is invisible in the statements and
/// would have to be carried by the dispatch. But the entry-point symbol *is*
/// identity-derived, and the identity folds the workgroup width — so two
/// alternatives of one region emit distinct entry points from identical bodies,
/// and a translation unit holding both would not collide.
#[test]
fn the_specialized_body_reuses_stock_metal_emission() {
    let program = serial_sum_program(ROWS, COLUMNS);
    let specialized = AcmeProvider::new(Specialization::WideWorkgroup);
    compile_with(
        &program,
        &InstalledPhysicalProviders::installed([
            &specialized as &dyn PhysicalImplementationProvider
        ])
        .expect("one identity installs"),
    )
    .expect("the composed environment compiles");

    let target = target_facts();
    let emission = MetalEmissionRealization::new(LaunchIndexRealization::ThreadPositionInGridUInt);
    let exchanged = specialized.exchanged();
    assert!(!exchanged.is_empty(), "nothing was proposed to emit");
    for exchange in exchanged {
        let lowered = |region| {
            lower_scheduled_region(
                &ScheduledRegionBuilder::from_region(region)
                    .build()
                    .expect("the body verifies"),
            )
            .expect("the body refines to structured kernel IR")
        };
        let proposed = lowered(exchange.proposed);
        let baseline = lowered(exchange.baseline);

        let unit = emit_translation_unit(&[&proposed], &target, emission)
            .expect("the specialized kernel emits through stock tiler-metal");
        let entry = &unit.entry_points()[0];
        assert!(
            unit.source()
                .contains(&format!("kernel void {}(", entry.symbol())),
            "stock emission produced the entry point",
        );
        assert!(
            unit.unstated_subnormal_arithmetic().is_empty(),
            "every arithmetic type this unit used has a stated fact",
        );
        unit.require_declared_realization()
            .expect("the realization the compiler resolved is what Apple f32 delivers");

        let baseline_unit = emit_translation_unit(&[&baseline], &target, emission)
            .expect("the governed-shaped kernel emits through stock tiler-metal");
        assert_ne!(
            baseline_unit.entry_points()[0].symbol(),
            entry.symbol(),
            "the entry symbol is derived from an identity the workgroup width \
             participates in, so the two alternatives do not collide",
        );
        assert_eq!(
            without_identity_digests(unit.source()),
            without_identity_digests(baseline_unit.source()),
            "with identity-derived names elided the two bodies are the same: launch \
             geometry is carried by the dispatch, not by the emitted statements",
        );
    }
}

/// Replaces every 16-hex-digit identity digest with a fixed placeholder.
///
/// Sixteen is the exact width `tiler-metal` renders a kernel or scheduled-region
/// identity digest at, and no immediate, mask, or helper name in the emitted
/// vocabulary reaches that width — the widest is the eight-digit `0x7fc00000u`
/// NaN pattern. So this elides identity and nothing else, which is what makes
/// the comparison above a statement about the kernel body.
fn without_identity_digests(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut run = String::new();
    for character in source.chars() {
        if character.is_ascii_hexdigit() && !character.is_ascii_uppercase() {
            run.push(character);
            continue;
        }
        flush_digest_run(&mut output, &mut run);
        output.push(character);
    }
    flush_digest_run(&mut output, &mut run);
    output
}

fn flush_digest_run(output: &mut String, run: &mut String) {
    if run.len() == 16 {
        output.push_str("<digest>");
    } else {
        output.push_str(run);
    }
    run.clear();
}

/// Claim 5 — a trusted provider is not a believed one.
///
/// Two independent structural rules, perturbed separately so each shows which
/// assertion is load-bearing: a zero-thread workgroup and a grid one thread
/// short of its iteration domain. Both are bodies the host's own intrinsic
/// verifier refuses, and both must fail the whole compilation closed as invalid
/// compiler output — reporting a provider whose IR is wrong as "this provider
/// had nothing to offer" would make a defect indistinguishable from silence.
#[test]
fn a_structurally_invalid_body_fails_the_compilation_closed() {
    let program = serial_sum_program(ROWS, COLUMNS);
    for (label, specialization) in [
        ("zero-thread workgroup", Specialization::ZeroThreadWorkgroup),
        ("undercovered grid", Specialization::UndercoveredGrid),
    ] {
        let malformed = AcmeProvider::new(specialization);
        // Matched rather than `expect_err`, because the success value is a whole
        // compilation and printing it would bury the failure this test reports.
        let Err(class) = compile_with(
            &program,
            &InstalledPhysicalProviders::installed([
                &malformed as &dyn PhysicalImplementationProvider
            ])
            .expect("one identity installs"),
        ) else {
            panic!("{label}: a structurally invalid body compiled");
        };
        assert!(
            matches!(class, CompileFailureClass::InvalidCompilerOutput),
            "{label} was reported as {class:?} rather than invalid compiler output",
        );
    }
}

/// Claim 6 — the offered lowering and physical environments are distinct.
///
/// A compilation exposes two complete frozen environments under qualified
/// names. Lowering identities cannot leak into the physical environment, and
/// neither the governed nor caller-installed physical identity can leak into
/// the lowering environment. The physical population is ordered governed first
/// and then caller installation order; the selected population remains a
/// separate statement about which providers a retained plan actually used.
#[test]
fn the_offered_lowering_and_physical_environments_are_distinct() {
    let program = serial_sum_program(ROWS, COLUMNS);
    let specialized = AcmeProvider::new(Specialization::WideWorkgroup);
    let composed = compile_with(
        &program,
        &InstalledPhysicalProviders::installed([
            &specialized as &dyn PhysicalImplementationProvider
        ])
        .expect("one identity installs"),
    )
    .expect("the composed environment compiles");

    let lowering = composed.offered_lowering_providers();
    assert!(
        !lowering.is_empty(),
        "the governed lowering providers are disclosed, so an empty answer here \
         would make the two assertions below vacuous",
    );
    assert!(
        !lowering.contains(&acme_provider::identity()),
        "the installed physical provider leaked into the lowering environment: {lowering:?}",
    );
    assert!(
        !lowering.contains(&governed_identity()),
        "the governed physical provider leaked into the lowering environment: {lowering:?}",
    );

    let physical = composed.offered_physical_providers();
    assert_eq!(
        physical,
        [governed_identity(), acme_provider::identity()],
        "the physical environment is governed first and then caller installation order",
    );
    // The control that keeps *offered* distinct from *selected*: this provider
    // reached a retained plan too, but that is a separate population and claim.
    assert!(
        selected_provider_identities(&composed).contains(&acme_provider::identity()),
        "the offered physical provider did not reach a retained plan",
    );
}

/// Claim 7 — installation refuses a forged or repeated identity.
///
/// Both are refused at installation, before any compilation runs, so a forged
/// identity never reaches a plan's provenance to be compared against. The
/// revision pair is what stops the duplicate check degrading into the weaker
/// namespace-and-name one: two revisions of one provider are two identities and
/// both install.
#[test]
fn installation_refuses_a_forged_or_repeated_identity() {
    let first = AcmeProvider::new(Specialization::WideWorkgroup);
    let repeated = AcmeProvider::new(Specialization::UndercoveredGrid);
    let error = InstalledPhysicalProviders::installed([
        &first as &dyn PhysicalImplementationProvider,
        &repeated,
    ])
    .expect_err("one identity installed twice is refused");
    assert!(matches!(
        error,
        PhysicalProviderInstallationError::DuplicateIdentity { .. }
    ));
    assert!(error.to_string().contains("duplicate-identity"));

    let revised = AcmeProvider::named(
        acme_provider::NAME,
        acme_provider::REVISION + 1,
        Specialization::WideWorkgroup,
    );
    InstalledPhysicalProviders::installed([
        &first as &dyn PhysicalImplementationProvider,
        &revised,
    ])
    .expect("two revisions of one provider are two identities");

    let (namespace, name, revision) = GOVERNED_PHYSICAL_PROVIDER;
    let impostor = AcmeProvider::impersonating(namespace, name, revision);
    let forged =
        InstalledPhysicalProviders::installed([&impostor as &dyn PhysicalImplementationProvider])
            .expect_err("claiming the governed identity is refused");
    match forged {
        PhysicalProviderInstallationError::GovernedIdentity { identity } => {
            assert_eq!(identity, governed_identity());
        }
        other => panic!("the governed-identity collision was reported as {other:?}"),
    }
}

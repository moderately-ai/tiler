//! What a separately authored provider can and cannot reach today.
//!
//! Each test is one claim. Together they bound the answer from both sides: the
//! compile path is reachable from outside the workspace, the proposal body is
//! constructible, verifiable, and emittable through stock `tiler-metal` — and
//! the frontier that would admit it as an alternative is reachable by nobody
//! outside `tiler-compiler`. The compile-fail half of that second statement is
//! in `ui.rs`, because a missing item is evidence only when the compiler says
//! so.

use acme_provider::PointwiseSubject;
use tiler_compiler::session::{NumericalContract, compile_governed};
use tiler_ir::kernel::lower_scheduled_region;
use tiler_ir::schedule::{ScheduledRegionBuilder, ScheduledRegionDiagnostic};
use tiler_metal::emit::emit_translation_unit;
use tiler_metal::target::{
    LaunchIndexRealization, MetalDeploymentMinimum, MetalEmissionRealization,
    MetalFloatArithmeticType, MetalFlushedZeroSign, MetalPlatform, MetalSubnormalArithmetic,
    MetalSubnormalArithmeticFacts, MetalTargetFacts, MslLanguageVersion,
};

use composition_probe::{COLUMNS, ROWS, serial_sum_program};

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

/// Claim 1 — the public compile path is reachable from outside the workspace.
///
/// The control the negative results depend on. Without it, every "the custom
/// provider contributed nothing" observation below would be indistinguishable
/// from a probe that never compiled anything.
#[test]
fn governed_compile_path_is_reachable_from_an_out_of_tree_crate() {
    let program = serial_sum_program(ROWS, COLUMNS);
    let compilation = compile_governed(&program, NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32)
        .expect("the governed program compiles under the flushing contract");
    assert_eq!(
        compilation.target_profile_key(),
        "tiler.prototype-target-neutral-baseline.v1",
    );
    assert!(
        compilation.selected().is_some(),
        "a compiled program has a selected plan",
    );
    // Measured, not assumed: for this shape the portfolio's non-dominated set
    // is the single fused alternative. Recorded because it is the reason the
    // ticket's "registration order does not pick the winner" demonstration is
    // out of reach from here — which alternative wins is decided by structural
    // cost domination inside `select_physical_plans`, and the public surface
    // reports only the surviving set.
    let fused: Vec<bool> = compilation
        .alternatives()
        .map(|alternative| alternative.is_fused())
        .collect();
    assert_eq!(fused, [true]);
}

/// Claim 2 — no physical provider is nameable at the public boundary.
///
/// `Compilation::offered_providers` is the only provider set the public surface
/// reports, and it is populated from the *lowering* capability registry
/// (`crates/tiler-compiler/src/session.rs:1443`). The governed physical
/// provider's own identity, `tiler/prototype-serial-sum-physical`, does not
/// appear in it — so the boundary neither accepts a physical provider nor
/// discloses the one it uses. A third party cannot register, and cannot observe
/// that it failed to.
#[test]
fn public_surface_names_no_physical_provider() {
    let program = serial_sum_program(ROWS, COLUMNS);
    let compilation = compile_governed(&program, NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32)
        .expect("the governed program compiles under the flushing contract");
    let offered: Vec<String> = compilation
        .offered_providers()
        .iter()
        .map(ToString::to_string)
        .collect();
    assert!(
        !offered.is_empty(),
        "the governed lowering providers are disclosed",
    );
    assert!(
        !offered
            .iter()
            .any(|provider| provider.contains(acme_provider::NAMESPACE)),
        "the separately authored provider is absent: {offered:?}",
    );
    assert!(
        !offered
            .iter()
            .any(|provider| provider.contains("prototype-serial-sum-physical")),
        "even the governed physical provider is not disclosed here: {offered:?}",
    );
}

/// Claim 3 — the specialized body is constructible and intrinsically verifies.
///
/// `ScheduledRegionBuilder::from_region(..).build()` is the exact call the
/// frontier makes on a provider's body before admitting it
/// (`crates/tiler-compiler/src/physical.rs:652`), so this exercises the real
/// verifier rather than a spike-local imitation. What it does *not* exercise is
/// the request-subject binding and the feasibility assessment either side of
/// it, both of which are private.
#[test]
fn specialized_body_is_constructible_and_intrinsically_verifies() {
    let subject = PointwiseSubject::spike_default();
    let baseline =
        ScheduledRegionBuilder::from_region(acme_provider::governed_shaped_region(subject))
            .build()
            .expect("the governed-shaped body verifies");
    let specialized =
        ScheduledRegionBuilder::from_region(acme_provider::specialized_region(subject))
            .build()
            .expect("the specialized body verifies");

    assert_eq!(baseline.region().schedule.threads_per_workgroup, 1);
    assert_eq!(
        specialized.region().schedule.threads_per_workgroup,
        acme_provider::SPECIALIZED_THREADS_PER_WORKGROUP,
    );
    assert_ne!(
        baseline.canonical_identity(),
        specialized.canonical_identity(),
        "the workgroup width is folded into canonical identity, so the two are \
         additive alternatives rather than one implementation twice",
    );
    assert_eq!(
        baseline.region().index,
        specialized.region().index,
        "the specialization is confined to the schedule; the index region the \
         request-subject binding compares is unchanged",
    );
}

/// Claim 4 — the specialized body reuses stock Metal emission unchanged.
///
/// The provider crate does not depend on `tiler-metal` at all: the probe lowers
/// its region with `tiler_ir::kernel::lower_scheduled_region` and emits with
/// `tiler_metal::emit::emit_translation_unit`, both public. So the reuse the
/// ticket asks about is available — no interface prevents a custom physical
/// provider from reusing standard emission, because emission consumes verified
/// kernels and knows nothing about who proposed them.
///
/// It also records two boundaries of that reuse. Launch geometry is not part of
/// the emitted body, so the specialization is invisible in the statements and
/// would have to be carried by the dispatch that runs the kernel. But the
/// entry-point symbol *is* identity-derived, and the scheduled-region identity
/// the symbol folds includes the workgroup width — so two alternatives of one
/// region emit distinct entry points from identical bodies, and a translation
/// unit holding both would not collide.
#[test]
fn specialized_body_reuses_stock_metal_emission() {
    let subject = PointwiseSubject::spike_default();
    let baseline = lower_scheduled_region(
        &ScheduledRegionBuilder::from_region(acme_provider::governed_shaped_region(subject))
            .build()
            .expect("the governed-shaped body verifies"),
    )
    .expect("the governed-shaped body refines to structured kernel IR");
    let specialized = lower_scheduled_region(
        &ScheduledRegionBuilder::from_region(acme_provider::specialized_region(subject))
            .build()
            .expect("the specialized body verifies"),
    )
    .expect("the specialized body refines to structured kernel IR");

    let target = target_facts();
    let emission = MetalEmissionRealization::new(LaunchIndexRealization::ThreadPositionInGridUInt);
    let unit = emit_translation_unit(&[&specialized], &target, emission)
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
        .expect("the flushing realization this provider declares is what Apple f32 delivers");

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

/// Claim 5 — a malformed body is rejected by that same verifier.
///
/// The perturbation is one field: `launch.grid_threads` one short of the
/// iteration domain. The failure is a typed `LaunchCoverage` diagnostic, and it
/// is the diagnostic the compiler maps onto `PhysicalError::Intrinsic`
/// (`crates/tiler-compiler/src/physical.rs:684`) when a provider's body fails
/// verification. Nothing about the perturbation is detectable by inspecting the
/// provider's identity or cost: only re-verifying the body finds it.
#[test]
fn malformed_body_is_rejected_by_the_same_intrinsic_verifier() {
    let subject = PointwiseSubject::spike_default();
    let error =
        ScheduledRegionBuilder::from_region(acme_provider::malformed_specialized_region(subject))
            .build()
            .expect_err("a launch plan that does not cover the domain must not verify");
    assert_eq!(
        error.diagnostics(),
        [ScheduledRegionDiagnostic::LaunchCoverage],
    );
    assert_eq!(error.diagnostics()[0].rule(), "launch-coverage");
}

/// Claim 6 — the perturbation is detected because of the check, not by accident.
///
/// The control for claim 5. If `malformed_specialized_region` differed from
/// `specialized_region` in some way the verifier happened to reject for another
/// reason, claim 5 would still be green and would still prove nothing about
/// launch coverage. This pins the difference to the single perturbed field.
#[test]
fn only_the_perturbed_field_differs() {
    let subject = PointwiseSubject::spike_default();
    let good = acme_provider::specialized_region(subject);
    let bad = acme_provider::malformed_specialized_region(subject);
    assert_eq!(good.index, bad.index);
    assert_eq!(
        good.schedule.launch.grid_threads,
        bad.schedule.launch.grid_threads + 1,
    );
    let mut normalized = bad;
    normalized.schedule.launch.grid_threads = good.schedule.launch.grid_threads;
    assert_eq!(good, normalized);
}

use super::support::{request_with_targets, semantic, tensor_add_chain};
use super::*;

struct CensusSpecialist {
    threads: u32,
}

impl PhysicalImplementationProvider for CensusSpecialist {
    fn provenance(
        &self,
    ) -> Result<
        crate::frontier::PhysicalProviderProvenance,
        crate::frontier::PhysicalProviderProvenanceError,
    > {
        crate::frontier::PhysicalProviderProvenance::new(
            tiler_ir::semantic::ProviderIdentity::new("tiler.test", "frontier-census", 1)
                .expect("the census provider identity is valid"),
        )
    }

    fn propose(
        &self,
        context: &crate::frontier::ImplementationContext<'_>,
    ) -> crate::frontier::ProviderOffer {
        let Some(baseline) = context.baseline() else {
            return crate::frontier::ProviderOffer::default().decline(
                crate::frontier::DeclinedStrategy::new(
                    "tiler.test.frontier-census",
                    crate::frontier::StrategyDeclineCause::NoAdmissibleShape {
                        rule: "test.frontier-census.no-baseline",
                        extent: u64::try_from(context.subject().covered_occurrences())
                            .unwrap_or(u64::MAX),
                    },
                ),
            );
        };
        let mut region = baseline.region().clone();
        region.schedule.threads_per_workgroup = self.threads;
        region.schedule.launch.threads_per_workgroup = self.threads;
        crate::frontier::ProviderOffer::proposing(vec![
            crate::frontier::ImplementationProposal::scheduled_kernel(
                region,
                crate::frontier::TargetApplicability::for_targets([context
                    .request()
                    .target_profile()
                    .profile_key()
                    .clone()]),
                baseline.cost(),
            ),
        ])
    }
}

/// Request-wide physical work census used by the retained frontier-budget
/// calibration spike.
///
/// This test stays crate-private because raw provider outcomes and intermediate
/// plan-combination populations are compiler-owned accounting facts, not a new
/// public observation seam. The public spike independently drives target order,
/// installed-provider emissions, and host cost.
#[test]
fn request_wide_physical_planning_population_is_pinned() {
    use tiler_ir::schedule::{FlushedZeroSign, NumericalPermission, SubnormalMode};

    let perturb = std::env::var("TILER_FRONTIER_CENSUS_PERTURB").ok();
    let strict_target_count = if perturb.as_deref() == Some("target-count") {
        15
    } else {
        16
    };
    let mut strict_profiles = (0..strict_target_count)
        .map(|index| {
            TargetProfile::numerical_realization_for_test(
                &format!("test.frontier-census-strict-{index}.v1"),
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
            )
        })
        .collect::<Vec<_>>();
    if perturb.as_deref() == Some("target-order") {
        strict_profiles.reverse();
    }
    let strict_program = semantic(false);
    let strict_request = request_with_targets(
        &strict_program,
        strict_profiles,
        vec![StrictF32NumericalContract::governed()],
    );
    let (strict_product, strict) =
        crate::workcount::observe_physical_planning(|| compile(strict_request));
    let strict_product = strict_product.expect("the sixteen-target strict request compiles");
    assert_eq!(
        strict_product.targets.len(),
        16,
        "the request-wide census must reach all sixteen admitted target slots",
    );
    let strict_keys = strict_product
        .targets
        .iter()
        .map(|target| target.target_profile().profile_key().as_str())
        .collect::<Vec<_>>();
    let expected_strict_keys = (0..16)
        .map(|index| format!("test.frontier-census-strict-{index}.v1"))
        .collect::<Vec<_>>();
    assert_eq!(
        strict_keys, expected_strict_keys,
        "the compiler must preserve caller target order in the population under test",
    );
    assert_eq!(
        strict,
        crate::workcount::PhysicalPlanningCensus {
            provider_invocations: 272,
            proposals: 48,
            declines: 256,
            proposal_assessments_started: 48,
            schedule_verifications: 48,
            admitted_implementations: 48,
            retained_implementations: 48,
            proposal_rejections: 0,
            frontier_rejections: 256,
            admitted_sort_items: 48,
            rejection_sort_items: 256,
            plan_combinations: 32,
            accepted_plan_combinations: 32,
            rejected_plan_combinations: 0,
            retained_complete_plans: 32,
            complete_plan_sort_items: 32,
        },
        "the governed provider's request-wide work population changed",
    );
    println!("MEASURE request-wide strict governed census: {strict:?}");

    let installed_profiles = (0..16)
        .map(|index| {
            TargetProfile::numerical_realization_for_test(
                &format!("test.frontier-census-installed-{index}.v1"),
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
            )
        })
        .collect();
    let installed_request = request_with_targets(
        &strict_program,
        installed_profiles,
        vec![StrictF32NumericalContract::governed()],
    );
    let governed = crate::frontier::GovernedPhysicalProvider;
    let specialist = CensusSpecialist { threads: 32 };
    let (installed_product, installed) = crate::workcount::observe_physical_planning(|| {
        let providers: Vec<&dyn PhysicalImplementationProvider> =
            if perturb.as_deref() == Some("governed-outcome-inclusion") {
                vec![&specialist]
            } else {
                vec![&governed, &specialist]
            };
        compile_with_physical_providers(installed_request, providers)
    });
    let installed_product = installed_product.expect("the installed-provider request compiles");
    assert_eq!(installed_product.targets.len(), 16);
    assert_eq!(
        installed,
        crate::workcount::PhysicalPlanningCensus {
            provider_invocations: 544,
            proposals: 96,
            declines: 480,
            proposal_assessments_started: 96,
            schedule_verifications: 96,
            admitted_implementations: 96,
            retained_implementations: 96,
            proposal_rejections: 0,
            frontier_rejections: 480,
            admitted_sort_items: 96,
            rejection_sort_items: 480,
            plan_combinations: 96,
            accepted_plan_combinations: 96,
            rejected_plan_combinations: 0,
            retained_complete_plans: 96,
            complete_plan_sort_items: 96,
        },
        "the raw-outcome authority must include governed and installed emissions",
    );
    println!("MEASURE request-wide strict installed census: {installed:?}");

    let infeasible_profiles = (0..16)
        .map(|index| {
            TargetProfile::governed_with_workgroup_limit_for_test(
                &format!("test.frontier-census-infeasible-{index}.v1"),
                64,
            )
        })
        .collect();
    let infeasible_request = request_with_targets(
        &strict_program,
        infeasible_profiles,
        vec![StrictF32NumericalContract::governed()],
    );
    let governed = crate::frontier::GovernedPhysicalProvider;
    let infeasible = CensusSpecialist { threads: 512 };
    let (infeasible_product, infeasible_census) =
        crate::workcount::observe_physical_planning(|| {
            compile_with_physical_providers(infeasible_request, vec![&governed, &infeasible])
        });
    let infeasible_product =
        infeasible_product.expect("infeasible specialist proposals remain local rejections");
    assert_eq!(infeasible_product.targets.len(), 16);
    assert_eq!(
        infeasible_census,
        crate::workcount::PhysicalPlanningCensus {
            provider_invocations: 544,
            proposals: 96,
            declines: 480,
            proposal_assessments_started: 96,
            schedule_verifications: 96,
            admitted_implementations: 48,
            retained_implementations: 48,
            proposal_rejections: 48,
            frontier_rejections: 528,
            admitted_sort_items: 48,
            rejection_sort_items: 528,
            plan_combinations: 32,
            accepted_plan_combinations: 32,
            rejected_plan_combinations: 0,
            retained_complete_plans: 32,
            complete_plan_sort_items: 32,
        },
        "infeasible proposals must be assessed, verified, rejected, and sorted without retention",
    );
    println!("MEASURE request-wide strict infeasible census: {infeasible_census:?}");

    let flush = SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    };
    let grouped_profiles = (0..16)
        .map(|index| {
            let (subnormals, reassociation) = match index % 4 {
                0 => (SubnormalMode::Preserve, NumericalPermission::Forbidden),
                1 => (flush, NumericalPermission::Forbidden),
                2 => (SubnormalMode::Preserve, NumericalPermission::Permitted),
                _ => (flush, NumericalPermission::Permitted),
            };
            TargetProfile::numerical_realization_for_test(
                &format!("test.frontier-census-grouped-{index}.v1"),
                subnormals,
                reassociation,
            )
        })
        .collect();
    let grouped_program = if perturb.as_deref() == Some("candidate-contract-population") {
        semantic(false)
    } else {
        tensor_add_chain()
    };
    let grouped_request = request_with_targets(
        &grouped_program,
        grouped_profiles,
        vec![
            StrictF32NumericalContract::governed(),
            StrictF32NumericalContract::governed_flush_to_zero(),
            StrictF32NumericalContract::governed_reassociating(),
            StrictF32NumericalContract::governed_flush_and_reassociate(),
        ],
    );
    let (grouped_product, grouped) =
        crate::workcount::observe_physical_planning(|| compile(grouped_request));
    let grouped_product = grouped_product.expect("all four contract groups compile");
    assert_eq!(grouped_product.targets.len(), 16);
    assert_eq!(
        grouped,
        crate::workcount::PhysicalPlanningCensus {
            provider_invocations: 248,
            proposals: 24,
            declines: 224,
            proposal_assessments_started: 24,
            schedule_verifications: 24,
            admitted_implementations: 24,
            retained_implementations: 24,
            proposal_rejections: 0,
            frontier_rejections: 224,
            admitted_sort_items: 24,
            rejection_sort_items: 224,
            plan_combinations: 24,
            accepted_plan_combinations: 24,
            rejected_plan_combinations: 0,
            retained_complete_plans: 24,
            complete_plan_sort_items: 24,
        },
        "the four-contract semantic-candidate population changed",
    );
    println!("MEASURE request-wide grouped governed census: {grouped:?}");
}

// The compiling contrast for `fail/frontier_enumeration_is_not_reachable.rs`
// and for `fail/verified_request_is_not_reachable.rs`.
//
// A diagnostic alone says what the compiler rejects and not what it accepts,
// and the finding here is an *asymmetry*: `mod frontier` is private, and every
// item a provider implementation must name is nonetheless reachable, through
// `tiler_compiler::physical_provider`. Without this file the two failing cases
// would be consistent with a vocabulary that is simply unavailable, which is
// the finding this spike recorded until 2026-08-08 and which no longer holds.
//
// Every import below is load-bearing: the trait, both method signatures, the
// four types a `propose` body constructs, the decline channel and its cause,
// the installation type, and the one readable cost-model key. A shorter list
// would leave the claim "the vocabulary is reachable" true of a subset.

use tiler_compiler::physical_provider::{
    BaselineImplementation, DeclinedStrategy, FrontierRegionSubject,
    GOVERNED_PHYSICAL_COST_MODEL_KEY, ImplementationContext, ImplementationProposal,
    InstalledPhysicalProviders, PhysicalCostEstimate, PhysicalImplementationProvider,
    PhysicalProviderInstallationError, PhysicalProviderProvenance, PhysicalProviderProvenanceError,
    ProviderOffer, StrategyDeclineCause, TargetApplicability,
};
use tiler_ir::semantic::ProviderIdentity;

struct Local(ProviderIdentity);

impl PhysicalImplementationProvider for Local {
    fn provenance(&self) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
        PhysicalProviderProvenance::new(self.0.clone())
    }

    fn propose(&self, context: &ImplementationContext<'_>) -> ProviderOffer {
        let subject: &FrontierRegionSubject = context.subject();
        let Some(baseline): Option<&BaselineImplementation> = context.baseline() else {
            return ProviderOffer::default().decline(DeclinedStrategy::new(
                "local.identity",
                StrategyDeclineCause::NoAdmissibleShape {
                    rule: "local.no-baseline",
                    extent: u64::try_from(subject.covered_occurrences()).unwrap_or(u64::MAX),
                },
            ));
        };
        let cost: PhysicalCostEstimate = baseline.cost();
        assert_eq!(cost.model_key(), GOVERNED_PHYSICAL_COST_MODEL_KEY);
        ProviderOffer::proposing(vec![ImplementationProposal::scheduled_kernel(
            baseline.region().clone(),
            TargetApplicability::for_targets([context.target_profile().profile_key().clone()]),
            cost,
        )])
    }
}

fn main() {
    let local = Local(ProviderIdentity::new("local", "identity-provider", 1).unwrap());
    let installed =
        InstalledPhysicalProviders::installed([&local as &dyn PhysicalImplementationProvider])
            .expect("one identity installs");
    assert_eq!(installed.identities().len(), 1);

    // The refusal type is nameable too, which is what lets a caller tell an
    // installation it got wrong from a compilation that refused its program.
    let twice = InstalledPhysicalProviders::installed([
        &local as &dyn PhysicalImplementationProvider,
        &local,
    ]);
    assert!(matches!(
        twice,
        Err(PhysicalProviderInstallationError::DuplicateIdentity { .. })
    ));
}

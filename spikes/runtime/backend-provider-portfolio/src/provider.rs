//! A separately authored Metal physical-implementation provider.
//!
//! This is the portfolio's stand-in for a third party that contributes **one**
//! specialized Metal implementation of a region Tiler already implements,
//! without forking `tiler-compiler` and without replacing `tiler-metal`. It
//! implements the whole of [`PhysicalImplementationProvider`] against the
//! public surface only.
//!
//! The specialization is the workgroup width: [`SPECIALIZED_THREADS_PER_WORKGROUP`]
//! instead of the governed provider's one. That axis is free under the
//! intrinsic verifier and is folded into the canonical scheduled-region
//! identity, so the two implementations of one region are distinct alternatives.

use tiler_compiler::physical_provider::{
    DeclinedStrategy, ImplementationContext, ImplementationProposal,
    PhysicalImplementationProvider, PhysicalProviderProvenance, PhysicalProviderProvenanceError,
    ProviderOffer, StrategyDeclineCause, TargetApplicability,
};
use tiler_ir::schedule::ScheduledRegion;
use tiler_ir::semantic::ProviderIdentity;

/// Namespace of this separately authored provider.
///
/// Deliberately not `tiler`: provider provenance is a versioned identity
/// separated from semantic meaning (ADR 0072). Installation refuses the
/// governed namespace outright.
pub const NAMESPACE: &str = "acme";

/// Name of this separately authored provider.
pub const NAME: &str = "simdgroup-pointwise-metal";

/// Output-affecting revision of this provider.
pub const REVISION: u32 = 4;

/// The workgroup width this provider specializes on.
///
/// One Apple SIMD group. The governed provider emits one thread per workgroup.
pub const SPECIALIZED_THREADS_PER_WORKGROUP: u32 = 32;

/// The stable name this provider declines a withheld strategy under.
pub const WIDE_WORKGROUP_STRATEGY: &str = "acme.wide-workgroup";

/// The decline rule this provider names when a subject has no single-dispatch baseline.
pub const NO_BASELINE_RULE: &str = "acme.no-single-dispatch-baseline";

/// Returns this provider's identity.
///
/// # Panics
///
/// Panics only if the compile-time components violate the canonical
/// provider-identity grammar, which no reachable input can cause.
#[must_use]
pub fn identity() -> ProviderIdentity {
    ProviderIdentity::new(NAMESPACE, NAME, REVISION)
        .expect("the spike provider identity is well formed")
}

/// A separately authored provider that specializes one schedule axis.
pub struct AcmeProvider {
    identity: ProviderIdentity,
}

impl AcmeProvider {
    /// Builds a provider under this crate's own identity.
    #[must_use]
    pub fn new() -> Self {
        Self {
            identity: identity(),
        }
    }

    fn specialize(baseline: &ScheduledRegion) -> ScheduledRegion {
        let mut region = baseline.clone();
        region.schedule.threads_per_workgroup = SPECIALIZED_THREADS_PER_WORKGROUP;
        region.schedule.launch.threads_per_workgroup = SPECIALIZED_THREADS_PER_WORKGROUP;
        region
    }
}

impl Default for AcmeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicalImplementationProvider for AcmeProvider {
    fn provenance(&self) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
        PhysicalProviderProvenance::new(self.identity.clone())
    }

    fn propose(&self, context: &ImplementationContext<'_>) -> ProviderOffer {
        let Some(baseline) = context.baseline() else {
            return ProviderOffer::default().decline(DeclinedStrategy::new(
                WIDE_WORKGROUP_STRATEGY,
                StrategyDeclineCause::NoAdmissibleShape {
                    rule: NO_BASELINE_RULE,
                    extent: u64::try_from(context.subject().covered_occurrences())
                        .unwrap_or(u64::MAX),
                },
            ));
        };
        ProviderOffer::proposing(vec![ImplementationProposal::scheduled_kernel(
            Self::specialize(baseline.region()),
            TargetApplicability::for_targets([context.target_profile().profile_key().clone()]),
            baseline.cost(),
        )])
    }
}

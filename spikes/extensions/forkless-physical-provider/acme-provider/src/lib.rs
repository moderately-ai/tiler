//! A separately authored physical-implementation provider for Tiler's Metal
//! vertical, written against the public surface only.
//!
//! This crate is the spike's stand-in for a third party that wants to
//! contribute **one** specialized Metal implementation of a region Tiler
//! already implements, without forking `tiler-compiler` and without replacing
//! `tiler-metal`. It depends on `tiler-compiler` and `tiler-ir` exactly as an
//! out-of-tree crate would: by path, with no feature flag, no `#[path]`
//! include, and no access to any private module. Its workspace is not the
//! repository's, so the dependency graph it compiles against is resolved by its
//! own lockfile rather than by the one that builds `tiler-compiler`'s tests.
//!
//! # What it implements
//!
//! The whole of [`PhysicalImplementationProvider`], reached through
//! `tiler_compiler::physical_provider`. The specialization is the workgroup
//! width: [`SPECIALIZED_THREADS_PER_WORKGROUP`] threads per workgroup instead
//! of the governed provider's one. That axis is free under the intrinsic
//! verifier, which requires only that `schedule.threads_per_workgroup` equal
//! `schedule.launch.threads_per_workgroup` and be non-zero, and it is folded
//! into the canonical scheduled-region identity, so the two implementations of
//! one region are distinct alternatives rather than one implementation twice.
//!
//! # Where the body comes from, and why that is not a detail
//!
//! It is [`ImplementationContext::baseline`] — this host's own spelling of the
//! region subject — cloned and perturbed in exactly one field. Nothing here
//! builds a scheduled region from scratch any more, and the change is the whole
//! reason this crate is now short. The request-subject binding compares a
//! proposed region's identity, iteration shape, scalar program, semantic
//! members, and access map against the compiler's own normalization, so a
//! hand-built body has to reproduce all five; the two earlier revisions of this
//! crate did, and both stopped compiling when `tiler-ir` moved underneath them.
//! Specializing the host's spelling is the operation the landed seam supports,
//! and it is also the one a third party can keep working.
//!
//! # What it deliberately does not state
//!
//! Provider identity on a proposal, exact resource requirements, the boundary
//! contract, hard feasibility, and cost-model attribution. There is no public
//! spelling for any of them: the host derives all five. The probe's compile-fail
//! fixtures are where that absence is recorded as evidence rather than as a
//! sentence.

use std::cell::RefCell;

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
/// separated from semantic meaning (ADR 0072), so a third party's proposals
/// must be attributable to the third party. Installation refuses the governed
/// namespace outright, which the probe exercises.
pub const NAMESPACE: &str = "acme";

/// Name of this separately authored provider.
pub const NAME: &str = "simdgroup-pointwise-metal";

/// Output-affecting revision of this provider.
///
/// Output-affecting in the literal sense the identity contract means: bumping
/// it must accompany a change to the bytes this crate proposes. It moved from
/// `3` to `4` when the body stopped being hand-built and became the host's
/// baseline specialized in one field, which changes those bytes. It has never
/// been `1`, so a reader cannot mistake it for the governed provider's revision
/// by coincidence.
pub const REVISION: u32 = 4;

/// The workgroup width this provider specializes on.
///
/// One Apple SIMD group. The governed provider emits one thread per workgroup
/// unconditionally, which is the launch geometry a Metal backend would most
/// obviously want to improve on, and it is the narrowest specialization that is
/// a genuine physical difference rather than a re-spelling.
pub const SPECIALIZED_THREADS_PER_WORKGROUP: u32 = 32;

/// The stable name this provider declines a withheld strategy under.
pub const WIDE_WORKGROUP_STRATEGY: &str = "acme.wide-workgroup";

/// The decline rule this provider names when a subject has no single-dispatch
/// baseline to specialize.
pub const NO_BASELINE_RULE: &str = "acme.no-single-dispatch-baseline";

/// Returns this provider's identity.
///
/// # Panics
///
/// Panics only if the compile-time components above violate the canonical
/// provider-identity grammar, which no reachable input can cause.
#[must_use]
pub fn identity() -> ProviderIdentity {
    ProviderIdentity::new(NAMESPACE, NAME, REVISION)
        .expect("the spike provider identity is well formed")
}

/// How this provider perturbs the host's baseline region.
///
/// Two of the three are deliberately invalid. A provider is *trusted* native
/// code, so the interesting question is not whether a correct body is accepted
/// but whether an incorrect one is believed, and that has to be asked with a
/// body the host must refuse.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Specialization {
    /// A wider workgroup, which the intrinsic verifier leaves free.
    WideWorkgroup,
    /// A zero-thread workgroup: structurally invalid IR, not an expensive plan.
    ZeroThreadWorkgroup,
    /// A grid one thread short of the iteration domain, which is the launch
    /// coverage rule rather than a target limit.
    UndercoveredGrid,
}

/// A separately authored provider that specializes one schedule axis.
///
/// It retains the baselines it read and the bodies it proposed, in the order the
/// enumeration asked for them, so the probe can state claims about what actually
/// reached the frontier rather than about what this crate intended to send.
pub struct AcmeProvider {
    identity: ProviderIdentity,
    specialization: Specialization,
    exchanged: RefCell<Vec<Exchange>>,
}

/// One baseline this provider was offered and the body it proposed for it.
#[derive(Clone, Debug)]
pub struct Exchange {
    /// The host's own spelling of the region subject.
    pub baseline: ScheduledRegion,
    /// The specialized body this provider proposed for it.
    pub proposed: ScheduledRegion,
}

impl AcmeProvider {
    /// Builds a provider under this crate's own identity.
    #[must_use]
    pub fn new(specialization: Specialization) -> Self {
        Self::named(NAME, REVISION, specialization)
    }

    /// Builds a provider under a stated name and revision.
    ///
    /// Present so the probe can install two providers, and two revisions of one
    /// provider, without a second implementation of `propose` that could drift
    /// from this one.
    ///
    /// # Panics
    ///
    /// Panics if `name` and `revision` do not form a canonical provider
    /// identity, which is a defect in the caller rather than a reachable input.
    #[must_use]
    pub fn named(name: &str, revision: u32, specialization: Specialization) -> Self {
        Self {
            identity: ProviderIdentity::new(NAMESPACE, name, revision)
                .expect("the spike provider identity is well formed"),
            specialization,
            exchanged: RefCell::new(Vec::new()),
        }
    }

    /// Builds a provider claiming an arbitrary namespace, name, and revision.
    ///
    /// The one constructor that can spell Tiler's own governed identity, which
    /// is what the probe's forged-identity claim needs and what installation
    /// refuses.
    ///
    /// # Panics
    ///
    /// Panics if the three components do not form a canonical provider identity.
    #[must_use]
    pub fn impersonating(namespace: &str, name: &str, revision: u32) -> Self {
        Self {
            identity: ProviderIdentity::new(namespace, name, revision)
                .expect("the impersonated identity is well formed"),
            specialization: Specialization::WideWorkgroup,
            exchanged: RefCell::new(Vec::new()),
        }
    }

    /// Returns the baselines read and the bodies proposed, in enumeration order.
    #[must_use]
    pub fn exchanged(&self) -> Vec<Exchange> {
        self.exchanged.borrow().clone()
    }

    fn specialize(&self, baseline: &ScheduledRegion) -> ScheduledRegion {
        let mut region = baseline.clone();
        match self.specialization {
            Specialization::WideWorkgroup => {
                region.schedule.threads_per_workgroup = SPECIALIZED_THREADS_PER_WORKGROUP;
                region.schedule.launch.threads_per_workgroup = SPECIALIZED_THREADS_PER_WORKGROUP;
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
            // Silence and a named decline are different answers, and this is
            // the second: the strategy applied and this subject admitted no
            // shape for it. A subject whose cover made it a published-and-
            // consumed region has no single-dispatch baseline, and there is no
            // public spelling of a two-dispatch body to propose instead.
            return ProviderOffer::default().decline(DeclinedStrategy::new(
                WIDE_WORKGROUP_STRATEGY,
                StrategyDeclineCause::NoAdmissibleShape {
                    rule: NO_BASELINE_RULE,
                    // `try_from` rather than `as`: the count is a `usize` and
                    // the cause carries a `u64`, and a saturating conversion
                    // states what an unrepresentable count would mean instead
                    // of wrapping one silently.
                    extent: u64::try_from(context.subject().covered_occurrences())
                        .unwrap_or(u64::MAX),
                },
            ));
        };
        let proposed = self.specialize(baseline.region());
        self.exchanged.borrow_mut().push(Exchange {
            baseline: baseline.region().clone(),
            proposed: proposed.clone(),
        });
        ProviderOffer::proposing(vec![ImplementationProposal::scheduled_kernel(
            proposed,
            // The key comes from the request rather than from a constant of
            // this crate's own, so a profile rename fails to compile here
            // instead of making every proposal silently inapplicable.
            TargetApplicability::for_targets([context.target_profile().profile_key().clone()]),
            // The host's own estimate for the region this specializes. A wider
            // workgroup changes no structural dimension, so inventing a lower
            // number would win a comparison that measured nothing.
            baseline.cost(),
        )])
    }
}

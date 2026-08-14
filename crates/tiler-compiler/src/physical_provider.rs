//! The installable physical-implementation provider seam.
//!
//! [ADR 0090] item 2's public boundary: a separately authored crate contributes
//! one specialized *implementation* of a region this compiler already
//! understands, without forking `tiler-compiler` and without replacing any
//! emitter. Its candidates are re-verified by the host and considered additively
//! beside the governed provider's, and the plan that selects one records exactly
//! which provider proposed it.
//!
//! # What a provider is, and what it is not
//!
//! A provider claims **one** row of the composed-backend responsibility matrix:
//! proposing physical implementations. It is not a backend, a bundle, or a
//! registration of anything else. There is no emitter registration here, no
//! runtime-adapter registration, no target-profile registration, and no
//! lowering-capability registration — the last of those is a different seam with
//! a different rule, reached through [`crate::session::InstalledCapabilities`].
//!
//! That difference is load-bearing and neither rule generalizes to the other.
//! Two lowering authorities claiming one occurrence are a *contradiction*: only
//! one statement about what an occurrence means can be true. Two physical
//! implementations of one verified region are *alternatives*: both compute the
//! region, and a compiler that discarded one would be discarding a legal plan.
//! So installing a second lowering provider for one occurrence is an ambiguity
//! refusal, while installing a second physical provider is the ordinary case.
//!
//! # Trust: proposed, then re-verified
//!
//! A provider is trusted native code, statically linked, supplied explicitly to
//! one compilation — the linkage model [ADR 0045] fixes, extended by nothing.
//! Trust is not belief. Every body reaching [`ImplementationProposal::scheduled_kernel`]
//! is resubmitted through this host's own whole-region intrinsic verification,
//! the request-subject binding, and the single hard-feasibility decision before
//! it can be admitted, and five things a provider might otherwise assert are
//! derived by the host instead:
//!
//! | Subject | Stated by | Why |
//! | --- | --- | --- |
//! | Provider identity | the host, from registration | a proposal cannot claim another provider's name |
//! | Exact resource requirements | the host, from the verified region | a declared requirement could understate what the body needs |
//! | Boundary contract | the host, from the verified region | ownership and visibility are proofs, not claims |
//! | Hard feasibility | the host's decision procedure | a second implementation of it would drift |
//! | Cost-model attribution | the host's one governed key | estimates that are not comparable must not be ranked |
//!
//! The provider states a body, an applicability predicate, a cost estimate, and
//! the strategies it considered and withheld. Nothing else.
//!
//! # Five refusals a caller can tell apart
//!
//! An empty [`ProviderOffer`] is a legitimate local result and is *not* an
//! error: it means this provider recognizes nothing about this region and
//! target. It stays distinguishable from the four other outcomes — a hard
//! target rejection naming the disproved capability predicate, an analysis that
//! could not be completed, malformed provider output that fails the whole
//! enumeration closed, and a proposal that was admitted and lost on cost. A
//! provider that offers nothing but *declines a named strategy*
//! ([`ProviderOffer::decline`]) says something stronger than silence, which is
//! why the decline channel exists at all.
//!
//! # Accepted boundary
//!
//! Tom accepted this module's exact included and excluded public surface on
//! 2026-08-11 under [ADR 0075]. The excluded set is deliberate and is stated in
//! [`ImplementationProposal::scheduled_kernel`],
//! [`ImplementationContext::baseline`], and
//! [`FrontierRegionSubject::covered_occurrences`] rather than left to be
//! inferred from what is missing.
//!
//! # Proving the boundary refuses
//!
//! Each case below is compile-fail evidence carrying its exact error code, which
//! is what makes it evidence: a bare `compile_fail` passes on *any* failure,
//! including a typo, so it would record a boundary that had quietly moved as
//! coverage. Together they pin the four bypasses a provider might reach for.
//!
//! **The verified request is not reachable, so a provider cannot re-derive the
//! host's normalization and disagree with it.**
//!
//! ```compile_fail,E0624
//! use tiler_compiler::physical_provider::ImplementationContext;
//!
//! fn read(context: &ImplementationContext<'_>) {
//!     let _ = context.request();
//! }
//! ```
//!
//! **A cost estimate cannot name a model of the provider's own**, which is what
//! stops two incomparable estimates being ranked against each other.
//!
//! ```compile_fail,E0624
//! use tiler_compiler::physical_provider::PhysicalCostEstimate;
//!
//! let _ = PhysicalCostEstimate::new("acme.cost.my-own.v1", 1, 4, 0);
//! ```
//!
//! **A region subject's members are not readable**, so a graph-local authoring
//! coordinate cannot reach a provider's decision or, through a decline cause,
//! the trace.
//!
//! ```compile_fail,E0624
//! use tiler_compiler::physical_provider::FrontierRegionSubject;
//!
//! fn read(subject: &FrontierRegionSubject) {
//!     let _ = subject.semantic_members();
//! }
//! ```
//!
//! **The enumeration itself is not reachable**, so nothing bypasses the compile
//! path to admit a body outside a compilation's own authorities.
//!
//! ```compile_fail,E0603
//! use tiler_compiler::frontier::enumerate_frontier;
//! ```
//!
//! [ADR 0045]: https://github.com/moderately-ai/tiler/blob/main/docs/decisions/0045-bound-proc-macro-providers-to-host-dependencies.md
//! [ADR 0075]: https://github.com/moderately-ai/tiler/blob/main/docs/decisions/0075-scope-public-boundary-approval-by-change-category.md
//! [ADR 0090]: https://github.com/moderately-ai/tiler/blob/main/docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md

use std::error::Error;
use std::fmt;

use tiler_ir::semantic::ProviderIdentity;

pub use crate::frontier::{
    BaselineImplementation, DeclinedStrategy, FrontierRegionSubject, ImplementationContext,
    ImplementationProposal, PhysicalCostEstimate, PhysicalImplementationProvider,
    PhysicalProviderProvenance, PhysicalProviderProvenanceError, ProviderOffer,
    StrategyDeclineCause, TargetApplicability,
};

use crate::frontier::{COST_MODEL_KEY, GovernedPhysicalProvider};

/// The one cost model a physical implementation proposal may be attributed to.
///
/// Published because it is the only admissible attribution and a provider
/// comparing its own estimate against a plan's needs to name it. It is not a
/// choice: [`PhysicalCostEstimate::structural`] is the only public constructor
/// and already attributes to this key, so the constant is a *reading* surface.
/// Relaxing the attribution would let two providers' incomparable estimates be
/// ranked against each other, which is a silently wrong selection rather than a
/// refusal.
pub const GOVERNED_PHYSICAL_COST_MODEL_KEY: &str = COST_MODEL_KEY;

/// The frozen physical-implementation provider environment of one compilation.
///
/// Frozen, per-session, and immutable, in the idiom [ADR 0078] item 4 already
/// closed for lowering. There is no global registry, no link-time collector, no
/// environment-variable search path, and no discovery: a compilation's identity
/// must be a deterministic function of its request, and a registry a linked-in
/// crate could mutate would make two identical requests produce two identities
/// for a reason nothing records.
///
/// # Installation order is not precedence
///
/// The order is retained for reporting and decides nothing. The frontier returns
/// its admitted set in canonical, provider-order-independent order, so there is
/// nothing for an ordering to order. A caller relying on order to pick a winner
/// is relying on an accident this type does not have.
///
/// # The governed provider is always present
///
/// Installing providers **adds** to Tiler's own governed provider; it does not
/// replace it. A partial provider is the normal case — the measured out-of-tree
/// provider wanted one of thirteen responsibilities and reused twelve — so a
/// registry that replaced the governed one would leave every region the
/// installed set does not recognize with no implementation at all, and turn a
/// specialization into a whole-compiler reimplementation. Removing the governed
/// provider has no spelling here, deliberately.
///
/// [ADR 0078]: https://github.com/moderately-ai/tiler/blob/main/docs/decisions/0078-name-the-intended-public-extension-seams.md
#[derive(Clone)]
pub struct InstalledPhysicalProviders<'providers> {
    installed: Vec<&'providers dyn PhysicalImplementationProvider>,
    /// The installed identities in installation order, resolved once at
    /// installation so a later read cannot observe a different answer than the
    /// duplicate check did.
    identities: Vec<ProviderIdentity>,
}

impl fmt::Debug for InstalledPhysicalProviders<'_> {
    /// Renders the installed identities, which is the whole of what this type
    /// says about itself; a provider is a borrowed implementation with no
    /// printable state of its own.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InstalledPhysicalProviders")
            .field("installed", &self.identities)
            .finish()
    }
}

impl<'providers> InstalledPhysicalProviders<'providers> {
    /// The physical authorities this build ships, with nothing installed.
    ///
    /// The default of every [`crate::session::CompileRequest`], which is what
    /// makes a compilation that never mentions this module compile exactly as it
    /// did before this seam existed.
    #[must_use]
    pub const fn governed() -> Self {
        Self {
            installed: Vec::new(),
            identities: Vec::new(),
        }
    }

    /// Installs a caller's providers beside the governed one.
    ///
    /// # Errors
    ///
    /// Returns [`PhysicalProviderInstallationError`] when the stated set cannot
    /// be a frozen environment: a provider whose identity cannot be rendered
    /// into the bounded explain subject every outcome retains, an identity
    /// installed twice, or an identity that collides with Tiler's own governed
    /// provider. Each is a refusal rather than a replacement — a silent
    /// last-wins registration would let a plan be selected under one authority
    /// and reported under another, and a collision with the governed identity
    /// would make the compiler's own provenance forgeable.
    pub fn installed(
        providers: impl IntoIterator<Item = &'providers dyn PhysicalImplementationProvider>,
    ) -> Result<Self, PhysicalProviderInstallationError> {
        let governed = GovernedPhysicalProvider::identity();
        let installed: Vec<&'providers dyn PhysicalImplementationProvider> =
            providers.into_iter().collect();
        let mut identities: Vec<ProviderIdentity> = Vec::with_capacity(installed.len());
        for provider in &installed {
            let identity =
                provider
                    .provenance()
                    .map_err(|source| {
                        PhysicalProviderInstallationError::UnrepresentableProvenance { source }
                    })?
                    .provider()
                    .clone();
            if identity == governed {
                return Err(PhysicalProviderInstallationError::GovernedIdentity { identity });
            }
            if identities.contains(&identity) {
                return Err(PhysicalProviderInstallationError::DuplicateIdentity { identity });
            }
            identities.push(identity);
        }
        Ok(Self {
            installed,
            identities,
        })
    }

    /// Returns the installed provider identities in installation order.
    ///
    /// The governed provider is deliberately absent: this reports what the
    /// *caller* installed, so an empty answer means "nothing was installed"
    /// rather than "the environment was empty", and a caller can tell its own
    /// registration failing to take effect from its provider losing on cost.
    #[must_use]
    pub fn identities(&self) -> &[ProviderIdentity] {
        &self.identities
    }

    /// The complete provider list one compilation enumerates against.
    ///
    /// Governed first, then the caller's in installation order. First is a
    /// reporting convention rather than a precedence: see the type documentation.
    pub(crate) fn providers(&self) -> Vec<&'providers dyn PhysicalImplementationProvider> {
        let mut providers: Vec<&'providers dyn PhysicalImplementationProvider> =
            Vec::with_capacity(self.installed.len() + 1);
        providers.push(&GovernedPhysicalProvider);
        providers.extend_from_slice(&self.installed);
        providers
    }

    /// The identities of exactly the providers [`Self::providers`] enumerates.
    ///
    /// Positionally the same list, which is what makes
    /// [`crate::session::Compilation::offered_physical_providers`] a *reading*
    /// of the environment the frontier was actually given rather than a second
    /// derivation of it. The installed identities are the ones resolved at
    /// installation, so a provider whose `provenance` answers differently on a
    /// later call cannot make the compilation report a name it was not
    /// installed under.
    ///
    /// **A different subject from [`Self::identities`], and the difference is
    /// the whole point of the offered half.** That accessor answers what the
    /// *caller* installed, so its empty answer means "I installed nothing";
    /// this one answers what the *compilation* was offered, and is never empty
    /// because the governed provider is always asked. Reading the first as the
    /// second would make a compilation look as though it enumerated no physical
    /// authority at all.
    pub(crate) fn offered_identities(&self) -> Vec<ProviderIdentity> {
        let mut offered: Vec<ProviderIdentity> = Vec::with_capacity(self.identities.len() + 1);
        offered.push(GovernedPhysicalProvider::identity());
        offered.extend(self.identities.iter().cloned());
        offered
    }
}

impl Default for InstalledPhysicalProviders<'_> {
    fn default() -> Self {
        Self::governed()
    }
}

/// Why a stated physical-provider set could not become a frozen environment.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PhysicalProviderInstallationError {
    /// A provider's identity has no bounded explain-subject rendering.
    UnrepresentableProvenance {
        /// The refusal the provenance itself raised.
        source: PhysicalProviderProvenanceError,
    },
    /// One identity was installed more than once.
    ///
    /// Two revisions of one provider are two identities and both may install;
    /// this is the same identity twice, which no ordering could resolve.
    DuplicateIdentity {
        /// The identity installed twice.
        identity: ProviderIdentity,
    },
    /// An installed provider claimed Tiler's own governed provider identity.
    GovernedIdentity {
        /// The governed identity the installation tried to claim.
        identity: ProviderIdentity,
    },
}

impl fmt::Display for PhysicalProviderInstallationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnrepresentableProvenance { source } => write!(
                formatter,
                "physical-provider.installation.unrepresentable-provenance: {source}"
            ),
            Self::DuplicateIdentity { identity } => write!(
                formatter,
                "physical-provider.installation.duplicate-identity: {identity} is installed twice"
            ),
            Self::GovernedIdentity { identity } => write!(
                formatter,
                "physical-provider.installation.governed-identity: {identity} is Tiler's own governed physical provider"
            ),
        }
    }
}

impl Error for PhysicalProviderInstallationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnrepresentableProvenance { source } => Some(source),
            Self::DuplicateIdentity { .. } | Self::GovernedIdentity { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        InstalledPhysicalProviders, PhysicalImplementationProvider,
        PhysicalProviderInstallationError, PhysicalProviderProvenance,
        PhysicalProviderProvenanceError, ProviderIdentity, ProviderOffer,
    };
    use crate::frontier::{GovernedPhysicalProvider, ImplementationContext};

    /// A provider that answers with one stated identity and proposes nothing.
    struct NamedProvider(ProviderIdentity);

    impl PhysicalImplementationProvider for NamedProvider {
        fn provenance(
            &self,
        ) -> Result<PhysicalProviderProvenance, PhysicalProviderProvenanceError> {
            PhysicalProviderProvenance::new(self.0.clone())
        }

        fn propose(&self, _: &ImplementationContext<'_>) -> ProviderOffer {
            ProviderOffer::default()
        }
    }

    fn named(name: &str, revision: u32) -> NamedProvider {
        NamedProvider(
            ProviderIdentity::new("acme", name, revision).expect("the test identity is valid"),
        )
    }

    /// **The governed provider is present whether or not anything is installed,
    /// and installing never removes it.**
    ///
    /// The population is counted rather than asserted by shape, so a future
    /// governed provider pair cannot make this silently assert less.
    #[test]
    fn installed_providers_are_additive_beside_the_governed_one() {
        assert_eq!(InstalledPhysicalProviders::governed().providers().len(), 1);
        assert!(
            InstalledPhysicalProviders::governed()
                .identities()
                .is_empty()
        );

        let first = named("first", 1);
        let second = named("second", 1);
        let installed = InstalledPhysicalProviders::installed([
            &first as &dyn PhysicalImplementationProvider,
            &second,
        ])
        .expect("two distinct identities install");
        assert_eq!(installed.providers().len(), 3);
        assert_eq!(installed.identities().len(), 2);
    }

    /// **The offered identities are exactly the enumerated providers' own, in
    /// the same positions.**
    ///
    /// The two lists are built by separate code, so nothing but this test stops
    /// them drifting — and a drift would be silent in the worst direction: a
    /// compilation would report an environment the frontier did not enumerate,
    /// which is the conflation the offered half exists to remove. Each identity
    /// is read back off the provider it is claimed to name rather than compared
    /// against a restated literal, so a rename of Tiler's own provider cannot
    /// leave this guarding a name nothing produces.
    #[test]
    fn the_offered_identities_name_exactly_the_enumerated_providers() {
        let first = named("first", 1);
        let second = named("second", 1);
        for environment in [
            InstalledPhysicalProviders::governed(),
            InstalledPhysicalProviders::installed([
                &first as &dyn PhysicalImplementationProvider,
                &second,
            ])
            .expect("two distinct identities install"),
        ] {
            let enumerated: Vec<ProviderIdentity> = environment
                .providers()
                .iter()
                .map(|provider| {
                    provider
                        .provenance()
                        .expect("every enumerated provider renders its provenance")
                        .provider()
                        .clone()
                })
                .collect();
            assert_eq!(environment.offered_identities(), enumerated);
        }
    }

    /// **One identity installed twice is a refusal, never a replacement.**
    ///
    /// Two *revisions* of one provider are two identities and both install, so
    /// the check is on the whole identity rather than on namespace and name —
    /// the second assertion is what stops it degrading into the weaker one.
    #[test]
    fn one_identity_cannot_be_installed_twice() {
        let provider = named("repeated", 1);
        let error = InstalledPhysicalProviders::installed([
            &provider as &dyn PhysicalImplementationProvider,
            &provider,
        ])
        .expect_err("the same identity installed twice is refused");
        assert!(matches!(
            error,
            PhysicalProviderInstallationError::DuplicateIdentity { .. }
        ));

        let revised = named("repeated", 2);
        InstalledPhysicalProviders::installed([
            &provider as &dyn PhysicalImplementationProvider,
            &revised,
        ])
        .expect("two revisions of one provider are two identities");
    }

    /// **An installed provider may not claim the governed identity.**
    ///
    /// The identity is read from the governed provider rather than restated as a
    /// literal, so a rename of Tiler's own provider cannot leave this test
    /// guarding a name nothing uses.
    #[test]
    fn the_governed_identity_cannot_be_claimed_by_an_installed_provider() {
        let impostor = NamedProvider(GovernedPhysicalProvider::identity());
        let error = InstalledPhysicalProviders::installed([
            &impostor as &dyn PhysicalImplementationProvider
        ])
        .expect_err("claiming the governed identity is refused");
        match error {
            PhysicalProviderInstallationError::GovernedIdentity { identity } => {
                assert_eq!(identity, GovernedPhysicalProvider::identity());
            }
            other => panic!("the governed-identity collision was reported as {other:?}"),
        }
    }
}

//! The read-side vocabulary one refusal reports a declared target fact through.
//!
//! These types borrow the retained fact the feasibility authority refused on,
//! so a diagnostic cannot be handed a provenance record it could edit. They are
//! deliberately wider than the declaration vocabulary in [`super::source`]: a
//! refusal must be able to name every authority the compiler itself attributes,
//! including the two no external declarer may claim.

use tiler_ir::program::abi::AvailabilityPhase;

use crate::target::feasibility::{FactAuthority, FactValidityScope};
use crate::target::honourability::{
    CompilerBuildIdentity, CompilerBuildRole, ExecutionEnvironmentIdentity, FactEvidenceBasis,
    MeasurementContext, NumericalRefusalEvidence, ProvenanceIdentity,
};
use crate::target::key::TargetProfileKey;

// ---------------------------------------------------------------------------
// Reading a declared fact back.
//
// `super::source`'s types declare target facts; the ones here read one back out
// of a refusal. They are separate vocabularies on purpose. A declaration constructor
// validates caller-supplied text and takes ownership, and is deliberately narrow
// — `MeasuredFactAuthority` omits the governed and external authorities because
// no caller may claim them. A refusal view must be able to report *any*
// authority the compiler itself can attribute, including those two, and it
// borrows from the retained fact rather than copying it, so a diagnostic cannot
// be handed a provenance record it could edit and hand back.
// ---------------------------------------------------------------------------

/// The class of authority vouching for one declared target fact.
///
/// The complete read-side space, unlike
/// [`MeasuredFactAuthority`](crate::target::MeasuredFactAuthority), which is the
/// narrower set an external declarer may claim.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum TargetFactAuthority {
    /// A governed, conservative compile-time profile guarantee.
    GovernedProfile,
    /// A named external producer's normative target-family declaration.
    ExternalProfile,
    /// An empirical compile-profile measurement.
    MeasuredProfile,
    /// Evidence attributed to a produced artifact.
    ArtifactEvidence,
    /// Evidence observed from a live device runtime.
    DeviceRuntime,
    /// Evidence observed after a kernel or pipeline was prepared.
    PreparedKernel,
    /// Evidence observed for one concrete launch.
    LaunchInstance,
}

impl TargetFactAuthority {
    /// Exhaustive so a widened internal authority is a build error here rather
    /// than an authority a public refusal cannot name.
    const fn from_internal(authority: FactAuthority) -> Self {
        match authority {
            FactAuthority::GovernedProfile => Self::GovernedProfile,
            FactAuthority::ExternalProfile => Self::ExternalProfile,
            FactAuthority::MeasuredProfile => Self::MeasuredProfile,
            FactAuthority::ArtifactEvidence => Self::ArtifactEvidence,
            FactAuthority::DeviceRuntime => Self::DeviceRuntime,
            FactAuthority::PreparedKernel => Self::PreparedKernel,
            FactAuthority::LaunchInstance => Self::LaunchInstance,
        }
    }
}

/// The scope over which one declared target fact remains valid.
///
/// Reported beside the authority because the two are independent claims: a
/// measured fact is true of the population it was measured on, and reading it
/// as a portable guarantee is exactly the mistake this scope prevents.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum TargetFactValidityScope {
    /// Valid for any device matching the portable profile.
    PortableProfile,
    /// Valid only for the exact measured compiler and environment population.
    MeasuredEnvironment,
    /// Valid for one device instance only.
    DeviceInstance,
    /// Valid for one prepared artifact only.
    PreparedArtifact,
    /// Valid for one launch instance only.
    LaunchInstance,
}

impl TargetFactValidityScope {
    /// Exhaustive for the same reason as [`TargetFactAuthority::from_internal`].
    const fn from_internal(validity: FactValidityScope) -> Self {
        match validity {
            FactValidityScope::PortableProfile => Self::PortableProfile,
            FactValidityScope::MeasuredEnvironment => Self::MeasuredEnvironment,
            FactValidityScope::DeviceInstance => Self::DeviceInstance,
            FactValidityScope::PreparedArtifact => Self::PreparedArtifact,
            FactValidityScope::LaunchInstance => Self::LaunchInstance,
        }
    }
}

/// A borrowed versioned identity naming a producer, guarantee, or role.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TargetProvenanceReference<'a> {
    key: &'a str,
    revision: u32,
}

impl<'a> TargetProvenanceReference<'a> {
    fn borrow(identity: &'a ProvenanceIdentity) -> Self {
        Self {
            key: identity.key(),
            revision: identity.revision(),
        }
    }

    /// Returns the canonical identity key.
    #[must_use]
    pub const fn key(&self) -> &'a str {
        self.key
    }

    /// Returns the nonzero output-affecting revision of that key.
    #[must_use]
    pub const fn revision(&self) -> u32 {
        self.revision
    }
}

/// Semantic role one compiler build performed in a measurement, as read back.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum TargetCompilerRoleReference<'a> {
    /// Source-language frontend.
    Frontend,
    /// Optimizer.
    Optimizer,
    /// Intermediate representation translator.
    IntermediateTranslator,
    /// Machine-code or target-source generator.
    CodeGenerator,
    /// Assembler.
    Assembler,
    /// Linker.
    Linker,
    /// Runtime compiler.
    RuntimeCompiler,
    /// A producer-defined role with its own versioned identity.
    ProducerDefined(TargetProvenanceReference<'a>),
}

impl<'a> TargetCompilerRoleReference<'a> {
    /// Exhaustive so a widened internal role vocabulary is a build error rather
    /// than a role a refusal reports as something it is not.
    fn borrow(role: &'a CompilerBuildRole) -> Self {
        match role {
            CompilerBuildRole::Frontend => Self::Frontend,
            CompilerBuildRole::Optimizer => Self::Optimizer,
            CompilerBuildRole::IntermediateTranslator => Self::IntermediateTranslator,
            CompilerBuildRole::CodeGenerator => Self::CodeGenerator,
            CompilerBuildRole::Assembler => Self::Assembler,
            CompilerBuildRole::Linker => Self::Linker,
            CompilerBuildRole::RuntimeCompiler => Self::RuntimeCompiler,
            CompilerBuildRole::ProviderDefined(identity) => {
                Self::ProducerDefined(TargetProvenanceReference::borrow(identity))
            }
        }
    }
}

/// One exact compiler component a measurement rests on, as read back.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetCompilerBuildReference<'a>(&'a CompilerBuildIdentity);

impl<'a> TargetCompilerBuildReference<'a> {
    /// Returns the semantic pipeline role this build performed.
    #[must_use]
    pub fn role(&self) -> TargetCompilerRoleReference<'a> {
        TargetCompilerRoleReference::borrow(self.0.role())
    }

    /// Returns the implementation key of the build.
    #[must_use]
    pub fn implementation(&self) -> &'a str {
        self.0.implementation()
    }

    /// Returns the build's version text.
    #[must_use]
    pub fn version(&self) -> &'a str {
        self.0.version()
    }

    /// Returns the exact build identity, when the producer stated one.
    #[must_use]
    pub fn build(&self) -> Option<&'a str> {
        self.0.build()
    }
}

/// The exact execution environment of one measurement, as read back.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetExecutionEnvironmentReference<'a>(&'a ExecutionEnvironmentIdentity);

impl<'a> TargetExecutionEnvironmentReference<'a> {
    /// Returns the platform family.
    #[must_use]
    pub fn platform(&self) -> &'a str {
        self.0.platform()
    }

    /// Returns the platform version.
    #[must_use]
    pub fn platform_version(&self) -> &'a str {
        self.0.platform_version()
    }

    /// Returns the exact platform build identity.
    #[must_use]
    pub fn platform_build(&self) -> &'a str {
        self.0.platform_build()
    }

    /// Returns the architecture.
    #[must_use]
    pub fn architecture(&self) -> &'a str {
        self.0.architecture()
    }

    /// Returns the hardware identity.
    #[must_use]
    pub fn hardware(&self) -> &'a str {
        self.0.hardware()
    }
}

/// One measured compiler-build set with the environment it ran in, read back.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetMeasurementContextReference<'a>(&'a MeasurementContext);

impl<'a> TargetMeasurementContextReference<'a> {
    /// Returns the compiler builds participating in this context, in canonical
    /// order. Never empty: a context with no build is refused at construction.
    #[must_use]
    pub fn compiler_builds(&self) -> TargetCompilerBuilds<'a> {
        TargetCompilerBuilds(self.0.compiler_builds())
    }

    /// Returns the environment in which those builds ran.
    #[must_use]
    pub const fn environment(&self) -> TargetExecutionEnvironmentReference<'a> {
        TargetExecutionEnvironmentReference(self.0.environment())
    }
}

/// The compiler builds of one measurement context, in canonical order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetCompilerBuilds<'a>(&'a [CompilerBuildIdentity]);

impl<'a> TargetCompilerBuilds<'a> {
    /// Returns how many builds the context names.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the set is empty. It never is for a checked context.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the build at `index`, or [`None`] past the end.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<TargetCompilerBuildReference<'a>> {
        self.0.get(index).map(TargetCompilerBuildReference)
    }

    /// Iterates the builds in canonical order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = TargetCompilerBuildReference<'a>> {
        self.0.iter().map(TargetCompilerBuildReference)
    }
}

/// The measurement contexts one measured fact rests on, in canonical order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetMeasurementContexts<'a>(&'a [MeasurementContext]);

impl<'a> TargetMeasurementContexts<'a> {
    /// Returns how many contexts the fact rests on.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the set is empty. It never is for a measured fact.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the context at `index`, or [`None`] past the end.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<TargetMeasurementContextReference<'a>> {
        self.0.get(index).map(TargetMeasurementContextReference)
    }

    /// Iterates the contexts in canonical order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = TargetMeasurementContextReference<'a>> {
        self.0.iter().map(TargetMeasurementContextReference)
    }
}

/// Why the authority may make the fact behind one refusal.
///
/// A normative guarantee and an empirical measurement are different claims, and
/// this is where the difference is visible: only the measured arm can name the
/// compiler builds and execution environments a reader would compare against
/// its own deployment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetNumericalEvidenceBasis<'a> {
    /// A normative guarantee governed by this compiler build.
    GovernedGuarantee {
        /// The versioned guarantee cited.
        guarantee: TargetProvenanceReference<'a>,
    },
    /// A normative or specification-backed guarantee from an external producer.
    ExternalGuarantee {
        /// The versioned normative reference cited.
        reference: TargetProvenanceReference<'a>,
    },
    /// One or more exact, independently readable measurement contexts.
    Measurement {
        /// The contexts measured, in canonical order. Never empty.
        contexts: TargetMeasurementContexts<'a>,
    },
}

/// Borrowed, read-only view of the exact checked fact behind one refusal.
///
/// It borrows the fact the feasibility authority refused on rather than a copy
/// assembled at the boundary, which is what makes it evidence: nothing between
/// the declaration and this view can substitute a plausible provenance for the
/// one the declarer supplied, and nothing a caller holds can edit it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetNumericalRefusalEvidence<'a>(&'a NumericalRefusalEvidence);

impl<'a> TargetNumericalRefusalEvidence<'a> {
    pub(crate) const fn borrow(evidence: &'a NumericalRefusalEvidence) -> Self {
        Self(evidence)
    }

    /// Returns the earliest phase from which the declaration is available.
    #[must_use]
    pub fn available_at(&self) -> AvailabilityPhase {
        self.0.phase()
    }

    /// Returns the class of authority vouching for the declaration.
    #[must_use]
    pub fn authority(&self) -> TargetFactAuthority {
        TargetFactAuthority::from_internal(self.0.authority())
    }

    /// Returns the scope over which the declaration remains valid.
    #[must_use]
    pub fn validity(&self) -> TargetFactValidityScope {
        TargetFactValidityScope::from_internal(self.0.validity())
    }

    /// Returns the versioned identity of the authority that made the claim.
    #[must_use]
    pub fn authority_identity(&self) -> TargetProvenanceReference<'a> {
        TargetProvenanceReference::borrow(self.0.source().authority_identity())
    }

    /// Returns why that authority may make the claim.
    #[must_use]
    pub fn basis(&self) -> TargetNumericalEvidenceBasis<'a> {
        match self.0.source().basis() {
            FactEvidenceBasis::GovernedGuarantee { guarantee } => {
                TargetNumericalEvidenceBasis::GovernedGuarantee {
                    guarantee: TargetProvenanceReference::borrow(guarantee),
                }
            }
            FactEvidenceBasis::ExternalGuarantee { reference } => {
                TargetNumericalEvidenceBasis::ExternalGuarantee {
                    reference: TargetProvenanceReference::borrow(reference),
                }
            }
            FactEvidenceBasis::Measurement { contexts } => {
                TargetNumericalEvidenceBasis::Measurement {
                    contexts: TargetMeasurementContexts(contexts),
                }
            }
        }
    }

    /// Returns the profile key that declared the fact.
    #[must_use]
    pub fn target_profile(&self) -> &'a TargetProfileKey {
        self.0.profile().public_key()
    }
}

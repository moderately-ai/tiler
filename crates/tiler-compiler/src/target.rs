//! Public target-profile declaration vocabulary.
//!
//! Request coordination consumes an immutable checked profile; it does not own
//! the vocabulary by which target producers state capability, numerical, or
//! dtype-dispatchability facts.
//!
//! Tom accepted this public boundary at commit `4ad5a2e` on 2026-07-30.
//! It exposes externally attributed normative guarantees and measurements.
//! The compiler-governed source and exact-emulation proof constructors remain
//! private because a caller cannot manufacture either authority.
//!
//! ```
//! use tiler_compiler::target::{
//!     DTypeDispatchability, DeviceAddressWidth, IndexArithmeticSupport,
//!     TargetFactProducerIdentity, TargetFactSource, TargetNormativeReferenceIdentity,
//!     TargetProfileBuilder, TargetProfileKey, TargetRequest,
//! };
//! use tiler_ir::semantic::F32;
//!
//! let producer = TargetFactProducerIdentity::new("acme.gpu-profile.v1".to_owned(), 1)?;
//! let specification =
//!     TargetNormativeReferenceIdentity::new("acme.gpu-specification.v3".to_owned(), 3)?;
//! let source = TargetFactSource::external_guarantee(producer, specification);
//! let mut builder =
//!     TargetProfileBuilder::new(TargetProfileKey::new("acme.gpu.family-a.v1".to_owned())?);
//! builder.declare_max_threads_per_grid_axis(65_535, source.clone())?;
//! builder.declare_max_threads_per_workgroup(256, source.clone())?;
//! builder.declare_max_buffer_bindings_per_entry(31, source.clone())?;
//! builder.declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())?;
//! builder.declare_device_address_width(DeviceAddressWidth::Bits64, source.clone())?;
//! builder.declare_device_memory(true, source.clone())?;
//! builder.declare_local_memory_bytes(32_768, source.clone())?;
//! builder.declare_dtype_dispatchability(
//!     F32::resolved_type(),
//!     DTypeDispatchability::Dispatchable,
//!     source,
//! )?;
//! let profile = builder.build()?;
//! let targets = TargetRequest::new([profile])?;
//! assert_eq!(targets.profiles().len(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! A dimension-specific method cannot be paired with another dimension's
//! behaviour vocabulary:
//!
//! ```compile_fail
//! # use tiler_compiler::target::*;
//! # use tiler_ir::schedule::NumericalPermission;
//! # let producer = TargetFactProducerIdentity::new("acme.profile.v1".to_owned(), 1).unwrap();
//! # let reference = TargetNormativeReferenceIdentity::new("acme.spec.v1".to_owned(), 1).unwrap();
//! # let source = TargetFactSource::external_guarantee(producer, reference);
//! # let mut builder =
//! #     TargetProfileBuilder::new(TargetProfileKey::new("acme.gpu.v1".to_owned()).unwrap());
//! builder.declare_input_subnormals(
//!     ScalarArithmetic::f32(),
//!     NumericalPermission::Forbidden,
//!     ScalarSupport::Exact,
//!     source,
//! );
//! ```
//!
//! Nor can an external producer assert that the compiler proved an exact
//! emulation:
//!
//! ```compile_fail
//! use tiler_compiler::target::ScalarSupport;
//! let _ = ScalarSupport::SupportedWithExactEmulation;
//! ```
//!
//! Producer and normative-reference identities cannot be silently swapped:
//!
//! ```compile_fail
//! use tiler_compiler::target::{
//!     TargetFactProducerIdentity, TargetFactSource, TargetNormativeReferenceIdentity,
//! };
//! let producer = TargetFactProducerIdentity::new("acme.profile.v1".to_owned(), 1).unwrap();
//! let reference =
//!     TargetNormativeReferenceIdentity::new("acme.spec.v1".to_owned(), 1).unwrap();
//! let _ = TargetFactSource::external_guarantee(reference, producer);
//! ```
//!
//! Measurement authority fixes its phase and validity; callers cannot spell a
//! portable measured fact:
//!
//! ```compile_fail
//! use tiler_compiler::target::{
//!     MeasuredFactAuthority, TargetFactProducerIdentity, TargetFactSource,
//! };
//! let producer = TargetFactProducerIdentity::new("acme.probe.v1".to_owned(), 1).unwrap();
//! let _ = TargetFactSource::measured(
//!     producer,
//!     MeasuredFactAuthority::PortableProfile,
//!     [],
//! );
//! ```
//!
//! Compile-profile measurement provenance cannot be constructed as a tuple or
//! erased into the general source type:
//!
//! ```compile_fail
//! use tiler_compiler::target::TargetCompileProfileMeasurementSource;
//! let _ = TargetCompileProfileMeasurementSource;
//! ```
//!
//! ```compile_fail
//! use tiler_compiler::target::{
//!     TargetCompileProfileMeasurementSource, TargetFactSource,
//! };
//! fn erase(source: TargetCompileProfileMeasurementSource) -> TargetFactSource {
//!     source.into()
//! }
//! ```

use std::sync::{Arc, OnceLock};

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::program::abi::{
    AvailabilityPhase, TargetPropertyKey, TargetPropertyProviderIdentity, TargetPropertyQuery,
};
use tiler_ir::schedule::{
    ArithmeticType, ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission, SubnormalMode,
};
use tiler_ir::semantic::{F32, ResolvedValueType};

use crate::feasibility::{
    CapabilityAxis, CapabilityFact, CapabilityQuery, CheckedTargetProfile, FactAuthority,
    FactProvenance, FactValidityScope, FeasibilityError, MAX_TARGET_PROFILE_DESCRIPTOR_BYTES,
};
use crate::honourability::{
    CompilerBuildIdentity, CompilerBuildRole, DeclaredBehaviour, DimensionBehaviour,
    ExecutionEnvironmentIdentity, FactEvidenceBasis, FactSourceProvenance, HonouringMeans,
    MAX_COMPILER_BUILDS_PER_CONTEXT, MAX_MEASUREMENT_CONTEXTS_PER_SOURCE,
    MAX_PROVENANCE_TEXT_BYTES, MeasurementContext, NumericalDimension, NumericalRefusalEvidence,
    ProvenanceIdentity, governed_profile_source,
};

pub(crate) const GOVERNED_TARGET_PROFILE_KEY: &str = "tiler.prototype-target-neutral-baseline.v1";
/// Domain of the complete producer declaration carried into artifact identity.
///
/// This is a new grammar, not a continuation of feasibility's
/// `tiler.target-profile.descriptor.v9`: the checked descriptor remains an
/// internal feasibility component, while this v10 declaration encodes the same
/// capability and numerical semantics plus exact dtype dispatch through one
/// shared provenance table. A reader of an older domain therefore cannot
/// mistake these bytes for the new grammar.
const COMPLETE_PROFILE_DESCRIPTOR_DOMAIN: &[u8] = b"tiler.target-profile.declaration.v10\0";
const PROFILE_SOURCE_DOMAIN: &[u8] = b"tiler.target-profile.fact-sources.v4\0";
const DISPATCHABILITY_DOMAIN: &[u8] = b"tiler.target-profile.dtype-dispatchability.v2\0";

/// Maximum byte length of one target-profile key.
pub const MAX_TARGET_PROFILE_KEY_BYTES: usize = 128;
/// Maximum UTF-8 byte length of one target-fact provenance field.
pub const MAX_TARGET_PROVENANCE_TEXT_BYTES: usize = MAX_PROVENANCE_TEXT_BYTES;
/// Maximum compiler builds admitted in one measurement context.
pub const MAX_TARGET_COMPILER_BUILDS_PER_CONTEXT: usize = MAX_COMPILER_BUILDS_PER_CONTEXT;
/// Maximum measurement contexts admitted in one measured source.
pub const MAX_TARGET_MEASUREMENT_CONTEXTS_PER_SOURCE: usize = MAX_MEASUREMENT_CONTEXTS_PER_SOURCE;
/// Maximum target profiles admitted in one compilation request.
pub const MAX_TARGET_PROFILES_PER_REQUEST: usize = 16;

/// Typed target-profile key validation diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetProfileKeyError {
    /// The key was empty.
    Empty,
    /// The encoded key exceeded the bounded identity field.
    TooLong {
        /// Actual encoded byte length.
        actual: usize,
        /// Maximum admitted encoded byte length.
        max: usize,
    },
    /// One byte was outside the canonical key alphabet.
    InvalidByte {
        /// Zero-based byte offset.
        index: usize,
        /// Refused byte value.
        value: u8,
    },
}

impl std::fmt::Display for TargetProfileKeyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for TargetProfileKeyError {}

/// The owned, validated key of one declared target profile.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TargetProfileKey(Arc<str>);

impl TargetProfileKey {
    /// Names a key governed by this compiler build.
    pub(crate) fn governed(key: &'static str) -> Self {
        Self::declared(key.to_owned()).expect("a source-governed target-profile key is valid")
    }

    /// Validates and retains a caller-owned key.
    ///
    /// # Errors
    ///
    /// Returns a key-specific diagnostic for an empty, oversized, or
    /// noncanonical key.
    pub fn new(key: String) -> Result<Self, TargetProfileKeyError> {
        let admitted = |byte: u8| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        };
        if key.is_empty() {
            return Err(TargetProfileKeyError::Empty);
        }
        if key.len() > MAX_TARGET_PROFILE_KEY_BYTES {
            return Err(TargetProfileKeyError::TooLong {
                actual: key.len(),
                max: MAX_TARGET_PROFILE_KEY_BYTES,
            });
        }
        if let Some((index, value)) = key.bytes().enumerate().find(|(_, byte)| !admitted(*byte)) {
            return Err(TargetProfileKeyError::InvalidByte { index, value });
        }
        Ok(Self(Arc::from(key)))
    }

    /// Returns the canonical validated spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TargetProfileKey {
    fn declared(key: String) -> Result<Self, TargetProfileKeyError> {
        Self::new(key)
    }
}

impl std::fmt::Display for TargetProfileKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl AsRef<str> for TargetProfileKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// A validated versioned identity for an external target-fact producer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetFactProducerIdentity(ProvenanceIdentity);

impl TargetFactProducerIdentity {
    /// Validates a producer key and its nonzero revision.
    ///
    /// # Errors
    ///
    /// Returns a field-specific diagnostic for an invalid key or zero revision.
    pub fn new(key: String, revision: u32) -> Result<Self, TargetFactSourceError> {
        validate_key_field(&key, "producer.key")?;
        validate_revision(revision, "producer.revision")?;
        let identity = ProvenanceIdentity::new(key, revision);
        Ok(Self(identity))
    }
}

/// A validated versioned identity for a cited normative guarantee.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetNormativeReferenceIdentity(ProvenanceIdentity);

impl TargetNormativeReferenceIdentity {
    /// Validates a normative-reference key and its nonzero revision.
    ///
    /// # Errors
    ///
    /// Returns a field-specific diagnostic for an invalid key or zero revision.
    pub fn new(key: String, revision: u32) -> Result<Self, TargetFactSourceError> {
        validate_key_field(&key, "normative-reference.key")?;
        validate_revision(revision, "normative-reference.revision")?;
        let identity = ProvenanceIdentity::new(key, revision);
        Ok(Self(identity))
    }
}

/// A validated versioned identity for a producer-defined compiler role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetCompilerRoleIdentity(ProvenanceIdentity);

impl TargetCompilerRoleIdentity {
    /// Validates a compiler-role key and its nonzero revision.
    ///
    /// # Errors
    ///
    /// Returns a field-specific diagnostic for an invalid key or zero revision.
    pub fn new(key: String, revision: u32) -> Result<Self, TargetFactSourceError> {
        validate_key_field(&key, "compiler-role.key")?;
        validate_revision(revision, "compiler-role.revision")?;
        let identity = ProvenanceIdentity::new(key, revision);
        Ok(Self(identity))
    }
}

/// The authority class of an externally measured fact.
///
/// `GovernedProfile` and the new external normative-profile authority are
/// intentionally absent. They are selected only by their dedicated private and
/// public constructors respectively.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeasuredFactAuthority {
    /// Evidence read from an artifact record.
    ArtifactEvidence,
    /// Evidence observed from a live device runtime.
    DeviceRuntime,
    /// Evidence observed after a kernel or pipeline was prepared.
    PreparedKernel,
    /// Evidence observed for one concrete launch.
    LaunchInstance,
}

impl MeasuredFactAuthority {
    const fn internal(self) -> (AvailabilityPhase, FactAuthority, FactValidityScope) {
        match self {
            Self::ArtifactEvidence => (
                AvailabilityPhase::ArtifactEvidence,
                FactAuthority::ArtifactEvidence,
                FactValidityScope::PreparedArtifact,
            ),
            Self::DeviceRuntime => (
                AvailabilityPhase::LiveDevicePreflight,
                FactAuthority::DeviceRuntime,
                FactValidityScope::DeviceInstance,
            ),
            Self::PreparedKernel => (
                AvailabilityPhase::PreparedKernelPreflight,
                FactAuthority::PreparedKernel,
                FactValidityScope::PreparedArtifact,
            ),
            Self::LaunchInstance => (
                AvailabilityPhase::LaunchPreflight,
                FactAuthority::LaunchInstance,
                FactValidityScope::LaunchInstance,
            ),
        }
    }
}

/// Semantic role performed by one compiler build in a measurement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TargetCompilerRole {
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
    ProducerDefined(TargetCompilerRoleIdentity),
}

impl TargetCompilerRole {
    fn internal(self) -> CompilerBuildRole {
        match self {
            Self::Frontend => CompilerBuildRole::Frontend,
            Self::Optimizer => CompilerBuildRole::Optimizer,
            Self::IntermediateTranslator => CompilerBuildRole::IntermediateTranslator,
            Self::CodeGenerator => CompilerBuildRole::CodeGenerator,
            Self::Assembler => CompilerBuildRole::Assembler,
            Self::Linker => CompilerBuildRole::Linker,
            Self::RuntimeCompiler => CompilerBuildRole::RuntimeCompiler,
            Self::ProducerDefined(identity) => CompilerBuildRole::ProviderDefined(identity.0),
        }
    }
}

/// One exact compiler component participating in a target measurement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetCompilerBuild(CompilerBuildIdentity);

impl TargetCompilerBuild {
    /// Constructs and validates a compiler-build record.
    ///
    /// # Errors
    ///
    /// Returns a field-specific diagnostic when required identity text is
    /// absent, noncanonical, or oversized.
    pub fn new(
        role: TargetCompilerRole,
        implementation: String,
        version: String,
        build: Option<String>,
    ) -> Result<Self, TargetFactSourceError> {
        validate_key_field(&implementation, "compiler-build.implementation")?;
        validate_text_field(&version, "compiler-build.version")?;
        if let Some(build) = &build {
            validate_text_field(build, "compiler-build.build")?;
        }
        let value = CompilerBuildIdentity::new(role.internal(), implementation, version, build);
        Ok(Self(value))
    }
}

/// Exact execution environment of one target measurement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetExecutionEnvironment(ExecutionEnvironmentIdentity);

impl TargetExecutionEnvironment {
    /// Starts a named execution-environment declaration.
    #[must_use]
    pub fn builder() -> TargetExecutionEnvironmentBuilder {
        TargetExecutionEnvironmentBuilder::default()
    }
}

/// Named construction surface for one exact measurement environment.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TargetExecutionEnvironmentBuilder {
    platform: Option<String>,
    platform_version: Option<String>,
    platform_build: Option<String>,
    architecture: Option<String>,
    hardware: Option<String>,
}

impl TargetExecutionEnvironmentBuilder {
    /// Sets the platform family.
    #[must_use]
    pub fn platform(mut self, value: String) -> Self {
        self.platform = Some(value);
        self
    }

    /// Sets the platform version.
    #[must_use]
    pub fn platform_version(mut self, value: String) -> Self {
        self.platform_version = Some(value);
        self
    }

    /// Sets the platform build identity.
    #[must_use]
    pub fn platform_build(mut self, value: String) -> Self {
        self.platform_build = Some(value);
        self
    }

    /// Sets the architecture.
    #[must_use]
    pub fn architecture(mut self, value: String) -> Self {
        self.architecture = Some(value);
        self
    }

    /// Sets the hardware identity.
    #[must_use]
    pub fn hardware(mut self, value: String) -> Self {
        self.hardware = Some(value);
        self
    }

    /// Validates and freezes the complete execution-environment record.
    ///
    /// # Errors
    ///
    /// Returns a field-specific diagnostic when a required environment field
    /// is absent, noncanonical, or oversized.
    pub fn build(self) -> Result<TargetExecutionEnvironment, TargetFactSourceError> {
        let Some(platform) = self.platform else {
            return Err(TargetFactSourceError::MissingField {
                field: "environment.platform",
            });
        };
        let Some(platform_version) = self.platform_version else {
            return Err(TargetFactSourceError::MissingField {
                field: "environment.platform-version",
            });
        };
        let Some(platform_build) = self.platform_build else {
            return Err(TargetFactSourceError::MissingField {
                field: "environment.platform-build",
            });
        };
        let Some(architecture) = self.architecture else {
            return Err(TargetFactSourceError::MissingField {
                field: "environment.architecture",
            });
        };
        let Some(hardware) = self.hardware else {
            return Err(TargetFactSourceError::MissingField {
                field: "environment.hardware",
            });
        };
        validate_key_field(&platform, "environment.platform")?;
        validate_text_field(&platform_version, "environment.platform-version")?;
        validate_text_field(&platform_build, "environment.platform-build")?;
        validate_key_field(&architecture, "environment.architecture")?;
        validate_text_field(&hardware, "environment.hardware")?;
        let value = ExecutionEnvironmentIdentity::new(
            platform,
            platform_version,
            platform_build,
            architecture,
            hardware,
        );
        Ok(TargetExecutionEnvironment(value))
    }
}

/// One compiler-build set paired with the environment in which it ran.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetMeasurementContext(MeasurementContext);

impl TargetMeasurementContext {
    /// Constructs a context. At least one distinct compiler build is required.
    ///
    /// # Errors
    ///
    /// Returns a set-specific diagnostic for an empty, duplicated, or oversized
    /// compiler-build set.
    pub fn new(
        compiler_builds: impl IntoIterator<Item = TargetCompilerBuild>,
        environment: TargetExecutionEnvironment,
    ) -> Result<Self, TargetFactSourceError> {
        let compiler_builds = compiler_builds
            .into_iter()
            .take(MAX_TARGET_COMPILER_BUILDS_PER_CONTEXT + 1)
            .collect::<Vec<_>>();
        if compiler_builds.is_empty() {
            return Err(TargetFactSourceError::EmptyCompilerBuildSet);
        }
        if compiler_builds.len() > MAX_TARGET_COMPILER_BUILDS_PER_CONTEXT {
            return Err(TargetFactSourceError::TooManyCompilerBuilds {
                actual: compiler_builds.len(),
                max: MAX_TARGET_COMPILER_BUILDS_PER_CONTEXT,
            });
        }
        if compiler_builds
            .iter()
            .enumerate()
            .any(|(index, build)| compiler_builds[..index].contains(build))
        {
            return Err(TargetFactSourceError::DuplicateCompilerBuild);
        }
        let value = MeasurementContext::new(
            compiler_builds.into_iter().map(|build| build.0).collect(),
            environment.0,
        );
        Ok(Self(value))
    }
}

/// Structured source attribution for target-profile facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetFactSource(Arc<FactSourceProvenance>);

/// Empirical compiler-profile provenance bound to exact measurement contexts.
///
/// Unlike [`TargetFactSource::external_guarantee`], this source cannot claim
/// portable normative authority. Its phase, authority, and validity are fixed
/// to compile profile, measured profile, and measured environment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetCompileProfileMeasurementSource(Arc<FactSourceProvenance>);

impl TargetCompileProfileMeasurementSource {
    /// Constructs compiler-profile measurement provenance.
    ///
    /// # Errors
    ///
    /// Returns a set-specific diagnostic for an empty, duplicated, or oversized
    /// measurement-context collection.
    pub fn new(
        producer: TargetFactProducerIdentity,
        contexts: impl IntoIterator<Item = TargetMeasurementContext>,
    ) -> Result<Self, TargetFactSourceError> {
        let contexts = collect_measurement_contexts(contexts)?;
        Ok(Self(Arc::new(FactSourceProvenance::measured(
            AvailabilityPhase::CompileProfile,
            FactAuthority::MeasuredProfile,
            FactValidityScope::MeasuredEnvironment,
            producer.0,
            contexts,
        ))))
    }
}

impl TargetFactSource {
    /// Attributes a portable normative/spec-backed guarantee to an external
    /// producer.
    ///
    /// This does not create compiler-governed evidence. The canonical source
    /// retains the producer and reference as two independently versioned
    /// identities.
    #[must_use]
    pub fn external_guarantee(
        producer: TargetFactProducerIdentity,
        reference: TargetNormativeReferenceIdentity,
    ) -> Self {
        Self(Arc::new(FactSourceProvenance::externally_guaranteed(
            producer.0,
            reference.0,
        )))
    }

    /// Attributes an empirical fact to its producer, authority, and exact
    /// measurement contexts.
    ///
    /// # Errors
    ///
    /// Returns a set-specific diagnostic for an empty, duplicated, or oversized
    /// context set.
    pub fn measured(
        producer: TargetFactProducerIdentity,
        authority: MeasuredFactAuthority,
        contexts: impl IntoIterator<Item = TargetMeasurementContext>,
    ) -> Result<Self, TargetFactSourceError> {
        let (phase, authority, validity) = authority.internal();
        let contexts = collect_measurement_contexts(contexts)?;
        let value =
            FactSourceProvenance::measured(phase, authority, validity, producer.0, contexts);
        Ok(Self(Arc::new(value)))
    }
}

fn collect_measurement_contexts(
    contexts: impl IntoIterator<Item = TargetMeasurementContext>,
) -> Result<Vec<MeasurementContext>, TargetFactSourceError> {
    let contexts = contexts
        .into_iter()
        .take(MAX_TARGET_MEASUREMENT_CONTEXTS_PER_SOURCE + 1)
        .collect::<Vec<_>>();
    if contexts.is_empty() {
        return Err(TargetFactSourceError::EmptyMeasurementContextSet);
    }
    if contexts.len() > MAX_TARGET_MEASUREMENT_CONTEXTS_PER_SOURCE {
        return Err(TargetFactSourceError::TooManyMeasurementContexts {
            actual: contexts.len(),
            max: MAX_TARGET_MEASUREMENT_CONTEXTS_PER_SOURCE,
        });
    }
    if contexts
        .iter()
        .enumerate()
        .any(|(index, context)| contexts[..index].contains(context))
    {
        return Err(TargetFactSourceError::DuplicateMeasurementContext);
    }
    Ok(contexts.into_iter().map(|context| context.0).collect())
}

// ---------------------------------------------------------------------------
// Reading a declared fact back.
//
// The types above declare target facts; the ones below read one back out of a
// refusal. They are separate vocabularies on purpose. A declaration constructor
// validates caller-supplied text and takes ownership, and is deliberately narrow
// — `MeasuredFactAuthority` omits the governed and external authorities because
// no caller may claim them. A refusal view must be able to report *any*
// authority the compiler itself can attribute, including those two, and it
// borrows from the retained fact rather than copying it, so a diagnostic cannot
// be handed a provenance record it could edit and hand back.
// ---------------------------------------------------------------------------

/// The class of authority vouching for one declared target fact.
///
/// The complete read-side space, unlike [`MeasuredFactAuthority`], which is the
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

/// Typed refusal from external target-fact source construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetFactSourceError {
    /// A required named field was absent or empty.
    MissingField {
        /// Stable field path.
        field: &'static str,
    },
    /// A field contained a byte outside its canonical alphabet.
    InvalidFieldByte {
        /// Stable field path.
        field: &'static str,
        /// Zero-based byte offset.
        index: usize,
        /// Refused byte value.
        value: u8,
    },
    /// A field exceeded its admitted UTF-8 byte bound.
    FieldTooLong {
        /// Stable field path.
        field: &'static str,
        /// Actual encoded byte length.
        actual: usize,
        /// Maximum admitted encoded byte length.
        max: usize,
    },
    /// A versioned identity used the reserved zero revision.
    ZeroRevision {
        /// Stable revision field path.
        field: &'static str,
    },
    /// No compiler build was supplied for a measurement context.
    EmptyCompilerBuildSet,
    /// One exact compiler build appeared more than once.
    DuplicateCompilerBuild,
    /// A compiler-build set exceeded its admitted cardinality.
    TooManyCompilerBuilds {
        /// Observed cardinality, capped at `max + 1`.
        actual: usize,
        /// Maximum admitted cardinality.
        max: usize,
    },
    /// No measurement context was supplied for a measured source.
    EmptyMeasurementContextSet,
    /// One exact measurement context appeared more than once.
    DuplicateMeasurementContext,
    /// A measurement-context set exceeded its admitted cardinality.
    TooManyMeasurementContexts {
        /// Observed cardinality, capped at `max + 1`.
        actual: usize,
        /// Maximum admitted cardinality.
        max: usize,
    },
}

impl std::fmt::Display for TargetFactSourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for TargetFactSourceError {}

fn validate_revision(revision: u32, field: &'static str) -> Result<(), TargetFactSourceError> {
    if revision == 0 {
        return Err(TargetFactSourceError::ZeroRevision { field });
    }
    Ok(())
}

fn validate_key_field(value: &str, field: &'static str) -> Result<(), TargetFactSourceError> {
    validate_field_bound(value, field)?;
    if let Some((index, value)) = value.bytes().enumerate().find(|(_, byte)| {
        !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_'))
    }) {
        return Err(TargetFactSourceError::InvalidFieldByte {
            field,
            index,
            value,
        });
    }
    Ok(())
}

fn validate_text_field(value: &str, field: &'static str) -> Result<(), TargetFactSourceError> {
    validate_field_bound(value, field)?;
    if value.trim() != value {
        let index = if value.trim_start().len() == value.len() {
            value.trim_end().len()
        } else {
            0
        };
        return Err(TargetFactSourceError::InvalidFieldByte {
            field,
            index,
            value: value.as_bytes()[index],
        });
    }
    if let Some((index, value)) = value
        .bytes()
        .enumerate()
        .find(|(_, byte)| !(byte.is_ascii_graphic() || *byte == b' '))
    {
        return Err(TargetFactSourceError::InvalidFieldByte {
            field,
            index,
            value,
        });
    }
    Ok(())
}

fn validate_field_bound(value: &str, field: &'static str) -> Result<(), TargetFactSourceError> {
    if value.is_empty() {
        return Err(TargetFactSourceError::MissingField { field });
    }
    if value.len() > MAX_TARGET_PROVENANCE_TEXT_BYTES {
        return Err(TargetFactSourceError::FieldTooLong {
            field,
            actual: value.len(),
            max: MAX_TARGET_PROVENANCE_TEXT_BYTES,
        });
    }
    Ok(())
}

/// Owned identity of the profile that attributed a checked fact.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct TargetProfileIdentity {
    key: TargetProfileKey,
}

impl TargetProfileIdentity {
    #[cfg(test)]
    pub(crate) fn new(key: &'static str) -> Self {
        Self::governed(key)
    }

    pub(crate) fn from_key(key: TargetProfileKey) -> Self {
        Self { key }
    }

    #[cfg(test)]
    pub(crate) fn governed(key: &'static str) -> Self {
        Self::from_key(TargetProfileKey::governed(key))
    }

    pub(crate) fn key(&self) -> &str {
        self.key.as_str()
    }

    pub(crate) const fn public_key(&self) -> &TargetProfileKey {
        &self.key
    }
}

impl From<&TargetProfileIdentity> for TargetProfileIdentity {
    fn from(value: &TargetProfileIdentity) -> Self {
        value.clone()
    }
}

/// One scalar-arithmetic policy subject with its complete semantic dtype.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarArithmetic {
    arithmetic: ArithmeticType,
    resolved_type: ResolvedValueType,
}

impl ScalarArithmetic {
    fn governed_f32() -> Self {
        Self::new(ArithmeticType::F32, F32::resolved_type())
            .expect("the governed F32 arithmetic subject is registered")
    }

    /// Returns the sole scalar-arithmetic subject this compiler currently
    /// registers: the complete governed `tiler::f32@1` resolved type.
    ///
    /// There is no `(ArithmeticType, ResolvedValueType)` public constructor.
    /// Such a constructor could pair a width with a merely similar type and
    /// claim arithmetic semantics no registry admitted.
    #[must_use]
    pub fn f32() -> Self {
        Self::governed_f32()
    }

    fn new(
        arithmetic: ArithmeticType,
        resolved_type: ResolvedValueType,
    ) -> Result<Self, TargetProfileBuildError> {
        // `F32` is the only arithmetic/type association this compiler has
        // registered. Similar-looking names are not evidence that an F16,
        // BF16, or F64 semantic type exists, so those pairs remain behind this
        // validation seam until a named registry authority can prove them.
        if arithmetic != ArithmeticType::F32 || resolved_type != F32::resolved_type() {
            return Err(TargetProfileBuildError::UnvalidatedScalarArithmetic);
        }
        Ok(Self {
            arithmetic,
            resolved_type,
        })
    }

    pub(crate) const fn arithmetic(&self) -> ArithmeticType {
        self.arithmetic
    }

    /// Returns the complete resolved semantic value type.
    #[must_use]
    pub const fn resolved_type(&self) -> &ResolvedValueType {
        &self.resolved_type
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        bytes.push(self.arithmetic.tag());
        push_slice(bytes, self.resolved_type.canonical_encoding().as_bytes());
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScalarHonourabilityDeclaration {
    subject: ScalarArithmetic,
    dimension: NumericalDimension,
    behaviour: DimensionBehaviour,
    means: HonouringMeans,
    source: Arc<FactSourceProvenance>,
}

impl ScalarHonourabilityDeclaration {
    fn governed_exact(dimension: NumericalDimension, behaviour: DimensionBehaviour) -> Self {
        Self {
            subject: ScalarArithmetic::governed_f32(),
            dimension,
            behaviour,
            means: HonouringMeans::SupportedExactly,
            source: governed_profile_source(),
        }
    }

    fn validate(&self) -> Result<(), TargetProfileBuildError> {
        if !self.dimension.admits(self.behaviour) {
            return Err(TargetProfileBuildError::InvalidDimensionBehaviour);
        }
        match &self.means {
            HonouringMeans::SupportedExactly | HonouringMeans::Unsupported => {}
            HonouringMeans::SupportedWithExactEmulation => {
                return Err(TargetProfileBuildError::UnverifiedExactEmulation);
            }
            HonouringMeans::SupportedOnlyUnderDeclaredRelaxation { relaxation } => {
                if !relaxation.dimension().admits(relaxation.behaviour())
                    || relaxation.arithmetic() != self.subject.arithmetic()
                    || relaxation.resolved_type() != self.subject.resolved_type()
                {
                    return Err(TargetProfileBuildError::InvalidRelaxation);
                }
            }
        }
        if !self.source.is_valid() {
            return Err(TargetProfileBuildError::InvalidProducerClaim);
        }
        Ok(())
    }

    fn declared(&self) -> DeclaredBehaviour {
        DeclaredBehaviour::new(
            self.dimension,
            self.subject.arithmetic(),
            self.subject.resolved_type().clone(),
            self.behaviour,
            self.means.clone(),
            Arc::clone(&self.source),
        )
    }

    fn encode(&self, bytes: &mut Vec<u8>) {
        self.subject.encode(bytes);
        bytes.push(self.dimension.tag());
        self.behaviour.encode(bytes);
        self.means.encode(bytes);
        push_slice(bytes, self.source.canonical_bytes().as_slice());
    }
}

/// Support for the governed KIR index-arithmetic family.
///
/// This is deliberately not a raw integer width. `CompleteU64` means the target
/// supports every unsigned-64 operation that [`tiler_ir::kernel::KernelType::Index`]
/// may emit, rather than merely storing a 64-bit scalar.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum IndexArithmeticSupport {
    /// The governed unsigned-64 index operation family is unsupported.
    Unsupported,
    /// The governed unsigned-64 index operation family is supported completely.
    CompleteU64,
}

impl IndexArithmeticSupport {
    const fn bound(self) -> u64 {
        match self {
            Self::Unsupported => 0,
            Self::CompleteU64 => 1,
        }
    }
}

/// Width of a target's device address model.
///
/// This fact does not describe integer arithmetic, buffer length, or launch
/// coordinate delivery. A profile omits it when no applicable authority has
/// established the address model.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DeviceAddressWidth {
    /// A 32-bit device address model.
    Bits32,
    /// A 64-bit device address model.
    Bits64,
}

impl DeviceAddressWidth {
    /// Returns the width in bits.
    #[must_use]
    pub const fn bits(self) -> u8 {
        match self {
            Self::Bits32 => 32,
            Self::Bits64 => 64,
        }
    }
}

/// Qualitative ability of a target family to dispatch one exact dtype.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DTypeDispatchability {
    /// The exact dtype can be dispatched.
    Dispatchable,
    /// The exact dtype is explicitly unsupported.
    Unsupported,
}

impl DTypeDispatchability {
    const fn tag(self) -> u8 {
        match self {
            Self::Dispatchable => 0x01,
            Self::Unsupported => 0x02,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DTypeDispatchabilityFact {
    resolved_type: ResolvedValueType,
    verdict: DTypeDispatchability,
    source: Arc<FactSourceProvenance>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct QuantitativeCapabilityDeclaration {
    axis: CapabilityAxis,
    bound: u64,
    source: Arc<FactSourceProvenance>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct QuantitativeCapabilityQueryDeclaration {
    axis: CapabilityAxis,
    query: TargetPropertyQuery,
}

impl QuantitativeCapabilityDeclaration {
    fn validate(&self) -> Result<(), TargetProfileBuildError> {
        if !self.source.is_valid() {
            return Err(TargetProfileBuildError::InvalidProducerClaim);
        }
        Ok(())
    }

    fn encode_source_index(bytes: &mut Vec<u8>, source_index: usize) {
        encode_compact_index(bytes, source_index);
    }
}

impl DTypeDispatchabilityFact {
    fn validate(&self) -> Result<(), TargetProfileBuildError> {
        if !self.source.is_valid() {
            return Err(TargetProfileBuildError::InvalidProducerClaim);
        }
        Ok(())
    }

    fn encode(&self, bytes: &mut Vec<u8>, source_index: usize) {
        push_slice(bytes, self.resolved_type.canonical_encoding().as_bytes());
        bytes.push(self.verdict.tag());
        encode_compact_index(bytes, source_index);
    }
}

fn encode_compact_index(bytes: &mut Vec<u8>, mut value: usize) {
    loop {
        let low = u8::try_from(value & 0x7f).expect("seven masked bits fit in u8");
        value >>= 7;
        if value == 0 {
            bytes.push(low);
            break;
        }
        bytes.push(low | 0x80);
    }
}

/// Result of an exact dtype-dispatchability lookup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DTypeDispatchabilityResolution {
    /// An exact declaration admits dispatch.
    Dispatchable,
    /// An exact declaration refuses dispatch.
    Unsupported,
    /// An exact declaration exists, but only from this later phase.
    Deferred {
        /// Earliest phase at which an exact declaration can resolve.
        available_at: AvailabilityPhase,
    },
    /// No exact declaration exists.
    Unknown,
}

/// One immutable, intrinsically checked target declaration.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetProfile {
    data: Arc<TargetProfileData>,
}

#[derive(Debug, Eq, PartialEq)]
struct TargetProfileData {
    key: TargetProfileKey,
    checked: CheckedTargetProfile,
    quantitative: Box<[QuantitativeCapabilityDeclaration]>,
    scalar: Box<[ScalarHonourabilityDeclaration]>,
    dispatchability: Box<[DTypeDispatchabilityFact]>,
    descriptor: Box<[u8]>,
}

/// Consuming producer-side builder for one immutable target profile.
#[derive(Clone, Debug)]
pub struct TargetProfileBuilder {
    key: TargetProfileKey,
    quantitative: Vec<QuantitativeCapabilityDeclaration>,
    queries: Vec<QuantitativeCapabilityQueryDeclaration>,
    scalar: Vec<ScalarHonourabilityDeclaration>,
    dispatchability: Vec<DTypeDispatchabilityFact>,
}

impl TargetProfileBuilder {
    /// Starts one externally declared sparse profile from a validated key.
    ///
    /// Every omitted quantitative axis remains unknown.
    #[must_use]
    pub fn new(key: TargetProfileKey) -> Self {
        Self {
            key,
            quantitative: Vec::new(),
            queries: Vec::new(),
            scalar: Vec::new(),
            dispatchability: Vec::new(),
        }
    }

    fn governed() -> Self {
        // The grid row is the deliberately conservative four-thread guarantee
        // of this bounded macOS Metal profile, not a hardware maximum. In the
        // macOS 26.5 SDK, `MTLComputeCommandEncoder.h` documents
        // `dispatchThreads:threadsPerThreadgroup:` as accepting an
        // arbitrarily-sized grid whose dimensions need not be threadgroup
        // multiples, and `MTLTypes.h` defines each `MTLSize` dimension as
        // `NSUInteger`; the API is available from macOS 10.13. Those primary
        // declarations prove that extent four is representable on the governed
        // profile. They do not prove 65,535, an Apple-family maximum, or any
        // prepared pipeline's workgroup capacity. The shared source below
        // identifies the compiler-governed prototype guarantee; the production
        // Metal profile ticket replaces it with its full per-row authority
        // ledger rather than mislabelling this as a device measurement.
        let source = TargetFactSource(governed_profile_source());
        let mut builder = Self::new(TargetProfileKey::governed(GOVERNED_TARGET_PROFILE_KEY));
        builder
            .declare_max_threads_per_grid_axis(4, source.clone())
            .expect("the governed grid-axis declaration is valid");
        builder
            .declare_max_threads_per_workgroup_query(
                TargetPropertyQuery::new(
                    TargetPropertyKey::new(
                        "tiler.target.prepared-entry.max-threads-per-workgroup.v1",
                    )
                    .expect("the governed workgroup property key is valid"),
                    AvailabilityPhase::PreparedKernelPreflight,
                    TargetPropertyProviderIdentity::new("tiler", "prepared-entry-properties", 1)
                        .expect("the governed target-query provider identity is valid"),
                )
                .expect("the governed workgroup query is deferred"),
            )
            .expect("the governed workgroup query declaration is valid");
        builder
            .declare_max_buffer_bindings_per_entry(2, source.clone())
            .expect("the governed binding declaration is valid");
        builder
            .declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
            .expect("the governed index-arithmetic declaration is valid");
        builder
            .declare_device_memory(true, source.clone())
            .expect("the governed device-memory declaration is valid");
        builder
            .declare_local_memory_bytes(0, source)
            .expect("the governed local-memory declaration is valid");
        builder.scalar = governed_target_honourability();
        // This is a compiler-governed prototype, target-neutral dispatch fact.
        // It does not claim Metal support or any device-family measurement.
        builder
            .declare_dtype_dispatchability(
                F32::resolved_type(),
                DTypeDispatchability::Dispatchable,
                TargetFactSource(governed_profile_source()),
            )
            .expect("the governed F32 dispatch declaration is valid");
        builder
    }

    fn declare_quantitative(
        &mut self,
        axis: CapabilityAxis,
        bound: u64,
        source: Arc<FactSourceProvenance>,
    ) -> Result<(), TargetProfileBuildError> {
        let declaration = QuantitativeCapabilityDeclaration {
            axis,
            bound,
            source,
        };
        declaration.validate()?;
        if self.queries.iter().any(|query| query.axis == axis) {
            return Err(
                TargetProfileBuildError::ConflictingQuantitativeFactAndQuery { axis: axis.key() },
            );
        }
        if self.quantitative.iter().any(|existing| {
            existing.axis == declaration.axis
                && existing.source.phase() == declaration.source.phase()
        }) {
            return Err(TargetProfileBuildError::DuplicateQuantitativeCapability {
                axis: declaration.axis.key(),
                phase: declaration.source.phase(),
            });
        }
        self.quantitative.push(declaration);
        Ok(())
    }

    /// Declares the maximum launch-grid extent along one axis.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_max_threads_per_grid_axis(
        &mut self,
        bound: u64,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::GridAxisThreads, bound, source.0)
    }

    /// Declares a measured maximum launch-grid extent along one axis.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_measured_max_threads_per_grid_axis(
        &mut self,
        bound: u64,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::GridAxisThreads, bound, source.0)
    }

    /// Declares the maximum number of threads in one workgroup.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_max_threads_per_workgroup(
        &mut self,
        bound: u32,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::WorkgroupThreads, u64::from(bound), source.0)
    }

    /// Declares a measured maximum number of threads in one workgroup.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_measured_max_threads_per_workgroup(
        &mut self,
        bound: u32,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::WorkgroupThreads, u64::from(bound), source.0)
    }

    /// Declares the prepared-entry query that supplies the exact maximum
    /// threads-per-workgroup value for a future compiled kernel.
    ///
    /// This is deliberately separate from
    /// [`Self::declare_max_threads_per_workgroup`]: that method records an
    /// available value, while this one records how an exact prepared entry will
    /// produce a value later.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting a duplicate or wrong-phase
    /// query. A live-device maximum cannot substitute for the prepared
    /// pipeline's function-specific maximum.
    pub fn declare_max_threads_per_workgroup_query(
        &mut self,
        query: TargetPropertyQuery,
    ) -> Result<(), TargetProfileBuildError> {
        if query.available_at() != AvailabilityPhase::PreparedKernelPreflight {
            return Err(TargetProfileBuildError::InvalidQuantitativeQueryPhase {
                axis: CapabilityAxis::WorkgroupThreads.key(),
                required: AvailabilityPhase::PreparedKernelPreflight,
                actual: query.available_at(),
            });
        }
        if self
            .quantitative
            .iter()
            .any(|existing| existing.axis == CapabilityAxis::WorkgroupThreads)
        {
            return Err(
                TargetProfileBuildError::ConflictingQuantitativeFactAndQuery {
                    axis: CapabilityAxis::WorkgroupThreads.key(),
                },
            );
        }
        if self
            .queries
            .iter()
            .any(|existing| existing.axis == CapabilityAxis::WorkgroupThreads)
        {
            return Err(TargetProfileBuildError::DuplicateQuantitativeQuery {
                axis: CapabilityAxis::WorkgroupThreads.key(),
            });
        }
        self.queries.push(QuantitativeCapabilityQueryDeclaration {
            axis: CapabilityAxis::WorkgroupThreads,
            query,
        });
        Ok(())
    }

    /// Declares the maximum distinct buffer bindings per kernel entry.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_max_buffer_bindings_per_entry(
        &mut self,
        bound: u32,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::BufferBindings, u64::from(bound), source.0)
    }

    /// Declares a measured maximum number of buffer bindings per kernel entry.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_measured_max_buffer_bindings_per_entry(
        &mut self,
        bound: u32,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::BufferBindings, u64::from(bound), source.0)
    }

    /// Declares support for the governed KIR index-arithmetic family.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_index_arithmetic(
        &mut self,
        support: IndexArithmeticSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(
            CapabilityAxis::IndexArithmeticU64,
            support.bound(),
            source.0,
        )
    }

    /// Declares measured support for the governed KIR index-arithmetic family.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_measured_index_arithmetic(
        &mut self,
        support: IndexArithmeticSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(
            CapabilityAxis::IndexArithmeticU64,
            support.bound(),
            source.0,
        )
    }

    /// Declares the exact device address-model width.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_device_address_width(
        &mut self,
        width: DeviceAddressWidth,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(
            CapabilityAxis::DeviceAddressWidthBits,
            u64::from(width.bits()),
            source.0,
        )
    }

    /// Declares a measured exact device address-model width.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_measured_device_address_width(
        &mut self,
        width: DeviceAddressWidth,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(
            CapabilityAxis::DeviceAddressWidthBits,
            u64::from(width.bits()),
            source.0,
        )
    }

    /// Declares whether an explicitly addressable device memory space exists.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_device_memory(
        &mut self,
        supported: bool,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(
            CapabilityAxis::DeviceAddressSpace,
            u64::from(supported),
            source.0,
        )
    }

    /// Declares measured support for an explicitly addressable device memory space.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_measured_device_memory(
        &mut self,
        supported: bool,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(
            CapabilityAxis::DeviceAddressSpace,
            u64::from(supported),
            source.0,
        )
    }

    /// Declares the maximum explicitly staged local memory in bytes.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_local_memory_bytes(
        &mut self,
        bound: u64,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::LocalMemoryBytes, bound, source.0)
    }

    /// Declares a measured maximum explicitly staged local memory size.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    pub fn declare_measured_local_memory_bytes(
        &mut self,
        bound: u64,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_quantitative(CapabilityAxis::LocalMemoryBytes, bound, source.0)
    }

    fn declare_scalar(
        &mut self,
        subject: ScalarArithmetic,
        dimension: NumericalDimension,
        behaviour: DimensionBehaviour,
        support: ScalarSupport,
        source: Arc<FactSourceProvenance>,
    ) -> Result<(), TargetProfileBuildError> {
        let declaration = ScalarHonourabilityDeclaration {
            subject,
            dimension,
            behaviour,
            means: support.means(),
            source,
        };
        declaration.validate()?;
        if self.scalar.iter().any(|existing| {
            existing.dimension == declaration.dimension
                && existing.subject == declaration.subject
                && existing.behaviour == declaration.behaviour
                && existing.source.phase() == declaration.source.phase()
        }) {
            return Err(TargetProfileBuildError::DuplicateScalarDeclaration);
        }
        self.scalar.push(declaration);
        Ok(())
    }

    /// Declares support for one exact input-subnormal behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_input_subnormals(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: SubnormalMode,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::InputSubnormals,
            DimensionBehaviour::Subnormals(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one exact result-subnormal behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_result_subnormals(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: SubnormalMode,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::ResultSubnormals,
            DimensionBehaviour::Subnormals(behaviour),
            support,
            source.0,
        )
    }

    /// Declares the one measured scalar input-subnormal realization delivered
    /// by a compiler profile, and explicitly refuses the other two realizations.
    ///
    /// The input-subnormal dimension receives a complete, exclusive three-row
    /// table. If that dimension already contains any row for the exact subject
    /// at any phase or behaviour, this operation refuses before inserting
    /// anything.
    ///
    /// # Errors
    ///
    /// Returns a typed conflict naming the exact subject, dimension, and phase
    /// of the first pre-existing row, without mutating the builder.
    pub fn declare_measured_input_subnormal_behaviour(
        &mut self,
        subject: ScalarArithmetic,
        delivered: SubnormalMode,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_measured_subnormal_dimension(
            subject,
            NumericalDimension::InputSubnormals,
            delivered,
            source,
        )
    }

    /// Declares the one measured scalar result-subnormal realization delivered
    /// by a compiler profile, and explicitly refuses the other two realizations.
    ///
    /// The result-subnormal dimension receives a complete, exclusive three-row
    /// table. If that dimension already contains any row for the exact subject
    /// at any phase or behaviour, this operation refuses before inserting
    /// anything.
    ///
    /// # Errors
    ///
    /// Returns a typed conflict naming the exact subject, dimension, and phase
    /// of the first pre-existing row, without mutating the builder.
    pub fn declare_measured_result_subnormal_behaviour(
        &mut self,
        subject: ScalarArithmetic,
        delivered: SubnormalMode,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_measured_subnormal_dimension(
            subject,
            NumericalDimension::ResultSubnormals,
            delivered,
            source,
        )
    }

    fn declare_measured_subnormal_dimension(
        &mut self,
        subject: ScalarArithmetic,
        dimension: NumericalDimension,
        delivered: SubnormalMode,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        debug_assert!(matches!(
            dimension,
            NumericalDimension::InputSubnormals | NumericalDimension::ResultSubnormals
        ));
        if let Some(existing) = self
            .scalar
            .iter()
            .find(|existing| existing.subject == subject && existing.dimension == dimension)
        {
            return Err(TargetProfileBuildError::ConflictingSubnormalDeclaration {
                subject: Box::new(subject),
                dimension: existing.dimension.key(),
                phase: existing.source.phase(),
            });
        }

        let (preserve, signed, positive) = match delivered {
            SubnormalMode::Preserve => (
                ScalarSupport::Exact,
                ScalarSupport::Unsupported,
                ScalarSupport::Unsupported,
            ),
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            } => (
                ScalarSupport::Unsupported,
                ScalarSupport::Exact,
                ScalarSupport::Unsupported,
            ),
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            } => (
                ScalarSupport::Unsupported,
                ScalarSupport::Unsupported,
                ScalarSupport::Exact,
            ),
        };
        let rows = [
            (SubnormalMode::Preserve, preserve),
            (
                SubnormalMode::FlushToZero {
                    zero_sign: FlushedZeroSign::PreservesSign,
                },
                signed,
            ),
            (
                SubnormalMode::FlushToZero {
                    zero_sign: FlushedZeroSign::AlwaysPositive,
                },
                positive,
            ),
        ];
        let source = source.0;
        let declarations = rows.map(|(behaviour, support)| ScalarHonourabilityDeclaration {
            subject: subject.clone(),
            dimension,
            behaviour: DimensionBehaviour::Subnormals(behaviour),
            means: support.means(),
            source: Arc::clone(&source),
        });
        for declaration in &declarations {
            declaration.validate()?;
        }
        self.scalar.extend(declarations);
        Ok(())
    }

    /// Declares support for one contraction permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_contraction(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::Contraction,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one contraction permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_contraction(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::Contraction,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one reassociation permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_reassociation(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::Reassociation,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one reassociation permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_reassociation(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::Reassociation,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one operand-permutation permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_permutation(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::Permutation,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one operand-permutation permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_permutation(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::Permutation,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one signed-zero permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_signed_zero(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::SignedZero,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one signed-zero permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_signed_zero(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::SignedZero,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one reciprocal-transform permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_reciprocal_transform(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::ReciprocalTransform,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one reciprocal-transform permission.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_reciprocal_transform(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: NumericalPermission,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::ReciprocalTransform,
            DimensionBehaviour::Transform(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one approximation envelope.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_approximate_intrinsics(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: tiler_ir::schedule::ApproximationEnvelope,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::ApproximateIntrinsics,
            DimensionBehaviour::Approximation(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one approximation envelope.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_approximate_intrinsics(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: tiler_ir::schedule::ApproximationEnvelope,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::ApproximateIntrinsics,
            DimensionBehaviour::Approximation(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one NaN-assumption behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_nan_assumptions(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: ExceptionalValueAssumption,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::NanAssumptions,
            DimensionBehaviour::ExceptionalValue(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one NaN-assumption behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_nan_assumptions(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: ExceptionalValueAssumption,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::NanAssumptions,
            DimensionBehaviour::ExceptionalValue(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one infinity-assumption behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_infinity_assumptions(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: ExceptionalValueAssumption,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::InfinityAssumptions,
            DimensionBehaviour::ExceptionalValue(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one infinity-assumption behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_infinity_assumptions(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: ExceptionalValueAssumption,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::InfinityAssumptions,
            DimensionBehaviour::ExceptionalValue(behaviour),
            support,
            source.0,
        )
    }

    /// Declares support for one observable materialization-rounding behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_materialization_rounding(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: tiler_ir::schedule::MaterializationRounding,
        support: ScalarSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::MaterializationRounding,
            DimensionBehaviour::Rounding(behaviour),
            support,
            source.0,
        )
    }

    /// Declares measured support for one observable materialization-rounding behaviour.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate declaration.
    pub fn declare_measured_materialization_rounding(
        &mut self,
        subject: ScalarArithmetic,
        behaviour: tiler_ir::schedule::MaterializationRounding,
        support: ScalarSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_scalar(
            subject,
            NumericalDimension::MaterializationRounding,
            DimensionBehaviour::Rounding(behaviour),
            support,
            source.0,
        )
    }

    /// Declares dispatchability for one exact full resolved value type.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    ///
    /// No neighbouring nominal type, parameterized type, or encoded value
    /// inherits this declaration.
    pub fn declare_dtype_dispatchability(
        &mut self,
        resolved_type: ResolvedValueType,
        verdict: DTypeDispatchability,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_dtype_dispatchability_with_source(resolved_type, verdict, source.0)
    }

    /// Declares measured dispatchability for one exact full resolved value type.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate fact.
    ///
    /// No neighbouring nominal type, parameterized type, or encoded value
    /// inherits this declaration.
    pub fn declare_measured_dtype_dispatchability(
        &mut self,
        resolved_type: ResolvedValueType,
        verdict: DTypeDispatchability,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_dtype_dispatchability_with_source(resolved_type, verdict, source.0)
    }

    fn declare_dtype_dispatchability_with_source(
        &mut self,
        resolved_type: ResolvedValueType,
        verdict: DTypeDispatchability,
        source: Arc<FactSourceProvenance>,
    ) -> Result<(), TargetProfileBuildError> {
        let fact = DTypeDispatchabilityFact {
            resolved_type,
            verdict,
            source,
        };
        fact.validate()?;
        if self.dispatchability.iter().any(|existing| {
            existing.resolved_type == fact.resolved_type
                && existing.source.phase() == fact.source.phase()
        }) {
            return Err(TargetProfileBuildError::DuplicateDispatchability);
        }
        self.dispatchability.push(fact);
        Ok(())
    }

    /// Verifies and freezes this profile.
    ///
    /// # Errors
    ///
    /// Returns the first intrinsic checking or bounded-descriptor diagnostic.
    /// Public declaration methods reject invalid or duplicate rows atomically,
    /// before insertion, so a failed build has no repairable draft to return.
    pub fn build(self) -> Result<TargetProfile, TargetProfileBuildError> {
        self.freeze()
    }

    fn validate_declarations(&self) -> Result<(), TargetProfileBuildError> {
        for declaration in &self.quantitative {
            declaration.validate()?;
            if self
                .queries
                .iter()
                .any(|query| query.axis == declaration.axis)
            {
                return Err(
                    TargetProfileBuildError::ConflictingQuantitativeFactAndQuery {
                        axis: declaration.axis.key(),
                    },
                );
            }
            if self
                .quantitative
                .iter()
                .filter(|candidate| {
                    candidate.axis == declaration.axis
                        && candidate.source.phase() == declaration.source.phase()
                })
                .count()
                > 1
            {
                return Err(TargetProfileBuildError::DuplicateQuantitativeCapability {
                    axis: declaration.axis.key(),
                    phase: declaration.source.phase(),
                });
            }
        }
        for declaration in &self.queries {
            if declaration.axis == CapabilityAxis::WorkgroupThreads
                && declaration.query.available_at() != AvailabilityPhase::PreparedKernelPreflight
            {
                return Err(TargetProfileBuildError::InvalidQuantitativeQueryPhase {
                    axis: declaration.axis.key(),
                    required: AvailabilityPhase::PreparedKernelPreflight,
                    actual: declaration.query.available_at(),
                });
            }
            if self
                .queries
                .iter()
                .filter(|candidate| candidate.axis == declaration.axis)
                .count()
                > 1
            {
                return Err(TargetProfileBuildError::DuplicateQuantitativeQuery {
                    axis: declaration.axis.key(),
                });
            }
        }
        for declaration in &self.scalar {
            declaration.validate()?;
            if self
                .scalar
                .iter()
                .filter(|candidate| {
                    candidate.dimension == declaration.dimension
                        && candidate.subject == declaration.subject
                        && candidate.behaviour == declaration.behaviour
                        && candidate.source.phase() == declaration.source.phase()
                })
                .count()
                > 1
            {
                return Err(TargetProfileBuildError::DuplicateScalarDeclaration);
            }
        }
        for fact in &self.dispatchability {
            fact.validate()?;
            if self
                .dispatchability
                .iter()
                .filter(|candidate| {
                    candidate.resolved_type == fact.resolved_type
                        && candidate.source.phase() == fact.source.phase()
                })
                .count()
                > 1
            {
                return Err(TargetProfileBuildError::DuplicateDispatchability);
            }
        }
        Ok(())
    }

    fn canonicalize(&mut self) {
        self.quantitative
            .sort_by_key(|declaration| (declaration.axis, declaration.source.phase()));
        self.queries.sort_by_key(|declaration| declaration.axis);
        self.scalar.sort_by_cached_key(|declaration| {
            let mut bytes = Vec::new();
            declaration.encode(&mut bytes);
            bytes
        });
        self.dispatchability.sort_by(|left, right| {
            left.resolved_type
                .cmp(&right.resolved_type)
                .then(left.source.phase().cmp(&right.source.phase()))
        });
    }

    fn freeze(mut self) -> Result<TargetProfile, TargetProfileBuildError> {
        self.validate_declarations()?;
        self.canonicalize();
        let identity = TargetProfileIdentity::from_key(self.key.clone());
        let numerical: Vec<_> = self
            .scalar
            .iter()
            .map(ScalarHonourabilityDeclaration::declared)
            .collect();
        let fact = |declaration: &QuantitativeCapabilityDeclaration| {
            CapabilityFact::new(
                declaration.axis,
                declaration.bound,
                declaration.source.phase(),
                declaration.source.authority(),
                declaration.source.validity(),
                FactProvenance::declared_by(identity.clone()),
            )
        };
        let honourability = numerical
            .iter()
            .map(|declared| declared.attributed_to(identity.clone()))
            .collect();
        let checked = CheckedTargetProfile::new_with_queries(
            identity.clone(),
            self.quantitative.iter().map(fact).collect(),
            self.queries
                .iter()
                .map(|declaration| {
                    CapabilityQuery::new(declaration.axis, declaration.query.clone())
                })
                .collect(),
            honourability,
        )
        .map_err(TargetProfileBuildError::from)?;

        let descriptor = complete_descriptor(
            &self.key,
            &self.quantitative,
            &self.queries,
            &self.scalar,
            &self.dispatchability,
        );
        if descriptor.len() > MAX_TARGET_PROFILE_DESCRIPTOR_BYTES {
            return Err(TargetProfileBuildError::DescriptorTooLong {
                actual: descriptor.len(),
                max: MAX_TARGET_PROFILE_DESCRIPTOR_BYTES,
            });
        }
        let Self {
            key,
            quantitative,
            queries: _,
            scalar,
            dispatchability,
        } = self;
        Ok(TargetProfile {
            data: Arc::new(TargetProfileData {
                key,
                checked,
                quantitative: quantitative.into_boxed_slice(),
                scalar: scalar.into_boxed_slice(),
                dispatchability: dispatchability.into_boxed_slice(),
                descriptor: descriptor.into_boxed_slice(),
            }),
        })
    }

    #[cfg(test)]
    fn try_build(self) -> Result<TargetProfile, TargetProfileBuildError> {
        self.build()
    }
}

/// Ordered, nonempty, unique target set for one compilation request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetRequest {
    profiles: Vec<TargetProfile>,
}

impl TargetRequest {
    /// Validates the target set without reordering it.
    ///
    /// # Errors
    ///
    /// Returns [`TargetRequestError::Empty`],
    /// [`TargetRequestError::TooManyProfiles`], or
    /// [`TargetRequestError::DuplicateProfile`].
    pub fn new(
        profiles: impl IntoIterator<Item = TargetProfile>,
    ) -> Result<Self, TargetRequestError> {
        let profiles: Vec<_> = profiles
            .into_iter()
            .take(MAX_TARGET_PROFILES_PER_REQUEST + 1)
            .collect();
        if profiles.is_empty() {
            return Err(TargetRequestError::Empty);
        }
        if profiles.len() > MAX_TARGET_PROFILES_PER_REQUEST {
            return Err(TargetRequestError::TooManyProfiles {
                actual: profiles.len(),
                max: MAX_TARGET_PROFILES_PER_REQUEST,
            });
        }
        let mut keys: Vec<_> = profiles
            .iter()
            .enumerate()
            .map(|(index, profile)| (profile.profile_key(), index))
            .collect();
        keys.sort_unstable_by(|left, right| left.0.cmp(right.0).then(left.1.cmp(&right.1)));
        if let Some(pair) = keys.windows(2).find(|pair| pair[0].0 == pair[1].0) {
            let (first, duplicate) = if pair[0].1 < pair[1].1 {
                (pair[0].1, pair[1].1)
            } else {
                (pair[1].1, pair[0].1)
            };
            return Err(TargetRequestError::DuplicateProfile {
                profile: pair[0].0.clone(),
                first,
                duplicate,
            });
        }
        Ok(Self { profiles })
    }

    /// Returns the profiles in caller-declared result order.
    #[must_use]
    pub fn profiles(&self) -> &[TargetProfile] {
        &self.profiles
    }

    pub(crate) fn into_profiles(self) -> Vec<TargetProfile> {
        self.profiles
    }
}

/// Typed target-set construction failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetRequestError {
    /// No target was stated.
    Empty,
    /// The target set exceeded its admitted cardinality.
    TooManyProfiles {
        /// Observed cardinality, capped at `max + 1`.
        actual: usize,
        /// Maximum admitted cardinality.
        max: usize,
    },
    /// Two profiles have the same validated profile key.
    DuplicateProfile {
        /// The duplicated profile key.
        profile: TargetProfileKey,
        /// Zero-based position of the first occurrence.
        first: usize,
        /// Zero-based position of the duplicate occurrence.
        duplicate: usize,
    },
}

impl std::fmt::Display for TargetRequestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for TargetRequestError {}

/// Public scalar-declaration disposition.
///
/// Exact emulation is intentionally absent: only the compiler can verify that
/// emitted replacement operations prove an exact emulation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScalarSupport {
    /// The target directly honours the stated behaviour.
    Exact,
    /// The target explicitly cannot honour the stated behaviour.
    Unsupported,
}

impl ScalarSupport {
    const fn means(self) -> HonouringMeans {
        match self {
            Self::Exact => HonouringMeans::SupportedExactly,
            Self::Unsupported => HonouringMeans::Unsupported,
        }
    }
}

fn complete_descriptor(
    key: &TargetProfileKey,
    quantitative: &[QuantitativeCapabilityDeclaration],
    queries: &[QuantitativeCapabilityQueryDeclaration],
    scalar: &[ScalarHonourabilityDeclaration],
    dispatchability: &[DTypeDispatchabilityFact],
) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_slice(&mut bytes, COMPLETE_PROFILE_DESCRIPTOR_DOMAIN);
    push_slice(&mut bytes, key.as_str().as_bytes());
    push_slice(&mut bytes, PROFILE_SOURCE_DOMAIN);
    let mut sources: Vec<_> = quantitative
        .iter()
        .map(|fact| fact.source.as_ref())
        .chain(scalar.iter().map(|declaration| declaration.source.as_ref()))
        .chain(dispatchability.iter().map(|fact| fact.source.as_ref()))
        .map(|source| (source.canonical_bytes(), source))
        .collect();
    sources.sort_by(|left, right| left.0.cmp(&right.0));
    sources.dedup_by(|left, right| left.0 == right.0);
    push_len(&mut bytes, sources.len());
    for (source_bytes, _) in &sources {
        push_slice(&mut bytes, source_bytes);
    }
    push_len(&mut bytes, quantitative.len());
    for fact in quantitative {
        push_slice(&mut bytes, fact.axis.key().as_bytes());
        bytes.extend_from_slice(&fact.bound.to_le_bytes());
        let source_bytes = fact.source.canonical_bytes();
        let source_index = sources
            .binary_search_by(|candidate| candidate.0.cmp(&source_bytes))
            .expect("every quantitative source was inserted into the source table");
        QuantitativeCapabilityDeclaration::encode_source_index(&mut bytes, source_index);
    }
    push_len(&mut bytes, queries.len());
    for query in queries {
        push_slice(&mut bytes, query.axis.key().as_bytes());
        push_slice(&mut bytes, &query.query.canonical_bytes());
    }
    let mut subjects = scalar
        .iter()
        .map(|declaration| {
            let mut subject = Vec::new();
            declaration.subject.encode(&mut subject);
            subject
        })
        .collect::<Vec<_>>();
    subjects.sort();
    subjects.dedup();
    push_len(&mut bytes, subjects.len());
    for subject in &subjects {
        push_slice(&mut bytes, subject);
    }
    let mut scalar_rows = Vec::with_capacity(scalar.len());
    for declaration in scalar {
        let mut subject = Vec::new();
        declaration.subject.encode(&mut subject);
        let subject_index = subjects
            .binary_search(&subject)
            .expect("every numerical subject was inserted into the subject table");
        let source_bytes = declaration.source.canonical_bytes();
        let source_index = sources
            .binary_search_by(|candidate| candidate.0.cmp(&source_bytes))
            .expect("every numerical source was inserted into the source table");
        let mut row = Vec::new();
        encode_compact_index(&mut row, subject_index);
        row.push(declaration.dimension.tag());
        declaration.behaviour.encode(&mut row);
        declaration.means.encode(&mut row);
        encode_compact_index(&mut row, source_index);
        scalar_rows.push(row);
    }
    scalar_rows.sort_unstable();
    push_len(&mut bytes, scalar_rows.len());
    for row in scalar_rows {
        bytes.extend_from_slice(&row);
    }
    push_slice(&mut bytes, DISPATCHABILITY_DOMAIN);
    push_len(&mut bytes, dispatchability.len());
    for fact in dispatchability {
        let source_bytes = fact.source.canonical_bytes();
        let source_index = sources
            .binary_search_by(|candidate| candidate.0.cmp(&source_bytes))
            .expect("every dispatch source was inserted into the source table");
        fact.encode(&mut bytes, source_index);
    }
    bytes
}

impl TargetProfile {
    /// Returns the compiler-governed bounded prototype profile.
    ///
    /// # Panics
    ///
    /// Panics only if this compiler build's own governed declaration violates
    /// its construction invariants.
    pub fn governed() -> Self {
        static GOVERNED: OnceLock<TargetProfile> = OnceLock::new();
        GOVERNED
            .get_or_init(|| {
                TargetProfileBuilder::governed()
                    .build()
                    .expect("the governed target profile is intrinsically valid")
            })
            .clone()
    }

    /// Returns this profile's owned key.
    #[must_use]
    pub fn profile_key(&self) -> &TargetProfileKey {
        &self.data.key
    }

    pub(crate) fn checked(&self) -> &CheckedTargetProfile {
        &self.data.checked
    }

    /// Returns the complete canonical declaration bytes used for identity.
    #[must_use]
    pub fn canonical_descriptor(&self) -> &[u8] {
        &self.data.descriptor
    }

    pub(crate) fn request_subject_bytes(&self) -> &[u8] {
        &self.data.descriptor
    }

    /// Resolves dispatchability only for an exactly equal resolved type,
    /// preferring the latest declaration available through `available_phase`.
    #[must_use]
    pub fn dtype_dispatchability(
        &self,
        resolved_type: &ResolvedValueType,
        available_phase: AvailabilityPhase,
    ) -> DTypeDispatchabilityResolution {
        let mut now = None;
        let mut later = None;
        for fact in self
            .data
            .dispatchability
            .iter()
            .filter(|fact| &fact.resolved_type == resolved_type)
        {
            let phase = fact.source.phase();
            if phase <= available_phase {
                now = Some(fact.verdict);
            } else if later.is_none() {
                later = Some(phase);
            }
        }
        match (now, later) {
            (Some(DTypeDispatchability::Dispatchable), _) => {
                DTypeDispatchabilityResolution::Dispatchable
            }
            (Some(DTypeDispatchability::Unsupported), _) => {
                DTypeDispatchabilityResolution::Unsupported
            }
            (None, Some(available_at)) => DTypeDispatchabilityResolution::Deferred { available_at },
            (None, None) => DTypeDispatchabilityResolution::Unknown,
        }
    }

    #[cfg(test)]
    pub(crate) fn governed_without_numerical_declarations() -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.scalar.clear();
        builder
            .build()
            .expect("the sparse test profile is intrinsically valid")
    }

    #[cfg(test)]
    pub(crate) fn governed_with_grid_axis_limit(limit: u64) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder
            .quantitative
            .iter_mut()
            .find(|declaration| declaration.axis == CapabilityAxis::GridAxisThreads)
            .expect("the governed profile declares the grid-axis limit")
            .bound = limit;
        builder
            .build()
            .expect("the test target profile is intrinsically valid")
    }

    #[cfg(test)]
    pub(crate) fn governed_with_key_for_test(key: &str) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::declared(key.to_owned())
            .expect("the test target-profile key is valid");
        builder
            .build()
            .expect("the keyed test target profile is intrinsically valid")
    }

    #[cfg(test)]
    pub(crate) fn without_numerical_declarations_for_test(key: &str) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::declared(key.to_owned())
            .expect("the test target-profile key is valid");
        builder.scalar.clear();
        builder
            .build()
            .expect("the sparse keyed test profile is intrinsically valid")
    }

    #[cfg(test)]
    pub(crate) fn flush_only_for_test(key: &str) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::declared(key.to_owned())
            .expect("the test target-profile key is valid");
        builder.scalar.retain(|declaration| {
            !matches!(
                declaration.dimension,
                NumericalDimension::InputSubnormals | NumericalDimension::ResultSubnormals
            ) || matches!(
                declaration.behaviour,
                DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
                    zero_sign: FlushedZeroSign::PreservesSign,
                })
            )
        });
        builder
            .build()
            .expect("the flush-only test profile is intrinsically valid")
    }

    #[cfg(test)]
    pub(crate) fn with_grid_axis_limit_for_test(key: &str, limit: u64) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::declared(key.to_owned())
            .expect("the test target-profile key is valid");
        builder
            .quantitative
            .iter_mut()
            .find(|declaration| declaration.axis == CapabilityAxis::GridAxisThreads)
            .expect("the governed profile declares the grid-axis limit")
            .bound = limit;
        builder
            .build()
            .expect("the bounded keyed test profile is intrinsically valid")
    }

    /// The governed profile with preserved input subnormals declared
    /// unsupported under `source`.
    ///
    /// A strict contract is then refused by a *named* authority rather than
    /// merely undeclared, which is the only shape that carries provenance: an
    /// undeclared dimension has no fact to cite. Varying `source` alone varies
    /// exactly the evidence and nothing about what was required.
    #[cfg(test)]
    pub(crate) fn refusing_preserved_subnormals_for_test(
        key: &str,
        source: Arc<FactSourceProvenance>,
    ) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::declared(key.to_owned())
            .expect("the test target-profile key is valid");
        let preserve = DimensionBehaviour::Subnormals(SubnormalMode::Preserve);
        let declaration = builder
            .scalar
            .iter_mut()
            .find(|declaration| {
                declaration.dimension == NumericalDimension::InputSubnormals
                    && declaration.behaviour == preserve
            })
            .expect("the governed profile declares preserved input subnormals");
        declaration.means = HonouringMeans::Unsupported;
        declaration.source = source;
        builder
            .build()
            .expect("the refusing keyed test profile is intrinsically valid")
    }

    #[cfg(test)]
    pub(crate) fn governed_declared_behaviours() -> Vec<DeclaredBehaviour> {
        TargetProfileBuilder::governed()
            .scalar
            .iter()
            .map(ScalarHonourabilityDeclaration::declared)
            .collect()
    }
}

fn governed_target_honourability() -> Vec<ScalarHonourabilityDeclaration> {
    let exact =
        |dimension, behaviour| ScalarHonourabilityDeclaration::governed_exact(dimension, behaviour);
    vec![
        exact(
            NumericalDimension::InputSubnormals,
            DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
        ),
        exact(
            NumericalDimension::InputSubnormals,
            DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            }),
        ),
        exact(
            NumericalDimension::ResultSubnormals,
            DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
        ),
        exact(
            NumericalDimension::ResultSubnormals,
            DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            }),
        ),
        exact(
            NumericalDimension::Contraction,
            DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        ),
        exact(
            NumericalDimension::Contraction,
            DimensionBehaviour::Transform(NumericalPermission::Permitted),
        ),
        exact(
            NumericalDimension::Reassociation,
            DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        ),
        exact(
            NumericalDimension::Reassociation,
            DimensionBehaviour::Transform(NumericalPermission::Permitted),
        ),
        exact(
            NumericalDimension::Permutation,
            DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        ),
        exact(
            NumericalDimension::SignedZero,
            DimensionBehaviour::Transform(NumericalPermission::Forbidden),
        ),
        exact(
            NumericalDimension::NanAssumptions,
            DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
        ),
        exact(
            NumericalDimension::InfinityAssumptions,
            DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption),
        ),
    ]
}

/// Typed target-profile construction diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetProfileBuildError {
    /// A numerical dimension was paired with another dimension's behaviour space.
    InvalidDimensionBehaviour,
    /// A declared relaxation did not match the subject and dimension.
    InvalidRelaxation,
    /// No semantic authority registered the arithmetic/resolved-type association.
    UnvalidatedScalarArithmetic,
    /// A caller attempted to assert compiler-proved exact emulation.
    UnverifiedExactEmulation,
    /// Structured producer attribution was incomplete or incoherent.
    InvalidProducerClaim,
    /// The same quantitative capability axis was declared twice at one phase.
    DuplicateQuantitativeCapability {
        /// Stable governed axis key.
        axis: &'static str,
        /// Availability phase at which both rows claimed authority.
        phase: AvailabilityPhase,
    },
    /// A quantitative query was declared for an availability phase that cannot
    /// answer that axis's exact requirement.
    InvalidQuantitativeQueryPhase {
        /// Stable governed axis key.
        axis: &'static str,
        /// Earliest phase that can answer this axis correctly.
        required: AvailabilityPhase,
        /// Phase the rejected query declared.
        actual: AvailabilityPhase,
    },
    /// The same quantitative capability axis received two query schemas.
    DuplicateQuantitativeQuery {
        /// Stable governed axis key.
        axis: &'static str,
    },
    /// One axis cannot carry both an available fact and a deferred query.
    ConflictingQuantitativeFactAndQuery {
        /// Stable governed axis key.
        axis: &'static str,
    },
    /// The same numerical behaviour was declared twice at the same phase.
    DuplicateScalarDeclaration,
    /// A complete measured subnormal table would overlap an existing row.
    ConflictingSubnormalDeclaration {
        /// Exact scalar subject whose table was already partially declared.
        subject: Box<ScalarArithmetic>,
        /// Stable numerical-dimension key of the conflicting row.
        dimension: &'static str,
        /// Availability phase of the conflicting row.
        phase: AvailabilityPhase,
    },
    /// The same exact resolved type received more than one dispatch verdict at
    /// one availability phase.
    DuplicateDispatchability,
    /// The canonical descriptor exceeded the artifact identity bound.
    DescriptorTooLong {
        /// Encoded byte length.
        actual: usize,
        /// Maximum admitted encoded byte length.
        max: usize,
    },
    /// The quantitative feasibility profile was malformed.
    MalformedProfile {
        /// Stable refusing rule.
        rule: &'static str,
    },
}

impl std::fmt::Display for TargetProfileBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for TargetProfileBuildError {}

impl From<FeasibilityError> for TargetProfileBuildError {
    fn from(value: FeasibilityError) -> Self {
        match value {
            FeasibilityError::MalformedProfile { rule } => Self::MalformedProfile { rule },
            FeasibilityError::DescriptorTooLong { actual, .. } => Self::DescriptorTooLong {
                actual,
                max: MAX_TARGET_PROFILE_DESCRIPTOR_BYTES,
            },
            FeasibilityError::MalformedProposal { .. } => Self::MalformedProfile {
                rule: "unexpected-proposal-validation",
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feasibility::{
        AvailabilityPhase, AxisRequirement, FactAuthority, FactValidityScope, FeasibilityOutcome,
        FeasibilityProposal,
    };
    use crate::honourability::{
        CompilerBuildIdentity, CompilerBuildRole, ExecutionEnvironmentIdentity, MeasurementContext,
        NumericalRequirement, ProvenanceIdentity, RelaxationRequirement,
    };
    use tiler_ir::semantic::{CanonicalValue, TypeArguments, TypeKey};

    fn nominal(name: impl AsRef<str>) -> ResolvedValueType {
        ResolvedValueType::nominal(TypeKey::new("test", name, 1).unwrap())
    }

    fn dispatch_fact(
        resolved_type: ResolvedValueType,
        verdict: DTypeDispatchability,
    ) -> DTypeDispatchabilityFact {
        DTypeDispatchabilityFact {
            resolved_type,
            verdict,
            source: governed_profile_source(),
        }
    }

    fn measured_capability_source() -> Arc<FactSourceProvenance> {
        Arc::new(FactSourceProvenance::measured(
            AvailabilityPhase::LiveDevicePreflight,
            FactAuthority::DeviceRuntime,
            FactValidityScope::DeviceInstance,
            ProvenanceIdentity::new("test.capability-producer.v1", 1),
            vec![MeasurementContext::new(
                vec![CompilerBuildIdentity::new(
                    CompilerBuildRole::RuntimeCompiler,
                    "test-compiler",
                    "1.0",
                    None,
                )],
                ExecutionEnvironmentIdentity::new(
                    "test-platform",
                    "1.0",
                    "build-1",
                    "test-architecture",
                    "test-hardware",
                ),
            )],
        ))
    }

    fn public_external_source(reference_revision: u32) -> TargetFactSource {
        TargetFactSource::external_guarantee(
            TargetFactProducerIdentity::new("test.external-profile-producer.v1".to_owned(), 1)
                .unwrap(),
            TargetNormativeReferenceIdentity::new(
                "test.external-profile-specification.v1".to_owned(),
                reference_revision,
            )
            .unwrap(),
        )
    }

    fn public_builder(key: &str) -> TargetProfileBuilder {
        let source = public_external_source(1);
        let mut builder = TargetProfileBuilder::new(TargetProfileKey::new(key.to_owned()).unwrap());
        builder
            .declare_max_threads_per_grid_axis(65_535, source.clone())
            .unwrap();
        builder
            .declare_max_threads_per_workgroup(256, source.clone())
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
        builder
    }

    fn compile_profile_measurement_source(
        compiler_version: &str,
        platform_build: &str,
    ) -> TargetCompileProfileMeasurementSource {
        compile_profile_measurement_source_with(1, compiler_version, platform_build)
    }

    fn compile_profile_measurement_source_with(
        producer_revision: u32,
        compiler_version: &str,
        platform_build: &str,
    ) -> TargetCompileProfileMeasurementSource {
        let compiler = TargetCompilerBuild::new(
            TargetCompilerRole::CodeGenerator,
            "test-code-generator".to_owned(),
            compiler_version.to_owned(),
            Some("exact-build".to_owned()),
        )
        .unwrap();
        let environment = TargetExecutionEnvironment::builder()
            .platform("test-platform".to_owned())
            .platform_version("1.0".to_owned())
            .platform_build(platform_build.to_owned())
            .architecture("test-architecture".to_owned())
            .hardware("test-hardware".to_owned())
            .build()
            .unwrap();
        let context = TargetMeasurementContext::new([compiler], environment).unwrap();
        TargetCompileProfileMeasurementSource::new(
            TargetFactProducerIdentity::new(
                "test.compile-profile-measurement.v1".to_owned(),
                producer_revision,
            )
            .unwrap(),
            [context],
        )
        .unwrap()
    }

    #[test]
    fn caller_profile_keys_are_owned_and_validated() {
        let source = String::from("acme.family-a.v1");
        let key = TargetProfileKey::declared(source.clone()).unwrap();
        drop(source);
        assert_eq!(key.as_str(), "acme.family-a.v1");
        assert_eq!(
            TargetProfileKey::declared(String::new()),
            Err(TargetProfileKeyError::Empty)
        );
        assert_eq!(
            TargetProfileKey::declared("Acme family".to_owned()),
            Err(TargetProfileKeyError::InvalidByte {
                index: 0,
                value: b'A',
            })
        );
        assert_eq!(
            TargetProfileKey::declared("a".repeat(MAX_TARGET_PROFILE_KEY_BYTES + 1)),
            Err(TargetProfileKeyError::TooLong {
                actual: MAX_TARGET_PROFILE_KEY_BYTES + 1,
                max: MAX_TARGET_PROFILE_KEY_BYTES,
            })
        );
    }

    #[test]
    fn scalar_subject_admits_only_the_registered_f32_association() {
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::F32, nominal("u4")),
            Err(TargetProfileBuildError::UnvalidatedScalarArithmetic)
        );
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::F16, nominal("f16")),
            Err(TargetProfileBuildError::UnvalidatedScalarArithmetic)
        );
    }

    #[test]
    fn scalar_declarations_reject_invalid_behaviour_relaxation_and_exact_emulation() {
        let subject = ScalarArithmetic::governed_f32();
        let base = |dimension, behaviour, means| ScalarHonourabilityDeclaration {
            subject: subject.clone(),
            dimension,
            behaviour,
            means,
            source: governed_profile_source(),
        };
        assert_eq!(
            base(
                NumericalDimension::Contraction,
                DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
                HonouringMeans::SupportedExactly,
            )
            .validate(),
            Err(TargetProfileBuildError::InvalidDimensionBehaviour)
        );
        assert_eq!(
            base(
                NumericalDimension::InputSubnormals,
                DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
                HonouringMeans::SupportedOnlyUnderDeclaredRelaxation {
                    relaxation: RelaxationRequirement::new(
                        NumericalDimension::Contraction,
                        ArithmeticType::F64,
                        nominal("future-f64"),
                        DimensionBehaviour::Transform(NumericalPermission::Permitted),
                    ),
                },
            )
            .validate(),
            Err(TargetProfileBuildError::InvalidRelaxation)
        );
        assert_eq!(
            base(
                NumericalDimension::InputSubnormals,
                DimensionBehaviour::Subnormals(SubnormalMode::Preserve),
                HonouringMeans::SupportedWithExactEmulation,
            )
            .validate(),
            Err(TargetProfileBuildError::UnverifiedExactEmulation)
        );
    }

    #[test]
    fn scalar_duplicate_detection_compares_the_complete_subject() {
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.scalar-subject.v1".to_owned()).unwrap(),
        );
        let source = public_external_source(1);
        for resolved_type in [F32::resolved_type(), nominal("future-bf16-subject")] {
            builder.scalar.push(ScalarHonourabilityDeclaration {
                subject: ScalarArithmetic {
                    arithmetic: ArithmeticType::F32,
                    resolved_type,
                },
                dimension: NumericalDimension::Contraction,
                behaviour: DimensionBehaviour::Transform(NumericalPermission::Forbidden),
                means: HonouringMeans::SupportedExactly,
                source: Arc::clone(&source.0),
            });
        }
        assert_eq!(builder.validate_declarations(), Ok(()));
    }

    #[test]
    fn malformed_structured_producer_attribution_is_rejected() {
        let mut builder = TargetProfileBuilder::governed();
        builder.quantitative[0].source = Arc::new(FactSourceProvenance::measured(
            AvailabilityPhase::LiveDevicePreflight,
            FactAuthority::DeviceRuntime,
            FactValidityScope::DeviceInstance,
            ProvenanceIdentity::new("test.empty-measurement.v1", 1),
            Vec::new(),
        ));
        assert_eq!(
            builder.try_build(),
            Err(TargetProfileBuildError::InvalidProducerClaim)
        );

        let mut builder = TargetProfileBuilder::governed();
        builder.scalar[0].source = Arc::new(FactSourceProvenance::measured(
            AvailabilityPhase::LiveDevicePreflight,
            FactAuthority::DeviceRuntime,
            FactValidityScope::DeviceInstance,
            ProvenanceIdentity::new("test.empty-scalar-measurement.v1", 1),
            Vec::new(),
        ));
        assert_eq!(
            builder.try_build(),
            Err(TargetProfileBuildError::InvalidProducerClaim)
        );

        let mut builder = TargetProfileBuilder::governed();
        builder.dispatchability = vec![DTypeDispatchabilityFact {
            resolved_type: F32::resolved_type(),
            verdict: DTypeDispatchability::Dispatchable,
            source: Arc::new(FactSourceProvenance::measured(
                AvailabilityPhase::LiveDevicePreflight,
                FactAuthority::DeviceRuntime,
                FactValidityScope::DeviceInstance,
                ProvenanceIdentity::new("test.empty-dispatch-measurement.v1", 1),
                Vec::new(),
            )),
        }];
        assert_eq!(
            builder.try_build(),
            Err(TargetProfileBuildError::InvalidProducerClaim)
        );
    }

    #[test]
    fn external_guarantees_are_not_compiler_governed_or_measurements() {
        let external = public_external_source(1);
        assert_eq!(external.0.authority(), FactAuthority::ExternalProfile);
        assert!(matches!(
            external.0.basis(),
            crate::honourability::FactEvidenceBasis::ExternalGuarantee { .. }
        ));
        assert_ne!(
            external.0.canonical_bytes(),
            governed_profile_source().canonical_bytes()
        );

        let first = public_builder("test.external-a.v1").build().unwrap();
        let mut second = TargetProfileBuilder::new(
            TargetProfileKey::new("test.external-a.v1".to_owned()).unwrap(),
        );
        let revised_source = public_external_source(2);
        second
            .declare_max_threads_per_grid_axis(65_535, revised_source.clone())
            .unwrap();
        second
            .declare_max_threads_per_workgroup(256, revised_source.clone())
            .unwrap();
        second
            .declare_max_buffer_bindings_per_entry(31, revised_source.clone())
            .unwrap();
        second
            .declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, revised_source.clone())
            .unwrap();
        second
            .declare_device_address_width(DeviceAddressWidth::Bits64, revised_source.clone())
            .unwrap();
        second
            .declare_device_memory(true, revised_source.clone())
            .unwrap();
        second
            .declare_local_memory_bytes(32_768, revised_source.clone())
            .unwrap();
        let second = second.build().unwrap();
        assert_ne!(
            first.canonical_descriptor(),
            second.canonical_descriptor(),
            "the normative reference revision is identity-bearing"
        );
    }

    #[test]
    fn measured_authorities_derive_the_only_valid_phase_and_scope() {
        let compiler = TargetCompilerBuild::new(
            TargetCompilerRole::RuntimeCompiler,
            "test-compiler".to_owned(),
            "1.0".to_owned(),
            None,
        )
        .unwrap();
        let environment = TargetExecutionEnvironment::builder()
            .platform("test-platform".to_owned())
            .platform_version("1.0".to_owned())
            .platform_build("build-1".to_owned())
            .architecture("test-architecture".to_owned())
            .hardware("test-hardware".to_owned())
            .build()
            .unwrap();
        let context = TargetMeasurementContext::new([compiler], environment).unwrap();
        for (index, (authority, phase, internal_authority, validity)) in [
            (
                MeasuredFactAuthority::ArtifactEvidence,
                AvailabilityPhase::ArtifactEvidence,
                FactAuthority::ArtifactEvidence,
                FactValidityScope::PreparedArtifact,
            ),
            (
                MeasuredFactAuthority::DeviceRuntime,
                AvailabilityPhase::LiveDevicePreflight,
                FactAuthority::DeviceRuntime,
                FactValidityScope::DeviceInstance,
            ),
            (
                MeasuredFactAuthority::PreparedKernel,
                AvailabilityPhase::PreparedKernelPreflight,
                FactAuthority::PreparedKernel,
                FactValidityScope::PreparedArtifact,
            ),
            (
                MeasuredFactAuthority::LaunchInstance,
                AvailabilityPhase::LaunchPreflight,
                FactAuthority::LaunchInstance,
                FactValidityScope::LaunchInstance,
            ),
        ]
        .into_iter()
        .enumerate()
        {
            let producer =
                TargetFactProducerIdentity::new(format!("test.measurement-producer-{index}.v1"), 1)
                    .unwrap();
            let source =
                TargetFactSource::measured(producer, authority, [context.clone()]).unwrap();
            assert_eq!(source.0.phase(), phase);
            assert_eq!(source.0.authority(), internal_authority);
            assert_eq!(source.0.validity(), validity);
        }
    }

    #[test]
    fn compiler_profile_measurement_source_fixes_empirical_authority_and_scope() {
        let source = compile_profile_measurement_source("1.0", "build-1");
        assert_eq!(source.0.phase(), AvailabilityPhase::CompileProfile);
        assert_eq!(source.0.authority(), FactAuthority::MeasuredProfile);
        assert_eq!(source.0.validity(), FactValidityScope::MeasuredEnvironment);
        assert!(matches!(
            source.0.basis(),
            crate::honourability::FactEvidenceBasis::Measurement { contexts }
                if contexts.len() == 1
                    && contexts[0].compiler_builds()[0].version() == "1.0"
                    && contexts[0].environment().platform_build() == "build-1"
        ));
    }

    #[test]
    fn compiler_profile_measurement_source_reaches_every_profile_fact_family() {
        let source = compile_profile_measurement_source("1.0", "build-1");
        let subject = ScalarArithmetic::f32();
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.measured-all-families.v1".to_owned()).unwrap(),
        );
        builder
            .declare_measured_max_threads_per_grid_axis(65_535, source.clone())
            .unwrap();
        builder
            .declare_measured_max_threads_per_workgroup(256, source.clone())
            .unwrap();
        builder
            .declare_measured_max_buffer_bindings_per_entry(31, source.clone())
            .unwrap();
        builder
            .declare_measured_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
            .unwrap();
        builder
            .declare_measured_device_address_width(DeviceAddressWidth::Bits64, source.clone())
            .unwrap();
        builder
            .declare_measured_device_memory(true, source.clone())
            .unwrap();
        builder
            .declare_measured_local_memory_bytes(32_768, source.clone())
            .unwrap();
        builder
            .declare_measured_input_subnormal_behaviour(
                subject.clone(),
                SubnormalMode::Preserve,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_result_subnormal_behaviour(
                subject.clone(),
                SubnormalMode::Preserve,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_contraction(
                subject.clone(),
                NumericalPermission::Forbidden,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_reassociation(
                subject.clone(),
                NumericalPermission::Forbidden,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_permutation(
                subject.clone(),
                NumericalPermission::Forbidden,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_signed_zero(
                subject.clone(),
                NumericalPermission::Forbidden,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_reciprocal_transform(
                subject.clone(),
                NumericalPermission::Forbidden,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_approximate_intrinsics(
                subject.clone(),
                tiler_ir::schedule::ApproximationEnvelope::Forbidden,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_nan_assumptions(
                subject.clone(),
                ExceptionalValueAssumption::MakeNoAssumption,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_infinity_assumptions(
                subject.clone(),
                ExceptionalValueAssumption::MakeNoAssumption,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_materialization_rounding(
                subject,
                tiler_ir::schedule::MaterializationRounding::NearestTiesToEven,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_measured_dtype_dispatchability(
                F32::resolved_type(),
                DTypeDispatchability::Dispatchable,
                source,
            )
            .unwrap();

        assert_eq!(builder.quantitative.len(), 7);
        assert_eq!(builder.scalar.len(), 15);
        assert_eq!(builder.dispatchability.len(), 1);
        for provenance in builder
            .quantitative
            .iter()
            .map(|declaration| declaration.source.as_ref())
            .chain(
                builder
                    .scalar
                    .iter()
                    .map(|declaration| declaration.source.as_ref()),
            )
            .chain(
                builder
                    .dispatchability
                    .iter()
                    .map(|declaration| declaration.source.as_ref()),
            )
        {
            assert_eq!(provenance.phase(), AvailabilityPhase::CompileProfile);
            assert_eq!(provenance.authority(), FactAuthority::MeasuredProfile);
            assert_eq!(
                provenance.validity(),
                FactValidityScope::MeasuredEnvironment
            );
        }
        builder.build().unwrap();
    }

    #[test]
    fn measured_profile_declarations_reject_conflicts_without_partial_insertion() {
        let source = || compile_profile_measurement_source("1.0", "build-1");
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.measured-conflicts.v1".to_owned()).unwrap(),
        );
        builder
            .declare_measured_max_threads_per_workgroup(256, source())
            .unwrap();
        let quantitative = builder.quantitative.clone();
        assert_eq!(
            builder.declare_measured_max_threads_per_workgroup(128, source()),
            Err(TargetProfileBuildError::DuplicateQuantitativeCapability {
                axis: "threads-per-workgroup",
                phase: AvailabilityPhase::CompileProfile,
            })
        );
        assert_eq!(builder.quantitative, quantitative);

        builder
            .declare_measured_contraction(
                ScalarArithmetic::f32(),
                NumericalPermission::Forbidden,
                ScalarSupport::Exact,
                source(),
            )
            .unwrap();
        let scalar = builder.scalar.clone();
        assert_eq!(
            builder.declare_measured_contraction(
                ScalarArithmetic::f32(),
                NumericalPermission::Forbidden,
                ScalarSupport::Unsupported,
                source(),
            ),
            Err(TargetProfileBuildError::DuplicateScalarDeclaration)
        );
        assert_eq!(builder.scalar, scalar);

        builder
            .declare_measured_dtype_dispatchability(
                F32::resolved_type(),
                DTypeDispatchability::Dispatchable,
                source(),
            )
            .unwrap();
        let dispatchability = builder.dispatchability.clone();
        assert_eq!(
            builder.declare_measured_dtype_dispatchability(
                F32::resolved_type(),
                DTypeDispatchability::Unsupported,
                source(),
            ),
            Err(TargetProfileBuildError::DuplicateDispatchability)
        );
        assert_eq!(builder.dispatchability, dispatchability);
    }

    #[test]
    fn quantitative_facts_and_queries_reject_overlap_atomically_in_both_orders() {
        let query = || {
            TargetPropertyQuery::new(
                TargetPropertyKey::new("test.prepared-entry.workgroup-limit.v1").unwrap(),
                AvailabilityPhase::PreparedKernelPreflight,
                TargetPropertyProviderIdentity::new("test", "prepared-entry", 1).unwrap(),
            )
            .unwrap()
        };

        let mut fact_first = TargetProfileBuilder::new(
            TargetProfileKey::new("test.fact-first.v1".to_owned()).unwrap(),
        );
        fact_first
            .declare_max_threads_per_workgroup(256, public_external_source(1))
            .unwrap();
        assert_eq!(
            fact_first.declare_max_threads_per_workgroup_query(query()),
            Err(
                TargetProfileBuildError::ConflictingQuantitativeFactAndQuery {
                    axis: "threads-per-workgroup",
                }
            )
        );
        assert!(fact_first.queries.is_empty());

        let mut query_first = TargetProfileBuilder::new(
            TargetProfileKey::new("test.query-first.v1".to_owned()).unwrap(),
        );
        query_first
            .declare_max_threads_per_workgroup_query(query())
            .unwrap();
        assert_eq!(
            query_first.declare_max_threads_per_workgroup(256, public_external_source(1)),
            Err(
                TargetProfileBuildError::ConflictingQuantitativeFactAndQuery {
                    axis: "threads-per-workgroup",
                }
            )
        );
        assert!(query_first.quantitative.is_empty());
    }

    #[test]
    fn measured_profile_identity_binds_source_and_fact_values() {
        let descriptor =
            |producer_revision, compiler_version, platform_build, threads, verdict, support| {
                let source = compile_profile_measurement_source_with(
                    producer_revision,
                    compiler_version,
                    platform_build,
                );
                let mut builder = TargetProfileBuilder::new(
                    TargetProfileKey::new("test.measured-identity.v1".to_owned()).unwrap(),
                );
                builder
                    .declare_measured_max_threads_per_workgroup(threads, source.clone())
                    .unwrap();
                builder
                    .declare_measured_contraction(
                        ScalarArithmetic::f32(),
                        NumericalPermission::Forbidden,
                        support,
                        source.clone(),
                    )
                    .unwrap();
                builder
                    .declare_measured_dtype_dispatchability(F32::resolved_type(), verdict, source)
                    .unwrap();
                builder.build().unwrap().canonical_descriptor().to_vec()
            };
        let baseline = descriptor(
            1,
            "1.0",
            "build-1",
            256,
            DTypeDispatchability::Dispatchable,
            ScalarSupport::Exact,
        );
        for changed in [
            descriptor(
                2,
                "1.0",
                "build-1",
                256,
                DTypeDispatchability::Dispatchable,
                ScalarSupport::Exact,
            ),
            descriptor(
                1,
                "2.0",
                "build-1",
                256,
                DTypeDispatchability::Dispatchable,
                ScalarSupport::Exact,
            ),
            descriptor(
                1,
                "1.0",
                "build-2",
                256,
                DTypeDispatchability::Dispatchable,
                ScalarSupport::Exact,
            ),
            descriptor(
                1,
                "1.0",
                "build-1",
                128,
                DTypeDispatchability::Dispatchable,
                ScalarSupport::Exact,
            ),
            descriptor(
                1,
                "1.0",
                "build-1",
                256,
                DTypeDispatchability::Unsupported,
                ScalarSupport::Exact,
            ),
            descriptor(
                1,
                "1.0",
                "build-1",
                256,
                DTypeDispatchability::Dispatchable,
                ScalarSupport::Unsupported,
            ),
        ] {
            assert_ne!(baseline, changed);
        }
    }

    #[test]
    fn measured_scalar_subnormal_declarations_build_independent_exclusive_tables() {
        let behaviours = [
            SubnormalMode::Preserve,
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::PreservesSign,
            },
            SubnormalMode::FlushToZero {
                zero_sign: FlushedZeroSign::AlwaysPositive,
            },
        ];
        for delivered in behaviours {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.measured-subnormal-table.v1".to_owned()).unwrap(),
            );
            builder
                .declare_measured_input_subnormal_behaviour(
                    ScalarArithmetic::f32(),
                    delivered,
                    compile_profile_measurement_source("1.0", "build-1"),
                )
                .unwrap();
            builder
                .declare_measured_result_subnormal_behaviour(
                    ScalarArithmetic::f32(),
                    delivered,
                    compile_profile_measurement_source("1.0", "build-1"),
                )
                .unwrap();
            assert_eq!(builder.scalar.len(), 6);
            for dimension in [
                NumericalDimension::InputSubnormals,
                NumericalDimension::ResultSubnormals,
            ] {
                for behaviour in behaviours {
                    let row = builder
                        .scalar
                        .iter()
                        .find(|row| {
                            row.dimension == dimension
                                && row.behaviour == DimensionBehaviour::Subnormals(behaviour)
                        })
                        .expect("every destination row is explicit");
                    assert_eq!(
                        row.means,
                        if behaviour == delivered {
                            HonouringMeans::SupportedExactly
                        } else {
                            HonouringMeans::Unsupported
                        }
                    );
                    assert_eq!(row.source.phase(), AvailabilityPhase::CompileProfile);
                    assert_eq!(row.source.authority(), FactAuthority::MeasuredProfile);
                    assert_eq!(
                        row.source.validity(),
                        FactValidityScope::MeasuredEnvironment
                    );
                }
            }
        }
    }

    #[test]
    fn measured_scalar_subnormal_dimension_rejects_cross_phase_rows_atomically() {
        let subject = ScalarArithmetic::f32();
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.measured-subnormal-conflict.v1".to_owned()).unwrap(),
        );
        builder
            .declare_result_subnormals(
                subject.clone(),
                SubnormalMode::FlushToZero {
                    zero_sign: FlushedZeroSign::AlwaysPositive,
                },
                ScalarSupport::Unsupported,
                TargetFactSource(measured_capability_source()),
            )
            .unwrap();
        let before = builder.scalar.clone();
        assert_eq!(
            builder.declare_measured_result_subnormal_behaviour(
                subject.clone(),
                SubnormalMode::Preserve,
                compile_profile_measurement_source("1.0", "build-1"),
            ),
            Err(TargetProfileBuildError::ConflictingSubnormalDeclaration {
                subject: Box::new(subject),
                dimension: "numerics.result-subnormals",
                phase: AvailabilityPhase::LiveDevicePreflight,
            })
        );
        assert_eq!(
            builder.scalar, before,
            "refusal must insert no partial table"
        );
    }

    #[test]
    fn measured_subnormal_table_identity_binds_behaviour_build_and_environment() {
        let descriptor = |delivered, compiler_version, platform_build| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.measured-subnormal-identity.v1".to_owned()).unwrap(),
            );
            builder
                .declare_measured_input_subnormal_behaviour(
                    ScalarArithmetic::f32(),
                    delivered,
                    compile_profile_measurement_source(compiler_version, platform_build),
                )
                .unwrap();
            builder
                .declare_measured_result_subnormal_behaviour(
                    ScalarArithmetic::f32(),
                    delivered,
                    compile_profile_measurement_source(compiler_version, platform_build),
                )
                .unwrap();
            builder.build().unwrap().canonical_descriptor().to_vec()
        };
        let baseline = descriptor(SubnormalMode::Preserve, "1.0", "build-1");
        assert_ne!(
            baseline,
            descriptor(
                SubnormalMode::FlushToZero {
                    zero_sign: FlushedZeroSign::AlwaysPositive,
                },
                "1.0",
                "build-1",
            )
        );
        assert_ne!(
            baseline,
            descriptor(SubnormalMode::Preserve, "2.0", "build-1")
        );
        assert_ne!(
            baseline,
            descriptor(SubnormalMode::Preserve, "1.0", "build-2")
        );
    }

    #[test]
    fn public_provenance_bounds_stop_after_the_first_excess_item() {
        let compiler = TargetCompilerBuild::new(
            TargetCompilerRole::RuntimeCompiler,
            "test-compiler".to_owned(),
            "1.0".to_owned(),
            None,
        )
        .unwrap();
        let environment = || {
            TargetExecutionEnvironment::builder()
                .platform("test-platform".to_owned())
                .platform_version("1.0".to_owned())
                .platform_build("build-1".to_owned())
                .architecture("test-architecture".to_owned())
                .hardware("test-hardware".to_owned())
                .build()
                .unwrap()
        };
        let compiler_items = std::cell::Cell::new(0);
        let compiler_stream = std::iter::repeat_with(|| {
            compiler_items.set(compiler_items.get() + 1);
            compiler.clone()
        });
        assert_eq!(
            TargetMeasurementContext::new(compiler_stream, environment()),
            Err(TargetFactSourceError::TooManyCompilerBuilds {
                actual: MAX_TARGET_COMPILER_BUILDS_PER_CONTEXT + 1,
                max: MAX_TARGET_COMPILER_BUILDS_PER_CONTEXT,
            })
        );
        assert_eq!(
            compiler_items.get(),
            MAX_TARGET_COMPILER_BUILDS_PER_CONTEXT + 1
        );

        let context = TargetMeasurementContext::new([compiler], environment()).unwrap();
        let context_items = std::cell::Cell::new(0);
        let context_stream = std::iter::repeat_with(|| {
            context_items.set(context_items.get() + 1);
            context.clone()
        });
        assert_eq!(
            TargetFactSource::measured(
                TargetFactProducerIdentity::new("test.measurement-bound.v1".to_owned(), 1).unwrap(),
                MeasuredFactAuthority::DeviceRuntime,
                context_stream,
            ),
            Err(TargetFactSourceError::TooManyMeasurementContexts {
                actual: MAX_TARGET_MEASUREMENT_CONTEXTS_PER_SOURCE + 1,
                max: MAX_TARGET_MEASUREMENT_CONTEXTS_PER_SOURCE,
            })
        );
        assert_eq!(
            context_items.get(),
            MAX_TARGET_MEASUREMENT_CONTEXTS_PER_SOURCE + 1
        );
    }

    #[test]
    fn public_provenance_sets_reject_empty_and_duplicate_members() {
        let compiler = TargetCompilerBuild::new(
            TargetCompilerRole::RuntimeCompiler,
            "test-compiler".to_owned(),
            "1.0".to_owned(),
            None,
        )
        .unwrap();
        let environment = || {
            TargetExecutionEnvironment::builder()
                .platform("test-platform".to_owned())
                .platform_version("1.0".to_owned())
                .platform_build("build-1".to_owned())
                .architecture("test-architecture".to_owned())
                .hardware("test-hardware".to_owned())
                .build()
                .unwrap()
        };
        assert_eq!(
            TargetMeasurementContext::new(std::iter::empty(), environment()),
            Err(TargetFactSourceError::EmptyCompilerBuildSet)
        );
        assert_eq!(
            TargetMeasurementContext::new([compiler.clone(), compiler.clone()], environment()),
            Err(TargetFactSourceError::DuplicateCompilerBuild)
        );

        let context = TargetMeasurementContext::new([compiler], environment()).unwrap();
        let producer =
            || TargetFactProducerIdentity::new("test.measurement-set.v1".to_owned(), 1).unwrap();
        assert_eq!(
            TargetFactSource::measured(
                producer(),
                MeasuredFactAuthority::DeviceRuntime,
                std::iter::empty(),
            ),
            Err(TargetFactSourceError::EmptyMeasurementContextSet)
        );
        assert_eq!(
            TargetFactSource::measured(
                producer(),
                MeasuredFactAuthority::DeviceRuntime,
                [context.clone(), context],
            ),
            Err(TargetFactSourceError::DuplicateMeasurementContext)
        );
        assert_eq!(
            TargetCompileProfileMeasurementSource::new(producer(), std::iter::empty()),
            Err(TargetFactSourceError::EmptyMeasurementContextSet)
        );
        let context = TargetMeasurementContext::new(
            [TargetCompilerBuild::new(
                TargetCompilerRole::RuntimeCompiler,
                "test-compiler".to_owned(),
                "1.0".to_owned(),
                None,
            )
            .unwrap()],
            environment(),
        )
        .unwrap();
        assert_eq!(
            TargetCompileProfileMeasurementSource::new(producer(), [context.clone(), context],),
            Err(TargetFactSourceError::DuplicateMeasurementContext)
        );
    }

    #[test]
    fn public_provenance_errors_name_the_exact_field_and_bound() {
        assert_eq!(
            TargetFactProducerIdentity::new("Bad".to_owned(), 1),
            Err(TargetFactSourceError::InvalidFieldByte {
                field: "producer.key",
                index: 0,
                value: b'B',
            })
        );
        assert_eq!(
            TargetNormativeReferenceIdentity::new("test.reference.v1".to_owned(), 0),
            Err(TargetFactSourceError::ZeroRevision {
                field: "normative-reference.revision",
            })
        );
        assert_eq!(
            TargetCompilerRoleIdentity::new("Bad".to_owned(), 1),
            Err(TargetFactSourceError::InvalidFieldByte {
                field: "compiler-role.key",
                index: 0,
                value: b'B',
            })
        );
        assert_eq!(
            TargetCompilerRoleIdentity::new("test.compiler-role.v1".to_owned(), 0),
            Err(TargetFactSourceError::ZeroRevision {
                field: "compiler-role.revision",
            })
        );
        assert_eq!(
            TargetCompilerBuild::new(
                TargetCompilerRole::RuntimeCompiler,
                "x".repeat(MAX_TARGET_PROVENANCE_TEXT_BYTES + 1),
                "1".to_owned(),
                None,
            ),
            Err(TargetFactSourceError::FieldTooLong {
                field: "compiler-build.implementation",
                actual: MAX_TARGET_PROVENANCE_TEXT_BYTES + 1,
                max: MAX_TARGET_PROVENANCE_TEXT_BYTES,
            })
        );
        assert_eq!(
            TargetCompilerBuild::new(
                TargetCompilerRole::RuntimeCompiler,
                "test-runtime".to_owned(),
                "version 1 ".to_owned(),
                None,
            ),
            Err(TargetFactSourceError::InvalidFieldByte {
                field: "compiler-build.version",
                index: 9,
                value: b' ',
            })
        );
        assert_eq!(
            TargetExecutionEnvironment::builder().build(),
            Err(TargetFactSourceError::MissingField {
                field: "environment.platform",
            })
        );
    }

    #[test]
    fn public_declarations_reject_duplicates_atomically_before_insertion() {
        let mut builder = public_builder("test.atomic.v1");
        let source = public_external_source(1);
        let quantitative_len = builder.quantitative.len();
        assert_eq!(
            builder.declare_index_arithmetic(IndexArithmeticSupport::Unsupported, source.clone(),),
            Err(TargetProfileBuildError::DuplicateQuantitativeCapability {
                axis: "index-arithmetic-u64",
                phase: AvailabilityPhase::CompileProfile,
            })
        );
        assert_eq!(builder.quantitative.len(), quantitative_len);

        builder
            .declare_input_subnormals(
                ScalarArithmetic::f32(),
                SubnormalMode::Preserve,
                ScalarSupport::Exact,
                source.clone(),
            )
            .unwrap();
        let scalar_len = builder.scalar.len();
        assert_eq!(
            builder.declare_input_subnormals(
                ScalarArithmetic::f32(),
                SubnormalMode::Preserve,
                ScalarSupport::Unsupported,
                source.clone(),
            ),
            Err(TargetProfileBuildError::DuplicateScalarDeclaration)
        );
        assert_eq!(builder.scalar.len(), scalar_len);

        let f32 = F32::resolved_type();
        builder
            .declare_dtype_dispatchability(
                f32.clone(),
                DTypeDispatchability::Dispatchable,
                source.clone(),
            )
            .unwrap();
        let dispatch_len = builder.dispatchability.len();
        assert_eq!(
            builder.declare_dtype_dispatchability(f32, DTypeDispatchability::Unsupported, source,),
            Err(TargetProfileBuildError::DuplicateDispatchability)
        );
        assert_eq!(builder.dispatchability.len(), dispatch_len);
        builder
            .build()
            .expect("every retained declaration is valid");
    }

    #[test]
    fn declaration_order_does_not_change_the_canonical_profile() {
        let key = TargetProfileKey::new("test.canonical-order.v1".to_owned()).unwrap();
        let source = public_external_source(1);
        let first_type = nominal("canonical-a");
        let second_type = nominal("canonical-b");
        let mut forward = TargetProfileBuilder::new(key.clone());
        forward
            .declare_max_threads_per_grid_axis(64, source.clone())
            .unwrap();
        forward
            .declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
            .unwrap();
        forward
            .declare_dtype_dispatchability(
                first_type.clone(),
                DTypeDispatchability::Dispatchable,
                source.clone(),
            )
            .unwrap();
        forward
            .declare_dtype_dispatchability(
                second_type.clone(),
                DTypeDispatchability::Unsupported,
                source.clone(),
            )
            .unwrap();
        let mut reverse = TargetProfileBuilder::new(key);
        reverse
            .declare_index_arithmetic(IndexArithmeticSupport::CompleteU64, source.clone())
            .unwrap();
        reverse
            .declare_max_threads_per_grid_axis(64, source.clone())
            .unwrap();
        reverse
            .declare_dtype_dispatchability(
                second_type,
                DTypeDispatchability::Unsupported,
                source.clone(),
            )
            .unwrap();
        reverse
            .declare_dtype_dispatchability(first_type, DTypeDispatchability::Dispatchable, source)
            .unwrap();
        let forward = forward.build().unwrap();
        let reverse = reverse.build().unwrap();
        assert_eq!(forward, reverse);
        assert_eq!(
            forward.canonical_descriptor(),
            reverse.canonical_descriptor()
        );
        assert_eq!(
            forward.request_subject_bytes(),
            reverse.request_subject_bytes()
        );
    }

    #[test]
    fn malformed_capability_declarations_fail_at_the_checked_boundary() {
        let mut builder = TargetProfileBuilder::governed();
        builder
            .quantitative
            .iter_mut()
            .find(|declaration| declaration.axis == CapabilityAxis::IndexArithmeticU64)
            .unwrap()
            .bound = 2;
        assert_eq!(
            builder.try_build(),
            Err(TargetProfileBuildError::MalformedProfile { rule: "fact-bound" })
        );
    }

    #[test]
    fn quantitative_facts_retain_and_derive_from_their_structured_producer() {
        let mut builder = TargetProfileBuilder::governed();
        let source = measured_capability_source();
        for declaration in &mut builder.quantitative {
            declaration.source = Arc::clone(&source);
        }
        builder.dispatchability.clear();
        let profile = builder.try_build().unwrap();
        assert!(
            profile
                .data
                .quantitative
                .iter()
                .all(|declaration| declaration.source == source)
        );
        for fact in profile.checked().facts() {
            assert_eq!(fact.phase(), AvailabilityPhase::LiveDevicePreflight);
            assert_eq!(fact.authority(), FactAuthority::DeviceRuntime);
            assert_eq!(fact.validity(), FactValidityScope::DeviceInstance);
        }
    }

    #[test]
    fn sparse_quantitative_omission_resolves_to_unknown() {
        let source = public_external_source(1);
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.sparse-quantitative.v1".to_owned()).unwrap(),
        );
        builder
            .declare_max_threads_per_grid_axis(64, source)
            .unwrap();
        let profile = builder.build().unwrap();
        let proposal = FeasibilityProposal::new(
            "requires-workgroup",
            vec![AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 1)],
            Vec::new(),
        )
        .unwrap();
        assert!(matches!(
            profile
                .checked()
                .assess(&proposal, AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Unknown(_)
        ));
    }

    #[test]
    fn measured_profile_omissions_remain_unknown_across_fact_families() {
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.sparse-measured.v1".to_owned()).unwrap(),
        );
        builder
            .declare_measured_max_threads_per_grid_axis(
                64,
                compile_profile_measurement_source("1.0", "build-1"),
            )
            .unwrap();
        let profile = builder.build().unwrap();
        let proposal = FeasibilityProposal::new(
            "requires-omitted-facts",
            vec![AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 1)],
            vec![NumericalRequirement::new(
                NumericalDimension::Contraction,
                ArithmeticType::F32,
                F32::resolved_type(),
                DimensionBehaviour::Transform(NumericalPermission::Forbidden),
            )],
        )
        .unwrap();
        assert!(matches!(
            profile
                .checked()
                .assess(&proposal, AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Unknown(_)
        ));
        assert_eq!(
            profile.dtype_dispatchability(&F32::resolved_type(), AvailabilityPhase::CompileProfile),
            DTypeDispatchabilityResolution::Unknown
        );
    }

    #[test]
    fn each_quantitative_axis_binds_its_own_source_into_identity() {
        let baseline = public_builder("test.mixed-sources.v1").build().unwrap();
        let mut mixed = public_builder("test.mixed-sources.v1");
        let revised = public_external_source(2);
        mixed
            .quantitative
            .iter_mut()
            .find(|declaration| declaration.axis == CapabilityAxis::LocalMemoryBytes)
            .unwrap()
            .source = Arc::clone(&revised.0);
        let mixed = mixed.build().unwrap();
        assert_ne!(
            baseline.canonical_descriptor(),
            mixed.canonical_descriptor(),
            "changing one axis's source must change complete identity"
        );
        assert_ne!(
            baseline.request_subject_bytes(),
            mixed.request_subject_bytes(),
            "the request subject must bind per-axis provenance"
        );
    }

    #[test]
    fn one_axis_may_be_refined_at_a_later_availability_phase() {
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.staged-quantitative.v1".to_owned()).unwrap(),
        );
        builder
            .declare_max_threads_per_workgroup(64, public_external_source(1))
            .unwrap();
        builder
            .declare_max_threads_per_workgroup(32, TargetFactSource(measured_capability_source()))
            .unwrap();
        let profile = builder
            .build()
            .expect("facts at distinct phases do not collide");
        let proposal = FeasibilityProposal::new(
            "requires-workgroup",
            vec![AxisRequirement::new(CapabilityAxis::WorkgroupThreads, 48)],
            Vec::new(),
        )
        .unwrap();
        assert!(matches!(
            profile
                .checked()
                .assess(&proposal, AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Proven(_)
        ));
        assert!(matches!(
            profile
                .checked()
                .assess(&proposal, AvailabilityPhase::LiveDevicePreflight),
            FeasibilityOutcome::Rejected(_)
        ));
    }

    #[test]
    fn request_subject_binds_local_memory() {
        let baseline = public_builder("test.request-subject.v1").build().unwrap();
        let mut changed = public_builder("test.request-subject.v1");
        changed
            .quantitative
            .iter_mut()
            .find(|declaration| declaration.axis == CapabilityAxis::LocalMemoryBytes)
            .unwrap()
            .bound += 1;
        let changed = changed.build().unwrap();
        assert_ne!(
            baseline.request_subject_bytes(),
            changed.request_subject_bytes()
        );
        assert_eq!(
            baseline.request_subject_bytes(),
            baseline.canonical_descriptor()
        );
    }

    #[test]
    fn arithmetic_support_and_device_address_width_move_identity_independently() {
        let baseline = public_builder("test.width-independence.v1")
            .build()
            .unwrap();
        let mut arithmetic = public_builder("test.width-independence.v1");
        arithmetic
            .quantitative
            .iter_mut()
            .find(|declaration| declaration.axis == CapabilityAxis::IndexArithmeticU64)
            .unwrap()
            .bound = IndexArithmeticSupport::Unsupported.bound();
        let arithmetic = arithmetic.build().unwrap();
        let mut address = public_builder("test.width-independence.v1");
        address
            .quantitative
            .iter_mut()
            .find(|declaration| declaration.axis == CapabilityAxis::DeviceAddressWidthBits)
            .unwrap()
            .bound = u64::from(DeviceAddressWidth::Bits32.bits());
        let address = address.build().unwrap();

        assert_ne!(
            baseline.canonical_descriptor(),
            arithmetic.canonical_descriptor()
        );
        assert_ne!(
            baseline.canonical_descriptor(),
            address.canonical_descriptor()
        );
        assert_ne!(
            arithmetic.canonical_descriptor(),
            address.canonical_descriptor()
        );
    }

    #[test]
    fn governed_profile_does_not_invent_a_device_address_width() {
        assert!(
            TargetProfile::governed()
                .checked()
                .facts()
                .iter()
                .all(|fact| fact.axis() != CapabilityAxis::DeviceAddressWidthBits)
        );
    }

    #[test]
    fn dtype_dispatch_is_exact_sparse_and_has_no_inheritance() {
        let f32 = F32::resolved_type();
        let parameterized_f32 = ResolvedValueType::parameterized(
            TypeKey::new("test", "wrapped", 1).unwrap(),
            TypeArguments::new([CanonicalValue::value_type(f32.clone())]).unwrap(),
        )
        .unwrap();
        let mut builder = TargetProfileBuilder::governed();
        builder.dispatchability = vec![dispatch_fact(
            f32.clone(),
            DTypeDispatchability::Dispatchable,
        )];
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.dtype_dispatchability(&f32, AvailabilityPhase::CompileProfile),
            DTypeDispatchabilityResolution::Dispatchable
        );
        assert_eq!(
            profile.dtype_dispatchability(&parameterized_f32, AvailabilityPhase::CompileProfile,),
            DTypeDispatchabilityResolution::Unknown
        );

        let mut unsupported = TargetProfileBuilder::governed();
        unsupported.dispatchability = vec![dispatch_fact(
            parameterized_f32.clone(),
            DTypeDispatchability::Unsupported,
        )];
        assert_eq!(
            unsupported
                .try_build()
                .unwrap()
                .dtype_dispatchability(&parameterized_f32, AvailabilityPhase::CompileProfile,),
            DTypeDispatchabilityResolution::Unsupported
        );
    }

    #[test]
    fn dtype_dispatch_refines_by_phase_and_defers_before_its_first_fact() {
        let f32 = F32::resolved_type();
        let mut staged = TargetProfileBuilder::new(
            TargetProfileKey::new("test.staged-dispatch.v1".to_owned()).unwrap(),
        );
        staged.dispatchability.push(dispatch_fact(
            f32.clone(),
            DTypeDispatchability::Dispatchable,
        ));
        staged.dispatchability.push(DTypeDispatchabilityFact {
            resolved_type: f32.clone(),
            verdict: DTypeDispatchability::Unsupported,
            source: measured_capability_source(),
        });
        let staged = staged.build().unwrap();
        assert_eq!(
            staged.dtype_dispatchability(&f32, AvailabilityPhase::CompileProfile),
            DTypeDispatchabilityResolution::Dispatchable
        );
        assert_eq!(
            staged.dtype_dispatchability(&f32, AvailabilityPhase::LiveDevicePreflight),
            DTypeDispatchabilityResolution::Unsupported
        );

        let mut later_only = TargetProfileBuilder::new(
            TargetProfileKey::new("test.later-dispatch.v1".to_owned()).unwrap(),
        );
        later_only.dispatchability.push(DTypeDispatchabilityFact {
            resolved_type: f32.clone(),
            verdict: DTypeDispatchability::Dispatchable,
            source: measured_capability_source(),
        });
        let later_only = later_only.build().unwrap();
        assert_eq!(
            later_only.dtype_dispatchability(&f32, AvailabilityPhase::CompileProfile),
            DTypeDispatchabilityResolution::Deferred {
                available_at: AvailabilityPhase::LiveDevicePreflight,
            }
        );
    }

    #[test]
    fn scalar_subject_row_swaps_change_identity_and_exact_feasibility() {
        let source = governed_profile_source();
        let a = nominal("scalar-a");
        let b = nominal("scalar-b");
        let row = |resolved_type, behaviour| ScalarHonourabilityDeclaration {
            subject: ScalarArithmetic {
                arithmetic: ArithmeticType::F32,
                resolved_type,
            },
            dimension: NumericalDimension::InputSubnormals,
            behaviour,
            means: HonouringMeans::SupportedExactly,
            source: Arc::clone(&source),
        };
        let preserve = DimensionBehaviour::Subnormals(SubnormalMode::Preserve);
        let flush = DimensionBehaviour::Subnormals(SubnormalMode::FlushToZero {
            zero_sign: FlushedZeroSign::PreservesSign,
        });
        let profile = |key: &str,
                       first: ScalarHonourabilityDeclaration,
                       second: ScalarHonourabilityDeclaration| {
            let mut builder =
                TargetProfileBuilder::new(TargetProfileKey::new(key.to_owned()).unwrap());
            builder.scalar = vec![first, second];
            builder.build().unwrap()
        };
        let left = profile(
            "test.scalar-row-swap.v1",
            row(a.clone(), preserve),
            row(b.clone(), flush),
        );
        let right = profile(
            "test.scalar-row-swap.v1",
            row(a.clone(), flush),
            row(b, preserve),
        );
        assert_ne!(left.canonical_descriptor(), right.canonical_descriptor());

        let proposal = FeasibilityProposal::new(
            "exact-scalar-subject",
            Vec::new(),
            vec![NumericalRequirement::new(
                NumericalDimension::InputSubnormals,
                ArithmeticType::F32,
                a,
                preserve,
            )],
        )
        .unwrap();
        assert!(matches!(
            left.checked()
                .assess(&proposal, AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Proven(_)
        ));
        assert!(matches!(
            right
                .checked()
                .assess(&proposal, AvailabilityPhase::CompileProfile),
            FeasibilityOutcome::Unknown(_)
        ));
    }

    #[test]
    fn duplicate_exact_dtype_dispatch_claims_are_rejected() {
        let value_type = nominal("same");
        let mut builder = TargetProfileBuilder::governed();
        builder.dispatchability = vec![
            dispatch_fact(value_type.clone(), DTypeDispatchability::Dispatchable),
            dispatch_fact(value_type, DTypeDispatchability::Unsupported),
        ];
        assert_eq!(
            builder.try_build(),
            Err(TargetProfileBuildError::DuplicateDispatchability)
        );
    }

    #[test]
    fn complete_descriptor_is_cached_canonical_and_schema_distinct() {
        let mut left = TargetProfileBuilder::governed();
        left.scalar.reverse();
        let right = TargetProfileBuilder::governed();
        let left = left.try_build().unwrap();
        let right = right.try_build().unwrap();
        assert_eq!(left.canonical_descriptor(), right.canonical_descriptor());
        let cloned = left.clone();
        assert!(
            std::ptr::eq(left.canonical_descriptor(), cloned.canonical_descriptor()),
            "cloning a frozen profile retains its shared immutable allocation"
        );
        assert!(std::ptr::eq(
            left.canonical_descriptor(),
            left.canonical_descriptor()
        ));
        assert_ne!(
            left.canonical_descriptor(),
            left.checked().canonical_descriptor()
        );
        assert!(
            left.canonical_descriptor()
                .windows(COMPLETE_PROFILE_DESCRIPTOR_DOMAIN.len())
                .any(|window| window == COMPLETE_PROFILE_DESCRIPTOR_DOMAIN)
        );
    }

    #[test]
    fn exact_dtype_and_verdict_participate_in_complete_identity() {
        let descriptor = |resolved_type, verdict| {
            let mut builder = TargetProfileBuilder::governed();
            builder.dispatchability = vec![dispatch_fact(resolved_type, verdict)];
            builder.try_build().unwrap().canonical_descriptor().to_vec()
        };
        assert_ne!(
            descriptor(nominal("a"), DTypeDispatchability::Dispatchable),
            descriptor(nominal("b"), DTypeDispatchability::Dispatchable)
        );
        assert_ne!(
            descriptor(nominal("a"), DTypeDispatchability::Dispatchable),
            descriptor(nominal("a"), DTypeDispatchability::Unsupported)
        );
    }

    #[test]
    fn complete_descriptor_obeys_the_artifact_identity_bound() {
        let mut builder = TargetProfileBuilder::governed();
        builder.dispatchability = (0..1_024)
            .map(|index| {
                dispatch_fact(
                    nominal(format!("dtype-{index:02}")),
                    DTypeDispatchability::Dispatchable,
                )
            })
            .collect();
        builder.dispatchability.reverse();
        let failure = builder.build().unwrap_err();
        assert!(matches!(
            failure,
            TargetProfileBuildError::DescriptorTooLong { actual, max }
                if actual > max && max == MAX_TARGET_PROFILE_DESCRIPTOR_BYTES
        ));
    }

    #[test]
    fn target_requests_are_nonempty_unique_and_preserve_order() {
        assert_eq!(TargetRequest::new([]), Err(TargetRequestError::Empty));
        let first = public_builder("test.ordered-a.v1").build().unwrap();
        let second = public_builder("test.ordered-b.v1").build().unwrap();
        assert_eq!(
            TargetRequest::new([first.clone(), first.clone()]),
            Err(TargetRequestError::DuplicateProfile {
                profile: first.profile_key().clone(),
                first: 0,
                duplicate: 1,
            })
        );
        let request = TargetRequest::new([second.clone(), first.clone()]).unwrap();
        assert_eq!(request.profiles(), &[second, first]);
    }
}

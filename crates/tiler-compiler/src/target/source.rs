//! The declaration-side vocabulary a producer attributes a target fact to.
//!
//! Everything here is what an external declarer *supplies*: versioned producer
//! and normative-reference identities, the exact compiler builds and execution
//! environment of a measurement, the two source constructors, and the field
//! validators and bounds they enforce. Reading a retained fact back is the
//! separate vocabulary in [`super::evidence`], and the asymmetry is deliberate.

use std::sync::Arc;

use crate::target::honourability::{
    CompilationSelectionIdentity, CompileProfileMeasurementContext, CompilerBuildIdentity,
    CompilerBuildRole, ExecutionEnvironmentIdentity, FactSourceProvenance,
    MAX_COMPILATION_SELECTION_IDENTITY_BYTES, MAX_COMPILER_BUILDS_PER_CONTEXT,
    MAX_MEASUREMENT_CONTEXTS_PER_SOURCE, MAX_PROVENANCE_TEXT_BYTES, MeasurementContext,
    PostCompileMeasurementAuthority, ProvenanceIdentity,
};

/// Maximum UTF-8 byte length of one target-fact provenance field.
pub const MAX_TARGET_PROVENANCE_TEXT_BYTES: usize = MAX_PROVENANCE_TEXT_BYTES;
/// Maximum compiler builds admitted in one measurement context.
pub const MAX_TARGET_COMPILER_BUILDS_PER_CONTEXT: usize = MAX_COMPILER_BUILDS_PER_CONTEXT;
/// Maximum measurement contexts admitted in one measured source.
pub const MAX_TARGET_MEASUREMENT_CONTEXTS_PER_SOURCE: usize = MAX_MEASUREMENT_CONTEXTS_PER_SOURCE;
/// Maximum byte length of one compile-profile compilation-selection identity.
///
/// Exactly the existing complete-target-descriptor ceiling, not a new narrower
/// policy; the profile builder's cumulative descriptor limit remains the final
/// authority when several contexts and sources are combined.
pub const MAX_TARGET_COMPILATION_SELECTION_IDENTITY_BYTES: usize =
    MAX_COMPILATION_SELECTION_IDENTITY_BYTES;

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
    /// The IR post-compile authority this public authority derives, which is
    /// what fixes the phase/authority/validity triple by construction.
    const fn internal(self) -> PostCompileMeasurementAuthority {
        match self {
            Self::ArtifactEvidence => PostCompileMeasurementAuthority::ArtifactEvidence,
            Self::DeviceRuntime => PostCompileMeasurementAuthority::DeviceRuntime,
            Self::PreparedKernel => PostCompileMeasurementAuthority::PreparedKernel,
            Self::LaunchInstance => PostCompileMeasurementAuthority::LaunchInstance,
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
        let compiler_builds = collect_compiler_builds(compiler_builds)?;
        let value = MeasurementContext::new(compiler_builds, environment.0);
        Ok(Self(value))
    }
}

/// The exact backend-owned compilation-selection identity of one
/// compile-profile measurement.
///
/// Opaque to the compiler on purpose: the producing backend owns the grammar,
/// and this layer only validates the envelope, retains the exact bytes,
/// compares them, and exposes them.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetCompilationSelectionIdentity(CompilationSelectionIdentity);

impl TargetCompilationSelectionIdentity {
    /// Admits exact selection bytes.
    ///
    /// # Errors
    ///
    /// Returns [`TargetFactSourceError::EmptyCompilationSelectionIdentity`] for
    /// no bytes and
    /// [`TargetFactSourceError::CompilationSelectionIdentityTooLong`] past the
    /// existing complete-target-descriptor ceiling. Both refuse before any
    /// proportional allocation.
    pub fn from_bytes(value: impl AsRef<[u8]>) -> Result<Self, TargetFactSourceError> {
        use tiler_ir::numerics::CompilationSelectionIdentityError;
        CompilationSelectionIdentity::from_bytes(value)
            .map(Self)
            .map_err(|error| match error {
                CompilationSelectionIdentityError::Empty => {
                    TargetFactSourceError::EmptyCompilationSelectionIdentity
                }
                CompilationSelectionIdentityError::TooLong { actual, max } => {
                    TargetFactSourceError::CompilationSelectionIdentityTooLong { actual, max }
                }
                // The IR error vocabulary is non-exhaustive; a widened refusal
                // must gain its own target spelling rather than borrow one.
                _ => unreachable!("the selection-identity bounds are empty and too-long"),
            })
    }

    /// The exact admitted bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// One compile-profile measurement: compiler builds, the environment they ran
/// in, and the required exact compilation selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetCompileProfileMeasurementContext(CompileProfileMeasurementContext);

impl TargetCompileProfileMeasurementContext {
    /// Constructs a compile-profile context. At least one distinct compiler
    /// build and exactly one admitted selection are required; there is no
    /// selection-free spelling.
    ///
    /// # Errors
    ///
    /// Returns a set-specific diagnostic for an empty, duplicated, or oversized
    /// compiler-build set.
    pub fn new(
        compiler_builds: impl IntoIterator<Item = TargetCompilerBuild>,
        environment: TargetExecutionEnvironment,
        compilation_selection: TargetCompilationSelectionIdentity,
    ) -> Result<Self, TargetFactSourceError> {
        let compiler_builds = collect_compiler_builds(compiler_builds)?;
        let value = CompileProfileMeasurementContext::new(
            compiler_builds,
            environment.0,
            compilation_selection.0,
        );
        Ok(Self(value))
    }
}

fn collect_compiler_builds(
    compiler_builds: impl IntoIterator<Item = TargetCompilerBuild>,
) -> Result<Vec<CompilerBuildIdentity>, TargetFactSourceError> {
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
    Ok(compiler_builds.into_iter().map(|build| build.0).collect())
}

/// Structured source attribution for target-profile facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetFactSource(pub(super) Arc<FactSourceProvenance>);

/// Empirical compiler-profile provenance bound to exact measurement contexts.
///
/// Unlike [`TargetFactSource::external_guarantee`], this source cannot claim
/// portable normative authority. Its phase, authority, and validity are fixed
/// to compile profile, measured profile, and measured environment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetCompileProfileMeasurementSource(pub(super) Arc<FactSourceProvenance>);

impl TargetCompileProfileMeasurementSource {
    /// Constructs compiler-profile measurement provenance.
    ///
    /// Every context carries its required exact compilation selection: the
    /// count bound is over contexts, not the aggregate encoded source length,
    /// so a standalone admitted source can retain at most 64 × 64 KiB of
    /// selection payload, and the complete-descriptor ceiling remains the
    /// cumulative authority.
    ///
    /// # Errors
    ///
    /// Returns a set-specific diagnostic for an empty, duplicated, or oversized
    /// measurement-context collection.
    pub fn new(
        producer: TargetFactProducerIdentity,
        contexts: impl IntoIterator<Item = TargetCompileProfileMeasurementContext>,
    ) -> Result<Self, TargetFactSourceError> {
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
        Ok(Self(Arc::new(
            FactSourceProvenance::compile_profile_measured(
                producer.0,
                contexts.into_iter().map(|context| context.0).collect(),
            ),
        )))
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
        let contexts = collect_measurement_contexts(contexts)?;
        let value =
            FactSourceProvenance::post_compile_measured(authority.internal(), producer.0, contexts);
        Ok(Self(Arc::new(value)))
    }

    /// The shared structured provenance this source carries.
    pub(super) fn provenance(&self) -> Arc<FactSourceProvenance> {
        Arc::clone(&self.0)
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
    /// A compile-profile compilation-selection identity carried no bytes.
    ///
    /// Absence is not a selection, and there is no default, sentinel, or
    /// inference to fall back to.
    EmptyCompilationSelectionIdentity,
    /// A compilation-selection identity exceeded the complete-target-descriptor
    /// ceiling.
    CompilationSelectionIdentityTooLong {
        /// Actual byte length offered.
        actual: usize,
        /// Maximum admitted byte length.
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

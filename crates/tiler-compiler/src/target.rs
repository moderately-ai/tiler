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
//! [`ScalarArithmetic::new`] is **not** covered by that acceptance. It is the
//! validated arithmetic/value-type construction route added later so a BF16
//! numerical row could be stated at all, and a new public method on an
//! already-public type falls in the gap ADR 0075's two category lists leave —
//! neither a new namespace, trait, promotion, or breaking signature change, nor
//! one of the categories a coordinator may merge unaided. It is a tested
//! concrete draft of that interface awaiting a boundary decision of its own, and
//! this sentence is what a reader should find rather than an acceptance that was
//! never given.
//!
//! The **evaluation-order-preservation** family carries an acceptance of its
//! own rather than the `4ad5a2e` one. Tom accepted it on 2026-08-06 at the live
//! session's decision round, as-is with no exclusion, under
//! `accept-the-evaluation-order-preservation-target-fact`:
//! [`BackendArithmeticLicence`], [`EvaluationOrderPreservation`],
//! [`EvaluationOrderResolution`],
//! [`TargetProfileBuilder::declare_evaluation_order_preservation`],
//! [`TargetProfileBuilder::declare_measured_evaluation_order_preservation`],
//! [`TargetProfile::evaluation_order_preservation`], and
//! [`TargetProfileBuildError::DuplicateEvaluationOrderPreservation`].
//!
//! The **elementary-realization** family is a **labelled draft** under ADR
//! 0075. Tom accepted the whole-subject *shape* on 2026-08-11 — one validated
//! record, operation derived from a verified contract, both complete evidence
//! records, compile-profile-phase source, stored canonical rows, no governed
//! shortcut — and has not accepted this crate's exact method, type, or
//! refusal-candidate spelling. [`ElementaryRealization`],
//! [`TargetProfileBuilder::declare_elementary_realization`],
//! [`TargetProfile::declared_elementary_realizations`], and
//! [`TargetProfileBuildError::DuplicateElementaryRealization`] are that draft.
//!
//! The **measured-cost-row** family is an **Accepted public surface**. Tom
//! accepted it on 2026-08-07 under
//! `accept-the-measured-cost-row-public-surface`:
//! [`TargetCostRowResolution`],
//! [`TargetProfileBuilder::declare_saturated_parallel_fold_steps`],
//! [`TargetProfileBuilder::declare_measured_saturated_parallel_fold_steps`],
//! [`TargetProfile::saturated_parallel_fold_steps`], and
//! [`TargetProfileBuildError::DuplicateCostRow`]. Its selection may consult a
//! measured term where a qualified profile declares one, carried as a kind
//! distinct from a capability axis, with silence meaning *no preference* rather
//! than *no plan*.
//!
//! The **subgroup-realization** family is a **labelled draft** under ADR 0075.
//! Tom accepted the whole-subject *shape* on 2026-08-11 — one checked subject
//! over a literal width, an exact arithmetic type, and an operation-specific
//! transfer, matched only by equality, with `Realized` and `Unrealizable`
//! explicit and silence/`Unknown` for neighbours — and has not accepted this
//! crate's exact type, constructor, or error spelling.
//! [`SubgroupSupport`], [`SubgroupRealizationResolution`],
//! [`TargetProfileBuilder::declare_subgroup_realization`],
//! [`TargetProfileBuilder::declare_measured_subgroup_realization`],
//! [`TargetProfile::subgroup_realization`], and
//! [`TargetProfileBuildError::DuplicateSubgroupRealization`] are that draft.
//!
//! The **workgroup-tree-width-policy** family is an **Accepted public surface**.
//! Tom delegated the choice to the coordinator on 2026-08-11 under
//! `gate-the-workgroup-tree-on-an-explicit-qualified-width-policy`:
//! [`WorkgroupTreeWidthPolicy`], [`WorkgroupTreeWidthPolicyResolution`],
//! [`TargetProfileBuilder::declare_workgroup_tree_width_policy`],
//! [`TargetProfileBuilder::declare_measured_workgroup_tree_width_policy`],
//! [`TargetProfile::workgroup_tree_width_policy`], and
//! [`TargetProfileBuildError::DuplicateWorkgroupTreeWidthPolicy`]. One closed
//! variant, no omitted/default case, and no public numeric cap: a profile that
//! does not declare an accepted policy makes the single-workgroup tree
//! unavailable with a typed reason. Silence is not a preference and is not a
//! clamp onto `256`.
//!
//! Four exclusions were accepted with the evaluation-order-preservation family
//! and are deliberate rather than gaps: no
//! math-mode spelling, because `safe`/`relaxed`/`fast` are one backend driver's
//! option tokens and the licence is what the measurement attributes the
//! behaviour to; no twelfth numerical dimension, because this states what
//! Tiler's emission grants the backend translator rather than what a caller's
//! contract grants Tiler; no `Unknown` variant on the verdict, because absence
//! is the `Unknown` as it is for dtype dispatchability; and no feasibility
//! consumer, the fact being declared and resolvable while nothing yet admits or
//! refuses on it.
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

// The two crate-private children of this cluster, declared here rather than at
// the crate root so the direction runs one way from a single visible place: a
// producer declares facts through this root, `feasibility` turns them into hard
// admit/reject predicates, and `honourability` owns the per-dimension numerical
// vocabulary those predicates quantify over. Cost lives in `component_cost` and
// stays outside the cluster: feasibility is not a cost.
//
// `pub(crate)` rather than private because a private child of a module is
// visible only within that module and its descendants, and these two are
// consumed across the compiler — at the crate root the same declarations were
// crate-visible for free. `pub(crate)` restores exactly that reach and no more:
// neither module is nameable outside this crate, so nothing here widens the
// reviewed public `target` facade.
pub(crate) mod accuracy;
pub(crate) mod feasibility;
pub(crate) mod honourability;

pub use accuracy::{ElementaryRealization, ElementaryRealizationError};

use std::sync::{Arc, OnceLock};

use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::program::abi::{
    AvailabilityPhase, TargetPropertyKey, TargetPropertyProviderIdentity, TargetPropertyQuery,
};
use tiler_ir::schedule::{
    ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission, SubgroupRealizationSubject,
    SubnormalMode, SynchronizationSubject,
};
use tiler_ir::semantic::{F32, ResolvedValueType};

use crate::target::feasibility::{
    CapabilityAxis, CapabilityFact, CapabilityQuery, CheckedTargetProfile,
    DeclaredSubgroupRealization, DeclaredSynchronizationRealization, FactAuthority, FactProvenance,
    FactValidityScope, FeasibilityError, MAX_TARGET_PROFILE_DESCRIPTOR_BYTES, SubgroupRealization,
    SynchronizationRealization,
};
use crate::target::honourability::{
    CompilerBuildIdentity, CompilerBuildRole, DeclaredBehaviour, DimensionBehaviour,
    ExecutionEnvironmentIdentity, FactEvidenceBasis, FactSourceProvenance, HonouringMeans,
    MAX_COMPILER_BUILDS_PER_CONTEXT, MAX_MEASUREMENT_CONTEXTS_PER_SOURCE,
    MAX_PROVENANCE_TEXT_BYTES, MeasurementContext, NumericalDimension, NumericalRefusalEvidence,
    ProvenanceIdentity, governed_profile_source,
};

pub(crate) const GOVERNED_TARGET_PROFILE_KEY: &str = "tiler.prototype-target-neutral-baseline.v1";
/// Domain of the complete producer declaration carried into artifact identity.
///
/// This is a new grammar, not a continuation of feasibility's checked
/// descriptor: that one remains an internal feasibility component, while this
/// declaration encodes the same capability and numerical semantics plus exact
/// dtype dispatch and synchronization realization through one shared provenance
/// table. A reader of an older domain therefore cannot mistake these bytes for
/// the new grammar.
///
/// `v11` appends the synchronization-realization rows. Every profile's bytes
/// move, including a profile that declares none: the row family writes its
/// domain separator and a count, so "this target says nothing about
/// synchronization" becomes a recorded fact rather than an absence recoverable
/// from bytes that never stated it. That is the point of the step — a `v10`
/// declaration could not distinguish a target that had been asked from one that
/// had not, and those admit different candidates.
const COMPLETE_PROFILE_DESCRIPTOR_DOMAIN: &[u8] = b"tiler.target-profile.declaration.v11\0";
const PROFILE_SOURCE_DOMAIN: &[u8] = b"tiler.target-profile.fact-sources.v4\0";
const DISPATCHABILITY_DOMAIN: &[u8] = b"tiler.target-profile.dtype-dispatchability.v2\0";
/// Domain separating the synchronization-realization rows of one declaration.
///
/// A separator of its own, exactly as the dispatchability rows have one: the two
/// grammars are independent, so a reader must not be able to consume one row
/// family's bytes at the other's offset.
const SYNCHRONIZATION_DOMAIN: &[u8] = b"tiler.target-profile.synchronization-realization.v1\0";
/// Domain separating the evaluation-order-preservation rows of one declaration.
///
/// Its own separator, for the reason the two families above have one: the
/// grammars are independent, so no reader may consume one family's bytes at
/// another's offset. Unlike the synchronization separator this one is written
/// **only when the family is non-empty**, and
/// [`complete_descriptor`] states the derivation that licenses the difference.
const EVALUATION_ORDER_DOMAIN: &[u8] = b"tiler.target-profile.evaluation-order-preservation.v1\0";
/// Domain separating the measured cost rows of one declaration.
///
/// Its own separator, for the reason the three families above have one, and
/// written **only when the family is non-empty** for the reason the
/// evaluation-order family is: silence about a cost row means *no preference*,
/// which is what a profile that never carried the family already recorded, so
/// writing a zero count would move every existing profile's bytes to record
/// nothing new. [`complete_descriptor`] states the derivation.
const COST_ROW_DOMAIN: &[u8] = b"tiler.target-profile.cost-row.v1\0";
/// Domain separating the elementary-realization rows of one declaration.
///
/// Its own separator, for the reason the families above have one, and written
/// **only when the family is non-empty** for the reason the evaluation-order
/// and cost-row families are: an empty family and an absent family both mean
/// no installed realization, which is what every profile encoded before this
/// family existed. Writing a zero count would move every existing profile's
/// bytes to record nothing new. [`complete_descriptor`] states the derivation.
const ELEMENTARY_REALIZATION_DOMAIN: &[u8] = b"tiler.target-profile.elementary-realization.v1\0";
/// Domain separating the workgroup-tree-width-policy rows of one declaration.
///
/// Its own separator, for the reason the families above have one, and written
/// **only when the family is non-empty** so a profile that never carried the
/// family keeps the bytes it already encoded. Silence here is not a preference:
/// it makes the single-workgroup tree unavailable. Writing a zero count would
/// move every existing profile's identity to record that it still has no
/// policy. [`complete_descriptor`] states the derivation.
const WORKGROUP_TREE_WIDTH_POLICY_DOMAIN: &[u8] =
    b"tiler.target-profile.workgroup-tree-width-policy.v1\0";
/// Domain separating the subgroup-realization rows of one declaration.
///
/// Its own separator, for the reason the families above have one, and written
/// **only when the family is non-empty** so a profile that never carried the
/// family keeps the bytes it already encoded. Silence here is `Unknown` for
/// every subject: it is not a default width and not a neighbouring realization.
/// Writing a zero count would move every existing profile's identity to record
/// that it still has no subgroup row. [`complete_descriptor`] states the
/// derivation.
const SUBGROUP_REALIZATION_DOMAIN: &[u8] = b"tiler.target-profile.subgroup-realization.v1\0";

/// Maximum byte length of one target-profile key.
///
/// A *minting* bound: what a profile key this compiler build names may occupy.
/// `tiler_artifact::program::MAX_GOVERNED_KEY_BYTES` is 256 because that layer
/// holds keys minted by producers other than this compiler, and the smaller
/// number here is what makes the two safe together — every key this compiler
/// can name is packageable there. Neither crate depends on the other, so a
/// change requires checking both.
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
///
/// A key is non-empty, at most [`MAX_TARGET_PROFILE_KEY_BYTES`], and spelled in
/// ASCII lowercase, ASCII digits, `.`, `-`, and `_`.
///
/// # The alphabet is shared with the artifact layer, and shared deliberately
///
/// `tiler_artifact::program::TargetProfileKey` is a different type with the
/// same name — this one is what a compilation is *assessed against*, that one
/// is what a packaged artifact *carries* — and it admits exactly this alphabet.
/// The two agree because a profile key's whole job is to be compared byte for
/// byte against one some other producer minted: a spelling only one side admits
/// would leave two keys a reader sees as one comparing unequal, and a key
/// carrying case, whitespace, or a control byte cannot be reproduced from the
/// rejection that prints it. Neither crate depends on the other, so widening
/// either alphabet requires checking both.
///
/// The byte bounds are not shared and are not meant to be;
/// [`MAX_TARGET_PROFILE_KEY_BYTES`] records why.
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

    /// The shared structured provenance this source carries.
    fn provenance(&self) -> Arc<FactSourceProvenance> {
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
///
/// The type, and the catalog validation behind [`ScalarArithmetic::new`], are
/// `tiler_ir::numerics::ScalarArithmeticSubject`. Only the siting changed: the
/// governed built-in scalar catalog the constructor consults lives in
/// `tiler-ir`, and the artifact record has to name the same subject this
/// compiler declares about, so a subject minted here and one read off a record
/// are one type rather than two that must be kept in agreement.
pub use tiler_ir::numerics::ScalarArithmeticSubject as ScalarArithmetic;
/// The registered value identity one arithmetic type names.
pub(crate) use tiler_ir::numerics::registered_arithmetic_value_type;

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
            subject: ScalarArithmetic::f32(),
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
                // The relaxation names a subject rather than a loose
                // (arithmetic, type) pair, so the whole subject is compared in
                // one step. A profile may only condition a declaration on an
                // authorization stated for the same subject it is declaring
                // about; a relaxation naming another subject would make the
                // condition unresolvable against the caller's contract.
                if !relaxation.dimension().admits(relaxation.behaviour())
                    || relaxation.subject() != &self.subject.identity()
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

/// The arithmetic-rewriting licence a backend translation is granted.
///
/// **Accepted public surface.** The key half of the evaluation-order fact, which
/// Tom accepted on 2026-08-06 under
/// `accept-the-evaluation-order-preservation-target-fact`.
///
/// # Why the key is a licence rather than a backend flag spelling
///
/// The measurement this vocabulary exists to carry is indexed by Metal's
/// `-fmetal-math-mode`, whose three values are `safe`, `relaxed`, and `fast`.
/// Those are one backend driver's option tokens, and a consumer-agnostic profile
/// that named them would have learnt a Metal flag. What the measurement actually
/// attributes the behaviour to is the *licence set the emitted operations carry*:
/// [finding 34](../../../docs/research/apple-targets/numerical-behaviour.md)
/// records the reordering firing exactly where the emitted set carries LLVM's
/// `reassoc`, and names `reassoc` as "the licence that authorizes regrouping".
/// `safe` withholds it; `relaxed` and `fast` both grant it, differing only in
/// `nnan`/`ninf`, which no measured cell attributes an order change to.
///
/// So the two values below cover all three modes without inheriting between
/// them, and a row that separated `relaxed` from `fast` would be a third value
/// here — a build error at every match, never a silent reading of a neighbour's
/// fact.
///
/// This is **not** a caller permission. [`tiler_ir::schedule::NumericalPermission`]
/// states what a caller's contract allows *Tiler* to do; this states what Tiler's
/// emission allows the *backend translator* to do, and ADR 0011's rule that one
/// permission never implies another applies across the two vocabularies too.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BackendArithmeticLicence {
    /// The backend translation is granted no licence to rewrite floating-point
    /// arithmetic.
    Withheld,
    /// The backend translation is licensed to rewrite floating-point arithmetic,
    /// including regrouping a same-operation operand sequence.
    Granted,
}

impl BackendArithmeticLicence {
    /// Returns the stable governed key naming this licence.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Withheld => "arithmetic-rewriting-withheld",
            Self::Granted => "arithmetic-rewriting-granted",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::Withheld => 0x01,
            Self::Granted => 0x02,
        }
    }
}

/// Whether a backend translation preserves the evaluation order the emitted
/// program pins.
///
/// **Accepted public surface**, with the licence key above.
///
/// Two valued, and the negative is *statable*, exactly as
/// [`SynchronizationSupport`] is: a target measured to re-serialize a written
/// grouping records that, and a target nobody asked records nothing. Those are
/// different states — a typed refusal and an `Unknown` — and a vocabulary with
/// only a positive spelling would collapse them into one silence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EvaluationOrderPreservation {
    /// The backend executes the evaluation order the emitted program names.
    Preserved,
    /// The backend may execute some other legal order than the one emitted.
    NotPreserved,
}

impl EvaluationOrderPreservation {
    /// Returns the stable governed key naming this verdict.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Preserved => "evaluation-order-preserved",
            Self::NotPreserved => "evaluation-order-not-preserved",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::Preserved => 0x01,
            Self::NotPreserved => 0x02,
        }
    }
}

/// Result of an evaluation-order-preservation lookup.
///
/// **Accepted public surface**, with the two vocabularies above.
///
/// [`Self::Unknown`] is the fail-closed answer and the overwhelmingly common
/// one: a profile that declares nothing about the property answers it, and a
/// consumer may not read a neighbouring subject's or a neighbouring licence's
/// row in its place. The oracle's refusal class 3 is what consumes it — a plan
/// whose pinned order the backend is permitted to change is refused rather than
/// qualified — so an `Unknown` never becomes an admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvaluationOrderResolution {
    /// An exact declaration states the order is preserved.
    Preserved,
    /// An exact declaration states the order may be changed.
    NotPreserved,
    /// An exact declaration exists, but only from this later phase.
    Deferred {
        /// Earliest phase at which an exact declaration can resolve.
        available_at: AvailabilityPhase,
    },
    /// No exact declaration exists for this subject and licence.
    Unknown,
}

/// One target's evaluation-order-preservation row.
#[derive(Clone, Debug, Eq, PartialEq)]
struct EvaluationOrderFact {
    subject: ScalarArithmetic,
    licence: BackendArithmeticLicence,
    preservation: EvaluationOrderPreservation,
    source: Arc<FactSourceProvenance>,
}

impl EvaluationOrderFact {
    fn validate(&self) -> Result<(), TargetProfileBuildError> {
        if !self.source.is_valid() {
            return Err(TargetProfileBuildError::InvalidProducerClaim);
        }
        Ok(())
    }

    /// The canonical key this row is unique under: the exact scalar subject, the
    /// licence, and the phase. The verdict is deliberately excluded, for the
    /// reason [`TargetProfileBuildError::DuplicateSynchronizationRealization`]
    /// excludes it — a profile stating one subject both preserved and not
    /// preserved has stated a contradiction, and admitting both rows would leave
    /// whichever the sort put first deciding.
    fn subject_key(&self) -> (Vec<u8>, u8, AvailabilityPhase) {
        let mut subject = Vec::new();
        self.subject.encode(&mut subject);
        (subject, self.licence.tag(), self.source.phase())
    }

    fn encode(&self, bytes: &mut Vec<u8>, source_index: usize) {
        self.subject.encode(bytes);
        bytes.push(self.licence.tag());
        bytes.push(self.preservation.tag());
        encode_compact_index(bytes, source_index);
    }
}

/// One measured machine quantity a target may state a *preference* on.
///
/// **Deliberately not a [`CapabilityAxis`], and the distinction is the whole
/// point of the type.** Every capability axis is a hard bound: silence about one
/// resolves `Unknown`, and an `Unknown` never reaches an executable frontier.
/// `docs/research/program-planning/flash-class-capability-set.md` already
/// eliminated that shape for a bandwidth number and the argument transfers
/// unchanged — a cost row declared as a capability axis would make silence render
/// a profile **unexecutable for a quantity no feasibility predicate reads**,
/// which is the wrong failure direction. Silence about a cost row means *no
/// preference*, never *no plan*, and [`TargetCostRowResolution`] is where that is
/// written down.
///
/// Private, and the public surface is one `declare_*` / `declare_measured_*` pair
/// plus one reader per row, exactly as the quantitative axes are spelled. A
/// second row lands as a variant here plus its own pair, additively.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum CostRow {
    /// Fold steps the device retires at once when it is saturated.
    SaturatedParallelFoldSteps,
}

impl CostRow {
    /// The stable governed key naming this row.
    const fn key(self) -> &'static str {
        match self {
            Self::SaturatedParallelFoldSteps => "cost.saturated-parallel-fold-steps",
        }
    }
}

/// One declared cost row, its value, and who vouches for it.
#[derive(Clone, Debug, Eq, PartialEq)]
struct CostRowFact {
    row: CostRow,
    value: u64,
    source: Arc<FactSourceProvenance>,
}

impl CostRowFact {
    fn validate(&self) -> Result<(), TargetProfileBuildError> {
        if !self.source.is_valid() {
            return Err(TargetProfileBuildError::InvalidProducerClaim);
        }
        Ok(())
    }

    fn encode(&self, bytes: &mut Vec<u8>, source_index: usize) {
        push_slice(bytes, self.row.key().as_bytes());
        bytes.extend_from_slice(&self.value.to_le_bytes());
        encode_compact_index(bytes, source_index);
    }
}

/// Result of a cost-row lookup.
///
/// **Accepted public surface**, accepted by Tom on 2026-08-07 under
/// `accept-the-measured-cost-row-public-surface`, with the declaration pair and
/// reader below.
///
/// [`Self::Unknown`] is the common answer and it means **no preference**, not no
/// plan. A consumer must treat it, and [`Self::Deferred`], as evidence it does not
/// have — never as a refusal, and never as a zero. Nothing is inherited from a
/// neighbouring row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetCostRowResolution {
    /// An exact declaration states this value.
    Declared {
        /// The declared quantity, in the row's own unit.
        value: u64,
    },
    /// An exact declaration exists, but only from this later phase.
    Deferred {
        /// Earliest phase at which an exact declaration can resolve.
        available_at: AvailabilityPhase,
    },
    /// No declaration exists, which is a stated absence of preference.
    Unknown,
}

/// The closed tree-width policy a target must declare to offer the
/// single-workgroup tree.
///
/// **Accepted public surface.** Tom delegated the choice to the coordinator on
/// 2026-08-11 under `gate-the-workgroup-tree-on-an-explicit-qualified-width-policy`.
///
/// One variant, and there is deliberately no omitted/default case and no public
/// numeric cap. A profile that does not declare an accepted policy makes the
/// tree unavailable with a typed reason. The fixed internal `256` stays private
/// to the partition rule this variant names.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorkgroupTreeWidthPolicy {
    /// The existing nearest-admissible-width rule around the fixed internal
    /// value `256`, ties going to the narrower. Qualified by the retained
    /// 2026-08-07 Apple9 partition calibration.
    MeasuredNearestCap256V1,
}

impl WorkgroupTreeWidthPolicy {
    /// Returns the stable governed key naming this policy.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::MeasuredNearestCap256V1 => "workgroup-tree-width.measured-nearest-cap-256.v1",
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::MeasuredNearestCap256V1 => 0x01,
        }
    }
}

/// Result of a workgroup-tree-width-policy lookup.
///
/// **Accepted public surface**, with the declaration pair and reader.
///
/// [`Self::Unknown`] is the fail-closed answer: a profile that declares nothing
/// does not offer the single-workgroup tree. It is not a preference, not a
/// clamp onto `256`, and not a substitution of the balanced partition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkgroupTreeWidthPolicyResolution {
    /// An exact declaration states this closed policy.
    Declared(WorkgroupTreeWidthPolicy),
    /// An exact declaration exists, but only from this later phase.
    Deferred {
        /// Earliest phase at which an exact declaration can resolve.
        available_at: AvailabilityPhase,
    },
    /// No declaration exists, so the tree is unavailable.
    Unknown,
}

/// One declared workgroup-tree-width policy and who vouches for it.
#[derive(Clone, Debug, Eq, PartialEq)]
struct WorkgroupTreeWidthPolicyFact {
    policy: WorkgroupTreeWidthPolicy,
    source: Arc<FactSourceProvenance>,
}

impl WorkgroupTreeWidthPolicyFact {
    fn validate(&self) -> Result<(), TargetProfileBuildError> {
        if !self.source.is_valid() {
            return Err(TargetProfileBuildError::InvalidProducerClaim);
        }
        Ok(())
    }

    fn encode(&self, bytes: &mut Vec<u8>, source_index: usize) {
        bytes.push(self.policy.tag());
        encode_compact_index(bytes, source_index);
    }
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
    evaluation_order: Box<[EvaluationOrderFact]>,
    cost_rows: Box<[CostRowFact]>,
    tree_width_policies: Box<[WorkgroupTreeWidthPolicyFact]>,
    elementary: Box<[ElementaryRealization]>,
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
    synchronization: Vec<DeclaredSynchronizationRealization>,
    evaluation_order: Vec<EvaluationOrderFact>,
    cost_rows: Vec<CostRowFact>,
    tree_width_policies: Vec<WorkgroupTreeWidthPolicyFact>,
    elementary: Vec<ElementaryRealization>,
    subgroup: Vec<DeclaredSubgroupRealization>,
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
            synchronization: Vec::new(),
            evaluation_order: Vec::new(),
            cost_rows: Vec::new(),
            tree_width_policies: Vec::new(),
            elementary: Vec::new(),
            subgroup: Vec::new(),
        }
    }

    /// Declares one whole elementary-realization subject.
    ///
    /// **Labelled draft** under ADR 0075. The subject is already validated:
    /// its operation comes from a verified contract, both evidence records are
    /// complete, and its source is compile-profile-phase. This method stores
    /// the canonical row and refuses only an exact duplicate. Distinct
    /// contracts for one operation remain separate candidates. No row is
    /// replaced, merged, or preferred, and a half that cannot discharge is
    /// still stored so assessment can refuse it as `undischarged-evidence`.
    ///
    /// # Errors
    ///
    /// Returns [`TargetProfileBuildError::DuplicateElementaryRealization`] when
    /// the same complete row is already declared.
    pub fn declare_elementary_realization(
        &mut self,
        realization: ElementaryRealization,
    ) -> Result<(), TargetProfileBuildError> {
        if self
            .elementary
            .iter()
            .any(|existing| existing == &realization)
        {
            return Err(TargetProfileBuildError::DuplicateElementaryRealization);
        }
        self.elementary.push(realization);
        Ok(())
    }

    /// Declares whether this target realizes one *complete* subgroup subject.
    ///
    /// **Labelled draft** under ADR 0075. The whole subject is one argument on
    /// purpose, and there is deliberately no per-dimension spelling — no
    /// `declare_subgroup_width`, no `declare_subgroup_arithmetic`. Each
    /// dimension is separately true of some realization on some machine, so a
    /// profile able to state them independently would let a caller's conjunction
    /// be satisfied by facts none of which is about it. A target realizing two
    /// subjects declares two facts.
    ///
    /// The verdict is stated rather than implied by presence, so a measured
    /// negative is recordable: an absent declaration is `Unknown` and rejects
    /// before executable-frontier admission, while
    /// [`SubgroupSupport::Unrealizable`] is a typed refusal a caller can act on.
    ///
    /// Generic construction validates provenance and structure. Backend-family
    /// correspondence stays in the backend-owned binding layer; there is no
    /// default row, inherited target-family row, or generic wrong-backend guess.
    ///
    /// # Errors
    ///
    /// Returns [`TargetProfileBuildError::DuplicateSubgroupRealization`] when a
    /// fact for the same subject and phase is already declared.
    pub fn declare_subgroup_realization(
        &mut self,
        subject: SubgroupRealizationSubject,
        support: SubgroupSupport,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_subgroup_with_source(subject, support, source.0)
    }

    /// Declares a measured subgroup realization.
    ///
    /// **Labelled draft** under ADR 0075, with the constructor above.
    ///
    /// Taking [`TargetCompileProfileMeasurementSource`] rather than the general
    /// [`TargetFactSource`] is what fixes its validity at
    /// [`TargetFactValidityScope::MeasuredEnvironment`] and stops it widening
    /// into a portable claim.
    ///
    /// # Errors
    ///
    /// Returns [`TargetProfileBuildError::DuplicateSubgroupRealization`] when a
    /// fact for the same subject and phase is already declared.
    pub fn declare_measured_subgroup_realization(
        &mut self,
        subject: SubgroupRealizationSubject,
        support: SubgroupSupport,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_subgroup_with_source(subject, support, source.0)
    }

    fn declare_subgroup_with_source(
        &mut self,
        subject: SubgroupRealizationSubject,
        support: SubgroupSupport,
        source: Arc<FactSourceProvenance>,
    ) -> Result<(), TargetProfileBuildError> {
        if !source.is_valid() {
            return Err(TargetProfileBuildError::InvalidProducerClaim);
        }
        let realization = support.realization();
        if let Some(existing) = self
            .subgroup
            .iter()
            .find(|declared| declared.subject() == subject && declared.phase() == source.phase())
        {
            let exact_duplicate = existing.realization() == realization;
            let contradiction = existing.realization() != realization;
            if exact_duplicate || contradiction {
                return Err(TargetProfileBuildError::DuplicateSubgroupRealization);
            }
        }
        self.subgroup.push(DeclaredSubgroupRealization::new(
            subject,
            realization,
            source,
        ));
        Ok(())
    }

    /// Declares whether this target realizes one *complete* synchronization
    /// subject.
    ///
    /// The whole subject is one argument on purpose, and there is deliberately
    /// no per-dimension spelling — no `declare_barrier_execution_scope`, no
    /// `declare_fenced_spaces`. Each dimension is separately true of some
    /// realization on some machine, so a profile able to state them
    /// independently would let a caller's conjunction be satisfied by facts none
    /// of which is about it. A target realizing two subjects declares two facts.
    ///
    /// The verdict is stated rather than implied by presence, so a measured
    /// negative is recordable: an absent declaration is `Unknown` and rejects
    /// before executable-frontier admission, while
    /// [`SynchronizationSupport::Unrealizable`] is a typed refusal a caller can
    /// act on.
    ///
    /// # Errors
    ///
    /// Returns [`TargetProfileBuildError`] when a fact for the same subject and
    /// phase is already declared, or when the subject fences no memory domain —
    /// a fence over nothing publishes nothing, so no handoff could consume it.
    pub fn declare_synchronization_realization(
        &mut self,
        subject: SynchronizationSubject,
        support: SynchronizationSupport,
        source: &TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        let realization = support.realization();
        if subject.fenced_spaces.is_empty() {
            return Err(TargetProfileBuildError::VacuousSynchronizationSubject);
        }
        let source = source.provenance();
        if let Some(existing) = self
            .synchronization
            .iter()
            .find(|declared| declared.subject() == subject && declared.phase() == source.phase())
        {
            // Exact restatement and same-key contradiction refuse independently
            // of each other and of sort order. The public error is one variant
            // because the uniqueness key already excludes the verdict; a second
            // public type would be a new boundary Tom has not accepted.
            let exact_duplicate = existing.realization() == realization;
            let contradiction = existing.realization() != realization;
            if exact_duplicate || contradiction {
                return Err(TargetProfileBuildError::DuplicateSynchronizationRealization);
            }
        }
        self.synchronization
            .push(DeclaredSynchronizationRealization::new(
                subject,
                realization,
                source,
            ));
        Ok(())
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
        // Four is the widest signature the bounded profile can now assemble: a
        // recognized pointwise body has three leaves, so at most three input
        // tensors plus one output, which is also exactly what the strict-affine
        // dequantize region binds. It is stated as the governed budget's own
        // `buffers` bound rather than derived per strategy, so a plan the
        // request already admitted cannot then be refused for a binding count
        // the same build considered legal.
        //
        // It remains a compiler-governed prototype guarantee, **not** a device
        // measurement: `declare_measured_max_buffer_bindings_per_entry` is the
        // separate constructor a measured profile uses. Metal's own
        // documented per-stage buffer argument table bounds this far above —
        // the production profile ticket declares that figure with its per-row
        // authority ledger — so a conservative four claims nothing this
        // prototype authority cannot support.
        builder
            .declare_max_buffer_bindings_per_entry(4, source.clone())
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

    /// Declares whether a backend translation under one arithmetic-rewriting
    /// licence preserves the evaluation order the emitted program pins.
    ///
    /// **Accepted public surface**, accepted by Tom on 2026-08-06 under
    /// `accept-the-evaluation-order-preservation-target-fact`.
    ///
    /// The fact is keyed by the exact scalar subject as well as the licence,
    /// because nothing establishes that one width's answer is another's — the
    /// measurement behind the vocabulary is `f32` only, and the two subnormal
    /// dimensions are already measured *disagreeing* across widths on one Apple
    /// row. A subject or licence this profile does not speak about resolves
    /// [`EvaluationOrderResolution::Unknown`] rather than inheriting a
    /// neighbour's row.
    ///
    /// A profile that declares nothing here answers `Unknown` for every subject
    /// and licence, which is the fail-closed default: the oracle's refusal class
    /// 3 refuses a plan whose pinned order the backend may change rather than
    /// qualifying it.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate row.
    pub fn declare_evaluation_order_preservation(
        &mut self,
        subject: ScalarArithmetic,
        licence: BackendArithmeticLicence,
        preservation: EvaluationOrderPreservation,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_evaluation_order_with_source(subject, licence, preservation, source.0)
    }

    /// Declares a measured evaluation-order-preservation row.
    ///
    /// **Accepted public surface**, with the constructor above.
    ///
    /// The measured spelling is the one a target row is expected to use: the
    /// property is a fact about an exact backend compiler build, which no
    /// normative document this repository holds states — the vendored MSL 4.0
    /// and 4.1 specifications contain no occurrence of `evaluation order` at
    /// all, and the sentence that comes closest is already refuted as a
    /// universal claim by the same profile's subnormal rows.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate row.
    pub fn declare_measured_evaluation_order_preservation(
        &mut self,
        subject: ScalarArithmetic,
        licence: BackendArithmeticLicence,
        preservation: EvaluationOrderPreservation,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_evaluation_order_with_source(subject, licence, preservation, source.0)
    }

    /// Declares how many fold steps this target retires at once when its launch
    /// saturates the device.
    ///
    /// **Accepted public surface**, accepted by Tom on 2026-08-07 under
    /// `accept-the-measured-cost-row-public-surface`, with
    /// [`TargetCostRowResolution`] and the measured constructor below.
    ///
    /// This is a **cost row, not a capability axis**, and the difference is
    /// load-bearing rather than presentational. A capability axis is a hard bound
    /// a feasibility predicate reads, and silence about one is an `Unknown` that
    /// never reaches an executable frontier. Nothing reads this row for
    /// feasibility, so declaring it that way would make silence render a profile
    /// unexecutable for a quantity no predicate consults. Silence here means *no
    /// preference*: a profile declaring nothing selects exactly as it did before
    /// this row existed, byte for byte, and its canonical descriptor does not move.
    ///
    /// A value of zero is admitted and is a statement rather than an absence — it
    /// says the target retires no fold step in parallel — but no consumer in this
    /// build acts on it, because a selector dividing by it would have nothing to
    /// compare.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate row.
    pub fn declare_saturated_parallel_fold_steps(
        &mut self,
        steps: u64,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_cost_row(CostRow::SaturatedParallelFoldSteps, steps, source.0)
    }

    /// Declares a measured saturated-parallel-fold-step count.
    ///
    /// **Accepted public surface**, accepted by Tom on 2026-08-07 under
    /// `accept-the-measured-cost-row-public-surface`, with the constructor above.
    ///
    /// The measured spelling is the one a target row is expected to use, and it is
    /// the *only* one any profile in this repository uses. The quantity is a
    /// property of one device under one toolchain, fitted from a dispatch sweep;
    /// no normative document states it, and none could. Taking
    /// [`TargetCompileProfileMeasurementSource`] rather than the general
    /// [`TargetFactSource`] is what fixes its validity at
    /// [`TargetFactValidityScope::MeasuredEnvironment`] and stops it widening into
    /// a portable claim.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate row.
    pub fn declare_measured_saturated_parallel_fold_steps(
        &mut self,
        steps: u64,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_cost_row(CostRow::SaturatedParallelFoldSteps, steps, source.0)
    }

    fn declare_cost_row(
        &mut self,
        row: CostRow,
        value: u64,
        source: Arc<FactSourceProvenance>,
    ) -> Result<(), TargetProfileBuildError> {
        let fact = CostRowFact { row, value, source };
        fact.validate()?;
        if self
            .cost_rows
            .iter()
            .any(|existing| existing.row == row && existing.source.phase() == fact.source.phase())
        {
            return Err(TargetProfileBuildError::DuplicateCostRow {
                row: row.key(),
                phase: fact.source.phase(),
            });
        }
        self.cost_rows.push(fact);
        Ok(())
    }

    /// Declares the closed tree-width policy this target uses when it offers
    /// the single-workgroup tree.
    ///
    /// **Accepted public surface**, accepted by Tom's 2026-08-11 delegation
    /// under `gate-the-workgroup-tree-on-an-explicit-qualified-width-policy`,
    /// with [`WorkgroupTreeWidthPolicyResolution`] and the measured constructor
    /// below.
    ///
    /// This is **not a cost row and not a capability axis**. A cost row's
    /// silence means no preference; this family's silence makes the tree
    /// unavailable. A capability axis would make silence render a profile
    /// unexecutable for a quantity no feasibility predicate reads. The policy
    /// is a qualification on offering one strategy, decided before a region
    /// exists. There is no public numeric cap, no default, and no clamp.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate row.
    pub fn declare_workgroup_tree_width_policy(
        &mut self,
        policy: WorkgroupTreeWidthPolicy,
        source: TargetFactSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_tree_width_policy(policy, source.0)
    }

    /// Declares a measured workgroup-tree-width policy.
    ///
    /// **Accepted public surface**, with the constructor above.
    ///
    /// The measured spelling is the one a target row is expected to use, and it
    /// is the *only* one any production profile in this repository uses. Taking
    /// [`TargetCompileProfileMeasurementSource`] rather than the general
    /// [`TargetFactSource`] is what fixes its validity at
    /// [`TargetFactValidityScope::MeasuredEnvironment`] and stops it widening
    /// into a portable claim.
    ///
    /// # Errors
    ///
    /// Returns a typed error without inserting an invalid or duplicate row.
    pub fn declare_measured_workgroup_tree_width_policy(
        &mut self,
        policy: WorkgroupTreeWidthPolicy,
        source: TargetCompileProfileMeasurementSource,
    ) -> Result<(), TargetProfileBuildError> {
        self.declare_tree_width_policy(policy, source.0)
    }

    fn declare_tree_width_policy(
        &mut self,
        policy: WorkgroupTreeWidthPolicy,
        source: Arc<FactSourceProvenance>,
    ) -> Result<(), TargetProfileBuildError> {
        let fact = WorkgroupTreeWidthPolicyFact { policy, source };
        fact.validate()?;
        if self
            .tree_width_policies
            .iter()
            .any(|existing| existing.source.phase() == fact.source.phase())
        {
            return Err(TargetProfileBuildError::DuplicateWorkgroupTreeWidthPolicy {
                phase: fact.source.phase(),
            });
        }
        self.tree_width_policies.push(fact);
        Ok(())
    }

    fn declare_evaluation_order_with_source(
        &mut self,
        subject: ScalarArithmetic,
        licence: BackendArithmeticLicence,
        preservation: EvaluationOrderPreservation,
        source: Arc<FactSourceProvenance>,
    ) -> Result<(), TargetProfileBuildError> {
        let fact = EvaluationOrderFact {
            subject,
            licence,
            preservation,
            source,
        };
        fact.validate()?;
        let key = fact.subject_key();
        if self
            .evaluation_order
            .iter()
            .any(|existing| existing.subject_key() == key)
        {
            return Err(
                TargetProfileBuildError::DuplicateEvaluationOrderPreservation {
                    licence: licence.key(),
                    phase: fact.source.phase(),
                },
            );
        }
        self.evaluation_order.push(fact);
        Ok(())
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
        for fact in &self.synchronization {
            let key = fact.sort_key();
            let same_key: Vec<_> = self
                .synchronization
                .iter()
                .filter(|candidate| candidate.sort_key() == key)
                .collect();
            if same_key.len() <= 1 {
                continue;
            }
            let exact_duplicate = same_key
                .windows(2)
                .any(|pair| pair[0].realization() == pair[1].realization());
            let contradiction = same_key
                .windows(2)
                .any(|pair| pair[0].realization() != pair[1].realization());
            if exact_duplicate || contradiction {
                return Err(TargetProfileBuildError::DuplicateSynchronizationRealization);
            }
        }
        for fact in &self.evaluation_order {
            fact.validate()?;
            let key = fact.subject_key();
            if self
                .evaluation_order
                .iter()
                .filter(|candidate| candidate.subject_key() == key)
                .count()
                > 1
            {
                return Err(
                    TargetProfileBuildError::DuplicateEvaluationOrderPreservation {
                        licence: fact.licence.key(),
                        phase: fact.source.phase(),
                    },
                );
            }
        }
        for fact in &self.cost_rows {
            fact.validate()?;
            if self
                .cost_rows
                .iter()
                .filter(|candidate| {
                    candidate.row == fact.row && candidate.source.phase() == fact.source.phase()
                })
                .count()
                > 1
            {
                return Err(TargetProfileBuildError::DuplicateCostRow {
                    row: fact.row.key(),
                    phase: fact.source.phase(),
                });
            }
        }
        for fact in &self.tree_width_policies {
            fact.validate()?;
            if self
                .tree_width_policies
                .iter()
                .filter(|candidate| candidate.source.phase() == fact.source.phase())
                .count()
                > 1
            {
                return Err(TargetProfileBuildError::DuplicateWorkgroupTreeWidthPolicy {
                    phase: fact.source.phase(),
                });
            }
        }
        for fact in &self.elementary {
            if self
                .elementary
                .iter()
                .filter(|candidate| *candidate == fact)
                .count()
                > 1
            {
                return Err(TargetProfileBuildError::DuplicateElementaryRealization);
            }
        }
        for fact in &self.subgroup {
            let key = fact.sort_key();
            let same_key: Vec<_> = self
                .subgroup
                .iter()
                .filter(|candidate| candidate.sort_key() == key)
                .collect();
            if same_key.len() <= 1 {
                continue;
            }
            let exact_duplicate = same_key
                .windows(2)
                .any(|pair| pair[0].realization() == pair[1].realization());
            let contradiction = same_key
                .windows(2)
                .any(|pair| pair[0].realization() != pair[1].realization());
            if exact_duplicate || contradiction {
                return Err(TargetProfileBuildError::DuplicateSubgroupRealization);
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
        // Subject, then phase — the complete uniqueness key, excluding the
        // verdict so a contradiction cannot survive as two adjacent rows
        // whose sort order would pick a winner. The complete descriptor
        // encodes this family in the order this sort produces.
        self.synchronization
            .sort_by_key(DeclaredSynchronizationRealization::sort_key);
        // Subject, then licence, then phase — so the rows of one (subject,
        // licence) group are contiguous and phase-ascending, which is what makes
        // `evaluation_order_preservation`'s "latest available phase wins" scan
        // deterministic rather than dependent on declaration order.
        self.evaluation_order
            .sort_by_cached_key(EvaluationOrderFact::subject_key);
        // Row, then phase — so one row's declarations are contiguous and
        // phase-ascending, which is what makes the reader's "latest available
        // phase wins" scan deterministic rather than declaration-order dependent.
        self.cost_rows
            .sort_by_key(|fact| (fact.row, fact.source.phase()));
        self.tree_width_policies
            .sort_by_key(|fact| fact.source.phase());
        // Whole-row canonical encoding, so two profiles that declare the same
        // rows in different insertion orders share one identity, and distinct
        // contracts for one operation stay distinct candidates.
        self.elementary
            .sort_by_cached_key(ElementaryRealization::sort_key);
        // Subject, then phase — the complete uniqueness key, excluding the
        // verdict so a contradiction cannot survive as two adjacent rows
        // whose sort order would pick a winner. The complete descriptor
        // encodes this family in the order this sort produces.
        self.subgroup
            .sort_by_key(DeclaredSubgroupRealization::sort_key);
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
        let checked = CheckedTargetProfile::new_complete(
            identity.clone(),
            self.quantitative.iter().map(fact).collect(),
            self.queries
                .iter()
                .map(|declaration| {
                    CapabilityQuery::new(declaration.axis, declaration.query.clone())
                })
                .collect(),
            honourability,
            self.synchronization
                .iter()
                .map(|declared| declared.clone().attributed_to(identity.clone()))
                .collect(),
            self.subgroup
                .iter()
                .map(|declared| declared.clone().attributed_to(identity.clone()))
                .collect(),
        )
        .map_err(TargetProfileBuildError::from)?;

        let descriptor = complete_descriptor(
            &self.key,
            &self.quantitative,
            &self.queries,
            &self.scalar,
            &self.dispatchability,
            &self.synchronization,
            &self.evaluation_order,
            &self.cost_rows,
            &self.tree_width_policies,
            &self.elementary,
            &self.subgroup,
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
            synchronization: _,
            evaluation_order,
            cost_rows,
            tree_width_policies,
            elementary,
            subgroup: _,
        } = self;
        Ok(TargetProfile {
            data: Arc::new(TargetProfileData {
                key,
                checked,
                quantitative: quantitative.into_boxed_slice(),
                scalar: scalar.into_boxed_slice(),
                dispatchability: dispatchability.into_boxed_slice(),
                evaluation_order: evaluation_order.into_boxed_slice(),
                cost_rows: cost_rows.into_boxed_slice(),
                tree_width_policies: tree_width_policies.into_boxed_slice(),
                elementary: elementary.into_boxed_slice(),
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

/// Public synchronization-declaration disposition.
///
/// Two valued, and the negative is *statable*: a target that has been measured
/// not to provide a realization records that, and a target that was never asked
/// records nothing. Those are different states — a typed rejection and an
/// `Unknown` — and a vocabulary with only a positive spelling would collapse
/// them into one silence.
///
/// There is deliberately no "supported under a relaxation" spelling. A weaker
/// realization is a *different subject*, so a target that provides one declares
/// that subject; letting a caller's subject be satisfied by a neighbouring one
/// is exactly the composition the atomic fact exists to prevent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SynchronizationSupport {
    /// The target realizes exactly the declared subject.
    Realized,
    /// The target explicitly does not realize it.
    Unrealizable,
}

impl SynchronizationSupport {
    const fn realization(self) -> SynchronizationRealization {
        match self {
            Self::Realized => SynchronizationRealization::Realized,
            Self::Unrealizable => SynchronizationRealization::Unrealizable,
        }
    }
}

/// Public subgroup-declaration disposition.
///
/// **Labelled draft** under ADR 0075. Tom accepted the two-valued *shape* on
/// 2026-08-11 — `Realized` and `Unrealizable` are explicit; silence is
/// `Unknown` — and has not accepted this crate's exact type spelling.
///
/// Two valued, and the negative is *statable*: a target that has been measured
/// not to provide a realization records that, and a target that was never asked
/// records nothing. Those are different states — a typed rejection and an
/// `Unknown` — and a vocabulary with only a positive spelling would collapse
/// them into one silence.
///
/// There is deliberately no "supported under a relaxation" spelling. A weaker
/// realization is a *different subject*, so a target that provides one declares
/// that subject; letting a caller's subject be satisfied by a neighbouring one
/// is exactly the composition the atomic fact exists to prevent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubgroupSupport {
    /// The target realizes exactly the declared subject.
    Realized,
    /// The target explicitly does not realize it.
    Unrealizable,
}

impl SubgroupSupport {
    const fn realization(self) -> SubgroupRealization {
        match self {
            Self::Realized => SubgroupRealization::Realized,
            Self::Unrealizable => SubgroupRealization::Unrealizable,
        }
    }
}

/// Result of a subgroup-realization lookup.
///
/// **Labelled draft** under ADR 0075, with [`SubgroupSupport`] and the
/// declaration pair.
///
/// [`Self::Unknown`] is the fail-closed answer and the overwhelmingly common
/// one: a profile that declares nothing about the subject answers it, and a
/// consumer may not read a neighbouring subject's row in its place. There is
/// deliberately no `Deferred` arm: no query vocabulary can ask a device whether
/// it realizes one complete subgroup subject, so a later-phase fact is
/// `Unknown` rather than a promise nothing can keep.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubgroupRealizationResolution {
    /// An exact declaration states the target realizes this subject.
    Realized,
    /// An exact declaration states the target does not realize it.
    Unrealizable,
    /// No exact declaration exists for this subject, or none is available yet.
    Unknown,
}

/// Encodes one complete producer declaration.
///
/// # Why the evaluation-order family did not step [`COMPLETE_PROFILE_DESCRIPTOR_DOMAIN`]
///
/// The rule is a byte rule: the domain steps when previously-encodable bytes
/// **move**, because a reader of the older domain would then be reading the same
/// bytes under a different grammar. The evaluation-order family moves none. It
/// is written last, behind its own separator, and **only when it holds a row**,
/// so every profile assembled before it existed — the governed baseline, the
/// bound macOS Metal declaration, every test profile — encodes byte for byte
/// what it encoded at `v11`. Its sources join the shared source table through an
/// iterator that is empty for those profiles, so no source index shifts either.
///
/// Injectivity survives the conditional section because every earlier section is
/// self-delimiting: two descriptors agreeing on the `v11` prefix agree on every
/// earlier row, and the remainder is then either empty or this family's bytes,
/// which its separator distinguishes from any continuation. An empty family and
/// an absent family denote the same thing here — `Unknown` for every subject and
/// licence, which no admission path can act on differently — so nothing a
/// candidate's admission depends on is lost by not writing a zero count.
///
/// The synchronization family one section above frames itself *unconditionally*
/// and therefore had to step `v10` to `v11`. That was a choice about what
/// silence should record, not a rule this family breaks: `v11` decided that "no
/// synchronization was declared" should be a recorded fact rather than an
/// absence, and paid for the decision by moving every profile's bytes. This
/// family records silence as absence, and pays nothing.
///
/// # The cost-row family takes the same shape, and for a stronger reason
///
/// It is written last, behind its own separator, and only when it holds a row,
/// so it too moves no earlier byte. The reason it *must* is the silence rule the
/// activating ticket's acceptance made testable rather than aspirational: **a
/// profile declaring no cost row selects bit-identically to a build without the
/// family at all.** Selection reads the row, and a profile's canonical descriptor
/// is folded into every artifact identity and cache subject derived from it — so
/// an unconditional section would move every existing profile's identity to
/// record that it still has no preference. Injectivity survives for the reason it
/// survives above: every earlier section is self-delimiting, and this family's
/// separator distinguishes its bytes from any continuation of the last one.
///
/// # The elementary-realization family is the same silence rule, rederived
///
/// It is written last, behind its own separator, and only when it holds a row.
/// An empty family and an absent family both mean no installed realization,
/// which is already what every profile encoded before this family existed —
/// including the governed profile, which does not regain its three Metal rows
/// here. Writing a zero count would move every existing descriptor to record
/// that it still has no elementary row. Injectivity survives for the same
/// reason as the two families above: every earlier section is self-delimiting,
/// and this family's separator distinguishes its bytes from any continuation.
///
/// # The workgroup-tree-width-policy family is the same silence-as-absence
///
/// It is written last, behind its own separator, and only when it holds a row.
/// An empty family and an absent family both mean no accepted policy, which is
/// already what every profile encoded before this family existed. Writing a
/// zero count would move every existing descriptor to record that it still has
/// no policy. Injectivity survives for the same reason as the families above.
/// The owning declaration domain therefore stays at `v11`.
///
/// # The subgroup-realization family is the same silence-as-absence
///
/// It is written last, behind its own separator, and only when it holds a row.
/// An empty family and an absent family both mean `Unknown` for every subgroup
/// subject, which is already what every profile encoded before this family
/// existed — including every standard profile, which stays silent until its
/// own evidence ticket and prepared-entry gate complete. Writing a zero count
/// would move every existing descriptor to record that it still has no
/// subgroup row. Injectivity survives for the same reason as the families
/// above. The owning declaration domain therefore stays at `v11`.
#[allow(
    clippy::too_many_arguments,
    reason = "one parameter per declared row family, threaded explicitly so the encoder reads as the grammar it writes; grouping them behind a struct would put the canonical byte order under two authorities"
)]
fn complete_descriptor(
    key: &TargetProfileKey,
    quantitative: &[QuantitativeCapabilityDeclaration],
    queries: &[QuantitativeCapabilityQueryDeclaration],
    scalar: &[ScalarHonourabilityDeclaration],
    dispatchability: &[DTypeDispatchabilityFact],
    synchronization: &[DeclaredSynchronizationRealization],
    evaluation_order: &[EvaluationOrderFact],
    cost_rows: &[CostRowFact],
    tree_width_policies: &[WorkgroupTreeWidthPolicyFact],
    elementary: &[ElementaryRealization],
    subgroup: &[DeclaredSubgroupRealization],
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
        .chain(
            synchronization
                .iter()
                .map(DeclaredSynchronizationRealization::source_ref),
        )
        .chain(evaluation_order.iter().map(|fact| fact.source.as_ref()))
        .chain(cost_rows.iter().map(|fact| fact.source.as_ref()))
        .chain(tree_width_policies.iter().map(|fact| fact.source.as_ref()))
        .chain(elementary.iter().map(ElementaryRealization::source))
        .chain(subgroup.iter().map(DeclaredSubgroupRealization::source_ref))
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
    // The complete subject and its verdict, in uniqueness-key order
    // `(subject, phase)`. Insertion order is not identity: two profiles that
    // declare the same rows in different sequences encode one descriptor.
    // Every dimension is encoded: two profiles differing only in which memory
    // domain they fence declare different realizations and must not share a
    // descriptor, which is the whole reason the fact is atomic.
    push_slice(&mut bytes, SYNCHRONIZATION_DOMAIN);
    push_len(&mut bytes, synchronization.len());
    for declared in synchronization {
        let subject = declared.subject();
        bytes.push(subject.kind.tag());
        bytes.push(subject.execution_scope.tag());
        bytes.push(subject.visibility_scope.tag());
        bytes.push(u8::from(subject.fenced_spaces.workgroup));
        bytes.push(u8::from(subject.fenced_spaces.device));
        bytes.push(subject.ordering.tag());
        bytes.push(declared.realization().tag());
        let source_bytes = declared.source_ref().canonical_bytes();
        let source_index = sources
            .binary_search_by(|candidate| candidate.0.cmp(&source_bytes))
            .expect("every synchronization source was inserted into the source table");
        encode_compact_index(&mut bytes, source_index);
    }
    // The conditional sections. See this function's header for the derivation
    // that keeps `COMPLETE_PROFILE_DESCRIPTOR_DOMAIN` at `v11`.
    if !evaluation_order.is_empty() {
        push_slice(&mut bytes, EVALUATION_ORDER_DOMAIN);
        push_len(&mut bytes, evaluation_order.len());
        for fact in evaluation_order {
            let source_bytes = fact.source.canonical_bytes();
            let source_index = sources
                .binary_search_by(|candidate| candidate.0.cmp(&source_bytes))
                .expect("every evaluation-order source was inserted into the source table");
            fact.encode(&mut bytes, source_index);
        }
    }
    if !cost_rows.is_empty() {
        push_slice(&mut bytes, COST_ROW_DOMAIN);
        push_len(&mut bytes, cost_rows.len());
        for fact in cost_rows {
            let source_bytes = fact.source.canonical_bytes();
            let source_index = sources
                .binary_search_by(|candidate| candidate.0.cmp(&source_bytes))
                .expect("every cost-row source was inserted into the source table");
            fact.encode(&mut bytes, source_index);
        }
    }
    if !tree_width_policies.is_empty() {
        push_slice(&mut bytes, WORKGROUP_TREE_WIDTH_POLICY_DOMAIN);
        push_len(&mut bytes, tree_width_policies.len());
        for fact in tree_width_policies {
            let source_bytes = fact.source.canonical_bytes();
            let source_index = sources
                .binary_search_by(|candidate| candidate.0.cmp(&source_bytes))
                .expect(
                    "every workgroup-tree-width-policy source was inserted into the source table",
                );
            fact.encode(&mut bytes, source_index);
        }
    }
    if !elementary.is_empty() {
        push_slice(&mut bytes, ELEMENTARY_REALIZATION_DOMAIN);
        push_len(&mut bytes, elementary.len());
        for realization in elementary {
            push_slice(
                &mut bytes,
                realization.contract().canonical_encoding().as_bytes(),
            );
            push_slice(
                &mut bytes,
                &realization.bound_evidence().canonical_encoding(),
            );
            push_slice(
                &mut bytes,
                &realization.exceptional_evidence().canonical_encoding(),
            );
            let source_bytes = realization.source().canonical_bytes();
            let source_index = sources
                .binary_search_by(|candidate| candidate.0.cmp(&source_bytes))
                .expect("every elementary-realization source was inserted into the source table");
            encode_compact_index(&mut bytes, source_index);
        }
    }
    if !subgroup.is_empty() {
        push_slice(&mut bytes, SUBGROUP_REALIZATION_DOMAIN);
        push_len(&mut bytes, subgroup.len());
        for declared in subgroup {
            declared.subject().encode(&mut bytes);
            bytes.push(declared.realization().tag());
            let source_bytes = declared.source_ref().canonical_bytes();
            let source_index = sources
                .binary_search_by(|candidate| candidate.0.cmp(&source_bytes))
                .expect("every subgroup-realization source was inserted into the source table");
            encode_compact_index(&mut bytes, source_index);
        }
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

    /// Test-only synthetic profile that explicitly admits permutation.
    ///
    /// This is not Apple evidence and is never exposed to production request
    /// construction. It exists solely to exercise compiler structure that the
    /// standard live Apple profile correctly refuses.
    #[cfg(test)]
    pub(crate) fn synthetic_permutation_for_test() -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::new("tiler.test.synthetic-permutation.v1".to_owned())
            .expect("the test profile key is valid");
        builder
            .scalar
            .retain(|declaration| declaration.dimension != NumericalDimension::Permutation);
        let source = TargetFactSource(governed_profile_source());
        builder.quantitative.retain(|declaration| {
            !matches!(
                declaration.axis,
                CapabilityAxis::GridAxisThreads | CapabilityAxis::LocalMemoryBytes
            )
        });
        builder
            .declare_max_threads_per_grid_axis(1_024, source.clone())
            .expect("the synthetic grid bound is valid");
        builder
            .declare_local_memory_bytes(4_096, source.clone())
            .expect("the synthetic local-memory bound is valid");
        let synchronization = tiler_ir::schedule::workgroup_tree_tile(2)
            .expect("two participants form a tile")
            .synchronization[0]
            .subject;
        builder
            .declare_synchronization_realization(
                synchronization,
                SynchronizationSupport::Realized,
                &source,
            )
            .expect("the synthetic synchronization row is valid");
        let subject = ScalarArithmetic::f32();
        for permission in [
            NumericalPermission::Forbidden,
            NumericalPermission::Permitted,
        ] {
            builder
                .declare_permutation(
                    subject.clone(),
                    permission,
                    ScalarSupport::Exact,
                    source.clone(),
                )
                .expect("the synthetic permutation row is valid");
        }
        builder.build().expect("the synthetic profile is coherent")
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

    /// Returns the elementary realizations this profile declared, in canonical
    /// row order.
    ///
    /// **Labelled draft** under ADR 0075. A borrowed view of the stored rows.
    /// The slice is empty when the profile declared none, including the
    /// governed profile until a later evidence ticket can discharge both
    /// halves of a Metal row. Assessment reads this view; it does not
    /// reconstruct governed rows from descriptor equality.
    #[must_use]
    pub fn declared_elementary_realizations(&self) -> &[ElementaryRealization] {
        &self.data.elementary
    }

    /// Resolves whether this target realizes one complete subgroup subject.
    ///
    /// **Labelled draft** under ADR 0075, with the declaration pair.
    ///
    /// The match is one equality over the whole subject. A neighbouring
    /// width, arithmetic type, or transfer is `Unknown`, not a partial match.
    /// Silence is `Unknown`. A later-phase fact is `Unknown` rather than
    /// deferred: there is no query contract that could obtain the answer
    /// before routing commits.
    #[must_use]
    pub fn subgroup_realization(
        &self,
        subject: SubgroupRealizationSubject,
        available_phase: AvailabilityPhase,
    ) -> SubgroupRealizationResolution {
        let mut resolved: Option<&crate::target::feasibility::SubgroupRealizationFact> = None;
        for fact in self.data.checked.subgroup() {
            if fact.subject() != subject || fact.phase() > available_phase {
                continue;
            }
            resolved = Some(match resolved {
                Some(current) if current.phase() >= fact.phase() => current,
                _ => fact,
            });
        }
        match resolved {
            None => SubgroupRealizationResolution::Unknown,
            Some(fact) => match fact.realization() {
                crate::target::feasibility::SubgroupRealization::Realized => {
                    SubgroupRealizationResolution::Realized
                }
                crate::target::feasibility::SubgroupRealization::Unrealizable => {
                    SubgroupRealizationResolution::Unrealizable
                }
            },
        }
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

    /// Resolves whether a backend translation under `licence` preserves the
    /// emitted evaluation order for exactly `subject`, preferring the latest
    /// declaration available through `available_phase`.
    ///
    /// **Accepted public surface**, with the declaration constructors.
    ///
    /// Returns [`EvaluationOrderResolution::Unknown`] for a subject or licence
    /// this profile does not speak about, which is every subject and licence of
    /// a profile that declares none. Nothing is inherited: a neighbouring
    /// arithmetic type's row and the other licence's row are both silence here.
    #[must_use]
    pub fn evaluation_order_preservation(
        &self,
        subject: &ScalarArithmetic,
        licence: BackendArithmeticLicence,
        available_phase: AvailabilityPhase,
    ) -> EvaluationOrderResolution {
        let mut now = None;
        let mut later = None;
        for fact in self
            .data
            .evaluation_order
            .iter()
            .filter(|fact| &fact.subject == subject && fact.licence == licence)
        {
            let phase = fact.source.phase();
            if phase <= available_phase {
                now = Some(fact.preservation);
            } else if later.is_none() {
                later = Some(phase);
            }
        }
        match (now, later) {
            (Some(EvaluationOrderPreservation::Preserved), _) => {
                EvaluationOrderResolution::Preserved
            }
            (Some(EvaluationOrderPreservation::NotPreserved), _) => {
                EvaluationOrderResolution::NotPreserved
            }
            (None, Some(available_at)) => EvaluationOrderResolution::Deferred { available_at },
            (None, None) => EvaluationOrderResolution::Unknown,
        }
    }

    /// Resolves how many fold steps this target retires at once when saturated,
    /// preferring the latest declaration available through `available_phase`.
    ///
    /// **Accepted public surface**, accepted by Tom on 2026-08-07 under
    /// `accept-the-measured-cost-row-public-surface`, with the two declaration
    /// constructors.
    ///
    /// Returns [`TargetCostRowResolution::Unknown`] for a profile that declares
    /// nothing, which is every profile but the qualified Apple9 macOS one. That
    /// answer is an absence of preference and never a refusal: a consumer that
    /// treated it as a bound, a zero, or an infeasibility would invert the failure
    /// direction this row exists to avoid.
    #[must_use]
    pub fn saturated_parallel_fold_steps(
        &self,
        available_phase: AvailabilityPhase,
    ) -> TargetCostRowResolution {
        self.cost_row(CostRow::SaturatedParallelFoldSteps, available_phase)
    }

    fn cost_row(
        &self,
        row: CostRow,
        available_phase: AvailabilityPhase,
    ) -> TargetCostRowResolution {
        let mut now = None;
        let mut later = None;
        for fact in self.data.cost_rows.iter().filter(|fact| fact.row == row) {
            let phase = fact.source.phase();
            if phase <= available_phase {
                now = Some(fact.value);
            } else if later.is_none() {
                later = Some(phase);
            }
        }
        match (now, later) {
            (Some(value), _) => TargetCostRowResolution::Declared { value },
            (None, Some(available_at)) => TargetCostRowResolution::Deferred { available_at },
            (None, None) => TargetCostRowResolution::Unknown,
        }
    }

    /// Resolves the closed tree-width policy this target declared, preferring
    /// the latest declaration available through `available_phase`.
    ///
    /// **Accepted public surface**, accepted by Tom's 2026-08-11 delegation
    /// under `gate-the-workgroup-tree-on-an-explicit-qualified-width-policy`.
    ///
    /// Returns [`WorkgroupTreeWidthPolicyResolution::Unknown`] for a profile
    /// that declares nothing. That answer makes the single-workgroup tree
    /// unavailable. It is never a clamp onto `256`, never a substitution of
    /// the balanced partition, and never a preference.
    #[must_use]
    pub fn workgroup_tree_width_policy(
        &self,
        available_phase: AvailabilityPhase,
    ) -> WorkgroupTreeWidthPolicyResolution {
        let mut now = None;
        let mut later = None;
        for fact in &self.data.tree_width_policies {
            let phase = fact.source.phase();
            if phase <= available_phase {
                now = Some(fact.policy);
            } else if later.is_none() {
                later = Some(phase);
            }
        }
        match (now, later) {
            (Some(policy), _) => WorkgroupTreeWidthPolicyResolution::Declared(policy),
            (None, Some(available_at)) => {
                WorkgroupTreeWidthPolicyResolution::Deferred { available_at }
            }
            (None, None) => WorkgroupTreeWidthPolicyResolution::Unknown,
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

    /// Returns the governed profile plus the exact gather-index dispatch row.
    ///
    /// Test-only because this is a diagnostic-layering probe, not a production
    /// claim that the governed target can dispatch integer tensors. Building it
    /// through the ordinary profile builder retains every governed declaration
    /// and recomputes the complete canonical descriptor with the added row.
    #[cfg(test)]
    pub(crate) fn governed_with_gather_index_dispatch_for_test() -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_dtype_dispatchability(
                tiler_ir::semantic::gather_index_resolved_type(),
                DTypeDispatchability::Dispatchable,
                TargetFactSource(governed_profile_source()),
            )
            .expect("the exact gather-index test dispatch declaration is valid");
        builder
            .build()
            .expect("the widened test target profile is intrinsically valid")
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
    pub(crate) fn governed_with_workgroup_limit_for_test(key: &str, limit: u32) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::declared(key.to_owned())
            .expect("the test target-profile key is valid");
        builder
            .queries
            .retain(|query| query.axis != CapabilityAxis::WorkgroupThreads);
        builder
            .declare_max_threads_per_workgroup(limit, TargetFactSource(governed_profile_source()))
            .expect("the test workgroup limit replaces the governed query");
        builder
            .build()
            .expect("the bounded test target profile is intrinsically valid")
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

    /// One exact subnormal/reassociation realization for request-population
    /// tests that must force distinct numerical-contract groups.
    #[cfg(test)]
    pub(crate) fn numerical_realization_for_test(
        key: &str,
        subnormals: SubnormalMode,
        reassociation: NumericalPermission,
    ) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        builder.key = TargetProfileKey::declared(key.to_owned())
            .expect("the test target-profile key is valid");
        builder
            .scalar
            .retain(|declaration| match declaration.dimension {
                NumericalDimension::InputSubnormals | NumericalDimension::ResultSubnormals => {
                    declaration.behaviour == DimensionBehaviour::Subnormals(subnormals)
                }
                NumericalDimension::Contraction => {
                    declaration.behaviour
                        == DimensionBehaviour::Transform(NumericalPermission::Forbidden)
                }
                NumericalDimension::Reassociation => {
                    declaration.behaviour == DimensionBehaviour::Transform(reassociation)
                }
                _ => true,
            });
        builder
            .build()
            .expect("the exact numerical test profile is intrinsically valid")
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

    /// The governed profile widened until a single-workgroup tree is assessable.
    ///
    /// **Deliberately test-only, and that is the finding rather than a
    /// convenience.** `TargetProfileBuilder::governed` declares
    /// `local-memory-bytes` as *zero* and declares nothing at all about
    /// synchronization, so the bounded prototype baseline rejects every
    /// cooperative region twice over — first on threadgroup memory it guarantees
    /// none of, then on a realization it has never been asked about. Both
    /// refusals are correct and both are exercised by their own tests; what they
    /// mean is that the baseline cannot be the profile a synchronized strategy is
    /// *admitted* against.
    ///
    /// Raising the baseline's own rows instead would be a capability claim: the
    /// prototype authority has no evidence for a threadgroup-memory budget or a
    /// barrier realization, and inventing one would promote support from a test's
    /// convenience — precisely what the atomic synchronization fact exists to
    /// prevent. `realize-parallel-reduction-strategies-on-metal` owns the real
    /// declaration, and the question of what the *prototype baseline* should
    /// guarantee is Tom's, not this ticket's.
    ///
    /// `synchronization` is `None` for a profile that has never been asked, which
    /// is what makes the undeclared rejection drivable separately from a declared
    /// refusal.
    #[cfg(test)]
    pub(crate) fn workgroup_tree_target_for_test(
        local_memory_bytes: u64,
        grid_axis_threads: u64,
        synchronization: Option<SynchronizationSupport>,
    ) -> Self {
        Self::workgroup_tree_target_with_cost_row_for_test(
            local_memory_bytes,
            grid_axis_threads,
            synchronization,
            None,
        )
    }

    /// The same widened test profile, optionally carrying the measured cost row.
    ///
    /// `None` and `Some` are the two halves of the silence rule the activating
    /// ticket's acceptance made testable: a profile declaring no row must select
    /// bit-identically to one built before the row existed, and this is the
    /// constructor that lets one compile drive both sides with nothing else
    /// varying.
    #[cfg(test)]
    pub(crate) fn workgroup_tree_target_with_cost_row_for_test(
        local_memory_bytes: u64,
        grid_axis_threads: u64,
        synchronization: Option<SynchronizationSupport>,
        saturated_parallel_fold_steps: Option<u64>,
    ) -> Self {
        Self::workgroup_tree_target_parts_for_test(
            local_memory_bytes,
            grid_axis_threads,
            synchronization,
            saturated_parallel_fold_steps,
            Some(WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1),
        )
    }

    /// The same widened test profile with no tree-width policy declared.
    ///
    /// The negative half of the policy gate: omission makes the tree unavailable
    /// and must not substitute `256` or `governed_partition`.
    #[cfg(test)]
    pub(crate) fn workgroup_tree_target_without_width_policy_for_test(
        local_memory_bytes: u64,
        grid_axis_threads: u64,
        synchronization: Option<SynchronizationSupport>,
    ) -> Self {
        Self::workgroup_tree_target_parts_for_test(
            local_memory_bytes,
            grid_axis_threads,
            synchronization,
            None,
            None,
        )
    }

    #[cfg(test)]
    fn workgroup_tree_target_parts_for_test(
        local_memory_bytes: u64,
        grid_axis_threads: u64,
        synchronization: Option<SynchronizationSupport>,
        saturated_parallel_fold_steps: Option<u64>,
        width_policy: Option<WorkgroupTreeWidthPolicy>,
    ) -> Self {
        let mut builder = TargetProfileBuilder::governed();
        if let Some(policy) = width_policy {
            builder
                .declare_workgroup_tree_width_policy(
                    policy,
                    TargetFactSource(governed_profile_source()),
                )
                .expect("the test tree-width-policy declaration is valid");
        }
        if let Some(steps) = saturated_parallel_fold_steps {
            builder
                .declare_saturated_parallel_fold_steps(
                    steps,
                    TargetFactSource(governed_profile_source()),
                )
                .expect("the test cost-row declaration is valid");
        }
        for (axis, bound) in [
            (CapabilityAxis::LocalMemoryBytes, local_memory_bytes),
            (CapabilityAxis::GridAxisThreads, grid_axis_threads),
        ] {
            builder
                .quantitative
                .iter_mut()
                .find(|declaration| declaration.axis == axis)
                .expect("the governed profile declares this axis")
                .bound = bound;
        }
        if let Some(support) = synchronization {
            // Derived from the canonical tile's own edges rather than restated,
            // so a test profile cannot declare a realization the strategy does
            // not require and then "admit" it.
            let tile = tiler_ir::schedule::workgroup_tree_tile(2)
                .expect("two participants are within the enumeration bound");
            let subject = tiler_ir::schedule::required_subject(&tile.visibility_edges())
                .expect("the canonical tree tile carries one handoff");
            builder
                .declare_synchronization_realization(
                    subject,
                    support,
                    &TargetFactSource(governed_profile_source()),
                )
                .expect("the test synchronization declaration is valid");
        }
        builder
            .build()
            .expect("the widened test target profile is intrinsically valid")
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
    /// The same synchronization subject was declared twice at one phase.
    ///
    /// The verdict is deliberately not part of that key: a profile declaring one
    /// subject both realized and unrealizable has stated a contradiction, and
    /// admitting both rows would leave whichever the sort put first deciding.
    DuplicateSynchronizationRealization,
    /// A declared synchronization subject fences no memory domain.
    ///
    /// A fence over nothing publishes nothing, so no handoff could consume it and
    /// a realization of one would be a permission for an operation with no effect.
    VacuousSynchronizationSubject,
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
    /// The same scalar subject and backend licence received two
    /// evaluation-order verdicts at one availability phase.
    ///
    /// The verdict is deliberately not part of that key, for the reason
    /// [`Self::DuplicateSynchronizationRealization`] excludes it: a profile
    /// declaring one subject both preserved and not preserved has stated a
    /// contradiction, and admitting both rows would leave whichever the sort put
    /// first deciding.
    DuplicateEvaluationOrderPreservation {
        /// Stable governed key of the licence both rows claimed.
        licence: &'static str,
        /// Availability phase at which both rows claimed authority.
        phase: AvailabilityPhase,
    },
    /// The same measured cost row was declared twice at one availability phase.
    ///
    /// The value is deliberately not part of that key, for the reason
    /// [`Self::DuplicateQuantitativeCapability`] excludes its bound: a profile
    /// stating one machine quantity twice has stated a contradiction, and
    /// admitting both rows would leave whichever the sort put first deciding.
    DuplicateCostRow {
        /// Stable governed key of the row both declarations claimed.
        row: &'static str,
        /// Availability phase at which both rows claimed authority.
        phase: AvailabilityPhase,
    },
    /// The same workgroup-tree-width policy phase was declared twice.
    ///
    /// The variant is deliberately not part of that key: a profile stating two
    /// policies at one phase has stated a contradiction, and admitting both
    /// would leave whichever the sort put first deciding. One variant exists
    /// today, so the refusal is a restatement; a second variant would still
    /// refuse at the same phase rather than encode a choice.
    DuplicateWorkgroupTreeWidthPolicy {
        /// Availability phase at which both rows claimed authority.
        phase: AvailabilityPhase,
    },
    /// The same complete elementary-realization row was declared twice.
    ///
    /// Distinct contracts for one operation remain legal. Only an exact
    /// restatement of the verified contract, both evidence records, and the
    /// source is rejected. No row is replaced, merged, or preferred.
    DuplicateElementaryRealization,
    /// The same subgroup subject was declared twice at one phase.
    ///
    /// The verdict is deliberately not part of that key: a profile declaring one
    /// subject both realized and unrealizable has stated a contradiction, and
    /// admitting both rows would leave whichever the sort put first deciding.
    DuplicateSubgroupRealization,
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
    use crate::target::feasibility::{
        AvailabilityPhase, AxisRequirement, FactAuthority, FactValidityScope, FeasibilityOutcome,
        FeasibilityProposal,
    };
    use crate::target::honourability::{
        CANONICAL_DIMENSIONS, CompilerBuildIdentity, CompilerBuildRole,
        ExecutionEnvironmentIdentity, MeasurementContext, NumericalRequirement, ProvenanceIdentity,
    };
    use tiler_ir::numerics::{
        RelaxationRequirement, ScalarArithmeticSubjectError, ScalarArithmeticSubjectIdentity,
        registered_arithmetic_facts, registered_scalar_format,
    };
    use tiler_ir::schedule::{
        ApproximationEnvelope, ArithmeticType, FencedSpaces, MaterializationRounding,
        MemoryOrdering, SubgroupRealizationSubject, SubgroupTransfer, SubgroupWidth,
        SynchronizationKind, SynchronizationScope,
    };
    use tiler_ir::semantic::{
        CanonicalValue, TypeArguments, TypeKey, builtin_scalar_value_type_facts,
        builtin_scalar_value_types,
    };

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

    /// The governed identity `arithmetic` names, as the catalog spells it.
    fn governed_scalar(name: &str) -> ResolvedValueType {
        ResolvedValueType::nominal(TypeKey::new("tiler", name, 1).unwrap())
    }

    /// Every arithmetic type constructs a subject over its own governed identity.
    ///
    /// All four, not BF16 alone: a route that admitted one named dtype would be
    /// the widened equality check under another name, and the point of deriving
    /// admissibility from the catalog is that no dtype is special-cased in it.
    #[test]
    fn every_arithmetic_type_constructs_a_subject_over_its_own_governed_identity() {
        for (arithmetic, name) in [
            (ArithmeticType::F16, "f16"),
            (ArithmeticType::Bf16, "bf16"),
            (ArithmeticType::F32, "f32"),
            (ArithmeticType::F64, "f64"),
        ] {
            let resolved_type = governed_scalar(name);
            let subject = ScalarArithmetic::new(arithmetic, resolved_type.clone())
                .unwrap_or_else(|error| panic!("{name} is a governed identity: {error}"));
            assert_eq!(subject.arithmetic(), arithmetic);
            assert_eq!(subject.resolved_type(), &resolved_type);
        }
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::F32, F32::resolved_type()),
            Ok(ScalarArithmetic::f32()),
            "the governed F32 subject keeps the exact pair every existing profile names",
        );
    }

    /// Each way a pair can fail the catalog is refused, and none is a build error.
    ///
    /// The width and class cases are chosen so that exactly one field
    /// disagrees: `tiler::f16@1` shares `f32`'s `ieee-binary` class and states a
    /// different width, and `tiler::u32@1` states `f32`'s width and a different
    /// class. A rule reading only one of the two fields would admit one of them.
    #[test]
    fn a_pair_the_catalog_does_not_back_is_refused_for_a_stated_reason() {
        let refused = Err(ScalarArithmeticSubjectError::UnvalidatedScalarArithmetic);

        // An unregistered identity, against every arithmetic type: a `test`
        // namespace is not the governed catalog however the name is spelled.
        for arithmetic in ArithmeticType::ALL {
            for name in ["f16", "bf16", "f32", "f64", "u4"] {
                assert_eq!(
                    ScalarArithmetic::new(arithmetic, nominal(name)),
                    refused,
                    "test::{name}@1 is not a registered governed identity",
                );
            }
        }

        // A registered identity whose stated width disagrees with the
        // arithmetic type's.
        let f32_width = registered_scalar_format(
            &registered_arithmetic_facts(ArithmeticType::F32).expect("f32 is governed"),
        )
        .expect("the governed f32 row states a format")
        .1;
        let f16_width = registered_scalar_format(
            &builtin_scalar_value_type_facts(&governed_scalar("f16")).expect("f16 is governed"),
        )
        .expect("the governed f16 row states a format")
        .1;
        assert_eq!((f32_width, f16_width), (32, 16));
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::F32, governed_scalar("f16")),
            refused,
        );
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::F64, governed_scalar("f32")),
            refused,
        );

        // A registered identity of the arithmetic type's exact width whose
        // format class is another family's.
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::F32, governed_scalar("u32")),
            refused,
        );
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::F32, governed_scalar("decimal32")),
            refused,
        );
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::Bf16, governed_scalar("f16")),
            refused,
            "bf16 and f16 share a width and differ in class",
        );

        // A registered identity whose descriptor states no width at all.
        assert_eq!(
            ScalarArithmetic::new(ArithmeticType::F16, governed_scalar("bool")),
            refused,
        );

        // An identity that is not a nominal scalar row.
        let complex = ResolvedValueType::parameterized(
            TypeKey::new("tiler", "complex", 1).unwrap(),
            TypeArguments::new([CanonicalValue::value_type(F32::resolved_type())]).unwrap(),
        )
        .unwrap();
        assert_eq!(ScalarArithmetic::new(ArithmeticType::F32, complex), refused);
    }

    /// A format is unique to one governed identity, over the whole catalog.
    ///
    /// This is the invariant that lets admissibility be decided on class and
    /// width: were two governed rows to share both, each would be constructible
    /// as the other's arithmetic subject. Nothing in `tiler-ir` promises that,
    /// so it is counted here rather than assumed, and a catalog row added with a
    /// colliding format fails this test instead of silently widening a subject.
    #[test]
    fn every_arithmetic_type_names_exactly_one_governed_format() {
        for arithmetic in ArithmeticType::ALL {
            let facts = registered_arithmetic_facts(arithmetic)
                .unwrap_or_else(|| panic!("{} is registered", arithmetic.canonical_type_key()));
            let format = registered_scalar_format(&facts)
                .unwrap_or_else(|| panic!("{} states a format", arithmetic.canonical_type_key()));
            let sharing: Vec<_> = builtin_scalar_value_types()
                .into_iter()
                .filter(|value| {
                    builtin_scalar_value_type_facts(value)
                        .is_some_and(|facts| registered_scalar_format(&facts) == Some(format))
                })
                .filter_map(|value| value.nominal_key().map(TypeKey::to_string))
                .collect();
            assert_eq!(
                sharing,
                vec![arithmetic.canonical_type_key().to_owned()],
                "{} shares its format class and width with another governed identity",
                arithmetic.canonical_type_key(),
            );
        }
    }

    /// Constructing a subject declares nothing about it.
    ///
    /// A profile carrying the complete governed F32 declaration says nothing
    /// about BF16, and the fail-closed clause applies to the subject coordinate
    /// exactly as it does to the dimension one. Every dimension is required in
    /// one proposal and the undeclared set is counted, so a resolution that
    /// answered some of them would not be mistaken for silence about all.
    #[test]
    fn a_constructed_subject_no_profile_declares_is_unknown_on_every_dimension() {
        let subject = ScalarArithmetic::new(ArithmeticType::Bf16, governed_scalar("bf16"))
            .expect("bf16 is a governed identity");
        let behaviour = |dimension| match dimension {
            NumericalDimension::InputSubnormals | NumericalDimension::ResultSubnormals => {
                DimensionBehaviour::Subnormals(SubnormalMode::Preserve)
            }
            NumericalDimension::Contraction
            | NumericalDimension::Reassociation
            | NumericalDimension::Permutation
            | NumericalDimension::SignedZero
            | NumericalDimension::ReciprocalTransform => {
                DimensionBehaviour::Transform(NumericalPermission::Forbidden)
            }
            NumericalDimension::ApproximateIntrinsics => {
                DimensionBehaviour::Approximation(ApproximationEnvelope::Forbidden)
            }
            NumericalDimension::NanAssumptions | NumericalDimension::InfinityAssumptions => {
                DimensionBehaviour::ExceptionalValue(ExceptionalValueAssumption::MakeNoAssumption)
            }
            NumericalDimension::MaterializationRounding => {
                DimensionBehaviour::Rounding(MaterializationRounding::NearestTiesToEven)
            }
        };
        let proposal = FeasibilityProposal::new(
            "undeclared-bf16-subject",
            Vec::new(),
            CANONICAL_DIMENSIONS
                .iter()
                .map(|dimension| {
                    NumericalRequirement::new(
                        *dimension,
                        subject.arithmetic(),
                        subject.resolved_type().clone(),
                        behaviour(*dimension),
                    )
                })
                .collect(),
        )
        .unwrap();
        let profile = TargetProfileBuilder::governed().try_build().unwrap();
        let FeasibilityOutcome::Unknown(unknown) = profile
            .checked()
            .assess(&proposal, AvailabilityPhase::CompileProfile)
        else {
            panic!("a profile declaring only F32 rows answers nothing about BF16");
        };
        let undeclared: Vec<_> = unknown
            .dimensions()
            .iter()
            .map(|dimension| {
                assert_eq!(dimension.arithmetic(), ArithmeticType::Bf16);
                assert_eq!(dimension.resolved_type(), subject.resolved_type());
                dimension.dimension()
            })
            .collect();
        assert_eq!(undeclared, CANONICAL_DIMENSIONS);
    }

    #[test]
    fn scalar_declarations_reject_invalid_behaviour_relaxation_and_exact_emulation() {
        let subject = ScalarArithmetic::f32();
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
                        ScalarArithmeticSubjectIdentity::from_parts(
                            ArithmeticType::F64,
                            nominal("future-f64").canonical_encoding().as_bytes(),
                        )
                        .expect("a nominal identity is well formed"),
                        NumericalDimension::Contraction,
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
        // Two *governed* subjects, because the relocated validator refuses a
        // subject over an unregistered type. Holding the arithmetic fixed while
        // varying the resolved type is no longer constructible at all — the
        // catalog admits exactly one value identity per arithmetic type — which
        // is a stronger guarantee than this test used to assert against a
        // hand-built pair.
        for subject in [
            ScalarArithmetic::f32(),
            ScalarArithmetic::new(ArithmeticType::F16, governed_scalar("f16"))
                .expect("the catalog registers f16 over tiler::f16@1"),
        ] {
            builder.scalar.push(ScalarHonourabilityDeclaration {
                subject,
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
            crate::target::honourability::FactEvidenceBasis::ExternalGuarantee { .. }
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
            crate::target::honourability::FactEvidenceBasis::Measurement { contexts }
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
                source.clone(),
            )
            .unwrap();
        builder
            .declare_elementary_realization(ElementaryRealization::measured(
                &verified_silu_contract(),
                discharging_evidence(
                    "measured-family bound half",
                    b"fixture:measured-family-bound-v1",
                ),
                discharging_evidence(
                    "measured-family exceptional half",
                    b"fixture:measured-family-exceptional-v1",
                ),
                &source,
            ))
            .unwrap();
        builder
            .declare_measured_subgroup_realization(
                subgroup_subject(32, ArithmeticType::F32),
                SubgroupSupport::Realized,
                source.clone(),
            )
            .unwrap();

        assert_eq!(builder.quantitative.len(), 7);
        assert_eq!(builder.scalar.len(), 15);
        assert_eq!(builder.dispatchability.len(), 1);
        assert_eq!(builder.elementary.len(), 1);
        assert_eq!(builder.subgroup.len(), 1);
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
            .chain(builder.elementary.iter().map(ElementaryRealization::source))
            .chain(
                builder
                    .subgroup
                    .iter()
                    .map(DeclaredSubgroupRealization::source_ref),
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

    fn atomic_realization_subject(kind: SynchronizationKind) -> SynchronizationSubject {
        SynchronizationSubject {
            kind,
            execution_scope: SynchronizationScope::Workgroup,
            visibility_scope: SynchronizationScope::Workgroup,
            fenced_spaces: FencedSpaces {
                workgroup: true,
                device: false,
            },
            ordering: MemoryOrdering::AcquireRelease,
        }
    }

    fn atomic_realization_neighbour() -> SynchronizationSubject {
        SynchronizationSubject {
            fenced_spaces: FencedSpaces {
                workgroup: true,
                device: true,
            },
            ..atomic_realization_subject(SynchronizationKind::ControlBarrier)
        }
    }

    fn declare_atomic_pair(
        key: &str,
        first: &(
            SynchronizationSubject,
            SynchronizationSupport,
            TargetFactSource,
        ),
        second: &(
            SynchronizationSubject,
            SynchronizationSupport,
            TargetFactSource,
        ),
    ) -> TargetProfile {
        let mut builder = TargetProfileBuilder::new(TargetProfileKey::new(key.to_owned()).unwrap());
        builder
            .declare_synchronization_realization(first.0, first.1, &first.2)
            .unwrap();
        builder
            .declare_synchronization_realization(second.0, second.1, &second.2)
            .unwrap();
        builder.build().unwrap()
    }

    /// Insertion order is not identity for the atomic realization family.
    ///
    /// Two profiles that declare the same two rows in opposite order share one
    /// complete descriptor and one checked descriptor, and the stored
    /// population is uniqueness-key order — `(subject, phase)` — not
    /// declaration order.
    #[test]
    fn atomic_realization_insertion_order_is_not_identity() {
        let source = public_external_source(1);
        let control = atomic_realization_subject(SynchronizationKind::ControlBarrier);
        let collective = atomic_realization_subject(SynchronizationKind::Collective);
        assert!(control < collective, "the fixture relies on kind order");
        let forward = declare_atomic_pair(
            "test.atomic-order.v1",
            &(control, SynchronizationSupport::Realized, source.clone()),
            &(
                collective,
                SynchronizationSupport::Unrealizable,
                source.clone(),
            ),
        );
        let reverse = declare_atomic_pair(
            "test.atomic-order.v1",
            &(
                collective,
                SynchronizationSupport::Unrealizable,
                source.clone(),
            ),
            &(control, SynchronizationSupport::Realized, source),
        );
        assert_eq!(
            forward.canonical_descriptor(),
            reverse.canonical_descriptor()
        );
        assert_eq!(
            forward.checked().canonical_descriptor(),
            reverse.checked().canonical_descriptor()
        );
        for profile in [&forward, &reverse] {
            let subjects: Vec<_> = profile
                .checked()
                .synchronization()
                .iter()
                .map(crate::target::feasibility::SynchronizationRealizationFact::subject)
                .collect();
            assert_eq!(subjects, [control, collective]);
        }
    }

    #[test]
    fn an_exact_duplicate_atomic_realization_is_refused_before_insertion() {
        let source = public_external_source(1);
        let subject = atomic_realization_subject(SynchronizationKind::ControlBarrier);
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.atomic-duplicate.v1".to_owned()).unwrap(),
        );
        builder
            .declare_synchronization_realization(subject, SynchronizationSupport::Realized, &source)
            .unwrap();
        let len = builder.synchronization.len();
        assert_eq!(
            builder.declare_synchronization_realization(
                subject,
                SynchronizationSupport::Realized,
                &source,
            ),
            Err(TargetProfileBuildError::DuplicateSynchronizationRealization)
        );
        assert_eq!(builder.synchronization.len(), len);
    }

    #[test]
    fn a_contradictory_atomic_realization_verdict_is_refused_before_insertion() {
        let source = public_external_source(1);
        let subject = atomic_realization_subject(SynchronizationKind::ControlBarrier);
        for (first, second) in [
            (
                SynchronizationSupport::Realized,
                SynchronizationSupport::Unrealizable,
            ),
            (
                SynchronizationSupport::Unrealizable,
                SynchronizationSupport::Realized,
            ),
        ] {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.atomic-contradiction.v1".to_owned()).unwrap(),
            );
            builder
                .declare_synchronization_realization(subject, first, &source)
                .unwrap();
            let len = builder.synchronization.len();
            assert_eq!(
                builder.declare_synchronization_realization(subject, second, &source),
                Err(TargetProfileBuildError::DuplicateSynchronizationRealization),
                "sort order must not choose a winner between {first:?} then {second:?}"
            );
            assert_eq!(builder.synchronization.len(), len);
        }
    }

    /// Freeze-time validation refuses both cases even when insert-time is
    /// bypassed, so a mutated draft cannot encode a contradiction.
    #[test]
    fn freeze_refuses_duplicate_and_contradictory_atomic_realizations_independently() {
        let source = public_external_source(1).provenance();
        let subject = atomic_realization_subject(SynchronizationKind::ControlBarrier);
        let realized = DeclaredSynchronizationRealization::new(
            subject,
            SynchronizationRealization::Realized,
            Arc::clone(&source),
        );
        let unrealizable = DeclaredSynchronizationRealization::new(
            subject,
            SynchronizationRealization::Unrealizable,
            source,
        );
        for rows in [
            vec![realized.clone(), realized.clone()],
            vec![realized, unrealizable],
        ] {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.atomic-freeze.v1".to_owned()).unwrap(),
            );
            builder.synchronization = rows;
            assert_eq!(
                builder.try_build(),
                Err(TargetProfileBuildError::DuplicateSynchronizationRealization)
            );
        }
    }

    /// Distinct phases of one subject coexist, and declaring the later phase
    /// first does not move either descriptor.
    #[test]
    fn atomic_realization_phase_is_part_of_the_uniqueness_key_and_not_insertion_order() {
        let compile = public_external_source(1);
        let later = device_runtime_source();
        let subject = atomic_realization_subject(SynchronizationKind::ControlBarrier);
        let forward = declare_atomic_pair(
            "test.atomic-phase.v1",
            &(subject, SynchronizationSupport::Realized, compile.clone()),
            &(subject, SynchronizationSupport::Unrealizable, later.clone()),
        );
        let reverse = declare_atomic_pair(
            "test.atomic-phase.v1",
            &(subject, SynchronizationSupport::Unrealizable, later),
            &(subject, SynchronizationSupport::Realized, compile),
        );
        assert_eq!(
            forward.canonical_descriptor(),
            reverse.canonical_descriptor()
        );
        assert_eq!(
            forward.checked().canonical_descriptor(),
            reverse.checked().canonical_descriptor()
        );
        for profile in [&forward, &reverse] {
            let phases: Vec<_> = profile
                .checked()
                .synchronization()
                .iter()
                .map(crate::target::feasibility::SynchronizationRealizationFact::phase)
                .collect();
            assert_eq!(
                phases,
                [
                    AvailabilityPhase::CompileProfile,
                    AvailabilityPhase::LiveDevicePreflight
                ]
            );
        }
    }

    /// Source is identity-bearing in the complete declaration and not a
    /// uniqueness-key component: two sources at one `(subject, phase)` refuse,
    /// and two profiles that differ only in source revision do not share a
    /// complete descriptor.
    #[test]
    fn atomic_realization_source_participates_in_complete_identity_independently() {
        let subject = atomic_realization_subject(SynchronizationKind::ControlBarrier);
        let first = public_external_source(1);
        let second = public_external_source(2);
        let mut colliding = TargetProfileBuilder::new(
            TargetProfileKey::new("test.atomic-source.v1".to_owned()).unwrap(),
        );
        colliding
            .declare_synchronization_realization(subject, SynchronizationSupport::Realized, &first)
            .unwrap();
        assert_eq!(
            colliding.declare_synchronization_realization(
                subject,
                SynchronizationSupport::Realized,
                &second,
            ),
            Err(TargetProfileBuildError::DuplicateSynchronizationRealization)
        );

        let descriptor = |source: TargetFactSource| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.atomic-source.v1".to_owned()).unwrap(),
            );
            builder
                .declare_synchronization_realization(
                    subject,
                    SynchronizationSupport::Realized,
                    &source,
                )
                .unwrap();
            builder.build().unwrap()
        };
        let left = descriptor(first);
        let right = descriptor(second);
        assert_ne!(
            left.canonical_descriptor(),
            right.canonical_descriptor(),
            "a source-revision change must move the complete declaration"
        );
        assert_eq!(
            left.checked().canonical_descriptor(),
            right.checked().canonical_descriptor(),
            "the checked descriptor encodes phase, authority, and validity, not the source identity"
        );
    }

    /// Every dimension of the subject, and the verdict, participates in both
    /// descriptors. A neighbouring subject is a different row, not a
    /// restatement.
    #[test]
    fn atomic_realization_subject_and_verdict_participate_in_identity_independently() {
        let source = public_external_source(1);
        let baseline_subject = atomic_realization_subject(SynchronizationKind::ControlBarrier);
        let descriptor = |subject, support| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.atomic-subject.v1".to_owned()).unwrap(),
            );
            builder
                .declare_synchronization_realization(subject, support, &source)
                .unwrap();
            builder.build().unwrap()
        };
        let realized = descriptor(baseline_subject, SynchronizationSupport::Realized);
        let refused = descriptor(baseline_subject, SynchronizationSupport::Unrealizable);
        assert_ne!(
            realized.canonical_descriptor(),
            refused.canonical_descriptor()
        );
        assert_ne!(
            realized.checked().canonical_descriptor(),
            refused.checked().canonical_descriptor()
        );
        let neighbour = descriptor(
            atomic_realization_neighbour(),
            SynchronizationSupport::Realized,
        );
        assert_ne!(
            realized.canonical_descriptor(),
            neighbour.canonical_descriptor()
        );
        assert_ne!(
            realized.checked().canonical_descriptor(),
            neighbour.checked().canonical_descriptor()
        );
        let collective = descriptor(
            atomic_realization_subject(SynchronizationKind::Collective),
            SynchronizationSupport::Realized,
        );
        assert_ne!(
            realized.canonical_descriptor(),
            collective.canonical_descriptor()
        );
    }

    fn subgroup_width(lanes: u32) -> SubgroupWidth {
        SubgroupWidth::new(lanes).expect("nonzero width")
    }

    fn subgroup_subject(lanes: u32, arithmetic: ArithmeticType) -> SubgroupRealizationSubject {
        SubgroupRealizationSubject::new(
            subgroup_width(lanes),
            arithmetic,
            SubgroupTransfer::InRangeXorShuffle,
        )
        .expect("power-of-two width at least 2 defines an XOR shuffle")
    }

    /// Silence is `Unknown` for every subject, and it costs a profile that
    /// declares nothing not one descriptor byte.
    #[test]
    fn a_profile_declaring_no_subgroup_row_resolves_unknown() {
        let required = subgroup_subject(32, ArithmeticType::F32);
        for profile in [
            TargetProfile::governed(),
            public_builder("acme.silent-subgroup.v1")
                .try_build()
                .unwrap(),
        ] {
            assert_eq!(
                profile.subgroup_realization(required, AvailabilityPhase::LaunchPreflight),
                SubgroupRealizationResolution::Unknown,
            );
            assert!(
                !profile
                    .canonical_descriptor()
                    .windows(SUBGROUP_REALIZATION_DOMAIN.len())
                    .any(|window| window == SUBGROUP_REALIZATION_DOMAIN),
                "an undeclaring profile writes none of the family's bytes, which is \
                 why the complete-declaration domain did not step"
            );
        }
    }

    #[test]
    fn declared_subgroup_rows_resolve_by_whole_subject_equality() {
        let source = public_external_source(1);
        let required = subgroup_subject(32, ArithmeticType::F32);
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-realized.v1".to_owned()).unwrap(),
        );
        builder
            .declare_subgroup_realization(required, SubgroupSupport::Realized, source)
            .unwrap();
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.subgroup_realization(required, AvailabilityPhase::CompileProfile),
            SubgroupRealizationResolution::Realized
        );
        assert_eq!(
            profile.subgroup_realization(
                subgroup_subject(64, ArithmeticType::F32),
                AvailabilityPhase::CompileProfile,
            ),
            SubgroupRealizationResolution::Unknown,
            "a neighbouring width must not satisfy the required subject"
        );
        assert_eq!(
            profile.subgroup_realization(
                subgroup_subject(32, ArithmeticType::Bf16),
                AvailabilityPhase::CompileProfile,
            ),
            SubgroupRealizationResolution::Unknown,
            "a neighbouring arithmetic type must not satisfy the required subject"
        );
        assert!(
            profile
                .canonical_descriptor()
                .windows(SUBGROUP_REALIZATION_DOMAIN.len())
                .any(|window| window == SUBGROUP_REALIZATION_DOMAIN)
        );
    }

    #[test]
    fn a_declared_unrealizable_subgroup_is_explicit() {
        let source = public_external_source(1);
        let required = subgroup_subject(32, ArithmeticType::F32);
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-unrealizable.v1".to_owned()).unwrap(),
        );
        builder
            .declare_subgroup_realization(required, SubgroupSupport::Unrealizable, source)
            .unwrap();
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.subgroup_realization(required, AvailabilityPhase::CompileProfile),
            SubgroupRealizationResolution::Unrealizable
        );
    }

    #[test]
    fn a_later_phase_subgroup_row_is_unknown_rather_than_deferred() {
        let required = subgroup_subject(32, ArithmeticType::F32);
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-later.v1".to_owned()).unwrap(),
        );
        builder
            .declare_subgroup_realization(
                required,
                SubgroupSupport::Realized,
                device_runtime_source(),
            )
            .unwrap();
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.subgroup_realization(required, AvailabilityPhase::CompileProfile),
            SubgroupRealizationResolution::Unknown,
        );
        assert_eq!(
            profile.subgroup_realization(required, AvailabilityPhase::LiveDevicePreflight),
            SubgroupRealizationResolution::Realized,
        );
    }

    #[test]
    fn an_exact_duplicate_subgroup_realization_is_refused_before_insertion() {
        let source = public_external_source(1);
        let required = subgroup_subject(32, ArithmeticType::F32);
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-duplicate.v1".to_owned()).unwrap(),
        );
        builder
            .declare_subgroup_realization(required, SubgroupSupport::Realized, source.clone())
            .unwrap();
        let len = builder.subgroup.len();
        assert_eq!(
            builder.declare_subgroup_realization(required, SubgroupSupport::Realized, source),
            Err(TargetProfileBuildError::DuplicateSubgroupRealization)
        );
        assert_eq!(builder.subgroup.len(), len);
    }

    #[test]
    fn a_contradictory_subgroup_verdict_is_refused_before_insertion() {
        let source = public_external_source(1);
        let required = subgroup_subject(32, ArithmeticType::F32);
        for (first, second) in [
            (SubgroupSupport::Realized, SubgroupSupport::Unrealizable),
            (SubgroupSupport::Unrealizable, SubgroupSupport::Realized),
        ] {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.subgroup-contradiction.v1".to_owned()).unwrap(),
            );
            builder
                .declare_subgroup_realization(required, first, source.clone())
                .unwrap();
            let len = builder.subgroup.len();
            assert_eq!(
                builder.declare_subgroup_realization(required, second, source.clone()),
                Err(TargetProfileBuildError::DuplicateSubgroupRealization),
                "sort order must not choose a winner between {first:?} then {second:?}"
            );
            assert_eq!(builder.subgroup.len(), len);
        }
    }

    #[test]
    fn subgroup_insertion_order_is_not_identity() {
        let source = public_external_source(1);
        let first = subgroup_subject(32, ArithmeticType::F32);
        let second = subgroup_subject(64, ArithmeticType::F32);
        assert!(first < second, "the fixture relies on subject order");
        let declare = |rows: [(SubgroupRealizationSubject, SubgroupSupport); 2]| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.subgroup-order.v1".to_owned()).unwrap(),
            );
            for (subject, support) in rows {
                builder
                    .declare_subgroup_realization(subject, support, source.clone())
                    .unwrap();
            }
            builder.try_build().unwrap()
        };
        let forward = declare([
            (first, SubgroupSupport::Realized),
            (second, SubgroupSupport::Unrealizable),
        ]);
        let reverse = declare([
            (second, SubgroupSupport::Unrealizable),
            (first, SubgroupSupport::Realized),
        ]);
        assert_eq!(
            forward.canonical_descriptor(),
            reverse.canonical_descriptor()
        );
        assert_eq!(
            forward.checked().canonical_descriptor(),
            reverse.checked().canonical_descriptor()
        );
    }

    #[test]
    fn subgroup_source_participates_in_complete_identity_independently() {
        let required = subgroup_subject(32, ArithmeticType::F32);
        let first = public_external_source(1);
        let second = public_external_source(2);
        let mut colliding = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-source.v1".to_owned()).unwrap(),
        );
        colliding
            .declare_subgroup_realization(required, SubgroupSupport::Realized, first.clone())
            .unwrap();
        assert_eq!(
            colliding.declare_subgroup_realization(
                required,
                SubgroupSupport::Realized,
                second.clone(),
            ),
            Err(TargetProfileBuildError::DuplicateSubgroupRealization)
        );

        let descriptor = |source: TargetFactSource| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.subgroup-source.v1".to_owned()).unwrap(),
            );
            builder
                .declare_subgroup_realization(required, SubgroupSupport::Realized, source)
                .unwrap();
            builder.try_build().unwrap()
        };
        let left = descriptor(first);
        let right = descriptor(second);
        assert_ne!(
            left.canonical_descriptor(),
            right.canonical_descriptor(),
            "a source-revision change must move the complete declaration"
        );
        assert_eq!(
            left.checked().canonical_descriptor(),
            right.checked().canonical_descriptor(),
            "the checked descriptor encodes phase, authority, and validity, not the source identity"
        );
    }

    #[test]
    fn measured_subgroup_declaration_uses_the_measured_source_authority() {
        let required = subgroup_subject(32, ArithmeticType::F32);
        let source = compile_profile_measurement_source("1.0", "build-1");
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-measured.v1".to_owned()).unwrap(),
        );
        builder
            .declare_measured_subgroup_realization(required, SubgroupSupport::Realized, source)
            .unwrap();
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.subgroup_realization(required, AvailabilityPhase::CompileProfile),
            SubgroupRealizationResolution::Realized
        );
        let fact = &profile.checked().subgroup()[0];
        assert_eq!(fact.authority(), FactAuthority::MeasuredProfile);
        assert_eq!(fact.validity(), FactValidityScope::MeasuredEnvironment);
    }

    #[test]
    fn subgroup_subject_and_verdict_participate_in_identity_independently() {
        let source = public_external_source(1);
        let baseline = subgroup_subject(32, ArithmeticType::F32);
        let descriptor = |subject, support| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.subgroup-subject.v1".to_owned()).unwrap(),
            );
            builder
                .declare_subgroup_realization(subject, support, source.clone())
                .unwrap();
            builder.try_build().unwrap()
        };
        let realized = descriptor(baseline, SubgroupSupport::Realized);
        let refused = descriptor(baseline, SubgroupSupport::Unrealizable);
        assert_ne!(
            realized.canonical_descriptor(),
            refused.canonical_descriptor()
        );
        assert_ne!(
            realized.checked().canonical_descriptor(),
            refused.checked().canonical_descriptor()
        );
        for (dimension, neighbour) in [
            ("width", subgroup_subject(64, ArithmeticType::F32)),
            ("arithmetic", subgroup_subject(32, ArithmeticType::Bf16)),
        ] {
            let other = descriptor(neighbour, SubgroupSupport::Realized);
            assert_ne!(
                realized.canonical_descriptor(),
                other.canonical_descriptor(),
                "the {dimension} dimension does not reach the complete descriptor"
            );
            assert_ne!(
                realized.checked().canonical_descriptor(),
                other.checked().canonical_descriptor(),
                "the {dimension} dimension does not reach the checked descriptor"
            );
        }
    }

    #[test]
    fn subgroup_perturbations_quote_distinct_failures() {
        let required = subgroup_subject(32, ArithmeticType::F32);
        let source = public_external_source(1);
        let profile = |subject, support: SubgroupSupport| {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.subgroup-perturb.v1".to_owned()).unwrap(),
            );
            builder
                .declare_subgroup_realization(subject, support, source.clone())
                .unwrap();
            builder.try_build().unwrap()
        };
        let realized = profile(required, SubgroupSupport::Realized);
        assert_eq!(
            format!(
                "{:?}",
                realized.subgroup_realization(required, AvailabilityPhase::CompileProfile)
            ),
            "Realized"
        );
        assert_eq!(
            format!(
                "{:?}",
                realized.subgroup_realization(
                    subgroup_subject(64, ArithmeticType::F32),
                    AvailabilityPhase::CompileProfile,
                )
            ),
            "Unknown",
            "width perturbation must be Unknown"
        );
        assert_eq!(
            format!(
                "{:?}",
                realized.subgroup_realization(
                    subgroup_subject(32, ArithmeticType::Bf16),
                    AvailabilityPhase::CompileProfile,
                )
            ),
            "Unknown",
            "arithmetic perturbation must be Unknown"
        );
        assert_eq!(
            format!(
                "{:?}",
                profile(required, SubgroupSupport::Unrealizable)
                    .subgroup_realization(required, AvailabilityPhase::CompileProfile)
            ),
            "Unrealizable"
        );
        let later = {
            let mut builder = TargetProfileBuilder::new(
                TargetProfileKey::new("test.subgroup-perturb-phase.v1".to_owned()).unwrap(),
            );
            builder
                .declare_subgroup_realization(
                    required,
                    SubgroupSupport::Realized,
                    device_runtime_source(),
                )
                .unwrap();
            builder.try_build().unwrap()
        };
        assert_eq!(
            format!(
                "{:?}",
                later.subgroup_realization(required, AvailabilityPhase::CompileProfile)
            ),
            "Unknown",
            "compile-phase lookup of a later-phase row must be Unknown"
        );
        let silent = public_builder("test.subgroup-perturb-silence.v1")
            .try_build()
            .unwrap();
        assert_eq!(
            format!(
                "{:?}",
                silent.subgroup_realization(required, AvailabilityPhase::LaunchPreflight)
            ),
            "Unknown",
            "silence must be Unknown"
        );
        let mut colliding = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-perturb-source.v1".to_owned()).unwrap(),
        );
        colliding
            .declare_subgroup_realization(required, SubgroupSupport::Realized, source)
            .unwrap();
        assert_eq!(
            format!(
                "{:?}",
                colliding.declare_subgroup_realization(
                    required,
                    SubgroupSupport::Realized,
                    public_external_source(2),
                )
            ),
            "Err(DuplicateSubgroupRealization)",
            "a second source at the same subject and phase must refuse"
        );
    }

    #[test]
    fn independently_true_subgroup_neighbours_compose_into_no_permission() {
        let source = public_external_source(1);
        let required = subgroup_subject(32, ArithmeticType::F32);
        let mut builder = TargetProfileBuilder::new(
            TargetProfileKey::new("test.subgroup-compose.v1".to_owned()).unwrap(),
        );
        builder
            .declare_subgroup_realization(
                subgroup_subject(64, ArithmeticType::F32),
                SubgroupSupport::Realized,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_subgroup_realization(
                subgroup_subject(32, ArithmeticType::Bf16),
                SubgroupSupport::Realized,
                source,
            )
            .unwrap();
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.subgroup_realization(required, AvailabilityPhase::CompileProfile),
            SubgroupRealizationResolution::Unknown,
            "independently true neighbouring facts must not compose into a permission"
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
        // Two governed subjects rather than two hand-built nominal ones: the
        // relocated validator admits exactly one value identity per arithmetic
        // type, so a subject over an unregistered type cannot be constructed.
        let a = ScalarArithmetic::f32();
        let b = ScalarArithmetic::new(ArithmeticType::F16, governed_scalar("f16"))
            .expect("the catalog registers f16 over tiler::f16@1");
        let row = |subject, behaviour| ScalarHonourabilityDeclaration {
            subject,
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
                a.arithmetic(),
                a.resolved_type().clone(),
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

    fn bf16_subject() -> ScalarArithmetic {
        ScalarArithmetic::new(
            ArithmeticType::Bf16,
            registered_arithmetic_value_type(ArithmeticType::Bf16)
                .expect("the governed catalog registers bf16"),
        )
        .expect("the bf16 policy subject is validated")
    }

    fn device_runtime_source() -> TargetFactSource {
        let compiler = TargetCompilerBuild::new(
            TargetCompilerRole::RuntimeCompiler,
            "test-runtime-compiler".to_owned(),
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
        TargetFactSource::measured(
            TargetFactProducerIdentity::new("test.evaluation-order-observer.v1".to_owned(), 1)
                .unwrap(),
            MeasuredFactAuthority::DeviceRuntime,
            [TargetMeasurementContext::new([compiler], environment).unwrap()],
        )
        .unwrap()
    }

    /// The fail-closed half: silence is `Unknown` for every subject and licence,
    /// and it costs a profile that declares nothing not one descriptor byte.
    #[test]
    fn a_profile_declaring_no_evaluation_order_row_resolves_unknown() {
        for profile in [
            TargetProfile::governed(),
            public_builder("acme.silent.v1").try_build().unwrap(),
        ] {
            for subject in [ScalarArithmetic::f32(), bf16_subject()] {
                for licence in [
                    BackendArithmeticLicence::Withheld,
                    BackendArithmeticLicence::Granted,
                ] {
                    assert_eq!(
                        profile.evaluation_order_preservation(
                            &subject,
                            licence,
                            AvailabilityPhase::LaunchPreflight,
                        ),
                        EvaluationOrderResolution::Unknown,
                        "an undeclared {} row must not resolve",
                        licence.key()
                    );
                }
            }
            assert!(
                !profile
                    .canonical_descriptor()
                    .windows(EVALUATION_ORDER_DOMAIN.len())
                    .any(|window| window == EVALUATION_ORDER_DOMAIN),
                "an undeclaring profile writes none of the family's bytes, which is \
                 why the complete-declaration domain did not step"
            );
        }
    }

    /// The observing half: a declared row resolves per licence, and neither the
    /// other licence nor a neighbouring arithmetic type inherits it.
    #[test]
    fn declared_evaluation_order_rows_resolve_per_licence_and_are_not_inherited() {
        let source = public_external_source(1);
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_evaluation_order_preservation(
                ScalarArithmetic::f32(),
                BackendArithmeticLicence::Withheld,
                EvaluationOrderPreservation::Preserved,
                source.clone(),
            )
            .unwrap();
        builder
            .declare_evaluation_order_preservation(
                ScalarArithmetic::f32(),
                BackendArithmeticLicence::Granted,
                EvaluationOrderPreservation::NotPreserved,
                source,
            )
            .unwrap();
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.evaluation_order_preservation(
                &ScalarArithmetic::f32(),
                BackendArithmeticLicence::Withheld,
                AvailabilityPhase::CompileProfile,
            ),
            EvaluationOrderResolution::Preserved
        );
        assert_eq!(
            profile.evaluation_order_preservation(
                &ScalarArithmetic::f32(),
                BackendArithmeticLicence::Granted,
                AvailabilityPhase::CompileProfile,
            ),
            EvaluationOrderResolution::NotPreserved
        );
        for licence in [
            BackendArithmeticLicence::Withheld,
            BackendArithmeticLicence::Granted,
        ] {
            assert_eq!(
                profile.evaluation_order_preservation(
                    &bf16_subject(),
                    licence,
                    AvailabilityPhase::CompileProfile,
                ),
                EvaluationOrderResolution::Unknown,
                "an `f32` row is not evidence about `bf16`"
            );
        }
        assert!(
            profile
                .canonical_descriptor()
                .windows(EVALUATION_ORDER_DOMAIN.len())
                .any(|window| window == EVALUATION_ORDER_DOMAIN)
        );
        assert_ne!(
            profile.canonical_descriptor(),
            TargetProfile::governed().canonical_descriptor()
        );
    }

    #[test]
    fn a_later_phase_evaluation_order_row_defers_rather_than_resolving() {
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_evaluation_order_preservation(
                ScalarArithmetic::f32(),
                BackendArithmeticLicence::Granted,
                EvaluationOrderPreservation::NotPreserved,
                device_runtime_source(),
            )
            .unwrap();
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.evaluation_order_preservation(
                &ScalarArithmetic::f32(),
                BackendArithmeticLicence::Granted,
                AvailabilityPhase::CompileProfile,
            ),
            EvaluationOrderResolution::Deferred {
                available_at: AvailabilityPhase::LiveDevicePreflight,
            }
        );
        assert_eq!(
            profile.evaluation_order_preservation(
                &ScalarArithmetic::f32(),
                BackendArithmeticLicence::Granted,
                AvailabilityPhase::LiveDevicePreflight,
            ),
            EvaluationOrderResolution::NotPreserved
        );
    }

    #[test]
    fn a_second_evaluation_order_verdict_at_one_phase_is_refused_atomically() {
        let source = public_external_source(1);
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_evaluation_order_preservation(
                ScalarArithmetic::f32(),
                BackendArithmeticLicence::Withheld,
                EvaluationOrderPreservation::Preserved,
                source.clone(),
            )
            .unwrap();
        assert_eq!(
            builder.declare_evaluation_order_preservation(
                ScalarArithmetic::f32(),
                BackendArithmeticLicence::Withheld,
                EvaluationOrderPreservation::NotPreserved,
                source,
            ),
            Err(
                TargetProfileBuildError::DuplicateEvaluationOrderPreservation {
                    licence: BackendArithmeticLicence::Withheld.key(),
                    phase: AvailabilityPhase::CompileProfile,
                }
            )
        );
        // The refusal inserted nothing, so the first verdict still stands.
        assert_eq!(
            builder.try_build().unwrap().evaluation_order_preservation(
                &ScalarArithmetic::f32(),
                BackendArithmeticLicence::Withheld,
                AvailabilityPhase::CompileProfile,
            ),
            EvaluationOrderResolution::Preserved
        );
    }

    #[test]
    fn evaluation_order_subject_licence_and_verdict_participate_in_complete_identity() {
        let descriptor = |subject: ScalarArithmetic, licence, preservation| {
            let mut builder = TargetProfileBuilder::governed();
            builder
                .declare_evaluation_order_preservation(
                    subject,
                    licence,
                    preservation,
                    public_external_source(1),
                )
                .unwrap();
            builder.try_build().unwrap().canonical_descriptor().to_vec()
        };
        let baseline = descriptor(
            ScalarArithmetic::f32(),
            BackendArithmeticLicence::Withheld,
            EvaluationOrderPreservation::Preserved,
        );
        assert_ne!(
            baseline,
            descriptor(
                ScalarArithmetic::f32(),
                BackendArithmeticLicence::Withheld,
                EvaluationOrderPreservation::NotPreserved,
            )
        );
        assert_ne!(
            baseline,
            descriptor(
                ScalarArithmetic::f32(),
                BackendArithmeticLicence::Granted,
                EvaluationOrderPreservation::Preserved,
            )
        );
        assert_ne!(
            baseline,
            descriptor(
                bf16_subject(),
                BackendArithmeticLicence::Withheld,
                EvaluationOrderPreservation::Preserved,
            )
        );
    }

    /// Silence is `Unknown` and costs a profile that declares nothing not one
    /// descriptor byte.
    #[test]
    fn a_profile_declaring_no_tree_width_policy_resolves_unknown() {
        for profile in [
            TargetProfile::governed(),
            public_builder("acme.silent-tree-width.v1")
                .try_build()
                .unwrap(),
        ] {
            assert_eq!(
                profile.workgroup_tree_width_policy(AvailabilityPhase::LaunchPreflight),
                WorkgroupTreeWidthPolicyResolution::Unknown,
            );
            assert!(
                !profile
                    .canonical_descriptor()
                    .windows(WORKGROUP_TREE_WIDTH_POLICY_DOMAIN.len())
                    .any(|window| window == WORKGROUP_TREE_WIDTH_POLICY_DOMAIN),
                "an undeclaring profile writes none of the family's bytes, which is \
                 why the complete-declaration domain did not step"
            );
        }
    }

    #[test]
    fn a_declared_tree_width_policy_resolves_and_moves_identity() {
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_workgroup_tree_width_policy(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1,
                public_external_source(1),
            )
            .unwrap();
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.workgroup_tree_width_policy(AvailabilityPhase::CompileProfile),
            WorkgroupTreeWidthPolicyResolution::Declared(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1
            )
        );
        assert!(
            profile
                .canonical_descriptor()
                .windows(WORKGROUP_TREE_WIDTH_POLICY_DOMAIN.len())
                .any(|window| window == WORKGROUP_TREE_WIDTH_POLICY_DOMAIN)
        );
        assert_ne!(
            profile.canonical_descriptor(),
            TargetProfile::governed().canonical_descriptor()
        );
    }

    #[test]
    fn a_later_phase_tree_width_policy_defers_rather_than_resolving() {
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_workgroup_tree_width_policy(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1,
                device_runtime_source(),
            )
            .unwrap();
        let profile = builder.try_build().unwrap();
        assert_eq!(
            profile.workgroup_tree_width_policy(AvailabilityPhase::CompileProfile),
            WorkgroupTreeWidthPolicyResolution::Deferred {
                available_at: AvailabilityPhase::LiveDevicePreflight,
            }
        );
        assert_eq!(
            profile.workgroup_tree_width_policy(AvailabilityPhase::LiveDevicePreflight),
            WorkgroupTreeWidthPolicyResolution::Declared(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1
            )
        );
    }

    #[test]
    fn a_second_tree_width_policy_at_one_phase_is_refused_atomically() {
        let source = public_external_source(1);
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_workgroup_tree_width_policy(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1,
                source.clone(),
            )
            .unwrap();
        assert_eq!(
            builder.declare_workgroup_tree_width_policy(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1,
                source,
            ),
            Err(TargetProfileBuildError::DuplicateWorkgroupTreeWidthPolicy {
                phase: AvailabilityPhase::CompileProfile,
            })
        );
        assert_eq!(
            builder
                .try_build()
                .unwrap()
                .workgroup_tree_width_policy(AvailabilityPhase::CompileProfile),
            WorkgroupTreeWidthPolicyResolution::Declared(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1
            )
        );
    }

    #[test]
    fn changing_the_tree_width_policy_tag_moves_canonical_identity() {
        let mut builder = TargetProfileBuilder::governed();
        builder
            .declare_workgroup_tree_width_policy(
                WorkgroupTreeWidthPolicy::MeasuredNearestCap256V1,
                public_external_source(1),
            )
            .unwrap();
        let declared = builder.try_build().unwrap().canonical_descriptor().to_vec();
        assert!(
            declared
                .windows(WORKGROUP_TREE_WIDTH_POLICY_DOMAIN.len())
                .any(|window| window == WORKGROUP_TREE_WIDTH_POLICY_DOMAIN)
        );
        assert_ne!(declared, TargetProfile::governed().canonical_descriptor());
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

    /// Candidate shapes where all three reduction strategies are expressible, for
    /// one grid-axis bound.
    ///
    /// This helper deliberately reads only the algebraic and launch conditions it
    /// models: `governed_partition` withholds the split,
    /// `capped_tree_partition` withholds the tree — the two choose different
    /// participant counts and are asked separately rather than one standing in
    /// for the other — and the grid-axis bound assesses the prologue's
    /// one-invocation-per-element launch. It does not assess a plan's target
    /// feasibility: local memory and synchronization realization are read later
    /// by the physical feasibility path, which can withhold the tree for every
    /// shape on a profile such as the governed baseline.
    fn three_strategy_domain(grid_axis_bound: u64) -> Vec<(u64, u64)> {
        let mut domain = Vec::new();
        for rows in 1..=grid_axis_bound {
            for contributors in 1..=grid_axis_bound {
                if crate::physical::governed_partition(contributors).is_some()
                    && crate::physical::capped_tree_partition(contributors).is_some()
                    && rows * contributors <= grid_axis_bound
                {
                    domain.push((rows, contributors));
                }
            }
        }
        domain
    }

    /// **The prototype baseline has one three-strategy candidate shape, and it
    /// is not the profile calibration measures against.**
    ///
    /// This test was written as the measured-calibration trigger for
    /// [`calibrate-and-activate-parallel-reduction-selection`], and it could not
    /// have fired. It reads the bound from [`TargetProfileBuilder::governed`] —
    /// the *target-neutral prototype baseline*, keyed
    /// `tiler.prototype-target-neutral-baseline.v1` — while calibration measures
    /// against `tiler_build::BoundMetalCompileDeclaration::first_macos_apple9`.
    /// Both declared four, so the difference was invisible until one of them
    /// moved.
    ///
    /// **On 2026-08-04 the Metal row moved to a measured 268,435,456 and this
    /// one deliberately did not.** A macOS Apple9 device measurement is evidence
    /// about one target; a baseline standing in for every target cannot be
    /// widened by it, and widening it on the compiler's own say-so would be a
    /// number chosen rather than sourced. So the prototype row keeps its
    /// conservative four, and the real trigger lives in the crate that can see
    /// the profile it is about:
    /// `tiler_build::metal_plan::tests::the_measured_grid_axis_admits_more_than_one_three_strategy_shape`.
    ///
    /// What this test still checks is worth keeping and is what its name now
    /// says: for *this helper* the derivation `4 <= contributors <=
    /// rows * contributors <= bound` closes on `(1, 4)`, because both partition
    /// rules — `governed_partition` for the split and `capped_tree_partition`
    /// for the tree — withhold their strategy below four contributors. The two
    /// disagree about *which* participant count to take, never about which
    /// extents admit one, so the floor in that derivation is one number and not
    /// a coincidence. If the prototype baseline is ever widened — which is a
    /// product question about what a target-neutral guarantee should offer, not
    /// an authority question this ticket could answer — this fires.
    ///
    /// The raised-bound case below is not decoration: without it a domain
    /// computation that returned a one-element vector unconditionally would pass
    /// the real assertion, and the check would be indistinguishable from one that
    /// never ran.
    ///
    /// [`calibrate-and-activate-parallel-reduction-selection`]:
    ///     ../../../tickets/calibrate-and-activate-parallel-reduction-selection.md
    #[test]
    fn the_prototype_baseline_has_one_three_strategy_candidate_shape() {
        let bound = TargetProfileBuilder::governed()
            .quantitative
            .iter()
            .find(|declaration| declaration.axis == CapabilityAxis::GridAxisThreads)
            .expect("the governed profile declares the grid-axis limit")
            .bound;

        let domain = three_strategy_domain(bound);
        assert_eq!(
            domain,
            vec![(1, 4)],
            "the prototype baseline's three-strategy domain moved at grid-axis bound {bound}. \
             This is the helper's algebraic-and-launch domain, not a feasibility result. \
             The target-neutral baseline is not the profile calibration measures against: \
             widening it needs an authority covering every target, which no device measurement \
             can supply. The Metal profile's domain is reported by tiler-build's \
             the_measured_grid_axis_admits_more_than_one_three_strategy_shape"
        );

        // The same derivation at a wider bound, so the single point above is a
        // property of this profile rather than of the computation.
        let widened = three_strategy_domain(8);
        assert!(
            widened.len() > 1,
            "raising the grid-axis bound must admit more shapes, or this check cannot \
             distinguish a narrow profile from a broken domain computation: {widened:?}"
        );
        assert!(
            widened.contains(&(1, 4)) && widened.contains(&(2, 4)),
            "the widened domain must extend the narrow one rather than replace it: {widened:?}"
        );
    }

    fn verified_silu_contract() -> tiler_ir::semantic::accuracy::VerifiedAccuracyContract {
        let contract = tiler_ir::semantic::silu_f32_exponential_accuracy_contract();
        let facts = builtin_scalar_value_type_facts(contract.result_type())
            .expect("F32 carries builtin value-type facts");
        contract
            .verify(&facts)
            .expect("the registered SiLU contract verifies")
    }

    fn discharging_evidence(
        scope: &str,
        digest: &[u8],
    ) -> tiler_ir::semantic::accuracy::ConformanceEvidence {
        let reference = |text: &str| {
            tiler_ir::semantic::NormativeDefinitionRef::new(text)
                .expect("a fixture evidence field is canonical")
        };
        tiler_ir::semantic::accuracy::ConformanceEvidence::new(
            tiler_ir::semantic::accuracy::ConformanceEvidenceClass::NormativeGuarantee,
            reference(scope),
            reference("synthetic both-halves fixture, not a Metal specification claim"),
            reference("fixture.elementary.declaration"),
            reference("tiler test fixture, not a toolchain row"),
            None,
            None,
            None,
            digest,
        )
        .expect("the discharging fixture is well formed")
    }

    fn silu_realization(source: &TargetFactSource) -> ElementaryRealization {
        ElementaryRealization::new(
            &verified_silu_contract(),
            discharging_evidence(
                "fixture bound half for tiler::silu-f32@1",
                b"fixture:silu-bound-v1",
            ),
            discharging_evidence(
                "fixture exceptional half for tiler::silu-f32@1",
                b"fixture:silu-exceptional-v1",
            ),
            source,
        )
        .expect("a compile-profile source is accepted")
    }

    #[test]
    fn later_phase_source_is_refused_at_subject_construction() {
        let later = deferred_measurement_source();
        let error = ElementaryRealization::new(
            &verified_silu_contract(),
            discharging_evidence("later-phase bound", b"fixture:later-bound-v1"),
            discharging_evidence("later-phase exceptional", b"fixture:later-exceptional-v1"),
            &later,
        )
        .expect_err("a live-device source cannot speak at compile profile");
        assert_eq!(
            error,
            ElementaryRealizationError::LaterPhaseSource {
                phase: AvailabilityPhase::LiveDevicePreflight,
            }
        );
    }

    fn deferred_measurement_source() -> TargetFactSource {
        let compiler = TargetCompilerBuild::new(
            TargetCompilerRole::RuntimeCompiler,
            "test-runtime-compiler".to_owned(),
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
        TargetFactSource::measured(
            TargetFactProducerIdentity::new("test.runtime-probe.v1".to_owned(), 1).unwrap(),
            MeasuredFactAuthority::DeviceRuntime,
            [context],
        )
        .unwrap()
    }

    #[test]
    fn exact_duplicate_elementary_realization_is_refused() {
        let source = public_external_source(1);
        let realization = silu_realization(&source);
        let mut builder = public_builder("test.elementary-duplicate.v1");
        builder
            .declare_elementary_realization(realization.clone())
            .unwrap();
        assert_eq!(
            builder.declare_elementary_realization(realization),
            Err(TargetProfileBuildError::DuplicateElementaryRealization)
        );
    }

    #[test]
    fn distinct_same_operation_contracts_remain_separate_candidates() {
        let source = public_external_source(1);
        let first = silu_realization(&source);
        let second = ElementaryRealization::new(
            &verified_silu_contract(),
            discharging_evidence("second bound half", b"fixture:silu-bound-v2"),
            discharging_evidence("second exceptional half", b"fixture:silu-exceptional-v2"),
            &source,
        )
        .unwrap();
        let mut builder = public_builder("test.elementary-distinct.v1");
        builder
            .declare_elementary_realization(first.clone())
            .unwrap();
        builder
            .declare_elementary_realization(second.clone())
            .unwrap();
        let profile = builder.build().unwrap();
        let declared = profile.declared_elementary_realizations();
        assert_eq!(declared.len(), 2);
        assert_eq!(declared[0].operation(), first.operation());
        assert_eq!(declared[1].operation(), second.operation());
        assert_ne!(declared[0], declared[1]);
    }

    #[test]
    fn a_profile_declaring_no_elementary_row_encodes_like_a_build_without_the_family() {
        let silent = public_builder("test.elementary-silent.v1").build().unwrap();
        let governed = TargetProfile::governed();
        assert!(silent.declared_elementary_realizations().is_empty());
        assert!(governed.declared_elementary_realizations().is_empty());
        assert!(
            !silent
                .canonical_descriptor()
                .windows(ELEMENTARY_REALIZATION_DOMAIN.len())
                .any(|window| window == ELEMENTARY_REALIZATION_DOMAIN)
        );
        assert!(
            !governed
                .canonical_descriptor()
                .windows(ELEMENTARY_REALIZATION_DOMAIN.len())
                .any(|window| window == ELEMENTARY_REALIZATION_DOMAIN)
        );
    }

    #[test]
    fn declaring_an_elementary_row_appends_the_terminal_family_without_stepping_the_domain() {
        let source = public_external_source(1);
        let mut builder = public_builder("test.elementary-encoded.v1");
        let silent = builder.clone().build().unwrap();
        builder
            .declare_elementary_realization(silu_realization(&source))
            .unwrap();
        let declared = builder.build().unwrap();
        assert_ne!(
            silent.canonical_descriptor(),
            declared.canonical_descriptor()
        );
        assert!(
            declared
                .canonical_descriptor()
                .windows(COMPLETE_PROFILE_DESCRIPTOR_DOMAIN.len())
                .any(|window| window == COMPLETE_PROFILE_DESCRIPTOR_DOMAIN)
        );
        assert!(
            declared
                .canonical_descriptor()
                .windows(ELEMENTARY_REALIZATION_DOMAIN.len())
                .any(|window| window == ELEMENTARY_REALIZATION_DOMAIN)
        );
        assert_eq!(declared.declared_elementary_realizations().len(), 1);
        assert_eq!(
            declared.declared_elementary_realizations()[0].source_producer_key(),
            "test.external-profile-producer.v1"
        );
    }

    #[test]
    fn elementary_declaration_order_is_not_identity() {
        let source = public_external_source(1);
        let first = silu_realization(&source);
        let second = ElementaryRealization::new(
            &verified_silu_contract(),
            discharging_evidence("order bound half", b"fixture:silu-bound-order-v2"),
            discharging_evidence(
                "order exceptional half",
                b"fixture:silu-exceptional-order-v2",
            ),
            &source,
        )
        .unwrap();
        let mut left = public_builder("test.elementary-order.v1");
        left.declare_elementary_realization(first.clone()).unwrap();
        left.declare_elementary_realization(second.clone()).unwrap();
        let mut right = public_builder("test.elementary-order.v1");
        right.declare_elementary_realization(second).unwrap();
        right.declare_elementary_realization(first).unwrap();
        assert_eq!(
            left.build().unwrap().canonical_descriptor(),
            right.build().unwrap().canonical_descriptor()
        );
    }
}

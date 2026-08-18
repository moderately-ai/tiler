//! Provider-versioned canonical target-environment declarations, receipts, and
//! the plan-determinism scope vocabulary.
//!
//! This module is the artifact half of the accepted ADR 0013 stability-subject
//! carrier (`decide-the-adr-0013-plan-determinism-stability-subject`, accepted
//! 2026-08-18). ADR 0013's plan-deterministic guarantee holds four premises
//! fixed, and the one this module owns is the **declared target-environment
//! compatibility identity**: the class of execution environments within which a
//! plan-deterministic route promises identical output bits.
//!
//! # Provider-versioned, and why no neutral field list exists
//!
//! No accepted evidence makes one finite backend-neutral field list complete
//! across native objects, runtime translation, interpreters, CPUs, and future
//! backends, so the *inner* descriptor vocabulary belongs to each backend
//! provider under its own governed identity and exact schema version. The
//! generic layer here decides every outer field, bound, canonical encoding,
//! owner, comparison, and refusal. A provider must put every output-affecting
//! runtime, compiler, device, and process arithmetic condition into its
//! descriptor schema; if it cannot prove that equality of those bytes is
//! sufficient, it cannot register positive support.
//!
//! # Raw bytes never self-certify
//!
//! A [`TargetEnvironmentDeclaration`] is explicitly a *declaration*, never an
//! attestation: anyone may state one, and stating one proves nothing. Positive
//! support requires an independently selected authority exposing the exact
//! [`TargetEnvironmentDescriptorSchema`] — at build time the backend's
//! installed payload verifier, at run time the consumer-selected adapter — to
//! validate the declaration and, at run time, to produce a live observation
//! after binding a context. Every conversion toward a positive claim goes
//! through a type this module mints privately:
//! [`ValidatedTargetEnvironmentDeclaration`] only through
//! [`TargetEnvironmentDeclaration::validate`], and
//! [`PayloadPlanDeterminismReceipt`] only through
//! [`PayloadPlanDeterminismVerifier::verify`].

use std::error::Error;
use std::fmt;

use tiler_digest::Digest;
use tiler_ir::identity::push_slice;
use tiler_ir::kernel::PlanDeterminismWitness;
use tiler_ir::semantic::ProviderIdentity;

use super::keys::{
    BackendKey, MAX_TARGET_PROFILE_DESCRIPTOR_BYTES, PayloadDigest, RepresentationKey,
    TargetProfileRef,
};
use super::model::{BackendPayloadDescriptor, SchemaVersion};

/// Versioned domain separator of one canonical target-environment identity.
///
/// Enumerated by `crate::domains` so the crate-wide no-prefix obligation covers
/// it. Spelled outside the `tiler.artifact` prefix deliberately: the identity
/// it opens is a runtime-compatibility class shared by the artifact and runtime
/// layers, not an artifact-program subject, and the accepted packet fixes this
/// exact spelling.
pub(crate) const TARGET_ENVIRONMENT_COMPATIBILITY_DOMAIN: &[u8] =
    b"tiler.target-environment-compatibility.v1\0";

/// Maximum byte length of one canonical target-environment descriptor.
///
/// The same ceiling as a target-profile descriptor, per the accepted packet:
/// both are canonical descriptors a comparison is linear in, and a larger
/// environment class than a compile profile has no warrant.
pub const MAX_TARGET_ENVIRONMENT_DESCRIPTOR_BYTES: usize = MAX_TARGET_PROFILE_DESCRIPTOR_BYTES;

/// Maximum byte length of one governed schema-violation reason code.
pub const MAX_TARGET_ENVIRONMENT_REASON_BYTES: usize = 256;

/// Why a target-environment declaration was refused.
///
/// The exact refusal vocabulary the accepted packet names for the declaration
/// grammar and its schema validation. Provider-identity grammar errors are not
/// duplicated here: [`ProviderIdentity`]'s own constructor retains its
/// empty/length/alphabet/nonzero-revision errors.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetEnvironmentDeclarationError {
    /// The descriptor schema declares major version zero, which is reserved as
    /// unversioned and can never name an exact class.
    ZeroSchemaMajor,
    /// The descriptor exceeds the governed byte ceiling.
    DescriptorTooLong {
        /// Attempted byte length.
        bytes: usize,
        /// Maximum admitted byte length.
        limit: usize,
    },
    /// The declaration names a provider the validating schema does not.
    ProviderMismatch {
        /// Provider the declaration names.
        declared: Box<ProviderIdentity>,
        /// Provider the schema is registered under.
        registered: Box<ProviderIdentity>,
    },
    /// The declaration names a schema version the validating schema does not.
    SchemaMismatch {
        /// Schema version the declaration names.
        declared: SchemaVersion,
        /// Schema version the registered schema exposes.
        registered: SchemaVersion,
    },
    /// The descriptor is not the schema's exactly-one canonical byte spelling.
    NoncanonicalDescriptor {
        /// Provider whose schema refused the descriptor.
        provider: Box<ProviderIdentity>,
        /// Exact schema version that refused it.
        schema: SchemaVersion,
        /// The schema's bounded governed reason code.
        reason: TargetEnvironmentReasonCode,
    },
}

impl fmt::Display for TargetEnvironmentDeclarationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroSchemaMajor => formatter.write_str(
                "target-environment.zero-schema-major: a descriptor schema must declare a \
                 nonzero major version",
            ),
            Self::DescriptorTooLong { bytes, limit } => write!(
                formatter,
                "target-environment.descriptor-too-long: {bytes} byte(s) exceed the \
                 {limit}-byte descriptor ceiling",
            ),
            Self::ProviderMismatch {
                declared,
                registered,
            } => write!(
                formatter,
                "target-environment.provider-mismatch: the declaration names \
                 {}::{}@{} and the registered schema is {}::{}@{}",
                declared.namespace(),
                declared.name(),
                declared.revision(),
                registered.namespace(),
                registered.name(),
                registered.revision(),
            ),
            Self::SchemaMismatch {
                declared,
                registered,
            } => write!(
                formatter,
                "target-environment.schema-mismatch: the declaration names {}.{} and the \
                 registered schema exposes {}.{}",
                declared.major(),
                declared.minor(),
                registered.major(),
                registered.minor(),
            ),
            Self::NoncanonicalDescriptor {
                provider,
                schema,
                reason,
            } => write!(
                formatter,
                "target-environment.noncanonical-descriptor: {}::{}@{} schema {}.{} refused \
                 the descriptor: {}",
                provider.namespace(),
                provider.name(),
                provider.revision(),
                schema.major(),
                schema.minor(),
                reason.as_str(),
            ),
        }
    }
}

impl Error for TargetEnvironmentDeclarationError {}

/// A bounded governed reason code a descriptor schema refuses with.
///
/// The same alphabet every governed key uses — ASCII lowercase, digits, `.`,
/// `-`, `_` — so a refusal is copyable from an explanation back into a search.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TargetEnvironmentReasonCode(String);

impl TargetEnvironmentReasonCode {
    /// Creates a validated governed reason code.
    ///
    /// # Errors
    ///
    /// Returns [`TargetEnvironmentReasonCodeError`] for an empty code, a code
    /// beyond [`MAX_TARGET_ENVIRONMENT_REASON_BYTES`], or a byte outside the
    /// governed-key alphabet.
    pub fn new(value: impl AsRef<str>) -> Result<Self, TargetEnvironmentReasonCodeError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(TargetEnvironmentReasonCodeError::Empty);
        }
        if value.len() > MAX_TARGET_ENVIRONMENT_REASON_BYTES {
            return Err(TargetEnvironmentReasonCodeError::TooLong { bytes: value.len() });
        }
        if let Some((index, byte)) = value
            .bytes()
            .enumerate()
            .find(|(_, byte)| !super::keys::admits(*byte))
        {
            return Err(TargetEnvironmentReasonCodeError::NoncanonicalByte { index, value: byte });
        }
        Ok(Self(value.to_owned()))
    }

    /// Returns the exact reason-code text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Why a schema-violation reason code was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetEnvironmentReasonCodeError {
    /// The reason code was empty.
    Empty,
    /// The reason code exceeded [`MAX_TARGET_ENVIRONMENT_REASON_BYTES`].
    TooLong {
        /// Attempted byte length.
        bytes: usize,
    },
    /// The reason code carried a byte outside the governed-key alphabet.
    NoncanonicalByte {
        /// Zero-based byte offset of the refused byte.
        index: usize,
        /// Refused byte value.
        value: u8,
    },
}

impl fmt::Display for TargetEnvironmentReasonCodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl Error for TargetEnvironmentReasonCodeError {}

/// The canonical descriptor bytes of one declared target environment.
///
/// Private bounded bytes: opaque to the neutral layer, canonical under exactly
/// one provider schema. The neutral layer bounds and compares them and never
/// interprets them.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TargetEnvironmentDescriptor(Box<[u8]>);

impl TargetEnvironmentDescriptor {
    /// Wraps canonical descriptor bytes a provider schema governs.
    ///
    /// # Errors
    ///
    /// Returns [`TargetEnvironmentDeclarationError::DescriptorTooLong`] beyond
    /// [`MAX_TARGET_ENVIRONMENT_DESCRIPTOR_BYTES`]. Emptiness is admitted: a
    /// provider schema whose class is carried entirely by its exact provider
    /// revision and schema version legitimately canonicalizes to zero bytes,
    /// and whether that is sufficient is the provider's own registration
    /// obligation.
    pub fn new(value: impl AsRef<[u8]>) -> Result<Self, TargetEnvironmentDeclarationError> {
        let value = value.as_ref();
        if value.len() > MAX_TARGET_ENVIRONMENT_DESCRIPTOR_BYTES {
            return Err(TargetEnvironmentDeclarationError::DescriptorTooLong {
                bytes: value.len(),
                limit: MAX_TARGET_ENVIRONMENT_DESCRIPTOR_BYTES,
            });
        }
        Ok(Self(value.into()))
    }

    /// Returns the exact canonical descriptor bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// One provider-versioned declared target environment.
///
/// **A declaration, never an attestation.** Anyone may state one — a producer
/// beside its payload, a decoder from bytes — and stating one certifies
/// nothing. It becomes usable toward a positive plan-determinism claim only
/// through [`Self::validate`] against an independently selected schema.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TargetEnvironmentDeclaration {
    provider: ProviderIdentity,
    descriptor_schema: SchemaVersion,
    descriptor: TargetEnvironmentDescriptor,
}

impl TargetEnvironmentDeclaration {
    /// States one raw declaration.
    ///
    /// # Errors
    ///
    /// Returns [`TargetEnvironmentDeclarationError::ZeroSchemaMajor`] for a
    /// schema whose major version is zero. The provider's own grammar was
    /// already validated by [`ProviderIdentity`]'s constructor.
    pub fn new(
        provider: ProviderIdentity,
        descriptor_schema: SchemaVersion,
        descriptor: TargetEnvironmentDescriptor,
    ) -> Result<Self, TargetEnvironmentDeclarationError> {
        if descriptor_schema.major() == 0 {
            return Err(TargetEnvironmentDeclarationError::ZeroSchemaMajor);
        }
        Ok(Self {
            provider,
            descriptor_schema,
            descriptor,
        })
    }

    /// Returns the provider identity that owns the descriptor schema.
    #[must_use]
    pub const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the exact declared descriptor schema version.
    #[must_use]
    pub const fn descriptor_schema(&self) -> SchemaVersion {
        self.descriptor_schema
    }

    /// Returns the declared canonical descriptor.
    #[must_use]
    pub const fn descriptor(&self) -> &TargetEnvironmentDescriptor {
        &self.descriptor
    }

    /// Validates this declaration against one registered provider schema.
    ///
    /// Exact provider identity and revision, exact schema major and minor, and
    /// the schema's exactly-one canonical byte spelling; no widening is
    /// implicit anywhere.
    ///
    /// # Errors
    ///
    /// Returns [`TargetEnvironmentDeclarationError::ProviderMismatch`],
    /// [`TargetEnvironmentDeclarationError::SchemaMismatch`], or
    /// [`TargetEnvironmentDeclarationError::NoncanonicalDescriptor`].
    pub fn validate(
        &self,
        schema: &dyn TargetEnvironmentDescriptorSchema,
    ) -> Result<ValidatedTargetEnvironmentDeclaration, TargetEnvironmentDeclarationError> {
        if self.provider != *schema.provider() {
            return Err(TargetEnvironmentDeclarationError::ProviderMismatch {
                declared: Box::new(self.provider.clone()),
                registered: Box::new(schema.provider().clone()),
            });
        }
        if self.descriptor_schema != schema.schema_version() {
            return Err(TargetEnvironmentDeclarationError::SchemaMismatch {
                declared: self.descriptor_schema,
                registered: schema.schema_version(),
            });
        }
        if let Err(reason) = schema.validate_canonical_descriptor(self.descriptor.as_bytes()) {
            return Err(TargetEnvironmentDeclarationError::NoncanonicalDescriptor {
                provider: Box::new(self.provider.clone()),
                schema: self.descriptor_schema,
                reason,
            });
        }
        Ok(ValidatedTargetEnvironmentDeclaration {
            declaration: self.clone(),
        })
    }
}

/// One provider's registered target-environment descriptor contract.
///
/// The provider-specific half of the accepted subject: it names its owning
/// provider and exact schema version, and validates that a descriptor is the
/// schema's exactly-one canonical byte spelling. Registering one is a claim of
/// authority — the provider asserts that equality of validated descriptor
/// bytes, under this exact provider revision and schema version, is sufficient
/// for the arithmetic-identical execution class ADR 0013 requires. A provider
/// that cannot prove that must not register positive support.
pub trait TargetEnvironmentDescriptorSchema {
    /// Returns the provider identity, including its exact nonzero revision.
    fn provider(&self) -> &ProviderIdentity;

    /// Returns the exact schema version this contract validates.
    fn schema_version(&self) -> SchemaVersion;

    /// Validates that `descriptor` is this schema's one canonical spelling.
    ///
    /// # Errors
    ///
    /// Returns the schema's bounded governed reason code for any other byte
    /// spelling, including a well-formed non-canonical one.
    fn validate_canonical_descriptor(
        &self,
        descriptor: &[u8],
    ) -> Result<(), TargetEnvironmentReasonCode>;
}

/// A declaration one registered provider schema validated exactly.
///
/// Opaque, and minted only by [`TargetEnvironmentDeclaration::validate`]. It is
/// still not an attestation — the schema validated a *spelling*, not a live
/// context — but it is the only value from which a
/// [`CanonicalTargetEnvironmentCompatibilityIdentity`] can be derived, so an
/// unvalidated declaration can never reach an identity comparison.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedTargetEnvironmentDeclaration {
    declaration: TargetEnvironmentDeclaration,
}

impl ValidatedTargetEnvironmentDeclaration {
    /// Returns the exact declaration the schema validated.
    #[must_use]
    pub const fn declaration(&self) -> &TargetEnvironmentDeclaration {
        &self.declaration
    }

    /// Derives the complete canonical target-environment compatibility identity.
    ///
    /// Only reachable from a validated declaration plus the exact compile
    /// profile, backend family, and executable representation the payload
    /// declares — the accepted subject's six components, with no hidden host
    /// choice and no default.
    #[must_use]
    pub fn compatibility_identity(
        &self,
        target_profile: &TargetProfileRef,
        backend: &BackendKey,
        representation: &RepresentationKey,
    ) -> CanonicalTargetEnvironmentCompatibilityIdentity {
        CanonicalTargetEnvironmentCompatibilityIdentity::derive(
            target_profile,
            backend,
            representation,
            &self.declaration,
        )
    }
}

/// The complete canonical target-environment compatibility identity.
///
/// The declared execution-environment class of ADR 0013's plan-deterministic
/// guarantee: compile profile (governed key plus exact descriptor identity),
/// backend family, executable representation, provider identity with its
/// nonzero revision, exact descriptor schema version, and the canonical
/// descriptor bytes. Equality is byte equality of the canonical encoding.
///
/// It deliberately excludes dtype-dispatch statements, delivery position,
/// live-device handles or serial numbers, queue and context identity,
/// capacities, timing, cost, and input bindings: dtype is checked independently
/// as route eligibility, delivery is the selected route coordinate, device and
/// context objects are neither portable nor compatibility classes, and
/// capacities are feasibility.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalTargetEnvironmentCompatibilityIdentity {
    target_profile: TargetProfileRef,
    backend: BackendKey,
    representation: RepresentationKey,
    provider: ProviderIdentity,
    descriptor_schema: SchemaVersion,
    descriptor: TargetEnvironmentDescriptor,
    bytes: Vec<u8>,
}

impl CanonicalTargetEnvironmentCompatibilityIdentity {
    fn derive(
        target_profile: &TargetProfileRef,
        backend: &BackendKey,
        representation: &RepresentationKey,
        declaration: &TargetEnvironmentDeclaration,
    ) -> Self {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(TARGET_ENVIRONMENT_COMPATIBILITY_DOMAIN);
        push_slice(&mut bytes, target_profile.key.as_str().as_bytes());
        push_slice(&mut bytes, target_profile.descriptor.as_bytes());
        push_slice(&mut bytes, backend.as_str().as_bytes());
        push_slice(&mut bytes, representation.as_str().as_bytes());
        push_slice(&mut bytes, declaration.provider.namespace().as_bytes());
        push_slice(&mut bytes, declaration.provider.name().as_bytes());
        bytes.extend_from_slice(&declaration.provider.revision().to_be_bytes());
        bytes.extend_from_slice(&declaration.descriptor_schema.major().to_be_bytes());
        bytes.extend_from_slice(&declaration.descriptor_schema.minor().to_be_bytes());
        push_slice(&mut bytes, declaration.descriptor.as_bytes());
        Self {
            target_profile: target_profile.clone(),
            backend: backend.clone(),
            representation: representation.clone(),
            provider: declaration.provider.clone(),
            descriptor_schema: declaration.descriptor_schema,
            descriptor: declaration.descriptor.clone(),
            bytes,
        }
    }

    /// Returns the canonical identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the compile profile half of the identity.
    #[must_use]
    pub const fn target_profile(&self) -> &TargetProfileRef {
        &self.target_profile
    }

    /// Returns the governed backend family key.
    #[must_use]
    pub const fn backend(&self) -> &BackendKey {
        &self.backend
    }

    /// Returns the governed executable-representation key.
    #[must_use]
    pub const fn representation(&self) -> &RepresentationKey {
        &self.representation
    }

    /// Returns the provider identity, including its exact revision.
    #[must_use]
    pub const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the exact descriptor schema version.
    #[must_use]
    pub const fn descriptor_schema(&self) -> SchemaVersion {
        self.descriptor_schema
    }

    /// Returns the canonical descriptor bytes.
    #[must_use]
    pub const fn descriptor(&self) -> &TargetEnvironmentDescriptor {
        &self.descriptor
    }
}

/// The plan-determinism scope one variant claims at one delivery position.
///
/// Exhaustive, deliberately not `#[non_exhaustive]` (ADR 0074 convention 5b):
/// a scope is an instruction to a router, and a reader that gained a wildcard
/// arm would silently route a newly governed scope class as one of these two.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PlanDeterminismScope {
    /// No determinism claim: the cell stays routable with no stability subject.
    Unclaimed,
    /// The ADR 0013 plan-deterministic claim, proof-bound at build.
    Plan,
}

impl PlanDeterminismScope {
    pub(super) const fn tag(self) -> u8 {
        match self {
            Self::Unclaimed => 0x01,
            Self::Plan => 0x02,
        }
    }

    /// Resolves a governed wire tag, or `None` for an unrecognized scope.
    pub(super) const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::Unclaimed),
            0x02 => Some(Self::Plan),
            _ => None,
        }
    }
}

/// Why a backend's payload verifier refused a plan-determinism receipt.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a later backend judgment
/// lands additively.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PayloadPlanDeterminismRefusal {
    /// The backend's translation of these bytes makes, or may make, a
    /// run-dependent choice the declared environment does not fix.
    RunDependentTranslation,
    /// The witness proves a different kernel program than the payload realizes.
    KernelProgramMismatch,
    /// The payload's compilation subject is not the one the verifier compiled.
    PayloadSubjectMismatch,
    /// The emitted object bytes are not the ones the verifier emitted.
    ObjectDigestMismatch,
    /// The declared target environment is not the one the verifier can bind
    /// its translation evidence to.
    TargetEnvironmentMismatch,
}

impl fmt::Display for PayloadPlanDeterminismRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rule = match self {
            Self::RunDependentTranslation => "payload-plan-determinism.run-dependent-translation",
            Self::KernelProgramMismatch => "payload-plan-determinism.kernel-program-mismatch",
            Self::PayloadSubjectMismatch => "payload-plan-determinism.payload-subject-mismatch",
            Self::ObjectDigestMismatch => "payload-plan-determinism.object-digest-mismatch",
            Self::TargetEnvironmentMismatch => {
                "payload-plan-determinism.target-environment-mismatch"
            }
        };
        formatter.write_str(rule)
    }
}

impl Error for PayloadPlanDeterminismRefusal {}

/// A backend's proof that one emitted payload preserves plan determinism.
///
/// Privately minted by [`PayloadPlanDeterminismVerifier::verify`] from the
/// exact verified kernel-program identity, the payload's compilation subject,
/// the emitted object's section digest, and the validated declared target
/// environment. There is no public constructor and no accessor is settable, so
/// low-level artifact construction cannot pass a bool or a raw declaration as
/// proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayloadPlanDeterminismReceipt {
    kernel_program: Vec<u8>,
    payload_subject: PayloadDigest,
    object_section_digest: Digest,
    environment: CanonicalTargetEnvironmentCompatibilityIdentity,
}

impl PayloadPlanDeterminismReceipt {
    /// Returns the canonical kernel-program identity bytes the receipt binds.
    #[must_use]
    pub fn kernel_program_identity(&self) -> &[u8] {
        &self.kernel_program
    }

    /// Returns the payload compilation subject the receipt binds.
    #[must_use]
    pub const fn payload_subject(&self) -> &PayloadDigest {
        &self.payload_subject
    }

    /// Returns the emitted object's governed section digest.
    #[must_use]
    pub const fn object_section_digest(&self) -> &Digest {
        &self.object_section_digest
    }

    /// Returns the complete declared target-environment identity.
    #[must_use]
    pub const fn environment(&self) -> &CanonicalTargetEnvironmentCompatibilityIdentity {
        &self.environment
    }
}

/// A backend's installed payload plan-determinism verifier.
///
/// The backend-owned judgment the accepted carrier requires before any payload
/// can support a `Plan` cell: only the backend whose representation the object
/// is can say whether its translation to an executable state is free of
/// run-dependent choices under the declared environment.
///
/// [`Self::verify`] is provided and is the only path to a
/// [`PayloadPlanDeterminismReceipt`]: an implementor supplies
/// [`Self::assess`], and cannot mint a receipt directly because the receipt has
/// no public constructor — an implementation that overrode `verify` could only
/// refuse, which is the fail-closed direction.
pub trait PayloadPlanDeterminismVerifier {
    /// The backend's own judgment over one emitted payload.
    ///
    /// An implementor must refuse — never guess — when it cannot prove the
    /// witness's program is the one this payload realizes
    /// ([`PayloadPlanDeterminismRefusal::KernelProgramMismatch`]), that the
    /// compilation subject is its own
    /// ([`PayloadPlanDeterminismRefusal::PayloadSubjectMismatch`]), that the
    /// object bytes are the ones it emitted
    /// ([`PayloadPlanDeterminismRefusal::ObjectDigestMismatch`]), that its
    /// translation makes no run-dependent choice
    /// ([`PayloadPlanDeterminismRefusal::RunDependentTranslation`]), or that
    /// the declared environment is the one its evidence binds
    /// ([`PayloadPlanDeterminismRefusal::TargetEnvironmentMismatch`]).
    ///
    /// # Errors
    ///
    /// Returns the refusal naming the unproven obligation.
    fn assess(
        &self,
        witness: &PlanDeterminismWitness<'_>,
        descriptor: &BackendPayloadDescriptor,
        object_bytes: &[u8],
        declaration: &ValidatedTargetEnvironmentDeclaration,
    ) -> Result<(), PayloadPlanDeterminismRefusal>;

    /// Verifies one payload and mints its proof-bound receipt.
    ///
    /// The receipt's bound values are derived here from the exact inputs —
    /// never restated by the implementor — so a receipt always names the
    /// witness's program identity, the descriptor's compilation subject, the
    /// governed section digest of the exact object bytes, and the identity the
    /// validated declaration resolves to under the descriptor's own profile,
    /// backend, and representation.
    ///
    /// # Errors
    ///
    /// Returns the implementor's [`PayloadPlanDeterminismRefusal`].
    fn verify(
        &self,
        witness: &PlanDeterminismWitness<'_>,
        descriptor: &BackendPayloadDescriptor,
        object_bytes: &[u8],
        declaration: &ValidatedTargetEnvironmentDeclaration,
    ) -> Result<PayloadPlanDeterminismReceipt, PayloadPlanDeterminismRefusal> {
        self.assess(witness, descriptor, object_bytes, declaration)?;
        Ok(PayloadPlanDeterminismReceipt {
            kernel_program: witness.kernel_program_identity().as_bytes().to_vec(),
            payload_subject: descriptor.digest.clone(),
            object_section_digest: super::codec::payload_code_section_digest(object_bytes),
            environment: declaration.compatibility_identity(
                &descriptor.compatibility,
                &descriptor.backend,
                &descriptor.representation,
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::super::keys::{TargetProfileDescriptorDigest, TargetProfileKey};
    use super::*;

    fn provider(revision: u32) -> ProviderIdentity {
        ProviderIdentity::new("tiler-test", "environment-authority", revision).unwrap()
    }

    fn declaration_of(
        provider: ProviderIdentity,
        schema: SchemaVersion,
        descriptor: &[u8],
    ) -> TargetEnvironmentDeclaration {
        TargetEnvironmentDeclaration::new(
            provider,
            schema,
            TargetEnvironmentDescriptor::new(descriptor).unwrap(),
        )
        .unwrap()
    }

    /// One registration admitting exactly the given spelling.
    struct ExactSchema {
        provider: ProviderIdentity,
        schema: SchemaVersion,
        admitted: Vec<u8>,
    }

    impl TargetEnvironmentDescriptorSchema for ExactSchema {
        fn provider(&self) -> &ProviderIdentity {
            &self.provider
        }

        fn schema_version(&self) -> SchemaVersion {
            self.schema
        }

        fn validate_canonical_descriptor(
            &self,
            descriptor: &[u8],
        ) -> Result<(), TargetEnvironmentReasonCode> {
            if descriptor == self.admitted {
                Ok(())
            } else {
                Err(TargetEnvironmentReasonCode::new("descriptor-not-canonical").unwrap())
            }
        }
    }

    fn schema_admitting(declaration: &TargetEnvironmentDeclaration) -> ExactSchema {
        ExactSchema {
            provider: declaration.provider().clone(),
            schema: declaration.descriptor_schema(),
            admitted: declaration.descriptor().as_bytes().to_vec(),
        }
    }

    /// A schema major of zero is refused at declaration.
    #[test]
    fn a_zero_schema_major_declaration_is_refused() {
        assert_eq!(
            TargetEnvironmentDeclaration::new(
                provider(1),
                SchemaVersion::new(0, 4),
                TargetEnvironmentDescriptor::new(b"x").unwrap(),
            )
            .unwrap_err(),
            TargetEnvironmentDeclarationError::ZeroSchemaMajor,
        );
    }

    /// The governed descriptor bound refuses one byte past it and admits it exactly.
    #[test]
    fn the_descriptor_bound_is_exact() {
        let at_limit = vec![0x61; MAX_TARGET_ENVIRONMENT_DESCRIPTOR_BYTES];
        assert!(TargetEnvironmentDescriptor::new(&at_limit).is_ok());
        assert_eq!(
            TargetEnvironmentDescriptor::new(vec![
                0x61;
                MAX_TARGET_ENVIRONMENT_DESCRIPTOR_BYTES + 1
            ])
            .unwrap_err(),
            TargetEnvironmentDeclarationError::DescriptorTooLong {
                bytes: MAX_TARGET_ENVIRONMENT_DESCRIPTOR_BYTES + 1,
                limit: MAX_TARGET_ENVIRONMENT_DESCRIPTOR_BYTES,
            },
        );
    }

    /// An empty descriptor is a state, not an absence.
    ///
    /// A provider whose compatibility class is carried entirely by its exact
    /// revision and schema version canonicalizes to zero bytes; whether that
    /// is sufficient is its registration obligation, not this grammar's.
    #[test]
    fn an_empty_descriptor_is_admitted() {
        let declaration = declaration_of(provider(1), SchemaVersion::new(1, 0), b"");
        assert!(
            declaration
                .validate(&schema_admitting(&declaration))
                .is_ok()
        );
    }

    /// The reason-code grammar is the governed-key alphabet, bounded.
    #[test]
    fn the_reason_code_grammar_is_the_governed_alphabet() {
        assert!(TargetEnvironmentReasonCode::new("descriptor-not-canonical.v1_x").is_ok());
        assert_eq!(
            TargetEnvironmentReasonCode::new("").unwrap_err(),
            TargetEnvironmentReasonCodeError::Empty,
        );
        assert_eq!(
            TargetEnvironmentReasonCode::new("a".repeat(MAX_TARGET_ENVIRONMENT_REASON_BYTES + 1))
                .unwrap_err(),
            TargetEnvironmentReasonCodeError::TooLong {
                bytes: MAX_TARGET_ENVIRONMENT_REASON_BYTES + 1,
            },
        );
        assert_eq!(
            TargetEnvironmentReasonCode::new("not Canonical").unwrap_err(),
            TargetEnvironmentReasonCodeError::NoncanonicalByte {
                index: 3,
                value: b' ',
            },
        );
    }

    /// Validation is exact over provider, revision, schema, and spelling.
    #[test]
    fn validation_is_exact_over_provider_schema_and_spelling() {
        let declaration = declaration_of(provider(1), SchemaVersion::new(1, 0), b"class-a");
        assert!(
            declaration
                .validate(&schema_admitting(&declaration))
                .is_ok()
        );

        // A revision step is a different authority, not a compatible one.
        let mut revised = schema_admitting(&declaration);
        revised.provider = provider(2);
        assert!(matches!(
            declaration.validate(&revised).unwrap_err(),
            TargetEnvironmentDeclarationError::ProviderMismatch { .. },
        ));

        let mut minor_stepped = schema_admitting(&declaration);
        minor_stepped.schema = SchemaVersion::new(1, 1);
        assert_eq!(
            declaration.validate(&minor_stepped).unwrap_err(),
            TargetEnvironmentDeclarationError::SchemaMismatch {
                declared: SchemaVersion::new(1, 0),
                registered: SchemaVersion::new(1, 1),
            },
        );

        // A well-formed spelling the schema does not admit surfaces the
        // schema's own bounded reason, never a silent normalization.
        let mut other_spelling = schema_admitting(&declaration);
        other_spelling.admitted = b"class-b".to_vec();
        match declaration.validate(&other_spelling).unwrap_err() {
            TargetEnvironmentDeclarationError::NoncanonicalDescriptor { reason, .. } => {
                assert_eq!(reason.as_str(), "descriptor-not-canonical");
            }
            other => panic!("expected a noncanonical-descriptor refusal, got {other:?}"),
        }
    }

    /// Every component of the accepted subject separates the identity.
    ///
    /// Population-counted: the baseline plus one perturbation per component,
    /// all pairwise distinct in canonical bytes, so no field is silently
    /// absent from the encoding.
    #[test]
    fn every_component_separates_the_compatibility_identity() {
        let profile = |key: &str, digest: &[u8]| TargetProfileRef {
            key: TargetProfileKey::new(key).unwrap(),
            descriptor: TargetProfileDescriptorDigest::from_bytes(digest).unwrap(),
        };
        let identity = |profile: &TargetProfileRef,
                        backend: &str,
                        representation: &str,
                        provider: ProviderIdentity,
                        schema: SchemaVersion,
                        descriptor: &[u8]| {
            let declaration = declaration_of(provider, schema, descriptor);
            declaration
                .validate(&schema_admitting(&declaration))
                .unwrap()
                .compatibility_identity(
                    profile,
                    &BackendKey::new(backend).unwrap(),
                    &RepresentationKey::new(representation).unwrap(),
                )
        };
        let base_profile = profile("tiler.test.baseline", &[0x01]);
        let baseline = identity(
            &base_profile,
            "tiler.metal",
            "metallib",
            provider(1),
            SchemaVersion::new(1, 0),
            b"class-a",
        );
        let perturbed = [
            (
                "profile key",
                identity(
                    &profile("tiler.test.other", &[0x01]),
                    "tiler.metal",
                    "metallib",
                    provider(1),
                    SchemaVersion::new(1, 0),
                    b"class-a",
                ),
            ),
            (
                "profile descriptor",
                identity(
                    &profile("tiler.test.baseline", &[0x02]),
                    "tiler.metal",
                    "metallib",
                    provider(1),
                    SchemaVersion::new(1, 0),
                    b"class-a",
                ),
            ),
            (
                "backend",
                identity(
                    &base_profile,
                    "tiler.cuda",
                    "metallib",
                    provider(1),
                    SchemaVersion::new(1, 0),
                    b"class-a",
                ),
            ),
            (
                "representation",
                identity(
                    &base_profile,
                    "tiler.metal",
                    "air",
                    provider(1),
                    SchemaVersion::new(1, 0),
                    b"class-a",
                ),
            ),
            (
                "provider namespace",
                identity(
                    &base_profile,
                    "tiler.metal",
                    "metallib",
                    ProviderIdentity::new("tiler-other", "environment-authority", 1).unwrap(),
                    SchemaVersion::new(1, 0),
                    b"class-a",
                ),
            ),
            (
                "provider name",
                identity(
                    &base_profile,
                    "tiler.metal",
                    "metallib",
                    ProviderIdentity::new("tiler-test", "other-authority", 1).unwrap(),
                    SchemaVersion::new(1, 0),
                    b"class-a",
                ),
            ),
            (
                "provider revision",
                identity(
                    &base_profile,
                    "tiler.metal",
                    "metallib",
                    provider(2),
                    SchemaVersion::new(1, 0),
                    b"class-a",
                ),
            ),
            (
                "schema major",
                identity(
                    &base_profile,
                    "tiler.metal",
                    "metallib",
                    provider(1),
                    SchemaVersion::new(2, 0),
                    b"class-a",
                ),
            ),
            (
                "schema minor",
                identity(
                    &base_profile,
                    "tiler.metal",
                    "metallib",
                    provider(1),
                    SchemaVersion::new(1, 1),
                    b"class-a",
                ),
            ),
            (
                "descriptor",
                identity(
                    &base_profile,
                    "tiler.metal",
                    "metallib",
                    provider(1),
                    SchemaVersion::new(1, 0),
                    b"class-b",
                ),
            ),
        ];
        assert_eq!(
            perturbed.len(),
            10,
            "one perturbation per encoded component"
        );
        let mut seen: HashMap<Vec<u8>, &str> = HashMap::with_capacity(perturbed.len() + 1);
        seen.insert(baseline.as_bytes().to_vec(), "baseline");
        for (component, identity) in &perturbed {
            assert!(
                identity
                    .as_bytes()
                    .starts_with(TARGET_ENVIRONMENT_COMPATIBILITY_DOMAIN),
                "{component} left the governed domain"
            );
            if let Some(previous) = seen.insert(identity.as_bytes().to_vec(), component) {
                panic!("{component} and {previous} share one identity encoding");
            }
        }
        assert_eq!(seen.len(), 11);
    }
}

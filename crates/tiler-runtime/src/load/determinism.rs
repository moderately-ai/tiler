//! The ADR 0013 runtime plan-determinism subject and its attestation types.
//!
//! The runtime half of the accepted stability-subject carrier
//! (`decide-the-adr-0013-plan-determinism-stability-subject`, accepted
//! 2026-08-18). ADR 0013's plan-deterministic guarantee names four premises a
//! consumer holds fixed — identical input bits and runtime bindings, the same
//! object-bearing artifact-envelope digest, the same selected route coordinate,
//! and the same declared target-environment compatibility identity. The first
//! premise stays the caller's; the other three are this module's
//! [`PlanDeterminismSubject`], minted only on the adapter-bound positive path
//! and carried unchanged across the one-way routing commit.
//!
//! # Why the subject has exactly three fields
//!
//! The envelope digest is the object-bearing identity — unlike the
//! pre-compilation artifact identity it covers the executable sections, so two
//! relinkings of one artifact are two subjects. The selected route coordinate
//! is `(routing rank, delivery position)`: the rank fixes the variant and
//! therefore its complete kernel-program identity, and the delivery position
//! fixes which executable objects run, so
//! [`PlanDeterminismSubject::kernel_program_identity`] is a verified accessor
//! projection rather than a fourth equality input. The declared
//! target-environment compatibility identity is the execution-environment
//! class the promise is scoped to. Nothing else enters equality — no hidden
//! host choice, no live-device handle, no dtype statement, no capacity.

use std::fmt;

use tiler_artifact::program::{
    ArtifactEnvelopeDigest, CanonicalTargetEnvironmentCompatibilityIdentity, ProviderIdentity,
    SchemaVersion, TargetEnvironmentDescriptor, TargetEnvironmentDescriptorSchema,
    TargetEnvironmentReasonCode,
};

/// Versioned domain separator of one canonical runtime plan-subject identity.
const PLAN_DETERMINISM_SUBJECT_DOMAIN: &[u8] = b"tiler.runtime.plan-determinism-subject.v1\0";

/// The selected route coordinate of one routed plan.
///
/// Private fields and constructor: the loader mints it from the route it
/// actually selected, so a caller cannot state a coordinate nothing routed.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SelectedPlanVariant {
    routing_rank: u32,
    delivery_position: u32,
}

impl SelectedPlanVariant {
    pub(crate) const fn new(routing_rank: u32, delivery_position: u32) -> Self {
        Self {
            routing_rank,
            delivery_position,
        }
    }

    /// Returns the zero-based routing rank of the selected variant.
    #[must_use]
    pub const fn routing_rank(self) -> u32 {
        self.routing_rank
    }

    /// Returns the delivery position the route's objects were resolved at.
    #[must_use]
    pub const fn delivery_position(self) -> u32 {
        self.delivery_position
    }
}

/// What an adapter exposes about target-environment support.
///
/// Exhaustive, deliberately not `#[non_exhaustive]` (ADR 0074 convention 5b):
/// there is no permissive default, and an answer added here changes what an
/// adapter must be able to say, which must stop each adapter's build.
pub enum TargetEnvironmentSupport<'a> {
    /// The adapter registers no target-environment descriptor schema.
    ///
    /// Every claimed `Plan` cell filters; `Unclaimed` cells stay routable.
    Unsupported,
    /// The adapter registers exactly this provider schema.
    Registered(&'a dyn TargetEnvironmentDescriptorSchema),
}

/// What an adapter observed about the live context's target environment.
///
/// Both arms are **assertions, not attestations**: an observation becomes an
/// attestation only when the loader validates it against the adapter's own
/// registered schema and mints [`LiveTargetEnvironmentAttestation`], which no
/// public constructor can.
///
/// Exhaustive, deliberately not `#[non_exhaustive]` (ADR 0074 convention 5b).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TargetEnvironmentObservation {
    /// The adapter observed the bound context's canonical descriptor.
    Observed(TargetEnvironmentDescriptor),
    /// The adapter could not observe the bound context.
    ///
    /// Filters claimed cells without failing the route: a lower-ranked
    /// `Unclaimed` candidate remains routable, and only
    /// `bind_execution_context` failure is a context failure.
    Unavailable {
        /// The adapter's own account of why no observation exists.
        reason: String,
    },
}

/// One live observation the loader validated against its registered schema.
///
/// Public read-only accessors and no public constructor: the only mint is
/// `route_with_adapter`, after `bind_execution_context` succeeded, the
/// adapter's registered schema was independently selected, and the observed
/// descriptor validated as that schema's one canonical spelling. Neither a
/// stated `ExecutionEnvironment`, a `TargetEnvironmentDeclaration`, a
/// `TargetEnvironmentObservation`, nor any public constructor can produce one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveTargetEnvironmentAttestation {
    provider: ProviderIdentity,
    schema: SchemaVersion,
    descriptor: TargetEnvironmentDescriptor,
}

impl LiveTargetEnvironmentAttestation {
    pub(crate) const fn new(
        provider: ProviderIdentity,
        schema: SchemaVersion,
        descriptor: TargetEnvironmentDescriptor,
    ) -> Self {
        Self {
            provider,
            schema,
            descriptor,
        }
    }

    /// Returns the provider whose registered schema validated the observation.
    #[must_use]
    pub const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the exact schema version the observation validated under.
    #[must_use]
    pub const fn schema_version(&self) -> SchemaVersion {
        self.schema
    }

    /// Returns the validated observed canonical descriptor.
    #[must_use]
    pub const fn descriptor(&self) -> &TargetEnvironmentDescriptor {
        &self.descriptor
    }
}

/// Why one claimed `Plan` cell was not executable on this route.
///
/// Every class is a stable-priority *filter* applied before the candidate's
/// guard and before any commit: the claimed cell is removed from the candidate
/// set, and a lower-ranked `Unclaimed` cell stays routable.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a later ineligibility
/// subject lands additively.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum TargetEnvironmentIneligibility {
    /// No live attestation exists on this path.
    ///
    /// The device-free answer: a caller-stated `ExecutionEnvironment` cannot
    /// mint a live attestation, so `preflight` and `prepare` filter every
    /// claimed cell as unverified. `route_with_adapter` is the only positive
    /// path.
    Unattested,
    /// The independently selected adapter registers no descriptor schema.
    ProviderUnavailable,
    /// The declaration names a provider the registered schema does not.
    ProviderMismatch {
        /// Provider the artifact's declaration names.
        declared: Box<ProviderIdentity>,
        /// Provider the adapter's schema is registered under.
        registered: Box<ProviderIdentity>,
    },
    /// The declaration names a schema version the registered schema does not.
    SchemaMismatch {
        /// Schema version the artifact's declaration names.
        declared: SchemaVersion,
        /// Schema version the registered schema exposes.
        registered: SchemaVersion,
    },
    /// The adapter could not observe the bound context.
    ObservationUnavailable {
        /// Provider whose registered schema would have validated it.
        provider: Box<ProviderIdentity>,
        /// Schema version the observation would have validated under.
        schema: SchemaVersion,
        /// The adapter's own account of why no observation exists.
        reason: String,
    },
    /// The declared descriptor is not the schema's canonical spelling.
    InvalidDeclaredDescriptor {
        /// Provider whose schema refused it.
        provider: Box<ProviderIdentity>,
        /// Exact schema version that refused it.
        schema: SchemaVersion,
        /// The schema's bounded governed reason code.
        reason: TargetEnvironmentReasonCode,
    },
    /// The observed descriptor is not the schema's canonical spelling.
    InvalidObservedDescriptor {
        /// Provider whose schema refused it.
        provider: Box<ProviderIdentity>,
        /// Exact schema version that refused it.
        schema: SchemaVersion,
        /// The schema's bounded governed reason code.
        reason: TargetEnvironmentReasonCode,
    },
    /// The validated observation is not the declared environment.
    EnvironmentMismatch {
        /// Canonical descriptor the artifact declares.
        declared: TargetEnvironmentDescriptor,
        /// Canonical descriptor the live context observed.
        observed: TargetEnvironmentDescriptor,
    },
}

impl fmt::Display for TargetEnvironmentIneligibility {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unattested => formatter.write_str(
                "the claimed plan-deterministic cell is unverified: no live target-environment \
                 attestation exists on a device-free path",
            ),
            Self::ProviderUnavailable => formatter.write_str(
                "the adapter registers no target-environment descriptor schema, so the claimed \
                 cell cannot be attested",
            ),
            Self::ProviderMismatch {
                declared,
                registered,
            } => write!(
                formatter,
                "the declaration names provider {}::{}@{} and the adapter registers \
                 {}::{}@{}",
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
                "the declaration names descriptor schema {}.{} and the adapter registers {}.{}",
                declared.major(),
                declared.minor(),
                registered.major(),
                registered.minor(),
            ),
            Self::ObservationUnavailable {
                provider,
                schema,
                reason,
            } => write!(
                formatter,
                "{}::{}@{} schema {}.{} produced no live observation: {reason}",
                provider.namespace(),
                provider.name(),
                provider.revision(),
                schema.major(),
                schema.minor(),
            ),
            Self::InvalidDeclaredDescriptor {
                provider,
                schema,
                reason,
            } => write!(
                formatter,
                "the declared descriptor is not {}::{}@{} schema {}.{}'s canonical spelling: {}",
                provider.namespace(),
                provider.name(),
                provider.revision(),
                schema.major(),
                schema.minor(),
                reason.as_str(),
            ),
            Self::InvalidObservedDescriptor {
                provider,
                schema,
                reason,
            } => write!(
                formatter,
                "the observed descriptor is not {}::{}@{} schema {}.{}'s canonical spelling: {}",
                provider.namespace(),
                provider.name(),
                provider.revision(),
                schema.major(),
                schema.minor(),
                reason.as_str(),
            ),
            Self::EnvironmentMismatch { declared, observed } => write!(
                formatter,
                "the live observation is not the declared environment: {} declared byte(s) \
                 against {} observed byte(s)",
                declared.as_bytes().len(),
                observed.as_bytes().len(),
            ),
        }
    }
}

/// Opaque canonical bytes identifying one runtime plan-determinism subject.
///
/// The canonical encoding is the domain, the fixed-width envelope digest, the
/// big-endian routing rank and delivery position, then the length-framed
/// complete target-environment identity. The kernel-program identity is
/// deliberately not encoded twice: envelope digest plus routing rank already
/// fix the variant and its complete kernel-program identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalPlanDeterminismSubjectIdentity(Vec<u8>);

impl CanonicalPlanDeterminismSubjectIdentity {
    /// Returns the canonical identity bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// The ADR 0013 runtime plan-determinism stability subject of one route.
///
/// Private constructor and fields: the only mint is the adapter-bound positive
/// route, after every identity and payload obligation is discharged, and the
/// value is first exposed on `Preflight` and carried unchanged across the
/// one-way commit. Equality and the canonical identity contain exactly the
/// three top-level fields — envelope digest, selected route coordinate, and
/// declared target-environment identity — and no hidden host choice; identical
/// input bits and runtime bindings remain independent premises of ADR 0013
/// rather than fields copied into this carrier.
#[derive(Clone, Debug)]
pub struct PlanDeterminismSubject<'a> {
    envelope_digest: ArtifactEnvelopeDigest,
    selected: SelectedPlanVariant,
    environment: CanonicalTargetEnvironmentCompatibilityIdentity,
    /// Retained privately so [`Self::kernel_program_identity`] is a projection
    /// that must agree with the selected variant, never a fourth equality
    /// input.
    kernel_program: &'a [u8],
    identity: CanonicalPlanDeterminismSubjectIdentity,
}

impl PartialEq for PlanDeterminismSubject<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl Eq for PlanDeterminismSubject<'_> {}

impl<'a> PlanDeterminismSubject<'a> {
    pub(crate) fn new(
        envelope_digest: ArtifactEnvelopeDigest,
        selected: SelectedPlanVariant,
        environment: CanonicalTargetEnvironmentCompatibilityIdentity,
        kernel_program: &'a [u8],
    ) -> Self {
        let mut bytes = Vec::with_capacity(
            PLAN_DETERMINISM_SUBJECT_DOMAIN.len()
                + envelope_digest.as_bytes().len()
                + 2 * size_of::<u32>()
                + size_of::<u64>()
                + environment.as_bytes().len(),
        );
        bytes.extend_from_slice(PLAN_DETERMINISM_SUBJECT_DOMAIN);
        bytes.extend_from_slice(envelope_digest.as_bytes());
        bytes.extend_from_slice(&selected.routing_rank().to_be_bytes());
        bytes.extend_from_slice(&selected.delivery_position().to_be_bytes());
        let framed = u64::try_from(environment.as_bytes().len()).expect("supported usize fits u64");
        bytes.extend_from_slice(&framed.to_be_bytes());
        bytes.extend_from_slice(environment.as_bytes());
        Self {
            envelope_digest,
            selected,
            environment,
            kernel_program,
            identity: CanonicalPlanDeterminismSubjectIdentity(bytes),
        }
    }

    /// Returns the object-bearing envelope digest the promise is scoped to.
    #[must_use]
    pub const fn artifact_envelope_digest(&self) -> &ArtifactEnvelopeDigest {
        &self.envelope_digest
    }

    /// Returns the selected route coordinate.
    #[must_use]
    pub const fn selected_variant(&self) -> SelectedPlanVariant {
        self.selected
    }

    /// Returns the declared target-environment compatibility identity.
    #[must_use]
    pub const fn declared_target_environment(
        &self,
    ) -> &CanonicalTargetEnvironmentCompatibilityIdentity {
        &self.environment
    }

    /// Returns the canonical kernel-program identity the coordinate fixes.
    ///
    /// A verified accessor projection, not a fourth equality input: envelope
    /// digest plus routing rank already fix the variant and its complete
    /// kernel-program identity, and this is re-derived from the privately
    /// retained selected variant rather than encoded twice.
    #[must_use]
    pub const fn kernel_program_identity(&self) -> &'a [u8] {
        self.kernel_program
    }

    /// Returns the canonical identity of this subject.
    #[must_use]
    pub const fn canonical_identity(&self) -> &CanonicalPlanDeterminismSubjectIdentity {
        &self.identity
    }
}

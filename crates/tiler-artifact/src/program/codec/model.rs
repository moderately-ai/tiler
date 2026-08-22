//! The canonical neutral artifact envelope, and its projection from a plan.
//!
//! An [`ArtifactEnvelope`] is the *packaged* form of a
//! [`VerifiedArtifactProgram`](super::super::VerifiedArtifactProgram): exactly
//! the neutral facts a runtime, a cache, or a backend assembler consumes,
//! arranged in one canonical order so that two artifacts with equal identity
//! encode to equal bytes.
//!
//! # It is not a second authority
//!
//! An envelope is produced in exactly two ways, and neither manufactures a
//! verified value. [`ArtifactEnvelope::project`] reads an artifact the checked
//! builder already verified; [`super::decode`] reads bytes and re-proves every
//! obligation that is decidable from the manifest alone. Nothing here can
//! produce a `VerifiedArtifactProgram`, because the shared-IR programs a
//! variant packages are not reconstructable from an envelope — see [`super`]'s
//! module documentation for exactly what that costs and who owns it.
//!
//! The artifact's canonical identity is derived from this type and from nothing
//! else: `super::super::model::encode_identity` is a function of an
//! `ArtifactEnvelope`, and the builder reaches it by projecting first. There is
//! therefore one identity encoder, not an encoder and a codec that agree by
//! inspection.

use std::borrow::Cow;

use tiler_ir::program::abi::compare_expr_nodes;
use tiler_ir::program::{DependencyReasonView, StageRef, VerifiedKernelProgram};
use tiler_ir::schedule::{
    ApproximationEnvelope, ExceptionalValueAssumption, NumericalPermission, NumericalRealization,
    ResourceRequirements, SubnormalMode,
};
use tiler_ir::semantic::{InputKey, OutputKey};

use super::super::environment::PlanDeterminismScope;
use super::super::error::{ArtifactDiagnostic, ArtifactEntityKind};
use super::super::expr::ExprNode;
use super::super::keys::{BackendEntryKey, FeasibilityRuleSetRef, TargetProfileRef};
use super::super::model::{
    ArtifactProgramData, ArtifactSchema, BackendPayloadDescriptor, BindingData,
    CanonicalArtifactProgramIdentity, DeferredPredicateData, ExtentOperandData, InterfaceEntryData,
    LaunchData, RoutingPolicy, SchemaVersion, SelectedLoweringProvider, StageDependencyData,
    StageDependencyReason, VariantData, canonical_deferred_order, canonical_precondition_order,
    encode_identity, stage_key,
};
use super::super::realization::codec::{
    ArtifactCrossCheck, RealizationCodecError, validate_against_artifact,
};
use super::super::realization::{DeliveredRealizationRecord, EntryRealization};
use super::super::requirement::{RouteRequirement, canonical_requirement_order};
use super::error::{ArtifactCodecError, CodecLimitKind, codec_limit};
use super::payload::{PayloadContent, encode_metadata};

/// Maximum bytes of one received opaque identity subject.
///
/// Matched to the shared IR's own canonical-work budget for a semantic program,
/// so an envelope can carry any subject `tiler-ir` can derive.
pub(super) const MAX_SUBJECT_BYTES: usize = 16 * 1024 * 1024;
/// Maximum named interface entries admitted by one artifact envelope.
pub(super) const MAX_INTERFACE_ENTRIES: usize = 4_096;
/// Maximum rank of one declared interface shape.
pub(super) const MAX_INTERFACE_SHAPE_RANK: usize = 4_096;
/// Maximum bytes of one encoded text run.
pub(super) const MAX_TEXT_BYTES: usize = 4_096;
/// Maximum required features admitted by one artifact envelope.
pub(super) const MAX_FEATURES: usize = 64;
/// Maximum framed sections admitted by one artifact envelope.
///
/// This profile frames one section per distinct packaged kernel program, so the
/// bound is the artifact model's own variant bound rather than a second budget.
pub(super) const MAX_SECTIONS: usize = super::super::MAX_ARTIFACT_VARIANTS;
/// Maximum bytes of one framed section.
///
/// A section carries one canonical kernel-program identity, so the bound is the
/// shared IR's own identity budget.
pub(super) const MAX_SECTION_BYTES: usize = 64 * 1024 * 1024;

/// Governed feature key required when a portfolio carries more than one variant.
pub(super) const FEATURE_MULTI_VARIANT_ROUTING: &str =
    "tiler.artifact.feature.multi-variant-routing";
/// Governed feature key required when any variant defers a feasibility predicate.
pub(super) const FEATURE_DEFERRED_PREDICATES: &str = "tiler.artifact.feature.deferred-predicates";
/// Governed feature key required when any entry declares a launch precondition.
pub(super) const FEATURE_LAUNCH_PRECONDITIONS: &str = "tiler.artifact.feature.launch-preconditions";
/// Governed feature key required when any variant dispatches more than one stage.
///
/// Emitted *and* supported. Until `carry-the-stage-execution-order-in-the-envelope`
/// it was emitted and refused: the neutral program section carries a program's
/// canonical identity and not its dependency graph, so a reader could not
/// recover the order in which two stages must run, and refusing was the
/// fail-closed form of that gap. The envelope now carries the execution order
/// and the typed dependency edges it discharges, both derived from the packaged
/// program, so the order is readable and checkable rather than absent.
///
/// The feature key remains because it still says something true: a reader that
/// predates those rows cannot sequence such a variant.
pub(super) const FEATURE_MULTI_STAGE_PROGRAM: &str = "tiler.artifact.feature.multi-stage-program";
/// Governed feature key required when any payload carries its object bytes.
///
/// A reader that predates carried payloads would see the descriptors and none
/// of the code, and would have no way to notice: the manifest it understands is
/// complete on its own. Requiring the feature makes such a reader refuse rather
/// than load an artifact whose executable half it silently dropped.
pub(super) const FEATURE_EMBEDDED_PAYLOAD_CODE: &str =
    "tiler.artifact.feature.embedded-payload-code";
/// Governed feature key required when the artifact declares several delivery positions.
///
/// A reader that resolves no delivery position takes the sole payload realizing
/// an entry, which is correct for the one-position artifact and silently wrong
/// for any other: it would hand a consumer whichever object happened to be
/// first, and `docs/research/apple-targets/artifact-compatibility.md` records
/// that a wrong-family object can load and dispatch without error. Refusing is
/// the fail-closed form of not knowing which position a reader is.
///
/// A one-position artifact emits no key, so the ordinary single-target artifact
/// stays readable by a reader that predates the family. The asymmetry is the
/// same one [`FEATURE_ROUTE_REQUIREMENTS`] draws, and for the same reason: one
/// position is a state such a reader can honour, and two is not.
pub(super) const FEATURE_MULTI_PAYLOAD_DELIVERY: &str =
    "tiler.artifact.feature.multi-payload-delivery";
/// Governed feature key required when any variant declares a route requirement.
///
/// Required rather than optional, and for the reason the mechanism exists: a
/// reader that predates this family would parse a manifest that looks complete
/// and would route without evaluating a device precondition the producer
/// declared. Refusing is the fail-closed form of not understanding a row.
///
/// A variant with *no* rows emits no key, so an artifact whose route needs
/// nothing additional stays readable by a reader that predates the family. That
/// asymmetry is deliberate: zero rows is a state a `9.0` reader can honour, and
/// one row is not.
pub(super) const FEATURE_ROUTE_REQUIREMENTS: &str = "tiler.artifact.feature.route-requirements";

/// Every feature key this build of the codec can *read*.
///
/// The difference between this set and the keys the projector emits is the
/// whole point of the mechanism: a producer records what an artifact needs, and
/// a reader that cannot supply it refuses rather than executing an
/// approximation.
pub(super) const SUPPORTED_FEATURES: &[&str] = &[
    FEATURE_DEFERRED_PREDICATES,
    FEATURE_EMBEDDED_PAYLOAD_CODE,
    FEATURE_LAUNCH_PRECONDITIONS,
    FEATURE_MULTI_PAYLOAD_DELIVERY,
    FEATURE_MULTI_STAGE_PROGRAM,
    FEATURE_MULTI_VARIANT_ROUTING,
    FEATURE_ROUTE_REQUIREMENTS,
];

macro_rules! received_subject {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        ///
        /// The bytes are opaque: this crate compares and encodes them and never
        /// re-derives them locally, because it is not the authority for the
        /// subject they name (ADR 0074 convention 2).
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub(crate) struct $name(Vec<u8>);

        impl $name {
            /// Wraps subject bytes another authority derived.
            ///
            /// # Errors
            ///
            /// Returns [`ArtifactCodecError::Limit`] beyond [`MAX_SUBJECT_BYTES`].
            pub(super) fn from_bytes(value: &[u8]) -> Result<Self, ArtifactCodecError> {
                codec_limit(value.len(), MAX_SUBJECT_BYTES, CodecLimitKind::SubjectBytes)?;
                Ok(Self(value.to_vec()))
            }

            /// Returns the exact subject bytes.
            pub(crate) fn as_bytes(&self) -> &[u8] {
                &self.0
            }
        }
    };
}

received_subject!(
    SemanticGraphSubject,
    "The canonical semantic-graph identity the packaged plan realizes."
);
received_subject!(
    ReachedDefinitionsSubject,
    "The provider-independent semantic definitions the plan reached."
);
received_subject!(
    AdmissionProvenanceSubject,
    "The provider-attributed admission provenance the plan reached."
);
received_subject!(
    StageSubject,
    "The canonical content key of the program stage one entry dispatches."
);

/// The reached semantic subjects an artifact envelope carries.
///
/// The frozen registry snapshot is deliberately absent. It is the subject that
/// moves when a provider the plan never used changes, and ADR 0072 keeps it out
/// of packaged artifact identity; carrying it here would put it back into the
/// envelope's bytes and therefore into its digest.
///
/// The fifth `SemanticIdentity` subject — the shape environment — travels as
/// [`super::super::retained::RetainedShapeEnvironment`], the lossless artifact
/// projection of every declaration, root binding, and semantic input
/// constraint. Invocation values are not part of those bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SemanticSubjects {
    pub(crate) graph: SemanticGraphSubject,
    pub(crate) reached_definitions: ReachedDefinitionsSubject,
    pub(crate) admission_provenance: AdmissionProvenanceSubject,
    pub(crate) retained_shape: super::super::retained::RetainedShapeEnvironment,
}

/// The declared numerical realization of one entry's bound kernel.
///
/// This mirrors [`NumericalRealization`] with an owned profile key, and the
/// split is **decided rather than pending**.
///
/// The two records sit on opposite sides of a serialization boundary and own
/// different things. A [`NumericalRealization`] is compiler IR: the only thing
/// that mints one is a compiling build, whose contract keys are its own
/// compile-time constants, so `&'static str` is what that key *is* rather than
/// a limitation. This record is a decoded dispatch record, and its key arrived
/// as bytes — which is not a narrowing of the other but the definition of the
/// boundary.
///
/// Building takes the shared-IR record directly; only decoding produces this
/// one. That asymmetry is the accepted policy that decoding yields a dispatch
/// record rather than reconstructed compiler IR, and a decoder's inability to
/// rebuild the IR record is therefore not a defect to repair.
///
/// **What would change it.** If something ever needed to turn a decoded
/// artifact back into schedulable IR, one owned record would have to cross both
/// boundaries — costing `NumericalRealization` its `Copy` and `const fn new`
/// across roughly two dozen value-semantic call sites. Nothing needs that
/// today, and paying for it in advance would make an IR type more expensive to
/// serve a use the accepted policy excludes.
///
/// The two enum vocabularies are reused rather than restated. Both are matched
/// totally by this codec, which makes them ADR 0074 convention 5b types:
/// neither carries `#[non_exhaustive]`, so widening either is a compile error
/// at the encoder instead of a silently dropped numerical fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NumericalFacts {
    pub(crate) profile_key: String,
    pub(crate) canonical_arithmetic_nan_bits: u32,
    pub(crate) input_subnormals: SubnormalMode,
    pub(crate) result_subnormals: SubnormalMode,
    pub(crate) contraction: NumericalPermission,
    pub(crate) reassociation: NumericalPermission,
    pub(crate) permutation: NumericalPermission,
    pub(crate) signed_zero: NumericalPermission,
    pub(crate) reciprocal_transform: NumericalPermission,
    pub(crate) approximate_intrinsics: ApproximationEnvelope,
    pub(crate) nan_assumptions: ExceptionalValueAssumption,
    pub(crate) infinity_assumptions: ExceptionalValueAssumption,
}

impl NumericalFacts {
    /// Projects the shared-IR realization into an owned-key record.
    fn project(numerical: NumericalRealization) -> Self {
        let NumericalRealization {
            profile_key,
            canonical_arithmetic_nan_bits,
            input_subnormals,
            result_subnormals,
            contraction,
            reassociation,
            permutation,
            signed_zero,
            reciprocal_transform,
            approximate_intrinsics,
            nan_assumptions,
            infinity_assumptions,
        } = numerical;
        Self {
            profile_key: profile_key.to_owned(),
            canonical_arithmetic_nan_bits,
            input_subnormals,
            result_subnormals,
            contraction,
            reassociation,
            permutation,
            signed_zero,
            reciprocal_transform,
            approximate_intrinsics,
            nan_assumptions,
            infinity_assumptions,
        }
    }

    /// Projects this dispatch record onto the record's cross-check subject.
    ///
    /// The second of the two sites [`EntryRealization`] exists to serve. The
    /// destructuring is exhaustive and field-named, so widening either record is
    /// a build error here rather than a cross-check that silently stops covering
    /// a dimension.
    pub(super) fn entry_realization(&self) -> EntryRealization {
        let Self {
            profile_key: _,
            canonical_arithmetic_nan_bits: _,
            input_subnormals,
            result_subnormals,
            contraction,
            reassociation,
            permutation,
            signed_zero,
            reciprocal_transform,
            approximate_intrinsics,
            nan_assumptions,
            infinity_assumptions,
        } = self;
        EntryRealization {
            input_subnormals: *input_subnormals,
            result_subnormals: *result_subnormals,
            contraction: *contraction,
            reassociation: *reassociation,
            permutation: *permutation,
            signed_zero: *signed_zero,
            reciprocal_transform: *reciprocal_transform,
            approximate_intrinsics: *approximate_intrinsics,
            nan_assumptions: *nan_assumptions,
            infinity_assumptions: *infinity_assumptions,
        }
    }
}

/// The governed purpose of one framed envelope section.
///
/// Deliberately **not** `#[non_exhaustive]` (ADR 0074 convention 5c): a section
/// purpose is a recognizer, and a backend assembler outside this crate that
/// gained a wildcard arm would silently route a newly governed purpose into
/// reject-unknown. Adding a purpose must break the build at every reader.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum SectionKind {
    /// The canonical identity of one packaged variant's kernel program.
    KernelProgramSubject,
    /// The canonical compilation subject of one carried backend payload.
    ///
    /// This section's exact bytes are the payload's identity subject; see
    /// [`super::payload`].
    BackendPayloadMetadata,
    /// The emitted object bytes of one carried backend payload.
    ///
    /// Carried opaquely. Its content digest is integrity of this encoding and
    /// is deliberately not folded into artifact identity.
    BackendPayloadCode,
}

impl SectionKind {
    pub(super) const fn tag(self) -> u8 {
        match self {
            Self::KernelProgramSubject => 0x01,
            Self::BackendPayloadMetadata => 0x02,
            Self::BackendPayloadCode => 0x03,
        }
    }

    pub(super) const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::KernelProgramSubject),
            0x02 => Some(Self::BackendPayloadMetadata),
            0x03 => Some(Self::BackendPayloadCode),
            _ => None,
        }
    }

    /// Returns the schema version of this purpose's section content.
    ///
    /// It is a property of the *purpose*, not of the instance: one purpose has
    /// one content schema in a given build. It is nonetheless written into each
    /// descriptor, because the whole point of carrying it is a reader that does
    /// not recognize the purpose and therefore cannot derive it.
    pub(super) const fn schema(self) -> SchemaVersion {
        match self {
            Self::KernelProgramSubject
            | Self::BackendPayloadMetadata
            | Self::BackendPayloadCode => SchemaVersion::new(1, 0),
        }
    }

    /// Returns whether a reader may skip this purpose when it does not know it.
    ///
    /// Every purpose this build writes is [`SectionDisposition::Required`], and
    /// an unrecognized purpose is refused outright, so no skip path exists yet.
    /// The field is written anyway: it is the mechanism the contract's
    /// "unknown optional sections may be skipped only when their schema
    /// explicitly permits it" needs, and it can only be added for free while
    /// nothing is persisted.
    pub(super) const fn disposition(self) -> SectionDisposition {
        match self {
            Self::KernelProgramSubject
            | Self::BackendPayloadMetadata
            | Self::BackendPayloadCode => SectionDisposition::Required,
        }
    }
}

/// Whether a reader that does not recognize a section's purpose may skip it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SectionDisposition {
    /// The section must be understood; not recognizing it fails closed.
    Required,
    /// The section may be skipped by a reader its schema permits to skip it.
    Optional,
}

impl SectionDisposition {
    pub(super) const fn tag(self) -> u8 {
        match self {
            Self::Required => 0x01,
            Self::Optional => 0x02,
        }
    }

    pub(super) const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::Required),
            0x02 => Some(Self::Optional),
            _ => None,
        }
    }
}

/// One framed envelope section.
///
/// A section's descriptor — its identifier, exact byte length, and content
/// digest — is *derived* from its position and bytes at encode time and
/// re-derived and compared at decode time. It is never stored beside the bytes
/// it describes, so the two can never disagree in memory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Section {
    pub(crate) kind: SectionKind,
    pub(crate) bytes: Vec<u8>,
}

impl Section {
    /// Returns the canonical order key of one framed section.
    ///
    /// Purpose precedes content so that sections of one purpose stay
    /// contiguous, and content decides within a purpose so the table is a
    /// function of what is carried rather than of declaration order.
    pub(super) fn canonical_key(&self) -> (u8, &[u8]) {
        (self.kind.tag(), &self.bytes)
    }
}

/// The two framed sections one carried backend payload occupies.
///
/// A payload that is *named* but not carried has no sections, which is the
/// descriptor-only artifact the model always admitted. A payload that is
/// carried has exactly one compilation-subject section and exactly one object
/// section, and the descriptor's digest is the identity of the first.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PayloadSections {
    pub(crate) metadata: u32,
    pub(crate) code: u32,
}

/// One executable entry of a plan variant, as packaged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EntryRow {
    pub(crate) stage: StageSubject,
    pub(crate) resources: ResourceRequirements,
    pub(crate) numerical: NumericalFacts,
    pub(crate) bindings: Vec<BindingData>,
    pub(crate) input_extents: Vec<ExtentOperandData>,
    pub(crate) launch: LaunchData,
    /// Canonical payload positions realizing this entry, one per delivery position.
    ///
    /// Order is meaning — position `p` is what a consumer's build target
    /// resolves to — so this run is retained as stated rather than canonicalized
    /// like the payload table it indexes.
    pub(crate) payloads: Vec<u32>,
    pub(crate) entry_key: BackendEntryKey,
}

/// One complete plan variant, as packaged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VariantRow {
    pub(crate) program_section: u32,
    pub(crate) guard: u32,
    pub(crate) profile: TargetProfileRef,
    pub(crate) feasibility_rules: FeasibilityRuleSetRef,
    pub(crate) deferred: Vec<DeferredPredicateData>,
    /// Additional requirements this variant's route places on a live device.
    ///
    /// Canonically ordered and distinct by subject. Ordered by canonical content
    /// rather than by declaration, because which precondition a producer
    /// happened to enumerate first is not meaning.
    pub(crate) route_requirements: Vec<RouteRequirement>,
    pub(crate) entries: Vec<EntryRow>,
    /// Positions of [`Self::entries`] in the order a consumer must dispatch them.
    ///
    /// A permutation of the entry table. Entry order in that table is canonical
    /// stage-key order — identity's order — which is deliberately not execution
    /// order, so without this row a consumer holding only bytes cannot sequence
    /// a variant that dispatches more than one stage.
    ///
    /// Derived from the packaged program's own `execution_order()`, which the
    /// shared IR documents as "a deterministic topological order of the
    /// dependency graph, broken by canonical stage content rather than by
    /// insertion". A producer states nothing here and so can contradict nothing.
    pub(crate) execution_order: Vec<u32>,
    /// The ordering obligations [`Self::execution_order`] discharges.
    ///
    /// Carried beside the order rather than left implicit in it, because an
    /// order alone cannot be checked: it says *an* order and not *why*, so a
    /// consumer could not tell a required sequence from an incidental one, and a
    /// decoder could not refuse an order that contradicts the program. With the
    /// edges present the order is verifiable against them, which is what makes
    /// this a fact rather than a claim.
    ///
    /// Canonically ordered and distinct.
    pub(crate) dependencies: Vec<StageDependencyData>,
    /// One plan-determinism scope cell per delivery position.
    ///
    /// Length-locked to the artifact's delivery positions; `validate` re-proves
    /// the lock and every `Plan` cell's structural coherence for an envelope
    /// decoded from bytes no builder wrote.
    pub(crate) scope: Vec<PlanDeterminismScope>,
}

/// The canonical neutral artifact envelope.
///
/// Ordering is meaning wherever the model says it is and canonical everywhere
/// else. Variant order is routing priority and is retained; named interface
/// order is the semantic interface's and is retained; ABI binding order is the
/// kernel signature's and is retained. Provider, payload, deferred-predicate,
/// launch-precondition, entry, expression, and section order are all replaced
/// by the canonical content order artifact identity already uses, so an
/// artifact's envelope bytes are a function of its identity rather than of the
/// order a producer happened to declare things in.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactEnvelope {
    pub(super) schema: ArtifactSchema,
    pub(super) routing: RoutingPolicy,
    pub(super) features: Vec<String>,
    pub(super) semantic: SemanticSubjects,
    pub(super) inputs: Vec<InterfaceEntryData<InputKey>>,
    pub(super) outputs: Vec<InterfaceEntryData<OutputKey>>,
    pub(super) providers: Vec<SelectedLoweringProvider>,
    pub(super) payloads: Vec<BackendPayloadDescriptor>,
    /// Section references of each payload, aligned with `payloads`.
    pub(super) payload_content: Vec<Option<PayloadSections>>,
    pub(super) expressions: Vec<ExprNode>,
    pub(super) variants: Vec<VariantRow>,
    /// The numerical realization the packaged artifact delivered.
    ///
    /// Carried verbatim rather than re-derived: the builder already put its
    /// entry bindings in the flat canonical packaged-entry space, so a projection
    /// that remapped them again would be a second definition of that space.
    pub(super) realization: DeliveredRealizationRecord,
    pub(super) sections: Vec<Section>,
}

impl ArtifactEnvelope {
    /// Projects a verified artifact program's data into its canonical envelope.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactDiagnostic::AmbiguousCanonicalKey`] when a variant's
    /// entries do not correspond one-to-one with its program's stages, or
    /// [`ArtifactDiagnostic::IdentityLimit`] when a received subject exceeds the
    /// governed envelope bound.
    pub(crate) fn project(data: &ArtifactProgramData) -> Result<Self, ArtifactDiagnostic> {
        let ProjectedTables {
            payloads,
            payload_of,
            sections,
            program_sections,
            payload_content,
        } = project_carried(data);
        let (expressions, expression_of) = project_expressions(data);
        let mut providers = data.providers.clone();
        providers.sort_unstable_by_key(SelectedLoweringProvider::canonical_key);

        let mut variants = Vec::with_capacity(data.variants.len());
        for (declared, variant) in data.variants.iter().enumerate() {
            let stage_keys: Vec<Vec<u8>> = variant
                .program
                .stages()
                .map(|stage| stage_key(&variant.program, stage))
                .collect();
            if stage_keys.len() != variant.entries.len() {
                return Err(ArtifactDiagnostic::AmbiguousCanonicalKey {
                    entity: ArtifactEntityKind::Entry,
                });
            }
            // `entry_of[declared] = canonical`, the inverse of the stage-key
            // sort `project_entries` applies. Both the order and the edges name
            // canonical positions, because a declared ordinal is exactly the
            // transient fact this envelope replaces everywhere else.
            let entry_of = canonical_entry_positions(&stage_keys);
            let mut deferred: Vec<DeferredPredicateData> = variant
                .deferred
                .iter()
                .map(|predicate| DeferredPredicateData {
                    predicate: expression_of[position(predicate.predicate)],
                    requirement: predicate.requirement.clone(),
                    entry: entry_of[position(predicate.entry)],
                })
                .collect();
            // The shared canonical order, not a key sort: the stored order is
            // what the identity's numbering is derived from, so the two must be
            // one definition rather than two that happen to agree.
            let order = canonical_deferred_order(&expressions, &deferred);
            deferred = order
                .into_iter()
                .map(|index| deferred[index].clone())
                .collect();
            // The shared canonical order again, for the same reason: the stored
            // order is what the identity folds, so one definition rather than
            // two that happen to agree.
            let route_requirements = canonical_requirement_order(&variant.route_requirements)
                .into_iter()
                .map(|index| variant.route_requirements[index].clone())
                .collect();
            let entries = project_entries(
                variant,
                &stage_keys,
                &expression_of,
                &expressions,
                &payload_of,
            )?;
            let execution_order = project_execution_order(variant, &entry_of);
            let dependencies = project_dependencies(variant, &entry_of);

            variants.push(VariantRow {
                program_section: program_sections[declared],
                guard: expression_of[position(variant.guard)],
                profile: variant.profile.clone(),
                feasibility_rules: variant.feasibility_rules.clone(),
                deferred,
                route_requirements,
                entries,
                execution_order,
                dependencies,
                // Delivery order is meaning, so the run is carried as stated.
                scope: variant.scope.clone(),
            });
        }

        let mut envelope = Self {
            schema: data.schema,
            routing: data.routing,
            features: Vec::new(),
            semantic: project_semantic(data)?,
            inputs: data.inputs.clone(),
            outputs: data.outputs.clone(),
            providers,
            payloads,
            payload_content,
            expressions,
            variants,
            realization: data.realization.clone(),
            sections,
        };
        envelope.features = envelope.derived_features();
        Ok(envelope)
    }

    /// Projects one verified artifact program into its canonical envelope.
    ///
    /// # Errors
    ///
    /// Returns the same diagnostics as [`ArtifactEnvelope::project`].
    #[allow(
        dead_code,
        reason = "test-facing projection convenience; production projects from ArtifactProgramData while codec tests exercise the verified-program wrapper, so remove when a production caller needs that wrapper spelling"
    )]
    pub(crate) fn of(
        artifact: &super::super::VerifiedArtifactProgram,
    ) -> Result<Self, ArtifactDiagnostic> {
        Self::project(&artifact.data)
    }

    /// Assembles one envelope from decoded manifest content and framed sections.
    ///
    /// This is deliberately not a public constructor. It is reachable only from
    /// [`super::decode`], which has already proven framing, integrity, canonical
    /// form, and every structural obligation, and which re-derives and compares
    /// the artifact identity before returning the result.
    pub(super) fn from_decoded(body: super::decode::DecodedBody, sections: Vec<Section>) -> Self {
        let super::decode::DecodedBody {
            schema,
            routing,
            features,
            semantic,
            inputs,
            outputs,
            providers,
            payloads,
            payload_content,
            expressions,
            variants,
            realization,
        } = body;
        Self {
            schema,
            routing,
            features,
            semantic,
            inputs,
            outputs,
            providers,
            payloads,
            payload_content,
            expressions,
            variants,
            realization,
            sections,
        }
    }

    /// Returns the governed component schema versions the artifact was written at.
    pub(crate) const fn schema(&self) -> ArtifactSchema {
        self.schema
    }

    /// Returns the canonical routing policy of the portfolio.
    pub(crate) const fn routing_policy(&self) -> RoutingPolicy {
        self.routing
    }

    /// Returns the governed features a reader must implement, in canonical order.
    pub(crate) fn features(&self) -> &[String] {
        &self.features
    }

    /// Returns the three reached semantic subjects the artifact realizes.
    pub(crate) const fn semantic(&self) -> &SemanticSubjects {
        &self.semantic
    }

    /// Returns the named program inputs in semantic interface order.
    pub(crate) fn inputs(&self) -> &[InterfaceEntryData<InputKey>] {
        &self.inputs
    }

    /// Returns the named program outputs in semantic interface order.
    pub(crate) fn outputs(&self) -> &[InterfaceEntryData<OutputKey>] {
        &self.outputs
    }

    /// Returns the selected capability providers in canonical key order.
    pub(crate) fn providers(&self) -> &[SelectedLoweringProvider] {
        &self.providers
    }

    /// Returns the backend payload descriptors in canonical key order.
    pub(crate) fn payloads(&self) -> &[BackendPayloadDescriptor] {
        &self.payloads
    }

    /// Returns each payload's carried sections, aligned with the descriptors.
    pub(crate) fn payload_content(&self) -> &[Option<PayloadSections>] {
        &self.payload_content
    }

    /// Returns the shared ABI expression arena in canonical order.
    pub(crate) fn expressions(&self) -> &[ExprNode] {
        &self.expressions
    }

    /// Returns the plan variants in routing priority order.
    pub(crate) fn variants(&self) -> &[VariantRow] {
        &self.variants
    }

    /// Returns how many delivery positions this envelope carries a payload for.
    ///
    /// Read from the first entry, which answers for all of them: `validate`
    /// re-proves that every entry declares the same non-zero count before a
    /// decoded envelope is returned. Zero means the portfolio is empty or a
    /// variant has no entry, both of which `validate` refuses on their own
    /// terms.
    pub(crate) fn delivery_positions(&self) -> usize {
        self.variants
            .first()
            .and_then(|variant| variant.entries.first())
            .map_or(0, |entry| entry.payloads.len())
    }

    /// Returns the framed sections in canonical content order.
    pub(crate) fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// Returns the numerical realization the packaged artifact delivered.
    pub(crate) const fn realization(&self) -> &DeliveredRealizationRecord {
        &self.realization
    }

    /// Cross-checks the delivered-realization record against the artifact.
    ///
    /// One function reached from both sides of the wire, because the obligation
    /// is one obligation: `super::super::ArtifactProgramBuilder::build` runs it
    /// on the envelope it projects, and `super::validate` runs it on the envelope
    /// it decodes. Running it on the *envelope* rather than on the builder's
    /// draft is what fixes the packaged-entry ordinal space — a variant's entries
    /// are in canonical stage-key order here, and a flat walk over the variants
    /// in routing priority order is the space the record's bindings name.
    ///
    /// Three things are proved, and no more. The record's profile equals the
    /// profile of **every** packaged variant, which is the artifact's single
    /// `TargetProfileRef` — compared per variant rather than against a
    /// portfolio-wide copy, because a decoded envelope carries one profile per
    /// variant row and nothing else re-proves they agree. Every packaged entry
    /// references an existing policy subject. And the record's eight overlapping
    /// resolutions equal every bound entry's own realization statement.
    ///
    /// What it does not prove is on `super::super::validate_against_artifact`
    /// and is load-bearing: an untrusted producer can write a wholly
    /// self-consistent record, a false `NotRequired` included, and every check
    /// here passes.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactDiagnostic::DeliveredRealization`] carrying the typed
    /// cause: a profile mismatch, an unbound entry, a dangling subject
    /// reference, or an overlapping-behaviour disagreement.
    pub(crate) fn check_realization(&self) -> Result<(), ArtifactDiagnostic> {
        let realization = |cause| ArtifactDiagnostic::DeliveredRealization {
            cause: Box::new(cause),
        };
        // Every variant against the record, rather than every variant against
        // the first: the record's profile is the fixed point, so one comparison
        // per variant proves both that the record names the artifact's profile
        // and that the artifact has only one.
        for variant in &self.variants {
            if variant.profile != *self.realization.profile() {
                return Err(realization(RealizationCodecError::ProfileMismatch {
                    recorded: Box::new(self.realization.profile().clone()),
                    artifact: Box::new(variant.profile.clone()),
                }));
            }
        }
        let entries: Vec<EntryRealization> = self
            .variants
            .iter()
            .flat_map(|variant| &variant.entries)
            .map(|entry| entry.numerical.entry_realization())
            .collect();
        validate_against_artifact(
            &self.realization,
            &ArtifactCrossCheck {
                profile: self.realization.profile(),
                entries: &entries,
            },
        )
        .map_err(realization)
    }

    /// Derives the canonical identity of the artifact this envelope packages.
    ///
    /// # Errors
    ///
    /// Returns the artifact model's own diagnostic when two entities collide on
    /// a canonical key, a shared-IR variant is unrecognized, or the encoding
    /// exceeds its governed bound.
    pub(crate) fn canonical_identity(
        &self,
    ) -> Result<CanonicalArtifactProgramIdentity, ArtifactDiagnostic> {
        encode_identity(self)
    }

    /// Returns the governed features a reader must implement to use this envelope.
    ///
    /// The set is derived from content, never declared by a producer, so it
    /// cannot understate what the artifact needs.
    pub(super) fn derived_features(&self) -> Vec<String> {
        let mut features = Vec::new();
        if self.variants.len() > 1 {
            features.push(FEATURE_MULTI_VARIANT_ROUTING.to_owned());
        }
        if self
            .variants
            .iter()
            .any(|variant| !variant.deferred.is_empty())
        {
            features.push(FEATURE_DEFERRED_PREDICATES.to_owned());
        }
        if self.variants.iter().any(|variant| {
            variant
                .entries
                .iter()
                .any(|entry| !entry.launch.preconditions.is_empty())
        }) {
            features.push(FEATURE_LAUNCH_PRECONDITIONS.to_owned());
        }
        if self
            .variants
            .iter()
            .any(|variant| variant.entries.len() > 1)
        {
            features.push(FEATURE_MULTI_STAGE_PROGRAM.to_owned());
        }
        if self
            .variants
            .iter()
            .any(|variant| !variant.route_requirements.is_empty())
        {
            features.push(FEATURE_ROUTE_REQUIREMENTS.to_owned());
        }
        if self.payload_content.iter().any(Option::is_some) {
            features.push(FEATURE_EMBEDDED_PAYLOAD_CODE.to_owned());
        }
        if self.delivery_positions() > 1 {
            features.push(FEATURE_MULTI_PAYLOAD_DELIVERY.to_owned());
        }
        features.sort_unstable();
        features
    }
}

/// One arena node waiting in the canonical order's ready set.
///
/// Ordered by [`compare_expr_nodes`], with the arena position as the tie-break
/// the comparator cannot supply. Two positions compare equal only when their
/// expressions are structurally identical, which both the transactional builder
/// and [`super::decode`] refuse before an arena is ordered, so the tie-break
/// keeps the order total rather than deciding anything reachable.
struct ReadyNode<'a> {
    nodes: &'a [ExprNode],
    index: u32,
}

impl ReadyNode<'_> {
    fn order(&self, other: &Self) -> std::cmp::Ordering {
        compare_expr_nodes(self.nodes, self.index, other.index)
            .then_with(|| self.index.cmp(&other.index))
    }
}

impl PartialEq for ReadyNode<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.order(other).is_eq()
    }
}

impl Eq for ReadyNode<'_> {}

impl PartialOrd for ReadyNode<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ReadyNode<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order(other)
    }
}

/// Returns the canonical arena order as a list of positions in the source arena.
///
/// The order is the unique topological order that always emits the smallest
/// available node under [`compare_expr_nodes`]. That comparison is a total,
/// content-derived order and every node's operands precede it, so the result is
/// deterministic and independent of how a producer assembled the arena.
///
/// **The comparator rather than a key table, and the difference is a bound
/// rather than a preference.** A canonical content key frames each operand's
/// whole key inside its node's, so an arena of `d` chained nodes carries key
/// bytes quadratic in `d` — bytes a producer, or a forger, chooses. A comparison
/// walks both subtrees and stops at the first difference, so it never
/// materializes one; `compare_expr_nodes` states that as its reason for
/// existing. The two are different relations, not two spellings of one: a key
/// compares an operand's *length* before its content through the eight-byte
/// frame, while the comparator compares structure directly, so the switch moved
/// `MANIFEST_SCHEMA` to `14.0`.
pub(super) fn canonical_expression_order(nodes: &[ExprNode]) -> Vec<u32> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

    let ready_node = |index: u32| Reverse(ReadyNode { nodes, index });
    let mut dependents: Vec<Vec<u32>> = vec![Vec::new(); nodes.len()];
    let mut remaining = vec![0_usize; nodes.len()];
    for (index, node) in nodes.iter().enumerate() {
        let mut operands = node_operands(node);
        operands.sort_unstable();
        operands.dedup();
        remaining[index] = operands.len();
        for operand in operands {
            dependents[position(operand)].push(ordinal(index));
        }
    }
    let mut ready: BinaryHeap<Reverse<ReadyNode<'_>>> = (0..nodes.len())
        .filter(|index| remaining[*index] == 0)
        .map(|index| ready_node(ordinal(index)))
        .collect();
    let mut order = Vec::with_capacity(nodes.len());
    while let Some(Reverse(ReadyNode { index, .. })) = ready.pop() {
        order.push(index);
        for dependent in dependents[position(index)].clone() {
            let slot = &mut remaining[position(dependent)];
            *slot -= 1;
            if *slot == 0 {
                ready.push(ready_node(dependent));
            }
        }
    }
    debug_assert_eq!(
        order.len(),
        nodes.len(),
        "an arena whose operands precede their nodes is acyclic",
    );
    order
}

/// Returns the operand positions one arena node reads.
pub(super) fn node_operands(node: &ExprNode) -> Vec<u32> {
    match node {
        ExprNode::Root(_) => Vec::new(),
        ExprNode::Unary { operand, .. } => vec![*operand],
        ExprNode::Binary { left, right, .. } => vec![*left, *right],
        ExprNode::Select {
            condition,
            if_true,
            if_false,
        } => vec![*condition, *if_true, *if_false],
    }
}

/// Rewrites one arena node's operands through a position remapping.
fn remap_node(node: &ExprNode, remap: &[u32]) -> ExprNode {
    let at = |operand: u32| remap[position(operand)];
    match node {
        ExprNode::Root(root) => ExprNode::Root(root.clone()),
        ExprNode::Unary { op, operand } => ExprNode::Unary {
            op: *op,
            operand: at(*operand),
        },
        ExprNode::Binary { op, left, right } => ExprNode::Binary {
            op: *op,
            left: at(*left),
            right: at(*right),
        },
        ExprNode::Select {
            condition,
            if_true,
            if_false,
        } => ExprNode::Select {
            condition: at(*condition),
            if_true: at(*if_true),
            if_false: at(*if_false),
        },
    }
}

fn project_semantic(data: &ArtifactProgramData) -> Result<SemanticSubjects, ArtifactDiagnostic> {
    let subject = |bytes: &[u8]| ArtifactDiagnostic::IdentityLimit {
        bytes: bytes.len(),
        limit: MAX_SUBJECT_BYTES,
    };
    let graph = data.semantic.graph().as_bytes();
    let definitions = data.semantic.reached_definitions().as_bytes();
    let provenance = data.semantic.admission_provenance().as_bytes();
    Ok(SemanticSubjects {
        graph: SemanticGraphSubject::from_bytes(graph).map_err(|_| subject(graph))?,
        reached_definitions: ReachedDefinitionsSubject::from_bytes(definitions)
            .map_err(|_| subject(definitions))?,
        admission_provenance: AdmissionProvenanceSubject::from_bytes(provenance)
            .map_err(|_| subject(provenance))?,
        retained_shape: data.retained.clone(),
    })
}

/// Builds the content-addressed section table and its two lookup maps.
///
/// Two variants that package the same program share one section, and two
/// payloads that carry the same object share one section: content is the
/// address, so sharing is a stated property of these section purposes rather
/// than an accident of equal bytes.
///
/// `carried` supplies each canonically ordered payload's content, so the
/// returned section references are aligned with the canonical payload table
/// rather than with declaration order.
///
/// # Why the table is assembled from borrowed content
///
/// This is the publication path, and a carried object is the largest thing an
/// envelope holds — up to [`MAX_SECTION_BYTES`] of compiled library per payload.
/// The projection therefore borrows every candidate's bytes and copies only the
/// distinct survivors, once each, into the [`Section`] values the encoder reads.
/// [`ArtifactEnvelope::project`] takes `&ArtifactProgramData` and `Section` owns
/// its bytes, so that one copy is the floor rather than a chosen budget.
///
/// The table's own canonical order is what makes a borrowed lookup possible: it
/// is sorted and distinct, so `binary_search_by` resolves a `(purpose, content)`
/// key without materializing it. An owned-key map would have to hold a second
/// copy of every candidate object just to address the first.
fn project_sections(
    data: &ArtifactProgramData,
    carried: &[Option<&PayloadContent>],
) -> ProjectedSections {
    // Derived rather than borrowed, because a compilation subject is a *record*
    // in the artifact and bytes only here. Kept alive beside the table that
    // borrows them, and small: a subject is source, flags, and provenance.
    let subjects: Vec<Option<Vec<u8>>> = carried
        .iter()
        .map(|content| content.map(|content| encode_metadata(&content.metadata)))
        .collect();
    let mut contents: Vec<(u8, Cow<'_, [u8]>)> = data
        .variants
        .iter()
        .map(|variant| {
            (
                SectionKind::KernelProgramSubject.tag(),
                Cow::Borrowed(variant.program.canonical_identity().as_bytes()),
            )
        })
        .collect();
    for (content, subject) in carried.iter().zip(&subjects) {
        let (Some(content), Some(subject)) = (content, subject) else {
            continue;
        };
        contents.push((
            SectionKind::BackendPayloadMetadata.tag(),
            Cow::Borrowed(subject.as_slice()),
        ));
        contents.push((
            SectionKind::BackendPayloadCode.tag(),
            Cow::Borrowed(content.code.as_slice()),
        ));
    }
    contents.sort_unstable();
    contents.dedup();
    let section_of = |kind: SectionKind, bytes: &[u8]| {
        let canonical = contents
            .binary_search_by(|(tag, content)| (*tag, &**content).cmp(&(kind.tag(), bytes)))
            .expect("a section this projection contributed before the table was ordered");
        ordinal(canonical)
    };
    let payload_content = carried
        .iter()
        .zip(&subjects)
        .map(|(content, subject)| {
            let (Some(content), Some(subject)) = (content, subject) else {
                return None;
            };
            Some(PayloadSections {
                metadata: section_of(SectionKind::BackendPayloadMetadata, subject),
                code: section_of(SectionKind::BackendPayloadCode, &content.code),
            })
        })
        .collect();
    let programs = data
        .variants
        .iter()
        .map(|variant| {
            section_of(
                SectionKind::KernelProgramSubject,
                variant.program.canonical_identity().as_bytes(),
            )
        })
        .collect();
    let sections = contents
        .into_iter()
        .map(|(kind, bytes)| Section {
            kind: SectionKind::from_tag(kind).expect("a section purpose this encoder just wrote"),
            bytes: bytes.into_owned(),
        })
        .collect();
    ProjectedSections {
        sections,
        programs,
        payload_content,
    }
}

/// Maps each declared entry position to its canonical stage-key position.
///
/// The inverse of the sort [`project_entries`] applies, factored out so the two
/// cannot disagree about which canonical slot a declared entry landed in. Both
/// call sites derive from the same `stage_keys`, so a change to the ordering
/// rule moves them together.
pub(crate) fn canonical_entry_positions(stage_keys: &[Vec<u8>]) -> Vec<u32> {
    let mut order: Vec<usize> = (0..stage_keys.len()).collect();
    order.sort_unstable_by(|left, right| stage_keys[*left].cmp(&stage_keys[*right]));
    let mut entry_of = vec![0_u32; stage_keys.len()];
    for (canonical, declared) in order.into_iter().enumerate() {
        entry_of[declared] = ordinal(canonical);
    }
    entry_of
}

/// Reads one stage's declared position within its own program.
///
/// By identity rather than by content key: `StageRef`'s equality is the program
/// it belongs to and its position in that program, so this is exact even when
/// two stages of one program share a canonical key. Matching on `stage_key`
/// instead would be ambiguous in precisely that case.
fn declared_stage_position(program: &VerifiedKernelProgram, stage: StageRef<'_>) -> usize {
    program
        .stages()
        .position(|candidate| candidate == stage)
        .expect("a stage of this program is one of its own stages")
}

/// Projects the packaged program's execution order onto canonical entry slots.
fn project_execution_order(variant: &VariantData, entry_of: &[u32]) -> Vec<u32> {
    variant
        .program
        .execution_order()
        .map(|stage| entry_of[declared_stage_position(&variant.program, stage)])
        .collect()
}

/// Projects the packaged program's typed dependency edges onto entry slots.
///
/// Sorted and deduplicated: the edges are a set, and two edges between one pair
/// of stages for one reason are one obligation however many times the program
/// enumerated it.
fn project_dependencies(variant: &VariantData, entry_of: &[u32]) -> Vec<StageDependencyData> {
    let mut edges: Vec<StageDependencyData> = variant
        .program
        .dependencies()
        .map(|edge| StageDependencyData {
            predecessor: entry_of[declared_stage_position(&variant.program, edge.predecessor())],
            successor: entry_of[declared_stage_position(&variant.program, edge.successor())],
            // Exhaustive and wildcard-free: a widened shared-IR reason must stop
            // the build here rather than be encoded as whichever arm a catch-all
            // named, which would tell a consumer this edge is a data dependency
            // when it is not.
            reason: match edge.reason() {
                DependencyReasonView::Data(_) => StageDependencyReason::Data,
                DependencyReasonView::StorageHandoff(_) => StageDependencyReason::StorageHandoff,
            },
        })
        .collect();
    edges.sort_unstable();
    edges.dedup();
    edges
}

/// Projects one variant's executable entries into canonical stage-key order.
///
/// Entry order is canonical rather than declared: the ordinal a producer
/// happened to push an entry at is presentation, while the stage it realizes is
/// identity. Every expression reference is remapped to the canonical arena at
/// the same time, so a row never mixes a declared ordinal with a canonical one.
///
/// # Errors
///
/// Returns [`ArtifactDiagnostic::IdentityLimit`] when a stage key exceeds the
/// governed opaque-subject bound.
fn project_entries(
    variant: &VariantData,
    stage_keys: &[Vec<u8>],
    expression_of: &[u32],
    expressions: &[ExprNode],
    payload_of: &[u32],
) -> Result<Vec<EntryRow>, ArtifactDiagnostic> {
    let mut order: Vec<usize> = (0..variant.entries.len()).collect();
    order.sort_unstable_by(|left, right| stage_keys[*left].cmp(&stage_keys[*right]));
    let mut entries = Vec::with_capacity(order.len());
    for entry in order {
        let stage = variant
            .program
            .stages()
            .nth(entry)
            .expect("a verified entry names a stage of its own program");
        let source = &variant.entries[entry];
        let declared: Vec<u32> = source
            .launch
            .preconditions
            .iter()
            .map(|node| expression_of[position(*node)])
            .collect();
        // The shared canonical order, not a local sort: the identity encoder
        // reaches the same function through `variant_order`, so the stored order
        // and the order identity folds are one definition rather than two that
        // happen to agree.
        let preconditions = canonical_precondition_order(expressions, &declared);
        entries.push(EntryRow {
            stage: StageSubject::from_bytes(&stage_keys[entry]).map_err(|_| {
                ArtifactDiagnostic::IdentityLimit {
                    bytes: stage_keys[entry].len(),
                    limit: MAX_SUBJECT_BYTES,
                }
            })?,
            resources: stage.kernel().requirements(),
            numerical: NumericalFacts::project(stage.kernel().numerical()),
            bindings: source
                .bindings
                .iter()
                .map(|binding| BindingData {
                    accessible_offset: expression_of[position(binding.accessible_offset)],
                    accessible_bytes: expression_of[position(binding.accessible_bytes)],
                    ..binding.clone()
                })
                .collect(),
            input_extents: source.input_extents.clone(),
            launch: LaunchData {
                grid_threads: expression_of[position(source.launch.grid_threads)],
                threads_per_workgroup: expression_of[position(source.launch.threads_per_workgroup)],
                zero_work_skips_dispatch: source.launch.zero_work_skips_dispatch,
                preconditions,
            },
            payloads: source
                .implementation
                .payloads
                .iter()
                .map(|payload| payload_of[position(*payload)])
                .collect(),
            entry_key: source.implementation.entry_key.clone(),
        });
    }
    Ok(entries)
}

/// Projects the payload table and the section table, which are interdependent.
///
/// A carried payload's content becomes two sections, and a section reference is
/// only meaningful against the *canonical* payload order, so the two tables
/// cannot be derived independently: payloads are ordered first, and the section
/// table is built from that order.
fn project_carried(data: &ArtifactProgramData) -> ProjectedTables {
    let (payloads, payload_of, carried) = project_payloads(data);
    let ProjectedSections {
        sections,
        programs,
        payload_content,
    } = project_sections(data, &carried);
    ProjectedTables {
        payloads,
        payload_of,
        sections,
        program_sections: programs,
        payload_content,
    }
}

/// The payload and section tables, in canonical order, with their remappings.
struct ProjectedTables {
    /// Backend payload descriptors in canonical content order.
    payloads: Vec<BackendPayloadDescriptor>,
    /// Declared payload position to canonical payload position.
    payload_of: Vec<u32>,
    /// The content-addressed section table in canonical order.
    sections: Vec<Section>,
    /// The section carrying each variant's program identity, in declared order.
    ///
    /// Aligned with `data.variants` rather than keyed by identity bytes: the
    /// caller walks the declared variants, and two variants packaging one
    /// program name one section here because they contributed one content
    /// address to the table.
    program_sections: Vec<u32>,
    /// Section references of each canonically ordered payload.
    payload_content: Vec<Option<PayloadSections>>,
}

/// The section table and the two lookups a projection derives with it.
struct ProjectedSections {
    /// The content-addressed section table in canonical order.
    sections: Vec<Section>,
    /// The section carrying each variant's program identity, in declared order.
    programs: Vec<u32>,
    /// Section references of each canonically ordered payload.
    payload_content: Vec<Option<PayloadSections>>,
}

/// Sorts payload descriptors canonically and returns the declaration remapping.
///
/// The carried content travels with its descriptor, so a producer's declaration
/// order cannot separate a payload from the object it carries. It travels as a
/// *borrow*: this reorders payloads, and reordering is not a reason to copy a
/// compiled library.
fn project_payloads(
    data: &ArtifactProgramData,
) -> (
    Vec<BackendPayloadDescriptor>,
    Vec<u32>,
    Vec<Option<&PayloadContent>>,
) {
    let mut order: Vec<usize> = (0..data.payloads.len()).collect();
    order.sort_unstable_by_key(|payload| data.payloads[*payload].canonical_key());
    let mut remap = vec![0_u32; data.payloads.len()];
    for (canonical, declared) in order.iter().enumerate() {
        remap[*declared] = ordinal(canonical);
    }
    let carried = order
        .iter()
        .map(|payload| data.payload_content[*payload].as_ref())
        .collect();
    let payloads = order
        .into_iter()
        .map(|payload| data.payloads[payload].clone())
        .collect();
    (payloads, remap, carried)
}

/// Reorders the expression arena canonically and returns the remapping.
fn project_expressions(data: &ArtifactProgramData) -> (Vec<ExprNode>, Vec<u32>) {
    let order = canonical_expression_order(&data.expressions);
    let mut remap = vec![0_u32; data.expressions.len()];
    for (canonical, declared) in order.iter().enumerate() {
        remap[position(*declared)] = ordinal(canonical);
    }
    let expressions = order
        .into_iter()
        .map(|node| remap_node(&data.expressions[position(node)], &remap))
        .collect();
    (expressions, remap)
}

pub(crate) fn position(index: u32) -> usize {
    usize::try_from(index).expect("u32 fits every supported host usize")
}

pub(super) fn ordinal(index: usize) -> u32 {
    u32::try_from(index).expect("a bounded envelope table fits u32")
}

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

use std::collections::BTreeMap;

use tiler_ir::program::{DependencyReasonView, StageRef, VerifiedKernelProgram};
use tiler_ir::schedule::{
    NumericalPermission, NumericalRealization, ResourceRequirements, SubnormalMode,
};
use tiler_ir::semantic::{InputKey, OutputKey};

use super::super::error::{ArtifactDiagnostic, ArtifactEntityKind};
use super::super::expr::{ExprNode, expr_key};
use super::super::keys::{BackendEntryKey, FeasibilityRuleSetRef, TargetProfileRef};
use super::super::model::{
    ArtifactProgramData, ArtifactSchema, BackendPayloadDescriptor, BindingData,
    CanonicalArtifactProgramIdentity, DeferredPredicateData, InterfaceEntryData, LaunchData,
    RoutingPolicy, SchemaVersion, SelectedProvider, StageDependencyData, StageDependencyReason,
    VariantData, deferred_key, encode_identity, stage_key,
};
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
    FEATURE_MULTI_STAGE_PROGRAM,
    FEATURE_MULTI_VARIANT_ROUTING,
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

/// The three reached semantic subjects an artifact envelope carries.
///
/// The frozen registry snapshot is deliberately absent. It is the subject that
/// moves when a provider the plan never used changes, and ADR 0072 keeps it out
/// of packaged artifact identity; carrying it here would put it back into the
/// envelope's bytes and therefore into its digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SemanticSubjects {
    pub(crate) graph: SemanticGraphSubject,
    pub(crate) reached_definitions: ReachedDefinitionsSubject,
    pub(crate) admission_provenance: AdmissionProvenanceSubject,
}

/// The declared numerical realization of one entry's bound kernel.
///
/// This mirrors [`NumericalRealization`] with an owned profile key. The
/// shared-IR record spells its key `&'static str`, so it names a compile-time
/// constant of the producing build and cannot represent a key read from bytes;
/// `own-the-numerical-realization-profile-key` records the durable fix.
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
        } = numerical;
        Self {
            profile_key: profile_key.to_owned(),
            canonical_arithmetic_nan_bits,
            input_subnormals,
            result_subnormals,
            contraction,
            reassociation,
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
    pub(crate) launch: LaunchData,
    pub(crate) payload: u32,
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
    pub(super) providers: Vec<SelectedProvider>,
    pub(super) payloads: Vec<BackendPayloadDescriptor>,
    /// Section references of each payload, aligned with `payloads`.
    pub(super) payload_content: Vec<Option<PayloadSections>>,
    pub(super) expressions: Vec<ExprNode>,
    pub(super) variants: Vec<VariantRow>,
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
            section_of,
            payload_content,
        } = project_carried(data);
        let (expressions, expression_of) = project_expressions(data);
        let keys = expression_keys(&expressions);
        let mut providers = data.providers.clone();
        providers.sort_unstable_by_key(SelectedProvider::canonical_key);

        let mut variants = Vec::with_capacity(data.variants.len());
        for variant in &data.variants {
            let mut deferred: Vec<DeferredPredicateData> = variant
                .deferred
                .iter()
                .map(|predicate| DeferredPredicateData {
                    predicate: expression_of[position(predicate.predicate)],
                    phase: predicate.phase,
                    authority: predicate.authority.clone(),
                })
                .collect();
            deferred.sort_unstable_by_key(|predicate| deferred_key(&keys, predicate));

            let stage_keys: Vec<Vec<u8>> = variant.program.stages().map(stage_key).collect();
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
            let entries =
                project_entries(variant, &stage_keys, &expression_of, &keys, &payload_of)?;
            let execution_order = project_execution_order(variant, &entry_of);
            let dependencies = project_dependencies(variant, &entry_of);

            let content = variant.program.canonical_identity().as_bytes();
            variants.push(VariantRow {
                program_section: *section_of
                    .get(content)
                    .expect("every packaged program contributed its own section"),
                guard: expression_of[position(variant.guard)],
                profile: variant.profile.clone(),
                feasibility_rules: variant.feasibility_rules.clone(),
                deferred,
                entries,
                execution_order,
                dependencies,
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
    pub(crate) fn providers(&self) -> &[SelectedProvider] {
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

    /// Returns the framed sections in canonical content order.
    pub(crate) fn sections(&self) -> &[Section] {
        &self.sections
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
        if self.payload_content.iter().any(Option::is_some) {
            features.push(FEATURE_EMBEDDED_PAYLOAD_CODE.to_owned());
        }
        features.sort_unstable();
        features
    }
}

/// Derives every arena node's canonical content key in arena order.
pub(crate) fn expression_keys(nodes: &[ExprNode]) -> Vec<Vec<u8>> {
    let mut keys: Vec<Vec<u8>> = Vec::with_capacity(nodes.len());
    for node in nodes {
        let key = expr_key(node, &keys);
        keys.push(key);
    }
    keys
}

/// Returns the canonical arena order as a list of positions in the source arena.
///
/// The order is the unique topological order that always emits the smallest
/// available node by canonical content key. Content keys are a total order and
/// every node's operands precede it, so the result is deterministic and
/// independent of how a producer assembled the arena.
pub(super) fn canonical_expression_order(nodes: &[ExprNode], keys: &[Vec<u8>]) -> Vec<u32> {
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;

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
    let mut ready: BinaryHeap<Reverse<(&[u8], u32)>> = (0..nodes.len())
        .filter(|index| remaining[*index] == 0)
        .map(|index| Reverse((keys[index].as_slice(), ordinal(index))))
        .collect();
    let mut order = Vec::with_capacity(nodes.len());
    while let Some(Reverse((_, index))) = ready.pop() {
        order.push(index);
        for dependent in dependents[position(index)].clone() {
            let slot = &mut remaining[position(dependent)];
            *slot -= 1;
            if *slot == 0 {
                ready.push(Reverse((keys[position(dependent)].as_slice(), dependent)));
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
fn project_sections(
    data: &ArtifactProgramData,
    carried: &[Option<PayloadContent>],
) -> ProjectedSections {
    let mut contents: Vec<(u8, Vec<u8>)> = data
        .variants
        .iter()
        .map(|variant| {
            (
                SectionKind::KernelProgramSubject.tag(),
                variant.program.canonical_identity().as_bytes().to_vec(),
            )
        })
        .collect();
    let encoded: Vec<Option<(Vec<u8>, Vec<u8>)>> = carried
        .iter()
        .map(|content| {
            content
                .as_ref()
                .map(|content| (encode_metadata(&content.metadata), content.code.clone()))
        })
        .collect();
    for payload in encoded.iter().flatten() {
        contents.push((SectionKind::BackendPayloadMetadata.tag(), payload.0.clone()));
        contents.push((SectionKind::BackendPayloadCode.tag(), payload.1.clone()));
    }
    contents.sort_unstable();
    contents.dedup();
    let index: BTreeMap<(u8, Vec<u8>), u32> = contents
        .iter()
        .enumerate()
        .map(|(canonical, content)| (content.clone(), ordinal(canonical)))
        .collect();
    let payload_content = encoded
        .iter()
        .map(|payload| {
            payload.as_ref().map(|(metadata, code)| PayloadSections {
                metadata: index[&(SectionKind::BackendPayloadMetadata.tag(), metadata.clone())],
                code: index[&(SectionKind::BackendPayloadCode.tag(), code.clone())],
            })
        })
        .collect();
    let programs = index
        .iter()
        .filter(|((kind, _), _)| *kind == SectionKind::KernelProgramSubject.tag())
        .map(|((_, bytes), section)| (bytes.clone(), *section))
        .collect();
    let sections = contents
        .into_iter()
        .map(|(kind, bytes)| Section {
            kind: SectionKind::from_tag(kind).expect("a section purpose this encoder just wrote"),
            bytes,
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
fn canonical_entry_positions(stage_keys: &[Vec<u8>]) -> Vec<u32> {
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
    keys: &[Vec<u8>],
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
        let mut preconditions: Vec<u32> = source
            .launch
            .preconditions
            .iter()
            .map(|node| expression_of[position(*node)])
            .collect();
        preconditions.sort_unstable_by_key(|node| keys[position(*node)].clone());
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
                    accessible_bytes: expression_of[position(binding.accessible_bytes)],
                    ..binding.clone()
                })
                .collect(),
            launch: LaunchData {
                grid_threads: expression_of[position(source.launch.grid_threads)],
                threads_per_workgroup: expression_of[position(source.launch.threads_per_workgroup)],
                zero_work_skips_dispatch: source.launch.zero_work_skips_dispatch,
                preconditions,
            },
            payload: payload_of[position(source.implementation.payload)],
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
        section_of: programs,
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
    /// Kernel-program identity bytes to the section carrying them.
    section_of: BTreeMap<Vec<u8>, u32>,
    /// Section references of each canonically ordered payload.
    payload_content: Vec<Option<PayloadSections>>,
}

/// The section table and the two lookups a projection derives with it.
struct ProjectedSections {
    /// The content-addressed section table in canonical order.
    sections: Vec<Section>,
    /// Kernel-program identity bytes to the section carrying them.
    programs: BTreeMap<Vec<u8>, u32>,
    /// Section references of each canonically ordered payload.
    payload_content: Vec<Option<PayloadSections>>,
}

/// Sorts payload descriptors canonically and returns the declaration remapping.
///
/// The carried content travels with its descriptor, so a producer's declaration
/// order cannot separate a payload from the object it carries.
fn project_payloads(
    data: &ArtifactProgramData,
) -> (
    Vec<BackendPayloadDescriptor>,
    Vec<u32>,
    Vec<Option<PayloadContent>>,
) {
    let mut order: Vec<usize> = (0..data.payloads.len()).collect();
    order.sort_unstable_by_key(|payload| data.payloads[*payload].canonical_key());
    let mut remap = vec![0_u32; data.payloads.len()];
    for (canonical, declared) in order.iter().enumerate() {
        remap[*declared] = ordinal(canonical);
    }
    let carried = order
        .iter()
        .map(|payload| data.payload_content[*payload].clone())
        .collect();
    let payloads = order
        .into_iter()
        .map(|payload| data.payloads[payload].clone())
        .collect();
    (payloads, remap, carried)
}

/// Reorders the expression arena canonically and returns the remapping.
fn project_expressions(data: &ArtifactProgramData) -> (Vec<ExprNode>, Vec<u32>) {
    let order = canonical_expression_order(&data.expressions, &data.expression_keys);
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

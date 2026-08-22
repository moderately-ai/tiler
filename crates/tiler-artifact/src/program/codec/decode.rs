//! Bounded, fail-closed decoding and validation of one artifact envelope.
//!
//! Validation is monotonic, and each stage is a strictly weaker claim than the
//! next. Framing and integrity say only that these are the exact bytes someone
//! wrote. Canonical form says the manifest has one byte representation and this
//! is it. Structural validity says the tables close over one another. Re-proven
//! model obligations say the decoded content still satisfies the rules the
//! transactional builder proved at construction. Identity agreement says the
//! content is the artifact the manifest claims it is.
//!
//! Nothing here manufactures a verified value. A decoded [`ArtifactEnvelope`]
//! is a validated *envelope*, not a `VerifiedArtifactProgram`: the shared-IR
//! programs a variant packages are carried as their canonical identity, so a
//! decoder can prove which program an artifact names but cannot resurrect the
//! program itself.
//!
//! **That is the decided design, not a gap awaiting closure.** Tom decided on
//! 2026-07-25, on `carry-reconstructable-kernel-programs-in-the-neutral-envelope`,
//! that a decoded envelope is a dispatch record and never a reconstruction, and
//! full IR reconstruction was excluded on evidence rather than preference: the
//! registry the builder needs holds behaviour, not data, so rebuilding was
//! impossible at any encoding cost rather than merely expensive. A consumer that
//! needs a `VerifiedKernelProgram` must hold the one it compiled — permanently,
//! not until some ticket lands.

use tiler_ir::program::abi::{
    PreparedEntryTargetRequirement, PreparedEntryTargetRequirementError,
    TargetPropertyProviderIdentity, TargetPropertyQuery, TargetPropertyRequirementRelation,
};
use tiler_ir::program::{
    AlignmentRequirement, BitPackedEncoding, PackedBitOrder, PackedTailRule, StorageEncoding,
    StorageScalar,
};
use tiler_ir::schedule::{
    ArithmeticType, ExceptionalValueAssumption, FencedSpaces, ResourceRequirements,
    SubgroupRealizationSubject, SubgroupTransfer, SubgroupWidth, SynchronizationSubject,
};
use tiler_ir::semantic::{EncodedComponentRole, InputKey, OpKey, OutputKey, ProviderIdentity};
use tiler_ir::shape::{Extent, Shape, ShapeSymbol, SourcedExtent, SymbolScope};

use super::super::error::ArtifactBuildError;
use super::super::expr::{
    AbiBinaryOp, AbiRoot, AbiType, AbiUnaryOp, AvailabilityPhase, ExprNode, TargetPropertyKey,
    binary_operand_type, node_type, unary_operand_type,
};
use super::super::keys::{
    BackendEntryKey, BackendKey, CapabilityFamilyKey, FeasibilityRuleSetKey, FeasibilityRuleSetRef,
    PayloadDigest, PhysicalImplementationProposalIdentity, PhysicalRegionOccurrenceIdentity,
    RepresentationKey, RouteFeatureKey, TargetProfileDescriptorDigest, TargetProfileKey,
    TargetProfileRef,
};
use super::super::model::{
    ArtifactSchema, BINDING_TARGET_INTERNAL, BINDING_TARGET_PROGRAM_INPUT,
    BINDING_TARGET_PROGRAM_OUTPUT, BackendPayloadDescriptor, BindingData, BindingKind,
    BindingTargetData, DeferredPredicateData, InterfaceComponentData, InterfaceEntryData,
    LaunchData, LoweringCapabilitySubject, PHYSICAL_SELECTION_KEY_DOMAIN,
    PHYSICAL_SELECTION_RUN_TAG, PhysicalProposalKind, RoutingPolicy, SOURCED_EXTENT_LITERAL,
    SOURCED_EXTENT_SYMBOL, SUBGROUP_REQUIREMENT_BLOCK_TAG, SchemaVersion, SelectedLoweringProvider,
    SelectedPhysicalImplementation, StageDependencyData, StageDependencyReason,
    address_space_from_tag, approximation_envelope_from_tag, buffer_access_from_tag,
    element_type_from_tag, exceptional_assumption_from_tag, index_arithmetic_from_tag,
    memory_ordering_from_tag, permission_from_tag, storage_scalar_from_tag,
    subgroup_transfer_from_tag, subnormal_from_tag, synchronization_kind_from_tag,
    synchronization_scope_from_tag,
};
use super::super::realization::DeliveredRealizationRecord;
use super::super::realization::codec::decode as decode_realization;
use super::super::requirement::{
    BackendFeatureRequirement, RouteRequirement, RouteRequirementError, RouteResourceDimension,
    RouteResourceRequirement,
};
use super::super::{
    MAX_ABI_EXPRESSIONS, MAX_ARTIFACT_PAYLOADS, MAX_ARTIFACT_VARIANTS, MAX_DEFERRED_PREDICATES,
    MAX_DELIVERY_POSITIONS, MAX_ENTRY_BINDINGS, MAX_ENTRY_EXTENTS, MAX_LAUNCH_PRECONDITIONS,
    MAX_ROUTE_FEATURE_PAYLOAD_BYTES, MAX_ROUTE_REQUIREMENTS, MAX_SELECTED_LOWERING_PROVIDERS,
    MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS, MAX_STAGE_DEPENDENCIES, MAX_VARIANT_ENTRIES,
};
use super::encode::{
    CANONICAL_ENCODING, ENVELOPE_FORMAT, HEADER_BYTES, IDENTITY_DIGEST_DOMAIN, MAGIC,
    MANIFEST_DIGEST_DOMAIN, MANIFEST_DOMAIN, MANIFEST_SCHEMA, MAX_ENVELOPE_BYTES,
    MAX_MANIFEST_BYTES, matches_canonical_encoding, section_digest,
};
use super::error::{
    ArtifactCodecError, CodecLimitKind, ComponentSchemaKind, OrderedSubject, ReferenceSubject,
    TagSubject, codec_limit,
};
use super::model::{
    AdmissionProvenanceSubject, ArtifactEnvelope, EntryRow, MAX_FEATURES, MAX_INTERFACE_ENTRIES,
    MAX_INTERFACE_SHAPE_RANK, MAX_SECTION_BYTES, MAX_SECTIONS, MAX_TEXT_BYTES, NumericalFacts,
    PayloadSections, ReachedDefinitionsSubject, SUPPORTED_FEATURES, Section, SectionDisposition,
    SectionKind, SemanticGraphSubject, SemanticSubjects, StageSubject, VariantRow,
    canonical_expression_order, ordinal, position,
};
use super::validate::validate;
use tiler_digest::{Digest, DigestAlgorithm};

/// Decodes and fully validates one encoded artifact envelope.
///
/// # Errors
///
/// Returns the typed [`ArtifactCodecError`] naming the first boundary that
/// rejected. A rejection never yields a partially validated envelope, and a
/// framing, schema, canonical-form, structural, or identity failure is never
/// reinterpreted as a plan-applicability miss.
pub(crate) fn decode(bytes: &[u8]) -> Result<ArtifactEnvelope, ArtifactCodecError> {
    codec_limit(
        bytes.len(),
        MAX_ENVELOPE_BYTES,
        CodecLimitKind::EnvelopeBytes,
    )?;
    let mut cursor = Cursor::new(bytes);
    let header = read_header(&mut cursor, bytes.len())?;

    let manifest = cursor.take(header.manifest_bytes)?;
    if header.algorithm.digest(MANIFEST_DIGEST_DOMAIN, manifest) != header.manifest_digest {
        return Err(ArtifactCodecError::ManifestDigestMismatch);
    }
    let parsed = parse_manifest(manifest)?;
    if parsed.sections.len() != header.section_count {
        return Err(ArtifactCodecError::SectionCountMismatch {
            header: header.section_count,
            manifest: parsed.sections.len(),
        });
    }
    let (sections, section_digests) =
        read_sections(&mut cursor, &parsed.sections, header.algorithm)?;
    if cursor.remaining() != 0 {
        return Err(ArtifactCodecError::TrailingBytes {
            count: cursor.remaining(),
        });
    }

    let envelope = ArtifactEnvelope::from_decoded(parsed.body, sections);
    validate(&envelope)?;
    let derived = envelope
        .canonical_identity()
        .map_err(|cause| ArtifactCodecError::IdentityDerivation { cause })?;
    // The manifest declares its identity by digest rather than by preimage, so
    // the comparison digests the derivation rather than comparing it whole. The
    // refused set is the same one: this has always been a check on whether a
    // producer's two derivations of one artifact agree, never a check of the
    // wire against the world, and nothing downstream reads the carried bytes.
    if header
        .algorithm
        .digest(IDENTITY_DIGEST_DOMAIN, derived.as_bytes())
        != parsed.identity_digest
    {
        return Err(ArtifactCodecError::ArtifactIdentityMismatch);
    }
    // The manifest is fully understood, so re-deriving the canonical encoding
    // must reproduce the exact bytes that were read. This is the backstop that
    // makes one artifact have one byte identity: any well-formed but
    // non-canonical spelling a named check did not already catch fails here
    // rather than being silently normalized on the way in.
    //
    // The derivation is compared against the bytes run by run rather than
    // accumulated and compared whole. The claim is identical — the same encoder
    // over the same envelope — and the difference is the peak: a decode that
    // built the second copy owned two envelopes at once, which for a carried
    // payload is twice the object.
    //
    // **What it uniquely covers is the manifest's wire-only fields**, measured
    // rather than argued — see the per-form table on
    // `decide-whether-the-canonicity-re-encode-is-redundant`. Every ordered
    // collection above (sections, providers, payloads, expressions, features,
    // entries, deferred predicates, launch preconditions) is read into the model
    // verbatim and written back verbatim, so re-encoding reproduces a
    // non-canonical spelling of any of them exactly and this comparison is blind
    // to all eight; their named checks are the only thing rejecting them. What
    // the wire carries and the model does not is a different class, and there
    // this comparison is the last line: neutering the section-identifier check
    // at `read_sections` leaves *this* rejecting the forgery, and a component
    // schema declared below the governed minor is normalized to the governed
    // constant by `parse_component_schemas` and caught by nothing else at all.
    // That last one is unreachable only because every governed component minor
    // is zero today; it becomes live the moment one is raised.
    if !matches_canonical_encoding(&envelope, &derived, &section_digests, bytes)? {
        return Err(ArtifactCodecError::NonCanonicalManifest);
    }
    Ok(envelope)
}

/// The fixed framing header, read before anything is allocated for the body.
struct FramingHeader {
    algorithm: DigestAlgorithm,
    manifest_bytes: usize,
    section_count: usize,
    manifest_digest: Digest,
}

/// Reads and checks the fixed framing header.
///
/// Every bound the rest of decoding relies on is established here, before a
/// single byte of the manifest or of any section is copied.
fn read_header(
    cursor: &mut Cursor<'_>,
    supplied: usize,
) -> Result<FramingHeader, ArtifactCodecError> {
    if cursor.take(MAGIC.len())? != MAGIC {
        return Err(ArtifactCodecError::BadMagic);
    }
    let format = (cursor.u16()?, cursor.u16()?);
    if format.0 != ENVELOPE_FORMAT.0 || format.1 > ENVELOPE_FORMAT.1 {
        return Err(ArtifactCodecError::UnsupportedEnvelopeFormat {
            major: format.0,
            minor: format.1,
        });
    }
    let encoding = (cursor.u16()?, cursor.u16()?);
    if encoding.0 != CANONICAL_ENCODING.0 || encoding.1 > CANONICAL_ENCODING.1 {
        return Err(ArtifactCodecError::UnsupportedCanonicalEncoding {
            major: encoding.0,
            minor: encoding.1,
        });
    }
    let algorithm_tag = cursor.u8()?;
    let algorithm = DigestAlgorithm::from_tag(algorithm_tag)
        .ok_or(ArtifactCodecError::UnsupportedDigestAlgorithm { tag: algorithm_tag })?;
    let declared_total = cursor.u64()?;
    let actual_total = u64::try_from(supplied).expect("supported usize fits u64");
    if declared_total != actual_total {
        return Err(ArtifactCodecError::TotalLengthMismatch {
            declared: declared_total,
            actual: actual_total,
        });
    }
    let manifest_bytes = cursor.count(MAX_MANIFEST_BYTES, CodecLimitKind::ManifestBytes)?;
    let section_count = position(cursor.u32()?);
    codec_limit(section_count, MAX_SECTIONS, CodecLimitKind::Sections)?;
    let manifest_digest = Digest::from_wire(cursor.array()?);
    debug_assert_eq!(
        cursor.position, HEADER_BYTES,
        "the framing header is fixed width",
    );
    Ok(FramingHeader {
        algorithm,
        manifest_bytes,
        section_count,
        manifest_digest,
    })
}

/// Reads the framed section stream against the manifest's descriptors.
///
/// A section's bytes are retained only once its declared identifier, exact
/// length, and content digest all agree with what the manifest described.
///
/// The derived content digests are returned alongside the sections because the
/// canonicity backstop needs the same values and deriving them is the single
/// most expensive thing a decode does; see [`encode_with_identity`] for why
/// handing them on is a memoization rather than a weakened check.
///
/// [`encode_with_identity`]: super::encode::encode_with_identity
fn read_sections(
    cursor: &mut Cursor<'_>,
    descriptors: &[SectionDescriptor],
    algorithm: DigestAlgorithm,
) -> Result<(Vec<Section>, Vec<Digest>), ArtifactCodecError> {
    let mut sections = Vec::with_capacity(descriptors.len());
    let mut digests = Vec::with_capacity(descriptors.len());
    for (index, descriptor) in descriptors.iter().enumerate() {
        let declared_id = cursor.u32()?;
        if position(declared_id) != index || descriptor.id != declared_id {
            return Err(ArtifactCodecError::NonCanonicalSectionId {
                position: index,
                declared: declared_id,
            });
        }
        let framed = cursor.count(MAX_SECTION_BYTES, CodecLimitKind::SectionBytes)?;
        let framed_len = u64::try_from(framed).expect("supported usize fits u64");
        if framed_len != descriptor.exact_len {
            return Err(ArtifactCodecError::SectionLengthMismatch {
                section: declared_id,
                declared: descriptor.exact_len,
                framed: framed_len,
            });
        }
        let content = cursor.take(framed)?;
        let section = Section {
            kind: descriptor.kind,
            bytes: content.to_vec(),
        };
        let digest = section_digest(algorithm, &section);
        if digest != descriptor.digest {
            return Err(ArtifactCodecError::SectionDigestMismatch {
                section: declared_id,
            });
        }
        sections.push(section);
        digests.push(digest);
    }
    Ok((sections, digests))
}

/// The decoded manifest and the two derived values validated against it.
struct ParsedManifest {
    body: DecodedBody,
    sections: Vec<SectionDescriptor>,
    /// The producer's declared identity, carried as a digest of it.
    ///
    /// A *claim* until [`decode`] digests its own derivation and compares, in
    /// exactly the sense [`Digest::from_wire`] documents.
    identity_digest: Digest,
}

/// One section descriptor, held only until its framed bytes are validated.
struct SectionDescriptor {
    id: u32,
    kind: SectionKind,
    exact_len: u64,
    digest: Digest,
}

/// Everything the manifest carries except its derived section descriptors.
pub(super) struct DecodedBody {
    pub(super) schema: ArtifactSchema,
    pub(super) routing: RoutingPolicy,
    pub(super) features: Vec<String>,
    pub(super) semantic: SemanticSubjects,
    pub(super) inputs: Vec<InterfaceEntryData<InputKey>>,
    pub(super) outputs: Vec<InterfaceEntryData<OutputKey>>,
    pub(super) providers: Vec<SelectedLoweringProvider>,
    pub(super) payloads: Vec<BackendPayloadDescriptor>,
    /// Each payload's carried sections, aligned with `payloads`.
    pub(super) payload_content: Vec<Option<PayloadSections>>,
    pub(super) expressions: Vec<ExprNode>,
    pub(super) variants: Vec<VariantRow>,
    pub(super) realization: DeliveredRealizationRecord,
}

fn parse_manifest(bytes: &[u8]) -> Result<ParsedManifest, ArtifactCodecError> {
    let mut cursor = Cursor::new(bytes);
    if cursor.take(MANIFEST_DOMAIN.len())? != MANIFEST_DOMAIN {
        return Err(ArtifactCodecError::BadManifestDomain);
    }
    let schema_version = (cursor.u16()?, cursor.u16()?);
    if schema_version.0 != MANIFEST_SCHEMA.0 || schema_version.1 > MANIFEST_SCHEMA.1 {
        return Err(ArtifactCodecError::UnsupportedManifestSchema {
            major: schema_version.0,
            minor: schema_version.1,
        });
    }
    let schema = parse_component_schemas(&mut cursor)?;
    let routing_tag = cursor.u8()?;
    let routing = RoutingPolicy::from_tag(routing_tag).ok_or(ArtifactCodecError::UnknownTag {
        subject: TagSubject::RoutingPolicy,
        tag: routing_tag,
    })?;

    let features = cursor.vec(MAX_FEATURES, CodecLimitKind::Features, Cursor::text)?;
    require_sorted_and_distinct(&features, OrderedSubject::Feature)?;
    for feature in &features {
        if !SUPPORTED_FEATURES.contains(&feature.as_str()) {
            return Err(ArtifactCodecError::UnsupportedRequiredFeature {
                feature: feature.clone(),
            });
        }
    }

    let semantic = SemanticSubjects {
        graph: SemanticGraphSubject::from_bytes(cursor.slice()?)?,
        reached_definitions: ReachedDefinitionsSubject::from_bytes(cursor.slice()?)?,
        admission_provenance: AdmissionProvenanceSubject::from_bytes(cursor.slice()?)?,
        retained_shape: super::super::retained::RetainedShapeEnvironment::from_bytes(
            cursor.slice()?,
        )
        .map_err(ArtifactCodecError::from)?,
    };

    let inputs = read_inputs(&mut cursor)?;
    let outputs = read_outputs(&mut cursor)?;
    let providers = read_providers(&mut cursor)?;
    let (payloads, payload_content) = read_payloads(&mut cursor)?;

    let expressions = parse_expressions(&mut cursor)?;
    let variants = parse_variants(&mut cursor, expressions.len(), payloads.len())?;
    // Framed here, decoded by the record's own codec. The framing is this
    // module's obligation and the content is that module's: every canonical
    // order, reference, coverage range, tag, and provenance-completeness rule is
    // re-proven there, fail-closed, and reported as its own typed rule rather
    // than as a manifest error that lost the reason.
    let realization = decode_realization(cursor.slice()?).map_err(|cause| {
        ArtifactCodecError::DeliveredRealization {
            cause: Box::new(cause),
        }
    })?;

    let sections = parse_section_descriptors(&mut cursor)?;
    // Fixed width and unframed, like every other digest the envelope carries.
    let identity_digest = Digest::from_wire(cursor.array()?);
    if cursor.remaining() != 0 {
        return Err(ArtifactCodecError::TrailingManifestBytes {
            count: cursor.remaining(),
        });
    }

    for variant in &variants {
        section_of_kind(
            &sections,
            variant.program_section,
            SectionKind::KernelProgramSubject,
        )?;
    }
    // A carried payload names its two sections, and each must exist *and* carry
    // the purpose the reference claims. Resolving the index alone would let a
    // forged manifest point a code reference at a compilation subject: both
    // sections are well formed, both digests verify, and the artifact would
    // load with its object bytes silently replaced by its own metadata.
    for content in payload_content.iter().flatten() {
        section_of_kind(
            &sections,
            content.metadata,
            SectionKind::BackendPayloadMetadata,
        )?;
        section_of_kind(&sections, content.code, SectionKind::BackendPayloadCode)?;
    }

    Ok(ParsedManifest {
        body: DecodedBody {
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
        },
        sections,
        identity_digest,
    })
}

/// Reads the named program inputs in semantic interface order.
fn read_inputs(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<InterfaceEntryData<InputKey>>, ArtifactCodecError> {
    cursor.vec(
        MAX_INTERFACE_ENTRIES,
        CodecLimitKind::InterfaceEntries,
        |cursor| {
            Ok(InterfaceEntryData {
                key: InputKey::from_owned(cursor.text()?)
                    .map_err(|cause| ArtifactCodecError::InvalidInterfaceKey { cause })?,
                extents: cursor.sourced_extents()?,
                logical_type: cursor.slice()?.to_vec(),
                components: cursor.interface_components()?,
            })
        },
    )
}

/// Reads the named program outputs in semantic interface order.
fn read_outputs(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<InterfaceEntryData<OutputKey>>, ArtifactCodecError> {
    cursor.vec(
        MAX_INTERFACE_ENTRIES,
        CodecLimitKind::InterfaceEntries,
        |cursor| {
            Ok(InterfaceEntryData {
                key: OutputKey::from_owned(cursor.text()?)
                    .map_err(|cause| ArtifactCodecError::InvalidInterfaceKey { cause })?,
                extents: cursor.sourced_extents()?,
                logical_type: cursor.slice()?.to_vec(),
                components: cursor.interface_components()?,
            })
        },
    )
}

/// Reads the selected lowering-capability providers, proving canonical order.
pub(super) fn read_providers(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<SelectedLoweringProvider>, ArtifactCodecError> {
    let providers = cursor.vec(
        MAX_SELECTED_LOWERING_PROVIDERS,
        CodecLimitKind::SelectedLoweringProviders,
        |cursor| {
            let provider = cursor.provider()?;
            let family = CapabilityFamilyKey::from_owned(cursor.text()?)
                .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?;
            let namespace = cursor.text()?;
            let name = cursor.text()?;
            let semantic_version = cursor.u32()?;
            let operation = OpKey::from_owned(namespace, name, semantic_version)
                .map_err(|cause| ArtifactCodecError::InvalidOperationKey { cause })?;
            Ok(SelectedLoweringProvider {
                provider,
                capability: LoweringCapabilitySubject { family, operation },
                capability_revision: cursor.u32()?,
            })
        },
    )?;
    require_sorted_and_distinct(
        &providers
            .iter()
            .map(SelectedLoweringProvider::canonical_key)
            .collect::<Vec<_>>(),
        OrderedSubject::SelectedLoweringProvider,
    )?;
    Ok(providers)
}

/// Parses the section descriptor table and proves each descriptor is honest.
///
/// A descriptor carries its purpose's disposition and content schema for a
/// reader that does *not* recognize the purpose, which is the only reader that
/// cannot derive them. This reader recognizes every purpose it admits, having
/// just refused the alternative, so for it both fields are checkable against
/// its own table — and checking them is what stops a descriptor asserting a
/// schema or a skip permission the purpose does not carry.
///
/// # Errors
///
/// Returns the typed [`ArtifactCodecError`] naming the first boundary that
/// rejected: an exhausted section budget, an unrecognized purpose or
/// disposition tag, or a declared disposition or schema the purpose contradicts.
fn parse_section_descriptors(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<SectionDescriptor>, ArtifactCodecError> {
    cursor.vec(MAX_SECTIONS, CodecLimitKind::Sections, |cursor| {
        let id = cursor.u32()?;
        let tag = cursor.u8()?;
        let kind = SectionKind::from_tag(tag).ok_or(ArtifactCodecError::UnknownTag {
            subject: TagSubject::SectionKind,
            tag,
        })?;
        // The disposition and the content schema are carried for a reader that
        // does *not* recognize the purpose. This reader recognizes every
        // purpose it admits, having just rejected the alternative, so for it
        // the two fields are checkable against its own table rather than
        // informative — and checking them is what stops a descriptor asserting
        // a schema or a skip permission the purpose does not have.
        let disposition_tag = cursor.u8()?;
        let disposition = SectionDisposition::from_tag(disposition_tag).ok_or(
            ArtifactCodecError::UnknownTag {
                subject: TagSubject::SectionDisposition,
                tag: disposition_tag,
            },
        )?;
        if disposition != kind.disposition() {
            return Err(ArtifactCodecError::SectionDispositionMismatch {
                section: id,
                declared: disposition_tag,
                expected: kind.disposition().tag(),
            });
        }
        let schema = SchemaVersion::new(cursor.u16()?, cursor.u16()?);
        if schema != kind.schema() {
            return Err(ArtifactCodecError::UnsupportedSectionSchema {
                section: id,
                major: schema.major(),
                minor: schema.minor(),
            });
        }
        Ok(SectionDescriptor {
            id,
            kind,
            exact_len: cursor.u64()?,
            digest: Digest::from_wire(cursor.array()?),
        })
    })
}

/// Reads the backend payload descriptors and proves their canonical order.
///
/// Each descriptor is followed by its content reference, so a descriptor and
/// the object it names cannot be separated by a table edit that leaves both
/// individually well formed. The returned content vector is aligned with the
/// descriptors.
fn read_payloads(
    cursor: &mut Cursor<'_>,
) -> Result<(Vec<BackendPayloadDescriptor>, Vec<Option<PayloadSections>>), ArtifactCodecError> {
    let rows = cursor.vec(MAX_ARTIFACT_PAYLOADS, CodecLimitKind::Payloads, |cursor| {
        let mut descriptor = BackendPayloadDescriptor {
            // Written by the trailing environment run below; the wire orders
            // the declaration after the content reference.
            environment: None,
            backend: BackendKey::from_owned(cursor.text()?)
                .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?,
            representation: RepresentationKey::from_owned(cursor.text()?)
                .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?,
            payload_schema: SchemaVersion::new(cursor.u16()?, cursor.u16()?),
            digest: PayloadDigest::from_bytes(cursor.slice()?)
                .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?,
            compatibility: TargetProfileRef {
                key: TargetProfileKey::from_owned(cursor.text()?)
                    .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?,
                descriptor: TargetProfileDescriptorDigest::from_bytes(cursor.slice()?)
                    .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?,
            },
            execution_policy: cursor.execution_policy()?,
        };
        let content = match cursor.u8()? {
            0x00 => None,
            0x01 => Some(PayloadSections {
                metadata: cursor.u32()?,
                code: cursor.u32()?,
            }),
            tag => {
                return Err(ArtifactCodecError::UnknownTag {
                    subject: TagSubject::PayloadContent,
                    tag,
                });
            }
        };
        descriptor.environment = match cursor.u8()? {
            0x00 => None,
            0x01 => {
                let provider = cursor.provider()?;
                let descriptor_schema = SchemaVersion::new(cursor.u16()?, cursor.u16()?);
                // Bounded before the grammar sees it, so a hostile length is
                // refused as the budget it exceeded rather than as a model
                // rule. The grammar then re-proves the bound and the nonzero
                // schema major; semantic provider validation is deliberately
                // unavailable to a neutral decoder.
                let bytes = cursor.slice()?;
                codec_limit(
                    bytes.len(),
                    super::super::MAX_TARGET_ENVIRONMENT_DESCRIPTOR_BYTES,
                    CodecLimitKind::TargetEnvironmentDescriptorBytes,
                )?;
                let descriptor = super::super::TargetEnvironmentDescriptor::new(bytes)
                    .map_err(|cause| ArtifactCodecError::InvalidTargetEnvironment { cause })?;
                Some(
                    super::super::TargetEnvironmentDeclaration::new(
                        provider,
                        descriptor_schema,
                        descriptor,
                    )
                    .map_err(|cause| ArtifactCodecError::InvalidTargetEnvironment { cause })?,
                )
            }
            tag => {
                return Err(ArtifactCodecError::UnknownTag {
                    subject: TagSubject::TargetEnvironmentPresence,
                    tag,
                });
            }
        };
        Ok((descriptor, content))
    })?;
    let (payloads, payload_content): (Vec<_>, Vec<_>) = rows.into_iter().unzip();
    require_sorted_and_distinct(
        &payloads
            .iter()
            .map(BackendPayloadDescriptor::canonical_key)
            .collect::<Vec<_>>(),
        OrderedSubject::Payload,
    )?;
    Ok((payloads, payload_content))
}

/// Resolves one section reference and proves it names the purpose it claims.
fn section_of_kind(
    sections: &[SectionDescriptor],
    reference: u32,
    kind: SectionKind,
) -> Result<(), ArtifactCodecError> {
    let Some(section) = sections.get(position(reference)) else {
        return Err(ArtifactCodecError::MissingReference {
            subject: ReferenceSubject::Section,
            index: u64::from(reference),
        });
    };
    if section.kind != kind {
        return Err(ArtifactCodecError::SectionPurposeMismatch {
            section: reference,
            expected: kind.tag(),
            actual: section.kind.tag(),
        });
    }
    Ok(())
}

fn parse_component_schemas(cursor: &mut Cursor<'_>) -> Result<ArtifactSchema, ArtifactCodecError> {
    let governed = ArtifactSchema::GOVERNED;
    let mut read = [SchemaVersion::new(0, 0); 4];
    for slot in &mut read {
        *slot = SchemaVersion::new(cursor.u16()?, cursor.u16()?);
    }
    let expected = [
        (ComponentSchemaKind::Program, governed.program()),
        (
            ComponentSchemaKind::AbiExpression,
            governed.abi_expression(),
        ),
        (
            ComponentSchemaKind::GuardAndRouting,
            governed.guard_and_routing(),
        ),
        (
            ComponentSchemaKind::TargetRequirement,
            governed.target_requirement(),
        ),
    ];
    for (encoded, (component, supported)) in read.iter().zip(expected) {
        if encoded.major() != supported.major() || encoded.minor() > supported.minor() {
            return Err(ArtifactCodecError::UnsupportedComponentSchema {
                component,
                major: encoded.major(),
                minor: encoded.minor(),
            });
        }
    }
    // The versions are equal to the governed set in this lockstep profile, so
    // the artifact model's own constant is retained rather than a reassembled
    // copy that could disagree with it field by field.
    Ok(governed)
}

/// Parses the shared ABI expression arena and proves it well formed.
///
/// The three obligations are exactly the ones the transactional builder
/// discharges at insertion: every operand precedes its node, every operand has
/// the value type its operation requires, and no two nodes denote the same
/// expression. The last keeps each canonical arena position naming a distinct
/// expression, so a fixed-width position reference remains injective.
///
/// **Distinctness is recognized by shallow structural equality, and that decides
/// deep structural equality by induction** — the argument
/// `ArtifactProgramBuilder::push_node` writes out, reached here because the
/// operand check above has already proven every operand strictly precedes the
/// node naming it. Take the prefix `0..q` to be free of duplicates. If node `q`
/// denotes the same expression as some earlier `p`, their operands denote the
/// same expressions pairwise and every operand lies in that duplicate-free
/// prefix, so each pair is the *same position* and `p` and `q` are equal as
/// stored records. The converse is immediate. So a hash set over the nodes
/// themselves refuses exactly what a table of canonical content keys refused, in
/// linear space rather than in space quadratic in arena depth.
fn parse_expressions(cursor: &mut Cursor<'_>) -> Result<Vec<ExprNode>, ArtifactCodecError> {
    let count = cursor.count(MAX_ABI_EXPRESSIONS, CodecLimitKind::Expressions)?;
    let mut nodes: Vec<ExprNode> = Vec::with_capacity(count);
    let mut types: Vec<AbiType> = Vec::with_capacity(count);
    for index in 0..count {
        let node = cursor.expression_node(ordinal(index))?;
        check_node_types(&node, &types, index)?;
        types.push(node_type(&node, &types));
        nodes.push(node);
    }
    let mut seen: std::collections::HashSet<&ExprNode> = std::collections::HashSet::new();
    for node in &nodes {
        if !seen.insert(node) {
            return Err(ArtifactCodecError::DuplicateItem {
                subject: OrderedSubject::Expression,
            });
        }
    }
    if canonical_expression_order(&nodes) != (0..ordinal(nodes.len())).collect::<Vec<_>>() {
        return Err(ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::Expression,
        });
    }
    Ok(nodes)
}

/// Parses one standalone expression arena, for arena-shaped adversarial tests.
///
/// Forging a malformed arena through the envelope model is not possible for any
/// node the identity reaches: `canonical_arena_traversal` numbers every operand
/// before the node naming it, so a self- or forward-referencing operand never
/// reaches bytes. Driving the parser directly is the only way to prove the
/// rejection is the named one.
#[cfg(test)]
pub(super) fn parse_expression_arena(bytes: &[u8]) -> Result<Vec<ExprNode>, ArtifactCodecError> {
    let mut cursor = Cursor::new(bytes);
    let nodes = parse_expressions(&mut cursor)?;
    if cursor.remaining() == 0 {
        Ok(nodes)
    } else {
        Err(ArtifactCodecError::TrailingManifestBytes {
            count: cursor.remaining(),
        })
    }
}

fn check_node_types(
    node: &ExprNode,
    types: &[AbiType],
    index: usize,
) -> Result<(), ArtifactCodecError> {
    let at = |operand: u32| types[position(operand)];
    let expect = |operand: u32, expected: AbiType| {
        if at(operand) == expected {
            Ok(())
        } else {
            Err(ArtifactCodecError::ExpressionOperandType {
                node: u64::try_from(index).expect("supported usize fits u64"),
                expected,
                actual: at(operand),
            })
        }
    };
    match node {
        ExprNode::Root(_) => Ok(()),
        ExprNode::Unary { op, operand } => expect(*operand, unary_operand_type(*op)),
        ExprNode::Binary { op, left, right } => {
            expect(*left, binary_operand_type(*op))?;
            expect(*right, binary_operand_type(*op))
        }
        ExprNode::Select {
            condition,
            if_true,
            if_false,
        } => {
            expect(*condition, AbiType::Boolean)?;
            if at(*if_true) == at(*if_false) {
                Ok(())
            } else {
                Err(ArtifactCodecError::ExpressionSelectBranchType {
                    node: u64::try_from(index).expect("supported usize fits u64"),
                    if_true: at(*if_true),
                    if_false: at(*if_false),
                })
            }
        }
    }
}

fn parse_variants(
    cursor: &mut Cursor<'_>,
    expressions: usize,
    payloads: usize,
) -> Result<Vec<VariantRow>, ArtifactCodecError> {
    let count = cursor.count(MAX_ARTIFACT_VARIANTS, CodecLimitKind::Variants)?;
    let mut variants = Vec::with_capacity(count);
    for _ in 0..count {
        let program_section = cursor.u32()?;
        let guard = cursor.expression_ref(expressions)?;
        let profile = TargetProfileRef {
            key: TargetProfileKey::from_owned(cursor.text()?)
                .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?,
            descriptor: TargetProfileDescriptorDigest::from_bytes(cursor.slice()?)
                .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?,
        };
        let feasibility_rules = FeasibilityRuleSetRef {
            key: FeasibilityRuleSetKey::from_owned(cursor.text()?)
                .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?,
            revision: cursor.u32()?,
        };
        let selected_physical_implementations = parse_selected_physical_run(cursor)?;
        let deferred = cursor.vec(
            MAX_DEFERRED_PREDICATES,
            CodecLimitKind::DeferredPredicates,
            |cursor| {
                let predicate = cursor.expression_ref(expressions)?;
                let entry = cursor.u32()?;
                let key = TargetPropertyKey::from_owned(cursor.text()?).map_err(|cause| {
                    ArtifactCodecError::InvalidGovernedKey {
                        cause: cause.into(),
                    }
                })?;
                let phase = cursor.phase()?;
                let namespace = cursor.text()?;
                let name = cursor.text()?;
                let revision = cursor.u32()?;
                let provider =
                    TargetPropertyProviderIdentity::new(namespace, name, revision).map_err(
                        |cause| ArtifactCodecError::InvalidProviderIdentity { cause },
                    )?;
                let query = TargetPropertyQuery::new(key, phase, provider).map_err(|_| {
                    ArtifactCodecError::ModelRule {
                        cause: Box::new(ArtifactBuildError::UnsupportedDeferredQueryPhase {
                            phase,
                        }),
                    }
                })?;
                let required = cursor.u64()?;
                let relation_tag = cursor.u8()?;
                let relation = match relation_tag {
                    0x01 => TargetPropertyRequirementRelation::ObservedAtLeastRequired,
                    0x02 => TargetPropertyRequirementRelation::ObservedEqualsRequired,
                    0x03 => TargetPropertyRequirementRelation::RequiredImpliesObserved,
                    tag => {
                        return Err(ArtifactCodecError::UnknownTag {
                            subject: TagSubject::TargetPropertyRequirementRelation,
                            tag,
                        });
                    }
                };
                let requirement =
                    PreparedEntryTargetRequirement::new(query, required, relation).map_err(
                        |cause| ArtifactCodecError::ModelRule {
                            cause: Box::new(match cause {
                                PreparedEntryTargetRequirementError::QueryIsNotPreparedEntryScoped {
                                    available_at,
                                } => ArtifactBuildError::UnsupportedDeferredQueryPhase {
                                    phase: available_at,
                                },
                                PreparedEntryTargetRequirementError::ImplicationRequirementIsNotBoolean {
                                    required,
                                } => ArtifactBuildError::DeferredImplicationRequirementNotBoolean {
                                    required,
                                },
                            }),
                        },
                    )?;
                Ok(DeferredPredicateData {
                    predicate,
                    requirement,
                    entry,
                })
            },
        )?;
        let route_requirements = parse_route_requirements(cursor)?;
        let entries = cursor.vec(MAX_VARIANT_ENTRIES, CodecLimitKind::Entries, |cursor| {
            parse_entry(cursor, expressions, payloads)
        })?;
        // The first point the relation is decidable, and therefore where it is
        // decided: the run was read long before the entry table it is measured
        // against. Applied ahead of the deferred cross-reference, execution
        // order, and dependency checks so a variant whose stated selections
        // cannot describe its stated entries is refused as that contradiction
        // rather than as whichever later rule happened to trip.
        if selected_physical_implementations.len() > entries.len() {
            return Err(ArtifactCodecError::ModelRule {
                cause: Box::new(ArtifactBuildError::PhysicalSelectionCardinality {
                    selected: selected_physical_implementations.len(),
                    entries: entries.len(),
                }),
            });
        }
        for predicate in &deferred {
            if position(predicate.entry) >= entries.len() {
                return Err(ArtifactCodecError::MissingReference {
                    subject: ReferenceSubject::Entry,
                    index: u64::from(predicate.entry),
                });
            }
        }
        let rank = u64::try_from(variants.len()).expect("supported usize fits u64");
        let execution_order = parse_execution_order(cursor, rank, entries.len())?;
        let dependencies = parse_dependencies(cursor, rank, entries.len(), &execution_order)?;
        let scope = cursor.vec(
            MAX_DELIVERY_POSITIONS,
            CodecLimitKind::PlanDeterminismScopeCells,
            |cursor| {
                let tag = cursor.u8()?;
                super::super::environment::PlanDeterminismScope::from_tag(tag).ok_or(
                    ArtifactCodecError::UnknownTag {
                        subject: TagSubject::PlanDeterminismScope,
                        tag,
                    },
                )
            },
        )?;
        variants.push(VariantRow {
            program_section,
            guard,
            profile,
            feasibility_rules,
            selected_physical_implementations,
            deferred,
            route_requirements,
            entries,
            execution_order,
            dependencies,
            scope,
        });
    }
    Ok(variants)
}

/// Reads one variant's selected physical-implementation run.
///
/// Every reachable refusal happens before the offending row joins the vector,
/// so a refused row never leaves a partially decoded variant behind: the tag
/// and the count precede any allocation, and each row is completed and ordered
/// against its predecessor before it is pushed.
///
/// There is deliberately **no** byte budget here, per identity or in aggregate.
/// `read_header` refused a manifest over `MAX_MANIFEST_BYTES` before
/// `parse_manifest` received these borrowed bytes, and both framed identities
/// and the complete run are strict subsets of them, so such a limit would
/// declare a refusal no admitted stream can reach.
fn parse_selected_physical_run(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<SelectedPhysicalImplementation>, ArtifactCodecError> {
    let tag = cursor.u8()?;
    if tag != PHYSICAL_SELECTION_RUN_TAG {
        return Err(ArtifactCodecError::UnknownTag {
            subject: TagSubject::PhysicalSelectionRun,
            tag,
        });
    }
    // Bounded before the vector is reserved, so a forged count cannot make this
    // reader allocate for rows that are not there.
    let count = cursor.count(
        MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS,
        CodecLimitKind::SelectedPhysicalImplementations,
    )?;
    if count == 0 {
        return Err(ArtifactCodecError::ModelRule {
            cause: Box::new(ArtifactBuildError::EmptySelectedPhysicalImplementations),
        });
    }
    let mut rows: Vec<SelectedPhysicalImplementation> = Vec::with_capacity(count);
    for _ in 0..count {
        let row = parse_selected_physical_row(cursor)?;
        if let Some(previous) = rows.last() {
            match previous
                .region_occurrence
                .as_bytes()
                .cmp(row.region_occurrence.as_bytes())
            {
                std::cmp::Ordering::Less => {}
                std::cmp::Ordering::Equal => {
                    return Err(ArtifactCodecError::DuplicateItem {
                        subject: OrderedSubject::SelectedPhysicalImplementation,
                    });
                }
                std::cmp::Ordering::Greater => {
                    return Err(ArtifactCodecError::NonCanonicalOrder {
                        subject: OrderedSubject::SelectedPhysicalImplementation,
                    });
                }
            }
        }
        rows.push(row);
    }
    Ok(rows)
}

/// Reads one outer-framed selected physical-implementation row key.
///
/// The row travels as the *complete* canonical key the identity encoder writes,
/// framed by its length, so this opens a bounded nested cursor over exactly
/// those bytes. Requiring the row domain inside that frame is what stops another
/// framed key of the right length being read as this subject, and requiring the
/// nested cursor to be empty afterwards is what stops a second statement hiding
/// inside one row.
fn parse_selected_physical_row(
    cursor: &mut Cursor<'_>,
) -> Result<SelectedPhysicalImplementation, ArtifactCodecError> {
    let key = cursor.slice()?;
    let mut row = Cursor::new(key);
    if row.take(PHYSICAL_SELECTION_KEY_DOMAIN.len())? != PHYSICAL_SELECTION_KEY_DOMAIN {
        return Err(ArtifactCodecError::BadPhysicalSelectionDomain);
    }
    let region_occurrence = PhysicalRegionOccurrenceIdentity::from_bytes(row.slice()?)
        .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?;
    let implementation_proposal = PhysicalImplementationProposalIdentity::from_bytes(row.slice()?)
        .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?;
    let provider = row.provider()?;
    let kind_tag = row.u8()?;
    let proposal_kind =
        PhysicalProposalKind::from_tag(kind_tag).ok_or(ArtifactCodecError::UnknownTag {
            subject: TagSubject::PhysicalProposalKind,
            tag: kind_tag,
        })?;
    if row.remaining() != 0 {
        return Err(ArtifactCodecError::TrailingPhysicalSelectionKeyBytes {
            remaining: row.remaining(),
        });
    }
    Ok(SelectedPhysicalImplementation {
        region_occurrence,
        implementation_proposal,
        provider,
        proposal_kind,
    })
}

/// Reads one variant's live-device route requirements.
///
/// Every field is decided here rather than deferred to a consumer: an
/// unrecognized kind or dimension tag rejects by name with the tag byte, a
/// governed key that does not satisfy its grammar rejects as an invalid key, and
/// a quantity, version, or payload the vocabulary refuses rejects as the model
/// rule it broke. A reader that admitted any of them would carry a requirement no
/// adapter could decide, and the fail-closed reading of "cannot decide" is
/// "cannot route".
fn parse_route_requirements(
    cursor: &mut Cursor<'_>,
) -> Result<Vec<RouteRequirement>, ArtifactCodecError> {
    cursor.vec(
        MAX_ROUTE_REQUIREMENTS,
        CodecLimitKind::RouteRequirements,
        |cursor| match cursor.u8()? {
            0x01 => {
                let tag = cursor.u8()?;
                let dimension = RouteResourceDimension::from_tag(tag).ok_or(
                    ArtifactCodecError::UnknownTag {
                        subject: TagSubject::RouteResourceDimension,
                        tag,
                    },
                )?;
                let required = cursor.u64()?;
                let resource =
                    RouteResourceRequirement::new(dimension, required).map_err(invalid_route)?;
                Ok(RouteRequirement::Resource(resource))
            }
            0x02 => {
                let owner = BackendKey::from_owned(cursor.text()?)
                    .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?;
                let key = RouteFeatureKey::from_owned(cursor.text()?)
                    .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?;
                let version = cursor.u32()?;
                // Bounded before the vocabulary sees it, so a hostile length is
                // refused as the budget it exceeded rather than as a model rule.
                let payload = cursor.slice()?;
                codec_limit(
                    payload.len(),
                    MAX_ROUTE_FEATURE_PAYLOAD_BYTES,
                    CodecLimitKind::RouteFeaturePayloadBytes,
                )?;
                let feature = BackendFeatureRequirement::new(owner, key, version, payload)
                    .map_err(invalid_route)?;
                Ok(RouteRequirement::BackendFeature(feature))
            }
            tag => Err(ArtifactCodecError::UnknownTag {
                subject: TagSubject::RouteRequirementKind,
                tag,
            }),
        },
    )
}

/// Reports a route-requirement vocabulary rejection as this artifact's own rule.
fn invalid_route(cause: RouteRequirementError) -> ArtifactCodecError {
    ArtifactCodecError::ModelRule {
        cause: Box::new(ArtifactBuildError::InvalidRouteRequirement { cause }),
    }
}

/// Reads a variant's execution order and proves it sequences every entry once.
///
/// A permutation, checked rather than assumed. An order that omitted an entry
/// would leave a stage undispatched and an order that repeated one would
/// dispatch it twice, and a consumer following either would run a program the
/// artifact does not describe.
fn parse_execution_order(
    cursor: &mut Cursor<'_>,
    variant: u64,
    entries: usize,
) -> Result<Vec<u32>, ArtifactCodecError> {
    let order = cursor.vec(MAX_VARIANT_ENTRIES, CodecLimitKind::Entries, Cursor::u32)?;
    let mut sequenced = vec![false; entries];
    for entry in &order {
        let slot =
            sequenced
                .get_mut(position(*entry))
                .ok_or(ArtifactCodecError::MissingReference {
                    subject: ReferenceSubject::Entry,
                    index: u64::from(*entry),
                })?;
        if std::mem::replace(slot, true) {
            return Err(ArtifactCodecError::StageOrderNotAPermutation {
                variant,
                entries: u64::try_from(entries).expect("supported usize fits u64"),
                stated: u64::try_from(order.len()).expect("supported usize fits u64"),
            });
        }
    }
    if sequenced.iter().any(|entry| !entry) {
        return Err(ArtifactCodecError::StageOrderNotAPermutation {
            variant,
            entries: u64::try_from(entries).expect("supported usize fits u64"),
            stated: u64::try_from(order.len()).expect("supported usize fits u64"),
        });
    }
    Ok(order)
}

/// Reads a variant's dependency edges and proves the stated order discharges them.
///
/// This is what makes the order above a fact rather than a claim. Each edge is
/// an obligation the packaged program proved; an order that runs a successor
/// before its predecessor contradicts the artifact's own dependency graph, and
/// is refused instead of being executed as a different valid schedule.
pub(super) fn parse_dependencies(
    cursor: &mut Cursor<'_>,
    variant: u64,
    entries: usize,
    order: &[u32],
) -> Result<Vec<StageDependencyData>, ArtifactCodecError> {
    let edges = cursor.vec(
        MAX_STAGE_DEPENDENCIES,
        CodecLimitKind::StageDependencies,
        |cursor| {
            let predecessor = cursor.u32()?;
            let successor = cursor.u32()?;
            let reason = cursor.stage_dependency_reason()?;
            for endpoint in [predecessor, successor] {
                if position(endpoint) >= entries {
                    return Err(ArtifactCodecError::MissingReference {
                        subject: ReferenceSubject::Entry,
                        index: u64::from(endpoint),
                    });
                }
            }
            if predecessor == successor {
                return Err(ArtifactCodecError::StageDependencyOnItself {
                    variant,
                    entry: u64::from(predecessor),
                });
            }
            Ok(StageDependencyData {
                predecessor,
                successor,
                reason,
            })
        },
    )?;
    require_sorted_and_distinct(&edges, OrderedSubject::StageDependency)?;

    // Position within the stated order, so an edge is checked against the
    // sequence a consumer will actually follow rather than against the entry
    // table's canonical order, which carries no ordering meaning.
    let mut step = vec![0_usize; entries];
    for (step_index, entry) in order.iter().enumerate() {
        step[position(*entry)] = step_index;
    }
    for edge in &edges {
        if step[position(edge.predecessor)] >= step[position(edge.successor)] {
            return Err(ArtifactCodecError::StageDependencyOutOfOrder {
                variant,
                predecessor: u64::from(edge.predecessor),
                successor: u64::from(edge.successor),
            });
        }
    }
    Ok(edges)
}

fn parse_entry(
    cursor: &mut Cursor<'_>,
    expressions: usize,
    payloads: usize,
) -> Result<EntryRow, ArtifactCodecError> {
    let stage = StageSubject::from_bytes(cursor.slice()?)?;
    let (buffer_bindings, threads_per_workgroup, local_memory_bytes) =
        (cursor.u32()?, cursor.u32()?, cursor.u64()?);
    let requires_device_memory = cursor.boolean()?;
    let index_arithmetic = cursor.index_arithmetic()?;
    let synchronization = cursor.synchronization()?;
    // The wire grammar carries the ten floating-point rows every artifact ever
    // wrote, so the decoder always constructs the `FloatingPoint` arm: no
    // encodable artifact can state the bit-preserving-copy absence until the
    // delivery-state ticket lands its tagged entry row, and `push_resources`
    // refuses that arm on the way in.
    let numerical = tiler_ir::schedule::RegionNumericalRequirements::FloatingPoint {
        input_subnormals: cursor.subnormal()?,
        result_subnormals: cursor.subnormal()?,
        contraction: cursor.permission()?,
        reassociation: cursor.permission()?,
        permutation: cursor.permission()?,
        signed_zero: cursor.permission()?,
        reciprocal_transform: cursor.permission()?,
        approximate_intrinsics: cursor.approximation_envelope()?,
        nan_assumptions: cursor.exceptional_assumption()?,
        infinity_assumptions: cursor.exceptional_assumption()?,
    };
    let resources = ResourceRequirements {
        buffer_bindings,
        threads_per_workgroup,
        local_memory_bytes,
        requires_device_memory,
        index_arithmetic,
        synchronization,
        numerical,
        // The conditional block is physically last even though the model keeps
        // the field beside synchronization. Its absence is distinguished from
        // the following bounded text length, not from the nonzero resource tags
        // above, so reading it earlier would be ambiguous.
        subgroup: cursor.subgroup_requirement()?,
    };
    let numerical = NumericalFacts {
        profile_key: cursor.text()?,
        canonical_arithmetic_nan_bits: cursor.u32()?,
        input_subnormals: cursor.subnormal()?,
        result_subnormals: cursor.subnormal()?,
        contraction: cursor.permission()?,
        reassociation: cursor.permission()?,
        permutation: cursor.permission()?,
        signed_zero: cursor.permission()?,
        reciprocal_transform: cursor.permission()?,
        approximate_intrinsics: cursor.approximation_envelope()?,
        nan_assumptions: cursor.exceptional_assumption()?,
        infinity_assumptions: cursor.exceptional_assumption()?,
    };
    let bindings = cursor.vec(
        MAX_ENTRY_BINDINGS,
        CodecLimitKind::EntryBindings,
        |cursor| {
            Ok(BindingData {
                kind: cursor.binding_kind()?,
                storage_scalar: cursor.storage_scalar()?,
                access_type: cursor.element_type()?,
                component_role: cursor.component_role()?,
                encoding: cursor.storage_encoding()?,
                address_space: cursor.address_space()?,
                access: cursor.buffer_access()?,
                alignment: AlignmentRequirement::new(cursor.u32()?)
                    .map_err(|cause| ArtifactCodecError::InvalidAlignment { cause })?,
                target: cursor.binding_target()?,
                accessible_offset: cursor.expression_ref(expressions)?,
                accessible_bytes: cursor.expression_ref(expressions)?,
            })
        },
    )?;
    let launch = LaunchData {
        grid_threads: cursor.expression_ref(expressions)?,
        threads_per_workgroup: cursor.expression_ref(expressions)?,
        zero_work_skips_dispatch: cursor.boolean()?,
        preconditions: cursor.vec(
            MAX_LAUNCH_PRECONDITIONS,
            CodecLimitKind::LaunchPreconditions,
            |cursor| cursor.expression_ref(expressions),
        )?,
    };
    // Bounded before it is allocated against, and every reference range-checked
    // as it is read. Whether the count agrees with the rest of the artifact is
    // `validate`'s, because it is a whole-envelope obligation this row cannot
    // see.
    let realizations = cursor.vec(
        MAX_DELIVERY_POSITIONS,
        CodecLimitKind::DeliveryPositions,
        |cursor| {
            let payload = cursor.u32()?;
            if position(payload) >= payloads {
                return Err(ArtifactCodecError::MissingReference {
                    subject: ReferenceSubject::Payload,
                    index: u64::from(payload),
                });
            }
            Ok(payload)
        },
    )?;
    let entry_key = BackendEntryKey::from_bytes(cursor.slice()?)
        .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?;
    let input_extents = if cursor.peek_u8() == Some(super::super::model::INPUT_EXTENT_BLOCK_TAG) {
        let _ = cursor.u8()?;
        cursor.vec(MAX_ENTRY_EXTENTS, CodecLimitKind::EntryExtents, |cursor| {
            let key = InputKey::from_owned(cursor.text()?)
                .map_err(|cause| ArtifactCodecError::InvalidInterfaceKey { cause })?;
            let axis = tiler_ir::shape::Axis::new(cursor.u32()?);
            let tag = cursor.u8()?;
            let value_type = super::super::model::abi_type_from_tag(tag).ok_or(
                ArtifactCodecError::UnknownTag {
                    subject: TagSubject::ExtentOperandType,
                    tag,
                },
            )?;
            Ok(super::super::model::ExtentOperandData {
                key,
                axis,
                value_type,
            })
        })?
    } else {
        Vec::new()
    };
    Ok(EntryRow {
        stage,
        resources,
        numerical,
        bindings,
        input_extents,
        launch,
        payloads: realizations,
        entry_key,
    })
}

/// Proves a set-meaning collection is in canonical order with no repeat.
fn require_sorted_and_distinct<T: Ord>(
    items: &[T],
    subject: OrderedSubject,
) -> Result<(), ArtifactCodecError> {
    for pair in items.windows(2) {
        match pair[0].cmp(&pair[1]) {
            std::cmp::Ordering::Less => {}
            std::cmp::Ordering::Equal => {
                return Err(ArtifactCodecError::DuplicateItem { subject });
            }
            std::cmp::Ordering::Greater => {
                return Err(ArtifactCodecError::NonCanonicalOrder { subject });
            }
        }
    }
    Ok(())
}

/// A bounded reader over an exact byte run.
///
/// Every read is length-checked against the remaining bytes before it consumes
/// anything, and every count is checked against its governed budget before a
/// collection is reserved for it.
pub(super) struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    pub(super) const fn remaining(&self) -> usize {
        self.bytes.len() - self.position
    }

    fn peek_u8(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    pub(super) fn take(&mut self, len: usize) -> Result<&'a [u8], ArtifactCodecError> {
        let end = self
            .position
            .checked_add(len)
            .ok_or(ArtifactCodecError::Truncated {
                needed: len,
                available: self.remaining(),
            })?;
        let taken = self
            .bytes
            .get(self.position..end)
            .ok_or(ArtifactCodecError::Truncated {
                needed: len,
                available: self.remaining(),
            })?;
        self.position = end;
        Ok(taken)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], ArtifactCodecError> {
        Ok(self
            .take(N)?
            .try_into()
            .expect("a checked read of N bytes is an N-byte array"))
    }

    pub(super) fn u8(&mut self) -> Result<u8, ArtifactCodecError> {
        Ok(self.array::<1>()?[0])
    }

    pub(super) fn u16(&mut self) -> Result<u16, ArtifactCodecError> {
        Ok(u16::from_be_bytes(self.array()?))
    }

    pub(super) fn u32(&mut self) -> Result<u32, ArtifactCodecError> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, ArtifactCodecError> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    /// Reads one governed count and proves it fits its budget before use.
    fn count(
        &mut self,
        limit: usize,
        resource: CodecLimitKind,
    ) -> Result<usize, ArtifactCodecError> {
        let declared = self.u64()?;
        let count = usize::try_from(declared).map_err(|_| ArtifactCodecError::Limit {
            resource,
            actual: declared,
            limit: u64::try_from(limit).expect("supported usize fits u64"),
        })?;
        codec_limit(count, limit, resource)?;
        Ok(count)
    }

    /// Reads one length-prefixed opaque byte run.
    ///
    /// The declared length is checked against what remains before anything is
    /// consumed, so a forged length reports truncation rather than reserving
    /// memory for content that is not there. The semantic bound on each such
    /// run belongs to the constructor that wraps it.
    pub(super) fn slice(&mut self) -> Result<&'a [u8], ArtifactCodecError> {
        let declared = self.u64()?;
        let available = self.remaining();
        let len = usize::try_from(declared).map_err(|_| ArtifactCodecError::Truncated {
            needed: available.saturating_add(1),
            available,
        })?;
        self.take(len)
    }

    pub(super) fn text(&mut self) -> Result<String, ArtifactCodecError> {
        let len = self.count(MAX_TEXT_BYTES, CodecLimitKind::TextBytes)?;
        let bytes = self.take(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| ArtifactCodecError::InvalidText)
    }

    fn shape(&mut self) -> Result<Shape, ArtifactCodecError> {
        let rank = self.count(MAX_INTERFACE_SHAPE_RANK, CodecLimitKind::ShapeRank)?;
        let mut extents = Vec::with_capacity(rank);
        for _ in 0..rank {
            extents.push(self.u64()?);
        }
        Shape::try_from_dims(extents).map_err(|cause| ArtifactCodecError::InvalidShape { cause })
    }

    /// Reads one published interface boundary as a run of tagged axes.
    ///
    /// The inverse of `super::super::model::push_sourced_extents`. The tag is
    /// unconditional, so a wholly literal boundary is read by this one path
    /// rather than by a second untagged grammar, and an unrecognized tag is
    /// refused by name instead of being consumed as extent bytes.
    fn sourced_extents(&mut self) -> Result<Vec<SourcedExtent>, ArtifactCodecError> {
        let rank = self.count(MAX_INTERFACE_SHAPE_RANK, CodecLimitKind::ShapeRank)?;
        let mut extents = Vec::with_capacity(rank);
        for _ in 0..rank {
            extents.push(match self.u8()? {
                SOURCED_EXTENT_LITERAL => SourcedExtent::Static(Extent::new(self.u64()?)),
                SOURCED_EXTENT_SYMBOL => {
                    let scope = SymbolScope::new(self.slice()?)
                        .map_err(|cause| ArtifactCodecError::InvalidInterfaceSymbol { cause })?;
                    let name = self.text()?;
                    SourcedExtent::Symbol(
                        ShapeSymbol::new(scope, name).map_err(|cause| {
                            ArtifactCodecError::InvalidInterfaceSymbol { cause }
                        })?,
                    )
                }
                tag => {
                    return Err(ArtifactCodecError::UnknownTag {
                        subject: TagSubject::InterfaceExtentSource,
                        tag,
                    });
                }
            });
        }
        Ok(extents)
    }

    fn interface_components(&mut self) -> Result<Vec<InterfaceComponentData>, ArtifactCodecError> {
        self.vec(
            MAX_ENTRY_BINDINGS,
            CodecLimitKind::EntryBindings,
            |cursor| {
                let role = cursor.component_role()?;
                let shape = cursor.shape()?;
                let resolved_type = match cursor.u8()? {
                    0 => None,
                    1 => Some(cursor.slice()?.to_vec()),
                    tag => {
                        return Err(ArtifactCodecError::UnknownTag {
                            subject: TagSubject::ComponentPresence,
                            tag,
                        });
                    }
                };
                Ok(InterfaceComponentData {
                    role,
                    shape,
                    resolved_type,
                    storage_scalar: cursor.storage_scalar()?,
                    encoding: cursor.storage_encoding()?,
                    access_type: cursor.element_type()?,
                })
            },
        )
    }

    fn component_role(&mut self) -> Result<Option<EncodedComponentRole>, ArtifactCodecError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(EncodedComponentRole::new(self.u32()?))),
            tag => Err(ArtifactCodecError::UnknownTag {
                subject: TagSubject::ComponentPresence,
                tag,
            }),
        }
    }

    fn storage_encoding(&mut self) -> Result<StorageEncoding, ArtifactCodecError> {
        match self.u8()? {
            0x01 => Ok(StorageEncoding::Unpacked),
            0x02 => {
                let bits = self.u8()?;
                let order = match self.u8()? {
                    0x01 => PackedBitOrder::LeastSignificantElementFirst,
                    0x02 => PackedBitOrder::MostSignificantElementFirst,
                    tag => {
                        return Err(ArtifactCodecError::UnknownTag {
                            subject: TagSubject::PackedBitOrder,
                            tag,
                        });
                    }
                };
                let tail = match self.u8()? {
                    0x01 => PackedTailRule::Zero,
                    tag => {
                        return Err(ArtifactCodecError::UnknownTag {
                            subject: TagSubject::PackedTailRule,
                            tag,
                        });
                    }
                };
                BitPackedEncoding::new(bits, order, tail)
                    .map(StorageEncoding::BitPacked)
                    .ok_or(ArtifactCodecError::UnknownTag {
                        subject: TagSubject::StorageEncoding,
                        tag: bits,
                    })
            }
            tag => Err(ArtifactCodecError::UnknownTag {
                subject: TagSubject::StorageEncoding,
                tag,
            }),
        }
    }

    /// Reads one entry's synchronization realization, or its recorded absence.
    ///
    /// The presence byte is a governed tag with exactly two admitted values, so
    /// a third is `UnknownTag` rather than a silent absence: an artifact whose
    /// presence byte a forger flipped to something unrecognized must not read as
    /// "requires no synchronization", which is the reading that would let a
    /// synchronized program dispatch against a target that never attested to
    /// ordering it.
    fn synchronization(&mut self) -> Result<Option<SynchronizationSubject>, ArtifactCodecError> {
        let tag = self.u8()?;
        match tag {
            0x00 => Ok(None),
            0x01 => Ok(Some(SynchronizationSubject {
                kind: self.synchronization_kind()?,
                execution_scope: self.synchronization_scope()?,
                visibility_scope: self.synchronization_scope()?,
                fenced_spaces: FencedSpaces {
                    workgroup: self.boolean()?,
                    device: self.boolean()?,
                },
                ordering: self.memory_ordering()?,
            })),
            tag => Err(ArtifactCodecError::UnknownTag {
                subject: TagSubject::SynchronizationPresence,
                tag,
            }),
        }
    }

    /// Reads the conditional subgroup tail at the resource/numerical boundary.
    ///
    /// A zero belongs to the following bounded `u64` text length and is peeked
    /// without consumption. The nonzero block tag instead claims six fixed
    /// bytes, all rebuilt through the schedule vocabulary's checked public
    /// constructors. No unknown or invalid byte becomes an absent requirement.
    pub(super) fn subgroup_requirement(
        &mut self,
    ) -> Result<Option<SubgroupRealizationSubject>, ArtifactCodecError> {
        match self.peek_u8() {
            None | Some(0x00) => return Ok(None),
            Some(SUBGROUP_REQUIREMENT_BLOCK_TAG) => {
                let _ = self.u8()?;
            }
            Some(tag) => {
                return Err(ArtifactCodecError::UnknownTag {
                    subject: TagSubject::SubgroupPresence,
                    tag,
                });
            }
        }

        let width = SubgroupWidth::new(self.u32()?)
            .map_err(|cause| ArtifactCodecError::InvalidSubgroupRealization { cause })?;
        let arithmetic_tag = self.u8()?;
        let arithmetic =
            ArithmeticType::from_tag(arithmetic_tag).ok_or(ArtifactCodecError::UnknownTag {
                subject: TagSubject::SubgroupArithmetic,
                tag: arithmetic_tag,
            })?;
        let transfer_tag = self.u8()?;
        let transfer: SubgroupTransfer =
            subgroup_transfer_from_tag(transfer_tag).ok_or(ArtifactCodecError::UnknownTag {
                subject: TagSubject::SubgroupTransfer,
                tag: transfer_tag,
            })?;
        let subject = SubgroupRealizationSubject::new(width, arithmetic, transfer)
            .map_err(|cause| ArtifactCodecError::InvalidSubgroupRealization { cause })?;

        match self.peek_u8() {
            None | Some(0x00) => Ok(Some(subject)),
            Some(SUBGROUP_REQUIREMENT_BLOCK_TAG) => {
                Err(ArtifactCodecError::DuplicateSubgroupRequirement)
            }
            Some(tag) => Err(ArtifactCodecError::UnknownTag {
                subject: TagSubject::SubgroupPresence,
                tag,
            }),
        }
    }

    fn boolean(&mut self) -> Result<bool, ArtifactCodecError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            tag => Err(ArtifactCodecError::UnknownTag {
                subject: TagSubject::Boolean,
                tag,
            }),
        }
    }

    fn provider(&mut self) -> Result<ProviderIdentity, ArtifactCodecError> {
        let namespace = self.text()?;
        let name = self.text()?;
        let revision = self.u32()?;
        ProviderIdentity::from_owned(namespace, name, revision)
            .map_err(|cause| ArtifactCodecError::InvalidProviderIdentity { cause })
    }

    fn expression_ref(&mut self, expressions: usize) -> Result<u32, ArtifactCodecError> {
        let node = self.u32()?;
        if position(node) >= expressions {
            return Err(ArtifactCodecError::MissingReference {
                subject: ReferenceSubject::Expression,
                index: u64::from(node),
            });
        }
        Ok(node)
    }

    pub(super) fn vec<T>(
        &mut self,
        limit: usize,
        resource: CodecLimitKind,
        mut parse: impl FnMut(&mut Self) -> Result<T, ArtifactCodecError>,
    ) -> Result<Vec<T>, ArtifactCodecError> {
        let count = self.count(limit, resource)?;
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            items.push(parse(self)?);
        }
        Ok(items)
    }

    fn expression_node(&mut self, index: u32) -> Result<ExprNode, ArtifactCodecError> {
        let tag = self.u8()?;
        match tag {
            0x01 => Ok(ExprNode::Root(self.root()?)),
            0x02 => {
                let op_tag = self.u8()?;
                Ok(ExprNode::Unary {
                    op: AbiUnaryOp::from_tag(op_tag).ok_or(ArtifactCodecError::UnknownTag {
                        subject: TagSubject::UnaryOperation,
                        tag: op_tag,
                    })?,
                    operand: self.operand(index)?,
                })
            }
            0x03 => {
                let op_tag = self.u8()?;
                Ok(ExprNode::Binary {
                    op: AbiBinaryOp::from_tag(op_tag).ok_or(ArtifactCodecError::UnknownTag {
                        subject: TagSubject::BinaryOperation,
                        tag: op_tag,
                    })?,
                    left: self.operand(index)?,
                    right: self.operand(index)?,
                })
            }
            0x04 => Ok(ExprNode::Select {
                condition: self.operand(index)?,
                if_true: self.operand(index)?,
                if_false: self.operand(index)?,
            }),
            tag => Err(ArtifactCodecError::UnknownTag {
                subject: TagSubject::ExpressionNode,
                tag,
            }),
        }
    }

    /// Reads one operand and proves it strictly precedes the node that uses it.
    fn operand(&mut self, node: u32) -> Result<u32, ArtifactCodecError> {
        let operand = self.u32()?;
        if operand >= node {
            return Err(ArtifactCodecError::ExpressionOperandOrder {
                node: u64::from(node),
                operand: u64::from(operand),
            });
        }
        Ok(operand)
    }

    /// Reads what one binding slot addresses.
    ///
    /// The output-key list is read as a canonically ordered set with its own
    /// governed budget: it is bounded by the interface it names, and a repeat
    /// would make one buffer answer to one name twice while the artifact's
    /// identity folded the repetition as meaning.
    fn binding_target(&mut self) -> Result<BindingTargetData, ArtifactCodecError> {
        let tag = self.u8()?;
        match tag {
            BINDING_TARGET_PROGRAM_INPUT => Ok(BindingTargetData::ProgramInput(
                InputKey::from_owned(self.text()?)
                    .map_err(|cause| ArtifactCodecError::InvalidInterfaceKey { cause })?,
            )),
            BINDING_TARGET_PROGRAM_OUTPUT => {
                let keys = self.vec(
                    MAX_INTERFACE_ENTRIES,
                    CodecLimitKind::BindingTargetKeys,
                    |cursor| {
                        OutputKey::from_owned(cursor.text()?)
                            .map_err(|cause| ArtifactCodecError::InvalidInterfaceKey { cause })
                    },
                )?;
                if keys.is_empty() {
                    return Err(ArtifactCodecError::EmptyBindingTarget);
                }
                require_sorted_and_distinct(&keys, OrderedSubject::BindingTargetKey)?;
                Ok(BindingTargetData::ProgramOutput(keys))
            }
            BINDING_TARGET_INTERNAL => Ok(BindingTargetData::Internal),
            tag => Err(ArtifactCodecError::UnknownTag {
                subject: TagSubject::BindingTarget,
                tag,
            }),
        }
    }

    fn root(&mut self) -> Result<AbiRoot, ArtifactCodecError> {
        let tag = self.u8()?;
        match tag {
            0x01 => Ok(AbiRoot::UnsignedLiteral(self.u64()?)),
            0x02 => Ok(AbiRoot::BooleanLiteral(self.boolean()?)),
            0x03 => Ok(AbiRoot::InputExtent {
                key: InputKey::from_owned(self.text()?)
                    .map_err(|cause| ArtifactCodecError::InvalidInterfaceKey { cause })?,
                axis: tiler_ir::shape::Axis::new(self.u32()?),
            }),
            0x04 => Ok(AbiRoot::TargetProperty {
                key: TargetPropertyKey::from_owned(self.text()?).map_err(|cause| {
                    ArtifactCodecError::InvalidGovernedKey {
                        cause: cause.into(),
                    }
                })?,
                phase: self.phase()?,
            }),
            tag => Err(ArtifactCodecError::UnknownTag {
                subject: TagSubject::ExpressionRoot,
                tag,
            }),
        }
    }
}

/// Reads one governed tag and resolves it, or rejects it by name.
macro_rules! tag_reader {
    ($method:ident, $value:ty, $resolve:path, $subject:expr) => {
        impl Cursor<'_> {
            fn $method(&mut self) -> Result<$value, ArtifactCodecError> {
                let tag = self.u8()?;
                $resolve(tag).ok_or(ArtifactCodecError::UnknownTag {
                    subject: $subject,
                    tag,
                })
            }
        }
    };
}

tag_reader!(
    phase,
    AvailabilityPhase,
    AvailabilityPhase::from_tag,
    TagSubject::AvailabilityPhase
);
tag_reader!(
    binding_kind,
    BindingKind,
    BindingKind::from_tag,
    TagSubject::BindingKind
);
tag_reader!(
    stage_dependency_reason,
    StageDependencyReason,
    StageDependencyReason::from_tag,
    TagSubject::StageDependencyReason
);
tag_reader!(
    execution_policy,
    super::super::model::ArtifactExecutionPolicy,
    super::super::model::ArtifactExecutionPolicy::from_tag,
    TagSubject::ExecutionPolicy
);
tag_reader!(
    element_type,
    tiler_ir::kernel::KernelType,
    element_type_from_tag,
    TagSubject::ElementType
);
tag_reader!(
    storage_scalar,
    StorageScalar,
    storage_scalar_from_tag,
    TagSubject::StorageScalar
);
tag_reader!(
    address_space,
    tiler_ir::kernel::AddressSpace,
    address_space_from_tag,
    TagSubject::AddressSpace
);
tag_reader!(
    buffer_access,
    tiler_ir::kernel::BufferAccess,
    buffer_access_from_tag,
    TagSubject::BufferAccess
);
tag_reader!(
    subnormal,
    tiler_ir::schedule::SubnormalMode,
    subnormal_from_tag,
    TagSubject::SubnormalMode
);
tag_reader!(
    permission,
    tiler_ir::schedule::NumericalPermission,
    permission_from_tag,
    TagSubject::NumericalPermission
);
tag_reader!(
    approximation_envelope,
    tiler_ir::schedule::ApproximationEnvelope,
    approximation_envelope_from_tag,
    TagSubject::ApproximationEnvelope
);
tag_reader!(
    index_arithmetic,
    tiler_ir::schedule::IndexArithmetic,
    index_arithmetic_from_tag,
    TagSubject::IndexArithmetic
);
tag_reader!(
    exceptional_assumption,
    ExceptionalValueAssumption,
    exceptional_assumption_from_tag,
    TagSubject::ExceptionalValueAssumption
);
tag_reader!(
    synchronization_kind,
    tiler_ir::schedule::SynchronizationKind,
    synchronization_kind_from_tag,
    TagSubject::SynchronizationKind
);
tag_reader!(
    synchronization_scope,
    tiler_ir::schedule::SynchronizationScope,
    synchronization_scope_from_tag,
    TagSubject::SynchronizationScope
);
tag_reader!(
    memory_ordering,
    tiler_ir::schedule::MemoryOrdering,
    memory_ordering_from_tag,
    TagSubject::MemoryOrdering
);

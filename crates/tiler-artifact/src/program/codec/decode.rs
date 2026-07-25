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
//! program itself. The ticket
//! `carry-reconstructable-kernel-programs-in-the-neutral-envelope` owns closing
//! that, and until it does, a consumer that needs a `VerifiedKernelProgram`
//! must hold the one it compiled.

use tiler_ir::schedule::ResourceRequirements;
use tiler_ir::semantic::{InputKey, OutputKey, ProviderIdentity};
use tiler_ir::shape::Shape;

use super::super::expr::{
    AbiBinaryOp, AbiRoot, AbiType, AbiUnaryOp, AvailabilityPhase, ExprNode, TargetPropertyKey,
    binary_operand_type, node_type, unary_operand_type,
};
use super::super::keys::{
    BackendEntryKey, BackendKey, CapabilityKey, FeasibilityRuleSetKey, FeasibilityRuleSetRef,
    PayloadDigest, RepresentationKey, TargetProfileDescriptorDigest, TargetProfileKey,
    TargetProfileRef,
};
use super::super::model::{
    ArtifactSchema, BINDING_TARGET_INTERNAL, BINDING_TARGET_PROGRAM_INPUT,
    BINDING_TARGET_PROGRAM_OUTPUT, BackendPayloadDescriptor, BindingData, BindingKind,
    BindingTargetData, DeferredPredicateData, InterfaceEntryData, LaunchData, RoutingPolicy,
    SchemaVersion, SelectedProvider, address_space_from_tag, buffer_access_from_tag,
    element_type_from_tag, permission_from_tag, subnormal_from_tag,
};
use super::super::{
    MAX_ABI_EXPRESSIONS, MAX_ARTIFACT_PAYLOADS, MAX_ARTIFACT_VARIANTS, MAX_DEFERRED_PREDICATES,
    MAX_ENTRY_BINDINGS, MAX_LAUNCH_PRECONDITIONS, MAX_SELECTED_PROVIDERS, MAX_VARIANT_ENTRIES,
};
use super::digest::{Digest, DigestAlgorithm};
use super::encode::{
    CANONICAL_ENCODING, ENVELOPE_FORMAT, HEADER_BYTES, MAGIC, MANIFEST_DIGEST_DOMAIN,
    MANIFEST_DOMAIN, MANIFEST_SCHEMA, MAX_ENVELOPE_BYTES, MAX_MANIFEST_BYTES, encode,
    section_digest,
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
    canonical_expression_order, expression_keys, ordinal, position,
};
use super::validate::validate;

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
    let sections = read_sections(&mut cursor, &parsed.sections, header.algorithm)?;
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
    if derived.as_bytes() != parsed.identity {
        return Err(ArtifactCodecError::ArtifactIdentityMismatch);
    }
    // The manifest is fully understood, so re-encoding it must reproduce the
    // exact bytes that were read. This is the backstop that makes one artifact
    // have one byte identity: any well-formed but non-canonical spelling a
    // named check did not already catch fails here rather than being silently
    // normalized on the way in.
    if encode(&envelope)? != bytes {
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
fn read_sections(
    cursor: &mut Cursor<'_>,
    descriptors: &[SectionDescriptor],
    algorithm: DigestAlgorithm,
) -> Result<Vec<Section>, ArtifactCodecError> {
    let mut sections = Vec::with_capacity(descriptors.len());
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
        if section_digest(algorithm, &section) != descriptor.digest {
            return Err(ArtifactCodecError::SectionDigestMismatch {
                section: declared_id,
            });
        }
        sections.push(section);
    }
    Ok(sections)
}

/// The decoded manifest and the two derived values validated against it.
struct ParsedManifest {
    body: DecodedBody,
    sections: Vec<SectionDescriptor>,
    identity: Vec<u8>,
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
    pub(super) providers: Vec<SelectedProvider>,
    pub(super) payloads: Vec<BackendPayloadDescriptor>,
    /// Each payload's carried sections, aligned with `payloads`.
    pub(super) payload_content: Vec<Option<PayloadSections>>,
    pub(super) expressions: Vec<ExprNode>,
    pub(super) variants: Vec<VariantRow>,
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
    };

    let inputs = read_inputs(&mut cursor)?;
    let outputs = read_outputs(&mut cursor)?;
    let providers = read_providers(&mut cursor)?;
    let (payloads, payload_content) = read_payloads(&mut cursor)?;

    let expressions = parse_expressions(&mut cursor)?;
    let variants = parse_variants(&mut cursor, expressions.len(), payloads.len())?;

    let sections = parse_section_descriptors(&mut cursor)?;
    let identity = cursor.slice()?.to_vec();
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
        },
        sections,
        identity,
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
                shape: cursor.shape()?,
                element_type: cursor.element_type()?,
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
                shape: cursor.shape()?,
                element_type: cursor.element_type()?,
            })
        },
    )
}

/// Reads the selected capability providers and proves their canonical order.
fn read_providers(cursor: &mut Cursor<'_>) -> Result<Vec<SelectedProvider>, ArtifactCodecError> {
    let providers = cursor.vec(
        MAX_SELECTED_PROVIDERS,
        CodecLimitKind::SelectedProviders,
        |cursor| {
            Ok(SelectedProvider {
                provider: cursor.provider()?,
                capability: CapabilityKey::from_owned(cursor.text()?)
                    .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?,
                capability_revision: cursor.u32()?,
            })
        },
    )?;
    require_sorted_and_distinct(
        &providers
            .iter()
            .map(SelectedProvider::canonical_key)
            .collect::<Vec<_>>(),
        OrderedSubject::Provider,
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
        let descriptor = BackendPayloadDescriptor {
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
/// the value type its operation requires, and no two nodes share a canonical
/// content key. The last is what keeps a cross-reference by key injective.
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
    let keys = expression_keys(&nodes);
    let mut sorted: Vec<&Vec<u8>> = keys.iter().collect();
    sorted.sort_unstable();
    if sorted.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(ArtifactCodecError::DuplicateItem {
            subject: OrderedSubject::Expression,
        });
    }
    if canonical_expression_order(&nodes, &keys) != (0..ordinal(nodes.len())).collect::<Vec<_>>() {
        return Err(ArtifactCodecError::NonCanonicalOrder {
            subject: OrderedSubject::Expression,
        });
    }
    Ok(nodes)
}

/// Parses one standalone expression arena, for arena-shaped adversarial tests.
///
/// Forging a malformed arena through the envelope model is not possible: the
/// canonical content key of a node reads its operands' keys, so a self- or
/// forward-referencing operand cannot be encoded at all. Driving the parser
/// directly is the only way to prove the rejection is the named one.
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
        let deferred = cursor.vec(
            MAX_DEFERRED_PREDICATES,
            CodecLimitKind::DeferredPredicates,
            |cursor| {
                Ok(DeferredPredicateData {
                    predicate: cursor.expression_ref(expressions)?,
                    phase: cursor.phase()?,
                    authority: cursor.provider()?,
                })
            },
        )?;
        let entries = cursor.vec(MAX_VARIANT_ENTRIES, CodecLimitKind::Entries, |cursor| {
            parse_entry(cursor, expressions, payloads)
        })?;
        variants.push(VariantRow {
            program_section,
            guard,
            profile,
            feasibility_rules,
            deferred,
            entries,
        });
    }
    Ok(variants)
}

fn parse_entry(
    cursor: &mut Cursor<'_>,
    expressions: usize,
    payloads: usize,
) -> Result<EntryRow, ArtifactCodecError> {
    let stage = StageSubject::from_bytes(cursor.slice()?)?;
    let resources = ResourceRequirements {
        buffer_bindings: cursor.u32()?,
        threads_per_workgroup: cursor.u32()?,
        local_memory_bytes: cursor.u64()?,
        barriers: cursor.u32()?,
        requires_device_memory: cursor.boolean()?,
        input_subnormals: cursor.subnormal()?,
        result_subnormals: cursor.subnormal()?,
        contraction: cursor.permission()?,
        reassociation: cursor.permission()?,
    };
    let numerical = NumericalFacts {
        profile_key: cursor.text()?,
        canonical_arithmetic_nan_bits: cursor.u32()?,
        input_subnormals: cursor.subnormal()?,
        result_subnormals: cursor.subnormal()?,
        contraction: cursor.permission()?,
        reassociation: cursor.permission()?,
    };
    let bindings = cursor.vec(
        MAX_ENTRY_BINDINGS,
        CodecLimitKind::EntryBindings,
        |cursor| {
            Ok(BindingData {
                kind: cursor.binding_kind()?,
                element_type: cursor.element_type()?,
                address_space: cursor.address_space()?,
                access: cursor.buffer_access()?,
                alignment: cursor.u32()?,
                target: cursor.binding_target()?,
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
    let payload = cursor.u32()?;
    if position(payload) >= payloads {
        return Err(ArtifactCodecError::MissingReference {
            subject: ReferenceSubject::Payload,
            index: u64::from(payload),
        });
    }
    Ok(EntryRow {
        stage,
        resources,
        numerical,
        bindings,
        launch,
        payload,
        entry_key: BackendEntryKey::from_bytes(cursor.slice()?)
            .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?,
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

    fn u8(&mut self) -> Result<u8, ArtifactCodecError> {
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

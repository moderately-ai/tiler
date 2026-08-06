//! Canonical encoding of one artifact envelope into its exact bytes.
//!
//! The encoder is the authority for the wire form; [`super::decode`] is written
//! against it and every field's inverse is pinned by a round-trip test.
//!
//! Two properties are load-bearing and are worth stating separately from the
//! field list. **Every variable-length run carries a fixed-width length before
//! its content**, so no concatenation of fields is ambiguous. And **every
//! encoded enumeration is written through the one governed tag table its
//! vocabulary owns**, never through a Rust discriminant, so inserting a variant
//! cannot silently renumber a value that is already on disk.

use super::super::error::ArtifactDiagnostic;
use super::super::expr::ExprNode;
use super::super::model::{address_space_tag, buffer_access_tag};
use super::super::model::{
    element_type_tag, push_binding_target, push_component_role, push_numerical, push_resources,
    push_shape, push_storage_encoding, storage_scalar_tag,
};
use super::super::requirement::RouteRequirement;
use super::budget::check_budgets;
use super::digest::{DIGEST_BYTES, Digest, DigestAlgorithm};
use super::error::{ArtifactCodecError, CodecLimitKind, codec_limit};
use super::model::{ArtifactEnvelope, EntryRow, MAX_SECTION_BYTES, Section, ordinal};
use tiler_ir::identity::{push_len, push_slice};
use tiler_ir::program::abi::TargetPropertyRequirementRelation;

/// Fixed framing magic of the target-neutral artifact envelope.
pub(super) const MAGIC: [u8; 8] = *b"TILERART";
/// Exact byte length of the fixed framing header.
pub(super) const HEADER_BYTES: usize = 69;
/// Envelope framing format version this build writes and reads.
pub(super) const ENVELOPE_FORMAT: (u16, u16) = (1, 0);
/// Canonical byte-encoding profile version this build writes and reads.
pub(super) const CANONICAL_ENCODING: (u16, u16) = (1, 0);
/// Neutral manifest schema version this build writes and reads.
///
/// Raised to `2.0` when the section descriptor grew its purpose disposition and
/// content schema, to `3.0` when each ABI binding row replaced its program
/// role tag with the interface reference naming what the slot addresses, to
/// `4.0` when a selected provider row replaced its `u16` capability API version
/// with the `u32` capability revision, and to `5.0` when each ABI binding row
/// gained the accessible offset placing its range inside the value it binds,
/// and to `6.0` when the two numerical records gained permutation, signed-zero,
/// NaN-assumption, and infinity-assumption fields, and to `7.0` when interface
/// entries gained ordered logical components and binding rows gained component
/// roles, physical storage scalars, complete storage encodings, and kernel
/// access types, to `8.0` when the entry resource record stopped carrying the
/// invalid numeric barrier count, and to `9.0` when each deferred predicate
/// gained its exact prepared-entry subject and complete target-property query,
/// and to `10.0` when each plan variant gained the additional requirements its
/// route places on a live device.
/// All nine are deliberately **major** steps rather than the minor ones they
/// might look like: the reader admits `minor <= implemented`, so a minor bump
/// would have left it accepting an older manifest whose rows it can no longer
/// parse. A field changed *or added* inside a fixed-width record is not
/// additive — the `5.0` step inserts four bytes ahead of an existing field, so a
/// `4.0` reader would consume the offset as the extent and lose framing for
/// everything after it, the `6.0` step appends fields inside each entry before
/// its bindings, and the `8.0` step removes four bytes ahead of existing fields
/// — and the `4.0` step also moved a field's width. The `10.0` step inserts a
/// counted run inside each variant record ahead of its entries, so a `9.0`
/// reader would consume the route-requirement count as the entry count and lose
/// framing for the rest of the manifest.
///
/// Raised to `11.0` when an entry row replaced its single fixed-width payload
/// reference with a counted run of them, one per delivery position. Major for
/// the plainest of the reasons above: a `10.0` reader would consume the count as
/// the payload reference and then read the first reference as the entry key's
/// length prefix, losing framing for the rest of the manifest. It would also be
/// wrong even if the framing survived — a reader that cannot resolve a delivery
/// position has no basis for choosing among several objects — which is what the
/// separate required-feature key states for a reader at this schema.
///
/// Raised to `12.0` when the fixed executable-entry resource record gained the
/// synchronization realization the entry's schedule requires. Major for the same
/// framing reason the `8.0` step was, in the other direction: an `11.0` reader
/// would consume the presence byte as the input-subnormal tag and lose framing
/// for every field after it. There is deliberately **no** required-feature key
/// beside it, and the asymmetry with the route-requirement family is the reason:
/// a variant with zero route rows stays readable by a reader that predates them,
/// so its feature key marks the artifacts that are not — whereas every artifact
/// at this schema writes the presence byte, so no `11.0` reader can read any of
/// them and a key would mark nothing.
///
/// Raised to `13.0` when the manifest gained the required delivered-realization
/// record, written as one framed run after the variant table. Major, and the
/// asymmetry with the route-requirement family decides it the same way the
/// `12.0` step was decided: **every** artifact at this schema writes the run, so
/// a `12.0` reader would consume its length prefix as the section-descriptor
/// count and lose framing for the rest of the manifest. No reader that predates
/// the family can read any artifact carrying one, so a required-feature key
/// beside it would mark nothing.
pub(super) const MANIFEST_SCHEMA: (u16, u16) = (13, 0);

/// Versioned domain tag opening the canonical manifest bytes.
pub(super) const MANIFEST_DOMAIN: &[u8] = b"tiler.artifact-envelope.manifest.v1\0";
/// Domain separator of the manifest digest carried in the framing header.
pub(crate) const MANIFEST_DIGEST_DOMAIN: &[u8] = b"tiler.artifact-envelope.manifest-digest.v1\0";
/// Domain separator of one section's exact-content digest.
///
/// The pre-image is the separator, the section's purpose tag, its content
/// schema, and then its exact bytes. Binding the purpose is what lets a section
/// digest serve as a *standalone* content address: without it, two sections
/// with equal bytes under different purposes would share one address.
pub(crate) const SECTION_DIGEST_DOMAIN: &[u8] = b"tiler.artifact-envelope.section-digest.v1\0";
/// Domain separator of the external digest over a complete encoded envelope.
pub(crate) const ENVELOPE_DIGEST_DOMAIN: &[u8] = b"tiler.artifact-envelope.envelope-digest.v1\0";

/// Maximum bytes of one complete encoded envelope.
pub(super) const MAX_ENVELOPE_BYTES: usize = 256 * 1024 * 1024;
/// Maximum bytes of the canonical manifest.
pub(super) const MAX_MANIFEST_BYTES: usize = 64 * 1024 * 1024;

/// Encodes one artifact envelope into its exact canonical bytes.
///
/// # Errors
///
/// Returns [`ArtifactCodecError::IdentityDerivation`] when the artifact model
/// refuses to derive an identity for the envelope's content, or
/// [`ArtifactCodecError::Limit`] when the envelope exceeds a governed encoder
/// budget. The budgets are exactly the decoder's, so an envelope this function
/// accepts always survives a round trip rather than producing bytes no reader
/// admits.
pub(crate) fn encode(envelope: &ArtifactEnvelope) -> Result<Vec<u8>, ArtifactCodecError> {
    let identity = envelope
        .canonical_identity()
        .map_err(|cause| ArtifactCodecError::IdentityDerivation { cause })?;
    let digests: Vec<Digest> = envelope
        .sections()
        .iter()
        .map(|section| section_digest(DigestAlgorithm::GOVERNED, section))
        .collect();
    encode_with_identity(envelope, &identity, &digests)
}

/// Encodes an envelope whose identity and section digests the caller has derived.
///
/// **Both are parameters because deriving them is not cheap and the one caller
/// that needs this already holds them.** `decode` derives the identity to
/// compare against the manifest's and digests every section to compare against
/// its descriptor, and then re-derives the canonical encoding as its canonicity
/// backstop — which derived the very same identity and the very same digests a
/// second time, from the same values, on every decode and therefore on every
/// cache hit. Section content is 31% of the bytes a decode drives through
/// SHA-256, and the governed digest is where a decode spends most of its time.
///
/// **Reusing the section digests does not weaken the backstop, and reusing the
/// *manifest* digest would.** A section digest is a pure function of the
/// `Section` this envelope holds, so a caller passing the one it derived from
/// that same value supplies no information the encoder did not already have.
/// The manifest digest is different in kind: it covers the manifest bytes this
/// function is *rebuilding*, and whether those reproduce the ones on the wire is
/// the very question the backstop asks — so it is derived here, every time.
///
/// The backstop itself does not call this. It calls
/// [`matches_canonical_encoding`], which runs the same derivation against the
/// bytes it is checking instead of into a buffer nobody keeps.
///
/// # Panics
///
/// Panics when `section_digests` does not have exactly one entry per section.
/// Zipping the two instead would silently encode the shorter of them, which is
/// the framing desync [`encode_provenance_tables`] refuses for the same reason.
pub(crate) fn encode_with_identity(
    envelope: &ArtifactEnvelope,
    identity: &crate::program::CanonicalArtifactProgramIdentity,
    section_digests: &[Digest],
) -> Result<Vec<u8>, ArtifactCodecError> {
    let EncodedHead {
        header,
        manifest,
        total,
    } = encode_head(envelope, identity, section_digests)?;

    let mut bytes = Vec::with_capacity(total);
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(&manifest);
    // Released before the sections are appended. Holding it would make the
    // encoder's peak one whole envelope *plus* one manifest, and its content is
    // already copied.
    drop(manifest);
    for (position, section) in envelope.sections().iter().enumerate() {
        push_section_framing(&mut bytes, position, section);
        bytes.extend_from_slice(&section.bytes);
    }
    debug_assert_eq!(
        bytes.len(),
        total,
        "the derived total length is the encoding's length",
    );
    Ok(bytes)
}

/// Proves `bytes` is the exact canonical encoding of `envelope`.
///
/// The same derivation [`encode_with_identity`] performs, compared against the
/// bytes in hand run by run rather than accumulated into a second buffer. The
/// two are equivalent by construction — both drive [`encode_head`] and
/// [`push_section_framing`], and the section stream is the same slices in the
/// same order — and the difference is what a decode peaks at: comparing costs
/// one manifest, where accumulating cost one whole additional envelope, every
/// section byte of it, on every decode.
///
/// # Errors
///
/// Returns the typed [`ArtifactCodecError`] when the envelope cannot be encoded
/// at all — an exhausted budget or an unencodable table. A well-formed envelope
/// whose bytes simply are not canonical returns `Ok(false)`; the caller names
/// that rejection, because it is the caller's check rather than this one's.
///
/// # Panics
///
/// Panics under the same condition [`encode_with_identity`] does.
pub(super) fn matches_canonical_encoding(
    envelope: &ArtifactEnvelope,
    identity: &crate::program::CanonicalArtifactProgramIdentity,
    section_digests: &[Digest],
    bytes: &[u8],
) -> Result<bool, ArtifactCodecError> {
    let EncodedHead {
        header,
        manifest,
        total,
    } = encode_head(envelope, identity, section_digests)?;
    if bytes.len() != total {
        return Ok(false);
    }
    let Some(rest) = bytes.strip_prefix(header.as_slice()) else {
        return Ok(false);
    };
    let Some(mut rest) = rest.strip_prefix(manifest.as_slice()) else {
        return Ok(false);
    };
    drop(manifest);

    let mut framing: Vec<u8> = Vec::with_capacity(SECTION_FRAMING_BYTES);
    for (position, section) in envelope.sections().iter().enumerate() {
        framing.clear();
        push_section_framing(&mut framing, position, section);
        let Some(after_framing) = rest.strip_prefix(framing.as_slice()) else {
            return Ok(false);
        };
        let Some(after_content) = after_framing.strip_prefix(section.bytes.as_slice()) else {
            return Ok(false);
        };
        rest = after_content;
    }
    // Implied by the length agreement above, and checked rather than argued:
    // a section stream that consumed less than the whole remainder would mean
    // the derived total and the derived runs disagree.
    Ok(rest.is_empty())
}

/// The fixed framing header, the canonical manifest, and the exact total length.
///
/// The two runs are kept apart rather than concatenated because the comparing
/// caller never needs them adjacent, and joining them would cost it a copy of
/// the manifest it is about to compare against.
struct EncodedHead {
    /// The fixed-width framing header, exactly [`HEADER_BYTES`] long.
    header: Vec<u8>,
    /// The canonical manifest the header digests.
    manifest: Vec<u8>,
    /// Exact byte length of the complete encoding.
    total: usize,
}

/// Exact width of the framing that precedes one section's bytes.
const SECTION_FRAMING_BYTES: usize = size_of::<u32>() + size_of::<u64>();

/// Writes the framing that introduces one section's exact bytes.
///
/// One function rather than a written form and a compared form, so the two
/// callers cannot drift into two spellings of the same framing.
fn push_section_framing(bytes: &mut Vec<u8>, position: usize, section: &Section) {
    let before = bytes.len();
    bytes.extend_from_slice(&ordinal(position).to_be_bytes());
    push_len(bytes, section.bytes.len());
    debug_assert_eq!(
        bytes.len() - before,
        SECTION_FRAMING_BYTES,
        "the section framing is fixed width",
    );
}

/// Derives the framing header, the canonical manifest, and the total length.
///
/// The total is computed from the manifest and the section table before any of
/// it is written, which is what lets the header carry it as a derived field
/// rather than a value patched in afterwards — and lets an envelope that cannot
/// fit the governed bound be refused before it is assembled.
fn encode_head(
    envelope: &ArtifactEnvelope,
    identity: &crate::program::CanonicalArtifactProgramIdentity,
    section_digests: &[Digest],
) -> Result<EncodedHead, ArtifactCodecError> {
    assert_eq!(
        section_digests.len(),
        envelope.sections().len(),
        "a section digest per section",
    );
    check_budgets(envelope)?;
    let algorithm = DigestAlgorithm::GOVERNED;
    let manifest = encode_manifest(envelope, identity, section_digests)?;
    codec_limit(
        manifest.len(),
        MAX_MANIFEST_BYTES,
        CodecLimitKind::ManifestBytes,
    )?;

    // Saturating rather than checked, because saturation is already past the
    // governed bound the next line rejects against: a sum that could not be
    // represented is refused as the exhausted envelope budget it is.
    let mut total = HEADER_BYTES.saturating_add(manifest.len());
    for section in envelope.sections() {
        total = total.saturating_add(SECTION_FRAMING_BYTES.saturating_add(section.bytes.len()));
    }
    codec_limit(total, MAX_ENVELOPE_BYTES, CodecLimitKind::EnvelopeBytes)?;

    let mut header = Vec::with_capacity(HEADER_BYTES);
    header.extend_from_slice(&MAGIC);
    header.extend_from_slice(&ENVELOPE_FORMAT.0.to_be_bytes());
    header.extend_from_slice(&ENVELOPE_FORMAT.1.to_be_bytes());
    header.extend_from_slice(&CANONICAL_ENCODING.0.to_be_bytes());
    header.extend_from_slice(&CANONICAL_ENCODING.1.to_be_bytes());
    header.push(algorithm.tag());
    header.extend_from_slice(
        &u64::try_from(total)
            .expect("supported usize fits u64")
            .to_be_bytes(),
    );
    push_len(&mut header, manifest.len());
    // Four bytes rather than the eight framed on the line above, and not
    // `tiler_ir::identity` framing at all: a fixed-width header field that
    // `decode.rs` reads back as `cursor.u32()`, sized by the `u32` ordinal space
    // the envelope's section and expression tables are indexed in. Widening it
    // would change the artifact ABI rather than remove a duplicate.
    header.extend_from_slice(&ordinal(envelope.sections().len()).to_be_bytes());
    header.extend_from_slice(
        algorithm
            .digest(MANIFEST_DIGEST_DOMAIN, &manifest)
            .as_bytes(),
    );
    debug_assert_eq!(
        header.len(),
        HEADER_BYTES,
        "the framing header is fixed width"
    );
    Ok(EncodedHead {
        header,
        manifest,
        total,
    })
}

/// Derives the external digest over one complete encoded envelope.
///
/// The value is computed rather than stored: an in-band field covering the
/// bytes that contain it would be a recursive definition, which
/// `docs/artifact-abi.md` forbids outright.
pub(crate) fn envelope_digest(bytes: &[u8]) -> [u8; DIGEST_BYTES] {
    *DigestAlgorithm::GOVERNED
        .digest(ENVELOPE_DIGEST_DOMAIN, bytes)
        .as_bytes()
}

/// Encodes the canonical manifest bytes the framing header digests.
fn encode_manifest(
    envelope: &ArtifactEnvelope,
    identity: &crate::program::CanonicalArtifactProgramIdentity,
    section_digests: &[Digest],
) -> Result<Vec<u8>, ArtifactCodecError> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(MANIFEST_DOMAIN);
    bytes.extend_from_slice(&MANIFEST_SCHEMA.0.to_be_bytes());
    bytes.extend_from_slice(&MANIFEST_SCHEMA.1.to_be_bytes());
    let schema = envelope.schema();
    for component in [
        schema.program(),
        schema.abi_expression(),
        schema.guard_and_routing(),
        schema.target_requirement(),
    ] {
        bytes.extend_from_slice(&component.major().to_be_bytes());
        bytes.extend_from_slice(&component.minor().to_be_bytes());
    }
    bytes.push(envelope.routing_policy().tag());

    push_len(&mut bytes, envelope.features().len());
    for feature in envelope.features() {
        push_slice(&mut bytes, feature.as_bytes());
    }

    let semantic = envelope.semantic();
    push_slice(&mut bytes, semantic.graph.as_bytes());
    push_slice(&mut bytes, semantic.reached_definitions.as_bytes());
    push_slice(&mut bytes, semantic.admission_provenance.as_bytes());

    push_len(&mut bytes, envelope.inputs().len());
    for input in envelope.inputs() {
        push_slice(&mut bytes, input.key.as_str().as_bytes());
        push_shape(&mut bytes, &input.shape);
        encode_interface_components(&mut bytes, input);
    }
    push_len(&mut bytes, envelope.outputs().len());
    for output in envelope.outputs() {
        push_slice(&mut bytes, output.key.as_str().as_bytes());
        push_shape(&mut bytes, &output.shape);
        encode_interface_components(&mut bytes, output);
    }

    encode_provenance_tables(&mut bytes, envelope);
    encode_expressions(&mut bytes, envelope);
    encode_variants(&mut bytes, envelope);
    // One framed run of the record's own domain-separated canonical bytes,
    // written after the variants whose entries its bindings name. The manifest
    // does not restate the record's layout: `super::super::realization::codec`
    // owns both directions of it, so a reader that frames this run correctly and
    // then rejects its content rejects for the record's own typed reason.
    push_slice(&mut bytes, &envelope.realization().canonical_bytes());
    encode_section_descriptors(&mut bytes, envelope, section_digests)?;

    push_slice(&mut bytes, identity.as_bytes());
    Ok(bytes)
}

fn encode_interface_components<K>(
    bytes: &mut Vec<u8>,
    entry: &super::super::model::InterfaceEntryData<K>,
) {
    push_slice(bytes, &entry.logical_type);
    push_len(bytes, entry.components.len());
    for component in &entry.components {
        push_component_role(bytes, component.role);
        push_shape(bytes, &component.shape);
        match &component.resolved_type {
            None => bytes.push(0),
            Some(value_type) => {
                bytes.push(1);
                push_slice(bytes, value_type);
            }
        }
        bytes.push(storage_scalar_tag(component.storage_scalar));
        push_storage_encoding(bytes, component.encoding);
        bytes.push(element_type_tag(component.access_type));
    }
}

/// Derives one section's content digest over its purpose, schema, and bytes.
///
/// The qualifiers are fixed width and precede the variable-length content, so
/// the pre-image is unambiguous without a length prefix between them.
pub(super) fn section_digest(algorithm: DigestAlgorithm, section: &Section) -> Digest {
    let schema = section.kind.schema();
    algorithm.digest_parts(&[
        SECTION_DIGEST_DOMAIN,
        &[section.kind.tag()],
        &schema.major().to_be_bytes(),
        &schema.minor().to_be_bytes(),
        &section.bytes,
    ])
}

/// Encodes the selected providers and backend payload descriptors.
fn encode_provenance_tables(bytes: &mut Vec<u8>, envelope: &ArtifactEnvelope) {
    push_len(bytes, envelope.providers().len());
    for provider in envelope.providers() {
        push_slice(bytes, provider.provider.namespace().as_bytes());
        push_slice(bytes, provider.provider.name().as_bytes());
        bytes.extend_from_slice(&provider.provider.revision().to_be_bytes());
        push_slice(bytes, provider.capability.as_str().as_bytes());
        bytes.extend_from_slice(&provider.capability_revision.to_be_bytes());
    }
    push_len(bytes, envelope.payloads().len());
    // Indexed rather than zipped: a zip would silently stop at the shorter of
    // the two vectors while the count above already said how many rows follow,
    // so a descriptor table and a content table that disagreed in length would
    // produce a manifest whose declared count outran its rows. Reading each
    // content slot independently keeps the row count and the declared count the
    // same number by construction, and a payload with no content slot encodes
    // as the descriptor-only form the model already admits — which the unused
    // payload and unreferenced section obligations then decide on their own
    // terms rather than being pre-empted by a framing desync.
    for (position, payload) in envelope.payloads().iter().enumerate() {
        let content = envelope.payload_content().get(position).copied().flatten();
        push_slice(bytes, payload.backend.as_str().as_bytes());
        push_slice(bytes, payload.representation.as_str().as_bytes());
        bytes.extend_from_slice(&payload.payload_schema.major().to_be_bytes());
        bytes.extend_from_slice(&payload.payload_schema.minor().to_be_bytes());
        push_slice(bytes, payload.digest.as_bytes());
        push_slice(bytes, payload.compatibility.key.as_str().as_bytes());
        push_slice(bytes, payload.compatibility.descriptor.as_bytes());
        bytes.push(payload.execution_policy.tag());
        // A carried payload names its two sections here rather than in the
        // section table, so a descriptor and the object it names cannot be
        // separated by a table edit that leaves both individually well formed.
        match content {
            Some(sections) => {
                bytes.push(0x01);
                bytes.extend_from_slice(&sections.metadata.to_be_bytes());
                bytes.extend_from_slice(&sections.code.to_be_bytes());
            }
            None => bytes.push(0x00),
        }
    }
}

/// Encodes the shared ABI expression arena in canonical order.
fn encode_expressions(bytes: &mut Vec<u8>, envelope: &ArtifactEnvelope) {
    push_len(bytes, envelope.expressions().len());
    for node in envelope.expressions() {
        encode_node(bytes, node);
    }
}

/// Encodes the plan variants in routing priority order.
fn encode_variants(bytes: &mut Vec<u8>, envelope: &ArtifactEnvelope) {
    push_len(bytes, envelope.variants().len());
    for variant in envelope.variants() {
        bytes.extend_from_slice(&variant.program_section.to_be_bytes());
        bytes.extend_from_slice(&variant.guard.to_be_bytes());
        push_slice(bytes, variant.profile.key.as_str().as_bytes());
        push_slice(bytes, variant.profile.descriptor.as_bytes());
        push_slice(bytes, variant.feasibility_rules.key.as_str().as_bytes());
        bytes.extend_from_slice(&variant.feasibility_rules.revision.to_be_bytes());
        push_len(bytes, variant.deferred.len());
        for predicate in &variant.deferred {
            bytes.extend_from_slice(&predicate.predicate.to_be_bytes());
            bytes.extend_from_slice(&predicate.entry.to_be_bytes());
            let requirement = &predicate.requirement;
            push_slice(bytes, requirement.query().key().as_str().as_bytes());
            bytes.push(requirement.query().available_at().tag());
            push_slice(bytes, requirement.query().provider().namespace().as_bytes());
            push_slice(bytes, requirement.query().provider().name().as_bytes());
            bytes.extend_from_slice(&requirement.query().provider().revision().to_be_bytes());
            bytes.extend_from_slice(&requirement.required().to_be_bytes());
            bytes.push(match requirement.relation() {
                TargetPropertyRequirementRelation::ObservedAtLeastRequired => 0x01,
                TargetPropertyRequirementRelation::ObservedEqualsRequired => 0x02,
                TargetPropertyRequirementRelation::RequiredImpliesObserved => 0x03,
            });
        }
        encode_route_requirements(bytes, &variant.route_requirements);
        push_len(bytes, variant.entries.len());
        for entry in &variant.entries {
            encode_entry(bytes, entry);
        }
        push_len(bytes, variant.execution_order.len());
        for entry in &variant.execution_order {
            bytes.extend_from_slice(&entry.to_be_bytes());
        }
        push_len(bytes, variant.dependencies.len());
        for edge in &variant.dependencies {
            bytes.extend_from_slice(&edge.predecessor.to_be_bytes());
            bytes.extend_from_slice(&edge.successor.to_be_bytes());
            bytes.push(edge.reason.tag());
        }
    }
}

/// Encodes one variant's live-device route requirements in canonical order.
///
/// The kind tag leads each row so a reader frames the rest of it before reading
/// any field, which is what lets an unknown kind be refused by name rather than
/// mis-framed into the row after it.
fn encode_route_requirements(bytes: &mut Vec<u8>, requirements: &[RouteRequirement]) {
    push_len(bytes, requirements.len());
    for requirement in requirements {
        bytes.push(requirement.tag());
        match requirement {
            RouteRequirement::Resource(resource) => {
                bytes.push(resource.dimension().tag());
                bytes.extend_from_slice(&resource.required().to_be_bytes());
            }
            RouteRequirement::BackendFeature(feature) => {
                push_slice(bytes, feature.owner().as_str().as_bytes());
                push_slice(bytes, feature.key().as_str().as_bytes());
                bytes.extend_from_slice(&feature.version().to_be_bytes());
                push_slice(bytes, feature.payload());
            }
        }
    }
}

/// Encodes one executable entry.
fn encode_entry(bytes: &mut Vec<u8>, entry: &EntryRow) {
    push_slice(bytes, entry.stage.as_bytes());
    push_resources(bytes, entry.resources);
    push_numerical(bytes, &entry.numerical);
    push_len(bytes, entry.bindings.len());
    for binding in &entry.bindings {
        bytes.push(binding.kind.tag());
        bytes.push(storage_scalar_tag(binding.storage_scalar));
        bytes.push(element_type_tag(binding.access_type));
        push_component_role(bytes, binding.component_role);
        push_storage_encoding(bytes, binding.encoding);
        bytes.push(address_space_tag(binding.address_space));
        bytes.push(buffer_access_tag(binding.access));
        bytes.extend_from_slice(&binding.alignment.to_be_bytes());
        push_binding_target(bytes, &binding.target);
        bytes.extend_from_slice(&binding.accessible_offset.to_be_bytes());
        bytes.extend_from_slice(&binding.accessible_bytes.to_be_bytes());
    }
    bytes.extend_from_slice(&entry.launch.grid_threads.to_be_bytes());
    bytes.extend_from_slice(&entry.launch.threads_per_workgroup.to_be_bytes());
    bytes.push(u8::from(entry.launch.zero_work_skips_dispatch));
    push_len(bytes, entry.launch.preconditions.len());
    for precondition in &entry.launch.preconditions {
        bytes.extend_from_slice(&precondition.to_be_bytes());
    }
    // A counted run, and the count is what a reader frames the rest of the entry
    // around: the row that preceded it was one fixed-width payload reference, so
    // a reader of the earlier schema would consume this count as that reference
    // and lose framing for every row after it. That is why the schema step below
    // is major.
    push_len(bytes, entry.payloads.len());
    for payload in &entry.payloads {
        bytes.extend_from_slice(&payload.to_be_bytes());
    }
    push_slice(bytes, entry.entry_key.as_bytes());
}

/// Encodes one section descriptor per framed section.
///
/// A descriptor is never stored beside the bytes it describes; every field is
/// derived from the section's position and exact content, so the two cannot
/// disagree. The content digest is the one field the caller supplies, having
/// derived it from these same sections — see [`encode_with_identity`] for why
/// that is a memoized derivation rather than a claim this function trusts.
///
/// This is the *descriptor* table inside the manifest, not the framing that
/// introduces a section's bytes in the stream; that is
/// [`push_section_framing`].
fn encode_section_descriptors(
    bytes: &mut Vec<u8>,
    envelope: &ArtifactEnvelope,
    section_digests: &[Digest],
) -> Result<(), ArtifactCodecError> {
    push_len(bytes, envelope.sections().len());
    for (position, section) in envelope.sections().iter().enumerate() {
        codec_limit(
            section.bytes.len(),
            MAX_SECTION_BYTES,
            CodecLimitKind::SectionBytes,
        )?;
        bytes.extend_from_slice(&ordinal(position).to_be_bytes());
        bytes.push(section.kind.tag());
        bytes.push(section.kind.disposition().tag());
        let schema = section.kind.schema();
        bytes.extend_from_slice(&schema.major().to_be_bytes());
        bytes.extend_from_slice(&schema.minor().to_be_bytes());
        push_len(bytes, section.bytes.len());
        bytes.extend_from_slice(section_digests[position].as_bytes());
    }
    Ok(())
}

/// Encodes one ABI expression arena node.
///
/// The match is exhaustive with no wildcard arm, so a widened node vocabulary
/// stops the build here instead of writing bytes a reader would misparse.
fn encode_node(bytes: &mut Vec<u8>, node: &ExprNode) {
    match node {
        ExprNode::Root(root) => {
            bytes.push(0x01);
            root.encode(bytes);
        }
        ExprNode::Unary { op, operand } => {
            bytes.push(0x02);
            bytes.push(op.tag());
            bytes.extend_from_slice(&operand.to_be_bytes());
        }
        ExprNode::Binary { op, left, right } => {
            bytes.push(0x03);
            bytes.push(op.tag());
            bytes.extend_from_slice(&left.to_be_bytes());
            bytes.extend_from_slice(&right.to_be_bytes());
        }
        ExprNode::Select {
            condition,
            if_true,
            if_false,
        } => {
            bytes.push(0x04);
            bytes.extend_from_slice(&condition.to_be_bytes());
            bytes.extend_from_slice(&if_true.to_be_bytes());
            bytes.extend_from_slice(&if_false.to_be_bytes());
        }
    }
}

fn identity_cause(cause: ArtifactDiagnostic) -> ArtifactCodecError {
    ArtifactCodecError::IdentityDerivation { cause }
}

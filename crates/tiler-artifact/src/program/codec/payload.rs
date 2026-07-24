//! The carried backend payload: its compilation subject, and its object bytes.
//!
//! A [`BackendPayloadDescriptor`](super::super::BackendPayloadDescriptor) names
//! a payload. This module is what makes the envelope *carry* one: the exact
//! source that was compiled, the exact toolchain and flags it was compiled
//! with, the mapping from each neutral backend entry key to the backend's own
//! spelling, the target obligations the backend recorded, and — in a separate
//! section — the emitted object bytes.
//!
//! # The identity decision this module encodes
//!
//! **A carried payload is content-addressed over its compilation inputs, and
//! the emitted object is an opaque payload whose digest is integrity rather
//! than identity.**
//!
//! Mechanically: the descriptor's [`PayloadDigest`] is required to equal
//! [`payload_identity`] of the exact payload-metadata bytes, and those bytes
//! contain the source, the target, the flags, and the toolchain provenance and
//! **no object byte at all**. Artifact identity folds the descriptor, so it
//! folds the compilation subject and excludes the object. The object section
//! still carries its own content digest in the manifest, so a substituted
//! library is rejected — but as a corrupted encoding of this artifact, not as a
//! different artifact.
//!
//! The alternative — content-addressing the payload over the emitted bytes —
//! was rejected because it makes artifact identity a function of compiler
//! output reproducibility. `docs/artifact-abi.md` already refuses that promise:
//! "Tiler promises deterministic source, manifest, and identity construction;
//! it does not promise byte-identical Apple output across machines or toolchain
//! builds. A cache hit validates stored payload bytes and never depends on
//! recompiling to reproduce them." An identity that changed whenever a linker
//! embedded a fresh UUID would make that sentence unimplementable.
//!
//! **What this costs, stated rather than hidden.** The codec's other canonical
//! property — equal identity implies equal bytes — now holds for the
//! identity-bearing part of an envelope and not for the object sections. Two
//! bundles built from one compilation subject by a non-reproducible linker have
//! *equal* artifact identity and *different* envelope digests. The expansion
//! cache is therefore keyed by artifact identity, which `docs/artifact-abi.md`
//! already states ("full artifact identity is the key"), and an envelope digest
//! names one published encoding rather than the artifact.
//!
//! # Neutral, and why that is not a compromise
//!
//! Nothing here is Metal. A provenance record names a governed toolchain key, a
//! normalized target string, a family, a language, a deployment minimum, an
//! ordered set of versioned tool components, an SDK identity, and the exact
//! ordered compile and link flags. Those are the identity dimensions
//! `docs/artifact-abi.md` requires for Metal, spelled so a CUDA payload fills
//! the same shape with `nvcc`, `ptxas`, and `sm_90`. The backend owns the
//! *values*; this layer owns the framing, the bounds, and the identity.
//!
//! An entry mapping likewise carries the neutral
//! [`BackendEntryKey`](super::super::BackendEntryKey), the backend's own symbol
//! text, and the ordered transport slots its bindings occupy. The artifact
//! layer never interprets a symbol or a slot; it proves the mapping covers
//! exactly the backend entry keys the artifact's executable entries name.

use super::super::error::ArtifactBuildError;
use super::super::expr::push_slice;
use super::super::keys::{BackendEntryKey, PayloadDigest, RepresentationKey};
use super::super::model::push_len;
use super::decode::Cursor;
use super::digest::DigestAlgorithm;
use super::error::{ArtifactCodecError, CodecLimitKind, OrderedSubject, codec_limit};

/// Versioned domain tag opening the canonical payload-metadata bytes.
pub(super) const PAYLOAD_METADATA_DOMAIN: &[u8] = b"tiler.artifact-envelope.payload-metadata.v1\0";
/// Domain separator of one carried payload's compilation identity.
pub(super) const PAYLOAD_IDENTITY_DOMAIN: &[u8] = b"tiler.artifact-envelope.payload-identity.v1\0";
/// Payload-metadata schema version this build writes and reads.
pub(super) const PAYLOAD_METADATA_SCHEMA: (u16, u16) = (1, 0);

/// Maximum bytes of one carried payload's exact compiled source.
pub(super) const MAX_PAYLOAD_SOURCE_BYTES: usize = 16 * 1024 * 1024;
/// Maximum entry mappings admitted by one carried payload.
pub(super) const MAX_PAYLOAD_ENTRY_MAPPINGS: usize = 4_096;
/// Maximum transport slots admitted by one entry mapping.
pub(super) const MAX_ENTRY_TRANSPORTS: usize = super::super::MAX_ENTRY_BINDINGS;
/// Maximum versioned tool components admitted by one provenance record.
pub(super) const MAX_PROVENANCE_COMPONENTS: usize = 16;
/// Maximum compiler or linker flags admitted by one provenance record.
pub(super) const MAX_PROVENANCE_FLAGS: usize = 256;
/// Maximum recorded target obligations admitted by one carried payload.
pub(super) const MAX_TARGET_OBLIGATIONS: usize = 64;

/// One versioned component of the toolchain that produced a payload.
///
/// The role is the governed name of the component's job — the compiler, the
/// linker — and the version is exactly what that component reported. Absolute
/// paths are deliberately absent: `docs/artifact-abi.md` records them as local
/// provenance rather than portable key material, and a payload identity that
/// folded one would differ between two hosts running the same toolchain.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct ToolComponent {
    /// Governed role key, such as the offline compiler or the linker.
    pub(crate) role: String,
    /// Exact version string the component reported.
    pub(crate) version: String,
}

/// The identity of the SDK one payload was compiled against.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PayloadSdkIdentity {
    /// Canonical SDK selector, such as `macosx`.
    pub(crate) name: String,
    /// Canonical SDK version.
    pub(crate) version: String,
    /// SDK build identifier.
    pub(crate) build: String,
}

/// Everything about *how* a payload was produced that participates in identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PayloadProvenance {
    /// Governed key of the toolchain family that produced the payload.
    pub(crate) toolchain: String,
    /// Normalized target the payload was compiled for.
    pub(crate) target: String,
    /// Artifact family the payload belongs to.
    pub(crate) family: String,
    /// Source language standard the payload was compiled under.
    pub(crate) language: String,
    /// Major component of the requested deployment minimum.
    pub(crate) deployment_major: u16,
    /// Minor component of the requested deployment minimum.
    pub(crate) deployment_minor: u16,
    /// Versioned tool components, in canonical role order.
    pub(crate) components: Vec<ToolComponent>,
    /// Identity of the SDK the payload was compiled against.
    pub(crate) sdk: PayloadSdkIdentity,
    /// The exact ordered compiler flags, excluding file paths.
    ///
    /// Order is meaning here, not presentation: a compiler resolves repeated or
    /// conflicting flags positionally, so a canonicalized list would name a
    /// different invocation.
    pub(crate) compile_flags: Vec<String>,
    /// The exact ordered linker flags, excluding file paths.
    pub(crate) link_flags: Vec<String>,
}

/// The backend's own spelling of one neutral executable entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PayloadEntryMapping {
    /// Neutral backend entry key the artifact's executable entry names.
    pub(crate) entry_key: BackendEntryKey,
    /// The backend's own entry-point symbol.
    pub(crate) symbol: String,
    /// Ordered transport slots this entry's ABI bindings occupy.
    pub(crate) transports: Vec<u32>,
}

/// One target obligation the backend recorded for this payload.
///
/// This is provenance a runtime and an explain surface read, not a predicate
/// the artifact layer evaluates: an obligation a plan can *check* is an ABI
/// expression in the neutral manifest. A backend records here what it could not
/// discharge and what it required of the compilation, so a reader can see why a
/// payload was accepted without re-deriving the backend's own reasoning.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct PayloadTargetObligation {
    /// Governed obligation key.
    pub(crate) key: String,
    /// Governed obligation value.
    pub(crate) value: String,
}

/// The complete compilation subject of one carried payload.
///
/// The canonical encoding of this record is the payload's identity subject, so
/// every field here is an *input* to the compilation. The emitted object is not
/// a field; it travels in its own section.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PayloadMetadata {
    /// Governed representation key of the retained source.
    pub(crate) source_representation: RepresentationKey,
    /// The exact source bytes that were compiled.
    pub(crate) source: Vec<u8>,
    /// How the payload was produced.
    pub(crate) provenance: PayloadProvenance,
    /// Entry mappings in canonical backend-entry-key order.
    pub(crate) entries: Vec<PayloadEntryMapping>,
    /// Recorded target obligations in canonical key order.
    pub(crate) obligations: Vec<PayloadTargetObligation>,
}

/// One carried backend payload: its compilation subject and its object bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PayloadContent {
    /// The compilation subject, which is the payload's identity.
    pub(crate) metadata: PayloadMetadata,
    /// The emitted object bytes, carried opaquely.
    pub(crate) code: Vec<u8>,
}

impl PayloadContent {
    /// Returns the payload identity this content establishes.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError`] when the canonical metadata bytes exceed
    /// the governed opaque-identity bound, which the fixed digest width makes
    /// unreachable and which is propagated rather than asserted.
    pub(crate) fn identity(&self) -> Result<PayloadDigest, ArtifactBuildError> {
        payload_identity(&encode_metadata(&self.metadata))
    }
}

/// Derives the compilation identity of one payload from its metadata bytes.
///
/// The subject is the exact canonical metadata encoding and nothing else, so
/// two compilations that agree on source, target, flags, and toolchain agree on
/// this value whether or not their linkers agree on a byte.
///
/// # Errors
///
/// Returns [`ArtifactBuildError`] from the wrapping constructor.
pub(crate) fn payload_identity(metadata: &[u8]) -> Result<PayloadDigest, ArtifactBuildError> {
    PayloadDigest::from_bytes(
        DigestAlgorithm::GOVERNED
            .digest(PAYLOAD_IDENTITY_DOMAIN, metadata)
            .as_bytes(),
    )
}

/// Encodes one payload's compilation subject into its exact canonical bytes.
///
/// Every variable-length run carries a fixed-width length before its content,
/// and every set-meaning collection is written in the canonical order the
/// decoder proves. Flag lists are the deliberate exception and keep their
/// declared order, which is meaning.
pub(crate) fn encode_metadata(metadata: &PayloadMetadata) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(PAYLOAD_METADATA_DOMAIN);
    bytes.extend_from_slice(&PAYLOAD_METADATA_SCHEMA.0.to_be_bytes());
    bytes.extend_from_slice(&PAYLOAD_METADATA_SCHEMA.1.to_be_bytes());
    push_slice(
        &mut bytes,
        metadata.source_representation.as_str().as_bytes(),
    );
    push_slice(&mut bytes, &metadata.source);

    let provenance = &metadata.provenance;
    push_slice(&mut bytes, provenance.toolchain.as_bytes());
    push_slice(&mut bytes, provenance.target.as_bytes());
    push_slice(&mut bytes, provenance.family.as_bytes());
    push_slice(&mut bytes, provenance.language.as_bytes());
    bytes.extend_from_slice(&provenance.deployment_major.to_be_bytes());
    bytes.extend_from_slice(&provenance.deployment_minor.to_be_bytes());
    push_len(&mut bytes, provenance.components.len());
    for component in &provenance.components {
        push_slice(&mut bytes, component.role.as_bytes());
        push_slice(&mut bytes, component.version.as_bytes());
    }
    push_slice(&mut bytes, provenance.sdk.name.as_bytes());
    push_slice(&mut bytes, provenance.sdk.version.as_bytes());
    push_slice(&mut bytes, provenance.sdk.build.as_bytes());
    push_len(&mut bytes, provenance.compile_flags.len());
    for flag in &provenance.compile_flags {
        push_slice(&mut bytes, flag.as_bytes());
    }
    push_len(&mut bytes, provenance.link_flags.len());
    for flag in &provenance.link_flags {
        push_slice(&mut bytes, flag.as_bytes());
    }

    push_len(&mut bytes, metadata.entries.len());
    for entry in &metadata.entries {
        push_slice(&mut bytes, entry.entry_key.as_bytes());
        push_slice(&mut bytes, entry.symbol.as_bytes());
        push_len(&mut bytes, entry.transports.len());
        for transport in &entry.transports {
            bytes.extend_from_slice(&transport.to_be_bytes());
        }
    }

    push_len(&mut bytes, metadata.obligations.len());
    for obligation in &metadata.obligations {
        push_slice(&mut bytes, obligation.key.as_bytes());
        push_slice(&mut bytes, obligation.value.as_bytes());
    }
    bytes
}

/// Decodes and structurally validates one payload's compilation subject.
///
/// # Errors
///
/// Returns the typed [`ArtifactCodecError`] naming the first boundary that
/// rejected: an unrecognized domain or schema, an exhausted governed budget, a
/// rejected governed key, a non-canonical or repeated set-meaning collection,
/// or trailing bytes.
pub(crate) fn decode_metadata(bytes: &[u8]) -> Result<PayloadMetadata, ArtifactCodecError> {
    let mut cursor = Cursor::new(bytes);
    if cursor.take(PAYLOAD_METADATA_DOMAIN.len())? != PAYLOAD_METADATA_DOMAIN {
        return Err(ArtifactCodecError::BadPayloadMetadataDomain);
    }
    let schema = (cursor.u16()?, cursor.u16()?);
    if schema.0 != PAYLOAD_METADATA_SCHEMA.0 || schema.1 > PAYLOAD_METADATA_SCHEMA.1 {
        return Err(ArtifactCodecError::UnsupportedPayloadMetadataSchema {
            major: schema.0,
            minor: schema.1,
        });
    }
    let source_representation = RepresentationKey::from_owned(cursor.text()?)
        .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?;
    let source = cursor.slice()?;
    codec_limit(
        source.len(),
        MAX_PAYLOAD_SOURCE_BYTES,
        CodecLimitKind::PayloadSourceBytes,
    )?;
    let source = source.to_vec();

    let provenance = decode_provenance(&mut cursor)?;

    let entries = cursor.vec(
        MAX_PAYLOAD_ENTRY_MAPPINGS,
        CodecLimitKind::PayloadEntryMappings,
        |cursor| {
            Ok(PayloadEntryMapping {
                entry_key: BackendEntryKey::from_bytes(cursor.slice()?)
                    .map_err(|cause| ArtifactCodecError::InvalidGovernedKey { cause })?,
                symbol: cursor.text()?,
                transports: cursor.vec(
                    MAX_ENTRY_TRANSPORTS,
                    CodecLimitKind::EntryTransports,
                    Cursor::u32,
                )?,
            })
        },
    )?;
    require_canonical(
        &entries
            .iter()
            .map(|entry| entry.entry_key.as_bytes().to_vec())
            .collect::<Vec<_>>(),
        OrderedSubject::PayloadEntryMapping,
    )?;

    let obligations = cursor.vec(
        MAX_TARGET_OBLIGATIONS,
        CodecLimitKind::TargetObligations,
        |cursor| {
            Ok(PayloadTargetObligation {
                key: cursor.text()?,
                value: cursor.text()?,
            })
        },
    )?;
    require_canonical(&obligations, OrderedSubject::TargetObligation)?;

    if cursor.remaining() != 0 {
        return Err(ArtifactCodecError::TrailingManifestBytes {
            count: cursor.remaining(),
        });
    }
    Ok(PayloadMetadata {
        source_representation,
        source,
        provenance,
        entries,
        obligations,
    })
}

/// Decodes the provenance half of one payload's compilation subject.
///
/// Split from [`decode_metadata`] because provenance is the one part of the
/// subject whose fields are heterogeneous rather than a repeated shape, so it
/// reads as a block on its own and would otherwise dominate its caller.
///
/// The two flag lists are read *without* a canonical-order requirement, unlike
/// every other collection here. A compiler resolves repeated or conflicting
/// flags positionally, so their order is meaning; sorting them would name a
/// different invocation, and rejecting an unsorted list would reject every real
/// one.
///
/// # Errors
///
/// Returns the typed [`ArtifactCodecError`] naming the first boundary that
/// rejected: an exhausted governed budget, invalid text, or a non-canonical or
/// repeated tool-component list.
fn decode_provenance(cursor: &mut Cursor<'_>) -> Result<PayloadProvenance, ArtifactCodecError> {
    let toolchain = cursor.text()?;
    let target = cursor.text()?;
    let family = cursor.text()?;
    let language = cursor.text()?;
    let deployment_major = cursor.u16()?;
    let deployment_minor = cursor.u16()?;
    let components = cursor.vec(
        MAX_PROVENANCE_COMPONENTS,
        CodecLimitKind::ProvenanceComponents,
        |cursor| {
            Ok(ToolComponent {
                role: cursor.text()?,
                version: cursor.text()?,
            })
        },
    )?;
    require_canonical(&components, OrderedSubject::ProvenanceComponent)?;
    let sdk = PayloadSdkIdentity {
        name: cursor.text()?,
        version: cursor.text()?,
        build: cursor.text()?,
    };
    let compile_flags = cursor.vec(
        MAX_PROVENANCE_FLAGS,
        CodecLimitKind::ProvenanceFlags,
        Cursor::text,
    )?;
    let link_flags = cursor.vec(
        MAX_PROVENANCE_FLAGS,
        CodecLimitKind::ProvenanceFlags,
        Cursor::text,
    )?;
    Ok(PayloadProvenance {
        toolchain,
        target,
        family,
        language,
        deployment_major,
        deployment_minor,
        components,
        sdk,
        compile_flags,
        link_flags,
    })
}

/// Proves a set-meaning collection is in canonical order with no repeat.
fn require_canonical<T: Ord>(
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

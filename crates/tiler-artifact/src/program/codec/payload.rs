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
//! **Consequence: a payload is named before it is built.** Every field of
//! [`PayloadMetadata`] is a compilation *input*, so [`PayloadMetadata::identity`]
//! is derivable as soon as the source, the flags, and the resolved toolchain are
//! known — which is before the backend compiler runs, not after it returns. A
//! caller can therefore name the exact descriptor the compiled artifact will
//! carry, and since artifact identity folds the descriptor and the descriptor
//! folds this digest, the *whole* canonical artifact-program identity is
//! available on a cache miss.
//! [`ArtifactProgramBuilder::push_pending_payload`](super::super::ArtifactProgramBuilder::push_pending_payload)
//! is that path and
//! [`push_carried_payload`](super::super::ArtifactProgramBuilder::push_carried_payload)
//! is the same construction once the object has arrived; one function builds the
//! descriptor for both, so the two cannot name different payloads for one
//! compilation.
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
//! normalized target string, a family, a language, an ordered set of versioned
//! tool components, the platform contract its toolchain resolved against, and
//! the exact ordered compile and link flags. Those are the identity dimensions
//! `docs/artifact-abi.md` requires for Metal, spelled so a CUDA payload fills
//! the same shape with `nvcc`, `ptxas`, and `sm_90`. The backend owns the
//! *values*; this layer owns the framing, the bounds, and the identity.
//!
//! **Which fields a payload owes follows the shape it declares, not a backend
//! this layer knows.** Every payload owes a toolchain, a target, a family, a
//! language, and a role and a version for each tool component it lists. A
//! payload whose toolchain resolved against a versioned platform SDK says so
//! with [`PayloadPlatform::VersionedSdk`] and additionally owes a deployment
//! minimum and all three SDK fields; a payload whose toolchain has no SDK says
//! so with [`PayloadPlatform::Unversioned`] and owes none of them. An owed field
//! left empty is refused by name as
//! [`ArtifactBuildError::IncompletePayloadProvenance`] where the payload's
//! identity is derived, and again on decode because an artifact's bytes arrive
//! from a producer this process never ran.
//!
//! The record this replaces made the deployment minimum and the SDK identity
//! unconditional, and ADR 0090 item 14 measured what that cost: the CPU vertical
//! had to mint an SDK for a target that has none. An approximated field that
//! enters durable identity is worse than an absent one, because it makes two
//! unlike artifacts comparable. So the obligation was *redistributed* and not
//! relaxed — a Metal payload owes every field it owed before, and now owes them
//! to a check rather than to a convention.
//!
//! An entry mapping likewise carries the neutral
//! [`BackendEntryKey`], the backend's own symbol
//! text, and the ordered transport slots its bindings occupy. The artifact layer
//! never interprets a symbol or a slot; what it proves is that the mapping
//! *covers* every backend entry key the artifact's executable entries name, and
//! that each mapping places exactly as many transport slots as its entry has
//! bindings plus live-extent operand rows. Both live in [`super::validate`]'s
//! `check_entry_mappings` and run
//! on every decode.
//!
//! The obligation is coverage rather than exhaustion: a compiled object may
//! export a symbol no entry dispatches, and a mapping for one costs a reader
//! nothing because it is folded into the compilation subject and therefore into
//! artifact identity.
//!
//! This paragraph previously claimed the mapping was proven to cover "exactly"
//! the entry keys, and nothing proved it at all — neither the builder nor the
//! decoder correlated the two tables, so an artifact could carry a payload that
//! mapped none of the entries it realized and still decode. The check exists as
//! of `expose-the-dispatch-record-on-a-decoded-artifact`, because a decoded
//! entry that cannot reach its symbol is not a dispatch record.
//!
//! # How the platform block reaches the wire without moving an identity
//!
//! **The versioned-SDK shape keeps the untagged encoding, and the widening is
//! one tag byte appended after the last field.** A [`PayloadPlatform::VersionedSdk`]
//! record encodes to exactly the bytes this module produced before a platform
//! block existed: the deployment minimum in its two `u16` positions ahead of the
//! component list, and the three SDK runs after it. A
//! [`PayloadPlatform::Unversioned`] record writes those same positions as two
//! zeroes and three empty runs, then appends
//! [`PAYLOAD_PLATFORM_UNVERSIONED_TAG`] after the obligation list.
//!
//! *No previously encodable payload's bytes moved.* Every record the previous
//! shape could encode is a versioned-SDK record, and the function encoding one
//! is unchanged byte for byte. Some of those records are no longer encodable at
//! all — an empty SDK name is now a named refusal — but a refusal is not a move.
//!
//! *The encoding stays injective, and the argument is per tag.* The untagged
//! grammar is self-delimiting: every run carries a fixed-width length and every
//! collection a fixed-width count, so the number of bytes one record occupies is
//! a function of the bytes themselves. A versioned encoding is therefore that
//! grammar followed by nothing and an unversioned encoding is that same grammar
//! followed by exactly one byte, so no record of one class can encode to a
//! record of the other's bytes whatever its remaining fields hold. Within the
//! untagged class the map is the previous injective one; within the tagged class
//! the platform positions are constants and every other field is encoded by that
//! same injective grammar, so two unversioned records that encode equally are
//! equal. A *second spelling of one record* is the only thing that could break
//! this, which is why [`decode_metadata`] refuses a tagged encoding that filled a
//! platform position — as [`ArtifactCodecError::PlatformFieldWithoutPlatform`],
//! rather than normalizing the values away — and why no tag value is admitted
//! for the versioned shape.
//!
//! The cost is 28 pinned bytes in every unversioned payload's identity subject,
//! plus the tag. The alternative — moving the platform block behind a leading
//! discriminator, or to the end under a new schema minor — would have moved
//! every already-published Metal payload's identity, and with it every artifact
//! identity and every expansion-cache key that folds one, to save 29 bytes.
//! A later platform shape is another appended tag with its own fields after it,
//! and it moves nothing either.

use super::super::error::{ArtifactBuildError, ProvenanceField};
use super::super::keys::{BackendEntryKey, PayloadDigest, RepresentationKey};
use super::decode::Cursor;
use super::error::{ArtifactCodecError, CodecLimitKind, OrderedSubject, TagSubject, codec_limit};
use tiler_digest::DigestAlgorithm;
use tiler_ir::identity::{push_len, push_slice};

/// Versioned domain tag opening the canonical payload-metadata bytes.
pub(crate) const PAYLOAD_METADATA_DOMAIN: &[u8] = b"tiler.artifact-envelope.payload-metadata.v1\0";
/// Domain separator of one carried payload's compilation identity.
pub(crate) const PAYLOAD_IDENTITY_DOMAIN: &[u8] = b"tiler.artifact-envelope.payload-identity.v1\0";
/// Payload-metadata schema version this build writes and reads.
pub(super) const PAYLOAD_METADATA_SCHEMA: (u16, u16) = (1, 0);

/// Maximum bytes of one carried payload's exact compiled source.
pub(super) const MAX_PAYLOAD_SOURCE_BYTES: usize = 16 * 1024 * 1024;
/// Maximum entry mappings admitted by one carried payload.
pub(super) const MAX_PAYLOAD_ENTRY_MAPPINGS: usize = 4_096;
/// Maximum transport slots admitted by one entry mapping.
pub(super) const MAX_ENTRY_TRANSPORTS: usize =
    super::super::MAX_ENTRY_BINDINGS.saturating_add(super::super::MAX_ENTRY_EXTENTS);
/// Maximum versioned tool components admitted by one provenance record.
pub(super) const MAX_PROVENANCE_COMPONENTS: usize = 16;
/// Maximum compiler or linker flags admitted by one provenance record.
pub(super) const MAX_PROVENANCE_FLAGS: usize = 256;
/// Maximum recorded target obligations admitted by one carried payload.
pub(super) const MAX_TARGET_OBLIGATIONS: usize = 64;

/// Appended tag of a payload whose toolchain resolved against no versioned SDK.
///
/// The one admitted tag value. The versioned-SDK shape is the *untagged*
/// encoding — the module documentation states why — so admitting a second tag
/// naming it would give one record two spellings and make payload identity
/// non-injective.
pub(super) const PAYLOAD_PLATFORM_UNVERSIONED_TAG: u8 = 0x00;

/// One versioned component of the toolchain that produced a payload.
///
/// The role is the governed name of the component's job — the compiler, the
/// linker — and the version is exactly what that component reported. Absolute
/// paths are deliberately absent: `docs/artifact-abi.md` records them as local
/// provenance rather than portable key material, and a payload identity that
/// folded one would differ between two hosts running the same toolchain.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ToolComponent {
    /// Governed role key, such as the offline compiler or the linker.
    pub role: String,
    /// Exact version string the component reported.
    pub version: String,
}

/// The identity of the SDK one payload was compiled against.
///
/// Reachable only through [`PayloadPlatform::VersionedSdk`], because an SDK
/// identity is a claim a backend either has or does not have. All three fields
/// are owed wherever this record appears.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayloadSdkIdentity {
    /// Canonical SDK selector, such as `macosx`.
    pub name: String,
    /// Canonical SDK version.
    pub version: String,
    /// SDK build identifier.
    pub build: String,
}

/// The platform contract one payload's toolchain resolved against.
///
/// Deliberately an enumeration rather than an optional SDK identity. "This
/// target has no versioned platform SDK" is a fact a backend *states*, and a
/// reader that has to distinguish it from "the producer left this blank" is
/// reading an absence where it needs a claim — which is the same conflation
/// that made a CPU payload mint an SDK name in the first place.
///
/// A later shape — a versioned toolkit with no deployment minimum, say — is a
/// third variant carried by a new appended tag, and adding one moves no existing
/// payload's bytes. The module documentation carries that derivation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PayloadPlatform {
    /// The toolchain resolves against no versioned SDK and states no deployment
    /// minimum.
    ///
    /// A payload here owes no platform field and may state none: a decoder
    /// refuses an encoding that fills one.
    Unversioned,
    /// The toolchain resolved against a versioned platform SDK.
    ///
    /// Every field below is owed. This is the shape the Apple toolchain fills,
    /// and it is the shape every payload encodable before the platform block
    /// existed is read as.
    VersionedSdk {
        /// Major component of the requested deployment minimum.
        ///
        /// Owed, and therefore non-zero: a deployment minimum of `0` is the
        /// absence of one, and stating an absence is what
        /// [`Self::Unversioned`] is for.
        deployment_major: u16,
        /// Minor component of the requested deployment minimum.
        ///
        /// Legitimately `0` — `15.0` is a real deployment minimum — so it
        /// carries no separate obligation.
        deployment_minor: u16,
        /// Identity of the SDK the payload was compiled against.
        sdk: PayloadSdkIdentity,
    },
}

/// Everything about *how* a payload was produced that participates in identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayloadProvenance {
    /// Governed key of the toolchain family that produced the payload.
    pub toolchain: String,
    /// Normalized target the payload was compiled for.
    pub target: String,
    /// Artifact family the payload belongs to.
    pub family: String,
    /// Source language standard the payload was compiled under.
    pub language: String,
    /// The platform contract the producing toolchain resolved against.
    pub platform: PayloadPlatform,
    /// Versioned tool components, in canonical role order.
    pub components: Vec<ToolComponent>,
    /// The exact ordered compiler flags, excluding file paths.
    ///
    /// Order is meaning here, not presentation: a compiler resolves repeated or
    /// conflicting flags positionally, so a canonicalized list would name a
    /// different invocation.
    pub compile_flags: Vec<String>,
    /// The exact ordered linker flags, excluding file paths.
    pub link_flags: Vec<String>,
}

/// Proves every field the record's declared shape owes carries a value.
///
/// The record is destructured irrefutably for the same reason the encoder is: a
/// field added to it must be considered here rather than silently owing nothing.
///
/// # Errors
///
/// Returns [`ArtifactBuildError::IncompletePayloadProvenance`] naming the first
/// owed field left empty.
pub(crate) fn check_provenance(provenance: &PayloadProvenance) -> Result<(), ArtifactBuildError> {
    let PayloadProvenance {
        toolchain,
        target,
        family,
        language,
        platform,
        components,
        // Not owed. Flag order is meaning and an empty list is a legitimate
        // invocation, so there is no emptiness rule over either list that would
        // not reject real compilations.
        compile_flags: _,
        link_flags: _,
    } = provenance;
    require_stated(toolchain, ProvenanceField::Toolchain)?;
    require_stated(target, ProvenanceField::Target)?;
    require_stated(family, ProvenanceField::Family)?;
    require_stated(language, ProvenanceField::Language)?;
    for ToolComponent { role, version } in components {
        require_stated(role, ProvenanceField::ToolComponentRole)?;
        require_stated(version, ProvenanceField::ToolComponentVersion)?;
    }
    match platform {
        PayloadPlatform::Unversioned => Ok(()),
        PayloadPlatform::VersionedSdk {
            deployment_major,
            deployment_minor: _,
            sdk:
                PayloadSdkIdentity {
                    name,
                    version,
                    build,
                },
        } => {
            if *deployment_major == 0 {
                return Err(ArtifactBuildError::IncompletePayloadProvenance {
                    field: ProvenanceField::DeploymentMinimum,
                });
            }
            require_stated(name, ProvenanceField::SdkName)?;
            require_stated(version, ProvenanceField::SdkVersion)?;
            require_stated(build, ProvenanceField::SdkBuild)
        }
    }
}

/// Rejects an owed provenance field that carries no value.
fn require_stated(value: &str, field: ProvenanceField) -> Result<(), ArtifactBuildError> {
    if value.is_empty() {
        return Err(ArtifactBuildError::IncompletePayloadProvenance { field });
    }
    Ok(())
}

/// The backend's own spelling of one neutral executable entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayloadEntryMapping {
    /// Neutral backend entry key the artifact's executable entry names.
    pub entry_key: BackendEntryKey,
    /// The backend's own entry-point symbol.
    pub symbol: String,
    /// Ordered transport slots this entry's ABI bindings occupy.
    pub transports: Vec<u32>,
}

/// One target obligation the backend recorded for this payload.
///
/// This is provenance a runtime and an explain surface read, not a predicate
/// the artifact layer evaluates: an obligation a plan can *check* is an ABI
/// expression in the neutral manifest. A backend records here what it could not
/// discharge and what it required of the compilation, so a reader can see why a
/// payload was accepted without re-deriving the backend's own reasoning.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct PayloadTargetObligation {
    /// Governed obligation key.
    pub key: String,
    /// Governed obligation value.
    pub value: String,
}

/// The complete compilation subject of one carried payload.
///
/// The canonical encoding of this record is the payload's identity subject, so
/// every field here is an *input* to the compilation. The emitted object is not
/// a field; it travels in its own section.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayloadMetadata {
    /// Governed representation key of the retained source.
    pub source_representation: RepresentationKey,
    /// The exact source bytes that were compiled.
    pub source: Vec<u8>,
    /// How the payload was produced.
    pub provenance: PayloadProvenance,
    /// Entry mappings in canonical backend-entry-key order.
    pub entries: Vec<PayloadEntryMapping>,
    /// Recorded target obligations in canonical key order.
    pub obligations: Vec<PayloadTargetObligation>,
}

impl PayloadMetadata {
    /// Returns the payload identity this compilation subject establishes.
    ///
    /// **Derivable before the compilation it describes has run.** Every field
    /// of this record is an input the caller already holds when it decides
    /// whether to compile at all, and the module documentation above states why
    /// no emitted byte joins them. That is the property an expansion cache
    /// rests on: the key is needed on a *miss*, and a digest that could only be
    /// taken once the object existed would name the answer rather than the
    /// question.
    ///
    /// It is the *same* derivation [`PayloadContent::identity`] performs, not a
    /// parallel one — that method delegates here — so a payload named before
    /// its compilation and the same payload named after it cannot disagree.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactBuildError::IncompletePayloadProvenance`] when the
    /// provenance omits a field the shape it declares owes. A subject that does
    /// not state what it claims to state has no identity rather than a weaker
    /// one, and this is the single derivation both payload push paths reach, so
    /// nothing enters an artifact without passing it.
    ///
    /// Also returns [`ArtifactBuildError`] when the canonical metadata bytes
    /// exceed the governed opaque-identity bound, which the fixed digest width
    /// makes unreachable and which is propagated rather than asserted.
    pub fn identity(&self) -> Result<PayloadDigest, ArtifactBuildError> {
        check_provenance(&self.provenance)?;
        payload_identity(&encode_metadata(self))
    }
}

/// One carried backend payload: its compilation subject and its object bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayloadContent {
    /// The compilation subject, which is the payload's identity.
    pub metadata: PayloadMetadata,
    /// The emitted object bytes, carried opaquely.
    pub code: Vec<u8>,
}

impl PayloadContent {
    /// Returns the payload identity this content establishes.
    ///
    /// The object is not consulted, so this equals
    /// [`PayloadMetadata::identity`] of the subject alone. It delegates rather
    /// than repeating the derivation.
    ///
    /// # Errors
    ///
    /// Returns the errors [`PayloadMetadata::identity`] returns.
    pub fn identity(&self) -> Result<PayloadDigest, ArtifactBuildError> {
        self.metadata.identity()
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
pub fn payload_identity(metadata: &[u8]) -> Result<PayloadDigest, ArtifactBuildError> {
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
///
/// Every record below is destructured irrefutably rather than read field by
/// field, so a field added to any of them fails to compile here instead of
/// silently leaving the subject. That mechanism is worth more on this record
/// than on most: these bytes are the payload's identity, artifact identity
/// folds that identity, and an expansion cache keys on artifact identity — so
/// an input that quietly missed the encoding would not be a weaker digest, it
/// would be two compilations sharing one cache entry.
pub(crate) fn encode_metadata(metadata: &PayloadMetadata) -> Vec<u8> {
    let PayloadMetadata {
        source_representation,
        source,
        provenance,
        entries,
        obligations,
    } = metadata;
    let PayloadProvenance {
        toolchain,
        target,
        family,
        language,
        platform,
        components,
        compile_flags,
        link_flags,
    } = provenance;
    // The platform block is written at the field positions the record used
    // before it had a platform block, and the *unversioned* shape is the one
    // that carries a tag — appended after the last field, so no already
    // encodable payload's bytes move. The module documentation carries the
    // per-tag injectivity argument that makes the two classes disjoint.
    let (deployment_major, deployment_minor, sdk_name, sdk_version, sdk_build, platform_tag) =
        match platform {
            PayloadPlatform::VersionedSdk {
                deployment_major,
                deployment_minor,
                sdk:
                    PayloadSdkIdentity {
                        name,
                        version,
                        build,
                    },
            } => (
                *deployment_major,
                *deployment_minor,
                name.as_str(),
                version.as_str(),
                build.as_str(),
                None,
            ),
            PayloadPlatform::Unversioned => {
                (0, 0, "", "", "", Some(PAYLOAD_PLATFORM_UNVERSIONED_TAG))
            }
        };

    let mut bytes = Vec::new();
    bytes.extend_from_slice(PAYLOAD_METADATA_DOMAIN);
    bytes.extend_from_slice(&PAYLOAD_METADATA_SCHEMA.0.to_be_bytes());
    bytes.extend_from_slice(&PAYLOAD_METADATA_SCHEMA.1.to_be_bytes());
    push_slice(&mut bytes, source_representation.as_str().as_bytes());
    push_slice(&mut bytes, source);

    push_slice(&mut bytes, toolchain.as_bytes());
    push_slice(&mut bytes, target.as_bytes());
    push_slice(&mut bytes, family.as_bytes());
    push_slice(&mut bytes, language.as_bytes());
    bytes.extend_from_slice(&deployment_major.to_be_bytes());
    bytes.extend_from_slice(&deployment_minor.to_be_bytes());
    push_len(&mut bytes, components.len());
    for ToolComponent { role, version } in components {
        push_slice(&mut bytes, role.as_bytes());
        push_slice(&mut bytes, version.as_bytes());
    }
    push_slice(&mut bytes, sdk_name.as_bytes());
    push_slice(&mut bytes, sdk_version.as_bytes());
    push_slice(&mut bytes, sdk_build.as_bytes());
    push_len(&mut bytes, compile_flags.len());
    for flag in compile_flags {
        push_slice(&mut bytes, flag.as_bytes());
    }
    push_len(&mut bytes, link_flags.len());
    for flag in link_flags {
        push_slice(&mut bytes, flag.as_bytes());
    }

    push_len(&mut bytes, entries.len());
    for PayloadEntryMapping {
        entry_key,
        symbol,
        transports,
    } in entries
    {
        push_slice(&mut bytes, entry_key.as_bytes());
        push_slice(&mut bytes, symbol.as_bytes());
        push_len(&mut bytes, transports.len());
        for transport in transports {
            bytes.extend_from_slice(&transport.to_be_bytes());
        }
    }

    push_len(&mut bytes, obligations.len());
    for PayloadTargetObligation { key, value } in obligations {
        push_slice(&mut bytes, key.as_bytes());
        push_slice(&mut bytes, value.as_bytes());
    }

    if let Some(tag) = platform_tag {
        bytes.push(tag);
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

    let draft = decode_provenance(&mut cursor)?;

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

    // The platform tag is the last thing in the record, and it has to be: an
    // untagged encoding is recognized by there being nothing left, which is only
    // decidable at the end. Everything before it was read at the positions the
    // record used before the tag existed.
    let platform = draft.platform(read_platform_tag(&mut cursor)?)?;
    let provenance = draft.resolve(platform);

    if cursor.remaining() != 0 {
        return Err(ArtifactCodecError::TrailingManifestBytes {
            count: cursor.remaining(),
        });
    }
    // The same obligation the producer's identity derivation proved, re-proven
    // because these bytes came from a producer this process never ran.
    check_provenance(&provenance)?;
    Ok(PayloadMetadata {
        source_representation,
        source,
        provenance,
        entries,
        obligations,
    })
}

/// One payload's provenance with its platform shape not yet resolved.
///
/// The platform fields are read at their fixed positions long before the tag
/// that says what they mean, so they are carried rather than interpreted. The
/// two halves are rejoined by [`Self::resolve`] and nowhere else.
struct ProvenanceDraft {
    toolchain: String,
    target: String,
    family: String,
    language: String,
    deployment_major: u16,
    deployment_minor: u16,
    sdk: PayloadSdkIdentity,
    components: Vec<ToolComponent>,
    compile_flags: Vec<String>,
    link_flags: Vec<String>,
}

impl ProvenanceDraft {
    /// Rejoins the fields read at fixed positions with the resolved shape.
    ///
    /// The platform values the draft carries are consumed by
    /// [`Self::platform`], which either promotes them into the versioned shape
    /// or proves they were unstated, so this takes the resolved shape rather
    /// than choosing one.
    fn resolve(self, platform: PayloadPlatform) -> PayloadProvenance {
        let Self {
            toolchain,
            target,
            family,
            language,
            deployment_major: _,
            deployment_minor: _,
            sdk: _,
            components,
            compile_flags,
            link_flags,
        } = self;
        PayloadProvenance {
            toolchain,
            target,
            family,
            language,
            platform,
            components,
            compile_flags,
            link_flags,
        }
    }

    /// Reads the platform shape this draft's fixed-position fields encode.
    ///
    /// # Errors
    ///
    /// Returns [`ArtifactCodecError::PlatformFieldWithoutPlatform`] when an
    /// unversioned encoding filled a platform position, or
    /// [`ArtifactCodecError::UnknownTag`] for a tag this build does not
    /// implement.
    fn platform(&self, tagged: bool) -> Result<PayloadPlatform, ArtifactCodecError> {
        if !tagged {
            return Ok(PayloadPlatform::VersionedSdk {
                deployment_major: self.deployment_major,
                deployment_minor: self.deployment_minor,
                sdk: self.sdk.clone(),
            });
        }
        refuse_stated(
            self.deployment_major != 0 || self.deployment_minor != 0,
            ProvenanceField::DeploymentMinimum,
        )?;
        refuse_stated(!self.sdk.name.is_empty(), ProvenanceField::SdkName)?;
        refuse_stated(!self.sdk.version.is_empty(), ProvenanceField::SdkVersion)?;
        refuse_stated(!self.sdk.build.is_empty(), ProvenanceField::SdkBuild)?;
        Ok(PayloadPlatform::Unversioned)
    }
}

/// Reads the appended platform tag, reporting whether the record carries one.
///
/// Nothing left is the versioned-SDK shape: it is what every payload encodable
/// before the platform block existed encodes to, and keeping it untagged is what
/// kept those bytes still. The tag byte is consumed when present, so the
/// trailing-bytes check that follows still sees an exhausted cursor.
///
/// # Errors
///
/// Returns [`ArtifactCodecError::UnknownTag`] for any value but
/// [`PAYLOAD_PLATFORM_UNVERSIONED_TAG`] — including a hypothetical tag for the
/// versioned shape, which is refused because one record with two spellings is
/// two payload identities.
fn read_platform_tag(cursor: &mut Cursor<'_>) -> Result<bool, ArtifactCodecError> {
    if cursor.remaining() == 0 {
        return Ok(false);
    }
    match cursor.u8()? {
        PAYLOAD_PLATFORM_UNVERSIONED_TAG => Ok(true),
        tag => Err(ArtifactCodecError::UnknownTag {
            subject: TagSubject::PayloadPlatform,
            tag,
        }),
    }
}

/// Rejects a platform field stated by a payload whose shape owes none.
fn refuse_stated(stated: bool, field: ProvenanceField) -> Result<(), ArtifactCodecError> {
    if stated {
        return Err(ArtifactCodecError::PlatformFieldWithoutPlatform { field });
    }
    Ok(())
}

/// Decodes the provenance half of one payload's compilation subject.
///
/// Split from [`decode_metadata`] because provenance is the one part of the
/// subject whose fields are heterogeneous rather than a repeated shape, so it
/// reads as a block on its own and would otherwise dominate its caller.
///
/// It yields a [`ProvenanceDraft`] rather than a finished record because the
/// platform fields it reads here are only *bytes at fixed positions* until the
/// tag at the end of the record says which shape they encode.
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
fn decode_provenance(cursor: &mut Cursor<'_>) -> Result<ProvenanceDraft, ArtifactCodecError> {
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
    Ok(ProvenanceDraft {
        toolchain,
        target,
        family,
        language,
        deployment_major,
        deployment_minor,
        sdk,
        components,
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

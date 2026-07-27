//! The one immutable self-validating bundle stored per key.
//!
//! # What a bundle has to be able to reject
//!
//! Not just damage. A bundle is read from a directory any process on the host
//! may write to, so it must reject bytes that are entirely something else, bytes
//! from an older or newer layout, bytes written for a *different* key, and bytes
//! that are internally consistent but filed in the wrong place. Every one of
//! those is a miss under ADR 0050, and every one of them carries the exact
//! boundary that refused so a permanently-rejecting cache is observable.
//!
//! # The frame
//!
//! ```text
//! 0..8     magic `TLRCACHE`
//! 8..10    schema major
//! 10..12   schema minor
//! 12       governed digest algorithm tag
//! 13..16   reserved, all zero
//! 16..24   exact total length of the bundle
//! 24..56   the compilation key this bundle was published under
//! 56..64   section count
//! 64..     one descriptor per section: purpose, offset, length, digest
//! ...      the section bytes, contiguous, in descriptor order
//! ```
//!
//! Big-endian throughout, matching every other canonical encoding in the
//! workspace.
//!
//! # Why the compilation subject travels beside the artifact
//!
//! The embedded key proves a bundle was published under key `K`. On its own that
//! only moves the question: a writer that derived `K` from one subject and
//! packaged an artifact for another would produce a bundle every reader accepts.
//! Carrying the exact subject bytes lets a reader *re-derive* `K` and refuse the
//! bundle when they disagree, so the key-to-subject binding is checked on every
//! hit instead of trusted. It also makes the subject available for diagnosis,
//! which is the difference between "this entry was rejected" and "this entry was
//! rejected and here is what it claimed to be".

use core::fmt;
use core::ops::Range;

use tiler_artifact::program::{DIGEST_BYTES, Digest, DigestAlgorithm};

use super::key::CacheKey;
use super::limits::Limits;

/// Opening bytes of every cache bundle.
const MAGIC: &[u8; 8] = b"TLRCACHE";
/// Bundle schema this build writes and reads.
const SCHEMA: (u16, u16) = (1, 0);
/// Domain separator of one framed section's content digest.
pub(super) const SECTION_DIGEST_DOMAIN: &[u8] = b"tiler.cache.bundle-section.v1\0";

/// Fixed-width framing header, before the descriptor table.
const HEADER_BYTES: usize = 64;
/// One section descriptor: purpose, offset, length, digest.
const DESCRIPTOR_BYTES: usize = 4 + 8 + 8 + DIGEST_BYTES;

const SCHEMA_MAJOR_AT: usize = 8;
const SCHEMA_MINOR_AT: usize = 10;
const ALGORITHM_AT: usize = 12;
const RESERVED_AT: usize = 13;
const TOTAL_LENGTH_AT: usize = 16;
const KEY_AT: usize = 24;
const SECTION_COUNT_AT: usize = 56;

/// What one framed section of a bundle carries.
///
/// Deliberately **not** `#[non_exhaustive]` (ADR 0074 convention 5b): the
/// encoder and the decoder map this vocabulary totally, and a wildcard arm at
/// either could only invent a meaning the variant alone determines.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BundleSection {
    /// The producer's canonical compilation subject, from which the key derives.
    CompilationSubject,
    /// The target-neutral artifact envelope this compilation produced.
    ArtifactEnvelope,
}

impl BundleSection {
    /// Sections in the exact order a bundle frames them.
    const ORDER: [Self; 2] = [Self::CompilationSubject, Self::ArtifactEnvelope];

    /// Returns this section's stable wire tag.
    ///
    /// An arm that states its constant, never a discriminant read from
    /// declaration order.
    const fn tag(self) -> u32 {
        match self {
            Self::CompilationSubject => 0x0000_0001,
            Self::ArtifactEnvelope => 0x0000_0002,
        }
    }

    /// Resolves a wire tag, or `None` for a purpose this build does not know.
    const fn from_tag(tag: u32) -> Option<Self> {
        match tag {
            0x0000_0001 => Some(Self::CompilationSubject),
            0x0000_0002 => Some(Self::ArtifactEnvelope),
            _ => None,
        }
    }

    /// Returns this section's stable lowercase identifier, for diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CompilationSubject => "compilation-subject",
            Self::ArtifactEnvelope => "artifact-envelope",
        }
    }
}

impl fmt::Display for BundleSection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Where each validated section sits inside the bytes it was decoded from.
///
/// Spans rather than slices, so a caller that owns the buffer can keep owning it
/// and still name the two sections. The frame already lays them out contiguously
/// inside one run, so the bytes a reader wants are the bytes it read — and
/// returning borrows instead would force any caller wanting to *keep* them to
/// copy the whole bundle a second time, which is what these ranges exist to
/// avoid.
///
/// Every range is produced only by [`decode`], and only after that function has
/// proven the span lies inside the buffer and matches its declared digest. So
/// indexing a buffer with one of these cannot panic *for the buffer it was
/// decoded from*; pairing a range with a different buffer is a caller error the
/// type does not prevent, and no call site does it.
#[derive(Clone, Debug)]
pub(crate) struct BundleView {
    pub(crate) key: CacheKey,
    pub(crate) subject: Range<usize>,
    pub(crate) envelope: Range<usize>,
}

/// Encodes one bundle.
///
/// The key is derived here from the subject rather than accepted from the
/// caller, so a published bundle cannot be filed under a key its subject does
/// not produce.
pub(crate) fn encode(
    subject: &[u8],
    envelope: &[u8],
    limits: &Limits,
) -> Result<(CacheKey, Vec<u8>), BundleRejection> {
    let key = CacheKey::derive_bytes(subject);
    let sections = [subject, envelope];
    let table_bytes = DESCRIPTOR_BYTES * sections.len();
    let body_bytes: usize = sections.iter().map(|section| section.len()).sum();
    let total = HEADER_BYTES + table_bytes + body_bytes;

    for (index, section) in sections.iter().enumerate() {
        let length = section.len() as u64;
        if length > limits.max_section_bytes {
            return Err(BundleRejection::SectionTooLarge {
                index,
                length,
                limit: limits.max_section_bytes,
            });
        }
    }
    let total_length = total as u64;
    if total_length > limits.max_bundle_bytes {
        return Err(BundleRejection::BundleTooLarge {
            declared: total_length,
            limit: limits.max_bundle_bytes,
        });
    }

    let mut bytes = Vec::with_capacity(total);
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&SCHEMA.0.to_be_bytes());
    bytes.extend_from_slice(&SCHEMA.1.to_be_bytes());
    bytes.push(DigestAlgorithm::GOVERNED.tag());
    bytes.extend_from_slice(&[0, 0, 0]);
    bytes.extend_from_slice(&total_length.to_be_bytes());
    bytes.extend_from_slice(key.as_bytes());
    // A fixed-offset header field and not `tiler_ir::identity` framing: it is
    // written at `SECTION_COUNT_AT`, read back by `read_u64`, and the sections
    // it counts are found through the descriptor table's explicit offsets rather
    // than by following a prefix. These bytes are never digested into an
    // identity — the key derives from the subject, and each section carries its
    // own digest. `ComposedSubject` is where this crate does frame an identity.
    bytes.extend_from_slice(&(sections.len() as u64).to_be_bytes());
    debug_assert_eq!(
        bytes.len(),
        HEADER_BYTES,
        "the framing header is fixed width"
    );

    let mut offset = (HEADER_BYTES + table_bytes) as u64;
    for (purpose, section) in BundleSection::ORDER.iter().zip(sections) {
        let length = section.len() as u64;
        bytes.extend_from_slice(&purpose.tag().to_be_bytes());
        bytes.extend_from_slice(&offset.to_be_bytes());
        bytes.extend_from_slice(&length.to_be_bytes());
        bytes.extend_from_slice(section_digest(section).as_bytes());
        offset += length;
    }
    for section in sections {
        bytes.extend_from_slice(section);
    }
    debug_assert_eq!(bytes.len(), total, "the encoded length is the declared one");
    Ok((key, bytes))
}

/// Decodes and completely validates one bundle against the key it was requested
/// under.
///
/// There is no partial mode and no order in which a caller may skip a step. The
/// checks run in the order below because each one bounds what the next may
/// read: nothing is indexed before the length that frames it has been proven
/// against the bytes actually present.
pub(crate) fn decode(
    bytes: &[u8],
    requested: &CacheKey,
    limits: &Limits,
) -> Result<BundleView, BundleRejection> {
    let header = decode_header(bytes, requested, limits)?;
    let (subject, envelope) = decode_sections(bytes, &header, limits)?;

    // The key is a *function* of the subject, so a bundle whose carried subject
    // does not produce the key it is filed under is refused even though every
    // digest in it verified. A forger recomputes digests; it cannot recompute
    // this without changing the path the entry lives at.
    let derived = CacheKey::derive_bytes(&bytes[subject.clone()]);
    if derived != header.key {
        return Err(BundleRejection::KeyNotDerivedFromSubject {
            embedded: header.key.label(),
            derived: derived.label(),
        });
    }

    Ok(BundleView {
        key: header.key,
        subject,
        envelope,
    })
}

/// What the fixed-width framing header declares, once every field of it has
/// been checked.
struct Header {
    key: CacheKey,
    total: u64,
    sections: usize,
    table_end: usize,
}

/// Validates the framing header and the embedded key.
fn decode_header(
    bytes: &[u8],
    requested: &CacheKey,
    limits: &Limits,
) -> Result<Header, BundleRejection> {
    if bytes.len() < HEADER_BYTES {
        return Err(BundleRejection::Truncated {
            found: bytes.len(),
            needed: HEADER_BYTES,
        });
    }
    if &bytes[..MAGIC.len()] != MAGIC {
        return Err(BundleRejection::Magic);
    }
    let major = read_u16(bytes, SCHEMA_MAJOR_AT);
    let minor = read_u16(bytes, SCHEMA_MINOR_AT);
    if major != SCHEMA.0 || minor > SCHEMA.1 {
        return Err(BundleRejection::Schema { major, minor });
    }
    let algorithm_tag = bytes[ALGORITHM_AT];
    // Resolved from the tag rather than inferred from a digest width, which
    // `docs/artifact-abi.md` forbids. A tag this build does not implement is a
    // refusal, never a best-effort read under a different algorithm.
    let Some(algorithm) = DigestAlgorithm::from_tag(algorithm_tag) else {
        return Err(BundleRejection::DigestAlgorithm { tag: algorithm_tag });
    };
    if algorithm != DigestAlgorithm::GOVERNED {
        return Err(BundleRejection::DigestAlgorithm { tag: algorithm_tag });
    }
    if bytes[RESERVED_AT..TOTAL_LENGTH_AT] != [0, 0, 0] {
        return Err(BundleRejection::ReservedNotZero);
    }
    let declared_total = read_u64(bytes, TOTAL_LENGTH_AT);
    if declared_total > limits.max_bundle_bytes {
        return Err(BundleRejection::BundleTooLarge {
            declared: declared_total,
            limit: limits.max_bundle_bytes,
        });
    }
    if declared_total != bytes.len() as u64 {
        return Err(BundleRejection::TotalLength {
            declared: declared_total,
            found: bytes.len() as u64,
        });
    }

    let mut embedded = [0_u8; DIGEST_BYTES];
    embedded.copy_from_slice(&bytes[KEY_AT..KEY_AT + DIGEST_BYTES]);
    let embedded = CacheKey::from_wire(embedded);
    if embedded != *requested {
        return Err(BundleRejection::KeyMismatch {
            requested: requested.label(),
            embedded: embedded.label(),
        });
    }

    let declared_sections = read_u64(bytes, SECTION_COUNT_AT);
    if declared_sections > u64::from(limits.max_sections) {
        return Err(BundleRejection::SectionCount {
            declared: declared_sections,
            limit: limits.max_sections,
        });
    }
    // Bounded by `max_sections` above, so the conversion cannot fail on any
    // profile whose pointers are at least 32 bits wide. It is still a checked
    // conversion into a typed rejection rather than a cast: a `usize` that could
    // not hold the count would silently frame a different number of sections.
    let sections =
        usize::try_from(declared_sections).map_err(|_| BundleRejection::SectionCount {
            declared: declared_sections,
            limit: limits.max_sections,
        })?;
    let table_end = HEADER_BYTES + DESCRIPTOR_BYTES * sections;
    if bytes.len() < table_end {
        return Err(BundleRejection::Truncated {
            found: bytes.len(),
            needed: table_end,
        });
    }

    Ok(Header {
        key: embedded,
        total: declared_total,
        sections,
        table_end,
    })
}

/// Validates every section descriptor and the bytes it frames.
fn decode_sections(
    bytes: &[u8],
    header: &Header,
    limits: &Limits,
) -> Result<(Range<usize>, Range<usize>), BundleRejection> {
    let mut subject: Option<Range<usize>> = None;
    let mut envelope: Option<Range<usize>> = None;
    let mut expected_offset = header.table_end as u64;
    for index in 0..header.sections {
        let at = HEADER_BYTES + DESCRIPTOR_BYTES * index;
        let tag = read_u32(bytes, at);
        let offset = read_u64(bytes, at + 4);
        let length = read_u64(bytes, at + 12);
        let mut declared = [0_u8; DIGEST_BYTES];
        declared.copy_from_slice(&bytes[at + 20..at + 20 + DIGEST_BYTES]);

        let Some(purpose) = BundleSection::from_tag(tag) else {
            return Err(BundleRejection::UnknownSectionPurpose { index, tag });
        };
        if length > limits.max_section_bytes {
            return Err(BundleRejection::SectionTooLarge {
                index,
                length,
                limit: limits.max_section_bytes,
            });
        }
        // Contiguity is what makes the frame canonical: sections start
        // immediately after the table and follow one another with no gap and no
        // overlap, so one byte run cannot be counted twice and no unreferenced
        // byte can ride along inside a validated total length.
        if offset != expected_offset {
            return Err(BundleRejection::SectionNotContiguous {
                index,
                expected: expected_offset,
                found: offset,
            });
        }
        let end = offset
            .checked_add(length)
            .ok_or(BundleRejection::SectionBounds {
                index,
                offset,
                length,
            })?;
        if end > header.total {
            return Err(BundleRejection::SectionBounds {
                index,
                offset,
                length,
            });
        }
        expected_offset = end;

        // Both bounds are at or below `header.total`, which the header proved
        // equal to `bytes.len()`, so each conversion is checked into the
        // rejection that already describes an out-of-range span rather than cast.
        let bounds = BundleRejection::SectionBounds {
            index,
            offset,
            length,
        };
        let (start, stop) = (
            usize::try_from(offset).map_err(|_| bounds.clone())?,
            usize::try_from(end).map_err(|_| bounds)?,
        );
        let content = &bytes[start..stop];
        if section_digest(content) != Digest::from_wire(declared) {
            return Err(BundleRejection::SectionDigest { purpose });
        }
        let slot = match purpose {
            BundleSection::CompilationSubject => &mut subject,
            BundleSection::ArtifactEnvelope => &mut envelope,
        };
        if slot.is_some() {
            return Err(BundleRejection::DuplicateSection { purpose });
        }
        *slot = Some(start..stop);
    }
    if expected_offset != header.total {
        return Err(BundleRejection::TrailingBytes {
            after: expected_offset,
            total: header.total,
        });
    }

    Ok((
        subject.ok_or(BundleRejection::MissingSection {
            purpose: BundleSection::CompilationSubject,
        })?,
        envelope.ok_or(BundleRejection::MissingSection {
            purpose: BundleSection::ArtifactEnvelope,
        })?,
    ))
}

/// Digests one section's exact bytes under the governed algorithm and domain.
fn section_digest(content: &[u8]) -> Digest {
    DigestAlgorithm::GOVERNED.digest(SECTION_DIGEST_DOMAIN, content)
}

fn read_u16(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().expect("a fixed-width field"))
}

fn read_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().expect("a fixed-width field"))
}

fn read_u64(bytes: &[u8], at: usize) -> u64 {
    u64::from_be_bytes(bytes[at..at + 8].try_into().expect("a fixed-width field"))
}

/// Why a byte run is not a valid bundle for the key it was requested under.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a caller logs or forwards
/// these and does not map them totally.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BundleRejection {
    /// Fewer bytes are present than a structure already declared needs.
    Truncated {
        /// Bytes actually present.
        found: usize,
        /// Bytes the declared structure requires.
        needed: usize,
    },
    /// The opening bytes are not a cache bundle's magic.
    Magic,
    /// The bundle schema is not one this build reads.
    Schema {
        /// Declared major version.
        major: u16,
        /// Declared minor version.
        minor: u16,
    },
    /// The digest algorithm tag is not the governed one this build implements.
    DigestAlgorithm {
        /// The tag that was found.
        tag: u8,
    },
    /// A reserved framing field is not zero.
    ReservedNotZero,
    /// The declared total length is not the number of bytes present.
    TotalLength {
        /// Length the header declares.
        declared: u64,
        /// Length actually present.
        found: u64,
    },
    /// The bundle exceeds the configured maximum size.
    BundleTooLarge {
        /// Length the header declares.
        declared: u64,
        /// Configured maximum.
        limit: u64,
    },
    /// The embedded key is not the key this entry was requested under.
    ///
    /// A valid bundle at the wrong content path, which ADR 0050 makes a miss.
    KeyMismatch {
        /// Rendered key that was requested.
        requested: String,
        /// Rendered key the bundle embeds.
        embedded: String,
    },
    /// The declared section count exceeds the configured maximum.
    SectionCount {
        /// Count the header declares.
        declared: u64,
        /// Configured maximum.
        limit: u32,
    },
    /// A section's offset and length do not lie inside the bundle.
    SectionBounds {
        /// Zero-based descriptor index.
        index: usize,
        /// Declared offset.
        offset: u64,
        /// Declared length.
        length: u64,
    },
    /// A section does not begin where the previous one ended.
    SectionNotContiguous {
        /// Zero-based descriptor index.
        index: usize,
        /// Offset the canonical frame requires.
        expected: u64,
        /// Offset the descriptor declares.
        found: u64,
    },
    /// A section exceeds the configured maximum size.
    SectionTooLarge {
        /// Zero-based descriptor index.
        index: usize,
        /// Declared length.
        length: u64,
        /// Configured maximum.
        limit: u64,
    },
    /// A section declares a purpose this build does not implement.
    UnknownSectionPurpose {
        /// Zero-based descriptor index.
        index: usize,
        /// The tag that was found.
        tag: u32,
    },
    /// Two sections declare the same purpose.
    DuplicateSection {
        /// The purpose that appeared twice.
        purpose: BundleSection,
    },
    /// A required section is absent.
    MissingSection {
        /// The purpose that is missing.
        purpose: BundleSection,
    },
    /// A section's content does not match its declared digest.
    SectionDigest {
        /// The purpose whose digest failed.
        purpose: BundleSection,
    },
    /// Bytes follow the last framed section.
    TrailingBytes {
        /// Offset at which the last section ended.
        after: u64,
        /// Declared total length.
        total: u64,
    },
    /// The embedded key is not the key the carried subject derives.
    KeyNotDerivedFromSubject {
        /// Rendered key the bundle embeds.
        embedded: String,
        /// Rendered key the carried subject produces.
        derived: String,
    },
}

impl fmt::Display for BundleRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated { found, needed } => write!(
                formatter,
                "a cache bundle declares a structure needing {needed} bytes and has {found}",
            ),
            Self::Magic => formatter.write_str("the bytes do not open with a cache bundle's magic"),
            Self::Schema { major, minor } => write!(
                formatter,
                "cache bundle schema {major}.{minor} is not read by this build, which implements \
                 {}.{}",
                SCHEMA.0, SCHEMA.1,
            ),
            Self::DigestAlgorithm { tag } => write!(
                formatter,
                "cache bundle digest algorithm tag {tag:#04x} is not the governed `{}`",
                DigestAlgorithm::GOVERNED.governed_key(),
            ),
            Self::ReservedNotZero => {
                formatter.write_str("a reserved cache bundle framing field is not zero")
            }
            Self::TotalLength { declared, found } => write!(
                formatter,
                "a cache bundle declares {declared} total bytes and has {found}",
            ),
            Self::BundleTooLarge { declared, limit } => write!(
                formatter,
                "a cache bundle declares {declared} bytes, above the configured maximum {limit}",
            ),
            Self::KeyMismatch {
                requested,
                embedded,
            } => write!(
                formatter,
                "a cache bundle stored for key {requested} embeds key {embedded}",
            ),
            Self::SectionCount { declared, limit } => write!(
                formatter,
                "a cache bundle declares {declared} sections, above the configured maximum {limit}",
            ),
            Self::SectionBounds {
                index,
                offset,
                length,
            } => write!(
                formatter,
                "cache bundle section {index} spans offset {offset} length {length}, which is not \
                 inside the bundle",
            ),
            Self::SectionNotContiguous {
                index,
                expected,
                found,
            } => write!(
                formatter,
                "cache bundle section {index} begins at {found} rather than {expected}",
            ),
            Self::SectionTooLarge {
                index,
                length,
                limit,
            } => write!(
                formatter,
                "cache bundle section {index} declares {length} bytes, above the configured \
                 maximum {limit}",
            ),
            Self::UnknownSectionPurpose { index, tag } => write!(
                formatter,
                "cache bundle section {index} declares purpose {tag:#010x}, which this build does \
                 not implement",
            ),
            Self::DuplicateSection { purpose } => {
                write!(formatter, "a cache bundle frames `{purpose}` twice")
            }
            Self::MissingSection { purpose } => {
                write!(formatter, "a cache bundle does not frame `{purpose}`")
            }
            Self::SectionDigest { purpose } => write!(
                formatter,
                "cache bundle section `{purpose}` does not match its declared digest",
            ),
            Self::TrailingBytes { after, total } => write!(
                formatter,
                "a cache bundle's last section ends at {after} and it declares {total} bytes",
            ),
            Self::KeyNotDerivedFromSubject { embedded, derived } => write!(
                formatter,
                "a cache bundle embeds key {embedded} and carries a subject deriving {derived}",
            ),
        }
    }
}

impl std::error::Error for BundleRejection {}

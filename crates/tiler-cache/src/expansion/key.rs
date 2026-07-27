//! The complete compilation key one cache entry is stored under.

use core::fmt;

use tiler_artifact::program::{DIGEST_BYTES, Digest, DigestAlgorithm};

use super::subject::ComposedSubject;

/// Versioned domain separator of one expansion-cache key.
///
/// A digest under this domain can never equal a digest of the same bytes taken
/// for another purpose, which is what keeps a key derived here from colliding
/// with, say, an artifact's payload identity over the same source.
pub(super) const CACHE_KEY_DOMAIN: &[u8] = b"tiler.cache.expansion-key.v1\0";

/// Width, in lowercase hexadecimal characters, of one rendered cache key.
///
/// Fixed, because the path parser accepts exactly this width and no other. A
/// parser that accepted a shorter key and padded it, or a longer one and cut it,
/// would map two distinct compilations onto one entry.
pub const KEY_LABEL_BYTES: usize = DIGEST_BYTES * 2;

/// The content-addressed key of one cached compilation.
///
/// # What this crate proves about a key, and what it does not
///
/// **It proves the key is a function of the subject bytes it was derived from.**
/// [`Self::derive`] is the only constructor reachable outside this module, the
/// digest is the governed one, and a published bundle carries the subject
/// alongside the artifact so that every read re-derives the key from it and
/// refuses a bundle whose stored subject does not hash to the key it is filed
/// under.
///
/// **It proves the subject names every facet of the envelope, because it can no
/// longer be given anything else.** A conforming subject must determine *every
/// byte the bundle carries*, which is a whole artifact envelope and not only the
/// compiled object inside it — `docs/backends/metal.md` states the same
/// requirement from the other side, "full artifact identity is the key". This
/// derivation therefore takes a [`ComposedSubject`] rather than a byte run, and
/// that type is constructable only by naming both the backend compilations and
/// the artifact program wrapped around them. Two artifacts sharing source,
/// flags, and toolchain and differing in their plan portfolio are two facet sets,
/// two composed subjects, and two keys; they can no longer be filed as one.
///
/// **It does not prove a facet's bytes are that authority's real subject.**
/// Telling a genuine artifact-program subject from an invented one means parsing
/// an encoding this crate does not own, which is exactly what ADR 0082 rejected
/// for the digest. [`super::SubjectFacets`] states what the composition does and
/// does not cover, and names which facet is still unreachable and why.
///
/// **It does not prove the subject describes the carried artifact.** Even given a
/// complete subject, a bundle proves it was published under key `K` and that `K`
/// derives from the subject it carries; nothing here ties that subject to the
/// compilation the carried envelope records.
/// `bind-the-cache-subject-to-the-carried-payload-provenance` owns that gap, and
/// composing the subject does not close it — it makes it *reachable*, because the
/// composed frame is this crate's own and its facets can be counted without
/// parsing any producer's encoding.
///
/// **A key does not carry a reuse scope, and this crate does not enforce one.**
/// `tiler-metal-aot`'s evidence class bounds reuse of its identities to the host
/// that observed the toolchain, and it encodes that class *into* the subject
/// bytes, so two evidence classes already produce two keys. What no code here
/// can check is that a configured cache root is host-local; a caller that points
/// this cache at a shared volume defeats the producer's bound and this crate
/// will not notice.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CacheKey(Digest);

impl CacheKey {
    /// Derives the cache key of one composed subject.
    #[must_use]
    pub fn derive(subject: &ComposedSubject) -> Self {
        Self::derive_bytes(subject.as_bytes())
    }

    /// Derives the cache key of an already-composed byte run.
    ///
    /// Crate-private, and the asymmetry with [`Self::derive`] is the point. This
    /// is what the bundle decoder re-derives a stored key from, and it must
    /// accept bytes rather than a [`ComposedSubject`] because a subject read off
    /// disk is untrusted input: recomposing it would mean parsing it, and the
    /// check that gives the re-derivation its value is precisely that it hashes
    /// the exact bytes present without understanding them. Keeping it out of the
    /// public surface is what stops a caller from reaching past
    /// [`ComposedSubject`] and keying an entry on a bare producer subject.
    pub(crate) fn derive_bytes(subject: &[u8]) -> Self {
        Self(DigestAlgorithm::GOVERNED.digest(CACHE_KEY_DOMAIN, subject))
    }

    /// Returns the fixed-width lowercase hexadecimal rendering of this key.
    ///
    /// This is the exact text the namespace uses for its shard directory, its
    /// entry file, and its lock file.
    #[must_use]
    pub fn label(&self) -> String {
        self.0.label()
    }

    /// Returns the exact key bytes, as embedded in a bundle.
    pub(crate) const fn as_bytes(&self) -> &[u8; DIGEST_BYTES] {
        self.0.as_bytes()
    }

    /// Wraps key bytes read from a bundle being validated.
    ///
    /// A *claim* until it is compared with a key this crate derived, exactly as
    /// [`Digest::from_wire`] is for the digest it wraps.
    pub(crate) const fn from_wire(bytes: [u8; DIGEST_BYTES]) -> Self {
        Self(Digest::from_wire(bytes))
    }

    /// Parses one rendered key.
    ///
    /// Accepts exactly [`KEY_LABEL_BYTES`] lowercase hexadecimal characters.
    /// Uppercase is refused rather than folded, so the map from key to text is
    /// injective in both directions and two spellings of one key cannot name two
    /// entries. Nothing is truncated or padded to fit: a text of any other width
    /// is a rejection carrying the width that was found.
    pub(crate) fn parse_label(text: &str) -> Result<Self, KeyTextRejection> {
        if text.len() != KEY_LABEL_BYTES {
            return Err(KeyTextRejection::Width { found: text.len() });
        }
        let mut bytes = [0_u8; DIGEST_BYTES];
        // `text.len()` counts UTF-8 bytes, so a multi-byte character would make
        // the width check pass on a string with fewer characters. Iterating the
        // bytes rather than the characters is what keeps the two counts the
        // same: every accepted byte is one ASCII hexadecimal digit.
        for (index, slot) in bytes.iter_mut().enumerate() {
            let high = decode_nibble(text.as_bytes()[index * 2], index * 2)?;
            let low = decode_nibble(text.as_bytes()[index * 2 + 1], index * 2 + 1)?;
            *slot = (high << 4) | low;
        }
        Ok(Self::from_wire(bytes))
    }
}

/// Decodes one lowercase hexadecimal digit.
fn decode_nibble(byte: u8, position: usize) -> Result<u8, KeyTextRejection> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(KeyTextRejection::NotLowercaseHexadecimal { position, byte }),
    }
}

impl fmt::Display for CacheKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.label())
    }
}

/// Why a text is not the rendering of a cache key.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a rejection vocabulary a
/// caller forwards or partially classifies rather than maps totally.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum KeyTextRejection {
    /// The text is not exactly [`KEY_LABEL_BYTES`] bytes wide.
    Width {
        /// The width that was found, in bytes.
        found: usize,
    },
    /// A byte is not a lowercase hexadecimal digit.
    NotLowercaseHexadecimal {
        /// Zero-based byte position of the offending byte.
        position: usize,
        /// The byte that was found.
        byte: u8,
    },
}

impl fmt::Display for KeyTextRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Width { found } => write!(
                formatter,
                "a cache key is exactly {KEY_LABEL_BYTES} lowercase hexadecimal bytes wide, \
                 found {found}",
            ),
            Self::NotLowercaseHexadecimal { position, byte } => write!(
                formatter,
                "byte {position} of a cache key is {byte:#04x}, which is not a lowercase \
                 hexadecimal digit",
            ),
        }
    }
}

impl std::error::Error for KeyTextRejection {}

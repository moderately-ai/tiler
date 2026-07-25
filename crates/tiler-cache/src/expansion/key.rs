//! The complete compilation key one cache entry is stored under.

use core::fmt;

use tiler_artifact::program::{DIGEST_BYTES, Digest, DigestAlgorithm};

/// Versioned domain separator of one expansion-cache key.
///
/// A digest under this domain can never equal a digest of the same bytes taken
/// for another purpose, which is what keeps a key derived here from colliding
/// with, say, an artifact's payload identity over the same source.
const CACHE_KEY_DOMAIN: &[u8] = b"tiler.cache.expansion-key.v1\0";

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
/// **It does not prove the subject is complete, and completeness here is a
/// stronger requirement than it first looks.** A conforming subject must
/// determine *every byte the bundle carries*, which is a whole artifact
/// envelope and not only the compiled object inside it. `docs/backends/metal.md`
/// states the same requirement from the other side — "full artifact identity is
/// the key" — and the reason is exactly the failure ADR 0050's complete-identity
/// clause exists to exclude: two artifacts that share source, flags, and
/// toolchain but differ in their plan variants, ABI bindings, or routing are
/// different artifacts, and a subject naming only the compilation would file
/// them under one key and serve either for the other.
///
/// `crates/tiler-metal-aot/src/identity.rs` is therefore *half* of a conforming
/// subject, not the whole of one. It is the good half — it determines the
/// `metallib` by a mechanism rather than by vigilance, because its request and
/// toolchain records are destructured irrefutably, so a new input fails to
/// compile until it reaches the subject — but it says nothing about the artifact
/// program that carries the object. **No component emits the composed subject as
/// one canonical byte run today**, and this crate deliberately does not invent
/// one: it cannot compose a subject without becoming an authority over encodings
/// it does not own. `compose-the-complete-expansion-cache-subject` owns closing
/// that, and until it does, a caller passing the driver's subject alone is
/// under-keying and this crate cannot detect it.
///
/// **It does not prove the subject describes the carried artifact.** Even given a
/// complete subject, a bundle proves it was published under key `K` and that `K`
/// derives from the subject it carries; nothing here ties that subject to the
/// compilation the carried envelope records, because doing so would require this
/// crate to parse the producer's subject encoding.
/// `bind-the-cache-subject-to-the-carried-payload-provenance` owns that gap.
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
    /// Derives the cache key of one canonical compilation subject.
    #[must_use]
    pub fn derive(subject: &[u8]) -> Self {
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

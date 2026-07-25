//! The governed content-digest algorithm of the artifact envelope.
//!
//! `docs/artifact-abi.md` requires every digest use to name an explicit
//! governed algorithm and domain separator, and forbids a parser from inferring
//! the algorithm from a digest width. The envelope therefore carries an
//! algorithm tag in its fixed header, and this module is the only place that
//! maps that tag to an implementation.
//!
//! # Why the implementation is local
//!
//! The workspace has no cryptographic dependency and the research contract
//! records the production algorithm choice as an open bounded decision
//! (`tiler.research.artifacts.target-neutral-envelope`, "Remaining bounded
//! decisions"). Depending on an external hash crate would answer that decision
//! by accident. The wire contract commits only to the governed tag, so swapping
//! this implementation for an audited crate is an internal change that leaves
//! every encoded envelope byte identical. The ticket
//! `select-the-governed-artifact-digest-implementation` owns that comparison.
//!
//! The implementation is FIPS 180-4 SHA-256 and is pinned by the standard
//! published test vectors plus the three message lengths that exercise every
//! padding branch.

use std::fmt;

/// Byte width of one governed artifact digest.
pub(crate) const DIGEST_BYTES: usize = 32;

/// The governed digest algorithm one envelope was written with.
///
/// Deliberately **not** `#[non_exhaustive]` (ADR 0074 convention 5b): every
/// consumer that maps this vocabulary maps it *totally* — a wildcard arm would
/// have to invent a hash function — so a second admitted algorithm must be a
/// compile error at every such site rather than a silently wrong digest.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) enum DigestAlgorithm {
    /// FIPS 180-4 SHA-256, governed as `tiler.digest.sha-256.v1`.
    Sha256,
}

impl DigestAlgorithm {
    /// The algorithm this build of the crate writes.
    pub(crate) const GOVERNED: Self = Self::Sha256;

    /// Returns the governed wire tag of this algorithm.
    pub(crate) const fn tag(self) -> u8 {
        match self {
            Self::Sha256 => 0x01,
        }
    }

    /// Returns the governed algorithm key, for diagnostics and explain output.
    pub(crate) const fn governed_key(self) -> &'static str {
        match self {
            Self::Sha256 => "tiler.digest.sha-256.v1",
        }
    }

    /// Resolves a governed wire tag, or `None` for an unrecognized algorithm.
    pub(crate) const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::Sha256),
            _ => None,
        }
    }

    /// Digests `bytes` under an explicit domain separator.
    pub(crate) fn digest(self, domain: &[u8], bytes: &[u8]) -> Digest {
        self.digest_parts(&[domain, bytes])
    }

    /// Digests the concatenation of `parts` in order.
    ///
    /// The caller owns unambiguity. Every use here opens with a governed domain
    /// separator and follows it with fixed-width qualifiers before any
    /// variable-length run, so no two distinct part sequences can produce one
    /// pre-image.
    pub(crate) fn digest_parts(self, parts: &[&[u8]]) -> Digest {
        match self {
            Self::Sha256 => {
                let mut state = Sha256::new();
                for part in parts {
                    state.update(part);
                }
                Digest(state.finish())
            }
        }
    }
}

impl fmt::Display for DigestAlgorithm {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.governed_key())
    }
}

/// Opaque fixed-width content digest of one exact byte run.
///
/// The bytes are derived by [`DigestAlgorithm::digest`] alone; there is no
/// public constructor, so no caller can assemble a digest naming bytes that
/// were never hashed (ADR 0074 convention 2).
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct Digest([u8; DIGEST_BYTES]);

impl Digest {
    /// Wraps digest bytes read from an envelope being validated.
    ///
    /// The result is a *claim* until it is compared with a digest this crate
    /// derived; decoding never treats a read digest as evidence on its own.
    pub(crate) const fn from_wire(bytes: [u8; DIGEST_BYTES]) -> Self {
        Self(bytes)
    }

    /// Returns the exact digest bytes.
    pub(crate) const fn as_bytes(&self) -> &[u8; DIGEST_BYTES] {
        &self.0
    }

    /// Returns the lowercase hexadecimal rendering, for diagnostics and fixtures.
    pub(crate) fn label(&self) -> String {
        let mut rendered = String::with_capacity(DIGEST_BYTES * 2);
        for byte in self.0 {
            rendered.push(char::from_digit(u32::from(byte >> 4), 16).expect("nibble is hex"));
            rendered.push(char::from_digit(u32::from(byte & 0x0f), 16).expect("nibble is hex"));
        }
        rendered
    }
}

impl fmt::Debug for Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("Digest")
            .field(&self.label())
            .finish()
    }
}

const ROUND_CONSTANTS: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

const INITIAL_STATE: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

const BLOCK_BYTES: usize = 64;

// Positions of the eight working variables of FIPS 180-4 section 6.2.2. They
// are held as one array so a round's shift is a rotation rather than eight
// separate single-letter bindings, and these names keep each read matching the
// specification it transcribes.
const A: usize = 0;
const B: usize = 1;
const C: usize = 2;
const E: usize = 4;
const F: usize = 5;
const G: usize = 6;
const H: usize = 7;

/// Streaming FIPS 180-4 SHA-256 state.
struct Sha256 {
    state: [u32; 8],
    buffer: [u8; BLOCK_BYTES],
    buffered: usize,
    message_bytes: u64,
}

impl Sha256 {
    const fn new() -> Self {
        Self {
            state: INITIAL_STATE,
            buffer: [0; BLOCK_BYTES],
            buffered: 0,
            message_bytes: 0,
        }
    }

    fn update(&mut self, mut bytes: &[u8]) {
        self.message_bytes = self
            .message_bytes
            .checked_add(u64::try_from(bytes.len()).expect("supported usize fits u64"))
            .expect("a digested message stays within the 64-bit length field");
        if self.buffered > 0 {
            let wanted = BLOCK_BYTES - self.buffered;
            let taken = wanted.min(bytes.len());
            self.buffer[self.buffered..self.buffered + taken].copy_from_slice(&bytes[..taken]);
            self.buffered += taken;
            bytes = &bytes[taken..];
            if self.buffered == BLOCK_BYTES {
                let block = self.buffer;
                self.compress(&block);
                self.buffered = 0;
            }
            // The partial block was not completed, so there is nothing left to
            // absorb. Falling through would rebuffer an empty remainder over
            // the count just accumulated and silently discard it.
            if bytes.is_empty() {
                return;
            }
        }
        debug_assert_eq!(
            self.buffered, 0,
            "a partial block is either completed and flushed or consumes the whole input",
        );
        let (blocks, rest) = bytes.as_chunks::<BLOCK_BYTES>();
        for block in blocks {
            self.compress(block);
        }
        self.buffer[..rest.len()].copy_from_slice(rest);
        self.buffered = rest.len();
    }

    fn finish(mut self) -> [u8; DIGEST_BYTES] {
        let bit_length = self
            .message_bytes
            .checked_mul(8)
            .expect("a digested message stays within the 64-bit bit-length field");
        self.update_without_length(&[0x80]);
        while self.buffered != BLOCK_BYTES - 8 {
            self.update_without_length(&[0x00]);
        }
        self.update_without_length(&bit_length.to_be_bytes());
        debug_assert_eq!(self.buffered, 0, "padding completes the final block");
        let mut digest = [0_u8; DIGEST_BYTES];
        let (words, _) = digest.as_chunks_mut::<4>();
        for (word, chunk) in self.state.iter().zip(words) {
            *chunk = word.to_be_bytes();
        }
        digest
    }

    /// Absorbs padding bytes, which are outside the declared message length.
    fn update_without_length(&mut self, bytes: &[u8]) {
        let recorded = self.message_bytes;
        self.update(bytes);
        self.message_bytes = recorded;
    }

    fn compress(&mut self, block: &[u8; BLOCK_BYTES]) {
        let mut schedule = [0_u32; 64];
        let (words, _) = block.as_chunks::<4>();
        for (slot, word) in schedule.iter_mut().zip(words) {
            *slot = u32::from_be_bytes(*word);
        }
        for index in 16..64 {
            let previous = schedule[index - 15];
            let ahead = schedule[index - 2];
            let s0 = previous.rotate_right(7) ^ previous.rotate_right(18) ^ (previous >> 3);
            let s1 = ahead.rotate_right(17) ^ ahead.rotate_right(19) ^ (ahead >> 10);
            schedule[index] = schedule[index - 16]
                .wrapping_add(s0)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(s1);
        }
        let mut working = self.state;
        for (word, constant) in schedule.iter().zip(ROUND_CONSTANTS) {
            let sigma1 = working[E].rotate_right(6)
                ^ working[E].rotate_right(11)
                ^ working[E].rotate_right(25);
            let choose = (working[E] & working[F]) ^ (!working[E] & working[G]);
            let temp1 = working[H]
                .wrapping_add(sigma1)
                .wrapping_add(choose)
                .wrapping_add(constant)
                .wrapping_add(*word);
            let sigma0 = working[A].rotate_right(2)
                ^ working[A].rotate_right(13)
                ^ working[A].rotate_right(22);
            let majority =
                (working[A] & working[B]) ^ (working[A] & working[C]) ^ (working[B] & working[C]);
            let temp2 = sigma0.wrapping_add(majority);
            // Shifting each variable one place forward is the rotation; only
            // the two positions the round redefines are then written.
            working.rotate_right(1);
            working[E] = working[E].wrapping_add(temp1);
            working[A] = temp1.wrapping_add(temp2);
        }
        for (held, added) in self.state.iter_mut().zip(working) {
            *held = held.wrapping_add(added);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DigestAlgorithm, Sha256};

    fn hex(bytes: &[u8]) -> String {
        let mut state = Sha256::new();
        state.update(bytes);
        super::Digest(state.finish()).label()
    }

    /// The FIPS 180-4 published vectors, including the one-million-character case.
    #[test]
    fn matches_the_published_sha_256_vectors() {
        assert_eq!(
            hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        );
        assert_eq!(
            hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        );
        assert_eq!(
            hex(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
        );
        assert_eq!(
            hex(&vec![b'a'; 1_000_000]),
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
        );
    }

    /// Padding has three branches; these lengths take each one.
    #[test]
    fn every_padding_branch_agrees_with_a_single_shot_digest() {
        for length in [54_usize, 55, 56, 63, 64, 65, 119, 120, 127, 128] {
            let message = vec![0x5a_u8; length];
            let mut split = Sha256::new();
            split.update(&message[..length / 2]);
            split.update(&message[length / 2..]);
            assert_eq!(
                super::Digest(split.finish()).label(),
                hex(&message),
                "chunked and single-shot digests disagree at length {length}",
            );
        }
    }

    #[test]
    fn the_domain_separator_changes_the_digest() {
        let algorithm = DigestAlgorithm::GOVERNED;
        assert_ne!(
            algorithm.digest(b"tiler.a\0", b"payload"),
            algorithm.digest(b"tiler.b\0", b"payload"),
        );
    }

    /// The separator is a prefix, so separation rests on the domains themselves.
    ///
    /// `digest(domain, body)` hashes `domain || body`, which distinguishes two
    /// subjects only when no admitted domain is a prefix of another — otherwise
    /// a longer domain and a shorter one with leading body bytes would collide.
    /// Every governed domain is a fixed constant of this crate, so the property
    /// is checkable here rather than assumed, and a new domain that violates it
    /// fails this test instead of silently merging two subjects.
    #[test]
    fn no_governed_domain_is_a_prefix_of_another() {
        let domains = [
            super::super::encode::MANIFEST_DIGEST_DOMAIN,
            super::super::encode::SECTION_DIGEST_DOMAIN,
            super::super::encode::ENVELOPE_DIGEST_DOMAIN,
        ];
        for (index, left) in domains.iter().enumerate() {
            for right in domains.iter().skip(index + 1) {
                assert!(
                    !left.starts_with(right) && !right.starts_with(left),
                    "one governed digest domain prefixes another",
                );
            }
        }
    }

    #[test]
    fn the_governed_tag_round_trips() {
        let algorithm = DigestAlgorithm::GOVERNED;
        assert_eq!(DigestAlgorithm::from_tag(algorithm.tag()), Some(algorithm));
        assert_eq!(DigestAlgorithm::from_tag(0x00), None);
        assert_eq!(DigestAlgorithm::from_tag(0xff), None);
    }
}

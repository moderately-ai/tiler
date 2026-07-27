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
//! The implementation is FIPS 180-4 SHA-256. It is pinned by the standard
//! published test vectors, by the message lengths that exercise every padding
//! branch, and by a digest over every message length `0..=192` compared against
//! a value Python's `hashlib` produced. The last of those is the only one that
//! is *independent* evidence: the other two compare this implementation with
//! itself or with a constant a reader cannot re-derive, so a consistently wrong
//! padding rule satisfies them.
//!
//! # Why this module is public
//!
//! Being the *only* place that maps the governed tag to an implementation is
//! the whole point, and that property is what a second component reaching for a
//! hash function would destroy. ADR 0050 requires the expansion cache to
//! validate a stored bundle's section digests on every hit; Tom decided on
//! 2026-07-25 that the cache is a dedicated crate reaching this algorithm rather
//! than owning one. [`DigestAlgorithm`] and [`Digest`] are therefore public.
//!
//! What stays private is as deliberate as what does not. `digest_parts` is
//! crate-private because its documented contract puts unambiguity on the caller,
//! and every use inside this crate discharges it with a governed domain followed
//! by fixed-width qualifiers. An outside caller gets [`DigestAlgorithm::digest`],
//! which takes exactly one domain and one run, and therefore cannot express the
//! ambiguous concatenation.

use std::fmt;

/// Byte width of one governed artifact digest.
pub const DIGEST_BYTES: usize = 32;

/// The governed digest algorithm one envelope was written with.
///
/// Deliberately **not** `#[non_exhaustive]` (ADR 0074 convention 5b): every
/// consumer that maps this vocabulary maps it *totally* — a wildcard arm would
/// have to invent a hash function — so a second admitted algorithm must be a
/// compile error at every such site rather than a silently wrong digest.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DigestAlgorithm {
    /// FIPS 180-4 SHA-256, governed as `tiler.digest.sha-256.v1`.
    Sha256,
}

impl DigestAlgorithm {
    /// The algorithm this build of the crate writes.
    pub const GOVERNED: Self = Self::Sha256;

    /// Returns the governed wire tag of this algorithm.
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::Sha256 => 0x01,
        }
    }

    /// Returns the governed algorithm key, for diagnostics and explain output.
    #[must_use]
    pub const fn governed_key(self) -> &'static str {
        match self {
            Self::Sha256 => "tiler.digest.sha-256.v1",
        }
    }

    /// Resolves a governed wire tag, or `None` for an unrecognized algorithm.
    #[must_use]
    pub const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0x01 => Some(Self::Sha256),
            _ => None,
        }
    }

    /// Digests `bytes` under an explicit domain separator.
    ///
    /// The domain is a prefix, so separation rests on no admitted domain being
    /// a prefix of another. Every domain this crate writes is one of its own
    /// constants and the property is checked over them; a caller outside this
    /// crate that introduces a domain owns the same obligation for its own set.
    #[must_use]
    pub fn digest(self, domain: &[u8], bytes: &[u8]) -> Digest {
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
pub struct Digest([u8; DIGEST_BYTES]);

impl Digest {
    /// Wraps digest bytes read from an envelope being validated.
    ///
    /// The result is a *claim* until it is compared with a digest this crate
    /// derived; decoding never treats a read digest as evidence on its own.
    #[must_use]
    pub const fn from_wire(bytes: [u8; DIGEST_BYTES]) -> Self {
        Self(bytes)
    }

    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; DIGEST_BYTES] {
        &self.0
    }

    /// Returns the lowercase hexadecimal rendering, for diagnostics and fixtures.
    ///
    /// Indexed from a sixteen-character table rather than converted, so the
    /// four-bit value that selects a character cannot be out of range and there
    /// is no unreachable failure to document or to handle.
    #[must_use]
    pub fn label(&self) -> String {
        const DIGITS: [u8; 16] = *b"0123456789abcdef";
        let mut rendered = String::with_capacity(DIGEST_BYTES * 2);
        for byte in self.0 {
            rendered.push(char::from(DIGITS[usize::from(byte >> 4)]));
            rendered.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
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

/// Longest padding [`Sha256::finish`] can append: the `0x80` byte, up to 63
/// zeros, and the eight-byte length. Reached when a block holds 63 bytes.
const MAX_PADDING_BYTES: usize = BLOCK_BYTES + 8;

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
        // FIPS 180-4 section 5.1.1 padding, assembled once and absorbed in one
        // call. Appending it a byte at a time re-entered `update` up to 64
        // times per digest, and each entry saved and restored the message
        // length, ran a `checked_add` and a `u64::try_from`, and split a
        // one-byte slice into chunks. That is amortised to nothing over an
        // 18 KB manifest, but `tiler-cache` digests nothing larger than 256
        // bytes — 966 of its 1,546 measured calls are under 64 bytes, a single
        // block — so there the padding was a per-digest fixed cost comparable
        // to the one compression it brackets.
        //
        // The zero count is the fewest that leaves the eight-byte length ending
        // exactly on a block boundary: `buffered + 1 + zeros ≡ 56 (mod 64)`,
        // so `zeros ≡ 55 - buffered`, taken modulo 64 to stay non-negative for
        // every `buffered` in `0..64`.
        let zeros = (2 * BLOCK_BYTES - 9 - self.buffered) % BLOCK_BYTES;
        let mut padding = [0_u8; MAX_PADDING_BYTES];
        padding[0] = 0x80;
        let length_at = 1 + zeros;
        padding[length_at..length_at + 8].copy_from_slice(&bit_length.to_be_bytes());
        self.update_without_length(&padding[..length_at + 8]);
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

    #[allow(
        clippy::many_single_char_names,
        reason = "a through h are the names FIPS 180-4 section 6.2.2 gives these eight working \
                  variables; renaming them would make each round stop matching the \
                  specification line it transcribes"
    )]
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
        // The eight working variables of FIPS 180-4 section 6.2.2, held as
        // separate bindings rather than an array.
        //
        // **This is not a style choice, and the array form it replaces was a
        // measured 69% of digest time.** Writing the round's shift as
        // `working.rotate_right(1)` on a `[u32; 8]` reads as a rotation, but an
        // array has no inherent `rotate_right`: the call derefs to the *slice*
        // method, which is the generic gcd-juggling `ptr::copy` routine. The
        // compiler never const-folded the length or the midpoint, so a release
        // build emitted it out of line with a 320-byte stack frame and three
        // calls to `_platform_memmove` — invoked 64 times per 64-byte block, on
        // the innermost loop of every hash in the workspace. A profile of an
        // 18 KB digest attributed 57.7% of active samples to `_platform_memmove`
        // and 11.0% to `<[u32]>::rotate_right`; throughput was 53 MiB/s.
        //
        // The trap is that the two spellings look identical three lines apart.
        // `e.rotate_right(6)` below is `u32::rotate_right`, a single `ror`
        // instruction, and is exactly what this code wants. `working
        // .rotate_right(1)` on the array was a libc memmove. Same method name,
        // same file, entirely different cost — so the receiver type is what has
        // to be read, and eight bindings remove the ambiguity by construction.
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        for (word, constant) in schedule.iter().zip(ROUND_CONSTANTS) {
            let sigma1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choose = (e & f) ^ (!e & g);
            let temp1 = h
                .wrapping_add(sigma1)
                .wrapping_add(choose)
                .wrapping_add(constant)
                .wrapping_add(*word);
            let sigma0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = sigma0.wrapping_add(majority);
            // Each variable shifts one place forward; only the two the round
            // redefines take a new value. These are register renames.
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        for (held, added) in self.state.iter_mut().zip([a, b, c, d, e, f, g, h]) {
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

    /// Every message length that reaches a distinct padding residue agrees with
    /// an independently computed digest.
    ///
    /// **The neighbouring padding test cannot catch what this one is for.** It
    /// compares a chunked digest with a single-shot digest, so both sides run
    /// *this* implementation and a padding rule that is consistently wrong
    /// satisfies it. The expected value here was produced by Python's
    /// `hashlib`, which shares no code with this module, so it is external
    /// evidence rather than a self-consistency check.
    ///
    /// The sweep is exhaustive rather than sampled because the padding length
    /// is computed modulo the block size: it is a function of `buffered`, whose
    /// domain is exactly `0..64`, and 0..=192 covers every residue three times
    /// over — including the two-block case where the length field is pushed
    /// into a following block. A sampled sweep would leave residues untested
    /// while reporting success, which is the failure mode the digest-of-digests
    /// form is chosen to avoid: one changed byte at any length changes the
    /// pinned value.
    #[test]
    fn every_padding_residue_matches_an_independent_implementation() {
        let algorithm = DigestAlgorithm::GOVERNED;
        let mut outer = Sha256::new();
        for length in 0..=192_usize {
            let message = vec![0x5a_u8; length];
            outer.update(algorithm.digest(b"m\0", &message).as_bytes());
        }
        assert_eq!(
            super::Digest(outer.finish()).label(),
            "2f4c4d8de88a0f18ec22cfbdd365ce45ec57c0ecf63936a8cf98a18ca24c156a",
            "a digest over every message length 0..=192 disagrees with the value \
             Python's hashlib produces for the same sequence",
        );
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
    /// is checkable rather than assumed, and a new domain that violates it fails
    /// a test instead of silently merging two subjects.
    ///
    /// **This test covers the envelope's three domains and not the crate's
    /// seven.** The property is global: one algorithm hashes both the envelope
    /// and the proof sidecar in one process, so a domain added to either
    /// container could collide with one in the other, and a check confined to
    /// three of the seven would report separation it had not established.
    /// `crate::proof::tests::no_governed_domain_of_either_container_prefixes_another`
    /// checks the union and is the authority for the property; this test is the
    /// envelope-local half. A fourth envelope domain must be added to **both**,
    /// and `docs/artifact-abi.md` records the union obligation normatively.
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

    /// The two message lengths this workspace actually digests.
    ///
    /// **These are measured, not chosen.** Instrumenting [`digest_parts`] to
    /// print its total input length and running the two suites that reach it
    /// gave 17,914 calls from `tiler-artifact` and 1,546 from `tiler-cache`,
    /// and the two populations barely overlap:
    ///
    /// | population | calls | modal length |
    /// | --- | --- | --- |
    /// | `tiler-artifact` manifest digest | 8,697 | 18,013 B |
    /// | `tiler-artifact` section digest | 8,398 | 8,122 B |
    /// | `tiler-cache` subject and bundle keys | 1,546 | 36-167 B, none above 256 B |
    ///
    /// So the artifact path is bulk compression — 95% of its calls exceed 4 KB
    /// — and the cache path is *entirely* one- and two-block messages, where
    /// the per-digest fixed cost of padding and finalization is comparable to
    /// the single compression it brackets. A change that helps one says nothing
    /// about the other, which is why both lengths are reported here.
    ///
    /// One figure this corrects: the manifest is **18,013 bytes**, not the
    /// 25,000 that `codec/tests.rs` records in
    /// `single_byte_corruptions_are_rejected`. The 26 KB that other tickets
    /// cite is the *envelope*, which the same instrumentation saw at 26,169
    /// bytes; the manifest is its interior and is smaller.
    const MEASURED_LENGTHS: [(usize, &str); 3] = [
        (36, "tiler-cache subject key"),
        (8_122, "tiler-artifact section digest"),
        (18_013, "tiler-artifact manifest digest"),
    ];

    /// Reports digest throughput at the measured message lengths.
    ///
    /// This test asserts nothing about time, exactly as
    /// `tiler-compiler`'s `hot_path` module does and for the same reason: a
    /// timing assertion fails on a loaded machine and passes on a fast one,
    /// which makes it a flake rather than a guard. What it is for is a
    /// reproducible number to read before and after a change.
    ///
    /// **Report the minimum, not the mean.** Every perturbation a host applies
    /// makes a run *slower* and none makes it faster, so the distribution has a
    /// hard floor at the true cost and an unbounded tail of noise. The minimum
    /// of enough repeats estimates the floor; the mean estimates the floor plus
    /// whatever else the machine was doing.
    ///
    /// Release matters — workspace crates build at `opt-level = 0` by default:
    ///
    /// ```text
    /// cargo nextest run --release -p tiler-artifact -E 'test(digest_throughput)' --no-capture
    /// ```
    #[test]
    fn digest_throughput_by_message_length() {
        use std::time::{Duration, Instant};

        for (length, population) in MEASURED_LENGTHS {
            let message = vec![0x5a_u8; length];
            // Enough repeats that the minimum is a floor rather than a lucky
            // sample, scaled so every length costs about the same wall time.
            let repeats = (2_000_000 / length).max(64);
            for _ in 0..repeats / 8 {
                std::hint::black_box(DigestAlgorithm::GOVERNED.digest(b"m\0", &message));
            }
            let mut best = Duration::MAX;
            for _ in 0..repeats {
                let start = Instant::now();
                std::hint::black_box(DigestAlgorithm::GOVERNED.digest(b"m\0", &message));
                best = best.min(start.elapsed());
            }
            let bytes = f64::from(u32::try_from(length).expect("a measured length fits u32"));
            println!(
                "MEASURE digest {length} B ({population}): min {best:?} over {repeats}, \
                 {:.0} MiB/s",
                bytes / best.as_secs_f64() / (1024.0 * 1024.0),
            );
        }
    }

    /// Digests in a loop long enough for a sampling profiler to attribute cost.
    ///
    /// `#[ignore]`d because it deliberately runs for seconds and asserts
    /// nothing. Record it with `samply`:
    ///
    /// ```text
    /// CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release --tests -p tiler-artifact
    /// TILER_PROFILE_SECONDS=20 samply record --save-only --unstable-presymbolicate \
    ///     --rate 4000 -o digest.profile.json.gz \
    ///     -- target/release/deps/tiler_artifact-<hash> \
    ///        --ignored --exact program::codec::digest::tests::digest_profile_loop --nocapture
    /// ```
    ///
    /// `CARGO_PROFILE_RELEASE_DEBUG=true` is required — the release profile
    /// carries no debug information and without it every frame symbolicates to
    /// a bare hex address. `--unstable-presymbolicate` writes the `*.syms.json`
    /// sidecar holding the names; the profile's own string table does not.
    #[test]
    #[ignore = "runs for seconds under a profiler; not part of the gate"]
    fn digest_profile_loop() {
        use std::time::{Duration, Instant};

        let seconds: u64 = std::env::var("TILER_PROFILE_SECONDS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(10);
        // The manifest length dominates the byte volume the workspace hashes.
        let message = vec![0x5a_u8; 18_013];
        let deadline = Instant::now() + Duration::from_secs(seconds);
        let mut digests = 0_u64;
        while Instant::now() < deadline {
            for _ in 0..256 {
                std::hint::black_box(DigestAlgorithm::GOVERNED.digest(b"m\0", &message));
            }
            digests += 256;
        }
        println!(
            "MEASURE profile loop: {digests} digests of {} B",
            message.len()
        );
    }
}

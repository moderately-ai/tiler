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
//! The wire contract governs FIPS 180-4 SHA-256 as `tiler.digest.sha-256.v1` with tag `0x01`; the envelope, manifest, section, and sidecar domains are likewise governed constants. `select-the-governed-artifact-digest-implementation` separately measured the implementation choice and adopted `sha2` 0.11.0. Keeping that implementation behind this module leaves the wire algorithm and every encoded envelope byte independent of the dependency that computes them.
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

use sha2::Digest as _;

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
                let mut state = sha2::Sha256::new();
                for part in parts {
                    state.update(part);
                }
                Digest(state.finalize().into())
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

/// SHA-256's block size, which the framing this module writes is stated against.
const BLOCK_BYTES: usize = 64;

#[cfg(test)]
mod tests {
    use super::DigestAlgorithm;

    /// Digests one byte run through the crate's own entry point.
    ///
    /// Deliberately `digest_parts` and not the internals of whatever
    /// implements SHA-256: these cases pin the bytes this module *publishes*,
    /// which is the contract artifact identity rests on, and they must keep
    /// meaning the same thing when the implementation behind it is replaced.
    fn hex(bytes: &[u8]) -> String {
        DigestAlgorithm::GOVERNED.digest_parts(&[bytes]).label()
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
            let split = DigestAlgorithm::GOVERNED
                .digest_parts(&[&message[..length / 2], &message[length / 2..]]);
            assert_eq!(
                split.label(),
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
        let mut accumulated = Vec::new();
        for length in 0..=192_usize {
            let message = vec![0x5a_u8; length];
            accumulated.extend_from_slice(algorithm.digest(b"m\0", &message).as_bytes());
        }
        assert_eq!(
            algorithm.digest_parts(&[&accumulated]).label(),
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

#![doc(test(attr(forbid(unsafe_code))))]
//! The one governed content-digest algorithm every Tiler identity hashes under.
//!
//! `docs/artifact-abi.md` requires every digest use to name an explicit governed
//! algorithm and domain separator, and forbids a parser from inferring the
//! algorithm from a digest width. The artifact envelope therefore carries an
//! algorithm tag in its fixed header, and this crate is the only place that maps
//! that tag to an implementation.
//!
//! # Why the implementation is local
//!
//! The wire contract governs FIPS 180-4 SHA-256 as `tiler.digest.sha-256.v1` with tag `0x01`; every domain separator a consumer hashes under is likewise a governed constant. `select-the-governed-artifact-digest-implementation` separately measured the implementation choice and adopted `sha2` 0.11.0. Keeping that implementation behind this crate leaves the wire algorithm and every encoded envelope byte independent of the dependency that computes them.
//!
//! The implementation is FIPS 180-4 SHA-256. It is pinned by the standard
//! published test vectors, by the message lengths that exercise every padding
//! branch, and by a digest over every message length `0..=192` compared against
//! a value Python's `hashlib` produced. The last of those is the only one that
//! is *independent* evidence: the other two compare this implementation with
//! itself or with a constant a reader cannot re-derive, so a consistently wrong
//! padding rule satisfies them.
//!
//! # Why this is a crate, and why it is the bottom one
//!
//! Being the *only* place that maps the governed tag to an implementation is the
//! whole point, and that property is what a second component reaching for a hash
//! function would destroy. The property used to be held by a private module of
//! `tiler-artifact`, which is where the first consumer happened to need it; ADR
//! 0050 then required the expansion cache to validate a stored bundle's section
//! digests on every hit, and Tom decided on 2026-07-25 that the cache is a
//! dedicated crate reaching this algorithm rather than owning one (ADR 0082).
//! That resolved a reachability problem by moving the *consumer*.
//!
//! [ADR 0104](../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md)
//! reached the case where that move is impossible. `tiler-ir` mints
//! `IndexRefinementExecutableCoverageIdentity` and folds the bound graph's
//! identity into it as a digest, and `tiler-artifact` is built on `tiler-ir` —
//! so `tiler-ir` cannot be relocated above `tiler-artifact`, and reversing the
//! edge is not available either. Tom decided on 2026-08-06 that the governed
//! digest is its own crate below both, on the grounds that hashing is a separate
//! responsibility from tensor IR, that the one-authority rule deserves a
//! structural home rather than riding in whichever crate needed it first, and
//! that a future layered-identity consumer should reach it without reopening the
//! boundary. `tiler-artifact` re-exports [`DigestAlgorithm`], [`Digest`], and
//! [`DIGEST_BYTES`] from `tiler_artifact::program`, so every path a consumer
//! already used still resolves.
//!
//! # What the public surface admits, and what it refuses to express
//!
//! What is absent is as deliberate as what is not. There is no entry point that
//! digests an arbitrary sequence of parts, because such a call puts unambiguity
//! entirely on the caller and a concatenation of variable-length runs has no
//! unambiguous reading. The two shapes every governed use in this workspace
//! actually takes carry that obligation in their signatures instead:
//! [`DigestAlgorithm::digest`] takes exactly one domain and one run, and
//! [`DigestAlgorithm::digest_qualified`] takes one domain, fixed-width
//! qualifiers, and exactly one trailing variable-length run. Both put every
//! variable-length byte in one place a reader can find, so neither can express
//! the ambiguous concatenation. `tiler-artifact` carried a `pub(crate)`
//! parts-digest for its section and sidecar-payload subjects while it owned this
//! module; both are qualified digests and say so now, and the general form is
//! gone rather than promoted across a crate boundary.
//!
//! # Two result subjects, and why the type system separates them
//!
//! [ADR 0111](../../../docs/decisions/0111-separate-externally-specified-raw-hashes-from-governed-tiler-digests.md)
//! admits a second *subject* here without admitting a second algorithm. Some
//! evidence this workspace compares against was digested by an outside
//! authority — the L3 realization probe's host handed a result buffer to
//! `CC_SHA256` and recorded the lowercase result — and reproducing that record
//! means hashing exactly the bytes it hashed, with no Tiler domain in front of
//! them. Prefixing a `tiler.*` domain would ask a different question and
//! invalidate every retained comparison.
//!
//! The tempting spelling, `GOVERNED.digest(b"", bytes)`, is refused for two
//! independent reasons. It publishes the empty byte string as an ordinary
//! domain on an API whose governed subjects all carry a real one, which is the
//! discipline that entry point exists to hold. And [`DigestAlgorithm::GOVERNED`]
//! means *the algorithm this build writes*, while the retained record means
//! SHA-256 permanently — so a future writer-policy change would silently
//! reinterpret old evidence.
//!
//! [`DigestAlgorithm::digest_external_record`] therefore returns
//! [`ExternalDigest`], which is a different type from [`Digest`] and converts to
//! it in neither direction. The two travel through the same private
//! implementation dispatch and diverge only at the result, so there is one SHA
//! implementation in this workspace and still no way to hand a raw external
//! reproduction to an API documented to carry governed Tiler content.
//!
//! # The domain-separation discipline
//!
//! A domain is a prefix of the pre-image, so separation rests on no admitted
//! domain being a prefix of another — otherwise a longer domain and a shorter
//! one followed by leading body bytes would collide. The property is global
//! rather than per container: one algorithm hashes the artifact envelope, the
//! proof sidecar, and the shared IR's layered identities in one process, so a
//! domain admitted anywhere could collide with one admitted anywhere else, and a
//! check confined to one container reports a separation it has not established.
//!
//! This crate cannot hold that check, because it deliberately knows none of the
//! domains: a domain belongs to the authority that decides what it names. What
//! each such authority owes is the check over the set it admits, plus the
//! argument that its set cannot prefix another's.
//! `tiler_artifact::domains::no_governed_domain_of_this_crate_prefixes_another`
//! is the authority for that crate's whole admitted set, which spans the
//! envelope, the proof sidecar, and the artifact program's identity and key
//! encodings. `tiler_artifact::domains::GovernedDomain` is that population and
//! is what sizes it. **This note deliberately states no count.** A number
//! written here would be maintained by hand beside a population maintained by a
//! type, so the two could only ever disagree, and a later reader would have no
//! way to tell which one was authoritative. `docs/artifact-abi.md` records the
//! obligation and the per-container split normatively.

use std::fmt;

use sha2::Digest as _;

/// Byte width of one governed content digest.
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
    /// The algorithm this build of the workspace writes.
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
    /// a prefix of another. Every caller that introduces a domain owns that
    /// obligation for its own set; see the crate documentation for why this
    /// crate cannot discharge it.
    #[must_use]
    pub fn digest(self, domain: &[u8], bytes: &[u8]) -> Digest {
        self.digest_qualified(domain, &[], bytes)
    }

    /// Digests `bytes` under a domain and a run of fixed-width qualifiers.
    ///
    /// The pre-image is the domain, then each qualifier in order, then `bytes`.
    /// **Every qualifier must be fixed width**, which is what makes the
    /// pre-image unambiguous without a length prefix between the qualifiers and
    /// the content: the number of bytes each one occupies is a property of the
    /// subject rather than of this call, so no two distinct qualifier tuples can
    /// produce one pre-image. Exactly one variable-length run is admitted and it
    /// is the last, which is the part the signature holds rather than the
    /// caller.
    ///
    /// This is what a content address over a *qualified* subject needs — a
    /// section digest binds its purpose and content schema, and a sidecar
    /// payload digest binds its canonical ordinal — so that the digest still
    /// distinguishes two subjects once it is lifted out of the container that
    /// distinguished them positionally.
    #[must_use]
    pub fn digest_qualified(self, domain: &[u8], qualifiers: &[&[u8]], bytes: &[u8]) -> Digest {
        Digest(self.compress(domain, qualifiers, bytes))
    }

    /// Reproduces an externally specified raw digest record over `bytes`.
    ///
    /// The pre-image is `bytes` and nothing else: no domain, no qualifier, no
    /// framing. That is what the subject requires — an outside authority
    /// digested exactly these bytes and published the result, and reproducing
    /// its record means asking its question rather than a Tiler one.
    ///
    /// **Name the variant the external record names.** The retained `CC_SHA256`
    /// corpus this workspace compares against says SHA-256, so its callers spell
    /// [`DigestAlgorithm::Sha256`]. [`DigestAlgorithm::GOVERNED`] is *not* that
    /// authority even while it aliases the same variant: it means the algorithm
    /// this build of Tiler writes, so calling through it would let a future
    /// writer-policy change silently reinterpret evidence an outside record
    /// already fixed. Spelling the variant makes that case a compile error
    /// instead — the exact algorithm either remains available or the caller
    /// stops building.
    ///
    /// The result is an [`ExternalDigest`] rather than a [`Digest`] because it
    /// is not a Tiler identity: it names no governed subject, carries no domain,
    /// and may not flow into an API documented to carry governed content.
    #[must_use]
    pub fn digest_external_record(self, bytes: &[u8]) -> ExternalDigest {
        ExternalDigest(self.compress(&[], &[], bytes))
    }

    /// The one implementation dispatch both public result paths run.
    ///
    /// Private because the pre-image it accepts is unconstrained, and an
    /// unconstrained pre-image is exactly what the public surface refuses to
    /// express: a caller that could pass any domain, any qualifiers, and any
    /// body would carry the unambiguity obligation the two governed shapes hold
    /// in their signatures. Each public entry point discharges that obligation
    /// before reaching here — [`Self::digest_qualified`] by requiring
    /// fixed-width qualifiers ahead of one trailing run, and
    /// [`Self::digest_external_record`] by admitting no prefix at all.
    fn compress(self, domain: &[u8], qualifiers: &[&[u8]], bytes: &[u8]) -> [u8; DIGEST_BYTES] {
        match self {
            Self::Sha256 => {
                let mut state = sha2::Sha256::new();
                state.update(domain);
                for qualifier in qualifiers {
                    state.update(qualifier);
                }
                state.update(bytes);
                state.finalize().into()
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
/// The bytes are derived by [`DigestAlgorithm::digest`] and
/// [`DigestAlgorithm::digest_qualified`] alone; there is no public constructor,
/// so no caller can assemble a digest naming bytes that were never hashed
/// (ADR 0074 convention 2).
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Digest([u8; DIGEST_BYTES]);

impl Digest {
    /// Wraps digest bytes read from an envelope being validated.
    ///
    /// The result is a *claim* until it is compared with a digest this workspace
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
    #[must_use]
    pub fn label(&self) -> String {
        label_of(&self.0)
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

/// Opaque fixed-width reproduction of an externally specified raw digest record.
///
/// Derived by [`DigestAlgorithm::digest_external_record`] alone. There is no
/// public constructor, so no caller can assemble one naming bytes that were
/// never hashed — the same rule [`Digest`] holds (ADR 0074 convention 2).
///
/// # What this type refuses, and why the refusals are the type
///
/// This is *not* a Tiler identity. It reproduces bytes an outside authority
/// digested and published; its provenance and trustworthiness are that
/// authority's responsibility, and nothing here makes the record authentic.
/// [ADR 0111](../../../docs/decisions/0111-separate-externally-specified-raw-hashes-from-governed-tiler-digests.md)
/// therefore keeps the two subject classes mutually exclusive in the public type
/// system rather than in prose, and the whole mechanism is what is *absent*:
/// there is no `from_wire`, no `From` or `Into` in either direction, no
/// cross-type comparison, and no serialization pairing it with [`Digest`].
///
/// A missing conversion is only a boundary if it stays missing, so each refusal
/// is compiled. There is no wire constructor (`E0599`):
///
/// ```compile_fail,E0599
/// use tiler_digest::ExternalDigest;
/// let claimed = ExternalDigest::from_wire([0_u8; 32]);
/// ```
///
/// The tuple field is private, so the type cannot be built directly either
/// (`E0423`):
///
/// ```compile_fail,E0423
/// use tiler_digest::ExternalDigest;
/// let claimed = ExternalDigest([0_u8; 32]);
/// ```
///
/// A governed digest does not convert into one (`E0277`):
///
/// ```compile_fail,E0277
/// use tiler_digest::{DigestAlgorithm, ExternalDigest};
/// let governed = DigestAlgorithm::GOVERNED.digest(b"tiler.a\0", b"payload");
/// let external: ExternalDigest = governed.into();
/// ```
///
/// Nor does one convert into a governed digest (`E0277`):
///
/// ```compile_fail,E0277
/// use tiler_digest::{Digest, DigestAlgorithm};
/// let external = DigestAlgorithm::Sha256.digest_external_record(b"payload");
/// let governed: Digest = external.into();
/// ```
///
/// The two cannot be compared, so a retained external record can never be
/// mistaken for agreement with a governed subject (`E0308` — [`Digest`]
/// implements `PartialEq` only against itself, so the right operand is a type
/// error rather than a missing-trait one):
///
/// ```compile_fail,E0308
/// use tiler_digest::DigestAlgorithm;
/// let governed = DigestAlgorithm::GOVERNED.digest(b"tiler.a\0", b"payload");
/// let external = DigestAlgorithm::Sha256.digest_external_record(b"payload");
/// let _ = governed == external;
/// ```
///
/// And the external entry point cannot be bound where a governed digest is
/// expected (`E0308`):
///
/// ```compile_fail,E0308
/// use tiler_digest::{Digest, DigestAlgorithm};
/// let governed: Digest = DigestAlgorithm::Sha256.digest_external_record(b"payload");
/// ```
///
/// What the type *does* admit is the observation an evidence consumer needs —
/// exact bytes and a lowercase label — and comparison with another reproduction
/// of the same kind:
///
/// ```
/// use tiler_digest::DigestAlgorithm;
/// let external = DigestAlgorithm::Sha256.digest_external_record(b"abc");
/// assert_eq!(
///     external.label(),
///     "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
/// );
/// assert_eq!(external, DigestAlgorithm::Sha256.digest_external_record(b"abc"));
/// assert_eq!(external.as_bytes().len(), tiler_digest::DIGEST_BYTES);
/// ```
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ExternalDigest([u8; DIGEST_BYTES]);

impl ExternalDigest {
    /// Returns the exact reproduced bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; DIGEST_BYTES] {
        &self.0
    }

    /// Returns the lowercase hexadecimal rendering.
    ///
    /// Lowercase because that is the spelling the external records this
    /// reproduces are written in, so a retained string compares directly.
    #[must_use]
    pub fn label(&self) -> String {
        label_of(&self.0)
    }
}

impl fmt::Debug for ExternalDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ExternalDigest")
            .field(&self.label())
            .finish()
    }
}

/// Renders digest bytes as lowercase hexadecimal.
///
/// Indexed from a sixteen-character table rather than converted, so the four-bit
/// value that selects a character cannot be out of range and there is no
/// unreachable failure to document or to handle.
///
/// Shared by both result types deliberately. The rendering is a property of the
/// width rather than of the subject, and two copies could only ever differ —
/// which, for [`ExternalDigest`], would mean a retained comparison failing on
/// case or padding rather than on the bytes it is about.
fn label_of(bytes: &[u8; DIGEST_BYTES]) -> String {
    const DIGITS: [u8; 16] = *b"0123456789abcdef";
    let mut rendered = String::with_capacity(DIGEST_BYTES * 2);
    for byte in bytes {
        rendered.push(char::from(DIGITS[usize::from(byte >> 4)]));
        rendered.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::{DIGEST_BYTES, DigestAlgorithm, ExternalDigest};

    /// Reproduces one externally specified raw record, as a lowercase label.
    ///
    /// **This is the entry point a published FIPS vector belongs on**, and it
    /// moved here when [`ExternalDigest`] was admitted. A published vector is
    /// precisely an externally specified raw digest over an exact byte run: it
    /// carries no Tiler domain and its algorithm is fixed by the publisher
    /// rather than by this build's writer policy. Comparing it used to require
    /// spelling the empty domain on the governed entry point — the one
    /// convention ADR 0111 refuses to publish — so the case that most needed a
    /// raw path was the one arguing hardest for an empty-domain habit.
    ///
    /// Deliberately the crate's own entry point and not the internals of
    /// whatever implements SHA-256: these cases pin the bytes this crate
    /// *publishes*, and they must keep meaning the same thing when the
    /// implementation behind them is replaced.
    fn hex(bytes: &[u8]) -> String {
        DigestAlgorithm::Sha256
            .digest_external_record(bytes)
            .label()
    }

    /// Digests one byte run under the empty domain, unqualified.
    ///
    /// Retained only for the self-consistency cases below, whose subject is the
    /// *governed* pre-image assembly rather than the algorithm's published
    /// output. Every governed subject in the workspace carries a real domain;
    /// the empty one here is a test fixture and not a convention this crate
    /// offers.
    fn governed_hex(bytes: &[u8]) -> String {
        DigestAlgorithm::GOVERNED.digest(b"", bytes).label()
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

    /// The external path is the same implementation, not a second one.
    ///
    /// **This is the property that keeps the workspace at one SHA authority.**
    /// The two entry points return types that deliberately cannot be compared,
    /// so nothing else in the workspace can observe that they agree — and if
    /// they were ever backed by different code, every consumer would keep
    /// passing while the crate quietly held two algorithms. Comparing the raw
    /// bytes is the one place that difference is visible, which is why it is
    /// asserted inside the crate that owns both.
    ///
    /// The pre-images coincide because a governed digest under the empty domain
    /// with no qualifiers is the bare byte run, which is exactly what an
    /// external record's pre-image is. That coincidence is what makes the
    /// comparison meaningful and is *not* an invitation to spell it at a call
    /// site: the empty domain reaches the governed entry point only from this
    /// module's fixtures.
    #[test]
    fn the_external_path_and_the_governed_path_share_one_implementation() {
        for message in [b"".as_slice(), b"abc", &[0x5a_u8; 200]] {
            assert_eq!(
                DigestAlgorithm::Sha256
                    .digest_external_record(message)
                    .as_bytes(),
                DigestAlgorithm::GOVERNED.digest(b"", message).as_bytes(),
                "the external and governed paths disagree on a {} byte message, so this crate \
                 is running two SHA implementations rather than one",
                message.len(),
            );
        }
    }

    /// An external reproduction is fixed width and lowercase hexadecimal.
    ///
    /// The retained records this reproduces are lowercase, fixed-width strings,
    /// so a comparison against one is only sound if the rendering matches on
    /// both counts. Asserted rather than assumed because a mismatch here would
    /// fail every retained comparison at once and read as a device defect.
    #[test]
    fn an_external_reproduction_renders_at_the_retained_width_and_case() {
        let external: ExternalDigest = DigestAlgorithm::Sha256.digest_external_record(b"abc");
        let label = external.label();
        assert_eq!(label.len(), DIGEST_BYTES * 2);
        assert!(
            label
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()),
            "{label}",
        );
        assert_eq!(
            format!("{external:?}"),
            format!("ExternalDigest(\"{label}\")")
        );
    }

    /// Distinct byte runs reproduce distinct external records.
    ///
    /// The trivial property, and the one that fails loudly if the external path
    /// ever stopped reaching its argument — a reproduction that ignored its
    /// bytes would satisfy nothing else here except by accident.
    #[test]
    fn an_external_reproduction_depends_on_its_bytes() {
        assert_ne!(
            DigestAlgorithm::Sha256.digest_external_record(b"payload"),
            DigestAlgorithm::Sha256.digest_external_record(b"payloae"),
        );
    }

    /// Padding has three branches; these lengths take each one.
    #[test]
    fn every_padding_branch_agrees_with_a_single_shot_digest() {
        for length in [54_usize, 55, 56, 63, 64, 65, 119, 120, 127, 128] {
            let message = vec![0x5a_u8; length];
            let split = DigestAlgorithm::GOVERNED.digest_qualified(
                b"",
                &[&message[..length / 2]],
                &message[length / 2..],
            );
            assert_eq!(
                split.label(),
                governed_hex(&message),
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
    /// `hashlib`, which shares no code with this crate, so it is external
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
            algorithm.digest(b"", &accumulated).label(),
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

    /// The two public entry points agree where their subjects coincide.
    ///
    /// [`DigestAlgorithm::digest`] is *defined* through
    /// [`DigestAlgorithm::digest_qualified`], so this holds the definition
    /// rather than restating it: an unqualified subject and a qualified one
    /// with no qualifiers are the same pre-image, and every governed digest in
    /// the workspace is one or the other. A future implementation that framed
    /// the qualifier run — a count, a length, anything at all — would move every
    /// unqualified digest ever taken, and this is the case that says so.
    #[test]
    fn a_qualified_digest_with_no_qualifiers_is_the_plain_digest() {
        let algorithm = DigestAlgorithm::GOVERNED;
        assert_eq!(
            algorithm.digest_qualified(b"tiler.a\0", &[], b"payload"),
            algorithm.digest(b"tiler.a\0", b"payload"),
        );
    }

    /// A qualifier separates two subjects that share a domain and a body.
    ///
    /// This is the property the qualified form exists for: a section digest
    /// binds its purpose so that two sections of different purposes carrying
    /// equal bytes do not share one content address once the digest is lifted
    /// out of the envelope that distinguished them positionally.
    #[test]
    fn a_qualifier_separates_two_subjects_sharing_a_domain_and_a_body() {
        let algorithm = DigestAlgorithm::GOVERNED;
        assert_ne!(
            algorithm.digest_qualified(b"tiler.a\0", &[&[0x01]], b"payload"),
            algorithm.digest_qualified(b"tiler.a\0", &[&[0x02]], b"payload"),
        );
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
    /// **These are measured, not chosen.** Instrumenting the algorithm to print
    /// its total input length and running the two suites that reach it
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
    /// One conflation this heads off: **18,013 bytes** is the *manifest*, while
    /// the ~26 KB other records cite is the *envelope* whose interior it is —
    /// the same instrumentation saw that envelope at 26,169 bytes. Both are
    /// from that one run and both have since shrunk: `tiler-artifact`'s
    /// `single_byte_corruptions_are_rejected` records the envelope reaching
    /// 15,030 bytes in the codec work of 2026-07-27.
    ///
    /// The counts predate [ADR 0104](../../../docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md),
    /// which added one short call per coverage record from `tiler-ir` and made
    /// every artifact-side message shorter. The lengths are kept because they
    /// are what the throughput report is comparable against; the third
    /// population `tiler-ir` now contributes is one 400-to-1,200-byte graph
    /// identity per record, which sits inside the cache path's regime rather
    /// than opening a new one.
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
    /// cargo nextest run --release -p tiler-digest -E 'test(digest_throughput)' --no-capture
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
    /// CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release --tests -p tiler-digest
    /// TILER_PROFILE_SECONDS=20 samply record --save-only --unstable-presymbolicate \
    ///     --rate 4000 -o digest.profile.json.gz \
    ///     -- target/release/deps/tiler_digest-<hash> \
    ///        --ignored --exact tests::digest_profile_loop --nocapture
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

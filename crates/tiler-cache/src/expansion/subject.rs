//! The composed subject one cache key is derived from.
//!
//! # Why a composed subject exists at all
//!
//! A bundle carries a whole artifact envelope, so a conforming subject must
//! determine every byte of that envelope. `docs/backends/metal.md` states the
//! same requirement from the other side — "full artifact identity is the key",
//! and "full identity from canonical plans, MSL, target, SDK, compiler, linker,
//! flags, and numerical realization".
//!
//! No single authority holds all of that. `tiler-metal-aot` owns the inputs that
//! determine the compiled object and cannot name the artifact program, because
//! ADR 0082 item 1 keeps its dependency closure empty. `tiler-artifact` owns the
//! envelope encoding and knows nothing about the compilation. Before this
//! module, the two halves were never joined: [`super::ExpansionCache`] took an
//! opaque byte run and a caller passing the driver's compilation subject alone
//! was under-keying, so two artifacts agreeing on source, flags, and toolchain
//! and differing in their plan portfolio hashed to one key and the cache served
//! either for the other.
//!
//! # Composing is not interpreting
//!
//! This crate still owns no producer encoding, and composing one does not make
//! it an authority over any. A facet's bytes are opaque here: they are counted,
//! length-prefixed, tagged with the role they fill, and never parsed. What this
//! module owns is the *frame* — which roles exist, in which order, and how their
//! runs are delimited — which is exactly the part no producer can own, because
//! no producer can see the others.
//!
//! Consequently [`tiler-metal-aot`'s subject is **wrapped**, not
//! subsumed](https://github.com/moderately-ai/tiler/blob/main/crates/tiler-metal-aot/src/identity.rs):
//! its exact bytes appear as one run of the [`SubjectFacet::BackendCompilations`]
//! facet. Subsuming it would mean restating its encoding here, which is the
//! second-authority failure ADR 0082 rejected for the digest and `family.rs`
//! rejects for its own selection bytes. Wrapping also preserves its `SameHost`
//! reuse bound with no work: that crate encodes its toolchain evidence class
//! *into* the subject bytes, those bytes travel through this frame unchanged, so
//! two evidence classes still produce two composed subjects and two keys.
//!
//! # What a composed subject determines, and what it does not
//!
//! **It determines the facet set.** [`SubjectFacets`] is destructured
//! irrefutably by [`ComposedSubject::compose`] and the facet table is sized by
//! [`SubjectFacet::ORDER`], so a facet added in either direction fails to compile
//! until it reaches the bytes. That is the same mechanism `identity.rs` uses, and
//! it is a claim about *roles*, not about their contents.
//!
//! **It does not determine that a facet's bytes are the authority's real
//! subject.** Nothing here can distinguish a genuine artifact-program subject
//! from one byte a caller invented, because telling them apart means parsing an
//! encoding this crate does not own. What it can do — and does — is require every
//! facet to be named and non-empty, so an omission is a typed refusal instead of
//! a shorter key nobody notices. Completeness *within* a facet is the supplying
//! authority's obligation, discharged by that authority's own mechanism.
//!
//! **No producer exists yet for [`SubjectFacet::ArtifactProgram`].** The artifact
//! layer derives `CanonicalArtifactProgramIdentity` from a *verified* artifact,
//! which needs the payload digest and therefore the compiled bytes — and the key
//! is needed on a miss, before compilation. A pre-compilation subject over the
//! plan portfolio, ABI, routing, target requirements, and selected providers is
//! what this facet is for, and deriving one is
//! `derive-the-pre-compilation-artifact-program-subject`. Until that lands, every
//! caller must supply the facet and none can supply it correctly, which is a loud
//! stop rather than a silent under-key.

use core::fmt;

/// Versioned domain tag opening every composed subject.
///
/// ADR 0074 convention 3. It is what keeps a bare producer subject from ever
/// equalling a composed one: a caller that reached past this module and handed
/// a raw compilation subject to the key derivation would produce bytes that
/// cannot open with this tag, so the two can never name one entry.
pub(super) const COMPOSED_SUBJECT_DOMAIN: &[u8] = b"tiler.cache.composed-subject.v1\0";

/// One role a composed subject frames the bytes of.
///
/// Deliberately **not** `#[non_exhaustive]` (ADR 0074 convention 5b): the
/// composer maps this vocabulary totally onto a wire tag, and a wildcard arm
/// there could only invent a tag the variant alone determines.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SubjectFacet {
    /// The canonical compilation subjects of the backend payloads the envelope
    /// carries, in the envelope's own payload order.
    ///
    /// A *sequence* rather than one run, because one artifact may carry several
    /// compiled payloads — `tiler_artifact::program::MAX_ARTIFACT_PAYLOADS` is
    /// sixteen, and a selection naming three Apple families is three
    /// compilations producing three independently identified payloads. A facet
    /// naming one compilation would under-key every multi-family artifact.
    BackendCompilations,
    /// The canonical subject of the artifact program wrapped around those
    /// payloads: its plan portfolio, ABI bindings, routing, declared target
    /// requirements, and selected capability providers.
    ArtifactProgram,
}

impl SubjectFacet {
    /// The facets a composed subject frames, in the exact order it frames them.
    ///
    /// The composer's facet table is sized by this array, so adding a variant
    /// here fails to compile until it is given bytes.
    const ORDER: [Self; 2] = [Self::BackendCompilations, Self::ArtifactProgram];

    /// Returns this facet's stable wire tag.
    ///
    /// An arm that states its constant, never a discriminant read from
    /// declaration order (ADR 0074 convention 3).
    const fn tag(self) -> u32 {
        match self {
            Self::BackendCompilations => 0x0000_0001,
            Self::ArtifactProgram => 0x0000_0002,
        }
    }

    /// Returns this facet's stable lowercase identifier, for diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BackendCompilations => "backend-compilations",
            Self::ArtifactProgram => "artifact-program",
        }
    }
}

impl fmt::Display for SubjectFacet {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The canonical bytes each authority contributes to one composed subject.
///
/// A caller-constructed leaf value record, so its fields are visible
/// (ADR 0074 convention 6). Every field is required: there is no builder, no
/// `Default`, and no optional facet, because an omitted facet is precisely the
/// under-keying this type exists to prevent.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SubjectFacets<'bytes> {
    /// The compilation subjects of the envelope's backend payloads, in the
    /// envelope's payload order.
    ///
    /// Order is part of identity because it is part of the envelope: two
    /// artifacts whose payload descriptors are permutations of one another are
    /// different byte runs, so a subject that ignored order would file them
    /// under one key.
    pub backend_compilations: &'bytes [&'bytes [u8]],
    /// The artifact program's canonical subject.
    pub artifact_program: &'bytes [u8],
}

/// The canonical byte run one cache key is the governed digest of.
///
/// A *derived* identity in the sense of ADR 0074 convention 2: its storage is
/// private, [`Self::compose`] is its only constructor, and [`Self::as_bytes`] is
/// its only reader. No caller can assemble one naming facets nobody framed.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ComposedSubject {
    bytes: Vec<u8>,
}

impl ComposedSubject {
    /// Composes one canonical subject from every authority's canonical bytes.
    ///
    /// # Errors
    ///
    /// Returns [`SubjectRefusal::NoRuns`] when a facet contributes nothing and
    /// [`SubjectRefusal::EmptyRun`] when one of its runs is empty. Both are the
    /// same defect seen from two sides — a caller that had no subject for a
    /// facet and supplied a placeholder — and refusing is what keeps it from
    /// becoming a key that silently names less than the envelope it will file.
    pub fn compose(facets: &SubjectFacets<'_>) -> Result<Self, SubjectRefusal> {
        // Irrefutable, so a field added to `SubjectFacets` is a compile error
        // here rather than an input that silently leaves identity. This is the
        // mechanism; the table below is only what conforms today.
        let SubjectFacets {
            backend_compilations,
            artifact_program,
        } = facets;

        let program = [*artifact_program];
        let table: [(SubjectFacet, &[&[u8]]); SubjectFacet::ORDER.len()] = [
            (SubjectFacet::BackendCompilations, backend_compilations),
            (SubjectFacet::ArtifactProgram, &program),
        ];
        debug_assert_eq!(
            table.map(|(facet, _)| facet),
            SubjectFacet::ORDER,
            "the composer writes facets in the canonical order it declares",
        );

        for (facet, runs) in table {
            // A facet requires at least one run. For `ArtifactProgram` the table
            // above makes that structural; for `BackendCompilations` it is a
            // decision: the expansion cache exists to spare an external
            // compilation (ADR 0050), so an envelope produced by no compilation
            // is not a case this cache admits, and refusing it explicitly is
            // preferable to keying it under a subject with nothing in that
            // facet.
            if runs.is_empty() {
                return Err(SubjectRefusal::NoRuns { facet });
            }
            for (index, run) in runs.iter().enumerate() {
                if run.is_empty() {
                    return Err(SubjectRefusal::EmptyRun { facet, index });
                }
            }
        }

        let mut bytes = Vec::new();
        bytes.extend_from_slice(COMPOSED_SUBJECT_DOMAIN);
        // The facet count leads the frame even though the domain version already
        // fixes it, so a build reading these bytes learns how many facets they
        // claim before indexing any, and so a facet added under one domain
        // version moves every byte that follows.
        push_count(&mut bytes, SubjectFacet::ORDER.len());
        for (facet, runs) in table {
            bytes.extend_from_slice(&facet.tag().to_be_bytes());
            push_count(&mut bytes, runs.len());
            for run in runs {
                push_run(&mut bytes, run);
            }
        }
        Ok(Self { bytes })
    }

    /// Returns the exact canonical bytes this subject was composed into.
    ///
    /// The caller that owns the governed digest hashes these; this type
    /// deliberately computes no digest of its own, so the key stays a function
    /// of one algorithm under one domain.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Writes a fixed-width big-endian count before a repeated run.
///
/// This crate's sole copy of the workspace's canonical length framing, and the
/// sole one it is permitted. `tiler_ir::identity` owns that framing, and a
/// composed subject is a genuine identity preimage rather than a container
/// field — but ADR 0082 item 2 decides this crate's closure is exactly
/// `tiler-artifact` and says in terms that `tiler-ir` is "an edge this record
/// decides the crate does not have", so the framing cannot be imported here and
/// has to be restated. A gate once admitted exactly this definition and
/// [`push_run`] beside it, so a second copy in this crate failed rather than
/// growing quietly; `e197176` deleted that gate along with the rest of the
/// Python tooling and gave it no successor. **A third copy appearing here is
/// now caught only by review of the diff that adds it.**
///
/// `u64` is wide enough for every sequence a 64-bit host can address, so no
/// real subject can be rejected or truncated here. What makes the conversion
/// total is the supported-platform policy — `AGENTS.md` states Tiler develops
/// on macOS only, and every admitted target is 64-bit — rather than a check:
/// the gate that once asserted it is gone.
fn push_count(bytes: &mut Vec<u8>, count: usize) {
    let count = u64::try_from(count).expect("the admitted profiles have a 64-bit address space");
    bytes.extend_from_slice(&count.to_be_bytes());
}

/// Writes one length-prefixed run.
///
/// Admitted alongside [`push_count`] under the same ADR 0082 item 2 closure.
///
/// The prefix is what keeps adjacent facets from being re-split: without it, a
/// compilation subject ending in the head of an artifact-program subject and a
/// shorter one followed by a longer program subject would concatenate to the
/// same bytes and file two different envelopes under one key.
fn push_run(bytes: &mut Vec<u8>, run: &[u8]) {
    push_count(bytes, run.len());
    bytes.extend_from_slice(run);
}

/// Why a set of facets is not a composable subject.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a rejection vocabulary a
/// caller forwards or partially classifies rather than maps totally.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SubjectRefusal {
    /// A facet contributed no runs at all.
    NoRuns {
        /// The facet that was left empty.
        facet: SubjectFacet,
    },
    /// One of a facet's runs is empty.
    ///
    /// Every canonical subject in this workspace opens with its own versioned
    /// domain tag, so no authority produces zero bytes. An empty run is a caller
    /// that had nothing to supply, which is the omission a composed subject
    /// exists to make impossible.
    EmptyRun {
        /// The facet the run belongs to.
        facet: SubjectFacet,
        /// Zero-based position of the run within that facet.
        index: usize,
    },
}

impl fmt::Display for SubjectRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRuns { facet } => write!(
                formatter,
                "the `{facet}` facet of a composed cache subject carries no canonical bytes",
            ),
            Self::EmptyRun { facet, index } => write!(
                formatter,
                "run {index} of the `{facet}` facet of a composed cache subject is empty, and no \
                 canonical subject is zero bytes wide",
            ),
        }
    }
}

impl std::error::Error for SubjectRefusal {}

//! Governed keys and opaque identities received at the artifact boundary.
//!
//! Two shapes appear here and they are deliberately different (ADR 0074 §2).
//!
//! A **governed key** is bounded UTF-8 text this crate compares and encodes as
//! meaning: which backend family, which executable representation, which
//! declared target profile, which feasibility rule set, which capability, which
//! backend-scoped route requirement. It has a validating constructor because a
//! producer legally names one.
//!
//! **Corrected 2026-08-08; the list was true when written.** It is one-to-one
//! with the `governed_key!` invocations below and was exact at `d5b63819`
//! (2026-07-24), when both were written. It then decayed twice with neither
//! change touching it: `d1a95e18` (2026-07-25) moved `TargetPropertyKey` to
//! `tiler_ir::program::abi`, for the reason recorded in the comment where that
//! key used to be declared, and `d715d5da` (2026-07-31) added
//! [`RouteFeatureKey`] without extending the sentence. The retired sixth item
//! read "which target property", quoted so it stays greppable — a later hit for
//! that phrase lands in this note rather than in a live claim.
//!
//! An **opaque identity** is bytes another authority derived — a payload
//! content digest, a backend entry key, a target-profile descriptor digest.
//! This crate treats them as opaque, never re-derives them, and offers only a
//! wrapping constructor. That constructor is a statement that this crate is not
//! the authority for that subject.
//!
//! Neither shape is the artifact's own derived identity;
//! [`super::CanonicalArtifactProgramIdentity`] is derived only by this crate's
//! encoder and has no public constructor.
//!
//! # A governed key is spelled in one alphabet
//!
//! Being the authority for a governed key settles its spelling as well as its
//! bound. Every one of these keys exists to be compared byte for byte, and the
//! comparisons join keys minted by producers that never met: a payload's
//! backend and representation pair against a loading host's, a packaged profile
//! against the profile a host declares, a route feature against the one its
//! owning backend answers. So [`validate_key`] admits ASCII lowercase, ASCII
//! digits, `.`, `-`, and `_`, and refuses every other byte with
//! [`ArtifactBuildError::NoncanonicalKeyByte`].
//!
//! What that forbids is what a byte comparison cannot express and a reader
//! cannot reproduce. Case would leave `tiler.Metal` and `tiler.metal` two
//! backend families that every reader sees as one and every comparison sees as
//! two. A space, a control byte, or a NUL would put a key into a rejection, an
//! explain output, or a trace in a form nobody can copy back — which defeats
//! the point of failing closed with an explainable error, since the explanation
//! is the part a consumer acts on.
//!
//! `tiler_compiler::target::TargetProfileKey` admits exactly this alphabet, for
//! this reason, and the two are deliberately equal so that every profile key
//! that compiler mints is packageable here. Neither crate depends on the other,
//! so widening either alphabet requires checking both.
//!
//! # The byte bounds deliberately do not reconcile with the alphabet
//!
//! [`MAX_GOVERNED_KEY_BYTES`] is 256 while `tiler_compiler::target`'s
//! `MAX_TARGET_PROFILE_KEY_BYTES` is 128, and that difference stays. An
//! alphabet is part of what a governed key *is*, so every producer owes it. A
//! byte bound is a resource ceiling, and this layer sets its own because key
//! minting is open ([ADR 0090](../../../../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md)
//! item 10) and a producer that is not that compiler is not bounded by that
//! compiler's number — the same reasoning as the section below, one step
//! further out.
//!
//! The direction is what makes the difference safe rather than a gap. The
//! smaller number is the *minting* bound, so every key that compiler can name
//! fits here; a key between the two is one this layer holds and that compiler
//! could not have minted, which is a fact about which producer named it rather
//! than a value anything mishandles. Were the inequality the other way, a
//! legally minted key would be unpackageable.
//!
//! # An opaque identity's bound belongs to whoever mints it
//!
//! Not being the authority for a subject settles its byte bound too: the number
//! that admits every value a producer can legally mint is the producer's own,
//! and any other number is this crate deciding something it just said it does
//! not decide. So each identity below names the bound of the authority that
//! derives it rather than sharing one bound chosen for their common *shape* —
//! `super::codec::budget`'s rule, applied one level out.
//!
//! The three do not share a subject and never did. A [`PayloadDigest`] is a
//! fixed-width digest under the governed algorithm. A [`BackendEntryKey`] is a
//! `tiler_ir::kernel::CanonicalKernelIdentity`, a canonical *encoding* of a
//! whole structured kernel, and it is bounded by
//! `tiler_ir::kernel::MAX_KERNEL_IDENTITY_BYTES` — the exact constant that
//! crate enforces when it mints one. A [`TargetProfileDescriptorDigest`] is a
//! digest in name only:
//! `tiler_compiler::feasibility` records that its bytes *are* the descriptor
//! identity rather than a hash of it, and it is under this crate's own
//! [`MAX_TARGET_PROFILE_DESCRIPTOR_BYTES`], which its `opaque_identity!`
//! invocation below passes as the limit and which `super::tests`'s
//! `an_opaque_identity_takes_the_bound_of_the_authority_that_mints_it` asserts.
//! That constant is this layer's **admission ceiling rather than a claim about
//! profiles**: `tiler_compiler::feasibility` holds the matching governing mint
//! bound under the same name, `pub(crate)` there where this one is `pub`, and
//! refuses where a descriptor is minted, so a governed producer cannot reach
//! this bound and a value that does reach it was not minted by one. This crate
//! still refuses it, because it validates what it is handed rather than
//! trusting where it came from.
//!
//! **Corrected 2026-08-08.** Two clauses above were wrong in different ways, so
//! they are treated differently.
//!
//! The bound was *true when written*, and is dated rather than substituted.
//! `22616630` (2026-07-27) wrote that this identity "is under
//! [`MAX_OPAQUE_IDENTITY_BYTES`]" while the `opaque_identity!` invocation below
//! genuinely passed that constant. It went stale at `0b7e59d3` (2026-07-30),
//! which gave the descriptor its own [`MAX_TARGET_PROFILE_DESCRIPTOR_BYTES`] in
//! this file and moved the invocation, the constant's own doc, and the type's
//! doc onto it — and left this paragraph behind.
//!
//! The visibility clause was *never* true as a reader would take it, and is
//! substituted with its wording quoted. `fe6d3a87` (2026-08-01) rewrote this
//! same sentence to say `tiler_compiler` "bounds a descriptor with its
//! crate-private `MAX_TARGET_PROFILE_DESCRIPTOR_BYTES`". That is literally true
//! of the compiler's `pub(crate)` constant and reads as though the compiler is
//! the only holder of one, when this file had declared its own `pub` constant
//! of that name two days earlier. A later hit for that quoted phrase lands in
//! this note rather than in a live claim.
//!
//! Both clauses sat beneath the heading above, which states the opposite rule.
//! `docs/artifact-abi.md` described the ceiling correctly throughout, so here
//! the contract outranked the source — the direction that makes this shape hard
//! to find.

use std::fmt;

use tiler_ir::kernel::MAX_KERNEL_IDENTITY_BYTES;

use super::error::{ArtifactBuildError, ArtifactKeyKind};

/// Maximum UTF-8 byte length of one governed artifact key.
pub const MAX_GOVERNED_KEY_BYTES: usize = 256;
/// Maximum byte length of a fixed-width digest-shaped opaque identity.
///
/// This bounds [`PayloadDigest`], which is fixed-width under the governed digest
/// algorithm and cannot approach it.
///
/// It does **not** bound a [`BackendEntryKey`]. That is a canonical kernel
/// identity and takes `tiler_ir::kernel::MAX_KERNEL_IDENTITY_BYTES`, the exact
/// constant that mints one; the 1,024 shared here admitted only a degenerate
/// single-contributor reduction and refused every real one.
///
/// That last sentence is a measured claim, and its evidence is
/// `tiler_conformance`'s
/// `serial_sum::tests::the_serial_sum_identity_crosses_the_shared_opaque_bound_at_the_second_contributor`,
/// which compiles a serial `f32` sum at one and at two contributors and asserts
/// the identity lands on either side of this constant. It is named here because
/// nothing in *this* crate can check it: no `tiler-compiler` edge reaches here,
/// for the reason stated above `[dependencies]` in the manifest, so the crate
/// that owns the bound cannot compile a reduction to measure against it.
pub const MAX_OPAQUE_IDENTITY_BYTES: usize = 1_024;
/// Maximum byte length of a target-profile descriptor identity.
///
/// Unlike a digest, this is the canonical descriptor itself. It grows with the
/// profile's typed capability, query, dtype, and numerical declarations, so it
/// has its own resource ceiling rather than borrowing the unrelated payload
/// digest bound. `tiler_compiler::feasibility` owns the equal governing mint
/// bound; neither crate depends on the other, so a change requires checking both.
pub const MAX_TARGET_PROFILE_DESCRIPTOR_BYTES: usize = 64 * 1_024;

/// Whether one byte is admitted by the governed-key alphabet.
///
/// Equal to `tiler_compiler::target::TargetProfileKey`'s admitted set by
/// design; the module documentation states why the alphabets reconcile and the
/// byte bounds deliberately do not.
fn admits(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
}

fn validate_key(value: &str, kind: ArtifactKeyKind) -> Result<(), ArtifactBuildError> {
    if value.is_empty() {
        return Err(ArtifactBuildError::EmptyKey { kind });
    }
    if value.len() > MAX_GOVERNED_KEY_BYTES {
        return Err(ArtifactBuildError::KeyTooLong {
            kind,
            bytes: value.len(),
            limit: MAX_GOVERNED_KEY_BYTES,
        });
    }
    if let Some((index, byte)) = value.bytes().enumerate().find(|(_, byte)| !admits(*byte)) {
        return Err(ArtifactBuildError::NoncanonicalKeyByte {
            kind,
            index,
            value: byte,
        });
    }
    Ok(())
}

fn validate_opaque(
    value: &[u8],
    kind: ArtifactKeyKind,
    limit: usize,
) -> Result<(), ArtifactBuildError> {
    if value.is_empty() {
        return Err(ArtifactBuildError::EmptyKey { kind });
    }
    if value.len() > limit {
        return Err(ArtifactBuildError::KeyTooLong {
            kind,
            bytes: value.len(),
            limit,
        });
    }
    Ok(())
}

macro_rules! governed_key {
    ($name:ident, $kind:expr, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            /// Creates a validated governed key.
            ///
            /// # Errors
            ///
            /// Returns [`ArtifactBuildError::EmptyKey`] for an empty key,
            /// [`ArtifactBuildError::KeyTooLong`] beyond
            /// [`MAX_GOVERNED_KEY_BYTES`], or
            /// [`ArtifactBuildError::NoncanonicalKeyByte`] for a byte outside
            /// the governed-key alphabet of ASCII lowercase, ASCII digits,
            /// `.`, `-`, and `_`.
            pub fn new(value: impl AsRef<str>) -> Result<Self, ArtifactBuildError> {
                let value = value.as_ref();
                validate_key(value, $kind)?;
                Ok(Self(value.to_owned()))
            }

            /// Validates and retains an already-owned key without copying it.
            ///
            /// # Errors
            ///
            /// Returns the same errors as [`Self::new`], before retaining the
            /// string.
            pub fn from_owned(value: String) -> Result<Self, ArtifactBuildError> {
                validate_key(&value, $kind)?;
                Ok(Self(value))
            }

            /// Returns the exact key text.
            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

macro_rules! opaque_identity {
    ($name:ident, $kind:expr, $limit:expr, $limit_doc:literal, $($docs:literal),+ $(,)?) => {
        $(#[doc = $docs])+
        ///
        /// The bytes are treated as opaque: this crate compares and encodes
        /// them, and never re-derives them locally.
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(Box<[u8]>);

        impl $name {
            /// Wraps opaque identity bytes derived by another authority.
            ///
            /// # Errors
            ///
            /// Returns [`ArtifactBuildError::EmptyKey`] for empty bytes, or
            #[doc = $limit_doc]
            pub fn from_bytes(value: impl AsRef<[u8]>) -> Result<Self, ArtifactBuildError> {
                let value = value.as_ref();
                validate_opaque(value, $kind, $limit)?;
                Ok(Self(value.into()))
            }

            /// Returns the opaque identity bytes.
            #[must_use]
            pub fn as_bytes(&self) -> &[u8] {
                &self.0
            }
        }
    };
}

governed_key!(
    BackendKey,
    ArtifactKeyKind::Backend,
    "A governed backend family key, such as the Metal backend's key."
);
governed_key!(
    RepresentationKey,
    ArtifactKeyKind::Representation,
    "A governed executable-representation key of one backend payload."
);
governed_key!(
    TargetProfileKey,
    ArtifactKeyKind::TargetProfile,
    "A governed declared target-profile key."
);
governed_key!(
    FeasibilityRuleSetKey,
    ArtifactKeyKind::FeasibilityRuleSet,
    "A governed feasibility rule-set key."
);
governed_key!(
    CapabilityFamilyKey,
    ArtifactKeyKind::CapabilityFamily,
    "A governed lowering-capability family key."
);
governed_key!(
    RouteFeatureKey,
    ArtifactKeyKind::RouteFeature,
    "A governed backend-scoped route-requirement key, in its owner's namespace."
);
// `TargetPropertyKey` is not defined here. It moved to `tiler_ir::program::abi`
// with the expression domain that names it (ADR 0068, via
// `relocate-abi-expressions-into-tiler-ir`); leaving the key behind would have
// reintroduced the external side table that ADR rejects. It is re-exported
// through `super::expr` for this crate's callers.

opaque_identity!(
    BackendEntryKey,
    ArtifactKeyKind::BackendEntry,
    MAX_KERNEL_IDENTITY_BYTES,
    "[`ArtifactBuildError::KeyTooLong`] beyond `tiler_ir::kernel::MAX_KERNEL_IDENTITY_BYTES`.",
    "The opaque backend entry key one executable entry is realized by.",
    "",
    "This is the canonical identity of the structured kernel the entry realizes,",
    "which is why the bound is `tiler-ir`'s own for that value. The artifact",
    "already carries these exact bytes a second time in the same executable",
    "entry — `super::model::stage_key` prefixes them into the entry's stage",
    "subject, which the codec admits to 16 MiB — so a smaller bound here refused",
    "values the envelope beside it had already accepted, and guarded no",
    "allocation that stage subject had not already made.",
);
opaque_identity!(
    PayloadDigest,
    ArtifactKeyKind::PayloadDigest,
    MAX_OPAQUE_IDENTITY_BYTES,
    "[`ArtifactBuildError::KeyTooLong`] beyond [`MAX_OPAQUE_IDENTITY_BYTES`].",
    "The opaque content digest of one backend payload's exact bytes."
);
opaque_identity!(
    TargetProfileDescriptorDigest,
    ArtifactKeyKind::TargetProfileDescriptor,
    MAX_TARGET_PROFILE_DESCRIPTOR_BYTES,
    "[`ArtifactBuildError::KeyTooLong`] beyond [`MAX_TARGET_PROFILE_DESCRIPTOR_BYTES`].",
    "The opaque descriptor identity of one declared target profile.",
    "",
    "Named a digest, and not one: `tiler-compiler` emits the canonical descriptor",
    "bytes themselves rather than a hash of them, deliberately, so that no second",
    "identity has to be kept in agreement with what it summarizes.",
    "",
    "The governing bound is `tiler_compiler`'s `MAX_TARGET_PROFILE_DESCRIPTOR_BYTES`,",
    "enforced where a descriptor is minted, because the authority that can name the",
    "profile is the one that can explain the refusal. [`MAX_TARGET_PROFILE_DESCRIPTOR_BYTES`]",
    "is what this crate will hold, and it still refuses past it, because it",
    "validates what it is handed rather than trusting where it came from.",
);

/// The declared target profile a plan variant was assessed against.
///
/// ADR 0043 requires both the governed key and the exact descriptor identity: a
/// profile key alone is not evidence that a variant is legal on a device that
/// advertises the same key under a different descriptor.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TargetProfileRef {
    /// Governed key of the declared profile.
    pub key: TargetProfileKey,
    /// Exact descriptor identity of the declared profile.
    pub descriptor: TargetProfileDescriptorDigest,
}

/// The feasibility rule set under which a plan variant was assessed.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FeasibilityRuleSetRef {
    /// Governed key of the rule set.
    pub key: FeasibilityRuleSetKey,
    /// Nonzero output-affecting revision of the rule set.
    pub revision: u32,
}

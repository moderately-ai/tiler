//! Governed keys and opaque identities received at the artifact boundary.
//!
//! Two shapes appear here and they are deliberately different (ADR 0074 §2).
//!
//! A **governed key** is bounded UTF-8 text this crate compares and encodes as
//! meaning: which backend family, which executable representation, which
//! declared target profile, which feasibility rule set, which capability, which
//! target property. It has a validating constructor because a producer legally
//! names one.
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

use std::fmt;

use super::error::{ArtifactBuildError, ArtifactKeyKind};

/// Maximum UTF-8 byte length of one governed artifact key.
pub const MAX_GOVERNED_KEY_BYTES: usize = 256;
/// Maximum byte length of one opaque identity received at a boundary.
pub const MAX_OPAQUE_IDENTITY_BYTES: usize = 1_024;

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
    Ok(())
}

fn validate_opaque(value: &[u8], kind: ArtifactKeyKind) -> Result<(), ArtifactBuildError> {
    if value.is_empty() {
        return Err(ArtifactBuildError::EmptyKey { kind });
    }
    if value.len() > MAX_OPAQUE_IDENTITY_BYTES {
        return Err(ArtifactBuildError::KeyTooLong {
            kind,
            bytes: value.len(),
            limit: MAX_OPAQUE_IDENTITY_BYTES,
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
            /// Returns [`ArtifactBuildError::EmptyKey`] for an empty key, or
            /// [`ArtifactBuildError::KeyTooLong`] beyond
            /// [`MAX_GOVERNED_KEY_BYTES`].
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
    ($name:ident, $kind:expr, $docs:literal) => {
        #[doc = $docs]
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
            /// [`ArtifactBuildError::KeyTooLong`] beyond
            /// [`MAX_OPAQUE_IDENTITY_BYTES`].
            pub fn from_bytes(value: impl AsRef<[u8]>) -> Result<Self, ArtifactBuildError> {
                let value = value.as_ref();
                validate_opaque(value, $kind)?;
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
    CapabilityKey,
    ArtifactKeyKind::Capability,
    "A governed capability key one provider was selected for."
);
// `TargetPropertyKey` is not defined here. It moved to `tiler_ir::program::abi`
// with the expression domain that names it (ADR 0068, via
// `relocate-abi-expressions-into-tiler-ir`); leaving the key behind would have
// reintroduced the external side table that ADR rejects. It is re-exported
// through `super::expr` for this crate's callers.

opaque_identity!(
    BackendEntryKey,
    ArtifactKeyKind::BackendEntry,
    "The opaque backend entry key one executable entry is realized by."
);
opaque_identity!(
    PayloadDigest,
    ArtifactKeyKind::PayloadDigest,
    "The opaque content digest of one backend payload's exact bytes."
);
opaque_identity!(
    TargetProfileDescriptorDigest,
    ArtifactKeyKind::TargetProfileDescriptor,
    "The opaque descriptor digest of one declared target profile."
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

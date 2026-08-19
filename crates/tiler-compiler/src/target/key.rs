//! The validated key one target profile is named and compared by.
//!
//! A profile key is what a compilation is assessed against and what an artifact
//! record is matched to, so the alphabet, the minting bound, and the owned
//! identity that carries a key into a checked fact are kept together here.

use std::sync::Arc;

pub(crate) const GOVERNED_TARGET_PROFILE_KEY: &str = "tiler.prototype-target-neutral-baseline.v1";

/// Maximum byte length of one target-profile key.
///
/// A *minting* bound: what a profile key this compiler build names may occupy.
/// `tiler_artifact::program::MAX_GOVERNED_KEY_BYTES` is 256 because that layer
/// holds keys minted by producers other than this compiler, and the smaller
/// number here is what makes the two safe together — every key this compiler
/// can name is packageable there. Neither crate depends on the other, so a
/// change requires checking both.
pub const MAX_TARGET_PROFILE_KEY_BYTES: usize = 128;

/// Typed target-profile key validation diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetProfileKeyError {
    /// The key was empty.
    Empty,
    /// The encoded key exceeded the bounded identity field.
    TooLong {
        /// Actual encoded byte length.
        actual: usize,
        /// Maximum admitted encoded byte length.
        max: usize,
    },
    /// One byte was outside the canonical key alphabet.
    InvalidByte {
        /// Zero-based byte offset.
        index: usize,
        /// Refused byte value.
        value: u8,
    },
}

impl std::fmt::Display for TargetProfileKeyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for TargetProfileKeyError {}

/// The owned, validated key of one declared target profile.
///
/// A key is non-empty, at most [`MAX_TARGET_PROFILE_KEY_BYTES`], and spelled in
/// ASCII lowercase, ASCII digits, `.`, `-`, and `_`.
///
/// # The alphabet is shared with the artifact layer, and shared deliberately
///
/// `tiler_artifact::program::TargetProfileKey` is a different type with the
/// same name — this one is what a compilation is *assessed against*, that one
/// is what a packaged artifact *carries* — and it admits exactly this alphabet.
/// The two agree because a profile key's whole job is to be compared byte for
/// byte against one some other producer minted: a spelling only one side admits
/// would leave two keys a reader sees as one comparing unequal, and a key
/// carrying case, whitespace, or a control byte cannot be reproduced from the
/// rejection that prints it. Neither crate depends on the other, so widening
/// either alphabet requires checking both.
///
/// The byte bounds are not shared and are not meant to be;
/// [`MAX_TARGET_PROFILE_KEY_BYTES`] records why.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TargetProfileKey(Arc<str>);

impl TargetProfileKey {
    /// Names a key governed by this compiler build.
    pub(crate) fn governed(key: &'static str) -> Self {
        Self::declared(key.to_owned()).expect("a source-governed target-profile key is valid")
    }

    /// Validates and retains a caller-owned key.
    ///
    /// # Errors
    ///
    /// Returns a key-specific diagnostic for an empty, oversized, or
    /// noncanonical key.
    pub fn new(key: String) -> Result<Self, TargetProfileKeyError> {
        let admitted = |byte: u8| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-' | b'_')
        };
        if key.is_empty() {
            return Err(TargetProfileKeyError::Empty);
        }
        if key.len() > MAX_TARGET_PROFILE_KEY_BYTES {
            return Err(TargetProfileKeyError::TooLong {
                actual: key.len(),
                max: MAX_TARGET_PROFILE_KEY_BYTES,
            });
        }
        if let Some((index, value)) = key.bytes().enumerate().find(|(_, byte)| !admitted(*byte)) {
            return Err(TargetProfileKeyError::InvalidByte { index, value });
        }
        Ok(Self(Arc::from(key)))
    }

    /// Returns the canonical validated spelling.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TargetProfileKey {
    pub(super) fn declared(key: String) -> Result<Self, TargetProfileKeyError> {
        Self::new(key)
    }
}

impl std::fmt::Display for TargetProfileKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl AsRef<str> for TargetProfileKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Owned identity of the profile that attributed a checked fact.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(crate) struct TargetProfileIdentity {
    key: TargetProfileKey,
}

impl TargetProfileIdentity {
    #[cfg(test)]
    pub(crate) fn new(key: &'static str) -> Self {
        Self::governed(key)
    }

    pub(crate) fn from_key(key: TargetProfileKey) -> Self {
        Self { key }
    }

    #[cfg(test)]
    pub(crate) fn governed(key: &'static str) -> Self {
        Self::from_key(TargetProfileKey::governed(key))
    }

    pub(crate) fn key(&self) -> &str {
        self.key.as_str()
    }

    pub(crate) const fn public_key(&self) -> &TargetProfileKey {
        &self.key
    }
}

impl From<&TargetProfileIdentity> for TargetProfileIdentity {
    fn from(value: &TargetProfileIdentity) -> Self {
        value.clone()
    }
}

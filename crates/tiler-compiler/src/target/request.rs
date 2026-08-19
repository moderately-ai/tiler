//! The ordered target set one compilation request is assessed against.
//!
//! Validation here is about the set rather than any profile in it: nonempty,
//! bounded, and unique by validated profile key, with the caller's declared
//! result order preserved because a consumer indexes its outputs by it.

use crate::target::key::TargetProfileKey;
use crate::target::profile::TargetProfile;

/// Maximum target profiles admitted in one compilation request.
pub const MAX_TARGET_PROFILES_PER_REQUEST: usize = 16;

/// Ordered, nonempty, unique target set for one compilation request.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetRequest {
    profiles: Vec<TargetProfile>,
}

impl TargetRequest {
    /// Validates the target set without reordering it.
    ///
    /// # Errors
    ///
    /// Returns [`TargetRequestError::Empty`],
    /// [`TargetRequestError::TooManyProfiles`], or
    /// [`TargetRequestError::DuplicateProfile`].
    pub fn new(
        profiles: impl IntoIterator<Item = TargetProfile>,
    ) -> Result<Self, TargetRequestError> {
        let profiles: Vec<_> = profiles
            .into_iter()
            .take(MAX_TARGET_PROFILES_PER_REQUEST + 1)
            .collect();
        if profiles.is_empty() {
            return Err(TargetRequestError::Empty);
        }
        if profiles.len() > MAX_TARGET_PROFILES_PER_REQUEST {
            return Err(TargetRequestError::TooManyProfiles {
                actual: profiles.len(),
                max: MAX_TARGET_PROFILES_PER_REQUEST,
            });
        }
        let mut keys: Vec<_> = profiles
            .iter()
            .enumerate()
            .map(|(index, profile)| (profile.profile_key(), index))
            .collect();
        keys.sort_unstable_by(|left, right| left.0.cmp(right.0).then(left.1.cmp(&right.1)));
        if let Some(pair) = keys.windows(2).find(|pair| pair[0].0 == pair[1].0) {
            let (first, duplicate) = if pair[0].1 < pair[1].1 {
                (pair[0].1, pair[1].1)
            } else {
                (pair[1].1, pair[0].1)
            };
            return Err(TargetRequestError::DuplicateProfile {
                profile: pair[0].0.clone(),
                first,
                duplicate,
            });
        }
        Ok(Self { profiles })
    }

    /// Returns the profiles in caller-declared result order.
    #[must_use]
    pub fn profiles(&self) -> &[TargetProfile] {
        &self.profiles
    }

    pub(crate) fn into_profiles(self) -> Vec<TargetProfile> {
        self.profiles
    }
}

/// Typed target-set construction failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TargetRequestError {
    /// No target was stated.
    Empty,
    /// The target set exceeded its admitted cardinality.
    TooManyProfiles {
        /// Observed cardinality, capped at `max + 1`.
        actual: usize,
        /// Maximum admitted cardinality.
        max: usize,
    },
    /// Two profiles have the same validated profile key.
    DuplicateProfile {
        /// The duplicated profile key.
        profile: TargetProfileKey,
        /// Zero-based position of the first occurrence.
        first: usize,
        /// Zero-based position of the duplicate occurrence.
        duplicate: usize,
    },
}

impl std::fmt::Display for TargetRequestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for TargetRequestError {}

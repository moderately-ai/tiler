use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct BuilderId(u64);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct VerifiedRegionOwner(u64);

static NEXT_BUILDER_ID: AtomicU64 = AtomicU64::new(1);

pub(super) fn next_builder_id() -> Option<BuilderId> {
    NEXT_BUILDER_ID
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .ok()
        .map(BuilderId)
}

impl BuilderId {
    pub(super) const fn verified_owner(self) -> VerifiedRegionOwner {
        VerifiedRegionOwner(self.0)
    }
}

macro_rules! draft_handle {
    ($name:ident) => {
        #[doc = concat!("A builder-owned `", stringify!($name), "` handle.")]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name {
            pub(super) owner: BuilderId,
            pub(super) index: u32,
        }

        impl $name {
            pub(super) fn from_len(owner: BuilderId, len: usize) -> Option<Self> {
                u32::try_from(len).ok().map(|index| Self { owner, index })
            }
        }
    };
}

draft_handle!(DimensionId);
draft_handle!(IndexExprId);
draft_handle!(TensorId);
draft_handle!(TensorAccessId);
draft_handle!(ScalarOperationId);
draft_handle!(ScalarValueId);

impl ScalarValueId {
    pub(super) fn as_usize(self) -> usize {
        usize::try_from(self.index).expect("u32 fits every supported host usize")
    }
}

macro_rules! verified_handle {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name {
            pub(super) owner: VerifiedRegionOwner,
            pub(super) index: u32,
        }

        impl $name {
            pub(super) const fn from_verified(owner: VerifiedRegionOwner, index: u32) -> Self {
                Self { owner, index }
            }

            pub(super) fn as_usize(self) -> usize {
                usize::try_from(self.index).expect("u32 fits every supported host usize")
            }
        }
    };
}

verified_handle!(
    VerifiedDimensionId,
    "A verified region-local domain dimension."
);
verified_handle!(
    VerifiedIndexExprId,
    "A verified region-local index expression."
);
verified_handle!(VerifiedTensorId, "A verified region-local tensor boundary.");
verified_handle!(
    VerifiedTensorAccessId,
    "A verified region-local logical access."
);
verified_handle!(
    VerifiedScalarOperationId,
    "A verified region-local scalar operation occurrence."
);
verified_handle!(
    VerifiedScalarValueId,
    "A verified region-local scalar SSA value."
);

/// Ordered result position within one scalar operation occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarResultIndex(u32);

impl ScalarResultIndex {
    pub(super) fn from_usize(index: usize) -> Option<Self> {
        u32::try_from(index).ok().map(Self)
    }
    /// Returns the zero-based result position.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// One verified value local to a particular reducer body occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VerifiedReducerBodyValueId {
    pub(super) owner: VerifiedRegionOwner,
    pub(super) reduction: u32,
    pub(super) index: u32,
}

/// One verified operation local to a particular reducer body occurrence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VerifiedReducerBodyOperationId {
    pub(super) owner: VerifiedRegionOwner,
    pub(super) reduction: u32,
    pub(super) index: u32,
}

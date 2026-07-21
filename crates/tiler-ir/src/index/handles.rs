use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct BuilderId(u64);

static NEXT_BUILDER_ID: AtomicU64 = AtomicU64::new(1);

pub(super) fn next_builder_id() -> Option<BuilderId> {
    NEXT_BUILDER_ID
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .ok()
        .map(BuilderId)
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

            pub(super) fn as_usize(self) -> usize {
                usize::try_from(self.index).expect("u32 fits every supported host usize")
            }
        }
    };
}

draft_handle!(DimensionId);
draft_handle!(IndexExprId);
draft_handle!(TensorId);
draft_handle!(TensorAccessId);
draft_handle!(ScalarExprId);

macro_rules! verified_handle {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(pub(super) u32);
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
    VerifiedScalarExprId,
    "A verified region-local scalar expression."
);

/// A verified bounds witness owned by one immutable index region.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BoundsWitnessId(pub(super) u32);

/// A verified complete, unique ordinary-write witness.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WriteOwnershipWitnessId(pub(super) u32);

//! Opaque layer-local reference newtypes for the structured kernel IR.
//!
//! Every durable reference is a kernel-specific newtype backed by a checked
//! compact `u32` and an ownership tag, per ADR 0071. A builder-owned handle
//! carries the identity of the builder that minted it, and a verified handle
//! carries the identity of the verified kernel that retains it, so a handle can
//! never be forged, reused across kernels, or fabricated from an integer.

use std::sync::atomic::{AtomicU64, Ordering};

/// Ownership tag of one open kernel builder.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct KernelBuilderId(u64);

/// Ownership tag of one verified kernel.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct VerifiedKernelOwner(u64);

static NEXT_KERNEL_BUILDER_ID: AtomicU64 = AtomicU64::new(1);

/// Mints a fresh builder ownership tag, or `None` when the space is exhausted.
pub(super) fn next_kernel_builder_id() -> Option<KernelBuilderId> {
    NEXT_KERNEL_BUILDER_ID
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .ok()
        .map(KernelBuilderId)
}

impl KernelBuilderId {
    /// Projects the builder tag into the verified-kernel tag it becomes.
    pub(super) const fn verified_owner(self) -> VerifiedKernelOwner {
        VerifiedKernelOwner(self.0)
    }
}

macro_rules! draft_handle {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name {
            pub(super) owner: KernelBuilderId,
            pub(super) index: u32,
        }

        impl $name {
            pub(super) fn from_len(owner: KernelBuilderId, len: usize) -> Option<Self> {
                u32::try_from(len).ok().map(|index| Self { owner, index })
            }

            pub(super) fn as_usize(self) -> usize {
                usize::try_from(self.index).expect("u32 fits every supported host usize")
            }
        }
    };
}

draft_handle!(
    KernelValueId,
    "A builder-owned structured-kernel SSA value."
);
draft_handle!(KernelBufferId, "A builder-owned kernel buffer parameter.");

macro_rules! verified_handle {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name {
            pub(super) owner: VerifiedKernelOwner,
            pub(super) index: u32,
        }

        impl $name {
            pub(super) const fn from_verified(owner: VerifiedKernelOwner, index: u32) -> Self {
                Self { owner, index }
            }

            pub(super) fn as_usize(self) -> usize {
                usize::try_from(self.index).expect("u32 fits every supported host usize")
            }
        }
    };
}

verified_handle!(
    VerifiedValueId,
    "A verified kernel-local structured SSA value."
);
verified_handle!(
    VerifiedBufferId,
    "A verified kernel-local buffer parameter."
);

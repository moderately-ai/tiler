//! Opaque layer-local reference newtypes for the kernel-program IR.
//!
//! Every durable reference is a program-specific newtype backed by a checked
//! compact `u32` and the ownership tag of the builder that minted it, per ADR
//! 0071. A handle can therefore never be forged from an integer, reused across
//! two programs, or resolved against a builder that did not define it.
//!
//! The verified product exposes its content through borrowed `*Ref<'_>` views
//! rather than a second verified handle space: a consumer walks stages,
//! dependencies, values, views, and allocations by reference, so no verified
//! identifier needs to be resolved and none can be misapplied.

use std::sync::atomic::{AtomicU64, Ordering};

/// Ownership tag of one open kernel-program builder.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct ProgramBuilderId(u64);

static NEXT_PROGRAM_BUILDER_ID: AtomicU64 = AtomicU64::new(1);

/// Mints a fresh builder ownership tag, or `None` when the space is exhausted.
pub(super) fn next_program_builder_id() -> Option<ProgramBuilderId> {
    NEXT_PROGRAM_BUILDER_ID
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .ok()
        .map(ProgramBuilderId)
}

macro_rules! draft_handle {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name {
            pub(super) owner: ProgramBuilderId,
            pub(super) index: u32,
        }

        impl $name {
            pub(super) fn from_len(owner: ProgramBuilderId, len: usize) -> Option<Self> {
                u32::try_from(len).ok().map(|index| Self { owner, index })
            }

            pub(super) fn as_usize(self) -> usize {
                usize::try_from(self.index).expect("u32 fits every supported host usize")
            }
        }
    };
}

draft_handle!(StageId, "A builder-owned program stage.");
draft_handle!(
    AbiExprId,
    "A builder-owned node of the program's ABI expression arena."
);
draft_handle!(
    MaterializedValueId,
    "A builder-owned materialized program value."
);
draft_handle!(AllocationId, "A builder-owned program storage allocation.");
draft_handle!(ViewId, "A builder-owned byte view of a materialized value.");

//! Opaque layer-local reference newtypes for the artifact program model.
//!
//! Every durable reference a producer holds is an artifact-specific newtype
//! backed by a checked compact `u32` and the ownership tag of the builder that
//! minted it (ADR 0071). A handle can therefore never be forged from an
//! integer, reused across two artifacts, or resolved against a builder that did
//! not define it.
//!
//! The verified product exposes its content through borrowed `*Ref<'_>` views
//! rather than a second verified handle space, so no verified identifier needs
//! resolving and none can be misapplied.

use std::sync::atomic::{AtomicU64, Ordering};

/// Ownership tag of one open artifact-program builder.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct ArtifactBuilderId(u64);

static NEXT_ARTIFACT_BUILDER_ID: AtomicU64 = AtomicU64::new(1);

/// Mints a fresh builder ownership tag, or `None` when the space is exhausted.
pub(super) fn next_artifact_builder_id() -> Option<ArtifactBuilderId> {
    NEXT_ARTIFACT_BUILDER_ID
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .ok()
        .map(ArtifactBuilderId)
}

macro_rules! draft_handle {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name {
            pub(super) owner: ArtifactBuilderId,
            pub(super) index: u32,
        }

        impl $name {
            pub(super) fn from_len(owner: ArtifactBuilderId, len: usize) -> Option<Self> {
                u32::try_from(len).ok().map(|index| Self { owner, index })
            }

            #[allow(
                dead_code,
                reason = "VariantId is minted for a producer's own bookkeeping; the builder never resolves it back"
            )]
            pub(super) fn as_usize(self) -> usize {
                usize::try_from(self.index).expect("u32 fits every supported host usize")
            }
        }
    };
}

draft_handle!(AbiExprId, "A builder-owned ABI expression arena node.");
draft_handle!(PayloadId, "A builder-owned backend payload descriptor.");
draft_handle!(VariantId, "A builder-owned complete plan variant.");

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub(super) struct GraphId(u64);

static NEXT_GRAPH_ID: AtomicU64 = AtomicU64::new(1);

pub(super) fn next_graph_id() -> Option<GraphId> {
    allocate_graph_id(&NEXT_GRAPH_ID)
}

fn allocate_graph_id(next: &AtomicU64) -> Option<GraphId> {
    next.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .ok()
        .map(GraphId)
}

/// A graph-owned semantic operation handle.
///
/// Handles are transient lookup capabilities, not stable or serializable identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OperationId {
    pub(super) owner: GraphId,
    pub(super) index: OperationIndex,
}

/// A graph-owned semantic value handle.
///
/// Handles are transient lookup capabilities, not stable or serializable identity.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ValueId {
    pub(super) owner: GraphId,
    pub(super) index: ValueIndex,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(super) struct OperationIndex(u32);

impl OperationIndex {
    pub(super) fn from_len(len: usize) -> Option<Self> {
        u32::try_from(len).ok().map(Self)
    }

    pub(super) fn from_verified_len(len: usize) -> Self {
        match Self::from_len(len) {
            Some(index) => index,
            None => verified_index_overflow(),
        }
    }

    pub(super) fn as_usize(self) -> usize {
        usize::try_from(self.0).expect("u32 fits every supported host usize")
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub(super) struct ValueIndex(u32);

impl ValueIndex {
    pub(super) fn from_len(len: usize) -> Option<Self> {
        u32::try_from(len).ok().map(Self)
    }

    pub(super) fn from_verified_len(len: usize) -> Self {
        match Self::from_len(len) {
            Some(index) => index,
            None => verified_index_overflow(),
        }
    }

    pub(super) const fn get(self) -> u32 {
        self.0
    }

    pub(super) fn as_usize(self) -> usize {
        usize::try_from(self.0).expect("u32 fits every supported host usize")
    }
}

#[cold]
#[track_caller]
fn verified_index_overflow() -> ! {
    panic!("verified semantic arena exceeded its fixed-width index space")
}

#[cfg(test)]
mod tests {
    use super::{OperationIndex, ValueIndex, allocate_graph_id};
    use std::mem::size_of;
    use std::sync::atomic::AtomicU64;

    #[test]
    fn private_indices_are_compact() {
        assert_eq!(size_of::<ValueIndex>(), size_of::<u32>());
        assert_eq!(size_of::<OperationIndex>(), size_of::<u32>());
    }

    #[test]
    fn graph_id_allocation_fails_without_reusing_the_last_live_id() {
        let next = AtomicU64::new(u64::MAX - 1);
        let last = allocate_graph_id(&next).expect("the final allocatable ID is available");

        assert!(allocate_graph_id(&next).is_none());
        assert!(allocate_graph_id(&next).is_none());
        assert_eq!(last, super::GraphId(u64::MAX - 1));
    }
}

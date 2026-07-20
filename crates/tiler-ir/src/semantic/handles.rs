use std::marker::PhantomData;
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

/// An exact statically typed authoring capability for one graph-owned value.
///
/// `T` is process-local evidence resolved through the graph's frozen semantic
/// registry. The canonical graph stores only [`ValueId`] and its authoritative
/// runtime [`ResolvedValueType`](super::ResolvedValueType).
#[repr(transparent)]
pub struct Value<T> {
    id: ValueId,
    marker: PhantomData<fn() -> T>,
}

impl<T> Value<T> {
    pub(super) const fn from_verified(id: ValueId) -> Self {
        Self {
            id,
            marker: PhantomData,
        }
    }

    /// Explicitly erases static type evidence to an unknown-typed identity.
    #[must_use]
    pub const fn erase(self) -> ValueId {
        self.id
    }
}

impl<T> Clone for Value<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Value<T> {}

impl<T> std::fmt::Debug for Value<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.debug_tuple("Value").field(&self.id).finish()
    }
}

impl<T> PartialEq for Value<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T> Eq for Value<T> {}

impl<T> std::hash::Hash for Value<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
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
    use super::{OperationIndex, Value, ValueId, ValueIndex, allocate_graph_id};
    use crate::semantic::ValueTypeMarker;
    use std::mem::size_of;
    use std::rc::Rc;
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

    #[allow(dead_code)]
    struct NonSendMarker(Rc<()>);
    impl ValueTypeMarker for NonSendMarker {}

    #[test]
    fn typed_handle_layout_and_thread_safety_do_not_depend_on_marker_layout() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_eq!(size_of::<Value<NonSendMarker>>(), size_of::<ValueId>());
        assert_send_sync::<Value<NonSendMarker>>();
    }
}

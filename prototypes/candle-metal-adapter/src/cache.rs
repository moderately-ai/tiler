//! The runtime library and pipeline cache, scoped to one device and context.
//!
//! # Why the scope is in the key rather than around the map
//!
//! A cache keyed by a name two devices could share is the defect this module
//! exists to make unrepresentable. A `MTLLibrary` and a `MTLComputePipelineState`
//! belong to the `MTLDevice` that created them; handing one to another device's
//! encoder is undefined, and no digest, symbol, or bundle name distinguishes the
//! two entries, because the bytes really are identical — it is the *device* that
//! differs.
//!
//! So [`DeviceScope`] is a field of every key rather than a property of the map,
//! and [`PipelineCache`] additionally records the scope it was built for.
//! Together those make the criterion structural in two independent ways: a
//! lookup minted under another device produces a key that is not equal to any
//! stored one, so it cannot hit; and [`PipelineCache::scoped_to`] refuses a
//! foreign scope outright, which is the check that can *say no* rather than
//! silently miss.
//!
//! Both halves are needed. Key inequality alone means a cross-device lookup
//! misses and then rebuilds — correct, but silent, and indistinguishable from a
//! cold cache. The explicit refusal is what turns "unusable from another device"
//! into an observable event a test can watch fail.
//!
//! # Why the scope is two identifiers and not one
//!
//! Candle's `MetalDevice::id` is a process-local counter, so it separates two
//! Candle devices in this process and says nothing across processes.
//! `MTLDevice.registryID` names the GPU and is stable across task boundaries,
//! but two Candle `MetalDevice`s wrapping the same GPU share it — and they do
//! *not* share an allocator, a command queue, or a residency set. Neither alone
//! is the scope; the pair is.
//!
//! **The context half is spelled through `Debug`, and that is a Candle API
//! limitation rather than a choice.** `candle_core::metal_backend::DeviceId` is
//! a public `Copy` type whose constructor is `pub(crate)` and whose field is
//! private, so a consumer can compare two of them and cannot mint one. A fixture
//! that needs two distinct contexts without a GPU therefore has no way to build
//! them from the type itself, and rendering the identity is the only
//! device-free-constructible spelling Candle leaves available. Nothing here
//! parses that rendering; it is compared for equality exactly as the `DeviceId`
//! would have been.

use std::collections::HashMap;
use std::fmt;

use candle_core::MetalDevice;
use candle_metal_kernels::metal::{ComputePipeline, Library};

use crate::refusal::RouteRefusal;

/// The device and context one cached device object belongs to.
///
/// `Hash` and `Eq` because it is a key component. It holds no reference to the
/// device, deliberately: a scope that borrowed one would tie every cache entry's
/// lifetime to a borrow rather than to the identity it names.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DeviceScope {
    /// Candle's own per-`MetalDevice` identity, unique within this process.
    context: String,
    /// The GPU's registry identifier, stable across task boundaries.
    registry: u64,
}

impl DeviceScope {
    /// Reads the scope one bound Candle Metal device defines.
    pub fn of(device: &MetalDevice) -> Self {
        Self {
            context: format!("{:?}", device.id()),
            registry: device.registry_id(),
        }
    }

    /// Builds a scope from its two identifiers.
    ///
    /// Exists for fixtures that must exhibit two distinct scopes without two
    /// GPUs; see the module documentation for why Candle leaves no other way.
    /// Test-only on purpose: a production caller that could state a scope could
    /// state one for a device it did not bind, which is the whole thing the
    /// scope exists to make impossible.
    #[cfg(test)]
    pub fn from_parts(context: impl Into<String>, registry: u64) -> Self {
        Self {
            context: context.into(),
            registry,
        }
    }
}

impl fmt::Display for DeviceScope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}@registry-{:#x}", self.context, self.registry)
    }
}

/// One cached Metal library, named by the device it belongs to and the object it was built from.
///
/// The artifact's own canonical identity names the bytes rather than a digest
/// taken here: the artifact layer already proved the carried object's integrity
/// digest, so hashing the object again would be a second identity for one thing.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct LibraryKey {
    scope: DeviceScope,
    /// The artifact identity these object bytes were carried by.
    artifact: Vec<u8>,
    /// Position of the entry in the route's execution order.
    ///
    /// Present because nothing requires two entries of one variant to be
    /// realized by the same payload, so an artifact identity alone does not name
    /// one object.
    entry: usize,
}

/// One cached compute pipeline, named by its library and the symbol it was built for.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PipelineKey {
    library: LibraryKey,
    symbol: String,
}

/// Libraries and pipelines this process built for one device and context.
///
/// Never global and never shared between devices: every read and write goes
/// through [`Self::scoped_to`], which refuses any device but the one this cache
/// was built for.
#[derive(Debug)]
pub struct PipelineCache {
    scope: DeviceScope,
    libraries: HashMap<LibraryKey, Library>,
    pipelines: HashMap<PipelineKey, ComputePipeline>,
}

impl PipelineCache {
    /// Builds an empty cache for one bound device and context.
    pub fn new(device: &MetalDevice) -> Self {
        Self::for_scope(DeviceScope::of(device))
    }

    /// Builds an empty cache for a scope stated directly.
    fn for_scope(scope: DeviceScope) -> Self {
        Self {
            scope,
            libraries: HashMap::new(),
            pipelines: HashMap::new(),
        }
    }

    /// Returns this cache's scope after proving it is the caller's own.
    ///
    /// # Errors
    ///
    /// Returns [`RouteRefusal::ForeignDeviceScope`] when the scope is not the
    /// one this cache was built for.
    pub fn scoped_to(&self, lookup: &DeviceScope) -> Result<DeviceScope, RouteRefusal> {
        if *lookup != self.scope {
            return Err(RouteRefusal::ForeignDeviceScope {
                cache: self.scope.to_string(),
                lookup: lookup.to_string(),
            });
        }
        Ok(self.scope.clone())
    }

    /// Builds the key one entry's library is cached under.
    ///
    /// # Errors
    ///
    /// Returns [`RouteRefusal::ForeignDeviceScope`] for a foreign scope.
    pub fn library_key(
        &self,
        lookup: &DeviceScope,
        artifact: &[u8],
        entry: usize,
    ) -> Result<LibraryKey, RouteRefusal> {
        Ok(LibraryKey {
            scope: self.scoped_to(lookup)?,
            artifact: artifact.to_vec(),
            entry,
        })
    }

    /// Returns a cached library, or `None`.
    pub fn library(&self, key: &LibraryKey) -> Option<&Library> {
        self.libraries.get(key)
    }

    /// Stores one library under its key.
    pub fn insert_library(&mut self, key: LibraryKey, library: Library) {
        self.libraries.insert(key, library);
    }

    /// Builds the key one entry's pipeline is cached under.
    pub fn pipeline_key(library: &LibraryKey, symbol: &str) -> PipelineKey {
        PipelineKey {
            library: library.clone(),
            symbol: symbol.to_owned(),
        }
    }

    /// Returns a cached pipeline, or `None`.
    pub fn pipeline(&self, key: &PipelineKey) -> Option<&ComputePipeline> {
        self.pipelines.get(key)
    }

    /// Stores one pipeline under its key.
    pub fn insert_pipeline(&mut self, key: PipelineKey, pipeline: ComputePipeline) {
        self.pipelines.insert(key, pipeline);
    }

    /// Returns how many libraries and pipelines this cache holds.
    ///
    /// Reported by the proof so a second route over the same artifact is
    /// visibly a hit rather than a rebuild that happened to be fast.
    pub fn occupancy(&self) -> (usize, usize) {
        (self.libraries.len(), self.pipelines.len())
    }
}

#[cfg(test)]
mod tests {
    use super::{DeviceScope, PipelineCache};

    /// Two GPUs under one context are two scopes.
    fn two_registries() -> (DeviceScope, DeviceScope) {
        (
            DeviceScope::from_parts("DeviceId(1)", 0x1),
            DeviceScope::from_parts("DeviceId(1)", 0x2),
        )
    }

    /// Two Candle contexts over one GPU are two scopes.
    fn two_contexts() -> (DeviceScope, DeviceScope) {
        (
            DeviceScope::from_parts("DeviceId(1)", 0x1),
            DeviceScope::from_parts("DeviceId(2)", 0x1),
        )
    }

    /// An entry built under one scope cannot be *found* from another.
    ///
    /// This is criterion 4's "unusable from another by construction": the
    /// assertion is about key inequality, so it holds even for an implementation
    /// that forgot to check anything. Both halves of the scope are varied
    /// independently, because a scope that dropped either would still pass a
    /// test that varied only the other.
    #[test]
    fn a_key_minted_under_one_scope_does_not_equal_another() {
        for (one, other) in [two_registries(), two_contexts()] {
            let cache = PipelineCache::for_scope(one.clone());
            let mine = cache
                .library_key(&one, b"artifact-identity", 0)
                .expect("the cache's own scope is admitted");
            let theirs = PipelineCache::for_scope(other.clone())
                .library_key(&other, b"artifact-identity", 0)
                .expect("the other cache's own scope is admitted");
            assert_ne!(
                mine, theirs,
                "identical object bytes under two scopes must not share a cache entry",
            );
        }
    }

    /// A foreign scope is refused rather than treated as a miss.
    ///
    /// The half key inequality cannot supply: a miss and a refusal are
    /// indistinguishable from the outside, and only the refusal is observable
    /// evidence that the cache knows whose it is.
    #[test]
    fn a_foreign_scope_is_refused() {
        let (mine, theirs) = two_registries();
        let cache = PipelineCache::for_scope(mine.clone());
        assert!(
            cache.scoped_to(&mine).is_ok(),
            "a cache must admit the scope it was built for, or this test proves nothing",
        );
        let refusal = cache
            .library_key(&theirs, b"artifact-identity", 0)
            .expect_err("a cache scoped to one device must refuse another");
        let rendered = refusal.to_string();
        assert!(
            rendered.contains("candle-metal.cache"),
            "{rendered:?} does not name the cache boundary",
        );
        assert!(
            rendered.contains(&mine.to_string()) && rendered.contains(&theirs.to_string()),
            "{rendered:?} must name both scopes so a reader can tell which is which",
        );
    }

    /// Two entries of one artifact are two libraries, and two symbols two pipelines.
    #[test]
    fn entry_position_and_symbol_both_separate_cache_entries() {
        let scope = DeviceScope::from_parts("DeviceId(1)", 0x1);
        let cache = PipelineCache::for_scope(scope.clone());
        let first = cache
            .library_key(&scope, b"artifact-identity", 0)
            .expect("the cache's own scope");
        let second = cache
            .library_key(&scope, b"artifact-identity", 1)
            .expect("the cache's own scope");
        assert_ne!(first, second);
        assert_ne!(
            PipelineCache::pipeline_key(&first, "tiler_kernel_a"),
            PipelineCache::pipeline_key(&first, "tiler_kernel_b"),
        );
    }

    /// An empty cache reports an empty occupancy, so a later hit is visible.
    #[test]
    fn a_fresh_cache_is_empty() {
        let cache = PipelineCache::for_scope(DeviceScope::from_parts("DeviceId(1)", 0x1));
        assert_eq!(cache.occupancy(), (0, 0));
    }
}

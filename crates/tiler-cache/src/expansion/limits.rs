//! The bounds a cache read allocates and publishes within.

use std::time::Duration;

/// Default maximum bytes of one stored bundle.
///
/// Sized so a read refuses before allocating rather than after: a hostile or
/// damaged length field is compared against this before any buffer is grown,
/// and the read itself is bounded so the file cannot outrun the check.
pub const DEFAULT_MAX_BUNDLE_BYTES: u64 = 256 * 1024 * 1024;

/// Default maximum framed sections in one bundle.
pub const DEFAULT_MAX_SECTIONS: u32 = 16;

/// Default maximum bytes of one framed section.
pub const DEFAULT_MAX_SECTION_BYTES: u64 = DEFAULT_MAX_BUNDLE_BYTES;

/// Default maximum bytes retained across all quarantined entries of one shard.
pub const DEFAULT_MAX_QUARANTINE_BYTES: u64 = 64 * 1024 * 1024;

/// Default age at which an abandoned temporary file may be swept.
///
/// Long enough that a slow but live writer's temporary is never removed under
/// it: a sweep that raced a writer would delete a file the writer is about to
/// rename, turning a correct publication into a spurious failure.
pub const DEFAULT_TEMPORARY_GRACE: Duration = Duration::from_hours(6);

/// The bounds one cache instance reads and publishes within.
///
/// # Why there is no maximum entry count here
///
/// Bounding the number of entries means evicting one when the bound is reached,
/// and choosing *which* is a garbage-collection policy the research note
/// requires to be designed and stress-tested separately
/// (`design-bounded-expansion-cache-garbage-collection`). A field that recorded
/// a bound nothing enforced would be worse than its absence: it would read as a
/// guarantee. Every field below is checked at the point it applies.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Maximum bytes of one stored bundle, checked before allocation.
    pub max_bundle_bytes: u64,
    /// Maximum framed sections in one bundle.
    pub max_sections: u32,
    /// Maximum bytes of one framed section.
    pub max_section_bytes: u64,
    /// Maximum bytes retained across all quarantined entries of one shard.
    ///
    /// Quarantine keeps a rejected entry for diagnosis instead of letting the
    /// replacing rename overwrite it. It is bounded because corrupt data must
    /// not be able to grow the cache without limit, and reaching the bound is
    /// *reported* rather than silently dropping the evidence.
    pub max_quarantine_bytes: u64,
    /// Age at which an abandoned temporary file may be swept.
    pub temporary_grace: Duration,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_bundle_bytes: DEFAULT_MAX_BUNDLE_BYTES,
            max_sections: DEFAULT_MAX_SECTIONS,
            max_section_bytes: DEFAULT_MAX_SECTION_BYTES,
            max_quarantine_bytes: DEFAULT_MAX_QUARANTINE_BYTES,
            temporary_grace: DEFAULT_TEMPORARY_GRACE,
        }
    }
}

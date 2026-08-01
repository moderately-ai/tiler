//! Why a cache did what it did.
//!
//! ADR 0050 makes a corrupt, truncated, misplaced, or schema-invalid entry a
//! *miss*, and the argument is in the record: treating cache failure as
//! compilation failure "would make an optional accelerator a correctness
//! dependency". That is control flow, not silence.
//!
//! This module is the difference. Every miss this crate produces carries a
//! [`MissReason`], every refused publication carries a [`PublicationRefusal`],
//! and both reach the caller inside a [`CacheReport`] attached to whatever the
//! cache returned. A cache that is permanently rejecting every entry — because
//! a disk is failing, because two Tiler versions disagree about a schema,
//! because a directory is not writable — is therefore observable rather than
//! merely slow. A rejection that is not reported is a defect even though the
//! miss itself is correct.

use core::fmt;
use std::io;
use std::path::PathBuf;

use tiler_artifact::program::ArtifactCodecFailure;

use super::bundle::BundleRejection;

/// The namespace operation that failed.
///
/// Deliberately **not** `#[non_exhaustive]`: it is a closed vocabulary of the
/// operations this crate performs, and a caller that classifies one maps it
/// totally.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CacheOperation {
    /// Creating a shard directory of the namespace.
    CreateDirectory,
    /// Opening or creating a stable per-key lock file.
    OpenLock,
    /// Acquiring the exclusive advisory lock.
    AcquireLock,
    /// Releasing the exclusive advisory lock.
    ReleaseLock,
    /// Opening a final entry for reading.
    OpenEntry,
    /// Reading a final entry's bytes.
    ReadEntry,
    /// Creating a unique temporary file.
    CreateTemporary,
    /// Writing the bundle into a temporary file.
    WriteTemporary,
    /// Re-opening a temporary file to validate the bytes actually on disk.
    ReopenTemporary,
    /// Synchronizing a temporary file under the `fsync` durability policy.
    SyncTemporary,
    /// Renaming a temporary file over the final entry.
    Publish,
    /// Synchronizing the containing entry directory after publication.
    SyncEntryDirectory,
    /// Moving a rejected entry aside before it is replaced.
    Quarantine,
    /// Removing a final entry during eviction.
    RemoveEntry,
    /// Listing a shard directory.
    ScanDirectory,
    /// Removing an abandoned temporary file.
    RemoveTemporary,
    /// Renaming the whole version namespace out of service, during a purge.
    RetireNamespace,
    /// Removing a namespace tree a purge has already retired.
    RemoveRetired,
}

impl CacheOperation {
    /// Returns this operation's stable lowercase identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CreateDirectory => "create-directory",
            Self::OpenLock => "open-lock",
            Self::AcquireLock => "acquire-lock",
            Self::ReleaseLock => "release-lock",
            Self::OpenEntry => "open-entry",
            Self::ReadEntry => "read-entry",
            Self::CreateTemporary => "create-temporary",
            Self::WriteTemporary => "write-temporary",
            Self::ReopenTemporary => "reopen-temporary",
            Self::SyncTemporary => "sync-temporary",
            Self::Publish => "publish",
            Self::SyncEntryDirectory => "sync-entry-directory",
            Self::Quarantine => "quarantine",
            Self::RemoveEntry => "remove-entry",
            Self::ScanDirectory => "scan-directory",
            Self::RemoveTemporary => "remove-temporary",
            Self::RetireNamespace => "retire-namespace",
            Self::RemoveRetired => "remove-retired",
        }
    }
}

impl fmt::Display for CacheOperation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One namespace operation that failed, with the path it was attempted on.
///
/// The originating [`io::Error`] stays reachable as this error's source rather
/// than being formatted into a message, so a caller can still match on the kind
/// it actually was.
#[derive(Debug)]
pub struct CacheUnavailable {
    operation: CacheOperation,
    path: PathBuf,
    source: io::Error,
}

impl CacheUnavailable {
    pub(crate) const fn new(operation: CacheOperation, path: PathBuf, source: io::Error) -> Self {
        Self {
            operation,
            path,
            source,
        }
    }

    /// Returns the operation that failed.
    #[must_use]
    pub const fn operation(&self) -> CacheOperation {
        self.operation
    }

    /// Returns the path the operation was attempted on.
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl fmt::Display for CacheUnavailable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "cache operation `{}` failed on {}: {}",
            self.operation,
            self.path.display(),
            self.source,
        )
    }
}

impl std::error::Error for CacheUnavailable {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Why a stored entry was refused.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a.
#[derive(Debug)]
#[non_exhaustive]
pub enum EntryRejection {
    /// The stored bytes are not a valid bundle for the requested key.
    Bundle(BundleRejection),
    /// The bundle framed an artifact envelope the artifact layer refuses.
    ///
    /// The classification is the artifact codec's own, unchanged: this crate
    /// does not re-implement, weaken, or re-word what
    /// [`tiler_artifact::program::decode_artifact`] decided.
    Payload(ArtifactCodecFailure),
    /// The stored entry exceeds the configured maximum bundle size.
    ///
    /// Distinct from [`BundleRejection::BundleTooLarge`], which is a *declared*
    /// length: this is the file refusing to fit before its own header has been
    /// read, so the bound is enforced on the bytes rather than on a claim about
    /// them.
    TooLarge {
        /// Bytes read before the bound was exceeded.
        found: u64,
        /// Configured maximum.
        limit: u64,
    },
}

impl fmt::Display for EntryRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bundle(rejection) => write!(formatter, "cache bundle refused: {rejection}"),
            Self::Payload(failure) => {
                write!(formatter, "cached artifact envelope refused: {failure}")
            }
            Self::TooLarge { found, limit } => write!(
                formatter,
                "a cache entry exceeds the configured maximum of {limit} bytes after {found}",
            ),
        }
    }
}

impl std::error::Error for EntryRejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Bundle(rejection) => Some(rejection),
            Self::Payload(failure) => Some(failure),
            Self::TooLarge { .. } => None,
        }
    }
}

/// Why a read did not produce a hit.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a.
#[derive(Debug)]
#[non_exhaustive]
pub enum MissReason {
    /// No entry exists at the content path.
    ///
    /// The ordinary miss, and the only one that is not evidence of a problem.
    Absent,
    /// An entry existed and was refused, with the boundary that refused it.
    Rejected(EntryRejection),
    /// The namespace itself could not be used.
    Unavailable(CacheUnavailable),
    /// This cache stores nothing, so there is no content path to read.
    ///
    /// Distinct from [`Self::Absent`], which reports that a *content path* held
    /// no entry. A cache built by
    /// [`ExpansionCache::disabled`](crate::expansion::ExpansionCache::disabled)
    /// has no root and therefore no content path at all, and reporting an
    /// absence would describe a lookup that never happened.
    Disabled,
}

impl fmt::Display for MissReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Absent => formatter.write_str("no cache entry exists for this key"),
            Self::Rejected(rejection) => write!(formatter, "{rejection}"),
            Self::Unavailable(unavailable) => write!(formatter, "{unavailable}"),
            Self::Disabled => {
                formatter.write_str("the expansion cache is disabled, so nothing is stored to read")
            }
        }
    }
}

impl std::error::Error for MissReason {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Absent | Self::Disabled => None,
            Self::Rejected(rejection) => Some(rejection),
            Self::Unavailable(unavailable) => Some(unavailable),
        }
    }
}

/// Why a validated result was not published.
///
/// Every variant leaves the caller with a validated artifact it may still embed.
/// None of them is a compilation failure.
///
/// **Strictly pre-publication.** Every variant here means the atomic rename did
/// not happen, so no content entry exists and the caller may rebuild and
/// republish freely. A failure *after* the rename cannot be reported through this
/// type, because the entry is already visible to other processes and describing
/// it as unpublished is a false statement about durable state; those are
/// [`CacheReport::durability_shortfall`] and [`CacheReport::cleanup_shortfall`].
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a.
#[derive(Debug)]
#[non_exhaustive]
pub enum PublicationRefusal {
    /// The namespace could not be used.
    Unavailable(CacheUnavailable),
    /// The completed bundle exceeds a configured bound.
    Oversize(BundleRejection),
    /// The temporary bundle did not validate through its own descriptor.
    ///
    /// This is the check that keeps a partially written or mis-encoded bundle
    /// out of the final path: the bytes validated are the bytes on disk, read
    /// back through a second descriptor, not the buffer this process wrote.
    TemporaryRejected(EntryRejection),
    /// `rename` would have crossed a filesystem boundary.
    ///
    /// Construction keeps the temporary under the same cache root, so this
    /// cannot arise from ordinary operation — a bind mount or a symbolic link
    /// inside the namespace can still produce it, and it is reported rather than
    /// assumed away.
    CrossesFilesystems {
        /// The temporary file that could not be renamed.
        temporary: PathBuf,
        /// The final path it would have been renamed to.
        entry: PathBuf,
    },
    /// This cache stores nothing, so no publication was attempted.
    ///
    /// The one variant here that is not a failure. Every other means a
    /// publication was tried and did not complete; this one means the caller
    /// asked for a cache that shares nothing, so there was never a temporary
    /// file, a lock, or a rename — see
    /// [`ExpansionCache::disabled`](crate::expansion::ExpansionCache::disabled).
    /// It is reported rather than left as `None`, because
    /// [`CacheReport::publication_refusal`] returning `None` states that the
    /// result *was* published.
    Disabled,
}

impl fmt::Display for PublicationRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable(unavailable) => write!(formatter, "{unavailable}"),
            Self::Oversize(rejection) => {
                write!(
                    formatter,
                    "the completed bundle was not published: {rejection}"
                )
            }
            Self::TemporaryRejected(rejection) => write!(
                formatter,
                "a temporary bundle failed its own validation and was not published: {rejection}",
            ),
            Self::CrossesFilesystems { temporary, entry } => write!(
                formatter,
                "publishing {} over {} would cross a filesystem boundary",
                temporary.display(),
                entry.display(),
            ),
            Self::Disabled => formatter.write_str(
                "the expansion cache is disabled, so the validated result was not stored",
            ),
        }
    }
}

impl std::error::Error for PublicationRefusal {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unavailable(unavailable) => Some(unavailable),
            Self::Oversize(rejection) => Some(rejection),
            Self::TemporaryRejected(rejection) => Some(rejection),
            Self::CrossesFilesystems { .. } | Self::Disabled => None,
        }
    }
}

/// What became of a rejected entry that a publication replaced.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a.
#[derive(Debug)]
#[non_exhaustive]
pub enum QuarantineOutcome {
    /// The rejected bytes were moved aside and kept.
    Retained {
        /// Where they were moved to.
        path: PathBuf,
    },
    /// The quarantine bound was already reached, so the bytes were not kept.
    ///
    /// Reported rather than silent: discarding evidence is a real cost, and a
    /// caller that sees this repeatedly is being told its quarantine bound is
    /// too small or that something is corrupting entries faster than they are
    /// being examined.
    BoundReached {
        /// Bytes already retained in this shard's quarantine.
        retained: u64,
        /// Configured maximum.
        limit: u64,
        /// Bytes of the entry that was therefore not kept.
        discarded: u64,
    },
    /// The rejected bytes could not be moved aside.
    ///
    /// Publication continues: the replacement is the correctness requirement and
    /// quarantine is diagnostics.
    Failed(CacheUnavailable),
}

impl fmt::Display for QuarantineOutcome {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Retained { path } => {
                write!(
                    formatter,
                    "a rejected entry was retained at {}",
                    path.display()
                )
            }
            Self::BoundReached {
                retained,
                limit,
                discarded,
            } => write!(
                formatter,
                "{discarded} bytes of a rejected entry were discarded: quarantine holds {retained} \
                 of a maximum {limit}",
            ),
            Self::Failed(unavailable) => {
                write!(
                    formatter,
                    "a rejected entry could not be retained: {unavailable}"
                )
            }
        }
    }
}

/// Everything the cache decided while serving one request.
///
/// Read it to explain an outcome. Each field is `None` exactly when its step did
/// not happen or did not refuse — a lock-free hit leaves every field `None`,
/// and a publication after a rejected entry fills three of them.
///
/// # Publication, durability, and cleanup are three facts
///
/// The atomic rename is the publication point: once it succeeds another process
/// may observe the valid immutable entry, and no later failure can undo that.
/// So the fields divide on which side of it they fall.
/// [`Self::publication_refusal`] can only be set before the rename and means no
/// content entry exists. [`Self::durability_shortfall`] and
/// [`Self::cleanup_shortfall`] can only be set after it and always accompany a
/// published entry — they weaken what is claimed *about* an entry that exists,
/// and never contradict its existence.
///
/// Reading them as one "did anything go wrong" flag loses exactly the
/// distinction they were separated to preserve: a refusal means rebuild and try
/// again, a shortfall means the entry is there and something about persisting or
/// tidying up did not complete.
#[derive(Debug, Default)]
pub struct CacheReport {
    lookup_miss: Option<MissReason>,
    recheck_miss: Option<MissReason>,
    publication_refusal: Option<PublicationRefusal>,
    durability_shortfall: Option<CacheUnavailable>,
    cleanup_shortfall: Option<CacheUnavailable>,
    quarantine: Option<QuarantineOutcome>,
}

impl CacheReport {
    /// Why the lock-free read did not hit, or `None` when it did.
    #[must_use]
    pub const fn lookup_miss(&self) -> Option<&MissReason> {
        self.lookup_miss.as_ref()
    }

    /// Why the post-lock recheck did not hit, or `None` when it hit or did not
    /// run.
    #[must_use]
    pub const fn recheck_miss(&self) -> Option<&MissReason> {
        self.recheck_miss.as_ref()
    }

    /// Why a validated result was not published, or `None` when it was.
    #[must_use]
    pub const fn publication_refusal(&self) -> Option<&PublicationRefusal> {
        self.publication_refusal.as_ref()
    }

    /// A published entry whose durability claim could not be completed.
    ///
    /// Set only after a successful rename, and only under
    /// [`Durability::Fsync`](crate::expansion::Durability::Fsync): the entry is
    /// published, valid, and readable by any process, and the operating system
    /// was not able to confirm the directory update is persisted. A power loss
    /// could therefore lose the entry that a reader can see right now.
    ///
    /// It is not a reason to republish. Republishing writes the same immutable
    /// content to the same content path and would meet the same failing
    /// filesystem.
    #[must_use]
    pub const fn durability_shortfall(&self) -> Option<&CacheUnavailable> {
        self.durability_shortfall.as_ref()
    }

    /// A published entry whose cleanup step did not complete.
    ///
    /// Set only after a successful rename. The entry is published and valid; a
    /// step after it — releasing the per-key lock — failed. Nothing about the
    /// stored content is in doubt, and the consequence is confined to the
    /// namespace's own housekeeping.
    #[must_use]
    pub const fn cleanup_shortfall(&self) -> Option<&CacheUnavailable> {
        self.cleanup_shortfall.as_ref()
    }

    /// What became of a rejected entry a publication replaced.
    #[must_use]
    pub const fn quarantine(&self) -> Option<&QuarantineOutcome> {
        self.quarantine.as_ref()
    }

    pub(crate) fn set_lookup_miss(&mut self, reason: MissReason) {
        self.lookup_miss = Some(reason);
    }

    pub(crate) fn set_recheck_miss(&mut self, reason: MissReason) {
        self.recheck_miss = Some(reason);
    }

    pub(crate) fn set_publication_refusal(&mut self, refusal: PublicationRefusal) {
        self.publication_refusal = Some(refusal);
    }

    pub(crate) fn set_durability_shortfall(&mut self, unavailable: CacheUnavailable) {
        self.durability_shortfall = Some(unavailable);
    }

    pub(crate) fn set_cleanup_shortfall(&mut self, unavailable: CacheUnavailable) {
        self.cleanup_shortfall = Some(unavailable);
    }

    pub(crate) fn set_quarantine(&mut self, outcome: QuarantineOutcome) {
        self.quarantine = Some(outcome);
    }
}

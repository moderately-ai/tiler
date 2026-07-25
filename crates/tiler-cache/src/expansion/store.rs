//! The protocol: lock-free validated read, locked recheck, atomic publication.

use core::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use tiler_artifact::program::{ArtifactCodecFailure, DecodedArtifact, decode_artifact};

use super::bundle::{self, BundleRejection};
use super::key::CacheKey;
use super::layout::Layout;
use super::limits::Limits;
use super::lock::KeyLock;
use super::report::{
    CacheOperation, CacheReport, CacheUnavailable, EntryRejection, MissReason, PublicationRefusal,
    QuarantineOutcome,
};
use super::subject::ComposedSubject;

/// Attempts to find an unused temporary path before giving up.
///
/// Uniqueness comes from `create_new`, so a collision means the nanosecond
/// nonce repeated within one process — vanishingly rare and, crucially,
/// self-correcting. A bound is still stated so a pathological filesystem
/// produces a reported refusal instead of an unbounded loop.
const TEMPORARY_ATTEMPTS: u32 = 8;

/// How hard a publication tries to persist.
///
/// Atomic visibility and durable persistence are different properties, and this
/// type is the seam between them. Neither policy changes what a reader accepts.
///
/// Deliberately **not** `#[non_exhaustive]`: the store maps it totally, and a
/// third policy must be a compile error at that match rather than a silent
/// fallback to the weaker one.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Durability {
    /// Write, validate, close, rename.
    ///
    /// A killed writer cannot expose a partial temporary file at the final path,
    /// and abandoned temporaries are ignored. **No operating-system or
    /// power-loss persistence is claimed.**
    ///
    /// This is the default because ADR 0050 recommends it, not because it has
    /// been measured against the alternative here; the research note's fourth
    /// follow-up gate holds that measurement open, and
    /// `measure-expansion-cache-durability-policies` owns it.
    #[default]
    ProcessCrash,
    /// Additionally synchronize the temporary file before the rename and the
    /// containing entry directory afterwards.
    ///
    /// This requests persistence of file bytes, file metadata, and the directory
    /// update through the operating system's APIs. It does **not** claim a
    /// Darwin drive-cache flush — `fsync(2)` there documents that data may
    /// remain in a device's volatile cache — and it does not claim equivalent
    /// behaviour on every filesystem.
    Fsync,
}

/// One validated cache entry.
///
/// Holding this value means the whole bundle frame validated against the
/// requested key *and* the carried envelope passed
/// [`tiler_artifact::program::decode_artifact`]. There is no way to construct
/// one otherwise.
#[derive(Debug)]
pub struct CachedEntry {
    key: CacheKey,
    subject: Vec<u8>,
    envelope: Vec<u8>,
    artifact: DecodedArtifact,
}

impl CachedEntry {
    /// Returns the key this entry is stored under.
    #[must_use]
    pub const fn key(&self) -> &CacheKey {
        &self.key
    }

    /// Returns the composed subject bytes this entry was published under.
    ///
    /// Carried so an entry can say what it claims to be. A reader has already
    /// proved this is the subject the key derives from — that check runs on
    /// every hit — so these are the exact bytes the producer named, not a
    /// separately recorded description of them that could disagree.
    ///
    /// Bytes rather than a [`ComposedSubject`], because a subject read off disk
    /// is untrusted: returning the composed type would assert this crate had
    /// re-derived its structure, and the re-derivation that actually ran hashed
    /// these bytes without parsing them.
    #[must_use]
    pub fn subject(&self) -> &[u8] {
        &self.subject
    }

    /// Returns the exact artifact envelope bytes, for embedding.
    #[must_use]
    pub fn envelope_bytes(&self) -> &[u8] {
        &self.envelope
    }

    /// Returns the decoded artifact.
    #[must_use]
    pub const fn artifact(&self) -> &DecodedArtifact {
        &self.artifact
    }
}

/// The result of a lock-free read.
#[derive(Debug)]
pub enum Lookup {
    /// A validated entry.
    ///
    /// Boxed because a decoded artifact is far larger than a miss reason, and an
    /// unboxed variant would make every miss carry the hit's footprint.
    Hit(Box<CachedEntry>),
    /// No validated entry, with the reason.
    Miss(MissReason),
}

/// What a `get_or_publish` call resolved to.
///
/// Every variant carries a validated artifact. The difference between them is
/// where it came from and whether it reached the cache.
#[derive(Debug)]
pub enum Resolution {
    /// A validated entry was already stored.
    Hit {
        /// The entry.
        entry: CachedEntry,
        /// What the cache decided on the way.
        report: CacheReport,
    },
    /// The caller's result was compiled and published.
    Published {
        /// The published entry, re-read and re-validated from its temporary
        /// file before publication.
        entry: CachedEntry,
        /// What the cache decided on the way.
        report: CacheReport,
    },
    /// The caller's result was compiled and validated but not published.
    ///
    /// The cache was unusable at some step. This is the fall-open path, and it
    /// is a complete success from the caller's point of view: the artifact is
    /// validated and embeddable, it simply was not stored.
    Uncached {
        /// The exact artifact envelope bytes the caller produced.
        envelope: Vec<u8>,
        /// The decoded artifact.
        artifact: DecodedArtifact,
        /// Why it was not published, among everything else the cache decided.
        report: CacheReport,
    },
}

/// Why a `get_or_publish` call could not produce a validated artifact at all.
///
/// Both variants are hard failures the caller must surface. Neither is a cache
/// problem: ADR 0050's fall-open rule covers the cache mechanism and explicitly
/// does not convert a compiler error or an invalid generated artifact into
/// success.
#[derive(Debug)]
pub enum PublishFailure<E> {
    /// The caller's build step failed.
    Build(E),
    /// The caller's build step produced bytes that are not a valid artifact.
    Artifact(ArtifactCodecFailure),
}

impl<E: fmt::Display> fmt::Display for PublishFailure<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Build(error) => write!(formatter, "the cached build step failed: {error}"),
            Self::Artifact(failure) => write!(
                formatter,
                "the cached build step produced an invalid artifact: {failure}",
            ),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for PublishFailure<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Build(error) => Some(error),
            Self::Artifact(failure) => Some(failure),
        }
    }
}

/// What an eviction found.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Eviction {
    /// An entry existed and was removed.
    Removed,
    /// No entry existed.
    Absent,
}

/// What a temporary-file sweep removed.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SweepReport {
    /// Abandoned temporary files removed.
    pub removed: u32,
    /// Bytes those files occupied.
    pub bytes: u64,
    /// Temporary files left in place because they are younger than the grace
    /// period.
    pub retained: u32,
}

/// One expansion cache rooted at a directory.
///
/// Opening one performs no I/O and creates no directory: a cache root that does
/// not exist yet is not an error, and one that can never be created is a miss
/// reported at the moment it is needed rather than a failure at construction.
#[derive(Clone, Debug)]
pub struct ExpansionCache {
    layout: Layout,
    limits: Limits,
    durability: Durability,
}

impl ExpansionCache {
    /// Opens a cache rooted at `root`.
    ///
    /// The root must be private to the user running Tiler. Integrity validation
    /// handles accidents, partial writes, and non-cooperating cleanup; it does
    /// not make a shared writable cache an adversarial boundary, because an
    /// attacker able to replace files can construct new internally consistent
    /// bytes.
    #[must_use]
    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self {
            layout: Layout::new(root.into()),
            limits: Limits::default(),
            durability: Durability::default(),
        }
    }

    /// Sets the durability policy.
    #[must_use]
    pub const fn with_durability(mut self, durability: Durability) -> Self {
        self.durability = durability;
        self
    }

    /// Sets the bounds reads and publications operate within.
    #[must_use]
    pub const fn with_limits(mut self, limits: Limits) -> Self {
        self.limits = limits;
        self
    }

    /// Returns the cache root.
    #[must_use]
    pub fn root(&self) -> &Path {
        self.layout.root()
    }

    /// Reads the entry for one composed subject, validating it completely.
    ///
    /// Lock-free. A reader takes no lock because a lock would not make unvalidated
    /// bytes correct and validation does not need one: the reader holds an open
    /// descriptor, and on the Unix and Darwin hosts this crate targets, an entry
    /// unlinked after that open stays readable through the descriptor.
    ///
    /// The parameter is a [`ComposedSubject`] and not a byte run, which is what
    /// makes under-keying unrepresentable here rather than merely documented: a
    /// caller cannot reach this with the backend compilation subject alone.
    #[must_use]
    pub fn lookup(&self, subject: &ComposedSubject) -> Lookup {
        let key = CacheKey::derive(subject);
        match self.read_entry(&key, &artifact_validator) {
            Ok(entry) => Lookup::Hit(Box::new(CachedEntry {
                key: entry.key,
                subject: entry.subject,
                envelope: entry.envelope,
                artifact: entry.payload,
            })),
            Err(reason) => Lookup::Miss(reason),
        }
    }

    /// Returns a validated artifact for `subject`, compiling and publishing one
    /// if the cache does not already hold it.
    ///
    /// # Errors
    ///
    /// Returns [`PublishFailure::Build`] when `build` fails and
    /// [`PublishFailure::Artifact`] when it succeeds but returns bytes that are
    /// not a valid artifact envelope. Nothing about the cache itself produces an
    /// error here: every cache problem resolves to
    /// [`Resolution::Uncached`] carrying the reason.
    pub fn get_or_publish<E>(
        &self,
        subject: &ComposedSubject,
        build: impl FnOnce() -> Result<Vec<u8>, E>,
    ) -> Result<Resolution, PublishFailure<E>> {
        Ok(
            match self.resolve(subject.as_bytes(), build, &artifact_validator)? {
                ProtocolOutcome::Hit {
                    entry,
                    report,
                    published,
                } => {
                    let entry = CachedEntry {
                        key: entry.key,
                        subject: entry.subject,
                        envelope: entry.envelope,
                        artifact: entry.payload,
                    };
                    if published {
                        Resolution::Published { entry, report }
                    } else {
                        Resolution::Hit { entry, report }
                    }
                }
                ProtocolOutcome::Uncached { entry, report } => Resolution::Uncached {
                    envelope: entry.envelope,
                    artifact: entry.payload,
                    report,
                },
            },
        )
    }

    /// Removes the entry for one key, holding that key's lock while doing so.
    ///
    /// Taking the lock is what serializes eviction against a writer: without it,
    /// a writer could publish between this call's decision and its unlink and
    /// have its fresh entry removed. The lock file itself is retained.
    ///
    /// # Errors
    ///
    /// Returns [`CacheUnavailable`] when the namespace could not be used.
    pub fn evict(&self, key: &CacheKey) -> Result<Eviction, CacheUnavailable> {
        self.prepare_directories(key)?;
        let lock = self.acquire_lock(key)?;
        let entry = self.layout.entry_path(key);
        let eviction = match fs::remove_file(&entry) {
            Ok(()) => Eviction::Removed,
            Err(error) if error.kind() == io::ErrorKind::NotFound => Eviction::Absent,
            Err(error) => {
                return Err(CacheUnavailable::new(
                    CacheOperation::RemoveEntry,
                    entry,
                    error,
                ));
            }
        };
        self.release_lock(lock, key)?;
        Ok(eviction)
    }

    /// Removes abandoned temporary files for one key that are older than the
    /// configured grace period.
    ///
    /// Held under the same per-key lock as publication, so a live writer's
    /// temporary can never be removed out from under it. The grace period is a
    /// second guard for the case the lock cannot cover: a temporary abandoned by
    /// a process that died still belongs to nobody, and one written moments ago
    /// might belong to a writer this process cannot see.
    ///
    /// # Errors
    ///
    /// Returns [`CacheUnavailable`] when the namespace could not be used.
    pub fn sweep_temporaries(&self, key: &CacheKey) -> Result<SweepReport, CacheUnavailable> {
        self.prepare_directories(key)?;
        let lock = self.acquire_lock(key)?;
        let directory = self.layout.temporary_dir(key);
        let mut report = SweepReport::default();
        let entries = fs::read_dir(&directory).map_err(|error| {
            CacheUnavailable::new(CacheOperation::ScanDirectory, directory.clone(), error)
        })?;
        let now = SystemTime::now();
        for entry in entries {
            let entry = entry.map_err(|error| {
                CacheUnavailable::new(CacheOperation::ScanDirectory, directory.clone(), error)
            })?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if !Layout::is_temporary_of(key, name) {
                continue;
            }
            let path = entry.path();
            let metadata = entry.metadata().map_err(|error| {
                CacheUnavailable::new(CacheOperation::ScanDirectory, path.clone(), error)
            })?;
            // A modification time the host cannot report, or one in the future,
            // leaves the age unknown. An unknown age is treated as *young*: the
            // cost of keeping an abandoned temporary is bounded disk use, and
            // the cost of removing a live one is a failed publication.
            let old_enough = metadata
                .modified()
                .ok()
                .and_then(|modified| now.duration_since(modified).ok())
                .is_some_and(|age| age >= self.limits.temporary_grace);
            if !old_enough {
                report.retained += 1;
                continue;
            }
            match fs::remove_file(&path) {
                Ok(()) => {
                    report.removed += 1;
                    report.bytes += metadata.len();
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(CacheUnavailable::new(
                        CacheOperation::RemoveTemporary,
                        path,
                        error,
                    ));
                }
            }
        }
        self.release_lock(lock, key)?;
        Ok(report)
    }

    // -- the protocol, over any payload validator -------------------------
    //
    // The public entry points above pin the validator to the artifact decoder,
    // so no caller can weaken what a hit means. These take it as a parameter so
    // the protocol itself — exclusion, recheck, publication, replacement — can
    // be exercised without constructing a real artifact envelope, which needs a
    // semantic program and therefore a crate this one deliberately does not
    // depend on.

    pub(crate) fn resolve<T, E>(
        &self,
        subject: &[u8],
        build: impl FnOnce() -> Result<Vec<u8>, E>,
        validate: &dyn Fn(&[u8]) -> Result<T, ArtifactCodecFailure>,
    ) -> Result<ProtocolOutcome<T>, PublishFailure<E>> {
        let key = CacheKey::derive_bytes(subject);
        let mut report = CacheReport::default();

        match self.read_entry(&key, validate) {
            Ok(entry) => {
                return Ok(ProtocolOutcome::Hit {
                    entry,
                    report,
                    published: false,
                });
            }
            Err(reason) => report.set_lookup_miss(reason),
        }

        // Everything from here can fail open. `lock` is `None` when the
        // namespace could not be prepared or locked, and a publication is not
        // attempted without it.
        let lock = match self.prepare_directories(&key).and_then(|()| {
            let lock = self.acquire_lock(&key)?;
            Ok(lock)
        }) {
            Ok(lock) => Some(lock),
            Err(unavailable) => {
                report.set_publication_refusal(PublicationRefusal::Unavailable(unavailable));
                None
            }
        };

        // The recheck is the whole reason the lock is taken before compiling:
        // another process may have published while this one waited for it.
        let mut replacing_rejected_entry = false;
        if lock.is_some() {
            match self.read_entry(&key, validate) {
                Ok(entry) => {
                    return Ok(ProtocolOutcome::Hit {
                        entry,
                        report,
                        published: false,
                    });
                }
                Err(reason) => {
                    replacing_rejected_entry = matches!(reason, MissReason::Rejected(_));
                    report.set_recheck_miss(reason);
                }
            }
        }

        let envelope = build().map_err(PublishFailure::Build)?;
        // A build failure and an invalid generated artifact are hard errors that
        // the cache's fall-open rule does not cover.
        let payload = validate(&envelope).map_err(PublishFailure::Artifact)?;
        let entry = ValidatedEntry {
            key,
            subject: subject.to_vec(),
            envelope,
            payload,
        };

        let Some(lock) = lock else {
            return Ok(ProtocolOutcome::Uncached { entry, report });
        };

        match self.publish(
            &entry.key,
            subject,
            &entry.envelope,
            validate,
            replacing_rejected_entry,
        ) {
            Ok(quarantine) => {
                if let Some(outcome) = quarantine {
                    report.set_quarantine(outcome);
                }
                if let Err(unavailable) = self.release_lock(lock, &entry.key) {
                    report.set_publication_refusal(PublicationRefusal::Unavailable(unavailable));
                }
                Ok(ProtocolOutcome::Hit {
                    entry,
                    report,
                    published: true,
                })
            }
            Err(refusal) => {
                report.set_publication_refusal(refusal);
                drop(lock);
                Ok(ProtocolOutcome::Uncached { entry, report })
            }
        }
    }

    /// Reads and completely validates the final entry for one key.
    pub(crate) fn read_entry<T>(
        &self,
        key: &CacheKey,
        validate: &dyn Fn(&[u8]) -> Result<T, ArtifactCodecFailure>,
    ) -> Result<ValidatedEntry<T>, MissReason> {
        let path = self.layout.entry_path(key);
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(MissReason::Absent);
            }
            Err(error) => {
                return Err(MissReason::Unavailable(CacheUnavailable::new(
                    CacheOperation::OpenEntry,
                    path,
                    error,
                )));
            }
        };
        let bytes = self.read_bounded(&path, file)?;
        // Validate the *path* the bytes were found at as well as the bytes. A
        // bundle whose own frame is perfect but which sits in the wrong shard is
        // misplaced, and the entry-path parser is where that is caught.
        match super::layout::key_of_entry_path(&path) {
            Ok(parsed) if parsed == *key => {}
            Ok(parsed) => {
                return Err(MissReason::Rejected(EntryRejection::Bundle(
                    BundleRejection::KeyMismatch {
                        requested: key.label(),
                        embedded: parsed.label(),
                    },
                )));
            }
            Err(_) => {
                // Unreachable for a path this crate constructed from a key, and
                // propagated rather than asserted: a panic on a hostile
                // directory layout would be the wrong failure mode even for a
                // branch that should not arise.
                return Err(MissReason::Rejected(EntryRejection::Bundle(
                    BundleRejection::Magic,
                )));
            }
        }
        let view = bundle::decode(&bytes, key, &self.limits)
            .map_err(|rejection| MissReason::Rejected(EntryRejection::Bundle(rejection)))?;
        let payload = validate(view.envelope)
            .map_err(|failure| MissReason::Rejected(EntryRejection::Payload(failure)))?;
        Ok(ValidatedEntry {
            key: view.key,
            subject: view.subject.to_vec(),
            envelope: view.envelope.to_vec(),
            payload,
        })
    }

    /// Reads a file, refusing rather than allocating past the configured bound.
    fn read_bounded(&self, path: &Path, file: File) -> Result<Vec<u8>, MissReason> {
        let limit = self.limits.max_bundle_bytes;
        let mut bytes = Vec::new();
        // One byte past the bound, so a file exactly at the bound is accepted
        // and the first byte over it is observed rather than inferred from a
        // separately-read length that a concurrent writer could have changed.
        let mut read = file.take(limit.saturating_add(1));
        read.read_to_end(&mut bytes).map_err(|error| {
            MissReason::Unavailable(CacheUnavailable::new(
                CacheOperation::ReadEntry,
                path.to_path_buf(),
                error,
            ))
        })?;
        if bytes.len() as u64 > limit {
            return Err(MissReason::Rejected(EntryRejection::TooLarge {
                found: bytes.len() as u64,
                limit,
            }));
        }
        Ok(bytes)
    }

    /// Encodes, writes, re-validates, and atomically publishes one bundle.
    fn publish<T>(
        &self,
        key: &CacheKey,
        subject: &[u8],
        envelope: &[u8],
        validate: &dyn Fn(&[u8]) -> Result<T, ArtifactCodecFailure>,
        replacing_rejected_entry: bool,
    ) -> Result<Option<QuarantineOutcome>, PublicationRefusal> {
        let (encoded_key, encoded) = bundle::encode(subject, envelope, &self.limits)
            .map_err(PublicationRefusal::Oversize)?;
        debug_assert_eq!(
            encoded_key, *key,
            "the bundle encoder derives the key from the same subject this call did",
        );

        let (temporary, mut file) = self.create_temporary(key)?;
        let write = file
            .write_all(&encoded)
            .and_then(|()| file.flush())
            .map_err(|error| {
                PublicationRefusal::Unavailable(CacheUnavailable::new(
                    CacheOperation::WriteTemporary,
                    temporary.clone(),
                    error,
                ))
            });
        if let Err(refusal) = write {
            remove_abandoned(&temporary);
            return Err(refusal);
        }

        // Read the bytes back through a *separate* descriptor. Validating the
        // buffer this process just wrote would prove only that the encoder
        // agrees with the decoder; validating the file proves what the next
        // reader will see.
        if let Err(refusal) = self.validate_written(&temporary, key, validate) {
            remove_abandoned(&temporary);
            return Err(refusal);
        }

        if self.durability == Durability::Fsync
            && let Err(error) = file.sync_all()
        {
            remove_abandoned(&temporary);
            return Err(PublicationRefusal::Unavailable(CacheUnavailable::new(
                CacheOperation::SyncTemporary,
                temporary,
                error,
            )));
        }
        drop(file);

        let entry = self.layout.entry_path(key);
        let quarantine = if replacing_rejected_entry {
            Some(self.quarantine(key, &entry))
        } else {
            None
        };

        if let Err(error) = fs::rename(&temporary, &entry) {
            remove_abandoned(&temporary);
            if crosses_filesystems(&error) {
                return Err(PublicationRefusal::CrossesFilesystems { temporary, entry });
            }
            return Err(PublicationRefusal::Unavailable(CacheUnavailable::new(
                CacheOperation::Publish,
                entry,
                error,
            )));
        }

        if self.durability == Durability::Fsync {
            let directory = entry
                .parent()
                .expect("an entry path always has a shard directory")
                .to_path_buf();
            let synced = File::open(&directory).and_then(|handle| handle.sync_all());
            if let Err(error) = synced {
                // The entry is already published and valid. Failing to persist
                // the directory update weakens the durability claim and does not
                // unpublish anything, so it is reported rather than rolled back.
                return Err(PublicationRefusal::Unavailable(CacheUnavailable::new(
                    CacheOperation::SyncEntryDirectory,
                    directory,
                    error,
                )));
            }
        }
        Ok(quarantine)
    }

    fn validate_written<T>(
        &self,
        temporary: &Path,
        key: &CacheKey,
        validate: &dyn Fn(&[u8]) -> Result<T, ArtifactCodecFailure>,
    ) -> Result<(), PublicationRefusal> {
        let handle = File::open(temporary).map_err(|error| {
            PublicationRefusal::Unavailable(CacheUnavailable::new(
                CacheOperation::ReopenTemporary,
                temporary.to_path_buf(),
                error,
            ))
        })?;
        let bytes = self.read_bounded(temporary, handle).map_err(|reason| {
            match reason {
                MissReason::Unavailable(unavailable) => {
                    PublicationRefusal::Unavailable(unavailable)
                }
                MissReason::Rejected(rejection) => PublicationRefusal::TemporaryRejected(rejection),
                // A file this call just created cannot be absent from its own
                // read; the read reports absence only through `File::open`,
                // which succeeded above.
                MissReason::Absent => PublicationRefusal::TemporaryRejected(
                    EntryRejection::Bundle(BundleRejection::Magic),
                ),
            }
        })?;
        let view = bundle::decode(&bytes, key, &self.limits).map_err(|rejection| {
            PublicationRefusal::TemporaryRejected(EntryRejection::Bundle(rejection))
        })?;
        validate(view.envelope).map_err(|failure| {
            PublicationRefusal::TemporaryRejected(EntryRejection::Payload(failure))
        })?;
        Ok(())
    }

    /// Moves a rejected entry aside so the bytes that were refused survive.
    fn quarantine(&self, key: &CacheKey, entry: &Path) -> QuarantineOutcome {
        let Ok(metadata) = fs::metadata(entry) else {
            // Nothing to retain: the entry disappeared between the recheck and
            // now, which is the ordinary external-deletion race.
            return QuarantineOutcome::BoundReached {
                retained: 0,
                limit: self.limits.max_quarantine_bytes,
                discarded: 0,
            };
        };
        let directory = self.layout.quarantine_dir(key);
        if let Err(error) = fs::create_dir_all(&directory) {
            return QuarantineOutcome::Failed(CacheUnavailable::new(
                CacheOperation::Quarantine,
                directory,
                error,
            ));
        }
        let retained = retained_bytes(&directory);
        let discarded = metadata.len();
        if retained.saturating_add(discarded) > self.limits.max_quarantine_bytes {
            return QuarantineOutcome::BoundReached {
                retained,
                limit: self.limits.max_quarantine_bytes,
                discarded,
            };
        }
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        let path = directory.join(format!("{}.{nonce}.bundle", key.label()));
        match fs::rename(entry, &path) {
            Ok(()) => QuarantineOutcome::Retained { path },
            Err(error) => QuarantineOutcome::Failed(CacheUnavailable::new(
                CacheOperation::Quarantine,
                path,
                error,
            )),
        }
    }

    fn create_temporary(&self, key: &CacheKey) -> Result<(PathBuf, File), PublicationRefusal> {
        let mut last = None;
        for attempt in 0..TEMPORARY_ATTEMPTS {
            let path = self.layout.temporary_path(key, attempt);
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => return Ok((path, file)),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    last = Some((path, error));
                }
                Err(error) => {
                    return Err(PublicationRefusal::Unavailable(CacheUnavailable::new(
                        CacheOperation::CreateTemporary,
                        path,
                        error,
                    )));
                }
            }
        }
        let (path, error) = last.expect("at least one attempt is made");
        Err(PublicationRefusal::Unavailable(CacheUnavailable::new(
            CacheOperation::CreateTemporary,
            path,
            error,
        )))
    }

    pub(crate) fn prepare_directories(&self, key: &CacheKey) -> Result<(), CacheUnavailable> {
        let entry = self.layout.entry_path(key);
        let shard = entry
            .parent()
            .expect("an entry path always has a shard directory");
        for directory in [
            shard,
            self.layout
                .lock_path(key)
                .parent()
                .expect("a lock path always has a shard directory"),
            &self.layout.temporary_dir(key),
        ] {
            fs::create_dir_all(directory).map_err(|error| {
                CacheUnavailable::new(
                    CacheOperation::CreateDirectory,
                    directory.to_path_buf(),
                    error,
                )
            })?;
        }
        Ok(())
    }

    pub(crate) fn acquire_lock(&self, key: &CacheKey) -> Result<KeyLock, CacheUnavailable> {
        let path = self.layout.lock_path(key);
        KeyLock::acquire(&path)
            .map_err(|error| CacheUnavailable::new(CacheOperation::AcquireLock, path, error))
    }

    #[cfg(test)]
    pub(crate) fn lock_path(&self, key: &CacheKey) -> PathBuf {
        self.layout.lock_path(key)
    }

    #[cfg(test)]
    pub(crate) fn entry_path(&self, key: &CacheKey) -> PathBuf {
        self.layout.entry_path(key)
    }

    fn release_lock(&self, lock: KeyLock, key: &CacheKey) -> Result<(), CacheUnavailable> {
        lock.release().map_err(|error| {
            CacheUnavailable::new(
                CacheOperation::ReleaseLock,
                self.layout.lock_path(key),
                error,
            )
        })
    }
}

/// A validated entry, generic over what its payload validator produced.
#[derive(Debug)]
pub(crate) struct ValidatedEntry<T> {
    pub(crate) key: CacheKey,
    pub(crate) subject: Vec<u8>,
    pub(crate) envelope: Vec<u8>,
    pub(crate) payload: T,
}

/// What the crate-private protocol resolved to.
///
/// The public [`Resolution`] is this, with the payload type fixed to a decoded
/// artifact. Keeping the type parameter *off* the public surface is the point:
/// a public parameter would let a caller choose what a hit means, and pinning
/// the validator is exactly what makes a hit mean "this bundle framed an
/// artifact envelope the artifact layer accepted".
#[derive(Debug)]
pub(crate) enum ProtocolOutcome<T> {
    /// A validated entry is in the cache.
    Hit {
        entry: ValidatedEntry<T>,
        report: CacheReport,
        /// True when this call is what put it there.
        published: bool,
    },
    /// A validated result that the cache did not store.
    Uncached {
        entry: ValidatedEntry<T>,
        report: CacheReport,
    },
}

/// The validator the public API pins: no caller can substitute a weaker one.
fn artifact_validator(bytes: &[u8]) -> Result<DecodedArtifact, ArtifactCodecFailure> {
    decode_artifact(bytes)
}

/// Removes a temporary file this call created and will not publish.
///
/// Best effort by construction: an abandoned temporary is inert — it is under
/// `tmp/`, never at a content path, and swept later by
/// [`ExpansionCache::sweep_temporaries`] — so failing to remove one must not
/// turn into a second reported failure that hides the first.
fn remove_abandoned(path: &Path) {
    let _ = fs::remove_file(path);
}

/// Sums the bytes of one quarantine directory.
///
/// An unreadable directory or entry counts as zero rather than failing: this
/// feeds a diagnostic bound, and a quarantine that cannot be measured must not
/// stop a publication.
fn retained_bytes(directory: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(directory) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| entry.metadata().ok())
        .map(|metadata| metadata.len())
        .sum()
}

/// True when a `rename` failed because it would cross a filesystem boundary.
fn crosses_filesystems(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::CrossesDevices
}

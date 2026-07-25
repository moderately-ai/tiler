//! The internal per-key advisory-lock adapter.
//!
//! `docs/architecture.md` requires the adapter even though the pinned nightly
//! carries the Rust 1.89 `File::lock` API, and the reason is in the research
//! note: Rust documents that the mapping to a platform primitive "may change"
//! and that the lock may be advisory. Naming the primitive in one place is what
//! makes replacing it — for a platform whose semantics differ, or for an audited
//! adapter under a different minimum compiler version — a change to this file
//! rather than to the protocol.
//!
//! # There is no stale-lock recovery, deliberately
//!
//! The lock is held by an open descriptor, and the operating system releases it
//! when the last descriptor closes — including when the holder is killed. That
//! is the whole recovery story. A PID file or a timestamp lease would need a
//! rule for deciding that some other process is dead, and every such rule is
//! wrong under process-identifier reuse, a stopped process, or a clock that
//! moved.

use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

/// An exclusive advisory lock on one key's stable lock file.
#[derive(Debug)]
pub(crate) struct KeyLock {
    /// Holding the descriptor *is* holding the lock.
    held: File,
}

impl KeyLock {
    /// Opens the stable lock file for a key and blocks until the lock is held.
    ///
    /// The file is created if absent and never truncated: it carries no content
    /// and its inode identity is the whole of what it contributes.
    pub(crate) fn acquire(path: &Path) -> io::Result<Self> {
        let held = Self::open(path)?;
        held.lock()?;
        Ok(Self { held })
    }

    /// Takes the lock if it is free, without blocking.
    ///
    /// `Ok(None)` means another holder has it — which is the observation a test
    /// of mutual exclusion needs, and which the blocking form cannot report.
    /// The protocol itself always blocks: a writer that gave up on a contended
    /// key would rebuild exactly what the holder is about to publish.
    #[cfg(test)]
    pub(crate) fn try_acquire(path: &Path) -> io::Result<Option<Self>> {
        use std::fs::TryLockError;

        let held = Self::open(path)?;
        match held.try_lock() {
            Ok(()) => Ok(Some(Self { held })),
            Err(TryLockError::WouldBlock) => Ok(None),
            Err(TryLockError::Error(error)) => Err(error),
        }
    }

    /// Releases the lock, reporting a failure the implicit close would discard.
    ///
    /// Dropping a [`KeyLock`] also releases it. This form exists for the
    /// publication path, which has just finished writing and wants an unlock
    /// error to be visible rather than swallowed by a destructor.
    pub(crate) fn release(self) -> io::Result<()> {
        self.held.unlock()
    }

    fn open(path: &Path) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
    }
}

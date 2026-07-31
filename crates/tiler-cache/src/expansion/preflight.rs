//! A read-mostly probe of the locally decidable filesystem properties.
//!
//! [`ExpansionCache::preflight`] answers one question: does this root behave
//! the way the publication protocol assumes? It is shaped like
//! [`ExpansionCache::account`] — an explicit call that reports and decides
//! nothing — and it is deliberately not on the expansion path.
//!
//! # Why nothing calls it automatically
//!
//! An automatic probe would put filesystem writes on the lookup path to answer
//! a question whose answer does not change between lookups, and would make an
//! optional accelerator's diagnostics a cost every consumer pays. The
//! collection design eliminated an automatic trigger for the same reason and
//! the argument transfers unchanged: a caller that wants the answer asks.
//!
//! # Why it refuses nothing
//!
//! `docs/artifact-abi.md` records that only the lock property can fail
//! invisibly and that it costs duplicate compiler work rather than correctness
//! — every filesystem failure resolves to a miss, a reported unavailability, an
//! unpublished result, or repeated work. Refusing an unrecognized root would
//! make an optional accelerator a correctness dependency. The report is the
//! deliverable.
//!
//! # What it cannot answer, and says so
//!
//! Whether an advisory lock excludes a process on **another host** is not
//! decidable from one host. Darwin's `mount_nfs(8)` `locallocks` and Linux's
//! `nfs(5)` `local_lock=` both make a lock succeed while excluding only the
//! local client, so a passing lock row here is evidence about this host and
//! nothing else. [`PreflightReport::cross_host_exclusion_caveat`] exists
//! so that a reader cannot mistake the row for the stronger claim.
//!
//! # No `statfs`
//!
//! Identifying a filesystem by type is exactly what the supported-filesystem
//! contract eliminated in favour of deciding membership by property.
//! `MetadataExt::dev` gives the only identity these checks need and is safe, so
//! this module adds no `unsafe` and no new dependency.

use std::fs::{self, OpenOptions};
use std::io;
use std::os::unix::fs::MetadataExt as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::lock::KeyLock;
use super::store::ExpansionCache;

/// One probed property and what this host answered for it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreflightVerdict {
    /// The property held.
    Holds,
    /// The property was refuted.
    Refuted,
    /// The probe could not run, so the property is unknown rather than refuted.
    ///
    /// Distinct from [`Self::Refuted`] because the remedies differ: a refuted
    /// property means this root is unsuitable, while an unrunnable probe means
    /// nothing was learned — most often that the root is not writable, which is
    /// itself worth reporting rather than reading as a filesystem verdict.
    NotRun,
}

/// The properties the publication protocol rests on, as this root answered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreflightReport {
    root: PathBuf,
    same_device: PreflightVerdict,
    create_new_excludes: PreflightVerdict,
    lock_excludes_locally: PreflightVerdict,
    rename_publishes: PreflightVerdict,
    modification_time_reported: PreflightVerdict,
}

impl PreflightReport {
    /// The root these verdicts describe.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `entries/`, `tmp/`, and `locks/` share one device.
    ///
    /// The publication rename is only atomic within a filesystem, so a root
    /// straddling two makes every publication an `EXDEV` failure.
    #[must_use]
    pub const fn same_device(&self) -> PreflightVerdict {
        self.same_device
    }

    /// `create_new` refuses a path that already exists.
    ///
    /// Temporary uniqueness rests on this rather than on the nonce in the file
    /// name, which is a courtesy to a human reading the directory.
    #[must_use]
    pub const fn create_new_excludes(&self) -> PreflightVerdict {
        self.create_new_excludes
    }

    /// An exclusive advisory lock is acquirable and released on drop.
    ///
    /// **Local exclusion only.** See
    /// [`Self::cross_host_exclusion_caveat`].
    #[must_use]
    pub const fn lock_excludes_locally(&self) -> PreflightVerdict {
        self.lock_excludes_locally
    }

    /// A rename from `tmp/` into `entries/` succeeds.
    #[must_use]
    pub const fn rename_publishes(&self) -> PreflightVerdict {
        self.rename_publishes
    }

    /// A written file reports a modification time.
    ///
    /// Collection orders by it. A root that reports none does not break
    /// correctness; it makes eviction order arbitrary.
    #[must_use]
    pub const fn modification_time_reported(&self) -> PreflightVerdict {
        self.modification_time_reported
    }

    /// The caveat every report carries beside its lock row.
    ///
    /// An associated function rather than a method, because it is a property of
    /// the *probe* and not of any one report: no probe on one host can decide
    /// whether an advisory lock excludes a process on another. A lock taken
    /// under a mount that arbitrates locally succeeds and reports success, so
    /// [`Self::lock_excludes_locally`] holding is not evidence that a second
    /// host is excluded, and it never will be from here.
    ///
    /// Returned as text rather than as a constant `true` so that a caller
    /// rendering the report prints the caveat instead of having to know it.
    #[must_use]
    pub const fn cross_host_exclusion_caveat() -> &'static str {
        "the lock row is evidence about this host only: a mount that arbitrates \
         locks locally reports success while excluding no other host, and no \
         probe run from one host can detect that"
    }

    /// Whether every probe that ran reported its property holding.
    ///
    /// `NotRun` does not count as holding: a report where nothing ran would
    /// otherwise read as a clean bill of health, which is the shape of vacuous
    /// pass this whole module is written to avoid.
    #[must_use]
    pub fn all_probed_properties_hold(&self) -> bool {
        [
            self.same_device,
            self.create_new_excludes,
            self.lock_excludes_locally,
            self.rename_publishes,
            self.modification_time_reported,
        ]
        .iter()
        .all(|verdict| *verdict == PreflightVerdict::Holds)
    }
}

impl ExpansionCache {
    /// Probes this root's filesystem properties, changing nothing that lasts.
    ///
    /// Creates its own probe files under the cache namespace and removes them,
    /// so a completed call leaves the root as it found it. It touches no cache
    /// entry, takes no key lock, and is never called by `lookup`,
    /// `get_or_publish`, or `resolve`.
    ///
    /// A probe that cannot run reports [`PreflightVerdict::NotRun`] rather than
    /// failing: an unwritable root is a fact about the root worth reporting,
    /// not an error that should deny a caller the rows that did run.
    #[must_use]
    pub fn preflight(&self) -> PreflightReport {
        let namespace = self.layout().version_root();
        let area = namespace.join("preflight");
        let prepared = fs::create_dir_all(&area).is_ok();

        let report = PreflightReport {
            root: self.layout().root().to_path_buf(),
            same_device: if prepared {
                probe_same_device(&namespace, &area)
            } else {
                PreflightVerdict::NotRun
            },
            create_new_excludes: verdict(prepared, || probe_create_new(&area)),
            lock_excludes_locally: verdict(prepared, || probe_lock(&area)),
            rename_publishes: verdict(prepared, || probe_rename(&area)),
            modification_time_reported: verdict(prepared, || probe_modification_time(&area)),
        };
        // Best effort: leaving a probe directory behind costs a caller nothing
        // and failing to remove it is not a property of the protocol.
        let _ = fs::remove_dir_all(&area);
        report
    }
}

/// Runs `probe` when the area was prepared, mapping an I/O failure to `NotRun`.
fn verdict(prepared: bool, probe: impl FnOnce() -> io::Result<bool>) -> PreflightVerdict {
    if !prepared {
        return PreflightVerdict::NotRun;
    }
    match probe() {
        Ok(true) => PreflightVerdict::Holds,
        Ok(false) => PreflightVerdict::Refuted,
        Err(_) => PreflightVerdict::NotRun,
    }
}

/// `create_new` on an existing path must fail with `AlreadyExists`.
fn probe_create_new(area: &Path) -> io::Result<bool> {
    let path = area.join("create-new.probe");
    let _ = fs::remove_file(&path);
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)?;
    let second = OpenOptions::new().write(true).create_new(true).open(&path);
    let refused = matches!(
        second.map_err(|error| error.kind()),
        Err(io::ErrorKind::AlreadyExists)
    );
    fs::remove_file(&path)?;
    Ok(refused)
}

/// A lock must be takeable, must exclude a second attempt while held, and must
/// be free again once dropped.
///
/// The middle step is what makes this more than a smoke test: a `try_acquire`
/// that returned a second holder would mean the primitive reports success
/// without excluding anything, which is exactly the silent failure the
/// supported-filesystem contract names.
fn probe_lock(area: &Path) -> io::Result<bool> {
    let path = area.join("lock.probe");
    let held = KeyLock::acquire(&path)?;
    let contended = KeyLock::try_acquire(&path)?.is_none();
    drop(held);
    let reacquired = KeyLock::try_acquire(&path)?.is_some();
    Ok(contended && reacquired)
}

/// A rename from the temporary tree into the entries tree must succeed and must
/// replace whatever was there.
fn probe_rename(area: &Path) -> io::Result<bool> {
    let source = area.join("rename.source.probe");
    let destination = area.join("rename.destination.probe");
    fs::write(&source, b"new")?;
    fs::write(&destination, b"old")?;
    fs::rename(&source, &destination)?;
    let replaced = fs::read(&destination)? == b"new";
    let source_gone = !source.exists();
    fs::remove_file(&destination)?;
    Ok(replaced && source_gone)
}

/// A written file must report a modification time the collector can order by.
fn probe_modification_time(area: &Path) -> io::Result<bool> {
    let path = area.join("mtime.probe");
    fs::write(&path, b"probe")?;
    let reported = fs::metadata(&path)?
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .is_some();
    fs::remove_file(&path)?;
    // Referenced so an unused-import warning cannot hide a future change that
    // stops consulting the clock at all.
    let _ = SystemTime::now();
    Ok(reported)
}

/// The three trees a publication moves between must share one device.
fn probe_same_device(namespace: &Path, area: &Path) -> PreflightVerdict {
    let mut devices = Vec::new();
    for directory in ["entries", "tmp", "locks"] {
        let path = namespace.join(directory);
        // Created rather than required: an unused cache has not made them
        // yet, and their absence is not evidence about the filesystem.
        if fs::create_dir_all(&path).is_err() {
            return PreflightVerdict::NotRun;
        }
        match fs::metadata(&path) {
            Ok(metadata) => devices.push(metadata.dev()),
            Err(_) => return PreflightVerdict::NotRun,
        }
    }
    match fs::metadata(area).map(|metadata| metadata.dev()) {
        Ok(device) => devices.push(device),
        Err(_) => return PreflightVerdict::NotRun,
    }
    if devices.windows(2).all(|pair| pair[0] == pair[1]) {
        PreflightVerdict::Holds
    } else {
        PreflightVerdict::Refuted
    }
}

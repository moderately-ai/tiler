//! The cache namespace, and the parser that reads a key back out of a path.

use core::fmt;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use super::key::{CacheKey, KeyTextRejection};

/// Namespace version directory. Bumped when the layout itself changes, so an
/// older Tiler's entries are never read by a newer one that lays them out
/// differently — they are simply in a directory it does not look in.
pub(crate) const NAMESPACE_VERSION: &str = "v1";

const ENTRIES_DIR: &str = "entries";
const LOCKS_DIR: &str = "locks";
const TEMPORARIES_DIR: &str = "tmp";
const QUARANTINE_DIR: &str = "quarantine";

const BUNDLE_EXTENSION: &str = "bundle";
const LOCK_EXTENSION: &str = "lock";
const TEMPORARY_EXTENSION: &str = "tmp";

/// Leading characters of a key that name its shard directory.
const SHARD_BYTES: usize = 2;

/// The paths of one cache root.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Layout {
    root: PathBuf,
}

impl Layout {
    pub(crate) const fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// `<root>/v1/entries/<K[0..2]>/<K>.bundle`
    pub(crate) fn entry_path(&self, key: &CacheKey) -> PathBuf {
        let label = key.label();
        self.shard(ENTRIES_DIR, &label)
            .join(format!("{label}.{BUNDLE_EXTENSION}"))
    }

    /// `<root>/v1/locks/<K[0..2]>/<K>.lock`
    ///
    /// A stable namespace object rather than a cache entry. Nothing in this
    /// crate unlinks one: unlinking a locked file lets a later process create a
    /// different inode at the same path and take an independent lock while the
    /// first process still holds the first inode, which silently splits
    /// contenders into two groups that do not exclude each other.
    pub(crate) fn lock_path(&self, key: &CacheKey) -> PathBuf {
        let label = key.label();
        self.shard(LOCKS_DIR, &label)
            .join(format!("{label}.{LOCK_EXTENSION}"))
    }

    /// `<root>/v1/tmp/<K[0..2]>/`
    pub(crate) fn temporary_dir(&self, key: &CacheKey) -> PathBuf {
        self.shard(TEMPORARIES_DIR, &key.label())
    }

    /// `<root>/v1/quarantine/<K[0..2]>/`
    pub(crate) fn quarantine_dir(&self, key: &CacheKey) -> PathBuf {
        self.shard(QUARANTINE_DIR, &key.label())
    }

    /// One candidate temporary path.
    ///
    /// The process identifier and the nonce are a courtesy to a human reading
    /// the directory, not the uniqueness mechanism: the caller creates the file
    /// with `create_new`, so uniqueness is established by the filesystem
    /// operation. Keeping the temporary under the same cache root is what makes
    /// a cross-filesystem rename impossible under normal operation — and the
    /// publication step still reports `EXDEV` rather than assuming it away.
    pub(crate) fn temporary_path(&self, key: &CacheKey, attempt: u32) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        self.temporary_dir(key).join(format!(
            "{}.{}.{nonce}.{attempt}.{TEMPORARY_EXTENSION}",
            key.label(),
            process::id(),
        ))
    }

    /// True when `name` is a temporary file belonging to `key`.
    pub(crate) fn is_temporary_of(key: &CacheKey, name: &str) -> bool {
        name.starts_with(&format!("{}.", key.label())) && name.ends_with(TEMPORARY_EXTENSION)
    }

    fn shard(&self, kind: &str, label: &str) -> PathBuf {
        self.root
            .join(NAMESPACE_VERSION)
            .join(kind)
            .join(&label[..SHARD_BYTES])
    }
}

/// Reads the key an entry path is filed under.
///
/// Every component is checked rather than assumed: the file name must be
/// `<K>.bundle` with `K` a fixed-width lowercase hexadecimal key, and the
/// containing directory must be exactly the first [`SHARD_BYTES`] characters of
/// that key. A bundle sitting under the wrong shard is *misplaced*, which
/// ADR 0050 makes a miss — a reader that ignored the shard would accept an
/// entry the writer filed somewhere the lock for that key does not protect.
pub(crate) fn key_of_entry_path(path: &Path) -> Result<CacheKey, PathRejection> {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(PathRejection::NotUnicode)?;
    let label = name
        .strip_suffix(&format!(".{BUNDLE_EXTENSION}"))
        .ok_or(PathRejection::Extension)?;
    let key = CacheKey::parse_label(label).map_err(PathRejection::Key)?;
    let shard = path
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .ok_or(PathRejection::NotUnicode)?;
    if shard != &label[..SHARD_BYTES] {
        return Err(PathRejection::Shard {
            expected: label[..SHARD_BYTES].to_owned(),
            found: shard.to_owned(),
        });
    }
    Ok(key)
}

/// Why a path is not the content path of a cache entry.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum PathRejection {
    /// A path component is not valid Unicode, so no key can be read from it.
    NotUnicode,
    /// The file name does not end in the bundle extension.
    Extension,
    /// The file stem is not the rendering of a cache key.
    Key(KeyTextRejection),
    /// The containing shard directory is not the key's own shard.
    Shard {
        /// The shard the key belongs in.
        expected: String,
        /// The shard the entry was found in.
        found: String,
    },
}

impl fmt::Display for PathRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotUnicode => {
                formatter.write_str("a cache entry path component is not valid Unicode")
            }
            Self::Extension => write!(
                formatter,
                "a cache entry file name must end in `.{BUNDLE_EXTENSION}`",
            ),
            Self::Key(rejection) => write!(formatter, "cache entry file stem: {rejection}"),
            Self::Shard { expected, found } => write!(
                formatter,
                "a cache entry belongs under shard `{expected}` and was found under `{found}`",
            ),
        }
    }
}

impl std::error::Error for PathRejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Key(rejection) => Some(rejection),
            Self::NotUnicode | Self::Extension | Self::Shard { .. } => None,
        }
    }
}

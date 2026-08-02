//! Running the build-time half, and reading back only what it wrote.
//!
//! # Why the producer is invoked through Cargo
//!
//! The producer is a target of `tiler-build`, and this suite is a target of
//! `tiler-runtime`. There is no way for one package's integration test to reach
//! another package's executable by path that is not a guess about the layout of
//! `target/` — and a guess that resolved to a stale binary would keep passing
//! after the producer changed. Asking Cargo for it is the one form that is
//! correct whether this suite was run as `cargo nextest run --workspace`, as
//! `-p tiler-runtime` alone, or from a cold target directory.
//!
//! The cost is one Cargo invocation per test process, which builds nothing after
//! the first. The alternative — re-executing *this* binary and calling the
//! producer in the child — is what the fixture must not do: this binary cannot
//! link `tiler-build`, and if it could, the separation the suite exists to
//! measure would be gone.
//!
//! # Why those invocations are serialized, and why nothing cheaper works
//!
//! nextest runs each test in its own **process**, so several `cargo run`
//! invocations contend for one `target/` at once. Cargo's build lock serializes
//! their builds and not their executions, so a sibling's relink replaces the
//! inode of an example binary that is already running and the kernel kills it —
//! `signal: 9 (SIGKILL)`, empty stdout, empty stderr, on a cold target directory
//! and never on a warm one. That is the hazard [AGENTS.md] names: a process that
//! re-executes a binary owns a private copy of it, because the shared Cargo
//! hardlink under `target/` is unlinked and relinked by sibling invocations.
//!
//! An exclusive advisory lock over the whole invocation is what closes it, and
//! the three cheaper shapes were each eliminated for a stated reason rather than
//! passed over. A `OnceLock` shares nothing: the thirteen cases are thirteen
//! processes, and an in-process cell is not a lock between them — this is the
//! decisive one. A nextest setup script would work under nextest and would leave
//! `cargo test` racing exactly as before, which makes a fixture correct only
//! under one runner. Copying the built example to a private path narrows the
//! window without closing it: the copy reads a file a sibling may be relinking,
//! and it still needs a contended `cargo build --example` first.
//!
//! The lock is held across the child's *execution* and not merely its build,
//! because the execution is the half that dies.
//!
//! [AGENTS.md]: ../../../../AGENTS.md
//!
//! # What is read back
//!
//! Two files per variant and nothing else. Nothing in this module reaches into
//! the producer's process, its exit code aside, and nothing carries a Rust value
//! across the boundary.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::sidecar::Sidecar;

/// One variant's transported bytes and the record written beside them.
pub struct Transported {
    /// The exact envelope bytes the producer wrote.
    pub bytes: Vec<u8>,
    /// The durable identity record the producer wrote beside them.
    pub sidecar: Sidecar,
}

/// One completed pair of producer runs, rooted at a directory this process owns.
pub struct Produced {
    root: PathBuf,
}

impl Produced {
    /// Returns one variant of one run.
    ///
    /// # Panics
    ///
    /// Panics when the producer did not write that variant, which is a defect in
    /// the producer rather than a case under test.
    pub fn variant(&self, run: &str, name: &str) -> Transported {
        let directory = self.root.join(run).join(name);
        let bytes = std::fs::read(directory.join("artifact.bin")).unwrap_or_else(|error| {
            panic!("the producer wrote no envelope for {run}/{name}: {error}")
        });
        let record =
            std::fs::read_to_string(directory.join("sidecar.txt")).unwrap_or_else(|error| {
                panic!("the producer wrote no sidecar for {run}/{name}: {error}")
            });
        Transported {
            bytes,
            sidecar: Sidecar::parse(&record),
        }
    }

    /// Removes this run's tree.
    ///
    /// Called at the end of a passing test and skipped by a panicking one, so a
    /// failure leaves the exact bytes it failed on under `CARGO_TARGET_TMPDIR`.
    pub fn discard(self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// The variant every sound case loads.
pub const SOUND: &str = "sound";

/// Runs the build-time producer twice and returns its output tree.
///
/// # Panics
///
/// Panics when Cargo cannot be run, when the producer exits nonzero, or when it
/// wrote no `run-a`. Each is a defect in the producer or the workspace rather
/// than a case under test, and the process output is included so the failure
/// says which.
pub fn produce() -> Produced {
    let base = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("identity-join");
    std::fs::create_dir_all(&base).expect("the producer's shared directory is creatable");
    // Keyed by process id so the *outputs* of concurrent cases never collide.
    // That was already true and is not what the lock below is for.
    let root = base.join(format!("{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the producer root is creatable");

    // One producer invocation at a time across every process running this
    // binary. See the module header for the failure this closes and for the
    // shapes that do not close it. The lock lives beside the per-process output
    // trees and outlives them, because a lock file a case deleted would let the
    // next pair of cases contend again.
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(base.join("producer.lock"))
        .expect("the producer lock file is creatable");
    lock.lock().expect("the producer lock is acquirable");

    let output = Command::new(env!("CARGO"))
        .args([
            "run",
            "--quiet",
            "--locked",
            "--offline",
            "--package",
            "tiler-build",
            "--example",
            "identity_join_producer",
            "--",
        ])
        .arg(&root)
        .current_dir(workspace_root())
        .output()
        .expect("cargo runs from the workspace root");
    // Released before the assertions rather than at the end of scope: everything
    // below reads this process's own tree, and holding the lock through a panic's
    // unwind would serialize a failure report for no reason.
    drop(lock);
    assert!(
        output.status.success(),
        "the build-time producer failed ({}):\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    // Named rather than inferred from "no error": a producer that exited zero
    // having written nothing would otherwise be indistinguishable from one that
    // wrote everything, and every case below reads through `variant`.
    assert!(
        root.join("run-a")
            .join(SOUND)
            .join("artifact.bin")
            .is_file(),
        "the producer exited zero and wrote no sound envelope under {}",
        root.display(),
    );
    Produced { root }
}

/// Returns the workspace root, from this package's own manifest directory.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

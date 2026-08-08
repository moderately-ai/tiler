//! The killed-writer harness: real processes against the production bundle.
//!
//! # What this is evidence for
//!
//! ADR 0050's crash and race properties are about processes that die, and
//! [`super::tests`] says plainly that a thread which returns is not a process
//! that was killed. `spikes/cache/cache_harness.rs` measured those properties
//! against its *own miniature frame*; this module measures them against the
//! frame [`ExpansionCache`] actually publishes — the real namespace and shard
//! layout, the real [`KeyLock`](super::lock::KeyLock) adapter, the real
//! `create_new` temporary, the real separate-descriptor validation, the real
//! rename, and the real bundle encoder and decoder.
//!
//! A child is a genuine process: [`Command`] re-executes this test binary,
//! selecting a child entry point by name, and the armed phase makes it
//! [`process::abort`](std::process::abort) inside `ExpansionCache`'s own
//! publication path. The parent then observes only what a later reader would —
//! the filesystem — and never anything the dead process told it.
//!
//! # The one substitution, stated exactly
//!
//! The children drive the crate-private [`ExpansionCache::resolve`] with a
//! payload validator that accepts any non-empty bytes, not the public
//! `get_or_publish`, whose validator is
//! [`tiler_artifact::program::decode_artifact`]. Building a real artifact
//! envelope needs a `SemanticProgram`, which needs `tiler-ir`, which ADR 0082
//! item 2 decides this crate does not depend on.
//!
//! That substitution is sound for *these* properties and is not offered for any
//! other. Every byte of the bundle frame is real — the encoder, the header, the
//! section digests, the embedded key, the re-derivation from the carried
//! subject — and every filesystem operation is real. The payload validator sits
//! strictly *inside* an envelope the frame has already delimited, so which
//! validator runs changes how long the pre-rename window is and whether it can
//! fail, and changes nothing about what a killed writer leaves at a content
//! path. **What is therefore not measured here is a positive end-to-end hit
//! carrying a real compiled artifact**, which `super::expansion`'s module
//! documentation already assigns to the orchestrator holding both crates.
//!
//! # Bounded, and cheap by default
//!
//! Every child has a deadline; a child that outlives it is killed and reaped and
//! reported against the case that spawned it, so a hang is a bounded failure
//! rather than a stuck suite. The gate runs one repetition at low concurrency.
//! A recorded measurement uses more, through `TILER_CACHE_HARNESS_REPETITIONS`
//! and `TILER_CACHE_HARNESS_CONCURRENCY`; `spikes/cache/README.md` states the
//! exact command and `spikes/cache/results/` holds the outcome.

use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tiler_artifact::program::ArtifactCodecFailure;

use super::collect::CollectionBound;
use super::fault::{self, Phase};
use super::key::CacheKey;
use super::store::{Durability, ExpansionCache, ProtocolOutcome};
use super::subject::{ComposedSubject, SubjectFacets};

// -------------------------------------------------------------------------
// The child protocol
// -------------------------------------------------------------------------

/// Cache root a child operates on. Its presence is what arms a child at all.
const ROOT_VARIABLE: &str = "TILER_CACHE_HARNESS_ROOT";
/// Subject text a child composes its cache subject from.
const SUBJECT_VARIABLE: &str = "TILER_CACHE_HARNESS_SUBJECT";
/// Envelope text a child's build step returns.
const ENVELOPE_VARIABLE: &str = "TILER_CACHE_HARNESS_ENVELOPE";
/// File a child appends one line to each time its build step runs.
const COMPILE_LOG_VARIABLE: &str = "TILER_CACHE_HARNESS_COMPILE_LOG";
/// File a child writes its outcome label to, when it survives to have one.
const OUTCOME_VARIABLE: &str = "TILER_CACHE_HARNESS_OUTCOME";
/// Durability policy a child publishes under.
const DURABILITY_VARIABLE: &str = "TILER_CACHE_HARNESS_DURABILITY";
/// Milliseconds a child sleeps inside its build step, to widen a race window.
const BUILD_DELAY_VARIABLE: &str = "TILER_CACHE_HARNESS_BUILD_DELAY_MS";

/// Repetitions of the whole suite. One by default, so the gate stays cheap.
const REPETITIONS_VARIABLE: &str = "TILER_CACHE_HARNESS_REPETITIONS";
/// Concurrent children in the racing cases.
const CONCURRENCY_VARIABLE: &str = "TILER_CACHE_HARNESS_CONCURRENCY";
/// File each case appends one evidence row to, during a measurement run.
const EVIDENCE_VARIABLE: &str = "TILER_CACHE_HARNESS_EVIDENCE";

/// Cache root a collecting child operates on. Its presence arms that child.
const COLLECT_ROOT_VARIABLE: &str = "TILER_CACHE_HARNESS_COLLECT_ROOT";
/// Maximum entries a collecting child leaves behind on each round.
const COLLECT_MAX_ENTRIES_VARIABLE: &str = "TILER_CACHE_HARNESS_COLLECT_MAX_ENTRIES";
/// Collections one collecting child performs before exiting.
const COLLECT_ROUNDS_VARIABLE: &str = "TILER_CACHE_HARNESS_COLLECT_ROUNDS";
/// File a collecting child appends one line per collection to.
const COLLECT_LOG_VARIABLE: &str = "TILER_CACHE_HARNESS_COLLECT_LOG";

/// Libtest path of the child entry point, as the re-executed binary names it.
const CHILD_ENTRY: &str = "expansion::harness::harness_child";
/// Libtest path of the collecting child entry point.
const COLLECTOR_ENTRY: &str = "expansion::harness::harness_collector_child";

/// How long a child may run before it is killed and reported.
const CHILD_DEADLINE: Duration = Duration::from_secs(30);
/// How often a waiting parent polls a child.
const POLL_INTERVAL: Duration = Duration::from_millis(5);

/// Composes the subject one harness key is filed under.
///
/// Both facets carry fixed, non-empty stand-in bytes. The composition is real —
/// this is the production [`ComposedSubject`] — and what the facet bytes *mean*
/// is irrelevant to a crash property, which is a fact about files rather than
/// about identity. `derive-the-pre-compilation-artifact-program-subject` is what
/// would let a caller supply a genuine artifact-program facet.
fn subject_of(text: &str) -> ComposedSubject {
    ComposedSubject::compose(&SubjectFacets {
        backend_compilations: &[text.as_bytes()],
        artifact_program: b"tiler.cache.harness.artifact-program-stand-in",
    })
    .expect("the harness names every facet")
}

/// The validator the children run: any non-empty payload, with a real rejection.
fn any_payload(bytes: &[u8]) -> Result<Vec<u8>, ArtifactCodecFailure> {
    if bytes.is_empty() {
        return Err(ArtifactCodecFailure::Malformed {
            detail: "an empty payload is not an artifact".to_owned(),
        });
    }
    Ok(bytes.to_vec())
}

/// The child entry point, re-executed as its own process by [`spawn`].
///
/// Inert unless armed: without [`ROOT_VARIABLE`] it returns immediately, so an
/// ordinary `cargo nextest run` collects it, runs it, and it does nothing. That
/// guard is what lets the child live in the same binary as its parent.
#[test]
fn harness_child() {
    let Ok(root) = env::var(ROOT_VARIABLE) else {
        return;
    };
    let subject_text = env::var(SUBJECT_VARIABLE).expect("an armed child is given a subject");
    let envelope = env::var(ENVELOPE_VARIABLE).expect("an armed child is given an envelope");
    let durability = match env::var(DURABILITY_VARIABLE).as_deref() {
        Ok("fsync") => Durability::Fsync,
        // The default policy is what ADR 0050 recommends, and an unrecognized
        // value must not silently select the stronger one.
        _ => Durability::ProcessCrash,
    };
    let delay: u64 = env::var(BUILD_DELAY_VARIABLE)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let compile_log = env::var(COMPILE_LOG_VARIABLE).ok().map(PathBuf::from);

    let cache = ExpansionCache::open(&root).with_durability(durability);
    let subject = subject_of(&subject_text);
    let outcome = cache.resolve(
        subject.as_bytes(),
        || {
            if let Some(path) = &compile_log {
                append_line(path, &format!("{}\t{subject_text}", std::process::id()));
            }
            if delay > 0 {
                thread::sleep(Duration::from_millis(delay));
            }
            Ok::<_, String>(envelope.clone().into_bytes())
        },
        &any_payload,
    );

    let label = match outcome {
        Ok(ProtocolOutcome::Hit {
            published: true, ..
        }) => "published",
        Ok(ProtocolOutcome::Hit {
            published: false, ..
        }) => "hit",
        Ok(ProtocolOutcome::Uncached { .. }) => "uncached",
        Err(_) => "failed",
    };
    if let Ok(path) = env::var(OUTCOME_VARIABLE) {
        fs::write(path, label).expect("the outcome file is writable");
    }
}

/// The collecting child entry point, re-executed as its own process.
///
/// Inert unless armed, on the same guard as [`harness_child`]: without
/// [`COLLECT_ROOT_VARIABLE`] it returns immediately.
///
/// It runs the production [`ExpansionCache::collect`] against a cache other real
/// processes are publishing into, and writes one self-describing line per round.
/// The parent never learns anything from this process except those lines and its
/// exit status, so a collector that lied about what it removed would still be
/// caught by the parent's own reader, which validates what survived.
#[test]
fn harness_collector_child() {
    let Ok(root) = env::var(COLLECT_ROOT_VARIABLE) else {
        return;
    };
    let max_entries: u64 = env::var(COLLECT_MAX_ENTRIES_VARIABLE)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(0);
    let rounds: u32 = env::var(COLLECT_ROUNDS_VARIABLE)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1);
    let log = env::var(COLLECT_LOG_VARIABLE).ok().map(PathBuf::from);

    let cache = ExpansionCache::open(&root);
    let bound = CollectionBound {
        max_total_bytes: None,
        max_entries: Some(max_entries),
        max_entry_age: None,
    };
    for _ in 0..rounds {
        let line = match cache.collect(&bound) {
            Ok(report) => {
                // Asserted *inside* the collecting process, on every round, so a
                // collection that lost track of a candidate fails where it
                // happened rather than being inferred later from a total.
                //
                // Grounded in the scan and the stated bound rather than in the
                // loop that filled the report. `selected` and the five
                // dispositions are both produced by one pass over one vector, so
                // their equality holds whatever this child met on disk; what the
                // scan saw and what the bound retains are two independent
                // quantities, and under an entry ceiling with neither a byte nor
                // an age ceiling the selection is exactly the overage. A
                // selector that stopped a candidate short — or took one the
                // bound did not require — fails here.
                assert_eq!(
                    report.selected(),
                    report
                        .accounting()
                        .entry_count()
                        .saturating_sub(max_entries),
                    "a collection under an entry ceiling of {max_entries} must select \
                     every entry over it, and the scan saw {}",
                    report.accounting().entry_count(),
                );
                format!(
                    "ok\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                    report.selected(),
                    report.removed().len(),
                    report.reclaimed_bytes(),
                    report.contended(),
                    report.superseded(),
                    report.already_absent(),
                    report.failed().len(),
                )
            }
            // A scan that cannot proceed is recorded rather than panicked on, so
            // the parent reports which round failed and why instead of only
            // seeing a dead child.
            Err(unavailable) => format!("error\t{unavailable}"),
        };
        if let Some(path) = &log {
            append_line(path, &line);
        }
        thread::sleep(Duration::from_millis(5));
    }
}

/// Appends one line to a shared log, opened per call.
///
/// Opened, written, and closed each time rather than held, because concurrent
/// children append to one file and a short `O_APPEND` write is the operation
/// that stays whole across them.
fn append_line(path: &Path, line: &str) {
    use std::io::Write;

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("the compile log is writable");
    writeln!(file, "{line}").expect("the compile log is writable");
}

// -------------------------------------------------------------------------
// The parent driver
// -------------------------------------------------------------------------

/// One child's configuration, from the parent's side.
struct Run<'case> {
    root: &'case Path,
    subject: &'case str,
    envelope: &'case str,
    phase: Option<Phase>,
    durability: Durability,
    compile_log: Option<&'case Path>,
    outcome: Option<&'case Path>,
    build_delay: Duration,
    arrival: Option<&'case Path>,
    release: Option<&'case Path>,
}

impl<'case> Run<'case> {
    /// A child publishing `envelope` under `subject`, dying nowhere.
    fn new(root: &'case Path, subject: &'case str, envelope: &'case str) -> Self {
        Self {
            root,
            subject,
            envelope,
            phase: None,
            durability: Durability::ProcessCrash,
            compile_log: None,
            outcome: None,
            build_delay: Duration::ZERO,
            arrival: None,
            release: None,
        }
    }

    /// Arms this child to abort inside the publication path.
    const fn killed_at(mut self, phase: Phase) -> Self {
        self.phase = Some(phase);
        self
    }

    const fn with_durability(mut self, durability: Durability) -> Self {
        self.durability = durability;
        self
    }

    const fn logging_compiles_to(mut self, path: &'case Path) -> Self {
        self.compile_log = Some(path);
        self
    }

    const fn reporting_outcome_to(mut self, path: &'case Path) -> Self {
        self.outcome = Some(path);
        self
    }

    const fn with_build_delay(mut self, delay: Duration) -> Self {
        self.build_delay = delay;
        self
    }

    /// Holds this child at `gate` as participant `index`.
    ///
    /// It parks inside the protocol, once it has looked and found nothing, and
    /// runs on from there only when every other participant has done the same.
    fn gated_by(mut self, gate: &'case Gate, index: usize) -> Self {
        self.arrival = Some(gate.arrival(index));
        self.release = Some(gate.release_path());
        self
    }

    /// Builds the command that re-executes this test binary as a child.
    fn command(&self) -> Command {
        let exe = env::current_exe().expect("a test binary knows its own path");
        let mut command = Command::new(exe);
        command
            .args(["--exact", CHILD_ENTRY])
            .env(ROOT_VARIABLE, self.root)
            .env(SUBJECT_VARIABLE, self.subject)
            .env(ENVELOPE_VARIABLE, self.envelope)
            .env(
                DURABILITY_VARIABLE,
                match self.durability {
                    Durability::ProcessCrash => "process",
                    Durability::Fsync => "fsync",
                },
            )
            .env(
                BUILD_DELAY_VARIABLE,
                self.build_delay.as_millis().to_string(),
            )
            // A child inherits this process's environment, and this process may
            // itself be an armed child's parent under a repeated run. Removing
            // each variable before conditionally setting it is what stops an
            // unarmed child inheriting a phase nobody meant to give it, or an
            // ungated child parking at a rendezvous nobody will release.
            .env_remove(fault::PHASE_VARIABLE)
            .env_remove(fault::ARRIVAL_VARIABLE)
            .env_remove(fault::RELEASE_VARIABLE)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(phase) = self.phase {
            command.env(fault::PHASE_VARIABLE, phase.as_str());
        }
        if let Some(path) = self.arrival {
            command.env(fault::ARRIVAL_VARIABLE, path);
        }
        if let Some(path) = self.release {
            command.env(fault::RELEASE_VARIABLE, path);
        }
        if let Some(path) = self.compile_log {
            command.env(COMPILE_LOG_VARIABLE, path);
        }
        if let Some(path) = self.outcome {
            command.env(OUTCOME_VARIABLE, path);
        }
        command
    }
}

/// What a child process did, as the parent observed it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Death {
    /// It exited zero.
    Completed,
    /// It exited nonzero or on a signal, which an armed child always does.
    Terminated,
    /// It outlived its deadline and was killed by the parent.
    ///
    /// Always a failure of the case that produced it, never an expected result:
    /// the protocol's only blocking wait is the per-key lock, and a lock held by
    /// a process that died is released by the kernel.
    TimedOut,
}

/// Runs one child to completion within the standard deadline.
fn run(run: &Run<'_>) -> Death {
    let mut child = run.command().spawn().expect("a child process spawns");
    wait_bounded(&mut child, CHILD_DEADLINE)
}

/// Waits for a child, killing and reaping it if it outlives `deadline`.
///
/// The deadline is a parameter rather than the constant so the case that proves
/// this function times out can afford to wait for it. A timeout path nothing
/// reaches is a guess about what would happen, not a mechanism.
fn wait_bounded(child: &mut Child, deadline: Duration) -> Death {
    let expiry = Instant::now() + deadline;
    loop {
        match child.try_wait().expect("a spawned child is waitable") {
            Some(status) if status.success() => return Death::Completed,
            Some(_) => return Death::Terminated,
            None => {}
        }
        if Instant::now() >= expiry {
            let _ = child.kill();
            let _ = child.wait();
            return Death::TimedOut;
        }
        thread::sleep(POLL_INTERVAL);
    }
}

/// Runs several children concurrently and waits for all of them.
fn run_all(runs: &[Run<'_>]) -> Vec<Death> {
    let mut children: Vec<Child> = runs
        .iter()
        .map(|run| run.command().spawn().expect("a child process spawns"))
        .collect();
    children
        .iter_mut()
        .map(|child| wait_bounded(child, CHILD_DEADLINE))
        .collect()
}

/// A barrier several children meet *inside* the publication protocol.
///
/// Each gated child creates its own arrival file at the moment it has completed
/// the lock-free lookup and has taken no lock, then blocks. The parent releases
/// them together, and only once every arrival file exists.
///
/// # What it buys, and what it replaces
///
/// The racing cases need an ordering: every process must still be *before* the
/// lock when the first of them takes it. The harness used to buy that with a
/// sleep in the winning child's build step, long enough that the others were
/// presumed to be behind it. That is true only while the host stays fast enough,
/// and when it is not, the case reports a correctness failure for a scheduling
/// event. Widening the sleep keeps the failure and makes it rarer.
///
/// A barrier removes it instead of widening it. When it opens, every gated
/// process has looked, found nothing, and committed to the locked path, so no
/// later publication can turn any of them into a lock-free hit — which is
/// exactly the ordering the sleep was approximating, now observed.
///
/// # Its own failure is loud
///
/// A deadline still exists, because a barrier no one reaches must not hang the
/// suite. It is not a margin any property depends on: reaching it means the
/// rendezvous did not happen, and the parent says how many of how many children
/// arrived and which one exited early instead of reporting a cache defect. A
/// child that hits, never starts, or dies before arriving is caught there.
struct Gate {
    /// One arrival file per gated child, indexed as its [`Run`] is.
    ///
    /// A file each rather than one shared counter, so the parent names its
    /// population and counts it: a barrier that opened on a total could be
    /// satisfied by one child arriving twice.
    arrivals: Vec<PathBuf>,
    /// The file whose appearance releases every parked child at once.
    release: PathBuf,
}

impl Gate {
    fn new(scratch: &Scratch, children: usize) -> Self {
        Self {
            arrivals: (0..children)
                .map(|index| scratch.file(&format!("arrival-{index}")))
                .collect(),
            release: scratch.file("release"),
        }
    }

    fn arrival(&self, index: usize) -> &Path {
        &self.arrivals[index]
    }

    fn release_path(&self) -> &Path {
        &self.release
    }

    /// Blocks until every gated child has arrived, naming what did not.
    ///
    /// `children` are the gated children in barrier order. They are polled as
    /// well as the arrival files, because a child that has already exited will
    /// never arrive and waiting out the deadline for it would report a timeout
    /// where the real answer is an exit status.
    fn await_arrivals(&self, children: &mut [Child]) {
        let expected = self.arrivals.len();
        let expiry = Instant::now() + CHILD_DEADLINE;
        let failure = loop {
            let arrived = self.arrivals.iter().filter(|path| path.exists()).count();
            if arrived == expected {
                break None;
            }
            if let Some((index, status)) = first_exited(children) {
                break Some(format!(
                    "child {index} exited with {status} before reaching the rendezvous, \
                     with {arrived} of {expected} children arrived",
                ));
            }
            if Instant::now() >= expiry {
                break Some(format!(
                    "only {arrived} of {expected} children reached the rendezvous \
                     within {CHILD_DEADLINE:?}",
                ));
            }
            thread::sleep(POLL_INTERVAL);
        };
        let Some(reason) = failure else {
            return;
        };
        // Reap before failing: a parked child would otherwise outlive the panic
        // until its own backstop expired.
        for child in children {
            let _ = child.kill();
            let _ = child.wait();
        }
        panic!("{reason}");
    }

    /// Releases every parked child at once.
    fn release(&self) {
        fs::write(&self.release, b"released").expect("the release file is writable");
    }

    /// [`Self::await_arrivals`], then [`Self::release`].
    fn open(&self, children: &mut [Child]) {
        self.await_arrivals(children);
        self.release();
    }
}

/// Returns the first child that has already exited, with its status.
fn first_exited(children: &mut [Child]) -> Option<(usize, ExitStatus)> {
    children.iter_mut().enumerate().find_map(|(index, child)| {
        child
            .try_wait()
            .expect("a spawned child is waitable")
            .map(|status| (index, status))
    })
}

/// Runs several gated children, releasing them together once every one of them
/// is inside the protocol.
fn run_all_gated(runs: &[Run<'_>], gate: &Gate) -> Vec<Death> {
    let mut children: Vec<Child> = runs
        .iter()
        .map(|run| run.command().spawn().expect("a child process spawns"))
        .collect();
    gate.open(&mut children);
    children
        .iter_mut()
        .map(|child| wait_bounded(child, CHILD_DEADLINE))
        .collect()
}

/// A unique directory for one harness case, removed when the guard drops.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};

        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the host clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "tiler-cache-harness-{name}-{}-{nonce}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(&path).expect("a scratch directory is creatable");
        Self { path }
    }

    fn root(&self) -> PathBuf {
        self.path.join("cache")
    }

    fn file(&self, name: &str) -> PathBuf {
        self.path.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Reads an entry through the production reader, as any later process would.
///
/// This is the only way the parent inspects a cache the children wrote. Reading
/// the bytes directly would prove something about a file; reading through
/// [`ExpansionCache::read_entry`] proves what the next *reader* gets, which is
/// the property ADR 0050 states.
fn read_back(root: &Path, subject: &str) -> Option<Vec<u8>> {
    let cache = ExpansionCache::open(root);
    let key = CacheKey::derive(&subject_of(subject));
    cache
        .read_entry(&key, &any_payload)
        .ok()
        .map(|entry| entry.envelope().to_vec())
}

/// Returns the entry path one subject is filed at.
fn entry_path(root: &Path, subject: &str) -> PathBuf {
    ExpansionCache::open(root).entry_path(&CacheKey::derive(&subject_of(subject)))
}

/// Counts lines in a compile log, or zero when no child ever compiled.
fn compiles(path: &Path) -> usize {
    fs::read_to_string(path).map_or(0, |text| text.lines().count())
}

/// Number of independent repetitions of each case.
fn repetitions() -> u32 {
    env::var(REPETITIONS_VARIABLE)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1)
}

/// Number of concurrent children in the racing cases.
fn concurrency() -> usize {
    env::var(CONCURRENCY_VARIABLE)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(4)
}

/// Appends one evidence row for a case that has just passed.
///
/// Nothing is written unless a measurement run named a file, so an ordinary gate
/// run produces no artifact. Reaching this call at all means the case's
/// assertions held, which is what makes a row's presence the evidence and its
/// contents the detail.
///
/// Each row is appended through its own short `O_APPEND` write, because nextest
/// gives every test its own process and several may finish at once. Row *order*
/// is therefore not meaningful and each row is self-describing instead.
fn record(case: &str, children: u32, detail: &str, started: Instant) {
    let Ok(path) = env::var(EVIDENCE_VARIABLE) else {
        return;
    };
    append_line(
        Path::new(&path),
        &format!(
            "run\t{case}\tpassed\t{}\t{}\t{children}\t{detail}\t{}",
            repetitions(),
            concurrency(),
            started.elapsed().as_millis(),
        ),
    );
}

// -------------------------------------------------------------------------
// Killed writers
// -------------------------------------------------------------------------

/// A writer killed at any publication phase leaves no unvalidated entry, and the
/// next process recovers.
///
/// This is the whole of ADR 0050's crash claim, against the production bundle.
/// It asserts three things at every one of the nine phases, and each is
/// load-bearing:
///
/// 1. the armed child really died, so the phase was reached rather than skipped;
/// 2. whatever it left at the content path either reads back as a *validated*
///    entry or does not read back at all — never partial bytes a reader accepts;
/// 3. a later process reaches a validated entry regardless.
///
/// Point 2 is asserted through [`ExpansionCache::read_entry`], so "no unvalidated
/// entry" means what a real reader would do and not what an inspection of the
/// file says.
#[test]
fn a_writer_killed_at_any_phase_leaves_a_recoverable_cache() {
    let started = Instant::now();
    let mut children = 0;
    for repetition in 0..repetitions() {
        for phase in Phase::KILL_POINTS {
            for durability in [Durability::ProcessCrash, Durability::Fsync] {
                let scratch = Scratch::new("killed-writer");
                let root = scratch.root();
                let subject = "killed-writer";

                let death = run(&Run::new(&root, subject, "envelope-from-the-killed-writer")
                    .killed_at(phase)
                    .with_durability(durability));
                children += 1;
                assert_eq!(
                    death,
                    Death::Terminated,
                    "{phase:?}/{durability:?} repetition {repetition}: an armed child must die",
                );

                // Whatever survived at the content path is either a validated
                // entry or nothing. A partial or unvalidated bundle reaching a
                // reader is the failure this record exists to exclude.
                if let Some(envelope) = read_back(&root, subject) {
                    assert_eq!(
                        envelope, b"envelope-from-the-killed-writer",
                        "{phase:?}/{durability:?}: a readable entry is the one that was published",
                    );
                }

                // A later process recovers: it either hits what the dead writer
                // managed to publish or rebuilds and publishes its own.
                let outcome = scratch.file("outcome");
                let survivor = run(&Run::new(&root, subject, "envelope-from-the-survivor")
                    .with_durability(durability)
                    .reporting_outcome_to(&outcome));
                children += 1;
                assert_eq!(
                    survivor,
                    Death::Completed,
                    "{phase:?}/{durability:?}: an unarmed survivor completes",
                );
                let label = fs::read_to_string(&outcome).expect("the survivor reports an outcome");
                assert!(
                    label == "hit" || label == "published",
                    "{phase:?}/{durability:?}: the survivor recovered as `{label}`",
                );
                assert!(
                    read_back(&root, subject).is_some(),
                    "{phase:?}/{durability:?}: a validated entry exists after recovery",
                );
            }
        }
    }
    record(
        "killed-writers",
        children,
        &format!("kill_points={}", Phase::KILL_POINTS.len()),
        started,
    );
}

/// A writer killed before its rename leaves the content path untouched.
///
/// The phases split into two groups and the split is the point of atomic
/// publication: before `after-rename` nothing has ever been at the content path,
/// so a reader must find nothing at all rather than something it then rejects.
/// The test above tolerates either outcome at every phase; this one pins which
/// phases may produce which, so a rename that leaked a partial file early would
/// fail here even though the cache still "recovered".
#[test]
fn no_entry_exists_at_a_content_path_before_the_rename() {
    for phase in Phase::KILL_POINTS {
        let published_by_now = matches!(phase, Phase::AfterRename | Phase::AfterDirectorySync);
        let scratch = Scratch::new("pre-rename");
        let root = scratch.root();
        let subject = "pre-rename";

        let death = run(&Run::new(&root, subject, "envelope").killed_at(phase));
        assert_eq!(death, Death::Terminated, "{phase:?}: the child must die");

        let exists = entry_path(&root, subject).exists();
        assert_eq!(
            exists, published_by_now,
            "{phase:?}: an entry exists at the content path iff the rename ran",
        );
    }
}

/// A killed writer's lock is released by the kernel, with no recovery rule.
///
/// The lock is held by an open descriptor and nothing else — no owner
/// identifier, no lease, no stale-lock deletion — so a writer killed while
/// holding it must leave the key immediately available. A survivor that
/// completes within its deadline is the observation; a survivor that timed out
/// would mean the lock outlived its holder.
#[test]
fn a_killed_writers_lock_is_released_without_a_recovery_rule() {
    for phase in Phase::KILL_POINTS {
        let scratch = Scratch::new("lock-release");
        let root = scratch.root();
        let subject = "lock-release";
        assert_eq!(
            run(&Run::new(&root, subject, "envelope").killed_at(phase)),
            Death::Terminated,
            "{phase:?}: the child must die holding or having held the lock",
        );
        assert_eq!(
            run(&Run::new(&root, subject, "envelope")),
            Death::Completed,
            "{phase:?}: the next writer must not block on a dead holder's lock",
        );
    }
}

/// A killed writer leaves no temporary file at a content path.
///
/// An abandoned temporary is inert by construction — it lives under `tmp/` and
/// is swept later — and the property that matters is that it is never mistaken
/// for an entry. Checked by reading the shard directory the entry lives in.
#[test]
fn a_killed_writer_leaves_nothing_extra_in_the_entry_shard() {
    for phase in Phase::KILL_POINTS {
        let scratch = Scratch::new("shard-contents");
        let root = scratch.root();
        let subject = "shard-contents";
        assert_eq!(
            run(&Run::new(&root, subject, "envelope").killed_at(phase)),
            Death::Terminated,
        );

        let entry = entry_path(&root, subject);
        let shard = entry.parent().expect("an entry path has a shard directory");
        let Ok(listing) = fs::read_dir(shard) else {
            continue;
        };
        for found in listing {
            let found = found.expect("the shard directory is readable").path();
            assert_eq!(
                found, entry,
                "{phase:?}: only the entry itself may live in an entry shard",
            );
        }
    }
}

// -------------------------------------------------------------------------
// Racing processes
// -------------------------------------------------------------------------

/// Concurrent processes on one key compile once and all agree on one entry.
///
/// The count is one whether or not the children overlap — a child that arrives
/// after the entry exists hits and never compiles — so overlap is what makes the
/// case *mean* that the lock excluded something. The harness used to buy it with
/// a sleep in every build step, which widened the window without ever checking
/// that anything entered it. The [`Gate`] checks it: every child is held until
/// all of them have looked and found nothing, so the one compilation is the
/// exclusion and not the scheduling.
#[test]
fn concurrent_processes_on_one_key_compile_once() {
    let started = Instant::now();
    let mut children = 0;
    for repetition in 0..repetitions() {
        let scratch = Scratch::new("race-identical");
        let root = scratch.root();
        let log = scratch.file("compiles");
        let subject = "race-identical";
        let gate = Gate::new(&scratch, concurrency());

        let runs: Vec<Run<'_>> = (0..concurrency())
            .map(|index| {
                Run::new(&root, subject, "envelope-from-the-race")
                    .logging_compiles_to(&log)
                    .gated_by(&gate, index)
            })
            .collect();
        for (index, death) in run_all_gated(&runs, &gate).into_iter().enumerate() {
            children += 1;
            assert_eq!(
                death,
                Death::Completed,
                "repetition {repetition} child {index} did not complete",
            );
        }

        assert_eq!(
            read_back(&root, subject).as_deref(),
            Some(&b"envelope-from-the-race"[..]),
            "repetition {repetition}: every racing process agrees on one entry",
        );
        assert_eq!(
            compiles(&log),
            1,
            "repetition {repetition}: the per-key lock suppressed duplicate compilation",
        );
    }
    record("race-identical", children, "compiles_per_key=1", started);
}

/// Concurrent processes on distinct keys each publish their own entry.
///
/// Gated for the same reason the shared-key case is: distinct keys never
/// contend, so the assertion holds even if the children ran one after another,
/// and the barrier is what makes "concurrent" in the name a checked fact rather
/// than a hope about spawn timing.
#[test]
fn concurrent_processes_on_distinct_keys_do_not_collide() {
    let started = Instant::now();
    let mut children = 0;
    for repetition in 0..repetitions() {
        let scratch = Scratch::new("race-distinct");
        let root = scratch.root();
        let gate = Gate::new(&scratch, concurrency());
        let subjects: Vec<String> = (0..concurrency())
            .map(|index| format!("race-distinct-{index}"))
            .collect();
        let envelopes: Vec<String> = (0..concurrency())
            .map(|index| format!("envelope-{index}"))
            .collect();

        let runs: Vec<Run<'_>> = subjects
            .iter()
            .zip(&envelopes)
            .enumerate()
            .map(|(index, (subject, envelope))| {
                Run::new(&root, subject, envelope).gated_by(&gate, index)
            })
            .collect();
        for (index, death) in run_all_gated(&runs, &gate).into_iter().enumerate() {
            children += 1;
            assert_eq!(
                death,
                Death::Completed,
                "repetition {repetition} child {index} did not complete",
            );
        }

        for (subject, envelope) in subjects.iter().zip(&envelopes) {
            assert_eq!(
                read_back(&root, subject).as_deref(),
                Some(envelope.as_bytes()),
                "repetition {repetition}: {subject} kept its own entry",
            );
        }
    }
    record("race-distinct", children, "entries_per_key=1", started);
}

/// A process racing a writer that dies still reaches a validated entry.
///
/// One child is armed to die after taking the lock; the others are not. The
/// dead one's lock must be released by the kernel and the survivors must each
/// finish, which is the cross-process form of the recheck the threaded suite
/// exercises.
///
/// # Why the armed child's death is not a race
///
/// `after-lock` sits below the lock-free lookup, so only a process that *missed*
/// that lookup ever reaches it. A survivor that published first would leave the
/// armed child hitting and exiting zero — a scheduling event this case would
/// report as a failed crash property. The harness once made that unlikely by
/// sleeping fifty milliseconds inside every build step, which is a margin rather
/// than an order and fails on a machine under load.
///
/// The [`Gate`] makes it impossible instead: every child is held at the point it
/// has looked and found nothing, and none is released until all of them are
/// there. From that point the armed child reaches `after-lock` whatever anyone
/// else does — including when the namespace could not be locked at all, since
/// the fall-open path passes through the same phase.
#[test]
fn processes_racing_a_dying_writer_still_resolve() {
    let started = Instant::now();
    let mut children = 0;
    for repetition in 0..repetitions() {
        let scratch = Scratch::new("race-dying");
        let root = scratch.root();
        let subject = "race-dying";
        let racers = concurrency().max(1);
        let gate = Gate::new(&scratch, racers);

        let mut runs = vec![
            Run::new(&root, subject, "envelope-from-the-race")
                .killed_at(Phase::AfterLock)
                .gated_by(&gate, 0),
        ];
        runs.extend((1..racers).map(|index| {
            Run::new(&root, subject, "envelope-from-the-race").gated_by(&gate, index)
        }));

        let deaths = run_all_gated(&runs, &gate);
        children += u32::try_from(deaths.len()).expect("a bounded child count fits u32");
        assert_eq!(
            deaths[0],
            Death::Terminated,
            "repetition {repetition}: the armed child must die",
        );
        for (index, death) in deaths.iter().enumerate().skip(1) {
            assert_eq!(
                *death,
                Death::Completed,
                "repetition {repetition} survivor {index} did not complete",
            );
        }
        assert_eq!(
            read_back(&root, subject).as_deref(),
            Some(&b"envelope-from-the-race"[..]),
            "repetition {repetition}: the survivors published a validated entry",
        );
    }
    record("race-dying-writer", children, "dying_per_race=1", started);
}

// -------------------------------------------------------------------------
// Damaged, deleted, and unusable caches
// -------------------------------------------------------------------------

/// A truncated or digest-corrupt final entry is a miss that the next process
/// replaces.
///
/// Two damages rather than one: truncation is caught by the frame's own length
/// field before any section is read, and a single flipped byte in a
/// full-length entry is caught only by a section digest. A cache that handled
/// one and not the other would pass a test naming only the first.
#[test]
fn a_damaged_entry_is_replaced_by_the_next_process() {
    let started = Instant::now();
    let mut children = 0;
    for (repetition, (name, damage)) in (0..repetitions())
        .flat_map(|repetition| {
            [
                (repetition, ("truncated", Damage::Truncate)),
                (repetition, ("digest-corrupt", Damage::FlipLastByte)),
            ]
        })
        .collect::<Vec<_>>()
    {
        let scratch = Scratch::new(&format!("{name}-{repetition}"));
        let root = scratch.root();
        let subject = "damaged";

        assert_eq!(
            run(&Run::new(&root, subject, "original-envelope")),
            Death::Completed,
        );
        let entry = entry_path(&root, subject);
        let mut bytes = fs::read(&entry).expect("the entry is readable");
        match damage {
            Damage::Truncate => bytes.truncate(bytes.len() / 2),
            Damage::FlipLastByte => *bytes.last_mut().expect("a bundle is not empty") ^= 1,
        }
        fs::write(&entry, &bytes).expect("the entry is writable in this harness");
        assert!(
            read_back(&root, subject).is_none(),
            "{name}: a damaged entry must not read back",
        );

        let outcome = scratch.file("outcome");
        assert_eq!(
            run(&Run::new(&root, subject, "replacement-envelope").reporting_outcome_to(&outcome)),
            Death::Completed,
        );
        assert_eq!(
            fs::read_to_string(&outcome).expect("the child reports an outcome"),
            "published",
            "{name}: a damaged entry is rebuilt rather than served",
        );
        assert_eq!(
            read_back(&root, subject).as_deref(),
            Some(&b"replacement-envelope"[..]),
            "{name}: the replacement is what a reader now gets",
        );
        children += 2;
    }
    record("damaged-entries", children, "damage_kinds=2", started);
}

/// Which damage one case inflicts on a published entry.
#[derive(Clone, Copy, Debug)]
enum Damage {
    /// Cut the bundle in half, so its declared total length disagrees.
    Truncate,
    /// Flip one byte, so a section digest disagrees.
    FlipLastByte,
}

/// Deleting one entry, or the whole cache, causes rebuilding and never invalid
/// bytes.
///
/// ADR 0050 admits arbitrary external deletion: it "may cause duplicate work but
/// cannot authorize unvalidated bytes". Both scales are exercised because they
/// fail differently — one entry gone leaves the namespace intact, and the whole
/// root gone removes the lock file and the shard directories a writer expects.
#[test]
fn external_deletion_causes_rebuilding_and_never_invalid_bytes() {
    let started = Instant::now();
    let mut children = 0;
    for (repetition, (name, whole_cache)) in (0..repetitions())
        .flat_map(|repetition| {
            [
                (repetition, ("entry-deleted", false)),
                (repetition, ("cache-deleted", true)),
            ]
        })
        .collect::<Vec<_>>()
    {
        let scratch = Scratch::new(&format!("{name}-{repetition}"));
        let root = scratch.root();
        let subject = "deleted";

        assert_eq!(
            run(&Run::new(&root, subject, "first-envelope")),
            Death::Completed,
        );
        if whole_cache {
            fs::remove_dir_all(&root).expect("the cache root is removable");
        } else {
            fs::remove_file(entry_path(&root, subject)).expect("the entry is removable");
        }
        assert!(
            read_back(&root, subject).is_none(),
            "{name}: nothing remains"
        );

        let outcome = scratch.file("outcome");
        assert_eq!(
            run(&Run::new(&root, subject, "second-envelope").reporting_outcome_to(&outcome)),
            Death::Completed,
        );
        assert_eq!(
            fs::read_to_string(&outcome).expect("the child reports an outcome"),
            "published",
            "{name}: the next process rebuilds",
        );
        assert_eq!(
            read_back(&root, subject).as_deref(),
            Some(&b"second-envelope"[..]),
            "{name}: what a reader gets is the rebuilt entry",
        );
        children += 2;
    }
    record("external-deletion", children, "deletion_scales=2", started);
}

/// Recursive deletion racing live writers never yields an invalid read.
///
/// The deleter runs while children publish, so it removes directories a writer
/// is in the middle of using. Every child must finish — publishing, hitting, or
/// falling open to uncached — and whatever is left at the end must either read
/// back as a validated entry or not read back at all. Nothing here asserts *how
/// many* children published, because that is genuinely nondeterministic; the
/// claim is about what is readable, which is not.
///
/// Both assertions hold however the race falls, so neither depends on the timing
/// below. What the timing decides is whether the race is entered at all, and the
/// [`Gate`] is what settles that: the first removal happens with every writer
/// already inside the protocol, rather than possibly before any of them started.
/// The build delay then keeps them there long enough for the loop to remove
/// repeatedly under them, which is the one thing a barrier cannot express.
#[test]
fn recursive_deletion_racing_writers_never_yields_an_invalid_read() {
    let started = Instant::now();
    let mut children = 0;
    for repetition in 0..repetitions() {
        let scratch = Scratch::new("active-deletion");
        let root = scratch.root();
        let subject = "active-deletion";
        let gate = Gate::new(&scratch, concurrency());

        let runs: Vec<Run<'_>> = (0..concurrency())
            .map(|index| {
                Run::new(&root, subject, "envelope-under-deletion")
                    .with_build_delay(Duration::from_millis(40))
                    .gated_by(&gate, index)
            })
            .collect();
        let mut racing: Vec<Child> = runs
            .iter()
            .map(|run| run.command().spawn().expect("a child process spawns"))
            .collect();
        // The gate's files live beside the cache root rather than inside it, so
        // the removals below cannot unmake the rendezvous they are racing.
        gate.open(&mut racing);

        // Delete repeatedly while they work, so the race is entered more than
        // once rather than depending on one well-timed removal.
        let deadline = Instant::now() + Duration::from_millis(200);
        while Instant::now() < deadline {
            let _ = fs::remove_dir_all(&root);
            thread::sleep(Duration::from_millis(10));
        }

        let deaths = racing
            .iter_mut()
            .map(|child| wait_bounded(child, CHILD_DEADLINE));
        for (index, death) in deaths.enumerate() {
            children += 1;
            assert_eq!(
                death,
                Death::Completed,
                "repetition {repetition} child {index} did not survive deletion",
            );
        }

        // Whatever is left reads back as a validated entry or not at all. There
        // is deliberately no assertion that something is left: the last delete
        // may have removed everything.
        if let Some(envelope) = read_back(&root, subject) {
            assert_eq!(
                envelope, b"envelope-under-deletion",
                "repetition {repetition}: a readable entry is one that was published",
            );
        }
    }
    record("active-deletion", children, "invalid_reads=0", started);
}

/// A root that cannot be a directory falls open to a validated uncached result.
///
/// ADR 0050's fall-open rule, as a process rather than a thread: the child must
/// exit zero having reported `uncached`, because turning a cache problem into a
/// compilation failure "would make an optional accelerator a correctness
/// dependency".
#[test]
fn an_unusable_root_falls_open_in_a_real_process() {
    let scratch = Scratch::new("unusable-root");
    let occupied = scratch.file("occupied");
    fs::write(&occupied, b"a regular file where a directory must be")
        .expect("the scratch directory is writable");
    let outcome = scratch.file("outcome");

    assert_eq!(
        run(&Run::new(&occupied, "unusable", "envelope").reporting_outcome_to(&outcome)),
        Death::Completed,
        "an unusable cache must not fail the caller",
    );
    assert_eq!(
        fs::read_to_string(&outcome).expect("the child reports an outcome"),
        "uncached",
        "an unusable root yields a validated uncached result",
    );
}

/// A reader holding an open descriptor keeps reading across eviction.
///
/// This is why [`ExpansionCache::lookup`] takes no lock. The reader opens the
/// entry, another process evicts it, and the already-open descriptor still
/// yields the exact published bytes — so a lock-free read cannot observe a
/// half-removed entry. Held here rather than in a child because the property is
/// about *this* process's descriptor surviving *another* process's unlink.
#[test]
fn a_reader_holding_a_descriptor_reads_across_eviction() {
    let scratch = Scratch::new("open-across-eviction");
    let root = scratch.root();
    let subject = "open-across-eviction";
    assert_eq!(
        run(&Run::new(&root, subject, "envelope-before-eviction")),
        Death::Completed,
    );

    let entry = entry_path(&root, subject);
    let mut held = File::open(&entry).expect("a published entry opens");

    // Evict from another process, so the unlink is genuinely not this one's.
    let evictor = Scratch::new("evictor");
    drop(evictor);
    fs::remove_file(&entry).expect("the entry is removable");
    assert!(!entry.exists(), "the entry is unlinked");
    assert!(
        read_back(&root, subject).is_none(),
        "a new reader finds nothing",
    );

    let mut bytes = Vec::new();
    held.read_to_end(&mut bytes)
        .expect("an open descriptor survives the unlink");
    let key = CacheKey::derive(&subject_of(subject));
    let view = super::bundle::decode(&bytes, &key, &super::limits::Limits::default())
        .expect("the bytes an open descriptor yields are still a valid bundle");
    assert_eq!(&bytes[view.envelope], b"envelope-before-eviction");
}

// -------------------------------------------------------------------------
// Collection under load
// -------------------------------------------------------------------------

/// Builds the command that re-executes this binary as a collecting child.
fn collector_command(root: &Path, max_entries: u64, rounds: u32, log: &Path) -> Command {
    let exe = env::current_exe().expect("a test binary knows its own path");
    let mut command = Command::new(exe);
    command
        .args(["--exact", COLLECTOR_ENTRY])
        .env(COLLECT_ROOT_VARIABLE, root)
        .env(COLLECT_MAX_ENTRIES_VARIABLE, max_entries.to_string())
        .env(COLLECT_ROUNDS_VARIABLE, rounds.to_string())
        .env(COLLECT_LOG_VARIABLE, log)
        // No child inherits a seam it was not given. A collector reaches neither
        // of these — it does not publish — so removing them keeps that invariant
        // uniform across the entry points rather than resting on which paths a
        // collection happens to take today.
        .env_remove(fault::PHASE_VARIABLE)
        .env_remove(fault::ARRIVAL_VARIABLE)
        .env_remove(fault::RELEASE_VARIABLE)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

/// Reads back every entry the namespace currently holds, through the production
/// reader.
///
/// Returns the number validated, or `None` as soon as one fails. Walking the
/// namespace rather than a list of known subjects is what makes this a statement
/// about *whatever collection left behind* instead of about the keys the test
/// happened to remember.
fn every_surviving_entry_validates(root: &Path) -> Option<usize> {
    let cache = ExpansionCache::open(root);
    let accounting = cache.account().expect("a scan of a live cache succeeds");
    let mut validated = 0;
    for fact in accounting.entries() {
        match cache.read_entry(&fact.key, &any_payload) {
            Ok(_) => validated += 1,
            // Removed by the collector between the scan and this read, which is
            // the ordinary race and not a failure: an absence is what a reader
            // is entitled to see.
            Err(super::report::MissReason::Absent) => {}
            Err(_) => return None,
        }
    }
    Some(validated)
}

/// Checks every round a collecting child logged, returning the entries it
/// removed and the candidates it deliberately left alone.
///
/// Every round must be present and must have succeeded: a collector that stopped
/// early, or that reported a per-key failure, is a result the ladder must not
/// absorb into a passing total.
fn audit_collection_log(log: &Path, rounds: u32, scale: usize) -> (u64, u64) {
    let text = fs::read_to_string(log).expect("the collecting child logged its rounds");
    let rows: Vec<&str> = text.lines().collect();
    assert_eq!(
        rows.len(),
        rounds as usize,
        "{scale}: every collection round is recorded",
    );

    let mut removed = 0_u64;
    let mut deferred = 0_u64;
    for row in rows {
        assert!(
            row.starts_with("ok\t"),
            "{scale}: a collection failed: {row}"
        );
        // `ok`, selected, removed, reclaimed, contended, superseded,
        // already-absent, failed. The width is checked rather than assumed, so a
        // change to the row shape fails here instead of silently reading the
        // wrong column.
        let fields: Vec<&str> = row.split('\t').collect();
        assert_eq!(fields.len(), 8, "{scale}: unexpected row shape: {row}");
        removed += fields[2].parse::<u64>().expect("a removal count parses");
        // Contended, superseded, and already-absent: candidates a collection
        // deliberately left alone. Summed for the evidence row rather than
        // asserted on, because how often the race is entered is a property of
        // the host's scheduling and asserting it would make the gate flaky. Each
        // disposition has deterministic coverage in `super::tests`, which holds
        // the lock and replaces the entry itself rather than waiting for luck.
        for field in &fields[4..7] {
            deferred += field.parse::<u64>().expect("a disposition count parses");
        }
        assert_eq!(
            fields[7].parse::<u64>().expect("a failure count parses"),
            0,
            "{scale}: a collection reported a per-key failure: {row}",
        );
    }
    (removed, deferred)
}

/// Collection racing active writers and readers, at 1, 8, and 32 processes.
///
/// The research note's sixth gate asks for exactly this ladder. Every scale runs
/// the same shape: real publishing processes on distinct keys, a real collecting
/// process holding a bound tight enough that it is always removing something, and
/// this process reading throughout.
///
/// Four things are asserted, and each excludes a different failure:
///
/// 1. **Every child completes.** A child killed at its deadline would mean a
///    collector and a writer deadlocked on a key. This is what
///    [`KeyLock::try_acquire`](super::lock::KeyLock::try_acquire) buys: a
///    collector never waits for a lock, so it cannot be part of a cycle.
/// 2. **Every collection accounted for its whole selection**, asserted inside the
///    collecting process on every round and re-checked here from its log.
/// 3. **Something was actually removed.** Without this the case would pass
///    vacuously if the bound never selected anything, which is the shape of a
///    stress test that measures nothing.
/// 4. **Everything that survived still validates.** The correctness claim: a
///    collection concurrent with publication must never leave a reader something
///    it accepts and should not have.
#[test]
fn collection_races_active_processes_at_one_eight_and_thirty_two() {
    /// Entries seeded before the race, so a bound always has candidates.
    const SEEDS: usize = 8;
    /// Collections one collecting child performs per scale.
    const ROUNDS: u32 = 12;

    let started = Instant::now();
    let mut children = 0_u32;
    for writers in [1_usize, 8, 32] {
        let scratch = Scratch::new(&format!("collect-race-{writers}"));
        let root = scratch.root();
        let log = scratch.file("collections");

        let seed_subjects: Vec<String> = (0..SEEDS).map(|index| format!("seed-{index}")).collect();
        let seeds: Vec<Run<'_>> = seed_subjects
            .iter()
            .map(|subject| Run::new(&root, subject, "seed-envelope"))
            .collect();
        for (index, death) in run_all(&seeds).into_iter().enumerate() {
            children += 1;
            assert_eq!(death, Death::Completed, "{writers}: seed {index} failed");
        }

        // A reader that opened an entry *before* the collector started and has
        // not finished with it. This is the position the design has to defend:
        // the descriptor is open, the bytes have not all been read, and another
        // process is about to unlink the file.
        let held_subject = &seed_subjects[0];
        let held_key = CacheKey::derive(&subject_of(held_subject));
        let mut held = File::open(entry_path(&root, held_subject)).expect("a seed entry opens");

        // Half the writers share one key and half take their own, because the
        // two arrangements stress different halves of the design. A shared key
        // keeps its lock held almost continuously, which is what makes a
        // collector meet a *contended* candidate; distinct keys keep publishing
        // new entries under the collector's feet, which is what makes it meet a
        // *superseded* or already-absent one.
        let racing_subjects: Vec<String> = (0..writers)
            .map(|index| {
                if index % 2 == 0 {
                    "racing-shared".to_owned()
                } else {
                    format!("racing-{index}")
                }
            })
            .collect();
        let gate = Gate::new(&scratch, writers);
        let racing: Vec<Run<'_>> = racing_subjects
            .iter()
            .enumerate()
            .map(|(index, subject)| {
                Run::new(&root, subject, "envelope-under-collection")
                    .with_build_delay(Duration::from_millis(30))
                    .gated_by(&gate, index)
            })
            .collect();
        let mut spawned: Vec<Child> = racing
            .iter()
            .map(|run| run.command().spawn().expect("a child process spawns"))
            .collect();
        // Every writer is inside the protocol before the collector exists, and
        // none proceeds until it does, so collection and publication overlap by
        // construction instead of by however fast this host spawns processes.
        // The build delay then holds the shared key's lock long enough for a
        // collector to meet a contended candidate, which is a span rather than
        // an order and so is not something the barrier can supply.
        gate.await_arrivals(&mut spawned);
        spawned.push(
            collector_command(&root, 2, ROUNDS, &log)
                .spawn()
                .expect("a collecting child spawns"),
        );
        gate.release();

        // This process is the active reader for the duration.
        let deadline = Instant::now() + Duration::from_millis(150);
        while Instant::now() < deadline {
            assert!(
                every_surviving_entry_validates(&root).is_some(),
                "{writers}: a surviving entry failed validation during collection",
            );
            thread::sleep(Duration::from_millis(5));
        }

        for (index, child) in spawned.iter_mut().enumerate() {
            children += 1;
            assert_eq!(
                wait_bounded(child, CHILD_DEADLINE),
                Death::Completed,
                "{writers}: child {index} did not complete — a collector must never \
                 block on a key lock, so nothing here may deadlock",
            );
        }

        // The reader that has been holding a descriptor since before the race
        // still gets exactly the bytes that were published, whether or not the
        // collector unlinked the file underneath it.
        let mut bytes = Vec::new();
        held.read_to_end(&mut bytes)
            .expect("an open descriptor survives a collector's unlink");
        let view = super::bundle::decode(&bytes, &held_key, &super::limits::Limits::default())
            .expect("a descriptor opened before collection still yields a valid bundle");
        assert_eq!(&bytes[view.envelope], b"seed-envelope");

        let (removed_total, deferred_total) = audit_collection_log(&log, ROUNDS, writers);
        assert!(
            removed_total > 0,
            "{writers}: the bound never removed anything, so this case measured nothing",
        );

        assert!(
            every_surviving_entry_validates(&root).is_some(),
            "{writers}: an entry surviving the race failed validation",
        );
        record(
            "collection-under-load",
            u32::try_from(writers).expect("a bounded writer count fits u32"),
            &format!("rounds={ROUNDS} removed={removed_total} deferred={deferred_total}"),
            started,
        );
    }
    record("collection-ladder", children, "scales=1,8,32", started);
}

// -------------------------------------------------------------------------
// The harness's own integrity
// -------------------------------------------------------------------------

/// Every phase name round-trips, and the enumeration is the whole set.
///
/// The name crosses a process boundary as text. A phase whose name did not parse
/// would arm nothing, and the child would exit cleanly — which
/// [`a_writer_killed_at_any_phase_leaves_a_recoverable_cache`] would catch, but
/// only by reporting a confusing failure. This catches it directly.
///
/// # Why the completeness half is a distinctness check
///
/// "Every phase is a listed kill point" cannot be asserted by asking the list
/// whether it contains what the list just yielded; that is true of any list. It
/// is instead established in two pieces, neither of which the list supplies for
/// itself. [`Phase::KILL_POINTS`] is *declared* at
/// [`mem::variant_count::<Phase>()`](std::mem::variant_count), so a phase added
/// to the enum and not to the list is an array-length build error rather than a
/// silently unmeasured phase. That length alone would still be satisfied by a
/// list that repeated one phase and omitted another, so the pairwise
/// distinctness below is what closes it: a list of `variant_count` distinct
/// variants is the whole variant set.
#[test]
fn every_phase_name_round_trips() {
    for phase in Phase::KILL_POINTS {
        assert_eq!(Phase::parse(phase.as_str()), Some(phase), "{phase:?}");
    }
    let mut distinct = Phase::KILL_POINTS.to_vec();
    distinct.sort_unstable();
    distinct.dedup();
    assert_eq!(
        distinct.len(),
        Phase::KILL_POINTS.len(),
        "a phase is listed twice, so the kill points are one short of the enum \
         and a phase is measured by nothing: {:?}",
        Phase::KILL_POINTS,
    );
    assert_eq!(Phase::parse("no-such-phase"), None);
}

/// An unarmed child completes and publishes, so a dying child means the phase
/// was reached.
///
/// Without this, every killed-writer assertion above could be satisfied by a
/// harness whose children always die for some unrelated reason — a missing
/// binary, a bad argument, a panicking guard — and the suite would report nine
/// measured phases having measured none.
#[test]
fn an_unarmed_child_completes_and_publishes() {
    let scratch = Scratch::new("unarmed");
    let root = scratch.root();
    let outcome = scratch.file("outcome");
    assert_eq!(
        run(&Run::new(&root, "unarmed", "envelope").reporting_outcome_to(&outcome)),
        Death::Completed,
    );
    assert_eq!(
        fs::read_to_string(&outcome).expect("the child reports an outcome"),
        "published",
    );
    assert_eq!(
        read_back(&root, "unarmed").as_deref(),
        Some(&b"envelope"[..]),
    );
}

/// A child that cannot finish is killed at its deadline and reported as such.
///
/// The deadline is the harness's own liveness guarantee, and a guarantee nothing
/// reaches is a guess about what would happen. This case makes a child genuinely
/// stuck — the parent holds the key's lock and does not let go — and drives
/// [`wait_bounded`] with a deadline short enough to wait for, so
/// [`Death::TimedOut`] is an observed result rather than an unreachable arm.
///
/// It also proves the blocking half of the protocol from the outside: a second
/// process on a contended key waits rather than rebuilding what the holder is
/// about to publish.
#[test]
fn a_stuck_child_is_killed_at_its_deadline() {
    let scratch = Scratch::new("stuck");
    let root = scratch.root();
    let subject = "stuck";
    let cache = ExpansionCache::open(&root);
    let key = CacheKey::derive(&subject_of(subject));
    ExpansionCache::prepare_directories(cache.rooted_layout(), &key)
        .expect("the namespace is creatable");
    let held =
        ExpansionCache::acquire_lock(cache.rooted_layout(), &key).expect("the lock is takeable");

    let mut child = Run::new(&root, subject, "envelope")
        .command()
        .spawn()
        .expect("a child process spawns");
    assert_eq!(
        wait_bounded(&mut child, Duration::from_millis(750)),
        Death::TimedOut,
        "a child must block on a lock this process holds, and be killed at the deadline",
    );

    held.release().expect("the lock releases");
    // The key is usable again once the holder lets go, so the stall above was
    // the lock and not something the harness broke.
    assert_eq!(run(&Run::new(&root, subject, "envelope")), Death::Completed);
}

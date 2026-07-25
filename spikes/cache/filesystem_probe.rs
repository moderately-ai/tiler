//! Measures, on one directory, the filesystem properties the expansion cache
//! protocol rests on.
//!
//! Dependency-free and `std`-only, matching `cache_harness.rs`. Build and run:
//!
//! ```sh
//! rustc --edition 2021 spikes/cache/filesystem_probe.rs -o /tmp/tiler-fs-probe
//! /tmp/tiler-fs-probe /path/to/candidate/cache/root \
//!     [--across /path/on/another/filesystem] [--evidence out.tsv]
//! ```
//!
//! Each check prints one tab-separated row: name, verdict, detail. The process
//! exits non-zero when a **required** property is refuted, so the probe is both
//! evidence and a usable "is this root supported" diagnostic.
//!
//! `--across` names a directory the caller believes is on a *different*
//! filesystem, which enables the one check that needs two of them: that a
//! `rename` between them is refused with the error the publication path
//! classifies as [`io::ErrorKind::CrossesDevices`].
//!
//! # What it cannot do
//!
//! Every check runs on one host. Cross-host advisory-lock exclusion — the case a
//! network filesystem breaks silently under `locallocks` (Darwin `mount_nfs(8)`)
//! or `local_lock=` (Linux `nfs(5)`) — needs a second machine mounting the same
//! export, so `lock-excludes-processes` passing here says nothing about it. That
//! limitation is a property of the question, not of this program: no local test
//! can distinguish a lock that excludes remote clients from one that does not.

use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Seconds between the steps of the access-time classification.
///
/// Two reads separated by less than the host's timestamp granularity would be
/// indistinguishable from a filesystem that never advances the access time, so
/// the interval has to exceed it by a comfortable margin.
const ATIME_INTERVAL: Duration = Duration::from_secs(2);

/// Files created back to back when measuring modification-time granularity.
const GRANULARITY_SAMPLES: usize = 64;

/// Publications the rename race performs while a reader spins.
const RENAME_ROUNDS: u32 = 400;

/// Environment variable naming the lock file a re-executed child should hold.
const CHILD_LOCK_PATH: &str = "TILER_FS_PROBE_CHILD_LOCK";

fn main() {
    if let Ok(path) = env::var(CHILD_LOCK_PATH) {
        // Re-executed child mode: take the lock, announce it, and block until
        // the parent kills or closes us.
        child_hold_lock(Path::new(&path));
        return;
    }

    let (root, across, evidence) = match parse_arguments() {
        Some(parsed) => parsed,
        None => {
            eprintln!(
                "usage: tiler-fs-probe <root> [--across <other-filesystem-dir>] \
                 [--evidence <path>]"
            );
            std::process::exit(2);
        }
    };

    let mut report = Report::new();
    if let Err(error) = run(&root, across.as_deref(), &mut report) {
        eprintln!("probe could not run under {}: {error}", root.display());
        std::process::exit(2);
    }
    report.print();
    if let Some(path) = evidence {
        if let Err(error) = report.write(&path, &root) {
            eprintln!("could not write evidence to {}: {error}", path.display());
            std::process::exit(2);
        }
    }
    if report.required_failures() > 0 {
        std::process::exit(1);
    }
}

/// Reads the command line, or `None` when it is not usable.
fn parse_arguments() -> Option<(PathBuf, Option<PathBuf>, Option<PathBuf>)> {
    let mut positional = None;
    let mut across = None;
    let mut evidence = None;
    let mut arguments = env::args_os().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.to_str() {
            Some("--across") => across = Some(PathBuf::from(arguments.next()?)),
            Some("--evidence") => evidence = Some(PathBuf::from(arguments.next()?)),
            Some(flag) if flag.starts_with("--") => return None,
            _ if positional.is_none() => positional = Some(PathBuf::from(argument)),
            _ => return None,
        }
    }
    Some((positional?, across, evidence))
}

// -- the checks -----------------------------------------------------------

fn run(root: &Path, across: Option<&Path>, report: &mut Report) -> io::Result<()> {
    let area = root.join("tiler-fs-probe");
    let _ = fs::remove_dir_all(&area);
    fs::create_dir_all(&area)?;

    host(report);
    same_device(&area, report)?;
    create_new_excludes(&area, report)?;
    rename_replaces(&area, report)?;
    rename_never_missing(&area, report)?;
    open_unlinked_reader(&area, report)?;
    lock_excludes_processes(&area, report)?;
    lock_released_on_kill(&area, report)?;
    modification_time_granularity(&area, report)?;
    access_time_class(&area, report)?;
    if let Some(other) = across {
        rename_across_filesystems(&area, other, report)?;
    }

    fs::remove_dir_all(&area)?;
    Ok(())
}

/// Records the host, so a row can never be read as a portable guarantee.
fn host(report: &mut Report) {
    let describe = |program: &str, arguments: &[&str]| -> String {
        Command::new(program)
            .args(arguments)
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().replace('\n', " "))
            .unwrap_or_else(|| "unknown".to_owned())
    };
    report.note("host", &describe("uname", &["-srm"]));
}

/// The temporary and final trees must share a device, or no rename can publish.
///
/// Measured through `st_dev` rather than assumed from the paths being siblings:
/// a bind mount, an `automount`, or a symbolic link inside the namespace puts
/// them on different filesystems while leaving the layout looking identical.
fn same_device(area: &Path, report: &mut Report) -> io::Result<()> {
    let entries = area.join("entries");
    let temporaries = area.join("tmp");
    let locks = area.join("locks");
    for directory in [&entries, &temporaries, &locks] {
        fs::create_dir_all(directory)?;
    }
    let device = fs::metadata(&entries)?.dev();
    let same = fs::metadata(&temporaries)?.dev() == device && fs::metadata(&locks)?.dev() == device;
    report.require(
        "same-device",
        same,
        &format!("entries/tmp/locks st_dev={device}"),
    );
    Ok(())
}

/// `create_new` must refuse an existing path, which is what makes a temporary
/// name unique by filesystem operation rather than by trusting a nonce.
fn create_new_excludes(area: &Path, report: &mut Report) -> io::Result<()> {
    let path = area.join("exclusive.tmp");
    let _first = OpenOptions::new().write(true).create_new(true).open(&path)?;
    let second = OpenOptions::new().write(true).create_new(true).open(&path);
    let refused = matches!(&second, Err(error) if error.kind() == io::ErrorKind::AlreadyExists);
    report.require(
        "create-new-excludes",
        refused,
        &match second {
            Ok(_) => "a second create_new succeeded".to_owned(),
            Err(error) => format!("{:?}", error.kind()),
        },
    );
    fs::remove_file(&path)?;
    Ok(())
}

/// `rename` must replace an existing target rather than fail, and a descriptor
/// opened before the rename must keep reading the bytes it was opened on.
fn rename_replaces(area: &Path, report: &mut Report) -> io::Result<()> {
    let target = area.join("entries").join("entry.bundle");
    let first = area.join("tmp").join("first.tmp");
    let second = area.join("tmp").join("second.tmp");
    fs::write(&first, b"first")?;
    fs::write(&second, b"second")?;
    fs::rename(&first, &target)?;

    let mut held = File::open(&target)?;
    fs::rename(&second, &target)?;

    let mut through_descriptor = String::new();
    held.read_to_string(&mut through_descriptor)?;
    let after = fs::read_to_string(&target)?;
    report.require(
        "rename-replaces",
        after == "second",
        &format!("target reads {after:?}"),
    );
    report.require(
        "rename-preserves-open-descriptor",
        through_descriptor == "first",
        &format!("descriptor opened before the rename reads {through_descriptor:?}"),
    );
    fs::remove_file(&target)?;
    Ok(())
}

/// A reader must never observe the final path missing across a replacement.
///
/// This is the observable consequence of atomicity that a program can actually
/// test. It cannot prove atomicity — an implementation that unlinks and relinks
/// quickly enough would pass — so a failure refutes the property and a pass only
/// fails to refute it.
fn rename_never_missing(area: &Path, report: &mut Report) -> io::Result<()> {
    let target = area.join("entries").join("raced.bundle");
    let seed = area.join("tmp").join("seed.tmp");
    fs::write(&seed, b"payload-0")?;
    fs::rename(&seed, &target)?;

    let stop = Arc::new(AtomicBool::new(false));
    let reader_stop = Arc::clone(&stop);
    let reader_target = target.clone();
    let reader = thread::spawn(move || {
        let mut missing = 0_u64;
        let mut short = 0_u64;
        let mut observations = 0_u64;
        while !reader_stop.load(Ordering::Relaxed) {
            match fs::read(&reader_target) {
                Ok(bytes) => {
                    observations += 1;
                    if !bytes.starts_with(b"payload-") {
                        short += 1;
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::NotFound => missing += 1,
                Err(_) => short += 1,
            }
        }
        (missing, short, observations)
    });

    let temporaries = area.join("tmp");
    for round in 0..RENAME_ROUNDS {
        let temporary = temporaries.join(format!("round-{round}.tmp"));
        fs::write(&temporary, format!("payload-{round}").as_bytes())?;
        fs::rename(&temporary, &target)?;
    }
    stop.store(true, Ordering::Relaxed);
    let (missing, short, observations) = reader.join().expect("the reader thread panicked");

    report.require(
        "rename-never-missing",
        missing == 0 && short == 0,
        &format!(
            "{RENAME_ROUNDS} publications, {observations} reads, {missing} absent, {short} \
             malformed"
        ),
    );
    fs::remove_file(&target)?;
    Ok(())
}

/// A file unlinked after a reader opened it must stay readable through that
/// descriptor. This is what lets readers take no lock.
fn open_unlinked_reader(area: &Path, report: &mut Report) -> io::Result<()> {
    let path = area.join("entries").join("unlinked.bundle");
    let payload: Vec<u8> = (0..64_u32).flat_map(u32::to_be_bytes).collect();
    fs::write(&path, &payload)?;
    let mut held = File::open(&path)?;
    fs::remove_file(&path)?;

    let mut read_back = Vec::new();
    let outcome = held.read_to_end(&mut read_back);
    let intact = matches!(outcome, Ok(_)) && read_back == payload;
    report.require(
        "open-unlinked-reader",
        intact,
        &match outcome {
            Ok(count) => format!("{count} bytes read after unlink, identical={}", read_back == payload),
            Err(error) => format!("{:?}", error.kind()),
        },
    );
    Ok(())
}

/// An exclusive advisory lock held by one process must refuse another.
///
/// The child is a re-executed copy of this program, so the two contenders are
/// genuinely separate processes and not two descriptors in one.
fn lock_excludes_processes(area: &Path, report: &mut Report) -> io::Result<()> {
    let path = area.join("locks").join("exclusion.lock");
    let mut child = spawn_lock_holder(&path)?;
    let acquired = wait_for_child_lock(&mut child);

    let verdict = if acquired {
        let contender = open_lock_file(&path)?;
        match contender.try_lock() {
            Ok(()) => Some(false),
            Err(_) => Some(true),
        }
    } else {
        None
    };
    let _ = child.kill();
    let _ = child.wait();

    match verdict {
        Some(excluded) => report.require(
            "lock-excludes-processes",
            excluded,
            if excluded {
                "a second process was refused the held lock"
            } else {
                "a second process acquired a lock another process holds"
            },
        ),
        None => report.require(
            "lock-excludes-processes",
            false,
            "the child never reported holding the lock",
        ),
    }
    Ok(())
}

/// Killing the holder must release its lock, because that is the whole of the
/// protocol's stale-lock recovery story.
fn lock_released_on_kill(area: &Path, report: &mut Report) -> io::Result<()> {
    let path = area.join("locks").join("kill.lock");
    let mut child = spawn_lock_holder(&path)?;
    let acquired = wait_for_child_lock(&mut child);
    child.kill()?;
    child.wait()?;

    let contender = open_lock_file(&path)?;
    let taken = contender.try_lock().is_ok();
    report.require(
        "lock-released-on-kill",
        acquired && taken,
        &format!("child held={acquired}, survivor acquired={taken}"),
    );
    Ok(())
}

/// The smallest non-zero modification-time difference the host reports.
///
/// The collector orders by modification time and re-checks it before removing an
/// entry, so a coarse granularity is what makes two publications indistinguishable.
fn modification_time_granularity(area: &Path, report: &mut Report) -> io::Result<()> {
    let directory = area.join("granularity");
    fs::create_dir_all(&directory)?;
    let mut stamps = Vec::with_capacity(GRANULARITY_SAMPLES);
    for index in 0..GRANULARITY_SAMPLES {
        let path = directory.join(format!("{index}.bin"));
        fs::write(&path, index.to_be_bytes())?;
        stamps.push(fs::metadata(&path)?.modified()?);
    }
    let mut smallest: Option<Duration> = None;
    for window in stamps.windows(2) {
        if let Ok(delta) = window[1].duration_since(window[0]) {
            if !delta.is_zero() {
                smallest = Some(smallest.map_or(delta, |best| best.min(delta)));
            }
        }
    }
    report.note(
        "mtime-granularity-ns",
        &match smallest {
            Some(delta) => delta.as_nanos().to_string(),
            None => format!("no distinct stamp over {GRANULARITY_SAMPLES} writes"),
        },
    );
    fs::remove_dir_all(&directory)?;
    Ok(())
}

/// Classifies how the host maintains the access time.
///
/// Three outcomes, and only the first would make access time usable as *use*
/// recency:
///
/// - `strict`   — every read advances it.
/// - `relatime` — a read advances it only while it is at or before the
///                modification time, so an entry published once and read many
///                times advances it exactly once, ever.
/// - `none`     — no read advances it.
fn access_time_class(area: &Path, report: &mut Report) -> io::Result<()> {
    let path = area.join("entries").join("atime.bundle");
    fs::write(&path, vec![7_u8; 64 * 1024])?;

    let created = access_time(&path)?;
    thread::sleep(ATIME_INTERVAL);
    read_whole(&path)?;
    let first = access_time(&path)?;
    thread::sleep(ATIME_INTERVAL);
    read_whole(&path)?;
    let second = access_time(&path)?;

    // Push the modification time past the access time and read again. Under a
    // relatime-like predicate that re-arms exactly one advance, which is what
    // separates it from a filesystem that simply stopped reporting.
    File::options()
        .write(true)
        .open(&path)?
        .set_modified(SystemTime::now())?;
    thread::sleep(ATIME_INTERVAL);
    read_whole(&path)?;
    let rearmed = access_time(&path)?;

    let advanced_once = first > created;
    let advanced_twice = second > first;
    let advanced_after_rearm = rearmed > second;
    let class = if advanced_once && advanced_twice {
        "strict"
    } else if advanced_once && advanced_after_rearm {
        "relatime"
    } else if advanced_once {
        "relatime-unconfirmed"
    } else {
        "none"
    };
    report.note(
        "atime-class",
        &format!(
            "{class} (created={created} read1={first} read2={second} after-mtime-bump={rearmed})"
        ),
    );
    report.note(
        "atime-usable-as-use-recency",
        if class == "strict" { "yes" } else { "no" },
    );
    fs::remove_file(&path)?;
    Ok(())
}

/// A publication that would cross a filesystem boundary must be *refused*, and
/// refused with the error the store classifies rather than a generic one.
///
/// `ExpansionCache::publish` maps [`io::ErrorKind::CrossesDevices`] to its own
/// `CrossesFilesystems` refusal and everything else to an unavailability. A host
/// that reported some other kind here would send a cross-device root down the
/// generic path, which is a reporting defect rather than a correctness one — the
/// publication is refused either way.
///
/// Skipped, not failed, when `other` turns out to share this filesystem: the
/// check would then be vacuous, and reporting a vacuous pass is the failure mode
/// this whole program exists to avoid.
fn rename_across_filesystems(area: &Path, other: &Path, report: &mut Report) -> io::Result<()> {
    let here = fs::metadata(area)?.dev();
    fs::create_dir_all(other)?;
    let there = fs::metadata(other)?.dev();
    if here == there {
        report.note(
            "rename-across-filesystems",
            &format!("skipped: --across shares st_dev={here} with the root"),
        );
        return Ok(());
    }

    let source = other.join("across.tmp");
    fs::write(&source, b"across")?;
    let destination = area.join("entries").join("across.bundle");
    let outcome = fs::rename(&source, &destination);
    let kind = outcome.as_ref().err().map(io::Error::kind);
    report.require(
        "rename-across-filesystems-refused",
        outcome.is_err(),
        &format!("st_dev {here} -> {there}: {kind:?}"),
    );
    report.note(
        "rename-across-filesystems-kind",
        &format!("{kind:?} (the store classifies CrossesDevices)"),
    );
    let _ = fs::remove_file(&source);
    let _ = fs::remove_file(&destination);
    Ok(())
}

// -- helpers --------------------------------------------------------------

fn read_whole(path: &Path) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut sink = Vec::new();
    file.read_to_end(&mut sink)?;
    Ok(())
}

/// Nanoseconds since the epoch of a path's access time.
fn access_time(path: &Path) -> io::Result<i128> {
    let metadata = fs::metadata(path)?;
    Ok(i128::from(metadata.atime()) * 1_000_000_000 + i128::from(metadata.atime_nsec()))
}

fn open_lock_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
}

fn spawn_lock_holder(path: &Path) -> io::Result<std::process::Child> {
    let program = env::current_exe()?;
    Command::new(program)
        .env(CHILD_LOCK_PATH, path)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
}

/// Waits for the child to report that it holds the lock.
///
/// The child writes one byte to its standard output *after* acquiring, so this
/// is a real handshake rather than a sleep chosen to be long enough.
fn wait_for_child_lock(child: &mut std::process::Child) -> bool {
    let Some(stdout) = child.stdout.as_mut() else {
        return false;
    };
    let mut byte = [0_u8; 1];
    matches!(stdout.read_exact(&mut byte), Ok(())) && byte[0] == b'H'
}

/// Child mode: hold the lock and block forever.
fn child_hold_lock(path: &Path) {
    let Ok(file) = open_lock_file(path) else {
        return;
    };
    if file.lock().is_err() {
        return;
    }
    let mut out = io::stdout();
    if out.write_all(b"H").is_err() || out.flush().is_err() {
        return;
    }
    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}

// -- reporting ------------------------------------------------------------

/// One measured property.
struct Row {
    name: String,
    verdict: &'static str,
    detail: String,
}

struct Report {
    rows: Vec<Row>,
}

impl Report {
    fn new() -> Self {
        Self { rows: Vec::new() }
    }

    /// Records a property the cache protocol requires.
    fn require(&mut self, name: &str, held: bool, detail: &str) {
        self.rows.push(Row {
            name: name.to_owned(),
            verdict: if held { "holds" } else { "REFUTED" },
            detail: detail.to_owned(),
        });
    }

    /// Records an observation that is not pass/fail.
    fn note(&mut self, name: &str, detail: &str) {
        self.rows.push(Row {
            name: name.to_owned(),
            verdict: "observed",
            detail: detail.to_owned(),
        });
    }

    fn required_failures(&self) -> usize {
        self.rows.iter().filter(|row| row.verdict == "REFUTED").count()
    }

    fn print(&self) {
        for row in &self.rows {
            println!("{}\t{}\t{}", row.name, row.verdict, row.detail);
        }
    }

    fn write(&self, path: &Path, root: &Path) -> io::Result<()> {
        let mut file = OpenOptions::new().create(true).write(true).open(path)?;
        file.seek(SeekFrom::End(0))?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs())
            .unwrap_or(0);
        for row in &self.rows {
            writeln!(
                file,
                "{stamp}\t{}\t{}\t{}\t{}",
                root.display(),
                row.name,
                row.verdict,
                row.detail
            )?;
        }
        file.flush()
    }
}

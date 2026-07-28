//! A proc macro whose expansion resolves a real artifact through the real
//! expansion cache.
//!
//! This crate exists so that `cargo` and `rust-analyzer` — rather than a harness
//! that spawns its own workers — are the processes driving
//! [`tiler_cache::expansion`]. ADR 0050's context sentence is that "Cargo and
//! rust-analyzer may run equivalent proc-macro expansions concurrently"; every
//! measurement before this one modelled that workload instead of running it.
//!
//! # What an expansion does
//!
//! One expansion composes a subject, calls
//! [`ExpansionCache::get_or_publish`](tiler_cache::expansion::ExpansionCache::get_or_publish),
//! and writes one event file describing what the cache did. The build closure is
//! a real compile-and-encode through `tiler-compiler` and `tiler-artifact`, so a
//! published entry carries a genuine artifact envelope and every hit is
//! validated by the real `decode_artifact`.
//!
//! # How it finds its cache root
//!
//! `CARGO_MANIFEST_DIR` is one of only two environment variables the
//! macro-environment spike measured as reliably present during expansion, and it
//! is present under both drivers. The root is derived from it rather than from a
//! variable a driver would have to inject, because `rust-analyzer` populates a
//! proc-macro's environment from the crate graph rather than from the editor's
//! own process environment. `TILER_EXERCISE_ROOT` overrides it when a driver can
//! set it, which is what lets one scenario point two builds at one root.

use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use proc_macro::TokenStream;
use tiler_cache::expansion::{
    ComposedSubject, ExpansionCache, PublishFailure, Resolution, SubjectFacets,
};

/// Resolves one artifact through the expansion cache and expands to a constant.
///
/// Invoked as `resolve!(NAME);`. The identifier names the emitted constant and
/// is also the key tag, so two invocations naming different identifiers occupy
/// different cache keys and two naming the same identifier contend for one.
///
/// # Panics
///
/// Panics when the build closure's artifact fails to encode, which is a defect
/// in this spike rather than a cache outcome. Every *cache* problem is a
/// [`Resolution::Uncached`] carrying its reason, which this macro records rather
/// than failing on.
#[proc_macro]
pub fn resolve(input: TokenStream) -> TokenStream {
    let tag = input.to_string().trim().to_owned();
    assert!(!tag.is_empty(), "resolve! needs an identifier");

    let root = exercise_root();
    // The cache directory is overridable independently of the state root, so a
    // scenario can make the cache unusable while its events still land
    // somewhere readable. That is what lets the negative control observe the
    // duplicate compilation an unusable cache produces.
    let cache = ExpansionCache::open(
        std::env::var_os("TILER_EXERCISE_CACHE_OVERRIDE")
            .map_or_else(|| root.join("cache"), PathBuf::from),
    );

    // Both facets are stand-ins, and deliberately so: no producer exists for
    // `SubjectFacet::ArtifactProgram` until
    // `derive-the-pre-compilation-artifact-program-subject` lands. This spike
    // measures process behaviour, for which the subject only has to be a stable
    // function of the invocation.
    let backend = format!("tiler.exercise.backend-compilation.{tag}").into_bytes();
    let program = b"tiler.exercise.artifact-program.v1".to_vec();
    let runs: [&[u8]; 1] = [&backend];
    let subject = ComposedSubject::compose(&SubjectFacets {
        backend_compilations: &runs,
        artifact_program: &program,
    })
    .expect("both facets are non-empty");

    let started = Instant::now();
    let started_wall = unix_nanos();
    let mut built = false;
    let outcome = cache.get_or_publish(&subject, || {
        built = true;
        build_with_observable_window(&root, &tag);
        Ok::<_, ArtifactBuildFailed>(exercise_envelope::encoded_envelope())
    });

    let elapsed = started.elapsed();
    let label = match &outcome {
        Ok(Resolution::Hit { .. }) => "hit",
        Ok(Resolution::Published { .. }) => "published",
        Ok(Resolution::Uncached { .. }) => "uncached",
        Err(PublishFailure::Build(_)) => "build-failed",
        Err(PublishFailure::Artifact(_)) => "artifact-invalid",
    };
    let reason = match &outcome {
        Ok(Resolution::Hit { report, .. } | Resolution::Published { report, .. }) => {
            format!("{report:?}")
        }
        Ok(Resolution::Uncached { report, .. }) => format!("{report:?}"),
        Err(failure) => format!("{failure:?}"),
    };

    record_event(
        &root,
        &Event {
            tag: &tag,
            label,
            built,
            elapsed,
            reason: &reason,
            started_wall,
            ended_wall: unix_nanos(),
        },
    );

    format!(
        "#[doc = \"How the expansion cache resolved `{tag}`.\"] \
         pub const {tag}: &str = \"{label}\";"
    )
    .parse()
    .expect("the emitted constant parses")
}

/// The failure type of the build closure, which this spike never produces.
///
/// The closure compiles and encodes in-process, so the only way it can fail is a
/// panic. Naming an uninhabited-in-practice error type keeps
/// [`PublishFailure::Build`] distinguishable from
/// [`PublishFailure::Artifact`] in the recorded outcome.
#[derive(Debug)]
struct ArtifactBuildFailed;

/// One recorded expansion.
///
/// The wall-clock bounds are what let a driver *prove* two expansions
/// overlapped. Without them, three builds that happened to serialize and three
/// that genuinely raced produce identical counts, and the concurrency the
/// scenario claims to exercise would be an assumption rather than an
/// observation.
struct Event<'text> {
    tag: &'text str,
    label: &'static str,
    built: bool,
    elapsed: Duration,
    reason: &'text str,
    started_wall: u128,
    ended_wall: u128,
}

/// The working directory the expansion ran in.
///
/// Recorded because the ticket asks whether the two drivers share one, and
/// because a frontend that derived a cache root from a relative path would
/// depend on the answer. `rust-analyzer` sends a `current_dir` with each
/// expansion request rather than inheriting the editor's, so this is not
/// predictable from how the process was launched.
fn working_directory() -> String {
    std::env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| "unknown".to_owned())
}

/// Reads the wall clock as nanoseconds since the Unix epoch.
///
/// Used only to compare windows recorded by processes on one host within one
/// run, which is what makes a single clock source adequate here.
fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos())
}

/// Runs the expensive build, announcing that it holds the key lock.
///
/// The marker is written *before* the delay and removed after it, so a driver
/// that wants to kill a writer mid-publication waits for the marker to appear
/// rather than for a wall-clock margin to elapse. `harness.rs`'s known defect is
/// exactly the latter, and this spike does not repeat it.
fn build_with_observable_window(root: &PathBuf, tag: &str) {
    let markers = root.join("markers");
    let _ = fs::create_dir_all(&markers);
    let marker = markers.join(format!("{}.{}.building", tag, process::id()));
    let _ = fs::write(&marker, b"holding the key lock");

    let budget = env_millis("TILER_EXERCISE_BUILD_DELAY_MS");
    if budget > Duration::ZERO {
        // Released by observed state when the driver asks, and by the budget
        // otherwise, so a driver that dies cannot wedge a build forever.
        let release = root.join("release");
        let deadline = Instant::now() + budget;
        while Instant::now() < deadline && !release.exists() {
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    let _ = fs::remove_file(&marker);
}

/// Writes one event as its own file.
///
/// One file per event rather than appended lines, because several uncoordinated
/// processes write here at once and a driver must be able to *count* the
/// population it is judging. Interleaved appends can lose that count silently.
fn record_event(root: &PathBuf, event: &Event<'_>) {
    let events = root.join("events");
    if fs::create_dir_all(&events).is_err() {
        return;
    }
    let path = events.join(format!("{}.{}.json", event.ended_wall, process::id()));
    let record = format!(
        concat!(
            "{{\"tag\":\"{}\",\"outcome\":\"{}\",\"built\":{},",
            "\"pid\":{},\"elapsed_ms\":{},\"driver\":\"{}\",\"cwd\":{:?},",
            "\"started_ns\":{},\"ended_ns\":{},\"report\":{:?}}}"
        ),
        event.tag,
        event.label,
        event.built,
        process::id(),
        event.elapsed.as_millis(),
        driver_name(),
        working_directory(),
        event.started_wall,
        event.ended_wall,
        event.reason,
    );
    let _ = fs::write(path, record);
}

/// Names the executable that is performing this expansion.
///
/// **Measured, and not what was first assumed.** The obvious discriminator —
/// whether `CARGO_PKG_NAME` is set — does not work: `rust-analyzer` populates a
/// proc-macro's environment from the crate graph it loaded, so that variable is
/// present under *both* drivers and a macro reading it concludes "cargo" in
/// both. The host executable is the reliable signal, because the two drivers run
/// the expansion in genuinely different programs: `rustc` under Cargo, and
/// `rust-analyzer-proc-macro-srv` under the editor.
fn driver_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

/// Resolves the directory this exercise keeps its cache and evidence under.
fn exercise_root() -> PathBuf {
    if let Some(root) = std::env::var_os("TILER_EXERCISE_ROOT") {
        return PathBuf::from(root);
    }
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is present during expansion under both drivers");
    PathBuf::from(manifest)
        .parent()
        .expect("the manifest directory has a parent")
        .join("exercise-state")
}

/// Reads a millisecond budget from the environment, defaulting to zero.
fn env_millis(name: &str) -> Duration {
    std::env::var(name)
        .ok()
        .and_then(|text| text.parse::<u64>().ok())
        .map_or(Duration::ZERO, Duration::from_millis)
}

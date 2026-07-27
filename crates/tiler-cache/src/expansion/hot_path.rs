//! Measurements for the validated-read path.
//!
//! # What these are for
//!
//! The same split [`crate`]'s compiler counterpart uses: these tests *print* and
//! assert nothing about time. A timing assertion fails on a loaded machine and
//! passes on a fast one, which makes it a flake rather than a guard. What they
//! are for is a reproducible number to compare across a change — run them before
//! and after and read the two.
//!
//! **Report the minimum, not the mean.** Every perturbation a host applies makes
//! a read *slower* and none makes it faster, so the distribution has a hard floor
//! at the true cost and an unbounded tail of noise. The minimum of enough runs
//! estimates that floor; the mean estimates the floor plus whatever else the
//! machine was doing.
//!
//! Reproduce with:
//!
//! ```text
//! cargo nextest run --release -p tiler-cache -E 'test(hot_path)' --no-capture
//! ```
//!
//! # The measurement boundary, stated exactly
//!
//! These drive the crate-private [`ExpansionCache::read_entry`] with a payload
//! validator that does nothing, for the reason [`super`]'s module documentation
//! already gives: building a real artifact envelope needs a `SemanticProgram`,
//! and this crate deliberately does not depend on `tiler-ir`.
//!
//! That makes the *denominator* smaller than a production hit's, never larger,
//! because the public path adds [`tiler_artifact::program::decode_artifact`] on
//! top of everything measured here and removes nothing. So a cost's share
//! reported below is an **upper bound** on its share of a real cache hit. This is
//! the direction that matters: a component measured as a small share here is a
//! smaller share in production, and that conclusion survives the substitution.
//!
//! What is *not* bounded this way is the reverse claim. A component measured as a
//! large share here may be a modest one in production, so nothing below is
//! offered as evidence that any single step dominates a real hit.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tiler_artifact::program::ArtifactCodecFailure;

use super::key::CacheKey;
use super::store::{Durability, ExpansionCache, ProtocolOutcome};
use super::subject::{ComposedSubject, SubjectFacets};

/// Bundle body sizes the sweep reports, in bytes of artifact envelope.
///
/// The middle entry is the size the ticket that prompted this harness named; the
/// outer two bracket it by roughly an order of magnitude each way, so a cost that
/// is per-byte and one that is per-read are distinguishable in the printed rows
/// rather than only in a profile.
const ENVELOPE_SIZES: [usize; 3] = [4 * 1024, 26 * 1024, 256 * 1024];

/// A validator that produces nothing, so the payload contributes no cost.
///
/// Deliberately not a stand-in for the artifact decoder — that is the whole
/// point, and it is what makes the reported shares an upper bound rather than a
/// guess at the real denominator. It refuses an empty envelope for the same
/// reason [`super::tests`]'s validator does: one that could not fail would let
/// the rejection path compile without ever being reachable through it.
fn no_payload(bytes: &[u8]) -> Result<(), ArtifactCodecFailure> {
    if bytes.is_empty() {
        return Err(ArtifactCodecFailure::Malformed {
            detail: "an empty payload is not an artifact".to_owned(),
        });
    }
    Ok(())
}

/// A unique directory for one measurement, removed when the guard drops.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Self {
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the host clock is after the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "tiler-cache-hot-path-{name}-{}-{nonce}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir_all(&path).expect("a scratch directory is creatable");
        Self { path }
    }

    fn root(&self) -> &Path {
        &self.path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

/// Composes the subject one measured entry is filed under.
fn subject_of(envelope_bytes: usize) -> ComposedSubject {
    let compilation = format!("tiler.cache.hot-path.compilation.{envelope_bytes}");
    ComposedSubject::compose(&SubjectFacets {
        backend_compilations: &[compilation.as_bytes()],
        artifact_program: b"tiler.cache.hot-path.artifact-program-stand-in",
    })
    .expect("the fixture names every facet")
}

/// Envelope bytes of a given length, varying so no digest sees a constant run.
fn envelope_of(bytes: usize) -> Vec<u8> {
    (0..bytes)
        .map(|index| u8::try_from(index % 251).expect("a remainder below 251 fits in a byte"))
        .collect()
}

/// Publishes one entry and returns the cache and the key it landed under.
fn published(scratch: &Scratch, envelope_bytes: usize) -> (ExpansionCache, CacheKey) {
    let cache = ExpansionCache::open(scratch.root().join("cache"));
    let subject = subject_of(envelope_bytes);
    let envelope = envelope_of(envelope_bytes);
    let key = CacheKey::derive(&subject);
    cache
        .resolve(
            subject.as_bytes(),
            || Ok::<_, String>(envelope),
            &no_payload,
        )
        .expect("the fixture publishes");
    (cache, key)
}

/// Reports publication time under both durability policies.
///
/// **This is the measurement `measure-expansion-cache-durability-policies`
/// holds open**, and what it can and cannot say is fixed by what `fsync` means
/// on this platform. It measures the *latency* the two policies cost. It says
/// nothing about survival: Darwin's `fsync(2)` documents that data may remain
/// in a device's volatile cache, so no timing here is evidence about power
/// loss, and a survival claim would need `F_FULLFSYNC` and a way to cut power.
///
/// Each publication is a fresh cache under a fresh scratch root, because a
/// second publication of the same subject is a hit and would measure a read.
/// The minimum of many runs is reported for the reason
/// `tiler-compiler`'s harness states: host noise only ever makes a run slower.
#[test]
fn hot_path_publication_by_durability() {
    for envelope_bytes in ENVELOPE_SIZES {
        for (name, durability) in [
            ("process-crash", Durability::ProcessCrash),
            ("fsync", Durability::Fsync),
        ] {
            let repeats = repeats_for(envelope_bytes).min(64);
            let mut best = Duration::MAX;
            let mut total = Duration::ZERO;
            for round in 0..repeats {
                let scratch = Scratch::new(&format!("publish-{name}-{envelope_bytes}-{round}"));
                let cache =
                    ExpansionCache::open(scratch.root().join("cache")).with_durability(durability);
                let subject = subject_of(envelope_bytes);
                let envelope = envelope_of(envelope_bytes);
                let start = Instant::now();
                let outcome = cache
                    .resolve(
                        subject.as_bytes(),
                        || Ok::<_, String>(envelope),
                        &no_payload,
                    )
                    .expect("the fixture publishes");
                let elapsed = start.elapsed();
                // Asserted rather than assumed: a hit here would mean the
                // measurement timed a read under a policy that does not affect
                // reads, and every row would be indistinguishable for a reason
                // that has nothing to do with durability.
                assert!(
                    matches!(
                        outcome,
                        ProtocolOutcome::Hit {
                            published: true,
                            ..
                        }
                    ),
                    "each round must publish rather than hit",
                );
                best = best.min(elapsed);
                total += elapsed;
            }
            println!(
                "MEASURE publish {name} envelope {envelope_bytes}B: min {best:?}, mean {:?} over {repeats}",
                total / repeats,
            );
        }
    }
}

/// Reports validated-read time by bundle size.
///
/// **The scaling is the finding, not the absolute number.** A per-read cost is
/// flat across the three rows and a per-byte cost grows with them, so the shape
/// of the three numbers says which kind of work a hit is made of before any
/// profiler is opened.
#[test]
fn hot_path_read_entry_by_size() {
    for envelope_bytes in ENVELOPE_SIZES {
        let scratch = Scratch::new("read-by-size");
        let (cache, key) = published(&scratch, envelope_bytes);

        // Warm the page cache and the allocator so the first timed read is not
        // measuring a cold file.
        for _ in 0..16 {
            cache.read_entry(&key, &no_payload).expect("a warm hit");
        }

        let repeats = repeats_for(envelope_bytes);
        let mut best = Duration::MAX;
        let total = Instant::now();
        for _ in 0..repeats {
            let start = Instant::now();
            let entry = cache.read_entry(&key, &no_payload).expect("a hit");
            best = best.min(start.elapsed());
            drop(entry);
        }
        println!(
            "MEASURE read_entry envelope {envelope_bytes}B: min {best:?}, mean {:?} over {repeats}",
            total.elapsed() / repeats,
        );
    }
}

/// Enough repetitions that the minimum is a floor rather than a lucky sample,
/// scaled down where one read is expensive.
///
/// Sized against the *debug* profile, which is what the ordinary gate runs and
/// where one read of the largest bundle costs ~13 ms rather than ~2.5 ms. Ten
/// times these counts moved the reported minimum by well under a percent and
/// cost five seconds of every gate run, which is a poor trade for a test that
/// asserts nothing.
const fn repeats_for(envelope_bytes: usize) -> u32 {
    if envelope_bytes >= 128 * 1024 {
        50
    } else {
        200
    }
}

/// Reads in a loop long enough for a sampling profiler to attribute the cost.
///
/// **This is the harness that says *where* the time goes.** It is `#[ignore]`d
/// because it deliberately runs for seconds and asserts nothing. Record it with
/// `samply`:
///
/// ```text
/// CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release --tests -p tiler-cache
/// TILER_PROFILE_SECONDS=20 samply record --save-only --unstable-presymbolicate \
///     --rate 4000 -o cache.profile.json.gz \
///     -- target/release/deps/tiler_cache-<hash> \
///        --ignored --exact expansion::hot_path::hot_path_profile_loop --nocapture
/// ```
///
/// Three details are load-bearing. `CARGO_PROFILE_RELEASE_DEBUG=true` is
/// required: the release profile carries no debug information, and without it
/// every frame symbolicates to a bare hex address. `--unstable-presymbolicate`
/// writes the `*.syms.json` sidecar that holds the names — the profile's own
/// string table does not. And the harness must run long enough to sample.
///
/// `TILER_PROFILE_SECONDS` sets the duration and defaults to ten;
/// `TILER_PROFILE_ENVELOPE_BYTES` sets the bundle size and defaults to the
/// middle of [`ENVELOPE_SIZES`].
#[test]
#[ignore = "runs for seconds under a profiler; not part of the gate"]
fn hot_path_profile_loop() {
    let seconds = std::env::var("TILER_PROFILE_SECONDS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(10);
    let envelope_bytes = std::env::var("TILER_PROFILE_ENVELOPE_BYTES")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(ENVELOPE_SIZES[1]);

    let scratch = Scratch::new("profile-loop");
    let (cache, key) = published(&scratch, envelope_bytes);
    let deadline = Instant::now() + Duration::from_secs(seconds);
    let mut reads = 0_u64;
    while Instant::now() < deadline {
        for _ in 0..64 {
            let _ = cache.read_entry(&key, &no_payload);
        }
        reads += 64;
    }
    println!("MEASURE profile loop: {reads} reads of a {envelope_bytes}B envelope in {seconds}s");
}

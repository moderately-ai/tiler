//! Measures what the expansion cache costs on its hot paths.
//!
//! # What this harness is, and what the crate's own one is not
//!
//! `crates/tiler-cache/src/expansion/hot_path.rs` measures the same protocol
//! with a payload validator that does nothing, and says so: building a real
//! artifact envelope needs a `SemanticProgram`, and ADR 0082 item 2 decides
//! `tiler-cache` does not depend on `tiler-ir`. Its own documentation therefore
//! offers every share it reports as an **upper bound**, and explicitly declines
//! to say that any step dominates a real hit.
//!
//! This harness is the orchestrator that lifts that substitution. It sits
//! outside the workspace, holds `tiler-ir`, `tiler-compiler`, `tiler-artifact`,
//! and `tiler-cache` together, and drives the **public** [`ExpansionCache`] —
//! `get_or_publish`, `lookup`, `account`, `collect` — whose validator is the
//! real [`decode_artifact`]. Every hit measured below decodes a real artifact.
//!
//! # The estimator, and why it is the minimum
//!
//! Every perturbation a host applies makes an operation *slower* and none makes
//! it faster, so the distribution has a hard floor at the true cost and an
//! unbounded tail of whatever else the machine is doing. The minimum of enough
//! runs estimates that floor. The median, the ninetieth percentile, and the
//! maximum are reported beside it so a reader can see how loaded the host was
//! rather than having to trust that it was not — a run whose maximum is two
//! orders of magnitude above its minimum was measured under load, and the row
//! says so.
//!
//! # Nothing here asserts a time
//!
//! A timing assertion fails on a loaded machine and passes on a fast one, which
//! makes it a flake rather than a guard. What *is* asserted is everything that
//! can be made deterministic: the returned bytes equal the published bytes on
//! every measured configuration, the population is counted before it is
//! reported, the corrupted-entry control is observed to be refused, and the
//! contended-lock control is observed to be genuinely held.
//!
//! # Reproducing
//!
//! Run it by hand from this directory. See `../README.md` for the recorded
//! invocation, the environment, and the retained results.

mod envelope;

use std::env;
use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tiler_artifact::program::{DigestAlgorithm, decode_artifact};
use tiler_cache::expansion::{
    CacheKey, CollectionBound, CollectionOutcome, ComposedSubject, Durability, ExpansionCache,
    Lookup, MaxEntryAge, Resolution, SubjectFacets,
};

use envelope::EnvelopeFactory;

/// Envelope lengths the sweep reports, in bytes.
///
/// Not round numbers, deliberately: these are the exact endpoints of the
/// envelope band `docs/research/embedding/self-contained-embedding.md` measured
/// and `MaxEntryAge::DEFAULT`'s ground cites — 32,136 to 47,803 bytes. Measuring
/// at the band's own endpoints is what lets a reader put a cost against the
/// sizes the corpus already claims are realistic, instead of against a round
/// number nothing produced.
const SIZES: [usize; 2] = [32_136, 47_803];

/// Populated entry counts the sweep measures at, cumulative in one cache root.
const DEFAULT_POPULATIONS: [u64; 4] = [10, 100, 1_000, 10_000];

/// The artifact-program facet every subject in this harness names.
///
/// A stand-in, exactly as `spikes/cache/build-tool-exercise` uses one and for
/// the same reason: no producer exists for `SubjectFacet::ArtifactProgram`. This
/// spike measures cost, for which a subject only has to be a stable function of
/// the invocation; it is not evidence about identity completeness.
const ARTIFACT_PROGRAM_STAND_IN: &[u8] = b"tiler.spike.hot-path.artifact-program-stand-in";

/// Domain separator of one framed bundle section's content digest.
///
/// **A restatement, not an import.** `bundle::SECTION_DIGEST_DOMAIN` is
/// `pub(super)` inside `tiler-cache`, so the decomposition below cannot call the
/// crate's own digest site and reproduces it instead. The restatement is exact
/// as of this spike's base commit, and the cost it attributes would be identical
/// under any domain of any length, because the domain is a fixed prefix hashed
/// once ahead of a section that is four orders of magnitude larger.
const BUNDLE_SECTION_DIGEST_DOMAIN: &[u8] = b"tiler.cache.bundle-section.v1\0";

/// Domain the oracle digest a child process reports is taken under.
///
/// This harness's own domain, never one the cache or the artifact layer uses:
/// the value travels between two processes of this spike and must not be
/// confusable with an identity either of those layers derives.
const ORACLE_DIGEST_DOMAIN: &[u8] = b"tiler.spike.hot-path.oracle.v1\0";

/// Fixed-width framing header of one cache bundle, before the descriptor table.
///
/// Restated from `bundle.rs` for the same reason as the digest domain above, and
/// checked rather than trusted: [`bundle_spans`] asserts that the spans it
/// derives from these constants actually contain the published envelope, so a
/// framing change fails this spike loudly instead of silently digesting the
/// wrong run.
const BUNDLE_HEADER_BYTES: usize = 64;
/// One bundle section descriptor: purpose, offset, length, digest.
const BUNDLE_DESCRIPTOR_BYTES: usize = 4 + 8 + 8 + 32;
/// Sections one published bundle frames.
const BUNDLE_SECTIONS: usize = 2;

/// How long a process waiting on another's marker file sleeps between checks.
///
/// Not an ordering mechanism: every wait below ends when the marker it names
/// exists and at no other time. This is only how often the waiter asks, and it
/// exists because a spin loop is a process competing with the measurement.
const POLL_INTERVAL: Duration = Duration::from_millis(1);

/// Leading characters of a rendered key that name its shard directory.
const SHARD_BYTES: usize = 2;
/// Namespace version directory the layout joins exactly.
const NAMESPACE_VERSION: &str = "v1";

fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        Some("child-hit") => child_hit(&arguments[1..]),
        Some("child-hold-lock") => child_hold_lock(&arguments[1..]),
        _ => run(&Options::parse(&arguments)),
    }
}

// -- options ---------------------------------------------------------------

/// What one run measures, and how hard.
struct Options {
    populations: Vec<u64>,
    repeats: usize,
    warmup: usize,
    cold_children: usize,
    publish_rounds: usize,
    scan_repeats: usize,
    record: Option<String>,
    keep: bool,
}

impl Options {
    fn parse(arguments: &[String]) -> Self {
        let mut options = Self {
            populations: DEFAULT_POPULATIONS.to_vec(),
            repeats: 2_000,
            warmup: 64,
            cold_children: 32,
            publish_rounds: 64,
            scan_repeats: 5,
            record: None,
            keep: false,
        };
        let mut index = 0;
        while index < arguments.len() {
            let argument = arguments[index].as_str();
            let mut value = || {
                index += 1;
                arguments
                    .get(index)
                    .unwrap_or_else(|| panic!("{argument} takes a value"))
                    .clone()
            };
            match argument {
                "--populations" => {
                    options.populations = value()
                        .split(',')
                        .map(|entry| entry.trim().parse().expect("a population is a number"))
                        .collect();
                }
                "--repeats" => options.repeats = value().parse().expect("a count"),
                "--warmup" => options.warmup = value().parse().expect("a count"),
                "--cold-children" => options.cold_children = value().parse().expect("a count"),
                "--publish-rounds" => options.publish_rounds = value().parse().expect("a count"),
                "--scan-repeats" => options.scan_repeats = value().parse().expect("a count"),
                "--record" => options.record = Some(value()),
                "--keep" => options.keep = true,
                "--quick" => {
                    options.populations = vec![10, 100];
                    options.repeats = 200;
                    options.cold_children = 4;
                    options.publish_rounds = 8;
                }
                other => panic!("unknown argument `{other}`"),
            }
            index += 1;
        }
        assert!(
            !options.populations.is_empty(),
            "a sweep needs at least one population",
        );
        assert!(
            options.populations.windows(2).all(|pair| pair[0] < pair[1]),
            "populations are cumulative and must be strictly increasing",
        );
        options
    }
}

// -- the run ---------------------------------------------------------------

fn run(options: &Options) -> ExitCode {
    let scratch = Scratch::new(options.keep);
    let mut recorder = Recorder::default();

    recorder.comment(format!(
        "cache-hot-path-efficiency; host={} {}; cpus={}; rustc={}",
        host_line("kern.ostype"),
        host_line("kern.osrelease"),
        host_line("hw.ncpu"),
        rustc_version(),
    ));
    recorder.comment(format!("cpu={}", host_line("machdep.cpu.brand_string")));
    recorder.comment(format!("scratch root={}", scratch.path().display()));
    recorder.comment(format!("load average at start={}", host_line("vm.loadavg")));
    recorder.comment(format!(
        "repeats={} warmup={} cold-children={} publish-rounds={} scan-repeats={} populations={:?}",
        options.repeats,
        options.warmup,
        options.cold_children,
        options.publish_rounds,
        options.scan_repeats,
        options.populations,
    ));

    let started = Instant::now();
    let factory = EnvelopeFactory::new();
    recorder.comment(format!(
        "envelope fixed overhead={} bytes; compiled in {:?}",
        factory.base_bytes(),
        started.elapsed(),
    ));

    oracle_and_negative_control(&mut recorder, scratch.path(), &factory);
    publish_latency(&mut recorder, scratch.path(), &factory, options);
    for size in SIZES {
        sweep_one_size(&mut recorder, scratch.path(), &factory, options, size);
    }

    recorder.comment(format!("load average at end={}", host_line("vm.loadavg")));
    recorder.comment(format!("total wall clock={:?}", started.elapsed()));

    if let Some(name) = &options.record {
        // Relative to the spike root, which is the directory the README records
        // the invocation from.
        let path = PathBuf::from("results").join(format!("hot-path-efficiency-{name}.tsv"));
        recorder.write(&path);
        println!("# recorded to {}", path.display());
    }
    ExitCode::SUCCESS
}

/// Proves the oracle can fail before any number is reported.
///
/// Three observations per size, in order, and the middle one is the whole point:
/// a comparison that always passes is indistinguishable from one that never
/// runs. Flipping a single byte of the stored envelope and watching the read
/// refuse is what makes the two equalities either side of it evidence.
fn oracle_and_negative_control(
    recorder: &mut Recorder,
    run_root: &Path,
    factory: &EnvelopeFactory,
) {
    for size in SIZES {
        let root = run_root.join(format!("oracle-{size}"));
        let cache = ExpansionCache::open(&root);
        let subject = subject_for(0);
        let published = factory.exactly(size);

        let payload = published.clone();
        let resolution = cache
            .get_or_publish(&subject, || Ok::<_, String>(payload))
            .expect("the fixture publishes");
        let Resolution::Published { entry, .. } = resolution else {
            panic!("the first resolution of a fresh root must publish");
        };
        assert_eq!(
            entry.envelope_bytes(),
            published.as_slice(),
            "a publication must return the exact bytes it was given",
        );
        drop(entry);

        let Lookup::Hit(entry) = cache.lookup(&subject) else {
            panic!("the entry just published must be a hit");
        };
        assert_eq!(
            entry.envelope_bytes(),
            published.as_slice(),
            "a hit must return the exact published bytes",
        );
        drop(entry);
        recorder.fact(
            "oracle",
            size,
            1,
            "returned-bytes-equal-published",
            "publication and hit both returned the exact published bytes",
        );

        let key = CacheKey::derive(&subject);
        let path = entry_path(&root, &key);
        let mut stored = fs::read(&path).expect("the published entry is readable");
        let at = stored.len() - 1;
        let original = stored[at];
        stored[at] ^= 0x01;
        fs::write(&path, &stored).expect("the entry is writable");
        let Lookup::Miss(reason) = cache.lookup(&subject) else {
            panic!("an entry with one flipped byte must not be a hit");
        };
        recorder.fact(
            "oracle",
            size,
            1,
            "one-flipped-byte-refused",
            &format!("{reason}"),
        );

        stored[at] = original;
        fs::write(&path, &stored).expect("the entry is writable");
        let Lookup::Hit(entry) = cache.lookup(&subject) else {
            panic!("restoring the flipped byte must restore the hit");
        };
        assert_eq!(
            entry.envelope_bytes(),
            published.as_slice(),
            "the restored entry must return the exact published bytes",
        );
        drop(entry);
        recorder.fact(
            "oracle",
            size,
            1,
            "restored-byte-restores-hit",
            "the same byte restored returns the exact published bytes",
        );
    }
}

/// Reports publication latency, which includes the atomic rename.
///
/// Each round is a fresh cache root, because a second publication of one
/// subject is a hit and would silently measure a read. That the round published
/// rather than hit is asserted, not assumed.
fn publish_latency(
    recorder: &mut Recorder,
    run_root: &Path,
    factory: &EnvelopeFactory,
    options: &Options,
) {
    for size in SIZES {
        let envelope = factory.exactly(size);
        for (name, durability) in [
            ("process-crash", Durability::ProcessCrash),
            ("fsync", Durability::Fsync),
        ] {
            let mut samples = Vec::with_capacity(options.publish_rounds);
            for round in 0..options.publish_rounds {
                let root = run_root.join(format!("publish-{size}-{name}-{round}"));
                let cache = ExpansionCache::open(&root).with_durability(durability);
                let subject = subject_for(0);
                let payload = envelope.clone();
                let (elapsed, resolution) =
                    timed(|| cache.get_or_publish(&subject, || Ok::<_, String>(payload)));
                let Resolution::Published { entry, .. } =
                    resolution.expect("the fixture publishes")
                else {
                    panic!("each round must publish rather than hit");
                };
                assert_eq!(
                    entry.envelope_bytes(),
                    envelope.as_slice(),
                    "a publication must return the exact bytes it was given",
                );
                drop(entry);
                samples.push(elapsed);
                fs::remove_dir_all(&root).expect("a scratch root is removable");
            }
            recorder.timing("publish", size, 1, name, &summarize(samples));
        }

        // The rename alone, over a file of the same length inside one shard, so
        // the share atomic publication costs is attributed rather than inferred
        // from the difference between two durability policies.
        let root = run_root.join(format!("rename-{size}"));
        let shard = root.join("shard");
        fs::create_dir_all(&shard).expect("a scratch directory is creatable");
        let mut samples = Vec::with_capacity(options.publish_rounds);
        for round in 0..options.publish_rounds {
            let from = shard.join(format!("temporary-{round}"));
            let to = shard.join("published");
            fs::write(&from, &envelope).expect("a scratch file is writable");
            let (elapsed, outcome) = timed(|| fs::rename(&from, &to));
            outcome.expect("a rename inside one directory succeeds");
            samples.push(elapsed);
        }
        recorder.timing("publish", size, 1, "bare-rename", &summarize(samples));
        fs::remove_dir_all(&root).expect("a scratch root is removable");
    }
}

/// Fills one cache root to each population in turn, measuring at every step.
fn sweep_one_size(
    recorder: &mut Recorder,
    run_root: &Path,
    factory: &EnvelopeFactory,
    options: &Options,
    size: usize,
) {
    let root = run_root.join(format!("sweep-{size}"));
    let cache = ExpansionCache::open(&root);
    let envelope = factory.exactly(size);
    let probe = subject_for(0);
    let oracle = DigestAlgorithm::GOVERNED
        .digest(ORACLE_DIGEST_DOMAIN, &envelope)
        .label();

    let mut published = 0_u64;
    let largest = *options.populations.last().expect("a non-empty sweep");
    for population in &options.populations {
        while published < *population {
            let subject = subject_for(published);
            let payload = envelope.clone();
            let resolution = cache
                .get_or_publish(&subject, || Ok::<_, String>(payload))
                .expect("the fixture publishes");
            assert!(
                matches!(resolution, Resolution::Published { .. }),
                "filling a fresh key must publish rather than hit",
            );
            published += 1;
        }

        // The population is counted from the namespace rather than from this
        // loop's own counter, so a fill that silently lost an entry is a failure
        // instead of a row reporting a population it never reached.
        let accounting = cache.account().expect("the namespace is scannable");
        assert_eq!(
            accounting.entry_count(),
            *population,
            "the scan must find exactly the entries the fill published",
        );

        warm_hits(recorder, &cache, &probe, &envelope, size, *population, options);
        cold_process_hits(recorder, &root, &oracle, size, *population, options);
        scan_costs(recorder, &cache, size, *population, options);
    }

    decompose_one_hit(recorder, &root, &cache, &probe, &envelope, size, largest);
    lock_costs(recorder, &root, &cache, &probe, &envelope, size, largest, options);
    destructive_collection(recorder, &cache, size, largest);

    fs::remove_dir_all(&root).expect("a scratch root is removable");
}

/// Steady-state hit latency in one long-lived process.
///
/// This is the `rust-analyzer` process pattern: one server holding the loaded
/// proc-macro dylib across edits, expanding repeatedly, with its allocator and
/// the page cache both warm. It is the *cheapest* hit the cache serves, which is
/// why it is reported beside the cold-process row rather than alone.
fn warm_hits(
    recorder: &mut Recorder,
    cache: &ExpansionCache,
    probe: &ComposedSubject,
    envelope: &[u8],
    size: usize,
    population: u64,
    options: &Options,
) {
    for _ in 0..options.warmup {
        let Lookup::Hit(entry) = cache.lookup(probe) else {
            panic!("the probe entry must be a hit");
        };
        drop(entry);
    }
    let mut samples = Vec::with_capacity(options.repeats);
    for _ in 0..options.repeats {
        let (elapsed, lookup) = timed(|| cache.lookup(probe));
        let Lookup::Hit(entry) = lookup else {
            panic!("the probe entry must be a hit");
        };
        // Outside the timed region, so the oracle costs the measurement nothing
        // and still runs on every single sample.
        assert_eq!(
            entry.envelope_bytes(),
            envelope,
            "every measured hit must return the exact published bytes",
        );
        drop(entry);
        samples.push(elapsed);
    }
    recorder.timing("hit-warm", size, population, "lookup", &summarize(samples));
}

/// The first and only hit a freshly started process takes.
///
/// This is the `cargo` process pattern: one `rustc` per crate, which expands and
/// exits. Each sample is a separate process that re-executes this binary and
/// times exactly one `lookup`.
///
/// **Its measurement boundary, stated rather than implied.** The child is cold
/// in *process* state — no warmed allocator, no warmed branch predictor, no
/// prior lookup — and warm in *page cache* state, because the parent published
/// the entry moments earlier on the same host. Purging the unified buffer cache
/// needs privileges this spike does not take, so a genuinely cold-storage first
/// hit is unmeasured here.
fn cold_process_hits(
    recorder: &mut Recorder,
    root: &Path,
    oracle: &str,
    size: usize,
    population: u64,
    options: &Options,
) {
    let executable = env::current_exe().expect("this binary has a path");
    let mut samples = Vec::with_capacity(options.cold_children);
    for _ in 0..options.cold_children {
        let output = Command::new(&executable)
            .arg("child-hit")
            .arg(root)
            .arg("0")
            .arg(oracle)
            .output()
            .expect("a child process starts");
        assert!(
            output.status.success(),
            "a cold-hit child failed: {}",
            String::from_utf8_lossy(&output.stderr),
        );
        let reported = String::from_utf8(output.stdout).expect("a child reports UTF-8");
        let elapsed = reported
            .trim()
            .strip_prefix("ns=")
            .and_then(|value| value.parse().ok())
            .unwrap_or_else(|| panic!("a child reports `ns=<n>`, got {reported:?}"));
        samples.push(elapsed);
    }
    recorder.timing("hit-cold", size, population, "lookup", &summarize(samples));
}

/// The cost of every whole-namespace scan a collection performs.
///
/// Three rows, and the difference between them is the finding rather than any
/// one of them: `account` is the walk, `collect` under `UNBOUNDED` is the walk
/// plus the selector's early return, and `collect` under an age nothing has
/// reached is the walk plus the clone and the sort the age predicate forces.
fn scan_costs(
    recorder: &mut Recorder,
    cache: &ExpansionCache,
    size: usize,
    population: u64,
    options: &Options,
) {
    let mut samples = Vec::with_capacity(options.scan_repeats);
    for _ in 0..options.scan_repeats {
        let (elapsed, accounting) = timed(|| cache.account());
        let accounting = accounting.expect("the namespace is scannable");
        assert_eq!(accounting.entry_count(), population, "the scan is complete");
        samples.push(elapsed);
    }
    recorder.timing("scan", size, population, "account", &summarize(samples));

    let mut samples = Vec::with_capacity(options.scan_repeats);
    for _ in 0..options.scan_repeats {
        let (elapsed, report) = timed(|| cache.collect(&CollectionBound::UNBOUNDED));
        let report = report.expect("the namespace is scannable");
        assert_eq!(report.selected(), 0, "an unbounded collection selects nothing");
        samples.push(elapsed);
    }
    recorder.timing(
        "scan",
        size,
        population,
        "collect-unbounded",
        &summarize(samples),
    );

    let retaining = CollectionBound {
        max_total_bytes: None,
        max_entries: None,
        max_entry_age: Some(
            MaxEntryAge::new(Duration::from_secs(365 * 24 * 60 * 60))
                .expect("a year is not zero"),
        ),
    };
    let mut samples = Vec::with_capacity(options.scan_repeats);
    for _ in 0..options.scan_repeats {
        let (elapsed, report) = timed(|| cache.collect(&retaining));
        let report = report.expect("the namespace is scannable");
        assert_eq!(
            report.selected(),
            0,
            "an age no entry has reached selects nothing",
        );
        assert!(
            matches!(report.outcome(), CollectionOutcome::WithinBound),
            "a selection of nothing is within the bound",
        );
        samples.push(elapsed);
    }
    recorder.timing(
        "scan",
        size,
        population,
        "collect-age-retaining-all",
        &summarize(samples),
    );
}

/// Attributes one hit's cost to the steps a hit is made of.
///
/// **Every component below is a reimplementation, and that is the boundary.**
/// `read_entry`, `bundle::decode`, and the entry-path parser are crate-private,
/// so this cannot call them and reproduces each instead from the public digest,
/// the public key derivation, and `std`. Two things keep the reproduction
/// honest: [`bundle_spans`] asserts that the spans it derives from the restated
/// frame constants actually delimit the published envelope, so a framing change
/// fails here rather than being digested around; and the residual row reports
/// what the components do *not* account for, so a component this harness has
/// mis-modelled shows up as a large residual instead of disappearing.
fn decompose_one_hit(
    recorder: &mut Recorder,
    root: &Path,
    cache: &ExpansionCache,
    probe: &ComposedSubject,
    envelope: &[u8],
    size: usize,
    population: u64,
) {
    let key = CacheKey::derive(probe);
    let path = entry_path(root, &key);
    let stored = fs::read(&path).expect("the probe entry is readable");
    let (subject_span, envelope_span) = bundle_spans(&stored, probe, envelope);

    // Sized to keep each component's total work comparable with the whole-hit
    // row without making the decomposition itself take minutes.
    let repeats = 2_000;

    let mut whole = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let (elapsed, lookup) = timed(|| cache.lookup(probe));
        let Lookup::Hit(entry) = lookup else {
            panic!("the probe entry must be a hit");
        };
        assert_eq!(entry.envelope_bytes(), envelope, "the oracle holds");
        drop(entry);
        whole.push(elapsed);
    }
    let whole = summarize(whole);
    recorder.timing("decompose", size, population, "whole-lookup", &whole);

    let mut components: Vec<(&str, u64)> = Vec::new();

    let mut samples = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let (elapsed, bytes) = timed(|| read_bounded(&path));
        assert_eq!(bytes.len(), stored.len(), "the read is complete");
        samples.push(elapsed);
    }
    let summary = summarize(samples);
    components.push(("open-and-read", summary.min));
    recorder.timing("decompose", size, population, "open-and-read", &summary);

    let mut samples = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let (elapsed, digests) = timed(|| {
            (
                DigestAlgorithm::GOVERNED
                    .digest(BUNDLE_SECTION_DIGEST_DOMAIN, &stored[subject_span.clone()]),
                DigestAlgorithm::GOVERNED
                    .digest(BUNDLE_SECTION_DIGEST_DOMAIN, &stored[envelope_span.clone()]),
            )
        });
        assert_ne!(digests.0, digests.1, "two sections digest differently");
        samples.push(elapsed);
    }
    let summary = summarize(samples);
    components.push(("bundle-section-digests", summary.min));
    recorder.timing(
        "decompose",
        size,
        population,
        "bundle-section-digests",
        &summary,
    );

    // Twice per hit: once to form the requested key, once to re-derive it from
    // the subject the bundle carries. The re-derivation is what refuses a bundle
    // filed under a key its own subject does not produce.
    let mut samples = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let (elapsed, derived) = timed(|| (CacheKey::derive(probe), CacheKey::derive(probe)));
        assert_eq!(derived.0, key, "the derivation is a function of the subject");
        samples.push(elapsed);
    }
    let summary = summarize(samples);
    components.push(("key-derivations", summary.min));
    recorder.timing("decompose", size, population, "key-derivations", &summary);

    let mut samples = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let (elapsed, formed) = timed(|| {
            let formed = entry_path(root, &key);
            let parsed = parse_label(&formed);
            (formed, parsed)
        });
        assert_eq!(formed.0, path, "the path is the one the entry lives at");
        assert_eq!(formed.1, key.label(), "the label round-trips through a path");
        samples.push(elapsed);
    }
    let summary = summarize(samples);
    components.push(("path-form-and-parse", summary.min));
    recorder.timing(
        "decompose",
        size,
        population,
        "path-form-and-parse",
        &summary,
    );

    let mut samples = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let (elapsed, decoded) = timed(|| decode_artifact(&stored[envelope_span.clone()]));
        decoded.expect("the stored envelope decodes");
        samples.push(elapsed);
    }
    let summary = summarize(samples);
    components.push(("decode-artifact", summary.min));
    recorder.timing("decompose", size, population, "decode-artifact", &summary);

    let accounted: u64 = components.iter().map(|(_, minimum)| *minimum).sum();
    recorder.fact(
        "decompose",
        size,
        population,
        "residual",
        &format!(
            "whole minimum {} ns; components sum to {} ns; residual {} ns ({:.1}% of the whole)",
            whole.min,
            accounted,
            whole.min.saturating_sub(accounted),
            percent(whole.min.saturating_sub(accounted), whole.min),
        ),
    );
    for (name, minimum) in &components {
        recorder.fact(
            "decompose",
            size,
            population,
            "share",
            &format!(
                "{name}: {minimum} ns, {:.1}% of the whole hit",
                percent(*minimum, whole.min),
            ),
        );
    }
    recorder.fact(
        "decompose",
        size,
        population,
        "validation-share",
        &format!(
            "fail-closed integrity (bundle section digests + decode_artifact) costs {} ns, {:.1}% \
             of the whole hit",
            components
                .iter()
                .filter(|(name, _)| *name == "bundle-section-digests" || *name == "decode-artifact")
                .map(|(_, minimum)| *minimum)
                .sum::<u64>(),
            percent(
                components
                    .iter()
                    .filter(|(name, _)| *name == "bundle-section-digests"
                        || *name == "decode-artifact")
                    .map(|(_, minimum)| *minimum)
                    .sum::<u64>(),
                whole.min,
            ),
        ),
    );

    // What a caller pays to *own* the envelope. The hit path itself performs no
    // such copy: a stored entry keeps the buffer the read allocated and hands
    // out spans of it, so `envelope_bytes` is a borrow. This row is the cost a
    // consumer adds when it needs an owned `Vec`, measured rather than assumed
    // to be free.
    let Lookup::Hit(entry) = cache.lookup(probe) else {
        panic!("the probe entry must be a hit");
    };
    let mut samples = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let (elapsed, owned) = timed(|| entry.envelope_bytes().to_vec());
        assert_eq!(owned.len(), envelope.len(), "the copy is complete");
        samples.push(elapsed);
    }
    recorder.timing(
        "decompose",
        size,
        population,
        "caller-owned-copy",
        &summarize(samples),
    );
    drop(entry);
}

/// What the per-key lock costs, and the evidence that a hit never takes one.
///
/// The positive claim — a validated hit is lock-free — is not argued from the
/// source here. It is observed: a *separate process* holds the probe key's lock,
/// this process proves the lock is genuinely held by being refused it, and the
/// hit is served anyway. Had the read path taken the lock, it would have blocked
/// until the holder was released, which is an observable difference rather than
/// a margin.
fn lock_costs(
    recorder: &mut Recorder,
    root: &Path,
    cache: &ExpansionCache,
    probe: &ComposedSubject,
    envelope: &[u8],
    size: usize,
    population: u64,
    options: &Options,
) {
    let key = CacheKey::derive(probe);
    let lock = lock_path(root, &key);
    assert!(
        lock.is_file(),
        "publication creates the key's stable lock file at {}",
        lock.display(),
    );

    let repeats = options.repeats.min(2_000);
    let mut samples = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let (elapsed, held) = timed(|| {
            let held = open_lock(&lock);
            held.lock().expect("an uncontended lock is takeable");
            held.unlock().expect("a held lock is releasable");
            held
        });
        drop(held);
        samples.push(elapsed);
    }
    recorder.timing(
        "lock",
        size,
        population,
        "acquire-release-uncontended",
        &summarize(samples),
    );

    let holder_ready = root.join("holder-ready");
    let holder_release = root.join("holder-release");
    let _ = fs::remove_file(&holder_ready);
    let _ = fs::remove_file(&holder_release);
    let executable = env::current_exe().expect("this binary has a path");
    let mut child = Command::new(&executable)
        .arg("child-hold-lock")
        .arg(&lock)
        .arg(&holder_ready)
        .arg(&holder_release)
        .spawn()
        .expect("a child process starts");

    // Ordering by observed state, never by a wall-clock margin: the child
    // creates the marker *after* it holds the lock, so its existence is the
    // event this loop waits for. The interval between polls is a courtesy to
    // the scheduler and not part of the ordering — the loop ends when the
    // marker exists and at no other time.
    while !holder_ready.exists() {
        assert!(
            child
                .try_wait()
                .expect("a child is waitable")
                .is_none_or(|status| status.success()),
            "the lock holder exited before it took the lock",
        );
        std::thread::sleep(POLL_INTERVAL);
    }

    // Proof the lock is genuinely held. Without it, "the hit was served" would
    // be consistent with a lock nobody was holding.
    let contender = open_lock(&lock);
    match contender.try_lock() {
        Err(TryLockError::WouldBlock) => recorder.fact(
            "lock",
            size,
            population,
            "held-by-another-process",
            "a non-blocking acquisition was refused while the child held the lock",
        ),
        Ok(()) => panic!("the child's lock was not held, so the contended row proves nothing"),
        Err(TryLockError::Error(error)) => panic!("the lock could not be probed: {error}"),
    }
    drop(contender);

    let mut samples = Vec::with_capacity(repeats);
    for _ in 0..repeats {
        let (elapsed, lookup) = timed(|| cache.lookup(probe));
        let Lookup::Hit(entry) = lookup else {
            panic!("a hit must be served while another process holds the key lock");
        };
        assert_eq!(entry.envelope_bytes(), envelope, "the oracle holds");
        drop(entry);
        samples.push(elapsed);
    }
    recorder.timing(
        "lock",
        size,
        population,
        "hit-while-key-lock-held",
        &summarize(samples),
    );

    fs::write(&holder_release, b"release").expect("the release marker is writable");
    let status = child.wait().expect("the lock holder is waitable");
    assert!(status.success(), "the lock holder exited cleanly");
    let _ = fs::remove_file(&holder_ready);
    let _ = fs::remove_file(&holder_release);
}

/// The cost of a collection that actually removes everything it scans.
///
/// Destructive, so it runs last against its root. The removal count is asserted
/// against the counted population and the report is required to account for its
/// whole selection, so a pass cannot mean "the collector did nothing quickly".
fn destructive_collection(
    recorder: &mut Recorder,
    cache: &ExpansionCache,
    size: usize,
    population: u64,
) {
    let expiring = CollectionBound {
        max_total_bytes: None,
        max_entries: None,
        max_entry_age: Some(MaxEntryAge::new(Duration::from_nanos(1)).expect("one nanosecond")),
    };
    let (elapsed, report) = timed(|| cache.collect(&expiring));
    let report = report.expect("the namespace is scannable");
    assert_eq!(
        report.removed().len() as u64,
        population,
        "an age of one nanosecond expires every datable entry",
    );
    assert!(
        report.accounts_for_every_entry(),
        "every selected entry has exactly one recorded disposition",
    );
    recorder.timing(
        "scan",
        size,
        population,
        "collect-age-removing-all",
        &Summary {
            samples: 1,
            min: elapsed,
            p50: elapsed,
            p90: elapsed,
            max: elapsed,
        },
    );
    recorder.fact(
        "scan",
        size,
        population,
        "removal-cost-per-entry",
        &format!(
            "{} ns over {population} entries, {} ns each, {} bytes reclaimed",
            elapsed,
            elapsed / population.max(1),
            report.reclaimed_bytes(),
        ),
    );
}

// -- child modes -----------------------------------------------------------

/// Times exactly one `lookup` in a freshly started process.
fn child_hit(arguments: &[String]) -> ExitCode {
    let root = PathBuf::from(&arguments[0]);
    let index: u64 = arguments[1].parse().expect("a subject index");
    let expected = &arguments[2];

    let cache = ExpansionCache::open(&root);
    let subject = subject_for(index);
    let (elapsed, lookup) = timed(|| cache.lookup(&subject));
    let Lookup::Hit(entry) = lookup else {
        eprintln!("the child did not observe a hit");
        return ExitCode::FAILURE;
    };
    let observed = DigestAlgorithm::GOVERNED
        .digest(ORACLE_DIGEST_DOMAIN, entry.envelope_bytes())
        .label();
    if &observed != expected {
        eprintln!("the child observed {observed} and expected {expected}");
        return ExitCode::FAILURE;
    }
    println!("ns={elapsed}");
    ExitCode::SUCCESS
}

/// Holds one key's lock until the parent says to let go.
fn child_hold_lock(arguments: &[String]) -> ExitCode {
    let lock = PathBuf::from(&arguments[0]);
    let ready = PathBuf::from(&arguments[1]);
    let release = PathBuf::from(&arguments[2]);

    let held = open_lock(&lock);
    held.lock().expect("the child takes the lock");
    fs::write(&ready, b"held").expect("the ready marker is writable");
    // Sleeping between polls rather than spinning, because this child is alive
    // for the whole of the contended-hit measurement and a spin loop is a
    // second process competing for a core and for the filesystem with the very
    // thing being timed. It measured 5–10%: the contended row read that much
    // above the uncontended one until this loop stopped burning a core.
    while !release.exists() {
        std::thread::sleep(POLL_INTERVAL);
    }
    held.unlock().expect("the child releases the lock");
    ExitCode::SUCCESS
}

// -- shared helpers --------------------------------------------------------

/// Composes the subject one measured entry is filed under.
fn subject_for(index: u64) -> ComposedSubject {
    let compilation = format!("tiler.spike.hot-path.compilation.{index}");
    ComposedSubject::compose(&SubjectFacets {
        backend_compilations: &[compilation.as_bytes()],
        artifact_program: ARTIFACT_PROGRAM_STAND_IN,
    })
    .expect("the fixture names every facet")
}

/// `<root>/v1/entries/<K[0..2]>/<K>.bundle`, restated from `layout.rs`.
fn entry_path(root: &Path, key: &CacheKey) -> PathBuf {
    let label = key.label();
    root.join(NAMESPACE_VERSION)
        .join("entries")
        .join(&label[..SHARD_BYTES])
        .join(format!("{label}.bundle"))
}

/// `<root>/v1/locks/<K[0..2]>/<K>.lock`, restated from `layout.rs`.
fn lock_path(root: &Path, key: &CacheKey) -> PathBuf {
    let label = key.label();
    root.join(NAMESPACE_VERSION)
        .join("locks")
        .join(&label[..SHARD_BYTES])
        .join(format!("{label}.lock"))
}

/// Reads a key's rendered label back out of its entry path.
///
/// The width and alphabet checks `CacheKey::parse_label` performs, restated so
/// the decomposition can attribute their cost; the crate's own parser is
/// crate-private.
fn parse_label(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .expect("an entry path names a file");
    let label = name
        .strip_suffix(".bundle")
        .expect("an entry file name ends in the bundle extension");
    assert_eq!(label.len(), 64, "a rendered key is 64 bytes wide");
    for byte in label.as_bytes() {
        assert!(
            byte.is_ascii_digit() || (b'a'..=b'f').contains(byte),
            "a rendered key is lowercase hexadecimal",
        );
    }
    let shard = path
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .expect("an entry path names a shard");
    assert_eq!(shard, &label[..SHARD_BYTES], "the entry sits in its shard");
    label.to_owned()
}

/// Opens one lock file exactly as `KeyLock::open` does.
fn open_lock(path: &Path) -> File {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .expect("a lock file is openable")
}

/// Reads a file the way `read_bounded` does: one exact-capacity allocation.
fn read_bounded(path: &Path) -> Vec<u8> {
    let limit = 256 * 1024 * 1024_u64;
    let file = File::open(path).expect("the entry is openable");
    let hint = file
        .metadata()
        .ok()
        .map(|metadata| metadata.len().min(limit))
        .and_then(|bytes| usize::try_from(bytes).ok())
        .unwrap_or_default();
    let mut bytes = Vec::with_capacity(hint);
    let mut read = file.take(limit.saturating_add(1));
    read.read_to_end(&mut bytes).expect("the entry is readable");
    bytes
}

/// Locates the two framed sections inside a published bundle.
///
/// Derived from the restated frame constants and then **checked against the
/// bytes**: the envelope run must equal the envelope that was published. A
/// framing change therefore fails here rather than silently pointing the
/// decomposition's digests at the wrong bytes.
fn bundle_spans(
    stored: &[u8],
    subject: &ComposedSubject,
    envelope: &[u8],
) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
    let table_end = BUNDLE_HEADER_BYTES + BUNDLE_DESCRIPTOR_BYTES * BUNDLE_SECTIONS;
    let subject_bytes = subject.as_bytes().len();
    let subject_span = table_end..table_end + subject_bytes;
    let envelope_span = subject_span.end..subject_span.end + envelope.len();
    assert_eq!(
        stored.len(),
        envelope_span.end,
        "the restated bundle frame accounts for every stored byte",
    );
    assert_eq!(
        &stored[subject_span.clone()],
        subject.as_bytes(),
        "the restated frame locates the carried subject",
    );
    assert_eq!(
        &stored[envelope_span.clone()],
        envelope,
        "the restated frame locates the carried envelope",
    );
    (subject_span, envelope_span)
}

/// Runs `f`, returning the nanoseconds it took beside its value.
fn timed<T>(f: impl FnOnce() -> T) -> (u64, T) {
    let start = Instant::now();
    let value = f();
    let elapsed = start.elapsed();
    (
        u64::try_from(elapsed.as_nanos()).expect("a measured operation fits 584 years"),
        value,
    )
}

/// One measured distribution.
struct Summary {
    samples: usize,
    min: u64,
    p50: u64,
    p90: u64,
    max: u64,
}

fn summarize(mut samples: Vec<u64>) -> Summary {
    assert!(!samples.is_empty(), "a summary needs at least one sample");
    samples.sort_unstable();
    let at = |fraction: usize| samples[(samples.len() - 1) * fraction / 100];
    Summary {
        samples: samples.len(),
        min: samples[0],
        p50: at(50),
        p90: at(90),
        max: samples[samples.len() - 1],
    }
}

fn percent(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss, reason = "a reported percentage")]
    {
        part as f64 * 100.0 / whole as f64
    }
}

/// One `sysctl` reading, or `unknown` when the host does not answer.
fn host_line(name: &str) -> String {
    Command::new("sysctl")
        .arg("-n")
        .arg(name)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map_or_else(|| "unknown".to_owned(), |text| text.trim().to_owned())
}

fn rustc_version() -> String {
    Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map_or_else(|| "unknown".to_owned(), |text| text.trim().to_owned())
}

/// Collects the rows one run reports and writes them where they can be retained.
#[derive(Default)]
struct Recorder {
    lines: Vec<String>,
}

impl Recorder {
    fn comment(&mut self, text: String) {
        let line = format!("# {text}");
        println!("{line}");
        self.lines.push(line);
    }

    fn timing(
        &mut self,
        section: &str,
        size: usize,
        population: u64,
        variant: &str,
        summary: &Summary,
    ) {
        // The trailing note column carries `-` rather than being left empty:
        // a row ending in a separator is trailing whitespace, which the
        // repository's own `git diff --check` refuses.
        self.row(&format!(
            "timing\t{section}\t{size}\t{population}\t{variant}\t{}\t{}\t{}\t{}\t{}\t-",
            summary.samples, summary.min, summary.p50, summary.p90, summary.max,
        ));
    }

    fn fact(&mut self, section: &str, size: usize, population: u64, variant: &str, note: &str) {
        self.row(&format!(
            "fact\t{section}\t{size}\t{population}\t{variant}\t-\t-\t-\t-\t-\t{note}",
        ));
    }

    fn row(&mut self, line: &str) {
        println!("{line}");
        self.lines.push(line.to_owned());
    }

    fn write(&self, path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("the results directory is creatable");
        }
        let mut body = String::new();
        for line in &self.lines {
            if line.starts_with('#') {
                writeln!(body, "{line}").expect("a string is writable");
            }
        }
        writeln!(
            body,
            "kind\tsection\tsize_bytes\tpopulation\tvariant\tsamples\tmin_ns\tp50_ns\tp90_ns\t\
             max_ns\tnote",
        )
        .expect("a string is writable");
        for line in &self.lines {
            if !line.starts_with('#') {
                writeln!(body, "{line}").expect("a string is writable");
            }
        }
        fs::write(path, body).expect("the results file is writable");
    }
}

/// A directory this run owns entirely, removed when the run finishes.
///
/// Unique by process identifier and nanosecond, so a sibling build or a second
/// copy of this harness shares no mutable path with it.
struct Scratch {
    path: PathBuf,
    keep: bool,
}

impl Scratch {
    fn new(keep: bool) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the host clock is after the Unix epoch")
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "tiler-cache-hot-path-efficiency-{}-{nonce}",
            std::process::id(),
        ));
        fs::create_dir_all(&path).expect("a scratch directory is creatable");
        Self { path, keep }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        if self.keep {
            println!("# scratch retained at {}", self.path.display());
            return;
        }
        let _ = fs::remove_dir_all(&self.path);
    }
}

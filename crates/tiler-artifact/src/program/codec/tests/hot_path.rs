//! Reproducible cost measurements. These assert nothing about time.

use super::super::super::tests::default_artifact;
use super::super::decode::decode;
use super::super::encode::{MANIFEST_DIGEST_DOMAIN, section_digest};
use super::super::model::position;
use super::super::payload::decode_metadata;
use super::support::{carried_artifact, encoded};
use std::time::Instant;
use tiler_digest::DigestAlgorithm;

// -------------------------------------------------------------------------
// Hot-path measurements
// -------------------------------------------------------------------------
//
// These print and assert nothing about time. A timing assertion fails on a
// loaded machine and passes on a fast one, which is a flake rather than a
// guard; what they are for is a reproducible number to read before and after a
// change. Reproduce with:
//
//   cargo nextest run --release -p tiler-artifact -E 'test(hot_path)' --no-capture
//
// Release matters: workspace crates build at `opt-level = 0` by default and the
// codec measures ~5x slower in dev.

/// Times one closure and reports the minimum and the mean of many runs.
///
/// **The minimum is the number to compare; the mean is printed beside it to
/// show how loaded the host was.** Every perturbation a machine applies — a
/// scheduler preemption, a frequency drop, a competing build — makes a run
/// *slower* and none makes it faster, so the distribution has a hard floor at
/// the true cost and an unbounded tail of noise. The minimum of enough runs
/// estimates that floor; the mean estimates the floor plus whatever else the
/// machine happened to be doing, which is why a mean-of-five once read a real
/// improvement in this codec as a regression.
fn min_and_mean(repeats: u32, mut run: impl FnMut()) -> (std::time::Duration, std::time::Duration) {
    // Warm the allocator and the branch predictors so the first timed run is
    // not measuring first-touch page faults.
    for _ in 0..16 {
        run();
    }
    let mut best = std::time::Duration::MAX;
    let total = Instant::now();
    for _ in 0..repeats {
        let start = Instant::now();
        run();
        best = best.min(start.elapsed());
    }
    (best, total.elapsed() / repeats)
}

/// Reports decode cost beside the cost of independently encoding its result.
///
/// A fresh encode derives identity and section digests before writing the whole
/// envelope. Decode's canonicity backstop instead reuses the identity and
/// section digests it already derived and compares the canonical runs without
/// accumulating another envelope. The ratio is therefore a reproducible cost
/// comparison, not the backstop's share of decode.
#[test]
fn hot_path_decode_and_fresh_encode_costs() {
    const REPEATS: u32 = 400;
    let bytes = encoded(&default_artifact());

    let (full, full_mean) = min_and_mean(REPEATS, || {
        let _ = decode(&bytes).expect("the fixture decodes");
    });

    let envelope = decode(&bytes).expect("the fixture decodes");
    let (fresh_encode, fresh_encode_mean) = min_and_mean(REPEATS, || {
        let _ = super::super::encode::encode(&envelope).expect("the envelope re-encodes");
    });

    println!("MEASURE envelope bytes   : {}", bytes.len());
    println!("MEASURE decode           : min {full:?}, mean {full_mean:?} over {REPEATS}");
    println!(
        "MEASURE fresh encode     : min {fresh_encode:?}, mean {fresh_encode_mean:?} over {REPEATS}"
    );
    println!(
        "MEASURE encode / decode  : {:.1}%",
        (fresh_encode.as_secs_f64() / full.as_secs_f64()) * 100.0
    );
}

/// Reports what one decode spends on selected stages and nested operations.
///
/// **The split is the finding, and it is not inferred from the source.**
/// `decode` parses the framing and manifest, re-proves the model's obligations,
/// derives the canonical identity to compare its digest against the manifest's,
/// and passes that same derivation to the canonicity backstop. The backstop
/// hashes the identity into the manifest's fixed-width declaration and hashes
/// the rebuilt manifest; it neither derives identity nor section digests a
/// second time, and it does not embed the identity preimage. The last stage is
/// the canonical arena order, printed because it is where the decode's worst
/// amplification used to sit: it derived a content-key table quadratic in arena
/// depth, and now walks the same arena through `compare_expr_nodes` without
/// materializing one. The stages are measured against the same decoded envelope
/// so the numbers can be compared with the full decode above them.
#[test]
fn hot_path_decode_stage_budget() {
    const REPEATS: u32 = 400;
    let bytes = encoded(&default_artifact());
    let envelope = decode(&bytes).expect("the fixture decodes");
    let identity = envelope.canonical_identity().expect("the envelope derives");
    let digests: Vec<_> = envelope
        .sections()
        .iter()
        .map(|section| section_digest(DigestAlgorithm::GOVERNED, section))
        .collect();

    let (full, _) = min_and_mean(REPEATS, || {
        let _ = decode(&bytes).expect("the fixture decodes");
    });
    let (validate, _) = min_and_mean(REPEATS, || {
        super::super::validate::validate(&envelope).expect("the envelope validates");
    });
    let (derive, _) = min_and_mean(REPEATS, || {
        let _ = envelope.canonical_identity().expect("the envelope derives");
    });
    let (backstop, _) = min_and_mean(REPEATS, || {
        assert!(
            super::super::encode::matches_canonical_encoding(
                &envelope, &identity, &digests, &bytes,
            )
            .expect("the canonical envelope compares")
        );
    });
    let (section_hash, _) = min_and_mean(REPEATS, || {
        for section in envelope.sections() {
            let _ = section_digest(DigestAlgorithm::GOVERNED, section);
        }
    });
    let (arena_order, _) = min_and_mean(REPEATS, || {
        let _ = super::super::model::canonical_expression_order(envelope.expressions());
    });

    let share = |part: std::time::Duration| (part.as_secs_f64() / full.as_secs_f64()) * 100.0;
    println!("MEASURE envelope bytes    : {}", bytes.len());
    println!("MEASURE decode total      : {full:?}");
    println!(
        "MEASURE   validate        : {validate:?} ({:.1}%)",
        share(validate)
    );
    println!(
        "MEASURE   derive identity : {derive:?} ({:.1}%)",
        share(derive)
    );
    println!(
        "MEASURE   canonicity      : {backstop:?} ({:.1}%)",
        share(backstop)
    );
    println!(
        "MEASURE   section digests : {section_hash:?} ({:.1}%, no longer paid twice)",
        share(section_hash)
    );
    println!(
        "MEASURE   arena order     : {arena_order:?} ({:.1}% of decode)",
        share(arena_order)
    );
}

/// Reports what a carried payload's repeated metadata decode costs.
///
/// `super::super::validate` decodes each carried payload's compilation subject once to
/// bind it to the descriptor's digest, and again per realized entry to resolve
/// that entry's backend mapping — `2 + E` decodes of the same bytes, each
/// re-allocating a `PayloadMetadata` including a copy of the carried source.
/// The repetition is real; this is what one of them is worth.
#[test]
fn hot_path_carried_metadata_decode() {
    const REPEATS: u32 = 400;
    let artifact = carried_artifact(b"kernel void fused() {}", b"\x00metallib\xff");
    let bytes = encoded(&artifact);
    let envelope = decode(&bytes).expect("a carried envelope decodes");
    let sections = envelope.payload_content()[0].expect("the payload is carried");
    let metadata = envelope.sections()[position(sections.metadata)]
        .bytes
        .clone();

    let (full, _) = min_and_mean(REPEATS, || {
        let _ = decode(&bytes).expect("a carried envelope decodes");
    });
    let (once, _) = min_and_mean(REPEATS, || {
        decode_metadata(&metadata).expect("the carried subject decodes");
    });
    println!("MEASURE carried envelope  : {} bytes", bytes.len());
    println!("MEASURE decode total      : {full:?}");
    println!(
        "MEASURE decode_metadata x1: {once:?} ({:.2}% of decode)",
        (once.as_secs_f64() / full.as_secs_f64()) * 100.0
    );
}

/// Reports governed digest throughput over one envelope-sized byte run.
///
/// This is a per-byte baseline, not a reconstruction of decode's hash traffic.
/// Decode hashes the received manifest and each section once, its derived
/// identity once for the declaration check, then the same identity and the
/// rebuilt manifest once in the canonicity backstop; section digests are reused.
/// [`hot_path_decode_stage_budget`] measures those stages in context.
#[test]
fn hot_path_digest_throughput() {
    const REPEATS: u32 = 400;
    let bytes = encoded(&default_artifact());
    let algorithm = DigestAlgorithm::GOVERNED;
    let (hash, _) = min_and_mean(REPEATS, || {
        let _ = algorithm.digest(MANIFEST_DIGEST_DOMAIN, &bytes);
    });
    println!("MEASURE envelope bytes    : {}", bytes.len());
    println!("MEASURE digest once over  : {hash:?}");
    println!(
        "MEASURE digest throughput : {:.1} MB/s",
        (f64::from(u32::try_from(bytes.len()).expect("the fixture fits u32")) / hash.as_secs_f64())
            / 1e6
    );
}

/// Decodes in a loop long enough for a sampling profiler to attribute the cost.
///
/// **This is the harness that says *where* decode time goes.** A single decode
/// is microseconds, which is one sample; reading a list of suspected costs off
/// the source instead optimizes the list it started with. Recording this loop
/// supplies frame-level attribution for whichever stage dominates at the
/// measured commit; [`hot_path_decode_stage_budget`] and
/// [`hot_path_digest_throughput`] isolate the current candidates.
///
/// It is `#[ignore]`d because it deliberately runs for seconds and asserts
/// nothing. Record it with:
///
/// ```text
/// CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release --tests -p tiler-artifact
/// TILER_PROFILE_SECONDS=20 samply record --save-only --unstable-presymbolicate \
///     --rate 4000 -o decode.profile.json.gz \
///     -- target/release/deps/tiler_artifact-<hash> \
///        --ignored --exact program::codec::tests::hot_path_decode_profile_loop --nocapture
/// ```
///
/// Three details are load-bearing. `CARGO_PROFILE_RELEASE_DEBUG=true` is
/// required: the release profile carries no debug information, and without it
/// every frame symbolicates to a bare hex address. `--unstable-presymbolicate`
/// writes the `*.syms.json` sidecar that holds the names — the profile's own
/// string table does not. And `--release` matters on its own: workspace crates
/// build at `opt-level = 0` by default and the codec measures ~5x slower in dev.
///
/// `TILER_PROFILE_SECONDS` sets the duration and defaults to ten.
#[test]
#[ignore = "runs for seconds under a profiler; not part of the gate"]
fn hot_path_decode_profile_loop() {
    let seconds: u64 = std::env::var("TILER_PROFILE_SECONDS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(10);
    let bytes = encoded(&default_artifact());
    let deadline = Instant::now() + std::time::Duration::from_secs(seconds);
    let mut decodes = 0_u64;
    while Instant::now() < deadline {
        for _ in 0..64 {
            let _ = decode(&bytes).expect("the fixture decodes");
        }
        decodes += 64;
    }
    println!("MEASURE profile loop: {decodes} decodes in {seconds}s");
}

/// Reports how large a canonical identity is relative to the bytes it names.
///
/// Artifact identity remains a canonical byte string rather than a digest. Its
/// ABI-expression portion writes the reached arena once in canonical order and
/// names every use by canonical position; the manifest carries only a digest of
/// those identity bytes. This measurement therefore no longer tracks a pending
/// flattening. It reports the current identity preimage size beside the envelope
/// size as a reproducible baseline for the governed identity budget and later
/// identity-grammar changes.
#[test]
fn hot_path_identity_size() {
    let artifact = default_artifact();
    let bytes = encoded(&artifact);
    println!("MEASURE envelope bytes   : {}", bytes.len());
    println!(
        "MEASURE artifact identity: {} bytes",
        artifact.canonical_identity().as_bytes().len()
    );
}

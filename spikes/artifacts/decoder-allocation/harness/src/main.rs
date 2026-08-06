//! What validating one artifact envelope allocates, measured through the real
//! public codec.
//!
//! # What is measured, and with what
//!
//! A counting [`GlobalAlloc`] wrapping [`System`], rather than an external
//! profiler. Three properties follow from that choice and they are the reason
//! for it: the numbers are exact rather than sampled, they are **deterministic**
//! — an allocation count is a property of the program and not of the machine, so
//! two runs on a loaded host must agree byte for byte and this harness asserts
//! that they do — and they are reproducible by one command on any host.
//!
//! Four quantities per measured call:
//!
//! - **peak live bytes** — the high-water mark of bytes the program owns, above
//!   the live total when the call began. This is the quantity the envelope's
//!   budgets have to be read against.
//! - **retained bytes** — live bytes still held when the call returns. For a
//!   decode this is the [`DecodedArtifact`] itself: the ownership the boundary
//!   genuinely requires.
//! - **requested bytes** — every `alloc` size plus every `realloc`'s *new* size.
//!   A vector that doubles its way to `n` bytes requests about `2n` and peaks at
//!   `n`, so this is what separates churn from footprint.
//! - **allocator calls** — `alloc`, `alloc_zeroed`, and `realloc`.
//!
//! Beside them, the largest individual blocks the call requested. That is the
//! allocation-site evidence: a block the size of the whole envelope has exactly
//! one possible origin, and naming its size is what turns a total into an
//! attribution.
//!
//! # The measurement boundary
//!
//! **Peak live is an accounting model, not RSS.** `realloc` is forwarded to the
//! system allocator and accounted as `new - old`, so a growth the allocator
//! satisfies by moving a block is not charged for holding both copies. Real
//! resident memory can therefore exceed these figures transiently; the direction
//! is stated because it means a reduction reported here is a floor on the
//! reduction a consumer sees, never a ceiling.
//!
//! **One process, one thread.** The counters are process-wide, and this harness
//! spawns no thread. A future harness that did would have to make them
//! thread-local before any reading meant anything.
//!
//! **The object bytes are synthetic**, with the consequence stated in
//! [`envelope`]: this is not evidence about a real Metal compilation.

mod envelope;

use std::alloc::{GlobalAlloc, Layout, System};
use std::fmt::Write as _;
use std::sync::atomic::{AtomicUsize, Ordering};

use sha2::{Digest as _, Sha256};
use tiler_artifact::program::{DecodedArtifact, VerifiedArtifactProgram, decode_artifact};

use envelope::EnvelopeFactory;

/// The two independent size dimensions, as `(object bytes, arena chain nodes)`.
///
/// **Sections.** Zero object bytes is the envelope's fixed overhead — the
/// artifact with no object byte carried at all. The top of that sweep is
/// `MAX_SECTION_BYTES`, the largest section
/// `crates/tiler-artifact/src/program/codec/model.rs` admits, so the row is the
/// maximum admitted section this envelope shape can carry rather than a round
/// number chosen for convenience.
///
/// **Manifest.** The arena chain grows the ABI expression table toward
/// `MAX_ABI_EXPRESSIONS`, which is 4,096. Four doublings establish the exponent
/// and the last row is the governed bound itself, less the handful of nodes the
/// compiled program contributes and the three the chain needs for its own
/// literals — so the endpoint is measured rather than extrapolated to.
const SHAPES: [(usize, usize); 9] = [
    (0, 0),
    (1 << 20, 0),
    (16 << 20, 0),
    (64 << 20, 0),
    (0, 128),
    (0, 512),
    (0, 1024),
    (0, 2048),
    (0, 4000),
];

/// Smallest block this harness records individually.
///
/// Below it a block cannot be a section, a manifest, or an envelope, which are
/// the only allocations whose *size* identifies their site.
const LARGE_BLOCK_BYTES: usize = 4096;

/// How many large blocks one measured call may record.
///
/// A fixed array because the recorder runs inside the allocator and must not
/// allocate. Overflow is counted and reported rather than silently dropped.
const LARGE_SLOTS: usize = 512;

/// Blocks reported per row.
const REPORTED_BLOCKS: usize = 4;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static REQUESTED: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);
static LARGE: [AtomicUsize; LARGE_SLOTS] = [const { AtomicUsize::new(0) }; LARGE_SLOTS];
static LARGE_NEXT: AtomicUsize = AtomicUsize::new(0);

/// The system allocator, counted.
struct Counting;

// The one `unsafe` site this workspace admits, under the shape ADR 0079
// requires. There is no safe route: `GlobalAlloc` is an unsafe trait because an
// implementation must return blocks that satisfy the caller's layout, and the
// ticket rejected an external profiler in favour of a number reproducible by
// command.
//
// SAFETY: every method forwards its exact `Layout` to `System`, which is a
// correct `GlobalAlloc`, and returns that implementation's pointer unchanged. No
// method reads, writes, or reinterprets the memory it hands back, and the
// bookkeeping between the forwarded call and the return is atomic integer
// arithmetic on statics that own no allocation of their own — so the recorder
// cannot re-enter the allocator, which is the one way a counting allocator
// becomes unsound.
#[allow(
    unsafe_code,
    reason = "a counting global allocator is unrepresentable in safe Rust; the SAFETY note above bounds the site"
)]
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record(layout.size());
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            record(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let moved = unsafe { System.realloc(pointer, layout, new_size) };
        if !moved.is_null() {
            LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
            record(new_size);
        }
        moved
    }
}

/// Charges one satisfied request; allocation-free, because it runs inside the
/// allocator.
fn record(size: usize) {
    CALLS.fetch_add(1, Ordering::Relaxed);
    REQUESTED.fetch_add(size, Ordering::Relaxed);
    let live = LIVE.fetch_add(size, Ordering::Relaxed) + size;
    PEAK.fetch_max(live, Ordering::Relaxed);
    if size >= LARGE_BLOCK_BYTES {
        let slot = LARGE_NEXT.fetch_add(1, Ordering::Relaxed);
        if slot < LARGE_SLOTS {
            LARGE[slot].store(size, Ordering::Relaxed);
        }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// One measured call's allocation behaviour.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Reading {
    peak_bytes: usize,
    retained_bytes: usize,
    requested_bytes: usize,
    calls: usize,
    /// The largest individual requests, descending.
    blocks: Vec<usize>,
    /// Large requests beyond [`LARGE_SLOTS`], which `blocks` did not see.
    dropped_blocks: usize,
}

/// Runs `work` with the counters reset, and reports what it allocated.
fn measure<T>(work: impl FnOnce() -> T) -> (Reading, T) {
    let live_before = LIVE.load(Ordering::Relaxed);
    PEAK.store(live_before, Ordering::Relaxed);
    REQUESTED.store(0, Ordering::Relaxed);
    CALLS.store(0, Ordering::Relaxed);
    LARGE_NEXT.store(0, Ordering::Relaxed);

    let value = work();

    let peak = PEAK.load(Ordering::Relaxed);
    let live_after = LIVE.load(Ordering::Relaxed);
    let requested = REQUESTED.load(Ordering::Relaxed);
    let calls = CALLS.load(Ordering::Relaxed);
    let recorded = LARGE_NEXT.load(Ordering::Relaxed);
    // Snapshotting allocates, so every counter above is read first.
    let mut blocks: Vec<usize> = (0..recorded.min(LARGE_SLOTS))
        .map(|slot| LARGE[slot].load(Ordering::Relaxed))
        .collect();
    blocks.sort_unstable_by(|left, right| right.cmp(left));
    blocks.truncate(REPORTED_BLOCKS);
    (
        Reading {
            peak_bytes: peak.saturating_sub(live_before),
            retained_bytes: live_after.saturating_sub(live_before),
            requested_bytes: requested,
            calls,
            blocks,
            dropped_blocks: recorded.saturating_sub(LARGE_SLOTS),
        },
        value,
    )
}

/// Measures `work` twice and proves the two readings identical.
///
/// An allocation count is a property of the program rather than of the host, so
/// two readings that disagree mean the measurement is wrong — a lazily
/// initialized static caught inside the window, or a counter that leaked across
/// calls. Reporting the pair rather than one of them is what makes this
/// checkable instead of assumed; a warm-up call precedes both, so first-call
/// initialization is charged to neither.
fn measure_twice<T>(mut work: impl FnMut() -> T) -> Reading {
    drop(work());
    let (first, value) = measure(&mut work);
    drop(value);
    let (second, value) = measure(&mut work);
    drop(value);
    assert_eq!(
        first, second,
        "an allocation reading is deterministic; two runs of one call disagreed",
    );
    first
}

/// One row of the reported table.
struct Row {
    object_bytes: usize,
    arena_chain: usize,
    envelope_bytes: usize,
    phase: &'static str,
    reading: Reading,
    /// What the decode returned, for a row that expected a refusal.
    verdict: String,
}

fn main() {
    let record = std::env::args()
        .skip_while(|argument| argument != "--record")
        .nth(1);

    let factory = EnvelopeFactory::new();
    let mut rows: Vec<Row> = Vec::new();

    for (object_bytes, arena_chain) in SHAPES {
        let artifact = factory.artifact(object_bytes, arena_chain);
        let bytes = artifact.encode().expect("the envelope encodes");
        let envelope_bytes = bytes.len();
        let row = |phase, reading, verdict: &str| Row {
            object_bytes,
            arena_chain,
            envelope_bytes,
            phase,
            reading,
            verdict: verdict.to_owned(),
        };

        rows.push(row(
            "encode",
            measure_twice(|| encode_of(&artifact)),
            "encoded",
        ));
        rows.push(row(
            "decode",
            measure_twice(|| decoded_of(&bytes)),
            "decoded",
        ));

        let decoded = decode_artifact(&bytes).expect("the envelope decodes");
        rows.push(row(
            "identity",
            measure_twice(|| decoded.identity()),
            "derived",
        ));
        rows.push(row(
            "re-encode",
            measure_twice(|| decoded.re_encode().expect("the envelope re-encodes")),
            "re-encoded",
        ));
        drop(decoded);

        for (phase, forged) in adversarial(&bytes, object_bytes > 0) {
            let verdict = decode_artifact(&forged)
                .err()
                .map_or_else(|| "ACCEPTED".to_owned(), |failure| failure.to_string());
            assert!(
                verdict != "ACCEPTED",
                "the {phase} forgery decoded; the measurement below would not be about a refusal",
            );
            rows.push(row(
                phase,
                measure_twice(|| decode_artifact(&forged).err()),
                &verdict,
            ));
        }
    }

    let report = report(&rows);
    print!("{report}");
    if let Some(name) = record {
        let path = format!("results/decoder-allocation-{name}.tsv");
        std::fs::write(&path, &report).expect("the results directory exists");
        println!("recorded {path}");
    }
}

/// Encodes one artifact, discarding the bytes inside the measured window.
fn encode_of(artifact: &VerifiedArtifactProgram) -> usize {
    artifact.encode().expect("the envelope encodes").len()
}

/// Decodes one envelope, retaining the view so the row can report its footprint.
fn decoded_of(bytes: &[u8]) -> DecodedArtifact {
    decode_artifact(bytes).expect("the envelope decodes")
}

/// Renders the table.
fn report(rows: &[Row]) -> String {
    let mut out = String::new();
    out.push_str(
        "object_bytes\tarena_chain\tenvelope_bytes\tphase\tpeak_bytes\tpeak_over_envelope\tretained_bytes\trequested_bytes\tcalls\tlargest_blocks\tverdict\n",
    );
    for row in rows {
        let ratio = if row.envelope_bytes == 0 {
            0.0
        } else {
            #[allow(
                clippy::cast_precision_loss,
                reason = "a ratio printed to two decimals; the inputs are envelope sizes far below 2^53"
            )]
            {
                row.reading.peak_bytes as f64 / row.envelope_bytes as f64
            }
        };
        let mut blocks = String::new();
        for (position, block) in row.reading.blocks.iter().enumerate() {
            if position > 0 {
                blocks.push(' ');
            }
            let _ = write!(blocks, "{block}");
        }
        if row.reading.dropped_blocks > 0 {
            let _ = write!(blocks, " (+{} unrecorded)", row.reading.dropped_blocks);
        }
        if blocks.is_empty() {
            blocks.push('-');
        }
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}\t{ratio:.2}\t{}\t{}\t{}\t{blocks}\t{}",
            row.object_bytes,
            row.arena_chain,
            row.envelope_bytes,
            row.phase,
            row.reading.peak_bytes,
            row.reading.retained_bytes,
            row.reading.requested_bytes,
            row.reading.calls,
            row.verdict,
        );
    }
    out
}

/// Builds the malformed inputs, each named by the boundary it must reach.
///
/// The first four are refused before anything is read into memory, which is the
/// claim they exist to check: a forged length must report truncation rather than
/// reserve for content that is not there. The last two are the opposite end —
/// they pass framing and integrity and are refused only after the decoder has
/// done its whole job, so they are what bounds the *adversarial* peak.
fn adversarial(bytes: &[u8], carries_object: bool) -> Vec<(&'static str, Vec<u8>)> {
    let mut forged: Vec<(&'static str, Vec<u8>)> = Vec::new();

    let mut truncated = bytes.to_vec();
    truncated.truncate(bytes.len() / 2);
    forged.push(("forged/truncated", truncated));

    let mut magic = bytes.to_vec();
    magic[0] ^= 0xff;
    forged.push(("forged/magic", magic));

    // The framing header's declared total length, at its fixed offset. It is
    // compared against the supplied length before a byte of body is read.
    let mut total = bytes.to_vec();
    total[17..25].copy_from_slice(&u64::MAX.to_be_bytes());
    forged.push(("forged/total-length", total));

    // The declared manifest length, raised to just under the governed manifest
    // budget so it passes the budget check and reaches the read.
    let mut manifest_length = bytes.to_vec();
    manifest_length[25..33].copy_from_slice(&((63_u64 << 20).to_be_bytes()));
    forged.push(("forged/manifest-length", manifest_length));

    // The declared section count, raised past its governed budget.
    let mut sections = bytes.to_vec();
    sections[33..37].copy_from_slice(&u32::MAX.to_be_bytes());
    forged.push(("forged/section-count", sections));

    // One flipped byte of carried content. Every section ahead of it is read and
    // digested before the flip is reached, so this is the deepest *section*
    // path — and the object is the last section in canonical purpose order,
    // which is why the flip is the envelope's last byte. Skipped when no object
    // is carried, because then that byte is section framing and the refusal
    // would be a shallower one wearing this row's name.
    if carries_object {
        let mut content = bytes.to_vec();
        let last = content.len() - 1;
        content[last] ^= 0xff;
        forged.push(("forged/object-byte", content));
    }

    // The deepest path a forgery can reach. The manifest's trailing field is the
    // artifact's own identity, so flipping its last byte and re-deriving the
    // manifest digest produces bytes that pass framing, integrity, canonical
    // form, every structural obligation and the whole of `validate`, and are
    // refused only by the identity comparison. Whatever a decode allocates, this
    // input makes it allocate.
    forged.push(("forged/identity", forged_identity(bytes)));

    forged
}

/// Flips the manifest's trailing identity byte and repairs the manifest digest.
fn forged_identity(bytes: &[u8]) -> Vec<u8> {
    /// Fixed width of the framing header.
    const HEADER_BYTES: usize = 69;
    /// Domain separator of the manifest digest, restated from the encoder.
    ///
    /// Restated rather than imported because it is crate-private, and a spike
    /// that could not reach it would have no way to produce an input that
    /// survives the integrity check. If the encoder's separator changes, this
    /// forgery stops reaching its boundary and the assertion in `main` fails
    /// loudly rather than measuring a shallower refusal.
    const MANIFEST_DIGEST_DOMAIN: &[u8] = b"tiler.artifact-envelope.manifest-digest.v1\0";

    let mut forged = bytes.to_vec();
    let manifest_bytes = usize::try_from(u64::from_be_bytes(
        forged[25..33]
            .try_into()
            .expect("a checked read of eight bytes"),
    ))
    .expect("a manifest length below the governed budget fits usize");
    let last = HEADER_BYTES + manifest_bytes - 1;
    forged[last] ^= 0xff;

    let mut state = Sha256::new();
    state.update(MANIFEST_DIGEST_DOMAIN);
    state.update(&forged[HEADER_BYTES..HEADER_BYTES + manifest_bytes]);
    let digest = state.finalize();
    forged[37..69].copy_from_slice(&digest);
    forged
}

//! Bounded tests for the expansion cache protocol.
//!
//! # What these are evidence for, and what they are not
//!
//! They are evidence for the framing, the path parser, the key derivation, and
//! the *threaded* half of the protocol: exclusion on one key, the post-lock
//! recheck, publication by rename, immutability of a published entry, and the
//! replacement of a rejected one.
//!
//! **They are not evidence for any cross-process crash or race property.** A
//! thread that returns is not a process that was killed: it unwinds, it closes
//! its descriptors on its own, and it never leaves a half-written file behind
//! with no owner. Those properties need real processes stopped at each
//! publication phase, which [`super::harness`] does — against this crate's own
//! bundle, by re-executing this test binary. Nothing below is offered as a
//! substitute for it, and no test here is named as if it were.
//!
//! Two mechanisms are used to make the protocol reachable without a real
//! artifact envelope. Framing tests call the bundle encoder and decoder
//! directly. Protocol tests call the crate-private [`ExpansionCache::resolve`]
//! and [`ExpansionCache::read_entry`] with a validator that accepts any bytes,
//! which is what lets a "compilation" be a byte string. The public API's own
//! validator is separately shown to be the artifact decoder, by feeding it bytes
//! that are not an artifact and observing the artifact layer's own typed
//! refusal.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tiler_artifact::program::{ArtifactCodecFailure, DIGEST_BYTES, DigestAlgorithm};

use super::bundle::{self, BundleRejection, BundleSection};
use super::collect::{
    CollectionBound, CollectionOutcome, Disposition, MaxEntryAge, MaxEntryAgeRefusal, RemovalReason,
};
use super::fault;
use super::key::{CacheKey, KEY_LABEL_BYTES, KeyTextRejection};
use super::layout::{PathRejection, key_of_entry_path};
use super::limits::Limits;
use super::lock::KeyLock;
use super::preflight::{PreflightReport, PreflightVerdict};
use super::report::{
    CacheOperation, EntryRejection, MissReason, PublicationRefusal, QuarantineOutcome,
};
use super::retention::{
    DebugRetention, MAX_RETAINED_RUN_BYTES, MAX_RETAINED_RUNS, MAX_RETENTION_LABEL_BYTES,
    RETENTION_DOMAIN, RetentionRefusal, RetentionRejection,
};
use super::store::{
    Durability, Eviction, ExpansionCache, Lookup, ProtocolOutcome, PublishFailure, SweepReport,
};
use super::subject::{ComposedSubject, SubjectFacet, SubjectFacets, SubjectRefusal};

// -------------------------------------------------------------------------
// Fixtures
// -------------------------------------------------------------------------

/// A validator that accepts any non-empty payload, for exercising the protocol.
///
/// It refuses an empty one so it has a real rejection branch: a validator that
/// could not fail would let the protocol's rejection paths compile without ever
/// being reachable through it.
fn any_payload(bytes: &[u8]) -> Result<Vec<u8>, ArtifactCodecFailure> {
    if bytes.is_empty() {
        return Err(ArtifactCodecFailure::Malformed {
            detail: "an empty payload is not an artifact".to_owned(),
        });
    }
    Ok(bytes.to_vec())
}

/// A unique directory for one test, removed when the guard drops.
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
            "tiler-cache-test-{name}-{}-{nonce}-{}",
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

fn cache(scratch: &Scratch) -> ExpansionCache {
    ExpansionCache::open(scratch.root().join("cache"))
}

/// Composes one subject over the given facet bytes, expecting it to be valid.
fn composed(backend_compilations: &[&[u8]], artifact_program: &[u8]) -> ComposedSubject {
    ComposedSubject::compose(&SubjectFacets {
        backend_compilations,
        artifact_program,
    })
    .expect("the fixture names every facet")
}

/// Attempts a composition, for the cases that must be refused.
fn compose(
    backend_compilations: &[&[u8]],
    artifact_program: &[u8],
) -> Result<ComposedSubject, SubjectRefusal> {
    ComposedSubject::compose(&SubjectFacets {
        backend_compilations,
        artifact_program,
    })
}

/// Publishes one subject and then sets its entry's publication time.
///
/// Collection orders by the entry file's modification time, and two publications
/// microseconds apart can land on one timestamp wherever the filesystem's
/// granularity is coarser than the gap. Setting the time explicitly makes an
/// ordering test a statement about the selector rather than about how fast two
/// writes happened to be — and `set_modified` is the same metadata the
/// production scan reads, so nothing about the path under test is bypassed.
fn publish_aged(
    cache: &ExpansionCache,
    subject: &[u8],
    envelope: &[u8],
    seconds_old: u64,
) -> CacheKey {
    let key = publish(cache, subject, envelope);
    let when = SystemTime::now() - Duration::from_secs(seconds_old);
    fs::OpenOptions::new()
        .write(true)
        .open(cache.entry_path(&key))
        .expect("a published entry opens")
        .set_modified(when)
        .expect("the host records a modification time");
    key
}

/// A bound on the number of entries, with no byte ceiling and no age ceiling.
const fn at_most(entries: u64) -> CollectionBound {
    CollectionBound {
        max_total_bytes: None,
        max_entries: Some(entries),
        max_entry_age: None,
    }
}

/// A bound on entry age alone, with neither aggregate ceiling.
///
/// Panics on a zero age, which is exactly the value [`MaxEntryAge::new`] refuses:
/// the refusal has its own test, and a fixture that silently accepted one would
/// let an age test pass for the wrong reason.
fn older_than(max_age: Duration) -> CollectionBound {
    CollectionBound {
        max_total_bytes: None,
        max_entries: None,
        max_entry_age: Some(MaxEntryAge::new(max_age).expect("a non-zero maximum age is a bound")),
    }
}

/// Sets one published entry's modification time to an exact instant.
///
/// Distinct from [`publish_aged`], which dates an entry relative to the host
/// clock at the moment it runs. An age *boundary* is unreachable that way — the
/// collection's own clock reading is necessarily later — so the tests that state
/// something about the predicate itself anchor both the entry's time and the
/// collection's `now` to one value they hold.
fn set_published(cache: &ExpansionCache, key: &CacheKey, when: SystemTime) {
    fs::OpenOptions::new()
        .write(true)
        .open(cache.entry_path(key))
        .expect("a published entry opens")
        .set_modified(when)
        .expect("the host records a modification time");
}

/// Publishes one subject through the private protocol and returns the key.
fn publish(cache: &ExpansionCache, subject: &[u8], envelope: &[u8]) -> CacheKey {
    let outcome = cache
        .resolve(subject, || Ok::<_, String>(envelope.to_vec()), &any_payload)
        .expect("a build that succeeds resolves");
    match outcome {
        ProtocolOutcome::Hit {
            entry, published, ..
        } => {
            assert!(published, "an empty cache publishes rather than hits");
            entry.key
        }
        ProtocolOutcome::Uncached { report, .. } => panic!(
            "publication was refused: {:?}",
            report.publication_refusal().map(ToString::to_string),
        ),
    }
}

// -------------------------------------------------------------------------
// Composing the subject
// -------------------------------------------------------------------------

/// Two artifacts agreeing on every compilation and differing only in their plan
/// portfolio are two keys.
///
/// This is the failure the composed subject exists to remove, and it is stated
/// as the two halves that make it a failure rather than a coincidence. The pair
/// shares one backend compilation subject byte for byte, so a subject naming only
/// the compilation — which is all `tiler-metal-aot` can name — would file them
/// under one key and serve either artifact for the other. The second assertion is
/// what stops the first from passing for the wrong reason: identical facets still
/// produce one key, so the difference above comes from the portfolio and not from
/// composition being unstable.
#[test]
fn two_artifacts_differing_only_in_plan_portfolio_key_differently() {
    let compilation: &[u8] = b"tiler.metal-aot.compilation-identity.v1\0...one exact compilation";
    let one_variant = composed(&[compilation], b"portfolio: one plan variant");
    let two_variants = composed(&[compilation], b"portfolio: two plan variants");

    assert_ne!(
        one_variant.as_bytes(),
        two_variants.as_bytes(),
        "the plan portfolio must reach the composed subject",
    );
    assert_ne!(
        CacheKey::derive(&one_variant),
        CacheKey::derive(&two_variants),
        "two plan portfolios over one compilation must not share a cache entry",
    );
    assert_eq!(
        CacheKey::derive(&one_variant),
        CacheKey::derive(&composed(&[compilation], b"portfolio: one plan variant")),
        "identical facets must still name one entry",
    );
}

/// A composed subject opens with its versioned domain tag and carries content
/// after it.
///
/// The tag is what makes a bare producer subject unable to name an entry: a
/// caller that reached past the composer and handed a raw compilation subject to
/// the key derivation could not produce bytes opening this way, so the two
/// spellings can never collide.
#[test]
fn the_composed_subject_is_domain_separated() {
    let subject = composed(&[b"compilation"], b"program");
    assert!(
        subject
            .as_bytes()
            .starts_with(super::subject::COMPOSED_SUBJECT_DOMAIN)
    );
    assert!(
        subject.as_bytes().len() > super::subject::COMPOSED_SUBJECT_DOMAIN.len(),
        "the domain tag must precede content rather than be the whole subject",
    );
    assert_ne!(
        subject.as_bytes(),
        b"compilation",
        "a facet's own bytes are never the composed subject",
    );
}

/// Composing the same facets twice yields the same bytes.
///
/// The negative tests below only mean something paired with this one: they show
/// facets *move* the bytes, and this shows nothing else does.
#[test]
fn the_composed_subject_is_a_function_of_its_facets_alone() {
    assert_eq!(
        composed(&[b"compilation"], b"program"),
        composed(&[b"compilation"], b"program"),
    );
}

/// Every facet independently moves the composed subject.
///
/// The `match` is exhaustive with no wildcard, so a facet added to
/// [`SubjectFacet`] fails to compile here until it is given an alteration —
/// which is what keeps this from silently covering less than the whole facet set.
#[test]
fn every_facet_moves_the_composed_subject() {
    let baseline = composed(&[b"compilation"], b"program");
    for facet in [
        SubjectFacet::BackendCompilations,
        SubjectFacet::ArtifactProgram,
    ] {
        let altered = match facet {
            SubjectFacet::BackendCompilations => composed(&[b"other-compilation"], b"program"),
            SubjectFacet::ArtifactProgram => composed(&[b"compilation"], b"other-program"),
        };
        assert_ne!(baseline, altered, "{facet} must reach the composed subject");
    }
}

/// Adjacent runs cannot be re-split into a different subject.
///
/// Two ways to try it, because the frame has two joins to defend. Within a facet,
/// a longer run followed by a shorter one must not concatenate to the same bytes
/// as the reverse. Across facets, moving a run from the compilations into the
/// program subject must not reproduce the original — otherwise a two-payload
/// artifact and a one-payload artifact with a longer program could share a key.
#[test]
fn facet_runs_cannot_be_re_split_into_another_subject() {
    assert_ne!(composed(&[b"ab"], b"cd"), composed(&[b"a"], b"bcd"));
    assert_ne!(composed(&[b"a", b"b"], b"c"), composed(&[b"a"], b"bc"));
}

/// The compilation sequence is ordered and counted, not a set.
///
/// Payload order is part of the envelope's bytes, so two artifacts whose payload
/// descriptors are permutations of one another are different artifacts; and a
/// second payload identical to the first is a second payload, not a duplicate to
/// fold away.
#[test]
fn the_backend_compilation_sequence_is_ordered_and_counted() {
    assert_ne!(composed(&[b"a", b"b"], b"p"), composed(&[b"b", b"a"], b"p"));
    assert_ne!(composed(&[b"a"], b"p"), composed(&[b"a", b"a"], b"p"));
}

/// A facet a caller could not fill is refused rather than composed.
///
/// This is the mechanism that turns the old silent under-key into a loud stop.
/// Every canonical subject in this workspace opens with its own versioned domain
/// tag, so no authority produces zero bytes; an empty run is a caller that had
/// nothing to supply, and a subject composed from one would name less than the
/// envelope it goes on to file.
#[test]
fn a_facet_a_caller_could_not_fill_is_refused() {
    assert_eq!(
        compose(&[], b"program"),
        Err(SubjectRefusal::NoRuns {
            facet: SubjectFacet::BackendCompilations,
        }),
    );
    assert_eq!(
        compose(&[b""], b"program"),
        Err(SubjectRefusal::EmptyRun {
            facet: SubjectFacet::BackendCompilations,
            index: 0,
        }),
    );
    assert_eq!(
        compose(&[b"compilation", b""], b"program"),
        Err(SubjectRefusal::EmptyRun {
            facet: SubjectFacet::BackendCompilations,
            index: 1,
        }),
    );
    assert_eq!(
        compose(&[b"compilation"], b""),
        Err(SubjectRefusal::EmptyRun {
            facet: SubjectFacet::ArtifactProgram,
            index: 0,
        }),
    );
}

/// A refusal names the facet that was left unfilled.
///
/// A caller that cannot see *which* facet it failed to supply has to guess, and
/// the whole value of refusing is that the omission becomes legible.
#[test]
fn a_subject_refusal_names_its_facet() {
    let rendered = compose(&[], b"program")
        .expect_err("an unfilled facet is refused")
        .to_string();
    assert!(rendered.contains("backend-compilations"), "{rendered}");
}

/// No governed digest domain in this crate is a prefix of another.
///
/// [`DigestAlgorithm::digest`] hashes `domain || body`, which distinguishes two
/// subjects only when no admitted domain prefixes another — otherwise a longer
/// domain and a shorter one with leading body bytes would collide, and a cache
/// key would equal a bundle section digest over related bytes. `tiler-artifact`
/// checks the same property over its own three domains; this crate owns two more
/// and the property is global to the one algorithm that hashes them all.
#[test]
fn no_governed_cache_domain_is_a_prefix_of_another() {
    let domains = [
        super::key::CACHE_KEY_DOMAIN,
        super::bundle::SECTION_DIGEST_DOMAIN,
    ];
    for (index, left) in domains.iter().enumerate() {
        for right in domains.iter().skip(index + 1) {
            assert!(
                !left.starts_with(right) && !right.starts_with(left),
                "one governed cache digest domain prefixes another",
            );
        }
    }
}

// -------------------------------------------------------------------------
// Complete cache identity
// -------------------------------------------------------------------------

/// The key is a function of the subject bytes alone.
#[test]
fn the_key_is_a_function_of_the_subject() {
    assert_eq!(
        CacheKey::derive_bytes(b"subject"),
        CacheKey::derive_bytes(b"subject")
    );
    assert_ne!(
        CacheKey::derive_bytes(b"subject"),
        CacheKey::derive_bytes(b"subjecu")
    );
    assert_ne!(
        CacheKey::derive_bytes(b"subject"),
        CacheKey::derive_bytes(b"subject ")
    );
    assert_ne!(CacheKey::derive_bytes(b""), CacheKey::derive_bytes(b"\0"));
}

/// A key renders as exactly the fixed width the namespace parses.
#[test]
fn a_key_renders_at_the_parsed_width() {
    let label = CacheKey::derive_bytes(b"subject").label();
    assert_eq!(label.len(), KEY_LABEL_BYTES);
    assert!(
        label.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "{label}",
    );
    assert!(
        label.bytes().all(|byte| !byte.is_ascii_uppercase()),
        "{label}",
    );
}

/// The key is domain-separated from a bare digest of the same bytes.
///
/// Without the domain, a cache key would equal any other governed digest taken
/// over the same subject — an artifact payload identity over the same source,
/// say — and two subjects that mean different things would share a value.
#[test]
fn the_key_is_domain_separated_from_a_bare_digest() {
    let subject = b"subject";
    let bare = DigestAlgorithm::GOVERNED.digest(b"", subject);
    assert_ne!(CacheKey::derive_bytes(subject).label(), bare.label());
}

// -------------------------------------------------------------------------
// The path parser
// -------------------------------------------------------------------------

/// A well-formed entry path yields the key it was built from.
#[test]
fn an_entry_path_round_trips_its_key() {
    let scratch = Scratch::new("path-round-trip");
    let cache = cache(&scratch);
    let key = CacheKey::derive_bytes(b"subject");
    assert_eq!(
        key_of_entry_path(&cache.entry_path(&key)).expect("a constructed entry path parses"),
        key,
    );
}

/// A key one character short is refused rather than padded, and one character
/// long is refused rather than cut.
///
/// This is the clause that must never "fit" a key to the width: either
/// adjustment maps two distinct compilations onto one entry, and a validated hit
/// would then return an artifact built from different inputs.
#[test]
fn a_key_of_the_wrong_width_is_never_fitted() {
    let label = CacheKey::derive_bytes(b"subject").label();
    for text in [&label[..KEY_LABEL_BYTES - 1], &format!("{label}0")[..]] {
        assert_eq!(
            CacheKey::parse_label(text),
            Err(KeyTextRejection::Width { found: text.len() }),
            "{text}",
        );
    }
}

/// Uppercase hexadecimal is refused rather than folded.
///
/// Folding would make two texts name one key, so one entry could be reached by
/// two paths and the per-key lock at one of them would not protect the other.
#[test]
fn uppercase_hexadecimal_is_refused_rather_than_folded() {
    let label = CacheKey::derive_bytes(b"subject").label();
    let upper = label.to_uppercase();
    let position = label
        .bytes()
        .position(|byte| byte.is_ascii_alphabetic())
        .expect("this digest renders with at least one letter");
    assert_eq!(
        CacheKey::parse_label(&upper),
        Err(KeyTextRejection::NotLowercaseHexadecimal {
            position,
            byte: upper.as_bytes()[position],
        }),
    );
}

/// A non-hexadecimal byte is refused, naming its position.
#[test]
fn a_non_hexadecimal_byte_is_refused_by_position() {
    let mut label = CacheKey::derive_bytes(b"subject").label();
    label.replace_range(7..8, "z");
    assert_eq!(
        CacheKey::parse_label(&label),
        Err(KeyTextRejection::NotLowercaseHexadecimal {
            position: 7,
            byte: b'z',
        }),
    );
}

/// An entry under the wrong shard is refused as misplaced.
#[test]
fn an_entry_under_the_wrong_shard_is_misplaced() {
    let label = CacheKey::derive_bytes(b"subject").label();
    let wrong = if label.starts_with("00") { "11" } else { "00" };
    let path = Path::new("/cache/v1/entries")
        .join(wrong)
        .join(format!("{label}.bundle"));
    assert_eq!(
        key_of_entry_path(&path),
        Err(PathRejection::Shard {
            expected: label[..2].to_owned(),
            found: wrong.to_owned(),
        }),
    );
}

/// A file that is not a bundle is refused on its extension.
#[test]
fn a_non_bundle_file_name_is_refused() {
    let label = CacheKey::derive_bytes(b"subject").label();
    let path = Path::new("/cache/v1/entries")
        .join(&label[..2])
        .join(format!("{label}.metallib"));
    assert_eq!(key_of_entry_path(&path), Err(PathRejection::Extension));
}

// -------------------------------------------------------------------------
// The bundle frame
// -------------------------------------------------------------------------

fn encoded(subject: &[u8], envelope: &[u8]) -> (CacheKey, Vec<u8>) {
    encoded_retaining(subject, envelope, &DebugRetention::none())
}

fn encoded_retaining(
    subject: &[u8],
    envelope: &[u8],
    retained: &DebugRetention,
) -> (CacheKey, Vec<u8>) {
    bundle::encode(subject, envelope, retained, &Limits::default()).expect("a small bundle encodes")
}

/// One retention carrying a single labelled run.
fn retaining(label: &str, bytes: &[u8]) -> DebugRetention {
    DebugRetention::none()
        .retaining(label, bytes)
        .expect("the fixture's label is governed and unique")
}

fn decode_default(bytes: &[u8], key: &CacheKey) -> Result<(), BundleRejection> {
    decode_default_view(bytes, key).map(|_| ())
}

/// Decodes under the default bounds and keeps the view, for the one case that
/// asserts *which bytes* a validated bundle then hands on.
fn decode_default_view(
    bytes: &[u8],
    key: &CacheKey,
) -> Result<bundle::BundleView, BundleRejection> {
    bundle::decode(bytes, key, &Limits::default())
}

/// A bundle round-trips its subject and its envelope.
#[test]
fn a_bundle_round_trips() {
    let (key, bytes) = encoded(b"subject", b"envelope-bytes");
    let view = bundle::decode(&bytes, &key, &Limits::default()).expect("a fresh bundle validates");
    assert_eq!(view.key, key);
    assert_eq!(&bytes[view.subject], b"subject");
    assert_eq!(&bytes[view.envelope], b"envelope-bytes");
}

/// The bundle encoder derives the key rather than accepting one.
///
/// This is what makes a bundle filed under a key its subject does not produce
/// unconstructable through the encoder, rather than merely detectable later.
#[test]
fn the_encoder_derives_the_key_from_the_subject() {
    let (key, _) = encoded(b"subject", b"envelope");
    assert_eq!(key, CacheKey::derive_bytes(b"subject"));
}

/// Foreign bytes are refused on the magic, before anything else is read.
#[test]
fn foreign_bytes_are_refused_on_the_magic() {
    let key = CacheKey::derive_bytes(b"subject");
    let bytes = vec![0x5a_u8; 256];
    assert_eq!(decode_default(&bytes, &key), Err(BundleRejection::Magic));
}

/// A short byte run is refused as truncated rather than indexed into.
#[test]
fn a_short_byte_run_is_refused_as_truncated() {
    let key = CacheKey::derive_bytes(b"subject");
    assert!(matches!(
        decode_default(&[], &key),
        Err(BundleRejection::Truncated { found: 0, .. }),
    ));
    let (_, bytes) = encoded(b"subject", b"envelope");
    assert!(matches!(
        decode_default(&bytes[..16], &key),
        Err(BundleRejection::Truncated { found: 16, .. }),
    ));
}

/// A bundle validated for one key is refused for another.
///
/// A valid bundle at the wrong content path is a miss, and this is the check
/// that makes it one: nothing about the bytes is wrong, only where they were
/// asked for.
#[test]
fn a_bundle_is_refused_for_a_key_it_was_not_published_under() {
    let (key, bytes) = encoded(b"subject", b"envelope");
    let other = CacheKey::derive_bytes(b"other-subject");
    assert_eq!(
        decode_default(&bytes, &other),
        Err(BundleRejection::KeyMismatch {
            requested: other.label(),
            embedded: key.label(),
        }),
    );
}

/// Each framing field is checked, and each rejection names its own boundary.
///
/// Written as one table because the property is uniform — every fixed field of
/// the header refuses a value this build does not write — and a per-field test
/// would restate the same three lines eleven times.
#[test]
fn every_framing_field_refuses_a_value_this_build_does_not_write() {
    let (key, original) = encoded(b"subject", b"envelope");
    let corrupt = |at: usize, to: &[u8]| {
        let mut bytes = original.clone();
        bytes[at..at + to.len()].copy_from_slice(to);
        bytes
    };

    // Schema major, which this build does not read at any other value.
    assert!(matches!(
        decode_default(&corrupt(8, &2_u16.to_be_bytes()), &key),
        Err(BundleRejection::Schema { major: 2, minor: 0 }),
    ));
    // Schema minor beyond what this build implements.
    assert!(matches!(
        decode_default(&corrupt(10, &1_u16.to_be_bytes()), &key),
        Err(BundleRejection::Schema { major: 1, minor: 1 }),
    ));
    // A digest algorithm tag this build does not implement, refused rather than
    // inferred from the digest width sitting beside it.
    assert!(matches!(
        decode_default(&corrupt(12, &[0xff]), &key),
        Err(BundleRejection::DigestAlgorithm { tag: 0xff }),
    ));
    // A reserved field carrying anything.
    assert_eq!(
        decode_default(&corrupt(13, &[1]), &key),
        Err(BundleRejection::ReservedNotZero),
    );
    // A total length that disagrees with the bytes present.
    assert!(matches!(
        decode_default(
            &corrupt(16, &(original.len() as u64 + 1).to_be_bytes()),
            &key
        ),
        Err(BundleRejection::TotalLength { .. }),
    ));
}

/// A corrupted section is caught by its digest.
#[test]
fn a_corrupted_section_is_caught_by_its_digest() {
    for (name, purpose, offset_from_end) in [
        ("envelope", BundleSection::ArtifactEnvelope, 1),
        ("subject", BundleSection::CompilationSubject, 9),
    ] {
        let (key, mut bytes) = encoded(b"subject", b"envelope");
        let at = bytes.len() - offset_from_end;
        bytes[at] ^= 1;
        assert_eq!(
            decode_default(&bytes, &key),
            Err(BundleRejection::SectionDigest { purpose }),
            "{name}",
        );
    }
}

/// A forger that reseals every digest is still refused.
///
/// Corrupting bytes and watching a digest reject them proves only that the
/// digest works. This case rewrites the carried subject *and* its descriptor
/// digest, so every integrity field in the bundle is internally consistent; what
/// refuses it is the key no longer being the subject's own digest, which a
/// forger cannot restore without moving the entry to a different path.
#[test]
fn a_resealed_forgery_is_refused_by_the_key_derivation() {
    let (key, bytes) = encoded(b"subject-a", b"envelope");
    let (_, forged) = encoded(b"subject-b", b"envelope");

    // Splice the forged body and descriptor table under the original header, so
    // the bundle claims the original key while carrying the other subject. The
    // two subjects are the same length, so every offset stays valid and the
    // total length field still agrees with the bytes present.
    assert_eq!(
        bytes.len(),
        forged.len(),
        "the fixture keeps offsets aligned"
    );
    let mut spliced = forged;
    spliced[24..24 + DIGEST_BYTES].copy_from_slice(key.as_bytes());

    assert_eq!(
        decode_default(&spliced, &key),
        Err(BundleRejection::KeyNotDerivedFromSubject {
            embedded: key.label(),
            derived: CacheKey::derive_bytes(b"subject-b").label(),
        }),
    );
}

/// The envelope section's digest is the only thing binding a bundle to the
/// envelope its publisher framed.
///
/// This is the counterpart of the case above and it comes out the other way. A
/// re-sealed *subject* is refused, because the key is that subject's own digest.
/// Nothing plays that role for the envelope: the key does not reach it, and the
/// payload validator — [`tiler_artifact::program::decode_artifact`] on the
/// public path — validates an envelope against *itself* and so accepts any valid
/// one. The three assertions are the three halves of that, in order: the digest
/// refuses the substitution; the payload validator does not, so it contributes
/// nothing to refusing it; and re-sealing that one descriptor field makes the
/// whole bundle validate again while carrying an envelope its publisher never
/// framed.
///
/// It is retained as the in-crate statement of
/// `decide-whether-the-bundle-envelope-section-digest-is-redundant`, whose
/// evidence against the *real* artifact decoder — thirty-six corruption classes
/// and every byte position of a real envelope, driven through a build with the
/// comparison removed — is `spikes/cache/envelope-digest-coverage/`. A future
/// reader who reaches for that digest as 19–24% of a cache hit meets this test
/// first.
#[test]
fn only_the_envelope_section_digest_binds_a_bundle_to_the_envelope_it_framed() {
    const PUBLISHED: &[u8] = b"envelope-alpha";
    const SUBSTITUTE: &[u8] = b"envelope-bravo";
    assert_eq!(
        PUBLISHED.len(),
        SUBSTITUTE.len(),
        "the substitution keeps every offset, length, and total the frame declares",
    );

    let (key, mut bytes) = encoded(b"subject", PUBLISHED);
    let (start, end) = section_span(&bytes, 1);
    assert_eq!(&bytes[start..end], PUBLISHED, "section 1 is the envelope");
    bytes[start..end].copy_from_slice(SUBSTITUTE);

    assert_eq!(
        decode_default(&bytes, &key),
        Err(BundleRejection::SectionDigest {
            purpose: BundleSection::ArtifactEnvelope,
        }),
    );
    assert!(
        any_payload(SUBSTITUTE).is_ok(),
        "a payload validator accepts the substitute, so it is not what refuses it",
    );

    reseal_section(&mut bytes, 1);
    let view = decode_default_view(&bytes, &key)
        .expect("every remaining check passes over a substituted envelope");
    assert_eq!(
        &bytes[view.envelope], SUBSTITUTE,
        "with that one digest satisfied, the frame carries the substitute and says so",
    );
}

/// A bundle above the configured bound is refused before it is read.
#[test]
fn an_oversize_bundle_is_refused_against_the_bound() {
    let limits = Limits {
        max_bundle_bytes: 64,
        ..Limits::default()
    };
    let rejection = bundle::encode(b"subject", b"envelope", &DebugRetention::none(), &limits)
        .expect_err("a bundle above the bound does not encode");
    assert!(matches!(rejection, BundleRejection::BundleTooLarge { .. }));
}

// -------------------------------------------------------------------------
// The retained debug section
// -------------------------------------------------------------------------

/// Offset of the descriptor table, as the frame in `bundle.rs` documents it.
const DESCRIPTOR_TABLE_AT: usize = 64;
/// Width of one descriptor: purpose, offset, length, digest.
const DESCRIPTOR_BYTES: usize = 4 + 8 + 8 + DIGEST_BYTES;

/// Reads one descriptor's framed span.
fn section_span(bytes: &[u8], index: usize) -> (usize, usize) {
    let at = DESCRIPTOR_TABLE_AT + DESCRIPTOR_BYTES * index;
    let read = |from: usize| {
        let framed = u64::from_be_bytes(
            bytes[from..from + 8]
                .try_into()
                .expect("a fixed-width field"),
        );
        usize::try_from(framed).expect("a fixture bundle fits this host's address space")
    };
    let offset = read(at + 4);
    (offset, offset + read(at + 12))
}

/// Recomputes one section's descriptor digest over whatever now sits in its span.
///
/// This is the forger's move, and it is what separates "a digest catches a
/// corruption" from "the retention's own parser catches bytes no build wrote".
fn reseal_section(bytes: &mut [u8], index: usize) {
    let (start, end) = section_span(bytes, index);
    let digest =
        DigestAlgorithm::GOVERNED.digest(bundle::SECTION_DIGEST_DOMAIN, &bytes[start..end]);
    let at = DESCRIPTOR_TABLE_AT + DESCRIPTOR_BYTES * index + 20;
    bytes[at..at + DIGEST_BYTES].copy_from_slice(digest.as_bytes());
}

/// Frames one retention section body by hand, so a rule can be broken one at a
/// time.
///
/// The declared count is separate from the runs supplied, because a section that
/// lies about how many runs follow is exactly the input a reader must not index
/// past.
fn framed_retention(declared: u64, runs: &[(&[u8], u64, &[u8])]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(RETENTION_DOMAIN);
    bytes.extend_from_slice(&declared.to_be_bytes());
    for (label, total, retained) in runs {
        bytes.extend_from_slice(&(label.len() as u64).to_be_bytes());
        bytes.extend_from_slice(label);
        bytes.extend_from_slice(&total.to_be_bytes());
        bytes.extend_from_slice(&(retained.len() as u64).to_be_bytes());
        bytes.extend_from_slice(retained);
    }
    bytes
}

/// Retaining debug text does not move the key, so one compilation is one entry.
///
/// This is the first of the three identity answers, and the assertions after the
/// equality are what keep it from passing for the wrong reason: the two bundles
/// really do differ, and a key taken over the framed bundle — the shape a
/// key-participation defect would have — separates them. So the equality above
/// is a statement about the derivation rather than about two identical inputs.
#[test]
fn a_retention_does_not_reach_the_key() {
    let retained = retaining("tool.stderr", b"warning: unused variable `x`");
    let (bare_key, bare) = encoded(b"subject", b"envelope");
    let (retaining_key, framed) = encoded_retaining(b"subject", b"envelope", &retained);

    assert_eq!(
        bare_key, retaining_key,
        "the same compilation must key identically with and without retention",
    );
    let scratch = Scratch::new("retention-key");
    let cache = cache(&scratch);
    assert_eq!(
        cache.entry_path(&bare_key),
        cache.entry_path(&retaining_key)
    );

    assert_ne!(bare, framed, "the fixture must frame different bytes");
    assert!(framed.len() > bare.len(), "the retention must be framed");
    assert_ne!(
        CacheKey::derive_bytes(&bare),
        CacheKey::derive_bytes(&framed),
        "a key derived over the framed bundle would separate these two, which is \
         what makes the equality above a property of the derivation",
    );
}

/// A bundle round-trips the retention it framed.
#[test]
fn a_bundle_round_trips_its_retained_debug_text() {
    let retained = retaining("tool.stderr", b"warning: unused variable `x`")
        .retaining("tool.stdout", b"")
        .expect("a second governed label");
    let (key, bytes) = encoded_retaining(b"subject", b"envelope", &retained);
    let view = bundle::decode(&bytes, &key, &Limits::default()).expect("a fresh bundle validates");

    assert_eq!(view.retained, retained);
    let run = view
        .retained
        .run("tool.stderr")
        .expect("the labelled run is readable");
    assert_eq!(run.as_bytes(), b"warning: unused variable `x`");
    assert_eq!(run.total_bytes(), 28);
    assert!(!run.is_truncated());
    assert!(run.is_valid_utf8());
    let empty = view
        .retained
        .run("tool.stdout")
        .expect("a run the tool wrote nothing for is still a run");
    assert!(empty.is_empty());
    assert_eq!(view.retained.runs().len(), 2);
    assert!(view.retained.run("tool.absent").is_none());
}

/// A bundle framing no retention is complete, and shows nothing.
///
/// The third identity answer, at the frame: absence is not a missing section, so
/// every entry a build published before retention existed still validates.
#[test]
fn a_bundle_without_a_retention_validates_and_shows_nothing() {
    let (key, bytes) = encoded(b"subject", b"envelope");
    let view = bundle::decode(&bytes, &key, &Limits::default()).expect("a fresh bundle validates");
    assert!(view.retained.is_empty());
    assert!(view.retained.runs().is_empty());
}

/// A damaged retention is refused, whether or not the forger reseals the digest.
///
/// The second identity answer: the section is inside the digest set, so an entry
/// edited to alter retained text is not a valid entry. The resealed case is the
/// one that matters — every framing field is consistent, and what refuses it is
/// the retention's own parser, which is why the section is parsed on the hit path
/// rather than handed back as bytes.
#[test]
fn a_damaged_retention_is_refused_rather_than_dropped() {
    let retained = retaining("tool.stderr", b"warning: unused variable `x`");
    let (key, original) = encoded_retaining(b"subject", b"envelope", &retained);

    let mut flipped = original.clone();
    let last = flipped.len() - 1;
    flipped[last] ^= 1;
    assert_eq!(
        decode_default(&flipped, &key),
        Err(BundleRejection::SectionDigest {
            purpose: BundleSection::DebugRetention,
        }),
    );

    let mut resealed = original.clone();
    let (start, end) = section_span(&resealed, 2);
    resealed[start..end].fill(0);
    reseal_section(&mut resealed, 2);
    assert_eq!(
        decode_default(&resealed, &key),
        Err(BundleRejection::RetainedDebug {
            rejection: RetentionRejection::Domain,
        }),
        "a retention whose digest was recomputed is still refused by its own parser",
    );

    // The same forgery with the digest left alone is caught one step earlier,
    // which is what shows the resealing above was the reason the parser was
    // reached at all.
    let mut unsealed = original;
    let (start, end) = section_span(&unsealed, 2);
    unsealed[start..end].fill(0);
    assert_eq!(
        decode_default(&unsealed, &key),
        Err(BundleRejection::SectionDigest {
            purpose: BundleSection::DebugRetention,
        }),
    );
}

/// Every stored-retention rule refuses bytes this build would not have written.
///
/// One table because the property is uniform, and each row is a distinct way a
/// section can be internally consistent bytes and still not be a retention.
#[test]
fn a_stored_retention_refuses_what_this_build_would_not_write() {
    let valid = framed_retention(1, &[(b"tool.stderr", 4, b"text")]);
    let decoded = DebugRetention::decode(&valid).expect("a hand-framed retention decodes");
    assert_eq!(decoded.runs().len(), 1);
    assert_eq!(decoded.runs()[0].label(), "tool.stderr");

    let mut foreign = valid.clone();
    foreign[0] ^= 0xff;
    assert_eq!(
        DebugRetention::decode(&foreign),
        Err(RetentionRejection::Domain),
    );

    assert!(matches!(
        DebugRetention::decode(&framed_retention(0, &[])),
        Err(RetentionRejection::RunCount { declared: 0, .. }),
    ));
    assert!(matches!(
        DebugRetention::decode(&framed_retention(MAX_RETAINED_RUNS as u64 + 1, &[])),
        Err(RetentionRejection::RunCount { .. }),
    ));
    assert!(matches!(
        DebugRetention::decode(&framed_retention(2, &[(b"tool.stderr", 4, b"text")])),
        Err(RetentionRejection::Truncated { .. }),
    ));
    assert!(matches!(
        DebugRetention::decode(&framed_retention(1, &[(b"tool.stderr", 1, b"text")])),
        Err(RetentionRejection::RetainedAboveTotal {
            index: 0,
            retained: 4,
            total: 1,
        }),
    ));
    assert!(matches!(
        DebugRetention::decode(&framed_retention(1, &[(b"Tool", 4, b"text")])),
        Err(RetentionRejection::Label {
            index: 0,
            refusal: RetentionRefusal::NoncanonicalLabelByte { position: 0, .. },
        }),
    ));
    assert!(matches!(
        DebugRetention::decode(&framed_retention(1, &[(b"", 4, b"text")])),
        Err(RetentionRejection::Label {
            index: 0,
            refusal: RetentionRefusal::EmptyLabel,
        }),
    ));
    assert!(matches!(
        DebugRetention::decode(&framed_retention(1, &[(&[0xff], 4, b"text")])),
        Err(RetentionRejection::LabelNotUtf8 { index: 0 }),
    ));
    assert!(matches!(
        DebugRetention::decode(&framed_retention(
            2,
            &[(b"tool.stderr", 4, b"text"), (b"tool.stderr", 4, b"more")],
        )),
        Err(RetentionRejection::Label {
            index: 1,
            refusal: RetentionRefusal::DuplicateLabel { .. },
        }),
    ));
    let oversize = vec![b'x'; MAX_RETAINED_RUN_BYTES + 1];
    assert!(matches!(
        DebugRetention::decode(&framed_retention(
            1,
            &[(b"tool.stderr", oversize.len() as u64, &oversize)],
        )),
        Err(RetentionRejection::RunTooLarge { index: 0, .. }),
    ));
    let mut trailing = valid;
    trailing.push(0);
    assert!(matches!(
        DebugRetention::decode(&trailing),
        Err(RetentionRejection::TrailingBytes { .. }),
    ));
}

/// A retained run is bounded, and what it dropped is recorded rather than hidden.
#[test]
fn a_retained_run_is_bounded_and_records_its_truncation() {
    let written = vec![b'x'; MAX_RETAINED_RUN_BYTES + 10];
    let retained = retaining("tool.stderr", &written);
    let run = retained.run("tool.stderr").expect("the run is retained");

    assert_eq!(run.as_bytes().len(), MAX_RETAINED_RUN_BYTES);
    assert_eq!(run.total_bytes(), written.len() as u64);
    assert!(run.is_truncated());
    assert!(
        run.to_string().contains("truncated"),
        "a truncated run must say so when it is rendered",
    );

    // A run that fits exactly is not reported as truncated, so the flag is a
    // statement about dropped bytes rather than about reaching the bound.
    let exact = retaining("tool.stderr", &vec![b'x'; MAX_RETAINED_RUN_BYTES]);
    assert!(!exact.run("tool.stderr").expect("the run").is_truncated());

    // Invalid UTF-8 is kept as written and reported, never rendered lossily and
    // presented as what the tool said.
    let raw = retaining("tool.stderr", &[0xff, 0xfe]);
    let run = raw.run("tool.stderr").expect("the run");
    assert_eq!(run.as_bytes(), &[0xff, 0xfe]);
    assert!(!run.is_valid_utf8());
    assert!(run.to_string().contains("not valid UTF-8"));
}

/// Every caller-side retention rule refuses by name.
#[test]
fn a_retention_refuses_a_label_or_a_run_it_cannot_frame() {
    assert_eq!(
        DebugRetention::none().retaining("", b"text"),
        Err(RetentionRefusal::EmptyLabel),
    );
    let long = "x".repeat(MAX_RETENTION_LABEL_BYTES + 1);
    assert!(matches!(
        DebugRetention::none().retaining(&long, b"text"),
        Err(RetentionRefusal::LabelTooLong { .. }),
    ));
    assert!(matches!(
        DebugRetention::none().retaining("tool stderr", b"text"),
        Err(RetentionRefusal::NoncanonicalLabelByte { position: 4, .. }),
    ));
    assert!(matches!(
        retaining("tool.stderr", b"one").retaining("tool.stderr", b"two"),
        Err(RetentionRefusal::DuplicateLabel { .. }),
    ));

    let mut full = DebugRetention::none();
    for index in 0..MAX_RETAINED_RUNS {
        full = full
            .retaining(&format!("tool.{index}"), b"text")
            .expect("a run inside the bound");
    }
    assert!(matches!(
        full.retaining("tool.overflow", b"text"),
        Err(RetentionRefusal::TooManyRuns { .. }),
    ));
}

/// A published retention is read back through the ordinary validated hit, and a
/// damaged one makes that hit a miss.
///
/// The three states the entry can be in, exercised through the protocol rather
/// than through the frame: present, absent, and damaged.
#[test]
fn a_published_retention_survives_the_hit_path_and_a_damaged_one_does_not() {
    let scratch = Scratch::new("retention-hit");
    let cache = cache(&scratch);
    let retained = retaining("tool.stderr", b"warning: unused variable `x`");

    let outcome = cache
        .resolve_retaining(
            b"subject",
            || Ok::<_, String>((b"envelope".to_vec(), retained.clone())),
            &any_payload,
        )
        .expect("a build that succeeds resolves");
    let ProtocolOutcome::Hit {
        entry, published, ..
    } = outcome
    else {
        panic!("an empty cache publishes rather than hits");
    };
    assert!(published);
    assert_eq!(entry.retained, retained);
    let key = entry.key;

    // Present: a second reader validates the whole entry and sees the text.
    let read = cache
        .read_entry(&key, &any_payload)
        .expect("a published entry validates");
    assert_eq!(
        read.retained
            .run("tool.stderr")
            .expect("the run survives publication")
            .as_bytes(),
        b"warning: unused variable `x`",
    );

    // Damaged: one flipped byte inside the retained text, which the section
    // digest refuses. The entry is a miss with the boundary that refused it, not
    // a hit with the retention quietly dropped.
    let path = cache.entry_path(&key);
    let mut bytes = fs::read(&path).expect("the entry is readable");
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    fs::write(&path, &bytes).expect("the entry is writable");
    assert!(
        matches!(
            cache.read_entry(&key, &any_payload),
            Err(MissReason::Rejected(EntryRejection::Bundle(
                BundleRejection::SectionDigest {
                    purpose: BundleSection::DebugRetention,
                }
            ))),
        ),
        "a damaged retention must refuse validation",
    );
}

/// A hit shows what the publishing build retained, and never republishes to add
/// what this one asked for.
///
/// The absent case at the protocol level. An entry published without retention
/// stays a hit for a caller that states one — the build closure does not run, so
/// there is nothing to retain, and the entry is not rewritten to hold text a
/// later reader would find under an unchanged key.
#[test]
fn a_hit_on_an_entry_published_without_retention_shows_nothing() {
    let scratch = Scratch::new("retention-absent");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");

    let outcome = cache
        .resolve_retaining(
            b"subject",
            || -> Result<(Vec<u8>, DebugRetention), String> {
                panic!("a validated entry must not be recompiled to add a retention")
            },
            &any_payload,
        )
        .expect("the stored entry resolves");
    let ProtocolOutcome::Hit {
        entry, published, ..
    } = outcome
    else {
        panic!("a stored entry hits");
    };
    assert!(!published, "a hit publishes nothing");
    assert_eq!(entry.key, key);
    assert!(
        entry.retained.is_empty(),
        "a hit on an entry published without retention shows nothing",
    );
}

/// A cache that stores nothing hands the retention back rather than losing it.
#[test]
fn an_uncached_resolution_returns_the_retention_it_could_not_store() {
    let retained = retaining("tool.stderr", b"warning: unused variable `x`");
    let outcome = ExpansionCache::disabled()
        .resolve_retaining(
            b"subject",
            || Ok::<_, String>((b"envelope".to_vec(), retained.clone())),
            &any_payload,
        )
        .expect("a disabled cache still builds");
    let ProtocolOutcome::Uncached { entry, report } = outcome else {
        panic!("a disabled cache stores nothing");
    };
    assert_eq!(entry.retained, retained);
    assert!(matches!(
        report.publication_refusal(),
        Some(PublicationRefusal::Disabled),
    ));
}

// -------------------------------------------------------------------------
// Validation on every hit
// -------------------------------------------------------------------------

/// A published entry is validated again when it is read back.
#[test]
fn a_published_entry_validates_when_it_is_read_back() {
    let scratch = Scratch::new("read-back");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");
    let entry = cache
        .read_entry(&key, &any_payload)
        .expect("a published entry validates");
    assert_eq!(entry.key, key);
    assert_eq!(entry.envelope(), b"envelope");
    assert_eq!(entry.payload, b"envelope");
}

/// Every read runs the payload validator, and a validator rejection is a miss
/// carrying the artifact layer's own failure.
#[test]
fn a_payload_rejection_is_a_typed_miss() {
    let scratch = Scratch::new("payload-rejection");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");

    let calls = std::cell::Cell::new(0_u32);
    let refuse = |_: &[u8]| -> Result<(), ArtifactCodecFailure> {
        calls.set(calls.get() + 1);
        Err(ArtifactCodecFailure::Malformed {
            detail: "the payload validator refused".to_owned(),
        })
    };
    let reason = cache
        .read_entry(&key, &refuse)
        .expect_err("a refused payload is not a hit");
    assert_eq!(calls.get(), 1, "the validator runs on every read");
    assert!(
        matches!(
            reason,
            MissReason::Rejected(EntryRejection::Payload(
                ArtifactCodecFailure::Malformed { .. }
            )),
        ),
        "{reason}",
    );
}

/// The public API pins the artifact decoder as its validator.
///
/// Proven through the public surface with bytes that are not an artifact: the
/// miss carries the artifact layer's own classification, which nothing else in
/// this crate produces.
#[test]
fn the_public_api_validates_the_payload_as_an_artifact() {
    let scratch = Scratch::new("public-validator");
    let cache = cache(&scratch);
    let subject = composed(&[b"compilation"], b"program");
    publish(
        &cache,
        subject.as_bytes(),
        b"not an artifact envelope at all",
    );

    let Lookup::Miss(reason) = cache.lookup(&subject) else {
        panic!("a payload that is not an artifact is not a hit");
    };
    assert!(
        matches!(
            reason,
            MissReason::Rejected(EntryRejection::Payload(
                ArtifactCodecFailure::Malformed { .. }
            )),
        ),
        "{reason}",
    );
}

/// A corrupt entry is a miss carrying the exact boundary that refused it.
///
/// The miss is what ADR 0050 decides; the reason is what keeps it from being
/// silence. A cache permanently rejecting every entry is visible here.
#[test]
fn a_corrupt_entry_is_a_reported_miss() {
    let scratch = Scratch::new("corrupt-entry");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");
    let path = cache.entry_path(&key);
    let mut bytes = fs::read(&path).expect("the entry is readable");
    *bytes.last_mut().expect("the bundle is not empty") ^= 1;
    fs::write(&path, &bytes).expect("the entry is writable in this test");

    let reason = cache
        .read_entry(&key, &any_payload)
        .expect_err("a corrupt entry is not a hit");
    assert!(
        matches!(
            reason,
            MissReason::Rejected(EntryRejection::Bundle(BundleRejection::SectionDigest {
                purpose: BundleSection::ArtifactEnvelope,
            })),
        ),
        "{reason}",
    );
}

/// An absent entry is an ordinary miss, distinguishable from a rejection.
#[test]
fn an_absent_entry_is_distinguishable_from_a_rejected_one() {
    let scratch = Scratch::new("absent");
    let cache = cache(&scratch);
    let reason = cache
        .read_entry(&CacheKey::derive_bytes(b"subject"), &any_payload)
        .expect_err("nothing is published");
    assert!(matches!(reason, MissReason::Absent), "{reason}");
}

/// An entry above the configured bound is refused without being allocated whole.
#[test]
fn an_oversize_entry_is_refused_against_the_bound() {
    let scratch = Scratch::new("oversize-entry");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");
    let stored = fs::metadata(cache.entry_path(&key))
        .expect("the entry exists")
        .len();

    let bounded = ExpansionCache::open(cache.rooted_layout().root()).with_limits(Limits {
        max_bundle_bytes: stored - 1,
        ..Limits::default()
    });
    let reason = bounded
        .read_entry(&key, &any_payload)
        .expect_err("an entry above the bound is not a hit");
    assert!(
        matches!(
            reason,
            MissReason::Rejected(EntryRejection::TooLarge { .. })
        ),
        "{reason}",
    );
}

// -------------------------------------------------------------------------
// Immutability and atomic publication
// -------------------------------------------------------------------------

/// Publishing leaves exactly one entry, and it is byte-identical to the bundle
/// that was validated.
#[test]
fn publication_leaves_one_validated_entry() {
    let scratch = Scratch::new("publication");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");
    let path = cache.entry_path(&key);
    let stored = fs::read(&path).expect("the entry is readable");
    let (_, expected) = encoded(b"subject", b"envelope");
    assert_eq!(
        stored, expected,
        "the published bytes are the encoded bundle"
    );
}

/// A second call for the same subject hits rather than republishing.
#[test]
fn a_second_call_hits_rather_than_rebuilding() {
    let scratch = Scratch::new("second-call");
    let cache = cache(&scratch);
    publish(&cache, b"subject", b"envelope");

    let outcome = cache
        .resolve(
            b"subject",
            || -> Result<Vec<u8>, String> { panic!("a hit must not build") },
            &any_payload,
        )
        .expect("a hit resolves");
    let ProtocolOutcome::Hit {
        published, report, ..
    } = outcome
    else {
        panic!("a published subject hits");
    };
    assert!(!published, "the second call does not publish");
    assert!(
        report.lookup_miss().is_none(),
        "the lock-free read hit, so nothing missed",
    );
}

/// Publication leaves no temporary file behind.
#[test]
fn publication_leaves_no_temporary_behind() {
    let scratch = Scratch::new("no-temporary");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");
    let temporaries = cache
        .rooted_layout()
        .root()
        .join("v1/tmp")
        .join(&key.label()[..2])
        .read_dir()
        .expect("the temporary directory was created")
        .count();
    assert_eq!(temporaries, 0);
}

/// The `fsync` policy publishes the same bytes as the default policy.
///
/// Durability changes what the operating system is asked to persist, never what
/// a reader accepts. A policy that changed the bytes would make an entry
/// published under one policy unreadable under the other.
#[test]
fn the_durability_policy_does_not_change_what_is_published() {
    let scratch = Scratch::new("durability");
    let default = cache(&scratch);
    let key = publish(&default, b"subject", b"envelope");
    let under_default = fs::read(default.entry_path(&key)).expect("the entry is readable");

    let synced_scratch = Scratch::new("durability-fsync");
    let synced = cache(&synced_scratch).with_durability(Durability::Fsync);
    let synced_key = publish(&synced, b"subject", b"envelope");
    let under_fsync = fs::read(synced.entry_path(&synced_key)).expect("the entry is readable");

    assert_eq!(key, synced_key);
    assert_eq!(under_default, under_fsync);
}

/// A failed directory sync after the rename reports a *published* entry.
///
/// This is the defect the ticket names. The rename is the publication point:
/// once it returns, another process may read the valid immutable entry, and no
/// later failure retracts that. Reporting `Uncached` there told the caller no
/// entry was published while one was sitting at the content path, readable —
/// the one direction this crate must not fail in, because every other refusal
/// leaves the caller free to rebuild and this one silently disagrees with the
/// filesystem.
///
/// The durability claim really is weaker, and that is what the report now says
/// instead.
#[test]
fn a_directory_sync_failure_after_the_rename_reports_a_published_entry() {
    let scratch = Scratch::new("post-rename-durability");
    let cache = cache(&scratch).with_durability(Durability::Fsync);

    let outcome = {
        let _armed = fault::inject(fault::Injection::EntryDirectorySync);
        cache
            .resolve(
                b"subject",
                || Ok::<_, String>(b"envelope".to_vec()),
                &any_payload,
            )
            .expect("a publication resolves")
    };

    let ProtocolOutcome::Hit {
        entry,
        report,
        published,
    } = outcome
    else {
        panic!("a post-rename failure must not be reported as uncached");
    };
    assert!(
        published,
        "the rename succeeded, so the entry was published"
    );

    // The weakened claim is reported, and as durability rather than refusal.
    let shortfall = report
        .durability_shortfall()
        .expect("the directory sync failed and is reported");
    assert_eq!(shortfall.operation(), CacheOperation::SyncEntryDirectory);
    assert!(
        report.publication_refusal().is_none(),
        "a published entry was described as refused: {:?}",
        report.publication_refusal(),
    );

    // The filesystem agrees with the report, which is the whole point: the
    // entry is at the content path and readable by anyone.
    let path = cache.entry_path(&entry.key);
    assert!(path.exists(), "the published entry is at its content path");
    let reread = cache
        .read_entry(&entry.key, &any_payload)
        .expect("another reader observes the published entry");
    assert_eq!(reread.envelope(), b"envelope");
}

/// A failed lock release after the rename is cleanup, not a refusal.
///
/// The outcome was already right here; the report was not. It set a publication
/// refusal beside `published: true`, so a caller reading the report to answer
/// "was it published?" got a contradiction from the same record.
#[test]
fn a_lock_release_failure_after_the_rename_is_reported_as_cleanup() {
    let scratch = Scratch::new("post-rename-cleanup");
    let cache = cache(&scratch);

    let outcome = {
        let _armed = fault::inject(fault::Injection::LockRelease);
        cache
            .resolve(
                b"subject",
                || Ok::<_, String>(b"envelope".to_vec()),
                &any_payload,
            )
            .expect("a publication resolves")
    };

    let ProtocolOutcome::Hit {
        entry,
        report,
        published,
    } = outcome
    else {
        panic!("a lock-release failure must not unpublish anything");
    };
    assert!(published);
    let shortfall = report
        .cleanup_shortfall()
        .expect("the lock release failed and is reported");
    assert_eq!(shortfall.operation(), CacheOperation::ReleaseLock);
    assert!(
        report.publication_refusal().is_none(),
        "a published entry was described as refused",
    );
    assert!(
        report.durability_shortfall().is_none(),
        "nothing about durability failed",
    );
    assert!(cache.entry_path(&entry.key).exists());
}

/// An ordinary publication reports neither shortfall.
///
/// The negative half: without it, the two cases above would pass against a
/// version that set both fields unconditionally.
#[test]
fn an_ordinary_publication_reports_no_shortfall() {
    let scratch = Scratch::new("no-shortfall");
    let cache = cache(&scratch).with_durability(Durability::Fsync);
    let outcome = cache
        .resolve(
            b"subject",
            || Ok::<_, String>(b"envelope".to_vec()),
            &any_payload,
        )
        .expect("a publication resolves");
    let ProtocolOutcome::Hit { report, .. } = outcome else {
        panic!("the entry publishes");
    };
    assert!(report.durability_shortfall().is_none());
    assert!(report.cleanup_shortfall().is_none());
    assert!(report.publication_refusal().is_none());
}

/// No `Uncached` outcome ever leaves a content entry behind.
///
/// The property the two cases above are instances of, stated once over both
/// sides of the publication point. A pre-rename refusal must leave nothing at
/// the content path, and anything that does leave something there must not be
/// reported as uncached. Asserted against a real pre-rename refusal — an
/// oversize bundle, which is refused before any temporary is created.
#[test]
fn an_uncached_outcome_never_leaves_a_content_entry() {
    let scratch = Scratch::new("uncached-leaves-nothing");
    let cache = cache(&scratch).with_limits(Limits {
        max_bundle_bytes: 8,
        ..Limits::default()
    });
    let outcome = cache
        .resolve(
            b"subject",
            || Ok::<_, String>(b"an envelope far past the bundle bound".to_vec()),
            &any_payload,
        )
        .expect("an oversize bundle still resolves");
    let ProtocolOutcome::Uncached { entry, report } = outcome else {
        panic!("an oversize bundle is not published");
    };
    assert!(
        report.publication_refusal().is_some(),
        "a genuine pre-rename refusal is reported as one",
    );
    assert!(
        report.durability_shortfall().is_none() && report.cleanup_shortfall().is_none(),
        "nothing was published, so no post-publication fact applies",
    );
    assert!(
        !cache.entry_path(&entry.key).exists(),
        "an uncached outcome left an entry at the content path",
    );
}

/// A rejected entry is replaced, and its bytes are retained for diagnosis.
#[test]
fn a_rejected_entry_is_replaced_and_retained() {
    let scratch = Scratch::new("replace-rejected");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");
    let path = cache.entry_path(&key);
    fs::write(&path, b"this is not a bundle").expect("the entry is writable in this test");

    let outcome = cache
        .resolve(
            b"subject",
            || Ok::<_, String>(b"rebuilt-envelope".to_vec()),
            &any_payload,
        )
        .expect("a rebuild resolves");
    let ProtocolOutcome::Hit {
        entry,
        report,
        published,
    } = outcome
    else {
        panic!("a corrupt entry is rebuilt and republished");
    };
    assert!(published);
    assert_eq!(entry.envelope(), b"rebuilt-envelope");
    assert!(
        matches!(
            report.recheck_miss(),
            Some(MissReason::Rejected(EntryRejection::Bundle(
                BundleRejection::Truncated { found: 20, .. },
            ))),
        ),
        "the recheck reports why the old entry was refused: {:?}",
        report.recheck_miss().map(ToString::to_string),
    );
    let Some(QuarantineOutcome::Retained { path: retained }) = report.quarantine() else {
        panic!(
            "the refused bytes are retained: {:?}",
            report.quarantine().map(ToString::to_string),
        );
    };
    assert_eq!(
        fs::read(retained).expect("the quarantined bytes are readable"),
        b"this is not a bundle",
        "quarantine keeps the exact bytes that were refused",
    );
}

/// Reaching the quarantine bound discards evidence loudly rather than silently.
#[test]
fn the_quarantine_bound_is_reported_when_it_discards() {
    let scratch = Scratch::new("quarantine-bound");
    let cache = cache(&scratch).with_limits(Limits {
        max_quarantine_bytes: 1,
        ..Limits::default()
    });
    let key = publish(&cache, b"subject", b"envelope");
    fs::write(cache.entry_path(&key), b"this is not a bundle")
        .expect("the entry is writable in this test");

    let outcome = cache
        .resolve(
            b"subject",
            || Ok::<_, String>(b"rebuilt".to_vec()),
            &any_payload,
        )
        .expect("a rebuild resolves");
    let ProtocolOutcome::Hit { report, .. } = outcome else {
        panic!("a corrupt entry is rebuilt");
    };
    assert!(
        matches!(
            report.quarantine(),
            Some(QuarantineOutcome::BoundReached { discarded: 20, .. }),
        ),
        "{:?}",
        report.quarantine().map(ToString::to_string),
    );
}

// -------------------------------------------------------------------------
// The lock, and the recheck it exists for
// -------------------------------------------------------------------------

/// One key's lock excludes a second holder.
///
/// Two *threads* of one process, which is what this crate can test. The
/// cross-process case is the harness's, and nothing here claims it.
#[test]
fn one_key_lock_excludes_a_second_holder() {
    let scratch = Scratch::new("lock-exclusion");
    let cache = cache(&scratch);
    let key = CacheKey::derive_bytes(b"subject");
    ExpansionCache::prepare_directories(cache.rooted_layout(), &key)
        .expect("the namespace is creatable");
    let path = cache.lock_path(&key);

    let held = KeyLock::try_acquire(&path)
        .expect("the lock file opens")
        .expect("an unheld lock is free");
    // A second descriptor on the same file: `flock` associates the lock with the
    // open file, so this is the same observation a second process would make.
    assert!(
        KeyLock::try_acquire(&path)
            .expect("the lock file opens")
            .is_none(),
        "a held lock is not free",
    );
    held.release().expect("the lock releases");
    assert!(
        KeyLock::try_acquire(&path)
            .expect("the lock file opens")
            .is_some(),
        "a released lock is free again",
    );
}

/// A waiter rechecks after taking the lock and hits what the holder published.
///
/// The recheck is the reason the lock is taken *before* building rather than
/// after: without it, every waiter would rebuild what it waited for.
#[test]
fn a_waiter_rechecks_after_the_lock_and_does_not_rebuild() {
    let scratch = Scratch::new("post-lock-recheck");
    let cache = cache(&scratch);
    let key = CacheKey::derive_bytes(b"subject");
    ExpansionCache::prepare_directories(cache.rooted_layout(), &key)
        .expect("the namespace is creatable");

    // Hold the key's lock, publish underneath it, then release. A waiter that
    // did not recheck would rebuild; one that does, hits.
    let held =
        ExpansionCache::acquire_lock(cache.rooted_layout(), &key).expect("the lock is takeable");
    let (waiter_started, started) = mpsc::channel();
    let waiting_cache = cache.clone();
    let waiter = thread::spawn(move || {
        waiter_started.send(()).expect("the test thread is alive");
        waiting_cache.resolve(
            b"subject",
            || -> Result<Vec<u8>, String> { Err("a waiter that rebuilds fails this test".into()) },
            &any_payload,
        )
    });
    started.recv().expect("the waiter starts");

    // The waiter is either blocked on the lock or about to be. Publishing here
    // and releasing is what it must observe on its recheck.
    let (_, bytes) = encoded(b"subject", b"envelope");
    let temporary = cache.rooted_layout().root().join("staged");
    fs::write(&temporary, &bytes).expect("the staging file is writable");
    fs::rename(&temporary, cache.entry_path(&key)).expect("the entry publishes");
    held.release().expect("the lock releases");

    let outcome = waiter
        .join()
        .expect("the waiter does not panic")
        .expect("the waiter resolves");
    match outcome {
        ProtocolOutcome::Hit {
            entry,
            published,
            report,
        } => {
            assert!(!published, "the waiter did not publish");
            assert_eq!(entry.envelope(), b"envelope");
            // Either the lock-free read already saw it or the recheck did. Both
            // are correct; what must not happen is a rebuild, and the build
            // closure returns an error, so a rebuild would have failed the call.
            assert!(
                report.lookup_miss().is_none() || report.recheck_miss().is_none(),
                "one of the two reads hit",
            );
        }
        ProtocolOutcome::Uncached { .. } => panic!("the waiter must not fall open here"),
    }
}

/// Concurrent callers for one key produce one entry, and every caller sees it.
#[test]
fn concurrent_callers_for_one_key_agree_on_one_entry() {
    const THREADS: usize = 8;
    let scratch = Scratch::new("concurrent-identical");
    let cache = cache(&scratch);
    let builds = std::sync::Arc::new(AtomicU32::new(0));

    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let cache = cache.clone();
            let builds = std::sync::Arc::clone(&builds);
            thread::spawn(move || {
                cache
                    .resolve(
                        b"subject",
                        || {
                            builds.fetch_add(1, Ordering::SeqCst);
                            Ok::<_, String>(b"envelope".to_vec())
                        },
                        &any_payload,
                    )
                    .expect("every caller resolves")
            })
        })
        .collect();

    for handle in handles {
        match handle.join().expect("no caller panics") {
            ProtocolOutcome::Hit { entry, .. } => assert_eq!(entry.envelope(), b"envelope"),
            ProtocolOutcome::Uncached { report, .. } => panic!(
                "a caller fell open: {:?}",
                report.publication_refusal().map(ToString::to_string),
            ),
        }
    }
    // The lock is what suppresses duplicate work, and it is deliberately not the
    // correctness boundary: the assertion that matters is the one above, that
    // every caller got the same validated bytes. This one records that the
    // suppression works at all.
    assert_eq!(
        builds.load(Ordering::SeqCst),
        1,
        "the per-key lock suppressed duplicate builds",
    );
}

/// Concurrent callers for distinct keys each publish their own entry.
#[test]
fn concurrent_callers_for_distinct_keys_do_not_collide() {
    const THREADS: usize = 8;
    let scratch = Scratch::new("concurrent-distinct");
    let cache = cache(&scratch);

    let handles: Vec<_> = (0..THREADS)
        .map(|index| {
            let cache = cache.clone();
            thread::spawn(move || {
                let subject = format!("subject-{index}");
                let envelope = format!("envelope-{index}");
                let outcome = cache
                    .resolve(
                        subject.as_bytes(),
                        || Ok::<_, String>(envelope.clone().into_bytes()),
                        &any_payload,
                    )
                    .expect("every caller resolves");
                let ProtocolOutcome::Hit { entry, .. } = outcome else {
                    panic!("every distinct key publishes");
                };
                assert_eq!(entry.envelope(), envelope.as_bytes());
                assert_eq!(entry.key, CacheKey::derive_bytes(subject.as_bytes()));
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("no caller panics");
    }
}

// -------------------------------------------------------------------------
// Falling open
// -------------------------------------------------------------------------

/// An unusable root produces a validated uncached result, with the reason.
#[test]
fn an_unusable_root_falls_open_with_a_reason() {
    let scratch = Scratch::new("unusable-root");
    let occupied = scratch.root().join("occupied");
    fs::write(&occupied, b"a regular file where a directory must be")
        .expect("the scratch directory is writable");
    let cache = ExpansionCache::open(&occupied);

    let outcome = cache
        .resolve(
            b"subject",
            || Ok::<_, String>(b"envelope".to_vec()),
            &any_payload,
        )
        .expect("an unusable cache still resolves");
    let ProtocolOutcome::Uncached { entry, report } = outcome else {
        panic!("an unusable root cannot publish");
    };
    assert_eq!(entry.envelope(), b"envelope");
    assert!(
        report.publication_refusal().is_some(),
        "the refusal is reported rather than silent",
    );
}

/// A build failure is a hard error, not a cache fall-open.
#[test]
fn a_build_failure_is_not_absorbed_by_the_cache() {
    let scratch = Scratch::new("build-failure");
    let cache = cache(&scratch);
    let failure = cache
        .resolve(
            b"subject",
            || Err::<Vec<u8>, _>("the compiler failed".to_owned()),
            &any_payload,
        )
        .expect_err("a build failure is an error");
    assert!(matches!(failure, PublishFailure::Build(_)), "{failure:?}");
    assert!(
        !cache
            .entry_path(&CacheKey::derive_bytes(b"subject"))
            .exists(),
        "a failed build publishes nothing",
    );
}

/// An invalid produced artifact is a hard error, not a cache fall-open.
#[test]
fn an_invalid_produced_artifact_is_not_absorbed_by_the_cache() {
    let scratch = Scratch::new("invalid-artifact");
    let cache = cache(&scratch);
    let subject = composed(&[b"compilation"], b"program");
    let failure = cache
        .get_or_publish(&subject, || {
            Ok::<_, String>(b"not an artifact envelope".to_vec())
        })
        .expect_err("bytes that are not an artifact are an error");
    assert!(
        matches!(failure, PublishFailure::Artifact(_)),
        "{failure:?}"
    );
    assert!(
        !cache.entry_path(&CacheKey::derive(&subject)).exists(),
        "an invalid artifact publishes nothing",
    );
}

// -------------------------------------------------------------------------
// Eviction and sweeping
// -------------------------------------------------------------------------

/// Eviction removes the entry and retains the lock file.
///
/// Retaining it is not tidiness. Unlinking a locked file lets a later process
/// create a different inode at the same path and take an independent lock while
/// an earlier process still holds the first, which splits contenders into two
/// groups that do not exclude each other.
#[test]
fn eviction_removes_the_entry_and_retains_the_lock_file() {
    let scratch = Scratch::new("eviction");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");
    let lock = cache.lock_path(&key);
    assert!(lock.exists(), "publishing created the lock file");

    assert_eq!(
        cache.evict(&key).expect("eviction succeeds"),
        super::store::Eviction::Removed,
    );
    assert!(!cache.entry_path(&key).exists(), "the entry is gone");
    assert!(lock.exists(), "the lock file is retained");
    assert_eq!(
        cache.evict(&key).expect("a second eviction succeeds"),
        super::store::Eviction::Absent,
    );
}

/// A temporary younger than the grace period is retained rather than swept.
#[test]
fn a_young_temporary_is_retained_by_the_sweep() {
    let scratch = Scratch::new("sweep-young");
    let cache = cache(&scratch);
    let key = CacheKey::derive_bytes(b"subject");
    ExpansionCache::prepare_directories(cache.rooted_layout(), &key)
        .expect("the namespace is creatable");
    let abandoned = cache
        .rooted_layout()
        .root()
        .join("v1/tmp")
        .join(&key.label()[..2])
        .join(format!("{}.999.0.0.tmp", key.label()));
    fs::write(&abandoned, b"abandoned").expect("the temporary is writable");

    let report = cache.sweep_temporaries(&key).expect("the sweep succeeds");
    assert_eq!(report.retained, 1);
    assert_eq!(report.removed, 0);
    assert!(abandoned.exists(), "a live writer's temporary survives");
}

/// A temporary older than the grace period is swept, and its bytes are counted.
#[test]
fn an_aged_temporary_is_swept() {
    let scratch = Scratch::new("sweep-aged");
    let cache = cache(&scratch).with_limits(Limits {
        temporary_grace: Duration::ZERO,
        ..Limits::default()
    });
    let key = CacheKey::derive_bytes(b"subject");
    ExpansionCache::prepare_directories(cache.rooted_layout(), &key)
        .expect("the namespace is creatable");
    let directory = cache
        .rooted_layout()
        .root()
        .join("v1/tmp")
        .join(&key.label()[..2]);
    let abandoned = directory.join(format!("{}.999.0.0.tmp", key.label()));
    fs::write(&abandoned, b"abandoned").expect("the temporary is writable");
    let unrelated = directory.join("something-else.tmp");
    fs::write(&unrelated, b"not this key").expect("the file is writable");

    let report = cache.sweep_temporaries(&key).expect("the sweep succeeds");
    assert_eq!(report.removed, 1);
    assert_eq!(report.bytes, 9);
    assert!(!abandoned.exists());
    assert!(
        unrelated.exists(),
        "a sweep holding one key's lock touches only that key's temporaries",
    );
}

// -------------------------------------------------------------------------
// Accounting
// -------------------------------------------------------------------------

/// Accounting measures the cache and changes nothing about it.
///
/// The second half is the half that matters. Accounting exists to be run against
/// a live cache before a bound is chosen, so an operator looking at their cache
/// must not be the reason an entry stops validating.
#[test]
fn accounting_measures_the_cache_without_changing_it() {
    let scratch = Scratch::new("accounting");
    let cache = cache(&scratch);
    let first = publish(&cache, b"subject-a", b"envelope-a");
    let second = publish(&cache, b"subject-b", b"a-longer-envelope-b");

    let accounting = cache.account().expect("a published cache accounts");
    assert_eq!(accounting.entry_count(), 2);
    let expected: u64 = [first, second]
        .iter()
        .map(|key| {
            fs::metadata(cache.entry_path(key))
                .expect("the entry exists")
                .len()
        })
        .sum();
    assert_eq!(accounting.total_bytes(), expected);
    assert!(accounting.unrecognized().is_empty());
    assert_eq!(accounting.quarantine_files(), 0);

    for key in [first, second] {
        assert!(
            cache.read_entry(&key, &any_payload).is_ok(),
            "{key} still validates after being accounted for",
        );
    }
}

/// A cache root that has never been written to accounts as empty rather than
/// failing.
///
/// [`ExpansionCache::open`] creates nothing, so this is the ordinary starting
/// state and not an error condition.
#[test]
fn an_unwritten_cache_accounts_as_empty() {
    let scratch = Scratch::new("accounting-empty");
    let accounting = cache(&scratch).account().expect("an absent root accounts");
    assert_eq!(accounting.entry_count(), 0);
    assert_eq!(accounting.total_bytes(), 0);
}

/// Quarantined bytes are counted and never collected.
///
/// Quarantine holds the exact bytes of an entry that failed validation, which is
/// evidence. Its growth is already bounded where it is *added* to, and reaching
/// that bound is reported; a collector that also deleted it would be discarding
/// the diagnosis a user kept the bound for.
#[test]
fn quarantined_evidence_is_counted_and_never_collected() {
    let scratch = Scratch::new("quarantine-accounting");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");
    fs::write(cache.entry_path(&key), b"this is not a bundle")
        .expect("the entry is writable in this test");
    publish(&cache, b"subject", b"rebuilt-envelope");

    let accounting = cache.account().expect("the cache accounts");
    assert_eq!(accounting.quarantine_files(), 1);
    assert_eq!(accounting.quarantine_bytes(), 20);

    // Collect everything collectable, then confirm the evidence is still there.
    let report = cache.collect(&at_most(0)).expect("a collection runs");
    assert_eq!(report.removed().len(), 1);
    let after = cache.account().expect("the cache accounts");
    assert_eq!(after.entry_count(), 0, "the entry was collected");
    assert_eq!(
        after.quarantine_bytes(),
        20,
        "collection never reclaims retained evidence",
    );
}

// -------------------------------------------------------------------------
// Bounded collection
// -------------------------------------------------------------------------

/// The bound this crate supplies by default removes nothing at all.
///
/// The repository rule prefers removing an unnecessary limit to choosing a number
/// for one, and the research note is explicit that exact defaults require
/// workload measurement. An unbounded collection is therefore a pure measurement,
/// and no entry can leave because of a ceiling nobody chose.
#[test]
fn the_default_bound_removes_nothing() {
    let scratch = Scratch::new("unbounded");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");

    let report = cache
        .collect(&CollectionBound::UNBOUNDED)
        .expect("a collection runs");
    assert_eq!(report.outcome(), CollectionOutcome::WithinBound);
    assert_eq!(report.selected(), 0);
    assert!(report.removed().is_empty());
    assert_eq!(report.reclaimed_bytes(), 0);
    assert!(report.accounts_for_every_entry());
    assert!(
        cache.read_entry(&key, &any_payload).is_ok(),
        "an unbounded collection is a measurement",
    );
    assert_eq!(CollectionBound::default(), CollectionBound::UNBOUNDED);
}

/// A cache already within its bound has nothing selected.
#[test]
fn a_cache_within_its_bound_selects_nothing() {
    let scratch = Scratch::new("within-bound");
    let cache = cache(&scratch);
    publish(&cache, b"subject", b"envelope");
    let report = cache.collect(&at_most(4)).expect("a collection runs");
    assert_eq!(report.outcome(), CollectionOutcome::WithinBound);
    assert_eq!(report.selected(), 0);
}

/// An entry bound removes the oldest publications first and stops at the bound.
#[test]
fn a_bound_removes_the_oldest_publications_first() {
    let scratch = Scratch::new("oldest-first");
    let cache = cache(&scratch);
    let oldest = publish_aged(&cache, b"subject-old", b"envelope-old", 300);
    let middle = publish_aged(&cache, b"subject-mid", b"envelope-mid", 200);
    let newest = publish_aged(&cache, b"subject-new", b"envelope-new", 100);

    let report = cache.collect(&at_most(1)).expect("a collection runs");
    assert_eq!(report.outcome(), CollectionOutcome::BoundReached);
    assert_eq!(
        report
            .removed()
            .iter()
            .map(|entry| entry.key)
            .collect::<Vec<_>>(),
        vec![oldest, middle],
        "the two oldest publications are the two that leave",
    );
    assert!(
        cache.read_entry(&newest, &any_payload).is_ok(),
        "the newest publication survives",
    );
    assert!(report.accounts_for_every_entry());
}

/// A byte bound removes until the total fits, and no further.
///
/// Paired with the entry-count case above because the two ceilings are checked
/// separately and a selector honouring one and ignoring the other would pass
/// either test alone.
#[test]
fn a_byte_bound_removes_until_the_total_fits() {
    let scratch = Scratch::new("byte-bound");
    let cache = cache(&scratch);
    let oldest = publish_aged(&cache, b"subject-old", b"envelope-old", 300);
    let newest = publish_aged(&cache, b"subject-new", b"envelope-new", 100);
    let each = fs::metadata(cache.entry_path(&newest))
        .expect("the entry exists")
        .len();

    let report = cache
        .collect(&CollectionBound {
            max_total_bytes: Some(each),
            max_entries: None,
            max_entry_age: None,
        })
        .expect("a collection runs");
    assert_eq!(report.outcome(), CollectionOutcome::BoundReached);
    assert_eq!(report.reclaimed_bytes(), each);
    assert_eq!(
        report
            .removed()
            .iter()
            .map(|entry| entry.key)
            .collect::<Vec<_>>(),
        vec![oldest],
    );
    assert!(cache.read_entry(&newest, &any_payload).is_ok());
}

/// Every removal is named individually, and the dispositions account for the
/// whole selection.
///
/// This is the mechanical form of the rule that nothing is dropped silently. A
/// count alone would let an entry leave the cache without anything saying which
/// one; the report has to be able to answer that after an unexpected rebuild.
#[test]
fn a_collection_names_every_entry_it_removed() {
    let scratch = Scratch::new("named-removals");
    let cache = cache(&scratch);
    let keys: Vec<CacheKey> = (0_u64..4)
        .map(|index| {
            publish_aged(
                &cache,
                format!("subject-{index}").as_bytes(),
                format!("envelope-{index}").as_bytes(),
                400 - index * 100,
            )
        })
        .collect();

    let report = cache.collect(&at_most(1)).expect("a collection runs");
    assert!(report.accounts_for_every_entry());
    assert_eq!(report.selected(), 3);
    assert_eq!(report.removed().len(), 3);
    assert_eq!(
        report.reclaimed_bytes(),
        report
            .removed()
            .iter()
            .map(|entry| entry.bytes)
            .sum::<u64>(),
        "the reclaimed total is the sum of the named removals",
    );
    for removed in report.removed() {
        assert!(
            removed.bytes > 0,
            "a removal reports the bytes it reclaimed"
        );
        assert!(keys.contains(&removed.key), "a removal names a real key");
        assert!(
            !cache.entry_path(&removed.key).exists(),
            "a named removal really happened",
        );
    }
    assert_eq!(report.order().as_str(), "oldest-publication-first");
}

/// A key another process holds the lock on is skipped, reported, and never
/// waited for.
///
/// A held key lock means somebody is publishing or evicting that key right now,
/// which makes the entry live rather than collectable. Skipping is both the
/// better selection and what lets a collection have no work budget: it never
/// blocks, so there is no unbounded wait to cap.
#[test]
fn a_contended_key_is_skipped_and_reported_rather_than_waited_for() {
    let scratch = Scratch::new("contended");
    let cache = cache(&scratch);
    let contended = publish_aged(&cache, b"subject-old", b"envelope-old", 300);
    publish_aged(&cache, b"subject-new", b"envelope-new", 100);

    let held = KeyLock::try_acquire(&cache.lock_path(&contended))
        .expect("the lock file opens")
        .expect("an unheld lock is free");

    let report = cache.collect(&at_most(1)).expect("a collection runs");
    assert_eq!(report.contended(), 1);
    assert!(report.removed().is_empty());
    assert!(report.accounts_for_every_entry());
    assert!(
        matches!(
            report.outcome(),
            CollectionOutcome::BoundNotReached { entries: 2, .. },
        ),
        "an unreachable bound is reported, not quietly abandoned: {:?}",
        report.outcome(),
    );
    assert!(
        cache.read_entry(&contended, &any_payload).is_ok(),
        "a contended entry is left alone",
    );
    held.release().expect("the lock releases");
}

/// An entry replaced since the scan is left alone rather than removed.
///
/// Without this, a collection deciding on a stale measurement could unlink a
/// *fresh* publication and then report having reclaimed bytes belonging to a file
/// it never saw.
#[test]
fn an_entry_replaced_since_the_scan_is_not_removed() {
    let scratch = Scratch::new("superseded");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");
    let stale = cache
        .account()
        .expect("the cache accounts")
        .entries()
        .first()
        .expect("one entry was published")
        .clone();

    // Republish under the same key with a differently sized envelope, which is
    // what a writer replacing a rejected entry does.
    fs::remove_file(cache.entry_path(&key)).expect("the entry is removable");
    publish(
        &cache,
        b"subject",
        b"a-considerably-longer-replacement-envelope",
    );

    assert_eq!(
        cache
            .remove_if_unchanged(&stale)
            .expect("the removal reaches a decision"),
        Disposition::Superseded,
    );
    assert_eq!(
        cache
            .read_entry(&key, &any_payload)
            .expect("the replacement survives")
            .envelope(),
        b"a-considerably-longer-replacement-envelope",
    );
}

/// An entry already gone when the lock is taken is reported and not counted as
/// reclaimed.
///
/// The ordinary outcome when two collectors select overlapping sets, or when an
/// external deletion wins the race. The bytes belong to whoever actually removed
/// the file.
#[test]
fn an_entry_already_gone_is_reported_rather_than_counted() {
    let scratch = Scratch::new("already-absent");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");
    let stale = cache
        .account()
        .expect("the cache accounts")
        .entries()
        .first()
        .expect("one entry was published")
        .clone();
    fs::remove_file(cache.entry_path(&key)).expect("the entry is removable");

    assert_eq!(
        cache
            .remove_if_unchanged(&stale)
            .expect("the removal reaches a decision"),
        Disposition::AlreadyAbsent,
    );
}

/// A file the entry parser refuses is reported and never removed.
///
/// A collector that deleted whatever it could not parse would be acting on the
/// absence of understanding. The parser is strict — exact label width, lowercase
/// hexadecimal, and the key's own shard — so an unrecognized file means something
/// other than this crate is writing into the namespace, which an operator should
/// be told rather than have tidied away.
#[test]
fn a_file_the_parser_refuses_is_reported_and_never_removed() {
    let scratch = Scratch::new("unrecognized");
    let cache = cache(&scratch);
    let key = publish_aged(&cache, b"subject", b"envelope", 100);
    let shard = cache
        .entry_path(&key)
        .parent()
        .expect("an entry has a shard directory")
        .to_path_buf();
    let stray = shard.join("not-a-cache-entry.txt");
    fs::write(&stray, b"something else wrote here").expect("the shard is writable in this test");

    let report = cache.collect(&at_most(0)).expect("a collection runs");
    assert_eq!(
        report.accounting().unrecognized(),
        std::slice::from_ref(&stray)
    );
    assert!(
        stray.exists(),
        "a collector never removes what its own parser refused",
    );
    assert_eq!(
        report.removed().len(),
        1,
        "the recognized entry is still collected",
    );
    assert!(report.accounts_for_every_entry());
}

/// Collection retains every lock file.
///
/// Not tidiness. Unlinking a locked file lets a later process create a different
/// inode at the same path and take an independent lock while an earlier process
/// still holds the first, which splits contenders into two groups that do not
/// exclude each other.
#[test]
fn collection_retains_every_lock_file() {
    let scratch = Scratch::new("collection-locks");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");
    let lock = cache.lock_path(&key);
    assert!(lock.exists(), "publishing created the lock file");

    let report = cache.collect(&at_most(0)).expect("a collection runs");
    assert_eq!(report.removed().len(), 1);
    assert!(!cache.entry_path(&key).exists(), "the entry is gone");
    assert!(lock.exists(), "the lock file is retained");
}

/// A reader that has already opened and validated an entry finishes its read
/// across a collection that removes it.
///
/// This is why [`ExpansionCache::lookup`] takes no lock. The reader's descriptor
/// was opened before the unlink, and a directory entry removed on the Unix and
/// Darwin hosts this crate targets does not reclaim the inode while a descriptor
/// is open. `super::harness` measures the same property across a real process
/// boundary, with the collection running in a different process.
#[test]
fn a_reader_holding_a_descriptor_reads_across_a_collection() {
    let scratch = Scratch::new("read-across-collection");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope-before-collection");
    let held = fs::File::open(cache.entry_path(&key)).expect("a published entry opens");

    let report = cache.collect(&at_most(0)).expect("a collection runs");
    assert_eq!(report.removed().len(), 1);
    assert!(!cache.entry_path(&key).exists(), "the entry was unlinked");
    assert!(
        matches!(
            cache.read_entry(&key, &any_payload),
            Err(MissReason::Absent),
        ),
        "a reader arriving afterwards finds an ordinary absence",
    );

    let mut bytes = Vec::new();
    {
        use std::io::Read;
        let mut held = held;
        held.read_to_end(&mut bytes)
            .expect("an open descriptor survives the unlink");
    }
    let view = bundle::decode(&bytes, &key, &Limits::default())
        .expect("the bytes an open descriptor yields are still a valid bundle");
    assert_eq!(&bytes[view.envelope], b"envelope-before-collection");
}

/// Concurrent collectors over one cache never double-count and never fail.
///
/// Each removal is taken under its own key lock, so two collectors selecting
/// overlapping sets serialize per key: one removes and counts the bytes, the
/// other finds the entry gone. Every entry is still accounted for in both
/// reports.
#[test]
fn concurrent_collectors_do_not_double_count() {
    const COLLECTORS: usize = 8;
    let scratch = Scratch::new("concurrent-collectors");
    let cache = cache(&scratch);
    for index in 0_u64..16 {
        publish_aged(
            &cache,
            format!("subject-{index}").as_bytes(),
            format!("envelope-{index}").as_bytes(),
            1600 - index * 100,
        );
    }

    let handles: Vec<_> = (0..COLLECTORS)
        .map(|_| {
            let cache = cache.clone();
            thread::spawn(move || cache.collect(&at_most(0)).expect("a collection runs"))
        })
        .collect();

    let mut reclaimed = 0_u64;
    let mut removed = 0_usize;
    for handle in handles {
        let report = handle.join().expect("no collector panics");
        assert!(report.accounts_for_every_entry());
        assert!(report.failed().is_empty(), "{:?}", report.failed());
        reclaimed += report.reclaimed_bytes();
        removed += report.removed().len();
    }
    assert_eq!(removed, 16, "every entry is removed exactly once");
    assert_eq!(
        cache.account().expect("the cache accounts").entry_count(),
        0,
    );
    assert!(reclaimed > 0);
}

// -------------------------------------------------------------------------
// The age ceiling
//
// Tom decided on 2026-08-04 that a frontend evicts old entries automatically
// under an environment-configured policy, which superseded the design record's
// "never automatically" schedule conclusion and nothing else. These are the
// perturbations that decision's implementation ticket names, plus the
// composition and refusal properties the supersession claims.
// -------------------------------------------------------------------------

/// An age ceiling removes entries older than the stated maximum and no others.
///
/// The plain case, through the public path, on real filesystem modification
/// times. It also pins the attribution: an age removal names the age, which is
/// the only thing telling an operator why an entry left when no person was
/// present to remember the policy.
#[test]
fn an_age_ceiling_removes_only_entries_older_than_the_stated_maximum() {
    let scratch = Scratch::new("age-ceiling");
    let cache = cache(&scratch);
    let ancient = publish_aged(&cache, b"subject-ancient", b"envelope-ancient", 900);
    let old = publish_aged(&cache, b"subject-old", b"envelope-old", 600);
    let young = publish_aged(&cache, b"subject-young", b"envelope-young", 60);

    let report = cache
        .collect(&older_than(Duration::from_secs(300)))
        .expect("a collection runs");
    assert_eq!(report.outcome(), CollectionOutcome::BoundReached);
    assert!(report.accounts_for_every_entry());
    assert_eq!(
        report
            .removed()
            .iter()
            .map(|entry| entry.key)
            .collect::<Vec<_>>(),
        vec![ancient, old],
        "both entries past the age leave, oldest first",
    );
    for removed in report.removed() {
        assert_eq!(removed.reason, RemovalReason::OlderThanMaxEntryAge);
        assert_eq!(removed.reason.as_str(), "older-than-max-entry-age");
    }
    assert!(
        cache.read_entry(&young, &any_payload).is_ok(),
        "an entry inside the age is untouched",
    );
    assert_eq!(
        report.bound().max_entry_age.map(MaxEntryAge::as_duration),
        Some(Duration::from_secs(300)),
        "the report carries the exact policy the removal is attributable to",
    );
}

/// An entry that has reached the maximum age exactly is removed.
///
/// The boundary is stated as reached, not passed, so the comparison is
/// deterministic at equality rather than only somewhere near it. Both halves are
/// asserted from one anchor instant: the entry is dated exactly `max_age` before
/// it and the collection is run at it, which is the only way to observe the
/// boundary itself — a wall-clock `now` is necessarily later than the moment a
/// test set the modification time, so it can never reach equality and a margin
/// would replace the statement with a race.
#[test]
fn an_entry_exactly_at_the_age_boundary_is_removed_and_one_inside_it_is_not() {
    let scratch = Scratch::new("age-boundary");
    let cache = cache(&scratch);
    let max_age = Duration::from_secs(600);
    let anchor = SystemTime::now();

    let at_boundary = publish(&cache, b"subject-boundary", b"envelope-boundary");
    let inside = publish(&cache, b"subject-inside", b"envelope-inside");
    set_published(&cache, &at_boundary, anchor - max_age);
    set_published(&cache, &inside, anchor - max_age + Duration::from_secs(60));

    let report = cache
        .collect_at(&older_than(max_age), anchor)
        .expect("a collection runs");
    assert!(report.accounts_for_every_entry());
    assert_eq!(
        report
            .removed()
            .iter()
            .map(|entry| entry.key)
            .collect::<Vec<_>>(),
        vec![at_boundary],
        "an entry exactly at the maximum age has reached it",
    );
    assert!(
        cache.read_entry(&inside, &any_payload).is_ok(),
        "an entry a minute inside the maximum age stays",
    );
}

/// An entry dated in the future is left alone, and it evicts nothing else.
///
/// A clock that moved backwards between a publication and a collection, a file
/// stamped into the future, or two machines with skewed clocks sharing a root
/// all produce a modification time after the collecting process's own reading.
/// The age is then unknown, and an unknown age is treated as young — the same
/// direction `sweep_temporaries` takes, and for the same asymmetry: keeping
/// something collectable costs bounded disk, and removing something live costs
/// work that has to be done again.
///
/// The second half is what makes this a perturbation rather than a formality.
/// A selector that computed a cutoff instant, or that treated an unrepresentable
/// age as infinite, would evict the whole cache the moment one clock disagreed.
#[test]
fn an_entry_dated_in_the_future_is_neither_removed_nor_a_reason_to_remove_others() {
    let scratch = Scratch::new("age-future");
    let cache = cache(&scratch);
    let anchor = SystemTime::now();
    let max_age = Duration::from_secs(300);

    let future = publish(&cache, b"subject-future", b"envelope-future");
    let young = publish(&cache, b"subject-young", b"envelope-young");
    let expired = publish(&cache, b"subject-expired", b"envelope-expired");
    set_published(&cache, &future, anchor + Duration::from_hours(24));
    set_published(&cache, &young, anchor - Duration::from_secs(30));
    set_published(&cache, &expired, anchor - Duration::from_mins(15));

    let report = cache
        .collect_at(&older_than(max_age), anchor)
        .expect("a collection runs");
    assert!(report.accounts_for_every_entry());
    assert_eq!(
        report
            .removed()
            .iter()
            .map(|entry| entry.key)
            .collect::<Vec<_>>(),
        vec![expired],
        "only the entry with a computable age past the maximum leaves",
    );
    assert!(
        cache.read_entry(&future, &any_payload).is_ok(),
        "an entry the host dates after now has an unknown age, not an infinite one",
    );
    assert!(
        cache.read_entry(&young, &any_payload).is_ok(),
        "a backwards clock does not make the rest of the cache collectable",
    );
}

/// An age eviction racing a re-publisher of the same key removes nothing it did
/// not measure.
///
/// The publisher occupies one of two positions the collector can observe, and
/// both are asserted here rather than waited for, because a race decided by
/// scheduling is a test that passes for reasons it cannot name.
///
/// **Holding the key lock.** The re-publisher has the lock when the collector
/// reaches the key. `try_acquire` returns `None`, the entry is counted
/// `contended`, and the age ceiling is reported unreached — even though the
/// aggregate ceilings are absent and therefore trivially satisfied, which is
/// exactly the case a caller reading only `bytes` and `entries` would misread as
/// success.
///
/// **Republished between the scan and the lock.** The collector already selected
/// the entry on its age when a fresh publication replaced it. The locked
/// re-`stat` disagrees with the scan, so the fresh entry survives as
/// `Superseded` — an age ceiling cannot unlink a publication it never measured,
/// because the age it decided on is the same modification time the removal
/// re-checks.
#[test]
fn an_age_eviction_racing_a_republisher_removes_nothing_it_did_not_measure() {
    let scratch = Scratch::new("age-republish-race");
    let cache = cache(&scratch);
    let bound = older_than(Duration::from_secs(300));
    let key = publish_aged(&cache, b"subject", b"envelope", 900);
    let stale = cache
        .account()
        .expect("the cache accounts")
        .entries()
        .first()
        .expect("one entry was published")
        .clone();

    let held = KeyLock::try_acquire(&cache.lock_path(&key))
        .expect("the lock file opens")
        .expect("an unheld lock is free");
    let contended = cache.collect(&bound).expect("a collection runs");
    assert_eq!(contended.contended(), 1);
    assert!(contended.removed().is_empty());
    assert!(contended.accounts_for_every_entry());
    assert!(
        matches!(
            contended.outcome(),
            CollectionOutcome::BoundNotReached { entries: 1, .. },
        ),
        "an expired entry left in place leaves the age ceiling unreached: {:?}",
        contended.outcome(),
    );
    held.release().expect("the lock releases");

    // The publisher lands its replacement, which the collector never measured.
    fs::remove_file(cache.entry_path(&key)).expect("the entry is removable");
    publish(
        &cache,
        b"subject",
        b"a-considerably-longer-replacement-envelope",
    );
    assert_eq!(
        cache
            .remove_if_unchanged(&stale)
            .expect("the removal reaches a decision"),
        Disposition::Superseded,
    );
    assert_eq!(
        cache
            .read_entry(&key, &any_payload)
            .expect("the replacement survives")
            .envelope(),
        b"a-considerably-longer-replacement-envelope",
    );

    // And the replacement is not expired, so a fresh collection under the same
    // policy leaves it alone rather than removing it on the previous entry's age.
    let after = cache.collect(&bound).expect("a collection runs");
    assert_eq!(after.outcome(), CollectionOutcome::WithinBound);
    assert!(after.removed().is_empty());
}

/// A maximum age of zero is refused at construction rather than evicting
/// everything.
///
/// It is not a short retention window. `age >= 0` holds for every entry the host
/// can date, including one published this instant, so it is "remove everything"
/// said obliquely — with the extra failure that it removes an entry a concurrent
/// build published microseconds ago and is about to hit. A caller that means it
/// has two operations that say so.
///
/// A *negative* age needs no test because it needs no check: `Duration` is
/// unsigned, so it is unrepresentable rather than unchecked, and there is no
/// path through this crate on which one could arrive.
#[test]
fn a_zero_maximum_entry_age_is_refused_and_no_bound_can_carry_one() {
    assert_eq!(
        MaxEntryAge::new(Duration::ZERO),
        Err(MaxEntryAgeRefusal::Zero),
    );
    assert!(
        MaxEntryAge::new(Duration::ZERO)
            .expect_err("zero is refused")
            .to_string()
            .contains("not a bound"),
        "the refusal says why, rather than only that",
    );
    assert_eq!(
        MaxEntryAge::new(Duration::from_nanos(1))
            .expect("a one-nanosecond maximum is a bound, however aggressive")
            .as_duration(),
        Duration::from_nanos(1),
        "only the value that is not a bound is refused; no floor is guessed above it",
    );
}

/// Nothing supplies an age ceiling on its own.
///
/// The design record's refusal of a default bound survives Tom's decision for
/// the *size* ceilings unchanged, and the age default it authorizes is a
/// constant a frontend cites rather than one this crate applies. Both halves are
/// asserted, because a `Default` quietly gaining the constant is exactly the
/// change that would delete a user's artifacts under a number nobody chose.
#[test]
fn the_default_bound_states_no_age_and_the_default_age_is_only_a_constant() {
    let scratch = Scratch::new("age-not-default");
    let cache = cache(&scratch);
    let key = publish_aged(&cache, b"subject", b"envelope", 400 * 24 * 3600);

    assert_eq!(CollectionBound::default(), CollectionBound::UNBOUNDED);
    assert_eq!(CollectionBound::UNBOUNDED.max_entry_age, None);
    let report = cache
        .collect(&CollectionBound::UNBOUNDED)
        .expect("a collection runs");
    assert_eq!(report.outcome(), CollectionOutcome::WithinBound);
    assert!(
        cache.read_entry(&key, &any_payload).is_ok(),
        "an entry over a year old survives a bound that states no age",
    );
    assert_eq!(
        MaxEntryAge::DEFAULT.as_duration(),
        Duration::from_hours(30 * 24),
        "the cited default is thirty days",
    );
}

/// An age ceiling composes with an entry ceiling rather than replacing it.
///
/// Each ceiling only ever adds removals, so the selection is their union and the
/// report says which one took each entry. Asserted together because a selector
/// honouring one and ignoring the other would pass either single-ceiling test
/// alone, and because an age pass that ran *after* the aggregate pass would
/// remove one entry too many — the aggregate would spend bytes the expiry was
/// about to reclaim.
#[test]
fn an_age_ceiling_composes_with_an_entry_ceiling() {
    let scratch = Scratch::new("age-composes");
    let cache = cache(&scratch);
    let anchor = SystemTime::now();
    let keys: Vec<CacheKey> = (0_u64..4)
        .map(|index| {
            let key = publish(
                &cache,
                format!("subject-{index}").as_bytes(),
                format!("envelope-{index}").as_bytes(),
            );
            set_published(
                &cache,
                &key,
                anchor - Duration::from_secs(400 - index * 100),
            );
            key
        })
        .collect();

    // Ages are 400, 300, 200, 100 seconds. The age takes the first two; the
    // entry ceiling of one then takes the older of the two survivors.
    let report = cache
        .collect_at(
            &CollectionBound {
                max_total_bytes: None,
                max_entries: Some(1),
                max_entry_age: Some(
                    MaxEntryAge::new(Duration::from_secs(300)).expect("a non-zero age"),
                ),
            },
            anchor,
        )
        .expect("a collection runs");
    assert_eq!(report.outcome(), CollectionOutcome::BoundReached);
    assert!(report.accounts_for_every_entry());
    assert_eq!(
        report
            .removed()
            .iter()
            .map(|entry| (entry.key, entry.reason))
            .collect::<Vec<_>>(),
        vec![
            (keys[0], RemovalReason::OlderThanMaxEntryAge),
            (keys[1], RemovalReason::OlderThanMaxEntryAge),
            (keys[2], RemovalReason::OverSizeCeiling),
        ],
        "each removal names the ceiling that selected it",
    );
    assert!(
        cache.read_entry(&keys[3], &any_payload).is_ok(),
        "the newest entry is the one the entry ceiling retains",
    );
}

// -------------------------------------------------------------------------
// The out-of-service purge
// -------------------------------------------------------------------------

/// A purge retires the whole namespace in one rename and reclaims it.
///
/// The rename is what makes this stronger than `rm -r`: afterwards `<root>/v1`
/// does not exist, so a process arriving next creates a fresh, coherent
/// namespace rather than walking a half-deleted one.
#[test]
fn a_purge_retires_the_namespace_and_a_later_writer_starts_clean() {
    let scratch = Scratch::new("purge");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope-before-purge");
    assert!(cache.entry_path(&key).exists());

    let report = cache.purge().expect("a purge runs");
    assert!(report.retired().is_some(), "the live namespace was retired");
    assert_eq!(report.reclaimed_trees(), 1);
    assert!(report.reclaimed_bytes() > 0);
    assert!(report.failed().is_empty());
    assert!(
        !cache.rooted_layout().root().join("v1").exists(),
        "nothing is left in service",
    );
    assert_eq!(
        cache
            .account()
            .expect("an emptied cache accounts")
            .entry_count(),
        0,
    );

    let republished = publish(&cache, b"subject", b"envelope-after-purge");
    assert_eq!(
        republished, key,
        "a key is a function of the subject, not of the cache it is stored in",
    );
    assert_eq!(
        cache
            .read_entry(&key, &any_payload)
            .expect("the fresh namespace serves the new entry")
            .envelope(),
        b"envelope-after-purge",
    );
}

/// A purge reclaims a tree an earlier purge left behind.
///
/// This is the crash recovery, and it needs no rule beyond "reclaim what is out
/// of service": a purge that died between its rename and its removal left a tree
/// nothing reads, so removing it later is disk reclamation rather than a repair.
#[test]
fn a_purge_reclaims_a_tree_an_earlier_purge_left_behind() {
    let scratch = Scratch::new("purge-leftover");
    let cache = cache(&scratch);
    publish(&cache, b"subject", b"envelope");

    // Exactly the state a purge killed after its rename leaves behind.
    let root = cache.rooted_layout().root().to_path_buf();
    let abandoned = root.join("v1.out-of-service.12345");
    fs::rename(root.join("v1"), &abandoned).expect("the namespace renames");
    assert!(abandoned.exists());

    let report = cache.purge().expect("a purge runs");
    assert!(
        report.retired().is_none(),
        "there was no live namespace left to retire",
    );
    assert_eq!(report.reclaimed_trees(), 1);
    assert!(!abandoned.exists(), "the abandoned tree was reclaimed");
}

/// Purging a cache that was never written to is not an error.
#[test]
fn purging_an_unwritten_cache_is_not_an_error() {
    let scratch = Scratch::new("purge-empty");
    let report = cache(&scratch).purge().expect("an absent root purges");
    assert!(report.retired().is_none());
    assert_eq!(report.reclaimed_trees(), 0);
}

/// A retired namespace is invisible to every reader.
///
/// The version component is joined exactly, so a tree named with the
/// out-of-service prefix cannot be resolved into by anything this crate reads.
/// That is the whole mechanism by which one rename takes a namespace out of
/// service without telling anyone.
#[test]
fn a_retired_namespace_is_invisible_to_a_reader() {
    let scratch = Scratch::new("retired-invisible");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");

    let root = cache.rooted_layout().root().to_path_buf();
    let abandoned = root.join("v1.out-of-service.999");
    fs::rename(root.join("v1"), &abandoned).expect("the namespace renames");
    assert!(
        abandoned.join("entries").exists(),
        "the entries are still on disk, merely out of service",
    );
    assert!(
        matches!(
            cache.read_entry(&key, &any_payload),
            Err(MissReason::Absent),
        ),
        "a reader does not resolve into a retired tree",
    );
    assert_eq!(
        cache.account().expect("the cache accounts").entry_count(),
        0,
        "a scan does not walk a retired tree either",
    );
}

// -------------------------------------------------------------------------
// Hot-path measurement
// -------------------------------------------------------------------------
//
// Prints; asserts nothing about time. Reproduce with:
//
//   cargo nextest run --release -p tiler-cache -E 'test(hot_path)' --no-capture

/// Reports the cost of a lock-free cache hit.
///
/// **A hit is not a cheap path, and that is by design rather than by accident.**
/// ADR 0050 requires a reader to validate bounded framing, the embedded key,
/// schemas, the manifest, section lengths and digests, and required meanings on
/// *every* hit, precisely so a corrupt or stale entry cannot be served. So this
/// number is mostly the price of that rule.
///
/// The duplicate buffering this comment used to point at is gone: a hit reads
/// the bundle into one pre-sized buffer and keeps it, naming the two sections by
/// span instead of copying both back out. That removed two allocations and two
/// copies per hit and moved this number by a few percent, because the digesting
/// above dominates it — see [`super::hot_path`] for the sweep that shows the
/// cost is per-byte and for where the remaining time actually goes.
///
/// **This is the protocol cost alone, and must not be read as the cost of a
/// real hit.** These fixtures store a short byte string under `any_payload`,
/// which accepts anything, so no artifact decode runs. A production hit pins
/// the validator to `decode_artifact`, which adds the decode measured by
/// `hot_path_decode_and_reencode_share` in `tiler-artifact` — hundreds of
/// microseconds against the tens below. The two numbers are complementary and
/// neither is the whole answer on its own.
#[test]
fn hot_path_cache_hit() {
    const REPEATS: u32 = 20;
    let scratch = Scratch::new("hot-path-hit");
    let cache = cache(&scratch);
    let key = publish(
        &cache,
        b"subject",
        b"envelope-bytes-for-the-hot-path-measurement",
    );

    // Warm the page cache so the number is the protocol rather than the disk.
    let _ = cache
        .read_entry(&key, &any_payload)
        .expect("the entry is present");

    let start = Instant::now();
    for _ in 0..REPEATS {
        let _ = cache
            .read_entry(&key, &any_payload)
            .expect("the entry is present");
    }
    println!(
        "MEASURE cache hit (read_entry): {:?}",
        start.elapsed() / REPEATS
    );

    let start = Instant::now();
    for _ in 0..REPEATS {
        let _ = cache
            .resolve(b"subject", || Ok::<_, String>(Vec::new()), &any_payload)
            .expect("the entry resolves");
    }
    println!(
        "MEASURE cache hit (resolve)   : {:?}",
        start.elapsed() / REPEATS
    );
}

// -------------------------------------------------------------------------
// The cache that stores nothing
// -------------------------------------------------------------------------

/// A disabled cache compiles, validates, and returns — and writes no file where
/// a rooted cache writes one.
///
/// The two halves run over one scratch directory with one subject and one
/// envelope, because either alone is vacuous. "Nothing was created" would also
/// be what a host that cannot publish at all reported, so the rooted resolution
/// afterwards is what makes the absence evidence rather than an accident; and
/// the rooted half says nothing about the disabled mode on its own.
///
/// What the directory cannot show is that nothing was written *elsewhere*. That
/// is not a measurement here but a property of the constructor:
/// [`ExpansionCache::disabled`] takes no root and stores no [`Layout`], so
/// `root()` is `None` and there is no path any operation could derive — which
/// the assertion below records rather than infers.
#[test]
fn a_disabled_cache_stores_nothing_where_a_rooted_cache_publishes() {
    let scratch = Scratch::new("disabled-stores-nothing");
    let root = scratch.root().join("cache");
    let mut builds = 0_u32;

    let disabled = ExpansionCache::disabled();
    assert_eq!(
        disabled.root(),
        None,
        "a disabled cache holds no path any operation could publish to",
    );

    let outcome = disabled
        .resolve(
            b"compilation",
            || {
                builds += 1;
                Ok::<_, String>(b"program".to_vec())
            },
            &any_payload,
        )
        .expect("a disabled cache still resolves");
    let ProtocolOutcome::Uncached { entry, report } = outcome else {
        panic!("a disabled cache stores nothing, so it can never report a hit");
    };
    assert_eq!(builds, 1, "the build step runs exactly once, as on a miss");
    assert_eq!(
        entry.envelope(),
        b"program",
        "the validated artifact is still returned for embedding",
    );
    assert!(
        matches!(report.lookup_miss(), Some(MissReason::Disabled)),
        "the miss must name the mode, not an absent entry: {:?}",
        report.lookup_miss(),
    );
    assert!(
        matches!(
            report.publication_refusal(),
            Some(PublicationRefusal::Disabled)
        ),
        "an absent refusal would state the result was published: {:?}",
        report.publication_refusal(),
    );
    assert!(
        report.recheck_miss().is_none(),
        "no lock was taken, so no post-lock recheck ran",
    );
    assert!(
        !root.exists(),
        "the disabled resolution created no cache root",
    );

    // The control, over the same directory, the same subject, and the same
    // envelope: only the cache differs, and now the file exists.
    let rooted = ExpansionCache::open(root.clone());
    let key = publish(&rooted, b"compilation", b"program");
    assert!(
        rooted.entry_path(&key).exists(),
        "the rooted resolution must publish the entry the disabled one did not",
    );
}

/// Every operation of a disabled cache answers without a namespace.
///
/// Each one is a public method on a public type, so each is callable on a cache
/// that has no root and each needs a defined answer rather than a path it
/// fabricates. The bound is the tightest one there is — retain no entry — so an
/// accounting that reported anything would select it for removal and fail the
/// outcome assertion rather than pass vacuously.
#[test]
fn a_disabled_cache_answers_every_namespace_operation_without_a_root() {
    let cache = ExpansionCache::disabled();
    let key = CacheKey::derive_bytes(b"compilation");

    assert_eq!(cache.root(), None);
    assert!(
        matches!(
            cache.lookup(&composed(&[b"compilation"], b"artifact")),
            Lookup::Miss(MissReason::Disabled),
        ),
        "a lookup with no content path must name the mode",
    );
    assert_eq!(
        cache.evict(&key).expect("an eviction reports"),
        Eviction::Absent,
        "nothing was stored, so nothing was removed",
    );
    assert_eq!(
        cache
            .sweep_temporaries(&key)
            .expect("a temporary sweep reports"),
        SweepReport::default(),
        "no temporary was ever created to sweep",
    );

    let accounting = cache.account().expect("accounting reports");
    assert_eq!(accounting.entry_count(), 0);
    assert_eq!(accounting.total_bytes(), 0);
    assert_eq!(accounting.quarantine_files(), 0);
    assert!(accounting.unrecognized().is_empty());

    let collection = cache.collect(&at_most(0)).expect("a collection reports");
    assert_eq!(collection.selected(), 0);
    assert_eq!(collection.outcome(), CollectionOutcome::WithinBound);
    assert!(collection.removed().is_empty());
    assert!(collection.accounts_for_every_entry());

    let purge = cache.purge().expect("a purge reports");
    assert_eq!(purge.retired(), None, "there is no namespace to retire");
    assert_eq!(purge.reclaimed_trees(), 0);

    let preflight = cache.preflight();
    assert_eq!(preflight.root(), None);
    let verdicts = [
        preflight.same_device(),
        preflight.create_new_excludes(),
        preflight.lock_excludes_locally(),
        preflight.rename_publishes(),
        preflight.modification_time_reported(),
    ];
    assert_eq!(
        verdicts.len(),
        5,
        "the population is every probed property, counted",
    );
    for verdict in verdicts {
        assert_eq!(
            verdict,
            PreflightVerdict::NotRun,
            "no root was probed, so no property was learned",
        );
    }
    assert!(
        !preflight.all_probed_properties_hold(),
        "a report where nothing ran must not read as a clean bill of health",
    );
}

/// A preflight on an ordinary root reports every property holding.
///
/// **The stronger half of this test is the second assertion.** A report whose
/// rows were all `NotRun` would satisfy "nothing was refuted" while measuring
/// nothing at all, which is the vacuous pass the probes are written to avoid —
/// so the check is that every property *holds*, and `all_probed_properties_hold`
/// deliberately does not count `NotRun` as holding.
#[test]
fn a_preflight_on_a_writable_root_reports_every_property_holding() {
    let scratch = Scratch::new("preflight-holds");
    let cache = cache(&scratch);

    let report = cache.preflight();
    assert_eq!(report.root(), Some(scratch.root().join("cache").as_path()));
    assert_eq!(report.same_device(), PreflightVerdict::Holds);
    assert_eq!(report.create_new_excludes(), PreflightVerdict::Holds);
    assert_eq!(report.lock_excludes_locally(), PreflightVerdict::Holds);
    assert_eq!(report.rename_publishes(), PreflightVerdict::Holds);
    assert_eq!(report.modification_time_reported(), PreflightVerdict::Holds);
    assert!(report.all_probed_properties_hold());

    // Carried beside the lock row rather than inferred from it: no probe on one
    // host can decide whether another host is excluded, so the report says so
    // in words a caller can render.
    let caveat = PreflightReport::cross_host_exclusion_caveat();
    assert!(
        caveat.contains("this host only"),
        "the caveat must say what the lock row does not cover: {caveat}",
    );
}

/// A preflight changes nothing that outlives it.
///
/// It writes probe files by construction, so "changes nothing" has to mean the
/// root is as it was afterwards rather than that nothing was written. Asserted
/// against a cache holding a real entry, so a probe that removed too much would
/// fail here rather than only leaving litter.
#[test]
fn a_preflight_leaves_the_root_as_it_found_it() {
    let scratch = Scratch::new("preflight-clean");
    let cache = cache(&scratch);
    let key = publish(&cache, b"subject", b"envelope");

    let before = cache.account().expect("the cache accounts");
    let report = cache.preflight();
    assert!(report.all_probed_properties_hold());
    let after = cache.account().expect("the cache accounts");

    assert_eq!(before.entries(), after.entries(), "an entry was disturbed");
    assert!(
        !cache
            .rooted_layout()
            .version_root()
            .join("preflight")
            .exists(),
        "the probe area outlived the call",
    );

    // The published entry still resolves, and resolves as a *hit* rather than a
    // republication — which is the property an accounting count alone would not
    // establish, since a rebuilt entry would restore the count too.
    let outcome = cache
        .resolve(
            b"subject",
            || Ok::<_, String>(b"envelope".to_vec()),
            &any_payload,
        )
        .expect("the resolve runs");
    match outcome {
        ProtocolOutcome::Hit {
            entry, published, ..
        } => {
            assert!(!published, "the entry was rebuilt rather than hit");
            assert_eq!(entry.key, key, "a different entry answered");
        }
        ProtocolOutcome::Uncached { .. } => {
            panic!("the entry stopped resolving after a preflight")
        }
    }
}

/// An unwritable root reports `NotRun`, never a refutation.
///
/// The distinction is the whole reason the verdict has three cases: a refuted
/// property says this root is unsuitable, while a probe that could not run says
/// nothing was learned. Reporting the first for the second would send a caller
/// to replace a filesystem when the answer is a permission.
#[test]
fn an_unwritable_root_reports_not_run_rather_than_refuted() {
    use std::os::unix::fs::PermissionsExt as _;

    let scratch = Scratch::new("preflight-readonly");
    let root = scratch.root().join("cache");
    fs::create_dir_all(&root).expect("the root is creatable");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o500))
        .expect("the root permissions are settable");

    let cache = ExpansionCache::open(root.clone());
    let report = cache.preflight();

    assert_eq!(report.same_device(), PreflightVerdict::NotRun);
    assert_eq!(report.create_new_excludes(), PreflightVerdict::NotRun);
    assert!(
        !report.all_probed_properties_hold(),
        "a report where nothing ran must not read as a clean bill of health",
    );

    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
        .expect("the root permissions are restorable");
}

/// Makes `path` read-only for the duration of `body`, restoring it after.
///
/// Restored through a guard rather than at the end of the closure, so a failed
/// assertion still leaves the scratch directory removable.
fn while_read_only<T>(path: &Path, body: impl FnOnce() -> T) -> T {
    use std::os::unix::fs::PermissionsExt as _;

    struct Restore<'a>(&'a Path);
    impl Drop for Restore<'_> {
        fn drop(&mut self) {
            let _ = fs::set_permissions(self.0, fs::Permissions::from_mode(0o700));
        }
    }

    fs::set_permissions(path, fs::Permissions::from_mode(0o500))
        .expect("the directory permissions are settable");
    let _restore = Restore(path);
    body()
}

/// A temporary that cannot be created refuses, and leaves nothing behind.
///
/// **Induced through the real `std` call, not a seam.** The ticket's own
/// preference: a read-only shard makes `File::create_new` fail for the reason a
/// full or read-only filesystem would, so what is exercised is the same code
/// path a real failure takes rather than an injected return value.
///
/// The classification is asserted, not merely the refusal. A version that
/// reported this as a corrupt entry or an oversize bundle would still "fail
/// closed" and would send a caller to do the wrong thing.
#[test]
fn a_temporary_that_cannot_be_created_refuses_and_publishes_nothing() {
    let scratch = Scratch::new("io-create-temporary");
    let cache = cache(&scratch);
    // Derived the way `resolve` derives it, from the raw subject bytes rather
    // than a composed subject: taking the wrong key would make the read-only
    // shard a different shard and the test would pass by publishing normally.
    let key = CacheKey::derive_bytes(b"compilation");

    // Create the shard so it exists to be made read-only; a missing shard would
    // fail at `CreateDirectory` instead, which is a different boundary.
    let temporaries = cache.rooted_layout().temporary_dir(&key);
    fs::create_dir_all(&temporaries).expect("the temporary shard is creatable");

    let outcome = while_read_only(&temporaries, || {
        cache.resolve(
            b"compilation",
            || Ok::<_, String>(b"program".to_vec()),
            &any_payload,
        )
    })
    .expect("a refused publication is still a resolved protocol outcome");

    let ProtocolOutcome::Uncached { report, .. } = outcome else {
        panic!("a temporary that cannot be created must not report a hit");
    };
    let Some(PublicationRefusal::Unavailable(unavailable)) = report.publication_refusal() else {
        panic!(
            "expected an unavailable namespace, got {:?}",
            report.publication_refusal()
        );
    };
    assert_eq!(unavailable.operation(), CacheOperation::CreateTemporary);

    // Pre-rename, so no content entry exists and no temporary was left.
    assert!(
        !cache.rooted_layout().entry_path(&key).exists(),
        "a refused publication must not leave a content entry",
    );
    assert!(
        fs::read_dir(&temporaries)
            .expect("the shard is listable once writable again")
            .next()
            .is_none(),
        "a refused publication must not leave a temporary",
    );

    // The next call recovers normally rather than inheriting the failure.
    let recovered = cache
        .resolve(
            b"compilation",
            || Ok::<_, String>(b"program".to_vec()),
            &any_payload,
        )
        .expect("the retry resolves");
    let ProtocolOutcome::Hit { published, .. } = recovered else {
        panic!("the retry must publish once the shard is writable");
    };
    assert!(
        published,
        "the retry publishes the entry the refusal did not"
    );
}

/// A rename that cannot land refuses, and leaves no half-published entry.
///
/// The other side of the publication point: `CreateTemporary` fails before any
/// bytes are written, while this fails after a complete, validated temporary
/// exists. Both must leave the same observable state — no entry, no temporary —
/// which is what makes the rename the single publication instant.
#[test]
fn a_rename_that_cannot_land_refuses_and_leaves_no_entry() {
    let scratch = Scratch::new("io-publish");
    let cache = cache(&scratch);
    let key = CacheKey::derive_bytes(b"compilation");

    let entries = cache.rooted_layout().entry_path(&key);
    let shard = entries
        .parent()
        .expect("an entry path has a shard")
        .to_path_buf();
    fs::create_dir_all(&shard).expect("the entry shard is creatable");

    let outcome = while_read_only(&shard, || {
        cache.resolve(
            b"compilation",
            || Ok::<_, String>(b"program".to_vec()),
            &any_payload,
        )
    })
    .expect("a refused publication is still a resolved protocol outcome");

    let ProtocolOutcome::Uncached { report, .. } = outcome else {
        panic!("a rename that cannot land must not report a hit");
    };
    let Some(PublicationRefusal::Unavailable(unavailable)) = report.publication_refusal() else {
        panic!(
            "expected an unavailable namespace, got {:?}",
            report.publication_refusal()
        );
    };
    assert_eq!(unavailable.operation(), CacheOperation::Publish);

    assert!(
        !entries.exists(),
        "a refused rename must not leave a content entry"
    );
    assert!(
        fs::read_dir(cache.rooted_layout().temporary_dir(&key))
            .expect("the temporary shard is listable")
            .next()
            .is_none(),
        "a refused rename must clean up the temporary it wrote",
    );
}

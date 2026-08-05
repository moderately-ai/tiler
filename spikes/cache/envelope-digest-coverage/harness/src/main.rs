//! Which envelope corruptions only the cache bundle's envelope-section digest
//! catches.
//!
//! # The question
//!
//! `tiler_cache::expansion`'s bundle digests both framed sections on every hit.
//! The artifact-envelope one covers the same byte run that
//! [`tiler_artifact::program::decode_artifact`] then validates in full, and it
//! costs a fifth to a quarter of a validated hit
//! (`decide-whether-the-bundle-envelope-section-digest-is-redundant`). So: is it
//! redundant?
//!
//! Answering it means producing the set of envelope corruptions the bundle
//! digest catches and `decode_artifact` does **not**, by running both over the
//! same bytes rather than by reading the two decoders and arguing.
//!
//! # How a verdict is obtained, and why the second column is the neutered path
//!
//! Each corruption is driven twice.
//!
//! *The shipped column* writes the corrupted bundle to the cache's own entry
//! path and calls the public [`ExpansionCache::lookup`], so the verdict is the
//! one a real reader gets, produced by the real code.
//!
//! *The neutered column* calls `decode_artifact` on the corrupted envelope run
//! directly. That is exactly what the hit path would do next with the bundle's
//! envelope-section digest removed: `bundle::decode` derives the envelope span
//! from the descriptor's offset and length — never from its digest — and
//! `read_entry` hands `&bytes[view.envelope]` straight to the pinned validator,
//! so deleting the digest comparison changes which checks run and changes
//! nothing about the bytes they run on.
//!
//! **That reduction is not taken on trust.** `--mode neutered` runs the whole
//! table against a build whose digest comparison has actually been removed, and
//! the harness refuses to run in either mode until it has *observed* which mode
//! it is in — see [`observe_mode`]. The two retained results are the same table
//! either side of that one-line change, and the shipped column moving exactly
//! where the reduction predicts is what makes the reduction evidence.
//!
//! # What a class list cannot claim, and what the sweep claims instead
//!
//! An enumeration of corruption classes is only as complete as its author, so
//! the classes are stated to be refuted — every one names the exact bytes it
//! perturbs. Underneath them sits a sweep that needs no enumeration at all:
//! every byte position of the envelope run, two perturbations each, counted.
//! A class list argues; the sweep counts.

mod envelope;

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use tiler_artifact::program::{ArtifactCodecFailure, DigestAlgorithm, decode_artifact};
use tiler_cache::expansion::{
    BundleRejection, BundleSection, CacheKey, ComposedSubject, EntryRejection, ExpansionCache,
    Lookup, MissReason, Resolution, SubjectFacets,
};

use envelope::EnvelopeFactory;

// -- restated framing constants --------------------------------------------
//
// Every constant below belongs to a crate-private module of `tiler-cache` or
// `tiler-artifact` and is restated here rather than imported. Each restatement
// is *checked against the published bytes* before it is used — see
// `BundleFrame::locate` and `EnvelopeFrame::locate` — so a framing change in
// either crate fails this spike loudly instead of silently pointing a
// perturbation at the wrong run.

/// Fixed-width framing header of one cache bundle, before its descriptor table.
const BUNDLE_HEADER_BYTES: usize = 64;
/// One bundle section descriptor: purpose, offset, length, digest.
const BUNDLE_DESCRIPTOR_BYTES: usize = 4 + 8 + 8 + 32;
/// Sections one bundle published without a debug retention frames.
const BUNDLE_SECTIONS: usize = 2;
/// Offset of the bundle's declared total length.
const BUNDLE_TOTAL_LENGTH_AT: usize = 16;
/// Offset of a bundle descriptor's declared length, from the descriptor start.
const BUNDLE_DESCRIPTOR_LENGTH_AT: usize = 4 + 8;

/// Namespace version directory `layout.rs` joins exactly.
const NAMESPACE_VERSION: &str = "v1";
/// Leading characters of a rendered key that name its shard directory.
const SHARD_BYTES: usize = 2;

/// Fixed-width framing header of one artifact envelope.
const ENVELOPE_HEADER_BYTES: usize = 69;
/// Offset of the envelope's declared total length.
const ENVELOPE_TOTAL_LENGTH_AT: usize = 17;
/// Offset of the envelope's declared manifest length.
const ENVELOPE_MANIFEST_LENGTH_AT: usize = 25;
/// Offset of the envelope's declared section count.
const ENVELOPE_SECTION_COUNT_AT: usize = 33;
/// Offset of the envelope's carried manifest digest.
const ENVELOPE_MANIFEST_DIGEST_AT: usize = 37;
/// Domain separator of the manifest digest carried in the framing header.
const MANIFEST_DIGEST_DOMAIN: &[u8] = b"tiler.artifact-envelope.manifest-digest.v1\0";
/// Versioned domain tag opening the canonical manifest bytes.
const MANIFEST_DOMAIN: &[u8] = b"tiler.artifact-envelope.manifest.v1\0";
/// One manifest section descriptor: id, purpose, disposition, schema, length,
/// digest.
const MANIFEST_DESCRIPTOR_BYTES: usize = 4 + 1 + 1 + 2 + 2 + 8 + 32;

/// Shape of the published program's input.
///
/// The shape `spikes/cache/hot-path-efficiency` compiles, so the fixture is the
/// one the cache measurements already use.
const PUBLISHED_SHAPE: (u64, u64) = (4, 3);
/// Shape of the substitute program, deliberately different from the published
/// one so a substitution is a different artifact rather than a respelling: the
/// reduced extent is part of the semantic graph, so the two programs reach
/// different semantic subjects and different canonical artifact identities.
///
/// Both rows must stay at or below four. The governed target profile this
/// harness compiles against admits no feasible plan above that — probed over
/// `1..=8` rows by `1..=7` columns, where every row count at or below four
/// compiles at every column count and every row count above four fails
/// `NoFeasiblePlan` at all of them.
const SUBSTITUTE_SHAPE: (u64, u64) = (2, 5);
/// Object bytes beyond the larger program's fixed overhead.
///
/// Only large enough that both programs reach the same total length with room
/// to spare; nothing measured depends on the value.
const OBJECT_HEADROOM: usize = 4096;

/// Positions of the exhaustive sweep that are also driven through the public
/// path, spread evenly across the envelope run.
///
/// The sweep's own column is the artifact decoder's, which is cheap. Driving
/// every position through `lookup` as well would write a fresh multi-kilobyte
/// entry tens of thousands of times to answer a question one SHA-256 inequality
/// already answers, so the public path is sampled instead — and the sample
/// names its population and counts it rather than being described as "spot
/// checks".
const SWEEP_PUBLIC_SAMPLES: usize = 256;

fn main() -> ExitCode {
    let options = Options::parse(&env::args().skip(1).collect::<Vec<_>>());
    run(&options)
}

/// What this run was asked to do.
struct Options {
    /// Which build of `tiler-cache` the caller believes is being exercised.
    mode: Mode,
    /// Name of the retained result, or `None` to print without recording.
    record: Option<String>,
    /// Shorten the sweep for development.
    quick: bool,
}

impl Options {
    fn parse(arguments: &[String]) -> Self {
        let mut options = Self {
            mode: Mode::Shipped,
            record: None,
            quick: false,
        };
        let mut rest = arguments.iter();
        while let Some(argument) = rest.next() {
            match argument.as_str() {
                "--mode" => {
                    options.mode = match rest.next().map(String::as_str) {
                        Some("shipped") => Mode::Shipped,
                        Some("neutered") => Mode::Neutered,
                        other => panic!("--mode takes `shipped` or `neutered`, not {other:?}"),
                    };
                }
                "--record" => {
                    options.record = Some(rest.next().expect("--record takes a name").clone());
                }
                "--quick" => options.quick = true,
                other => panic!("unrecognized argument {other}"),
            }
        }
        options
    }
}

/// Which build of the cache bundle decoder is under the harness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    /// The shipped decoder: the envelope section's digest is compared.
    Shipped,
    /// A build with the envelope section's digest comparison removed.
    Neutered,
}

impl Mode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Shipped => "shipped",
            Self::Neutered => "neutered",
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one run is one linear procedure: fixtures, controls, the class table, the sweep, the report. Splitting it would hide the order the controls have to run in"
)]
fn run(options: &Options) -> ExitCode {
    let mut rows: Vec<Row> = Vec::new();

    // -- fixtures ----------------------------------------------------------

    let published_factory = EnvelopeFactory::new(PUBLISHED_SHAPE.0, PUBLISHED_SHAPE.1, 0x00);
    let substitute_factory = EnvelopeFactory::new(SUBSTITUTE_SHAPE.0, SUBSTITUTE_SHAPE.1, 0x5a);
    let target = published_factory
        .base_bytes()
        .max(substitute_factory.base_bytes())
        + OBJECT_HEADROOM;
    let published = published_factory.exactly(target);
    let substitute = substitute_factory.exactly(target);
    let longer_substitute = substitute_factory.exactly(target + 1024);

    assert_eq!(
        published.len(),
        substitute.len(),
        "the substitution class needs two envelopes of one length",
    );
    assert_ne!(
        published, substitute,
        "the substitution class needs two envelopes that differ",
    );
    // Both substitutes must be things `decode_artifact` accepts on their own,
    // or the substitution classes below would be testing malformed bytes rather
    // than a different valid artifact. Their identities must also differ from
    // the published one: a substitute that carried the *same* artifact would
    // make "the reader returned something else" a claim about bytes rather than
    // about meaning.
    let mut identities = Vec::new();
    for (name, bytes) in [
        ("published", &published),
        ("substitute", &substitute),
        ("longer-substitute", &longer_substitute),
    ] {
        let decoded = decode_artifact(bytes)
            .unwrap_or_else(|failure| panic!("the {name} fixture is a valid envelope: {failure}"));
        identities.push(decoded.identity().as_bytes().to_vec());
    }
    assert_ne!(
        identities[0], identities[1],
        "the equal-length substitute packages a different artifact",
    );
    assert_ne!(
        identities[0], identities[2],
        "the longer substitute packages a different artifact",
    );

    let scratch = Scratch::new();
    let cache = ExpansionCache::open(scratch.path());
    let subject = ComposedSubject::compose(&SubjectFacets {
        backend_compilations: &[b"tiler.spike.envelope-digest-coverage.compilation"],
        artifact_program: b"tiler.spike.envelope-digest-coverage.artifact-program",
    })
    .expect("the fixture names every facet");
    let key = CacheKey::derive(&subject);
    let entry = entry_path(scratch.path(), &key);

    match cache.get_or_publish(&subject, || Ok::<_, String>(published.clone())) {
        Ok(Resolution::Published { .. }) => {}
        other => panic!("the fixture publication did not publish: {other:?}"),
    }
    let stored = fs::read(&entry).expect("the published entry is readable");

    // -- controls that locate the two frames -------------------------------

    let bundle_frame = BundleFrame::locate(&stored, &subject, &published);
    let identity = decode_artifact(&published)
        .expect("the published fixture decodes")
        .identity();
    let envelope_frame = EnvelopeFrame::locate(&published, identity.as_bytes());
    rows.push(Row::control(
        "bundle-frame-restatement",
        format!(
            "envelope span {}..{} holds the published envelope",
            bundle_frame.envelope.0, bundle_frame.envelope.1
        ),
    ));
    rows.push(Row::control(
        "envelope-frame-restatement",
        format!(
            "manifest {}..{} digests to the header value; {} framed sections close the run; the \
             carried identity is {} bytes at {}..{}",
            envelope_frame.manifest.0,
            envelope_frame.manifest.1,
            envelope_frame.sections.len(),
            envelope_frame.identity.1 - envelope_frame.identity.0,
            envelope_frame.identity.0,
            envelope_frame.identity.1,
        ),
    ));
    rows.push(Row::control(
        "substitutes-are-different-artifacts",
        format!(
            "the equal-length and longer substitutes decode, and both carry a canonical artifact \
             identity that differs from the published one ({} bytes each)",
            identities[0].len(),
        ),
    ));

    // -- the control that says which build this is -------------------------

    let observed = observe_mode(&cache, &subject, &entry, &stored, &bundle_frame);
    rows.push(Row::control(
        "observed-mode",
        format!(
            "one flipped envelope content byte is refused by `{}`, so the bundle envelope-section \
             digest is {}",
            observed.verdict,
            if observed.mode == Mode::Shipped {
                "live"
            } else {
                "not live"
            },
        ),
    ));
    if observed.mode != options.mode {
        eprintln!(
            "refusing to record: --mode {} was requested and the build behaves as {}. The \
             refusing boundary observed for one flipped envelope content byte was `{}`.",
            options.mode.as_str(),
            observed.mode.as_str(),
            observed.verdict,
        );
        return ExitCode::FAILURE;
    }

    // -- the class table ---------------------------------------------------

    let classes = classes(&envelope_frame, &substitute, &longer_substitute);
    let mut stop_conditions = 0_u32;
    for class in &classes {
        let perturbed = (class.perturb)(&published);
        let neutered = verdict_of(decode_artifact(&perturbed).map(|_| ()));
        let shipped = shipped_verdict(&cache, &subject, &entry, &stored, &bundle_frame, &perturbed);
        let attribution = attribute(&shipped, &neutered);
        if attribution == Attribution::Neither && options.mode == Mode::Shipped {
            stop_conditions += 1;
        }
        rows.push(Row::class(class, &shipped, &neutered, attribution));
    }

    // -- the exhaustive single-byte sweep ----------------------------------

    let sweep = sweep(
        &cache,
        &subject,
        &entry,
        &stored,
        &bundle_frame,
        &published,
        options.quick,
    );
    rows.push(Row::sweep(&sweep));
    if !sweep.accepted.is_empty() && options.mode == Mode::Shipped {
        stop_conditions += u32::try_from(sweep.accepted.len())
            .expect("fewer accepted positions than a u32 counts");
    }

    // -- restore, so the run ends on a hit rather than on a rejection ------

    fs::write(&entry, &stored).expect("the published entry is writable");
    match cache.lookup(&subject) {
        Lookup::Hit(hit) => assert_eq!(
            hit.envelope_bytes(),
            published.as_slice(),
            "restoring the entry returns the published envelope",
        ),
        Lookup::Miss(reason) => panic!("the restored entry did not hit: {reason}"),
    }
    rows.push(Row::control(
        "restored",
        "the unperturbed entry hits and returns the exact published envelope".to_owned(),
    ));

    report(options, &rows, target, published.len());
    if stop_conditions == 0 {
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "{stop_conditions} corruption(s) were caught by neither check against the shipped \
             build; that is a correctness finding, not a result to record",
        );
        ExitCode::FAILURE
    }
}

// -- the corruption classes ------------------------------------------------

/// Produces one corrupted envelope run from the published one.
type Perturb = Box<dyn Fn(&[u8]) -> Vec<u8>>;

/// One named class of envelope corruption.
struct Class {
    /// Stable identifier, used as the result row's key.
    id: &'static str,
    /// Which region of the envelope run it perturbs.
    region: &'static str,
    /// Exactly which bytes it changes, so a reader can refute the class list.
    bytes: String,
    /// Produces the corrupted envelope run from the published one.
    perturb: Perturb,
}

/// Flips the low bit of one byte.
fn flip(at: usize) -> Perturb {
    Box::new(move |bytes: &[u8]| {
        let mut copy = bytes.to_vec();
        copy[at] ^= 0x01;
        copy
    })
}

/// Enumerates the classes, derived from the envelope encoder's own field order.
///
/// Every class names the exact bytes it changes. A reader who believes the list
/// is incomplete can name a byte of the run and check which class covers it —
/// and the sweep below covers every byte of the run without an enumeration at
/// all.
#[expect(
    clippy::too_many_lines,
    reason = "the list is the deliverable: one arm per class, each stating the exact bytes it perturbs"
)]
fn classes(frame: &EnvelopeFrame, substitute: &[u8], longer_substitute: &[u8]) -> Vec<Class> {
    let (manifest_start, manifest_end) = frame.manifest;
    let identity_at = frame.identity.0;
    let first_descriptor = frame.descriptor_table + 8;
    let substitute = substitute.to_vec();
    let longer = longer_substitute.to_vec();

    let mut classes = vec![
        Class {
            id: "header-magic",
            region: "framing header",
            bytes: "0..8".to_owned(),
            perturb: flip(0),
        },
        Class {
            id: "header-envelope-format-major",
            region: "framing header",
            bytes: "8..10".to_owned(),
            perturb: flip(9),
        },
        Class {
            id: "header-envelope-format-minor",
            region: "framing header",
            bytes: "10..12".to_owned(),
            perturb: flip(11),
        },
        Class {
            id: "header-canonical-encoding-major",
            region: "framing header",
            bytes: "12..14".to_owned(),
            perturb: flip(13),
        },
        Class {
            id: "header-canonical-encoding-minor",
            region: "framing header",
            bytes: "14..16".to_owned(),
            perturb: flip(15),
        },
        Class {
            id: "header-digest-algorithm-tag",
            region: "framing header",
            bytes: "16".to_owned(),
            perturb: flip(16),
        },
        Class {
            id: "header-total-length",
            region: "framing header",
            bytes: "17..25".to_owned(),
            perturb: flip(ENVELOPE_TOTAL_LENGTH_AT + 7),
        },
        Class {
            id: "header-manifest-length",
            region: "framing header",
            bytes: "25..33".to_owned(),
            perturb: flip(ENVELOPE_MANIFEST_LENGTH_AT + 7),
        },
        Class {
            id: "header-section-count",
            region: "framing header",
            bytes: "33..37".to_owned(),
            perturb: flip(ENVELOPE_SECTION_COUNT_AT + 3),
        },
        Class {
            id: "header-manifest-digest",
            region: "framing header",
            bytes: "37..69".to_owned(),
            perturb: flip(ENVELOPE_MANIFEST_DIGEST_AT),
        },
        Class {
            id: "manifest-domain-tag",
            region: "manifest",
            bytes: format!(
                "{manifest_start}..{}",
                manifest_start + MANIFEST_DOMAIN.len()
            ),
            perturb: flip(manifest_start),
        },
        Class {
            id: "manifest-schema-version",
            region: "manifest",
            bytes: format!(
                "{}..{}",
                manifest_start + MANIFEST_DOMAIN.len(),
                manifest_start + MANIFEST_DOMAIN.len() + 4
            ),
            perturb: flip(manifest_start + MANIFEST_DOMAIN.len() + 1),
        },
        Class {
            id: "manifest-component-schema-versions",
            region: "manifest",
            bytes: format!(
                "{}..{}",
                manifest_start + MANIFEST_DOMAIN.len() + 4,
                manifest_start + MANIFEST_DOMAIN.len() + 20
            ),
            perturb: flip(manifest_start + MANIFEST_DOMAIN.len() + 5),
        },
        Class {
            id: "manifest-interior-byte",
            region: "manifest",
            bytes: format!("{}", usize::midpoint(manifest_start, manifest_end)),
            perturb: flip(usize::midpoint(manifest_start, manifest_end)),
        },
        Class {
            id: "manifest-section-descriptor",
            region: "manifest",
            bytes: format!(
                "{first_descriptor}..{}",
                first_descriptor + MANIFEST_DESCRIPTOR_BYTES
            ),
            perturb: flip(first_descriptor),
        },
        Class {
            id: "manifest-carried-identity",
            region: "manifest",
            bytes: format!("{identity_at}..{}", frame.identity.1),
            perturb: flip(identity_at),
        },
    ];

    // The framed section stream: three fields per section, each named.
    for (index, section) in frame.sections.iter().enumerate() {
        classes.push(Class {
            id: leak(format!("section-{index}-framing-id")),
            region: "section stream",
            bytes: format!("{}..{}", section.id_at, section.id_at + 4),
            perturb: flip(section.id_at + 3),
        });
        classes.push(Class {
            id: leak(format!("section-{index}-framed-length")),
            region: "section stream",
            bytes: format!("{}..{}", section.id_at + 4, section.id_at + 12),
            perturb: flip(section.id_at + 11),
        });
        classes.push(Class {
            id: leak(format!("section-{index}-content")),
            region: "section stream",
            bytes: format!("{}..{}", section.content.0, section.content.1),
            perturb: flip(section.content.0 + (section.content.1 - section.content.0) / 2),
        });
    }

    // Re-sealed classes. A single flipped manifest byte never reaches the
    // decoder's later checks, because the manifest digest refuses it first — so
    // a table that stopped at the classes above would say nothing at all about
    // whether those later checks are reachable. Each class here restores the
    // manifest digest over its own edit, which is the forger's move rather than
    // damage, and is what puts the named canonicity checks, the section
    // digests, and the identity comparison on the record.
    for (id, at) in [
        ("resealed-manifest-carried-identity", identity_at),
        ("resealed-manifest-descriptor-id", first_descriptor + 3),
        (
            "resealed-manifest-descriptor-length",
            first_descriptor + 4 + 1 + 1 + 2 + 2 + 7,
        ),
        (
            "resealed-manifest-descriptor-digest",
            first_descriptor + 4 + 1 + 1 + 2 + 2 + 8,
        ),
    ] {
        classes.push(Class {
            id,
            region: "manifest, re-sealed",
            bytes: format!("{at}, with the header manifest digest recomputed"),
            perturb: Box::new(move |bytes: &[u8]| {
                let mut copy = bytes.to_vec();
                copy[at] ^= 0x01;
                reseal_manifest(&mut copy);
                copy
            }),
        });
    }

    // Structural classes: the run's own length and its whole contents.
    classes.push(Class {
        id: "structural-truncated-by-one",
        region: "structural",
        bytes: "the final byte of the run is removed".to_owned(),
        perturb: Box::new(|bytes: &[u8]| bytes[..bytes.len() - 1].to_vec()),
    });
    classes.push(Class {
        id: "structural-extended-by-one",
        region: "structural",
        bytes: "one byte is appended to the run".to_owned(),
        perturb: Box::new(|bytes: &[u8]| {
            let mut copy = bytes.to_vec();
            copy.push(0x00);
            copy
        }),
    });
    classes.push(Class {
        id: "structural-trailing-byte-inside-declared-length",
        region: "structural",
        bytes: "one byte is appended and the envelope's own total length follows it".to_owned(),
        perturb: Box::new(|bytes: &[u8]| {
            let mut copy = bytes.to_vec();
            copy.push(0x00);
            let total = u64::try_from(copy.len()).expect("a fixture envelope fits a u64");
            copy[ENVELOPE_TOTAL_LENGTH_AT..ENVELOPE_TOTAL_LENGTH_AT + 8]
                .copy_from_slice(&total.to_be_bytes());
            copy
        }),
    });
    classes.push(Class {
        id: "structural-two-bytes-transposed",
        region: "structural",
        bytes: "two content bytes exchange positions; no byte value changes".to_owned(),
        perturb: Box::new({
            let first = frame.sections[0].content.0;
            let second = frame.sections[frame.sections.len() - 1].content.1 - 1;
            move |bytes: &[u8]| {
                let mut copy = bytes.to_vec();
                copy.swap(first, second);
                copy
            }
        }),
    });
    classes.push(Class {
        id: "structural-substituted-equal-length-envelope",
        region: "structural",
        bytes: "the whole run becomes a different valid envelope of the same length".to_owned(),
        perturb: Box::new(move |_: &[u8]| substitute.clone()),
    });
    classes.push(Class {
        id: "structural-substituted-longer-envelope",
        region: "structural",
        bytes: "the whole run becomes a different valid envelope of a different length".to_owned(),
        perturb: Box::new(move |_: &[u8]| longer.clone()),
    });
    classes
}

/// Interns one generated class identifier for the lifetime of the process.
///
/// The class list is built once and read until the process exits, so leaking a
/// per-section name costs a handful of bytes and keeps [`Class::id`] a plain
/// `&'static str` for every arm rather than making the fixed classes carry an
/// owned string they never needed.
fn leak(name: String) -> &'static str {
    Box::leak(name.into_boxed_str())
}

/// Recomputes the framing header's manifest digest over the manifest now
/// present.
fn reseal_manifest(bytes: &mut [u8]) {
    let manifest_bytes = read_u64(bytes, ENVELOPE_MANIFEST_LENGTH_AT);
    let start = ENVELOPE_HEADER_BYTES;
    let end = start + usize::try_from(manifest_bytes).expect("a fixture manifest fits this host");
    let digest = DigestAlgorithm::GOVERNED.digest(MANIFEST_DIGEST_DOMAIN, &bytes[start..end]);
    bytes[ENVELOPE_MANIFEST_DIGEST_AT..ENVELOPE_MANIFEST_DIGEST_AT + 32]
        .copy_from_slice(digest.as_bytes());
}

// -- driving the two paths -------------------------------------------------

/// Which boundary refused, rendered for the result.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Verdict {
    /// `accepted`, or the class of check that refused.
    boundary: String,
    /// The refusing boundary's own message, or the empty string.
    detail: String,
}

impl Verdict {
    fn accepted() -> Self {
        Self {
            boundary: "accepted".to_owned(),
            detail: String::new(),
        }
    }

    fn is_accepted(&self) -> bool {
        self.boundary == "accepted"
    }
}

/// Renders one artifact-codec outcome.
fn verdict_of(outcome: Result<(), ArtifactCodecFailure>) -> Verdict {
    match outcome {
        Ok(()) => Verdict::accepted(),
        Err(failure) => Verdict {
            boundary: "artifact-codec".to_owned(),
            detail: failure.to_string(),
        },
    }
}

/// Splices a corrupted envelope run into the published bundle and reads it back
/// through the public API.
///
/// The bundle's own descriptor length and total length follow the run's length,
/// so a length-changing corruption is refused by whatever inspects the envelope
/// rather than by the bundle's contiguity chain. Nothing else about the bundle
/// is touched — in particular the envelope section's declared digest stays the
/// digest of the bytes the publisher framed, which is the whole point.
fn shipped_verdict(
    cache: &ExpansionCache,
    subject: &ComposedSubject,
    entry: &Path,
    stored: &[u8],
    frame: &BundleFrame,
    perturbed: &[u8],
) -> Verdict {
    fs::write(entry, frame.splice(stored, perturbed)).expect("the entry path is writable");
    match cache.lookup(subject) {
        Lookup::Hit(hit) => {
            assert_eq!(
                hit.envelope_bytes(),
                perturbed,
                "a hit returned bytes that are not the ones written to the entry",
            );
            Verdict::accepted()
        }
        Lookup::Miss(MissReason::Rejected(EntryRejection::Bundle(rejection))) => Verdict {
            boundary: match rejection {
                BundleRejection::SectionDigest {
                    purpose: BundleSection::ArtifactEnvelope,
                } => "bundle-envelope-section-digest".to_owned(),
                _ => "bundle-other".to_owned(),
            },
            detail: rejection.to_string(),
        },
        Lookup::Miss(MissReason::Rejected(EntryRejection::Payload(failure))) => Verdict {
            boundary: "artifact-codec".to_owned(),
            detail: failure.to_string(),
        },
        Lookup::Miss(other) => Verdict {
            boundary: "cache-other".to_owned(),
            detail: other.to_string(),
        },
    }
}

/// What the pair of verdicts says about the digest under test.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Attribution {
    /// Both checks refuse it.
    Both,
    /// Only the bundle's envelope-section digest refuses it.
    OnlyBundleDigest,
    /// A bundle check other than the digest under test refuses it.
    OtherBundleCheck,
    /// Neither refuses it.
    Neither,
}

impl Attribution {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Both => "both",
            Self::OnlyBundleDigest => "only-bundle-digest",
            Self::OtherBundleCheck => "other-bundle-check",
            Self::Neither => "NEITHER",
        }
    }
}

/// Attributes one corruption from the two columns.
fn attribute(shipped: &Verdict, neutered: &Verdict) -> Attribution {
    if shipped.is_accepted() {
        // The shipped path runs the artifact decoder after the digest, so a hit
        // means both checks passed. A decoder that refused here and a hit that
        // returned anyway would be a contradiction rather than an attribution.
        assert!(
            neutered.is_accepted(),
            "the shipped path accepted bytes its own validator refuses",
        );
        return Attribution::Neither;
    }
    if !neutered.is_accepted() {
        return Attribution::Both;
    }
    if shipped.boundary == "bundle-envelope-section-digest" {
        Attribution::OnlyBundleDigest
    } else {
        Attribution::OtherBundleCheck
    }
}

/// What the liveness control observed.
struct Observed {
    mode: Mode,
    verdict: String,
}

/// Observes whether the bundle's envelope-section digest is live in this build.
///
/// One flipped byte of envelope content is refused by the bundle digest in the
/// shipped build and by the artifact decoder in a neutered one, so the boundary
/// that answers *names the build*. Nothing below records a row until this has
/// run, because a table labelled with a mode nobody observed is a table whose
/// central claim is an assumption.
fn observe_mode(
    cache: &ExpansionCache,
    subject: &ComposedSubject,
    entry: &Path,
    stored: &[u8],
    frame: &BundleFrame,
) -> Observed {
    let mut probe = stored[frame.envelope.0..frame.envelope.1].to_vec();
    let at = probe.len() / 2;
    probe[at] ^= 0x01;
    let verdict = shipped_verdict(cache, subject, entry, stored, frame, &probe);
    let mode = if verdict.boundary == "bundle-envelope-section-digest" {
        Mode::Shipped
    } else {
        Mode::Neutered
    };
    Observed {
        mode,
        verdict: verdict.detail,
    }
}

// -- the exhaustive sweep --------------------------------------------------

/// What the per-position sweep found.
struct Sweep {
    positions: usize,
    perturbations: usize,
    decodes: usize,
    /// Positions and perturbations the artifact decoder accepted.
    accepted: Vec<String>,
    /// Sampled positions driven through the public path.
    sampled: usize,
    /// Sampled positions the public path refused by the digest under test.
    sampled_by_digest: usize,
}

/// Perturbs every byte position of the envelope run and counts the verdicts.
///
/// This is the part of the experiment that owes nothing to an enumeration. The
/// classes above can be incomplete; a count over every position of the run
/// cannot be, for the corruption shape it covers — one changed byte.
fn sweep(
    cache: &ExpansionCache,
    subject: &ComposedSubject,
    entry: &Path,
    stored: &[u8],
    frame: &BundleFrame,
    published: &[u8],
    quick: bool,
) -> Sweep {
    let perturbations: [u8; 2] = [0x01, 0x80];
    let stride = if quick { 97 } else { 1 };
    let sample_every = published.len() / SWEEP_PUBLIC_SAMPLES.max(1);
    let mut result = Sweep {
        positions: 0,
        perturbations: perturbations.len(),
        decodes: 0,
        accepted: Vec::new(),
        sampled: 0,
        sampled_by_digest: 0,
    };
    let mut probe = published.to_vec();
    for at in (0..published.len()).step_by(stride) {
        result.positions += 1;
        for mask in perturbations {
            probe[at] ^= mask;
            result.decodes += 1;
            if decode_artifact(&probe).is_ok() {
                result.accepted.push(format!("{at}^{mask:#04x}"));
            }
            probe[at] ^= mask;
        }
        if sample_every > 0 && at % sample_every == 0 {
            probe[at] ^= 0x01;
            let verdict = shipped_verdict(cache, subject, entry, stored, frame, &probe);
            probe[at] ^= 0x01;
            result.sampled += 1;
            if verdict.boundary == "bundle-envelope-section-digest" {
                result.sampled_by_digest += 1;
            }
        }
    }
    assert_eq!(
        probe, published,
        "the sweep restores every byte it perturbs",
    );
    result
}

// -- locating the two frames -----------------------------------------------

/// Where the published envelope sits inside the bundle that frames it.
struct BundleFrame {
    /// Span of the artifact-envelope section, which a retention-free bundle
    /// frames last.
    envelope: (usize, usize),
    /// Offset of the artifact-envelope descriptor's declared length.
    envelope_length_at: usize,
}

impl BundleFrame {
    /// Derives the frame from the restated constants and checks it.
    ///
    /// # Panics
    ///
    /// Panics when the derived envelope span does not hold the exact envelope
    /// that was published, or when that span does not end the bundle. Either
    /// means `bundle.rs`'s frame moved, and every perturbation below would
    /// otherwise be aimed at the wrong bytes.
    fn locate(stored: &[u8], subject: &ComposedSubject, envelope: &[u8]) -> Self {
        let table_end = BUNDLE_HEADER_BYTES + BUNDLE_DESCRIPTOR_BYTES * BUNDLE_SECTIONS;
        let start = table_end + subject.as_bytes().len();
        let end = start + envelope.len();
        assert_eq!(
            &stored[start..end],
            envelope,
            "the restated bundle frame locates the published envelope",
        );
        assert_eq!(
            end,
            stored.len(),
            "a retention-free bundle frames the envelope last, so the run ends the file",
        );
        Self {
            envelope: (start, end),
            envelope_length_at: BUNDLE_HEADER_BYTES
                + BUNDLE_DESCRIPTOR_BYTES
                + BUNDLE_DESCRIPTOR_LENGTH_AT,
        }
    }

    /// Rebuilds the bundle with `perturbed` in place of the envelope it framed.
    fn splice(&self, stored: &[u8], perturbed: &[u8]) -> Vec<u8> {
        let mut bytes = stored[..self.envelope.0].to_vec();
        bytes.extend_from_slice(perturbed);
        let length = u64::try_from(perturbed.len()).expect("a fixture envelope fits a u64");
        bytes[self.envelope_length_at..self.envelope_length_at + 8]
            .copy_from_slice(&length.to_be_bytes());
        let total = u64::try_from(bytes.len()).expect("a fixture bundle fits a u64");
        bytes[BUNDLE_TOTAL_LENGTH_AT..BUNDLE_TOTAL_LENGTH_AT + 8]
            .copy_from_slice(&total.to_be_bytes());
        bytes
    }
}

/// One framed section of the artifact envelope.
struct FramedSection {
    /// Offset of the section's framing identifier, which its length follows.
    id_at: usize,
    /// Span of the section's content bytes.
    content: (usize, usize),
}

/// Where the manifest and the framed sections sit inside one envelope.
struct EnvelopeFrame {
    manifest: (usize, usize),
    sections: Vec<FramedSection>,
    /// Span of the manifest's trailing canonical artifact identity.
    identity: (usize, usize),
    /// Offset of the manifest's section descriptor table, at its own count.
    descriptor_table: usize,
}

impl EnvelopeFrame {
    /// Walks the envelope's own frame and checks every restatement against it.
    ///
    /// The trailing canonical identity is *variable* width — it is a canonical
    /// byte run, not a digest — so its span is taken from the decoder's own
    /// [`DecodedArtifact::identity`] and then required to sit at the manifest's
    /// tail under its own length prefix, rather than assumed to be some fixed
    /// number of bytes. Everything behind it, the descriptor table included, is
    /// located from there.
    ///
    /// # Panics
    ///
    /// Panics when the magic, the declared total length, the manifest domain,
    /// the manifest digest, the declared section count, the closure of the
    /// section stream, the identity's placement, or the descriptor table's
    /// restated width disagrees with the bytes — that is, whenever
    /// `encode.rs`'s layout has moved and this file has not.
    fn locate(envelope: &[u8], identity: &[u8]) -> Self {
        assert_eq!(
            &envelope[..8],
            b"TILERART",
            "the envelope magic is restated"
        );
        assert_eq!(
            read_u64(envelope, ENVELOPE_TOTAL_LENGTH_AT),
            u64::try_from(envelope.len()).expect("a fixture envelope fits a u64"),
            "the declared total length is where the restatement says",
        );
        let manifest_bytes = usize::try_from(read_u64(envelope, ENVELOPE_MANIFEST_LENGTH_AT))
            .expect("a fixture manifest fits this host");
        let manifest = (
            ENVELOPE_HEADER_BYTES,
            ENVELOPE_HEADER_BYTES + manifest_bytes,
        );
        assert_eq!(
            &envelope[manifest.0..manifest.0 + MANIFEST_DOMAIN.len()],
            MANIFEST_DOMAIN,
            "the manifest opens with the restated domain",
        );
        assert_eq!(
            DigestAlgorithm::GOVERNED
                .digest(MANIFEST_DIGEST_DOMAIN, &envelope[manifest.0..manifest.1])
                .as_bytes(),
            &envelope[ENVELOPE_MANIFEST_DIGEST_AT..ENVELOPE_MANIFEST_DIGEST_AT + 32],
            "the restated manifest span and digest domain reproduce the carried digest",
        );

        let declared = usize::try_from(read_u32(envelope, ENVELOPE_SECTION_COUNT_AT))
            .expect("a u32 section count fits this host");
        let mut sections = Vec::with_capacity(declared);
        let mut at = manifest.1;
        for index in 0..declared {
            let id_at = at;
            assert_eq!(
                usize::try_from(read_u32(envelope, id_at)).expect("a u32 fits this host"),
                index,
                "a framed section identifier is its position",
            );
            let length =
                usize::try_from(read_u64(envelope, id_at + 4)).expect("a section fits this host");
            let content = (id_at + 12, id_at + 12 + length);
            assert!(
                content.1 <= envelope.len(),
                "a framed section lies inside the envelope",
            );
            sections.push(FramedSection { id_at, content });
            at = content.1;
        }
        assert_eq!(
            at,
            envelope.len(),
            "the framed section stream closes the envelope exactly",
        );
        // The identity closes the manifest under its own length prefix, and the
        // descriptor table is the run immediately before it. Both are checked
        // against the bytes, or the re-sealed classes would aim at the wrong
        // field of the wrong row.
        let identity_span = (manifest.1 - identity.len(), manifest.1);
        assert_eq!(
            &envelope[identity_span.0..identity_span.1],
            identity,
            "the decoder's canonical identity closes the manifest",
        );
        assert_eq!(
            usize::try_from(read_u64(envelope, identity_span.0 - 8))
                .expect("a fixture identity length fits this host"),
            identity.len(),
            "the manifest's trailing identity carries its own length prefix",
        );
        let descriptor_table = identity_span.0 - 8 - (8 + MANIFEST_DESCRIPTOR_BYTES * declared);
        assert_eq!(
            usize::try_from(read_u64(envelope, descriptor_table))
                .expect("a fixture section count fits this host"),
            declared,
            "the restated descriptor table starts at its own count",
        );
        for (index, section) in sections.iter().enumerate() {
            let row = descriptor_table + 8 + MANIFEST_DESCRIPTOR_BYTES * index;
            assert_eq!(
                usize::try_from(read_u32(envelope, row)).expect("a u32 fits this host"),
                index,
                "a manifest descriptor identifier is its position",
            );
            assert_eq!(
                usize::try_from(read_u64(envelope, row + 4 + 1 + 1 + 2 + 2))
                    .expect("a fixture section length fits this host"),
                section.content.1 - section.content.0,
                "a manifest descriptor declares the length the stream framed",
            );
        }
        Self {
            manifest,
            sections,
            identity: identity_span,
            descriptor_table,
        }
    }
}

fn read_u64(bytes: &[u8], at: usize) -> u64 {
    u64::from_be_bytes(bytes[at..at + 8].try_into().expect("a fixed-width field"))
}

fn read_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(bytes[at..at + 4].try_into().expect("a fixed-width field"))
}

// -- reporting -------------------------------------------------------------

/// One line of the retained result.
struct Row {
    section: &'static str,
    id: String,
    region: String,
    bytes: String,
    shipped: String,
    shipped_detail: String,
    neutered: String,
    neutered_detail: String,
    attribution: String,
}

impl Row {
    fn control(id: &str, detail: String) -> Self {
        Self {
            section: "control",
            id: id.to_owned(),
            region: String::new(),
            bytes: String::new(),
            shipped: "observed".to_owned(),
            shipped_detail: detail,
            neutered: String::new(),
            neutered_detail: String::new(),
            attribution: String::new(),
        }
    }

    fn class(
        class: &Class,
        shipped: &Verdict,
        neutered: &Verdict,
        attribution: Attribution,
    ) -> Self {
        Self {
            section: "class",
            id: class.id.to_owned(),
            region: class.region.to_owned(),
            bytes: class.bytes.clone(),
            shipped: shipped.boundary.clone(),
            shipped_detail: shipped.detail.clone(),
            neutered: neutered.boundary.clone(),
            neutered_detail: neutered.detail.clone(),
            attribution: attribution.as_str().to_owned(),
        }
    }

    fn sweep(sweep: &Sweep) -> Self {
        Self {
            section: "sweep",
            id: "every-position-single-byte".to_owned(),
            region: "whole run".to_owned(),
            bytes: format!(
                "{} positions x {} perturbations = {} decodes",
                sweep.positions, sweep.perturbations, sweep.decodes
            ),
            shipped: format!("{}/{} sampled", sweep.sampled_by_digest, sweep.sampled),
            shipped_detail: "sampled positions refused by the bundle envelope-section digest"
                .to_owned(),
            neutered: format!(
                "{}/{} refused",
                sweep.decodes - sweep.accepted.len(),
                sweep.decodes
            ),
            neutered_detail: if sweep.accepted.is_empty() {
                "no perturbation of any position was accepted".to_owned()
            } else {
                format!("accepted: {}", sweep.accepted.join(" "))
            },
            attribution: if sweep.accepted.is_empty() {
                Attribution::Both.as_str().to_owned()
            } else {
                Attribution::OnlyBundleDigest.as_str().to_owned()
            },
        }
    }
}

/// Prints the table and, when asked, retains it.
fn report(options: &Options, rows: &[Row], target: usize, envelope_bytes: usize) {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# mode\t{}\n# envelope-bytes\t{envelope_bytes}\n# solver-target\t{target}",
        options.mode.as_str(),
    );
    let _ = writeln!(
        out,
        "section\tid\tregion\tbytes\tshipped\tshipped_detail\tneutered\tneutered_detail\t\
         attribution",
    );
    for row in rows {
        // A cell a row does not fill is written as a dash rather than left
        // empty. An empty trailing cell ends the line in a tab, which the
        // repository's whitespace check reports and which makes "this row has no
        // such column" indistinguishable from "the value was the empty string".
        let cell = |value: &str| {
            if value.is_empty() {
                "-".to_owned()
            } else {
                value.to_owned()
            }
        };
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.section,
            cell(&row.id),
            cell(&row.region),
            cell(&row.bytes),
            cell(&row.shipped),
            cell(&row.shipped_detail),
            cell(&row.neutered),
            cell(&row.neutered_detail),
            cell(&row.attribution),
        );
    }
    print!("{out}");
    if let Some(name) = &options.record {
        let path = PathBuf::from("results").join(format!("envelope-digest-coverage-{name}.tsv"));
        fs::create_dir_all("results").expect("the results directory is creatable");
        fs::write(&path, out).expect("the result is writable");
        println!("# recorded {}", path.display());
    }
}

// -- scratch ---------------------------------------------------------------

/// `<root>/v1/entries/<K[0..2]>/<K>.bundle`, restated from `layout.rs`.
fn entry_path(root: &Path, key: &CacheKey) -> PathBuf {
    let label = key.label();
    root.join(NAMESPACE_VERSION)
        .join("entries")
        .join(&label[..SHARD_BYTES])
        .join(format!("{label}.bundle"))
}

/// A directory this run owns entirely, removed when the run finishes.
struct Scratch {
    path: PathBuf,
}

impl Scratch {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the host clock is after the Unix epoch")
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "tiler-cache-envelope-digest-coverage-{}-{nonce}",
            std::process::id(),
        ));
        fs::create_dir_all(&path).expect("a scratch directory is creatable");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

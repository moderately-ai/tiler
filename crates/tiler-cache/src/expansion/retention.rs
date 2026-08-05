//! Debug text a producer retains beside one published entry.
//!
//! # The three identity questions, and the answers this module implements
//!
//! Retained text is the first thing a bundle carries that is *not* part of what
//! the entry is. Each question below is answered by a mechanism rather than by a
//! convention, because a convention here is a wrong hit.
//!
//! **It does not participate in the key.** [`super::CacheKey::derive`] is a
//! function of the composed subject alone and the encoder derives the key from
//! the subject section it writes, so one compilation resolves to one entry
//! whether or not a caller retained anything. There is no code path by which a
//! retention could reach the derivation: it is not a [`super::SubjectFacet`], and
//! the only bytes hashed into a key are the ones a [`super::ComposedSubject`]
//! frames. A debug build and a release build therefore contend for one entry,
//! which is the point — the alternative doubles every compilation the moment
//! anyone turns retention on.
//!
//! **It does participate in the frame's digest set.** The section carries its own
//! content digest in the descriptor table, sits inside the declared total length,
//! and must begin exactly where the previous section ended. Adding, removing, or
//! editing retained text in a published entry therefore breaks the section
//! digest, the total length, or the contiguity chain, and the entry is refused
//! rather than served. An entry stays immutable in the strong sense: not "nobody
//! edits it" but "an edited one is not a valid entry".
//!
//! **An absent section is a hit with nothing to show.** An entry published by a
//! build that retained nothing is a complete, valid entry — the retention is not
//! a required section, [`DebugRetention::is_empty`] answers `true`, and no reader
//! misses because of it. Treating absence as a miss would make turning retention
//! on discard every entry the fleet already has, for text that is diagnostics.
//!
//! **Absence is the canonical spelling of "nothing retained".** An empty
//! retention encodes to *no section at all*, and a framed section declaring zero
//! runs is refused by [`RetentionRejection::RunCount`]. Two encodings of one fact
//! would be two byte runs for one entry, which is the same canonicity the frame
//! enforces when it refuses trailing bytes.
//!
//! # What the cache understands about retained text, which is nothing
//!
//! A label is opaque here. This crate frames, counts, bounds, and digests the
//! runs; it never interprets one, exactly as [`super::ComposedSubject`] wraps a
//! producer's canonical bytes without parsing them. That is what keeps
//! consumer-specific vocabulary — MSL, `metal`, a linker stage — out of a
//! consumer-agnostic crate: the producer names its own runs and reads them back
//! by the names it chose.
//!
//! # What belongs here, and what belongs in the envelope
//!
//! Only text the identity-bearing envelope cannot carry. A payload's canonical
//! *source* is already inside the artifact envelope this bundle carries —
//! `tiler_artifact::program::PayloadMetadata::source` is part of the payload
//! identity preimage — so it reaches a reader through the artifact on every hit,
//! under the digest that names it. Copying it here would create a second,
//! unkeyed authority over the same text that nothing could refuse when the two
//! disagreed. What has no other home is the *output of the tool run*: a
//! compiler's warnings are not a compilation input, so they cannot enter payload
//! metadata without making one compilation's identity depend on which host
//! emitted which warning.

use core::fmt;

/// Versioned domain tag opening every encoded retention section.
///
/// ADR 0074 convention 3, and the same role [`super::ComposedSubject`]'s tag
/// plays: bytes that do not open with it are not a retention this build wrote,
/// and a later encoding is a new tag rather than a reinterpretation of these
/// bytes.
pub(super) const RETENTION_DOMAIN: &[u8] = b"tiler.cache.debug-retention.v1\0";

/// Largest number of bytes one retained run keeps.
///
/// The bound and its reasoning are `tiler_metal_aot::diagnostic::ToolOutput`'s,
/// restated here rather than imported because ADR 0082 item 2 fixes this crate's
/// dependency closure to `tiler-artifact` alone: a failing or chatty tool can
/// emit megabytes, and an entry that carried all of it would make a diagnostic as
/// expensive to store and read as the artifact it describes. Truncation is
/// recorded rather than hidden — [`RetainedText::total_bytes`] and
/// [`RetainedText::is_truncated`] are what keep a prefix from being shown as the
/// whole.
pub const MAX_RETAINED_RUN_BYTES: usize = 16 * 1024;

/// Largest number of runs one retention carries.
///
/// A retention names the stages of one compilation, and a producer that reached
/// this many has stopped describing a compilation. The bound is checked when a
/// run is added and again when a stored section is decoded, so a hostile section
/// cannot make a reader allocate a run table without limit.
pub const MAX_RETAINED_RUNS: usize = 16;

/// Largest width, in bytes, of one run's label.
pub const MAX_RETENTION_LABEL_BYTES: usize = 64;

/// One labelled run of retained debug text.
///
/// **Bytes, not `String`.** A tool's output is whatever it wrote, and decoding
/// lossily at the capture site would replace an invalid sequence with `U+FFFD`
/// and leave nothing able to tell that from a tool that really emitted a
/// replacement character. The bytes are kept and [`Self::is_valid_utf8`] says
/// which case a reader is in — the same decision, for the same reason, that
/// `tiler_metal_aot::diagnostic::ToolOutput` records.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RetainedText {
    label: String,
    retained: Vec<u8>,
    total: u64,
}

impl RetainedText {
    /// Returns the producer's own name for this run.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the retained bytes exactly as the producer supplied them.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.retained
    }

    /// Returns how many bytes the producer had, including any not retained.
    #[must_use]
    pub const fn total_bytes(&self) -> u64 {
        self.total
    }

    /// Returns whether bytes were dropped to stay within
    /// [`MAX_RETAINED_RUN_BYTES`].
    #[must_use]
    pub fn is_truncated(&self) -> bool {
        self.total > self.retained.len() as u64
    }

    /// Returns whether the retained bytes are valid UTF-8.
    ///
    /// A reader that renders regardless should still know, because a lossy
    /// rendering of invalid bytes is not what the tool said.
    #[must_use]
    pub fn is_valid_utf8(&self) -> bool {
        core::str::from_utf8(&self.retained).is_ok()
    }

    /// Returns whether the producer had nothing to retain for this run.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.total == 0
    }
}

impl fmt::Display for RetainedText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: {}",
            self.label,
            String::from_utf8_lossy(&self.retained).trim(),
        )?;
        if !self.is_valid_utf8() {
            formatter.write_str(" [output was not valid UTF-8]")?;
        }
        if self.is_truncated() {
            write!(
                formatter,
                " [truncated: {} of {} bytes retained]",
                self.retained.len(),
                self.total,
            )?;
        }
        Ok(())
    }
}

/// The debug text one publication retains, in the order a producer named it.
///
/// A *stated input*, never a discovered one: this crate reads no environment
/// variable and consults no build profile to decide what an entry carries.
/// Retaining nothing is [`Self::none`] and is what every caller that does not ask
/// for retention gets, so the debug configuration lives entirely with the caller
/// that has one — the ADR 0089 root policy applied to a second decision.
///
/// A *derived* value in the sense of ADR 0074 convention 2: the runs are private,
/// [`Self::retaining`] is the only way to add one, and every bound is enforced
/// there and again when a stored section is decoded.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DebugRetention {
    runs: Vec<RetainedText>,
}

impl DebugRetention {
    /// Retains nothing.
    #[must_use]
    pub const fn none() -> Self {
        Self { runs: Vec::new() }
    }

    /// Retains up to [`MAX_RETAINED_RUN_BYTES`] of one labelled run.
    ///
    /// The bytes are truncated rather than refused, because a caller that had to
    /// pre-truncate would either duplicate this bound or lose the total; the
    /// total is recorded here so the truncation is reported rather than hidden.
    ///
    /// # Errors
    ///
    /// Returns [`RetentionRefusal`] for a label outside the governed alphabet, a
    /// label already retained, or a run past [`MAX_RETAINED_RUNS`]. A duplicate
    /// label is refused rather than appended so [`Self::run`] answers about the
    /// whole retention rather than about whichever run was found first.
    pub fn retaining(mut self, label: &str, bytes: &[u8]) -> Result<Self, RetentionRefusal> {
        check_label(label)?;
        if self.runs.iter().any(|run| run.label == label) {
            return Err(RetentionRefusal::DuplicateLabel {
                label: label.to_owned(),
            });
        }
        if self.runs.len() == MAX_RETAINED_RUNS {
            return Err(RetentionRefusal::TooManyRuns {
                limit: MAX_RETAINED_RUNS,
            });
        }
        self.runs.push(RetainedText {
            label: label.to_owned(),
            retained: bytes.iter().take(MAX_RETAINED_RUN_BYTES).copied().collect(),
            total: bytes.len() as u64,
        });
        Ok(self)
    }

    /// Returns whether nothing is retained.
    ///
    /// True for an entry published without retention *and* for a caller that
    /// asked for none: a hit with nothing to show is one state, not two.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }

    /// Returns the retained runs, in the order the producer named them.
    #[must_use]
    pub fn runs(&self) -> &[RetainedText] {
        &self.runs
    }

    /// Returns the run a producer retained under `label`, if any.
    #[must_use]
    pub fn run(&self, label: &str) -> Option<&RetainedText> {
        self.runs.iter().find(|run| run.label == label)
    }

    /// Encodes this retention into one canonical section body.
    ///
    /// Only called for a non-empty retention: an empty one is framed as the
    /// absence of the section, which is the canonical spelling of the same fact.
    pub(super) fn encode(&self) -> Vec<u8> {
        debug_assert!(
            !self.is_empty(),
            "an empty retention is framed as an absent section",
        );
        let mut bytes = Vec::new();
        bytes.extend_from_slice(RETENTION_DOMAIN);
        push_count(&mut bytes, self.runs.len() as u64);
        for run in &self.runs {
            push_run(&mut bytes, run.label.as_bytes());
            push_count(&mut bytes, run.total);
            push_run(&mut bytes, &run.retained);
        }
        bytes
    }

    /// Decodes and completely validates one stored retention section.
    ///
    /// Every bound [`Self::retaining`] enforces is enforced again here, because
    /// these bytes came off a disk any process on the host may write to and the
    /// producer's checks prove nothing about them.
    pub(super) fn decode(bytes: &[u8]) -> Result<Self, RetentionRejection> {
        let mut cursor = Cursor { bytes, at: 0 };
        if cursor.take(RETENTION_DOMAIN.len())? != RETENTION_DOMAIN {
            return Err(RetentionRejection::Domain);
        }
        let declared = cursor.count()?;
        if declared == 0 || declared > MAX_RETAINED_RUNS as u64 {
            return Err(RetentionRejection::RunCount {
                declared,
                limit: MAX_RETAINED_RUNS,
            });
        }
        // Bounded above, so the conversion cannot fail on an admitted profile. It
        // is still checked into the rejection that already describes a count this
        // build will not frame, rather than cast.
        let declared = usize::try_from(declared).map_err(|_| RetentionRejection::RunCount {
            declared,
            limit: MAX_RETAINED_RUNS,
        })?;
        let mut retention = Self::none();
        for index in 0..declared {
            let label = core::str::from_utf8(cursor.run()?)
                .map_err(|_| RetentionRejection::LabelNotUtf8 { index })?
                .to_owned();
            let total = cursor.count()?;
            // Both bounds are checked against the framed span before it is
            // copied, so a section declaring a run far larger than this build
            // writes is refused rather than duplicated first and refused after.
            let retained = cursor.run()?;
            if retained.len() as u64 > total {
                return Err(RetentionRejection::RetainedAboveTotal {
                    index,
                    retained: retained.len() as u64,
                    total,
                });
            }
            if retained.len() > MAX_RETAINED_RUN_BYTES {
                return Err(RetentionRejection::RunTooLarge {
                    index,
                    length: retained.len() as u64,
                    limit: MAX_RETAINED_RUN_BYTES,
                });
            }
            let retained = retained.to_vec();
            check_label(&label).map_err(|refusal| RetentionRejection::Label { index, refusal })?;
            if retention.runs.iter().any(|run| run.label == label) {
                return Err(RetentionRejection::Label {
                    index,
                    refusal: RetentionRefusal::DuplicateLabel { label },
                });
            }
            retention.runs.push(RetainedText {
                label,
                retained,
                total,
            });
        }
        if cursor.at != bytes.len() {
            return Err(RetentionRejection::TrailingBytes {
                after: cursor.at,
                total: bytes.len(),
            });
        }
        Ok(retention)
    }
}

/// A reader over one stored retention section.
///
/// Nothing is indexed before the length that frames it has been proven against
/// the bytes actually present, which is the same order the bundle frame reads its
/// own sections in.
struct Cursor<'bytes> {
    bytes: &'bytes [u8],
    at: usize,
}

impl<'bytes> Cursor<'bytes> {
    fn take(&mut self, len: usize) -> Result<&'bytes [u8], RetentionRejection> {
        let end = self
            .at
            .checked_add(len)
            .ok_or(RetentionRejection::Truncated {
                needed: usize::MAX,
                found: self.bytes.len(),
            })?;
        if end > self.bytes.len() {
            return Err(RetentionRejection::Truncated {
                needed: end,
                found: self.bytes.len(),
            });
        }
        let taken = &self.bytes[self.at..end];
        self.at = end;
        Ok(taken)
    }

    fn count(&mut self) -> Result<u64, RetentionRejection> {
        let bytes = self.take(COUNT_BYTES)?;
        Ok(u64::from_be_bytes(
            bytes.try_into().expect("a fixed-width field"),
        ))
    }

    fn run(&mut self) -> Result<&'bytes [u8], RetentionRejection> {
        let length = self.count()?;
        // The declared length is compared against the bytes present before it is
        // used to index, so a hostile length cannot make this allocate or panic:
        // a run longer than the section is a truncation rejection. A length wider
        // than the address space — unreachable on the 64-bit profiles this
        // workspace admits — saturates into the same rejection through the
        // overflow check in `take`.
        self.take(usize::try_from(length).unwrap_or(usize::MAX))
    }
}

/// Width of one framed count or length.
const COUNT_BYTES: usize = 8;

/// Writes a fixed-width big-endian count.
///
/// This crate's canonical length framing, admitted for the reason
/// [`super::ComposedSubject`]'s copy is: `tiler_ir::identity` owns the workspace's
/// framing and ADR 0082 item 2 keeps it outside this crate's dependency closure.
/// This module reaches the copy beside it rather than adding a third.
fn push_count(bytes: &mut Vec<u8>, count: u64) {
    bytes.extend_from_slice(&count.to_be_bytes());
}

/// Writes one length-prefixed run.
fn push_run(bytes: &mut Vec<u8>, run: &[u8]) {
    push_count(bytes, run.len() as u64);
    bytes.extend_from_slice(run);
}

/// Validates one run label.
///
/// The alphabet is the artifact layer's governed-key alphabet — ASCII lowercase,
/// ASCII digits, `.`, `-`, and `_` — restated rather than imported, because
/// `tiler_artifact`'s validator answers in that crate's build-error vocabulary
/// and a cache rejection must not be phrased as an artifact build failure. Fixing
/// the alphabet is what keeps a label a stable name a producer can look a run up
/// by, rather than arbitrary text that renders differently everywhere it is read.
fn check_label(label: &str) -> Result<(), RetentionRefusal> {
    if label.is_empty() {
        return Err(RetentionRefusal::EmptyLabel);
    }
    if label.len() > MAX_RETENTION_LABEL_BYTES {
        return Err(RetentionRefusal::LabelTooLong {
            found: label.len(),
            limit: MAX_RETENTION_LABEL_BYTES,
        });
    }
    // Iterating bytes rather than characters keeps the width check above and this
    // one counting the same units: every accepted byte is one ASCII character.
    for (position, byte) in label.bytes().enumerate() {
        if !matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'.' | b'-' | b'_') {
            return Err(RetentionRefusal::NoncanonicalLabelByte { position, byte });
        }
    }
    Ok(())
}

/// Why a caller's retention was refused.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a rejection vocabulary a
/// caller forwards or partially classifies rather than maps totally.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RetentionRefusal {
    /// The label is empty.
    EmptyLabel,
    /// The label is wider than [`MAX_RETENTION_LABEL_BYTES`].
    LabelTooLong {
        /// Width that was supplied, in bytes.
        found: usize,
        /// Configured maximum.
        limit: usize,
    },
    /// A label byte is outside the governed alphabet.
    NoncanonicalLabelByte {
        /// Zero-based byte position of the offending byte.
        position: usize,
        /// The byte that was found.
        byte: u8,
    },
    /// A run with this label is already retained.
    DuplicateLabel {
        /// The label that appeared twice.
        label: String,
    },
    /// The retention already carries [`MAX_RETAINED_RUNS`] runs.
    TooManyRuns {
        /// Configured maximum.
        limit: usize,
    },
}

impl fmt::Display for RetentionRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyLabel => {
                formatter.write_str("a retained run's label is empty, and no run is unnamed")
            }
            Self::LabelTooLong { found, limit } => write!(
                formatter,
                "a retained run's label is {found} bytes wide, above the maximum {limit}",
            ),
            Self::NoncanonicalLabelByte { position, byte } => write!(
                formatter,
                "byte {position} of a retained run's label is {byte:#04x}, which is outside the \
                 governed alphabet of ASCII lowercase, digits, `.`, `-`, and `_`",
            ),
            Self::DuplicateLabel { label } => write!(
                formatter,
                "a retention already carries a run labelled `{label}`",
            ),
            Self::TooManyRuns { limit } => {
                write!(formatter, "a retention carries at most {limit} runs")
            }
        }
    }
}

impl std::error::Error for RetentionRefusal {}

/// Why stored bytes are not a valid retention section.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RetentionRejection {
    /// The section does not open with this build's retention domain.
    Domain,
    /// Fewer bytes are present than a structure already declared needs.
    Truncated {
        /// Bytes the declared structure requires.
        needed: usize,
        /// Bytes actually present.
        found: usize,
    },
    /// The declared run count is zero or above [`MAX_RETAINED_RUNS`].
    ///
    /// Zero is refused because an empty retention is framed as an absent
    /// section: admitting both would give one fact two encodings.
    RunCount {
        /// Count the section declares.
        declared: u64,
        /// Configured maximum.
        limit: usize,
    },
    /// A run's label is not valid UTF-8.
    LabelNotUtf8 {
        /// Zero-based run index.
        index: usize,
    },
    /// A run's label is not one this build would have written.
    Label {
        /// Zero-based run index.
        index: usize,
        /// The rule the label breaks.
        refusal: RetentionRefusal,
    },
    /// A run declares fewer total bytes than it retains.
    ///
    /// The total is what makes truncation reportable, so a total below the
    /// retained length is a section claiming a prefix is longer than the whole.
    RetainedAboveTotal {
        /// Zero-based run index.
        index: usize,
        /// Retained bytes present.
        retained: u64,
        /// Total the run declares.
        total: u64,
    },
    /// A run retains more than [`MAX_RETAINED_RUN_BYTES`].
    RunTooLarge {
        /// Zero-based run index.
        index: usize,
        /// Retained length the run declares.
        length: u64,
        /// Configured maximum.
        limit: usize,
    },
    /// Bytes follow the last framed run.
    TrailingBytes {
        /// Offset at which the last run ended.
        after: usize,
        /// Bytes present in the section.
        total: usize,
    },
}

impl fmt::Display for RetentionRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Domain => formatter
                .write_str("a retention section does not open with this build's retention domain"),
            Self::Truncated { needed, found } => write!(
                formatter,
                "a retention section declares a structure needing {needed} bytes and has {found}",
            ),
            Self::RunCount { declared, limit } => write!(
                formatter,
                "a retention section declares {declared} runs, which is not between 1 and {limit}",
            ),
            Self::LabelNotUtf8 { index } => write!(
                formatter,
                "the label of retained run {index} is not valid UTF-8",
            ),
            Self::Label { index, refusal } => {
                write!(formatter, "retained run {index}: {refusal}")
            }
            Self::RetainedAboveTotal {
                index,
                retained,
                total,
            } => write!(
                formatter,
                "retained run {index} keeps {retained} bytes and declares {total} in total",
            ),
            Self::RunTooLarge {
                index,
                length,
                limit,
            } => write!(
                formatter,
                "retained run {index} keeps {length} bytes, above the maximum {limit}",
            ),
            Self::TrailingBytes { after, total } => write!(
                formatter,
                "a retention section's last run ends at {after} and the section has {total} bytes",
            ),
        }
    }
}

impl std::error::Error for RetentionRejection {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Label { refusal, .. } => Some(refusal),
            Self::Domain
            | Self::Truncated { .. }
            | Self::RunCount { .. }
            | Self::LabelNotUtf8 { .. }
            | Self::RetainedAboveTotal { .. }
            | Self::RunTooLarge { .. }
            | Self::TrailingBytes { .. } => None,
        }
    }
}

//! The retained realization-probe record, and the row a comparison against it
//! carries.
//!
//! # Why this module exists
//!
//! [`crate::envelope`] compares the SHA-256 of bytes *this* device produced
//! against a `result_sha256` a *different* run measured. That comparison is only
//! meaningful with two things stated: which record the digest came from, and
//! which environment row the record was taken on. Before this module both were
//! prose — the digests were transcribed literals and the row was compared by a
//! human once, in a ticket outcome. What is here reads the record itself, on
//! every host, and states the row difference as a value rather than as a
//! sentence.
//!
//! # The record is read, and the digests in the source are held against it
//!
//! [`direct_digests`] parses the retained `workload.tsv` and returns the
//! `direct` realization's row per cell. `crate::envelope::tests` compares it
//! against [`crate::envelope::L3_CORRECTNESS_CELLS`], so the six literals this
//! crate routes against are a *checked transcription* rather than an asserted
//! one, on every host including the ones that cannot measure. Reading the record
//! and then *using* what it says would remove the pin instead of checking it,
//! which is why the literals stay literals.
//!
//! # What a differing row does, and why it is not one rule for all six fields
//!
//! The retained record names an Apple M4 Max under macOS 27.0 `26A5388g`,
//! Xcode 26.6 `17F113`, SDK 26.5 `25F70`, and the offline Metal compiler
//! `32023.883`. Two of those have already moved: this repository's host resolves
//! SDK `macosx 27.0` and metal `32023.921`. **So no currently reachable host is
//! on the record's row**, and a rule that declined to compare on any difference
//! would make the comparison permanently unmade — turning the one executed
//! cross-workspace check in this crate into a boundary report on every run,
//! which is the opposite of what retaining it was for.
//!
//! The split is therefore by what a difference *means*:
//!
//! - **The device and its GPU family are refused.** A digest measured on one
//!   machine and reproduced on another is a different claim, and a disagreement
//!   there would arrive looking like a Tiler regression when it is a statement
//!   about two GPUs. [`RowComparison::hardware_differences`] is non-empty and
//!   `crate::envelope` declines the retained comparison, naming the fields —
//!   while still routing the member and comparing it against the published
//!   reference, which is a claim any Apple host can make.
//! - **The architecture, OS build, offline compiler, and SDK are announced and
//!   the comparison proceeds.** Each of them can move the bits, which is exactly
//!   why a difference must be *named* — but if it moved them, this comparison
//!   goes red and reports the row alongside the three digests, which is the
//!   correctness finding the record was retained to produce. Declining instead
//!   would trade a check that can fail for a sentence that cannot.
//!
//! Either way the difference is printed on every run, so a reader never has to
//! infer which row a green result covers.

use std::path::{Path, PathBuf};

use crate::measurement::MeasurementBoundary;

/// The retained correctness record's directory, relative to the repository root.
///
/// Named once. Both files this module reads live in it, and the digest literals
/// in `crate::envelope` cite it in prose; a second spelling would be a second
/// thing to move when a later record supersedes this one.
pub(crate) const RECORD_DIRECTORY: &str = "spikes/scheduling/metal_contraction_vertical/results/\
     2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883";

/// Why the retained record could not be read.
///
/// A defect rather than a boundary in every case: the record is a checked-in
/// file, so a host that cannot read it has a broken checkout rather than a
/// missing device.
#[derive(Debug)]
pub(crate) enum RecordFailure {
    /// A file of the record could not be read.
    Read(String, std::io::Error),
    /// A file of the record does not have the columns this reader needs.
    Malformed {
        /// Which file.
        file: &'static str,
        /// What was missing.
        detail: String,
    },
}

impl std::fmt::Display for RecordFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read(path, cause) => write!(
                formatter,
                "the retained realization-probe record {path} could not be read: {cause}",
            ),
            Self::Malformed { file, detail } => write!(
                formatter,
                "the retained realization-probe record's {file} does not carry what this reader \
                 needs: {detail}",
            ),
        }
    }
}

impl std::error::Error for RecordFailure {}

/// Returns one file of the retained record, as a path this host can open.
fn record_path(file: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(RECORD_DIRECTORY)
        .join(file)
}

/// Reads one tab-separated file of the record.
fn read(file: &'static str) -> Result<String, RecordFailure> {
    let path = record_path(file);
    std::fs::read_to_string(&path)
        .map_err(|cause| RecordFailure::Read(path.display().to_string(), cause))
}

/// The retained environment row, as the record's own `key`/`value` pairs.
///
/// Returned as pairs rather than as a struct of named fields: this reader states
/// which keys it *uses* in [`compare`], and a record carrying keys nothing here
/// reads is not a defect. What would be a defect is a key this reader needs and
/// the record does not carry, and that is reported by [`RecordFailure::Malformed`]
/// at the point of use.
///
/// # Errors
///
/// Returns [`RecordFailure::Read`] when the file cannot be opened and
/// [`RecordFailure::Malformed`] when it carries no `key`/`value` header.
pub(crate) fn environment_row() -> Result<Vec<(String, String)>, RecordFailure> {
    let text = read("environment.tsv")?;
    let mut lines = text.lines();
    let header = lines.next().unwrap_or_default();
    if header.split('\t').collect::<Vec<_>>() != ["key", "value"] {
        return Err(RecordFailure::Malformed {
            file: "environment.tsv",
            detail: format!("its header is {header:?} and this reader expects `key`, `value`"),
        });
    }
    Ok(lines
        .filter_map(|line| {
            line.split_once('\t')
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
        })
        .collect())
}

/// One cell's retained `direct` realization row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DirectRow {
    /// The record's own cell identifier.
    pub(crate) id: String,
    /// Rows of the activations operand and of the result.
    pub(crate) m: u64,
    /// Rows of the weights operand and columns of the result.
    pub(crate) n: u64,
    /// The contracted extent, shared by both operands.
    pub(crate) k: u64,
    /// SHA-256 of that realization's result bytes.
    pub(crate) result_sha256: String,
}

/// Reads the `direct` realization's row for every cell in the retained record.
///
/// **The `direct` realization and no other, and the filter is the point.** The
/// record carries six realizations per cell and four of them are permitted but
/// differently-grouped answers with different digests; comparing an executed
/// strict fold against `ksplit_strided`'s digest would report a wrong answer for
/// a right reason. The column is selected by name from the header rather than by
/// position, so a record that gained a column ahead of it is read correctly or
/// refused rather than read off by one.
///
/// # Errors
///
/// Returns [`RecordFailure::Read`] when the file cannot be opened and
/// [`RecordFailure::Malformed`] when a column this reader needs is absent or a
/// numeric field does not parse.
pub(crate) fn direct_digests() -> Result<Vec<DirectRow>, RecordFailure> {
    const NEEDED: [&str; 6] = ["cell", "realization", "m", "n", "k", "result_sha256"];

    let text = read("workload.tsv")?;
    let mut lines = text.lines();
    let header: Vec<&str> = lines.next().unwrap_or_default().split('\t').collect();
    let mut columns = Vec::with_capacity(NEEDED.len());
    for name in NEEDED {
        let position = header
            .iter()
            .position(|column| *column == name)
            .ok_or_else(|| RecordFailure::Malformed {
                file: "workload.tsv",
                detail: format!("it carries no {name:?} column; its header is {header:?}"),
            })?;
        columns.push(position);
    }
    let [cell, realization, m, n, k, digest] = <[usize; 6]>::try_from(columns.as_slice())
        .expect("one position was resolved for each needed column");

    let mut rows = Vec::new();
    for line in lines {
        let fields: Vec<&str> = line.split('\t').collect();
        let Some(kind) = fields.get(realization) else {
            continue;
        };
        if *kind != "direct" {
            continue;
        }
        let field = |position: usize, name: &'static str| -> Result<u64, RecordFailure> {
            fields
                .get(position)
                .and_then(|value| value.parse::<u64>().ok())
                .ok_or_else(|| RecordFailure::Malformed {
                    file: "workload.tsv",
                    detail: format!("a `direct` row's {name} is not an extent: {line:?}"),
                })
        };
        rows.push(DirectRow {
            id: (*fields.get(cell).unwrap_or(&"")).to_owned(),
            m: field(m, "m")?,
            n: field(n, "n")?,
            k: field(k, "k")?,
            result_sha256: (*fields.get(digest).unwrap_or(&"")).to_owned(),
        });
    }
    if rows.is_empty() {
        return Err(RecordFailure::Malformed {
            file: "workload.tsv",
            detail: "it carries no `direct` realization row at all, so a filter that had stopped \
                     matching would be indistinguishable from a record with nothing to compare"
                .to_owned(),
        });
    }
    Ok(rows)
}

/// One environment field, as the record states it and as this host observes it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RowField {
    /// The field's name in this comparison's own vocabulary.
    pub(crate) name: &'static str,
    /// What the retained record states.
    pub(crate) retained: String,
    /// What this host observed.
    pub(crate) observed: String,
    /// Whether the field names the *machine* rather than how the unit was built.
    ///
    /// The module header states what the distinction buys; in short, a hardware
    /// difference makes the retained comparison a claim about two GPUs and is
    /// declined, and a toolchain difference is announced and compared, because
    /// the record's toolchain row is no longer reachable on any current host.
    pub(crate) hardware: bool,
}

impl std::fmt::Display for RowField {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{}: record {:?}, this host {:?}",
            self.name, self.retained, self.observed,
        )
    }
}

/// How this host's observed row compares against the retained record's.
#[derive(Clone, Debug)]
pub(crate) struct RowComparison {
    /// Every compared field, in a stable order, whether or not it agreed.
    pub(crate) fields: Vec<RowField>,
}

impl RowComparison {
    /// The fields that differ, in the order they were compared.
    pub(crate) fn differences(&self) -> Vec<&RowField> {
        self.fields
            .iter()
            .filter(|field| field.retained != field.observed)
            .collect()
    }

    /// The differing fields that name the machine rather than the toolchain.
    ///
    /// Non-empty means the retained digest was measured on other hardware, and
    /// `crate::envelope` declines the comparison rather than reporting a GPU
    /// difference as a Tiler defect.
    pub(crate) fn hardware_differences(&self) -> Vec<&RowField> {
        self.differences()
            .into_iter()
            .filter(|field| field.hardware)
            .collect()
    }

    /// Renders the comparison as the one sentence a run prints before comparing.
    pub(crate) fn render(&self) -> String {
        let differences = self.differences();
        if differences.is_empty() {
            return format!(
                "this host is on the retained record's own row; all {} compared field(s) agree",
                self.fields.len(),
            );
        }
        format!(
            "this host differs from the retained record's row in {} of {} compared field(s) — {}",
            differences.len(),
            self.fields.len(),
            differences
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("; "),
        )
    }
}

/// Compares one observed measurement boundary against the retained record's row.
///
/// **Six fields, and the mapping between the two vocabularies is done here rather
/// than by spelling the record's keys into the boundary.** The record is a
/// spike's own TSV and the boundary is what this crate observes; neither may be
/// bent to the other's spelling, so this is the one place they are put side by
/// side. `xcode` is in the record and is deliberately not compared: nothing in
/// [`MeasurementBoundary`] observes it, and inventing an observation to compare
/// against would report a constant as a measurement.
///
/// # Errors
///
/// Returns [`RecordFailure`] when the record cannot be read, or
/// [`RecordFailure::Malformed`] when a key this comparison needs is absent — a
/// record that had lost `offline_compiler` would otherwise compare the observed
/// value against an empty string and report a difference that is about the
/// reader.
pub(crate) fn compare(observed: &MeasurementBoundary) -> Result<RowComparison, RecordFailure> {
    let row = environment_row()?;
    let value = |key: &'static str| -> Result<String, RecordFailure> {
        row.iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value.clone())
            .ok_or_else(|| RecordFailure::Malformed {
                file: "environment.tsv",
                detail: format!("it carries no {key:?} row, which this comparison needs"),
            })
    };

    // The record asks one family question and records the answer as a word; this
    // crate observes the highest family the device names. Restated rather than
    // compared raw, so the two vocabularies meet at a value both can express.
    let retained_family = if value("device_apple9")? == "supported" {
        "Apple9".to_owned()
    } else {
        format!("not Apple9 ({})", value("device_apple9")?)
    };

    Ok(RowComparison {
        fields: vec![
            RowField {
                name: "device",
                retained: value("device")?,
                observed: observed.device_name.clone(),
                hardware: true,
            },
            RowField {
                name: "gpu-family",
                retained: retained_family,
                observed: observed.gpu_family.clone(),
                hardware: true,
            },
            RowField {
                name: "architecture",
                retained: value("host_arch")?,
                observed: observed.architecture.clone(),
                hardware: false,
            },
            RowField {
                name: "os",
                retained: value("host_os")?,
                observed: format!("{} {}", observed.os_version, observed.os_build),
                hardware: false,
            },
            RowField {
                name: "offline-compiler",
                retained: value("offline_compiler")?,
                // The banner's first line only: `metallib`'s adds a legacy-linker
                // note the record never carried, and comparing it would report a
                // difference about the two tools' output formats.
                observed: observed
                    .metal_compiler
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .to_owned(),
                hardware: false,
            },
            RowField {
                name: "sdk",
                retained: format!(
                    "macosx {} build {}",
                    value("macos_sdk_version")?,
                    value("macos_sdk_build")?,
                ),
                observed: observed.sdk.clone(),
                hardware: false,
            },
        ],
    })
}

#[cfg(test)]
mod tests;

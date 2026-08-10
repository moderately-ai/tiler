//! Private comparison between retained execution records and the manually owned
//! dtype ledger.
//!
//! A run derives only the qualifier inside `Conformance evidence`. Maturity and
//! navigation remain manual inputs, while the complete Markdown cell is checked
//! so malformed syntax cannot masquerade as matching evidence. The other eight
//! layer columns are shape-checked and never interpreted.

use core::mem::variant_count;
use std::collections::BTreeSet;

use crate::measurement::MeasurementBoundary;

const PHYSICAL_HEADER: [&str; 10] = [
    "Family",
    "Physical carrier and encoding",
    "ABI and materialization",
    "Optimizer legality",
    "Kernel vocabulary",
    "Backend lowering",
    "Backend execution",
    "Runtime semantic validation",
    "Target-family dispatchability",
    "Conformance evidence",
];

/// A dtype-ledger row backed by retained execution evidence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ConformanceCell {
    /// IEEE binary32.
    F32,
    /// BF16.
    Bf16,
}

impl ConformanceCell {
    const ALL: [Self; variant_count::<Self>()] = [Self::F32, Self::Bf16];

    const fn row_label(self) -> &'static str {
        match self {
            Self::F32 => "IEEE `f32`",
            Self::Bf16 => "BF16",
        }
    }

    /// Manual maturity and navigation around the run-derived qualifier.
    const fn manual_shell(self) -> (&'static str, &'static str) {
        match self {
            Self::F32 => ("tested guarantee", "#ieee-f32"),
            Self::Bf16 => ("tested guarantee", "#other-ieee-binary-floats-and-bf16"),
        }
    }
}

/// Closed identity vocabulary for retained executions.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum RetainedRunIdentity {
    /// The checked F32 realization-probe directory.
    F32RealizationRecord,
    /// The retained BF16 vertical result.
    Bf16Vertical,
}

impl RetainedRunIdentity {
    fn parse(spelling: &str) -> Result<Self, String> {
        match spelling {
            "spikes/scheduling/metal_contraction_vertical/results/2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883" => {
                Ok(Self::F32RealizationRecord)
            }
            "pure-bf16-vertical@b7c01815" => Ok(Self::Bf16Vertical),
            other => Err(format!(
                "retained execution identity {other:?} is not governed by this conformance corpus"
            )),
        }
    }
}

/// Historical environment retained with an executed result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RetainedEnvironment {
    /// Device name.
    pub(crate) device: &'static str,
    /// Reported GPU family.
    pub(crate) gpu_family: &'static str,
    /// Host architecture.
    pub(crate) architecture: &'static str,
    /// Operating-system family.
    pub(crate) os_family: &'static str,
    /// Operating-system version.
    pub(crate) os_version: &'static str,
    /// Operating-system build.
    pub(crate) os_build: &'static str,
    /// Offline-compiler version fragment.
    pub(crate) metal_version: &'static str,
    /// Canonical SDK version/build fragment.
    pub(crate) sdk: &'static str,
}

impl RetainedEnvironment {
    /// The row on which both retained suites ran on 2026-08-07.
    pub(crate) const APPLE9_2026_08_07: Self = Self {
        device: "Apple M4 Max",
        gpu_family: "Apple9",
        architecture: "arm64",
        os_family: "macOS",
        os_version: "27.0",
        os_build: "26A5388g",
        metal_version: "32023.921",
        sdk: "macosx 27.0 build 26A5388f",
    };

    fn qualifier(self) -> String {
        let sdk = self
            .sdk
            .strip_prefix("macosx ")
            .unwrap_or(self.sdk)
            .replace(" build ", " ");
        format!(
            "{} / {} / {} {} {} / metal {} / SDK {} / {}",
            self.device,
            self.gpu_family,
            self.os_family,
            self.os_version,
            self.os_build,
            self.metal_version,
            sdk,
            self.architecture,
        )
    }

    fn differences(self, boundary: &MeasurementBoundary) -> Vec<String> {
        let exact = [
            ("device", self.device, boundary.device_name.as_str()),
            ("gpu-family", self.gpu_family, boundary.gpu_family.as_str()),
            (
                "architecture",
                self.architecture,
                boundary.architecture.as_str(),
            ),
            ("os-version", self.os_version, boundary.os_version.as_str()),
            ("os-build", self.os_build, boundary.os_build.as_str()),
        ];
        let mut differences: Vec<String> = exact
            .into_iter()
            .filter(|(_, retained, observed)| retained != observed)
            .map(|(name, retained, observed)| {
                format!("{name}: retained {retained:?}, fresh rerun {observed:?}")
            })
            .collect();
        if !self.os_family.eq_ignore_ascii_case(&boundary.os_family) {
            differences.push(format!(
                "os-family: retained {:?}, fresh rerun {:?}",
                self.os_family, boundary.os_family,
            ));
        }
        for (name, retained, observed) in [
            (
                "offline-compiler",
                self.metal_version,
                boundary.metal_compiler.as_str(),
            ),
            ("sdk", self.sdk, boundary.sdk.as_str()),
        ] {
            if !observed.contains(retained) {
                differences.push(format!(
                    "{name}: retained fragment {retained:?}, fresh rerun {observed:?}"
                ));
            }
        }
        differences
    }
}

/// Retained F32 execution tied to the checked probe record.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RetainedF32Execution {
    /// Governed record identity.
    pub(crate) identity: &'static str,
    /// Historical execution environment.
    pub(crate) environment: RetainedEnvironment,
}

/// Retained BF16 execution consumed by the live device test.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RetainedBf16Execution {
    /// Governed record identity.
    pub(crate) identity: &'static str,
    /// Historical execution environment.
    pub(crate) environment: RetainedEnvironment,
    /// Device-observed results in corpus order.
    pub(crate) observed: [u16; 15],
}

/// Which composition boundary a run crossed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompositionExtent {
    /// Planning, packaging, and routing all ran.
    RoutedArtifact,
    /// The BF16 run was hand-assembled below the refusing request boundary.
    HandAssembledBf16,
}

impl CompositionExtent {
    const fn qualifier(self) -> &'static str {
        match self {
            Self::RoutedArtifact => {
                "the routed runs cross `compile()`, the artifact envelope, and the routing commit"
            }
            Self::HandAssembledBf16 => {
                "one device run crossing neither the optimizer, the artifact envelope, nor the routing commit"
            }
        }
    }
}

/// Subject derived from executable corpus and retained-record state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RunSubject {
    cell: ConformanceCell,
    identity: RetainedRunIdentity,
    operation_extent: String,
    environment: RetainedEnvironment,
    composition: CompositionExtent,
}

impl RunSubject {
    fn qualifier(&self) -> String {
        format!(
            "{} on {}; retained historical execution; {}",
            self.operation_extent,
            self.environment.qualifier(),
            self.composition.qualifier(),
        )
    }

    fn markdown_cell(&self) -> String {
        let (maturity, target) = self.cell.manual_shell();
        format!("[{maturity}, {}]({target})", self.qualifier())
    }

    /// Announces whether a fresh rerun used the historical row without
    /// converting either outcome into the other.
    pub(crate) fn report_fresh_boundary(&self, boundary: &MeasurementBoundary) {
        let differences = self.environment.differences(boundary);
        if differences.is_empty() {
            eprintln!(
                "{:?}: fresh rerun matches retained historical environment {:?}",
                self.cell, self.identity,
            );
        } else {
            eprintln!(
                "{:?}: fresh rerun differs from retained historical environment {:?}:\n{}",
                self.cell,
                self.identity,
                differences.join("\n"),
            );
        }
    }

    /// Composition extent the actual run test validates.
    pub(crate) const fn composition(&self) -> CompositionExtent {
        self.composition
    }
}

/// Derives F32 evidence from the routed tables and checked retained record.
pub(crate) fn f32_subject() -> Result<RunSubject, String> {
    let record = crate::envelope::RETAINED_EXECUTION;
    let identity = RetainedRunIdentity::parse(record.identity)?;
    if identity != RetainedRunIdentity::F32RealizationRecord {
        return Err(format!(
            "the F32 suite resolved the wrong identity {identity:?}"
        ));
    }
    if record.identity != crate::retained_record::RECORD_DIRECTORY {
        return Err(format!(
            "the F32 identity {:?} does not name RECORD_DIRECTORY {:?}",
            record.identity,
            crate::retained_record::RECORD_DIRECTORY,
        ));
    }
    let retained_rows =
        crate::retained_record::direct_digests().map_err(|error| error.to_string())?;
    let routed_l3: Vec<_> = crate::envelope::CONTRACTION_MEMBERS
        .iter()
        .filter(|member| member.retained_result_sha256.is_some())
        .collect();
    if routed_l3.len() != 5 {
        return Err(format!(
            "the routed F32 subject has {} retained L3 cells; expected 5",
            routed_l3.len(),
        ));
    }
    for member in &routed_l3 {
        let cell = crate::envelope::L3_CORRECTNESS_CELLS
            .iter()
            .find(|cell| cell.class == member.class)
            .ok_or_else(|| {
                format!(
                    "routed F32 member {:?} has no retained L3 cell",
                    member.class,
                )
            })?;
        let row = retained_rows
            .iter()
            .find(|row| row.id == cell.id)
            .ok_or_else(|| format!("retained F32 record has no direct row {:?}", cell.id))?;
        if (row.m, row.n, row.k, row.result_sha256.as_str())
            != (cell.m, cell.n, cell.k, cell.result_sha256)
            || member.retained_result_sha256 != Some(cell.result_sha256)
        {
            return Err(format!(
                "routed F32 member {:?}, L3 cell {:?}, and retained direct row disagree",
                member.class, cell.id,
            ));
        }
    }
    let matrix_cases = crate::envelope::REDUCTION_CLASSES.len()
        * crate::envelope::PLAN_ROLES.len()
        * crate::publication::proof::serial_sum_case_count();
    Ok(RunSubject {
        cell: ConformanceCell::F32,
        identity,
        operation_extent: format!(
            "serial-sum and contraction device runs ({matrix_cases} routed cases plus {} retained L3 cells)",
            routed_l3.len(),
        ),
        environment: record.environment,
        composition: CompositionExtent::RoutedArtifact,
    })
}

/// Derives BF16 evidence from its semantic program, corpus, and retained output.
pub(crate) fn bf16_subject() -> Result<RunSubject, String> {
    bf16_subject_for(crate::bf16_vertical::RETAINED_EXECUTION)
}

fn bf16_subject_for(record: RetainedBf16Execution) -> Result<RunSubject, String> {
    let identity = RetainedRunIdentity::parse(record.identity)?;
    if identity != RetainedRunIdentity::Bf16Vertical {
        return Err(format!(
            "the BF16 suite resolved the wrong identity {identity:?}"
        ));
    }
    let corpus_len = crate::bf16_vertical::corpus().len();
    if corpus_len != record.observed.len() {
        return Err(format!(
            "the BF16 corpus has {corpus_len} cases but retained execution {identity:?} has {} observed results",
            record.observed.len(),
        ));
    }
    let key =
        tiler_ir::semantic::InputKey::new("operand").expect("the retained BF16 input key is valid");
    let program = crate::bf16_vertical::semantic_program(
        &key,
        u64::try_from(corpus_len).expect("the BF16 corpus length fits u64"),
    );
    let mut operations = BTreeSet::new();
    for operation in program.operations() {
        let operation_key = operation.key().to_string();
        let family = match operation_key.as_str() {
            "tiler::constant-bf16@1" => "constant",
            "tiler::multiply-bf16@1" => "multiply",
            "tiler::add-bf16@1" => "add",
            other => {
                return Err(format!(
                    "retained BF16 execution {identity:?} reaches unrecorded operation {other:?}"
                ));
            }
        };
        operations.insert(family);
    }
    let expected: BTreeSet<_> = ["add", "constant", "multiply"].into_iter().collect();
    if operations != expected {
        return Err(format!(
            "retained BF16 execution {identity:?} reaches operation families {operations:?}"
        ));
    }
    let count = match corpus_len {
        15 => "fifteen".to_owned(),
        16 => "sixteen".to_owned(),
        other => other.to_string(),
    };
    Ok(RunSubject {
        cell: ConformanceCell::Bf16,
        identity,
        operation_extent: format!("constant/multiply/add over {count} hand-derived cases"),
        environment: record.environment,
        composition: CompositionExtent::HandAssembledBf16,
    })
}

/// Validates a fresh BF16 result against retained execution.
pub(crate) fn validate_fresh_bf16(
    subject: &RunSubject,
    boundary: &MeasurementBoundary,
    observed: &[u16],
) {
    assert_eq!(subject.cell, ConformanceCell::Bf16);
    assert_eq!(
        observed,
        crate::bf16_vertical::RETAINED_EXECUTION.observed,
        "fresh BF16 device output disagrees with retained execution {:?}",
        subject.identity,
    );
    subject.report_fresh_boundary(boundary);
}

/// Validates the fresh routed F32 matrix against the executable subject.
pub(crate) fn validate_fresh_f32_matrix(
    subject: &RunSubject,
    boundary: &MeasurementBoundary,
    observed_cases: usize,
) {
    assert_eq!(subject.cell, ConformanceCell::F32);
    let expected_cases = crate::envelope::REDUCTION_CLASSES.len()
        * crate::envelope::PLAN_ROLES.len()
        * crate::publication::proof::serial_sum_case_count();
    assert_eq!(
        observed_cases, expected_cases,
        "fresh F32 routed matrix did not execute the retained subject's case population",
    );
    assert_eq!(subject.composition, CompositionExtent::RoutedArtifact);
    subject.report_fresh_boundary(boundary);
}

fn columns(line: &str) -> Result<Vec<&str>, String> {
    if !line.starts_with('|') || !line.ends_with('|') {
        return Err(format!("Markdown table row lacks boundary pipes: {line:?}"));
    }
    Ok(line.trim_matches('|').split('|').map(str::trim).collect())
}

struct PhysicalTable<'a> {
    header: Vec<&'a str>,
    rows: Vec<Vec<&'a str>>,
}

fn physical_table(document: &str) -> Result<PhysicalTable<'_>, String> {
    let lines: Vec<&str> = document.lines().collect();
    let headers: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            columns(line)
                .ok()
                .filter(|found| found.as_slice() == PHYSICAL_HEADER)
                .map(|_| index)
        })
        .collect();
    let [header_index] = headers.as_slice() else {
        return Err(format!(
            "expected exactly one physical/execution table header, found {}",
            headers.len(),
        ));
    };
    let header = columns(lines[*header_index])?;
    let separator = lines
        .get(header_index + 1)
        .ok_or_else(|| "the physical/execution header has no separator row".to_owned())?;
    let separator_columns = columns(separator)?;
    if separator_columns.len() != PHYSICAL_HEADER.len()
        || separator_columns.iter().any(|column| *column != "---")
    {
        return Err(format!(
            "the physical/execution separator has {} columns and values {separator_columns:?}; expected {} `---` columns",
            separator_columns.len(),
            PHYSICAL_HEADER.len(),
        ));
    }
    let mut rows = Vec::new();
    for line in lines.iter().skip(header_index + 2) {
        if !line.starts_with('|') {
            break;
        }
        let row = columns(line)?;
        if row.len() != PHYSICAL_HEADER.len() {
            return Err(format!(
                "physical/execution row {:?} has {} columns; the exact header requires {}",
                row.first().copied().unwrap_or_default(),
                row.len(),
                PHYSICAL_HEADER.len(),
            ));
        }
        rows.push(row);
    }
    if rows.is_empty() {
        return Err("the physical/execution table has no data rows".to_owned());
    }
    Ok(PhysicalTable { header, rows })
}

fn compare_document(document: &str, subjects: &[RunSubject]) -> Result<(), String> {
    let table = physical_table(document)?;
    let conformance_index = table
        .header
        .iter()
        .position(|column| *column == "Conformance evidence")
        .ok_or_else(|| "the exact header has no Conformance evidence column".to_owned())?;
    for cell in ConformanceCell::ALL {
        let matching: Vec<_> = table
            .rows
            .iter()
            .filter(|row| row.first().copied() == Some(cell.row_label()))
            .collect();
        let [row] = matching.as_slice() else {
            return Err(format!(
                "expected exactly one physical/execution row for {}, found {}",
                cell.row_label(),
                matching.len(),
            ));
        };
        let matching_subjects: Vec<_> = subjects
            .iter()
            .filter(|subject| subject.cell == cell)
            .collect();
        let [subject] = matching_subjects.as_slice() else {
            return Err(format!(
                "expected exactly one retained subject for {}, found {}",
                cell.row_label(),
                matching_subjects.len(),
            ));
        };
        let actual = row[conformance_index];
        let expected = subject.markdown_cell();
        if actual != expected {
            return Err(format!(
                "{} Conformance evidence Markdown cell disagrees with retained execution {:?}:\n  actual: {actual:?}\nexpected: {expected:?}",
                cell.row_label(),
                subject.identity,
            ));
        }
    }
    Ok(())
}

fn subjects() -> Result<[RunSubject; variant_count::<ConformanceCell>()], String> {
    Ok([f32_subject()?, bf16_subject()?])
}

#[test]
fn the_retained_subject_population_is_total_unique_and_source_linked() {
    let subjects = subjects().unwrap_or_else(|failure| panic!("{failure}"));
    let cells: BTreeSet<_> = subjects.iter().map(|subject| subject.cell).collect();
    assert_eq!(
        cells,
        ConformanceCell::ALL.into_iter().collect(),
        "retained subjects do not cover the typed ledger-cell population",
    );
    let identities: BTreeSet<_> = subjects.iter().map(|subject| subject.identity).collect();
    assert_eq!(
        identities.len(),
        subjects.len(),
        "one retained execution identity is assigned to more than one ledger cell",
    );
    eprintln!(
        "conformance ledger census: {} typed cells, {} source-linked retained executions",
        cells.len(),
        identities.len(),
    );
}

#[test]
fn the_manual_conformance_cells_match_the_executed_run_subjects() {
    let subjects = subjects().unwrap_or_else(|failure| panic!("{failure}"));
    compare_document(include_str!("../../../docs/dtype-support.md"), &subjects)
        .unwrap_or_else(|failure| panic!("{failure}"));
}

#[test]
fn an_earlier_decoy_physical_table_is_refused() {
    let subjects = subjects().expect("retained subjects resolve");
    let header = format!("| {} |", PHYSICAL_HEADER.join(" | "));
    let separator = format!("| {} |", ["---"; 10].join(" | "));
    let document = format!(
        "{header}\n{separator}\n\n{}",
        include_str!("../../../docs/dtype-support.md"),
    );
    let failure = compare_document(&document, &subjects).expect_err("a decoy table must fail");
    assert_eq!(
        failure,
        "expected exactly one physical/execution table header, found 2",
    );
    eprintln!("decoy-table perturbation: {failure}");
}

#[test]
fn an_extra_physical_row_column_is_refused() {
    let subjects = subjects().expect("retained subjects resolve");
    let document = include_str!("../../../docs/dtype-support.md")
        .lines()
        .map(|line| {
            if line.starts_with("| IEEE `f32` |") {
                format!("{} shifted |", line.trim_end())
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let failure = compare_document(&document, &subjects).expect_err("an extra column must fail");
    assert!(failure.contains("has 11 columns; the exact header requires 10"));
    eprintln!("extra-column perturbation: {failure}");
}

#[test]
fn a_wrong_conformance_link_target_is_refused() {
    let subjects = subjects().expect("retained subjects resolve");
    let subject = subjects
        .iter()
        .find(|subject| subject.cell == ConformanceCell::F32)
        .expect("F32 is in the typed population");
    let correct = subject.markdown_cell();
    let wrong = correct.replace("#ieee-f32", "#wrong-target");
    let document = include_str!("../../../docs/dtype-support.md").replacen(&correct, &wrong, 1);
    let failure = compare_document(&document, &subjects).expect_err("a wrong target must fail");
    assert!(failure.contains("Conformance evidence Markdown cell disagrees"));
    assert!(failure.contains("#wrong-target"));
    eprintln!("wrong-target perturbation: {failure}");
}

#[test]
fn a_retained_environment_change_moves_the_derived_cell() {
    let mut subjects = subjects().expect("retained subjects resolve");
    let f32 = subjects
        .iter_mut()
        .find(|subject| subject.cell == ConformanceCell::F32)
        .expect("F32 is in the typed population");
    f32.environment.sdk = "macosx 99.0 build drifted-sdk";
    let failure = compare_document(include_str!("../../../docs/dtype-support.md"), &subjects)
        .expect_err("a retained SDK change must move the derived qualifier");
    assert!(failure.contains("SDK 99.0 drifted-sdk"));
    eprintln!("retained-environment perturbation: {failure}");
}

#[test]
fn trailing_text_after_the_conformance_link_is_refused() {
    let subjects = subjects().expect("retained subjects resolve");
    let subject = subjects
        .iter()
        .find(|subject| subject.cell == ConformanceCell::Bf16)
        .expect("BF16 is in the typed population");
    let correct = subject.markdown_cell();
    let document = include_str!("../../../docs/dtype-support.md").replacen(
        &correct,
        &format!("{correct} stale-suffix"),
        1,
    );
    let failure = compare_document(&document, &subjects).expect_err("trailing text must fail");
    assert!(failure.contains("stale-suffix"));
    eprintln!("trailing-suffix perturbation: {failure}");
}

#[test]
fn an_ungoverned_retained_run_identity_is_refused() {
    let mut record = crate::bf16_vertical::RETAINED_EXECUTION;
    record.identity = "no-such-run@deadbeef";
    let failure =
        bf16_subject_for(record).expect_err("an arbitrary run spelling must not become evidence");
    assert_eq!(
        failure,
        "retained execution identity \"no-such-run@deadbeef\" is not governed by this conformance corpus",
    );
    eprintln!("bogus-identity perturbation: {failure}");
}

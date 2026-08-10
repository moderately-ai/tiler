//! The private comparison between executed-run declarations and the manually
//! owned dtype ledger.
//!
//! This module derives only the qualifier of the `Conformance evidence` cell.
//! It deliberately does not read the maturity phrase before that qualifier,
//! inspect the other eight physical/execution columns, or assign one of the
//! repository's evidence classes. Those remain statements by their owning
//! authorities. The document remains hand-edited; this check only makes a
//! disagreement between that prose and the retained run declarations loud.

use core::mem::variant_count;
use std::collections::BTreeSet;

/// A dtype-ledger row for which this crate retains executed evidence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ConformanceCell {
    /// The IEEE binary32 row.
    F32,
    /// The BF16 row.
    Bf16,
}

impl ConformanceCell {
    /// Every cell this comparison owns, sized from the type so widening the
    /// vocabulary without declaring the new member is a build error.
    const ALL: [Self; variant_count::<Self>()] = [Self::F32, Self::Bf16];

    /// The exact first-column spelling in the physical/execution matrix.
    const fn row_label(self) -> &'static str {
        match self {
            Self::F32 => "IEEE `f32`",
            Self::Bf16 => "BF16",
        }
    }
}

/// The exact environment row an executed declaration is bounded to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EnvironmentRow {
    /// The measured device name.
    pub(crate) device: &'static str,
    /// The Metal GPU family reported by that device.
    pub(crate) gpu_family: &'static str,
    /// The host architecture.
    pub(crate) architecture: &'static str,
    /// The operating-system version and build.
    pub(crate) os: &'static str,
    /// The offline Metal compiler version.
    pub(crate) offline_compiler: &'static str,
    /// The SDK version and build.
    pub(crate) sdk: &'static str,
}

impl EnvironmentRow {
    /// The shared measured row for the two retained declarations at this base.
    pub(crate) const APPLE9_2026_08_07: Self = Self {
        device: "Apple M4 Max",
        gpu_family: "Apple9",
        architecture: "arm64",
        os: "macOS 27.0 26A5388g",
        offline_compiler: "metal 32023.921",
        sdk: "SDK 27.0 26A5388f",
    };

    /// Renders only observed fields. In particular, Xcode is not introduced:
    /// the retained-record comparison deliberately does not compare it.
    fn qualifier(self) -> String {
        format!(
            "{} / {} / {} / {} / {} / {}",
            self.device,
            self.gpu_family,
            self.os,
            self.offline_compiler,
            self.sdk,
            self.architecture,
        )
    }
}

/// Whether the measured half represented by a retained declaration ran.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MeasuredHalf {
    /// The device half ran and its comparison succeeded on the declared row.
    Ran,
    /// The deterministic half ran but the device half was unavailable.
    #[allow(
        dead_code,
        reason = "the unavailable outcome must remain representable so a retained declaration cannot collapse it into Ran"
    )]
    Unavailable,
}

impl MeasuredHalf {
    const fn qualifier(self) -> &'static str {
        match self {
            Self::Ran => "measured half ran",
            Self::Unavailable => "measured half was unavailable",
        }
    }
}

/// Which composition boundary the retained run crossed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompositionExtent {
    /// The F32 routed evidence crossed planning, packaging, and routing.
    RoutedArtifact,
    /// The BF16 vertical was assembled below the refusing request boundary.
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

/// One aggregate retained declaration for one manually written ledger cell.
///
/// The cell is the identity because the comparison has one answer per ledger
/// row. Run identifiers are immutable provenance within that answer. A second
/// declaration for the same cell is a disagreement and fails rather than being
/// ordered, selected, or silently combined.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CellDeclaration {
    /// The manual ledger row this declaration constrains.
    pub(crate) cell: ConformanceCell,
    /// Stable identifiers for the executed records contributing to the cell.
    pub(crate) run_ids: &'static [&'static str],
    /// Exact operation and corpus extent established by those records.
    pub(crate) operation_extent: &'static str,
    /// The environment outside which the declaration makes no claim.
    pub(crate) environment: EnvironmentRow,
    /// Whether the measured half ran on that row.
    pub(crate) measured_half: MeasuredHalf,
    /// The composition boundary the run crossed.
    pub(crate) composition: CompositionExtent,
}

impl CellDeclaration {
    /// Derives the prose qualifier compared with the hand-maintained cell.
    fn qualifier(self) -> String {
        format!(
            "{} on {}; {}; {}",
            self.operation_extent,
            self.environment.qualifier(),
            self.measured_half.qualifier(),
            self.composition.qualifier(),
        )
    }
}

/// The complete retained population, taken from declarations beside the runs.
fn declarations() -> [CellDeclaration; variant_count::<ConformanceCell>()] {
    [
        crate::envelope::LEDGER_CELL,
        crate::bf16_vertical::LEDGER_CELL,
    ]
}

/// Returns the qualifier from the last column of one physical/execution row.
///
/// The phrase before the first comma is intentionally ignored: it is the
/// manually assigned maturity, and this crate has no authority to stamp it.
fn manual_qualifier(document: &str, cell: ConformanceCell) -> Result<&str, String> {
    const PHYSICAL_HEADER: &str = "| Family | Physical carrier and encoding |";
    let Some((_, physical_matrix)) = document.split_once(PHYSICAL_HEADER) else {
        return Err("the physical/execution matrix header is absent".to_owned());
    };
    let prefix = format!("| {} |", cell.row_label());
    let rows: Vec<&str> = physical_matrix
        .lines()
        .skip(1)
        .take_while(|line| line.starts_with('|'))
        .filter(|line| line.starts_with(&prefix))
        .collect();
    let [row] = rows.as_slice() else {
        return Err(format!(
            "expected one physical/execution row for {}, found {}",
            cell.row_label(),
            rows.len(),
        ));
    };
    let columns: Vec<&str> = row.trim_matches('|').split('|').map(str::trim).collect();
    let Some(conformance) = columns.last() else {
        return Err(format!("{} has no columns", cell.row_label()));
    };
    let Some(label_end) = conformance.find("](") else {
        return Err(format!(
            "{} conformance cell is not one markdown link: {conformance}",
            cell.row_label(),
        ));
    };
    let Some(label) = conformance.get(1..label_end) else {
        return Err(format!(
            "{} conformance cell has no link label: {conformance}",
            cell.row_label(),
        ));
    };
    let Some((_, qualifier)) = label.split_once(", ") else {
        return Err(format!(
            "{} conformance cell has no run-derived qualifier: {conformance}",
            cell.row_label(),
        ));
    };
    Ok(qualifier)
}

#[test]
fn the_retained_declaration_population_is_total_unique_and_executed() {
    let declarations = declarations();
    assert_eq!(
        declarations.len(),
        variant_count::<ConformanceCell>(),
        "the typed conformance-cell population changed without a declaration",
    );

    let mut cells = BTreeSet::new();
    let mut run_ids = BTreeSet::new();
    for declaration in declarations {
        assert!(
            cells.insert(declaration.cell),
            "two retained declarations disagree about the {:?} ledger cell",
            declaration.cell,
        );
        assert!(
            !declaration.run_ids.is_empty(),
            "the {:?} cell names no executed run",
            declaration.cell,
        );
        assert_eq!(
            declaration.measured_half,
            MeasuredHalf::Ran,
            "the {:?} cell names a measured half that did not run",
            declaration.cell,
        );
        for run_id in declaration.run_ids {
            assert!(
                !run_id.is_empty(),
                "the {:?} cell has an empty run id",
                declaration.cell
            );
            assert!(
                run_ids.insert(run_id),
                "executed run id {run_id:?} is assigned to more than one ledger cell",
            );
        }
    }
    assert_eq!(
        cells,
        ConformanceCell::ALL.into_iter().collect(),
        "the retained declarations do not cover the typed ledger-cell population",
    );
    eprintln!(
        "conformance ledger census: {} cells derived from {} executed run records",
        cells.len(),
        run_ids.len(),
    );
}

#[test]
fn the_manual_conformance_evidence_qualifiers_match_the_executed_runs() {
    let document = include_str!("../../../docs/dtype-support.md");
    for declaration in declarations() {
        let expected = declaration.qualifier();
        let actual = manual_qualifier(document, declaration.cell)
            .unwrap_or_else(|failure| panic!("{failure}"));
        assert_eq!(
            actual,
            expected,
            "{} Conformance evidence disagrees with executed run records {:?}; only the prose after the manually owned maturity phrase is compared",
            declaration.cell.row_label(),
            declaration.run_ids,
        );
    }
}

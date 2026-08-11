//! Audits raw-minimum and noise-plateau stability across retained tree widths.
//!
//! This binary is intentionally device-free. It validates the exact custody,
//! environment, schema, and finite population of the three already-opened
//! primary/repeat matrices before deriving one cross-run stability metric. It
//! fits no selector and supplies no production input.

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

const RESULT_HEADER: &str = "rows\tcontributors\telements\tstrategy\tpartitions\tper_partition\tproduction\twidest_workgroup\tthreadgroup_bytes\tencoders\treps\tbatch\tsubmit_min_us\tsubmit_p50_us\tsubmit_p90_us\tsubmit_stddev_us\tbatch_min_us\tbatch_p50_us\tbatch_p90_us\tbatch_stddev_us\tamortized_min_us\tamortized_p50_us\tamortized_stddev_us\tstatus";
const REPETITIONS: f64 = 30.0;

const SHAPE_ROWS: &[u64] = &[4, 1_024, 2_048, 16_384];
const SHAPE_ANCHORS: &[u64] = &[780, 1_042];
const SHAPE_FIT: &[u64] = &[756, 779, 840, 1_018, 1_020];
const SHAPE_HELD: &[u64] = &[768, 781, 960, 1_022, 1_046];
const SHAPE_ALL: &[u64] = &[
    780, 1_042, 756, 779, 840, 1_018, 1_020, 768, 781, 960, 1_022, 1_046,
];

const INTERACTION_ROWS: &[u64] = &[8, 528, 1_056, 2_112, 8_192];
const INTERACTION_ANCHORS: &[u64] = &[780, 1_042];
const INTERACTION_FIT: &[u64] = &[774, 783, 900, 1_006, 1_082, 1_280];
const INTERACTION_HELD: &[u64] = &[775, 785, 899, 1_008, 1_094, 1_282];
const INTERACTION_ALL: &[u64] = &[
    780, 1_042, 774, 783, 900, 1_006, 1_082, 1_280, 775, 785, 899, 1_008, 1_094, 1_282,
];

const TABLE_ROWS: &[u64] = &[16, 384, 1_536, 6_144, 12_288];
const TABLE_ANCHORS: &[u64] = &[1_024, 1_729];
const TABLE_FIT: &[u64] = &[1_080, 1_215, 1_320, 1_512, 1_638, 1_890];
const TABLE_HELD: &[u64] = &[1_050, 1_155, 1_274, 1_430, 1_575, 1_925];
const TABLE_ALL: &[u64] = &[
    1_080, 1_215, 1_320, 1_512, 1_638, 1_890, 1_050, 1_155, 1_274, 1_430, 1_575, 1_925, 1_024,
    1_729,
];

const COMMON_ENVIRONMENT: &[(&str, &str)] = &[
    ("environment.os.product", "macOS"),
    ("environment.os.version", "27.0"),
    ("environment.os.build", "26A5388g"),
    ("environment.architecture", "arm64"),
    ("environment.device", "Apple M4 Max"),
    ("environment.apple_gpu_family", "apple9"),
    ("environment.cpu.logical_cores", "14"),
    (
        "environment.developer_dir",
        "/Applications/Xcode.app/Contents/Developer",
    ),
    (
        "environment.default_developer_dir",
        "/Applications/Xcode-beta.app/Contents/Developer",
    ),
    ("environment.xcode", "Xcode 26.6 Build version 17F113"),
    ("environment.sdk.macosx.version", "26.5"),
    ("environment.sdk.macosx.build", "25F70"),
    (
        "environment.offline_compiler",
        "Apple metal version 32023.883 (metalfe-32023.883)",
    ),
    (
        "environment.offline_linker",
        "AIR-LLD 32023.883 (metalfe-32023.883) (compatible with legacy metallib linker)",
    ),
    (
        "environment.rustc",
        "rustc 1.99.0-nightly (eff8269f7 2026-07-18)",
    ),
    (
        "environment.toolchain",
        "nightly-2026-07-19-aarch64-apple-darwin",
    ),
];

const SHAPE_ENVIRONMENT: &[(&str, &str)] = &[
    ("environment.date_utc.primary", "2026-08-11T08:39:16Z"),
    ("environment.date_utc.repeat", "2026-08-11T08:46:42Z"),
    (
        "environment.repository_base",
        "b30e384497682c91771fcf93c5ce6854054d39a3",
    ),
    (
        "host.occupancy",
        "coordinator-reserved quiet window; primary then repeat ran sequentially; zero Cargo, rustc, or make processes before and after each timed run",
    ),
    ("host.load_before.primary", "3.62 6.52 7.58"),
    ("host.load_after.primary", "2.97 4.31 6.03"),
    ("host.load_before.repeat", "3.10 4.22 5.95"),
    ("host.load_after.repeat", "4.09 4.06 5.15"),
    ("matrix.rows", "4,1024,2048,16384"),
    ("matrix.anchor_contributors", "780,1042"),
    ("matrix.fit_contributors", "756,779,840,1018,1020"),
    ("matrix.held_out_contributors", "768,781,960,1022,1046"),
    ("matrix.shapes", "48"),
    ("matrix.arithmetic_variants_per_run", "616"),
    (
        "measurement.metric",
        "wall-clock commit-to-completed difference quotient",
    ),
    ("measurement.warmup", "8"),
    ("measurement.repetitions", "30"),
    ("measurement.batch", "64"),
    (
        "measurement.harness_relation",
        "retained source and lock digests identify the harness build inputs; the before/after executable digest identifies the unretained timed binary; no byte-identical rebuild claim",
    ),
    (
        "measurement.source_threadgroup_bytes",
        "4 * participants bytes",
    ),
    (
        "measurement.result_threadgroup_bytes",
        "prepared entry static allocation, source request rounded up to 16 bytes",
    ),
    (
        "timed.executable.sha256",
        "56fa8152ff5f5ff53c225398082e9f20e808df8208fc7a617023bb9912a34a59",
    ),
    ("timed.executable.size_bytes", "7807232"),
    ("timed.executable.mtime_utc", "2026-08-11T08:16:16Z"),
    (
        "timed.executable.retention",
        "digest and observed metadata only; target/release build product is not checked in",
    ),
];

const INTERACTION_ENVIRONMENT: &[(&str, &str)] = &[
    ("environment.date_utc.primary_start", "2026-08-11T10:02:50Z"),
    ("environment.date_utc.primary_end", "2026-08-11T10:06:56Z"),
    ("environment.date_utc.repeat_start", "2026-08-11T10:07:04Z"),
    ("environment.date_utc.repeat_end", "2026-08-11T10:11:13Z"),
    (
        "environment.repository_base",
        "b0c41639f9ad266879ef52dfaee8de5e35eb47f9",
    ),
    (
        "host.occupancy",
        "coordinator-reserved quiet window; primary then repeat ran sequentially; zero cargo, rustc, rustdoc, nextest, clippy-driver, or make processes before and after each timed run",
    ),
    ("host.load_before.primary", "4.33 3.79 3.75"),
    ("host.load_after.primary", "3.35 3.64 3.70"),
    ("host.load_before.repeat", "3.14 3.58 3.68"),
    ("host.load_after.repeat", "2.60 3.21 3.50"),
    ("matrix.rows", "8,528,1056,2112,8192"),
    ("matrix.anchor_contributors", "780,1042"),
    ("matrix.fit_contributors", "774,783,900,1006,1082,1280"),
    ("matrix.held_out_contributors", "775,785,899,1008,1094,1282"),
    ("matrix.shapes", "70"),
    ("matrix.arithmetic_variants_per_run", "625"),
    ("matrix.source_max_threadgroup_bytes", "2564"),
    ("matrix.prepared_max_threadgroup_bytes", "2576"),
    (
        "measurement.metric",
        "wall-clock commit-to-completed difference quotient",
    ),
    ("measurement.warmup", "8"),
    ("measurement.repetitions", "30"),
    ("measurement.batch", "64"),
    (
        "measurement.harness_relation",
        "retained source and lock digests identify the harness build inputs; the before/after executable digest identifies the unretained timed binary; no byte-identical rebuild claim",
    ),
    (
        "timed.executable.sha256",
        "f9bac263c0a140f5843667550a0b5373cc7a21c784dbefb79f837bf4fc6c7b29",
    ),
    ("timed.executable.size_bytes", "7823936"),
    ("timed.executable.mtime_utc", "2026-08-11T10:00:57Z"),
    (
        "timed.executable.retention",
        "digest and observed metadata only; target/release build product is not checked in",
    ),
];

const TABLE_ENVIRONMENT: &[(&str, &str)] = &[
    ("environment.date_utc.primary_start", "2026-08-11T11:22:31Z"),
    ("environment.date_utc.primary_end", "2026-08-11T11:40:38Z"),
    ("environment.date_utc.repeat_start", "2026-08-11T11:40:50Z"),
    ("environment.date_utc.repeat_end", "2026-08-11T11:58:38Z"),
    (
        "environment.repository_base",
        "f7287e1793b49008882ab5717e4b244a13ef34ad",
    ),
    (
        "host.occupancy",
        "coordinator-reserved quiet window; primary then repeat ran sequentially; zero cargo, rustc, rustdoc, nextest, clippy-driver, or make processes before and after each timed run",
    ),
    ("host.load_before.primary", "3.73 4.07 4.37"),
    ("host.load_after.primary", "2.75 2.72 3.18"),
    ("host.load_before.repeat", "2.94 2.77 3.19"),
    ("host.load_after.repeat", "3.42 3.30 3.19"),
    ("matrix.rows", "16,384,1536,6144,12288"),
    ("matrix.fit_contributors", "1080,1215,1320,1512,1638,1890"),
    (
        "matrix.held_out_contributors",
        "1050,1155,1274,1430,1575,1925",
    ),
    ("matrix.anchor_contributors", "1024,1729"),
    ("matrix.shapes", "70"),
    ("matrix.arithmetic_variants_per_run", "1265"),
    ("matrix.fit_variants_per_run", "760"),
    ("matrix.held_out_variants_per_run", "430"),
    ("matrix.anchor_variants_per_run", "75"),
    ("matrix.source_max_threadgroup_bytes", "3780"),
    ("matrix.prepared_max_threadgroup_bytes", "3792"),
    ("matrix.maximum_elements", "23654400"),
    (
        "measurement.metric",
        "wall-clock commit-to-completed difference quotient",
    ),
    ("measurement.warmup", "8"),
    ("measurement.repetitions", "30"),
    ("measurement.batch", "64"),
    ("measurement.source_threadgroup_bytes", "4 * participants"),
    (
        "measurement.result_threadgroup_bytes",
        "prepared entry static allocation, source request rounded up to 16 bytes",
    ),
    ("analysis.table_capacity", "30"),
    ("analysis.signature_encoding_bytes", "30"),
    ("analysis.noise_band", "2 * (SE_a + SE_b)"),
    (
        "measurement.harness_relation",
        "retained source and lock digests identify the harness build inputs; the before/after executable digest identifies the unretained timed binary; no byte-identical rebuild claim",
    ),
    (
        "timed.executable.sha256",
        "5d7d2d13daffbf05d002c2cf7438522bf303fd688f9c69b1017eebf5a427938e",
    ),
    ("timed.executable.size_bytes", "7823968"),
    ("timed.executable.mtime_utc", "2026-08-11T11:18:32Z"),
    (
        "timed.executable.retention",
        "digest and observed metadata only; target/release build product is not checked in",
    ),
];

const SHAPE_DIGEST_KEYS: &[&str] = &[
    "hash.main",
    "hash.regions",
    "hash.buffer",
    "hash.analysis",
    "hash.cargo_toml",
    "hash.cargo_lock",
    "hash.compiler_physical",
    "hash.compiler_measured_cost",
    "hash.sweep",
    "hash.repeat",
];

const EXTENDED_DIGEST_KEYS: &[&str] = &[
    "hash.main",
    "hash.regions",
    "hash.buffer",
    "hash.analysis_wrapper",
    "hash.analysis_shared",
    "hash.cargo_toml",
    "hash.cargo_lock",
    "hash.compiler_physical",
    "hash.compiler_measured_cost",
    "hash.compiler_target",
    "hash.metal_declaration",
    "hash.sweep",
    "hash.repeat",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Study {
    ShapeAware,
    Interactions,
    Table,
}

impl Study {
    const fn key(self) -> &'static str {
        match self {
            Self::ShapeAware => "shape-aware",
            Self::Interactions => "interactions",
            Self::Table => "table",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Role {
    Anchor,
    Fit,
    Held,
}

impl Role {
    const fn key(self) -> &'static str {
        match self {
            Self::Anchor => "anchor",
            Self::Fit => "fit",
            Self::Held => "held",
        }
    }
}

struct StudySpec {
    study: Study,
    result_spike: &'static str,
    mode: &'static str,
    rows: &'static [u64],
    anchors: &'static [u64],
    fit: &'static [u64],
    held: &'static [u64],
    all: &'static [u64],
    expected_cells: usize,
    expected_variants: usize,
    expected_values: &'static [(&'static str, &'static str)],
    digest_keys: &'static [&'static str],
    noise_multiplier: f64,
    noise_provenance: &'static str,
}

impl StudySpec {
    fn role(&self, contributors: u64) -> Role {
        if self.anchors.contains(&contributors) {
            Role::Anchor
        } else if self.fit.contains(&contributors) {
            Role::Fit
        } else if self.held.contains(&contributors) {
            Role::Held
        } else {
            panic!(
                "{}: P{contributors} has no frozen population role",
                self.study.key()
            );
        }
    }
}

fn study_specs() -> [StudySpec; 3] {
    [
        StudySpec {
            study: Study::ShapeAware,
            result_spike: "reduction-shape-aware-tree-width",
            mode: "ShapeAwareTreeWidth",
            rows: SHAPE_ROWS,
            anchors: SHAPE_ANCHORS,
            fit: SHAPE_FIT,
            held: SHAPE_HELD,
            all: SHAPE_ALL,
            expected_cells: 48,
            expected_variants: 616,
            expected_values: SHAPE_ENVIRONMENT,
            digest_keys: SHAPE_DIGEST_KEYS,
            noise_multiplier: 2.0,
            noise_provenance: "shape-aware frozen ticket + hash.analysis=806e2f41cdfdbc7c02fe936a958b663413f8d6271bdf447ca89f00213e6e6d72",
        },
        StudySpec {
            study: Study::Interactions,
            result_spike: "reduction-tree-width-interactions",
            mode: "TreeWidthInteractions",
            rows: INTERACTION_ROWS,
            anchors: INTERACTION_ANCHORS,
            fit: INTERACTION_FIT,
            held: INTERACTION_HELD,
            all: INTERACTION_ALL,
            expected_cells: 70,
            expected_variants: 625,
            expected_values: INTERACTION_ENVIRONMENT,
            digest_keys: EXTENDED_DIGEST_KEYS,
            noise_multiplier: 2.0,
            noise_provenance: "interaction frozen ticket + hash.analysis_shared=c4f2f807fb90c8b31743d04b362dc4945002350e893ee7b2e0b1fb4c961d8741",
        },
        StudySpec {
            study: Study::Table,
            result_spike: "reduction-tree-width-table",
            mode: "TreeWidthTable",
            rows: TABLE_ROWS,
            anchors: TABLE_ANCHORS,
            fit: TABLE_FIT,
            held: TABLE_HELD,
            all: TABLE_ALL,
            expected_cells: 70,
            expected_variants: 1_265,
            expected_values: TABLE_ENVIRONMENT,
            digest_keys: EXTENDED_DIGEST_KEYS,
            noise_multiplier: 2.0,
            noise_provenance: "table environment analysis.noise_band=2 * (SE_a + SE_b)",
        },
    ]
}

#[derive(Clone, Copy, Debug)]
struct Row {
    participants: u64,
    p50: f64,
    stddev: f64,
}

impl Row {
    fn standard_error(self) -> f64 {
        self.stddev / REPETITIONS.sqrt()
    }
}

struct Run {
    measured: BTreeMap<(u64, u64, u64), Row>,
    metadata: BTreeMap<String, String>,
}

impl Run {
    fn cell(&self, rows: u64, contributors: u64) -> Vec<Row> {
        self.measured
            .range((rows, contributors, 0)..=(rows, contributors, u64::MAX))
            .map(|(_, row)| *row)
            .collect()
    }

    fn row(&self, rows: u64, contributors: u64, participants: u64) -> Row {
        *self
            .measured
            .get(&(rows, contributors, participants))
            .unwrap_or_else(|| panic!("{rows}x{contributors}: repeated run has no P{participants}"))
    }
}

struct Pair {
    spec: StudySpec,
    primary: Run,
    repeat: Run,
    environment_digest: String,
    primary_digest: String,
    repeat_digest: String,
}

#[allow(
    clippy::struct_excessive_bools,
    reason = "the report retains three separately named directional/conjunctive predicates and exact agreement"
)]
#[derive(Clone)]
struct CellMetric {
    study: Study,
    role: Role,
    rows: u64,
    contributors: u64,
    primary_best: u64,
    repeat_best: u64,
    exact: bool,
    movement: f64,
    primary_plateau: BTreeSet<u64>,
    repeat_plateau: BTreeSet<u64>,
    jaccard: f64,
    primary_containment: f64,
    repeat_containment: f64,
    symmetric_containment: f64,
    primary_in_repeat_regret: f64,
    repeat_in_primary_regret: f64,
    reciprocal_regret: f64,
    primary_in_repeat_plateau: bool,
    repeat_in_primary_plateau: bool,
    reciprocal_plateau: bool,
}

#[derive(Clone, Debug)]
struct Aggregate {
    name: String,
    cells: usize,
    exact: usize,
    movement_upper_median: f64,
    movement_p90: f64,
    movement_max: f64,
    jaccard_lower_median: f64,
    jaccard_min: f64,
    containment_lower_median: f64,
    containment_min: f64,
    primary_in_repeat_plateau: usize,
    repeat_in_primary_plateau: usize,
    reciprocal_plateau: usize,
    regret_upper_median: f64,
    regret_p90: f64,
    regret_max: f64,
    stable: bool,
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments == ["--self-check"] {
        self_check();
        return;
    }
    let (inputs, golden) = match arguments.as_slice() {
        [
            manifest,
            shape_sweep,
            shape_repeat,
            shape_environment,
            interaction_sweep,
            interaction_repeat,
            interaction_environment,
            table_sweep,
            table_repeat,
            table_environment,
        ] => (
            [
                manifest.as_str(),
                shape_sweep.as_str(),
                shape_repeat.as_str(),
                shape_environment.as_str(),
                interaction_sweep.as_str(),
                interaction_repeat.as_str(),
                interaction_environment.as_str(),
                table_sweep.as_str(),
                table_repeat.as_str(),
                table_environment.as_str(),
            ],
            None,
        ),
        [
            manifest,
            shape_sweep,
            shape_repeat,
            shape_environment,
            interaction_sweep,
            interaction_repeat,
            interaction_environment,
            table_sweep,
            table_repeat,
            table_environment,
            check,
            golden,
        ] if check == "--check" => (
            [
                manifest.as_str(),
                shape_sweep.as_str(),
                shape_repeat.as_str(),
                shape_environment.as_str(),
                interaction_sweep.as_str(),
                interaction_repeat.as_str(),
                interaction_environment.as_str(),
                table_sweep.as_str(),
                table_repeat.as_str(),
                table_environment.as_str(),
            ],
            Some(golden.as_str()),
        ),
        _ => panic!(
            "usage: tree-width-label-stability-analysis <inputs.tsv> \
             <shape-sweep.tsv> <shape-repeat.tsv> <shape-environment.tsv> \
             <interaction-sweep.tsv> <interaction-repeat.tsv> <interaction-environment.tsv> \
             <table-sweep.tsv> <table-repeat.tsv> <table-environment.tsv> \
             [--check <analysis.txt>] | --self-check"
        ),
    };
    let report = analyze_inputs(inputs);
    if let Some(path) = golden {
        let expected = fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("{path} does not read: {error}"));
        assert!(
            report == expected,
            "retained stability analysis moved: expected {}, observed {}",
            digest_bytes(expected.as_bytes()),
            digest_bytes(report.as_bytes())
        );
        eprintln!("# exact_replay\tpassed");
    }
    print!("{report}");
}

#[allow(
    clippy::too_many_lines,
    reason = "the retained report writes one fixed schema and verdict from already-derived metrics"
)]
fn analyze_inputs(inputs: [&str; 10]) -> String {
    let manifest = parse_manifest(inputs[0]);
    let (path_groups, remainder) = inputs[1..].as_chunks::<3>();
    assert!(remainder.is_empty(), "the study-path population moved");
    let mut pairs = Vec::new();
    for (spec, paths) in study_specs().into_iter().zip(path_groups) {
        pairs.push(load_pair(spec, paths[0], paths[1], paths[2], &manifest));
    }
    assert_eq!(pairs.len(), 3, "the study-pair population moved");
    let mut metrics = Vec::new();
    for pair in &pairs {
        metrics.extend(metrics_for_pair(pair));
    }
    assert_eq!(metrics.len(), 188, "the combined cell population moved");
    let mut report = String::new();
    writeln!(report, "# analysis\treduction-tree-width-label-stability")
        .expect("writing to String cannot fail");
    writeln!(report, "# validation\tpassed").expect("writing to String cannot fail");
    writeln!(report, "# cells\t{}", metrics.len()).expect("writing to String cannot fail");
    writeln!(report, "# widths_per_run\t2506").expect("writing to String cannot fail");
    writeln!(report, "# measured_rows\t5012").expect("writing to String cannot fail");
    writeln!(
        report,
        "# percentile_indices\tupper_median=N/2; lower_median=(N-1)/2; p90=ceil(0.9*N)-1"
    )
    .expect("writing to String cannot fail");
    writeln!(report, "# stability_bar\texact>=ceil(0.80*N); movement_p90<=1.0; jaccard_lower_median>=0.50; containment_lower_median>=0.75; reciprocal_plateau>=ceil(0.90*N); regret_upper_median<=1.02; regret_p90<=1.10; regret_max<=1.25")
        .expect("writing to String cannot fail");
    writeln!(
        report,
        "# input_manifest_sha256\t{}",
        digest_path(inputs[0])
    )
    .expect("writing to String cannot fail");
    for pair in &pairs {
        writeln!(
            report,
            "# input\t{}\tenvironment={}\tprimary={}\trepeat={}",
            pair.spec.study.key(),
            pair.environment_digest,
            pair.primary_digest,
            pair.repeat_digest
        )
        .expect("writing to String cannot fail");
        writeln!(
            report,
            "# noise_rule\t{}\tmultiplier={:.1}\t{}",
            pair.spec.study.key(),
            pair.spec.noise_multiplier,
            pair.spec.noise_provenance
        )
        .expect("writing to String cannot fail");
    }
    writeln!(report).expect("writing to String cannot fail");
    writeln!(report, "## Per-cell stability").expect("writing to String cannot fail");
    writeln!(report).expect("writing to String cannot fail");
    writeln!(report, "study\trole\tshape\tprimary_best\trepeat_best\texact\tlog2_movement\tprimary_plateau\trepeat_plateau\tjaccard\tprimary_containment\trepeat_containment\tsymmetric_containment\tprimary_in_repeat_regret\trepeat_in_primary_regret\treciprocal_regret\tprimary_in_repeat_plateau\trepeat_in_primary_plateau\treciprocal_plateau")
        .expect("writing to String cannot fail");
    for metric in &metrics {
        writeln!(
            report,
            "{}\t{}\t{}x{}\t{}\t{}\t{}\t{:.9}\t{}\t{}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{}\t{}\t{}",
            metric.study.key(),
            metric.role.key(),
            metric.rows,
            metric.contributors,
            metric.primary_best,
            metric.repeat_best,
            metric.exact,
            metric.movement,
            set_text(&metric.primary_plateau),
            set_text(&metric.repeat_plateau),
            metric.jaccard,
            metric.primary_containment,
            metric.repeat_containment,
            metric.symmetric_containment,
            metric.primary_in_repeat_regret,
            metric.repeat_in_primary_regret,
            metric.reciprocal_regret,
            metric.primary_in_repeat_plateau,
            metric.repeat_in_primary_plateau,
            metric.reciprocal_plateau,
        )
        .expect("writing to String cannot fail");
    }
    let aggregates = aggregate_all(&metrics);
    writeln!(report).expect("writing to String cannot fail");
    writeln!(report, "## Frozen aggregates").expect("writing to String cannot fail");
    writeln!(report).expect("writing to String cannot fail");
    writeln!(report, "subset\tcells\texact\texact_rate\tmovement_upper_median\tmovement_p90\tmovement_max\tjaccard_lower_median\tjaccard_min\tcontainment_lower_median\tcontainment_min\tprimary_in_repeat_plateau\trepeat_in_primary_plateau\treciprocal_plateau\treciprocal_rate\tregret_upper_median\tregret_p90\tregret_max\tstable")
        .expect("writing to String cannot fail");
    for aggregate in &aggregates {
        writeln!(
            report,
            "{}\t{}\t{}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{}\t{}\t{}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{}",
            aggregate.name,
            aggregate.cells,
            aggregate.exact,
            ratio(aggregate.exact, aggregate.cells),
            aggregate.movement_upper_median,
            aggregate.movement_p90,
            aggregate.movement_max,
            aggregate.jaccard_lower_median,
            aggregate.jaccard_min,
            aggregate.containment_lower_median,
            aggregate.containment_min,
            aggregate.primary_in_repeat_plateau,
            aggregate.repeat_in_primary_plateau,
            aggregate.reciprocal_plateau,
            ratio(aggregate.reciprocal_plateau, aggregate.cells),
            aggregate.regret_upper_median,
            aggregate.regret_p90,
            aggregate.regret_max,
            aggregate.stable,
        )
        .expect("writing to String cannot fail");
    }
    let overall = aggregate_named(&aggregates, "overall");
    let studies_stable = ["shape-aware", "interactions", "table"]
        .into_iter()
        .all(|name| aggregate_named(&aggregates, name).stable);
    let passing_named: Vec<_> = [
        "shape-aware/anchor",
        "shape-aware/fit",
        "shape-aware/held",
        "interactions/anchor",
        "interactions/fit",
        "interactions/held",
        "table/anchor",
        "table/fit",
        "table/held",
    ]
    .into_iter()
    .filter(|name| aggregate_named(&aggregates, name).stable)
    .collect();
    let verdict = if overall.stable && studies_stable {
        "stable-across-records"
    } else if passing_named.is_empty() {
        "unstable-labels"
    } else {
        "stable-named-subsets-only"
    };
    writeln!(report).expect("writing to String cannot fail");
    writeln!(report, "## Verdict").expect("writing to String cannot fail");
    writeln!(report).expect("writing to String cannot fail");
    writeln!(report, "verdict\t{verdict}").expect("writing to String cannot fail");
    writeln!(
        report,
        "stable_study_role_subsets\t{}",
        if passing_named.is_empty() {
            "-".to_owned()
        } else {
            passing_named.join(",")
        }
    )
    .expect("writing to String cannot fail");
    writeln!(report, "policy_evidence\tnone-opened-label-stability-only")
        .expect("writing to String cannot fail");
    report
}

fn load_pair(
    spec: StudySpec,
    primary_path: &str,
    repeat_path: &str,
    environment_path: &str,
    manifest: &BTreeMap<String, String>,
) -> Pair {
    let expected_environment = manifest
        .get(spec.study.key())
        .unwrap_or_else(|| panic!("manifest has no {} environment", spec.study.key()));
    let environment_digest = digest_path(environment_path);
    assert_eq!(
        &environment_digest,
        expected_environment,
        "{}: environment digest moved",
        spec.study.key()
    );
    let environment = parse_environment(environment_path);
    validate_environment(&spec, &environment);
    validate_digest(&environment, "hash.sweep", primary_path);
    validate_digest(&environment, "hash.repeat", repeat_path);
    let primary_digest = digest_path(primary_path);
    let repeat_digest = digest_path(repeat_path);
    let primary = parse_run(&spec, primary_path);
    let repeat = parse_run(&spec, repeat_path);
    validate_run_custody(&spec, &environment, &primary, "primary");
    validate_run_custody(&spec, &environment, &repeat, "repeat");
    assert!(
        primary.measured.keys().eq(repeat.measured.keys()),
        "{}: primary/repeat measured-width populations differ",
        spec.study.key()
    );
    Pair {
        spec,
        primary,
        repeat,
        environment_digest,
        primary_digest,
        repeat_digest,
    }
}

fn parse_manifest(path: &str) -> BTreeMap<String, String> {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("{path} does not read: {error}"));
    let mut lines = text.lines();
    assert_eq!(
        lines.next(),
        Some("study\tenvironment_sha256"),
        "{path}: manifest header moved"
    );
    let mut values = BTreeMap::new();
    for line in lines {
        let (study, digest) = line
            .split_once('\t')
            .unwrap_or_else(|| panic!("{path}: manifest row has no tab: {line}"));
        assert_eq!(digest.len(), 64, "{path}: {study} digest width moved");
        assert!(
            digest.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "{path}: {study} digest is not hexadecimal"
        );
        assert!(
            values.insert(study.to_owned(), digest.to_owned()).is_none(),
            "{path}: duplicate study `{study}`"
        );
    }
    assert_eq!(
        values.keys().map(String::as_str).collect::<Vec<_>>(),
        vec!["interactions", "shape-aware", "table"],
        "{path}: exact manifest study population moved"
    );
    values
}

fn parse_environment(path: &str) -> BTreeMap<String, String> {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("{path} does not read: {error}"));
    let mut values = BTreeMap::new();
    for (position, line) in text.lines().enumerate() {
        if position == 0 {
            assert_eq!(line, "key\tvalue", "{path}: environment header moved");
            continue;
        }
        let (key, value) = line
            .split_once('\t')
            .unwrap_or_else(|| panic!("{path}: environment row has no tab: {line}"));
        assert!(
            values.insert(key.to_owned(), value.to_owned()).is_none(),
            "{path}: duplicate environment key `{key}`"
        );
    }
    values
}

fn validate_environment(spec: &StudySpec, environment: &BTreeMap<String, String>) {
    for (key, expected) in COMMON_ENVIRONMENT.iter().chain(spec.expected_values) {
        assert_eq!(
            environment.get(*key).map(String::as_str),
            Some(*expected),
            "{}: environment key `{key}` moved",
            spec.study.key()
        );
    }
    let expected_keys: BTreeSet<_> = COMMON_ENVIRONMENT
        .iter()
        .chain(spec.expected_values)
        .map(|(key, _)| *key)
        .chain(spec.digest_keys.iter().copied())
        .collect();
    assert_eq!(
        environment
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        expected_keys,
        "{}: exact environment key population moved",
        spec.study.key()
    );
    match spec.study {
        Study::ShapeAware => assert_eq!(
            environment.get("hash.analysis").map(String::as_str),
            Some("806e2f41cdfdbc7c02fe936a958b663413f8d6271bdf447ca89f00213e6e6d72"),
            "shape-aware noise-rule source digest moved"
        ),
        Study::Interactions => assert_eq!(
            environment.get("hash.analysis_shared").map(String::as_str),
            Some("c4f2f807fb90c8b31743d04b362dc4945002350e893ee7b2e0b1fb4c961d8741"),
            "interaction noise-rule source digest moved"
        ),
        Study::Table => assert_eq!(
            environment.get("analysis.noise_band").map(String::as_str),
            Some("2 * (SE_a + SE_b)"),
            "table noise-rule spelling moved"
        ),
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "one scan binds each result's exact schema, metadata, resources, production marks, and finite population before metrics"
)]
fn parse_run(spec: &StudySpec, path: &str) -> Run {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("{path} does not read: {error}"));
    let mut metadata = BTreeMap::new();
    let mut measured = BTreeMap::new();
    let mut outcomes = BTreeSet::new();
    let mut marked = BTreeMap::<(u64, u64), Vec<(u64, bool)>>::new();
    let mut header_seen = false;
    for line in text.lines() {
        if let Some(comment) = line.strip_prefix("# ") {
            if let Some((key, value)) = comment.split_once('\t') {
                assert!(
                    metadata.insert(key.to_owned(), value.to_owned()).is_none(),
                    "{path}: duplicate metadata key `{key}`"
                );
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }
        if line.starts_with("rows\t") {
            assert_eq!(line, RESULT_HEADER, "{path}: result schema moved");
            assert!(!header_seen, "{path}: duplicate result header");
            header_seen = true;
            continue;
        }
        assert!(header_seen, "{path}: result row precedes its header");
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 24, "{path}: result row width moved: {line}");
        assert_eq!(fields[3], "single-workgroup-tree");
        assert_eq!(
            fields[23], "measured",
            "{path}: every frozen width must measure"
        );
        assert!(matches!(fields[6], "production" | "-"));
        let rows = number(fields[0], "rows");
        let contributors = number(fields[1], "contributors");
        let participants = number(fields[4], "participants");
        assert_eq!(number(fields[2], "elements"), rows * contributors);
        assert!(contributors.is_multiple_of(participants));
        assert_eq!(
            number(fields[5], "contributors per partition"),
            contributors / participants
        );
        assert!(contributors / participants >= 2);
        assert!(
            outcomes.insert((rows, contributors, participants)),
            "{path}: duplicate outcome {rows}x{contributors} P{participants}"
        );
        let production = fields[6] == "production";
        marked
            .entry((rows, contributors))
            .or_default()
            .push((participants, production));
        let row = Row {
            participants,
            p50: float(fields[21], "amortized p50"),
            stddev: float(fields[22], "amortized spread"),
        };
        assert!(row.p50 > 0.0, "{path}: every measured p50 is positive");
        assert!(
            row.stddev >= 0.0,
            "{path}: every measured spread is non-negative"
        );
        assert_eq!(number(fields[7], "widest workgroup"), participants);
        assert_eq!(
            number(fields[8], "threadgroup bytes"),
            (4 * participants).div_ceil(16) * 16,
            "{path}: P{participants} prepared threadgroup allocation moved"
        );
        assert_eq!(number(fields[9], "encoders"), 2);
        assert_eq!(number(fields[10], "repetitions"), 30);
        assert_eq!(number(fields[11], "batch"), 64);
        for (position, name) in [
            (12, "submit min"),
            (13, "submit p50"),
            (14, "submit p90"),
            (15, "submit spread"),
            (16, "batch min"),
            (17, "batch p50"),
            (18, "batch p90"),
            (19, "batch spread"),
            (20, "amortized min"),
        ] {
            let _ = float(fields[position], name);
        }
        assert!(
            measured
                .insert((rows, contributors, participants), row)
                .is_none()
        );
    }
    assert!(header_seen, "{path}: missing result header");
    validate_metadata(spec, path, &metadata);
    let mut expected = BTreeSet::new();
    for &rows in spec.rows {
        for &contributors in spec.all {
            let production = production_width(contributors);
            let widths = admissible_participants(contributors);
            assert_eq!(
                marked
                    .get(&(rows, contributors))
                    .unwrap_or_else(|| panic!("{path}: missing {rows}x{contributors} cell"))
                    .iter()
                    .filter(|(_, is_production)| *is_production)
                    .map(|(participants, _)| *participants)
                    .collect::<Vec<_>>(),
                vec![production],
                "{path}: {rows}x{contributors} production mark moved"
            );
            expected.extend(
                widths
                    .into_iter()
                    .map(|participants| (rows, contributors, participants)),
            );
        }
    }
    assert_eq!(expected.len(), spec.expected_variants);
    if outcomes != expected {
        let missing: Vec<_> = expected.difference(&outcomes).copied().collect();
        let unexpected: Vec<_> = outcomes.difference(&expected).copied().collect();
        panic!(
            "{}: exact outcome population moved: missing {missing:?}; unexpected {unexpected:?}",
            spec.study.key()
        );
    }
    assert_eq!(measured.len(), spec.expected_variants);
    Run { measured, metadata }
}

#[allow(
    clippy::too_many_lines,
    reason = "the exact per-run metadata schema is intentionally validated in one visible census"
)]
fn validate_metadata(spec: &StudySpec, path: &str, metadata: &BTreeMap<String, String>) {
    for (key, expected) in [
        ("spike", spec.result_spike),
        ("mode", spec.mode),
        ("metric", "wall-clock microseconds, commit to completed"),
        ("warmup", "8"),
        ("repetitions", "30"),
        ("batch", "64"),
        ("contract", "FLUSH_AND_REASSOCIATE_F32"),
        (
            "declaration",
            "BoundMetalCompileDeclaration::first_macos_apple9",
        ),
        ("device", "Apple M4 Max"),
        ("device_apple9", "true"),
        ("device_max_threads_per_threadgroup", "1024"),
        ("device_max_threadgroup_memory", "32768"),
        ("oracle_tie", "4x16\treference-checked"),
    ] {
        assert_eq!(
            metadata.get(key).map(String::as_str),
            Some(expected),
            "{path}: metadata `{key}` moved"
        );
    }
    for (key, expected) in [
        ("shapes", spec.expected_cells),
        ("variants_attempted", spec.expected_variants),
        ("variants_measured", spec.expected_variants),
        ("variants_declined", 0),
    ] {
        assert_eq!(
            metadata.get(key).map(|value| number(value, key)),
            Some(u64::try_from(expected).expect("finite metadata count fits u64")),
            "{path}: metadata `{key}` moved"
        );
    }
    for key in [
        "load_before",
        "load_after",
        "concurrent_build_processes_before",
        "concurrent_build_processes_after",
        "executable_sha256_before",
        "executable_sha256_after",
    ] {
        assert!(
            metadata.contains_key(key),
            "{path}: missing metadata `{key}`"
        );
    }
    let mut expected_keys: BTreeSet<_> = [
        "spike",
        "mode",
        "metric",
        "warmup",
        "repetitions",
        "batch",
        "contract",
        "declaration",
        "device",
        "device_apple9",
        "device_max_threads_per_threadgroup",
        "device_max_threadgroup_memory",
        "load_before",
        "oracle_tie",
        "shapes",
        "variants_attempted",
        "variants_measured",
        "variants_declined",
        "load_after",
        "executable_sha256_before",
        "executable_sha256_after",
        "concurrent_build_processes_before",
        "concurrent_build_processes_after",
    ]
    .into_iter()
    .collect();
    if !matches!(spec.study, Study::ShapeAware) {
        expected_keys.extend([
            "widest_prepared_workgroup",
            "maximum_prepared_threadgroup_bytes",
        ]);
    }
    assert_eq!(
        metadata.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        expected_keys,
        "{path}: exact metadata key population moved"
    );
    if matches!(spec.study, Study::Interactions) {
        assert_eq!(
            metadata
                .get("widest_prepared_workgroup")
                .map(String::as_str),
            Some("641")
        );
        assert_eq!(
            metadata
                .get("maximum_prepared_threadgroup_bytes")
                .map(String::as_str),
            Some("2576")
        );
    }
    if matches!(spec.study, Study::Table) {
        assert_eq!(
            metadata
                .get("widest_prepared_workgroup")
                .map(String::as_str),
            Some("945")
        );
        assert_eq!(
            metadata
                .get("maximum_prepared_threadgroup_bytes")
                .map(String::as_str),
            Some("3792")
        );
    }
}

fn validate_run_custody(
    spec: &StudySpec,
    environment: &BTreeMap<String, String>,
    run: &Run,
    label: &str,
) {
    for direction in ["before", "after"] {
        let environment_key = format!("host.load_{direction}.{label}");
        let metadata_key = format!("load_{direction}");
        assert_eq!(
            run.metadata.get(&metadata_key),
            environment.get(&environment_key),
            "{} {label}: {direction}-load row moved",
            spec.study.key()
        );
    }
    assert_eq!(
        run.metadata
            .get("concurrent_build_processes_before")
            .map(String::as_str),
        Some("0"),
        "{} {label}: timed start was not process-quiet",
        spec.study.key()
    );
    assert_eq!(
        run.metadata
            .get("concurrent_build_processes_after")
            .map(String::as_str),
        Some("0"),
        "{} {label}: timed end was not process-quiet",
        spec.study.key()
    );
    let executable = environment
        .get("timed.executable.sha256")
        .expect("timed executable digest is retained");
    assert_eq!(
        run.metadata.get("executable_sha256_before"),
        Some(executable),
        "{} {label}: starting executable digest moved",
        spec.study.key()
    );
    assert_eq!(
        run.metadata.get("executable_sha256_after"),
        Some(executable),
        "{} {label}: ending executable digest moved",
        spec.study.key()
    );
}

fn metrics_for_pair(pair: &Pair) -> Vec<CellMetric> {
    let mut metrics = Vec::new();
    for &rows in pair.spec.rows {
        for &contributors in pair.spec.all {
            let primary_cell = pair.primary.cell(rows, contributors);
            let repeat_cell = pair.repeat.cell(rows, contributors);
            let primary_best = raw_best(&primary_cell);
            let repeat_best = raw_best(&repeat_cell);
            let primary_plateau = plateau(&pair.spec, &primary_cell, primary_best);
            let repeat_plateau = plateau(&pair.spec, &repeat_cell, repeat_best);
            let intersection = primary_plateau.intersection(&repeat_plateau).count();
            let union = primary_plateau.union(&repeat_plateau).count();
            assert!(union > 0, "{}: empty plateau union", pair.spec.study.key());
            let primary_containment = ratio(intersection, primary_plateau.len());
            let repeat_containment = ratio(intersection, repeat_plateau.len());
            let primary_in_repeat = pair
                .repeat
                .row(rows, contributors, primary_best.participants);
            let repeat_in_primary = pair
                .primary
                .row(rows, contributors, repeat_best.participants);
            let primary_in_repeat_regret = primary_in_repeat.p50 / repeat_best.p50;
            let repeat_in_primary_regret = repeat_in_primary.p50 / primary_best.p50;
            assert!(
                primary_in_repeat_regret >= 1.0 && repeat_in_primary_regret >= 1.0,
                "{}: a cross-run width beat that run's raw minimum",
                pair.spec.study.key()
            );
            let primary_in_repeat_plateau = repeat_plateau.contains(&primary_best.participants);
            let repeat_in_primary_plateau = primary_plateau.contains(&repeat_best.participants);
            let metric = CellMetric {
                study: pair.spec.study,
                role: pair.spec.role(contributors),
                rows,
                contributors,
                primary_best: primary_best.participants,
                repeat_best: repeat_best.participants,
                exact: primary_best.participants == repeat_best.participants,
                movement: (exact_f64(primary_best.participants).log2()
                    - exact_f64(repeat_best.participants).log2())
                .abs(),
                primary_plateau,
                repeat_plateau,
                jaccard: ratio(intersection, union),
                primary_containment,
                repeat_containment,
                symmetric_containment: primary_containment.min(repeat_containment),
                primary_in_repeat_regret,
                repeat_in_primary_regret,
                reciprocal_regret: primary_in_repeat_regret.max(repeat_in_primary_regret),
                primary_in_repeat_plateau,
                repeat_in_primary_plateau,
                reciprocal_plateau: primary_in_repeat_plateau && repeat_in_primary_plateau,
            };
            assert!(
                [
                    metric.movement,
                    metric.jaccard,
                    metric.primary_containment,
                    metric.repeat_containment,
                    metric.symmetric_containment,
                    metric.primary_in_repeat_regret,
                    metric.repeat_in_primary_regret,
                    metric.reciprocal_regret,
                ]
                .into_iter()
                .all(f64::is_finite),
                "{}: non-finite cell metric",
                pair.spec.study.key()
            );
            metrics.push(metric);
        }
    }
    assert_eq!(metrics.len(), pair.spec.expected_cells);
    metrics
}

fn plateau(spec: &StudySpec, cell: &[Row], best: Row) -> BTreeSet<u64> {
    let plateau: BTreeSet<_> = cell
        .iter()
        .filter(|row| within_plateau(spec, **row, best))
        .map(|row| row.participants)
        .collect();
    assert!(!plateau.is_empty(), "{}: empty plateau", spec.study.key());
    assert!(
        plateau.contains(&best.participants),
        "{}: raw minimum fell outside its own plateau",
        spec.study.key()
    );
    plateau
}

fn within_plateau(spec: &StudySpec, candidate: Row, best: Row) -> bool {
    candidate.p50 - best.p50
        <= spec.noise_multiplier * (candidate.standard_error() + best.standard_error())
}

fn raw_best(cell: &[Row]) -> Row {
    *cell
        .iter()
        .min_by(|left, right| {
            left.p50
                .total_cmp(&right.p50)
                .then_with(|| left.participants.cmp(&right.participants))
        })
        .expect("a validated cell is nonempty")
}

fn aggregate_all(metrics: &[CellMetric]) -> Vec<Aggregate> {
    let mut aggregates = Vec::new();
    aggregates.push(aggregate("overall", metrics.iter()));
    for study in [Study::ShapeAware, Study::Interactions, Study::Table] {
        aggregates.push(aggregate(
            study.key(),
            metrics.iter().filter(|metric| metric.study == study),
        ));
    }
    for role in [Role::Anchor, Role::Fit, Role::Held] {
        aggregates.push(aggregate(
            role.key(),
            metrics.iter().filter(|metric| metric.role == role),
        ));
    }
    for study in [Study::ShapeAware, Study::Interactions, Study::Table] {
        for role in [Role::Anchor, Role::Fit, Role::Held] {
            aggregates.push(aggregate(
                &format!("{}/{}", study.key(), role.key()),
                metrics
                    .iter()
                    .filter(|metric| metric.study == study && metric.role == role),
            ));
        }
    }
    let expected: BTreeMap<&str, usize> = BTreeMap::from([
        ("overall", 188),
        ("shape-aware", 48),
        ("interactions", 70),
        ("table", 70),
        ("anchor", 28),
        ("fit", 80),
        ("held", 80),
        ("shape-aware/anchor", 8),
        ("shape-aware/fit", 20),
        ("shape-aware/held", 20),
        ("interactions/anchor", 10),
        ("interactions/fit", 30),
        ("interactions/held", 30),
        ("table/anchor", 10),
        ("table/fit", 30),
        ("table/held", 30),
    ]);
    assert_eq!(aggregates.len(), expected.len());
    for aggregate in &aggregates {
        assert_eq!(
            expected.get(aggregate.name.as_str()),
            Some(&aggregate.cells),
            "{} subset population moved",
            aggregate.name
        );
    }
    aggregates
}

fn aggregate<'a>(name: &str, metrics: impl Iterator<Item = &'a CellMetric>) -> Aggregate {
    let metrics: Vec<_> = metrics.collect();
    let cells = metrics.len();
    assert!(cells > 0, "{name}: empty aggregate");
    let mut movement: Vec<_> = metrics.iter().map(|metric| metric.movement).collect();
    let mut jaccard: Vec<_> = metrics.iter().map(|metric| metric.jaccard).collect();
    let mut containment: Vec<_> = metrics
        .iter()
        .map(|metric| metric.symmetric_containment)
        .collect();
    let mut regret: Vec<_> = metrics
        .iter()
        .map(|metric| metric.reciprocal_regret)
        .collect();
    for values in [&mut movement, &mut jaccard, &mut containment, &mut regret] {
        values.sort_by(f64::total_cmp);
    }
    let exact = metrics.iter().filter(|metric| metric.exact).count();
    let primary_in_repeat_plateau = metrics
        .iter()
        .filter(|metric| metric.primary_in_repeat_plateau)
        .count();
    let repeat_in_primary_plateau = metrics
        .iter()
        .filter(|metric| metric.repeat_in_primary_plateau)
        .count();
    let reciprocal_plateau = metrics
        .iter()
        .filter(|metric| metric.reciprocal_plateau)
        .count();
    let movement_upper_median = movement[cells / 2];
    let movement_p90 = movement[p90_index(cells)];
    let movement_max = movement[cells - 1];
    let jaccard_lower_median = jaccard[(cells - 1) / 2];
    let jaccard_min = jaccard[0];
    let containment_lower_median = containment[(cells - 1) / 2];
    let containment_min = containment[0];
    let regret_upper_median = regret[cells / 2];
    let regret_p90 = regret[p90_index(cells)];
    let regret_max = regret[cells - 1];
    let stable = exact >= (4 * cells).div_ceil(5)
        && movement_p90 <= 1.0
        && jaccard_lower_median >= 0.50
        && containment_lower_median >= 0.75
        && reciprocal_plateau >= (9 * cells).div_ceil(10)
        && regret_upper_median <= 1.02
        && regret_p90 <= 1.10
        && regret_max <= 1.25;
    Aggregate {
        name: name.to_owned(),
        cells,
        exact,
        movement_upper_median,
        movement_p90,
        movement_max,
        jaccard_lower_median,
        jaccard_min,
        containment_lower_median,
        containment_min,
        primary_in_repeat_plateau,
        repeat_in_primary_plateau,
        reciprocal_plateau,
        regret_upper_median,
        regret_p90,
        regret_max,
        stable,
    }
}

fn aggregate_named<'a>(aggregates: &'a [Aggregate], name: &str) -> &'a Aggregate {
    aggregates
        .iter()
        .find(|aggregate| aggregate.name == name)
        .unwrap_or_else(|| panic!("missing frozen aggregate `{name}`"))
}

fn p90_index(count: usize) -> usize {
    (9 * count).div_ceil(10) - 1
}

fn set_text(values: &BTreeSet<u64>) -> String {
    values
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    assert!(denominator > 0);
    exact_usize_f64(numerator) / exact_usize_f64(denominator)
}

fn exact_f64(value: u64) -> f64 {
    f64::from(u32::try_from(value).expect("the frozen matrix fits exact f64 integers"))
}

fn exact_usize_f64(value: usize) -> f64 {
    exact_f64(u64::try_from(value).expect("the frozen count fits u64"))
}

fn admissible_participants(contributors: u64) -> Vec<u64> {
    (2..=contributors / 2)
        .filter(|participants| contributors.is_multiple_of(*participants))
        .collect()
}

fn production_width(contributors: u64) -> u64 {
    let widths = admissible_participants(contributors);
    let below = widths
        .iter()
        .copied()
        .filter(|participants| *participants <= 256)
        .max()
        .unwrap_or_else(|| {
            widths
                .iter()
                .copied()
                .find(|participants| *participants > 256)
                .expect("every frozen contributor is composite")
        });
    widths
        .iter()
        .copied()
        .find(|participants| *participants > 256 && *participants < 512 - below)
        .unwrap_or(below)
}

fn validate_digest(environment: &BTreeMap<String, String>, key: &str, path: &str) {
    let expected = environment
        .get(key)
        .unwrap_or_else(|| panic!("missing digest key `{key}`"));
    let observed = digest_path(path);
    assert_eq!(&observed, expected, "{path} digest moved");
}

fn digest_path(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    let bytes =
        fs::read(path).unwrap_or_else(|error| panic!("{} does not read: {error}", path.display()));
    digest_bytes(&bytes)
}

fn digest_bytes(bytes: &[u8]) -> String {
    let mut digest = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(digest, "{byte:02x}").expect("writing to String cannot fail");
    }
    digest
}

fn number(value: &str, label: &str) -> u64 {
    value
        .parse()
        .unwrap_or_else(|error| panic!("{label} `{value}` does not parse: {error}"))
}

fn float(value: &str, label: &str) -> f64 {
    let value: f64 = value
        .parse()
        .unwrap_or_else(|error| panic!("{label} `{value}` does not parse: {error}"));
    assert!(value.is_finite(), "{label} is not finite");
    value
}

fn self_check() {
    let specs = study_specs();
    let cells: usize = specs.iter().map(|spec| spec.expected_cells).sum();
    let widths: usize = specs.iter().map(|spec| spec.expected_variants).sum();
    assert_eq!(cells, 188, "the combined cell census moved");
    assert_eq!(widths, 2_506, "the combined width census moved");
    for spec in &specs {
        assert_eq!(
            spec.rows.len() * spec.all.len(),
            spec.expected_cells,
            "{} cell census moved",
            spec.study.key()
        );
        assert_eq!(
            spec.rows.len()
                * spec
                    .all
                    .iter()
                    .map(|contributors| admissible_participants(*contributors).len())
                    .sum::<usize>(),
            spec.expected_variants,
            "{} width census moved",
            spec.study.key()
        );
        assert_eq!(
            spec.noise_multiplier.to_bits(),
            2.0_f64.to_bits(),
            "{} retained noise multiplier moved",
            spec.study.key()
        );
        let standard = REPETITIONS.sqrt();
        let best = Row {
            participants: 2,
            p50: 100.0,
            stddev: standard,
        };
        let boundary = Row {
            participants: 4,
            p50: 104.0,
            ..best
        };
        assert!(
            within_plateau(spec, boundary, best),
            "{} retained noise rule moved at its inclusive boundary",
            spec.study.key()
        );
        let outside = Row {
            p50: 104.000_001,
            ..boundary
        };
        assert!(
            !within_plateau(spec, outside, best),
            "{} retained noise rule no longer separates its outside fixture",
            spec.study.key()
        );
    }
    let tied = [
        Row {
            participants: 4,
            p50: 10.0,
            stddev: 0.0,
        },
        Row {
            participants: 2,
            p50: 10.0,
            stddev: 0.0,
        },
    ];
    assert_eq!(
        raw_best(&tied).participants,
        2,
        "raw-minimum narrow tie moved"
    );
    assert_eq!(p90_index(8), 7);
    assert_eq!(p90_index(10), 8);
    assert_eq!(p90_index(20), 17);
    assert_eq!(p90_index(188), 169);
    println!("# self_check\tpassed");
    println!("# cells\t{cells}");
    println!("# widths_per_run\t{widths}");
    println!("# measured_rows\t{}", 2 * widths);
}

#[cfg(test)]
mod tests {
    #[test]
    fn frozen_protocol_self_check_is_green() {
        super::self_check();
    }
}

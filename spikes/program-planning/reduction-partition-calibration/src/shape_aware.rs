//! Validates and scores the frozen shape-aware tree-width study.
//!
//! The measuring binary needs the qualified Metal host. This binary does not:
//! it validates the exact matrix, retained source and result digests, executable
//! custody, environment, and every result row before fitting anything. Its
//! contributor-grouped training split and ridge protocol were frozen in the
//! README and owning ticket before the first timed submission.
//!
//! ```sh
//! cargo run --release --bin shape-aware-tree-width-analysis -- \
//!   results/<row>/sweep.tsv results/<row>/repeat.tsv results/<row>/environment.tsv
//! cargo run --release --bin shape-aware-tree-width-analysis -- --self-check
//! ```

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Exact rows in the frozen Cartesian matrix.
const MATRIX_ROWS: [u64; 4] = [4, 1_024, 2_048, 16_384];
/// Already-seen contributors, reported only for recurrence.
const ANCHOR_CONTRIBUTORS: [u64; 2] = [780, 1_042];
/// Contributor-grouped fit population.
const FIT_CONTRIBUTORS: [u64; 5] = [756, 779, 840, 1_018, 1_020];
/// Contributor-grouped sealed held-out population.
const HELD_OUT_CONTRIBUTORS: [u64; 5] = [768, 781, 960, 1_022, 1_046];
/// Every contributor in the emitted matrix's order-independent population.
const ALL_CONTRIBUTORS: [u64; 12] = [
    780, 1_042, 756, 779, 840, 1_018, 1_020, 768, 781, 960, 1_022, 1_046,
];
/// Fresh rows for the row-regime/divisor-interaction study.
#[allow(
    dead_code,
    reason = "used by the distinct interaction analyzer wrapper"
)]
const INTERACTION_ROWS: [u64; 5] = [8, 528, 1_056, 2_112, 8_192];
/// Historical contributors measured only as recurrence anchors on fresh rows.
#[allow(
    dead_code,
    reason = "used by the distinct interaction analyzer wrapper"
)]
const INTERACTION_ANCHORS: [u64; 2] = [780, 1_042];
/// Fresh contributor-grouped fit population.
#[allow(
    dead_code,
    reason = "used by the distinct interaction analyzer wrapper"
)]
const INTERACTION_FIT: [u64; 6] = [774, 783, 900, 1_006, 1_082, 1_280];
/// Fresh contributor-grouped sealed held-out population.
#[allow(
    dead_code,
    reason = "used by the distinct interaction analyzer wrapper"
)]
const INTERACTION_HELD: [u64; 6] = [775, 785, 899, 1_008, 1_094, 1_282];
/// Complete order-independent interaction population.
#[allow(
    dead_code,
    reason = "used by the distinct interaction analyzer wrapper"
)]
const INTERACTION_ALL: [u64; 14] = [
    780, 1_042, 774, 783, 900, 1_006, 1_082, 1_280, 775, 785, 899, 1_008, 1_094, 1_282,
];
/// Candidate ridge strengths, frozen before timing.
const LAMBDAS: [f64; 5] = [0.0, 0.000_001, 0.000_1, 0.01, 1.0];
/// Existing measured saturated-fold-step row.
const SATURATED_FOLD_STEPS: u64 = 1_056;
/// Deterministic solver refusal threshold.
const PIVOT_FLOOR: f64 = 1e-12;
/// Timed sample count carried by every measured row.
const REPETITIONS: f64 = 30.0;

/// Converts one matrix-bounded integer to an exactly represented model value.
fn exact_f64(value: u64, label: &str) -> f64 {
    f64::from(
        u32::try_from(value)
            .unwrap_or_else(|_| panic!("{label} exceeds this finite study's exact f64 domain")),
    )
}

/// Converts one matrix-bounded collection index to an exact model value.
fn exact_usize_f64(value: usize, label: &str) -> f64 {
    exact_f64(
        u64::try_from(value).unwrap_or_else(|_| panic!("{label} does not fit u64")),
        label,
    )
}

/// One target-admitted measured width.
#[derive(Clone, Copy, Debug)]
struct Row {
    rows: u64,
    contributors: u64,
    participants: u64,
    production: bool,
    p50: f64,
    stddev: f64,
}

/// One scored cell: shape, chosen row, raw-best row, and plateau miss.
type ScoredChoice = (u64, u64, Row, Row, bool);

impl Row {
    /// Standard error used by this spike and both predecessor records.
    fn standard_error(self) -> f64 {
        self.stddev / REPETITIONS.sqrt()
    }
}

/// One fully validated retained run.
struct Run {
    measured: BTreeMap<(u64, u64, u64), Row>,
    metadata: BTreeMap<String, String>,
}

impl Run {
    /// Target-admitted rows for one cell, in participant order.
    fn cell(&self, rows: u64, contributors: u64) -> Vec<Row> {
        self.measured
            .values()
            .filter(|row| row.rows == rows && row.contributors == contributors)
            .copied()
            .collect()
    }

    /// One exact measured row.
    fn row(&self, rows: u64, contributors: u64, participants: u64) -> Row {
        *self
            .measured
            .get(&(rows, contributors, participants))
            .unwrap_or_else(|| {
                panic!("{rows}x{contributors} carries no admitted P{participants} row")
            })
    }
}

/// The fitted information-set families, in least-information tie order.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Family {
    Contributor,
    RowsContributor,
    Lattice,
    #[allow(
        dead_code,
        reason = "used by the distinct interaction analyzer wrapper"
    )]
    Interaction,
    ExistingSaturation,
}

impl Family {
    /// Stable report key.
    const fn key(self) -> &'static str {
        match self {
            Self::Contributor => "contributor-only",
            Self::RowsContributor => "rows-plus-contributors",
            Self::Lattice => "divisor-lattice",
            Self::Interaction => "row-regime-lattice-interactions",
            Self::ExistingSaturation => "existing-saturation-1056",
        }
    }

    /// Whether this family is fitted rather than a zero-fit physical baseline.
    const fn fitted(self) -> bool {
        !matches!(self, Self::ExistingSaturation)
    }
}

/// Fit objective and held-out summary.
#[derive(Clone, Copy, Debug)]
struct Objective {
    worst: f64,
    median: f64,
    misses: usize,
}

impl Objective {
    /// Lexicographic comparison frozen by the protocol.
    fn compare(self, other: Self) -> std::cmp::Ordering {
        self.worst
            .total_cmp(&other.worst)
            .then_with(|| self.median.total_cmp(&other.median))
            .then_with(|| self.misses.cmp(&other.misses))
    }
}

/// Standardization and coefficients of one fitted family.
struct Model {
    family: Family,
    lambda: f64,
    means: Vec<f64>,
    scales: Vec<f64>,
    coefficients: Vec<f64>,
}

impl Model {
    /// Predicts natural-log regret for one candidate width.
    fn predict(&self, rows: u64, contributors: u64, participants: u64) -> f64 {
        let raw = features(self.family, rows, contributors, participants);
        assert_eq!(raw.len(), self.means.len(), "feature vector width moved");
        let mut prediction = self.coefficients[0];
        for (index, value) in raw.iter().enumerate() {
            let standardized = if self.scales[index] == 0.0 {
                0.0
            } else {
                (*value - self.means[index]) / self.scales[index]
            };
            prediction += self.coefficients[index + 1] * standardized;
        }
        prediction
    }
}

/// One fit response.
struct Observation {
    rows: u64,
    contributors: u64,
    participants: u64,
    response: f64,
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments == ["--self-check"] {
        self_check();
        return;
    }
    let [sweep_path, repeat_path, environment_path] = arguments.as_slice() else {
        panic!(
            "usage: shape-aware-tree-width-analysis <sweep.tsv> <repeat.tsv> \
             <environment.tsv> | --self-check"
        );
    };

    let environment = parse_environment(environment_path);
    validate_environment(&environment);
    for (key, path) in [
        ("hash.main", "src/main.rs"),
        ("hash.regions", "src/regions.rs"),
        ("hash.buffer", "src/buffer.rs"),
        ("hash.analysis", "src/shape_aware.rs"),
        ("hash.cargo_toml", "Cargo.toml"),
        ("hash.cargo_lock", "Cargo.lock"),
        (
            "hash.compiler_physical",
            "../../../crates/tiler-compiler/src/physical.rs",
        ),
        (
            "hash.compiler_measured_cost",
            "../../../crates/tiler-compiler/src/measured_cost.rs",
        ),
        ("hash.sweep", sweep_path),
        ("hash.repeat", repeat_path),
    ] {
        validate_digest(&environment, key, path);
    }

    let primary = parse_run(sweep_path);
    let repeat = parse_run(repeat_path);
    validate_run_custody(&environment, &primary, "primary");
    validate_run_custody(&environment, &repeat, "repeat");

    println!("# analysis\treduction-shape-aware-tree-width");
    println!("# validation\tpassed");
    println!("# shapes\t48");
    println!("# arithmetic_variants_per_run\t616");
    println!("# measured_primary\t{}", primary.measured.len());
    println!("# measured_repeat\t{}", repeat.measured.len());
    println!();
    analyze(&primary, &repeat);
}

/// Exercises the exact population, model families, and solver without a device.
fn self_check() {
    assert_eq!(ANCHOR_CONTRIBUTORS, [780, 1_042]);
    let census = ALL_CONTRIBUTORS
        .iter()
        .map(|contributors| admissible_participants(*contributors).len())
        .sum::<usize>();
    assert_eq!(census, 154, "the per-row arithmetic census moved");
    assert_eq!(census * MATRIX_ROWS.len(), 616, "the matrix census moved");

    let mut measured = BTreeMap::new();
    for rows in MATRIX_ROWS {
        for contributors in ALL_CONTRIBUTORS {
            let production = production_width(contributors);
            for participants in admissible_participants(contributors) {
                // A deterministic positive surface with a row interaction. It
                // is not evidence and exists only to reach every fit path.
                let depth = contributors / participants + participants;
                let p50 = exact_f64(
                    (rows * (contributors + participants)).max(depth * SATURATED_FOLD_STEPS),
                    "synthetic self-check cost",
                );
                measured.insert(
                    (rows, contributors, participants),
                    Row {
                        rows,
                        contributors,
                        participants,
                        production: participants == production,
                        p50,
                        stddev: p50 / 100.0,
                    },
                );
            }
        }
    }
    let run = Run {
        measured,
        metadata: BTreeMap::new(),
    };
    let selected = select_primary_family(&run);
    for family in [
        Family::Contributor,
        Family::RowsContributor,
        Family::Lattice,
    ] {
        let (lambda, _) = choose_lambda(&run, family);
        let model = fit_model(&run, family, lambda, &FIT_CONTRIBUTORS)
            .expect("a selected lambda fits the full synthetic population");
        let scored = score_cells(&run, &HELD_OUT_CONTRIBUTORS, |rows, contributors, cell| {
            choose_predicted(&model, rows, contributors, cell)
        });
        assert!(scored.0.worst.is_finite());
    }
    println!("# self_check\tpassed");
    println!("# widths_per_row\t{census}");
    println!("# variants_per_run\t{}", census * MATRIX_ROWS.len());
    println!("# fit_selected_family\t{}", selected.family.key());
}

/// Parses and validates one retained TSV, including declined outcomes.
#[allow(
    clippy::too_many_lines,
    reason = "one scan validates schema, metadata, exact outcome population, resource rows, and production marks before constructing a Run"
)]
fn parse_run(path: &str) -> Run {
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
            assert_eq!(
                line,
                "rows\tcontributors\telements\tstrategy\tpartitions\tper_partition\tproduction\twidest_workgroup\tthreadgroup_bytes\tencoders\treps\tbatch\tsubmit_min_us\tsubmit_p50_us\tsubmit_p90_us\tsubmit_stddev_us\tbatch_min_us\tbatch_p50_us\tbatch_p90_us\tbatch_stddev_us\tamortized_min_us\tamortized_p50_us\tamortized_stddev_us\tstatus",
                "{path}: result schema moved"
            );
            assert!(!header_seen, "{path}: duplicate result header");
            header_seen = true;
            continue;
        }
        assert!(header_seen, "{path}: result row precedes its header");
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 24, "{path}: result row width moved: {line}");
        assert_eq!(
            fields[3], "single-workgroup-tree",
            "{path}: the shape-aware study measures only the tree"
        );
        assert!(
            matches!(fields[6], "production" | "-"),
            "{path}: selection marker moved: {}",
            fields[6]
        );
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
        let is_production = fields[6] == "production";
        marked
            .entry((rows, contributors))
            .or_default()
            .push((participants, is_production));

        if fields[23] == "measured" {
            let row = Row {
                rows,
                contributors,
                participants,
                production: is_production,
                p50: float(fields[21], "amortized p50"),
                stddev: float(fields[22], "amortized spread"),
            };
            assert!(row.p50 > 0.0, "{path}: every measured p50 is positive");
            assert!(row.stddev >= 0.0, "{path}: every spread is non-negative");
            assert_eq!(number(fields[7], "widest workgroup"), participants);
            let requested = 4 * participants;
            assert_eq!(
                number(fields[8], "threadgroup bytes"),
                requested.div_ceil(16) * 16,
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
        } else {
            assert!(
                fields[23].starts_with("declined: "),
                "{path}: unknown outcome `{}`",
                fields[23]
            );
            assert!(fields[7..23].iter().all(|field| *field == "-"));
        }
    }
    assert!(header_seen, "{path}: missing result header");

    let fixed_metadata = [
        ("spike", "reduction-shape-aware-tree-width"),
        ("mode", "ShapeAwareTreeWidth"),
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
        ("shapes", "48"),
        ("variants_attempted", "616"),
    ];
    for (key, expected) in fixed_metadata {
        assert_eq!(
            metadata.get(key).map(String::as_str),
            Some(expected),
            "{path}: metadata `{key}` moved"
        );
    }
    let dynamic_metadata = [
        "load_before",
        "load_after",
        "variants_measured",
        "variants_declined",
        "concurrent_build_processes_before",
        "concurrent_build_processes_after",
        "executable_sha256_before",
        "executable_sha256_after",
    ];
    for key in dynamic_metadata {
        assert!(
            metadata.contains_key(key),
            "{path}: missing metadata `{key}`"
        );
    }
    let expected_metadata: BTreeSet<&str> = fixed_metadata
        .into_iter()
        .map(|(key, _)| key)
        .chain(dynamic_metadata)
        .collect();
    assert_eq!(
        metadata.keys().map(String::as_str).collect::<BTreeSet<_>>(),
        expected_metadata,
        "{path}: exact metadata key population moved"
    );
    assert_eq!(
        usize::try_from(number(
            metadata.get("variants_measured").expect("measured count"),
            "variants measured",
        ))
        .expect("the measured count fits usize"),
        measured.len()
    );
    assert_eq!(
        usize::try_from(number(
            metadata.get("variants_declined").expect("declined count"),
            "variants declined",
        ))
        .expect("the declined count fits usize"),
        outcomes.len() - measured.len()
    );

    let mut expected = BTreeSet::new();
    for rows in MATRIX_ROWS {
        for contributors in ALL_CONTRIBUTORS {
            let widths = admissible_participants(contributors);
            let cell_measured = widths
                .iter()
                .filter(|participants| measured.contains_key(&(rows, contributors, **participants)))
                .count();
            assert!(
                cell_measured > 0,
                "{path}: {rows}x{contributors} admits no measured width"
            );
            let production = production_width(contributors);
            assert!(
                measured.contains_key(&(rows, contributors, production)),
                "{path}: {rows}x{contributors} production P{production} declined"
            );
            let marks = marked
                .get(&(rows, contributors))
                .expect("every expected cell has outcomes");
            assert_eq!(
                marks
                    .iter()
                    .filter(|(_, production)| *production)
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
    assert_eq!(expected.len(), 616, "the expected census moved");
    if outcomes != expected {
        let missing: Vec<_> = expected.difference(&outcomes).copied().collect();
        let unexpected: Vec<_> = outcomes.difference(&expected).copied().collect();
        panic!(
            "{path}: exact outcome population moved: missing {missing:?}; unexpected {unexpected:?}"
        );
    }

    Run { measured, metadata }
}

/// Cross-checks dynamic TSV custody rows against equality-pinned environment rows.
fn validate_run_custody(environment: &BTreeMap<String, String>, run: &Run, label: &str) {
    let expected = |suffix: &str| {
        environment
            .get(&format!("host.{suffix}.{label}"))
            .unwrap_or_else(|| panic!("missing host.{suffix}.{label}"))
    };
    assert_eq!(
        run.metadata.get("load_before"),
        Some(expected("load_before")),
        "{label}: load-before row does not match the retained environment"
    );
    assert_eq!(
        run.metadata.get("load_after"),
        Some(expected("load_after")),
        "{label}: load-after row does not match the retained environment"
    );
    assert_eq!(
        run.metadata
            .get("concurrent_build_processes_before")
            .map(String::as_str),
        Some("0"),
        "{label}: timed start was not process-quiet"
    );
    assert_eq!(
        run.metadata
            .get("concurrent_build_processes_after")
            .map(String::as_str),
        Some("0"),
        "{label}: timed end was not process-quiet"
    );
    let executable = environment
        .get("timed.executable.sha256")
        .expect("timed executable digest is retained");
    assert_eq!(
        run.metadata.get("executable_sha256_before"),
        Some(executable),
        "{label}: starting executable digest does not match retained custody"
    );
    assert_eq!(
        run.metadata.get("executable_sha256_after"),
        Some(executable),
        "{label}: ending executable digest does not match retained custody"
    );
}

/// Performs fit-only family selection, held-out scoring, and repeat diagnostics.
fn analyze(primary: &Run, repeat: &Run) {
    let selected = select_primary_family(primary);
    let repeat_selected = select_primary_family(repeat);

    let models = fit_and_report(primary, selected.family, repeat_selected.family);
    print_selected_predictions(primary, selected.family, &models);
    let summaries = score_and_report(primary, repeat, &models);
    let primary_selected = summaries[&("primary", selected.family)];
    let repeat_selected_summary = summaries[&("repeat", selected.family)];
    let supported = qualifies_support(primary_selected, production_objective(primary))
        && qualifies_support(repeat_selected_summary, production_objective(repeat));
    println!();
    println!("fit_selected_family\t{}", selected.family.key());
    println!(
        "support_verdict\t{}",
        if supported {
            "supported-on-this-finite-qualified-host-population"
        } else {
            "finite-population-insufficient"
        }
    );
    println!();
    print_repeatability(primary, repeat, selected.family, &models);
    println!();
    print_anchor_boundaries("primary", primary);
    println!();
    print_anchor_boundaries("repeat", repeat);
}

/// Fits on primary fit contributors and reports the frozen selection objective.
fn fit_and_report(
    primary: &Run,
    selected: Family,
    repeat_selected: Family,
) -> BTreeMap<Family, Model> {
    println!("## Fit-only model selection");
    println!();
    println!("family\tlambda\tworst\tupper_median\tplateau_misses");
    let mut models = BTreeMap::new();
    for family in [
        Family::Contributor,
        Family::RowsContributor,
        Family::Lattice,
    ] {
        let (lambda, objective) = choose_lambda(primary, family);
        let model = fit_model(primary, family, lambda, &FIT_CONTRIBUTORS)
            .expect("the selected lambda fits all primary fit contributors");
        println!(
            "{}\t{lambda:.6}\t{:.6}\t{:.6}\t{}",
            family.key(),
            objective.worst,
            objective.median,
            objective.misses
        );
        models.insert(family, model);
    }
    let saturation = direct_objective(primary, &FIT_CONTRIBUTORS, Family::ExistingSaturation);
    println!(
        "{}\t-\t{:.6}\t{:.6}\t{}",
        Family::ExistingSaturation.key(),
        saturation.worst,
        saturation.median,
        saturation.misses
    );
    println!("selected_family\t{}", selected.key());
    println!("repeat_refit_selected_family\t{}", repeat_selected.key());
    println!();

    println!("## Final fitted parameters");
    println!();
    for model in models.values() {
        println!("family\t{}", model.family.key());
        println!("lambda\t{:.6}", model.lambda);
        println!("intercept\t{:.12}", model.coefficients[0]);
        for index in 0..model.means.len() {
            println!(
                "feature\t{index}\tmean={:.12}\tscale={:.12}\tcoefficient={:.12}",
                model.means[index],
                model.scales[index],
                model.coefficients[index + 1]
            );
        }
        println!();
    }
    models
}

/// Retains every prediction made by the primary fit-selected family.
fn print_selected_predictions(primary: &Run, selected: Family, models: &BTreeMap<Family, Model>) {
    println!("## Primary selected-family predictions");
    println!();
    if !selected.fitted() {
        println!("policy\t{}\tno fitted predictions", selected.key());
        println!();
        return;
    }
    let model = models
        .get(&selected)
        .expect("the fitted selected family has a final model");
    println!("rows\tcontributors\tparticipants\tpredicted_log_regret");
    for row in primary.measured.values() {
        println!(
            "{}\t{}\t{}\t{:.12}",
            row.rows,
            row.contributors,
            row.participants,
            model.predict(row.rows, row.contributors, row.participants)
        );
    }
    println!();
}

/// Scores every family on the two sealed held-out runs.
fn score_and_report(
    primary: &Run,
    repeat: &Run,
    models: &BTreeMap<Family, Model>,
) -> BTreeMap<(&'static str, Family), Objective> {
    let mut summaries = BTreeMap::new();
    println!("## Sealed held-out scoring");
    println!();
    println!("run\tpolicy\tworst_regret\tupper_median_regret\tplateau_misses");
    let mut reports = Vec::new();
    for (label, run) in [("primary", primary), ("repeat", repeat)] {
        for family in [
            Family::Contributor,
            Family::RowsContributor,
            Family::Lattice,
            Family::ExistingSaturation,
        ] {
            let scored = if family.fitted() {
                let model = models
                    .get(&family)
                    .expect("every fitted family has a model");
                score_cells(run, &HELD_OUT_CONTRIBUTORS, |rows, contributors, cell| {
                    choose_predicted(model, rows, contributors, cell)
                })
            } else {
                score_cells(run, &HELD_OUT_CONTRIBUTORS, |rows, contributors, cell| {
                    choose_direct(family, rows, contributors, cell)
                })
            };
            println!(
                "{label}\t{}\t{:.6}\t{:.6}\t{}",
                family.key(),
                scored.0.worst,
                scored.0.median,
                scored.0.misses
            );
            summaries.insert((label, family), scored.0);
            reports.push((label, family.key(), scored.1));
        }
        let production = score_cells(run, &HELD_OUT_CONTRIBUTORS, |_, _, cell| {
            *cell
                .iter()
                .find(|row| row.production)
                .expect("production is admitted")
        });
        println!(
            "{label}\tproduction-nearest-256\t{:.6}\t{:.6}\t{}",
            production.0.worst, production.0.median, production.0.misses
        );
        reports.push((label, "production-nearest-256", production.1));
    }
    println!();
    println!("run\tpolicy\tshape\tchosen_P\traw_best_P\tregret\toutside_plateau");
    for (label, policy, choices) in reports {
        for (rows, contributors, chosen, best, miss) in choices {
            println!(
                "{label}\t{policy}\t{rows}x{contributors}\t{}\t{}\t{:.6}\t{miss}",
                chosen.participants,
                best.participants,
                chosen.p50 / best.p50
            );
        }
    }
    summaries
}

/// Scores the unchanged production selector on the held-out population.
fn production_objective(run: &Run) -> Objective {
    score_cells(run, &HELD_OUT_CONTRIBUTORS, |_, _, cell| {
        *cell.iter().find(|row| row.production).expect("production")
    })
    .0
}

/// Applies the support bar independently to one primary or repeat run.
fn qualifies_support(candidate: Objective, production: Objective) -> bool {
    candidate.worst <= 1.10
        && candidate.median <= 1.02
        && candidate.misses <= 2
        && candidate.worst < production.worst
        && candidate.median <= production.median
}

/// One family's fit-only winner.
struct FamilySelection {
    family: Family,
    objective: Objective,
}

/// Selects the primary family using fit contributors only.
fn select_primary_family(run: &Run) -> FamilySelection {
    let mut selections = Vec::new();
    for family in [
        Family::Contributor,
        Family::RowsContributor,
        Family::Lattice,
    ] {
        let (_, objective) = choose_lambda(run, family);
        selections.push(FamilySelection { family, objective });
    }
    selections.push(FamilySelection {
        family: Family::ExistingSaturation,
        objective: direct_objective(run, &FIT_CONTRIBUTORS, Family::ExistingSaturation),
    });
    selections
        .into_iter()
        .min_by(|left, right| {
            left.objective
                .compare(right.objective)
                .then_with(|| left.family.cmp(&right.family))
        })
        .expect("four families are compared")
}

/// Chooses one fitted family's ridge strength by contributor-group LOO.
fn choose_lambda(run: &Run, family: Family) -> (f64, Objective) {
    choose_lambda_for(run, family, &FIT_CONTRIBUTORS, &MATRIX_ROWS)
}

/// Chooses one fitted family's ridge strength over explicit contributor groups.
fn choose_lambda_for(
    run: &Run,
    family: Family,
    fit_contributors: &[u64],
    matrix_rows: &[u64],
) -> (f64, Objective) {
    assert!(family.fitted());
    LAMBDAS
        .into_iter()
        .filter_map(|lambda| {
            let mut choices = Vec::new();
            for &held in fit_contributors {
                let contributors: Vec<u64> = fit_contributors
                    .iter()
                    .copied()
                    .filter(|candidate| *candidate != held)
                    .collect();
                let model = fit_model_for_rows(run, family, lambda, &contributors, matrix_rows)?;
                choices.extend(
                    score_cells_for(run, &[held], matrix_rows, |rows, count, cell| {
                        choose_predicted(&model, rows, count, cell)
                    })
                    .1,
                );
            }
            Some((lambda, summarize(&choices)))
        })
        .min_by(|left, right| {
            left.1
                .compare(right.1)
                .then_with(|| left.0.total_cmp(&right.0))
        })
        .unwrap_or_else(|| panic!("{}: every lambda produced a singular fit", family.key()))
}

/// Fits one standardized ridge model.
fn fit_model(run: &Run, family: Family, lambda: f64, contributors: &[u64]) -> Option<Model> {
    fit_model_for_rows(run, family, lambda, contributors, &MATRIX_ROWS)
}

/// Fits one standardized ridge model over an explicit row population.
fn fit_model_for_rows(
    run: &Run,
    family: Family,
    lambda: f64,
    contributors: &[u64],
    matrix_rows: &[u64],
) -> Option<Model> {
    let observations = observations_for_rows(run, contributors, matrix_rows);
    let raw: Vec<Vec<f64>> = observations
        .iter()
        .map(|observation| {
            features(
                family,
                observation.rows,
                observation.contributors,
                observation.participants,
            )
        })
        .collect();
    let width = raw.first()?.len();
    assert!(raw.iter().all(|features| features.len() == width));
    let count = exact_usize_f64(raw.len(), "fit observation count");
    let means: Vec<f64> = (0..width)
        .map(|column| raw.iter().map(|row| row[column]).sum::<f64>() / count)
        .collect();
    let scales: Vec<f64> = (0..width)
        .map(|column| {
            (raw.iter()
                .map(|row| (row[column] - means[column]).powi(2))
                .sum::<f64>()
                / count)
                .sqrt()
        })
        .collect();
    let columns = width + 1;
    let mut normal = vec![vec![0.0; columns]; columns];
    let mut target = vec![0.0; columns];
    for (position, observation) in observations.iter().enumerate() {
        let mut design = vec![1.0];
        design.extend((0..width).map(|column| {
            if scales[column] == 0.0 {
                0.0
            } else {
                (raw[position][column] - means[column]) / scales[column]
            }
        }));
        for left in 0..columns {
            target[left] += design[left] * observation.response;
            for right in 0..columns {
                normal[left][right] += design[left] * design[right];
            }
        }
    }
    for column in 1..columns {
        if scales[column - 1] == 0.0 {
            normal[column].fill(0.0);
            for row in &mut normal {
                row[column] = 0.0;
            }
            normal[column][column] = 1.0;
            target[column] = 0.0;
        } else {
            normal[column][column] += lambda;
        }
    }
    let coefficients = solve(normal, target)?;
    Some(Model {
        family,
        lambda,
        means,
        scales,
        coefficients,
    })
}

/// Deterministic Gauss-Jordan solution of one square system.
fn solve(mut matrix: Vec<Vec<f64>>, mut target: Vec<f64>) -> Option<Vec<f64>> {
    let size = target.len();
    for pivot in 0..size {
        let row = (pivot..size).max_by(|left, right| {
            matrix[*left][pivot]
                .abs()
                .total_cmp(&matrix[*right][pivot].abs())
                .then_with(|| right.cmp(left))
        })?;
        let value = matrix[row][pivot];
        if !value.is_finite() || value.abs() <= PIVOT_FLOOR {
            return None;
        }
        matrix.swap(pivot, row);
        target.swap(pivot, row);
        let value = matrix[pivot][pivot];
        for cell in &mut matrix[pivot][pivot..] {
            *cell /= value;
        }
        target[pivot] /= value;
        let normalized_pivot = matrix[pivot][pivot..].to_vec();
        for row in 0..size {
            if row == pivot {
                continue;
            }
            let factor = matrix[row][pivot];
            for (cell, pivot_cell) in matrix[row][pivot..].iter_mut().zip(&normalized_pivot) {
                *cell -= factor * pivot_cell;
            }
            target[row] -= factor * target[pivot];
        }
    }
    target
        .iter()
        .all(|value| value.is_finite())
        .then_some(target)
}

/// Fit observations over an explicit row population, centered per cell.
fn observations_for_rows(run: &Run, contributors: &[u64], matrix_rows: &[u64]) -> Vec<Observation> {
    let mut observations = Vec::new();
    for &rows in matrix_rows {
        for contributors in contributors {
            let cell = run.cell(rows, *contributors);
            let best = raw_best(&cell);
            observations.extend(cell.into_iter().map(|row| Observation {
                rows,
                contributors: *contributors,
                participants: row.participants,
                response: (row.p50 / best.p50).ln(),
            }));
        }
    }
    observations
}

/// Exact feature vector frozen in the ticket.
fn features(family: Family, rows: u64, contributors: u64, participants: u64) -> Vec<f64> {
    let participants_value = exact_f64(participants, "participant count");
    let w = participants_value.log2();
    let q = exact_f64(contributors / participants, "contributors per participant").log2();
    let r = exact_f64(rows, "row count").log2();
    let mut values = vec![w, q, w * w, q * q, w * q];
    if matches!(
        family,
        Family::RowsContributor | Family::Lattice | Family::Interaction
    ) {
        values.extend([r * w, r * q, r * w * w, r * q * q, r * w * q]);
    }
    if matches!(family, Family::Lattice | Family::Interaction) {
        let widths = admissible_participants(contributors);
        let position = widths
            .binary_search(&participants)
            .expect("the candidate belongs to its divisor lattice");
        let rank = if widths.len() == 1 {
            0.0
        } else {
            exact_usize_f64(position, "lattice position")
                / exact_usize_f64(widths.len() - 1, "lattice span")
        };
        let previous = if position == 0 {
            participants
        } else {
            widths[position - 1]
        };
        let next = widths.get(position + 1).copied().unwrap_or(participants);
        values.extend([
            rank,
            (participants_value / exact_f64(previous, "previous lattice width")).log2(),
            (exact_f64(next, "next lattice width") / participants_value).log2(),
            rank * exact_usize_f64(widths.len(), "lattice width count").log2(),
            (participants_value - 256.0) / 256.0,
        ]);
    }
    if matches!(family, Family::Interaction) {
        let widths = admissible_participants(contributors);
        let position = widths
            .binary_search(&participants)
            .expect("the candidate belongs to its divisor lattice");
        let rank = if widths.len() == 1 {
            0.0
        } else {
            exact_usize_f64(position, "lattice position")
                / exact_usize_f64(widths.len() - 1, "lattice span")
        };
        let previous = if position == 0 {
            participants
        } else {
            widths[position - 1]
        };
        let next = widths.get(position + 1).copied().unwrap_or(participants);
        let regime = (exact_f64(rows, "row count") / 1_056.0)
            .log2()
            .clamp(-1.0, 1.0);
        values.extend([
            regime * rank,
            regime * (participants_value / exact_f64(previous, "previous lattice width")).log2(),
            regime * (exact_f64(next, "next lattice width") / participants_value).log2(),
        ]);
    }
    values
}

/// Chooses a fitted model's minimum prediction, narrower on an exact tie.
fn choose_predicted(model: &Model, rows: u64, contributors: u64, cell: &[Row]) -> Row {
    *cell
        .iter()
        .min_by(|left, right| {
            model
                .predict(rows, contributors, left.participants)
                .total_cmp(&model.predict(rows, contributors, right.participants))
                .then_with(|| left.participants.cmp(&right.participants))
        })
        .expect("every scored cell has an admitted width")
}

/// Chooses the zero-fit existing-saturation family.
fn choose_direct(family: Family, rows: u64, contributors: u64, cell: &[Row]) -> Row {
    assert_eq!(family, Family::ExistingSaturation);
    *cell
        .iter()
        .min_by_key(|row| {
            let participants = row.participants;
            let work = rows * (contributors + participants);
            let depth = contributors / participants + participants;
            (work.max(SATURATED_FOLD_STEPS * depth), participants)
        })
        .expect("every scored cell has an admitted width")
}

/// Scores one policy over every row of contributor groups.
fn score_cells(
    run: &Run,
    contributors: &[u64],
    choose: impl FnMut(u64, u64, &[Row]) -> Row,
) -> (Objective, Vec<ScoredChoice>) {
    score_cells_for(run, contributors, &MATRIX_ROWS, choose)
}

/// Scores one policy over explicit rows of contributor groups.
fn score_cells_for(
    run: &Run,
    contributors: &[u64],
    matrix_rows: &[u64],
    mut choose: impl FnMut(u64, u64, &[Row]) -> Row,
) -> (Objective, Vec<ScoredChoice>) {
    let mut choices = Vec::new();
    for &rows in matrix_rows {
        for contributors in contributors {
            let cell = run.cell(rows, *contributors);
            let best = raw_best(&cell);
            let chosen = choose(rows, *contributors, &cell);
            let miss = chosen.p50 - best.p50 > separation_band(chosen, best);
            choices.push((rows, *contributors, chosen, best, miss));
        }
    }
    (summarize(&choices), choices)
}

/// Summarizes raw regret and plateau misses.
fn summarize(choices: &[ScoredChoice]) -> Objective {
    let mut regrets: Vec<f64> = choices
        .iter()
        .map(|(_, _, chosen, best, _)| chosen.p50 / best.p50)
        .collect();
    regrets.sort_by(f64::total_cmp);
    Objective {
        worst: *regrets.last().expect("at least one cell is scored"),
        median: regrets[regrets.len() / 2],
        misses: choices.iter().filter(|choice| choice.4).count(),
    }
}

/// Direct fit-population objective for a zero-fit family.
fn direct_objective(run: &Run, contributors: &[u64], family: Family) -> Objective {
    score_cells(run, contributors, |rows, contributors, cell| {
        choose_direct(family, rows, contributors, cell)
    })
    .0
}

/// Raw minimum, narrower on an exact p50 tie.
fn raw_best(cell: &[Row]) -> Row {
    *cell
        .iter()
        .min_by(|left, right| {
            left.p50
                .total_cmp(&right.p50)
                .then_with(|| left.participants.cmp(&right.participants))
        })
        .expect("every validated cell has a measured width")
}

/// Twice the sum of the two medians' standard errors.
fn separation_band(left: Row, right: Row) -> f64 {
    2.0 * (left.standard_error() + right.standard_error())
}

/// Prints primary/repeat variation and selected-row plateau agreement.
fn print_repeatability(
    primary: &Run,
    repeat: &Run,
    selected: Family,
    models: &BTreeMap<Family, Model>,
) {
    assert!(
        primary.measured.keys().eq(repeat.measured.keys()),
        "primary and repeat measured-width populations differ"
    );
    let mut all = Vec::new();
    for (key, row) in &primary.measured {
        let repeated = repeat
            .measured
            .get(key)
            .expect("primary and repeat measured populations agree");
        all.push((row.p50 - repeated.p50).abs() / row.p50);
    }
    all.sort_by(f64::total_cmp);
    let choose = |_run: &Run, rows, contributors, cell: &[Row]| {
        if selected.fitted() {
            choose_predicted(
                models.get(&selected).expect("selected model exists"),
                rows,
                contributors,
                cell,
            )
        } else {
            choose_direct(selected, rows, contributors, cell)
        }
    };
    let (_, primary_choices) = score_cells(primary, &HELD_OUT_CONTRIBUTORS, |r, c, cell| {
        choose(primary, r, c, cell)
    });
    let (_, repeat_choices) = score_cells(repeat, &HELD_OUT_CONTRIBUTORS, |r, c, cell| {
        choose(repeat, r, c, cell)
    });
    let mut selected_differences = Vec::new();
    let mut plateau_agreement = 0_usize;
    for (primary_choice, repeat_choice) in primary_choices.iter().zip(&repeat_choices) {
        assert_eq!(
            (primary_choice.0, primary_choice.1),
            (repeat_choice.0, repeat_choice.1)
        );
        assert_eq!(primary_choice.2.participants, repeat_choice.2.participants);
        selected_differences
            .push((primary_choice.2.p50 - repeat_choice.2.p50).abs() / primary_choice.2.p50);
        plateau_agreement += usize::from(primary_choice.4 == repeat_choice.4);
    }
    selected_differences.sort_by(f64::total_cmp);
    println!("## Repeatability");
    println!();
    println!("rows_compared\t{}", all.len());
    println!(
        "all_rows_upper_median_relative_difference\t{:.6}",
        all[all.len() / 2]
    );
    println!(
        "all_rows_maximum_relative_difference\t{:.6}",
        all[all.len() - 1]
    );
    println!(
        "selected_rows_upper_median_relative_difference\t{:.6}",
        selected_differences[selected_differences.len() / 2]
    );
    println!(
        "selected_rows_maximum_relative_difference\t{:.6}",
        selected_differences[selected_differences.len() - 1]
    );
    println!("selected_plateau_membership_agreement\t{plateau_agreement}/20");
}

/// Prints the dense and sparse anchor comparisons at all four rows.
fn print_anchor_boundaries(label: &str, run: &Run) {
    println!("## Anchor boundaries, {label}");
    println!();
    println!("shape\traw_best_P\tplateau_P\tproduction_P\tproduction_regret");
    for rows in MATRIX_ROWS {
        for contributors in ANCHOR_CONTRIBUTORS {
            let cell = run.cell(rows, contributors);
            let best = raw_best(&cell);
            let plateau = cell
                .iter()
                .filter(|row| row.p50 - best.p50 <= separation_band(**row, best))
                .map(|row| row.participants.to_string())
                .collect::<Vec<_>>()
                .join(",");
            let production = cell
                .iter()
                .find(|row| row.production)
                .expect("an anchor production row is admitted");
            println!(
                "{rows}x{contributors}\t{}\t{plateau}\t{}\t{:.6}",
                best.participants,
                production.participants,
                production.p50 / best.p50
            );
        }
    }
    println!();
    println!("shape\tleft_P\tright_P\tverdict");
    for rows in MATRIX_ROWS {
        for (contributors, left, right) in [(780, 195, 260), (780, 260, 390), (1_042, 2, 521)] {
            let left = run.row(rows, contributors, left);
            let right = run.row(rows, contributors, right);
            let delta = left.p50 - right.p50;
            let band = separation_band(left, right);
            let verdict = if delta > band {
                "right-is-faster"
            } else if -delta > band {
                "left-is-faster"
            } else {
                "within-noise"
            };
            println!(
                "{rows}x{contributors}\t{}\t{}\t{verdict}",
                left.participants, right.participants
            );
        }
    }
}

/// Every exact arithmetic width offered to preparation.
fn admissible_participants(contributors: u64) -> Vec<u64> {
    (2..=contributors / 2)
        .filter(|participants| contributors.is_multiple_of(*participants))
        .collect()
}

/// Current production nearest-256 width, independently restated for validation.
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
                .expect("every matrix contributor is composite")
        });
    widths
        .iter()
        .copied()
        .find(|participants| *participants > 256 && *participants < 512 - below)
        .unwrap_or(below)
}

/// Entry point used by the distinct fresh-interaction analyzer binary.
#[allow(
    dead_code,
    reason = "called by the distinct interaction analyzer wrapper"
)]
pub(crate) fn interaction_main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    if arguments == ["--self-check"] {
        interaction_self_check();
        return;
    }
    let [sweep_path, repeat_path, environment_path] = arguments.as_slice() else {
        panic!(
            "usage: tree-width-interactions-analysis <sweep.tsv> <repeat.tsv> \
             <environment.tsv> | --self-check"
        );
    };
    let environment = parse_environment(environment_path);
    validate_interaction_environment(&environment);
    for (key, path) in [
        ("hash.main", "src/main.rs"),
        ("hash.regions", "src/regions.rs"),
        ("hash.buffer", "src/buffer.rs"),
        ("hash.analysis_wrapper", "src/interactions.rs"),
        ("hash.analysis_shared", "src/shape_aware.rs"),
        ("hash.cargo_toml", "Cargo.toml"),
        ("hash.cargo_lock", "Cargo.lock"),
        (
            "hash.compiler_physical",
            "../../../crates/tiler-compiler/src/physical.rs",
        ),
        (
            "hash.compiler_measured_cost",
            "../../../crates/tiler-compiler/src/measured_cost.rs",
        ),
        (
            "hash.compiler_target",
            "../../../crates/tiler-compiler/src/target.rs",
        ),
        (
            "hash.metal_declaration",
            "../../../crates/tiler-build/src/metal_declaration.rs",
        ),
        ("hash.sweep", sweep_path),
        ("hash.repeat", repeat_path),
    ] {
        validate_digest(&environment, key, path);
    }
    let primary = parse_interaction_run(sweep_path);
    let repeat = parse_interaction_run(repeat_path);
    validate_run_custody(&environment, &primary, "primary");
    validate_run_custody(&environment, &repeat, "repeat");
    println!("# analysis\treduction-tree-width-interactions");
    println!("# validation\tpassed");
    println!("# shapes\t70");
    println!("# arithmetic_variants_per_run\t625");
    println!("# measured_primary\t{}", primary.measured.len());
    println!("# measured_repeat\t{}", repeat.measured.len());
    println!();
    analyze_interactions(&primary, &repeat);
}

/// Exercises the exact interaction population, features, solver, and gate.
#[allow(
    dead_code,
    reason = "called through the distinct interaction analyzer wrapper"
)]
fn interaction_self_check() {
    assert_eq!(INTERACTION_ROWS, [8, 528, 1_056, 2_112, 8_192]);
    assert_eq!(INTERACTION_ANCHORS, [780, 1_042]);
    assert_eq!(INTERACTION_FIT, [774, 783, 900, 1_006, 1_082, 1_280]);
    assert_eq!(INTERACTION_HELD, [775, 785, 899, 1_008, 1_094, 1_282]);
    assert!(
        INTERACTION_FIT
            .iter()
            .chain(&INTERACTION_HELD)
            .all(
                |contributors| ![756, 779, 840, 1_018, 1_020, 768, 781, 960, 1_022, 1_046]
                    .contains(contributors)
            )
    );
    let widths_per_row = INTERACTION_ALL
        .iter()
        .map(|contributors| admissible_participants(*contributors).len())
        .sum::<usize>();
    assert_eq!(widths_per_row, 125, "the interaction width census moved");
    assert_eq!(widths_per_row * INTERACTION_ROWS.len(), 625);
    assert_eq!(
        INTERACTION_FIT
            .iter()
            .map(|contributors| admissible_participants(*contributors).len())
            .sum::<usize>(),
        61
    );
    assert_eq!(
        INTERACTION_HELD
            .iter()
            .map(|contributors| admissible_participants(*contributors).len())
            .sum::<usize>(),
        40
    );
    assert_eq!(features(Family::Contributor, 8, 900, 30).len(), 5);
    assert_eq!(features(Family::RowsContributor, 8, 900, 30).len(), 10);
    assert_eq!(features(Family::Lattice, 8, 900, 30).len(), 15);
    assert_eq!(features(Family::Interaction, 8, 900, 30).len(), 18);

    let mut measured = BTreeMap::new();
    for rows in INTERACTION_ROWS {
        for contributors in INTERACTION_ALL {
            let production = production_width(contributors);
            for participants in admissible_participants(contributors) {
                let work = rows * (contributors + participants);
                let depth = contributors / participants + participants;
                let regime = (exact_f64(rows, "synthetic row") / 1_056.0)
                    .log2()
                    .clamp(-1.0, 1.0);
                let interaction =
                    exact_f64(participants, "synthetic width") * (1.0 + regime / 20.0);
                let p50 = exact_f64(work.max(1_056 * depth), "synthetic cost") + interaction;
                measured.insert(
                    (rows, contributors, participants),
                    Row {
                        rows,
                        contributors,
                        participants,
                        production: participants == production,
                        p50,
                        stddev: p50 / 100.0,
                    },
                );
            }
        }
    }
    let run = Run {
        measured,
        metadata: BTreeMap::new(),
    };
    for family in interaction_fitted_families() {
        let (lambda, objective) =
            choose_lambda_for(&run, family, &INTERACTION_FIT, &INTERACTION_ROWS);
        assert!(objective.worst.is_finite());
        assert!(
            fit_model_for_rows(&run, family, lambda, &INTERACTION_FIT, &INTERACTION_ROWS).is_some()
        );
    }
    let selection = select_interaction_family(&run);
    println!("# self_check\tpassed");
    println!("# widths_per_row\t{widths_per_row}");
    println!(
        "# variants_per_run\t{}",
        widths_per_row * INTERACTION_ROWS.len()
    );
    println!("# fit_selected_family\t{}", selection.selected.family.key());
    println!("# interaction_eligible\t{}", selection.interaction_eligible);
}

/// The four fitted families, in exact least-information order.
#[allow(
    dead_code,
    reason = "called through the distinct interaction analyzer wrapper"
)]
const fn interaction_fitted_families() -> [Family; 4] {
    [
        Family::Contributor,
        Family::RowsContributor,
        Family::Lattice,
        Family::Interaction,
    ]
}

/// Parses and validates one fresh-interaction retained TSV.
#[allow(
    dead_code,
    clippy::too_many_lines,
    reason = "called by the distinct wrapper; one pass binds schema, resources, production marks, metadata, and the complete finite population"
)]
fn parse_interaction_run(path: &str) -> Run {
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
            assert_eq!(
                line,
                "rows\tcontributors\telements\tstrategy\tpartitions\tper_partition\tproduction\twidest_workgroup\tthreadgroup_bytes\tencoders\treps\tbatch\tsubmit_min_us\tsubmit_p50_us\tsubmit_p90_us\tsubmit_stddev_us\tbatch_min_us\tbatch_p50_us\tbatch_p90_us\tbatch_stddev_us\tamortized_min_us\tamortized_p50_us\tamortized_stddev_us\tstatus",
                "{path}: result schema moved"
            );
            assert!(!header_seen, "{path}: duplicate result header");
            header_seen = true;
            continue;
        }
        assert!(header_seen, "{path}: result row precedes its header");
        let fields: Vec<&str> = line.split('\t').collect();
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
        assert!(outcomes.insert((rows, contributors, participants)));
        let production = fields[6] == "production";
        marked
            .entry((rows, contributors))
            .or_default()
            .push((participants, production));
        let row = Row {
            rows,
            contributors,
            participants,
            production,
            p50: float(fields[21], "amortized p50"),
            stddev: float(fields[22], "amortized spread"),
        };
        assert!(row.p50 > 0.0 && row.stddev >= 0.0);
        assert_eq!(number(fields[7], "widest workgroup"), participants);
        let requested = 4 * participants;
        assert_eq!(
            number(fields[8], "threadgroup bytes"),
            requested.div_ceil(16) * 16,
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
    for (key, expected) in [
        ("spike", "reduction-tree-width-interactions"),
        ("mode", "TreeWidthInteractions"),
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
        ("shapes", "70"),
        ("variants_attempted", "625"),
        ("variants_measured", "625"),
        ("variants_declined", "0"),
        ("widest_prepared_workgroup", "641"),
        ("maximum_prepared_threadgroup_bytes", "2576"),
    ] {
        assert_eq!(
            metadata.get(key).map(String::as_str),
            Some(expected),
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
    let mut expected = BTreeSet::new();
    for rows in INTERACTION_ROWS {
        for contributors in INTERACTION_ALL {
            let widths = admissible_participants(contributors);
            let production = production_width(contributors);
            assert!(measured.contains_key(&(rows, contributors, production)));
            assert_eq!(
                marked[&(rows, contributors)]
                    .iter()
                    .filter(|(_, marked)| *marked)
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
    assert_eq!(expected.len(), 625);
    if outcomes != expected {
        let missing: Vec<_> = expected.difference(&outcomes).copied().collect();
        let unexpected: Vec<_> = outcomes.difference(&expected).copied().collect();
        panic!(
            "{path}: exact outcome population moved: missing {missing:?}; unexpected {unexpected:?}"
        );
    }
    assert_eq!(measured.len(), 625);
    Run { measured, metadata }
}

/// Selection details including the explicit nested richness decision.
#[allow(
    dead_code,
    reason = "used by the distinct interaction analyzer wrapper"
)]
struct InteractionSelection {
    selected: FamilySelection,
    lambdas: BTreeMap<Family, (f64, Objective)>,
    interaction_eligible: bool,
    strictly_better_folds: usize,
}

/// Selects the simpler winner, then admits the richer family only by its gate.
#[allow(
    dead_code,
    clippy::too_many_lines,
    reason = "called through the distinct wrapper; keeping lambda selection, simpler-family ordering, and the nested fold gate together makes the frozen decision protocol directly auditable"
)]
fn select_interaction_family(run: &Run) -> InteractionSelection {
    let mut lambdas = BTreeMap::new();
    for family in interaction_fitted_families() {
        lambdas.insert(
            family,
            choose_lambda_for(run, family, &INTERACTION_FIT, &INTERACTION_ROWS),
        );
    }
    let saturation = score_cells_for(
        run,
        &INTERACTION_FIT,
        &INTERACTION_ROWS,
        |rows, contributors, cell| {
            choose_direct(Family::ExistingSaturation, rows, contributors, cell)
        },
    )
    .0;
    let simpler = [
        FamilySelection {
            family: Family::Contributor,
            objective: lambdas[&Family::Contributor].1,
        },
        FamilySelection {
            family: Family::RowsContributor,
            objective: lambdas[&Family::RowsContributor].1,
        },
        FamilySelection {
            family: Family::Lattice,
            objective: lambdas[&Family::Lattice].1,
        },
        FamilySelection {
            family: Family::ExistingSaturation,
            objective: saturation,
        },
    ]
    .into_iter()
    .min_by(|left, right| {
        left.objective
            .compare(right.objective)
            .then_with(|| left.family.cmp(&right.family))
    })
    .expect("four simpler families are compared");

    let lattice_lambda = lambdas[&Family::Lattice].0;
    let interaction_lambda = lambdas[&Family::Interaction].0;
    let mut every_fold_no_worse = true;
    let mut strictly_better_folds = 0_usize;
    for held in INTERACTION_FIT {
        let training: Vec<u64> = INTERACTION_FIT
            .into_iter()
            .filter(|candidate| *candidate != held)
            .collect();
        let lattice = fit_model_for_rows(
            run,
            Family::Lattice,
            lattice_lambda,
            &training,
            &INTERACTION_ROWS,
        )
        .expect("the selected aggregate-LOO lattice lambda fits every fold");
        let interaction = fit_model_for_rows(
            run,
            Family::Interaction,
            interaction_lambda,
            &training,
            &INTERACTION_ROWS,
        )
        .expect("the selected aggregate-LOO interaction lambda fits every fold");
        let lattice_objective = score_cells_for(
            run,
            &[held],
            &INTERACTION_ROWS,
            |rows, contributors, cell| choose_predicted(&lattice, rows, contributors, cell),
        )
        .0;
        let interaction_objective = score_cells_for(
            run,
            &[held],
            &INTERACTION_ROWS,
            |rows, contributors, cell| choose_predicted(&interaction, rows, contributors, cell),
        )
        .0;
        let no_worse = interaction_objective
            .worst
            .total_cmp(&lattice_objective.worst)
            .is_le()
            && interaction_objective
                .median
                .total_cmp(&lattice_objective.median)
                .is_le()
            && interaction_objective.misses <= lattice_objective.misses;
        let strict = no_worse
            && (interaction_objective
                .worst
                .total_cmp(&lattice_objective.worst)
                .is_lt()
                || interaction_objective
                    .median
                    .total_cmp(&lattice_objective.median)
                    .is_lt()
                || interaction_objective.misses < lattice_objective.misses);
        every_fold_no_worse &= no_worse;
        strictly_better_folds += usize::from(strict);
    }
    let interaction_objective = lambdas[&Family::Interaction].1;
    let interaction_eligible = every_fold_no_worse
        && strictly_better_folds >= 3
        && interaction_objective.compare(simpler.objective).is_lt();
    let selected = if interaction_eligible {
        FamilySelection {
            family: Family::Interaction,
            objective: interaction_objective,
        }
    } else {
        simpler
    };
    InteractionSelection {
        selected,
        lambdas,
        interaction_eligible,
        strictly_better_folds,
    }
}

/// Runs fit-only selection, sealed scoring, repeat diagnostics, and anchors.
#[allow(
    dead_code,
    clippy::too_many_lines,
    reason = "called by the distinct wrapper; the report keeps the frozen decision sequence visible"
)]
fn analyze_interactions(primary: &Run, repeat: &Run) {
    let selection = select_interaction_family(primary);
    let repeat_selection = select_interaction_family(repeat);
    println!("## Fit-only model selection");
    println!();
    println!("family\tlambda\tworst\tupper_median\tplateau_misses");
    let mut models = BTreeMap::new();
    for family in interaction_fitted_families() {
        let (lambda, objective) = selection.lambdas[&family];
        println!(
            "{}\t{lambda:.6}\t{:.6}\t{:.6}\t{}",
            family.key(),
            objective.worst,
            objective.median,
            objective.misses
        );
        models.insert(
            family,
            fit_model_for_rows(primary, family, lambda, &INTERACTION_FIT, &INTERACTION_ROWS)
                .expect("the selected lambda fits all primary fit contributors"),
        );
    }
    let saturation = score_cells_for(
        primary,
        &INTERACTION_FIT,
        &INTERACTION_ROWS,
        |rows, contributors, cell| {
            choose_direct(Family::ExistingSaturation, rows, contributors, cell)
        },
    )
    .0;
    println!(
        "{}\t-\t{:.6}\t{:.6}\t{}",
        Family::ExistingSaturation.key(),
        saturation.worst,
        saturation.median,
        saturation.misses
    );
    println!(
        "interaction_strictly_better_folds\t{}",
        selection.strictly_better_folds
    );
    println!("interaction_eligible\t{}", selection.interaction_eligible);
    println!("selected_family\t{}", selection.selected.family.key());
    println!(
        "repeat_refit_selected_family\t{}",
        repeat_selection.selected.family.key()
    );
    println!(
        "repeat_refit_interaction_eligible\t{}",
        repeat_selection.interaction_eligible
    );
    println!();

    println!("## Final fitted parameters");
    println!();
    for model in models.values() {
        println!("family\t{}", model.family.key());
        println!("lambda\t{:.6}", model.lambda);
        println!("intercept\t{:.12}", model.coefficients[0]);
        for index in 0..model.means.len() {
            println!(
                "feature_{index}\tmean={:.12}\tscale={:.12}\tcoefficient={:.12}",
                model.means[index],
                model.scales[index],
                model.coefficients[index + 1]
            );
        }
        println!();
    }

    println!("## Held-out summaries");
    println!();
    println!("run\tfamily\tworst\tupper_median\tplateau_misses");
    let mut summaries = BTreeMap::new();
    for (label, run) in [("primary", primary), ("repeat", repeat)] {
        for family in interaction_fitted_families() {
            let model = &models[&family];
            let objective = score_cells_for(
                run,
                &INTERACTION_HELD,
                &INTERACTION_ROWS,
                |rows, contributors, cell| choose_predicted(model, rows, contributors, cell),
            )
            .0;
            println!(
                "{label}\t{}\t{:.6}\t{:.6}\t{}",
                family.key(),
                objective.worst,
                objective.median,
                objective.misses
            );
            summaries.insert((label, family), objective);
        }
        let objective = score_cells_for(
            run,
            &INTERACTION_HELD,
            &INTERACTION_ROWS,
            |rows, contributors, cell| {
                choose_direct(Family::ExistingSaturation, rows, contributors, cell)
            },
        )
        .0;
        println!(
            "{label}\t{}\t{:.6}\t{:.6}\t{}",
            Family::ExistingSaturation.key(),
            objective.worst,
            objective.median,
            objective.misses
        );
        summaries.insert((label, Family::ExistingSaturation), objective);
        let production = interaction_production_objective(run);
        println!(
            "{label}\tproduction-nearest-256\t{:.6}\t{:.6}\t{}",
            production.worst, production.median, production.misses
        );
    }
    let selected = selection.selected.family;
    let supported = interaction_qualifies(
        summaries[&("primary", selected)],
        interaction_production_objective(primary),
    ) && interaction_qualifies(
        summaries[&("repeat", selected)],
        interaction_production_objective(repeat),
    );
    println!();
    println!("fit_selected_family\t{}", selected.key());
    println!(
        "support_verdict\t{}",
        if supported {
            "supported-on-this-finite-qualified-host-population"
        } else {
            "finite-population-insufficient"
        }
    );
    println!();
    print_interaction_repeatability(primary, repeat, selected, &models);
    println!();
    print_interaction_anchors("primary", primary);
    println!();
    print_interaction_anchors("repeat", repeat);
}

/// Scores unchanged production over the sealed fresh contributors.
#[allow(
    dead_code,
    reason = "called through the distinct interaction analyzer wrapper"
)]
fn interaction_production_objective(run: &Run) -> Objective {
    score_cells_for(run, &INTERACTION_HELD, &INTERACTION_ROWS, |_, _, cell| {
        *cell.iter().find(|row| row.production).expect("production")
    })
    .0
}

/// Applies every support clause independently to one retained run.
#[allow(
    dead_code,
    reason = "called through the distinct interaction analyzer wrapper"
)]
fn interaction_qualifies(candidate: Objective, production: Objective) -> bool {
    candidate.worst <= 1.10
        && candidate.median <= 1.02
        && candidate.misses <= 3
        && candidate.worst < production.worst
        && candidate.median <= production.median
}

/// Reports primary/repeat variation without allowing repeat refitting to select.
#[allow(
    dead_code,
    reason = "called through the distinct interaction analyzer wrapper"
)]
fn print_interaction_repeatability(
    primary: &Run,
    repeat: &Run,
    selected: Family,
    models: &BTreeMap<Family, Model>,
) {
    assert!(primary.measured.keys().eq(repeat.measured.keys()));
    let mut all: Vec<f64> = primary
        .measured
        .iter()
        .map(|(key, row)| {
            let repeated = repeat.measured.get(key).expect("matching repeated row");
            (row.p50 - repeated.p50).abs() / row.p50
        })
        .collect();
    all.sort_by(f64::total_cmp);
    let choose = |rows, contributors, cell: &[Row]| {
        if selected.fitted() {
            choose_predicted(&models[&selected], rows, contributors, cell)
        } else {
            choose_direct(selected, rows, contributors, cell)
        }
    };
    let (_, primary_choices) =
        score_cells_for(primary, &INTERACTION_HELD, &INTERACTION_ROWS, choose);
    let (_, repeat_choices) = score_cells_for(repeat, &INTERACTION_HELD, &INTERACTION_ROWS, choose);
    let mut selected_differences = Vec::new();
    let mut plateau_agreement = 0_usize;
    for (left, right) in primary_choices.iter().zip(&repeat_choices) {
        assert_eq!(
            (left.0, left.1, left.2.participants),
            (right.0, right.1, right.2.participants)
        );
        selected_differences.push((left.2.p50 - right.2.p50).abs() / left.2.p50);
        plateau_agreement += usize::from(left.4 == right.4);
    }
    selected_differences.sort_by(f64::total_cmp);
    println!("## Repeatability");
    println!();
    println!(
        "population\tupper_median_relative_p50_difference\tmaximum_relative_p50_difference\tplateau_agreement"
    );
    println!(
        "all-widths\t{:.6}\t{:.6}\t-",
        all[all.len() / 2],
        all[all.len() - 1]
    );
    println!(
        "selected-held-out\t{:.6}\t{:.6}\t{plateau_agreement}/30",
        selected_differences[selected_differences.len() / 2],
        selected_differences[selected_differences.len() - 1]
    );
}

/// Reports the predeclared recurrence comparisons, never used for selection.
#[allow(
    dead_code,
    reason = "called through the distinct interaction analyzer wrapper"
)]
fn print_interaction_anchors(label: &str, run: &Run) {
    println!("## {label} anchor raw minima and plateaus");
    println!();
    println!("shape\traw_minimum\tplateau\tproduction\tproduction_regret");
    for rows in INTERACTION_ROWS {
        for contributors in INTERACTION_ANCHORS {
            let cell = run.cell(rows, contributors);
            let best = raw_best(&cell);
            let production = *cell
                .iter()
                .find(|row| row.production)
                .expect("anchor production");
            let plateau = cell
                .iter()
                .filter(|row| row.p50 - best.p50 <= separation_band(**row, best))
                .map(|row| row.participants.to_string())
                .collect::<Vec<_>>()
                .join(",");
            println!(
                "{rows}x{contributors}\t{}\t{plateau}\t{}\t{:.6}",
                best.participants,
                production.participants,
                production.p50 / best.p50
            );
        }
    }
    println!();
    println!("## {label} anchor comparisons");
    println!();
    println!("shape\tleft\tright\tverdict");
    for rows in INTERACTION_ROWS {
        for (contributors, left_width, right_width) in
            [(780, 195, 260), (780, 260, 390), (1_042, 2, 521)]
        {
            let left = run.row(rows, contributors, left_width);
            let right = run.row(rows, contributors, right_width);
            let delta = left.p50 - right.p50;
            let band = separation_band(left, right);
            let verdict = if delta > band {
                "right-is-faster"
            } else if -delta > band {
                "left-is-faster"
            } else {
                "within-noise"
            };
            println!("{rows}x{contributors}\t{left_width}\t{right_width}\t{verdict}");
        }
    }
}

/// Exact fresh-study fields known before timing; dynamic rows are frozen later.
#[allow(
    dead_code,
    reason = "used by the distinct interaction analyzer wrapper"
)]
const INTERACTION_CLAIMED_ENVIRONMENT: &[(&str, &str)] = &[
    ("environment.date_utc.primary_start", "2026-08-11T10:02:50Z"),
    ("environment.date_utc.primary_end", "2026-08-11T10:06:56Z"),
    ("environment.date_utc.repeat_start", "2026-08-11T10:07:04Z"),
    ("environment.date_utc.repeat_end", "2026-08-11T10:11:13Z"),
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

/// Digest keys binding the fresh result to every source it claims.
#[allow(
    dead_code,
    reason = "used by the distinct interaction analyzer wrapper"
)]
const INTERACTION_DIGEST_KEYS: [&str; 13] = [
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

/// Equality-pins every claim-bearing fresh environment field and key.
#[allow(
    dead_code,
    reason = "called through the distinct interaction analyzer wrapper"
)]
fn validate_interaction_environment(environment: &BTreeMap<String, String>) {
    for (key, expected) in INTERACTION_CLAIMED_ENVIRONMENT {
        assert_eq!(
            environment.get(*key).map(String::as_str),
            Some(*expected),
            "environment key `{key}` moved"
        );
    }
    let expected: BTreeSet<&str> = INTERACTION_CLAIMED_ENVIRONMENT
        .iter()
        .map(|(key, _)| *key)
        .chain(INTERACTION_DIGEST_KEYS)
        .collect();
    assert_eq!(
        environment
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        expected,
        "the exact environment key population moved"
    );
}

/// Parses the two-column retained environment record.
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

/// Exact claim-bearing environment fields. Timed values are filled only after
/// the quiet-window run and deliberately make a premature result fail.
const CLAIMED_ENVIRONMENT: &[(&str, &str)] = &[
    ("environment.date_utc.primary", "2026-08-11T08:39:16Z"),
    ("environment.date_utc.repeat", "2026-08-11T08:46:42Z"),
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

/// Digest rows binding retained sources and results.
const DIGEST_KEYS: [&str; 10] = [
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

/// Pins every claim-bearing environment field and the exact key population.
fn validate_environment(environment: &BTreeMap<String, String>) {
    for (key, expected) in CLAIMED_ENVIRONMENT {
        assert_eq!(
            environment.get(*key).map(String::as_str),
            Some(*expected),
            "environment key `{key}` moved"
        );
    }
    let expected: BTreeSet<&str> = CLAIMED_ENVIRONMENT
        .iter()
        .map(|(key, _)| *key)
        .chain(DIGEST_KEYS)
        .collect();
    assert_eq!(
        environment
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        expected,
        "the exact environment key population moved"
    );
}

/// Requires one retained subject to match its environment digest.
fn validate_digest(environment: &BTreeMap<String, String>, key: &str, path: impl AsRef<Path>) {
    let path = path.as_ref();
    let expected = environment
        .get(key)
        .unwrap_or_else(|| panic!("missing digest key `{key}`"));
    let bytes =
        fs::read(path).unwrap_or_else(|error| panic!("{} does not read: {error}", path.display()));
    let mut observed = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        write!(observed, "{byte:02x}").expect("writing to a String cannot fail");
    }
    assert_eq!(&observed, expected, "{} digest moved", path.display());
}

/// Parses one unsigned field.
fn number(value: &str, what: &str) -> u64 {
    value
        .parse()
        .unwrap_or_else(|error| panic!("{what} `{value}` does not parse: {error}"))
}

/// Parses one finite floating-point field.
fn float(value: &str, what: &str) -> f64 {
    let parsed: f64 = value
        .parse()
        .unwrap_or_else(|error| panic!("{what} `{value}` does not parse: {error}"));
    assert!(parsed.is_finite(), "{what} `{value}` is not finite");
    parsed
}

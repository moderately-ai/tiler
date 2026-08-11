//! Validates and analyzes the retained tree-width excursion measurement.
//!
//! The measuring binary needs the qualified Metal host and offline toolchain;
//! this binary needs neither. It refuses any result whose metadata, finite
//! matrix, admissible participant population, production-width mark, resource
//! fields, environment row, or retained digest differs from the predeclared
//! experiment. Only after those checks does it compare costs. A missing row
//! therefore cannot look like a small or flat curve.
//!
//! ```sh
//! cargo run --release --bin tree-width-excursion-analysis -- \
//!   results/<row>/sweep.tsv results/<row>/repeat.tsv results/<row>/environment.tsv
//! ```

use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Exact predeclared shapes and their production-selected participant counts.
const SHAPES: [(u64, u64, u64); 6] = [
    (4, 514, 257),
    (16_384, 514, 257),
    (4, 780, 260),
    (16_384, 780, 260),
    (4, 1_042, 2),
    (16_384, 1_042, 2),
];

/// One measured tree width.
#[derive(Clone, Copy, Debug)]
struct Row {
    rows: u64,
    contributors: u64,
    participants: u64,
    production: bool,
    p50: f64,
    stddev: f64,
}

impl Row {
    /// Standard error used by the retained partition calibration.
    fn standard_error(self) -> f64 {
        self.stddev / 30.0_f64.sqrt()
    }
}

/// One validated retained run.
struct Run {
    rows: BTreeMap<(u64, u64, u64), Row>,
}

impl Run {
    /// Every row for one shape in ascending participant order.
    fn cell(&self, rows: u64, contributors: u64) -> Vec<Row> {
        self.rows
            .values()
            .filter(|row| row.rows == rows && row.contributors == contributors)
            .copied()
            .collect()
    }

    /// One exact participant row.
    fn row(&self, rows: u64, contributors: u64, participants: u64) -> Row {
        *self
            .rows
            .get(&(rows, contributors, participants))
            .unwrap_or_else(|| {
                panic!("{rows}x{contributors} carries no P{participants} measurement")
            })
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [sweep_path, repeat_path, environment_path] = arguments.as_slice() else {
        panic!("usage: tree-width-excursion-analysis <sweep.tsv> <repeat.tsv> <environment.tsv>");
    };

    let environment = parse_environment(environment_path);
    validate_environment(&environment);
    validate_digest(&environment, "hash.main", "src/main.rs");
    validate_digest(&environment, "hash.regions", "src/regions.rs");
    validate_digest(&environment, "hash.buffer", "src/buffer.rs");
    validate_digest(&environment, "hash.analysis", "src/excursion.rs");
    validate_digest(&environment, "hash.sweep", sweep_path);
    validate_digest(&environment, "hash.repeat", repeat_path);

    let sweep = parse_run(sweep_path);
    let repeat = parse_run(repeat_path);

    println!("# analysis\treduction-tree-width-excursion");
    println!("# validation\tpassed");
    println!("# shapes\t{}", SHAPES.len());
    println!("# variants_per_run\t{}", sweep.rows.len());
    println!();
    print_primary(&sweep);
    println!();
    print_boundary_comparisons("primary", &sweep);
    println!();
    print_repeatability(&sweep, &repeat);
    println!();
    print_boundary_comparisons("repeat", &repeat);
}

/// Parses and fully validates one retained sweep.
#[allow(
    clippy::too_many_lines,
    reason = "one linear scan validates the 24-field row schema, fixed metadata, exact matrix, and production marks together so no partially checked Run can escape"
)]
fn parse_run(path: &str) -> Run {
    let text =
        fs::read_to_string(path).unwrap_or_else(|error| panic!("{path} does not read: {error}"));
    let mut metadata: BTreeMap<String, String> = BTreeMap::new();
    let mut rows = BTreeMap::new();
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
        assert!(header_seen, "{path}: measurement row precedes its header");
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(
            fields.len(),
            24,
            "{path}: a measurement row carries 24 fields: {line}"
        );
        assert_eq!(
            fields[3], "single-workgroup-tree",
            "{path}: the excursion measures only the tree"
        );
        assert_eq!(fields[23], "measured", "{path}: every variant must measure");
        let row = Row {
            rows: number(fields[0], "rows"),
            contributors: number(fields[1], "contributors"),
            participants: number(fields[4], "participants"),
            production: fields[6] == "production",
            p50: float(fields[21], "amortized p50"),
            stddev: float(fields[22], "amortized spread"),
        };
        assert!(row.p50 > 0.0, "{path}: every amortized median is positive");
        assert!(
            row.stddev >= 0.0,
            "{path}: every amortized spread is non-negative"
        );
        assert!(
            matches!(fields[6], "production" | "-"),
            "{path}: selection marker `{}` is neither production nor alternative",
            fields[6]
        );
        assert_eq!(number(fields[2], "elements"), row.rows * row.contributors);
        assert_eq!(
            number(fields[5], "contributors per partition"),
            row.contributors / row.participants
        );
        assert_eq!(number(fields[7], "widest workgroup"), row.participants);
        let requested_threadgroup_bytes = 4 * row.participants;
        let prepared_threadgroup_bytes = requested_threadgroup_bytes.div_ceil(16) * 16;
        assert_eq!(
            number(fields[8], "threadgroup bytes"),
            prepared_threadgroup_bytes,
            "{path}: P{} requests {} source bytes and the prepared Metal function must report the observed 16-byte-aligned allocation",
            row.participants,
            requested_threadgroup_bytes
        );
        assert_eq!(number(fields[9], "encoders"), 2);
        assert_eq!(number(fields[10], "repetitions"), 30);
        assert_eq!(number(fields[11], "batch"), 64);
        assert!(
            row.contributors.is_multiple_of(row.participants),
            "{path}: P{} does not divide {}",
            row.participants,
            row.contributors
        );
        assert!(
            row.contributors / row.participants >= 2,
            "{path}: P{} folds fewer than two contributors",
            row.participants
        );
        assert!(
            rows.insert((row.rows, row.contributors, row.participants), row)
                .is_none(),
            "{path}: duplicate row {}x{} P{}",
            row.rows,
            row.contributors,
            row.participants
        );
    }

    assert!(header_seen, "{path}: missing result header");

    for (key, expected) in [
        ("spike", "reduction-tree-width-excursion"),
        ("mode", "TreeWidthExcursion"),
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
        ("shapes", "6"),
        ("variants_attempted", "52"),
        ("variants_measured", "52"),
        ("variants_declined", "0"),
    ] {
        assert_eq!(
            metadata.get(key).map(String::as_str),
            Some(expected),
            "{path}: metadata `{key}` moved"
        );
    }
    for key in ["load_before", "load_after"] {
        assert!(metadata.contains_key(key), "{path}: missing `{key}`");
    }

    let mut expected_keys = BTreeSet::new();
    for (shape_rows, contributors, production) in SHAPES {
        let admissible = admissible_participants(contributors);
        for participants in &admissible {
            expected_keys.insert((shape_rows, contributors, *participants));
        }
        let cell: Vec<Row> = rows
            .values()
            .filter(|row| row.rows == shape_rows && row.contributors == contributors)
            .copied()
            .collect();
        assert_eq!(
            cell.len(),
            admissible.len(),
            "{path}: {shape_rows}x{contributors} did not measure its full admissible population"
        );
        let marked: Vec<u64> = cell
            .iter()
            .filter(|row| row.production)
            .map(|row| row.participants)
            .collect();
        assert_eq!(
            marked,
            vec![production],
            "{path}: {shape_rows}x{contributors} production mark moved"
        );
    }
    assert_eq!(
        rows.keys().copied().collect::<BTreeSet<_>>(),
        expected_keys,
        "{path}: the exact finite row population moved"
    );
    assert_eq!(rows.len(), 52, "{path}: the exact 52-row population moved");
    Run { rows }
}

/// Prints the primary run's per-shape production and best widths.
fn print_primary(run: &Run) {
    println!("## Production against the best measured width");
    println!();
    println!(
        "shape\tproduction_P\tproduction_us\tbest_P\tbest_us\tratio\tband_us\tverdict\tplateau"
    );
    for (rows, contributors, production) in SHAPES {
        let cell = run.cell(rows, contributors);
        let production = run.row(rows, contributors, production);
        let best = *cell
            .iter()
            .min_by(|left, right| left.p50.total_cmp(&right.p50))
            .expect("every cell is non-empty");
        let band = separation_band(production, best);
        let separated = production.p50 - best.p50 > band;
        let plateau: Vec<u64> = cell
            .iter()
            .filter(|row| row.p50 - best.p50 <= separation_band(**row, best))
            .map(|row| row.participants)
            .collect();
        println!(
            "{}x{}\t{}\t{:.4}\t{}\t{:.4}\t{:.3}\t{:.4}\t{}\t{}",
            rows,
            contributors,
            production.participants,
            production.p50,
            best.participants,
            best.p50,
            production.p50 / best.p50,
            band,
            if separated {
                "production is beaten"
            } else {
                "within noise of best"
            },
            join(&plateau),
        );
    }
}

/// Prints the four comparisons the matrix was designed to answer.
fn print_boundary_comparisons(label: &str, run: &Run) {
    println!("## Boundary comparisons, {label}");
    println!();
    println!("shape\tleft_P\tleft_us\tright_P\tright_us\tband_us\tverdict");
    for rows in [4_u64, 16_384] {
        for (contributors, left, right) in [
            (514, 2, 257),
            (780, 195, 260),
            (780, 260, 390),
            (1_042, 2, 521),
        ] {
            let left = run.row(rows, contributors, left);
            let right = run.row(rows, contributors, right);
            let band = separation_band(left, right);
            let delta = left.p50 - right.p50;
            let verdict = if delta > band {
                "right is faster"
            } else if -delta > band {
                "left is faster"
            } else {
                "within noise"
            };
            println!(
                "{}x{}\t{}\t{:.4}\t{}\t{:.4}\t{:.4}\t{verdict}",
                rows,
                contributors,
                left.participants,
                left.p50,
                right.participants,
                right.p50,
                band
            );
        }
    }
}

/// Prints cross-run median variation over the exact same 52 rows.
fn print_repeatability(primary: &Run, repeat: &Run) {
    let mut relative: Vec<f64> = primary
        .rows
        .iter()
        .map(|(key, row)| {
            let repeated = repeat.rows.get(key).expect("the populations agree");
            (row.p50 - repeated.p50).abs() / row.p50
        })
        .collect();
    relative.sort_by(f64::total_cmp);
    let median = relative[relative.len() / 2];
    let maximum = *relative.last().expect("the population is non-empty");
    println!("## Repeatability");
    println!();
    println!("rows_compared\t{}", relative.len());
    println!("median_relative_p50_difference\t{median:.4}");
    println!("maximum_relative_p50_difference\t{maximum:.4}");
}

/// Twice the sum of two medians' standard errors, the retained rule.
fn separation_band(left: Row, right: Row) -> f64 {
    2.0 * (left.standard_error() + right.standard_error())
}

/// Every exact participant count the spike promises to measure.
fn admissible_participants(contributors: u64) -> Vec<u64> {
    (2..=contributors / 2)
        .filter(|participants| contributors.is_multiple_of(*participants))
        .collect()
}

/// Parses the two-column environment and digest record.
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
            "{path}: duplicate environment key {key}"
        );
    }
    values
}

/// Every non-digest value claimed by the retained record.
const CLAIMED_ENVIRONMENT: [(&str, &str); 39] = [
    ("environment.date_utc.primary", "2026-08-11T06:23:36Z"),
    ("environment.date_utc.repeat", "2026-08-11T06:24:34Z"),
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
        "946e032863a707f2cc50f44572ef78dbb3909bb3",
    ),
    (
        "host.occupancy",
        "coordinator-reserved quiet window; primary then repeat ran sequentially; no other Cargo or full gate ran during either timed submission",
    ),
    ("host.load.primary.before", "2.97 4.33 4.66"),
    ("host.load.primary.after", "3.03 4.15 4.57"),
    ("host.load.repeat.before", "2.95 4.12 4.56"),
    ("host.load.repeat.after", "2.86 3.96 4.48"),
    ("matrix.rows", "4,16384"),
    ("matrix.contributors", "514,780,1042"),
    ("matrix.shapes", "6"),
    ("matrix.variants_per_run", "52"),
    (
        "measurement.metric",
        "wall-clock commit-to-completed difference quotient",
    ),
    ("measurement.warmup", "8"),
    ("measurement.repetitions", "30"),
    ("measurement.batch", "64"),
    (
        "measurement.harness_relation",
        "timed executable predates the post-run production-label and documentation/lint-only replay-source repairs; kernel construction, source/ABI anchors, input, oracle, preparation, warm-up, timing, and numeric fields are unchanged",
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
        "c9c3e5718a3a7aa3531179d735783f01c254b4907dda7f1a345ae96e670b571d",
    ),
    ("timed.executable.size_bytes", "7807056"),
    ("timed.executable.mtime_utc", "2026-08-11T06:23:37Z"),
    (
        "timed.executable.retention",
        "digest and observed metadata only; target/release build product is not checked in",
    ),
];

/// Digest rows that bind retained subjects independently of their exact values.
const DIGEST_KEYS: [&str; 6] = [
    "hash.main",
    "hash.regions",
    "hash.buffer",
    "hash.analysis",
    "hash.sweep",
    "hash.repeat",
];

/// Pins every environment component the measurement claims.
fn validate_environment(environment: &BTreeMap<String, String>) {
    for (key, expected) in CLAIMED_ENVIRONMENT {
        assert_eq!(
            environment.get(key).map(String::as_str),
            Some(expected),
            "environment key `{key}` moved"
        );
    }

    let expected_keys: BTreeSet<&str> = CLAIMED_ENVIRONMENT
        .iter()
        .map(|(key, _)| *key)
        .chain(DIGEST_KEYS)
        .collect();
    assert_eq!(
        environment
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        expected_keys,
        "the exact retained environment key population moved"
    );
}

/// Requires one retained file to match the digest the environment record names.
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

/// Parses one unsigned TSV field.
fn number(value: &str, what: &str) -> u64 {
    value
        .parse()
        .unwrap_or_else(|error| panic!("{what} `{value}` does not parse: {error}"))
}

/// Parses one finite floating-point TSV field.
fn float(value: &str, what: &str) -> f64 {
    let parsed: f64 = value
        .parse()
        .unwrap_or_else(|error| panic!("{what} `{value}` does not parse: {error}"));
    assert!(parsed.is_finite(), "{what} `{value}` is not finite");
    parsed
}

/// Renders one participant plateau.
fn join(values: &[u64]) -> String {
    let mut joined = String::new();
    for (position, value) in values.iter().enumerate() {
        if position > 0 {
            joined.push(',');
        }
        write!(joined, "{value}").expect("writing to a String cannot fail");
    }
    joined
}

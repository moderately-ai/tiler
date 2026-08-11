//! Scores the retained partition sweep, without a device.
//!
//! The measuring half needs a Metal host and the qualified offline toolchain;
//! this half needs neither. It reads a retained TSV and derives every claim the
//! spike's README makes — the per-strategy verdict against the production
//! partition, the plateau of partitions a shape cannot tell apart, and what a
//! fixed replacement rule would cost — so those claims can be audited and
//! recomputed on any machine rather than being numbers a document asserts.
//!
//! Separating it also keeps the *selection* of a replacement honest. Picking the
//! constant that minimizes regret over the same seven shapes it is then reported
//! on is fitting to the population, so this binary additionally scores each
//! candidate leave-one-out: the constant is chosen on six shapes and paid for on
//! the seventh. A rule that survives that is evidence; a rule that only wins on
//! the full population is a description of these seven cells.
//!
//! ```sh
//! cargo run --release --bin partition-regret -- results/<date>-<host>/sweep.tsv
//! ```

use std::collections::BTreeSet;
use std::fmt::Write as _;

/// One measured row of a retained sweep.
#[derive(Clone, Copy, Debug)]
struct Row {
    rows: u64,
    contributors: u64,
    strategy: Strategy,
    partitions: u64,
    production: bool,
    /// Median per-plan cost in microseconds, submission round trip cancelled.
    p50: f64,
    /// Conservative spread of that median, in microseconds.
    stddev: f64,
    /// Timed rounds the summary was taken over.
    reps: f64,
}

impl Row {
    /// The standard error of this row's median.
    fn standard_error(self) -> f64 {
        self.stddev / self.reps.sqrt()
    }
}

/// Which parallel strategy a row measures.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Strategy {
    /// A three-stage program writing and then consuming materialized partials.
    Split,
    /// A two-stage program whose reduction workgroup holds every participant.
    Tree,
}

impl Strategy {
    /// Parses one recorded strategy key.
    fn parse(key: &str) -> Option<Self> {
        match key {
            "multi-pass-split" => Some(Self::Split),
            "single-workgroup-tree" => Some(Self::Tree),
            _ => None,
        }
    }

    /// The stable code naming this strategy.
    const fn key(self) -> &'static str {
        match self {
            Self::Split => "multi-pass-split",
            Self::Tree => "single-workgroup-tree",
        }
    }
}

/// Every measured row of one shape under one strategy, ascending by partition.
struct Cell {
    rows: u64,
    contributors: u64,
    strategy: Strategy,
    measured: Vec<Row>,
}

impl Cell {
    /// The row carrying the partition the compiler chooses.
    fn production(&self) -> Row {
        *self
            .measured
            .iter()
            .find(|row| row.production)
            .expect("every shape measures the production partition")
    }

    /// The fastest measured row.
    fn best(&self) -> Row {
        *self
            .measured
            .iter()
            .min_by(|left, right| left.p50.total_cmp(&right.p50))
            .expect("every cell measures at least one partition")
    }

    /// Every partition this cell cannot separate from its fastest.
    ///
    /// The rule is the retained crossover sweep's: a gap counts only when it
    /// exceeds two combined standard errors of the two medians. Reporting the
    /// plateau rather than the argmin alone is what keeps a one-percent ordering
    /// between two indistinguishable partitions from reading as a preference.
    fn plateau(&self) -> Vec<u64> {
        let best = self.best();
        self.measured
            .iter()
            .filter(|row| {
                row.p50 - best.p50 <= 2.0 * (row.standard_error() + best.standard_error())
            })
            .map(|row| row.partitions)
            .collect()
    }

    /// The partition a cap selects here: the largest measured one not above the
    /// cap, or the smallest measured one when the cap is below all of them.
    ///
    /// Measured rather than admissible in the abstract, deliberately. A rule
    /// cannot select a participant count the prepared entry refuses, so the
    /// domain a candidate rule is scored over is exactly the set of partitions
    /// that produced a dispatchable plan.
    fn under_cap(&self, cap: u64) -> Row {
        self.measured
            .iter()
            .filter(|row| row.partitions <= cap)
            .max_by_key(|row| row.partitions)
            .copied()
            .unwrap_or_else(|| {
                *self
                    .measured
                    .iter()
                    .min_by_key(|row| row.partitions)
                    .expect("every cell measures at least one partition")
            })
    }

    /// What following a cap costs here, as a multiple of this cell's best.
    fn regret(&self, cap: u64) -> f64 {
        self.under_cap(cap).p50 / self.best().p50
    }
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("usage: partition-regret <sweep.tsv>");
    let text = std::fs::read_to_string(&path).expect("the retained sweep reads");
    let cells = parse(&text);
    assert!(
        !cells.is_empty(),
        "the retained sweep carried no measured row"
    );

    println!("# analysis\treduction-partition-calibration");
    println!("# source\t{path}");
    println!("# cells\t{}", cells.len());
    println!();

    verdicts(&cells);
    println!();
    caps(&cells);
}

/// Reports, per shape and strategy, whether the production partition is best.
fn verdicts(cells: &[Cell]) {
    println!("## The production partition against the best measured one");
    println!();
    println!(
        "shape\tstrategy\tproduction_P\tproduction_us\tbest_P\tbest_us\tratio\tband_us\tverdict\t\
         plateau"
    );
    for cell in cells {
        let production = cell.production();
        let best = cell.best();
        let band = 2.0 * (production.standard_error() + best.standard_error());
        let separated = production.p50 - best.p50 > band;
        let plateau = cell.plateau();
        let inside = plateau.contains(&production.partitions);
        println!(
            "{}x{}\t{}\t{}\t{:.2}\t{}\t{:.2}\t{:.3}\t{:.2}\t{}\t{}",
            cell.rows,
            cell.contributors,
            cell.strategy.key(),
            production.partitions,
            production.p50,
            best.partitions,
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
        assert!(
            separated != inside,
            "{}x{} {}: the production partition is both separated from the best and inside the \
             plateau, so the two separation rules disagree",
            cell.rows,
            cell.contributors,
            cell.strategy.key(),
        );
    }
}

/// Scores every fixed-cap rule, on the full population and leave-one-out.
fn caps(cells: &[Cell]) {
    let candidates: BTreeSet<u64> = cells
        .iter()
        .flat_map(|cell| cell.measured.iter().map(|row| row.partitions))
        .collect();
    let candidates: Vec<u64> = candidates.into_iter().collect();

    for strategy in [Strategy::Split, Strategy::Tree] {
        let group: Vec<&Cell> = cells
            .iter()
            .filter(|cell| cell.strategy == strategy)
            .collect();
        println!();
        println!("## Fixed-cap rules, {}", strategy.key());
        println!();
        println!("cap\tworst_regret\tmedian_regret\tper_shape");
        let mut scored: Vec<(u64, f64, f64)> = candidates
            .iter()
            .map(|&cap| {
                let mut regrets: Vec<f64> = group.iter().map(|cell| cell.regret(cap)).collect();
                let worst = regrets.iter().copied().fold(f64::MIN, f64::max);
                regrets.sort_by(f64::total_cmp);
                (cap, worst, regrets[regrets.len() / 2])
            })
            .collect();
        scored.sort_by(|left, right| left.1.total_cmp(&right.1));
        for (cap, worst, median) in &scored {
            let per_shape: Vec<String> = group
                .iter()
                .map(|cell| format!("{:.2}", cell.regret(*cap)))
                .collect();
            println!("{cap}\t{worst:.3}\t{median:.3}\t{}", per_shape.join(" "));
        }

        // The production choice is scored the same way, so the comparison is like for
        // like: it is a rule over the same population, not a baseline measured
        // by another standard.
        let production_worst = group
            .iter()
            .map(|cell| cell.production().p50 / cell.best().p50)
            .fold(f64::MIN, f64::max);
        println!();
        println!("production_worst_regret\t{production_worst:.3}");

        println!();
        println!("held_out_shape\tcap_chosen_on_the_other_six\tregret");
        let mut held_out_worst = f64::MIN;
        for held in &group {
            let rest: Vec<&&Cell> = group
                .iter()
                .filter(|cell| cell.rows != held.rows || cell.contributors != held.contributors)
                .collect();
            // Ties broken toward the smaller cap, so the selection cannot depend
            // on the iteration order of the candidate set.
            let chosen = candidates
                .iter()
                .copied()
                .min_by(|&left, &right| {
                    let score = |cap: u64| {
                        rest.iter()
                            .map(|cell| cell.regret(cap))
                            .fold(f64::MIN, f64::max)
                    };
                    score(left).total_cmp(&score(right)).then(left.cmp(&right))
                })
                .expect("the candidate set is non-empty");
            let regret = held.regret(chosen);
            held_out_worst = held_out_worst.max(regret);
            println!("{}x{}\t{chosen}\t{regret:.3}", held.rows, held.contributors);
        }
        println!("held_out_worst_regret\t{held_out_worst:.3}");
    }
}

/// Renders a partition list compactly.
fn join(partitions: &[u64]) -> String {
    let mut rendered = String::new();
    for (index, partition) in partitions.iter().enumerate() {
        if index > 0 {
            rendered.push(',');
        }
        write!(rendered, "{partition}").expect("writing to a String cannot fail");
    }
    rendered
}

/// Parses the measured rows of a retained sweep into cells.
///
/// Declined rows are skipped rather than treated as missing data: a partition
/// the prepared entry refused has no cost, and scoring a rule that selects one
/// would be scoring a plan that does not exist.
fn parse(text: &str) -> Vec<Cell> {
    let mut rows: Vec<Row> = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.first() == Some(&"rows") {
            continue;
        }
        assert_eq!(
            fields.len(),
            24,
            "a retained row carries 24 fields and this one carries {}: {line}",
            fields.len(),
        );
        if fields[23] != "measured" {
            continue;
        }
        let Some(strategy) = Strategy::parse(fields[3]) else {
            panic!("unrecognized strategy key {}", fields[3]);
        };
        assert!(
            matches!(fields[6], "governed" | "production" | "-"),
            "unrecognized production marker {}",
            fields[6]
        );
        rows.push(Row {
            rows: fields[0].parse().expect("the row count parses"),
            contributors: fields[1].parse().expect("the contributor count parses"),
            strategy,
            partitions: fields[4].parse().expect("the partition count parses"),
            production: matches!(fields[6], "governed" | "production"),
            p50: fields[21].parse().expect("the amortized median parses"),
            stddev: fields[22].parse().expect("the amortized spread parses"),
            reps: fields[10].parse().expect("the repetition count parses"),
        });
    }

    let mut keys: Vec<(u64, u64, Strategy)> = rows
        .iter()
        .map(|row| (row.rows, row.contributors, row.strategy))
        .collect();
    keys.dedup();
    let mut seen: BTreeSet<(u64, u64, Strategy)> = BTreeSet::new();
    let mut cells = Vec::new();
    for key in keys {
        if !seen.insert(key) {
            continue;
        }
        let mut measured: Vec<Row> = rows
            .iter()
            .filter(|row| (row.rows, row.contributors, row.strategy) == key)
            .copied()
            .collect();
        measured.sort_by_key(|row| row.partitions);
        cells.push(Cell {
            rows: key.0,
            contributors: key.1,
            strategy: key.2,
            measured,
        });
    }
    cells
}

//! Fits the analytical cost model to a retained sweep and scores it on held-out
//! rows.
//!
//! Deliberately a second binary that touches no device. The measurement is the
//! expensive, environment-bound half; the fit is arithmetic over a retained TSV,
//! and separating them means the calibration can be audited, rerun, and
//! perturbed by anyone with the file — including on a machine that has no Metal
//! device, and without re-dispatching anything.
//!
//! # What is fitted, and against what objective
//!
//! [`model::CostParameters`], three numbers, each a property of the machine and
//! none of a strategy: the fixed cost of one dispatch, the fold steps the device
//! retires at once, and what one step costs on the critical path.
//!
//! **The objective is measured decision regret, not predicted latency.** For
//! each fitted cell the model names a strategy, and the penalty is the measured
//! time of that strategy divided by the measured time of the fastest one; the
//! fit minimizes the mean squared log of those ratios. Magnitude error breaks
//! ties and never leads, because it is the wrong target: a model uniformly twice
//! too slow selects perfectly, and one within 5% everywhere can still invert two
//! near-ties. Selection consumes an ordering, so the calibration is scored on
//! the ordering.
//!
//! This is not the same thing as choosing a constant until a preferred plan
//! wins. The objective ranges over cells whose measured winners are not all the
//! same strategy — it includes shapes the serial fold wins by seven times and
//! shapes it loses by forty-eight — so no parameter can favour a strategy
//! without paying for it wherever that strategy loses.
//!
//! # The held-out split, and why it is this one
//!
//! The fit set is every cell whose contributor count is a **perfect square**;
//! the held-out set is the rest. On a square count `governed_partition`'s
//! partition count and its contributors-per-partition are equal, so a model
//! fitted only to squares is indistinguishable from one that memorized the
//! square root of the reduction length. The held-out counts — 32, 128, 2,048
//! and 8,192 — split into `(8, 4)`, `(16, 8)`, `(64, 32)` and `(128, 64)`, so
//! predicting them requires the model to have the *decomposition* right rather
//! than a curve through the diagonal. A random split would not have tested that.
//!
//! # Which cells carry weight, stated rather than averaged away
//!
//! A cell's verdict is **separated** when its winner beats the runner-up by more
//! than two combined standard errors of their medians. Unseparated cells are
//! scored like any other and are also scored on their own, because a model that
//! inverts a 0.2% difference inside the noise has not made an error a consumer
//! can measure, while one that inverts a separated factor of two has.
//!
//! **The fit is taken over the cells whose serial-or-parallel verdict is
//! separated.** Fitting to a cell whose measured ordering is noise means fitting
//! to noise, and this matrix has such cells by construction: at a few hundred
//! elements the whole plan costs less than one dispatch, so the three strategies
//! are indistinguishable and their recorded order is a coin toss. That
//! particular separation is the criterion rather than the three-way one because
//! it is the decision the model is for, and because the three-way test excludes
//! cells whose parallel-or-not verdict is perfectly resolved and whose two
//! parallel strategies happen to tie. Every cell is still *scored*, separated or
//! not, so the cost of the exclusion is visible rather than hidden.
//!
//! # Running it
//!
//! ```sh
//! cargo run --release --bin reduction-cost-fit -- results/<date>-<host>/sweep.tsv
//! ```
//!
//! `--perturb <encoder|parallel|step> <factor>` refits nothing and instead
//! scales one fitted parameter by `factor`, reporting how the predicted winners
//! move. That is the mutation proof of the calibration: a parameter the
//! selection evidence does not actually depend on would leave every predicted
//! winner unchanged.

// A bin target is its own crate root, so this module is compiled twice, once per
// binary, and each sees only what its own binary calls. The sweep drives the
// stage model and the classification; this binary drives the parameters and the
// prediction. Neither set is dead — they are the two halves of one shared model
// — so the unused half is allowed here rather than split into two files that
// could drift apart.
#[allow(
    dead_code,
    reason = "the sweep binary consumes the stage model and the strategy classifier; this binary consumes the parameters and the prediction. One shared module, two crate roots."
)]
mod model;

use std::collections::BTreeMap;
use std::process::ExitCode;

use model::{CostParameters, Stage, Strategy};

/// One measured `(cell, strategy)` row of a retained sweep.
#[derive(Clone, Debug)]
struct Row {
    strategy: Strategy,
    stages: Vec<Stage>,
    /// Per-plan cost derived from the minimum sample at each encode count.
    min_us: f64,
    /// Per-plan cost derived from the median sample at each encode count.
    p50_us: f64,
    /// The standard error of `p50_us`.
    ///
    /// Derived from the sweep's recorded spread and its repetition count as
    /// `stddev / sqrt(reps)`. **The spread itself is the wrong scale for asking
    /// whether two strategies differ**: it describes how much one submission
    /// varies from the next, which is dominated by the round trip's own jitter,
    /// while the question is how precisely the *median of thirty* is known. The
    /// recorded spread is already conservative — the amortization adds the two
    /// encode counts' spreads rather than combining them in quadrature — so this
    /// stays an over-estimate of the uncertainty after the division.
    standard_error_us: f64,
}

/// One shape's measured strategies, with the model's verdict on them.
#[derive(Clone, Copy)]
struct Cell {
    rows: u64,
    contributors: u64,
    measured: Strategy,
    predicted: Strategy,
    /// The winner the *minimum* sample names, which need not be the median's.
    ///
    /// Reported rather than reconciled. The two statistics answer different
    /// questions — the least contaminated single observation, and the cost a
    /// caller typically pays — and a cell where they disagree is one whose
    /// verdict rests on noise.
    min_winner: Strategy,
    measured_us: f64,
    /// Ratio of the runner-up's measured cost to the winner's.
    margin: f64,
    /// Ratio of the predicted winner's measured cost to the measured winner's.
    regret: f64,
    /// Ratio of the serial fold's measured cost to the best parallel strategy's.
    ///
    /// Above one the parallel strategies are worth having, below one the serial
    /// fold is, and the crossover is this quantity's unit contour. It is printed
    /// per cell rather than summarized because its range — two orders of
    /// magnitude across this matrix — is the finding.
    parallel_speedup: f64,
    /// Whether the winner beats the runner-up by more than the noise allows.
    ///
    /// Two combined standard errors, which is the band inside which the ordering
    /// of two medians is not evidence. An unseparated cell is still measured and
    /// still reported; what it is not is a fact a selection decision may be
    /// scored against.
    separated: bool,
    /// The same test applied to the serial fold against the best parallel plan.
    ///
    /// **A different question from [`Cell::separated`], and the one selection
    /// turns on.** Over most of this matrix the two parallel strategies sit
    /// inside each other's noise while the serial fold is far outside it, so a
    /// cell can have an unresolvable *winner* and a completely resolved
    /// *parallel-or-not* verdict. Scoring only the first would report a model
    /// that parallelizes on the wrong side of the contour as untestable.
    parallel_separated: bool,
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(path) = arguments.first() else {
        eprintln!(
            "usage: reduction-cost-fit <sweep.tsv> [--perturb <encoder|parallel|step> <factor>]"
        );
        return ExitCode::FAILURE;
    };
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("cannot read {path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let measured = parse(&text);
    if measured.is_empty() {
        eprintln!("{path} carried no measured rows");
        return ExitCode::FAILURE;
    }

    let fit_shapes: Vec<Shape<'_>> = measured
        .iter()
        .filter(|((_, contributors), _)| is_fit_column(*contributors))
        .collect();
    let held_shapes: Vec<Shape<'_>> = measured
        .iter()
        .filter(|((_, contributors), _)| !is_fit_column(*contributors))
        .collect();
    let fitted = fit(&fit_shapes);
    println!(
        "# fitted on {} of {} fit-set cells, the ones whose measured serial-or-parallel verdict \
         is separated from the noise",
        fit_shapes
            .iter()
            .filter(|(_, members)| is_parallel_separated(members))
            .count(),
        fit_shapes.len(),
    );

    let parameters = match perturbation(&arguments) {
        Ok(None) => fitted,
        Ok(Some((name, factor))) => {
            println!("# PERTURBED: {name} scaled by {factor}");
            perturb(fitted, name, factor)
        }
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    println!("## Fitted parameters");
    println!(
        "encoder_seconds\t{:.9e}\t{:.4} us per dispatch",
        parameters.encoder_seconds,
        parameters.encoder_seconds * 1e6,
    );
    println!(
        "parallel_threads\t{:.6e}\tfold steps retired at once when saturated",
        parameters.parallel_threads,
    );
    println!(
        "step_seconds\t{:.9e}\t{:.4} ns per critical-path fold step",
        parameters.step_seconds,
        parameters.step_seconds * 1e9,
    );
    println!();

    let fit_cells = cells(&fit_shapes, parameters);
    let held_cells = cells(&held_shapes, parameters);

    println!("## Decision accuracy: the three-way choice");
    println!("set\tcells\texact\tmedian_regret\tp90_regret\tworst_regret");
    report_regret("fit", &fit_cells);
    report_regret("held-out", &held_cells);
    println!();

    println!("## The same, on the cells whose winner is separated from its runner-up");
    println!("set\tcells\texact\tmedian_regret\tp90_regret\tworst_regret");
    report_regret("fit", &separated(&fit_cells));
    report_regret("held-out", &separated(&held_cells));
    println!();

    println!("## Decision accuracy: the serial fold against a parallel strategy");
    println!("set\tcells\tagreed\tworst_regret_when_wrong");
    report_binary("fit", &fit_cells);
    report_binary("held-out", &held_cells);
    println!();

    println!("## The same, on the cells whose serial-or-parallel verdict is separated");
    println!("set\tcells\tagreed\tworst_regret_when_wrong");
    report_binary("fit", &parallel_separated(&fit_cells));
    report_binary("held-out", &parallel_separated(&held_cells));
    println!();

    println!("## Magnitude accuracy, |predicted/measured - 1| against the median sample");
    println!("set\trows\tmedian\tp90\tmax");
    report_magnitude("fit", &fit_shapes, parameters);
    report_magnitude("held-out", &held_shapes, parameters);
    println!();

    println!("## Every cell");
    println!(
        "set\trows\tcontributors\tmeasured\tpredicted\tmeasured_us\tmargin\tregret\t\
         parallel_speedup\tseparated\tparallel_separated\tmin_sample_winner"
    );
    print_cells("fit", &fit_cells);
    print_cells("held-out", &held_cells);

    ExitCode::SUCCESS
}

/// One shape and the strategies measured on it.
type Shape<'measured> = (&'measured (u64, u64), &'measured Vec<Row>);

/// Whether a contributor count belongs to the fit set rather than the held-out one.
fn is_fit_column(contributors: u64) -> bool {
    let root = contributors.isqrt();
    root * root == contributors
}

/// Which parameter a `--perturb` invocation names, and by how much.
fn perturbation(arguments: &[String]) -> Result<Option<(&str, f64)>, String> {
    let Some(position) = arguments.iter().position(|value| value == "--perturb") else {
        return Ok(None);
    };
    let name = arguments
        .get(position + 1)
        .ok_or_else(|| "--perturb needs a parameter name".to_owned())?;
    let factor = arguments
        .get(position + 2)
        .ok_or_else(|| "--perturb needs a factor".to_owned())?
        .parse::<f64>()
        .map_err(|error| format!("the perturbation factor does not parse: {error}"))?;
    match name.as_str() {
        "encoder" | "parallel" | "step" => Ok(Some((name.as_str(), factor))),
        other => Err(format!(
            "unknown parameter \"{other}\"; expected encoder, parallel, or step"
        )),
    }
}

/// Scales one fitted parameter, leaving the other two where the fit put them.
fn perturb(parameters: CostParameters, name: &str, factor: f64) -> CostParameters {
    let mut perturbed = parameters;
    match name {
        "encoder" => perturbed.encoder_seconds *= factor,
        "parallel" => perturbed.parallel_threads *= factor,
        "step" => perturbed.step_seconds *= factor,
        _ => unreachable!("the parameter name was validated"),
    }
    perturbed
}

/// Parses a retained sweep, skipping its `#` provenance lines and its header.
///
/// Keyed by shape, so every consumer below sees a cell's strategies together.
/// The **amortized** columns are read rather than the raw submission ones: a
/// submission's fixed round trip is identical for all three strategies, so
/// including it cannot produce a crossover and can only bury one.
fn parse(text: &str) -> BTreeMap<(u64, u64), Vec<Row>> {
    let mut measured: BTreeMap<(u64, u64), Vec<Row>> = BTreeMap::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') || line.starts_with("rows\t") {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let parsed = (|| {
            Some((
                (
                    fields.first()?.parse::<u64>().ok()?,
                    fields.get(1)?.parse::<u64>().ok()?,
                ),
                Row {
                    strategy: Strategy::parse(fields.get(5)?)?,
                    stages: parse_stages(fields.get(9)?)?,
                    min_us: fields.get(20)?.parse().ok()?,
                    p50_us: fields.get(21)?.parse().ok()?,
                    standard_error_us: fields.get(22)?.parse::<f64>().ok()?
                        / fields.get(10)?.parse::<f64>().ok()?.sqrt(),
                },
            ))
        })();
        match parsed {
            Some((shape, row)) => measured.entry(shape).or_default().push(row),
            None => panic!("this line is not a sweep row: {line}"),
        }
    }
    measured
}

/// Parses the recorded `threads:work:depth` stage column.
fn parse_stages(field: &str) -> Option<Vec<Stage>> {
    field
        .split('|')
        .map(|stage| {
            let mut parts = stage.split(':');
            let parsed = Stage {
                threads: parts.next()?.parse().ok()?,
                work: parts.next()?.parse().ok()?,
                depth: parts.next()?.parse().ok()?,
            };
            parts.next().is_none().then_some(parsed)
        })
        .collect()
}

/// Fits the three parameters by successively refined search in log space.
///
/// Deterministic and dependency-free: a coarse logarithmic grid over each
/// parameter, then refinement passes each narrowing the bracket around the
/// running best. The objective is decision regret with magnitude error as a tie
/// break, weighted so magnitude can never outvote a decision.
///
/// **The tie break is load-bearing rather than cosmetic.** Regret alone is
/// invariant under scaling the encoder and step parameters together — every
/// prediction moves by one factor and no ordering changes — so it fixes only two
/// of the three numbers and would leave the third wherever the grid happened to
/// stop. Magnitude error is what pins the scale, which is what makes the
/// reported parameters quantities of the machine rather than an arbitrary point
/// on a ray.
fn fit(shapes: &[Shape<'_>]) -> CostParameters {
    /// Logarithmic samples per parameter per refinement pass.
    const STEPS: usize = 28;

    let mut brackets = [
        (1e-9_f64, 1e-2_f64),  // encoder seconds
        (1e1_f64, 1e9_f64),    // fold steps retired at once
        (1e-13_f64, 1e-5_f64), // step seconds
    ];
    let mut best = CostParameters {
        encoder_seconds: 1e-6,
        parallel_threads: 1e3,
        step_seconds: 1e-8,
    };
    let mut best_error = f64::INFINITY;

    for _ in 0..10 {
        for encoder_index in 0..=STEPS {
            let encoder_seconds = log_sample(brackets[0], encoder_index, STEPS);
            for parallel_index in 0..=STEPS {
                let parallel_threads = log_sample(brackets[1], parallel_index, STEPS);
                for step_index in 0..=STEPS {
                    let candidate = CostParameters {
                        encoder_seconds,
                        parallel_threads,
                        step_seconds: log_sample(brackets[2], step_index, STEPS),
                    };
                    let error = objective(shapes, candidate);
                    if error < best_error {
                        best_error = error;
                        best = candidate;
                    }
                }
            }
        }
        brackets = [
            narrow(brackets[0], best.encoder_seconds),
            narrow(brackets[1], best.parallel_threads),
            narrow(brackets[2], best.step_seconds),
        ];
    }
    best
}

/// Mean squared log regret, with mean squared log magnitude error as a tie break.
fn objective(shapes: &[Shape<'_>], parameters: CostParameters) -> f64 {
    let mut regret = 0.0_f64;
    let mut magnitude = 0.0_f64;
    let mut separated_shapes = 0_usize;
    let mut rows = 0_usize;
    for shape in shapes {
        let (_, members) = shape;
        if is_parallel_separated(members) {
            let best = members
                .iter()
                .map(|row| row.p50_us)
                .fold(f64::INFINITY, f64::min);
            let chosen = cheapest(members, parameters);
            regret += (chosen.p50_us / best).ln().powi(2);
            separated_shapes += 1;
        }
        for row in *members {
            magnitude += (parameters.predict(&row.stages) * 1e6 / row.p50_us)
                .ln()
                .powi(2);
            rows += 1;
        }
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "the counts here are in the hundreds"
    )]
    let (shape_count, row_count) = (separated_shapes.max(1) as f64, rows.max(1) as f64);
    regret / shape_count * 1e3 + magnitude / row_count
}

/// Whether one shape's measured winner is separated from its runner-up.
///
/// Computed from the measurement alone, so it does not depend on the parameters
/// being fitted and cannot make the fit circular.
fn is_separated(members: &[Row]) -> bool {
    let mut ranked: Vec<&Row> = members.iter().collect();
    ranked.sort_by(|left, right| left.p50_us.total_cmp(&right.p50_us));
    ranked[1].p50_us - ranked[0].p50_us
        > 2.0 * (ranked[0].standard_error_us + ranked[1].standard_error_us)
}

/// Whether the serial fold is separated from the best parallel strategy.
fn is_parallel_separated(members: &[Row]) -> bool {
    let Some(serial) = members
        .iter()
        .find(|row| row.strategy == Strategy::SerialFold)
    else {
        return false;
    };
    let Some(parallel) = members
        .iter()
        .filter(|row| row.strategy != Strategy::SerialFold)
        .min_by(|left, right| left.p50_us.total_cmp(&right.p50_us))
    else {
        return false;
    };
    (serial.p50_us - parallel.p50_us).abs()
        > 2.0 * (serial.standard_error_us + parallel.standard_error_us)
}

/// The strategy this model would choose from one shape's measured strategies.
fn cheapest(members: &[Row], parameters: CostParameters) -> &Row {
    members
        .iter()
        .min_by(|left, right| {
            parameters
                .predict(&left.stages)
                .total_cmp(&parameters.predict(&right.stages))
        })
        .expect("every shape carries at least one strategy")
}

/// The `index`-th of `steps + 1` logarithmically spaced points in a bracket.
fn log_sample(bracket: (f64, f64), index: usize, steps: usize) -> f64 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "both indices are below 32 and f64 represents every integer below 2^53 exactly"
    )]
    let fraction = index as f64 / steps as f64;
    (bracket.0.ln() + fraction * (bracket.1.ln() - bracket.0.ln())).exp()
}

/// Narrows a bracket to a quarter of its log width around `centre`.
fn narrow(bracket: (f64, f64), centre: f64) -> (f64, f64) {
    let width = (bracket.1.ln() - bracket.0.ln()) / 8.0;
    ((centre.ln() - width).exp(), (centre.ln() + width).exp())
}

/// Resolves each shape into its measured and predicted verdicts.
fn cells(shapes: &[Shape<'_>], parameters: CostParameters) -> Vec<Cell> {
    shapes
        .iter()
        .map(|&(&(rows, contributors), members)| {
            let mut ranked: Vec<&Row> = members.iter().collect();
            ranked.sort_by(|left, right| left.p50_us.total_cmp(&right.p50_us));
            let winner = ranked[0];
            let runner_up = ranked[1];
            let predicted = cheapest(members, parameters);
            let serial = members
                .iter()
                .find(|row| row.strategy == Strategy::SerialFold)
                .expect("every shape retains the serial fold");
            let parallel = members
                .iter()
                .filter(|row| row.strategy != Strategy::SerialFold)
                .map(|row| row.p50_us)
                .fold(f64::INFINITY, f64::min);
            Cell {
                rows,
                contributors,
                measured: winner.strategy,
                predicted: predicted.strategy,
                min_winner: members
                    .iter()
                    .min_by(|left, right| left.min_us.total_cmp(&right.min_us))
                    .expect("every shape carries at least one strategy")
                    .strategy,
                measured_us: winner.p50_us,
                margin: runner_up.p50_us / winner.p50_us,
                regret: predicted.p50_us / winner.p50_us,
                parallel_speedup: serial.p50_us / parallel,
                separated: is_separated(members),
                parallel_separated: is_parallel_separated(members),
            }
        })
        .collect()
}

/// The subset whose serial-or-parallel verdict survives the noise.
fn parallel_separated(cells: &[Cell]) -> Vec<Cell> {
    cells
        .iter()
        .copied()
        .filter(|cell| cell.parallel_separated)
        .collect()
}

/// The subset whose measured winner survives the noise.
fn separated(cells: &[Cell]) -> Vec<Cell> {
    cells
        .iter()
        .copied()
        .filter(|cell| cell.separated)
        .collect()
}

/// Reports the distribution of decision regret over one cell set.
fn report_regret(label: &str, cells: &[Cell]) {
    if cells.is_empty() {
        println!("{label}\t0\t-\t-\t-\t-");
        return;
    }
    let exact = cells
        .iter()
        .filter(|cell| cell.measured == cell.predicted)
        .count();
    let mut regrets: Vec<f64> = cells.iter().map(|cell| cell.regret).collect();
    regrets.sort_by(f64::total_cmp);
    println!(
        "{label}\t{}\t{exact}\t{:.4}\t{:.4}\t{:.4}",
        cells.len(),
        quantile(&regrets, 0.50),
        quantile(&regrets, 0.90),
        regrets[regrets.len() - 1],
    );
}

/// Reports how often the model agrees on the serial-or-parallel question alone.
///
/// The three-way report is the strict one; this is the decision-shaped one.
/// Selection's consequential choice on this program family is whether to
/// parallelize at all: the two parallel strategies sit inside each other's noise
/// over most of this matrix, while the serial fold is separated from both by up
/// to two orders of magnitude. Picking the wrong parallel strategy costs a few
/// percent; parallelizing on the wrong side of the contour costs a factor.
fn report_binary(label: &str, cells: &[Cell]) {
    if cells.is_empty() {
        println!("{label}\t0\t-\t-");
        return;
    }
    let mut disagreements = Vec::new();
    let mut agreed = 0_usize;
    for cell in cells {
        let measured_serial = cell.measured == Strategy::SerialFold;
        let predicted_serial = cell.predicted == Strategy::SerialFold;
        if measured_serial == predicted_serial {
            agreed += 1;
        } else {
            disagreements.push(cell.regret);
        }
    }
    disagreements.sort_by(f64::total_cmp);
    println!(
        "{label}\t{}\t{agreed}\t{:.4}",
        cells.len(),
        disagreements.last().copied().unwrap_or(1.0),
    );
}

/// Reports the distribution of relative magnitude error over one row set.
fn report_magnitude(label: &str, shapes: &[Shape<'_>], parameters: CostParameters) {
    let mut errors: Vec<f64> = shapes
        .iter()
        .flat_map(|(_, members)| members.iter())
        .map(|row| (parameters.predict(&row.stages) * 1e6 / row.p50_us - 1.0).abs())
        .collect();
    if errors.is_empty() {
        println!("{label}\t0\t-\t-\t-");
        return;
    }
    errors.sort_by(f64::total_cmp);
    println!(
        "{label}\t{}\t{:.4}\t{:.4}\t{:.4}",
        errors.len(),
        quantile(&errors, 0.50),
        quantile(&errors, 0.90),
        errors[errors.len() - 1],
    );
}

/// Prints one set's per-cell verdicts.
fn print_cells(label: &str, cells: &[Cell]) {
    for cell in cells {
        println!(
            "{label}\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{}\t{}\t{}",
            cell.rows,
            cell.contributors,
            cell.measured.key(),
            cell.predicted.key(),
            cell.measured_us,
            cell.margin,
            cell.regret,
            cell.parallel_speedup,
            if cell.separated { "yes" } else { "no" },
            if cell.parallel_separated { "yes" } else { "no" },
            cell.min_winner.key(),
        );
    }
}

/// The nearest-rank quantile of an ascending sample set.
fn quantile(sorted: &[f64], fraction: f64) -> f64 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "the sample count is in the hundreds"
    )]
    let count = sorted.len() as f64;
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the product of a fraction in [0, 1] and a small positive count is a small non-negative value"
    )]
    let rank = ((fraction * count).ceil() as usize).max(1);
    sorted[rank.min(sorted.len()) - 1]
}

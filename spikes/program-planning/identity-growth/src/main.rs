//! Measures kernel-program identity growth against `MAX_PROGRAM_IDENTITY_BYTES`.
//!
//! `measure-executable-coverage-identity-growth-against-the-program-identity-bound`
//! owns a structural inference that had exactly one measured point behind it:
//! `CanonicalKernelProgramIdentity` embeds one whole reached-only
//! executable-coverage identity per covered occurrence, one record per graph
//! operation, and each of those records embeds the complete
//! `SemanticGraphIdentity` of the bound graph — so program identity should be
//! quadratic in graph size against a hard 64 MiB bound that fails closed.
//!
//! This sweep replaces the inference's single point with a curve. It compiles
//! programs of increasing operation count through the **ordinary** path — the
//! public `tiler_compiler::session::compile` boundary, whose lowering mints
//! real index-refinement receipts, derives `CoveredOccurrence` records from
//! them, and drives `KernelProgramBuilder` — and reads the identity byte length
//! off the verified program each compilation produced. Nothing here constructs
//! an identity, a receipt, or a coverage record itself; a synthetic one would
//! measure this file rather than the compiler.
//!
//! # It refuses rather than measuring garbage
//!
//! Every row rests on a program that actually verified. A compilation that
//! refuses, a target slot that refuses, a portfolio with no selected
//! alternative, or a coverage record set that does not cover every semantic
//! operation each end the run with a diagnosis on stderr and a non-zero exit,
//! because a measurement harness that degrades to a partial row publishes a
//! number nobody can tell apart from a real one.
//!
//! Three `--perturb` modes exist to watch those refusals fire rather than
//! trust them; see [`Perturbation`]. Each exits non-zero.
//!
//! # Reading the output
//!
//! One TSV row per ladder point on stdout, then a summary block of `#` comment
//! lines carrying the structural decomposition, the exact quadratic fit, and
//! the extrapolated refusal point solved from it. The run ends by compiling one
//! program past the governed budget and requiring it to refuse, so the ladder's
//! claim to be the whole reachable domain is measured rather than asserted.
//!
//! Run it from this directory:
//!
//! ```sh
//! cargo run --release > results/<date>-<host>/growth.tsv
//! ```

use std::process::ExitCode;
use std::time::Instant;

use tiler_build::BoundMetalCompileDeclaration;
use tiler_compiler::session::{CompileRequest, NumericalContract, compile};
use tiler_compiler::target::TargetRequest;
use tiler_ir::program::MAX_PROGRAM_IDENTITY_BYTES;
use tiler_ir::semantic::{
    F32, F32Constant, F32Multiply, F32Reindex, InputKey, OutputKey, ReindexForm, SemanticProgram,
    SemanticProgramBuilder,
};
use tiler_ir::shape::{Axis, Shape};

/// Operation counts swept, which is the whole reachable domain.
///
/// **This ladder is not a sample; it is every program size the ordinary
/// compilation path admits.** `DeterministicBudgets::governed` caps
/// `semantic_operations` at 8, that budget is `pub(crate)`, and
/// `CompileRequest` binds `InstalledCapabilities::governed`, so no public
/// caller can state a wider one. A nine-operation program is refused before any
/// kernel program exists — which [`probe_the_wall`] demonstrates rather than
/// assumes, because a budget read from a constant and a budget that actually
/// refuses are different facts.
///
/// The generator emits one shared constant and a chain of multiplies, so the
/// operation count is `1 + multiplies` and every integer in the domain is
/// reachable. Seven points over 2..=8 is a denser ladder than a doubling one
/// could have been inside the same wall.
const OPERATIONS: &[usize] = &[2, 3, 4, 5, 6, 7, 8];

/// The first operation count the governed budget refuses.
const BEYOND_THE_WALL: usize = 9;

/// The tensor extent every program in the sweep is built over.
///
/// Held fixed and small deliberately. Extent enters the graph identity as a
/// handful of bytes per value and enters nothing else this sweep measures, so
/// varying it would add a second axis that moves the curve's constant without
/// touching its exponent — while a large extent costs launch geometry the
/// target profile has to admit.
const EXTENT: u64 = 4;

/// The contract every compilation in the sweep states.
///
/// What the measured Apple row delivers, and pinned so that one program family
/// compiles at every ladder point. The generator emits a pure multiply chain
/// rather than a mixed multiply/add body for the same reason: a region holding
/// a multiply adjacent to an add is refused under the one contract that permits
/// arithmetic contraction, and a generator whose admissibility depended on the
/// contract would put a second variable in a one-variable sweep.
const CONTRACT: NumericalContract = NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32;

/// A deliberate corruption used to watch one of the harness's refusals fire.
///
/// AGENTS.md requires a check to be run against a case that must fail before
/// its passing verdict means anything. Both arms below end the run non-zero,
/// and each exercises a different refusal: the compile path and the coverage
/// completeness assertion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Perturbation {
    /// None; the ordinary sweep.
    None,
    /// Emit a program the compiler cannot lower, and watch the run refuse.
    ///
    /// `tiler::reindex-f32@1` is refused for recognition rather than for
    /// numerics: its access relation is one `LogicalAccess` cannot spell, so no
    /// projection exists to make and every contract refuses it. That makes it a
    /// program which genuinely does not reach a verified kernel program, which
    /// is the failure the sweep must not paper over.
    Program,
    /// Corrupt the expected coverage count, and watch the completeness
    /// assertion refuse.
    ///
    /// The assertion compares the coverage records the compiler emitted against
    /// the semantic operation count. Moving the expected value by one is what
    /// proves the comparison can say no — without it, "every operation is
    /// covered" would be indistinguishable from a comparison that never ran.
    Coverage,
    /// Corrupt one measured byte count, and watch the exact-fit check refuse.
    ///
    /// The summary's whole claim is that the observed curve *is*
    /// `a*n^2 + b*n + c` rather than merely resembling one, and the refusal
    /// point is solved from those coefficients. A residual check that cannot
    /// fail would turn that into an assertion; moving one row by a single byte
    /// is what proves it can.
    Fit,
}

/// One measured ladder point.
struct Row {
    /// Operation count the generator was asked for.
    requested: usize,
    /// Output-reachable semantic operations in the compiled graph.
    operations: usize,
    /// Coverage records the selected alternative carries, summed over stages.
    coverage_records: usize,
    /// Stages the selected alternative dispatches.
    stages: usize,
    /// Retained plan alternatives in the portfolio.
    alternatives: usize,
    /// Canonical semantic graph identity length.
    graph_bytes: usize,
    /// Canonical kernel-program identity length of the selected alternative.
    program_bytes: usize,
    /// Largest program identity over every retained alternative.
    widest_alternative_bytes: usize,
    /// Summed reached-only executable-coverage identity length over all records.
    coverage_bytes: usize,
    /// Compile wall time, the faster of the two runs behind this row.
    compile_ms: u128,
}

fn main() -> ExitCode {
    let perturbation = match parse_perturbation() {
        Ok(perturbation) => perturbation,
        Err(argument) => {
            eprintln!("unknown argument {argument:?}; expected --perturb=program|coverage|fit");
            return ExitCode::FAILURE;
        }
    };

    let declaration = match BoundMetalCompileDeclaration::first_macos_apple9() {
        Ok(declaration) => declaration,
        Err(error) => {
            eprintln!("the authoritative macOS Apple9 declaration does not bind: {error:?}");
            return ExitCode::FAILURE;
        }
    };

    println!(
        "requested\toperations\tcoverage_records\tstages\talternatives\tgraph_bytes\t\
         program_bytes\twidest_alternative_bytes\tcoverage_bytes\tmean_record_bytes\t\
         bytes_per_op_squared\tcompile_ms"
    );

    let mut rows: Vec<Row> = Vec::new();
    for operations in OPERATIONS {
        match measure(*operations, &declaration, perturbation) {
            Ok(mut row) => {
                if perturbation == Perturbation::Fit && *operations == OPERATIONS[1] {
                    row.program_bytes += 1;
                }
                print_row(&row);
                rows.push(row);
            }
            Err(refusal) => {
                eprintln!("REFUSED at operations={operations}: {refusal}");
                eprintln!(
                    "no row is printed for a refused point, and the sweep stops rather than \
                     continuing past a program that did not verify."
                );
                return ExitCode::FAILURE;
            }
        }
    }

    if !summarize(&rows) {
        return ExitCode::FAILURE;
    }
    if !probe_the_wall(&declaration) {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// Compiles one operation past the governed budget, and requires a refusal.
///
/// The ladder above claims to be the *whole* reachable domain, and that claim
/// is only worth something if the first point outside it actually refuses. A
/// success here does not mean the sweep is wrong; it means the budget moved and
/// the domain is now wider than this ladder, which is a finding rather than a
/// pass — so it ends the run non-zero and says what to do.
fn probe_the_wall(declaration: &BoundMetalCompileDeclaration) -> bool {
    println!("#");
    let program = chain_program(BEYOND_THE_WALL);
    match compile_once(&program, declaration) {
        Err(refusal) => {
            println!(
                "# WALL CONFIRMED at {BEYOND_THE_WALL} operations: {}",
                refusal.replace(['\n', '\t'], " ")
            );
            println!(
                "# so the ladder above is the entire domain the ordinary compilation path admits, \
                 measured rather than read off a constant."
            );
            true
        }
        Ok((compiled, _)) => {
            eprintln!(
                "THE WALL MOVED: {BEYOND_THE_WALL} operations compiled to a {}-byte identity, so \
                 the governed semantic-operations budget is no longer 8 and this ladder is no \
                 longer the whole reachable domain. Widen OPERATIONS and rerun; the recorded \
                 result and its verdict are stale.",
                compiled.identity.len()
            );
            false
        }
    }
}

/// Reads the one optional argument, rejecting anything else.
fn parse_perturbation() -> Result<Perturbation, String> {
    let mut perturbation = Perturbation::None;
    for argument in std::env::args().skip(1) {
        perturbation = match argument.as_str() {
            "--perturb=program" => Perturbation::Program,
            "--perturb=coverage" => Perturbation::Coverage,
            "--perturb=fit" => Perturbation::Fit,
            _ => return Err(argument),
        };
    }
    Ok(perturbation)
}

/// Compiles one ladder point twice and returns its measured row.
///
/// Twice rather than once for two reasons that are one procedure. The byte
/// counts are deterministic by construction, so a second compilation whose
/// identity bytes differ would mean the encoding is not the function of program
/// content this measurement assumes — the run refuses in that case rather than
/// reporting the first answer. The pair also supplies the two timing samples
/// the reported wall time is the minimum of; that number is an indication of
/// reachability, never a performance claim.
fn measure(
    requested: usize,
    declaration: &BoundMetalCompileDeclaration,
    perturbation: Perturbation,
) -> Result<Row, String> {
    let program = if perturbation == Perturbation::Program {
        unrecognized_program()
    } else {
        chain_program(requested)
    };
    let operations = program.operation_count();

    let (first, first_ms) = compile_once(&program, declaration)?;
    let (second, second_ms) = compile_once(&program, declaration)?;
    if first.identity != second.identity {
        return Err(format!(
            "two compilations of one program produced different identity bytes ({} then {}); the \
             encoding is not a function of program content and no byte count here means anything",
            first.identity.len(),
            second.identity.len()
        ));
    }

    let expected_coverage = match perturbation {
        Perturbation::Coverage => operations + 1,
        _ => operations,
    };
    if first.coverage_records != expected_coverage {
        return Err(format!(
            "the selected alternative covers {} semantic occurrences but the graph has {} \
             operations; a coverage set that is not the whole graph is not the subject this \
             measurement is about",
            first.coverage_records, expected_coverage
        ));
    }

    Ok(Row {
        requested,
        operations,
        coverage_records: first.coverage_records,
        stages: first.stages,
        alternatives: first.alternatives,
        graph_bytes: first.graph_bytes,
        program_bytes: first.identity.len(),
        widest_alternative_bytes: first.widest_alternative_bytes,
        coverage_bytes: first.coverage_bytes,
        compile_ms: first_ms.min(second_ms),
    })
}

/// Everything one compilation contributes to a row.
struct Compiled {
    identity: Vec<u8>,
    graph_bytes: usize,
    coverage_records: usize,
    coverage_bytes: usize,
    stages: usize,
    alternatives: usize,
    widest_alternative_bytes: usize,
}

/// Drives the public compile boundary once and reads the verified program.
fn compile_once(
    program: &SemanticProgram,
    declaration: &BoundMetalCompileDeclaration,
) -> Result<(Compiled, u128), String> {
    let targets = TargetRequest::new([declaration.profile().clone()])
        .map_err(|error| format!("the singleton target request does not build: {error:?}"))?;
    let request = CompileRequest::new(program, CONTRACT, targets);

    let started = Instant::now();
    let batch = compile(request).map_err(|failure| {
        format!(
            "the compilation batch refused: {failure:?}{}",
            failure
                .explain()
                .map(|report| format!(" | {}", report.render().replace(['\n', '\t'], " ")))
                .unwrap_or_default()
        )
    })?;
    let elapsed = started.elapsed().as_millis();

    let outcome = batch
        .into_targets()
        .pop()
        .ok_or_else(|| "the batch carried no target outcome".to_owned())?
        .into_parts()
        .1;
    let compilation = outcome.map_err(|refusal| {
        format!(
            "the target slot refused: {refusal:?}{}",
            refusal
                .explain()
                .map(|report| format!(" | {}", report.render().replace(['\n', '\t'], " ")))
                .unwrap_or_default()
        )
    })?;

    let alternatives = compilation.alternatives().len();
    let widest_alternative_bytes = compilation
        .alternatives()
        .map(|alternative| {
            alternative
                .abi()
                .kernel_program()
                .canonical_identity()
                .as_bytes()
                .len()
        })
        .max()
        .ok_or_else(|| "the portfolio retained no alternative".to_owned())?;

    let selected = compilation
        .selected()
        .ok_or_else(|| "the portfolio named no selected alternative".to_owned())?;
    let verified = selected.abi().kernel_program();

    let mut coverage_records = 0_usize;
    let mut coverage_bytes = 0_usize;
    for stage in verified.stages() {
        for covered in stage.coverage() {
            coverage_records += 1;
            coverage_bytes += covered.refinement().as_bytes().len();
        }
    }

    Ok((
        Compiled {
            identity: verified.canonical_identity().as_bytes().to_vec(),
            graph_bytes: verified.semantic_graph_identity().as_bytes().len(),
            coverage_records,
            coverage_bytes,
            stages: verified.stages().len(),
            alternatives,
            widest_alternative_bytes,
        },
        elapsed,
    ))
}

/// Builds `input`, one shared constant, and a chain of `operations - 1` multiplies.
///
/// The chain is the cheapest program family whose operation count is a free
/// parameter while every other property this measurement depends on stays
/// fixed: one input, one output, one dtype, one extent, and a body the
/// pointwise expression vocabulary spells at any depth. The constant is hoisted
/// and shared rather than minted per step, so the operation count is exactly
/// `1 + multiplies` — every integer in the reachable domain, and a column a
/// reader can check by hand.
///
/// # Panics
///
/// If `operations` is below two; the constant reaches the output only through a
/// multiply, so a one-operation chain is an unreachable constant beside an
/// input forwarded straight to the output — a different program shape, and one
/// this build refuses for recognition.
fn chain_program(operations: usize) -> SemanticProgram {
    assert!(
        operations >= 2,
        "the chain needs a multiply to make its constant output-reachable"
    );
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
            Shape::from_dims([EXTENT]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the scale applies");

    let mut current = input;
    for _ in 1..operations {
        current = F32Multiply::apply(&mut builder, current, scale).expect("the product applies");
    }

    builder
        .output(
            OutputKey::new("result").expect("the output key is valid"),
            current,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// A semantically valid program this build cannot lower.
///
/// Used only by `--perturb=program`. It is a *verified* semantic program — the
/// perturbation is not a malformed graph — whose access relation the scheduled
/// region vocabulary cannot express, so the compiler refuses it for
/// recognition under every contract.
fn unrecognized_program() -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
            Shape::from_dims([EXTENT]),
        )
        .expect("the input binds");
    let reversed = F32Reindex::apply(
        &mut builder,
        &ReindexForm::reverse_axis(Axis::new(0)).expect("the reversal form is valid"),
        input,
    )
    .expect("the reindex applies");
    builder
        .output(
            OutputKey::new("result").expect("the output key is valid"),
            reversed,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Writes one measured row.
#[allow(clippy::cast_precision_loss, reason = "reported to three decimals")]
fn print_row(row: &Row) {
    let mean_record = if row.coverage_records == 0 {
        0.0
    } else {
        row.coverage_bytes as f64 / row.coverage_records as f64
    };
    let per_square = row.program_bytes as f64 / (row.operations as f64).powi(2);
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{mean_record:.1}\t{per_square:.3}\t{}",
        row.requested,
        row.operations,
        row.coverage_records,
        row.stages,
        row.alternatives,
        row.graph_bytes,
        row.program_bytes,
        row.widest_alternative_bytes,
        row.coverage_bytes,
        row.compile_ms,
    );
}

/// Prints the growth reading, the structural decomposition, and the refusal point.
///
/// Returns whether the measured curve supported the claims made about it.
#[allow(clippy::cast_precision_loss, reason = "reported to a few decimals")]
fn summarize(rows: &[Row]) -> bool {
    println!();
    println!("# measured points: {}", rows.len());
    if rows.len() < 3 {
        println!("# TOO FEW POINTS TO READ AN EXPONENT: a growth claim needs at least three.");
        return false;
    }

    print_growth_tables(rows);

    let exponent = log_log_exponent(rows);
    println!("#");
    println!(
        "# log-log least-squares exponent over all {} points: {exponent:.4}",
        rows.len()
    );
    println!(
        "# READ THAT WITH THE FIT BELOW, NOT INSTEAD OF IT. An exponent near 1 does not refute \
         Theta(n^2) here; it reports that the linear term still dominates everywhere the governed \
         budget lets a program reach."
    );

    let last = &rows[rows.len() - 1];
    println!("#");
    println!(
        "# MAX_PROGRAM_IDENTITY_BYTES = {MAX_PROGRAM_IDENTITY_BYTES} ({} MiB)",
        MAX_PROGRAM_IDENTITY_BYTES / (1024 * 1024)
    );
    println!(
        "# widest measured point: {} operations at {} bytes, {:.5}% of the bound",
        last.operations,
        last.program_bytes,
        100.0 * last.program_bytes as f64 / MAX_PROGRAM_IDENTITY_BYTES as f64
    );

    let Some(program_fit) = exact_quadratic(rows, |row| row.program_bytes) else {
        println!("#");
        println!(
            "# NO EXACT QUADRATIC FITS the measured program-identity curve, so no refusal point \
             is stated. The structural prediction is Theta(n^2); a curve that is not exactly \
             quadratic over consecutive operation counts means the encoding carries a term this \
             sweep did not model, and extrapolating through it would invent a number."
        );
        return false;
    };
    let graph_fit = exact_quadratic(rows, |row| row.graph_bytes);

    println!("#");
    println!(
        "# EXACT FIT over every measured point: program_bytes(n) = {:.0}n^2 + {:.0}n + {:.0}",
        program_fit.0, program_fit.1, program_fit.2
    );
    if let Some(graph) = graph_fit {
        println!(
            "#   graph_bytes(n) = {:.0}n^2 + {:.0}n + {:.0}",
            graph.0, graph.1, graph.2
        );
        println!(
            "#   THE MECHANISM, stated as an equality a reader can check: the program curve's \
             quadratic coefficient ({:.0}) is the graph curve's per-operation slope ({:.0}). That \
             is one whole graph identity embedded per coverage record, one record per operation — \
             not a resemblance to n^2 but the product that makes it one.",
            program_fit.0, graph.1
        );
    }

    // Where the quadratic term overtakes the linear one. Below it the curve
    // reads as linear no matter how many points are sampled, which is why the
    // log-log exponent above is what it is.
    if program_fit.0 > 0.0 {
        println!(
            "#   the quadratic term overtakes the linear term at n = {:.0} operations; every \
             measured point sits below that, which is what the exponent above is reporting.",
            program_fit.1 / program_fit.0
        );
    }

    let refusal = first_refusing_operation_count(program_fit);
    println!("#");
    println!(
        "# EXTRAPOLATED refusal point: the fitted curve first exceeds the bound at n = {refusal} \
         operations ({} bytes at n = {refusal}, {} bytes at n = {}).",
        evaluate(program_fit, refusal),
        evaluate(program_fit, refusal - 1),
        refusal - 1
    );
    println!(
        "# THE FIT IS EXACT ON ITS DOMAIN AND THE DOMAIN IS 2..=8 OPERATIONS. Every coefficient \
         above is a property of this one program family: graph identity per operation depends on \
         operation-key length, arity, result rank, and attribute width, and the per-record \
         remainder depends on the region, the reached definitions, and the admission provenance. \
         A different family moves all three coefficients, so this number is the order of \
         magnitude at which the bound becomes binding, not a refusal a caller can rely on."
    );
    true
}

/// Writes the structural decomposition and the consecutive-growth tables.
///
/// The decomposition is the structural claim checked directly rather than only
/// through an exponent. Each coverage record embeds one whole
/// `SemanticGraphIdentity`, so the mean record exceeds the graph identity by a
/// per-record remainder; and there is one record per operation, so the product
/// of the two is what makes the total quadratic. Reading the remainder column
/// is what tells a quadratic mechanism apart from a curve that merely resembles
/// one over this ladder.
#[allow(clippy::cast_precision_loss, reason = "reported to a few decimals")]
fn print_growth_tables(rows: &[Row]) {
    println!("#");
    println!("# structural decomposition (the mechanism, not just the exponent):");
    println!("#   operations\tgraph_bytes\tmean_record_bytes\trecord_minus_graph");
    for row in rows {
        let mean_record = row.coverage_bytes as f64 / row.coverage_records as f64;
        println!(
            "#   {}\t{}\t{mean_record:.1}\t{:.1}",
            row.operations,
            row.graph_bytes,
            mean_record - row.graph_bytes as f64
        );
    }

    println!("#");
    println!("# consecutive growth (each step adds one operation):");
    println!("#   ops_from -> ops_to\tops_ratio\tprogram_bytes_ratio\tlocal_exponent");
    for pair in rows.windows(2) {
        let ops_ratio = pair[1].operations as f64 / pair[0].operations as f64;
        let byte_ratio = pair[1].program_bytes as f64 / pair[0].program_bytes as f64;
        println!(
            "#   {} -> {}\t{ops_ratio:.3}\t{byte_ratio:.3}\t{:.4}",
            pair[0].operations,
            pair[1].operations,
            byte_ratio.ln() / ops_ratio.ln()
        );
    }
}

/// The log-log least-squares slope of program bytes against operation count.
///
/// A slope over the whole ladder rather than a ratio between two points: the
/// ladder spans one order of magnitude at most, so one pair's fixed per-program
/// overhead would otherwise decide the exponent.
#[allow(clippy::cast_precision_loss, reason = "byte counts are small integers")]
fn log_log_exponent(rows: &[Row]) -> f64 {
    let count = rows.len() as f64;
    let mean_x = rows
        .iter()
        .map(|row| (row.operations as f64).ln())
        .sum::<f64>()
        / count;
    let mean_y = rows
        .iter()
        .map(|row| (row.program_bytes as f64).ln())
        .sum::<f64>()
        / count;
    let (mut covariance, mut variance) = (0.0_f64, 0.0_f64);
    for row in rows {
        let x = (row.operations as f64).ln() - mean_x;
        covariance += x * ((row.program_bytes as f64).ln() - mean_y);
        variance += x * x;
    }
    covariance / variance
}

/// Fits `a*n^2 + b*n + c` by finite differences and returns it only if exact.
///
/// Exact means every measured point is reproduced to the byte. The refusal
/// point is solved from these coefficients, so an approximate fit silently
/// accepted here would become a precise-looking number with nothing behind it.
///
/// Requires consecutive integer operation counts, which is what makes second
/// differences a fit rather than an interpolation; returns `None` otherwise.
#[allow(clippy::cast_precision_loss, reason = "byte counts are small integers")]
fn exact_quadratic(rows: &[Row], of: impl Fn(&Row) -> usize) -> Option<(f64, f64, f64)> {
    if rows.len() < 3 {
        return None;
    }
    if rows
        .windows(2)
        .any(|pair| pair[1].operations != pair[0].operations + 1)
    {
        return None;
    }

    let first = rows[0].operations as f64;
    let (y0, y1, y2) = (
        of(&rows[0]) as f64,
        of(&rows[1]) as f64,
        of(&rows[2]) as f64,
    );
    // Half the second difference, which is the quadratic coefficient over
    // consecutive integers. Spelled as the difference it is rather than through
    // `f64::midpoint`, whose name would describe a different computation.
    #[allow(
        clippy::manual_midpoint,
        reason = "a second difference, not a midpoint"
    )]
    let a = (y2 - 2.0 * y1 + y0) / 2.0;
    // Recovered at n = first, then shifted back to the polynomial in n.
    let b = y1 - y0 - a * (2.0 * first + 1.0);
    let c = y0 - a * first * first - b * first;

    let exact = rows.iter().all(|row| {
        let n = row.operations as f64;
        ((a * n * n + b * n + c) - of(row) as f64).abs() < 0.5
    });
    exact.then_some((a, b, c))
}

/// Evaluates a fitted curve at one operation count.
#[allow(clippy::cast_precision_loss, reason = "byte counts are small integers")]
#[allow(clippy::cast_possible_truncation, reason = "reported as a byte count")]
#[allow(clippy::cast_sign_loss, reason = "the fitted curve is positive here")]
fn evaluate(fit: (f64, f64, f64), operations: usize) -> u64 {
    let n = operations as f64;
    (fit.0 * n * n + fit.1 * n + fit.2).round() as u64
}

/// The smallest operation count whose fitted identity exceeds the bound.
///
/// Walked up from the closed-form root rather than trusted from it, so the
/// reported integer is the one the fitted curve actually crosses at instead of
/// a rounding of a square root.
#[allow(clippy::cast_precision_loss, reason = "byte counts are small integers")]
#[allow(clippy::cast_possible_truncation, reason = "an operation count")]
#[allow(clippy::cast_sign_loss, reason = "the root is positive")]
fn first_refusing_operation_count(fit: (f64, f64, f64)) -> usize {
    let limit = MAX_PROGRAM_IDENTITY_BYTES as u64;
    let discriminant = fit.1.mul_add(fit.1, 4.0 * fit.0 * (limit as f64 - fit.2));
    let root = (-fit.1 + discriminant.sqrt()) / (2.0 * fit.0);
    let mut operations = (root as usize).saturating_sub(2).max(1);
    while evaluate(fit, operations) <= limit {
        operations += 1;
    }
    operations
}

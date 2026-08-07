//! Measures kernel-program identity growth against `MAX_PROGRAM_IDENTITY_BYTES`.
//!
//! `measure-executable-coverage-identity-growth-against-the-program-identity-bound`
//! owned a structural inference with exactly one measured point behind it:
//! `CanonicalKernelProgramIdentity` embeds one whole reached-only
//! executable-coverage identity per covered occurrence, one record per graph
//! operation, and each of those records embedded the complete
//! `SemanticGraphIdentity` of the bound graph — so program identity should be
//! quadratic in graph size against a hard 64 MiB bound that fails closed. The
//! first sweep measured exactly that. ADR 0104 then folded the per-record graph
//! identity to a fixed-width digest, which is why the fit this sweep now reports
//! is linear: the general form fitted here is `a*n^2 + b*n + c`, and `a` is a
//! measured *outcome* rather than an assumption, so the same harness reads both
//! encodings and names which one the tree is in.
//!
//! It compiles programs of increasing operation count through the **ordinary**
//! path — the public `tiler_compiler::session::compile` boundary, whose lowering
//! mints real index-refinement receipts, derives `CoveredOccurrence` records
//! from them, and drives `KernelProgramBuilder` — and reads the identity byte
//! length off the verified program each compilation produced. Nothing here
//! constructs an identity, a receipt, or a coverage record itself; a synthetic
//! one would measure this file rather than the compiler.
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
//! Four `--perturb` modes exist to watch those refusals fire rather than
//! trust them; see [`Perturbation`]. Each exits non-zero.
//!
//! # Reading the output
//!
//! One TSV row per ladder point on stdout, then a summary block of `#` comment
//! lines carrying the structural decomposition, the exact fit, and the
//! extrapolated refusal point solved from it. The run ends by compiling every
//! program in [`WALLS`] and requiring each to refuse *with the class recorded
//! beside it*, so the ladder's claim to be the whole reachable domain, and the
//! attribution of each wall above it, are both measured rather than asserted.
//!
//! Run it from this directory:
//!
//! ```sh
//! cargo run --release > results/<date>-<host>/growth.tsv
//! ```

use std::process::ExitCode;
use std::time::Instant;

use tiler_build::BoundMetalCompileDeclaration;
use tiler_compiler::session::{
    CompileFailureClass, CompileRequest, ExplainReport, NumericalContract, compile,
};
use tiler_compiler::target::TargetRequest;
use tiler_ir::program::MAX_PROGRAM_IDENTITY_BYTES;
use tiler_ir::semantic::{
    F32, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;

/// Operation counts swept, which is the whole reachable domain.
///
/// **This ladder is not a sample; it is every program size the ordinary
/// compilation path admits for this program family.** It is *not* the domain
/// `DeterministicBudgets::governed`'s `semantic_operations` names. That budget
/// is 62, and 62 is not reachable: a search bound below it stops this family
/// first, and [`WALLS`] compiles at each point and records which bound refuses.
///
/// The derivation, measured rather than read off constants:
///
/// - **2..=11 compiles.** Every point verifies, carries one coverage record per
///   semantic operation, and retains a selected alternative.
/// - **12..=62 refuses `NoFeasiblePlan`.** `region_expansions` (10,000) stops
///   candidate growth before the whole-program region is reached, and every
///   surviving singleton cover names an unimplemented region, so the portfolio
///   is empty.
/// - **63 refuses `BudgetExhausted`**, which is `semantic_operations = 62`
///   itself — the only one of the two that is about program *size*.
///
/// **Eleven was a wall on 2026-08-06 and is a ladder point now.** It refused
/// `InvalidCompilerOutput` because the coverage-gap explain rule emitted one
/// record per (cover, region) pair and cover enumeration reached about 2,300 of
/// them against a single unimplemented singleton region, exhausting the explain
/// writer's canonical-byte ceiling — a program-size limit no budget declared,
/// filed and fixed as
/// `refuse-nothing-legal-on-the-explain-detail-ceiling`. The rule now emits one
/// record per unimplemented region carrying the count of covers it blocked, and
/// the point this table was written to report is that the wall it removed was
/// never about the program.
///
/// So the domain widened from seven points to ten, and it widened by three
/// rather than by fifty-five because the budget that moved is not the budget
/// that binds. Ten consecutive integers is what makes the second-difference fit
/// in [`exact_quadratic`] a fit rather than an interpolation.
///
/// The generator emits one shared constant and a chain of multiplies, so the
/// operation count is `1 + multiplies` and every integer in the domain is
/// reachable.
const OPERATIONS: &[usize] = &[2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

/// One refusal above the ladder, and the class that must raise it.
struct Wall {
    /// The operation count probed.
    operations: usize,
    /// The class the compiler must refuse with.
    class: CompileFailureClass,
    /// What that refusal is, in the terms of the bound that produces it.
    why: &'static str,
}

/// The refusals bounding the ladder, each probed rather than read off a source.
///
/// The predecessor of this table was a single nine-operation probe asserting
/// that the governed budget refused. It fired on 2026-08-06 — the budget had
/// moved from 8 to 62 and the probe compiled instead of refusing — which is the
/// finding it exists to report and the reason this file was rewritten.
///
/// A single point cannot re-anchor the discipline now, because distinct bounds
/// refuse between the ladder's top and the governed budget and they are not
/// interchangeable: one is a search bound whose exhaustion the compiler reports
/// as an infeasible target, and only the other is the program-size budget.
/// Probing each **with its class** is what makes a wall that moves *in kind* —
/// a search budget widened, a program budget moved — fail loudly rather than
/// pass as "something refused".
///
/// **This table has fired twice, and both firings are the reason to keep it.**
/// The 2026-08-06 run reported a probe that compiled where the governed budget
/// was expected to refuse, which is what replaced its single-point predecessor.
/// The run below it reported the same arm again at eleven operations, where an
/// explain-ceiling entry stood: `refuse-nothing-legal-on-the-explain-detail-ceiling`
/// removed the per-cover restatement that exhausted it, the probe compiled, and
/// the row moved into [`OPERATIONS`]. An entry leaving this table because the
/// defect behind it was fixed is the outcome it exists to produce.
///
/// 62 is probed explicitly because it is the governed budget's own maximum: the
/// largest program the profile admits by size is measured to refuse for a reason
/// that has nothing to do with size.
const WALLS: &[Wall] = &[
    Wall {
        operations: 12,
        class: CompileFailureClass::NoFeasiblePlan,
        why: "region_expansions (10,000) stops growth before the whole-program candidate, leaving \
              a portfolio whose every cover names an unimplemented region",
    },
    Wall {
        operations: 62,
        class: CompileFailureClass::NoFeasiblePlan,
        why: "the governed semantic_operations maximum, which the same expansion bound refuses \
              long before its own budget would",
    },
    Wall {
        operations: 63,
        class: CompileFailureClass::BudgetExhausted,
        why: "semantic_operations = 62, the one wall here that is about program size",
    },
];

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
/// its passing verdict means anything. Every arm below ends the run non-zero,
/// and each exercises a different refusal: the compile path, the coverage
/// completeness assertion, the exact-fit residual check, and the wall table's
/// class comparison.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Perturbation {
    /// None; the ordinary sweep.
    None,
    /// Emit a program no plan covers, and watch the run refuse.
    ///
    /// See [`unplannable_program`] for which program and why it is derived from
    /// the wall table rather than written down. What it exercises is the arm the
    /// sweep must never paper over: a compilation that does not reach a verified
    /// kernel program stops the run instead of leaving a gap in the ladder.
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
    /// Expect the wrong class at the first wall, and watch the table refuse.
    ///
    /// [`WALLS`]'s other arm — a probe that compiles where a refusal was
    /// expected — has fired for real: it is what the 2026-08-06 run reported
    /// when `semantic_operations` moved from 8 to 62, and it is why this table
    /// exists. The class comparison is the arm that has never fired, so it is
    /// the one that needs watching. Naming the wrong expected class leaves the
    /// compiler untouched and moves only the harness's expectation, which is
    /// what makes the refusal attributable to the comparison.
    Wall,
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
            eprintln!(
                "unknown argument {argument:?}; expected --perturb=program|coverage|fit|wall"
            );
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
         bytes_per_op\tcompile_ms"
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
                eprintln!("REFUSED at operations={operations}: {}", refusal.diagnosis);
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
    if !probe_the_walls(&declaration, perturbation) {
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

/// Compiles every program in [`WALLS`], requiring each stated refusal.
///
/// The ladder above claims to be the *whole* reachable domain, and that claim is
/// only worth something if the points outside it actually refuse — and refuse
/// for the reasons the ladder's own derivation names. Two ways this can fail and
/// both are findings rather than passes: a probe that **compiles** means the
/// domain is wider than the ladder, and a probe that refuses with a **different
/// class** means the bound that binds has changed identity. Either ends the run
/// non-zero and says what to do; neither is a hardware or timing property, so a
/// loaded host cannot produce one.
///
/// Every wall is probed even after one fails, because "which of the four moved"
/// is the whole content of the report.
fn probe_the_walls(declaration: &BoundMetalCompileDeclaration, perturbation: Perturbation) -> bool {
    println!("#");
    println!(
        "# THE WALLS ABOVE THE LADDER, each compiled and required to refuse with the class named:"
    );
    let mut held = true;
    for (index, wall) in WALLS.iter().enumerate() {
        // The perturbation moves only this harness's expectation, so a refusal
        // it produces is attributable to the comparison and to nothing else.
        let expected = if perturbation == Perturbation::Wall && index == 0 {
            CompileFailureClass::BudgetExhausted
        } else {
            wall.class
        };
        let program = chain_program(wall.operations);
        match compile_once(&program, declaration) {
            Err(refusal) if refusal.class == Some(expected) => {
                println!(
                    "#   {} operations: CONFIRMED {expected:?} — {} [{}]",
                    wall.operations, wall.why, refusal.summary
                );
            }
            Err(refusal) => {
                eprintln!(
                    "THE WALL CHANGED KIND at {} operations: this table expects {expected:?} and \
                     the compiler refused with {}. The bound that binds here is no longer the one \
                     the ladder's derivation names, so the recorded domain and every figure \
                     derived from it are stale. Re-derive WALLS and rerun.",
                    wall.operations, refusal.summary
                );
                held = false;
            }
            Ok((compiled, _)) => {
                eprintln!(
                    "THE WALL MOVED: {} operations compiled to a {}-byte identity where \
                     {expected:?} was required, so this ladder is no longer the whole reachable \
                     domain. Widen OPERATIONS, re-derive WALLS, and rerun; the recorded result and \
                     its verdict are stale.",
                    wall.operations,
                    compiled.identity.len()
                );
                held = false;
            }
        }
    }
    if held {
        println!(
            "# so the ladder above is the entire domain the ordinary compilation path admits for \
             this family, and the governed semantic_operations budget of 62 is measured to be \
             unreachable rather than assumed to bound it."
        );
    }
    held
}

/// Reads the one optional argument, rejecting anything else.
fn parse_perturbation() -> Result<Perturbation, String> {
    let mut perturbation = Perturbation::None;
    for argument in std::env::args().skip(1) {
        perturbation = match argument.as_str() {
            "--perturb=program" => Perturbation::Program,
            "--perturb=coverage" => Perturbation::Coverage,
            "--perturb=fit" => Perturbation::Fit,
            "--perturb=wall" => Perturbation::Wall,
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
) -> Result<Row, Refusal> {
    let program = if perturbation == Perturbation::Program {
        unplannable_program()
    } else {
        chain_program(requested)
    };
    let operations = program.operation_count();

    let (first, first_ms) = compile_once(&program, declaration)?;
    let (second, second_ms) = compile_once(&program, declaration)?;
    if first.identity != second.identity {
        return Err(Refusal::harness(&format!(
            "two compilations of one program produced different identity bytes ({} then {}); the \
             encoding is not a function of program content and no byte count here means anything",
            first.identity.len(),
            second.identity.len()
        )));
    }

    let expected_coverage = match perturbation {
        Perturbation::Coverage => operations + 1,
        _ => operations,
    };
    if first.coverage_records != expected_coverage {
        return Err(Refusal::harness(&format!(
            "the selected alternative covers {} semantic occurrences but the graph has {} \
             operations; a coverage set that is not the whole graph is not the subject this \
             measurement is about",
            first.coverage_records, expected_coverage
        )));
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

/// One compilation that did not produce a verified program.
///
/// The class is the compiler's own, read through the public accessor rather than
/// scraped from a rendered trace — `ExplainReport` documents its text as a
/// diagnostic and not a parse target, so [`WALLS`] compares the typed value and
/// nothing else. It is `None` for a refusal this harness raised about a
/// compilation that *succeeded*, which no compiler class describes.
///
/// Two renderings because they have two sinks. A refused ladder point aborts the
/// sweep and its whole trace belongs on stderr; a confirmed wall is one line in
/// a retained result, and a wall's trace runs to hundreds of records — the
/// eleven-operation one reached 3,478 before its explain ceiling was fixed,
/// which is a megabyte of TSV comment nobody reads.
struct Refusal {
    /// The compiler's classification, absent for a harness-raised refusal.
    class: Option<CompileFailureClass>,
    /// One short line: which boundary refused, with what class and trace size.
    summary: String,
    /// The summary and, where one exists, the complete rendered explain trace.
    diagnosis: String,
}

impl Refusal {
    /// A refusal this harness raised about an otherwise successful compilation.
    fn harness(diagnosis: &str) -> Self {
        Self {
            class: None,
            summary: diagnosis.to_owned(),
            diagnosis: diagnosis.to_owned(),
        }
    }

    /// A refusal the compiler raised, with its class and its trace kept apart.
    fn compiler(
        class: CompileFailureClass,
        summary: String,
        trace: Option<ExplainReport<'_>>,
    ) -> Self {
        let rendered = trace
            .map(|report| format!(" | {}", report.render().replace(['\n', '\t'], " ")))
            .unwrap_or_default();
        Self {
            class: Some(class),
            diagnosis: format!("{summary}{rendered}"),
            summary,
        }
    }
}

/// Drives the public compile boundary once and reads the verified program.
fn compile_once(
    program: &SemanticProgram,
    declaration: &BoundMetalCompileDeclaration,
) -> Result<(Compiled, u128), Refusal> {
    let targets = TargetRequest::new([declaration.profile().clone()]).map_err(|error| {
        Refusal::harness(&format!(
            "the singleton target request does not build: {error:?}"
        ))
    })?;
    let request = CompileRequest::new(program, CONTRACT, targets);

    let started = Instant::now();
    let batch = compile(request).map_err(|failure| {
        Refusal::compiler(
            failure.class(),
            format!("the compilation batch refused: {failure:?}"),
            failure.explain(),
        )
    })?;
    let elapsed = started.elapsed().as_millis();

    let outcome = batch
        .into_targets()
        .pop()
        .ok_or_else(|| Refusal::harness("the batch carried no target outcome"))?
        .into_parts()
        .1;
    let compilation = outcome.map_err(|refusal| {
        Refusal::compiler(
            refusal.class(),
            format!("the target slot refused: {refusal:?}"),
            refusal.explain(),
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
        .ok_or_else(|| Refusal::harness("the portfolio retained no alternative"))?;

    let selected = compilation
        .selected()
        .ok_or_else(|| Refusal::harness("the portfolio named no selected alternative"))?;
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

/// A semantically valid program this build reaches no verified plan for.
///
/// Used only by `--perturb=program`. It is a *verified* semantic program — the
/// perturbation is not a malformed graph — that no portfolio covers, so the
/// compilation refuses and the sweep must abort rather than print a partial
/// ladder.
///
/// # Why it is read out of [`WALLS`] rather than written here
///
/// Its predecessor was a reverse-axis `tiler::reindex-f32@1`, justified by an
/// access relation the scheduled region vocabulary could not spell. That
/// justification expired: all six `ReindexFormKind` arms are recognized on this
/// tree, so the perturbed program compiled, the sweep measured nine copies of a
/// one-operation graph, and the run still exited non-zero — from the fit check
/// refusing a degenerate ladder rather than from the arm this mode exists to
/// watch. A perturbation that stops perturbing while its exit code stays 1 is
/// worse than none.
///
/// Taking the point from the wall table removes the standing claim entirely. The
/// same run that uses this program also compiles it under [`probe_the_walls`]
/// and requires the refusal, so this mode cannot silently stop testing what it
/// says it tests: the wall would fail first, loudly, and say which one moved.
/// The `NoFeasiblePlan` wall rather than the `BudgetExhausted` one because an
/// infeasible target refuses a *verified* program, which is the arm this mode
/// watches, where a budget refusal never reaches planning at all.
fn unplannable_program() -> SemanticProgram {
    let wall = WALLS
        .iter()
        .find(|wall| wall.class == CompileFailureClass::NoFeasiblePlan)
        .expect("the wall table names a point no plan covers");
    chain_program(wall.operations)
}

/// Writes one measured row.
#[allow(clippy::cast_precision_loss, reason = "reported to three decimals")]
fn print_row(row: &Row) {
    let mean_record = if row.coverage_records == 0 {
        0.0
    } else {
        row.coverage_bytes as f64 / row.coverage_records as f64
    };
    // Bytes per operation rather than per operation squared. The column exists
    // to be read down: a value that settles is a linear curve, a value that
    // climbs is a quadratic one, and which of the two the encoding produces is
    // what ADR 0104 changed.
    let per_operation = row.program_bytes as f64 / row.operations as f64;
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{mean_record:.1}\t{per_operation:.3}\t{}",
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
        "# READ THAT WITH THE FIT BELOW, NOT INSTEAD OF IT. An exponent is a summary of where the \
         domain is and not a statement of what the curve is: over the pre-ADR-0104 quadratic it \
         read 1.09, because the linear term dominated everywhere a program could reach. Only the \
         exact fit below distinguishes the two encodings, and it does so by reproducing every \
         measured point to the byte."
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
             is stated. Both encodings this harness has read are inside the fitted form — the \
             pre-ADR-0104 restatement is quadratic and the folded digest is linear, which is the \
             same form with a zero leading coefficient — so a curve that is not exactly quadratic \
             over consecutive operation counts means the encoding carries a term this sweep did \
             not model, and extrapolating through it would invent a number."
        );
        return false;
    };
    let graph_fit = exact_quadratic(rows, |row| row.graph_bytes);

    println!("#");
    if program_fit.0 == 0.0 {
        println!(
            "# EXACT FIT over every measured point: program_bytes(n) = {:.0}n + {:.0} — LINEAR, \
             with a quadratic coefficient of exactly zero rather than a small one.",
            program_fit.1, program_fit.2
        );
    } else {
        println!(
            "# EXACT FIT over every measured point: program_bytes(n) = {:.0}n^2 + {:.0}n + {:.0}",
            program_fit.0, program_fit.1, program_fit.2
        );
    }
    if let Some(graph) = graph_fit {
        println!(
            "#   graph_bytes(n) = {:.0}n^2 + {:.0}n + {:.0}",
            graph.0, graph.1, graph.2
        );
        print_mechanism(program_fit.0, graph.1);
    }

    // Where the quadratic term overtakes the linear one. Below it the curve
    // reads as linear no matter how many points are sampled, which is why the
    // log-log exponent above is what it is. A zero quadratic coefficient has no
    // such crossover, and dividing by it would print one.
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
        "# THE FIT IS EXACT ON ITS DOMAIN AND THE DOMAIN IS {}..={} OPERATIONS. Every coefficient \
         above is a property of this one program family: graph identity per operation depends on \
         operation-key length, arity, result rank, and attribute width, and the per-record \
         remainder depends on the region, the reached definitions, and the admission provenance. \
         A different family moves all three coefficients, so this number is the order of \
         magnitude at which the bound becomes binding, not a refusal a caller can rely on.",
        rows[0].operations, last.operations
    );
    println!(
        "# IT IS ALSO AN EXTRAPOLATION ACROSS THREE ORDERS OF MAGNITUDE, and the walls below say \
         why no wider ladder is available: the ordinary compilation path refuses this family at \
         {} operations for reasons that are not program size, so the widest point any measurement \
         can reach is {} and the refusal point above is {:.0}x beyond it.",
        WALLS[0].operations,
        last.operations,
        refusal as f64 / last.operations as f64
    );
    true
}

/// Names which encoding the measured coefficients say the tree carries.
///
/// The quadratic coefficient and the graph curve's per-operation slope are the
/// two numbers whose *relation* is the mechanism. When they are equal, one whole
/// graph identity is written per coverage record and one record per operation,
/// so the product is what makes the total quadratic. When the quadratic
/// coefficient is zero and the graph slope is not, the per-record reference is
/// bounded-width and the product is gone. Reporting the relation rather than
/// asserting either state is what lets one harness read both encodings.
#[allow(clippy::cast_precision_loss, reason = "reported to a few decimals")]
fn print_mechanism(quadratic: f64, graph_slope: f64) {
    if quadratic == 0.0 && graph_slope > 0.0 {
        println!(
            "#   THE MECHANISM, stated as the relation a reader can check: the program curve's \
             quadratic coefficient is 0 while the graph curve still grows at {graph_slope:.0} \
             bytes per operation. The per-record graph reference no longer scales with the graph, \
             so the product that used to make the total quadratic — one whole graph identity per \
             coverage record, one record per operation — is gone. That is ADR 0104's fold, read \
             off the curve rather than off the encoder."
        );
    } else if (quadratic - graph_slope).abs() < 0.5 {
        println!(
            "#   THE MECHANISM, stated as an equality a reader can check: the program curve's \
             quadratic coefficient ({quadratic:.0}) is the graph curve's per-operation slope \
             ({graph_slope:.0}). That is one whole graph identity embedded per coverage record, \
             one record per operation — not a resemblance to n^2 but the product that makes it \
             one."
        );
    } else {
        println!(
            "#   THE MECHANISM IS NEITHER SHAPE THIS HARNESS HAS READ: the program curve's \
             quadratic coefficient is {quadratic:.0} and the graph curve's per-operation slope is \
             {graph_slope:.0}. They are neither equal (one graph identity per record) nor is the \
             first zero (a bounded-width per-record reference), so the encoding carries a term \
             neither reading explains and the coefficients above are a fit without a mechanism."
        );
    }
}

/// Writes the structural decomposition and the consecutive-growth tables.
///
/// The decomposition is the structural claim checked directly rather than only
/// through an exponent, and the three columns are chosen so a reader can tell
/// the two encodings apart by eye. `graph_bytes` grows with the graph under
/// both. `mean_record_bytes` tracked it under the restatement and is decoupled
/// from it under the fold. `coverage_step` — the bytes one added operation adds
/// to the whole coverage section — is the discriminator: it climbs with `n`
/// while each record carries a whole graph identity, and settles to a constant
/// once each record carries a bounded-width reference instead.
#[allow(clippy::cast_precision_loss, reason = "reported to a few decimals")]
fn print_growth_tables(rows: &[Row]) {
    println!("#");
    println!("# structural decomposition (the mechanism, not just the exponent):");
    println!("#   operations\tgraph_bytes\tmean_record_bytes\tcoverage_step");
    let mut previous_coverage: Option<usize> = None;
    for row in rows {
        let mean_record = row.coverage_bytes as f64 / row.coverage_records as f64;
        let step = match previous_coverage {
            Some(previous) => format!("{}", row.coverage_bytes.saturating_sub(previous)),
            None => "-".to_owned(),
        };
        previous_coverage = Some(row.coverage_bytes);
        println!(
            "#   {}\t{}\t{mean_record:.1}\t{step}",
            row.operations, row.graph_bytes,
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
/// a rounding of a root.
///
/// The linear case is solved as a line rather than fed through the quadratic
/// formula. With a zero leading coefficient that formula divides zero by zero,
/// and the `NaN` it produces would cast to a starting point of zero and reach
/// the right answer by a route no reader could check.
#[allow(clippy::cast_precision_loss, reason = "byte counts are small integers")]
#[allow(clippy::cast_possible_truncation, reason = "an operation count")]
#[allow(clippy::cast_sign_loss, reason = "the root is positive")]
fn first_refusing_operation_count(fit: (f64, f64, f64)) -> usize {
    let limit = MAX_PROGRAM_IDENTITY_BYTES as u64;
    let root = if fit.0 == 0.0 {
        (limit as f64 - fit.2) / fit.1
    } else {
        let discriminant = fit.1.mul_add(fit.1, 4.0 * fit.0 * (limit as f64 - fit.2));
        (-fit.1 + discriminant.sqrt()) / (2.0 * fit.0)
    };
    let mut operations = (root as usize).saturating_sub(2).max(1);
    while evaluate(fit, operations) <= limit {
        operations += 1;
    }
    operations
}

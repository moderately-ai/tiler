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
//! trust them; see [`Perturbation`]. Each exits non-zero, and each names the
//! check that refused it — the exit code alone cannot separate a working
//! perturbation from one whose program has quietly started compiling, which is
//! the failure [`unplannable_program`] records twice.
//!
//! # Reading the output
//!
//! One TSV row per ladder point on stdout, then a summary block of `#` comment
//! lines carrying the structural decomposition, the exact fit, and the
//! extrapolated refusal point solved from it. The run ends by compiling every
//! program in [`WALLS`] and requiring each to refuse *with the class and the
//! phase recorded beside it*, and by compiling each control entry and requiring
//! it to succeed — so the ladder's claim to be the whole reachable domain, and
//! the attribution of each bound beside it, are measured rather than asserted.
//!
//! **Every stage runs and the exit code is their conjunction.** A failed ladder
//! or a failed fit does not skip the wall table; the table is the only stage
//! that can attribute a dead perturbation, so an early return there would make
//! `--perturb=program`'s verdict unfalsifiable.
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
/// compilation path admits for this program family.** Since 2026-08-07 it is
/// also exactly the domain `DeterministicBudgets::governed`'s
/// `semantic_operations` names, which it had never been before: for the whole
/// life of this file a bound on *region* size stopped the family first, so the
/// ladder was a truncation of the governed budget rather than the whole of it.
/// [`WALLS`] compiles the point above and records which bound refuses.
///
/// The domain is over the *operation count* with [`EXTENT`] held fixed. The
/// generator's other free parameter has a domain of its own, and [`WALLS`]
/// brackets its upper end rather than sweeping it: no measured column varies
/// with extent, so a second ladder would add rows and no information.
///
/// The derivation, measured rather than read off constants:
///
/// - **2..=62 compiles.** Every point verifies, carries one coverage record per
///   semantic operation, and retains a selected alternative.
/// - **63 refuses `BudgetExhausted`** on `semantic_operations = 62`, which is
///   now the only wall here and is about program size.
///
/// **Thirty-three through sixty-two were a wall until 2026-08-07 and are ladder
/// points now.** They refused `BudgetExhausted` because `region_members` was the
/// bare constant `32` while `semantic_operations` was 62: the recognized
/// partition of a pointwise family is its whole program and nothing smaller is
/// implementable, so the whole-program region was the only cover with a plan and
/// a thirty-three-operation chain needed a region bigger than the profile would
/// form. `derive-the-region-shape-budgets-from-the-declaration` made all three
/// region-shape bounds derivations over the declaration rather than constants —
/// `region_members` from `semantic_operations` (62), `region_live_values` from
/// `semantic_values` (80), and `region_boundary_outputs` from the declared
/// output count (3, *narrower* than the 8 it replaced) — on the ground that a
/// region is a subset of the program it covers. The stated admission envelope
/// and the actual planning envelope became the same formulas over one
/// declaration, and the wall between them dissolved.
///
/// **Twelve was a wall on 2026-08-06 and became a ladder point before that**,
/// for a different reason and in a different class. 12..=62 refused
/// `NoFeasiblePlan` because `region_expansions` (10,000) stopped candidate
/// growth before the whole-program region was formed, leaving every surviving
/// cover naming an unimplemented region. Growth reaches the whole-program set
/// last, so a bound documented to cost an alternative cost the only plan;
/// `region-expansion-exhaustion-loses-the-only-feasible-plan` made region
/// formation retain *both* coverage extremes before growth starts, so the
/// expansion bound now costs what it says it costs.
///
/// So the domain widened from ten points to thirty-one to sixty-one, and it
/// stops at 62 because that is `semantic_operations` itself. Sixty-one
/// consecutive integers is what makes the second-difference fit in
/// [`exact_quadratic`] a fit rather than an interpolation.
///
/// **The three retained results under `results/` older than 2026-08-07 were
/// measured against the constant `region_members = 32`, and their ladders stop
/// at thirty-two for that reason rather than because the curve does anything
/// there.** `results/README.md` records which regime each belongs to; they are
/// evidence about the trees they measured and are not regenerated.
///
/// The generator emits one shared constant and a chain of multiplies, so the
/// operation count is `1 + multiplies` and every integer in the domain is
/// reachable.
const OPERATIONS: &[usize] = &[
    2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
    28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51,
    52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62,
];

/// Which declared bound one wall entry is about.
///
/// An enum rather than a string because [`print_refusal_point`] asks for one
/// entry by name and the table's order is presentation: a widened table must be
/// a build error at the lookup rather than a silent change of which wall the
/// extrapolation report quotes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WallSubject {
    /// `semantic_operations`, the governed budget that ends the ladder's domain.
    ProgramSize,
    /// The declaration's own measured maximum threads per grid axis.
    LaunchGeometry,
}

impl WallSubject {
    /// The label this subject is reported under.
    const fn label(self) -> &'static str {
        match self {
            Self::ProgramSize => "program size",
            Self::LaunchGeometry => "launch geometry",
        }
    }
}

/// One program this build must refuse, and the class and phase that must do it.
struct Wall {
    /// The declared bound this entry probes.
    subject: WallSubject,
    /// Builds the exact program compiled here.
    ///
    /// **A constructor rather than an operation count, and that is what makes a
    /// `reaches_planning: true` entry statable at all.** While an integer named
    /// the program, the table's whole vocabulary was `chain_program(n)` at the
    /// fixed [`EXTENT`] — within which every point either compiles (2..=62) or
    /// refuses at request verification on `semantic_operations` (≥ 63). No
    /// program in that vocabulary refuses after planning at any `n`, so the
    /// deliverable "an entry with `reaches_planning: true`" had an empty solution
    /// set. The generator has a second free parameter the integer could not
    /// reach, and the refusal that parameter runs into is raised inside the
    /// target loop.
    ///
    /// The operation count is read off the built program rather than restated
    /// beside it, so the reported number cannot disagree with what was compiled.
    program: fn() -> SemanticProgram,
    /// A program that must **compile**, or `None` when the ladder is the control.
    ///
    /// **A refusal probed without one is consistent with a broken boundary.**
    /// The first entry needs none: it is one operation above sixty-one measured
    /// rows of the identical family and extent, so the ladder is its control and
    /// a session boundary that refused everything would print no rows at all. The
    /// second needs one, because its parameter is a bound this file cannot read
    /// off the declaration — no public accessor exposes the measured
    /// grid-axis maximum — so the number beside it is written down. The pair is
    /// what removes the standing claim: the refusing program sits one thread
    /// above the recorded bound and the control sits exactly on it, so a measured
    /// row that moved in *either* direction fires one of the two.
    control: Option<fn() -> SemanticProgram>,
    /// The class the compiler must refuse with.
    class: CompileFailureClass,
    /// Whether the refusal is raised after a per-target trace exists.
    ///
    /// A refusal raised while planning one target seals that target's trace and
    /// hands it to the caller; one raised while verifying the request refuses
    /// before any target-qualified trace exists. The two are different phases,
    /// and it is what separates two entries that share one class. It is an
    /// independent property of each wall rather than a restatement of the class:
    /// a `semantic_operations` refusal that started arriving *after* planning
    /// would mean the program-size gate had moved behind the target loop, and a
    /// target rejection arriving *before* it would mean a per-target refusal had
    /// moved in front of the trace boundary. Neither is a finding the class alone
    /// can report.
    reaches_planning: bool,
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
/// **It holds two entries, they are about different bounds, and only one of
/// them is above the ladder.** Between 2026-08-06 and 2026-08-07 it carried two
/// and then three points of the ladder's own sweep, because distinct bounds
/// refused between the ladder's top and the governed budget and they were not
/// interchangeable — one a search bound whose exhaustion the compiler reported
/// as an infeasible target, one a region-shape bound, and only the last the
/// program-size budget. Probing each **with its class and its phase** is what
/// made a wall that moved *in kind* fail loudly rather than pass as "something
/// refused". Every one of those intermediate bounds has since stopped refusing
/// this family, leaving one wall above the ladder, and it refuses before
/// planning.
///
/// **This table has fired four times, and every firing is the reason to keep
/// it.** The 2026-08-06 run reported a probe that compiled where the governed
/// budget was expected to refuse, which is what replaced its single-point
/// predecessor. The run below it reported the same arm at eleven operations,
/// where an explain-ceiling entry stood:
/// `refuse-nothing-legal-on-the-explain-detail-ceiling` removed the per-cover
/// restatement that exhausted it and the row moved into [`OPERATIONS`]. The run
/// below *that* reported both arms at once —
/// `region-expansion-exhaustion-loses-the-only-feasible-plan` made the twelfth
/// point compile *and* changed the sixty-second point's class from
/// `NoFeasiblePlan` to `BudgetExhausted`, so the table said which of the two had
/// happened at each point rather than that "something moved". The fourth firing
/// is this one: `derive-the-region-shape-budgets-from-the-declaration` made
/// `region_members` a derivation from `semantic_operations`, and the
/// thirty-three- and sixty-two-operation entries both **compiled** where
/// `BudgetExhausted` was required. Both left the table and became ladder rows.
/// An entry leaving this table because the bound behind it dissolved is the
/// outcome it exists to produce.
///
/// 62 is no longer probed as a wall because it is no longer one: it is the
/// ladder's widest measured point, and the governed budget's own maximum is
/// therefore measured rather than extrapolated to.
///
/// **The second entry is beside the ladder rather than above it, and it is the
/// one that reaches planning.** The first bounds the ladder's *domain*: it is
/// the point past its top, and the ladder's claim to be the whole reachable
/// domain rests on it. The second bounds the ladder's other fixed parameter, the
/// extent, which no operation count reaches. It is a *two*-operation chain — the
/// smallest the generator builds — over a rank-1 tensor one element wider than
/// the declaration's measured maximum threads per grid axis, and it is the
/// smallest program this build recognizes, covers, offers to the physical
/// provider, and then refuses. That is the compilation [`Perturbation::Program`]
/// exists to watch and the only arm of [`compile_once`] that reads a target
/// slot's own refusal.
///
/// It restores the phase coverage the region-shape derivations dissolved, and it
/// restores it in a way the earlier chain entries did not have: those refused
/// because a *compiler-internal* ceiling stopped a region, and this one refuses
/// because a **measured hardware row** does. It cannot dissolve the way they
/// did, because widening it would mean measuring a wider Apple row rather than
/// deriving a bound differently.
const WALLS: &[Wall] = &[
    Wall {
        subject: WallSubject::ProgramSize,
        program: chain_past_the_operation_budget,
        control: None,
        class: CompileFailureClass::BudgetExhausted,
        reaches_planning: false,
        why: "semantic_operations = 62, the one wall about program size and the one that refuses \
              before any target-qualified trace exists",
    },
    Wall {
        subject: WallSubject::LaunchGeometry,
        program: chain_past_the_grid_axis_bound,
        control: Some(chain_at_the_grid_axis_bound),
        class: CompileFailureClass::NoFeasiblePlan,
        reaches_planning: true,
        why: "the whole-program region's launch geometry needs one thread per element and the \
              declaration measures max_threads_per_grid_axis at 268,435,456, so target.grid-axis \
              rejects every region of every cover and the portfolio is empty — a hard target \
              rejection raised inside the target loop, after the trace is opened",
    },
];

/// The measured maximum threads per grid axis this declaration carries.
///
/// **Written down because no public accessor exposes it**, and cross-checked by
/// the [`WALLS`] entry that brackets it rather than trusted: the refusing probe
/// sits one element above and the control sits exactly on it, so this constant
/// disagreeing with the declaration fires one of the two. Its source is
/// `tiler_build`'s `FIRST_MACOS_APPLE9` row `grid_axis_threads`, which the
/// declaration projects through `declare_measured_max_threads_per_grid_axis`.
const MEASURED_GRID_AXIS_THREADS: u64 = 268_435_456;

/// Builds the chain program one operation past the ladder's top.
fn chain_past_the_operation_budget() -> SemanticProgram {
    chain_program_over(OPERATIONS[OPERATIONS.len() - 1] + 1, EXTENT)
}

/// Builds the smallest chain over an extent one past the measured grid axis.
fn chain_past_the_grid_axis_bound() -> SemanticProgram {
    chain_program_over(2, MEASURED_GRID_AXIS_THREADS + 1)
}

/// Builds the same chain over an extent exactly on the measured grid axis.
fn chain_at_the_grid_axis_bound() -> SemanticProgram {
    chain_program_over(2, MEASURED_GRID_AXIS_THREADS)
}

/// Returns the entry probing one declared bound.
///
/// Looked up by subject rather than by position, because the table's order is
/// presentation and a report quoting "the wall" means one particular one.
///
/// # Panics
///
/// If no entry carries the subject, which is a defect in [`WALLS`].
fn wall(subject: WallSubject) -> &'static Wall {
    WALLS
        .iter()
        .find(|wall| wall.subject == subject)
        .unwrap_or_else(|| panic!("the wall table names the {} bound", subject.label()))
}

/// The tensor extent every *ladder* program is built over.
///
/// Held fixed and small deliberately. Extent enters the graph identity as a
/// handful of bytes per value and enters nothing else this sweep measures, so
/// varying it would add a second axis that moves the curve's constant without
/// touching its exponent — while a large extent costs launch geometry the
/// target profile has to admit.
///
/// **That last clause is now measured rather than only stated.** It is the
/// generator's second free parameter, and the bound it runs into is the one
/// refusal this family has that arrives *after* planning, so [`WALLS`] brackets
/// it. Every row of the ladder still sits at this extent; nothing about the
/// measured curve moves.
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
    /// Emit a program this build reaches no verified kernel program for, and
    /// watch the run refuse.
    ///
    /// See [`unplannable_program`] for which program and why it is derived from
    /// the wall table rather than written down. What it exercises is the arm the
    /// sweep must never paper over: a compilation that does not reach a verified
    /// kernel program stops the run instead of leaving a gap in the ladder.
    ///
    /// **It exercises the *later* of the two aborts again.** The program it
    /// selects reaches planning, so the refusal travels back through
    /// [`compile_once`]'s `into_targets`, `into_parts`, and target-slot
    /// `map_err` arm — the last of which no other run reaches — and carries a
    /// sealed trace the diagnosis prints in full. Between 2026-08-07's two runs
    /// the selection had no such entry and silently took an earlier one.
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
    /// Both of [`WALLS`]'s arms have now fired for real. The compiled-where-a-
    /// refusal-was-expected arm fired when `semantic_operations` moved from 8 to
    /// 62, again at eleven and twelve operations as two defects behind those
    /// walls were fixed, and again at thirty-three and sixty-two when
    /// `derive-the-region-shape-budgets-from-the-declaration` dissolved the
    /// `region_members` wall. The class comparison fired when
    /// `region-expansion-exhaustion-loses-the-only-feasible-plan` moved the
    /// sixty-two-operation refusal from `NoFeasiblePlan` to `BudgetExhausted`.
    /// This mode keeps the second watchable between real firings: naming the
    /// wrong expected class leaves the compiler untouched and moves only the
    /// harness's expectation, which is what makes the refusal attributable to
    /// the comparison.
    ///
    /// It perturbs the first entry only. The table's third arm — a control that
    /// refuses where it must compile — is deliberately not perturbable: moving a
    /// control's expectation would mean asserting that a program which compiles
    /// does not, which is a claim about nothing.
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
    let mut swept = true;
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
                swept = false;
                break;
            }
        }
    }

    // Every verdict below runs whatever the ones before it decided, and the exit
    // code is their conjunction. Returning on the first `false` is what made
    // `--perturb=program`'s documented verdict unable to say no: that mode's
    // program is a [`WALLS`] entry, so the wall table is the only thing in the
    // run that can tell a live perturbation from one whose program has started
    // compiling — and under an early return the ladder or the fit refused first
    // and the table was never reached. Both worlds then exited 1, which is the
    // predecessor failure [`unplannable_program`] records, one layer up.
    let summarized = summarize(&rows);
    let walls_held = probe_the_walls(&declaration, perturbation);
    if swept && summarized && walls_held {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Compiles every program in [`WALLS`], requiring each stated refusal.
///
/// The ladder above claims to be the *whole* reachable domain for its family,
/// and that claim is only worth something if the points outside it actually
/// refuse — and refuse for the reasons the ladder's own derivation names. Two
/// ways this can fail and both are findings rather than passes: a probe that
/// **compiles** means the domain is wider than the table says, and a probe that
/// refuses with a **different class or in a different phase** means the bound
/// that binds has changed identity. Either ends the run non-zero and says what
/// to do; neither is a hardware or timing property, so a loaded host cannot
/// produce one.
///
/// **It runs whatever the ladder and the fit decided**, which is what makes
/// [`unplannable_program`]'s cross-check real rather than asserted: the mode
/// that perturbs the program compiles it here in the same run and requires the
/// refusal, so a perturbation whose program started compiling is reported here
/// by name instead of hiding behind a non-zero exit some other check produced.
///
/// Every wall is probed even after one fails, because "which of them moved" is
/// the whole content of the report. The table has held one, two, and three
/// entries; the loop does not assume a count.
fn probe_the_walls(declaration: &BoundMetalCompileDeclaration, perturbation: Perturbation) -> bool {
    println!("#");
    println!("# THE WALLS, each compiled and required to refuse with the class and phase named:");
    let mut held = true;
    for (index, wall) in WALLS.iter().enumerate() {
        // The perturbation moves only this harness's expectation, so a refusal
        // it produces is attributable to the comparison and to nothing else.
        let expected = if perturbation == Perturbation::Wall && index == 0 {
            CompileFailureClass::NoFeasiblePlan
        } else {
            wall.class
        };
        let subject = wall.subject.label();
        let program = (wall.program)();
        let operations = program.operation_count();
        let phase = if wall.reaches_planning {
            "after"
        } else {
            "before"
        };
        match compile_once(&program, declaration) {
            Err(refusal)
                if refusal.class == Some(expected) && refusal.traced == wall.reaches_planning =>
            {
                println!(
                    "#   {subject}, at {operations} operations: CONFIRMED {expected:?} {phase} \
                     planning — {} [{}]",
                    wall.why, refusal.summary
                );
            }
            Err(refusal) => {
                eprintln!(
                    "THE WALL CHANGED KIND at the {subject} bound, {operations} operations: this \
                     table expects {expected:?} raised {phase} planning, and the compiler refused \
                     with {} (trace {}). The bound that binds here is no longer the one this table \
                     names, so the recorded domain and every figure derived from it are stale. \
                     Re-derive WALLS and rerun.",
                    refusal.summary,
                    if refusal.traced { "present" } else { "absent" },
                );
                held = false;
            }
            Ok((compiled, _)) => {
                eprintln!(
                    "THE WALL MOVED at the {subject} bound: {operations} operations compiled to a \
                     {}-byte identity where {expected:?} was required, so the recorded domain is \
                     no longer what this table says it is. Widen OPERATIONS or re-derive WALLS, \
                     and rerun; the recorded result and its verdict are stale. If this is the \
                     entry --perturb=program reads, that mode has stopped perturbing and its \
                     non-zero exit means nothing until this is fixed.",
                    compiled.identity.len()
                );
                held = false;
            }
        }
        let Some(control) = wall.control else {
            continue;
        };
        // The control must compile, and it is not perturbed: a refusal here is
        // the wall's own bound having widened onto a program this table records
        // as admitted, or a boundary refusing everything — either of which makes
        // the confirmation above evidence about nothing.
        match compile_once(&control(), declaration) {
            // The identity length is reported rather than discarded because it
            // is the one measurement this program carries: read against the
            // ladder row at the same operation count, it says how much the
            // parameter the ladder holds fixed moves the curve it fits.
            Ok((compiled, _)) => println!(
                "#   {subject}, control: CONFIRMED the program one step inside the recorded bound \
                 compiles, to a {}-byte identity, so the refusal above is that bound's and not \
                 the session boundary's.",
                compiled.identity.len()
            ),
            Err(refusal) => {
                eprintln!(
                    "THE CONTROL REFUSED at the {subject} bound: the program one step inside the \
                     recorded bound must compile and refused with {}. The refusal recorded beside \
                     it therefore attributes nothing — a boundary refusing both programs would \
                     report exactly what this run reported. Re-derive the bound and rerun.",
                    refusal.summary
                );
                held = false;
            }
        }
    }
    if held {
        println!(
            "# so the ladder above is the entire domain the ordinary compilation path admits for \
             this family at this extent, and the governed semantic_operations budget of 62 is \
             measured to be the bound that ends it rather than assumed to. Every bound that used \
             to refuse below 62 is measured here to admit instead, and the extent bound beside \
             them is measured to refuse after planning rather than before it."
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
/// a retained result, and a wall's trace can run to hundreds of records — the
/// eleven-operation one reached 3,478 before its explain ceiling was fixed,
/// which is a megabyte of TSV comment nobody reads. The split is load-bearing
/// again now that a wall reaches planning: the launch-geometry entry seals a
/// 26-record trace, which the confirmation line reports the *size* of and
/// `--perturb=program` prints in full.
struct Refusal {
    /// The compiler's classification, absent for a harness-raised refusal.
    class: Option<CompileFailureClass>,
    /// Whether a sealed per-target trace travelled with the refusal.
    ///
    /// Which phase refused, read structurally: a request-verification refusal
    /// precedes the trace boundary and carries none, and a planning refusal
    /// carries the whole trace. [`WALLS`] compares it as an independent property
    /// of the wall, which is what it was worth when several entries shared one
    /// class and remains worth now that one entry is left.
    traced: bool,
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
            traced: false,
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
        let traced = trace.is_some();
        let rendered = trace
            .map(|report| format!(" | {}", report.render().replace(['\n', '\t'], " ")))
            .unwrap_or_default();
        Self {
            class: Some(class),
            traced,
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

/// Builds one ladder program: a chain at the sweep's fixed [`EXTENT`].
fn chain_program(operations: usize) -> SemanticProgram {
    chain_program_over(operations, EXTENT)
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
/// The extent is a parameter here and fixed at [`chain_program`] because the
/// ladder holds it fixed while [`WALLS`] brackets it. Nothing that varies it
/// enters a measured row.
///
/// # Panics
///
/// If `operations` is below two; the constant reaches the output only through a
/// multiply, so a one-operation chain is an unreachable constant beside an
/// input forwarded straight to the output — a different program shape, and one
/// this build refuses for recognition.
fn chain_program_over(operations: usize, extent: u64) -> SemanticProgram {
    assert!(
        operations >= 2,
        "the chain needs a multiply to make its constant output-reachable"
    );
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
            Shape::from_dims([extent]),
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
/// perturbation is not a malformed graph — that this build reaches no kernel
/// program for, so the compilation refuses and the sweep must abort rather than
/// print a partial ladder.
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
/// says it tests: the wall reports it, loudly, and says which one moved.
///
/// **That sentence was asserted here while `main` prevented it, and the fix is
/// what makes it true.** `probe_the_walls` used to run only after a successful
/// `summarize`, so under this mode the table was never reached in *either*
/// world: a live perturbation aborted the ladder first, and a dead one left
/// sixty-one identical rows whose operation counts are not consecutive, which
/// the exact fit refuses. Both exited non-zero without probing anything, so the
/// documented verdict "each exits non-zero" was true in every reachable state
/// including the dead one — the predecessor failure above, one layer up. `main`
/// now accumulates a verdict and runs every check.
///
/// # Which wall it selects, and why on the phase
///
/// The arm worth watching is a compilation that verified, planned, and reached
/// no kernel program — a strictly later abort than a request the program-size
/// budget refuses before any target compiles, and the only one that reaches
/// [`compile_once`]'s target-slot refusal arm. So the selection is on
/// `reaches_planning` rather than on position.
///
/// Two points of this family reached planning while `region_members` was the
/// constant `32`, and `derive-the-region-shape-budgets-from-the-declaration`
/// dissolved both. **No chain program refuses after planning at any operation
/// count**, which is why this selection had no candidate between then and
/// 2026-08-07's later run, and read `WALLS.first()` instead — quietly selecting
/// a refusal one phase earlier than the one it documents. The launch-geometry
/// entry restores a candidate without writing down a program of its own: it is a
/// table entry, so the same run compiles it, requires its class *and* its phase,
/// and compiles the control beside it.
///
/// # Panics
///
/// If no entry reaches planning. That is the honest failure: this mode's whole
/// subject is the later abort, and silently falling back to an earlier one is
/// what it did between 2026-08-07's two runs and what
/// `restore-a-planning-phase-refusal-to-the-identity-growth-harness` was filed
/// about.
fn unplannable_program() -> SemanticProgram {
    let wall = WALLS
        .iter()
        .find(|wall| wall.reaches_planning)
        .expect("the wall table names a refusal raised after planning");
    (wall.program)()
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

    print_refusal_point(program_fit, rows[0].operations, last.operations);
    true
}

/// Writes the extrapolated refusal point and the two bounds on reading it.
///
/// Split out of [`summarize`] so each reads as one claim: that function decides
/// whether the measured curve supports a fit at all, and this one states what
/// the fit does and does not license once it does.
#[allow(clippy::cast_precision_loss, reason = "operation counts are small")]
fn print_refusal_point(fit: (f64, f64, f64), first: usize, last: usize) {
    let refusal = first_refusing_operation_count(fit);
    println!("#");
    println!(
        "# EXTRAPOLATED refusal point: the fitted curve first exceeds the bound at n = {refusal} \
         operations ({} bytes at n = {refusal}, {} bytes at n = {}).",
        evaluate(fit, refusal),
        evaluate(fit, refusal - 1),
        refusal - 1
    );
    println!(
        "# THE FIT IS EXACT ON ITS DOMAIN AND THE DOMAIN IS {first}..={last} OPERATIONS. Every \
         coefficient above is a property of this one program family: graph identity per operation \
         depends on operation-key length, arity, result rank, and attribute width, and the \
         per-record remainder depends on the region, the reached definitions, and the admission \
         provenance. A different family moves all three coefficients, so this number is the order \
         of magnitude at which the bound becomes binding, not a refusal a caller can rely on."
    );
    println!(
        "# IT IS ALSO AN EXTRAPOLATION ACROSS THREE ORDERS OF MAGNITUDE, and the wall below says \
         why no wider ladder is available: the ordinary compilation path refuses this family at \
         {} operations on semantic_operations, the governed program-size budget itself, so the \
         widest point any measurement can reach is {last} and the refusal point above is {:.0}x \
         beyond it. Widening the ladder further is a budget decision rather than a harness one.",
        (wall(WallSubject::ProgramSize).program)().operation_count(),
        refusal as f64 / last as f64
    );
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

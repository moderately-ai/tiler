//! Sweeps the reduction shape space to find where a crossover could be measured.
//!
//! `calibrate-and-activate-parallel-reduction-selection` needs a shape/workgroup
//! matrix over which the three retained reduction alternatives — serial fold,
//! single-workgroup tree, multi-pass split — can each be timed on the qualified
//! Metal host, so that a crossover region can be identified. This spike answers
//! the question that has to be settled *before* any timing harness is worth
//! writing: **over which shapes does the authoritative Apple profile retain all
//! three alternatives at once?**
//!
//! It is deliberately a compile-only sweep. It emits nothing, links nothing, and
//! dispatches nothing, because the domain question is decided entirely by
//! compile-phase feasibility and never reaches the device.
//!
//! # What it drives
//!
//! For each contributor count and row count in the predeclared matrix below it
//! builds the same program shape the parallel-portfolio test uses — an
//! elementwise multiply-add prologue feeding a sum over the trailing axis —
//! compiles it under `NumericalContract::FLUSH_AND_REASSOCIATE_F32` against
//! `BoundMetalCompileDeclaration::first_macos_apple9`, and records one row per
//! shape: whether a portfolio was produced at all, how many alternatives it
//! retained, the (kernel count, widest declared workgroup) pair of each, and
//! which alternative selection chose.
//!
//! The three strategies are told apart by the same device-independent structure
//! the realization ticket used: the multi-pass split is the alternative with
//! three kernels, the single-workgroup tree is the one whose widest declared
//! workgroup exceeds one thread, and the serial fold is the one with neither.
//!
//! # Reading the output
//!
//! One TSV row per shape on stdout, plus a trailing summary naming the shapes
//! that retained all three. A shape that produced no portfolio carries the
//! refusal's own rendering in its last column, so a reader can tell a feasibility
//! refusal from a numerical one without rerunning.
//!
//! Run it from this directory:
//!
//! ```sh
//! cargo run --release > results/<date>-<host>/sweep.tsv
//! ```

use tiler_build::BoundMetalCompileDeclaration;
use tiler_compiler::session::{Compilation, CompileRequest, NumericalContract, compile};
use tiler_compiler::target::TargetRequest;
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

/// Contributor counts swept along the reduced axis.
///
/// Chosen to bracket the two structural thresholds this sweep exists to locate
/// rather than to sample uniformly: `governed_partition` splits nothing below
/// four contributors, and the profile's grid-axis row is the first capacity a
/// growing prologue can exceed. Counts below four establish that the split is
/// absent for a stated reason, counts at and just above four locate the upper
/// edge, and the larger powers of two confirm the edge does not reopen.
const CONTRIBUTORS: [u64; 12] = [1, 2, 3, 4, 5, 6, 8, 9, 12, 16, 64, 1024];

/// Row counts swept across the retained axis.
///
/// Two rows rather than one is the smallest change that multiplies the prologue's
/// work items without changing the reduction's own length, which separates "the
/// reduction is too long" from "the program is too wide" in the recorded refusal.
const ROWS: [u64; 3] = [1, 2, 4];

/// How one alternative's structure classifies it as a reduction strategy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Strategy {
    /// One dispatch chain with a single-threaded fold.
    SerialFold,
    /// A cooperative fold whose declared workgroup exceeds one thread.
    SingleWorkgroupTree,
    /// A three-stage program writing and then consuming partials.
    MultiPassSplit,
}

impl Strategy {
    /// The stable code naming this strategy in the recorded sweep.
    const fn key(self) -> &'static str {
        match self {
            Self::SerialFold => "serial-fold",
            Self::SingleWorkgroupTree => "single-workgroup-tree",
            Self::MultiPassSplit => "multi-pass-split",
        }
    }

    /// Classifies one alternative from its kernel count and widest workgroup.
    ///
    /// The split is tested first because a three-stage program is the one
    /// unambiguous structural signature; a cooperative fold and a serial fold
    /// both carry two stages and are told apart only by the declared width.
    const fn classify(kernels: usize, widest_workgroup: u64) -> Self {
        if kernels >= 3 {
            Self::MultiPassSplit
        } else if widest_workgroup > 1 {
            Self::SingleWorkgroupTree
        } else {
            Self::SerialFold
        }
    }
}

fn main() {
    let declaration = BoundMetalCompileDeclaration::first_macos_apple9()
        .expect("the authoritative macOS Apple9 declaration binds");

    println!(
        "rows\tcontributors\twork_items\tportfolio\talternatives\tstrategies\tselected\trefusal"
    );

    let mut all_three: Vec<(u64, u64)> = Vec::new();

    for rows in ROWS {
        for contributors in CONTRIBUTORS {
            let program = reduction_program(rows, contributors);
            let request = CompileRequest::new(
                &program,
                NumericalContract::FLUSH_AND_REASSOCIATE_F32,
                TargetRequest::new([declaration.profile().clone()]).expect("a singleton request"),
            );
            let work_items = rows * contributors;

            // The batch call can itself fail, and one way it does is a known
            // defect rather than a property of the shape: below four
            // contributors the split declines, and recording that decline
            // produces an explain stage event the writer rejects, failing the
            // whole compilation as `InvalidCompilerOutput`. That is
            // `correct-the-declined-strategy-record-for-an-unsplittable-reduction`,
            // not a statement about the measurable domain — so it is recorded in
            // its own column rather than allowed to end the sweep.
            let outcome = match compile(request) {
                Ok(batch) => {
                    batch
                        .into_targets()
                        .pop()
                        .expect("one target outcome")
                        .into_parts()
                        .1
                }
                Err(batch_failure) => {
                    let rendered = format!("{batch_failure:?}").replace(['\n', '\t'], " ");
                    println!(
                        "{rows}\t{contributors}\t{work_items}\tbatch-failed\t0\t\t\t{rendered}"
                    );
                    continue;
                }
            };

            match outcome {
                Ok(compilation) => {
                    let strategies = retained_strategies(&compilation);
                    let selected = compilation.selected().map_or_else(
                        || "none".to_owned(),
                        |alternative| {
                            let width = widest_workgroup(&alternative);
                            Strategy::classify(alternative.kernels().len(), width)
                                .key()
                                .to_owned()
                        },
                    );
                    let rendered = strategies
                        .iter()
                        .map(|strategy| strategy.key())
                        .collect::<Vec<_>>()
                        .join(",");
                    if strategies.len() == 3 {
                        all_three.push((rows, contributors));
                    }
                    println!(
                        "{rows}\t{contributors}\t{work_items}\tyes\t{}\t{rendered}\t{selected}\t",
                        strategies.len()
                    );
                }
                Err(refusal) => {
                    // The refusing predicate is read from the explain report
                    // rather than from the failure class, because the class only
                    // says *that* no plan was feasible. Which capability axis
                    // refused is the whole finding, and it is exactly what a
                    // `Debug` rendering of the failure omits.
                    let rendered = refusal.explain().map_or_else(
                        || format!("{refusal:?}"),
                        |report| refusing_predicates(&report.render()),
                    );
                    let rendered = rendered.replace(['\n', '\t'], " ");
                    println!("{rows}\t{contributors}\t{work_items}\tno\t0\t\t\t{rendered}");
                }
            }
        }
    }

    println!();
    println!(
        "# shapes retaining all three strategies: {}",
        all_three.len()
    );
    for (rows, contributors) in &all_three {
        println!("#   rows={rows} contributors={contributors}");
    }
    if all_three.len() < 2 {
        println!(
            "# NO CROSSOVER IS MEASURABLE: a crossover needs at least two shapes on which all \
             three alternatives exist, and this profile admits {}.",
            all_three.len()
        );
    }
}

/// Extracts the distinct refusing feasibility events from a rendered report.
///
/// **The report is one whitespace-separated line, not a list of lines**, and the
/// refusal token is `rejected:` rather than `disproved`. Both facts were
/// established by dumping a real report rather than assumed: a first version of
/// this filter split on lines and matched `disproved`, and it returned the same
/// empty answer for every shape in the matrix — a uniform result over a
/// population known to be heterogeneous, which is the signature of a check that
/// never ran rather than one that found nothing.
///
/// Set `TILER_SWEEP_FULL_EXPLAIN=1` to get the whole report instead, which is
/// what to do when a refusal appears on a predicate this filter does not name.
fn refusing_predicates(report: &str) -> String {
    if std::env::var_os("TILER_SWEEP_FULL_EXPLAIN").is_some() {
        return report.to_owned();
    }
    let mut refusing: Vec<&str> = report
        .split_whitespace()
        .filter(|token| token.starts_with("event=") && token.contains(":rejected:"))
        .collect();
    refusing.sort_unstable();
    refusing.dedup();
    if refusing.is_empty() {
        "NO REJECTED EVENT FOUND — rerun with TILER_SWEEP_FULL_EXPLAIN=1".to_owned()
    } else {
        refusing.join(" | ")
    }
}

/// The strategies one compilation's portfolio retained, deduplicated and ordered.
fn retained_strategies(compilation: &Compilation) -> Vec<Strategy> {
    let mut strategies: Vec<Strategy> = compilation
        .alternatives()
        .map(|alternative| {
            let width = widest_workgroup(&alternative);
            Strategy::classify(alternative.kernels().len(), width)
        })
        .collect();
    strategies.sort_unstable_by_key(|strategy| strategy.key());
    strategies.dedup();
    strategies
}

/// The widest workgroup any entry of one alternative declares.
///
/// Read from the alternative's own ABI construction, which is where the declared
/// literal lives, rather than from the kernel source: a width recovered from
/// emitted text would be a claim about the emitter instead of about the plan.
fn widest_workgroup(alternative: &tiler_compiler::session::PlanAlternative<'_>) -> u64 {
    use tiler_ir::program::abi::{AbiRoot, ExprNode};

    let abi = alternative.abi();
    let expressions = abi.expressions();
    abi.entries()
        .map(|entry| {
            let position = usize::try_from(entry.threads_per_workgroup())
                .expect("an arena position fits a usize");
            match expressions.get(position) {
                Some(ExprNode::Root(AbiRoot::UnsignedLiteral(width))) => *width,
                other => panic!("the workgroup width is not a declared literal: {other:?}"),
            }
        })
        .max()
        .unwrap_or(0)
}

/// Builds a `rows x contributors` multiply-add prologue feeding a trailing sum.
///
/// The same shape the authoritative parallel-portfolio test uses, parameterized
/// on both extents. The prologue is kept because it is what makes the multi-pass
/// split expressible at all — the split divides the *materialized* reduction, so
/// a bare sum with no prologue is a different program.
fn reduction_program(rows: u64, contributors: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the semantic profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
            Shape::from_dims([rows, contributors]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the scale applies");
    let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).expect("the bias applies");
    let product = F32Multiply::apply(&mut builder, input, scale).expect("the product applies");
    let mapped = F32Add::apply(&mut builder, product, bias).expect("the bias applies");
    let sum =
        StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).expect("the sum applies");
    builder
        .output(
            OutputKey::new("result").expect("the output key is valid"),
            sum,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

//! Sweeps the *partition* at fixed shapes, which is the inverse of the retained
//! crossover sweep.
//!
//! [`spikes/program-planning/reduction-dispatch-crossover`] varied the shape
//! across 92 cells and timed three strategies at each, and every one of those
//! cells used whatever partition `crates/tiler-compiler/src/physical.rs`'s
//! `governed_partition` returned. The partition was a constant of that
//! experiment rather than a variable in it, so no value of that function is
//! confirmed or refuted by it. This spike holds the shape fixed and varies the
//! partition instead, on shapes drawn from that sweep's separated cells so the
//! two records compose.
//!
//! # How the partition is varied, and what is *not* changed to vary it
//!
//! `governed_partition` is `pub(crate)` and its result is a total function of
//! the contributor count, so no request reaches a second value through the
//! public `compile` entry point. [`regions`] therefore rebuilds the two
//! reduction regions from `tiler-ir`'s published [`tiler_ir::schedule`]
//! vocabulary with the partition as a parameter, lowers them through the
//! compiler's own `lower_scheduled_region`, and emits them beside the *compiler's
//! own* elementwise prologue kernel, taken from the compiler's plan unmodified.
//!
//! **Nothing shipped changes, and the mechanism is checked rather than
//! asserted.** For every shape the sweep first requires that the rebuilt plan at
//! each strategy's current production partition emits the byte-identical
//! translation unit the compiler emits for the same alternative, and declares
//! the same launch extents the compiler's ABI publishes. A shape whose anchor
//! fails is refused outright, so a transcription that drifted from `physical.rs`
//! is a hard failure rather than a partition sweep of a different program. That
//! check is the whole licence for reading the off-production rows as evidence
//! about the compiler's plans.
//!
//! # Both strategies, because they consume the number differently
//!
//! The split's partition count is a launch extent and its
//! contributors-per-partition is a fold length. The tree's partition count is
//! *also* its declared workgroup width and, through the tile's staging, its
//! threadgroup reservation. A partition best for one need not be best for the
//! other, so the two are swept separately and reported separately, and a single
//! value that is best for neither is named as the compromise it would be.
//!
//! # What is measured, and what that number is not
//!
//! Identical to the retained sweep, deliberately. One sample is the wall clock
//! across `commit()` and `wait_until_completed()` for one submission of one
//! whole plan — prologue included, because that is what a consumer pays. `metal`
//! 0.33.0 exposes no accessor for `MTLCommandBuffer`'s `GPUStartTime` or
//! `GPUEndTime`, and reading them would need a new `unsafe` site, which is a
//! decision under ADR 0079 rather than a convenience a spike may take. **This is
//! not a GPU-busy measurement and nothing here should be quoted as one.**
//!
//! The submission round trip costs about 200 microseconds on this host before
//! any kernel runs, which is more than a partition difference at most of these
//! shapes. Each variant is therefore measured at two encode counts — the plan
//! once, and the plan [`BATCH`] times in one command buffer — and the per-plan
//! cost is `(batched - single) / (BATCH - 1)`. The two submissions differ by
//! exactly `BATCH - 1` extra encodes of the same plan and by nothing else, so
//! the fixed cost divides out. It is identical across every partition, so
//! including it could only bury a difference and never manufacture one.
//!
//! # Noise controls
//!
//! - Every variant of a shape is fully prepared — emitted, linked, pipelined,
//!   allocated, input written — before any timing starts.
//! - [`WARMUP`] untimed submissions per variant at each encode count precede the
//!   timed ones.
//! - The timed submissions are **interleaved across every partition and both
//!   strategies of the shape at once**: each round submits every variant once and
//!   the round's starting variant rotates, so a thermal or scheduling drift lands
//!   on all partitions alike instead of on whichever ran last. This is the
//!   control the comparison actually needs, because the compared rows differ only
//!   in the partition and a drift that tracked partition order would look exactly
//!   like a partition effect.
//! - Minimum, median, p90 and sample standard deviation are reported at both
//!   encode counts rather than a single number.
//! - Load averages are recorded before and after.
//!
//! # The oracle
//!
//! Every operand is `1.0`, so a row's declared sum is exactly the contributor
//! count, representable in `f32` for every count reached here. **Every grouping
//! of that row produces the same bits**, which is what makes one expected value
//! valid for every partition of every strategy under a contract that permits
//! regrouping — and a dropped, double-counted, or unsynchronized contributor
//! changes the sum and is caught. Every output element of every variant is
//! checked before that variant is timed, and each variant owns its output buffer
//! so a previous variant's correct answer cannot stand in for a missing write.
//!
//! That closed form is tied to `tiler-reference`'s independent evaluation of the
//! same semantic program once per run, on [`ORACLE_TIE`], before any shape is
//! measured. **Regrouped rounding is not observed and is not claimed**: unit
//! operands cannot expose it.
//!
//! # Running it
//!
//! ```sh
//! cd spikes/program-planning/reduction-partition-calibration
//! DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
//!   cargo run --release --bin reduction-partition-sweep > results/<date>-<host>/sweep.tsv
//! ```
//!
//! `DEVELOPER_DIR` selects the offline toolchain the authority ledger's
//! compilation-environment row names. Without it a host whose default selection
//! is a newer Xcode links through a compiler the profile was not measured under,
//! which is a different environment and makes the run unqualified.
//!
//! No `make` target reaches here, per `spikes/README.md`.
//!
//! [`spikes/program-planning/reduction-dispatch-crossover`]:
//!     ../../reduction-dispatch-crossover/README.md

mod buffer;
mod regions;

use std::time::Instant;

use metal::{
    Buffer, CommandBufferRef, CommandQueue, ComputePipelineDescriptor, ComputePipelineState,
    Device, MTLCommandBufferStatus, MTLGPUFamily, MTLResourceOptions, MTLSize,
};
use regions::{Subject, admissible_partitions, governed_partition};
use tiler_build::BoundMetalCompileDeclaration;
use tiler_compiler::session::{
    CompileRequest as CompilerRequest, NumericalContract, PlanAlternative, compile,
};
use tiler_compiler::target::TargetRequest;
use tiler_ir::kernel::VerifiedKernel;
use tiler_ir::program::abi::{AbiRoot, ExprNode};
use tiler_ir::schedule::{ContributorPartition, NumericalRealization};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};
use tiler_metal::emit::emit_translation_unit;
use tiler_metal::record::MetalTranslationUnit;
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::input::{CompileRequest, OptimizationLevel};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

/// The shapes the partition is swept at.
///
/// **Every one is a cell of the retained crossover sweep, and every one is
/// separated there**, so a partition verdict taken here composes with a strategy
/// verdict taken there rather than describing a different regime. They are drawn
/// across the measured contour on purpose: `(4, 8192)` and `(64, 8192)` sit deep
/// on the side where parallelizing pays by more than an order of magnitude,
/// `(256, 16384)` and `(1024, 4096)` sit where it still pays but by a small
/// factor, `(4096, 2048)` sits astride the contour, and `(16384, 32)` and
/// `(65536, 16)` sit on the side where the serial fold wins and a parallel plan
/// is overhead. A partition effect that appeared only where parallelism already
/// dominates would be a different finding from one that holds across the
/// contour, and this matrix can tell those apart.
///
/// Contributor count four is deliberately absent even though the retained sweep
/// separates several of its cells: four admits exactly one exact split, so it
/// contributes no comparison and would only pad the population.
const SHAPES: [(u64, u64); 7] = [
    (4, 8192),
    (64, 8192),
    (256, 16384),
    (1024, 4096),
    (4096, 2048),
    (16384, 32),
    (65536, 16),
];

/// The finite non-power-of-two matrix measuring the tree's cap excursion.
///
/// Row count four is deep on the retained contour's parallel side and 16,384 is
/// deep on its serial side. The contributor counts separate three divisor
/// lattices: 514 admits only 2 and 257 and production widens to 257; 780 has a
/// dense lattice below the cap and only 260 and 390 above it, with 260 just four
/// participants past the cap and selected; 1,042 admits only 2 and 521 and
/// production stays at 2. The Cartesian product is six shapes and 52 tree
/// variants, all predeclared before any result was observed.
const TREE_WIDTH_EXCURSION_SHAPES: [(u64, u64); 6] = [
    (4, 514),
    (16_384, 514),
    (4, 780),
    (16_384, 780),
    (4, 1_042),
    (16_384, 1_042),
];

/// The shape the closed-form oracle is tied to `tiler-reference` on, once.
///
/// Small enough to evaluate through the reference's boxed element vocabulary on
/// the host. It is not a measured cell: the check it performs is that the closed
/// form *is* the oracle's answer, which is a statement about the arithmetic
/// rather than about the shape, so establishing it once establishes it for every
/// shape whose contributor count `f32` represents exactly.
const ORACLE_TIE: (u64, u64) = (4, 16);

/// Untimed submissions per variant, at each encode count, before any sample.
const WARMUP: usize = 8;

/// Timed rounds per shape; each round submits every variant once, at each of the
/// two encode counts.
const REPETITIONS: usize = 30;

/// How many times the batched submission encodes the same plan.
///
/// **Sixty-four rather than the retained crossover sweep's sixteen, and the
/// reason is the quantity being compared.** That sweep resolved differences
/// between *strategies*, which span two orders of magnitude on this matrix;
/// this one resolves differences between *partitions of one strategy*, which a
/// pilot run at sixteen encodes put at a few percent on several shapes — inside
/// the noise band sixteen encodes leaves. The amortized spread is
/// `(sigma_batched + sigma_single) / (BATCH - 1)` and the single-submission
/// spread is a floor that does not shrink, so the encode count is the only lever
/// that sharpens it: sixty-four moves the band from roughly four microseconds to
/// roughly one, which is what makes a five-percent verdict a verdict instead of
/// a coin toss.
///
/// The per-plan quantity is unchanged by the choice — it is the same difference
/// quotient — so rows here remain comparable to that sweep's. What deepens is
/// the standing limitation both share: sixty-four back-to-back encodes hold the
/// device busier than one cold submission would, so this is a steady-state
/// per-plan cost and not a first-call latency.
const BATCH: usize = 64;

/// Byte width of one `f32`.
const F32_BYTES: u64 = 4;

/// Which predeclared matrix this invocation runs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RunMode {
    /// The retained seven-shape, two-strategy calibration.
    Calibration,
    /// The six-shape tree excursion, including timing.
    TreeWidthExcursion,
    /// The excursion's anchors and per-element oracle, without timing.
    VerifyTreeWidthExcursion,
}

impl RunMode {
    /// Parses the complete command-line vocabulary.
    fn from_args() -> Self {
        let arguments: Vec<String> = std::env::args().skip(1).collect();
        match arguments.as_slice() {
            [] => Self::Calibration,
            [argument] if argument == "--tree-width-excursion" => Self::TreeWidthExcursion,
            [argument] if argument == "--verify-tree-width-excursion" => {
                Self::VerifyTreeWidthExcursion
            }
            _ => panic!(
                "usage: reduction-partition-sweep [--tree-width-excursion|--verify-tree-width-excursion]"
            ),
        }
    }

    /// The stable result key printed in the retained record.
    const fn key(self) -> &'static str {
        match self {
            Self::Calibration => "reduction-partition-calibration",
            Self::TreeWidthExcursion | Self::VerifyTreeWidthExcursion => {
                "reduction-tree-width-excursion"
            }
        }
    }

    /// Whether this invocation records timing samples.
    const fn timed(self) -> bool {
        !matches!(self, Self::VerifyTreeWidthExcursion)
    }

    /// Whether this invocation measures only the tree strategy.
    const fn tree_only(self) -> bool {
        !matches!(self, Self::Calibration)
    }
}

fn main() {
    let mode = RunMode::from_args();
    let declaration = BoundMetalCompileDeclaration::first_macos_apple9()
        .expect("the authoritative macOS Apple9 declaration binds");
    let device = Device::system_default().expect("this host has a Metal device");
    // One queue for the whole sweep, created before any timing.
    let queue = device.new_command_queue();
    let toolchain = Toolchain::system();

    if mode.tree_only() {
        assert_eq!(
            std::env::var("DEVELOPER_DIR").as_deref(),
            Ok("/Applications/Xcode.app/Contents/Developer"),
            "the excursion must select the authority ledger's Xcode through DEVELOPER_DIR"
        );
        assert_eq!(
            device.name(),
            "Apple M4 Max",
            "the excursion is bounded to the authority ledger's exact execution device"
        );
        assert!(
            device.supports_family(MTLGPUFamily::Apple9),
            "the excursion requires a live device reporting Apple9 support"
        );
    }

    println!("# spike\t{}", mode.key());
    println!("# mode\t{mode:?}");
    println!("# metric\twall-clock microseconds, commit to completed");
    println!("# warmup\t{WARMUP}");
    println!("# repetitions\t{REPETITIONS}");
    println!("# batch\t{BATCH}");
    println!("# contract\tFLUSH_AND_REASSOCIATE_F32");
    println!("# declaration\tBoundMetalCompileDeclaration::first_macos_apple9");
    println!("# device\t{}", device.name());
    println!(
        "# device_apple9\t{}",
        device.supports_family(MTLGPUFamily::Apple9)
    );
    println!(
        "# device_max_threads_per_threadgroup\t{}",
        device.max_threads_per_threadgroup().width,
    );
    println!(
        "# device_max_threadgroup_memory\t{}",
        device.max_threadgroup_memory_length(),
    );
    println!("# load_before\t{}", load_average());

    tie_oracle();
    println!(
        "# oracle_tie\t{}x{}\treference-checked",
        ORACLE_TIE.0, ORACLE_TIE.1
    );

    if mode.timed() {
        println!(
            "rows\tcontributors\telements\tstrategy\tpartitions\tper_partition\tproduction\t\
             widest_workgroup\tthreadgroup_bytes\tencoders\treps\tbatch\tsubmit_min_us\t\
             submit_p50_us\tsubmit_p90_us\tsubmit_stddev_us\tbatch_min_us\tbatch_p50_us\t\
             batch_p90_us\tbatch_stddev_us\tamortized_min_us\tamortized_p50_us\t\
             amortized_stddev_us\tstatus"
        );
    }

    let mut attempted = 0_usize;
    let mut measured = 0_usize;
    let mut declined = 0_usize;
    let shapes: &[(u64, u64)] = if mode.tree_only() {
        &TREE_WIDTH_EXCURSION_SHAPES
    } else {
        &SHAPES
    };
    for &(rows, contributors) in shapes {
        let counts = measure_shape(
            &device,
            &queue,
            &toolchain,
            &declaration,
            Subject { rows, contributors },
            mode,
        );
        attempted += counts.0;
        measured += counts.1;
        declined += counts.2;
    }

    println!();
    println!("# shapes\t{}", shapes.len());
    println!("# variants_attempted\t{attempted}");
    println!(
        "# {}\t{measured}",
        if mode.timed() {
            "variants_measured"
        } else {
            "variants_verified"
        }
    );
    println!("# variants_declined\t{declined}");
    println!("# load_after\t{}", load_average());
}

/// Compiles, anchors, prepares, verifies, and times every partition of one shape.
///
/// Returns the attempted, measured, and declined variant counts, so the run can
/// state its own population rather than leaving a reader to count rows.
#[allow(
    clippy::too_many_lines,
    reason = "one shape must compile and identify production, anchor both strategies, allocate one shared input, prepare the full variant population, verify it, and only then time it; extracting those stateful phases would obscure their required order"
)]
fn measure_shape(
    device: &Device,
    queue: &CommandQueue,
    toolchain: &Toolchain,
    declaration: &BoundMetalCompileDeclaration,
    subject: Subject,
    mode: RunMode,
) -> (usize, usize, usize) {
    let Subject { rows, contributors } = subject;
    let elements = rows * contributors;
    let program = reduction_program(rows, contributors);
    let request = CompilerRequest::new(
        &program,
        NumericalContract::FLUSH_AND_REASSOCIATE_F32,
        TargetRequest::new([declaration.profile().clone()]).expect("a singleton target request"),
    );
    let compilation = compile(request)
        .expect("the batch compiles")
        .into_targets()
        .pop()
        .expect("one target outcome")
        .into_parts()
        .1
        .expect("the shape reaches a plan");

    let alternatives: Vec<_> = compilation.alternatives().collect();
    assert_eq!(
        alternatives.len(),
        3,
        "{rows}x{contributors} retained {} alternative(s); the predeclared matrix admits only \
         shapes retaining all three",
        alternatives.len(),
    );

    let split = alternatives
        .iter()
        .find(|alternative| stage_launches(alternative).len() >= 3)
        .expect("the split alternative is the one with three stages");
    let tree = alternatives
        .iter()
        .find(|alternative| {
            stage_launches(alternative).len() == 2
                && stage_launches(alternative)[1].threads_per_workgroup > 1
        })
        .expect("the tree alternative is the two-stage one wider than one thread");

    // The prologue is taken from the compiler's plan rather than rebuilt. It is
    // identical across all three strategies and across every partition, so
    // rebuilding it would have added a second transcription to check for no gain.
    let split_program = split.abi().kernel_program();
    let ordered: Vec<&VerifiedKernel> = split_program
        .execution_order()
        .map(tiler_ir::program::StageRef::kernel)
        .collect();
    let prologue = ordered[0];
    let prologue_launch = stage_launches(split)[0];
    // The numerical realization the compiler resolved for this request, read off
    // its own reduction kernel rather than reconstructed from the contract: the
    // contract's projection is `pub(crate)`, and a reconstruction would be a
    // third thing to keep in step with `physical.rs`.
    let numerical = ordered[1].numerical();

    let split_production = governed_partition(contributors)
        .expect("every swept contributor count admits a balanced exact split");
    let tree_participants = stage_launches(tree)[1].threads_per_workgroup;
    assert!(
        contributors.is_multiple_of(tree_participants),
        "{rows}x{contributors}: the compiler-published tree width {tree_participants} does not exactly partition the contributor sequence"
    );
    let tree_production = ContributorPartition {
        partitions: tree_participants,
        contributors_per_partition: contributors / tree_participants,
    };
    assert!(
        tree_production.covers(contributors),
        "{rows}x{contributors}: the compiler-published tree partition does not cover the contributor sequence"
    );

    anchor(
        subject,
        split_production,
        tree_production,
        numerical,
        declaration,
        split,
        tree,
        prologue,
    );

    // Two buffers are shared by every variant of the shape. The input is
    // read-only after one write. The prologue's materialized output is rewritten
    // in full by every submission — one invocation per element, unconditionally
    // — so no variant can read another's residue, and duplicating it per variant
    // would cost gigabytes at these shapes for nothing. **The program output is
    // deliberately not shared**: a variant that failed to write some position
    // would otherwise read the previous variant's correct answer and pass the
    // oracle.
    let operands =
        vec![1.0_f32; usize::try_from(elements).expect("the cell's elements fit a usize")];
    let input = device.new_buffer(elements * F32_BYTES, MTLResourceOptions::StorageModeShared);
    buffer::write_f32(&input, &operands);
    let mapped = device.new_buffer(elements * F32_BYTES, MTLResourceOptions::StorageModePrivate);

    let partitions = admissible_partitions(contributors);
    let mut prepared: Vec<Variant> = Vec::new();
    let mut attempted = 0_usize;
    let mut declines: Vec<(Strategy, ContributorPartition, String)> = Vec::new();
    let strategies: &[Strategy] = if mode.tree_only() {
        &[Strategy::Tree]
    } else {
        &[Strategy::Split, Strategy::Tree]
    };
    for &strategy in strategies {
        for partition in &partitions {
            attempted += 1;
            match prepare(
                device,
                toolchain,
                declaration,
                subject,
                strategy,
                *partition,
                numerical,
                prologue,
                prologue_launch,
                &input,
                &mapped,
            ) {
                Ok(variant) => prepared.push(variant),
                Err(reason) => declines.push((strategy, *partition, reason)),
            }
        }
    }

    #[allow(
        clippy::cast_precision_loss,
        reason = "the contributor count is at most 2^14 and f32 represents every integer below 2^24 exactly"
    )]
    let expected = contributors as f32;
    verify(queue, &prepared, expected, subject);
    if !mode.timed() {
        println!(
            "# verified\t{}x{}\t{} variant(s)",
            rows,
            contributors,
            prepared.len()
        );
        return (attempted, prepared.len(), declines.len());
    }
    warm(queue, &prepared);
    let single = timed_rounds(queue, &prepared, 1);
    let batched = timed_rounds(queue, &prepared, BATCH);

    report(
        subject,
        split_production,
        tree_production,
        &prepared,
        &single,
        &batched,
        &declines,
    );
    (attempted, prepared.len(), declines.len())
}

/// Writes one shape's measured rows and its declined ones.
///
/// **Declines are rows rather than omissions.** A participant count the prepared
/// entry will not admit is part of what a calibration reports — it is the edge of
/// the admissible range — and leaving it out would make the population look like
/// the range.
fn report(
    subject: Subject,
    split_production: ContributorPartition,
    tree_production: ContributorPartition,
    prepared: &[Variant],
    single: &[Vec<f64>],
    batched: &[Vec<f64>],
    declines: &[(Strategy, ContributorPartition, String)],
) {
    let Subject { rows, contributors } = subject;
    let elements = rows * contributors;
    for (index, variant) in prepared.iter().enumerate() {
        let submit_summary = Summary::of(&single[index]);
        let batch_summary = Summary::of(&batched[index]);
        let amortized = Amortized::of(&submit_summary, &batch_summary);
        println!(
            "{rows}\t{contributors}\t{elements}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{REPETITIONS}\t\
             {BATCH}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.4}\t{:.4}\t\
             {:.4}\tmeasured",
            variant.strategy.key(),
            variant.partition.partitions,
            variant.partition.contributors_per_partition,
            production_mark(
                variant.strategy,
                variant.partition,
                split_production,
                tree_production,
            ),
            variant.widest_workgroup,
            variant.threadgroup_bytes,
            variant.encoded.len(),
            submit_summary.min,
            submit_summary.p50,
            submit_summary.p90,
            submit_summary.stddev,
            batch_summary.min,
            batch_summary.p50,
            batch_summary.p90,
            batch_summary.stddev,
            amortized.min,
            amortized.p50,
            amortized.stddev,
        );
    }
    for (strategy, partition, reason) in declines {
        println!(
            "{rows}\t{contributors}\t{elements}\t{}\t{}\t{}\t{}\t-\t-\t-\t-\t-\t-\t-\t-\t\
             -\t-\t-\t-\t-\t-\t-\t-\tdeclined: {reason}",
            strategy.key(),
            partition.partitions,
            partition.contributors_per_partition,
            production_mark(*strategy, *partition, split_production, tree_production,),
        );
    }
}

/// Marks the one row of a shape that carries the partition the compiler chooses.
const fn production_mark(
    strategy: Strategy,
    partition: ContributorPartition,
    split_production: ContributorPartition,
    tree_production: ContributorPartition,
) -> &'static str {
    let production = match strategy {
        Strategy::Split => split_production,
        Strategy::Tree => tree_production,
    };
    if partition.partitions == production.partitions {
        "production"
    } else {
        "-"
    }
}

/// Requires the rebuilt plans at each production partition to *be* the compiler's.
///
/// Two independent equalities, both hard. The emitted translation unit must be
/// byte-identical, which covers the kernel bodies, the fold bounds, the declared
/// workgroup width, the staging declaration, and every numerical realization
/// decision the emitter makes. And the launch extents this spike derives must
/// equal the ones the compiler's ABI publishes, which covers the dispatch the
/// source cannot state.
///
/// **This is what makes the off-production rows evidence.** Without it the sweep
/// would be comparing the compiler's plan against a lookalike, and any measured
/// difference could be the transcription rather than the partition.
#[allow(
    clippy::too_many_arguments,
    reason = "the anchor compares two production partitions and alternatives against one shared subject, realization, declaration, and prologue; grouping them would only move this exact evidence tuple"
)]
fn anchor(
    subject: Subject,
    split_production: ContributorPartition,
    tree_production: ContributorPartition,
    numerical: NumericalRealization,
    declaration: &BoundMetalCompileDeclaration,
    split: &PlanAlternative<'_>,
    tree: &PlanAlternative<'_>,
    prologue: &VerifiedKernel,
) {
    let emit = |kernels: &[&VerifiedKernel]| {
        emit_translation_unit(kernels, declaration.metal_facts(), declaration.emission())
            .expect("the alternative emits")
            .source()
            .to_owned()
    };

    let split_stages = regions::split_stages(subject, split_production, numerical)
        .expect("the production split rebuilds");
    let rebuilt_split = emit(&[prologue, &split_stages[0].kernel, &split_stages[1].kernel]);
    let compiler_split = emit(&split.kernels().iter().collect::<Vec<_>>());
    assert!(
        rebuilt_split == compiler_split,
        "{}x{}: the rebuilt multi-pass split at the governed partition ({} x {}) does not emit \
         the source the compiler emits; the transcription in `regions.rs` has drifted from \
         `crates/tiler-compiler/src/physical.rs` and every off-governed row would be about a \
         different program",
        subject.rows,
        subject.contributors,
        split_production.partitions,
        split_production.contributors_per_partition,
    );

    let tree_stages = regions::tree_stages(subject, tree_production, numerical)
        .expect("the production tree rebuilds");
    let rebuilt_tree = emit(&[prologue, &tree_stages[0].kernel]);
    let compiler_tree = emit(&tree.kernels().iter().collect::<Vec<_>>());
    assert!(
        rebuilt_tree == compiler_tree,
        "{}x{}: the rebuilt single-workgroup tree at the production participant count ({}) does \
         not emit the source the compiler emits",
        subject.rows,
        subject.contributors,
        tree_production.partitions,
    );

    let published = stage_launches(split);
    let rebuilt: Vec<Launch> = split_stages
        .iter()
        .map(|stage| Launch {
            grid_threads: stage.launch.grid_threads,
            threads_per_workgroup: stage.launch.threads_per_workgroup,
        })
        .collect();
    assert_eq!(
        published[1..],
        rebuilt[..],
        "{}x{}: the compiler publishes split launch extents {:?} and this spike derives {:?}",
        subject.rows,
        subject.contributors,
        &published[1..],
        &rebuilt[..],
    );

    let published = stage_launches(tree);
    let rebuilt = Launch {
        grid_threads: tree_stages[0].launch.grid_threads,
        threads_per_workgroup: tree_stages[0].launch.threads_per_workgroup,
    };
    assert_eq!(
        published[1], rebuilt,
        "{}x{}: the compiler publishes tree launch extents {:?} and this spike derives {rebuilt:?}",
        subject.rows, subject.contributors, published[1],
    );
}

/// Which parallel strategy one variant realizes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Strategy {
    /// A three-stage program writing and then consuming materialized partials.
    Split,
    /// A two-stage program whose reduction workgroup holds every participant.
    Tree,
}

impl Strategy {
    /// The stable code naming this strategy in the recorded sweep.
    const fn key(self) -> &'static str {
        match self {
            Self::Split => "multi-pass-split",
            Self::Tree => "single-workgroup-tree",
        }
    }
}

/// The extents one stage is dispatched at.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Launch {
    /// Invocations along the grid axis.
    grid_threads: u64,
    /// Invocations in one workgroup.
    threads_per_workgroup: u64,
}

/// One partition of one strategy, compiled and allocated, ready to submit.
struct Variant {
    strategy: Strategy,
    partition: ContributorPartition,
    widest_workgroup: u64,
    threadgroup_bytes: u64,
    encoded: Vec<EncodedStage>,
    #[allow(
        dead_code,
        reason = "held for the lifetime of the shape so every bound buffer outlives every submission that reads it; the encoder reaches them through `EncodedStage::placements`"
    )]
    owned: Vec<Buffer>,
    output: Buffer,
    readback: usize,
}

/// One stage's pipeline, buffer placements, and launch extents.
struct EncodedStage {
    pipeline: ComputePipelineState,
    placements: Vec<(u64, Buffer)>,
    launch: Launch,
}

impl Variant {
    /// Submits this variant's whole plan `repeats` times and reads back.
    fn submit(&self, queue: &CommandQueue, repeats: usize) -> Result<Vec<f32>, &'static str> {
        submit(queue, &self.output, self.readback, |command_buffer| {
            for _ in 0..repeats {
                for stage in &self.encoded {
                    let encoder = command_buffer.new_compute_command_encoder();
                    encoder.set_compute_pipeline_state(&stage.pipeline);
                    for (index, buffer) in &stage.placements {
                        encoder.set_buffer(*index, Some(buffer), 0);
                    }
                    encoder.dispatch_threads(
                        MTLSize::new(stage.launch.grid_threads, 1, 1),
                        MTLSize::new(stage.launch.threads_per_workgroup, 1, 1),
                    );
                    encoder.end_encoding();
                }
            }
        })
    }
}

/// Rebuilds, emits, links, pipelines, and allocates one partition of one
/// strategy.
///
/// # Errors
///
/// Returns the reason this partition produced no dispatchable plan. Every one is
/// a fact about the partition on this profile rather than a harness fault, and
/// each is recorded as a declined row rather than aborting the shape: a
/// participant count the prepared pipeline will not admit is exactly the kind of
/// bound a calibration has to report, and asserting on it would have hidden the
/// edge of the admissible range instead of measuring up to it.
#[allow(
    clippy::too_many_arguments,
    reason = "every input is a distinct required part of one variant's preparation, and grouping them into a struct would only move the argument list"
)]
fn prepare(
    device: &Device,
    toolchain: &Toolchain,
    declaration: &BoundMetalCompileDeclaration,
    subject: Subject,
    strategy: Strategy,
    partition: ContributorPartition,
    numerical: NumericalRealization,
    prologue: &VerifiedKernel,
    prologue_launch: Launch,
    input: &Buffer,
    mapped: &Buffer,
) -> Result<Variant, String> {
    let stages = match strategy {
        Strategy::Split => regions::split_stages(subject, partition, numerical)?,
        Strategy::Tree => regions::tree_stages(subject, partition, numerical)?,
    };

    let mut kernels: Vec<&VerifiedKernel> = vec![prologue];
    kernels.extend(stages.iter().map(|stage| &stage.kernel));
    let unit = emit_translation_unit(&kernels, declaration.metal_facts(), declaration.emission())
        .map_err(|error| format!("the unit does not emit: {error:?}"))?;
    // Emission succeeds even when the target cannot honour the declared
    // contract, so conformance is asked explicitly rather than inferred.
    unit.require_declared_realization().map_err(|error| {
        format!("the emitted unit does not realize the declared numerics: {error:?}")
    })?;
    let compiled = toolchain
        .compile(&CompileRequest::new(
            unit.source(),
            declaration.aot_target(),
            OptimizationLevel::Default,
            declaration.numerical_realization(),
        ))
        .map_err(|error| format!("the offline toolchain does not link the unit: {error:?}"))?;

    let output = device.new_buffer(
        subject.rows * F32_BYTES,
        MTLResourceOptions::StorageModeShared,
    );
    let mut owned = vec![output.clone()];
    // The split stages one partial per partition per output position; the tree
    // stages its partials in workgroup memory and needs no device buffer.
    let partials = matches!(strategy, Strategy::Split).then(|| {
        let buffer = device.new_buffer(
            subject.rows * partition.partitions * F32_BYTES,
            MTLResourceOptions::StorageModePrivate,
        );
        owned.push(buffer.clone());
        buffer
    });

    // The buffer each stage binds, in the order that stage declares its accesses
    // — reads first, then its one owning write. That order is the region's own,
    // stated in `regions.rs`, and the argument-table index each one lands at is
    // read from the emitter's binding table rather than assumed.
    let bindings: Vec<Vec<&Buffer>> = match strategy {
        Strategy::Split => vec![
            vec![input, mapped],
            vec![mapped, partials.as_ref().expect("a split stages partials")],
            vec![partials.as_ref().expect("a split stages partials"), &output],
        ],
        Strategy::Tree => vec![vec![input, mapped], vec![mapped, &output]],
    };
    let launches: Vec<Launch> = std::iter::once(prologue_launch)
        .chain(stages.iter().map(|stage| Launch {
            grid_threads: stage.launch.grid_threads,
            threads_per_workgroup: stage.launch.threads_per_workgroup,
        }))
        .collect();

    let mut encoded = Vec::new();
    let mut widest_workgroup = 0_u64;
    let mut threadgroup_bytes = 0_u64;
    for (position, kernel) in kernels.iter().enumerate() {
        let stage = encode_stage(
            device,
            &unit,
            &compiled.metallib,
            kernel,
            launches[position],
            &bindings[position],
        )?;
        widest_workgroup = widest_workgroup.max(stage.launch.threads_per_workgroup);
        threadgroup_bytes = threadgroup_bytes.max(stage.reserved);
        encoded.push(stage.encoded);
    }

    let readback = usize::try_from(subject.rows).expect("the output element count fits a usize");
    Ok(Variant {
        strategy,
        partition,
        widest_workgroup,
        threadgroup_bytes,
        encoded,
        owned,
        output,
        readback,
    })
}

/// One stage's pipeline and bindings, with the threadgroup memory it reserved.
struct PreparedStage {
    encoded: EncodedStage,
    launch: Launch,
    /// Threadgroup bytes the prepared entry statically reserves.
    reserved: u64,
}

/// Builds one stage's pipeline and binds its buffers, or states the bound it hit.
///
/// **Both refusals here are the profile's, not this spike's.** The authoritative
/// Apple9 declaration fills its max-threads-per-workgroup row with a
/// prepared-entry *query* rather than a literal, so the pipeline's own
/// `maxTotalThreadsPerThreadgroup` is the declared bound and comparing against it
/// is asking the profile. The threadgroup-memory check is the device's own
/// reported limit. Both are returned rather than asserted so the sweep can record
/// the edge of the admissible participant range instead of stopping at it.
fn encode_stage(
    device: &Device,
    unit: &MetalTranslationUnit,
    object: &[u8],
    kernel: &VerifiedKernel,
    launch: Launch,
    buffers: &[&Buffer],
) -> Result<PreparedStage, String> {
    let identity = kernel.canonical_identity();
    let emitted = unit
        .entry_points()
        .iter()
        .find(|entry| entry.kernel_identity() == identity)
        .ok_or_else(|| "an emitted entry point is missing for a stage".to_owned())?;
    let pipeline = pipeline_for(device, object, emitted.symbol())
        .map_err(|error| format!("the pipeline does not build: {error}"))?;

    if launch.threads_per_workgroup > pipeline.max_total_threads_per_threadgroup() {
        return Err(format!(
            "the plan declares {} threads per workgroup and the prepared entry admits {}",
            launch.threads_per_workgroup,
            pipeline.max_total_threads_per_threadgroup(),
        ));
    }
    let reserved = pipeline.static_threadgroup_memory_length();
    if reserved > device.max_threadgroup_memory_length() {
        return Err(format!(
            "the entry reserves {reserved} byte(s) of threadgroup memory and this device admits {}",
            device.max_threadgroup_memory_length(),
        ));
    }

    // The argument-table index each buffer lands at is read from the emitter's
    // binding table; only the *order* is this spike's, and that order is the
    // region's own declared access order.
    let table = emitted.buffers();
    let mut placements = Vec::new();
    for (slot, buffer) in buffers.iter().enumerate() {
        let binding = table
            .get(slot)
            .ok_or_else(|| "a declared access has no emitted binding".to_owned())?;
        placements.push((u64::from(binding.index()), (*buffer).clone()));
    }
    Ok(PreparedStage {
        encoded: EncodedStage {
            pipeline,
            placements,
            launch,
        },
        launch,
        reserved,
    })
}

/// Checks every variant against the oracle.
///
/// **Correctness before timing, always.** A partition that returns the wrong bits
/// has no meaningful cost, and timing it would put a number on a plan that does
/// not compute the program.
fn verify(queue: &CommandQueue, prepared: &[Variant], expected: f32, subject: Subject) {
    for variant in prepared {
        let readback = variant
            .submit(queue, 1)
            .expect("the verification submission completes");
        for (index, value) in readback.iter().enumerate() {
            assert!(
                value.to_bits() == expected.to_bits(),
                "{}x{} {} at {} x {}: output[{index}] is {value} ({:08x}), expected {expected} \
                 ({:08x})",
                subject.rows,
                subject.contributors,
                variant.strategy.key(),
                variant.partition.partitions,
                variant.partition.contributors_per_partition,
                value.to_bits(),
                expected.to_bits(),
            );
        }
    }
}

/// Warms every verified variant at both encode counts.
fn warm(queue: &CommandQueue, prepared: &[Variant]) {
    for variant in prepared {
        for _ in 0..WARMUP {
            variant
                .submit(queue, 1)
                .expect("the warm-up submission completes");
            variant
                .submit(queue, BATCH)
                .expect("the warm-up submission completes");
        }
    }
}

/// Times `REPETITIONS` interleaved rounds at one encode count.
///
/// Each round submits every variant once and the round's starting variant
/// rotates, so a drift over the shape lands on every partition and both
/// strategies alike rather than on whichever happened to run last.
fn timed_rounds(queue: &CommandQueue, prepared: &[Variant], repeats: usize) -> Vec<Vec<f64>> {
    let mut samples: Vec<Vec<f64>> = vec![Vec::with_capacity(REPETITIONS); prepared.len()];
    for round in 0..REPETITIONS {
        for offset in 0..prepared.len() {
            let index = (round + offset) % prepared.len();
            let started = Instant::now();
            let outcome = prepared[index].submit(queue, repeats);
            let elapsed = started.elapsed();
            outcome.expect("the timed submission completes");
            samples[index].push(elapsed.as_secs_f64() * 1e6);
        }
    }
    samples
}

/// The per-plan cost left once the submission round trip is cancelled.
///
/// `(batched - single) / (BATCH - 1)`. The spread is added rather than
/// propagated in quadrature: the two samples are two submissions on one machine
/// whose noise is common-mode, so the conservative bound is what a reader should
/// be handed when deciding whether a partition's advantage survives its noise.
/// It can come out negative, and it is reported rather than clamped.
struct Amortized {
    min: f64,
    p50: f64,
    stddev: f64,
}

impl Amortized {
    /// Derives the per-plan cost from the two encode counts' summaries.
    fn of(single: &Summary, batched: &Summary) -> Self {
        #[allow(
            clippy::cast_precision_loss,
            reason = "BATCH is 64 and f64 represents every small integer exactly"
        )]
        let divisor = (BATCH - 1) as f64;
        Self {
            min: (batched.min - single.min) / divisor,
            p50: (batched.p50 - single.p50) / divisor,
            stddev: (batched.stddev + single.stddev) / divisor,
        }
    }
}

/// The distribution of one variant's timed samples.
struct Summary {
    min: f64,
    p50: f64,
    p90: f64,
    stddev: f64,
}

impl Summary {
    /// Summarizes one sample set, which must be non-empty.
    fn of(samples: &[f64]) -> Self {
        let mut sorted = samples.to_vec();
        sorted.sort_by(f64::total_cmp);
        #[allow(
            clippy::cast_precision_loss,
            reason = "the sample count is REPETITIONS, far below 2^53"
        )]
        let count = sorted.len() as f64;
        let mean = sorted.iter().sum::<f64>() / count;
        let variance = sorted
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / (count - 1.0).max(1.0);
        Self {
            min: sorted[0],
            p50: percentile(&sorted, 0.50),
            p90: percentile(&sorted, 0.90),
            stddev: variance.sqrt(),
        }
    }
}

/// The nearest-rank percentile of an ascending sample set.
fn percentile(sorted: &[f64], fraction: f64) -> f64 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "the sample count is REPETITIONS, far below 2^53"
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

/// The launch extents one alternative's ABI publishes, in execution order.
fn stage_launches(alternative: &PlanAlternative<'_>) -> Vec<Launch> {
    let program = alternative.abi().kernel_program();
    let expressions = program.abi_expressions();
    program
        .execution_order()
        .map(|stage| {
            let launch = stage.launch();
            Launch {
                grid_threads: literal_extent(expressions, launch.grid_threads),
                threads_per_workgroup: literal_extent(expressions, launch.threads_per_workgroup),
            }
        })
        .collect()
}

/// Resolves one ABI arena position to the unsigned literal it must be.
fn literal_extent(expressions: &[ExprNode], position: u32) -> u64 {
    let index = usize::try_from(position).expect("an arena position fits a usize");
    match expressions.get(index) {
        Some(ExprNode::Root(AbiRoot::UnsignedLiteral(value))) => *value,
        other => panic!("the launch extent at {position} is not a declared literal: {other:?}"),
    }
}

/// Builds a compute pipeline for one named function of one object image.
fn pipeline_for(
    device: &Device,
    object: &[u8],
    symbol: &str,
) -> Result<ComputePipelineState, String> {
    let library = device
        .new_library_with_data(object)
        .map_err(|error| format!("the linked object does not load: {error}"))?;
    let function = library
        .get_function(symbol, None)
        .map_err(|error| format!("the emitted symbol does not resolve: {error}"))?;
    let descriptor = ComputePipelineDescriptor::new();
    descriptor.set_compute_function(Some(&function));
    device
        .new_compute_pipeline_state(&descriptor)
        .map_err(|error| format!("the pipeline state does not build: {error}"))
}

/// Submits one encoded command buffer and reads the output back.
///
/// The command buffer's terminal state is checked *before* the host reads
/// anything, and the accepted state is exactly `Completed`.
fn submit(
    queue: &CommandQueue,
    output: &Buffer,
    count: usize,
    encode: impl FnOnce(&CommandBufferRef),
) -> Result<Vec<f32>, &'static str> {
    let command_buffer = queue.new_command_buffer();
    encode(command_buffer);
    command_buffer.commit();
    command_buffer.wait_until_completed();
    match command_buffer.status() {
        MTLCommandBufferStatus::Completed => Ok(buffer::read_f32(output, count)),
        MTLCommandBufferStatus::Error => Err("the device reported an execution error"),
        MTLCommandBufferStatus::NotEnqueued => Err("the command buffer was never enqueued"),
        MTLCommandBufferStatus::Enqueued => Err("the wait returned with the buffer enqueued"),
        MTLCommandBufferStatus::Committed => Err("the wait returned with the buffer committed"),
        MTLCommandBufferStatus::Scheduled => Err("the wait returned with the buffer scheduled"),
    }
}

/// Ties the closed-form expected value to `tiler-reference`'s own evaluation.
///
/// Run once, before any shape is measured. A row of `n` unit operands sums to
/// `n` under every grouping, exactly, for every `n` below 2^24 — and that is an
/// assertion about the arithmetic which this check makes falsifiable rather than
/// asserted beside the timing.
fn tie_oracle() {
    let (rows, contributors) = ORACLE_TIE;
    let program = reduction_program(rows, contributors);
    let operands =
        vec![1.0_f32; usize::try_from(rows * contributors).expect("the tie shape fits a usize")];
    #[allow(
        clippy::cast_precision_loss,
        reason = "the tie contributor count is sixteen"
    )]
    let closed_form = contributors as f32;
    let key = InputKey::new("input").expect("the input key is valid");
    let tensor = Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([rows, contributors]),
        operands
            .iter()
            .map(|value| {
                ReferenceElement::from_float_bits(
                    value.to_bits().to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .expect("the operand is a valid f32 pattern")
            })
            .collect(),
    )
    .expect("the input tensor is well formed");
    let outputs = ReferenceEvaluator::standard()
        .expect("the governed reference profile composes")
        .evaluate(&program, &[InputBinding::new(&key, &tensor)])
        .expect("the reference evaluates the program");
    let TensorPayloadView::Dense(elements) = outputs[0].payload() else {
        panic!("expected a dense f32 reference output");
    };
    assert!(
        !elements.is_empty(),
        "the oracle tie evaluated no output elements, so it proved nothing"
    );
    for element in elements {
        let bits = u32::from_be_bytes(
            <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
        );
        assert!(
            f32::from_bits(bits).to_bits() == closed_form.to_bits(),
            "the oracle answered {} where the closed form says {closed_form}",
            f32::from_bits(bits),
        );
    }
}

/// Builds a `rows x contributors` multiply-add prologue feeding a trailing sum.
///
/// The same program family the retained dispatch sweep compiles, so a cell here
/// and a cell there describe the same plans. The prologue is kept because it is
/// what makes the multi-pass split expressible at all: the split divides the
/// *materialized* reduction, so a bare sum with no prologue is a different
/// program.
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

/// The host's one-, five-, and fifteen-minute load averages.
fn load_average() -> String {
    std::process::Command::new("/usr/sbin/sysctl")
        .args(["-n", "vm.loadavg"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map_or_else(
            || "unavailable".to_owned(),
            |value| value.trim().replace(['{', '}'], "").trim().to_owned(),
        )
}

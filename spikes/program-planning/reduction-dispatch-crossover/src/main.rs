//! Times the three retained reduction strategies on the qualified Metal host.
//!
//! `spikes/program-planning/reduction-crossover` beside this one answers *where
//! a crossover could be measured* — over which shapes the authoritative Apple
//! profile retains all three alternatives at once. Against the measured
//! grid-axis row that domain is wide, so this spike answers the next question
//! and the one `calibrate-and-activate-parallel-reduction-selection` actually
//! owns: **at which shapes does the fastest strategy change, and can one
//! analytical model predict where?**
//!
//! It dispatches. For every cell of a predeclared shape matrix it compiles the
//! same program family the compile-only sweep uses, emits MSL for each retained
//! alternative, links it with `xcrun`, builds the pipelines, allocates the
//! program's own buffers, and submits the whole plan — every stage, in order,
//! in one command buffer — repeatedly, recording the wall-clock distribution.
//!
//! # What is measured, and what that number is not
//!
//! One sample is the wall clock across `commit()` and `wait_until_completed()`
//! for one submission of one plan. That is what a consumer pays to run the plan
//! once, and it is the quantity a selection decision is about.
//!
//! **It includes host submission cost and cannot separate it out.** `metal`
//! 0.33.0 exposes `commit`, `status`, `wait_until_completed`, and the handler
//! registrations, and no accessor for `MTLCommandBuffer`'s `GPUStartTime` or
//! `GPUEndTime`; reading those would need an `unsafe` `msg_send!`, and a new
//! unsafe site is a decision under ADR 0079 rather than a convenience a spike
//! may take. So the recorded time is end-to-end and every strategy pays the
//! same submission once — which is fair between them and is *not* a GPU-busy
//! measurement. Nothing here should be quoted as one.
//!
//! # Noise controls, stated because the metric is wall clock
//!
//! - Every alternative of a cell is fully prepared — emitted, linked, pipelined,
//!   allocated, input written — before any timing starts. No compilation,
//!   allocation, or host copy happens inside a timed region.
//! - [`WARMUP`] untimed submissions per alternative precede the timed ones, so
//!   no sample carries first-touch page faults or a cold pipeline.
//! - The timed submissions are **interleaved**: each round submits every
//!   alternative once, and the round's starting alternative rotates. A thermal
//!   or scheduling drift over the cell therefore lands on all three strategies
//!   alike instead of on whichever ran last.
//! - The distribution is reported rather than a single number: minimum, median,
//!   p90, mean, and sample standard deviation over [`REPETITIONS`] rounds. The
//!   minimum is the least contaminated estimate and the spread is what says
//!   whether a reported difference survives the noise.
//! - The host's one-, five-, and fifteen-minute load averages are recorded
//!   before and after the sweep, because a wall-clock claim taken on a loaded
//!   machine is not a claim about the device.
//!
//! # The oracle, and what agreement here does and does not prove
//!
//! Every operand is `1.0`, so the declared sum of a row is exactly the
//! contributor count, which is representable in `f32` for every count this
//! matrix reaches. **Every grouping of that row therefore produces the same
//! bits**, which is what makes one expected value valid for three strategies
//! under a contract that *permits* regrouping. A dropped, double-counted, or
//! unsynchronized contributor changes the sum and is caught.
//!
//! That closed form is checked against `tiler-reference`'s independent
//! evaluation of the same semantic program on every cell small enough to
//! evaluate on the host (see [`ORACLE_ELEMENT_LIMIT`]), so the constant is
//! tied to the oracle rather than asserted beside it. **Regrouped rounding is
//! not observed and is not claimed**: unit operands cannot expose it, and
//! `drive-a-grouping-sensitive-numerical-case-through-the-parallel-reduction-strategies`
//! owns that evidence. A timing run is not the place to acquire it.
//!
//! # Running it
//!
//! ```sh
//! cd spikes/program-planning/reduction-dispatch-crossover
//! DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer \
//!   cargo run --release --bin reduction-dispatch-sweep > results/<date>-<host>/sweep.tsv
//! ```
//!
//! `DEVELOPER_DIR` selects the offline toolchain the authority ledger's
//! compilation-environment row names. Without it a host whose default selection
//! is a newer Xcode links through a compiler the profile was not measured under,
//! which is a different environment and makes the run unqualified.
//!
//! No `make` target reaches here, per `spikes/README.md`.

mod buffer;
// A bin target is its own crate root, so this module is compiled twice, once per
// binary, and each sees only what its own binary calls. This one drives the
// stage model and the strategy classifier; `reduction-cost-fit` drives the
// parameters and the prediction. Neither set is dead — they are the two halves
// of one shared model — so the unused half is allowed here rather than split
// into two files that could drift apart.
#[allow(
    dead_code,
    reason = "this binary consumes the stage model and the strategy classifier; the fit binary consumes the parameters and the prediction. One shared module, two crate roots."
)]
mod model;

use std::time::Instant;

use metal::{
    Buffer, CommandBufferRef, CommandQueue, ComputePipelineDescriptor, ComputePipelineState,
    Device, MTLCommandBufferStatus, MTLResourceOptions, MTLSize,
};
use model::{Stage, Strategy};
use tiler_build::BoundMetalCompileDeclaration;
use tiler_compiler::session::{
    CompileRequest as CompilerRequest, NumericalContract, PlanAlternative, compile,
};
use tiler_compiler::target::TargetRequest;
use tiler_ir::program::abi::{AbiRoot, ExprNode};
use tiler_ir::program::{ValueRole, VerifiedKernelProgram};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};
use tiler_metal::emit::emit_translation_unit;
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::input::{CompileRequest, OptimizationLevel};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

/// Contributor counts swept along the reduced axis.
///
/// Every one admits `governed_partition`'s balanced exact split, because a
/// count that does not retains only the serial fold and contributes no
/// comparison. They are deliberately **not** all perfect squares: on a square
/// count the split's partition count and its contributors-per-partition are
/// equal, so a model fitted only to squares cannot be told apart from one that
/// memorized the square root. The four non-square counts — 32, 128, 2,048, and
/// 8,192 — split into `(8, 4)`, `(16, 8)`, `(64, 32)` and `(128, 64)`, and they
/// are the held-out set the fit is scored on.
const CONTRIBUTORS: [u64; 11] = [4, 16, 32, 64, 128, 256, 1024, 2048, 4096, 8192, 16384];

/// Row counts swept across the retained axis.
///
/// The row count is the program's independent parallelism: every row's fold is
/// independent of every other, so this axis is what decides whether the device
/// is already saturated before a strategy splits anything. It spans from one
/// row — where the serial fold's reduction stage launches a single invocation —
/// to a quarter of a million, where it launches more than the device can hold.
/// A crossover, if there is one, has to be somewhere in that span.
const ROWS: [u64; 10] = [1, 4, 16, 64, 256, 1024, 4096, 16384, 65536, 262_144];

/// Largest `rows x contributors` element count a cell may reach.
///
/// 2^24 elements is 64 MiB of `f32` input, which bounds the sweep's allocation
/// and its running time without truncating either axis: every row count is
/// reached by some contributor count and the reverse. It is a budget, not a
/// capability edge — the profile's grid-axis row admits 268,435,456, sixteen
/// times more.
const MAX_ELEMENTS: u64 = 1 << 24;

/// Cells at or below this element count are cross-checked against the oracle.
///
/// `tiler-reference` evaluates on the host through a boxed element vocabulary,
/// so evaluating every cell would dominate the run. The check it performs is
/// that the closed-form expected value *is* the oracle's answer, which is a
/// statement about the arithmetic rather than about the shape, so establishing
/// it on the small cells establishes it. Cells above the limit still compare
/// every output element against that closed form.
const ORACLE_ELEMENT_LIMIT: u64 = 4096;

/// Untimed submissions per alternative before any sample is taken.
const WARMUP: usize = 8;

/// Timed rounds per cell; each round submits every alternative once, at each
/// of the two encode counts.
const REPETITIONS: usize = 30;

/// How many times the batched submission encodes the same plan.
///
/// **This is the measurement's whole answer to a floor it cannot remove.** One
/// `commit` and `wait_until_completed` round trip costs about 180 microseconds
/// on this host before any kernel runs, which is more than every small cell's
/// arithmetic put together; a difference between strategies at those shapes
/// would be invisible under it. Encoding the plan `BATCH` times into one
/// command buffer pays that floor once and the plan `BATCH` times, so
/// subtracting the single-encode submission and dividing by `BATCH - 1` leaves
/// the per-plan cost with the floor cancelled.
///
/// Metal orders compute encoders within a command buffer unconditionally, so
/// the repeats run one after another rather than overlapping. Each repeat reads
/// the same input and writes the same output, which is idempotent for every
/// strategy here.
const BATCH: usize = 16;

/// Byte width of one `f32`.
const F32_BYTES: u64 = 4;

fn main() {
    let declaration = BoundMetalCompileDeclaration::first_macos_apple9()
        .expect("the authoritative macOS Apple9 declaration binds");
    let device = Device::system_default().expect("this host has a Metal device");
    // One queue for the whole sweep, created before any timing. A queue built
    // inside the timed region would put its construction into every sample, and
    // it is the single largest fixed cost on this path — measured at roughly
    // 200 microseconds, which is more than most cells' whole submission.
    let queue = device.new_command_queue();
    let toolchain = Toolchain::system();

    println!("# spike\treduction-dispatch-crossover");
    println!("# metric\twall-clock microseconds, commit to completed");
    println!("# warmup\t{WARMUP}");
    println!("# repetitions\t{REPETITIONS}");
    println!("# batch\t{BATCH}");
    println!("# contract\tFLUSH_AND_REASSOCIATE_F32");
    println!("# declaration\tBoundMetalCompileDeclaration::first_macos_apple9");
    println!("# device\t{}", device.name());
    println!(
        "# device_max_threads_per_threadgroup\t{}",
        device.max_threads_per_threadgroup().width,
    );
    println!(
        "# device_max_threadgroup_memory\t{}",
        device.max_threadgroup_memory_length(),
    );
    println!("# load_before\t{}", load_average());

    println!(
        "rows\tcontributors\telements\tpartitions\tper_partition\tstrategy\tencoders\t\
         widest_workgroup\tthreadgroup_bytes\tstages\treps\tbatch\tsubmit_min_us\t\
         submit_p50_us\tsubmit_p90_us\tsubmit_stddev_us\tbatch_min_us\tbatch_p50_us\t\
         batch_p90_us\tbatch_stddev_us\tamortized_min_us\tamortized_p50_us\t\
         amortized_stddev_us\toracle"
    );

    let mut cells = 0_usize;
    for rows in ROWS {
        for contributors in CONTRIBUTORS {
            let elements = rows * contributors;
            if elements > MAX_ELEMENTS {
                continue;
            }
            cells += 1;
            measure_cell(
                &device,
                &queue,
                &toolchain,
                &declaration,
                rows,
                contributors,
            );
        }
    }

    println!();
    println!("# cells\t{cells}");
    println!("# load_after\t{}", load_average());
}

/// Compiles, verifies, and times every retained alternative of one shape.
fn measure_cell(
    device: &Device,
    queue: &CommandQueue,
    toolchain: &Toolchain,
    declaration: &BoundMetalCompileDeclaration,
    rows: u64,
    contributors: u64,
) {
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

    let (partitions, per_partition) = model::governed_partition(contributors)
        .expect("every swept contributor count admits a balanced exact split");
    let operands =
        vec![1.0_f32; usize::try_from(elements).expect("the cell's elements fit a usize")];
    let expected = expected_row_sum(&program, &operands, rows, contributors);

    let mut prepared: Vec<Prepared> = alternatives
        .into_iter()
        .map(|alternative| {
            prepare(
                device,
                toolchain,
                declaration,
                alternative,
                &operands,
                rows,
                contributors,
            )
        })
        .collect();
    prepared.sort_by_key(|entry| entry.strategy);

    verify_and_warm(queue, &prepared, expected, rows, contributors);
    let single = timed_rounds(queue, &prepared, 1);
    let batched = timed_rounds(queue, &prepared, BATCH);

    let oracle = if elements <= ORACLE_ELEMENT_LIMIT {
        "reference-checked"
    } else {
        "closed-form"
    };
    for (index, entry) in prepared.iter().enumerate() {
        let submit_summary = Summary::of(&single[index]);
        let batch_summary = Summary::of(&batched[index]);
        let amortized = Amortized::of(&submit_summary, &batch_summary);
        let stages = entry
            .stages
            .iter()
            .map(|stage| format!("{}:{}:{}", stage.threads, stage.work, stage.depth))
            .collect::<Vec<_>>()
            .join("|");
        println!(
            "{rows}\t{contributors}\t{elements}\t{partitions}\t{per_partition}\t{}\t{}\t{}\t{}\t\
             {stages}\t{REPETITIONS}\t{BATCH}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t\
             {:.3}\t{:.3}\t{:.4}\t{:.4}\t{:.4}\t{oracle}",
            entry.strategy.key(),
            entry.encoders,
            entry.widest_workgroup,
            entry.threadgroup_bytes,
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
}

/// Checks every alternative against the oracle, then warms every one of them.
///
/// **Correctness before timing, always.** A strategy that returns the wrong bits
/// has no meaningful cost, and timing it would put a number on a plan that does
/// not compute the program. The warm-up follows rather than precedes it, at both
/// encode counts, so no sample carries a first-touch page fault or a cold
/// pipeline.
fn verify_and_warm(
    queue: &CommandQueue,
    prepared: &[Prepared],
    expected: f32,
    rows: u64,
    contributors: u64,
) {
    for entry in prepared {
        let readback = entry
            .submit(queue, 1)
            .expect("the verification submission completes");
        for (index, value) in readback.iter().enumerate() {
            assert!(
                value.to_bits() == expected.to_bits(),
                "{rows}x{contributors} {}: output[{index}] is {value} ({:08x}), expected \
                 {expected} ({:08x})",
                entry.strategy.key(),
                value.to_bits(),
                expected.to_bits(),
            );
        }
    }
    for entry in prepared {
        for _ in 0..WARMUP {
            entry
                .submit(queue, 1)
                .expect("the warm-up submission completes");
            entry
                .submit(queue, BATCH)
                .expect("the warm-up submission completes");
        }
    }
}

/// Times `REPETITIONS` interleaved rounds at one encode count.
///
/// Each round submits every alternative once and the round's starting
/// alternative rotates, so a thermal or scheduling drift over the cell lands on
/// all three strategies alike rather than on whichever happened to run last.
fn timed_rounds(queue: &CommandQueue, prepared: &[Prepared], repeats: usize) -> Vec<Vec<f64>> {
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
/// `(batched - single) / (BATCH - 1)`. The two submissions differ by exactly
/// `BATCH - 1` extra encodes of the same plan and by nothing else — same
/// pipelines, same buffers, same queue, one `commit` and one wait each — so the
/// difference is those encodes and the fixed cost divides out.
///
/// **The spread is added rather than propagated in quadrature.** The two
/// samples are not independent draws from one distribution; they are two
/// different submissions on one machine whose noise is common-mode, so the
/// conservative bound is what a reader should be handed when deciding whether a
/// cell's verdict survives its noise.
///
/// **It can come out negative**, and it is reported rather than clamped: that
/// is what a cell whose plan cost is far below the round trip's own variance
/// looks like, and clamping it to zero would present noise as a measurement of
/// something small.
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
            reason = "BATCH is 16 and f64 represents every small integer exactly"
        )]
        let divisor = (BATCH - 1) as f64;
        Self {
            min: (batched.min - single.min) / divisor,
            p50: (batched.p50 - single.p50) / divisor,
            stddev: (batched.stddev + single.stddev) / divisor,
        }
    }
}

/// The distribution of one alternative's timed samples.
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

/// One alternative, compiled and allocated, ready to submit repeatedly.
///
/// Holding the buffers and the pipelines together for the whole cell is what
/// keeps the timed region free of allocation, and it is also the retention the
/// dispatch owes: a split's intermediate is referenced by the stage that writes
/// it and the stage that reads it, and dropping either view would leave an
/// encoder bound to freed memory.
struct Prepared {
    strategy: Strategy,
    encoders: usize,
    widest_workgroup: u64,
    threadgroup_bytes: u64,
    stages: Vec<Stage>,
    encoded: Vec<EncodedStage>,
    #[allow(
        dead_code,
        reason = "held for the lifetime of the cell so every bound buffer outlives every submission that reads it; the encoder reaches them through `EncodedStage::placements`"
    )]
    buffers: Vec<Buffer>,
    output: Buffer,
    readback: usize,
}

/// One stage's pipeline, buffer placements, and launch extents.
struct EncodedStage {
    pipeline: ComputePipelineState,
    placements: Vec<(u64, Buffer, u64)>,
    grid_threads: u64,
    threads_per_workgroup: u64,
}

impl Prepared {
    /// Submits this alternative's whole plan `repeats` times and reads back.
    ///
    /// One encoder per stage, which is the ordering guarantee: Metal orders
    /// encoders within a command buffer unconditionally, so a combining stage
    /// never overlaps the partial stage whose output it reads — and, at
    /// `repeats > 1`, so one repeat never overlaps the next.
    fn submit(&self, queue: &CommandQueue, repeats: usize) -> Result<Vec<f32>, &'static str> {
        submit(queue, &self.output, self.readback, |command_buffer| {
            for _ in 0..repeats {
                for stage in &self.encoded {
                    let encoder = command_buffer.new_compute_command_encoder();
                    encoder.set_compute_pipeline_state(&stage.pipeline);
                    for (index, buffer, offset) in &stage.placements {
                        encoder.set_buffer(*index, Some(buffer), *offset);
                    }
                    encoder.dispatch_threads(
                        MTLSize::new(stage.grid_threads, 1, 1),
                        MTLSize::new(stage.threads_per_workgroup, 1, 1),
                    );
                    encoder.end_encoding();
                }
            }
        })
    }
}

/// Emits, links, and allocates one retained alternative.
///
/// Every dispatch parameter is read from the compiler's own record: the
/// argument-table index of each buffer from the emitter's binding table, the
/// byte window from the program's own view, and both launch extents from the
/// ABI arena. Nothing about the topology is assumed, which is why one function
/// prepares the fold, the split, and the tree unchanged.
fn prepare(
    device: &Device,
    toolchain: &Toolchain,
    declaration: &BoundMetalCompileDeclaration,
    alternative: PlanAlternative<'_>,
    operands: &[f32],
    rows: u64,
    contributors: u64,
) -> Prepared {
    let kernels: Vec<_> = alternative.kernels().iter().collect();
    let unit = emit_translation_unit(&kernels, declaration.metal_facts(), declaration.emission())
        .expect("the alternative emits");
    // Emission succeeds even when the target cannot honour the declared
    // contract, so conformance is asked explicitly rather than inferred.
    unit.require_declared_realization()
        .expect("the emitted unit realizes the declared numerics");
    let compiled = toolchain
        .compile(&CompileRequest::new(
            unit.source(),
            declaration.aot_target(),
            OptimizationLevel::Default,
            declaration.numerical_realization(),
        ))
        .expect("the offline toolchain links the unit");

    let program = alternative.abi().kernel_program();
    let expressions = program.abi_expressions();
    let (buffers, output, readback) = allocate(device, program, operands);

    let mut encoded = Vec::new();
    let mut stages = Vec::new();
    let mut widest_workgroup = 0_u64;
    let mut threadgroup_bytes = 0_u64;
    for stage in program.execution_order() {
        let identity = stage.kernel().canonical_identity();
        let emitted = unit
            .entry_points()
            .iter()
            .find(|entry| entry.kernel_identity() == identity)
            .expect("every dispatched stage has an emitted entry point");
        let pipeline = pipeline_for(device, &compiled.metallib, emitted.symbol());

        let launch = stage.launch();
        let grid_threads = literal_extent(expressions, launch.grid_threads);
        let threads_per_workgroup = literal_extent(expressions, launch.threads_per_workgroup);
        assert!(grid_threads > 0, "a stage launched zero threads");
        assert!(
            threads_per_workgroup <= pipeline.max_total_threads_per_threadgroup(),
            "the plan declares {threads_per_workgroup} threads per workgroup and \"{}\" admits {}",
            emitted.symbol(),
            pipeline.max_total_threads_per_threadgroup(),
        );
        let reserved = pipeline.static_threadgroup_memory_length();
        assert!(
            reserved <= device.max_threadgroup_memory_length(),
            "\"{}\" reserves {reserved} byte(s) of threadgroup memory and this device admits {}",
            emitted.symbol(),
            device.max_threadgroup_memory_length(),
        );
        widest_workgroup = widest_workgroup.max(threads_per_workgroup);
        threadgroup_bytes = threadgroup_bytes.max(reserved);

        let bindings = emitted.buffers();
        let mut placements = Vec::new();
        for (position, access) in stage.accesses().enumerate() {
            let binding = bindings
                .get(position)
                .expect("every access has an emitted binding");
            let view = access.view();
            let slot = program
                .allocations()
                .position(|candidate| candidate == view.value().allocation())
                .expect("every accessed value's allocation is one this program declares");
            placements.push((
                u64::from(binding.index()),
                buffers[slot].clone(),
                view.window().offset,
            ));
        }
        stages.push(grid_threads);
        encoded.push(EncodedStage {
            pipeline,
            placements,
            grid_threads,
            threads_per_workgroup,
        });
    }

    let strategy = Strategy::classify(encoded.len(), widest_workgroup);
    Prepared {
        strategy,
        encoders: encoded.len(),
        widest_workgroup,
        threadgroup_bytes,
        stages: checked_stages(strategy, rows, contributors, &stages),
        encoded,
        buffers,
        output,
        readback,
    }
}

/// Pairs this spike's stage model against the launch geometry the plan published.
///
/// The depths the cost model consumes cannot be read off a plan — the compiler
/// publishes launch extents, never how many fold steps one invocation performs
/// — so they are derived from the documented topology of each strategy. That
/// derivation is only trustworthy while the topology it names is the one being
/// dispatched, so the **thread counts**, which *are* published, are compared
/// element by element. A plan whose stage structure moved fails the cell here
/// rather than producing a depth column that quietly describes a different
/// program.
fn checked_stages(
    strategy: Strategy,
    rows: u64,
    contributors: u64,
    observed: &[u64],
) -> Vec<Stage> {
    let modelled =
        model::stages(strategy, rows, contributors).expect("the strategy's stage model resolves");
    let modelled_threads: Vec<u64> = modelled.iter().map(|stage| stage.threads).collect();
    assert_eq!(
        modelled_threads,
        observed,
        "the {} plan published launch extents {observed:?}, and this spike's stage model expects \
         {modelled_threads:?}; the model is stale and every depth it reports is unreliable",
        strategy.key(),
    );
    modelled
}

/// Allocates every buffer one alternative's program needs, input already written.
///
/// **One buffer per *allocation*, never per binding.** Two stages of a split
/// address one intermediate, and the program states that by placing both values
/// in one allocation. Allocating per binding would hand each stage a fresh
/// buffer, the producer's partials would never reach the consumer, and the
/// reduction would read uninitialised device memory — a wrong answer rather
/// than a refusal.
fn allocate(
    device: &Device,
    program: &VerifiedKernelProgram,
    operands: &[f32],
) -> (Vec<Buffer>, Buffer, usize) {
    let allocations: Vec<_> = program.allocations().collect();
    let mut buffers = Vec::with_capacity(allocations.len());
    for allocation in &allocations {
        let host_visible = allocation
            .values()
            .any(|value| matches!(value.role(), ValueRole::Input | ValueRole::Output));
        let options = if host_visible {
            MTLResourceOptions::StorageModeShared
        } else {
            MTLResourceOptions::StorageModePrivate
        };
        buffers.push(device.new_buffer(allocation.capacity_bytes().max(1), options));
    }

    let index_of = |target: &_| {
        allocations
            .iter()
            .position(|candidate| candidate == target)
            .expect("every value's allocation is one this program declares")
    };

    let mut output = None;
    let mut readback = 0_usize;
    let mut inputs = 0_usize;
    for value in program.values() {
        let slot = index_of(&value.allocation());
        match value.role() {
            ValueRole::Input => {
                inputs += 1;
                assert_eq!(inputs, 1, "this program family declares one input");
                buffer::write_f32(&buffers[slot], operands);
            }
            ValueRole::Output => {
                readback = usize::try_from(value.required_bytes() / F32_BYTES)
                    .expect("the cell's output element count fits a usize");
                output = Some(buffers[slot].clone());
            }
            ValueRole::Temporary => {}
        }
    }

    let output = output.expect("the program declares an output");
    (buffers, output, readback)
}

/// Builds a compute pipeline for one named function of one object image.
fn pipeline_for(device: &Device, object: &[u8], symbol: &str) -> ComputePipelineState {
    let library = device
        .new_library_with_data(object)
        .expect("the linked object loads");
    let function = library
        .get_function(symbol, None)
        .expect("the emitted symbol resolves");
    let descriptor = ComputePipelineDescriptor::new();
    descriptor.set_compute_function(Some(&function));
    device
        .new_compute_pipeline_state(&descriptor)
        .expect("the pipeline builds")
}

/// Submits one encoded command buffer and reads the output back.
///
/// The command buffer's terminal state is checked *before* the host reads
/// anything, and the accepted state is exactly `Completed`. A failed submission
/// leaves the output buffer holding whatever it held before, and both the
/// verification and the timing would take that as a result.
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

/// Resolves one ABI arena position to the unsigned literal it must be.
fn literal_extent(expressions: &[ExprNode], position: u32) -> u64 {
    let index = usize::try_from(position).expect("an arena position fits a usize");
    match expressions.get(index) {
        Some(ExprNode::Root(AbiRoot::UnsignedLiteral(value))) => *value,
        other => panic!("the launch extent at {position} is not a declared literal: {other:?}"),
    }
}

/// The value every output element must hold, tied to the independent oracle.
///
/// On a cell small enough to evaluate on the host, `tiler-reference` evaluates
/// the same semantic program and every element of its answer is required to
/// equal the closed form. On a larger cell the closed form stands alone, and
/// what licenses it is the arithmetic rather than the shape: a row of `n` unit
/// operands sums to `n` under every grouping, exactly, for every `n` below
/// 2^24.
fn expected_row_sum(
    program: &SemanticProgram,
    operands: &[f32],
    rows: u64,
    contributors: u64,
) -> f32 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "the contributor count is at most 2^14 and f32 represents every integer below 2^24 exactly"
    )]
    let closed_form = contributors as f32;
    if rows * contributors > ORACLE_ELEMENT_LIMIT {
        return closed_form;
    }
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
        .evaluate(program, &[InputBinding::new(&key, &tensor)])
        .expect("the reference evaluates the program");
    let TensorPayloadView::Dense(elements) = outputs[0].payload() else {
        panic!("expected a dense f32 reference output");
    };
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
    closed_form
}

/// Builds a `rows x contributors` multiply-add prologue feeding a trailing sum.
///
/// The same program family `spikes/program-planning/reduction-crossover`
/// compiles, so a cell here and a cell there describe the same plans. The
/// prologue is kept because it is what makes the multi-pass split expressible
/// at all: the split divides the *materialized* reduction, so a bare sum with
/// no prologue is a different program.
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
///
/// Recorded rather than assumed, because a wall-clock measurement taken while
/// the machine is busy is a measurement of the machine's queue and not of the
/// device. Read through `sysctl` so the spike states the number a reader can
/// reproduce with `uptime`.
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

//! The first multi-round cooperative kernel, compiled and executed by an
//! eligible Metal backend and compared bit for bit against the reference-owned
//! grouping.
//!
//! # What one run crosses, and where it stops
//!
//! The oracle side evaluates the declared participant/round/contributor
//! grouping through [`tiler_reference::cooperative_grouped_sum`], which shares
//! no host expression with the KIR lowering, the Metal emitter, or the
//! dispatch. The device side assembles the verified multi-round scheduled
//! region through `tiler_ir`'s public builders, lowers it, emits MSL against
//! the authoritative macOS Apple9 declaration, compiles that through the real
//! Apple offline toolchain, and dispatches the linked `metallib` on this
//! host's GPU. Launch geometry is read from the scheduled program. An
//! agreement is therefore two independent implementations of one declared
//! grouping arriving at the same bits.
//!
//! **The compiler is not in the path, and that is a measurement boundary.**
//! `tiler-compiler` still assembles every cooperative plan with `rounds == 1`
//! (`workgroup_tree_tile` states it). Nothing here may be read as evidence
//! that a caller can *ask* for a multi-round strategy. The accepted single-
//! round neighbour is the same tile the compiler does assemble; the multi-round
//! subject is the verified schedule the lowering ticket landed, executed
//! rather than host-interpreted.
//!
//! **The production scalar CPU profile is not a candidate.** The accepted
//! bounded scalar profile refuses barriers and concurrency by name
//! ([CPU backend](../../../docs/backends/cpu.md)); the spike implements that
//! refusal as "a barrier, which has no participants in a scalar execution
//! model". A later threaded CPU realization must run this property matrix
//! before claiming support. It is not inferred from the Metal result.
//!
//! # Emission is not execution
//!
//! A host that cannot offer Apple9 reports the measured half as unavailable
//! and names what was missing. Successful lowering, MSL emission, or offline
//! compilation is never counted as a device result.
//!
//! # Four perturbations, each with its own failure
//!
//! Round contribution arithmetic, barrier placement, launch width, and
//! grouping are perturbed independently. The first and last are wrong-result
//! comparisons against the reference-owned neighbour groupings. Launch width
//! is a wrong-geometry comparison: the scheduled launch is not the staging
//! allocation, not the grid, and not a fixture constant. Barrier placement is
//! a source comparison: the execution subject is the emitted peeled body, and
//! a source with its fences deleted is a different program this run will not
//! dispatch.

use tiler_build::{BoundMetalCompileDeclaration, BoundMetalDeclarationError};
use tiler_compiler::session::NumericalContract;
use tiler_ir::kernel::lower_scheduled_region;
use tiler_ir::schedule::{
    Access, AccessMode, ArithmeticType, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContributorArrival, ContributorCoverage, ContributorOrder, ContributorPartition,
    ConvergenceEvidence, ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess,
    NumericalRealization, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
    ReductionTopology, RegionId, ScalarProgram, ScheduledRegionBuilder, SynchronizationPlacement,
    SynchronizationPoint, TailPolicy, TensorRole, VerifiedScheduledRegion, workgroup_tree_tile,
};
use tiler_ir::semantic::{CANONICAL_F32_ARITHMETIC_NAN_BITS, F32};
use tiler_ir::shape::{Axis, Shape};
use tiler_metal::emit::emit_translation_unit;
use tiler_metal::record::MetalTranslationUnit;
use tiler_reference::{
    CooperativeGrouping, ReferenceElement, Tensor, TensorPayloadView, cooperative_grouped_sum,
};

use crate::dispatch::Launch;
use crate::serial_sum::{F32_BYTES, pack_f32, unpack_f32};

/// Rows of both fixtures; each row reduces to one output element.
pub(crate) const ROWS: u64 = 2;
/// Contributors per row, covering both the single-round and multi-round splits.
pub(crate) const COLUMNS: u64 = 6;
/// Participants the scheduled program launches per workgroup.
pub(crate) const PARTICIPANTS: u64 = 3;

/// Distinct powers of two: every subset sum is unique, so a dropped or
/// double-counted contributor cannot cancel.
///
/// **What it cannot say.** Every order-preserving regrouping of one row is
/// exact, so the two layouts and the one-round neighbour all produce the same
/// bits. It is the contributor-set half, not the grouping half.
pub(crate) const CONTRIBUTOR_SET_BITS: [u32; 12] = [
    0x3f80_0000, // 1
    0x4000_0000, // 2
    0x4080_0000, // 4
    0x4100_0000, // 8
    0x4180_0000, // 16
    0x4200_0000, // 32
    0x4280_0000, // 64
    0x4300_0000, // 128
    0x4380_0000, // 256
    0x4400_0000, // 512
    0x4480_0000, // 1024
    0x4500_0000, // 2048
];

/// Magnitudes that put the cancelling pair in one round or split it, depending
/// on the layout.
///
/// `5e19` is far enough above the unit ulp that adding one to it is the
/// identity. The two rows are sensitive in opposite directions, so neither the
/// declared round-major grouping nor the participant-major neighbour can agree
/// with the other by luck on both. Each row also carries a small value in a
/// round the other's cancellation does not reach, so a body that folded round
/// zero's range twice disagrees on both rows rather than on one.
///
/// **What it cannot say.** Of the twelve single-contributor replacements by
/// identity, some leave a row's declared total unchanged because a cancelled
/// large value is already invisible beside its pair. It is the grouping half,
/// not the contributor-set half.
pub(crate) const GROUPING_SENSITIVE_BITS: [u32; 12] = [
    5.0e19_f32.to_bits(),
    1.0_f32.to_bits(),
    (-5.0e19_f32).to_bits(),
    3.0_f32.to_bits(),
    0.0_f32.to_bits(),
    0.0_f32.to_bits(),
    0.0_f32.to_bits(),
    5.0e19_f32.to_bits(),
    0.0_f32.to_bits(),
    (-5.0e19_f32).to_bits(),
    2.0_f32.to_bits(),
    0.0_f32.to_bits(),
];

/// The declared grouping the single-round neighbour emits.
#[must_use]
pub(crate) const fn single_round_grouping() -> CooperativeGrouping {
    CooperativeGrouping::declared(PARTICIPANTS, 2, 1)
}

/// The declared grouping the multi-round subject emits.
#[must_use]
pub(crate) const fn multi_round_grouping() -> CooperativeGrouping {
    CooperativeGrouping::declared(PARTICIPANTS, 1, 2)
}

/// The neighbouring layout the contributor arithmetic must not compute.
#[must_use]
pub(crate) const fn participant_major_grouping() -> CooperativeGrouping {
    CooperativeGrouping::participant_major(PARTICIPANTS, 1, 2)
}

/// The one-round regrouping a dropped round term computes on the same cells.
#[must_use]
pub(crate) const fn dropped_round_grouping() -> CooperativeGrouping {
    CooperativeGrouping::declared(PARTICIPANTS, 2, 1)
}

/// The realization Apple9 can honour and a cooperative split is allowed to
/// spend.
fn declared_realization() -> NumericalRealization {
    let contract = NumericalContract::FLUSH_AND_REASSOCIATE_F32;
    NumericalRealization::new(
        contract.key(),
        CANONICAL_F32_ARITHMETIC_NAN_BITS,
        contract.input_subnormals(),
        contract.result_subnormals(),
        contract.contraction(),
        contract.reassociation(),
        contract.permutation(),
        contract.signed_zero(),
        contract.nan_assumptions(),
        contract.infinity_assumptions(),
    )
}

fn linear_schedule(work_items: u64) -> KernelSchedule {
    KernelSchedule {
        binding: ExecutionBinding::GlobalLinearInvocation,
        work_items,
        threads_per_workgroup: 1,
        tail: TailPolicy::Exact,
        output_owner: OwnershipWitnessId::new(0),
        reduction: ReductionTopology::None,
        launch: LaunchPlan {
            grid_threads: work_items,
            threads_per_workgroup: 1,
            zero_work_skips_dispatch: true,
        },
    }
}

/// Builds the `[2, 6] -> [2]` cooperative region at one contributor split.
///
/// The tile comes from [`workgroup_tree_tile`] so the dataflow cannot drift
/// from the canonical single-workgroup tree. A multi-round subject then
/// rewrites only the round count, the per-round contributor width, and the
/// round-boundary point.
fn cooperative_region(contributors_per_partition: u64, rounds: u64) -> VerifiedScheduledRegion {
    let input = Shape::from_dims([ROWS, COLUMNS]);
    let output = Shape::from_dims([ROWS]);
    let axes = vec![Axis::new(1)];
    let work_items = ROWS * PARTICIPANTS;
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(23));
    builder
        .iteration_shape(Shape::from_dims([ROWS, PARTICIPANTS]))
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input.clone(),
                output_shape: output.clone(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input,
                output_shape: output.clone(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: ROWS,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: ROWS },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::StrictSerialSum {
            axes: axes.clone(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: CANONICAL_F32_ARITHMETIC_NAN_BITS,
            empty_identity_bits: 0.0_f32.to_bits(),
        })
        .unwrap();
    builder.numerical(declared_realization()).unwrap();
    let mut tile = workgroup_tree_tile(PARTICIPANTS).expect("the canonical tree tile");
    if rounds > 1 {
        tile.rounds = rounds;
        let phase = tile.synchronization[0];
        tile.synchronization[0].convergence = ConvergenceEvidence::required_for_rounds(rounds);
        tile.synchronization.push(SynchronizationPoint {
            id: tiler_ir::schedule::SyncPointId::new(1),
            placement: SynchronizationPlacement::RoundBoundary,
            convergence: ConvergenceEvidence::required_for_rounds(rounds),
            ..phase
        });
    }
    builder
        .schedule(KernelSchedule {
            threads_per_workgroup: u32::try_from(PARTICIPANTS).expect("participants fit a u32"),
            reduction: ReductionTopology::CooperativeWorkgroup {
                coverage: ContributorCoverage::Exact(ContributorPartition {
                    partitions: PARTICIPANTS,
                    contributors_per_partition,
                }),
                tile,
                axes,
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: false,
                arrival: ContributorArrival::AscendingParticipant,
            },
            launch: LaunchPlan {
                grid_threads: work_items,
                threads_per_workgroup: u32::try_from(PARTICIPANTS).expect("participants fit a u32"),
                zero_work_skips_dispatch: true,
            },
            ..linear_schedule(work_items)
        })
        .unwrap();
    builder.build().expect("the cooperative region verifies")
}

/// The accepted single-round neighbour: three participants, two contributors.
#[must_use]
pub(crate) fn single_round_region() -> VerifiedScheduledRegion {
    cooperative_region(2, 1)
}

/// The multi-round subject: three participants, one contributor, two rounds.
#[must_use]
pub(crate) fn multi_round_region() -> VerifiedScheduledRegion {
    cooperative_region(1, 2)
}

/// Launch geometry the scheduled program publishes.
///
/// Read from [`KernelSchedule::launch`], never from the staging allocation or
/// from a fixture constant. Those two happen to equal the workgroup width on
/// this fixture; the grid does not, and that is what the launch-width
/// perturbation uses.
#[must_use]
pub(crate) fn scheduled_launch(region: &VerifiedScheduledRegion) -> Launch {
    let launch = region.region().schedule.launch;
    Launch {
        grid_threads: launch.grid_threads,
        threads_per_workgroup: u64::from(launch.threads_per_workgroup),
    }
}

/// Workgroup slots the tile allocates, which is not a launch quantity.
#[must_use]
pub(crate) fn staging_slots(region: &VerifiedScheduledRegion) -> u64 {
    let tile = tiler_ir::schedule::cooperative_tile(&region.region().schedule.reduction)
        .expect("a cooperative region carries a tile");
    tile.staging[0].slots
}

/// The declared grouping read from the scheduled program.
///
/// Participants, per-partition contributors, and rounds come from the tile
/// and the partition the region published. The layout is the declared
/// round-major order the body emits; the neighbour is constructed separately
/// so a vacuous comparison has a name.
#[must_use]
pub(crate) fn scheduled_grouping(region: &VerifiedScheduledRegion) -> CooperativeGrouping {
    let ReductionTopology::CooperativeWorkgroup { coverage, tile, .. } =
        &region.region().schedule.reduction
    else {
        panic!("the cooperative fixture builds a cooperative topology")
    };
    CooperativeGrouping::declared(
        tile.coordinates
            .participants
            .participants()
            .expect("the tile's participant product fits"),
        coverage.partition().contributors_per_partition,
        tile.rounds,
    )
}

/// Evaluates one operand set through the reference-owned grouping.
#[must_use]
pub(crate) fn grouped_bits(bits: &[u32], grouping: CooperativeGrouping) -> Vec<u32> {
    let tensor = Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([ROWS, COLUMNS]),
        bits.iter()
            .map(|value| {
                ReferenceElement::from_float_bits(
                    value.to_be_bytes(),
                    tiler_reference::FloatBitOrder::MostSignificantByteFirst,
                )
                .expect("the operand is a valid f32 pattern")
            })
            .collect(),
    )
    .expect("the input tensor is well formed");
    let reduced = cooperative_grouped_sum(&tensor, &[Axis::new(1)], grouping)
        .expect("the declared grouping covers the contributor sequence");
    let TensorPayloadView::Dense(elements) = reduced.payload() else {
        panic!("a dense f32 reference output was expected");
    };
    elements
        .iter()
        .map(|element| {
            u32::from_be_bytes(
                <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
            )
        })
        .collect()
}

/// The emitted, target-bound half of one cooperative kernel, device-free.
pub(crate) struct EmittedCooperative {
    /// The authoritative macOS Apple9 declaration this unit was emitted against.
    pub(crate) declaration: BoundMetalCompileDeclaration,
    /// The emitted Metal translation unit.
    pub(crate) unit: MetalTranslationUnit,
    /// Argument-table index of the read buffer.
    pub(crate) operand_index: u64,
    /// Argument-table index of the write buffer.
    pub(crate) result_index: u64,
    /// Threads the schedule's launch covers.
    pub(crate) grid_threads: u64,
    /// Threads per workgroup the schedule declares.
    pub(crate) threads_per_workgroup: u64,
    /// Grouping the scheduled program declared.
    pub(crate) grouping: CooperativeGrouping,
}

/// Why the emitted half could not be assembled.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EmitFailure {
    /// The authoritative declaration did not assemble.
    Declaration(String),
    /// The region did not lower to a verified kernel.
    Lowering(String),
    /// Emission refused the kernel for this target.
    Emission(String),
    /// The target cannot honour the region's declared numerical realization.
    UnrealizableNumerics(String),
    /// The emitted unit did not declare the boundary shape this run binds.
    UnexpectedSignature(String),
}

impl std::fmt::Display for EmitFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Declaration(cause) => write!(formatter, "the declaration refused: {cause}"),
            Self::Lowering(cause) => write!(formatter, "the region did not lower: {cause}"),
            Self::Emission(cause) => write!(formatter, "emission refused: {cause}"),
            Self::UnrealizableNumerics(cause) => write!(
                formatter,
                "the target cannot honour the declared realization: {cause}",
            ),
            Self::UnexpectedSignature(detail) => {
                write!(
                    formatter,
                    "the emitted signature is not the bound one: {detail}"
                )
            }
        }
    }
}

impl std::error::Error for EmitFailure {}

/// Emits one cooperative region against the authoritative macOS Apple9 row.
///
/// # Errors
///
/// Returns the named refusal of whichever layer declined.
pub(crate) fn emit_region(
    region: &VerifiedScheduledRegion,
) -> Result<EmittedCooperative, EmitFailure> {
    let declaration = BoundMetalCompileDeclaration::first_macos_apple9()
        .map_err(|cause: BoundMetalDeclarationError| EmitFailure::Declaration(cause.to_string()))?;
    let kernel = lower_scheduled_region(region)
        .map_err(|cause| EmitFailure::Lowering(format!("{cause:?}")))?;
    let unit = emit_translation_unit(
        &[&kernel],
        declaration.metal_facts(),
        declaration.emission(),
    )
    .map_err(|cause| EmitFailure::Emission(cause.to_string()))?;
    unit.require_declared_realization()
        .map_err(|cause| EmitFailure::UnrealizableNumerics(cause.to_string()))?;

    let [entry] = unit.entry_points() else {
        return Err(EmitFailure::UnexpectedSignature(format!(
            "{} entry point(s), expected one",
            unit.entry_points().len()
        )));
    };
    let mut operand_index = None;
    let mut result_index = None;
    for binding in entry.buffers() {
        let parameter = binding.parameter();
        match parameter.tensor {
            TensorRole::Output => result_index = Some(u64::from(binding.index())),
            TensorRole::Intermediate | TensorRole::Input { .. } => {
                operand_index = Some(u64::from(binding.index()));
            }
        }
    }
    let (Some(operand_index), Some(result_index)) = (operand_index, result_index) else {
        return Err(EmitFailure::UnexpectedSignature(
            "the entry point does not declare one read and one write boundary".to_owned(),
        ));
    };
    let launch = scheduled_launch(region);
    Ok(EmittedCooperative {
        declaration,
        unit,
        operand_index,
        result_index,
        grid_threads: launch.grid_threads,
        threads_per_workgroup: launch.threads_per_workgroup,
        grouping: scheduled_grouping(region),
    })
}

/// Source text with every `threadgroup_barrier` statement deleted.
///
/// The barrier-placement perturbation: this is a different program from the
/// emitted peeled body, and this run will not dispatch it.
#[must_use]
pub(crate) fn source_without_barriers(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.contains("threadgroup_barrier"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The corruption census one operand set survives under one declared grouping.
///
/// Population is every single-contributor replacement by the reduction
/// identity. The escaped count is how many the declared-grouping oracle fails
/// to notice.
#[must_use]
pub(crate) fn identity_corruption_census(
    operands: &[u32],
    declared: CooperativeGrouping,
) -> (usize, usize) {
    let correct = grouped_bits(operands, declared);
    let mut population = 0_usize;
    let mut escaped = 0_usize;
    for slot in 0..operands.len() {
        let mut corrupt = operands.to_vec();
        corrupt[slot] = 0.0_f32.to_bits();
        population += 1;
        if grouped_bits(&corrupt, declared) == correct {
            escaped += 1;
        }
    }
    (population, escaped)
}

/// Runs the measured half for one emitted kernel over one operand set.
pub(crate) fn measured_execution(
    emitted: &EmittedCooperative,
    bits: &[u32],
) -> crate::measurement::Measured<Vec<u32>> {
    apple::run(emitted, bits)
}

#[cfg(target_os = "macos")]
mod apple {
    use tiler_metal_aot::input::{CompileRequest, OptimizationLevel};

    use super::{EmittedCooperative, F32_BYTES, ROWS, pack_f32, unpack_f32};
    use crate::dispatch::{Storage, run_entry_point};
    use crate::measurement::host::{self, Unresolved};
    use crate::measurement::{Measured, MeasurementBoundary};

    pub(super) fn run(emitted: &EmittedCooperative, bits: &[u32]) -> Measured<Vec<u32>> {
        let apple = match host::resolve() {
            Ok(apple) => apple,
            Err(Unresolved::Absent(reason)) => return Measured::Unavailable(reason),
            Err(Unresolved::Defect(detail)) => return Measured::Failed(detail),
        };
        let request = CompileRequest::new(
            emitted.unit.source(),
            emitted.declaration.aot_target(),
            OptimizationLevel::Default,
            emitted.declaration.numerical_realization(),
        );
        let compiled = match apple.toolchain.compile(&request) {
            Ok(compiled) => compiled,
            Err(error) => {
                return Measured::Failed(format!(
                    "the emitted cooperative unit did not compile and link: {error}"
                ));
            }
        };
        let Some(entry) = emitted.unit.entry_points().first() else {
            return Measured::Failed("the emitted unit declares no entry point".to_owned());
        };
        let storage = Storage {
            operand_bytes: pack_f32(
                bits,
                usize::try_from(F32_BYTES).expect("a carrier width fits a usize"),
            ),
            operand_index: emitted.operand_index,
            result_capacity: usize::try_from(ROWS * F32_BYTES).expect("two f32s fit a usize"),
            result_index: emitted.result_index,
        };
        let launch = crate::dispatch::Launch {
            grid_threads: emitted.grid_threads,
            threads_per_workgroup: emitted.threads_per_workgroup,
        };
        let result_bytes = match run_entry_point(
            &apple.device,
            &compiled.metallib,
            entry.symbol(),
            &storage,
            launch,
        ) {
            Ok(bytes) => bytes,
            Err(error) => {
                return Measured::Failed(format!("the dispatch did not complete: {error}"));
            }
        };
        let boundary: MeasurementBoundary =
            host::boundary(&apple, &emitted.declaration, compiled.metallib.len());
        Measured::Ran {
            boundary: Box::new(boundary),
            observed: unpack_f32(&result_bytes, 4, usize::try_from(ROWS).expect("two rows")),
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod apple {
    use super::EmittedCooperative;
    use crate::measurement::Measured;

    pub(super) fn run(_emitted: &EmittedCooperative, _bits: &[u32]) -> Measured<Vec<u32>> {
        Measured::Unavailable(crate::measurement::absent_apple_row())
    }
}

#[cfg(test)]
mod tests;

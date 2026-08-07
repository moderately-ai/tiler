//! The mechanism: the compiler's own reduction regions, at a partition it did
//! not choose.
//!
//! `governed_partition` is `pub(crate)` in `tiler-compiler` and the partition it
//! returns is a total function of the contributor count, so no shape and no
//! request reaches a second value through the public `compile` entry point.
//! Calibrating the choice needs plans the compiler would build *if* it chose
//! differently, and this module is how they are obtained without widening a
//! compiler boundary and without changing anything shipped.
//!
//! **It rebuilds the two reduction regions, and nothing else.** `tiler-ir`
//! publishes [`ScheduledRegion`], [`ContributorPartition`], and
//! `lower_scheduled_region`, so the three constructors below are transcriptions
//! of `crates/tiler-compiler/src/physical.rs`'s `partial_reduction_region`,
//! `final_reduction_region`, and `single_workgroup_tree_region` with the
//! partition as a parameter rather than a call to the governed choice. Every
//! other input is read from the compilation rather than restated: the numerical
//! realization comes off the compiler's own reduction kernel, and the elementwise
//! prologue kernel is taken from the compiler's plan unmodified and re-emitted
//! beside these.
//!
//! **A transcription is a claim, so it is checked rather than trusted.** At the
//! governed partition these constructors must emit the *byte-identical*
//! translation unit the compiler emits for the same alternative, and must
//! publish the same launch extents the compiler's ABI publishes. The sweep
//! refuses a shape whose anchor fails, so a transcription that drifted from the
//! compiler is a hard failure rather than a partition sweep of some other
//! program. That check is what licenses reading the off-governed rows as
//! evidence about the compiler's plans.
//!
//! Nothing here is reachable from `crates/`. It is spike-local by construction:
//! the shipped compiler still calls `governed_partition` and this file cannot
//! change that.

use tiler_ir::kernel::{VerifiedKernel, lower_scheduled_region};
use tiler_ir::schedule::{
    Access, AccessMode, ArithmeticType, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContributorArrival, ContributorOrder, ContributorPartition, ExecutionBinding, IndexRegion,
    KernelSchedule, LaunchPlan, LogicalAccess, NumericalRealization, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, ReductionPass, ReductionTopology, RegionId,
    ScalarProgram, ScheduledRegion, ScheduledRegionBuilder, TailPolicy, TensorRole,
    partial_reduction_axis, partial_reduction_shape, workgroup_tree_tile,
};
use tiler_ir::shape::{Axis, Shape};

/// The balanced exact split `crates/tiler-compiler/src/physical.rs` chooses.
///
/// Transcribed rather than reached for, because the compiler's is `pub(crate)`
/// and a spike may not widen a compiler boundary to measure it. This copy is the
/// *anchor* of the sweep rather than a convenience: the partition it names is
/// the one whose rebuilt plan must reproduce the compiler's emitted source byte
/// for byte, so a divergence between this copy and the compiler's fails the
/// shape instead of silently mislabelling one column.
#[must_use]
pub fn governed_partition(contributors: u64) -> Option<ContributorPartition> {
    if contributors < 4 {
        return None;
    }
    let mut candidate = contributors.isqrt();
    while candidate >= 2 {
        if contributors.is_multiple_of(candidate) {
            let partitions = contributors / candidate;
            if partitions >= 2 {
                return Some(ContributorPartition {
                    partitions,
                    contributors_per_partition: candidate,
                });
            }
        }
        candidate -= 1;
    }
    None
}

/// Every exact split of `contributors` into at least two partitions of at least
/// two, ascending by partition count.
///
/// This is the population the sweep varies over, and its rule is exactly the one
/// `governed_partition` searches within: an inexact split leaves a ragged final
/// partition this profile does not lower, and a partition holding one
/// contributor folds nothing. The governed choice is one member of this set, so
/// "is the balanced exact split best" is a question about a row of it rather
/// than a comparison against a differently-generated candidate.
#[must_use]
pub fn admissible_partitions(contributors: u64) -> Vec<ContributorPartition> {
    let mut partitions = Vec::new();
    let mut count = 2;
    while count <= contributors / 2 {
        if contributors.is_multiple_of(count) {
            partitions.push(ContributorPartition {
                partitions: count,
                contributors_per_partition: contributors / count,
            });
        }
        count += 1;
    }
    partitions
}

/// The launch extents one rebuilt stage declares.
///
/// Read back from the schedule this module built rather than recomputed at the
/// dispatch site, so the number encoded into the command buffer is the number
/// the region declares. At the governed partition both are compared against the
/// compiler's published ABI extents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Launch {
    /// Invocations along the grid axis.
    pub grid_threads: u64,
    /// Invocations in one workgroup.
    pub threads_per_workgroup: u64,
}

/// One rebuilt reduction stage: its kernel and the extents it is launched at.
pub struct Stage {
    /// The lowered kernel, ready to emit beside the compiler's prologue.
    pub kernel: VerifiedKernel,
    /// The extents its schedule declares.
    pub launch: Launch,
}

impl Stage {
    /// Verifies, lowers, and records one scheduled region.
    fn of(region: ScheduledRegion) -> Result<Self, String> {
        let launch = Launch {
            grid_threads: region.schedule.launch.grid_threads,
            threads_per_workgroup: u64::from(region.schedule.launch.threads_per_workgroup),
        };
        let verified = ScheduledRegionBuilder::from_region(region)
            .build()
            .map_err(|error| format!("the rebuilt region does not verify: {error:?}"))?;
        let kernel = lower_scheduled_region(&verified)
            .map_err(|error| format!("the verified region does not lower: {error:?}"))?;
        Ok(Self { kernel, launch })
    }
}

/// The shape family every constructor below is stated for.
///
/// One rank-two input reduced along its trailing axis into a rank-one output —
/// the program family the retained dispatch sweep measured, restated here
/// because the region constructors need its shapes and this spike does not
/// re-derive them from the semantic program the way the compiler's normalizer
/// does.
#[derive(Clone, Copy, Debug)]
pub struct Subject {
    /// Independent output positions.
    pub rows: u64,
    /// Contributors one output position folds.
    pub contributors: u64,
}

impl Subject {
    /// The reduction's declared input shape.
    fn input_shape(self) -> Shape {
        Shape::from_dims([self.rows, self.contributors])
    }

    /// The reduction's declared output shape.
    fn output_shape(self) -> Shape {
        Shape::from_dims([self.rows])
    }
}

/// The reduced axes, which this program family fixes at the trailing one.
///
/// A free function rather than a method on [`Subject`]: the axis set is a
/// property of the family and not of any one shape in it, and spelling it as a
/// method would suggest a shape could change it.
fn reduced_axes() -> Vec<Axis> {
    vec![Axis::new(1)]
}

/// The linear schedule every non-cooperative region in `physical.rs` starts from.
fn linear_schedule(work_items: u64, owner: OwnershipWitnessId) -> KernelSchedule {
    KernelSchedule {
        binding: ExecutionBinding::GlobalLinearInvocation,
        work_items,
        threads_per_workgroup: 1,
        tail: TailPolicy::Exact,
        output_owner: owner,
        reduction: ReductionTopology::None,
        launch: LaunchPlan {
            grid_threads: work_items,
            threads_per_workgroup: 1,
            zero_work_skips_dispatch: true,
        },
    }
}

/// The multi-pass topology both split passes declare.
fn multi_pass_topology(
    pass: ReductionPass,
    partition: ContributorPartition,
    axes: Vec<Axis>,
    numerical: NumericalRealization,
) -> ReductionTopology {
    ReductionTopology::MultiPass {
        pass,
        partition,
        axes,
        order: ContributorOrder::OriginalAxisLexicographic,
        accumulation: ArithmeticType::F32,
        permits_reassociation: numerical.permits_reassociation(),
        permits_permutation: numerical.permits_permutation(),
    }
}

/// The partial pass of a multi-pass split, at an arbitrary partition.
///
/// `physical.rs::partial_reduction_region`, with `partition` supplied rather
/// than chosen. It reads the prologue's materialized tensor through the
/// reduction-contributor relation and writes one partial per partition per
/// output position.
fn partial_region(
    subject: Subject,
    partition: ContributorPartition,
    numerical: NumericalRealization,
) -> Result<ScheduledRegion, String> {
    let output_shape = subject.output_shape();
    let partial_shape = partial_reduction_shape(&output_shape, partition)
        .ok_or_else(|| "the partial shape is unrepresentable".to_owned())?;
    let partial_elements = subject
        .rows
        .checked_mul(partition.partitions)
        .ok_or_else(|| "the partial element count overflows".to_owned())?;
    Ok(ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(2),
            iteration_shape: partial_shape,
            accesses: vec![
                Access {
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: subject.input_shape(),
                        output_shape: output_shape.clone(),
                        axes: reduced_axes(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(4),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(5),
                    ownership: Some(OwnershipWitnessId::new(2)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(4),
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: subject.input_shape(),
                        output_shape,
                        axes: reduced_axes(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(5),
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: partial_elements,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(2),
                tensor: TensorRole::Intermediate,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: partial_elements,
                },
            },
            scalar_program: ScalarProgram::StrictSerialSum {
                axes: reduced_axes(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: numerical.canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical,
        },
        schedule: KernelSchedule {
            reduction: multi_pass_topology(
                ReductionPass::Partial,
                partition,
                reduced_axes(),
                numerical,
            ),
            ..linear_schedule(partial_elements, OwnershipWitnessId::new(2))
        },
    })
}

/// The final pass of a multi-pass split, at an arbitrary partition.
///
/// `physical.rs::final_reduction_region`. Its reduced axis is the partial
/// tensor's trailing partition axis and deliberately not the request's, which
/// the partial pass already consumed.
fn final_region(
    subject: Subject,
    partition: ContributorPartition,
    numerical: NumericalRealization,
) -> Result<ScheduledRegion, String> {
    let output_shape = subject.output_shape();
    let partial_shape = partial_reduction_shape(&output_shape, partition)
        .ok_or_else(|| "the partial shape is unrepresentable".to_owned())?;
    let axes = vec![
        partial_reduction_axis(&output_shape)
            .ok_or_else(|| "the partition axis is unrepresentable".to_owned())?,
    ];
    Ok(ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(3),
            iteration_shape: output_shape.clone(),
            accesses: vec![
                Access {
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: partial_shape.clone(),
                        output_shape: output_shape.clone(),
                        axes: axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(6),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Output,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(7),
                    ownership: Some(OwnershipWitnessId::new(3)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(6),
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: partial_shape,
                        output_shape,
                        axes: axes.clone(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(7),
                    tensor: TensorRole::Output,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: subject.rows,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(3),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: subject.rows,
                },
            },
            scalar_program: ScalarProgram::StrictSerialSum {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: numerical.canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical,
        },
        schedule: KernelSchedule {
            reduction: multi_pass_topology(ReductionPass::Final, partition, axes, numerical),
            ..linear_schedule(subject.rows, OwnershipWitnessId::new(3))
        },
    })
}

/// The single-workgroup tree, at an arbitrary participant count.
///
/// `physical.rs::single_workgroup_tree_region`. **This is where the partition
/// does three jobs at once**, which is why the tree is swept separately rather
/// than assumed to follow the split: `partition.partitions` is the participant
/// count, the declared workgroup width, and — through the tile's staging — the
/// threadgroup reservation, while `contributors_per_partition` is level zero's
/// fold length.
fn tree_region(
    subject: Subject,
    partition: ContributorPartition,
    numerical: NumericalRealization,
) -> Result<ScheduledRegion, String> {
    let output_shape = subject.output_shape();
    let participants = partition.partitions;
    let tile = workgroup_tree_tile(participants)
        .ok_or_else(|| "the canonical tree tile is unrepresentable".to_owned())?;
    let iteration_shape = partial_reduction_shape(&output_shape, partition)
        .ok_or_else(|| "the tree iteration shape is unrepresentable".to_owned())?;
    let work_items = subject
        .rows
        .checked_mul(participants)
        .ok_or_else(|| "the tree work-item count overflows".to_owned())?;
    let threads_per_workgroup = u32::try_from(participants)
        .map_err(|_| "the participant count exceeds a workgroup width".to_owned())?;
    Ok(ScheduledRegion {
        index: IndexRegion {
            id: RegionId::new(4),
            iteration_shape,
            accesses: vec![
                Access {
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    mode: AccessMode::Read,
                    map: LogicalAccess::ReductionContributor {
                        input_shape: subject.input_shape(),
                        output_shape: output_shape.clone(),
                        axes: reduced_axes(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                    bounds: BoundsWitnessId::new(8),
                    ownership: None,
                },
                Access {
                    tensor: TensorRole::Output,
                    component_role: None,
                    mode: AccessMode::Write,
                    map: LogicalAccess::LinearIdentity,
                    bounds: BoundsWitnessId::new(9),
                    ownership: Some(OwnershipWitnessId::new(4)),
                },
            ],
            bounds_proofs: vec![
                BoundsProof {
                    id: BoundsWitnessId::new(8),
                    tensor: TensorRole::Intermediate,
                    component_role: None,
                    kind: BoundsProofKind::ReductionDomain {
                        input_shape: subject.input_shape(),
                        output_shape,
                        axes: reduced_axes(),
                        order: ContributorOrder::OriginalAxisLexicographic,
                    },
                },
                BoundsProof {
                    id: BoundsWitnessId::new(9),
                    tensor: TensorRole::Output,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: subject.rows,
                    },
                },
            ],
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(4),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: subject.rows,
                },
            },
            scalar_program: ScalarProgram::StrictSerialSum {
                axes: reduced_axes(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: numerical.canonical_arithmetic_nan_bits,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical,
        },
        schedule: KernelSchedule {
            threads_per_workgroup,
            reduction: ReductionTopology::CooperativeWorkgroup {
                partition,
                tile,
                axes: reduced_axes(),
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: numerical.permits_reassociation(),
                permits_permutation: numerical.permits_permutation(),
                arrival: ContributorArrival::AscendingParticipant,
            },
            launch: LaunchPlan {
                grid_threads: work_items,
                threads_per_workgroup,
                zero_work_skips_dispatch: true,
            },
            ..linear_schedule(work_items, OwnershipWitnessId::new(4))
        },
    })
}

/// The reduction stages of one multi-pass split, at an arbitrary partition.
///
/// # Errors
///
/// Returns the reason the partition produced no verifiable pair, which is a fact
/// about the partition rather than a harness fault and is recorded as a decline.
pub fn split_stages(
    subject: Subject,
    partition: ContributorPartition,
    numerical: NumericalRealization,
) -> Result<Vec<Stage>, String> {
    Ok(vec![
        Stage::of(partial_region(subject, partition, numerical)?)?,
        Stage::of(final_region(subject, partition, numerical)?)?,
    ])
}

/// The reduction stage of one single-workgroup tree, at an arbitrary partition.
///
/// # Errors
///
/// Returns the reason the participant count produced no verifiable region.
pub fn tree_stages(
    subject: Subject,
    partition: ContributorPartition,
    numerical: NumericalRealization,
) -> Result<Vec<Stage>, String> {
    Ok(vec![Stage::of(tree_region(
        subject, partition, numerical,
    )?)?])
}

//! Is a staged kernel's intra-workgroup combine structure a function of program scope?
//!
//! The probe builds two verified scheduled regions over **one** subject —
//! `[2, 6] -> [2]`, three participants, six contributors — that differ only in
//! the round structure of their cooperative tile, lowers each to a
//! `VerifiedKernel`, and compares two vectors per region:
//!
//! - the **program-scope observation**: every fact about the staged kernel that
//!   a holder of a `VerifiedKernelProgram` can read — the staging parameter
//!   list, the buffer signature, the admitted builtins, the launch geometry a
//!   stage publishes, and the scheduled-region identity the kernel retains;
//! - the **schedule-scope grouping**: the participant/round/contributor split
//!   the region declares, which is what fixes the combine tree.
//!
//! If the observations coincide while the groupings differ, program scope does
//! not determine the combine structure, and a witness derived from program
//! scope alone would have to guess between two different trees.
//!
//! # Stop condition, fixed before the run
//!
//! The probe answers UNDECIDED, naming the field that separated them, if either
//! region fails to verify or lower, or if the two program-scope observations
//! differ in any component. It answers DETERMINED if the groupings coincide.
//! Only equal observations over differing groupings support NOT-DETERMINED.
//!
//! # Negative control
//!
//! The subject is perturbed, never the assertion. Pair 2 rebuilds the *same*
//! region twice, so a probe that reports non-determination for a pair that has
//! none is broken. Pair 3 joins each kernel against the wrong region's identity
//! and must reject, which is what shows the identity join can say "no".

use tiler_compiler::session::NumericalContract;
use tiler_ir::kernel::{VerifiedKernel, lower_scheduled_region};
use tiler_ir::schedule::{
    Access, AccessMode, ArithmeticType, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContributorArrival, ContributorCoverage, ContributorOrder, ContributorPartition,
    ConvergenceEvidence, ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess,
    NumericalRealization, OwnershipProof, OwnershipProofKind, OwnershipWitnessId, ReductionTopology,
    RegionId, RegionProgram, ScalarProgram, ScheduledRegionBuilder, SynchronizationPlacement,
    SynchronizationPoint, TailPolicy, TensorRole, VerifiedScheduledRegion, workgroup_tree_tile,
};
use tiler_ir::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS;
use tiler_ir::shape::{Axis, Shape};

/// Rows of the fixture; each row reduces to one output element.
const ROWS: u64 = 2;
/// Contributors per row. Both splits below cover exactly this many.
const COLUMNS: u64 = 6;
/// Participants the scheduled program launches per workgroup.
const PARTICIPANTS: u64 = 3;

/// The declared participant/round/contributor split of one cooperative region.
///
/// This is the whole arithmetic content of the combine: participant `p` of
/// round `r` owns the contiguous contributor range at group index
/// `r * partitions + p`, the staged partials of one round are folded in
/// ascending participant order, and the round-carried accumulator is combined
/// with each round's staged total.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Grouping {
    partitions: u64,
    contributors_per_partition: u64,
    rounds: u64,
}

impl Grouping {
    /// Renders the exact binary combine tree this grouping fixes.
    ///
    /// Derived from the declared split rather than from the emitted body, which
    /// is the point: this rendering is available to a reader holding the
    /// region and to nobody holding only the program.
    fn tree(self) -> String {
        let mut carried: Option<String> = None;
        for round in 0..self.rounds {
            let mut round_total: Option<String> = None;
            for participant in 0..self.partitions {
                let group = round * self.partitions + participant;
                let first = group * self.contributors_per_partition;
                let mut partial = format!("c{first}");
                for offset in 1..self.contributors_per_partition {
                    partial = format!("({partial}+c{})", first + offset);
                }
                round_total = Some(match round_total {
                    None => partial,
                    Some(total) => format!("({total}+{partial})"),
                });
            }
            let round_total = round_total.unwrap_or_else(|| "0".to_owned());
            carried = Some(match carried {
                None => round_total,
                Some(accumulator) => format!("({accumulator}+{round_total})"),
            });
        }
        carried.unwrap_or_else(|| "0".to_owned())
    }

    /// Contributors this grouping covers, which must equal the fixture's.
    const fn covered(self) -> u64 {
        self.partitions * self.contributors_per_partition * self.rounds
    }
}

/// Every fact about a staged kernel a holder of program scope can read.
///
/// Deliberately a value with `Eq`: the probe's whole question is whether two
/// regions with different combine trees produce the same one of these.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ProgramScopeObservation {
    /// One `(staging ordinal, element type, address space, slots)` row per
    /// declared workgroup allocation. This is the entire staging vocabulary a
    /// kernel signature carries.
    staging: Vec<(u32, String, String, u64)>,
    /// The staging predicate the contraction witness actually tests today.
    stages_anything: bool,
    /// Buffer parameters, in argument-table order.
    buffers: Vec<(String, String, u64)>,
    /// Governed launch builtins the signature admits.
    builtins: Vec<String>,
    /// Launch geometry a program stage publishes for this kernel.
    launch: (u64, u32),
    /// Planning ordinal of the scheduled region the kernel refines.
    region: u32,
}

impl ProgramScopeObservation {
    /// Reads the observation from the kernel and the launch its stage publishes.
    fn read(kernel: &VerifiedKernel, region: &VerifiedScheduledRegion) -> Self {
        let launch = region.region().schedule.launch;
        Self {
            staging: kernel
                .staging()
                .map(|parameter| {
                    (
                        parameter.staging.get(),
                        format!("{:?}", parameter.element_type),
                        format!("{:?}", parameter.address_space),
                        parameter.element_count,
                    )
                })
                .collect(),
            stages_anything: kernel.staging().len() != 0,
            buffers: kernel
                .buffers()
                .map(|parameter| {
                    (
                        format!("{:?}", parameter.tensor),
                        format!("{:?}", parameter.access),
                        parameter.element_count,
                    )
                })
                .collect(),
            builtins: kernel
                .admitted_builtins()
                .iter()
                .map(|builtin| format!("{builtin:?}"))
                .collect(),
            launch: (launch.grid_threads, launch.threads_per_workgroup),
            region: kernel.scheduled_region().get(),
        }
    }

    /// Names the first component that separates two observations.
    fn first_difference(&self, other: &Self) -> Option<&'static str> {
        if self.staging != other.staging {
            return Some("staging parameter list");
        }
        if self.stages_anything != other.stages_anything {
            return Some("staging-nonempty predicate");
        }
        if self.buffers != other.buffers {
            return Some("buffer signature");
        }
        if self.builtins != other.builtins {
            return Some("admitted builtins");
        }
        if self.launch != other.launch {
            return Some("launch geometry");
        }
        if self.region != other.region {
            return Some("scheduled region ordinal");
        }
        None
    }
}

/// The realization a cooperative split is allowed to spend.
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
        contract.reciprocal_transform(),
        contract.approximate_intrinsics(),
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
/// The tile comes from `workgroup_tree_tile` so the dataflow cannot drift from
/// the canonical single-workgroup tree; a multi-round subject rewrites only the
/// round count, the per-round contributor width, and the round-boundary point.
/// This mirrors the shape `tiler-conformance`'s loop-carried fixture builds.
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
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictSerialSum {
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: CANONICAL_F32_ARITHMETIC_NAN_BITS,
                empty_identity_bits: 0.0_f32.to_bits(),
            },
            numerical: declared_realization(),
        })
        .unwrap();
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

/// Reads the declared grouping off the region, which is where it lives.
fn scheduled_grouping(region: &VerifiedScheduledRegion) -> Grouping {
    let ReductionTopology::CooperativeWorkgroup { coverage, tile, .. } =
        &region.region().schedule.reduction
    else {
        panic!("the fixture builds a cooperative topology")
    };
    let partition = coverage.partition();
    Grouping {
        partitions: partition.partitions,
        contributors_per_partition: partition.contributors_per_partition,
        rounds: tile.rounds,
    }
}

/// One named region under test, lowered and observed.
struct Subject {
    name: &'static str,
    region: VerifiedScheduledRegion,
    kernel: VerifiedKernel,
}

impl Subject {
    fn build(name: &'static str, contributors_per_partition: u64, rounds: u64) -> Self {
        let region = cooperative_region(contributors_per_partition, rounds);
        let kernel = lower_scheduled_region(&region).expect("the cooperative region lowers");
        Self {
            name,
            region,
            kernel,
        }
    }

    fn observation(&self) -> ProgramScopeObservation {
        ProgramScopeObservation::read(&self.kernel, &self.region)
    }

    fn grouping(&self) -> Grouping {
        scheduled_grouping(&self.region)
    }

    fn report(&self) {
        let grouping = self.grouping();
        let observation = self.observation();
        println!("  {}:", self.name);
        println!(
            "    schedule scope  grouping = {} participants x {} contributors x {} rounds (covers {})",
            grouping.partitions,
            grouping.contributors_per_partition,
            grouping.rounds,
            grouping.covered()
        );
        println!("    schedule scope  combine tree = {}", grouping.tree());
        println!("    program scope   staging = {:?}", observation.staging);
        println!(
            "    program scope   staging().len() != 0 = {}",
            observation.stages_anything
        );
        println!(
            "    program scope   launch = {} threads, {} per workgroup",
            observation.launch.0, observation.launch.1
        );
        println!("    program scope   builtins = {:?}", observation.builtins);
        println!(
            "    program scope   kernel identity = {} bytes, first 8 = {:02x?}",
            self.kernel.canonical_identity().as_bytes().len(),
            &self
                .kernel
                .canonical_identity()
                .as_bytes()
                .iter()
                .take(8)
                .collect::<Vec<_>>()
        );
    }
}

/// Compares one pair and prints the verdict the stop condition fixes.
fn compare(label: &str, left: &Subject, right: &Subject) -> bool {
    println!("\n{label}");
    left.report();
    right.report();

    let (left_observation, right_observation) = (left.observation(), right.observation());
    let (left_grouping, right_grouping) = (left.grouping(), right.grouping());

    println!("\n  verdict:");
    if let Some(field) = left_observation.first_difference(&right_observation) {
        println!("    UNDECIDED — program scope separates these two at: {field}.");
        println!("    The probe claims nothing about determination for this pair.");
        return false;
    }
    println!("    program-scope observations are IDENTICAL in every component.");
    if left_grouping == right_grouping {
        println!("    combine groupings are identical too, so this pair is DETERMINED.");
        println!("    (This is the negative control: no discrimination is claimed.)");
        return false;
    }
    println!("    combine groupings DIFFER:");
    println!("      {} -> {}", left.name, left_grouping.tree());
    println!("      {} -> {}", right.name, right_grouping.tree());
    println!(
        "    NOT DETERMINED — one program-scope observation, two different combine trees.\n    A witness deriving from program scope alone would have to guess between them."
    );
    true
}

/// Probes the identity join the region carry would rest on.
fn identity_join(left: &Subject, right: &Subject) {
    println!("\nPair 3 — the identity join, and whether it can say no");
    let joins = |subject: &Subject, region: &VerifiedScheduledRegion| -> bool {
        subject.kernel.scheduled_region_identity().as_bytes() == region.canonical_identity().as_bytes()
    };
    println!(
        "    {} kernel joined against its own region:   {}",
        left.name,
        if joins(left, &left.region) {
            "ACCEPT"
        } else {
            "REJECT"
        }
    );
    println!(
        "    {} kernel joined against the other region: {}",
        left.name,
        if joins(left, &right.region) {
            "ACCEPT"
        } else {
            "REJECT"
        }
    );
    println!(
        "    {} kernel joined against its own region:   {}",
        right.name,
        if joins(right, &right.region) {
            "ACCEPT"
        } else {
            "REJECT"
        }
    );
    println!(
        "    {} kernel joined against the other region: {}",
        right.name,
        if joins(right, &left.region) {
            "ACCEPT"
        } else {
            "REJECT"
        }
    );
    println!(
        "\n    An exact join that accepts the true pairing and rejects the crossed one is\n    what a region carry would rest on. It needs no new encoding: the bytes are\n    the existing CanonicalScheduledRegionIdentity the kernel already retains."
    );
}

fn main() {
    println!("Staged intra-workgroup combine structure: is it a function of program scope?");
    println!(
        "\nSubject: [{ROWS}, {COLUMNS}] -> [{ROWS}], {PARTICIPANTS} participants, {COLUMNS} contributors per row."
    );
    println!(
        "Both members of pair 1 cover the same {COLUMNS} contributors with the same\nparticipant count, and differ only in the round structure of the tile."
    );

    let single_round = Subject::build("single round (3 x 2 x 1)", 2, 1);
    let multi_round = Subject::build("two rounds  (3 x 1 x 2)", 1, 2);
    let duplicate = Subject::build("single round again", 2, 1);

    assert_eq!(
        single_round.grouping().covered(),
        COLUMNS,
        "the single-round split must cover the fixture exactly"
    );
    assert_eq!(
        multi_round.grouping().covered(),
        COLUMNS,
        "the multi-round split must cover the fixture exactly"
    );

    let discriminated = compare(
        "Pair 1 — the subject: same contributors, same participants, different rounds",
        &single_round,
        &multi_round,
    );
    let control = compare(
        "Pair 2 — negative control: the same region built twice",
        &single_round,
        &duplicate,
    );

    identity_join(&single_round, &multi_round);

    println!("\n{}", "=".repeat(78));
    println!("RESULT");
    println!("{}", "=".repeat(78));
    if discriminated && !control {
        println!(
            "NOT DERIVABLE from program scope. Two verified programs agree on every staged\nfact a VerifiedKernelProgram exposes — the staging parameter list, the staging\npredicate the contraction witness tests, the buffer signature, the builtins, the\nlaunch geometry, and the region ordinal — while declaring different combine\ntrees. The separating fact, the tile's round structure, lives only on the\nregion's ReductionTopology, which program scope does not carry."
        );
        println!(
            "\nThe control pair reported DETERMINED, so the probe is not reporting\nnon-determination unconditionally."
        );
        println!(
            "\nThe strongest counterargument, stated rather than hidden: the two kernel\nidentities above are NOT equal, because the emitted bodies differ. Program\nscope is therefore not information-theoretically empty — the structure is\nimplied by the body's staged addresses, barrier phases, and loop bounds. What\nthe probe shows is narrower and is the load-bearing claim: no *declarative*\nrecord in program scope states it. Recovering it from the body means\nsymbolically executing thread-id-dependent staging addresses across\nbarrier-separated phases, which is a second semantics of the body that must\nagree exactly with the emitter; where it disagrees the derived tree is silently\nwrong, which is the failure the witness exists to prevent."
        );
    } else {
        println!(
            "UNDECIDED — the probe did not reach its precondition. Pair 1 discriminated: {discriminated}. Control discriminated: {control}."
        );
    }
}

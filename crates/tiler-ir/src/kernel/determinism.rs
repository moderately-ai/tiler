//! The ADR 0013 plan-determinism witness over one verified kernel program.
//!
//! # What the witness asserts, and what it deliberately does not
//!
//! ADR 0013's accepted initial guarantee is **plan deterministic**: identical
//! input bits and runtime bindings, the same object-bearing artifact-envelope
//! digest, the same selected route coordinate, and the same declared
//! target-environment compatibility identity produce identical output bits. The
//! premises about inputs, bindings, envelope, route, and environment are owned
//! by the artifact and runtime layers; what this module owns is the one premise
//! only the IR can decide — that every evaluation-order choice the program
//! makes is **fixed by its canonical program bytes** rather than left to a
//! runtime race, an atomic arrival order, or a device-varying selection.
//!
//! [`verify_plan_determinism`] is an independent whole-program backstop, not
//! the first line of defence. The schedule verifier already refuses
//! `NondeterministicArrival`, `AtomicAccumulation`, and
//! `SynchronizationKind::Atomic` by name before a verified schedule exists, so
//! no program the current builders can assemble carries one. The witness exists
//! so a *future* admitted construct cannot inherit plan determinism silently:
//! every stage, operation, synchronization subject, numerical freedom, launch
//! expression, execution edge, multipass dependency, and stage owner is walked
//! through exhaustive, wildcard-free matches, so widening any of those
//! vocabularies stops this build here until the new member is classified.
//!
//! # The one refusal reachable today, and why it is a refusal
//!
//! A stage whose declared realization **permits contributor permutation** is
//! refused as [`PlanDeterminismRefusal::UnfixedContributorArrival`]. Permission
//! is not exploitation — a permutation-permitted schedule could in principle
//! still fix one order — but the permission is exactly the freedom an unfixed
//! arrival spelling consumes, and nothing in a verified kernel program proves
//! the freedom went unused. Accepting it would let a later admitted
//! `NondeterministicArrival` or `AtomicAccumulation` construct, which consumes
//! this same permission, arrive already holding a witness. Refusing the
//! permission itself is the fail-closed reading the accepted decision requires:
//! granting permutation must not turn an unfixed arrival into a
//! plan-deterministic witness.
//!
//! [`PlanDeterminismRefusal::RuntimeDependentSelection`] refuses a stage whose
//! launch geometry reads a target property: a property observed from a live
//! device may vary between two devices that share one declared
//! target-environment compatibility identity, so a launch shaped by one is a
//! plan choice the stability subject does not fix. Input extents are the
//! opposite case — ADR 0013 holds input bits and runtime bindings fixed as
//! premises, so an extent-shaped launch is fixed under the subject.
//!
//! [`PlanDeterminismRefusal::OutputAffectingAtomic`] and
//! [`PlanDeterminismRefusal::UnverifiedOpaqueStage`] are backstops that no
//! current builder output reaches: an atomic synchronization subject is refused
//! before a schedule verifies, and whole-program verification proves each stage
//! has exactly one closed owner. Both arms stay because this verifier's claim
//! is about the program in hand, not about which builder produced it.

use std::error::Error;
use std::fmt;

use crate::program::abi::{AbiRoot, ExprNode};
use crate::program::{StageRef, VerifiedKernelProgram};
use crate::schedule::{
    CanonicalScheduledRegionIdentity, RegionNumericalRequirements, ResourceRequirements,
    SubgroupTransfer, SynchronizationKind, SynchronizationSubject,
};

use super::model::{BlockRef, OperationView};
use crate::program::CanonicalKernelProgramIdentity;

/// Proof that one verified kernel program's evaluation is plan deterministic.
///
/// Minted only by [`verify_plan_determinism`], and it borrows the program it
/// proves, so it cannot be replayed against another owner: the borrow ties the
/// witness to the exact `VerifiedKernelProgram` value that was verified, and a
/// consumer joining it with other evidence compares
/// [`Self::kernel_program_identity`] rather than trusting the association.
#[derive(Clone, Copy, Debug)]
pub struct PlanDeterminismWitness<'program> {
    program: &'program VerifiedKernelProgram,
}

impl<'program> PlanDeterminismWitness<'program> {
    /// Returns the canonical identity of the program this witness proves.
    #[must_use]
    pub fn kernel_program_identity(&self) -> &'program CanonicalKernelProgramIdentity {
        self.program.canonical_identity()
    }

    /// Returns each stage's scheduled-region identity, in stage order.
    ///
    /// The topology binding: every reduction topology, contributor order,
    /// staging phase, and synchronization choice is encoded into the scheduled
    /// region's canonical identity, which the kernel retains and folds into
    /// kernel and kernel-program identity. Two programs differing in any
    /// topology field differ here as well as in
    /// [`Self::kernel_program_identity`].
    #[must_use]
    pub fn scheduled_region_identities(
        &self,
    ) -> impl ExactSizeIterator<Item = &'program CanonicalScheduledRegionIdentity> {
        self.program
            .stages()
            .map(|stage| stage.kernel().scheduled_region_identity())
    }
}

/// Why one verified kernel program cannot claim plan determinism.
///
/// `stage` is the zero-based position of the refusing stage in the program's
/// own [`VerifiedKernelProgram::stages`] order — the canonical order identity
/// folds, not the execution order.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: this is a classification
/// a caller consumes to decide what to do next, and a later admitted construct
/// must be able to land its refusal additively.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum PlanDeterminismRefusal {
    /// A stage's contributor-arrival order is not fixed by canonical program
    /// bytes.
    ///
    /// Reached today by a stage whose declared realization permits contributor
    /// permutation: the permission is the freedom an unfixed arrival consumes,
    /// and nothing in the program proves the freedom went unused.
    UnfixedContributorArrival {
        /// Zero-based stage position in the program's canonical stage order.
        stage: usize,
    },
    /// A stage requires an atomic synchronization realization, whose arrival
    /// interleaving affects the produced bits.
    OutputAffectingAtomic {
        /// Zero-based stage position in the program's canonical stage order.
        stage: usize,
    },
    /// A stage's launch geometry depends on a runtime-observed target property,
    /// which the declared target-environment compatibility identity does not
    /// fix.
    RuntimeDependentSelection {
        /// Zero-based stage position in the program's canonical stage order.
        stage: usize,
    },
    /// A stage carries no closed realization or publication owner, so its
    /// evaluation-order obligations cannot be attributed and verified.
    UnverifiedOpaqueStage {
        /// Zero-based stage position in the program's canonical stage order.
        stage: usize,
    },
}

impl fmt::Display for PlanDeterminismRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnfixedContributorArrival { stage } => write!(
                formatter,
                "plan-determinism.unfixed-contributor-arrival: stage {stage}'s declared \
                 realization permits contributor permutation, so its arrival order is not fixed \
                 by canonical program bytes",
            ),
            Self::OutputAffectingAtomic { stage } => write!(
                formatter,
                "plan-determinism.output-affecting-atomic: stage {stage} requires an atomic \
                 synchronization realization whose interleaving affects the produced bits",
            ),
            Self::RuntimeDependentSelection { stage } => write!(
                formatter,
                "plan-determinism.runtime-dependent-selection: stage {stage}'s launch geometry \
                 reads a runtime-observed target property the declared target environment does \
                 not fix",
            ),
            Self::UnverifiedOpaqueStage { stage } => write!(
                formatter,
                "plan-determinism.unverified-opaque-stage: stage {stage} carries no closed \
                 realization or publication owner, so its evaluation order cannot be verified",
            ),
        }
    }
}

impl Error for PlanDeterminismRefusal {}

/// Verifies that one program's evaluation-order choices are fixed by its bytes.
///
/// Exhaustive over every stage, schedule-derived requirement, kernel-body
/// operation, launch expression, execution edge, multipass dependency, and
/// stage owner; the module documentation states which refusal each check can
/// produce and which arms are unreachable backstops for the current builder
/// vocabulary.
///
/// # Errors
///
/// Returns the first [`PlanDeterminismRefusal`] in stage order. A refusal names
/// the stage so an explain surface can attribute it; it never yields a partial
/// witness.
pub fn verify_plan_determinism(
    program: &VerifiedKernelProgram,
) -> Result<PlanDeterminismWitness<'_>, PlanDeterminismRefusal> {
    for (position, stage) in program.stages().enumerate() {
        check_stage_owner(program, stage, position)?;
        check_requirements(stage.kernel().requirements(), position)?;
        check_body(stage.kernel().body())?;
        check_launch(program, stage, position)?;
    }
    // The multipass structure — split combiners, staged handoffs, publishing
    // copies, dependency edges, and the execution order — is a fact of the
    // canonical program bytes: every producer/consumer pair, partition count,
    // and edge reason is folded into kernel-program identity, and the shared
    // IR's whole-program verification proves the order discharges the edges.
    // Nothing there is a choice left open at run time, so the walk above is the
    // whole of what can refuse; these views are read here only to state that
    // they were considered rather than overlooked.
    let _ = program.partial_reductions().len();
    let _ = program.staged_realizations().len();
    let _ = program.dependencies().len();
    let _ = program.execution_order().len();
    Ok(PlanDeterminismWitness { program })
}

/// Proves one stage carries exactly one closed owner.
///
/// A realization stage owns proof-bound coverage (directly or as a split or
/// staged continuation); an administrative publisher owns exact named-output
/// publication claims. A stage with neither has evaluation-order obligations
/// nothing attributes, and one with both is not the closed one-owner form
/// whole-program verification admits; either is refused rather than presumed
/// benign.
fn check_stage_owner(
    program: &VerifiedKernelProgram,
    stage: StageRef<'_>,
    position: usize,
) -> Result<(), PlanDeterminismRefusal> {
    let realizes = !stage.coverage().is_empty()
        || program
            .partial_reductions()
            .any(|split| split.combiner() == stage)
        || program
            .staged_realizations()
            .any(|row| row.consumer() == stage);
    let publishes = program
        .publishing_copies()
        .any(|copy| copy.publisher() == stage);
    if realizes == publishes {
        return Err(PlanDeterminismRefusal::UnverifiedOpaqueStage { stage: position });
    }
    Ok(())
}

/// Checks the schedule-derived requirements of one stage's bound kernel.
///
/// Destructured irrefutably and matched without wildcards, so a widened
/// requirement record, synchronization vocabulary, or numerical-requirement arm
/// stops this build until the new member is classified.
fn check_requirements(
    requirements: ResourceRequirements,
    position: usize,
) -> Result<(), PlanDeterminismRefusal> {
    let ResourceRequirements {
        buffer_bindings: _,
        threads_per_workgroup: _,
        local_memory_bytes: _,
        requires_device_memory: _,
        // One governed complete-u64 family; a capacity, not an order choice.
        index_arithmetic: _,
        synchronization,
        subgroup,
        numerical,
    } = requirements;
    if let Some(SynchronizationSubject {
        kind,
        execution_scope: _,
        visibility_scope: _,
        fenced_spaces: _,
        ordering: _,
    }) = synchronization
    {
        match kind {
            // Order-fixed constructs: each names a schedule point every
            // participant reaches, so what is synchronized is *when* effects
            // become visible, never which value wins a race.
            SynchronizationKind::ControlBarrier
            | SynchronizationKind::AsynchronousCopy
            | SynchronizationKind::SplitPhaseBarrier
            | SynchronizationKind::Collective
            | SynchronizationKind::InterDispatchDependency => {}
            SynchronizationKind::Atomic => {
                return Err(PlanDeterminismRefusal::OutputAffectingAtomic { stage: position });
            }
        }
    }
    if let Some(subject) = subgroup {
        // The declared width and arithmetic fix the tree shape; the transfer is
        // matched so a widened transfer vocabulary is classified here.
        match subject.transfer() {
            SubgroupTransfer::InRangeXorShuffle => {}
        }
    }
    match numerical {
        RegionNumericalRequirements::FloatingPoint {
            input_subnormals: _,
            result_subnormals: _,
            // Contraction and reassociation freedoms select among topologies at
            // *compile* time; the selected topology is encoded in the scheduled
            // region's canonical identity, so the choice is fixed by bytes.
            contraction: _,
            reassociation: _,
            permutation,
            signed_zero: _,
            reciprocal_transform: _,
            approximate_intrinsics: _,
            nan_assumptions: _,
            infinity_assumptions: _,
        } => {
            if permutation == crate::schedule::NumericalPermission::Permitted {
                return Err(PlanDeterminismRefusal::UnfixedContributorArrival { stage: position });
            }
        }
        // A copy moves bits and performs no arithmetic; there is no order to
        // leave unfixed.
        RegionNumericalRequirements::BitPreservingCopy => {}
    }
    Ok(())
}

/// Walks one kernel block and proves every operation's outcome is order-fixed.
///
/// Every current operation is deterministic given fixed operands: loads and
/// stores are witness-bound to disjoint schedule-proven ranges, serial loops
/// carry an explicit fixed iteration order, barriers fix visibility edges, and
/// the pure operations are functions of their inputs. The match is exhaustive
/// and wildcard-free inside the defining crate, so an admitted operation this
/// walk has not classified — an atomic read-modify-write, say — is a build
/// error here rather than an inherited acceptance.
/// The result seam is deliberate although every current arm accepts: a future
/// output-affecting operation adds its refusing arm here, and the stage
/// coordinate it must name travels through the caller that already holds it.
fn check_body(block: BlockRef<'_>) -> Result<(), PlanDeterminismRefusal> {
    for operation in block.operations() {
        match operation.view() {
            OperationView::Builtin { .. }
            | OperationView::Constant { .. }
            | OperationView::Binary { .. }
            | OperationView::Compare { .. }
            | OperationView::Convert { .. }
            | OperationView::Unary { .. }
            | OperationView::PackedExtract { .. }
            | OperationView::Load { .. }
            | OperationView::GuardedLoad { .. }
            | OperationView::Store { .. }
            | OperationView::InputExtent { .. }
            | OperationView::Barrier { .. }
            | OperationView::StagedStore { .. }
            | OperationView::StagedLoad { .. } => {}
            OperationView::Predicated { body, .. } => check_body(body)?,
            OperationView::SerialLoop(serial) => check_body(serial.body())?,
        }
    }
    Ok(())
}

/// Proves one stage's launch geometry is fixed by the stability subject.
///
/// A launch expression may read literals and input extents — both fixed under
/// ADR 0013's premises — and must not read a target property: an observed
/// property can differ between two devices sharing one declared
/// target-environment compatibility identity, so a launch shaped by one is a
/// runtime-dependent plan choice.
fn check_launch(
    program: &VerifiedKernelProgram,
    stage: StageRef<'_>,
    position: usize,
) -> Result<(), PlanDeterminismRefusal> {
    let launch = stage.launch();
    let arena = program.abi_expressions();
    for root in [launch.grid_threads, launch.threads_per_workgroup] {
        if reads_target_property(arena, root) {
            return Err(PlanDeterminismRefusal::RuntimeDependentSelection { stage: position });
        }
    }
    Ok(())
}

/// Returns whether any root reachable from `node` is a target property.
fn reads_target_property(arena: &[ExprNode], node: u32) -> bool {
    let Some(expression) = arena.get(usize::try_from(node).expect("u32 fits usize")) else {
        // A verified program's launch references resolve; an unresolvable one
        // cannot be proven property-free, so it is treated as reading one.
        return true;
    };
    match expression {
        ExprNode::Root(AbiRoot::TargetProperty { .. }) => true,
        ExprNode::Root(
            AbiRoot::UnsignedLiteral(_) | AbiRoot::BooleanLiteral(_) | AbiRoot::InputExtent { .. },
        ) => false,
        ExprNode::Unary { operand, .. } => reads_target_property(arena, *operand),
        ExprNode::Binary { left, right, .. } => {
            reads_target_property(arena, *left) || reads_target_property(arena, *right)
        }
        ExprNode::Select {
            condition,
            if_true,
            if_false,
        } => {
            reads_target_property(arena, *condition)
                || reads_target_property(arena, *if_true)
                || reads_target_property(arena, *if_false)
        }
    }
}

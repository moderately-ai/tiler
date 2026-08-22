//! Plan-owned combine-tree witness for one contraction occurrence.
//!
//! Scalar set membership is insufficient for a relaxed contraction: a body
//! claiming tree A must be checked against A even when its wrong result
//! happens to equal a value tree B can produce. The witness binds one
//! [`OrderedContractionF32Tree`] to the exact semantic graph, verified kernel
//! program, and occurrence it claims, derived from the program's own verified
//! records rather than trusted from a producer.
//!
//! The first public surface supports one uniform tree template for every
//! output coordinate of a **static-`K`** occurrence: a static direct
//! realization becomes the canonical left chain, and an exact static-`K`
//! regular split — the program-scope [`PartialReduction`] declaration — becomes
//! a left chain inside each positive contiguous partition followed by a
//! left-chain merge of partials in ascending partition order. The accepted
//! current live-extent contraction, whose contributor count arrives from a
//! runtime input-axis extent, is refused as
//! [`ContractionF32PlanWitnessError::LiveContributorCount`]; a future live- or
//! coordinate-dependent tree mapping must become identity-bearing in
//! schedule/kernel/artifact encoding and gain a new witness representation
//! rather than reusing this constructor.
//!
//! # Workgroup staging, and the scheduled region that explains it
//!
//! Program scope does not state what a kernel's workgroup staging *does*. Two
//! verified regions over one subject, differing only in their cooperative
//! tile's round structure, produce byte-identical program-scope observations —
//! the same staging row, the same launch, the same builtins — while declaring
//! different associations of the same contributors; the executable probe is
//! [`spikes/reference/staged-combine-derivability`]. So
//! [`Self::from_program`][ContractionF32PlanWitness::from_program], which sees
//! program scope alone, refuses every kernel that declares any staging at all.
//!
//! [`ContractionF32PlanWitness::from_program_and_regions`] takes the missing
//! record instead of guessing at it. A verified kernel retains the canonical
//! identity of the scheduled region it refines, so the region that states the
//! combine structure can be *joined* back by identity, and the tree is read
//! from that record. It is never recovered from the kernel body: doing that
//! would mean symbolically executing thread-id-dependent staging addresses
//! across barrier-separated phases, a second semantics of the body that yields
//! a silently wrong tree wherever it disagrees with the emitter — which is the
//! failure this witness exists to prevent.
//!
//! [`spikes/reference/staged-combine-derivability`]: ../../../../../spikes/reference/staged-combine-derivability/README.md
//!
//! [`PartialReduction`]: super::PartialReduction

use std::error::Error;
use std::fmt;

use crate::kernel::VerifiedKernel;
use crate::schedule::{
    ContractionF32TopologyLimits, ContractionF32TreeError, ContractionF32TreeNode,
    OrderedContractionF32Tree, ReductionTopology, VerifiedScheduledRegion, element_count,
};
use crate::semantic::{
    CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, ContractionIndexStructure, OpKey, SemanticGraphIdentity,
    SemanticProgram, tensor_contraction_f32_op,
};

use super::model::{CanonicalKernelProgramIdentity, SemanticOccurrence, VerifiedKernelProgram};

/// A typed refusal of one witness derivation.
///
/// Exhaustive over the accepted vocabulary. Several variants are reserved
/// refusals for topology facts the program encoding cannot state at this base:
/// no verified program today declares padded coverage, an unfixed arrival, a
/// permuted contributor order, or a per-output topology, so those variants
/// name the population this witness must refuse the day such an encoding
/// lands, rather than a path reachable now. [`Self::TopologyUnsupported`] is
/// the live catch-all for a covering realization whose exact binary combine
/// tree cannot be derived from the records the caller supplied — under
/// [`ContractionF32PlanWitness::from_program`], which is handed program scope
/// alone, that includes any kernel that declares workgroup staging, whose
/// intra-workgroup combine structure the program does not carry.
///
/// [`ContractionF32PlanWitness::from_program_and_regions`] is additionally
/// handed the scheduled regions, so a staged kernel refuses there only when the
/// joined region's own topology states no tree this witness can express, and
/// [`Self::ScheduledRegionUnjoined`] separates "you did not supply the record"
/// from "the record does not say".
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractionF32PlanWitnessError {
    /// The program realizes a different semantic graph.
    SemanticGraphMismatch,
    /// The occurrence ordinal names no operation of the semantic program.
    OccurrenceOutOfRange,
    /// The occurrence is not the governed contraction.
    WrongOperation {
        /// The occurrence's actual operation key.
        actual: OpKey,
    },
    /// No stage of the program covers the occurrence.
    OccurrenceNotCovered,
    /// No covering stage carries a numerical realization.
    ///
    /// Reserved: every verified kernel carries one today, so this names the
    /// refusal a future realization-free stage encoding must take.
    MissingRealization,
    /// The occurrence's covering stages disagree on their realization, or two
    /// split declarations claim one occurrence.
    AmbiguousRealization,
    /// The contributor count arrives from a live runtime extent.
    LiveContributorCount,
    /// A declared split does not cover the contributor sequence exactly.
    MalformedPartition,
    /// The topology covers an identity-padded contributor sequence.
    ///
    /// Reserved: the program-scope split declaration is exact by construction
    /// today.
    PaddedCoverageUnsupported,
    /// The topology permutes the contributor sequence.
    ///
    /// Reserved: no program encoding states a permuted membership today.
    PermutationUnsupported,
    /// The topology's combine order is not fixed by the program.
    ///
    /// Reserved: the program-scope split declaration fixes ascending partition
    /// order by construction today.
    ArrivalNotFixed,
    /// The covering realization's exact binary combine tree cannot be derived
    /// from program scope.
    TopologyUnsupported,
    /// A kernel declares workgroup staging and no supplied scheduled region
    /// carries the canonical identity that kernel refines.
    ///
    /// Distinct from [`Self::TopologyUnsupported`] because the two are fixed
    /// differently: this one says the caller can supply the missing record,
    /// while the catch-all says the record itself states no expressible tree.
    /// Reachable only through
    /// [`ContractionF32PlanWitness::from_program_and_regions`];
    /// [`ContractionF32PlanWitness::from_program`] supplies no regions at all
    /// and refuses a staged kernel under the catch-all, as it always has.
    ScheduledRegionUnjoined,
    /// The topology differs per output coordinate.
    ///
    /// Reserved: no program encoding states a coordinate-dependent topology
    /// today.
    PerOutputTopologyUnsupported,
    /// The derived node vector failed tree validation or a caller limit.
    Tree(ContractionF32TreeError),
}

impl fmt::Display for ContractionF32PlanWitnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SemanticGraphMismatch => {
                formatter.write_str("kernel program realizes a different semantic graph")
            }
            Self::OccurrenceOutOfRange => {
                formatter.write_str("occurrence ordinal names no operation of the program")
            }
            Self::WrongOperation { actual } => {
                write!(
                    formatter,
                    "occurrence is {actual}, not the governed contraction"
                )
            }
            Self::OccurrenceNotCovered => {
                formatter.write_str("no stage of the program covers the occurrence")
            }
            Self::MissingRealization => {
                formatter.write_str("no covering stage carries a numerical realization")
            }
            Self::AmbiguousRealization => {
                formatter.write_str("the occurrence's covering realization is not unique")
            }
            Self::LiveContributorCount => {
                formatter.write_str("contributor count arrives from a live runtime extent")
            }
            Self::MalformedPartition => {
                formatter.write_str("declared split does not cover the contributors exactly")
            }
            Self::PaddedCoverageUnsupported => {
                formatter.write_str("identity-padded coverage has no witness support")
            }
            Self::PermutationUnsupported => {
                formatter.write_str("a permuted contributor sequence has no witness support")
            }
            Self::ArrivalNotFixed => {
                formatter.write_str("the combine order is not fixed by the program")
            }
            Self::TopologyUnsupported => formatter
                .write_str("the exact binary combine tree cannot be derived from program scope"),
            Self::ScheduledRegionUnjoined => formatter.write_str(
                "no supplied scheduled region matches the staged kernel's region identity",
            ),
            Self::PerOutputTopologyUnsupported => {
                formatter.write_str("a per-output-coordinate topology has no witness support")
            }
            Self::Tree(source) => source.fmt(formatter),
        }
    }
}

impl Error for ContractionF32PlanWitnessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Tree(source) => Some(source),
            _ => None,
        }
    }
}

impl From<ContractionF32TreeError> for ContractionF32PlanWitnessError {
    fn from(source: ContractionF32TreeError) -> Self {
        Self::Tree(source)
    }
}

/// One validated plan-owned combine tree, bound to its exact subjects.
///
/// Opaque: only [`Self::from_program`] constructs one, so holding a witness is
/// evidence that the semantic graph join, the reached contraction occurrence,
/// its unique effective realization, and the verified program topology were
/// all validated before the tree was derived.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractionF32PlanWitness {
    semantic_graph: SemanticGraphIdentity,
    kernel_program: CanonicalKernelProgramIdentity,
    occurrence: SemanticOccurrence,
    tree: OrderedContractionF32Tree,
}

impl ContractionF32PlanWitness {
    /// Derives the plan's combine tree for one contraction occurrence.
    ///
    /// # Errors
    ///
    /// Returns [`ContractionF32PlanWitnessError`] naming the first failed
    /// validation: the semantic graph join, the occurrence's operation, its
    /// coverage and unique realization, the topology derivation, and finally
    /// tree validation under the caller's limits.
    pub fn from_program(
        semantic: &SemanticProgram,
        program: &VerifiedKernelProgram,
        occurrence: SemanticOccurrence,
        limits: ContractionF32TopologyLimits,
    ) -> Result<Self, ContractionF32PlanWitnessError> {
        Self::derive(semantic, program, occurrence, RegionJoin::Unjoined, limits)
    }

    /// Derives the plan's combine tree, joining the scheduled regions that
    /// state what each kernel's workgroup staging does.
    ///
    /// The same derivation as [`Self::from_program`] with one difference: where
    /// that constructor refuses every kernel declaring workgroup staging —
    /// because program scope carries no record of what the staging does — this
    /// one resolves the kernel's region out of `regions` by canonical scheduled
    /// region identity and reads the region's own [`ReductionTopology`].
    ///
    /// The join is exact in both directions. A kernel matches only the region
    /// whose canonical identity it retains, so a crossed pairing is refused as
    /// [`ContractionF32PlanWitnessError::ScheduledRegionUnjoined`] rather than
    /// silently answered from the wrong record; and because that identity is a
    /// pure function of normalized schedule content, two entries that both match
    /// state the same topology and either answers.
    ///
    /// Supplying regions never *changes* a derived tree — it only converts a
    /// refusal into an admission, and only where the joined region proves the
    /// staging leaves the tree this witness already derives intact. Regions the
    /// join does not need are ignored, so passing the whole plan's regions is
    /// the ordinary call.
    ///
    /// # Errors
    ///
    /// As [`Self::from_program`], plus
    /// [`ContractionF32PlanWitnessError::ScheduledRegionUnjoined`] when a
    /// staged kernel's region is not among `regions`.
    pub fn from_program_and_regions(
        semantic: &SemanticProgram,
        program: &VerifiedKernelProgram,
        occurrence: SemanticOccurrence,
        regions: &[VerifiedScheduledRegion],
        limits: ContractionF32TopologyLimits,
    ) -> Result<Self, ContractionF32PlanWitnessError> {
        Self::derive(
            semantic,
            program,
            occurrence,
            RegionJoin::Supplied(regions),
            limits,
        )
    }

    /// The one derivation both constructors run, parameterized by the join.
    fn derive(
        semantic: &SemanticProgram,
        program: &VerifiedKernelProgram,
        occurrence: SemanticOccurrence,
        join: RegionJoin<'_>,
        limits: ContractionF32TopologyLimits,
    ) -> Result<Self, ContractionF32PlanWitnessError> {
        if semantic.semantic_identity().graph() != program.semantic_graph_identity() {
            return Err(ContractionF32PlanWitnessError::SemanticGraphMismatch);
        }
        let Some(operation) = semantic
            .operations()
            .find(|operation| semantic.canonical_operation_ordinal(*operation) == occurrence.get())
        else {
            return Err(ContractionF32PlanWitnessError::OccurrenceOutOfRange);
        };
        if operation.key() != &tensor_contraction_f32_op() {
            return Err(ContractionF32PlanWitnessError::WrongOperation {
                actual: operation.key().clone(),
            });
        }

        // Contributor count `K`, from the occurrence's validated structure and
        // static operand extents. A live or symbolic contracted extent is the
        // refused live-`K` population.
        let Some(structure_value) = operation
            .attributes()
            .get(CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE)
        else {
            // A verified graph's contraction always carries its structure; a
            // program that does not has no derivable combine tree.
            return Err(ContractionF32PlanWitnessError::TopologyUnsupported);
        };
        let Ok(structure) = ContractionIndexStructure::from_canonical_value(structure_value) else {
            return Err(ContractionF32PlanWitnessError::TopologyUnsupported);
        };
        let mut contributor_count: u64 = 1;
        for contracted in structure.contracted() {
            let mut extent: Option<u64> = None;
            'search: for (position, tuple) in structure.operands().enumerate() {
                if let Some(axis) = tuple.iter().position(|index| index == contracted) {
                    let Some(value) = operation.operands().nth(position) else {
                        return Err(ContractionF32PlanWitnessError::TopologyUnsupported);
                    };
                    let Ok(shape) = semantic.shape(value) else {
                        return Err(ContractionF32PlanWitnessError::TopologyUnsupported);
                    };
                    let Some(shape) = shape.as_static() else {
                        return Err(ContractionF32PlanWitnessError::LiveContributorCount);
                    };
                    extent = shape.extents().get(axis).map(|extent| extent.get());
                    break 'search;
                }
            }
            let Some(extent) = extent else {
                return Err(ContractionF32PlanWitnessError::TopologyUnsupported);
            };
            contributor_count = contributor_count.checked_mul(extent).ok_or(
                ContractionF32PlanWitnessError::Tree(ContractionF32TreeError::NodeCountOverflow),
            )?;
        }

        // Coverage: whole-program coverage keys on the occurrence and refuses
        // one occurrence twice, so at most one stage covers it.
        let Some(covering) = program.stages().find(|stage| {
            stage
                .coverage()
                .iter()
                .any(|covered| covered.occurrence() == occurrence)
        }) else {
            return Err(ContractionF32PlanWitnessError::OccurrenceNotCovered);
        };
        // A staged-realization chain continues the occurrence across stages;
        // its combined tree is not derivable from program scope.
        if program
            .staged_realizations()
            .any(|staged| staged.occurrence() == occurrence)
        {
            return Err(ContractionF32PlanWitnessError::TopologyUnsupported);
        }
        // A kernel declaring workgroup staging may combine inside the workgroup,
        // and the program does not carry that combine structure. Unjoined, that
        // is the end of it; joined, the region says which it is. Classified here
        // rather than after the split below so an unjoined derivation refuses at
        // exactly the point, and with exactly the cause, it always has.
        let covering_staging = staged_role(covering.kernel(), join)?;

        let splits: Vec<_> = program
            .partial_reductions()
            .filter(|split| split.occurrence() == occurrence)
            .collect();
        if splits.len() > 1 {
            return Err(ContractionF32PlanWitnessError::AmbiguousRealization);
        }
        let split = splits.first().copied();

        // Unique effective realization across the covering producer and, for a
        // split, its combiner.
        let realization = covering.kernel().numerical();
        if let Some(split) = split {
            let combiner_staging = staged_role(split.combiner().kernel(), join)?;
            // The combiner folds one partial per partition, so that — not `K` —
            // is the sequence its staging must leave intact.
            combiner_staging.admits(split.partitions())?;
            if split.combiner().kernel().numerical() != realization
                || split.producer().kernel().numerical() != realization
            {
                return Err(ContractionF32PlanWitnessError::AmbiguousRealization);
            }
        }
        // The covering kernel folds every contributor when it realizes the
        // occurrence alone, and one partition's worth when a split declares it.
        covering_staging.admits(match split {
            None => contributor_count,
            Some(split) => split.contributors_per_partition(),
        })?;

        let nodes = match split {
            None => left_chain_nodes(contributor_count)?,
            Some(split) => {
                let partitions = split.partitions();
                let per_partition = split.contributors_per_partition();
                if partitions == 0
                    || per_partition == 0
                    || partitions
                        .checked_mul(per_partition)
                        .is_none_or(|total| total != contributor_count)
                {
                    return Err(ContractionF32PlanWitnessError::MalformedPartition);
                }
                partitioned_chain_nodes(partitions, per_partition, contributor_count)?
            }
        };
        let tree = OrderedContractionF32Tree::try_from_postorder(contributor_count, nodes, limits)?;

        Ok(Self {
            semantic_graph: semantic.semantic_identity().graph().clone(),
            kernel_program: program.canonical_identity().clone(),
            occurrence,
            tree,
        })
    }

    /// Returns the semantic graph identity this witness is bound to.
    #[must_use]
    pub const fn semantic_graph_identity(&self) -> &SemanticGraphIdentity {
        &self.semantic_graph
    }

    /// Returns the verified kernel-program identity this witness is bound to.
    #[must_use]
    pub const fn kernel_program_identity(&self) -> &CanonicalKernelProgramIdentity {
        &self.kernel_program
    }

    /// Returns the witnessed occurrence.
    #[must_use]
    pub const fn occurrence(&self) -> SemanticOccurrence {
        self.occurrence
    }

    /// Returns the validated combine tree, one uniform template for every
    /// output coordinate.
    #[must_use]
    pub const fn tree(&self) -> &OrderedContractionF32Tree {
        &self.tree
    }
}

/// The scheduled regions a derivation may consult, if any.
///
/// Two constructors and one derivation, so the difference between them is a
/// value rather than a duplicated body: [`ContractionF32PlanWitness::from_program`]
/// passes [`Self::Unjoined`] and cannot reach a region at all, which is what
/// keeps its refusals exactly what they were.
#[derive(Clone, Copy, Debug)]
enum RegionJoin<'a> {
    /// No regions supplied. A kernel that declares workgroup staging refuses.
    Unjoined,
    /// Regions the caller supplied, to be resolved by canonical identity.
    Supplied(&'a [VerifiedScheduledRegion]),
}

/// What one kernel's declared workgroup staging does to the combine tree.
///
/// Only the shapes this witness can *express* appear. An unrecognized staging
/// shape has no variant here and refuses in [`staged_role`], so widening the
/// schedule vocabulary cannot silently fall through to a derived tree.
#[derive(Clone, Copy, Debug)]
enum StagedRole {
    /// The kernel declares no workgroup staging, so nothing was joined and
    /// nothing needs to be: the program-scope derivation stands.
    Unstaged,
    /// The kernel stages operand tiles and folds every contributor of its own
    /// sequence into one carried accumulator.
    ///
    /// Carries the contributor count the joined region folds, so the caller can
    /// require it to be the sequence the witness is deriving a tree over.
    CarriedAccumulator {
        /// Points of the region's declared contracted iteration space.
        contributors: u64,
    },
}

impl StagedRole {
    /// Checks that this staging leaves a fold of `contributors` intact.
    ///
    /// [`Self::Unstaged`] passes unconditionally — there is no staging to
    /// reconcile. [`Self::CarriedAccumulator`] passes when the region folds
    /// exactly the sequence the caller is deriving a tree over; a region folding
    /// some other count is not describing this fold, and is refused rather than
    /// answered with a tree shaped by the semantic count alone.
    ///
    /// **The refusal is reserved at this base.** For a direct realization the
    /// program layer already forces the two counts to agree: a stage's operand
    /// extent is checked against its kernel's declared buffer extent, so a
    /// disagreement is refused as `StageElementCount` before a witness is ever
    /// asked for. Only a declared [`PartialReduction`] could separate them,
    /// because its partition counts are program-scope numbers no region check
    /// ties to the joined topology — and no contraction split is constructible
    /// here, since `ContractionAxisSource` cannot factor a contracted axis into
    /// a partition and a within-partition axis. The relation is kept because it
    /// is the correct one the day a split becomes expressible.
    ///
    /// [`PartialReduction`]: super::PartialReduction
    fn admits(self, contributors: u64) -> Result<(), ContractionF32PlanWitnessError> {
        match self {
            Self::Unstaged => Ok(()),
            Self::CarriedAccumulator {
                contributors: folded,
            } if folded == contributors => Ok(()),
            Self::CarriedAccumulator { .. } => {
                Err(ContractionF32PlanWitnessError::MalformedPartition)
            }
        }
    }
}

/// Classifies one kernel's workgroup staging against the region that states it.
///
/// # Why the tree may be read from the region and never from the body
///
/// A verified kernel retains the canonical identity of the region it refines,
/// and that identity is a pure function of normalized schedule content. So the
/// region resolved here is *the* record the kernel was lowered from, and the
/// combine structure it declares is the one the emitter was obliged to realize
/// — which the structured-kernel verifier separately proves of the emitted
/// body. Reading the body instead would be a second semantics of it, correct
/// only where the two agree and silently wrong everywhere else.
///
/// # Why only the carried accumulator is admitted
///
/// [`ReductionTopology::CooperativeContraction`] is the one staged topology a
/// contraction region can carry, and its fold is the declared contributor
/// sequence itself. Three independent statements agree, and this arm rests on
/// the first two — the third is corroboration, not the source:
///
/// 1. the variant's own `permits_reassociation` doc, at the anchor `ascending
///    contracted order straight through the round`;
/// 2. `verify_cooperative_contraction`, which states the same relation at the
///    anchor `ascending contracted order across the whole round` and names the
///    regrouped alternative's reserved vocabulary; and
/// 3. `emit_cooperative_contraction`'s `fold_tile`, which seeds from the first
///    product and threads one accumulator through every round, at the anchor
///    `into a subtotal of their own`.
///
/// So the staging moves *memory* and not the association: the tree stays the
/// canonical left chain this module already derives, and the join's whole effect
/// is to license that chain for a kernel program scope could only refuse.
///
/// Every other topology refuses. [`ReductionTopology::CooperativeWorkgroup`]
/// stages *partials* and its tree is a partitioned, round-structured chain that
/// this witness has no representation for; it also cannot describe a contraction
/// fold, because a cooperative reduction region is admitted only for a scalar
/// program `split_family` classifies and it classifies no contraction. A region
/// declaring staging under any non-cooperative topology is inconsistent with its
/// own kernel, and is refused for the same reason.
///
/// **Those arms are reserved at this base**, and the match is written
/// exhaustively so that a widened [`ReductionTopology`] is a build error here
/// rather than a staging shape that silently keeps the left chain. No verified
/// kernel program can currently pair a contraction occurrence with a
/// cooperative-workgroup kernel: the fold gate hands such a region exactly one
/// read, and a contraction has two operands, so the program either leaves the
/// second unread or omits it.
/// `a_cooperative_workgroup_kernel_cannot_cover_a_contraction_occurrence` in
/// this crate's program tests watches that wall and fails if it ever moves.
fn staged_role(
    kernel: &VerifiedKernel,
    join: RegionJoin<'_>,
) -> Result<StagedRole, ContractionF32PlanWitnessError> {
    if kernel.staging().len() == 0 {
        return Ok(StagedRole::Unstaged);
    }
    let RegionJoin::Supplied(regions) = join else {
        return Err(ContractionF32PlanWitnessError::TopologyUnsupported);
    };
    let Some(region) = regions
        .iter()
        .find(|region| region.canonical_identity() == kernel.scheduled_region_identity())
    else {
        return Err(ContractionF32PlanWitnessError::ScheduledRegionUnjoined);
    };
    match &region.region().schedule.reduction {
        ReductionTopology::CooperativeContraction {
            contracted_shape, ..
        } => {
            let contributors = element_count(contracted_shape)
                .map_err(|_| ContractionF32PlanWitnessError::TopologyUnsupported)?;
            Ok(StagedRole::CarriedAccumulator { contributors })
        }
        ReductionTopology::None
        | ReductionTopology::Serial { .. }
        | ReductionTopology::MultiPass { .. }
        | ReductionTopology::Contraction { .. }
        | ReductionTopology::LiveContraction { .. }
        | ReductionTopology::CooperativeWorkgroup { .. } => {
            Err(ContractionF32PlanWitnessError::TopologyUnsupported)
        }
    }
}

/// Builds the canonical left chain's postorder node vector.
fn left_chain_nodes(
    contributor_count: u64,
) -> Result<Vec<ContractionF32TreeNode>, ContractionF32PlanWitnessError> {
    if contributor_count == 0 {
        return Err(ContractionF32PlanWitnessError::Tree(
            ContractionF32TreeError::EmptyContributors,
        ));
    }
    let count = usize::try_from(contributor_count)
        .ok()
        .and_then(|count| count.checked_mul(2))
        .and_then(|count| count.checked_sub(1))
        .ok_or(ContractionF32PlanWitnessError::Tree(
            ContractionF32TreeError::NodeCountOverflow,
        ))?;
    let mut nodes = Vec::with_capacity(count);
    nodes.push(ContractionF32TreeNode::Leaf { contributor: 0 });
    let mut root: u32 = 0;
    for contributor in 1..contributor_count {
        nodes.push(ContractionF32TreeNode::Leaf { contributor });
        let leaf = u32::try_from(nodes.len() - 1).expect("node count is bounded above");
        nodes.push(ContractionF32TreeNode::Add {
            left: root,
            right: leaf,
        });
        root = u32::try_from(nodes.len() - 1).expect("node count is bounded above");
    }
    Ok(nodes)
}

/// Builds the regular-split postorder node vector: a left chain inside each
/// contiguous partition, then a left-chain merge of the partition roots in
/// ascending partition order.
fn partitioned_chain_nodes(
    partitions: u64,
    per_partition: u64,
    contributor_count: u64,
) -> Result<Vec<ContractionF32TreeNode>, ContractionF32PlanWitnessError> {
    let count = usize::try_from(contributor_count)
        .ok()
        .and_then(|count| count.checked_mul(2))
        .and_then(|count| count.checked_sub(1))
        .ok_or(ContractionF32PlanWitnessError::Tree(
            ContractionF32TreeError::NodeCountOverflow,
        ))?;
    let mut nodes = Vec::with_capacity(count);
    let mut merge_root: Option<u32> = None;
    for partition in 0..partitions {
        let first = partition * per_partition;
        nodes.push(ContractionF32TreeNode::Leaf { contributor: first });
        let mut partition_root =
            u32::try_from(nodes.len() - 1).expect("node count is bounded above");
        for offset in 1..per_partition {
            nodes.push(ContractionF32TreeNode::Leaf {
                contributor: first + offset,
            });
            let leaf = u32::try_from(nodes.len() - 1).expect("node count is bounded above");
            nodes.push(ContractionF32TreeNode::Add {
                left: partition_root,
                right: leaf,
            });
            partition_root = u32::try_from(nodes.len() - 1).expect("node count is bounded above");
        }
        merge_root = Some(match merge_root {
            None => partition_root,
            Some(left) => {
                nodes.push(ContractionF32TreeNode::Add {
                    left,
                    right: partition_root,
                });
                u32::try_from(nodes.len() - 1).expect("node count is bounded above")
            }
        });
    }
    Ok(nodes)
}

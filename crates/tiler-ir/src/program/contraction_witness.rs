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
//! [`PartialReduction`]: super::PartialReduction

use std::error::Error;
use std::fmt;

use crate::schedule::{
    ContractionF32TopologyLimits, ContractionF32TreeError, ContractionF32TreeNode,
    OrderedContractionF32Tree,
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
/// tree cannot be derived from program scope — including any kernel that
/// declares workgroup staging, whose intra-workgroup combine structure the
/// program does not carry.
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
        // A kernel declaring workgroup staging combines inside the workgroup;
        // the program does not carry that combine structure.
        if covering.kernel().staging().len() != 0 {
            return Err(ContractionF32PlanWitnessError::TopologyUnsupported);
        }

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
            if split.combiner().kernel().staging().len() != 0 {
                return Err(ContractionF32PlanWitnessError::TopologyUnsupported);
            }
            if split.combiner().kernel().numerical() != realization
                || split.producer().kernel().numerical() != realization
            {
                return Err(ContractionF32PlanWitnessError::AmbiguousRealization);
            }
        }

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

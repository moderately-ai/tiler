//! The concrete relaxed reference route: one witnessed tree, evaluated exactly.
//!
//! The ordinary registered evaluator remains strict-only, and
//! [`ReferenceNumericalConformance::from_realization`] refuses a realization
//! permitting reassociation before it can run — so there is no ordinary
//! no-witness relaxed route. This module is the **only** relaxed reference
//! route, and its request always requires a witness: the semantic result set
//! under permitted effective reassociation is a *set*, and set membership is
//! not checkable — a body claiming tree A must be checked against A even when
//! its wrong result happens to equal a value tree B can produce. The evaluator
//! therefore computes exactly the witnessed tree with the descriptor's leaf
//! and reducer primitives, in `O(output_count * K)` time and `O(K)` traversal
//! memory; it never enumerates the Catalan-sized semantic set, and no error
//! defaults to strict evaluation.
//!
//! The caller owns and must pass the budget; the crate never chooses a
//! default. Resource units are exact: retained topology nodes are `2K - 1`
//! once; topology visits are `output_count * (2K - 1)`; arithmetic steps are
//! `output_count * (2K - 1)` combined as one multiplication per leaf and one
//! addition per internal node; and depth is the validated tree depth. Every
//! sum and product uses checked arithmetic, and all four bounds are
//! preflighted before result allocation or arithmetic. The existing
//! tensor-element, retained-byte, extent-binding, and output-window limits
//! remain independently enforced and are not widened by this budget.
//!
//! [`ReferenceNumericalConformance::from_realization`]: crate::ReferenceNumericalConformance::from_realization

use std::error::Error;
use std::fmt;

use tiler_ir::numerics::NumericalDimension;
use tiler_ir::program::{ContractionF32PlanWitness, SemanticOccurrence};
use tiler_ir::schedule::{
    ContractionF32TreeNode, EffectiveContractionF32Profile, ExceptionalValueAssumption,
};
use tiler_ir::semantic::{
    CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, ContractionF32ResultClass, ContractionIndexStructure,
    SemanticProgram, tensor_contraction_f32_op, tensor_contraction_f32_reduction_descriptor,
};

use super::{ContractionContract, ContractionFold};
use crate::conformance::ReferenceNumericalConformance;
use crate::error::ReferenceOperationError;
use crate::extent_bindings::ExtentBindingContext;
use crate::registry::{CanonicalReferenceRegistryIdentity, FrozenReferenceRegistry};
use crate::tensor::Tensor;

/// One budgeted resource of a topology evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractionF32ReferenceResource {
    /// Multiply and add steps across every output coordinate.
    ArithmeticSteps,
    /// Topology nodes retained once for the whole evaluation.
    TopologyNodes,
    /// Topology node visits across every output coordinate.
    TopologyNodeVisits,
    /// The validated tree depth.
    TopologyDepth,
}

impl fmt::Display for ContractionF32ReferenceResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ArithmeticSteps => "arithmetic steps",
            Self::TopologyNodes => "topology nodes",
            Self::TopologyNodeVisits => "topology node visits",
            Self::TopologyDepth => "topology depth",
        })
    }
}

/// Why one budget value is invalid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidContractionF32ReferenceBudget {
    /// One allowance is zero.
    Zero {
        /// The zero allowance.
        resource: ContractionF32ReferenceResource,
    },
    /// The node allowance exceeds the `u32` node-index capacity.
    NodeIndexCapacity {
        /// The maximum admissible node allowance.
        maximum: usize,
    },
}

impl fmt::Display for InvalidContractionF32ReferenceBudget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero { resource } => {
                write!(formatter, "budget allowance for {resource} is zero")
            }
            Self::NodeIndexCapacity { maximum } => write!(
                formatter,
                "budget node allowance exceeds the node-index capacity {maximum}"
            ),
        }
    }
}

impl Error for InvalidContractionF32ReferenceBudget {}

/// The caller-owned resource budget of one topology evaluation.
///
/// No `Default`: the caller states its allowances, and the crate never chooses
/// them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(
    clippy::struct_field_names,
    reason = "the four `max_` fields are the accepted accessor spellings, each an allowance the accessor of the same name returns"
)]
pub struct ContractionF32ReferenceBudget {
    max_arithmetic_steps: u64,
    max_topology_nodes: usize,
    max_topology_node_visits: u64,
    max_topology_depth: usize,
}

impl ContractionF32ReferenceBudget {
    /// Builds one budget.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidContractionF32ReferenceBudget`] for a zero allowance
    /// or a node allowance above `u32::MAX`, because node indices are `u32`.
    pub const fn new(
        max_arithmetic_steps: u64,
        max_topology_nodes: usize,
        max_topology_node_visits: u64,
        max_topology_depth: usize,
    ) -> Result<Self, InvalidContractionF32ReferenceBudget> {
        if max_arithmetic_steps == 0 {
            return Err(InvalidContractionF32ReferenceBudget::Zero {
                resource: ContractionF32ReferenceResource::ArithmeticSteps,
            });
        }
        if max_topology_nodes == 0 {
            return Err(InvalidContractionF32ReferenceBudget::Zero {
                resource: ContractionF32ReferenceResource::TopologyNodes,
            });
        }
        if max_topology_node_visits == 0 {
            return Err(InvalidContractionF32ReferenceBudget::Zero {
                resource: ContractionF32ReferenceResource::TopologyNodeVisits,
            });
        }
        if max_topology_depth == 0 {
            return Err(InvalidContractionF32ReferenceBudget::Zero {
                resource: ContractionF32ReferenceResource::TopologyDepth,
            });
        }
        if max_topology_nodes > u32::MAX as usize {
            return Err(InvalidContractionF32ReferenceBudget::NodeIndexCapacity {
                maximum: u32::MAX as usize,
            });
        }
        Ok(Self {
            max_arithmetic_steps,
            max_topology_nodes,
            max_topology_node_visits,
            max_topology_depth,
        })
    }

    /// Returns the arithmetic-step allowance.
    #[must_use]
    pub const fn max_arithmetic_steps(&self) -> u64 {
        self.max_arithmetic_steps
    }

    /// Returns the retained topology-node allowance.
    #[must_use]
    pub const fn max_topology_nodes(&self) -> usize {
        self.max_topology_nodes
    }

    /// Returns the topology node-visit allowance.
    #[must_use]
    pub const fn max_topology_node_visits(&self) -> u64 {
        self.max_topology_node_visits
    }

    /// Returns the topology depth allowance.
    #[must_use]
    pub const fn max_topology_depth(&self) -> usize {
        self.max_topology_depth
    }
}

/// Why the topology evaluator is unavailable from a frozen registry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractionF32TopologyEvaluatorUnavailable {
    /// The registered successor contraction capability is absent.
    CapabilityMissing,
    /// The capability is owned by a provider other than the standard one.
    ProviderMismatch,
    /// The provider or capability revision is not the one that owns both
    /// strict and topology evaluation.
    RevisionMismatch,
    /// The reached semantic authority is not compatible with the registered
    /// capability's, or the reached governed definition fails the sole
    /// descriptor decoder.
    SemanticAuthorityMismatch,
}

impl fmt::Display for ContractionF32TopologyEvaluatorUnavailable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::CapabilityMissing => "the successor contraction capability is not registered",
            Self::ProviderMismatch => {
                "the contraction capability is not owned by the standard reference provider"
            }
            Self::RevisionMismatch => {
                "the provider or capability revision does not own topology evaluation"
            }
            Self::SemanticAuthorityMismatch => {
                "the reached semantic authority does not match the registered capability"
            }
        })
    }
}

impl Error for ContractionF32TopologyEvaluatorUnavailable {}

/// A typed refusal of one topology evaluation.
///
/// Exhaustive. Effective-profile construction makes forbidden contraction,
/// permutation, signed-zero elimination, and canonical-NaN mismatch
/// unrepresentable here; the evaluator requires the
/// [`ContractionF32ResultClass::OrderedFullBinaryTrees`] result class and
/// refuses exceptional absence assumptions it cannot discharge. No error
/// defaults to strict evaluation.
#[derive(Clone, Debug, PartialEq)]
pub enum ContractionF32TopologyEvaluationError {
    /// The request's semantic subject is not the witness's, or is foreign to
    /// this evaluator's registry authority.
    SemanticSubjectMismatch,
    /// The request's occurrence is not the witness's occurrence.
    OccurrenceMismatch,
    /// The effective profile does not denote the ordered-tree result class.
    ResultClass {
        /// The class this evaluator requires.
        expected: ContractionF32ResultClass,
        /// The profile's derived class.
        actual: ContractionF32ResultClass,
    },
    /// The ceiling assumes an exceptional value absent, which this reference
    /// cannot discharge.
    ExceptionalAssumptionUnsupported {
        /// The assumed dimension.
        dimension: NumericalDimension,
    },
    /// The witness failed revalidation against this request.
    Witness(tiler_ir::program::ContractionF32PlanWitnessError),
    /// One budgeted resource is over its caller allowance.
    BudgetExceeded {
        /// The exceeded resource.
        resource: ContractionF32ReferenceResource,
        /// The caller's allowance.
        limit: u64,
        /// The required amount.
        actual: u64,
    },
    /// A budget quantity overflowed checked arithmetic.
    BudgetArithmeticOverflow {
        /// The overflowed resource.
        resource: ContractionF32ReferenceResource,
    },
    /// The operands, attributes, shapes, or bindings were refused.
    Operation(ReferenceOperationError),
}

impl fmt::Display for ContractionF32TopologyEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SemanticSubjectMismatch => {
                formatter.write_str("the request's semantic subject is not the witness's")
            }
            Self::OccurrenceMismatch => {
                formatter.write_str("the request's occurrence is not the witness's")
            }
            Self::ResultClass { expected, actual } => write!(
                formatter,
                "the effective profile derives {actual:?} where this evaluator requires {expected:?}"
            ),
            Self::ExceptionalAssumptionUnsupported { dimension } => write!(
                formatter,
                "the ceiling assumes {} absent, which this reference cannot discharge",
                dimension.key()
            ),
            Self::Witness(source) => write!(formatter, "witness revalidation failed: {source}"),
            Self::BudgetExceeded {
                resource,
                limit,
                actual,
            } => write!(
                formatter,
                "{resource} {actual} exceeds the caller allowance {limit}"
            ),
            Self::BudgetArithmeticOverflow { resource } => {
                write!(formatter, "{resource} overflowed checked arithmetic")
            }
            Self::Operation(source) => source.fmt(formatter),
        }
    }
}

impl Error for ContractionF32TopologyEvaluationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Witness(source) => Some(source),
            Self::Operation(source) => Some(source),
            _ => None,
        }
    }
}

/// One topology evaluation request, complete and borrowed.
#[derive(Clone, Copy)]
pub struct ContractionF32TopologyEvaluationRequest<'a> {
    semantic: &'a SemanticProgram,
    occurrence: SemanticOccurrence,
    operands: [&'a Tensor; 2],
    extent_bindings: &'a ExtentBindingContext,
    profile: EffectiveContractionF32Profile,
    witness: &'a ContractionF32PlanWitness,
    budget: ContractionF32ReferenceBudget,
}

impl fmt::Debug for ContractionF32TopologyEvaluationRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ContractionF32TopologyEvaluationRequest")
            .field("occurrence", &self.occurrence)
            .field("profile", &self.profile)
            .field("budget", &self.budget)
            .finish()
    }
}

impl<'a> ContractionF32TopologyEvaluationRequest<'a> {
    /// Assembles one complete request.
    #[must_use]
    #[allow(
        clippy::too_many_arguments,
        reason = "every subject of the accepted request shape is an explicit required argument so widening the contract breaks every caller instead of silently defaulting a new obligation"
    )]
    pub const fn new(
        semantic: &'a SemanticProgram,
        occurrence: SemanticOccurrence,
        operands: [&'a Tensor; 2],
        extent_bindings: &'a ExtentBindingContext,
        profile: EffectiveContractionF32Profile,
        witness: &'a ContractionF32PlanWitness,
        budget: ContractionF32ReferenceBudget,
    ) -> Self {
        Self {
            semantic,
            occurrence,
            operands,
            extent_bindings,
            profile,
            witness,
            budget,
        }
    }
}

/// The owned result of one topology evaluation.
#[derive(Clone, Debug)]
pub struct ContractionF32TopologyEvaluation {
    tensor: Tensor,
    reference_registry_identity: CanonicalReferenceRegistryIdentity,
    kernel_program_identity: tiler_ir::program::CanonicalKernelProgramIdentity,
    occurrence: SemanticOccurrence,
}

impl ContractionF32TopologyEvaluation {
    /// Returns the evaluated tensor.
    #[must_use]
    pub const fn tensor(&self) -> &Tensor {
        &self.tensor
    }

    /// Consumes this evaluation, returning the tensor.
    #[must_use]
    pub fn into_tensor(self) -> Tensor {
        self.tensor
    }

    /// Returns the evaluating registry's canonical identity.
    #[must_use]
    pub const fn reference_registry_identity(&self) -> &CanonicalReferenceRegistryIdentity {
        &self.reference_registry_identity
    }

    /// Returns the kernel-program identity bound inside the validated witness.
    ///
    /// The request supplies no second program identity to compare; the
    /// witness's bound identity is the authority.
    #[must_use]
    pub const fn kernel_program_identity(
        &self,
    ) -> &tiler_ir::program::CanonicalKernelProgramIdentity {
        &self.kernel_program_identity
    }

    /// Returns the evaluated occurrence.
    #[must_use]
    pub const fn occurrence(&self) -> SemanticOccurrence {
        self.occurrence
    }
}

/// The concrete, non-extensible first-vertical topology evaluator.
///
/// Its private constructor is reached only by
/// [`FrozenReferenceRegistry::contraction_f32_topology_evaluator`], which
/// requires the registered successor F32 signature, compatible reached
/// semantic authority, and the standard reference provider and capability
/// revision that own both strict and topology evaluation. This adds no
/// unversioned callback and no second general registry role; third-party
/// topology-reference providers are outside the first surface, and adding
/// them later requires its own registry-role and identity review.
pub struct ContractionF32TopologyEvaluator {
    registry: FrozenReferenceRegistry,
    contract: ContractionContract,
}

impl fmt::Debug for ContractionF32TopologyEvaluator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ContractionF32TopologyEvaluator")
            .finish_non_exhaustive()
    }
}

impl FrozenReferenceRegistry {
    /// Returns the concrete contraction topology evaluator, when this
    /// registry's standard provider owns it.
    ///
    /// # Errors
    ///
    /// Returns [`ContractionF32TopologyEvaluatorUnavailable`] naming the
    /// missing authority: an absent successor capability, a foreign owning
    /// provider, a revision other than the one owning topology evaluation, or
    /// incompatible reached semantic authority.
    pub fn contraction_f32_topology_evaluator(
        &self,
    ) -> Result<ContractionF32TopologyEvaluator, ContractionF32TopologyEvaluatorUnavailable> {
        let (provider, revision, authority) = self
            .contraction_capability_provenance()
            .ok_or(ContractionF32TopologyEvaluatorUnavailable::CapabilityMissing)?;
        if provider.namespace() != "tiler" || provider.name() != "standard-reference" {
            return Err(ContractionF32TopologyEvaluatorUnavailable::ProviderMismatch);
        }
        // Provider revision 8 is the one whose behaviour includes topology
        // evaluation, and capability revision 8 is the successor row.
        if provider.revision() != 8 || revision.get() != 8 {
            return Err(ContractionF32TopologyEvaluatorUnavailable::RevisionMismatch);
        }
        let expected = self
            .semantic_registry()
            .project_operation_authority(
                &tensor_contraction_f32_op(),
                authority_signature().operands(),
                authority_signature().results(),
            )
            .map_err(|_| ContractionF32TopologyEvaluatorUnavailable::SemanticAuthorityMismatch)?;
        if expected.reached_definitions() != authority.reached_definitions()
            || expected.admission_provenance() != authority.admission_provenance()
        {
            return Err(ContractionF32TopologyEvaluatorUnavailable::SemanticAuthorityMismatch);
        }
        // The strict-cell contract, through the sole decoder, from the exact
        // semantic registry this reference registry is built against. A
        // decode failure means the reached governed definition is not the
        // accepted contract — unreachable through any frozen registry, which
        // the semantic registrar guards, and refused as a semantic-authority
        // mismatch rather than assumed away.
        let contract = ContractionContract::from_registry(self.semantic_registry())
            .map_err(|_| ContractionF32TopologyEvaluatorUnavailable::SemanticAuthorityMismatch)?;
        Ok(ContractionF32TopologyEvaluator {
            registry: self.clone(),
            contract,
        })
    }
}

/// The successor capability's exact binary F32 signature.
fn authority_signature() -> crate::registry::ReferenceSignature {
    use tiler_ir::semantic::F32;
    crate::registry::ReferenceSignature::new(
        [F32::resolved_type(), F32::resolved_type()],
        [F32::resolved_type()],
    )
    .expect("the governed binary F32 signature is bounded")
}

impl ContractionF32TopologyEvaluator {
    /// Evaluates exactly the witnessed tree for one occurrence.
    ///
    /// The evaluator rechecks registry authority, the request's semantic graph
    /// and occurrence against the witness, the descriptor-derived profile, the
    /// occurrence's attributes, the operands, the live bindings, and the
    /// witness before any arithmetic. Evaluation then computes the selected
    /// tree with the descriptor's leaf and reducer primitives under the
    /// profile's ceiling transforms, in `O(output_count * K)` time and `O(K)`
    /// traversal memory.
    ///
    /// # Errors
    ///
    /// Returns [`ContractionF32TopologyEvaluationError`] naming the first
    /// failed validation or exceeded budget. No error defaults to strict
    /// evaluation.
    pub fn evaluate(
        &self,
        request: ContractionF32TopologyEvaluationRequest<'_>,
    ) -> Result<ContractionF32TopologyEvaluation, ContractionF32TopologyEvaluationError> {
        let ContractionF32TopologyEvaluationRequest {
            semantic,
            occurrence,
            operands,
            extent_bindings,
            profile,
            witness,
            budget,
        } = request;

        // Semantic subjects: the request's program must be the witness's
        // graph, under this evaluator's own registry authority.
        if semantic.semantic_identity().graph() != witness.semantic_graph_identity() {
            return Err(ContractionF32TopologyEvaluationError::SemanticSubjectMismatch);
        }
        if semantic.semantic_identity().registry_snapshot()
            != self.registry.semantic_registry().snapshot_identity()
        {
            return Err(ContractionF32TopologyEvaluationError::SemanticSubjectMismatch);
        }
        if occurrence != witness.occurrence() {
            return Err(ContractionF32TopologyEvaluationError::OccurrenceMismatch);
        }

        // Effective profile: this route answers the ordered-tree result class
        // only; the strict cell belongs to the ordinary evaluator.
        if profile.result_class() != ContractionF32ResultClass::OrderedFullBinaryTrees {
            return Err(ContractionF32TopologyEvaluationError::ResultClass {
                expected: ContractionF32ResultClass::OrderedFullBinaryTrees,
                actual: profile.result_class(),
            });
        }
        let ceiling = profile.ceiling();
        if let ExceptionalValueAssumption::AssumeAbsent { .. } = ceiling.nan_assumptions {
            return Err(
                ContractionF32TopologyEvaluationError::ExceptionalAssumptionUnsupported {
                    dimension: NumericalDimension::NanAssumptions,
                },
            );
        }
        if let ExceptionalValueAssumption::AssumeAbsent { .. } = ceiling.infinity_assumptions {
            return Err(
                ContractionF32TopologyEvaluationError::ExceptionalAssumptionUnsupported {
                    dimension: NumericalDimension::InfinityAssumptions,
                },
            );
        }
        // The descriptor is re-decoded from this evaluator's own semantic
        // authority, so the profile cannot have been resolved against a
        // foreign descriptor: every decoded descriptor of the governed key is
        // the one accepted contract, and the resolver already required the
        // ceiling's canonical NaN payload to match it.
        let descriptor =
            tensor_contraction_f32_reduction_descriptor(self.registry.semantic_registry())
                .map_err(|_| ContractionF32TopologyEvaluationError::SemanticSubjectMismatch)?;
        debug_assert_eq!(
            descriptor.canonical_nan_bits(),
            ceiling.canonical_arithmetic_nan_bits
        );

        // The occurrence, its structure, and its declared operand shapes.
        let Some(operation) = semantic
            .operations()
            .find(|operation| semantic.canonical_operation_ordinal(*operation) == occurrence.get())
        else {
            return Err(ContractionF32TopologyEvaluationError::OccurrenceMismatch);
        };
        if operation.key() != &tensor_contraction_f32_op() {
            return Err(ContractionF32TopologyEvaluationError::Operation(
                ReferenceOperationError::InvalidApplication,
            ));
        }
        let structure = operation
            .attributes()
            .get(CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE)
            .and_then(|value| ContractionIndexStructure::from_canonical_value(value).ok())
            .ok_or(ContractionF32TopologyEvaluationError::Operation(
                ReferenceOperationError::InvalidApplication,
            ))?;
        // Every declared operand extent resolves through the authenticated
        // bindings and must equal the offered tensor's, so a tensor standing
        // in for a different occurrence is refused before arithmetic.
        for (value, tensor) in operation.operands().zip(operands) {
            let declared = semantic.shape(value).map_err(|_| {
                ContractionF32TopologyEvaluationError::Operation(
                    ReferenceOperationError::InvalidApplication,
                )
            })?;
            if declared.rank() != tensor.shape().rank() {
                return Err(ContractionF32TopologyEvaluationError::Operation(
                    ReferenceOperationError::InvalidApplication,
                ));
            }
            for (declared, actual) in declared.extents().zip(tensor.shape().extents()) {
                let resolved = extent_bindings.resolve(&declared).map_err(|_| {
                    ContractionF32TopologyEvaluationError::Operation(
                        ReferenceOperationError::InvalidApplication,
                    )
                })?;
                if resolved != actual.get() {
                    return Err(ContractionF32TopologyEvaluationError::Operation(
                        ReferenceOperationError::InvalidApplication,
                    ));
                }
            }
        }

        // The fold plan: validates operand types, extents, structure
        // agreement, and the output preflight, under the crate's existing
        // limits.
        let [left, right] = operands;
        let fold = ContractionFold::plan(&self.contract, &structure, left, right)
            .map_err(ContractionF32TopologyEvaluationError::Operation)?;

        // Witness revalidation against this occurrence's derived K.
        let tree = witness.tree();
        let contributor_count = u64::try_from(fold.contracted_count).unwrap_or(u64::MAX);
        if tree.contributor_count() != contributor_count {
            return Err(ContractionF32TopologyEvaluationError::Witness(
                tiler_ir::program::ContractionF32PlanWitnessError::Tree(
                    tiler_ir::schedule::ContractionF32TreeError::RootCoverage {
                        expected: contributor_count,
                        actual_start: 0,
                        actual_end: tree.contributor_count(),
                    },
                ),
            ));
        }

        // Budget preflight, all four bounds before allocation or arithmetic.
        let nodes = tree.nodes().len();
        let node_count = u64::try_from(nodes).map_err(|_| {
            ContractionF32TopologyEvaluationError::BudgetArithmeticOverflow {
                resource: ContractionF32ReferenceResource::TopologyNodes,
            }
        })?;
        if nodes > budget.max_topology_nodes() {
            return Err(ContractionF32TopologyEvaluationError::BudgetExceeded {
                resource: ContractionF32ReferenceResource::TopologyNodes,
                limit: u64::try_from(budget.max_topology_nodes()).unwrap_or(u64::MAX),
                actual: node_count,
            });
        }
        if tree.depth() > budget.max_topology_depth() {
            return Err(ContractionF32TopologyEvaluationError::BudgetExceeded {
                resource: ContractionF32ReferenceResource::TopologyDepth,
                limit: u64::try_from(budget.max_topology_depth()).unwrap_or(u64::MAX),
                actual: u64::try_from(tree.depth()).unwrap_or(u64::MAX),
            });
        }
        let output_count = u64::try_from(fold.output_count).map_err(|_| {
            ContractionF32TopologyEvaluationError::BudgetArithmeticOverflow {
                resource: ContractionF32ReferenceResource::TopologyNodeVisits,
            }
        })?;
        let visits = output_count.checked_mul(node_count).ok_or(
            ContractionF32TopologyEvaluationError::BudgetArithmeticOverflow {
                resource: ContractionF32ReferenceResource::TopologyNodeVisits,
            },
        )?;
        if visits > budget.max_topology_node_visits() {
            return Err(ContractionF32TopologyEvaluationError::BudgetExceeded {
                resource: ContractionF32ReferenceResource::TopologyNodeVisits,
                limit: budget.max_topology_node_visits(),
                actual: visits,
            });
        }
        // One multiplication per leaf and one addition per internal node is
        // exactly one arithmetic step per node.
        let steps = visits;
        if steps > budget.max_arithmetic_steps() {
            return Err(ContractionF32TopologyEvaluationError::BudgetExceeded {
                resource: ContractionF32ReferenceResource::ArithmeticSteps,
                limit: budget.max_arithmetic_steps(),
                actual: steps,
            });
        }

        // The conformance realizing the ceiling's two subnormal dimensions.
        // Deliberately not `from_realization`, whose refusal of a permitted
        // reassociation is exactly what makes this route the only relaxed one;
        // the order freedoms are already governed by the effective profile.
        let conformance =
            ReferenceNumericalConformance::new(ceiling.input_subnormals, ceiling.result_subnormals);
        let results = fold
            .evaluate_every_output_tree(&self.contract, conformance, tree)
            .map_err(ContractionF32TopologyEvaluationError::Operation)?;
        let tensor = Tensor::dense(
            tiler_ir::semantic::F32::resolved_type(),
            fold.output_shape.clone(),
            results,
        )
        .map_err(|source| {
            ContractionF32TopologyEvaluationError::Operation(crate::error::dense_result_error(
                &source,
            ))
        })?;

        Ok(ContractionF32TopologyEvaluation {
            tensor,
            reference_registry_identity: self.registry.canonical_identity().clone(),
            kernel_program_identity: witness.kernel_program_identity().clone(),
            occurrence,
        })
    }
}

impl ContractionFold<'_> {
    /// Folds every output element along one validated tree.
    ///
    /// Every leaf is the same `P` and every internal node the same `A` the
    /// strict fold applies — one rounding for the product, one for each add,
    /// the canonical NaN committed after each and at the boundary, and both
    /// subnormal dimensions at their declared sites. What the tree changes is
    /// only the grouping of the adds, which is exactly the freedom the
    /// ordered-tree result class denotes.
    pub(super) fn evaluate_every_output_tree(
        &self,
        contract: &ContractionContract,
        conformance: ReferenceNumericalConformance,
        tree: &tiler_ir::schedule::OrderedContractionF32Tree,
    ) -> Result<Vec<crate::tensor::ReferenceElement>, ReferenceOperationError> {
        use crate::evaluate::{decode_coordinate, decode_f32, f32_element};

        let nodes = tree.nodes();
        let mut output_coordinate = vec![0_usize; self.output_shape.rank()];
        let mut contracted_coordinate = vec![0_usize; self.contracted_shape.rank()];
        let mut values = vec![0.0_f32; nodes.len()];
        let mut results = Vec::with_capacity(self.output_count);
        for output_linear in 0..self.output_count {
            decode_coordinate(
                output_linear,
                &self.output_shape,
                &self.output_strides,
                &mut output_coordinate,
            )?;
            for (slot, node) in nodes.iter().enumerate() {
                let value = match node {
                    ContractionF32TreeNode::Leaf { contributor } => {
                        let contracted_linear = usize::try_from(*contributor)
                            .map_err(|_| ReferenceOperationError::ShapeTooLarge)?;
                        decode_coordinate(
                            contracted_linear,
                            &self.contracted_shape,
                            &self.contracted_strides,
                            &mut contracted_coordinate,
                        )?;
                        let mut factors = [0.0_f32; 2];
                        for (position, factor) in factors.iter_mut().enumerate() {
                            let offset = self.readers[position]
                                .iter()
                                .zip(&self.operand_strides[position])
                                .try_fold(0_usize, |offset, (reader, stride)| {
                                    let coordinate = match reader {
                                        super::AxisReader::Output(axis) => output_coordinate[*axis],
                                        super::AxisReader::Contracted(axis) => {
                                            contracted_coordinate[*axis]
                                        }
                                    };
                                    coordinate
                                        .checked_mul(*stride)
                                        .and_then(|scaled| offset.checked_add(scaled))
                                        .ok_or(ReferenceOperationError::ShapeTooLarge)
                                })?;
                            let element = self.elements[position]
                                .get(offset)
                                .ok_or(ReferenceOperationError::InvalidApplication)?;
                            *factor = conformance.apply_to_operand(decode_f32(element)?);
                        }
                        conformance.apply_to_result(contract.canonicalize(factors[0] * factors[1]))
                    }
                    ContractionF32TreeNode::Add { left, right } => {
                        let left = values[*left as usize];
                        let right = values[*right as usize];
                        conformance.apply_to_result(contract.canonicalize(
                            conformance.apply_to_operand(left)
                                + conformance.apply_to_operand(right),
                        ))
                    }
                };
                values[slot] = value;
            }
            let value = values[tree.root() as usize];
            results.push(f32_element(contract.canonicalize(value))?);
        }
        Ok(results)
    }
}

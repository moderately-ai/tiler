//! The ordered associations a checked realization carries.
//!
//! A binding is what makes a receipt say *which* semantic value each verified
//! boundary answers. The operand side expands encoded compound inputs into
//! their contract components and records one entry per (operand use, expanded
//! component, reading stage); the result side groups a region's output roots by
//! the tensor they write and records one entry per root, so a partitioned
//! output states every member rather than electing one. Both populations are
//! counted against their ceilings before either vector is allocated.

use crate::index::{
    MAX_BOUNDARY_TENSORS, StagedInputSource, TensorRole, VerifiedIndexRegion,
    VerifiedIndexRegionSequence, VerifiedScalarValueId, VerifiedTensorAccessId, VerifiedTensorId,
};
use crate::semantic::{EncodedComponentRole, ResolvedValueType};

use super::MAX_INDEX_REFINEMENT_OPERAND_BINDINGS;
use super::error::IndexRefinementVerificationError;
use super::subject::{IndexRefinementBoundary, IndexRefinementSubject};

/// One ordered operand projection bound to its verified region input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OperandBinding {
    pub(super) stage: usize,
    pub(super) operand: usize,
    pub(super) input: usize,
    pub(super) input_tensor: VerifiedTensorId,
    pub(super) component_role: Option<EncodedComponentRole>,
}

impl OperandBinding {
    /// Returns the ordered realization stage that reads this operand.
    ///
    /// A tensor handle is region-local, so this is what says which stage's
    /// region [`Self::input_tensor`] resolves against. One occurrence input read
    /// by two stages produces one binding per reading stage.
    #[must_use]
    pub const fn stage(&self) -> usize {
        self.stage
    }
    /// Returns the ordered operand position.
    #[must_use]
    pub const fn operand(&self) -> usize {
        self.operand
    }
    /// Returns the occurrence-local semantic value.
    #[must_use]
    pub const fn input(&self) -> usize {
        self.input
    }
    /// Returns the verified input tensor carrying the value.
    #[must_use]
    pub const fn input_tensor(&self) -> VerifiedTensorId {
        self.input_tensor
    }
    /// Returns the encoded logical component carried by this input tensor.
    ///
    /// `None` names an ordinary whole-value input. An encoded operand produces
    /// one binding per component in its contract's semantic order.
    #[must_use]
    pub const fn component_role(&self) -> Option<EncodedComponentRole> {
        self.component_role
    }
}

/// One ordered result bound to one verified output root that writes it.
///
/// A result whose output is written whole by a single root has exactly one
/// binding, which is every realization the closed law vocabulary produces. A
/// result whose output is *partitioned* — several roots, each total over its own
/// declared partition and the partitions jointly disjoint and covering — has one
/// binding per member, all carrying the same [`Self::result`] and
/// [`Self::output_tensor`] and each carrying its own write. So this is one
/// binding per root and not one per result, the same way an
/// [`OperandBinding`] is one per reading stage and not one per operand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResultBinding {
    pub(super) result: usize,
    pub(super) output_tensor: VerifiedTensorId,
    pub(super) write_access: VerifiedTensorAccessId,
    pub(super) written_value: VerifiedScalarValueId,
}

impl ResultBinding {
    /// Returns the ordered result position.
    ///
    /// Repeated across the bindings of a partitioned result, which is what
    /// groups its members: two bindings agreeing here name two writes of one
    /// semantic result.
    #[must_use]
    pub const fn result(&self) -> usize {
        self.result
    }
    /// Returns the verified output tensor.
    #[must_use]
    pub const fn output_tensor(&self) -> VerifiedTensorId {
        self.output_tensor
    }
    /// Returns this root's proved write.
    ///
    /// It is total over the whole output when the result has one binding, and
    /// total over this member's declared partition when it has several; the
    /// region's own [`WriteOwnershipProofView`](crate::index::WriteOwnershipProofView) says
    /// which, and refinement admits no root carrying neither.
    #[must_use]
    pub const fn write_access(&self) -> VerifiedTensorAccessId {
        self.write_access
    }
    /// Returns the scalar value written by the output root.
    #[must_use]
    pub const fn written_value(&self) -> VerifiedScalarValueId {
        self.written_value
    }
}

/// One expanded semantic input boundary and the type and shape it demands.
///
/// An ordinary input expands to one entry; an encoded compound input expands to
/// one entry per component in its contract order, each carrying that component's
/// own resolved type and derived shape.
struct ExpandedInput {
    input: usize,
    component_role: Option<EncodedComponentRole>,
    value_type: ResolvedValueType,
    sourced: crate::shape::SourcedShape,
}

pub(super) fn bind_operands(
    occurrence: &IndexRefinementSubject,
    realization: &VerifiedIndexRegionSequence,
) -> Result<Vec<OperandBinding>, IndexRefinementVerificationError> {
    let region_inputs = realization
        .stages()
        .map(|stage| {
            stage
                .tensors()
                .filter(|tensor| tensor.role() == TensorRole::Input)
                .count()
        })
        .try_fold(0_usize, usize::checked_add)
        .unwrap_or(usize::MAX);
    let expanded = expand_inputs(&occurrence.inputs, region_inputs)?;

    // Every stage input sourced from the occurrence is checked against the
    // boundary it claims, and the tensor is recorded against that boundary. One
    // boundary can be claimed by several stages — a value a fold reads and the
    // pass consuming the fold reads again is the motivating case — so this is a
    // list per boundary rather than a single tensor.
    let mut physical_by_expanded = vec![Vec::new(); expanded.len()];
    for (stage, region) in realization.stages().enumerate() {
        let inputs = region
            .tensors()
            .filter(|tensor| tensor.role() == TensorRole::Input)
            .collect::<Vec<_>>();
        let sources = realization.stage_sources(stage).ok_or(
            IndexRefinementVerificationError::OperandArity {
                region_inputs,
                expanded_inputs: expanded.len(),
            },
        )?;
        for (slot, source) in sources.iter().enumerate() {
            let StagedInputSource::Occurrence(position) = source else {
                // An intermediate is the sequence's own value: it binds to no
                // semantic operand, and the chain check already proved it agrees
                // with the boundary that produced it.
                continue;
            };
            let boundary =
                expanded
                    .get(*position)
                    .ok_or(IndexRefinementVerificationError::OperandArity {
                        region_inputs,
                        expanded_inputs: expanded.len(),
                    })?;
            let input = inputs[slot];
            if input.value_type() != &boundary.value_type || input.shape() != &boundary.sourced {
                return Err(IndexRefinementVerificationError::OperandInterface {
                    position: *position,
                });
            }
            physical_by_expanded[*position].push((stage, input.id()));
        }
    }
    // A declared boundary no stage reads is an arity disagreement, not a silent
    // omission: the occurrence states an input the realization never consumes.
    // Reported as arity rather than interface because nothing disagreed about
    // the boundary — there was no tensor to disagree with it.
    if physical_by_expanded.iter().any(Vec::is_empty) {
        return Err(IndexRefinementVerificationError::OperandArity {
            region_inputs,
            expanded_inputs: expanded.len(),
        });
    }

    let component_counts = occurrence
        .inputs
        .iter()
        .enumerate()
        .map(|(input, _)| {
            expanded
                .iter()
                .enumerate()
                .filter(|(_, boundary)| boundary.input == input)
                .map(|(position, _)| physical_by_expanded[position].len())
                .try_fold(0_usize, usize::checked_add)
                .unwrap_or(usize::MAX)
        })
        .collect::<Vec<_>>();
    let binding_count = count_operand_bindings(&occurrence.operands, &component_counts)?;
    let mut bindings = Vec::with_capacity(binding_count);
    for (position, input) in occurrence.operands.iter().copied().enumerate() {
        for (expanded_position, boundary) in expanded.iter().enumerate() {
            if boundary.input != input {
                continue;
            }
            for (stage, input_tensor) in &physical_by_expanded[expanded_position] {
                bindings.push(OperandBinding {
                    stage: *stage,
                    operand: position,
                    input,
                    input_tensor: *input_tensor,
                    component_role: boundary.component_role,
                });
            }
        }
    }
    debug_assert_eq!(bindings.len(), binding_count);
    Ok(bindings)
}

/// Expands semantic inputs to the ordered boundary list a realization sources.
fn expand_inputs(
    inputs: &[IndexRefinementBoundary],
    region_inputs: usize,
) -> Result<Vec<ExpandedInput>, IndexRefinementVerificationError> {
    let expanded_inputs = count_expanded_inputs(inputs, region_inputs)?;
    let mut expanded = Vec::with_capacity(expanded_inputs);
    for (input, boundary) in inputs.iter().enumerate() {
        if let Some((_, contract)) = boundary.value_type.encoded_numeric_parts() {
            for component in contract.components() {
                let shape = component.shape_relation().component_shape(&boundary.shape);
                expanded.push(ExpandedInput {
                    input,
                    component_role: Some(component.role()),
                    value_type: component.resolved_type().clone(),
                    sourced: crate::shape::SourcedShape::from(shape),
                });
            }
        } else {
            expanded.push(ExpandedInput {
                input,
                component_role: None,
                value_type: boundary.value_type.clone(),
                sourced: boundary.sourced.clone(),
            });
        }
    }
    debug_assert_eq!(expanded.len(), expanded_inputs);
    Ok(expanded)
}

/// Counts component-expanded semantic inputs without deriving component shapes.
///
/// The verified-region boundary ceiling is the authoritative retained
/// population bound. Counting first prevents a wide signature of maximum-size
/// encoded contracts from multiplying component-shape allocations before the
/// arity mismatch is known.
pub(super) fn count_expanded_inputs(
    inputs: &[IndexRefinementBoundary],
    region_inputs: usize,
) -> Result<usize, IndexRefinementVerificationError> {
    let mut expanded_inputs = 0_usize;
    for (input, boundary) in inputs.iter().enumerate() {
        let contribution = if let Some((_, contract)) = boundary.value_type.encoded_numeric_parts()
        {
            if contract.components().is_empty() {
                return Err(
                    IndexRefinementVerificationError::EmptyEncodedOperandComponents { input },
                );
            }
            contract.components().len()
        } else {
            1
        };
        expanded_inputs = expanded_inputs.saturating_add(contribution);
    }
    if expanded_inputs > MAX_BOUNDARY_TENSORS {
        return Err(IndexRefinementVerificationError::OperandArity {
            region_inputs,
            expanded_inputs,
        });
    }
    Ok(expanded_inputs)
}

/// Counts final operand-use bindings before allocating the retained receipt
/// population.
pub(super) fn count_operand_bindings(
    operands: &[usize],
    component_counts: &[usize],
) -> Result<usize, IndexRefinementVerificationError> {
    let mut bindings = 0_usize;
    for input in operands {
        let contribution = component_counts.get(*input).copied().unwrap_or(usize::MAX);
        bindings = bindings.saturating_add(contribution);
    }
    if bindings > MAX_INDEX_REFINEMENT_OPERAND_BINDINGS {
        return Err(IndexRefinementVerificationError::OperandBindingsTooLarge {
            actual: bindings,
            limit: MAX_INDEX_REFINEMENT_OPERAND_BINDINGS,
        });
    }
    Ok(bindings)
}

/// Binds each ordered semantic result to every output root that writes it.
///
/// `region.outputs()` counts *roots*, and the partitioned write-ownership
/// contract admits several roots over one output tensor, each total over its own
/// declared partition of it. So the ordered population a result is matched
/// against is the region's distinct output *tensors*, and the roots writing one
/// of them are that result's partition members. Comparing root count against
/// result count instead would refuse a well-formed partitioned region as an
/// arity mismatch, which is a refusal for the wrong reason: the write-ownership
/// obligation such a region owes is discharged by
/// [`WriteOwnershipProofView::PartitionMember`](crate::index::WriteOwnershipProofView),
/// not by there being one root.
///
/// **Every member gets its own binding, and the result position is what
/// repeats.** This is the shape [`bind_operands`] already gives the same
/// question on the operand side, where one occurrence input read by two stages
/// produces one binding per reading stage. The alternative — one binding
/// carrying a set of accesses — would state the same association while changing
/// a public type no consumer reads per-root, and picking one member to name
/// would make the receipt a claim about a write the region does not make alone.
///
/// A result owning exactly one root binds to exactly what it bound to before
/// partitions existed: first-encounter tensor order is root order, each group
/// has one member, and the binding's four fields are unchanged. That is what
/// keeps every pinned executable-coverage identity — which encodes these
/// bindings — byte-identical.
pub(super) fn bind_results(
    occurrence: &IndexRefinementSubject,
    region: &VerifiedIndexRegion,
) -> Result<Vec<ResultBinding>, IndexRefinementVerificationError> {
    let roots = region.outputs().collect::<Vec<_>>();
    // Distinct output tensors in first-encounter order, each with the ordinals
    // of the roots writing it. Roots over one tensor need not be authored
    // adjacently, so membership is resolved by tensor rather than by run.
    let mut outputs: Vec<VerifiedTensorId> = Vec::new();
    let mut members: Vec<Vec<usize>> = Vec::new();
    for (ordinal, root) in roots.iter().enumerate() {
        let tensor = region.access(root.access())?.tensor();
        if let Some(position) = outputs.iter().position(|bound| *bound == tensor) {
            members[position].push(ordinal);
        } else {
            outputs.push(tensor);
            members.push(vec![ordinal]);
        }
    }
    if outputs.len() != occurrence.results.len() {
        return Err(IndexRefinementVerificationError::ResultArity {
            region_outputs: outputs.len(),
            results: occurrence.results.len(),
        });
    }
    // Bounded by the region's own `MAX_OUTPUT_ROOTS` ceiling, which is why this
    // population needs no separate receipt limit the way alias-expanded operand
    // bindings do.
    let mut bindings = Vec::with_capacity(roots.len());
    for (position, (tensor, result)) in outputs.iter().zip(&occurrence.results).enumerate() {
        // Every member owns its own write before the shared boundary is
        // compared, so a partition with an unproved member is reported as the
        // incomplete write it is rather than as a boundary disagreement.
        for ordinal in &members[position] {
            let access = region.access(roots[*ordinal].access())?;
            if access.write_ownership_proof().is_none() {
                return Err(IndexRefinementVerificationError::IncompleteWrite { position });
            }
        }
        let output = region.tensor(*tensor)?;
        if output.role() != TensorRole::Output
            || output.value_type() != &result.value_type
            || output.shape() != result.sourced_shape()
        {
            return Err(IndexRefinementVerificationError::ResultInterface { position });
        }
        for ordinal in &members[position] {
            let root = roots[*ordinal];
            let written = region.scalar_value(root.value())?;
            if written.value_type() != &result.value_type {
                return Err(IndexRefinementVerificationError::ResultValueType { position });
            }
            bindings.push(ResultBinding {
                result: position,
                output_tensor: output.id(),
                write_access: root.access(),
                written_value: root.value(),
            });
        }
    }
    Ok(bindings)
}

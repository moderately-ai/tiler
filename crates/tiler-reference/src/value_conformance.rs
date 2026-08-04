//! Reference-tensor logical views, and the conformance the evaluator holds.
//!
//! The compiler core owns what a bound value must satisfy; this module owns how
//! a host reference [`Tensor`] presents its authoritative logical view to that
//! check. Nothing here restates a value domain: the domains come from the
//! [`ResolvedValueConformanceContract`] derived from the value's own type, so
//! narrowing a scheme's code range or scale domain narrows this path with
//! nothing to update here.
//!
//! # Why the evaluator holds proofs rather than revalidating
//!
//! The evaluator binds every input and then evaluates operations whose results
//! it must also trust. A directly bound input is scanned once, and a value the
//! evaluator *produced* is not scanned at all: its conformance composes from
//! the operands it carried through, the occurrence's discharged preconditions,
//! and the operation's own declared semantics. Rescanning a produced value
//! would recompute a fact the producer already established, and would answer a
//! different question the moment the two disagreed — which is exactly the case
//! where a silent second authority is worst.
//!
//! A host [`Tensor`] is immutable for the life of an evaluation, so every
//! subject here is [`ValueStability::ImmutableHost`]: there is no version or
//! coherence epoch to name, and inventing one would be a claim about a
//! mutability this representation does not have.

use std::collections::HashMap;

use tiler_ir::semantic::{
    ComposedOperand, ConformanceValidatorIdentity, DENSE_VALUE_COMPONENT_ROLE,
    EncodedComponentRole, EncodedLogicalView, F32, InputKey, LogicalScalar, LogicalViewFault,
    OpKey, OperationRef, PresentedComponent, ResolvedValueConformanceContract, ResolvedValueType,
    RouteDependency, STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE, SemanticLogicalView,
    SemanticPreconditionRef, SemanticPreconditionStatus, SemanticPreconditionsDischarged,
    SemanticProgram, U4, U8, ValueConformanceEvidence, ValueConformanceSubject, ValueId,
    ValueOrigin, ValueStability, assemble_strict_affine_op, compose_produced_conformance,
    quantize_strict_affine_op, scan_bound_value, standard_binding_validator,
};
use tiler_ir::shape::Shape;

use super::error::EvaluationError;
use super::tensor::{Tensor, TensorPayloadView};

/// One reference tensor presented as an authoritative logical view.
///
/// A compound tensor presents its ordered components exactly as it holds them —
/// including a wrong role, a missing one, or one belonging to another value,
/// because the point of the view is to present what the binding claims and let
/// the contract decide. A dense tensor presents one component under
/// [`DENSE_VALUE_COMPONENT_ROLE`].
#[derive(Clone, Copy, Debug)]
pub struct TensorLogicalView<'a> {
    tensor: &'a Tensor,
}

impl<'a> TensorLogicalView<'a> {
    /// Presents one reference tensor as a logical view.
    #[must_use]
    pub const fn new(tensor: &'a Tensor) -> Self {
        Self { tensor }
    }

    fn component(self, position: usize) -> Option<&'a Tensor> {
        match self.tensor.payload() {
            TensorPayloadView::Dense(_) => (position == 0).then_some(self.tensor),
            TensorPayloadView::Compound(components) => components
                .get(position)
                .map(super::tensor::ReferenceComponent::tensor),
        }
    }
}

impl EncodedLogicalView for TensorLogicalView<'_> {
    fn presented_components(&self) -> usize {
        match self.tensor.payload() {
            TensorPayloadView::Dense(_) => 1,
            TensorPayloadView::Compound(components) => components.len(),
        }
    }

    fn presented_component(&self, position: usize) -> Option<PresentedComponent<'_>> {
        let role = match self.tensor.payload() {
            TensorPayloadView::Dense(_) => DENSE_VALUE_COMPONENT_ROLE,
            TensorPayloadView::Compound(components) => components.get(position)?.role(),
        };
        let component = self.component(position)?;
        Some(PresentedComponent {
            role,
            resolved_type: component.resolved_type(),
            shape: component.shape(),
        })
    }

    fn read_logical_scalar(
        &self,
        position: usize,
        index: u64,
    ) -> Result<LogicalScalar, LogicalViewFault> {
        let component = self
            .component(position)
            .ok_or(LogicalViewFault::UnreconstructableIndex)?;
        let TensorPayloadView::Dense(elements) = component.payload() else {
            // A component that is itself compound has no logical scalar at this
            // index; nested encoded values are refused at contract derivation,
            // and this is the same refusal reached through the payload.
            return Err(LogicalViewFault::UnreconstructableIndex);
        };
        let position =
            usize::try_from(index).map_err(|_| LogicalViewFault::UnreconstructableIndex)?;
        let bytes = elements
            .get(position)
            .ok_or(LogicalViewFault::UnreconstructableIndex)?
            .as_bytes();
        logical_scalar(component.resolved_type(), bytes)
    }
}

/// Reads one canonical element payload as the logical scalar its type names.
///
/// The width comes from the component's own resolved type rather than from the
/// byte run's length, so a payload of the wrong width is refused instead of
/// being reinterpreted as a value of some other type that happens to fit.
fn logical_scalar(
    resolved_type: &ResolvedValueType,
    bytes: &[u8],
) -> Result<LogicalScalar, LogicalViewFault> {
    if resolved_type == &U4::resolved_type() || resolved_type == &U8::resolved_type() {
        let [code] = bytes else {
            return Err(LogicalViewFault::UnrepresentableScalar);
        };
        return Ok(LogicalScalar::UnsignedCode(*code));
    }
    if resolved_type == &F32::resolved_type() {
        let bits: [u8; 4] = bytes
            .try_into()
            .map_err(|_| LogicalViewFault::UnrepresentableScalar)?;
        return Ok(LogicalScalar::F32Bits(u32::from_be_bytes(bits)));
    }
    Err(LogicalViewFault::UnrepresentableScalar)
}

/// The conformance proofs one evaluation holds, keyed by the value they prove.
///
/// It is deliberately not a cache: an entry is only ever read back for the
/// exact value it was minted for, and every read re-checks the subject the
/// proof authorizes rather than trusting the key it was filed under.
#[derive(Debug, Default)]
pub(crate) struct ValueConformanceLedger {
    proofs: HashMap<ValueId, ValueConformanceEvidence>,
}

impl ValueConformanceLedger {
    pub(crate) fn validator() -> ConformanceValidatorIdentity {
        standard_binding_validator()
    }

    /// Scans one directly bound input whose type carries a governed contract.
    ///
    /// Returns `Ok(false)` when the type has no admitted contract, which states
    /// no obligation for this validator to discharge and leaves the registry's
    /// own representation validator as its authority. Nothing is approximated
    /// in either direction, and a governed type is scanned exactly once: the
    /// registry validator reaches the same
    /// [`tiler_ir::semantic::check_bound_value`] authority, so running both
    /// would recompute one answer rather than check two things.
    pub(crate) fn bind_input(
        &mut self,
        program: &SemanticProgram,
        key: &InputKey,
        value: ValueId,
        tensor: &Tensor,
    ) -> Result<bool, EvaluationError> {
        if !ResolvedValueConformanceContract::is_governed(tensor.resolved_type()) {
            return Ok(false);
        }
        let subject = ValueConformanceSubject::new(
            ValueOrigin::direct_binding(key.clone()),
            ValueStability::ImmutableHost,
            route_dependency(program),
            SemanticLogicalView::WholeValue,
            tensor.resolved_type().clone(),
            tensor.shape().clone(),
        );
        let evidence = scan_bound_value(
            &Self::validator(),
            &subject,
            &TensorLogicalView::new(tensor),
        )
        .map_err(|rejection| EvaluationError::ValueConformance {
            key: Some(key.clone()),
            rejection: Box::new(rejection),
        })?;
        self.proofs.insert(value, evidence);
        Ok(true)
    }

    /// Establishes conformance of one produced result without reading it.
    ///
    /// Returns `Ok(false)` when the producer has no admitted composition rule
    /// or the result carries no governed contract, which leaves the registry's
    /// own value validator as the authority for that result exactly as before.
    pub(crate) fn produce_result(
        &mut self,
        program: &SemanticProgram,
        operation: OperationRef<'_>,
        result: ValueId,
        resolved_type: &ResolvedValueType,
        shape: &Shape,
    ) -> Result<bool, EvaluationError> {
        let Some(operand_roles) = carried_operand_roles(operation.key()) else {
            return Ok(false);
        };
        if !ResolvedValueConformanceContract::is_governed(resolved_type) {
            return Ok(false);
        }
        let discharged: Vec<_> = operation
            .semantic_preconditions()
            .filter(|precondition| precondition.status() == SemanticPreconditionStatus::Residual)
            .filter_map(SemanticPreconditionRef::obligation_identity)
            .collect();
        // The evaluator refuses an out-of-domain scale inside the operation
        // itself, so reaching this line means every residual this occurrence
        // declared has been enforced against the payload that discharges it.
        let preconditions = SemanticPreconditionsDischarged::for_occurrence(operation, &discharged)
            .map_err(|source| EvaluationError::ValueConformanceComposition {
                operation: operation.key().clone(),
                detail: source.to_string(),
            })?;
        let operands: Vec<ValueId> = operation.operands().collect();
        let mut composed = Vec::with_capacity(operand_roles.len());
        for (position, role) in operand_roles {
            let operand = operands.get(position).copied().ok_or_else(|| {
                EvaluationError::ValueConformanceComposition {
                    operation: operation.key().clone(),
                    detail: format!("operand {position} is absent from the occurrence"),
                }
            })?;
            let evidence = self.proofs.get(&operand).ok_or_else(|| {
                EvaluationError::ValueConformanceComposition {
                    operation: operation.key().clone(),
                    detail: format!("operand {position} carries no conformance proof"),
                }
            })?;
            composed.push(ComposedOperand { role, evidence });
        }
        let subject = ValueConformanceSubject::new(
            ValueOrigin::produced_result(operation, result).map_err(|source| {
                EvaluationError::ValueConformanceComposition {
                    operation: operation.key().clone(),
                    detail: source.to_string(),
                }
            })?,
            ValueStability::ImmutableHost,
            route_dependency(program),
            SemanticLogicalView::WholeValue,
            resolved_type.clone(),
            shape.clone(),
        );
        let evidence =
            compose_produced_conformance(&Self::validator(), &subject, &preconditions, &composed)
                .map_err(|source| EvaluationError::ValueConformanceComposition {
                operation: operation.key().clone(),
                detail: source.to_string(),
            })?;
        self.proofs.insert(result, evidence);
        Ok(true)
    }
}

/// Returns which operand positions a producer carries into which result roles.
///
/// `Assemble` associates existing components, so both its codes and its zero
/// point are carried through. `Quantize` computes its codes under the declared
/// clamp and nearest-even rounding, so only its zero point is. Any other
/// operation has no admitted rule and composes nothing.
fn carried_operand_roles(operation: &OpKey) -> Option<Vec<(usize, EncodedComponentRole)>> {
    if operation == &assemble_strict_affine_op() {
        Some(vec![
            (0, STRICT_AFFINE_CODES_ROLE),
            (2, STRICT_AFFINE_ZERO_POINT_ROLE),
        ])
    } else if operation == &quantize_strict_affine_op() {
        Some(vec![(2, STRICT_AFFINE_ZERO_POINT_ROLE)])
    } else {
        None
    }
}

/// The route one host evaluation's proofs depend on.
///
/// A host evaluation's route is its semantic graph: two programs are two
/// routes, and a proof taken against one does not authorize a subject in the
/// other even when the value looks identical.
fn route_dependency(program: &SemanticProgram) -> RouteDependency {
    RouteDependency::new(program.semantic_identity().graph().as_bytes())
}

#[cfg(test)]
mod tests;

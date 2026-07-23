//! The deterministic `NormalizeSemantics` stage.
//!
//! Normalization runs after request verification and before region formation.
//! It produces one canonical semantic graph for a class of programs that differ
//! only in redundant spelling, and it never produces alternatives: an
//! alternative-producing rewrite engine is a separate later authority.
//!
//! The first profile proves exactly one rule, common-subexpression elimination
//! over referentially transparent operations. Breadth is deliberately not the
//! goal; the machinery and its guarantees are:
//!
//! - **Termination.** Detection is a single forward pass over a finite verified
//!   operation list. There is no fixpoint loop, so termination does not depend
//!   on a measure decreasing.
//! - **Traversal order.** Operations are visited in verified topological order
//!   by ascending graph-local ordinal, and results within an operation by
//!   ascending result position. The earliest occurrence of a congruence class
//!   always becomes its canonical representative.
//! - **Budgets.** [`DeterministicBudgets::normalization_rewrites`] bounds the
//!   rewrites one compilation may commit. Exhaustion abandons the whole rewrite
//!   and keeps the verified input program, so a budget never yields a partially
//!   canonicalized graph.
//! - **Transactional failure.** The input [`SemanticProgram`] is immutable and
//!   is never mutated. A rewrite is built as a separate candidate program and is
//!   adopted only after it passes every postcondition.
//! - **Semantic revalidation.** The candidate is rebuilt through the ordinary
//!   checked [`SemanticProgramBuilder`], so the frozen semantic authority
//!   re-infers and re-validates every operation. The stage never copies verified
//!   structure forward and never trusts its own output structurally.
//! - **Canonical identity.** The adopted program's `SemanticIdentity` is the
//!   canonical identity of the normalized result, and re-running detection on it
//!   is required to find no further rewrite.
//!
//! Reference revalidation is deliberately *not* performed inside this stage.
//! `tiler-reference` is an executable oracle whose cost is proportional to the
//! materialized element count, and the compiler admits programs whose element
//! counts reach billions, so evaluating every rewrite at compile time is not a
//! viable contract. Differential reference equivalence between the input and the
//! normalized program is instead proven by the checked conformance tests in this
//! module, which is where this crate's reference dependency lives.

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use tiler_ir::semantic::{
    OpKey, OperationAttributes, OperationEffect, ResolvedValueType, SemanticProgram,
    SemanticProgramBuilder, ValueId,
};
use tiler_ir::shape::Shape;

use crate::explain::{
    EvidenceBasis, ExplainError, ExplainEvent, ExplainFact, ExplainRecordId, ExplainStage,
    ExplainWriter, FactValue, PredicateAssessment, RejectionClass, ResourceKey, RuleRef,
    SubjectKey, SubjectKind,
};
use crate::request::{DeterministicBudgets, StrictF32NumericalContract};

/// Stable identity of the normalization stage rule.
pub(crate) const NORMALIZE_STAGE_RULE: &str = "normalize.semantics.v1";
/// Stable identity of the one proved rewrite in the first profile.
pub(crate) const NORMALIZE_SHARED_VALUE_RULE: &str = "normalize.common-subexpression.v1";
/// Stable subject key for whole-program normalization records.
pub(crate) const NORMALIZATION_SUBJECT: &str = "normalization:program";
const REWRITE_BUDGET_RESOURCE: &str = "normalization-rewrites";

/// Typed failure of the deterministic normalization stage.
///
/// Every variant is invalid compiler output rather than a rejected user program:
/// the stage only ever observes an already verified [`SemanticProgram`], and a
/// rewrite it produced failing revalidation is a compiler defect.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NormalizeError {
    /// The verified input program violated a stage precondition.
    Structure { rule: &'static str },
    /// The checked semantic builder rejected the candidate rewrite.
    Rebuild { rule: &'static str },
    /// The rebuilt program violated a normalization postcondition.
    InvalidRewrite { rule: &'static str },
}

impl NormalizeError {
    pub(crate) const fn reason(self) -> &'static str {
        match self {
            Self::Structure { rule } | Self::Rebuild { rule } | Self::InvalidRewrite { rule } => {
                rule
            }
        }
    }

    const fn class(self) -> &'static str {
        match self {
            Self::Structure { .. } => "structure",
            Self::Rebuild { .. } => "rebuild",
            Self::InvalidRewrite { .. } => "invalid-rewrite",
        }
    }
}

impl fmt::Display for NormalizeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "compile.normalize.{}.{}: deterministic normalization produced invalid compiler output",
            self.class(),
            self.reason()
        )
    }
}

impl Error for NormalizeError {}

/// One committed merge of a redundant operation into its canonical occurrence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SharedValueMerge {
    /// Graph-local ordinal of the retained canonical operation.
    canonical: usize,
    /// Graph-local ordinal of the redundant operation it replaced.
    merged: usize,
}

/// The deterministic result of running `NormalizeSemantics` once.
#[derive(Clone, Debug)]
pub(crate) struct NormalizationOutcome {
    normalized: Option<SemanticProgram>,
    merges: Vec<SharedValueMerge>,
    operations_before: usize,
    operations_after: usize,
    budget_stop: Option<(u64, u64)>,
    numerical_contract_key: &'static str,
    canonical_graph_digest: u64,
}

impl NormalizationOutcome {
    /// Returns the adopted canonical program, or `None` when the verified input
    /// was already canonical for this profile.
    pub(crate) const fn normalized_program(&self) -> Option<&SemanticProgram> {
        self.normalized.as_ref()
    }

    /// Returns the committed merges in deterministic traversal order.
    #[cfg(test)]
    pub(crate) fn merges(&self) -> &[SharedValueMerge] {
        &self.merges
    }

    /// Returns the declared budget and the demand that stopped the rewrite.
    #[cfg(test)]
    pub(crate) const fn budget_stop(&self) -> Option<(u64, u64)> {
        self.budget_stop
    }

    /// Emits this outcome through the typed explain authority.
    ///
    /// Records form one linear causal chain rooted at `cause` so a record never
    /// accumulates an unbounded cause set, and the returned identifier is the
    /// stage receipt later stages depend on.
    pub(crate) fn record(
        &self,
        explain: &mut ExplainWriter,
        cause: Option<ExplainRecordId>,
    ) -> Result<Option<ExplainRecordId>, ExplainError> {
        let mut cause = cause;
        if let Some((limit, actual)) = self.budget_stop {
            let subject = explain.subject(SubjectKind::Normalization, NORMALIZATION_SUBJECT)?;
            cause = explain
                .push_detail(
                    RuleRef::builtin(NORMALIZE_STAGE_RULE)?,
                    vec![subject],
                    ExplainEvent::BudgetStop {
                        stage: ExplainStage::Normalization,
                        resource: ResourceKey::new(REWRITE_BUDGET_RESOURCE)?,
                        limit,
                        actual,
                    },
                    cause.into_iter().collect(),
                )?
                .or(cause);
        }
        for merge in &self.merges {
            let key = format!("normalization:shared-value/operation:{}", merge.merged);
            let subject = explain.subject(SubjectKind::Normalization, &key)?;
            let assessment = PredicateAssessment::proven(
                "normalize.shared-value-identity",
                EvidenceBasis::CheckedInvariant,
            )?
            .with_fact(ExplainFact::new(
                "canonical-operation",
                FactValue::Count(count(merge.canonical)),
            )?)?
            .with_fact(ExplainFact::new(
                "merged-operation",
                FactValue::Count(count(merge.merged)),
            )?)?;
            cause = explain
                .push_detail(
                    RuleRef::builtin(NORMALIZE_SHARED_VALUE_RULE)?,
                    vec![subject],
                    ExplainEvent::Check {
                        stage: ExplainStage::Normalization,
                        assessment,
                        rejection: RejectionClass::IntrinsicInvalid,
                    },
                    cause.into_iter().collect(),
                )?
                .or(cause);
        }
        let assessment = PredicateAssessment::proven(
            "normalize.canonical-fixpoint",
            EvidenceBasis::CheckedInvariant,
        )?
        .with_fact(ExplainFact::new(
            "rewrite-count",
            FactValue::Count(count(self.merges.len())),
        )?)?
        .with_fact(ExplainFact::new(
            "operations-before",
            FactValue::Count(count(self.operations_before)),
        )?)?
        .with_fact(ExplainFact::new(
            "operations-after",
            FactValue::Count(count(self.operations_after)),
        )?)?
        .with_fact(ExplainFact::new(
            "numerical-contract",
            FactValue::Identity(SubjectKey::new(self.numerical_contract_key)?),
        )?)?
        .with_fact(ExplainFact::new(
            "canonical-graph-digest",
            FactValue::Identity(SubjectKey::new(format!(
                "{:016x}",
                self.canonical_graph_digest
            ))?),
        )?)?;
        let subject = explain.subject(SubjectKind::Normalization, NORMALIZATION_SUBJECT)?;
        explain.push_detail(
            RuleRef::builtin(NORMALIZE_STAGE_RULE)?,
            vec![subject],
            ExplainEvent::Check {
                stage: ExplainStage::Normalization,
                assessment,
                rejection: RejectionClass::IntrinsicInvalid,
            },
            cause.into_iter().collect(),
        )
    }
}

fn count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

/// Runs the deterministic normalization stage over one verified program.
///
/// The input is never mutated. When a rewrite is committed the returned outcome
/// carries a separately built and fully revalidated program whose semantic
/// identity is the canonical identity of the normalized result.
pub(crate) fn normalize_semantics(
    program: &SemanticProgram,
    budgets: DeterministicBudgets,
    numerical_contract: StrictF32NumericalContract,
) -> Result<NormalizationOutcome, NormalizeError> {
    let congruence = detect_shared_values(program)?;
    let operations_before = program.operation_count();
    let demand = count(congruence.merges.len());
    let limit = u64::from(budgets.normalization_rewrites);
    if demand > limit {
        // A partially applied canonicalization would make the result depend on
        // the budget rather than on the program, so the whole rewrite is
        // abandoned and the verified input stays authoritative.
        return Ok(NormalizationOutcome {
            normalized: None,
            merges: Vec::new(),
            operations_before,
            operations_after: operations_before,
            budget_stop: Some((limit, demand)),
            numerical_contract_key: numerical_contract.key,
            canonical_graph_digest: digest(program),
        });
    }
    if congruence.merges.is_empty() {
        return Ok(NormalizationOutcome {
            normalized: None,
            merges: Vec::new(),
            operations_before,
            operations_after: operations_before,
            budget_stop: None,
            numerical_contract_key: numerical_contract.key,
            canonical_graph_digest: digest(program),
        });
    }
    let normalized = rebuild(program, &congruence)?;
    verify_normalized(program, &normalized, &congruence)?;
    Ok(NormalizationOutcome {
        operations_before,
        operations_after: normalized.operation_count(),
        merges: congruence.merges,
        budget_stop: None,
        numerical_contract_key: numerical_contract.key,
        canonical_graph_digest: digest(&normalized),
        normalized: Some(normalized),
    })
}

/// Canonical equality of one operation occurrence.
///
/// This is exactly the accepted semantic-value equality: operation key, ordered
/// operand identities taken after congruence, canonical attributes, and ordered
/// inferred result types. Source origin — declaration position, handles, and
/// graph ownership — is deliberately absent so it can be preserved for
/// explanation without participating in equality. The numerical contract is a
/// whole-request property in this IR and is checked once by the caller rather
/// than repeated per occurrence.
#[derive(Debug, Eq, Hash, PartialEq)]
struct OperationSignature {
    key: OpKey,
    attributes: OperationAttributes,
    operands: Vec<usize>,
    results: Vec<(ResolvedValueType, Shape)>,
}

/// Congruence classes discovered by one deterministic detection pass.
struct Congruence {
    /// Canonical value ordinal for every graph-local value ordinal.
    representative: Vec<usize>,
    /// Whether each graph-local operation survives normalization.
    retained: Vec<bool>,
    /// Ordered result value ordinals of each graph-local operation.
    operation_results: Vec<Vec<usize>>,
    /// Committed merges in traversal order.
    merges: Vec<SharedValueMerge>,
}

fn detect_shared_values(program: &SemanticProgram) -> Result<Congruence, NormalizeError> {
    let ordinals: HashMap<ValueId, usize> = program
        .values()
        .enumerate()
        .map(|(ordinal, value)| (value.id(), ordinal))
        .collect();
    let mut congruence = Congruence {
        representative: (0..program.value_count()).collect(),
        retained: vec![true; program.operation_count()],
        operation_results: Vec::with_capacity(program.operation_count()),
        merges: Vec::new(),
    };
    let mut canonical: HashMap<OperationSignature, usize> = HashMap::new();
    let facts: Vec<(ResolvedValueType, Shape)> = program
        .values()
        .map(|value| (value.resolved_type().clone(), value.shape().clone()))
        .collect();
    for (index, operation) in program.operations().enumerate() {
        let results = operation
            .results()
            .map(|result| ordinal(&ordinals, result))
            .collect::<Result<Vec<_>, _>>()?;
        congruence.operation_results.push(results.clone());
        let definition = program
            .semantic_registry()
            .operation_definition(operation.key())
            .ok_or(NormalizeError::Structure {
                rule: "operation-definition",
            })?;
        // Only a referentially transparent occurrence may be replaced by an
        // earlier one. An effect class this profile cannot prove transparent is
        // left untouched rather than approximated, which is the fail-closed
        // direction for an optimization.
        if !matches!(definition.effect(), OperationEffect::Pure) {
            continue;
        }
        let mut operands = Vec::with_capacity(operation.operands().len());
        for operand in operation.operands() {
            operands.push(congruence.representative[ordinal(&ordinals, operand)?]);
        }
        let mut result_facts = Vec::with_capacity(results.len());
        for result in &results {
            result_facts.push(
                facts
                    .get(*result)
                    .ok_or(NormalizeError::Structure {
                        rule: "result-value",
                    })?
                    .clone(),
            );
        }
        let signature = OperationSignature {
            key: operation.key().clone(),
            attributes: operation.attributes().clone(),
            operands,
            results: result_facts,
        };
        if let Some(existing) = canonical.get(&signature).copied() {
            congruence.retained[index] = false;
            congruence.merges.push(SharedValueMerge {
                canonical: existing,
                merged: index,
            });
            let canonical_results = &congruence.operation_results[existing];
            if canonical_results.len() != results.len() {
                return Err(NormalizeError::Structure {
                    rule: "congruent-result-arity",
                });
            }
            for (position, result) in results.iter().enumerate() {
                congruence.representative[*result] = canonical_results[position];
            }
        } else {
            canonical.insert(signature, index);
        }
    }
    Ok(congruence)
}

fn ordinal(ordinals: &HashMap<ValueId, usize>, value: ValueId) -> Result<usize, NormalizeError> {
    ordinals
        .get(&value)
        .copied()
        .ok_or(NormalizeError::Structure {
            rule: "value-ordinal",
        })
}

/// Builds the candidate normalized program through the checked semantic builder.
///
/// Every operation is re-applied through the frozen authority, so result types
/// and shapes are re-inferred rather than copied from the input program.
fn rebuild(
    program: &SemanticProgram,
    congruence: &Congruence,
) -> Result<SemanticProgram, NormalizeError> {
    let ordinals: HashMap<ValueId, usize> = program
        .values()
        .enumerate()
        .map(|(ordinal, value)| (value.id(), ordinal))
        .collect();
    let mut builder = SemanticProgramBuilder::try_new(program.semantic_registry().clone())
        .map_err(|_| NormalizeError::Rebuild {
            rule: "builder-create",
        })?;
    let mut mapped: Vec<Option<ValueId>> = vec![None; program.value_count()];
    for input in program.inputs() {
        let position = ordinal(&ordinals, input.value())?;
        let value = program
            .value(input.value())
            .map_err(|_| NormalizeError::Structure {
                rule: "input-value",
            })?;
        let rebuilt = builder
            .input_resolved(
                input.key().clone(),
                value.shape().clone(),
                value.resolved_type().clone(),
            )
            .map_err(|_| NormalizeError::Rebuild { rule: "input" })?;
        mapped[position] = Some(rebuilt);
    }
    for (index, operation) in program.operations().enumerate() {
        if !congruence.retained[index] {
            continue;
        }
        let mut operands = Vec::with_capacity(operation.operands().len());
        for operand in operation.operands() {
            operands.push(resolve(congruence, &mapped, ordinal(&ordinals, operand)?)?);
        }
        let results = builder
            .apply(
                operation.key().clone(),
                operation.attributes().clone(),
                &operands,
            )
            .map_err(|_| NormalizeError::Rebuild { rule: "operation" })?;
        let expected = &congruence.operation_results[index];
        if results.len() != expected.len() {
            return Err(NormalizeError::InvalidRewrite {
                rule: "result-arity",
            });
        }
        for (position, result) in results.into_iter().enumerate() {
            mapped[expected[position]] = Some(result);
        }
    }
    for output in program.outputs() {
        let value = resolve(congruence, &mapped, ordinal(&ordinals, output.value())?)?;
        builder
            .output_resolved(output.key().clone(), value)
            .map_err(|_| NormalizeError::Rebuild { rule: "output" })?;
    }
    builder.build().map_err(|_| NormalizeError::Rebuild {
        rule: "semantic-verification",
    })
}

fn resolve(
    congruence: &Congruence,
    mapped: &[Option<ValueId>],
    position: usize,
) -> Result<ValueId, NormalizeError> {
    let canonical = *congruence
        .representative
        .get(position)
        .ok_or(NormalizeError::Structure {
            rule: "representative",
        })?;
    mapped
        .get(canonical)
        .copied()
        .flatten()
        .ok_or(NormalizeError::InvalidRewrite {
            rule: "unmapped-value",
        })
}

/// Checks every postcondition before the candidate program may be adopted.
fn verify_normalized(
    original: &SemanticProgram,
    normalized: &SemanticProgram,
    congruence: &Congruence,
) -> Result<(), NormalizeError> {
    let expected_operations = original
        .operation_count()
        .checked_sub(congruence.merges.len())
        .ok_or(NormalizeError::InvalidRewrite {
            rule: "operation-count",
        })?;
    if normalized.operation_count() != expected_operations
        || normalized.input_count() != original.input_count()
        || normalized.output_count() != original.output_count()
    {
        return Err(NormalizeError::InvalidRewrite {
            rule: "operation-count",
        });
    }
    for (before, after) in original.inputs().zip(normalized.inputs()) {
        let before_value = original
            .value(before.value())
            .map_err(|_| NormalizeError::InvalidRewrite { rule: "interface" })?;
        let after_value = normalized
            .value(after.value())
            .map_err(|_| NormalizeError::InvalidRewrite { rule: "interface" })?;
        if before.key() != after.key()
            || before_value.shape() != after_value.shape()
            || before_value.resolved_type() != after_value.resolved_type()
        {
            return Err(NormalizeError::InvalidRewrite {
                rule: "input-interface",
            });
        }
    }
    for (before, after) in original.outputs().zip(normalized.outputs()) {
        let before_value = original
            .value(before.value())
            .map_err(|_| NormalizeError::InvalidRewrite { rule: "interface" })?;
        let after_value = normalized
            .value(after.value())
            .map_err(|_| NormalizeError::InvalidRewrite { rule: "interface" })?;
        if before.key() != after.key()
            || before_value.shape() != after_value.shape()
            || before_value.resolved_type() != after_value.resolved_type()
        {
            return Err(NormalizeError::InvalidRewrite {
                rule: "output-interface",
            });
        }
    }
    let before = original.semantic_identity();
    let after = normalized.semantic_identity();
    // Removing a redundant occurrence changes graph meaning but never the set of
    // reached semantic definitions, the providers that admitted them, or the
    // registry snapshot that validated them.
    if before.reached_definitions() != after.reached_definitions()
        || before.admission_provenance() != after.admission_provenance()
        || before.registry_snapshot() != after.registry_snapshot()
    {
        return Err(NormalizeError::InvalidRewrite {
            rule: "semantic-authority",
        });
    }
    if before.graph() == after.graph() {
        return Err(NormalizeError::InvalidRewrite {
            rule: "graph-identity",
        });
    }
    if !detect_shared_values(normalized)?.merges.is_empty() {
        return Err(NormalizeError::InvalidRewrite { rule: "fixpoint" });
    }
    Ok(())
}

fn digest(program: &SemanticProgram) -> u64 {
    program
        .semantic_identity()
        .graph()
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::explain::ExplainLimits;
    use crate::request::{CompilationRequest, verify_request};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder,
        StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};
    use tiler_reference::{
        FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor,
        TensorPayloadView,
    };

    /// Builds the governed serial-sum program.
    ///
    /// `share_constants` selects whether the scale and bias constants are one
    /// authored value or two identical redundant occurrences.
    fn program(scale: f32, bias: f32, share_constants: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale_value = F32Constant::apply(&mut builder, scale.to_bits()).unwrap();
        let bias_value = if share_constants {
            scale_value
        } else {
            F32Constant::apply(&mut builder, bias.to_bits()).unwrap()
        };
        let product = F32Multiply::apply(&mut builder, input, scale_value).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias_value).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    fn normalize(program: &SemanticProgram) -> NormalizationOutcome {
        normalize_semantics(
            program,
            DeterministicBudgets::governed(),
            StrictF32NumericalContract::governed(),
        )
        .unwrap()
    }

    fn evaluate(program: &SemanticProgram, values: &[f32]) -> Vec<u32> {
        let key = InputKey::new("input").unwrap();
        let tensor = Tensor::dense(
            F32::resolved_type(),
            Shape::from_dims([2, 3]),
            values
                .iter()
                .map(|value| {
                    ReferenceElement::from_float_bits(
                        value.to_bits().to_be_bytes(),
                        FloatBitOrder::MostSignificantByteFirst,
                    )
                    .unwrap()
                })
                .collect(),
        )
        .unwrap();
        let outputs = ReferenceEvaluator::standard()
            .unwrap()
            .evaluate(program, &[InputBinding::new(&key, &tensor)])
            .unwrap();
        match outputs[0].payload() {
            TensorPayloadView::Dense(elements) => elements
                .iter()
                .map(|element| u32::from_be_bytes(<[u8; 4]>::try_from(element.as_bytes()).unwrap()))
                .collect(),
            _ => panic!("expected a dense f32 reference output"),
        }
    }

    #[test]
    fn already_canonical_programs_are_left_untouched() {
        let distinct = program(2.0, 1.0, false);
        let outcome = normalize(&distinct);

        assert!(outcome.normalized_program().is_none());
        assert!(outcome.merges().is_empty());
        assert_eq!(outcome.budget_stop(), None);
        assert_eq!(outcome.operations_before, outcome.operations_after);
    }

    #[test]
    fn identical_pure_invocations_normalize_to_one_semantic_value() {
        let duplicated = program(2.0, 2.0, false);
        assert_eq!(duplicated.operation_count(), 5);
        let outcome = normalize(&duplicated);
        let normalized = outcome
            .normalized_program()
            .expect("a redundant constant is rewritten");

        assert_eq!(
            outcome.merges(),
            [SharedValueMerge {
                canonical: 0,
                merged: 1,
            }]
        );
        assert_eq!(normalized.operation_count(), 4);
        assert_eq!(normalized.value_count(), duplicated.value_count() - 1);
        // The surviving constant feeds both pointwise operations.
        let constant = normalized
            .operations()
            .next()
            .expect("the canonical constant is first in topological order");
        let constant_result = constant.results().next().unwrap();
        assert_eq!(
            normalized
                .operations()
                .filter(|operation| operation
                    .operands()
                    .any(|operand| operand == constant_result))
                .count(),
            2
        );
    }

    #[test]
    fn normalization_converges_on_one_canonical_graph_identity() {
        let duplicated = program(2.0, 2.0, false);
        let shared = program(2.0, 2.0, true);
        assert_ne!(
            duplicated.semantic_identity().graph(),
            shared.semantic_identity().graph(),
            "the fixture must start from genuinely different graphs"
        );

        let normalized = normalize(&duplicated);
        let normalized = normalized
            .normalized_program()
            .expect("the duplicated program is rewritten");
        let already_canonical = normalize(&shared);

        assert!(already_canonical.normalized_program().is_none());
        assert_eq!(
            normalized.semantic_identity().graph(),
            shared.semantic_identity().graph()
        );
        assert_eq!(
            normalized.semantic_identity().reached_definitions(),
            shared.semantic_identity().reached_definitions()
        );
    }

    #[test]
    fn normalization_is_idempotent_and_deterministic() {
        let outcome = normalize(&program(2.0, 2.0, false));
        let normalized = outcome.normalized_program().unwrap().clone();

        // Renormalizing the result reaches the declared fixpoint.
        let again = normalize(&normalized);
        assert!(again.normalized_program().is_none());
        assert_eq!(again.canonical_graph_digest, digest(&normalized));

        // Two independent runs over equal inputs agree on identity and merges.
        let repeated = normalize(&program(2.0, 2.0, false));
        assert_eq!(
            repeated.normalized_program().unwrap().semantic_identity(),
            normalized.semantic_identity()
        );
        assert_eq!(repeated.merges(), outcome.merges());
        assert_eq!(
            repeated.canonical_graph_digest,
            outcome.canonical_graph_digest
        );
    }

    #[test]
    fn normalized_program_matches_the_reference_evaluator_bitwise() {
        let values = [1.0, -2.0, 3.5, f32::MIN_POSITIVE, -0.0, f32::NAN];
        for (scale, bias) in [
            (2.0_f32, 2.0_f32),
            (0.0, 0.0),
            (f32::NAN, f32::NAN),
            (f32::INFINITY, f32::INFINITY),
            (-0.0, -0.0),
        ] {
            let original = program(scale, bias, false);
            let outcome = normalize(&original);
            let normalized = outcome
                .normalized_program()
                .expect("identical constants are always merged");

            assert_eq!(
                evaluate(&original, &values),
                evaluate(normalized, &values),
                "normalization must preserve exact reference semantics"
            );
        }
    }

    #[test]
    fn negative_zero_and_nan_constants_are_distinguished_by_canonical_bits() {
        // `-0.0 == 0.0` and `NaN != NaN` under float comparison, so a rule that
        // compared decoded floats instead of canonical attribute bytes would
        // merge or split these incorrectly.
        let signed_zero = program(0.0, -0.0, false);
        assert!(normalize(&signed_zero).normalized_program().is_none());

        let nan = program(f32::NAN, f32::NAN, false);
        let outcome = normalize(&nan);
        assert_eq!(outcome.merges().len(), 1);
        assert_eq!(
            evaluate(&nan, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]),
            evaluate(
                outcome.normalized_program().unwrap(),
                &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]
            )
        );
    }

    #[test]
    fn an_exhausted_rewrite_budget_abandons_the_whole_rewrite() {
        let duplicated = program(2.0, 2.0, false);
        let mut budgets = DeterministicBudgets::governed();
        budgets.normalization_rewrites = 0;
        let outcome =
            normalize_semantics(&duplicated, budgets, StrictF32NumericalContract::governed())
                .unwrap();

        assert!(outcome.normalized_program().is_none());
        assert!(outcome.merges().is_empty());
        assert_eq!(outcome.budget_stop(), Some((0, 1)));
        assert_eq!(outcome.operations_after, duplicated.operation_count());
    }

    #[test]
    fn stage_records_are_typed_bounded_and_causally_chained() {
        let duplicated = program(2.0, 2.0, false);
        let outcome = normalize(&duplicated);
        let normalized = outcome.normalized_program().unwrap();
        let verified = verify_request(CompilationRequest::governed(normalized)).unwrap();
        let target = verified.for_target(verified.target_profiles()[0]).unwrap();
        let mut explain = ExplainWriter::new(&target, ExplainLimits::default()).unwrap();

        let receipt = outcome.record(&mut explain, None).unwrap().unwrap();
        let alternative = explain
            .subject(SubjectKind::Alternative, "alternative:test")
            .unwrap();
        explain
            .note_selection(
                alternative,
                crate::explain::SelectionOutcome::Selected,
                None,
            )
            .unwrap();
        let trace = explain
            .finish_success(&["alternative:test"], "alternative:test")
            .unwrap();

        let merge = trace
            .records()
            .iter()
            .find(|record| record.rule().key().as_str() == NORMALIZE_SHARED_VALUE_RULE)
            .expect("the committed merge is explained");
        let summary = trace
            .records()
            .iter()
            .find(|record| record.id() == receipt)
            .expect("the stage receipt is retained");
        assert_eq!(summary.causes(), [merge.id()]);
        assert_eq!(summary.subjects()[0].key().as_str(), NORMALIZATION_SUBJECT);
        let ExplainEvent::Check { assessment, .. } = summary.event() else {
            panic!("the stage receipt is a checked assertion");
        };
        assert!(assessment.facts().iter().any(|fact| {
            fact.key().as_str() == "rewrite-count" && matches!(fact.value(), FactValue::Count(1))
        }));
        assert!(assessment.facts().iter().any(|fact| {
            fact.key().as_str() == "numerical-contract"
                && matches!(fact.value(), FactValue::Identity(key)
                    if key.as_str() == "tiler.strict-f32.v1")
        }));
        assert!(trace.render().contains("normalization admitted"));
    }

    #[test]
    fn budget_stop_is_rendered_as_a_typed_normalization_event() {
        let duplicated = program(2.0, 2.0, false);
        let mut budgets = DeterministicBudgets::governed();
        budgets.normalization_rewrites = 0;
        let outcome =
            normalize_semantics(&duplicated, budgets, StrictF32NumericalContract::governed())
                .unwrap();
        let verified = verify_request(CompilationRequest::governed(&duplicated)).unwrap();
        let target = verified.for_target(verified.target_profiles()[0]).unwrap();
        let mut explain = ExplainWriter::new(&target, ExplainLimits::default()).unwrap();

        outcome.record(&mut explain, None).unwrap().unwrap();
        let alternative = explain
            .subject(SubjectKind::Alternative, "alternative:test")
            .unwrap();
        explain
            .note_selection(
                alternative,
                crate::explain::SelectionOutcome::Selected,
                None,
            )
            .unwrap();
        let trace = explain
            .finish_success(&["alternative:test"], "alternative:test")
            .unwrap();

        assert!(trace.records().iter().any(|record| {
            matches!(
                record.event(),
                ExplainEvent::BudgetStop {
                    stage: ExplainStage::Normalization,
                    limit: 0,
                    actual: 1,
                    ..
                }
            )
        }));
        assert!(
            trace
                .render()
                .contains("budget-stop:normalization-rewrites:0:1")
        );
    }

    #[test]
    fn errors_report_their_exact_class_and_rule() {
        let error = NormalizeError::InvalidRewrite { rule: "fixpoint" };
        assert_eq!(error.reason(), "fixpoint");
        assert_eq!(
            error.to_string(),
            "compile.normalize.invalid-rewrite.fixpoint: deterministic normalization produced invalid compiler output"
        );
        assert_eq!(
            NormalizeError::Structure {
                rule: "value-ordinal"
            }
            .to_string(),
            "compile.normalize.structure.value-ordinal: deterministic normalization produced invalid compiler output"
        );
        assert_eq!(
            NormalizeError::Rebuild { rule: "input" }.to_string(),
            "compile.normalize.rebuild.input: deterministic normalization produced invalid compiler output"
        );
    }
}

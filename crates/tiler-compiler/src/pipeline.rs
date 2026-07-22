use std::error::Error;
use std::fmt;

use crate::fusion::{
    CandidateError, CandidateKind, FusionNumericalProof, enumerate_candidates, prove_fused_numerics,
};
use crate::physical::{
    PhysicalError, VerifiedScheduledRegion, VerifiedStructuredKernel, build_fused_scheduled_region,
    build_scheduled_regions, lower_structured_kernel,
};
use crate::program::{
    ArtifactConstructionPlan, KernelProgram, ProgramError, assert_kernels_match_program,
    build_artifact_plan, build_fused_kernel_program, build_kernel_program,
    verify_semantic_output_type,
};
use crate::request::{CompilationRequest, RequestError, verify_request};

const SELECTION_POLICY_KEY: &str = "tiler.selection.structural-pareto.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExplainPhase {
    RequestVerification,
    RegionFormation,
    IntrinsicSchedule,
    TargetFeasibility,
    KernelRefinement,
    ProgramVerification,
    ArtifactPlanning,
    CandidateEnumeration,
    NumericalLegality,
    PortfolioSelection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExplainOutcome {
    Accepted,
    Rejected,
    Selected,
    NotSelected,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExplainRecord {
    pub(crate) phase: ExplainPhase,
    pub(crate) rule: &'static str,
    pub(crate) subject: String,
    pub(crate) outcome: ExplainOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompilationProduct {
    pub(crate) targets: Vec<TargetCompilationProduct>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TargetCompilationProduct {
    pub(crate) target_profile_key: &'static str,
    pub(crate) portfolio: ProgramPortfolio,
    pub(crate) explain: Vec<ExplainRecord>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProgramAlternativeKind {
    Materialized,
    Fused,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StructuralCost {
    pub(crate) dispatch_count: u32,
    pub(crate) temporary_allocation_count: u32,
    pub(crate) materialized_bytes: u64,
    pub(crate) intermediate_global_reads: u64,
    pub(crate) intermediate_global_writes: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EquivalenceEvidence {
    MaterializedReference,
    Fused(FusionNumericalProof),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProgramAlternative {
    pub(crate) stable_id: &'static str,
    pub(crate) kind: ProgramAlternativeKind,
    pub(crate) scheduled_regions: Vec<VerifiedScheduledRegion>,
    pub(crate) kernels: Vec<VerifiedStructuredKernel>,
    pub(crate) program: KernelProgram,
    pub(crate) artifact_plan: ArtifactConstructionPlan,
    pub(crate) structural_cost: StructuralCost,
    pub(crate) equivalence: EquivalenceEvidence,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PortfolioSelection {
    pub(crate) policy_key: &'static str,
    pub(crate) selected_alternative: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProgramPortfolio {
    pub(crate) alternatives: Vec<ProgramAlternative>,
    pub(crate) selection: PortfolioSelection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CompileError {
    InvalidRequest(RequestError),
    UnsupportedCapability(RequestError),
    BudgetExhausted(RequestError),
    NoFeasiblePlan(NoFeasiblePlanError),
    InvalidCompilerOutput(CompilerOutputError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NoFeasiblePlanError {
    Request(RequestError),
    Physical(PhysicalError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CompilerOutputError {
    Physical(PhysicalError),
    Program(ProgramError),
    Candidate(CandidateError),
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(error)
            | Self::UnsupportedCapability(error)
            | Self::BudgetExhausted(error)
            | Self::NoFeasiblePlan(NoFeasiblePlanError::Request(error)) => error.fmt(formatter),
            Self::NoFeasiblePlan(NoFeasiblePlanError::Physical(error)) => error.fmt(formatter),
            Self::InvalidCompilerOutput(CompilerOutputError::Physical(error)) => {
                error.fmt(formatter)
            }
            Self::InvalidCompilerOutput(CompilerOutputError::Program(error)) => {
                error.fmt(formatter)
            }
            Self::InvalidCompilerOutput(CompilerOutputError::Candidate(error)) => {
                error.fmt(formatter)
            }
        }
    }
}

impl Error for CompileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidRequest(error)
            | Self::UnsupportedCapability(error)
            | Self::BudgetExhausted(error)
            | Self::NoFeasiblePlan(NoFeasiblePlanError::Request(error)) => Some(error),
            Self::NoFeasiblePlan(NoFeasiblePlanError::Physical(error))
            | Self::InvalidCompilerOutput(CompilerOutputError::Physical(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Program(error)) => Some(error),
            Self::InvalidCompilerOutput(CompilerOutputError::Candidate(error)) => Some(error),
        }
    }
}

impl From<RequestError> for CompileError {
    fn from(value: RequestError) -> Self {
        match value {
            RequestError::UnsupportedCapability { .. } => Self::UnsupportedCapability(value),
            RequestError::ShapeProductOverflow { .. } => {
                Self::NoFeasiblePlan(NoFeasiblePlanError::Request(value))
            }
            RequestError::BudgetExceeded { .. } => Self::BudgetExhausted(value),
            RequestError::UnsupportedRequestVersion
            | RequestError::EmptyTargetSet
            | RequestError::DuplicateTargetProfile => Self::InvalidRequest(value),
        }
    }
}

impl From<PhysicalError> for CompileError {
    fn from(value: PhysicalError) -> Self {
        match value {
            PhysicalError::Refinement { .. } => {
                Self::InvalidCompilerOutput(CompilerOutputError::Physical(value))
            }
            PhysicalError::Intrinsic { .. }
            | PhysicalError::Target { .. }
            | PhysicalError::ShapeProductOverflow { .. } => {
                Self::NoFeasiblePlan(NoFeasiblePlanError::Physical(value))
            }
        }
    }
}

impl From<ProgramError> for CompileError {
    fn from(value: ProgramError) -> Self {
        Self::InvalidCompilerOutput(CompilerOutputError::Program(value))
    }
}

pub(crate) fn compile(request: CompilationRequest<'_>) -> Result<CompilationProduct, CompileError> {
    let semantic = request.program;
    let verified = verify_request(request)?;
    verify_semantic_output_type(semantic)?;
    let targets = verified
        .target_profiles
        .iter()
        .copied()
        .map(|target| compile_target(semantic, &verified.for_target(target)))
        .collect::<Result<_, _>>()?;
    Ok(CompilationProduct { targets })
}

fn compile_target(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
) -> Result<TargetCompilationProduct, CompileError> {
    let baseline = build_baseline_alternative(semantic, verified)?;
    let mut explain = baseline_explain();
    let mut alternatives = vec![baseline];
    consider_fused_alternative(semantic, verified, &mut alternatives, &mut explain)?;
    let selected_alternative = select_structural_pareto(&alternatives);
    record_selection(&alternatives, selected_alternative, &mut explain);
    Ok(TargetCompilationProduct {
        target_profile_key: verified.target_profile.key,
        portfolio: ProgramPortfolio {
            alternatives,
            selection: PortfolioSelection {
                policy_key: SELECTION_POLICY_KEY,
                selected_alternative,
            },
        },
        explain,
    })
}

fn build_baseline_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
) -> Result<ProgramAlternative, CompileError> {
    let baseline_regions = build_scheduled_regions(verified)?;
    let baseline_kernels = baseline_regions
        .iter()
        .map(lower_structured_kernel)
        .collect::<Result<Vec<_>, _>>()?;
    let baseline_program = build_kernel_program(verified, &baseline_regions)?;
    assert_kernels_match_program(&baseline_program, &baseline_kernels)?;
    let baseline_artifact = build_artifact_plan(
        semantic,
        verified,
        &baseline_program,
        vec![verified.capabilities.materialized_serial_sum],
    )?;
    let input_bytes =
        verified
            .serial_sum()
            .input_elements
            .checked_mul(4)
            .ok_or(CompileError::NoFeasiblePlan(NoFeasiblePlanError::Request(
                RequestError::ShapeProductOverflow {
                    role: "input-bytes",
                },
            )))?;
    Ok(ProgramAlternative {
        stable_id: "alternative:materialized-serial-sum.v1",
        kind: ProgramAlternativeKind::Materialized,
        scheduled_regions: baseline_regions,
        kernels: baseline_kernels,
        program: baseline_program,
        artifact_plan: baseline_artifact,
        structural_cost: StructuralCost {
            dispatch_count: 2,
            temporary_allocation_count: 1,
            materialized_bytes: input_bytes,
            intermediate_global_reads: input_bytes,
            intermediate_global_writes: input_bytes,
        },
        equivalence: EquivalenceEvidence::MaterializedReference,
    })
}

fn baseline_explain() -> Vec<ExplainRecord> {
    vec![
        accepted(
            ExplainPhase::RequestVerification,
            "compile.request.general-boundary",
            "semantic-program",
        ),
        accepted(
            ExplainPhase::RegionFormation,
            "compile.region.pointwise",
            "region-0",
        ),
        accepted(
            ExplainPhase::RegionFormation,
            "compile.region.strict-sum",
            "region-1",
        ),
        accepted(
            ExplainPhase::RegionFormation,
            "compile.boundary.materialized",
            "pointwise-to-sum",
        ),
        accepted(
            ExplainPhase::IntrinsicSchedule,
            "schedule.coverage-and-ownership",
            "both-regions",
        ),
        accepted(
            ExplainPhase::TargetFeasibility,
            "target.prototype-target-neutral-baseline.v1",
            "both-regions",
        ),
        accepted(
            ExplainPhase::KernelRefinement,
            "kernel.schedule-refinement",
            "both-entries",
        ),
        accepted(
            ExplainPhase::ProgramVerification,
            "program.two-stage-materialized",
            "kernel-program",
        ),
        accepted(
            ExplainPhase::ArtifactPlanning,
            "artifact.neutral-construction-plan",
            "artifact-plan",
        ),
    ]
}

fn consider_fused_alternative(
    semantic: &tiler_ir::semantic::SemanticProgram,
    verified: &crate::request::VerifiedTargetRequest,
    alternatives: &mut Vec<ProgramAlternative>,
    explain: &mut Vec<ExplainRecord>,
) -> Result<(), CompileError> {
    match enumerate_candidates(verified) {
        Err(CandidateError::Budget { .. }) => explain.push(rejected(
            ExplainPhase::CandidateEnumeration,
            "fusion.candidates.budget",
            "candidate:fused-serial-sum",
        )),
        Err(error @ CandidateError::Invalid { .. }) => {
            return Err(CompileError::InvalidCompilerOutput(
                CompilerOutputError::Candidate(error),
            ));
        }
        Ok(candidates) => {
            for candidate in &candidates {
                explain.push(accepted(
                    ExplainPhase::CandidateEnumeration,
                    "fusion.candidate.legal",
                    candidate.stable_id.clone(),
                ));
            }
            let fused_candidate = candidates
                .iter()
                .find(|candidate| candidate.kind == CandidateKind::FusedSerialSum)
                .expect("governed enumeration always includes the fused candidate");
            if let Some(provider) = verified.capabilities.fused_serial_sum {
                let proof = prove_fused_numerics(verified, fused_candidate).map_err(|error| {
                    CompileError::InvalidCompilerOutput(CompilerOutputError::Candidate(error))
                })?;
                explain.push(accepted(
                    ExplainPhase::NumericalLegality,
                    "fusion.strict-f32-equivalence",
                    fused_candidate.stable_id.clone(),
                ));
                match build_fused_scheduled_region(verified) {
                    Err(PhysicalError::Target { .. }) => explain.push(rejected(
                        ExplainPhase::TargetFeasibility,
                        "fusion.target-infeasible",
                        fused_candidate.stable_id.clone(),
                    )),
                    Err(error) => return Err(error.into()),
                    Ok(fused_region) => {
                        let fused_kernel = lower_structured_kernel(&fused_region)?;
                        let fused_program = build_fused_kernel_program(verified, &fused_region)?;
                        assert_kernels_match_program(
                            &fused_program,
                            std::slice::from_ref(&fused_kernel),
                        )?;
                        let fused_artifact = build_artifact_plan(
                            semantic,
                            verified,
                            &fused_program,
                            vec![provider],
                        )?;
                        alternatives.push(fused_alternative(
                            fused_region,
                            fused_kernel,
                            fused_program,
                            fused_artifact,
                            proof,
                        ));
                    }
                }
            } else {
                explain.push(rejected(
                    ExplainPhase::KernelRefinement,
                    "fusion.provider-unavailable",
                    fused_candidate.stable_id.clone(),
                ));
            }
        }
    }
    Ok(())
}

fn fused_alternative(
    region: VerifiedScheduledRegion,
    kernel: VerifiedStructuredKernel,
    program: KernelProgram,
    artifact_plan: ArtifactConstructionPlan,
    proof: FusionNumericalProof,
) -> ProgramAlternative {
    ProgramAlternative {
        stable_id: "alternative:fused-serial-sum.v1",
        kind: ProgramAlternativeKind::Fused,
        scheduled_regions: vec![region],
        kernels: vec![kernel],
        program,
        artifact_plan,
        structural_cost: StructuralCost {
            dispatch_count: 1,
            temporary_allocation_count: 0,
            materialized_bytes: 0,
            intermediate_global_reads: 0,
            intermediate_global_writes: 0,
        },
        equivalence: EquivalenceEvidence::Fused(proof),
    }
}

fn record_selection(
    alternatives: &[ProgramAlternative],
    selected_alternative: usize,
    explain: &mut Vec<ExplainRecord>,
) {
    for (index, alternative) in alternatives.iter().enumerate() {
        explain.push(ExplainRecord {
            phase: ExplainPhase::PortfolioSelection,
            rule: SELECTION_POLICY_KEY,
            subject: alternative.stable_id.to_owned(),
            outcome: if index == selected_alternative {
                ExplainOutcome::Selected
            } else {
                ExplainOutcome::NotSelected
            },
        });
    }
}

fn select_structural_pareto(alternatives: &[ProgramAlternative]) -> usize {
    let mut selected = 0;
    for candidate in 1..alternatives.len() {
        if structurally_dominates(
            alternatives[candidate].structural_cost,
            alternatives[selected].structural_cost,
        ) {
            selected = candidate;
        }
    }
    selected
}

const fn structurally_dominates(candidate: StructuralCost, incumbent: StructuralCost) -> bool {
    let no_worse = candidate.dispatch_count <= incumbent.dispatch_count
        && candidate.temporary_allocation_count <= incumbent.temporary_allocation_count
        && candidate.materialized_bytes <= incumbent.materialized_bytes
        && candidate.intermediate_global_reads <= incumbent.intermediate_global_reads
        && candidate.intermediate_global_writes <= incumbent.intermediate_global_writes;
    let strictly_better = candidate.dispatch_count < incumbent.dispatch_count
        || candidate.temporary_allocation_count < incumbent.temporary_allocation_count
        || candidate.materialized_bytes < incumbent.materialized_bytes
        || candidate.intermediate_global_reads < incumbent.intermediate_global_reads
        || candidate.intermediate_global_writes < incumbent.intermediate_global_writes;
    no_worse && strictly_better
}

fn accepted(phase: ExplainPhase, rule: &'static str, subject: impl Into<String>) -> ExplainRecord {
    ExplainRecord {
        phase,
        rule,
        subject: subject.into(),
        outcome: ExplainOutcome::Accepted,
    }
}

fn rejected(phase: ExplainPhase, rule: &'static str, subject: impl Into<String>) -> ExplainRecord {
    ExplainRecord {
        phase,
        rule,
        subject: subject.into(),
        outcome: ExplainOutcome::Rejected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physical::{ContributorOrder, StructuredBody, TensorRole};
    use crate::program::{DependencyReason, MaterializedValueId, ValueRole};
    use tiler_ir::semantic::{
        F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
        SemanticProgramBuilder, StrictSerialF32Sum,
    };
    use tiler_ir::shape::{Axis, Shape};
    use tiler_reference::{
        FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor,
        TensorPayloadView,
    };

    fn semantic(reverse_constants: bool) -> SemanticProgram {
        semantic_case(
            Shape::from_dims([2, 3]),
            2.0_f32.to_bits(),
            1.0_f32.to_bits(),
            reverse_constants,
        )
    }

    fn semantic_case(
        shape: Shape,
        scale_bits: u32,
        bias_bits: u32,
        reverse_constants: bool,
    ) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), shape)
            .unwrap();
        let (scale, bias) = if reverse_constants {
            let bias = F32Constant::apply(&mut builder, bias_bits).unwrap();
            let scale = F32Constant::apply(&mut builder, scale_bits).unwrap();
            (scale, bias)
        } else {
            let scale = F32Constant::apply(&mut builder, scale_bits).unwrap();
            let bias = F32Constant::apply(&mut builder, bias_bits).unwrap();
            (scale, bias)
        };
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let mapped = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    fn interpret_fused(kernel: &VerifiedStructuredKernel, input: &[f32]) -> Vec<f32> {
        match &kernel.kernel.body {
            StructuredBody::FusedEmptyReduction {
                output_count,
                identity_bits,
                ..
            } => vec![f32::from_bits(*identity_bits); usize::try_from(*output_count).unwrap()],
            StructuredBody::FusedNonEmptySerialReduction {
                output_count,
                contributor_count,
                scale_bits,
                bias_bits,
                canonical_nan_bits,
                contraction,
                prologue_operations,
                ..
            } => {
                assert!(!contraction);
                assert_eq!(
                    prologue_operations,
                    &[
                        crate::physical::BinaryF32::Multiply,
                        crate::physical::BinaryF32::Add,
                    ]
                );
                let scale = f32::from_bits(*scale_bits);
                let bias = f32::from_bits(*bias_bits);
                let canonicalize = |value: f32| {
                    if value.is_nan() {
                        f32::from_bits(*canonical_nan_bits)
                    } else {
                        value
                    }
                };
                let contributors = usize::try_from(*contributor_count).unwrap();
                let outputs = usize::try_from(*output_count).unwrap();
                assert_eq!(input.len(), outputs * contributors);
                input
                    .chunks_exact(contributors)
                    .map(|chunk| {
                        let mut mapped = chunk.iter().map(|value| {
                            let product = canonicalize(*value * scale);
                            canonicalize(product + bias)
                        });
                        let first = mapped.next().expect("non-empty fused reduction");
                        canonicalize(mapped.fold(first, |sum, value| canonicalize(sum + value)))
                    })
                    .collect()
            }
            _ => panic!("expected fused structured body"),
        }
    }

    fn assert_fused_matches_reference(
        shape: Shape,
        values: Vec<f32>,
        scale_bits: u32,
        bias_bits: u32,
    ) {
        let semantic = semantic_case(shape.clone(), scale_bits, bias_bits, false);
        let product = compile(CompilationRequest::governed(&semantic)).unwrap();
        let fused = &product.targets[0].portfolio.alternatives[1];
        let actual = interpret_fused(&fused.kernels[0], &values);
        let key = InputKey::new("input").unwrap();
        let tensor = Tensor::dense(
            F32::resolved_type(),
            shape,
            values
                .into_iter()
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
        let expected = ReferenceEvaluator::standard()
            .unwrap()
            .evaluate(&semantic, &[InputBinding::new(&key, &tensor)])
            .unwrap();
        assert_eq!(
            actual
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>(),
            match expected[0].payload() {
                TensorPayloadView::Dense(elements) => elements
                    .iter()
                    .map(|element| {
                        u32::from_be_bytes(<[u8; 4]>::try_from(element.as_bytes()).unwrap())
                    })
                    .collect::<Vec<_>>(),
                _ => panic!("expected dense f32 reference output"),
            }
        );
    }

    #[test]
    fn product_is_deterministic_and_preserves_the_materialized_boundary() {
        let first = semantic(false);
        let second = semantic(true);
        assert_eq!(
            first.semantic_identity().graph(),
            second.semantic_identity().graph()
        );
        let first = compile(CompilationRequest::governed(&first)).unwrap();
        let second = compile(CompilationRequest::governed(&second)).unwrap();

        assert_eq!(first, second);
        let first = &first.targets[0];
        assert_eq!(first.portfolio.alternatives.len(), 2);
        assert_eq!(first.portfolio.selection.selected_alternative, 1);
        let materialized = &first.portfolio.alternatives[0];
        let fused = &first.portfolio.alternatives[1];
        assert_eq!(materialized.program.stages.len(), 2);
        assert_eq!(
            materialized.program.buffer_plan.values[1].role,
            ValueRole::Temporary
        );
        assert_eq!(
            materialized.program.dependencies[0].reason,
            DependencyReason::Data(MaterializedValueId(1))
        );
        assert_eq!(
            materialized.kernels[0].kernel.buffers[1].tensor,
            TensorRole::Intermediate
        );
        assert_eq!(
            materialized.kernels[1].kernel.buffers[0].tensor,
            TensorRole::Intermediate
        );
        assert!(matches!(
            materialized.kernels[1].kernel.body,
            StructuredBody::NonEmptySerialReduction {
                order: ContributorOrder::OriginalAxisLexicographic,
                loop_start: 1,
                loop_end: 3,
                ..
            }
        ));
        assert_eq!(fused.program.stages.len(), 1);
        assert_eq!(fused.program.buffer_plan.values.len(), 2);
        assert_eq!(
            materialized.structural_cost,
            StructuralCost {
                dispatch_count: 2,
                temporary_allocation_count: 1,
                materialized_bytes: 24,
                intermediate_global_reads: 24,
                intermediate_global_writes: 24,
            }
        );
        assert_eq!(
            fused.structural_cost,
            StructuralCost {
                dispatch_count: 1,
                temporary_allocation_count: 0,
                materialized_bytes: 0,
                intermediate_global_reads: 0,
                intermediate_global_writes: 0,
            }
        );
        assert_eq!(
            materialized.artifact_plan.lowering_providers,
            [crate::request::CompilerCapabilitySnapshot::governed().materialized_serial_sum]
        );
        assert_eq!(
            fused.artifact_plan.lowering_providers,
            [crate::request::CompilerCapabilitySnapshot::governed()
                .fused_serial_sum
                .unwrap()]
        );
        assert!(matches!(
            fused.kernels[0].kernel.body,
            StructuredBody::FusedNonEmptySerialReduction {
                contributor_count: 3,
                loop_start: 1,
                loop_end: 3,
                ..
            }
        ));
        assert!(first.explain.iter().any(|record| {
            record.rule == "compile.boundary.materialized"
                && record.outcome == ExplainOutcome::Accepted
        }));
    }

    #[test]
    fn valid_but_unsupported_program_has_a_capability_failure() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), input)
            .unwrap();
        let semantic = builder.build().unwrap();
        let error = compile(CompilationRequest::governed(&semantic)).unwrap_err();
        assert_eq!(
            error,
            CompileError::UnsupportedCapability(RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "signature",
            })
        );
        assert_eq!(
            error.to_string(),
            "compile.unsupported.strategy.signature: no installed capability can compile this valid semantic program"
        );
    }

    #[test]
    fn budget_exhaustion_is_not_reported_as_unsupported() {
        let semantic = semantic(false);
        let mut request = CompilationRequest::governed(&semantic);
        request.budgets.semantic_operations = 4;
        let error = compile(request).unwrap_err();
        assert_eq!(
            error,
            CompileError::BudgetExhausted(RequestError::BudgetExceeded {
                resource: "semantic-operations",
                limit: 4,
                actual: 5,
            })
        );
    }

    #[test]
    fn malformed_request_is_not_reported_as_missing_capability() {
        let semantic = semantic(false);
        let mut request = CompilationRequest::governed(&semantic);
        request.target_profiles.clear();
        assert_eq!(
            compile(request),
            Err(CompileError::InvalidRequest(RequestError::EmptyTargetSet))
        );
    }

    #[test]
    fn target_resource_failure_is_a_no_feasible_plan_outcome() {
        let semantic = semantic(false);
        let mut request = CompilationRequest::governed(&semantic);
        request.target_profiles[0].max_threads_per_grid_axis = 1;
        let error = compile(request).unwrap_err();
        assert!(matches!(
            error,
            CompileError::NoFeasiblePlan(NoFeasiblePlanError::Physical(PhysicalError::Target {
                rule: "grid-axis",
                region: crate::physical::RegionId(0),
                required: 6,
                available: 1,
            }))
        ));
    }

    #[test]
    fn missing_provider_and_fusion_budget_retain_the_verified_baseline() {
        let semantic = semantic(false);
        let mut missing_provider = CompilationRequest::governed(&semantic);
        missing_provider.capabilities.fused_serial_sum = None;
        let product = compile(missing_provider).unwrap();
        assert_eq!(product.targets[0].portfolio.alternatives.len(), 1);
        assert_eq!(
            product.targets[0].portfolio.selection.selected_alternative,
            0
        );
        assert!(product.targets[0].explain.iter().any(|record| {
            record.rule == "fusion.provider-unavailable"
                && record.outcome == ExplainOutcome::Rejected
        }));

        let mut bounded = CompilationRequest::governed(&semantic);
        bounded.budgets.fusion_candidates = 6;
        let product = compile(bounded).unwrap();
        assert_eq!(product.targets[0].portfolio.alternatives.len(), 1);
        assert!(product.targets[0].explain.iter().any(|record| {
            record.rule == "fusion.candidates.budget" && record.outcome == ExplainOutcome::Rejected
        }));
    }

    #[test]
    fn structural_policy_requires_pareto_dominance_instead_of_guessing_latency() {
        let incumbent = StructuralCost {
            dispatch_count: 2,
            temporary_allocation_count: 0,
            materialized_bytes: 0,
            intermediate_global_reads: 0,
            intermediate_global_writes: 0,
        };
        let tradeoff = StructuralCost {
            dispatch_count: 1,
            temporary_allocation_count: 1,
            materialized_bytes: 4,
            intermediate_global_reads: 4,
            intermediate_global_writes: 4,
        };
        assert!(!structurally_dominates(tradeoff, incumbent));
        assert!(structurally_dominates(
            StructuralCost {
                dispatch_count: 1,
                ..incumbent
            },
            incumbent
        ));
    }

    #[test]
    fn fused_kir_matches_the_authoritative_reference_on_adversarial_f32_cases() {
        assert_fused_matches_reference(
            Shape::from_dims([2, 3]),
            vec![1.0, -2.0, 3.5, f32::MIN_POSITIVE, -0.0, 0.0],
            2.0_f32.to_bits(),
            1.0_f32.to_bits(),
        );
        assert_fused_matches_reference(
            Shape::from_dims([4, 1]),
            vec![-0.0, f32::from_bits(1), f32::INFINITY, f32::NAN],
            1.0_f32.to_bits(),
            0.0_f32.to_bits(),
        );
        assert_fused_matches_reference(
            Shape::from_dims([2, 0]),
            Vec::new(),
            f32::NAN.to_bits(),
            f32::NEG_INFINITY.to_bits(),
        );
        let contraction_input = 1.000_000_1_f32;
        let contraction_scale = 1.000_000_1_f32;
        let contraction_bias = -1.000_000_2_f32;
        assert_ne!(
            (contraction_input * contraction_scale + contraction_bias).to_bits(),
            contraction_input
                .mul_add(contraction_scale, contraction_bias)
                .to_bits(),
            "the conformance vector must distinguish separate operations from FMA"
        );
        assert_fused_matches_reference(
            Shape::from_dims([1, 2]),
            vec![contraction_input, -1.0],
            contraction_scale.to_bits(),
            contraction_bias.to_bits(),
        );
    }
}

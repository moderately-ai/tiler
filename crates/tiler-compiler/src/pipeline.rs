use std::error::Error;
use std::fmt;

use crate::physical::{
    PhysicalError, VerifiedScheduledRegion, VerifiedStructuredKernel, build_scheduled_regions,
    lower_structured_kernel,
};
use crate::program::{
    ArtifactConstructionPlan, KernelProgram, ProgramError, assert_kernels_match_program,
    build_artifact_plan, build_kernel_program, verify_semantic_output_type,
};
use crate::request::{CompilationRequest, RequestError, verify_request};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExplainPhase {
    RequestVerification,
    RegionFormation,
    IntrinsicSchedule,
    TargetFeasibility,
    KernelRefinement,
    ProgramVerification,
    ArtifactPlanning,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExplainOutcome {
    Accepted,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExplainRecord {
    pub(crate) phase: ExplainPhase,
    pub(crate) rule: &'static str,
    pub(crate) subject: &'static str,
    pub(crate) outcome: ExplainOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompilationProduct {
    pub(crate) targets: Vec<TargetCompilationProduct>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TargetCompilationProduct {
    pub(crate) target_profile_key: &'static str,
    pub(crate) scheduled_regions: Vec<VerifiedScheduledRegion>,
    pub(crate) kernels: Vec<VerifiedStructuredKernel>,
    pub(crate) program: KernelProgram,
    pub(crate) artifact_plan: ArtifactConstructionPlan,
    pub(crate) explain: Vec<ExplainRecord>,
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
            | RequestError::DuplicateTargetProfile
            | RequestError::SemanticAuthority => Self::InvalidRequest(value),
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
    let scheduled_regions = build_scheduled_regions(verified)?;
    let kernels = scheduled_regions
        .iter()
        .map(lower_structured_kernel)
        .collect::<Result<Vec<_>, _>>()?;
    let program = build_kernel_program(verified, &scheduled_regions)?;
    assert_kernels_match_program(&program, &kernels)?;
    let artifact_plan = build_artifact_plan(semantic, verified, &program)?;
    let explain = vec![
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
    ];
    Ok(TargetCompilationProduct {
        target_profile_key: verified.target_profile.key,
        scheduled_regions,
        kernels,
        program,
        artifact_plan,
        explain,
    })
}

const fn accepted(phase: ExplainPhase, rule: &'static str, subject: &'static str) -> ExplainRecord {
    ExplainRecord {
        phase,
        rule,
        subject,
        outcome: ExplainOutcome::Accepted,
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

    fn semantic(reverse_constants: bool) -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let (scale, bias) = if reverse_constants {
            let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
            let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
            (scale, bias)
        } else {
            let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
            let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
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

    #[test]
    fn product_is_deterministic_and_preserves_the_materialized_boundary() {
        let first = semantic(false);
        let second = semantic(true);
        assert_eq!(first.canonical_identity(), second.canonical_identity());
        let first = compile(CompilationRequest::governed(&first)).unwrap();
        let second = compile(CompilationRequest::governed(&second)).unwrap();

        assert_eq!(first, second);
        let first = &first.targets[0];
        assert_eq!(first.program.stages.len(), 2);
        assert_eq!(
            first.program.buffer_plan.values[1].role,
            ValueRole::Temporary
        );
        assert_eq!(
            first.program.dependencies[0].reason,
            DependencyReason::Data(MaterializedValueId(1))
        );
        assert_eq!(
            first.kernels[0].kernel.buffers[1].tensor,
            TensorRole::Intermediate
        );
        assert_eq!(
            first.kernels[1].kernel.buffers[0].tensor,
            TensorRole::Intermediate
        );
        assert!(matches!(
            first.kernels[1].kernel.body,
            StructuredBody::NonEmptySerialReduction {
                order: ContributorOrder::OriginalAxisLexicographic,
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
}

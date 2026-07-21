use std::error::Error;
use std::fmt;

use tiler_ir::semantic::{
    CanonicalSemanticAuthorityProjection, CanonicalValueView, F32, F32_CONSTANT_BITS_ATTRIBUTE,
    InputKey, OpKey, OutputKey, REDUCTION_AXES_ATTRIBUTE, SemanticProgram, ValueId, add_f32_op,
    constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
};
use tiler_ir::shape::{Axis, Shape};

const REQUEST_SCHEMA_VERSION: u32 = 1;
const NUMERICAL_CONTRACT_KEY: &str = "tiler.strict-f32.v1";
const TARGET_PROFILE_KEY: &str = "tiler.prototype-target-neutral-baseline.v1";
const BASELINE_PROVIDER_KEY: &str = "tiler.prototype.materialized-serial-sum";
const FUSED_PROVIDER_KEY: &str = "tiler.prototype.fused-serial-sum";
const PROVIDER_REVISION: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StaticShapeEnvironment {
    schema_version: u32,
}

impl StaticShapeEnvironment {
    pub(crate) const fn governed() -> Self {
        Self {
            schema_version: REQUEST_SCHEMA_VERSION,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StrictF32NumericalContract {
    pub(crate) key: &'static str,
    pub(crate) canonical_arithmetic_nan_bits: u32,
    pub(crate) input_subnormals: SubnormalMode,
    pub(crate) result_subnormals: SubnormalMode,
    pub(crate) contraction: NumericalPermission,
    pub(crate) reassociation: NumericalPermission,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SubnormalMode {
    Preserve,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumericalPermission {
    Forbidden,
}

impl StrictF32NumericalContract {
    pub(crate) const fn governed() -> Self {
        Self {
            key: NUMERICAL_CONTRACT_KEY,
            canonical_arithmetic_nan_bits: tiler_ir::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS,
            input_subnormals: SubnormalMode::Preserve,
            result_subnormals: SubnormalMode::Preserve,
            contraction: NumericalPermission::Forbidden,
            reassociation: NumericalPermission::Forbidden,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DeterministicBudgets {
    pub(crate) semantic_values: u32,
    pub(crate) semantic_operations: u32,
    pub(crate) regions: u32,
    pub(crate) host_expression_nodes: u32,
    pub(crate) buffers: u32,
    pub(crate) fusion_candidates: u32,
}

impl DeterministicBudgets {
    pub(crate) const fn governed() -> Self {
        Self {
            semantic_values: 16,
            semantic_operations: 8,
            regions: 2,
            host_expression_nodes: 32,
            buffers: 3,
            fusion_candidates: 7,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LoweringProviderIdentity {
    pub(crate) key: &'static str,
    pub(crate) revision: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CompilerCapabilitySnapshot {
    pub(crate) schema_version: u32,
    pub(crate) materialized_serial_sum: LoweringProviderIdentity,
    pub(crate) fused_serial_sum: Option<LoweringProviderIdentity>,
}

impl CompilerCapabilitySnapshot {
    pub(crate) const fn governed() -> Self {
        Self {
            schema_version: REQUEST_SCHEMA_VERSION,
            materialized_serial_sum: LoweringProviderIdentity {
                key: BASELINE_PROVIDER_KEY,
                revision: PROVIDER_REVISION,
            },
            fused_serial_sum: Some(LoweringProviderIdentity {
                key: FUSED_PROVIDER_KEY,
                revision: PROVIDER_REVISION,
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PrototypeTargetProfile {
    pub(crate) key: &'static str,
    pub(crate) max_threads_per_grid_axis: u64,
    pub(crate) max_threads_per_workgroup: u32,
    pub(crate) max_buffer_bindings_per_entry: u32,
    pub(crate) index_bits: u8,
    pub(crate) supports_device_memory: bool,
    pub(crate) supports_strict_f32: bool,
}

impl PrototypeTargetProfile {
    pub(crate) const fn governed() -> Self {
        Self {
            key: TARGET_PROFILE_KEY,
            max_threads_per_grid_axis: 65_535,
            max_threads_per_workgroup: 1,
            max_buffer_bindings_per_entry: 2,
            index_bits: 64,
            supports_device_memory: true,
            supports_strict_f32: true,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct CompilationRequest<'a> {
    pub(crate) program: &'a SemanticProgram,
    pub(crate) shape_environment: StaticShapeEnvironment,
    pub(crate) numerical_contract: StrictF32NumericalContract,
    pub(crate) budgets: DeterministicBudgets,
    pub(crate) target_profiles: Vec<PrototypeTargetProfile>,
    pub(crate) capabilities: CompilerCapabilitySnapshot,
}

impl<'a> CompilationRequest<'a> {
    pub(crate) fn governed(program: &'a SemanticProgram) -> Self {
        Self {
            program,
            shape_environment: StaticShapeEnvironment::governed(),
            numerical_contract: StrictF32NumericalContract::governed(),
            budgets: DeterministicBudgets::governed(),
            target_profiles: vec![PrototypeTargetProfile::governed()],
            capabilities: CompilerCapabilitySnapshot::governed(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedSerialSum {
    pub(crate) input_key: InputKey,
    pub(crate) output_key: OutputKey,
    pub(crate) input_shape: Shape,
    pub(crate) output_shape: Shape,
    pub(crate) reduction_axes: Vec<Axis>,
    pub(crate) scale_bits: u32,
    pub(crate) bias_bits: u32,
    pub(crate) input: ValueId,
    pub(crate) pointwise_result: ValueId,
    pub(crate) output: ValueId,
    pub(crate) input_elements: u64,
    pub(crate) output_elements: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NormalizedProgram {
    SerialSum(NormalizedSerialSum),
}

impl NormalizedProgram {
    pub(crate) const fn serial_sum(&self) -> &NormalizedSerialSum {
        match self {
            Self::SerialSum(normalized) => normalized,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedCompilationRequest {
    pub(crate) normalized: NormalizedProgram,
    pub(crate) semantic_authority: CanonicalSemanticAuthorityProjection,
    pub(crate) numerical_contract: StrictF32NumericalContract,
    pub(crate) budgets: DeterministicBudgets,
    pub(crate) target_profiles: Vec<PrototypeTargetProfile>,
    pub(crate) capabilities: CompilerCapabilitySnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedTargetRequest {
    pub(crate) normalized: NormalizedProgram,
    pub(crate) semantic_authority: CanonicalSemanticAuthorityProjection,
    pub(crate) numerical_contract: StrictF32NumericalContract,
    pub(crate) budgets: DeterministicBudgets,
    pub(crate) target_profile: PrototypeTargetProfile,
    pub(crate) capabilities: CompilerCapabilitySnapshot,
}

impl VerifiedTargetRequest {
    pub(crate) const fn serial_sum(&self) -> &NormalizedSerialSum {
        self.normalized.serial_sum()
    }
}

impl VerifiedCompilationRequest {
    pub(crate) fn for_target(
        &self,
        target_profile: PrototypeTargetProfile,
    ) -> VerifiedTargetRequest {
        VerifiedTargetRequest {
            normalized: self.normalized.clone(),
            semantic_authority: self.semantic_authority.clone(),
            numerical_contract: self.numerical_contract,
            budgets: self.budgets,
            target_profile,
            capabilities: self.capabilities,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RequestError {
    UnsupportedRequestVersion,
    EmptyTargetSet,
    DuplicateTargetProfile,
    BudgetExceeded {
        resource: &'static str,
        limit: u32,
        actual: usize,
    },
    UnsupportedCapability {
        phase: &'static str,
        rule: &'static str,
    },
    ShapeProductOverflow {
        role: &'static str,
    },
    SemanticAuthority,
}

impl fmt::Display for RequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedRequestVersion => {
                formatter.write_str("compile.request.schema: unsupported static shape environment")
            }
            Self::EmptyTargetSet => formatter
                .write_str("compile.request.targets.empty: at least one target is required"),
            Self::DuplicateTargetProfile => formatter
                .write_str("compile.request.targets.duplicate: target profile keys must be unique"),
            Self::BudgetExceeded {
                resource,
                limit,
                actual,
            } => write!(
                formatter,
                "compile.budget.{resource}: {actual} exceeds deterministic limit {limit}"
            ),
            Self::UnsupportedCapability { phase, rule } => {
                write!(
                    formatter,
                    "compile.unsupported.{phase}.{rule}: no installed capability can compile this valid semantic program"
                )
            }
            Self::ShapeProductOverflow { role } => write!(
                formatter,
                "compile.shape.{role}.element-count: static element count exceeds u64"
            ),
            Self::SemanticAuthority => formatter.write_str(
                "compile.request.semantic-authority: frozen capability projection failed",
            ),
        }
    }
}

impl Error for RequestError {}

pub(crate) fn verify_request(
    request: CompilationRequest<'_>,
) -> Result<VerifiedCompilationRequest, RequestError> {
    if request.shape_environment != StaticShapeEnvironment::governed() {
        return Err(RequestError::UnsupportedRequestVersion);
    }
    let governed_capabilities = CompilerCapabilitySnapshot::governed();
    if request.capabilities.schema_version != governed_capabilities.schema_version
        || request.capabilities.materialized_serial_sum
            != governed_capabilities.materialized_serial_sum
        || request
            .capabilities
            .fused_serial_sum
            .is_some_and(|provider| Some(provider) != governed_capabilities.fused_serial_sum)
    {
        return Err(RequestError::UnsupportedRequestVersion);
    }
    if request.target_profiles.is_empty() {
        return Err(RequestError::EmptyTargetSet);
    }
    if request.numerical_contract != StrictF32NumericalContract::governed() {
        return unsupported("numerics", "strict-f32");
    }
    if request
        .target_profiles
        .iter()
        .any(|target| target.key != PrototypeTargetProfile::governed().key)
    {
        return unsupported("target", "prototype-target-neutral-baseline-v1");
    }
    let mut target_keys: Vec<_> = request
        .target_profiles
        .iter()
        .map(|target| target.key)
        .collect();
    target_keys.sort_unstable();
    if target_keys.windows(2).any(|keys| keys[0] == keys[1]) {
        return Err(RequestError::DuplicateTargetProfile);
    }
    check_budget(
        "semantic-values",
        request.budgets.semantic_values,
        request.program.value_count(),
    )?;
    check_budget(
        "semantic-operations",
        request.budgets.semantic_operations,
        request.program.operation_count(),
    )?;
    check_budget("regions", request.budgets.regions, 2)?;
    check_budget(
        "host-expression-nodes",
        request.budgets.host_expression_nodes,
        9,
    )?;
    check_budget("buffers", request.budgets.buffers, 3)?;

    let normalized = select_supported_strategy(request.program)?;
    let semantic_authority = project_semantic_authority(request.program)?;
    Ok(VerifiedCompilationRequest {
        normalized,
        semantic_authority,
        numerical_contract: request.numerical_contract,
        budgets: request.budgets,
        target_profiles: request.target_profiles,
        capabilities: request.capabilities,
    })
}

fn select_supported_strategy(program: &SemanticProgram) -> Result<NormalizedProgram, RequestError> {
    normalize_serial_sum(program).map(NormalizedProgram::SerialSum)
}

fn project_semantic_authority(
    program: &SemanticProgram,
) -> Result<CanonicalSemanticAuthorityProjection, RequestError> {
    let value_types: Vec<_> = program
        .values()
        .map(|value| value.resolved_type().clone())
        .collect();
    let operations: Vec<_> = program
        .operations()
        .map(|operation| operation.key().clone())
        .collect();
    program
        .semantic_registry()
        .project_reached(value_types.iter(), operations.iter())
        .map_err(|_| RequestError::SemanticAuthority)
}

fn check_budget(resource: &'static str, limit: u32, actual: usize) -> Result<(), RequestError> {
    if usize::try_from(limit).expect("u32 fits every supported host") < actual {
        return Err(RequestError::BudgetExceeded {
            resource,
            limit,
            actual,
        });
    }
    Ok(())
}

fn normalize_serial_sum(program: &SemanticProgram) -> Result<NormalizedSerialSum, RequestError> {
    if program.input_count() != 1 || program.output_count() != 1 || program.operation_count() != 5 {
        return mismatch("signature");
    }
    if program
        .values()
        .any(|value| value.resolved_type() != &F32::resolved_type())
    {
        return mismatch("dtype-f32");
    }

    let input = program.inputs().next().expect("input count checked");
    let output = program.outputs().next().expect("output count checked");
    let sum = producer(program, output.value(), &strict_serial_sum_f32_op())?;
    let sum_operands: Vec<_> = sum.operands().collect();
    let sum_results: Vec<_> = sum.results().collect();
    let [pointwise_result] = sum_operands.as_slice() else {
        return mismatch("sum-signature");
    };
    if sum_results.as_slice() != [output.value()] {
        return mismatch("sum-output");
    }

    let add = producer(program, *pointwise_result, &add_f32_op())?;
    let (multiply_result, bias) = split_tensor_and_scalar(program, &add)?;
    let multiply = producer(program, multiply_result, &multiply_f32_op())?;
    let (tensor_input, scale) = split_tensor_and_scalar(program, &multiply)?;
    if tensor_input != input.value() {
        return mismatch("pointwise-input");
    }
    let scale = constant_bits(program, scale)?;
    let bias = constant_bits(program, bias)?;
    let axes = reduction_axes(sum.attributes())?;

    let input_shape = program
        .shape(input.value())
        .map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "input-handle",
        })?
        .clone();
    if input_shape.rank() == 0 {
        return mismatch("input-rank");
    }
    if program.shape(*pointwise_result).ok() != Some(&input_shape) {
        return mismatch("pointwise-shape");
    }
    let output_shape = input_shape.without_axes(&axes);
    if program.shape(output.value()).ok() != Some(&output_shape) {
        return mismatch("sum-shape");
    }
    let input_elements = element_count_u64(&input_shape, "input")?;
    let output_elements = element_count_u64(&output_shape, "output")?;

    Ok(NormalizedSerialSum {
        input_key: input.key().clone(),
        output_key: output.key().clone(),
        input_shape,
        output_shape,
        reduction_axes: axes,
        scale_bits: scale,
        bias_bits: bias,
        input: input.value(),
        pointwise_result: *pointwise_result,
        output: output.value(),
        input_elements,
        output_elements,
    })
}

fn producer<'a>(
    program: &'a SemanticProgram,
    value: ValueId,
    expected: &OpKey,
) -> Result<tiler_ir::semantic::OperationRef<'a>, RequestError> {
    let operation = program
        .operations()
        .find(|operation| operation.results().any(|result| result == value))
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "missing-producer",
        })?;
    if operation.key() != expected {
        return mismatch("operation-family");
    }
    Ok(operation)
}

fn split_tensor_and_scalar(
    program: &SemanticProgram,
    operation: &tiler_ir::semantic::OperationRef<'_>,
) -> Result<(ValueId, ValueId), RequestError> {
    let operands: Vec<_> = operation.operands().collect();
    let [left, right] = operands.as_slice() else {
        return mismatch("pointwise-arity");
    };
    match (
        program.shape(*left).map(Shape::rank),
        program.shape(*right).map(Shape::rank),
    ) {
        (Ok(left_rank), Ok(0)) if left_rank > 0 => Ok((*left, *right)),
        (Ok(0), Ok(right_rank)) if right_rank > 0 => Ok((*right, *left)),
        _ => mismatch("scalar-broadcast"),
    }
}

fn constant_bits(program: &SemanticProgram, value: ValueId) -> Result<u32, RequestError> {
    let operation = producer(program, value, &constant_f32_op())?;
    if operation.operands().len() != 0 || operation.results().len() != 1 {
        return mismatch("constant-signature");
    }
    let Some(CanonicalValueView::Unsigned(bits)) = operation
        .attributes()
        .get(F32_CONSTANT_BITS_ATTRIBUTE)
        .map(tiler_ir::semantic::CanonicalValue::view)
    else {
        return mismatch("constant-bits");
    };
    u32::try_from(bits).map_err(|_| RequestError::UnsupportedCapability {
        phase: "strategy",
        rule: "constant-bits",
    })
}

fn reduction_axes(
    attributes: &tiler_ir::semantic::OperationAttributes,
) -> Result<Vec<Axis>, RequestError> {
    let Some(CanonicalValueView::Sequence(values)) = attributes
        .get(REDUCTION_AXES_ATTRIBUTE)
        .map(tiler_ir::semantic::CanonicalValue::view)
    else {
        return mismatch("sum-axes");
    };
    values
        .iter()
        .map(|value| {
            let CanonicalValueView::Unsigned(axis) = value.view() else {
                return mismatch("sum-axes");
            };
            u32::try_from(axis)
                .map(Axis::new)
                .map_err(|_| RequestError::UnsupportedCapability {
                    phase: "strategy",
                    rule: "sum-axes",
                })
        })
        .collect()
}

fn element_count_u64(shape: &Shape, role: &'static str) -> Result<u64, RequestError> {
    if shape.extents().iter().any(|extent| extent.get() == 0) {
        return Ok(0);
    }
    shape.extents().iter().try_fold(1_u64, |count, extent| {
        count
            .checked_mul(extent.get())
            .ok_or(RequestError::ShapeProductOverflow { role })
    })
}

fn mismatch<T>(rule: &'static str) -> Result<T, RequestError> {
    unsupported("strategy", rule)
}

fn unsupported<T>(phase: &'static str, rule: &'static str) -> Result<T, RequestError> {
    Err(RequestError::UnsupportedCapability { phase, rule })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tiler_ir::semantic::{
        F32Add, F32Constant, F32Multiply, SemanticProgramBuilder, StrictSerialF32Sum,
    };

    fn program() -> SemanticProgram {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
        let bias = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
        let product = F32Multiply::apply(&mut builder, input, scale).unwrap();
        let pointwise = F32Add::apply(&mut builder, product, bias).unwrap();
        let sum = StrictSerialF32Sum::apply(&mut builder, pointwise, [Axis::new(1)]).unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), sum)
            .unwrap();
        builder.build().unwrap()
    }

    #[test]
    fn governed_request_selects_the_supported_serial_sum_strategy() {
        let program = program();
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
        let normalized = verified.normalized.serial_sum();
        assert_eq!(normalized.input_shape, Shape::from_dims([2, 3]));
        assert_eq!(normalized.output_shape, Shape::from_dims([2]));
        assert_eq!(normalized.reduction_axes, [Axis::new(1)]);
        assert_eq!(normalized.scale_bits, 2.0_f32.to_bits());
        assert_eq!(normalized.bias_bits, 1.0_f32.to_bits());
        assert_eq!(normalized.input_elements, 6);
        assert_eq!(normalized.output_elements, 2);
        assert_eq!(
            verified.target_profiles,
            [PrototypeTargetProfile::governed()]
        );
    }

    #[test]
    fn request_rejects_profile_and_budget_mismatches_stably() {
        let program = program();
        let mut request = CompilationRequest::governed(&program);
        request.budgets.semantic_operations = 4;
        assert_eq!(
            verify_request(request),
            Err(RequestError::BudgetExceeded {
                resource: "semantic-operations",
                limit: 4,
                actual: 5,
            })
        );

        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 3]))
            .unwrap();
        builder
            .output(OutputKey::new("result").unwrap(), input)
            .unwrap();
        let invalid = builder.build().unwrap();
        assert_eq!(
            verify_request(CompilationRequest::governed(&invalid)),
            Err(RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "signature",
            })
        );
    }

    #[test]
    fn request_requires_a_nonempty_unique_target_set() {
        let program = program();
        let mut empty = CompilationRequest::governed(&program);
        empty.target_profiles.clear();
        assert_eq!(verify_request(empty), Err(RequestError::EmptyTargetSet));

        let mut duplicate = CompilationRequest::governed(&program);
        duplicate
            .target_profiles
            .push(PrototypeTargetProfile::governed());
        assert_eq!(
            verify_request(duplicate),
            Err(RequestError::DuplicateTargetProfile)
        );
    }
}

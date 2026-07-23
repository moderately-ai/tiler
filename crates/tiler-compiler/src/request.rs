use std::error::Error;
use std::fmt;

use tiler_ir::semantic::{
    CanonicalIntegerWidth, CanonicalValueView, F32, F32_CONSTANT_BITS_ATTRIBUTE, InputKey, OpKey,
    OutputKey, REDUCTION_AXES_ATTRIBUTE, SemanticIdentity, SemanticProgram, TypeKey, ValueId,
    add_f32_op, constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
};
use tiler_ir::shape::{Axis, Shape};

use crate::region::SemanticMemberId;

const REQUEST_SCHEMA_VERSION: u32 = 1;
const NUMERICAL_CONTRACT_KEY: &str = "tiler.strict-f32.v1";
const TARGET_PROFILE_KEY: &str = "tiler.prototype-target-neutral-baseline.v1";
const BASELINE_PROVIDER_KEY: &str = "tiler.prototype.materialized-serial-sum";
const FUSED_PROVIDER_KEY: &str = "tiler.prototype.fused-serial-sum";
const PROVIDER_REVISION: u32 = 1;
/// Recognized operation count when both pointwise constants are one shared value.
const RECOGNIZED_OPERATIONS_MIN: usize = 4;
/// Recognized operation count when each pointwise constant is a distinct value.
const RECOGNIZED_OPERATIONS_MAX: usize = 5;

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
    /// Rewrites the deterministic normalization stage may commit.
    ///
    /// Normalization visits each verified operation exactly once, so its
    /// traversal is already bounded by `semantic_operations`. This is the
    /// stage's own explicit budget over committed rewrites.
    pub(crate) normalization_rewrites: u32,
    /// Semantic occurrences admitted in one region candidate.
    pub(crate) region_members: u32,
    /// Retained boundary outputs admitted for one region candidate.
    pub(crate) region_boundary_outputs: u32,
    /// Boundary and member-result values live across one region candidate.
    pub(crate) region_live_values: u32,
    /// Grown candidates admitted for one seed occurrence.
    ///
    /// Singleton coverage is emitted before growth starts and is never bounded
    /// by this budget, so exhausting it loses fused alternatives rather than the
    /// unfused plan.
    pub(crate) region_candidates_per_seed: u32,
    /// Candidate expansion attempts admitted for one compilation request.
    pub(crate) region_expansions: u32,
}

impl DeterministicBudgets {
    #[cfg(test)]
    pub(crate) const fn governed() -> Self {
        Self {
            semantic_values: 16,
            semantic_operations: 8,
            regions: 2,
            host_expression_nodes: 32,
            buffers: 3,
            normalization_rewrites: 8,
            region_members: 32,
            region_boundary_outputs: 8,
            region_live_values: 64,
            region_candidates_per_seed: 32,
            region_expansions: 10_000,
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

impl CompilationRequest<'_> {
    #[cfg(test)]
    pub(crate) fn governed(program: &SemanticProgram) -> CompilationRequest<'_> {
        CompilationRequest {
            program,
            shape_environment: StaticShapeEnvironment::governed(),
            numerical_contract: StrictF32NumericalContract::governed(),
            budgets: DeterministicBudgets::governed(),
            target_profiles: vec![PrototypeTargetProfile::governed()],
            capabilities: CompilerCapabilitySnapshot::governed(),
        }
    }
}

/// The recognized serial-sum occurrences as canonical region member sets.
///
/// The strategy recognizer already walks the verified program to identify these
/// operations, so the exact occurrences it matched are retained instead of being
/// re-encoded as a fixed role vocabulary downstream. Only the ascending member
/// sets are retained: two programs that `tiler-ir` gives one canonical graph
/// identity may store the pointwise constants in either order, and the recognized
/// coverage must not depend on which spelling the caller authored. A shared
/// pointwise constant simply contributes one member instead of two.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RecognizedSerialSumMembers {
    pointwise: Vec<SemanticMemberId>,
    reduction: Vec<SemanticMemberId>,
}

impl RecognizedSerialSumMembers {
    fn new(scale_constant: u32, multiply: u32, bias_constant: u32, add: u32, sum: u32) -> Self {
        Self {
            pointwise: ascending([scale_constant, multiply, bias_constant, add]),
            reduction: ascending([sum]),
        }
    }

    /// Returns the pointwise prologue members in ascending order.
    pub(crate) fn pointwise(&self) -> &[SemanticMemberId] {
        &self.pointwise
    }

    /// Returns the reduction members in ascending order.
    pub(crate) fn reduction(&self) -> &[SemanticMemberId] {
        &self.reduction
    }

    /// Returns every recognized member in ascending order.
    pub(crate) fn all(&self) -> Vec<SemanticMemberId> {
        let mut members: Vec<_> = self
            .pointwise
            .iter()
            .chain(&self.reduction)
            .copied()
            .collect();
        members.sort_unstable();
        members.dedup();
        members
    }
}

fn ascending<const N: usize>(ordinals: [u32; N]) -> Vec<SemanticMemberId> {
    let mut ordinals = ordinals;
    ordinals.sort_unstable();
    let mut members: Vec<_> = ordinals.into_iter().map(SemanticMemberId).collect();
    members.dedup();
    members
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
    pub(crate) members: RecognizedSerialSumMembers,
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

    #[cfg(test)]
    fn serial_sum_mut(&mut self) -> &mut NormalizedSerialSum {
        match self {
            Self::SerialSum(normalized) => normalized,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedCompilationRequest {
    normalized: NormalizedProgram,
    semantic_identity: SemanticIdentity,
    numerical_contract: StrictF32NumericalContract,
    budgets: DeterministicBudgets,
    target_profiles: Vec<PrototypeTargetProfile>,
    capabilities: CompilerCapabilitySnapshot,
    authorities: Vec<VerifiedRequestSubject>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedTargetRequest {
    normalized: NormalizedProgram,
    semantic_identity: SemanticIdentity,
    numerical_contract: StrictF32NumericalContract,
    budgets: DeterministicBudgets,
    target_profile: PrototypeTargetProfile,
    capabilities: CompilerCapabilitySnapshot,
    authority: VerifiedRequestSubject,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedRequestSubject {
    normalized: NormalizedSerialSumSubject,
    semantic_identity: SemanticIdentity,
    numerical_contract: StrictF32NumericalContract,
    budgets: DeterministicBudgets,
    target_profile: PrototypeTargetProfile,
    capabilities: CompilerCapabilitySnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NormalizedSerialSumSubject {
    input_key: InputKey,
    output_key: OutputKey,
    input_shape: Shape,
    output_shape: Shape,
    reduction_axes: Vec<Axis>,
    scale_bits: u32,
    bias_bits: u32,
    members: RecognizedSerialSumMembers,
    input_elements: u64,
    output_elements: u64,
}

impl VerifiedTargetRequest {
    pub(crate) const fn serial_sum(&self) -> &NormalizedSerialSum {
        self.normalized.serial_sum()
    }

    pub(crate) fn subject(&self) -> VerifiedRequestSubject {
        request_subject(
            &self.normalized,
            &self.semantic_identity,
            self.numerical_contract,
            self.budgets,
            self.target_profile,
            self.capabilities,
        )
    }

    pub(crate) fn is_authoritative(&self) -> bool {
        self.subject() == self.authority
    }

    pub(crate) const fn numerical_contract(&self) -> StrictF32NumericalContract {
        self.numerical_contract
    }

    pub(crate) const fn budgets(&self) -> DeterministicBudgets {
        self.budgets
    }

    pub(crate) const fn target_profile(&self) -> PrototypeTargetProfile {
        self.target_profile
    }

    pub(crate) const fn capabilities(&self) -> CompilerCapabilitySnapshot {
        self.capabilities
    }

    pub(crate) const fn semantic_identity(&self) -> &SemanticIdentity {
        &self.semantic_identity
    }
}

impl VerifiedRequestSubject {
    pub(crate) const fn normalized(&self) -> &NormalizedSerialSumSubject {
        &self.normalized
    }

    pub(crate) const fn numerical_contract(&self) -> StrictF32NumericalContract {
        self.numerical_contract
    }

    pub(crate) fn canonical_explain_subject_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"tiler.compiler.request-subject.v1\0");
        encode_explain_bytes(&mut bytes, self.semantic_identity.graph().as_bytes());
        encode_explain_bytes(
            &mut bytes,
            self.semantic_identity.reached_definitions().as_bytes(),
        );
        encode_explain_bytes(
            &mut bytes,
            self.semantic_identity.admission_provenance().as_bytes(),
        );
        encode_explain_bytes(
            &mut bytes,
            self.semantic_identity.registry_snapshot().as_bytes(),
        );
        encode_explain_bytes(&mut bytes, self.normalized.input_key.as_str().as_bytes());
        encode_explain_bytes(&mut bytes, self.normalized.output_key.as_str().as_bytes());
        encode_explain_shape(&mut bytes, &self.normalized.input_shape);
        encode_explain_shape(&mut bytes, &self.normalized.output_shape);
        bytes.extend_from_slice(
            &u64::try_from(self.normalized.reduction_axes.len())
                .unwrap_or(u64::MAX)
                .to_be_bytes(),
        );
        for axis in &self.normalized.reduction_axes {
            bytes.extend_from_slice(&axis.get().to_be_bytes());
        }
        bytes.extend_from_slice(&self.normalized.scale_bits.to_be_bytes());
        bytes.extend_from_slice(&self.normalized.bias_bits.to_be_bytes());
        for members in [
            self.normalized.members.pointwise(),
            self.normalized.members.reduction(),
        ] {
            bytes.extend_from_slice(
                &u64::try_from(members.len())
                    .unwrap_or(u64::MAX)
                    .to_be_bytes(),
            );
            for member in members {
                bytes.extend_from_slice(&member.0.to_be_bytes());
            }
        }
        bytes.extend_from_slice(&self.normalized.input_elements.to_be_bytes());
        bytes.extend_from_slice(&self.normalized.output_elements.to_be_bytes());
        encode_explain_bytes(&mut bytes, self.numerical_contract.key.as_bytes());
        bytes.extend_from_slice(
            &self
                .numerical_contract
                .canonical_arithmetic_nan_bits
                .to_be_bytes(),
        );
        bytes.push(self.numerical_contract.input_subnormals as u8);
        bytes.push(self.numerical_contract.result_subnormals as u8);
        bytes.push(self.numerical_contract.contraction as u8);
        bytes.push(self.numerical_contract.reassociation as u8);
        for budget in [
            self.budgets.semantic_values,
            self.budgets.semantic_operations,
            self.budgets.regions,
            self.budgets.host_expression_nodes,
            self.budgets.buffers,
            self.budgets.normalization_rewrites,
            self.budgets.region_members,
            self.budgets.region_boundary_outputs,
            self.budgets.region_live_values,
            self.budgets.region_candidates_per_seed,
            self.budgets.region_expansions,
        ] {
            bytes.extend_from_slice(&budget.to_be_bytes());
        }
        encode_explain_bytes(&mut bytes, self.target_profile.key.as_bytes());
        bytes.extend_from_slice(&self.target_profile.max_threads_per_grid_axis.to_be_bytes());
        bytes.extend_from_slice(&self.target_profile.max_threads_per_workgroup.to_be_bytes());
        bytes.extend_from_slice(
            &self
                .target_profile
                .max_buffer_bindings_per_entry
                .to_be_bytes(),
        );
        bytes.push(self.target_profile.index_bits);
        bytes.push(u8::from(self.target_profile.supports_device_memory));
        bytes.push(u8::from(self.target_profile.supports_strict_f32));
        bytes.extend_from_slice(&self.capabilities.schema_version.to_be_bytes());
        encode_explain_provider(&mut bytes, self.capabilities.materialized_serial_sum);
        match self.capabilities.fused_serial_sum {
            Some(provider) => {
                bytes.push(1);
                encode_explain_provider(&mut bytes, provider);
            }
            None => bytes.push(0),
        }
        bytes
    }
}

fn encode_explain_bytes(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(&u64::try_from(value.len()).unwrap_or(u64::MAX).to_be_bytes());
    output.extend_from_slice(value);
}

fn encode_explain_shape(output: &mut Vec<u8>, shape: &Shape) {
    output.extend_from_slice(
        &u64::try_from(shape.rank())
            .unwrap_or(u64::MAX)
            .to_be_bytes(),
    );
    for extent in shape.extents() {
        output.extend_from_slice(&extent.get().to_be_bytes());
    }
}

fn encode_explain_provider(output: &mut Vec<u8>, provider: LoweringProviderIdentity) {
    encode_explain_bytes(output, provider.key.as_bytes());
    output.extend_from_slice(&provider.revision.to_be_bytes());
}

impl NormalizedSerialSumSubject {
    pub(crate) const fn input_shape(&self) -> &Shape {
        &self.input_shape
    }
    pub(crate) const fn output_shape(&self) -> &Shape {
        &self.output_shape
    }
    pub(crate) fn reduction_axes(&self) -> &[Axis] {
        &self.reduction_axes
    }
    pub(crate) const fn scale_bits(&self) -> u32 {
        self.scale_bits
    }
    pub(crate) const fn bias_bits(&self) -> u32 {
        self.bias_bits
    }
    pub(crate) const fn members(&self) -> &RecognizedSerialSumMembers {
        &self.members
    }
    pub(crate) const fn input_elements(&self) -> u64 {
        self.input_elements
    }
    pub(crate) const fn output_elements(&self) -> u64 {
        self.output_elements
    }
}

impl VerifiedCompilationRequest {
    pub(crate) fn target_profiles(&self) -> &[PrototypeTargetProfile] {
        &self.target_profiles
    }

    /// Returns the verified deterministic budgets bound to this request.
    pub(crate) const fn budgets(&self) -> DeterministicBudgets {
        self.budgets
    }

    /// Returns the verified numerical contract bound to this request.
    pub(crate) const fn numerical_contract(&self) -> StrictF32NumericalContract {
        self.numerical_contract
    }

    pub(crate) fn for_target(
        &self,
        target_profile: PrototypeTargetProfile,
    ) -> Result<VerifiedTargetRequest, RequestError> {
        let Some(index) = self
            .target_profiles
            .iter()
            .position(|profile| *profile == target_profile)
        else {
            return Err(RequestError::UnverifiedTargetSelection);
        };
        let current_authority = request_subject(
            &self.normalized,
            &self.semantic_identity,
            self.numerical_contract,
            self.budgets,
            target_profile,
            self.capabilities,
        );
        if target_profile != PrototypeTargetProfile::governed()
            || self
                .target_profiles
                .iter()
                .any(|profile| *profile != PrototypeTargetProfile::governed())
            || self.numerical_contract != StrictF32NumericalContract::governed()
            || self.authorities.get(index) != Some(&current_authority)
        {
            return Err(RequestError::UnverifiedTargetSelection);
        }
        Ok(VerifiedTargetRequest {
            normalized: self.normalized.clone(),
            semantic_identity: self.semantic_identity.clone(),
            numerical_contract: self.numerical_contract,
            budgets: self.budgets,
            target_profile,
            capabilities: self.capabilities,
            authority: current_authority,
        })
    }
}

fn request_subject(
    normalized: &NormalizedProgram,
    semantic_identity: &SemanticIdentity,
    numerical_contract: StrictF32NumericalContract,
    budgets: DeterministicBudgets,
    target_profile: PrototypeTargetProfile,
    capabilities: CompilerCapabilitySnapshot,
) -> VerifiedRequestSubject {
    let normalized = normalized.serial_sum();
    VerifiedRequestSubject {
        normalized: NormalizedSerialSumSubject {
            input_key: normalized.input_key.clone(),
            output_key: normalized.output_key.clone(),
            input_shape: normalized.input_shape.clone(),
            output_shape: normalized.output_shape.clone(),
            reduction_axes: normalized.reduction_axes.clone(),
            scale_bits: normalized.scale_bits,
            bias_bits: normalized.bias_bits,
            members: normalized.members.clone(),
            input_elements: normalized.input_elements,
            output_elements: normalized.output_elements,
        },
        semantic_identity: semantic_identity.clone(),
        numerical_contract,
        budgets,
        target_profile,
        capabilities,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RequestError {
    UnsupportedRequestVersion,
    EmptyTargetSet,
    DuplicateTargetProfile,
    UnverifiedTargetSelection,
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
            Self::UnverifiedTargetSelection => formatter.write_str(
                "compile.request.targets.selection: target was not verified by the request",
            ),
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
        .any(|target| *target != PrototypeTargetProfile::governed())
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
    let semantic_identity = request.program.semantic_identity().clone();
    let authorities = request
        .target_profiles
        .iter()
        .map(|target| {
            request_subject(
                &normalized,
                &semantic_identity,
                request.numerical_contract,
                request.budgets,
                *target,
                request.capabilities,
            )
        })
        .collect();
    Ok(VerifiedCompilationRequest {
        normalized,
        semantic_identity,
        numerical_contract: request.numerical_contract,
        budgets: request.budgets,
        target_profiles: request.target_profiles,
        capabilities: request.capabilities,
        authorities,
    })
}

fn select_supported_strategy(program: &SemanticProgram) -> Result<NormalizedProgram, RequestError> {
    normalize_serial_sum(program).map(NormalizedProgram::SerialSum)
}

fn check_budget(resource: &'static str, limit: u32, actual: usize) -> Result<(), RequestError> {
    if u64::try_from(actual).map_or(true, |actual| actual > u64::from(limit)) {
        return Err(RequestError::BudgetExceeded {
            resource,
            limit,
            actual,
        });
    }
    Ok(())
}

fn normalize_serial_sum(program: &SemanticProgram) -> Result<NormalizedSerialSum, RequestError> {
    // The recognized structure is exactly one reduction, two pointwise
    // operations, and one or two constants; a shared constant is the normalized
    // spelling of the same program. The exact count is pinned against the
    // distinct recognized set once the structural walk has identified it.
    if program.input_count() != 1
        || program.output_count() != 1
        || !(RECOGNIZED_OPERATIONS_MIN..=RECOGNIZED_OPERATIONS_MAX)
            .contains(&program.operation_count())
    {
        return mismatch("signature");
    }
    if program
        .values()
        .any(|value| value.resolved_type() != &F32::resolved_type())
    {
        return mismatch("dtype-f32");
    }

    let input = program
        .inputs()
        .next()
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "missing-input",
        })?;
    let output = program
        .outputs()
        .next()
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "missing-output",
        })?;
    let (sum_operation, sum) = producer(program, output.value(), &strict_serial_sum_f32_op())?;
    let sum_operands: Vec<_> = sum.operands().collect();
    let sum_results: Vec<_> = sum.results().collect();
    let [pointwise_result] = sum_operands.as_slice() else {
        return mismatch("sum-signature");
    };
    if sum_results.as_slice() != [output.value()] {
        return mismatch("sum-output");
    }

    let (add_operation, add) = producer(program, *pointwise_result, &add_f32_op())?;
    let (multiply_result, bias) = split_tensor_and_scalar(program, &add)?;
    let (multiply_operation, multiply) = producer(program, multiply_result, &multiply_f32_op())?;
    let (tensor_input, scale) = split_tensor_and_scalar(program, &multiply)?;
    if tensor_input != input.value() {
        return mismatch("pointwise-input");
    }
    let (scale, scale_operation) = constant_bits(program, scale)?;
    let (bias, bias_operation) = constant_bits(program, bias)?;
    let members = RecognizedSerialSumMembers::new(
        scale_operation,
        multiply_operation,
        bias_operation,
        add_operation,
        sum_operation,
    );

    check_recognized_operation_cover(program, &members)?;
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
    check_canonical_reduction_axes(&axes, input_shape.rank())?;
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
        members,
        input: input.value(),
        pointwise_result: *pointwise_result,
        output: output.value(),
        input_elements,
        output_elements,
    })
}

/// Requires reduction axes to be in range and in strictly ascending order.
fn check_canonical_reduction_axes(axes: &[Axis], rank: usize) -> Result<(), RequestError> {
    let mut previous = None;
    for axis in axes {
        let index =
            usize::try_from(axis.get()).map_err(|_| RequestError::UnsupportedCapability {
                phase: "strategy",
                rule: "sum-axis-range",
            })?;
        if index >= rank {
            return mismatch("sum-axis-range");
        }
        if previous.is_some_and(|previous| previous >= axis.get()) {
            return mismatch("sum-axes-canonical");
        }
        previous = Some(axis.get());
    }
    Ok(())
}

/// Requires the recognized operations to cover the whole program exactly.
///
/// A built program retains only output-reachable operations, so demanding that
/// the reachable count equal the distinct recognized set rejects any operation
/// outside this exact structure. One constant shared by both pointwise operands
/// is the normalized spelling of the same program and covers four distinct
/// operations instead of five.
fn check_recognized_operation_cover(
    program: &SemanticProgram,
    recognized: &RecognizedSerialSumMembers,
) -> Result<(), RequestError> {
    if program.operation_count() != recognized.all().len() {
        return mismatch("signature");
    }
    Ok(())
}

fn producer<'a>(
    program: &'a SemanticProgram,
    value: ValueId,
    expected: &OpKey,
) -> Result<(u32, tiler_ir::semantic::OperationRef<'a>), RequestError> {
    let (ordinal, operation) = program
        .operations()
        .enumerate()
        .find(|(_, operation)| operation.results().any(|result| result == value))
        .ok_or(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "missing-producer",
        })?;
    if operation.key() != expected {
        return mismatch("operation-family");
    }
    let ordinal = u32::try_from(ordinal).map_err(|_| RequestError::UnsupportedCapability {
        phase: "strategy",
        rule: "operation-ordinal",
    })?;
    Ok((ordinal, operation))
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

fn constant_bits(program: &SemanticProgram, value: ValueId) -> Result<(u32, u32), RequestError> {
    let (ordinal, operation) = producer(program, value, &constant_f32_op())?;
    if operation.operands().len() != 0 || operation.results().len() != 1 {
        return mismatch("constant-signature");
    }
    let Some(CanonicalValueView::FloatBits(bits)) = operation
        .attributes()
        .get(F32_CONSTANT_BITS_ATTRIBUTE)
        .map(tiler_ir::semantic::CanonicalValue::view)
    else {
        return mismatch("constant-bits");
    };
    let governed_f32 =
        TypeKey::new("tiler", "f32", 1).map_err(|_| RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "governed-f32-key",
        })?;
    if bits.format() != &governed_f32 {
        return mismatch("constant-bits-format");
    }
    <[u8; 4]>::try_from(bits.bits())
        .map(|bytes| (u32::from_be_bytes(bytes), ordinal))
        .map_err(|_| RequestError::UnsupportedCapability {
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
            let CanonicalValueView::Unsigned { width, bits } = value.view() else {
                return mismatch("sum-axes");
            };
            if width != CanonicalIntegerWidth::Bits32 {
                return mismatch("sum-axes-width");
            }
            u32::try_from(bits)
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
    use std::sync::Arc;

    use super::*;
    use tiler_ir::semantic::{
        CanonicalValue, CanonicalValueKind, F32Add, F32Constant, F32Multiply,
        NormativeDefinitionRef, OperationArity, OperationAttributeSchema, OperationConformance,
        OperationDefinition, OperationDefinitionFacts, OperationEffect, OperationInferenceError,
        OperationInferencer, OperationSchema, ProviderDiagnosticCode, ProviderIdentity,
        RegistryError, SemanticProgramBuilder, SemanticRegistryBuilder, SemanticRegistryProvider,
        SemanticRegistryRegistrar, StrictSerialF32Sum, TypeDefinitionFacts, ValueFact,
        ValueTypeDefinition, ValueTypeDefinitionKey,
    };

    fn diagnostic_code(value: &str) -> ProviderDiagnosticCode {
        ProviderDiagnosticCode::new(value).unwrap()
    }

    fn program() -> SemanticProgram {
        program_with_builder(SemanticProgramBuilder::try_standard().unwrap())
    }

    fn program_with_builder(mut builder: SemanticProgramBuilder) -> SemanticProgram {
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

    #[derive(Clone, Copy)]
    enum TestOperation {
        Constant,
        Binary,
        Sum,
    }

    impl OperationInferencer for TestOperation {
        fn infer(
            &self,
            request: tiler_ir::semantic::OperationInferenceRequest<'_>,
            outputs: &mut tiler_ir::semantic::OperationInferenceOutputs<'_>,
        ) -> Result<(), OperationInferenceError> {
            let operands = request.operands();
            let attributes = request.attributes();
            match self {
                Self::Constant => {
                    outputs.try_push(ValueFact::new(F32::resolved_type(), Shape::new([])))
                }
                Self::Binary => {
                    let left = operands[0].shape();
                    let right = operands[1].shape();
                    let shape = if left.rank() == 0 {
                        right.clone()
                    } else if right.rank() == 0 || left == right {
                        left.clone()
                    } else {
                        return Err(OperationInferenceError::new(
                            diagnostic_code("test.binary.shape"),
                            "operands must have equal shapes or include one scalar",
                        )
                        .unwrap());
                    };
                    outputs.try_push(ValueFact::new(F32::resolved_type(), shape))
                }
                Self::Sum => {
                    let Some(CanonicalValueView::Sequence(values)) = attributes
                        .get(REDUCTION_AXES_ATTRIBUTE)
                        .map(CanonicalValue::view)
                    else {
                        return Err(OperationInferenceError::new(
                            diagnostic_code("test.sum.axes"),
                            "sum axes must be a sequence",
                        )
                        .unwrap());
                    };
                    let axes = values
                        .iter()
                        .map(|value| match value.view() {
                            CanonicalValueView::Unsigned {
                                width: CanonicalIntegerWidth::Bits32,
                                bits,
                            } => u32::try_from(bits).map(Axis::new).map_err(|_| {
                                OperationInferenceError::new(
                                    diagnostic_code("test.sum.axis-width"),
                                    "sum axis exceeds u32",
                                )
                                .unwrap()
                            }),
                            _ => Err(OperationInferenceError::new(
                                diagnostic_code("test.sum.axis-kind"),
                                "sum axes must be u32 values",
                            )
                            .unwrap()),
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    outputs.try_push(ValueFact::new(
                        F32::resolved_type(),
                        operands[0].shape().without_axes(&axes),
                    ))
                }
            }
        }
    }

    struct GovernedTestSemantics {
        revision: u32,
    }

    impl SemanticRegistryProvider for GovernedTestSemantics {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("tiler-test", "governed-semantics", self.revision).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            registrar.register_marked_value_type::<F32>(
                ValueTypeDefinition::structurally_valid(
                    ValueTypeDefinitionKey::Nominal(
                        TypeKey::new("tiler", "f32", 1).expect("the test F32 key is valid"),
                    ),
                    NormativeDefinitionRef::new("test binary32 semantics")?,
                    TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
                ),
                F32::resolved_type(),
            )?;
            register_test_operation(
                registrar,
                constant_f32_op(),
                0,
                [OperationAttributeSchema::required(
                    F32_CONSTANT_BITS_ATTRIBUTE,
                    CanonicalValueKind::FloatBits,
                )],
                TestOperation::Constant,
            )?;
            register_test_operation(registrar, multiply_f32_op(), 2, [], TestOperation::Binary)?;
            register_test_operation(registrar, add_f32_op(), 2, [], TestOperation::Binary)?;
            register_test_operation(
                registrar,
                strict_serial_sum_f32_op(),
                1,
                [OperationAttributeSchema::required(
                    REDUCTION_AXES_ATTRIBUTE,
                    CanonicalValueKind::Sequence,
                )],
                TestOperation::Sum,
            )
        }
    }

    fn register_test_operation<const N: usize>(
        registrar: &mut SemanticRegistryRegistrar<'_>,
        key: OpKey,
        operands: u32,
        attributes: [OperationAttributeSchema; N],
        inferencer: TestOperation,
    ) -> Result<(), RegistryError> {
        registrar.register_operation(OperationDefinition::new(
            key,
            OperationSchema::new(
                OperationArity::exact(operands),
                OperationArity::exact(1),
                attributes,
            )
            .expect("the test operation schema is valid"),
            NormativeDefinitionRef::new("test governed operation semantics")?,
            OperationDefinitionFacts::new(CanonicalValue::boolean(true)),
            OperationConformance::new(CanonicalValue::boolean(true)),
            OperationEffect::Pure,
            Arc::new(inferencer),
        ))
    }

    fn governed_test_program(revision: u32) -> SemanticProgram {
        let mut registry = SemanticRegistryBuilder::new();
        registry
            .register_provider(&GovernedTestSemantics { revision })
            .unwrap();
        program_with_builder(SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap())
    }

    struct UnusedSemantics {
        revision: u32,
    }

    impl SemanticRegistryProvider for UnusedSemantics {
        fn identity(&self) -> ProviderIdentity {
            ProviderIdentity::new("tiler-test", "unused-semantics", self.revision).unwrap()
        }

        fn register(
            &self,
            registrar: &mut SemanticRegistryRegistrar<'_>,
        ) -> Result<(), RegistryError> {
            registrar.register_value_type(ValueTypeDefinition::structurally_valid(
                ValueTypeDefinitionKey::Nominal(
                    TypeKey::new("tiler-test", "unused", 1).expect("the test key is valid"),
                ),
                NormativeDefinitionRef::new("unused test semantics")?,
                TypeDefinitionFacts::new(CanonicalValue::boolean(true)),
            ))
        }
    }

    fn program_with_unused_provider(revision: u32) -> SemanticProgram {
        let mut registry = SemanticRegistryBuilder::standard().unwrap();
        registry
            .register_provider(&UnusedSemantics { revision })
            .unwrap();
        program_with_builder(SemanticProgramBuilder::try_new(registry.freeze().unwrap()).unwrap())
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

    #[test]
    fn verified_request_receipts_reject_post_verification_mutation() {
        let program = program();
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
        let governed_target = PrototypeTargetProfile::governed();

        let mut forged = verified.clone();
        forged.budgets.buffers += 1;
        assert_eq!(
            forged.for_target(governed_target),
            Err(RequestError::UnverifiedTargetSelection)
        );

        let mut forged = verified.clone();
        forged.capabilities.materialized_serial_sum.revision += 1;
        assert_eq!(
            forged.for_target(governed_target),
            Err(RequestError::UnverifiedTargetSelection)
        );

        let mut forged = verified.clone();
        forged.target_profiles[0].max_threads_per_grid_axis -= 1;
        assert_eq!(
            forged.for_target(governed_target),
            Err(RequestError::UnverifiedTargetSelection)
        );

        let mut forged = verified.clone();
        forged.semantic_identity = program_with_unused_provider(7).semantic_identity().clone();
        assert_eq!(
            forged.for_target(governed_target),
            Err(RequestError::UnverifiedTargetSelection)
        );

        let mut forged = verified.clone();
        forged.normalized.serial_sum_mut().scale_bits = 3.0_f32.to_bits();
        assert_eq!(
            forged.for_target(governed_target),
            Err(RequestError::UnverifiedTargetSelection)
        );

        let mut forged = verified;
        forged.normalized.serial_sum_mut().output_key = OutputKey::new("forged").unwrap();
        assert_eq!(
            forged.for_target(governed_target),
            Err(RequestError::UnverifiedTargetSelection)
        );
    }

    #[test]
    fn verified_target_receipt_detects_every_governed_subject_mutation_class() {
        let program = program();
        let verified = verify_request(CompilationRequest::governed(&program)).unwrap();
        let target = verified.for_target(verified.target_profiles[0]).unwrap();

        let mut forged = target.clone();
        forged.target_profile.max_buffer_bindings_per_entry -= 1;
        assert!(!forged.is_authoritative());

        let mut forged = target.clone();
        forged.capabilities.materialized_serial_sum.revision += 1;
        assert!(!forged.is_authoritative());

        let mut forged = target.clone();
        forged.budgets.regions += 1;
        assert!(!forged.is_authoritative());

        let mut forged = target.clone();
        forged.semantic_identity = program_with_unused_provider(11).semantic_identity().clone();
        assert!(!forged.is_authoritative());

        let mut forged = target.clone();
        forged.normalized.serial_sum_mut().bias_bits ^= 1;
        assert!(!forged.is_authoritative());

        let mut forged = target;
        forged.normalized.serial_sum_mut().input_key = InputKey::new("forged").unwrap();
        assert!(!forged.is_authoritative());
    }

    #[test]
    fn used_provider_revision_changes_admission_and_snapshot_subjects() {
        let first = governed_test_program(1);
        let second = governed_test_program(2);
        let first = verify_request(CompilationRequest::governed(&first)).unwrap();
        let second = verify_request(CompilationRequest::governed(&second)).unwrap();

        assert_eq!(
            first.semantic_identity.graph(),
            second.semantic_identity.graph()
        );
        assert_eq!(
            first.semantic_identity.reached_definitions(),
            second.semantic_identity.reached_definitions()
        );
        assert_ne!(
            first.semantic_identity.admission_provenance(),
            second.semantic_identity.admission_provenance()
        );
        assert_ne!(
            first.semantic_identity.registry_snapshot(),
            second.semantic_identity.registry_snapshot()
        );
    }

    #[test]
    fn unused_provider_revision_changes_only_the_snapshot_subject() {
        let first = program_with_unused_provider(1);
        let second = program_with_unused_provider(2);
        let first = verify_request(CompilationRequest::governed(&first)).unwrap();
        let second = verify_request(CompilationRequest::governed(&second)).unwrap();

        assert_eq!(
            first.semantic_identity.graph(),
            second.semantic_identity.graph()
        );
        assert_eq!(
            first.semantic_identity.reached_definitions(),
            second.semantic_identity.reached_definitions()
        );
        assert_eq!(
            first.semantic_identity.admission_provenance(),
            second.semantic_identity.admission_provenance()
        );
        assert_ne!(
            first.semantic_identity.registry_snapshot(),
            second.semantic_identity.registry_snapshot()
        );
    }
}

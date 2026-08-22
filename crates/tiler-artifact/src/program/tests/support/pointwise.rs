//! The one-region pointwise fixture at both arithmetic widths.

use super::super::super::ScalarArithmeticSubject;
use super::super::super::{
    AbiRoot, ArtifactProgramBuilder, CompilationEnvironment, SelectedLoweringProvider,
    VerifiedArtifactProgram,
};
use super::artifacts::{declare_realization_at, formulas, lowering_subject, payload, variant};
use super::graphs::{
    BIAS_BITS, SCALE_BITS, checked_coverage_under, input_shape, strict, strict_contract,
};
use tiler_ir::index::NumericalContractIdentity;
use tiler_ir::kernel::{KernelType, VerifiedKernel, lower_scheduled_region};
use tiler_ir::program::{
    AlignmentGuarantee, AlignmentRequirement, AllocationOwnership, AllocationSpec,
    KernelProgramBuilder, MaterializedOrigin, MaterializedValueSpec, MemorySpace,
    RoutingCommitState, RoutingCommitTransition, StageAccess, StageAccessMode, StageLaunch,
    StorageEncoding, StorageScalar, ValueRole, VerifiedKernelProgram,
};
use tiler_ir::schedule::{
    Access, AccessMode, AccessOrdinal, ApproximationEnvelope, ArithmeticType,
    Bf16NumericalContractKey, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ExceptionalValueAssumption, ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess,
    MaterializationRounding, NumericalPermission, NumericalRealization, OwnershipProof,
    OwnershipProofKind, OwnershipWitnessId, PointwiseBf16ExpressionBuilder,
    PointwiseF32ExpressionBuilder, ReductionTopology, RegionId, RegionProgram, ScalarProgram,
    ScheduledRegionBuilder, SubnormalMode, TailPolicy, TensorRole,
};
use tiler_ir::semantic::{
    Bf16, Bf16Add, Bf16Constant, Bf16Multiply, F32, F32Add, F32Constant, F32Multiply, InputKey,
    OutputKey, ProviderIdentity, SemanticProgram, SemanticProgramBuilder,
};

// -------------------------------------------------------------------------
// The pointwise producer path at two widths
// -------------------------------------------------------------------------
//
// `admit-a-bf16-index-realization-law-and-refinement-contract` made a pure-BF16
// occurrence able to obtain executable coverage, so a BF16 kernel program now
// verifies. This crate owns what that unblocked and `tiler-ir` cannot reach: the
// packaging half, since the dependency direction is `tiler-artifact → tiler-ir`.
//
// The candidate index regions are still hand-built with `IndexRegionBuilder`
// through `checked_coverage_under`, exactly as every other fixture here does —
// `IndexRealizationLaw::realize` is `pub(crate)` to `tiler-ir` — which is the
// stronger arrangement anyway: a caller that could ask the law for its own
// answer and hand it straight back would turn the verifier into a rubber stamp.

/// `2.0` in the ratified BF16 operand format.
pub(crate) const BF16_SCALE_BITS: u16 = 0x4000;
/// `1.0` in the same format.
pub(crate) const BF16_BIAS_BITS: u16 = 0x3f80;

/// The two arithmetic widths the pointwise fixture is built at.
///
/// One parameterized construction rather than two hand-written fixture families,
/// because the identity comparison below is only worth making if the width is
/// the *sole* difference between the two artifacts. Two twins written out
/// separately drift, and a drifted pair still yields two distinct identities —
/// for a reason no later reader could attribute to the carrier.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PointwiseWidth {
    F32,
    Bf16,
}

impl PointwiseWidth {
    /// Storage width of one element, in bytes.
    pub(crate) const fn element_bytes(self) -> u64 {
        match self {
            Self::F32 => 4,
            Self::Bf16 => 2,
        }
    }

    pub(crate) const fn storage_scalar(self) -> StorageScalar {
        match self {
            Self::F32 => StorageScalar::F32,
            Self::Bf16 => StorageScalar::Bf16,
        }
    }

    pub(crate) const fn access_type(self) -> KernelType {
        match self {
            Self::F32 => KernelType::F32,
            Self::Bf16 => KernelType::Bf16,
        }
    }

    /// The scalar-arithmetic subject the delivered-realization record names.
    pub(crate) fn subject(self) -> ScalarArithmeticSubject {
        match self {
            Self::F32 => ScalarArithmeticSubject::f32(),
            Self::Bf16 => ScalarArithmeticSubject::new(ArithmeticType::Bf16, Bf16::resolved_type())
                .expect("the governed bf16 arithmetic subject is registered"),
        }
    }

    /// The governed strict contract for this width.
    ///
    /// The two keys are separate types rather than one key carrying a width,
    /// which is what makes a contract stated for the other width a *named*
    /// refusal at refinement rather than a silent mismatch.
    pub(crate) fn contract(self) -> NumericalContractIdentity {
        match self {
            Self::F32 => strict_contract(),
            Self::Bf16 => Bf16NumericalContractKey::new(
                SubnormalMode::Preserve,
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                ApproximationEnvelope::Forbidden,
                ExceptionalValueAssumption::MakeNoAssumption,
                ExceptionalValueAssumption::MakeNoAssumption,
                MaterializationRounding::NearestTiesToEven,
            )
            .expect("the fixture bf16 contract vector is coherent")
            .into(),
        }
    }

    /// The scheduled region's numerical realization for this width.
    ///
    /// The canonical NaN payload is this width's own: `verify_pointwise_bf16`
    /// refuses a `bf16` region declaring any other, so the two arms cannot be
    /// collapsed onto one constant.
    pub(crate) fn numerical(self) -> NumericalRealization {
        match self {
            Self::F32 => strict(),
            Self::Bf16 => NumericalRealization::new(
                "tiler.test.strict-bf16",
                u32::from(tiler_ir::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS),
                SubnormalMode::Preserve,
                SubnormalMode::Preserve,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                NumericalPermission::Forbidden,
                ApproximationEnvelope::Forbidden,
                ExceptionalValueAssumption::MakeNoAssumption,
                ExceptionalValueAssumption::MakeNoAssumption,
            ),
        }
    }

    /// The four-operation graph `result = input * 2.0 + 1.0` at this width.
    ///
    /// Constant, multiply, and add are the complete registered `bf16` semantic
    /// vocabulary, so the `Bf16` arm is the widest pure-`bf16` program the
    /// semantic layer can state rather than a subset chosen to be easy; the
    /// `F32` arm states the same four operations so the two are twins.
    pub(crate) fn semantic(self) -> SemanticProgram {
        let mut draft = SemanticProgramBuilder::try_standard().expect("standard registry");
        let key = InputKey::new("input").expect("input key");
        let result = OutputKey::new("result").expect("output key");
        match self {
            Self::F32 => {
                let input = draft.input::<F32>(key, input_shape()).expect("input");
                let scale = F32Constant::apply(&mut draft, SCALE_BITS).expect("scale");
                let bias = F32Constant::apply(&mut draft, BIAS_BITS).expect("bias");
                let product = F32Multiply::apply(&mut draft, input, scale).expect("product");
                let mapped = F32Add::apply(&mut draft, product, bias).expect("mapped");
                draft.output(result, mapped).expect("output");
            }
            Self::Bf16 => {
                let input = draft.input::<Bf16>(key, input_shape()).expect("input");
                let scale = Bf16Constant::apply(&mut draft, BF16_SCALE_BITS).expect("scale");
                let bias = Bf16Constant::apply(&mut draft, BF16_BIAS_BITS).expect("bias");
                let product = Bf16Multiply::apply(&mut draft, input, scale).expect("product");
                let mapped = Bf16Add::apply(&mut draft, product, bias).expect("mapped");
                draft.output(result, mapped).expect("output");
            }
        }
        let program = draft
            .build()
            .expect("a verified pointwise semantic program");
        assert_eq!(program.operation_count(), 4);
        program
    }

    /// The fused pointwise expression the scheduled region computes.
    pub(crate) fn scalar_program(self) -> ScalarProgram {
        match self {
            Self::F32 => {
                let mut expression = PointwiseF32ExpressionBuilder::new();
                let leaf = expression.input(AccessOrdinal::FIRST).expect("input");
                let scale = expression.constant(SCALE_BITS).expect("scale");
                let product = expression.multiply(leaf, scale).expect("product");
                let bias = expression.constant(BIAS_BITS).expect("bias");
                let root = expression.add(product, bias).expect("sum");
                ScalarProgram::PointwiseF32(
                    expression.build(root).expect("an f32 pointwise expression"),
                )
            }
            Self::Bf16 => {
                let mut expression = PointwiseBf16ExpressionBuilder::new();
                let leaf = expression.input(AccessOrdinal::FIRST).expect("input");
                let scale = expression.constant(BF16_SCALE_BITS).expect("scale");
                let product = expression.multiply(leaf, scale).expect("product");
                let bias = expression.constant(BF16_BIAS_BITS).expect("bias");
                let root = expression.add(product, bias).expect("sum");
                ScalarProgram::PointwiseBf16(
                    expression.build(root).expect("a bf16 pointwise expression"),
                )
            }
        }
    }

    /// The one-region kernel: a whole `[2, 3]` read to a whole `[2, 3]` write.
    pub(crate) fn kernel(self) -> VerifiedKernel {
        let count = elements();
        let owner = OwnershipWitnessId::new(0);
        let mut region = ScheduledRegionBuilder::new(RegionId::new(0));
        region
            .iteration_shape(input_shape())
            .expect("iteration shape");
        for (tensor, mode, bounds, ownership) in [
            (
                TensorRole::Input,
                AccessMode::Read,
                BoundsWitnessId::new(0),
                None,
            ),
            (
                TensorRole::Output,
                AccessMode::Write,
                BoundsWitnessId::new(1),
                Some(owner),
            ),
        ] {
            region
                .push_access(Access {
                    tensor,
                    component_role: None,
                    mode,
                    map: LogicalAccess::LinearIdentity,
                    bounds,
                    ownership,
                })
                .expect("a whole-tensor access");
            region
                .push_bounds_proof(BoundsProof {
                    id: bounds,
                    tensor,
                    component_role: None,
                    kind: BoundsProofKind::LinearRange {
                        element_count: count,
                    },
                })
                .expect("linear bounds");
        }
        region
            .ownership_proof(OwnershipProof {
                id: owner,
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: count,
                },
            })
            .expect("output ownership");
        region
            .program(RegionProgram::Numerical {
                scalar: self.scalar_program(),
                numerical: self.numerical(),
            })
            .expect("scalar program");
        region
            .schedule(KernelSchedule {
                binding: ExecutionBinding::GlobalLinearInvocation,
                work_items: count,
                threads_per_workgroup: 1,
                tail: TailPolicy::Exact,
                output_owner: owner,
                reduction: ReductionTopology::None,
                launch: LaunchPlan {
                    grid_threads: count,
                    threads_per_workgroup: 1,
                    zero_work_skips_dispatch: true,
                },
            })
            .expect("schedule");
        lower_scheduled_region(&region.build().expect("a verified pointwise region"))
            .expect("a verified pointwise kernel")
    }

    pub(crate) fn value(
        self,
        origin: MaterializedOrigin,
        role: ValueRole,
    ) -> MaterializedValueSpec {
        MaterializedValueSpec {
            origin,
            role,
            shape: input_shape(),
            storage_scalar: self.storage_scalar(),
            element_type: self.access_type(),
            encoding: StorageEncoding::Unpacked,
            alignment: AlignmentRequirement::natural_for(self.storage_scalar()),
            memory_space: MemorySpace::Device,
        }
    }

    /// The single-stage kernel program the artifact packages.
    pub(crate) fn program(self, semantic: &SemanticProgram) -> VerifiedKernelProgram {
        self.program_with_kernel(semantic, &self.kernel())
    }

    pub(crate) fn program_with_kernel(
        self,
        semantic: &SemanticProgram,
        kernel: &VerifiedKernel,
    ) -> VerifiedKernelProgram {
        let coverage = checked_coverage_under(semantic, &self.contract());
        let bytes = self.element_bytes() * elements();
        let mut plan = KernelProgramBuilder::new(semantic).expect("program builder");
        let external = plan
            .push_allocation(AllocationSpec {
                capacity_bytes: bytes,
                alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
                memory_space: MemorySpace::Device,
                ownership: AllocationOwnership::External,
            })
            .expect("input allocation");
        let owned = plan
            .push_allocation(AllocationSpec {
                capacity_bytes: bytes,
                alignment: AlignmentGuarantee::natural_for(StorageScalar::F32),
                memory_space: MemorySpace::Device,
                ownership: AllocationOwnership::Program,
            })
            .expect("output allocation");
        let source = plan
            .push_value(
                self.value(
                    MaterializedOrigin::ProgramInput {
                        key: InputKey::new("input").expect("input key"),
                    },
                    ValueRole::Input,
                ),
                external,
            )
            .expect("input value");
        let result = plan
            .push_value(
                self.value(MaterializedOrigin::Internal, ValueRole::Output),
                owned,
            )
            .expect("output value");
        let read = plan.push_whole_view(source).expect("input view");
        let write = plan.push_whole_view(result).expect("output view");
        // Only the quantities this one stage names. Minting the byte counts the
        // reduction fixtures share would leave an ABI expression no stage
        // references, which the program verifier refuses by name.
        let mut literal = |value| {
            plan.push_abi_root(AbiRoot::UnsignedLiteral(value))
                .expect("abi literal")
        };
        let value_bytes = literal(bytes);
        let grid_threads = literal(elements());
        let threads_per_workgroup = literal(1);
        let guard = plan
            .push_abi_root(AbiRoot::BooleanLiteral(true))
            .expect("guard predicate");
        plan.applicability_guard(guard)
            .expect("applicability guard");
        for (from, to, fallback_permitted) in [
            (
                RoutingCommitState::Preflight,
                RoutingCommitState::Committed,
                true,
            ),
            (
                RoutingCommitState::Committed,
                RoutingCommitState::Executing,
                false,
            ),
            (
                RoutingCommitState::Executing,
                RoutingCommitState::Published,
                false,
            ),
        ] {
            plan.push_routing_commit_transition(RoutingCommitTransition {
                from,
                to,
                fallback_permitted,
            })
            .expect("routing-commit transition");
        }
        plan.push_stage(
            kernel,
            &coverage,
            &[
                StageAccess {
                    view: read,
                    mode: StageAccessMode::Read,
                    accessible_bytes: value_bytes,
                },
                StageAccess {
                    view: write,
                    mode: StageAccessMode::Write,
                    accessible_bytes: value_bytes,
                },
            ],
            StageLaunch {
                grid_threads,
                threads_per_workgroup,
            },
        )
        .expect("the pointwise stage covers every occurrence of its bound graph");
        plan.push_output(OutputKey::new("result").expect("output key"), result)
            .expect("named output");
        plan.build().expect("a verified pointwise kernel program")
    }

    /// The one-variant artifact packaging that program.
    pub(crate) fn artifact(self) -> VerifiedArtifactProgram {
        let semantic = self.semantic();
        let program = self.program(&semantic);
        let provider =
            ProviderIdentity::new("tiler-test", "pointwise-scale-bias", 1).expect("provider");
        let environment = CompilationEnvironment::new([provider.clone()], []).expect("environment");
        let mut draft =
            ArtifactProgramBuilder::new(&semantic, environment).expect("artifact builder");
        draft
            .select_lowering_provider(SelectedLoweringProvider {
                provider,
                capability: lowering_subject("tiler", "pointwise-scale-bias", 1),
                capability_revision: 1,
            })
            .expect("selected provider");
        let descriptor = draft.push_payload(payload(0xb1)).expect("payload");
        let formulas = formulas(&mut draft);
        draft
            .push_variant(&program, variant(&formulas, descriptor, b"pointwise"))
            .expect("pointwise variant");
        declare_realization_at(&mut draft, &program, &self.subject(), 1);
        draft.build().expect("a verified pointwise artifact")
    }
}

/// Element count of the `[2, 3]` tensor both widths' fixtures address whole.
pub(crate) fn elements() -> u64 {
    input_shape()
        .extents()
        .iter()
        .map(|extent| extent.get())
        .product()
}

/// The pure-BF16 artifact reached through the ordinary producer path.
pub(crate) fn bf16_pointwise_artifact() -> VerifiedArtifactProgram {
    PointwiseWidth::Bf16.artifact()
}

/// Its F32 twin: the same four operations, the same shape, the other width.
pub(crate) fn f32_pointwise_artifact() -> VerifiedArtifactProgram {
    PointwiseWidth::F32.artifact()
}

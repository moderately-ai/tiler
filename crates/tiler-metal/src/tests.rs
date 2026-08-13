//! Golden, determinism, and fail-closed tests for Metal source emission.
//!
//! Golden fixtures pin the exact emitted bytes for the bounded proof profile's
//! kernels. On their own they are **not** compiler validation: a fixture can be
//! byte-identical to what emission produces and still be rejected by the Metal
//! compiler, so a passing golden test here proves only that emission is stable
//! and structured as intended. [`crate::golden_compilation`] closes that gap by
//! compiling every fixture through the offline `tiler-metal-aot` driver, and it
//! self-skips where no qualified Apple toolchain resolves.
//!
//! The numerical tests here therefore pin *structure*: that the NaN predicate
//! contains no floating-point operation, and that an obligation the target
//! cannot realize is recorded rather than mapped to a flag. What those
//! structures buy at run time is a device measurement, recorded in the
//! `prototype-metal-numerical-realization` ticket outcome with its exact
//! toolchain and commands.
//!
//! To regenerate a stale fixture, run the failing test and copy the actual
//! source from the assertion output; the assertion prints both sides in full.

use std::collections::BTreeSet;

use tiler_ir::kernel::{
    AddressSpace, BarrierOrdering, BarrierSpec, BinaryOp, BufferAccess, BufferParameter, Builtin,
    CompareOp, ExecutionScope, KernelBufferId, KernelBuilder, KernelConstant, KernelDiagnostic,
    KernelType, MemoryScope, VerifiedKernel, lower_scheduled_region,
};
use tiler_ir::schedule::{
    Access, AccessMode, ArithmeticType, AxisDecode, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContractionAxisSource, ContributorArrival, ContributorCoverage, ContributorOrder,
    ContributorPartition, ConvergenceEvidence, ExceptionalValueAssumption, ExecutionBinding,
    FlushedZeroSign, InputOrdinal, KernelSchedule, LaunchPlan, LogicalAccess, NumericalPermission,
    NumericalRealization, OwnershipProof, OwnershipProofKind, OwnershipWitnessId,
    PointwiseBf16Expression, PointwiseBf16ExpressionBuilder, PointwiseF32Expression,
    PointwiseF32ExpressionBuilder, ReductionTopology, RegionId, ScalarProgram,
    ScheduledRegionBuilder, SubnormalFreedom, SubnormalMode, SyncPointId, SynchronizationPlacement,
    SynchronizationPoint, TailPolicy, TensorRole, ValueDomainProvenance, VerifiedScheduledRegion,
    element_count, workgroup_tree_tile,
};
use tiler_ir::semantic::{CANONICAL_BF16_ARITHMETIC_NAN_BITS, RMS_NORM_F32_REFERENCE_EPS_BITS};
use tiler_ir::semantic::{
    STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE,
};
use tiler_ir::shape::{Axis, Shape};

use crate::diagnostic::{BarrierRejection, MetalEmitError};
use crate::emit::{
    address_space_declaration, barrier_call, bf16_canonical_nan, binary_realization,
    constant_minuend, emit_translation_unit as emit_with_realization, is_bf16_nan, is_f32_nan,
    msl_type, realization_requirements, reserve_symbol,
};
use crate::record::{MetalNumericalGap, MetalNumericalRequirement, MetalTranslationUnit};
use crate::target::{
    LaunchIndexRealization, MetalDeploymentMinimum, MetalEmissionRealization,
    MetalFloatArithmeticType, MetalFlushedZeroSign, MetalPlatform, MetalSubnormalArithmetic,
    MetalSubnormalArithmeticFacts, MetalTargetFacts, MslLanguageVersion,
};

const NAN_BITS: u32 = 0x7fc0_0000;
const SCALE_BITS: u32 = 0x4000_0000;
const BIAS_BITS: u32 = 0x3f80_0000;

/// The emitted binary32 canonicalization helper's symbol.
///
/// Written out rather than composed from the emitter's own prefix, so a change
/// to that prefix is a failing assertion here instead of a silently renamed
/// symbol every consumer of the emitted text has to rediscover.
const CANONICALIZE_F32_SYMBOL: &str = "tiler_canonicalize_nan_f32_7fc00000";

/// The emitted `bfloat16` canonicalization helper's symbol.
///
/// The Apple numerical probe harness's recognizer reads the C++-mangled form of
/// this exact identifier, `_ZL32tiler_canonicalize_nan_bf16_7fc0DF16b`, whose
/// `32` is this string's length and whose `DF16b` is its `bfloat` parameter.
const CANONICALIZE_BF16_SYMBOL: &str = "tiler_canonicalize_nan_bf16_7fc0";

fn scale_then_bias_expression(scale_bits: u32, bias_bits: u32) -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression
        .input(InputOrdinal::FIRST)
        .expect("pointwise input");
    let scale = expression.constant(scale_bits).expect("scale constant");
    let product = expression
        .multiply(input, scale)
        .expect("scale multiplication");
    let bias = expression.constant(bias_bits).expect("bias constant");
    let root = expression.add(product, bias).expect("bias addition");
    expression
        .build(root)
        .expect("verified pointwise expression")
}

/// The measured Apple profile: `f32` arithmetic flushes subnormals to the
/// sign-preserving zero and `f16` arithmetic preserves them.
fn target() -> MetalTargetFacts {
    MetalTargetFacts::new(
        MslLanguageVersion::Metal3_1,
        MetalPlatform::MacOs,
        MetalDeploymentMinimum::new(14, 0),
        subnormal_facts(APPLE_FLUSH),
        31,
    )
}

const fn emission() -> MetalEmissionRealization {
    MetalEmissionRealization::new(LaunchIndexRealization::ThreadPositionInGridUInt)
}

fn emit_translation_unit(
    kernels: &[&VerifiedKernel],
    target: &MetalTargetFacts,
) -> Result<MetalTranslationUnit, MetalEmitError> {
    emit_with_realization(kernels, target, emission())
}

/// The Apple row's per-type subnormal facts, with the `f32` entry varied.
///
/// The `f16` and `bf16` entries are the measured ones in every case: no test
/// here emits arithmetic at either width, so varying them would exercise
/// nothing, and stating them keeps the fixtures a faithful copy of the measured
/// macOS row rather than a target that happens to be silent about two of the
/// three dtypes. This is a `MacOs` profile, which is the only family `bf16` was
/// dispatched on.
const fn subnormal_facts(f32_behaviour: MetalSubnormalArithmetic) -> MetalSubnormalArithmeticFacts {
    MetalSubnormalArithmeticFacts::unmeasured()
        .stating(MetalFloatArithmeticType::F32, f32_behaviour)
        .stating(
            MetalFloatArithmeticType::F16,
            MetalSubnormalArithmetic::PreservesSubnormals,
        )
        .stating(
            MetalFloatArithmeticType::Bf16,
            MetalSubnormalArithmetic::FlushesToZero {
                zero_sign: MetalFlushedZeroSign::PreservesSign,
            },
        )
}

/// The strict declared realization every golden fixture is emitted under.
fn numerical(nan_bits: u32) -> NumericalRealization {
    subnormal_realization(
        "tiler.test.strict-f32",
        nan_bits,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
    )
}

/// A declared realization that varies only the two subnormal dimensions.
///
/// Contraction and reassociation stay forbidden, which is what the one governed
/// contract accepting a flush also does: accepting flushing widens exactly that
/// dimension and authorizes nothing else. Holding the other two fixed keeps
/// every assertion below about the subnormal comparison and nothing else.
fn subnormal_realization(
    profile_key: &'static str,
    nan_bits: u32,
    input_subnormals: SubnormalMode,
    result_subnormals: SubnormalMode,
) -> NumericalRealization {
    NumericalRealization::new(
        profile_key,
        nan_bits,
        input_subnormals,
        result_subnormals,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
    )
}

fn linear_schedule(work_items: u64) -> KernelSchedule {
    KernelSchedule {
        binding: ExecutionBinding::GlobalLinearInvocation,
        work_items,
        threads_per_workgroup: 1,
        tail: TailPolicy::Exact,
        output_owner: OwnershipWitnessId::new(0),
        reduction: ReductionTopology::None,
        launch: LaunchPlan {
            grid_threads: work_items,
            threads_per_workgroup: 1,
            zero_work_skips_dispatch: true,
        },
    }
}

/// A pointwise scale-then-bias region over `shape` under the strict realization.
fn pointwise_region(id: RegionId, shape: &Shape, nan_bits: u32) -> VerifiedScheduledRegion {
    pointwise_region_under(id, shape, numerical(nan_bits))
}

/// The pinned `tiler::silu-f32@1` expression, `x / (1 + Exp(-x))`.
///
/// The negation is spelled as a multiplication by `-1.0`, which is exact in
/// IEEE-754 for every operand including both zeros and both infinities, so it
/// introduces no rounding the reference does not have. There is deliberately no
/// negate node in the vocabulary to reach for and no reciprocal node to
/// substitute for the division.
fn silu_expression() -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression
        .input(InputOrdinal::FIRST)
        .expect("pointwise input");
    let negative_one = expression.constant(0xbf80_0000).expect("negative one");
    let negated = expression
        .multiply(input.clone(), negative_one)
        .expect("exact negation");
    let exponential = expression
        .exp(negated)
        .expect("the subordinate exponential");
    let one = expression.constant(0x3f80_0000).expect("one");
    let divisor = expression.add(one, exponential).expect("1 + exp(-x)");
    let root = expression
        .divide(input, divisor)
        .expect("x / (1 + exp(-x))");
    expression
        .build(root)
        .expect("verified SiLU pointwise expression")
}

/// The bounded `SiLU` fixture under one stated declared realization.
fn silu_kernel_under(realization: NumericalRealization) -> VerifiedKernel {
    lower_scheduled_region(&pointwise_region_with(
        RegionId::new(21),
        &Shape::from_dims([4]),
        realization,
        ScalarProgram::PointwiseF32(silu_expression()),
    ))
    .expect("the bounded SiLU fixture lowers")
}

/// A `SiLU` region under the strict declared realization.
pub(crate) fn silu_kernel() -> VerifiedKernel {
    silu_kernel_under(numerical(NAN_BITS))
}

/// The `tiler::rms-norm-f32@1` epilogue as a pointwise expression.
///
/// `y = w * (x * Rsqrt(u + eps))`, reading the normalized value at ordinal zero,
/// the already-broadcast weight at ordinal one, and the row's mean of squares at
/// ordinal two. It is the second of the normalization's two passes: the first is
/// the squaring-prologue reduction below, and the epilogue is separate because
/// the two iterate different domains.
///
/// The `eps` payload is the pinned workload's, and the reciprocal square root is
/// one node rather than a reciprocal of a square root — the vocabulary has no
/// `sqrt` node to compose, which is what makes the two-rounding spelling
/// unstatable here.
fn rms_norm_epilogue_expression() -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let value = expression
        .input(InputOrdinal::FIRST)
        .expect("the normalized value");
    let weight = expression
        .input(InputOrdinal::new(1))
        .expect("the broadcast weight");
    let mean = expression
        .input(InputOrdinal::new(2))
        .expect("the row mean of squares");
    let eps = expression
        .constant(RMS_NORM_F32_REFERENCE_EPS_BITS)
        .expect("the governed eps payload");
    let argument = expression.add(mean, eps).expect("u + eps");
    let scale = expression
        .rsqrt(argument)
        .expect("the subordinate reciprocal square root");
    let normalized = expression.multiply(value, scale).expect("x * r");
    let root = expression
        .multiply(weight, normalized)
        .expect("w * (x * r)");
    expression
        .build(root)
        .expect("verified normalization epilogue expression")
}

/// The normalization epilogue region, under the strict declared realization.
///
/// Three read accesses because the expression has three input leaves, which the
/// schedule verifier requires to correspond one for one with the region's reads.
pub(crate) fn rms_norm_epilogue_kernel() -> VerifiedKernel {
    let shape = Shape::from_dims([4]);
    let elements = element_count(&shape).expect("bounded fixture shape");
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(23));
    builder.iteration_shape(shape).unwrap();
    for ordinal in 0..3_u32 {
        builder
            .push_access(Access {
                tensor: TensorRole::Input {
                    ordinal: InputOrdinal::new(ordinal),
                },
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::LinearIdentity,
                bounds: BoundsWitnessId::new(ordinal),
                ownership: None,
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(ordinal),
                tensor: TensorRole::Input {
                    ordinal: InputOrdinal::new(ordinal),
                },
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: elements,
                },
            })
            .unwrap();
    }
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(3),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(3),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::PointwiseF32(rms_norm_epilogue_expression()))
        .unwrap();
    builder.numerical(numerical(NAN_BITS)).unwrap();
    builder.schedule(linear_schedule(elements)).unwrap();
    lower_scheduled_region(&builder.build().unwrap())
        .expect("the bounded normalization epilogue fixture lowers")
}

/// A pointwise scale-then-bias region carrying one stated declared realization.
fn pointwise_region_under(
    id: RegionId,
    shape: &Shape,
    realization: NumericalRealization,
) -> VerifiedScheduledRegion {
    pointwise_region_with(
        id,
        shape,
        realization,
        ScalarProgram::PointwiseF32(scale_then_bias_expression(SCALE_BITS, BIAS_BITS)),
    )
}

/// A one-input, one-output pointwise region carrying one stated scalar program.
fn pointwise_region_with(
    id: RegionId,
    shape: &Shape,
    realization: NumericalRealization,
    scalar: ScalarProgram,
) -> VerifiedScheduledRegion {
    let elements = element_count(shape).expect("bounded fixture shape");
    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(shape.clone()).unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    for (witness, tensor) in [
        (
            0,
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
        ),
        (1, TensorRole::Intermediate),
    ] {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: elements,
                },
            })
            .unwrap();
    }
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: elements,
            },
        })
        .unwrap();
    builder.scalar_program(scalar).unwrap();
    builder.numerical(realization).unwrap();
    builder.schedule(linear_schedule(elements)).unwrap();
    builder.build().unwrap()
}

// ---------------------------------------------------------------------------
// BF16 fixtures
// ---------------------------------------------------------------------------

/// `bf16` 2.0, the sibling of [`SCALE_BITS`] at the narrower width.
const BF16_SCALE_BITS: u16 = 0x4000;

/// `bf16` 1.0, the sibling of [`BIAS_BITS`].
const BF16_BIAS_BITS: u16 = 0x3f80;

/// `bf16`'s canonical arithmetic NaN, zero-extended into the 32-bit field.
///
/// Read from the semantic layer rather than written as a literal, so a fixture
/// cannot drift from the payload the region verifier requires a `bf16` region
/// to declare. The field is 32 bits wide and the payload is 16, and the
/// zero-extension is the producer's declaration rather than this backend's
/// reading of a wider value.
fn bf16_nan_bits() -> u32 {
    u32::from(CANONICAL_BF16_ARITHMETIC_NAN_BITS)
}

/// `(x * 2.0) + 1.0` in `bf16`, the direct sibling of
/// [`scale_then_bias_expression`].
///
/// The payloads are the same *values* as the `f32` fixture's and not the same
/// bits, which is the point: a lowering that reused the `f32` constants would
/// emit patterns no `bfloat` can hold.
fn bf16_scale_then_bias_expression() -> PointwiseBf16Expression {
    let mut expression = PointwiseBf16ExpressionBuilder::new();
    let input = expression
        .input(InputOrdinal::FIRST)
        .expect("pointwise input");
    let scale = expression
        .constant(BF16_SCALE_BITS)
        .expect("scale constant");
    let product = expression
        .multiply(input, scale)
        .expect("scale multiplication");
    let bias = expression.constant(BF16_BIAS_BITS).expect("bias constant");
    let root = expression.add(product, bias).expect("bias addition");
    expression
        .build(root)
        .expect("verified pointwise bf16 expression")
}

/// The strict `bf16` declared realization: preserve on both subnormal
/// dimensions.
fn bf16_numerical() -> NumericalRealization {
    subnormal_realization(
        "tiler.test.strict-bf16",
        bf16_nan_bits(),
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
    )
}

/// The bounded BF16 pointwise fixture under one stated declared realization.
fn bf16_pointwise_kernel_under(realization: NumericalRealization) -> VerifiedKernel {
    let region = pointwise_region_with(
        RegionId::new(30),
        &Shape::from_dims([4]),
        realization,
        ScalarProgram::PointwiseBf16(bf16_scale_then_bias_expression()),
    );
    lower_scheduled_region(&region).expect("the bounded bf16 pointwise fixture lowers")
}

/// The bounded BF16 pointwise fixture under the strict declared realization.
pub(crate) fn bf16_pointwise_kernel() -> VerifiedKernel {
    bf16_pointwise_kernel_under(bf16_numerical())
}

/// The Apple row's per-type facts with the `bf16` entry varied.
///
/// The counterpart of [`subnormal_facts`], varying the entry the BF16 tests are
/// about while stating the measured `f32` and `f16` rows beside it. Both
/// neighbours are stated deliberately: a target silent about them could not
/// distinguish "the `bf16` fact was read" from "no fact was found anywhere".
const fn bf16_subnormal_facts(
    bf16_behaviour: MetalSubnormalArithmetic,
) -> MetalSubnormalArithmeticFacts {
    MetalSubnormalArithmeticFacts::unmeasured()
        .stating(MetalFloatArithmeticType::F32, APPLE_FLUSH)
        .stating(
            MetalFloatArithmeticType::F16,
            MetalSubnormalArithmetic::PreservesSubnormals,
        )
        .stating(MetalFloatArithmeticType::Bf16, bf16_behaviour)
}

/// Emits the BF16 fixture under one declared realization and one `bf16` target
/// fact.
///
/// The sibling of [`emit_pointwise_under`], and it varies the same two things
/// at the other width. The `f32` fact stays the measured flush in every case,
/// so a result that changed with the `bf16` argument alone is evidence the
/// obligation was recorded against the right dtype.
fn emit_bf16_pointwise_under(
    realization: NumericalRealization,
    bf16_behaviour: MetalSubnormalArithmetic,
) -> MetalTranslationUnit {
    let kernel = bf16_pointwise_kernel_under(realization);
    let mut facts = target();
    facts.subnormal_arithmetic = bf16_subnormal_facts(bf16_behaviour);
    emit_translation_unit(&[&kernel], &facts).expect("the bounded bf16 fixture emits")
}

fn strict_affine_u4_dequantize_kernel() -> VerifiedKernel {
    let logical_elements = 5;
    let owner = OwnershipWitnessId::new(0);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(17));
    builder
        .iteration_shape(Shape::from_dims([logical_elements]))
        .unwrap();
    for access in [
        Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: Some(STRICT_AFFINE_CODES_ROLE),
            mode: AccessMode::Read,
            map: LogicalAccess::PackedU4LsbZeroTail { logical_elements },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        },
        Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: Some(STRICT_AFFINE_SCALE_ROLE),
            mode: AccessMode::Read,
            map: LogicalAccess::ScalarBroadcast,
            bounds: BoundsWitnessId::new(1),
            ownership: None,
        },
        Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: Some(STRICT_AFFINE_ZERO_POINT_ROLE),
            mode: AccessMode::Read,
            map: LogicalAccess::ScalarBroadcast,
            bounds: BoundsWitnessId::new(2),
            ownership: None,
        },
        Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(3),
            ownership: Some(owner),
        },
    ] {
        builder.push_access(access).unwrap();
    }
    for (id, tensor, component_role, element_count) in [
        (
            0,
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            Some(STRICT_AFFINE_CODES_ROLE),
            logical_elements.div_ceil(2),
        ),
        (
            1,
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            Some(STRICT_AFFINE_SCALE_ROLE),
            1,
        ),
        (
            2,
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            Some(STRICT_AFFINE_ZERO_POINT_ROLE),
            1,
        ),
        (3, TensorRole::Output, None, logical_elements),
    ] {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(id),
                tensor,
                component_role,
                kind: BoundsProofKind::LinearRange { element_count },
            })
            .unwrap();
    }
    builder
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: logical_elements,
            },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::StrictAffineU4Dequantize {
            codes_role: STRICT_AFFINE_CODES_ROLE,
            scale_role: STRICT_AFFINE_SCALE_ROLE,
            zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
        })
        .unwrap();
    builder.numerical(numerical(NAN_BITS)).unwrap();
    builder.schedule(linear_schedule(logical_elements)).unwrap();
    lower_scheduled_region(&builder.build().unwrap()).unwrap()
}

/// Which reduction a fixture region carries.
///
/// A named enum rather than a boolean, because there are now four cases and they
/// are not four settings of one shape. The two prologues differ in kind —
/// `scale * x + bias` is affine in the contributor and `x * x` is quadratic, so
/// neither expresses the other — and [`Self::Maximum`] is not a prologue at all
/// but a different *reducer*, which is why the enum is named for the reduction
/// rather than for the prologue it once only distinguished.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FixtureReduction {
    /// The bare strict serial sum; the contributor enters the fold unchanged.
    BareSum,
    /// The scale-then-bias prologue of the fused serial sum.
    ScaleBiasSum,
    /// The squaring prologue of `tiler::rms-norm-f32@1`'s embedded reduction.
    SquaredSum,
    /// The `Maximum` fold of `tiler::softmax-f32@1`'s first pass.
    ///
    /// A different combiner rather than a prologue, and the only fixture whose
    /// scalar program carries no empty-domain identity.
    Maximum,
}

/// A serial reduction region over `axes` of `input`, optionally fusing a
/// per-contributor prologue into every contributor.
fn reduction_region(
    id: RegionId,
    input: &Shape,
    axes: &[Axis],
    reduction: FixtureReduction,
) -> VerifiedScheduledRegion {
    let output = input.without_axes(axes);
    let output_elements = element_count(&output).expect("bounded fixture shape");
    // A prologue reads the original input; a bare fold reads an intermediate.
    // The extrema fold reads the original input too, because the softmax's
    // maximum pass reads the scores rather than a materialized intermediate.
    let read_tensor = match reduction {
        FixtureReduction::BareSum => TensorRole::Intermediate,
        FixtureReduction::ScaleBiasSum
        | FixtureReduction::SquaredSum
        | FixtureReduction::Maximum => TensorRole::Input {
            ordinal: InputOrdinal::FIRST,
        },
    };
    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(output.clone()).unwrap();
    builder
        .push_access(Access {
            tensor: read_tensor,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input.clone(),
                output_shape: output.clone(),
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: read_tensor,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input.clone(),
                output_shape: output.clone(),
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: output_elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: output_elements,
            },
        })
        .unwrap();
    let scalar = match reduction {
        FixtureReduction::ScaleBiasSum => ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits: SCALE_BITS,
            bias_bits: BIAS_BITS,
            axes: axes.to_vec(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: NAN_BITS,
            empty_identity_bits: 0.0_f32.to_bits(),
            contraction: false,
        },
        FixtureReduction::SquaredSum => ScalarProgram::SquaredSerialSum {
            axes: axes.to_vec(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: NAN_BITS,
            empty_identity_bits: 0.0_f32.to_bits(),
        },
        // No `empty_identity_bits`, because the extrema family has none — the
        // field does not exist on this variant rather than being defaulted here.
        FixtureReduction::Maximum => ScalarProgram::StrictSerialMaximum {
            axes: axes.to_vec(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: NAN_BITS,
        },
        FixtureReduction::BareSum => ScalarProgram::StrictSerialSum {
            axes: axes.to_vec(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: NAN_BITS,
            empty_identity_bits: 0.0_f32.to_bits(),
        },
    };
    builder.scalar_program(scalar).unwrap();
    builder.numerical(numerical(NAN_BITS)).unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(output_elements)
        })
        .unwrap();
    builder.build().unwrap()
}

/// The `direct` contraction of the profile's own index structure, `td,od->to`.
///
/// `[m, k] x [n, k] -> [m, n]`, one invocation per output element, each folding
/// its own contracted sequence in ascending `d`. Operand 0 reads output position
/// 0 then the contracted coordinate; operand 1 reads output position 1 then the
/// same contracted coordinate — which is exactly what makes the weight layout
/// `[out_features, in_features]` a *different* structure from the ordinary
/// `[K, N]` matmul.
fn contraction_region(id: RegionId, m: u64, n: u64, k: u64) -> VerifiedScheduledRegion {
    let left = Shape::from_dims([m, k]);
    let right = Shape::from_dims([n, k]);
    let output = Shape::from_dims([m, n]);
    let contracted = Shape::from_dims([k]);
    let output_elements = element_count(&output).unwrap();
    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(output.clone()).unwrap();
    for (ordinal, (operand, free)) in [(&left, 0_u32), (&right, 1)].into_iter().enumerate() {
        let witness = u32::try_from(ordinal).unwrap();
        let tensor = TensorRole::Input {
            ordinal: InputOrdinal::new(witness),
        };
        builder
            .push_access(Access {
                tensor,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::ContractionOperand {
                    operand_shape: operand.clone(),
                    output_shape: output.clone(),
                    contracted_shape: contracted.clone(),
                    sources: vec![
                        ContractionAxisSource::Output { position: free },
                        ContractionAxisSource::Contracted { position: 0 },
                    ],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
                bounds: BoundsWitnessId::new(witness),
                ownership: None,
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: element_count(operand).unwrap(),
                },
            })
            .unwrap();
    }
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(2),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: output_elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: output_elements,
            },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::StrictTensorContraction {
            contracted_shape: contracted.clone(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: NAN_BITS,
        })
        .unwrap();
    builder.numerical(numerical(NAN_BITS)).unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::Contraction {
                contracted_shape: contracted,
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(output_elements)
        })
        .unwrap();
    builder.build().unwrap()
}

pub(crate) fn contraction_kernel() -> VerifiedKernel {
    lower_scheduled_region(&contraction_region(RegionId::new(9), 2, 3, 4))
        .expect("bounded contraction fixture lowers")
}

/// The single-workgroup tree realization of a `[2, 6] -> [2]` strict sum.
///
/// Three participants per workgroup, each folding two contributors into its own
/// staging slot, all three reading the staged set back, one committing. The tile
/// comes from [`workgroup_tree_tile`] rather than being spelled out here, so this
/// fixture cannot drift from the canonical dataflow the schedule verifier and
/// the structured-kernel lowering are both written against.
///
/// This is the *only* fixture in this crate whose kernel names a local
/// invocation coordinate, declares workgroup storage, stages values, and carries
/// a barrier — so it is what makes the staged and local-index emission paths
/// reachable at all.
fn cooperative_region(id: RegionId) -> VerifiedScheduledRegion {
    const PARTICIPANTS: u64 = 3;
    const CONTRIBUTORS_PER_PARTITION: u64 = 2;
    let input = Shape::from_dims([2, 6]);
    let output = Shape::from_dims([2]);
    let axes = vec![Axis::new(1)];
    let output_elements = element_count(&output).unwrap();
    let work_items = output_elements * PARTICIPANTS;

    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(Shape::from_dims([2, 3])).unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: input.clone(),
                output_shape: output.clone(),
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: input,
                output_shape: output,
                axes: axes.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: output_elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: output_elements,
            },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::StrictSerialSum {
            axes: axes.clone(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: NAN_BITS,
            empty_identity_bits: 0.0_f32.to_bits(),
        })
        .unwrap();
    // A cooperative split regroups the declared contributor sequence, so the
    // contract has to permit reassociation or the schedule verifier refuses the
    // topology outright. Every other dimension stays strict.
    builder
        .numerical(NumericalRealization {
            reassociation: NumericalPermission::Permitted,
            ..numerical(NAN_BITS)
        })
        .unwrap();
    builder
        .schedule(KernelSchedule {
            threads_per_workgroup: u32::try_from(PARTICIPANTS).unwrap(),
            reduction: ReductionTopology::CooperativeWorkgroup {
                coverage: ContributorCoverage::Exact(ContributorPartition {
                    partitions: PARTICIPANTS,
                    contributors_per_partition: CONTRIBUTORS_PER_PARTITION,
                }),
                tile: workgroup_tree_tile(PARTICIPANTS).expect("the canonical tree tile"),
                axes,
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: false,
                arrival: ContributorArrival::AscendingParticipant,
            },
            launch: LaunchPlan {
                grid_threads: work_items,
                threads_per_workgroup: u32::try_from(PARTICIPANTS).unwrap(),
                zero_work_skips_dispatch: true,
            },
            ..linear_schedule(work_items)
        })
        .unwrap();
    builder.build().unwrap()
}

pub(crate) fn cooperative_kernel() -> VerifiedKernel {
    lower_scheduled_region(&cooperative_region(RegionId::new(10)))
        .expect("bounded cooperative fixture lowers")
}

/// The same tile with its phases run twice and its slots rewritten.
///
/// Built by re-verifying the single-round fixture's own region rather than by a
/// second literal, so the only differences are the ones the capability requires:
/// each participant now folds one contributor per round instead of two, both
/// points name the round-loop convergence derivation, and a round boundary
/// discharges the rewrite.
fn loop_carried_cooperative_region(id: RegionId) -> VerifiedScheduledRegion {
    let mut region = cooperative_region(id).region().clone();
    let ReductionTopology::CooperativeWorkgroup { coverage, tile, .. } =
        &mut region.schedule.reduction
    else {
        panic!("the cooperative fixture builds a cooperative topology")
    };
    let ContributorCoverage::Exact(partition) = coverage else {
        panic!("the fixture is exact coverage")
    };
    partition.contributors_per_partition = 1;
    tile.rounds = 2;
    let phase = tile.synchronization[0];
    tile.synchronization[0].convergence = ConvergenceEvidence::required_for_rounds(2);
    tile.synchronization.push(SynchronizationPoint {
        id: SyncPointId::new(1),
        placement: SynchronizationPlacement::RoundBoundary,
        convergence: ConvergenceEvidence::required_for_rounds(2),
        ..phase
    });
    ScheduledRegionBuilder::from_region(region)
        .build()
        .expect("the loop-carried region verifies")
}

fn loop_carried_cooperative_kernel() -> VerifiedKernel {
    lower_scheduled_region(&loop_carried_cooperative_region(RegionId::new(11)))
        .expect("the loop-carried fixture lowers")
}

// ---------------------------------------------------------------------------
// Structural fixtures
// ---------------------------------------------------------------------------

/// The identity pointwise expression: the value written is the value read.
///
/// A structural family computes nothing, so the region carries a scalar program
/// whose root is the input leaf itself. That is what makes the emitted body the
/// *access relation* and nothing else — a fixture with arithmetic in it would
/// mix the offset statements this section is about with statements that are
/// already covered elsewhere.
fn identity_expression() -> PointwiseF32Expression {
    let mut expression = PointwiseF32ExpressionBuilder::new();
    let input = expression
        .input(InputOrdinal::FIRST)
        .expect("pointwise input");
    expression
        .build(input)
        .expect("verified identity pointwise expression")
}

/// A structural region reading one operand through `map` and writing densely.
///
/// The two structural relations differ only in that argument. Both state the
/// region's own iteration shape as their result shape, which the region verifier
/// requires — the decodes are divisors of *this* region's linear coordinate — and
/// both prove the same domain: the contiguous element range of the operand they
/// read, which is why a single `LinearRange` witness serves either.
fn structural_region(
    id: RegionId,
    iteration_shape: &Shape,
    operand_shape: &Shape,
    map: LogicalAccess,
) -> VerifiedScheduledRegion {
    let result_elements = element_count(iteration_shape).expect("bounded fixture shape");
    let operand_elements = element_count(operand_shape).expect("bounded fixture operand shape");
    let mut builder = ScheduledRegionBuilder::new(id);
    builder.iteration_shape(iteration_shape.clone()).unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            mode: AccessMode::Read,
            map,
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: operand_elements,
            },
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: result_elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: result_elements,
            },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::PointwiseF32(identity_expression()))
        .unwrap();
    builder.numerical(numerical(NAN_BITS)).unwrap();
    builder.schedule(linear_schedule(result_elements)).unwrap();
    builder.build().unwrap()
}

/// `out = reverse(a)` on a `[2, 2]` operand, reversing axis 1.
///
/// **The one fixture whose body emits `BinaryOp::IndexSubtract`**, which is the
/// construct the mirror needs and the only one in the vocabulary whose contract
/// an unsigned realization can violate silently. The program is the compiler's
/// own `a_reindex_reaches_a_kernel_matching_the_reference_evaluator` fixture —
/// same shape, same axis, same admitted form — built by hand here because
/// `tiler-metal` does not and must not depend on `tiler-compiler`, so the region
/// is restated rather than imported.
///
/// The two decodes are the transposition-style windows a reversal keeps: axis 0
/// takes the leading window of the linear result coordinate (`divisor` 2,
/// `modulus` 2) and axis 1 takes the trailing one (`divisor` 1) with the mirror
/// set. Sorted by descending divisor they telescope `2 * 2 == 4` and `1 * 2 == 2`,
/// which is what `reindex_decodes_are_bijective` checks, and mirroring is
/// irrelevant to that check because `c -> extent - 1 - c` is a bijection of any
/// axis onto itself.
fn mirrored_reindex_region() -> VerifiedScheduledRegion {
    let shape = Shape::from_dims([2, 2]);
    structural_region(
        RegionId::new(40),
        &shape,
        &shape,
        LogicalAccess::ReindexBijection {
            operand_shape: shape.clone(),
            result_shape: shape.clone(),
            axes: vec![
                AxisDecode::read(2, 2),
                AxisDecode {
                    divisor: 1,
                    modulus: 2,
                    mirrored: true,
                },
            ],
        },
    )
}

pub(crate) fn mirrored_reindex_kernel() -> VerifiedKernel {
    lower_scheduled_region(&mirrored_reindex_region())
        .expect("the bounded mirrored reindex fixture lowers")
}

/// The mirrored fixture's own signature, declared through the public builder.
fn structural_signature(
    builder: &mut KernelBuilder,
    region: &VerifiedScheduledRegion,
) -> (KernelBufferId, KernelBufferId) {
    let read = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Read,
            element_count: 4,
        })
        .unwrap();
    let write = builder
        .declare_buffer(BufferParameter {
            tensor: TensorRole::Output,
            component_role: None,
            element_type: KernelType::F32,
            address_space: AddressSpace::Device,
            access: BufferAccess::Write,
            element_count: 4,
        })
        .unwrap();
    builder
        .admit_builtin(Builtin::GlobalInvocationIndex)
        .unwrap();
    builder.numerical(numerical(NAN_BITS)).unwrap();
    builder.requirements(region.requirements()).unwrap();
    (read, write)
}

/// The mirrored fixture's body with its subtraction's operands exchanged.
///
/// **The one perturbation the offline compiler cannot catch.** `c - (extent - 1)`
/// is well-formed MSL that translates, links, and — on every coordinate below the
/// mirror point — computes `c - 1` modulo `2^64`, which is an index near `2^64`
/// scaled into a buffer subscript. No `metal` diagnostic can see that, because
/// there is nothing ill-formed about it.
///
/// Everything else is the canonical lowering's body, statement for statement, so
/// a refusal names the exchange and not some other divergence. This returns the
/// builder rather than a kernel because `build()` **refuses it**, which is the
/// fact `the_verifier_refuses_a_reordered_mirror_before_emission_sees_it`
/// records.
fn reordered_mirror_builder() -> KernelBuilder {
    let region = mirrored_reindex_region();
    let mut builder = KernelBuilder::new(&region).unwrap();
    let (read, write) = structural_signature(&mut builder, &region);
    let invocation = builder.builtin(Builtin::GlobalInvocationIndex).unwrap();
    let elements = builder.constant(KernelConstant::Index(4)).unwrap();
    let active = builder
        .compare(CompareOp::IndexLessThan, invocation, elements)
        .unwrap();
    builder
        .predicated(active, |builder| {
            let divisor = builder.constant(KernelConstant::Index(2))?;
            let quotient = builder.binary(BinaryOp::IndexDivide, invocation, divisor)?;
            let stride = builder.constant(KernelConstant::Index(2))?;
            let leading = builder.binary(BinaryOp::IndexMultiply, quotient, stride)?;
            let modulus = builder.constant(KernelConstant::Index(2))?;
            let coordinate = builder.binary(BinaryOp::IndexModulo, invocation, modulus)?;
            let last = builder.constant(KernelConstant::Index(1))?;
            // The exchange, and the only difference from the canonical body:
            // the lowering emits `last - coordinate`.
            let mirrored = builder.binary(BinaryOp::IndexSubtract, coordinate, last)?;
            let offset = builder.binary(BinaryOp::IndexAdd, leading, mirrored)?;
            let loaded = builder.load(read, offset, BoundsWitnessId::new(0))?;
            builder.store(
                write,
                invocation,
                loaded,
                BoundsWitnessId::new(1),
                OwnershipWitnessId::new(0),
            )
        })
        .unwrap();
    builder
}

/// `out = broadcast(w)` widening a `[2]` operand across a `[2, 2]` result.
///
/// The other structural relation, and it pins a different property: the read and
/// the write declare *different element counts* on one entry point, which is the
/// signature shape a widening broadcast needs and which no fixture but the
/// contraction had. Its decode names result axis 1, so result axis 0 is the
/// replicated one the read is invariant in — visible in the emitted body as an
/// offset that wraps the launch coordinate and never divides it.
pub(crate) fn widening_broadcast_kernel() -> VerifiedKernel {
    let operand = Shape::from_dims([2]);
    let result = Shape::from_dims([2, 2]);
    lower_scheduled_region(&structural_region(
        RegionId::new(41),
        &result,
        &operand,
        LogicalAccess::BroadcastReplication {
            operand_shape: operand.clone(),
            result_shape: result.clone(),
            axes: vec![AxisDecode::read(1, 2)],
        },
    ))
    .expect("the bounded widening broadcast fixture lowers")
}

pub(crate) fn pointwise_kernel() -> VerifiedKernel {
    lower_scheduled_region(&pointwise_region(
        RegionId::new(0),
        &Shape::from_dims([4]),
        NAN_BITS,
    ))
    .expect("bounded pointwise fixture lowers")
}

pub(crate) fn single_axis_reduction_kernel() -> VerifiedKernel {
    lower_scheduled_region(&reduction_region(
        RegionId::new(1),
        &Shape::from_dims([2, 3]),
        &[Axis::new(1)],
        FixtureReduction::BareSum,
    ))
    .expect("bounded reduction fixture lowers")
}

pub(crate) fn multi_axis_reduction_kernel() -> VerifiedKernel {
    lower_scheduled_region(&reduction_region(
        RegionId::new(2),
        &Shape::from_dims([2, 3, 4]),
        &[Axis::new(1), Axis::new(2)],
        FixtureReduction::BareSum,
    ))
    .expect("bounded multi-axis reduction fixture lowers")
}

pub(crate) fn fused_reduction_kernel() -> VerifiedKernel {
    lower_scheduled_region(&reduction_region(
        RegionId::new(3),
        &Shape::from_dims([2, 3]),
        &[Axis::new(1)],
        FixtureReduction::ScaleBiasSum,
    ))
    .expect("bounded fused reduction fixture lowers")
}

/// The squaring-prologue serial sum `tiler::rms-norm-f32@1` embeds.
pub(crate) fn squared_reduction_kernel() -> VerifiedKernel {
    lower_scheduled_region(&reduction_region(
        RegionId::new(22),
        &Shape::from_dims([2, 3]),
        &[Axis::new(1)],
        FixtureReduction::SquaredSum,
    ))
    .expect("bounded squaring-prologue reduction fixture lowers")
}

/// The `Maximum` fold `tiler::softmax-f32@1`'s first pass embeds.
pub(crate) fn maximum_reduction_kernel() -> VerifiedKernel {
    lower_scheduled_region(&reduction_region(
        RegionId::new(24),
        &Shape::from_dims([2, 3]),
        &[Axis::new(1)],
        FixtureReduction::Maximum,
    ))
    .expect("bounded maximum reduction fixture lowers")
}

/// A reduction whose reduced axis has extent zero.
///
/// Its body writes the reduction's identity element for every output element
/// and reads its input buffer never, which is what made it unemittable while
/// the argument table was derived from body use.
pub(crate) fn empty_domain_reduction_kernel() -> VerifiedKernel {
    lower_scheduled_region(&reduction_region(
        RegionId::new(4),
        &Shape::from_dims([2, 0]),
        &[Axis::new(1)],
        FixtureReduction::BareSum,
    ))
    .expect("bounded empty-domain reduction fixture lowers")
}

fn emit_one(kernel: &VerifiedKernel) -> String {
    emit_translation_unit(&[kernel], &target())
        .expect("bounded fixture emits")
        .source()
        .to_owned()
}

/// Emits the pointwise fixture under one declared realization and one target
/// fact.
///
/// The pointwise fixture is used rather than a materialization-only kernel
/// because the subnormal comparison is only reached from emitted `f32`
/// arithmetic. A kernel with no arithmetic conforms vacuously, so a "no gap"
/// result over one would be evidence of nothing.
fn emit_pointwise_under(
    realization: NumericalRealization,
    subnormal_arithmetic: MetalSubnormalArithmetic,
) -> MetalTranslationUnit {
    let region = pointwise_region_under(RegionId::new(0), &Shape::from_dims([4]), realization);
    let kernel = lower_scheduled_region(&region).expect("bounded pointwise fixture lowers");
    let mut facts = target();
    facts.subnormal_arithmetic = subnormal_facts(subnormal_arithmetic);
    emit_translation_unit(&[&kernel], &facts).expect("bounded fixture emits")
}

/// The two flush-to-zero declarations, one per zero the vocabulary names.
const fn flush(zero_sign: FlushedZeroSign) -> SubnormalMode {
    SubnormalMode::FlushToZero { zero_sign }
}

/// The measured Apple target fact: a sign-preserving flush.
const APPLE_FLUSH: MetalSubnormalArithmetic = MetalSubnormalArithmetic::FlushesToZero {
    zero_sign: MetalFlushedZeroSign::PreservesSign,
};

/// Compares emitted source against a checked-in fixture.
///
/// Golden agreement pins determinism and structure. It is not evidence that the
/// Metal compiler accepts the source.
fn assert_golden(name: &str, expected: &str, actual: &str) {
    assert_eq!(
        actual, expected,
        "golden fixture crates/tiler-metal/goldens/{name} is stale"
    );
}

#[test]
fn pointwise_matches_its_golden_source() {
    assert_golden(
        "pointwise_scale_bias.metal",
        include_str!("../goldens/pointwise_scale_bias.metal"),
        &emit_one(&pointwise_kernel()),
    );
}

#[test]
fn single_axis_reduction_matches_its_golden_source() {
    assert_golden(
        "reduction_single_axis.metal",
        include_str!("../goldens/reduction_single_axis.metal"),
        &emit_one(&single_axis_reduction_kernel()),
    );
}

#[test]
fn multi_axis_reduction_matches_its_golden_source() {
    assert_golden(
        "reduction_multi_axis.metal",
        include_str!("../goldens/reduction_multi_axis.metal"),
        &emit_one(&multi_axis_reduction_kernel()),
    );
}

#[test]
fn fused_reduction_matches_its_golden_source() {
    assert_golden(
        "reduction_fused_multiply_add.metal",
        include_str!("../goldens/reduction_fused_multiply_add.metal"),
        &emit_one(&fused_reduction_kernel()),
    );
}

#[test]
fn contraction_matches_its_golden_source() {
    assert_golden(
        "contraction_strict_tensor.metal",
        include_str!("../goldens/contraction_strict_tensor.metal"),
        &emit_one(&contraction_kernel()),
    );
}

#[test]
fn cooperative_workgroup_reduction_matches_its_golden_source() {
    assert_golden(
        "cooperative_workgroup_reduction.metal",
        include_str!("../goldens/cooperative_workgroup_reduction.metal"),
        &emit_one(&cooperative_kernel()),
    );
}

#[test]
fn mirrored_reindex_matches_its_golden_source() {
    assert_golden(
        "structural_mirrored_reindex.metal",
        include_str!("../goldens/structural_mirrored_reindex.metal"),
        &emit_one(&mirrored_reindex_kernel()),
    );
}

#[test]
fn widening_broadcast_matches_its_golden_source() {
    assert_golden(
        "structural_widening_broadcast.metal",
        include_str!("../goldens/structural_widening_broadcast.metal"),
        &emit_one(&widening_broadcast_kernel()),
    );
}

/// The only fixture carrying an elementary function call and a division.
///
/// It exists to be compiled. Every assertion this crate makes about
/// `precise::exp` and the division operator is a string match over emitted text,
/// and a namespace-qualified call to a header-declared overload is exactly the
/// class of spelling that can match a string and still not translate — which is
/// what checking the fixture in buys, since
/// [`crate::golden_compilation`] compiles the whole `goldens/` directory.
#[test]
fn silu_matches_its_golden_source() {
    assert_golden(
        "elementary_silu_activation.metal",
        include_str!("../goldens/elementary_silu_activation.metal"),
        &emit_one(&silu_kernel()),
    );
}

/// The mirrored offset emits an unsigned difference that says what carries it.
///
/// Asserted on the exact statement rather than on the golden as a whole, because
/// the golden would stay green if the annotation moved to a line where it means
/// nothing. Two halves matter and they are separate claims: the difference is
/// emitted *exactly* — no clamp, no widening, no saturating call, any of which
/// would keep the index in range by reading a different element — and the text
/// attributes the non-negativity to the IR's proof rather than implying this
/// backend tested it.
#[test]
fn the_mirrored_offset_emits_an_exact_difference_naming_its_proof() {
    let source = emit_one(&mirrored_reindex_kernel());
    assert!(
        source.contains(
            "uint64_t v10 = v9 - v8;  // unsigned; v8 <= v9 by the IR's proof, not by a test\n"
        ),
        "the mirror must emit the plain difference with its provenance: {source}"
    );
    // The bound the annotation names is established by the statement above it,
    // and that statement is the emitted wrap rather than a comment about one.
    assert!(source.contains("uint64_t v8 = v0 % v7;\n"));
    // No clamping or saturating spelling reached the text. `min` would keep a
    // violated proof in range and silently address the wrong element, which is
    // the failure the exact difference refuses to hide.
    assert!(!source.contains("min("));
    assert!(!source.contains("clamp("));
}

/// The exchanged mirror never reaches emission: `tiler-ir` refuses it first.
///
/// **This is the wrap perturbation, and where it is actually caught.**
/// `c - (extent - 1)` is well-formed MSL — the offline compiler accepts it and
/// links it, and it computes an index near `2^64` for every coordinate below the
/// mirror point — so no compile-stage test can be the defence. The defence is
/// that the structured kernel verifier re-derives the offset expression from the
/// region's access relation and answers `BodyRefinement`, which this pins
/// against a builder that differs from the canonical lowering in exactly the two
/// exchanged operands.
///
/// Stated here rather than left implicit because it is what bounds
/// [`constant_minuend`]'s claim: that check is defence in depth against a
/// producer building a kernel some other way, not the thing standing between the
/// mirror and a wrapped index.
#[test]
fn the_verifier_refuses_a_reordered_mirror_before_emission_sees_it() {
    let diagnostics = reordered_mirror_builder()
        .build()
        .expect_err("an exchanged mirror is not a refinement of its region")
        .into_parts()
        .1;
    assert!(
        diagnostics.contains(&KernelDiagnostic::BodyRefinement),
        "the exchange must be refused as a refinement failure: {diagnostics:?}"
    );
    // The canonical body of the same region verifies and emits, so the refusal
    // above is about the exchanged operands and not about the fixture.
    assert!(emit_translation_unit(&[&mirrored_reindex_kernel()], &target()).is_ok());
}

/// A computed minuend is refused rather than emitted as an unsigned difference.
///
/// [`constant_minuend`] is exercised directly, for the reason
/// [`bf16_canonical_nan`]'s refusals are: the verifier above makes it
/// unreachable through `lower_scheduled_region`, and a rule with no test is a
/// rule that silently stops holding. Both refusing shapes are covered — no
/// constant at all, and a constant of the wrong role — because reading the
/// second as a bound would be reading an `f32` bit pattern as an index.
#[test]
fn an_index_subtraction_from_a_computed_minuend_is_refused() {
    assert_eq!(constant_minuend(Some(KernelConstant::Index(1))), Ok(1));
    for rejected in [None, Some(KernelConstant::F32Bits(BIAS_BITS))] {
        assert_eq!(
            constant_minuend(rejected),
            Err(MetalEmitError::MalformedKernel {
                rule: "non-constant-minuend",
            }),
        );
    }
}

/// Every construct the binary vocabulary names has an emitted Metal realization.
///
/// **The declared array length is the check, and it is the only mechanism this
/// crate has for it.** [`BinaryOp`] is `#[non_exhaustive]`, so
/// [`binary_realization`]'s match must carry a wildcard and `rustc` will never
/// close it here; a construct appended in `tiler-ir` therefore reaches the
/// backend as a run-time refusal that no test exercises, which is exactly how
/// `IndexSubtract` arrived unemittable and stayed that way. Declaring this array
/// at `variant_count` makes the same append an array-length build error in this
/// crate — the mechanism `applicability::MetalGpuFamily::ALL` already uses, and
/// the reason `#![feature(variant_count)]` is enabled at the crate root.
///
/// The distinctness assertion is what stops the length check from being
/// satisfied by a repeated construct, which would make the array long enough and
/// leave the new one untested.
#[test]
fn every_binary_construct_has_a_metal_realization() {
    const OPS: [BinaryOp; core::mem::variant_count::<BinaryOp>()] = [
        BinaryOp::IndexAdd,
        BinaryOp::IndexMultiply,
        BinaryOp::IndexDivide,
        BinaryOp::IndexModulo,
        BinaryOp::IndexSubtract,
        BinaryOp::I32Subtract,
        BinaryOp::F32Add,
        BinaryOp::F32Multiply,
        BinaryOp::F32Divide,
        BinaryOp::F32Maximum,
        BinaryOp::Bf16Add,
        BinaryOp::Bf16Multiply,
    ];
    let distinct: BTreeSet<BinaryOp> = OPS.into_iter().collect();
    assert_eq!(
        distinct.len(),
        OPS.len(),
        "a repeated construct would satisfy the length check and test nothing"
    );
    for op in OPS {
        assert!(
            binary_realization(op).is_ok(),
            "{op:?} reaches the backend with no Metal realization"
        );
    }
}

/// The cooperative kernel emits every construct its tile requires, and no other.
///
/// Stated over the emitted text rather than left to the golden alone: a golden
/// proves the bytes did not change, and this proves *which* bytes have to be
/// there. If threadgroup storage, the local coordinate, the staged handoff, or
/// the fence were dropped, the golden would simply be rebaselined by whoever
/// broke it, and nothing would say the result was still a cooperative kernel.
#[test]
fn the_cooperative_kernel_emits_storage_a_local_index_a_handoff_and_a_fence() {
    let source = emit_one(&cooperative_kernel());
    assert!(
        source.contains("threadgroup float tg0[3];"),
        "the tile's workgroup storage must be declared in the entry point: {source}"
    );
    assert!(
        source.contains("uint tiler_local_invocation_index [[thread_index_in_threadgroup]]"),
        "a cooperative kernel must name its participants' local coordinate: {source}"
    );
    assert!(
        source.contains("tg0[") && source.contains("] = "),
        "the producing phase must store into staging: {source}"
    );
    assert!(
        source.contains("threadgroup_barrier(mem_flags::mem_threadgroup);"),
        "the staged handoff must be fenced: {source}"
    );
    // The fence separates the two phases, so the barrier must sit between the
    // staged store and the staged load in the emitted order — the property the
    // structured-kernel verifier proves and the emitter must not reorder.
    let store = source.find("  // tile phase 0").expect("a staged store");
    let fence = source
        .find("threadgroup_barrier")
        .expect("the discharging barrier");
    let load = source.find("  // tile phase 1").expect("a staged load");
    assert!(
        store < fence && fence < load,
        "the barrier must separate the staged write from the staged read: {source}"
    );
}

/// A loop-carried tile emits the peeled round, the round loop, and both
/// barriers, in that order.
///
/// This is emission structure, not execution: a golden or a compile success
/// would still say nothing about whether the barriers synchronize on a device.
/// What it pins is that the MSL is the peeled body the KIR lowering produces —
/// round zero's phase fence at the top level, then a `1..rounds` loop whose
/// first statement is the round-boundary fence — so a later edit that hoisted
/// the round fence out of the loop, or dropped it, cannot hide behind an
/// unchanged single-round golden.
#[test]
fn the_loop_carried_kernel_emits_a_peeled_round_loop_and_both_fences() {
    let source = emit_one(&loop_carried_cooperative_kernel());
    assert!(
        source.contains("threadgroup float tg0[3];"),
        "the tile's workgroup storage must be declared: {source}"
    );
    assert!(
        source.contains("uint tiler_local_invocation_index [[thread_index_in_threadgroup]]"),
        "a cooperative kernel must name its participants' local coordinate: {source}"
    );
    let peel_store = source
        .find("  // tile phase 0")
        .expect("the peeled staged store");
    let peel_fence = source
        .find("threadgroup_barrier")
        .expect("the peeled phase fence");
    let round_loop = source
        .find("// serial loop over [1, 2)")
        .expect("the 1..rounds loop");
    assert!(
        peel_store < peel_fence && peel_fence < round_loop,
        "the peeled phase fence must sit between the peeled store and the round loop: {source}"
    );
    let loop_body = &source[round_loop..];
    let loop_fences = loop_body.matches("threadgroup_barrier").count();
    assert!(
        loop_fences >= 2,
        "the round body must realize the round boundary and the phase boundary: {source}"
    );
    let first_in_loop = loop_body
        .find("threadgroup_barrier")
        .expect("a fence inside the round loop");
    let first_loop_store = loop_body
        .find("  // tile phase 0")
        .expect("the loop-body staged store");
    assert!(
        first_in_loop < first_loop_store,
        "the round-boundary fence must be the first effect of the round body: {source}"
    );
}

/// Staging is declared inside the entry point, never in the argument table.
///
/// The ordinals are positional, so admitting workgroup storage as a
/// `[[buffer(N)]]` parameter would re-base every later index and change what an
/// existing signature position means. This pins that the cooperative kernel's
/// two boundary tensors still occupy buffers 0 and 1.
#[test]
fn workgroup_staging_takes_no_argument_table_position() {
    let source = emit_one(&cooperative_kernel());
    assert!(
        source.contains("device const float *b0 [[buffer(0)]]"),
        "{source}"
    );
    assert!(
        source.contains("device float *b1 [[buffer(1)]]"),
        "{source}"
    );
    assert!(
        !source.contains("[[buffer(2)]]"),
        "workgroup staging must not claim an argument-table ordinal: {source}"
    );
    assert!(
        !source.contains("[[threadgroup("),
        "statically extended staging is a function-scope declaration, not a binding: {source}"
    );
}

#[test]
fn independently_lowered_kernels_emit_identical_bytes() {
    // Two lowerings of one scheduled region carry different builder ownership
    // tags, so identical bytes prove no handle identity reaches the output.
    let region = pointwise_region(RegionId::new(0), &Shape::from_dims([4]), NAN_BITS);
    let first = lower_scheduled_region(&region).unwrap();
    let second = lower_scheduled_region(&region).unwrap();
    assert_eq!(emit_one(&first), emit_one(&second));
}

#[test]
fn portfolio_order_does_not_change_emitted_bytes() {
    let pointwise = pointwise_kernel();
    let reduction = single_axis_reduction_kernel();
    let forward = emit_translation_unit(&[&pointwise, &reduction], &target()).unwrap();
    let reverse = emit_translation_unit(&[&reduction, &pointwise], &target()).unwrap();
    assert_eq!(forward.source(), reverse.source());
    assert_eq!(forward.entry_points(), reverse.entry_points());
}

#[test]
fn an_empty_portfolio_emits_a_declaration_free_translation_unit() {
    // An empty portfolio is a degenerate but legal request: the provenance
    // header is still emitted, and nothing is declared.
    let unit = emit_translation_unit(&[], &target()).unwrap();
    assert!(unit.entry_points().is_empty());
    assert!(unit.numerical_requirements().is_empty());
    assert!(!unit.source().contains("kernel void "));
    assert!(unit.source().contains("#include <metal_stdlib>"));
    // No emitted f32 arithmetic means no obligation the target cannot realize,
    // so a unit with nothing to compute conforms vacuously.
    assert!(unit.numerical_gaps().is_empty());
    unit.require_declared_realization().unwrap();
    assert!(
        unit.source()
            .contains("// Declared numerical obligations this profile cannot realize: none.")
    );
}

/// Spellings that would mean a fused or simdgroup accumulation had been asked for.
const FUSED_ACCUMULATION_SPELLINGS: [&str; 6] = [
    "fma(",
    "metal::fma",
    "precise::fma",
    "simdgroup",
    "multiply_accumulate",
    "mad(",
];

/// Emits the bounded contraction fixture and returns its translation unit.
fn contraction_unit() -> MetalTranslationUnit {
    let kernel = contraction_kernel();
    emit_translation_unit(&[&kernel], &target()).expect("the contraction fixture emits")
}

/// The fusion subject: no fused spelling and no adjacent multiply-add.
///
/// [Finding 16 of the Apple numerical-behaviour record] established that
/// `-ffp-contract=off` is a defence against the *compiler* contracting a written
/// multiply and add, and no defence at all against a fused operation the source
/// asks for; the L3 realization probe reproduced it at a new construct, where
/// `simdgroup_multiply_accumulate` returned the fused `0x3fc58f9d` under exactly
/// the governed flags that kept the four scalar kernels at `0x3fc58f9e`.
///
/// [Finding 16 of the Apple numerical-behaviour record]: ../../docs/research/apple-targets/numerical-behaviour.md
fn refuse_fused_accumulation(source: &str) {
    for forbidden in FUSED_ACCUMULATION_SPELLINGS {
        assert!(
            !source.contains(forbidden),
            "{forbidden} must not appear on a path whose contract forbids contraction:\n{source}"
        );
    }
    let arithmetic: Vec<&str> = source
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("float ") && line.contains(" = "))
        .collect();
    assert!(
        arithmetic.iter().any(|line| line.contains(" * ")),
        "the fixture must actually emit the product:\n{source}"
    );
    assert!(
        arithmetic.iter().any(|line| line.contains(" + ")),
        "the fixture must actually emit the accumulation:\n{source}"
    );
    for line in &arithmetic {
        assert!(
            !(line.contains(" * ") && line.contains(" + ")),
            "one statement carries both operators, so the pair is fusable:\n{line}"
        );
    }
    assert_eq!(
        arithmetic
            .iter()
            .filter(|line| line.contains(" * "))
            .count(),
        2,
        "one product for the seed and one inside the fold:\n{source}"
    );
    assert_eq!(
        arithmetic
            .iter()
            .filter(|line| line.contains(" + "))
            .count(),
        1,
        "one accumulation, inside the fold:\n{source}"
    );
}

/// The seed subject: the fold starts at the first product, never at `+0.0`.
fn refuse_positive_zero_seed(source: &str) {
    assert!(
        source.contains("// serial loop over [1, "),
        "the emitted fold must start at the first product:\n{source}"
    );
    assert!(
        !source.contains("// serial loop over [0, "),
        "a +0.0 seed is a different operation from the first-product fold:\n{source}"
    );
}

/// The NaN/order subject: every combine commits the canonical payload.
fn refuse_missing_per_combine_canonicalization(source: &str) {
    assert_eq!(
        source
            .matches("tiler_canonicalize_nan_f32_7fc00000(")
            .count(),
        // The helper definition, the seed's product, the fold's product, and the
        // fold's sum.
        4,
        "each emitted product and sum commits the canonical payload:\n{source}"
    );
}

/// The contraction's accumulation path carries no fused multiply-add.
///
/// **The flag is not what holds this line, which is why the check is on the
/// text.** So the property asserted here is per-statement structure, which no
/// flag can change: every emitted arithmetic line binds one operator over two
/// already named locals, so no statement contains a product feeding a sum.
/// The flag requirement is still recorded — the last assertion — but as a second
/// line of defence rather than as the guarantee.
#[test]
fn the_contraction_kernel_emits_no_fused_multiply_add_on_its_accumulation_path() {
    let unit = contraction_unit();
    let source = unit.source();
    refuse_fused_accumulation(source);
    assert!(
        unit.numerical_requirements()
            .contains(&MetalNumericalRequirement::NoFloatingPointContraction),
        "the forbidden contraction dimension still places its flag obligation"
    );
}

/// The emitted fold starts at contributor 1, which is the first-product seed.
#[test]
fn the_contraction_kernel_seeds_from_the_first_product() {
    refuse_positive_zero_seed(contraction_unit().source());
}

/// Every product and sum installs the declared canonical NaN payload.
#[test]
fn the_contraction_kernel_canonicalizes_after_every_combine() {
    refuse_missing_per_combine_canonicalization(contraction_unit().source());
}

/// Finding 16's `contraction_pair` and the L3 `negative_zero_seed` pair.
///
/// Host IEEE reproductions of the two observations that make a fused MMA and a
/// `+0.0` seed different operations from `@1`. They do not prove unpublished
/// contributor order or internal NaN behaviour.
#[test]
fn contraction_pair_and_negative_zero_seed_remain_the_distinguishing_observations() {
    let operand = f32::from_bits(0x3eb9_7ef9);
    assert_eq!((operand * 1.5 + 1.0).to_bits(), 0x3fc5_8f9e);
    assert_eq!(operand.mul_add(1.5, 1.0).to_bits(), 0x3fc5_8f9d);

    let product = f32::from_bits(0xbf80_0000) * f32::from_bits(0x0000_0000);
    let mut first_product = product;
    for _ in 1..16 {
        first_product += product;
    }
    let mut positive_zero_seed = 0.0_f32;
    for _ in 0..16 {
        positive_zero_seed += product;
    }
    assert_eq!(first_product.to_bits(), 0x8000_0000);
    assert_eq!(positive_zero_seed.to_bits(), 0x0000_0000);
}

/// Each load-bearing subject fails with its own message when that subject alone
/// is perturbed. The production emitter is not the subject of these rows; a
/// copy of its text is, so fusion, seed, and NaN stay independently watchable.
#[test]
fn fusion_seed_and_nan_subjects_fail_independently() {
    let source = contraction_unit().source().to_string();
    refuse_fused_accumulation(&source);
    refuse_positive_zero_seed(&source);
    refuse_missing_per_combine_canonicalization(&source);

    let fused = format!("{source}\nfloat _perturb = fma(1.0f, 1.0f, 1.0f);\n");
    let fused_failure = std::panic::catch_unwind(|| refuse_fused_accumulation(&fused));
    let fused_message = panic_message(
        fused_failure
            .expect_err("a fused spelling must fail")
            .as_ref(),
    );
    assert!(
        fused_message.contains("fma( must not appear on a path whose contract forbids contraction"),
        "fusion must quote its own refusal: {fused_message}"
    );
    refuse_positive_zero_seed(&fused);
    refuse_missing_per_combine_canonicalization(&fused);

    let seeded = source.replace("// serial loop over [1, ", "// serial loop over [0, ");
    let seed_failure = std::panic::catch_unwind(|| refuse_positive_zero_seed(&seeded));
    let seed_message = panic_message(seed_failure.expect_err("a +0.0 seed must fail").as_ref());
    assert!(
        seed_message.contains("the emitted fold must start at the first product"),
        "seed must quote its own refusal: {seed_message}"
    );
    refuse_fused_accumulation(&seeded);
    refuse_missing_per_combine_canonicalization(&seeded);

    let denan = source.replace(
        "tiler_canonicalize_nan_f32_7fc00000(",
        "tiler_identity_f32(",
    );
    let nan_failure =
        std::panic::catch_unwind(|| refuse_missing_per_combine_canonicalization(&denan));
    let nan_message = panic_message(
        nan_failure
            .expect_err("dropping canonicalize must fail")
            .as_ref(),
    );
    assert!(
        nan_message.contains("each emitted product and sum commits the canonical payload"),
        "NaN must quote its own refusal: {nan_message}"
    );
    refuse_fused_accumulation(&denan);
    refuse_positive_zero_seed(&denan);
}

fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_string())
        })
        .unwrap_or_else(|| "non-string panic payload".to_string())
}

/// The `SiLU` kernel selects the precise exponential intrinsic, by name.
///
/// **The negative half is the load-bearing one.** Under the compiler's own
/// default — which is fast math — an unqualified `exp(x)` selects
/// `air.fast_exp.f32`, whose accuracy is Metal's Table 8.2 input-dependent bound
/// rather than the constant `4 ulp` the registered contract is derived from.
/// Writing `precise::exp` selects `air.exp.f32` under both settings, so the flag
/// requirement below is a second line of defence rather than the only one.
#[test]
fn the_silu_kernel_emits_the_precise_exponential_and_a_division() {
    let kernel = silu_kernel();
    let unit = emit_translation_unit(&[&kernel], &target()).expect("the SiLU fixture emits");
    let source = unit.source();
    assert_eq!(
        source.matches("precise::exp(").count(),
        1,
        "the subordinate exponential is emitted once, in the precise namespace:\n{source}"
    );
    for forbidden in ["fast::exp", "fast_exp", "metal::divide(", "1.0f /"] {
        assert!(
            !source.contains(forbidden),
            "{forbidden} must not appear; it is a different contract:\n{source}"
        );
    }
    // The division is emitted as the operator, which is the spelling Table 8.1
    // states an accuracy for, and it is one statement rather than a reciprocal
    // followed by a multiply.
    assert_eq!(
        source.matches(" / ").count(),
        1,
        "exactly one division statement:\n{source}"
    );
}

/// The `SiLU` emission requires both governed math flags, and says which.
#[test]
fn the_silu_kernel_requires_the_precise_and_safe_selections() {
    let kernel = silu_kernel();
    let unit = emit_translation_unit(&[&kernel], &target()).expect("the SiLU fixture emits");
    let requirements = unit.numerical_requirements();
    assert!(requirements.contains(&MetalNumericalRequirement::PreciseFp32Functions));
    assert!(requirements.contains(&MetalNumericalRequirement::SafeMathMode));
    assert_eq!(
        MetalNumericalRequirement::PreciseFp32Functions.flag(),
        "-fmetal-math-fp32-functions=precise",
        "the requirement names the exact flag the applicability clause rests on"
    );
    assert_eq!(
        MetalNumericalRequirement::PreciseFp32Functions.rule(),
        "precise-fp32-functions"
    );

    // The scale-then-bias fixture emits no elementary function, so it requires
    // no precise selection. Without this the assertion above would pass for a
    // requirement that had simply been added to every unit.
    let pointwise = pointwise_kernel();
    let plain = emit_translation_unit(&[&pointwise], &target()).expect("emits");
    assert!(
        !plain
            .numerical_requirements()
            .contains(&MetalNumericalRequirement::PreciseFp32Functions),
        "a kernel with no elementary function does not demand the precise selection"
    );
}

/// The exponential is arithmetic, so it carries the subnormal obligation.
///
/// The measured Apple row flushes `f32` subnormals, and the fixture's declared
/// realization preserves them, so a gap is recorded. A construct that had been
/// wired without calling the obligation recorder would report none.
///
/// **Both verdicts are asserted, and the second is what makes the first mean
/// something.** A refusal on its own does not distinguish "this contract is
/// unrealizable on this row" from "an elementary function is refused here",
/// which would be a very different claim about the activation family. Emitting
/// the same kernel under a declaration that accepts the flush the row delivers
/// leaves an empty gap set and conforms, so the refusal above is a decision
/// about the declared realization rather than about `precise::exp`. This is the
/// shape `a_strict_bf16_contract_is_refused_on_the_measured_macos_row` takes at
/// the other width.
#[test]
fn the_silu_kernel_records_the_f32_subnormal_gap() {
    let kernel = silu_kernel();
    let unit = emit_translation_unit(&[&kernel], &target()).expect("the SiLU fixture emits");
    assert!(
        !unit.numerical_gaps().is_empty(),
        "the emitted f32 arithmetic is compared against the target's declared behaviour"
    );
    unit.require_declared_realization()
        .expect_err("a declared preservation on a flushing target fails closed");

    // The flush the measured row actually delivers, declared. Only the two
    // subnormal dimensions move: contraction and reassociation stay forbidden,
    // so nothing else could account for the change in verdict.
    let honoured_kernel = silu_kernel_under(subnormal_realization(
        "tiler.test.flush-f32",
        NAN_BITS,
        flush(FlushedZeroSign::PreservesSign),
        flush(FlushedZeroSign::PreservesSign),
    ));
    let honoured =
        emit_translation_unit(&[&honoured_kernel], &target()).expect("the SiLU fixture emits");
    assert!(honoured.numerical_gaps().is_empty());
    assert!(
        honoured.unstated_subnormal_arithmetic().is_empty(),
        "an empty gap set computed while a fact is missing would be incomplete, not clean"
    );
    honoured.require_declared_realization().unwrap();
    // The unit under test is still the one carrying the elementary function, so
    // the acceptance is about a SiLU translation unit and not about arithmetic
    // that happens to share its shape.
    assert_eq!(honoured.source().matches("precise::exp(").count(), 1);
    assert!(
        honoured
            .numerical_requirements()
            .contains(&MetalNumericalRequirement::PreciseFp32Functions),
        "the accepted unit still places the precise-selection obligation",
    );
}

#[test]
fn repeating_a_kernel_emits_one_entry_point() {
    let kernel = pointwise_kernel();
    let unit = emit_translation_unit(&[&kernel, &kernel, &kernel], &target()).unwrap();
    assert_eq!(unit.entry_points().len(), 1);
    assert_eq!(unit.source().matches("kernel void ").count(), 1);
}

#[test]
fn a_portfolio_shares_one_prologue_and_one_helper() {
    let pointwise = pointwise_kernel();
    let reduction = single_axis_reduction_kernel();
    let unit = emit_translation_unit(&[&pointwise, &reduction], &target()).unwrap();
    let source = unit.source();
    assert_eq!(source.matches("#include <metal_stdlib>").count(), 1);
    assert_eq!(source.matches("using namespace metal;").count(), 1);
    assert_eq!(
        source
            .matches("static inline float tiler_canonicalize_nan_f32_7fc00000(")
            .count(),
        1,
        "both entry points canonicalize to the same pattern, so one helper suffices"
    );
    assert_eq!(source.matches("kernel void ").count(), 2);
    assert_eq!(unit.entry_points().len(), 2);
}

#[test]
fn entry_points_are_ordered_by_canonical_identity() {
    let pointwise = pointwise_kernel();
    let reduction = single_axis_reduction_kernel();
    let unit = emit_translation_unit(&[&pointwise, &reduction], &target()).unwrap();
    let identities: Vec<&[u8]> = unit
        .entry_points()
        .iter()
        .map(|entry| entry.kernel_identity().as_bytes())
        .collect();
    assert!(identities.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn every_entry_point_symbol_is_content_derived_and_declared() {
    let unit = emit_translation_unit(
        &[&pointwise_kernel(), &multi_axis_reduction_kernel()],
        &target(),
    )
    .unwrap();
    let symbols: BTreeSet<&str> = unit
        .entry_points()
        .iter()
        .map(crate::record::MetalEntryPoint::symbol)
        .collect();
    assert_eq!(symbols.len(), unit.entry_points().len());
    for symbol in symbols {
        assert!(symbol.starts_with("tiler_kernel_"));
        assert!(
            unit.source().contains(&format!("kernel void {symbol}(\n")),
            "{symbol} must be declared in the emitted source"
        );
    }
}

#[test]
fn the_binding_table_matches_the_emitted_subscripts() {
    let kernel = pointwise_kernel();
    let unit = emit_translation_unit(&[&kernel], &target()).unwrap();
    let entry = &unit.entry_points()[0];
    let bindings = entry.buffers();
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].index(), 0);
    assert_eq!(
        bindings[0].parameter().tensor,
        TensorRole::Input {
            ordinal: InputOrdinal::FIRST
        }
    );
    assert_eq!(bindings[0].parameter().access, BufferAccess::Read);
    assert_eq!(bindings[1].index(), 1);
    assert_eq!(bindings[1].parameter().tensor, TensorRole::Intermediate);
    assert_eq!(bindings[1].parameter().access, BufferAccess::Write);
    for binding in bindings {
        assert!(
            unit.source()
                .contains(&format!("*b{0} [[buffer({0})]]", binding.index()))
        );
    }
}

fn live_row_major_kernel() -> VerifiedKernel {
    let rows = 2_u64;
    let inner = Axis::new(1);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(40));
    builder
        .iteration_shape(Shape::from_dims([rows]))
        .expect("rows");
    builder
        .push_access(Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LiveRowMajor { inner_axis: inner },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .expect("read");
    builder
        .push_access(Access {
            tensor: TensorRole::Intermediate,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LiveRowMajor { inner_axis: inner },
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .expect("write");
    for (witness, tensor) in [
        (
            0,
            TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
        ),
        (1, TensorRole::Intermediate),
    ] {
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LinearRange { element_count: 0 },
            })
            .expect("bounds");
    }
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Intermediate,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: rows },
        })
        .expect("ownership");
    builder
        .scalar_program(ScalarProgram::PointwiseF32(scale_then_bias_expression(
            SCALE_BITS, BIAS_BITS,
        )))
        .expect("scalar");
    builder.numerical(numerical(NAN_BITS)).expect("numerical");
    builder.schedule(linear_schedule(rows)).expect("schedule");
    lower_scheduled_region(&builder.build().expect("region")).expect("lowers")
}

/// A live input extent is a read-only scalar parameter, not a baked literal.
#[test]
fn a_live_extent_is_emitted_as_a_constant_parameter() {
    let kernel = live_row_major_kernel();
    let unit = emit_translation_unit(&[&kernel], &target()).unwrap();
    let source = unit.source();
    assert!(
        source.contains("constant ulong& e0 [[buffer(2)]]"),
        "{source}"
    );
    assert!(source.contains("= e0;"), "{source}");
    assert!(
        !source.contains("14ul") && !source.contains("15ul"),
        "live N must not be baked: {source}"
    );
}

#[test]
fn every_f32_immediate_is_emitted_as_an_exact_bit_pattern() {
    let source = emit_one(&pointwise_kernel());
    assert!(source.contains(&format!("as_type<float>({SCALE_BITS:#010x}u)")));
    assert!(source.contains(&format!("as_type<float>({BIAS_BITS:#010x}u)")));
    // A decimal rendering would put arithmetic behind the Metal compiler's
    // literal parsing, so none is emitted.
    assert!(!source.contains("2.0f"));
    assert!(!source.contains("1.0f"));
}

#[test]
fn the_launch_index_is_widened_explicitly() {
    let unit = emit_translation_unit(&[&pointwise_kernel()], &target()).unwrap();
    let source = unit.source();
    assert_eq!(unit.emission_realization(), emission());
    assert!(source.contains("uint tiler_global_invocation_index [[thread_position_in_grid]]"));
    assert!(
        source.contains(
            "uint64_t v0 = uint64_t(tiler_global_invocation_index);  // widened from uint"
        )
    );
    assert!(source.contains(
        "// Structured index arithmetic: uint64_t, widened explicitly from uint delivery."
    ));
    assert!(!source.contains("Launch precondition"));
}

#[test]
fn the_guard_and_the_reduction_loop_survive_translation() {
    let source = emit_one(&single_axis_reduction_kernel());
    // The tail guard is the IR's own predicated region, not a launch assumption.
    assert!(source.contains("bool v2 = v0 < v1;"));
    assert!(source.contains("if (v2) {"));
    // The reduction order is the IR's bounded loop, not a backend choice.
    assert!(source.contains("// serial loop over [1, 3)"));
}

#[test]
fn strict_numerics_require_safe_math_and_no_contraction() {
    let unit = emit_translation_unit(&[&pointwise_kernel()], &target()).unwrap();
    assert_eq!(
        unit.numerical_requirements(),
        [
            MetalNumericalRequirement::SafeMathMode,
            MetalNumericalRequirement::NoFloatingPointContraction,
        ],
    );
    assert_eq!(
        MetalNumericalRequirement::SafeMathMode.flag(),
        "-fmetal-math-mode=safe"
    );
    assert_eq!(
        MetalNumericalRequirement::NoFloatingPointContraction.flag(),
        "-ffp-contract=off"
    );
}

/// Every newly consumable dimension independently reaches Metal selection.
///
/// The baseline authorizes every safe-math licence and proves exceptional
/// values absent; contraction stays forbidden because the pointwise scalar
/// program itself requires that exact refinement. Each case then tightens one
/// safe-math dimension. This prevents the strict fixture's already-forbidden
/// reassociation from making a dropped field look covered.
#[test]
fn each_consumable_dimension_independently_requires_safe_math() {
    let baseline = NumericalRealization::new(
        "tiler.test.relaxed-f32",
        NAN_BITS,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Permitted,
        NumericalPermission::Permitted,
        NumericalPermission::Permitted,
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::CompilerProven,
        },
        ExceptionalValueAssumption::AssumeAbsent {
            provenance: ValueDomainProvenance::RuntimeValidated,
        },
    );
    let requirements = |realization| {
        realization_requirements(realization)
            .into_iter()
            .collect::<Vec<_>>()
    };
    assert_eq!(
        requirements(baseline),
        [MetalNumericalRequirement::NoFloatingPointContraction],
    );

    for realization in [
        NumericalRealization {
            permutation: NumericalPermission::Forbidden,
            ..baseline
        },
        NumericalRealization {
            signed_zero: NumericalPermission::Forbidden,
            ..baseline
        },
        NumericalRealization {
            nan_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
            ..baseline
        },
        NumericalRealization {
            infinity_assumptions: ExceptionalValueAssumption::MakeNoAssumption,
            ..baseline
        },
        NumericalRealization {
            nan_assumptions: ExceptionalValueAssumption::AssumeAbsent {
                provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
            },
            ..baseline
        },
    ] {
        assert_eq!(
            requirements(realization),
            [
                MetalNumericalRequirement::SafeMathMode,
                MetalNumericalRequirement::NoFloatingPointContraction,
            ],
        );
    }
}

/// Pins the ticket's central obligation on the one operation that can carry it.
///
/// The canonicalization predicate must be realized by the emitted operations,
/// not inherited from how a particular front end lowers `isnan` under a
/// particular math mode. Integer operations carry no floating-point relaxation
/// licence, so this predicate means the same thing under `safe`, `relaxed`, and
/// `fast`; a floating-point predicate would not.
#[test]
fn the_nan_predicate_is_an_integer_test_no_math_mode_can_relax() {
    let source = emit_one(&pointwise_kernel());
    assert!(
        !source.contains("isnan"),
        "a floating-point NaN predicate would depend on the selected math mode"
    );
    assert!(source.contains("uint pattern = as_type<uint>(value);"));
    assert!(source.contains("bool nan = (pattern & 0x7f800000u) == 0x7f800000u"));
    assert!(source.contains("&& (pattern & 0x007fffffu) != 0x00000000u;"));
    assert!(source.contains("return nan ? as_type<float>(0x7fc00000u) : value;"));
}

/// The emitted predicate and the host predicate must agree on every pattern.
///
/// They are written from the same two constants, so this walks the exponent and
/// significand boundaries the masks separate rather than trusting the sharing.
#[test]
fn the_emitted_and_host_nan_predicates_agree() {
    for bits in [
        0x0000_0000, // +0
        0x8000_0000, // -0
        0x0000_0001, // smallest subnormal
        0x007f_ffff, // largest subnormal
        0x0080_0000, // smallest normal
        0x7f7f_ffff, // largest finite
        0x7f80_0000, // +inf
        0xff80_0000, // -inf
        0x7f80_0001, // smallest signalling NaN
        0x7fbf_ffff, // largest signalling NaN
        0x7fc0_0000, // canonical quiet NaN
        0xffff_ffff, // negative quiet NaN, all significand bits set
    ] {
        let exponent_all_ones = bits & 0x7f80_0000 == 0x7f80_0000;
        let significand_nonzero = bits & 0x007f_ffff != 0;
        assert_eq!(
            is_f32_nan(bits),
            exponent_all_ones && significand_nonzero,
            "{bits:#010x}"
        );
    }
}

/// Subnormal preservation has no realization on the measured Apple profile.
///
/// The emitter must not name a compiler flag for it: measurement shows every
/// `-fmetal-math-mode` flushes subnormal operands and results through `f32`
/// arithmetic. Recording it as a gap keeps hard feasibility separate from a
/// flag selection, and the conformance step fails closed.
#[test]
fn subnormal_preservation_is_recorded_as_an_unrealizable_gap() {
    let unit = emit_translation_unit(&[&pointwise_kernel()], &target()).unwrap();
    assert_eq!(
        unit.numerical_gaps(),
        [MetalNumericalGap::SubnormalFlushInArithmetic],
    );
    let error = unit.require_declared_realization().unwrap_err();
    assert_eq!(
        error,
        MetalEmitError::UnrealizableNumericalObligation {
            gap: MetalNumericalGap::SubnormalFlushInArithmetic,
        }
    );
    assert_eq!(error.rule(), "unrealizable-numerical-obligation");
    assert_eq!(
        MetalNumericalGap::SubnormalFlushInArithmetic.rule(),
        "subnormal-flush-in-arithmetic"
    );
    // The generated source states the gap too, so it cannot be lost by a caller
    // that only keeps the text.
    assert!(
        unit.source()
            .contains("// Declared numerical obligations this profile cannot realize:")
    );
    assert!(unit.source().contains("//   subnormal-flush-in-arithmetic"));
    assert!(unit.source().contains("//   f32: flushes-to-zero"));
}

/// The strict-affine decode is honourable on the measured Apple profile.
///
/// This kernel used to be refused with `SubnormalFlushInArithmetic`, and the
/// refusal was correct while the value contract admitted a subnormal scale: the
/// decode declares subnormal preservation, and this row flushes. The contract
/// now admits only a positive *normal* scale, which makes every operand and
/// result of the decode's multiply either `+0.0` or at least the scale in
/// magnitude — so the flush has nothing to act on and the two behaviours return
/// identical bits.
///
/// **The contract was not weakened to get here.** The kernel still declares
/// `Preserve` on both dimensions, and `pointwise_kernel` declaring exactly the
/// same thing is still refused below, which is what shows this is a decision
/// about the value domain rather than a relaxed comparison.
#[test]
fn strict_affine_u4_dequantization_is_honoured_on_the_measured_apple_profile() {
    let kernel = strict_affine_u4_dequantize_kernel();
    assert_eq!(
        kernel.subnormal_freedom(),
        SubnormalFreedom::StrictAffineNormalScaleDecode,
    );
    assert_eq!(kernel.numerical().input_subnormals, SubnormalMode::Preserve);
    assert_eq!(
        kernel.numerical().result_subnormals,
        SubnormalMode::Preserve
    );

    let unit = emit_translation_unit(&[&kernel], &target()).expect("mechanical translation");
    assert!(unit.source().contains("& 0x0fu"));
    assert!(unit.source().contains("int("));
    assert!(unit.source().contains("float("));
    assert!(unit.numerical_gaps().is_empty());
    unit.require_declared_realization()
        .expect("a normal-scale decode honours its declared subnormal contract");

    // The same declaration, on a kernel with no such freedom, is still refused.
    let pointwise = pointwise_kernel();
    assert_eq!(pointwise.subnormal_freedom(), SubnormalFreedom::Unproven);
    assert_eq!(
        emit_translation_unit(&[&pointwise], &target())
            .unwrap()
            .require_declared_realization()
            .unwrap_err(),
        MetalEmitError::UnrealizableNumericalObligation {
            gap: MetalNumericalGap::SubnormalFlushInArithmetic,
        }
    );
}

/// A discharged obligation does not need the target to have stated a fact.
///
/// Where no subnormal can occur, the target's `f32` behaviour is not merely
/// agreeable — it is unobservable, so an *unmeasured* row answers the decode
/// just as well as the measured one. Asserting this pins the order of the two
/// checks: consulting the freedom after resolving the fact would report the
/// decode as using an arithmetic type with no stated behaviour and fail closed
/// on a question that has no content.
#[test]
fn a_discharged_decode_needs_no_stated_subnormal_fact() {
    let kernel = strict_affine_u4_dequantize_kernel();
    let mut facts = target();
    facts.subnormal_arithmetic = MetalSubnormalArithmeticFacts::unmeasured();
    let unit = emit_translation_unit(&[&kernel], &facts).expect("mechanical translation");
    assert!(unit.unstated_subnormal_arithmetic().is_empty());
    assert!(unit.numerical_gaps().is_empty());
    unit.require_declared_realization().unwrap();

    // Non-vacuity: the same unmeasured row refuses a kernel without the freedom.
    let unstated = emit_translation_unit(&[&pointwise_kernel()], &facts).unwrap();
    assert_eq!(
        unstated.unstated_subnormal_arithmetic(),
        [MetalFloatArithmeticType::F32],
    );
    assert_eq!(
        unstated.require_declared_realization().unwrap_err().rule(),
        "unstated-subnormal-arithmetic",
    );
}

/// The freedom is typed, and the type it does not cover still records its gap.
///
/// The decode's derivation rests on `f32`'s exponent range and on integers up
/// to 255 being exactly representable in `f32`; neither premise transfers to a
/// narrower format. Nothing emits `f16` arithmetic today, so this exercises the
/// discrimination directly rather than through a kernel that cannot exist.
#[test]
fn a_decode_freedom_discharges_f32_alone() {
    let freedom = SubnormalFreedom::StrictAffineNormalScaleDecode;
    assert!(freedom.discharges(ArithmeticType::F32));
    for other in [
        ArithmeticType::F16,
        ArithmeticType::Bf16,
        ArithmeticType::F64,
    ] {
        assert!(!freedom.discharges(other), "{other:?} must not be covered");
    }
    for arithmetic in ArithmeticType::ALL {
        assert!(!SubnormalFreedom::Unproven.discharges(arithmetic));
    }
}

/// Every arithmetic type occupies its own slot in the facts record.
///
/// The record is a fixed array keyed by [`MetalFloatArithmeticType::index`], so
/// two types sharing a slot would silently make one type's stated fact answer
/// for the other — the exact failure this ticket exists to remove. The map is
/// written as a match rather than read from the discriminant, and this proves it
/// is a bijection onto `0..COUNT` rather than trusting that it is.
#[test]
fn every_arithmetic_type_indexes_to_its_own_slot() {
    let mut seen = BTreeSet::new();
    for arithmetic_type in MetalFloatArithmeticType::ALL {
        let facts = MetalSubnormalArithmeticFacts::unmeasured().stating(
            arithmetic_type,
            MetalSubnormalArithmetic::PreservesSubnormals,
        );
        for other in MetalFloatArithmeticType::ALL {
            let stated = facts.behaviour(other).is_ok();
            assert_eq!(
                stated,
                other == arithmetic_type,
                "stating {arithmetic_type} answered for {other}"
            );
        }
        assert!(
            seen.insert(arithmetic_type.as_str()),
            "two arithmetic types share the identifier {arithmetic_type}"
        );
    }
    assert_eq!(seen.len(), MetalFloatArithmeticType::COUNT);
}

/// `bf16` is its own slot and inherits nothing, including from a dtype that
/// happens to behave identically.
///
/// The measured `bf16` flush and the measured `f32` flush are the same value,
/// which is exactly why this needs asserting: a record that answered `bf16` from
/// the `f32` entry would look correct on this row and be a guess. It would also
/// be wrong in the one direction that matters -- `f16` preserves, so a
/// neighbour-reading record has an even chance of reporting a flush against a
/// subnormal the device carries exactly.
#[test]
fn bf16_is_unknown_until_it_is_stated_even_beside_an_identical_f32_fact() {
    let flush = MetalSubnormalArithmetic::FlushesToZero {
        zero_sign: MetalFlushedZeroSign::PreservesSign,
    };
    let without = MetalSubnormalArithmeticFacts::unmeasured()
        .stating(MetalFloatArithmeticType::F32, flush)
        .stating(
            MetalFloatArithmeticType::F16,
            MetalSubnormalArithmetic::PreservesSubnormals,
        );
    let unstated = without
        .behaviour(MetalFloatArithmeticType::Bf16)
        .expect_err("an unstated bf16 is Unknown, not the f32 fact beside it");
    assert_eq!(
        unstated.arithmetic_type(),
        MetalFloatArithmeticType::Bf16,
        "the rejection names the dtype nothing was stated for",
    );
    assert_eq!(unstated.rule(), "unstated-subnormal-arithmetic");

    // Stated, it answers with its own measured row and leaves the others alone.
    let with = without.stating(MetalFloatArithmeticType::Bf16, flush);
    assert_eq!(with.behaviour(MetalFloatArithmeticType::Bf16), Ok(flush));
    assert_eq!(with.behaviour(MetalFloatArithmeticType::F32), Ok(flush));
    assert_eq!(
        with.behaviour(MetalFloatArithmeticType::F16),
        Ok(MetalSubnormalArithmetic::PreservesSubnormals),
        "adding a third row did not disturb the two already stated",
    );
}

/// A fact stated twice for one arithmetic type is a rejection, not a last-wins.
///
/// Two statements about one type are two claims. Keeping either silently would
/// drop a measurement, and a caller assembling a target profile from more than
/// one source has to reconcile them rather than discover afterwards which
/// survived.
#[test]
#[should_panic(expected = "a subnormal-arithmetic fact was stated twice")]
fn stating_one_arithmetic_type_twice_is_refused() {
    let _ = MetalSubnormalArithmeticFacts::unmeasured()
        .stating(MetalFloatArithmeticType::F32, APPLE_FLUSH)
        .stating(
            MetalFloatArithmeticType::F32,
            MetalSubnormalArithmetic::PreservesSubnormals,
        );
}

/// An unstated arithmetic type is never answered by another type's fact.
///
/// This is the whole contract consequence of finding 21. The target below states
/// the measured `f16` behaviour and says nothing about `f32`; the kernel
/// performs `f32` arithmetic. Emission must not read the `f16` fact — which
/// would report no gap and approve the strict contract — and must not assume a
/// flush either. It records the type as unstated, and the conformance claim
/// fails closed naming it.
///
/// The empty gap list is the part worth reading twice: gaps are only computed
/// for types the target speaks to, so an empty set here is an *incomplete*
/// answer rather than a conformant one, which is why
/// `require_declared_realization` consults the unstated set first.
#[test]
fn an_unstated_arithmetic_type_is_not_answered_by_another_types_fact() {
    let mut facts = target();
    facts.subnormal_arithmetic = MetalSubnormalArithmeticFacts::unmeasured().stating(
        MetalFloatArithmeticType::F16,
        MetalSubnormalArithmetic::PreservesSubnormals,
    );
    let unit = emit_translation_unit(&[&pointwise_kernel()], &facts).unwrap();

    assert_eq!(
        unit.unstated_subnormal_arithmetic(),
        [MetalFloatArithmeticType::F32],
    );
    assert!(unit.numerical_gaps().is_empty());
    let error = unit.require_declared_realization().unwrap_err();
    assert_eq!(error.rule(), "unstated-subnormal-arithmetic");
    assert_eq!(error.to_string(), "unstated-subnormal-arithmetic: f32");
    let MetalEmitError::UnstatedSubnormalArithmetic { unstated } = error else {
        panic!("an unstated fact must reject as itself, not as a gap");
    };
    assert_eq!(unstated.arithmetic_type(), MetalFloatArithmeticType::F32);

    // A caller keeping only the emitted text still carries it, and the text says
    // the obligation list above cannot be read as complete.
    assert!(
        unit.source()
            .contains("// Arithmetic types used with no stated subnormal fact:\n//   f32\n")
    );
    assert!(
        unit.source()
            .contains("// The obligations above are therefore incomplete.")
    );
    assert!(unit.source().contains("//   f32: not stated"));
    assert!(unit.source().contains("//   f16: preserves-subnormals"));
}

/// A type the target says nothing about costs nothing to a unit that never uses
/// it.
///
/// The unstated set is a property of the arithmetic this unit emitted, not of
/// the target record, so a target measured for no type at all is fully
/// conformant for a unit that performs no arithmetic. The empty portfolio is
/// the only arithmetic-free unit this crate's fixtures can build — every scalar
/// program in the bounded profile multiplies or adds — so this bounds the claim
/// to that case rather than to materialization in general.
#[test]
fn a_unit_with_no_arithmetic_reports_no_unstated_type() {
    let mut facts = target();
    facts.subnormal_arithmetic = MetalSubnormalArithmeticFacts::unmeasured();
    let unit = emit_translation_unit(&[], &facts).unwrap();
    assert!(unit.unstated_subnormal_arithmetic().is_empty());
    assert!(unit.numerical_gaps().is_empty());
    unit.require_declared_realization().unwrap();
    assert!(
        unit.source()
            .contains("// Arithmetic types used with no stated subnormal fact: none.")
    );
    assert!(unit.source().contains("//   f32: not stated"));
}

/// An unstated type is reported ahead of a gap.
///
/// A gap set computed while a fact is missing is incomplete, so reporting a gap
/// from it would present a partial comparison as a total one. The unit is
/// assembled directly because no emission can currently produce both: the
/// structured kernel IR resolves one floating-point element type, so a unit
/// whose `f32` fact is missing has no other type left to derive a gap from.
#[test]
fn an_unstated_type_is_reported_before_a_gap() {
    let unit = MetalTranslationUnit::new(
        target(),
        emission(),
        String::new(),
        Vec::new(),
        Vec::new(),
        vec![MetalNumericalGap::SubnormalFlushInArithmetic],
        vec![MetalFloatArithmeticType::F16],
    );
    let error = unit.require_declared_realization().unwrap_err();
    assert_eq!(error.rule(), "unstated-subnormal-arithmetic");
}

/// The gap is a stated target fact, not an assumption compiled into emission.
#[test]
fn a_subnormal_preserving_target_has_no_gap() {
    let mut facts = target();
    facts.subnormal_arithmetic = subnormal_facts(MetalSubnormalArithmetic::PreservesSubnormals);
    let unit = emit_translation_unit(&[&pointwise_kernel()], &facts).unwrap();
    assert!(unit.numerical_gaps().is_empty());
    unit.require_declared_realization().unwrap();
    // The requirement set is unchanged: a target that preserves subnormals still
    // relaxes signed zero and reassociation under a non-safe math mode.
    assert_eq!(
        unit.numerical_requirements(),
        [
            MetalNumericalRequirement::SafeMathMode,
            MetalNumericalRequirement::NoFloatingPointContraction,
        ],
    );
}

/// Every kernel of the bounded proof profile carries the same gap.
///
/// The reduction kernels reach `f32` arithmetic through a loop body rather than
/// a straight-line prologue, so this checks the obligation is recorded from the
/// operation vocabulary and not from one emission path.
#[test]
fn every_arithmetic_kernel_records_the_subnormal_gap() {
    for kernel in [
        pointwise_kernel(),
        single_axis_reduction_kernel(),
        multi_axis_reduction_kernel(),
        fused_reduction_kernel(),
    ] {
        let unit = emit_translation_unit(&[&kernel], &target()).unwrap();
        assert_eq!(
            unit.numerical_gaps(),
            [MetalNumericalGap::SubnormalFlushInArithmetic],
        );
    }
}

/// A flush the target honours is a positive conformance claim, not a weaker one.
///
/// This is the arm the widened vocabulary made expressible, and it is the one
/// that turns the Apple row from "refuse every contract" into "honour the one
/// the hardware delivers". The non-vacuity assertion matters more than the
/// conformance one: the same kernel under the strict realization records the
/// flush gap, which proves this fixture reaches `record_subnormal_obligation`
/// at all, so the empty gap set below is a decision about the contract rather
/// than a kernel with no arithmetic in it.
#[test]
fn a_flush_the_target_delivers_is_honoured_over_real_arithmetic() {
    let strict = emit_pointwise_under(numerical(NAN_BITS), APPLE_FLUSH);
    assert_eq!(
        strict.numerical_gaps(),
        [MetalNumericalGap::SubnormalFlushInArithmetic],
        "this fixture must reach the subnormal comparison for the check below to mean anything"
    );

    let unit = emit_pointwise_under(
        subnormal_realization(
            "tiler.test.flush-f32",
            NAN_BITS,
            flush(FlushedZeroSign::PreservesSign),
            flush(FlushedZeroSign::PreservesSign),
        ),
        APPLE_FLUSH,
    );
    assert!(unit.numerical_gaps().is_empty());
    unit.require_declared_realization().unwrap();
    assert!(
        unit.source()
            .contains("// Declared numerical obligations this profile cannot realize: none.")
    );
    assert!(
        unit.source()
            .contains("//   f32: flushes-to-zero-preserving-sign")
    );
}

/// A declared zero the target does not produce fails closed.
///
/// The two zeros are different results, not different precisions: the measured
/// Apple flush preserves the sign of the value it replaces, so a program that
/// asked for `AlwaysPositive` would read `0x80000000` where it required
/// `0x00000000`. Returning no gap here would be a wrong answer rather than a
/// relaxed one, which is why the sign is compared instead of assumed.
#[test]
fn a_flush_to_the_other_zero_is_refused() {
    let unit = emit_pointwise_under(
        subnormal_realization(
            "tiler.test.flush-f32-always-positive",
            NAN_BITS,
            flush(FlushedZeroSign::AlwaysPositive),
            flush(FlushedZeroSign::AlwaysPositive),
        ),
        APPLE_FLUSH,
    );
    assert_eq!(
        unit.numerical_gaps(),
        [MetalNumericalGap::FlushedZeroSignMismatch],
    );
    assert_eq!(
        unit.require_declared_realization().unwrap_err(),
        MetalEmitError::UnrealizableNumericalObligation {
            gap: MetalNumericalGap::FlushedZeroSignMismatch,
        }
    );
    assert_eq!(
        MetalNumericalGap::FlushedZeroSignMismatch.rule(),
        "flushed-zero-sign-mismatch"
    );
    assert!(unit.source().contains("//   flushed-zero-sign-mismatch"));
}

/// Agreement, not flushing, is what the honoured arm requires.
///
/// No governed Apple family has been measured to flush to the always-positive
/// zero. Stating a target that does proves the comparison honours agreement in
/// both directions rather than special-casing the one measured value, and it is
/// the only exercise of that target fact's emitted spelling.
#[test]
fn an_always_positive_flush_is_honoured_by_an_always_positive_target() {
    let unit = emit_pointwise_under(
        subnormal_realization(
            "tiler.test.flush-f32-always-positive",
            NAN_BITS,
            flush(FlushedZeroSign::AlwaysPositive),
            flush(FlushedZeroSign::AlwaysPositive),
        ),
        MetalSubnormalArithmetic::FlushesToZero {
            zero_sign: MetalFlushedZeroSign::AlwaysPositive,
        },
    );
    assert!(unit.numerical_gaps().is_empty());
    unit.require_declared_realization().unwrap();
    assert!(
        unit.source()
            .contains("//   f32: flushes-to-zero-always-positive")
    );
}

/// A declared flush is not honoured by a target that preserves subnormals.
///
/// Emission never narrows, widens, or substitutes the declared contract to fit
/// a target, and this is the direction where doing so would be tempting because
/// preservation is the stronger behaviour. It is not the *declared* behaviour:
/// honouring the flush would mean emitting an explicit one, which is emulation,
/// which this backend does not express.
#[test]
fn a_flush_contract_on_a_preserving_target_is_refused() {
    let unit = emit_pointwise_under(
        subnormal_realization(
            "tiler.test.flush-f32",
            NAN_BITS,
            flush(FlushedZeroSign::PreservesSign),
            flush(FlushedZeroSign::PreservesSign),
        ),
        MetalSubnormalArithmetic::PreservesSubnormals,
    );
    assert_eq!(
        unit.numerical_gaps(),
        [MetalNumericalGap::SubnormalPreservationInArithmetic],
    );
    assert_eq!(
        MetalNumericalGap::SubnormalPreservationInArithmetic.rule(),
        "subnormal-preservation-in-arithmetic"
    );
}

/// The two subnormal dimensions are compared independently.
///
/// A target that couples input and result flushing in one execution mode does
/// not couple the contract's semantic dimensions (ADR 0019). Declaring a
/// mismatched flush on the input dimension and preservation on the result
/// dimension therefore yields *two different* gaps from one kernel — which a
/// pair of cases that happened to produce the same gap could not distinguish
/// from a single coupled comparison.
///
/// It also pins the documented rejection order: `require_declared_realization`
/// names the first gap in ascending governed order.
#[test]
fn the_two_subnormal_dimensions_are_compared_independently() {
    let unit = emit_pointwise_under(
        subnormal_realization(
            "tiler.test.mixed-subnormal-f32",
            NAN_BITS,
            flush(FlushedZeroSign::AlwaysPositive),
            SubnormalMode::Preserve,
        ),
        APPLE_FLUSH,
    );
    assert_eq!(
        unit.numerical_gaps(),
        [
            MetalNumericalGap::SubnormalFlushInArithmetic,
            MetalNumericalGap::FlushedZeroSignMismatch,
        ],
    );
    assert_eq!(
        unit.require_declared_realization().unwrap_err(),
        MetalEmitError::UnrealizableNumericalObligation {
            gap: MetalNumericalGap::SubnormalFlushInArithmetic,
        }
    );
}

/// Pins the ticket's central property mechanically.
///
/// The emitter must translate the structured operation vocabulary and nothing
/// else. Naming a semantic-graph or schedule shape here would mean either
/// reconstructing meaning the kernel IR already states or papering over a gap
/// in it. The check reads the module's own source, so it pins the absence of
/// the identifier rather than a claim about behaviour.
#[test]
fn emission_never_names_a_semantic_or_schedule_shape() {
    const EMITTER: &str = include_str!("emit.rs");
    for forbidden in [
        "ScalarProgram",
        "ReductionTopology",
        "LogicalAccess",
        "BoundsProofKind",
        "OwnershipProofKind",
        "ExecutionBinding",
        "TailPolicy",
        "SemanticProgram",
        "IndexRegion",
        "ScheduledRegion",
    ] {
        assert!(
            !EMITTER.contains(forbidden),
            "emit.rs must not reference {forbidden}: translation reads OperationView only"
        );
    }
}

#[test]
fn governed_types_map_to_their_metal_spellings() {
    assert_eq!(msl_type(KernelType::Bool), Ok("bool"));
    assert_eq!(msl_type(KernelType::U8), Ok("uchar"));
    assert_eq!(msl_type(KernelType::I32), Ok("int"));
    assert_eq!(msl_type(KernelType::Index), Ok("uint64_t"));
    assert_eq!(msl_type(KernelType::F32), Ok("float"));
    assert_eq!(msl_type(KernelType::Bf16), Ok("bfloat"));
}

/// The unspelled-type refusal keeps its identifier and rendering while its arm
/// is vacant.
///
/// Every governed `KernelType` now has an MSL spelling, so `msl_type` cannot
/// currently return this — and that is a statement about today's vocabulary,
/// not about the diagnostic. `KernelType` is deliberately not
/// `#[non_exhaustive]`, so `F16` or `F64` stops the build at that match, and the
/// decision available there has to include "refuse". Keeping the variant
/// exercised means the widening that reaches for it finds a rule identifier and
/// a rendering that already work, rather than a surface nothing has checked
/// since BF16 stopped using it.
#[test]
fn the_unspelled_value_type_refusal_keeps_its_rule_and_rendering() {
    let refusal = MetalEmitError::UnsupportedValueType {
        value_type: KernelType::Bf16,
    };
    assert_eq!(refusal.rule(), "unsupported-value-type");
    assert_eq!(refusal.to_string(), "unsupported-value-type: Bf16");
    // The type it names is carried rather than formatted away, so a widened
    // vocabulary can report which member was refused.
    assert_ne!(
        refusal,
        MetalEmitError::UnsupportedValueType {
            value_type: KernelType::F32,
        },
    );
}

/// A BF16 kernel emits `bfloat` in every position a type appears in.
///
/// Every position rather than the arithmetic alone, because a lowering that
/// spelled the operators at `bfloat` and the buffers at `float` would compute
/// something the region does not mean while still passing an "is there a
/// `bfloat` in it" question — and at two versus four bytes per element it would
/// also misread the whole buffer. The `float` neighbour is asserted absent for
/// the same reason: the fixture is pure BF16, so any `float` in it came from a
/// path that fell back rather than translated.
#[test]
fn a_bf16_kernel_spells_bfloat_at_every_position() {
    let unit = emit_bf16_pointwise_under(bf16_numerical(), APPLE_FLUSH);
    let source = unit.source();

    // Signature: both buffers, in declaration order, at the region's own width.
    assert!(
        source.contains("device const bfloat *b0 [[buffer(0)]]"),
        "{source}"
    );
    assert!(
        source.contains("device bfloat *b1 [[buffer(1)]]"),
        "{source}"
    );
    // Body: the load, both constants, both operators, and both
    // canonicalizations. Anchored on the guarded body's own indentation, so the
    // helper's `(bfloat value)` parameter is not counted as one of them.
    assert_eq!(
        source.matches("\n        bfloat v").count(),
        7,
        "load, two constants, two operators, and two canonicalizations: {source}"
    );
    assert!(source.contains("bfloat v5 = v3 * v4;"), "{source}");
    assert!(source.contains("bfloat v8 = v6 + v7;"), "{source}");
    assert!(source.contains("b1[v0] = v9;"), "{source}");

    // No `float` spelling reaches a value of this kernel. The token is
    // space-anchored so `bfloat` is not itself read as a match, and the launch
    // index is still `uint` with the index arithmetic still `uint64_t` — this
    // is a claim about the floating-point positions, not about the whole file.
    assert!(!source.contains(" float "), "{source}");
    assert!(!source.contains("as_type<float>"), "{source}");
    assert!(
        !source.contains(CANONICALIZE_F32_SYMBOL),
        "a bf16 kernel must not reach the f32 canonicalization: {source}"
    );
}

/// A BF16 immediate is its exact sixteen-bit pattern, carried by `ushort`.
///
/// Two claims and they fail differently. The pattern must be the payload the
/// region declared — a decimal rendering, or a widening through `f32` and back,
/// would be a different value at every pattern the two roundings disagree on.
/// And the carrier must be `ushort`: an unsuffixed MSL integer literal is
/// `uint`, and `as_type` requires equal sizes, so the `f32` spelling applied at
/// this width would not compile at all.
#[test]
fn bf16_immediates_are_exact_patterns_reinterpreted_through_ushort() {
    let unit = emit_bf16_pointwise_under(bf16_numerical(), APPLE_FLUSH);
    let source = unit.source();
    assert!(
        source.contains("as_type<bfloat>(ushort(0x4000u))"),
        "the bf16 2.0 payload is emitted unchanged: {source}"
    );
    assert!(
        source.contains("as_type<bfloat>(ushort(0x3f80u))"),
        "the bf16 1.0 payload is emitted unchanged: {source}"
    );
    // The `f32` payloads of the same two values are absent, which is what
    // separates "emitted at the right width" from "emitted at all".
    assert!(!source.contains("0x40000000u"), "{source}");
    assert!(!source.contains("0x3f800000u"), "{source}");
}

/// The BF16 canonicalization helper is its own function, named the way the
/// Apple probe harness's recognizer reads it.
///
/// The recognizer matches the C++-mangled spelling
/// `_ZL32tiler_canonicalize_nan_bf16_7fc0DF16b`, which encodes the identifier's
/// length and its `bfloat` parameter — so an emitted helper that merely
/// contained the dtype somewhere would mangle to a different symbol and be
/// invisible to it. The unmangled name is therefore pinned character for
/// character, and the length prefix `32` is asserted from the identifier itself
/// rather than copied, so a rename cannot leave the two disagreeing.
///
/// **This is a name-shape agreement, not a run.** Nothing here dispatches, and
/// nothing here establishes that the harness would classify a module this
/// backend emitted; it establishes that the symbol it would look for is the
/// symbol this backend writes.
#[test]
fn the_bf16_canonicalization_helper_matches_the_apple_harness_recognizer() {
    let unit = emit_bf16_pointwise_under(bf16_numerical(), APPLE_FLUSH);
    let source = unit.source();

    assert_eq!(CANONICALIZE_BF16_SYMBOL, "tiler_canonicalize_nan_bf16_7fc0");
    assert_eq!(
        CANONICALIZE_BF16_SYMBOL.len(),
        32,
        "the harness's mangled spelling carries this identifier's length"
    );
    assert!(
        source.contains(&format!(
            "static inline bfloat {CANONICALIZE_BF16_SYMBOL}(bfloat value) {{"
        )),
        "{source}"
    );
    // Declared once, called twice: one canonicalization per arithmetic result.
    assert_eq!(
        source.matches(CANONICALIZE_BF16_SYMBOL).count(),
        3,
        "{source}"
    );

    // The predicate is an integer test at this width, so no math-mode
    // relaxation licence reaches it, and no floating-point NaN predicate is
    // emitted in its place.
    assert!(
        source.contains("ushort pattern = as_type<ushort>(value);"),
        "{source}"
    );
    assert!(
        source.contains("bool nan = (pattern & 0x7f80u) == 0x7f80u"),
        "{source}"
    );
    assert!(
        source.contains("&& (pattern & 0x007fu) != 0x0000u;"),
        "{source}"
    );
    assert!(
        source.contains("return nan ? as_type<bfloat>(ushort(0x7fc0u)) : value;"),
        "{source}"
    );
    assert!(!source.contains("isnan"), "{source}");

    // Distinct from the binary32 helper rather than an overload of it: the two
    // take different parameter types and one unit can need both.
    assert_ne!(CANONICALIZE_BF16_SYMBOL, CANONICALIZE_F32_SYMBOL);
}

/// A portfolio holding both widths emits both helpers, once each, in a fixed
/// order.
///
/// This is the case the single-kernel tests cannot reach and the one that makes
/// the two helpers' separate names load-bearing. It also pins that adding the
/// `bf16` helper did not disturb the emitted order of the two that existed.
#[test]
fn a_mixed_width_portfolio_emits_both_canonicalization_helpers_once() {
    let f32_kernel = pointwise_kernel();
    let bf16_kernel = bf16_pointwise_kernel();
    let mut facts = target();
    facts.subnormal_arithmetic = bf16_subnormal_facts(APPLE_FLUSH);
    let unit = emit_translation_unit(&[&f32_kernel, &bf16_kernel], &facts).unwrap();
    let source = unit.source();

    assert_eq!(unit.entry_points().len(), 2);
    let f32_definition = source
        .find(&format!("static inline float {CANONICALIZE_F32_SYMBOL}("))
        .expect("the binary32 helper is defined");
    let bf16_definition = source
        .find(&format!("static inline bfloat {CANONICALIZE_BF16_SYMBOL}("))
        .expect("the bfloat16 helper is defined");
    assert!(
        f32_definition < bf16_definition,
        "helpers are emitted in a fixed order: {source}"
    );
    assert_eq!(
        source
            .matches(&format!("static inline float {CANONICALIZE_F32_SYMBOL}("))
            .count(),
        1,
    );
    assert_eq!(
        source
            .matches(&format!("static inline bfloat {CANONICALIZE_BF16_SYMBOL}("))
            .count(),
        1,
    );
    // Portfolio order does not change the bytes, at mixed widths too.
    assert_eq!(
        source,
        emit_translation_unit(&[&bf16_kernel, &f32_kernel], &facts)
            .unwrap()
            .source(),
    );
}

/// The subnormal obligation is recorded against `bf16` and never against `f32`.
///
/// The two measured Apple rows agree on this host, which is exactly why this
/// needs a target that disagrees: the `f32` entry stays the measured flush in
/// both halves below and only the `bf16` entry moves, so a verdict that changed
/// with it is evidence the lookup used the operation's own arithmetic type. A
/// backend that read the `f32` fact would report the same gap in both halves.
#[test]
fn bf16_arithmetic_reads_the_bf16_fact_and_not_the_f32_one() {
    let flushing = emit_bf16_pointwise_under(bf16_numerical(), APPLE_FLUSH);
    assert_eq!(
        flushing.numerical_gaps(),
        [MetalNumericalGap::SubnormalFlushInArithmetic],
    );

    let preserving = emit_bf16_pointwise_under(
        bf16_numerical(),
        MetalSubnormalArithmetic::PreservesSubnormals,
    );
    assert!(
        preserving.numerical_gaps().is_empty(),
        "the f32 flush beside it must not answer for bf16 arithmetic",
    );
    preserving.require_declared_realization().unwrap();
    assert!(
        preserving
            .source()
            .contains("//   f32: flushes-to-zero-preserving-sign"),
        "the f32 row is still stated, so the empty gap set is a dtype decision",
    );
    assert!(
        preserving
            .source()
            .contains("//   bf16: preserves-subnormals")
    );
}

/// A strict subnormal-preserving BF16 contract is refused on the measured macOS
/// row, and the refusal names the numerical gap.
///
/// Finding 24 measures BF16 arithmetic flushing subnormal operands and results
/// on that row, sign-preserving, with an execution witness on every verdict. So
/// a contract that requires preservation is unrealizable there by any compiler
/// selection, and the fail-closed answer is a named gap rather than a flag that
/// would not deliver it.
#[test]
fn a_strict_bf16_contract_is_refused_on_the_measured_macos_row() {
    let unit = emit_bf16_pointwise_under(bf16_numerical(), APPLE_FLUSH);
    assert_eq!(
        unit.numerical_gaps(),
        [MetalNumericalGap::SubnormalFlushInArithmetic],
    );
    assert!(unit.unstated_subnormal_arithmetic().is_empty());
    assert_eq!(
        unit.require_declared_realization().unwrap_err(),
        MetalEmitError::UnrealizableNumericalObligation {
            gap: MetalNumericalGap::SubnormalFlushInArithmetic,
        },
    );
    assert!(
        unit.source().contains("//   subnormal-flush-in-arithmetic"),
        "a caller keeping only the emitted text still reads the gap",
    );

    // The flush the row actually delivers is honoured, which is what makes the
    // refusal above a decision about the contract rather than a blanket one.
    let honoured = emit_bf16_pointwise_under(
        subnormal_realization(
            "tiler.test.flush-bf16",
            bf16_nan_bits(),
            flush(FlushedZeroSign::PreservesSign),
            flush(FlushedZeroSign::PreservesSign),
        ),
        APPLE_FLUSH,
    );
    assert!(honoured.numerical_gaps().is_empty());
    honoured.require_declared_realization().unwrap();
}

/// A target stating no BF16 subnormal fact refuses the unit, ahead of any gap.
///
/// This is the `Unknown` path, and it is the one an unmeasured family reaches:
/// the retained record leaves `bf16` unmeasured on `IOsDevice` because nothing
/// dispatched it there. The two neighbouring facts are stated and *disagree*
/// with each other, so there is no fallback that is merely less precise, and
/// the refusal has to be the missing measurement rather than a guess.
#[test]
fn an_unstated_bf16_fact_refuses_the_unit_naming_the_dtype() {
    let mut facts = target();
    facts.subnormal_arithmetic = MetalSubnormalArithmeticFacts::unmeasured()
        .stating(MetalFloatArithmeticType::F32, APPLE_FLUSH)
        .stating(
            MetalFloatArithmeticType::F16,
            MetalSubnormalArithmetic::PreservesSubnormals,
        );
    let kernel = bf16_pointwise_kernel();
    let unit = emit_translation_unit(&[&kernel], &facts).unwrap();

    assert_eq!(
        unit.unstated_subnormal_arithmetic(),
        [MetalFloatArithmeticType::Bf16],
    );
    assert!(
        unit.numerical_gaps().is_empty(),
        "gaps computed while a fact is missing are incomplete, not shorter",
    );
    let error = unit.require_declared_realization().unwrap_err();
    assert_eq!(error.rule(), "unstated-subnormal-arithmetic");
    assert_eq!(error.to_string(), "unstated-subnormal-arithmetic: bf16");
    let MetalEmitError::UnstatedSubnormalArithmetic { unstated } = error else {
        panic!("an unstated fact must reject as itself, not as a gap");
    };
    assert_eq!(unstated.arithmetic_type(), MetalFloatArithmeticType::Bf16);
    assert!(
        unit.source()
            .contains("// Arithmetic types used with no stated subnormal fact:\n//   bf16\n")
    );
    assert!(unit.source().contains("//   bf16: not stated"));
}

/// The iOS-Simulator profile refuses the same BF16 program before any
/// compilation or submission.
///
/// **What this establishes and what it does not.** The refusal here is the
/// `Unknown` subnormal fact, taken at the conformance claim — which is before
/// the source is handed to a compiler, and therefore before any pipeline
/// creation or command submission. Finding 26 measures that family compiling
/// and *linking* every `bfloat` module and then failing pipeline creation with
/// `XPC_ERROR_CONNECTION_INTERRUPTED`, so a refusal that waited for the
/// toolchain would not have refused at all.
///
/// It is **not** the dtype-dispatchability refusal. That fact is a target
/// profile's (`DTypeDispatchability` at `AvailabilityPhase::CompileProfile`,
/// owned by `tiler-compiler` and declared by `tiler-build`), it is what stops a
/// BF16 program on a family that compiles the module and cannot run it, and
/// `validate-bf16-at-the-runtime-routing-boundary` is the ticket that consumes
/// it before the one-way routing commit. Nothing in this crate can reach it:
/// `tiler-metal` does not depend on `tiler-compiler` and must not.
///
/// The emitted *source* is deliberately identical to the macOS one, which is
/// the reason the refusal cannot live in emission: the two families differ in
/// whether the module runs, not in what it says.
#[test]
fn the_ios_simulator_profile_refuses_a_bf16_unit_before_any_compilation() {
    let kernel = bf16_pointwise_kernel();
    let mut simulator = target();
    simulator.platform = MetalPlatform::IOsSimulator;
    simulator.subnormal_arithmetic = MetalSubnormalArithmeticFacts::unmeasured()
        .stating(MetalFloatArithmeticType::F32, APPLE_FLUSH)
        .stating(
            MetalFloatArithmeticType::F16,
            MetalSubnormalArithmetic::PreservesSubnormals,
        );
    let unit = emit_translation_unit(&[&kernel], &simulator).unwrap();
    assert_eq!(
        unit.require_declared_realization().unwrap_err().rule(),
        "unstated-subnormal-arithmetic",
    );
    assert!(
        unit.source()
            .contains("// Artifact family: ios-simulator (deployment minimum 14.0)")
    );

    // An `f32` kernel on the same profile is unaffected, so the refusal is
    // about the dtype rather than about the family being refused wholesale.
    let f32_unit = emit_translation_unit(&[&pointwise_kernel()], &simulator).unwrap();
    assert!(f32_unit.unstated_subnormal_arithmetic().is_empty());
}

/// The contraction defence is recorded at BF16's own width.
///
/// Finding 28 measures one per-dtype difference in the strictest cell: under
/// `safe` with `-ffp-contract=fast`, `f16` fuses and `bf16` does not. So an
/// `f16` conclusion does not carry across, and `-ffp-contract=off` is asserted
/// for a BF16 unit directly rather than inherited from the `f32` fixture.
///
/// The per-statement emission is the other half of the defence and is asserted
/// beside it: each arithmetic operation is its own statement, so no contraction
/// can form across two structured operations even under `-ffp-contract=on`.
#[test]
fn a_bf16_unit_records_the_contraction_defence_at_its_own_width() {
    let unit = emit_bf16_pointwise_under(bf16_numerical(), APPLE_FLUSH);
    assert_eq!(
        unit.numerical_requirements(),
        [
            MetalNumericalRequirement::SafeMathMode,
            MetalNumericalRequirement::NoFloatingPointContraction,
        ],
    );
    assert_eq!(
        MetalNumericalRequirement::NoFloatingPointContraction.flag(),
        "-ffp-contract=off",
    );
    // The multiply and the add are separate statements, and no `fma` is
    // emitted between them: MSL has no `bfloat` overload of it, so a BF16
    // contraction has nothing to lower to at the source level.
    let source = unit.source();
    assert!(source.contains(" * "), "{source}");
    assert!(source.contains(" + "), "{source}");
    assert!(!source.contains("fma("), "{source}");
}

/// Only a `bfloat16` NaN encoding is accepted as a canonical BF16 NaN.
///
/// The predicate is exercised directly and the narrowing beside it, because the
/// scheduled-region verifier already requires a `bf16` region to declare
/// exactly the zero-extended `0x7fc0` — so neither refusal is reachable through
/// a verified kernel, and a test that only drove emission would prove the
/// checks exist without ever making one say no.
#[test]
fn only_a_bf16_nan_encoding_is_accepted_as_a_canonical_bf16_nan() {
    assert!(is_bf16_nan(0x7fc0));
    assert!(is_bf16_nan(0xffc1));
    assert!(is_bf16_nan(0x7f81));
    assert!(!is_bf16_nan(0x0000));
    // Infinity: the exponent field is full and the significand is empty.
    assert!(!is_bf16_nan(0x7f80));
    assert!(!is_bf16_nan(0x3f80));

    // The declared payload the region verifier requires, narrowed.
    assert_eq!(bf16_canonical_nan(bf16_nan_bits()), Ok(0x7fc0));

    // A non-NaN low half is refused rather than canonicalized to a finite
    // value.
    assert_eq!(
        bf16_canonical_nan(0x0000_3f80),
        Err(MetalEmitError::InvalidCanonicalNan { bits: 0x0000_3f80 }),
    );

    // **The binary32 canonical NaN is refused rather than truncated.** Its low
    // half is `0x0000`, so a narrowing that dropped the high bits would have
    // emitted a "canonicalization" to positive zero — the exact wrong-tensor
    // outcome the separate widths exist to prevent, and one that would have
    // compiled.
    assert_eq!(
        bf16_canonical_nan(NAN_BITS),
        Err(MetalEmitError::InvalidCanonicalNan { bits: NAN_BITS }),
    );
    assert_eq!(
        bf16_canonical_nan(NAN_BITS).unwrap_err().rule(),
        "invalid-canonical-nan",
    );
}

/// The BF16 fixture matches its checked-in golden source.
#[test]
fn bf16_pointwise_matches_its_golden_source() {
    let unit = emit_bf16_pointwise_under(bf16_numerical(), APPLE_FLUSH);
    assert_golden(
        "pointwise_scale_bias_bf16.metal",
        include_str!("../goldens/pointwise_scale_bias_bf16.metal"),
        unit.source(),
    );
}

#[test]
fn only_a_nan_encoding_is_accepted_as_a_canonical_nan() {
    assert!(is_f32_nan(0x7fc0_0000));
    assert!(is_f32_nan(0xffc0_0001));
    assert!(!is_f32_nan(0x0000_0000));
    assert!(!is_f32_nan(0x7f80_0000));
    assert!(!is_f32_nan(0x3f80_0000));
}

#[test]
fn a_non_nan_canonical_pattern_is_rejected() {
    // The scheduled-region contract does not require the canonical pattern to
    // be a NaN, so emitting the helper would compile and compute the wrong
    // thing. Emission fails closed instead.
    let region = pointwise_region(RegionId::new(0), &Shape::from_dims([4]), 0);
    let kernel = lower_scheduled_region(&region).unwrap();
    let error = emit_translation_unit(&[&kernel], &target()).unwrap_err();
    assert_eq!(error, MetalEmitError::InvalidCanonicalNan { bits: 0 });
    assert_eq!(error.rule(), "invalid-canonical-nan");
}

#[test]
fn a_signature_exceeding_the_binding_table_is_rejected() {
    let kernel = pointwise_kernel();
    let mut facts = target();
    facts.buffer_binding_limit = 1;
    let error = emit_translation_unit(&[&kernel], &facts).unwrap_err();
    assert_eq!(
        error,
        MetalEmitError::BufferBindingLimit {
            required: 2,
            limit: 1,
        }
    );
    assert_eq!(error.rule(), "buffer-binding-limit");
}

#[test]
fn only_argument_table_address_spaces_become_buffer_parameters() {
    assert_eq!(
        address_space_declaration(AddressSpace::Device, BufferAccess::Read).unwrap(),
        "device const"
    );
    assert_eq!(
        address_space_declaration(AddressSpace::Device, BufferAccess::Write).unwrap(),
        "device"
    );
    assert_eq!(
        address_space_declaration(AddressSpace::Constant, BufferAccess::Read).unwrap(),
        "constant"
    );
    assert_eq!(
        address_space_declaration(AddressSpace::Constant, BufferAccess::Write).unwrap_err(),
        MetalEmitError::UnsupportedBufferAccess {
            space: AddressSpace::Constant,
            access: BufferAccess::Write,
        }
    );
    for space in [AddressSpace::Workgroup, AddressSpace::InvocationPrivate] {
        assert_eq!(
            address_space_declaration(space, BufferAccess::Read).unwrap_err(),
            MetalEmitError::UnsupportedAddressSpace { space }
        );
    }
}

fn barrier(
    execution: ExecutionScope,
    memory: MemoryScope,
    fenced: &[AddressSpace],
) -> Result<String, MetalEmitError> {
    // The schedule point a barrier realizes is a verification reference, not an
    // emission fact: `barrier_call` reads the scopes, fences, and ordering and
    // never the point, which is why every case below fixes it at the first
    // ordinal rather than varying it.
    barrier_call(&BarrierSpec {
        point: SyncPointId::FIRST,
        execution_scope: execution,
        memory_scope: memory,
        fenced_spaces: fenced.to_vec(),
        ordering: BarrierOrdering::AcquireRelease,
    })
}

#[test]
fn a_workgroup_barrier_fences_its_named_spaces_in_governed_order() {
    assert_eq!(
        barrier(
            ExecutionScope::Workgroup,
            MemoryScope::Workgroup,
            &[AddressSpace::Workgroup, AddressSpace::Device],
        )
        .unwrap(),
        "threadgroup_barrier(mem_flags::mem_device | mem_flags::mem_threadgroup);"
    );
    assert_eq!(
        barrier(ExecutionScope::Workgroup, MemoryScope::Workgroup, &[]).unwrap(),
        "threadgroup_barrier(mem_flags::mem_none);"
    );
}

#[test]
fn no_metal_barrier_establishes_device_wide_visibility() {
    assert_eq!(
        barrier(
            ExecutionScope::Workgroup,
            MemoryScope::Device,
            &[AddressSpace::Device],
        )
        .unwrap_err(),
        MetalEmitError::UnsupportedBarrier {
            reason: BarrierRejection::MemoryVisibility {
                execution: ExecutionScope::Workgroup,
                memory: MemoryScope::Device,
            },
        }
    );
}

#[test]
fn a_simd_group_barrier_cannot_claim_workgroup_visibility() {
    // The governed memory scopes cannot name SIMD-group visibility, so no
    // admissible scope exists for a SIMD-group barrier. That is a gap in the
    // portable vocabulary, not a Metal limitation, and it is rejected rather
    // than widened.
    assert_eq!(
        barrier(
            ExecutionScope::Subgroup,
            MemoryScope::Workgroup,
            &[AddressSpace::Workgroup],
        )
        .unwrap_err(),
        MetalEmitError::UnsupportedBarrier {
            reason: BarrierRejection::MemoryVisibility {
                execution: ExecutionScope::Subgroup,
                memory: MemoryScope::Workgroup,
            },
        }
    );
}

#[test]
fn a_space_without_a_fence_flag_is_rejected() {
    for space in [AddressSpace::Constant, AddressSpace::InvocationPrivate] {
        assert_eq!(
            barrier(ExecutionScope::Workgroup, MemoryScope::Workgroup, &[space]).unwrap_err(),
            MetalEmitError::UnsupportedBarrier {
                reason: BarrierRejection::FencedSpace { space },
            }
        );
    }
}

#[test]
fn a_symbol_claimed_by_two_identities_is_rejected() {
    let mut symbols = std::collections::BTreeMap::new();
    reserve_symbol(&mut symbols, "tiler_kernel_0", b"identity-a".as_slice()).unwrap();
    reserve_symbol(&mut symbols, "tiler_kernel_0", b"identity-a".as_slice()).unwrap();
    let error =
        reserve_symbol(&mut symbols, "tiler_kernel_0", b"identity-b".as_slice()).unwrap_err();
    assert_eq!(
        error,
        MetalEmitError::SymbolCollision {
            symbol: "tiler_kernel_0".to_owned(),
        }
    );
    assert_eq!(error.rule(), "symbol-collision");
}

/// A reduction over an empty domain emits, and declares every parameter.
///
/// The case `prototype-metal-runtime-proof` could not cover. Emission refused it
/// with `unreferenced-buffer-parameter` because the argument table was derived
/// from what the body read, and a reduction over zero contributors reads its
/// input never — so the declared input had no position and the count check
/// fired. The table now comes from the declaration, so the parameter is declared
/// and simply not read, which is legal MSL and leaves the ABI as declared.
#[test]
fn an_empty_domain_reduction_emits_every_declared_parameter() {
    let kernel = empty_domain_reduction_kernel();
    let declared = kernel.buffers().len();
    assert_eq!(declared, 2, "the fixture declares an input and an output");

    let unit = emit_translation_unit(&[&kernel], &target()).expect("an empty domain emits");
    unit.require_declared_realization()
        .expect("an empty domain honours the declared contract");

    let entry = &unit.entry_points()[0];
    assert_eq!(
        entry.buffers().len(),
        declared,
        "every declared parameter occupies an argument-table position",
    );
    let source = unit.source();
    assert!(
        source.contains("[[buffer(0)]]") && source.contains("[[buffer(1)]]"),
        "both declared parameters appear in the signature:\n{source}",
    );
}

/// The argument table follows declaration order, not the order the body reads.
///
/// This is the property the empty-domain kernel is the sharpest witness for, and
/// it is a correctness claim rather than a stylistic one. The table used to be
/// built in first-use order while the artifact's own binding table is in
/// declaration order, and the runtime pairs artifact slot *i* with the emitted
/// table's *i*-th transport. For a body that does not touch its buffers in
/// declaration sequence those two disagree, and the disagreement is silent: each
/// side is internally consistent and the wrong buffer is bound.
///
/// Here the input is never read at all, so first-use order would have given the
/// output index 0 — the position the input declares. Asserting the output sits
/// at its declared ordinal is what distinguishes the two schemes.
#[test]
fn the_argument_table_follows_declaration_order() {
    let kernel = empty_domain_reduction_kernel();
    let declared: Vec<_> = kernel.buffers().collect();
    let unit = emit_translation_unit(&[&kernel], &target()).expect("an empty domain emits");
    let emitted = unit.entry_points()[0].buffers();

    for (ordinal, (parameter, binding)) in declared.iter().zip(emitted).enumerate() {
        let ordinal = u32::try_from(ordinal).expect("a bounded parameter count");
        assert_eq!(
            binding.index(),
            ordinal,
            "parameter {ordinal} was emitted at argument-table index {}",
            binding.index(),
        );
        assert_eq!(
            binding.parameter().tensor,
            parameter.tensor,
            "the table's {ordinal}th entry is the {ordinal}th declared parameter",
        );
    }
}

/// The normalization epilogue selects the precise reciprocal square root, by name.
///
/// **The negative half is the load-bearing one, and its hazard is a *different
/// contract* rather than a coarser one.** Under the compiler's own default —
/// which is fast math — an unqualified `rsqrt(x)` selects `air.fast_rsqrt.f32`,
/// whose accuracy is Metal's Table 8.2 `<= 2 ulp` where Table 8.1 states
/// correctly rounded. Writing `precise::rsqrt` selects `air.rsqrt.f32` under both
/// settings, so the flag requirement below is a second line of defence rather
/// than the only one. There is also no `sqrt` in the emission: `1 / sqrt(x)`
/// rounds twice and the retained probe measures the two spellings disagreeing at
/// the `eps` argument this workload's zero and subnormal rows reach.
#[test]
fn the_normalization_epilogue_emits_the_precise_reciprocal_square_root() {
    let kernel = rms_norm_epilogue_kernel();
    let unit =
        emit_translation_unit(&[&kernel], &target()).expect("the normalization fixture emits");
    let source = unit.source();
    assert_eq!(
        source.matches("precise::rsqrt(").count(),
        1,
        "the subordinate reciprocal square root is emitted once, in the precise namespace:\n{source}"
    );
    for forbidden in [
        "fast::rsqrt",
        "fast_rsqrt",
        "sqrt(",
        "1.0f /",
        "metal::divide(",
    ] {
        if forbidden == "sqrt(" {
            // `precise::rsqrt(` contains `sqrt(`, so the check is that no *other*
            // occurrence exists rather than that the substring is absent.
            assert_eq!(
                source.matches("sqrt(").count(),
                source.matches("precise::rsqrt(").count(),
                "every sqrt occurrence belongs to the reciprocal square root:\n{source}"
            );
            continue;
        }
        assert!(
            !source.contains(forbidden),
            "{forbidden} must not appear; it is a different contract:\n{source}"
        );
    }
    // The governed eps payload reaches the emitted source as its exact bits
    // rather than a decimal literal someone rounded on the way.
    assert!(
        source.contains(&format!("0x{RMS_NORM_F32_REFERENCE_EPS_BITS:08x}"))
            || source.contains(&format!(
                "{}",
                f32::from_bits(RMS_NORM_F32_REFERENCE_EPS_BITS)
            )),
        "the eps constant is emitted:\n{source}"
    );
}

/// The normalization epilogue requires both governed math flags.
#[test]
fn the_normalization_epilogue_requires_the_precise_and_safe_selections() {
    let kernel = rms_norm_epilogue_kernel();
    let unit =
        emit_translation_unit(&[&kernel], &target()).expect("the normalization fixture emits");
    let requirements = unit.numerical_requirements();
    assert!(requirements.contains(&MetalNumericalRequirement::PreciseFp32Functions));
    assert!(requirements.contains(&MetalNumericalRequirement::SafeMathMode));

    // The scale-then-bias fixture emits no elementary function, so it requires
    // no precise selection. Without this the assertion above would pass for a
    // requirement that had simply been added to every unit.
    let pointwise = pointwise_kernel();
    let plain = emit_translation_unit(&[&pointwise], &target()).expect("emits");
    assert!(
        !plain
            .numerical_requirements()
            .contains(&MetalNumericalRequirement::PreciseFp32Functions),
        "a kernel with no elementary function does not demand the precise selection"
    );
}

/// The squaring-prologue sum multiplies each contributor by itself, once.
///
/// One product per contributor and no constant beside it, which is what
/// distinguishes it from the scale-then-bias prologue emitted by the fused sum:
/// that one multiplies by a scale and adds a bias, two roundings, and this one
/// squares, one rounding. The two fixtures are emitted side by side so the
/// difference is asserted rather than described.
#[test]
fn the_squaring_prologue_sum_squares_each_contributor_once() {
    let squared = emit_one(&squared_reduction_kernel());
    let fused = emit_one(&fused_reduction_kernel());

    // The seed and the loop body each square, so the squaring appears twice in a
    // fixture whose reduced extent is three — and every one of the emitted
    // `float` products is a value multiplied by *itself*, from one load rather
    // than from two reads agreeing.
    let self_products = |source: &str| {
        source
            .lines()
            .filter_map(|line| {
                let body = line.trim().strip_prefix("float ")?;
                let (_, expression) = body.split_once(" = ")?;
                let (lhs, rhs) = expression.trim_end_matches(';').split_once(" * ")?;
                Some(lhs == rhs)
            })
            .collect::<Vec<bool>>()
    };
    let products = self_products(&squared);
    assert_eq!(
        products.len(),
        2,
        "one float product per contributor position, seed and loop body:\n{squared}"
    );
    assert!(
        products.iter().all(|same| *same),
        "every product squares one loaded value:\n{squared}"
    );

    // The scale-bias prologue's products are *not* self-products, so the
    // assertion above distinguishes the two programs rather than holding for any
    // reduction the emitter produces.
    let fused_products = self_products(&fused);
    assert!(!fused_products.is_empty());
    assert!(
        fused_products.iter().all(|same| !*same),
        "the scale-bias prologue multiplies by a constant, not by itself:\n{fused}"
    );
    assert_ne!(squared, fused);
}

/// The two prologue-carrying reductions do not share a kernel identity.
///
/// They read the same tensor with the same access relation over the same
/// contributor order, so nothing but the scalar program distinguishes them — and
/// an appended scalar-program tag that had collided with an existing one would
/// make these equal.
#[test]
fn the_squaring_and_scale_bias_reductions_carry_different_identities() {
    let squared = squared_reduction_kernel();
    let fused = fused_reduction_kernel();
    let bare = single_axis_reduction_kernel();
    assert_ne!(squared.canonical_identity(), fused.canonical_identity());
    assert_ne!(squared.canonical_identity(), bare.canonical_identity());
}

/// The extrema fold emits an exact fixup, and never `fmax`.
///
/// **The negative half is the whole test.** ADR 0023 admits two extrema families
/// and Metal's `fmax` implements neither: it prefers numbers, so it is not the
/// propagating family, and its signed-zero result can depend on operand order, so
/// it is not the deterministic number-preferring one either. Selecting it would
/// be the substitution the ADR forbids, and the substitution is *available* —
/// `fmax(a, b)` compiles and the retained emission probe measures which intrinsic
/// it selects — which is why the absence is asserted rather than assumed.
///
/// The positive half pins the three clauses the fixup needs to be exact: the two
/// ordered comparisons, the bitwise `and` that orders `-0.0` below `+0.0`, and
/// the canonical NaN the unordered case returns.
#[test]
fn the_extrema_fold_emits_an_exact_fixup_rather_than_fmax() {
    let kernel = maximum_reduction_kernel();
    let unit = emit_translation_unit(&[&kernel], &target()).expect("the extrema fixture emits");
    let source = unit.source();

    // No spelling of the intrinsic, precise or fast, qualified or not. The
    // *call* spellings rather than the bare words, because the helper's own
    // comment names `fmax` in order to say it is not being used — and a check
    // that could not tell the two apart would forbid explaining the decision.
    for forbidden in [
        "fmax(",
        "fmin(",
        "max(",
        "min(",
        "metal::max(",
        "precise::fmax(",
        "fast::fmax(",
    ] {
        assert!(
            !source.contains(forbidden),
            "{forbidden} must not appear; it implements neither admitted extrema family:\n{source}"
        );
    }
    // And the word does appear, exactly once, in the sentence that says why.
    assert_eq!(
        source.matches("Deliberately not fmax").count(),
        1,
        "the emitted text states why the intrinsic was not selected:\n{source}"
    );

    // The helper is defined once and called once, so the fold goes through it
    // rather than through an inlined approximation of it.
    assert_eq!(
        source
            .matches("static inline float tiler_maximum_f32(")
            .count(),
        1,
        "the fixup is defined exactly once:\n{source}"
    );
    assert_eq!(
        source.matches("tiler_maximum_f32(").count(),
        2,
        "one definition and one call site:\n{source}"
    );

    // The three clauses, each of which the fixup would be wrong without.
    assert!(
        source.contains("if (left < right) { return right; }"),
        "the ordered clause is emitted:\n{source}"
    );
    assert!(
        source.contains("as_type<uint>(left) & as_type<uint>(right)"),
        "the signed-zero clause orders -0.0 below +0.0 by clearing the sign bit:\n{source}"
    );
    assert!(
        source.contains("return as_type<float>(0x7fc00000u);"),
        "the unordered clause propagates a NaN rather than preferring a number:\n{source}"
    );
}

/// The extrema fold requires the safe math mode and *not* the precise selection.
///
/// Two claims, and the second is the one worth asserting: the fixup calls no F32
/// math function at all, so `-fmetal-math-fp32-functions` governs nothing in it
/// and demanding that flag would be a requirement nothing in the emitted text
/// needs. The safe mode is separately load-bearing, because `nnan` licenses
/// folding the unordered comparison the NaN clause rests on.
#[test]
fn the_extrema_fold_requires_the_safe_mode_and_not_the_precise_selection() {
    let kernel = maximum_reduction_kernel();
    let unit = emit_translation_unit(&[&kernel], &target()).expect("the extrema fixture emits");
    let requirements = unit.numerical_requirements();
    assert!(requirements.contains(&MetalNumericalRequirement::SafeMathMode));
    assert!(
        !requirements.contains(&MetalNumericalRequirement::PreciseFp32Functions),
        "the fixup calls no F32 math function, so it demands no precise selection"
    );

    // The control: the normalization epilogue *does* demand it, so the absence
    // above is a property of this kernel rather than of the requirement.
    let epilogue = rms_norm_epilogue_kernel();
    let precise = emit_translation_unit(&[&epilogue], &target()).expect("emits");
    assert!(
        precise
            .numerical_requirements()
            .contains(&MetalNumericalRequirement::PreciseFp32Functions)
    );
}

/// The extrema fold commits no empty-domain identity, because it has none.
///
/// Every sum in the vocabulary writes a declared identity when its reduced domain
/// is empty; the extrema family has no identity, so the schedule verifier refuses
/// the region before a kernel can exist. This asserts that refusal at the layer
/// that owns it, with the bare sum over the same shape as the control — so the
/// refusal is about the family rather than about the zero extent.
#[test]
fn an_empty_extrema_domain_is_refused_where_an_empty_sum_commits_its_identity() {
    let empty = Shape::from_dims([2, 0]);
    let axes = [Axis::new(1)];

    // The control: the bare sum over the same empty domain builds and emits.
    let sum = reduction_region(RegionId::new(25), &empty, &axes, FixtureReduction::BareSum);
    assert!(lower_scheduled_region(&sum).is_ok());

    // The extrema fold over the same shape does not even reach a verified region.
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(26));
    builder.iteration_shape(Shape::from_dims([2])).unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::ReductionContributor {
                input_shape: empty.clone(),
                output_shape: Shape::from_dims([2]),
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
            bounds: BoundsWitnessId::new(0),
            ownership: None,
        })
        .unwrap();
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(1),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Input {
                ordinal: InputOrdinal::FIRST,
            },
            component_role: None,
            kind: BoundsProofKind::ReductionDomain {
                input_shape: empty,
                output_shape: Shape::from_dims([2]),
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
            },
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange { element_count: 2 },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput { output_count: 2 },
        })
        .unwrap();
    builder
        .scalar_program(ScalarProgram::StrictSerialMaximum {
            axes: axes.to_vec(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: NAN_BITS,
        })
        .unwrap();
    builder.numerical(numerical(NAN_BITS)).unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::Serial {
                axes: axes.to_vec(),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(2)
        })
        .unwrap();
    assert!(
        builder.build().is_err(),
        "an identity-less fold over an empty reduced domain has no value to commit"
    );
}

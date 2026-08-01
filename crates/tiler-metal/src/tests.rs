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
    AddressSpace, BarrierOrdering, BarrierSpec, BufferAccess, ExecutionScope, KernelType,
    MemoryScope, VerifiedKernel, lower_scheduled_region,
};
use tiler_ir::schedule::{
    Access, AccessMode, ArithmeticType, BoundsProof, BoundsProofKind, BoundsWitnessId,
    ContributorOrder, ExceptionalValueAssumption, ExecutionBinding, FlushedZeroSign, InputOrdinal,
    KernelSchedule, LaunchPlan, LogicalAccess, NumericalPermission, NumericalRealization,
    OwnershipProof, OwnershipProofKind, OwnershipWitnessId, PointwiseF32Expression,
    PointwiseF32ExpressionBuilder, ReductionTopology, RegionId, ScalarProgram,
    ScheduledRegionBuilder, SubnormalFreedom, SubnormalMode, TailPolicy, TensorRole,
    ValueDomainProvenance, VerifiedScheduledRegion, element_count,
};
use tiler_ir::semantic::{
    STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE,
};
use tiler_ir::shape::{Axis, Shape};

use crate::diagnostic::{BarrierRejection, MetalEmitError};
use crate::emit::{
    address_space_declaration, barrier_call, emit_translation_unit as emit_with_realization,
    is_f32_nan, msl_type, realization_requirements, reserve_symbol,
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

/// A pointwise scale-then-bias region carrying one stated declared realization.
fn pointwise_region_under(
    id: RegionId,
    shape: &Shape,
    realization: NumericalRealization,
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
    builder
        .scalar_program(ScalarProgram::PointwiseF32(scale_then_bias_expression(
            SCALE_BITS, BIAS_BITS,
        )))
        .unwrap();
    builder.numerical(realization).unwrap();
    builder.schedule(linear_schedule(elements)).unwrap();
    builder.build().unwrap()
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

/// A serial reduction region over `axes` of `input`, optionally fusing a
/// scale-then-bias prologue into every contributor.
fn reduction_region(
    id: RegionId,
    input: &Shape,
    axes: &[Axis],
    fused: bool,
) -> VerifiedScheduledRegion {
    let output = input.without_axes(axes);
    let output_elements = element_count(&output).expect("bounded fixture shape");
    let read_tensor = if fused {
        TensorRole::Input {
            ordinal: InputOrdinal::FIRST,
        }
    } else {
        TensorRole::Intermediate
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
    let scalar = if fused {
        ScalarProgram::FusedMultiplyAddSerialSum {
            scale_bits: SCALE_BITS,
            bias_bits: BIAS_BITS,
            axes: axes.to_vec(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: NAN_BITS,
            empty_identity_bits: 0.0_f32.to_bits(),
            contraction: false,
        }
    } else {
        ScalarProgram::StrictSerialSum {
            axes: axes.to_vec(),
            order: ContributorOrder::OriginalAxisLexicographic,
            canonical_nan_bits: NAN_BITS,
            empty_identity_bits: 0.0_f32.to_bits(),
        }
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
        false,
    ))
    .expect("bounded reduction fixture lowers")
}

pub(crate) fn multi_axis_reduction_kernel() -> VerifiedKernel {
    lower_scheduled_region(&reduction_region(
        RegionId::new(2),
        &Shape::from_dims([2, 3, 4]),
        &[Axis::new(1), Axis::new(2)],
        false,
    ))
    .expect("bounded multi-axis reduction fixture lowers")
}

pub(crate) fn fused_reduction_kernel() -> VerifiedKernel {
    lower_scheduled_region(&reduction_region(
        RegionId::new(3),
        &Shape::from_dims([2, 3]),
        &[Axis::new(1)],
        true,
    ))
    .expect("bounded fused reduction fixture lowers")
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
        false,
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
    assert_eq!(msl_type(KernelType::Bool), "bool");
    assert_eq!(msl_type(KernelType::U8), "uchar");
    assert_eq!(msl_type(KernelType::I32), "int");
    assert_eq!(msl_type(KernelType::Index), "uint64_t");
    assert_eq!(msl_type(KernelType::F32), "float");
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
    barrier_call(&BarrierSpec {
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

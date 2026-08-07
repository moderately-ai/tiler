//! Typed facades for Tiler's governed initial operation profile.

use crate::shape::{Axis, ShapeEvidence, StaticShape};

use super::{
    BF16_CONSTANT_BITS_ATTRIBUTE, BROADCAST_AXIS_MAPPING_ATTRIBUTE, Bf16, BroadcastAxisMapping,
    BuildError, CONCATENATE_AXIS_ATTRIBUTE, CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, CanonicalField,
    CanonicalValue, ContractionIndexStructure, F32, F32_CONSTANT_BITS_ATTRIBUTE,
    OperationAttributes, REDUCTION_AXES_ATTRIBUTE, REINDEX_MAPPING_ATTRIBUTE, ReindexForm,
    SLICE_SELECTION_ATTRIBUTE, SemanticProgramBuilder, ShapedValue, SliceSelection, Value, ValueId,
    add_bf16_op, add_f32_op, broadcast_f32_op, canonical_bf16_bits, concatenate_f32_axis_attribute,
    concatenate_f32_op, constant_bf16_op, constant_f32_op, multiply_bf16_op, multiply_f32_op,
    reindex_f32_op, rms_norm_f32_axis_attribute, rms_norm_f32_eps_attribute, rms_norm_f32_op,
    silu_f32_op, slice_f32_op, softmax_f32_axis_attribute, softmax_f32_op,
    strict_serial_sum_f32_op, strict_tensor_contraction_f32_op,
};

/// Exact binary32 constant from its IEEE-754 payload.
#[derive(Clone, Copy, Debug, Default)]
pub struct F32Constant;

impl F32Constant {
    /// Applies the registered scalar constant semantics.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on
    /// failure.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        bits: u32,
    ) -> Result<Value<F32>, BuildError> {
        let attributes = OperationAttributes::new([CanonicalField::new(
            F32_CONSTANT_BITS_ATTRIBUTE,
            canonical_f32_bits(bits)?,
        )])
        .map_err(BuildError::InvalidOperationAttributes)?;
        apply_single(builder, constant_f32_op(), attributes, &[])
    }

    /// Applies the scalar constant semantics and preserves its exact shape.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on
    /// failure.
    pub fn apply_shaped(
        builder: &mut SemanticProgramBuilder,
        bits: u32,
    ) -> Result<ShapedValue<F32, StaticShape<0, { [] }>>, BuildError> {
        let attributes = constant_attributes(bits)?;
        apply_shaped_single(builder, constant_f32_op(), attributes, &[])
    }
}

/// Separate binary32 multiplication with scalar broadcast.
#[derive(Clone, Copy, Debug, Default)]
pub struct F32Multiply;

impl F32Multiply {
    /// Applies the registered multiplication semantics.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on
    /// failure.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        left: Value<F32>,
        right: Value<F32>,
    ) -> Result<Value<F32>, BuildError> {
        apply_single(
            builder,
            multiply_f32_op(),
            OperationAttributes::empty(),
            &[left.erase(), right.erase()],
        )
    }

    /// Applies multiplication through the canonical path and rechecks the
    /// shared operand evidence on its result.
    ///
    /// # Errors
    ///
    /// Returns a typed construction or shape-refinement error.
    pub fn apply_shaped<E: ShapeEvidence>(
        builder: &mut SemanticProgramBuilder,
        left: ShapedValue<F32, E>,
        right: ShapedValue<F32, E>,
    ) -> Result<ShapedValue<F32, E>, BuildError> {
        apply_shaped_single(
            builder,
            multiply_f32_op(),
            OperationAttributes::empty(),
            &[left.weaken().erase(), right.weaken().erase()],
        )
    }

    /// Multiplies a scalar left operand by a shaped right operand and
    /// preserves the right operand's evidence.
    ///
    /// # Errors
    ///
    /// Returns a typed construction or shape-refinement error.
    pub fn apply_scalar_left<E: ShapeEvidence>(
        builder: &mut SemanticProgramBuilder,
        left: ShapedValue<F32, StaticShape<0, { [] }>>,
        right: ShapedValue<F32, E>,
    ) -> Result<ShapedValue<F32, E>, BuildError> {
        apply_shaped_single(
            builder,
            multiply_f32_op(),
            OperationAttributes::empty(),
            &[left.weaken().erase(), right.weaken().erase()],
        )
    }

    /// Multiplies a shaped left operand by a scalar right operand and
    /// preserves the left operand's evidence.
    ///
    /// # Errors
    ///
    /// Returns a typed construction or shape-refinement error.
    pub fn apply_scalar_right<E: ShapeEvidence>(
        builder: &mut SemanticProgramBuilder,
        left: ShapedValue<F32, E>,
        right: ShapedValue<F32, StaticShape<0, { [] }>>,
    ) -> Result<ShapedValue<F32, E>, BuildError> {
        apply_shaped_single(
            builder,
            multiply_f32_op(),
            OperationAttributes::empty(),
            &[left.weaken().erase(), right.weaken().erase()],
        )
    }
}

/// Separate binary32 addition with scalar broadcast.
#[derive(Clone, Copy, Debug, Default)]
pub struct F32Add;

impl F32Add {
    /// Applies the registered addition semantics.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on
    /// failure.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        left: Value<F32>,
        right: Value<F32>,
    ) -> Result<Value<F32>, BuildError> {
        apply_single(
            builder,
            add_f32_op(),
            OperationAttributes::empty(),
            &[left.erase(), right.erase()],
        )
    }

    /// Applies addition through the canonical path and rechecks the shared
    /// operand evidence on its result.
    ///
    /// # Errors
    ///
    /// Returns a typed construction or shape-refinement error.
    pub fn apply_shaped<E: ShapeEvidence>(
        builder: &mut SemanticProgramBuilder,
        left: ShapedValue<F32, E>,
        right: ShapedValue<F32, E>,
    ) -> Result<ShapedValue<F32, E>, BuildError> {
        apply_shaped_single(
            builder,
            add_f32_op(),
            OperationAttributes::empty(),
            &[left.weaken().erase(), right.weaken().erase()],
        )
    }

    /// Adds a scalar left operand to a shaped right operand and preserves the
    /// right operand's evidence.
    ///
    /// # Errors
    ///
    /// Returns a typed construction or shape-refinement error.
    pub fn apply_scalar_left<E: ShapeEvidence>(
        builder: &mut SemanticProgramBuilder,
        left: ShapedValue<F32, StaticShape<0, { [] }>>,
        right: ShapedValue<F32, E>,
    ) -> Result<ShapedValue<F32, E>, BuildError> {
        apply_shaped_single(
            builder,
            add_f32_op(),
            OperationAttributes::empty(),
            &[left.weaken().erase(), right.weaken().erase()],
        )
    }

    /// Adds a shaped left operand to a scalar right operand and preserves the
    /// left operand's evidence.
    ///
    /// # Errors
    ///
    /// Returns a typed construction or shape-refinement error.
    pub fn apply_scalar_right<E: ShapeEvidence>(
        builder: &mut SemanticProgramBuilder,
        left: ShapedValue<F32, E>,
        right: ShapedValue<F32, StaticShape<0, { [] }>>,
    ) -> Result<ShapedValue<F32, E>, BuildError> {
        apply_shaped_single(
            builder,
            add_f32_op(),
            OperationAttributes::empty(),
            &[left.weaken().erase(), right.weaken().erase()],
        )
    }
}

/// Strict serial binary32 Sum over canonical logical axes.
#[derive(Clone, Copy, Debug, Default)]
pub struct StrictSerialF32Sum;

impl StrictSerialF32Sum {
    /// Applies the registered strict serial Sum semantics.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on
    /// failure.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        input: Value<F32>,
        axes: impl IntoIterator<Item = Axis>,
    ) -> Result<Value<F32>, BuildError> {
        let axes = CanonicalValue::sequence(
            axes.into_iter()
                .map(|axis| CanonicalValue::unsigned_u32(axis.get())),
        )
        .map_err(BuildError::InvalidOperationAttributes)?;
        let attributes =
            OperationAttributes::new([CanonicalField::new(REDUCTION_AXES_ATTRIBUTE, axes)])
                .map_err(BuildError::InvalidOperationAttributes)?;
        apply_single(
            builder,
            strict_serial_sum_f32_op(),
            attributes,
            &[input.erase()],
        )
    }
}

/// Strict binary32 tensor contraction over a canonical index structure.
///
/// The *tensor* sense of contraction: a sum over indices shared by both
/// operands. It is unrelated to ADR 0015's contraction, the permission to fuse a
/// multiply and an add into one rounding, which this family forbids.
#[derive(Clone, Copy, Debug, Default)]
pub struct F32TensorContraction;

impl F32TensorContraction {
    /// Applies the registered tensor-contraction semantics.
    ///
    /// The structure is stated once and validated once, by the registered
    /// operation authority; a frontend never chooses among contraction keys
    /// because there is only one. The result's shape is derived from the
    /// structure's output tuple and the operands' extents, never declared here.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on
    /// failure, including the named structural rule the structure violated
    /// against this occurrence's operands.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        structure: &ContractionIndexStructure,
        left: Value<F32>,
        right: Value<F32>,
    ) -> Result<Value<F32>, BuildError> {
        let attributes = OperationAttributes::new([CanonicalField::new(
            CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE,
            structure.canonical_value().clone(),
        )])
        .map_err(BuildError::InvalidOperationAttributes)?;
        apply_single(
            builder,
            strict_tensor_contraction_f32_op(),
            attributes,
            &[left.erase(), right.erase()],
        )
    }
}

/// Binary32 `Reindex` over one admitted coordinate mapping form.
///
/// A reindex changes which coordinate reads which element and changes no value.
/// It makes no claim that storage was transposed or copied; whether an
/// occurrence costs a dispatch is a planning outcome.
#[derive(Clone, Copy, Debug, Default)]
pub struct F32Reindex;

impl F32Reindex {
    /// Applies the registered reindex semantics.
    ///
    /// The form is stated once and validated once, by the registered operation
    /// authority: a form outside the admitted set, and a form the operand's
    /// shape does not admit, are both refused here rather than approximated. The
    /// result's shape is derived from the form and the operand's extents, never
    /// declared by a caller.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on
    /// failure, naming the mapping rule the form violated against this
    /// occurrence's operand.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        form: &ReindexForm,
        input: Value<F32>,
    ) -> Result<Value<F32>, BuildError> {
        let attributes = OperationAttributes::new([CanonicalField::new(
            REINDEX_MAPPING_ATTRIBUTE,
            form.canonical_value().clone(),
        )])
        .map_err(BuildError::InvalidOperationAttributes)?;
        apply_single(builder, reindex_f32_op(), attributes, &[input.erase()])
    }
}

/// Binary32 `Broadcast` over one explicit axis mapping.
///
/// Every result axis is accounted for and every many-to-one relation is stated,
/// so nothing about the replication is inferred from a shape.
#[derive(Clone, Copy, Debug, Default)]
pub struct F32Broadcast;

impl F32Broadcast {
    /// Applies the registered broadcast semantics.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on
    /// failure, naming the mapping rule the axis mapping violated against this
    /// occurrence's operand.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        mapping: &BroadcastAxisMapping,
        input: Value<F32>,
    ) -> Result<Value<F32>, BuildError> {
        let attributes = OperationAttributes::new([CanonicalField::new(
            BROADCAST_AXIS_MAPPING_ATTRIBUTE,
            mapping.canonical_value().clone(),
        )])
        .map_err(BuildError::InvalidOperationAttributes)?;
        apply_single(builder, broadcast_f32_op(), attributes, &[input.erase()])
    }
}

/// Binary32 `Concatenate` along one named axis.
///
/// Operand order is semantic — the result's coordinates on the concatenated axis
/// run through the operands in the order given — so this facade takes a slice
/// rather than a set, and reordering it states a different computation.
#[derive(Clone, Copy, Debug, Default)]
pub struct F32Concatenate;

impl F32Concatenate {
    /// Applies the registered concatenation semantics.
    ///
    /// The axis is stated once and validated once, by the registered operation
    /// authority. The result's shape is derived from the axis and the operands'
    /// extents, never declared by a caller: the extent on the concatenated axis is
    /// the exact sum of theirs, and an occurrence whose sum leaves the extent
    /// domain is refused rather than saturated, because this profile can state no
    /// additive extent relation to tie a bound extent back to its operands.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on failure,
    /// naming the rule the occurrence violated — an inadmissible operand count, a
    /// rank disagreement, an axis outside the operands' rank, an extent
    /// disagreement on an axis other than the concatenated one (naming both
    /// observed extents), an operand that is not `tiler::f32@1`, or an
    /// unrelatable result extent.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        inputs: &[Value<F32>],
        axis: Axis,
    ) -> Result<Value<F32>, BuildError> {
        let attributes = OperationAttributes::new([CanonicalField::new(
            CONCATENATE_AXIS_ATTRIBUTE,
            concatenate_f32_axis_attribute(axis),
        )])
        .map_err(BuildError::InvalidOperationAttributes)?;
        let operands: Vec<ValueId> = inputs.iter().map(|input| input.erase()).collect();
        apply_single(builder, concatenate_f32_op(), attributes, &operands)
    }
}

/// Binary32 `Slice` over one total per-axis selection.
///
/// A selection reads a rectangular sub-region and changes no value. It makes no
/// claim that storage was copied or viewed; whether an occurrence costs a
/// dispatch, an offset in a consumer's access map, or nothing at all is a
/// planning outcome.
#[derive(Clone, Copy, Debug, Default)]
pub struct F32Slice;

impl F32Slice {
    /// Applies the registered selection semantics.
    ///
    /// The selection is stated once and validated once, by the registered
    /// operation authority: a reserved or unadmitted relation, a selection whose
    /// entries are not the operand's axes, and a window that leaves its axis are
    /// all refused here rather than clamped or approximated. The result's shape is
    /// derived from the selection and the operand's extents, never declared by a
    /// caller, and rank is preserved.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on failure,
    /// naming the rule the occurrence violated against this operand.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        selection: &SliceSelection,
        input: Value<F32>,
    ) -> Result<Value<F32>, BuildError> {
        let attributes = OperationAttributes::new([CanonicalField::new(
            SLICE_SELECTION_ATTRIBUTE,
            selection.canonical_value().clone(),
        )])
        .map_err(BuildError::InvalidOperationAttributes)?;
        apply_single(builder, slice_f32_op(), attributes, &[input.erase()])
    }
}

/// The binary32 `SiLU` activation, `y = x / (1 + Exp(-x))`.
///
/// One atomic operation rather than a composition of an exponential, an addition,
/// and a division: none of those three is a registered semantic key, and the
/// activation's resolved accuracy contract for its subordinate exponential is part
/// of *this* key's identity. A caller that wants the sigmoid-product spelling is
/// asking for a different binary32 function and does not get it from here.
#[derive(Clone, Copy, Debug, Default)]
pub struct F32Silu;

impl F32Silu {
    /// Applies the registered `SiLU` semantics.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on failure,
    /// including the named refusal when the operand is not `tiler::f32@1`.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        input: Value<F32>,
    ) -> Result<Value<F32>, BuildError> {
        apply_single(
            builder,
            silu_f32_op(),
            OperationAttributes::empty(),
            &[input.erase()],
        )
    }

    /// Applies `SiLU` through the canonical path and rechecks the operand evidence
    /// on its result.
    ///
    /// The activation is elementwise, so the result carries the operand's own
    /// shape evidence unchanged.
    ///
    /// # Errors
    ///
    /// Returns a typed construction or shape-refinement error.
    pub fn apply_shaped<E: ShapeEvidence>(
        builder: &mut SemanticProgramBuilder,
        input: ShapedValue<F32, E>,
    ) -> Result<ShapedValue<F32, E>, BuildError> {
        apply_shaped_single(
            builder,
            silu_f32_op(),
            OperationAttributes::empty(),
            &[input.weaken().erase()],
        )
    }
}

/// Binary32 root-mean-square normalization over one named axis.
///
/// Two operands and never one: the weight arrives already shaped like the value,
/// because the graph admits no implicit broadcasting and a per-channel weight is
/// widened by a `tiler::broadcast-f32@1` occurrence the caller writes. Passing
/// the narrow weight here is a typed refusal rather than a convenience.
///
/// `eps` is taken as an exact binary32 payload rather than an `f32`, so a caller
/// cannot reach the attribute through a decimal literal whose rounding it never
/// inspected. The pinned workload's constant is
/// [`RMS_NORM_F32_REFERENCE_EPS_BITS`](super::RMS_NORM_F32_REFERENCE_EPS_BITS).
#[derive(Clone, Copy, Debug, Default)]
pub struct F32RmsNorm;

impl F32RmsNorm {
    /// Applies the registered RMS normalization semantics.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on failure,
    /// including the named refusals for an absent, duplicated, multi-valued, or
    /// out-of-range axis, for a weight whose shape differs from the value's, and
    /// for a non-positive or non-finite `eps`.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        value: Value<F32>,
        weight: Value<F32>,
        axis: Axis,
        eps_bits: u32,
    ) -> Result<Value<F32>, BuildError> {
        let attributes = OperationAttributes::new([
            CanonicalField::new(
                super::RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                rms_norm_f32_axis_attribute(axis),
            ),
            CanonicalField::new(
                super::RMS_NORM_EPS_BITS_ATTRIBUTE,
                rms_norm_f32_eps_attribute(eps_bits),
            ),
        ])
        .map_err(BuildError::InvalidOperationAttributes)?;
        apply_single(
            builder,
            rms_norm_f32_op(),
            attributes,
            &[value.erase(), weight.erase()],
        )
    }
}

/// Binary32 softmax over one named axis.
///
/// One operand and never two: the causal mask is added *upstream* by a
/// `tiler::add-f32@1` occurrence over a broadcast F32 mask input, so what reaches
/// this key is already the shifted score tensor. That is why decision **D-1** is
/// not a case this operation can recognize — by the time the scores arrive, a
/// masked position is an ordinary very negative number.
///
/// Shape-preserving, so the result carries the operand's shape unchanged and a
/// zero-length reduced axis produces a zero-length result.
#[derive(Clone, Copy, Debug, Default)]
pub struct F32Softmax;

impl F32Softmax {
    /// Applies the registered softmax semantics.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on failure,
    /// including the named refusals for an absent, duplicated, multi-valued, or
    /// out-of-range reduced axis.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        scores: Value<F32>,
        axis: Axis,
    ) -> Result<Value<F32>, BuildError> {
        let attributes = OperationAttributes::new([CanonicalField::new(
            super::SOFTMAX_REDUCED_AXES_ATTRIBUTE,
            softmax_f32_axis_attribute(axis),
        )])
        .map_err(BuildError::InvalidOperationAttributes)?;
        apply_single(builder, softmax_f32_op(), attributes, &[scores.erase()])
    }

    /// Applies the softmax through the canonical path and rechecks the operand
    /// evidence on its result.
    ///
    /// The operation is shape-preserving, so the result carries the operand's own
    /// shape evidence unchanged — which is exactly what distinguishes it from a
    /// reduction over the same axis.
    ///
    /// # Errors
    ///
    /// Returns a typed construction or shape-refinement error.
    pub fn apply_shaped<E: ShapeEvidence>(
        builder: &mut SemanticProgramBuilder,
        scores: ShapedValue<F32, E>,
        axis: Axis,
    ) -> Result<ShapedValue<F32, E>, BuildError> {
        let attributes = OperationAttributes::new([CanonicalField::new(
            super::SOFTMAX_REDUCED_AXES_ATTRIBUTE,
            softmax_f32_axis_attribute(axis),
        )])
        .map_err(BuildError::InvalidOperationAttributes)?;
        apply_shaped_single(
            builder,
            softmax_f32_op(),
            attributes,
            &[scores.weaken().erase()],
        )
    }
}

/// Exact BF16 constant from its payload in the ratified RISC-V BF16 operand format.
///
/// Not a peer that widens [`F32Constant`]: the two build different operations
/// whose payload formats are separate identities, and neither accepts the
/// other's bits.
#[derive(Clone, Copy, Debug, Default)]
pub struct Bf16Constant;

impl Bf16Constant {
    /// Applies the registered BF16 scalar constant semantics.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on
    /// failure.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        bits: u16,
    ) -> Result<Value<Bf16>, BuildError> {
        builder.apply_typed_single(constant_bf16_op(), bf16_constant_attributes(bits)?, &[])
    }

    /// Applies the BF16 scalar constant semantics and preserves its exact shape.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on
    /// failure.
    pub fn apply_shaped(
        builder: &mut SemanticProgramBuilder,
        bits: u16,
    ) -> Result<ShapedValue<Bf16, StaticShape<0, { [] }>>, BuildError> {
        builder.apply_shaped_single(constant_bf16_op(), bf16_constant_attributes(bits)?, &[])
    }
}

/// Separate BF16 multiplication with scalar broadcast.
///
/// Separate in the ADR 0015 sense: this multiply is not fusable with an adjacent
/// add, and no fused BF16 primitive exists to fuse it into.
#[derive(Clone, Copy, Debug, Default)]
pub struct Bf16Multiply;

impl Bf16Multiply {
    /// Applies the registered BF16 multiplication semantics.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on
    /// failure, including the named refusal when an operand is not BF16.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        left: Value<Bf16>,
        right: Value<Bf16>,
    ) -> Result<Value<Bf16>, BuildError> {
        builder.apply_typed_single(
            multiply_bf16_op(),
            OperationAttributes::empty(),
            &[left.erase(), right.erase()],
        )
    }

    /// Applies BF16 multiplication and rechecks the shared operand evidence on
    /// its result.
    ///
    /// # Errors
    ///
    /// Returns a typed construction or shape-refinement error.
    pub fn apply_shaped<E: ShapeEvidence>(
        builder: &mut SemanticProgramBuilder,
        left: ShapedValue<Bf16, E>,
        right: ShapedValue<Bf16, E>,
    ) -> Result<ShapedValue<Bf16, E>, BuildError> {
        builder.apply_shaped_single(
            multiply_bf16_op(),
            OperationAttributes::empty(),
            &[left.weaken().erase(), right.weaken().erase()],
        )
    }
}

/// Separate BF16 addition with scalar broadcast.
#[derive(Clone, Copy, Debug, Default)]
pub struct Bf16Add;

impl Bf16Add {
    /// Applies the registered BF16 addition semantics.
    ///
    /// # Errors
    ///
    /// Returns a typed construction error without mutating the graph on
    /// failure, including the named refusal when an operand is not BF16.
    pub fn apply(
        builder: &mut SemanticProgramBuilder,
        left: Value<Bf16>,
        right: Value<Bf16>,
    ) -> Result<Value<Bf16>, BuildError> {
        builder.apply_typed_single(
            add_bf16_op(),
            OperationAttributes::empty(),
            &[left.erase(), right.erase()],
        )
    }

    /// Applies BF16 addition and rechecks the shared operand evidence on its
    /// result.
    ///
    /// # Errors
    ///
    /// Returns a typed construction or shape-refinement error.
    pub fn apply_shaped<E: ShapeEvidence>(
        builder: &mut SemanticProgramBuilder,
        left: ShapedValue<Bf16, E>,
        right: ShapedValue<Bf16, E>,
    ) -> Result<ShapedValue<Bf16, E>, BuildError> {
        builder.apply_shaped_single(
            add_bf16_op(),
            OperationAttributes::empty(),
            &[left.weaken().erase(), right.weaken().erase()],
        )
    }
}

fn bf16_constant_attributes(bits: u16) -> Result<OperationAttributes, BuildError> {
    OperationAttributes::new([CanonicalField::new(
        BF16_CONSTANT_BITS_ATTRIBUTE,
        canonical_bf16_bits(bits),
    )])
    .map_err(BuildError::InvalidOperationAttributes)
}

fn apply_single(
    builder: &mut SemanticProgramBuilder,
    key: super::OpKey,
    attributes: OperationAttributes,
    operands: &[super::ValueId],
) -> Result<Value<F32>, BuildError> {
    builder.apply_typed_single(key, attributes, operands)
}

fn apply_shaped_single<E: ShapeEvidence>(
    builder: &mut SemanticProgramBuilder,
    key: super::OpKey,
    attributes: OperationAttributes,
    operands: &[super::ValueId],
) -> Result<ShapedValue<F32, E>, BuildError> {
    builder.apply_shaped_single(key, attributes, operands)
}

fn constant_attributes(bits: u32) -> Result<OperationAttributes, BuildError> {
    OperationAttributes::new([CanonicalField::new(
        F32_CONSTANT_BITS_ATTRIBUTE,
        canonical_f32_bits(bits)?,
    )])
    .map_err(BuildError::InvalidOperationAttributes)
}

fn canonical_f32_bits(bits: u32) -> Result<CanonicalValue, BuildError> {
    CanonicalValue::float_bits(
        super::TypeKey::new("tiler", "f32", 1).expect("the governed F32 type identity is valid"),
        bits.to_be_bytes(),
    )
    .map_err(BuildError::InvalidOperationAttributes)
}

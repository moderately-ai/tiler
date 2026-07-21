//! Typed facades for Tiler's governed initial operation profile.

use crate::shape::{Axis, ShapeEvidence, StaticShape};

use super::{
    BuildError, CanonicalField, CanonicalValue, F32, F32_CONSTANT_BITS_ATTRIBUTE,
    OperationAttributes, REDUCTION_AXES_ATTRIBUTE, SemanticProgramBuilder, ShapedValue, Value,
    add_f32_op, constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
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
            CanonicalValue::unsigned(u64::from(bits)),
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
                .map(|axis| CanonicalValue::unsigned(u64::from(axis.get()))),
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
        CanonicalValue::unsigned(u64::from(bits)),
    )])
    .map_err(BuildError::InvalidOperationAttributes)
}

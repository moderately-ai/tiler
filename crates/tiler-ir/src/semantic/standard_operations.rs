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
        let value = Self::apply(builder, bits)?;
        builder.refine(value).map_err(BuildError::ShapeRefinement)
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
        let value = Self::apply(builder, left.weaken(), right.weaken())?;
        builder.refine(value).map_err(BuildError::ShapeRefinement)
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
        let value = Self::apply(builder, left.weaken(), right.weaken())?;
        builder.refine(value).map_err(BuildError::ShapeRefinement)
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
    let mut results = builder.apply(key, attributes, operands)?;
    debug_assert_eq!(results.len(), 1);
    builder.reify(results.remove(0)).map_err(BuildError::Reify)
}

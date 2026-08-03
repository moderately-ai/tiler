//! Closed, typed semantic laws for canonical logical index realization.
//!
//! A law is registered by the same semantic-provider transaction that defines
//! an operation. It is data, not a verdict callback: the verifier interprets it
//! without exposing the candidate region, builds the expected canonical region,
//! and compares the two only after both have passed ordinary structural checks.

use core::fmt;
use std::error::Error;

use crate::semantic::{
    AttributeFieldId, BROADCAST_AXIS_MAPPING_ATTRIBUTE, BroadcastAxisMapping, BroadcastAxisSource,
    CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, CanonicalField, CanonicalIntegerWidth, CanonicalValue,
    CanonicalValueView, ContractionIndex, ContractionIndexStructure, F32_CONSTANT_BITS_ATTRIBUTE,
    OperationAttributes, REDUCTION_AXES_ATTRIBUTE, REINDEX_MAPPING_ATTRIBUTE, ReindexForm,
    ReindexFormKind, ResolvedValueType, TypeKey,
};
use crate::shape::{Axis, Extent, Shape};

use super::{
    DimensionId, DomainRole, FrozenScalarRegistry, IndexBuildError, IndexExprId, IndexInteger,
    IndexRefinementSubject, IndexRegionBuildError, IndexRegionBuilder, ScalarAttributes,
    ScalarOpKey, ScalarReducerBodyBuilder, ScalarValueId, SourcedExtent, SymbolicExtentError,
    TensorAccessId, TensorId, TensorRole, VerifiedIndexRegion, add_f32_scalar_op,
    canonicalize_nan_f32_scalar_op, constant_f32_scalar_op, divide_f32_scalar_op,
    exp_f32_scalar_op, multiply_f32_scalar_op,
};

/// A bounded semantic template for one canonical logical index realization.
///
/// This is deliberately not a universal IR. Each variant is an atomic template
/// whose complete interpretation is owned here; semantics outside this set are
/// unsupported and therefore cannot mint refinement evidence. Verification
/// requires the candidate region's exact canonical identity to equal the region
/// this law constructs. A semantically equivalent alternate logical index form
/// is deliberately refused; physical alternatives belong to later planning.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum IndexRealizationLaw {
    /// Rank-zero constant from one float-bits semantic attribute.
    ConstantFromFloatBits {
        /// Attribute holding the exact payload.
        attribute: AttributeFieldId,
        /// Scalar constant operation receiving that payload.
        scalar: ScalarOpKey,
    },
    /// Binary pointwise scalar application with rank-zero broadcasting only.
    PointwiseBinary {
        /// Scalar operation applied to the two ordered operands.
        scalar: ScalarOpKey,
    },
    /// The governed precise `x / (1 + exp(-x))` scalar chain.
    PreciseSiluF32,
    /// Lexicographic left fold over axes named by one semantic attribute.
    StrictSerialSumF32 {
        /// Attribute containing the ordered reduction axes.
        axes_attribute: AttributeFieldId,
    },
    /// Payload-preserving coordinate transform from a reindex attribute.
    Reindex {
        /// Attribute containing the canonical reindex form.
        mapping_attribute: AttributeFieldId,
    },
    /// Payload-preserving coordinate map from a broadcast attribute.
    Broadcast {
        /// Attribute containing the canonical broadcast map.
        mapping_attribute: AttributeFieldId,
    },
    /// Strict contraction over an explicit index-structure attribute.
    StrictTensorContractionF32 {
        /// Attribute containing the canonical contraction structure.
        structure_attribute: AttributeFieldId,
    },
}

impl IndexRealizationLaw {
    pub(crate) const fn accepts_numerical_contract(
        contract: &super::NumericalContractIdentity,
    ) -> bool {
        matches!(contract.arithmetic(), crate::schedule::ArithmeticType::F32)
    }

    /// Standard constant-f32 law.
    #[must_use]
    pub fn constant_f32() -> Self {
        Self::ConstantFromFloatBits {
            attribute: F32_CONSTANT_BITS_ATTRIBUTE,
            scalar: constant_f32_scalar_op(),
        }
    }

    /// Standard multiply-f32 law.
    #[must_use]
    pub fn multiply_f32() -> Self {
        Self::PointwiseBinary {
            scalar: multiply_f32_scalar_op(),
        }
    }

    /// Standard add-f32 law.
    #[must_use]
    pub fn add_f32() -> Self {
        Self::PointwiseBinary {
            scalar: add_f32_scalar_op(),
        }
    }

    /// Standard strict-serial-sum-f32 law.
    #[must_use]
    pub const fn strict_serial_sum_f32() -> Self {
        Self::StrictSerialSumF32 {
            axes_attribute: REDUCTION_AXES_ATTRIBUTE,
        }
    }

    /// Standard reindex-f32 law.
    #[must_use]
    pub const fn reindex_f32() -> Self {
        Self::Reindex {
            mapping_attribute: REINDEX_MAPPING_ATTRIBUTE,
        }
    }

    /// Standard broadcast-f32 law.
    #[must_use]
    pub const fn broadcast_f32() -> Self {
        Self::Broadcast {
            mapping_attribute: BROADCAST_AXIS_MAPPING_ATTRIBUTE,
        }
    }

    /// Standard strict-tensor-contraction-f32 law.
    #[must_use]
    pub const fn strict_tensor_contraction_f32() -> Self {
        Self::StrictTensorContractionF32 {
            structure_attribute: CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE,
        }
    }

    /// Builds the exact canonical logical region required by this law.
    ///
    /// The candidate is intentionally absent from this API. A law can describe
    /// expected work but cannot inspect or approve provider output.
    pub(crate) fn realize(
        &self,
        subject: &IndexRefinementSubject,
        scalars: &FrozenScalarRegistry,
    ) -> Result<VerifiedIndexRegion, IndexRealizationLawError> {
        let mut builder = IndexRegionBuilder::new(scalars.clone())?;
        {
            let mut context = LawContext {
                builder: &mut builder,
                subject,
            };
            match self {
                Self::ConstantFromFloatBits { attribute, scalar } => {
                    realize_constant(&mut context, *attribute, scalar.clone())?;
                }
                Self::PointwiseBinary { scalar } => {
                    realize_pointwise(&mut context, scalar.clone())?;
                }
                Self::PreciseSiluF32 => realize_silu(&mut context)?,
                Self::StrictSerialSumF32 { axes_attribute } => {
                    realize_serial_sum(&mut context, *axes_attribute)?;
                }
                Self::Reindex { mapping_attribute } => {
                    realize_reindex(&mut context, *mapping_attribute)?;
                }
                Self::Broadcast { mapping_attribute } => {
                    realize_broadcast(&mut context, *mapping_attribute)?;
                }
                Self::StrictTensorContractionF32 {
                    structure_attribute,
                } => realize_contraction(&mut context, *structure_attribute)?,
            }
        }
        builder.build().map_err(IndexRealizationLawError::Build)
    }

    pub(crate) fn encode(&self, output: &mut Vec<u8>) {
        match self {
            Self::ConstantFromFloatBits { attribute, scalar } => {
                output.push(1);
                output.extend_from_slice(&attribute.get().to_be_bytes());
                encode_scalar(output, scalar);
            }
            Self::PointwiseBinary { scalar } => {
                output.push(2);
                encode_scalar(output, scalar);
            }
            Self::PreciseSiluF32 => output.push(3),
            Self::StrictSerialSumF32 { axes_attribute } => {
                output.push(4);
                output.extend_from_slice(&axes_attribute.get().to_be_bytes());
            }
            Self::Reindex { mapping_attribute } => {
                output.push(5);
                output.extend_from_slice(&mapping_attribute.get().to_be_bytes());
            }
            Self::Broadcast { mapping_attribute } => {
                output.push(6);
                output.extend_from_slice(&mapping_attribute.get().to_be_bytes());
            }
            Self::StrictTensorContractionF32 {
                structure_attribute,
            } => {
                output.push(7);
                output.extend_from_slice(&structure_attribute.get().to_be_bytes());
            }
        }
    }
}

fn encode_scalar(output: &mut Vec<u8>, scalar: &ScalarOpKey) {
    for component in [scalar.namespace(), scalar.name()] {
        output.extend_from_slice(&(component.len() as u64).to_be_bytes());
        output.extend_from_slice(component.as_bytes());
    }
    output.extend_from_slice(&scalar.semantic_version().to_be_bytes());
}

/// Failure to interpret a registered logical realization law.
#[derive(Debug)]
#[non_exhaustive]
pub(crate) enum IndexRealizationLawError {
    /// The controlled canonical builder rejected an emission.
    Emit(IndexBuildError),
    /// A sourced extent was unavailable in this static law profile.
    Extent(SymbolicExtentError),
    /// The semantic occurrence is outside the law's exact supported form.
    Unsupported {
        /// Stable rule naming the refused fact.
        rule: &'static str,
    },
    /// Whole-region verification rejected the expected construction.
    Build(IndexRegionBuildError),
}

impl fmt::Display for IndexRealizationLawError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Emit(source) => write!(formatter, "law emission failed: {source}"),
            Self::Extent(source) => write!(formatter, "law extent failed: {source}"),
            Self::Unsupported { rule } => write!(formatter, "law does not support {rule}"),
            Self::Build(source) => write!(formatter, "law region failed verification: {source}"),
        }
    }
}

impl Error for IndexRealizationLawError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Emit(source) => Some(source),
            Self::Extent(source) => Some(source),
            Self::Build(source) => Some(source),
            Self::Unsupported { .. } => None,
        }
    }
}

impl IndexRealizationLawError {
    pub(crate) const fn rule(&self) -> &'static str {
        match self {
            Self::Emit(_) => "canonical-emission",
            Self::Extent(_) => "extent-authority",
            Self::Unsupported { rule } => rule,
            Self::Build(_) => "expected-region-verification",
        }
    }
}

impl From<IndexBuildError> for IndexRealizationLawError {
    fn from(source: IndexBuildError) -> Self {
        Self::Emit(source)
    }
}

impl From<SymbolicExtentError> for IndexRealizationLawError {
    fn from(source: SymbolicExtentError) -> Self {
        Self::Extent(source)
    }
}

fn unsupported(rule: &'static str) -> IndexRealizationLawError {
    IndexRealizationLawError::Unsupported { rule }
}

struct LawContext<'a> {
    builder: &'a mut IndexRegionBuilder,
    subject: &'a IndexRefinementSubject,
}

impl LawContext<'_> {
    fn dimension(
        &mut self,
        role: DomainRole,
        extent: Extent,
    ) -> Result<DimensionId, IndexRealizationLawError> {
        Ok(self.builder.dimension(role, extent)?)
    }
    fn tensor(
        &mut self,
        role: TensorRole,
        value_type: ResolvedValueType,
        shape: Shape,
    ) -> Result<TensorId, IndexRealizationLawError> {
        Ok(self.builder.tensor(role, value_type, shape)?)
    }
    fn constant(&mut self, value: IndexInteger) -> Result<IndexExprId, IndexRealizationLawError> {
        Ok(self.builder.constant(value)?)
    }
    fn dimension_expr(
        &mut self,
        dimension: DimensionId,
    ) -> Result<IndexExprId, IndexRealizationLawError> {
        Ok(self.builder.dimension_expr(dimension)?)
    }
    fn linear_combination(
        &mut self,
        constant: IndexInteger,
        terms: &[(IndexInteger, IndexExprId)],
    ) -> Result<IndexExprId, IndexRealizationLawError> {
        Ok(self.builder.linear_combination(constant, terms)?)
    }
    fn floor_div(
        &mut self,
        value: IndexExprId,
        divisor: SourcedExtent,
    ) -> Result<IndexExprId, IndexRealizationLawError> {
        Ok(self.builder.floor_div(value, divisor)?)
    }
    fn modulo(
        &mut self,
        value: IndexExprId,
        divisor: SourcedExtent,
    ) -> Result<IndexExprId, IndexRealizationLawError> {
        Ok(self.builder.modulo(value, divisor)?)
    }
    fn read(
        &mut self,
        tensor: TensorId,
        domain: &[DimensionId],
        coordinates: &[IndexExprId],
    ) -> Result<ScalarValueId, IndexRealizationLawError> {
        Ok(self.builder.read(tensor, domain, coordinates)?)
    }
    fn write(
        &mut self,
        tensor: TensorId,
        domain: &[DimensionId],
        coordinates: &[IndexExprId],
    ) -> Result<TensorAccessId, IndexRealizationLawError> {
        Ok(self.builder.write(tensor, domain, coordinates)?)
    }
    fn apply(
        &mut self,
        key: ScalarOpKey,
        attributes: ScalarAttributes,
        operands: &[ScalarValueId],
    ) -> Result<super::ScalarResults, IndexRealizationLawError> {
        Ok(self.builder.apply(key, attributes, operands)?)
    }
    fn reduce<F>(
        &mut self,
        dimensions: &[DimensionId],
        init: &[ScalarValueId],
        contributors: &[ScalarValueId],
        build: F,
    ) -> Result<super::ScalarResults, IndexRealizationLawError>
    where
        F: FnOnce(&mut ScalarReducerBodyBuilder<'_>) -> Result<(), IndexBuildError>,
    {
        Ok(self.builder.reduce(dimensions, init, contributors, build)?)
    }
    fn output(
        &mut self,
        access: TensorAccessId,
        value: ScalarValueId,
    ) -> Result<(), IndexRealizationLawError> {
        Ok(self.builder.output(access, value)?)
    }
}

fn realize_constant(
    context: &mut LawContext<'_>,
    attribute: AttributeFieldId,
    scalar: ScalarOpKey,
) -> Result<(), IndexRealizationLawError> {
    if !context.subject.inputs().is_empty() {
        return Err(unsupported("constant-operand-arity"));
    }
    let [result] = context.subject.results() else {
        return Err(unsupported("constant-result-arity"));
    };
    if result.shape().rank() != 0 {
        return Err(unsupported("constant-result-rank"));
    }
    let [field] = context.subject.attributes().fields() else {
        return Err(unsupported("constant-attributes"));
    };
    if field.id() != attribute {
        return Err(unsupported("constant-attribute-key"));
    }
    let attributes = scalar_attributes(field.value().clone())?;
    let output = context.tensor(
        TensorRole::Output,
        result.value_type().clone(),
        result.shape().clone(),
    )?;
    let value = single_result(&context.apply(scalar, attributes, &[])?, "constant")?;
    let write = context.write(output, &[], &[])?;
    context.output(write, value)
}

fn realize_pointwise(
    context: &mut LawContext<'_>,
    scalar: ScalarOpKey,
) -> Result<(), IndexRealizationLawError> {
    let [result] = context.subject.results() else {
        return Err(unsupported("pointwise-result-arity"));
    };
    if context.subject.operands().len() != 2 {
        return Err(unsupported("pointwise-operand-arity"));
    }
    let shape = result.shape().clone();
    let dimensions = declare_parallel_domain(context, &shape)?;
    let coordinates = dimension_expressions(context, &dimensions)?;
    let inputs = context.subject.inputs().to_vec();
    let operands = context.subject.operands().to_vec();
    let mut tensors = Vec::with_capacity(inputs.len());
    for input in &inputs {
        tensors.push(context.tensor(
            TensorRole::Input,
            input.value_type().clone(),
            input.shape().clone(),
        )?);
    }
    let mut values = Vec::with_capacity(2);
    for position in operands {
        let boundary = &inputs[position];
        let value = if boundary.shape() == &shape {
            context.read(tensors[position], &dimensions, &coordinates)?
        } else if boundary.shape().rank() == 0 {
            context.read(tensors[position], &[], &[])?
        } else {
            return Err(unsupported("pointwise-broadcast"));
        };
        values.push(value);
    }
    let value = single_result(
        &context.apply(scalar, ScalarAttributes::empty(), &values)?,
        "pointwise",
    )?;
    let output = context.tensor(TensorRole::Output, result.value_type().clone(), shape)?;
    let write = context.write(output, &dimensions, &coordinates)?;
    context.output(write, value)
}

fn realize_silu(context: &mut LawContext<'_>) -> Result<(), IndexRealizationLawError> {
    let [result] = context.subject.results() else {
        return Err(unsupported("silu-result-arity"));
    };
    if context.subject.operands() != [0] || context.subject.inputs().len() != 1 {
        return Err(unsupported("silu-operand-arity"));
    }
    let input = context.subject.inputs()[0].clone();
    if input.shape() != result.shape() {
        return Err(unsupported("silu-shape"));
    }
    let shape = result.shape().clone();
    let dimensions = declare_parallel_domain(context, &shape)?;
    let coordinates = dimension_expressions(context, &dimensions)?;
    let tensor = context.tensor(TensorRole::Input, input.value_type().clone(), shape.clone())?;
    let argument = context.read(tensor, &dimensions, &coordinates)?;
    let negative_one = scalar_constant(context, 0xbf80_0000)?;
    let negated = apply_one(context, multiply_f32_scalar_op(), &[argument, negative_one])?;
    let exponential = apply_one(context, exp_f32_scalar_op(), &[negated])?;
    let one = scalar_constant(context, 0x3f80_0000)?;
    let divisor = apply_one(context, add_f32_scalar_op(), &[one, exponential])?;
    let value = apply_one(context, divide_f32_scalar_op(), &[argument, divisor])?;
    let output = context.tensor(TensorRole::Output, result.value_type().clone(), shape)?;
    let write = context.write(output, &dimensions, &coordinates)?;
    context.output(write, value)
}

fn scalar_constant(
    context: &mut LawContext<'_>,
    bits: u32,
) -> Result<ScalarValueId, IndexRealizationLawError> {
    let value = CanonicalValue::float_bits(
        TypeKey::new("tiler", "f32", 1).expect("the governed f32 key is valid"),
        bits.to_be_bytes(),
    )
    .map_err(|_| unsupported("scalar-constant"))?;
    let values = context.apply(constant_f32_scalar_op(), scalar_attributes(value)?, &[])?;
    single_result(&values, "scalar-constant")
}

fn scalar_attributes(bits: CanonicalValue) -> Result<ScalarAttributes, IndexRealizationLawError> {
    let record = CanonicalValue::record([CanonicalField::new(F32_CONSTANT_BITS_ATTRIBUTE, bits)])
        .map_err(|_| unsupported("scalar-constant-attributes"))?;
    ScalarAttributes::new(record).map_err(|_| unsupported("scalar-constant-attributes"))
}

fn apply_one(
    context: &mut LawContext<'_>,
    key: ScalarOpKey,
    operands: &[ScalarValueId],
) -> Result<ScalarValueId, IndexRealizationLawError> {
    let values = context.apply(key, ScalarAttributes::empty(), operands)?;
    single_result(&values, "scalar-application")
}

fn single_result(
    values: &super::ScalarResults,
    rule: &'static str,
) -> Result<ScalarValueId, IndexRealizationLawError> {
    if values.len() != 1 {
        return Err(unsupported(rule));
    }
    values.get(0).ok_or_else(|| unsupported(rule))
}

fn declare_parallel_domain(
    context: &mut LawContext<'_>,
    shape: &Shape,
) -> Result<Vec<DimensionId>, IndexRealizationLawError> {
    shape
        .extents()
        .iter()
        .copied()
        .map(|extent| context.dimension(DomainRole::Parallel, extent))
        .collect()
}

fn dimension_expressions(
    context: &mut LawContext<'_>,
    dimensions: &[DimensionId],
) -> Result<Vec<IndexExprId>, IndexRealizationLawError> {
    dimensions
        .iter()
        .copied()
        .map(|dimension| context.dimension_expr(dimension))
        .collect()
}

// The remaining non-pointwise templates are filled below; keeping their entry
// points explicit makes unsupported support impossible to confuse with a law.
fn realize_serial_sum(
    context: &mut LawContext<'_>,
    attribute: AttributeFieldId,
) -> Result<(), IndexRealizationLawError> {
    let plan = SumPlan::derive(context.subject, attribute)?;
    let kept = plan.declare_kept_domain(context)?;
    let kept_coordinates = dimension_expressions(context, &kept)?;
    let input = context.tensor(
        TensorRole::Input,
        plan.value_type.clone(),
        plan.input_shape.clone(),
    )?;
    let output = context.tensor(
        TensorRole::Output,
        plan.value_type.clone(),
        plan.output_shape.clone(),
    )?;
    let total = if plan.reduced_points == 0 {
        plan.fold_empty(context, input, &kept, &kept_coordinates)?
    } else {
        let seed = plan.read_contributor(context, input, &kept, &kept_coordinates, None)?;
        if plan.reduced_points == 1 {
            apply_one(context, canonicalize_nan_f32_scalar_op(), &[seed])?
        } else {
            plan.fold_tail(context, input, &kept, &kept_coordinates, seed)?
        }
    };
    let write = context.write(output, &kept, &kept_coordinates)?;
    context.output(write, total)
}
fn realize_reindex(
    context: &mut LawContext<'_>,
    attribute: AttributeFieldId,
) -> Result<(), IndexRealizationLawError> {
    let ([input], [result]) = (context.subject.inputs(), context.subject.results()) else {
        return Err(unsupported("reindex-arity"));
    };
    if context.subject.operands() != [0] {
        return Err(unsupported("reindex-operand-binding"));
    }
    let value = context
        .subject
        .attributes()
        .get(attribute)
        .ok_or_else(|| unsupported("reindex-form-missing"))?;
    let form = ReindexForm::from_canonical_value(value).map_err(|_| unsupported("reindex-form"))?;
    let input_shape = input.shape().clone();
    let result_shape = result.shape().clone();
    if form
        .result_shape(&input_shape)
        .map_err(|_| unsupported("reindex-form"))?
        != result_shape
    {
        return Err(unsupported("reindex-result-shape"));
    }
    let dimensions = declare_parallel_domain(context, &result_shape)?;
    let coordinates = dimension_expressions(context, &dimensions)?;
    let operand_coordinates =
        reindex_operand_coordinates(context, &form, &input_shape, &coordinates)?;
    let domain = match form.kind() {
        ReindexFormKind::InsertUnitAxis => {
            let inserted = axis_position(
                *form
                    .axes()
                    .first()
                    .ok_or_else(|| unsupported("reindex-axis"))?,
            )?;
            dimensions
                .iter()
                .enumerate()
                .filter(|(position, _)| *position != inserted)
                .map(|(_, dimension)| *dimension)
                .collect()
        }
        _ => dimensions.clone(),
    };
    let tensor = context.tensor(TensorRole::Input, input.value_type().clone(), input_shape)?;
    let value = context.read(tensor, &domain, &operand_coordinates)?;
    let output = context.tensor(
        TensorRole::Output,
        result.value_type().clone(),
        result_shape,
    )?;
    let write = context.write(output, &dimensions, &coordinates)?;
    context.output(write, value)
}
fn realize_broadcast(
    context: &mut LawContext<'_>,
    attribute: AttributeFieldId,
) -> Result<(), IndexRealizationLawError> {
    let ([input], [result]) = (context.subject.inputs(), context.subject.results()) else {
        return Err(unsupported("broadcast-arity"));
    };
    if context.subject.operands() != [0] {
        return Err(unsupported("broadcast-operand-binding"));
    }
    let value = context
        .subject
        .attributes()
        .get(attribute)
        .ok_or_else(|| unsupported("broadcast-mapping-missing"))?;
    let mapping = BroadcastAxisMapping::from_canonical_value(value)
        .map_err(|_| unsupported("broadcast-mapping"))?;
    let input_shape = input.shape().clone();
    let result_shape = result.shape().clone();
    if mapping
        .result_shape(&input_shape)
        .map_err(|_| unsupported("broadcast-mapping"))?
        != result_shape
    {
        return Err(unsupported("broadcast-result-shape"));
    }
    let dimensions = declare_parallel_domain(context, &result_shape)?;
    let coordinates = dimension_expressions(context, &dimensions)?;
    let zero = context.constant(IndexInteger::from_u64(0))?;
    let domain = mapping
        .sources()
        .iter()
        .zip(&dimensions)
        .filter(|(source, _)| matches!(source, BroadcastAxisSource::FromOperand(_)))
        .map(|(_, dimension)| *dimension)
        .collect::<Vec<_>>();
    let mut operand_coordinates = vec![None; input_shape.rank()];
    for (result_axis, source) in mapping.sources().iter().enumerate() {
        let Some(axis) = source.operand_axis() else {
            continue;
        };
        let index = axis_position(axis)?;
        let slot = operand_coordinates
            .get_mut(index)
            .ok_or_else(|| unsupported("broadcast-axis"))?;
        let coordinate = match source {
            BroadcastAxisSource::FromOperand(_) => *coordinates
                .get(result_axis)
                .ok_or_else(|| unsupported("broadcast-coordinate"))?,
            BroadcastAxisSource::StretchUnit(_) => zero,
            BroadcastAxisSource::Replicate => unreachable!("replication has no operand axis"),
        };
        if slot.replace(coordinate).is_some() {
            return Err(unsupported("broadcast-axis-repeated"));
        }
    }
    let operand_coordinates = operand_coordinates
        .into_iter()
        .map(|coordinate| coordinate.ok_or_else(|| unsupported("broadcast-axis-unmapped")))
        .collect::<Result<Vec<_>, _>>()?;
    let tensor = context.tensor(TensorRole::Input, input.value_type().clone(), input_shape)?;
    let value = context.read(tensor, &domain, &operand_coordinates)?;
    let output = context.tensor(
        TensorRole::Output,
        result.value_type().clone(),
        result_shape,
    )?;
    let write = context.write(output, &dimensions, &coordinates)?;
    context.output(write, value)
}
fn realize_contraction(
    context: &mut LawContext<'_>,
    attribute: AttributeFieldId,
) -> Result<(), IndexRealizationLawError> {
    let plan = ContractionPlan::derive(context.subject, attribute)?;
    let output = declare_parallel_domain(context, &plan.output_shape)?;
    let output_coordinates = dimension_expressions(context, &output)?;
    let mut tensors = Vec::with_capacity(plan.operand_shapes.len());
    for shape in &plan.operand_shapes {
        tensors.push(context.tensor(TensorRole::Input, plan.value_type.clone(), shape.clone())?);
    }
    let result = context.tensor(
        TensorRole::Output,
        plan.value_type.clone(),
        plan.output_shape.clone(),
    )?;
    let seed = plan.product(context, &tensors, &output, &output_coordinates, None)?;
    let total = if plan.contracted_points == 1 {
        seed
    } else {
        let tail = context.dimension(
            DomainRole::Reduction,
            Extent::new(plan.contracted_points - 1),
        )?;
        let contributor =
            plan.product(context, &tensors, &output, &output_coordinates, Some(tail))?;
        let folded = context.reduce(&[tail], &[seed], &[contributor], add_reducer)?;
        single_result(&folded, "contraction")?
    };
    let write = context.write(result, &output, &output_coordinates)?;
    context.output(write, total)
}

fn axis_position(axis: Axis) -> Result<usize, IndexRealizationLawError> {
    usize::try_from(axis.get()).map_err(|_| unsupported("axis-width"))
}

struct SumPlan {
    value_type: ResolvedValueType,
    input_shape: Shape,
    output_shape: Shape,
    reduced: Vec<bool>,
    reduced_strides: Vec<u64>,
    reduced_extents: Vec<u64>,
    reduced_points: u64,
}

impl SumPlan {
    fn derive(
        subject: &IndexRefinementSubject,
        attribute: AttributeFieldId,
    ) -> Result<Self, IndexRealizationLawError> {
        let ([input], [result]) = (subject.inputs(), subject.results()) else {
            return Err(unsupported("sum-arity"));
        };
        if subject.operands() != [0] {
            return Err(unsupported("sum-operand-binding"));
        }
        let axes = reduction_axes(subject.attributes(), attribute)?;
        let input_shape = input.shape().clone();
        let mut reduced = vec![false; input_shape.rank()];
        for axis in &axes {
            let index = axis_position(*axis)?;
            let slot = reduced
                .get_mut(index)
                .ok_or_else(|| unsupported("sum-axis-range"))?;
            if std::mem::replace(slot, true) {
                return Err(unsupported("sum-axis-duplicate"));
            }
        }
        if &input_shape.without_axes(&axes) != result.shape() {
            return Err(unsupported("sum-result-shape"));
        }
        let reduced_extents = input_shape
            .extents()
            .iter()
            .zip(&reduced)
            .filter(|(_, reduced)| **reduced)
            .map(|(extent, _)| extent.get())
            .collect::<Vec<_>>();
        let mut reduced_strides = vec![0_u64; reduced_extents.len()];
        let mut stride = 1_u64;
        for (position, extent) in reduced_extents.iter().enumerate().rev() {
            reduced_strides[position] = stride;
            stride = stride
                .checked_mul(*extent)
                .ok_or_else(|| unsupported("sum-reduced-extent-overflow"))?;
        }
        Ok(Self {
            value_type: input.value_type().clone(),
            input_shape,
            output_shape: result.shape().clone(),
            reduced,
            reduced_strides,
            reduced_extents,
            reduced_points: stride,
        })
    }

    fn declare_kept_domain(
        &self,
        context: &mut LawContext<'_>,
    ) -> Result<Vec<DimensionId>, IndexRealizationLawError> {
        self.input_shape
            .extents()
            .iter()
            .zip(&self.reduced)
            .filter(|(_, reduced)| !**reduced)
            .map(|(extent, _)| context.dimension(DomainRole::Parallel, *extent))
            .collect()
    }

    fn read_contributor(
        &self,
        context: &mut LawContext<'_>,
        input: TensorId,
        kept: &[DimensionId],
        kept_coordinates: &[IndexExprId],
        tail: Option<DimensionId>,
    ) -> Result<ScalarValueId, IndexRealizationLawError> {
        let offset = match tail {
            Some(tail) => {
                let induction = context.dimension_expr(tail)?;
                let one = IndexInteger::from_u64(1);
                Some(context.linear_combination(one.clone(), &[(one, induction)])?)
            }
            None => None,
        };
        let zero = context.constant(IndexInteger::from_u64(0))?;
        let mut coordinates = Vec::with_capacity(self.input_shape.rank());
        let mut kept_position = 0;
        let mut reduced_position = 0;
        for reduced in &self.reduced {
            if *reduced {
                coordinates.push(match offset {
                    Some(offset) => self.decode_reduced(context, offset, reduced_position)?,
                    None => zero,
                });
                reduced_position += 1;
            } else {
                coordinates.push(kept_coordinates[kept_position]);
                kept_position += 1;
            }
        }
        let mut domain = kept.to_vec();
        domain.extend(tail);
        context.read(input, &domain, &coordinates)
    }

    fn decode_reduced(
        &self,
        context: &mut LawContext<'_>,
        offset: IndexExprId,
        position: usize,
    ) -> Result<IndexExprId, IndexRealizationLawError> {
        let stride = self.reduced_strides[position];
        let extent = self.reduced_extents[position];
        let wrapped = if position == 0 {
            offset
        } else {
            let modulus = stride
                .checked_mul(extent)
                .ok_or_else(|| unsupported("sum-reduced-extent-overflow"))?;
            context.modulo(offset, SourcedExtent::Static(Extent::new(modulus)))?
        };
        if stride == 1 {
            Ok(wrapped)
        } else {
            context.floor_div(wrapped, SourcedExtent::Static(Extent::new(stride)))
        }
    }

    fn fold_empty(
        &self,
        context: &mut LawContext<'_>,
        input: TensorId,
        kept: &[DimensionId],
        kept_coordinates: &[IndexExprId],
    ) -> Result<ScalarValueId, IndexRealizationLawError> {
        let mut reduced_dimensions = Vec::new();
        let mut coordinates = Vec::with_capacity(self.input_shape.rank());
        let mut kept_position = 0;
        for (extent, reduced) in self.input_shape.extents().iter().zip(&self.reduced) {
            if *reduced {
                let dimension = context.dimension(DomainRole::Reduction, *extent)?;
                coordinates.push(context.dimension_expr(dimension)?);
                reduced_dimensions.push(dimension);
            } else {
                coordinates.push(kept_coordinates[kept_position]);
                kept_position += 1;
            }
        }
        let mut domain = kept.to_vec();
        domain.extend(reduced_dimensions.iter().copied());
        let contributor = context.read(input, &domain, &coordinates)?;
        let identity = scalar_constant(context, 0.0_f32.to_bits())?;
        let folded = context.reduce(
            &reduced_dimensions,
            &[identity],
            &[contributor],
            add_reducer,
        )?;
        single_result(&folded, "reduction")
    }

    fn fold_tail(
        &self,
        context: &mut LawContext<'_>,
        input: TensorId,
        kept: &[DimensionId],
        kept_coordinates: &[IndexExprId],
        seed: ScalarValueId,
    ) -> Result<ScalarValueId, IndexRealizationLawError> {
        let tail = context.dimension(
            DomainRole::Reduction,
            Extent::new(self.reduced_points.saturating_sub(1)),
        )?;
        let contributor =
            self.read_contributor(context, input, kept, kept_coordinates, Some(tail))?;
        let folded = context.reduce(&[tail], &[seed], &[contributor], add_reducer)?;
        single_result(&folded, "reduction")
    }
}

fn add_reducer(body: &mut ScalarReducerBodyBuilder<'_>) -> Result<(), IndexBuildError> {
    let state = body.state(0).expect("one state");
    let contributor = body.contributor(0).expect("one contributor");
    let accumulated = body
        .apply(
            add_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[state, contributor],
        )?
        .get(0)
        .expect("governed add has one result");
    body.yield_values(&[accumulated])
}

fn reduction_axes(
    attributes: &OperationAttributes,
    attribute: AttributeFieldId,
) -> Result<Vec<Axis>, IndexRealizationLawError> {
    let value = attributes
        .get(attribute)
        .ok_or_else(|| unsupported("sum-axes-missing"))?;
    let CanonicalValueView::Sequence(values) = value.view() else {
        return Err(unsupported("sum-axes-kind"));
    };
    values
        .iter()
        .map(|value| {
            let CanonicalValueView::Unsigned {
                width: CanonicalIntegerWidth::Bits32,
                bits,
            } = value.view()
            else {
                return Err(unsupported("sum-axes-element"));
            };
            Ok(Axis::new(
                u32::try_from(bits).map_err(|_| unsupported("sum-axes-width"))?,
            ))
        })
        .collect()
}

struct ContractionPlan {
    value_type: ResolvedValueType,
    operand_shapes: Vec<Shape>,
    output_shape: Shape,
    sources: Vec<Vec<AxisSource>>,
    contracted_strides: Vec<u64>,
    contracted_extents: Vec<u64>,
    contracted_points: u64,
}

#[derive(Clone, Copy)]
enum AxisSource {
    Output(usize),
    Contracted(usize),
}

impl ContractionPlan {
    fn derive(
        subject: &IndexRefinementSubject,
        attribute: AttributeFieldId,
    ) -> Result<Self, IndexRealizationLawError> {
        let ([left, right], [result]) = (subject.inputs(), subject.results()) else {
            return Err(unsupported("contraction-arity"));
        };
        if subject.operands() != [0, 1] {
            return Err(unsupported("contraction-operand-binding"));
        }
        let [field] = subject.attributes().fields() else {
            return Err(unsupported("contraction-attributes"));
        };
        if field.id() != attribute {
            return Err(unsupported("contraction-attributes"));
        }
        let structure = ContractionIndexStructure::from_canonical_value(field.value())
            .map_err(|_| unsupported("contraction-structure"))?;
        let boundaries = [left, right];
        if structure.operand_count() != boundaries.len() {
            return Err(unsupported("contraction-operand-count"));
        }
        let mut extents: Vec<(ContractionIndex, Extent)> = Vec::new();
        let mut operand_shapes = Vec::with_capacity(boundaries.len());
        for (tuple, boundary) in structure.operands().zip(boundaries) {
            let shape = boundary.shape().clone();
            if shape.rank() != tuple.len() {
                return Err(unsupported("contraction-rank"));
            }
            for (axis, index) in tuple.iter().enumerate() {
                let extent = shape.extents()[axis];
                match extents.iter().find(|(bound, _)| bound == index) {
                    Some((_, bound)) if *bound != extent => {
                        return Err(unsupported("contraction-extent"));
                    }
                    Some(_) => {}
                    None => extents.push((*index, extent)),
                }
            }
            operand_shapes.push(shape);
        }
        let shape_over = |indices: &[ContractionIndex]| {
            Shape::try_new(
                indices
                    .iter()
                    .map(|index| {
                        extents
                            .iter()
                            .find(|(bound, _)| bound == index)
                            .map(|(_, extent)| *extent)
                            .ok_or_else(|| unsupported("contraction-extent"))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .map_err(|_| unsupported("contraction-shape"))
        };
        let output_shape = shape_over(structure.output())?;
        if &output_shape != result.shape() {
            return Err(unsupported("contraction-result-shape"));
        }
        let contracted_shape = shape_over(structure.contracted())?;
        let mut sources = Vec::with_capacity(structure.operand_count());
        for tuple in structure.operands() {
            let mut operand = Vec::with_capacity(tuple.len());
            for index in tuple {
                if let Some(position) = structure
                    .output()
                    .iter()
                    .position(|candidate| candidate == index)
                {
                    operand.push(AxisSource::Output(position));
                } else if let Some(position) = structure
                    .contracted()
                    .iter()
                    .position(|candidate| candidate == index)
                {
                    operand.push(AxisSource::Contracted(position));
                } else {
                    return Err(unsupported("contraction-index"));
                }
            }
            sources.push(operand);
        }
        let contracted_extents = contracted_shape
            .extents()
            .iter()
            .map(|extent| extent.get())
            .collect::<Vec<_>>();
        let mut contracted_strides = vec![0_u64; contracted_extents.len()];
        let mut stride = 1_u64;
        for (position, extent) in contracted_extents.iter().enumerate().rev() {
            contracted_strides[position] = stride;
            stride = stride
                .checked_mul(*extent)
                .ok_or_else(|| unsupported("contraction-extent-overflow"))?;
        }
        if stride == 0 {
            return Err(unsupported("contraction-empty-domain"));
        }
        Ok(Self {
            value_type: result.value_type().clone(),
            operand_shapes,
            output_shape,
            sources,
            contracted_strides,
            contracted_extents,
            contracted_points: stride,
        })
    }

    fn product(
        &self,
        context: &mut LawContext<'_>,
        tensors: &[TensorId],
        output: &[DimensionId],
        output_coordinates: &[IndexExprId],
        tail: Option<DimensionId>,
    ) -> Result<ScalarValueId, IndexRealizationLawError> {
        let offset = match tail {
            Some(tail) => {
                let induction = context.dimension_expr(tail)?;
                let one = IndexInteger::from_u64(1);
                Some(context.linear_combination(one.clone(), &[(one, induction)])?)
            }
            None => None,
        };
        let zero = context.constant(IndexInteger::from_u64(0))?;
        let mut domain = output.to_vec();
        domain.extend(tail);
        let mut values = Vec::with_capacity(tensors.len());
        for (position, tensor) in tensors.iter().enumerate() {
            let coordinates = self.sources[position]
                .iter()
                .map(|source| match source {
                    AxisSource::Output(axis) => output_coordinates
                        .get(*axis)
                        .copied()
                        .ok_or_else(|| unsupported("contraction-coordinate")),
                    AxisSource::Contracted(axis) => match offset {
                        Some(offset) => self.decode_contracted(context, offset, *axis),
                        None => Ok(zero),
                    },
                })
                .collect::<Result<Vec<_>, _>>()?;
            values.push(context.read(*tensor, &domain, &coordinates)?);
        }
        apply_one(context, multiply_f32_scalar_op(), &values)
    }

    fn decode_contracted(
        &self,
        context: &mut LawContext<'_>,
        offset: IndexExprId,
        position: usize,
    ) -> Result<IndexExprId, IndexRealizationLawError> {
        let stride = self.contracted_strides[position];
        let extent = self.contracted_extents[position];
        let wrapped = if position == 0 {
            offset
        } else {
            let modulus = stride
                .checked_mul(extent)
                .ok_or_else(|| unsupported("contraction-extent-overflow"))?;
            context.modulo(offset, SourcedExtent::Static(Extent::new(modulus)))?
        };
        if stride == 1 {
            Ok(wrapped)
        } else {
            context.floor_div(wrapped, SourcedExtent::Static(Extent::new(stride)))
        }
    }
}

fn reindex_operand_coordinates(
    context: &mut LawContext<'_>,
    form: &ReindexForm,
    input_shape: &Shape,
    coordinates: &[IndexExprId],
) -> Result<Vec<IndexExprId>, IndexRealizationLawError> {
    let extents = input_shape.extents();
    let at = |position: usize| {
        coordinates
            .get(position)
            .copied()
            .ok_or_else(|| unsupported("reindex-coordinate"))
    };
    match form.kind() {
        ReindexFormKind::PermuteAxes => {
            let mut operand = vec![None; extents.len()];
            for (position, axis) in form.axes().iter().enumerate() {
                let slot = operand
                    .get_mut(axis_position(*axis)?)
                    .ok_or_else(|| unsupported("reindex-axis"))?;
                if slot.replace(at(position)?).is_some() {
                    return Err(unsupported("reindex-permutation"));
                }
            }
            operand
                .into_iter()
                .map(|value| value.ok_or_else(|| unsupported("reindex-permutation")))
                .collect()
        }
        ReindexFormKind::SplitAxis => {
            let axis = axis_position(
                *form
                    .axes()
                    .first()
                    .ok_or_else(|| unsupported("reindex-axis"))?,
            )?;
            let factors = form.factors();
            let mut strides = vec![1_u64; factors.len()];
            let mut stride = 1_u64;
            for (position, factor) in factors.iter().enumerate().rev() {
                strides[position] = stride;
                stride = stride
                    .checked_mul(factor.get())
                    .ok_or_else(|| unsupported("reindex-split-overflow"))?;
            }
            let terms = strides
                .iter()
                .enumerate()
                .map(|(position, stride)| {
                    Ok((IndexInteger::from_u64(*stride), at(axis + position)?))
                })
                .collect::<Result<Vec<_>, IndexRealizationLawError>>()?;
            let linearized = context.linear_combination(IndexInteger::from_u64(0), &terms)?;
            (0..extents.len())
                .map(|position| match position.cmp(&axis) {
                    std::cmp::Ordering::Less => at(position),
                    std::cmp::Ordering::Equal => Ok(linearized),
                    std::cmp::Ordering::Greater => at(position + factors.len() - 1),
                })
                .collect()
        }
        ReindexFormKind::MergeAxes => {
            let axes = form.axes();
            let first = axis_position(*axes.first().ok_or_else(|| unsupported("reindex-axis"))?)?;
            let merged = axes
                .iter()
                .map(|axis| {
                    extents
                        .get(axis_position(*axis)?)
                        .map(|extent| extent.get())
                        .ok_or_else(|| unsupported("reindex-axis"))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let mut strides = vec![1_u64; merged.len()];
            let mut stride = 1_u64;
            for (position, extent) in merged.iter().enumerate().rev() {
                strides[position] = stride;
                stride = stride
                    .checked_mul(*extent)
                    .ok_or_else(|| unsupported("reindex-merge-overflow"))?;
            }
            let linear = at(first)?;
            let mut decoded = Vec::with_capacity(merged.len());
            for position in 0..merged.len() {
                let wrapped = if position == 0 {
                    linear
                } else {
                    let modulus = strides[position]
                        .checked_mul(merged[position])
                        .ok_or_else(|| unsupported("reindex-merge-overflow"))?;
                    context.modulo(linear, SourcedExtent::Static(Extent::new(modulus)))?
                };
                decoded.push(if strides[position] == 1 {
                    wrapped
                } else {
                    context.floor_div(
                        wrapped,
                        SourcedExtent::Static(Extent::new(strides[position])),
                    )?
                });
            }
            (0..extents.len())
                .map(|position| {
                    if position < first {
                        at(position)
                    } else if position < first + merged.len() {
                        Ok(decoded[position - first])
                    } else {
                        at(position - merged.len() + 1)
                    }
                })
                .collect()
        }
        ReindexFormKind::InsertUnitAxis => {
            let inserted = axis_position(
                *form
                    .axes()
                    .first()
                    .ok_or_else(|| unsupported("reindex-axis"))?,
            )?;
            (0..extents.len())
                .map(|axis| at(if axis < inserted { axis } else { axis + 1 }))
                .collect()
        }
        ReindexFormKind::RemoveUnitAxis => {
            let removed = axis_position(
                *form
                    .axes()
                    .first()
                    .ok_or_else(|| unsupported("reindex-axis"))?,
            )?;
            let zero = context.constant(IndexInteger::from_u64(0))?;
            (0..extents.len())
                .map(|axis| match axis.cmp(&removed) {
                    std::cmp::Ordering::Less => at(axis),
                    std::cmp::Ordering::Equal => Ok(zero),
                    std::cmp::Ordering::Greater => at(axis - 1),
                })
                .collect()
        }
        ReindexFormKind::ReverseAxis => {
            let reversed = axis_position(
                *form
                    .axes()
                    .first()
                    .ok_or_else(|| unsupported("reindex-axis"))?,
            )?;
            let last = i128::from(
                extents
                    .get(reversed)
                    .ok_or_else(|| unsupported("reindex-axis"))?
                    .get(),
            )
            .checked_sub(1)
            .ok_or_else(|| unsupported("reindex-reverse-extent"))?;
            let mirrored = context.linear_combination(
                IndexInteger::from_i128(last),
                &[(IndexInteger::from_i128(-1), at(reversed)?)],
            )?;
            (0..extents.len())
                .map(|axis| {
                    if axis == reversed {
                        Ok(mirrored)
                    } else {
                        at(axis)
                    }
                })
                .collect()
        }
    }
}

//! The lowering capabilities the bounded compiler profile ships with.
//!
//! Each governed semantic family — `tiler.constant-f32`, `tiler.multiply-f32`,
//! `tiler.add-f32`, and `tiler.strict-serial-sum-f32` — gets exactly one
//! [`IndexAccessLoweringProvider`] registered against
//! [`FrozenScalarRegistry::standard`]. The providers are shape- and
//! attribute-driven: every extent, every broadcast, and every constant bit
//! pattern is read from the [`LoweredOccurrence`] facts the host hands them.
//! That is what lets one registered capability cover a family instead of one per
//! program shape, and it is why two `tiler.constant-f32` occurrences with
//! different bits are two lowerings rather than an unresolvable registry
//! ambiguity.
//!
//! Scope boundary: these providers describe *what index work realizes an
//! occurrence*. They are not the physical schedule, and refinement binds their
//! structure and reached authority, never their per-point arithmetic. The
//! numerical contract of a compiled program remains the `fusion`, `physical`,
//! and structured-kernel authorities' to prove.

use std::sync::Arc;

use tiler_ir::index::{
    DomainRole, FrozenScalarRegistry, IndexExprId, IndexInteger, ScalarAttributes, ScalarOpKey,
    ScalarRegistryError, ScalarValueId, SourcedExtent, add_f32_scalar_op,
    canonicalize_nan_f32_scalar_op, constant_f32_scalar_op, multiply_f32_scalar_op,
};
use tiler_ir::semantic::{
    CanonicalField, CanonicalIntegerWidth, CanonicalValue, CanonicalValueView, F32,
    F32_CONSTANT_BITS_ATTRIBUTE, OpKey, OperationAttributes, ProviderIdentity,
    REDUCTION_AXES_ATTRIBUTE, ResolvedValueType, TypeKey, add_f32_op, constant_f32_op,
    multiply_f32_op, strict_serial_sum_f32_op,
};
use tiler_ir::shape::{Axis, Extent, Shape};

use crate::capability::{
    FrozenLoweringCapabilityRegistry, IndexAccessLoweringContext, IndexAccessLoweringProvider,
    LoweredOccurrence, LoweringCapabilityRegistryBuilder, LoweringCapabilityRevision,
    LoweringEmitError, LoweringRegistryError, LoweringSignature,
};

/// Output-affecting revision of every governed lowering capability.
const GOVERNED_CAPABILITY_REVISION: u32 = 1;

/// A failure to assemble the governed lowering-capability registry.
///
/// Every variant is a defect in Tiler's own governed profile rather than in a
/// caller's request, so the compile path treats it as invalid compiler output.
#[derive(Clone, Debug)]
pub(crate) enum GovernedRegistryError {
    /// The governed scalar authority rejected its own standard profile.
    Scalar(Arc<ScalarRegistryError>),
    /// The governed lowering registry rejected its own registration.
    Registry(LoweringRegistryError),
}

impl std::fmt::Display for GovernedRegistryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scalar(source) => {
                write!(formatter, "governed scalar authority failed: {source}")
            }
            Self::Registry(source) => {
                write!(formatter, "governed lowering registration failed: {source}")
            }
        }
    }
}

impl std::error::Error for GovernedRegistryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Scalar(source) => Some(source.as_ref()),
            Self::Registry(source) => Some(source),
        }
    }
}

impl From<LoweringRegistryError> for GovernedRegistryError {
    fn from(value: LoweringRegistryError) -> Self {
        Self::Registry(value)
    }
}

/// Returns the governed scalar authority every governed lowering emits against.
///
/// # Errors
///
/// Returns [`GovernedRegistryError::Scalar`] when the governed standard scalar
/// profile rejects itself.
pub(crate) fn governed_scalars() -> Result<FrozenScalarRegistry, GovernedRegistryError> {
    FrozenScalarRegistry::standard()
        .map_err(|source| GovernedRegistryError::Scalar(Arc::new(source)))
}

/// Builds the governed index-access lowering capabilities.
///
/// The registry is composed against the exact semantic authority the governed
/// scalar profile was composed with, so the three snapshots the capability
/// identity binds cannot drift apart.
///
/// # Errors
///
/// Returns [`GovernedRegistryError`] when a governed registration violates the
/// same public contract an external provider is held to.
pub(crate) fn governed_lowering_capabilities(
    scalars: &FrozenScalarRegistry,
) -> Result<FrozenLoweringCapabilityRegistry, GovernedRegistryError> {
    let mut builder = LoweringCapabilityRegistryBuilder::new(
        scalars.semantic_authority().clone(),
        scalars.clone(),
    );
    for capability in governed_index_access_capabilities()? {
        capability.register(&mut builder)?;
    }
    Ok(builder.freeze())
}

/// Registers the shipped index-access capabilities onto a caller's builder,
/// except the operation families the caller is substituting for.
///
/// This is the affordance [`GovernedIndexAccess`]'s own documentation describes
/// — composing a registry from a chosen subset so an external provider replaces
/// one governed family without re-implementing the other three — made reachable
/// from outside the crate. Until now it existed and was crate-private, which is
/// why the conformance gate that exercises exactly this could only live inside
/// `pipeline.rs`.
///
/// # Errors
///
/// Returns [`LoweringRegistryError`] when the builder refuses a registration,
/// which for a governed capability means the composed authority is not the one
/// it was written against.
///
/// # Panics
///
/// Panics when Tiler's own governed signatures violate their governed
/// structural bound, which is a defect in this crate rather than a caller error.
pub(crate) fn install_governed_index_access(
    builder: &mut LoweringCapabilityRegistryBuilder,
    substituted: &[OpKey],
) -> Result<(), LoweringRegistryError> {
    let capabilities =
        governed_index_access_capabilities().expect("the governed signatures are well formed");
    for capability in capabilities {
        if substituted.contains(&capability.operation) {
            continue;
        }
        capability.register(builder)?;
    }
    Ok(())
}

/// One shipped index-access capability, before it is registered.
///
/// Keeping the four descriptors addressable lets a caller compose a registry
/// from a chosen subset of them, which is how an external provider substitutes
/// for one governed family without re-implementing the other three.
pub(crate) struct GovernedIndexAccess {
    provider: ProviderIdentity,
    operation: OpKey,
    signature: LoweringSignature,
    emitted: Vec<ScalarOpKey>,
    implementation: Arc<dyn IndexAccessLoweringProvider>,
}

impl GovernedIndexAccess {
    /// Returns the semantic family this capability lowers.
    #[cfg(test)]
    pub(crate) const fn operation(&self) -> &OpKey {
        &self.operation
    }

    /// Registers this capability on a composed builder.
    ///
    /// # Errors
    ///
    /// Returns [`LoweringRegistryError`] when the builder refuses it, which for
    /// a governed capability means the composed authority is not the one it was
    /// written against.
    pub(crate) fn register(
        self,
        builder: &mut LoweringCapabilityRegistryBuilder,
    ) -> Result<(), LoweringRegistryError> {
        let revision = LoweringCapabilityRevision::new(GOVERNED_CAPABILITY_REVISION)
            .expect("the governed capability revision is nonzero");
        builder.register_index_access(
            self.provider,
            self.operation,
            self.signature,
            &self.emitted,
            revision,
            self.implementation,
        )
    }
}

/// Returns the four shipped index-access capabilities in canonical family order.
///
/// # Errors
///
/// Returns [`GovernedRegistryError`] when a governed signature exceeds its
/// governed structural bound.
pub(crate) fn governed_index_access_capabilities()
-> Result<[GovernedIndexAccess; 4], GovernedRegistryError> {
    let f32_type = F32::resolved_type();
    let pointwise =
        || LoweringSignature::new([f32_type.clone(), f32_type.clone()], [f32_type.clone()]);
    Ok([
        GovernedIndexAccess {
            provider: governed_provider("constant-f32"),
            operation: constant_f32_op(),
            signature: LoweringSignature::new([], [f32_type.clone()])?,
            emitted: vec![constant_f32_scalar_op()],
            implementation: Arc::new(GovernedConstantF32),
        },
        GovernedIndexAccess {
            provider: governed_provider("multiply-f32"),
            operation: multiply_f32_op(),
            signature: pointwise()?,
            emitted: vec![multiply_f32_scalar_op()],
            implementation: Arc::new(GovernedPointwiseF32 {
                scalar: PointwiseScalar::Multiply,
            }),
        },
        GovernedIndexAccess {
            provider: governed_provider("add-f32"),
            operation: add_f32_op(),
            signature: pointwise()?,
            emitted: vec![add_f32_scalar_op()],
            implementation: Arc::new(GovernedPointwiseF32 {
                scalar: PointwiseScalar::Add,
            }),
        },
        GovernedIndexAccess {
            provider: governed_provider("strict-serial-sum-f32"),
            operation: strict_serial_sum_f32_op(),
            signature: LoweringSignature::new([f32_type.clone()], [f32_type])?,
            // One capability lowers every shape of the family, so the declared
            // set is the union over shapes, not what any one occurrence reaches:
            // an empty reduced domain reaches only the identity constant, a lone
            // contributor only the result-boundary canonicalization, and a
            // longer one the add. Refinement checks containment, which is what
            // makes that union the right declaration.
            emitted: vec![
                constant_f32_scalar_op(),
                add_f32_scalar_op(),
                canonicalize_nan_f32_scalar_op(),
            ],
            implementation: Arc::new(GovernedStrictSerialSumF32),
        },
    ])
}

/// Returns the governed lowering provider identity for one family.
fn governed_provider(family: &str) -> ProviderIdentity {
    ProviderIdentity::new("tiler", format!("governed-index-access.{family}"), 1)
        .expect("the governed lowering provider identity is valid")
}

/// Emits the rank-zero region realizing one `tiler.constant-f32` occurrence.
struct GovernedConstantF32;

impl IndexAccessLoweringProvider for GovernedConstantF32 {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        let occurrence = context.occurrence();
        if !occurrence.inputs().is_empty() {
            return Err(occurrence_error("constant-operand-arity"));
        }
        let [result] = occurrence.results() else {
            return Err(occurrence_error("constant-result-arity"));
        };
        if result.shape().rank() != 0 {
            return Err(occurrence_error("constant-result-rank"));
        }
        let attributes = constant_scalar_attributes(occurrence.attributes())?;
        let value_type = result.value_type().clone();
        let shape = result.shape().clone();
        let output = context.output_tensor(value_type, shape)?;
        let constant = context.apply(constant_f32_scalar_op(), attributes, &[])?;
        let value = single_result(&constant, "constant")?;
        let write = context.write(output, &[], &[])?;
        context.output(write, value)
    }
}

/// Which governed per-point scalar a pointwise lowering applies.
#[derive(Clone, Copy)]
enum PointwiseScalar {
    Multiply,
    Add,
}

impl PointwiseScalar {
    fn key(self) -> ScalarOpKey {
        match self {
            Self::Multiply => multiply_f32_scalar_op(),
            Self::Add => add_f32_scalar_op(),
        }
    }

    const fn rule(self) -> &'static str {
        match self {
            Self::Multiply => "multiply",
            Self::Add => "add",
        }
    }
}

/// Emits the elementwise region realizing one binary pointwise `f32` occurrence.
///
/// Exactly one broadcast form is supported: a rank-zero operand read once and
/// applied at every point. Any other rank mismatch is rejected explicitly rather
/// than approximated by an implicit alignment rule the semantic contract has not
/// stated.
struct GovernedPointwiseF32 {
    scalar: PointwiseScalar,
}

impl IndexAccessLoweringProvider for GovernedPointwiseF32 {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        let occurrence = context.occurrence();
        let [result] = occurrence.results() else {
            return Err(occurrence_error("pointwise-result-arity"));
        };
        if occurrence.operands().len() != 2 {
            return Err(occurrence_error("pointwise-operand-arity"));
        }
        let shape = result.shape().clone();
        let result_type = result.value_type().clone();
        let inputs: Vec<_> = occurrence.inputs().to_vec();
        let operands = occurrence.operands().to_vec();

        let dimensions = declare_parallel_domain(context, &shape)?;
        let coordinates = dimension_expressions(context, &dimensions)?;
        let mut tensors = Vec::with_capacity(inputs.len());
        for input in &inputs {
            tensors.push(context.input_tensor(input.value_type().clone(), input.shape().clone())?);
        }
        let mut values = Vec::with_capacity(operands.len());
        for position in &operands {
            let boundary = &inputs[*position];
            let tensor = tensors[*position];
            let value = if boundary.shape() == &shape {
                context.read(tensor, &dimensions, &coordinates)?
            } else if boundary.shape().rank() == 0 {
                context.read(tensor, &[], &[])?
            } else {
                return Err(occurrence_error("pointwise-broadcast"));
            };
            values.push(value);
        }
        let applied = context.apply(self.scalar.key(), ScalarAttributes::empty(), &values)?;
        let value = single_result(&applied, self.scalar.rule())?;
        let output = context.output_tensor(result_type, shape)?;
        let write = context.write(output, &dimensions, &coordinates)?;
        context.output(write, value)
    }
}

/// Emits the region realizing one `tiler.strict-serial-sum-f32` occurrence.
///
/// The fold seeds with the *first* contributor and combines the remaining ones
/// in ascending lexicographic order. Seeding with a `+0.0` identity instead
/// would be observably wrong: `0.0 + (-0.0)` is `+0.0`, so a single-element
/// reduction over `-0.0` would lose its sign. An empty reduced domain is the one
/// case whose result is the `+0.0` identity, and it is emitted as such.
///
/// The reduction also canonicalizes at its result boundary, which ADR 0055 and
/// the numerical contract require "even when the contributor sequence is a
/// singleton". A lone contributor is exactly where that rule bites: no combine
/// has run, so nothing else has replaced a non-canonical NaN payload. It is
/// applied as the `canonicalize-nan-f32` conversion rather than an arithmetic
/// step, because every arithmetic realization available here — adding the
/// `+0.0` identity in particular — would perturb a signed zero.
struct GovernedStrictSerialSumF32;

impl IndexAccessLoweringProvider for GovernedStrictSerialSumF32 {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        let plan = SumPlan::derive(context.occurrence())?;
        let kept = declare_kept_domain(context, &plan)?;
        let kept_coordinates = dimension_expressions(context, &kept)?;
        let input = context.input_tensor(plan.value_type.clone(), plan.input_shape.clone())?;
        let output = context.output_tensor(plan.value_type.clone(), plan.output_shape.clone())?;

        let total = if plan.reduced_points == 0 {
            plan.fold_empty(context, input, &kept, &kept_coordinates)?
        } else {
            let seed = plan.read_contributor(context, input, &kept, &kept_coordinates, None)?;
            if plan.reduced_points == 1 {
                // The lone contributor is the whole strict-serial value, and it
                // is the one boundary value no combine has canonicalized. Every
                // other path ends in the governed add, which canonicalizes its
                // own result, so this is the only place the boundary rule has
                // work to do.
                let canonical = context.apply(
                    canonicalize_nan_f32_scalar_op(),
                    ScalarAttributes::empty(),
                    &[seed],
                )?;
                single_result(&canonical, "reduction-result-canonicalization")?
            } else {
                plan.fold_tail(context, input, &kept, &kept_coordinates, seed)?
            }
        };
        let write = context.write(output, &kept, &kept_coordinates)?;
        context.output(write, total)
    }
}

/// The exact reduction geometry one serial-sum occurrence describes.
struct SumPlan {
    value_type: ResolvedValueType,
    input_shape: Shape,
    output_shape: Shape,
    /// Per input axis, whether the axis is reduced.
    reduced: Vec<bool>,
    /// Row-major strides of the reduced sub-shape, per reduced axis in order.
    reduced_strides: Vec<u64>,
    /// Extents of the reduced sub-shape, per reduced axis in order.
    reduced_extents: Vec<u64>,
    /// Number of points in the reduced sub-shape.
    reduced_points: u64,
}

impl SumPlan {
    fn derive(occurrence: &LoweredOccurrence) -> Result<Self, LoweringEmitError> {
        let ([input], [result]) = (occurrence.inputs(), occurrence.results()) else {
            return Err(occurrence_error("sum-arity"));
        };
        if occurrence.operands() != [0] {
            return Err(occurrence_error("sum-operand-binding"));
        }
        let axes = reduction_axes(occurrence.attributes())?;
        let input_shape = input.shape().clone();
        let mut reduced = vec![false; input_shape.rank()];
        for axis in &axes {
            let index = usize::try_from(axis.get()).map_err(|_| occurrence_error("sum-axis"))?;
            let Some(slot) = reduced.get_mut(index) else {
                return Err(occurrence_error("sum-axis-range"));
            };
            if std::mem::replace(slot, true) {
                return Err(occurrence_error("sum-axis-duplicate"));
            }
        }
        if &input_shape.without_axes(&axes) != result.shape() {
            return Err(occurrence_error("sum-result-shape"));
        }
        let reduced_extents: Vec<u64> = input_shape
            .extents()
            .iter()
            .zip(&reduced)
            .filter(|(_, reduced)| **reduced)
            .map(|(extent, _)| extent.get())
            .collect();
        let mut reduced_strides = vec![0_u64; reduced_extents.len()];
        let mut stride = 1_u64;
        for (position, extent) in reduced_extents.iter().enumerate().rev() {
            reduced_strides[position] = stride;
            stride = stride
                .checked_mul(*extent)
                .ok_or_else(|| occurrence_error("sum-reduced-extent-overflow"))?;
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

    /// Reads one contributor at the reduced offset `tail + 1`, or at zero.
    fn read_contributor(
        &self,
        context: &mut IndexAccessLoweringContext<'_>,
        input: tiler_ir::index::TensorId,
        kept: &[tiler_ir::index::DimensionId],
        kept_coordinates: &[IndexExprId],
        tail: Option<tiler_ir::index::DimensionId>,
    ) -> Result<ScalarValueId, LoweringEmitError> {
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
                let coordinate = match offset {
                    Some(offset) => self.decode_reduced(context, offset, reduced_position)?,
                    None => zero,
                };
                coordinates.push(coordinate);
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

    /// Decodes one reduced axis coordinate from a linearized reduced offset.
    fn decode_reduced(
        &self,
        context: &mut IndexAccessLoweringContext<'_>,
        offset: IndexExprId,
        position: usize,
    ) -> Result<IndexExprId, LoweringEmitError> {
        let stride = self.reduced_strides[position];
        let extent = self.reduced_extents[position];
        // The leading reduced axis needs no wrap: the offset is already below
        // the product of every reduced extent.
        let wrapped = if position == 0 {
            offset
        } else {
            let modulus = stride
                .checked_mul(extent)
                .ok_or_else(|| occurrence_error("sum-reduced-extent-overflow"))?;
            context.modulo(offset, SourcedExtent::Static(Extent::new(modulus)))?
        };
        if stride == 1 {
            Ok(wrapped)
        } else {
            Ok(context.floor_div(wrapped, SourcedExtent::Static(Extent::new(stride)))?)
        }
    }

    /// Folds zero contributors over the empty reduced domain onto `+0.0`.
    ///
    /// The result is the reduction identity, which is the one case where the
    /// identity is the correct answer rather than a sign-losing substitute for a
    /// first contributor. The operand is still read — over the vacuous domain —
    /// because the occurrence declares it and a region that never touched its
    /// declared input would not realize that occurrence.
    fn fold_empty(
        &self,
        context: &mut IndexAccessLoweringContext<'_>,
        input: tiler_ir::index::TensorId,
        kept: &[tiler_ir::index::DimensionId],
        kept_coordinates: &[IndexExprId],
    ) -> Result<ScalarValueId, LoweringEmitError> {
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
        let attributes = f32_constant_attributes(0.0_f32.to_bits())?;
        let identity = context.apply(constant_f32_scalar_op(), attributes, &[])?;
        let identity = single_result(&identity, "reduction-identity")?;
        let folded = context.reduce(&reduced_dimensions, &[identity], &[contributor], |body| {
            let state = body
                .state(0)
                .expect("the reduction declares one state parameter");
            let contributor = body
                .contributor(0)
                .expect("the reduction declares one contributor parameter");
            let accumulated = body.apply(
                add_f32_scalar_op(),
                ScalarAttributes::empty(),
                &[state, contributor],
            )?;
            let accumulated = accumulated
                .get(0)
                .expect("the governed add contract produces one result");
            body.yield_values(&[accumulated])
        })?;
        single_result(&folded, "reduction")
    }

    /// Folds contributors `1..reduced_points` onto the seed in ascending order.
    fn fold_tail(
        &self,
        context: &mut IndexAccessLoweringContext<'_>,
        input: tiler_ir::index::TensorId,
        kept: &[tiler_ir::index::DimensionId],
        kept_coordinates: &[IndexExprId],
        seed: ScalarValueId,
    ) -> Result<ScalarValueId, LoweringEmitError> {
        let tail = context.dimension(
            DomainRole::Reduction,
            Extent::new(self.reduced_points.saturating_sub(1)),
        )?;
        let contributor =
            self.read_contributor(context, input, kept, kept_coordinates, Some(tail))?;
        let folded = context.reduce(&[tail], &[seed], &[contributor], |body| {
            let state = body
                .state(0)
                .expect("the reduction declares one state parameter");
            let contributor = body
                .contributor(0)
                .expect("the reduction declares one contributor parameter");
            let accumulated = body.apply(
                add_f32_scalar_op(),
                ScalarAttributes::empty(),
                &[state, contributor],
            )?;
            let accumulated = accumulated
                .get(0)
                .expect("the governed add contract produces one result");
            body.yield_values(&[accumulated])
        })?;
        single_result(&folded, "reduction")
    }
}

/// Declares one parallel dimension per axis of `shape`, in axis order.
fn declare_parallel_domain(
    context: &mut IndexAccessLoweringContext<'_>,
    shape: &Shape,
) -> Result<Vec<tiler_ir::index::DimensionId>, LoweringEmitError> {
    let mut dimensions = Vec::with_capacity(shape.rank());
    for extent in shape.extents() {
        dimensions.push(context.dimension(DomainRole::Parallel, *extent)?);
    }
    Ok(dimensions)
}

/// Declares one parallel dimension per kept axis of a reduction, in axis order.
fn declare_kept_domain(
    context: &mut IndexAccessLoweringContext<'_>,
    plan: &SumPlan,
) -> Result<Vec<tiler_ir::index::DimensionId>, LoweringEmitError> {
    let mut dimensions = Vec::with_capacity(plan.output_shape.rank());
    for (extent, reduced) in plan.input_shape.extents().iter().zip(&plan.reduced) {
        if !reduced {
            dimensions.push(context.dimension(DomainRole::Parallel, *extent)?);
        }
    }
    Ok(dimensions)
}

fn dimension_expressions(
    context: &mut IndexAccessLoweringContext<'_>,
    dimensions: &[tiler_ir::index::DimensionId],
) -> Result<Vec<IndexExprId>, LoweringEmitError> {
    let mut expressions = Vec::with_capacity(dimensions.len());
    for dimension in dimensions {
        expressions.push(context.dimension_expr(*dimension)?);
    }
    Ok(expressions)
}

fn single_result(
    results: &tiler_ir::index::ScalarResults,
    rule: &'static str,
) -> Result<ScalarValueId, LoweringEmitError> {
    results.get(0).ok_or_else(|| occurrence_error(rule))
}

/// Forwards an occurrence's `f32` constant bits into governed scalar attributes.
fn constant_scalar_attributes(
    attributes: &OperationAttributes,
) -> Result<ScalarAttributes, LoweringEmitError> {
    let Some(bits) = attributes.get(F32_CONSTANT_BITS_ATTRIBUTE) else {
        return Err(occurrence_error("constant-bits-missing"));
    };
    let CanonicalValueView::FloatBits(view) = bits.view() else {
        return Err(occurrence_error("constant-bits-kind"));
    };
    if view.format() != &f32_format() {
        return Err(occurrence_error("constant-bits-format"));
    }
    scalar_attributes(bits.clone())
}

fn f32_constant_attributes(bits: u32) -> Result<ScalarAttributes, LoweringEmitError> {
    let value = CanonicalValue::float_bits(f32_format(), bits.to_be_bytes())
        .map_err(|_| occurrence_error("constant-bits-encoding"))?;
    scalar_attributes(value)
}

fn scalar_attributes(bits: CanonicalValue) -> Result<ScalarAttributes, LoweringEmitError> {
    let record = CanonicalValue::record([CanonicalField::new(F32_CONSTANT_BITS_ATTRIBUTE, bits)])
        .map_err(|_| occurrence_error("constant-bits-encoding"))?;
    ScalarAttributes::new(record).map_err(|_| occurrence_error("constant-bits-encoding"))
}

fn f32_format() -> TypeKey {
    TypeKey::new("tiler", "f32", 1).expect("the governed f32 format key is valid")
}

/// Reads the strictly ascending reduction axes an occurrence declares.
fn reduction_axes(attributes: &OperationAttributes) -> Result<Vec<Axis>, LoweringEmitError> {
    let Some(value) = attributes.get(REDUCTION_AXES_ATTRIBUTE) else {
        return Err(occurrence_error("sum-axes-missing"));
    };
    let CanonicalValueView::Sequence(values) = value.view() else {
        return Err(occurrence_error("sum-axes-kind"));
    };
    let mut axes = Vec::with_capacity(values.len());
    for value in values {
        let CanonicalValueView::Unsigned {
            width: CanonicalIntegerWidth::Bits32,
            bits,
        } = value.view()
        else {
            return Err(occurrence_error("sum-axes-element"));
        };
        let reduced = u32::try_from(bits).map_err(|_| occurrence_error("sum-axes-width"))?;
        axes.push(Axis::new(reduced));
    }
    Ok(axes)
}

fn occurrence_error(rule: &'static str) -> LoweringEmitError {
    LoweringEmitError::Occurrence { rule }
}

#[cfg(test)]
mod tests {
    use super::{governed_lowering_capabilities, governed_scalars};
    use crate::capability::LoweringSignature;
    use crate::legality::{
        IndexRefinement, NumericalContractIdentity, OccurrenceOperand, OccurrenceResult,
        OccurrenceValueId, SemanticOccurrence, SemanticOccurrenceIdentity, refine_index_region,
    };
    use tiler_ir::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS;
    use tiler_ir::semantic::{
        CanonicalField, CanonicalValue, F32, F32_CONSTANT_BITS_ATTRIBUTE, OpKey,
        OperationAttributes, OperationEffect, REDUCTION_AXES_ATTRIBUTE, ResolvedValueType, TypeKey,
        add_f32_op, constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
    };
    use tiler_ir::shape::{Axis, Shape};
    use tiler_reference::{
        FloatBitOrder, FrozenReferenceRegistry, FrozenScalarReferenceRegistry,
        IndexRegionAuthority, IndexRegionEvaluator, IndexRegionInput, ReferenceElement, Tensor,
        TensorPayloadView,
    };

    fn f32_type() -> ResolvedValueType {
        F32::resolved_type()
    }

    fn contract() -> NumericalContractIdentity {
        NumericalContractIdentity::from_key("tiler.strict-f32.v1")
    }

    fn constant_attributes(bits: u32) -> OperationAttributes {
        OperationAttributes::new([CanonicalField::new(
            F32_CONSTANT_BITS_ATTRIBUTE,
            CanonicalValue::float_bits(
                TypeKey::new("tiler", "f32", 1).unwrap(),
                bits.to_be_bytes(),
            )
            .unwrap(),
        )])
        .unwrap()
    }

    fn axes_attributes(axes: &[u32]) -> OperationAttributes {
        OperationAttributes::new([CanonicalField::new(
            REDUCTION_AXES_ATTRIBUTE,
            CanonicalValue::sequence(axes.iter().copied().map(CanonicalValue::unsigned_u32))
                .unwrap(),
        )])
        .unwrap()
    }

    /// Refines one occurrence through the governed registry.
    fn refine(
        operation: OpKey,
        operands: Vec<OccurrenceOperand>,
        results: Vec<OccurrenceResult>,
        attributes: OperationAttributes,
    ) -> IndexRefinement {
        let scalars = governed_scalars().unwrap();
        let registry = governed_lowering_capabilities(&scalars).unwrap();
        let occurrence = SemanticOccurrence::new(
            operation,
            operands,
            results,
            attributes,
            OperationEffect::Pure,
            contract(),
            SemanticOccurrenceIdentity::from_bytes(b"governed-fixture".to_vec()),
        );
        let signature = LoweringSignature::new(
            occurrence
                .operands()
                .iter()
                .map(|operand| operand.value_type().clone()),
            occurrence
                .results()
                .iter()
                .map(|result| result.value_type().clone()),
        )
        .unwrap();
        let resolved = registry
            .resolve_index_access(occurrence.operation(), &signature)
            .unwrap();
        refine_index_region(&resolved, &occurrence, &scalars)
            .unwrap_or_else(|error| panic!("governed lowering must refine: {error:?}"))
            .into_refined()
            .expect("governed lowering discharges every index-domain predicate")
    }

    #[test]
    fn the_governed_constant_lowering_refines_its_occurrence() {
        let refinement = refine(
            constant_f32_op(),
            Vec::new(),
            vec![OccurrenceResult::new(f32_type(), Shape::new([]))],
            constant_attributes(2.0_f32.to_bits()),
        );
        assert!(refinement.operand_bindings().is_empty());
        assert_eq!(refinement.result_bindings().len(), 1);
    }

    /// Two constant occurrences differing only in their bits are two lowerings.
    #[test]
    fn constants_with_different_bits_produce_different_regions() {
        let region = |bits: u32| {
            refine(
                constant_f32_op(),
                Vec::new(),
                vec![OccurrenceResult::new(f32_type(), Shape::new([]))],
                constant_attributes(bits),
            )
            .content()
            .region_identity()
            .clone()
        };
        assert_ne!(region(2.0_f32.to_bits()), region(1.0_f32.to_bits()));
    }

    #[test]
    fn the_governed_pointwise_lowerings_refine_a_scalar_broadcast() {
        for operation in [multiply_f32_op(), add_f32_op()] {
            let tensor = Shape::from_dims([2, 3]);
            let refinement = refine(
                operation,
                vec![
                    OccurrenceOperand::new(OccurrenceValueId(0), f32_type(), tensor.clone()),
                    OccurrenceOperand::new(OccurrenceValueId(1), f32_type(), Shape::new([])),
                ],
                vec![OccurrenceResult::new(f32_type(), tensor)],
                OperationAttributes::empty(),
            );
            assert_eq!(refinement.operand_bindings().len(), 2);
            assert_ne!(
                refinement.operand_bindings()[0].input_tensor(),
                refinement.operand_bindings()[1].input_tensor()
            );
        }
    }

    #[test]
    fn the_governed_serial_sum_refines_trailing_leading_and_interior_axes() {
        for (dims, axes) in [
            (vec![2_u64, 3], vec![1_u32]),
            (vec![3, 2], vec![0]),
            (vec![2, 3, 2], vec![1]),
            (vec![2, 3, 2], vec![0, 2]),
            (vec![2, 1], vec![1]),
            (vec![2, 0], vec![1]),
        ] {
            let input = Shape::try_from_dims(dims).unwrap();
            let output =
                input.without_axes(&axes.iter().copied().map(Axis::new).collect::<Vec<_>>());
            let refinement = refine(
                strict_serial_sum_f32_op(),
                vec![OccurrenceOperand::new(
                    OccurrenceValueId(0),
                    f32_type(),
                    input,
                )],
                vec![OccurrenceResult::new(f32_type(), output)],
                axes_attributes(&axes),
            );
            assert_eq!(refinement.operand_bindings().len(), 1);
            assert_eq!(refinement.result_bindings().len(), 1);
        }
    }

    // ---------------------------------------------------------------------
    // The emitted regions, executed by the independent oracle
    // ---------------------------------------------------------------------
    //
    // Every case above proves a governed lowering *refines* its occurrence:
    // structure, interface, reached authority, ownership. None of them runs the
    // arithmetic. `crates/tiler-reference/tests/governed_scalar_reference.rs`
    // does run it, but against hand-written *mirrors* of these emissions, so a
    // mirror that drifted from `governed.rs` would keep passing.
    //
    // These cases close that gap by executing the region
    // `refine_index_region` actually returned. They live here rather than in
    // `tiler-reference` because the oracle must not depend on the compiler:
    // `tiler-reference` depends only on `tiler-ir`, and inverting that would
    // break the layering `AGENTS.md` requires. `tiler-compiler` dev-depends on
    // `tiler-reference`, so this is the one direction that composes.
    //
    // Comparison is on exact bit patterns, not `f32` equality: `-0.0 == 0.0` is
    // true and a NaN equals nothing, so float comparison would silently accept
    // exactly the results a numerical contract exists to pin.

    /// The least positive `f32` subnormal.
    const LEAST_SUBNORMAL: u32 = 0x0000_0001;
    /// A quiet NaN whose payload is *not* the canonical arithmetic pattern.
    const NONCANONICAL_NAN: u32 = 0x7fc0_1234;

    fn bit_tensor(shape: Shape, bits: &[u32]) -> Tensor {
        Tensor::dense(
            f32_type(),
            shape,
            bits.iter()
                .map(|value| {
                    ReferenceElement::from_float_bits(
                        value.to_be_bytes(),
                        FloatBitOrder::MostSignificantByteFirst,
                    )
                    .expect("the operand is a valid f32 pattern")
                })
                .collect(),
        )
        .expect("the tensor is well formed")
    }

    fn output_bits(tensor: &Tensor) -> Vec<u32> {
        let TensorPayloadView::Dense(elements) = tensor.payload() else {
            panic!("expected a dense f32 tensor")
        };
        elements
            .iter()
            .map(|value| {
                u32::from_be_bytes(
                    <[u8; 4]>::try_from(value.as_bytes()).expect("an f32 element is four bytes"),
                )
            })
            .collect()
    }

    /// Executes one refined governed region through the standard scalar oracle.
    ///
    /// The evaluator is built from `FrozenScalarReferenceRegistry::standard()`,
    /// the same governed profile the lowerings emit against, so an operation a
    /// lowering emits but the oracle cannot execute is a failure here rather
    /// than a silently skipped check.
    fn evaluate_refined(refinement: &IndexRefinement, inputs: &[(usize, &Tensor)]) -> Vec<u32> {
        let scalars = governed_scalars().expect("the governed scalar authority composes");
        let evaluator = IndexRegionEvaluator::new(
            FrozenReferenceRegistry::standard().expect("the governed value profile composes"),
            FrozenScalarReferenceRegistry::standard().expect("the governed scalar oracle composes"),
        );
        let bound: Vec<IndexRegionInput<'_>> = inputs
            .iter()
            .map(|(operand, value)| {
                IndexRegionInput::new(
                    refinement.operand_bindings()[*operand].input_tensor(),
                    value,
                )
            })
            .collect();
        let evaluation = evaluator
            .evaluate(
                refinement.region(),
                IndexRegionAuthority::new(&scalars),
                &bound,
            )
            .expect("the governed region executes on the oracle");
        output_bits(&evaluation.outputs()[0])
    }

    /// The constant lowering emits the exact declared payload, NaN included.
    #[test]
    fn the_governed_constant_region_reproduces_its_declared_bits() {
        for bits in [
            0x3f80_0000,
            (-0.0_f32).to_bits(),
            LEAST_SUBNORMAL,
            NONCANONICAL_NAN,
        ] {
            let refinement = refine(
                constant_f32_op(),
                Vec::new(),
                vec![OccurrenceResult::new(f32_type(), Shape::new([]))],
                constant_attributes(bits),
            );
            assert_eq!(
                evaluate_refined(&refinement, &[]),
                vec![bits],
                "a declared constant is bit-preserving, including a non-canonical NaN",
            );
        }
    }

    /// Multiply and add canonicalize every NaN they produce and preserve every
    /// other payload, in the region the lowering actually emitted.
    #[test]
    fn the_governed_pointwise_regions_execute_their_declared_contract() {
        let shape = Shape::from_dims([4]);
        let left = bit_tensor(
            shape.clone(),
            &[
                0x3f80_0000,
                (-0.0_f32).to_bits(),
                LEAST_SUBNORMAL,
                NONCANONICAL_NAN,
            ],
        );
        let right = bit_tensor(
            shape.clone(),
            &[0x4000_0000, 0x3f80_0000, 0x3f80_0000, 0x3f80_0000],
        );

        let multiply = refine(
            multiply_f32_op(),
            vec![
                OccurrenceOperand::new(OccurrenceValueId(0), f32_type(), shape.clone()),
                OccurrenceOperand::new(OccurrenceValueId(1), f32_type(), shape.clone()),
            ],
            vec![OccurrenceResult::new(f32_type(), shape.clone())],
            OperationAttributes::new([]).expect("an empty attribute set is valid"),
        );
        assert_eq!(
            evaluate_refined(&multiply, &[(0, &left), (1, &right)]),
            vec![
                0x4000_0000,
                (-0.0_f32).to_bits(),
                LEAST_SUBNORMAL,
                CANONICAL_F32_ARITHMETIC_NAN_BITS,
            ],
            "1*2=2; -0*1 keeps its sign; a subnormal survives; a NaN canonicalizes",
        );

        let add = refine(
            add_f32_op(),
            vec![
                OccurrenceOperand::new(OccurrenceValueId(0), f32_type(), shape.clone()),
                OccurrenceOperand::new(OccurrenceValueId(1), f32_type(), shape.clone()),
            ],
            vec![OccurrenceResult::new(f32_type(), shape)],
            OperationAttributes::new([]).expect("an empty attribute set is valid"),
        );
        assert_eq!(
            evaluate_refined(&add, &[(0, &left), (1, &right)])[3],
            CANONICAL_F32_ARITHMETIC_NAN_BITS,
            "an add over a non-canonical NaN produces the canonical payload",
        );
    }

    /// The strict serial sum's emitted fold, executed on exceptional values.
    ///
    /// The single-contributor case is included rather than excluded. The
    /// ordering note on this ticket deferred it to
    /// `reconcile-single-contributor-strict-sum-nan-canonicalization`, which is
    /// now `done`: a lone contributor canonicalizes at the reduction's result
    /// boundary, so the three implementations agree and the case discriminates.
    #[test]
    fn the_governed_serial_sum_region_executes_its_declared_contract() {
        let reduce = |extent: u64, bits: &[u32]| {
            let input = Shape::from_dims([extent]);
            let refinement = refine(
                strict_serial_sum_f32_op(),
                vec![OccurrenceOperand::new(
                    OccurrenceValueId(0),
                    f32_type(),
                    input.clone(),
                )],
                vec![OccurrenceResult::new(f32_type(), Shape::new([]))],
                axes_attributes(&[0]),
            );
            let tensor = bit_tensor(input, bits);
            evaluate_refined(&refinement, &[(0, &tensor)])
        };

        assert_eq!(
            reduce(3, &[0x3f80_0000, 0x4000_0000, 0x4040_0000]),
            vec![0x40c0_0000],
            "1 + 2 + 3 = 6",
        );
        assert_eq!(
            reduce(1, &[NONCANONICAL_NAN]),
            vec![CANONICAL_F32_ARITHMETIC_NAN_BITS],
            "a lone contributor canonicalizes at the reduction result boundary",
        );
        assert_eq!(
            reduce(1, &[(-0.0_f32).to_bits()]),
            vec![(-0.0_f32).to_bits()],
            "the boundary rule is a conversion, not an addition: -0.0 survives",
        );
        assert_eq!(
            reduce(2, &[LEAST_SUBNORMAL, 0x0000_0000]),
            vec![LEAST_SUBNORMAL],
            "a subnormal survives a strict-preserving fold",
        );
    }
}

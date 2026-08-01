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
    canonicalize_nan_f32_scalar_op, constant_f32_scalar_op, divide_f32_scalar_op,
    exp_f32_scalar_op, multiply_f32_scalar_op,
};
use tiler_ir::semantic::{
    BROADCAST_AXIS_MAPPING_ATTRIBUTE, BroadcastAxisMapping, BroadcastAxisSource,
    CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, CanonicalField, CanonicalIntegerWidth, CanonicalValue,
    CanonicalValueView, ContractionIndex, ContractionIndexStructure, F32,
    F32_CONSTANT_BITS_ATTRIBUTE, OpKey, OperationAttributes, ProviderIdentity,
    REDUCTION_AXES_ATTRIBUTE, REINDEX_MAPPING_ATTRIBUTE, ReindexForm, ReindexFormKind,
    ResolvedValueType, TypeKey, add_f32_op, broadcast_f32_op, constant_f32_op, multiply_f32_op,
    reindex_f32_op, silu_f32_op, strict_serial_sum_f32_op, strict_tensor_contraction_f32_op,
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

/// Returns the eight shipped index-access capabilities in canonical family order.
///
/// # Errors
///
/// Returns [`GovernedRegistryError`] when a governed signature exceeds its
/// governed structural bound.
pub(crate) fn governed_index_access_capabilities()
-> Result<[GovernedIndexAccess; 8], GovernedRegistryError> {
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
            signature: LoweringSignature::new([f32_type.clone()], [f32_type.clone()])?,
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
        GovernedIndexAccess {
            provider: governed_provider("silu-f32"),
            operation: silu_f32_op(),
            signature: LoweringSignature::new([f32_type.clone()], [f32_type.clone()])?,
            // The complete set the lowering reaches, and the whole of it is a
            // positive claim about what the region emits: the negation's
            // multiply, the precise exponential, the addition of one, and the
            // division. `divide-f32` in particular is here rather than a
            // reciprocal and a second multiply, which is the substitution the
            // pinned reference forbids and which refinement's containment check
            // would otherwise have no way to see.
            emitted: vec![
                constant_f32_scalar_op(),
                multiply_f32_scalar_op(),
                exp_f32_scalar_op(),
                add_f32_scalar_op(),
                divide_f32_scalar_op(),
            ],
            implementation: Arc::new(GovernedSiluF32),
        },
        GovernedIndexAccess {
            provider: governed_provider("reindex-f32"),
            operation: reindex_f32_op(),
            signature: LoweringSignature::new([f32_type.clone()], [f32_type.clone()])?,
            // Deliberately empty, and not an omission. A reindex applies no
            // scalar operation at all: the value written is the value read, so
            // the emitted region reaches no scalar authority. Declaring one
            // anyway would make refinement's containment check pass over an
            // operation the region never emits, which is the reverse of what the
            // declaration is for.
            emitted: Vec::new(),
            implementation: Arc::new(GovernedReindexF32),
        },
        GovernedIndexAccess {
            provider: governed_provider("broadcast-f32"),
            operation: broadcast_f32_op(),
            signature: LoweringSignature::new([f32_type.clone()], [f32_type.clone()])?,
            emitted: Vec::new(),
            implementation: Arc::new(GovernedBroadcastF32),
        },
        GovernedIndexAccess {
            provider: governed_provider("strict-tensor-contraction-f32"),
            operation: strict_tensor_contraction_f32_op(),
            signature: LoweringSignature::new([f32_type.clone(), f32_type.clone()], [f32_type])?,
            // The union over shapes, like the serial sum's: a contracted space
            // with one point reaches only the product, and a longer one reaches
            // the add as well. There is deliberately no
            // `canonicalize-nan-f32` — unlike the serial sum, whose singleton
            // case commits a *raw load*, every value this lowering can commit is
            // the result of a governed arithmetic operation, and those
            // canonicalize their own results. Declaring one anyway would make
            // refinement's containment check pass over an operation the region
            // never emits.
            emitted: vec![multiply_f32_scalar_op(), add_f32_scalar_op()],
            implementation: Arc::new(GovernedStrictTensorContractionF32),
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

/// Emits the region realizing one `tiler::silu-f32@1` occurrence.
///
/// The emitted chain is the pinned reference read left to right:
/// `x * -1.0`, then the precise exponential, then `1.0 + e`, then `x / d`. Three
/// properties of that chain are load-bearing rather than incidental.
///
/// **The negation is a multiplication by `-1.0`, and it is exact.** IEEE-754
/// multiplication by negative one flips the sign of every operand — both zeros
/// and both infinities included — with no rounding, so it delivers exactly what
/// the reference's "exact sign manipulation" means. There is no negate scalar to
/// reach for and this does not need one.
///
/// **The divisor is `1.0 + e`, in that order.** Binary32 addition is commutative,
/// so the order is not observable here; it is written this way because the
/// reference is, and a reader comparing the two should not have to reconcile a
/// difference that has no consequence.
///
/// **The result is a division.** `x * (1.0 / d)` rounds twice and would be a
/// different binary32 function — measurably so at `0xc2b00000`, where the two
/// spellings differ by one ULP. The scalar vocabulary has no reciprocal key, so
/// the substitution is unstatable here rather than merely forbidden.
struct GovernedSiluF32;

impl IndexAccessLoweringProvider for GovernedSiluF32 {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        let occurrence = context.occurrence();
        let [result] = occurrence.results() else {
            return Err(occurrence_error("silu-result-arity"));
        };
        if occurrence.operands().len() != 1 || occurrence.inputs().len() != 1 {
            return Err(occurrence_error("silu-operand-arity"));
        }
        let shape = result.shape().clone();
        let result_type = result.value_type().clone();
        let boundary = occurrence.inputs()[0].clone();
        if boundary.shape() != &shape {
            return Err(occurrence_error("silu-elementwise-shape"));
        }

        let dimensions = declare_parallel_domain(context, &shape)?;
        let coordinates = dimension_expressions(context, &dimensions)?;
        let tensor = context.input_tensor(boundary.value_type().clone(), shape.clone())?;
        let argument = context.read(tensor, &dimensions, &coordinates)?;

        let negative_one = apply_one(
            context,
            constant_f32_scalar_op(),
            f32_constant_attributes(0xbf80_0000)?,
            &[],
            "silu-negative-one",
        )?;
        let negated = apply_one(
            context,
            multiply_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[argument, negative_one],
            "silu-negation",
        )?;
        let exponential = apply_one(
            context,
            exp_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[negated],
            "silu-exponential",
        )?;
        let one = apply_one(
            context,
            constant_f32_scalar_op(),
            f32_constant_attributes(0x3f80_0000)?,
            &[],
            "silu-one",
        )?;
        let divisor = apply_one(
            context,
            add_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[one, exponential],
            "silu-divisor",
        )?;
        let value = apply_one(
            context,
            divide_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[argument, divisor],
            "silu-division",
        )?;

        let output = context.output_tensor(result_type, shape)?;
        let write = context.write(output, &dimensions, &coordinates)?;
        context.output(write, value)
    }
}

/// Applies one scalar operation that produces exactly one result.
fn apply_one(
    context: &mut IndexAccessLoweringContext<'_>,
    key: ScalarOpKey,
    attributes: ScalarAttributes,
    operands: &[ScalarValueId],
    rule: &'static str,
) -> Result<ScalarValueId, LoweringEmitError> {
    let applied = context.apply(key, attributes, operands)?;
    single_result(&applied, rule)
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

/// Emits the region realizing one `tiler::strict-tensor-contraction-f32@1`
/// occurrence.
///
/// The `direct` realization: one iteration point per output element, folding its
/// own contracted sequence in ascending canonical order. Three properties are
/// load-bearing.
///
/// **The accumulator seeds at the first product, never at `+0.0`.** The two
/// differ observably on a vector whose every product is `-0.0`, where the seeded
/// fold returns `+0.0`; the registered family declares no seed, so seeding would
/// compute a contraction carrying an explicit `initial` — a different operation.
///
/// **The product and the sum round separately.** Each is its own governed scalar
/// application, so the single-rounding fused form the family declares forbidden
/// is not merely unselected here, it is unstatable: no scalar key in this
/// vocabulary fuses a multiply into an add.
///
/// **No result-boundary canonicalization is emitted, and its absence is
/// derived.** The serial sum needs one because its singleton case commits a raw
/// load. Every value this region can commit is a governed multiply's or add's
/// result, and those canonicalize their own results, so a conversion here would
/// be provably redundant — and would put an operation in the emitted set that
/// refinement's containment check could then never see fail.
struct GovernedStrictTensorContractionF32;

impl IndexAccessLoweringProvider for GovernedStrictTensorContractionF32 {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        let plan = ContractionPlan::derive(context.occurrence())?;
        let output = declare_parallel_domain(context, &plan.output_shape)?;
        let output_coordinates = dimension_expressions(context, &output)?;
        let mut tensors = Vec::with_capacity(plan.operand_shapes.len());
        for shape in &plan.operand_shapes {
            tensors.push(context.input_tensor(plan.value_type.clone(), shape.clone())?);
        }
        let result = context.output_tensor(plan.value_type.clone(), plan.output_shape.clone())?;

        let seed = plan.product(context, &tensors, &output, &output_coordinates, None)?;
        let total = if plan.contracted_points == 1 {
            seed
        } else {
            let tail = context.dimension(
                DomainRole::Reduction,
                Extent::new(plan.contracted_points.saturating_sub(1)),
            )?;
            let contributor =
                plan.product(context, &tensors, &output, &output_coordinates, Some(tail))?;
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
            single_result(&folded, "contraction")?
        };
        let write = context.write(result, &output, &output_coordinates)?;
        context.output(write, total)
    }
}

/// The exact contraction geometry one occurrence describes.
///
/// Every extent is re-derived from the structure and the operand shapes rather
/// than taken from the occurrence's declared result, and the derived output
/// shape is then required to equal that declaration: the semantic registry
/// already refused a malformed occurrence at construction, so a disagreement is
/// invalid state and this region must realize the occurrence it was handed
/// rather than its own derivation of one.
struct ContractionPlan {
    value_type: ResolvedValueType,
    operand_shapes: Vec<Shape>,
    output_shape: Shape,
    /// Per operand, per axis: whether the axis reads an output or a contracted
    /// coordinate, and at which position.
    sources: Vec<Vec<AxisSource>>,
    /// Row-major strides of the contracted space, per contracted position.
    contracted_strides: Vec<u64>,
    /// Extents of the contracted space, per contracted position.
    contracted_extents: Vec<u64>,
    /// Points of the contracted space; the fold length per output element.
    contracted_points: u64,
}

/// Which coordinate one operand axis reads.
#[derive(Clone, Copy)]
enum AxisSource {
    Output(usize),
    Contracted(usize),
}

impl ContractionPlan {
    fn derive(occurrence: &LoweredOccurrence) -> Result<Self, LoweringEmitError> {
        let ([left, right], [result]) = (occurrence.inputs(), occurrence.results()) else {
            return Err(occurrence_error("contraction-arity"));
        };
        if occurrence.operands() != [0, 1] {
            return Err(occurrence_error("contraction-operand-binding"));
        }
        let [field] = occurrence.attributes().fields() else {
            return Err(occurrence_error("contraction-attributes"));
        };
        if field.id() != CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE {
            return Err(occurrence_error("contraction-attributes"));
        }
        let structure = ContractionIndexStructure::from_canonical_value(field.value())
            .map_err(|_| occurrence_error("contraction-structure"))?;
        let boundaries = [left, right];
        if structure.operand_count() != boundaries.len() {
            return Err(occurrence_error("contraction-operand-count"));
        }

        let mut extents: Vec<(ContractionIndex, Extent)> = Vec::new();
        let mut operand_shapes = Vec::with_capacity(boundaries.len());
        for (tuple, boundary) in structure.operands().zip(boundaries) {
            let shape = boundary.shape().clone();
            if shape.rank() != tuple.len() {
                return Err(occurrence_error("contraction-rank"));
            }
            for (axis, index) in tuple.iter().enumerate() {
                let extent = shape.extents()[axis];
                match extents.iter().find(|(bound, _)| bound == index) {
                    Some((_, bound)) if *bound != extent => {
                        return Err(occurrence_error("contraction-extent"));
                    }
                    Some(_) => {}
                    None => extents.push((*index, extent)),
                }
            }
            operand_shapes.push(shape);
        }
        let shape_over = |indices: &[ContractionIndex]| -> Result<Shape, LoweringEmitError> {
            Shape::try_new(
                indices
                    .iter()
                    .map(|index| {
                        extents
                            .iter()
                            .find(|(bound, _)| bound == index)
                            .map(|(_, extent)| *extent)
                            .ok_or_else(|| occurrence_error("contraction-extent"))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            )
            .map_err(|_| occurrence_error("contraction-shape"))
        };
        let output_shape = shape_over(structure.output())?;
        if &output_shape != result.shape() {
            return Err(occurrence_error("contraction-result-shape"));
        }
        let contracted_shape = shape_over(structure.contracted())?;

        let mut sources = Vec::with_capacity(structure.operand_count());
        for tuple in structure.operands() {
            let mut operand = Vec::with_capacity(tuple.len());
            for index in tuple {
                let source = if let Some(position) =
                    structure.output().iter().position(|free| free == index)
                {
                    AxisSource::Output(position)
                } else if let Some(position) = structure
                    .contracted()
                    .iter()
                    .position(|summed| summed == index)
                {
                    AxisSource::Contracted(position)
                } else {
                    // Every operand index is free or contracted by the
                    // structure's own derivation, so this is invalid state
                    // rather than a caller error. Refused rather than assumed
                    // away.
                    return Err(occurrence_error("contraction-index"));
                };
                operand.push(source);
            }
            sources.push(operand);
        }

        let contracted_extents: Vec<u64> = contracted_shape
            .extents()
            .iter()
            .map(|extent| extent.get())
            .collect();
        let mut contracted_strides = vec![0_u64; contracted_extents.len()];
        let mut stride = 1_u64;
        for (position, extent) in contracted_extents.iter().enumerate().rev() {
            contracted_strides[position] = stride;
            stride = stride
                .checked_mul(*extent)
                .ok_or_else(|| occurrence_error("contraction-extent-overflow"))?;
        }
        // The one precondition. The registered family declares an empty
        // contracted domain refused rather than identity-valued, so there is no
        // value this region could commit for one.
        if stride == 0 {
            return Err(occurrence_error("contraction-empty-domain"));
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

    /// Emits the separately rounded product at contracted offset `tail + 1`, or
    /// at zero.
    fn product(
        &self,
        context: &mut IndexAccessLoweringContext<'_>,
        tensors: &[tiler_ir::index::TensorId],
        output: &[tiler_ir::index::DimensionId],
        output_coordinates: &[IndexExprId],
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
        let mut domain = output.to_vec();
        domain.extend(tail);
        let mut values = Vec::with_capacity(tensors.len());
        for (position, tensor) in tensors.iter().enumerate() {
            let mut coordinates = Vec::with_capacity(self.sources[position].len());
            for source in &self.sources[position] {
                coordinates.push(match source {
                    AxisSource::Output(axis) => *output_coordinates
                        .get(*axis)
                        .ok_or_else(|| occurrence_error("contraction-coordinate"))?,
                    AxisSource::Contracted(axis) => match offset {
                        Some(offset) => self.decode_contracted(context, offset, *axis)?,
                        None => zero,
                    },
                });
            }
            values.push(context.read(*tensor, &domain, &coordinates)?);
        }
        // One rounding for the product, which the governed multiply also
        // canonicalizes. The fused single-rounding form is the permission this
        // family declares forbidden, and no scalar key here can express it.
        //
        // The evaluation scope is implicit rather than declared: ADR 0087's
        // second structural rule puts every contracted index in *both* operands,
        // so both reads already range over the contracted dimension and the
        // product inherits it. Naming it again would be the duplicate
        // `apply_in` refuses.
        let applied =
            context.apply(multiply_f32_scalar_op(), ScalarAttributes::empty(), &values)?;
        single_result(&applied, "contraction-product")
    }

    /// Decodes one contracted coordinate from a linearized contracted offset.
    fn decode_contracted(
        &self,
        context: &mut IndexAccessLoweringContext<'_>,
        offset: IndexExprId,
        position: usize,
    ) -> Result<IndexExprId, LoweringEmitError> {
        let stride = self.contracted_strides[position];
        let extent = self.contracted_extents[position];
        // The leading contracted axis needs no wrap: the offset is already below
        // the product of every contracted extent. The same wrap-then-divide
        // convention the serial sum's reduced-offset decode uses.
        let wrapped = if position == 0 {
            offset
        } else {
            let modulus = stride
                .checked_mul(extent)
                .ok_or_else(|| occurrence_error("contraction-extent-overflow"))?;
            context.modulo(offset, SourcedExtent::Static(Extent::new(modulus)))?
        };
        if stride == 1 {
            Ok(wrapped)
        } else {
            Ok(context.floor_div(wrapped, SourcedExtent::Static(Extent::new(stride)))?)
        }
    }
}

/// Emits the copy region realizing one `tiler.reindex-f32` occurrence.
///
/// Every admitted form becomes one read whose coordinates are index expressions
/// over the result's iteration dimensions, and one write at the identity
/// coordinates. No scalar operation is applied, because a reindex computes
/// nothing: the value written is the value read.
///
/// The emitted region is *not* a claim that a copy must happen. It is the access
/// relation the occurrence denotes; whether the surrounding plan materializes it,
/// composes it into a neighbouring kernel's addressing, or elides it entirely is
/// a scheduling outcome this authority does not decide.
struct GovernedReindexF32;

impl IndexAccessLoweringProvider for GovernedReindexF32 {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        let occurrence = context.occurrence();
        let ([input], [result]) = (occurrence.inputs(), occurrence.results()) else {
            return Err(occurrence_error("reindex-arity"));
        };
        if occurrence.operands() != [0] {
            return Err(occurrence_error("reindex-operand-binding"));
        }
        let Some(value) = occurrence.attributes().get(REINDEX_MAPPING_ATTRIBUTE) else {
            return Err(occurrence_error("reindex-form-missing"));
        };
        let form = ReindexForm::from_canonical_value(value)
            .map_err(|_| occurrence_error("reindex-form"))?;
        let input_shape = input.shape().clone();
        let result_shape = result.shape().clone();
        // The occurrence is re-derived rather than trusted: the form must produce
        // exactly this result from exactly this operand, or the region about to
        // be emitted would realize a different occurrence than the one requested.
        if form
            .result_shape(&input_shape)
            .map_err(|_| occurrence_error("reindex-form"))?
            != result_shape
        {
            return Err(occurrence_error("reindex-result-shape"));
        }
        let value_type = result.value_type().clone();
        let input_type = input.value_type().clone();

        let dimensions = declare_parallel_domain(context, &result_shape)?;
        let coordinates = dimension_expressions(context, &dimensions)?;
        let operand_coordinates =
            reindex_operand_coordinates(context, &form, &input_shape, &coordinates)?;
        // Every admitted form's coordinates range over every result dimension,
        // with one exception: an inserted unit axis has no operand axis behind it
        // and is omitted, which is what makes the read invariant in it.
        let domain: Vec<_> = match form.kind() {
            ReindexFormKind::InsertUnitAxis => {
                let inserted = usize::try_from(
                    form.axes()
                        .first()
                        .ok_or_else(|| occurrence_error("reindex-axis"))?
                        .get(),
                )
                .map_err(|_| occurrence_error("reindex-axis"))?;
                dimensions
                    .iter()
                    .enumerate()
                    .filter(|(position, _)| *position != inserted)
                    .map(|(_, dimension)| *dimension)
                    .collect()
            }
            _ => dimensions.clone(),
        };

        let tensor = context.input_tensor(input_type, input_shape)?;
        let value = context.read(tensor, &domain, &operand_coordinates)?;
        let output = context.output_tensor(value_type, result_shape)?;
        let write = context.write(output, &dimensions, &coordinates)?;
        context.output(write, value)
    }
}

/// Builds the operand coordinate per input axis for one admitted reindex form.
///
/// `coordinates` are the result's iteration coordinates in result-axis order.
fn reindex_operand_coordinates(
    context: &mut IndexAccessLoweringContext<'_>,
    form: &ReindexForm,
    input_shape: &Shape,
    coordinates: &[IndexExprId],
) -> Result<Vec<IndexExprId>, LoweringEmitError> {
    let extents = input_shape.extents();
    let at = |position: usize| -> Result<IndexExprId, LoweringEmitError> {
        coordinates
            .get(position)
            .copied()
            .ok_or_else(|| occurrence_error("reindex-coordinate"))
    };
    let axis_of = |axis: Axis| -> Result<usize, LoweringEmitError> {
        usize::try_from(axis.get()).map_err(|_| occurrence_error("reindex-axis"))
    };
    match form.kind() {
        // Result axis `k` reads operand axis `order[k]`, so the operand's axis
        // `order[k]` takes the result's `k`-th coordinate. Written as a scatter
        // into the operand's axis order rather than a gather, because that is the
        // direction the attribute states.
        ReindexFormKind::PermuteAxes => {
            let mut operand = vec![None; extents.len()];
            for (position, axis) in form.axes().iter().enumerate() {
                let index = axis_of(*axis)?;
                let slot = operand
                    .get_mut(index)
                    .ok_or_else(|| occurrence_error("reindex-axis"))?;
                if slot.replace(at(position)?).is_some() {
                    return Err(occurrence_error("reindex-permutation"));
                }
            }
            operand
                .into_iter()
                .map(|slot| slot.ok_or_else(|| occurrence_error("reindex-permutation")))
                .collect()
        }
        // The split's result axes linearize back into one operand coordinate,
        // major factor first: `sum(d_j * prod(factors[j+1..]))`. This is one
        // affine combination rather than a chain of multiplies, which is what
        // keeps the access affine.
        ReindexFormKind::SplitAxis => {
            let axis = axis_of(
                *form
                    .axes()
                    .first()
                    .ok_or_else(|| occurrence_error("reindex-axis"))?,
            )?;
            let factors = form.factors();
            let mut strides = vec![1_u64; factors.len()];
            let mut stride = 1_u64;
            for (position, factor) in factors.iter().enumerate().rev() {
                strides[position] = stride;
                stride = stride
                    .checked_mul(factor.get())
                    .ok_or_else(|| occurrence_error("reindex-split-overflow"))?;
            }
            let mut terms = Vec::with_capacity(factors.len());
            for (position, stride) in strides.iter().enumerate() {
                terms.push((
                    IndexInteger::from_u64(*stride),
                    at(axis.saturating_add(position))?,
                ));
            }
            let linearized = context.linear_combination(IndexInteger::from_u64(0), &terms)?;
            let mut operand = Vec::with_capacity(extents.len());
            for position in 0..extents.len() {
                operand.push(match position.cmp(&axis) {
                    std::cmp::Ordering::Less => at(position)?,
                    std::cmp::Ordering::Equal => linearized,
                    std::cmp::Ordering::Greater => {
                        at(position.saturating_add(factors.len()).saturating_sub(1))?
                    }
                });
            }
            Ok(operand)
        }
        // The merge decodes one result coordinate back into the merged run, using
        // the same wrap-then-divide shape the serial sum's reduced-offset decode
        // uses so that one decoding convention exists in this module.
        ReindexFormKind::MergeAxes => {
            let axes = form.axes();
            let first = axis_of(
                *axes
                    .first()
                    .ok_or_else(|| occurrence_error("reindex-axis"))?,
            )?;
            let count = axes.len();
            let merged: Vec<u64> = axes
                .iter()
                .map(|axis| {
                    axis_of(*axis).and_then(|index| {
                        extents
                            .get(index)
                            .map(|extent| extent.get())
                            .ok_or_else(|| occurrence_error("reindex-axis"))
                    })
                })
                .collect::<Result<_, _>>()?;
            let mut strides = vec![1_u64; count];
            let mut stride = 1_u64;
            for (position, extent) in merged.iter().enumerate().rev() {
                strides[position] = stride;
                stride = stride
                    .checked_mul(*extent)
                    .ok_or_else(|| occurrence_error("reindex-merge-overflow"))?;
            }
            let linear = at(first)?;
            let mut decoded = Vec::with_capacity(count);
            for position in 0..count {
                // The leading merged axis needs no wrap: the result coordinate is
                // already below the product of every merged extent.
                let wrapped = if position == 0 {
                    linear
                } else {
                    let modulus = strides[position]
                        .checked_mul(merged[position])
                        .ok_or_else(|| occurrence_error("reindex-merge-overflow"))?;
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
            let mut operand = Vec::with_capacity(extents.len());
            for position in 0..extents.len() {
                operand.push(if position < first {
                    at(position)?
                } else if position < first.saturating_add(count) {
                    decoded[position - first]
                } else {
                    at(position.saturating_sub(count).saturating_add(1))?
                });
            }
            Ok(operand)
        }
        // The inserted result axis has extent one and no operand axis behind it,
        // so it contributes no coordinate and the read is invariant in it.
        ReindexFormKind::InsertUnitAxis => {
            let position = axis_of(
                *form
                    .axes()
                    .first()
                    .ok_or_else(|| occurrence_error("reindex-axis"))?,
            )?;
            (0..extents.len())
                .map(|axis| {
                    if axis < position {
                        at(axis)
                    } else {
                        at(axis.saturating_add(1))
                    }
                })
                .collect()
        }
        // The removed operand axis has extent one, so its only coordinate is zero.
        ReindexFormKind::RemoveUnitAxis => {
            let removed = axis_of(
                *form
                    .axes()
                    .first()
                    .ok_or_else(|| occurrence_error("reindex-axis"))?,
            )?;
            let zero = context.constant(IndexInteger::from_u64(0))?;
            (0..extents.len())
                .map(|axis| match axis.cmp(&removed) {
                    std::cmp::Ordering::Less => at(axis),
                    std::cmp::Ordering::Equal => Ok(zero),
                    std::cmp::Ordering::Greater => at(axis.saturating_sub(1)),
                })
                .collect()
        }
        // `i -> extent - 1 - i`, the one within-axis coordinate permutation the
        // family admits. It is affine — a constant plus a coefficient of minus
        // one — which is exactly why it is the form D-10 admits.
        ReindexFormKind::ReverseAxis => {
            let reversed = axis_of(
                *form
                    .axes()
                    .first()
                    .ok_or_else(|| occurrence_error("reindex-axis"))?,
            )?;
            let extent = extents
                .get(reversed)
                .ok_or_else(|| occurrence_error("reindex-axis"))?
                .get();
            let last = i128::from(extent)
                .checked_sub(1)
                .ok_or_else(|| occurrence_error("reindex-reverse-extent"))?;
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

/// Emits the region realizing one `tiler.broadcast-f32` occurrence.
///
/// The read omits every replicated result dimension and maps every stretched one
/// to zero, which is precisely how the IR contract describes a broadcast's access
/// map. The write covers the whole result domain, so ownership is a coordinate
/// permutation even though the read aliases.
struct GovernedBroadcastF32;

impl IndexAccessLoweringProvider for GovernedBroadcastF32 {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        let occurrence = context.occurrence();
        let ([input], [result]) = (occurrence.inputs(), occurrence.results()) else {
            return Err(occurrence_error("broadcast-arity"));
        };
        if occurrence.operands() != [0] {
            return Err(occurrence_error("broadcast-operand-binding"));
        }
        let Some(value) = occurrence
            .attributes()
            .get(BROADCAST_AXIS_MAPPING_ATTRIBUTE)
        else {
            return Err(occurrence_error("broadcast-mapping-missing"));
        };
        let mapping = BroadcastAxisMapping::from_canonical_value(value)
            .map_err(|_| occurrence_error("broadcast-mapping"))?;
        let input_shape = input.shape().clone();
        let result_shape = result.shape().clone();
        if mapping
            .result_shape(&input_shape)
            .map_err(|_| occurrence_error("broadcast-mapping"))?
            != result_shape
        {
            return Err(occurrence_error("broadcast-result-shape"));
        }
        let value_type = result.value_type().clone();
        let input_type = input.value_type().clone();

        let dimensions = declare_parallel_domain(context, &result_shape)?;
        let coordinates = dimension_expressions(context, &dimensions)?;
        let zero = context.constant(IndexInteger::from_u64(0))?;
        // The read ranges only over the result dimensions a one-to-one
        // correspondence carries. A replicated dimension is omitted and a
        // stretched one maps to zero, which is exactly the IR contract's "a
        // broadcast omits an iteration coordinate or maps it to zero".
        let domain: Vec<_> = mapping
            .sources()
            .iter()
            .zip(&dimensions)
            .filter(|(source, _)| matches!(source, BroadcastAxisSource::FromOperand(_)))
            .map(|(_, dimension)| *dimension)
            .collect();
        let mut operand_coordinates = vec![None; input_shape.rank()];
        for (result_axis, source) in mapping.sources().iter().enumerate() {
            let Some(axis) = source.operand_axis() else {
                continue;
            };
            let index = usize::try_from(axis.get())
                .ok()
                .filter(|index| *index < input_shape.rank())
                .ok_or_else(|| occurrence_error("broadcast-axis"))?;
            let coordinate = match source {
                BroadcastAxisSource::FromOperand(_) => *coordinates
                    .get(result_axis)
                    .ok_or_else(|| occurrence_error("broadcast-coordinate"))?,
                // An extent-one operand axis has exactly one coordinate, so the
                // stretched result dimension maps to zero rather than being
                // omitted: the axis exists on the operand and must be indexed.
                BroadcastAxisSource::StretchUnit(_) => zero,
                BroadcastAxisSource::Replicate => unreachable!("a replication names no axis"),
            };
            if operand_coordinates[index].replace(coordinate).is_some() {
                return Err(occurrence_error("broadcast-axis-repeated"));
            }
        }
        let operand_coordinates: Vec<IndexExprId> = operand_coordinates
            .into_iter()
            .map(|slot| slot.ok_or_else(|| occurrence_error("broadcast-axis-unmapped")))
            .collect::<Result<_, _>>()?;

        let tensor = context.input_tensor(input_type, input_shape)?;
        let value = context.read(tensor, &domain, &operand_coordinates)?;
        let output = context.output_tensor(value_type, result_shape)?;
        let write = context.write(output, &dimensions, &coordinates)?;
        context.output(write, value)
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
mod contraction_conformance;

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
        BROADCAST_AXIS_MAPPING_ATTRIBUTE, BroadcastAxisMapping, BroadcastAxisSource,
        CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, CanonicalField, CanonicalValue, ContractionIndex,
        ContractionIndexStructure, F32, F32_CONSTANT_BITS_ATTRIBUTE, OpKey, OperationAttributes,
        OperationEffect, REDUCTION_AXES_ATTRIBUTE, REINDEX_MAPPING_ATTRIBUTE, ReindexForm,
        ResolvedValueType, TypeKey, add_f32_op, broadcast_f32_op, constant_f32_op, multiply_f32_op,
        reindex_f32_op, strict_serial_sum_f32_op, strict_tensor_contraction_f32_op,
    };
    use tiler_ir::shape::{Axis, Extent, Shape};
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

    // ---------------------------------------------------------------------
    // The two structural families' access maps
    // ---------------------------------------------------------------------
    //
    // Every case below refines the emitted region *and* executes it, because a
    // coordinate map that refines proves only that the region is well formed
    // against its occurrence — the interface, the ownership, the reached
    // authority. Which element each result coordinate reads is exactly what
    // refinement does not check, and it is the whole content of these families.
    //
    // The fixtures are ascending integers rather than exceptional payloads, so a
    // wrong coordinate map produces a wrong *value* and not a coincidence. The
    // bit-preservation property is checked separately, once, at the end.

    fn reindex_attributes(form: &ReindexForm) -> OperationAttributes {
        OperationAttributes::new([CanonicalField::new(
            REINDEX_MAPPING_ATTRIBUTE,
            form.canonical_value().clone(),
        )])
        .unwrap()
    }

    fn broadcast_attributes(mapping: &BroadcastAxisMapping) -> OperationAttributes {
        OperationAttributes::new([CanonicalField::new(
            BROADCAST_AXIS_MAPPING_ATTRIBUTE,
            mapping.canonical_value().clone(),
        )])
        .unwrap()
    }

    /// Refines and executes one structural occurrence over ascending integers.
    fn structural_result(
        operation: OpKey,
        attributes: OperationAttributes,
        input: Shape,
        result: Shape,
    ) -> Vec<u32> {
        let count = input.element_count().expect("a test shape is bounded");
        let bits: Vec<u32> = (0..count)
            .map(|value| u32::try_from(value).expect("a test operand is small"))
            .collect();
        let refinement = refine(
            operation,
            vec![OccurrenceOperand::new(
                OccurrenceValueId(0),
                f32_type(),
                input.clone(),
            )],
            vec![OccurrenceResult::new(f32_type(), result)],
            attributes,
        );
        let tensor = bit_tensor(input, &bits);
        evaluate_refined(&refinement, &[(0, &tensor)])
    }

    /// Every admitted reindex form, emitted and executed against a hand-derived
    /// expected permutation of the operand's elements.
    #[test]
    fn the_governed_reindex_region_realizes_every_admitted_form() {
        // A transpose of `[2, 3]`: row-major 0..6 becomes column-major.
        assert_eq!(
            structural_result(
                reindex_f32_op(),
                reindex_attributes(
                    &ReindexForm::permute_axes([Axis::new(1), Axis::new(0)]).unwrap()
                ),
                Shape::from_dims([2, 3]),
                Shape::from_dims([3, 2]),
            ),
            vec![0, 3, 1, 4, 2, 5],
        );

        // A split of a six-wide axis into (3, 2), major factor first. Row-major
        // order is unchanged, which is what makes a split a reshape.
        assert_eq!(
            structural_result(
                reindex_f32_op(),
                reindex_attributes(
                    &ReindexForm::split_axis(Axis::new(0), [Extent::new(3), Extent::new(2)])
                        .unwrap()
                ),
                Shape::from_dims([6]),
                Shape::from_dims([3, 2]),
            ),
            vec![0, 1, 2, 3, 4, 5],
        );

        // The merge back, and one over an inner run of a rank-three operand,
        // which is the case a wrong stride order would get wrong while the
        // rank-two case still passed.
        assert_eq!(
            structural_result(
                reindex_f32_op(),
                reindex_attributes(&ReindexForm::merge_axes([Axis::new(0), Axis::new(1)]).unwrap()),
                Shape::from_dims([3, 2]),
                Shape::from_dims([6]),
            ),
            vec![0, 1, 2, 3, 4, 5],
        );
        assert_eq!(
            structural_result(
                reindex_f32_op(),
                reindex_attributes(&ReindexForm::merge_axes([Axis::new(1), Axis::new(2)]).unwrap()),
                Shape::from_dims([2, 2, 3]),
                Shape::from_dims([2, 6]),
            ),
            (0..12).collect::<Vec<_>>(),
        );

        // Unit-axis insertion and removal move no element.
        assert_eq!(
            structural_result(
                reindex_f32_op(),
                reindex_attributes(&ReindexForm::insert_unit_axis(Axis::new(1)).unwrap()),
                Shape::from_dims([3]),
                Shape::from_dims([3, 1]),
            ),
            vec![0, 1, 2],
        );
        assert_eq!(
            structural_result(
                reindex_f32_op(),
                reindex_attributes(&ReindexForm::remove_unit_axis(Axis::new(1)).unwrap()),
                Shape::from_dims([3, 1]),
                Shape::from_dims([3]),
            ),
            vec![0, 1, 2],
        );

        // The D-10 form. On a `[2, 2, 3]` operand the size-two axis 1 reverses,
        // which swaps the two three-element rows within each outer block — the
        // exact map `rotate_half` performs at head dimension two.
        assert_eq!(
            structural_result(
                reindex_f32_op(),
                reindex_attributes(&ReindexForm::reverse_axis(Axis::new(1)).unwrap()),
                Shape::from_dims([2, 2, 3]),
                Shape::from_dims([2, 2, 3]),
            ),
            vec![3, 4, 5, 0, 1, 2, 9, 10, 11, 6, 7, 8],
        );
        // A wider axis, so the reversal is tested where it is more than a swap.
        assert_eq!(
            structural_result(
                reindex_f32_op(),
                reindex_attributes(&ReindexForm::reverse_axis(Axis::new(0)).unwrap()),
                Shape::from_dims([4]),
                Shape::from_dims([4]),
            ),
            vec![3, 2, 1, 0],
        );
    }

    /// Both many-to-one relations, emitted and executed.
    #[test]
    fn the_governed_broadcast_region_realizes_both_many_to_one_relations() {
        let replicate = BroadcastAxisSource::Replicate;
        // A rank pad: `[3]` against `[2, 3]`, the normalization weight's shape.
        assert_eq!(
            structural_result(
                broadcast_f32_op(),
                broadcast_attributes(
                    &BroadcastAxisMapping::new(
                        [Extent::new(2), Extent::new(3)],
                        [replicate, BroadcastAxisSource::FromOperand(Axis::new(0))],
                    )
                    .unwrap()
                ),
                Shape::from_dims([3]),
                Shape::from_dims([2, 3]),
            ),
            vec![0, 1, 2, 0, 1, 2],
        );

        // A unit stretch: `[2, 1]` against `[2, 3]`, the rotary sign operand's
        // shape. The distinguishing case — a rank pad of `[2]` to `[2, 3]` is not
        // even expressible, because the operand axis would have nowhere to go.
        assert_eq!(
            structural_result(
                broadcast_f32_op(),
                broadcast_attributes(
                    &BroadcastAxisMapping::new(
                        [Extent::new(2), Extent::new(3)],
                        [
                            BroadcastAxisSource::FromOperand(Axis::new(0)),
                            BroadcastAxisSource::StretchUnit(Axis::new(1)),
                        ],
                    )
                    .unwrap()
                ),
                Shape::from_dims([2, 1]),
                Shape::from_dims([2, 3]),
            ),
            vec![0, 0, 0, 1, 1, 1],
        );

        // An interior rank pad, which is the rotary table's `[T, D]` against
        // `[T, heads, D]` and the one shape a leading-pad-only implementation
        // would get wrong.
        assert_eq!(
            structural_result(
                broadcast_f32_op(),
                broadcast_attributes(
                    &BroadcastAxisMapping::new(
                        [Extent::new(2), Extent::new(2), Extent::new(3)],
                        [
                            BroadcastAxisSource::FromOperand(Axis::new(0)),
                            replicate,
                            BroadcastAxisSource::FromOperand(Axis::new(1)),
                        ],
                    )
                    .unwrap()
                ),
                Shape::from_dims([2, 3]),
                Shape::from_dims([2, 2, 3]),
            ),
            vec![0, 1, 2, 0, 1, 2, 3, 4, 5, 3, 4, 5],
        );
    }

    /// Neither family may rewrite a payload it only transports.
    ///
    /// The arithmetic families canonicalize every NaN they produce, and that rule
    /// must not leak into a family that produces nothing. A structural region
    /// that applied it would return a canonical NaN where the program supplied a
    /// signalling one, which is a value change wearing a numerical contract's
    /// clothes.
    #[test]
    fn the_structural_regions_transport_exceptional_payloads_unchanged() {
        let payloads = [
            NONCANONICAL_NAN,
            0x7f80_0001,
            (-0.0_f32).to_bits(),
            LEAST_SUBNORMAL,
        ];
        let shape = Shape::from_dims([4]);
        let tensor = bit_tensor(shape.clone(), &payloads);

        let reversed = refine(
            reindex_f32_op(),
            vec![OccurrenceOperand::new(
                OccurrenceValueId(0),
                f32_type(),
                shape.clone(),
            )],
            vec![OccurrenceResult::new(f32_type(), shape.clone())],
            reindex_attributes(&ReindexForm::reverse_axis(Axis::new(0)).unwrap()),
        );
        assert_eq!(
            evaluate_refined(&reversed, &[(0, &tensor)]),
            vec![
                LEAST_SUBNORMAL,
                (-0.0_f32).to_bits(),
                0x7f80_0001,
                NONCANONICAL_NAN,
            ],
            "a reindex reorders payloads and rewrites none of them",
        );

        let widened = refine(
            broadcast_f32_op(),
            vec![OccurrenceOperand::new(
                OccurrenceValueId(0),
                f32_type(),
                shape,
            )],
            vec![OccurrenceResult::new(f32_type(), Shape::from_dims([2, 4]))],
            broadcast_attributes(
                &BroadcastAxisMapping::new(
                    [Extent::new(2), Extent::new(4)],
                    [
                        BroadcastAxisSource::Replicate,
                        BroadcastAxisSource::FromOperand(Axis::new(0)),
                    ],
                )
                .unwrap(),
            ),
        );
        assert_eq!(
            evaluate_refined(&widened, &[(0, &tensor)])[4..],
            payloads,
            "and a broadcast replicates them unchanged",
        );
    }

    /// A lowering emits a region for the occurrence it was handed, or refuses.
    ///
    /// The occurrence's declared result shape is the host's, and the form's is
    /// derived. A lowering that emitted its own derivation regardless would
    /// produce a region that realizes a different occurrence than the one
    /// requested, and refinement's interface check is not what catches it —
    /// the region would be internally consistent and simply wrong.
    #[test]
    fn a_structural_lowering_refuses_an_occurrence_its_mapping_does_not_describe() {
        let scalars = governed_scalars().unwrap();
        let registry = governed_lowering_capabilities(&scalars).unwrap();
        let signature = LoweringSignature::new([f32_type()], [f32_type()]).unwrap();

        let mismatched = SemanticOccurrence::new(
            reindex_f32_op(),
            vec![OccurrenceOperand::new(
                OccurrenceValueId(0),
                f32_type(),
                Shape::from_dims([6]),
            )],
            // The form derives `[3, 2]`; the occurrence declares `[2, 3]`.
            vec![OccurrenceResult::new(f32_type(), Shape::from_dims([2, 3]))],
            reindex_attributes(
                &ReindexForm::split_axis(Axis::new(0), [Extent::new(3), Extent::new(2)]).unwrap(),
            ),
            OperationEffect::Pure,
            contract(),
            SemanticOccurrenceIdentity::from_bytes(b"mismatched-fixture".to_vec()),
        );
        let resolved = registry
            .resolve_index_access(mismatched.operation(), &signature)
            .unwrap();
        assert!(
            refine_index_region(&resolved, &mismatched, &scalars).is_err(),
            "a reindex whose declared result the form does not derive is refused",
        );
    }

    /// Builds the attribute record carrying one contraction index structure.
    fn contraction_attributes(structure: &ContractionIndexStructure) -> OperationAttributes {
        OperationAttributes::new([CanonicalField::new(
            CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE,
            structure.canonical_value().clone(),
        )])
        .unwrap()
    }

    /// Refines one contraction occurrence through the governed registry.
    fn refine_contraction(
        structure: &ContractionIndexStructure,
        left: Shape,
        right: Shape,
        result: Shape,
    ) -> IndexRefinement {
        refine(
            strict_tensor_contraction_f32_op(),
            vec![
                OccurrenceOperand::new(OccurrenceValueId(0), f32_type(), left),
                OccurrenceOperand::new(OccurrenceValueId(1), f32_type(), right),
            ],
            vec![OccurrenceResult::new(f32_type(), result)],
            contraction_attributes(structure),
        )
    }

    /// The profile's own index structure, `td,od->to`, spelled with arbitrary
    /// frontend labels so the canonicalization is exercised rather than assumed.
    fn projection_structure() -> ContractionIndexStructure {
        ContractionIndexStructure::new(
            [
                [ContractionIndex::new(19), ContractionIndex::new(3)],
                [ContractionIndex::new(14), ContractionIndex::new(3)],
            ],
            [ContractionIndex::new(19), ContractionIndex::new(14)],
        )
        .unwrap()
    }

    #[test]
    fn the_governed_contraction_lowering_refines_its_occurrence() {
        let refinement = refine_contraction(
            &projection_structure(),
            Shape::from_dims([2, 3]),
            Shape::from_dims([4, 3]),
            Shape::from_dims([2, 4]),
        );
        assert_eq!(refinement.operand_bindings().len(), 2);
        assert_eq!(refinement.result_bindings().len(), 1);
        assert_ne!(
            refinement.operand_bindings()[0].input_tensor(),
            refinement.operand_bindings()[1].input_tensor()
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

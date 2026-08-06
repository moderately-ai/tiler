//! The lowering capabilities the bounded compiler profile ships with.
//!
//! Each semantic family in the governed logical-realization profile gets one
//! [`IndexAccessLoweringProvider`] registered against
//! [`FrozenScalarRegistry::standard`]. The providers are shape- and
//! attribute-driven: every extent, every broadcast, and every constant bit
//! pattern is read from the [`tiler_ir::index::IndexRefinementSubject`] facts
//! the host hands them.
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
    ScalarRegistryError, ScalarValueId, SourcedExtent, StagedInputSource, add_f32_scalar_op,
    canonicalize_nan_f32_scalar_op, constant_f32_scalar_op, divide_f32_scalar_op,
    exp_f32_scalar_op, multiply_f32_scalar_op, rsqrt_f32_scalar_op,
    strict_affine_u4_dequantize_scalar_op,
};
use tiler_ir::semantic::{
    AttributeFieldId, BROADCAST_AXIS_MAPPING_ATTRIBUTE, BroadcastAxisMapping, BroadcastAxisSource,
    CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, CanonicalField, CanonicalIntegerWidth, CanonicalValue,
    CanonicalValueView, ContractionIndex, ContractionIndexStructure, F32,
    F32_CONSTANT_BITS_ATTRIBUTE, OpKey, OperationAttributes, ProviderIdentity,
    REDUCTION_AXES_ATTRIBUTE, REINDEX_MAPPING_ATTRIBUTE, RMS_NORM_EPS_BITS_ATTRIBUTE,
    RMS_NORM_REDUCED_AXES_ATTRIBUTE, ReindexForm, ReindexFormKind, ResolvedValueType,
    STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE,
    StrictAffineU4, TypeKey, add_f32_op, broadcast_f32_op, constant_f32_op,
    dequantize_strict_affine_op, multiply_f32_op, reindex_f32_op, rms_norm_f32_op, silu_f32_op,
    strict_serial_sum_f32_op, strict_tensor_contraction_f32_op,
};
use tiler_ir::shape::{Axis, Extent, Shape};

use crate::capability::{
    FrozenLoweringCapabilityRegistry, IndexAccessLoweringContext, IndexAccessLoweringProvider,
    IndexAccessSequenceContext, LoweringCapabilityRegistryBuilder, LoweringCapabilityRevision,
    LoweringEmitError, LoweringRegistryError, LoweringSignature,
};
use crate::elementary::{ElementaryPointSink, silu_point_body};

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
    )?;
    for capability in governed_index_access_capabilities()? {
        capability.register(&mut builder)?;
    }
    Ok(builder.freeze())
}

/// Builds the independent semantic-realization authority for governed families.
#[cfg(test)]
pub(crate) fn governed_realization_laws(
    scalars: &FrozenScalarRegistry,
) -> tiler_ir::index::FrozenIndexRealizationLawRegistry {
    tiler_ir::index::FrozenIndexRealizationLawRegistry::from_semantic(
        scalars.semantic_authority().clone(),
        scalars.clone(),
    )
    .expect("the governed scalar registry retains its exact semantic authority")
}

/// Registers the shipped index-access capabilities onto a caller's builder,
/// except the operation families the caller is substituting for.
///
/// This is the affordance [`GovernedIndexAccess`]'s own documentation describes
/// — composing a registry from a chosen subset so an external provider replaces
/// one governed family without re-implementing the others — made reachable
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
/// Keeping the descriptors addressable lets a caller compose a registry
/// from a chosen subset of them, which is how an external provider substitutes
/// for one governed family without re-implementing the others.
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

/// Returns the ten shipped index-access capabilities in canonical family order.
///
/// # Errors
///
/// Returns [`GovernedRegistryError`] when a governed signature exceeds its
/// governed structural bound.
pub(crate) fn governed_index_access_capabilities()
-> Result<[GovernedIndexAccess; 10], GovernedRegistryError> {
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
        GovernedIndexAccess {
            provider: governed_provider("rms-norm-f32"),
            operation: rms_norm_f32_op(),
            signature: LoweringSignature::new(
                [F32::resolved_type(), F32::resolved_type()],
                [F32::resolved_type()],
            )?,
            // The union over shapes of what the two stages reach. The constant
            // is the folded extent and the declared bias; the multiply is the
            // per-contributor square and both of the second stage's products;
            // the add is the fold's combine and the bias; the canonicalization
            // is the singleton fold's result boundary; the division and the
            // reciprocal square root are the epilogue's. There is no reduction
            // identity constant in the list beyond those two, because an empty
            // fold is refused rather than seeded — `folded_extent_bits` says
            // why.
            emitted: vec![
                constant_f32_scalar_op(),
                multiply_f32_scalar_op(),
                add_f32_scalar_op(),
                canonicalize_nan_f32_scalar_op(),
                divide_f32_scalar_op(),
                rsqrt_f32_scalar_op(),
            ],
            implementation: Arc::new(GovernedRootMeanSquareScaleF32),
        },
        GovernedIndexAccess {
            provider: governed_provider("strict-affine-u4-dequantize"),
            operation: dequantize_strict_affine_op(),
            signature: LoweringSignature::new(
                [StrictAffineU4::resolved_type()],
                [F32::resolved_type()],
            )?,
            emitted: vec![strict_affine_u4_dequantize_scalar_op()],
            implementation: Arc::new(GovernedStrictAffineU4Dequantize),
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

/// Emits the canonical logical component access for strict-affine U4 decode.
///
/// The three input tensors are not three semantic operands. They are the
/// ordered component projection of one encoded logical operand, and refinement
/// binds each tensor back to that operand and its stable component role.
struct GovernedStrictAffineU4Dequantize;

impl IndexAccessLoweringProvider for GovernedStrictAffineU4Dequantize {
    fn lower(&self, context: &mut IndexAccessLoweringContext<'_>) -> Result<(), LoweringEmitError> {
        let occurrence = context.occurrence();
        let ([input], [result]) = (occurrence.inputs(), occurrence.results()) else {
            return Err(occurrence_error("strict-affine-arity"));
        };
        if occurrence.operands() != [0] || !occurrence.attributes().fields().is_empty() {
            return Err(occurrence_error("strict-affine-subject"));
        }
        if input.value_type() != &StrictAffineU4::resolved_type()
            || result.value_type() != &F32::resolved_type()
            || input.shape() != result.shape()
        {
            return Err(occurrence_error("strict-affine-interface"));
        }
        let (_, contract) = input
            .value_type()
            .encoded_numeric_parts()
            .ok_or_else(|| occurrence_error("strict-affine-encoded-contract"))?;
        let components = contract.components();
        let roles = [
            STRICT_AFFINE_CODES_ROLE,
            STRICT_AFFINE_SCALE_ROLE,
            STRICT_AFFINE_ZERO_POINT_ROLE,
        ];
        if components.len() != roles.len()
            || components
                .iter()
                .zip(roles)
                .any(|(component, role)| component.role() != role)
        {
            return Err(occurrence_error("strict-affine-component-roles"));
        }

        let shape = result.shape().clone();
        let result_type = result.value_type().clone();
        let component_boundaries = components
            .iter()
            .map(|component| {
                (
                    component.resolved_type().clone(),
                    component.shape_relation().component_shape(input.shape()),
                )
            })
            .collect::<Vec<_>>();
        let dimensions = declare_parallel_domain(context, &shape)?;
        let coordinates = dimension_expressions(context, &dimensions)?;
        let mut tensors = Vec::with_capacity(component_boundaries.len());
        for (value_type, component_shape) in component_boundaries {
            tensors.push(context.input_tensor(value_type, component_shape)?);
        }
        let codes = context.read(tensors[0], &dimensions, &coordinates)?;
        let scale = context.read(tensors[1], &[], &[])?;
        let zero_point = context.read(tensors[2], &[], &[])?;
        let value = apply_one(
            context,
            strict_affine_u4_dequantize_scalar_op(),
            ScalarAttributes::empty(),
            &[codes, scale, zero_point],
            "strict-affine-u4-dequantize",
        )?;
        let output = context.output_tensor(result_type, shape)?;
        let write = context.write(output, &dimensions, &coordinates)?;
        context.output(write, value)
    }
}

/// Emits the region realizing one `tiler::silu-f32@1` occurrence.
///
/// The per-point chain is not written here. [`crate::elementary::silu_point_body`]
/// is the one statement of the composition in this crate, and this provider
/// supplies the index-region half of its realization — the parallel domain, the
/// read, the write, and the scalar vocabulary the body is emitted into. The
/// request boundary drives the *same* function into the physical expression
/// vocabulary, so the two realizations cannot state different compositions; that
/// module's own documentation states which properties of the chain are
/// load-bearing.
struct GovernedSiluF32;

/// Emits an elementary body as governed index-region scalar applications.
struct GovernedElementarySink<'a, 'b> {
    context: &'a mut IndexAccessLoweringContext<'b>,
}

impl ElementaryPointSink for GovernedElementarySink<'_, '_> {
    type Value = ScalarValueId;
    type Error = LoweringEmitError;

    fn constant(&mut self, bits: u32, rule: &'static str) -> Result<Self::Value, Self::Error> {
        let attributes = f32_constant_attributes(bits)?;
        apply_one(
            self.context,
            constant_f32_scalar_op(),
            attributes,
            &[],
            rule,
        )
    }

    fn add(
        &mut self,
        lhs: Self::Value,
        rhs: Self::Value,
        rule: &'static str,
    ) -> Result<Self::Value, Self::Error> {
        apply_one(
            self.context,
            add_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[lhs, rhs],
            rule,
        )
    }

    fn multiply(
        &mut self,
        lhs: Self::Value,
        rhs: Self::Value,
        rule: &'static str,
    ) -> Result<Self::Value, Self::Error> {
        apply_one(
            self.context,
            multiply_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[lhs, rhs],
            rule,
        )
    }

    fn divide(
        &mut self,
        lhs: Self::Value,
        rhs: Self::Value,
        rule: &'static str,
    ) -> Result<Self::Value, Self::Error> {
        apply_one(
            self.context,
            divide_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[lhs, rhs],
            rule,
        )
    }

    fn exp(
        &mut self,
        argument: Self::Value,
        rule: &'static str,
    ) -> Result<Self::Value, Self::Error> {
        apply_one(
            self.context,
            exp_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[argument],
            rule,
        )
    }
}

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

        let value = {
            let mut sink = GovernedElementarySink { context };
            silu_point_body(&mut sink, &argument)?
        };

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

        let total = plan.fold(context, input, &kept, &kept_coordinates)?;
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
    /// Scalar each contributor is squared with before the fold combines it.
    ///
    /// `None` folds the operand's own elements, which is every plain strict
    /// serial sum. `Some(scalar)` applies `scalar(v, v)` to each contributor
    /// first, which is the per-contributor prologue
    /// `IndexRealizationLaw::StagedRootMeanSquareScaleF32` carries and this
    /// provider must reproduce exactly.
    contributor_square: Option<ScalarOpKey>,
}

impl SumPlan {
    fn derive(
        occurrence: crate::capability::IndexAccessOccurrence<'_>,
    ) -> Result<Self, LoweringEmitError> {
        let ([input], [result]) = (occurrence.inputs(), occurrence.results()) else {
            return Err(occurrence_error("sum-arity"));
        };
        if occurrence.operands() != [0] {
            return Err(occurrence_error("sum-operand-binding"));
        }
        let axes = reduction_axes(occurrence.attributes(), REDUCTION_AXES_ATTRIBUTE)?;
        Self::for_boundaries(input.value_type(), input.shape(), result.shape(), &axes)
    }

    /// Derives the fold geometry from explicit boundaries rather than a result.
    ///
    /// A staged realization's fold publishes an intermediate, which is nobody's
    /// occurrence result, so the shape it must produce is a parameter. This
    /// mirrors `IndexRealizationLaw`'s own split for the same reason: the two
    /// derivations must agree axis for axis or the emitted region is a different
    /// region from the one the law builds.
    fn for_boundaries(
        value_type: &ResolvedValueType,
        input_shape: &Shape,
        output_shape: &Shape,
        axes: &[Axis],
    ) -> Result<Self, LoweringEmitError> {
        let input_shape = input_shape.clone();
        let mut reduced = vec![false; input_shape.rank()];
        for axis in axes {
            let index = usize::try_from(axis.get()).map_err(|_| occurrence_error("sum-axis"))?;
            let Some(slot) = reduced.get_mut(index) else {
                return Err(occurrence_error("sum-axis-range"));
            };
            if std::mem::replace(slot, true) {
                return Err(occurrence_error("sum-axis-duplicate"));
            }
        }
        if &input_shape.without_axes(axes) != output_shape {
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
            value_type: value_type.clone(),
            input_shape,
            output_shape: output_shape.clone(),
            reduced,
            reduced_strides,
            reduced_extents,
            reduced_points: stride,
            contributor_square: None,
        })
    }

    /// Folds `scalar(v, v)` per contributor rather than the contributor itself.
    fn squaring_contributors(mut self, scalar: ScalarOpKey) -> Self {
        self.contributor_square = Some(scalar);
        self
    }

    /// Emits the complete fold and returns its value, writing nothing.
    ///
    /// Separate from the serial sum's own `lower` because a staged realization
    /// transforms the fold *inside the producing region* — the normalization
    /// divides it, biases it, and takes its reciprocal square root before
    /// anything is written — so a fold that could only write its own result
    /// would force that epilogue into the consuming stage, where it would run
    /// once per point instead of once per folded row.
    fn fold(
        &self,
        context: &mut IndexAccessLoweringContext<'_>,
        input: tiler_ir::index::TensorId,
        kept: &[tiler_ir::index::DimensionId],
        kept_coordinates: &[IndexExprId],
    ) -> Result<ScalarValueId, LoweringEmitError> {
        if self.reduced_points == 0 {
            return self.fold_empty(context, input, kept, kept_coordinates);
        }
        let seed = self.read_contributor(context, input, kept, kept_coordinates, None)?;
        if self.reduced_points == 1 {
            // The lone contributor is the whole strict-serial value, and it is
            // the one boundary value no combine has canonicalized. Every other
            // path ends in the governed add, which canonicalizes its own result,
            // so this is the only place the boundary rule has work to do.
            apply_one(
                context,
                canonicalize_nan_f32_scalar_op(),
                ScalarAttributes::empty(),
                &[seed],
                "reduction-result-canonicalization",
            )
        } else {
            self.fold_tail(context, input, kept, kept_coordinates, seed)
        }
    }

    /// Applies the per-contributor square, when this plan carries one.
    fn square(
        &self,
        context: &mut IndexAccessLoweringContext<'_>,
        contributor: ScalarValueId,
    ) -> Result<ScalarValueId, LoweringEmitError> {
        match &self.contributor_square {
            Some(scalar) => apply_one(
                context,
                scalar.clone(),
                ScalarAttributes::empty(),
                &[contributor, contributor],
                "reduction-contributor-square",
            ),
            None => Ok(contributor),
        }
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
        let contributor = context.read(input, &domain, &coordinates)?;
        self.square(context, contributor)
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
        let contributor = self.square(context, contributor)?;
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

/// Emits the ordered two-region realization of one `tiler::rms-norm-f32@1`
/// occurrence.
///
/// **The first governed capability whose realization is a region *sequence*.**
/// The registered law
/// [`IndexRealizationLaw::StagedRootMeanSquareScaleF32`](tiler_ir::index::IndexRealizationLaw::StagedRootMeanSquareScaleF32)
/// pins the chain: stage zero folds the *square* of the value operand over the
/// declared axes, divides the fold by the folded contributor count, adds the
/// declared bias, applies the reciprocal square root, and publishes the result;
/// stage one reads the value operand, the weight operand, and that published
/// value, and writes `weight * (value * published)` pointwise.
///
/// **Refinement compares exact canonical region identity, so this emission is
/// the law's own order rather than an equivalent one.** Every step below is
/// stated in the order [`SumPlan`] and the law's emitters state it — kept
/// domain, input boundary, output boundary, fold, epilogue, write — because a
/// different emission order is a different region and would be refused as one.
///
/// **Where the split falls is the law's, not a schedule choice.** The published
/// value is read once per *point* and computed once per *folded row*, so stage
/// zero carries the whole epilogue. Publishing the raw fold instead would put
/// the division, the bias, and the reciprocal square root inside the pointwise
/// pass, evaluating each once per contributor: a different scalar program.
struct GovernedRootMeanSquareScaleF32;

impl IndexAccessLoweringProvider for GovernedRootMeanSquareScaleF32 {
    /// There is no single region realizing a staged occurrence, and saying so is
    /// the honest answer rather than dead code: it is exactly what
    /// `IndexRealizationLaw::realize` answers for the same law.
    fn lower(
        &self,
        _context: &mut IndexAccessLoweringContext<'_>,
    ) -> Result<(), LoweringEmitError> {
        Err(occurrence_error("rms-scale-requires-a-region-sequence"))
    }

    fn lower_sequence(
        &self,
        sequence: &mut IndexAccessSequenceContext<'_>,
    ) -> Result<(), LoweringEmitError> {
        let plan = RootMeanSquarePlan::derive(sequence.occurrence())?;
        sequence.stage(&[StagedInputSource::Occurrence(0)], |context| {
            plan.emit_fold(context)
        })?;
        sequence.stage(
            &[
                StagedInputSource::Occurrence(0),
                StagedInputSource::Occurrence(1),
                StagedInputSource::Intermediate(0),
            ],
            |context| plan.emit_scale(context),
        )
    }
}

/// The exact geometry one root-mean-square-scale occurrence describes.
struct RootMeanSquarePlan {
    /// The squaring fold stage zero publishes.
    sum: SumPlan,
    /// Exact binary32 payload of the folded contributor count.
    extent_bits: u32,
    /// The declared bias, forwarded as the constant's own attribute payload.
    eps: CanonicalValue,
    /// Shape of the published value, which is the occurrence shape less the
    /// reduced axes.
    intermediate_shape: Shape,
    /// The occurrence's common operand and result shape.
    shape: Shape,
    value_type: ResolvedValueType,
}

impl RootMeanSquarePlan {
    fn derive(
        occurrence: crate::capability::IndexAccessOccurrence<'_>,
    ) -> Result<Self, LoweringEmitError> {
        let ([value, weight], [result]) = (occurrence.inputs(), occurrence.results()) else {
            return Err(occurrence_error("rms-scale-arity"));
        };
        if occurrence.operands() != [0, 1] {
            return Err(occurrence_error("rms-scale-operand-binding"));
        }
        let expected = F32::resolved_type();
        if value.value_type() != &expected
            || weight.value_type() != &expected
            || result.value_type() != &expected
        {
            return Err(occurrence_error("rms-scale-value-type"));
        }
        if value.shape() != result.shape() || weight.shape() != result.shape() {
            return Err(occurrence_error("rms-scale-shape"));
        }
        // The law requires the occurrence's declared field set to be exactly the
        // two it names, so the axes reader's tolerance for a record carrying
        // more fields than it reads cannot silently drop the bias. A region
        // emitted without that requirement would realize a different operation
        // and be refused by the comparison; refusing here names the occurrence.
        let fields = occurrence.attributes().fields();
        if fields.len() != 2
            || !fields.iter().all(|field| {
                field.id() == RMS_NORM_REDUCED_AXES_ATTRIBUTE
                    || field.id() == RMS_NORM_EPS_BITS_ATTRIBUTE
            })
        {
            return Err(occurrence_error("rms-scale-attributes"));
        }
        let Some(eps) = occurrence.attributes().get(RMS_NORM_EPS_BITS_ATTRIBUTE) else {
            return Err(occurrence_error("rms-scale-eps-missing"));
        };
        if !matches!(eps.view(), CanonicalValueView::FloatBits(_)) {
            return Err(occurrence_error("rms-scale-eps-kind"));
        }
        let axes = reduction_axes(occurrence.attributes(), RMS_NORM_REDUCED_AXES_ATTRIBUTE)?;
        let intermediate_shape = value.shape().without_axes(&axes);
        let sum = SumPlan::for_boundaries(
            value.value_type(),
            value.shape(),
            &intermediate_shape,
            &axes,
        )?
        .squaring_contributors(multiply_f32_scalar_op());
        let extent_bits = folded_extent_bits(sum.reduced_points)?;
        Ok(Self {
            sum,
            extent_bits,
            eps: eps.clone(),
            intermediate_shape,
            shape: result.shape().clone(),
            value_type: expected,
        })
    }

    /// Emits stage zero: the squared fold and the epilogue it publishes.
    fn emit_fold(
        &self,
        context: &mut IndexAccessLoweringContext<'_>,
    ) -> Result<(), LoweringEmitError> {
        let kept = declare_kept_domain(context, &self.sum)?;
        let kept_coordinates = dimension_expressions(context, &kept)?;
        let input = context.input_tensor(self.value_type.clone(), self.sum.input_shape.clone())?;
        let output =
            context.output_tensor(self.value_type.clone(), self.sum.output_shape.clone())?;
        let total = self.sum.fold(context, input, &kept, &kept_coordinates)?;
        let extent = apply_one(
            context,
            constant_f32_scalar_op(),
            f32_constant_attributes(self.extent_bits)?,
            &[],
            "rms-scale-extent-constant",
        )?;
        let mean = apply_one(
            context,
            divide_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[total, extent],
            "rms-scale-mean",
        )?;
        let bias = apply_one(
            context,
            constant_f32_scalar_op(),
            scalar_attributes(self.eps.clone())?,
            &[],
            "rms-scale-eps-constant",
        )?;
        let biased = apply_one(
            context,
            add_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[mean, bias],
            "rms-scale-bias",
        )?;
        let root = apply_one(
            context,
            rsqrt_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[biased],
            "rms-scale-root",
        )?;
        let write = context.write(output, &kept, &kept_coordinates)?;
        context.output(write, root)
    }

    /// Emits stage one: the weighted scale over the published value.
    fn emit_scale(
        &self,
        context: &mut IndexAccessLoweringContext<'_>,
    ) -> Result<(), LoweringEmitError> {
        let dimensions = declare_parallel_domain(context, &self.shape)?;
        let coordinates = dimension_expressions(context, &dimensions)?;
        // The published value is one per folded row, so it is read at the kept
        // coordinates of this stage's own point domain rather than pointwise or
        // once for the whole region.
        let kept: Vec<_> = dimensions
            .iter()
            .zip(&self.sum.reduced)
            .filter(|(_, reduced)| !**reduced)
            .map(|(dimension, _)| *dimension)
            .collect();
        let kept_coordinates: Vec<_> = coordinates
            .iter()
            .zip(&self.sum.reduced)
            .filter(|(_, reduced)| !**reduced)
            .map(|(coordinate, _)| *coordinate)
            .collect();
        let value_tensor = context.input_tensor(self.value_type.clone(), self.shape.clone())?;
        let weight_tensor = context.input_tensor(self.value_type.clone(), self.shape.clone())?;
        let root_tensor =
            context.input_tensor(self.value_type.clone(), self.intermediate_shape.clone())?;
        let element = context.read(value_tensor, &dimensions, &coordinates)?;
        let weight_element = context.read(weight_tensor, &dimensions, &coordinates)?;
        let root = context.read(root_tensor, &kept, &kept_coordinates)?;
        let scaled = apply_one(
            context,
            multiply_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[element, root],
            "rms-scale-scaled",
        )?;
        let weighted = apply_one(
            context,
            multiply_f32_scalar_op(),
            ScalarAttributes::empty(),
            &[weight_element, scaled],
            "rms-scale-weighted",
        )?;
        let output = context.output_tensor(self.value_type.clone(), self.shape.clone())?;
        let write = context.write(output, &dimensions, &coordinates)?;
        context.output(write, weighted)
    }
}

/// Returns the exact binary32 payload of the folded contributor count.
///
/// The pinned reference divides by the extent itself — never by a reciprocal,
/// and therefore never by a divisor that is merely close to it. Above the
/// binary32 significand's width the integers are not all representable, so a
/// count whose nearest binary32 is not the count would make the emitted division
/// a different function from the one the operation pins; it is refused rather
/// than rounded. The representability test is integer-only, so it does not
/// depend on the rounding it exists to detect: an integer is a binary32 value
/// exactly when its odd part fits in the twenty-four-bit significand.
///
/// An empty fold is refused because it has no first contributor to seed at, so
/// the reference's own fold is undefined before the division by zero is reached.
/// This is the same derivation the registered law states, restated here because
/// the provider must refuse what the law refuses rather than emit a region the
/// comparison would then reject for a reason naming the wrong authority.
fn folded_extent_bits(points: u64) -> Result<u32, LoweringEmitError> {
    if points == 0 {
        return Err(occurrence_error("rms-scale-empty-fold"));
    }
    if points >> points.trailing_zeros() >= 1 << 24 {
        return Err(occurrence_error("rms-scale-extent-not-exact"));
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "the representability test above proves this conversion is exact"
    )]
    let extent = points as f32;
    Ok(extent.to_bits())
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
    fn derive(
        occurrence: crate::capability::IndexAccessOccurrence<'_>,
    ) -> Result<Self, LoweringEmitError> {
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
///
/// The field identifier is a parameter because attribute identifiers are
/// record-local: the strict serial sum numbers its axes field one and so does
/// the normalization, and a reader hard-coding either would build the other
/// family's record correctly only by that coincidence.
fn reduction_axes(
    attributes: &OperationAttributes,
    field: AttributeFieldId,
) -> Result<Vec<Axis>, LoweringEmitError> {
    let Some(value) = attributes.get(field) else {
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
    use super::{
        GovernedPointwiseF32, PointwiseScalar, governed_lowering_capabilities,
        governed_realization_laws, governed_scalars,
    };
    use crate::capability::{
        IndexAccessLoweringContext, IndexAccessLoweringProvider, LoweringCapabilityRegistryBuilder,
        LoweringCapabilityRevision, LoweringEmitError, LoweringSignature,
    };
    use crate::legality::{IndexRefinement, RefinementError, refine_index_region};
    use std::sync::Arc;
    use tiler_ir::index::{
        DomainRole, IndexInteger, IndexRefinementSubject, NumericalContractIdentity,
        ScalarAttributes, add_f32_scalar_op, constant_f32_scalar_op, divide_f32_scalar_op,
        multiply_f32_scalar_op, rsqrt_f32_scalar_op, strict_affine_u4_dequantize_scalar_op,
    };
    use tiler_ir::program::SemanticOccurrence;
    use tiler_ir::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS;
    use tiler_ir::semantic::{
        BROADCAST_AXIS_MAPPING_ATTRIBUTE, BroadcastAxisMapping, BroadcastAxisSource,
        CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, CanonicalField, CanonicalValue, ContractionIndex,
        ContractionIndexStructure, F32, F32_CONSTANT_BITS_ATTRIBUTE, InputKey, OpKey,
        OperationAttributes, OutputKey, ProviderIdentity, REDUCTION_AXES_ATTRIBUTE,
        REINDEX_MAPPING_ATTRIBUTE, RMS_NORM_EPS_BITS_ATTRIBUTE, RMS_NORM_REDUCED_AXES_ATTRIBUTE,
        ReindexForm, ResolvedValueType, STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE,
        STRICT_AFFINE_ZERO_POINT_ROLE, SemanticProgramBuilder, StrictAffineU4, TypeKey, U4,
        add_f32_op, broadcast_f32_op, constant_f32_op, dequantize_strict_affine_op,
        multiply_f32_op, reindex_f32_op, rms_norm_f32_eps_attribute, rms_norm_f32_op,
        strict_serial_sum_f32_op, strict_tensor_contraction_f32_op,
    };

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    struct OccurrenceValueId(u32);

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct OccurrenceOperand {
        value: OccurrenceValueId,
        value_type: ResolvedValueType,
        shape: Shape,
    }

    impl OccurrenceOperand {
        const fn new(
            value: OccurrenceValueId,
            value_type: ResolvedValueType,
            shape: Shape,
        ) -> Self {
            Self {
                value,
                value_type,
                shape,
            }
        }
        const fn value(&self) -> OccurrenceValueId {
            self.value
        }
        const fn value_type(&self) -> &ResolvedValueType {
            &self.value_type
        }
        const fn shape(&self) -> &Shape {
            &self.shape
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    struct OccurrenceResult {
        value_type: ResolvedValueType,
        shape: Shape,
    }

    impl OccurrenceResult {
        const fn new(value_type: ResolvedValueType, shape: Shape) -> Self {
            Self { value_type, shape }
        }
        const fn value_type(&self) -> &ResolvedValueType {
            &self.value_type
        }
        const fn shape(&self) -> &Shape {
            &self.shape
        }
    }
    use tiler_ir::shape::{Axis, Extent, Shape};
    use tiler_reference::{
        FloatBitOrder, FrozenReferenceRegistry, FrozenScalarReferenceRegistry,
        IndexRegionAuthority, IndexRegionEvaluationError, IndexRegionEvaluator, IndexRegionInput,
        ReferenceElement, ReferenceOperationError, Tensor, TensorPayloadView,
    };

    fn f32_type() -> ResolvedValueType {
        F32::resolved_type()
    }

    fn contract() -> NumericalContractIdentity {
        NumericalContractIdentity::try_from_key(
            crate::request::StrictF32NumericalContract::governed().key,
        )
        .unwrap()
    }

    fn strict_affine_provider_error(
        name: &str,
        provider: Arc<dyn IndexAccessLoweringProvider>,
    ) -> RefinementError {
        let scalars = governed_scalars().unwrap();
        let mut lowerings = LoweringCapabilityRegistryBuilder::new(
            scalars.semantic_authority().clone(),
            scalars.clone(),
        )
        .unwrap();
        lowerings
            .register_index_access(
                ProviderIdentity::new("test", name, 1).unwrap(),
                dequantize_strict_affine_op(),
                LoweringSignature::new([StrictAffineU4::resolved_type()], [F32::resolved_type()])
                    .unwrap(),
                &[strict_affine_u4_dequantize_scalar_op()],
                LoweringCapabilityRevision::new(1).unwrap(),
                provider,
            )
            .unwrap();
        let lowerings = lowerings.freeze();
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let input = program
            .input_resolved(
                InputKey::new("encoded").unwrap(),
                Shape::from_dims([5]),
                StrictAffineU4::resolved_type(),
            )
            .unwrap();
        let result = program
            .apply(
                dequantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[input],
            )
            .unwrap()[0];
        program
            .output_resolved(OutputKey::new("result").unwrap(), result)
            .unwrap();
        let program = program.build().unwrap();
        let subject = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            contract(),
        )
        .unwrap();
        let signature = LoweringSignature::new(
            subject.signature().operands().iter().cloned(),
            subject.signature().results().iter().cloned(),
        )
        .unwrap();
        let capability = lowerings
            .resolve_index_access(subject.operation(), &signature)
            .unwrap();
        refine_index_region(
            &capability,
            &subject,
            &governed_realization_laws(&scalars),
            &scalars,
        )
        .unwrap_err()
    }

    struct ReversedStrictAffineU4;

    struct SwappedStrictAffineComponents;

    impl IndexAccessLoweringProvider for SwappedStrictAffineComponents {
        fn lower(
            &self,
            context: &mut IndexAccessLoweringContext<'_>,
        ) -> Result<(), LoweringEmitError> {
            let occurrence = context.occurrence();
            let ([input], [result]) = (occurrence.inputs(), occurrence.results()) else {
                return Err(super::occurrence_error("strict-affine-test-arity"));
            };
            let shape = result.shape().clone();
            let result_type = result.value_type().clone();
            let input_shape = input.shape().clone();
            let dimension = context.dimension(DomainRole::Parallel, shape.extents()[0])?;
            let coordinate = context.dimension_expr(dimension)?;
            let codes = context.input_tensor(U4::resolved_type(), input_shape)?;
            // Deliberately reverse the two rank-zero component boundaries while
            // preserving their uses, so the region remains structurally valid.
            let zero = context.input_tensor(U4::resolved_type(), Shape::new([]))?;
            let scale = context.input_tensor(F32::resolved_type(), Shape::new([]))?;
            let codes = context.read(codes, &[dimension], &[coordinate])?;
            let scale = context.read(scale, &[], &[])?;
            let zero = context.read(zero, &[], &[])?;
            let value = context
                .apply(
                    strict_affine_u4_dequantize_scalar_op(),
                    ScalarAttributes::empty(),
                    &[codes, scale, zero],
                )?
                .get(0)
                .unwrap();
            let output = context.output_tensor(result_type, shape)?;
            let write = context.write(output, &[dimension], &[coordinate])?;
            context.output(write, value)
        }
    }

    struct ScalarCodesStrictAffine;

    impl IndexAccessLoweringProvider for ScalarCodesStrictAffine {
        fn lower(
            &self,
            context: &mut IndexAccessLoweringContext<'_>,
        ) -> Result<(), LoweringEmitError> {
            let occurrence = context.occurrence();
            let ([_], [result]) = (occurrence.inputs(), occurrence.results()) else {
                return Err(super::occurrence_error("strict-affine-test-arity"));
            };
            let shape = result.shape().clone();
            let result_type = result.value_type().clone();
            let dimension = context.dimension(DomainRole::Parallel, shape.extents()[0])?;
            let coordinate = context.dimension_expr(dimension)?;
            let codes = context.input_tensor(U4::resolved_type(), Shape::new([]))?;
            let scale = context.input_tensor(F32::resolved_type(), Shape::new([]))?;
            let zero = context.input_tensor(U4::resolved_type(), Shape::new([]))?;
            let codes = context.read(codes, &[], &[])?;
            let scale = context.read(scale, &[], &[])?;
            let zero = context.read(zero, &[], &[])?;
            let value = context
                .apply(
                    strict_affine_u4_dequantize_scalar_op(),
                    ScalarAttributes::empty(),
                    &[codes, scale, zero],
                )?
                .get(0)
                .unwrap();
            let output = context.output_tensor(result_type, shape)?;
            let write = context.write(output, &[dimension], &[coordinate])?;
            context.output(write, value)
        }
    }

    impl IndexAccessLoweringProvider for ReversedStrictAffineU4 {
        fn lower(
            &self,
            context: &mut IndexAccessLoweringContext<'_>,
        ) -> Result<(), LoweringEmitError> {
            let occurrence = context.occurrence();
            let ([input], [result]) = (occurrence.inputs(), occurrence.results()) else {
                return Err(super::occurrence_error("strict-affine-test-arity"));
            };
            let (_, contract) = input.value_type().encoded_numeric_parts().unwrap();
            let components = contract
                .components()
                .iter()
                .map(|component| {
                    (
                        component.resolved_type().clone(),
                        component.shape_relation().component_shape(input.shape()),
                    )
                })
                .collect::<Vec<_>>();
            let shape = result.shape().clone();
            let result_type = result.value_type().clone();
            let dimension = context.dimension(DomainRole::Parallel, shape.extents()[0])?;
            let induction = context.dimension_expr(dimension)?;
            let reversed = context.linear_combination(
                IndexInteger::from_i128(i128::from(shape.extents()[0].get()) - 1),
                &[(IndexInteger::from_i128(-1), induction)],
            )?;
            let tensors = components
                .into_iter()
                .map(|(value_type, shape)| context.input_tensor(value_type, shape))
                .collect::<Result<Vec<_>, _>>()?;
            let codes = context.read(tensors[0], &[dimension], &[reversed])?;
            let scale = context.read(tensors[1], &[], &[])?;
            let zero = context.read(tensors[2], &[], &[])?;
            let value = context
                .apply(
                    strict_affine_u4_dequantize_scalar_op(),
                    ScalarAttributes::empty(),
                    &[codes, scale, zero],
                )?
                .get(0)
                .unwrap();
            let output = context.output_tensor(result_type, shape)?;
            let write = context.write(output, &[dimension], &[reversed])?;
            context.output(write, value)
        }
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

    /// The normalization's own two-field attribute record.
    ///
    /// Its field identifiers are the family's, not the serial sum's: attribute
    /// identifiers are record-local and both families number an axes field one.
    fn rms_norm_attributes(axes: &[u32], eps_bits: u32) -> OperationAttributes {
        OperationAttributes::new([
            CanonicalField::new(
                RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                CanonicalValue::sequence(axes.iter().copied().map(CanonicalValue::unsigned_u32))
                    .unwrap(),
            ),
            CanonicalField::new(
                RMS_NORM_EPS_BITS_ATTRIBUTE,
                rms_norm_f32_eps_attribute(eps_bits),
            ),
        ])
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
        let realizations = governed_realization_laws(&scalars);
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let mut inputs = Vec::new();
        for operand in &operands {
            if inputs.iter().any(|(value, _)| *value == operand.value()) {
                continue;
            }
            let input = builder
                .input_resolved(
                    InputKey::new(format!("input-{}", operand.value().0)).unwrap(),
                    operand.shape().clone(),
                    operand.value_type().clone(),
                )
                .unwrap();
            inputs.push((operand.value(), input));
        }
        let ordered = operands
            .into_iter()
            .map(|operand| {
                inputs
                    .iter()
                    .find(|(value, _)| *value == operand.value())
                    .unwrap()
                    .1
            })
            .collect::<Vec<_>>();
        let produced = builder.apply(operation, attributes, &ordered).unwrap();
        assert_eq!(produced.len(), results.len());
        for (position, value) in produced.into_iter().enumerate() {
            builder
                .output_resolved(OutputKey::new(format!("result-{position}")).unwrap(), value)
                .unwrap();
        }
        let program = builder.build().unwrap();
        let occurrence = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            contract(),
        )
        .unwrap();
        for (actual, expected) in occurrence.results().iter().zip(results) {
            assert_eq!(actual.value_type(), expected.value_type());
            assert_eq!(actual.shape(), expected.shape());
        }
        let signature = LoweringSignature::new(
            occurrence.signature().operands().iter().cloned(),
            occurrence.signature().results().iter().cloned(),
        )
        .unwrap();
        let resolved = registry
            .resolve_index_access(occurrence.operation(), &signature)
            .unwrap();
        refine_index_region(&resolved, &occurrence, &realizations, &scalars)
            .unwrap_or_else(|error| panic!("governed lowering must refine: {error:?}"))
            .into_refined()
            .expect("governed lowering discharges every index-domain predicate")
    }

    /// The normalization's two-region realization refines through the ordinary
    /// path, and both stages are the ones the registered law requires.
    ///
    /// **What this moves.** `tiler::rms-norm-f32@1` carried a registered law and
    /// no provider, so `refine_index_region` reached a resolved capability that
    /// emitted nothing — the wall
    /// `crates/tiler-compiler/tests/two_region_occurrence_lowering.rs` records
    /// with a counting fixture. The governed profile now emits the chain, so the
    /// family has refinement evidence.
    ///
    /// **Every assertion below is about the chain rather than about success.**
    /// A refinement that merely returned would be consistent with a one-region
    /// realization certified against the wrong law, so the stage count, the one
    /// handed value's producer, consumer, and reduced shape, the two stages'
    /// genuinely different reached scalar authorities, and the operand bindings
    /// naming two different stages are each checked. The reached sets are the
    /// sharpest of these: the fold reaches the square, the combine, the
    /// division, the bias, and the root, while the pass reaches the multiply
    /// alone — so a provider that put the epilogue in the pass would fail here
    /// even though its arithmetic composes to the same values.
    #[test]
    fn the_governed_normalization_lowering_refines_its_two_stage_occurrence() {
        let shape = Shape::from_dims([2, 4]);
        let refinement = refine(
            rms_norm_f32_op(),
            vec![
                OccurrenceOperand::new(OccurrenceValueId(0), f32_type(), shape.clone()),
                OccurrenceOperand::new(OccurrenceValueId(1), f32_type(), shape.clone()),
            ],
            vec![OccurrenceResult::new(f32_type(), shape)],
            rms_norm_attributes(&[1], 1.0e-6_f32.to_bits()),
        );

        let realization = refinement.realization();
        assert_eq!(realization.stage_count(), 2);
        let [intermediate] = realization.intermediates() else {
            panic!("a two-stage chain hands exactly one value on")
        };
        assert_eq!(intermediate.producer(), 0);
        assert_eq!(intermediate.consumer(), 1);
        assert_eq!(
            intermediate.shape(),
            &Shape::from_dims([2]),
            "the published value is one per folded row"
        );
        assert_eq!(intermediate.value_type(), &f32_type());

        let authorities = refinement.content().scalar_authorities();
        assert_eq!(authorities.len(), 2);
        assert_eq!(
            authorities[0].reached_operations(),
            // Canonical key order, which is what a reached set is retained in.
            [
                add_f32_scalar_op(),
                constant_f32_scalar_op(),
                divide_f32_scalar_op(),
                multiply_f32_scalar_op(),
                rsqrt_f32_scalar_op(),
            ],
            "the fold stage carries the square, the combine, and the whole epilogue"
        );
        assert_eq!(
            authorities[1].reached_operations(),
            [multiply_f32_scalar_op()],
            "the pass stage carries the two products and nothing else"
        );

        assert_eq!(
            refinement
                .operand_bindings()
                .iter()
                .map(|binding| (binding.operand(), binding.stage()))
                .collect::<Vec<_>>(),
            vec![(0, 0), (0, 1), (1, 1)],
            "the value operand is read by both stages — the fold squares it and \
             the pass scales it — while the weight is the pass's alone"
        );
        assert_eq!(refinement.result_bindings().len(), 1);
        assert!(
            refinement.single_region().is_none(),
            "no single-region view of a chain is offered"
        );
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
            .realization_identity()
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
    fn the_governed_strict_affine_u4_lowering_retains_real_component_receipts() {
        let shape = Shape::from_dims([5]);
        let refinement = refine(
            dequantize_strict_affine_op(),
            vec![OccurrenceOperand::new(
                OccurrenceValueId(0),
                StrictAffineU4::resolved_type(),
                shape.clone(),
            )],
            vec![OccurrenceResult::new(F32::resolved_type(), shape)],
            OperationAttributes::empty(),
        );
        let bindings = refinement.operand_bindings();
        assert_eq!(bindings.len(), 3);
        assert_eq!(
            bindings
                .iter()
                .map(tiler_ir::index::OperandBinding::component_role)
                .collect::<Vec<_>>(),
            [
                Some(STRICT_AFFINE_CODES_ROLE),
                Some(STRICT_AFFINE_SCALE_ROLE),
                Some(STRICT_AFFINE_ZERO_POINT_ROLE),
            ]
        );
        assert!(bindings.iter().all(|binding| binding.operand() == 0));
        assert!(bindings.iter().all(|binding| binding.input() == 0));
        assert_eq!(refinement.result_bindings().len(), 1);
        assert_eq!(
            refinement.scalar_authority().reached_operations(),
            [strict_affine_u4_dequantize_scalar_op()]
        );
        assert_eq!(
            refinement.receipt().occurrence(),
            SemanticOccurrence::new(0)
        );
    }

    #[test]
    fn the_governed_strict_affine_u4_region_executes_exact_widened_decode() {
        let shape = Shape::from_dims([5]);
        let refinement = refine(
            dequantize_strict_affine_op(),
            vec![OccurrenceOperand::new(
                OccurrenceValueId(0),
                StrictAffineU4::resolved_type(),
                shape.clone(),
            )],
            vec![OccurrenceResult::new(F32::resolved_type(), shape.clone())],
            OperationAttributes::empty(),
        );
        let element = |bytes: &[u8]| ReferenceElement::new(bytes).unwrap();
        let codes = Tensor::dense(
            U4::resolved_type(),
            shape,
            [0_u8, 1, 7, 8, 15].map(|value| element(&[value])).to_vec(),
        )
        .unwrap();
        let scale = Tensor::scalar(
            F32::resolved_type(),
            element(&0.5_f32.to_bits().to_be_bytes()),
        )
        .unwrap();
        let zero_point = Tensor::scalar(U4::resolved_type(), element(&[7_u8])).unwrap();
        let bindings = refinement.operand_bindings();
        let inputs = [
            IndexRegionInput::new(bindings[0].input_tensor(), &codes),
            IndexRegionInput::new(bindings[1].input_tensor(), &scale),
            IndexRegionInput::new(bindings[2].input_tensor(), &zero_point),
        ];
        let scalars = governed_scalars().unwrap();
        let evaluator = IndexRegionEvaluator::new(
            FrozenReferenceRegistry::standard().unwrap(),
            FrozenScalarReferenceRegistry::standard().unwrap(),
        );
        let evaluation = evaluator
            .evaluate(
                refinement
                    .single_region()
                    .expect("every governed family realizes its occurrence in one region"),
                IndexRegionAuthority::new(&scalars),
                &inputs,
            )
            .unwrap();
        assert_eq!(
            output_bits(&evaluation.outputs()[0]),
            [-3.5_f32, -3.0, 0.0, 0.5, 4.0].map(f32::to_bits).to_vec()
        );
    }

    #[test]
    fn strict_affine_u4_scalar_covers_both_centered_boundaries() {
        for (code, zero_point, expected) in [(0_u8, 15_u8, -15.0_f32), (15, 0, 15.0)] {
            let shape = Shape::from_dims([1]);
            let refinement = refine(
                dequantize_strict_affine_op(),
                vec![OccurrenceOperand::new(
                    OccurrenceValueId(0),
                    StrictAffineU4::resolved_type(),
                    shape.clone(),
                )],
                vec![OccurrenceResult::new(F32::resolved_type(), shape.clone())],
                OperationAttributes::empty(),
            );
            let element = |bytes: &[u8]| ReferenceElement::new(bytes).unwrap();
            let codes = Tensor::dense(U4::resolved_type(), shape, vec![element(&[code])]).unwrap();
            let scale = Tensor::scalar(
                F32::resolved_type(),
                element(&1.0_f32.to_bits().to_be_bytes()),
            )
            .unwrap();
            let zero_point = Tensor::scalar(U4::resolved_type(), element(&[zero_point])).unwrap();
            let bindings = refinement.operand_bindings();
            let inputs = [
                IndexRegionInput::new(bindings[0].input_tensor(), &codes),
                IndexRegionInput::new(bindings[1].input_tensor(), &scale),
                IndexRegionInput::new(bindings[2].input_tensor(), &zero_point),
            ];
            let scalars = governed_scalars().unwrap();
            let evaluation = IndexRegionEvaluator::new(
                FrozenReferenceRegistry::standard().unwrap(),
                FrozenScalarReferenceRegistry::standard().unwrap(),
            )
            .evaluate(
                refinement
                    .single_region()
                    .expect("every governed family realizes its occurrence in one region"),
                IndexRegionAuthority::new(&scalars),
                &inputs,
            )
            .unwrap();
            assert_eq!(output_bits(&evaluation.outputs()[0]), [expected.to_bits()]);
        }
    }

    #[test]
    fn strict_affine_u4_scalar_refuses_every_invalid_scale_class() {
        let shape = Shape::from_dims([1]);
        let refinement = refine(
            dequantize_strict_affine_op(),
            vec![OccurrenceOperand::new(
                OccurrenceValueId(0),
                StrictAffineU4::resolved_type(),
                shape.clone(),
            )],
            vec![OccurrenceResult::new(F32::resolved_type(), shape.clone())],
            OperationAttributes::empty(),
        );
        let element = |bytes: &[u8]| ReferenceElement::new(bytes).unwrap();
        let codes = Tensor::dense(U4::resolved_type(), shape, vec![element(&[15])]).unwrap();
        let zero_point = Tensor::scalar(U4::resolved_type(), element(&[0])).unwrap();
        let bindings = refinement.operand_bindings();
        let scalars = governed_scalars().unwrap();
        let evaluator = IndexRegionEvaluator::new(
            FrozenReferenceRegistry::standard().unwrap(),
            FrozenScalarReferenceRegistry::standard().unwrap(),
        );
        for scale_bits in [
            0.0_f32.to_bits(),
            (-0.0_f32).to_bits(),
            (-1.0_f32).to_bits(),
            0x0000_0001,
            0x8000_0001,
            f32::NAN.to_bits(),
            f32::INFINITY.to_bits(),
            f32::NEG_INFINITY.to_bits(),
        ] {
            let scale =
                Tensor::scalar(F32::resolved_type(), element(&scale_bits.to_be_bytes())).unwrap();
            let inputs = [
                IndexRegionInput::new(bindings[0].input_tensor(), &codes),
                IndexRegionInput::new(bindings[1].input_tensor(), &scale),
                IndexRegionInput::new(bindings[2].input_tensor(), &zero_point),
            ];
            assert!(matches!(
                evaluator.evaluate(
                    refinement
                        .single_region()
                        .expect("every governed family realizes its occurrence in one region"),
                    IndexRegionAuthority::new(&scalars),
                    &inputs,
                ),
                Err(IndexRegionEvaluationError::ScalarOperation {
                    source: ReferenceOperationError::InvalidApplication,
                    ..
                })
            ));
        }
    }

    #[test]
    fn a_non_strict_numerical_contract_mints_no_strict_affine_receipt() {
        let scalars = governed_scalars().unwrap();
        let lowerings = governed_lowering_capabilities(&scalars).unwrap();
        let realizations = governed_realization_laws(&scalars);
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let input = program
            .input_resolved(
                InputKey::new("encoded").unwrap(),
                Shape::from_dims([2]),
                StrictAffineU4::resolved_type(),
            )
            .unwrap();
        let result = program
            .apply(
                dequantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[input],
            )
            .unwrap()[0];
        program
            .output_resolved(OutputKey::new("result").unwrap(), result)
            .unwrap();
        let program = program.build().unwrap();
        let relaxed = NumericalContractIdentity::try_from_key(
            crate::request::StrictF32NumericalContract::governed_flush_to_zero().key,
        )
        .unwrap();
        let subject = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            relaxed,
        )
        .unwrap();
        let signature = LoweringSignature::new(
            subject.signature().operands().iter().cloned(),
            subject.signature().results().iter().cloned(),
        )
        .unwrap();
        let capability = lowerings
            .resolve_index_access(subject.operation(), &signature)
            .unwrap();
        let error =
            refine_index_region(&capability, &subject, &realizations, &scalars).unwrap_err();
        let RefinementError::IrVerifier(source) = error else {
            panic!("numerical mismatch must remain typed: {error:?}");
        };
        assert!(matches!(
            source.as_ref(),
            tiler_ir::index::IndexRefinementVerificationError::NumericalContractNotGoverned
        ));
    }

    #[test]
    fn a_strict_affine_capability_cannot_mint_for_another_operation() {
        let scalars = governed_scalars().unwrap();
        let lowerings = governed_lowering_capabilities(&scalars).unwrap();
        let mut strict_program = SemanticProgramBuilder::try_standard().unwrap();
        let encoded = strict_program
            .input_resolved(
                InputKey::new("encoded").unwrap(),
                Shape::from_dims([2]),
                StrictAffineU4::resolved_type(),
            )
            .unwrap();
        let decoded = strict_program
            .apply(
                dequantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[encoded],
            )
            .unwrap()[0];
        strict_program
            .output_resolved(OutputKey::new("decoded").unwrap(), decoded)
            .unwrap();
        let strict_program = strict_program.build().unwrap();
        let strict_subject = IndexRefinementSubject::derive(
            &strict_program,
            strict_program.operations().next().unwrap().id(),
            contract(),
        )
        .unwrap();
        let strict_signature = LoweringSignature::new(
            strict_subject.signature().operands().iter().cloned(),
            strict_subject.signature().results().iter().cloned(),
        )
        .unwrap();
        let strict_capability = lowerings
            .resolve_index_access(strict_subject.operation(), &strict_signature)
            .unwrap();

        let mut multiply_program = SemanticProgramBuilder::try_standard().unwrap();
        let left = multiply_program
            .input::<F32>(InputKey::new("left").unwrap(), Shape::from_dims([2]))
            .unwrap();
        let right = multiply_program
            .input::<F32>(InputKey::new("right").unwrap(), Shape::from_dims([2]))
            .unwrap();
        let product =
            tiler_ir::semantic::F32Multiply::apply(&mut multiply_program, left, right).unwrap();
        multiply_program
            .output(OutputKey::new("product").unwrap(), product)
            .unwrap();
        let multiply_program = multiply_program.build().unwrap();
        let multiply_subject = IndexRefinementSubject::derive(
            &multiply_program,
            multiply_program.operations().next().unwrap().id(),
            contract(),
        )
        .unwrap();
        assert!(matches!(
            refine_index_region(
                &strict_capability,
                &multiply_subject,
                &governed_realization_laws(&scalars),
                &scalars,
            ),
            Err(RefinementError::OperationMismatch { .. })
        ));
    }

    #[test]
    fn a_semantically_equivalent_noncanonical_strict_affine_region_mints_no_receipt() {
        let scalars = governed_scalars().unwrap();
        let mut lowerings = LoweringCapabilityRegistryBuilder::new(
            scalars.semantic_authority().clone(),
            scalars.clone(),
        )
        .unwrap();
        lowerings
            .register_index_access(
                ProviderIdentity::new("test", "reversed-strict-affine", 1).unwrap(),
                dequantize_strict_affine_op(),
                LoweringSignature::new([StrictAffineU4::resolved_type()], [F32::resolved_type()])
                    .unwrap(),
                &[strict_affine_u4_dequantize_scalar_op()],
                LoweringCapabilityRevision::new(1).unwrap(),
                Arc::new(ReversedStrictAffineU4),
            )
            .unwrap();
        let lowerings = lowerings.freeze();
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let input = program
            .input_resolved(
                InputKey::new("encoded").unwrap(),
                Shape::from_dims([5]),
                StrictAffineU4::resolved_type(),
            )
            .unwrap();
        let result = program
            .apply(
                dequantize_strict_affine_op(),
                OperationAttributes::empty(),
                &[input],
            )
            .unwrap()[0];
        program
            .output_resolved(OutputKey::new("result").unwrap(), result)
            .unwrap();
        let program = program.build().unwrap();
        let subject = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            contract(),
        )
        .unwrap();
        let signature = LoweringSignature::new(
            subject.signature().operands().iter().cloned(),
            subject.signature().results().iter().cloned(),
        )
        .unwrap();
        let capability = lowerings
            .resolve_index_access(subject.operation(), &signature)
            .unwrap();
        let laws = governed_realization_laws(&scalars);
        let error = refine_index_region(&capability, &subject, &laws, &scalars).unwrap_err();
        let RefinementError::IrVerifier(source) = error else {
            panic!("canonical mismatch must remain typed: {error:?}");
        };
        assert!(matches!(
            source.as_ref(),
            tiler_ir::index::IndexRefinementVerificationError::SemanticRealizationMismatch { .. }
        ));
    }

    #[test]
    fn reordered_strict_affine_components_mint_no_receipt() {
        assert!(matches!(
            strict_affine_provider_error(
                "swapped-strict-affine-components",
                Arc::new(SwappedStrictAffineComponents),
            ),
            RefinementError::OperandInterface { position: 1 }
        ));
    }

    #[test]
    fn wrong_strict_affine_component_shape_mints_no_receipt() {
        assert!(matches!(
            strict_affine_provider_error(
                "scalar-strict-affine-codes",
                Arc::new(ScalarCodesStrictAffine),
            ),
            RefinementError::OperandInterface { position: 0 }
        ));
    }

    /// Deliberate ADR 0078 perturbation: the selected multiply capability emits
    /// add while every structural/interface fact remains valid. The semantic
    /// provider's multiply law is unchanged, so no receipt may mint.
    #[test]
    fn a_multiply_descriptor_that_emits_add_is_refused_by_semantic_law() {
        let scalars = governed_scalars().unwrap();
        let mut lowerings = LoweringCapabilityRegistryBuilder::new(
            scalars.semantic_authority().clone(),
            scalars.clone(),
        )
        .unwrap();
        lowerings
            .register_index_access(
                ProviderIdentity::new("test", "multiply-emits-add", 1).unwrap(),
                multiply_f32_op(),
                LoweringSignature::new(
                    [F32::resolved_type(), F32::resolved_type()],
                    [F32::resolved_type()],
                )
                .unwrap(),
                &[add_f32_scalar_op()],
                LoweringCapabilityRevision::new(1).unwrap(),
                Arc::new(GovernedPointwiseF32 {
                    scalar: PointwiseScalar::Add,
                }),
            )
            .unwrap();
        let lowerings = lowerings.freeze();
        let shape = Shape::from_dims([2]);
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let left = program
            .input::<F32>(InputKey::new("left").unwrap(), shape.clone())
            .unwrap();
        let right = program
            .input::<F32>(InputKey::new("right").unwrap(), shape)
            .unwrap();
        let product = tiler_ir::semantic::F32Multiply::apply(&mut program, left, right).unwrap();
        program
            .output(OutputKey::new("product").unwrap(), product)
            .unwrap();
        let program = program.build().unwrap();
        let subject = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            contract(),
        )
        .unwrap();
        let signature = LoweringSignature::new(
            subject.signature().operands().iter().cloned(),
            subject.signature().results().iter().cloned(),
        )
        .unwrap();
        let resolved = lowerings
            .resolve_index_access(&multiply_f32_op(), &signature)
            .unwrap();
        let laws = governed_realization_laws(&scalars);
        let error = refine_index_region(&resolved, &subject, &laws, &scalars).unwrap_err();
        let RefinementError::IrVerifier(source) = error else {
            panic!("semantic-law mismatch must remain typed: {error:?}");
        };
        assert!(matches!(
            source.as_ref(),
            tiler_ir::index::IndexRefinementVerificationError::SemanticRealizationMismatch { .. }
        ));
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
                refinement
                    .single_region()
                    .expect("every governed family realizes its occurrence in one region"),
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

    /// A refinement subject cannot independently declare a result shape that
    /// disagrees with the semantic operation's inference.
    #[test]
    fn a_structural_subject_uses_the_semantically_derived_result_shape() {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let input = builder
            .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([6]))
            .unwrap();
        let [result] = builder
            .apply(
                reindex_f32_op(),
                reindex_attributes(
                    &ReindexForm::split_axis(Axis::new(0), [Extent::new(3), Extent::new(2)])
                        .unwrap(),
                ),
                &[input.erase()],
            )
            .unwrap()
            .try_into()
            .unwrap();
        builder
            .output_resolved(OutputKey::new("result").unwrap(), result)
            .unwrap();
        let program = builder.build().unwrap();
        let subject = IndexRefinementSubject::derive(
            &program,
            program.operations().next().unwrap().id(),
            contract(),
        )
        .unwrap();
        assert_eq!(subject.results()[0].shape(), &Shape::from_dims([3, 2]));
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

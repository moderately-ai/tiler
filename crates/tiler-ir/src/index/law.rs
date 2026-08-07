//! Closed, typed semantic laws for canonical logical index realization.
//!
//! A law is registered by the same semantic-provider transaction that defines
//! an operation. It is data, not a verdict callback: the verifier interprets it
//! without exposing the candidate region, builds the expected canonical region,
//! and compares the two only after both have passed ordinary structural checks.

use core::fmt;
use std::error::Error;

use crate::schedule::{
    ApproximationEnvelope, ExceptionalValueAssumption, F32NumericalContractKey,
    MaterializationRounding, NumericalPermission, SubnormalMode,
};
use crate::semantic::{
    AttributeFieldId, BF16_CONSTANT_BITS_ATTRIBUTE, BROADCAST_AXIS_MAPPING_ATTRIBUTE,
    BroadcastAxisMapping, BroadcastAxisSource, CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE,
    CanonicalField, CanonicalIntegerWidth, CanonicalValue, CanonicalValueView, ContractionIndex,
    ContractionIndexStructure, EncodedComponentRole, F32_CONSTANT_BITS_ATTRIBUTE,
    OperationAttributes, REDUCTION_AXES_ATTRIBUTE, REINDEX_MAPPING_ATTRIBUTE,
    RMS_NORM_EPS_BITS_ATTRIBUTE, RMS_NORM_REDUCED_AXES_ATTRIBUTE, ReindexForm, ReindexFormKind,
    ResolvedValueType, STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE,
    STRICT_AFFINE_ZERO_POINT_ROLE, StrictAffineU4, TypeKey,
};
use crate::shape::{Axis, Extent, Shape};

use super::{
    DimensionId, DomainRole, FrozenScalarRegistry, IndexBuildError, IndexExprId, IndexInteger,
    IndexRefinementSubject, IndexRegionBuildError, IndexRegionBuilder, IndexRegionSequenceError,
    ScalarAttributes, ScalarOpKey, ScalarReducerBodyBuilder, ScalarValueId, SourcedExtent,
    StagedInputSource, SymbolicExtentError, TensorAccessId, TensorId, TensorRole,
    VerifiedIndexRegion, VerifiedIndexRegionSequence, add_bf16_scalar_op, add_f32_scalar_op,
    canonicalize_nan_f32_scalar_op, constant_bf16_scalar_op, constant_f32_scalar_op,
    divide_f32_scalar_op, exp_f32_scalar_op, multiply_bf16_scalar_op, multiply_f32_scalar_op,
    rsqrt_f32_scalar_op, strict_affine_u4_dequantize_scalar_op,
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
    /// A strict serial sum whose materialized result a pointwise pass consumes.
    ///
    /// The first law form whose realization is an ordered *sequence* of regions
    /// rather than one region. Stage zero folds operand zero over the axes the
    /// named attribute carries and publishes the fold; stage one applies `scalar`
    /// to operand one and that published value, pointwise over the result shape.
    ///
    /// **Why the sequence is the law's shape and not a physical choice.** The
    /// intermediate is *read more than once* — every point of stage one consumes
    /// the same fold — so a single-region spelling would either recompute the
    /// fold per point, which is a different scalar program, or need a value with
    /// no region-local definition. The materialization is therefore part of what
    /// the realization means, and the region sequence is where it is stated.
    /// Which memory the intermediate occupies, and whether a target can keep it
    /// in registers, remain physical planning's questions.
    ///
    /// This is the reduction-then-elementwise *shape*, and it is deliberately not
    /// the normalization's own law. `tiler::rms-norm-f32@1` folds a square rather
    /// than the operand's own elements, transforms the published fold three times
    /// before the pass consumes it, and reads three values in that pass; this form
    /// expresses none of the three, so the normalization carries
    /// [`Self::StagedRootMeanSquareScaleF32`] instead. What the two share is the
    /// staged emitters below, not a template.
    StagedStrictSerialSumThenPointwiseF32 {
        /// Attribute containing the ordered axes stage zero folds over.
        axes_attribute: AttributeFieldId,
        /// Scalar operation stage one applies to operand one and the fold.
        scalar: ScalarOpKey,
    },
    /// The governed root-mean-square scale: a squared fold, an epilogue, a weight.
    ///
    /// Stage zero folds the *square* of operand zero over the axes the named
    /// attribute carries, divides that fold by the folded contributor count,
    /// adds the exact payload the second named attribute carries, applies the
    /// reciprocal square root, and publishes the result. Stage one reads operand
    /// zero, operand one, and that published value, and writes
    /// `weight * (value * published)` pointwise over the result shape.
    ///
    /// **Why the chain is fixed and only the two attributes are law data.** Every
    /// other shape was eliminated against this vocabulary's own contract. Carrying
    /// the epilogue and the pass as *data* would need a scalar-program language
    /// inside a law, which is the universal IR this module's header refuses; and a
    /// template with five independently settable scalar keys is one whose complete
    /// interpretation is no longer owned here, because most of its 5-tuples denote
    /// programs nothing means. Fixing the chain and naming the attributes is the
    /// same split [`Self::ConstantFromFloatBits`] draws: what a second row of one
    /// template varies is its record-local field identifiers, which are
    /// record-local precisely because two families number them alike.
    ///
    /// **What generalizes instead is the emission machinery.** A fold carrying a
    /// per-contributor square, a fold whose value an epilogue transforms inside
    /// the producing region, and a pass reading a reduced-rank published value at
    /// its kept coordinates are three capabilities the staged vocabulary did not
    /// have. They are stated as reusable emitters, so the next staged family
    /// instantiates them rather than this template.
    StagedRootMeanSquareScaleF32 {
        /// Attribute containing the ordered axes stage zero folds over.
        axes_attribute: AttributeFieldId,
        /// Attribute containing the exact bias added inside the root's argument.
        ///
        /// Named rather than assumed, and checked against the occurrence's exact
        /// declared field set: the payload is part of the semantic operation's
        /// identity, so a realization that read the axes and ignored this would be
        /// a different operation wearing this one's law.
        eps_attribute: AttributeFieldId,
    },
    /// Per-point decode of one governed compound strict-affine U4 value.
    StrictAffineU4Dequantize {
        /// Ordered logical codes component role.
        codes_role: EncodedComponentRole,
        /// Ordered positive-normal scale component role.
        scale_role: EncodedComponentRole,
        /// Ordered logical zero-point component role.
        zero_point_role: EncodedComponentRole,
        /// Atomic scalar operation implementing the widened strict decode.
        scalar: ScalarOpKey,
    },
}

impl IndexRealizationLaw {
    pub(crate) fn accepts_numerical_contract(&self, subject: &IndexRefinementSubject) -> bool {
        match self {
            Self::StrictAffineU4Dequantize { .. } => {
                let strict = F32NumericalContractKey::new(
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
                .expect("the governed strict F32 contract is coherent");
                subject.numerical_contract().as_str() == strict.as_str()
            }
            Self::ConstantFromFloatBits { .. }
            | Self::PointwiseBinary { .. }
            | Self::PreciseSiluF32
            | Self::StrictSerialSumF32 { .. }
            | Self::Reindex { .. }
            | Self::Broadcast { .. }
            | Self::StagedStrictSerialSumThenPointwiseF32 { .. }
            | Self::StagedRootMeanSquareScaleF32 { .. }
            | Self::StrictTensorContractionF32 { .. } => governs_result_arithmetic(subject),
        }
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

    /// Standard constant-bf16 law.
    ///
    /// The same template the `f32` constant uses, carrying this family's own
    /// attribute and its own scalar. Neither the template nor its encoding tag
    /// is new: what distinguishes the two rows is the payload they carry, which
    /// is exactly the distinction the semantic keys already draw.
    #[must_use]
    pub fn constant_bf16() -> Self {
        Self::ConstantFromFloatBits {
            attribute: BF16_CONSTANT_BITS_ATTRIBUTE,
            scalar: constant_bf16_scalar_op(),
        }
    }

    /// Standard multiply-bf16 law.
    #[must_use]
    pub fn multiply_bf16() -> Self {
        Self::PointwiseBinary {
            scalar: multiply_bf16_scalar_op(),
        }
    }

    /// Standard add-bf16 law.
    #[must_use]
    pub fn add_bf16() -> Self {
        Self::PointwiseBinary {
            scalar: add_bf16_scalar_op(),
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

    /// Standard staged strict-serial-sum-then-multiply-f32 law.
    ///
    /// A constructor for the governed spelling of the staged form, one of the two
    /// of this law's ten variants whose realization is a region *sequence*; the
    /// other eight are single-region, and `realizes_region_sequence` decides which
    /// is which in one match. No standard operation carries this row: the
    /// normalization, which is the family this shape was derived for, needs a fold
    /// prologue, an epilogue, and a ternary pass that this form does not express
    /// and carries [`Self::staged_root_mean_square_scale_f32`] instead. So the
    /// law-registry sidecar is unchanged by this row's existence.
    #[must_use]
    pub fn staged_strict_serial_sum_then_multiply_f32() -> Self {
        Self::StagedStrictSerialSumThenPointwiseF32 {
            axes_attribute: REDUCTION_AXES_ATTRIBUTE,
            scalar: multiply_f32_scalar_op(),
        }
    }

    /// Standard root-mean-square scale law, as `tiler::rms-norm-f32@1` registers it.
    ///
    /// Names that family's own two attribute identifiers. They are record-local,
    /// so this constructor is what ties the general template to the one family
    /// whose record numbers its axes field one and its `eps` field two.
    #[must_use]
    pub const fn staged_root_mean_square_scale_f32() -> Self {
        Self::StagedRootMeanSquareScaleF32 {
            axes_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
            eps_attribute: RMS_NORM_EPS_BITS_ATTRIBUTE,
        }
    }

    /// Standard strict-affine U4-to-F32 decode law.
    #[must_use]
    pub fn strict_affine_u4_dequantize() -> Self {
        Self::StrictAffineU4Dequantize {
            codes_role: STRICT_AFFINE_CODES_ROLE,
            scale_role: STRICT_AFFINE_SCALE_ROLE,
            zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
            scalar: strict_affine_u4_dequantize_scalar_op(),
        }
    }

    /// Whether this law's realization is an ordered sequence of regions.
    ///
    /// Asked *before* any interface checking, so a caller offering one region
    /// for a staged law is told that rather than being told its lone region's
    /// boundaries disagree with the occurrence — which is true, but names the
    /// symptom instead of the mismatch.
    pub(crate) const fn realizes_region_sequence(&self) -> bool {
        matches!(
            self,
            Self::StagedStrictSerialSumThenPointwiseF32 { .. }
                | Self::StagedRootMeanSquareScaleF32 { .. }
        )
    }

    /// Builds the exact canonical region sequence required by this law.
    ///
    /// The candidate is intentionally absent from this API. A law can describe
    /// expected work but cannot inspect or approve provider output.
    ///
    /// Every single-region template answers a one-stage sequence, whose identity
    /// is its region's identity byte for byte, so the sequence vocabulary changes
    /// nothing a single-region law ever produced.
    pub(crate) fn realize_sequence(
        &self,
        subject: &IndexRefinementSubject,
        scalars: &FrozenScalarRegistry,
    ) -> Result<VerifiedIndexRegionSequence, IndexRealizationLawError> {
        match self {
            Self::StagedStrictSerialSumThenPointwiseF32 {
                axes_attribute,
                scalar,
            } => {
                realize_staged_sum_then_pointwise(subject, scalars, *axes_attribute, scalar.clone())
            }
            Self::StagedRootMeanSquareScaleF32 {
                axes_attribute,
                eps_attribute,
            } => realize_root_mean_square_scale(subject, scalars, *axes_attribute, *eps_attribute),
            _ => Ok(VerifiedIndexRegionSequence::single(
                self.realize(subject, scalars)?,
            )),
        }
    }

    /// Builds the exact canonical logical region required by a one-region law.
    ///
    /// # Errors
    ///
    /// A law whose realization is a region sequence refuses here rather than
    /// answering one of its stages, which would be a truncated realization
    /// wearing the shape of a complete one.
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
                Self::StrictAffineU4Dequantize {
                    codes_role,
                    scale_role,
                    zero_point_role,
                    scalar,
                } => realize_strict_affine_u4_dequantize(
                    &mut context,
                    [*codes_role, *scale_role, *zero_point_role],
                    scalar.clone(),
                )?,
                Self::StagedStrictSerialSumThenPointwiseF32 { .. }
                | Self::StagedRootMeanSquareScaleF32 { .. } => {
                    return Err(unsupported("staged-law-requires-region-sequence"));
                }
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
            Self::StrictAffineU4Dequantize {
                codes_role,
                scale_role,
                zero_point_role,
                scalar,
            } => {
                // Tag 8 is append-only. Tags 1..=7 and their payloads are
                // unchanged. A row is self-delimiting through the canonical
                // operation and provider encodings, fixed-width revision, and
                // the tagged law payload, so this form cannot equal an old row.
                output.push(8);
                for role in [codes_role, scale_role, zero_point_role] {
                    output.extend_from_slice(&role.get().to_be_bytes());
                }
                encode_scalar(output, scalar);
            }
            Self::StagedStrictSerialSumThenPointwiseF32 {
                axes_attribute,
                scalar,
            } => {
                // Tag 9 is append-only. Tags 1..=8 and their payloads are
                // unchanged, and no registered row carries this tag, so every
                // sidecar byte a law registry has ever encoded is untouched.
                //
                // Injectivity at this site: the first byte discriminates, and the
                // payload that follows is a fixed-width attribute identifier
                // followed by the self-delimiting scalar encoding — the same
                // shape tag 4 writes for its attribute and tag 2 for its scalar,
                // but no other tag writes both, so this form is reachable from
                // exactly one variant.
                output.push(9);
                output.extend_from_slice(&axes_attribute.get().to_be_bytes());
                encode_scalar(output, scalar);
            }
            Self::StagedRootMeanSquareScaleF32 {
                axes_attribute,
                eps_attribute,
            } => {
                // Tag 10 is append-only. Tags 1..=9 and their payloads are
                // unchanged, so every sidecar byte a law registry has ever
                // encoded is byte-identical under this addition; only the row
                // this variant newly occupies is added.
                //
                // Injectivity at this site. The first byte discriminates, so no
                // other variant's encoding can be read as this one whatever
                // follows. Within this tag the payload is two fixed-width
                // attribute identifiers written in a fixed order and nothing
                // else, so the map from `(axes_attribute, eps_attribute)` to
                // bytes is a pair of injections on disjoint fixed offsets and is
                // therefore itself injective: two rows differing in either field
                // differ in the four bytes that field owns. The pair is ordered
                // rather than a set, so the transposed row — axes and `eps`
                // exchanged — encodes distinctly, which matters because that
                // transposition is a real construction error rather than a
                // hypothetical one and `realize_root_mean_square_scale` must be
                // able to refuse it as a different law rather than the same one.
                output.push(10);
                output.extend_from_slice(&axes_attribute.get().to_be_bytes());
                output.extend_from_slice(&eps_attribute.get().to_be_bytes());
            }
        }
    }
}

/// Whether the contract is stated for the arithmetic this law's result carries.
///
/// **Derived from the verified semantic subject, never declared on the law.**
/// Every template that reaches here builds its output tensor with the subject's
/// own result type, so that type *is* the arithmetic the expected region will
/// emit; restating it as law data would be a second authority over one fact, and
/// the two could disagree. The comparison goes through
/// [`ArithmeticType::canonical_type_key`], which is the single durable spelling
/// of a dtype identity, so a contract stated for a width the result is not
/// produced in — an `f32` program under a `bf16` contract, or the reverse — is
/// refused rather than governed by a contract about another format's subnormals
/// and rounding.
///
/// A result type that is not nominal names no arithmetic and is refused, which
/// is strictly tighter than the `f32`-only test this replaced.
fn governs_result_arithmetic(subject: &IndexRefinementSubject) -> bool {
    let [result] = subject.results() else {
        // Not a single-result subject. Every template here refuses one by its
        // own arity rule, which names the defect precisely; answering `false`
        // would replace that diagnostic with a contract complaint about a
        // subject no law was ever going to accept.
        return true;
    };
    let expected = subject
        .numerical_contract()
        .arithmetic()
        .canonical_type_key();
    result
        .value_type()
        .nominal_key()
        .is_some_and(|key| key.to_string() == expected)
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
    /// The staged regions this law built do not compose into a chain.
    ///
    /// Reaching this is a defect in the law's own construction rather than in
    /// any candidate: the sequence is built from the law's regions alone.
    Sequence(IndexRegionSequenceError),
}

impl fmt::Display for IndexRealizationLawError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Emit(source) => write!(formatter, "law emission failed: {source}"),
            Self::Extent(source) => write!(formatter, "law extent failed: {source}"),
            Self::Unsupported { rule } => write!(formatter, "law does not support {rule}"),
            Self::Build(source) => write!(formatter, "law region failed verification: {source}"),
            Self::Sequence(source) => write!(formatter, "law sequence is not chained: {source}"),
        }
    }
}

impl Error for IndexRealizationLawError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Emit(source) => Some(source),
            Self::Extent(source) => Some(source),
            Self::Build(source) => Some(source),
            Self::Sequence(source) => Some(source),
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
            Self::Sequence(_) => "expected-sequence-chaining",
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
    let attributes = scalar_attributes(attribute, field.value().clone())?;
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
    let result = ((*result).value_type().clone(), (*result).shape().clone());
    let inputs = context
        .subject
        .inputs()
        .iter()
        .map(|input| (input.value_type().clone(), input.shape().clone()))
        .collect::<Vec<_>>();
    let operands = context.subject.operands().to_vec();
    emit_pointwise(context, scalar, &inputs, &operands, &result)
}

/// Emits `result[i] = scalar(operands[0][i], operands[1][i])` over explicit
/// boundaries.
///
/// The boundaries are parameters rather than reads of the subject because a
/// staged realization's later stage consumes a value that is not an occurrence
/// input at all: the fold an earlier stage published. Deriving them from the
/// subject would make that stage unstatable.
fn emit_pointwise(
    context: &mut LawContext<'_>,
    scalar: ScalarOpKey,
    inputs: &[(ResolvedValueType, Shape)],
    operands: &[usize],
    result: &(ResolvedValueType, Shape),
) -> Result<(), IndexRealizationLawError> {
    if operands.len() != 2 {
        return Err(unsupported("pointwise-operand-arity"));
    }
    let shape = result.1.clone();
    let dimensions = declare_parallel_domain(context, &shape)?;
    let coordinates = dimension_expressions(context, &dimensions)?;
    let mut tensors = Vec::with_capacity(inputs.len());
    for (value_type, input_shape) in inputs {
        tensors.push(context.tensor(TensorRole::Input, value_type.clone(), input_shape.clone())?);
    }
    let mut values = Vec::with_capacity(2);
    for position in operands.iter().copied() {
        let (_, boundary) = inputs
            .get(position)
            .ok_or_else(|| unsupported("pointwise-operand-binding"))?;
        let tensor = tensors[position];
        let value = if boundary == &shape {
            context.read(tensor, &dimensions, &coordinates)?
        } else if boundary.rank() == 0 {
            context.read(tensor, &[], &[])?
        } else {
            return Err(unsupported("pointwise-broadcast"));
        };
        values.push(value);
    }
    let value = single_result(
        &context.apply(scalar, ScalarAttributes::empty(), &values)?,
        "pointwise",
    )?;
    let output = context.tensor(TensorRole::Output, result.0.clone(), shape)?;
    let write = context.write(output, &dimensions, &coordinates)?;
    context.output(write, value)
}

fn realize_strict_affine_u4_dequantize(
    context: &mut LawContext<'_>,
    roles: [EncodedComponentRole; 3],
    scalar: ScalarOpKey,
) -> Result<(), IndexRealizationLawError> {
    let ([input], [result]) = (context.subject.inputs(), context.subject.results()) else {
        return Err(unsupported("strict-affine-arity"));
    };
    if context.subject.operands() != [0] {
        return Err(unsupported("strict-affine-operand-binding"));
    }
    if !context.subject.attributes().fields().is_empty() {
        return Err(unsupported("strict-affine-attributes"));
    }
    if input.value_type() != &StrictAffineU4::resolved_type() {
        return Err(unsupported("strict-affine-encoded-contract"));
    }
    if result.value_type() != &crate::semantic::F32::resolved_type() {
        return Err(unsupported("strict-affine-result-type"));
    }
    if input.shape() != result.shape() {
        return Err(unsupported("strict-affine-result-shape"));
    }
    let (_, contract) = input
        .value_type()
        .encoded_numeric_parts()
        .ok_or_else(|| unsupported("strict-affine-encoded-contract"))?;
    let components = contract.components();
    if components.len() != roles.len()
        || components
            .iter()
            .zip(roles)
            .any(|(component, role)| component.role() != role)
    {
        return Err(unsupported("strict-affine-component-roles"));
    }

    let shape = result.shape().clone();
    let dimensions = declare_parallel_domain(context, &shape)?;
    let coordinates = dimension_expressions(context, &dimensions)?;
    let mut tensors = Vec::with_capacity(components.len());
    for component in components {
        tensors.push(context.tensor(
            TensorRole::Input,
            component.resolved_type().clone(),
            component.shape_relation().component_shape(input.shape()),
        )?);
    }
    let codes = context.read(tensors[0], &dimensions, &coordinates)?;
    let scale = context.read(tensors[1], &[], &[])?;
    let zero_point = context.read(tensors[2], &[], &[])?;
    let value = apply_one(context, scalar, &[codes, scale, zero_point])?;
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
    let values = context.apply(
        constant_f32_scalar_op(),
        scalar_attributes(F32_CONSTANT_BITS_ATTRIBUTE, value)?,
        &[],
    )?;
    single_result(&values, "scalar-constant")
}

/// Wraps one exact payload in the attribute record its scalar constant declares.
///
/// The field is a parameter rather than the `f32` constant's own identifier
/// because attribute field IDs are record-local: the `f32` and `bf16` constants
/// number their payload field alike, and a writer hard-coding one of them would
/// build the other family's record correctly only by that coincidence.
fn scalar_attributes(
    field: AttributeFieldId,
    bits: CanonicalValue,
) -> Result<ScalarAttributes, IndexRealizationLawError> {
    let record = CanonicalValue::record([CanonicalField::new(field, bits)])
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
    emit_serial_sum(context, &plan)
}

/// Emits one strict lexicographic left fold from an already-derived plan.
fn emit_serial_sum(
    context: &mut LawContext<'_>,
    plan: &SumPlan,
) -> Result<(), IndexRealizationLawError> {
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
    let total = plan.fold(context, input, &kept, &kept_coordinates)?;
    let write = context.write(output, &kept, &kept_coordinates)?;
    context.output(write, total)
}
/// Builds the two-stage fold-then-pointwise realization.
///
/// The occurrence is `(operand 0, operand 1) -> result`: stage zero folds operand
/// zero over the attribute's axes into a value that is no occurrence boundary,
/// and stage one applies the scalar to operand one and that value.
///
/// Stage one declares its input boundaries in the order `(operand 1,
/// intermediate)`, which is the order its sources report and therefore the order
/// operand binding walks. The pointwise emitter's own broadcast rule is what
/// decides whether the fold is legible from stage one: a fold that removed every
/// axis is rank zero and read once per point, and a fold whose result is exactly
/// the result shape is read pointwise. Any other reduced shape refuses with
/// `pointwise-broadcast` rather than being stretched by a rule this law has not
/// stated.
fn realize_staged_sum_then_pointwise(
    subject: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
    axes_attribute: AttributeFieldId,
    scalar: ScalarOpKey,
) -> Result<VerifiedIndexRegionSequence, IndexRealizationLawError> {
    let ([folded, elementwise], [result]) = (subject.inputs(), subject.results()) else {
        return Err(unsupported("staged-sum-pointwise-arity"));
    };
    if subject.operands() != [0, 1] {
        return Err(unsupported("staged-sum-pointwise-operand-binding"));
    }
    let axes = reduction_axes(subject.attributes(), axes_attribute)?;
    let intermediate_shape = folded.shape().without_axes(&axes);
    let plan = SumPlan::for_boundaries(
        folded.value_type(),
        folded.shape(),
        &intermediate_shape,
        &axes,
    )?;

    let mut fold = IndexRegionBuilder::new(scalars.clone())?;
    {
        let mut context = LawContext {
            builder: &mut fold,
            subject,
        };
        emit_serial_sum(&mut context, &plan)?;
    }
    let fold = fold.build().map_err(IndexRealizationLawError::Build)?;

    let mut apply = IndexRegionBuilder::new(scalars.clone())?;
    {
        let mut context = LawContext {
            builder: &mut apply,
            subject,
        };
        emit_pointwise(
            &mut context,
            scalar,
            &[
                (
                    elementwise.value_type().clone(),
                    elementwise.shape().clone(),
                ),
                (folded.value_type().clone(), intermediate_shape),
            ],
            &[0, 1],
            &(result.value_type().clone(), result.shape().clone()),
        )?;
    }
    let apply = apply.build().map_err(IndexRealizationLawError::Build)?;

    VerifiedIndexRegionSequence::try_new(
        vec![fold, apply],
        vec![
            vec![StagedInputSource::Occurrence(0)],
            vec![
                StagedInputSource::Occurrence(1),
                StagedInputSource::Intermediate(0),
            ],
        ],
    )
    .map_err(IndexRealizationLawError::Sequence)
}

/// Builds the two-stage root-mean-square scale realization.
///
/// The occurrence is `(operand 0 = value, operand 1 = weight) -> result`, and the
/// realization is the pinned reference in order: `q_i = square(x_i)`, `a` the
/// strict left fold of `q`, `u = a / N`, `t = u + eps`, `r = Rsqrt(t)`, and
/// `y_i = w_i * (x_i * r)`.
///
/// **Where the split falls, and why there.** `r` is read once per *point* and
/// computed once per *folded row*, so stage zero carries the whole epilogue and
/// publishes `r` rather than publishing `a` and leaving the epilogue to stage
/// one. Publishing `a` would put the division, the bias, and the reciprocal
/// square root inside the pointwise pass, evaluating each `N` times per row: a
/// different scalar program, not a different schedule for this one.
///
/// **Every declared attribute is consumed by name.** The occurrence's field set
/// is required to be exactly the two this law names, so the tolerance of
/// [`reduction_axes`] for a record carrying more fields than it reads cannot
/// silently drop the `eps` payload here — which is the whole reason the plain
/// staged template is not this family's law.
fn realize_root_mean_square_scale(
    subject: &IndexRefinementSubject,
    scalars: &FrozenScalarRegistry,
    axes_attribute: AttributeFieldId,
    eps_attribute: AttributeFieldId,
) -> Result<VerifiedIndexRegionSequence, IndexRealizationLawError> {
    let ([value, weight], [result]) = (subject.inputs(), subject.results()) else {
        return Err(unsupported("rms-scale-arity"));
    };
    if subject.operands() != [0, 1] {
        return Err(unsupported("rms-scale-operand-binding"));
    }
    let expected = crate::semantic::F32::resolved_type();
    if value.value_type() != &expected
        || weight.value_type() != &expected
        || result.value_type() != &expected
    {
        return Err(unsupported("rms-scale-value-type"));
    }
    if value.shape() != result.shape() || weight.shape() != result.shape() {
        return Err(unsupported("rms-scale-shape"));
    }

    // Two distinct named identifiers, and a record of exactly two fields both of
    // which are named, means each is present exactly once. Aliased identifiers
    // would satisfy the count while leaving one of the two unread, so they are
    // refused first rather than left to make the inference false.
    if axes_attribute == eps_attribute {
        return Err(unsupported("rms-scale-attribute-aliasing"));
    }
    let fields = subject.attributes().fields();
    if fields.len() != 2
        || !fields
            .iter()
            .all(|field| field.id() == axes_attribute || field.id() == eps_attribute)
    {
        return Err(unsupported("rms-scale-attributes"));
    }
    let eps = subject
        .attributes()
        .get(eps_attribute)
        .ok_or_else(|| unsupported("rms-scale-eps-missing"))?
        .clone();
    if !matches!(eps.view(), CanonicalValueView::FloatBits(_)) {
        return Err(unsupported("rms-scale-eps-kind"));
    }
    let axes = reduction_axes(subject.attributes(), axes_attribute)?;

    let intermediate_shape = value.shape().without_axes(&axes);
    let plan = SumPlan::for_boundaries(
        value.value_type(),
        value.shape(),
        &intermediate_shape,
        &axes,
    )?
    .squaring_contributors(multiply_f32_scalar_op());
    let extent_bits = folded_extent_bits(plan.reduced_points)?;

    let mut fold = IndexRegionBuilder::new(scalars.clone())?;
    {
        let mut context = LawContext {
            builder: &mut fold,
            subject,
        };
        let kept = plan.declare_kept_domain(&mut context)?;
        let kept_coordinates = dimension_expressions(&mut context, &kept)?;
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
        let total = plan.fold(&mut context, input, &kept, &kept_coordinates)?;
        let extent = scalar_constant(&mut context, extent_bits)?;
        let mean = apply_one(&mut context, divide_f32_scalar_op(), &[total, extent])?;
        let bias = single_result(
            &context.apply(
                constant_f32_scalar_op(),
                scalar_attributes(F32_CONSTANT_BITS_ATTRIBUTE, eps)?,
                &[],
            )?,
            "rms-scale-eps-constant",
        )?;
        let biased = apply_one(&mut context, add_f32_scalar_op(), &[mean, bias])?;
        let root = apply_one(&mut context, rsqrt_f32_scalar_op(), &[biased])?;
        let write = context.write(output, &kept, &kept_coordinates)?;
        context.output(write, root)?;
    }
    let fold = fold.build().map_err(IndexRealizationLawError::Build)?;

    let mut scale = IndexRegionBuilder::new(scalars.clone())?;
    {
        let mut context = LawContext {
            builder: &mut scale,
            subject,
        };
        let shape = result.shape().clone();
        let dimensions = declare_parallel_domain(&mut context, &shape)?;
        let coordinates = dimension_expressions(&mut context, &dimensions)?;
        // The published value is one per folded row, so it is read at the kept
        // coordinates of this stage's own point domain. That is neither the
        // rank-zero nor the whole-shape case the binary pointwise emitter admits,
        // which is the third thing this family needs that the staged template
        // cannot state.
        let kept = dimensions
            .iter()
            .zip(&plan.reduced)
            .filter(|(_, reduced)| !**reduced)
            .map(|(dimension, _)| *dimension)
            .collect::<Vec<_>>();
        let kept_coordinates = coordinates
            .iter()
            .zip(&plan.reduced)
            .filter(|(_, reduced)| !**reduced)
            .map(|(coordinate, _)| *coordinate)
            .collect::<Vec<_>>();
        let value_tensor = context.tensor(TensorRole::Input, expected.clone(), shape.clone())?;
        let weight_tensor = context.tensor(TensorRole::Input, expected.clone(), shape.clone())?;
        let root_tensor =
            context.tensor(TensorRole::Input, expected.clone(), intermediate_shape)?;
        let element = context.read(value_tensor, &dimensions, &coordinates)?;
        let weight_element = context.read(weight_tensor, &dimensions, &coordinates)?;
        let root = context.read(root_tensor, &kept, &kept_coordinates)?;
        let scaled = apply_one(&mut context, multiply_f32_scalar_op(), &[element, root])?;
        let weighted = apply_one(
            &mut context,
            multiply_f32_scalar_op(),
            &[weight_element, scaled],
        )?;
        let output = context.tensor(TensorRole::Output, expected, shape)?;
        let write = context.write(output, &dimensions, &coordinates)?;
        context.output(write, weighted)?;
    }
    let scale = scale.build().map_err(IndexRealizationLawError::Build)?;

    VerifiedIndexRegionSequence::try_new(
        vec![fold, scale],
        vec![
            vec![StagedInputSource::Occurrence(0)],
            vec![
                StagedInputSource::Occurrence(0),
                StagedInputSource::Occurrence(1),
                StagedInputSource::Intermediate(0),
            ],
        ],
    )
    .map_err(IndexRealizationLawError::Sequence)
}

/// Returns the exact binary32 payload of the folded contributor count.
///
/// The reference divides by the extent itself — never by a reciprocal, and
/// therefore never by a divisor that is merely close to it. Above the binary32
/// significand's width the integers are not all representable, so a count whose
/// nearest binary32 is not the count would make the emitted division a different
/// function from the one the operation pins; it is refused rather than rounded.
///
/// The representability test is integer-only, so it does not depend on the
/// rounding it exists to detect: an integer is a binary32 value exactly when its
/// odd part fits in the twenty-four-bit significand.
fn folded_extent_bits(points: u64) -> Result<u32, IndexRealizationLawError> {
    if points == 0 {
        // An empty fold has no first contributor to seed at, so the reference's
        // own fold is undefined here before the division by zero is reached.
        return Err(unsupported("rms-scale-empty-fold"));
    }
    if points >> points.trailing_zeros() >= 1 << 24 {
        return Err(unsupported("rms-scale-extent-not-exact"));
    }
    #[allow(
        clippy::cast_precision_loss,
        reason = "the representability test above proves this conversion is exact"
    )]
    let extent = points as f32;
    Ok(extent.to_bits())
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
    /// Scalar each contributor is squared with before the fold combines it.
    ///
    /// `None` folds the operand's own elements, which is every plain strict
    /// serial sum. `Some(scalar)` applies `scalar(v, v)` to each contributor
    /// first, which is the only per-contributor transform the registered scalar
    /// vocabulary can spell as one application to one read value — a wider
    /// prologue would need a scalar-program language in law data, which this
    /// module deliberately does not have.
    contributor_square: Option<ScalarOpKey>,
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
        Self::for_boundaries(input.value_type(), input.shape(), result.shape(), &axes)
    }

    /// Derives the fold plan from explicit boundaries rather than the subject.
    ///
    /// A staged realization's fold publishes an intermediate, which is no
    /// occurrence result, so the output shape it must produce is a parameter.
    fn for_boundaries(
        value_type: &ResolvedValueType,
        input_shape: &Shape,
        output_shape: &Shape,
        axes: &[Axis],
    ) -> Result<Self, IndexRealizationLawError> {
        let input_shape = input_shape.clone();
        let mut reduced = vec![false; input_shape.rank()];
        for axis in axes {
            let index = axis_position(*axis)?;
            let slot = reduced
                .get_mut(index)
                .ok_or_else(|| unsupported("sum-axis-range"))?;
            if std::mem::replace(slot, true) {
                return Err(unsupported("sum-axis-duplicate"));
            }
        }
        if &input_shape.without_axes(axes) != output_shape {
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
    /// Separate from [`emit_serial_sum`] because a staged realization transforms
    /// the fold *inside the producing region* — the normalization divides it,
    /// biases it, and takes its reciprocal square root before anything is
    /// written — and a fold that could only write its own result would force
    /// that epilogue into the consuming stage, where it would run once per point
    /// instead of once per folded row. That is a different scalar program, not a
    /// different placement.
    fn fold(
        &self,
        context: &mut LawContext<'_>,
        input: TensorId,
        kept: &[DimensionId],
        kept_coordinates: &[IndexExprId],
    ) -> Result<ScalarValueId, IndexRealizationLawError> {
        if self.reduced_points == 0 {
            return self.fold_empty(context, input, kept, kept_coordinates);
        }
        let seed = self.read_contributor(context, input, kept, kept_coordinates, None)?;
        if self.reduced_points == 1 {
            apply_one(context, canonicalize_nan_f32_scalar_op(), &[seed])
        } else {
            self.fold_tail(context, input, kept, kept_coordinates, seed)
        }
    }

    /// Applies the per-contributor square, when this plan carries one.
    fn square(
        &self,
        context: &mut LawContext<'_>,
        contributor: ScalarValueId,
    ) -> Result<ScalarValueId, IndexRealizationLawError> {
        match &self.contributor_square {
            Some(scalar) => apply_one(context, scalar.clone(), &[contributor, contributor]),
            None => Ok(contributor),
        }
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
        let contributor = context.read(input, &domain, &coordinates)?;
        self.square(context, contributor)
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
        let contributor = self.square(context, contributor)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{
        FrozenScalarRegistry, NumericalContractIdentity, ScalarOperationKindRef,
        ScalarOperationRef, ScalarValueDefinitionView, VerifiedScalarOperationId,
        VerifiedScalarValueId,
    };
    use crate::semantic::{
        F32, InputKey, OperationAttributes, OutputKey, RMS_NORM_F32_QWEN3_EPS_BITS,
        SemanticProgramBuilder, StrictAffineU8, dequantize_strict_affine_op,
        rms_norm_f32_axis_attribute, rms_norm_f32_eps_attribute, rms_norm_f32_op,
    };

    /// Domain separating this file's identity pins from every governed digest.
    ///
    /// A pin needs a stable pre-image and nothing else; it is deliberately not
    /// one of the identity domains the workspace publishes, so a fixture digest
    /// can never be mistaken for evidence about a subject the compiler derives.
    const SEQUENCE_IDENTITY_PIN_DOMAIN: &[u8] = b"tiler.test.index-region-sequence-identity-pin\0";

    fn strict_contract() -> NumericalContractIdentity {
        NumericalContractIdentity::from(
            F32NumericalContractKey::new(
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
            .unwrap(),
        )
    }

    fn subject(value_type: ResolvedValueType) -> IndexRefinementSubject {
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let input = program
            .input_resolved(
                InputKey::new("encoded").unwrap(),
                Shape::from_dims([5]),
                value_type,
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
        let operation = program.operations().next().unwrap().id();
        IndexRefinementSubject::derive(&program, operation, strict_contract()).unwrap()
    }

    /// Every one-reader chain's identity is unchanged, byte for byte.
    ///
    /// **The pin is over exact bytes, taken before either widening landed.** The
    /// three digests below were captured on base commit `dd9def76` — that is,
    /// before `tiler.scalar::maximum-f32@1` was registered and before
    /// [`VerifiedIndexRegionSequence::try_new`](crate::index::VerifiedIndexRegionSequence::try_new)
    /// admitted a value with more than one reader — and they are asserted here
    /// unchanged. The length is pinned beside the digest so a chain that moved
    /// reports *how* rather than only that it did.
    ///
    /// **One pin covers both widenings, and that is the point of pinning here
    /// rather than in `sequence.rs`.** A realized law's sequence identity is
    /// built from its regions' canonical identities, and a region's identity
    /// carries the projection of the scalar definitions it *reaches*. So a new
    /// scalar key that no existing law emits must leave every byte alone, and a
    /// wider admitted chain shape that no existing law spells must leave every
    /// byte alone; both claims fail here if either is false.
    ///
    /// The rows are the two live instances plus the plain staged template: the
    /// normalization's own law at rank two and at rank one, and the
    /// fold-then-pointwise form no registered operation carries.
    #[test]
    fn the_landed_one_reader_chain_identities_are_unchanged_byte_for_byte() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        for (name, sequence, bytes, digest) in [
            (
                "rms-norm-3x4-axis1",
                IndexRealizationLaw::staged_root_mean_square_scale_f32()
                    .realize_sequence(
                        &rms_norm_subject(&[3, 4], 1, RMS_NORM_F32_QWEN3_EPS_BITS),
                        &scalars,
                    )
                    .unwrap(),
                4072_usize,
                "77a5cd34f014391433cc5e3e7da8e1e5483d5cd686e1242ef6fa160a949c5acf",
            ),
            (
                "rms-norm-rank1-4-axis0",
                IndexRealizationLaw::staged_root_mean_square_scale_f32()
                    .realize_sequence(
                        &rms_norm_subject(&[4], 0, RMS_NORM_F32_QWEN3_EPS_BITS),
                        &scalars,
                    )
                    .unwrap(),
                3649,
                "b318507aae49b1a97232b2a209b249ad28effa481b8273a73a73fd0a960c0efb",
            ),
            (
                "staged-template-rank1-4-axis0",
                IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32 {
                    axes_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                    scalar: multiply_f32_scalar_op(),
                }
                .realize_sequence(
                    &rms_norm_subject(&[4], 0, RMS_NORM_F32_QWEN3_EPS_BITS),
                    &scalars,
                )
                .unwrap(),
                2023,
                "3ddd3268089e163410195e628e70addf5a6213493df25b4ffe099bb3b0324e34",
            ),
        ] {
            let identity = sequence.identity().as_bytes();
            assert_eq!(identity.len(), bytes, "{name} changed length");
            assert_eq!(
                tiler_digest::DigestAlgorithm::GOVERNED
                    .digest(SEQUENCE_IDENTITY_PIN_DOMAIN, identity)
                    .label(),
                digest,
                "{name} changed bytes"
            );
        }
    }

    #[test]
    fn strict_affine_law_refuses_wrong_contract_role_and_scalar_independently() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let u8_subject = subject(StrictAffineU8::resolved_type());
        let error = IndexRealizationLaw::strict_affine_u4_dequantize()
            .realize(&u8_subject, &scalars)
            .unwrap_err();
        assert_eq!(error.rule(), "strict-affine-encoded-contract");

        let subject = subject(StrictAffineU4::resolved_type());
        let wrong_role = IndexRealizationLaw::StrictAffineU4Dequantize {
            codes_role: STRICT_AFFINE_SCALE_ROLE,
            scale_role: STRICT_AFFINE_CODES_ROLE,
            zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
            scalar: strict_affine_u4_dequantize_scalar_op(),
        };
        assert_eq!(
            wrong_role.realize(&subject, &scalars).unwrap_err().rule(),
            "strict-affine-component-roles"
        );

        let wrong_scalar = IndexRealizationLaw::StrictAffineU4Dequantize {
            codes_role: STRICT_AFFINE_CODES_ROLE,
            scale_role: STRICT_AFFINE_SCALE_ROLE,
            zero_point_role: STRICT_AFFINE_ZERO_POINT_ROLE,
            scalar: add_f32_scalar_op(),
        };
        assert_eq!(
            wrong_scalar.realize(&subject, &scalars).unwrap_err().rule(),
            "canonical-emission"
        );
    }

    #[test]
    fn strict_affine_law_tag_is_append_only_and_distinct() {
        let mut strict = Vec::new();
        IndexRealizationLaw::strict_affine_u4_dequantize().encode(&mut strict);
        assert_eq!(strict.first(), Some(&8));
        for old in [
            IndexRealizationLaw::constant_f32(),
            IndexRealizationLaw::multiply_f32(),
            IndexRealizationLaw::PreciseSiluF32,
            IndexRealizationLaw::strict_serial_sum_f32(),
            IndexRealizationLaw::reindex_f32(),
            IndexRealizationLaw::broadcast_f32(),
            IndexRealizationLaw::strict_tensor_contraction_f32(),
        ] {
            let mut encoded = Vec::new();
            old.encode(&mut encoded);
            assert_ne!(encoded, strict);
            assert!((1..=7).contains(encoded.first().unwrap()));
        }
    }

    #[test]
    fn the_staged_law_tag_is_append_only_and_distinct() {
        let mut staged = Vec::new();
        IndexRealizationLaw::staged_strict_serial_sum_then_multiply_f32().encode(&mut staged);
        assert_eq!(staged.first(), Some(&9));
        // The staged form names an attribute like tag 4 and a scalar like tag 2.
        // Neither prefix can be mistaken for it, because the discriminating byte
        // comes first and no earlier tag writes both payloads.
        for old in [
            IndexRealizationLaw::constant_f32(),
            IndexRealizationLaw::constant_bf16(),
            IndexRealizationLaw::multiply_f32(),
            IndexRealizationLaw::add_f32(),
            IndexRealizationLaw::multiply_bf16(),
            IndexRealizationLaw::add_bf16(),
            IndexRealizationLaw::PreciseSiluF32,
            IndexRealizationLaw::strict_serial_sum_f32(),
            IndexRealizationLaw::reindex_f32(),
            IndexRealizationLaw::broadcast_f32(),
            IndexRealizationLaw::strict_tensor_contraction_f32(),
            IndexRealizationLaw::strict_affine_u4_dequantize(),
        ] {
            let mut encoded = Vec::new();
            old.encode(&mut encoded);
            assert_ne!(encoded, staged);
            assert!((1..=8).contains(encoded.first().unwrap()));
        }
        // Its own payload separates two staged rows that differ only in the
        // scalar their pass applies, or only in the attribute their fold reads.
        let mut other_scalar = Vec::new();
        IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32 {
            axes_attribute: REDUCTION_AXES_ATTRIBUTE,
            scalar: add_f32_scalar_op(),
        }
        .encode(&mut other_scalar);
        assert_ne!(other_scalar, staged);
        let mut other_attribute = Vec::new();
        IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32 {
            axes_attribute: AttributeFieldId::new(REDUCTION_AXES_ATTRIBUTE.get() + 1),
            scalar: multiply_f32_scalar_op(),
        }
        .encode(&mut other_attribute);
        assert_ne!(other_attribute, staged);
    }

    /// A staged law cannot answer the single-region realization API.
    ///
    /// Answering one of its stages would be a truncated realization wearing the
    /// shape of a complete one, and the single-region `verify` path — the one
    /// the compiler drives today — would then compare a candidate against a
    /// fragment.
    #[test]
    fn a_staged_law_refuses_the_single_region_realization() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let subject = subject(StrictAffineU4::resolved_type());
        assert_eq!(
            IndexRealizationLaw::staged_strict_serial_sum_then_multiply_f32()
                .realize(&subject, &scalars)
                .unwrap_err()
                .rule(),
            "staged-law-requires-region-sequence"
        );
    }

    /// Derives one `tiler::rms-norm-f32@1` occurrence's refinement subject.
    fn rms_norm_subject(dims: &[u64], axis: u32, eps_bits: u32) -> IndexRefinementSubject {
        let shape = Shape::try_new(dims.iter().copied().map(Extent::new).collect::<Vec<_>>())
            .expect("the test shape is canonical");
        let mut program = SemanticProgramBuilder::try_standard().unwrap();
        let value = program
            .input_resolved(
                InputKey::new("value").unwrap(),
                shape.clone(),
                F32::resolved_type(),
            )
            .unwrap();
        let weight = program
            .input_resolved(
                InputKey::new("weight").unwrap(),
                shape,
                F32::resolved_type(),
            )
            .unwrap();
        let attributes = OperationAttributes::new([
            CanonicalField::new(
                RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                rms_norm_f32_axis_attribute(Axis::new(axis)),
            ),
            CanonicalField::new(
                RMS_NORM_EPS_BITS_ATTRIBUTE,
                rms_norm_f32_eps_attribute(eps_bits),
            ),
        ])
        .unwrap();
        let result = program
            .apply(rms_norm_f32_op(), attributes, &[value, weight])
            .unwrap()[0];
        program
            .output_resolved(OutputKey::new("normalized").unwrap(), result)
            .unwrap();
        let program = program.build().unwrap();
        let operation = program.operations().next().unwrap().id();
        IndexRefinementSubject::derive(&program, operation, strict_contract()).unwrap()
    }

    /// Names one stage's scalar steps in emission order.
    fn stage_steps(stage: &VerifiedIndexRegion) -> Vec<String> {
        stage
            .scalar_operations()
            .map(|operation| match operation.kind() {
                ScalarOperationKindRef::Apply { key, .. } => key.name().to_owned(),
                ScalarOperationKindRef::Reduce(_) => "reduce".to_owned(),
            })
            .collect()
    }

    /// Returns the operation defining one value, or `None` when it is a read.
    fn defined_by(
        region: &VerifiedIndexRegion,
        value: VerifiedScalarValueId,
    ) -> Option<VerifiedScalarOperationId> {
        match region.scalar_value(value).unwrap().definition() {
            ScalarValueDefinitionView::OperationResult { operation, .. } => Some(operation),
            ScalarValueDefinitionView::AccessRead(_) => None,
        }
    }

    fn operand_definitions(
        region: &VerifiedIndexRegion,
        operation: ScalarOperationRef<'_>,
    ) -> Vec<Option<VerifiedScalarOperationId>> {
        operation
            .operands()
            .map(|value| defined_by(region, value))
            .collect()
    }

    /// Returns the boundary one read value comes from, or `None` for a computed value.
    fn read_boundary(
        region: &VerifiedIndexRegion,
        value: VerifiedScalarValueId,
    ) -> Option<super::super::VerifiedTensorId> {
        match region.scalar_value(value).unwrap().definition() {
            ScalarValueDefinitionView::AccessRead(access) => {
                Some(region.access(access).unwrap().tensor())
            }
            ScalarValueDefinitionView::OperationResult { .. } => None,
        }
    }

    fn by_id(
        region: &VerifiedIndexRegion,
        id: VerifiedScalarOperationId,
    ) -> ScalarOperationRef<'_> {
        region.scalar_operation(id).unwrap()
    }

    /// Returns the operation defining one value, refusing a read.
    fn operation_defining(
        region: &VerifiedIndexRegion,
        value: VerifiedScalarValueId,
    ) -> ScalarOperationRef<'_> {
        by_id(
            region,
            defined_by(region, value).expect("this value is computed rather than read"),
        )
    }

    fn applied_key(operation: ScalarOperationRef<'_>) -> String {
        match operation.kind() {
            ScalarOperationKindRef::Apply { key, .. } => key.name().to_owned(),
            ScalarOperationKindRef::Reduce(_) => "reduce".to_owned(),
        }
    }

    fn applied_attributes(operation: ScalarOperationRef<'_>) -> &CanonicalValue {
        match operation.kind() {
            ScalarOperationKindRef::Apply { attributes, .. } => attributes.value(),
            ScalarOperationKindRef::Reduce(_) => panic!("a reduction carries no attribute record"),
        }
    }

    fn f32_bits_record(bits: u32) -> CanonicalValue {
        CanonicalValue::record([CanonicalField::new(
            F32_CONSTANT_BITS_ATTRIBUTE,
            CanonicalValue::float_bits(
                TypeKey::new("tiler", "f32", 1).unwrap(),
                bits.to_be_bytes(),
            )
            .unwrap(),
        )])
        .unwrap()
    }

    /// The realization is the pinned reference, in the pinned order.
    ///
    /// `rms_norm_f32_reference_semantics` fixes `q_i = x_i * x_i`, `a = fold(q)`,
    /// `u = a / N`, `t = u + eps`, `r = Rsqrt(t)`, `y_i = w_i * (x_i * r)`. This
    /// walks the two realized stages and pins each of those six steps: the scalar
    /// applied, the values it consumes, and — for the two constants, which are the
    /// only places an exact payload enters — the payload itself.
    #[test]
    fn the_normalization_law_realizes_the_pinned_reference_step_for_step() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let subject = rms_norm_subject(&[3, 4], 1, RMS_NORM_F32_QWEN3_EPS_BITS);
        let realization = IndexRealizationLaw::staged_root_mean_square_scale_f32()
            .realize_sequence(&subject, &scalars)
            .expect("the normalization's law realizes its own occurrence");

        assert_eq!(realization.stage_count(), 2);
        let [intermediate] = realization.intermediates() else {
            panic!("the normalization hands exactly one value on")
        };
        // One published value per folded row, not one per point and not one per
        // program: `r` is rank one over the kept axis of a rank-two operand.
        assert_eq!(intermediate.shape(), &Shape::from_dims([3]));
        assert_eq!(
            realization.stage_sources(1),
            Some(
                [
                    StagedInputSource::Occurrence(0),
                    StagedInputSource::Occurrence(1),
                    StagedInputSource::Intermediate(0),
                ]
                .as_slice()
            ),
            "the scale stage reads the value again, the weight, and the fold"
        );

        let fold = realization.stage(0).expect("the folding stage");
        // The population first, so a walk that missed a step could not pass: a
        // verified region orders its operations canonically rather than in
        // emission order, which is why the walk below navigates by definition
        // rather than by position.
        assert_eq!(
            stage_steps(fold),
            [
                "constant-f32",
                "constant-f32",
                "multiply-f32",
                "multiply-f32",
                "reduce",
                "divide-f32",
                "add-f32",
                "rsqrt-f32",
            ]
        );

        // r = Rsqrt(t) is exactly what the stage publishes.
        let root = operation_defining(fold, fold.outputs().next().unwrap().value());
        assert_eq!(applied_key(root), "rsqrt-f32");
        // t = u + eps, with the exact declared payload and no other.
        let [biased] = operand_definitions(fold, root)[..] else {
            panic!("the reciprocal square root takes one argument")
        };
        let biased = by_id(fold, biased.expect("t is computed"));
        assert_eq!(applied_key(biased), "add-f32");
        let [mean, bias] = operand_definitions(fold, biased)[..] else {
            panic!("the bias is an addition")
        };
        let bias = by_id(fold, bias.expect("eps enters as a constant"));
        assert_eq!(
            applied_attributes(bias),
            &f32_bits_record(RMS_NORM_F32_QWEN3_EPS_BITS)
        );
        // u = a / N, a division by the extent itself and never by a reciprocal.
        let mean = by_id(fold, mean.expect("u is computed"));
        assert_eq!(applied_key(mean), "divide-f32");
        let [total, extent] = operand_definitions(fold, mean)[..] else {
            panic!("the mean is a division")
        };
        let extent = by_id(fold, extent.expect("the extent enters as a constant"));
        assert_eq!(
            applied_attributes(extent),
            &f32_bits_record(4.0_f32.to_bits())
        );
        // a = the strict left fold of q seeded at the first contributor: the
        // reduction's state starts at a squared seed and combines squared tail
        // contributors, so nothing unsquared reaches the accumulator.
        let total = by_id(fold, total.expect("a is computed"));
        let ScalarOperationKindRef::Reduce(reduction) = total.kind() else {
            panic!("the fold is a reduction")
        };
        let seed = reduction
            .init()
            .map(|value| operation_defining(fold, value))
            .collect::<Vec<_>>();
        let contributors = reduction
            .contributors()
            .map(|value| operation_defining(fold, value))
            .collect::<Vec<_>>();
        assert_eq!(seed.len(), 1);
        assert_eq!(contributors.len(), 1);
        // q_i = x_i * x_i: one read squared against itself, at both the seed and
        // the tail. A square of two different reads would be a product.
        for square in [seed[0], contributors[0]] {
            assert_eq!(applied_key(square), "multiply-f32");
            let operands = square.operands().collect::<Vec<_>>();
            assert_eq!(operands.len(), 2);
            assert_eq!(operands[0], operands[1]);
            assert_eq!(defined_by(fold, operands[0]), None);
        }
        assert_ne!(
            seed[0].id(),
            contributors[0].id(),
            "the seed and the tail read different contributors"
        );

        let scale = realization.final_stage();
        assert_eq!(stage_steps(scale), ["multiply-f32", "multiply-f32"]);
        let scale_steps = scale.scalar_operations().collect::<Vec<_>>();
        let boundaries = scale
            .tensors()
            .filter(|tensor| tensor.role() == TensorRole::Input)
            .map(super::super::model::TensorRef::id)
            .collect::<Vec<_>>();
        // y_i = w_i * (x_i * r): the inner product is the value and the published
        // root — in that order — and the outer applies the weight to it. The
        // boundary each read comes from is what separates the value from the
        // weight, which agree on element type and shape.
        let inner = scale_steps[0].operands().collect::<Vec<_>>();
        assert_eq!(
            inner
                .iter()
                .map(|value| read_boundary(scale, *value))
                .collect::<Vec<_>>(),
            vec![Some(boundaries[0]), Some(boundaries[2])]
        );
        let outer = scale_steps[1].operands().collect::<Vec<_>>();
        assert_eq!(read_boundary(scale, outer[0]), Some(boundaries[1]));
        assert_eq!(defined_by(scale, outer[1]), Some(scale_steps[0].id()));
        assert_eq!(
            defined_by(scale, scale.outputs().next().unwrap().value()),
            Some(scale_steps[1].id())
        );
    }

    /// The `eps` payload is consumed, and the template that would drop it is not.
    ///
    /// **The hazard is watched here rather than described.** `reduction_axes`
    /// reads its attribute by field identifier and tolerates a record carrying
    /// more, so the plain staged template realizes this very occurrence while
    /// never looking at `eps` — the first assertion below is that silent success.
    /// `eps` is part of the operation's identity, so a law that did that would be
    /// realizing a different operation. The normalization's own law therefore
    /// pins the exact declared field set, and the remaining assertions watch that
    /// refusal fire for every way the pair can fail to name the record.
    #[test]
    fn the_normalization_law_consumes_eps_where_the_staged_template_drops_it() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        // Rank one so the plain template's own broadcast rule does not refuse
        // first: the fold removes the only axis, so what it publishes is rank
        // zero and legible to a binary pointwise pass.
        let subject = rms_norm_subject(&[4], 0, RMS_NORM_F32_QWEN3_EPS_BITS);
        assert!(
            IndexRealizationLaw::StagedStrictSerialSumThenPointwiseF32 {
                axes_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                scalar: multiply_f32_scalar_op(),
            }
            .realize_sequence(&subject, &scalars)
            .is_ok(),
            "the hazard this law exists to close: the staged template realizes the \
             normalization's occurrence without ever reading its eps attribute"
        );

        // Every way of failing to name the declared record refuses, by name.
        for (law, rule) in [
            (
                IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
                    axes_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                    eps_attribute: AttributeFieldId::new(RMS_NORM_EPS_BITS_ATTRIBUTE.get() + 1),
                },
                "rms-scale-attributes",
            ),
            (
                IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
                    axes_attribute: AttributeFieldId::new(
                        RMS_NORM_REDUCED_AXES_ATTRIBUTE.get() + 8,
                    ),
                    eps_attribute: RMS_NORM_EPS_BITS_ATTRIBUTE,
                },
                "rms-scale-attributes",
            ),
            (
                // Aliased identifiers would satisfy a field count while leaving
                // one of the two declared payloads unread.
                IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
                    axes_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                    eps_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                },
                "rms-scale-attribute-aliasing",
            ),
            (
                // The transposed pair names both declared fields, so the field-set
                // check passes and the payloads are read as each other's.
                IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
                    axes_attribute: RMS_NORM_EPS_BITS_ATTRIBUTE,
                    eps_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
                },
                "rms-scale-eps-kind",
            ),
        ] {
            assert_eq!(
                law.realize_sequence(&subject, &scalars)
                    .expect_err("a law that does not name this record's fields realizes nothing")
                    .rule(),
                rule
            );
        }
    }

    /// The division is by the extent, so an extent that is not one is refused.
    #[test]
    fn a_folded_extent_no_binary32_value_equals_is_refused_rather_than_rounded() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        // 2^24 + 1 is odd and above the significand's width, so its nearest
        // binary32 is 2^24 — a divisor that is not the extent.
        let subject = rms_norm_subject(&[16_777_217], 0, RMS_NORM_F32_QWEN3_EPS_BITS);
        assert_eq!(
            IndexRealizationLaw::staged_root_mean_square_scale_f32()
                .realize_sequence(&subject, &scalars)
                .unwrap_err()
                .rule(),
            "rms-scale-extent-not-exact"
        );
        // The neighbouring even extent is exactly representable and realizes.
        let subject = rms_norm_subject(&[16_777_216], 0, RMS_NORM_F32_QWEN3_EPS_BITS);
        assert!(
            IndexRealizationLaw::staged_root_mean_square_scale_f32()
                .realize_sequence(&subject, &scalars)
                .is_ok(),
            "the bound is representability, not size"
        );
    }

    #[test]
    fn the_root_mean_square_law_tag_is_append_only_and_distinct() {
        let mut normalization = Vec::new();
        IndexRealizationLaw::staged_root_mean_square_scale_f32().encode(&mut normalization);
        assert_eq!(normalization.first(), Some(&10));
        for old in [
            IndexRealizationLaw::constant_f32(),
            IndexRealizationLaw::constant_bf16(),
            IndexRealizationLaw::multiply_f32(),
            IndexRealizationLaw::add_f32(),
            IndexRealizationLaw::multiply_bf16(),
            IndexRealizationLaw::add_bf16(),
            IndexRealizationLaw::PreciseSiluF32,
            IndexRealizationLaw::strict_serial_sum_f32(),
            IndexRealizationLaw::reindex_f32(),
            IndexRealizationLaw::broadcast_f32(),
            IndexRealizationLaw::strict_tensor_contraction_f32(),
            IndexRealizationLaw::strict_affine_u4_dequantize(),
            IndexRealizationLaw::staged_strict_serial_sum_then_multiply_f32(),
        ] {
            let mut encoded = Vec::new();
            old.encode(&mut encoded);
            assert_ne!(encoded, normalization);
            assert!((1..=9).contains(encoded.first().unwrap()));
        }
        // Within the tag, each field owns four fixed bytes, so a row differing in
        // either one encodes distinctly — and the pair is ordered, so the
        // transposition encodes as a third distinct row rather than the same one.
        let mut moved_axes = Vec::new();
        IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
            axes_attribute: AttributeFieldId::new(RMS_NORM_REDUCED_AXES_ATTRIBUTE.get() + 1),
            eps_attribute: RMS_NORM_EPS_BITS_ATTRIBUTE,
        }
        .encode(&mut moved_axes);
        let mut moved_eps = Vec::new();
        IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
            axes_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
            eps_attribute: AttributeFieldId::new(RMS_NORM_EPS_BITS_ATTRIBUTE.get() + 1),
        }
        .encode(&mut moved_eps);
        let mut transposed = Vec::new();
        IndexRealizationLaw::StagedRootMeanSquareScaleF32 {
            axes_attribute: RMS_NORM_EPS_BITS_ATTRIBUTE,
            eps_attribute: RMS_NORM_REDUCED_AXES_ATTRIBUTE,
        }
        .encode(&mut transposed);
        let rows = [&normalization, &moved_axes, &moved_eps, &transposed];
        for (position, row) in rows.iter().enumerate() {
            for other in &rows[position + 1..] {
                assert_ne!(row, other);
            }
        }
    }

    /// A staged law cannot answer the single-region realization API.
    #[test]
    fn the_root_mean_square_law_refuses_the_single_region_realization() {
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let subject = rms_norm_subject(&[3, 4], 1, RMS_NORM_F32_QWEN3_EPS_BITS);
        assert_eq!(
            IndexRealizationLaw::staged_root_mean_square_scale_f32()
                .realize(&subject, &scalars)
                .unwrap_err()
                .rule(),
            "staged-law-requires-region-sequence"
        );
    }

    /// A resolution answers the realization's shape from its own authorities.
    ///
    /// The two accessors exist so a consumer that needs a law's stage structure
    /// — region formation is the caller — reads it from the resolution rather
    /// than deriving a second account of the law. Three claims: the staged
    /// family answers its two-stage sequence, a single-region family answers
    /// one stage behind a false predicate, and a subject the law refuses maps
    /// to the same typed refusal refinement reports, carrying the law's own
    /// rule.
    #[test]
    fn a_resolved_realization_exposes_its_laws_sequence_shape() {
        use crate::index::{FrozenIndexRealizationLawRegistry, IndexRefinementVerificationError};
        let scalars = FrozenScalarRegistry::standard().unwrap();
        let semantic = crate::semantic::FrozenSemanticRegistry::standard().unwrap();
        let laws =
            FrozenIndexRealizationLawRegistry::from_semantic(semantic, scalars.clone()).unwrap();

        let staged = laws
            .resolve(&rms_norm_subject(&[3, 4], 1, RMS_NORM_F32_QWEN3_EPS_BITS))
            .unwrap();
        assert!(staged.realizes_region_sequence());
        let sequence = staged.realize_sequence().unwrap();
        assert_eq!(sequence.stage_count(), 2);
        assert_eq!(sequence.intermediates().len(), 1);

        let single = laws
            .resolve(&subject(StrictAffineU4::resolved_type()))
            .unwrap();
        assert!(!single.realizes_region_sequence());
        let sequence = single.realize_sequence().unwrap();
        assert_eq!(sequence.stage_count(), 1);
        assert!(sequence.intermediates().is_empty());

        // The refusal is the law's, in refinement's vocabulary: an extent no
        // binary32 value equals refuses the realization, not the resolution.
        let refused = laws
            .resolve(&rms_norm_subject(
                &[16_777_217],
                0,
                RMS_NORM_F32_QWEN3_EPS_BITS,
            ))
            .unwrap();
        assert!(matches!(
            refused.realize_sequence().unwrap_err(),
            IndexRefinementVerificationError::SemanticRealizationLawRefused { rule, .. }
                if rule == "rms-scale-extent-not-exact"
        ));
    }

    #[test]
    fn an_existing_law_payload_is_unchanged_by_the_appended_tag() {
        let mut encoded = Vec::new();
        IndexRealizationLaw::multiply_f32().encode(&mut encoded);
        let expected = [
            vec![2],
            12_u64.to_be_bytes().to_vec(),
            b"tiler.scalar".to_vec(),
            12_u64.to_be_bytes().to_vec(),
            b"multiply-f32".to_vec(),
            1_u32.to_be_bytes().to_vec(),
        ]
        .concat();
        assert_eq!(encoded, expected);
    }
}

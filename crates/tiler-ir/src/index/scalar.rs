use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::identity::{push_len, push_slice};
use crate::semantic::{
    AttributeFieldId, CanonicalValue, CanonicalValueKind, CanonicalValueView,
    FrozenSemanticRegistry, MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES, NormativeDefinitionRef,
    ProviderDiagnosticCode, ProviderDiagnosticError, ProviderIdentity, RegistryError,
    ResolvedValueType, SemanticAdmissionProvenanceIdentity, SemanticDefinitionProjectionIdentity,
    SemanticRegistrySnapshotIdentity, TypeIdentityError, TypeKey,
};

use super::{
    CanonicalIndexRegionIdentity, ScalarOperationKindRef, VerifiedIndexHandleError,
    VerifiedIndexRegion,
};

const MAX_SCALAR_ATTRIBUTES: usize = 256;
const MAX_SCALAR_ARITY: usize = 4_096;
const MAX_SCALAR_DEFINITIONS: usize = 65_536;
const MAX_SCALAR_REGISTRY_CANONICAL_BYTES: usize = 16 * 1024 * 1024;
const MAX_SCALAR_DEFINITION_PROJECTION_BYTES: usize = 8 * 1024 * 1024;

/// Returns the governed per-point `f32` constant scalar operation key.
#[must_use]
pub fn constant_f32_scalar_op() -> ScalarOpKey {
    governed_scalar_op("constant-f32")
}

/// Returns the governed per-point `f32` multiplication scalar operation key.
#[must_use]
pub fn multiply_f32_scalar_op() -> ScalarOpKey {
    governed_scalar_op("multiply-f32")
}

/// Returns the governed per-point `f32` addition scalar operation key.
#[must_use]
pub fn add_f32_scalar_op() -> ScalarOpKey {
    governed_scalar_op("add-f32")
}

/// Returns the governed per-point `f32` division scalar operation key.
///
/// A division, and deliberately not a reciprocal followed by a multiplication:
/// the two round a different number of times and are different binary32
/// functions. There is no reciprocal scalar key beside it, so the substitution
/// has nothing to name rather than being forbidden by a rule someone must
/// remember.
#[must_use]
pub fn divide_f32_scalar_op() -> ScalarOpKey {
    governed_scalar_op("divide-f32")
}

/// Returns the governed per-point `f32` natural-exponential scalar operation key.
///
/// The *precise* exponential. It is the first scalar key whose result is not a
/// rational function of its operand, so what it may deliver is a resolved ADR
/// 0042 accuracy contract rather than IEEE-754 alone — and that contract lives on
/// the semantic operation this scalar realizes, `tiler::silu-f32@1`, not here. A
/// second copy of the tolerance at this layer would be a second authority over
/// one obligation.
#[must_use]
pub fn exp_f32_scalar_op() -> ScalarOpKey {
    governed_scalar_op("exp-f32")
}

/// Returns the governed per-point `f32` reciprocal-square-root scalar operation key.
///
/// **Accepted boundary** (Tom, 2026-08-06, at the live session's decision
/// round; relayed and executed by the coordinator rather than witnessed at this
/// site, and the provenance packet is the `## Accepted 2026-08-06` section of
/// the closed acceptance node
/// [`accept-the-governed-reciprocal-square-root-scalar-key`], whose own
/// "Closes when" routed this label flip here). Acceptance is not stabilization:
/// this is accepted pre-alpha vocabulary, not a published API with
/// compatibility obligations.
///
/// What was accepted is the key, its name, its arity, and its fact record.
/// Admitting a scalar operation is a semantic surface rather than an
/// implementation detail, because the key becomes part of every reached-
/// definition projection a region carrying it derives an identity from.
///
/// The *precise* reciprocal square root, `1/sqrt(t)`. Like
/// [`exp_f32_scalar_op`] its result is not a rational function of its operand,
/// so what it may deliver is a resolved ADR 0042 accuracy contract rather than
/// IEEE-754 alone — and that contract lives on the semantic operation this
/// scalar realizes, `tiler::rms-norm-f32@1`, not here. The two elementary keys
/// share a fact record for exactly that reason and differ in the contract their
/// operations state: the exponential's is `BoundedPiecewise` at twelve ULP,
/// this one's is `Faithful`.
///
/// **One operation, deliberately not a square root followed by a division and
/// not a reciprocal of a square root.** Each of those rounds twice where this
/// rounds once, so they are different binary32 functions;
/// `tiler::rms-norm-f32@1` pins `Rsqrt` in its reference semantics and
/// withholds the reciprocal-transform permission rather than leaving the
/// substitution open. There is no square-root scalar key beside this one, so
/// the substitution has nothing to name rather than being forbidden by a rule
/// someone must remember — the argument [`divide_f32_scalar_op`] states for its
/// own missing sibling.
///
/// [`accept-the-governed-reciprocal-square-root-scalar-key`]: ../../../../tickets/accept-the-governed-reciprocal-square-root-scalar-key.md
#[must_use]
pub fn rsqrt_f32_scalar_op() -> ScalarOpKey {
    governed_scalar_op("rsqrt-f32")
}

/// Returns the governed per-point binary32 `maximum` scalar operation key.
///
/// **Labelled draft.** The key, its name, its arity, and its fact record are a
/// concrete draft pending Tom's review; see
/// [`accept-the-governed-maximum-scalar-key`]. Admitting a scalar operation is a
/// semantic surface rather than an implementation detail, because the key becomes
/// part of every reached-definition projection a region carrying it derives an
/// identity from — the reason [`rsqrt_f32_scalar_op`] carries an acceptance node
/// of its own.
///
/// The **NaN-propagating** IEEE 754-2019 extrema family, ordering `-0.0 < +0.0`.
/// It is what `tiler::softmax-f32@1`'s row maximum pins, and it is the per-point
/// counterpart of [`crate::kernel::BinaryOp::F32Maximum`].
///
/// # The name, and why the bare spelling is admissible
///
/// [ADR 0023](../../../../docs/decisions/0023-floating-point-extrema-semantics.md)
/// admits *two* families and requires an operation to name the one it means, so a
/// bare `maximum` is admissible only if the number-preferring sibling can never
/// later be registered under a name that reads as its complement. It cannot,
/// because the two names are already complements in the standard's own
/// vocabulary: IEEE 754-2019 spells the propagating family `maximum` and the
/// number-preferring one `maximumNumber`, and ADR 0023 carries that pair over as
/// `Maximum` and `MaximumNumber`. Under this module's naming rule — the spec's
/// own name, kebab-cased, with the operand width appended, as
/// [`rsqrt_f32_scalar_op`] and [`divide_f32_scalar_op`] already are — the sibling
/// spells `maximum-number-f32` and this key reads as exactly its complement. A
/// disambiguating name such as `maximum-propagating-f32` would *diverge* from the
/// name ADR 0023 anchors on, which is the opposite of what naming the family for
/// safety would achieve.
///
/// What a bare name does not do on its own is separate this family from a host or
/// backend spelling that shares it: Rust's `f32::max` and Metal's `fmax` are both
/// the *other* family. That separation is carried where this module already
/// carries it for `divide-f32`'s missing reciprocal sibling — in the registered
/// normative definition, which names the family, the NaN rule, and the zero
/// ordering explicitly and is part of this definition's encoded identity.
///
/// # The signed-zero ordering is Tiler's own fact
///
/// `-0.0 < +0.0`, so `maximum(-0.0, +0.0)` is `+0.0` in either operand order.
/// ADR 0023 requires the ordering of *both* Tiler families, and it is stated here
/// as this operation's own contract rather than as a reproduction of any
/// reference. The reference model does not implement it and is cited only as
/// contrast: in the retained probe's pinned environment `torch.max` over
/// `[+0.0, -0.0]` is `-0.0` and over `[-0.0, +0.0]` is `+0.0` while `torch.amax`
/// answers the other way on both, so each spelling returns a fixed *position*
/// rather than a fixed value — the four `torch_max_of_signed_zeros_*` and
/// `torch_amax_of_signed_zeros_*` rows of
/// `spikes/numerics/transformer_reference_semantics/results/2026-08-01-cpu-f32-torch2.6.0-transformers4.51.0/record.tsv`.
/// Nothing in this key rests on them.
///
/// # Why it shares the exact-bit-pattern fact record, and states no third rule
///
/// The derivation is on `exact_bit_pattern_f32_scalar_facts` in this module,
/// beside the record itself. The short form: this operation rounds nothing and
/// installs the governed canonical arithmetic NaN for a NaN result, which is what
/// the NaN-canonicalizing conversion beside it also does, so the two share one
/// record for the reason the exponential and the reciprocal square root share
/// theirs.
///
/// [`accept-the-governed-maximum-scalar-key`]: ../../../../tickets/accept-the-governed-maximum-scalar-key.md
#[must_use]
pub fn maximum_f32_scalar_op() -> ScalarOpKey {
    governed_scalar_op("maximum-f32")
}

/// Returns the governed per-point `f32` NaN-canonicalization scalar key.
///
/// This is the index-region counterpart of the structured kernel's
/// `ConvertOp::CanonicalizeF32Nan`: a named typed conversion, deliberately not
/// arithmetic. It exists because the numerical contract places a
/// canonicalization at a reduction's *result boundary* — where no combine has
/// necessarily run — so a lowering needs to apply the governed canonical
/// arithmetic-NaN payload without performing an addition that would perturb an
/// observable signed zero.
///
/// The operation identity fixes the payload, matching the versioned
/// `tiler::canonical-arithmetic-nan-f32@1` profile; it carries no attribute
/// selecting a different pattern.
#[must_use]
pub fn canonicalize_nan_f32_scalar_op() -> ScalarOpKey {
    governed_scalar_op("canonicalize-nan-f32")
}

/// Returns the governed per-point strict-affine U4-to-F32 decode operation.
///
/// Its ordered operands are the logical U4 code, positive-normal F32 scale,
/// and logical U4 zero point. The operation performs widened I32 subtraction,
/// exact I32-to-F32 conversion, and one separately rounded F32 multiplication.
#[must_use]
pub fn strict_affine_u4_dequantize_scalar_op() -> ScalarOpKey {
    governed_scalar_op("strict-affine-u4-dequantize")
}

/// Returns the governed per-point `bf16` constant scalar operation key.
///
/// A separate key beside [`constant_f32_scalar_op`] rather than one constant
/// parameterized by its payload's format, for the reason
/// [`crate::semantic::constant_bf16_op`] gives at the tensor layer: operand and
/// result type are part of an operation's identity, and one key meaning two
/// formats would mean two payload widths and two roundings under one identity.
#[must_use]
pub fn constant_bf16_scalar_op() -> ScalarOpKey {
    governed_scalar_op("constant-bf16")
}

/// Returns the governed per-point `bf16` multiplication scalar operation key.
#[must_use]
pub fn multiply_bf16_scalar_op() -> ScalarOpKey {
    governed_scalar_op("multiply-bf16")
}

/// Returns the governed per-point `bf16` addition scalar operation key.
///
/// There is deliberately no `bf16` division, exponential, or NaN-canonicalization
/// key beside these three: the semantic layer registers exactly the constant,
/// multiply, and add for this width, and a scalar with no registered operation
/// above it is one a law could emit into a region nothing means.
#[must_use]
pub fn add_bf16_scalar_op() -> ScalarOpKey {
    governed_scalar_op("add-bf16")
}

fn governed_scalar_op(name: &str) -> ScalarOpKey {
    ScalarOpKey::new("tiler.scalar", name, 1).expect("the governed scalar key is valid")
}

/// Stable identity of one scalar operation family.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarOpKey(TypeKey);

impl ScalarOpKey {
    /// Creates a portable, versioned operation identity.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] when any identity component is invalid.
    pub fn new(
        namespace: impl AsRef<str>,
        name: impl AsRef<str>,
        version: u32,
    ) -> Result<Self, TypeIdentityError> {
        TypeKey::new(namespace, name, version).map(Self)
    }

    /// Validates and retains already-owned operation-key components without copying them.
    ///
    /// # Errors
    ///
    /// Returns [`TypeIdentityError`] before retaining invalid components.
    pub fn from_owned(
        namespace: String,
        name: String,
        version: u32,
    ) -> Result<Self, TypeIdentityError> {
        TypeKey::from_owned(namespace, name, version).map(Self)
    }
    /// Returns the namespace.
    #[must_use]
    pub fn namespace(&self) -> &str {
        self.0.namespace()
    }
    /// Returns the name.
    #[must_use]
    pub fn name(&self) -> &str {
        self.0.name()
    }
    /// Returns the semantic version.
    #[must_use]
    pub const fn semantic_version(&self) -> u32 {
        self.0.semantic_version()
    }
}

/// One bounded canonical scalar attribute record.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarAttributes(CanonicalValue);

impl ScalarAttributes {
    /// Creates attributes from a canonical record.
    ///
    /// # Errors
    ///
    /// Returns [`ScalarRegistryError::AttributesNotRecord`] for any other value kind.
    pub fn new(value: CanonicalValue) -> Result<Self, ScalarRegistryError> {
        if !matches!(value.view(), CanonicalValueView::Record(_)) {
            return Err(ScalarRegistryError::AttributesNotRecord);
        }
        Ok(Self(value))
    }
    /// Creates an empty record.
    ///
    /// # Panics
    ///
    /// Panics only if the semantic canonical-value implementation rejects an empty record.
    #[must_use]
    pub fn empty() -> Self {
        Self(CanonicalValue::record([]).expect("empty record is valid"))
    }
    /// Returns the canonical value.
    #[must_use]
    pub const fn value(&self) -> &CanonicalValue {
        &self.0
    }
}

/// One scalar attribute-schema field.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarAttributeField {
    id: AttributeFieldId,
    kind: CanonicalValueKind,
    required: bool,
    default: Option<CanonicalValue>,
}

impl ScalarAttributeField {
    /// Creates one required field.
    #[must_use]
    pub const fn required(id: AttributeFieldId, kind: CanonicalValueKind) -> Self {
        Self {
            id,
            kind,
            required: true,
            default: None,
        }
    }
    /// Creates one optional field without a default.
    #[must_use]
    pub const fn optional(id: AttributeFieldId, kind: CanonicalValueKind) -> Self {
        Self {
            id,
            kind,
            required: false,
            default: None,
        }
    }
    /// Creates an optional field whose explicit default canonicalizes to omission.
    ///
    /// # Errors
    ///
    /// Returns [`ScalarRegistryError::AttributeDefaultKind`] for a category mismatch.
    pub fn defaulted(
        id: AttributeFieldId,
        kind: CanonicalValueKind,
        default: CanonicalValue,
    ) -> Result<Self, ScalarRegistryError> {
        if canonical_kind(&default) != kind {
            return Err(ScalarRegistryError::AttributeDefaultKind { id });
        }
        Ok(Self {
            id,
            kind,
            required: false,
            default: Some(default),
        })
    }
    /// Returns the stable field ID.
    #[must_use]
    pub const fn id(&self) -> AttributeFieldId {
        self.id
    }
    /// Returns the required canonical value kind.
    #[must_use]
    pub const fn kind(&self) -> CanonicalValueKind {
        self.kind
    }
    /// Returns whether the field must be present.
    #[must_use]
    pub const fn is_required(&self) -> bool {
        self.required
    }
    /// Returns the schema-owned default, if any.
    #[must_use]
    pub const fn default(&self) -> Option<&CanonicalValue> {
        self.default.as_ref()
    }
}

/// Bounded field-ID ordered scalar attribute schema.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarAttributeSchema(Vec<ScalarAttributeField>);

impl ScalarAttributeSchema {
    /// Creates a checked schema.
    ///
    /// # Errors
    ///
    /// Returns an error when the schema exceeds its bound or repeats a field ID.
    pub fn new(
        fields: impl IntoIterator<Item = ScalarAttributeField>,
    ) -> Result<Self, ScalarRegistryError> {
        let mut collected = Vec::new();
        for field in fields {
            if collected.len() == MAX_SCALAR_ATTRIBUTES {
                return Err(ScalarRegistryError::TooManyAttributeFields {
                    actual: MAX_SCALAR_ATTRIBUTES + 1,
                });
            }
            collected.push(field);
        }
        collected.sort_by_key(|field| field.id);
        if collected.windows(2).any(|pair| pair[0].id == pair[1].id) {
            return Err(ScalarRegistryError::DuplicateAttributeField);
        }
        Ok(Self(collected))
    }
    /// Returns an empty schema.
    #[must_use]
    pub const fn empty() -> Self {
        Self(Vec::new())
    }
    /// Returns fields in stable field-ID order.
    #[must_use]
    pub fn fields(&self) -> &[ScalarAttributeField] {
        &self.0
    }
    fn validate(&self, attributes: &ScalarAttributes) -> Result<(), ScalarRegistryError> {
        let CanonicalValueView::Record(values) = attributes.0.view() else {
            return Err(ScalarRegistryError::AttributesNotRecord);
        };
        for value in values {
            let Some(field) = self.0.iter().find(|field| field.id == value.id()) else {
                return Err(ScalarRegistryError::UnknownAttribute { id: value.id() });
            };
            if canonical_kind(value.value()) != field.kind {
                return Err(ScalarRegistryError::AttributeKind { id: value.id() });
            }
        }
        for field in &self.0 {
            if field.required && !values.iter().any(|value| value.id() == field.id) {
                return Err(ScalarRegistryError::MissingAttribute { id: field.id });
            }
        }
        Ok(())
    }

    fn normalize(
        &self,
        attributes: &ScalarAttributes,
    ) -> Result<ScalarAttributes, ScalarRegistryError> {
        self.validate(attributes)?;
        let CanonicalValueView::Record(values) = attributes.value().view() else {
            return Err(ScalarRegistryError::AttributesNotRecord);
        };
        let fields = values.iter().filter(|field| {
            self.0
                .binary_search_by_key(&field.id(), ScalarAttributeField::id)
                .ok()
                .and_then(|index| self.0[index].default.as_ref())
                != Some(field.value())
        });
        let value = CanonicalValue::record(fields.cloned())
            .map_err(|error| ScalarRegistryError::CanonicalAttributes(Arc::new(error)))?;
        ScalarAttributes::new(value)
    }

    fn resolve_defaults(
        &self,
        canonical: &ScalarAttributes,
    ) -> Result<ScalarAttributes, ScalarRegistryError> {
        let CanonicalValueView::Record(values) = canonical.value().view() else {
            return Err(ScalarRegistryError::AttributesNotRecord);
        };
        let mut fields = values.to_vec();
        for schema in &self.0 {
            if let Some(default) = &schema.default
                && !values.iter().any(|field| field.id() == schema.id)
            {
                fields.push(crate::semantic::CanonicalField::new(
                    schema.id,
                    default.clone(),
                ));
            }
        }
        let value = CanonicalValue::record(fields)
            .map_err(|error| ScalarRegistryError::CanonicalAttributes(Arc::new(error)))?;
        ScalarAttributes::new(value)
    }
}

fn canonical_kind(value: &CanonicalValue) -> CanonicalValueKind {
    match value.view() {
        CanonicalValueView::Type(_) => CanonicalValueKind::Type,
        CanonicalValueView::Bool(_) => CanonicalValueKind::Bool,
        CanonicalValueView::Signed { .. } => CanonicalValueKind::Signed,
        CanonicalValueView::Unsigned { .. } => CanonicalValueKind::Unsigned,
        CanonicalValueView::FloatBits(_) => CanonicalValueKind::FloatBits,
        CanonicalValueView::Bytes(_) => CanonicalValueKind::Bytes,
        CanonicalValueView::Utf8(_) => CanonicalValueKind::Utf8,
        CanonicalValueView::Sequence(_) => CanonicalValueKind::Sequence,
        CanonicalValueView::Record(_) => CanonicalValueKind::Record,
    }
}

/// Inclusive operand or result arity bounds.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ScalarArity {
    min: usize,
    max: usize,
}

impl ScalarArity {
    /// Creates inclusive bounds.
    ///
    /// # Errors
    ///
    /// Returns [`ScalarRegistryError::InvalidArityRange`] for reversed or oversized bounds.
    pub fn range(min: usize, max: usize) -> Result<Self, ScalarRegistryError> {
        if min > max || max > MAX_SCALAR_ARITY {
            return Err(ScalarRegistryError::InvalidArityRange);
        }
        Ok(Self { min, max })
    }
    /// Creates an exact arity.
    ///
    /// # Errors
    ///
    /// Returns [`ScalarRegistryError::InvalidArityRange`] when `count` exceeds the bound.
    pub fn exact(count: usize) -> Result<Self, ScalarRegistryError> {
        Self::range(count, count)
    }
    /// Returns the inclusive minimum arity.
    #[must_use]
    pub const fn min(self) -> usize {
        self.min
    }
    /// Returns the inclusive maximum arity.
    #[must_use]
    pub const fn max(self) -> usize {
        self.max
    }
    /// Returns whether `actual` satisfies these bounds.
    #[must_use]
    pub fn accepts(self, actual: usize) -> bool {
        (self.min..=self.max).contains(&actual)
    }
}

/// Immutable input passed once to a provider inferencer during construction.
#[derive(Clone, Copy, Debug)]
pub struct ScalarInferenceRequest<'a> {
    operands: &'a [ResolvedValueType],
    attributes: &'a ScalarAttributes,
}

impl<'a> ScalarInferenceRequest<'a> {
    const fn new(operands: &'a [ResolvedValueType], attributes: &'a ScalarAttributes) -> Self {
        Self {
            operands,
            attributes,
        }
    }

    /// Returns operand types in semantic order.
    #[must_use]
    pub const fn operands(self) -> &'a [ResolvedValueType] {
        self.operands
    }

    /// Returns resolved canonical attributes.
    #[must_use]
    pub const fn attributes(self) -> &'a ScalarAttributes {
        self.attributes
    }
}

/// Stable provider rejection of one application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarInferenceError {
    code: ProviderDiagnosticCode,
    message: String,
}

impl ScalarInferenceError {
    /// Creates an inference rejection.
    ///
    /// # Errors
    ///
    /// Returns a provider-diagnostic contract error for an empty or oversized message.
    pub fn new(
        code: ProviderDiagnosticCode,
        message: impl Into<String>,
    ) -> Result<Self, ProviderDiagnosticError> {
        let message = message.into();
        if message.is_empty() {
            return Err(ProviderDiagnosticError::EmptyMessage);
        }
        if message.len() > MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES {
            return Err(ProviderDiagnosticError::MessageTooLong {
                bytes: message.len(),
            });
        }
        Ok(Self { code, message })
    }
    /// Returns the stable diagnostic code.
    #[must_use]
    pub const fn code(&self) -> &ProviderDiagnosticCode {
        &self.code
    }
    /// Returns diagnostic detail.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}
impl fmt::Display for ScalarInferenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl Error for ScalarInferenceError {}

/// Complete provider-attributed rejection of one scalar application.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarApplicationRejection {
    key: ScalarOpKey,
    provider: ProviderIdentity,
    source: ScalarInferenceError,
}

impl ScalarApplicationRejection {
    /// Returns the rejected scalar operation family.
    #[must_use]
    pub const fn key(&self) -> &ScalarOpKey {
        &self.key
    }

    /// Returns the provider governing the rejected definition.
    #[must_use]
    pub const fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    /// Returns the provider's bounded diagnostic.
    #[must_use]
    pub const fn rejection(&self) -> &ScalarInferenceError {
        &self.source
    }
}

impl fmt::Display for ScalarApplicationRejection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "scalar operation {:?} rejected by {}: {}",
            self.key, self.provider, self.source
        )
    }
}

impl Error for ScalarApplicationRejection {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

fn host_inference_error(code: &'static str, message: &'static str) -> ScalarInferenceError {
    ScalarInferenceError::new(
        ProviderDiagnosticCode::new(code).expect("host diagnostic code is canonical"),
        message,
    )
    .expect("host diagnostic message is canonical")
}

/// Host-owned bounded writer for ordered scalar inference results.
///
/// A rejected push permanently poisons the writer. Ignoring the returned error
/// therefore cannot commit a truncated result list.
#[derive(Debug)]
pub struct ScalarInferenceOutputs {
    results: Vec<ResolvedValueType>,
    contract_maximum: usize,
    host_result_slots: usize,
    result_count_before: usize,
    result_limit: usize,
    remaining_canonical_bytes: usize,
    initial_canonical_bytes: usize,
    retained_bytes_before: usize,
    retained_byte_limit: usize,
    per_result_overhead: usize,
    byte_multiplier: usize,
    provider_failure: Option<ScalarInferenceError>,
    host_failure: Option<ScalarInferenceHostFailure>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ScalarInferenceCapacity {
    pub result_slots: usize,
    pub result_count_before: usize,
    pub result_limit: usize,
    pub retained_bytes: usize,
    pub retained_bytes_before: usize,
    pub retained_byte_limit: usize,
    pub per_result_overhead: usize,
    pub byte_multiplier: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ScalarInferenceHostFailure {
    ResultSlots { actual: usize, limit: usize },
    CanonicalBytes { actual: usize, limit: usize },
}

#[derive(Debug, Eq, PartialEq)]
enum ScalarInferenceFinishError {
    Provider(ScalarInferenceError),
    Host(ScalarInferenceHostFailure),
}

#[derive(Debug)]
pub(super) enum ScalarApplyError {
    Authority(ScalarRegistryError),
    Host(ScalarInferenceHostFailure),
}

impl ScalarInferenceOutputs {
    fn new(maximum: usize, capacity: ScalarInferenceCapacity) -> Self {
        Self {
            results: Vec::new(),
            contract_maximum: maximum,
            host_result_slots: capacity.result_slots,
            result_count_before: capacity.result_count_before,
            result_limit: capacity.result_limit,
            remaining_canonical_bytes: capacity.retained_bytes,
            initial_canonical_bytes: capacity.retained_bytes,
            retained_bytes_before: capacity.retained_bytes_before,
            retained_byte_limit: capacity.retained_byte_limit,
            per_result_overhead: capacity.per_result_overhead,
            byte_multiplier: capacity.byte_multiplier,
            provider_failure: None,
            host_failure: None,
        }
    }

    /// Appends one inferred result in semantic order.
    ///
    /// # Errors
    ///
    /// Returns a sticky host diagnostic after the registered result maximum or
    /// aggregate canonical-byte budget is exceeded.
    pub fn try_push(&mut self, value_type: ResolvedValueType) -> Result<(), ScalarInferenceError> {
        if let Some(failure) = self.host_failure {
            return Err(host_failure_diagnostic(failure));
        }
        if let Some(error) = &self.provider_failure {
            return Err(error.clone());
        }
        if self.results.len() >= self.contract_maximum {
            let error = host_inference_error(
                "tiler.scalar.result-limit",
                "scalar inference produced more results than its registered contract permits",
            );
            self.provider_failure = Some(error.clone());
            return Err(error);
        }
        if self.results.len() >= self.host_result_slots {
            let failure = ScalarInferenceHostFailure::ResultSlots {
                actual: self
                    .result_count_before
                    .saturating_add(self.results.len())
                    .saturating_add(1),
                limit: self.result_limit,
            };
            self.host_failure = Some(failure);
            return Err(host_failure_diagnostic(failure));
        }
        let bytes = value_type
            .canonical_encoding()
            .as_bytes()
            .len()
            .saturating_add(self.per_result_overhead)
            .saturating_mul(self.byte_multiplier);
        let Some(remaining) = self.remaining_canonical_bytes.checked_sub(bytes) else {
            let consumed = self
                .retained_bytes_before
                .saturating_add(
                    self.initial_canonical_bytes
                        .saturating_sub(self.remaining_canonical_bytes),
                )
                .saturating_add(bytes);
            let failure = ScalarInferenceHostFailure::CanonicalBytes {
                actual: consumed,
                limit: self.retained_byte_limit,
            };
            self.host_failure = Some(failure);
            return Err(host_failure_diagnostic(failure));
        };
        self.results.push(value_type);
        self.remaining_canonical_bytes = remaining;
        Ok(())
    }

    fn finish(
        self,
        callback: Result<(), ScalarInferenceError>,
        minimum: usize,
    ) -> Result<Vec<ResolvedValueType>, ScalarInferenceFinishError> {
        if let Some(failure) = self.host_failure {
            return Err(ScalarInferenceFinishError::Host(failure));
        }
        if let Some(error) = self.provider_failure {
            return Err(ScalarInferenceFinishError::Provider(error));
        }
        callback.map_err(ScalarInferenceFinishError::Provider)?;
        if self.results.len() < minimum {
            return Err(ScalarInferenceFinishError::Provider(host_inference_error(
                "tiler.scalar.result-minimum",
                "scalar inference produced fewer results than its registered contract requires",
            )));
        }
        Ok(self.results)
    }
}

fn host_failure_diagnostic(failure: ScalarInferenceHostFailure) -> ScalarInferenceError {
    match failure {
        ScalarInferenceHostFailure::ResultSlots { .. } => host_inference_error(
            "tiler.scalar.host-result-capacity",
            "scalar inference exceeds the enclosing graph's result capacity",
        ),
        ScalarInferenceHostFailure::CanonicalBytes { .. } => host_inference_error(
            "tiler.scalar.host-byte-capacity",
            "scalar inference exceeds the enclosing graph's canonical-byte capacity",
        ),
    }
}

/// Pure construction-time result-type inference.
pub trait ScalarOperationInferencer: Send + Sync + 'static {
    /// Infers all ordered result types.
    ///
    /// # Errors
    ///
    /// Returns a stable provider error when the operand types or attributes are unsupported.
    fn infer(
        &self,
        request: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError>;
}

/// Host-enforced effect contract for scalar operations admitted to CSE.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ScalarEffect {
    /// Deterministic, side-effect-free semantics depending only on explicit inputs.
    Pure,
}

/// Declarative, provider-independent contract of one scalar operation family.
#[derive(Clone, Debug)]
pub struct ScalarOperationContract {
    attributes: ScalarAttributeSchema,
    operands: ScalarArity,
    results: ScalarArity,
    effect: ScalarEffect,
    facts: CanonicalValue,
    conformance: CanonicalValue,
}

impl ScalarOperationContract {
    /// Creates a complete additive scalar operation contract.
    #[must_use]
    pub const fn new(
        attributes: ScalarAttributeSchema,
        operands: ScalarArity,
        results: ScalarArity,
        effect: ScalarEffect,
        facts: CanonicalValue,
        conformance: CanonicalValue,
    ) -> Self {
        Self {
            attributes,
            operands,
            results,
            effect,
            facts,
            conformance,
        }
    }

    /// Returns the canonical attribute schema.
    #[must_use]
    pub const fn attributes(&self) -> &ScalarAttributeSchema {
        &self.attributes
    }

    /// Returns admitted operand arity.
    #[must_use]
    pub const fn operands(&self) -> ScalarArity {
        self.operands
    }

    /// Returns admitted result arity.
    #[must_use]
    pub const fn results(&self) -> ScalarArity {
        self.results
    }

    /// Returns the effect contract.
    #[must_use]
    pub const fn effect(&self) -> ScalarEffect {
        self.effect
    }

    /// Returns canonical semantic facts.
    #[must_use]
    pub const fn facts(&self) -> &CanonicalValue {
        &self.facts
    }

    /// Returns canonical conformance requirements.
    #[must_use]
    pub const fn conformance(&self) -> &CanonicalValue {
        &self.conformance
    }
}

/// Provider-independent portable scalar operation definition.
#[derive(Clone)]
pub struct ScalarOperationDefinition {
    key: ScalarOpKey,
    normative_definition: NormativeDefinitionRef,
    attributes: ScalarAttributeSchema,
    operands: ScalarArity,
    results: ScalarArity,
    effect: ScalarEffect,
    facts: CanonicalValue,
    conformance: CanonicalValue,
    inferencer: Arc<dyn ScalarOperationInferencer>,
}

impl fmt::Debug for ScalarOperationDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScalarOperationDefinition")
            .field("key", &self.key)
            .field("normative_definition", &self.normative_definition)
            .field("attributes", &self.attributes)
            .field("operands", &self.operands)
            .field("results", &self.results)
            .field("effect", &self.effect)
            .field("facts", &self.facts)
            .field("conformance", &self.conformance)
            .finish_non_exhaustive()
    }
}

impl ScalarOperationDefinition {
    /// Creates a complete definition. The host validates every application.
    #[must_use]
    pub fn new(
        key: ScalarOpKey,
        normative_definition: NormativeDefinitionRef,
        contract: ScalarOperationContract,
        inferencer: Arc<dyn ScalarOperationInferencer>,
    ) -> Self {
        Self {
            key,
            normative_definition,
            attributes: contract.attributes,
            operands: contract.operands,
            results: contract.results,
            effect: contract.effect,
            facts: contract.facts,
            conformance: contract.conformance,
            inferencer,
        }
    }
    /// Returns the operation key.
    #[must_use]
    pub const fn key(&self) -> &ScalarOpKey {
        &self.key
    }
    /// Returns the host-enforced effect contract.
    #[must_use]
    pub const fn effect(&self) -> ScalarEffect {
        self.effect
    }
    /// Returns the stable normative definition identity.
    #[must_use]
    pub const fn normative_definition(&self) -> &NormativeDefinitionRef {
        &self.normative_definition
    }
    /// Returns the canonical attribute schema.
    #[must_use]
    pub const fn attributes(&self) -> &ScalarAttributeSchema {
        &self.attributes
    }
    /// Returns admitted operand arity.
    #[must_use]
    pub const fn operands(&self) -> ScalarArity {
        self.operands
    }
    /// Returns admitted result arity.
    #[must_use]
    pub const fn results(&self) -> ScalarArity {
        self.results
    }
    /// Returns canonical semantic facts.
    #[must_use]
    pub const fn facts(&self) -> &CanonicalValue {
        &self.facts
    }
    /// Returns canonical conformance requirements.
    #[must_use]
    pub const fn conformance(&self) -> &CanonicalValue {
        &self.conformance
    }
}

#[derive(Clone)]
struct RegisteredScalarOperation {
    definition: ScalarOperationDefinition,
    provider: ProviderIdentity,
}

/// # The governed scalar fact-field vocabulary
///
/// Published for the same reason the semantic layer's is: `facts()` is
/// publicly readable and an out-of-crate reference capability or index-access
/// lowering provider is the consumer these facts were declared for. Without
/// these it must hardcode a bare integer against a numbering no contract
/// states.
///
/// **These identifiers are local to the scalar fact record.** They are not the
/// semantic layer's, and an equal integer there names a different field —
/// scalar field 4 is contraction, while the semantic arithmetic record spells
/// contraction as its field 3. Nothing normalizes them, and the storage shape
/// matching is not a reason to.
///
/// **Renumbering a published ID is a breaking identity change.**
/// Rounding rule a governed scalar operation applies to its result.
///
/// Its meaning is the same on every governed scalar definition, so a consumer
/// that reads this field on one operation reads it the same way on all of them.
pub const SCALAR_FACT_ROUNDING: AttributeFieldId = AttributeFieldId::new(1);

/// Rule deciding which NaN payload a governed scalar operation's result carries.
///
/// Every governed scalar definition states this field. It is the field that
/// makes a *preserving* operation distinguishable from one whose payload
/// behaviour was merely never written down: absence of
/// [`SCALAR_FACT_CANONICAL_NAN_BITS`] never carries meaning on its own.
///
/// **The admitted values are [`CANONICAL_ARITHMETIC_NAN_PROFILE`] and
/// [`DECLARED_PAYLOAD_PRESERVED`], and the vocabulary is deliberately still
/// two.** The field's scope is what keeps it that way: it decides the payload of
/// a *NaN result*, so an operation that installs nothing on its ordinary domain
/// still answers it whenever it produces a NaN at all. Neither the NaN
/// canonicalization nor the maximum is arithmetic, and both name the profile;
/// `exact_bit_pattern_f32_scalar_facts` in this module carries that derivation
/// and the evidence boundary behind it. An operand-payload-*propagating* rule would be a
/// third value, and nothing registered here has one: no governed scalar lets a
/// NaN operand's payload reach its result.
pub const SCALAR_FACT_NAN_RESULT_RULE: AttributeFieldId = AttributeFieldId::new(2);

/// Exact canonical arithmetic-NaN payload the operation installs, when it does.
///
/// Present exactly when [`SCALAR_FACT_NAN_RESULT_RULE`] names the canonical
/// arithmetic-NaN profile; an operation that installs no payload omits it
/// rather than declaring one it never produces.
pub const SCALAR_FACT_CANONICAL_NAN_BITS: AttributeFieldId = AttributeFieldId::new(3);

/// Whether the operation may be contracted with an adjacent arithmetic scalar.
///
/// Present only on the arithmetic scalars, because contraction is defined over
/// a pattern of arithmetic operations; a constant or a conversion is not a
/// participant, and asserting `false` there would answer a question the
/// numerical contract does not pose.
pub const SCALAR_FACT_CONTRACTION_PERMITTED: AttributeFieldId = AttributeFieldId::new(4);

/// Names the versioned NaN profile the governed arithmetic scalars realize.
///
/// This is the exact profile `docs/numerical-semantics.md` names, not a
/// synonym, so a reader can trace the fact to the normative clause that decides
/// it.
pub const CANONICAL_ARITHMETIC_NAN_PROFILE: &str = "tiler::canonical-arithmetic-nan-f32@1";

/// Names the opposite rule: the declared payload survives verbatim.
///
/// `docs/numerical-semantics.md` states it as "Constants retain their declared
/// bit pattern until an operation's semantics produce a new value."
pub const DECLARED_PAYLOAD_PRESERVED: &str = "declared-payload-preserved";

/// Facts of the governed `f32` constant: an exact payload, canonicalized never.
fn constant_f32_facts() -> Result<CanonicalValue, ScalarRegistryError> {
    scalar_facts([
        (SCALAR_FACT_ROUNDING, utf8_fact("exact-binary32-bits")?),
        (
            SCALAR_FACT_NAN_RESULT_RULE,
            utf8_fact(DECLARED_PAYLOAD_PRESERVED)?,
        ),
    ])
}

/// Facts shared by the governed binary `f32` arithmetic scalars.
///
/// These restate the rounding and canonical-NaN rules that
/// [`crate::semantic::FrozenSemanticRegistry::standard`] declares for the
/// tensor-level families these scalars realize. The two records are checked
/// against each other rather than derived from one another; see
/// `standard_scalar_conformance` for why.
fn arithmetic_f32_scalar_facts() -> Result<CanonicalValue, ScalarRegistryError> {
    scalar_facts([
        (
            SCALAR_FACT_ROUNDING,
            utf8_fact("binary32-round-to-nearest-ties-even")?,
        ),
        (
            SCALAR_FACT_NAN_RESULT_RULE,
            utf8_fact(CANONICAL_ARITHMETIC_NAN_PROFILE)?,
        ),
        (
            SCALAR_FACT_CANONICAL_NAN_BITS,
            crate::semantic::canonical_f32_bits(crate::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS),
        ),
        (
            SCALAR_FACT_CONTRACTION_PERMITTED,
            CanonicalValue::boolean(false),
        ),
    ])
}

/// Facts shared by the governed precise binary32 elementary functions.
///
/// They state the same NaN rule and the same canonical payload as the arithmetic
/// scalars, and they deliberately state **no rounding rule of their own**: an
/// elementary function's admitted result set is its operation's resolved accuracy
/// contract, and writing "round-to-nearest ties-to-even" here would claim a
/// correctly rounded result that nothing establishes. The contraction field is
/// absent for the same reason it is absent from a conversion — there is no
/// adjacent product to fuse into.
///
/// One record rather than one per key, on the same ground
/// [`arithmetic_f32_scalar_facts`] is shared by three: the three fields say the
/// same thing about the exponential and the reciprocal square root, and two
/// copies would be two authorities over one statement that could drift. What
/// separates the two keys is not this record but the resolved accuracy contract
/// their operations state, which is a different layer's authority — and the
/// rounding field says so by naming that layer rather than a rule.
fn elementary_f32_scalar_facts() -> Result<CanonicalValue, ScalarRegistryError> {
    scalar_facts([
        (
            SCALAR_FACT_ROUNDING,
            utf8_fact("resolved-by-the-operation-accuracy-contract")?,
        ),
        (
            SCALAR_FACT_NAN_RESULT_RULE,
            utf8_fact(CANONICAL_ARITHMETIC_NAN_PROFILE)?,
        ),
        (
            SCALAR_FACT_CANONICAL_NAN_BITS,
            crate::semantic::canonical_f32_bits(crate::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS),
        ),
    ])
}

/// Facts shared by the governed binary32 scalars that select rather than compute.
///
/// Its two members are [`canonicalize_nan_f32_scalar_op`] and
/// [`maximum_f32_scalar_op`]. Neither is arithmetic: each rounds nothing and
/// reproduces an operand's binary32 pattern verbatim on every non-NaN input,
/// including the sign of a zero — the conversion reproduces its one operand, and
/// the maximum reproduces whichever operand the ordering selects. That exactness
/// is the reason a reduction can canonicalize a singleton result without an
/// addition, and the reason the maximum carries no rounding obligation and its
/// reduction declares no accumulator width.
///
/// **The three fields say the same thing about both, which is why there is one
/// record and not two** — the ground [`elementary_f32_scalar_facts`] states for
/// the exponential and the reciprocal square root, and two copies would be two
/// authorities over one statement that could drift. What separates the two keys
/// is their arity and their registered normative definition, not this record.
///
/// # Why the maximum names the canonical arithmetic-NaN profile, and mints no third rule
///
/// [`SCALAR_FACT_NAN_RESULT_RULE`] decides *which NaN payload a result carries*,
/// and this operation's answer is the governed canonical one. That is derived
/// rather than chosen, from three agreeing authorities:
///
/// - **ADR 0023's Decision, verbatim:** "Portable-bitwise NaN results use the
///   existing canonical arithmetic-NaN contract" — stated of both extrema
///   families, beside the `-0.0 < +0.0` requirement.
/// - **[Numerical semantics](../../../../docs/numerical-semantics.md), "Min and
///   max":** "Under portable-bitwise conformance, a produced NaN follows the
///   canonical arithmetic-NaN contract." The clause's only subject is the two
///   extrema families.
/// - **Both delivered realizations, which agree with it.** The Metal fixup's
///   unordered arm returns the canonical pattern directly rather than propagating
///   an operand (`maximum_helper` in `crates/tiler-metal/src/emit.rs`), and the
///   reference's `maximum_f32` returns `f32::NAN`, which is that pattern
///   (`crates/tiler-reference/src/softmax.rs`).
///
/// **So the operand-payload-selecting rule this key was expected to need does not
/// exist, and asserting one would be false.** The expectation rested on reading
/// "performs no arithmetic" as "installs no payload"; those are different claims,
/// and [`canonicalize_nan_f32_scalar_op`] already separates them — it is
/// explicitly not arithmetic, computes nothing, and names this profile. On an
/// *ordered* pair the maximum installs nothing, but that is a statement about
/// non-NaN results, which is not what this field decides and which the profile
/// value therefore does not claim.
///
/// A **signalling** NaN operand needs no clause of its own and gets none. It
/// makes the pair unordered exactly as a quiet NaN does, so the value answer is
/// identical and both delivered realizations reach it without a special case; and
/// the invalid-operation signal IEEE 754 would raise is outside Tiler's
/// observable contract altogether, which numerical semantics fixes as value-only
/// (`RaiseNoFlag`) rather than leaving to a host. An evidence boundary belongs
/// beside that: IEEE Std 754-2019 is `metadata-only` in
/// `docs/research/numerics/sources` — purchased and not redistributable — so the
/// standard's own clause text is not readable from this tree. What the repository
/// holds is the reading in
/// [floating-point extrema precedents](../../../../docs/research/numerics/floating-point-extrema-precedents.md):
/// that `minimum`/`maximum` propagate NaN and order `-0.0 < +0.0` separately from
/// `minimumNumber`/`maximumNumber`. That record states no payload rule and no
/// sNaN rule for these families, which is why the payload above is derived from
/// Tiler's own accepted contract rather than cited to the standard.
fn exact_bit_pattern_f32_scalar_facts() -> Result<CanonicalValue, ScalarRegistryError> {
    scalar_facts([
        (SCALAR_FACT_ROUNDING, utf8_fact("exact-binary32-bits")?),
        (
            SCALAR_FACT_NAN_RESULT_RULE,
            utf8_fact(CANONICAL_ARITHMETIC_NAN_PROFILE)?,
        ),
        (
            SCALAR_FACT_CANONICAL_NAN_BITS,
            crate::semantic::canonical_f32_bits(crate::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS),
        ),
    ])
}

/// Facts of the governed per-point `bf16` constant.
///
/// It rounds nothing — the declared payload is already the exact `bf16`
/// encoding — and preserves the payload it was given, which is why it states no
/// canonical arithmetic NaN and no contraction permission. This restates, in the
/// scalar fact vocabulary, what [`crate::semantic::constant_bf16_facts`]
/// declares at the tensor layer; the two records are checked against each other
/// rather than derived from one another, as the `f32` pair already are.
fn constant_bf16_scalar_facts() -> Result<CanonicalValue, ScalarRegistryError> {
    scalar_facts([
        (SCALAR_FACT_ROUNDING, utf8_fact("exact-bf16-bits")?),
        (
            SCALAR_FACT_NAN_RESULT_RULE,
            utf8_fact(DECLARED_PAYLOAD_PRESERVED)?,
        ),
    ])
}

/// Facts shared by the governed binary `bf16` arithmetic scalars.
///
/// The canonical NaN payload is `bf16`'s own sixteen-bit pattern, not binary32's
/// zero-extended: a scalar declaring the wider payload would name a value this
/// format cannot hold. Contraction is stated `false` for the same reason the
/// tensor family states it — `metal` admits no `fma(bfloat, bfloat, bfloat)`, so
/// there is no fused primitive to permit.
fn arithmetic_bf16_scalar_facts() -> Result<CanonicalValue, ScalarRegistryError> {
    scalar_facts([
        (
            SCALAR_FACT_ROUNDING,
            utf8_fact("bf16-round-to-nearest-ties-even")?,
        ),
        (
            SCALAR_FACT_NAN_RESULT_RULE,
            utf8_fact(CANONICAL_ARITHMETIC_NAN_PROFILE)?,
        ),
        (
            SCALAR_FACT_CANONICAL_NAN_BITS,
            crate::semantic::canonical_bf16_bits(
                crate::semantic::CANONICAL_BF16_ARITHMETIC_NAN_BITS,
            ),
        ),
        (
            SCALAR_FACT_CONTRACTION_PERMITTED,
            CanonicalValue::boolean(false),
        ),
    ])
}

fn utf8_fact(value: &str) -> Result<CanonicalValue, ScalarRegistryError> {
    CanonicalValue::utf8(value)
        .map_err(|source| ScalarRegistryError::CanonicalAttributes(Arc::new(source)))
}

fn scalar_facts<const N: usize>(
    fields: [(AttributeFieldId, CanonicalValue); N],
) -> Result<CanonicalValue, ScalarRegistryError> {
    CanonicalValue::record(
        fields
            .into_iter()
            .map(|(id, value)| crate::semantic::CanonicalField::new(id, value)),
    )
    .map_err(|source| ScalarRegistryError::CanonicalAttributes(Arc::new(source)))
}

/// Builds the conformance identity of one governed scalar definition.
///
/// The prefix is `tiler.scalar.conformance.`, deliberately distinct from the
/// semantic layer's `tiler.conformance.`. The two layers govern different
/// contracts over the same names — one is a whole-tensor operation family, the
/// other a per-point scalar — so letting them share a conformance string would
/// give two subjects one identity.
fn standard_scalar_conformance(name: &str) -> Result<CanonicalValue, ScalarRegistryError> {
    let canonical = |source| ScalarRegistryError::CanonicalAttributes(Arc::new(source));
    CanonicalValue::record([
        crate::semantic::CanonicalField::new(
            AttributeFieldId::new(1),
            CanonicalValue::utf8_owned(format!("tiler.scalar.conformance.{name}"))
                .map_err(canonical)?,
        ),
        crate::semantic::CanonicalField::new(
            AttributeFieldId::new(2),
            CanonicalValue::unsigned_u32(1),
        ),
    ])
    .map_err(canonical)
}

/// Assembles one governed standard scalar definition.
///
/// The conformance identity is derived from the key rather than passed in, so a
/// definition cannot be registered under one name while claiming conformance to
/// another.
fn standard_definition(
    key: ScalarOpKey,
    normative: &str,
    attributes: ScalarAttributeSchema,
    operands: ScalarArity,
    facts: CanonicalValue,
    inferencer: Arc<dyn ScalarOperationInferencer>,
) -> Result<ScalarOperationDefinition, ScalarRegistryError> {
    let authority = |source| ScalarRegistryError::TypeAuthority(Arc::new(source));
    let conformance = standard_scalar_conformance(key.name())?;
    Ok(ScalarOperationDefinition::new(
        key,
        NormativeDefinitionRef::new(normative).map_err(authority)?,
        ScalarOperationContract::new(
            attributes,
            operands,
            ScalarArity::exact(1)?,
            ScalarEffect::Pure,
            facts,
            conformance,
        ),
        inferencer,
    ))
}

fn constant_attribute_schema() -> Result<ScalarAttributeSchema, ScalarRegistryError> {
    // The governed scalar constant reuses the semantic family's stable field
    // identifier, so an index-access lowering forwards an occurrence's attribute
    // record without re-keying it.
    ScalarAttributeSchema::new([ScalarAttributeField::required(
        crate::semantic::F32_CONSTANT_BITS_ATTRIBUTE,
        CanonicalValueKind::FloatBits,
    )])
}

fn constant_bf16_attribute_schema() -> Result<ScalarAttributeSchema, ScalarRegistryError> {
    // The governed `bf16` family's own payload field, not the `f32` one. The two
    // identifiers are record-local and happen to number alike; naming this one
    // explicitly is what keeps the agreement a decision rather than a
    // coincidence a renumbering would silently break.
    ScalarAttributeSchema::new([ScalarAttributeField::required(
        crate::semantic::BF16_CONSTANT_BITS_ATTRIBUTE,
        CanonicalValueKind::FloatBits,
    )])
}

/// Infers the governed `f32` result of a nullary scalar constant.
struct StandardF32Constant;

impl ScalarOperationInferencer for StandardF32Constant {
    fn infer(
        &self,
        _: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        outputs.try_push(crate::semantic::F32::resolved_type())
    }
}

/// Infers the shared operand type of a governed homogeneous `f32` scalar.
///
/// The operand types are required to be `f32` rather than merely numeric: a
/// governed `f32` operation has no defined mixed-type behaviour, so an
/// application that would need one is rejected instead of silently resolving to
/// the first operand. The rule is arity independent, so the binary arithmetic
/// operations and the unary NaN canonicalization share it.
struct StandardF32Homogeneous;

impl ScalarOperationInferencer for StandardF32Homogeneous {
    fn infer(
        &self,
        request: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        let f32_type = crate::semantic::F32::resolved_type();
        if request
            .operands()
            .iter()
            .any(|operand| operand != &f32_type)
        {
            return Err(ScalarInferenceError::new(
                ProviderDiagnosticCode::new("tiler.scalar.operand-type")
                    .expect("the governed diagnostic code is valid"),
                "governed f32 scalars require f32 operands",
            )
            .expect("the governed diagnostic message is bounded"));
        }
        outputs.try_push(f32_type)
    }
}

/// Infers the governed `bf16` result of a nullary scalar constant.
///
/// The payload's declared format and width are checked here rather than left to
/// the schema's `FloatBits` kind alone. A binary32 pattern wearing the `bf16`
/// field would otherwise reach a region whose canonical identity claims a `bf16`
/// constant while carrying a value the format cannot hold, and the refusal
/// naming that is cheaper here than anywhere downstream.
struct StandardBf16Constant {
    payload_bytes: usize,
}

impl ScalarOperationInferencer for StandardBf16Constant {
    fn infer(
        &self,
        request: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        let CanonicalValueView::Record(fields) = request.attributes().value().view() else {
            return Err(bf16_scalar_error(
                "tiler.scalar.bf16-constant-bits",
                "the governed bf16 scalar constant requires a canonical attribute record",
            ));
        };
        let Some(CanonicalValueView::FloatBits(bits)) = fields
            .iter()
            .find(|field| field.id() == crate::semantic::BF16_CONSTANT_BITS_ATTRIBUTE)
            .map(|field| field.value().view())
        else {
            return Err(bf16_scalar_error(
                "tiler.scalar.bf16-constant-bits",
                "the governed bf16 scalar constant requires exact FloatBits in its payload field",
            ));
        };
        // Format before width, so a binary32 payload and a bf16-format payload
        // of the wrong width are two distinct refusals rather than one hiding
        // the other.
        let bf16_format = crate::semantic::TypeKey::new("tiler", "bf16", 1)
            .expect("the governed bf16 key is valid");
        if bits.format() != &bf16_format {
            return Err(bf16_scalar_error(
                "tiler.scalar.bf16-constant-format",
                "the governed bf16 scalar constant admits no payload of another float format",
            ));
        }
        if bits.bits().len() != self.payload_bytes {
            return Err(bf16_scalar_error(
                "tiler.scalar.bf16-constant-width",
                "the governed bf16 scalar constant payload must be the registered bf16 width",
            ));
        }
        outputs.try_push(crate::semantic::Bf16::resolved_type())
    }
}

/// Infers the shared operand type of a governed homogeneous `bf16` scalar.
///
/// Operands must be `bf16` rather than merely numeric, for the reason the `f32`
/// sibling states: this family declares no mixed precision and no implicit
/// promotion, so an application needing either is rejected instead of silently
/// resolving to the first operand's type.
struct StandardBf16Homogeneous;

impl ScalarOperationInferencer for StandardBf16Homogeneous {
    fn infer(
        &self,
        request: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        let bf16_type = crate::semantic::Bf16::resolved_type();
        if request
            .operands()
            .iter()
            .any(|operand| operand != &bf16_type)
        {
            return Err(bf16_scalar_error(
                "tiler.scalar.operand-type",
                "governed bf16 scalars require bf16 operands",
            ));
        }
        outputs.try_push(bf16_type)
    }
}

fn bf16_scalar_error(code: &str, message: &str) -> ScalarInferenceError {
    ScalarInferenceError::new(
        ProviderDiagnosticCode::new(code).expect("the governed diagnostic code is valid"),
        message,
    )
    .expect("the governed diagnostic message is bounded")
}

/// Infers the dense F32 result of the governed strict-affine U4 scalar.
struct StandardStrictAffineU4Dequantize;

impl ScalarOperationInferencer for StandardStrictAffineU4Dequantize {
    fn infer(
        &self,
        request: ScalarInferenceRequest<'_>,
        outputs: &mut ScalarInferenceOutputs,
    ) -> Result<(), ScalarInferenceError> {
        let expected = [
            crate::semantic::U4::resolved_type(),
            crate::semantic::F32::resolved_type(),
            crate::semantic::U4::resolved_type(),
        ];
        if request.operands() != expected {
            return Err(ScalarInferenceError::new(
                ProviderDiagnosticCode::new("tiler.scalar.strict-affine-u4-operands")
                    .expect("the governed diagnostic code is valid"),
                "strict-affine U4 decode requires ordered U4 codes, F32 scale, and U4 zero point",
            )
            .expect("the governed diagnostic message is bounded"));
        }
        outputs.try_push(crate::semantic::F32::resolved_type())
    }
}

/// Failure while defining or applying scalar authority.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ScalarRegistryError {
    /// Scalar attributes were not encoded as a canonical record.
    AttributesNotRecord,
    /// An attribute schema exceeded its governed field count.
    TooManyAttributeFields {
        /// Supplied field count.
        actual: usize,
    },
    /// An attribute schema repeated a field ID.
    DuplicateAttributeField,
    /// An arity range was reversed or exceeded its governed maximum.
    InvalidArityRange,
    /// A registered definition admitted zero results.
    ZeroResultDefinition,
    /// The operation key was already registered.
    DuplicateDefinition {
        /// Duplicated key.
        key: ScalarOpKey,
    },
    /// No operation definition exists for the requested key.
    UnknownOperation {
        /// Unknown key.
        key: ScalarOpKey,
    },
    /// Operand count violates the registered arity.
    OperandArity {
        /// Applied operation key.
        key: ScalarOpKey,
        /// Supplied operand count.
        actual: usize,
    },
    /// Inferred result count violates the registered arity.
    ResultArity {
        /// Applied operation key.
        key: ScalarOpKey,
        /// Inferred result count.
        actual: usize,
    },
    /// Attributes contained an undeclared field.
    UnknownAttribute {
        /// Undeclared field ID.
        id: AttributeFieldId,
    },
    /// Attributes omitted a required field.
    MissingAttribute {
        /// Missing field ID.
        id: AttributeFieldId,
    },
    /// An attribute value did not match its declared kind.
    AttributeKind {
        /// Mismatched field ID.
        id: AttributeFieldId,
    },
    /// A schema default had the wrong canonical value category.
    AttributeDefaultKind {
        /// Invalid default field.
        id: AttributeFieldId,
    },
    /// Canonical attribute normalization failed.
    CanonicalAttributes(Arc<TypeIdentityError>),
    /// A stored application retained an explicit schema default or otherwise noncanonical record.
    NonCanonicalAttributes {
        /// Scalar operation carrying the record.
        key: ScalarOpKey,
    },
    /// An opaque verified region exposed an internally inconsistent handle.
    InvalidVerifiedRegionHandle(VerifiedIndexHandleError),
    /// The semantic type authority rejected embedded or inferred type data.
    TypeAuthority(Arc<RegistryError>),
    /// The operation-specific inferencer rejected an application.
    Inference(Arc<ScalarApplicationRejection>),
    /// The registry exceeded its governed definition count.
    DefinitionCountLimit {
        /// Attempted definition count.
        actual: usize,
        /// Maximum definition count.
        limit: usize,
    },
    /// A reached-definition projection exceeded its governed byte count.
    ProjectionByteLimit {
        /// Attempted projection bytes.
        actual: usize,
        /// Maximum projection bytes.
        limit: usize,
    },
    /// A reached-definition projection exceeded its governed definition count.
    ProjectionDefinitionCountLimit {
        /// Attempted distinct reached-definition count.
        actual: usize,
        /// Maximum distinct reached-definition count.
        limit: usize,
    },
    /// The registry exceeded its aggregate canonical definition-byte limit.
    RegistryByteLimit {
        /// Attempted aggregate canonical bytes.
        actual: usize,
        /// Maximum aggregate canonical bytes.
        limit: usize,
    },
    /// Revalidation inferred a result type different from the stored structural result.
    RevalidatedResultTypeMismatch {
        /// Scalar operation being revalidated.
        key: ScalarOpKey,
        /// Ordered result position.
        position: usize,
        /// Type stored by the region.
        stored: Arc<ResolvedValueType>,
        /// Type inferred by the selected authority.
        inferred: Arc<ResolvedValueType>,
    },
    /// Revalidation inferred a different number of ordered results.
    RevalidatedResultArity {
        /// Scalar operation being revalidated.
        key: ScalarOpKey,
        /// Result count stored by the region.
        stored: usize,
        /// Result count inferred by the selected authority.
        inferred: usize,
    },
}
impl fmt::Display for ScalarRegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for ScalarRegistryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CanonicalAttributes(source) => Some(source.as_ref()),
            Self::InvalidVerifiedRegionHandle(source) => Some(source),
            Self::TypeAuthority(source) => Some(source.as_ref()),
            Self::Inference(source) => Some(source.as_ref()),
            _ => None,
        }
    }
}

/// Mutable scalar authority composed with an exact semantic type authority.
pub struct ScalarRegistryBuilder {
    semantic: FrozenSemanticRegistry,
    definitions: BTreeMap<ScalarOpKey, RegisteredScalarOperation>,
    canonical_bytes: usize,
}

impl ScalarRegistryBuilder {
    /// Creates an empty builder. Empty snapshots support load/copy-only regions.
    #[must_use]
    pub fn new(semantic: FrozenSemanticRegistry) -> Self {
        Self {
            semantic,
            definitions: BTreeMap::new(),
            canonical_bytes: 0,
        }
    }

    /// Creates the mutable governed standard scalar profile.
    ///
    /// It is composed with [`FrozenSemanticRegistry::standard`] and defines the
    /// exact per-point scalar operations the governed semantic families lower
    /// to: [`constant_f32_scalar_op`], [`multiply_f32_scalar_op`],
    /// [`add_f32_scalar_op`], [`divide_f32_scalar_op`], [`exp_f32_scalar_op`],
    /// [`rsqrt_f32_scalar_op`], [`maximum_f32_scalar_op`],
    /// [`canonicalize_nan_f32_scalar_op`],
    /// [`strict_affine_u4_dequantize_scalar_op`], [`constant_bf16_scalar_op`],
    /// [`multiply_bf16_scalar_op`], and [`add_bf16_scalar_op`]. NaN
    /// canonicalization and the strict-affine decode are conversion operations
    /// rather than homogeneous F32 arithmetic, and the maximum is a selection
    /// rather than either. There is deliberately no number-preferring extrema
    /// sibling beside the maximum and no `minimum-f32` at all: ADR 0023 makes each
    /// a separate operation, and a scalar with no registered semantic operation
    /// above it is one a law could emit into a region nothing means. The `bf16` triple is the complete
    /// per-point vocabulary of the registered `bf16` families and nothing wider:
    /// there is no `bf16` division, elementary function, or NaN canonicalization
    /// here because no `bf16` semantic operation states one. An extension
    /// composes with this profile by registering further definitions on the
    /// returned builder.
    ///
    /// # Errors
    ///
    /// Returns [`ScalarRegistryError`] when the governed semantic authority or a
    /// governed scalar definition violates the same public contract an
    /// extension is held to.
    pub fn standard() -> Result<Self, ScalarRegistryError> {
        let semantic = FrozenSemanticRegistry::standard()
            .map_err(|source| ScalarRegistryError::TypeAuthority(Arc::new(source)))?;
        let mut builder = Self::new(semantic);
        let provider = ProviderIdentity::new("tiler", "standard-scalars", 1)
            .map_err(|source| ScalarRegistryError::TypeAuthority(Arc::new(source)))?;
        builder.register(
            provider.clone(),
            standard_definition(
                constant_f32_scalar_op(),
                "IEEE 754-2019 binary32 constant; tiler.scalar::constant-f32@1",
                constant_attribute_schema()?,
                ScalarArity::exact(0)?,
                constant_f32_facts()?,
                Arc::new(StandardF32Constant),
            )?,
        )?;
        builder.register(
            provider.clone(),
            standard_definition(
                multiply_f32_scalar_op(),
                "IEEE 754-2019 binary32 multiplication; tiler.scalar::multiply-f32@1",
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(2)?,
                arithmetic_f32_scalar_facts()?,
                Arc::new(StandardF32Homogeneous),
            )?,
        )?;
        builder.register(
            provider.clone(),
            standard_definition(
                add_f32_scalar_op(),
                "IEEE 754-2019 binary32 addition; tiler.scalar::add-f32@1",
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(2)?,
                arithmetic_f32_scalar_facts()?,
                Arc::new(StandardF32Homogeneous),
            )?,
        )?;
        builder.register(
            provider.clone(),
            standard_definition(
                divide_f32_scalar_op(),
                "IEEE 754-2019 binary32 division; tiler.scalar::divide-f32@1",
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(2)?,
                arithmetic_f32_scalar_facts()?,
                Arc::new(StandardF32Homogeneous),
            )?,
        )?;
        builder.register(
            provider.clone(),
            standard_definition(
                exp_f32_scalar_op(),
                "the natural exponential over IEEE 754-2019 binary32, precise family; its admitted \
                 result set is the resolved accuracy contract of the semantic operation it \
                 realizes; tiler.scalar::exp-f32@1",
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(1)?,
                elementary_f32_scalar_facts()?,
                Arc::new(StandardF32Homogeneous),
            )?,
        )?;
        // The second precise elementary key. Registering it widens this
        // snapshot's identity and therefore every whole-snapshot provenance
        // derived from it, exactly as the `bf16` triple's arrival did; it leaves
        // reached-only projections alone, so every existing occurrence's
        // executable coverage — and so its kernel-program and artifact identity
        // — stays byte-identical.
        builder.register(
            provider.clone(),
            standard_definition(
                rsqrt_f32_scalar_op(),
                "the reciprocal square root 1/sqrt(t) over IEEE 754-2019 binary32, precise family, \
                 as one operation and deliberately not a square root followed by a division; its \
                 admitted result set is the resolved accuracy contract of the semantic operation \
                 it realizes; tiler.scalar::rsqrt-f32@1",
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(1)?,
                elementary_f32_scalar_facts()?,
                Arc::new(StandardF32Homogeneous),
            )?,
        )?;
        builder.register(
            provider.clone(),
            standard_definition(
                canonicalize_nan_f32_scalar_op(),
                "governed canonical arithmetic-NaN conversion; \
                 tiler.scalar::canonicalize-nan-f32@1",
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(1)?,
                exact_bit_pattern_f32_scalar_facts()?,
                Arc::new(StandardF32Homogeneous),
            )?,
        )?;
        // The per-point extrema key. Like the reciprocal square root before it,
        // registering it widens this snapshot's identity and therefore every
        // whole-snapshot provenance derived from it, and leaves reached-only
        // projections alone — so every existing occurrence's executable coverage,
        // and so its kernel-program and artifact identity, stays byte-identical.
        // `the_landed_one_reader_chain_identities_are_unchanged_byte_for_byte` in
        // `super::law` is that claim pinned over exact bytes.
        builder.register(
            provider.clone(),
            standard_definition(
                maximum_f32_scalar_op(),
                "the NaN-propagating IEEE 754-2019 maximum over binary32, ordering -0.0 below +0.0 \
                 and deliberately not maximumNumber; every non-NaN result is one operand's exact \
                 bit pattern and no value is computed, and a NaN result carries the governed \
                 canonical arithmetic-NaN payload whichever operand was a NaN and whether it was \
                 quiet or signalling; deliberately not Rust's f32::max or Metal's fmax, which are \
                 the number-preferring family; tiler.scalar::maximum-f32@1",
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(2)?,
                exact_bit_pattern_f32_scalar_facts()?,
                Arc::new(StandardF32Homogeneous),
            )?,
        )?;
        builder.register(
            provider.clone(),
            standard_definition(
                strict_affine_u4_dequantize_scalar_op(),
                "strict affine U4-to-F32 decode: widen code and zero point to i32, subtract, \
                 convert exactly to f32, then multiply once under binary32 round-to-nearest-ties-even; \
                 tiler.scalar::strict-affine-u4-dequantize@1",
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(3)?,
                arithmetic_f32_scalar_facts()?,
                Arc::new(StandardStrictAffineU4Dequantize),
            )?,
        )?;
        // The three per-point `bf16` scalars the governed `tiler::constant-bf16@1`,
        // `tiler::multiply-bf16@1`, and `tiler::add-bf16@1` families realize.
        // Registering them widens this snapshot's identity and therefore every
        // whole-snapshot provenance derived from it; it leaves reached-only
        // projections alone, which is what keeps an existing `f32` occurrence's
        // executable coverage — and so its kernel-program and artifact
        // identity — byte-identical.
        let payload_bytes = crate::semantic::registered_bf16_payload_bytes();
        builder.register(
            provider.clone(),
            standard_definition(
                constant_bf16_scalar_op(),
                "exact BF16 constant in the ratified RISC-V BF16 operand format; \
                 tiler.scalar::constant-bf16@1",
                constant_bf16_attribute_schema()?,
                ScalarArity::exact(0)?,
                constant_bf16_scalar_facts()?,
                Arc::new(StandardBf16Constant { payload_bytes }),
            )?,
        )?;
        builder.register(
            provider.clone(),
            standard_definition(
                multiply_bf16_scalar_op(),
                "separate multiplication over the ratified RISC-V BF16 operand format; \
                 tiler.scalar::multiply-bf16@1",
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(2)?,
                arithmetic_bf16_scalar_facts()?,
                Arc::new(StandardBf16Homogeneous),
            )?,
        )?;
        builder.register(
            provider,
            standard_definition(
                add_bf16_scalar_op(),
                "separate addition over the ratified RISC-V BF16 operand format; \
                 tiler.scalar::add-bf16@1",
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(2)?,
                arithmetic_bf16_scalar_facts()?,
                Arc::new(StandardBf16Homogeneous),
            )?,
        )?;
        Ok(builder)
    }
    /// Registers one definition with separate admission provenance.
    ///
    /// # Errors
    ///
    /// Returns an error for duplicate or invalid definitions and unknown embedded types.
    pub fn register(
        &mut self,
        provider: ProviderIdentity,
        definition: ScalarOperationDefinition,
    ) -> Result<(), ScalarRegistryError> {
        let key = definition.key.clone();
        if self.definitions.contains_key(&key) {
            return Err(ScalarRegistryError::DuplicateDefinition { key });
        }
        if self.definitions.len() >= MAX_SCALAR_DEFINITIONS {
            return Err(ScalarRegistryError::DefinitionCountLimit {
                actual: self.definitions.len().saturating_add(1),
                limit: MAX_SCALAR_DEFINITIONS,
            });
        }
        if definition.results.min == 0 {
            return Err(ScalarRegistryError::ZeroResultDefinition);
        }
        for field in definition.attributes.fields() {
            if let Some(default) = field.default() {
                validate_canonical_types(&self.semantic, default)?;
            }
        }
        validate_canonical_types(&self.semantic, &definition.facts)?;
        validate_canonical_types(&self.semantic, &definition.conformance)?;
        let definition_bytes = encoded_definition_len(&definition);
        let actual = self.canonical_bytes.saturating_add(definition_bytes);
        if actual > MAX_SCALAR_REGISTRY_CANONICAL_BYTES {
            return Err(ScalarRegistryError::RegistryByteLimit {
                actual,
                limit: MAX_SCALAR_REGISTRY_CANONICAL_BYTES,
            });
        }
        self.definitions.insert(
            key,
            RegisteredScalarOperation {
                definition,
                provider,
            },
        );
        self.canonical_bytes = actual;
        Ok(())
    }
    /// Freezes this exact snapshot.
    #[must_use]
    pub fn freeze(self) -> FrozenScalarRegistry {
        let snapshot = compute_scalar_snapshot_identity(&self.definitions);
        FrozenScalarRegistry(Arc::new(ScalarRegistryData {
            semantic: self.semantic,
            definitions: self.definitions,
            snapshot,
        }))
    }
}

struct ScalarRegistryData {
    semantic: FrozenSemanticRegistry,
    definitions: BTreeMap<ScalarOpKey, RegisteredScalarOperation>,
    snapshot: CanonicalScalarRegistrySnapshotIdentity,
}

/// Immutable scalar authority. Dynamic callbacks run only while constructing SSA.
#[derive(Clone)]
pub struct FrozenScalarRegistry(Arc<ScalarRegistryData>);

impl fmt::Debug for FrozenScalarRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrozenScalarRegistry")
            .field("definition_count", &self.0.definitions.len())
            .finish()
    }
}

impl FrozenScalarRegistry {
    /// Builds the governed standard scalar profile.
    ///
    /// The snapshot is computed once and shared, so every consumer that lowers
    /// the governed `f32` families binds the same scalar authority instead of
    /// composing an ad-hoc one whose snapshot identity nothing else agrees with.
    ///
    /// # Errors
    ///
    /// Returns [`ScalarRegistryError`] when a governed definition violates the
    /// same public contract used by extensions.
    pub fn standard() -> Result<Self, ScalarRegistryError> {
        static STANDARD: std::sync::OnceLock<Result<FrozenScalarRegistry, ScalarRegistryError>> =
            std::sync::OnceLock::new();
        STANDARD
            .get_or_init(|| Ok(ScalarRegistryBuilder::standard()?.freeze()))
            .clone()
    }

    /// Returns complete scalar-registry snapshot provenance.
    #[must_use]
    pub fn snapshot_identity(&self) -> &CanonicalScalarRegistrySnapshotIdentity {
        &self.0.snapshot
    }

    /// Returns the exact semantic type authority this snapshot is composed with.
    #[must_use]
    pub fn semantic_authority(&self) -> &FrozenSemanticRegistry {
        &self.0.semantic
    }
    pub(super) fn validate_type(
        &self,
        value: &ResolvedValueType,
    ) -> Result<(), ScalarRegistryError> {
        self.0
            .semantic
            .validate_type(value)
            .map_err(|error| ScalarRegistryError::TypeAuthority(Arc::new(error)))
    }

    pub(super) fn minimum_results(&self, key: &ScalarOpKey) -> Result<usize, ScalarRegistryError> {
        self.0
            .definitions
            .get(key)
            .map(|registered| registered.definition.results.min())
            .ok_or_else(|| ScalarRegistryError::UnknownOperation { key: key.clone() })
    }

    pub(super) fn infer(
        &self,
        key: &ScalarOpKey,
        operands: &[ResolvedValueType],
        attributes: &ScalarAttributes,
        capacity: ScalarInferenceCapacity,
    ) -> Result<Vec<ResolvedValueType>, ScalarApplyError> {
        let registered = self
            .0
            .definitions
            .get(key)
            .ok_or_else(|| ScalarRegistryError::UnknownOperation { key: key.clone() })
            .map_err(ScalarApplyError::Authority)?;
        let definition = &registered.definition;
        if !definition.operands.accepts(operands.len()) {
            return Err(ScalarApplyError::Authority(
                ScalarRegistryError::OperandArity {
                    key: key.clone(),
                    actual: operands.len(),
                },
            ));
        }
        let canonical = definition
            .attributes
            .normalize(attributes)
            .map_err(ScalarApplyError::Authority)?;
        let resolved = definition
            .attributes
            .resolve_defaults(&canonical)
            .map_err(ScalarApplyError::Authority)?;
        validate_canonical_types(&self.0.semantic, attributes.value())
            .map_err(ScalarApplyError::Authority)?;
        for operand in operands {
            self.validate_type(operand)
                .map_err(ScalarApplyError::Authority)?;
        }
        let request = ScalarInferenceRequest::new(operands, &resolved);
        let mut outputs = ScalarInferenceOutputs::new(definition.results.max(), capacity);
        let callback = definition.inferencer.infer(request, &mut outputs);
        let results = outputs
            .finish(callback, definition.results.min())
            .map_err(|error| match error {
                ScalarInferenceFinishError::Provider(source) => ScalarApplyError::Authority(
                    ScalarRegistryError::Inference(Arc::new(ScalarApplicationRejection {
                        key: key.clone(),
                        provider: registered.provider.clone(),
                        source,
                    })),
                ),
                ScalarInferenceFinishError::Host(failure) => ScalarApplyError::Host(failure),
            })?;
        for result in &results {
            self.validate_type(result)
                .map_err(ScalarApplyError::Authority)?;
        }
        Ok(results)
    }

    pub(super) fn normalize_attributes(
        &self,
        key: &ScalarOpKey,
        attributes: ScalarAttributes,
    ) -> Result<ScalarAttributes, ScalarRegistryError> {
        let registered = self
            .0
            .definitions
            .get(key)
            .ok_or_else(|| ScalarRegistryError::UnknownOperation { key: key.clone() })?;
        let canonical = registered.definition.attributes.normalize(&attributes);
        drop(attributes);
        canonical
    }

    /// Returns admission provenance for diagnostics; it is not structural IR identity.
    #[must_use]
    pub fn provider(&self, key: &ScalarOpKey) -> Option<&ProviderIdentity> {
        self.0.definitions.get(key).map(|entry| &entry.provider)
    }

    /// Returns one provider-independent scalar definition.
    #[must_use]
    pub fn definition(&self, key: &ScalarOpKey) -> Option<&ScalarOperationDefinition> {
        self.0.definitions.get(key).map(|entry| &entry.definition)
    }

    /// Projects only provider-independent definitions reached by a region.
    ///
    /// # Errors
    ///
    /// Returns an error when any reached operation key is absent from this snapshot.
    ///
    /// # Panics
    ///
    /// Panics only if an internally validated arity cannot be represented as `u64`.
    pub fn project_reached<'a>(
        &self,
        keys: impl IntoIterator<Item = &'a ScalarOpKey>,
    ) -> Result<CanonicalScalarDefinitionProjection, ScalarRegistryError> {
        let mut reached_keys = std::collections::BTreeSet::new();
        for key in keys {
            reached_keys.insert(key.clone());
            if reached_keys.len() > MAX_SCALAR_DEFINITIONS {
                return Err(ScalarRegistryError::ProjectionDefinitionCountLimit {
                    actual: reached_keys.len(),
                    limit: MAX_SCALAR_DEFINITIONS,
                });
            }
        }
        let mut output = b"tiler.scalar-definition-projection.v2\0".to_vec();
        push_len(&mut output, reached_keys.len());
        for key in reached_keys {
            let definition = self
                .definition(&key)
                .ok_or_else(|| ScalarRegistryError::UnknownOperation { key: key.clone() })?;
            let encoded = encode_definition(definition);
            let actual = output.len().saturating_add(encoded.len());
            if actual > MAX_SCALAR_DEFINITION_PROJECTION_BYTES {
                return Err(ScalarRegistryError::ProjectionByteLimit {
                    actual,
                    limit: MAX_SCALAR_DEFINITION_PROJECTION_BYTES,
                });
            }
            output.extend_from_slice(&encoded);
        }
        Ok(CanonicalScalarDefinitionProjection(output))
    }

    /// Revalidates every reached scalar application and binds exact authority evidence to it.
    ///
    /// The returned receipt is separate from structural region identity. It records the selected
    /// definitions and admission providers without changing structural reuse equality.
    ///
    /// # Errors
    ///
    /// Returns an error for missing authority, rejected inference, or any stored/inferred type
    /// disagreement.
    pub fn revalidate_region(
        &self,
        region: &VerifiedIndexRegion,
    ) -> Result<ScalarAuthorityEvidence, ScalarRegistryError> {
        for tensor in region.tensors() {
            self.validate_type(tensor.value_type())?;
        }
        let reached = self.revalidate_region_operations(region)?;
        let definitions = self.project_reached(reached.iter())?;
        let mut value_types = Vec::new();
        value_types.extend(region.tensors().map(super::model::TensorRef::value_type));
        value_types.extend(
            region
                .scalar_values()
                .map(super::model::ScalarValueRef::value_type),
        );
        for operation in region.scalar_operations() {
            if let ScalarOperationKindRef::Reduce(reduction) = operation.kind() {
                value_types.extend(
                    reduction
                        .body()
                        .values()
                        .map(super::model::ReducerBodyValueRef::value_type),
                );
            }
        }
        let canonical_values = self.authority_canonical_values(region, &reached)?;
        let type_authority = self
            .0
            .semantic
            .project_value_set_authority(value_types, canonical_values)
            .map_err(|error| ScalarRegistryError::TypeAuthority(Arc::new(error)))?;
        let mut admission = b"tiler.scalar-admission-provenance.v1\0".to_vec();
        push_len(&mut admission, reached.len());
        for key in &reached {
            encode_key(&mut admission, key);
            let provider = self
                .provider(key)
                .ok_or_else(|| ScalarRegistryError::UnknownOperation { key: key.clone() })?;
            push_slice(&mut admission, provider.namespace().as_bytes());
            push_slice(&mut admission, provider.name().as_bytes());
            admission.extend_from_slice(&provider.revision().to_be_bytes());
        }
        Ok(ScalarAuthorityEvidence {
            region: region.canonical_identity().clone(),
            reached: reached.into_iter().collect(),
            definitions,
            admission: ScalarAdmissionProvenanceIdentity(admission),
            type_definitions: type_authority.reached_definitions().clone(),
            type_admission: type_authority.admission_provenance().clone(),
            semantic_snapshot: type_authority.registry_snapshot().clone(),
            scalar_snapshot: self.snapshot_identity().clone(),
        })
    }

    fn revalidate_region_operations(
        &self,
        region: &VerifiedIndexRegion,
    ) -> Result<std::collections::BTreeSet<ScalarOpKey>, ScalarRegistryError> {
        let mut reached = std::collections::BTreeSet::new();
        for operation in region.scalar_operations() {
            match operation.kind() {
                ScalarOperationKindRef::Apply { key, attributes } => {
                    reached.insert(key.clone());
                    let operands = operation
                        .operands()
                        .map(|id| {
                            region
                                .scalar_value(id)
                                .map(|value| value.value_type().clone())
                                .map_err(ScalarRegistryError::InvalidVerifiedRegionHandle)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let stored = operation
                        .results()
                        .map(|id| {
                            region
                                .scalar_value(id)
                                .map(|value| value.value_type().clone())
                                .map_err(ScalarRegistryError::InvalidVerifiedRegionHandle)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    self.revalidate_application(key, attributes, &operands, &stored)?;
                }
                ScalarOperationKindRef::Reduce(reduction) => {
                    let body = reduction.body();
                    for application in body.operations() {
                        reached.insert(application.key().clone());
                        let operands = application
                            .operands()
                            .map(|id| {
                                region
                                    .reducer_body_value(id)
                                    .map(|value| value.value_type().clone())
                                    .map_err(ScalarRegistryError::InvalidVerifiedRegionHandle)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let stored = application
                            .results()
                            .map(|id| {
                                region
                                    .reducer_body_value(id)
                                    .map(|value| value.value_type().clone())
                                    .map_err(ScalarRegistryError::InvalidVerifiedRegionHandle)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        self.revalidate_application(
                            application.key(),
                            application.attributes(),
                            &operands,
                            &stored,
                        )?;
                    }
                }
            }
        }
        Ok(reached)
    }

    fn authority_canonical_values<'a>(
        &'a self,
        region: &'a VerifiedIndexRegion,
        reached: &'a std::collections::BTreeSet<ScalarOpKey>,
    ) -> Result<Vec<&'a CanonicalValue>, ScalarRegistryError> {
        let mut canonical_values = Vec::new();
        for key in reached {
            let definition = self
                .definition(key)
                .ok_or_else(|| ScalarRegistryError::UnknownOperation { key: key.clone() })?;
            canonical_values.extend(
                definition
                    .attributes()
                    .fields()
                    .iter()
                    .filter_map(ScalarAttributeField::default),
            );
            canonical_values.push(definition.facts());
            canonical_values.push(definition.conformance());
        }
        for operation in region.scalar_operations() {
            match operation.kind() {
                ScalarOperationKindRef::Apply { attributes, .. } => {
                    canonical_values.push(attributes.value());
                }
                ScalarOperationKindRef::Reduce(reduction) => {
                    canonical_values.extend(
                        reduction
                            .body()
                            .operations()
                            .map(|operation| operation.attributes().value()),
                    );
                }
            }
        }
        Ok(canonical_values)
    }

    fn revalidate_application(
        &self,
        key: &ScalarOpKey,
        attributes: &ScalarAttributes,
        operands: &[ResolvedValueType],
        stored: &[ResolvedValueType],
    ) -> Result<(), ScalarRegistryError> {
        let canonical = self.normalize_attributes(key, attributes.clone())?;
        if &canonical != attributes {
            return Err(ScalarRegistryError::NonCanonicalAttributes { key: key.clone() });
        }
        let inferred = self
            .infer(
                key,
                operands,
                attributes,
                ScalarInferenceCapacity {
                    result_slots: MAX_SCALAR_ARITY,
                    result_count_before: 0,
                    result_limit: MAX_SCALAR_ARITY,
                    retained_bytes: usize::MAX,
                    retained_bytes_before: 0,
                    retained_byte_limit: usize::MAX,
                    per_result_overhead: 0,
                    byte_multiplier: 1,
                },
            )
            .map_err(|error| match error {
                ScalarApplyError::Authority(error) => error,
                ScalarApplyError::Host(_) => {
                    unreachable!("unbounded revalidation capacity cannot be exhausted")
                }
            })?;
        if stored.len() != inferred.len() {
            return Err(ScalarRegistryError::RevalidatedResultArity {
                key: key.clone(),
                stored: stored.len(),
                inferred: inferred.len(),
            });
        }
        for (position, (stored, inferred)) in stored.iter().zip(&inferred).enumerate() {
            if stored != inferred {
                return Err(ScalarRegistryError::RevalidatedResultTypeMismatch {
                    key: key.clone(),
                    position,
                    stored: Arc::new(stored.clone()),
                    inferred: Arc::new(inferred.clone()),
                });
            }
        }
        Ok(())
    }
}

fn encode_definition(definition: &ScalarOperationDefinition) -> Vec<u8> {
    let exact_capacity = encoded_definition_len(definition);
    let mut encoded = Vec::with_capacity(exact_capacity);
    encode_key(&mut encoded, &definition.key);
    push_slice(
        &mut encoded,
        definition.normative_definition.as_str().as_bytes(),
    );
    encoded.push(match definition.effect {
        ScalarEffect::Pure => 1,
    });
    push_len(&mut encoded, definition.operands.min);
    push_len(&mut encoded, definition.operands.max);
    push_len(&mut encoded, definition.results.min);
    push_len(&mut encoded, definition.results.max);
    push_len(&mut encoded, definition.attributes.0.len());
    for field in &definition.attributes.0 {
        encoded.extend_from_slice(&field.id.get().to_be_bytes());
        encoded.push(canonical_kind_tag(field.kind));
        encoded.push(match (&field.default, field.required) {
            (None, true) => 1,
            (None, false) => 2,
            (Some(_), false) => 3,
            (Some(_), true) => unreachable!("required fields cannot carry defaults"),
        });
        if let Some(default) = &field.default {
            encode_canonical(&mut encoded, default);
        }
    }
    encode_canonical(&mut encoded, &definition.facts);
    encode_canonical(&mut encoded, &definition.conformance);
    debug_assert_eq!(encoded.len(), exact_capacity);
    encoded
}

fn encoded_definition_len(definition: &ScalarOperationDefinition) -> usize {
    let mut bytes = encoded_key_len(&definition.key)
        .saturating_add(encoded_bytes_len(
            definition.normative_definition.as_str().len(),
        ))
        .saturating_add(1)
        .saturating_add(4 * std::mem::size_of::<u64>())
        .saturating_add(std::mem::size_of::<u64>());
    for field in &definition.attributes.0 {
        bytes = bytes.saturating_add(std::mem::size_of::<u32>() + 2);
        if let Some(default) = &field.default {
            bytes = bytes.saturating_add(default.encoded_len());
        }
    }
    bytes
        .saturating_add(definition.facts.encoded_len())
        .saturating_add(definition.conformance.encoded_len())
}

fn encoded_key_len(key: &ScalarOpKey) -> usize {
    encoded_bytes_len(key.namespace().len())
        .saturating_add(encoded_bytes_len(key.name().len()))
        .saturating_add(std::mem::size_of::<u32>())
}

const fn encoded_bytes_len(bytes: usize) -> usize {
    std::mem::size_of::<u64>().saturating_add(bytes)
}

fn compute_scalar_snapshot_identity(
    definitions: &BTreeMap<ScalarOpKey, RegisteredScalarOperation>,
) -> CanonicalScalarRegistrySnapshotIdentity {
    let mut bytes = b"tiler.scalar-registry-snapshot.v1\0".to_vec();
    push_len(&mut bytes, definitions.len());
    for (key, registered) in definitions {
        encode_key(&mut bytes, key);
        push_slice(&mut bytes, &encode_definition(&registered.definition));
        push_slice(&mut bytes, registered.provider.namespace().as_bytes());
        push_slice(&mut bytes, registered.provider.name().as_bytes());
        bytes.extend_from_slice(&registered.provider.revision().to_be_bytes());
    }
    CanonicalScalarRegistrySnapshotIdentity(bytes)
}

/// Canonical provider-independent projection of reached scalar definitions.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalScalarDefinitionProjection(Vec<u8>);
impl CanonicalScalarDefinitionProjection {
    /// Returns collision-free projection bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Provider-attributed scalar admission provenance for one reached operation set.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ScalarAdmissionProvenanceIdentity(Vec<u8>);
impl ScalarAdmissionProvenanceIdentity {
    /// Returns collision-free provenance bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Complete provider-attributed frozen scalar-registry snapshot identity.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalScalarRegistrySnapshotIdentity(Vec<u8>);
impl CanonicalScalarRegistrySnapshotIdentity {
    /// Returns collision-free snapshot bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Checked scalar authority evidence bound to one exact structural region.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScalarAuthorityEvidence {
    region: CanonicalIndexRegionIdentity,
    reached: Vec<ScalarOpKey>,
    definitions: CanonicalScalarDefinitionProjection,
    admission: ScalarAdmissionProvenanceIdentity,
    type_definitions: SemanticDefinitionProjectionIdentity,
    type_admission: SemanticAdmissionProvenanceIdentity,
    semantic_snapshot: SemanticRegistrySnapshotIdentity,
    scalar_snapshot: CanonicalScalarRegistrySnapshotIdentity,
}
impl ScalarAuthorityEvidence {
    /// Returns the structural region identity this evidence revalidated.
    #[must_use]
    pub const fn region(&self) -> &CanonicalIndexRegionIdentity {
        &self.region
    }
    /// Returns the distinct scalar operations the region reached, in key order.
    ///
    /// The projection is the canonical identity contribution; these are the
    /// exact keys behind it, so an authority that must decide whether a region
    /// stayed inside a declared permission can compare sets instead of comparing
    /// opaque bytes for equality.
    #[must_use]
    pub fn reached_operations(&self) -> &[ScalarOpKey] {
        &self.reached
    }

    /// Returns reached provider-independent definitions.
    #[must_use]
    pub const fn definitions(&self) -> &CanonicalScalarDefinitionProjection {
        &self.definitions
    }
    /// Returns reached provider-attributed admission provenance.
    #[must_use]
    pub const fn admission(&self) -> &ScalarAdmissionProvenanceIdentity {
        &self.admission
    }
    /// Returns reached provider-independent semantic type definitions.
    #[must_use]
    pub const fn type_definitions(&self) -> &SemanticDefinitionProjectionIdentity {
        &self.type_definitions
    }
    /// Returns provider-attributed semantic type-admission provenance.
    #[must_use]
    pub const fn type_admission(&self) -> &SemanticAdmissionProvenanceIdentity {
        &self.type_admission
    }
    /// Returns the complete semantic registry snapshot provenance.
    #[must_use]
    pub const fn semantic_snapshot(&self) -> &SemanticRegistrySnapshotIdentity {
        &self.semantic_snapshot
    }
    /// Returns the complete scalar registry snapshot provenance.
    #[must_use]
    pub const fn scalar_snapshot(&self) -> &CanonicalScalarRegistrySnapshotIdentity {
        &self.scalar_snapshot
    }
}

fn validate_canonical_types(
    registry: &FrozenSemanticRegistry,
    value: &CanonicalValue,
) -> Result<(), ScalarRegistryError> {
    match value.view() {
        CanonicalValueView::Type(value_type) => registry
            .validate_type(value_type)
            .map_err(|error| ScalarRegistryError::TypeAuthority(Arc::new(error)))?,
        CanonicalValueView::FloatBits(value) => registry
            .validate_type(&ResolvedValueType::nominal(value.format().clone()))
            .map_err(|error| ScalarRegistryError::TypeAuthority(Arc::new(error)))?,
        CanonicalValueView::Sequence(values) => {
            for value in values {
                validate_canonical_types(registry, value)?;
            }
        }
        CanonicalValueView::Record(fields) => {
            for field in fields {
                validate_canonical_types(registry, field.value())?;
            }
        }
        CanonicalValueView::Bool(_)
        | CanonicalValueView::Signed { .. }
        | CanonicalValueView::Unsigned { .. }
        | CanonicalValueView::Bytes(_)
        | CanonicalValueView::Utf8(_) => {}
    }
    Ok(())
}

fn canonical_kind_tag(kind: CanonicalValueKind) -> u8 {
    match kind {
        CanonicalValueKind::Type => 1,
        CanonicalValueKind::Bool => 2,
        CanonicalValueKind::Signed => 3,
        CanonicalValueKind::Unsigned => 4,
        CanonicalValueKind::FloatBits => 5,
        CanonicalValueKind::Bytes => 6,
        CanonicalValueKind::Utf8 => 7,
        CanonicalValueKind::Sequence => 8,
        CanonicalValueKind::Record => 9,
    }
}

pub(super) fn encode_key(output: &mut Vec<u8>, key: &ScalarOpKey) {
    push_slice(output, key.namespace().as_bytes());
    push_slice(output, key.name().as_bytes());
    output.extend_from_slice(&key.semantic_version().to_be_bytes());
}
pub(super) fn encode_canonical(output: &mut Vec<u8>, value: &CanonicalValue) {
    value.encode(output);
}

#[cfg(test)]
mod governed_fact_tests {
    use super::{
        CANONICAL_ARITHMETIC_NAN_PROFILE, DECLARED_PAYLOAD_PRESERVED, FrozenScalarRegistry,
        SCALAR_FACT_CANONICAL_NAN_BITS, SCALAR_FACT_CONTRACTION_PERMITTED,
        SCALAR_FACT_NAN_RESULT_RULE, ScalarOpKey, add_f32_scalar_op,
        canonicalize_nan_f32_scalar_op, constant_f32_scalar_op, multiply_f32_scalar_op,
    };
    use crate::semantic::{
        AttributeFieldId, CanonicalValue, CanonicalValueView, FrozenSemanticRegistry, OpKey,
        add_f32_op, constant_f32_op, multiply_f32_op,
    };

    fn field(value: &CanonicalValue, id: AttributeFieldId) -> Option<CanonicalValueView<'_>> {
        let CanonicalValueView::Record(fields) = value.view() else {
            panic!("a governed facts record is a canonical record");
        };
        fields
            .iter()
            .find(|field| field.id() == id)
            .map(|field| field.value().view())
    }

    /// Returns every exact binary32 payload a facts record declares.
    ///
    /// The two layers number their fact fields independently, so this collects
    /// by value category rather than by field ID: the claim under test is that
    /// the declared *payload* agrees, not that the records happen to be shaped
    /// alike.
    fn declared_float_payloads(value: &CanonicalValue) -> Vec<Vec<u8>> {
        let CanonicalValueView::Record(fields) = value.view() else {
            panic!("a governed facts record is a canonical record");
        };
        fields
            .iter()
            .filter_map(|field| match field.value().view() {
                CanonicalValueView::FloatBits(bits) => Some(bits.bits().to_vec()),
                _ => None,
            })
            .collect()
    }

    fn utf8(view: CanonicalValueView<'_>) -> String {
        match view {
            CanonicalValueView::Utf8(value) => value.to_owned(),
            other => panic!("expected a utf8 fact, found {other:?}"),
        }
    }

    /// Every governed scalar definition states which NaN payload its result
    /// carries, and names a conformance revision an implementation can claim.
    ///
    /// This is the clause that makes the authority self-contained: before it,
    /// a second reference capability or a third-party index-access lowering
    /// provider had nothing in the scalar authority to conform to.
    #[test]
    fn every_governed_scalar_states_a_nan_rule_and_a_conformance_identity() {
        let registry = FrozenScalarRegistry::standard().expect("the governed profile composes");
        for key in [
            constant_f32_scalar_op(),
            multiply_f32_scalar_op(),
            add_f32_scalar_op(),
            canonicalize_nan_f32_scalar_op(),
            super::maximum_f32_scalar_op(),
        ] {
            let definition = registry
                .definition(&key)
                .expect("the definition is governed");
            let rule = field(definition.facts(), super::SCALAR_FACT_NAN_RESULT_RULE)
                .unwrap_or_else(|| panic!("{} states no NaN-result rule", key.name()));
            assert!(
                matches!(rule, CanonicalValueView::Utf8(_)),
                "{} must name its NaN-result rule",
                key.name()
            );
            assert_eq!(
                field(definition.conformance(), AttributeFieldId::new(1))
                    .map(utf8)
                    .as_deref(),
                Some(format!("tiler.scalar.conformance.{}", key.name()).as_str()),
                "{} must name a scalar-scoped conformance identity",
                key.name()
            );
        }
    }

    /// The published fact-field vocabulary reads the records it names.
    ///
    /// **Without this the constants are an assertion about the records rather
    /// than a fact about them.** An out-of-crate reference capability reads
    /// facts through exactly these identifiers, so a constant naming the wrong
    /// field would compile, publish, and mislead every consumer that trusted it.
    #[test]
    fn the_published_fact_fields_read_the_governed_records_at_both_layers() {
        let registry = FrozenScalarRegistry::standard().expect("the governed profile composes");
        for key in [multiply_f32_scalar_op(), add_f32_scalar_op()] {
            let definition = registry
                .definition(&key)
                .expect("the governed scalar is registered");
            assert!(
                field(definition.facts(), super::SCALAR_FACT_ROUNDING).is_some(),
                "{} states its rounding rule in the published field",
                key.name(),
            );
            assert!(
                field(definition.facts(), super::SCALAR_FACT_NAN_RESULT_RULE).is_some(),
                "{} states its NaN-result rule in the published field",
                key.name(),
            );
            assert!(
                field(definition.facts(), super::SCALAR_FACT_CONTRACTION_PERMITTED).is_some(),
                "an arithmetic scalar states contraction in the published field",
            );
        }

        // Absence carries meaning only where stated. Contraction is defined
        // over a pattern of arithmetic operations, so a constant is not a
        // participant and omits the field rather than asserting `false` — which
        // would answer a question the numerical contract does not pose.
        let constant = registry
            .definition(&constant_f32_scalar_op())
            .expect("the governed constant scalar is registered");
        assert!(
            field(constant.facts(), super::SCALAR_FACT_CONTRACTION_PERMITTED).is_none(),
            "a constant is not a contraction participant and must omit the field",
        );

        // The semantic half, now that `FrozenSemanticRegistry` offers a read
        // path. Both layers are exercised in one test because the published
        // vocabularies are only meaningful together: a constant proven at its
        // construction site and never at a read site is an assertion about the
        // record rather than a fact about it.
        let semantic = registry.semantic_authority();
        let multiply = semantic
            .operation_facts(&crate::semantic::multiply_f32_op())
            .expect("the governed multiply is registered");
        assert_eq!(
            field(
                multiply.value(),
                crate::semantic::ARITHMETIC_F32_FACT_ROUNDING
            )
            .map(utf8)
            .as_deref(),
            Some("binary32-round-to-nearest-ties-even"),
        );
        assert!(
            matches!(
                field(
                    multiply.value(),
                    crate::semantic::ARITHMETIC_F32_FACT_CONTRACTION_PERMITTED
                ),
                Some(CanonicalValueView::Bool(false))
            ),
            "the governed multiply is separate, so contraction is stated false",
        );
        let sum = semantic
            .operation_facts(&crate::semantic::strict_serial_sum_f32_op())
            .expect("the governed reduction is registered");
        assert_eq!(
            field(sum.value(), crate::semantic::SERIAL_SUM_F32_FACT_FOLD_ORDER)
                .map(utf8)
                .as_deref(),
            Some("strict-left-fold"),
        );

        // Record-local numbering, and this is the pair that proves it: the
        // semantic arithmetic record spells contraction as its field 3 and this
        // one as field 4. Reading either number against the other record
        // answers a different question, which is why nothing normalizes them.
        assert_ne!(
            super::SCALAR_FACT_CONTRACTION_PERMITTED,
            crate::semantic::ARITHMETIC_F32_FACT_CONTRACTION_PERMITTED,
            "the two layers number the same concept differently, deliberately",
        );
    }

    /// A canonicalizing scalar declares the payload it installs; the preserving
    /// one declares that it installs none.
    ///
    /// The preserving case is asserted beside its canonicalizing neighbours so
    /// the absent payload is evidence about `constant-f32` specifically, rather
    /// than a check that never fires.
    #[test]
    fn only_the_canonicalizing_scalars_declare_a_payload() {
        let registry = FrozenScalarRegistry::standard().expect("the governed profile composes");
        let canonical =
            crate::semantic::canonical_f32_bits(crate::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS);
        let expected = declared_float_payloads(
            &CanonicalValue::record([crate::semantic::CanonicalField::new(
                AttributeFieldId::new(1),
                canonical,
            )])
            .expect("a one-field record is canonical"),
        );

        for key in [
            multiply_f32_scalar_op(),
            add_f32_scalar_op(),
            canonicalize_nan_f32_scalar_op(),
            super::maximum_f32_scalar_op(),
        ] {
            let facts = registry
                .definition(&key)
                .expect("the definition is governed")
                .facts();
            assert_eq!(
                field(facts, SCALAR_FACT_NAN_RESULT_RULE)
                    .map(utf8)
                    .as_deref(),
                Some(CANONICAL_ARITHMETIC_NAN_PROFILE),
                "{} must name the canonical arithmetic-NaN profile",
                key.name()
            );
            assert_eq!(
                declared_float_payloads(facts),
                expected,
                "{} must declare the exact payload it installs",
                key.name()
            );
            assert!(
                field(facts, SCALAR_FACT_CANONICAL_NAN_BITS).is_some(),
                "{} must carry its payload in the governed field",
                key.name()
            );
        }

        let constant = registry
            .definition(&constant_f32_scalar_op())
            .expect("the definition is governed")
            .facts();
        assert_eq!(
            field(constant, SCALAR_FACT_NAN_RESULT_RULE)
                .map(utf8)
                .as_deref(),
            Some(DECLARED_PAYLOAD_PRESERVED),
            "the governed constant must state preservation as a positive rule"
        );
        assert!(
            declared_float_payloads(constant).is_empty(),
            "the governed constant installs no payload, so it declares none"
        );
    }

    /// Only the arithmetic scalars answer the contraction question.
    ///
    /// A constant and a conversion are not participants in an arithmetic
    /// pattern, so declaring `false` there would answer a question the
    /// numerical contract does not pose about them.
    #[test]
    fn contraction_is_stated_exactly_where_it_is_defined() {
        let registry = FrozenScalarRegistry::standard().expect("the governed profile composes");
        let stated = |key: &ScalarOpKey| {
            field(
                registry
                    .definition(key)
                    .expect("the definition is governed")
                    .facts(),
                SCALAR_FACT_CONTRACTION_PERMITTED,
            )
            .map(|view| match view {
                CanonicalValueView::Bool(permitted) => permitted,
                other => panic!("the contraction fact is a boolean, found {other:?}"),
            })
        };
        for key in [multiply_f32_scalar_op(), add_f32_scalar_op()] {
            assert_eq!(
                stated(&key),
                Some(false),
                "{} must forbid contraction explicitly",
                key.name()
            );
        }
        for key in [
            constant_f32_scalar_op(),
            canonicalize_nan_f32_scalar_op(),
            super::maximum_f32_scalar_op(),
        ] {
            assert!(
                stated(&key).is_none(),
                "{} is not an arithmetic contraction participant",
                key.name()
            );
        }
    }

    /// The scalar and semantic layers agree on the canonical NaN payload.
    ///
    /// The records are written independently — see the module's
    /// `standard_scalar_conformance` note for why they are not derived from one
    /// another — so this is the check that keeps them from drifting apart.
    #[test]
    fn scalar_and_semantic_facts_agree_on_the_canonical_payload() {
        let scalars = FrozenScalarRegistry::standard().expect("the governed profile composes");
        let semantic = FrozenSemanticRegistry::standard().expect("the governed authority composes");
        for (scalar_key, semantic_key) in [
            (multiply_f32_scalar_op(), multiply_f32_op()),
            (add_f32_scalar_op(), add_f32_op()),
        ] {
            let scalar = declared_float_payloads(
                scalars
                    .definition(&scalar_key)
                    .expect("the scalar definition is governed")
                    .facts(),
            );
            let tensor = declared_float_payloads(
                semantic
                    .operation_definition(&semantic_key)
                    .expect("the semantic definition is governed")
                    .canonical_facts()
                    .value(),
            );
            assert_eq!(
                scalar,
                tensor,
                "{} and its semantic counterpart must declare one payload",
                scalar_key.name()
            );
        }

        // The preserving pair agrees too: neither layer declares a payload for
        // the constant, so the agreement covers the negative case rather than
        // only the operations that canonicalize.
        assert!(
            declared_float_payloads(
                scalars
                    .definition(&constant_f32_scalar_op())
                    .expect("the scalar definition is governed")
                    .facts()
            )
            .is_empty()
        );
        assert!(
            declared_float_payloads(
                semantic
                    .operation_definition(&constant_f32_op())
                    .expect("the semantic definition is governed")
                    .canonical_facts()
                    .value()
            )
            .is_empty()
        );
    }

    /// The governed conversion has no semantic counterpart to derive from.
    ///
    /// This is the fact that decides the restate-and-check design: a rule that
    /// copied each scalar's facts from its semantic operation would have no
    /// source for this one, which exists because a reduction's *result
    /// boundary* must canonicalize where no combine necessarily ran.
    #[test]
    fn the_canonical_nan_conversion_has_no_semantic_counterpart() {
        let semantic = FrozenSemanticRegistry::standard().expect("the governed authority composes");
        assert!(
            semantic
                .operation_definition(
                    &OpKey::new("tiler", "canonicalize-nan-f32", 1).expect("the key is valid")
                )
                .is_none()
        );
        let scalars = FrozenScalarRegistry::standard().expect("the governed profile composes");
        assert!(
            scalars
                .definition(&canonicalize_nan_f32_scalar_op())
                .is_some()
        );
    }

    /// The two layers' conformance identities are distinct for the same name.
    ///
    /// They govern different contracts — one a whole-tensor operation family,
    /// the other a per-point scalar — so a shared identity string would give
    /// two subjects one identity.
    #[test]
    fn scalar_conformance_is_domain_separated_from_semantic_conformance() {
        let scalars = FrozenScalarRegistry::standard().expect("the governed profile composes");
        let semantic = FrozenSemanticRegistry::standard().expect("the governed authority composes");
        let scalar = scalars
            .definition(&add_f32_scalar_op())
            .expect("the scalar definition is governed")
            .conformance();
        let tensor = semantic
            .operation_definition(&add_f32_op())
            .expect("the semantic definition is governed")
            .conformance()
            .value();
        assert_ne!(scalar, tensor);
        assert_eq!(
            field(scalar, AttributeFieldId::new(1)).map(utf8).as_deref(),
            Some("tiler.scalar.conformance.add-f32")
        );
        assert_eq!(
            field(tensor, AttributeFieldId::new(1)).map(utf8).as_deref(),
            Some("tiler.conformance.add-f32")
        );
    }

    /// The reciprocal square root is registered and shares the elementary fact
    /// record with the exponential.
    ///
    /// **The equality is the claim, not a convenience.** The two keys state one
    /// record because the three fields say the same thing about both: neither
    /// rounds under a rule this layer can name, both install the canonical
    /// arithmetic NaN, and neither is an arithmetic-contraction participant.
    /// What separates them is the resolved accuracy contract their *operations*
    /// state — `BoundedPiecewise` at twelve ULP for `tiler::silu-f32@1`'s
    /// exponential, `Faithful` for `tiler::rms-norm-f32@1`'s reciprocal square
    /// root — and that is a different layer's authority. Asserting the equality
    /// here is what makes a later divergence deliberate rather than accidental,
    /// exactly as `the_two_exponential_tolerances_agree_because_the_derivation_is_one`
    /// does for the tolerances one layer up.
    #[test]
    fn the_reciprocal_square_root_shares_the_elementary_fact_record() {
        let registry = FrozenScalarRegistry::standard().expect("the governed profile composes");
        let rsqrt = registry
            .definition(&super::rsqrt_f32_scalar_op())
            .expect("the governed reciprocal square root is registered");
        let exp = registry
            .definition(&super::exp_f32_scalar_op())
            .expect("the governed exponential is registered");
        assert_eq!(rsqrt.facts(), exp.facts());

        // The rounding field names the deferral rather than a rule, which is
        // the whole reason the record can be shared: a key that stated
        // "round-to-nearest ties-to-even" here would claim a correctly rounded
        // reciprocal square root that nothing establishes.
        assert_eq!(
            field(rsqrt.facts(), super::SCALAR_FACT_ROUNDING)
                .map(utf8)
                .as_deref(),
            Some("resolved-by-the-operation-accuracy-contract"),
        );
        assert_eq!(
            field(rsqrt.facts(), SCALAR_FACT_NAN_RESULT_RULE)
                .map(utf8)
                .as_deref(),
            Some(CANONICAL_ARITHMETIC_NAN_PROFILE),
        );
        assert!(
            field(rsqrt.facts(), SCALAR_FACT_CONTRACTION_PERMITTED).is_none(),
            "an elementary function has no adjacent product to fuse into",
        );
        assert_eq!(rsqrt.operands().min(), 1);
        assert_eq!(rsqrt.operands().max(), 1);
        assert_eq!(
            field(rsqrt.conformance(), AttributeFieldId::new(1))
                .map(utf8)
                .as_deref(),
            Some("tiler.scalar.conformance.rsqrt-f32"),
        );

        // The two keys are nevertheless distinct definitions: a shared fact
        // record must not make them one row, or the projection a region derives
        // its identity from could not tell an exponential from a reciprocal
        // square root.
        assert_ne!(super::rsqrt_f32_scalar_op(), super::exp_f32_scalar_op());
        assert_ne!(
            registry
                .project_reached([&super::rsqrt_f32_scalar_op()])
                .expect("the governed reciprocal square root projects")
                .as_bytes(),
            registry
                .project_reached([&super::exp_f32_scalar_op()])
                .expect("the governed exponential projects")
                .as_bytes(),
        );
    }

    /// The reciprocal square root refuses the two applications a homogeneous unary
    /// elementary scalar has no meaning for.
    ///
    /// Both perturbations were observed failing before the assertions were
    /// written: a `bf16` operand and a second operand each reach a different
    /// refusal, so neither check is satisfied by the other one firing.
    #[test]
    fn the_reciprocal_square_root_refuses_a_foreign_operand_and_a_second_one() {
        let registry = FrozenScalarRegistry::standard().expect("the governed profile composes");
        let key = super::rsqrt_f32_scalar_op();
        let f32_type = crate::semantic::F32::resolved_type();
        let capacity = super::ScalarInferenceCapacity {
            result_slots: 1,
            result_count_before: 0,
            result_limit: 1,
            retained_bytes: usize::MAX,
            retained_bytes_before: 0,
            retained_byte_limit: usize::MAX,
            per_result_overhead: 0,
            byte_multiplier: 1,
        };
        let attributes = super::ScalarAttributes::empty();

        let inferred = registry
            .infer(&key, std::slice::from_ref(&f32_type), &attributes, capacity)
            .expect("a unary f32 application is admitted");
        assert_eq!(inferred, vec![f32_type.clone()]);

        // A `bf16` operand: this family declares no mixed precision and no
        // implicit promotion, so it is rejected rather than resolved to the
        // operand's own type.
        let foreign = registry
            .infer(
                &key,
                &[crate::semantic::Bf16::resolved_type()],
                &attributes,
                capacity,
            )
            .expect_err("a bf16 operand is not an f32 one");
        let super::ScalarApplyError::Authority(super::ScalarRegistryError::Inference(rejection)) =
            foreign
        else {
            panic!("a foreign operand must be the inferencer's refusal, not a host one");
        };
        assert_eq!(rejection.key(), &key);
        assert_eq!(
            rejection.rejection().code(),
            &crate::semantic::ProviderDiagnosticCode::new("tiler.scalar.operand-type")
                .expect("the governed diagnostic code is valid"),
        );

        // A second operand: the reciprocal square root is unary, and an arity
        // the definition does not admit is refused by the registered contract
        // before the inferencer runs at all.
        let arity = registry
            .infer(&key, &[f32_type.clone(), f32_type], &attributes, capacity)
            .expect_err("the reciprocal square root admits exactly one operand");
        assert!(
            matches!(
                arity,
                super::ScalarApplyError::Authority(super::ScalarRegistryError::OperandArity {
                    actual: 2,
                    ..
                })
            ),
            "a second operand must be an arity refusal; observed {arity:?}",
        );
    }

    /// The maximum is registered and shares the exact-bit-pattern fact record
    /// with the NaN canonicalization.
    ///
    /// **The equality is the claim.** Both operations select rather than compute:
    /// each reproduces an operand's binary32 pattern verbatim on every non-NaN
    /// input and installs the governed canonical arithmetic NaN for a NaN result,
    /// so the three fields say the same thing about both. What separates them is
    /// arity and their registered normative definitions, which is a different
    /// part of the definition — asserted below so a shared record cannot make the
    /// two one row. This mirrors
    /// `the_reciprocal_square_root_shares_the_elementary_fact_record` exactly.
    ///
    /// The rule assertion is the one worth reading twice: the key names
    /// [`CANONICAL_ARITHMETIC_NAN_PROFILE`] rather than a third vocabulary value,
    /// because ADR 0023's Decision and `docs/numerical-semantics.md`'s "Min and
    /// max" clause both put the extrema families' NaN results under the canonical
    /// arithmetic-NaN contract, and both delivered realizations agree.
    #[test]
    fn the_maximum_shares_the_exact_bit_pattern_fact_record() {
        let registry = FrozenScalarRegistry::standard().expect("the governed profile composes");
        let maximum = registry
            .definition(&super::maximum_f32_scalar_op())
            .expect("the governed maximum is registered");
        let canonicalize = registry
            .definition(&canonicalize_nan_f32_scalar_op())
            .expect("the governed NaN canonicalization is registered");
        assert_eq!(maximum.facts(), canonicalize.facts());

        // It rounds nothing, because it computes nothing: the rounding field is
        // the exact-bits one the constant and the conversion state, never the
        // arithmetic rule and never the elementary functions' deferral.
        assert_eq!(
            field(maximum.facts(), super::SCALAR_FACT_ROUNDING)
                .map(utf8)
                .as_deref(),
            Some("exact-binary32-bits"),
        );
        assert_eq!(
            field(maximum.facts(), SCALAR_FACT_NAN_RESULT_RULE)
                .map(utf8)
                .as_deref(),
            Some(CANONICAL_ARITHMETIC_NAN_PROFILE),
        );
        assert!(
            field(maximum.facts(), SCALAR_FACT_CANONICAL_NAN_BITS).is_some(),
            "an operation naming the profile declares the payload it installs",
        );
        assert!(
            field(maximum.facts(), SCALAR_FACT_CONTRACTION_PERMITTED).is_none(),
            "a selection is not an arithmetic-contraction participant",
        );

        // Binary, where the conversion is unary, and separately named.
        assert_eq!(maximum.operands().min(), 2);
        assert_eq!(maximum.operands().max(), 2);
        assert_eq!(
            field(maximum.conformance(), AttributeFieldId::new(1))
                .map(utf8)
                .as_deref(),
            Some("tiler.scalar.conformance.maximum-f32"),
        );

        // The registered normative definition is where the family, the zero
        // ordering, and the excluded host and backend spellings are pinned. A
        // shared fact record leaves this the only place they are stated, so it is
        // asserted rather than assumed — and it is part of the encoded
        // definition, so a projection separates the two keys on it.
        let normative = maximum.normative_definition().as_str();
        for clause in [
            "NaN-propagating",
            "ordering -0.0 below +0.0",
            "deliberately not maximumNumber",
            "f32::max",
            "fmax",
        ] {
            assert!(
                normative.contains(clause),
                "the maximum's normative definition must pin {clause:?}",
            );
        }
        assert_ne!(
            super::maximum_f32_scalar_op(),
            canonicalize_nan_f32_scalar_op()
        );
        assert_ne!(
            registry
                .project_reached([&super::maximum_f32_scalar_op()])
                .expect("the governed maximum projects")
                .as_bytes(),
            registry
                .project_reached([&canonicalize_nan_f32_scalar_op()])
                .expect("the governed canonicalization projects")
                .as_bytes(),
        );
    }

    /// The maximum refuses the applications a homogeneous binary `f32` scalar has
    /// no meaning for.
    ///
    /// Both perturbations were observed failing before the assertions were
    /// written, and they reach different refusals: a `bf16` operand is the
    /// inferencer's, and a third operand is the registered contract's before the
    /// inferencer runs. A mixed pair is asserted beside the uniform foreign pair
    /// because this key is the first *binary* one whose operands could disagree
    /// with each other rather than only with `f32`.
    #[test]
    fn the_maximum_refuses_a_foreign_operand_a_mixed_pair_and_a_third_operand() {
        let registry = FrozenScalarRegistry::standard().expect("the governed profile composes");
        let key = super::maximum_f32_scalar_op();
        let f32_type = crate::semantic::F32::resolved_type();
        let bf16_type = crate::semantic::Bf16::resolved_type();
        let capacity = super::ScalarInferenceCapacity {
            result_slots: 1,
            result_count_before: 0,
            result_limit: 1,
            retained_bytes: usize::MAX,
            retained_bytes_before: 0,
            retained_byte_limit: usize::MAX,
            per_result_overhead: 0,
            byte_multiplier: 1,
        };
        let attributes = super::ScalarAttributes::empty();

        let inferred = registry
            .infer(
                &key,
                &[f32_type.clone(), f32_type.clone()],
                &attributes,
                capacity,
            )
            .expect("a binary f32 application is admitted");
        assert_eq!(inferred, vec![f32_type.clone()]);

        for operands in [
            vec![bf16_type.clone(), bf16_type.clone()],
            vec![f32_type.clone(), bf16_type],
        ] {
            let foreign = registry
                .infer(&key, &operands, &attributes, capacity)
                .expect_err("this family declares no mixed precision and no promotion");
            let super::ScalarApplyError::Authority(super::ScalarRegistryError::Inference(
                rejection,
            )) = foreign
            else {
                panic!("a foreign operand must be the inferencer's refusal, not a host one");
            };
            assert_eq!(rejection.key(), &key);
            assert_eq!(
                rejection.rejection().code(),
                &crate::semantic::ProviderDiagnosticCode::new("tiler.scalar.operand-type")
                    .expect("the governed diagnostic code is valid"),
            );
        }

        // A third operand: an extrema *reduction* folds pairwise, so the scalar
        // is binary and a wider application is refused by the registered contract
        // rather than reduced by an implied fold.
        let arity = registry
            .infer(
                &key,
                &[f32_type.clone(), f32_type.clone(), f32_type],
                &attributes,
                capacity,
            )
            .expect_err("the governed maximum admits exactly two operands");
        assert!(
            matches!(
                arity,
                super::ScalarApplyError::Authority(super::ScalarRegistryError::OperandArity {
                    actual: 3,
                    ..
                })
            ),
            "a third operand must be an arity refusal; observed {arity:?}",
        );
    }

    /// The maximum has no semantic counterpart to derive its facts from.
    ///
    /// The same fact that decides the restate-and-check design for the NaN
    /// canonicalization decides it here: the graph admits no `Maximum` reduction
    /// as a semantic key — `tiler::softmax-f32@1` carries the extrema family in
    /// its own facts instead — so a rule that copied each scalar's record from its
    /// semantic operation would have no source for this one either.
    #[test]
    fn the_maximum_has_no_semantic_counterpart() {
        let semantic = FrozenSemanticRegistry::standard().expect("the governed authority composes");
        for name in ["maximum-f32", "maximum-number-f32", "minimum-f32"] {
            assert!(
                semantic
                    .operation_definition(&OpKey::new("tiler", name, 1).expect("the key is valid"))
                    .is_none(),
                "no semantic {name} family is registered",
            );
        }
        let scalars = FrozenScalarRegistry::standard().expect("the governed profile composes");
        assert!(
            scalars
                .definition(&super::maximum_f32_scalar_op())
                .is_some()
        );
        // And no sibling extrema scalar beside it: ADR 0023 makes each family a
        // separate operation, so a second one is a registration rather than a
        // reading of this one.
        for name in ["maximum-number-f32", "minimum-f32", "minimum-number-f32"] {
            assert!(
                scalars
                    .definition(
                        &ScalarOpKey::new("tiler.scalar", name, 1).expect("the key is valid")
                    )
                    .is_none(),
                "no governed {name} scalar is registered",
            );
        }
    }
}

#[cfg(test)]
mod resource_order_tests {
    use std::cell::Cell;
    use std::sync::Arc;

    use super::{
        ScalarArity, ScalarAttributeSchema, ScalarEffect, ScalarInferenceCapacity,
        ScalarInferenceError, ScalarInferenceOutputs, ScalarInferenceRequest, ScalarOpKey,
        ScalarOperationContract, ScalarOperationDefinition, ScalarOperationInferencer,
        encode_definition, encoded_definition_len,
    };
    use crate::semantic::{
        CanonicalValue, NormativeDefinitionRef, ProviderDiagnosticCode, ResolvedValueType, TypeKey,
    };

    struct NoResults;
    impl ScalarOperationInferencer for NoResults {
        fn infer(
            &self,
            _: ScalarInferenceRequest<'_>,
            _: &mut ScalarInferenceOutputs,
        ) -> Result<(), ScalarInferenceError> {
            Ok(())
        }
    }

    fn record() -> CanonicalValue {
        CanonicalValue::record([]).unwrap()
    }

    #[test]
    fn definition_size_prepass_matches_the_encoder_path_exactly() {
        let definition = ScalarOperationDefinition::new(
            ScalarOpKey::new("test", "sized", 1).unwrap(),
            NormativeDefinitionRef::new("urn:test:sized:v1").unwrap(),
            ScalarOperationContract::new(
                ScalarAttributeSchema::empty(),
                ScalarArity::exact(0).unwrap(),
                ScalarArity::exact(1).unwrap(),
                ScalarEffect::Pure,
                CanonicalValue::bytes(vec![7_u8; 4_096]).unwrap(),
                record(),
            ),
            Arc::new(NoResults),
        );
        assert_eq!(
            encoded_definition_len(&definition),
            encode_definition(&definition).len()
        );
    }

    #[test]
    fn inference_writer_uses_exact_enclosing_slots_without_rejecting_schema_maximum() {
        let value_type = ResolvedValueType::nominal(TypeKey::new("test", "value", 1).unwrap());
        let mut outputs = ScalarInferenceOutputs::new(
            4_096,
            ScalarInferenceCapacity {
                result_slots: 1,
                result_count_before: 12,
                result_limit: 13,
                retained_bytes: value_type.canonical_encoding().as_bytes().len() + 7,
                retained_bytes_before: 29,
                retained_byte_limit: 4_096,
                per_result_overhead: 7,
                byte_multiplier: 1,
            },
        );
        outputs.try_push(value_type.clone()).unwrap();
        assert_eq!(outputs.finish(Ok(()), 1).unwrap(), vec![value_type]);

        let mut outputs = ScalarInferenceOutputs::new(
            4_096,
            ScalarInferenceCapacity {
                result_slots: 0,
                result_count_before: 13,
                result_limit: 13,
                retained_bytes: usize::MAX,
                retained_bytes_before: 0,
                retained_byte_limit: usize::MAX,
                per_result_overhead: 0,
                byte_multiplier: 1,
            },
        );
        let error = outputs.try_push(ResolvedValueType::nominal(
            TypeKey::new("test", "value", 1).unwrap(),
        ));
        assert_eq!(
            error.unwrap_err().code(),
            &ProviderDiagnosticCode::new("tiler.scalar.host-result-capacity").unwrap()
        );
        assert_eq!(
            outputs.finish(Ok(()), 0),
            Err(super::ScalarInferenceFinishError::Host(
                super::ScalarInferenceHostFailure::ResultSlots {
                    actual: 14,
                    limit: 13,
                }
            ))
        );
    }

    #[test]
    fn host_slot_failure_wins_over_the_callback_result_without_provider_attribution() {
        let value_type = ResolvedValueType::nominal(TypeKey::new("test", "value", 1).unwrap());
        let calls = Cell::new(0_u32);
        let mut outputs = ScalarInferenceOutputs::new(
            2,
            ScalarInferenceCapacity {
                result_slots: 1,
                result_count_before: 65_535,
                result_limit: 65_536,
                retained_bytes: usize::MAX,
                retained_bytes_before: 0,
                retained_byte_limit: usize::MAX,
                per_result_overhead: 0,
                byte_multiplier: 1,
            },
        );
        calls.set(1);
        outputs.try_push(value_type.clone()).unwrap();
        calls.set(2);
        let ignored = outputs.try_push(value_type);
        calls.set(3);
        assert_eq!(
            ignored.unwrap_err().code(),
            &ProviderDiagnosticCode::new("tiler.scalar.host-result-capacity").unwrap()
        );
        let callback = Err(ScalarInferenceError::new(
            ProviderDiagnosticCode::new("provider.after-host-failure").unwrap(),
            "provider returned an error after ignoring the host writer failure",
        )
        .unwrap());
        assert_eq!(
            outputs.finish(callback, 1),
            Err(super::ScalarInferenceFinishError::Host(
                super::ScalarInferenceHostFailure::ResultSlots {
                    actual: 65_537,
                    limit: 65_536,
                }
            ))
        );
        assert_eq!(calls.get(), 3);
    }

    #[test]
    fn host_byte_failure_reports_exact_enclosing_usage_and_wins_callback_error() {
        let value_type = ResolvedValueType::nominal(TypeKey::new("test", "value", 1).unwrap());
        let result_bytes = value_type.canonical_encoding().as_bytes().len() + 7;
        let before = 1_000;
        let limit = before + result_bytes - 1;
        let mut outputs = ScalarInferenceOutputs::new(
            1,
            ScalarInferenceCapacity {
                result_slots: 1,
                result_count_before: 0,
                result_limit: 65_536,
                retained_bytes: result_bytes - 1,
                retained_bytes_before: before,
                retained_byte_limit: limit,
                per_result_overhead: 7,
                byte_multiplier: 1,
            },
        );
        let ignored = outputs.try_push(value_type);
        assert_eq!(
            ignored.unwrap_err().code(),
            &ProviderDiagnosticCode::new("tiler.scalar.host-byte-capacity").unwrap()
        );
        let callback = Err(ScalarInferenceError::new(
            ProviderDiagnosticCode::new("provider.after-host-failure").unwrap(),
            "provider returned an error after ignoring the host writer failure",
        )
        .unwrap());
        assert_eq!(
            outputs.finish(callback, 0),
            Err(super::ScalarInferenceFinishError::Host(
                super::ScalarInferenceHostFailure::CanonicalBytes {
                    actual: before + result_bytes,
                    limit,
                }
            ))
        );
    }

    #[test]
    fn multiplied_host_byte_capacity_is_exact_at_both_sides_of_the_boundary() {
        let value_type = ResolvedValueType::nominal(TypeKey::new("test", "value", 1).unwrap());
        let multiplier = 3;
        let unit_bytes = value_type.canonical_encoding().as_bytes().len() + 7;
        let required = unit_bytes * multiplier;
        let before = 2_000;
        let capacity = |retained_bytes, retained_byte_limit| ScalarInferenceCapacity {
            result_slots: 1,
            result_count_before: 0,
            result_limit: 65_536,
            retained_bytes,
            retained_bytes_before: before,
            retained_byte_limit,
            per_result_overhead: 7,
            byte_multiplier: multiplier,
        };

        let mut exact = ScalarInferenceOutputs::new(1, capacity(required, before + required));
        exact.try_push(value_type.clone()).unwrap();
        assert_eq!(exact.finish(Ok(()), 1).unwrap(), vec![value_type.clone()]);

        let mut short =
            ScalarInferenceOutputs::new(1, capacity(required - 1, before + required - 1));
        short.try_push(value_type).unwrap_err();
        assert_eq!(
            short.finish(Ok(()), 0),
            Err(super::ScalarInferenceFinishError::Host(
                super::ScalarInferenceHostFailure::CanonicalBytes {
                    actual: before + required,
                    limit: before + required - 1,
                }
            ))
        );
    }
}

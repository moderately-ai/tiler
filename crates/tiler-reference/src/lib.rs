//! Host reference values and evaluation for verified Tiler semantic programs.
//!
//! Two independent oracles share one exact tensor value boundary. The semantic
//! evaluator executes a verified [`tiler_ir::semantic::SemanticProgram`], and
//! the index-region oracle executes a verified
//! [`tiler_ir::index::VerifiedIndexRegion`] without reusing any graph-specific
//! host expression. **That independence is the point of the crate**: two
//! oracles that shared a lowering would agree with each other for reasons that
//! say nothing about either being right.
//!
//! # Where each thing lives
//!
//! This root is a facade. It declares the modules, re-exports the public
//! boundary, states the resource limits every module bounds itself by, and owns
//! the one rule that spans all of them — the arithmetic NaN canonicalization
//! below, which both oracles apply and neither may define separately.
//!
//! | module | authority |
//! | --- | --- |
//! | `accuracy` | certified enclosures and the transcendental conformance decision |
//! | `bf16` | the exact-rational pure-BF16 value set, arithmetic, and rounding |
//! | `tensor` | what a reference value *is* — elements, components, tensors |
//! | `registry` | the semantic capability registry and its dispatch vocabulary |
//! | `evaluate` | executing a semantic program against that registry |
//! | `silu` | the certified binary32 `SiLU` reference and its exponential |
//! | `rms_norm` | the certified binary32 RMS normalization and its reciprocal square root |
//! | `softmax` | the binary32 softmax, its extrema family, and its two folds |
//! | `standard` | the one provider this crate ships, a consumer of the registry |
//! | `structural` | reference semantics for the element-moving families |
//! | `contraction` | reference semantics for the governed tensor contraction |
//! | `identity` | canonical identity encoding for a frozen registry |
//! | `oracle` | the *other* oracle: scalar dispatch and index-region evaluation |
//! | `arithmetic` | exact scalar arithmetic shared by both |
//! | `error` | every typed failure any of the above reports |
//!
//! **`registry` and `oracle` both dispatch behaviour by key and are deliberately
//! not merged.** One governs semantic capabilities and the other scalar ones;
//! they carry different identities and different extension obligations, so a
//! shared mechanism would erase a distinction the contracts depend on.

mod accuracy;
mod arithmetic;
mod bf16;
mod conformance;
mod contraction;
mod error;
mod evaluate;
mod identity;
mod oracle;
mod quantization;
mod registry;
mod rms_norm;
mod silu;
mod softmax;
mod standard;
mod structural;
mod tensor;

pub use accuracy::{
    CertifiedEnclosure, ConformanceDecision, EnclosureError, EnclosurePrecision,
    UndecidedConformance, decide_contract, decide_predicate, exact_binary32_candidate,
    exp_enclosure, rsqrt_enclosure,
};
pub use conformance::{ReferenceNumericalConformance, UnsupportedReferenceContract};
pub use contraction::StagedStrictTensorContractionF32;
pub use error::{
    EvaluationError, ReferenceOperationError, ReferenceRegistryError, ReferenceRegistryResource,
    ReferenceResource, ReferenceValueError, StagedContractionError, UnsupportedBf16Declaration,
    UnsupportedContractionDeclaration,
};
pub use evaluate::{ReferenceEvaluator, strict_partial_sums, strict_partitioned_sum};
pub use oracle::{
    CanonicalScalarReferenceRegistryIdentity, FrozenScalarReferenceRegistry,
    IndexReferenceResource, IndexRegionAuthority, IndexRegionEvaluation,
    IndexRegionEvaluationError, IndexRegionEvaluator, IndexRegionInput,
    ScalarCapabilityAttribution, ScalarReferenceOperation, ScalarReferenceOutputs,
    ScalarReferenceRegistryBuilder, ScalarReferenceRegistryError, ScalarReferenceRequest,
    UnsupportedRegionFeature,
};
pub use registry::{
    CanonicalReferenceRegistryIdentity, FrozenReferenceRegistry, ReferenceCapabilityRevision,
    ReferenceEvaluationRequest, ReferenceOperation, ReferenceOutputs, ReferenceRegistryBuilder,
    ReferenceRegistryProvider, ReferenceRegistryRegistrar, ReferenceSignature,
    ReferenceValueValidator,
};
pub use rms_norm::{certified_rsqrt_f32, rms_norm_f32};
pub use silu::{certified_exp_f32, silu_f32};
pub use softmax::softmax_f32;
pub use tensor::{
    FloatBitOrder, InputBinding, ReferenceComponent, ReferenceComponentRole, ReferenceElement,
    Tensor, TensorPayloadView,
};

use tiler_ir::semantic::CANONICAL_F32_ARITHMETIC_NAN_BITS;

const MAX_REFERENCE_ELEMENT_BYTES: usize = 1024 * 1024;
pub(crate) const MAX_REFERENCE_TENSOR_ELEMENTS: usize = 16 * 1024 * 1024;
const MAX_REFERENCE_TENSOR_BYTES: usize = 64 * 1024 * 1024;
const MAX_REFERENCE_COMPONENTS: usize = 1_024;
const MAX_REFERENCE_COMPONENT_DEPTH: usize = 32;
pub(crate) const MAX_REFERENCE_CAPABILITIES: usize = 4_096;
pub(crate) const MAX_REFERENCE_REGISTRY_IDENTITY_BYTES: usize = 16 * 1024 * 1024;

/// Replaces any NaN produced by a binary32 arithmetic operation with the one
/// payload the governed contract declares.
///
/// The governed `tiler::multiply-f32@1` and `tiler::add-f32@1` definitions carry
/// [`CANONICAL_F32_ARITHMETIC_NAN_BITS`] as a declared operation fact, so an
/// arithmetic NaN has exactly one observable representation and a reference
/// result never depends on the host's choice of propagated payload. It applies
/// to an *arithmetic result*: a value that is only read, or an exact constant
/// payload, keeps its bits.
///
/// **The BF16 family applies this same rule, not a second one.** `bf16` has no
/// host float type whose `is_nan` this function could ask, so [`bf16`] decides the
/// class from the registered descriptor's own exponent and significand fields and
/// takes its payload from the family's declared `BF16_FACT_CANONICAL_NAN_BITS`.
/// What is shared is the rule stated here — one declared payload for every
/// arithmetic NaN result and the operand's payload never — and what differs is
/// only which declaration supplies that payload.
pub(crate) fn canonicalize_arithmetic_f32(value: f32) -> f32 {
    if value.is_nan() {
        f32::from_bits(CANONICAL_F32_ARITHMETIC_NAN_BITS)
    } else {
        value
    }
}

#[cfg(test)]
mod tests;

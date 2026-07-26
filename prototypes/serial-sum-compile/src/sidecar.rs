//! The proof-case sidecar this producer publishes beside its envelope.
//!
//! # Why the producer owns the operands and the expected bits
//!
//! Both halves of this vertical slice used to derive them independently: the
//! producer wrote nothing but the artifact identity, and the runner built the
//! same twelve operands from its own copy of the pattern and evaluated the same
//! reference to get its own expected outputs. Two derivations of one fact cannot
//! disagree loudly -- they agree until the day they do not, and then the runner
//! compares device bits against a reference the published artifact never
//! claimed. The sidecar makes the producer the sole author of both, so what the
//! runner checks is what the artifact was published against.
//!
//! It also subsumes the bare `.identity` file. That carried one field this
//! record carries too, and a consumer reading identity from one file and
//! operands from another can be handed a mismatched pair without either file
//! looking wrong.
//!
//! # What the sidecar does and does not establish
//!
//! `ProofSidecarBuilder` derives the envelope association here rather than
//! accepting it: it encodes the artifact and digests those exact bytes, so the
//! record names the envelope being written rather than one the producer claims
//! to have written. It also refuses a stated semantic graph that is not the
//! artifact's.
//!
//! The numerical and reference identities are opaque strings the artifact layer
//! cannot check, and are therefore claims by this producer rather than proofs.

use tiler_artifact::program::VerifiedArtifactProgram;
use tiler_artifact::proof::{
    ProofBuildError, ProofCaseKey, ProofCaseSpec, ProofCodecError, ProofNumericalIdentity,
    ProofProvenance, ProofReferenceIdentity, ProofSidecarBuilder,
};
use tiler_ir::semantic::{F32, InputKey, OutputKey, SemanticProgram};
use tiler_ir::shape::Shape;
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

use crate::{COLUMNS, ROWS};

/// Governed key of the numerical contract the expected bytes are normative
/// under. It names the flush-to-zero contract because that is the one this
/// producer compiles for; the strict contract is unhonourable on every governed
/// Apple family.
const NUMERICAL_IDENTITY: &[u8] = b"tiler.numerical.flush-subnormals-to-zero-f32";
/// Governed key of the implementation that produced the expected bytes.
const REFERENCE_IDENTITY: &[u8] = b"tiler.reference.standard-evaluator.v1";
/// Stable key of the single case this producer publishes.
const CASE_KEY: &str = "serial-sum-mixed-operands";
/// The artifact's declared input key.
const INPUT_KEY: &str = "input";
/// The artifact's declared output key.
const OUTPUT_KEY: &str = "result";

/// One row of operands, cycled to fill any declared shape.
///
/// Chosen to exercise the contract rather than to be arithmetically convenient:
/// a negative zero, the least positive subnormal, a non-canonical NaN payload,
/// and an infinity all appear, because those are the values where a numerical
/// contract either holds or is decorative. The interesting operand leads each
/// row, so a narrower reduction keeps every one of them.
const ROW_PATTERNS: [[u32; 3]; 4] = [
    [0x3f80_0000, 0x4000_0000, 0x4040_0000], // 1.0, 2.0, 3.0
    [0x8000_0000, 0x0000_0001, 0x3f80_0000], // -0.0, least subnormal, 1.0
    [0x7fc0_1234, 0x3f80_0000, 0x4000_0000], // non-canonical NaN, 1.0, 2.0
    [0x7f80_0000, 0x3f80_0000, 0xbf80_0000], // +inf, 1.0, -1.0
];

/// Why a proof-case sidecar could not be published.
#[derive(Debug)]
pub enum SidecarError {
    /// The draft refused the artifact, the provenance, or the case.
    Build(ProofBuildError),
    /// The verified record did not encode.
    Encode(ProofCodecError),
}

impl std::fmt::Display for SidecarError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Build(cause) => write!(formatter, "the proof sidecar draft refused: {cause:?}"),
            Self::Encode(cause) => write!(formatter, "the proof sidecar did not encode: {cause:?}"),
        }
    }
}

/// Fills one `rows` by `columns` input from [`ROW_PATTERNS`].
///
/// Cycling rather than indexing, so the pattern defines an input for any shape
/// an artifact might declare.
fn input_bits(rows: u64, columns: u64) -> Vec<u32> {
    let mut bits = Vec::new();
    for row in 0..rows {
        for column in 0..columns {
            let pattern = ROW_PATTERNS[usize::try_from(row % 4).expect("a bounded row index")];
            bits.push(pattern[usize::try_from(column % 3).expect("a bounded column index")]);
        }
    }
    bits
}

/// Evaluates the program under the governed reference to get expected outputs.
fn reference_bits(program: &SemanticProgram, bits: &[u32]) -> Vec<u32> {
    let key = InputKey::new(INPUT_KEY).expect("the input key is valid");
    let tensor = Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([ROWS, COLUMNS]),
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
    .expect("the input tensor is well formed");
    let outputs = ReferenceEvaluator::standard()
        .expect("the governed reference profile composes")
        .evaluate(program, &[InputBinding::new(&key, &tensor)])
        .expect("the reference evaluates the program");
    match outputs[0].payload() {
        TensorPayloadView::Dense(elements) => elements
            .iter()
            .map(|element| {
                u32::from_be_bytes(
                    <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
                )
            })
            .collect(),
        _ => panic!("expected a dense f32 reference output"),
    }
}

/// Encodes big-endian `f32` bit patterns as the sidecar's payload bytes.
///
/// Most-significant byte first throughout, matching the order the reference
/// elements were built from, so a payload never depends on host endianness.
fn payload_bytes(bits: &[u32]) -> Vec<u8> {
    bits.iter().flat_map(|value| value.to_be_bytes()).collect()
}

/// Builds and encodes the proof-case sidecar for one published artifact.
///
/// # Errors
///
/// Returns [`SidecarError`] naming the boundary that refused. Nothing is worked
/// around: a rejection here means the record would not have described the
/// artifact it travels with.
pub fn encoded(
    artifact: &VerifiedArtifactProgram,
    program: &SemanticProgram,
) -> Result<Vec<u8>, SidecarError> {
    let inputs = input_bits(ROWS, COLUMNS);
    let expected = reference_bits(program, &inputs);

    let mut draft = ProofSidecarBuilder::new(
        artifact,
        ProofProvenance {
            semantic_graph: program.semantic_identity().graph().clone(),
            numerical: ProofNumericalIdentity::from_bytes(NUMERICAL_IDENTITY)
                .expect("the governed numerical key is in bounds"),
            reference: ProofReferenceIdentity::from_bytes(REFERENCE_IDENTITY)
                .expect("the governed reference key is in bounds"),
        },
    )
    .map_err(SidecarError::Build)?;

    draft
        .push_case(ProofCaseSpec {
            key: ProofCaseKey::new(CASE_KEY).expect("the case key is valid"),
            inputs: vec![(
                InputKey::new(INPUT_KEY).expect("the input key is valid"),
                payload_bytes(&inputs),
            )],
            expected: vec![(
                OutputKey::new(OUTPUT_KEY).expect("the output key is valid"),
                payload_bytes(&expected),
            )],
        })
        .map_err(SidecarError::Build)?;

    draft
        .build()
        .map_err(SidecarError::Build)?
        .encode()
        .map_err(SidecarError::Encode)
}

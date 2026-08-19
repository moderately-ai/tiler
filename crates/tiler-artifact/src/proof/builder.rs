//! Transactional construction of one proof sidecar.
//!
//! The lifecycle is ADR 0071's, restated for this layer rather than re-decided:
//! a builder that owns private storage and checks local invariants on every
//! insertion, and a terminal that consumes the builder, runs whole-object
//! verification, and yields an opaque immutable product or a typed failure.
//!
//! # What the builder refuses to accept from its caller
//!
//! Two facts a producer could get wrong are derived here instead of supplied.
//!
//! **The association.** The builder is handed the verified artifact program
//! itself, encodes it, and digests the exact bytes. A producer therefore cannot
//! pair a sidecar with an identity it did not compute or with bytes it did not
//! write, which is the failure mode a `(identity, digest)` parameter pair would
//! have made easy and undetectable.
//!
//! **The bound interface.** The keys a case supplies payloads for are the
//! artifact's own declared inputs and outputs, held in the artifact's interface
//! order. A case names its keys and the builder places them; it cannot
//! introduce a key the artifact does not declare, omit one it does, or reorder
//! them.
//!
//! One fact is deliberately *not* derived: the semantic graph the expectation
//! was evaluated over. It is supplied, because the risk this check exists to
//! catch is a producer that reference-evaluated a different program from the
//! one it compiled. Deriving it from the artifact would make the check
//! tautological; requiring it and comparing makes the mismatch a build failure.

use std::error::Error;
use std::fmt;

use tiler_ir::semantic::{InputKey, OutputKey, SemanticGraphIdentity};

use crate::program::{
    ArtifactCodecFailure, RecordedArtifactEnvelopeDigest, VerifiedArtifactProgram, envelope_digest,
};

use super::budget::{CaseLens, ProofBudgetError, project_from_data, project_sidecar};
use super::codec::{ProofLimitExceeded, ProofLimitKind, derive_identity, proof_limit};
use super::model::{
    ProofCaseData, ProofCaseKey, ProofCaseKeyError, ProofNumericalIdentity, ProofReferenceIdentity,
    ProofSemanticSubject, ProofSidecarData, ProofSubjectError, ProofSubjects, VerifiedProofSidecar,
};
use super::{MAX_PROOF_CASES, MAX_PROOF_INTERFACE_ENTRIES};

/// Which half of the named interface an obligation is about.
///
/// Deliberately **not** `#[non_exhaustive]` (ADR 0074 convention 5c): a
/// consumer that reports or routes on this matches it to decide behaviour, and
/// a third direction must break such a match rather than fall into a wildcard.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProofDirection {
    /// A bit-preserving case input.
    Input,
    /// A normative expected case output.
    Expected,
}

impl fmt::Display for ProofDirection {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Input => "input",
            Self::Expected => "expected",
        })
    }
}

/// A disagreement between a sidecar's cases and the interface they bind.
///
/// Both halves of the obligation report through this one vocabulary — the half
/// a decoder can prove on its own, and the half that needs the artifact — so a
/// consumer classifies one kind of failure rather than two.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProofInterfaceError {
    /// The sidecar binds a different number of entries than the artifact declares.
    Arity {
        /// Which half of the interface disagreed.
        direction: ProofDirection,
        /// Entries the sidecar binds.
        sidecar: usize,
        /// Entries the artifact declares.
        artifact: usize,
    },
    /// The sidecar binds a different input key than the artifact declares there.
    InputKeyMismatch {
        /// Interface position that disagreed.
        position: usize,
        /// Key the sidecar binds.
        sidecar: InputKey,
        /// Key the artifact declares.
        artifact: InputKey,
    },
    /// The sidecar binds a different output key than the artifact declares there.
    OutputKeyMismatch {
        /// Interface position that disagreed.
        position: usize,
        /// Key the sidecar binds.
        sidecar: OutputKey,
        /// Key the artifact declares.
        artifact: OutputKey,
    },
    /// One case supplied a payload count that is not the bound entry count.
    PayloadArity {
        /// The case that disagreed.
        case: ProofCaseKey,
        /// Which half of the interface disagreed.
        direction: ProofDirection,
        /// Payloads the case supplied.
        supplied: usize,
        /// Entries the sidecar binds.
        bound: usize,
    },
    /// One payload is not a whole number of elements of its declared shape.
    ///
    /// The check is divisibility and nonemptiness, not a width: this crate does
    /// not know the storage width of a governed element type, and inventing one
    /// would assert a byte count no verifier examined.
    PayloadNotWholeElements {
        /// The case that disagreed.
        case: ProofCaseKey,
        /// Which half of the interface disagreed.
        direction: ProofDirection,
        /// Interface position that disagreed.
        position: usize,
        /// Byte length the case supplied.
        bytes: usize,
        /// Dense element count of the declared shape.
        elements: usize,
    },
    /// Two cases supplied different byte lengths for one interface entry.
    ///
    /// An artifact's declared shapes are fixed, so every case's payload for a
    /// given entry holds the same element count at the same width. Two lengths
    /// therefore mean two different interpretations of one buffer, and this is
    /// provable without the artifact — which is why a decoder proves it before
    /// any artifact is offered.
    PayloadLengthDisagreement {
        /// Which half of the interface disagreed.
        direction: ProofDirection,
        /// Interface position that disagreed.
        position: usize,
        /// Byte length the canonically first case supplied.
        first: usize,
        /// The case that disagreed with it.
        case: ProofCaseKey,
        /// Byte length that case supplied.
        bytes: usize,
    },
    /// A declared shape's dense element count is not representable on this host.
    ShapeNotRepresentable {
        /// Which half of the interface could not be projected.
        direction: ProofDirection,
        /// Interface position that could not be projected.
        position: usize,
    },
}

impl fmt::Display for ProofInterfaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Arity {
                direction,
                sidecar,
                artifact,
            } => write!(
                formatter,
                "the sidecar binds {sidecar} {direction} entries and the artifact declares {artifact}"
            ),
            Self::InputKeyMismatch {
                position,
                sidecar,
                artifact,
            } => write!(
                formatter,
                "input position {position} binds `{}` and the artifact declares `{}`",
                sidecar.as_str(),
                artifact.as_str(),
            ),
            Self::OutputKeyMismatch {
                position,
                sidecar,
                artifact,
            } => write!(
                formatter,
                "output position {position} binds `{}` and the artifact declares `{}`",
                sidecar.as_str(),
                artifact.as_str(),
            ),
            Self::PayloadArity {
                case,
                direction,
                supplied,
                bound,
            } => write!(
                formatter,
                "case `{case}` supplied {supplied} {direction} payloads for {bound} bound entries"
            ),
            Self::PayloadNotWholeElements {
                case,
                direction,
                position,
                bytes,
                elements,
            } => write!(
                formatter,
                "case `{case}` {direction} position {position} carries {bytes} bytes, which is not a whole number of the {elements} declared elements"
            ),
            Self::PayloadLengthDisagreement {
                direction,
                position,
                first,
                case,
                bytes,
            } => write!(
                formatter,
                "{direction} position {position} is {first} bytes in the first case and {bytes} in case `{case}`"
            ),
            Self::ShapeNotRepresentable {
                direction,
                position,
            } => write!(
                formatter,
                "the declared shape of {direction} position {position} has no host-representable element count"
            ),
        }
    }
}

impl Error for ProofInterfaceError {}

/// Why an artifact's declared interface could not be bound.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum InterfaceProjectionError {
    /// The interface exceeds a governed structural bound.
    Limit(ProofLimitExceeded),
    /// A declared shape could not be projected.
    Interface(ProofInterfaceError),
}

/// A typed failure of proof-sidecar construction.
///
/// `#[non_exhaustive]` under ADR 0074 convention 5a: a rejection vocabulary a
/// producer reports and forwards, never one any crate maps totally.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ProofBuildError {
    /// A stable case key was rejected.
    CaseKey(ProofCaseKeyError),
    /// A received provenance subject was rejected.
    Subject(ProofSubjectError),
    /// Two cases share one stable key.
    DuplicateCaseKey {
        /// The key supplied twice.
        key: ProofCaseKey,
    },
    /// A case supplied a payload for a key the artifact does not declare.
    UnknownInput {
        /// The undeclared key.
        key: InputKey,
    },
    /// A case supplied an expectation for a key the artifact does not declare.
    UnknownOutput {
        /// The undeclared key.
        key: OutputKey,
    },
    /// A case supplied no payload for a declared input.
    MissingInput {
        /// The unsupplied key.
        key: InputKey,
    },
    /// A case supplied no expectation for a declared output.
    MissingOutput {
        /// The unsupplied key.
        key: OutputKey,
    },
    /// A case supplied two payloads for one declared input.
    DuplicateInput {
        /// The key supplied twice.
        key: InputKey,
    },
    /// A case supplied two expectations for one declared output.
    DuplicateOutput {
        /// The key supplied twice.
        key: OutputKey,
    },
    /// A governed structural bound was exceeded.
    Limit(ProofLimitExceeded),
    /// The reference run's semantic subject is not the artifact's.
    ///
    /// The producer stated which graph it evaluated to obtain the expected
    /// bytes, and it is not the graph the artifact realizes. Accepting this
    /// would produce a sidecar whose expectations are normative for a different
    /// program, which no later comparison could detect: a device that computed
    /// the artifact correctly would fail against them, and the failure would
    /// look like a numerical defect.
    SemanticSubjectMismatch {
        /// Graph identity bytes the producer stated.
        declared: Vec<u8>,
        /// Graph identity bytes the artifact realizes.
        artifact: Vec<u8>,
    },
    /// The associated artifact could not be encoded, so it has no envelope to name.
    EnvelopeEncoding(ArtifactCodecFailure),
    /// The sidecar's cases disagree with the artifact's named interface.
    Interface(ProofInterfaceError),
    /// The sidecar carries no case.
    ///
    /// An empty sidecar is refused rather than admitted, because it would
    /// present as validated evidence while proving nothing, and a consumer that
    /// iterated zero cases would report success.
    NoCases,
    /// A projected encoded size overflowed this host's `usize`.
    Unrepresentable {
        /// The bound whose projected size could not be represented.
        kind: ProofLimitKind,
    },
}

impl fmt::Display for ProofBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CaseKey(cause) => write!(formatter, "proof-case key rejected: {cause}"),
            Self::Subject(cause) => write!(formatter, "provenance subject rejected: {cause}"),
            Self::DuplicateCaseKey { key } => {
                write!(formatter, "two proof cases share the key `{key}`")
            }
            Self::UnknownInput { key } => {
                write!(
                    formatter,
                    "the artifact declares no input `{}`",
                    key.as_str()
                )
            }
            Self::UnknownOutput { key } => write!(
                formatter,
                "the artifact declares no output `{}`",
                key.as_str(),
            ),
            Self::MissingInput { key } => write!(
                formatter,
                "no case payload was supplied for input `{}`",
                key.as_str(),
            ),
            Self::MissingOutput { key } => write!(
                formatter,
                "no expected payload was supplied for output `{}`",
                key.as_str(),
            ),
            Self::DuplicateInput { key } => write!(
                formatter,
                "two payloads were supplied for input `{}`",
                key.as_str(),
            ),
            Self::DuplicateOutput { key } => write!(
                formatter,
                "two payloads were supplied for output `{}`",
                key.as_str(),
            ),
            Self::Limit(cause) => write!(formatter, "{cause}"),
            Self::SemanticSubjectMismatch { .. } => formatter.write_str(
                "the semantic graph the expectations were evaluated over is not the artifact's",
            ),
            Self::EnvelopeEncoding(cause) => {
                write!(
                    formatter,
                    "the associated artifact has no envelope: {cause}"
                )
            }
            Self::Interface(cause) => write!(formatter, "interface disagreement: {cause}"),
            Self::NoCases => formatter.write_str("a proof sidecar carries no case"),
            Self::Unrepresentable { kind } => write!(
                formatter,
                "a projected {kind} size is not representable on this host"
            ),
        }
    }
}

impl Error for ProofBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CaseKey(cause) => Some(cause),
            Self::Subject(cause) => Some(cause),
            Self::Limit(cause) => Some(cause),
            Self::EnvelopeEncoding(cause) => Some(cause),
            Self::Interface(cause) => Some(cause),
            Self::DuplicateCaseKey { .. }
            | Self::UnknownInput { .. }
            | Self::UnknownOutput { .. }
            | Self::MissingInput { .. }
            | Self::MissingOutput { .. }
            | Self::DuplicateInput { .. }
            | Self::DuplicateOutput { .. }
            | Self::SemanticSubjectMismatch { .. }
            | Self::NoCases
            | Self::Unrepresentable { .. } => None,
        }
    }
}

impl From<ProofCaseKeyError> for ProofBuildError {
    fn from(cause: ProofCaseKeyError) -> Self {
        Self::CaseKey(cause)
    }
}

impl From<ProofSubjectError> for ProofBuildError {
    fn from(cause: ProofSubjectError) -> Self {
        Self::Subject(cause)
    }
}

impl From<ProofInterfaceError> for ProofBuildError {
    fn from(cause: ProofInterfaceError) -> Self {
        Self::Interface(cause)
    }
}

impl From<ProofLimitExceeded> for ProofBuildError {
    fn from(cause: ProofLimitExceeded) -> Self {
        Self::Limit(cause)
    }
}

impl From<ProofBudgetError> for ProofBuildError {
    fn from(cause: ProofBudgetError) -> Self {
        match cause {
            ProofBudgetError::Limit(cause) => Self::Limit(cause),
            ProofBudgetError::Unrepresentable { kind } => Self::Unrepresentable { kind },
        }
    }
}

impl From<InterfaceProjectionError> for ProofBuildError {
    fn from(cause: InterfaceProjectionError) -> Self {
        match cause {
            InterfaceProjectionError::Limit(cause) => Self::Limit(cause),
            InterfaceProjectionError::Interface(cause) => Self::Interface(cause),
        }
    }
}

/// The three authorities a producer states for its expectations.
///
/// A caller-constructed input record, so it exposes fields and carries no
/// `#[non_exhaustive]` (ADR 0074 convention 5a's stated asymmetry): a caller
/// must be able to write the literal, and growing the record is a
/// constructor-signature change either way.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofProvenance {
    /// The semantic graph the expected bytes were evaluated over.
    ///
    /// Typed rather than opaque, because this crate can compare it with the
    /// artifact's own and refuse a mismatch. The other two are opaque because
    /// this crate has no way to check them, and pretending otherwise would be a
    /// stronger claim than the code makes.
    pub semantic_graph: SemanticGraphIdentity,
    /// The numerical contract the expected bytes are normative under.
    pub numerical: ProofNumericalIdentity,
    /// The reference implementation that produced the expected bytes.
    pub reference: ProofReferenceIdentity,
}

/// One proof case as a producer states it.
///
/// A caller-constructed input record; see [`ProofProvenance`] for why it
/// exposes fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofCaseSpec {
    /// The stable key naming this case.
    pub key: ProofCaseKey,
    /// One bit-preserving payload per declared artifact input, in any order.
    pub inputs: Vec<(InputKey, Vec<u8>)>,
    /// One normative expected payload per declared artifact output, in any order.
    pub expected: Vec<(OutputKey, Vec<u8>)>,
}

/// The artifact-declared interface a sidecar's cases bind, with the dense
/// element count of each entry's declared shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BoundInterface {
    pub(super) inputs: Vec<(InputKey, usize)>,
    pub(super) outputs: Vec<(OutputKey, usize)>,
}

/// Reads the artifact's named interface and each entry's dense element count.
///
/// # Errors
///
/// Returns [`InterfaceProjectionError::Limit`] when either half exceeds
/// [`MAX_PROOF_INTERFACE_ENTRIES`], or
/// [`InterfaceProjectionError::Interface`] when a declared shape's element
/// count does not fit this host.
pub(super) fn project_interface(
    artifact: &VerifiedArtifactProgram,
) -> Result<BoundInterface, InterfaceProjectionError> {
    proof_limit(
        artifact.inputs().len(),
        MAX_PROOF_INTERFACE_ENTRIES,
        ProofLimitKind::InterfaceEntries,
    )
    .map_err(InterfaceProjectionError::Limit)?;
    proof_limit(
        artifact.outputs().len(),
        MAX_PROOF_INTERFACE_ENTRIES,
        ProofLimitKind::InterfaceEntries,
    )
    .map_err(InterfaceProjectionError::Limit)?;
    let mut inputs = Vec::with_capacity(artifact.inputs().len());
    for (position, entry) in artifact.inputs().enumerate() {
        let elements = entry
            .static_shape()
            .and_then(|shape| shape.element_count())
            .ok_or(InterfaceProjectionError::Interface(
                ProofInterfaceError::ShapeNotRepresentable {
                    direction: ProofDirection::Input,
                    position,
                },
            ))?;
        inputs.push((entry.key().clone(), elements));
    }
    let mut outputs = Vec::with_capacity(artifact.outputs().len());
    for (position, entry) in artifact.outputs().enumerate() {
        let elements = entry
            .static_shape()
            .and_then(|shape| shape.element_count())
            .ok_or(InterfaceProjectionError::Interface(
                ProofInterfaceError::ShapeNotRepresentable {
                    direction: ProofDirection::Expected,
                    position,
                },
            ))?;
        outputs.push((entry.key().clone(), elements));
    }
    Ok(BoundInterface { inputs, outputs })
}

/// Proves the obligations over a sidecar's cases that need no artifact.
///
/// Every case supplies exactly one payload per bound interface entry, and all
/// cases agree on each entry's byte length. Both follow from the sidecar's own
/// content, which is why a decoder proves them before any artifact is offered:
/// a container that fails either is malformed evidence regardless of what it is
/// later paired with.
///
/// # Errors
///
/// Returns the [`ProofInterfaceError`] naming the first obligation that failed.
pub(super) fn verify_case_payloads(data: &ProofSidecarData) -> Result<(), ProofInterfaceError> {
    let mut first_input_bytes: Vec<Option<usize>> = vec![None; data.input_keys.len()];
    let mut first_expected_bytes: Vec<Option<usize>> = vec![None; data.output_keys.len()];
    for case in &data.cases {
        check_lengths(
            &case.key,
            ProofDirection::Input,
            &case.inputs,
            data.input_keys.len(),
            &mut first_input_bytes,
        )?;
        check_lengths(
            &case.key,
            ProofDirection::Expected,
            &case.expected,
            data.output_keys.len(),
            &mut first_expected_bytes,
        )?;
    }
    Ok(())
}

/// Proves every obligation between a sidecar's cases and one artifact's
/// declared interface.
///
/// This is the single implementation of that obligation. The builder's terminal
/// calls it over the data it is about to freeze, and
/// [`DecodedProofSidecar::bind_to_artifact`] calls it over data that arrived as
/// bytes; a producer-side copy and a consumer-side copy could agree today and
/// drift tomorrow, and the drift would be invisible because each half would
/// still pass its own tests.
///
/// [`DecodedProofSidecar::bind_to_artifact`]: super::DecodedProofSidecar::bind_to_artifact
///
/// # Errors
///
/// Returns the [`ProofInterfaceError`] naming the first obligation that failed.
pub(super) fn verify_cases(
    interface: &BoundInterface,
    data: &ProofSidecarData,
) -> Result<(), ProofInterfaceError> {
    if data.input_keys.len() != interface.inputs.len() {
        return Err(ProofInterfaceError::Arity {
            direction: ProofDirection::Input,
            sidecar: data.input_keys.len(),
            artifact: interface.inputs.len(),
        });
    }
    if data.output_keys.len() != interface.outputs.len() {
        return Err(ProofInterfaceError::Arity {
            direction: ProofDirection::Expected,
            sidecar: data.output_keys.len(),
            artifact: interface.outputs.len(),
        });
    }
    for (position, (bound, (declared, _))) in
        data.input_keys.iter().zip(&interface.inputs).enumerate()
    {
        if bound != declared {
            return Err(ProofInterfaceError::InputKeyMismatch {
                position,
                sidecar: bound.clone(),
                artifact: declared.clone(),
            });
        }
    }
    for (position, (bound, (declared, _))) in
        data.output_keys.iter().zip(&interface.outputs).enumerate()
    {
        if bound != declared {
            return Err(ProofInterfaceError::OutputKeyMismatch {
                position,
                sidecar: bound.clone(),
                artifact: declared.clone(),
            });
        }
    }
    verify_case_payloads(data)?;
    for case in &data.cases {
        check_elements(
            &case.key,
            ProofDirection::Input,
            &case.inputs,
            &interface.inputs,
        )?;
        check_elements(
            &case.key,
            ProofDirection::Expected,
            &case.expected,
            &interface.outputs,
        )?;
    }
    Ok(())
}

/// Checks one case's payload count and cross-case length agreement.
fn check_lengths(
    case: &ProofCaseKey,
    direction: ProofDirection,
    payloads: &[Vec<u8>],
    bound: usize,
    first_seen: &mut [Option<usize>],
) -> Result<(), ProofInterfaceError> {
    if payloads.len() != bound {
        return Err(ProofInterfaceError::PayloadArity {
            case: case.clone(),
            direction,
            supplied: payloads.len(),
            bound,
        });
    }
    for (position, payload) in payloads.iter().enumerate() {
        let bytes = payload.len();
        match first_seen[position] {
            None => first_seen[position] = Some(bytes),
            Some(first) if first != bytes => {
                return Err(ProofInterfaceError::PayloadLengthDisagreement {
                    direction,
                    position,
                    first,
                    case: case.clone(),
                    bytes,
                });
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// Checks one case's payloads against the declared element counts.
fn check_elements<K>(
    case: &ProofCaseKey,
    direction: ProofDirection,
    payloads: &[Vec<u8>],
    entries: &[(K, usize)],
) -> Result<(), ProofInterfaceError> {
    for (position, (payload, (_, elements))) in payloads.iter().zip(entries).enumerate() {
        let bytes = payload.len();
        // An empty logical domain carries no bytes, and a nonempty one carries
        // at least one whole byte per element. The width itself is deliberately
        // not derived: a governed element type's storage width is a backend
        // fact this crate does not own, so the check is divisibility and
        // nonemptiness rather than an invented byte count.
        let whole = if *elements == 0 {
            bytes == 0
        } else {
            bytes % elements == 0 && bytes / elements >= 1
        };
        if !whole {
            return Err(ProofInterfaceError::PayloadNotWholeElements {
                case: case.clone(),
                direction,
                position,
                bytes,
                elements: *elements,
            });
        }
    }
    Ok(())
}

/// A transactional draft of one proof sidecar.
///
/// The draft is unchanged when an insertion is rejected, and [`Self::build`]
/// consumes it.
#[derive(Clone, Debug)]
pub struct ProofSidecarBuilder {
    artifact_identity: Vec<u8>,
    envelope_digest: RecordedArtifactEnvelopeDigest,
    subjects: ProofSubjects,
    interface: BoundInterface,
    cases: Vec<ProofCaseData>,
}

impl ProofSidecarBuilder {
    /// Opens a draft bound to one verified artifact and one provenance record.
    ///
    /// The association is derived here, not supplied: the artifact is encoded
    /// and its exact bytes digested, so the sidecar names the envelope the
    /// producer is about to write rather than one it claims to have written.
    ///
    /// # Errors
    ///
    /// Returns [`ProofBuildError::SemanticSubjectMismatch`] when the stated
    /// reference-run graph is not the artifact's,
    /// [`ProofBuildError::EnvelopeEncoding`] when the artifact does not encode,
    /// [`ProofBuildError::Subject`] when a provenance subject is out of bounds,
    /// or [`ProofBuildError::Limit`] / [`ProofBuildError::Interface`] when the
    /// artifact's declared interface cannot be bound.
    pub fn new(
        artifact: &VerifiedArtifactProgram,
        provenance: ProofProvenance,
    ) -> Result<Self, ProofBuildError> {
        let ProofProvenance {
            semantic_graph,
            numerical,
            reference,
        } = provenance;
        if &semantic_graph != artifact.semantic_graph_identity() {
            return Err(ProofBuildError::SemanticSubjectMismatch {
                declared: semantic_graph.as_bytes().to_vec(),
                artifact: artifact.semantic_graph_identity().as_bytes().to_vec(),
            });
        }
        let bytes = artifact
            .encode()
            .map_err(ProofBuildError::EnvelopeEncoding)?;
        let interface = project_interface(artifact)?;
        Ok(Self {
            artifact_identity: artifact.canonical_identity().as_bytes().to_vec(),
            envelope_digest: RecordedArtifactEnvelopeDigest::from_wire(envelope_digest(&bytes)),
            subjects: ProofSubjects {
                semantic: ProofSemanticSubject::from_bytes(semantic_graph.as_bytes())?,
                numerical,
                reference,
            },
            interface,
            cases: Vec::new(),
        })
    }

    /// Admits one proof case.
    ///
    /// The case names its payloads by interface key in any order; they are
    /// placed into the artifact's interface order here. The draft is left
    /// unchanged when the case is rejected.
    ///
    /// # Errors
    ///
    /// Returns [`ProofBuildError::DuplicateCaseKey`] for a repeated stable key,
    /// [`ProofBuildError::UnknownInput`] or [`ProofBuildError::UnknownOutput`]
    /// for a key the artifact does not declare,
    /// [`ProofBuildError::DuplicateInput`] or
    /// [`ProofBuildError::DuplicateOutput`] for a key supplied twice,
    /// [`ProofBuildError::MissingInput`] or [`ProofBuildError::MissingOutput`]
    /// for a declared key left unsupplied, [`ProofBuildError::Limit`] beyond
    /// [`MAX_PROOF_CASES`] or a projected identity, manifest, or
    /// complete-sidecar bound, or [`ProofBuildError::Unrepresentable`] when the
    /// projected size overflows this host's `usize`.
    pub fn push_case(&mut self, case: ProofCaseSpec) -> Result<(), ProofBuildError> {
        let attempted =
            self.cases
                .len()
                .checked_add(1)
                .ok_or(ProofBuildError::Unrepresentable {
                    kind: ProofLimitKind::Cases,
                })?;
        proof_limit(attempted, MAX_PROOF_CASES, ProofLimitKind::Cases)?;
        if self.cases.iter().any(|held| held.key == case.key) {
            return Err(ProofBuildError::DuplicateCaseKey { key: case.key });
        }
        let input_slots = resolve_slots(
            &case.inputs,
            &self.interface.inputs,
            |key| ProofBuildError::UnknownInput { key },
            |key| ProofBuildError::DuplicateInput { key },
            |key| ProofBuildError::MissingInput { key },
        )?;
        let expected_slots = resolve_slots(
            &case.expected,
            &self.interface.outputs,
            |key| ProofBuildError::UnknownOutput { key },
            |key| ProofBuildError::DuplicateOutput { key },
            |key| ProofBuildError::MissingOutput { key },
        )?;
        self.project_with(
            &case.key,
            &case.inputs,
            &input_slots,
            &case.expected,
            &expected_slots,
        )?
        .check()?;
        let inputs = take_placed(case.inputs, &input_slots);
        let expected = take_placed(case.expected, &expected_slots);
        self.cases.push(ProofCaseData {
            key: case.key,
            inputs,
            expected,
        });
        Ok(())
    }

    /// Consumes the draft and yields the verified sidecar.
    ///
    /// Cases are ordered canonically by stable key here, so two producers that
    /// admitted the same cases in different orders emit the same bytes and the
    /// same identity.
    ///
    /// # Errors
    ///
    /// Returns [`ProofBuildError::NoCases`] for an empty draft,
    /// [`ProofBuildError::Interface`] when a case disagrees with the artifact's
    /// declared interface, [`ProofBuildError::Limit`] when a projected identity,
    /// manifest, or complete-sidecar size exceeds its governed bound, or
    /// [`ProofBuildError::Unrepresentable`] when a projected size overflows
    /// this host's `usize`.
    pub fn build(mut self) -> Result<VerifiedProofSidecar, ProofBuildError> {
        if self.cases.is_empty() {
            return Err(ProofBuildError::NoCases);
        }
        self.cases.sort_by(|left, right| left.key.cmp(&right.key));
        let data = ProofSidecarData {
            artifact_identity: self.artifact_identity,
            envelope_digest: self.envelope_digest,
            subjects: self.subjects,
            input_keys: self
                .interface
                .inputs
                .iter()
                .map(|(key, _)| key.clone())
                .collect(),
            output_keys: self
                .interface
                .outputs
                .iter()
                .map(|(key, _)| key.clone())
                .collect(),
            cases: self.cases,
        };
        verify_cases(&self.interface, &data)?;
        project_from_data(&data)?.check()?;
        let identity = derive_identity(&data)?;
        Ok(VerifiedProofSidecar { data, identity })
    }

    fn project_with<IK: Eq, OK: Eq>(
        &self,
        key: &ProofCaseKey,
        inputs: &[(IK, Vec<u8>)],
        input_slots: &[usize],
        expected: &[(OK, Vec<u8>)],
        expected_slots: &[usize],
    ) -> Result<super::budget::ProjectedSizes, ProofBudgetError> {
        let mut cases: Vec<CaseLens> = self
            .cases
            .iter()
            .map(|held| CaseLens {
                key_len: held.key.as_str().len(),
                input_lens: held.inputs.iter().map(Vec::len).collect(),
                expected_lens: held.expected.iter().map(Vec::len).collect(),
            })
            .collect();
        cases.push(CaseLens {
            key_len: key.as_str().len(),
            input_lens: input_slots
                .iter()
                .map(|&slot| inputs[slot].1.len())
                .collect(),
            expected_lens: expected_slots
                .iter()
                .map(|&slot| expected[slot].1.len())
                .collect(),
        });
        project_sidecar(
            self.artifact_identity.len(),
            self.subjects.semantic.as_bytes().len(),
            self.subjects.numerical.as_bytes().len(),
            self.subjects.reference.as_bytes().len(),
            self.interface
                .inputs
                .iter()
                .map(|(entry, _)| entry.as_str().len()),
            self.interface
                .outputs
                .iter()
                .map(|(entry, _)| entry.as_str().len()),
            cases,
        )
    }
}

/// Resolves a case's keyed payloads onto the artifact's interface order.
///
/// A miss in either direction is a distinct named failure: a key the artifact
/// does not declare, a key supplied twice, and a declared key left unsupplied
/// are three different producer mistakes and a caller reacts differently to
/// each. The returned slots are indexes into `supplied`; nothing is cloned.
fn resolve_slots<K: Clone + Eq>(
    supplied: &[(K, Vec<u8>)],
    entries: &[(K, usize)],
    unknown: impl Fn(K) -> ProofBuildError,
    duplicate: impl Fn(K) -> ProofBuildError,
    missing: impl Fn(K) -> ProofBuildError,
) -> Result<Vec<usize>, ProofBuildError> {
    let mut slots: Vec<Option<usize>> = vec![None; entries.len()];
    for (index, (key, _bytes)) in supplied.iter().enumerate() {
        let position = entries
            .iter()
            .position(|(declared, _)| declared == key)
            .ok_or_else(|| unknown(key.clone()))?;
        if slots[position].is_some() {
            return Err(duplicate(key.clone()));
        }
        slots[position] = Some(index);
    }
    slots
        .into_iter()
        .enumerate()
        .map(|(position, slot)| slot.ok_or_else(|| missing(entries[position].0.clone())))
        .collect()
}

/// Moves already-validated payloads into interface order.
fn take_placed<K>(supplied: Vec<(K, Vec<u8>)>, slots: &[usize]) -> Vec<Vec<u8>> {
    let mut owned: Vec<Option<Vec<u8>>> =
        supplied.into_iter().map(|(_, bytes)| Some(bytes)).collect();
    slots
        .iter()
        .map(|&index| {
            owned[index]
                .take()
                .expect("each resolved slot names a unique supplied payload")
        })
        .collect()
}

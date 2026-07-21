use std::error::Error;
use std::fmt;
use std::sync::Arc;

use super::interface::{InputKey, InterfaceKind, OutputKey};
use super::registry::{RegistryError, RegistryLookupError};
use super::types::{ResolvedValueType, TypeIdentityError};
use crate::shape::{Shape, ShapeExpectation};

/// A fixed-width semantic arena or interface entity category.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum EntityKind {
    /// An input interface entry.
    Input,
    /// An atomic semantic operation.
    Operation,
    /// A semantic value.
    Value,
    /// An output interface entry.
    Output,
}

impl fmt::Display for EntityKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input => formatter.write_str("input"),
            Self::Operation => formatter.write_str("operation"),
            Self::Value => formatter.write_str("value"),
            Self::Output => formatter.write_str("output"),
        }
    }
}

/// A value's role in one transactional builder admission.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ValueRole {
    /// An ordered semantic-operation operand.
    OperationOperand {
        /// Zero-based position in the operation's ordered operand list.
        index: u32,
    },
    /// The value named by a program output declaration.
    ProgramOutput,
}

impl fmt::Display for ValueRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OperationOperand { index } => write!(formatter, "operation operand {index}"),
            Self::ProgramOutput => formatter.write_str("program output value"),
        }
    }
}

/// Failure to create an independent semantic graph owner.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BuilderCreateError {
    /// The process-local graph-owner identifier space is exhausted.
    GraphIdentityExhausted,
    /// The governed standard semantic registry could not be constructed.
    StandardRegistry(RegistryError),
}

impl fmt::Display for BuilderCreateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GraphIdentityExhausted => {
                formatter.write_str("semantic graph identity space is exhausted")
            }
            Self::StandardRegistry(error) => error.fmt(formatter),
        }
    }
}

impl Error for BuilderCreateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::GraphIdentityExhausted => None,
            Self::StandardRegistry(error) => Some(error),
        }
    }
}

/// A graph-owned handle lookup failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HandleError {
    /// The handle belongs to another draft or completed program.
    ForeignGraph {
        /// Kind of entity being looked up.
        entity: EntityKind,
    },
    /// The handle has the right owner but no corresponding local entity.
    InvalidLocal {
        /// Kind of entity being looked up.
        entity: EntityKind,
    },
}

impl fmt::Display for HandleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ForeignGraph { entity } => {
                write!(
                    formatter,
                    "{entity} handle belongs to another semantic graph"
                )
            }
            Self::InvalidLocal { entity } => {
                write!(
                    formatter,
                    "{entity} handle is invalid in this semantic graph"
                )
            }
        }
    }
}

impl Error for HandleError {}

/// A typed incremental construction failure.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BuildError {
    /// An interface key was empty.
    EmptyInterfaceKey {
        /// The interface whose key was empty.
        interface: InterfaceKind,
    },
    /// Two inputs used the same stable key.
    DuplicateInputKey(InputKey),
    /// Two outputs used the same stable key.
    DuplicateOutputKey(OutputKey),
    /// A value handle belonged to another semantic graph.
    ForeignValue {
        /// Role occupied by the rejected value.
        role: ValueRole,
    },
    /// A value handle had the right owner but no corresponding local value.
    InvalidLocalValue {
        /// Role occupied by the rejected value.
        role: ValueRole,
    },
    /// Frozen semantic authority rejected an operation application.
    SemanticRegistry(RegistryError),
    /// Canonical operation attributes exceeded host-owned structural rules.
    InvalidOperationAttributes(TypeIdentityError),
    /// A local Rust marker was not bound in the frozen registry.
    RegistryLookup(RegistryLookupError),
    /// A typed facade could not recover the result evidence promised by its
    /// registered semantic signature.
    Reify(ReifyError),
    /// A typed facade could not revalidate promised result shape evidence.
    ShapeRefinement(ShapeRefineError),
    /// A tensor rank exceeds the fixed-width logical axis space.
    RankTooLarge {
        /// Rejected logical rank.
        rank: usize,
    },
    /// An arena exhausted its fixed-width local identifier space.
    TooManyEntities {
        /// Arena entity kind that exhausted its identifier space.
        entity: EntityKind,
    },
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInterfaceKey { interface } => write!(formatter, "{interface} key is empty"),
            Self::DuplicateInputKey(key) => {
                write!(formatter, "duplicate input key {:?}", key.as_str())
            }
            Self::DuplicateOutputKey(key) => {
                write!(formatter, "duplicate output key {:?}", key.as_str())
            }
            Self::ForeignValue { role } => {
                write!(formatter, "{role} belongs to another semantic graph")
            }
            Self::InvalidLocalValue { role } => {
                write!(formatter, "{role} is invalid in this semantic graph")
            }
            Self::SemanticRegistry(error) => error.fmt(formatter),
            Self::InvalidOperationAttributes(error) => {
                write!(formatter, "invalid operation attributes: {error}")
            }
            Self::RegistryLookup(error) => error.fmt(formatter),
            Self::Reify(error) => error.fmt(formatter),
            Self::ShapeRefinement(error) => error.fmt(formatter),
            Self::RankTooLarge { rank } => {
                write!(
                    formatter,
                    "tensor rank {rank} exceeds the u32 logical axis space"
                )
            }
            Self::TooManyEntities { entity } => write!(formatter, "too many {entity} entities"),
        }
    }
}

impl Error for BuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SemanticRegistry(error) => Some(error),
            Self::InvalidOperationAttributes(error) => Some(error),
            Self::RegistryLookup(error) => Some(error),
            Self::Reify(error) => Some(error),
            Self::ShapeRefinement(error) => Some(error),
            _ => None,
        }
    }
}

/// Failure to attach requested Rust-side shape evidence to a semantic value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ShapeRefineError {
    /// The value identity is foreign or invalid locally.
    Handle(HandleError),
    /// The evidence disagrees with the authoritative graph shape.
    EvidenceMismatch {
        /// Requested evidence, rendered independently of Rust type names.
        expected: ShapeExpectation,
        /// Authoritative shape recorded by the semantic graph.
        actual: Shape,
    },
}

impl fmt::Display for ShapeRefineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Handle(error) => error.fmt(formatter),
            Self::EvidenceMismatch { expected, actual } => write!(
                formatter,
                "requested {expected} does not match authoritative shape {actual}"
            ),
        }
    }
}

impl Error for ShapeRefineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Handle(error) => Some(error),
            Self::EvidenceMismatch { .. } => None,
        }
    }
}

/// Position of a value in an ordered binary shape predicate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ShapeWitnessSubject {
    /// First predicate subject.
    Left,
    /// Second predicate subject.
    Right,
}

impl fmt::Display for ShapeWitnessSubject {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left => formatter.write_str("left witness subject"),
            Self::Right => formatter.write_str("right witness subject"),
        }
    }
}

/// Failure to prove or validate a graph-owned shape relationship.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ShapeWitnessError {
    /// One requested subject is foreign or invalid locally.
    SubjectHandle {
        /// Ordered subject whose handle failed.
        subject: ShapeWitnessSubject,
        /// Underlying graph-handle failure.
        error: HandleError,
    },
    /// Canonical subject shapes do not satisfy equality.
    NotSameShape {
        /// Authoritative left shape.
        left: Shape,
        /// Authoritative right shape.
        right: Shape,
    },
    /// The witness was created by another graph.
    ForeignWitness,
    /// The witness proves the predicate for a different ordered subject pair.
    SubjectMismatch,
}

impl fmt::Display for ShapeWitnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SubjectHandle { subject, error } => write!(formatter, "{subject}: {error}"),
            Self::NotSameShape { left, right } => write!(
                formatter,
                "cannot prove equal shapes for {left} and {right}"
            ),
            Self::ForeignWitness => {
                formatter.write_str("shape witness belongs to another semantic graph")
            }
            Self::SubjectMismatch => {
                formatter.write_str("shape witness proves a different ordered subject pair")
            }
        }
    }
}

impl Error for ShapeWitnessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SubjectHandle { error, .. } => Some(error),
            Self::NotSameShape { .. } | Self::ForeignWitness | Self::SubjectMismatch => None,
        }
    }
}

/// Failure to recover exact static type evidence for an existing value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ReifyError {
    /// The value identity is foreign or invalid locally.
    Handle(HandleError),
    /// The requested Rust marker is not bound in this registry snapshot.
    RegistryLookup(RegistryLookupError),
    /// The marker's exact type does not match the value's authoritative type.
    TypeMismatch {
        /// Type bound to the requested marker.
        expected: Arc<ResolvedValueType>,
        /// Type stored by the semantic value.
        actual: Arc<ResolvedValueType>,
    },
}

impl fmt::Display for ReifyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Handle(error) => error.fmt(formatter),
            Self::RegistryLookup(error) => error.fmt(formatter),
            Self::TypeMismatch { expected, actual } => write!(
                formatter,
                "cannot reify value of type {:?} as marker bound to {:?}",
                actual.canonical_encoding().as_bytes(),
                expected.canonical_encoding().as_bytes()
            ),
        }
    }
}

impl Error for ReifyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Handle(error) => Some(error),
            Self::RegistryLookup(error) => Some(error),
            Self::TypeMismatch { .. } => None,
        }
    }
}

/// One whole-program invariant violation found by [`super::SemanticProgramBuilder::validate`].
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ValidationDiagnostic {
    /// The program has no observable output.
    NoProgramOutputs,
    /// Internal graph state violates a construction invariant.
    InvalidInternalGraph {
        /// Stable, user-facing invariant description.
        reason: &'static str,
    },
}

impl fmt::Display for ValidationDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoProgramOutputs => {
                formatter.write_str("a semantic program requires at least one output")
            }
            Self::InvalidInternalGraph { reason } => {
                write!(formatter, "invalid internal semantic graph: {reason}")
            }
        }
    }
}

/// A nonempty collection of whole-program diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationDiagnostics(Vec<ValidationDiagnostic>);

impl ValidationDiagnostics {
    pub(super) fn new(diagnostics: Vec<ValidationDiagnostic>) -> Option<Self> {
        (!diagnostics.is_empty()).then_some(Self(diagnostics))
    }

    /// Returns diagnostics in deterministic validation order.
    #[must_use]
    pub fn as_slice(&self) -> &[ValidationDiagnostic] {
        &self.0
    }

    /// Consumes the collection and returns its diagnostics.
    #[must_use]
    pub fn into_vec(self) -> Vec<ValidationDiagnostic> {
        self.0
    }
}

impl fmt::Display for ValidationDiagnostics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "semantic program validation failed with {} diagnostic(s)",
            self.0.len()
        )
    }
}

impl Error for ValidationDiagnostics {}

/// The reason a consuming semantic-program commitment failed.
#[derive(Debug)]
#[non_exhaustive]
pub enum ProgramBuildFailure {
    /// Whole-program validation rejected the draft.
    Validation(ValidationDiagnostics),
    /// A distinct completed-program owner could not be allocated.
    GraphIdentityExhausted,
}

impl fmt::Display for ProgramBuildFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(diagnostics) => diagnostics.fmt(formatter),
            Self::GraphIdentityExhausted => {
                formatter.write_str("semantic graph identity space is exhausted")
            }
        }
    }
}

impl Error for ProgramBuildFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Validation(diagnostics) => Some(diagnostics),
            Self::GraphIdentityExhausted => None,
        }
    }
}

/// A terminal validation failure that preserves the original builder.
#[derive(Debug)]
pub struct ProgramBuildError {
    pub(super) builder: Box<super::program::SemanticProgramBuilder>,
    pub(super) failure: ProgramBuildFailure,
}

impl ProgramBuildError {
    /// Returns the exact commitment failure.
    #[must_use]
    pub const fn failure(&self) -> &ProgramBuildFailure {
        &self.failure
    }

    /// Returns validation diagnostics when validation rejected the draft.
    #[must_use]
    pub const fn diagnostics(&self) -> Option<&ValidationDiagnostics> {
        match &self.failure {
            ProgramBuildFailure::Validation(diagnostics) => Some(diagnostics),
            ProgramBuildFailure::GraphIdentityExhausted => None,
        }
    }

    /// Returns the intact builder for inspection before recovery or retry.
    #[must_use]
    pub fn builder(&self) -> &super::program::SemanticProgramBuilder {
        &self.builder
    }

    /// Recovers the original builder for correction and retry.
    #[must_use]
    pub fn into_builder(self) -> super::program::SemanticProgramBuilder {
        *self.builder
    }

    /// Recovers both the original builder and exact failure.
    #[must_use]
    pub fn into_parts(self) -> (super::program::SemanticProgramBuilder, ProgramBuildFailure) {
        (*self.builder, self.failure)
    }
}

impl fmt::Display for ProgramBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.failure.fmt(formatter)
    }
}

impl Error for ProgramBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.failure)
    }
}

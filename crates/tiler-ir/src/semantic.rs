//! Public semantic tensor-program vocabulary.
//!
//! Construction is mutable and transactional. Successful
//! [`SemanticProgramBuilder::build`](crate::semantic::SemanticProgramBuilder::build) performs a
//! one-way, output-reachable compaction into an immutable
//! [`SemanticProgram`](crate::semantic::SemanticProgram).

mod error;
mod handles;
mod identity;
mod interface;
mod operation;
mod program;
mod registry;
mod types;

pub use error::{
    BuildError, BuilderCreateError, EntityKind, HandleError, ProgramBuildError,
    ProgramBuildFailure, ValidationDiagnostic, ValidationDiagnostics, ValueRole,
};
pub use handles::{OperationId, ValueId};
pub use identity::CanonicalIdentity;
pub use interface::{
    InputIndex, InputKey, InterfaceKind, OutputKey, OutputSelector, ProgramInputRef,
    ProgramOutputRef,
};
pub use operation::{
    CANONICAL_F32_ARITHMETIC_NAN_BITS, Definition, OperationKind, OperationRef, ResultIndex,
    ValueRef,
};
pub use program::{SemanticProgram, SemanticProgramBuilder};
pub use registry::{
    CanonicalSemanticRegistryIdentity, F32, FrozenSemanticRegistry, ProviderIdentity,
    RegistryError, RegistryLookupError, SemanticRegistryBuilder, SemanticRegistryProvider,
    SemanticRegistryRegistrar, ValueTypeDefinition, ValueTypeMarker,
};
pub use types::{
    CanonicalEncodedNumericContract, CanonicalResolvedValueType, IdentityComponent, QuantSchemeKey,
    ResolvedValueType, ResolvedValueTypeArgument, ResolvedValueTypeField, TypeIdentityError,
    TypeKey,
};

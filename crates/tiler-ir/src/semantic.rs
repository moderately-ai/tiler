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
    CANONICAL_F32_ARITHMETIC_NAN_BITS, CanonicalValueKind, Definition, F32_CONSTANT_BITS_ATTRIBUTE,
    MAX_OPERATION_ATTRIBUTES, OpKey, OperationArity, OperationAttributeSchema, OperationAttributes,
    OperationConformance, OperationDefinition, OperationDefinitionFacts, OperationEffect,
    OperationInferenceError, OperationInferencer, OperationRef, OperationSchema,
    OperationSchemaError, REDUCTION_AXES_ATTRIBUTE, ResultIndex, ValueFact, ValueRef, add_f32_op,
    constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
};
pub use program::{SemanticProgram, SemanticProgramBuilder};
pub use registry::{
    CanonicalSemanticAuthorityProjection, CanonicalSemanticRegistryIdentity, F32,
    FrozenSemanticRegistry, NormativeDefinitionRef, OperationApplicationRejection,
    ProviderIdentity, RegistryError, RegistryLookupError, SemanticRegistryBuilder,
    SemanticRegistryProvider, SemanticRegistryRegistrar, TypeDefinitionFacts, TypeInstanceError,
    TypeInstanceRejection, ValueTypeDefinition, ValueTypeDefinitionKey, ValueTypeInstanceValidator,
    ValueTypeMarker,
};
pub use types::{
    CanonicalField, CanonicalResolvedValueType, CanonicalValue, CanonicalValueView,
    EncodedNumericContract, IdentityComponent, QuantSchemeKey, ResolvedValueType, TypeArguments,
    TypeIdentityError, TypeKey,
};

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
mod shape_evidence;
mod standard_operations;
mod types;

pub use error::{
    BuildError, BuilderCreateError, EntityKind, HandleError, ProgramBuildError,
    ProgramBuildFailure, ReifyError, ShapeRefineError, ShapeWitnessError, ShapeWitnessSubject,
    ValidationDiagnostic, ValidationDiagnostics, ValueRole,
};
pub use handles::{OperationId, Value, ValueId};
pub use identity::{SemanticGraphIdentity, SemanticIdentity};
pub use interface::{
    InputIndex, InputKey, InterfaceKind, MAX_INTERFACE_KEY_BYTES, Output, OutputKey,
    OutputSelector, ProgramInputRef, ProgramOutputRef, TypedProgramOutputRef,
};
pub use operation::{
    ARITHMETIC_F32_FACT_CANONICAL_NAN_BITS, ARITHMETIC_F32_FACT_CONTRACTION_PERMITTED,
    ARITHMETIC_F32_FACT_ROUNDING, CANONICAL_F32_ARITHMETIC_NAN_BITS, CONFORMANCE_FACT_IDENTITY,
    CONFORMANCE_FACT_VERSION, CONSTANT_F32_FACT_PAYLOAD_RULE, CanonicalOperationAttributes,
    CanonicalValueKind, Definition, F32_CONSTANT_BITS_ATTRIBUTE, F32_TYPE_FACT_CLASS,
    F32_TYPE_FACT_WIDTH_BITS, MAX_OPERATION_ATTRIBUTES, MAX_OPERATION_OPERANDS,
    MAX_OPERATION_RESULTS, MAX_PROVIDER_DIAGNOSTIC_CODE_BYTES,
    MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES, OpKey, OperationAlgebraicCapabilities, OperationArity,
    OperationArityRole, OperationAttributeSchema, OperationAttributes, OperationConformance,
    OperationDefinition, OperationDefinitionFacts, OperationEffect, OperationInferenceError,
    OperationInferenceOutputs, OperationInferenceRequest, OperationInferencer, OperationRef,
    OperationSchema, OperationSchemaError, ProviderDiagnosticCode, ProviderDiagnosticError,
    REDUCTION_AXES_ATTRIBUTE, ResultIndex, SERIAL_SUM_F32_FACT_ACCUMULATION,
    SERIAL_SUM_F32_FACT_CANONICAL_NAN_BITS, SERIAL_SUM_F32_FACT_FOLD_ORDER, ValueFact, ValueRef,
    add_f32_op, constant_f32_op, multiply_f32_op, strict_serial_sum_f32_op,
};
pub use program::{SemanticProgram, SemanticProgramBuilder};
pub(crate) use registry::canonical_f32_bits;
pub use registry::{
    DefinitionValueSubject, F32, FrozenSemanticRegistry, NormativeDefinitionRef,
    OperationApplicationRejection, ProviderIdentity, RegistryError, RegistryLookupError,
    SemanticAdmissionProvenanceIdentity, SemanticAuthorityResource, SemanticCapabilityAuthority,
    SemanticDefinitionProjectionIdentity, SemanticRegistryBuilder, SemanticRegistryProvider,
    SemanticRegistryRegistrar, SemanticRegistryResource, SemanticRegistrySnapshotIdentity,
    TypeDefinitionFacts, TypeInstanceError, TypeInstanceRejection, ValueTypeDefinition,
    ValueTypeDefinitionKey, ValueTypeInstanceValidator, ValueTypeMarker,
};
pub use shape_evidence::{SameShape, ShapePredicate, ShapeWitness, ShapedValue};
pub use standard_operations::{F32Add, F32Constant, F32Multiply, StrictSerialF32Sum};
pub use types::{
    AttributeFieldId, CanonicalField, CanonicalFloatBitsRef, CanonicalIntegerWidth,
    CanonicalResolvedValueType, CanonicalValue, CanonicalValueView, EncodedNumericContract,
    IdentityComponent, QuantSchemeKey, ResolvedValueType, TypeArguments, TypeIdentityError,
    TypeKey,
};

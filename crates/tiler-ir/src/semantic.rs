//! Public semantic tensor-program vocabulary.
//!
//! Construction is mutable and transactional. Successful
//! [`SemanticProgramBuilder::build`](crate::semantic::SemanticProgramBuilder::build) performs a
//! one-way, output-reachable compaction into an immutable
//! [`SemanticProgram`](crate::semantic::SemanticProgram).

mod catalog;
mod contraction;
mod error;
mod handles;
mod identity;
mod interface;
mod operation;
mod precondition;
mod program;
mod quantization;
mod registry;
mod shape_evidence;
mod standard_operations;
mod types;

pub use catalog::{
    admitted_complex_component_types, builtin_scalar_value_types, complex_type_constructor,
    complex_value_type, microscaling_scheme_keys,
};
pub use contraction::{
    CONTRACTION_F32_FACT_ACCUMULATOR_TYPE, CONTRACTION_F32_FACT_ARITHMETIC_CONTRACTION_PERMITTED,
    CONTRACTION_F32_FACT_CANONICAL_NAN_BITS, CONTRACTION_F32_FACT_COMPUTATION_PRECISION,
    CONTRACTION_F32_FACT_CONTRIBUTOR_SEQUENCE, CONTRACTION_F32_FACT_CONVERSION,
    CONTRACTION_F32_FACT_DETERMINISM, CONTRACTION_F32_FACT_DISTRIBUTIVITY,
    CONTRACTION_F32_FACT_EMPTY_CONTRACTED_DOMAIN, CONTRACTION_F32_FACT_NAN_CANONICALIZATION,
    CONTRACTION_F32_FACT_PERMUTATION_PERMITTED, CONTRACTION_F32_FACT_REASSOCIATION_PERMITTED,
    CONTRACTION_F32_FACT_RESULT_TYPE, CONTRACTION_F32_FACT_SEED,
    CONTRACTION_INDEX_STRUCTURE_ATTRIBUTE, CONTRACTION_STRUCTURE_CONTRACTED_INDICES,
    CONTRACTION_STRUCTURE_OPERAND_INDICES, CONTRACTION_STRUCTURE_OUTPUT_INDICES,
    CanonicalContractionIndexStructure, ContractionAttributeSubject, ContractionIndex,
    ContractionIndexStructure, ContractionStructureError, MAX_CONTRACTION_OPERANDS,
    MAX_CONTRACTION_TUPLE_INDICES, strict_tensor_contraction_f32_op,
};
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
    CanonicalValueKind, Definition, F32_CONSTANT_BITS_ATTRIBUTE, MAX_OPERATION_ATTRIBUTES,
    MAX_OPERATION_OPERANDS, MAX_OPERATION_RESULTS, MAX_PROVIDER_DIAGNOSTIC_CODE_BYTES,
    MAX_PROVIDER_DIAGNOSTIC_MESSAGE_BYTES, OpKey, OperationAlgebraicCapabilities, OperationArity,
    OperationArityRole, OperationAttributeSchema, OperationAttributes, OperationConformance,
    OperationDefinition, OperationDefinitionFacts, OperationEffect, OperationInferenceError,
    OperationInferenceOutputs, OperationInferenceRequest, OperationInferencer, OperationRef,
    OperationSchema, OperationSchemaError, ProviderDiagnosticCode, ProviderDiagnosticError,
    REDUCTION_AXES_ATTRIBUTE, ResultIndex, SCALAR_TYPE_FACT_ALIAS_POLICY,
    SCALAR_TYPE_FACT_BLOCK_SIZE, SCALAR_TYPE_FACT_CLASS, SCALAR_TYPE_FACT_COEFFICIENT_DIGITS,
    SCALAR_TYPE_FACT_COMPONENT_ORDER, SCALAR_TYPE_FACT_COMPONENT_TYPES,
    SCALAR_TYPE_FACT_EXPONENT_BIAS, SCALAR_TYPE_FACT_EXPONENT_BITS,
    SCALAR_TYPE_FACT_HAS_INFINITIES, SCALAR_TYPE_FACT_HAS_NAN, SCALAR_TYPE_FACT_HAS_SIGNED_ZERO,
    SCALAR_TYPE_FACT_HAS_SUBNORMALS, SCALAR_TYPE_FACT_HAS_ZERO, SCALAR_TYPE_FACT_SCALE_SELECTION,
    SCALAR_TYPE_FACT_SIGN_BITS, SCALAR_TYPE_FACT_TRAILING_SIGNIFICAND_BITS,
    SCALAR_TYPE_FACT_VALUE_CARDINALITY, SCALAR_TYPE_FACT_WIDTH_BITS,
    SERIAL_SUM_F32_FACT_ACCUMULATION, SERIAL_SUM_F32_FACT_CANONICAL_NAN_BITS,
    SERIAL_SUM_F32_FACT_FOLD_ORDER, ValueFact, ValueRef, add_f32_op, constant_f32_op,
    multiply_f32_op, strict_serial_sum_f32_op,
};
pub use precondition::{
    MAX_OPERATION_SEMANTIC_PRECONDITIONS, MAX_SEMANTIC_PRECONDITION_OBLIGATION_IDENTITY_BYTES,
    OperationOperandIndex, SemanticInvalidInputCode, SemanticLogicalView,
    SemanticPreconditionDeclaration, SemanticPreconditionDeclarationError,
    SemanticPreconditionDeclarations, SemanticPreconditionDisproof,
    SemanticPreconditionObligationIdentity, SemanticPreconditionOrdinal,
    SemanticPreconditionProofBasis, SemanticPreconditionRef, SemanticPreconditionStatus,
    SemanticPredicateIdentity, no_nan_predicate, positive_finite_scalar_predicate,
};
pub use program::{SemanticProgram, SemanticProgramBuilder};
pub use quantization::{
    ENCODED_NUMERIC_CODE_MAX, ENCODED_NUMERIC_CODE_MIN, ENCODED_NUMERIC_CODE_TYPE,
    ENCODED_NUMERIC_COMPUTE_TYPE, ENCODED_NUMERIC_DECODE_EVALUATION,
    ENCODED_NUMERIC_EXPRESSED_TYPE, ENCODED_NUMERIC_MATERIALIZATION, ENCODED_NUMERIC_NAN_BEHAVIOR,
    ENCODED_NUMERIC_ROUNDING, ENCODED_NUMERIC_SATURATION, STRICT_AFFINE_CODES_ROLE,
    STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE, StrictAffineU4, StrictAffineU8, U4,
    U8, assemble_strict_affine_op, dequantize_strict_affine_op, quantize_strict_affine_op,
    strict_affine_scheme,
};
pub(crate) use registry::canonical_f32_bits;
pub use registry::{
    CanonicalValueTypeDescriptor, DefinitionValueSubject, F32, FrozenSemanticRegistry,
    NormativeDefinitionRef, OperationApplicationRejection, ProviderIdentity, RegistryError,
    RegistryLookupError, SemanticAdmissionProvenanceIdentity, SemanticAuthorityResource,
    SemanticCapabilityAuthority, SemanticDefinitionProjectionIdentity, SemanticRegistryBuilder,
    SemanticRegistryProvider, SemanticRegistryRegistrar, SemanticRegistryResource,
    SemanticRegistrySnapshotIdentity, TypeDefinitionFacts, TypeInstanceError,
    TypeInstanceRejection, ValueTypeDefinition, ValueTypeDefinitionKey, ValueTypeInstanceValidator,
    ValueTypeMarker,
};
pub use shape_evidence::{SameShape, ShapePredicate, ShapeWitness, ShapedValue};
pub use standard_operations::{
    F32Add, F32Constant, F32Multiply, F32TensorContraction, StrictSerialF32Sum,
};
pub use types::{
    AttributeFieldId, CanonicalField, CanonicalFloatBitsRef, CanonicalIntegerWidth,
    CanonicalResolvedValueType, CanonicalValue, CanonicalValueView, EncodedComponentDeclaration,
    EncodedComponentRole, EncodedComponentShape, EncodedNumericContract, IdentityComponent,
    ParameterIndexMap, QuantSchemeKey, ResolvedValueType, TypeArguments, TypeIdentityError,
    TypeKey,
};

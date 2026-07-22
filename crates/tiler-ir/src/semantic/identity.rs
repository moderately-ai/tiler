use super::operation::{OperationData, ValueDefinition};
use super::program::ProgramData;
use super::registry::{
    SemanticAdmissionProvenanceIdentity, SemanticDefinitionProjectionIdentity,
    SemanticRegistrySnapshotIdentity,
};
use crate::shape::Shape;

pub(super) const MAX_SEMANTIC_PROGRAM_CANONICAL_WORK_BYTES: usize = 16 * 1024 * 1024;
const LENGTH_BYTES: usize = std::mem::size_of::<u64>();
const VALUE_ID_BYTES: usize = std::mem::size_of::<u64>();
const RESULT_INDEX_BYTES: usize = std::mem::size_of::<u32>();
const EXTENT_BYTES: usize = std::mem::size_of::<u64>();
const GRAPH_DOMAIN: &[u8] = b"tiler.semantic-graph.v2\0";

/// Collision-free canonical semantic-graph identity bytes.
///
/// This identifies graph meaning. Provider implementations, registry snapshots,
/// and compilation provenance are deliberately excluded.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SemanticGraphIdentity(Vec<u8>);

impl SemanticGraphIdentity {
    /// Returns the canonical byte encoding.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Complete, non-forgeable semantic identity of one checked program.
///
/// The four subjects remain separately typed because they answer different
/// equality questions, but this owner prevents downstream code from assembling
/// components from different programs.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SemanticIdentity {
    graph: SemanticGraphIdentity,
    reached_definitions: SemanticDefinitionProjectionIdentity,
    admission_provenance: SemanticAdmissionProvenanceIdentity,
    registry_snapshot: SemanticRegistrySnapshotIdentity,
}

impl SemanticIdentity {
    pub(super) fn new(
        graph: SemanticGraphIdentity,
        reached_definitions: SemanticDefinitionProjectionIdentity,
        admission_provenance: SemanticAdmissionProvenanceIdentity,
        registry_snapshot: SemanticRegistrySnapshotIdentity,
    ) -> Self {
        Self {
            graph,
            reached_definitions,
            admission_provenance,
            registry_snapshot,
        }
    }

    /// Returns canonical graph meaning and ordered-interface identity.
    #[must_use]
    pub const fn graph(&self) -> &SemanticGraphIdentity {
        &self.graph
    }

    /// Returns all provider-independent semantic definitions reached by the program.
    #[must_use]
    pub const fn reached_definitions(&self) -> &SemanticDefinitionProjectionIdentity {
        &self.reached_definitions
    }

    /// Returns provider-attributed provenance for all reached semantic authority.
    #[must_use]
    pub const fn admission_provenance(&self) -> &SemanticAdmissionProvenanceIdentity {
        &self.admission_provenance
    }

    /// Returns provenance for the complete registry snapshot used for validation.
    #[must_use]
    pub const fn registry_snapshot(&self) -> &SemanticRegistrySnapshotIdentity {
        &self.registry_snapshot
    }
}

pub(super) fn compute_graph_identity(program: &ProgramData) -> SemanticGraphIdentity {
    let traversal = canonical_traversal(program);
    let encoded_len = graph_identity_encoded_len(program, &traversal);
    debug_assert_eq!(encoded_len, program.graph_identity_encoded_len);
    assert!(
        encoded_len <= program.canonical_work_bytes
            && encoded_len <= MAX_SEMANTIC_PROGRAM_CANONICAL_WORK_BYTES,
        "verified graph identity exceeds its admitted canonical-work budget"
    );
    let mut bytes = Vec::with_capacity(encoded_len);
    bytes.extend_from_slice(GRAPH_DOMAIN);
    encode_len(&mut bytes, program.inputs.len());
    for input in &program.inputs {
        encode_string(&mut bytes, input.key.as_str());
        let value = &program.values[input.value.as_usize()];
        value.resolved_type.encode(&mut bytes);
        encode_shape(&mut bytes, &value.shape);
    }
    encode_len(&mut bytes, traversal.operation_order.len());
    for operation_index in traversal.operation_order {
        let operation = &program.operations[operation_index.as_usize()];
        encode_len(&mut bytes, operation_record_encoded_len(program, operation));
        encode_operation(&mut bytes, operation);
        encode_len(&mut bytes, operation.operands.len());
        for operand in &operation.operands {
            let operand_id = traversal.canonical_ids[operand.as_usize()]
                .expect("canonical traversal assigns every reached operand");
            bytes.extend_from_slice(&operand_id.to_be_bytes());
        }
        encode_len(&mut bytes, operation.results.len());
        for result in &operation.results {
            let value_data = &program.values[result.as_usize()];
            let ValueDefinition::OperationResult { result_index, .. } = value_data.definition
            else {
                unreachable!("verified operation result has an operation definition")
            };
            bytes.extend_from_slice(&result_index.get().to_be_bytes());
            value_data.resolved_type.encode(&mut bytes);
            encode_shape(&mut bytes, &value_data.shape);
        }
    }
    encode_len(&mut bytes, program.outputs.len());
    for (output, canonical_id) in program.outputs.iter().zip(traversal.output_ids) {
        encode_string(&mut bytes, output.key.as_str());
        bytes.extend_from_slice(&canonical_id.to_be_bytes());
    }
    debug_assert_eq!(bytes.len(), encoded_len);
    SemanticGraphIdentity(bytes)
}

struct CanonicalTraversal {
    canonical_ids: Vec<Option<u64>>,
    operation_order: Vec<super::handles::OperationIndex>,
    output_ids: Vec<u64>,
}

fn canonical_traversal(program: &ProgramData) -> CanonicalTraversal {
    let mut canonical_ids = vec![None; program.values.len()];
    let mut encoded_operations = vec![false; program.operations.len()];
    let mut next_value_id = u64::try_from(program.inputs.len()).expect("entity count fits u64");

    for (position, input) in program.inputs.iter().enumerate() {
        canonical_ids[input.value.as_usize()] =
            Some(u64::try_from(position).expect("usize fits u64"));
    }

    let mut operation_order = Vec::with_capacity(program.operations.len());
    let mut output_ids = Vec::with_capacity(program.outputs.len());
    for output in &program.outputs {
        output_ids.push(visit_value(
            program,
            output.value,
            &mut canonical_ids,
            &mut encoded_operations,
            &mut next_value_id,
            &mut operation_order,
        ));
    }
    CanonicalTraversal {
        canonical_ids,
        operation_order,
        output_ids,
    }
}

fn visit_value(
    program: &ProgramData,
    value: super::handles::ValueIndex,
    canonical_ids: &mut [Option<u64>],
    encoded_operations: &mut [bool],
    next_value_id: &mut u64,
    operation_order: &mut Vec<super::handles::OperationIndex>,
) -> u64 {
    enum Work {
        Enter(super::handles::ValueIndex),
        Exit(super::handles::OperationIndex),
    }

    let mut work = vec![Work::Enter(value)];
    while let Some(item) = work.pop() {
        match item {
            Work::Enter(current) => {
                if canonical_ids[current.as_usize()].is_some() {
                    continue;
                }
                let ValueDefinition::OperationResult { operation, .. } =
                    program.values[current.as_usize()].definition
                else {
                    unreachable!("verified input value has a canonical ID")
                };
                if encoded_operations[operation.as_usize()] {
                    continue;
                }
                work.push(Work::Exit(operation));
                work.extend(
                    program.operations[operation.as_usize()]
                        .operands
                        .iter()
                        .rev()
                        .copied()
                        .map(Work::Enter),
                );
            }
            Work::Exit(operation_index) => {
                if encoded_operations[operation_index.as_usize()] {
                    continue;
                }
                let operation = &program.operations[operation_index.as_usize()];
                operation_order.push(operation_index);
                for result in &operation.results {
                    canonical_ids[result.as_usize()] = Some(*next_value_id);
                    *next_value_id = next_value_id
                        .checked_add(1)
                        .expect("verified entity count fits u64");
                }
                encoded_operations[operation_index.as_usize()] = true;
            }
        }
    }
    canonical_ids[value.as_usize()].expect("worklist assigns the requested value")
}

pub(super) fn empty_graph_canonical_work_bytes() -> usize {
    GRAPH_DOMAIN.len().saturating_add(3 * LENGTH_BYTES)
}

pub(super) fn input_canonical_work_bytes(
    key: &super::interface::InputKey,
    resolved_type: &super::types::ResolvedValueType,
    shape: &Shape,
) -> usize {
    string_encoded_len(key.as_str())
        .saturating_add(resolved_type.encoded_len())
        .saturating_add(shape_encoded_len(shape))
}

pub(super) fn operation_canonical_work_bytes(
    key: &super::operation::OpKey,
    attributes: &super::operation::OperationAttributes,
    operand_count: usize,
    results: &[super::operation::ValueFact],
) -> usize {
    LENGTH_BYTES.saturating_add(
        key.encoded_len()
            .saturating_add(attributes.encoded_len())
            .saturating_add(LENGTH_BYTES)
            .saturating_add(operand_count.saturating_mul(VALUE_ID_BYTES))
            .saturating_add(LENGTH_BYTES)
            .saturating_add(
                results
                    .iter()
                    .map(|fact| {
                        RESULT_INDEX_BYTES
                            .saturating_add(fact.resolved_type().encoded_len())
                            .saturating_add(shape_encoded_len(fact.shape()))
                    })
                    .fold(0_usize, usize::saturating_add),
            ),
    )
}

pub(super) fn output_canonical_work_bytes(key: &super::interface::OutputKey) -> usize {
    string_encoded_len(key.as_str()).saturating_add(VALUE_ID_BYTES)
}

pub(super) fn graph_identity_encoded_len_for_verified(program: &ProgramData) -> usize {
    let traversal = canonical_traversal(program);
    graph_identity_encoded_len(program, &traversal)
}

fn graph_identity_encoded_len(program: &ProgramData, traversal: &CanonicalTraversal) -> usize {
    GRAPH_DOMAIN
        .len()
        .saturating_add(LENGTH_BYTES)
        .saturating_add(
            program
                .inputs
                .iter()
                .map(|input| {
                    let value = &program.values[input.value.as_usize()];
                    input_canonical_work_bytes(&input.key, &value.resolved_type, &value.shape)
                })
                .fold(0_usize, usize::saturating_add),
        )
        .saturating_add(LENGTH_BYTES)
        .saturating_add(
            traversal
                .operation_order
                .iter()
                .map(|index| {
                    LENGTH_BYTES.saturating_add(operation_record_encoded_len(
                        program,
                        &program.operations[index.as_usize()],
                    ))
                })
                .fold(0_usize, usize::saturating_add),
        )
        .saturating_add(LENGTH_BYTES)
        .saturating_add(
            program
                .outputs
                .iter()
                .map(|output| output_canonical_work_bytes(&output.key))
                .fold(0_usize, usize::saturating_add),
        )
}

fn operation_record_encoded_len(program: &ProgramData, operation: &OperationData) -> usize {
    operation
        .key
        .encoded_len()
        .saturating_add(operation.attributes.encoded_len())
        .saturating_add(LENGTH_BYTES)
        .saturating_add(operation.operands.len().saturating_mul(VALUE_ID_BYTES))
        .saturating_add(LENGTH_BYTES)
        .saturating_add(
            operation
                .results
                .iter()
                .map(|result| {
                    let value = &program.values[result.as_usize()];
                    RESULT_INDEX_BYTES
                        .saturating_add(value.resolved_type.encoded_len())
                        .saturating_add(shape_encoded_len(&value.shape))
                })
                .fold(0_usize, usize::saturating_add),
        )
}

fn string_encoded_len(value: &str) -> usize {
    LENGTH_BYTES.saturating_add(value.len())
}

fn shape_encoded_len(shape: &Shape) -> usize {
    LENGTH_BYTES.saturating_add(shape.rank().saturating_mul(EXTENT_BYTES))
}

fn encode_operation(output: &mut Vec<u8>, operation: &OperationData) {
    operation.key.encode(output);
    operation.attributes.encode(output);
}

fn encode_len(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(
        &u64::try_from(value)
            .expect("supported usize fits u64")
            .to_be_bytes(),
    );
}

fn encode_string(output: &mut Vec<u8>, value: &str) {
    encode_len(output, value.len());
    output.extend_from_slice(value.as_bytes());
}

fn encode_shape(output: &mut Vec<u8>, shape: &Shape) {
    encode_len(output, shape.rank());
    for extent in shape.extents() {
        output.extend_from_slice(&extent.get().to_be_bytes());
    }
}

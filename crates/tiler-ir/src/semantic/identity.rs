use super::SemanticProgram;
use super::operation::{CANONICAL_F32_ARITHMETIC_NAN_BITS, OperationKind, ValueDefinition};
use super::program::ProgramData;
use crate::shape::Shape;

/// Collision-free canonical semantic identity bytes for the bounded prototype.
///
/// This is a canonical encoding, not a cryptographic digest or artifact codec.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CanonicalIdentity(Vec<u8>);

impl CanonicalIdentity {
    /// Returns the canonical byte encoding.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl SemanticProgram {
    /// Produces a deterministic semantic identity encoding.
    ///
    /// Reachable sharing, ordered interfaces, shapes, exact float bits, operand
    /// order, and reduction axes participate. Runtime graph IDs, arena numbering,
    /// and dead operations do not.
    #[must_use]
    pub fn canonical_identity(&self) -> &CanonicalIdentity {
        self.data
            .identity
            .get_or_init(|| compute_identity(&self.data))
    }
}

fn compute_identity(program: &ProgramData) -> CanonicalIdentity {
    let mut records = Vec::new();
    let mut canonical_ids = vec![None; program.values.len()];

    for (position, input) in program.inputs.iter().enumerate() {
        canonical_ids[input.value.as_usize()] =
            Some(u64::try_from(position).expect("usize fits u64"));
    }

    let mut output_ids = Vec::with_capacity(program.outputs.len());
    for output in &program.outputs {
        output_ids.push(visit_value(
            program,
            output.value,
            &mut canonical_ids,
            &mut records,
        ));
    }

    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"tiler.semantic.f32.v3\0");
    bytes.extend_from_slice(b"tiler::canonical-arithmetic-nan-f32@1\0");
    bytes.extend_from_slice(&CANONICAL_F32_ARITHMETIC_NAN_BITS.to_be_bytes());
    encode_len(&mut bytes, program.inputs.len());
    for input in &program.inputs {
        encode_string(&mut bytes, input.key.as_str());
        let value = &program.values[input.value.as_usize()];
        value.resolved_type.encode(&mut bytes);
        encode_shape(&mut bytes, &value.shape);
    }
    encode_len(&mut bytes, records.len());
    for record in records {
        encode_len(&mut bytes, record.len());
        bytes.extend_from_slice(&record);
    }
    encode_len(&mut bytes, program.outputs.len());
    for (output, canonical_id) in program.outputs.iter().zip(output_ids) {
        encode_string(&mut bytes, output.key.as_str());
        bytes.extend_from_slice(&canonical_id.to_be_bytes());
    }
    CanonicalIdentity(bytes)
}

fn visit_value(
    program: &ProgramData,
    value: super::handles::ValueIndex,
    canonical_ids: &mut [Option<u64>],
    records: &mut Vec<Vec<u8>>,
) -> u64 {
    enum Work {
        Enter(super::handles::ValueIndex),
        Exit(super::handles::ValueIndex),
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
                work.push(Work::Exit(current));
                work.extend(
                    program.operations[operation.as_usize()]
                        .operands
                        .iter()
                        .rev()
                        .copied()
                        .map(Work::Enter),
                );
            }
            Work::Exit(current) => {
                if canonical_ids[current.as_usize()].is_some() {
                    continue;
                }
                let value_data = &program.values[current.as_usize()];
                let ValueDefinition::OperationResult {
                    operation,
                    result_index,
                } = value_data.definition
                else {
                    unreachable!("verified input value has a canonical ID")
                };
                let operation = &program.operations[operation.as_usize()];
                let id = u64::try_from(program.inputs.len() + records.len())
                    .expect("entity count fits u64");
                let mut record = Vec::new();
                encode_operation(&mut record, &operation.kind);
                encode_len(&mut record, operation.operands.len());
                for operand in &operation.operands {
                    let operand_id = canonical_ids[operand.as_usize()]
                        .expect("postorder visits operands before their consumer");
                    record.extend_from_slice(&operand_id.to_be_bytes());
                }
                record.extend_from_slice(&result_index.get().to_be_bytes());
                value_data.resolved_type.encode(&mut record);
                encode_shape(&mut record, &value_data.shape);
                records.push(record);
                canonical_ids[current.as_usize()] = Some(id);
            }
        }
    }
    canonical_ids[value.as_usize()].expect("worklist assigns the requested value")
}

fn encode_operation(output: &mut Vec<u8>, kind: &OperationKind) {
    match kind {
        OperationKind::ConstantF32 { bits } => {
            output.push(1);
            output.extend_from_slice(&bits.to_be_bytes());
        }
        OperationKind::MultiplyF32 => output.push(2),
        OperationKind::AddF32 => output.push(3),
        OperationKind::StrictSerialSumF32 { axes } => {
            output.push(4);
            encode_len(output, axes.len());
            for axis in axes {
                output.extend_from_slice(&axis.get().to_be_bytes());
            }
        }
    }
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

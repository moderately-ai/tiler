//! The serial-Sum value proof, carried end to end onto real hardware.
//!
//! One declarative tensor program goes through every stage in one process:
//! semantic construction, compilation through the public compiler boundary,
//! MSL emission, offline compilation to a real `metallib` by `xcrun`, loading
//! that library onto this machine's GPU, dispatch, and a bit-for-bit comparison
//! of the device's output against `tiler-reference`'s independent evaluation.
//!
//! # What makes the comparison worth anything
//!
//! The reference evaluator shares no code with the compiler's lowering, the
//! emitter, or the kernel. It evaluates the *semantic* program directly. An
//! agreement here is therefore two independent implementations of one declared
//! contract arriving at the same bits, not one implementation checked against
//! itself.
//!
//! The comparison is on exact bit patterns rather than an epsilon. The program
//! declares a numerical contract; a result that is close but not equal has
//! violated it, and reporting that as success would make the contract
//! decorative.
//!
//! # The contract this runs under
//!
//! `NumericalContract::FlushSubnormalsToZeroF32`, stated rather than defaulted.
//! Apple `f32` arithmetic flushes subnormals to the sign-preserving zero in
//! every math mode, so the strict contract is not deliverable here and emission
//! refuses it. Stating the flush contract makes running on this hardware a
//! choice this program made about what it means; the two contracts carry
//! different keys and different identities.

mod buffer;

use std::process::ExitCode;

use metal::{
    ComputePipelineDescriptor, Device, MTLCommandBufferStatus, MTLResourceOptions, MTLSize,
};
use tiler_compiler::session::{CompileFailure, NumericalContract, compile_governed};
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};
use tiler_metal::emit::emit_translation_unit;
use tiler_metal::target::{
    LaunchIndexRealization, MetalDeploymentMinimum, MetalFlushedZeroSign, MetalPlatform,
    MetalSubnormalArithmetic, MetalTargetFacts, MslLanguageVersion,
};
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::input::{
    AppleSdk, CompileRequest, DeploymentMinimum, MetalTarget, MslVersion, NumericalRealization,
    OptimizationLevel,
};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

/// Rows of the proof's input; each row reduces to one output element.
const ROWS: u64 = 4;
/// Columns of the proof's input; the reduced axis.
const COLUMNS: u64 = 3;
/// Buffer argument-table capacity Apple states per compute function.
const BUFFER_BINDING_LIMIT: u32 = 31;

/// The exact input bits this proof reduces.
///
/// Chosen to exercise the contract rather than to be arithmetically convenient:
/// a negative zero, the least positive subnormal, a non-canonical NaN payload,
/// and an infinity all appear, because those are the values where a numerical
/// contract either holds or is decorative.
fn input_bits() -> Vec<u32> {
    vec![
        0x3f80_0000,
        0x4000_0000,
        0x4040_0000, // 1.0, 2.0, 3.0
        0x8000_0000,
        0x0000_0001,
        0x3f80_0000, // -0.0, least subnormal, 1.0
        0x7fc0_1234,
        0x3f80_0000,
        0x4000_0000, // non-canonical NaN, 1.0, 2.0
        0x7f80_0000,
        0x3f80_0000,
        0xbf80_0000, // +inf, 1.0, -1.0
    ]
}

fn target_facts() -> MetalTargetFacts {
    MetalTargetFacts::new(
        MslLanguageVersion::Metal3_1,
        MetalPlatform::MacOs,
        MetalDeploymentMinimum::new(13, 0),
        LaunchIndexRealization::ThreadPositionInGridUInt,
        MetalSubnormalArithmetic::FlushesToZero {
            zero_sign: MetalFlushedZeroSign::PreservesSign,
        },
        BUFFER_BINDING_LIMIT,
    )
}

/// Builds `sum((input * 1.0) + 0.0)` over the reduced axis.
fn serial_sum_program() -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
            Shape::from_dims([ROWS, COLUMNS]),
        )
        .expect("the input binds");
    let scale = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).expect("the scale applies");
    let bias = F32Constant::apply(&mut builder, 0.0_f32.to_bits()).expect("the bias applies");
    let product = F32Multiply::apply(&mut builder, input, scale).expect("the product applies");
    let mapped = F32Add::apply(&mut builder, product, bias).expect("the bias applies");
    let sum =
        StrictSerialF32Sum::apply(&mut builder, mapped, [Axis::new(1)]).expect("the sum applies");
    builder
        .output(
            OutputKey::new("result").expect("the output key is valid"),
            sum,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Evaluates the same semantic program through the independent oracle.
fn reference_bits(program: &SemanticProgram, bits: &[u32]) -> Vec<u32> {
    let key = InputKey::new("input").expect("the input key is valid");
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

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("serial-sum runtime proof failed: {failure}");
            ExitCode::FAILURE
        }
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "the proof is one linear narrative from a semantic program to compared bits; splitting it would hide the ordering that is its point"
)]
fn run() -> Result<(), ProofError> {
    let program = serial_sum_program();
    let bits = input_bits();

    let compilations = compile_governed(&program, NumericalContract::FlushSubnormalsToZeroF32)
        .map_err(ProofError::Compile)?;
    let compilation = compilations.first().ok_or(ProofError::NoTarget)?;
    let selected = compilation.selected().ok_or(ProofError::NoSelection)?;
    println!("selected alternative: {}", selected.stable_id());

    let facts = target_facts();
    let kernels: Vec<_> = selected.kernels().iter().collect();
    let unit = emit_translation_unit(&kernels, &facts).map_err(|_| ProofError::Emit)?;
    // Emission succeeds even when the target cannot honour the declared
    // contract, so conformance is asked explicitly rather than inferred.
    unit.require_declared_realization()
        .map_err(|_| ProofError::UnrealizableNumerics)?;

    let request = CompileRequest::new(
        unit.source(),
        MetalTarget::new(
            AppleSdk::MacOs,
            DeploymentMinimum::new(13, 0),
            MslVersion::Metal3_1,
        ),
        OptimizationLevel::Default,
        NumericalRealization::strict_baseline(),
    );
    let artifact = Toolchain::system()
        .compile(&request)
        .map_err(|_| ProofError::Toolchain)?;
    println!("compiled {} bytes of metallib", artifact.metallib.len());

    // ---- loading -------------------------------------------------------
    let device = Device::system_default().ok_or(ProofError::NoDevice)?;
    println!("device: {}", device.name());
    let library = device
        .new_library_with_data(&artifact.metallib)
        .map_err(ProofError::LibraryLoad)?;
    let entry = unit.entry_points().first().ok_or(ProofError::Emit)?;
    let function = library
        .get_function(entry.symbol(), None)
        .map_err(ProofError::FunctionLookup)?;
    let descriptor = ComputePipelineDescriptor::new();
    descriptor.set_compute_function(Some(&function));
    let pipeline = device
        .new_compute_pipeline_state(&descriptor)
        .map_err(ProofError::Pipeline)?;

    // ---- dispatch ------------------------------------------------------
    let element_count =
        usize::try_from(ROWS * COLUMNS).expect("the proof's element count fits a usize");
    let output_count = usize::try_from(ROWS).expect("the proof's row count fits a usize");
    let input_buffer = device.new_buffer(
        element_count as u64 * 4,
        MTLResourceOptions::StorageModeShared,
    );
    let output_buffer = device.new_buffer(
        output_count as u64 * 4,
        MTLResourceOptions::StorageModeShared,
    );
    let operands: Vec<f32> = bits.iter().map(|value| f32::from_bits(*value)).collect();
    buffer::write_f32(&input_buffer, &operands);

    let queue = device.new_command_queue();
    let command_buffer = queue.new_command_buffer();
    let encoder = command_buffer.new_compute_command_encoder();
    encoder.set_compute_pipeline_state(&pipeline);
    encoder.set_buffer(0, Some(&input_buffer), 0);
    encoder.set_buffer(1, Some(&output_buffer), 0);
    encoder.dispatch_threads(
        MTLSize::new(ROWS, 1, 1),
        MTLSize::new(pipeline.thread_execution_width().min(ROWS), 1, 1),
    );
    encoder.end_encoding();
    command_buffer.commit();
    command_buffer.wait_until_completed();

    // The command buffer's terminal state is checked *before* the host reads
    // anything back. A failed submission leaves the output buffer holding
    // whatever it held before, and comparing that against the reference would
    // report a numerical disagreement for what is actually a dispatch failure.
    let status = command_buffer.status();
    if status != MTLCommandBufferStatus::Completed {
        return Err(ProofError::Dispatch(format!("{status:?}")));
    }

    // ---- numerical verification ----------------------------------------
    let device_bits: Vec<u32> = buffer::read_f32(&output_buffer, output_count)
        .iter()
        .map(|value| value.to_bits())
        .collect();
    let expected = reference_bits(&program, &bits);

    println!("device:    {device_bits:08x?}");
    println!("reference: {expected:08x?}");
    if device_bits == expected {
        println!("bit-for-bit agreement on {output_count} output element(s)");
        Ok(())
    } else {
        Err(ProofError::Mismatch {
            device: device_bits,
            reference: expected,
        })
    }
}

/// Why one end-to-end proof did not complete.
///
/// The stages stay apart: a program this build cannot compile, a target that
/// cannot honour the contract, a missing toolchain, a missing device, a failed
/// dispatch, and a numerical disagreement are six different things to do next,
/// and only the last is a claim about arithmetic.
#[derive(Debug)]
enum ProofError {
    Compile(CompileFailure),
    NoTarget,
    NoSelection,
    Emit,
    UnrealizableNumerics,
    Toolchain,
    NoDevice,
    LibraryLoad(String),
    FunctionLookup(String),
    Pipeline(String),
    Dispatch(String),
    Mismatch {
        device: Vec<u32>,
        reference: Vec<u32>,
    },
}

impl std::fmt::Display for ProofError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Compile(failure) => write!(formatter, "the program did not compile: {failure:?}"),
            Self::NoTarget => formatter.write_str("the compilation returned no target profile"),
            Self::NoSelection => formatter.write_str("the portfolio retained no selected plan"),
            Self::Emit => formatter.write_str("the selected kernels have no Metal realization"),
            Self::UnrealizableNumerics => formatter
                .write_str("the target cannot honour the kernels' declared numerical contract"),
            Self::Toolchain => formatter.write_str("the offline toolchain produced no metallib"),
            Self::NoDevice => formatter.write_str("no system default Metal device"),
            Self::LibraryLoad(cause) => write!(formatter, "the metallib did not load: {cause}"),
            Self::FunctionLookup(cause) => write!(formatter, "the entry point is absent: {cause}"),
            Self::Pipeline(cause) => write!(formatter, "no compute pipeline state: {cause}"),
            Self::Dispatch(cause) => write!(formatter, "the command buffer failed: {cause}"),
            Self::Mismatch { device, reference } => write!(
                formatter,
                "device returned {device:08x?}, reference requires {reference:08x?}",
            ),
        }
    }
}

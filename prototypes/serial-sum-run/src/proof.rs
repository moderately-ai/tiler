//! The serial-Sum value proof, carried end to end onto real hardware.
//!
//! One declarative tensor program is dispatched **twice** on this machine's GPU
//! and both results are compared bit for bit against `tiler-reference`'s
//! independent evaluation of the same semantic program.
//!
//! # The two paths, and why both are kept
//!
//! **The direct path** compiles the program here, emits MSL, compiles it to a
//! `metallib` with `xcrun`, and hands those in-memory bytes straight to
//! `newLibraryWithData:`. Nothing is packaged, encoded, decoded, or validated.
//! It is evidence about the *compiler and the emitter*.
//!
//! **The envelope path** reads an artifact `prototypes/serial-sum-compile`
//! wrote to a file and dispatches from that alone: `tiler-runtime` decodes it,
//! discharges every host obligation, commits a route, and the device loads the
//! object bytes *the envelope carries*. The entry symbol, the argument-table
//! index of every buffer, the bytes each must reach, and the launch geometry all
//! come from the decoded dispatch record. Nothing this process compiled reaches
//! the device on this path. It is evidence about the *delivery mechanism*.
//!
//! Keeping both is what makes a disagreement diagnosable. If the direct path
//! matches the reference and the envelope path does not, the envelope is at
//! fault; if both fail together, the compiler is. Collapsing them would leave
//! only "the bits are wrong".
//!
//! # Usage
//!
//! ```text
//! cargo run -p tiler-prototype-compile -- --out /tmp/serial-sum.tiler
//! cargo run -p tiler-prototype-run     -- --artifact /tmp/serial-sum.tiler
//! ```
//!
//! The artifact path is required. A run that silently skipped the envelope path
//! because a file was missing would report success for half a proof, so a
//! missing or unreadable artifact is a hard failure naming the producer command
//! that creates it.
//!
//! # What binds the two processes together
//!
//! Three checks, and they are independent of one another. [`decode_artifact`]
//! re-derives the artifact's identity from its own content and refuses on
//! mismatch, so holding a decoded artifact is already evidence that the bytes
//! are internally consistent. The producer's identity **sidecar** is compared
//! against that, which proves these are the bytes that producer published rather
//! than some other valid artifact — worth exactly what the sidecar is worth, and
//! not adversarial evidence. And the routed variant's kernel program identity is
//! compared against the identity of the program *this* process compiled, which
//! proves by content that the artifact packages the same computation the
//! reference oracle is about to evaluate.
//!
//! No module, type, or Cargo edge crosses between producer and runner. The file
//! is the interface, which is what an artifact is for.
//!
//! [`decode_artifact`]: tiler_artifact::program::decode_artifact
//!
//! # Where the routing commit falls, and why it falls there
//!
//! ADR 0051 permits a fallback only before the commit, so every question this
//! host can answer about whether it can *carry out* a route is answered while
//! the [`Preflight`] is still held. [`plan_route`] resolves each routed ABI slot
//! to storage this proof can supply and refuses a launch that covers no threads;
//! only then is `commit` called. What stays after the commit is what needs a
//! device — the pipeline's maximum threadgroup size and the length of an
//! allocation actually made — and a refusal there is a failure reported, never a
//! fallback taken. This binary has no fallback path at all: the direct path runs
//! first and independently, as a control, and a refused envelope fails the whole
//! proof rather than being quietly covered by it.
//!
//! [`probe_fail_closed`] establishes the other half before the positive route is
//! claimed: a damaged, truncated, unexpected, or wrongly-targeted input is
//! refused under its *own* class rather than as a variant that did not apply.
//! Each probe is paired with [`probe_accepted_baseline`], which requires the
//! unperturbed subject to route, so a refusal is evidence about the one thing
//! that probe changed. The same probe functions run in the repository gate,
//! against an envelope this crate's test module assembles from the live builder;
//! this call is what carries them onto a real artifact on hardware.
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

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use metal::{
    Buffer, ComputeCommandEncoderRef, ComputePipelineDescriptor, ComputePipelineState, Device,
    MTLCommandBufferStatus, MTLGPUFamily, MTLResourceOptions, MTLSize,
};
use tiler_artifact::program::{
    AbiFactBinder, AbiFacts, ArtifactCodecFailure, AvailabilityPhase, BackendKey, BindingTarget,
    RepresentationKey, TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
};
use tiler_artifact::proof::{
    DecodedProofSidecar, ProofAssociationError, ProofCodecError, decode_proof_sidecar,
};
use tiler_compiler::session::{Compilation, CompileFailure, NumericalContract, compile_governed};
use tiler_ir::kernel::KernelType;
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};
use tiler_metal::emit::emit_translation_unit;
use tiler_metal::target::{
    LaunchIndexRealization, MetalDeploymentMinimum, MetalFloatArithmeticType, MetalFlushedZeroSign,
    MetalPlatform, MetalSubnormalArithmetic, MetalSubnormalArithmeticFacts, MetalTargetFacts,
    MslLanguageVersion,
};
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::input::{
    AppleSdk, CompileRequest, DeploymentMinimum, MetalTarget, MslVersion, NumericalRealization,
    OptimizationLevel,
};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};
use tiler_runtime::load::{
    DecodedProgram, ExecutionEnvironment, LoadRejection, Preflight, RoutedDispatch,
    TargetCompatibility, TargetDeclaration,
};

/// Rows of the direct path's input; each row reduces to one output element.
const ROWS: u64 = 4;
/// Columns of the direct path's input; the reduced axis.
///
/// The direct path fixes its own shape because its job is the *numerical*
/// claim, and three contributors per row is what makes a serial reduction's
/// ordering observable. The envelope path still does not fix one — it takes
/// whatever shape the artifact declares — and the two now coincide, which is
/// the property [`bind_interface`] proves rather than assumes.
const COLUMNS: u64 = 3;
/// Buffer argument-table capacity Apple states per compute function.
const BUFFER_BINDING_LIMIT: u32 = 31;
/// Interface key of the program's one input.
const INPUT_KEY: &str = "input";
/// Interface key of the program's one output.
const OUTPUT_KEY: &str = "result";
/// Suffix appended to the envelope path to name the proof-case sidecar.
///
/// `prototypes/serial-sum-compile` writes this name. Nothing links the two
/// crates, so each pins it in a test rather than sharing a constant neither may
/// import: the producer wrote `.proof` while this half still opened `.identity`
/// for a whole commit, and no compilation could see it.
const SIDECAR_SUFFIX: &str = ".proof";
/// Governed backend family key this host executes.
const BACKEND_KEY: &str = "tiler.metal";
/// Governed executable-representation key this host consumes.
const REPRESENTATION_KEY: &str = "metallib";
/// Byte width of one `f32`.
const F32_BYTES: u64 = 4;

/// The operand pattern each row of an input is filled from.
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

/// Fills one `rows` by `columns` input from [`ROW_PATTERNS`].
///
/// Cycling rather than indexing, so the pattern defines an input for any shape
/// an artifact might declare. At the direct path's own four-by-three shape it
/// reproduces the exact twelve operands this proof has always reduced.
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

/// Reads exactly `elements` big-endian `f32` bit patterns out of a sidecar
/// payload, or refuses the payload.
///
/// Most-significant byte first, matching the order the producer wrote, so the
/// operands never depend on host endianness. Bit patterns throughout: a signed
/// zero, a subnormal, and a non-canonical NaN must survive to the comparison
/// unchanged, which they would not if these were parsed as numbers.
///
/// The length is checked rather than truncated to a whole number of elements.
/// A payload that decodes short would reach the comparison as a shorter vector
/// and be reported as [`ProofError::Mismatch`] — a claim about the *device's*
/// arithmetic, made about a defect in the record. Refusing here keeps a
/// malformed sidecar in the sidecar's own error class.
fn decode_f32_bits(
    role: &'static str,
    elements: u64,
    bytes: &[u8],
) -> Result<Vec<u32>, ProofError> {
    let needed = elements
        .checked_mul(F32_BYTES)
        .and_then(|needed| usize::try_from(needed).ok())
        .ok_or(ProofError::SidecarShapeMismatch {
            role,
            declared: elements,
            recorded: bytes.len(),
        })?;
    if bytes.len() != needed {
        return Err(ProofError::SidecarShapeMismatch {
            role,
            declared: elements,
            recorded: bytes.len(),
        });
    }
    Ok(bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| u32::from_be_bytes(*chunk))
        .collect())
}

/// The measured Apple row, one subnormal behaviour per arithmetic type: `f32`
/// flushes and `f16` preserves on the same hardware in the same math modes.
fn target_facts() -> MetalTargetFacts {
    MetalTargetFacts::new(
        MslLanguageVersion::Metal3_1,
        MetalPlatform::MacOs,
        MetalDeploymentMinimum::new(13, 0),
        LaunchIndexRealization::ThreadPositionInGridUInt,
        MetalSubnormalArithmeticFacts::unmeasured()
            .stating(
                MetalFloatArithmeticType::F32,
                MetalSubnormalArithmetic::FlushesToZero {
                    zero_sign: MetalFlushedZeroSign::PreservesSign,
                },
            )
            .stating(
                MetalFloatArithmeticType::F16,
                MetalSubnormalArithmetic::PreservesSubnormals,
            ),
        BUFFER_BINDING_LIMIT,
    )
}

/// Builds `sum((input * 1.0) + 0.0)` over the reduced axis of a given shape.
fn serial_sum_program(rows: u64, columns: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new(INPUT_KEY).expect("the input key is valid"),
            Shape::from_dims([rows, columns]),
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
            OutputKey::new(OUTPUT_KEY).expect("the output key is valid"),
            sum,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Evaluates the same semantic program through the independent oracle.
fn reference_bits(program: &SemanticProgram, bits: &[u32], rows: u64, columns: u64) -> Vec<u32> {
    let key = InputKey::new(INPUT_KEY).expect("the input key is valid");
    let tensor = Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([rows, columns]),
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

/// Returns the envelope path the invocation names.
///
/// Hand-parsed rather than reached for a dependency, and an unrecognized
/// argument is refused instead of ignored so a typo cannot look like a run that
/// simply read somewhere else.
fn artifact_path() -> Result<PathBuf, ProofError> {
    let mut arguments = std::env::args_os().skip(1);
    let (Some(flag), Some(path), None) = (arguments.next(), arguments.next(), arguments.next())
    else {
        return Err(ProofError::Usage);
    };
    if flag != "--artifact" {
        return Err(ProofError::Usage);
    }
    Ok(PathBuf::from(path))
}

/// Reads the envelope bytes and the identity the producer recorded beside them.
///
/// The sidecar is the only thing that makes `preflight`'s identity check mean
/// anything here: an identity re-read from the envelope would be a tautology, so
/// the expected one has to come from whatever named the artifact, and here that
/// is the producer.
fn read_artifact(path: &Path) -> Result<(Vec<u8>, DecodedProofSidecar), ProofError> {
    let mut sidecar_path = path.as_os_str().to_owned();
    sidecar_path.push(SIDECAR_SUFFIX);
    let sidecar_path = PathBuf::from(sidecar_path);
    let bytes =
        std::fs::read(path).map_err(|cause| ProofError::Read(path.display().to_string(), cause))?;
    let sidecar_bytes = std::fs::read(&sidecar_path)
        .map_err(|cause| ProofError::Read(sidecar_path.display().to_string(), cause))?;
    let sidecar = decode_proof_sidecar(&sidecar_bytes).map_err(ProofError::Sidecar)?;

    // The record names an exact envelope by digest and by artifact identity, so
    // a sidecar paired with the wrong artifact is caught here rather than
    // surviving to be compared against bits it never described. A torn write
    // between the two files fails the same way, loudly.
    sidecar
        .bind_to_envelope(&bytes)
        .map_err(ProofError::SidecarAssociation)?;
    println!(
        "artifact: {} ({} bytes), sidecar {} ({} bytes, {} case(s))",
        path.display(),
        bytes.len(),
        sidecar_path.display(),
        sidecar_bytes.len(),
        sidecar.cases().len(),
    );
    Ok((bytes, sidecar))
}

/// Reads the shape the artifact declares, proves it is this program's *form*,
/// and binds its extents.
///
/// The envelope carries no semantic program — the oracle's input — so the runner
/// reconstructs one to compare against. What it takes from the artifact is the
/// interface: the keys, the element types, and the exact input shape. What it
/// supplies is the body, and a disagreement there cannot be checked here; it
/// would surface as a bit disagreement, which is why the direct path exists.
///
/// **The declared shape is read rather than asserted equal to [`COLUMNS`], and
/// that is the design rather than a gap.** What this runner may take from an
/// artifact is what the artifact says; asserting a shape here would replace the
/// artifact's declaration with this build's expectation, and the two paths would
/// then agree because they were told to rather than because one packaged what
/// the other runs.
///
/// They do agree today. They did not until
/// `bound-the-backend-entry-key-by-the-identity-it-carries`, because the
/// artifact layer bounded a `BackendEntryKey` at 1,024 bytes while a
/// two-or-more-contributor serial sum's kernel identity measures 1,121, so the
/// producer could package only the degenerate single-contributor reduction and
/// this path ran a `4x1` against the direct path's `4x3`. Nothing here changed
/// when that closed, which is what reading rather than asserting bought.
fn bind_interface(decoded: &DecodedProgram) -> Result<(u64, u64, AbiFacts), ProofError> {
    let inputs: Vec<_> = decoded.inputs().collect();
    let [input] = inputs.as_slice() else {
        return Err(ProofError::Interface(format!(
            "the artifact declares {} inputs and this program declares 1",
            inputs.len(),
        )));
    };
    let [rows, columns] = input.shape().extents() else {
        return Err(ProofError::Interface(format!(
            "the artifact's input is {}, and this program reduces a rank-2 input",
            input.shape(),
        )));
    };
    if input.key().as_str() != INPUT_KEY || input.element_type() != KernelType::F32 {
        return Err(ProofError::Interface(format!(
            "the artifact's input is {:?} of {:?}, this program's is {INPUT_KEY:?} of F32",
            input.key().as_str(),
            input.element_type(),
        )));
    }

    let outputs: Vec<_> = decoded.outputs().collect();
    let [output] = outputs.as_slice() else {
        return Err(ProofError::Interface(format!(
            "the artifact declares {} outputs and this program declares 1",
            outputs.len(),
        )));
    };
    let published: u64 = output
        .shape()
        .extents()
        .iter()
        .map(|extent| extent.get())
        .product();
    if output.key().as_str() != OUTPUT_KEY
        || output.element_type() != KernelType::F32
        || published != rows.get()
    {
        return Err(ProofError::Interface(format!(
            "the artifact's output is {:?} of {} {:?}, and reducing its input's inner axis \
             publishes {} F32 element(s) under {OUTPUT_KEY:?}",
            output.key().as_str(),
            output.shape(),
            output.element_type(),
            rows.get(),
        )));
    }

    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_input_shape(input.key(), input.shape())
        .map_err(|cause| {
            ProofError::Interface(format!("the declared input shape does not bind: {cause}"))
        })?;
    Ok((rows.get(), columns.get(), binder.build()))
}

/// States what this host offers, from the compiler's own target authority.
///
/// Deliberately **not** read from the artifact. The whole substance of
/// `ExecutionEnvironment::classify` is comparing what an artifact declares
/// against what a host independently states, so stating it from the artifact
/// would make the check a tautology. `Compilation` is the only public authority
/// in this workspace that mints a target profile descriptor, so this host's
/// statement comes from the same registry the producer's compilation consulted —
/// which is why the two agree, and why an artifact assessed against another
/// profile revision would be refused here rather than loaded.
fn host_environment(compilation: &Compilation) -> Result<ExecutionEnvironment, ProofError> {
    Ok(ExecutionEnvironment {
        target_profile: TargetProfileRef {
            key: TargetProfileKey::new(compilation.target_profile_key())
                .map_err(|_| ProofError::HostProfile)?,
            descriptor: TargetProfileDescriptorDigest::from_bytes(
                compilation.target_profile_descriptor(),
            )
            .map_err(|_| ProofError::HostProfile)?,
        },
        backend: BackendKey::new(BACKEND_KEY).map_err(|_| ProofError::HostProfile)?,
        representation: RepresentationKey::new(REPRESENTATION_KEY)
            .map_err(|_| ProofError::HostProfile)?,
    })
}

/// Builds a compute pipeline for one named function of one object image.
fn pipeline_for(
    device: &Device,
    object: &[u8],
    symbol: &str,
) -> Result<ComputePipelineState, ProofError> {
    let library = device
        .new_library_with_data(object)
        .map_err(ProofError::LibraryLoad)?;
    let function = library
        .get_function(symbol, None)
        .map_err(ProofError::FunctionLookup)?;
    let descriptor = ComputePipelineDescriptor::new();
    descriptor.set_compute_function(Some(&function));
    device
        .new_compute_pipeline_state(&descriptor)
        .map_err(ProofError::Pipeline)
}

/// Allocates the host storage this proof binds, with the input already written.
fn host_storage(device: &Device, bits: &[u32], rows: u64) -> (Buffer, Buffer, usize) {
    let elements = u64::try_from(bits.len()).expect("the proof's element count fits a u64");
    let count = usize::try_from(rows).expect("the proof's row count fits a usize");
    let input = device.new_buffer(elements * F32_BYTES, MTLResourceOptions::StorageModeShared);
    let output = device.new_buffer(rows * F32_BYTES, MTLResourceOptions::StorageModeShared);
    let operands: Vec<f32> = bits.iter().map(|value| f32::from_bits(*value)).collect();
    crate::buffer::write_f32(&input, &operands);
    (input, output, count)
}

/// Submits one encoded command buffer and reads the output back.
///
/// The command buffer's terminal state is checked *before* the host reads
/// anything, and the accepted state is exactly `Completed`. A failed submission
/// leaves the output buffer holding whatever it held before, and comparing that
/// against the reference would report a numerical disagreement for what is
/// actually a dispatch failure.
///
/// **The refusal names the status and not Metal's own error, and that is a
/// limitation of the binding rather than a choice.** `metal` 0.33.0's
/// `CommandBufferRef` exposes `commit`, `status`, `wait_until_completed` and the
/// handler registrations, and no accessor for the buffer's `NSError`; the
/// `MTLCommandBufferError` enum it declares is returned by nothing. Reading it
/// would mean an `unsafe` `msg_send!`, and a new unsafe site is a decision under
/// ADR 0079 rather than a convenience this proof may take. So a failed dispatch
/// is reported as its exact terminal status, and no claim is made about *why*
/// the device rejected it.
fn submit(
    device: &Device,
    output: &Buffer,
    count: usize,
    encode: impl FnOnce(&ComputeCommandEncoderRef),
) -> Result<Vec<u32>, ProofError> {
    let queue = device.new_command_queue();
    let command_buffer = queue.new_command_buffer();
    let encoder = command_buffer.new_compute_command_encoder();
    encode(encoder);
    encoder.end_encoding();
    command_buffer.commit();
    command_buffer.wait_until_completed();

    let status = command_buffer.status();
    if status != MTLCommandBufferStatus::Completed {
        return Err(ProofError::Dispatch(format!("{status:?}")));
    }
    Ok(crate::buffer::read_f32(output, count)
        .iter()
        .map(|value| value.to_bits())
        .collect())
}

/// Dispatches the object this process compiled, with no envelope involved.
///
/// Every dispatch parameter here is local knowledge: the symbol comes from the
/// emitter's own record, the argument-table indices are written out by hand, and
/// the launch is one thread per output row. This is the direct path the routing
/// ticket requires be **retained** — as the control that separates an envelope
/// defect from a compiler defect, never as a fallback for the envelope path.
fn dispatch_direct(
    device: &Device,
    object: &[u8],
    symbol: &str,
    bits: &[u32],
) -> Result<Vec<u32>, ProofError> {
    let pipeline = pipeline_for(device, object, symbol)?;
    let (input, output, count) = host_storage(device, bits, ROWS);
    let width = pipeline.thread_execution_width().min(ROWS);
    submit(device, &output, count, |encoder| {
        encoder.set_compute_pipeline_state(&pipeline);
        encoder.set_buffer(0, Some(&input), 0);
        encoder.set_buffer(1, Some(&output), 0);
        encoder.dispatch_threads(MTLSize::new(ROWS, 1, 1), MTLSize::new(width, 1, 1));
    })
}

/// The exact inputs a fail-closed probe perturbs one element of.
///
/// Grouped rather than passed as four arguments so a probe's signature shows
/// that it changes *one* of them and leaves the rest alone. That is what makes a
/// refusal evidence about the perturbation rather than about the whole kind: the
/// same subject routes under [`probe_accepted_baseline`], so a probe that gets a
/// refusal has isolated its cause.
#[derive(Clone, Copy)]
struct ProbeSubject<'a> {
    /// The exact encoded envelope bytes under test.
    bytes: &'a [u8],
    /// The canonical identity bytes whatever named this artifact recorded.
    expected: &'a [u8],
    /// What the host running these probes independently states it offers.
    environment: &'a ExecutionEnvironment,
    /// The ABI facts bound from the artifact's own declared interface.
    abi: &'a AbiFacts,
}

/// Reports a probe whose refusal did not arrive under the class it must.
fn refused(probe: &'static str, outcome: String) -> ProofError {
    ProofError::NotFailedClosed { probe, outcome }
}

/// Proves the loader **accepts** the unperturbed subject, before anything is
/// perturbed.
///
/// This is the neighbour every probe below is paired against, and without it
/// each of them proves close to nothing. A refusal is the easy outcome to
/// obtain: a subject whose bytes never decoded, whose recorded identity was
/// wrong, or whose host profile never matched would refuse *every* perturbation
/// under a plausible-looking class, and the probes would report a fail-closed
/// loader while measuring a broken harness. Establishing the positive route
/// first is what makes each refusal below attributable to the one thing that
/// probe changed.
fn probe_accepted_baseline(subject: &ProbeSubject<'_>) -> Result<String, ProofError> {
    let decoded = DecodedProgram::decode(subject.bytes).map_err(ProofError::ProbeBaseline)?;
    let preflight = decoded
        .preflight(subject.environment, subject.expected, subject.abi)
        .map_err(ProofError::ProbeBaseline)?;
    Ok(format!(
        "the unperturbed subject routes: {} thread(s) over {} binding(s)",
        preflight.launch().grid_threads(),
        preflight.bindings().len(),
    ))
}

/// A flipped byte inside a framed section's content is an **integrity** failure.
///
/// The class is *derived* rather than observed. The encoder writes the framing
/// header, then the manifest, then each section as its ordinal, its length, and
/// its exact content, so the last section's content ends the envelope — asserted
/// here rather than assumed. The manifest carries that section's content digest,
/// so a changed content byte can only be caught by a digest comparison: a
/// section digest, the payload identity derived from the metadata section, or
/// the artifact identity re-derived from decoded content. All three classify as
/// [`ArtifactCodecFailure::IntegrityFailure`], and none of them is a routing
/// question.
///
/// Pinning the exact class is the whole point. A damaged file reported as
/// `NoApplicableVariant` reads as "this artifact does not apply to your host",
/// which sends a reader to rebuild a plan when the repair is to re-fetch the
/// bytes; one reported as `Malformed` sends them to look for a different file.
fn probe_damaged_section_content(subject: &ProbeSubject<'_>) -> Result<String, ProofError> {
    let decoded = DecodedProgram::decode(subject.bytes).map_err(ProofError::ProbeBaseline)?;
    let content = decoded
        .sections()
        .last()
        .ok_or(ProofError::UnprobableEnvelope {
            detail: "the envelope frames no section to damage",
        })?
        .bytes()
        .to_vec();
    if content.is_empty() || !subject.bytes.ends_with(&content) {
        return Err(ProofError::UnprobableEnvelope {
            detail: "the last framed section's content does not end the envelope",
        });
    }
    let at = subject.bytes.len() - content.len();

    let mut damaged = subject.bytes.to_vec();
    damaged[at] ^= 0x01;
    match DecodedProgram::decode(&damaged) {
        Err(rejection @ LoadRejection::Artifact(ArtifactCodecFailure::IntegrityFailure { .. })) => {
            Ok(format!(
                "a flipped byte at section offset {at}: {rejection}"
            ))
        }
        Err(other) => Err(refused("a damaged section", other.to_string())),
        Ok(_) => Err(refused(
            "a damaged section",
            "the envelope decoded as valid".to_owned(),
        )),
    }
}

/// A flipped byte at an arbitrary interior offset never survives into routing.
///
/// Retained beside [`probe_damaged_section_content`] because it perturbs the
/// envelope the way damage actually arrives — at an offset nobody chose — and
/// deliberately asserts less. Which boundary refuses is a function of where the
/// byte lands: inside the manifest or a section's content it is an integrity
/// failure, inside a framed length it is malformed, inside a section ordinal it
/// is invalid. What must hold for *every* offset is that the artifact layer
/// refuses, so that is what is asserted; pinning one of those classes here would
/// pin an accident of this envelope's size rather than a property of the loader.
///
/// **Measurement**, on an Apple M4 Max against the producer's 32,449-byte
/// envelope: the midpoint lands in the manifest and the refusal is
/// `ManifestDigestMismatch`, an integrity failure. That is one envelope's
/// arithmetic, not a guarantee, which is exactly why it is not asserted.
fn probe_damaged_interior_byte(subject: &ProbeSubject<'_>) -> Result<String, ProofError> {
    let mut damaged = subject.bytes.to_vec();
    let midpoint = damaged.len() / 2;
    damaged[midpoint] ^= 0x01;
    match DecodedProgram::decode(&damaged) {
        Err(rejection @ LoadRejection::Artifact(_)) => {
            Ok(format!("a flipped byte at offset {midpoint}: {rejection}"))
        }
        Err(other) => Err(refused("a flipped interior byte", other.to_string())),
        Ok(_) => Err(refused(
            "a flipped interior byte",
            "the envelope decoded as valid".to_owned(),
        )),
    }
}

/// A truncated envelope is **malformed**, and that class is derivable.
///
/// The framing header states the envelope's own total length, which is a
/// derived field of the exact encoding rather than a producer claim. No proper
/// prefix satisfies it, so a prefix long enough to carry the header is refused
/// as a total-length disagreement and a shorter one is refused as truncation.
/// Both classify as [`ArtifactCodecFailure::Malformed`], for either length, so
/// nothing about this class depends on where the cut falls.
fn probe_truncated_envelope(subject: &ProbeSubject<'_>) -> Result<String, ProofError> {
    let midpoint = subject.bytes.len() / 2;
    match DecodedProgram::decode(&subject.bytes[..midpoint]) {
        Err(rejection @ LoadRejection::Artifact(ArtifactCodecFailure::Malformed { .. })) => {
            Ok(format!("truncated to {midpoint} byte(s): {rejection}"))
        }
        Err(other) => Err(refused("a truncated envelope", other.to_string())),
        Ok(_) => Err(refused(
            "a truncated envelope",
            "the envelope decoded as valid".to_owned(),
        )),
    }
}

/// An artifact that is not the expected one is a **program mismatch**.
///
/// Not a variant that failed to apply, and not damage. These bytes decode and
/// are internally consistent; what is wrong is that they are some other valid
/// artifact, which is a stale cache entry or a mixed-up path rather than a plan
/// to rebuild.
fn probe_foreign_expected_identity(subject: &ProbeSubject<'_>) -> Result<String, ProofError> {
    let decoded = DecodedProgram::decode(subject.bytes).map_err(ProofError::ProbeBaseline)?;
    let mut foreign = subject.expected.to_vec();
    if let Some(last) = foreign.last_mut() {
        *last ^= 0x01;
    }
    match decoded.preflight(subject.environment, &foreign, subject.abi) {
        Err(rejection @ LoadRejection::ProgramMismatch { .. }) => Ok(format!(
            "an expected identity that is not this artifact's: {rejection}"
        )),
        Err(other) => Err(refused("a foreign expected identity", other.to_string())),
        Ok(_) => Err(refused(
            "a foreign expected identity",
            "the route was accepted".to_owned(),
        )),
    }
}

/// A host offering another profile descriptor is an **incompatible target**,
/// named on the *variant's* declaration.
///
/// Both halves of the class are pinned. Which declaration refused separates a
/// plan assessed for another profile from an object compiled for one, and those
/// are different repairs; the classification separates the same target family
/// under a descriptor this host does not offer from an artifact built for
/// another family entirely. Asserting only that something refused would erase
/// both distinctions at the moment a caller needs them.
fn probe_other_profile_descriptor(subject: &ProbeSubject<'_>) -> Result<String, ProofError> {
    let decoded = DecodedProgram::decode(subject.bytes).map_err(ProofError::ProbeBaseline)?;
    let mut descriptor = subject
        .environment
        .target_profile
        .descriptor
        .as_bytes()
        .to_vec();
    if let Some(last) = descriptor.last_mut() {
        *last ^= 0x01;
    }
    let other_host = ExecutionEnvironment {
        target_profile: TargetProfileRef {
            key: subject.environment.target_profile.key.clone(),
            descriptor: TargetProfileDescriptorDigest::from_bytes(&descriptor)
                .map_err(|_| ProofError::HostProfile)?,
        },
        backend: subject.environment.backend.clone(),
        representation: subject.environment.representation.clone(),
    };
    match decoded.preflight(&other_host, subject.expected, subject.abi) {
        Err(
            rejection @ LoadRejection::IncompatibleTarget {
                declaration: TargetDeclaration::Variant,
                classification: TargetCompatibility::DescriptorMismatch { .. },
            },
        ) => Ok(format!(
            "a host offering another profile descriptor: {rejection}"
        )),
        Err(other) => Err(refused("another profile descriptor", other.to_string())),
        Ok(_) => Err(refused(
            "another profile descriptor",
            "the route was accepted".to_owned(),
        )),
    }
}

/// A host stating another backend family is an **unexecutable payload**.
///
/// Refused on that ground rather than on the target profile it happens to
/// share, which is why this probe changes only the backend key: the host still
/// offers the exact profile the variant was assessed against, so the refusal
/// cannot come from the compatibility classification.
fn probe_other_backend_family(subject: &ProbeSubject<'_>) -> Result<String, ProofError> {
    let decoded = DecodedProgram::decode(subject.bytes).map_err(ProofError::ProbeBaseline)?;
    let other_backend = ExecutionEnvironment {
        target_profile: subject.environment.target_profile.clone(),
        backend: BackendKey::new("tiler.some-other-backend")
            .map_err(|_| ProofError::HostProfile)?,
        representation: subject.environment.representation.clone(),
    };
    match decoded.preflight(&other_backend, subject.expected, subject.abi) {
        Err(rejection @ LoadRejection::UnexecutablePayload { .. }) => Ok(format!(
            "a host stating another backend family: {rejection}"
        )),
        Err(other) => Err(refused("another backend family", other.to_string())),
        Ok(_) => Err(refused(
            "another backend family",
            "the route was accepted".to_owned(),
        )),
    }
}

/// Proves the loader fails closed on inputs that are not this artifact.
///
/// Run against the **real** envelope this process just read, not against a
/// synthetic fixture, and run *before* the positive route is claimed. Each probe
/// perturbs exactly one thing and pins the class of the refusal, because the
/// failure mode this guards against is not "it was accepted" — it is a refusal
/// arriving under the wrong class. That is the "corrupt artifacts must not
/// become route misses" obligation, and it is only observable by asserting the
/// variant.
///
/// The probes are decidable without a device, and the crate's own test module
/// runs every one of them in the repository gate against an envelope it
/// assembles from the live builder. This call is what carries the same
/// assertions onto a real `xcrun`-produced artifact on hardware; neither
/// subsumes the other, because the gate cannot reach a Metal toolchain on both
/// CI profiles and the hardware run is not a gate.
fn probe_fail_closed(subject: &ProbeSubject<'_>) -> Result<(), ProofError> {
    for probe in [
        probe_accepted_baseline as fn(&ProbeSubject<'_>) -> Result<String, ProofError>,
        probe_damaged_section_content,
        probe_damaged_interior_byte,
        probe_truncated_envelope,
        probe_foreign_expected_identity,
        probe_other_profile_descriptor,
        probe_other_backend_family,
    ] {
        println!("  {}", probe(subject)?);
    }
    Ok(())
}

/// Which storage this proof will supply for one routed ABI slot.
///
/// Resolved before the commit and carried as an owned decision, so the encoder
/// never re-asks a question whose answer could have refused the route.
#[derive(Clone, Copy, Debug)]
enum Placement {
    /// The buffer holding the program input the artifact names.
    Input,
    /// The buffer receiving the program output the artifact names.
    Output,
    /// Entry-internal storage: named by nothing, sized by its own
    /// accessible-byte expression, and allocated rather than bound — which is
    /// what the artifact layer says a loader does with one.
    Internal,
}

/// One routed ABI slot, resolved to storage this host can actually supply.
#[derive(Clone, Copy, Debug)]
struct PlacedSlot {
    transport: u32,
    needed: u64,
    placement: Placement,
}

/// Decides whether this host can carry out a route, while abandoning it is
/// still permitted.
///
/// **Every refusal here is a refusal the host owes itself before the commit,
/// and that is the whole point of the function.** `Preflight` publishes the
/// launch geometry and the routed bindings precisely so a caller can judge them
/// and decline; a host that instead committed and *then* discovered it binds no
/// storage for some slot would have destroyed its own fallback authority for a
/// reason that was decidable while it still held it. ADR 0051 permits a
/// fallback only before the commit, so a check that could have run before it
/// must not run after.
///
/// What this function cannot decide is everything that needs a device — the
/// library, the pipeline, the threadgroup capacity, the allocations. Those are
/// not device-*free*, but they are decidable, and [`device_preflight`] takes
/// them before the same commit. Nothing that a device can answer is left for
/// after it.
fn plan_route(preflight: &Preflight<'_>) -> Result<Vec<PlacedSlot>, ProofError> {
    let launch = preflight.launch();
    if launch.grid_threads() == 0 {
        // The artifact states whether an empty launch is skipped or encoded, so
        // the answer is read rather than assumed. Either way this proof has no
        // output to compare and says so instead of reporting agreement on
        // whatever the output buffer happened to hold.
        return Err(ProofError::EmptyLaunch {
            skipped: launch.zero_work_skips_dispatch(),
        });
    }

    let mut plan = Vec::with_capacity(preflight.bindings().len());
    for binding in preflight.bindings() {
        let placement = match binding.binding().target() {
            BindingTarget::ProgramInput(key) if key.as_str() == INPUT_KEY => Placement::Input,
            BindingTarget::ProgramOutput(keys)
                if keys.len() == 1 && keys[0].as_str() == OUTPUT_KEY =>
            {
                Placement::Output
            }
            BindingTarget::Internal => Placement::Internal,
            other => {
                return Err(ProofError::UnboundBinding {
                    slot: binding.slot(),
                    target: format!("{other:?}"),
                });
            }
        };
        plan.push(PlacedSlot {
            transport: binding.transport_slot(),
            needed: binding.accessible_bytes(),
            placement,
        });
    }
    Ok(plan)
}

/// Which stage of the device preflight reached a decision.
///
/// Ordered as they run, and the order is the useful one: a refusal names the
/// earliest obligation that failed, so a library that will not load is never
/// reported as a launch-geometry problem.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PreflightPhase {
    /// Building an executable library from the payload's object bytes.
    Library,
    /// Resolving the entry symbol the payload's subject names.
    Function,
    /// Creating compute pipeline state for a resolved function.
    Pipeline,
    /// Comparing the declared launch against what the pipeline admits.
    LaunchGeometry,
    /// Allocating and sizing every bound buffer and every internal scratch slot.
    Resources,
}

impl PreflightPhase {
    /// A stable lowercase identifier for this stage.
    const fn as_str(self) -> &'static str {
        match self {
            Self::Library => "library",
            Self::Function => "function",
            Self::Pipeline => "pipeline",
            Self::LaunchGeometry => "launch-geometry",
            Self::Resources => "resources",
        }
    }
}

/// What a caller should do about a refusal, which is why phases are typed at all.
///
/// A host that cannot tell these apart either retries work that can never
/// succeed or abandons an artifact that had a working route. They are a
/// contract, not a diagnostic convenience.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PreflightClass {
    /// This route does not fit *this device*, and another variant might.
    ///
    /// A fallback is permitted and is the indicated response. Every refusal in
    /// this class compares something the artifact declared against something the
    /// device reported, so a differently-declared variant is exactly the remedy.
    RouteMiss,
    /// These bytes passed decode and integrity validation and still do not yield
    /// a runnable library.
    ///
    /// Distinct from an integrity failure, which the codec already refused
    /// before any of this ran: the digest matched, so the object *is* what the
    /// producer published, and it is content that will not execute. A caller
    /// re-fetches or rebuilds; retrying another variant of the same bytes is not
    /// indicated.
    CorruptArtifact,
    /// The host cannot serve any route, whatever it declares.
    Systemic,
}

impl PreflightClass {
    /// A stable lowercase identifier for this class.
    const fn as_str(self) -> &'static str {
        match self {
            Self::RouteMiss => "route-miss",
            Self::CorruptArtifact => "corrupt-artifact",
            Self::Systemic => "systemic",
        }
    }
}

/// One refusal the device preflight reached, before any commit.
///
/// Carries the numbers the decision was made from rather than a rendered
/// sentence, so [`Self::phase`] and [`Self::class`] are total functions over the
/// variant and a caller acts on the class without parsing anything.
#[derive(Clone, Debug, Eq, PartialEq)]
enum PreflightRefusal {
    /// The payload's object bytes did not produce a library.
    LibraryRejected { detail: String },
    /// The library loaded and publishes no function by the entry symbol.
    FunctionAbsent { symbol: String, detail: String },
    /// The device refused pipeline state for a function it did publish.
    PipelineRejected { symbol: String, detail: String },
    /// The declared workgroup is larger than this pipeline admits.
    WorkgroupTooLarge {
        symbol: String,
        declared: u64,
        capacity: u64,
    },
    /// A binding must reach more bytes than one buffer can hold here.
    BindingExceedsBufferLimit {
        slot: usize,
        needed: u64,
        limit: u64,
    },
    /// An allocation came back shorter than the route requires.
    UndersizedAllocation { slot: usize, needed: u64, held: u64 },
}

impl PreflightRefusal {
    /// The stage this refusal came from.
    ///
    /// Exhaustive rather than a wildcard, so a refusal added later is placed in
    /// a stage deliberately instead of inheriting whichever one a catch-all
    /// named.
    const fn phase(&self) -> PreflightPhase {
        match self {
            Self::LibraryRejected { .. } => PreflightPhase::Library,
            Self::FunctionAbsent { .. } => PreflightPhase::Function,
            Self::PipelineRejected { .. } => PreflightPhase::Pipeline,
            Self::WorkgroupTooLarge { .. } => PreflightPhase::LaunchGeometry,
            Self::BindingExceedsBufferLimit { .. } | Self::UndersizedAllocation { .. } => {
                PreflightPhase::Resources
            }
        }
    }

    /// What a caller should do about this refusal.
    ///
    /// **`PipelineRejected` is a route miss, and the direction is derived rather
    /// than guessed.** Metal reports pipeline-creation failure as a message
    /// string that does not reliably separate "this function exceeds a device
    /// limit" from "the device is out of resources". Of the two ways to be
    /// wrong, calling a systemic failure a route miss costs a retry that then
    /// fails; calling a route miss systemic abandons an artifact that had a
    /// working variant. Only the second forfeits the fallback ADR 0051 grants
    /// while it is still held, so the classification takes the recoverable
    /// direction.
    ///
    /// `UndersizedAllocation` is systemic rather than a route miss because it is
    /// an assertion against the device's own report — every buffer is requested
    /// at the length the route states — so reaching it means the allocator did
    /// not honour a request it accepted, which no other variant improves.
    const fn class(&self) -> PreflightClass {
        match self {
            Self::LibraryRejected { .. } | Self::FunctionAbsent { .. } => {
                PreflightClass::CorruptArtifact
            }
            Self::PipelineRejected { .. }
            | Self::WorkgroupTooLarge { .. }
            | Self::BindingExceedsBufferLimit { .. } => PreflightClass::RouteMiss,
            Self::UndersizedAllocation { .. } => PreflightClass::Systemic,
        }
    }
}

impl fmt::Display for PreflightRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}/{}: ",
            self.phase().as_str(),
            self.class().as_str(),
        )?;
        match self {
            Self::LibraryRejected { detail } => {
                write!(formatter, "the carried object did not load: {detail}")
            }
            Self::FunctionAbsent { symbol, detail } => {
                write!(formatter, "the library publishes no {symbol:?}: {detail}")
            }
            Self::PipelineRejected { symbol, detail } => {
                write!(formatter, "no pipeline state for {symbol:?}: {detail}")
            }
            Self::WorkgroupTooLarge {
                symbol,
                declared,
                capacity,
            } => write!(
                formatter,
                "{symbol:?} admits {capacity} thread(s) per threadgroup and the artifact declares {declared}"
            ),
            Self::BindingExceedsBufferLimit {
                slot,
                needed,
                limit,
            } => write!(
                formatter,
                "slot {slot} must reach {needed} byte(s) and one buffer holds at most {limit}"
            ),
            Self::UndersizedAllocation { slot, needed, held } => write!(
                formatter,
                "slot {slot} needs {needed} byte(s) and the allocation returned {held}"
            ),
        }
    }
}

/// What the device reported about itself, recorded rather than checked.
///
/// **No artifact field names a required GPU family, a threadgroup floor, or a
/// buffer-length floor**, so there is nothing here to compare these against.
/// Declaring a requirement the artifact never made would be inventing one, so
/// these are provenance: they say which device produced a measurement, and they
/// are what a future artifact-side family declaration would be checked against.
///
/// The two limits that *do* have an artifact-side counterpart — the pipeline's
/// threadgroup capacity and the per-buffer length bound — are checked in
/// [`device_preflight`] rather than recorded here, because a declared launch and
/// a declared accessible range are things the artifact does state.
#[derive(Clone, Debug)]
struct DeviceFacts {
    name: String,
    max_threads_per_threadgroup: u64,
    max_buffer_length: u64,
    recommended_working_set: u64,
    highest_apple_family: Option<&'static str>,
}

/// One route this device has proved it can carry out, with everything it needs.
///
/// Held across the commit: every device object the encode touches is created
/// here, so the post-commit path allocates nothing, looks nothing up, and has no
/// failure to report. That is the property the stage exists for.
struct PreparedRoute {
    pipeline: ComputePipelineState,
    /// Buffers in the route's own binding order, paired with the argument-table
    /// index each occupies.
    ///
    /// Retained here until the command buffer completes: entry-internal storage
    /// is the loader's to allocate, and dropping it would leave the encoder
    /// holding a binding to a freed allocation. This value outlives the `submit`
    /// call, which waits for the command buffer's terminal state, so every
    /// buffer is alive through its final device use.
    placements: Vec<(u32, Buffer)>,
    /// The buffer the program's output lands in, for read-back.
    output: Buffer,
    /// How many `f32` elements to read back out of it.
    readback: usize,
    facts: DeviceFacts,
}

/// Proves this device can carry out a route, while declining is still permitted.
///
/// **This is the stage the ticket exists to add, and the ordering is its whole
/// substance.** `Preflight::commit` is infallible and documents why: every
/// decidable obligation is supposed to have been discharged before it. Three
/// were not. The library, the pipeline, and the comparison of the declared
/// workgroup against the capacity that pipeline reports all ran inside the
/// dispatch, after the commit. The last of those is what makes it a correctness
/// bug rather than an ordering preference: a workgroup this pipeline cannot run
/// is a fact about *this route on this device*, so another variant might satisfy
/// it, and ADR 0051 permits that fallback only while the commit has not been
/// taken. Reporting it afterwards converts a fallback the host was still owed
/// into a failure it merely reports.
///
/// Nothing here is observable if the route is then abandoned: it allocates and
/// fills host-visible storage and creates pipeline state, and encodes nothing.
fn device_preflight(
    device: &Device,
    preflight: &Preflight<'_>,
    plan: &[PlacedSlot],
    operands: &[u32],
    rows: u64,
) -> Result<PreparedRoute, PreflightRefusal> {
    let facts = device_facts(device);

    // The library and the entry symbol come from the artifact, never from this
    // process. The identical call in `dispatch_direct` cannot fail this way for
    // the same reason: there the object is one this build just emitted, so a
    // rejection is a defect in Tiler, and here it is a statement about published
    // bytes. Same device call, different meaning, because the provenance of the
    // bytes differs — which is why the two paths do not share a refusal type.
    let library = device
        .new_library_with_data(preflight.object())
        .map_err(|detail| PreflightRefusal::LibraryRejected { detail })?;
    let symbol = preflight.entry_symbol();
    let function =
        library
            .get_function(symbol, None)
            .map_err(|detail| PreflightRefusal::FunctionAbsent {
                symbol: symbol.to_owned(),
                detail,
            })?;

    // One function, because the loader routes one entry: `accept_entry` selects
    // exactly one, and this reader refuses a multi-stage variant outright.
    // Looping over a collection that cannot hold two would claim coverage that
    // does not exist. `preflight-every-entry-of-a-multi-stage-route` owns the
    // runtime half when a route can carry more than one.
    let descriptor = ComputePipelineDescriptor::new();
    descriptor.set_compute_function(Some(&function));
    let pipeline = device
        .new_compute_pipeline_state(&descriptor)
        .map_err(|detail| PreflightRefusal::PipelineRejected {
            symbol: symbol.to_owned(),
            detail,
        })?;

    let launch = preflight.launch();
    workgroup_fits(
        symbol,
        launch.threads_per_workgroup(),
        pipeline.max_total_threads_per_threadgroup(),
    )?;

    // Sized from the route rather than from the operand slice: the artifact
    // states how many bytes each binding must reach, and deriving a length from
    // the host's own data would re-answer a question the artifact answered.
    let mut placements = Vec::with_capacity(plan.len());
    let mut output = None;
    for (slot, placed) in plan.iter().enumerate() {
        binding_fits(slot, placed.needed, facts.max_buffer_length)?;
        let options = match placed.placement {
            Placement::Input | Placement::Output => MTLResourceOptions::StorageModeShared,
            Placement::Internal => MTLResourceOptions::StorageModePrivate,
        };
        let storage = device.new_buffer(placed.needed.max(1), options);
        allocation_fits(slot, placed.needed, storage.length())?;
        match placed.placement {
            Placement::Input => {
                // The assertion inside `write_f32` is the backstop for a length
                // disagreement, and it is unreachable here: the operand count
                // was checked against the shape the artifact declares, and this
                // buffer's length is that same shape's accessible byte range.
                let values: Vec<f32> = operands.iter().map(|bits| f32::from_bits(*bits)).collect();
                crate::buffer::write_f32(&storage, &values);
            }
            Placement::Output => output = Some(storage.clone()),
            Placement::Internal => {}
        }
        placements.push((placed.transport, storage));
    }

    Ok(PreparedRoute {
        pipeline,
        placements,
        // `plan_route` refuses every binding target this proof does not place,
        // and this program declares one output, so the placement pass bound it.
        output: output.expect("the placement pass bound this program's one output"),
        readback: usize::try_from(rows).expect("the proof's row count fits a usize"),
        facts,
    })
}

/// Whether a declared workgroup fits what a pipeline admits.
///
/// Split from the device call so the decision is testable without hardware: the
/// device contributes two numbers and this contributes the comparison.
fn workgroup_fits(symbol: &str, declared: u64, capacity: u64) -> Result<(), PreflightRefusal> {
    if declared > capacity {
        return Err(PreflightRefusal::WorkgroupTooLarge {
            symbol: symbol.to_owned(),
            declared,
            capacity,
        });
    }
    Ok(())
}

/// Whether one binding's accessible range fits in a single buffer here.
fn binding_fits(slot: usize, needed: u64, limit: u64) -> Result<(), PreflightRefusal> {
    if needed > limit {
        return Err(PreflightRefusal::BindingExceedsBufferLimit {
            slot,
            needed,
            limit,
        });
    }
    Ok(())
}

/// Whether an allocation the device returned reaches the length it was asked for.
fn allocation_fits(slot: usize, needed: u64, held: u64) -> Result<(), PreflightRefusal> {
    if held < needed {
        return Err(PreflightRefusal::UndersizedAllocation { slot, needed, held });
    }
    Ok(())
}

/// Reads what this device reports about itself.
fn device_facts(device: &Device) -> DeviceFacts {
    // Highest first: the families are cumulative, so the first supported one is
    // the most specific true statement. `None` is reported rather than guessed
    // when the device claims none of them.
    let highest_apple_family = [
        (MTLGPUFamily::Apple9, "Apple9"),
        (MTLGPUFamily::Apple8, "Apple8"),
        (MTLGPUFamily::Apple7, "Apple7"),
        (MTLGPUFamily::Apple6, "Apple6"),
        (MTLGPUFamily::Apple5, "Apple5"),
    ]
    .into_iter()
    .find(|(family, _)| device.supports_family(*family))
    .map(|(_, name)| name);

    DeviceFacts {
        name: device.name().to_owned(),
        max_threads_per_threadgroup: device.max_threads_per_threadgroup().width,
        max_buffer_length: device.max_buffer_length(),
        recommended_working_set: device.recommended_max_working_set_size(),
        highest_apple_family,
    }
}

/// Injects each device-preflight refusal against the real route, before the
/// commit.
///
/// The device-free unit cases pin the comparisons and the classification; these
/// pin that the *device* produces the refusal this code claims it does. A Metal
/// binding's rejection of an object that is not a `metallib`, or of a symbol a
/// library does not publish, is a fact about Metal rather than about this file,
/// and asserting it needs a device. Run by the hardware proof, like
/// [`probe_fail_closed`], because `make full` reaches no device.
///
/// Every probe here perturbs one input and leaves the rest alone, so a refusal
/// is evidence about the perturbation: the same device, the same route, and the
/// same operands succeeded moments earlier in [`run`].
fn probe_device_preflight(
    device: &Device,
    preflight: &Preflight<'_>,
    plan: &[PlacedSlot],
    operands: &[u32],
    rows: u64,
) -> Result<(), ProofError> {
    // A library built from bytes that are not a metallib. The digest over these
    // bytes matched, so this is content that will not execute rather than an
    // integrity failure — the distinction `PreflightClass::CorruptArtifact`
    // exists to carry.
    let refusal = device
        .new_library_with_data(b"tiler probe object; not an executable image")
        .err()
        .map(|detail| PreflightRefusal::LibraryRejected { detail })
        .ok_or(ProofError::ProbeAccepted(
            "a library from non-metallib bytes",
        ))?;
    report_refusal("an object that is not a metallib", &refusal);

    // A symbol the real library does not publish.
    let library = device
        .new_library_with_data(preflight.object())
        .map_err(|detail| {
            ProofError::DevicePreflight(Box::new(PreflightRefusal::LibraryRejected { detail }))
        })?;
    let refusal = library
        .get_function("tiler_kernel_this_object_does_not_publish", None)
        .err()
        .map(|detail| PreflightRefusal::FunctionAbsent {
            symbol: "tiler_kernel_this_object_does_not_publish".to_owned(),
            detail,
        })
        .ok_or(ProofError::ProbeAccepted("an absent entry symbol"))?;
    report_refusal("an entry symbol the object does not publish", &refusal);

    // A workgroup one thread larger than the pipeline admits, using the capacity
    // this device actually reported rather than an invented number. This is the
    // refusal that used to arrive after the commit.
    let function = library
        .get_function(preflight.entry_symbol(), None)
        .map_err(|detail| {
            ProofError::DevicePreflight(Box::new(PreflightRefusal::FunctionAbsent {
                symbol: preflight.entry_symbol().to_owned(),
                detail,
            }))
        })?;
    let descriptor = ComputePipelineDescriptor::new();
    descriptor.set_compute_function(Some(&function));
    let pipeline = device
        .new_compute_pipeline_state(&descriptor)
        .map_err(|detail| {
            ProofError::DevicePreflight(Box::new(PreflightRefusal::PipelineRejected {
                symbol: preflight.entry_symbol().to_owned(),
                detail,
            }))
        })?;
    let capacity = pipeline.max_total_threads_per_threadgroup();
    let refusal = workgroup_fits(preflight.entry_symbol(), capacity + 1, capacity)
        .err()
        .ok_or(ProofError::ProbeAccepted(
            "a workgroup larger than the pipeline admits",
        ))?;
    report_refusal("a workgroup one thread past this pipeline", &refusal);

    // A binding needing one byte more than this device holds in one buffer.
    let limit = device_facts(device).max_buffer_length;
    let refusal = binding_fits(0, limit + 1, limit)
        .err()
        .ok_or(ProofError::ProbeAccepted("a binding past the buffer limit"))?;
    report_refusal("a binding one byte past the buffer limit", &refusal);

    // The unperturbed route still prepares, which is what makes each refusal
    // above evidence about its own perturbation rather than about the route.
    device_preflight(device, preflight, plan, operands, rows)
        .map_err(|refusal| ProofError::DevicePreflight(Box::new(refusal)))?;
    println!("  the unperturbed route prepares: every stage cleared before the commit");
    Ok(())
}

/// Prints one injected refusal with the phase and class it was classified into.
fn report_refusal(probe: &str, refusal: &PreflightRefusal) {
    println!("  {probe}: {refusal}");
}

/// Dispatches a route this device already proved it can carry out.
///
/// Every device object was created before the commit, so this function looks
/// nothing up, allocates nothing, and has no refusal of its own to report. What
/// remains is encoding and submission, and `submit` owns the one thing that can
/// still go wrong: a command buffer that does not reach `Completed`, checked
/// before the host reads anything back.
fn dispatch_prepared(
    device: &Device,
    routed: &RoutedDispatch<'_>,
    prepared: &PreparedRoute,
) -> Result<Vec<u32>, ProofError> {
    let launch = routed.launch();
    submit(device, &prepared.output, prepared.readback, |encoder| {
        encoder.set_compute_pipeline_state(&prepared.pipeline);
        for (transport, storage) in &prepared.placements {
            encoder.set_buffer(u64::from(*transport), Some(storage), 0);
        }
        encoder.dispatch_threads(
            MTLSize::new(launch.grid_threads(), 1, 1),
            MTLSize::new(launch.threads_per_workgroup(), 1, 1),
        );
    })
}

pub(crate) fn main() -> ExitCode {
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
    reason = "the proof is one linear narrative from a semantic program through two independent dispatches to compared bits; splitting it would hide the ordering that is its point"
)]
fn run() -> Result<(), ProofError> {
    let envelope_path = artifact_path()?;
    let program = serial_sum_program(ROWS, COLUMNS);
    let bits = input_bits(ROWS, COLUMNS);
    let reference = reference_bits(&program, &bits, ROWS, COLUMNS);

    let device = &Device::system_default().ok_or(ProofError::NoDevice)?;
    println!("device: {}", device.name());

    // ---- the direct path -------------------------------------------------
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
    let compiled = Toolchain::system()
        .compile(&request)
        .map_err(|_| ProofError::Toolchain)?;
    println!("compiled {} bytes of metallib", compiled.metallib.len());
    let emitted = unit.entry_points().first().ok_or(ProofError::Emit)?;
    let direct = dispatch_direct(device, &compiled.metallib, emitted.symbol(), &bits)?;

    // ---- the envelope path -----------------------------------------------
    let (bytes, sidecar) = read_artifact(&envelope_path)?;
    let decoded = DecodedProgram::decode(&bytes).map_err(ProofError::Load)?;
    println!(
        "decoded: {} variant(s), required features {:?}",
        decoded.variant_count(),
        decoded.required_features(),
    );
    let (rows, columns, abi) = bind_interface(&decoded)?;
    println!("the artifact declares a {rows} by {columns} input");
    let environment = host_environment(compilation)?;

    // Established before the positive route is claimed: a loader that accepted
    // these bytes would say nothing about what it refuses, and the refusals are
    // half of what makes the acceptance mean anything.
    println!("fail-closed probes against these exact bytes:");
    probe_fail_closed(&ProbeSubject {
        bytes: &bytes,
        expected: sidecar.artifact_identity_bytes(),
        environment: &environment,
        abi: &abi,
    })?;

    let preflight = decoded
        .preflight(&environment, sidecar.artifact_identity_bytes(), &abi)
        .map_err(ProofError::Load)?;

    // Compiled here only to *name* the program the artifact claims to package.
    // Nothing is emitted, nothing is linked, and nothing from it reaches the
    // device: the check is that the packaged kernel program's canonical identity
    // is the one this build derives for the shape the artifact declares, and
    // that is the one binding between the two processes a sidecar cannot forge.
    let envelope_program = serial_sum_program(rows, columns);
    let envelope_compilations = compile_governed(
        &envelope_program,
        NumericalContract::FlushSubnormalsToZeroF32,
    )
    .map_err(ProofError::Compile)?;
    let envelope_plan = envelope_compilations
        .first()
        .ok_or(ProofError::NoTarget)?
        .selected()
        .ok_or(ProofError::NoSelection)?;
    // Bound rather than chained: the ABI view borrows the plan alternative, so a
    // temporary would not outlive the comparison.
    let construction = envelope_plan.abi();
    let local = construction
        .kernel_program()
        .canonical_identity()
        .as_bytes();
    // Checked before the commit, because a route to a program this process did
    // not derive is a reason to abandon rather than to execute and compare.
    if preflight.kernel_program_identity() != local {
        return Err(ProofError::ForeignProgram {
            packaged: preflight.kernel_program_identity().len(),
            compiled: local.len(),
        });
    }

    // Read from the record the producer published, never re-derived here. This
    // process could evaluate the same reference over the same operands and
    // usually get the same answer, and that is exactly the problem: it would be
    // checking the device against its own opinion rather than against the claim
    // the artifact was published under. A producer and a runner that each derive
    // the normative bits agree until the day they do not.
    //
    // Read before the commit because the operands are an input to the device
    // preflight: the input buffer is allocated and filled while declining the
    // route is still permitted.
    let case = sidecar
        .cases()
        .next()
        .ok_or(ProofError::SidecarWithoutCases)?;
    // Both payloads are checked against the element count the *artifact*
    // declares, not against each other: a record that agrees with itself and
    // not with the interface it names is still describing another program.
    let envelope_bits = case
        .inputs()
        .next()
        .ok_or(ProofError::SidecarWithoutCases)
        .and_then(|payload| decode_f32_bits("input", rows * columns, payload.bytes()))?;
    let envelope_reference = case
        .expected()
        .next()
        .ok_or(ProofError::SidecarWithoutCases)
        .and_then(|payload| decode_f32_bits("expected", rows, payload.bytes()))?;

    // Placement first, then the device. Both are decided while a fallback is
    // still permitted, and between them they discharge every obligation this
    // host can decide — which is what makes the commit below infallible in fact
    // and not only in signature. See `plan_route` and `device_preflight`.
    let plan = plan_route(&preflight)?;
    let prepared = device_preflight(device, &preflight, &plan, &envelope_bits, rows)
        .map_err(|refusal| ProofError::DevicePreflight(Box::new(refusal)))?;
    let facts = &prepared.facts;
    println!(
        "device preflight: {} ({}), {} thread(s) per threadgroup, buffers to {} byte(s), \
         working set {} byte(s)",
        facts.name,
        facts
            .highest_apple_family
            .unwrap_or("no Apple family reported"),
        facts.max_threads_per_threadgroup,
        facts.max_buffer_length,
        facts.recommended_working_set,
    );
    println!("device-preflight refusals against this exact route:");
    probe_device_preflight(device, &preflight, &plan, &envelope_bits, rows)?;

    let routed = preflight.commit();
    println!(
        "routed: symbol {:?}, {} object byte(s), {} thread(s) in groups of {}",
        routed.entry_symbol(),
        routed.object().len(),
        routed.launch().grid_threads(),
        routed.launch().threads_per_workgroup(),
    );
    for binding in routed.bindings() {
        println!(
            "  abi slot {} -> transport {}, {} byte(s), {:?}",
            binding.slot(),
            binding.transport_slot(),
            binding.accessible_bytes(),
            binding.binding().target(),
        );
    }
    let envelope = dispatch_prepared(device, &routed, &prepared)?;

    // ---- numerical verification ------------------------------------------
    // Each path is compared against the oracle's evaluation of the program that
    // path ran. They are the same program whenever the artifact carries the
    // direct path's shape; while it cannot, comparing one path's bits against
    // the other's reference would be comparing two different computations.
    println!("direct    {ROWS}x{COLUMNS}: {direct:08x?} against {reference:08x?}");
    println!("envelope  {rows}x{columns}: {envelope:08x?} against {envelope_reference:08x?}");
    if direct != reference {
        return Err(ProofError::Mismatch {
            path: "direct",
            device: direct,
            reference,
        });
    }
    if envelope != envelope_reference {
        return Err(ProofError::Mismatch {
            path: "envelope",
            device: envelope,
            reference: envelope_reference,
        });
    }
    println!(
        "bit-for-bit agreement: direct on {} element(s), envelope on {} element(s)",
        reference.len(),
        envelope_reference.len(),
    );
    Ok(())
}

/// Why one end-to-end proof did not complete.
///
/// The stages stay apart: a program this build cannot compile, a target that
/// cannot honour the contract, a missing toolchain, a missing device, an
/// artifact this host refuses, an artifact that is not this program's, a failed
/// dispatch, and a numerical disagreement are different things to do next, and
/// only the last is a claim about arithmetic.
#[derive(Debug)]
enum ProofError {
    Usage,
    Read(String, std::io::Error),
    Sidecar(ProofCodecError),
    SidecarWithoutCases,
    SidecarShapeMismatch {
        role: &'static str,
        declared: u64,
        recorded: usize,
    },
    SidecarAssociation(ProofAssociationError),
    Compile(CompileFailure),
    NoTarget,
    NoSelection,
    Emit,
    UnrealizableNumerics,
    Toolchain,
    NoDevice,
    HostProfile,
    Load(LoadRejection),
    ProbeBaseline(LoadRejection),
    UnprobableEnvelope {
        detail: &'static str,
    },
    NotFailedClosed {
        probe: &'static str,
        outcome: String,
    },
    Interface(String),
    ForeignProgram {
        packaged: usize,
        compiled: usize,
    },
    UnboundBinding {
        slot: usize,
        target: String,
    },
    EmptyLaunch {
        skipped: bool,
    },
    /// The device refused the route, before any commit.
    ///
    /// Boxed because it is the largest variant by a wide margin and every other
    /// one would otherwise pay for it. It carries the phase and the class rather
    /// than a rendered string, so a caller decides whether to re-route,
    /// re-fetch, or stop without parsing this.
    DevicePreflight(Box<PreflightRefusal>),
    /// An injected device-preflight perturbation was *accepted*.
    ///
    /// A probe that cannot fail measures nothing, so a perturbation the device
    /// admits is reported as loudly as a refusal that arrives in the wrong
    /// stage.
    ProbeAccepted(&'static str),
    LibraryLoad(String),
    FunctionLookup(String),
    Pipeline(String),
    Dispatch(String),
    Mismatch {
        path: &'static str,
        device: Vec<u32>,
        reference: Vec<u32>,
    },
}

impl fmt::Display for ProofError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str(
                "usage: tiler-prototype-run --artifact <path>; create it first with \
                 `cargo run -p tiler-prototype-compile -- --out <path>`",
            ),
            Self::Read(path, cause) => write!(formatter, "{path} could not be read: {cause}"),
            Self::SidecarWithoutCases => formatter.write_str(
                "the proof sidecar carries no case with an input and an expected output",
            ),
            Self::SidecarShapeMismatch {
                role,
                declared,
                recorded,
            } => write!(
                formatter,
                "the artifact declares {declared} {role} element(s), which is {} byte(s), \
                 and the sidecar records {recorded}",
                declared.saturating_mul(F32_BYTES),
            ),
            Self::Sidecar(cause) => {
                write!(formatter, "the proof sidecar did not decode: {cause}")
            }
            Self::SidecarAssociation(cause) => write!(
                formatter,
                "the proof sidecar does not describe this envelope: {cause}"
            ),
            Self::Compile(failure) => write!(formatter, "the program did not compile: {failure:?}"),
            Self::NoTarget => formatter.write_str("the compilation returned no target profile"),
            Self::NoSelection => formatter.write_str("the portfolio retained no selected plan"),
            Self::Emit => formatter.write_str("the selected kernels have no Metal realization"),
            Self::UnrealizableNumerics => formatter
                .write_str("the target cannot honour the kernels' declared numerical contract"),
            Self::Toolchain => formatter.write_str("the offline toolchain produced no metallib"),
            Self::NoDevice => formatter.write_str("no system default Metal device"),
            Self::HostProfile => formatter
                .write_str("the compiler's target profile does not compose a host environment"),
            Self::Load(rejection) => write!(formatter, "the artifact was refused: {rejection}"),
            Self::ProbeBaseline(rejection) => write!(
                formatter,
                "the fail-closed probes have no accepted neighbour to perturb: the unperturbed \
                 subject was itself refused: {rejection}",
            ),
            Self::UnprobableEnvelope { detail } => write!(
                formatter,
                "a fail-closed probe could not be constructed from these bytes: {detail}",
            ),
            Self::NotFailedClosed { probe, outcome } => write!(
                formatter,
                "the loader did not fail closed on {probe}: {outcome}",
            ),
            Self::Interface(detail) => write!(
                formatter,
                "the artifact's interface is not this program's: {detail}",
            ),
            Self::ForeignProgram { packaged, compiled } => write!(
                formatter,
                "the artifact packages a kernel program of {packaged} identity bytes and this \
                 process compiled one of {compiled}; the two prototypes have drifted",
            ),
            Self::UnboundBinding { slot, target } => write!(
                formatter,
                "ABI slot {slot} addresses {target}, which this proof binds no storage for",
            ),
            Self::EmptyLaunch { skipped } => write!(
                formatter,
                "the routed launch covers no threads (skipped: {skipped}), so there is no result \
                 to compare",
            ),
            Self::DevicePreflight(refusal) => write!(
                formatter,
                "this device refused the route before the commit: {refusal}",
            ),
            Self::ProbeAccepted(probe) => write!(
                formatter,
                "the device preflight accepted {probe}, so that probe proves nothing",
            ),
            Self::LibraryLoad(cause) => write!(formatter, "the metallib did not load: {cause}"),
            Self::FunctionLookup(cause) => write!(formatter, "the entry point is absent: {cause}"),
            Self::Pipeline(cause) => write!(formatter, "no compute pipeline state: {cause}"),
            Self::Dispatch(cause) => write!(formatter, "the command buffer failed: {cause}"),
            Self::Mismatch {
                path,
                device,
                reference,
            } => write!(
                formatter,
                "the {path} path returned {device:08x?}, reference requires {reference:08x?}",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    //! The fail-closed probes, carried into the repository gate.
    //!
    //! # What is asserted, and why a refusal alone would not be worth asserting
    //!
    //! Every case below runs one of the crate's own probe functions against an
    //! envelope this module assembles, and each of those functions pins the
    //! *class* of the refusal rather than the fact of one. The class is the
    //! property: it decides whether a reader re-fetches bytes, looks for a
    //! different file, rebuilds a plan, or rebuilds an object, and a loader that
    //! started reporting a corrupt file as `NoApplicableVariant` would still
    //! refuse every one of these inputs.
    //!
    //! Each case additionally asserts the rendered class prefix at the call
    //! site, so the guarantee is legible where it is claimed as well as where it
    //! is enforced. [`the_unperturbed_envelope_routes`] is the neighbour they
    //! are all paired against: without it a harness that produced garbage would
    //! refuse everything and report a fail-closed loader.
    //!
    //! # The closure taken, and the two that were eliminated
    //!
    //! The probes need a *valid* artifact, and this workspace's only producer of
    //! one is `tiler-prototype-compile`. Three closures were available.
    //!
    //! **A checked-in envelope fixture — eliminated.** It is the cheapest and it
    //! is a claim on disk that outlives whatever produced it: an encoder change
    //! leaves the fixture testing a format nobody emits any more, and nothing in
    //! the repository compares the two. `AGENTS.md` governs exactly this shape of
    //! retained artifact, and no predicate over a byte fixture survives an edit
    //! to the encoder beside it.
    //!
    //! **A unit test inside `tiler-runtime` — eliminated by scope rather than by
    //! design, and it is the better home.** `ArtifactProgramBuilder::new` takes a
    //! `tiler_ir::semantic::SemanticProgram`, and `tiler-runtime` depends on
    //! `tiler-artifact` alone, so an in-crate test needs a `tiler-ir`
    //! dev-dependency. That edits `Cargo.lock`, which is the
    //! `implementation/cargo-lock` scope this ticket does not hold, and
    //! `cargo test --locked` would refuse the change. Relocating these cases into
    //! the crate is a move, not new evidence.
    //!
    //! **Assembling the envelope here — taken.** This crate's `[[bin]]` declares
    //! `test = true`, so `cargo test --workspace --locked` — the exact command
    //! `scripts/check_rust.py` runs — builds and runs this module, and the crate
    //! already depends on every crate the assembly needs. Nothing can go stale:
    //! the envelope is minted by the live builder through the live encoder in the
    //! same compilation as the loader under test, so a builder or encoder change
    //! is a build failure rather than a fixture that quietly describes
    //! yesterday's format.
    //!
    //! # What this fixture is not
    //!
    //! It is a loader fixture, not a second producer. It substitutes a synthetic
    //! carried payload for a real `xcrun` link, which the loader can neither
    //! observe nor interpret: a payload's object bytes are opaque to every check
    //! `DecodedProgram` performs. The substitution is what keeps these cases
    //! device-free and toolchain-free, so they hold on both CI profiles rather
    //! than only where a Metal toolchain exists. It is deliberately *not*
    //! evidence about what the producer emits; `prototypes/serial-sum-compile`
    //! owns that, and the binary above carries these same probes onto a real
    //! artifact on hardware.
    //!
    //! # Why this is not the duplication a closed ticket rejected
    //!
    //! `share-the-serial-sum-artifact-assembler` considered exactly this file as
    //! its option (c) — "duplicate the assembler into `prototypes/serial-sum-run`"
    //! — and rejected it, on the ground that "two independently maintained
    //! descriptions of one compilation is the exact defect the routing ticket
    //! exists to remove". That rejection is correct and still binding for the
    //! case it was about, and it does not cover this one. The distinction is not
    //! size: [`assemble`] is comparable in length to the producer's.
    //!
    //! It was about an assembler on the **proof's own path**, giving the runner
    //! an in-process `VerifiedArtifactProgram` to dispatch from *instead of* the
    //! producer's file. Two such assemblers really are two descriptions of one
    //! compilation, and the proof would have had no way to tell which it ran.
    //! This one is `#[cfg(test)]`, reaches no device, and is never named by
    //! [`run`]: the hardware proof still reads the producer's envelope, and this
    //! assembly cannot substitute for it. Nor does anything here compare the two
    //! or claim they agree — the fixture's only obligation is to be *a* valid
    //! artifact, which the artifact layer decides on its own terms.
    //!
    //! What that leaves is a real and bounded drift risk, stated rather than
    //! dismissed: a builder or encoder change breaks this at compile time, but a
    //! change to what the *producer chooses* to package — a deferred predicate,
    //! a second variant — would leave this fixture valid and no longer shaped
    //! like the artifact it stands in for. It would then exercise a different
    //! legal envelope, which is a weaker probe rather than a wrong one, and the
    //! hardware run is what would notice.

    use super::{
        BACKEND_KEY, ProbeSubject, REPRESENTATION_KEY, ROWS, bind_interface, host_environment,
        probe_accepted_baseline, probe_damaged_interior_byte, probe_damaged_section_content,
        probe_foreign_expected_identity, probe_other_backend_family,
        probe_other_profile_descriptor, probe_truncated_envelope, serial_sum_program,
    };
    use tiler_artifact::program::{
        AbiExprId, AbiFacts, ArtifactExecutionPolicy, ArtifactProgramBuilder, BackendEntryKey,
        BackendEntryRef, BackendKey, BindingKind, BindingSpec, CapabilityKey,
        CompilationEnvironment, EntrySpec, FeasibilityRuleSetKey, FeasibilityRuleSetRef,
        LaunchSpec, PayloadContent, PayloadEntryMapping, PayloadMetadata, PayloadProvenance,
        PayloadSdkIdentity, RepresentationKey, SchemaVersion, SelectedProvider,
        TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef, ToolComponent,
        VariantSpec, VerifiedArtifactProgram,
    };
    use tiler_compiler::session::{
        Compilation, NumericalContract, PlanAlternative, compile_governed,
    };
    use tiler_ir::program::abi::ExprNode;
    use tiler_ir::semantic::SemanticProgram;
    use tiler_runtime::load::{DecodedProgram, ExecutionEnvironment};

    /// Columns of the fixture's input; the reduced axis.
    ///
    /// **One, and not by choice**, for the same measured reason
    /// `prototypes/serial-sum-compile` packages one: a `BackendEntryKey` is
    /// bounded at `MAX_OPAQUE_IDENTITY_BYTES` = 1,024 and the canonical kernel
    /// identity of a serial sum with two or more contributors measures 1,113
    /// bytes, so an entry keyed on it does not construct. The fixture keys its
    /// entry on that identity rather than on a short synthetic string precisely
    /// so it inherits the producer's real constraint instead of quietly routing
    /// around it. `bound-the-backend-entry-key-by-the-identity-it-carries` owns
    /// closing the gap; when it does, this becomes [`super::COLUMNS`].
    const FIXTURE_COLUMNS: u64 = 1;

    /// The object bytes the fixture's payload carries.
    ///
    /// Never loaded, never parsed, never compared. `DecodedProgram` treats a
    /// payload's object as opaque — its content digest is integrity rather than
    /// identity, and no loader check reads a byte of it — so a real `metallib`
    /// would change nothing these cases assert and would tie them to a host with
    /// a Metal toolchain.
    const PROBE_OBJECT: &[u8] = b"tiler probe object; not an executable image";

    /// One assembled envelope and everything a probe needs to route it.
    ///
    /// Owned rather than borrowed from the compilation that produced it, so a
    /// case can hold the subject after the `Compilation` has been dropped.
    struct Fixture {
        bytes: Vec<u8>,
        expected: Vec<u8>,
        environment: ExecutionEnvironment,
        abi: AbiFacts,
    }

    impl Fixture {
        fn subject(&self) -> ProbeSubject<'_> {
            ProbeSubject {
                bytes: &self.bytes,
                expected: &self.expected,
                environment: &self.environment,
                abi: &self.abi,
            }
        }
    }

    /// Compiles, packages, and encodes one valid envelope for the probes.
    ///
    /// The three facts a probe perturbs are each taken from the authority that
    /// owns it, exactly as the binary takes them: the expected identity from the
    /// artifact this function assembled, the host environment from the compiler's
    /// own target registry rather than from the artifact, and the ABI facts from
    /// the interface the *decoded* envelope declares. Reading any of them back
    /// out of the envelope would make the corresponding probe a tautology.
    fn fixture() -> Fixture {
        let semantic = serial_sum_program(ROWS, FIXTURE_COLUMNS);
        let compilations = compile_governed(&semantic, NumericalContract::FlushSubnormalsToZeroF32)
            .expect("the governed program compiles");
        let compilation = compilations.first().expect("one governed target profile");
        let plan = compilation.selected().expect("a selected plan alternative");

        let artifact = assemble(&semantic, compilation, plan);
        let bytes = artifact.encode().expect("the envelope encodes");
        let expected = artifact.canonical_identity().as_bytes().to_vec();
        let environment = host_environment(compilation).expect("the host environment composes");

        let decoded = DecodedProgram::decode(&bytes).expect("the assembled envelope decodes");
        let (_, _, abi) = bind_interface(&decoded).expect("the declared interface binds");
        Fixture {
            bytes,
            expected,
            environment,
            abi,
        }
    }

    /// Packages one plan alternative and a synthetic payload as an artifact.
    ///
    /// Deliberately a second, smaller assembler rather than a reach into
    /// `prototypes/serial-sum-compile`: that one lives in a `[[bin]]`-only
    /// package in another ticket scope, so it is not linkable from here at all.
    /// What it shares with the producer is everything a loader can observe — the
    /// compiler's own expressions, entry keys, target profile, and rule set — and
    /// what it omits is the toolchain.
    #[allow(
        clippy::too_many_lines,
        reason = "one artifact is assembled top to bottom in the order the builder requires, and that order is the readable part"
    )]
    fn assemble(
        semantic: &SemanticProgram,
        compilation: &Compilation,
        plan: PlanAlternative<'_>,
    ) -> VerifiedArtifactProgram {
        let profile = TargetProfileRef {
            key: TargetProfileKey::new(compilation.target_profile_key())
                .expect("the compiler mints a governed profile key"),
            descriptor: TargetProfileDescriptorDigest::from_bytes(
                compilation.target_profile_descriptor(),
            )
            .expect("the compiler mints a profile descriptor"),
        };
        let rules = FeasibilityRuleSetRef {
            key: FeasibilityRuleSetKey::new(compilation.feasibility_rule_set_key())
                .expect("the compiler mints a governed rule-set key"),
            revision: compilation.feasibility_rule_set_revision(),
        };

        let environment = CompilationEnvironment::new(
            plan.selected_capabilities()
                .map(|selected| selected.provider().clone()),
        )
        .expect("the offered providers compose an environment");
        let mut builder =
            ArtifactProgramBuilder::new(semantic, environment).expect("a builder identity remains");
        for selected in plan.selected_capabilities() {
            builder
                .select_provider(SelectedProvider {
                    provider: selected.provider().clone(),
                    capability: CapabilityKey::new(selected.capability_key())
                        .expect("the compiler mints a governed capability key"),
                    capability_revision: selected.capability_revision(),
                })
                .expect("a selected provider was offered");
        }

        let abi = plan.abi();
        let program = abi.kernel_program();

        // One mapping per stage, keyed on the same canonical kernel identity the
        // artifact's executable entry names, because the decoder proves the two
        // tables correlate and a mapping keyed on anything else is refused as an
        // unmapped backend entry.
        let mut mappings: Vec<PayloadEntryMapping> = abi
            .entries()
            .zip(program.stages())
            .enumerate()
            .map(|(position, (entry, stage))| PayloadEntryMapping {
                entry_key: BackendEntryKey::from_bytes(
                    stage.kernel().canonical_identity().as_bytes(),
                )
                .expect("the packaged kernel identity fits a backend entry key"),
                symbol: format!("tiler_probe_entry_{position}"),
                transports: (0..u32::try_from(entry.accessible_bytes().len())
                    .expect("a bounded binding count fits a u32"))
                    .collect(),
            })
            .collect();
        mappings.sort_by(|left, right| left.entry_key.cmp(&right.entry_key));

        let payload = builder
            .push_carried_payload(
                BackendKey::new(BACKEND_KEY).expect("a governed backend key"),
                RepresentationKey::new(REPRESENTATION_KEY).expect("a governed representation key"),
                SchemaVersion::new(1, 0),
                profile.clone(),
                // The loader refuses anything a device-free path cannot deliver,
                // so the fixture declares a native image for the accepted
                // neighbour to exist at all.
                ArtifactExecutionPolicy::NativeImage,
                PayloadContent {
                    metadata: PayloadMetadata {
                        source_representation: RepresentationKey::new("tiler.probe.source")
                            .expect("a governed representation key"),
                        source: b"// the probe fixture compiles nothing".to_vec(),
                        provenance: PayloadProvenance {
                            toolchain: "tiler.probe.toolchain".to_owned(),
                            target: "tiler-probe-target".to_owned(),
                            family: "tiler.probe.family".to_owned(),
                            language: "tiler.probe.language".to_owned(),
                            deployment_major: 1,
                            deployment_minor: 0,
                            components: vec![ToolComponent {
                                role: "compiler".to_owned(),
                                version: "0".to_owned(),
                            }],
                            sdk: PayloadSdkIdentity {
                                name: "tiler.probe.sdk".to_owned(),
                                version: "0".to_owned(),
                                build: "0".to_owned(),
                            },
                            compile_flags: Vec::new(),
                            link_flags: Vec::new(),
                        },
                        entries: mappings,
                        obligations: Vec::new(),
                    },
                    code: PROBE_OBJECT.to_vec(),
                },
            )
            .expect("the synthetic payload is carried");

        let minted = replay(&mut builder, abi.expressions(), &variant_roots(plan));
        let resolve = |position: u32| {
            minted[usize::try_from(position).expect("a bounded arena position fits a usize")]
                .expect("every use site names a position the variant's own roots reach")
        };

        let entries: Vec<EntrySpec> = abi
            .entries()
            .zip(program.stages())
            .map(|(entry, stage)| EntrySpec {
                bindings: entry
                    .accessible_bytes()
                    .map(|position| BindingSpec {
                        kind: BindingKind::Buffer,
                        accessible_bytes: resolve(position),
                    })
                    .collect(),
                launch: LaunchSpec {
                    grid_threads: resolve(entry.grid_threads()),
                    threads_per_workgroup: resolve(entry.threads_per_workgroup()),
                    // Not a choice: `tiler_ir::schedule`'s intrinsic verifier
                    // refuses a scheduled region whose launch plan does not skip a
                    // zero-thread dispatch, so every verified region carries it.
                    zero_work_skips_dispatch: true,
                    preconditions: Vec::new(),
                },
                implementation: BackendEntryRef {
                    payload,
                    entry_key: BackendEntryKey::from_bytes(
                        stage.kernel().canonical_identity().as_bytes(),
                    )
                    .expect("the packaged kernel identity fits a backend entry key"),
                },
            })
            .collect();

        builder
            .push_variant(
                program,
                VariantSpec {
                    applicability_guard: resolve(abi.applicability_guard()),
                    target_profile: profile,
                    feasibility_rules: rules,
                    deferred_predicates: Vec::new(),
                    entries,
                },
            )
            .expect("the variant packages the plan it was built from");
        builder.build().expect("the assembled artifact verifies")
    }

    /// Returns the arena positions one variant names directly.
    fn variant_roots(plan: PlanAlternative<'_>) -> Vec<u32> {
        let abi = plan.abi();
        let mut roots = vec![abi.applicability_guard()];
        for entry in abi.entries() {
            roots.extend(entry.accessible_bytes());
            roots.push(entry.grid_threads());
            roots.push(entry.threads_per_workgroup());
        }
        roots
    }

    /// Transliterates the reachable sub-DAG of one arena onto the builder's own.
    ///
    /// Pruned to the variant's roots rather than replayed wholesale, because the
    /// artifact layer refuses an arena node no use site reaches and the compiler's
    /// canonical graph serves both plan alternatives, so one variant's use sites
    /// reach a subset of it. Whether a wholesale replay would survive on any
    /// particular graph is a question about that graph — the builder deduplicates
    /// by content key, so it survives when every unreachable node repeats content
    /// a reachable one carries — and this fixture does not depend on the answer.
    ///
    /// One forward pass suffices: operands precede the node naming them in the
    /// compiler's arena, and the reachable set is operand-closed.
    fn replay(
        builder: &mut ArtifactProgramBuilder,
        arena: &[ExprNode],
        roots: &[u32],
    ) -> Vec<Option<AbiExprId>> {
        let reachable = reachable_from(arena, roots);
        let mut minted: Vec<Option<AbiExprId>> = vec![None; arena.len()];
        let resolve = |minted: &[Option<AbiExprId>], position: u32| {
            minted[usize::try_from(position).expect("a bounded arena position fits a usize")]
                .expect("an operand precedes the node naming it")
        };
        for (position, node) in arena.iter().enumerate() {
            if !reachable[position] {
                continue;
            }
            let id = match node {
                ExprNode::Root(root) => builder.push_root(root.clone()),
                ExprNode::Unary { op, operand } => {
                    builder.push_unary(*op, resolve(&minted, *operand))
                }
                ExprNode::Binary { op, left, right } => {
                    builder.push_binary(*op, resolve(&minted, *left), resolve(&minted, *right))
                }
                ExprNode::Select {
                    condition,
                    if_true,
                    if_false,
                } => builder.push_select(
                    resolve(&minted, *condition),
                    resolve(&minted, *if_true),
                    resolve(&minted, *if_false),
                ),
            }
            .expect("a well-typed compiler expression replays onto the artifact arena");
            minted[position] = Some(id);
        }
        minted
    }

    /// Marks every arena position reachable from a set of use sites.
    fn reachable_from(arena: &[ExprNode], roots: &[u32]) -> Vec<bool> {
        let mut reached = vec![false; arena.len()];
        let mut work: Vec<u32> = roots.to_vec();
        while let Some(node) = work.pop() {
            let at = usize::try_from(node).expect("a bounded arena position fits a usize");
            if reached[at] {
                continue;
            }
            reached[at] = true;
            match &arena[at] {
                ExprNode::Root(_) => {}
                ExprNode::Unary { operand, .. } => work.push(*operand),
                ExprNode::Binary { left, right, .. } => {
                    work.push(*left);
                    work.push(*right);
                }
                ExprNode::Select {
                    condition,
                    if_true,
                    if_false,
                } => {
                    work.push(*condition);
                    work.push(*if_true);
                    work.push(*if_false);
                }
            }
        }
        reached
    }

    /// This half of the filename interface, pinned.
    ///
    /// `prototypes/serial-sum-compile` carries the identical assertion. The two
    /// crates share no code, so this pair of tests is the only thing that
    /// compares their idea of the name, and a rename that updates one fails in
    /// the other.
    #[test]
    fn the_sidecar_suffix_is_the_one_the_producer_writes() {
        assert_eq!(super::SIDECAR_SUFFIX, ".proof");
    }

    /// A payload that is not exactly the declared element count is refused as a
    /// sidecar defect, not carried into the numerical comparison.
    ///
    /// The three lengths are the three ways a record can disagree with the
    /// interface it names, and the middle one is why this is a length check
    /// rather than a chunk count: a payload one byte short of two elements has
    /// a whole first element, so truncating to whole chunks would decode it and
    /// report the missing element as a device disagreement.
    #[test]
    fn a_payload_that_is_not_the_declared_length_is_a_sidecar_defect() {
        assert_eq!(
            super::decode_f32_bits("input", 2, &[0, 0, 0, 1, 0, 0, 0, 2])
                .expect("the exact length decodes"),
            vec![1, 2],
        );
        for bytes in [
            &[0, 0, 0, 1, 0, 0, 0][..],       // one byte short of two elements
            &[0, 0, 0, 1][..],                // one element where two are declared
            &[0, 0, 0, 1, 0, 0, 0, 2, 0][..], // two elements and a trailing byte
        ] {
            let refusal = super::decode_f32_bits("input", 2, bytes)
                .expect_err("a payload of the wrong length is refused");
            assert!(
                matches!(refusal, super::ProofError::SidecarShapeMismatch { .. }),
                "a malformed record must not be reported as arithmetic: {refusal}",
            );
        }
    }

    /// Every device-preflight refusal lands in the phase and class it claims.
    ///
    /// The classification is what a caller acts on — re-route, re-fetch, or stop
    /// — so a refusal filed under the wrong class is a wrong instruction rather
    /// than a wrong label. Each variant is listed explicitly rather than derived
    /// from the functions under test, so a variant that silently changed class
    /// fails here instead of agreeing with itself.
    #[test]
    fn each_device_preflight_refusal_carries_its_phase_and_class() {
        let cases = [
            (
                super::PreflightRefusal::LibraryRejected {
                    detail: "not a metallib".to_owned(),
                },
                super::PreflightPhase::Library,
                super::PreflightClass::CorruptArtifact,
            ),
            (
                super::PreflightRefusal::FunctionAbsent {
                    symbol: "absent".to_owned(),
                    detail: "no such function".to_owned(),
                },
                super::PreflightPhase::Function,
                super::PreflightClass::CorruptArtifact,
            ),
            (
                super::PreflightRefusal::PipelineRejected {
                    symbol: "k".to_owned(),
                    detail: "too many registers".to_owned(),
                },
                super::PreflightPhase::Pipeline,
                super::PreflightClass::RouteMiss,
            ),
            (
                super::PreflightRefusal::WorkgroupTooLarge {
                    symbol: "k".to_owned(),
                    declared: 2,
                    capacity: 1,
                },
                super::PreflightPhase::LaunchGeometry,
                super::PreflightClass::RouteMiss,
            ),
            (
                super::PreflightRefusal::BindingExceedsBufferLimit {
                    slot: 0,
                    needed: 2,
                    limit: 1,
                },
                super::PreflightPhase::Resources,
                super::PreflightClass::RouteMiss,
            ),
            (
                super::PreflightRefusal::UndersizedAllocation {
                    slot: 0,
                    needed: 2,
                    held: 1,
                },
                super::PreflightPhase::Resources,
                super::PreflightClass::Systemic,
            ),
        ];
        assert_eq!(cases.len(), 6, "a refusal was added without a case here");
        for (refusal, phase, class) in cases {
            assert_eq!(refusal.phase(), phase, "wrong phase for {refusal}");
            assert_eq!(refusal.class(), class, "wrong class for {refusal}");
            // The rendered form leads with both, because a log line that does
            // not carry the class makes the reader infer what the type states.
            let rendered = refusal.to_string();
            assert!(
                rendered.starts_with(&format!("{}/{}: ", phase.as_str(), class.as_str())),
                "the rendering drops the phase or the class: {rendered}",
            );
        }
    }

    /// The three comparisons refuse exactly at their boundary, not near it.
    ///
    /// Each is tested at the largest accepted value and the smallest refused
    /// one, because an off-by-one here either rejects a route the device would
    /// have run or admits one it cannot — and the second is the failure the
    /// whole stage exists to move before the commit.
    #[test]
    fn the_device_comparisons_refuse_exactly_at_their_boundary() {
        super::workgroup_fits("k", 1024, 1024).expect("a workgroup at capacity fits");
        assert!(matches!(
            super::workgroup_fits("k", 1025, 1024),
            Err(super::PreflightRefusal::WorkgroupTooLarge {
                declared: 1025,
                capacity: 1024,
                ..
            })
        ));

        super::binding_fits(0, 4096, 4096).expect("a binding at the limit fits");
        assert!(matches!(
            super::binding_fits(0, 4097, 4096),
            Err(super::PreflightRefusal::BindingExceedsBufferLimit {
                slot: 0,
                needed: 4097,
                limit: 4096,
            })
        ));

        super::allocation_fits(0, 48, 48).expect("an allocation of exactly the needed length fits");
        super::allocation_fits(0, 48, 64).expect("a longer allocation fits");
        assert!(matches!(
            super::allocation_fits(0, 48, 47),
            Err(super::PreflightRefusal::UndersizedAllocation {
                slot: 0,
                needed: 48,
                held: 47,
            })
        ));
    }

    /// The accepted neighbour every refusal below is evidence against.
    ///
    /// Asserted first and separately because it is what the other cases borrow
    /// their meaning from. A subject that never routed would refuse each
    /// perturbation under some plausible class, and the suite would report a
    /// fail-closed loader while measuring nothing at all.
    #[test]
    fn the_unperturbed_envelope_routes() {
        let fixture = fixture();
        let outcome =
            probe_accepted_baseline(&fixture.subject()).expect("the assembled envelope routes");
        // The geometry, not merely the fact of a route: it is evaluated from the
        // artifact's own launch expression against the facts bound from the
        // decoded interface, so one thread per reduced row is evidence that the
        // preflight reached and answered that expression rather than stopping
        // somewhere earlier with a `Preflight` that happens to exist.
        assert!(
            outcome.contains(&format!("{ROWS} thread(s)")),
            "the reduction launches one thread per row: {outcome}",
        );
    }

    /// A damaged section is an integrity failure, not a route miss.
    #[test]
    fn a_damaged_section_is_an_integrity_failure() {
        let fixture = fixture();
        let outcome = probe_damaged_section_content(&fixture.subject())
            .expect("a flipped section byte is refused as an integrity failure");
        assert!(
            outcome.contains("artifact.integrity"),
            "the refusal names the integrity class: {outcome}",
        );
    }

    /// A flipped byte at an arbitrary offset is refused by the artifact layer.
    ///
    /// The exact class is deliberately not pinned here; see the probe for why.
    #[test]
    fn a_flipped_interior_byte_never_reaches_routing() {
        let fixture = fixture();
        let outcome = probe_damaged_interior_byte(&fixture.subject())
            .expect("a flipped interior byte is refused by the artifact layer");
        assert!(
            outcome.contains("runtime.artifact"),
            "the refusal is the artifact layer's own: {outcome}",
        );
    }

    /// A truncated envelope is malformed, not damaged and not inapplicable.
    #[test]
    fn a_truncated_envelope_is_malformed() {
        let fixture = fixture();
        let outcome = probe_truncated_envelope(&fixture.subject())
            .expect("a truncated envelope is refused as malformed");
        assert!(
            outcome.contains("artifact.malformed"),
            "the refusal names the malformed class: {outcome}",
        );
    }

    /// A valid artifact that is not the expected one is a program mismatch.
    #[test]
    fn a_foreign_expected_identity_is_a_program_mismatch() {
        let fixture = fixture();
        let outcome = probe_foreign_expected_identity(&fixture.subject())
            .expect("a foreign expected identity is refused as a program mismatch");
        assert!(
            outcome.contains("runtime.program-mismatch"),
            "the refusal names the program-mismatch class: {outcome}",
        );
    }

    /// Another profile descriptor is an incompatible target on the variant.
    #[test]
    fn another_profile_descriptor_is_an_incompatible_target() {
        let fixture = fixture();
        let outcome = probe_other_profile_descriptor(&fixture.subject())
            .expect("another profile descriptor is refused as an incompatible target");
        assert!(
            outcome.contains("runtime.incompatible-target"),
            "the refusal names the incompatible-target class: {outcome}",
        );
        assert!(
            outcome.contains("DescriptorMismatch"),
            "the refusal separates a rebuild from a wrong artifact: {outcome}",
        );
    }

    /// Another backend family is an unexecutable payload, not a profile problem.
    #[test]
    fn another_backend_family_is_an_unexecutable_payload() {
        let fixture = fixture();
        let outcome = probe_other_backend_family(&fixture.subject())
            .expect("another backend family is refused as an unexecutable payload");
        assert!(
            outcome.contains("runtime.unexecutable-payload"),
            "the refusal names the unexecutable-payload class: {outcome}",
        );
    }
}

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

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use metal::{
    Buffer, ComputeCommandEncoderRef, ComputePipelineDescriptor, ComputePipelineState, Device,
    MTLCommandBufferStatus, MTLResourceOptions, MTLSize,
};
use tiler_artifact::program::{
    AbiFactBinder, AbiFacts, AvailabilityPhase, BackendKey, BindingTarget, RepresentationKey,
    TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
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
use tiler_runtime::load::{DecodedProgram, ExecutionEnvironment, LoadRejection, RoutedDispatch};

/// Rows of the direct path's input; each row reduces to one output element.
const ROWS: u64 = 4;
/// Columns of the direct path's input; the reduced axis.
///
/// The direct path fixes its own shape because its job is the *numerical*
/// claim, and three contributors per row is what makes a serial reduction's
/// ordering observable. The envelope path deliberately does not fix one: it
/// takes whatever shape the artifact declares, so that when the artifact layer
/// can carry this shape the two coincide with no code change. See
/// [`bind_interface`] for why they do not coincide yet.
const COLUMNS: u64 = 3;
/// Buffer argument-table capacity Apple states per compute function.
const BUFFER_BINDING_LIMIT: u32 = 31;
/// Interface key of the program's one input.
const INPUT_KEY: &str = "input";
/// Interface key of the program's one output.
const OUTPUT_KEY: &str = "result";
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
fn read_artifact(path: &Path) -> Result<(Vec<u8>, Vec<u8>), ProofError> {
    let mut sidecar = path.as_os_str().to_owned();
    sidecar.push(".identity");
    let sidecar = PathBuf::from(sidecar);
    let bytes =
        std::fs::read(path).map_err(|cause| ProofError::Read(path.display().to_string(), cause))?;
    let identity = std::fs::read(&sidecar)
        .map_err(|cause| ProofError::Read(sidecar.display().to_string(), cause))?;
    println!(
        "artifact: {} ({} bytes), expected identity {} bytes",
        path.display(),
        bytes.len(),
        identity.len(),
    );
    Ok((bytes, identity))
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
/// that is a limitation being tracked rather than a design.** The artifact layer
/// bounds a `BackendEntryKey` at `MAX_OPAQUE_IDENTITY_BYTES` = 1,024, and the
/// canonical kernel identity of any serial sum with two or more contributors
/// measures 1,113 bytes, so the producer can package only the degenerate
/// single-contributor reduction. Fixing the shape here would make this path
/// unreachable rather than make it stronger.
/// `bound-the-backend-entry-key-by-the-identity-it-carries` records the exact
/// measurement; when it closes, the producer packages the direct path's own
/// shape and the two coincide with no change here.
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
    buffer::write_f32(&input, &operands);
    (input, output, count)
}

/// Submits one encoded command buffer and reads the output back.
///
/// The command buffer's terminal state is checked *before* the host reads
/// anything. A failed submission leaves the output buffer holding whatever it
/// held before, and comparing that against the reference would report a
/// numerical disagreement for what is actually a dispatch failure.
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
    Ok(buffer::read_f32(output, count)
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

/// Dispatches a committed route, using nothing this process compiled.
///
/// The object image, the entry symbol, the argument-table index of every buffer,
/// how many bytes each must reach, and the launch geometry are all read from the
/// route. The only thing this function contributes is the host storage the
/// artifact's own interface keys name.
fn dispatch_routed(
    device: &Device,
    routed: &RoutedDispatch<'_>,
    bits: &[u32],
    rows: u64,
) -> Result<Vec<u32>, ProofError> {
    let pipeline = pipeline_for(device, routed.object(), routed.entry_symbol())?;
    let (input, output, count) = host_storage(device, bits, rows);

    // Retained until the command buffer completes: entry-internal storage is the
    // loader's to allocate, and dropping it at the end of this loop would leave
    // the encoder holding a binding to a freed allocation.
    let mut placements = Vec::with_capacity(routed.bindings().len());
    for binding in routed.bindings() {
        let needed = binding.accessible_bytes();
        let storage = match binding.binding().target() {
            BindingTarget::ProgramInput(key) if key.as_str() == INPUT_KEY => input.clone(),
            BindingTarget::ProgramOutput(keys)
                if keys.len() == 1 && keys[0].as_str() == OUTPUT_KEY =>
            {
                output.clone()
            }
            // Named by nothing, sized by its own accessible-byte expression, and
            // allocated rather than bound — which is what the artifact layer
            // says a loader does with one.
            BindingTarget::Internal => {
                device.new_buffer(needed.max(1), MTLResourceOptions::StorageModePrivate)
            }
            other => {
                return Err(ProofError::UnboundBinding {
                    slot: binding.slot(),
                    target: format!("{other:?}"),
                });
            }
        };
        if storage.length() < needed {
            return Err(ProofError::UndersizedBinding {
                slot: binding.slot(),
                needed,
                held: storage.length(),
            });
        }
        placements.push((binding.transport_slot(), storage));
    }

    let launch = routed.launch();
    let workgroup = launch.threads_per_workgroup();
    let capacity = pipeline.max_total_threads_per_threadgroup();
    if workgroup > capacity {
        return Err(ProofError::WorkgroupTooLarge {
            declared: workgroup,
            capacity,
        });
    }
    if launch.grid_threads() == 0 {
        // The artifact states whether an empty launch is skipped or encoded, so
        // the answer is read rather than assumed. Either way this proof has no
        // output to compare and says so instead of reporting agreement on
        // whatever the output buffer happened to hold.
        return Err(ProofError::EmptyLaunch {
            skipped: launch.zero_work_skips_dispatch(),
        });
    }

    submit(device, &output, count, |encoder| {
        encoder.set_compute_pipeline_state(&pipeline);
        for (transport, storage) in &placements {
            encoder.set_buffer(u64::from(*transport), Some(storage), 0);
        }
        encoder.dispatch_threads(
            MTLSize::new(launch.grid_threads(), 1, 1),
            MTLSize::new(workgroup, 1, 1),
        );
    })
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
    let (bytes, expected) = read_artifact(&envelope_path)?;
    let decoded = DecodedProgram::decode(&bytes).map_err(ProofError::Load)?;
    println!(
        "decoded: {} variant(s), required features {:?}",
        decoded.variant_count(),
        decoded.required_features(),
    );
    let (rows, columns, abi) = bind_interface(&decoded)?;
    println!("the artifact declares a {rows} by {columns} input");
    let environment = host_environment(compilation)?;
    let preflight = decoded
        .preflight(&environment, &expected, &abi)
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
    let envelope_bits = input_bits(rows, columns);
    let envelope_reference = reference_bits(&envelope_program, &envelope_bits, rows, columns);
    let envelope = dispatch_routed(device, &routed, &envelope_bits, rows)?;

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
    Compile(CompileFailure),
    NoTarget,
    NoSelection,
    Emit,
    UnrealizableNumerics,
    Toolchain,
    NoDevice,
    HostProfile,
    Load(LoadRejection),
    Interface(String),
    ForeignProgram {
        packaged: usize,
        compiled: usize,
    },
    UnboundBinding {
        slot: usize,
        target: String,
    },
    UndersizedBinding {
        slot: usize,
        needed: u64,
        held: u64,
    },
    WorkgroupTooLarge {
        declared: u64,
        capacity: u64,
    },
    EmptyLaunch {
        skipped: bool,
    },
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
            Self::UndersizedBinding { slot, needed, held } => write!(
                formatter,
                "ABI slot {slot} must reach {needed} byte(s) and the bound storage holds {held}",
            ),
            Self::WorkgroupTooLarge { declared, capacity } => write!(
                formatter,
                "the artifact declares {declared} threads per workgroup and this pipeline admits \
                 {capacity}",
            ),
            Self::EmptyLaunch { skipped } => write!(
                formatter,
                "the routed launch covers no threads (skipped: {skipped}), so there is no result \
                 to compare",
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

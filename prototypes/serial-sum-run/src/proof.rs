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
//! # Two different authority questions, and only one of them is claimed here
//!
//! **"Is this host eligible to offer the declared profile?"** is asked by
//! [`offer_the_declared_profile`], from a host observation and nothing else, and
//! the answer is always no: [ADR 0086](../../../docs/decisions/0086-require-attributable-or-attested-native-translation.md)
//! decides that native device translation of a metallib during pipeline creation
//! is a typed capability fact whose authority is `Unknown` on every macOS row
//! currently observable. That refusal is printed before any routing commit and
//! is one of this binary's deliverables.
//!
//! **"Does this artifact name the profile the producer declared?"** is what the
//! envelope path answers, through [`ExecutionEnvironment::classify`] against
//! [`declared_route_environment`]. It is **producer-declared equality, NOT
//! host-earned eligibility**, and the run says so in those words. Keeping that
//! route is Tom's recorded resolution for
//! `construct-and-bind-the-first-authoritative-metal-compile-profile`: the
//! runtime machinery — decode, route, ABI binding, two-stage qualification,
//! dispatch — is worth exercising on real hardware, and the honest way to keep
//! exercising it is to label what the route does and does not establish rather
//! than to let a green run read as an eligibility claim.
//!
//! What was removed is the previous middle position. The routed environment used
//! to be rebuilt from a local `Compilation`, which looked like an independent
//! host statement while restating the producer's own authority. Nothing in this
//! binary derives a profile offer from a compilation or from the artifact it is
//! validating any more.
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
//! # What holds the published shape, and where it is checked
//!
//! Nothing routed here fixes a shape: [`bind_interface`] reads one from the
//! artifact, and [`ROWS`] is the direct path's alone. That discipline is the
//! whole defence, and it is exactly what failed — [`prove_member`] compiled
//! `serial_sum_program(ROWS, columns)` against artifacts published with one row,
//! so every packaged program was foreign and the matrix proved nothing for a
//! month. The proof still passed everywhere it ran, because it ran only on
//! hardware, by hand.
//!
//! The shape is therefore held by a fixture the repository gate reaches:
//! `the_published_shape_matrix_survives_this_builds_shape_handling` assembles an
//! envelope at every shape the producer publishes, runs
//! [`compile_for_declared_shape`] and [`require_derived_program`] — the two
//! functions [`prove_member`] itself calls — and requires the packaged program
//! to be derived, then requires the same check to *refuse* when [`ROWS`] is
//! substituted for the declared rows. Both halves are device-free and
//! toolchain-free.
//!
//! Its validity condition is a pinned pair, in the same idiom as
//! [`SIDECAR_SUFFIX`]: the published shape matrix is asserted here and in
//! `prototypes/serial-sum-compile`, so a producer that moves to another shape
//! fails rather than leaving this fixture assembling envelopes nobody publishes.
//! That pair is not a second mechanism — on its own it would have watched both
//! halves agree on one row while the code used four.
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
//! `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32`, stated rather than defaulted.
//! Apple `f32` arithmetic flushes subnormals to the sign-preserving zero in
//! every math mode, so the strict contract is not deliverable here and emission
//! refuses it. Stating the flush contract makes running on this hardware a
//! choice this program made about what it means; the two contracts carry
//! different keys and different identities.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use metal::{
    Buffer, CommandBufferRef, ComputePipelineDescriptor, ComputePipelineState, Device,
    MTLCommandBufferStatus, MTLGPUFamily, MTLResourceOptions, MTLSize,
};
use tiler_artifact::program::{
    AbiFactBinder, AbiFacts, ArtifactCodecFailure, AvailabilityPhase, BackendKey, BindingTarget,
    RecordedArtifactIdentityError, RecordedArtifactProgramIdentity, RepresentationKey,
    RouteRequirement, RouteResourceDimension, TargetProfileDescriptorDigest, TargetProfileKey,
    TargetProfileRef,
};
use tiler_artifact::proof::{
    DecodedProofSidecar, ProofAssociationError, ProofCaseRef, ProofCodecError, decode_proof_sidecar,
};
use tiler_build::{BoundMetalCompileDeclaration, BoundMetalDeclarationError};
use tiler_compiler::session::{
    Compilation, CompileFailure, CompileRequest as CompilerRequest, NumericalContract,
    PlanAlternative, compile,
};
use tiler_compiler::target::{TargetRequest, TargetRequestError};
use tiler_ir::program::abi::{AbiRoot, ExprNode};
use tiler_ir::program::{ValueRole, VerifiedKernelProgram};
use tiler_ir::schedule::ContributorPartition;
use tiler_ir::semantic::{
    ContractionIndex, ContractionIndexStructure, F32, F32Add, F32Constant, F32Multiply,
    F32TensorContraction, InputKey, OutputKey, SemanticProgram, SemanticProgramBuilder,
    StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};
use tiler_metal::applicability::{
    AppleGpuFamilyConstant, MetalGpuFamily, MetalGpuFamilySupport, MetalHostApplicabilityPolicy,
    MetalHostApplicabilityRefusal, MetalHostObservation, evaluate_metal_host_applicability,
    observe_highest_gpu_family,
};
use tiler_metal::emit::emit_translation_unit;
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::input::{CompileRequest, OptimizationLevel};
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
    strict_partitioned_sum,
};
use tiler_runtime::load::{
    DecodedProgram, ExecutionEnvironment, LiveDeviceObservation, LiveDeviceQualification,
    LiveDeviceRequest, LoadRejection, Preflight, RoutePreparation, RoutedDispatch, RoutedEntry,
    TargetCompatibility, VariantIneligibility,
};

/// The one delivery position every artifact here is built for.
///
/// A delivery position is the ordered slot a consumer's build target resolves
/// to, and these artifacts are built for a single target, so the sole position
/// is zero. Named rather than written as a bare `0` at each call, because the
/// argument decides *which compiled object* is loaded and a literal there says
/// nothing about why that one.
const SOLE_DELIVERY: usize = 0;

/// Rows of the direct path's input; each row reduces to one output element.
///
/// The direct path's own number, and deliberately not the one the producer
/// publishes. Nothing on the envelope path may read it: an envelope's shape is
/// the artifact's to declare, and substituting this constant for the declared
/// rows is the exact defect the gate's published-shape case catches.
const ROWS: u64 = 4;
/// Columns of the direct path's input; the reduced axis.
///
/// The direct path fixes its own shape because its job is the *numerical*
/// claim, and three contributors per row is what makes a serial reduction's
/// ordering observable. The envelope path fixes none — it takes whatever shape
/// the artifact declares — and the two agree on the reduced extent while
/// deliberately disagreeing on the row count, which is what the gate's
/// published-shape case relies on.
const COLUMNS: u64 = 3;
/// Rows of the parallel strategies' input.
///
/// **One, because the widest stage has to fit the grid axis.** The
/// authoritative profile's `GridAxisThreads` row admits four threads, and the
/// multi-pass split's *pointwise* stage launches one invocation per element. At
/// one row of [`PARALLEL_COLUMNS`] contributors that is four, exactly at the
/// limit; a second row makes it eight and the whole compilation fails
/// `target.grid-axis` before any plan composes, so there would be no strategy
/// left to execute.
const PARALLEL_ROWS: u64 = 1;
/// Contributors reduced per output on the parallel strategies' input.
///
/// **Four, because below that nothing splits.** `governed_partition` requires at
/// least two partitions of at least two contributors each, so four is the
/// smallest extent at which a split or a workgroup tree exists to be retained at
/// all. It is also the smallest extent *above*
/// `correct-the-declined-strategy-record-for-an-unsplittable-reduction`, which
/// records a sub-four reduction failing with `InvalidCompilerOutput` under a
/// reassociation-permitting contract: this shape is sized above that defect
/// rather than around it, so a regression there fails here rather than hiding.
const PARALLEL_COLUMNS: u64 = 4;
/// The **contributor-set** half of the parallel operand pair.
///
/// **Every grouping is exact, which is what makes one serial-fold oracle valid
/// for all three strategies.** The contract these run under *permits* ordered
/// regrouping, so a split and a tree may legitimately sum in an order the
/// reference's declared left fold does not. Distinct small powers of two are
/// exactly representable and their partial sums are too, so every partition and
/// every tree depth produces the identical `f32`.
///
/// **Every subset has a distinct sum, so a dropped or double-counted
/// contributor cannot cancel.** These are the failure modes a parallel reduction
/// actually has — a partition boundary off by one, a participant whose partial
/// is never combined, an unsynchronized read of a partial written by another
/// invocation — and with powers of two each of them changes the result to a
/// value no correct grouping produces. [`ROW_PATTERNS`] would not do this job:
/// its rows repeat `1.0`, so dropping one contributor and double-counting
/// another agree.
///
/// **What it cannot say, stated exactly.** Because every grouping is exact,
/// every order-preserving regrouping of these four operands produces
/// `0x41700000` and *no other value* — so the comparison against the serial
/// fold has an empty refusal population among legal groupings and cannot
/// observe rounding at all. That is why it is one half of a pair rather than
/// the whole claim; [`GROUPING_SENSITIVE_OPERANDS`] is the other half, and
/// `the_operand_pair_covers_what_each_half_alone_cannot` in this file's test
/// module pins both counts. Named rather than linked: that module is
/// `#[cfg(test)]`, so an intra-doc link to it does not resolve when the crate is
/// documented, which is a rustdoc error rather than a dead link.
const PARALLEL_OPERANDS: [u32; 4] = [
    0x3f80_0000, // 1.0
    0x4000_0000, // 2.0
    0x4080_0000, // 4.0
    0x4100_0000, // 8.0
];
/// The **rounding** half of the parallel operand pair, chosen so the declared
/// regroupings disagree by exactly one rounding step.
///
/// Written as bit patterns rather than decimal literals because the whole point
/// is which representable value each operand is: `4.4703484e-8` names a printed
/// approximation, and `0x3340_0000` names the operand.
///
/// | bits | value | in units of `ulp(1.0)` = `2^-23` |
/// | --- | --- | --- |
/// | `0x3f40_0000` | `0.75` | — |
/// | `0x3e80_0000` | `0.25` | — |
/// | `0x3340_0000` | `3 * 2^-26` | `0.375` |
/// | `0x3300_0000` | `2^-25` | `0.25` |
///
/// **The derivation, so the two answers are attributable rather than merely
/// different.** `governed_partition(4)` is two partitions of two, so both
/// parallel strategies fold `(a0 + a1) + (a2 + a3)` while the serial fold folds
/// `((a0 + a1) + a2) + a3`; both share the prefix `0.75 + 0.25 = 1.0`, exact.
/// The serial fold then adds `0.375 ulp` and `0.25 ulp` in turn, and each lands
/// below the half-ulp boundary on its own, so each rounds back to `1.0`. The
/// declared regrouping adds them to each other first — `0.625 ulp`, exact,
/// because both are dyadic — and one add of `1.0 + 0.625 ulp` rounds *up*. So
/// the parallel answer is `0x3f800001` and the serial answer is `0x3f800000`:
/// one ULP apart, and the difference is one named rounding step rather than a
/// tolerance.
///
/// **No step is a tie**, deliberately: `0.375`, `0.25`, and `0.625` are each
/// strictly off the half-ulp boundary, so nothing here depends on round-half-to-
/// even and a host resolving ties differently would still produce these bits.
/// Every operand is normal — the smallest is `2^-25`, a hundred binades above
/// the subnormal boundary — so the flush-to-zero half of the contract changes
/// none of them, and `x * 1.0 + 0.0` is bit-identity on each, which is what lets
/// the reduction oracle be applied to these operands rather than to the
/// prologue's output.
///
/// **What it cannot say, stated exactly.** Its subset sums are *not* distinct:
/// of the sixteen single-contributor corruptions of the declared grouping —
/// each slot dropped, and each slot taking another slot's value — fifteen change
/// the answer and one does not (slot 3 taking slot 2's value also yields
/// `0x3f800001`). [`PARALLEL_OPERANDS`] leaves none of the sixteen undetected,
/// which is why both sets run rather than one replacing the other.
const GROUPING_SENSITIVE_OPERANDS: [u32; 4] = [
    0x3f40_0000, // 0.75
    0x3e80_0000, // 0.25
    0x3340_0000, // 3 * 2^-26, which is 0.375 ulp(1.0)
    0x3300_0000, // 2^-25,     which is 0.25  ulp(1.0)
];
/// Interface key of the serial sum's one input.
const INPUT_KEY: &str = "input";
/// Interface key of the serial sum's one output.
const OUTPUT_KEY: &str = "result";
/// Interface key of the contraction's first operand, `[M, K]`.
const CONTRACTION_ACTIVATIONS_KEY: &str = "activations";
/// Interface key of the contraction's second operand, `[N, K]`.
///
/// The key the whole ticket turns on: it is the first program input in this
/// workspace that is not the *only* program input, so every place that resolved
/// a binding by comparing against one constant had to learn to resolve an
/// ordinal instead.
const CONTRACTION_WEIGHTS_KEY: &str = "weights";
/// Interface key of the contraction's one output, `[M, N]`.
const CONTRACTION_OUTPUT_KEY: &str = "projected";
/// Class name of the published contraction member.
///
/// `prototypes/serial-sum-compile` writes this name. Nothing links the two
/// crates, so each pins it in a test naming the other side, exactly as
/// [`SIDECAR_SUFFIX`] and [`REDUCTION_CLASSES`] are pinned.
const CONTRACTION_CLASS: &str = "contraction";
/// Class name of the published L3 correctness cell, `w_decode_kv`.
///
/// `prototypes/serial-sum-compile` writes this name, and both halves pin it in
/// the same idiom [`CONTRACTION_CLASS`] is under.
const L3_CELL_CLASS: &str = "contraction-w-decode-kv";
/// SHA-256 of the `direct` realization's result bytes for `w_decode_kv`,
/// retained by the L3 realization probe.
///
/// **This is a measurement, not a constant this workspace derived.** It was
/// recorded on an Apple M4 Max under macOS 27.0 `26A5388g`, Xcode 26.6
/// `17F113`, SDK 26.5 `25F70`, and the offline Metal compiler
/// `32023.883`, by `spikes/scheduling/metal_contraction_vertical`, and it lives
/// in that spike's
/// `results/2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/workload.tsv`.
/// A run on any other host row is a different claim: this binary states the row
/// it compared against and a reader who is not on it must treat the comparison
/// as unmade rather than as evidence.
///
/// The digest domain is the probe's own — little-endian `f32` bytes in row-major
/// order, exactly the buffer the probe's host handed to `CC_SHA256` — so
/// [`result_digest`] reproduces it from readback bit patterns without an
/// intervening shape or dtype.
const L3_CELL_RESULT_SHA256: &str =
    "79810ce471cbd6cd05e5c0c30ea6023e74b997bd5b349212b71cd4a23fe8701f";
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

/// Reads one case's operand payloads, one per input the artifact declares.
///
/// **Every payload, and each at its own declared element count.** The superseded
/// spelling was `case.inputs().next()`, which read the leading payload and
/// ignored the rest; against a one-input artifact that is the whole set and
/// against a two-input one it is half of it, silently. The sidecar layer already
/// guarantees one payload per declared input — it refuses a case that supplies
/// any other number — so a count disagreement here is this reader's own defect
/// and is reported as one.
fn case_operands(
    interface: &DeclaredInterface,
    case: ProofCaseRef<'_>,
) -> Result<Vec<Vec<u32>>, ProofError> {
    let payloads: Vec<_> = case.inputs().collect();
    if payloads.len() != interface.inputs.len() {
        return Err(ProofError::SidecarInterfaceArity {
            sidecar: payloads.len(),
            artifact: interface.inputs.len(),
        });
    }
    payloads
        .iter()
        .zip(&interface.inputs)
        .map(|(payload, declared)| {
            // Bound to the key as well as to the position: the sidecar places
            // its payloads into the artifact's interface order, so a
            // disagreement here means the two orders have drifted apart and
            // every operand after it would be written into the wrong buffer.
            if payload.key().as_str() != declared.key {
                return Err(ProofError::SidecarInterfaceKey {
                    sidecar: payload.key().as_str().to_owned(),
                    artifact: declared.key.clone(),
                });
            }
            decode_f32_bits("input", declared.elements, payload.bytes())
        })
        .collect()
}

/// Reads one case's expected output payload at the declared element count.
fn case_expected(
    interface: &DeclaredInterface,
    case: ProofCaseRef<'_>,
) -> Result<Vec<u32>, ProofError> {
    let payload = case
        .expected()
        .next()
        .ok_or(ProofError::SidecarWithoutCases)?;
    decode_f32_bits("expected", interface.output_elements, payload.bytes())
}

/// The authoritative macOS Metal declaration both paths compile and emit under.
///
/// Stated by `tiler-build` rather than here. This runner used to hold its own
/// `MetalTargetFacts` — an MSL 3.1 / macOS 14.0 record with per-dtype subnormal
/// behaviour and a buffer capacity — and its direct path separately spelled an
/// `air64-apple-macos14.0` compilation target. Two hand-written copies of a
/// target, in one process, with nothing comparing them; the migration replaces
/// both with the one declaration whose rows have named authorities.
fn declaration() -> Result<BoundMetalCompileDeclaration, ProofError> {
    BoundMetalCompileDeclaration::first_macos_apple9().map_err(ProofError::Declaration)
}

/// Compiles one program against the authoritative declaration's profile.
///
/// `compile_governed` is deliberately not used any more: it selects the
/// compiler's governed prototype profile, and the artifacts this runner routes
/// are published against the authoritative one, so a compile through the old
/// path would name a kernel program the packaged variant does not.
fn compile_under(
    declaration: &BoundMetalCompileDeclaration,
    program: &SemanticProgram,
) -> Result<Compilation, ProofError> {
    let targets =
        TargetRequest::new([declaration.profile().clone()]).map_err(ProofError::TargetRequest)?;
    let batch = compile(CompilerRequest::new(
        program,
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
        targets,
    ))
    .map_err(ProofError::Compile)?;
    batch
        .into_targets()
        .pop()
        .ok_or(ProofError::NoSelection)?
        .into_parts()
        .1
        .map_err(|_| ProofError::UnrealizableNumerics)
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

/// The L3 profile's index structure, `td,od->to`.
///
/// Spelled with the same arbitrary frontend index labels the producer uses, so
/// the two processes reach the same canonical encoding through the
/// renaming-invariant rule ADR 0087 requires rather than by both happening to
/// write the canonical labels.
fn contraction_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [
            [ContractionIndex::new(19), ContractionIndex::new(3)],
            [ContractionIndex::new(14), ContractionIndex::new(3)],
        ],
        [ContractionIndex::new(19), ContractionIndex::new(14)],
    )
    .expect("the profile's index structure passes every structural admission rule")
}

/// Builds `activations[m, k] x weights[n, k] -> projected[m, n]`.
///
/// Reconstructed here for the same reason the serial sum is: the envelope
/// carries no semantic program, so naming the computation the artifact packages
/// requires deriving it independently and comparing canonical identities.
fn contraction_program(m: u64, n: u64, k: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let activations = builder
        .input::<F32>(
            InputKey::new(CONTRACTION_ACTIVATIONS_KEY).expect("the activations key is valid"),
            Shape::from_dims([m, k]),
        )
        .expect("the activations operand binds");
    let weights = builder
        .input::<F32>(
            InputKey::new(CONTRACTION_WEIGHTS_KEY).expect("the weights key is valid"),
            Shape::from_dims([n, k]),
        )
        .expect("the weights operand binds");
    let projected =
        F32TensorContraction::apply(&mut builder, &contraction_structure(), activations, weights)
            .expect("the contraction applies");
    builder
        .output(
            OutputKey::new(CONTRACTION_OUTPUT_KEY).expect("the output key is valid"),
            projected,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Builds the reference tensor one operand set states.
///
/// Bit patterns throughout, in the byte order the sidecar reader uses, so a
/// signed zero, a subnormal, and a non-canonical NaN reach the oracle unchanged.
fn operand_tensor(bits: &[u32], rows: u64, columns: u64) -> Tensor {
    Tensor::dense(
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
    .expect("the input tensor is well formed")
}

/// Reads a dense reference tensor back as `f32` bit patterns.
fn dense_bits(tensor: &Tensor) -> Vec<u32> {
    match tensor.payload() {
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

/// Evaluates the same semantic program through the independent oracle.
fn reference_bits(program: &SemanticProgram, bits: &[u32], rows: u64, columns: u64) -> Vec<u32> {
    let key = InputKey::new(INPUT_KEY).expect("the input key is valid");
    let tensor = operand_tensor(bits, rows, columns);
    let outputs = ReferenceEvaluator::standard()
        .expect("the governed reference profile composes")
        .evaluate(program, &[InputBinding::new(&key, &tensor)])
        .expect("the reference evaluates the program");
    dense_bits(&outputs[0])
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
/// The reduction classes the producer publishes, as `(name, reduced extent)`.
///
/// Mirrors `prototypes/serial-sum-compile`'s own array. Nothing links the two
/// crates, so each states the matrix independently and both pin it in a test
/// naming the other side — the same arrangement [`SIDECAR_SUFFIX`] is under, and
/// for the same reason: a producer that writes one set of names while the runner
/// opens another leaves a green gate over a slice that cannot run.
///
/// The empty domain leads, because it is the boundary the other two cannot
/// speak for: a reduction over zero contributors reads its input buffer never,
/// and its result is a reduction's identity element rather than a sum.
const REDUCTION_CLASSES: [(&str, u64); 3] = [
    ("empty-domain", 0),
    ("singleton", 1),
    ("nontrivial", COLUMNS),
];

/// The plan roles the producer publishes for each reduction class.
const PLAN_ROLES: [&str; 2] = ["selected", "materialized"];

/// How many entries and shared allocations a member of each role must show.
///
/// This is the proof's central observable, not a formality. `selected` is the
/// fused plan: one dispatch, no intermediate. `materialized` computes the same
/// function as two dispatches through one shared allocation. Asserting the
/// counts is what separates "both agreed" from "both ran the same program
/// twice", and the latter would agree trivially.
fn expected_shape(role: &str) -> (usize, usize) {
    if role == "selected" { (1, 0) } else { (2, 1) }
}

/// Returns the envelope path for one published member of the proof matrix.
///
/// Derived exactly as the producer derives it, from the base path this run was
/// given.
fn proof_member(base: &Path, class: &str, role: &str) -> PathBuf {
    let mut name = base.as_os_str().to_owned();
    name.push(format!(".{class}.{role}"));
    PathBuf::from(name)
}

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

/// One input an artifact declares, read rather than assumed.
///
/// The extents are kept alongside the element count because the two answer
/// different questions: a buffer is sized from the count, and a *family* is
/// recognized from the shape — `[M, K]` and `[N, K]` are the same count at
/// `M = N` and are still two different operands.
#[derive(Clone, Debug)]
struct DeclaredInput {
    key: String,
    extents: Vec<u64>,
    elements: u64,
}

/// The whole interface one artifact declares.
///
/// **Ordered as the artifact orders it, and that order is load-bearing.** The
/// sidecar's case payloads, the routed `BindingTarget::ProgramInput` keys, and
/// this vector are three views of one interface; the ordinal a binding resolves
/// to here is the ordinal its operands are read from, so a host that sorted or
/// deduplicated this list would bind the right number of buffers with the wrong
/// bytes in them — a wrong answer rather than a refusal.
#[derive(Clone, Debug)]
struct DeclaredInterface {
    inputs: Vec<DeclaredInput>,
    output_key: String,
    output_elements: u64,
    abi: AbiFacts,
}

/// Reads the interface an artifact declares, and binds every declared shape.
///
/// The envelope carries no semantic program — the oracle's input — so the runner
/// reconstructs one to compare against. What it takes from the artifact is the
/// interface: the keys, the logical resolved types, and the exact operand
/// shapes. What it supplies is the body, and a disagreement there cannot be
/// checked here; it would surface as a bit disagreement, which is why the direct
/// path exists.
///
/// **The declared shape is read rather than asserted equal to [`COLUMNS`], and
/// that is the design rather than a gap.** What this runner may take from an
/// artifact is what the artifact says; asserting a shape here would replace the
/// artifact's declaration with this build's expectation, and the two paths would
/// then agree because they were told to rather than because one packaged what
/// the other runs.
///
/// The reduced extents agree today and the row counts do not: the producer
/// publishes one row and the direct path reduces [`ROWS`]. The
/// extents did not agree until
/// `bound-the-backend-entry-key-by-the-identity-it-carries`, because the
/// artifact layer bounded a `BackendEntryKey` at 1,024 bytes while a
/// two-or-more-contributor serial sum's kernel identity measures 1,121, so the
/// producer could package only the degenerate single-contributor reduction and
/// this path ran a `4x1` against the direct path's `4x3`. Nothing here changed
/// when that closed, which is what reading rather than asserting bought.
///
/// **No input count is expected here**, deliberately. This function reads what
/// the artifact declares and refuses only what it cannot represent; which
/// cardinality a given program family requires is
/// [`require_serial_sum_interface`]'s and [`require_contraction_interface`]'s to
/// state.
fn bind_declared_interface(decoded: &DecodedProgram) -> Result<DeclaredInterface, ProofError> {
    let f32_type = F32::resolved_type().canonical_encoding();
    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    let mut inputs = Vec::with_capacity(decoded.inputs().len());
    // **Every declared input, in the artifact's own interface order.** This loop
    // is the widening: it used to be `let [input] = inputs.as_slice()`, which
    // refused a two-operand program at the interface before any of the rest of
    // this file could be wrong about it, and which bound exactly one shape into
    // the ABI facts. A contraction declares two, and both of their extents are
    // ABI inputs — the result's own shape depends on one axis of each — so
    // binding only the leading operand would leave the launch geometry resolved
    // against a fact the artifact never supplied.
    for (position, input) in decoded.inputs().enumerate() {
        if input.resolved_type_encoding() != f32_type.as_bytes() {
            return Err(ProofError::Interface(format!(
                "the artifact's input {position} {:?} has logical type {:02x?} and this proof \
                 binds canonical F32 only",
                input.key().as_str(),
                input.resolved_type_encoding(),
            )));
        }
        let extents: Vec<u64> = input
            .shape()
            .extents()
            .iter()
            .map(|extent| extent.get())
            .collect();
        binder
            .bind_input_shape(input.key(), input.shape())
            .map_err(|cause| {
                ProofError::Interface(format!(
                    "the declared shape of input {position} {:?} does not bind: {cause}",
                    input.key().as_str(),
                ))
            })?;
        inputs.push(DeclaredInput {
            key: input.key().as_str().to_owned(),
            elements: extents.iter().product(),
            extents,
        });
    }

    let outputs: Vec<_> = decoded.outputs().collect();
    let [output] = outputs.as_slice() else {
        return Err(ProofError::Interface(format!(
            "the artifact declares {} outputs and this proof reads back exactly 1",
            outputs.len(),
        )));
    };
    if output.resolved_type_encoding() != f32_type.as_bytes() {
        return Err(ProofError::Interface(format!(
            "the artifact's output {:?} has logical type {:02x?} and this proof reads canonical \
             F32 only",
            output.key().as_str(),
            output.resolved_type_encoding(),
        )));
    }

    Ok(DeclaredInterface {
        inputs,
        output_key: output.key().as_str().to_owned(),
        output_elements: output
            .shape()
            .extents()
            .iter()
            .map(|extent| extent.get())
            .product(),
        abi: binder.build(),
    })
}

/// Requires the declared interface to be the serial sum's, and returns its extents.
///
/// Split from [`bind_declared_interface`] so the *reading* of an interface and
/// the *expectation* of one program family are two steps. Reading is what the
/// runner may take from an artifact; expecting is this build's own claim, and
/// keeping them apart is what let a second family be added without either one
/// growing a special case for the other.
fn require_serial_sum_interface(interface: &DeclaredInterface) -> Result<(u64, u64), ProofError> {
    let [input] = interface.inputs.as_slice() else {
        return Err(ProofError::Interface(format!(
            "the artifact declares {} input(s) and the serial sum declares 1",
            interface.inputs.len(),
        )));
    };
    let [rows, columns] = input.extents.as_slice() else {
        return Err(ProofError::Interface(format!(
            "the artifact's input has rank {} and the serial sum reduces a rank-2 input",
            input.extents.len(),
        )));
    };
    if input.key != INPUT_KEY || interface.output_key != OUTPUT_KEY {
        return Err(ProofError::Interface(format!(
            "the artifact's interface is {:?} -> {:?} and the serial sum's is \
             {INPUT_KEY:?} -> {OUTPUT_KEY:?}",
            input.key, interface.output_key,
        )));
    }
    if interface.output_elements != *rows {
        return Err(ProofError::Interface(format!(
            "the artifact publishes {} F32 element(s) and reducing a {rows}x{columns} input's \
             inner axis publishes {rows}",
            interface.output_elements,
        )));
    }
    Ok((*rows, *columns))
}

/// Requires the declared interface to be the contraction's, and returns `(M, N, K)`.
///
/// **The shared contracted extent is checked rather than taken from one
/// operand.** `td,od->to` requires `activations[M, K]` and `weights[N, K]` to
/// agree on `K`, and an artifact whose two operands disagreed would describe a
/// program the structure's own extent-agreement rule refuses — so reading `K`
/// off the first operand and never looking at the second would turn a
/// malformed interface into a silently wrong buffer length.
fn require_contraction_interface(
    interface: &DeclaredInterface,
) -> Result<(u64, u64, u64), ProofError> {
    let [activations, weights] = interface.inputs.as_slice() else {
        return Err(ProofError::Interface(format!(
            "the artifact declares {} input(s) and the contraction declares 2",
            interface.inputs.len(),
        )));
    };
    if activations.key != CONTRACTION_ACTIVATIONS_KEY
        || weights.key != CONTRACTION_WEIGHTS_KEY
        || interface.output_key != CONTRACTION_OUTPUT_KEY
    {
        return Err(ProofError::Interface(format!(
            "the artifact's interface is ({:?}, {:?}) -> {:?} and the contraction's is \
             ({CONTRACTION_ACTIVATIONS_KEY:?}, {CONTRACTION_WEIGHTS_KEY:?}) -> \
             {CONTRACTION_OUTPUT_KEY:?}",
            activations.key, weights.key, interface.output_key,
        )));
    }
    let ([m, left_k], [n, right_k]) = (activations.extents.as_slice(), weights.extents.as_slice())
    else {
        return Err(ProofError::Interface(format!(
            "the contraction's operands have ranks {} and {}, and `td,od->to` reads two rank-2 \
             operands",
            activations.extents.len(),
            weights.extents.len(),
        )));
    };
    if left_k != right_k {
        return Err(ProofError::Interface(format!(
            "the artifact's operands contract over {left_k} and {right_k}, and `td,od->to` \
             shares one contracted extent",
        )));
    }
    let published = m
        .checked_mul(*n)
        .ok_or_else(|| ProofError::Interface(format!("a {m}x{n} result has no element count")))?;
    if interface.output_elements != published {
        return Err(ProofError::Interface(format!(
            "the artifact publishes {} F32 element(s) and a {m}x{n} contraction publishes \
             {published}",
            interface.output_elements,
        )));
    }
    Ok((*m, *n, *left_k))
}

/// Reads the shape the artifact declares and binds this build's serial-sum
/// expectation onto it.
///
/// Retained under its original name because its contract is unchanged for the
/// serial sum: the extents come from the artifact and never from [`ROWS`].
fn bind_interface(decoded: &DecodedProgram) -> Result<(u64, u64, AbiFacts), ProofError> {
    let interface = bind_declared_interface(decoded)?;
    let (rows, columns) = require_serial_sum_interface(&interface)?;
    Ok((rows, columns, interface.abi))
}

/// Compiles this build's alternatives for the shape *the artifact* declares.
///
/// **The one place a declared shape becomes a program, and the reason it is a
/// function rather than four lines inside [`prove_member`].** The historic
/// defect was those four lines compiling `serial_sum_program(ROWS, columns)` —
/// this crate's own row count against a producer that had moved to one row — so
/// every packaged program was foreign and the whole
/// matrix pass could prove nothing. Nothing in the repository could see it,
/// because the matrix runs only against real published members on hardware.
///
/// Routing it through one named function is what lets the gate run the same
/// code: `the_published_shape_matrix_survives_this_builds_shape_handling`
/// assembles an envelope at each shape the producer publishes and requires this
/// function's compilation to derive the packaged program, and requires the
/// substitution to be refused. Both halves need no device.
fn compile_for_declared_shape(
    declaration: &BoundMetalCompileDeclaration,
    decoded: &DecodedProgram,
) -> Result<(u64, u64, Compilation), ProofError> {
    let (rows, columns, _) = bind_interface(decoded)?;
    let compilation = compile_under(declaration, &serial_sum_program(rows, columns))?;
    Ok((rows, columns, compilation))
}

/// Requires a packaged kernel program to be one this build derived for that
/// declared shape.
///
/// The packaged program is matched against *some* alternative rather than
/// against the selected one: the producer legitimately packages a plan the
/// portfolio did not rank first, and demanding `selected` would refuse the
/// materialized member for being exactly what it is meant to be. The set is
/// still this build's own governed compilation of the shape the artifact
/// declares, so this is a narrower claim than "some program" by a wide margin.
fn require_derived_program(compilation: &Compilation, packaged: &[u8]) -> Result<(), ProofError> {
    if compilation.alternatives().any(|alternative| {
        alternative
            .abi()
            .kernel_program()
            .canonical_identity()
            .as_bytes()
            == packaged
    }) {
        return Ok(());
    }
    Err(ProofError::ForeignProgram {
        packaged: packaged.len(),
        alternatives: compilation.alternatives().count(),
    })
}

/// The environment the **diagnostic** envelope path routes under.
///
/// # This is producer-declared equality, NOT host-earned eligibility
///
/// Read that literally. The profile below is the one `tiler-build` *declares*
/// for this Metal target; nothing about this host earned the right to offer it.
/// [`ExecutionEnvironment::classify`] therefore answers a real question — does
/// this artifact name the profile the producer declared, under the same exact
/// descriptor — and does not answer the question ADR 0086 gates, which is
/// whether this machine is a host the profile is applicable to.
/// [`offer_the_declared_profile`] is where that second question is asked, and it
/// refuses.
///
/// The previous spelling took a `&Compilation` and rebuilt the descriptor from a
/// local compile. That was worse in a way worth recording: it looked like an
/// independent host statement while being a restatement of the producer's own
/// authority, so a reader could mistake a green route for evidence about the
/// machine. The declaration is the same authority stated once, out loud.
fn declared_route_environment(
    declaration: &BoundMetalCompileDeclaration,
) -> Result<ExecutionEnvironment, ProofError> {
    let profile = declaration
        .target_profile_ref()
        .map_err(|_| ProofError::HostProfile)?;
    Ok(ExecutionEnvironment {
        target_profile: TargetProfileRef {
            key: TargetProfileKey::new(profile.key.as_str())
                .map_err(|_| ProofError::HostProfile)?,
            descriptor: profile.descriptor,
        },
        backend: BackendKey::new(BACKEND_KEY).map_err(|_| ProofError::HostProfile)?,
        representation: RepresentationKey::new(REPRESENTATION_KEY)
            .map_err(|_| ProofError::HostProfile)?,
    })
}

/// Reads one `sw_vers` field, or nothing when the tool does not answer.
///
/// A tool that is missing, fails, or prints nothing leaves the predicate
/// *unobserved* rather than supplying a placeholder. The policy has a typed
/// refusal for an unanswered predicate, and inventing a value here would spend
/// that distinction to make an adapter bug look like a host fact.
fn sw_vers(field: &str) -> Option<String> {
    let output = Command::new("/usr/bin/sw_vers").arg(field).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8(output.stdout).ok()?;
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

/// Normalizes a Rust architecture name into the spelling the records use.
///
/// `std::env::consts::ARCH` reports `aarch64` for the machine every retained
/// record spells `arm64`. Exactly that one spelling is mapped; everything else
/// passes through unchanged, so an architecture nobody measured is refused by
/// its own name rather than renamed into the one the policy wants.
fn normalized_architecture(arch: &str) -> &str {
    if arch == "aarch64" { "arm64" } else { arch }
}

/// Every Apple `MTLGPUFamily` enumerator `metal` 0.33.0 names, ascending.
///
/// This is the *binding's* vocabulary and not Tiler's, and the two are joined by
/// Apple's own enumerator value rather than by a pair table. `MTLGPUFamily` is
/// `#[repr(i64)]` with each variant declared at the number `MTLDevice.h` gives
/// it, and [`AppleGpuFamilyConstant`] carries that same number transcribed from
/// the same header, so the correspondence is arithmetic that already exists
/// rather than a second table someone has to keep in step.
///
/// The list is hand-written because it has to be: the binding's enum is
/// `#[non_exhaustive]`, publishes no iteration, and offers no `TryFrom`, so
/// nothing here can enumerate it or convert into it. What keeps the list honest
/// is that it is not the *population* — the walk below is driven from
/// [`MetalGpuFamily::ALL`] — and that the assertion below rejects a build in
/// which it has fallen behind that vocabulary.
const BINDING_APPLE_FAMILIES: [MTLGPUFamily; 9] = [
    MTLGPUFamily::Apple1,
    MTLGPUFamily::Apple2,
    MTLGPUFamily::Apple3,
    MTLGPUFamily::Apple4,
    MTLGPUFamily::Apple5,
    MTLGPUFamily::Apple6,
    MTLGPUFamily::Apple7,
    MTLGPUFamily::Apple8,
    MTLGPUFamily::Apple9,
];

/// Names one governed Apple enumerator back into the type this binding's device
/// call takes.
///
/// The residual step [`observe_highest_gpu_family`] cannot take for a caller:
/// `tiler-metal` hands out the raw `NSInteger` because that is what crosses to
/// every binding, and `metal` 0.33.0 wants its own enum. `objc2-metal` takes the
/// raw value directly, which is why `prototypes/candle-metal-adapter` has no
/// function like this one.
///
/// `None` is an enumerator this binding cannot name. It is reachable rather than
/// theoretical: the macOS 26.5 SDK declares `MTLGPUFamilyApple10 = 1010` and this
/// binding stops at `Apple9`, so widening [`MetalGpuFamily`] reaches it. Both
/// callers turn it into a refusal, because answering `false` would report a
/// question nobody asked as a device that answered no.
const fn binding_apple_enumerator(constant: AppleGpuFamilyConstant) -> Option<MTLGPUFamily> {
    let mut index = 0;
    while index < BINDING_APPLE_FAMILIES.len() {
        let candidate = BINDING_APPLE_FAMILIES[index];
        if candidate as isize == constant.value() {
            return Some(candidate);
        }
        index += 1;
    }
    None
}

/// Compiles only while this binding can name every family the vocabulary probes.
///
/// The counted half is the literal, and the literal is the point: nothing else in
/// this file states how many families it expects to be able to ask about, so a
/// vocabulary that grew would otherwise reach the runtime refusal below on every
/// host with the tree green. Widening [`MetalGpuFamily`] is a build error here,
/// which is where whoever widens it learns that this runner needs a newer `metal`
/// binding before its applicability observation means anything again.
///
/// The sweep half is why bumping the literal is not the repair. It asks the same
/// question the probe asks at runtime, of the same population, and keeps failing
/// until the binding genuinely names the added family — so the two halves fail
/// for different reasons and a build that passes both has actually gained the
/// enumerator rather than been told to expect one more.
///
/// Neither half makes the runtime refusal redundant. An assertion is a claim
/// about this build and can be relaxed in one line; what the probe *answers* when
/// it cannot name an enumerator is the part that must stay fail-closed on its
/// own.
const _: () = {
    assert!(
        MetalGpuFamily::COUNT == 5,
        "this runner expects the governed vocabulary to name five Apple families; \
         `metal` 0.33.0 stops at Apple9, so a widened vocabulary needs a newer binding here \
         before the count is raised",
    );
    let mut index = 0;
    while index < MetalGpuFamily::ALL.len() {
        assert!(
            binding_apple_enumerator(MetalGpuFamily::ALL[index].apple_constant()).is_some(),
            "`metal` 0.33.0 cannot name an Apple enumerator MetalGpuFamily::ALL declares, so \
             this runner would leave the GPU-family predicate unobserved on every host",
        );
        index += 1;
    }
};

/// What this binding could learn about the Apple families a device supports.
///
/// Two outcomes rather than a [`MetalGpuFamilySupport`], because "the device
/// named no family this vocabulary knows" and "this binding could not ask" are
/// different facts with different repairs — the first is a host to change and the
/// second is a Metal binding to upgrade — and collapsing them is the defect this
/// type exists to prevent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProbedGpuFamily {
    /// The governed vocabulary's own answer, from a walk this binding completed.
    Answered(MetalGpuFamilySupport),
    /// [`MetalGpuFamily`] named an enumerator this binding cannot, so the device
    /// was never asked about it and there is no answer to report.
    Unnameable(AppleGpuFamilyConstant),
}

impl fmt::Display for ProbedGpuFamily {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Answered(MetalGpuFamilySupport::Highest(family)) => {
                formatter.write_str(family.as_str())
            }
            Self::Answered(MetalGpuFamilySupport::NoneNamed) => {
                formatter.write_str("no named Apple family")
            }
            Self::Unnameable(constant) => write!(
                formatter,
                "unobserved: the governed vocabulary names MTLGPUFamily {constant}, which this \
                 binding cannot name, so this device was never asked",
            ),
        }
    }
}

/// Asks this device about exactly the families the governed vocabulary names.
///
/// The population, the order, and the name of the answer are all `tiler-metal`'s:
/// this supplies the device call and the binding-specific step of turning a raw
/// enumerator into the type that call takes. **It used to pair each variant with
/// its Apple constant here, and that was the defect** — a pair table has no arm
/// that can be missing, so a family added to [`MetalGpuFamily`] compiled cleanly,
/// the device was never asked about it, and the applicability policy was then
/// refused against a lower family reported as though it were the most specific
/// true statement the device made.
///
/// One unnameable enumerator discards the whole walk rather than only its own
/// query, and that is not caution — it is what the answer means.
/// [`observe_highest_gpu_family`] walks highest first and stops at the first
/// supported family, so a family above the one that answered leaves
/// `Highest(lower)` an understatement wearing the shape of a most-specific claim.
fn probe_apple_families(device: &Device) -> ProbedGpuFamily {
    let mut unnameable = None;
    let observed =
        observe_highest_gpu_family(|constant| match binding_apple_enumerator(constant) {
            Some(enumerator) => device.supports_family(enumerator),
            None => {
                unnameable = Some(constant);
                // Not an answer, and nothing downstream treats it as one: the
                // caller discards this walk entirely. Returning `false` here is
                // only how this closure declines to end the walk on a family it
                // never asked about.
                false
            }
        });
    unnameable.map_or(
        ProbedGpuFamily::Answered(observed),
        ProbedGpuFamily::Unnameable,
    )
}

/// States the probed family on an observation, or deliberately states nothing.
///
/// Split from the device so the refusal it produces is exercised without one.
/// Leaving the predicate unset is the adapter saying it did not ask, and
/// `MetalHostApplicabilityRefusal::Unobserved { predicate: GpuFamily }` is the
/// typed outcome that already exists for exactly that. Calling
/// `observing_gpu_family` with anything at all would be the adapter claiming it
/// asked.
fn stating_probed_family(
    observation: MetalHostObservation,
    probed: ProbedGpuFamily,
) -> MetalHostObservation {
    match probed {
        ProbedGpuFamily::Answered(support) => observation.observing_gpu_family(support),
        ProbedGpuFamily::Unnameable(_) => observation,
    }
}

/// Observes the four host predicates that need no device.
///
/// Split from the device half so the composition is exercised without Metal, and
/// so the policy's own cases never need either. Nothing here reads an artifact,
/// a compilation, or a compiler identity: the whole point of the separate
/// applicability check is that it cannot be satisfied by the producer's
/// declaration.
fn observe_host_environment() -> MetalHostObservation {
    let mut observation = MetalHostObservation::unobserved()
        .observing_os_family(std::env::consts::OS)
        .observing_architecture(normalized_architecture(std::env::consts::ARCH));
    if let Some(version) = sw_vers("-productVersion") {
        observation = observation.observing_os_version(version);
    }
    if let Some(build) = sw_vers("-buildVersion") {
        observation = observation.observing_os_build(build);
    }
    observation
}

/// Observes every predicate the first Metal profile's applicability row names.
///
/// The device contributes exactly two of them — the name it reports for itself
/// and the Apple family it claims — and nothing else about the device reaches
/// the policy. In particular the registry ID does not: ADR 0086 excludes it by
/// name, because the retained records report two different values for this same
/// named Apple M4 Max.
///
/// The family is passed in rather than probed here so the caller can report the
/// exact enumerator a refusal is about; a probe hidden in here could only leave
/// the predicate unobserved without saying why.
fn observe_metal_host(device: &Device, probed: ProbedGpuFamily) -> MetalHostObservation {
    stating_probed_family(
        observe_host_environment().observing_device_name(device.name()),
        probed,
    )
}

/// The production offer path: earn the right to offer the declared profile, or
/// refuse.
///
/// This is the only route in this binary that *claims authority*. It observes
/// the host — and nothing else; no artifact, no compilation, no compiler
/// identity can reach it — and asks
/// [`evaluate_metal_host_applicability`] whether that observation satisfies the
/// measured applicability row. On every host observable today the answer is
/// [`MetalHostApplicabilityRefusal::UnknownNativeTranslationAuthority`], because
/// ADR 0086 decides that native device translation of a metallib during pipeline
/// creation is a typed capability fact whose authority is `Unknown`, and ADR
/// 0043's disposal of `Unknown` keeps an unknown candidate out of an executable
/// frontier.
///
/// So this returns the refusal rather than an environment, and the returned
/// value is used for exactly one thing: printing what was refused and why. It is
/// asked *before* any routing commit, because a refusal after a commit would be
/// a fallback ADR 0051 does not permit.
///
/// **It does not gate the diagnostic route below**, and that separation is Tom's
/// recorded resolution for this ticket rather than a convenience: the runtime
/// machinery — decode, route, ABI binding, two-stage qualification, dispatch —
/// is worth exercising on hardware, and the honest way to keep exercising it is
/// to state that the route runs on producer-declared equality and makes no
/// applicability claim. Gating on this refusal would stop the value proof from
/// running while proving nothing new about eligibility, because the refusal is
/// structural: [`tiler_metal::applicability::MetalHostEligibility`] holds an
/// uninhabited authority, so no host can produce a receipt.
fn offer_the_declared_profile(device: &Device) -> MetalHostApplicabilityRefusal {
    let probed = probe_apple_families(device);
    let observation = observe_metal_host(device, probed);
    println!(
        "host applicability observation: os {}/{}/{}, arch {}, device {}, family {probed}",
        observation.os_family().unwrap_or("unobserved"),
        observation.os_version().unwrap_or("unobserved"),
        observation.os_build().unwrap_or("unobserved"),
        observation.architecture().unwrap_or("unobserved"),
        observation.device_name().unwrap_or("unobserved"),
    );
    let policy = MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9;
    match evaluate_metal_host_applicability(policy, &observation) {
        // Unreachable at the type level inside `tiler-metal`, where the receipt
        // is visibly uninhabited. From here it is an opaque struct, so the arm
        // is required — and writing it costs nothing, because reaching it needs
        // a superseding decision under ADR 0086 rather than a code change.
        Ok(receipt) => panic!(
            "a host earned an eligibility receipt under {}, which is impossible without a \
             superseding ADR 0086 decision",
            receipt.policy().id(),
        ),
        Err(refusal) => refusal,
    }
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
    encode: impl FnOnce(&CommandBufferRef),
) -> Result<Vec<u32>, ProofError> {
    let queue = device.new_command_queue();
    let command_buffer = queue.new_command_buffer();
    encode(command_buffer);
    command_buffer.commit();
    command_buffer.wait_until_completed();

    // The only decision left after the commit, and the only path to a readback.
    match submission_outcome(command_buffer.status()) {
        SubmissionOutcome::Completed => Ok(crate::buffer::read_f32(output, count)
            .iter()
            .map(|value| value.to_bits())
            .collect()),
        SubmissionOutcome::ExecutionError => Err(ProofError::Dispatch {
            status: "Error",
            detail: "the device reported an execution error for this command buffer",
        }),
        SubmissionOutcome::NotTerminal(status) => Err(ProofError::Dispatch {
            status,
            detail: "the wait returned with the command buffer in a non-terminal state",
        }),
    }
}

/// What a command buffer's status permits after the wait.
///
/// **Three outcomes, and deliberately no fourth.** There is no retry and no
/// fallback variant, because the runtime execution contract's transition table
/// says "never" for every post-commit transition — in-flight to
/// validation-observed included. Stating that in the type is what keeps it from
/// being a rule a later edit can forget: there is nothing here to return that
/// would mean "try another route".
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SubmissionOutcome {
    /// The one status that permits a readback.
    Completed,
    /// The device reported a terminal execution error.
    ExecutionError,
    /// The wait returned and the buffer had not reached a terminal state.
    ///
    /// Carries the status name because which non-terminal state it stopped in
    /// is the whole diagnostic value: `NotEnqueued` means nothing was ever
    /// submitted, and `Scheduled` means the work was accepted and had not
    /// finished.
    NotTerminal(&'static str),
}

/// Classifies one command-buffer status into what it permits.
///
/// **Apple defines exactly two terminal states, `Completed` and `Error`**, and
/// the runtime execution contract records the consequence: `waitUntilCompleted`
/// returns no success value, so "a pre-wait non-error status is not evidence of
/// successful completion". A check written as `status != Completed` is correct
/// today and collapses that distinction — it reports a buffer that never left
/// the queue in the same breath as one the GPU rejected, which are different
/// things for a caller to do next.
///
/// Matched exhaustively and wildcard-free, so a status added to the binding is a
/// build error here rather than falling into whichever arm a catch-all named.
/// That is the same posture every other vocabulary match in this workspace
/// takes, and this is the one place a wrong answer would be read as arithmetic:
/// a readback taken from a buffer whose dispatch failed returns whatever the
/// output held before, which compares against the reference as a numerical
/// disagreement.
const fn submission_outcome(status: MTLCommandBufferStatus) -> SubmissionOutcome {
    match status {
        MTLCommandBufferStatus::Completed => SubmissionOutcome::Completed,
        MTLCommandBufferStatus::Error => SubmissionOutcome::ExecutionError,
        MTLCommandBufferStatus::NotEnqueued => SubmissionOutcome::NotTerminal("NotEnqueued"),
        MTLCommandBufferStatus::Enqueued => SubmissionOutcome::NotTerminal("Enqueued"),
        MTLCommandBufferStatus::Committed => SubmissionOutcome::NotTerminal("Committed"),
        MTLCommandBufferStatus::Scheduled => SubmissionOutcome::NotTerminal("Scheduled"),
    }
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
    submit(device, &output, count, |command_buffer| {
        let encoder = command_buffer.new_compute_command_encoder();
        encoder.set_compute_pipeline_state(&pipeline);
        encoder.set_buffer(0, Some(&input), 0);
        encoder.set_buffer(1, Some(&output), 0);
        encoder.dispatch_threads(MTLSize::new(ROWS, 1, 1), MTLSize::new(width, 1, 1));
        encoder.end_encoding();
    })
}

/// Which parallel reduction strategy one retained alternative realizes.
///
/// **Recognized by an observable each strategy alone has, not by a name.** The
/// compiler publishes a plan alternative's kernels and its ABI, never its
/// reduction topology, so asking "is this the tree?" has to be answered from
/// what the alternative *declares*. The multi-pass split is the only alternative
/// with three stages — pointwise, partial, and final. The single-workgroup tree
/// is the only one declaring an entry wider than one thread per workgroup: it
/// launches one invocation per participant inside one workgroup, where every
/// independent-invocation region declares a width of one. The serial fold
/// declares neither.
///
/// This mirrors `tiler-build`'s own
/// `a_flush_and_reassociate_contract_reaches_a_parallel_portfolio`, which
/// recognizes the same two strategies through the same two observables. It is
/// deliberately the same rule rather than a second one: that fixture proves the
/// portfolio *retains* them on this profile, and this binary proves they *run*,
/// so a divergence in what "the tree" means would make the two claims about
/// different things.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ParallelStrategy {
    /// Three stages: map, reduce each partition, combine the partials.
    MultiPassSplit,
    /// One workgroup whose participants reduce cooperatively through a tree.
    SingleWorkgroupTree,
}

impl ParallelStrategy {
    /// A stable lowercase identifier for this strategy.
    const fn as_str(self) -> &'static str {
        match self {
            Self::MultiPassSplit => "multi-pass-split",
            Self::SingleWorkgroupTree => "single-workgroup-tree",
        }
    }
}

impl fmt::Display for ParallelStrategy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Resolves one ABI arena position to the unsigned literal it must be.
///
/// **A position is not a width, and reading it as one is silent.** Every launch
/// quantity on the alternatives this profile retains is a declared literal, so a
/// node that is not one means the derivation moved and this reader stopped
/// measuring what it names. That is a refusal rather than a skip: a skipped
/// entry would leave a strategy unrecognized and the run would report proving
/// one strategy while believing it had proved two.
fn literal_extent(expressions: &[ExprNode], position: u32) -> Result<u64, ProofError> {
    let index = usize::try_from(position).expect("an arena position fits a usize");
    match expressions.get(index) {
        Some(ExprNode::Root(AbiRoot::UnsignedLiteral(value))) => Ok(*value),
        other => Err(ProofError::NonLiteralLaunch {
            position,
            node: format!("{other:?}"),
        }),
    }
}

/// Classifies one retained alternative by the two observables above.
///
/// Returns `None` for the serial fold, which declares neither three stages nor a
/// workgroup wider than one thread.
fn classify_strategy(
    alternative: PlanAlternative<'_>,
) -> Result<Option<ParallelStrategy>, ProofError> {
    let abi = alternative.abi();
    let expressions = abi.expressions();
    let mut widest = 0_u64;
    for entry in abi.entries() {
        widest = widest.max(literal_extent(expressions, entry.threads_per_workgroup())?);
    }
    if widest > 1 {
        return Ok(Some(ParallelStrategy::SingleWorkgroupTree));
    }
    if alternative.kernels().len() == 3 {
        return Ok(Some(ParallelStrategy::MultiPassSplit));
    }
    Ok(None)
}

/// What one dispatched alternative did, beyond the bits it produced.
///
/// Reported so the run's own output distinguishes the three strategies by
/// evidence rather than by the label this binary assigned them: a "tree" that
/// launched one thread per workgroup and reserved no threadgroup memory would be
/// a misclassification, and printing both quantities is what makes that visible
/// instead of plausible.
struct DispatchReport {
    bits: Vec<u32>,
    /// Widest workgroup any stage of this alternative declared.
    widest_workgroup: u64,
    /// Most threadgroup memory any of its compiled pipelines statically reserves.
    threadgroup_bytes: u64,
    /// How many command encoders the submission carried, in execution order.
    encoders: usize,
}

/// The device storage one alternative's dispatch binds, resolved before encoding.
///
/// Held as one value so every buffer outlives the submission that reads it.
/// A multi-pass split's intermediate is referenced by the stage that writes it
/// and by the stage that reads it, and dropping either view would leave the
/// encoder holding a binding to freed memory — the one failure in this file that
/// would be a wrong answer rather than a refusal.
struct AlternativeStorage {
    /// One buffer per program allocation, in the program's own allocation order.
    buffers: Vec<Buffer>,
    /// The buffer the named program output lands in.
    output: Buffer,
    /// How many `f32` elements to read back out of it.
    readback: usize,
}

/// Allocates every buffer one alternative's program needs, input already written.
///
/// **One buffer per *allocation*, never per binding.** Two stages of a split
/// address one intermediate, and the program states that by placing both values
/// in one allocation. A host allocating per binding would hand the consumer a
/// fresh buffer, the producer's partials would never reach it, and the reduction
/// would read uninitialised device memory — a wrong answer rather than a
/// refusal. `AllocationRef` compares by identity within its program, which is
/// what makes the lookup below exact rather than a length coincidence.
fn allocate_alternative(
    device: &Device,
    program: &VerifiedKernelProgram,
    bits: &[u32],
) -> Result<AlternativeStorage, ProofError> {
    let allocations: Vec<_> = program.allocations().collect();
    let mut buffers = Vec::with_capacity(allocations.len());
    for allocation in &allocations {
        // Host-visible only where the host actually reads or writes: a program
        // input it fills and a program output it reads back. A temporary is
        // private, which is what the split's intermediate is.
        let host_visible = allocation
            .values()
            .any(|value| matches!(value.role(), ValueRole::Input | ValueRole::Output));
        let options = if host_visible {
            MTLResourceOptions::StorageModeShared
        } else {
            MTLResourceOptions::StorageModePrivate
        };
        buffers.push(device.new_buffer(allocation.capacity_bytes().max(1), options));
    }

    let index_of = |target: &_| {
        allocations
            .iter()
            .position(|candidate| candidate == target)
            .expect("every value's allocation is one this program declares")
    };

    let mut output = None;
    let mut readback = 0_usize;
    let mut inputs = 0_usize;
    for value in program.values() {
        let slot = index_of(&value.allocation());
        match value.role() {
            ValueRole::Input => {
                // **One program input, and this path says so rather than
                // assuming it.** `bits` is a single operand slice, so a program
                // with two inputs would have the same operands written into
                // both — a plausible tensor computed from the wrong bytes, which
                // is the one failure class this file treats as worse than a
                // refusal. The direct path is the serial sum's alone and the
                // envelope path is where multi-operand programs are routed, so
                // the honest move is to refuse here rather than to widen a path
                // nothing multi-operand reaches; a caller that brings one must
                // widen this deliberately.
                inputs += 1;
                if inputs > 1 {
                    return Err(ProofError::DirectPathMultiInput { inputs });
                }
                let operands: Vec<f32> = bits.iter().map(|value| f32::from_bits(*value)).collect();
                crate::buffer::write_f32(&buffers[slot], &operands);
            }
            ValueRole::Output => {
                readback = usize::try_from(value.required_bytes() / F32_BYTES)
                    .expect("the proof's output element count fits a usize");
                output = Some(buffers[slot].clone());
            }
            ValueRole::Temporary => {}
        }
    }

    Ok(AlternativeStorage {
        buffers,
        output: output.ok_or(ProofError::NoProgramOutput)?,
        readback,
    })
}

/// Emits, compiles, and dispatches one retained alternative on this device.
///
/// **Every dispatch parameter is read from the compiler's own record**, and that
/// is what makes a multi-stage launch here evidence rather than a hand-written
/// guess: the argument-table index of each buffer comes from the emitter's
/// binding table, the byte window from the program's own view, and both launch
/// extents from the ABI arena. Nothing about the topology is assumed, which is
/// why one function dispatches the fold, the split, and the tree unchanged.
///
/// This is the *direct* path — local knowledge, no envelope — and it stays
/// labelled as such. It is evidence about the compiler and the emitter, never
/// about the delivery mechanism, and it is never a fallback for the envelope
/// path.
fn dispatch_alternative(
    device: &Device,
    declaration: &BoundMetalCompileDeclaration,
    alternative: PlanAlternative<'_>,
    bits: &[u32],
) -> Result<DispatchReport, ProofError> {
    let kernels: Vec<_> = alternative.kernels().iter().collect();
    let unit = emit_translation_unit(&kernels, declaration.metal_facts(), declaration.emission())
        .map_err(|_| ProofError::Emit)?;
    // Emission succeeds even when the target cannot honour the declared
    // contract, so conformance is asked explicitly rather than inferred.
    unit.require_declared_realization()
        .map_err(|_| ProofError::UnrealizableNumerics)?;
    let request = CompileRequest::new(
        unit.source(),
        declaration.aot_target(),
        OptimizationLevel::Default,
        declaration.numerical_realization(),
    );
    let compiled = Toolchain::system()
        .compile(&request)
        .map_err(|_| ProofError::Toolchain)?;

    let program = alternative.abi().kernel_program();
    let expressions = program.abi_expressions();
    let storage = allocate_alternative(device, program, bits)?;

    // Resolved before the submission, so the encode below looks nothing up and
    // has no failure of its own to report.
    let mut stages = Vec::new();
    let mut widest_workgroup = 0_u64;
    let mut threadgroup_bytes = 0_u64;
    for stage in program.execution_order() {
        let identity = stage.kernel().canonical_identity();
        let emitted = unit
            .entry_points()
            .iter()
            .find(|entry| entry.kernel_identity() == identity)
            .ok_or(ProofError::Emit)?;
        let pipeline = pipeline_for(device, &compiled.metallib, emitted.symbol())?;

        let launch = stage.launch();
        let grid_threads = literal_extent(expressions, launch.grid_threads)?;
        let threads_per_workgroup = literal_extent(expressions, launch.threads_per_workgroup)?;
        // `dispatch_threads` has no meaning at zero and inventing one thread
        // would run a body the plan did not ask for, so a zero-thread stage is
        // refused rather than encoded — the same rule `plan_route` applies on
        // the envelope path. No shape this function is called at produces one;
        // the guard is here so that stays a fact rather than an assumption.
        if grid_threads == 0 {
            return Err(ProofError::EmptyLaunch {
                entry: stages.len(),
                skipped: false,
            });
        }
        // The declared workgroup is compared against what *this* pipeline
        // admits, before anything is encoded. A tree declaring more
        // participants than the compiled function accepts is a refusal here
        // rather than a submission failure later.
        workgroup_fits(
            stages.len(),
            emitted.symbol(),
            threads_per_workgroup,
            pipeline.max_total_threads_per_threadgroup(),
        )
        .map_err(|refusal| ProofError::DevicePreflight(Box::new(refusal)))?;
        // Threadgroup memory the compiled function statically reserves, against
        // what this device admits per threadgroup. The tree is the first
        // strategy here that reserves any, and an unchecked overrun is a
        // pipeline the device would refuse at encode time.
        let reserved = pipeline.static_threadgroup_memory_length();
        local_memory_fits(
            stages.len(),
            emitted.symbol(),
            reserved,
            device.max_threadgroup_memory_length(),
        )
        .map_err(|refusal| ProofError::DevicePreflight(Box::new(refusal)))?;
        widest_workgroup = widest_workgroup.max(threads_per_workgroup);
        threadgroup_bytes = threadgroup_bytes.max(reserved);

        // The emitter states which argument-table index each buffer parameter
        // binds at, and a stage binds its buffers to its accesses positionally.
        let buffers = emitted.buffers();
        let mut placements = Vec::new();
        for (position, access) in stage.accesses().enumerate() {
            let binding = buffers.get(position).ok_or(ProofError::Emit)?;
            let view = access.view();
            let slot = program
                .allocations()
                .position(|candidate| candidate == view.value().allocation())
                .expect("every accessed value's allocation is one this program declares");
            placements.push((
                binding.index(),
                storage.buffers[slot].clone(),
                view.window().offset,
            ));
        }
        stages.push((pipeline, placements, grid_threads, threads_per_workgroup));
    }

    let encoders = stages.len();
    let bits = submit(
        device,
        &storage.output,
        storage.readback,
        |command_buffer| {
            // **One encoder per stage, and that is the ordering guarantee.**
            // Metal orders encoders within a command buffer unconditionally,
            // with an implicit barrier between them, so a combining stage never
            // overlaps the partial stage whose output it reads. Commands inside
            // one encoder carry no such order, which is why the split does not
            // share one.
            for (pipeline, placements, grid_threads, threads_per_workgroup) in &stages {
                let encoder = command_buffer.new_compute_command_encoder();
                encoder.set_compute_pipeline_state(pipeline);
                for (index, buffer, offset) in placements {
                    encoder.set_buffer(u64::from(*index), Some(buffer), *offset);
                }
                encoder.dispatch_threads(
                    MTLSize::new(*grid_threads, 1, 1),
                    MTLSize::new(*threads_per_workgroup, 1, 1),
                );
                encoder.end_encoding();
            }
        },
    )?;

    // Both `storage` and `stages` are still live here, and that is the retention
    // this function owes: `submit` waits for the command buffer's terminal state
    // before returning, so every buffer the encode bound is held across the whole
    // device lifetime of the work that reads it. They hold *clones* of one
    // `MTLBuffer` per allocation rather than separate allocations — a retain, not
    // a copy — so it is the pair outliving the submission that matters, not
    // either one alone.
    drop(stages);
    drop(storage);
    Ok(DispatchReport {
        bits,
        widest_workgroup,
        threadgroup_bytes,
        encoders,
    })
}

/// Executes both parallel reduction strategies and compares each to the oracle.
///
/// **This is the claim the compiling cooperative golden cannot make.** A golden
/// establishes that a cooperative kernel *compiles*; it says nothing about
/// whether the barrier synchronizes, whether the threadgroup allocation is
/// reachable, or whether the tree computes the declared sum. Compilation success
/// is not a capability fact, so each strategy is emitted, linked, dispatched,
/// and compared bit for bit against `tiler-reference`'s independent evaluation
/// of the same semantic program.
///
/// **Both strategies, and the serial fold beside them.** The portfolio is
/// required to retain all three: a run that found only the fold would mean the
/// contract or the profile stopped reaching the parallel strategies, and a run
/// that found the strategies but not the fold would mean they replaced rather
/// than joined it. Each is dispatched, so an agreement is three independent
/// realizations of one declared reduction arriving at the same bits.
///
/// The comparison is on exact bit patterns rather than an epsilon. The contract
/// permits ordered regrouping and the reference evaluates the *declared*
/// contributor order, so agreement here is a statement that the regrouping the
/// compiler chose is one the contract actually authorizes.
fn prove_parallel_strategies(
    device: &Device,
    declaration: &BoundMetalCompileDeclaration,
) -> Result<(), ProofError> {
    let program = serial_sum_program(PARALLEL_ROWS, PARALLEL_COLUMNS);
    let bits = PARALLEL_OPERANDS.to_vec();
    let reference = reference_bits(&program, &bits, PARALLEL_ROWS, PARALLEL_COLUMNS);

    // The composed contract, stated rather than defaulted. Every parallel
    // reduction regroups the declared contributor sequence, and Apple `f32`
    // arithmetic flushes subnormals in every math mode, so this is the one
    // contract under which a split or a tree is a legal implementation of this
    // program on this hardware.
    let targets =
        TargetRequest::new([declaration.profile().clone()]).map_err(ProofError::TargetRequest)?;
    let compilation = compile(CompilerRequest::new(
        &program,
        NumericalContract::FLUSH_AND_REASSOCIATE_F32,
        targets,
    ))
    .map_err(ProofError::Compile)?
    .into_targets()
    .pop()
    .ok_or(ProofError::NoSelection)?
    .into_parts()
    .1
    .map_err(|_| ProofError::UnrealizableNumerics)?;

    let retained = compilation.alternatives().len();
    println!(
        "parallel portfolio at {PARALLEL_ROWS}x{PARALLEL_COLUMNS} under a flush-and-reassociate \
         contract: {retained} alternative(s)",
    );

    let mut seen = Vec::new();
    let mut folds = 0_usize;
    for alternative in compilation.alternatives() {
        let strategy = classify_strategy(alternative)?;
        let report = dispatch_alternative(device, declaration, alternative, &bits)?;
        let label = if let Some(strategy) = strategy {
            seen.push(strategy);
            strategy.as_str()
        } else {
            folds += 1;
            "serial-fold"
        };
        println!(
            "  {label} ({}): {} encoder(s) in order, widest workgroup {}, {} byte(s) of \
             threadgroup memory reserved, {:08x?} against {reference:08x?}",
            alternative.stable_id(),
            report.encoders,
            report.widest_workgroup,
            report.threadgroup_bytes,
            report.bits,
        );
        if report.bits != reference {
            return Err(ProofError::Mismatch {
                path: "parallel",
                device: report.bits,
                reference,
            });
        }
    }

    for strategy in [
        ParallelStrategy::MultiPassSplit,
        ParallelStrategy::SingleWorkgroupTree,
    ] {
        if !seen.contains(&strategy) {
            return Err(ProofError::StrategyAbsent { strategy, retained });
        }
    }
    if folds == 0 {
        return Err(ProofError::SerialFoldReplaced { retained });
    }
    println!(
        "both parallel strategies and the serial fold agree bit for bit with the reference on {} \
         element(s)",
        reference.len(),
    );
    Ok(())
}

/// The blocked contributor partition one dispatched alternative declares.
///
/// **Read from the plan's own published launch geometry, never assumed.** The
/// two parallel strategies declare the *same* split — `governed_partition`'s
/// balanced exact split, blocked and contiguous — and each publishes it in a
/// different observable, which is why this reads a different quantity per
/// strategy rather than one field:
///
/// - The **tree** runs one participant per partition inside one workgroup, so
///   its declared `threads_per_workgroup` is the partition count. That is the
///   same observable [`classify_strategy`] recognizes it by, read here from the
///   kernel program's own stages rather than from the artifact-facing ABI view,
///   because these are the literals [`dispatch_alternative`] actually encodes.
/// - The **split** stages the partials in a tensor whose partition axis is
///   innermost, so its partial pass launches `output_elements * partitions`
///   threads where its final pass launches `output_elements`. The ratio is the
///   partition count and needs no row count from this build.
/// - The **serial fold** declares no split at all, and the degenerate partition
///   of one contributor each is exactly its left fold. Nothing is read for it,
///   and that is stated rather than hidden: what makes it non-circular is that
///   [`prove_grouping_sensitive_case`] cross-checks this partition's oracle
///   against `tiler-reference`'s evaluation of the whole semantic program,
///   which is the independent statement of the declared order.
///
/// A partition that does not cover the contributor sequence exactly once each
/// is refused rather than rounded: both strategies decline an inexact split
/// rather than padding one, so a ragged partition here means this reader stopped
/// measuring what it names.
fn declared_partition(
    alternative: PlanAlternative<'_>,
    strategy: Option<ParallelStrategy>,
    contributors: u64,
) -> Result<ContributorPartition, ProofError> {
    let program = alternative.abi().kernel_program();
    let expressions = program.abi_expressions();
    let mut stages = Vec::new();
    for stage in program.execution_order() {
        let launch = stage.launch();
        stages.push((
            literal_extent(expressions, launch.grid_threads)?,
            literal_extent(expressions, launch.threads_per_workgroup)?,
        ));
    }
    let partitions = match strategy {
        Some(ParallelStrategy::SingleWorkgroupTree) => stages
            .iter()
            .map(|(_, workgroup)| *workgroup)
            .max()
            .unwrap_or(1),
        Some(ParallelStrategy::MultiPassSplit) => {
            let [_pointwise, partial, combine] = stages.as_slice() else {
                return Err(ProofError::UndeclaredGrouping {
                    strategy: strategy_label(strategy).to_owned(),
                    detail: format!(
                        "a split declares three stages and this one declares {}",
                        stages.len()
                    ),
                });
            };
            if combine.0 == 0 {
                return Err(ProofError::UndeclaredGrouping {
                    strategy: strategy_label(strategy).to_owned(),
                    detail:
                        "the combining stage launches no thread, so it names no partition count"
                            .to_owned(),
                });
            }
            partial.0 / combine.0
        }
        None => contributors,
    };
    if partitions == 0 || !contributors.is_multiple_of(partitions) {
        return Err(ProofError::UndeclaredGrouping {
            strategy: strategy_label(strategy).to_owned(),
            detail: format!(
                "{partitions} partition(s) do not cover {contributors} contributor(s) exactly once \
                 each"
            ),
        });
    }
    let partition = ContributorPartition {
        partitions,
        contributors_per_partition: contributors / partitions,
    };
    if !partition.covers(contributors) {
        return Err(ProofError::UndeclaredGrouping {
            strategy: strategy_label(strategy).to_owned(),
            detail: format!("{partition:?} does not cover {contributors} contributor(s)"),
        });
    }
    Ok(partition)
}

/// Names one classified alternative for a message.
fn strategy_label(strategy: Option<ParallelStrategy>) -> &'static str {
    match strategy {
        Some(strategy) => strategy.as_str(),
        None => "serial-fold",
    }
}

/// Evaluates the reduction one declared grouping computes, through the
/// independent oracle.
///
/// `tiler_reference::strict_partitioned_sum` is the second exact oracle the
/// reference crate already owns for exactly this question, and its own
/// documentation states why it has to exist: "a contract that permits
/// reassociation admits a set of results, so no oracle can answer *the* value
/// for it; what a plan can be checked against is the one order it selected".
/// This proof reaches that oracle from a device rather than restating it.
///
/// It is applied to the *operands* rather than to the pointwise prologue's
/// output, and that is sound only while `x * 1.0 + 0.0` is bit-identity on this
/// operand set — which [`prove_grouping_sensitive_case`]'s calibration step
/// checks by requiring the degenerate partition's answer to equal the reference
/// evaluator's answer for the whole program, prologue included.
fn partitioned_reference(
    bits: &[u32],
    rows: u64,
    columns: u64,
    partition: ContributorPartition,
) -> Result<Vec<u32>, ProofError> {
    let tensor = operand_tensor(bits, rows, columns);
    let reduced = strict_partitioned_sum(
        &tensor,
        &[Axis::new(1)],
        partition.partitions,
        partition.contributors_per_partition,
    )
    .map_err(|cause| ProofError::UndeclaredGrouping {
        strategy: "reference".to_owned(),
        detail: format!("{partition:?} is not an evaluable split: {cause}"),
    })?;
    Ok(dense_bits(&reduced))
}

/// Every `f32` value an order-preserving regrouping of one contributor sequence
/// can produce.
///
/// This is the *permitted set* — the population a reassociating contract
/// authorizes — and it is deliberately not the acceptance criterion. Requiring
/// membership would accept a strategy that produced some other legal grouping
/// than the one it declared, which is precisely the failure a declared-grouping
/// oracle exists to catch. What it is used for is the refusal population: every
/// member that is not the declared grouping's answer is a wrong-but-in-range
/// answer the oracle must say no to, and a run that cannot name one has a check
/// that cannot fail.
///
/// **Membership is not asserted either, and the absence is deliberate.** A
/// [`ContributorPartition`] expresses only a blocked uniform split, and every
/// blocked split of a contributor sequence is an order-preserving regrouping of
/// it, so an assertion that the declared grouping's value lies in this set could
/// not fail for any partition [`declared_partition`] can return. It was written,
/// perturbed, and found unreachable — the calibration step below catches a wrong
/// oracle first and the empty-population refusal catches a truncated
/// enumeration first — so it was removed rather than kept as a check that cannot
/// say no.
///
/// Enumerated by splitting at every position and combining the two sides'
/// values, which is the same construction
/// [Numerical semantics](../../../docs/numerical-semantics.md)'s bounded
/// result-set oracle uses for three through six leaves; for four contributors it
/// yields the five full binary groupings that preserve leaf order.
fn ordered_associations(bits: &[u32]) -> Vec<u32> {
    let [single] = bits else {
        let mut values = Vec::new();
        for split in 1..bits.len() {
            for left in ordered_associations(&bits[..split]) {
                for right in ordered_associations(&bits[split..]) {
                    values.push((f32::from_bits(left) + f32::from_bits(right)).to_bits());
                }
            }
        }
        return values;
    };
    vec![*single]
}

/// The one comparison this case's oracle makes.
///
/// Named rather than written inline at each site, because "the check was watched
/// failing" is only true if the refusal below is produced by the *same*
/// expression that accepted the observed answer. Two spellings of equality would
/// leave the observed comparison unexercised by the refusal.
fn declared_grouping_admits(expected: &[u32], candidate: &[u32]) -> bool {
    expected == candidate
}

/// This case enumerates one row's contributor sequence, so it holds while the
/// parallel shape is one row and stops the build otherwise.
const _: () = assert!(
    PARALLEL_ROWS == 1,
    "the grouping-sensitive case enumerates one row's orderings; a wider shape needs the \
     enumeration to run per row before this constant moves",
);

/// Executes both parallel strategies on operands whose grouping changes the
/// answer, and holds each to the grouping it declared.
///
/// **This is the claim [`prove_parallel_strategies`] deliberately cannot make.**
/// That case runs operands every grouping of which is exact, so its serial-fold
/// oracle is valid for all three strategies — and its refusal population among
/// legal groupings is therefore *empty*: no answer a reassociating contract
/// permits would have failed it. What it proves is contributor-set correctness.
/// This case runs the same three alternatives on operands where the declared
/// regroupings genuinely disagree, so the oracle has to change shape.
///
/// # The oracle, and why a serial fold and a tolerance are both wrong here
///
/// A serial fold is wrong because disagreement with it is the *expected* outcome
/// for a legally regrouped strategy: it would refuse the split and the tree for
/// being right. A tolerance is wrong because
/// [Correctness and testing](../../../docs/correctness-and-testing.md) holds
/// that a difference is attributed to a named cause or it is a defect, and
/// "within a bound" admits every value in an interval — including values no
/// legal grouping produces, and including the *other* strategy's answer, so it
/// could not tell a strategy that grouped as it declared from one that did not.
///
/// What replaces both is the exact value the strategy's *own declared grouping*
/// produces, evaluated through the independent reference. The comparison stays
/// bit for bit; what moves is which order the oracle is asked about, and it is
/// read from the plan by [`declared_partition`] rather than assumed.
///
/// # Why this is not the device checked against its own opinion
///
/// The partition is a *declaration* — what the plan published it would do — and
/// the bits are what the device did, so the comparison is "this device grouped
/// the way this plan declared". The evaluation is `tiler-reference`'s, which
/// shares no code with the compiler's lowering, the emitter, or the kernel. And
/// the degenerate partition is cross-checked against the reference evaluator's
/// run of the whole semantic program, so the oracle is calibrated against the
/// declared order by an independent path before any strategy is judged by it.
fn prove_grouping_sensitive_case(
    device: &Device,
    declaration: &BoundMetalCompileDeclaration,
) -> Result<(), ProofError> {
    let program = serial_sum_program(PARALLEL_ROWS, PARALLEL_COLUMNS);
    let bits = GROUPING_SENSITIVE_OPERANDS.to_vec();
    let contributors = PARALLEL_COLUMNS;

    // The calibration step, before any strategy is judged. The degenerate
    // partition — one contributor each, combined in ascending order — *is* the
    // declared serial order, so the partitioned oracle at it must agree with the
    // reference evaluator's run of the whole program. Agreement establishes two
    // things at once: that this file is asking the oracle about the order the
    // program declares, and that the pointwise prologue `x * 1.0 + 0.0` is
    // bit-identity on these operands, which is what lets every partition below
    // be evaluated over the operands rather than over the prologue's output.
    let serial_order = ContributorPartition {
        partitions: contributors,
        contributors_per_partition: 1,
    };
    let reference = reference_bits(&program, &bits, PARALLEL_ROWS, PARALLEL_COLUMNS);
    let serial_expected =
        partitioned_reference(&bits, PARALLEL_ROWS, PARALLEL_COLUMNS, serial_order)?;
    if !declared_grouping_admits(&reference, &serial_expected) {
        return Err(ProofError::GroupingOracleUncalibrated {
            evaluator: reference,
            partitioned: serial_expected,
        });
    }

    let permitted = ordered_associations(&bits);
    let mut distinct = permitted.clone();
    distinct.sort_unstable();
    distinct.dedup();
    println!(
        "  operands {:08x?}: {} order-preserving grouping(s) over {contributors} contributor(s) \
         producing {} distinct value(s) {:08x?}; the declared serial order is {:08x?}",
        bits,
        permitted.len(),
        distinct.len(),
        distinct,
        serial_expected,
    );

    let targets =
        TargetRequest::new([declaration.profile().clone()]).map_err(ProofError::TargetRequest)?;
    let compilation = compile(CompilerRequest::new(
        &program,
        NumericalContract::FLUSH_AND_REASSOCIATE_F32,
        targets,
    ))
    .map_err(ProofError::Compile)?
    .into_targets()
    .pop()
    .ok_or(ProofError::NoSelection)?
    .into_parts()
    .1
    .map_err(|_| ProofError::UnrealizableNumerics)?;

    let retained = compilation.alternatives().len();
    let mut seen = Vec::new();
    let mut folds = 0_usize;
    let mut refusals = 0_usize;
    for alternative in compilation.alternatives() {
        let strategy = classify_strategy(alternative)?;
        let label = strategy_label(strategy);
        if let Some(strategy) = strategy {
            seen.push(strategy);
        } else {
            folds += 1;
        }
        let partition = declared_partition(alternative, strategy, contributors)?;
        let expected = partitioned_reference(&bits, PARALLEL_ROWS, PARALLEL_COLUMNS, partition)?;

        // The refusal population, built by asking the oracle about every value
        // this contract permits and keeping the ones it says no to. The ask is
        // the refusal — there is no second pass re-checking the same predicate
        // on the same values, because that pass could not fail. Empty means the
        // oracle had nothing legal to refuse on these operands, which is the
        // exact condition `PARALLEL_OPERANDS` is in and the reason this case
        // exists.
        let mut foreign = Vec::new();
        for value in &distinct {
            if !declared_grouping_admits(&expected, std::slice::from_ref(value)) {
                foreign.push(*value);
            }
        }
        if foreign.is_empty() {
            return Err(ProofError::NoRefusableGrouping {
                strategy: label.to_owned(),
                permitted: distinct.clone(),
            });
        }

        let report = dispatch_alternative(device, declaration, alternative, &bits)?;
        println!(
            "  {label} ({}): declared {} partition(s) of {} contributor(s), {} encoder(s), widest \
             workgroup {}, {} byte(s) of threadgroup memory, {:08x?} against its declared \
             grouping's {:08x?} — {} from the serial fold's {:08x?}",
            alternative.stable_id(),
            partition.partitions,
            partition.contributors_per_partition,
            report.encoders,
            report.widest_workgroup,
            report.threadgroup_bytes,
            report.bits,
            expected,
            if declared_grouping_admits(&serial_expected, &expected) {
                "indistinguishable"
            } else {
                "one legal regrouping away"
            },
            serial_expected,
        );
        if !declared_grouping_admits(&expected, &report.bits) {
            return Err(ProofError::GroupingMismatch {
                strategy: label.to_owned(),
                partitions: partition.partitions,
                contributors_per_partition: partition.contributors_per_partition,
                device: report.bits,
                expected,
            });
        }

        // Each refused value is a *legal* answer under this contract, so what
        // the count records is the oracle saying no to a wrong-but-permitted
        // result — by the same function that just admitted the device's bits.
        refusals += foreign.len();
        println!(
            "    refused {} legal grouping(s) this strategy did not declare: {:08x?}",
            foreign.len(),
            foreign,
        );
    }

    for strategy in [
        ParallelStrategy::MultiPassSplit,
        ParallelStrategy::SingleWorkgroupTree,
    ] {
        if !seen.contains(&strategy) {
            return Err(ProofError::StrategyAbsent { strategy, retained });
        }
    }
    if folds == 0 {
        return Err(ProofError::SerialFoldReplaced { retained });
    }
    println!(
        "every alternative matched its own declared grouping bit for bit, and {refusals} \
         wrong-but-permitted grouping(s) were refused across {retained} alternative(s)",
    );
    Ok(())
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
    /// The identity whatever named this artifact recorded, stated as such.
    expected: &'a RecordedArtifactProgramIdentity,
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
    let mut decoded =
        DecodedProgram::decode(subject.bytes, SOLE_DELIVERY).map_err(ProofError::ProbeBaseline)?;
    let preflight = decoded
        .prepare(subject.environment, subject.expected, subject.abi)
        .and_then(|qualification| {
            // This subject declares no live-device requirement, so the stage is
            // passed through. The resolver is still supplied rather than
            // skipped, because the stage is not skippable — which is what keeps
            // a route that *does* declare one from reaching a commit unchecked.
            qualification.resolve_live_device_requirements(|_| LiveDeviceObservation::Unrecognized)
        })
        .and_then(|preparation| preparation.resolve_target_properties(|_| u64::MAX))
        .map_err(ProofError::ProbeBaseline)?;
    let entries = preflight.entries();
    let threads: u64 = entries
        .iter()
        .map(|entry| entry.launch().grid_threads())
        .sum();
    let bindings: usize = entries.iter().map(|entry| entry.bindings().len()).sum();
    Ok(format!(
        "the unperturbed subject routes: {} entr(y/ies), {threads} thread(s) over {bindings} \
         binding(s), {} shared allocation(s)",
        entries.len(),
        preflight.shared_allocations().len(),
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
    let decoded =
        DecodedProgram::decode(subject.bytes, SOLE_DELIVERY).map_err(ProofError::ProbeBaseline)?;
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
    match DecodedProgram::decode(&damaged, SOLE_DELIVERY) {
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
    match DecodedProgram::decode(&damaged, SOLE_DELIVERY) {
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
    match DecodedProgram::decode(&subject.bytes[..midpoint], SOLE_DELIVERY) {
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
///
/// The perturbation is in the *trailing* byte deliberately. A recorded identity
/// is domain-checked when it is stated, so flipping a leading byte would be
/// refused at the assertion boundary and never reach the loader — a different
/// refusal, and not the one this probe is about.
fn probe_foreign_expected_identity(subject: &ProbeSubject<'_>) -> Result<String, ProofError> {
    let mut decoded =
        DecodedProgram::decode(subject.bytes, SOLE_DELIVERY).map_err(ProofError::ProbeBaseline)?;
    let mut bytes = subject.expected.as_bytes().to_vec();
    if let Some(last) = bytes.last_mut() {
        *last ^= 0x01;
    }
    let foreign = RecordedArtifactProgramIdentity::from_bytes(&bytes)
        .map_err(ProofError::RecordedIdentity)?;
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

/// Returns the sole exclusion of a one-variant artifact no eligible variant survived.
///
/// The three probes below all perturb what the *host* states, and a host-relative
/// exclusion is now a filter applied before any guard is evaluated rather than a
/// terminal mismatch — the ordering
/// `select-executable-variants-across-registered-backend-families` inverted, so
/// that an artifact packaging plans for two backend families cannot have its
/// first plan refuse on behalf of a host the second one fits. What each probe
/// pins therefore moved from the rejection's class onto the exclusion it carries,
/// and the class it pins in addition — that *every* packaged variant was filtered
/// — is what says the artifact offers this host nothing at all.
fn sole_exclusion<T>(
    name: &'static str,
    outcome: Result<T, LoadRejection>,
) -> Result<(VariantIneligibility, String), ProofError> {
    match outcome {
        Err(
            ref rejection @ LoadRejection::NoEligibleVariant {
                packaged,
                ref filtered,
            },
        ) if filtered.len() == packaged => match filtered.as_slice() {
            [only] => Ok((only.reason.clone(), rejection.to_string())),
            _ => Err(refused(
                name,
                format!("this artifact packages one variant, and {filtered:?} names another"),
            )),
        },
        Err(other) => Err(refused(name, other.to_string())),
        Ok(_) => Err(refused(name, "the route was accepted".to_owned())),
    }
}

/// A host offering another profile descriptor **filters** the only variant, on
/// the profile that variant was assessed against.
///
/// Both halves of the exclusion are pinned. Which declaration excluded it
/// separates a plan assessed for another profile from an object compiled for
/// one, and those are different repairs; the classification separates the same
/// target family under a descriptor this host does not offer from an artifact
/// built for another family entirely. Asserting only that something refused
/// would erase both distinctions at the moment a caller needs them.
fn probe_other_profile_descriptor(subject: &ProbeSubject<'_>) -> Result<String, ProofError> {
    let mut decoded =
        DecodedProgram::decode(subject.bytes, SOLE_DELIVERY).map_err(ProofError::ProbeBaseline)?;
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
    let name = "another profile descriptor";
    let (reason, rendered) = sole_exclusion(
        name,
        decoded.preflight(&other_host, subject.expected, subject.abi),
    )?;
    match reason {
        VariantIneligibility::AssessedProfile {
            classification: TargetCompatibility::DescriptorMismatch { .. },
        } => Ok(format!(
            "a host offering another profile descriptor: {rendered}"
        )),
        other => Err(refused(name, other.to_string())),
    }
}

/// A host offering another profile *key* filters the only variant too, and the
/// classification names the family rather than the descriptor.
///
/// The sibling of [`probe_other_profile_descriptor`], and separate from it
/// because the two are different repairs: a key mismatch means this artifact was
/// built for another target family and the consumer should look for a different
/// artifact, while a descriptor mismatch under the same key means this family
/// under a profile revision the host does not offer and the consumer should
/// rebuild. A probe covering only the descriptor would pass while a loader that
/// compared descriptors and ignored keys admitted every foreign family.
///
/// The perturbed key is a *valid* profile key that no artifact here declares, so
/// the refusal is the classification and not a key-validation failure.
fn probe_other_profile_key(subject: &ProbeSubject<'_>) -> Result<String, ProofError> {
    let mut decoded =
        DecodedProgram::decode(subject.bytes, SOLE_DELIVERY).map_err(ProofError::ProbeBaseline)?;
    let other_host = ExecutionEnvironment {
        target_profile: TargetProfileRef {
            key: TargetProfileKey::new("tiler.metal.some-other-target-family.v1")
                .map_err(|_| ProofError::HostProfile)?,
            descriptor: subject.environment.target_profile.descriptor.clone(),
        },
        backend: subject.environment.backend.clone(),
        representation: subject.environment.representation.clone(),
    };
    let name = "another profile key";
    let (reason, rendered) = sole_exclusion(
        name,
        decoded.preflight(&other_host, subject.expected, subject.abi),
    )?;
    match reason {
        VariantIneligibility::AssessedProfile {
            classification: TargetCompatibility::ProfileKeyMismatch { .. },
        } => Ok(format!("a host offering another profile key: {rendered}")),
        other => Err(refused(name, other.to_string())),
    }
}

/// A host stating another backend family filters the variant on the
/// **representation** it cannot execute.
///
/// Excluded on that ground rather than on the target profile it happens to
/// share, which is why this probe changes only the backend key: the host still
/// offers the exact profile the variant was assessed against, so the exclusion
/// cannot come from the compatibility classification. The entry position is
/// pinned as well, because a multi-entry route realized by two payloads must say
/// which of them this host is not.
fn probe_other_backend_family(subject: &ProbeSubject<'_>) -> Result<String, ProofError> {
    let mut decoded =
        DecodedProgram::decode(subject.bytes, SOLE_DELIVERY).map_err(ProofError::ProbeBaseline)?;
    let other_backend = ExecutionEnvironment {
        target_profile: subject.environment.target_profile.clone(),
        backend: BackendKey::new("tiler.some-other-backend")
            .map_err(|_| ProofError::HostProfile)?,
        representation: subject.environment.representation.clone(),
    };
    let name = "another backend family";
    let (reason, rendered) = sole_exclusion(
        name,
        decoded.prepare(&other_backend, subject.expected, subject.abi),
    )?;
    match reason {
        VariantIneligibility::UnsupportedRepresentation { entry: 0, .. } => {
            Ok(format!("a host stating another backend family: {rendered}"))
        }
        other => Err(refused(name, other.to_string())),
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
        probe_other_profile_key,
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
    /// The buffer holding one program input the artifact names.
    ///
    /// **Carries the ordinal of that input in the artifact's declared interface,
    /// and that is the widening.** This used to be a bare `Input`, which was
    /// sufficient only while exactly one program input existed: a two-operand
    /// route binds two buffers, and a placement that could not say *which*
    /// operand a slot takes would fill both from the same host slice and return
    /// a plausible tensor computed from the wrong bytes.
    Input(usize),
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
    offset: u64,
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
fn plan_route(
    preflight: &Preflight<'_>,
    interface: &DeclaredInterface,
) -> Result<Vec<Vec<PlacedSlot>>, ProofError> {
    let mut plan = Vec::with_capacity(preflight.entries().len());
    for (position, routed) in preflight.entries().iter().enumerate() {
        let launch = routed.launch();
        // An entry covering no threads is legitimate rather than exceptional: a
        // reduction over an empty domain maps zero elements before reducing them
        // to its identity element, so its first stage has nothing to run and its
        // second still produces every output. The artifact *states* which of the
        // two an empty launch is, so the answer is read rather than assumed, and
        // a route that demands a zero-thread dispatch be encoded is refused —
        // `dispatch_threads` has no meaning at zero and inventing one thread
        // would run a body the plan did not ask for.
        if launch.grid_threads() == 0 && !launch.zero_work_skips_dispatch() {
            return Err(ProofError::EmptyLaunch {
                entry: position,
                skipped: false,
            });
        }

        let mut slots = Vec::with_capacity(routed.bindings().len());
        for binding in routed.bindings() {
            let placement = match binding.binding().target() {
                // Resolved against the artifact's *own* declared input order
                // rather than against a key constant this build holds. That is
                // what makes one function place a one-operand reduction and a
                // two-operand contraction without knowing which it is looking
                // at: a program input the artifact does not declare has no
                // ordinal, and falls through to the refusal below.
                BindingTarget::ProgramInput(key) => interface
                    .inputs
                    .iter()
                    .position(|declared| declared.key == key.as_str())
                    .map_or_else(
                        || {
                            Err(ProofError::UnboundBinding {
                                entry: position,
                                slot: binding.slot(),
                                target: format!(
                                    "ProgramInput({:?}), which this artifact does not declare",
                                    key.as_str(),
                                ),
                            })
                        },
                        |ordinal| Ok(Placement::Input(ordinal)),
                    )?,
                BindingTarget::ProgramOutput(keys)
                    if keys.len() == 1 && keys[0].as_str() == interface.output_key =>
                {
                    Placement::Output
                }
                BindingTarget::Internal => Placement::Internal,
                // Named rather than left to a wildcard, and the widening is why:
                // every `ProgramInput` is now resolved above, so the only target
                // that can still fall through is an output whose key or arity is
                // not this artifact's. A catch-all here would additionally
                // swallow a *new* `BindingTarget` variant as an ordinary
                // refusal, where the repository's posture is that a variant
                // added to the vocabulary must be a build error at every site
                // that decides on it.
                other @ BindingTarget::ProgramOutput(_) => {
                    return Err(ProofError::UnboundBinding {
                        entry: position,
                        slot: binding.slot(),
                        target: format!("{other:?}"),
                    });
                }
            };
            let offset = binding.accessible_offset();
            let needed = offset.checked_add(binding.accessible_bytes()).ok_or(
                ProofError::BindingRangeOverflow {
                    entry: position,
                    slot: binding.slot(),
                    offset,
                    extent: binding.accessible_bytes(),
                },
            )?;
            slots.push(PlacedSlot {
                transport: binding.transport_slot(),
                offset,
                needed,
                placement,
            });
        }
        plan.push(slots);
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
    LibraryRejected { entry: usize, detail: String },
    /// The library loaded and publishes no function by the entry symbol.
    FunctionAbsent {
        entry: usize,
        symbol: String,
        detail: String,
    },
    /// The device refused pipeline state for a function it did publish.
    PipelineRejected {
        entry: usize,
        symbol: String,
        detail: String,
    },
    /// The declared workgroup is larger than this pipeline admits.
    WorkgroupTooLarge {
        entry: usize,
        symbol: String,
        declared: u64,
        capacity: u64,
    },
    /// An entry reserves more threadgroup memory than this device admits.
    ///
    /// **The one derived requirement that had no reader.**
    /// `crates/tiler-artifact/src/program/requirement.rs` states that
    /// threadgroup memory is deliberately absent from the neutral
    /// `RouteResourceDimension` vocabulary because the requirement side is
    /// already stated — by `ResourceRequirements::local_memory_bytes` — and is
    /// "checked directly against the device by an adapter", naming this
    /// prototype as the adapter that does it. Until this variant existed the
    /// document described a check nothing performed: no code in the workspace
    /// read `local_memory_bytes` off a routed entry, and none read
    /// `maxThreadgroupMemoryLength` at all. A cooperative reduction is the first
    /// strategy here that reserves any, so the gap became reachable at the same
    /// moment a plan that uses it did.
    ThreadgroupMemoryExceeded {
        entry: usize,
        symbol: String,
        declared: u64,
        capacity: u64,
    },
    /// A binding must reach more bytes than one buffer can hold here.
    BindingExceedsBufferLimit {
        entry: usize,
        slot: usize,
        needed: u64,
        limit: u64,
    },
    /// An allocation came back shorter than the route requires.
    UndersizedAllocation {
        entry: usize,
        slot: usize,
        needed: u64,
        held: u64,
    },
    /// No entry of the route binds the program output this proof compares.
    ///
    /// Systemic rather than a route miss: `plan_route` already refused every
    /// binding target this proof does not place, so a route that reaches here
    /// declares an interface the proof cannot observe at all.
    NoOutputBinding,
    /// A routed slot takes a program input the caller supplied no operands for.
    ///
    /// Systemic, and an assertion against this binary's own composition rather
    /// than against the artifact: the ordinal was resolved from the same
    /// declared interface the operand set is built from, so reaching this means
    /// a caller passed an operand set of the wrong arity — one slice for a
    /// two-operand route, say. It is stated as a typed refusal rather than an
    /// index panic because a wrong-arity operand set would otherwise either
    /// abort or, worse, silently reuse operand zero for every input.
    UnsuppliedOperand {
        entry: usize,
        slot: usize,
        ordinal: usize,
        supplied: usize,
    },
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
            // A resource stage rather than a launch-geometry one: the quantity
            // is storage the entry reserves, and it is compared against a
            // device capacity rather than against a pipeline's thread capacity.
            Self::ThreadgroupMemoryExceeded { .. }
            | Self::BindingExceedsBufferLimit { .. }
            | Self::UndersizedAllocation { .. }
            | Self::NoOutputBinding
            | Self::UnsuppliedOperand { .. } => PreflightPhase::Resources,
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
            | Self::ThreadgroupMemoryExceeded { .. }
            | Self::BindingExceedsBufferLimit { .. } => PreflightClass::RouteMiss,
            Self::UndersizedAllocation { .. }
            | Self::NoOutputBinding
            | Self::UnsuppliedOperand { .. } => PreflightClass::Systemic,
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
            Self::LibraryRejected { entry, detail } => write!(
                formatter,
                "entry {entry}'s carried object did not load: {detail}"
            ),
            Self::FunctionAbsent {
                entry,
                symbol,
                detail,
            } => write!(
                formatter,
                "entry {entry}'s library publishes no {symbol:?}: {detail}"
            ),
            Self::PipelineRejected {
                entry,
                symbol,
                detail,
            } => write!(
                formatter,
                "no pipeline state for entry {entry}'s {symbol:?}: {detail}"
            ),
            Self::WorkgroupTooLarge {
                entry,
                symbol,
                declared,
                capacity,
            } => write!(
                formatter,
                "entry {entry}'s {symbol:?} admits {capacity} thread(s) per threadgroup and the artifact declares {declared}"
            ),
            Self::ThreadgroupMemoryExceeded {
                entry,
                symbol,
                declared,
                capacity,
            } => write!(
                formatter,
                "entry {entry}'s {symbol:?} reserves {declared} byte(s) of threadgroup memory and this device admits {capacity}"
            ),
            Self::BindingExceedsBufferLimit {
                entry,
                slot,
                needed,
                limit,
            } => write!(
                formatter,
                "entry {entry} slot {slot} must reach {needed} byte(s) and one buffer holds at most {limit}"
            ),
            Self::UndersizedAllocation {
                entry,
                slot,
                needed,
                held,
            } => write!(
                formatter,
                "entry {entry} slot {slot} needs {needed} byte(s) and the allocation returned {held}"
            ),
            Self::NoOutputBinding => {
                formatter.write_str("no entry of this route binds the program output")
            }
            Self::UnsuppliedOperand {
                entry,
                slot,
                ordinal,
                supplied,
            } => write!(
                formatter,
                "entry {entry} slot {slot} takes declared input {ordinal} and this run supplied \
                 {supplied} operand set(s)"
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
    max_threadgroup_memory_length: u64,
    max_buffer_length: u64,
    recommended_working_set: u64,
    apple_family: ProbedGpuFamily,
}

/// One entry of a route, with the device objects its dispatch needs.
struct PreparedEntry {
    pipeline: ComputePipelineState,
    /// Buffers in this entry's own binding order, paired with the argument-table
    /// index and byte offset each occupies.
    placements: Vec<(u32, Buffer, u64)>,
    grid_threads: u64,
    threads_per_workgroup: u64,
    /// This entry covers no threads and the artifact says to skip its dispatch.
    ///
    /// Its buffers are still allocated and still retained: an empty producing
    /// stage shares its intermediate with the consumer that follows, and the
    /// consumer must bind an allocation rather than nothing.
    skipped: bool,
}

/// One route this device has proved it can carry out, with everything it needs.
///
/// Held across the commit: every device object the encode touches is created
/// here, so the post-commit path allocates nothing, looks nothing up, and has no
/// failure to report. That is the property the stage exists for.
///
/// Every buffer stays owned by this value until the command buffer completes.
/// Entry-internal storage is the loader's to allocate, and a shared intermediate
/// is referenced by two entries at once, so dropping either view would leave the
/// encoder holding a binding to a freed allocation. This value outlives the
/// `submit` call, which waits for the terminal state.
struct PreparedRoute {
    entries: Vec<PreparedEntry>,
    /// The buffer the program's output lands in, for read-back.
    output: Buffer,
    /// How many `f32` elements to read back out of it.
    readback: usize,
    facts: DeviceFacts,
}

/// Builds every exact entry pipeline before any deferred property is answered.
fn prepare_pipelines(
    device: &Device,
    entries: &[RoutedEntry<'_>],
) -> Result<Vec<ComputePipelineState>, PreflightRefusal> {
    entries
        .iter()
        .enumerate()
        .map(|(position, entry)| {
            let library = device
                .new_library_with_data(entry.object())
                .map_err(|detail| PreflightRefusal::LibraryRejected {
                    entry: position,
                    detail,
                })?;
            let symbol = entry.entry_symbol();
            let function = library.get_function(symbol, None).map_err(|detail| {
                PreflightRefusal::FunctionAbsent {
                    entry: position,
                    symbol: symbol.to_owned(),
                    detail,
                }
            })?;
            let descriptor = ComputePipelineDescriptor::new();
            descriptor.set_compute_function(Some(&function));
            device
                .new_compute_pipeline_state(&descriptor)
                .map_err(|detail| PreflightRefusal::PipelineRejected {
                    entry: position,
                    symbol: symbol.to_owned(),
                    detail,
                })
        })
        .collect()
}

/// Governed key of the Metal requirement naming a minimum Apple GPU family.
///
/// Owned by `tiler.metal`, which is the backend key this host states, so the
/// loader refuses a row owned by anything else before this adapter is asked.
const METAL_MINIMUM_GPU_FAMILY: &str = "tiler.metal.route-requirement.minimum-gpu-family";

/// Governed version of [`METAL_MINIMUM_GPU_FAMILY`]'s meaning.
///
/// Matched exactly. A version this adapter does not know is `Unrecognized`
/// rather than approximated, because one key at two versions can mean two
/// things and guessing which is how a route runs on a device it was refused on.
const METAL_MINIMUM_GPU_FAMILY_VERSION: u32 = 1;

/// Decides one live-device route requirement from normalized device facts.
///
/// Pure, and split from the device exactly as
/// [`tiler_metal::applicability`] splits its policy: an adapter observes, this
/// decides, and every case — including the ones no machine in this workspace can
/// produce — runs in the ordinary gate without Metal.
///
/// # What Metal cannot answer here, stated rather than approximated
///
/// [`RouteResourceDimension::SubgroupThreads`] is a live-device property in
/// general — Vulkan publishes `subgroupSize` on the physical device — and Metal
/// publishes no device-scoped equivalent: `MTLDevice.h` in the macOS 26.5 SDK
/// declares no execution-width property, and `threadExecutionWidth` lives on
/// `MTLComputePipelineState`, which is a *prepared-kernel* fact. So this adapter
/// answers `Unrecognized`, which refuses the route. A route that genuinely needs
/// that width on Metal must state it as a `PreparedEntryTargetRequirement`
/// against the prepared pipeline, which is the authority that has it; answering
/// it from a family table here would report a documentation constant as a device
/// observation.
fn decide_live_device_requirement(
    facts: &DeviceFacts,
    request: LiveDeviceRequest<'_>,
) -> LiveDeviceObservation {
    // Exhaustive on both the kind and the dimension: a row this adapter has
    // never seen must stop this build rather than reach an arm that guesses.
    match request.requirement() {
        RouteRequirement::ResourceFloor(floor) => match floor.dimension() {
            RouteResourceDimension::SubgroupThreads => LiveDeviceObservation::Unrecognized,
        },
        RouteRequirement::BackendFeature(feature) => {
            if feature.key().as_str() != METAL_MINIMUM_GPU_FAMILY
                || feature.version() != METAL_MINIMUM_GPU_FAMILY_VERSION
            {
                return LiveDeviceObservation::Unrecognized;
            }
            let Some(required) = gpu_family_from_payload(feature.payload()) else {
                return LiveDeviceObservation::Unrecognized;
            };
            // Cumulative families, which is the same property
            // `probe_apple_families` already relies on: the highest supported
            // family implies every lower one, so the ordering decides support
            // without a second device call. A device naming none of them
            // satisfies no family requirement.
            let supported = match facts.apple_family {
                ProbedGpuFamily::Answered(MetalGpuFamilySupport::Highest(highest)) => {
                    highest >= required
                }
                ProbedGpuFamily::Answered(MetalGpuFamilySupport::NoneNamed) => false,
                // This adapter owns the row and still has no observation to
                // decide it from, which is what `Unrecognized` is for: it
                // refuses the route. `Feature(false)` would be this adapter
                // reporting a device that answered no to a question its binding
                // could not put.
                ProbedGpuFamily::Unnameable(_) => return LiveDeviceObservation::Unrecognized,
            };
            LiveDeviceObservation::Feature(supported)
        }
    }
}

/// Reads a canonical family payload through the governed vocabulary's own spelling.
///
/// Scanned against `MetalGpuFamily::ALL` rather than matched against a second
/// table of names written here: one spelling authority, so a family added to
/// that vocabulary cannot be silently unreadable at this boundary.
fn gpu_family_from_payload(payload: &[u8]) -> Option<MetalGpuFamily> {
    MetalGpuFamily::ALL
        .into_iter()
        .find(|family| family.as_str().as_bytes() == payload)
}

/// Answers every live-device requirement of a route from this device.
fn qualify_live_device<'a>(
    device: &Device,
    qualification: LiveDeviceQualification<'a>,
) -> Result<RoutePreparation<'a>, ProofError> {
    let facts = device_facts(device);
    qualification
        .resolve_live_device_requirements(|request| decide_live_device_requirement(&facts, request))
        .map_err(ProofError::Load)
}

/// Answers each requirement from its exact prepared pipeline and preserves the pipelines for execution.
///
/// Two device stages in the order their facts become true: the live-device rows
/// first, from the bound device alone, and only then the prepared-entry
/// properties, which need a pipeline to exist. Nothing here is irreversible, so
/// abandoning between them is still the permitted fallback.
fn resolve_prepared_route<'a>(
    device: &Device,
    qualification: LiveDeviceQualification<'a>,
) -> Result<(Preflight<'a>, Vec<ComputePipelineState>), ProofError> {
    let preparation = qualify_live_device(device, qualification)?;
    let pipelines = prepare_pipelines(device, preparation.entries())
        .map_err(|refusal| ProofError::DevicePreflight(Box::new(refusal)))?;
    let preflight = preparation
        .resolve_target_properties(|request| {
            pipelines[request.entry()].max_total_threads_per_threadgroup()
        })
        .map_err(ProofError::Load)?;
    Ok((preflight, pipelines))
}

/// Proves this device can carry out a route, while declining is still permitted.
///
/// **Every entry, not the first one.** `prototype-metal-runtime-preflight` moved
/// every device-decidable obligation before the commit and bought the property
/// that `Preflight::commit` is infallible in fact rather than only in signature.
/// That property was stated over one entry: a two-entry route whose *second*
/// pipeline fails to build would reintroduce exactly the defect that ticket
/// removed. So the library, the function, the pipeline, and the launch capacity
/// are discharged per entry, and every refusal names the entry it came from —
/// "some pipeline in this route failed" is not actionable.
///
/// Nothing here is observable if the route is then abandoned: it allocates and
/// fills host-visible storage and creates pipeline state, and encodes nothing.
fn device_preflight(
    device: &Device,
    preflight: &Preflight<'_>,
    pipelines: &[ComputePipelineState],
    plan: &[Vec<PlacedSlot>],
    operands: &[Vec<u32>],
    readback: u64,
) -> Result<PreparedRoute, PreflightRefusal> {
    let facts = device_facts(device);
    let routed = preflight.entries();

    // Allocated before any entry is prepared, because a shared buffer belongs to
    // two entries and neither owns it. `None` marks a slot still to be filled by
    // the per-entry pass below.
    let mut storage: Vec<Vec<Option<Buffer>>> =
        plan.iter().map(|slots| vec![None; slots.len()]).collect();

    // The pairing the loader derived from the variant's own data dependencies.
    // One allocation is made and *both* slots reference it; a loader that
    // allocated per binding would hand the consumer a fresh buffer and it would
    // read uninitialised device memory — a wrong answer rather than a refusal.
    for shared in preflight.shared_allocations() {
        let (producer, consumer) = (shared.producer(), shared.consumer());
        let needed = plan[producer.entry()][producer.slot()]
            .needed
            .max(plan[consumer.entry()][consumer.slot()].needed);
        binding_fits(
            producer.entry(),
            producer.slot(),
            needed,
            facts.max_buffer_length,
        )?;
        let buffer = device.new_buffer(needed.max(1), MTLResourceOptions::StorageModePrivate);
        allocation_fits(producer.entry(), producer.slot(), needed, buffer.length())?;
        storage[producer.entry()][producer.slot()] = Some(buffer.clone());
        storage[consumer.entry()][consumer.slot()] = Some(buffer);
    }

    let mut output = None;
    let mut entries = Vec::with_capacity(routed.len());
    for (position, entry) in routed.iter().enumerate() {
        let symbol = entry.entry_symbol();
        let pipeline = pipelines[position].clone();

        let launch = entry.launch();
        workgroup_fits(
            position,
            symbol,
            launch.threads_per_workgroup(),
            pipeline.max_total_threads_per_threadgroup(),
        )?;
        // The requirement the *artifact* proved, against the capacity the
        // device reports. Read from the routed entry's own resource record
        // rather than from the prepared pipeline: the pipeline's static
        // reservation is what the compiled function happens to hold, and the
        // record is what the packaged program declared it needs. A disagreement
        // between them is a producer defect, and comparing the declared side is
        // what lets this refuse a route the device would otherwise accept and
        // then run short.
        local_memory_fits(
            position,
            symbol,
            entry.entry().resources().local_memory_bytes,
            facts.max_threadgroup_memory_length,
        )?;

        // Sized from the route rather than from the operand slice: the artifact
        // states how many bytes each binding must reach, and deriving a length
        // from the host's own data would re-answer a question it answered.
        let mut placements = Vec::with_capacity(plan[position].len());
        for (slot, placed) in plan[position].iter().enumerate() {
            binding_fits(position, slot, placed.needed, facts.max_buffer_length)?;
            // An occupied slot was already allocated as one half of a shared
            // pair, and taking it is what makes the two entries address one
            // buffer rather than two that merely have the same length.
            let buffer = if let Some(shared) = storage[position][slot].clone() {
                shared
            } else {
                let options = match placed.placement {
                    Placement::Input(_) | Placement::Output => {
                        MTLResourceOptions::StorageModeShared
                    }
                    Placement::Internal => MTLResourceOptions::StorageModePrivate,
                };
                let buffer = device.new_buffer(placed.needed.max(1), options);
                allocation_fits(position, slot, placed.needed, buffer.length())?;
                storage[position][slot] = Some(buffer.clone());
                buffer
            };
            match placed.placement {
                Placement::Input(ordinal) => {
                    // Indexed by the ordinal `plan_route` resolved from the
                    // artifact's own interface, so each operand buffer receives
                    // the payload the sidecar supplied for *that* input. The
                    // lookup cannot miss: the ordinal came from a position in
                    // this same interface, and the caller supplies one operand
                    // set per declared input.
                    let bits =
                        operands
                            .get(ordinal)
                            .ok_or(PreflightRefusal::UnsuppliedOperand {
                                entry: position,
                                slot,
                                ordinal,
                                supplied: operands.len(),
                            })?;
                    // The assertion inside `write_f32` is the backstop for a
                    // length disagreement, and it is unreachable here: each
                    // operand count was checked against the shape the artifact
                    // declares for its own input, and this buffer's length is
                    // that same shape's accessible byte range.
                    let values: Vec<f32> = bits.iter().map(|bits| f32::from_bits(*bits)).collect();
                    crate::buffer::write_f32(&buffer, &values);
                }
                Placement::Output => output = Some(buffer.clone()),
                Placement::Internal => {}
            }
            placements.push((placed.transport, buffer, placed.offset));
        }

        entries.push(PreparedEntry {
            pipeline,
            placements,
            grid_threads: launch.grid_threads(),
            threads_per_workgroup: launch.threads_per_workgroup(),
            // The pipeline above was still built for a skipped entry, and
            // deliberately: a route is only ready if every object it names
            // loads, and an entry that runs no threads on this input may run
            // some on the next one. Skipping preparation as well would make
            // readiness depend on the operands.
            skipped: launch.grid_threads() == 0,
        });
    }

    Ok(PreparedRoute {
        entries,
        // `plan_route` refuses every binding target this proof does not place,
        // and this program declares one output, so some entry bound it.
        output: output.ok_or(PreflightRefusal::NoOutputBinding)?,
        readback: usize::try_from(readback).expect("the proof's output count fits a usize"),
        facts,
    })
}

/// Whether a declared workgroup fits what a pipeline admits.
///
/// Split from the device call so the decision is testable without hardware: the
/// device contributes two numbers and this contributes the comparison.
fn workgroup_fits(
    entry: usize,
    symbol: &str,
    declared: u64,
    capacity: u64,
) -> Result<(), PreflightRefusal> {
    if declared > capacity {
        return Err(PreflightRefusal::WorkgroupTooLarge {
            entry,
            symbol: symbol.to_owned(),
            declared,
            capacity,
        });
    }
    Ok(())
}

/// Whether one entry's reserved threadgroup memory fits what this device admits.
///
/// Split from the device exactly as [`workgroup_fits`] is: the device
/// contributes two numbers and this contributes the comparison, so every case —
/// including the ones no machine in this workspace can produce — runs in the
/// ordinary gate without Metal.
///
/// The relation is `declared > capacity`, not `>=`: a function reserving exactly
/// the device maximum fits, and refusing it would reject a legal route.
fn local_memory_fits(
    entry: usize,
    symbol: &str,
    declared: u64,
    capacity: u64,
) -> Result<(), PreflightRefusal> {
    if declared > capacity {
        return Err(PreflightRefusal::ThreadgroupMemoryExceeded {
            entry,
            symbol: symbol.to_owned(),
            declared,
            capacity,
        });
    }
    Ok(())
}

/// Whether one binding's accessible range fits in a single buffer here.
fn binding_fits(
    entry: usize,
    slot: usize,
    needed: u64,
    limit: u64,
) -> Result<(), PreflightRefusal> {
    if needed > limit {
        return Err(PreflightRefusal::BindingExceedsBufferLimit {
            entry,
            slot,
            needed,
            limit,
        });
    }
    Ok(())
}

/// Whether an allocation the device returned reaches the length it was asked for.
fn allocation_fits(
    entry: usize,
    slot: usize,
    needed: u64,
    held: u64,
) -> Result<(), PreflightRefusal> {
    if held < needed {
        return Err(PreflightRefusal::UndersizedAllocation {
            entry,
            slot,
            needed,
            held,
        });
    }
    Ok(())
}

/// Reads what this device reports about itself.
fn device_facts(device: &Device) -> DeviceFacts {
    DeviceFacts {
        name: device.name().to_owned(),
        max_threads_per_threadgroup: device.max_threads_per_threadgroup().width,
        max_threadgroup_memory_length: device.max_threadgroup_memory_length(),
        max_buffer_length: device.max_buffer_length(),
        recommended_working_set: device.recommended_max_working_set_size(),
        apple_family: probe_apple_families(device),
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
    plan: &[Vec<PlacedSlot>],
    operands: &[Vec<u32>],
    readback: u64,
) -> Result<(), ProofError> {
    // Every perturbation below targets the route's first entry. One entry is
    // enough to establish that the device produces each refusal, and the
    // per-entry loop that applies them to the rest is device-free code the unit
    // cases cover.
    let first = preflight
        .entries()
        .first()
        .ok_or(ProofError::ProbeAccepted("a route with no entries"))?;
    // A library built from bytes that are not a metallib. The digest over these
    // bytes matched, so this is content that will not execute rather than an
    // integrity failure — the distinction `PreflightClass::CorruptArtifact`
    // exists to carry.
    let refusal = device
        .new_library_with_data(b"tiler probe object; not an executable image")
        .err()
        .map(|detail| PreflightRefusal::LibraryRejected { entry: 0, detail })
        .ok_or(ProofError::ProbeAccepted(
            "a library from non-metallib bytes",
        ))?;
    report_refusal("an object that is not a metallib", &refusal);

    // A symbol the real library does not publish.
    let library = device
        .new_library_with_data(first.object())
        .map_err(|detail| {
            ProofError::DevicePreflight(Box::new(PreflightRefusal::LibraryRejected {
                entry: 0,
                detail,
            }))
        })?;
    let refusal = library
        .get_function("tiler_kernel_this_object_does_not_publish", None)
        .err()
        .map(|detail| PreflightRefusal::FunctionAbsent {
            entry: 0,
            symbol: "tiler_kernel_this_object_does_not_publish".to_owned(),
            detail,
        })
        .ok_or(ProofError::ProbeAccepted("an absent entry symbol"))?;
    report_refusal("an entry symbol the object does not publish", &refusal);

    // A workgroup one thread larger than the pipeline admits, using the capacity
    // this device actually reported rather than an invented number. This is the
    // refusal that used to arrive after the commit.
    let function = library
        .get_function(first.entry_symbol(), None)
        .map_err(|detail| {
            ProofError::DevicePreflight(Box::new(PreflightRefusal::FunctionAbsent {
                entry: 0,
                symbol: first.entry_symbol().to_owned(),
                detail,
            }))
        })?;
    let descriptor = ComputePipelineDescriptor::new();
    descriptor.set_compute_function(Some(&function));
    let pipeline = device
        .new_compute_pipeline_state(&descriptor)
        .map_err(|detail| {
            ProofError::DevicePreflight(Box::new(PreflightRefusal::PipelineRejected {
                entry: 0,
                symbol: first.entry_symbol().to_owned(),
                detail,
            }))
        })?;
    let capacity = pipeline.max_total_threads_per_threadgroup();
    let refusal = workgroup_fits(0, first.entry_symbol(), capacity + 1, capacity)
        .err()
        .ok_or(ProofError::ProbeAccepted(
            "a workgroup larger than the pipeline admits",
        ))?;
    report_refusal("a workgroup one thread past this pipeline", &refusal);

    // An entry reserving one byte more threadgroup memory than this device
    // admits, using the capacity the device actually reported. The route's own
    // entries reserve none, so the quantity is injected rather than found:
    // what this establishes is that the *device's* reported capacity drives the
    // refusal, which is the half the device-free case cannot reach.
    let facts = device_facts(device);
    let threadgroup = facts.max_threadgroup_memory_length;
    let refusal = local_memory_fits(0, first.entry_symbol(), threadgroup + 1, threadgroup)
        .err()
        .ok_or(ProofError::ProbeAccepted(
            "an entry past this device's threadgroup memory",
        ))?;
    report_refusal(
        "an entry one byte past this device's threadgroup memory",
        &refusal,
    );

    // A binding needing one byte more than this device holds in one buffer.
    let limit = facts.max_buffer_length;
    let refusal = binding_fits(0, 0, limit + 1, limit)
        .err()
        .ok_or(ProofError::ProbeAccepted("a binding past the buffer limit"))?;
    report_refusal("a binding one byte past the buffer limit", &refusal);

    // The unperturbed route still prepares, which is what makes each refusal
    // above evidence about its own perturbation rather than about the route.
    let pipelines = prepare_pipelines(device, preflight.entries())
        .map_err(|refusal| ProofError::DevicePreflight(Box::new(refusal)))?;
    device_preflight(device, preflight, &pipelines, plan, operands, readback)
        .map_err(|refusal| ProofError::DevicePreflight(Box::new(refusal)))?;
    println!("  the unperturbed route prepares: every stage cleared before the commit");
    Ok(())
}

/// Prints one injected refusal with the phase and class it was classified into.
fn report_refusal(probe: &str, refusal: &PreflightRefusal) {
    println!("  {probe}: {refusal}");
}

/// Observes the terminal-status check refusing a real, live command buffer that
/// has not reached a terminal state.
///
/// **This is the contract's own warning case, injected rather than argued.**
/// `waitUntilCompleted` returns no success value, so the runtime execution
/// contract records that "a pre-wait non-error status is not evidence of
/// successful completion". A command buffer that has just been created and never
/// committed is exactly that: alive, valid, and carrying a status that must not
/// admit a readback. Nothing is committed and nothing is encoded, so the probe
/// costs one allocation and reaches no GPU work.
///
/// **The terminal `Error` state is deliberately not injected**, and the boundary
/// is stated rather than left as apparent coverage: forcing a command buffer to
/// fail means provoking a GPU fault, which risks a device reset and would not
/// reproduce. `one_status_permits_a_readback_and_none_permits_a_retry` covers
/// that arm over the complete status vocabulary without hardware.
fn probe_submission_status(device: &Device) -> Result<(), ProofError> {
    let queue = device.new_command_queue();
    let uncommitted = queue.new_command_buffer();
    match submission_outcome(uncommitted.status()) {
        SubmissionOutcome::NotTerminal(reported) => {
            println!(
                "  a live command buffer that was never committed: {reported}, no readback taken"
            );
            Ok(())
        }
        SubmissionOutcome::Completed | SubmissionOutcome::ExecutionError => Err(
            ProofError::ProbeAccepted("an uncommitted command buffer as a terminal state"),
        ),
    }
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
    debug_assert_eq!(
        routed.entries().len(),
        prepared.entries.len(),
        "the prepared route was built from these committed entries",
    );
    submit(
        device,
        &prepared.output,
        prepared.readback,
        |command_buffer| {
            // **One encoder per entry, and that is the ordering guarantee.**
            // Commands within a single compute encoder are not ordered against
            // each other unless the encoder's dispatch type says so, and a
            // second stage reading what the first wrote must not overlap it.
            // Metal orders *encoders* within a command buffer unconditionally,
            // with an implicit barrier between them, so a separate encoder per
            // entry needs no assumption about dispatch type at all.
            for entry in &prepared.entries {
                // Skipped entries are not encoded at all. Encoding an empty
                // encoder would be harmless and pointless; encoding a
                // zero-thread dispatch is what `plan_route` already refused.
                if entry.skipped {
                    continue;
                }
                let encoder = command_buffer.new_compute_command_encoder();
                encoder.set_compute_pipeline_state(&entry.pipeline);
                for (transport, storage, offset) in &entry.placements {
                    encoder.set_buffer(u64::from(*transport), Some(storage), *offset);
                }
                encoder.dispatch_threads(
                    MTLSize::new(entry.grid_threads, 1, 1),
                    MTLSize::new(entry.threads_per_workgroup, 1, 1),
                );
                encoder.end_encoding();
            }
        },
    )
}

/// Proves one published member against every operand case its sidecar carries.
///
/// **One routing authority per case, not one per member.** `DecodedProgram` is
/// not `Clone` and `preflight` takes `&mut self`, so a decoded program yields
/// exactly one commit — that is ADR 0051 expressed structurally rather than
/// remembered. Each case therefore decodes the envelope afresh. Reusing one
/// decode across cases would not compile, and reaching for a way to make it
/// compile would be dismantling the property on purpose.
///
/// The dispatch shape is asserted per case rather than once per member because
/// the shape is derived from the artifact on every route; checking it once would
/// leave the remaining cases free to route differently and still be reported as
/// agreeing.
fn prove_member(
    device: &Device,
    declaration: &BoundMetalCompileDeclaration,
    base: &Path,
    class: &str,
    role: &str,
) -> Result<usize, ProofError> {
    let path = proof_member(base, class, role);
    let (bytes, sidecar) = read_artifact(&path)?;

    // The shape is read from the artifact, exactly as the deep proof reads it,
    // and never taken from this crate's own `ROWS`; `compile_for_declared_shape`
    // is where that discipline lives and where the gate reaches it. Compiled
    // only to *name* what the artifact claims to package: the routed environment
    // comes from the declaration, not from this compilation, per
    // `declared_route_environment`.
    let declared_shape = DecodedProgram::decode(&bytes, SOLE_DELIVERY).map_err(ProofError::Load)?;
    let (rows, columns, compilation) = compile_for_declared_shape(declaration, &declared_shape)?;
    drop(declared_shape);
    let environment = declared_route_environment(declaration)?;

    // The cold-consumer assertion, stated once: these bytes were written beside
    // the artifact by the producing process, and this process is stating them as
    // the identity it expects. Checked as a recording — non-empty, bounded, under
    // this build's artifact domain — and not thereby evidence of anything.
    let recorded = RecordedArtifactProgramIdentity::from_bytes(sidecar.artifact_identity_bytes())
        .map_err(ProofError::RecordedIdentity)?;

    let (expected_entries, expected_shared) = expected_shape(role);
    let mut proved = 0_usize;

    for case in sidecar.cases() {
        // A fresh decode per case: see this function's own note on why.
        let mut decoded =
            DecodedProgram::decode(&bytes, SOLE_DELIVERY).map_err(ProofError::Load)?;
        // Re-read per case rather than trusted from above, because the shape is
        // what every remaining check is scaled by; a member whose variants
        // disagreed about it would otherwise be measured against the first one.
        let interface = bind_declared_interface(&decoded)?;
        require_serial_sum_interface(&interface)?;

        let operands = case_operands(&interface, case)?;
        let expected = case_expected(&interface, case)?;

        let preparation = decoded
            .prepare(&environment, &recorded, &interface.abi)
            .map_err(ProofError::Load)?;
        let (preflight, pipelines) = resolve_prepared_route(device, preparation)?;

        // Checked before the commit, because a route to a program this process
        // did not derive is a reason to abandon rather than to execute and
        // compare.
        require_derived_program(&compilation, preflight.kernel_program_identity())?;

        let plan = plan_route(&preflight, &interface)?;
        let prepared = device_preflight(
            device,
            &preflight,
            &pipelines,
            &plan,
            &operands,
            interface.output_elements,
        )
        .map_err(|refusal| ProofError::DevicePreflight(Box::new(refusal)))?;

        let routed = preflight.commit();
        let entries = routed.entries().len();
        let shared = routed.shared_allocations().len();
        if entries != expected_entries || shared != expected_shared {
            return Err(ProofError::UnexpectedRouteShape {
                member: format!("{class}.{role}"),
                expected_entries,
                entries,
                expected_shared,
                shared,
            });
        }

        let observed = dispatch_prepared(device, &routed, &prepared)?;
        if observed != expected {
            return Err(ProofError::Mismatch {
                path: "envelope",
                device: observed,
                reference: expected,
            });
        }
        proved += 1;
    }

    if proved == 0 {
        return Err(ProofError::SidecarWithoutCases);
    }
    println!(
        "  {class}.{role}: {rows}x{columns} declared, {proved} case(s) agree, \
         {expected_entries} dispatch(es), {expected_shared} shared allocation(s)",
    );
    Ok(proved)
}

/// One published contraction member, and what its executed bytes are compared
/// against.
///
/// **The two members are the same route and two different claims**, which is why
/// one function drives both rather than two functions sharing a helper. The
/// `2x2x3` member's result has more than one row *and* more than one column, so
/// it is the one that can separate the two operand access relations, and its
/// five operand classes are adversarial numerical cases with no measured device
/// result anywhere to compare against. The L3 cell's `1x1024` result cannot
/// separate those relations at all, and carries the one thing the other cannot:
/// a `result_sha256` a device measured over these exact operands.
struct ContractionMember {
    /// The class name `prototypes/serial-sum-compile` publishes it under.
    class: &'static str,
    /// The retained `direct` result digest, for a member the L3 realization
    /// probe measured.
    ///
    /// `None` is a statement rather than an omission: no measurement exists for
    /// the adversarial member's operands, so there is nothing to compare its
    /// executed bytes against beyond the published reference, and a comparison
    /// against a digest computed here would be this process checking itself.
    retained_result_sha256: Option<&'static str>,
}

/// The two contraction members this proof routes, in the order it routes them.
const CONTRACTION_MEMBERS: [ContractionMember; 2] = [
    ContractionMember {
        class: CONTRACTION_CLASS,
        retained_result_sha256: None,
    },
    ContractionMember {
        class: L3_CELL_CLASS,
        retained_result_sha256: Some(L3_CELL_RESULT_SHA256),
    },
];

/// How one member's executed bytes compared against a retained measurement.
///
/// **Three facts reported together, because on a mismatch each one narrows the
/// cause and no two of them are the same claim.** `executed` is the digest of
/// the bytes this device produced and is the deliverable. `embedded` is the
/// digest of the expected bytes the *producer* published beside the artifact,
/// and is a validity condition on the fixture: it says the published record
/// describes the probe's workload rather than some other operand set. Reporting
/// only the first would leave a mismatch unable to say whether the device
/// computed the wrong answer or the record asked the wrong question.
#[derive(Debug)]
struct RetainedComparison {
    executed: String,
    embedded: String,
    retained: &'static str,
}

impl RetainedComparison {
    /// Whether the executed bytes carry the retained digest.
    fn executed_matches(&self) -> bool {
        self.executed == self.retained
    }

    /// Whether the producer's published expectation carries it too.
    fn embedded_matches(&self) -> bool {
        self.embedded == self.retained
    }
}

/// The probe's digest domain: little-endian `f32` bytes in row-major order.
///
/// The readback already yields bit patterns in the buffer's own element order —
/// [`crate::buffer::read_f32`] copies the mapping out verbatim — so this is the
/// identity re-encoding of the bytes the device wrote, not a reinterpretation of
/// them. Written as `to_le_bytes` rather than a raw byte copy so the byte order
/// is stated where a reader can check it against the probe's host, which digests
/// the result buffer's storage directly.
fn result_digest(bits: &[u32]) -> String {
    let bytes: Vec<u8> = bits.iter().flat_map(|value| value.to_le_bytes()).collect();
    sha256_hex(&bytes)
}

/// FIPS 180-4 SHA-256 over a byte string, as lowercase hexadecimal.
///
/// **Written out here rather than reached for, and the reason is scope rather
/// than preference.** `sha2` is a workspace dependency and `tiler-artifact` —
/// which this crate already takes — owns the governed artifact digest, but that
/// API digests under a mandatory domain separator and cannot express the raw
/// pre-image the probe hashed, while adding `sha2` to this manifest would edit
/// `Cargo.lock`, which this work does not own.
/// `crates/tiler-compiler/src/governed/contraction_conformance.rs` reached the
/// same conclusion for the same reason and this is the same implementation.
///
/// It is checked against the two published FIPS 180-4 vectors before any
/// comparison rests on it, by `the_digest_helper_reproduces_the_published_vectors`
/// in this module's tests and again at run time in [`prove_contraction`]: a digest function that
/// silently computed something else would make every retained-value comparison
/// disagree, and a reader would have no way to tell that from a device defect.
///
fn sha256_hex(message: &[u8]) -> String {
    use std::fmt::Write as _;

    const K: [u32; 64] = [
        0x428a_2f98,
        0x7137_4491,
        0xb5c0_fbcf,
        0xe9b5_dba5,
        0x3956_c25b,
        0x59f1_11f1,
        0x923f_82a4,
        0xab1c_5ed5,
        0xd807_aa98,
        0x1283_5b01,
        0x2431_85be,
        0x550c_7dc3,
        0x72be_5d74,
        0x80de_b1fe,
        0x9bdc_06a7,
        0xc19b_f174,
        0xe49b_69c1,
        0xefbe_4786,
        0x0fc1_9dc6,
        0x240c_a1cc,
        0x2de9_2c6f,
        0x4a74_84aa,
        0x5cb0_a9dc,
        0x76f9_88da,
        0x983e_5152,
        0xa831_c66d,
        0xb003_27c8,
        0xbf59_7fc7,
        0xc6e0_0bf3,
        0xd5a7_9147,
        0x06ca_6351,
        0x1429_2967,
        0x27b7_0a85,
        0x2e1b_2138,
        0x4d2c_6dfc,
        0x5338_0d13,
        0x650a_7354,
        0x766a_0abb,
        0x81c2_c92e,
        0x9272_2c85,
        0xa2bf_e8a1,
        0xa81a_664b,
        0xc24b_8b70,
        0xc76c_51a3,
        0xd192_e819,
        0xd699_0624,
        0xf40e_3585,
        0x106a_a070,
        0x19a4_c116,
        0x1e37_6c08,
        0x2748_774c,
        0x34b0_bcb5,
        0x391c_0cb3,
        0x4ed8_aa4a,
        0x5b9c_ca4f,
        0x682e_6ff3,
        0x748f_82ee,
        0x78a5_636f,
        0x84c8_7814,
        0x8cc7_0208,
        0x90be_fffa,
        0xa450_6ceb,
        0xbef9_a3f7,
        0xc671_78f2,
    ];
    let mut state: [u32; 8] = [
        0x6a09_e667,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];
    let mut padded = message.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    let bit_length = u64::try_from(message.len())
        .expect("a message length fits in u64")
        .wrapping_mul(8);
    padded.extend_from_slice(&bit_length.to_be_bytes());

    let (blocks, remainder) = padded.as_chunks::<64>();
    debug_assert!(
        remainder.is_empty(),
        "the padding makes the length a multiple of 64"
    );
    for block in blocks {
        let mut schedule = [0_u32; 64];
        let (words, _) = block.as_chunks::<4>();
        for (slot, bytes) in schedule.iter_mut().zip(words) {
            *slot = u32::from_be_bytes(*bytes);
        }
        for index in 16..64 {
            let s0 = schedule[index - 15].rotate_right(7)
                ^ schedule[index - 15].rotate_right(18)
                ^ (schedule[index - 15] >> 3);
            let s1 = schedule[index - 2].rotate_right(17)
                ^ schedule[index - 2].rotate_right(19)
                ^ (schedule[index - 2] >> 10);
            schedule[index] = schedule[index - 16]
                .wrapping_add(s0)
                .wrapping_add(schedule[index - 7])
                .wrapping_add(s1);
        }
        // The eight working variables, indexed rather than named: the standard
        // calls them `a` through `h`, and eight single-letter bindings is a
        // readability rule this workspace holds even where the source it
        // transcribes does not.
        let mut working = state;
        for index in 0..64 {
            let s1 = working[4].rotate_right(6)
                ^ working[4].rotate_right(11)
                ^ working[4].rotate_right(25);
            let choice = (working[4] & working[5]) ^ (!working[4] & working[6]);
            let temp1 = working[7]
                .wrapping_add(s1)
                .wrapping_add(choice)
                .wrapping_add(K[index])
                .wrapping_add(schedule[index]);
            let s0 = working[0].rotate_right(2)
                ^ working[0].rotate_right(13)
                ^ working[0].rotate_right(22);
            let majority =
                (working[0] & working[1]) ^ (working[0] & working[2]) ^ (working[1] & working[2]);
            let temp2 = s0.wrapping_add(majority);
            working = [
                temp1.wrapping_add(temp2),
                working[0],
                working[1],
                working[2],
                working[3].wrapping_add(temp1),
                working[4],
                working[5],
                working[6],
            ];
        }
        for (slot, value) in state.iter_mut().zip(working) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut hex = String::with_capacity(64);
    for byte in state.iter().flat_map(|word| word.to_be_bytes()) {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

/// Proves the published two-input contraction end to end through the accepted route.
///
/// **This is the L3 remainder, and what it establishes is the *route* rather
/// than the realization.** The L3 record measured six contraction realizations
/// under a hand-written Objective-C host: a spike that produces no artifact, has
/// no identity, resolves no capability, and answers no applicability predicate.
/// What runs here is an offline-produced metallib loaded through the accepted
/// AOT path, with artifact identity carrying the offline compiler's provenance
/// and the exact native translator identity left `Unknown` per ADR 0086 — the
/// refusal [`offer_the_declared_profile`] prints before any routing commit.
///
/// # What makes this member different from every other one
///
/// Two tensor inputs. Every other program this proof routes declares one, so
/// this is the first route whose entries bind two program-input buffers, whose
/// sidecar carries two operand payloads per case, and whose ABI facts are bound
/// from two declared shapes. The functions it calls are the same ones the
/// serial-sum members call — [`bind_declared_interface`], [`plan_route`],
/// [`device_preflight`] — and that is deliberate: a second code path would have
/// let the one-input assumptions survive in the first.
///
/// # The ordering is the contract, not a sequence
///
/// Every obligation this host can decide is discharged while `Preflight` is
/// still held: the interface, the operand lengths, the derived program identity,
/// the placements, the pipelines, the launch capacity, and the allocations. Only
/// then is `commit` called, and nothing after it may take a fallback. The
/// command buffer's terminal state is checked inside [`submit`] *before* the
/// host reads a byte back, so a failed dispatch is reported as a dispatch
/// failure rather than compared as arithmetic.
///
/// # Which comparison a member makes, stated exactly
///
/// Every member's executed bytes are compared against the expected bytes the
/// producer published beside the artifact. A member carrying a
/// [`ContractionMember::retained_result_sha256`] additionally has the SHA-256 of
/// **the bytes this device produced** compared against a digest a device
/// measured — which is the only comparison here that reaches outside this
/// workspace's own two implementations of the contraction. The digest of the
/// producer's *expected* bytes is computed and reported beside it, and it is a
/// validity condition on the published record rather than a second device claim:
/// it says the fixture asks the probe's question, and it would agree with the
/// retained value even if nothing had been dispatched at all.
fn prove_contraction(
    device: &Device,
    declaration: &BoundMetalCompileDeclaration,
    base: &Path,
    member: &ContractionMember,
) -> Result<usize, ProofError> {
    let path = proof_member(base, member.class, "selected");
    let (bytes, sidecar) = read_artifact(&path)?;
    let environment = declared_route_environment(declaration)?;
    let recorded = RecordedArtifactProgramIdentity::from_bytes(sidecar.artifact_identity_bytes())
        .map_err(ProofError::RecordedIdentity)?;

    // Read once for the report; every case re-reads it from its own decode.
    let declared = DecodedProgram::decode(&bytes, SOLE_DELIVERY).map_err(ProofError::Load)?;
    let shape = bind_declared_interface(&declared)?;
    let (m, n, k) = require_contraction_interface(&shape)?;
    drop(declared);
    // Compiled only to *name* the program the artifact claims to package, for
    // the shape the artifact itself declares. Nothing emitted here reaches the
    // device; what this buys is the one binding between the two processes a
    // sidecar cannot forge, and it is checked before the commit because a route
    // to a program this process did not derive is a reason to abandon rather
    // than to execute and compare.
    let compilation = compile_under(declaration, &contraction_program(m, n, k))?;
    println!(
        "  the artifact declares {} input(s): {} -> {:?} [{m}, {n}], contracted extent {k}",
        shape.inputs.len(),
        shape
            .inputs
            .iter()
            .map(|input| format!("{:?} {:?}", input.key, input.extents))
            .collect::<Vec<_>>()
            .join(", "),
        shape.output_key,
    );

    // The fail-closed probes against these exact bytes, before the positive
    // route is claimed. This is where the closing condition's "a deliberately
    // corrupted artifact is refused rather than executed" is discharged for the
    // contraction: `probe_damaged_section_content` flips a byte of the carried
    // metallib and requires the refusal, and `probe_accepted_baseline` requires
    // the unperturbed subject to route, so the refusal is evidence about the
    // damage rather than about the member.
    println!("  fail-closed probes against the contraction's exact bytes:");
    probe_fail_closed(&ProbeSubject {
        bytes: &bytes,
        expected: &recorded,
        environment: &environment,
        abi: &shape.abi,
    })?;

    let mut proved = 0_usize;
    for case in sidecar.cases() {
        // A fresh decode per case, for the reason `prove_member` records: a
        // decoded program yields exactly one commit, which is ADR 0051
        // expressed structurally rather than remembered.
        let mut decoded =
            DecodedProgram::decode(&bytes, SOLE_DELIVERY).map_err(ProofError::Load)?;
        let interface = bind_declared_interface(&decoded)?;
        require_contraction_interface(&interface)?;

        let operands = case_operands(&interface, case)?;
        let expected = case_expected(&interface, case)?;

        let preparation = decoded
            .prepare(&environment, &recorded, &interface.abi)
            .map_err(ProofError::Load)?;
        let (preflight, pipelines) = resolve_prepared_route(device, preparation)?;
        require_derived_program(&compilation, preflight.kernel_program_identity())?;
        let plan = plan_route(&preflight, &interface)?;

        // Both operand buffers are placed and filled here, before the commit,
        // and the count is asserted rather than assumed: a route that bound one
        // program-input slot would leave the second operand unwritten and
        // return a tensor computed from an uninitialised buffer.
        let bound_inputs: usize = plan
            .iter()
            .flatten()
            .filter(|slot| matches!(slot.placement, Placement::Input(_)))
            .count();
        if bound_inputs != interface.inputs.len() {
            return Err(ProofError::UnboundOperand {
                bound: bound_inputs,
                declared: interface.inputs.len(),
            });
        }

        let prepared = device_preflight(
            device,
            &preflight,
            &pipelines,
            &plan,
            &operands,
            interface.output_elements,
        )
        .map_err(|refusal| ProofError::DevicePreflight(Box::new(refusal)))?;

        // ---- the routing commit, one way ---------------------------------
        let routed = preflight.commit();
        let observed = dispatch_prepared(device, &routed, &prepared)?;

        // Both digests are taken before either comparison, so a mismatch reports
        // all three values at once. Returning at the first disagreement would
        // hide exactly the fact that separates a device defect from a fixture
        // that asks the wrong question.
        let comparison = member
            .retained_result_sha256
            .map(|retained| RetainedComparison {
                executed: result_digest(&observed),
                embedded: result_digest(&expected),
                retained,
            });

        if observed != expected {
            return Err(ProofError::Mismatch {
                path: "contraction",
                device: observed,
                reference: expected,
            });
        }
        // Both verdicts, not one: a member is only proved against its retained
        // measurement when the bytes this device produced *and* the record they
        // were compared through carry the digest.
        if let Some(comparison) = &comparison
            && !(comparison.executed_matches() && comparison.embedded_matches())
        {
            return Err(ProofError::RetainedDigestMismatch {
                member: member.class,
                case: case.key().as_str().to_owned(),
                executed: comparison.executed.clone(),
                embedded: comparison.embedded.clone(),
                retained: comparison.retained,
            });
        }

        // The whole result is printed for a handful of elements and elided for a
        // profile cell: a thousand hexadecimal words is not a reader's evidence,
        // and the digest below is. The element count is stated either way so an
        // elided line still says how much agreed.
        match &comparison {
            None => println!(
                "    {}: {observed:08x?} against {expected:08x?}",
                case.key(),
            ),
            Some(comparison) => println!(
                "    {}: {} element(s) agree with the published reference; SHA-256 of the \
                 EXECUTED result bytes {} == retained {} (the producer's published expectation \
                 hashes to {}, which is this fixture's validity condition and not a second \
                 device claim)",
                case.key(),
                observed.len(),
                comparison.executed,
                comparison.retained,
                comparison.embedded,
            ),
        }
        proved += 1;
    }

    if proved == 0 {
        return Err(ProofError::SidecarWithoutCases);
    }
    println!(
        "  {}: {m}x{n}x{k} contraction, {proved} operand case(s) agree bit for bit with the \
         published reference, over {} declared operand(s){}",
        member.class,
        shape.inputs.len(),
        if member.retained_result_sha256.is_some() {
            ", and the executed bytes carry the retained realization-probe digest"
        } else {
            "; no realization-probe measurement exists for these operands"
        },
    );
    Ok(proved)
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
    // A *base* path now, not an envelope. The producer publishes a matrix of
    // members beneath it, and the deep single-member proof below runs against
    // the nontrivial fused member because that is the one the optimizer
    // normally selects — the case a consumer would actually get.
    let base = artifact_path()?;
    let envelope_path = proof_member(&base, "nontrivial", "selected");
    let program = serial_sum_program(ROWS, COLUMNS);
    let bits = input_bits(ROWS, COLUMNS);
    let reference = reference_bits(&program, &bits, ROWS, COLUMNS);

    let device = &Device::system_default().ok_or(ProofError::NoDevice)?;
    println!("device: {}", device.name());

    let declaration = declaration()?;
    println!(
        "authoritative target profile: {} ({} descriptor byte(s)), AOT target {} under {}",
        declaration.profile().profile_key(),
        declaration.profile().canonical_descriptor().len(),
        declaration.aot_target().triple(),
        declaration.aot_target().std_token(),
    );

    // ---- the production offer path ---------------------------------------
    // Asked here, ahead of every routing commit this run makes, and this is the
    // only question in this binary whose answer would be an authority claim. It
    // refuses; the refusal is the deliverable.
    println!("production offer path — earn the right to offer this exact profile:");
    let refusal = offer_the_declared_profile(device);
    println!("  REFUSED before any routing commit: {refusal}");
    println!(
        "  predicate {}, rule {}",
        refusal.predicate(),
        refusal.rule(),
    );
    println!(
        "  consequence: this host does not offer {}. Everything below routes on \
         producer-declared equality, NOT host-earned eligibility.",
        declaration.profile().profile_key(),
    );

    // ---- the direct path -------------------------------------------------
    let compilation = compile_under(&declaration, &program)?;
    let selected = compilation.selected().ok_or(ProofError::NoSelection)?;
    println!("selected alternative: {}", selected.stable_id());

    let kernels: Vec<_> = selected.kernels().iter().collect();
    let unit = emit_translation_unit(&kernels, declaration.metal_facts(), declaration.emission())
        .map_err(|_| ProofError::Emit)?;
    // Emission succeeds even when the target cannot honour the declared
    // contract, so conformance is asked explicitly rather than inferred.
    unit.require_declared_realization()
        .map_err(|_| ProofError::UnrealizableNumerics)?;

    // The AOT target is the declaration's own total projection of the same
    // Metal facts, not a second spelling of a target this file chose. The
    // previous spelling named `air64-apple-macos14.0` under MSL 3.1 while the
    // measurements this profile carries were taken at MSL 4.0 for macOS 26.0.
    let request = CompileRequest::new(
        unit.source(),
        declaration.aot_target(),
        OptimizationLevel::Default,
        declaration.numerical_realization(),
    );
    let compiled = Toolchain::system()
        .compile(&request)
        .map_err(|_| ProofError::Toolchain)?;
    println!("compiled {} bytes of metallib", compiled.metallib.len());
    let emitted = unit.entry_points().first().ok_or(ProofError::Emit)?;
    let direct = dispatch_direct(device, &compiled.metallib, emitted.symbol(), &bits)?;

    // ---- the parallel reduction strategies -------------------------------
    // Still the direct path — local knowledge, no envelope — and run before the
    // envelope path so a compiler or emitter defect surfaces as itself rather
    // than as a delivery failure.
    println!("the parallel reduction strategies, dispatched and compared against the reference:");
    prove_parallel_strategies(device, &declaration)?;

    // ---- the grouping-sensitive half of the same pair ---------------------
    // The section above proves each strategy reduces the declared contributor
    // *set* correctly, on operands whose every grouping is exact. This one
    // proves each strategy rounds the way the grouping it published rounds, on
    // operands where the groupings disagree — which the section above cannot
    // observe and does not claim.
    println!(
        "the same strategies on grouping-sensitive operands, each held to its own declared \
              grouping:"
    );
    prove_grouping_sensitive_case(device, &declaration)?;

    // ---- the envelope path -----------------------------------------------
    let (bytes, sidecar) = read_artifact(&envelope_path)?;
    let mut decoded = DecodedProgram::decode(&bytes, SOLE_DELIVERY).map_err(ProofError::Load)?;
    println!(
        "decoded: {} variant(s), required features {:?}",
        decoded.variant_count(),
        decoded.required_features(),
    );
    let interface = bind_declared_interface(&decoded)?;
    let (rows, columns) = require_serial_sum_interface(&interface)?;
    let abi = interface.abi.clone();
    println!("the artifact declares a {rows} by {columns} input");
    // Producer-declared equality, NOT host-earned eligibility. The refusal above
    // is the applicability answer; this environment states the profile the
    // producer declared so the runtime machinery below can be exercised.
    let environment = declared_route_environment(&declaration)?;
    println!(
        "envelope route environment: DIAGNOSTIC — producer-declared equality against {}, NOT \
         host-earned eligibility",
        environment.target_profile.key.as_str(),
    );

    // The identity the producing process recorded beside these bytes, stated by
    // this process as the one it expects. See `prove_member` for why it is a
    // recording rather than a derivation.
    let recorded = RecordedArtifactProgramIdentity::from_bytes(sidecar.artifact_identity_bytes())
        .map_err(ProofError::RecordedIdentity)?;

    // Established before the positive route is claimed: a loader that accepted
    // these bytes would say nothing about what it refuses, and the refusals are
    // half of what makes the acceptance mean anything.
    println!("fail-closed probes against these exact bytes:");
    probe_fail_closed(&ProbeSubject {
        bytes: &bytes,
        expected: &recorded,
        environment: &environment,
        abi: &abi,
    })?;

    let preparation = decoded
        .prepare(&environment, &recorded, &abi)
        .map_err(ProofError::Load)?;

    // Compiled here only to *name* the program the artifact claims to package.
    // Nothing is emitted, nothing is linked, and nothing from it reaches the
    // device: the check is that the packaged kernel program's canonical identity
    // is the one this build derives for the shape the artifact declares, and
    // that is the one binding between the two processes a sidecar cannot forge.
    let envelope_program = serial_sum_program(rows, columns);
    let envelope_compilation = compile_under(&declaration, &envelope_program)?;
    let envelope_plan = envelope_compilation
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
    if preparation.kernel_program_identity() != local {
        return Err(ProofError::ForeignRoutedProgram {
            routed: preparation.kernel_program_identity().len(),
            derived: local.len(),
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
    let envelope_operands = case_operands(&interface, case)?;
    let envelope_reference = case_expected(&interface, case)?;

    // Placement first, then the device. Both are decided while a fallback is
    // still permitted, and between them they discharge every obligation this
    // host can decide — which is what makes the commit below infallible in fact
    // and not only in signature. See `plan_route` and `device_preflight`.
    let (preflight, pipelines) = resolve_prepared_route(device, preparation)?;
    let plan = plan_route(&preflight, &interface)?;
    let prepared = device_preflight(
        device,
        &preflight,
        &pipelines,
        &plan,
        &envelope_operands,
        interface.output_elements,
    )
    .map_err(|refusal| ProofError::DevicePreflight(Box::new(refusal)))?;
    let facts = &prepared.facts;
    println!(
        "device preflight: {} ({}), {} thread(s) per threadgroup, {} byte(s) of threadgroup \
         memory, buffers to {} byte(s), working set {} byte(s)",
        facts.name,
        facts.apple_family,
        facts.max_threads_per_threadgroup,
        facts.max_threadgroup_memory_length,
        facts.max_buffer_length,
        facts.recommended_working_set,
    );
    println!("device-preflight refusals against this exact route:");
    probe_device_preflight(
        device,
        &preflight,
        &plan,
        &envelope_operands,
        interface.output_elements,
    )?;
    println!("post-commit refusals, which no fallback follows:");
    probe_submission_status(device)?;

    let routed = preflight.commit();
    println!(
        "routed: {} entr(y/ies) in execution order, {} shared allocation(s)",
        routed.entries().len(),
        routed.shared_allocations().len(),
    );
    for (position, entry) in routed.entries().iter().enumerate() {
        println!(
            "  entry {position}: symbol {:?}, {} object byte(s), {} thread(s) in groups of {}",
            entry.entry_symbol(),
            entry.object().len(),
            entry.launch().grid_threads(),
            entry.launch().threads_per_workgroup(),
        );
        for binding in entry.bindings() {
            println!(
                "    abi slot {} -> transport {} at byte {}, {} byte(s), {:?}",
                binding.slot(),
                binding.transport_slot(),
                binding.accessible_offset(),
                binding.accessible_bytes(),
                binding.binding().target(),
            );
        }
    }
    for shared in routed.shared_allocations() {
        println!(
            "  shared: entry {} slot {} writes what entry {} slot {} reads",
            shared.producer().entry(),
            shared.producer().slot(),
            shared.consumer().entry(),
            shared.consumer().slot(),
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

    // ---- the matrix ------------------------------------------------------
    // The deep proof above establishes one member in detail: the refusals, the
    // pre-commit boundary, the post-commit behaviour, and one operand case. It
    // says nothing about the optimization, because a fused plan compared only
    // against itself is self-consistent by construction.
    //
    // This pass is where the claim is made. Each reduction class is proved
    // twice — once as the fused single-dispatch plan the optimizer selects, once
    // as the materialized plan that computes the same function through two
    // dispatches and one intermediate — over every operand class the producer
    // published. Agreement between them is a statement about the optimizer;
    // agreement with the sidecar's expected bytes is a statement about both.
    println!("the proof matrix, every published member against every operand case:");
    let mut proved = 0_usize;
    // The reduced extent names the member on disk; it is not handed to
    // `prove_member`, which reads the shape from the artifact it opened. Same
    // discipline `bind_interface` documents: what this runner may take from an
    // artifact is what the artifact says.
    for (class, _reduced_extent) in REDUCTION_CLASSES {
        for role in PLAN_ROLES {
            proved += prove_member(device, &declaration, &base, class, role)?;
        }
    }
    println!(
        "{proved} case(s) proved across {} member(s); fused and materialized agree bit for bit \
         with the published reference",
        REDUCTION_CLASSES.len() * PLAN_ROLES.len(),
    );

    // ---- the contraction vertical ----------------------------------------
    // The L3 remainder: contractions of the profile's index structure
    // `td,od->to`, carried through the accepted AOT and runtime route rather
    // than through a spike's own dispatch host. It runs last because everything
    // above establishes that the route works for a one-operand program, so a
    // failure here isolates to what these members add — a second tensor input,
    // and then a profile cell with a measured result to be compared against.
    //
    // The digest helper is checked against the published FIPS 180-4 vectors here
    // rather than only in the gate, because the comparison it carries out below
    // is the one claim in this binary that reaches a value measured outside this
    // workspace, and a helper that computed something else would report that
    // claim as failing for a reason no output would name.
    println!("the contraction vertical, through the accepted AOT and runtime route:");
    require_digest_vectors()?;
    let mut contraction_cases = 0_usize;
    let mut retained_compared = 0_usize;
    for member in &CONTRACTION_MEMBERS {
        contraction_cases += prove_contraction(device, &declaration, &base, member)?;
        retained_compared += usize::from(member.retained_result_sha256.is_some());
    }
    println!(
        "{contraction_cases} contraction case(s) proved through the accepted route across {} \
         member(s); {retained_compared} of them had the SHA-256 of its EXECUTED result bytes \
         compared against a retained L3 realization-probe measurement, on the host row that \
         measurement was taken on",
        CONTRACTION_MEMBERS.len(),
    );
    Ok(())
}

/// Requires the local digest helper to reproduce the published SHA-256 vectors.
///
/// A comparison against a sixty-four character constant passes trivially if the
/// bytes never reach it, and fails opaquely if the function hashing them is not
/// SHA-256. This is the second half: the run-time check that the helper carrying
/// the retained comparison is the algorithm that measurement was taken with.
fn require_digest_vectors() -> Result<(), ProofError> {
    const VECTORS: [(&[u8], &str); 2] = [
        (
            b"",
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            b"abc",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
    ];
    for (message, expected) in VECTORS {
        let observed = sha256_hex(message);
        if observed != expected {
            return Err(ProofError::DigestHelper {
                message: message.len(),
                observed,
                expected,
            });
        }
    }
    println!("  digest helper: both published FIPS 180-4 vectors reproduced");
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
    /// A case binds a different number of operands than the artifact declares.
    ///
    /// Reachable only through a reader defect: the sidecar layer refuses a case
    /// whose payload count is not the artifact's declared input count, so both
    /// halves of this comparison come from records that already agree. It is a
    /// named refusal rather than a `zip` that silently takes the shorter side,
    /// which is what would drop the second operand of a contraction.
    SidecarInterfaceArity {
        sidecar: usize,
        artifact: usize,
    },
    /// A case's operand is placed under a different key than the artifact declares.
    SidecarInterfaceKey {
        sidecar: String,
        artifact: String,
    },
    SidecarAssociation(ProofAssociationError),
    /// The identity recorded beside the artifact is not statable as one.
    ///
    /// Its own class rather than a load rejection: nothing was loaded. What
    /// failed is this process's *assertion* about which artifact it wants, so
    /// the repair is in the recording rather than in the envelope.
    RecordedIdentity(RecordedArtifactIdentityError),
    /// The authoritative macOS Metal declaration did not assemble.
    Declaration(BoundMetalDeclarationError),
    /// The declared profile is not a valid singleton target request.
    TargetRequest(TargetRequestError),
    Compile(CompileFailure),
    NoSelection,
    /// A parallel reduction strategy this profile is meant to retain is absent.
    ///
    /// A hard failure rather than a skip. `tiler-build`'s own portfolio fixture
    /// asserts that a flush-and-reassociate contract retains both strategies on
    /// this exact profile, so an absence here is a regression in the compiler
    /// or in this binary's recognition of it — and a run that quietly proved one
    /// strategy while reporting two would be worse than a red one.
    StrategyAbsent {
        strategy: ParallelStrategy,
        retained: usize,
    },
    /// The parallel strategies replaced the serial fold rather than joining it.
    ///
    /// Its own class because the repair is the opposite of [`Self::StrategyAbsent`]'s:
    /// a portfolio that dropped the fold has become *narrower* under a contract
    /// that only widens permissions, which is a pruning defect rather than a
    /// missing strategy.
    SerialFoldReplaced {
        retained: usize,
    },
    /// An alternative's published launch geometry names no covering partition.
    ///
    /// A refusal rather than a fallback to some default grouping: both parallel
    /// strategies decline an inexact split rather than padding one, so a
    /// partition that does not cover the contributor sequence exactly once each
    /// means this reader stopped measuring what it names — and an oracle asked
    /// about the wrong order would report the device as wrong.
    UndeclaredGrouping {
        strategy: String,
        detail: String,
    },
    /// The partitioned oracle at the declared serial order disagrees with the
    /// reference evaluator's run of the whole program.
    ///
    /// The calibration that makes every per-strategy comparison mean something,
    /// and its own class because the repair is in neither the device nor the
    /// strategies: either this file is asking the oracle about an order the
    /// program does not declare, or the pointwise prologue is not bit-identity
    /// on these operands and the reduction oracle may not be applied to them.
    GroupingOracleUncalibrated {
        evaluator: Vec<u32>,
        partitioned: Vec<u32>,
    },
    /// Every legal regrouping of these operands produces one value, so the
    /// oracle has nothing it could refuse.
    ///
    /// Reported as loudly as a wrong answer, because a check that cannot fail
    /// measures nothing. This is the exact condition [`PARALLEL_OPERANDS`] is
    /// in by construction, and reaching it here means the grouping-sensitive
    /// operands stopped being sensitive.
    NoRefusableGrouping {
        strategy: String,
        permitted: Vec<u32>,
    },
    /// A strategy's answer is not the one its own declared grouping produces.
    ///
    /// Distinct from [`Self::Mismatch`] because the reference is different: this
    /// carries the split the plan published, so a reader sees which order was
    /// expected rather than only that two bit patterns differ. Under a
    /// reassociating contract "the reference" is not a single value, and a
    /// message implying one would send a reader looking for the wrong defect.
    GroupingMismatch {
        strategy: String,
        partitions: u64,
        contributors_per_partition: u64,
        device: Vec<u32>,
        expected: Vec<u32>,
    },
    /// An ABI launch quantity is not the declared literal this reader requires.
    NonLiteralLaunch {
        position: u32,
        node: String,
    },
    /// The alternative's program publishes no named output to read back.
    NoProgramOutput,
    /// The route bound storage for fewer program inputs than the artifact declares.
    ///
    /// The check that makes "two inputs actually reached the device" a measured
    /// fact rather than an inference from the interface. A route binding one
    /// slot for a two-operand program would dispatch against an unwritten
    /// buffer, and an unwritten `StorageModeShared` allocation is zeroed rather
    /// than poisoned — so the result would be a plausible tensor, not a crash.
    UnboundOperand {
        bound: usize,
        declared: usize,
    },
    /// A member's bytes do not carry the digest a device measured over the same
    /// operands.
    ///
    /// **Its own class rather than a [`Self::Mismatch`], because it is a
    /// different disagreement.** `Mismatch` says this device and the published
    /// reference computed different bits. This says the two agreed with each
    /// other and disagreed with a *measurement* — which is a correctness finding
    /// about the vertical rather than about one dispatch, and the three digests
    /// it carries are what narrow it: an `embedded` that already misses the
    /// retained value indicts the operands the producer generated, and an
    /// `embedded` that matches while `executed` does not cannot happen without
    /// `Mismatch` firing first, so the pairing is itself diagnostic.
    RetainedDigestMismatch {
        member: &'static str,
        case: String,
        executed: String,
        embedded: String,
        retained: &'static str,
    },
    /// The local SHA-256 helper does not reproduce a published FIPS 180-4 vector.
    ///
    /// Separate from every comparison that uses it: a digest function computing
    /// something other than SHA-256 makes every retained-value comparison fail,
    /// and reporting that as a numerical disagreement would name the device for
    /// this process's own defect.
    DigestHelper {
        message: usize,
        observed: String,
        expected: &'static str,
    },
    /// The direct path was handed a program with more than one tensor input.
    ///
    /// The direct path binds one operand slice by local knowledge, so it cannot
    /// place a second input without being told which is which. A multi-operand
    /// program belongs on the envelope path, where the artifact's declared
    /// interface supplies the ordinals.
    DirectPathMultiInput {
        inputs: usize,
    },
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
    /// The packaged program's identity matches none of the alternatives this
    /// process compiles for the artifact's own declared shape.
    ///
    /// Two variants rather than one, because the two mismatches carry different
    /// numbers: this one compares one identity against a *count* of candidate
    /// alternatives, and rendering that count beside a byte length once read as
    /// "compiled one of 2 \[bytes\]" — a misdirection at exactly the moment
    /// somebody is diagnosing a drift.
    ForeignProgram {
        /// Byte length of the packaged program's canonical identity.
        packaged: usize,
        /// How many compiled alternatives were checked, none of which matched.
        alternatives: usize,
    },
    /// The routed program's identity is not the one this process constructed.
    ForeignRoutedProgram {
        /// Byte length of the identity the route prepared.
        routed: usize,
        /// Byte length of the identity this process derived.
        derived: usize,
    },
    /// A member routed to a different number of dispatches than its role means.
    ///
    /// The fused and materialized members must not converge on one shape. If
    /// they did, their bit-for-bit agreement would be the agreement of one
    /// program with itself, which proves nothing about the optimization the
    /// proof exists to check.
    UnexpectedRouteShape {
        member: String,
        expected_entries: usize,
        entries: usize,
        expected_shared: usize,
        shared: usize,
    },
    UnboundBinding {
        entry: usize,
        slot: usize,
        target: String,
    },
    BindingRangeOverflow {
        entry: usize,
        slot: usize,
        offset: u64,
        extent: u64,
    },
    EmptyLaunch {
        entry: usize,
        skipped: bool,
    },
    /// The device refused the route, before any commit.
    ///
    /// Boxed because it is the largest variant by a wide margin and every other
    /// one would otherwise pay for it. It carries the phase and the class rather
    /// than a rendered string, so a caller decides whether to re-route,
    /// re-fetch, or stop without parsing this.
    DevicePreflight(Box<PreflightRefusal>),
    /// An injected perturbation was *accepted* rather than refused.
    ///
    /// A probe that cannot fail measures nothing, so a perturbation something
    /// admits is reported as loudly as a refusal arriving in the wrong stage.
    /// Raised by both the device-preflight probes and the post-commit
    /// submission probe.
    ProbeAccepted(&'static str),
    LibraryLoad(String),
    FunctionLookup(String),
    Pipeline(String),
    /// The command buffer did not reach `Completed`, so nothing was read back.
    ///
    /// Carries the status it stopped in and what that status means, and makes no
    /// claim about *why* the device rejected the work: `metal` 0.33.0 exposes no
    /// accessor for the buffer's `NSError`, and reading it would be a new unsafe
    /// site whose only product is a better message. ADR 0079 does not admit a
    /// site for that — convenience is not a qualifying reason — so the boundary
    /// is recorded rather than crossed.
    Dispatch {
        status: &'static str,
        detail: &'static str,
    },
    Mismatch {
        path: &'static str,
        device: Vec<u32>,
        reference: Vec<u32>,
    },
}

impl fmt::Display for ProofError {
    // One arm per variant, and the match stays exhaustive on purpose: a wildcard
    // is what would stop a newly added variant from failing to compile here.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str(
                "usage: tiler-prototype-run --artifact <path>; create it first with \
                 `cargo run -p tiler-prototype-compile -- --out <path>`",
            ),
            Self::Read(path, cause) => write!(formatter, "{path} could not be read: {cause}"),
            Self::Declaration(cause) => write!(
                formatter,
                "the authoritative Metal declaration did not assemble: {cause}",
            ),
            Self::TargetRequest(cause) => write!(
                formatter,
                "the declared profile is not a valid target request: {cause}",
            ),
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
            Self::SidecarInterfaceArity { sidecar, artifact } => write!(
                formatter,
                "a proof case binds {sidecar} operand payload(s) and the artifact declares \
                 {artifact} input(s)",
            ),
            Self::SidecarInterfaceKey { sidecar, artifact } => write!(
                formatter,
                "a proof case places an operand under {sidecar:?} where the artifact declares \
                 {artifact:?}",
            ),
            Self::Sidecar(cause) => {
                write!(formatter, "the proof sidecar did not decode: {cause}")
            }
            Self::SidecarAssociation(cause) => write!(
                formatter,
                "the proof sidecar does not describe this envelope: {cause}"
            ),
            Self::RecordedIdentity(cause) => write!(
                formatter,
                "the recorded artifact identity is not statable: {cause}"
            ),
            Self::Compile(failure) => write!(formatter, "the program did not compile: {failure:?}"),
            Self::NoSelection => formatter.write_str("the portfolio retained no selected plan"),
            Self::StrategyAbsent { strategy, retained } => write!(
                formatter,
                "the portfolio retained {retained} alternative(s) and none of them is the {strategy}"
            ),
            Self::SerialFoldReplaced { retained } => write!(
                formatter,
                "the portfolio retained {retained} alternative(s) and the serial fold is not among them"
            ),
            Self::UndeclaredGrouping { strategy, detail } => write!(
                formatter,
                "the {strategy} publishes no contributor partition this oracle can be asked \
                 about: {detail}",
            ),
            Self::GroupingOracleUncalibrated {
                evaluator,
                partitioned,
            } => write!(
                formatter,
                "the reference evaluator returns {evaluator:08x?} for the whole program and the \
                 partitioned oracle returns {partitioned:08x?} at the declared serial order; \
                 either the orders disagree or the pointwise prologue is not bit-identity on \
                 these operands",
            ),
            Self::NoRefusableGrouping {
                strategy,
                permitted,
            } => write!(
                formatter,
                "every order-preserving regrouping of these operands produces {permitted:08x?}, \
                 so the {strategy}'s oracle has no wrong-but-permitted answer it could refuse and \
                 observes no rounding",
            ),
            Self::GroupingMismatch {
                strategy,
                partitions,
                contributors_per_partition,
                device,
                expected,
            } => write!(
                formatter,
                "the {strategy} declares {partitions} partition(s) of \
                 {contributors_per_partition} contributor(s) and returned {device:08x?}, and that \
                 grouping produces {expected:08x?}",
            ),
            Self::NonLiteralLaunch { position, node } => write!(
                formatter,
                "ABI arena position {position} is not a declared unsigned literal: {node}"
            ),
            Self::NoProgramOutput => {
                formatter.write_str("the alternative's program publishes no named output")
            }
            Self::UnboundOperand { bound, declared } => write!(
                formatter,
                "the route binds storage for {bound} program input(s) and the artifact declares \
                 {declared}; an unbound operand buffer is read as zeroes rather than refused",
            ),
            Self::RetainedDigestMismatch {
                member,
                case,
                executed,
                embedded,
                retained,
            } => write!(
                formatter,
                "{member} case {case:?}: the SHA-256 of the executed result bytes is {executed} \
                 and the retained realization-probe measurement is {retained}; the producer's \
                 published expectation hashes to {embedded}. This is a correctness finding about \
                 the contraction vertical, not a dispatch failure",
            ),
            Self::DigestHelper {
                message,
                observed,
                expected,
            } => write!(
                formatter,
                "this build's SHA-256 helper digests a {message}-byte published vector to \
                 {observed} and FIPS 180-4 publishes {expected}; no retained-value comparison in \
                 this run means anything until that is repaired",
            ),
            Self::DirectPathMultiInput { inputs } => write!(
                formatter,
                "the direct path binds one operand slice by local knowledge and this program \
                 declares {inputs} tensor input(s); route a multi-operand program through the \
                 envelope path, where the artifact declares which operand each binding takes",
            ),
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
            Self::UnexpectedRouteShape {
                member,
                expected_entries,
                entries,
                expected_shared,
                shared,
            } => write!(
                formatter,
                "{member} routed {entries} dispatch(es) over {shared} shared allocation(s), and \
                 its role means {expected_entries} over {expected_shared}",
            ),
            Self::ForeignProgram {
                packaged,
                alternatives,
            } => write!(
                formatter,
                "the artifact packages a kernel program whose {packaged}-byte identity matches \
                 none of the {alternatives} alternative(s) this process compiled for the \
                 artifact's own declared shape; the two prototypes have drifted",
            ),
            Self::ForeignRoutedProgram { routed, derived } => write!(
                formatter,
                "the route prepared a kernel program of {routed} identity bytes and this process \
                 derived one of {derived}; the routed program is not this build's",
            ),
            Self::UnboundBinding {
                entry,
                slot,
                target,
            } => write!(
                formatter,
                "entry {entry}'s ABI slot {slot} addresses {target}, which this proof binds no \
                 storage for",
            ),
            Self::BindingRangeOverflow {
                entry,
                slot,
                offset,
                extent,
            } => write!(
                formatter,
                "entry {entry}'s ABI slot {slot} starts at byte {offset} and reaches {extent} \
                 byte(s), which does not fit in a u64 allocation length",
            ),
            Self::EmptyLaunch { entry, skipped } => write!(
                formatter,
                "entry {entry}'s routed launch covers no threads (skipped: {skipped}), so there \
                 is no result to compare",
            ),
            Self::DevicePreflight(refusal) => write!(
                formatter,
                "this device refused the route before the commit: {refusal}",
            ),
            Self::ProbeAccepted(probe) => write!(
                formatter,
                "a probe was accepted rather than refused: {probe}, so that probe proves nothing",
            ),
            Self::LibraryLoad(cause) => write!(formatter, "the metallib did not load: {cause}"),
            Self::FunctionLookup(cause) => write!(formatter, "the entry point is absent: {cause}"),
            Self::Pipeline(cause) => write!(formatter, "no compute pipeline state: {cause}"),
            Self::Dispatch { status, detail } => write!(
                formatter,
                "the command buffer ended in {status}: {detail}, so nothing was read back",
            ),
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
        BACKEND_KEY, BINDING_APPLE_FAMILIES, CONTRACTION_ACTIVATIONS_KEY, CONTRACTION_CLASS,
        CONTRACTION_MEMBERS, CONTRACTION_OUTPUT_KEY, CONTRACTION_WEIGHTS_KEY, DeclaredInput,
        DeclaredInterface, DeviceFacts, L3_CELL_CLASS, L3_CELL_RESULT_SHA256,
        LiveDeviceObservation, LiveDeviceQualification, LoadRejection, METAL_MINIMUM_GPU_FAMILY,
        METAL_MINIMUM_GPU_FAMILY_VERSION, MetalGpuFamily, MetalGpuFamilySupport,
        MetalHostApplicabilityPolicy, PLAN_ROLES, Path, Placement, ProbeSubject, ProbedGpuFamily,
        ProofError, REDUCTION_CLASSES, REPRESENTATION_KEY, ROWS, RetainedComparison,
        RoutePreparation, RouteRequirement, RouteResourceDimension, SOLE_DELIVERY,
        bind_declared_interface, bind_interface, binding_apple_enumerator, compile_under,
        contraction_program, decide_live_device_requirement, declaration,
        declared_route_environment, evaluate_metal_host_applicability, expected_shape,
        normalized_architecture, observe_host_environment, probe_accepted_baseline,
        probe_damaged_interior_byte, probe_damaged_section_content,
        probe_foreign_expected_identity, probe_other_backend_family,
        probe_other_profile_descriptor, probe_other_profile_key, probe_truncated_envelope,
        proof_member, require_contraction_interface, require_serial_sum_interface, result_digest,
        serial_sum_program, sha256_hex, stating_probed_family,
    };
    use tiler_artifact::program::{
        AbiExprId, AbiFactBinder, AbiFacts, ArtifactExecutionPolicy, ArtifactProgramBuilder,
        AvailabilityPhase, BackendEntryKey, BackendEntryRef, BackendFeatureRequirement, BackendKey,
        BindingKind, BindingSpec, BindingTarget, BufferAccess, CapabilityKey,
        CompilationEnvironment, DeferredPredicateSpec, EntrySpec, FeasibilityRuleSetKey,
        FeasibilityRuleSetRef, LaunchSpec, PayloadContent, PayloadEntryMapping, PayloadMetadata,
        PayloadPlatform, PayloadProvenance, RecordedArtifactProgramIdentity, RepresentationKey,
        RouteFeatureKey, RouteRequirementSubject, RouteResourceFloor, SchemaVersion,
        SelectedProvider, TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
        ToolComponent, VariantSpec, VerifiedArtifactProgram,
    };
    use tiler_build::BoundMetalCompileDeclaration;
    use tiler_compiler::session::{Compilation, PlanAlternative};
    use tiler_ir::program::abi::ExprNode;
    use tiler_ir::program::{
        AbiExprId as ProgramAbiExprId, AllocationSpec, ByteWindow, DependencyReasonView,
        KernelProgramBuilder, MaterializedValueSpec, StageAccess, StageLaunch, StorageEncoding,
        StorageScalar, VerifiedKernelProgram,
    };
    use tiler_ir::semantic::SemanticProgram;
    use tiler_ir::shape::Shape;
    use tiler_metal::applicability::{
        MetalHostApplicabilityRefusal, MetalHostObservation, MetalHostPredicate,
    };
    use tiler_runtime::load::{DecodedProgram, ExecutionEnvironment};

    /// Columns of the **loader** fixtures' input; the reduced axis.
    ///
    /// **One, and now by choice.** It was one under duress: a `BackendEntryKey`
    /// was bounded at `MAX_OPAQUE_IDENTITY_BYTES` = 1,024 while the canonical
    /// kernel identity of a serial sum with two or more contributors measures
    /// 1,121 bytes, so an entry keyed on it did not construct.
    /// `bound-the-backend-entry-key-by-the-identity-it-carries` closed that by
    /// bounding the key at `tiler_ir::kernel::MAX_KERNEL_IDENTITY_BYTES`, and
    /// any reduced extent constructs now.
    ///
    /// It stays one because the cases that use it — the fail-closed probes, the
    /// multi-stage pairing, and the partial window — assert refusal classes,
    /// shared-allocation pairing, and byte offsets, none of which the reduced
    /// extent participates in, and each of them compiles a program. The extents
    /// the producer actually publishes are covered where they are load-bearing,
    /// by `the_published_shape_matrix_survives_this_builds_shape_handling`,
    /// which assembles at every one of them.
    const FIXTURE_COLUMNS: u64 = 1;

    /// Rows of every member `prototypes/serial-sum-compile` publishes.
    ///
    /// **One, and deliberately not [`ROWS`].** The producer packages one row so
    /// the materialized plan's pointwise stage stays inside the declared
    /// four-thread grid guarantee; the direct path reduces four.
    ///
    /// It lives under `#[cfg(test)]`, and that is the enforcement rather than a
    /// filing decision: the envelope path may take a shape from the artifact and
    /// from nowhere else, so a constant naming the producer's rows must be
    /// unreachable from [`super::prove_member`] by construction. What it is for
    /// is letting the gate assemble an envelope shaped like the ones the
    /// producer writes; `prototypes/serial-sum-compile` pins the same value in a
    /// test naming this side, exactly as [`super::SIDECAR_SUFFIX`] is pinned.
    ///
    /// The inequality with [`ROWS`] is load-bearing rather than incidental, and
    /// is asserted as such: it is what tells a runner that reads the artifact's
    /// declared shape apart from one that substituted its own row count.
    const PUBLISHED_ROWS: u64 = 1;

    /// First addressed byte of the partial-window fixture's scratch value.
    const PARTIAL_WINDOW_OFFSET: u64 = ROWS * FIXTURE_COLUMNS * super::F32_BYTES;

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
        expected: RecordedArtifactProgramIdentity,
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

    /// The authoritative declaration every fixture below compiles and routes under.
    fn declared() -> BoundMetalCompileDeclaration {
        declaration().expect("the authoritative declaration assembles")
    }

    /// The published contraction's extents, restated here as the runner's half
    /// of a pinned pair.
    ///
    /// `prototypes/serial-sum-compile` states the same three numbers and the
    /// same class name. Nothing links the two crates, so this pair is the only
    /// thing comparing them — the same arrangement [`SIDECAR_SUFFIX`] and the
    /// reduction matrix are under, and for the same reason: a producer that
    /// moved the published contraction while this half kept opening the old
    /// shape would leave a green gate over a member that cannot route.
    const FIXTURE_CONTRACTION: (u64, u64, u64) = (2, 2, 3);

    /// The runner's half of the published contraction interface, pinned.
    #[test]
    fn the_published_contraction_member_is_the_one_the_producer_writes() {
        assert_eq!(CONTRACTION_CLASS, "contraction");
        assert_eq!(FIXTURE_CONTRACTION, (2, 2, 3));
        assert_eq!(
            (
                CONTRACTION_ACTIVATIONS_KEY,
                CONTRACTION_WEIGHTS_KEY,
                CONTRACTION_OUTPUT_KEY,
            ),
            ("activations", "weights", "projected"),
        );
        assert_eq!(
            proof_member(
                std::path::Path::new("/tmp/a.tiler"),
                CONTRACTION_CLASS,
                "selected"
            )
            .display()
            .to_string(),
            "/tmp/a.tiler.contraction.selected",
        );
    }

    /// The runner's half of the published L3 cell, pinned.
    ///
    /// The same pair idiom, over the one member whose class name, extents, and
    /// retained digest must all three agree with the producer's: a cell renamed
    /// on one side leaves the other opening a file nobody writes, and a cell
    /// moved to other extents leaves this half comparing executed bytes against
    /// a measurement of a different program.
    const FIXTURE_L3_CELL: (u64, u64, u64) = (1, 1024, 1024);

    #[test]
    fn the_published_l3_cell_is_the_one_the_producer_writes() {
        assert_eq!(L3_CELL_CLASS, "contraction-w-decode-kv");
        assert_eq!(FIXTURE_L3_CELL, (1, 1024, 1024));
        assert_eq!(
            proof_member(
                std::path::Path::new("/tmp/a.tiler"),
                L3_CELL_CLASS,
                "selected"
            )
            .display()
            .to_string(),
            "/tmp/a.tiler.contraction-w-decode-kv.selected",
        );
        assert_eq!(
            L3_CELL_RESULT_SHA256,
            "79810ce471cbd6cd05e5c0c30ea6023e74b997bd5b349212b71cd4a23fe8701f",
            "the retained `w_decode_kv` `direct` digest, from this spike's own record: \
             spikes/scheduling/metal_contraction_vertical/results/\
             2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/workload.tsv",
        );
    }

    /// Exactly one routed member carries a retained measurement, and it is the
    /// L3 cell.
    ///
    /// **The negative half is the load-bearing one.** A `Some(...)` added to the
    /// adversarial member would compare its executed bytes against a digest no
    /// device ever produced for those operands, and that comparison would fail
    /// on hardware in a way that reads as a device defect. Pinned so the
    /// distinction between "measured elsewhere" and "not measured at all" cannot
    /// be lost by editing a table.
    #[test]
    fn only_the_l3_cell_is_compared_against_a_retained_measurement() {
        let compared: Vec<&str> = CONTRACTION_MEMBERS
            .iter()
            .filter(|member| member.retained_result_sha256.is_some())
            .map(|member| member.class)
            .collect();
        assert_eq!(compared, [L3_CELL_CLASS]);
        assert_eq!(
            CONTRACTION_MEMBERS.len(),
            2,
            "both published contraction members must be routed; a member the producer writes and \
             this half never opens is a file nobody reads",
        );
        assert_eq!(CONTRACTION_MEMBERS[0].class, CONTRACTION_CLASS);
    }

    /// The digest helper reproduces the published FIPS 180-4 vectors, and its
    /// domain is the probe's.
    ///
    /// **Both halves, because either alone is satisfiable by a wrong function.**
    /// The vectors say this is SHA-256. The domain case says the bytes it is fed
    /// are the ones the probe hashed — little-endian `f32`, row-major — which a
    /// correct SHA-256 over big-endian words would fail while passing every
    /// vector.
    #[test]
    fn the_digest_helper_reproduces_the_published_vectors() {
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        );
        // `1.0f32` is `0x3f800000`, whose little-endian bytes are
        // `00 00 80 3f`. Digesting the big-endian spelling instead would be a
        // different message, and this is the assertion that says which one.
        assert_eq!(
            result_digest(&[0x3f80_0000]),
            sha256_hex(&[0x00, 0x00, 0x80, 0x3f]),
        );
        assert_ne!(
            result_digest(&[0x3f80_0000]),
            sha256_hex(&[0x3f, 0x80, 0x00, 0x00]),
        );
        // Row-major order is element order: two results that differ only in the
        // order of their elements are different messages.
        assert_ne!(
            result_digest(&[0x3f80_0000, 0x4000_0000]),
            result_digest(&[0x4000_0000, 0x3f80_0000]),
        );
    }

    /// A retained comparison reports its two verdicts independently.
    ///
    /// The pairing is what makes a mismatch diagnosable, so it is asserted
    /// rather than left to the mismatch that would exercise it — which needs
    /// hardware and a defect at once.
    #[test]
    fn a_retained_comparison_separates_the_executed_bytes_from_the_published_record() {
        let matching = RetainedComparison {
            executed: L3_CELL_RESULT_SHA256.to_owned(),
            embedded: L3_CELL_RESULT_SHA256.to_owned(),
            retained: L3_CELL_RESULT_SHA256,
        };
        assert!(matching.executed_matches() && matching.embedded_matches());

        // The device disagreed with a record that asks the right question.
        let device_wrong = RetainedComparison {
            executed: sha256_hex(b"another result"),
            embedded: L3_CELL_RESULT_SHA256.to_owned(),
            retained: L3_CELL_RESULT_SHA256,
        };
        assert!(!device_wrong.executed_matches() && device_wrong.embedded_matches());

        // The record asks a different question, and the device answered it
        // faithfully — which is the case that must not read as a device defect.
        let record_wrong = RetainedComparison {
            executed: sha256_hex(b"another workload"),
            embedded: sha256_hex(b"another workload"),
            retained: L3_CELL_RESULT_SHA256,
        };
        assert!(!record_wrong.executed_matches() && !record_wrong.embedded_matches());
    }

    /// Builds one declared interface literal, for the family-recognition cases.
    fn interface_of(inputs: &[(&str, &[u64])], output: (&str, u64)) -> DeclaredInterface {
        DeclaredInterface {
            inputs: inputs
                .iter()
                .map(|(key, extents)| DeclaredInput {
                    key: (*key).to_owned(),
                    elements: extents.iter().product(),
                    extents: extents.to_vec(),
                })
                .collect(),
            output_key: output.0.to_owned(),
            output_elements: output.1,
            abi: AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight).build(),
        }
    }

    /// The contraction interface is recognized, and every way of not being one
    /// is refused.
    ///
    /// **The negatives are the point.** A recognizer that only ever saw the
    /// artifact its own producer writes would accept anything, and each row
    /// below is a way an interface could be wrong that would otherwise reach the
    /// device: a missing operand binds one buffer for a two-operand kernel, a
    /// contracted extent disagreement sizes one operand against the other's `K`,
    /// swapped keys write each operand into the other's buffer, and a wrong
    /// output count reads back the wrong number of elements.
    #[test]
    fn the_contraction_interface_is_recognized_and_every_miss_is_refused() {
        let good = interface_of(
            &[("activations", &[2, 3]), ("weights", &[2, 3])],
            ("projected", 4),
        );
        assert_eq!(
            require_contraction_interface(&good).expect("the published interface is recognized"),
            (2, 2, 3),
        );

        let misses: [(&str, DeclaredInterface); 5] = [
            (
                "one operand where the contraction declares two",
                interface_of(&[("activations", &[2, 3])], ("projected", 4)),
            ),
            (
                "operands that contract over different extents",
                interface_of(
                    &[("activations", &[2, 3]), ("weights", &[2, 5])],
                    ("projected", 4),
                ),
            ),
            (
                "operands under keys the contraction does not declare",
                interface_of(
                    &[("weights", &[2, 3]), ("activations", &[2, 3])],
                    ("projected", 4),
                ),
            ),
            (
                "an output element count that is not M times N",
                interface_of(
                    &[("activations", &[2, 3]), ("weights", &[2, 3])],
                    ("projected", 6),
                ),
            ),
            (
                "a rank-3 operand",
                interface_of(
                    &[("activations", &[2, 3, 1]), ("weights", &[2, 3])],
                    ("projected", 4),
                ),
            ),
        ];
        for (miss, interface) in misses {
            let refusal = require_contraction_interface(&interface)
                .expect_err(&format!("{miss} must be refused"));
            assert!(
                matches!(refusal, ProofError::Interface(_)),
                "{miss} was refused as {refusal} rather than as an interface disagreement",
            );
        }

        // And the serial sum's own interface is not mistaken for a contraction,
        // which is what keeps the two families' recognizers separate rather
        // than one accepting the other's artifacts.
        require_contraction_interface(&interface_of(&[("input", &[1, 3])], ("result", 1)))
            .expect_err("a one-input reduction is not a contraction");
        require_serial_sum_interface(&good).expect_err("a contraction is not a serial sum");
    }

    /// A two-operand route places each declared program input at its own ordinal.
    ///
    /// **This is the widening, checked without a device.** The binary proves the
    /// operands reach the GPU and return the reference's bits; this proves the
    /// step underneath it — that `plan_route` resolves *two distinct* ordinals
    /// from the artifact's own declared interface — in the ordinary gate, where
    /// it runs on every commit rather than by hand on hardware.
    ///
    /// The superseded spelling matched one key constant and produced a
    /// placement carrying no ordinal at all, so a two-operand route would have
    /// placed both slots identically. That defect is unreachable now, and this
    /// case is what keeps it unreachable.
    #[test]
    fn a_two_operand_route_places_each_declared_input_at_its_own_ordinal() {
        let (m, n, k) = FIXTURE_CONTRACTION;
        let semantic = contraction_program(m, n, k);
        let declaration = declared();
        let compilation =
            compile_under(&declaration, &semantic).expect("the declared contraction compiles");
        let alternative = compilation.selected().expect("a selected plan alternative");
        let artifact = assemble(&semantic, &compilation, alternative);
        let bytes = artifact.encode().expect("the contraction envelope encodes");
        let expected = recorded_identity(&artifact);
        let environment =
            declared_route_environment(&declaration).expect("the declared environment composes");

        let mut decoded = DecodedProgram::decode(&bytes, SOLE_DELIVERY)
            .expect("the assembled contraction envelope decodes");
        let interface = bind_declared_interface(&decoded).expect("the declared interface binds");
        assert_eq!(
            interface.inputs.len(),
            2,
            "the fixture must be a two-operand program or it checks nothing",
        );
        assert_eq!(
            require_contraction_interface(&interface).expect("the contraction is recognized"),
            (m, n, k),
        );

        let preflight = qualify_without_requirements(
            decoded
                .prepare(&environment, &expected, &interface.abi)
                .expect("the contraction route prepares"),
        )
        .resolve_target_properties(|_| u64::MAX)
        .expect("the contraction's target requirements hold");

        let placed =
            super::plan_route(&preflight, &interface).expect("the host places every routed slot");
        let ordinals: Vec<usize> = placed
            .iter()
            .flatten()
            .filter_map(|slot| match slot.placement {
                Placement::Input(ordinal) => Some(ordinal),
                Placement::Output | Placement::Internal => None,
            })
            .collect();
        assert_eq!(
            ordinals,
            vec![0, 1],
            "each declared operand must take its own ordinal; a repeated one would fill both \
             buffers from the same payload",
        );

        // The interface is genuinely consulted rather than recorded: routing the
        // same preflight against an interface that declares other input keys
        // leaves every program-input binding unresolvable, and the refusal names
        // the key the artifact actually declared.
        let foreign = interface_of(&[("input", &[m, k])], ("projected", m * n));
        let refusal = super::plan_route(&preflight, &foreign)
            .expect_err("an interface that declares other keys places no program input");
        assert!(
            matches!(refusal, ProofError::UnboundBinding { .. }),
            "the refusal must name the unplaceable binding: {refusal}",
        );
    }

    /// States a fixture artifact's derived identity as a recording.
    ///
    /// Every case below holds the very artifact it routes, so its recording is
    /// trivially correct — the tautology `DecodedProgram::preflight` documents,
    /// and the right shape for a fixture whose subject is the loader rather than
    /// the recording. [`probe_foreign_expected_identity`] is where a *wrong*
    /// recording is exercised, and the binary above is where a genuinely
    /// separate process supplies one.
    fn recorded_identity(artifact: &VerifiedArtifactProgram) -> RecordedArtifactProgramIdentity {
        RecordedArtifactProgramIdentity::from_bytes(artifact.canonical_identity().as_bytes())
            .expect("an encoder-derived identity is statable as a recording")
    }

    /// Compiles, packages, and encodes one valid envelope for the probes.
    ///
    /// The three facts a probe perturbs are each taken from the authority that
    /// owns it, exactly as the binary takes them: the expected identity from the
    /// artifact this function assembled, the routed environment from the
    /// authoritative declaration rather than from the artifact, and the ABI
    /// facts from the interface the *decoded* envelope declares. Reading any of
    /// them back out of the envelope would make the corresponding probe a
    /// tautology.
    ///
    /// The environment here is producer-declared equality, NOT host-earned
    /// eligibility — the same labelled diagnostic the binary routes under.
    fn fixture() -> Fixture {
        assembled_fixture(ROWS, FIXTURE_COLUMNS)
    }

    /// Compiles, packages, and encodes one valid envelope at an exact shape.
    ///
    /// Parameterized because the reduced extent is a *program*, not an operand
    /// set: it lives in the input shape, so it changes the semantic graph, the
    /// kernels, and the artifact identity. The published-shape cases need one
    /// envelope per class the producer publishes, and they must be the same
    /// assembly the loader cases route, or they would prove something about a
    /// second assembler instead.
    fn assembled_fixture(rows: u64, columns: u64) -> Fixture {
        let semantic = serial_sum_program(rows, columns);
        let declaration = declared();
        let compilation =
            compile_under(&declaration, &semantic).expect("the declared program compiles");
        let plan = compilation.selected().expect("a selected plan alternative");

        let artifact = assemble(&semantic, &compilation, plan);
        let bytes = artifact.encode().expect("the envelope encodes");
        let expected = recorded_identity(&artifact);
        let environment =
            declared_route_environment(&declaration).expect("the declared environment composes");

        let decoded =
            DecodedProgram::decode(&bytes, SOLE_DELIVERY).expect("the assembled envelope decodes");
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
        assemble_program(
            semantic,
            compilation,
            plan,
            plan.abi().kernel_program(),
            &[],
        )
    }

    /// Packages one explicit program under a compiled alternative's provenance.
    ///
    /// The ordinary fixture passes the alternative's own program. The
    /// partial-window fixture passes a checked reconstruction using the same
    /// kernels and ABI formulas but a larger scratch value viewed from a
    /// nonzero byte, because the compiler's bounded portfolio does not invent
    /// offset views merely to test the runtime that consumes them.
    #[allow(
        clippy::too_many_lines,
        reason = "one artifact is assembled top to bottom in the order the builder requires, and that order is the readable part"
    )]
    fn assemble_program(
        semantic: &SemanticProgram,
        compilation: &Compilation,
        plan: PlanAlternative<'_>,
        program: &VerifiedKernelProgram,
        route_requirements: &[RouteRequirement],
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

        // One mapping per stage, keyed on the same canonical kernel identity the
        // artifact's executable entry names, because the decoder proves the two
        // tables correlate and a mapping keyed on anything else is refused as an
        // unmapped backend entry.
        let mut mappings: Vec<PayloadEntryMapping> = program
            .stages()
            .enumerate()
            .map(|(position, stage)| PayloadEntryMapping {
                entry_key: BackendEntryKey::from_bytes(
                    stage.kernel().canonical_identity().as_bytes(),
                )
                .expect("the packaged kernel identity fits a backend entry key"),
                symbol: format!("tiler_probe_entry_{position}"),
                transports: (0..u32::try_from(stage.accesses().len())
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
                            // The probe compiles nothing, so it resolved against
                            // no SDK and requested no deployment minimum. Saying
                            // so is what ADR 0090 item 14's gap prevented.
                            platform: PayloadPlatform::Unversioned,
                            components: vec![ToolComponent {
                                role: "compiler".to_owned(),
                                version: "0".to_owned(),
                            }],
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

        // Still replayed, and still only for the pruning property it exercises:
        // the builder derives the variant's ABI from the program now, so nothing
        // below resolves a position. The builder deduplicates by content, so
        // this adds no node it does not already adopt.
        let minted = replay(
            &mut builder,
            program.abi_expressions(),
            &variant_roots(program),
        );
        debug_assert!(
            minted.iter().any(Option::is_some),
            "a non-empty root set must replay at least one node"
        );

        let entries: Vec<EntrySpec> = program
            .stages()
            .map(|stage| EntrySpec {
                // The accessible range, launch geometry, and applicability guard
                // are derived by `ArtifactProgramBuilder` from the program it is
                // given, so this consumer no longer restates them.
                bindings: stage
                    .accesses()
                    .map(|_| BindingSpec {
                        kind: BindingKind::Buffer,
                    })
                    .collect(),
                launch: LaunchSpec {
                    // Not a choice: `tiler_ir::schedule`'s intrinsic verifier
                    // refuses a scheduled region whose launch plan does not skip a
                    // zero-thread dispatch, so every verified region carries it.
                    zero_work_skips_dispatch: true,
                    preconditions: Vec::new(),
                },
                implementation: BackendEntryRef {
                    payloads: vec![payload],
                    entry_key: BackendEntryKey::from_bytes(
                        stage.kernel().canonical_identity().as_bytes(),
                    )
                    .expect("the packaged kernel identity fits a backend entry key"),
                },
            })
            .collect();

        let variant = builder
            .push_variant(
                program,
                VariantSpec {
                    target_profile: profile,
                    feasibility_rules: rules,
                    deferred_predicates: plan
                        .prepared_entry_target_requirements()
                        .map(|requirement| DeferredPredicateSpec {
                            requirement: requirement.requirement().clone(),
                            entry: requirement.entry(),
                        })
                        .collect(),
                    entries,
                },
            )
            .expect("the variant packages the plan it was built from");
        for requirement in route_requirements {
            builder
                .require_route(variant, requirement.clone())
                .expect("each declared route requirement names a distinct subject");
        }
        builder.build().expect("the assembled artifact verifies")
    }

    /// Passes a route with no live-device requirement through the qualification stage.
    ///
    /// The resolver panics rather than answering, so a fixture that grows a row
    /// says so instead of silently accepting whatever this closure returns. The
    /// stage is not skippable even when empty, and that is what these device-free
    /// cases are exercising as much as anything.
    fn qualify_without_requirements(
        qualification: LiveDeviceQualification<'_>,
    ) -> RoutePreparation<'_> {
        assert_eq!(
            qualification.live_device_requirements().len(),
            0,
            "this fixture declares no live-device route requirement",
        );
        qualification
            .resolve_live_device_requirements(|request| {
                panic!("an unexpected live-device requirement arrived: {request:?}")
            })
            .expect("a route requiring nothing of the device qualifies")
    }

    /// Builds a backend feature row naming a minimum Apple family.
    fn family_requirement(
        owner: &str,
        key: &str,
        version: u32,
        payload: &[u8],
    ) -> RouteRequirement {
        RouteRequirement::BackendFeature(
            BackendFeatureRequirement::new(
                BackendKey::new(owner).expect("a governed backend key"),
                RouteFeatureKey::new(key).expect("a governed route feature key"),
                version,
                payload,
            )
            .expect("a well-formed backend feature requirement"),
        )
    }

    /// Builds the well-formed Metal row this adapter owns, at one family.
    fn metal_family_requirement(family: MetalGpuFamily) -> RouteRequirement {
        family_requirement(
            BACKEND_KEY,
            METAL_MINIMUM_GPU_FAMILY,
            METAL_MINIMUM_GPU_FAMILY_VERSION,
            family.as_str().as_bytes(),
        )
    }

    /// Synthesizes the device facts the adapter decides from, without a device.
    fn observed_facts(probed: ProbedGpuFamily) -> DeviceFacts {
        DeviceFacts {
            name: "tiler.probe.device".to_owned(),
            max_threads_per_threadgroup: 1_024,
            // The value an Apple M4 Max reports; a synthesized fact, and the
            // adapter under test decides a family row rather than a capacity,
            // so nothing here reads it.
            max_threadgroup_memory_length: 32_768,
            max_buffer_length: 1 << 30,
            recommended_working_set: 1 << 30,
            apple_family: probed,
        }
    }

    /// One decoded fixture carrying the given live-device route requirements.
    struct RequiringFixture {
        bytes: Vec<u8>,
        expected: RecordedArtifactProgramIdentity,
        environment: ExecutionEnvironment,
        abi: AbiFacts,
    }

    /// Packages the ordinary fused plan with route requirements attached.
    fn requiring_fixture(requirements: &[RouteRequirement]) -> RequiringFixture {
        let semantic = serial_sum_program(ROWS, FIXTURE_COLUMNS);
        let declaration = declared();
        let compilation =
            compile_under(&declaration, &semantic).expect("the declared program compiles");
        let plan = compilation.selected().expect("a selected plan alternative");
        let artifact = assemble_program(
            &semantic,
            &compilation,
            plan,
            plan.abi().kernel_program(),
            requirements,
        );
        let bytes = artifact.encode().expect("the requiring envelope encodes");
        let expected = recorded_identity(&artifact);
        let environment =
            declared_route_environment(&declaration).expect("the declared environment composes");
        let decoded =
            DecodedProgram::decode(&bytes, SOLE_DELIVERY).expect("the requiring envelope decodes");
        let (_, _, abi) = bind_interface(&decoded).expect("the declared interface binds");
        RequiringFixture {
            bytes,
            expected,
            environment,
            abi,
        }
    }

    /// Returns the arena positions one variant names directly.
    fn variant_roots(program: &VerifiedKernelProgram) -> Vec<u32> {
        let mut roots = vec![program.applicability_guard()];
        for stage in program.stages() {
            roots.extend(stage.accesses().map(|access| access.accessible_bytes()));
            roots.push(stage.launch().grid_threads);
            roots.push(stage.launch().threads_per_workgroup);
        }
        roots
    }

    /// Rebuilds a checked materialized program with its scratch view shifted.
    ///
    /// Every semantic, kernel, ABI-expression, dependency, and lifecycle fact
    /// is copied from the compiler's verified materialized alternative. The one
    /// changed fact is storage: the temporary value and allocation are doubled,
    /// and every view of that value addresses the original working set in the
    /// upper half. `KernelProgramBuilder::build` re-verifies the result, so the
    /// fixture cannot manufacture an offset the bound kernels or program reject.
    #[allow(
        clippy::too_many_lines,
        reason = "the checked program is copied in dependency order so every owner-bound handle is visibly translated once"
    )]
    fn partial_window_program(
        semantic: &SemanticProgram,
        original: &VerifiedKernelProgram,
    ) -> VerifiedKernelProgram {
        let allocations: Vec<_> = original.allocations().collect();
        let values: Vec<_> = original.values().collect();
        let views: Vec<_> = original.views().collect();
        let stages: Vec<_> = original.stages().collect();
        let temporary = values
            .iter()
            .position(|value| value.role() == tiler_ir::program::ValueRole::Temporary)
            .expect("the materialized alternative carries one temporary");

        let mut builder =
            KernelProgramBuilder::new(semantic).expect("a program builder identity remains");
        let mut expressions: Vec<ProgramAbiExprId> =
            Vec::with_capacity(original.abi_expressions().len());
        let expression = |position: u32, minted: &[ProgramAbiExprId]| {
            minted[usize::try_from(position).expect("a bounded arena position fits a usize")]
        };
        for node in original.abi_expressions() {
            let minted = match node {
                ExprNode::Root(root) => builder.push_abi_root(root.clone()),
                ExprNode::Unary { op, operand } => {
                    builder.push_abi_unary(*op, expression(*operand, &expressions))
                }
                ExprNode::Binary { op, left, right } => builder.push_abi_binary(
                    *op,
                    expression(*left, &expressions),
                    expression(*right, &expressions),
                ),
                ExprNode::Select {
                    condition,
                    if_true,
                    if_false,
                } => builder.push_abi_select(
                    expression(*condition, &expressions),
                    expression(*if_true, &expressions),
                    expression(*if_false, &expressions),
                ),
            }
            .expect("a verified ABI arena replays");
            expressions.push(minted);
        }
        builder
            .applicability_guard(expression(original.applicability_guard(), &expressions))
            .expect("the verified guard replays");
        for transition in original.routing_commit_contract() {
            builder
                .push_routing_commit_transition(*transition)
                .expect("the verified routing lifecycle replays");
        }

        let allocation_ids: Vec<_> = allocations
            .iter()
            .map(|allocation| {
                let holds_temporary = allocation.values().any(|value| value == values[temporary]);
                builder
                    .push_allocation(AllocationSpec {
                        capacity_bytes: allocation.capacity_bytes()
                            + u64::from(holds_temporary) * PARTIAL_WINDOW_OFFSET,
                        alignment: allocation.alignment(),
                        memory_space: allocation.memory_space(),
                        ownership: allocation.ownership(),
                    })
                    .expect("the verified allocation replays")
            })
            .collect();

        let value_ids: Vec<_> = values
            .iter()
            .enumerate()
            .map(|(position, value)| {
                let allocation = allocations
                    .iter()
                    .position(|candidate| *candidate == value.allocation())
                    .expect("a value names a declared allocation");
                let shape = if position == temporary {
                    Shape::from_dims([ROWS * 2, FIXTURE_COLUMNS])
                } else {
                    value.shape().clone()
                };
                builder
                    .push_value(
                        MaterializedValueSpec {
                            origin: value.origin().clone(),
                            role: value.role(),
                            shape,
                            storage_scalar: StorageScalar::F32,
                            encoding: StorageEncoding::Unpacked,
                            element_type: value.element_type(),
                            alignment: value.alignment(),
                            memory_space: value.memory_space(),
                        },
                        allocation_ids[allocation],
                    )
                    .expect("the verified value replays")
            })
            .collect();

        let view_ids: Vec<_> = views
            .iter()
            .map(|view| {
                let value = values
                    .iter()
                    .position(|candidate| *candidate == view.value())
                    .expect("a view names a declared value");
                let window = if value == temporary {
                    ByteWindow {
                        offset: PARTIAL_WINDOW_OFFSET,
                        length: view.window().length,
                    }
                } else {
                    view.window()
                };
                builder
                    .push_view(value_ids[value], window)
                    .expect("the shifted view remains inside its enlarged value")
            })
            .collect();

        let view_position = |view: tiler_ir::program::ViewRef<'_>| {
            views
                .iter()
                .position(|candidate| {
                    candidate.value() == view.value() && candidate.window() == view.window()
                })
                .expect("an access names a declared view")
        };
        let stage_ids: Vec<_> = stages
            .iter()
            .map(|stage| {
                let accesses: Vec<_> = stage
                    .accesses()
                    .map(|access| StageAccess {
                        view: view_ids[view_position(access.view())],
                        mode: access.mode(),
                        accessible_bytes: expression(access.accessible_bytes(), &expressions),
                    })
                    .collect();
                let launch = stage.launch();
                builder
                    .push_stage(
                        stage.kernel(),
                        stage.coverage(),
                        &accesses,
                        StageLaunch {
                            grid_threads: expression(launch.grid_threads, &expressions),
                            threads_per_workgroup: expression(
                                launch.threads_per_workgroup,
                                &expressions,
                            ),
                        },
                    )
                    .expect("the shifted stage ABI still realizes the verified kernel")
            })
            .collect();

        for dependency in original.dependencies() {
            let predecessor = stages
                .iter()
                .position(|stage| *stage == dependency.predecessor())
                .expect("a dependency names a declared predecessor");
            let successor = stages
                .iter()
                .position(|stage| *stage == dependency.successor())
                .expect("a dependency names a declared successor");
            match dependency.reason() {
                DependencyReasonView::Data(value) => {
                    let value = values
                        .iter()
                        .position(|candidate| *candidate == value)
                        .expect("a dependency names a declared value");
                    builder
                        .push_data_dependency(
                            stage_ids[predecessor],
                            stage_ids[successor],
                            value_ids[value],
                        )
                        .expect("the data dependency replays");
                }
                DependencyReasonView::StorageHandoff(allocation) => {
                    let allocation = allocations
                        .iter()
                        .position(|candidate| *candidate == allocation)
                        .expect("a dependency names a declared allocation");
                    builder
                        .push_storage_handoff(
                            stage_ids[predecessor],
                            stage_ids[successor],
                            allocation_ids[allocation],
                        )
                        .expect("the storage handoff replays");
                }
            }
        }
        for output in original.outputs() {
            let value = values
                .iter()
                .position(|candidate| *candidate == output.value())
                .expect("an output names a declared value");
            builder
                .push_output(output.key().clone(), value_ids[value])
                .expect("the verified output replays");
        }
        builder.build().expect("the shifted program verifies")
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

    /// This half of the *member* filename interface, pinned.
    ///
    /// `prototypes/serial-sum-compile` derives the identical names and carries
    /// the identical assertion. The two crates share no code, so this pair of
    /// tests is the only thing that compares their idea of the matrix — both the
    /// names and which classes exist. A producer that adds a class the runner
    /// does not open, or renames one it does, fails here.
    #[test]
    fn the_member_names_are_the_ones_the_producer_writes() {
        let base = Path::new("/tmp/a.tiler");
        let names: Vec<String> = REDUCTION_CLASSES
            .iter()
            .flat_map(|(class, _)| {
                PLAN_ROLES
                    .iter()
                    .map(move |role| proof_member(base, class, role))
            })
            .map(|path| path.display().to_string())
            .collect();
        assert_eq!(
            names,
            [
                "/tmp/a.tiler.empty-domain.selected",
                "/tmp/a.tiler.empty-domain.materialized",
                "/tmp/a.tiler.singleton.selected",
                "/tmp/a.tiler.singleton.materialized",
                "/tmp/a.tiler.nontrivial.selected",
                "/tmp/a.tiler.nontrivial.materialized",
            ],
        );
    }

    /// Each role means a distinct dispatch shape, and that is the whole proof.
    ///
    /// If both roles expected the same shape, the matrix would compare a program
    /// against itself and report agreement, which is true and worthless. Pinned
    /// so a later edit cannot collapse them without saying so.
    #[test]
    fn the_two_roles_mean_different_dispatch_shapes() {
        assert_eq!(expected_shape("selected"), (1, 0));
        assert_eq!(expected_shape("materialized"), (2, 1));
        assert_ne!(
            expected_shape("selected"),
            expected_shape("materialized"),
            "a fused plan and a materialized plan agreeing is only evidence if \
             they ran differently",
        );
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

    /// This half of the published *shape* interface, pinned.
    ///
    /// `prototypes/serial-sum-compile` carries the identical assertion over its
    /// own `ROWS` and `REDUCTION_CLASSES`, for the reason the filename pins
    /// exist: the two crates share no code, so a value each states separately is
    /// compared by nothing else.
    ///
    /// **What this pin protects is the case below, not the proof.** A producer
    /// that published a different shape would still run — the envelope path
    /// reads every shape from the artifact — but the gate's published-shape
    /// fixture would quietly be assembling envelopes nobody publishes, which is
    /// the stale-fixture failure this module's own header warns about. Pinning
    /// the matrix on both sides is what makes that drift a red gate instead of a
    /// silently weaker check.
    ///
    /// The final assertion is the one that keeps the case below able to fail;
    /// see [`PUBLISHED_ROWS`].
    #[test]
    fn the_published_shape_matrix_is_the_one_the_producer_writes() {
        assert_eq!(
            PUBLISHED_ROWS, 1,
            "`prototypes/serial-sum-compile` publishes one row; a fixture assembled at any other \
             row count stands in for envelopes nobody writes",
        );
        assert_eq!(
            REDUCTION_CLASSES,
            [("empty-domain", 0), ("singleton", 1), ("nontrivial", 3)],
            "`prototypes/serial-sum-compile` states this same matrix; a class or reduced extent \
             changed there must change here too",
        );
        assert_ne!(
            PUBLISHED_ROWS, ROWS,
            "the published row count and this crate's own must differ, or substituting one for \
             the other becomes undetectable",
        );
    }

    /// Every shape the producer publishes survives this build's shape handling,
    /// and this build's own row count does not pass for it.
    ///
    /// **This is the case that would have caught the defect the ticket is
    /// about.** `prove_member` compiled `serial_sum_program(ROWS, columns)` from
    /// this crate's own four rows against artifacts published with one, so every
    /// packaged program was foreign and the whole matrix — six members, thirty
    /// operand cases — proved nothing for a month. Nothing saw it, because the
    /// matrix runs only against real published members on hardware.
    ///
    /// It runs [`compile_for_declared_shape`] and [`require_derived_program`],
    /// which are the two functions [`super::prove_member`] itself calls, over an
    /// envelope assembled at each published class. Every step is device-free and
    /// toolchain-free: the packaged identity comes from a `prepare`, which needs
    /// neither, so this holds wherever the workspace's tests do.
    ///
    /// **The refusal half is why the acceptance half means anything.** A shape
    /// handler that ignored the artifact and used [`ROWS`] would still compile,
    /// still route, and still produce a `Compilation` — it would simply be for
    /// another program. So the same check is run against exactly that
    /// substitution and required to refuse; without it, this case would pass
    /// against a handler that read nothing.
    ///
    /// [`compile_for_declared_shape`]: super::compile_for_declared_shape
    /// [`require_derived_program`]: super::require_derived_program
    #[test]
    fn the_published_shape_matrix_survives_this_builds_shape_handling() {
        let declaration = declared();
        let mut covered = 0_usize;
        for (class, extent) in REDUCTION_CLASSES {
            let fixture = assembled_fixture(PUBLISHED_ROWS, extent);

            let decoded = DecodedProgram::decode(&fixture.bytes, SOLE_DELIVERY)
                .expect("the published-shape envelope decodes");
            let (rows, columns, compilation) =
                super::compile_for_declared_shape(&declaration, &decoded)
                    .unwrap_or_else(|failure| panic!("{class}: {failure}"));
            assert_eq!(
                (rows, columns),
                (PUBLISHED_ROWS, extent),
                "{class}: the shape must be the artifact's own, not this build's",
            );

            // The packaged identity, read off the route rather than off the
            // compilation: taking it from the same `Compilation` the check
            // compares against would make the comparison a tautology.
            let mut routed = DecodedProgram::decode(&fixture.bytes, SOLE_DELIVERY)
                .expect("the published-shape envelope decodes again");
            let packaged = routed
                .prepare(&fixture.environment, &fixture.expected, &fixture.abi)
                .unwrap_or_else(|rejection| panic!("{class}: {rejection}"))
                .kernel_program_identity()
                .to_vec();

            // The shape is named as well as the class, because the whole point
            // of a refusal here is which shape was compiled: `ForeignProgram`
            // carries identity lengths deliberately, and a reader diagnosing a
            // drift needs the declared extents beside them.
            super::require_derived_program(&compilation, &packaged).unwrap_or_else(|failure| {
                panic!(
                    "{class}: the artifact declares {rows}x{columns} and this build compiled \
                     something else for it: {failure}"
                )
            });

            let substituted = compile_under(&declaration, &serial_sum_program(ROWS, columns))
                .expect("this crate's own row count compiles a program too");
            let refusal = super::require_derived_program(&substituted, &packaged)
                .expect_err("this build's own row count is not the published shape");
            assert!(
                matches!(refusal, super::ProofError::ForeignProgram { .. }),
                "{class}: a substituted shape must be reported as a foreign program: {refusal}",
            );
            covered += 1;
        }
        assert_eq!(
            covered,
            REDUCTION_CLASSES.len(),
            "every published reduction class is covered, not the ones that happened to run",
        );
    }

    /// The operand pair covers what each half alone cannot, counted on both
    /// sides.
    ///
    /// **The numbers are the point, and they are the reason two operand sets
    /// run rather than one.** Over four contributors there are five
    /// order-preserving groupings. On [`super::PARALLEL_OPERANDS`] all five
    /// produce one value, so a comparison against the serial fold has *nothing*
    /// it could refuse among legal answers — it cannot observe rounding, which
    /// is exactly what that constant's own documentation says and what this
    /// pins. On [`super::GROUPING_SENSITIVE_OPERANDS`] they produce two, so the
    /// declared-grouping oracle has a wrong-but-permitted answer to refuse.
    ///
    /// The converse count is asserted too, because the sensitive set is weaker
    /// where the exact one is strong: of the sixteen single-contributor
    /// corruptions of the declared grouping, the exact set leaves none
    /// undetected and the sensitive set leaves one. Neither half is a
    /// replacement for the other, and a later edit that dropped one would have
    /// to change these numbers to do it.
    #[test]
    fn the_operand_pair_covers_what_each_half_alone_cannot() {
        let exact = super::ordered_associations(&super::PARALLEL_OPERANDS);
        let sensitive = super::ordered_associations(&super::GROUPING_SENSITIVE_OPERANDS);
        assert_eq!(exact.len(), 5, "four contributors admit five orderings");
        assert_eq!(sensitive.len(), 5, "four contributors admit five orderings");

        let distinct = |mut values: Vec<u32>| {
            values.sort_unstable();
            values.dedup();
            values
        };
        assert_eq!(
            distinct(exact),
            vec![0x4170_0000],
            "every grouping of the exact operands is the same f32, so nothing legal is refusable",
        );
        assert_eq!(
            distinct(sensitive),
            vec![0x3f80_0000, 0x3f80_0001],
            "the sensitive operands must separate the declared groupings by exactly one rounding \
             step",
        );

        // The corruption counts, over the population this states: each slot
        // dropped, and each slot taking another slot's value. That is the
        // failure a partition boundary off by one or an unsynchronized staged
        // read produces, and it is the property the exact set holds and the
        // sensitive one does not.
        let escaped = |operands: [u32; 4]| {
            let declared = super::ContributorPartition {
                partitions: 2,
                contributors_per_partition: 2,
            };
            let correct = super::partitioned_reference(&operands, 1, 4, declared)
                .expect("the declared split is evaluable");
            let mut population = 0_usize;
            let mut escaped = 0_usize;
            for slot in 0..4 {
                for source in 0..5 {
                    let mut corrupt = operands;
                    // Source 4 is the dropped case: the contributor is replaced
                    // by the reduction's own identity element.
                    corrupt[slot] = if source == 4 {
                        0.0_f32.to_bits()
                    } else if source == slot {
                        continue;
                    } else {
                        operands[source]
                    };
                    population += 1;
                    let observed = super::partitioned_reference(&corrupt, 1, 4, declared)
                        .expect("a corrupted operand set is still evaluable");
                    if super::declared_grouping_admits(&correct, &observed) {
                        escaped += 1;
                    }
                }
            }
            (population, escaped)
        };
        assert_eq!(
            escaped(super::PARALLEL_OPERANDS),
            (16, 0),
            "the exact operands must leave no contributor corruption undetected",
        );
        assert_eq!(
            escaped(super::GROUPING_SENSITIVE_OPERANDS),
            (16, 1),
            "the sensitive operands leave exactly one corruption undetected, which is why the \
             exact set still runs",
        );
    }

    /// The grouping oracle refuses a legal regrouping the strategy did not
    /// declare.
    ///
    /// **This is the refusal the hardware run watches, carried into the gate.**
    /// The value refused is not garbage and not out of tolerance: it is the
    /// serial fold's answer, which a reassociation-permitting contract fully
    /// authorizes and which any bounded-error oracle would accept. What makes it
    /// wrong is only that the strategy under test published a different
    /// grouping, and that is the whole distinction a tolerance cannot draw.
    ///
    /// Both directions are asserted, so neither reading is an accident of which
    /// grouping happens to round up.
    #[test]
    fn the_grouping_oracle_refuses_a_legal_grouping_the_strategy_did_not_declare() {
        let operands = super::GROUPING_SENSITIVE_OPERANDS;
        let parallel = super::partitioned_reference(
            &operands,
            1,
            4,
            super::ContributorPartition {
                partitions: 2,
                contributors_per_partition: 2,
            },
        )
        .expect("the declared parallel split is evaluable");
        let serial = super::partitioned_reference(
            &operands,
            1,
            4,
            super::ContributorPartition {
                partitions: 4,
                contributors_per_partition: 1,
            },
        )
        .expect("the degenerate partition is the declared serial order");

        assert_eq!(parallel, vec![0x3f80_0001]);
        assert_eq!(serial, vec![0x3f80_0000]);
        assert!(
            super::declared_grouping_admits(&parallel, &parallel),
            "an oracle that refused the answer its own declared grouping produces would refuse \
             every correct strategy",
        );
        assert!(
            !super::declared_grouping_admits(&parallel, &serial),
            "the parallel oracle must refuse the serial fold's answer, which is legal under this \
             contract and is not what the parallel strategies declared",
        );
        assert!(
            !super::declared_grouping_admits(&serial, &parallel),
            "and the serial oracle must refuse the parallel answer, so neither direction is an \
             accident",
        );

        // The same refusal is unreachable on the exact operands, which is the
        // measured statement of why they cannot carry this claim.
        let exact_parallel = super::partitioned_reference(
            &super::PARALLEL_OPERANDS,
            1,
            4,
            super::ContributorPartition {
                partitions: 2,
                contributors_per_partition: 2,
            },
        )
        .expect("the declared parallel split is evaluable");
        let exact_serial = super::partitioned_reference(
            &super::PARALLEL_OPERANDS,
            1,
            4,
            super::ContributorPartition {
                partitions: 4,
                contributors_per_partition: 1,
            },
        )
        .expect("the degenerate partition is the declared serial order");
        assert!(
            super::declared_grouping_admits(&exact_parallel, &exact_serial),
            "on the exact operands the two groupings agree, so no refusal exists to watch",
        );
    }

    /// A partition that does not cover the contributor sequence is refused
    /// rather than rounded into one that does.
    #[test]
    fn a_partition_that_covers_nothing_is_refused_by_the_reference() {
        // Three partitions of two cover six, and this row has four. Both
        // strategies decline an inexact split rather than padding it, so the
        // oracle must decline to answer for one too.
        let refusal = super::partitioned_reference(
            &super::GROUPING_SENSITIVE_OPERANDS,
            1,
            4,
            super::ContributorPartition {
                partitions: 3,
                contributors_per_partition: 2,
            },
        )
        .expect_err("a split that does not cover the contributors has no exact value");
        assert!(
            matches!(refusal, ProofError::UndeclaredGrouping { .. }),
            "an inexact split must be refused as an undeclarable grouping: {refusal}",
        );
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

    /// Exactly one command-buffer status permits a readback, and no status
    /// permits a retry.
    ///
    /// All six variants the binding declares, which is the complete population
    /// rather than a sample — so this establishes the classification for every
    /// input that exists, not for the ones someone thought to list.
    ///
    /// The second assertion is the one the runtime execution contract cares
    /// about. Its transition table says "never" for every post-commit
    /// transition, and the way that is kept is structural: `SubmissionOutcome`
    /// has no retry variant, so no status can map to one. The test states the
    /// property the type already enforces, because a later edit that added such
    /// a variant would compile.
    #[test]
    fn one_status_permits_a_readback_and_none_permits_a_retry() {
        use metal::MTLCommandBufferStatus as Status;

        let population = [
            (Status::NotEnqueued, "NotEnqueued"),
            (Status::Enqueued, "Enqueued"),
            (Status::Committed, "Committed"),
            (Status::Scheduled, "Scheduled"),
            (Status::Completed, "Completed"),
            (Status::Error, "Error"),
        ];
        assert_eq!(
            population.len(),
            6,
            "the binding declares six statuses; a widened vocabulary belongs here too",
        );

        let mut readable = 0;
        for (status, name) in population {
            match super::submission_outcome(status) {
                super::SubmissionOutcome::Completed => {
                    readable += 1;
                    assert_eq!(name, "Completed", "{name} must not permit a readback");
                }
                super::SubmissionOutcome::ExecutionError => {
                    assert_eq!(name, "Error", "{name} is not the terminal error state");
                }
                // The status name is carried through rather than re-derived, so
                // a caller is told which non-terminal state the wait stopped in.
                super::SubmissionOutcome::NotTerminal(reported) => {
                    assert_eq!(
                        reported, name,
                        "the reported status is not the one observed"
                    );
                    assert!(
                        !matches!(name, "Completed" | "Error"),
                        "{name} is terminal and must not be reported as non-terminal",
                    );
                }
            }
        }
        assert_eq!(readable, 1, "exactly one status may be read back from");
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
                    entry: 0,
                    detail: "not a metallib".to_owned(),
                },
                super::PreflightPhase::Library,
                super::PreflightClass::CorruptArtifact,
            ),
            (
                super::PreflightRefusal::FunctionAbsent {
                    entry: 0,
                    symbol: "absent".to_owned(),
                    detail: "no such function".to_owned(),
                },
                super::PreflightPhase::Function,
                super::PreflightClass::CorruptArtifact,
            ),
            (
                super::PreflightRefusal::PipelineRejected {
                    entry: 1,
                    symbol: "k".to_owned(),
                    detail: "too many registers".to_owned(),
                },
                super::PreflightPhase::Pipeline,
                super::PreflightClass::RouteMiss,
            ),
            (
                super::PreflightRefusal::WorkgroupTooLarge {
                    entry: 1,
                    symbol: "k".to_owned(),
                    declared: 2,
                    capacity: 1,
                },
                super::PreflightPhase::LaunchGeometry,
                super::PreflightClass::RouteMiss,
            ),
            (
                super::PreflightRefusal::ThreadgroupMemoryExceeded {
                    entry: 1,
                    symbol: "k".to_owned(),
                    declared: 2,
                    capacity: 1,
                },
                super::PreflightPhase::Resources,
                super::PreflightClass::RouteMiss,
            ),
            (
                super::PreflightRefusal::BindingExceedsBufferLimit {
                    entry: 1,
                    slot: 0,
                    needed: 2,
                    limit: 1,
                },
                super::PreflightPhase::Resources,
                super::PreflightClass::RouteMiss,
            ),
            (
                super::PreflightRefusal::UndersizedAllocation {
                    entry: 0,
                    slot: 0,
                    needed: 2,
                    held: 1,
                },
                super::PreflightPhase::Resources,
                super::PreflightClass::Systemic,
            ),
            (
                super::PreflightRefusal::NoOutputBinding,
                super::PreflightPhase::Resources,
                super::PreflightClass::Systemic,
            ),
        ];
        assert_eq!(cases.len(), 8, "a refusal was added without a case here");
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

    /// The four comparisons refuse exactly at their boundary, not near it.
    ///
    /// Each is tested at the largest accepted value and the smallest refused
    /// one, because an off-by-one here either rejects a route the device would
    /// have run or admits one it cannot — and the second is the failure the
    /// whole stage exists to move before the commit.
    #[test]
    fn the_device_comparisons_refuse_exactly_at_their_boundary() {
        super::workgroup_fits(1, "k", 1024, 1024).expect("a workgroup at capacity fits");
        assert!(matches!(
            super::workgroup_fits(1, "k", 1025, 1024),
            Err(super::PreflightRefusal::WorkgroupTooLarge {
                entry: 1,
                declared: 1025,
                capacity: 1024,
                ..
            })
        ));

        // Zero is the ordinary case rather than an edge one: every non-cooperative
        // entry reserves no threadgroup memory, so a comparison that refused it
        // would refuse the serial fold on every device.
        super::local_memory_fits(1, "k", 0, 0).expect("an entry reserving nothing fits");
        super::local_memory_fits(1, "k", 32_768, 32_768)
            .expect("an entry at the device maximum fits");
        assert!(matches!(
            super::local_memory_fits(1, "k", 32_769, 32_768),
            Err(super::PreflightRefusal::ThreadgroupMemoryExceeded {
                entry: 1,
                declared: 32_769,
                capacity: 32_768,
                ..
            })
        ));

        super::binding_fits(1, 0, 4096, 4096).expect("a binding at the limit fits");
        assert!(matches!(
            super::binding_fits(1, 0, 4097, 4096),
            Err(super::PreflightRefusal::BindingExceedsBufferLimit {
                entry: 1,
                slot: 0,
                needed: 4097,
                limit: 4096,
            })
        ));

        super::allocation_fits(1, 0, 48, 48)
            .expect("an allocation of exactly the needed length fits");
        super::allocation_fits(1, 0, 48, 64).expect("a longer allocation fits");
        assert!(matches!(
            super::allocation_fits(1, 0, 48, 47),
            Err(super::PreflightRefusal::UndersizedAllocation {
                entry: 1,
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

    /// A multi-stage route preflights every entry and pairs its shared storage.
    ///
    /// This is the ticket's whole claim, and the single-stage fixture cannot
    /// make it: with one entry there is no execution order to get wrong and no
    /// intermediate to share. The materialized alternative dispatches two
    /// stages, so it is the shape that would have failed open.
    ///
    /// **The pairing is the assertion that matters.** An internal binding
    /// carries no name, so a loader allocating per binding hands the second
    /// stage a fresh buffer and it reads uninitialised device memory — plausible
    /// garbage rather than a refusal. Asserting only that two entries routed
    /// would pass with the data flow silently broken.
    #[test]
    fn a_multi_stage_route_preflights_every_entry_and_pairs_its_shared_storage() {
        let semantic = serial_sum_program(ROWS, FIXTURE_COLUMNS);
        let declaration = declared();
        let compilation =
            compile_under(&declaration, &semantic).expect("the declared program compiles");
        let materialized = compilation
            .alternatives()
            .find(|plan| !plan.is_fused())
            .expect("the materialized reference alternative is retained");
        assert!(
            materialized.kernels().len() > 1,
            "the materialized plan dispatches more than one stage",
        );

        let artifact = assemble(&semantic, &compilation, materialized);
        let bytes = artifact.encode().expect("the envelope encodes");
        let expected = recorded_identity(&artifact);
        let environment =
            declared_route_environment(&declaration).expect("the declared environment composes");
        let mut decoded = DecodedProgram::decode(&bytes, SOLE_DELIVERY)
            .expect("the multi-stage envelope decodes");
        let (_, _, abi) = bind_interface(&decoded).expect("the declared interface binds");

        assert!(
            matches!(
                decoded.preflight(&environment, &expected, &abi),
                Err(LoadRejection::UnansweredDeferredPredicates {
                    variant: 0,
                    deferred: 2,
                })
            ),
            "the device-free path remains fail-closed"
        );
        let mut decoded = DecodedProgram::decode(&bytes, SOLE_DELIVERY)
            .expect("the multi-stage envelope decodes again");
        let preparation = qualify_without_requirements(
            decoded
                .prepare(&environment, &expected, &abi)
                .expect("every entry of the multi-stage route prepares"),
        );
        let requests: Vec<_> = preparation.target_property_requests().collect();
        assert_eq!(
            requests.len(),
            2,
            "each prepared entry owns one exact query"
        );
        assert_eq!(
            requests[0].requirement().query().key(),
            requests[1].requirement().query().key(),
            "the fixture must exercise equal property keys on distinct entries",
        );
        let mut queried_entries: Vec<_> = requests.iter().map(|request| request.entry()).collect();
        queried_entries.sort_unstable();
        assert_eq!(
            queried_entries,
            vec![0, 1],
            "each query remains bound to its exact execution-order entry",
        );
        let refused_entry = requests[1].entry();
        let mut answer = 0;
        let rejection = preparation
            .resolve_target_properties(|_| {
                answer += 1;
                if answer == 1 { u64::MAX } else { 0 }
            })
            .expect_err("the second entry's insufficient answer must refuse independently");
        assert_eq!(answer, 2, "each exact-entry request is answered once");
        assert!(matches!(
            rejection,
            LoadRejection::UnsatisfiedDeferredPredicate {
                variant: 0,
                predicate: 1,
                entry,
            } if entry == refused_entry
        ));

        let mut decoded = DecodedProgram::decode(&bytes, SOLE_DELIVERY)
            .expect("the multi-stage envelope decodes again");
        let preflight = qualify_without_requirements(
            decoded
                .prepare(&environment, &expected, &abi)
                .expect("every entry of the multi-stage route prepares"),
        )
        .resolve_target_properties(|_| u64::MAX)
        .expect("both exact-entry requirements hold");

        assert_eq!(
            preflight.entries().len(),
            materialized.kernels().len(),
            "every stage is routed, not just the first",
        );

        // Exactly one intermediate flows between the two stages, so exactly one
        // pairing must be derived. Zero would mean the data flow was missed.
        let shared: Vec<_> = preflight.shared_allocations().to_vec();
        assert_eq!(
            shared.len(),
            1,
            "the one data dependency between these stages must pair one allocation",
        );
        let pair = shared[0];
        assert!(
            pair.producer().entry() < pair.consumer().entry(),
            "the producing entry precedes the consuming one in the execution order",
        );

        // Both ends address internal storage, and in opposite directions. That
        // is what makes the pair a data path rather than two unrelated slots.
        let producer = &preflight.entries()[pair.producer().entry()];
        let consumer = &preflight.entries()[pair.consumer().entry()];
        let slot_of = |entry: &tiler_runtime::load::RoutedEntry<'_>, slot: usize| {
            let binding = entry
                .bindings()
                .iter()
                .find(|binding| binding.slot() == slot)
                .expect("the pairing names a slot the entry declares");
            (
                matches!(binding.binding().target(), BindingTarget::Internal),
                binding.binding().access(),
            )
        };
        let (producer_internal, producer_access) = slot_of(producer, pair.producer().slot());
        let (consumer_internal, consumer_access) = slot_of(consumer, pair.consumer().slot());
        assert!(
            producer_internal && consumer_internal,
            "both ends of a shared allocation address entry-internal storage",
        );
        assert_eq!(
            producer_access,
            BufferAccess::Write,
            "the producing end writes the intermediate",
        );
        assert_eq!(
            consumer_access,
            BufferAccess::Read,
            "the consuming end reads it",
        );
    }

    /// A partial scratch window keeps its start byte through the runtime route.
    ///
    /// The fixture is necessarily two stages: the first writes the shared
    /// scratch value and the second reads it. Both bind the original working set
    /// in the upper half of an enlarged value, so publishing zero for either
    /// end would route successfully and silently connect the stages to the
    /// wrong bytes. The host plan additionally proves it sizes the allocation
    /// through the end of the window rather than allocating only its extent.
    #[test]
    fn a_partial_window_route_publishes_and_plans_the_artifact_offset() {
        let semantic = serial_sum_program(ROWS, FIXTURE_COLUMNS);
        let declaration = declared();
        let compilation =
            compile_under(&declaration, &semantic).expect("the declared program compiles");
        let materialized = compilation
            .alternatives()
            .find(|plan| !plan.is_fused())
            .expect("the materialized reference alternative is retained");
        let program = partial_window_program(&semantic, materialized.abi().kernel_program());
        let artifact = assemble_program(&semantic, &compilation, materialized, &program, &[]);
        let bytes = artifact
            .encode()
            .expect("the partial-window envelope encodes");
        let expected = recorded_identity(&artifact);
        let environment =
            declared_route_environment(&declaration).expect("the declared environment composes");
        let mut decoded = DecodedProgram::decode(&bytes, SOLE_DELIVERY)
            .expect("the partial-window envelope decodes");
        let interface = bind_declared_interface(&decoded).expect("the declared interface binds");
        let preflight = qualify_without_requirements(
            decoded
                .prepare(&environment, &expected, &interface.abi)
                .expect("the partial-window route prepares"),
        )
        .resolve_target_properties(|_| u64::MAX)
        .expect("the partial-window target requirement holds");

        let [shared] = preflight.shared_allocations() else {
            panic!("the two stages share exactly one scratch allocation");
        };
        for end in [shared.producer(), shared.consumer()] {
            let binding = preflight.entries()[end.entry()]
                .bindings()
                .iter()
                .find(|binding| binding.slot() == end.slot())
                .expect("the shared allocation names a routed binding");
            assert_eq!(
                binding.accessible_offset(),
                PARTIAL_WINDOW_OFFSET,
                "the runtime publishes the artifact's nonzero window start",
            );
            assert_eq!(
                binding.accessible_bytes(),
                PARTIAL_WINDOW_OFFSET,
                "the fixture addresses one original working set",
            );
        }

        let plan =
            super::plan_route(&preflight, &interface).expect("the host places every routed slot");
        for end in [shared.producer(), shared.consumer()] {
            let placed = plan[end.entry()][end.slot()];
            assert_eq!(placed.offset, PARTIAL_WINDOW_OFFSET);
            assert_eq!(
                placed.needed,
                PARTIAL_WINDOW_OFFSET * 2,
                "the allocation reaches through offset plus extent",
            );
        }
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

    /// Another profile key filters the variant and names the *family*.
    ///
    /// The half a descriptor-only probe cannot reach: a loader comparing
    /// descriptors and ignoring keys would pass the case below and admit an
    /// artifact built for a different target family entirely.
    #[test]
    fn another_profile_key_filters_every_variant() {
        let fixture = fixture();
        let outcome = probe_other_profile_key(&fixture.subject())
            .expect("another profile key leaves no eligible variant");
        assert!(
            outcome.contains("runtime.no-eligible-variant"),
            "the refusal names the no-eligible-variant class: {outcome}",
        );
        assert!(
            outcome.contains("ProfileKeyMismatch"),
            "the exclusion names a wrong artifact rather than a rebuild: {outcome}",
        );
    }

    /// Another profile descriptor filters the variant on its assessed profile.
    #[test]
    fn another_profile_descriptor_filters_every_variant() {
        let fixture = fixture();
        let outcome = probe_other_profile_descriptor(&fixture.subject())
            .expect("another profile descriptor leaves no eligible variant");
        assert!(
            outcome.contains("runtime.no-eligible-variant"),
            "the refusal names the no-eligible-variant class: {outcome}",
        );
        assert!(
            outcome.contains("DescriptorMismatch"),
            "the exclusion separates a rebuild from a wrong artifact: {outcome}",
        );
    }

    /// Another backend family filters on the representation, not the profile.
    ///
    /// The rendered exclusion has to name the declared pair, because the *class*
    /// is now the same one the two profile probes above produce. What separates
    /// "this host executes another family" from "this artifact is for another
    /// target" is the reason the refusal carries, so a probe asserting the class
    /// alone would no longer tell the three apart — which is exactly the failure
    /// mode this whole probe set exists to catch.
    #[test]
    fn another_backend_family_filters_on_the_representation() {
        let fixture = fixture();
        let outcome = probe_other_backend_family(&fixture.subject())
            .expect("another backend family leaves no eligible variant");
        assert!(
            outcome.contains("runtime.no-eligible-variant"),
            "the refusal names the no-eligible-variant class: {outcome}",
        );
        assert!(
            outcome.contains("is realized by a") && outcome.contains("this host states"),
            "the exclusion names the declared pair and the host's own: {outcome}",
        );
        assert!(
            !outcome.contains("Mismatch"),
            "a backend-family exclusion must not report a profile classification: {outcome}",
        );
    }

    /// Exactly one architecture spelling is rewritten, and nothing else is.
    ///
    /// The mapping exists because `std::env::consts::ARCH` and every retained
    /// record disagree on one name. A map that rewrote anything else would turn
    /// an unmeasured architecture into the measured one and hide the refusal the
    /// policy exists to produce.
    #[test]
    fn the_architecture_normalization_rewrites_one_spelling() {
        assert_eq!(normalized_architecture("aarch64"), "arm64");
        assert_eq!(normalized_architecture("arm64"), "arm64");
        for untouched in ["x86_64", "aarch64_be", "arm64e", "riscv64", ""] {
            assert_eq!(
                normalized_architecture(untouched),
                untouched,
                "only the `aarch64` spelling may be rewritten",
            );
        }
    }

    /// The device-free half answers its four predicates and invents no others.
    ///
    /// Runs on any macOS host: it asserts which predicates were *answered*, not
    /// what they say, because what they say is the very thing the policy is
    /// allowed to disagree with.
    #[test]
    fn the_device_free_observation_answers_only_the_device_free_predicates() {
        let observation = observe_host_environment();
        assert_eq!(observation.os_family(), Some(std::env::consts::OS));
        assert_eq!(
            observation.architecture(),
            Some(normalized_architecture(std::env::consts::ARCH)),
        );
        assert!(
            observation
                .os_version()
                .is_some_and(|value| !value.is_empty()),
            "sw_vers -productVersion answered nothing",
        );
        assert!(
            observation
                .os_build()
                .is_some_and(|value| !value.is_empty()),
            "sw_vers -buildVersion answered nothing",
        );
        assert_eq!(observation.device_name(), None);
        assert_eq!(observation.gpu_family(), None);
    }

    /// A device-free observation can never reach the translation-authority
    /// predicate, because two predicates before it are unanswered.
    #[test]
    fn a_device_free_observation_refuses_before_the_authority() {
        let refusal = evaluate_metal_host_applicability(
            MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9,
            &observe_host_environment(),
        )
        .expect_err("no observation earns a receipt");
        assert_ne!(
            refusal.predicate(),
            MetalHostPredicate::NativeTranslationAuthority,
            "an observation missing the device predicates must refuse on one of them",
        );
    }

    /// Composing both halves leaves no predicate unanswered.
    ///
    /// The device values are stated here rather than read from a device, so this
    /// runs without Metal. What it proves is about the *adapter*: the device-free
    /// half plus the two device fields covers every predicate the policy
    /// evaluates, so whatever refusal a real host gets is about the host and not
    /// about a field nobody filled in.
    #[test]
    fn the_composed_observation_answers_every_predicate() {
        let complete = observe_host_environment()
            .observing_device_name("Apple M4 Max")
            .observing_gpu_family(MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9));
        let refusal = evaluate_metal_host_applicability(
            MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9,
            &complete,
        )
        .expect_err("no observation earns a receipt");
        assert!(
            !matches!(refusal, MetalHostApplicabilityRefusal::Unobserved { .. }),
            "the adapter left a predicate unanswered: {refusal}",
        );
    }

    /// A route requiring a live-device fact is refused on the device-free path.
    ///
    /// The counterpart of the deferred-predicate refusal one phase earlier.
    /// `preflight` binds no device, so it can observe no device requirement, and
    /// routing past one would be assuming a fact nothing read.
    #[test]
    fn a_live_device_requirement_refuses_the_device_free_path() {
        let fixture = requiring_fixture(&[metal_family_requirement(MetalGpuFamily::Apple9)]);
        let mut decoded =
            DecodedProgram::decode(&fixture.bytes, SOLE_DELIVERY).expect("the envelope decodes");
        let rejection = decoded
            .preflight(&fixture.environment, &fixture.expected, &fixture.abi)
            .expect_err("a device-free path cannot observe a device");
        assert!(
            matches!(
                rejection,
                LoadRejection::UnansweredRouteRequirements {
                    variant: 0,
                    required: 1,
                },
            ),
            "expected an unanswered route-requirement refusal, got {rejection}",
        );
    }

    /// A row owned by another backend is refused before any adapter is asked.
    ///
    /// Decidable from the host's own declaration, so it needs neither a device
    /// nor an adapter — and asking an adapter about a namespace it does not own
    /// would be inviting it to answer for someone else.
    #[test]
    fn a_foreign_owner_is_refused_without_consulting_an_adapter() {
        let fixture = requiring_fixture(&[family_requirement(
            "tiler.cuda",
            "tiler.cuda.route-requirement.compute-capability",
            1,
            b"9.0",
        )]);
        let mut decoded =
            DecodedProgram::decode(&fixture.bytes, SOLE_DELIVERY).expect("the envelope decodes");
        let rejection = decoded
            .prepare(&fixture.environment, &fixture.expected, &fixture.abi)
            .err()
            .expect("a row owned by another backend cannot be decided here");
        let LoadRejection::ForeignRouteRequirementOwner {
            variant,
            position,
            owner,
            host_backend,
        } = &rejection
        else {
            panic!("expected a foreign-owner refusal, got {rejection}");
        };
        assert_eq!((*variant, *position), (0, 0));
        assert_eq!(owner, "tiler.cuda");
        assert_eq!(host_backend, BACKEND_KEY);
    }

    /// Every way one row can fail to decide is a refusal, and each is distinct.
    ///
    /// One fixture, four resolvers. Naming the population is what makes this a
    /// check rather than four assertions that happen to pass: the three refusal
    /// classes plus the satisfying case are exactly the outcomes
    /// `resolve_live_device_requirements` can produce for a well-formed row.
    #[test]
    fn each_undecidable_route_requirement_refuses_by_its_own_class() {
        let fixture = requiring_fixture(&[metal_family_requirement(MetalGpuFamily::Apple9)]);
        let subject = RouteRequirementSubject::BackendFeature {
            owner: BackendKey::new(BACKEND_KEY).expect("a governed backend key"),
            key: RouteFeatureKey::new(METAL_MINIMUM_GPU_FAMILY).expect("a governed key"),
            version: METAL_MINIMUM_GPU_FAMILY_VERSION,
        };

        let mut answered = 0;
        for (answer, expected) in [
            (
                LiveDeviceObservation::Unrecognized,
                LoadRejection::UnownedRouteRequirement {
                    variant: 0,
                    position: 0,
                    subject: subject.clone(),
                },
            ),
            (
                LiveDeviceObservation::Quantity(64),
                LoadRejection::MisansweredRouteRequirement {
                    variant: 0,
                    position: 0,
                    subject: subject.clone(),
                },
            ),
            (
                LiveDeviceObservation::Feature(false),
                LoadRejection::UnsatisfiedRouteRequirement {
                    variant: 0,
                    position: 0,
                    subject: subject.clone(),
                },
            ),
        ] {
            let mut decoded = DecodedProgram::decode(&fixture.bytes, SOLE_DELIVERY)
                .expect("the envelope decodes");
            let rejection = decoded
                .prepare(&fixture.environment, &fixture.expected, &fixture.abi)
                .expect("the route prepares")
                .resolve_live_device_requirements(|_| answer)
                .err()
                .expect("an undecided requirement refuses the route");
            assert_eq!(rejection, expected, "answer {answer:?} refused wrongly");
            assert!(
                rejection.to_string().contains(METAL_MINIMUM_GPU_FAMILY),
                "a refusal must name the exact unmet requirement: {rejection}",
            );
            answered += 1;
        }
        assert_eq!(answered, 3, "every refusal class was exercised");

        // The satisfying neighbour, without which the three above prove only
        // that this route refuses everything.
        let mut decoded =
            DecodedProgram::decode(&fixture.bytes, SOLE_DELIVERY).expect("the envelope decodes");
        let mut asked = 0;
        let _qualified = decoded
            .prepare(&fixture.environment, &fixture.expected, &fixture.abi)
            .expect("the route prepares")
            .resolve_live_device_requirements(|request| {
                asked += 1;
                assert_eq!(request.position(), 0);
                LiveDeviceObservation::Feature(true)
            })
            .expect("a satisfied requirement qualifies the route");
        assert_eq!(asked, 1, "each row is answered exactly once");
    }

    /// A route with no rows still passes through the stage, and asks nothing.
    #[test]
    fn a_route_requiring_nothing_qualifies_without_a_question() {
        let fixture = requiring_fixture(&[]);
        let mut decoded =
            DecodedProgram::decode(&fixture.bytes, SOLE_DELIVERY).expect("the envelope decodes");
        let qualification = decoded
            .prepare(&fixture.environment, &fixture.expected, &fixture.abi)
            .expect("the route prepares");
        assert_eq!(qualification.live_device_requirements().len(), 0);
        let _qualified = qualify_without_requirements(qualification);
    }

    /// The Metal adapter decides a family row from the family it observed.
    ///
    /// Device-free, because the adapter is split into an observation and this
    /// pure decision. Cumulative families are the property under test: a host
    /// reporting Apple9 satisfies an Apple8 requirement, and an Apple7 host does
    /// not satisfy an Apple9 one.
    #[test]
    fn the_metal_adapter_decides_a_family_row_from_the_family_it_observed() {
        let cases = [
            (
                MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9),
                MetalGpuFamily::Apple9,
                true,
            ),
            (
                MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple9),
                MetalGpuFamily::Apple8,
                true,
            ),
            (
                MetalGpuFamilySupport::Highest(MetalGpuFamily::Apple7),
                MetalGpuFamily::Apple9,
                false,
            ),
            (
                MetalGpuFamilySupport::NoneNamed,
                MetalGpuFamily::Apple5,
                false,
            ),
        ];
        for (observed, required, expected) in cases {
            let requirement = metal_family_requirement(required);
            let facts = observed_facts(ProbedGpuFamily::Answered(observed));
            let fixture = requiring_fixture(std::slice::from_ref(&requirement));
            let mut decoded = DecodedProgram::decode(&fixture.bytes, SOLE_DELIVERY)
                .expect("the envelope decodes");
            let qualification = decoded
                .prepare(&fixture.environment, &fixture.expected, &fixture.abi)
                .expect("the route prepares");
            let answers: Vec<_> = qualification
                .live_device_requirements()
                .map(|request| decide_live_device_requirement(&facts, request))
                .collect();
            assert_eq!(
                answers,
                vec![LiveDeviceObservation::Feature(expected)],
                "{observed:?} against {required:?}",
            );
        }
    }

    /// This binding names every family the governed vocabulary probes, counted.
    ///
    /// The compile-time assertion beside `binding_apple_enumerator` is the check
    /// that stops a build; this states the same population at runtime so the
    /// numbers a reader would otherwise have to take on trust are written down —
    /// how many families the vocabulary names, how many enumerators this binding
    /// names, and that the join between them is Apple's own value rather than a
    /// pairing.
    #[test]
    fn the_binding_names_every_family_the_governed_vocabulary_probes() {
        assert_eq!(
            MetalGpuFamily::COUNT,
            5,
            "the governed vocabulary's size is what this runner's probe population is",
        );
        assert_eq!(
            BINDING_APPLE_FAMILIES.len(),
            9,
            "`metal` 0.33.0 names Apple1 through Apple9",
        );
        let nameable = MetalGpuFamily::ALL
            .into_iter()
            .filter(|family| binding_apple_enumerator(family.apple_constant()).is_some())
            .count();
        assert_eq!(
            nameable,
            MetalGpuFamily::COUNT,
            "a family this binding cannot name leaves the GPU-family predicate unobserved",
        );
        for family in MetalGpuFamily::ALL {
            let enumerator = binding_apple_enumerator(family.apple_constant())
                .unwrap_or_else(|| panic!("this binding must name {family}"));
            assert_eq!(
                enumerator as isize,
                family.apple_constant().value(),
                "{family} must cross to the enumerator Apple declares at the same value",
            );
        }
    }

    /// The unnameable case is reachable rather than theoretical.
    ///
    /// Apple declares `MTLGPUFamilyApple10 = 1010` in the macOS 26.5 SDK and this
    /// binding stops at Apple9, so the moment the governed vocabulary widens, the
    /// probe meets an enumerator it cannot name. Pinned here because every
    /// refusal above it is only worth writing while that is true — a binding that
    /// gained the enumerator would make this fail and say so.
    #[test]
    fn this_binding_cannot_name_the_family_apple_declares_above_its_last() {
        assert!(
            !BINDING_APPLE_FAMILIES
                .iter()
                .any(|enumerator| *enumerator as isize == 1010),
            "`metal` 0.33.0 gained MTLGPUFamilyApple10; the vocabulary can now widen to it and \
             this runner's unnameable-enumerator refusal is no longer reachable through it",
        );
    }

    /// An enumerator this binding cannot name leaves the predicate unobserved.
    ///
    /// The pair is the point. The same observation, differing only in what the
    /// probe could learn, reaches ADR 0086's authority refusal when the device
    /// answered and stops at `Unobserved { predicate: GpuFamily }` when it was
    /// never asked — which is the policy's own word for an adapter that did not
    /// ask, and not the `GpuFamilyMismatch` a `false` answer would have produced.
    #[test]
    fn an_unnameable_enumerator_leaves_the_family_predicate_unobserved() {
        let policy = MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9;
        let measured = MetalHostObservation::unobserved()
            .observing_os_family(policy.os_family())
            .observing_os_version(policy.os_version())
            .observing_os_build(policy.os_build())
            .observing_architecture(policy.architecture())
            .observing_device_name(policy.device_name());

        let answered = stating_probed_family(
            measured.clone(),
            ProbedGpuFamily::Answered(MetalGpuFamilySupport::Highest(policy.gpu_family())),
        );
        assert_eq!(
            evaluate_metal_host_applicability(policy, &answered)
                .expect_err("no host earns a receipt")
                .predicate(),
            MetalHostPredicate::NativeTranslationAuthority,
            "an answered probe must carry the row past the GPU-family predicate",
        );

        let unnameable = stating_probed_family(
            measured,
            ProbedGpuFamily::Unnameable(MetalGpuFamily::Apple9.apple_constant()),
        );
        assert_eq!(
            evaluate_metal_host_applicability(policy, &unnameable)
                .expect_err("no host earns a receipt"),
            MetalHostApplicabilityRefusal::Unobserved {
                predicate: MetalHostPredicate::GpuFamily,
            },
            "a probe that could not ask must refuse as unobserved, not as a mismatch",
        );
    }

    /// A family row this adapter owns is `Unrecognized` when it could not ask.
    ///
    /// The route-requirement half of the same distinction. `Feature(false)` would
    /// refuse this route too, and would refuse it as a device that answered no —
    /// so a reader, and any future explain output, would read a binding gap as a
    /// hardware fact.
    #[test]
    fn a_family_row_is_unrecognized_when_the_binding_could_not_ask() {
        let facts = observed_facts(ProbedGpuFamily::Unnameable(
            MetalGpuFamily::Apple5.apple_constant(),
        ));
        let requirement = metal_family_requirement(MetalGpuFamily::Apple5);
        let fixture = requiring_fixture(std::slice::from_ref(&requirement));
        let mut decoded =
            DecodedProgram::decode(&fixture.bytes, SOLE_DELIVERY).expect("the envelope decodes");
        let qualification = decoded
            .prepare(&fixture.environment, &fixture.expected, &fixture.abi)
            .expect("the route prepares");
        let answers: Vec<_> = qualification
            .live_device_requirements()
            .map(|request| decide_live_device_requirement(&facts, request))
            .collect();
        assert_eq!(
            answers,
            vec![LiveDeviceObservation::Unrecognized],
            "the lowest family this vocabulary names is still unanswerable when nothing asked",
        );
    }

    /// Anything the Metal adapter does not own is `Unrecognized`, never a guess.
    ///
    /// The population is named: every row shape this adapter can be handed that
    /// it does not own — a foreign key, a version it predates, a payload naming
    /// no family — plus every neutral dimension it cannot observe.
    #[test]
    fn the_metal_adapter_refuses_every_row_it_does_not_own() {
        let facts = observed_facts(ProbedGpuFamily::Answered(MetalGpuFamilySupport::Highest(
            MetalGpuFamily::Apple9,
        )));
        let mut unowned: Vec<RouteRequirement> = vec![
            family_requirement(
                BACKEND_KEY,
                "tiler.metal.route-requirement.invented",
                1,
                b"apple9",
            ),
            family_requirement(
                BACKEND_KEY,
                METAL_MINIMUM_GPU_FAMILY,
                METAL_MINIMUM_GPU_FAMILY_VERSION + 1,
                b"apple9",
            ),
            family_requirement(
                BACKEND_KEY,
                METAL_MINIMUM_GPU_FAMILY,
                METAL_MINIMUM_GPU_FAMILY_VERSION,
                b"apple10",
            ),
        ];
        // Every neutral dimension, enumerated from the vocabulary rather than
        // listed here, so a dimension added to it lands in this check.
        unowned.extend(RouteResourceDimension::ALL.into_iter().map(|dimension| {
            RouteRequirement::ResourceFloor(
                RouteResourceFloor::new(dimension, 32).expect("a nonzero floor"),
            )
        }));
        assert_eq!(
            unowned.len(),
            3 + RouteResourceDimension::ALL.len(),
            "the enumerated population is the one this case claims to cover",
        );

        for requirement in &unowned {
            let fixture = requiring_fixture(std::slice::from_ref(requirement));
            let mut decoded = DecodedProgram::decode(&fixture.bytes, SOLE_DELIVERY)
                .expect("the envelope decodes");
            let qualification = decoded
                .prepare(&fixture.environment, &fixture.expected, &fixture.abi)
                .expect("the route prepares");
            let answers: Vec<_> = qualification
                .live_device_requirements()
                .map(|request| decide_live_device_requirement(&facts, request))
                .collect();
            assert_eq!(
                answers,
                vec![LiveDeviceObservation::Unrecognized],
                "{requirement:?} must not be decided by an adapter that does not own it",
            );
        }
    }
}

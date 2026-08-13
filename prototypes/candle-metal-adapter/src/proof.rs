//! The Candle adapter proof, carried end to end onto real hardware.
//!
//! # What this establishes
//!
//! A Candle user builds an ordinary `Tensor` on a Metal device and gets it
//! reduced by a **Tiler artifact** running as a Candle custom op. The artifact is
//! a file `prototypes/serial-sum-compile` published; nothing this process
//! compiles reaches the device, and the expected values are the ones the
//! producer's own reference evaluation recorded in the envelope's sidecar. An
//! agreement is therefore Candle storage, a Tiler artifact, and an independent
//! oracle arriving at the same bits.
//!
//! # Two authority questions, and only one of them is claimed
//!
//! **"Is this host eligible to offer the declared profile?"** is asked by
//! [`offer_the_declared_profile`] from a host observation and nothing else, and
//! the answer is always no: [ADR 0086](../../../docs/decisions/0086-require-attributable-or-attested-native-translation.md)
//! decides that native device translation of a metallib during pipeline creation
//! is a typed capability fact whose authority is `Unknown` on every macOS row
//! currently observable. That refusal is printed before any routing commit.
//!
//! **"Does this artifact name the profile the producer declared?"** is what the
//! adapter's bound environment answers, from `tiler-build`'s own declaration. It
//! is **producer-declared equality, NOT host-earned eligibility**, exactly as in
//! `prototypes/serial-sum-run`, and this binary says so in those words wherever a
//! reader could mistake a green route for an eligibility claim.
//!
//! # Usage
//!
//! ```text
//! cargo run -p tiler-prototype-compile -- --out /tmp/serial-sum.tiler
//! cargo run -p tiler-prototype-candle  -- --artifact /tmp/serial-sum.tiler
//! ```
//!
//! The base path is required and a missing member is a hard failure: a run that
//! quietly skipped a member because a file was absent would report success for
//! part of a proof.

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use candle_core::{DType, Device, Tensor, Var};

use objc2_metal::{MTLCommandBuffer, MTLCommandQueue, MTLDevice};

use tiler_artifact::program::{
    BackendKey, RecordedArtifactIdentityError, RecordedArtifactProgramIdentity, RepresentationKey,
    TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef,
};
use tiler_artifact::proof::{
    DecodedProofSidecar, ProofAssociationError, ProofCodecError, decode_proof_sidecar,
};
use tiler_build::{BoundMetalCompileDeclaration, BoundMetalDeclarationError, DTypeDispatchability};
use tiler_metal::applicability::{
    MetalGpuFamilySupport, MetalHostApplicabilityPolicy, MetalHostApplicabilityRefusal,
    MetalHostObservation, evaluate_metal_host_applicability,
};
use tiler_metal_aot::driver::Toolchain;
use tiler_metal_aot::input::{CompileRequest, OptimizationLevel};
use tiler_runtime::load::{DTypeDispatch, ExecutionEnvironment};

use crate::adapter::{
    PreparedPipeline, SubmissionOutcome, argument_slots_agree, load_library,
    prepare_pipeline_with_reflection, submission_outcome,
};
use crate::refusal::{Realization, RouteRefusal, TensorRefusal};
use crate::wrapper::{TilerPlan, WrapperError, candle_expression};

/// Governed backend family key this host executes.
const BACKEND_KEY: &str = "tiler.metal";
/// Governed executable-representation key this host consumes.
const REPRESENTATION_KEY: &str = "metallib";
/// Suffix the producer appends to an envelope path to name its proof sidecar.
///
/// `prototypes/serial-sum-compile` writes this name and nothing links the two
/// crates, so it is pinned here in the same idiom `prototypes/serial-sum-run`
/// uses: a producer that writes one name while a consumer opens another leaves a
/// green gate over a slice that cannot run.
const SIDECAR_SUFFIX: &str = ".proof";
/// Byte width of one `f32`.
const F32_BYTES: u64 = 4;

/// The published members this proof routes, as `(reduction class, plan role)`.
///
/// Mirrors the producer's own matrix. `selected` is the fused plan — one
/// dispatch, no intermediate — and `materialized` computes the same function as
/// two dispatches through one shared allocation, which is what separates "both
/// agreed" from "both ran the same program twice".
const MEMBERS: [(&str, &str); 6] = [
    // The empty domain leads, because it is the boundary the other two cannot
    // speak for: a reduction over zero contributors reads its input buffer
    // never, and its result is a reduction's identity element rather than a sum.
    // It is refused by name at this Candle pin rather than routed, and it stays
    // in this population for that reason: a member dropped from the matrix would
    // report the boundary as untested rather than as refused.
    ("empty-domain", "selected"),
    ("empty-domain", "materialized"),
    ("nontrivial", "selected"),
    ("nontrivial", "materialized"),
    ("singleton", "selected"),
    ("singleton", "materialized"),
];

/// The symbol every hand-written probe object publishes.
const PROBE_SYMBOL: &str = "tiler_probe_kernel";

/// The objects this proof compiles outside the emitter, and what each must do.
///
/// Hand-written MSL rather than a carried payload, because the object is the
/// side that cannot be perturbed: the envelope proves an integrity digest over
/// the bytes, so an edited object is refused as a damaged envelope long before a
/// pipeline exists. `tiler-metal`'s emitter cannot produce any of these either —
/// it emits `[[buffer(N)]]` parameters plus launch builtins and refuses the
/// workgroup address space outright — which is exactly why an object addressing
/// one of these resources needs writing by hand to be watched refused.
///
/// The tuple is `(the object, whether preparing it must be refused, its
/// source)`. Every kernel publishes [`PROBE_SYMBOL`] and takes the same
/// `[[buffer(0)]]` output, so the only thing that varies between rows is the
/// resource class under test.
const PROBE_OBJECTS: [(&str, bool, &str); 3] = [
    // The accepted neighbour, and it carries a measurement of its own: a
    // threadgroup allocation declared *inside* the kernel body is not an
    // argument, so it is not a reflected binding row, and refusing threadgroup
    // rows below therefore refuses the dynamically sized ones an encoder must
    // set a length for — not workgroup memory as such. Without this row, the
    // refusals below would be indistinguishable from a check that refuses every
    // object this proof compiles.
    (
        "an object declaring a threadgroup allocation inside the kernel body",
        false,
        "#include <metal_stdlib>\n\
         using namespace metal;\n\
         kernel void tiler_probe_kernel(\n\
             device float *out [[buffer(0)]],\n\
             uint gid [[thread_position_in_grid]]\n\
         ) {\n\
             threadgroup float scratch[4];\n\
             scratch[gid % 4] = float(gid);\n\
             threadgroup_barrier(mem_flags::mem_threadgroup);\n\
             out[gid] = scratch[0];\n\
         }\n",
    ),
    // The ticket's own case. Two undeclarable classes at index 0 beside a buffer
    // at index 0, because that is the shape the gap has: the buffer half can
    // agree exactly, and the encoder would bind it, dispatch, and leave the
    // texture and the sampler unbound.
    (
        "an object addressing a texture and a sampler",
        true,
        "#include <metal_stdlib>\n\
         using namespace metal;\n\
         kernel void tiler_probe_kernel(\n\
             device float *out [[buffer(0)]],\n\
             texture2d<float, access::sample> image [[texture(0)]],\n\
             sampler taps [[sampler(0)]],\n\
             uint gid [[thread_position_in_grid]]\n\
         ) {\n\
             out[gid] = image.sample(taps, float2(0.0f, 0.0f)).x;\n\
         }\n",
    ),
    // The threadgroup half, decided rather than omitted: this length is set by
    // `setThreadgroupMemoryLength:atIndex:` at encode time, the artifact ABI has
    // no way to state it, and this adapter never calls it — so the kernel would
    // address a zero-length allocation.
    (
        "an object taking a threadgroup memory argument",
        true,
        "#include <metal_stdlib>\n\
         using namespace metal;\n\
         kernel void tiler_probe_kernel(\n\
             device float *out [[buffer(0)]],\n\
             threadgroup float *scratch [[threadgroup(0)]],\n\
             uint gid [[thread_position_in_grid]]\n\
         ) {\n\
             scratch[0] = float(gid);\n\
             threadgroup_barrier(mem_flags::mem_threadgroup);\n\
             out[gid] = scratch[0];\n\
         }\n",
    ),
];

/// The reduction class this proof runs its Tensor-level probes against.
///
/// The nontrivial member, because three contributors per row is what makes a
/// serial reduction's ordering observable and therefore what makes the
/// realization claim mean something.
const PROBE_MEMBER: (&str, &str) = ("nontrivial", "selected");

/// Runs the proof and reports.
pub fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("candle adapter proof failed: {failure}");
            ExitCode::FAILURE
        }
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "the proof is one linear narrative from a Candle tensor through a Tiler artifact to compared bits; splitting it would hide the ordering that is its point"
)]
fn run() -> Result<(), ProofError> {
    let base = artifact_path()?;
    let declaration =
        BoundMetalCompileDeclaration::first_macos_apple9().map_err(ProofError::Declaration)?;
    let environment = declared_route_environment(&declaration)?;
    println!(
        "routed environment (PRODUCER-DECLARED EQUALITY, NOT HOST-EARNED ELIGIBILITY): {} / {} / {}",
        environment.target_profile.key.as_str(),
        environment.backend.as_str(),
        environment.representation.as_str(),
    );

    let device = Device::new_metal(0).map_err(|cause| ProofError::Device(cause.to_string()))?;
    let Device::Metal(metal) = &device else {
        return Err(ProofError::Device(
            "Candle returned a device that is not Metal".to_owned(),
        ));
    };
    println!(
        "candle metal device: {} (registry {:#x}, candle context {:?})",
        metal.metal_device().as_ref().name(),
        metal.registry_id(),
        metal.id(),
    );

    // Asked before anything routes, because a refusal after a commit would be a
    // fallback ADR 0051 does not permit — and printed rather than gating, which
    // is the same separation `prototypes/serial-sum-run` records.
    let refusal = offer_the_declared_profile(metal);
    println!(
        "host applicability (ADR 0086): REFUSED — predicate {}, rule {}\n  {refusal}",
        refusal.predicate().as_str(),
        refusal.rule(),
    );

    // The device-level fail-closed probes, before the positive route is claimed.
    probe_device_refusals(metal, &declaration)?;

    let mut proved = 0_usize;
    let mut routed = 0_usize;
    let mut refused: Vec<String> = Vec::new();
    for (class, role) in MEMBERS {
        let member = format!("{class}.{role}");
        match prove_member(&device, &environment, &base, class, role)? {
            MemberOutcome::Proved(cases) => {
                proved += cases;
                routed += 1;
            }
            MemberOutcome::Refused => refused.push(member),
        }
    }

    // The Tensor-level boundary, against the member whose ordering is
    // observable. Every probe perturbs one fact and leaves the rest alone, so a
    // refusal is evidence about that fact: the same plan, device, and operands
    // routed moments earlier.
    probe_tensor_refusals(&device, metal, &environment, &base)?;

    // One population, resolved member by member. The refused members are counted
    // and named here rather than subtracted out: an excluded count reads as a
    // matrix that is smaller than the one the producer published, and the whole
    // point of the empty domain is that it is a member with an outcome.
    println!(
        "candle adapter proof: {} of {} published member(s) resolved — {routed} routed and agreed \
         with the producer's recorded reference evaluation across {proved} case(s), {} refused by \
         a typed preflight refusal naming a zero extent ({})",
        routed + refused.len(),
        MEMBERS.len(),
        refused.len(),
        if refused.is_empty() {
            "none".to_owned()
        } else {
            refused.join(", ")
        },
    );
    Ok(())
}

/// What one published member's route established.
enum MemberOutcome {
    /// Every case the sidecar carries agreed with the producer's reference.
    Proved(usize),
    /// This member's declared interface was refused by name, before any storage.
    ///
    /// A typed refusal rather than a skipped member: a run that quietly omitted
    /// a published member would report success for part of a proof, and one that
    /// let Candle's allocator report the limitation would put another project's
    /// error on a boundary this adapter owns.
    ///
    /// Carries nothing, because the refusal is reported by [`prove_member`] where
    /// it arrives — beside that member's own report lines, exactly as a routed
    /// member's are. What the caller does with this is count it.
    Refused,
}

/// Routes one published member through Candle and compares against the sidecar.
fn prove_member(
    device: &Device,
    environment: &ExecutionEnvironment,
    base: &Path,
    class: &str,
    role: &str,
) -> Result<MemberOutcome, ProofError> {
    let path = proof_member(base, class, role);
    let (bytes, sidecar) = read_artifact(&path)?;
    let recorded = RecordedArtifactProgramIdentity::from_bytes(sidecar.artifact_identity_bytes())
        .map_err(ProofError::RecordedIdentity)?;
    let loaded = TilerPlan::load(
        bytes,
        recorded,
        environment.clone(),
        Realization::TilerFlushSubnormalsToZeroF32StrictOrder,
    );
    // **The empty-domain close path.** A declared empty axis is refused from the
    // artifact's own interface, before any Candle tensor is asked for, so the
    // caller reads this adapter's typed refusal instead of an allocator error
    // from under it. It is the one load refusal this proof reports as a member
    // outcome; every other one is a defect in the run.
    let plan = match loaded {
        Ok(plan) => plan,
        Err(WrapperError::Tensor(TensorRefusal::ZeroExtentInterface {
            value,
            axis,
            extents,
        })) => {
            // Rendered through the refusal's own `Display` rather than restated
            // here, so a refusal whose wording or fields changed shows up in this
            // line rather than only in a type.
            println!(
                "  {class}.{role}: REFUSED before any Candle storage is asked for — {}",
                TensorRefusal::ZeroExtentInterface {
                    value,
                    axis,
                    extents: extents.clone(),
                },
            );
            zero_extent_stays_unbuildable(device, &extents)?;
            return Ok(MemberOutcome::Refused);
        }
        Err(other) => return Err(ProofError::Wrapper(other)),
    };
    let (rows, columns) = plan.declared_shape();
    if class == PROBE_MEMBER.0 && role == PROBE_MEMBER.1 {
        println!(
            "  requested realization: {} (order-fixing: {})",
            plan.realization(),
            plan.realization().fixes_reduction_order(),
        );
    }

    let mut proved = 0_usize;
    for case in sidecar.cases() {
        let input_bits = case
            .inputs()
            .next()
            .ok_or(ProofError::SidecarWithoutCases)
            .and_then(|payload| decode_f32_bits("input", rows * columns, payload.bytes()))?;
        let expected = case
            .expected()
            .next()
            .ok_or(ProofError::SidecarWithoutCases)
            .and_then(|payload| decode_f32_bits("expected", rows, payload.bytes()))?;

        let input = tensor_from_bits(&input_bits, rows, columns, device)?;
        let applied = plan.apply(&input, device).map_err(ProofError::Wrapper)?;
        let observed = read_bits(&applied.tensor)?;
        if observed != expected {
            return Err(ProofError::Mismatch {
                member: format!("{class}.{role}"),
                observed,
                expected,
            });
        }

        let report = applied
            .report
            .as_ref()
            .ok_or(ProofError::FallbackTakenUnexpectedly)?;
        // Reported once per member rather than once per case. Every case routes
        // afresh and the shape it routes under is asserted every time; printing
        // the identical line twenty times would bury the four lines that differ.
        if proved == 0 {
            println!(
                "  {class}.{role}: {rows}x{columns} under profile {}, {}/{} entr(y/ies) encoded, \
                 {} shared allocation(s)",
                report.profile_key, report.encoded, report.entries, report.shared_allocations,
            );
            println!("    delivered: {}", applied.delivered);
            println!(
                "    device: {} (max buffer {} byte(s), highest named Apple family {})",
                report.facts.name,
                report.facts.max_buffer_length,
                match report.facts.highest_apple_family {
                    MetalGpuFamilySupport::Highest(family) => family.as_str(),
                    MetalGpuFamilySupport::NoneNamed => "no named Apple family",
                },
            );
        }
        proved += 1;
    }

    if proved == 0 {
        return Err(ProofError::SidecarWithoutCases);
    }
    println!("    {proved} case(s) agreed with the producer's recorded reference evaluation");
    Ok(MemberOutcome::Proved(proved))
}

/// Records that no Candle tensor of a refused shape exists at this pin.
///
/// The refusal itself is decided from the artifact alone and would stand whatever
/// Candle did, so this is what keeps it from being merely conservative: the
/// measurement that says the refused shape is genuinely unbuildable here, taken
/// after the refusal rather than instead of it.
///
/// **Fact — Candle's Metal allocator refuses a zero-length buffer.** It sizes a
/// request as `element_count * dtype.size_in_bytes()` and
/// `newBufferWithLength:options:` returns nil at length zero, which Candle
/// reports as a failed resource creation.
///
/// It is also the ticket's first activation trigger, watched rather than
/// re-derived by a reader: a Candle whose allocator admits a zero-length
/// allocation builds this tensor, and this run then fails instead of going on
/// reporting a member as refused that has become routable.
fn zero_extent_stays_unbuildable(device: &Device, extents: &[u64]) -> Result<(), ProofError> {
    let dims: Vec<usize> = extents
        .iter()
        .map(|extent| usize::try_from(*extent).unwrap_or(usize::MAX))
        .collect();
    let shape = candle_core::Shape::from_dims(&dims);
    match Tensor::from_vec(Vec::<f32>::new(), shape, device) {
        Err(cause) => {
            println!("    and Candle still builds no {dims:?} tensor of its own: {cause}");
            Ok(())
        }
        Ok(_) => Err(ProofError::ProbeAccepted(
            "a zero-element Metal tensor, which the refusal above records as unbuildable",
        )),
    }
}

/// Proves each device-side refusal arrives under the class it must.
///
/// Every probe changes one fact and leaves the rest alone, and each is paired
/// with a neighbour that is accepted: the two payload probes against the real
/// object [`run`] routes moments later, and each object in
/// [`probe_undeclarable_resources`] against the one compiled beside it that
/// prepares.
fn probe_device_refusals(
    metal: &candle_core::MetalDevice,
    declaration: &BoundMetalCompileDeclaration,
) -> Result<(), ProofError> {
    // Bytes that are not a metallib. The envelope digest matched for the real
    // object, so this is content that will not execute rather than an integrity
    // failure — the distinction the payload refusal exists to carry.
    let refusal = load_library(metal, 0, b"tiler probe object; not an executable image")
        .err()
        .ok_or(ProofError::ProbeAccepted(
            "a library from bytes that are not a metallib",
        ))?;
    println!("  probe: {refusal}");

    // A command buffer that was never committed must not classify as terminal,
    // or a readback would be taken from work that never ran.
    let queue = metal
        .metal_device()
        .new_command_queue()
        .map_err(|cause| ProofError::Device(cause.to_string()))?;
    let uncommitted = queue
        .commandBuffer()
        .ok_or_else(|| ProofError::Device("no command buffer".to_owned()))?;
    match submission_outcome(uncommitted.status()) {
        SubmissionOutcome::NotTerminal(status) => {
            println!(
                "  probe: a live command buffer that was never committed is {status}, no readback taken"
            );
        }
        SubmissionOutcome::Completed | SubmissionOutcome::ExecutionError => {
            return Err(ProofError::ProbeAccepted(
                "an uncommitted command buffer as a terminal state",
            ));
        }
    }

    probe_undeclarable_resources(metal, declaration)
}

/// Proves an object addressing a resource the ABI cannot declare is refused.
///
/// The half of ADR 0090 item 8's third obligation that no artifact can exhibit.
/// Each object is compiled from the source [`PROBE_OBJECTS`] carries, through the
/// same offline driver and the same authoritative target the producer compiles
/// with, and then loaded and prepared through the exact functions a route takes —
/// so a refusal here is about the resource class and not about a second code path
/// written to resemble the first.
///
/// A refusal is required to arrive under the undeclarable-resource class
/// specifically. An object refused because its pipeline would not build is not
/// evidence about this check, and reporting it as one is how a check that never
/// ran reads as a check that said no.
fn probe_undeclarable_resources(
    metal: &candle_core::MetalDevice,
    declaration: &BoundMetalCompileDeclaration,
) -> Result<(), ProofError> {
    for (object, must_refuse, source) in PROBE_OBJECTS {
        let request = CompileRequest::new(
            source,
            declaration.aot_target(),
            OptimizationLevel::Default,
            declaration.numerical_realization(),
        );
        // A toolchain that will not compile the probe leaves the *measurement*
        // unmade rather than the run failed, and says exactly what would make it:
        // the refusal itself is unit-tested against a classified table, and what
        // is missing without this is the evidence that Metal's reflection on this
        // row reports the class at all.
        let compiled = match Toolchain::system().compile(&request) {
            Ok(compiled) => compiled,
            Err(cause) => {
                println!(
                    "  probe {object}: NOT MEASURED — the offline Metal toolchain did not \
                     produce it ({cause}). The exact procedure is `xcrun --sdk {} metal {} \
                     <probe>.metal -o <probe>.air` then `xcrun --sdk {} metallib <probe>.air -o \
                     <probe>.metallib`, over the source this proof carries for that row.",
                    request.target.sdk().selector(),
                    request.compile_flags().join(" "),
                    request.target.sdk().selector(),
                );
                continue;
            }
        };

        match prepared_probe_pipeline(metal, &compiled.metallib) {
            Ok(_) if must_refuse => return Err(ProofError::ProbeAccepted(object)),
            Ok(prepared) => println!(
                "  probe {object}: prepared, addressing buffer argument(s) {:?} and no row the \
                 artifact ABI cannot declare",
                prepared.addressed_slots,
            ),
            Err(refusal) if must_refuse => {
                if !matches!(refusal, RouteRefusal::UndeclarableBindings { .. }) {
                    return Err(ProofError::ProbeMisclassified {
                        probe: object,
                        refusal: refusal.to_string(),
                    });
                }
                println!("  probe {object}: {refusal}");
            }
            Err(refusal) => return Err(ProofError::BaselineRefused(refusal.to_string())),
        }
    }
    Ok(())
}

/// Loads and prepares one hand-written probe object exactly as a route would.
fn prepared_probe_pipeline(
    device: &candle_core::MetalDevice,
    object: &[u8],
) -> Result<PreparedPipeline, RouteRefusal> {
    let library = load_library(device, 0, object)?;
    let function = library.get_function(PROBE_SYMBOL, None).map_err(|cause| {
        RouteRefusal::EntrySymbolAbsent {
            entry: 0,
            symbol: PROBE_SYMBOL.to_owned(),
            detail: cause.to_string(),
        }
    })?;
    prepare_pipeline_with_reflection(device, 0, PROBE_SYMBOL, &function)
}

/// Proves each Tensor-level refusal arrives by name, and that the fallback fails closed.
fn probe_tensor_refusals(
    device: &Device,
    metal: &candle_core::MetalDevice,
    environment: &ExecutionEnvironment,
    base: &Path,
) -> Result<(), ProofError> {
    let (class, role) = PROBE_MEMBER;
    let path = proof_member(base, class, role);
    let (bytes, sidecar) = read_artifact(&path)?;
    let recorded = RecordedArtifactProgramIdentity::from_bytes(sidecar.artifact_identity_bytes())
        .map_err(ProofError::RecordedIdentity)?;
    let plan = TilerPlan::load(
        bytes,
        recorded,
        environment.clone(),
        Realization::TilerFlushSubnormalsToZeroF32StrictOrder,
    )
    .map_err(ProofError::Wrapper)?;
    let (rows, columns) = plan.declared_shape();
    let case = sidecar
        .cases()
        .next()
        .ok_or(ProofError::SidecarWithoutCases)?;
    let input_bits = case
        .inputs()
        .next()
        .ok_or(ProofError::SidecarWithoutCases)
        .and_then(|payload| decode_f32_bits("input", rows * columns, payload.bytes()))?;
    let accepted = tensor_from_bits(&input_bits, rows, columns, device)?;

    // The neighbour every probe below is paired against. Without it a refusal
    // could be evidence about the plan rather than about the perturbation.
    plan.preflight(&accepted, device)
        .map_err(|refusal| ProofError::BaselineRefused(refusal.to_string()))?;
    println!("  probe baseline: the unperturbed tensor preflights");

    let rows_usize = usize::try_from(rows).unwrap_or(usize::MAX);
    let columns_usize = usize::try_from(columns).unwrap_or(usize::MAX);
    let values: Vec<f32> = input_bits.iter().copied().map(f32::from_bits).collect();

    let cpu = Tensor::from_vec(values.clone(), (rows_usize, columns_usize), &Device::Cpu)
        .map_err(|cause| ProofError::Device(cause.to_string()))?;
    expect_refusal("a tensor on the host device", plan.preflight(&cpu, device))?;

    let wrong_dtype = accepted
        .to_dtype(DType::F16)
        .map_err(|cause| ProofError::Device(cause.to_string()))?;
    expect_refusal("an f16 tensor", plan.preflight(&wrong_dtype, device))?;

    // A genuinely affine-strided view. It has to be built at more than one row,
    // and the reason is worth recording: Candle's `Layout::is_contiguous`
    // ignores the stride of any extent-1 axis, so transposing this artifact's
    // declared 1-by-N input produces an N-by-1 view Candle still calls
    // contiguous — a transposed view is therefore *not* a reliable
    // affine-stride perturbation at this shape, and one built by narrowing the
    // inner axis of a wider two-row tensor is.
    let strided = Tensor::zeros((2, columns_usize + 1), DType::F32, device)
        .and_then(|wide| wide.narrow(1, 0, columns_usize))
        .map_err(|cause| ProofError::Device(cause.to_string()))?;
    if strided.layout().is_contiguous() {
        return Err(ProofError::ProbeAccepted(
            "a narrowed inner axis that turned out to be contiguous",
        ));
    }
    expect_refusal("an affine-strided view", plan.preflight(&strided, device))?;

    let transposed = accepted
        .t()
        .map_err(|cause| ProofError::Device(cause.to_string()))?;
    expect_refusal("a transposed view", plan.preflight(&transposed, device))?;

    let broadcast = Tensor::from_vec(vec![1.0_f32], (1, 1), device)
        .and_then(|seed| seed.broadcast_as((rows_usize, columns_usize)))
        .map_err(|cause| ProofError::Device(cause.to_string()))?;
    expect_refusal("a broadcast view", plan.preflight(&broadcast, device))?;

    let flattened = accepted
        .flatten_all()
        .map_err(|cause| ProofError::Device(cause.to_string()))?;
    expect_refusal("a rank-1 tensor", plan.preflight(&flattened, device))?;

    let wider = Tensor::zeros((rows_usize, columns_usize + 1), DType::F32, device)
        .map_err(|cause| ProofError::Device(cause.to_string()))?;
    expect_refusal("a tensor one column wider", plan.preflight(&wider, device))?;

    let tracked =
        Var::from_tensor(&accepted).map_err(|cause| ProofError::Device(cause.to_string()))?;
    expect_refusal(
        "an autograd-tracked tensor",
        plan.preflight(tracked.as_tensor(), device),
    )?;

    // A contiguous view at a nonzero start offset must be **accepted**, and
    // produce the same bits. This is the half a refusal-only probe set would
    // leave open: the adapter must apply the offset rather than refuse it, and
    // must not bind offset zero merely because it holds the buffer.
    let padded_rows = rows_usize + 1;
    let mut padded = vec![0.0_f32; columns_usize];
    padded.extend_from_slice(&values);
    let offset_view = Tensor::from_vec(padded, (padded_rows, columns_usize), device)
        .and_then(|whole| whole.narrow(0, 1, rows_usize))
        .map_err(|cause| ProofError::Device(cause.to_string()))?;
    if offset_view.layout().start_offset() == 0 {
        return Err(ProofError::ProbeAccepted(
            "a narrowed view that turned out to start at offset zero",
        ));
    }
    let applied = plan
        .apply(&offset_view, device)
        .map_err(ProofError::Wrapper)?;
    let observed = read_bits(&applied.tensor)?;
    let expected = case
        .expected()
        .next()
        .ok_or(ProofError::SidecarWithoutCases)
        .and_then(|payload| decode_f32_bits("expected", rows, payload.bytes()))?;
    if observed != expected {
        return Err(ProofError::Mismatch {
            member: "nonzero start offset".to_owned(),
            observed,
            expected,
        });
    }
    println!(
        "  probe: a contiguous view starting at element {} is accepted and agrees",
        offset_view.layout().start_offset(),
    );

    // A tensor on a *second* Candle Metal device over the same GPU. Candle mints
    // a fresh `DeviceId` per `Device::new_metal`, so the two share a registry
    // identifier and share no allocator, queue, or residency set — which is
    // exactly the pair `ForeignMetalDevice` refuses.
    let second = Device::new_metal(0).map_err(|cause| ProofError::Device(cause.to_string()))?;
    let elsewhere = tensor_from_bits(&input_bits, rows, columns, &second)?;
    expect_refusal(
        "a tensor on another Candle Metal device",
        plan.preflight(&elsewhere, device),
    )?;

    // The fail-closed rule, exercised end to end rather than only unit-tested.
    // A refused preflight would take the ordinary Candle expression if that
    // expression realized the requested contract; this one does not, so the
    // wrapper must name the unmet realization instead of running the faster,
    // differently rounded path.
    let closed = plan
        .apply(&cpu, device)
        .err()
        .ok_or(ProofError::ProbeAccepted(
            "a host-device tensor, which must fail closed rather than fall back",
        ))?;
    if !matches!(
        closed,
        WrapperError::Tensor(TensorRefusal::NoRealizableFallback { .. })
    ) {
        return Err(ProofError::ProbeAccepted(
            "a refusal that was not the fail-closed realization refusal",
        ));
    }
    println!("  probe a refused tensor with no realizable fallback: {closed}");

    // A symbol the real published object does not publish, through the same
    // loader the route takes. A missing declared symbol is an artifact
    // invariant rather than an applicability miss, and the class says so.
    let absent = plan
        .probe_entry_symbol(metal, "tiler_kernel_this_object_does_not_publish")
        .err()
        .ok_or(ProofError::ProbeAccepted("an absent entry symbol"))?;
    println!("  probe an entry symbol the object does not publish: {absent}");

    // ADR 0090 item 8's third obligation, against the real published object's
    // real argument table. The baseline is the agreement itself: the artifact's
    // declared transport slots and the compiled function's reflected buffer
    // arguments must already be the same table, or every perturbation below
    // would be evidence about a route that was broken to begin with.
    let observed = plan.probe_argument_slots(metal).map_err(|cause| {
        ProofError::BaselineRefused(format!("the argument table could not be read: {cause}"))
    })?;
    argument_slots_agree(0, &observed.symbol, &observed.declared, &observed.addressed)
        .map_err(|refusal| ProofError::BaselineRefused(refusal.to_string()))?;
    println!(
        "  probe baseline: entry 0's {:?} addresses buffer argument(s) {:?} and the artifact \
         declares transport slot(s) {:?} — the same table, read from this object's own reflection",
        observed.symbol, observed.addressed, observed.declared,
    );

    // Each perturbation moves the *declaration* away from the object's own
    // table by one fact and leaves everything else alone, because the object is
    // the side that cannot be perturbed: the envelope proves an integrity digest
    // over the bytes, so an edited transport mapping is refused as a damaged
    // envelope — the probe above — rather than reaching this comparison.
    for (perturbation, declared) in [
        ("one slot the object does not address", {
            let mut wider = observed.declared.clone();
            wider.push(observed.addressed.iter().copied().max().unwrap_or(0) + 1);
            wider
        }),
        ("one declared slot dropped", {
            let mut narrower = observed.declared.clone();
            narrower.pop();
            narrower
        }),
        ("the same count at a renumbered slot", {
            let mut renumbered = observed.declared.clone();
            if let Some(last) = renumbered.last_mut() {
                *last += 1;
            }
            renumbered
        }),
    ] {
        // A perturbation that did not actually change the declaration would be
        // asserted as a refusal and prove nothing; a one-binding entry cannot
        // exhibit every shape, so this states which ones it did exhibit.
        if declared == observed.declared {
            println!(
                "  probe an argument table with {perturbation}: NOT EXHIBITABLE — this entry \
                 declares {} slot(s), and the perturbation leaves the table unchanged",
                observed.declared.len(),
            );
            continue;
        }
        let refusal = argument_slots_agree(0, &observed.symbol, &declared, &observed.addressed)
            .err()
            .ok_or(ProofError::ProbeAccepted(
                "an argument table that disagrees with the entry's declaration",
            ))?;
        println!("  probe an argument table with {perturbation}: {refusal}");
    }

    // A host offering another exact profile descriptor. Refused before Candle's
    // custom-op path is entered, which is what "target availability" means here.
    let (bytes, _) = read_artifact(&path)?;
    let mut foreign = environment.clone();
    foreign.target_profile.descriptor =
        TargetProfileDescriptorDigest::from_bytes(b"a descriptor this host does not offer")
            .map_err(|_| ProofError::HostProfile)?;
    let refused_profile = TilerPlan::load(
        bytes.clone(),
        RecordedArtifactProgramIdentity::from_bytes(sidecar.artifact_identity_bytes())
            .map_err(ProofError::RecordedIdentity)?,
        foreign,
        Realization::TilerFlushSubnormalsToZeroF32StrictOrder,
    )
    .err()
    .ok_or(ProofError::ProbeAccepted("a foreign profile descriptor"))?;
    println!("  probe a foreign profile descriptor: {refused_profile}");

    // A flipped byte inside the envelope. These bytes decode to nothing this
    // wrapper may run, and the class is the artifact layer's own integrity
    // classification rather than a variant that failed to apply.
    let mut damaged = bytes;
    let midpoint = damaged.len() / 2;
    damaged[midpoint] ^= 0xff;
    let refused_bytes = TilerPlan::load(
        damaged,
        RecordedArtifactProgramIdentity::from_bytes(sidecar.artifact_identity_bytes())
            .map_err(ProofError::RecordedIdentity)?,
        environment.clone(),
        Realization::TilerFlushSubnormalsToZeroF32StrictOrder,
    )
    .err()
    .ok_or(ProofError::ProbeAccepted("a damaged envelope"))?;
    println!("  probe a damaged envelope: {refused_bytes}");

    // The labelled numerical-scope comparison. Not a defect in either project:
    // a Tiler kernel and a Candle kernel differ by compiler build, math mode,
    // and — for a reduction — summation order, and nothing reconciles them.
    let candle = candle_expression(&accepted).map_err(ProofError::Device)?;
    let candle_bits = read_bits(&candle)?;
    println!(
        "  numerical scope: the ordinary Candle expression delivers {candle_bits:08x?} and this \
         artifact delivers {expected:08x?} ({}); the two are separate compilations under separate \
         math modes and separate reduction orders, so an agreement here is an observation about \
         these operands and not a guarantee, and Tiler's claim covers only the artifact's own \
         kernels",
        if candle_bits == expected {
            "identical on these operands"
        } else {
            "different on these operands"
        },
    );
    Ok(())
}

/// Requires one perturbation to be refused, and reports the class it produced.
fn expect_refusal(
    probe: &'static str,
    outcome: Result<(), TensorRefusal>,
) -> Result<(), ProofError> {
    let refusal = outcome.err().ok_or(ProofError::ProbeAccepted(probe))?;
    // Reported through the wrapper's own classification rather than restated
    // here, so a refusal whose fallback status changed would show up in this
    // line rather than only in a type.
    let wrapped = WrapperError::Tensor(refusal);
    println!(
        "  probe {probe}: {wrapped} (fallback still permitted at this stage: {})",
        wrapped.fallback_would_be_permitted(),
    );
    Ok(())
}

/// Builds a Candle tensor from exact `f32` bit patterns.
///
/// Bit patterns rather than parsed numbers throughout: a signed zero, a
/// subnormal, and a non-canonical NaN must survive to the comparison unchanged.
fn tensor_from_bits(
    bits: &[u32],
    rows: u64,
    columns: u64,
    device: &Device,
) -> Result<Tensor, ProofError> {
    let values: Vec<f32> = bits.iter().copied().map(f32::from_bits).collect();
    let rows = usize::try_from(rows).unwrap_or(usize::MAX);
    let columns = usize::try_from(columns).unwrap_or(usize::MAX);
    Tensor::from_vec(values, (rows, columns), device)
        .map_err(|cause| ProofError::Device(cause.to_string()))
}

/// Reads a result tensor back as exact `f32` bit patterns.
///
/// Reached only after the adapter observed terminal success for its own command
/// buffer: this call is Candle's ordinary read-back path, and it runs after
/// [`crate::wrapper::TilerPlan::apply`] returned.
fn read_bits(tensor: &Tensor) -> Result<Vec<u32>, ProofError> {
    tensor
        .to_vec1::<f32>()
        .map(|values| values.iter().map(|value| value.to_bits()).collect())
        .map_err(|cause| ProofError::Device(cause.to_string()))
}

/// Returns the envelope base path the invocation names.
///
/// Hand-parsed, and an unrecognized argument is refused instead of ignored so a
/// typo cannot look like a run that simply read somewhere else.
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

/// Returns the envelope path for one published member of the proof matrix.
fn proof_member(base: &Path, class: &str, role: &str) -> PathBuf {
    let mut name = base.as_os_str().to_owned();
    name.push(format!(".{class}.{role}"));
    PathBuf::from(name)
}

/// Reads one envelope and the proof record the producer wrote beside it.
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
    // surviving to be compared against bits it never described.
    sidecar
        .bind_to_envelope(&bytes)
        .map_err(ProofError::SidecarAssociation)?;
    Ok((bytes, sidecar))
}

/// Reads exactly `elements` big-endian `f32` bit patterns out of a sidecar payload.
///
/// The length is checked rather than truncated: a payload that decoded short
/// would reach the comparison as a shorter vector and be reported as a numerical
/// disagreement — a claim about the *device* made about a defect in the record.
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

/// The environment the diagnostic route runs under.
///
/// # This is producer-declared equality, NOT host-earned eligibility
///
/// The profile below is the one `tiler-build` *declares* for this Metal target;
/// nothing about this host earned the right to offer it.
/// [`ExecutionEnvironment::classify`] therefore answers a real question — does
/// this artifact name the profile the producer declared, under the same exact
/// descriptor — and does not answer the question ADR 0086 gates.
///
/// # The dtype rows are read, not transcribed, and the gap that leaves
///
/// Every field here comes from `declaration`, the dtype rows included:
/// [`BoundMetalCompileDeclaration::dtype_dispatchability_rows`] answers from the
/// same `TargetProfile` the compile gate consults, so a widened, narrowed, or
/// retracted measurement moves this environment with it. The literal that used
/// to stand here could not — it named `f32` and `bf16` on the strength of a
/// comment, and would have gone on naming them through a retraction.
///
/// **That removes a copy and not the authority gap.** These rows remain
/// *producer-declared*: this binary holds a real `MTLDevice` and asks it nothing
/// about either dtype, so what is stated is what the declaration measured on the
/// ledger's host, not what this one does. Earning a host row would need a
/// per-dtype observation on this device, and [`offer_the_declared_profile`] is
/// the reason that is not merely unwritten — ADR 0086 refuses the applicability
/// receipt on every macOS row currently observable, so no observation this
/// binary could take would make the profile this host's to offer.
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
        // Producer-declared, exactly like the profile above and with the same
        // caveat this function's heading states. A dtype the declaration states
        // nothing about produces no row at all, which is what keeps silence
        // fail-closed here: the loader refuses an undeclared dtype rather than
        // reading an absence as permission.
        dtype_dispatch: declaration
            .dtype_dispatchability_rows()
            .into_iter()
            .map(|(arithmetic, verdict)| (arithmetic, host_dtype_dispatch(verdict)))
            .collect(),
    })
}

/// Restates one declared dispatchability verdict in the host's own vocabulary.
///
/// An exhaustive match rather than a conversion helper, so a verdict added to
/// the compile-profile vocabulary stops this binary compiling instead of
/// reaching a wildcard that guesses. The two vocabularies are deliberately
/// separate: the compiler's is what a *profile* declares and the runtime's is
/// what a *host* states, and this function is the one place this consumer turns
/// the first into the second.
const fn host_dtype_dispatch(verdict: DTypeDispatchability) -> DTypeDispatch {
    match verdict {
        DTypeDispatchability::Dispatchable => DTypeDispatch::Dispatchable,
        DTypeDispatchability::Unsupported => DTypeDispatch::Unsupported,
    }
}

/// Reads one `sw_vers` field, or nothing when the tool does not answer.
///
/// A tool that is missing, fails, or prints nothing leaves the predicate
/// *unobserved* rather than supplying a placeholder: the policy has a typed
/// refusal for an unanswered predicate, and inventing a value here would spend
/// that distinction to make an adapter defect look like a host fact.
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
fn normalized_architecture(arch: &str) -> &str {
    if arch == "aarch64" { "arm64" } else { arch }
}

/// Earns the right to offer the declared profile, or reports the refusal.
///
/// The only route in this binary that *claims authority*. It observes the host —
/// and nothing else; no artifact, no compilation, no compiler identity reaches it
/// — and asks whether that observation satisfies the measured applicability row.
/// On every host observable today the answer is
/// [`MetalHostApplicabilityRefusal::UnknownNativeTranslationAuthority`].
///
/// It does not gate the route below, and the separation is the recorded
/// resolution rather than a convenience: the runtime machinery is worth
/// exercising on hardware, and the honest way to keep exercising it is to state
/// that the route runs on producer-declared equality and makes no applicability
/// claim.
fn offer_the_declared_profile(device: &candle_core::MetalDevice) -> MetalHostApplicabilityRefusal {
    let raw = device.metal_device().as_ref();
    let mut observation = MetalHostObservation::unobserved()
        .observing_os_family(std::env::consts::OS)
        .observing_architecture(normalized_architecture(std::env::consts::ARCH))
        .observing_device_name(raw.name().to_string())
        .observing_gpu_family(crate::adapter::observed_apple_family(device));
    if let Some(version) = sw_vers("-productVersion") {
        observation = observation.observing_os_version(version);
    }
    if let Some(build) = sw_vers("-buildVersion") {
        observation = observation.observing_os_build(build);
    }
    println!(
        "host applicability observation: os {}/{}/{}, arch {}, device {}, family {}",
        observation.os_family().unwrap_or("unobserved"),
        observation.os_version().unwrap_or("unobserved"),
        observation.os_build().unwrap_or("unobserved"),
        observation.architecture().unwrap_or("unobserved"),
        observation.device_name().unwrap_or("unobserved"),
        match observation.gpu_family() {
            Some(MetalGpuFamilySupport::Highest(family)) => family.as_str(),
            Some(MetalGpuFamilySupport::NoneNamed) => "no named Apple family",
            None => "unobserved",
        },
    );
    match evaluate_metal_host_applicability(
        MetalHostApplicabilityPolicy::FIRST_MACOS_APPLE9,
        &observation,
    ) {
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

/// Why the proof did not complete.
#[derive(Debug)]
pub enum ProofError {
    /// The invocation did not name exactly one artifact base path.
    Usage,
    /// A file could not be read.
    Read(String, std::io::Error),
    /// The producer's sidecar did not decode.
    Sidecar(ProofCodecError),
    /// The sidecar does not describe the envelope beside it.
    SidecarAssociation(ProofAssociationError),
    /// The sidecar carried no cases to prove.
    SidecarWithoutCases,
    /// A sidecar payload's length disagrees with the declared shape.
    SidecarShapeMismatch {
        /// Which payload.
        role: &'static str,
        /// Elements the artifact declares.
        declared: u64,
        /// Bytes the record holds.
        recorded: usize,
    },
    /// The recorded artifact identity is not a well-formed recording.
    RecordedIdentity(RecordedArtifactIdentityError),
    /// `tiler-build`'s authoritative declaration did not bind.
    Declaration(BoundMetalDeclarationError),
    /// The declaration's profile is not expressible as a routed environment.
    HostProfile,
    /// Candle or Metal refused something this proof needs.
    Device(String),
    /// The wrapper declined or the route did not complete.
    Wrapper(WrapperError),
    /// A probe's perturbation was accepted where it had to be refused.
    ProbeAccepted(&'static str),
    /// A probe was refused by a check other than the one it exercises.
    ///
    /// Separate from [`Self::ProbeAccepted`] because the remedy differs and the
    /// claim does too: the object *was* refused, so a run reporting this is not
    /// reporting a route that would have run — it is reporting that the refusal
    /// arrived from somewhere else, and therefore says nothing about the check
    /// the probe was built for.
    ProbeMisclassified {
        /// What the probe object addresses.
        probe: &'static str,
        /// The refusal that arrived instead.
        refusal: String,
    },
    /// The unperturbed baseline was refused, so no probe is evidence.
    BaselineRefused(String),
    /// The fallback ran where the artifact path had to.
    FallbackTakenUnexpectedly,
    /// The device's bits and the producer's recorded expectation disagree.
    Mismatch {
        /// Which member.
        member: String,
        /// What this run read back.
        observed: Vec<u32>,
        /// What the producer's reference evaluation recorded.
        expected: Vec<u32>,
    },
}

impl fmt::Display for ProofError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => formatter.write_str(
                "usage: tiler-prototype-candle --artifact <base path published by \
                 tiler-prototype-compile>",
            ),
            Self::Read(path, cause) => write!(formatter, "{path} could not be read: {cause}"),
            Self::Sidecar(cause) => write!(formatter, "the proof sidecar did not decode: {cause}"),
            Self::SidecarAssociation(cause) => write!(
                formatter,
                "the proof sidecar does not describe the envelope beside it: {cause}",
            ),
            Self::SidecarWithoutCases => {
                formatter.write_str("the proof sidecar carries no cases to prove")
            }
            Self::SidecarShapeMismatch {
                role,
                declared,
                recorded,
            } => write!(
                formatter,
                "the sidecar's {role} payload holds {recorded} byte(s) and the artifact declares \
                 {declared} element(s)",
            ),
            Self::RecordedIdentity(cause) => {
                write!(
                    formatter,
                    "the recorded artifact identity is malformed: {cause}"
                )
            }
            Self::Declaration(cause) => write!(
                formatter,
                "tiler-build's authoritative Metal declaration did not bind: {cause}",
            ),
            Self::HostProfile => formatter.write_str(
                "the authoritative declaration's profile is not expressible as a routed \
                 environment",
            ),
            Self::Device(detail) => write!(formatter, "candle/metal refused: {detail}"),
            Self::Wrapper(cause) => write!(formatter, "the wrapper did not deliver: {cause}"),
            Self::ProbeAccepted(probe) => write!(
                formatter,
                "the probe accepted {probe}, and it must be refused; the check that was supposed \
                 to say no did not",
            ),
            Self::ProbeMisclassified { probe, refusal } => write!(
                formatter,
                "the probe refused {probe} as {refusal}, which is not the \
                 undeclarable-resource class the probe exists to exercise, so it is not evidence \
                 about that check",
            ),
            Self::BaselineRefused(detail) => write!(
                formatter,
                "the unperturbed baseline was refused ({detail}), so no probe beside it is \
                 evidence about its own perturbation",
            ),
            Self::FallbackTakenUnexpectedly => formatter.write_str(
                "the ordinary Candle expression delivered the result where the artifact path had \
                 to, so the comparison would be against the wrong realization",
            ),
            Self::Mismatch {
                member,
                observed,
                expected,
            } => write!(
                formatter,
                "{member}: the device returned {observed:08x?} and the producer's reference \
                 evaluation recorded {expected:08x?}",
            ),
        }
    }
}

impl std::error::Error for ProofError {}

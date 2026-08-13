//! Carrying a published envelope onto this host's device.
//!
//! Everything decidable without a device lives in the parent module and runs in
//! the ordinary gate. What is here is what needs one: the live-device
//! qualification, the pipelines, the allocations, the encode, and the submission
//! — plus the injected refusals that establish the *device* produces each
//! refusal this crate claims it does.
//!
//! # The ordering is the contract, not a sequence
//!
//! Every obligation this host can decide is discharged while `Preflight` is
//! still held: the interface, the operand lengths, the derived program identity,
//! the placements, the pipelines, the launch capacity, and the allocations. Only
//! then is `commit` called, and nothing after it may take a fallback. The
//! command buffer's terminal state is checked *before* the host reads a byte
//! back, so a failed dispatch is reported as a dispatch failure rather than
//! compared as arithmetic.
//!
//! # The members are published here too, and only here
//!
//! [`crate::publication`] writes them, from the toolchain this module has already
//! resolved, into a private directory that is removed when the run ends. It runs
//! after `routing` rather than before it for one reason: an absent toolchain is a
//! boundary and a refused publication stage is a defect, and resolving the host
//! first is what keeps the two from wearing each other's shape.
//!
//! # One routing authority per case
//!
//! `DecodedProgram` is not `Clone` and `preflight` takes `&mut self`, so a
//! decoded program yields exactly one commit — that is ADR 0051 expressed
//! structurally rather than remembered. Each operand case therefore decodes the
//! envelope afresh. Reusing one decode across cases would not compile, and
//! reaching for a way to make it compile would be dismantling the property on
//! purpose.

use std::path::Path;

use metal::{Buffer, ComputePipelineDescriptor, ComputePipelineState, Device, MTLResourceOptions};
use tiler_artifact::program::RecordedArtifactProgramIdentity;
use tiler_build::BoundMetalCompileDeclaration;
use tiler_compiler::session::NumericalContract;
use tiler_runtime::load::{
    DecodedProgram, ExecutionEnvironment, LiveDeviceQualification, Preflight, RoutedEntry,
};

use super::{
    CONTRACTION_MEMBERS, ContractionMember, EnvelopeFailure, PLAN_ROLES, PlacedSlot, Placement,
    REDUCTION_CLASSES, RetainedComparison, RoutedMember, bind_declared_interface, case_expected,
    case_operands, compile_for_declared_shape, contraction_program, decide_live_device_requirement,
    declared_route_environment, expected_shape, plan_route, proof_member, read_artifact,
    require_contraction_interface, require_derived_program, require_serial_sum_interface,
    result_digest,
};
use crate::device_buffer::write_bytes;
use crate::device_preflight::{PreflightRefusal, allocation_fits, binding_fits, workgroup_fits};
use crate::dispatch::{
    DeviceFacts, PreparedStage, device_facts, probe_uncommitted_status, run_stages,
};
use crate::measurement::host::{self, Unresolved};
use crate::measurement::{Measured, MeasurementBoundary};
use crate::publication::{publish_contraction, publish_serial_sum_matrix};
use crate::serial_sum::{F32_BYTES, compile_under, declaration, pack_f32, unpack_f32};

/// How many result elements a routed case prints in full before eliding.
///
/// Small enough that the adversarial members' whole results are readable and the
/// L3 cells' are not: a thousand hexadecimal words is not a reader's evidence and
/// the digest is. Named here rather than written at the one match arm because the
/// number is a judgement about a log's readability, not about the comparison.
const PRINTED_ELEMENTS: usize = 16;

/// One route this device has proved it can carry out, with everything it needs.
///
/// Held across the commit: every device object the encode touches is created
/// before it, so the post-commit path allocates nothing, looks nothing up, and
/// has no failure to report. That is the property the stage exists for.
///
/// Every buffer stays owned by this value until the command buffer completes.
/// Entry-internal storage is the loader's to allocate, and a shared intermediate
/// is referenced by two entries at once, so dropping either view would leave the
/// encoder holding a binding to a freed allocation.
struct PreparedRoute {
    /// One prepared stage per routed entry, in execution order.
    stages: Vec<PreparedStage>,
    /// The buffer the program's output lands in, for read-back.
    output: Buffer,
    /// How many `f32` elements to read back out of it.
    readback: usize,
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

/// Answers each requirement from its exact prepared pipeline, and preserves the
/// pipelines for execution.
///
/// Two device stages in the order their facts become true: the live-device rows
/// first, from the bound device alone, and only then the prepared-entry
/// properties, which need a pipeline to exist. Nothing here is irreversible, so
/// abandoning between them is still the permitted fallback.
fn resolve_prepared_route<'a>(
    device: &Device,
    facts: &DeviceFacts,
    qualification: LiveDeviceQualification<'a>,
) -> Result<(Preflight<'a>, Vec<ComputePipelineState>), EnvelopeFailure> {
    let preparation = qualification
        .resolve_live_device_requirements(|request| {
            decide_live_device_requirement(facts.apple_family, request)
        })
        .map_err(EnvelopeFailure::Load)?;
    let pipelines = prepare_pipelines(device, preparation.entries())
        .map_err(|refusal| EnvelopeFailure::DevicePreflight(Box::new(refusal)))?;
    let preflight = preparation
        .resolve_target_properties(|request| {
            super::observe_metal_prepared_entry(
                request,
                pipelines[request.entry()].max_total_threads_per_threadgroup(),
            )
        })
        .map_err(EnvelopeFailure::Load)?;
    Ok((preflight, pipelines))
}

/// Proves this device can carry out a route, while declining is still permitted.
///
/// **Every entry, not the first one.** A two-entry route whose *second* pipeline
/// fails to build would reintroduce exactly the defect the pre-commit stage
/// removed. So the library, the function, the pipeline, the launch capacity, and
/// the allocations are discharged per entry, and every refusal names the entry it
/// came from — "some pipeline in this route failed" is not actionable.
///
/// Nothing here is observable if the route is then abandoned: it allocates and
/// fills host-visible storage and creates pipeline state, and encodes nothing.
fn device_preflight(
    device: &Device,
    facts: &DeviceFacts,
    preflight: &Preflight<'_>,
    pipelines: &[ComputePipelineState],
    plan: &[Vec<PlacedSlot>],
    operands: &[Vec<u32>],
    readback: u64,
) -> Result<PreparedRoute, PreflightRefusal> {
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
    let mut stages = Vec::with_capacity(routed.len());
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
        // The requirement the *artifact* proved, against the capacity the device
        // reports. Read from the routed entry's own resource record rather than
        // from the prepared pipeline: the pipeline's static reservation is what
        // the compiled function happens to hold, and the record is what the
        // packaged program declared it needs. A disagreement between them is a
        // producer defect, and comparing the declared side is what lets this
        // refuse a route the device would otherwise accept and then run short.
        crate::device_preflight::local_memory_fits(
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
                    // the payload the sidecar supplied for *that* input.
                    let bits =
                        operands
                            .get(ordinal)
                            .ok_or(PreflightRefusal::UnsuppliedOperand {
                                entry: position,
                                slot,
                                ordinal,
                                supplied: operands.len(),
                            })?;
                    write_bytes(
                        &buffer,
                        &pack_f32(
                            bits,
                            usize::try_from(F32_BYTES).expect("a carrier width fits a usize"),
                        ),
                    );
                }
                Placement::Output => output = Some(buffer.clone()),
                Placement::Internal => {}
            }
            placements.push((u64::from(placed.transport), buffer, placed.offset));
        }

        stages.push(PreparedStage {
            pipeline,
            placements,
            grid_threads: launch.grid_threads(),
            threads_per_workgroup: launch.threads_per_workgroup(),
            // The pipeline above was still built for a skipped entry, and
            // deliberately: a route is only ready if every object it names loads,
            // and an entry that runs no threads on this input may run some on the
            // next one. Skipping preparation as well would make readiness depend
            // on the operands.
            skipped: launch.grid_threads() == 0,
        });
    }

    Ok(PreparedRoute {
        stages,
        // `plan_route` refuses every binding target this run does not place, and
        // these programs declare one output, so some entry bound it.
        output: output.ok_or(PreflightRefusal::NoOutputBinding)?,
        readback: usize::try_from(readback).expect("an output element count fits a usize"),
    })
}

/// Dispatches a route this device already proved it can carry out.
fn dispatch_prepared(
    device: &Device,
    prepared: &PreparedRoute,
) -> Result<Vec<u32>, EnvelopeFailure> {
    let bytes = run_stages(
        device,
        &prepared.stages,
        &prepared.output,
        prepared.readback * 4,
    )
    .map_err(|cause| EnvelopeFailure::Stage(format!("the dispatch did not complete: {cause}")))?;
    Ok(unpack_f32(&bytes, 4, prepared.readback))
}

/// Injects each device-preflight refusal against the real route, before the
/// commit.
///
/// The device-free cases pin the comparisons and the classification; these pin
/// that the *device* produces the refusal this crate claims it does. A Metal
/// binding's rejection of an object that is not a `metallib`, or of a symbol a
/// library does not publish, is a fact about Metal rather than about this crate,
/// and asserting it needs a device.
///
/// Every probe here perturbs one input and leaves the rest alone, so a refusal is
/// evidence about the perturbation: the same device, the same route, and the same
/// operands routed moments earlier.
fn probe_device_preflight(
    device: &Device,
    facts: &DeviceFacts,
    preflight: &Preflight<'_>,
) -> Result<(), EnvelopeFailure> {
    let first = preflight.entries().first().ok_or_else(|| {
        EnvelopeFailure::Stage("a route with no entries has nothing to perturb".to_owned())
    })?;

    // A library built from bytes that are not a metallib. The digest over these
    // bytes matched, so this is content that will not execute rather than an
    // integrity failure — the distinction `CorruptArtifact` exists to carry.
    let refusal = device
        .new_library_with_data(b"tiler probe object; not an executable image")
        .err()
        .map(|detail| PreflightRefusal::LibraryRejected { entry: 0, detail })
        .ok_or_else(|| {
            EnvelopeFailure::Stage(
                "a library from non-metallib bytes was accepted, so that probe proves nothing"
                    .to_owned(),
            )
        })?;
    eprintln!("    an object that is not a metallib: {refusal}");

    // A symbol the real library does not publish.
    let library = device
        .new_library_with_data(first.object())
        .map_err(|detail| {
            EnvelopeFailure::DevicePreflight(Box::new(PreflightRefusal::LibraryRejected {
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
        .ok_or_else(|| {
            EnvelopeFailure::Stage(
                "an absent entry symbol resolved, so that probe proves nothing".to_owned(),
            )
        })?;
    eprintln!("    an entry symbol the object does not publish: {refusal}");

    // A workgroup one thread larger than the pipeline admits, using the capacity
    // this device actually reported rather than an invented number.
    let function = library
        .get_function(first.entry_symbol(), None)
        .map_err(|detail| {
            EnvelopeFailure::DevicePreflight(Box::new(PreflightRefusal::FunctionAbsent {
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
            EnvelopeFailure::DevicePreflight(Box::new(PreflightRefusal::PipelineRejected {
                entry: 0,
                symbol: first.entry_symbol().to_owned(),
                detail,
            }))
        })?;
    let capacity = pipeline.max_total_threads_per_threadgroup();
    let refusal = workgroup_fits(0, first.entry_symbol(), capacity + 1, capacity)
        .err()
        .ok_or_else(|| {
            EnvelopeFailure::Stage(
                "a workgroup larger than the pipeline admits was accepted".to_owned(),
            )
        })?;
    eprintln!("    a workgroup one thread past this pipeline: {refusal}");

    // An entry reserving one byte more threadgroup memory than this device
    // admits. The route's own entries reserve none, so the quantity is injected
    // rather than found: what this establishes is that the *device's* reported
    // capacity drives the refusal, which is the half a device-free case cannot
    // reach.
    let threadgroup = facts.max_threadgroup_memory_length;
    let refusal = crate::device_preflight::local_memory_fits(
        0,
        first.entry_symbol(),
        threadgroup + 1,
        threadgroup,
    )
    .err()
    .ok_or_else(|| {
        EnvelopeFailure::Stage(
            "an entry past this device's threadgroup memory was accepted".to_owned(),
        )
    })?;
    eprintln!("    an entry one byte past this device's threadgroup memory: {refusal}");

    // A binding needing one byte more than this device holds in one buffer.
    let limit = facts.max_buffer_length;
    let refusal = binding_fits(0, 0, limit + 1, limit).err().ok_or_else(|| {
        EnvelopeFailure::Stage("a binding past the buffer limit was accepted".to_owned())
    })?;
    eprintln!("    a binding one byte past the buffer limit: {refusal}");

    // The post-commit refusal, which no fallback follows.
    match probe_uncommitted_status(device) {
        Ok(reported) => eprintln!(
            "    a live command buffer that was never committed: {reported}, no readback taken",
        ),
        Err(terminal) => {
            return Err(EnvelopeFailure::Stage(format!(
                "an uncommitted command buffer classified as the terminal state {terminal}, so \
                 the submission probe proves nothing",
            )));
        }
    }
    Ok(())
}

/// Proves one published serial-sum member against every operand case its sidecar
/// carries.
///
/// The dispatch shape is asserted per case rather than once per member because
/// the shape is derived from the artifact on every route; checking it once would
/// leave the remaining cases free to route differently and still be reported as
/// agreeing.
fn prove_member(
    device: &Device,
    facts: &DeviceFacts,
    declaration: &BoundMetalCompileDeclaration,
    environment: &ExecutionEnvironment,
    base: &Path,
    class: &str,
    role: &str,
) -> Result<RoutedMember, EnvelopeFailure> {
    let path = proof_member(base, class, role);
    let (bytes, sidecar) = read_artifact(&path)?;

    // The shape is read from the artifact and never taken from this crate's own
    // row count; `compile_for_declared_shape` is where that discipline lives.
    // Compiled only to *name* what the artifact claims to package: the routed
    // environment comes from the declaration, not from this compilation.
    let declared_shape =
        DecodedProgram::decode(&bytes, super::SOLE_DELIVERY).map_err(EnvelopeFailure::Load)?;
    let (rows, columns, compilation) = compile_for_declared_shape(declaration, &declared_shape)?;
    drop(declared_shape);

    // The cold-consumer assertion, stated once: these bytes were written beside
    // the artifact by the producing process, and this process is stating them as
    // the identity it expects. Checked as a recording — non-empty, bounded, under
    // this build's artifact domain — and not thereby evidence of anything.
    let recorded = RecordedArtifactProgramIdentity::from_bytes(sidecar.artifact_identity_bytes())
        .map_err(|cause| EnvelopeFailure::RecordedIdentity(cause.to_string()))?;

    let (expected_entries, expected_shared) = expected_shape(role);
    let mut proved = 0_usize;
    let (mut routed_entries, mut routed_shared) = (expected_entries, expected_shared);
    for case in sidecar.cases() {
        // A fresh decode per case: see this module's own note on why.
        let mut decoded =
            DecodedProgram::decode(&bytes, super::SOLE_DELIVERY).map_err(EnvelopeFailure::Load)?;
        // Re-read per case rather than trusted from above, because the shape is
        // what every remaining check is scaled by; a member whose variants
        // disagreed about it would otherwise be measured against the first one.
        let interface = bind_declared_interface(&decoded)?;
        require_serial_sum_interface(&interface)?;

        let operands = case_operands(&interface, case)?;
        let expected = case_expected(&interface, case)?;

        let preparation = decoded
            .prepare(environment, &recorded, &interface.abi)
            .map_err(EnvelopeFailure::Load)?;
        let (preflight, pipelines) = resolve_prepared_route(device, facts, preparation)?;

        // Checked before the commit, because a route to a program this process
        // did not derive is a reason to abandon rather than to execute and
        // compare.
        require_derived_program(&compilation, preflight.kernel_program_identity())?;

        let plan = plan_route(&preflight, &interface)?;
        let prepared = device_preflight(
            device,
            facts,
            &preflight,
            &pipelines,
            &plan,
            &operands,
            interface.output_elements,
        )
        .map_err(|refusal| EnvelopeFailure::DevicePreflight(Box::new(refusal)))?;

        // ---- the routing commit, one way -----------------------------------
        let routed = preflight.commit();
        let entries = routed.entries().len();
        let shared = routed.shared_allocations().len();
        if entries != expected_entries || shared != expected_shared {
            return Err(EnvelopeFailure::UnexpectedRouteShape {
                member: format!("{class}.{role}"),
                expected_entries,
                entries,
                expected_shared,
                shared,
            });
        }
        (routed_entries, routed_shared) = (entries, shared);

        let observed = dispatch_prepared(device, &prepared)?;
        if observed != expected {
            return Err(EnvelopeFailure::Mismatch {
                path: "envelope",
                device: observed,
                reference: expected,
            });
        }
        proved += 1;
    }

    if proved == 0 {
        return Err(EnvelopeFailure::SidecarWithoutCases);
    }
    eprintln!(
        "  {class}.{role}: {rows}x{columns} declared, {proved} case(s) agree, \
         {expected_entries} dispatch(es), {expected_shared} shared allocation(s)",
    );
    // The *observed* shape, not the expected one. They agree — the loop above
    // refuses every case where they do not — and reporting the observed number
    // is what keeps a caller's own assertion over this value a second reading
    // rather than a restatement of the first.
    Ok(RoutedMember {
        name: format!("{class}.{role}"),
        proved,
        entries: routed_entries,
        shared: routed_shared,
        retained: None,
        retained_declined: None,
    })
}

/// Proves one published two-input contraction member end to end.
///
/// **The L3 remainder, and what it establishes is the *route* rather than the
/// realization.** The L3 record measured six contraction realizations under a
/// hand-written Objective-C host: a spike that produces no artifact, has no
/// identity, resolves no capability, and answers no applicability predicate. What
/// runs here is an offline-produced metallib loaded through the accepted AOT
/// path, with artifact identity carrying the offline compiler's provenance and the
/// exact native translator identity left `Unknown` per ADR 0086.
///
/// A member carrying a `retained_result_sha256` additionally has the SHA-256 of
/// **the bytes this device produced** compared against a digest a device
/// measured — which is the only comparison here that reaches outside this
/// workspace's own two implementations of the contraction. The digest of the
/// producer's *expected* bytes is computed beside it, and it is a validity
/// condition on the published record rather than a second device claim.
///
/// `retained_declined` is the caller's row verdict, resolved before anything was
/// published: `Some` means this hardware cannot speak for the retained
/// measurement, so the member still routes and is still compared against its
/// published reference and the retained comparison is not made. It is carried on
/// the result rather than dropped, because an absent comparison with a stated
/// reason and one with none are different outcomes.
fn prove_contraction(
    device: &Device,
    facts: &DeviceFacts,
    declaration: &BoundMetalCompileDeclaration,
    environment: &ExecutionEnvironment,
    base: &Path,
    member: &ContractionMember,
    retained_declined: Option<String>,
) -> Result<RoutedMember, EnvelopeFailure> {
    // Resolved once, before the case loop: the retained digest is compared only
    // where the row admits it, and folding the two into one value here is what
    // keeps the loop from re-deciding it per case.
    let retained = member
        .retained_result_sha256
        .filter(|_| retained_declined.is_none());
    let path = proof_member(base, member.class, "selected");
    let (bytes, sidecar) = read_artifact(&path)?;
    let recorded = RecordedArtifactProgramIdentity::from_bytes(sidecar.artifact_identity_bytes())
        .map_err(|cause| EnvelopeFailure::RecordedIdentity(cause.to_string()))?;

    // Read once for the report; every case re-reads it from its own decode.
    let declared =
        DecodedProgram::decode(&bytes, super::SOLE_DELIVERY).map_err(EnvelopeFailure::Load)?;
    let shape = bind_declared_interface(&declared)?;
    let (m, n, k) = require_contraction_interface(&shape)?;
    drop(declared);
    // Compiled only to *name* the program the artifact claims to package, for the
    // shape the artifact itself declares. Nothing emitted here reaches the
    // device; what this buys is the one binding between the two processes a
    // sidecar cannot forge.
    let compilation = compile_under(
        declaration,
        &contraction_program(m, n, k),
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
    )
    .map_err(|cause| EnvelopeFailure::Stage(cause.to_string()))?;
    eprintln!(
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

    // The fail-closed probes against these exact bytes, before the positive route
    // is claimed: `probe_damaged_section_content` flips a byte of the carried
    // metallib and requires the refusal, and `probe_accepted_baseline` requires
    // the unperturbed subject to route, so a refusal is evidence about the damage
    // rather than about the member.
    eprintln!("  fail-closed probes against the contraction's exact bytes:");
    super::probe_fail_closed(&super::ProbeSubject {
        bytes: &bytes,
        expected: &recorded,
        environment,
        abi: &shape.abi,
    })?;

    let mut proved = 0_usize;
    let mut last_comparison = None;
    for case in sidecar.cases() {
        let mut decoded =
            DecodedProgram::decode(&bytes, super::SOLE_DELIVERY).map_err(EnvelopeFailure::Load)?;
        let interface = bind_declared_interface(&decoded)?;
        require_contraction_interface(&interface)?;

        let operands = case_operands(&interface, case)?;
        let expected = case_expected(&interface, case)?;

        let preparation = decoded
            .prepare(environment, &recorded, &interface.abi)
            .map_err(EnvelopeFailure::Load)?;
        let (preflight, pipelines) = resolve_prepared_route(device, facts, preparation)?;
        require_derived_program(&compilation, preflight.kernel_program_identity())?;
        let plan = plan_route(&preflight, &interface)?;

        // Both operand buffers are placed and filled before the commit, and the
        // count is asserted rather than assumed: a route that bound one
        // program-input slot would leave the second operand unwritten and return
        // a tensor computed from an uninitialised buffer.
        let bound_inputs = plan
            .iter()
            .flatten()
            .filter(|slot| matches!(slot.placement, Placement::Input(_)))
            .count();
        if bound_inputs != interface.inputs.len() {
            return Err(EnvelopeFailure::UnboundOperand {
                bound: bound_inputs,
                declared: interface.inputs.len(),
            });
        }

        let prepared = device_preflight(
            device,
            facts,
            &preflight,
            &pipelines,
            &plan,
            &operands,
            interface.output_elements,
        )
        .map_err(|refusal| EnvelopeFailure::DevicePreflight(Box::new(refusal)))?;

        // ---- the routing commit, one way -----------------------------------
        let _routed = preflight.commit();
        let observed = dispatch_prepared(device, &prepared)?;

        // Both digests are taken before either comparison, so a mismatch reports
        // all three values at once. Returning at the first disagreement would
        // hide exactly the fact that separates a device defect from a fixture
        // that asks the wrong question.
        let comparison = retained.map(|retained| RetainedComparison {
            executed: result_digest(&observed),
            embedded: result_digest(&expected),
            retained,
        });

        if observed != expected {
            return Err(EnvelopeFailure::Mismatch {
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
            return Err(EnvelopeFailure::RetainedDigestMismatch {
                member: member.class,
                case: case.key().as_str().to_owned(),
                executed: comparison.executed.clone(),
                embedded: comparison.embedded.clone(),
                retained: comparison.retained,
            });
        }

        // The whole result is printed for a handful of elements and elided for a
        // profile cell: a thousand hexadecimal words is not a reader's evidence,
        // and the digest is. The element count is stated either way so an elided
        // line still says how much agreed.
        //
        // **Keyed on the result's size and not on whether a comparison was
        // made.** A cell whose retained comparison was declined still returns a
        // thousand elements, and keying on the comparison printed every one of
        // them — observed while watching the decline fire.
        match &comparison {
            None if observed.len() <= PRINTED_ELEMENTS => eprintln!(
                "    {}: {observed:08x?} against {expected:08x?}",
                case.key()
            ),
            None => eprintln!(
                "    {}: {} element(s) agree with the published reference; SHA-256 of the \
                 EXECUTED result bytes {}, not compared against any retained measurement",
                case.key(),
                observed.len(),
                result_digest(&observed),
            ),
            Some(comparison) => eprintln!(
                "    {}: {} element(s) agree with the published reference; SHA-256 of the \
                 EXECUTED result bytes {} == retained {} (the producer's published expectation \
                 hashes to {}, which is this fixture's validity condition and not a second device \
                 claim)",
                case.key(),
                observed.len(),
                comparison.executed,
                comparison.retained,
                comparison.embedded,
            ),
        }
        last_comparison = comparison;
        proved += 1;
    }

    if proved == 0 {
        return Err(EnvelopeFailure::SidecarWithoutCases);
    }
    eprintln!(
        "  {}: {m}x{n}x{k} contraction, {proved} operand case(s) agree bit for bit with the \
         published reference, over {} declared operand(s){}",
        member.class,
        shape.inputs.len(),
        match (member.retained_result_sha256, &retained_declined) {
            (Some(_), None) =>
                ", and the executed bytes carry the retained realization-probe digest".to_owned(),
            (Some(_), Some(reason)) => format!("; {reason}"),
            (None, _) => "; no realization-probe measurement exists for these operands".to_owned(),
        },
    );
    Ok(RoutedMember {
        name: format!("{}.selected", member.class),
        proved,
        entries: 1,
        shared: 0,
        retained: last_comparison,
        retained_declined,
    })
}

/// Everything one routed run needs from this host, resolved before it starts.
struct Routing {
    apple: host::AppleHost,
    facts: DeviceFacts,
    declaration: BoundMetalCompileDeclaration,
    environment: ExecutionEnvironment,
}

/// Resolves the host, the declaration, and the routed environment, or states why
/// this host cannot route.
fn routing() -> Result<Routing, Measured<()>> {
    let apple = match host::resolve() {
        Ok(apple) => apple,
        Err(Unresolved::Absent(reason)) => return Err(Measured::Unavailable(reason)),
        Err(Unresolved::Defect(detail)) => return Err(Measured::Failed(detail)),
    };
    let declaration = match declaration() {
        Ok(declaration) => declaration,
        Err(cause) => {
            return Err(Measured::Failed(format!(
                "the authoritative Metal declaration did not assemble: {cause}"
            )));
        }
    };
    let environment = match declared_route_environment(&declaration) {
        Ok(environment) => environment,
        Err(cause) => return Err(Measured::Failed(cause.to_string())),
    };
    let facts = device_facts(&apple.device);
    eprintln!("envelope route: device preflight — {facts}");
    eprintln!(
        "envelope route environment: DIAGNOSTIC — producer-declared equality against {}, NOT \
         host-earned eligibility",
        environment.target_profile.key.as_str(),
    );
    Ok(Routing {
        apple,
        facts,
        declaration,
        environment,
    })
}

/// Restates a resolution failure at the caller's own result type.
fn unresolved<T>(outcome: Measured<()>) -> Measured<T> {
    match outcome {
        Measured::Unavailable(reason) => Measured::Unavailable(reason),
        Measured::Failed(detail) => Measured::Failed(detail),
        Measured::Ran { .. } => {
            unreachable!("`routing` reports only an unavailable environment or a refused stage")
        }
    }
}

/// Publishes and routes every serial-sum member against every operand case.
pub(super) fn run_matrix() -> Measured<Vec<RoutedMember>> {
    let routing = match routing() {
        Ok(routing) => routing,
        Err(outcome) => return unresolved(outcome),
    };
    // Published with the toolchain `routing` already resolved, so a publication
    // refusal here is a stage that reached its environment and said no — a defect
    // — rather than the absent-toolchain boundary, which `routing` reported above.
    // The guard is held for the whole routed run below and removes the members
    // when it drops.
    let published = match publish_serial_sum_matrix(&routing.apple.toolchain, &routing.declaration)
    {
        Ok(published) => published,
        Err(cause) => return Measured::Failed(cause.to_string()),
    };
    let base = published.base();

    // The deep single-member run first: the fail-closed probes and the injected
    // device refusals, against the member the optimizer normally selects — the
    // case a consumer would actually get. It says nothing about the optimization,
    // because a fused plan compared only against itself is self-consistent by
    // construction, which is what the rest of the matrix is for.
    let deep = proof_member(base, "nontrivial", "selected");
    let (bytes, sidecar) = match read_artifact(&deep) {
        Ok(read) => read,
        Err(cause) => return Measured::Failed(cause.to_string()),
    };
    {
        let outcome = (|| -> Result<(), EnvelopeFailure> {
            let recorded =
                RecordedArtifactProgramIdentity::from_bytes(sidecar.artifact_identity_bytes())
                    .map_err(|cause| EnvelopeFailure::RecordedIdentity(cause.to_string()))?;
            let mut decoded = DecodedProgram::decode(&bytes, super::SOLE_DELIVERY)
                .map_err(EnvelopeFailure::Load)?;
            eprintln!(
                "  decoded: {} variant(s), required features {:?}",
                decoded.variant_count(),
                decoded.required_features(),
            );
            let interface = bind_declared_interface(&decoded)?;
            let (rows, columns) = require_serial_sum_interface(&interface)?;
            eprintln!("  the artifact declares a {rows} by {columns} input");

            // Established before the positive route is claimed: a loader that
            // accepted these bytes would say nothing about what it refuses, and
            // the refusals are half of what makes the acceptance mean anything.
            eprintln!("  fail-closed probes against these exact bytes:");
            super::probe_fail_closed(&super::ProbeSubject {
                bytes: &bytes,
                expected: &recorded,
                environment: &routing.environment,
                abi: &interface.abi,
            })?;

            let preparation = decoded
                .prepare(&routing.environment, &recorded, &interface.abi)
                .map_err(EnvelopeFailure::Load)?;
            let (preflight, _pipelines) =
                resolve_prepared_route(&routing.apple.device, &routing.facts, preparation)?;
            eprintln!("  device-preflight refusals against this exact route:");
            probe_device_preflight(&routing.apple.device, &routing.facts, &preflight)?;
            // The route is deliberately abandoned rather than committed: every
            // refusal above was taken while a fallback was still permitted, and
            // the matrix below is what commits and dispatches.
            drop(preflight);
            Ok(())
        })();
        if let Err(cause) = outcome {
            return Measured::Failed(cause.to_string());
        }
    }

    eprintln!("the proof matrix, every published member against every operand case:");
    let mut members = Vec::new();
    let mut proved = 0_usize;
    // The reduced extent names the member on disk; it is not handed to
    // `prove_member`, which reads the shape from the artifact it opened. Same
    // discipline `bind_declared_interface` documents: what a consumer may take
    // from an artifact is what the artifact says.
    for (class, _reduced_extent) in REDUCTION_CLASSES {
        for role in PLAN_ROLES {
            match prove_member(
                &routing.apple.device,
                &routing.facts,
                &routing.declaration,
                &routing.environment,
                base,
                class,
                role,
            ) {
                Ok(member) => {
                    proved += member.proved;
                    members.push(member);
                }
                Err(cause) => return Measured::Failed(cause.to_string()),
            }
        }
    }
    eprintln!(
        "{proved} case(s) proved across {} member(s); fused and materialized agree bit for bit \
         with the published reference",
        members.len(),
    );
    let boundary: MeasurementBoundary = host::boundary(&routing.apple, &routing.declaration, 0);
    Measured::Ran {
        boundary: Box::new(boundary),
        observed: members,
    }
}

/// Publishes and routes one contraction member.
pub(super) fn run_contraction(member: &ContractionMember) -> Measured<RoutedMember> {
    let routing = match routing() {
        Ok(routing) => routing,
        Err(outcome) => return unresolved(outcome),
    };
    debug_assert!(
        CONTRACTION_MEMBERS
            .iter()
            .any(|known| known.class == member.class),
        "a contraction member this module does not publish was routed",
    );

    let boundary: MeasurementBoundary = host::boundary(&routing.apple, &routing.declaration, 0);
    let declined = match retained_row(member, &boundary) {
        Ok(declined) => declined,
        // The record is a checked-in file, so failing to read it is a defect
        // rather than a boundary — and it must not degrade into comparing
        // anyway, which is what makes it `Failed` here.
        Err(cause) => return Measured::Failed(cause),
    };

    let published =
        match publish_contraction(&routing.apple.toolchain, &routing.declaration, member) {
            Ok(published) => published,
            Err(cause) => return Measured::Failed(cause.to_string()),
        };
    match prove_contraction(
        &routing.apple.device,
        &routing.facts,
        &routing.declaration,
        &routing.environment,
        published.base(),
        member,
        declined,
    ) {
        Ok(routed) => Measured::Ran {
            boundary: Box::new(boundary),
            observed: routed,
        },
        Err(cause) => Measured::Failed(cause.to_string()),
    }
}

/// Compares this host's row against the retained record's, and says whether the
/// retained comparison may be made.
///
/// **Called before anything is published**, which is the ordering the record's own
/// boundary asks for: a comparison against a measurement has to know which row it
/// is on before it makes the comparison rather than after.
/// [`crate::retained_record`] states what a difference in each field does; in
/// short, a difference in the *machine* declines the retained comparison by name
/// while the member still routes and is still compared against its published
/// reference, and a difference in the toolchain is announced and compared.
///
/// A member carrying no retained digest is not compared against a row at all —
/// there is nothing for the row to bound — and the sentence says so rather than
/// printing an agreement nothing rests on.
fn retained_row(
    member: &ContractionMember,
    boundary: &MeasurementBoundary,
) -> Result<Option<String>, String> {
    if member.retained_result_sha256.is_none() {
        eprintln!(
            "  retained row: {} carries no retained measurement, so no row bounds it",
            member.class,
        );
        return Ok(None);
    }
    let comparison =
        crate::retained_record::compare(boundary).map_err(|cause| cause.to_string())?;
    eprintln!("  retained row: {}", comparison.render());
    let hardware = comparison.hardware_differences();
    if hardware.is_empty() {
        return Ok(None);
    }
    let reason = format!(
        "the retained digest was measured on other hardware, so this run declines to compare \
         against it: {}",
        hardware
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("; "),
    );
    eprintln!("  {reason}");
    Ok(Some(reason))
}

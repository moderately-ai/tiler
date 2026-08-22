//! Carried compilation subjects and objects, and the record they publish.

use super::super::super::error::ArtifactDiagnostic;
use super::super::super::expr::{AbiType, AbiValue, AvailabilityPhase};
use super::super::super::facts::AbiFactBinder;
use super::super::super::keys::{BackendEntryKey, BackendKey, RepresentationKey};
use super::super::super::model::{
    ArtifactExecutionPolicy, BindingKind, BindingTarget, SchemaVersion,
};
use super::super::super::tests::offered_physical;
use super::super::super::tests::{
    SCALE_BITS, declare_realization, default_artifact, formulas, fused_program, lowering_provider,
    profile, selection, semantic_program, strict_affine_u4_dequantize_artifact, variant,
};
use super::super::super::{
    ArtifactProgramBuilder, CompilationEnvironment, VerifiedArtifactProgram,
};
use super::super::decode::decode;
use super::super::encode::{encode, encode_with_identity, section_digest};
use super::super::error::ArtifactCodecError;
use super::super::model::{ArtifactEnvelope, SectionKind, position};
use super::super::payload::{PayloadMetadata, decode_metadata};
use super::support::{
    artifact_with, carried_artifact, encoded, envelope_of, payload_content, payload_metadata,
    reject_artifact_forgery,
};
use tiler_digest::DigestAlgorithm;
use tiler_ir::kernel::{AddressSpace, BufferAccess, KernelType};
use tiler_ir::program::{StorageEncoding, StorageScalar};
use tiler_ir::schedule::{ExceptionalValueAssumption, NumericalPermission};
use tiler_ir::semantic::{
    OutputKey, STRICT_AFFINE_CODES_ROLE, STRICT_AFFINE_SCALE_ROLE, STRICT_AFFINE_ZERO_POINT_ROLE,
};

fn reject_forgery_with_stale_identity(
    artifact: &VerifiedArtifactProgram,
    forge: impl FnOnce(&mut ArtifactEnvelope),
) -> ArtifactCodecError {
    let mut envelope = envelope_of(artifact);
    forge(&mut envelope);
    let digests: Vec<_> = envelope
        .sections()
        .iter()
        .map(|section| section_digest(DigestAlgorithm::GOVERNED, section))
        .collect();
    let bytes = encode_with_identity(&envelope, artifact.canonical_identity(), &digests)
        .expect("the deliberately stale identity still frames");
    decode(&bytes).expect_err("a stale identity is rejected")
}

/// Assembles the same artifact from a payload nothing has compiled yet.
fn pending_artifact(source: &[u8]) -> VerifiedArtifactProgram {
    artifact_with(|draft| {
        draft.push_pending_payload(
            BackendKey::new("tiler.metal").unwrap(),
            RepresentationKey::new("metallib").unwrap(),
            SchemaVersion::new(1, 0),
            profile(),
            ArtifactExecutionPolicy::NativeImage,
            None,
            &payload_metadata(source),
        )
    })
}

/// A payload can be named before it is built, and names the same artifact.
///
/// This is the property an expansion cache key rests on, asserted as a byte
/// equality rather than argued in prose. A cache needs its key on a *miss*, so
/// the subject it composes there is derived from a portfolio whose payloads
/// have not been compiled; the entry it later publishes carries the compiled
/// artifact. If those two identities could differ, the cache would file a
/// result under a key no lookup produces — and once
/// `bind-the-cache-subject-to-the-carried-payload-provenance` ties a bundle to
/// its subject, the same gap becomes an entry served for a subject that is not
/// its own.
///
/// The equality holds for *any* emitted object, which is the second assertion:
/// identity follows the compilation subject, so a linker that is not
/// byte-reproducible cannot move it.
#[test]
fn a_pending_payload_identifies_the_artifact_its_compilation_will_produce() {
    let source = b"kernel void fused() {}";
    let pending = pending_artifact(source);
    for code in [b"first-link".as_slice(), b"second-link".as_slice()] {
        assert_eq!(
            pending.canonical_identity(),
            carried_artifact(source, code).canonical_identity(),
            "the identity derived before compiling must be the identity the \
             compiled artifact carries, whatever the linker emitted",
        );
    }
    assert_ne!(
        pending.canonical_identity(),
        pending_artifact(b"kernel void other() {}").canonical_identity(),
        "a different compilation subject is still a different artifact",
    );
}

/// The digest is a function of the subject, so the object is not needed to ask.
#[test]
fn a_payload_subject_yields_its_identity_without_its_object() {
    let metadata = payload_metadata(b"kernel void fused() {}");
    let identity = metadata
        .identity()
        .expect("a bounded subject has an identity");
    assert_eq!(
        identity,
        payload_content(b"kernel void fused() {}", b"link")
            .identity()
            .expect("a bounded subject has an identity"),
    );
    assert_ne!(
        identity,
        payload_metadata(b"kernel void other() {}")
            .identity()
            .expect("a bounded subject has an identity"),
    );
}

/// A refused build hands the draft back whole, carried object and all.
///
/// [`ArtifactProgramBuilder::build`] does not copy the draft's tables into the
/// artifact data it verifies. It **moves** them and returns them on the failure
/// path, so publishing an artifact that carries an `n`-byte compiled object no
/// longer costs `n` more for the possibility that the draft is wrong. The
/// recoverability [`ArtifactVerificationError`] promises therefore rests on that
/// return leg being complete, and a table left behind would do worse than lose
/// bytes: the corrected build would package a *different* artifact and say so
/// with a different identity. Hence the assertion is the whole envelope rather
/// than the presence of a payload.
///
/// Both envelopes descend from one clone of one draft, which is what makes the
/// comparison mean what it says — the only difference between them is that the
/// second draft failed a build first.
///
/// [`ArtifactVerificationError`]: super::super::super::ArtifactVerificationError
#[test]
fn a_recovered_builder_rebuilds_the_artifact_byte_for_byte() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], offered_physical()).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    // Every declaration but the provider selection, so the refusal below is one
    // correctable diagnostic and every table `build` takes is already populated
    // when it is taken.
    let descriptor = draft
        .push_carried_payload(
            BackendKey::new("tiler.metal").unwrap(),
            RepresentationKey::new("metallib").unwrap(),
            SchemaVersion::new(1, 0),
            profile(),
            ArtifactExecutionPolicy::NativeImage,
            None,
            payload_content(b"kernel void fused() {}", b"\x00metallib\xff"),
        )
        .unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    declare_realization(&mut draft, &program);

    let complete = |mut draft: ArtifactProgramBuilder| {
        draft
            .select_lowering_provider(selection(provider.clone()))
            .unwrap();
        encoded(&draft.build().expect("the amended draft verifies"))
    };
    let reference = complete(draft.clone());

    let error = draft.build().expect_err("an unattributed plan is rejected");
    assert_eq!(
        error.diagnostics(),
        [ArtifactDiagnostic::MissingSelectedLoweringProvider],
    );
    let (recovered, _) = error.into_parts();
    let rebuilt = complete(recovered);
    // Compared with `assert!`, because the operands are whole envelopes and a
    // failure must report their lengths rather than print two byte runs.
    assert!(
        rebuilt == reference,
        "a draft recovered from a failed build must rebuild the artifact it would have \
         built, byte for byte; rebuilt {} bytes against the reference's {}",
        rebuilt.len(),
        reference.len(),
    );
}

#[test]
fn a_carried_payload_round_trips_with_its_object_and_its_subject() {
    let artifact = carried_artifact(b"kernel void fused() {}", b"\x00metallib\xff");
    let bytes = encoded(&artifact);
    let decoded = decode(&bytes).expect("a carried envelope decodes");
    assert_eq!(decoded, envelope_of(&artifact));
    assert_eq!(decoded.payload_content().len(), 1);
    let sections = decoded.payload_content()[0].expect("the payload is carried");
    assert_eq!(
        decoded.sections()[position(sections.code)].kind,
        SectionKind::BackendPayloadCode,
    );
    assert_eq!(
        decoded.sections()[position(sections.code)].bytes,
        b"\x00metallib\xff",
    );
    assert_eq!(
        decode_metadata(&decoded.sections()[position(sections.metadata)].bytes)
            .expect("the carried subject decodes"),
        payload_metadata(b"kernel void fused() {}"),
    );
}

/// Re-seals a forged carried payload so only the check under test can reject it.
///
/// The descriptor's digest is required to equal the identity of the exact
/// metadata bytes, so editing a carried subject without restamping the digest is
/// caught by `PayloadIdentityMismatch` before anything else looks at it. Every
/// entry-mapping case below therefore rewrites both, leaving a perfectly
/// self-consistent envelope that only the obligation under test refuses.
fn reject_forged_subject(forge: impl FnOnce(&mut PayloadMetadata)) -> ArtifactCodecError {
    let artifact = carried_artifact(b"kernel void fused() {}", b"link");
    let mut envelope = envelope_of(&artifact);
    let sections = envelope.payload_content()[0].expect("the payload is carried");
    let mut metadata = payload_metadata(b"kernel void fused() {}");
    forge(&mut metadata);
    let bytes = super::super::payload::encode_metadata(&metadata);
    envelope.payloads[0].digest =
        super::super::payload::payload_identity(&bytes).expect("a bounded subject has an identity");
    envelope.sections[position(sections.metadata)].bytes = bytes;
    let encoded = encode(&envelope).expect("a forged envelope still encodes");
    decode(&encoded).expect_err("the forged envelope must be refused")
}

/// A consumer holding only bytes can name everything one dispatch needs.
///
/// This is the property the whole dispatch record exists for, so it is asserted
/// end to end from the public entry point rather than through the crate-private
/// envelope: `decode_artifact` takes bytes and nothing else, and every value
/// below is read back through the promoted view. Nothing here holds the
/// `VerifiedArtifactProgram`, the semantic program, a registry, or any producer
/// code — which is the point, because needing one would mean the artifact is not
/// the interface.
#[test]
fn a_decoded_artifact_carries_everything_one_dispatch_needs() {
    let artifact = carried_artifact(b"kernel void fused() {}", b"\x00metallib\xff");
    let bytes = artifact.encode().expect("a verified artifact encodes");
    let decoded =
        super::super::view::decode_artifact(&bytes).expect("the bytes are a valid artifact");

    // The named interface, which is also where an expression's free variables
    // come from.
    let inputs: Vec<_> = decoded.inputs().collect();
    let outputs: Vec<_> = decoded.outputs().collect();
    assert_eq!(inputs.len(), 1);
    assert_eq!(inputs[0].key().as_str(), "input");
    assert_eq!(inputs[0].extents().len(), 2);
    assert_eq!(outputs[0].key().as_str(), "result");

    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_declared_extents(inputs[0].key(), inputs[0].extents())
        .expect("the decoded interface binds its own declared shape");
    let facts = binder.build();

    let variant = decoded.variants().next().expect("one packaged variant");
    assert_eq!(variant.routing_rank(), 0);
    assert_eq!(variant.deferred_predicates().len(), 0);
    assert_eq!(
        variant.applicability_guard().evaluate(&facts),
        Ok(AbiValue::Boolean(true)),
        "the guard decides routing and must be evaluable from bytes",
    );

    let entry = variant.entries().next().expect("one executable entry");
    assert_eq!(entry.resources().buffer_bindings, 2);
    // The absence survives the round trip *as an absence*. It is recorded rather
    // than omitted, so a decoder recovers "this entry requires no realization"
    // from a byte that says so, never from bytes that never mentioned it.
    assert_eq!(entry.resources().synchronization, None);
    assert_eq!(entry.numerical().profile_key(), "tiler.test.strict-f32");
    assert_eq!(
        entry.numerical().contraction(),
        NumericalPermission::Forbidden,
    );
    assert_eq!(
        entry.numerical().permutation(),
        NumericalPermission::Forbidden,
    );
    assert_eq!(
        entry.numerical().signed_zero(),
        NumericalPermission::Forbidden,
    );
    assert_eq!(
        entry.numerical().nan_assumptions(),
        ExceptionalValueAssumption::MakeNoAssumption,
    );
    assert_eq!(
        entry.numerical().infinity_assumptions(),
        ExceptionalValueAssumption::MakeNoAssumption,
    );
    let tiler_ir::schedule::RegionNumericalRequirements::FloatingPoint {
        permutation,
        signed_zero,
        nan_assumptions,
        infinity_assumptions,
        ..
    } = entry.resources().numerical
    else {
        panic!("a packaged arithmetic entry carries floating-point resource rows");
    };
    assert_eq!(permutation, NumericalPermission::Forbidden);
    assert_eq!(signed_zero, NumericalPermission::Forbidden);
    assert_eq!(
        nan_assumptions,
        ExceptionalValueAssumption::MakeNoAssumption
    );
    assert_eq!(
        infinity_assumptions,
        ExceptionalValueAssumption::MakeNoAssumption
    );
    assert!(entry.zero_work_skips_dispatch());
    assert_eq!(entry.launch_preconditions().len(), 0);
    assert_eq!(
        entry.launch_threads().evaluate(&facts),
        Ok(AbiValue::Unsigned(2)),
    );
    assert_eq!(
        entry.threads_per_workgroup().evaluate(&facts),
        Ok(AbiValue::Unsigned(1)),
    );
    assert_eq!(entry.launch_threads().value_type(), AbiType::Unsigned);

    // The backend half: which symbol to look up and where each slot goes.
    assert_eq!(decoded.delivery_positions(), 1);
    assert_eq!(entry.delivery_positions(), 1);
    let payload = entry.payload(0).expect("the sole delivery position");
    assert_eq!(entry.backend_symbol(0), Some("tiler_fused_0"));
    assert_eq!(entry.transport_slots(0), Some([0, 1].as_slice()));
    assert_eq!(entry.backend_entry_key().as_bytes(), b"fused");
    // A position this artifact does not declare is `None` rather than the sole
    // payload under another name.
    assert_eq!(entry.payload(1), None);
    assert_eq!(entry.backend_symbol(1), None);
    assert_eq!(entry.transport_slots(1), None);
    assert_eq!(
        decoded.payload_object(payload),
        Some(b"\x00metallib\xff".as_slice()),
        "the committed object is the exact bytes the producer packaged",
    );
    assert_eq!(
        decoded.payloads()[payload].representation.as_str(),
        "metallib",
    );
    assert_eq!(
        decoded
            .payload_metadata(payload)
            .expect("the payload is carried")
            .provenance
            .target,
        "air64-apple-macosx26.0",
    );

    // And the half that decides which buffer each slot addresses.
    let bindings: Vec<_> = entry.bindings().collect();
    assert_eq!(bindings.len(), 2);
    assert_eq!(bindings[0].slot(), 0);
    assert_eq!(bindings[0].kind(), BindingKind::Buffer);
    assert_eq!(bindings[0].access(), BufferAccess::Read);
    assert_eq!(
        bindings[0].alignment(),
        tiler_ir::program::AlignmentRequirement::natural_for(tiler_ir::program::StorageScalar::F32)
    );
    assert_eq!(bindings[0].access_type(), KernelType::F32);
    assert_eq!(
        bindings[0].storage_scalar(),
        tiler_ir::program::StorageScalar::F32
    );
    assert_eq!(bindings[0].address_space(), AddressSpace::Device);
    assert_eq!(
        bindings[0].accessible_bytes().evaluate(&facts),
        Ok(AbiValue::Unsigned(24)),
    );
    assert_eq!(
        bindings[1].accessible_bytes().evaluate(&facts),
        Ok(AbiValue::Unsigned(8)),
    );
    let result = OutputKey::new("result").unwrap();
    assert_eq!(
        bindings[0].target(),
        BindingTarget::ProgramInput(inputs[0].key()),
    );
    assert_eq!(
        bindings[1].target(),
        BindingTarget::ProgramOutput(std::slice::from_ref(&result)),
    );

    // The program is named and deliberately not rebuilt.
    assert_eq!(
        variant.kernel_program_identity(),
        fused_program(&semantic_program(), SCALE_BITS)
            .canonical_identity()
            .as_bytes(),
    );
}

#[test]
fn strict_affine_components_round_trip_as_target_neutral_structural_abi() {
    let artifact = strict_affine_u4_dequantize_artifact();
    let bytes = artifact.encode().expect("strict-affine artifact encodes");
    let decoded =
        super::super::view::decode_artifact(&bytes).expect("strict-affine artifact decodes");
    assert_eq!(decoded.re_encode().expect("canonical re-encoding"), bytes);
    assert_eq!(decoded.identity(), artifact.canonical_identity().clone());

    let input = decoded.inputs().next().expect("strict-affine input");
    assert!(!input.resolved_type_encoding().is_empty());
    let components: Vec<_> = input.components().collect();
    assert_eq!(
        components
            .iter()
            .map(|component| component.role())
            .collect::<Vec<_>>(),
        [
            Some(STRICT_AFFINE_CODES_ROLE),
            Some(STRICT_AFFINE_SCALE_ROLE),
            Some(STRICT_AFFINE_ZERO_POINT_ROLE),
        ]
    );
    assert_eq!(
        components
            .iter()
            .map(|component| (
                component.storage_scalar(),
                component.storage_encoding(),
                component.access_type(),
            ))
            .collect::<Vec<_>>(),
        [
            (
                StorageScalar::U8,
                StorageEncoding::PACKED_U4_LSB_ZERO_TAIL,
                KernelType::U8,
            ),
            (
                StorageScalar::F32,
                StorageEncoding::Unpacked,
                KernelType::F32,
            ),
            (StorageScalar::U8, StorageEncoding::Unpacked, KernelType::U8,),
        ]
    );
    assert!(
        components
            .iter()
            .all(|component| component.resolved_type_encoding().is_some())
    );

    let mut binder = AbiFactBinder::new(AvailabilityPhase::LiveDevicePreflight);
    binder
        .bind_declared_extents(input.key(), input.extents())
        .expect("input shape");
    let facts = binder.build();
    let entry = decoded
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    let bindings: Vec<_> = entry.bindings().collect();
    assert_eq!(
        bindings
            .iter()
            .map(|binding| binding.component_role())
            .collect::<Vec<_>>(),
        [
            Some(STRICT_AFFINE_CODES_ROLE),
            Some(STRICT_AFFINE_SCALE_ROLE),
            Some(STRICT_AFFINE_ZERO_POINT_ROLE),
            None,
        ]
    );
    assert_eq!(
        bindings
            .iter()
            .map(|binding| binding.accessible_bytes().evaluate(&facts))
            .collect::<Vec<_>>(),
        [3, 4, 1, 20].map(|bytes| Ok(AbiValue::Unsigned(bytes)))
    );
    let result = OutputKey::new("result").expect("output key");
    for binding in &bindings[..3] {
        assert_eq!(binding.target(), BindingTarget::ProgramInput(input.key()));
    }
    assert_eq!(
        bindings[3].target(),
        BindingTarget::ProgramOutput(std::slice::from_ref(&result))
    );
    assert_eq!(
        decoded.payloads()[entry.payload(0).expect("the sole delivery position")]
            .backend
            .as_str(),
        "tiler.test.target-neutral"
    );
}

#[test]
fn strict_affine_component_corruptions_are_refused_with_typed_causes() {
    let artifact = strict_affine_u4_dequantize_artifact();
    assert_eq!(
        reject_artifact_forgery(&artifact, |envelope| {
            envelope.variants[0].entries[0].bindings[0].component_role =
                Some(tiler_ir::semantic::EncodedComponentRole::new(99));
        }),
        ArtifactCodecError::UnknownBindingTargetComponent { role: Some(99) }
    );
    assert_eq!(
        reject_artifact_forgery(&artifact, |envelope| {
            let binding = &mut envelope.variants[0].entries[0].bindings[0];
            binding.storage_scalar = StorageScalar::F32;
            binding.encoding = StorageEncoding::Unpacked;
            binding.access_type = KernelType::F32;
        }),
        ArtifactCodecError::BindingComponentMismatch
    );
    assert_eq!(
        reject_artifact_forgery(&artifact, |envelope| {
            envelope.variants[0].entries[0].bindings[0].encoding = StorageEncoding::Unpacked;
        }),
        ArtifactCodecError::BindingComponentMismatch
    );
    assert_eq!(
        reject_artifact_forgery(&artifact, |envelope| {
            envelope.variants[0].entries[0].bindings[0].access_type = KernelType::F32;
        }),
        ArtifactCodecError::BindingAccessTypeMismatch
    );
}

#[test]
fn strict_affine_component_order_is_part_of_artifact_identity() {
    let artifact = strict_affine_u4_dequantize_artifact();
    assert_eq!(
        reject_forgery_with_stale_identity(&artifact, |envelope| {
            envelope.inputs[0].components.swap(0, 1);
        }),
        ArtifactCodecError::ArtifactIdentityMismatch
    );
}

/// A descriptor-only payload reports no symbol rather than an invented one.
#[test]
fn an_uncarried_payload_publishes_no_backend_mapping() {
    let artifact = default_artifact();
    let bytes = artifact.encode().expect("a verified artifact encodes");
    let decoded =
        super::super::view::decode_artifact(&bytes).expect("the bytes are a valid artifact");
    let entry = decoded
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    let payload = entry.payload(0).expect("the sole delivery position");
    assert_eq!(entry.backend_symbol(0), None);
    assert_eq!(entry.transport_slots(0), None);
    assert_eq!(decoded.payload_object(payload), None);
    assert!(decoded.payload_metadata(payload).is_none());
    // A position past the descriptor table is the same answer and not a panic.
    assert!(decoded.payload_metadata(decoded.payloads().len()).is_none());
    assert_eq!(decoded.payload_object(decoded.payloads().len()), None);
}

/// A carried payload that maps none of the entries it realizes is refused.
///
/// Before this check the artifact layer declared such an envelope valid and left
/// a loader to discover it could not resolve a symbol. The mapping is what makes
/// a neutral entry key dispatchable, so an unmapped entry is a record that
/// cannot be dispatched from, which is a decode failure rather than a loader's
/// problem.
#[test]
fn a_carried_payload_that_maps_no_realized_entry_is_rejected() {
    assert_eq!(
        reject_forged_subject(|metadata| {
            metadata.entries[0].entry_key = BackendEntryKey::from_bytes(b"other").unwrap();
        }),
        ArtifactCodecError::UnmappedBackendEntry { payload: 0 },
    );
}

/// A mapping whose transport count is not the entry's binding count is refused.
#[test]
fn an_entry_mapping_that_does_not_place_every_binding_is_rejected() {
    assert_eq!(
        reject_forged_subject(|metadata| metadata.entries[0].transports.truncate(1)),
        ArtifactCodecError::EntryTransportCardinality {
            payload: 0,
            bindings: 2,
            extents: 0,
            transports: 1,
        },
    );
}

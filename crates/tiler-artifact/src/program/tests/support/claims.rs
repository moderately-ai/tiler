//! Plan-determinism claim fixtures (ADR 0013).

use super::super::super::{
    ArtifactBuildError, ArtifactExecutionPolicy, ArtifactProgramBuilder, BackendEntryKey,
    BackendEntryRef, BackendKey, BackendPayloadDescriptor, BindingKind, BindingSpec,
    CompilationEnvironment, EntrySpec, LaunchSpec, RepresentationKey, SchemaVersion, VariantSpec,
    VerifiedArtifactProgram,
};
use super::super::super::{
    PayloadContent, PayloadEntryMapping, PayloadMetadata, PayloadPlanDeterminismReceipt,
    PayloadPlanDeterminismRefusal, PayloadPlanDeterminismVerifier, PayloadPlatform,
    PayloadProvenance, PayloadSdkIdentity, TargetEnvironmentDeclaration,
    TargetEnvironmentDescriptor, TargetEnvironmentDescriptorSchema, TargetEnvironmentReasonCode,
    ToolComponent, ValidatedTargetEnvironmentDeclaration, VariantId,
};
use super::artifacts::offered_physical;
use super::artifacts::physical_run;
use super::artifacts::{
    declare_realization, formulas, lowering_provider, profile, rules, selection, variant,
};
use super::graphs::{SCALE_BITS, semantic_program};
use super::kernels::{fused_program, partial_window_program};
use tiler_ir::kernel::{PlanDeterminismWitness, verify_plan_determinism};
use tiler_ir::program::VerifiedKernelProgram;
use tiler_ir::semantic::ProviderIdentity;

// -------------------------------------------------------------------------
// Plan-determinism claims (ADR 0013)
// -------------------------------------------------------------------------

/// The claim fixtures' one canonical environment descriptor spelling.
pub(crate) const CLAIM_DESCRIPTOR: &[u8] = b"process-arithmetic-v1";

/// The exact object bytes the claim fixtures carry and bind.
pub(crate) const CLAIM_OBJECT: &[u8] = b"claim fixture object bytes";

pub(crate) fn claim_provider() -> ProviderIdentity {
    ProviderIdentity::new("tiler-test", "environment-authority", 3).unwrap()
}

/// One raw fixture declaration over `descriptor`.
pub(crate) fn claim_declaration_of(descriptor: &[u8]) -> TargetEnvironmentDeclaration {
    TargetEnvironmentDeclaration::new(
        claim_provider(),
        SchemaVersion::new(1, 0),
        TargetEnvironmentDescriptor::new(descriptor).unwrap(),
    )
    .unwrap()
}

pub(crate) fn claim_declaration() -> TargetEnvironmentDeclaration {
    claim_declaration_of(CLAIM_DESCRIPTOR)
}

/// The producer-side schema registration a declaration validates against.
///
/// Derived from the declaration itself, as a producer derives it from its own
/// registration: the artifact layer's join needs a validated declaration, not
/// agreement with any particular consumer's adapter.
pub(crate) struct ClaimSchema {
    pub(crate) provider: ProviderIdentity,
    pub(crate) schema: SchemaVersion,
    pub(crate) admitted: Vec<u8>,
}

impl TargetEnvironmentDescriptorSchema for ClaimSchema {
    fn provider(&self) -> &ProviderIdentity {
        &self.provider
    }

    fn schema_version(&self) -> SchemaVersion {
        self.schema
    }

    fn validate_canonical_descriptor(
        &self,
        descriptor: &[u8],
    ) -> Result<(), TargetEnvironmentReasonCode> {
        if descriptor == self.admitted {
            Ok(())
        } else {
            Err(TargetEnvironmentReasonCode::new("descriptor-not-canonical").unwrap())
        }
    }
}

pub(crate) fn validated(
    declaration: &TargetEnvironmentDeclaration,
) -> ValidatedTargetEnvironmentDeclaration {
    let schema = ClaimSchema {
        provider: declaration.provider().clone(),
        schema: declaration.descriptor_schema(),
        admitted: declaration.descriptor().as_bytes().to_vec(),
    };
    declaration.validate(&schema).unwrap()
}

/// A verifier whose backend judgment accepts; the receipt's bound values are
/// still minted by the artifact layer from the exact inputs, which is what the
/// join tests below exercise.
pub(crate) struct TrustingVerifier;

impl PayloadPlanDeterminismVerifier for TrustingVerifier {
    fn assess(
        &self,
        _witness: &PlanDeterminismWitness<'_>,
        _descriptor: &BackendPayloadDescriptor,
        _object_bytes: &[u8],
        _declaration: &ValidatedTargetEnvironmentDeclaration,
    ) -> Result<(), PayloadPlanDeterminismRefusal> {
        Ok(())
    }
}

/// One carried payload's compilation subject, keyed to one entry.
///
/// The provenance mirrors the codec suite's known-valid record; `entry_key`
/// and `source` differ per payload so two payloads in one artifact never
/// collide on the metadata-derived content digest.
pub(crate) fn claim_metadata(entry_key: &[u8], source: &[u8]) -> PayloadMetadata {
    PayloadMetadata {
        source_representation: RepresentationKey::new("metal-source").unwrap(),
        source: source.to_vec(),
        provenance: PayloadProvenance {
            toolchain: "tiler.toolchain.apple-metal".to_owned(),
            target: "air64-apple-macosx26.0".to_owned(),
            family: "apple-macos".to_owned(),
            language: "metal3.2".to_owned(),
            platform: PayloadPlatform::VersionedSdk {
                deployment_major: 26,
                deployment_minor: 0,
                sdk: PayloadSdkIdentity {
                    name: "macosx".to_owned(),
                    version: "26.0".to_owned(),
                    build: "26A5388g".to_owned(),
                },
            },
            components: vec![ToolComponent {
                role: "compiler".to_owned(),
                version: "32023.883".to_owned(),
            }],
            compile_flags: vec!["-fmetal-math-mode=safe".to_owned()],
            link_flags: Vec::new(),
        },
        entries: vec![PayloadEntryMapping {
            entry_key: BackendEntryKey::from_bytes(entry_key).unwrap(),
            symbol: format!(
                "tiler_{}_0",
                std::str::from_utf8(entry_key).expect("fixture keys are ASCII")
            ),
            transports: vec![0, 1],
        }],
        obligations: Vec::new(),
    }
}

pub(crate) fn claim_payload_content(entry_key: &[u8], code: &[u8]) -> PayloadContent {
    PayloadContent {
        metadata: claim_metadata(entry_key, b"kernel void claim() {}"),
        code: code.to_vec(),
    }
}

/// The exact descriptor `push_carried_payload` records for a claim payload.
///
/// Reconstructed from the same inputs because the draft does not expose its
/// payload table; the digest is the metadata-derived compilation subject, so
/// this names the same payload the builder holds.
pub(crate) fn claim_descriptor(
    content: &PayloadContent,
    environment: Option<TargetEnvironmentDeclaration>,
) -> BackendPayloadDescriptor {
    BackendPayloadDescriptor {
        backend: BackendKey::new("tiler.metal").unwrap(),
        representation: RepresentationKey::new("metallib").unwrap(),
        payload_schema: SchemaVersion::new(1, 0),
        digest: content.identity().unwrap(),
        compatibility: profile(),
        execution_policy: ArtifactExecutionPolicy::NativeImage,
        environment,
    }
}

/// Drives one claim attempt over the canonical one-entry fused fixture.
///
/// Everything up to the claim is shared: a carried (or pending) payload
/// declaring `environment`, the fused program's variant, and the realization
/// record. `drive` then runs the join under test and the fixture builds, so a
/// refused claim also proves the refusal left the draft coherent.
pub(crate) fn with_claim_draft(
    environment: Option<TargetEnvironmentDeclaration>,
    carried: bool,
    drive: impl FnOnce(&mut ArtifactProgramBuilder, VariantId, &VerifiedKernelProgram),
) -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let compilation = CompilationEnvironment::new([provider.clone()], offered_physical()).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, compilation).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let content = claim_payload_content(b"fused", CLAIM_OBJECT);
    let payload = if carried {
        draft
            .push_carried_payload(
                BackendKey::new("tiler.metal").unwrap(),
                RepresentationKey::new("metallib").unwrap(),
                SchemaVersion::new(1, 0),
                profile(),
                ArtifactExecutionPolicy::NativeImage,
                environment,
                content,
            )
            .unwrap()
    } else {
        draft
            .push_pending_payload(
                BackendKey::new("tiler.metal").unwrap(),
                RepresentationKey::new("metallib").unwrap(),
                SchemaVersion::new(1, 0),
                profile(),
                ArtifactExecutionPolicy::NativeImage,
                environment,
                &content.metadata,
            )
            .unwrap()
    };
    let formulas = formulas(&mut draft);
    let variant_id = draft
        .push_variant(&program, variant(&formulas, payload, b"fused"))
        .unwrap();
    declare_realization(&mut draft, &program);
    drive(&mut draft, variant_id, &program);
    draft.build().unwrap()
}

/// Mints the receipt a correct producer holds for the canonical claim fixture.
pub(crate) fn claim_receipt(witness: PlanDeterminismWitness<'_>) -> PayloadPlanDeterminismReceipt {
    let declaration = claim_declaration();
    let content = claim_payload_content(b"fused", CLAIM_OBJECT);
    let descriptor = claim_descriptor(&content, Some(declaration.clone()));
    TrustingVerifier
        .verify(
            &witness,
            &descriptor,
            CLAIM_OBJECT,
            &validated(&declaration),
        )
        .unwrap()
}

/// The complete proof-bound claim over the canonical fixture.
pub(crate) fn claimed_artifact() -> VerifiedArtifactProgram {
    with_claim_draft(
        Some(claim_declaration()),
        true,
        |draft, variant, program| {
            let witness = verify_plan_determinism(program).unwrap();
            let receipt = claim_receipt(witness);
            draft
                .publish_plan(variant, 0, &witness, &[receipt])
                .unwrap();
        },
    )
}

/// Drives one claim over the two-entry partial-window fixture.
///
/// Each entry carries its own payload declaring its entry of `declarations`,
/// with a receipt each entry's verifier minted against its own declaration, so
/// each entry is individually coherent. Returns the publication outcome and
/// the built artifact — refused or not, the draft stays buildable.
pub(crate) fn two_entry_claim(
    declarations: [TargetEnvironmentDeclaration; 2],
) -> (Result<(), ArtifactBuildError>, VerifiedArtifactProgram) {
    let semantic = semantic_program();
    let program = partial_window_program(&semantic);
    let provider = lowering_provider(1);
    let compilation = CompilationEnvironment::new([provider.clone()], offered_physical()).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, compilation).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();

    let mut entries = Vec::new();
    let mut receipts = Vec::new();
    for (key, declaration) in [&b"pointwise"[..], &b"reduction"[..]]
        .into_iter()
        .zip(declarations)
    {
        let content = claim_payload_content(key, CLAIM_OBJECT);
        let descriptor = claim_descriptor(&content, Some(declaration.clone()));
        let payload = draft
            .push_carried_payload(
                BackendKey::new("tiler.metal").unwrap(),
                RepresentationKey::new("metallib").unwrap(),
                SchemaVersion::new(1, 0),
                profile(),
                ArtifactExecutionPolicy::NativeImage,
                Some(declaration.clone()),
                content,
            )
            .unwrap();
        entries.push(EntrySpec {
            bindings: vec![
                BindingSpec {
                    kind: BindingKind::Buffer,
                },
                BindingSpec {
                    kind: BindingKind::Buffer,
                },
            ],
            launch: LaunchSpec {
                zero_work_skips_dispatch: true,
                preconditions: Vec::new(),
            },
            implementation: BackendEntryRef {
                payloads: vec![payload],
                entry_key: BackendEntryKey::from_bytes(key).unwrap(),
            },
        });
        let witness = verify_plan_determinism(&program).unwrap();
        receipts.push(
            TrustingVerifier
                .verify(
                    &witness,
                    &descriptor,
                    CLAIM_OBJECT,
                    &validated(&declaration),
                )
                .unwrap(),
        );
    }
    let variant_id = draft
        .push_variant(
            &program,
            VariantSpec {
                target_profile: profile(),
                feasibility_rules: rules(),
                selected_physical_implementations: physical_run(1),
                deferred_predicates: Vec::new(),
                entries,
            },
        )
        .unwrap();
    declare_realization(&mut draft, &program);
    let witness = verify_plan_determinism(&program).unwrap();
    let outcome = draft.publish_plan(variant_id, 0, &witness, &receipts);
    (outcome, draft.build().unwrap())
}

/// A coherently claimed two-entry artifact, for the codec suite's forgeries.
pub(crate) fn claimed_two_entry_artifact() -> VerifiedArtifactProgram {
    let (outcome, artifact) = two_entry_claim([claim_declaration(), claim_declaration()]);
    outcome.expect("agreeing declarations publish");
    artifact
}

//! Envelope projection, encoding, byte forgery, and the artifacts they apply to.

use super::super::super::error::ArtifactBuildError;
use super::super::super::handles::PayloadId;
use super::super::super::keys::{
    BackendEntryKey, BackendKey, CapabilityFamilyKey, RepresentationKey,
};
use super::super::super::model::{
    ArtifactExecutionPolicy, LoweringCapabilitySubject, SchemaVersion,
};
use super::super::super::tests::{
    OTHER_SCALE_BITS, SCALE_BITS, declare_realization, declare_realization_over, default_artifact,
    formulas, fused_program, lowering_provider, payload, prepared_requirement, profile, selection,
    semantic_program, variant,
};
use super::super::super::{
    ArtifactProgramBuilder, CompilationEnvironment, VerifiedArtifactProgram,
};
use super::super::decode::decode;
use super::super::encode::{HEADER_BYTES, MANIFEST_DIGEST_DOMAIN, encode};
use super::super::error::ArtifactCodecError;
use super::super::model::ArtifactEnvelope;
use super::super::payload::{
    PayloadContent, PayloadEntryMapping, PayloadMetadata, PayloadPlatform, PayloadProvenance,
    PayloadSdkIdentity, PayloadTargetObligation, ToolComponent,
};
use tiler_digest::DigestAlgorithm;
use tiler_ir::program::abi::TargetPropertyRequirementRelation;
use tiler_ir::semantic::OpKey;

// -------------------------------------------------------------------------
// Test-local helpers
// -------------------------------------------------------------------------

pub(crate) fn envelope_of(artifact: &VerifiedArtifactProgram) -> ArtifactEnvelope {
    ArtifactEnvelope::of(artifact).expect("a verified artifact projects")
}

pub(crate) fn encoded(artifact: &VerifiedArtifactProgram) -> Vec<u8> {
    encode(&envelope_of(artifact)).expect("a verified artifact encodes")
}

pub(crate) fn subject(
    family: &str,
    namespace: &str,
    name: &str,
    version: u32,
) -> LoweringCapabilitySubject {
    LoweringCapabilitySubject {
        family: CapabilityFamilyKey::new(family).expect("a governed family"),
        operation: OpKey::new(namespace, name, version).expect("a legal operation"),
    }
}

/// Offsets of the fields the framing header derives from the manifest bytes.
pub(crate) const TOTAL_LENGTH_AT: usize = 17;
pub(crate) const MANIFEST_LENGTH_AT: usize = 25;
pub(crate) const MANIFEST_DIGEST_AT: usize = 37;

/// Restores every derived framing field after a byte-level forgery.
///
/// A corruption that a digest catches proves only that the digest works. Every
/// adversarial case below reseals, so what rejects it is the check under test
/// rather than an integrity field the forger simply forgot to update.
pub(crate) fn reseal(bytes: &mut [u8]) {
    let total = u64::try_from(bytes.len()).expect("supported usize fits u64");
    bytes[TOTAL_LENGTH_AT..TOTAL_LENGTH_AT + 8].copy_from_slice(&total.to_be_bytes());
    let manifest_len = usize::try_from(u64::from_be_bytes(
        bytes[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8]
            .try_into()
            .expect("a fixed-width field"),
    ))
    .expect("the fixture manifest fits usize");
    let manifest = bytes[HEADER_BYTES..HEADER_BYTES + manifest_len].to_vec();
    let digest = DigestAlgorithm::GOVERNED.digest(MANIFEST_DIGEST_DOMAIN, &manifest);
    bytes[MANIFEST_DIGEST_AT..MANIFEST_DIGEST_AT + 32].copy_from_slice(digest.as_bytes());
}

pub(crate) fn manifest_len(bytes: &[u8]) -> usize {
    usize::try_from(u64::from_be_bytes(
        bytes[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8]
            .try_into()
            .expect("a fixed-width field"),
    ))
    .expect("the fixture manifest fits usize")
}

/// Inserts bytes inside the manifest and restores its two length declarations
/// and digest, so the parser under test sees the forgery rather than a stale
/// framing field.
pub(crate) fn insert_manifest_bytes(bytes: &mut Vec<u8>, at: usize, inserted: &[u8]) {
    let old_len = manifest_len(bytes);
    assert!((HEADER_BYTES..=HEADER_BYTES + old_len).contains(&at));
    bytes.splice(at..at, inserted.iter().copied());
    let new_len = old_len + inserted.len();
    bytes[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8].copy_from_slice(
        &u64::try_from(new_len)
            .expect("the fixture manifest fits u64")
            .to_be_bytes(),
    );
    reseal(bytes);
}

/// Returns the absolute offset of one unique byte pattern in the manifest.
pub(crate) fn manifest_offset(bytes: &[u8], pattern: &[u8]) -> usize {
    let found = manifest_occurrences(bytes, pattern);
    assert_eq!(found.len(), 1, "the pattern must locate exactly one field");
    found[0]
}

/// Returns every absolute offset at which one byte pattern occurs in the manifest.
///
/// A caller that expects one field uses [`manifest_offset`]; a caller reaching
/// for this one owes an assertion on how many it expects and why, and an
/// unstated count is how "the pattern moved" becomes indistinguishable from
/// "the pattern is now somewhere else too". Until manifest schema `15.0` the
/// usual second occurrence was the trailing artifact-identity *preimage*, whose
/// encoder writes several of the same fields the rows above it wrote; the
/// manifest now declares its identity by digest, so a pattern that still occurs
/// twice is restated by something other than the identity.
pub(crate) fn manifest_occurrences(bytes: &[u8], pattern: &[u8]) -> Vec<usize> {
    let manifest_len = usize::try_from(u64::from_be_bytes(
        bytes[MANIFEST_LENGTH_AT..MANIFEST_LENGTH_AT + 8]
            .try_into()
            .expect("a fixed-width field"),
    ))
    .expect("the fixture manifest fits usize");
    let manifest = &bytes[HEADER_BYTES..HEADER_BYTES + manifest_len];
    manifest
        .windows(pattern.len())
        .enumerate()
        .filter(|(_, window)| *window == pattern)
        .map(|(offset, _)| HEADER_BYTES + offset)
        .collect()
}

/// Encodes a deliberately invalid envelope and returns the decoder's rejection.
///
/// The forged model is encoded through the ordinary encoder, so the bytes
/// carry a correct manifest digest, correct section digests, and the canonical
/// identity of whatever the envelope now claims. Only a semantic check can
/// reject them.
pub(crate) fn reject_forged(forge: impl FnOnce(&mut ArtifactEnvelope)) -> ArtifactCodecError {
    let artifact = default_artifact();
    reject_artifact_forgery(&artifact, forge)
}

pub(crate) fn reject_artifact_forgery(
    artifact: &VerifiedArtifactProgram,
    forge: impl FnOnce(&mut ArtifactEnvelope),
) -> ArtifactCodecError {
    let mut envelope = envelope_of(artifact);
    forge(&mut envelope);
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    decode(&bytes).expect_err("a forged envelope is rejected")
}

/// Assembles an artifact carrying a deferred predicate and a launch precondition.
///
/// Two forgeries need it. Re-pointing the applicability guard needs the boolean
/// literal to stay reachable from a second use site, or the arena's closure
/// obligation rejects first and the type rejection is never reached; and
/// orphaning an expression needs a subtree that exactly one use site reaches.
pub(crate) fn guarded_artifact() -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft
        .select_lowering_provider(selection(provider.clone()))
        .unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.entries[0].launch.preconditions = vec![formulas.always];
    spec.deferred_predicates = vec![super::super::super::DeferredPredicateSpec {
        requirement: prepared_requirement(
            1,
            TargetPropertyRequirementRelation::ObservedAtLeastRequired,
        ),
        entry: 0,
    }];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    draft.build().unwrap()
}

/// Encodes a deliberately invalid forgery of [`guarded_artifact`].
pub(crate) fn reject_guarded_forgery(
    forge: impl FnOnce(&mut ArtifactEnvelope),
) -> ArtifactCodecError {
    let artifact = guarded_artifact();
    let mut envelope = envelope_of(&artifact);
    forge(&mut envelope);
    let bytes = encode(&envelope).expect("a forged envelope still encodes");
    decode(&bytes).expect_err("a forged envelope is rejected")
}

/// Assembles a two-variant, two-payload artifact for order-sensitive cases.
pub(crate) fn two_variant_artifact(forward: bool) -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let first = fused_program(&semantic, SCALE_BITS);
    let second = fused_program(&semantic, OTHER_SCALE_BITS);
    let providers = [lowering_provider(1), lowering_provider(2)];
    let environment = CompilationEnvironment::new(providers.iter().cloned(), []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let (left, right) = if forward { (0, 1) } else { (1, 0) };
    draft
        .select_lowering_provider(selection(providers[left].clone()))
        .unwrap();
    draft
        .select_lowering_provider(selection(providers[right].clone()))
        .unwrap();
    let (primary, spare) = if forward {
        let primary = draft.push_payload(payload(0x01)).unwrap();
        (primary, draft.push_payload(payload(0x02)).unwrap())
    } else {
        let spare = draft.push_payload(payload(0x02)).unwrap();
        (draft.push_payload(payload(0x01)).unwrap(), spare)
    };
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&first, variant(&formulas, primary, b"fused"))
        .unwrap();
    draft
        .push_variant(&second, variant(&formulas, spare, b"alternate"))
        .unwrap();
    declare_realization_over(&mut draft, &first, 2);
    draft.build().unwrap()
}

// -------------------------------------------------------------------------
// Carried backend payloads
// -------------------------------------------------------------------------

/// Builds one carried payload's compilation subject.
///
/// The provenance is deliberately not Metal-shaped beyond its values: the same
/// record shape holds a CUDA payload's `nvcc`, `ptxas`, and `sm_90`.
pub(crate) fn payload_metadata(source: &[u8]) -> PayloadMetadata {
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
            components: vec![
                ToolComponent {
                    role: "compiler".to_owned(),
                    version: "32023.883".to_owned(),
                },
                ToolComponent {
                    role: "linker".to_owned(),
                    version: "32023.883".to_owned(),
                },
            ],
            compile_flags: vec![
                "-fmetal-math-mode=safe".to_owned(),
                "-ffp-contract=off".to_owned(),
            ],
            link_flags: Vec::new(),
        },
        entries: vec![PayloadEntryMapping {
            entry_key: BackendEntryKey::from_bytes(b"fused").unwrap(),
            symbol: "tiler_fused_0".to_owned(),
            transports: vec![0, 1],
        }],
        obligations: vec![PayloadTargetObligation {
            key: "tiler.target.subnormal-arithmetic".to_owned(),
            value: "flushes-to-zero".to_owned(),
        }],
    }
}

pub(crate) fn payload_content(source: &[u8], code: &[u8]) -> PayloadContent {
    PayloadContent {
        metadata: payload_metadata(source),
        code: code.to_vec(),
    }
}

/// Assembles the one-variant fixture around a payload the closure declares.
///
/// Everything but the payload declaration is shared rather than duplicated,
/// which is what makes [`a_pending_payload_identifies_the_artifact_its_compilation_will_produce`]
/// mean something: the two artifacts it compares can differ only in the one
/// call under test.
pub(crate) fn artifact_with(
    declare: impl FnOnce(&mut ArtifactProgramBuilder) -> Result<PayloadId, ArtifactBuildError>,
) -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = declare(&mut draft).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    declare_realization(&mut draft, &program);
    draft.build().unwrap()
}

/// Assembles a one-variant artifact whose single payload carries its object.
pub(crate) fn carried_artifact(source: &[u8], code: &[u8]) -> VerifiedArtifactProgram {
    artifact_with(|draft| {
        draft.push_carried_payload(
            BackendKey::new("tiler.metal").unwrap(),
            RepresentationKey::new("metallib").unwrap(),
            SchemaVersion::new(1, 0),
            profile(),
            ArtifactExecutionPolicy::NativeImage,
            None,
            payload_content(source, code),
        )
    })
}

//! Artifact-level declarations and the artifacts this suite assembles.

use super::super::super::{
    AbiExprId, AbiRoot, ArtifactExecutionPolicy, ArtifactProgramBuilder, AvailabilityPhase,
    BackendEntryKey, BackendEntryRef, BackendKey, BackendPayloadDescriptor, BindingKind,
    BindingSpec, CapabilityFamilyKey, CompilationEnvironment, EntrySpec, FeasibilityRuleSetKey,
    FeasibilityRuleSetRef, LaunchSpec, LoweringCapabilitySubject, PayloadDigest, PayloadId,
    RepresentationKey, SchemaVersion, SelectedLoweringProvider, TargetProfileDescriptorDigest,
    TargetProfileKey, TargetProfileRef, TargetPropertyKey, VariantSpec, VerifiedArtifactProgram,
};
use super::super::super::{
    DeliveredRealizationBuilder, DeliveredRealizationRecord, DimensionBehaviour, EntryRealization,
    FactSourceProvenance, HonouringMeans, NumericalDimension, NumericalObligationKey, PolicyLocus,
    ProvenanceIdentity, ScalarArithmeticSubject, SemanticOccurrence, TargetEvidenceDeclaration,
    overlapping_behaviour,
};
use super::graphs::{SCALE_BITS, semantic_program};
use super::kernels::{fused_program, partial_window_program};
use tiler_ir::numerics::{CANONICAL_DIMENSIONS, DIMENSION_COUNT};
use tiler_ir::program::VerifiedKernelProgram;
use tiler_ir::program::abi::{
    PreparedEntryTargetRequirement, TargetPropertyProviderIdentity, TargetPropertyQuery,
    TargetPropertyRequirementRelation,
};
use tiler_ir::schedule::{
    ApproximationEnvelope, MaterializationRounding, NumericalPermission, NumericalRealization,
};
use tiler_ir::semantic::{OpKey, ProviderIdentity, SemanticProgram};

// Artifact fixtures
// -------------------------------------------------------------------------

pub(crate) fn lowering_provider(revision: u32) -> ProviderIdentity {
    ProviderIdentity::new("tiler-test", "fused-serial-sum", revision).unwrap()
}

pub(crate) fn spare_provider(revision: u32) -> ProviderIdentity {
    ProviderIdentity::new("tiler-test", "never-selected", revision).unwrap()
}

pub(crate) fn selection(provider: ProviderIdentity) -> SelectedLoweringProvider {
    SelectedLoweringProvider {
        provider,
        capability: lowering_subject("tiler", "strict-serial-sum-f32", 1),
        capability_revision: 1,
    }
}

pub(crate) fn lowering_subject(
    namespace: &str,
    name: &str,
    semantic_version: u32,
) -> LoweringCapabilitySubject {
    LoweringCapabilitySubject {
        family: CapabilityFamilyKey::new("index-access").unwrap(),
        operation: OpKey::new(namespace, name, semantic_version).unwrap(),
    }
}

pub(crate) fn payload(tag: u8) -> BackendPayloadDescriptor {
    BackendPayloadDescriptor {
        environment: None,
        backend: BackendKey::new("tiler.metal").unwrap(),
        representation: RepresentationKey::new("metallib").unwrap(),
        payload_schema: SchemaVersion::new(1, 0),
        digest: PayloadDigest::from_bytes([tag, 0xb2, 0xc3]).unwrap(),
        compatibility: profile(),
        execution_policy: ArtifactExecutionPolicy::NativeImage,
    }
}

pub(crate) fn profile() -> TargetProfileRef {
    TargetProfileRef {
        key: TargetProfileKey::new("tiler.test.baseline").unwrap(),
        descriptor: TargetProfileDescriptorDigest::from_bytes([0x01, 0x02]).unwrap(),
    }
}

pub(crate) fn rules() -> FeasibilityRuleSetRef {
    FeasibilityRuleSetRef {
        key: FeasibilityRuleSetKey::new("tiler.test.feasibility").unwrap(),
        revision: 1,
    }
}

/// Builds the delivered-realization record every artifact fixture must carry.
///
/// Derived from the packaged entries' own realization rather than written out,
/// which is what lets one helper serve every fixture in this file whatever
/// numerical contract its program schedules: the eight overlapping dimensions
/// come from [`overlapping_behaviour`], and the three the scheduled realization
/// does not carry take the strict values that contract implies. A fixture whose
/// realization changed would otherwise need its record edited beside it, and the
/// two would drift.
///
/// `entries` is the flat **declared** packaged-entry count — every variant's
/// entries, summed — because [`ArtifactProgramBuilder::declare_realization`]
/// takes the space a producer can see and `build` remaps it.
///
/// One obligation, at the computation locus of occurrence 0, so every fixture
/// exercises a `Required` disposition and a canonical obligation range rather
/// than only the all-`NotRequired` shape.
///
/// `subject` is the arithmetic the packaged program actually computes in, and it
/// is a parameter because nothing downstream can catch it being wrong:
/// [`validate_against_artifact`] compares the record's behaviours to each bound
/// entry's realization and never reads the subject's arithmetic type, so a
/// `bf16` artifact carrying the `f32` subject would build, encode, decode, and
/// state something false about which arithmetic its delivered numerics govern.
///
/// [`validate_against_artifact`]: super::super::super::validate_against_artifact
pub(crate) fn realization_record(
    profile: &TargetProfileRef,
    subject: &ScalarArithmeticSubject,
    numerical: NumericalRealization,
    entries: u32,
) -> DeliveredRealizationRecord {
    let entry = EntryRealization::of(numerical);
    let mut resolutions =
        [DimensionBehaviour::Transform(NumericalPermission::Forbidden); DIMENSION_COUNT];
    for dimension in CANONICAL_DIMENSIONS {
        resolutions[dimension.index()] =
            overlapping_behaviour(dimension, entry).unwrap_or(match dimension {
                NumericalDimension::ApproximateIntrinsics => {
                    DimensionBehaviour::Approximation(ApproximationEnvelope::Forbidden)
                }
                NumericalDimension::MaterializationRounding => {
                    DimensionBehaviour::Rounding(MaterializationRounding::NearestTiesToEven)
                }
                // Reciprocal transform, the third dimension no scheduled
                // realization carries. Written as the remaining arm rather than
                // a wildcard over all eleven, so a dimension leaving the
                // overlapping set stops the build here.
                _ => DimensionBehaviour::Transform(NumericalPermission::Forbidden),
            });
    }
    let subject = subject.identity();
    let mut record = DeliveredRealizationBuilder::new(profile.clone());
    record
        .declare_scalar_arithmetic(subject.clone(), resolutions)
        .expect("the fixture contract");
    record
        .require(
            &subject,
            NumericalDimension::Contraction,
            NumericalObligationKey::new(SemanticOccurrence::new(0), PolicyLocus::Computation),
            resolutions[NumericalDimension::Contraction.index()],
            TargetEvidenceDeclaration {
                declared: resolutions[NumericalDimension::Contraction.index()],
                means: HonouringMeans::SupportedExactly,
                profile: profile.clone(),
                source: FactSourceProvenance::governed(
                    ProvenanceIdentity::new("tiler.test.baseline", 1),
                    ProvenanceIdentity::new("tiler.test.guarantee", 1),
                ),
            },
        )
        .expect("the fixture obligation");
    for entry in 0..entries {
        record
            .bind_entry(entry, &subject)
            .expect("a packaged entry");
    }
    record.build().expect("the fixture record")
}

/// Declares the fixture record for a draft that packages one program once.
///
/// The overwhelmingly common shape in this file, spelled once so a fixture that
/// is *not* that shape is visible by not using it.
pub(crate) fn declare_realization(
    draft: &mut ArtifactProgramBuilder,
    program: &VerifiedKernelProgram,
) {
    declare_realization_over(draft, program, 1);
}

/// Declares the fixture record for a draft packaging one program `variants` times.
pub(crate) fn declare_realization_over(
    draft: &mut ArtifactProgramBuilder,
    program: &VerifiedKernelProgram,
    variants: u32,
) {
    declare_realization_at(draft, program, &ScalarArithmeticSubject::f32(), variants);
}

/// The same declaration for a program computing in `subject`'s arithmetic.
pub(crate) fn declare_realization_at(
    draft: &mut ArtifactProgramBuilder,
    program: &VerifiedKernelProgram,
    subject: &ScalarArithmeticSubject,
    variants: u32,
) {
    let numerical = program
        .stages()
        .next()
        .expect("a packaged program has a stage")
        .kernel()
        .numerical();
    let stages = u32::try_from(program.stages().len()).expect("a bounded stage table fits u32");
    draft
        .declare_realization(realization_record(
            &profile(),
            subject,
            numerical,
            stages * variants,
        ))
        .expect("the fixture record");
}

pub(crate) fn prepared_requirement(
    required: u64,
    relation: TargetPropertyRequirementRelation,
) -> PreparedEntryTargetRequirement {
    let query = TargetPropertyQuery::new(
        TargetPropertyKey::new("tiler.target.prepared-entry.max-threads-per-workgroup").unwrap(),
        AvailabilityPhase::PreparedKernelPreflight,
        TargetPropertyProviderIdentity::new("tiler", "prepared-entry-properties", 1).unwrap(),
    )
    .unwrap();
    PreparedEntryTargetRequirement::new(query, required, relation).unwrap()
}

/// The expression handles every fixture variant is assembled from.
pub(crate) struct Formulas {
    /// The literal `1`, used by launch-precondition fixtures.
    pub(crate) one: AbiExprId,
    /// The literal `true`, used by deferred-predicate fixtures.
    pub(crate) always: AbiExprId,
}

pub(crate) fn formulas(draft: &mut ArtifactProgramBuilder) -> Formulas {
    // Only what a caller still *supplies*. The applicability guard, launch
    // geometry, and accessible ranges are derived from the bound program now, so
    // minting the extent and byte-count formulas would leave them unreachable
    // from any use site -- the `UnusedExpression` the artifact refuses, and what
    // made two earlier attempts at this change look like an obligation conflict.
    let one = draft.push_root(AbiRoot::UnsignedLiteral(1)).unwrap();
    let always = draft.push_root(AbiRoot::BooleanLiteral(true)).unwrap();
    Formulas { one, always }
}

pub(crate) fn entry(_formulas: &Formulas, payload: PayloadId, key: &[u8]) -> EntrySpec {
    EntrySpec {
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
    }
}

pub(crate) fn variant(formulas: &Formulas, payload: PayloadId, key: &[u8]) -> VariantSpec {
    VariantSpec {
        target_profile: profile(),
        feasibility_rules: rules(),
        deferred_predicates: Vec::new(),
        entries: vec![entry(formulas, payload, key)],
    }
}

/// Assembles the canonical one-variant artifact over one packaged program.
pub(crate) fn build_artifact(
    semantic: &SemanticProgram,
    program: &VerifiedKernelProgram,
    selected: ProviderIdentity,
    available: &[ProviderIdentity],
) -> VerifiedArtifactProgram {
    // The physical role is offered empty throughout this suite: no fixture here
    // packages a selected physical implementation, so granting physical
    // authority would be a claim the artifacts do not make.
    let environment = CompilationEnvironment::new(available.iter().cloned(), []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(semantic, environment).unwrap();
    draft.select_lowering_provider(selection(selected)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    declare_realization(&mut draft, program);
    draft.build().unwrap()
}

/// Builds the canonical fixture with exact selected operation subjects.
pub(crate) fn artifact_with_selected_operations(
    operations: &[(&str, &str, u32)],
) -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    for (namespace, name, version) in operations {
        draft
            .select_lowering_provider(SelectedLoweringProvider {
                provider: provider.clone(),
                capability: lowering_subject(namespace, name, *version),
                capability_revision: 1,
            })
            .unwrap();
    }
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    declare_realization(&mut draft, &program);
    draft.build().unwrap()
}

/// The two-stage variant whose scratch bindings start at a nonzero offset.
///
/// Nothing here states that offset. The guard, launch geometry, and accessible
/// ranges — the offset included — are derived from the bound program, so the
/// spec only pairs each stage with its backend entry; a producer has no field
/// through which it could restate the placement, honestly or otherwise.
pub(crate) fn partial_window_variant(payload: PayloadId) -> VariantSpec {
    let entry = |key: &[u8]| EntrySpec {
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
    };
    VariantSpec {
        target_profile: profile(),
        feasibility_rules: rules(),
        deferred_predicates: Vec::new(),
        entries: vec![entry(b"pointwise"), entry(b"reduction")],
    }
}

/// Assembles the two-stage artifact whose temporary is bound at a nonzero offset.
pub(crate) fn partial_window_artifact() -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = partial_window_program(&semantic);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    draft
        .push_variant(&program, partial_window_variant(descriptor))
        .unwrap();
    declare_realization(&mut draft, &program);
    draft.build().unwrap()
}

pub(crate) fn default_artifact() -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    build_artifact(&semantic, &program, provider.clone(), &[provider])
}

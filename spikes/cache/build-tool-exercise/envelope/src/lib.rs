//! Compiles and encodes one real artifact envelope.
//!
//! This is the expensive step the expansion cache exists to spare, and it is a
//! genuine one: it runs `tiler-compiler`'s governed session and encodes the
//! result through `tiler-artifact`, so bytes produced here are accepted by the
//! real `decode_artifact` that
//! [`ExpansionCache`](tiler_cache::expansion::ExpansionCache) validates every hit
//! against.
//!
//! # Why nothing here is memoized
//!
//! A `OnceLock` around the compilation would make repeat expansions inside one
//! long-lived proc-macro server cheap — and would hide the exact quantity this
//! spike measures. The cache's claim is that it suppresses duplicate
//! *compilation*; an in-process memo would suppress it first and report a
//! success the cache did not earn.
//!
//! # What it does not use
//!
//! No Metal toolchain and no device. `tiler-compiler` depends only on
//! `tiler-ir`, so the whole path is host computation and the spike runs anywhere
//! the workspace builds. The payload is declared by descriptor rather than
//! carried, which is what keeps a real compiled object out of the picture — the
//! envelope is assembled to be identified, which is all a cache key and a
//! validation pass need.

use tiler_artifact::program::{
    ApproximationEnvelope, ArtifactBuildError, ArtifactExecutionPolicy, ArtifactProgramBuilder,
    BackendEntryKey, BackendEntryRef, BackendKey, BackendPayloadDescriptor, BindingKind,
    BindingSpec, CANONICAL_DIMENSIONS, CapabilityFamilyKey, CompilationEnvironment,
    DIMENSION_COUNT, DeliveredRealizationBuilder, DeliveredRealizationRecord, DimensionBehaviour,
    EntryRealization, EntrySpec, FactSourceProvenance, FeasibilityRuleSetKey,
    FeasibilityRuleSetRef, HonouringMeans, LaunchSpec, LoweringCapabilitySubject,
    MaterializationRounding, NumericalDimension, NumericalObligationKey, NumericalPermission,
    PayloadDigest, PolicyLocus, ProvenanceIdentity, RepresentationKey, ScalarArithmeticSubject,
    SchemaVersion, SelectedLoweringProvider, SemanticOccurrence, TargetEvidenceDeclaration,
    TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef, VariantSpec,
    VerifiedArtifactProgram, overlapping_behaviour,
};
use tiler_compiler::session::{Compilation, NumericalContract, PlanAlternative, compile_governed};
use tiler_ir::program::VerifiedKernelProgram;
use tiler_ir::semantic::{
    F32, F32Add, F32Constant, F32Multiply, InputKey, OutputKey, SemanticProgram,
    SemanticProgramBuilder, StrictSerialF32Sum,
};
use tiler_ir::shape::{Axis, Shape};

/// Rows of the exercised program's input.
const ROWS: u64 = 4;
/// Columns of the exercised program's input.
const COLUMNS: u64 = 3;

/// Compiles, assembles, and encodes one artifact envelope.
///
/// # Panics
///
/// Panics when the governed program fails to compile or the assembled artifact
/// fails to verify or encode. Each of those is a defect in this spike or in the
/// workspace it builds against, never a cache outcome, so failing loudly here
/// keeps it distinguishable from the cache falling open.
#[must_use]
pub fn encoded_envelope() -> Vec<u8> {
    let semantic = serial_sum_program(ROWS, COLUMNS);
    let compilation = compile_governed(&semantic, NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32)
        .expect("the governed program compiles");
    let plan = compilation.selected().expect("a selected plan alternative");
    assemble(&semantic, &compilation, plan)
        .expect("the compiler capability packages without narrowing")
        .encode()
        .expect("the envelope encodes")
}

/// Builds `sum((input * 1.0) + 0.0)` over the reduced axis.
fn serial_sum_program(rows: u64, columns: u64) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the governed profile composes");
    let input = builder
        .input::<F32>(
            InputKey::new("input").expect("the input key is valid"),
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
            OutputKey::new("result").expect("the output key is valid"),
            sum,
        )
        .expect("the output binds");
    builder.build().expect("the program verifies")
}

/// Packages one plan alternative and a declared payload as an artifact.
fn assemble(
    semantic: &SemanticProgram,
    compilation: &Compilation,
    plan: PlanAlternative<'_>,
) -> Result<VerifiedArtifactProgram, ArtifactBuildError> {
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
    [],
)
    .expect("the offered providers compose an environment");
    let mut builder =
        ArtifactProgramBuilder::new(semantic, environment).expect("a builder identity remains");
    for selected in plan.selected_capabilities() {
        let subject = selected.subject();
        builder
            .select_lowering_provider(SelectedLoweringProvider {
                provider: selected.provider().clone(),
                capability: LoweringCapabilitySubject {
                    family: CapabilityFamilyKey::new(subject.family().key_token())?,
                    operation: subject.operation().clone(),
                },
                capability_revision: selected.capability_revision(),
            })
            .expect("a selected provider was offered");
    }

    let payload = builder
        .push_payload(BackendPayloadDescriptor {
            backend: BackendKey::new("tiler.metal").expect("a governed backend key"),
            representation: RepresentationKey::new("metallib")
                .expect("a governed representation key"),
            payload_schema: SchemaVersion::new(1, 0),
            digest: PayloadDigest::from_bytes([0xe1, 0xe2, 0xe3])
                .expect("a bounded payload digest"),
            compatibility: profile.clone(),
            execution_policy: ArtifactExecutionPolicy::NativeImage,
        })
        .expect("the declared payload is accepted");

    let program = plan.abi().kernel_program();

    let entries: Vec<EntrySpec> = program
        .stages()
        .map(|stage| EntrySpec {
            bindings: stage
                .accesses()
                .map(|_| BindingSpec {
                    kind: BindingKind::Buffer,
                })
                .collect(),
            launch: LaunchSpec {
                // Not a choice: every verified scheduled region carries it.
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

    builder
        .push_variant(
            program,
            VariantSpec {
                target_profile: profile.clone(),
                feasibility_rules: rules,
                deferred_predicates: Vec::new(),
                entries,
            },
        )
        .expect("the variant packages the plan it was built from");
    builder
        .declare_realization(realization_record(&profile, program))
        .expect("the record agrees with the packaged portfolio");
    Ok(builder.build().expect("the assembled artifact verifies"))
}

/// Builds the delivered-realization record every executable artifact carries.
///
/// The eleven resolutions are derived from the packaged program's own scheduled
/// realization rather than restated here, so this harness cannot describe a
/// contract its plan does not schedule. Every packaged entry is bound, in the
/// flat declared space; one variant means that is the program's stage count.
///
/// The obligation is stated at the computation locus of occurrence 0 with
/// `SupportedExactly` means, which is what the strict contract this spike
/// compiles under actually rests on. A harness that invented a relaxation it did
/// not need would be writing a fact rather than recording one.
fn realization_record(
    profile: &TargetProfileRef,
    program: &VerifiedKernelProgram,
) -> DeliveredRealizationRecord {
    let entry = EntryRealization::of(
        program
            .stages()
            .next()
            .expect("a packaged program has a stage")
            .kernel()
            .numerical(),
    );
    let entries = u32::try_from(program.stages().len()).expect("a bounded stage table fits u32");
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
                // realization carries. The remaining arm rather than a wildcard
                // over all eleven, so a dimension leaving the overlapping set
                // stops the build here.
                _ => DimensionBehaviour::Transform(NumericalPermission::Forbidden),
            });
    }
    let subject = ScalarArithmeticSubject::f32().identity();
    let mut record = DeliveredRealizationBuilder::new(profile.clone());
    record
        .declare_scalar_arithmetic(subject.clone(), resolutions)
        .expect("the selected scalar contract");
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
                    ProvenanceIdentity::new(profile.key.as_str(), 1),
                    ProvenanceIdentity::new("tiler.spike.strict-f32-guarantee", 1),
                ),
            },
        )
        .expect("the obligation this packaged route relies on");
    for entry in 0..entries {
        record
            .bind_entry(entry, &subject)
            .expect("a packaged entry");
    }
    record.build().expect("the record this artifact delivers")
}

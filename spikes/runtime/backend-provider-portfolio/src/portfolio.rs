//! Package Metal and CPU members as one artifact sharing one variant target.

use tiler_artifact::program::{
    ArtifactBuildError, ArtifactExecutionPolicy, ArtifactProgramBuilder, BackendEntryKey,
    BackendEntryRef, BindingKind, BindingSpec, CapabilityKey, CompilationEnvironment,
    DeferredPredicateSpec, EntrySpec, FeasibilityRuleSetKey, FeasibilityRuleSetRef, LaunchSpec,
    PayloadContent, PayloadId, RecordedArtifactProgramIdentity, SchemaVersion, SelectedProvider,
    TargetProfileDescriptorDigest, TargetProfileKey, TargetProfileRef, VariantSpec,
    VerifiedArtifactProgram,
};
use tiler_build::realization;
use tiler_compiler::session::{Compilation, PlanAlternative};
use tiler_ir::semantic::SemanticProgram;

use crate::cpu::{self, ProducedCpu};
use crate::metal;

/// One packaged multi-family portfolio.
pub struct PackagedPortfolio {
    /// Encoded envelope bytes.
    pub bytes: Vec<u8>,
    /// Identity recorded beside those bytes.
    pub expected: RecordedArtifactProgramIdentity,
}

/// Why portfolio assembly failed.
#[derive(Debug)]
pub enum PortfolioError {
    /// Assembly refused for a reason this spike did not expect.
    Assemble(String),
}

impl std::fmt::Display for PortfolioError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assemble(message) => write!(formatter, "portfolio.assemble: {message}"),
        }
    }
}

impl std::error::Error for PortfolioError {}

/// Attempts to package two variants under two different assessed targets.
///
/// Returns the `TargetProfileMismatch` `check_subject` produces when the second
/// variant names a different profile than the first.
pub fn refuse_mixed_targets(
    semantic: &SemanticProgram,
    metal_compilation: &Compilation,
    metal_plan: PlanAlternative<'_>,
    cpu_compilation: &Compilation,
    cpu_plan: PlanAlternative<'_>,
    metal_content: &PayloadContent,
    cpu: &ProducedCpu,
) -> Result<ArtifactBuildError, PortfolioError> {
    let metal_profile = target_ref(metal_compilation);
    let mut other = metal_profile.clone();
    other.descriptor = TargetProfileDescriptorDigest::from_bytes(b"portfolio-other-descriptor")
        .map_err(|error| PortfolioError::Assemble(error.to_string()))?;
    match assemble_with(
        semantic,
        metal_compilation,
        metal_plan,
        cpu_compilation,
        cpu_plan,
        metal_profile,
        other,
        metal_content,
        cpu,
    ) {
        Err(ArtifactBuildError::TargetProfileMismatch) => {
            Ok(ArtifactBuildError::TargetProfileMismatch)
        }
        Err(error) => Err(PortfolioError::Assemble(format!(
            "mixed-target assembly refused as {error}, not TargetProfileMismatch"
        ))),
        Ok(_) => Err(PortfolioError::Assemble(
            "mixed-target assembly succeeded where TargetProfileMismatch was required".into(),
        )),
    }
}

/// Packages both families under one shared variant-level target.
pub fn assemble_shared(
    semantic: &SemanticProgram,
    metal_compilation: &Compilation,
    metal_plan: PlanAlternative<'_>,
    cpu_compilation: &Compilation,
    cpu_plan: PlanAlternative<'_>,
    shared: TargetProfileRef,
    metal_content: &PayloadContent,
    cpu: &ProducedCpu,
) -> Result<PackagedPortfolio, PortfolioError> {
    let artifact = assemble_with(
        semantic,
        metal_compilation,
        metal_plan,
        cpu_compilation,
        cpu_plan,
        shared.clone(),
        shared,
        metal_content,
        cpu,
    )
    .map_err(|error| PortfolioError::Assemble(error.to_string()))?;
    let bytes = artifact
        .encode()
        .map_err(|error| PortfolioError::Assemble(error.to_string()))?;
    let expected =
        RecordedArtifactProgramIdentity::from_bytes(artifact.canonical_identity().as_bytes())
            .map_err(|error| PortfolioError::Assemble(error.to_string()))?;
    Ok(PackagedPortfolio { bytes, expected })
}

fn assemble_with(
    semantic: &SemanticProgram,
    metal_compilation: &Compilation,
    metal_plan: PlanAlternative<'_>,
    cpu_compilation: &Compilation,
    cpu_plan: PlanAlternative<'_>,
    metal_profile: TargetProfileRef,
    cpu_profile: TargetProfileRef,
    metal_content: &PayloadContent,
    cpu: &ProducedCpu,
) -> Result<VerifiedArtifactProgram, ArtifactBuildError> {
    let environment = CompilationEnvironment::new(
        metal_compilation
            .offered_lowering_providers()
            .iter()
            .cloned(),
    )?;
    let mut draft = ArtifactProgramBuilder::new(semantic, environment)?;
    for selected in metal_plan.selected_capabilities() {
        draft.select_provider(SelectedProvider {
            provider: selected.provider().clone(),
            capability: CapabilityKey::new(selected.capability_key())?,
            capability_revision: selected.capability_revision(),
        })?;
    }

    let metal_payload = draft.push_carried_payload(
        metal::backend(),
        metal::representation(),
        SchemaVersion::new(1, 0),
        metal_profile.clone(),
        ArtifactExecutionPolicy::NativeImage,
        metal_content.clone(),
    )?;
    let cpu_payload = draft.push_carried_payload(
        cpu::backend(),
        cpu::representation(),
        cpu::payload_schema(),
        cpu_profile.clone(),
        ArtifactExecutionPolicy::NativeImage,
        cpu.content.clone(),
    )?;

    push_plan_variant(
        &mut draft,
        metal_compilation,
        metal_plan,
        metal_profile.clone(),
        metal_payload,
    )?;
    push_plan_variant(
        &mut draft,
        cpu_compilation,
        cpu_plan,
        cpu_profile,
        cpu_payload,
    )?;

    let entries = u32::try_from(
        metal_plan.abi().kernel_program().stages().len()
            + cpu_plan.abi().kernel_program().stages().len(),
    )
    .expect("a bounded entry table fits u32");
    draft.declare_realization(
        realization::translate(metal_plan.delivered_realization(), &metal_profile, entries)
            .map_err(|_| ArtifactBuildError::RealizationProfileMismatch)?,
    )?;
    draft
        .build()
        .map_err(|_error| ArtifactBuildError::InterfaceMismatch)
}

fn target_ref(compilation: &Compilation) -> TargetProfileRef {
    TargetProfileRef {
        key: TargetProfileKey::new(compilation.target_profile_key())
            .expect("the compilation's profile key is governed"),
        descriptor: TargetProfileDescriptorDigest::from_bytes(
            compilation.target_profile_descriptor(),
        )
        .expect("the compilation's descriptor is an identity"),
    }
}

fn push_plan_variant(
    draft: &mut ArtifactProgramBuilder,
    compilation: &Compilation,
    plan: PlanAlternative<'_>,
    profile: TargetProfileRef,
    payload: PayloadId,
) -> Result<(), ArtifactBuildError> {
    let program = plan.abi().kernel_program();
    let deferred_predicates = plan
        .prepared_entry_target_requirements()
        .map(|requirement| DeferredPredicateSpec {
            requirement: requirement.requirement().clone(),
            entry: requirement.entry(),
        })
        .collect();
    let mut entries = Vec::new();
    for stage in program.stages() {
        entries.push(EntrySpec {
            bindings: stage
                .accesses()
                .map(|_| BindingSpec {
                    kind: BindingKind::Buffer,
                })
                .collect(),
            launch: LaunchSpec {
                zero_work_skips_dispatch: true,
                preconditions: Vec::new(),
            },
            implementation: BackendEntryRef {
                payloads: vec![payload],
                entry_key: BackendEntryKey::from_bytes(
                    stage.kernel().canonical_identity().as_bytes(),
                )?,
            },
        });
    }
    draft.push_variant(
        program,
        VariantSpec {
            target_profile: profile,
            feasibility_rules: FeasibilityRuleSetRef {
                key: FeasibilityRuleSetKey::new(compilation.feasibility_rule_set_key())?,
                revision: compilation.feasibility_rule_set_revision(),
            },
            deferred_predicates,
            entries,
        },
    )?;
    Ok(())
}

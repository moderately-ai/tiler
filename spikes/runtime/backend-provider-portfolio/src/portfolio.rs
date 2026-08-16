//! Package Metal and CPU members as one artifact sharing one variant target.

use tiler_artifact::program::{
    ArtifactBuildError, ArtifactExecutionPolicy, ArtifactProgramBuilder, BackendEntryKey,
    BackendEntryRef, BindingKind, BindingSpec, CapabilityFamilyKey, CompilationEnvironment,
    DeferredPredicateSpec, EntrySpec, FeasibilityRuleSetKey, FeasibilityRuleSetRef, LaunchSpec,
    LoweringCapabilitySubject, PayloadContent, PayloadId, RecordedArtifactProgramIdentity,
    SchemaVersion, SelectedProvider, TargetProfileDescriptorDigest, TargetProfileKey,
    TargetProfileRef, VariantSpec, VerifiedArtifactProgram,
};
use tiler_build::realization;
use tiler_compiler::session::{Compilation, PlanAlternative};
use tiler_ir::program::abi::{
    PreparedEntryTargetRequirement, TargetPropertyKey, TargetPropertyProviderIdentity,
    TargetPropertyQuery,
};
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

/// One independent mutation of the prepared-entry request used by fail-closed probes.
#[derive(Clone, Copy, Debug)]
pub enum PreparedEntryProbe {
    /// Name a provider this CPU adapter does not own.
    UnknownProvider,
    /// Name a property key this CPU adapter does not own.
    UnknownProperty,
    /// Require one more thread than the CPU adapter's observed 1,024.
    RequiredAboveObserved,
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
        None,
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
        None,
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

/// Packages both families while perturbing only the CPU variant's prepared-entry request.
pub fn assemble_shared_property_probe(
    semantic: &SemanticProgram,
    metal_compilation: &Compilation,
    metal_plan: PlanAlternative<'_>,
    cpu_compilation: &Compilation,
    cpu_plan: PlanAlternative<'_>,
    shared: TargetProfileRef,
    metal_content: &PayloadContent,
    cpu: &ProducedCpu,
    probe: PreparedEntryProbe,
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
        Some(probe),
    )
    .map_err(|error| PortfolioError::Assemble(error.to_string()))?;
    package(artifact)
}

/// Packages a CPU-only control while perturbing its prepared-entry request.
pub fn assemble_cpu_property_probe(
    semantic: &SemanticProgram,
    compilation: &Compilation,
    plan: PlanAlternative<'_>,
    profile: TargetProfileRef,
    cpu: &ProducedCpu,
    probe: PreparedEntryProbe,
) -> Result<PackagedPortfolio, PortfolioError> {
    let environment =
        CompilationEnvironment::new(compilation.offered_lowering_providers().iter().cloned())
            .map_err(|error| PortfolioError::Assemble(error.to_string()))?;
    let mut draft = ArtifactProgramBuilder::new(semantic, environment)
        .map_err(|error| PortfolioError::Assemble(error.to_string()))?;
    select_capabilities(&mut draft, plan)
        .map_err(|error| PortfolioError::Assemble(error.to_string()))?;
    let payload = draft
        .push_carried_payload(
            cpu::backend(),
            cpu::representation(),
            cpu::payload_schema(),
            profile.clone(),
            ArtifactExecutionPolicy::NativeImage,
            cpu.content.clone(),
        )
        .map_err(|error| PortfolioError::Assemble(error.to_string()))?;
    push_plan_variant(
        &mut draft,
        compilation,
        plan,
        profile.clone(),
        payload,
        Some(probe),
    )
    .map_err(|error| PortfolioError::Assemble(error.to_string()))?;
    let entries = u32::try_from(plan.abi().kernel_program().stages().len())
        .expect("a bounded entry table fits u32");
    draft
        .declare_realization(
            realization::translate(plan.delivered_realization(), &profile, entries)
                .map_err(|_| PortfolioError::Assemble("realization profile mismatch".into()))?,
        )
        .map_err(|error| PortfolioError::Assemble(error.to_string()))?;
    let artifact = draft
        .build()
        .map_err(|error| PortfolioError::Assemble(error.to_string()))?;
    package(artifact)
}

fn package(artifact: VerifiedArtifactProgram) -> Result<PackagedPortfolio, PortfolioError> {
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
    cpu_probe: Option<PreparedEntryProbe>,
) -> Result<VerifiedArtifactProgram, ArtifactBuildError> {
    let environment = CompilationEnvironment::new(
        metal_compilation
            .offered_lowering_providers()
            .iter()
            .cloned(),
    )?;
    let mut draft = ArtifactProgramBuilder::new(semantic, environment)?;
    select_capabilities(&mut draft, metal_plan)?;

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
        None,
    )?;
    push_plan_variant(
        &mut draft,
        cpu_compilation,
        cpu_plan,
        cpu_profile,
        cpu_payload,
        cpu_probe,
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

fn select_capabilities(
    draft: &mut ArtifactProgramBuilder,
    plan: PlanAlternative<'_>,
) -> Result<(), ArtifactBuildError> {
    for selected in plan.selected_capabilities() {
        let subject = selected.subject();
        draft.select_provider(SelectedProvider {
            provider: selected.provider().clone(),
            capability: LoweringCapabilitySubject {
                family: CapabilityFamilyKey::new(subject.family().key_token())?,
                operation: subject.operation().clone(),
            },
            capability_revision: selected.capability_revision(),
        })?;
    }
    Ok(())
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
    probe: Option<PreparedEntryProbe>,
) -> Result<(), ArtifactBuildError> {
    let program = plan.abi().kernel_program();
    let requirements = plan.prepared_entry_target_requirements();
    if probe.is_some() {
        assert_eq!(
            requirements.len(),
            1,
            "a prepared-entry probe must mutate the spike's one governed request",
        );
    }
    let deferred_predicates = requirements
        .map(|requirement| DeferredPredicateSpec {
            requirement: probe.map_or_else(
                || requirement.requirement().clone(),
                |probe| perturb_requirement(requirement.requirement(), probe),
            ),
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

fn perturb_requirement(
    requirement: &PreparedEntryTargetRequirement,
    probe: PreparedEntryProbe,
) -> PreparedEntryTargetRequirement {
    let original = requirement.query();
    let original_provider = original.provider();
    let key = match probe {
        PreparedEntryProbe::UnknownProperty => {
            TargetPropertyKey::new("tiler.target.prepared-entry.unknown-property.v1")
        }
        PreparedEntryProbe::UnknownProvider | PreparedEntryProbe::RequiredAboveObserved => {
            TargetPropertyKey::new(original.key().as_str())
        }
    }
    .expect("each probe property key is valid");
    let provider = match probe {
        PreparedEntryProbe::UnknownProvider => {
            TargetPropertyProviderIdentity::new("acme", "prepared-entry-properties", 1)
        }
        PreparedEntryProbe::UnknownProperty | PreparedEntryProbe::RequiredAboveObserved => {
            TargetPropertyProviderIdentity::new(
                original_provider.namespace(),
                original_provider.name(),
                original_provider.revision(),
            )
        }
    }
    .expect("each probe provider identity is valid");
    let query = TargetPropertyQuery::new(key, original.available_at(), provider)
        .expect("each probe remains a deferred query");
    let required = match probe {
        PreparedEntryProbe::RequiredAboveObserved => 1_025,
        PreparedEntryProbe::UnknownProvider | PreparedEntryProbe::UnknownProperty => {
            requirement.required()
        }
    };
    PreparedEntryTargetRequirement::new(query, required, requirement.relation())
        .expect("each probe remains a prepared-entry requirement")
}

//! The governed budgets an artifact envelope is encoded and parsed under.
//!
//! Every budget is enforced on both sides and by the same constant. The
//! encoder checks a projected envelope up front so a legally built artifact
//! that could not be read back fails to encode rather than producing bytes no
//! reader admits; the decoder checks each count the moment it is read and
//! before anything is allocated for it, so a forged length cannot make a reader
//! reserve memory for content that does not exist.
//!
//! Where the artifact model already governs a quantity, that constant is
//! reused rather than restated. A codec-local bound would be a second authority
//! for the same limit, and the two would drift.

use super::super::error::ArtifactBuildError;
use super::super::model::BindingTargetData;
use super::super::requirement::RouteRequirement;
use super::super::{
    MAX_ABI_EXPRESSIONS, MAX_ARTIFACT_PAYLOADS, MAX_ARTIFACT_VARIANTS, MAX_DEFERRED_PREDICATES,
    MAX_ENTRY_BINDINGS, MAX_LAUNCH_PRECONDITIONS, MAX_ROUTE_FEATURE_PAYLOAD_BYTES,
    MAX_ROUTE_REQUIREMENTS, MAX_SELECTED_LOWERING_PROVIDERS, MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS,
    MAX_VARIANT_ENTRIES,
};
use super::error::{ArtifactCodecError, CodecLimitKind, codec_limit};
use super::model::{
    ArtifactEnvelope, MAX_FEATURES, MAX_INTERFACE_ENTRIES, MAX_INTERFACE_SHAPE_RANK,
    MAX_SECTION_BYTES, MAX_SECTIONS, MAX_TEXT_BYTES,
};
use tiler_ir::numerics::{CompilerBuildRole, FactEvidenceBasis, FactSourceProvenance};
use tiler_ir::semantic::OutputKey;

/// Proves one projected envelope fits every governed encoder budget.
///
/// # Errors
///
/// Returns [`ArtifactCodecError::Limit`] naming the first exhausted resource
/// with its attempted and permitted quantities.
pub(super) fn check_budgets(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    check_text_budgets(envelope)?;
    codec_limit(
        envelope.features().len(),
        MAX_FEATURES,
        CodecLimitKind::Features,
    )?;
    codec_limit(
        envelope.inputs().len(),
        MAX_INTERFACE_ENTRIES,
        CodecLimitKind::InterfaceEntries,
    )?;
    codec_limit(
        envelope.outputs().len(),
        MAX_INTERFACE_ENTRIES,
        CodecLimitKind::InterfaceEntries,
    )?;
    for rank in envelope
        .inputs()
        .iter()
        .map(super::super::model::InterfaceEntryData::rank)
        .chain(
            envelope
                .outputs()
                .iter()
                .map(super::super::model::InterfaceEntryData::rank),
        )
    {
        codec_limit(rank, MAX_INTERFACE_SHAPE_RANK, CodecLimitKind::ShapeRank)?;
    }
    codec_limit(
        envelope.providers().len(),
        MAX_SELECTED_LOWERING_PROVIDERS,
        CodecLimitKind::SelectedLoweringProviders,
    )?;
    codec_limit(
        envelope.payloads().len(),
        MAX_ARTIFACT_PAYLOADS,
        CodecLimitKind::Payloads,
    )?;
    for payload in envelope.payloads() {
        if let Some(declaration) = &payload.environment {
            codec_limit(
                declaration.descriptor().as_bytes().len(),
                super::super::MAX_TARGET_ENVIRONMENT_DESCRIPTOR_BYTES,
                CodecLimitKind::TargetEnvironmentDescriptorBytes,
            )?;
        }
    }
    codec_limit(
        envelope.expressions().len(),
        MAX_ABI_EXPRESSIONS,
        CodecLimitKind::Expressions,
    )?;
    codec_limit(
        envelope.variants().len(),
        MAX_ARTIFACT_VARIANTS,
        CodecLimitKind::Variants,
    )?;
    codec_limit(
        envelope.sections().len(),
        MAX_SECTIONS,
        CodecLimitKind::Sections,
    )?;
    for section in envelope.sections() {
        codec_limit(
            section.bytes.len(),
            MAX_SECTION_BYTES,
            CodecLimitKind::SectionBytes,
        )?;
    }
    for variant in envelope.variants() {
        codec_limit(
            variant.entries.len(),
            MAX_VARIANT_ENTRIES,
            CodecLimitKind::Entries,
        )?;
        // The count the decoder will check before it reserves the row vector,
        // repeated here so a legally built artifact that no reader could admit
        // fails to encode rather than producing bytes nothing accepts.
        codec_limit(
            variant.selected_physical_implementations.len(),
            MAX_SELECTED_PHYSICAL_IMPLEMENTATIONS,
            CodecLimitKind::SelectedPhysicalImplementations,
        )?;
        // The relational rule beside its absolute one, for the same reason: the
        // decoder refuses a run that outnumbers the entry table, so an encoder
        // that emitted one would write bytes it could not read back.
        if variant.selected_physical_implementations.len() > variant.entries.len() {
            return Err(ArtifactCodecError::ModelRule {
                cause: Box::new(ArtifactBuildError::PhysicalSelectionCardinality {
                    selected: variant.selected_physical_implementations.len(),
                    entries: variant.entries.len(),
                }),
            });
        }
        codec_limit(
            variant.deferred.len(),
            MAX_DEFERRED_PREDICATES,
            CodecLimitKind::DeferredPredicates,
        )?;
        codec_limit(
            variant.route_requirements.len(),
            MAX_ROUTE_REQUIREMENTS,
            CodecLimitKind::RouteRequirements,
        )?;
        codec_limit(
            variant.scope.len(),
            super::super::MAX_DELIVERY_POSITIONS,
            CodecLimitKind::PlanDeterminismScopeCells,
        )?;
        for requirement in &variant.route_requirements {
            if let RouteRequirement::BackendFeature(feature) = requirement {
                codec_limit(
                    feature.payload().len(),
                    MAX_ROUTE_FEATURE_PAYLOAD_BYTES,
                    CodecLimitKind::RouteFeaturePayloadBytes,
                )?;
            }
        }
        for entry in &variant.entries {
            codec_limit(
                entry.bindings.len(),
                MAX_ENTRY_BINDINGS,
                CodecLimitKind::EntryBindings,
            )?;
            codec_limit(
                entry.input_extents.len(),
                super::super::MAX_ENTRY_EXTENTS,
                CodecLimitKind::EntryExtents,
            )?;
            codec_limit(
                entry.launch.preconditions.len(),
                MAX_LAUNCH_PRECONDITIONS,
                CodecLimitKind::LaunchPreconditions,
            )?;
            // A binding target names published output storage under every key
            // that publishes it, so its budget is the interface's own: a
            // reference cannot name more outputs than the artifact declares.
            for binding in &entry.bindings {
                if let BindingTargetData::ProgramOutput(keys) = &binding.target {
                    codec_limit(
                        keys.len(),
                        MAX_INTERFACE_ENTRIES,
                        CodecLimitKind::BindingTargetKeys,
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// Proves every encoded text run fits the reader's per-run budget.
///
/// Most of these are governed keys whose own constructors already bound them
/// well below the parser budget, and checking them again costs nothing. Two
/// families are not. A numerical realization's profile key is a `&'static str`
/// chosen by the producing build (`tiler_ir::schedule::NumericalRealization`),
/// so nothing bounds it before it reaches this encoder. And the
/// delivered-realization record's provenance text — the authority and guarantee
/// identities, each compiler build's implementation, version and build string,
/// and each execution environment's five fields — arrives from whichever profile
/// declared the fact, bounded by `tiler_ir::numerics`'s own
/// `MAX_PROVENANCE_TEXT_BYTES` completeness rule rather than by this manifest's.
/// Without these rows an artifact could encode a text run no other manifest
/// field is permitted to carry, which would break the symmetry the encoder's
/// documentation claims.
fn check_text_budgets(envelope: &ArtifactEnvelope) -> Result<(), ArtifactCodecError> {
    let mut texts: Vec<&str> = Vec::new();
    let record = envelope.realization();
    texts.push(record.profile().key.as_str());
    for row in record.evidence() {
        texts.push(row.profile().key.as_str());
        push_provenance_text(&mut texts, row.source());
    }
    texts.extend(envelope.features().iter().map(String::as_str));
    texts.extend(envelope.inputs().iter().map(|input| input.key.as_str()));
    texts.extend(envelope.outputs().iter().map(|output| output.key.as_str()));
    for provider in envelope.providers() {
        texts.push(provider.provider.namespace());
        texts.push(provider.provider.name());
        texts.push(provider.capability.family.as_str());
        texts.push(provider.capability.operation.namespace());
        texts.push(provider.capability.operation.name());
    }
    for payload in envelope.payloads() {
        texts.push(payload.backend.as_str());
        texts.push(payload.representation.as_str());
        if let Some(declaration) = &payload.environment {
            texts.push(declaration.provider().namespace());
            texts.push(declaration.provider().name());
        }
    }
    for variant in envelope.variants() {
        texts.push(variant.profile.key.as_str());
        texts.push(variant.feasibility_rules.key.as_str());
        for predicate in &variant.deferred {
            texts.push(predicate.requirement.query().key().as_str());
            texts.push(predicate.requirement.query().provider().namespace());
            texts.push(predicate.requirement.query().provider().name());
        }
        for requirement in &variant.route_requirements {
            if let RouteRequirement::BackendFeature(feature) = requirement {
                texts.push(feature.owner().as_str());
                texts.push(feature.key().as_str());
            }
        }
        for entry in &variant.entries {
            texts.push(entry.numerical.profile_key.as_str());
            // The same interface keys as above, reached through the binding
            // targets. Listed rather than assumed equal: the equality is
            // `super::validate`'s obligation and it has not run when the encoder
            // checks its budgets, so relying on it here would be a check whose
            // premise is proven later.
            for binding in &entry.bindings {
                match &binding.target {
                    BindingTargetData::ProgramInput(key) => texts.push(key.as_str()),
                    BindingTargetData::ProgramOutput(keys) => {
                        texts.extend(keys.iter().map(OutputKey::as_str));
                    }
                    BindingTargetData::Internal => {}
                }
            }
        }
    }
    for text in texts {
        codec_limit(text.len(), MAX_TEXT_BYTES, CodecLimitKind::TextBytes)?;
    }
    Ok(())
}

/// Collects every text run one provenance statement writes.
///
/// Exhaustive over the basis vocabulary with no wildcard arm, so a fourth
/// evidence basis carrying text stops the build here rather than encoding a run
/// this budget never saw.
fn push_provenance_text<'a>(texts: &mut Vec<&'a str>, source: &'a FactSourceProvenance) {
    texts.push(source.authority_identity().key());
    match source.basis() {
        FactEvidenceBasis::GovernedGuarantee { guarantee } => texts.push(guarantee.key()),
        FactEvidenceBasis::ExternalGuarantee { reference } => texts.push(reference.key()),
        FactEvidenceBasis::Measurement { contexts } => {
            for context in contexts {
                push_context_text(texts, context.compiler_builds(), context.environment());
            }
        }
        // The compilation selection is opaque bytes rather than text, and its
        // own 64-KiB ceiling is enforced at construction; only the textual
        // fields join this budget.
        FactEvidenceBasis::CompileProfileMeasurement { contexts } => {
            for context in contexts {
                push_context_text(texts, context.compiler_builds(), context.environment());
            }
        }
    }
}

/// Collects the text runs one measurement context writes, either route.
fn push_context_text<'a>(
    texts: &mut Vec<&'a str>,
    compiler_builds: &'a [tiler_ir::numerics::CompilerBuildIdentity],
    environment: &'a tiler_ir::numerics::ExecutionEnvironmentIdentity,
) {
    for build in compiler_builds {
        if let CompilerBuildRole::ProviderDefined(identity) = build.role() {
            texts.push(identity.key());
        }
        texts.push(build.implementation());
        texts.push(build.version());
        texts.extend(build.build());
    }
    texts.push(environment.platform());
    texts.push(environment.platform_version());
    texts.push(environment.platform_build());
    texts.push(environment.architecture());
    texts.push(environment.hardware());
}

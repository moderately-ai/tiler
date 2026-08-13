//! Compile the shared program against the Apple Metal profile.

use tiler_build::BoundMetalCompileDeclaration;
use tiler_compiler::physical_provider::InstalledPhysicalProviders;
use tiler_compiler::session::{
    Compilation, CompileRequest, NumericalContract, PlanAlternative, compile,
};
use tiler_compiler::target::TargetRequest;
use tiler_ir::semantic::{ProviderIdentity, SemanticProgram};

use crate::provider::{self, AcmeProvider};

/// Tiler's own governed physical provider, spelled out.
///
/// The boundary does not export it. Presence checks compare against this
/// constant so a rename fails the assertion rather than silently weakening it.
pub const GOVERNED_PHYSICAL_PROVIDER: (&str, &str, u32) =
    ("tiler", "prototype-serial-sum-physical", 1);

/// One compilation of the shared program and the provenance this spike records.
pub struct CompiledProgram {
    /// The bound Apple Metal declaration every compilation here is assessed under.
    pub declaration: BoundMetalCompileDeclaration,
    /// The compilation produced with the custom physical provider installed.
    pub with_custom: Compilation,
    /// The compilation produced with only the governed physical provider.
    pub without_custom: Compilation,
}

/// Compiles the shared program twice: with and without the custom provider.
pub fn compile_portfolio(program: &SemanticProgram) -> Result<CompiledProgram, String> {
    let declaration = BoundMetalCompileDeclaration::first_macos_apple9()
        .map_err(|error| format!("metal declaration: {error}"))?;
    let profile = declaration.profile().clone();
    let custom = AcmeProvider::new();
    let installed = InstalledPhysicalProviders::installed([&custom as _])
        .map_err(|error| format!("install physical providers: {error}"))?;
    let with_custom = compile(
        CompileRequest::new(
            program,
            NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
            TargetRequest::new([profile.clone()])
                .map_err(|error| format!("target request: {error}"))?,
        )
        .with_physical_providers(installed),
    )
    .map_err(|error| format!("compile with custom provider: {error}"))?
    .into_targets()
    .pop()
    .ok_or_else(|| "compile with custom provider returned no target".to_owned())?
    .into_parts()
    .1
    .map_err(|error| format!("compile with custom provider target: {error}"))?;

    let without_custom = compile(CompileRequest::new(
        program,
        NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32,
        TargetRequest::new([profile]).map_err(|error| format!("target request: {error}"))?,
    ))
    .map_err(|error| format!("compile without custom provider: {error}"))?
    .into_targets()
    .pop()
    .ok_or_else(|| "compile without custom provider returned no target".to_owned())?
    .into_parts()
    .1
    .map_err(|error| format!("compile without custom provider target: {error}"))?;

    Ok(CompiledProgram {
        declaration,
        with_custom,
        without_custom,
    })
}

/// Returns the governed physical-provider identity.
#[must_use]
pub fn governed_identity() -> ProviderIdentity {
    let (namespace, name, revision) = GOVERNED_PHYSICAL_PROVIDER;
    ProviderIdentity::new(namespace, name, revision).expect("the governed identity is well formed")
}

/// Records offered and selected physical-provider identities for one compilation.
#[must_use]
pub fn physical_provenance(compilation: &Compilation) -> PhysicalProvenance {
    let offered = compilation
        .offered_physical_providers()
        .iter()
        .map(ToString::to_string)
        .collect();
    let alternatives = compilation
        .alternatives()
        .map(|alternative| AlternativeProvenance {
            stable_id: alternative.stable_id().to_owned(),
            selected: alternative
                .selected_physical_providers()
                .map(|selected| selected.provider().to_string())
                .collect(),
        })
        .collect();
    PhysicalProvenance {
        offered,
        alternatives,
    }
}

/// Offered and selected physical-provider identities for one compilation.
#[derive(Clone, Debug)]
pub struct PhysicalProvenance {
    /// The complete frozen physical-provider environment.
    pub offered: Vec<String>,
    /// Per retained alternative, the providers that alternative selected.
    pub alternatives: Vec<AlternativeProvenance>,
}

/// One retained plan alternative's selected physical providers.
#[derive(Clone, Debug)]
pub struct AlternativeProvenance {
    /// The alternative's stable identifier.
    pub stable_id: String,
    /// Providers selected for this alternative, in region order.
    pub selected: Vec<String>,
}

/// Returns whether the custom provider appears in a compilation's offered set.
#[must_use]
pub fn offered_custom(compilation: &Compilation) -> bool {
    compilation
        .offered_physical_providers()
        .iter()
        .any(|identity| identity == &provider::identity())
}

/// Returns whether any retained alternative selected the custom provider.
#[must_use]
pub fn selected_custom(compilation: &Compilation) -> bool {
    compilation.alternatives().any(|alternative| {
        alternative
            .selected_physical_providers()
            .any(|selected| selected.provider() == &provider::identity())
    })
}

/// Returns whether the governed provider is still named by a retained plan.
#[must_use]
pub fn selected_governed(compilation: &Compilation) -> bool {
    let governed = governed_identity();
    compilation.alternatives().any(|alternative| {
        alternative
            .selected_physical_providers()
            .any(|selected| selected.provider() == &governed)
    })
}

/// Returns the selected plan of one compilation.
pub fn selected_plan(compilation: &Compilation) -> Result<PlanAlternative<'_>, String> {
    compilation
        .selected()
        .ok_or_else(|| "compilation retained no selected alternative".to_owned())
}

//! Route requirement rows and the artifact that declares them.

use super::super::super::BackendFeatureRequirement;
use super::super::super::{
    ArtifactProgramBuilder, BackendKey, CompilationEnvironment, RouteFeatureKey, RouteRequirement,
    RouteResourceDimension, RouteResourceRequirement, VerifiedArtifactProgram,
};
use super::artifacts::{
    declare_realization, formulas, lowering_provider, payload, selection, variant,
};
use super::graphs::{SCALE_BITS, semantic_program};
use super::kernels::fused_program;

// -------------------------------------------------------------------------
// Live-device route requirements
// -------------------------------------------------------------------------

/// Builds one well-formed backend feature row in the Metal namespace.
pub(crate) fn route_feature(key: &str, version: u32, payload: &[u8]) -> RouteRequirement {
    RouteRequirement::BackendFeature(
        BackendFeatureRequirement::new(
            BackendKey::new("tiler.metal").expect("a governed backend key"),
            RouteFeatureKey::new(key).expect("a governed route feature key"),
            version,
            payload,
        )
        .expect("a well-formed backend feature requirement"),
    )
}

/// Builds one well-formed quantitative row.
pub(crate) fn route_resource(required: u64) -> RouteRequirement {
    RouteRequirement::Resource(
        RouteResourceRequirement::new(RouteResourceDimension::SubgroupThreads, required)
            .expect("a nonzero required quantity"),
    )
}

/// Assembles the canonical artifact with route requirements attached.
///
/// Declaration order is the caller's, which is what makes the canonical-order
/// cases meaningful: the builder retains it and the envelope projection is where
/// it stops mattering.
pub(crate) fn requiring_artifact(requirements: &[RouteRequirement]) -> VerifiedArtifactProgram {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let variant = draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    for requirement in requirements {
        draft
            .require_route(variant, requirement.clone())
            .expect("each requirement names a distinct subject");
    }
    declare_realization(&mut draft, &program);
    draft.build().unwrap()
}

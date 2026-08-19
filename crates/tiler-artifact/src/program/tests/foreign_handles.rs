//! Cross-program subjects and handles another builder minted are refused.

use super::super::{
    ArtifactBuildError, ArtifactEntityKind, ArtifactProgramBuilder, CompilationEnvironment,
};
use super::{
    SCALE_BITS, build_graph_scaled, formulas, fused_program, lowering_provider, payload, selection,
    semantic_program, variant,
};
use tiler_ir::semantic::SemanticProgramBuilder;

// -------------------------------------------------------------------------
// Cross-program and forged-input rejection
// -------------------------------------------------------------------------

#[test]
fn rejects_a_variant_realizing_another_semantic_graph() {
    let packaged = semantic_program();
    let other = build_graph_scaled(SemanticProgramBuilder::try_standard().unwrap(), 3.0);
    assert_ne!(
        packaged.semantic_identity().graph(),
        other.semantic_identity().graph(),
    );
    let foreign = fused_program(&other, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&packaged, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    assert_eq!(
        draft.push_variant(&foreign, variant(&formulas, descriptor, b"fused")),
        Err(ArtifactBuildError::SemanticSubjectMismatch),
    );
}

#[test]
fn rejects_an_expression_handle_from_another_builder() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut donor = ArtifactProgramBuilder::new(&semantic, environment.clone()).unwrap();
    let donor_formulas = formulas(&mut donor);
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, descriptor, b"fused");
    // Injected through a launch precondition, which is still caller-supplied.
    // The guard and launch geometry are derived from the program now, so they
    // are no longer a way to hand the builder a foreign handle at all.
    spec.entries[0]
        .launch
        .preconditions
        .push(donor_formulas.always);
    assert_eq!(
        draft.push_variant(&program, spec),
        Err(ArtifactBuildError::ForeignHandle {
            entity: ArtifactEntityKind::Expression,
        }),
    );
}

#[test]
fn rejects_a_payload_handle_from_another_builder() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()]).unwrap();
    let mut donor = ArtifactProgramBuilder::new(&semantic, environment.clone()).unwrap();
    let donor_payload = donor.push_payload(payload(0xa1)).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_provider(selection(provider)).unwrap();
    draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    assert_eq!(
        draft.push_variant(&program, variant(&formulas, donor_payload, b"fused")),
        Err(ArtifactBuildError::ForeignHandle {
            entity: ArtifactEntityKind::Payload,
        }),
    );
}

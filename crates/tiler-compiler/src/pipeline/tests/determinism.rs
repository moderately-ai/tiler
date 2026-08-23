use super::support::{alternative, selected_kind, semantic};
use super::*;

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "keeps the exact explain snapshot beside the end-to-end product invariants"
)]
fn product_is_deterministic_and_preserves_the_materialized_boundary() {
    let first = semantic(false);
    let second = semantic(true);
    assert_eq!(
        first.semantic_identity().graph(),
        second.semantic_identity().graph()
    );
    let occurrence_count = first.operations().count();
    crate::lowering::reset_refinement_proof_work();
    let first = compile(CompilationRequest::governed(&first)).unwrap();
    assert_eq!(
        crate::lowering::refinement_proof_work(),
        occurrence_count * 2,
        "each occurrence is refined once by planning and once by the independent portfolio verifier"
    );
    crate::lowering::reset_refinement_proof_work();
    let second = compile(CompilationRequest::governed(&second)).unwrap();
    assert_eq!(
        crate::lowering::refinement_proof_work(),
        occurrence_count * 2,
        "two retained alternatives must not multiply verifier proof work"
    );

    assert_eq!(first, second);
    for kind in [
        ProgramAlternativeKind::Materialized,
        ProgramAlternativeKind::Fused,
    ] {
        let forward = alternative(&first, kind);
        let reversed = alternative(&second, kind);
        let coverage = |alternative: &ProgramAlternative| {
            alternative
                .program
                .core()
                .stages()
                .map(|stage| stage.coverage().to_vec())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            coverage(forward),
            coverage(reversed),
            "{kind:?} stage coverage changed with authoring order"
        );
    }
    let target = &first.targets[0];
    let rendered = target.explain.render();
    assert!(rendered.starts_with("tiler-explain-v10 request="));
    assert!(rendered.contains("feasibility:threads-per-workgroup:deferred"));
    assert!(rendered.contains("feasibility:buffer-bindings:admitted"));
    assert!(rendered.contains("event=selection:tiler.selection.structural-pareto.v1:selected"));
    assert_eq!(target.portfolio.alternatives.len(), 2);
    assert_eq!(selected_kind(&first), ProgramAlternativeKind::Fused);
    let materialized = alternative(&first, ProgramAlternativeKind::Materialized);
    let fused = alternative(&first, ProgramAlternativeKind::Fused);
    assert_eq!(materialized.program.stage_count(), 2);
    let temporary = materialized
        .program
        .core()
        .values()
        .nth(1)
        .expect("the cross-stage temporary");
    assert_eq!(temporary.role(), ValueRole::Temporary);
    assert!(matches!(
        materialized
            .program
            .core()
            .dependencies()
            .next()
            .expect("one data dependency")
            .reason(),
        DependencyReasonView::Data(value) if value == temporary
    ));
    assert_eq!(
        materialized.kernels[0].buffers().nth(1).unwrap().tensor,
        TensorRole::Intermediate
    );
    assert_eq!(
        materialized.kernels[1].buffers().next().unwrap().tensor,
        TensorRole::Intermediate
    );
    assert_eq!(reduction_loop(&materialized.kernels[1]), Some((1, 2)));
    assert_eq!(fused.program.stage_count(), 1);
    assert_eq!(fused.program.core().values().len(), 2);
    // The exact aggregate structural cost is the sum of the per-region
    // estimates plus the cover's deliberate cross-region materializations.
    assert_eq!(materialized.structural_cost.dispatch_count(), 2);
    assert_eq!(materialized.structural_cost.launched_threads(), 6);
    assert_eq!(materialized.structural_cost.temporary_bytes(), 16);
    assert_eq!(materialized.structural_cost.materialization_count(), 1);
    assert_eq!(fused.structural_cost.dispatch_count(), 1);
    assert_eq!(fused.structural_cost.launched_threads(), 2);
    assert_eq!(fused.structural_cost.temporary_bytes(), 0);
    assert_eq!(fused.structural_cost.materialization_count(), 0);
    assert!(
        fused
            .structural_cost
            .dominates(&materialized.structural_cost)
    );
    // Lowering provenance is the set of providers the installed registry
    // resolved for the recognized occurrences. Both plan shapes cover the
    // same occurrences, so both name the same four governed providers: the
    // alternatives differ in their cover, not in who lowers each operation.
    // Provider and operation are named separately rather than one derived
    // from the other: they coincide by naming convention in the governed
    // registry, and a test that split the provider name would assert the
    // convention instead of the resolution.
    let expected_providers: Vec<_> = [
        ("governed-index-access.add-f32", "add-f32"),
        ("governed-index-access.constant-f32", "constant-f32"),
        ("governed-index-access.multiply-f32", "multiply-f32"),
        (
            "governed-index-access.strict-serial-sum-f32",
            "strict-serial-sum-f32",
        ),
    ]
    .into_iter()
    .map(|(provider, operation)| {
        crate::request::LoweringProviderIdentity::new(
            tiler_ir::semantic::ProviderIdentity::new("tiler", provider, 1).unwrap(),
            crate::capability::LoweringCapabilitySubject::new(
                crate::capability::LoweringFamily::IndexAccess,
                tiler_ir::semantic::OpKey::new("tiler", operation, 1).unwrap(),
            ),
            crate::capability::LoweringCapabilityRevision::new(1).unwrap(),
        )
    })
    .collect();
    assert_eq!(
        materialized.artifact_plan.lowering_providers(),
        expected_providers
    );
    assert_eq!(fused.artifact_plan.lowering_providers(), expected_providers);
    assert_eq!(reduction_loop(&fused.kernels[0]), Some((1, 2)));
    assert!(target.explain.records().iter().any(|record| {
        record.rule().key().as_str() == "compile.plan.boundary"
            && record.event().disposition() == ExplainDisposition::Admitted
    }));
    // The materialized plan discharges exactly one cross-region handoff; the
    // fused plan materializes nothing across a boundary.
    assert_eq!(materialized.plan.handoffs().len(), 1);
    assert!(fused.plan.handoffs().is_empty());
    // Stable identity binds the semantic origin and request contract as well as
    // the selected physical plan.
    for alternative in &target.portfolio.alternatives {
        assert_eq!(alternative.stable_id, alternative.identity.label());
    }
}

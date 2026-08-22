//! One case per whole-artifact rule, with the delivery positives they need.

use super::super::ScalarArithmeticSubject;
use super::super::{
    AbiRoot, ArtifactBuildError, ArtifactDiagnostic, ArtifactProgramBuilder, CompilationEnvironment,
};
use super::support::artifacts::partial_window_variant;
use super::{
    OTHER_SCALE_BITS, SCALE_BITS, declare_realization, declare_realization_over, formulas,
    fused_program, lowering_provider, partial_window_program, payload, profile, realization_record,
    selection, semantic_program, strict, variant,
};

// -------------------------------------------------------------------------
// Negative tests, one per whole-artifact rule
// -------------------------------------------------------------------------

#[test]
fn rejects_an_empty_portfolio() {
    let semantic = semantic_program();
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    // A record with no entry binding, because there is no packaged entry to
    // bind: what is under test is the empty portfolio, and a record naming an
    // entry this draft does not have would be refused for that instead.
    draft
        .declare_realization(realization_record(
            &profile(),
            &ScalarArithmeticSubject::f32(),
            strict(),
            0,
        ))
        .expect("a record over no packaged entry");
    let diagnostics = draft.build().expect_err("an empty portfolio is rejected");
    assert!(
        diagnostics
            .diagnostics()
            .contains(&ArtifactDiagnostic::EmptyPortfolio)
    );
}

#[test]
fn rejects_an_artifact_that_selected_no_provider() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let environment = CompilationEnvironment::new([lowering_provider(1)], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    declare_realization(&mut draft, &program);
    let error = draft.build().expect_err("unattributed plans are rejected");
    assert_eq!(
        error.diagnostics(),
        [ArtifactDiagnostic::MissingSelectedLoweringProvider],
    );
    // The builder comes back intact and the failure is recoverable.
    let (mut recovered, _) = error.into_parts();
    recovered
        .select_lowering_provider(selection(lowering_provider(1)))
        .unwrap();
    assert_eq!(
        recovered
            .build()
            .unwrap()
            .selected_lowering_providers()
            .len(),
        1
    );
}

#[test]
fn rejects_an_expression_no_use_site_reaches() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft.push_root(AbiRoot::UnsignedLiteral(999)).unwrap();
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    declare_realization(&mut draft, &program);
    assert_eq!(
        draft
            .build()
            .expect_err("an unreachable node is rejected")
            .diagnostics(),
        [ArtifactDiagnostic::UnusedExpression],
    );
}

#[test]
fn rejects_a_payload_no_entry_realizes() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    draft.push_payload(payload(0xb1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&program, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    declare_realization(&mut draft, &program);
    assert_eq!(
        draft
            .build()
            .expect_err("an unreferenced payload is rejected")
            .diagnostics(),
        [ArtifactDiagnostic::UnusedPayload],
    );
}

/// Two delivery positions carry two objects realizing the same entries.
///
/// The positive case the whole delivery-position record exists for: a selection
/// built for two consumer targets is one plan, one kernel program, and two
/// compiled objects, so the entry names one backend entry key at two payloads
/// and both are referenced. Identity folds the run *as stated*, which the
/// sibling case below turns into a measurement.
#[test]
fn packages_one_payload_per_delivery_position() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let first = draft.push_payload(payload(0xa1)).unwrap();
    let second = draft.push_payload(payload(0xb1)).unwrap();
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, first, b"fused");
    spec.entries[0].implementation.payloads = vec![first, second];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    let artifact = draft.build().expect("both positions are realized");

    assert_eq!(artifact.delivery_positions(), 2);
    assert_eq!(artifact.payloads().len(), 2);
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    assert_eq!(entry.payloads().len(), 2);
    assert_ne!(
        entry.payload(0).expect("position 0").digest,
        entry.payload(1).expect("position 1").digest,
        "one payload per built family means two objects, not one under two names",
    );
    assert!(entry.payload(2).is_none());
}

/// A one-position artifact and a two-position one are never one artifact.
///
/// The identity consequence the `tiler.artifact-program.v13` step exists for.
/// The one-position artifact below carries the *first* of the two-position
/// artifact's payloads, so the two differ only in whether a second family was
/// built — and an identity that folded only the sorted payload table would
/// still distinguish them. What this pins is stronger and is the property a
/// cache needs: the *order* is folded too, so the same two payloads delivered
/// the other way round is a third artifact.
#[test]
fn delivery_order_and_count_are_both_artifact_identity() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);

    let build = |positions: &[u8]| {
        let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
        let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
        draft
            .select_lowering_provider(selection(provider.clone()))
            .unwrap();
        let declared: Vec<_> = positions
            .iter()
            .map(|tag| draft.push_payload(payload(*tag)).unwrap())
            .collect();
        let formulas = formulas(&mut draft);
        let mut spec = variant(&formulas, declared[0], b"fused");
        spec.entries[0].implementation.payloads = declared;
        draft.push_variant(&program, spec).unwrap();
        declare_realization(&mut draft, &program);
        draft
            .build()
            .expect("every declared payload is realized")
            .canonical_identity()
            .as_bytes()
            .to_vec()
    };

    let one = build(&[0xa1]);
    let two = build(&[0xa1, 0xb1]);
    let reversed = build(&[0xb1, 0xa1]);
    assert_ne!(one, two, "a second family is a second artifact");
    assert_ne!(two, reversed, "delivery order is meaning");
}

/// An entry naming no payload has no consumer target that could dispatch it.
#[test]
fn rejects_an_entry_realized_at_no_delivery_position() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.entries[0].implementation.payloads = Vec::new();
    assert_eq!(
        draft.push_variant(&program, spec),
        Err(ArtifactBuildError::EmptyDelivery { entry: 0 }),
    );
}

/// Every entry of an artifact is realized at the same delivery positions.
///
/// A consumer resolves one position for the whole artifact, so an entry short of
/// it would leave that consumer with no object for a stage its route must
/// dispatch. The two-stage program makes the disagreement expressible: entry 0
/// establishes two positions and entry 1 declares one.
#[test]
fn rejects_an_entry_disagreeing_about_delivery_positions() {
    let semantic = semantic_program();
    let program = partial_window_program(&semantic);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let first = draft.push_payload(payload(0xa1)).unwrap();
    let second = draft.push_payload(payload(0xb1)).unwrap();
    let mut spec = partial_window_variant(first);
    spec.entries[0].implementation.payloads = vec![first, second];
    assert_eq!(
        draft.push_variant(&program, spec),
        Err(ArtifactBuildError::DeliveryCardinality {
            entry: 1,
            expected: 2,
            actual: 1,
        }),
    );
}

/// One payload may not stand in for two consumer build targets.
///
/// Two entries, two objects, and each object reached from a different delivery
/// position by one entry and the other position by the other. Every payload is
/// referenced and no `(payload, entry key)` pair repeats, so neither existing
/// obligation notices; what is wrong is that the artifact declares two consumer
/// targets and carries one object for each *position*, mixed. The neutral layer
/// cannot decide which target a shared object was built for, so it refuses the
/// shape.
#[test]
fn rejects_a_payload_reached_from_two_delivery_positions() {
    let semantic = semantic_program();
    let program = partial_window_program(&semantic);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let first = draft.push_payload(payload(0xa1)).unwrap();
    let second = draft.push_payload(payload(0xb1)).unwrap();
    let mut spec = partial_window_variant(first);
    spec.entries[0].implementation.payloads = vec![first, second];
    spec.entries[1].implementation.payloads = vec![second, first];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    let diagnostics = draft
        .build()
        .expect_err("one object cannot serve two delivery positions")
        .diagnostics()
        .to_vec();
    assert_eq!(
        diagnostics,
        [ArtifactDiagnostic::AmbiguousPayloadDelivery { payload: 1 }],
    );
}

/// One entry naming one payload at two positions is refused twice over.
///
/// Kept separate from the case above because it is decided by *both*
/// obligations, and observing that is what says the older one still does work
/// here: repeating a payload within one entry repeats a `(payload, entry key)`
/// pair, which the backend-entry injectivity rule that predates delivery
/// positions catches on its own, and it also puts one object at two positions.
/// Whole-artifact verification reports every diagnostic, so both are asserted
/// rather than only whichever happens to be first.
#[test]
fn rejects_one_payload_at_two_positions_of_one_entry() {
    let semantic = semantic_program();
    let program = fused_program(&semantic, SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    let mut spec = variant(&formulas, descriptor, b"fused");
    spec.entries[0].implementation.payloads = vec![descriptor, descriptor];
    draft.push_variant(&program, spec).unwrap();
    declare_realization(&mut draft, &program);
    assert_eq!(
        draft
            .build()
            .expect_err("one object cannot realize one entry twice")
            .diagnostics(),
        [
            ArtifactDiagnostic::DuplicateBackendEntry,
            ArtifactDiagnostic::AmbiguousPayloadDelivery { payload: 0 },
        ],
    );
}

#[test]
fn rejects_two_entries_claiming_one_backend_entry() {
    let semantic = semantic_program();
    let first = fused_program(&semantic, SCALE_BITS);
    let second = fused_program(&semantic, OTHER_SCALE_BITS);
    let provider = lowering_provider(1);
    let environment = CompilationEnvironment::new([provider.clone()], []).unwrap();
    let mut draft = ArtifactProgramBuilder::new(&semantic, environment).unwrap();
    draft.select_lowering_provider(selection(provider)).unwrap();
    let descriptor = draft.push_payload(payload(0xa1)).unwrap();
    let formulas = formulas(&mut draft);
    draft
        .push_variant(&first, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    draft
        .push_variant(&second, variant(&formulas, descriptor, b"fused"))
        .unwrap();
    declare_realization_over(&mut draft, &first, 2);
    assert_eq!(
        draft
            .build()
            .expect_err("a non-injective backend mapping is rejected")
            .diagnostics(),
        [ArtifactDiagnostic::DuplicateBackendEntry],
    );
}

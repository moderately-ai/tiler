//! Declared split reductions (`PartialReduction`) and the stage-owner derivation
//! every uncovering-stage account (split, publishing copy, staged realization) shares.

use super::super::{
    CoveredOccurrence, KernelProgramBuildError, KernelProgramDiagnostic, PartialReduction,
    SemanticOccurrence, VerifiedKernelProgram,
};
use super::support::{
    SCALE_BITS, TwoStage, TwoStageShape, canonical_program, checked_coverage, complete_two_stage,
    declare_program_contract, flush_contract, serial_sum_program, strict_contract, two_stage,
    wire_two_stage_structure,
};
use crate::semantic::{EncodedComponentRole, OutputKey, SemanticProgram};

/// Declares the canonical split contract over the two-stage fixture.
///
/// The fixture's temporary is `[2, 3]` and its output `[2]`, so a split of
/// three partitions each combining one contributor is the structurally exact
/// contract over it. That the pointwise stage is not *semantically* a partial
/// reducer is deliberate and not a gap: this layer proves the structure of a
/// split — who writes the partials, who reads them, and that the coverage
/// arithmetic closes — while whether each pass really is the reduction pass it
/// claims is proven by the region verifier in `crate::schedule`.
fn split_over(wired: &TwoStage) -> PartialReduction {
    PartialReduction {
        producer: wired.pointwise,
        combiner: wired.reduction,
        partial: wired.temporary,
        result: wired.output,
        occurrence: SemanticOccurrence::new(1),
        partitions: 3,
        contributors_per_partition: 1,
    }
}

fn program_with_split(
    semantic: &SemanticProgram,
    amend: impl FnOnce(&TwoStage, PartialReduction) -> PartialReduction,
) -> Result<VerifiedKernelProgram, KernelProgramDiagnostic> {
    let wired = two_stage(semantic, TwoStageShape::UncoveringSecondStage);
    let split = amend(&wired, split_over(&wired));
    let mut builder = wire_two_stage_structure(wired);
    builder
        .push_partial_reduction(split)
        .expect("a well-formed split declaration");
    declare_program_contract(&mut builder);
    builder
        .build()
        .map_err(|error| *error.diagnostics().first().expect("one diagnostic"))
}

/// Retains the assembled owner subject so these tests can perturb the graph the
/// owner derivation reads, rather than weakening an assertion around a verified
/// program that no longer contains the malformed shape.
fn owner_data_with_split(
    semantic: &SemanticProgram,
    amend: impl FnOnce(&TwoStage, PartialReduction) -> PartialReduction,
) -> super::super::model::KernelProgramData {
    let wired = two_stage(semantic, TwoStageShape::UncoveringSecondStage);
    let split = amend(&wired, split_over(&wired));
    let mut builder = wire_two_stage_structure(wired);
    builder
        .push_partial_reduction(split)
        .expect("the base split declaration is locally well formed");
    declare_program_contract(&mut builder);
    builder.into_data_for_owner_test()
}

fn owner_refusal(data: &super::super::model::KernelProgramData) -> KernelProgramDiagnostic {
    match super::super::verify::derive_stage_owners(data) {
        Ok(_) => panic!("the perturbed owner graph unexpectedly derived an owner"),
        Err(diagnostic) => diagnostic,
    }
}

#[test]
fn complete_stage_owner_refusals_reach_their_exact_graph_branches() {
    use super::super::model::{PublishingCopyData, StagedRealizationData};

    let semantic = serial_sum_program(SCALE_BITS);

    let mut missing_builder =
        wire_two_stage_structure(two_stage(&semantic, TwoStageShape::UncoveringSecondStage));
    declare_program_contract(&mut missing_builder);
    assert_eq!(
        owner_refusal(&missing_builder.into_data_for_owner_test()),
        KernelProgramDiagnostic::MissingStageOwner,
        "the uncovered combiner has no owner when its continuation declaration is absent",
    );

    let mut foreign = owner_data_with_split(&semantic, |_, split| PartialReduction {
        occurrence: SemanticOccurrence::new(4),
        ..split
    });
    foreign.stages[0]
        .coverage
        .retain(|covered| covered.occurrence().get() != 4);
    assert_eq!(
        owner_refusal(&foreign),
        KernelProgramDiagnostic::ForeignStageOwnerProof,
        "changing the split subject to an occurrence no stage covers must not invent a root",
    );

    let mut fork = owner_data_with_split(&semantic, |_, split| split);
    fork.partial_reductions.push(fork.partial_reductions[0]);
    assert_eq!(
        owner_refusal(&fork),
        KernelProgramDiagnostic::DuplicateStageOwnerOrdinal,
        "two continuation edges from one root are a fork, not two owners for ordinal one",
    );

    let mut looped = owner_data_with_split(&semantic, |_, split| split);
    let split = looped.partial_reductions[0];
    looped.staged_realizations.push(StagedRealizationData {
        producer: split.combiner,
        consumer: split.producer,
        handed: split.partial,
        occurrence: split.occurrence,
    });
    assert_eq!(
        owner_refusal(&looped),
        KernelProgramDiagnostic::DuplicateStageOwnerOrdinal,
        "a continuation that revisits a reached stage is a loop, not a new ordinal",
    );

    let mut merged = owner_data_with_split(&semantic, |_, split| split);
    merged.stages.push(merged.stages[1].clone());
    let split = merged.partial_reductions[0];
    merged.staged_realizations.push(StagedRealizationData {
        producer: 2,
        consumer: split.combiner,
        handed: split.partial,
        occurrence: split.occurrence,
    });
    assert_eq!(
        owner_refusal(&merged),
        KernelProgramDiagnostic::SkippedStageOwnerOrdinal,
        "a second incoming continuation is a merge only through an edge detached from the root path",
    );

    let mut disconnected_builder =
        wire_two_stage_structure(two_stage(&semantic, TwoStageShape::UncoveringSecondStage));
    declare_program_contract(&mut disconnected_builder);
    let mut disconnected = disconnected_builder.into_data_for_owner_test();
    disconnected
        .staged_realizations
        .push(StagedRealizationData {
            producer: 1,
            consumer: 0,
            handed: 1,
            occurrence: 1,
        });
    assert_eq!(
        owner_refusal(&disconnected),
        KernelProgramDiagnostic::SkippedStageOwnerOrdinal,
        "an edge not reachable from its proof-bound root is disconnected",
    );

    let mut publication = complete_two_stage(two_stage(&semantic, TwoStageShape::Canonical))
        .into_data_for_owner_test();
    publication.publishing_copies.push(PublishingCopyData {
        source_stage: 0,
        publisher: 1,
        source: 1,
        published: 1,
    });
    assert_eq!(
        owner_refusal(&publication),
        KernelProgramDiagnostic::MissingPublicationOwner,
        "a copy whose published value has no named output cannot claim publication ownership",
    );

    let mut mixed = complete_two_stage(two_stage(&semantic, TwoStageShape::Canonical))
        .into_data_for_owner_test();
    mixed.publishing_copies.push(PublishingCopyData {
        source_stage: 0,
        publisher: 1,
        source: 1,
        published: 2,
    });
    assert_eq!(
        owner_refusal(&mixed),
        KernelProgramDiagnostic::AmbiguousStageOwner,
        "a computing stage cannot also be the administrative publisher",
    );
}

#[test]
fn complete_stage_owner_identity_changes_only_for_admitted_owner_claims() {
    use super::super::model::{
        PublicationStageClaim, RealizationStageClaim, StageOwner, encoded_stage_owner_for_test,
    };

    let semantic = serial_sum_program(SCALE_BITS);
    let strict = checked_coverage(&semantic, &strict_contract());
    let flushed = checked_coverage(&semantic, &flush_contract());
    let bytes = |covered: CoveredOccurrence, ordinal| {
        encoded_stage_owner_for_test(&StageOwner::Realization(vec![RealizationStageClaim {
            covered,
            ordinal,
        }]))
    };
    let baseline = bytes(strict[0].clone(), 1);
    assert_ne!(
        baseline,
        bytes(strict[1].clone(), 1),
        "changing the proof-bound occurrence changes the complete owner subject",
    );
    assert_ne!(
        baseline,
        bytes(flushed[0].clone(), 1),
        "changing the reached refinement proof changes the complete owner subject",
    );
    assert_ne!(
        baseline,
        bytes(strict[0].clone(), 2),
        "changing only the continuation ordinal changes the complete owner subject",
    );
    assert_eq!(
        baseline,
        bytes(strict[0].clone(), 1),
        "the owner encoder has no downstream value, allocation, dependency, or builder-order input to distinguish",
    );

    // This is deliberately the crate-private owner projection rather than a
    // purported verified publisher. Existing fixtures can construct only a
    // plain-output copy, so the public artifact control proves `None` framing;
    // this encoder-level subject probe proves a nonempty component role is not
    // silently omitted when a future verified producer makes it reachable.
    let publication = |key, component_role| {
        encoded_stage_owner_for_test(&StageOwner::Publication(vec![PublicationStageClaim {
            key: OutputKey::new(key).expect("output key"),
            component_role,
        }]))
    };
    let publication_baseline = publication("published", None);
    assert_ne!(
        publication_baseline,
        publication("renamed-publication", None),
        "changing the exact publication key changes the owner subject",
    );
    assert_ne!(
        publication_baseline,
        publication("published", Some(EncodedComponentRole::new(99))),
        "changing the publication component role from None to a concrete role changes the owner subject",
    );
    assert_eq!(
        publication_baseline,
        publication("published", None),
        "the publication owner encoder has no producer, downstream value, allocation, dependency, or builder-order input",
    );
}

/// A declared split verifies and is readable back off the verified program.
#[test]
fn a_declared_split_reduction_is_verified_and_retained() {
    let semantic = serial_sum_program(SCALE_BITS);
    let program = program_with_split(&semantic, |_, split| split).expect("verified program");
    let split = program
        .partial_reductions()
        .next()
        .expect("one declared split");
    assert_eq!(split.partitions(), 3);
    assert_eq!(split.contributors_per_partition(), 1);
    assert_eq!(split.total_contributors(), Some(3));
    assert_eq!(split.producer(), program.stages().next().expect("a stage"));
    // The partials the producer stages are exactly the ones the combiner reads,
    // and the dispatch dependency between the two is the ordinary data edge.
    assert_eq!(split.partial().definition(), Some(split.producer()));
    assert!(program.dependencies().any(|edge| {
        edge.predecessor() == split.producer() && edge.successor() == split.combiner()
    }));
}

/// The split contract changes program identity, so two splits never collide.
#[test]
fn the_declared_split_separates_kernel_program_identity() {
    let semantic = serial_sum_program(SCALE_BITS);
    let undeclared = canonical_program(&semantic);
    let declared = program_with_split(&semantic, |_, split| split).expect("verified program");
    assert_ne!(
        undeclared.canonical_identity(),
        declared.canonical_identity(),
        "a program that proves a split must not share identity with one that does not"
    );
    // Contributor coverage is an independently declared split fact, so changing
    // it must move identity alongside the exact occurrence and owner claims.
    let restated = program_with_split(&semantic, |_, split| PartialReduction {
        contributors_per_partition: 7,
        ..split
    })
    .expect("verified program");
    assert_ne!(
        declared.canonical_identity(),
        restated.canonical_identity(),
        "two splits claiming different contributor coverage must differ"
    );
}

/// A split whose partial is written by some other stage is rejected.
#[test]
fn a_partial_not_initialized_by_its_producer_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    assert_eq!(
        program_with_split(&semantic, |wired, split| PartialReduction {
            producer: wired.reduction,
            combiner: wired.pointwise,
            ..split
        }),
        Err(KernelProgramDiagnostic::PartialNotInitializedByProducer)
    );
}

/// A split whose combiner does not produce the result is rejected.
#[test]
fn a_result_not_produced_by_its_combiner_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    assert_eq!(
        program_with_split(&semantic, |wired, split| PartialReduction {
            result: wired.temporary,
            partial: wired.output,
            ..split
        }),
        // The output is written by the combiner, not the producer, so the
        // partial obligation is the first one that fails.
        Err(KernelProgramDiagnostic::PartialNotInitializedByProducer)
    );
}

/// A split staging its partials in a published output is rejected.
#[test]
fn a_partial_that_is_not_an_internal_temporary_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    let wired = two_stage(&semantic, TwoStageShape::Canonical);
    let split = PartialReduction {
        producer: wired.reduction,
        combiner: wired.pointwise,
        partial: wired.output,
        result: wired.temporary,
        occurrence: SemanticOccurrence::new(1),
        partitions: 3,
        contributors_per_partition: 1,
    };
    let mut builder = wire_two_stage_structure(wired);
    builder
        .push_partial_reduction(split)
        .expect("a well-formed split declaration");
    declare_program_contract(&mut builder);
    // The output *is* written by the named producer and read by nobody, so this
    // reaches the consumption rule rather than the materialization one; either
    // way the published output cannot serve as a split's staging tensor.
    assert_eq!(
        builder
            .build()
            .map_err(|error| *error.diagnostics().first().expect("one diagnostic")),
        Err(KernelProgramDiagnostic::PartialNotConsumedByCombiner)
    );
}

/// A split whose partial extent is not one value per partition is rejected.
#[test]
fn a_partial_extent_that_is_not_one_value_per_partition_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    for partitions in [2, 4, 6] {
        assert_eq!(
            program_with_split(&semantic, |_, split| PartialReduction {
                partitions,
                ..split
            }),
            Err(KernelProgramDiagnostic::PartialExtentMismatch),
            "a `[2]` result and a `[2, 3]` partial admit only three partitions"
        );
    }
}

/// A split covering nothing, or an unrepresentable amount, is rejected.
#[test]
fn an_unrepresentable_split_coverage_is_rejected() {
    let semantic = serial_sum_program(SCALE_BITS);
    for (partitions, contributors_per_partition) in [(0, 4), (3, u64::MAX)] {
        assert_eq!(
            program_with_split(&semantic, |_, split| PartialReduction {
                partitions,
                contributors_per_partition,
                ..split
            }),
            Err(KernelProgramDiagnostic::PartialCoverageUnrepresentable),
            "{partitions} x {contributors_per_partition} states no checkable coverage"
        );
    }
}

/// One stage cannot be both passes, and one partial cannot be split twice.
#[test]
fn a_malformed_split_declaration_is_rejected_at_insertion() {
    let semantic = serial_sum_program(SCALE_BITS);
    let wired = two_stage(&semantic, TwoStageShape::Canonical);
    let split = split_over(&wired);
    let mut builder = wire_two_stage_structure(wired);
    assert_eq!(
        builder.push_partial_reduction(PartialReduction {
            combiner: split.producer,
            ..split
        }),
        Err(KernelProgramBuildError::SelfDependency)
    );
    builder
        .push_partial_reduction(split)
        .expect("the first declaration is well formed");
    assert_eq!(
        builder.push_partial_reduction(PartialReduction {
            partitions: 1,
            contributors_per_partition: 3,
            ..split
        }),
        Err(KernelProgramBuildError::DuplicatePartialReduction)
    );
}

/// A stage computing nothing is admitted only as a declared split's combiner.
///
/// Both directions are driven from the same coverage shape, so the difference
/// between them is exactly the declaration: without it the program has a
/// dispatch it cannot account for, and with it the dispatch is the final pass
/// of a split whose partial pass already claims the reduction.
#[test]
fn an_uncovering_stage_is_admitted_only_as_a_declared_splits_combiner() {
    let semantic = serial_sum_program(SCALE_BITS);

    let undeclared = complete_two_stage(two_stage(&semantic, TwoStageShape::UncoveringSecondStage));
    assert_eq!(
        undeclared
            .build()
            .map_err(|error| *error.diagnostics().first().expect("one diagnostic")),
        Err(KernelProgramDiagnostic::UncoveringStage)
    );

    let wired = two_stage(&semantic, TwoStageShape::UncoveringSecondStage);
    let split = split_over(&wired);
    let mut declared = wire_two_stage_structure(wired);
    declared
        .push_partial_reduction(split)
        .expect("a well-formed split declaration");
    declare_program_contract(&mut declared);
    let program = declared.build().expect("verified program");
    assert!(
        program.stages().any(|stage| stage.coverage().is_empty()),
        "the combiner is retained as the uncovering stage the split accounts for"
    );
}

//! Declared staged realizations (`StagedRealization`): the third account admitting
//! a stage that covers no occurrence.

use super::super::{
    KernelProgramBuildError, KernelProgramDiagnostic, SemanticOccurrence, StagedRealization,
    VerifiedKernelProgram,
};
use super::support::{
    SCALE_BITS, TwoChain, TwoStage, TwoStageShape, canonical_program, complete_two_stage,
    declare_program_contract, publish_two_chain, serial_sum_program, two_chain, two_chain_program,
    two_stage, wire_two_stage_structure,
};
use crate::semantic::SemanticProgram;

/// Declares one staged realization over the two-stage fixture and builds.
///
/// The fixture's second stage covers nothing, which is the coverage shape a
/// staged realization's consumer has: the stage that *began* the realization
/// claims the occurrence, and claiming it again would double-cover the graph.
fn program_with_staged_realization(
    semantic: &SemanticProgram,
    amend: impl FnOnce(&TwoStage, StagedRealization) -> StagedRealization,
) -> Result<VerifiedKernelProgram, KernelProgramDiagnostic> {
    let wired = two_stage(semantic, TwoStageShape::UncoveringSecondStage);
    let realization = amend(
        &wired,
        StagedRealization {
            producer: wired.pointwise,
            consumer: wired.reduction,
            handed: wired.temporary,
            occurrence: SemanticOccurrence::new(4),
        },
    );
    let mut builder = wire_two_stage_structure(wired);
    builder
        .push_staged_realization(realization)
        .expect("a well-formed staged declaration");
    declare_program_contract(&mut builder);
    builder
        .build()
        .map_err(|error| *error.diagnostics().first().expect("one diagnostic"))
}

/// Declares one staged realization over the two-chain fixture and builds.
fn two_chain_staged(
    semantic: &SemanticProgram,
    state: impl FnOnce(&TwoChain) -> StagedRealization,
) -> Result<VerifiedKernelProgram, KernelProgramDiagnostic> {
    let chains = two_chain(semantic, true);
    let realization = state(&chains);
    let mut builder = publish_two_chain(chains);
    builder
        .push_staged_realization(realization)
        .expect("a well-formed staged declaration");
    builder
        .build()
        .map_err(|error| *error.diagnostics().first().expect("one diagnostic"))
}

/// An uncovering stage is admitted by a declared staged realization.
///
/// **The two directions differ by exactly the declaration.** The undeclared
/// program has a dispatch it cannot account for; the declared one has the
/// consumer of a realization whose producer already claims the occurrence they
/// jointly compute. That is the third account beside a split's final pass and a
/// copy's publisher, and it is a second *account* rather than a relaxation:
/// nothing here weakens the rule that a dispatch computing no operation must be
/// explained.
///
/// **It is also the one of the three whose admitting path completes on this
/// fixture.** A publishing copy's does not — every fixture in this module writes
/// its output at a reduced extent, and a copy publishes what it read — while a
/// staged realization deliberately carries no extent obligation, because a
/// realization's later stage iterates its own domain. That asymmetry is the
/// declaration's whole reason for existing rather than a gap in the fixture.
///
/// The check that can say no is the declaration itself: dropping the
/// `push_staged_realization` call in [`program_with_staged_realization`] returns
/// the first assertion's `UncoveringStage`.
#[test]
fn an_uncovering_stage_is_admitted_as_a_declared_staged_realizations_consumer() {
    let semantic = serial_sum_program(SCALE_BITS);

    let undeclared = complete_two_stage(two_stage(&semantic, TwoStageShape::UncoveringSecondStage));
    assert_eq!(
        undeclared
            .build()
            .map_err(|error| *error.diagnostics().first().expect("one diagnostic")),
        Err(KernelProgramDiagnostic::UncoveringStage)
    );

    let program = program_with_staged_realization(&semantic, |_, realization| realization)
        .expect("the declared staged realization is admitted");
    let declared: Vec<_> = program.staged_realizations().collect();
    assert_eq!(declared.len(), 1);
    assert_eq!(declared[0].occurrence(), SemanticOccurrence::new(4));
    assert!(declared[0].consumer().coverage().is_empty());
    assert!(
        declared[0]
            .producer()
            .coverage()
            .iter()
            .any(|covered| covered.occurrence() == SemanticOccurrence::new(4)),
        "the chain is rooted at the stage that covers the occurrence it continues"
    );
}

/// The declaration is folded, and an otherwise identical program differs by it.
///
/// **Two programs alike in every stage, value, view, allocation, and edge.** The
/// only difference is whether the second stage is *declared* to continue the
/// first's realization, and identity says so — which is the property the domain
/// step exists for, and the one no comparison of the entities already folded
/// could have recovered.
///
/// The pair is stated over the canonical fixture, whose second stage covers an
/// occurrence of its own, rather than over the uncovering one: the declaration
/// is optional there and both sides verify, which is what makes the declaration
/// the only variable. Over the uncovering fixture the undeclared side cannot be
/// built at all, because `UncoveringStage` refuses it.
#[test]
fn a_declared_staged_realization_changes_program_identity() {
    let semantic = serial_sum_program(SCALE_BITS);
    let bare = canonical_program(&semantic);

    let wired = two_stage(&semantic, TwoStageShape::Canonical);
    let realization = StagedRealization {
        producer: wired.pointwise,
        consumer: wired.reduction,
        handed: wired.temporary,
        occurrence: SemanticOccurrence::new(0),
    };
    let mut builder = wire_two_stage_structure(wired);
    builder
        .push_staged_realization(realization)
        .expect("a well-formed staged declaration");
    declare_program_contract(&mut builder);
    let declared = builder.build().expect("the declared program verifies");

    assert_eq!(bare.stages().len(), declared.stages().len());
    assert_eq!(bare.values().len(), declared.values().len());
    assert_eq!(bare.dependencies().len(), declared.dependencies().len());
    assert_eq!(bare.staged_realizations().len(), 0);
    assert_eq!(declared.staged_realizations().len(), 1);
    assert_ne!(bare.canonical_identity(), declared.canonical_identity());
}

/// Each staged-realization row obligation is driven against a case that fails.
///
/// **Two fixtures, and which rows each carries is forced by their shapes.** The
/// two-stage fixture has an uncovering second stage, so its consumer must be
/// that stage for the coverage arm not to fire first. The two-chain fixture has
/// four stages, each covering a disjoint range, which is what a row naming
/// another chain's stage needs. Every row differs from a well-formed declaration
/// by exactly one named entity.
///
/// **One obligation is deliberately not driven here, and it is unreachable
/// rather than untested.** `HandedValueNotMaterialized` needs a handed value
/// that is neither a temporary nor an externally bound input, and no program can
/// present one: an input is refused a writer by `ExternalValueWritten` two
/// phases earlier, and `ValueRole::Output` fills only `TensorRole::Output`,
/// which is a write — so no stage can *read* an output-role value and the read
/// obligation above it always fires first. It is stated for the reason
/// `PartialNotMaterialized` is: the declaration owes the obligation whether or
/// not today's role vocabulary can spell a violation of it.
#[test]
fn the_staged_realization_row_obligations_can_each_say_no() {
    let semantic = serial_sum_program(SCALE_BITS);

    // The handed value is written by the consumer rather than by the named
    // producer, so the consumer would continue from values that stage never
    // produced.
    assert_eq!(
        program_with_staged_realization(&semantic, |wired, realization| StagedRealization {
            handed: wired.output,
            ..realization
        }),
        Err(KernelProgramDiagnostic::HandedValueNotInitializedByProducer)
    );

    let chained = two_chain_program();

    // The consumer never reads the value it claims to continue from: the first
    // chain's temporary is defined by the first chain's map stage and read only
    // by the first chain's reduction.
    assert_eq!(
        two_chain_staged(&chained, |chains| StagedRealization {
            producer: chains.first_map,
            consumer: chains.second_reduce,
            handed: chains.first_temporary,
            occurrence: SemanticOccurrence::new(0),
        }),
        Err(KernelProgramDiagnostic::HandedValueNotReadByConsumer)
    );
}

/// The chain must run from the stage that covers the occurrence it continues.
///
/// **The obligation no single declaration can see, and the one the row checks
/// above cannot reach.** Every named entity is right in both rows below — the
/// handed value's definer is the producer, the consumer reads it, and it is a
/// temporary — and both programs are still refused, because the occurrence each
/// claims to continue was begun by a stage its chain never runs from. A
/// realization's stages run in order and each runs once; a chain rooted
/// elsewhere describes later dispatches computing a stage nobody began.
///
/// The positive control is stated first and is what makes the two refusals
/// about the *root* rather than about the fixture: the identical declaration
/// naming an occurrence its producer covers verifies.
#[test]
fn a_staged_realization_chain_must_start_where_its_occurrence_is_covered() {
    let chained = two_chain_program();

    // The control: occurrence 0 is covered by `first_map`, which is this
    // declaration's producer, so the walk reaches its one row.
    two_chain_staged(&chained, |chains| StagedRealization {
        producer: chains.first_map,
        consumer: chains.first_reduce,
        handed: chains.first_temporary,
        occurrence: SemanticOccurrence::new(0),
    })
    .expect("a chain rooted at the covering stage verifies");

    // Occurrence 4 is the first chain's reduction, covered by this declaration's
    // *consumer*. The walk starts there, finds no continuation, and the one
    // declared row lies on no path.
    assert_eq!(
        two_chain_staged(&chained, |chains| StagedRealization {
            producer: chains.first_map,
            consumer: chains.first_reduce,
            handed: chains.first_temporary,
            occurrence: SemanticOccurrence::new(4),
        }),
        Err(KernelProgramDiagnostic::StagedRealizationChainBroken)
    );

    // Occurrence 7 is the second chain's reduction, covered by a stage in the
    // other chain entirely — so the walk starts somewhere this declaration's
    // two stages never appear.
    assert_eq!(
        two_chain_staged(&chained, |chains| StagedRealization {
            producer: chains.first_map,
            consumer: chains.first_reduce,
            handed: chains.first_temporary,
            occurrence: SemanticOccurrence::new(7),
        }),
        Err(KernelProgramDiagnostic::StagedRealizationChainBroken)
    );
}

/// One stage cannot be both halves, and one consumer cannot continue one
/// occurrence twice.
#[test]
fn a_malformed_staged_realization_declaration_is_rejected_at_insertion() {
    let semantic = serial_sum_program(SCALE_BITS);
    let wired = two_stage(&semantic, TwoStageShape::UncoveringSecondStage);
    let realization = StagedRealization {
        producer: wired.pointwise,
        consumer: wired.reduction,
        handed: wired.temporary,
        occurrence: SemanticOccurrence::new(4),
    };
    let other_value = wired.output;
    let mut builder = wire_two_stage_structure(wired);
    assert_eq!(
        builder.push_staged_realization(StagedRealization {
            consumer: realization.producer,
            ..realization
        }),
        Err(KernelProgramBuildError::SelfDependency)
    );
    assert_eq!(
        builder.push_staged_realization(StagedRealization {
            occurrence: SemanticOccurrence::new(5),
            ..realization
        }),
        Err(KernelProgramBuildError::CoverageOutOfRange {
            occurrence: SemanticOccurrence::new(5),
            operations: 5,
        }),
        "the fixture graph has five operations, so ordinal five names none of them"
    );
    builder
        .push_staged_realization(realization)
        .expect("the first declaration is well formed");
    // A second declaration by the same consumer naming a *different* occurrence
    // is admitted: one fused dispatch may continue several realizations, and the
    // key is the pair rather than the stage.
    builder
        .push_staged_realization(StagedRealization {
            occurrence: SemanticOccurrence::new(3),
            ..realization
        })
        .expect("one consumer may continue two occurrences");
    // The same pair a second time has no reading: two handed values for one
    // stage boundary leave which one carries the realization undecided.
    assert_eq!(
        builder.push_staged_realization(StagedRealization {
            handed: other_value,
            ..realization
        }),
        Err(KernelProgramBuildError::DuplicateStagedRealization)
    );
}

//! Declared publishing copies (`PublishingCopy`): the second account admitting a
//! stage that covers no occurrence.

use super::super::{
    KernelProgramBuildError, KernelProgramDiagnostic, PublishingCopy, VerifiedKernelProgram,
};
use super::support::{
    SCALE_BITS, TwoChain, TwoStage, TwoStageShape, complete_two_stage, declare_program_contract,
    publish_two_chain, serial_sum_program, two_chain, two_chain_program, two_stage,
    wire_two_stage_structure,
};
use crate::semantic::SemanticProgram;

/// Declares one publishing copy over the two-stage fixture and builds.
fn program_with_copy(
    semantic: &SemanticProgram,
    amend: impl FnOnce(&TwoStage, PublishingCopy) -> PublishingCopy,
) -> Result<VerifiedKernelProgram, KernelProgramDiagnostic> {
    let wired = two_stage(semantic, TwoStageShape::UncoveringSecondStage);
    let copy = amend(
        &wired,
        PublishingCopy {
            source_stage: wired.pointwise,
            publisher: wired.reduction,
            source: wired.temporary,
            published: wired.output,
        },
    );
    let mut builder = wire_two_stage_structure(wired);
    builder
        .push_publishing_copy(copy)
        .expect("a well-formed copy declaration");
    declare_program_contract(&mut builder);
    builder
        .build()
        .map_err(|error| *error.diagnostics().first().expect("one diagnostic"))
}

/// An uncovering stage is admitted by a declared copy, and refused without one.
///
/// **The two directions differ by exactly the declaration.** The undeclared
/// program has a dispatch it cannot account for; the declared one has the
/// publisher of a copy whose source stage already claims every occurrence. That
/// is the same shape a split's final pass has, one fold up, and it is why the
/// arm is a second *account* rather than a relaxation of the rule.
///
/// **Measurement boundary, and it bounds two claims rather than one.** This
/// drives the coverage arm alone: no fixture in this module can state a copy
/// whose obligations *all* hold, because a copy publishes what it read and every
/// fixture here writes its output at a reduced extent — the two-stage
/// temporary is `[2, 3]` against a `[2]` output, and both chains of the
/// two-chain fixture are the same shape. So the declared program below is
/// structurally a copy in every respect but its extents, and it is refused by
/// the extent obligation rather than admitted.
///
/// The complete admitting path is exercised end to end by `tiler-compiler`'s
/// `pipeline::conformance::a_published_and_consumed_intermediate_compiles_and_agrees`,
/// which asserts the declared copy, the single uncovering stage, and bit
/// agreement for both published outputs. The identity claim is bounded the same
/// way: that the declaration section is folded is evidenced by the domain step
/// this change carries and by
/// [`the_program_domain_separator_is_what_distinguishes_the_reinterpreting_steps`],
/// while *injectivity against an otherwise identical program* rests on the
/// section being length-framed and written unconditionally, and has no fixture
/// here that could state the pair. Building one would need a third kernel
/// writing an output at the temporary's extent, which would re-state the
/// compiler's evidence rather than add any.
#[test]
fn an_undeclared_uncovering_stage_still_refuses_by_name() {
    let semantic = serial_sum_program(SCALE_BITS);
    let undeclared = complete_two_stage(two_stage(&semantic, TwoStageShape::UncoveringSecondStage));
    assert_eq!(
        undeclared
            .build()
            .map_err(|error| *error.diagnostics().first().expect("one diagnostic")),
        Err(KernelProgramDiagnostic::UncoveringStage)
    );
    // With the declaration, the coverage arm no longer fires: the program now
    // fails on the copy's own extent obligation, which is a later phase and a
    // different rule.
    assert_eq!(
        program_with_copy(&semantic, |_, copy| copy),
        Err(KernelProgramDiagnostic::PublishedCopyExtentMismatch)
    );
}

/// Declares one publishing copy over the two-chain fixture and builds.
fn two_chain_copy(
    semantic: &SemanticProgram,
    state: impl FnOnce(&TwoChain) -> PublishingCopy,
) -> Result<VerifiedKernelProgram, KernelProgramDiagnostic> {
    let chains = two_chain(semantic, true);
    let copy = state(&chains);
    let mut builder = publish_two_chain(chains);
    builder
        .push_publishing_copy(copy)
        .expect("a well-formed copy declaration");
    builder
        .build()
        .map_err(|error| *error.diagnostics().first().expect("one diagnostic"))
}

/// Each publishing-copy obligation is driven against a case that must fail.
///
/// **Two fixtures, and which rows each carries is forced by their shapes rather
/// than chosen.** The two-stage fixture has an uncovering second stage, so its
/// publisher must be that stage for the coverage arm not to fire first — which
/// fixes what its rows can perturb. The two-chain fixture has four stages and
/// two independently published outputs, which is what a row naming *another*
/// stage's value needs. Every row differs from a well-formed declaration by
/// exactly one named entity.
#[test]
fn the_publishing_copy_obligations_can_each_say_no() {
    let semantic = serial_sum_program(SCALE_BITS);

    // The named source is written by the publisher rather than by the named
    // source stage, so the publisher would copy values that stage never
    // produced.
    assert_eq!(
        program_with_copy(&semantic, |wired, copy| PublishingCopy {
            source: wired.output,
            ..copy
        }),
        Err(KernelProgramDiagnostic::CopiedSourceNotInitializedBySourceStage)
    );

    // The published value is an internal temporary. A declaration naming one has
    // nothing to publish whichever stage wrote it, which is why the role is
    // checked before the writer.
    assert_eq!(
        program_with_copy(&semantic, |wired, copy| PublishingCopy {
            published: wired.temporary,
            ..copy
        }),
        Err(KernelProgramDiagnostic::PublishedCopyNotOutput)
    );

    let chained = two_chain_program();

    // The publisher never reads the value it claims to copy: the first chain's
    // temporary is defined by the first chain's map stage and read only by the
    // first chain's reduction.
    assert_eq!(
        two_chain_copy(&chained, |chains| PublishingCopy {
            source_stage: chains.first_map,
            publisher: chains.second_reduce,
            source: chains.first_temporary,
            published: chains.second_output,
        }),
        Err(KernelProgramDiagnostic::CopiedSourceNotReadByPublisher)
    );

    // The published value is a genuine output written by a *different* stage.
    assert_eq!(
        two_chain_copy(&chained, |chains| PublishingCopy {
            source_stage: chains.second_map,
            publisher: chains.second_reduce,
            source: chains.second_temporary,
            published: chains.first_output,
        }),
        Err(KernelProgramDiagnostic::PublishedCopyNotWrittenByPublisher)
    );
}

/// One stage cannot be both halves, and one value cannot be published twice.
#[test]
fn a_malformed_publishing_copy_declaration_is_rejected_at_insertion() {
    let semantic = serial_sum_program(SCALE_BITS);
    let wired = two_stage(&semantic, TwoStageShape::UncoveringSecondStage);
    let copy = PublishingCopy {
        source_stage: wired.pointwise,
        publisher: wired.reduction,
        source: wired.temporary,
        published: wired.output,
    };
    let mut builder = wire_two_stage_structure(wired);
    assert_eq!(
        builder.push_publishing_copy(PublishingCopy {
            publisher: copy.source_stage,
            ..copy
        }),
        Err(KernelProgramBuildError::SelfDependency)
    );
    builder
        .push_publishing_copy(copy)
        .expect("the first declaration is well formed");
    assert_eq!(
        builder.push_publishing_copy(PublishingCopy {
            source: copy.published,
            ..copy
        }),
        Err(KernelProgramBuildError::DuplicatePublishingCopy)
    );
}

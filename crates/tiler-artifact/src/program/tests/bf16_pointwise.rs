//! The pointwise producer path at two arithmetic widths.

use super::super::VerifiedArtifactProgram;
use super::support::graphs::checked_coverage_under;
use super::support::pointwise::PointwiseWidth;
use tiler_ir::kernel::KernelType;
use tiler_ir::program::StorageScalar;

/// A pure-BF16 program travels semantics to packaged artifact through the
/// ordinary producer path.
///
/// The composition `carry-bf16-through-the-artifact-encoding-and-identity`
/// recorded as unreachable, now walked end to end: every one of the four
/// coverage records is minted by the refinement verifier from a candidate region
/// this crate built, the program verifier accepts a stage claiming them, and the
/// artifact builder packages the result. Nothing here forges an envelope.
///
/// The carrier assertions are on what a *consumer* reads — the declared
/// interface component and the entry's binding windows — because those are what
/// a runtime uses to size a buffer, and twelve versus twenty-four bytes over the
/// same six-element tensor is the whole reason the width has to survive.
#[test]
fn a_pure_bf16_program_reaches_a_verified_artifact_through_the_builder() {
    let semantic = PointwiseWidth::Bf16.semantic();
    let coverage = checked_coverage_under(&semantic, &PointwiseWidth::Bf16.contract());
    assert_eq!(
        coverage
            .iter()
            .map(|covered| covered.occurrence().get())
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3],
        "the coverage partition is the graph's complete canonical occurrence run",
    );
    let program = PointwiseWidth::Bf16.program(&semantic);
    assert_eq!(program.stages().count(), 1);

    let artifact = PointwiseWidth::Bf16.artifact();
    let component = artifact
        .inputs()
        .next()
        .expect("one declared input")
        .components()
        .next()
        .expect("one dense component");
    assert_eq!(component.storage_scalar(), StorageScalar::Bf16);
    assert_eq!(component.access_type(), KernelType::Bf16);
    let entry = artifact
        .variants()
        .next()
        .expect("one variant")
        .entries()
        .next()
        .expect("one entry");
    assert_eq!(
        entry
            .bindings()
            .map(|binding| (
                binding.storage_scalar(),
                binding.access_type(),
                binding.window().length,
            ))
            .collect::<Vec<_>>(),
        [(StorageScalar::Bf16, KernelType::Bf16, 12); 2],
        "six bf16 elements are twelve bytes on both the read and the write side",
    );

    // The delivered-realization record names the arithmetic the program
    // computes in. Asserted because nothing downstream can: the artifact-level
    // cross-check compares behaviours against each entry's realization and never
    // reads the subject, so a record naming `f32` for this program would build,
    // encode, decode, and state something false to every consumer of it.
    let record = artifact.delivered_realization();
    assert!(
        record
            .scalar_arithmetic(&PointwiseWidth::Bf16.subject().identity())
            .is_some(),
        "the record must carry the bf16 scalar-arithmetic subject",
    );
    assert!(
        record
            .scalar_arithmetic(&PointwiseWidth::F32.subject().identity())
            .is_none(),
        "and must not carry the f32 one, which no other check would catch",
    );
}

/// The same program at the other width is a different artifact.
///
/// The producer-path counterpart of the encoding rung's carrier-only comparison,
/// and it answers a strictly larger question. There the two envelopes were one
/// artifact with two tag bytes rewritten, so the four differing identity bytes
/// were the carrier and nothing else. Here the two are separately *derived* from
/// separately verified semantic graphs, so the difference spans the semantic
/// operation keys, the refinement evidence minted under each width's own
/// contract, the scheduled expression, the canonical NaN payload, and the buffer
/// sizes — every place the width is load-bearing rather than only the two the
/// forgery could reach.
#[test]
fn the_bf16_artifact_and_its_f32_twin_are_two_artifacts() {
    let bf16 = PointwiseWidth::Bf16.artifact();
    let twin = PointwiseWidth::F32.artifact();
    assert_ne!(
        bf16.canonical_identity(),
        twin.canonical_identity(),
        "two artifacts differing in the arithmetic they compute in must not share an identity",
    );
    assert_eq!(
        bf16.canonical_identity(),
        PointwiseWidth::Bf16.artifact().canonical_identity(),
        "nothing else in the fixture varies between two builds at one width",
    );
    let window = |artifact: &VerifiedArtifactProgram| {
        artifact
            .variants()
            .next()
            .expect("one variant")
            .entries()
            .next()
            .expect("one entry")
            .bindings()
            .next()
            .expect("one read binding")
            .window()
            .length
    };
    assert_eq!(
        (window(&bf16), window(&twin)),
        (12, 24),
        "the twin addresses the same six elements at twice the width",
    );
}

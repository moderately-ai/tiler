use super::super::{
    AccessOrdinal, ArithmeticType, Axis, CompilationRequest, CompilerCapabilitySnapshot,
    DTypeDispatchRefusalDisposition, F32, InputKey, LogicalAccess, NormalizedGather,
    NormalizedOutput, OutputKey, PARAMETRIC_BROADCAST_ACCESS_TAG, RequestError, SemanticMemberId,
    SemanticProgram, Shape, TargetProfile, UNREAD_DECLARED_INPUT_TAG, VerifiedTargetRequest,
    encode_access_relation, encode_output_subject, output_subject, recognize_program_outputs,
    select_supported_strategy, verify_planned_request,
};
use super::support::{laws_of, packaged_program, planning_capability_rule};
use tiler_ir::semantic::{
    F32Constant, F32Gather, F32Multiply, SemanticProgramBuilder, gather_index_resolved_type,
};

/// One ordinary governed gather occurrence over the admitted F32/U32 signature.
fn gather_program() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let source = builder
        .input::<F32>(InputKey::new("source").unwrap(), Shape::from_dims([4, 2]))
        .unwrap();
    let index = builder
        .input_resolved(
            InputKey::new("index").unwrap(),
            Shape::from_dims([3]),
            gather_index_resolved_type(),
        )
        .unwrap();
    let gathered = F32Gather::apply(&mut builder, source, index, Axis::new(0)).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), gathered)
        .unwrap();
    builder.build().unwrap()
}

/// A gather stops first at exact target dispatch, then at the region vocabulary.
///
/// The second compile changes only the target's exact U32 dispatch fact. It
/// keeps the semantic program byte-for-byte identical, so the advance from the
/// target-local `DTypeNotDispatchable` refusal to the next layer pins the
/// request boundary's ordered diagnostic layers without granting Gather a
/// production target claim or a planning route.
///
/// **The second expectation is `("planning", "region-vocabulary")`, and what
/// keeps it there has changed.** The governed profile carries a
/// `tiler::gather-f32@1` index-access capability row, so `resolve_lowering`
/// answers `Ok` for this program and the compile advances past the lowering
/// stage entirely. The scheduled-region vocabulary used to decline *every*
/// gather member set for want of a route to the retained proof; that route now
/// exists, so what declines this program is narrower and is a fact about this
/// fixture: its `[4, 2]` source over an inhabited result reaches neither closed
/// bounds argument, so its own realization holds a
/// `GatherIndexValidationRequirement` and it is refused by name under
/// `RegionVocabularyWall::GatherIndexBoundsUnproved`.
///
/// **So this test now discriminates rather than merely refusing.** Its sibling
/// `a_gathers_spelling_follows_its_own_occurrences_bounds_evidence` runs the
/// same call over a statically proved gather and receives a spelling, which is
/// what makes the refusal here evidence about this program instead of about
/// every gather. Nothing about it is weaker: this fixture acquires no schedule,
/// kernel, artifact, cache, or dispatch route, and the receipt that would
/// discharge its outstanding obligation is another ticket's vocabulary.
///
/// Watched failing under a deliberate subject perturbation: removing the
/// U32 row from `governed_with_gather_index_dispatch_for_test` makes the
/// second compile return the same target-local refusal as the first.
#[test]
fn a_governed_gather_refuses_at_dispatch_then_at_the_region_vocabulary() {
    let program = gather_program();
    let product = crate::pipeline::compile(CompilationRequest::governed(&program))
        .expect("a target-local refusal is an ordinary compilation product");
    let [outcome] = product.targets.as_slice() else {
        panic!("the governed request carries one target outcome");
    };
    assert_eq!(
        outcome.failure(),
        Some(&crate::pipeline::CompileError::NoFeasiblePlan(
            crate::pipeline::NoFeasiblePlanError::Request(RequestError::DTypeNotDispatchable {
                target_profile: TargetProfile::governed().profile_key().clone(),
                resolved_type: Box::new(gather_index_resolved_type()),
                disposition: DTypeDispatchRefusalDisposition::Unknown,
            })
        )),
        "the governed target answers for the exact U32 index type before recognition",
    );

    let mut widened = CompilationRequest::governed(&program);
    widened.target_profiles = vec![TargetProfile::governed_with_gather_index_dispatch_for_test()];
    // The refusal is compared by its phase and rule alone. A `CompileError`
    // reaching this layer carries a whole explain trace, and comparing the
    // error whole would pin every byte of that trace here — a fixture-shaped
    // assertion that would move for reasons unrelated to the ordered layers
    // this test is about.
    let advanced = crate::pipeline::compile(widened).expect_err("the widened request refuses");
    assert_eq!(
        planning_capability_rule(&advanced)
            .unwrap_or_else(|| panic!("the widened request refused with {advanced:?}")),
        ("planning", "region-vocabulary"),
        "an exact U32 dispatch fact advances the same program past recognition and \
past governed lowering, which now carries a gather capability row, to the \
scheduled-region vocabulary",
    );
    // The class the request boundary reports is deliberately coarse — every
    // vocabulary gap shares it — so the *named* wall is read from the trace,
    // which is where the per-region typed decline is kept. Without this the
    // assertion above would equally pass if some unrelated vocabulary gap had
    // become the first refusal.
    let crate::pipeline::CompileError::Explained { explain, .. } = &advanced else {
        panic!("a planning refusal carries its explain trace, not {advanced:?}");
    };
    assert!(
        explain
            .render()
            .contains(crate::physical::RegionVocabularyWall::GatherIndexBoundsUnproved.reason()),
        "the trace must name the gather wall as the cause: {}",
        explain.render(),
    );
}

/// The real output recognizer resolves a Gather to its own recognized shape.
///
/// **This assertion is the inverse of the one it replaces, and the inversion is
/// the landing.** It previously required `operation-set` — the refusal a walk
/// reports for an occurrence no recognizer claims — because no gather arm
/// existed. `recognize_gather` is that arm, so the same fixture through the same
/// real realization-law authority and output walk now produces a recognized
/// shape, and leaving the old expectation in place would have left the suite
/// asserting the opposite of the tree.
///
/// Every field is checked against the fixture rather than the shape being merely
/// destructured, because the fields are what the request subject binds: the two
/// declared ordinals are the ADR 0108 amendment's checked association, and the
/// result shape is derived here rather than read from the graph.
///
/// Watched failing under a deliberate subject perturbation: swapping
/// `gather_program`'s two declared inputs so the U32 index is declared first
/// moves `source_input`/`index_input` to `1`/`0` and reddens this exact
/// assertion — the ordinals are read from declaration position, not assumed.
#[test]
fn the_real_request_recognizer_resolves_a_gather_to_its_own_shape() {
    let program = gather_program();
    let recognized = recognize_program_outputs(&program, &laws_of(&program), ArithmeticType::F32)
        .expect("the real output walk recognizes a gather");
    let [output] = recognized.outputs() else {
        panic!("the fixture declares one output");
    };
    let gather = output.gather().expect("the recognized shape is a gather");
    assert_eq!(gather.source_input, 0, "the source is declared first");
    assert_eq!(gather.index_input, 1, "the index is declared second");
    assert_eq!(gather.source_shape, Shape::from_dims([4, 2]));
    assert_eq!(gather.index_shape, Shape::from_dims([3]));
    assert_eq!(
        gather.result_shape,
        Shape::from_dims([3, 2]),
        "the index shape splices into the source at the gathered axis",
    );
    assert_eq!(gather.axis, Axis::new(0));
    assert_eq!(
        gather.index_access,
        AccessOrdinal::new(1),
        "the address read is canonical local access 1",
    );
    assert_eq!(
        [
            gather.source_elements,
            gather.index_elements,
            gather.result_elements
        ],
        [8, 3, 6],
    );
}

/// A gather whose source is not a declared program input is refused by name.
///
/// The accepted surface admits only declared program inputs as either gather
/// operand. This drives the `gather-operand-input` refusal specifically, rather
/// than letting such a program fall through to a neighbouring rule.
///
/// **The perturbation is on the subject, and it is one edge.** The fixture is
/// [`gather_program`] with a single `F32Multiply` interposed on the source, so
/// the gathered-from value is computed rather than declared while its type,
/// shape, the index operand, the axis, and the gather occurrence itself are all
/// unchanged. Removing that one multiply restores the recognized shape the test
/// above asserts, which is what shows this refusal is about the operand's source
/// and not about the family.
#[test]
fn a_gather_reading_a_computed_source_is_refused_under_operand_input() {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let source = builder
        .input::<F32>(InputKey::new("source").unwrap(), Shape::from_dims([4, 2]))
        .unwrap();
    let index = builder
        .input_resolved(
            InputKey::new("index").unwrap(),
            Shape::from_dims([3]),
            gather_index_resolved_type(),
        )
        .unwrap();
    let one = F32Constant::apply(&mut builder, 1.0_f32.to_bits()).unwrap();
    let computed = F32Multiply::apply(&mut builder, source, one).unwrap();
    let gathered = F32Gather::apply(&mut builder, computed, index, Axis::new(0)).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), gathered)
        .unwrap();
    let program = builder.build().unwrap();
    assert_eq!(
        recognize_program_outputs(&program, &laws_of(&program), ArithmeticType::F32),
        Err(RequestError::UnsupportedCapability {
            phase: "strategy",
            rule: "gather-operand-input",
        }),
    );
}

/// Builds a gather fixture over stated shapes and a stated gathered axis.
fn gather_program_over(source: [u64; 2], index: [u64; 1], axis: u32) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let source = builder
        .input::<F32>(InputKey::new("source").unwrap(), Shape::from_dims(source))
        .unwrap();
    let index = builder
        .input_resolved(
            InputKey::new("index").unwrap(),
            Shape::from_dims(index),
            gather_index_resolved_type(),
        )
        .unwrap();
    let gathered = F32Gather::apply(&mut builder, source, index, Axis::new(axis)).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), gathered)
        .unwrap();
    builder.build().unwrap()
}

/// The gather source relation takes its own request tag and encodes injectively.
///
/// **The tag is checked against the whole named space rather than against one
/// neighbour.** `encode_access_relation` writes `0x01`, `0x02`, `0x03`, `0x05`,
/// and the refusal `0x00`, and `UNREAD_DECLARED_INPUT_TAG` occupies `0x04` in
/// the run this encoder's output sits inside — so a gather taking any of those
/// would either collide with a relation or forge the unread-input marker. `0x06`
/// is the first value above all of them.
///
/// **`0x06` is deliberately not the schedule layer's `0x0C` for the same
/// relation.** Tag spaces here are per-frame, so the two frames each assign
/// their own next free value; this assertion pins the request frame's, and the
/// schedule frame's is pinned in `tiler-ir`.
///
/// Watched failing under three separate subject perturbations, each on the
/// encoder rather than on the assertion:
/// writing the gather at `PARAMETRIC_BROADCAST_ACCESS_TAG` collapses the first
/// assertion; writing it at `UNREAD_DECLARED_INPUT_TAG` collapses the second;
/// and swapping the source and index shape frames collapses the third, because
/// the two shapes differ.
#[test]
fn the_gather_source_relation_takes_its_own_request_tag_and_encodes_injectively() {
    let relation = |axis: u32, index_access: u32| LogicalAccess::GatherSource {
        source_shape: Shape::from_dims([4, 2]),
        result_shape: Shape::from_dims([3, 2]),
        axis: Axis::new(axis),
        index_access: AccessOrdinal::new(index_access),
        index_shape: Shape::from_dims([3]),
    };
    let encode = |map: &LogicalAccess| {
        let mut bytes = Vec::new();
        encode_access_relation(&mut bytes, map);
        bytes
    };

    let gather = encode(&relation(0, 1));
    assert_eq!(
        gather.first().copied(),
        Some(0x06),
        "the gather source relation takes the request frame's next free tag",
    );

    // **The distinctness check is derived from the encoder, not compared to the
    // literal above, and the separation is deliberate.** Asserting that the
    // gather's tag differs from each named constant would be unreachable by
    // pigeonhole: any perturbation of the gather tag trips the pin first, so
    // those assertions could never be the ones to fail and would prove only that
    // the pin runs. Collecting the encoder's *own* answers and requiring them
    // pairwise distinct fails independently — moving `LinearIdentity` onto
    // `0x06` reddens this while leaving the pin above green.
    //
    // `LinearIdentity` and the gather are the two relations constructible here
    // without a program fixture; the parametric carrier's tag is asserted from
    // its constant, and the reindex and replication tags are covered by the
    // pinned subject goldens elsewhere in this module.
    let mut tags: Vec<u8> = [
        encode(&LogicalAccess::LinearIdentity),
        encode(&LogicalAccess::ScalarBroadcast),
        gather.clone(),
    ]
    .iter()
    .filter_map(|bytes| bytes.first().copied())
    // The refusal tag is written for every relation this encoder declines, so
    // several relations legitimately share it and it is not part of the
    // distinct population.
    .filter(|tag| *tag != 0x00)
    .collect();
    let written = tags.len();
    tags.sort_unstable();
    tags.dedup();
    assert_eq!(
        tags.len(),
        written,
        "two encodable access relations share a request-subject tag: {tags:?}",
    );
    assert!(
        !tags.contains(&UNREAD_DECLARED_INPUT_TAG),
        "no relation may forge the unread-declared-input marker: {tags:?}",
    );
    assert!(
        !tags.contains(&PARAMETRIC_BROADCAST_ACCESS_TAG),
        "these relations are not the parametric carrier: {tags:?}",
    );

    // Each member the relation carries separates two encodings on its own.
    assert_ne!(
        gather,
        encode(&relation(1, 1)),
        "the gathered axis is identity",
    );
    assert_ne!(
        gather,
        encode(&relation(0, 2)),
        "the owned address read's local ordinal is identity",
    );
}

/// The `gather-f32.v1` output subject separates every field it carries.
///
/// **The arm is encoded directly rather than through the whole request subject,
/// and that is what makes this test able to fail.** A request subject opens with
/// the semantic graph identity, which already separates any two *programs* that
/// differ in a gather's axis or shapes — so a whole-subject comparison stays
/// green with a field dropped from this arm entirely, and would be asserting the
/// graph identity rather than the projection. Leaning on the enclosing subject
/// to separate arms is exactly the unstated invariant
/// [`encode_elementwise_reads`]'s own documentation forbids resting identity on.
///
/// **Two of these perturbations are unreachable from any program**, which is the
/// other reason the shape is a forge rather than a fixture pair. Declaration
/// order fixes `source_input`/`index_input`, and canonical access order fixes
/// `index_access` at one, so swapping the declared association or moving the
/// owned address ordinal cannot be expressed by authoring a different program.
/// The association swap is the load-bearing one: it is the ADR 0108
/// schedule-clause amendment's central claim that the checked
/// declared-input association lives *here*, in the compiler-private request
/// subject, and nowhere in shared schedule identity.
///
/// Watched failing under a deliberate subject perturbation: dropping
/// `normalized.axis` from `encode_output_subject`'s gather arm reddens the first
/// row with `the gathered axis must move the subject`, while leaving the whole
/// request subject's own goldens green — which is the defect the direct
/// encoding exists to catch.
#[test]
fn a_gather_output_subject_separates_every_field_it_carries() {
    let program = gather_program_over([4, 4], [4], 0);
    let normalized = select_supported_strategy(&program, &laws_of(&program))
        .expect("the gather fixture is recognized");
    let [recognized] = normalized.outputs() else {
        panic!("the fixture declares one output");
    };
    let encoded = |output: &NormalizedOutput| {
        let mut bytes = Vec::new();
        encode_output_subject(&mut bytes, &output_subject(output));
        bytes
    };
    let forge = |edit: fn(&mut NormalizedGather)| {
        let mut forged = recognized.clone();
        let NormalizedOutput::Gather(gather) = &mut forged else {
            panic!("the fixture recognizes as a gather");
        };
        edit(gather);
        encoded(&forged)
    };

    let base = encoded(recognized);
    assert!(!base.is_empty(), "the gather arm encodes a subject");

    for (label, forged) in [
        (
            "the gathered axis",
            forge(|gather| gather.axis = Axis::new(1)),
        ),
        (
            "the declared source/index association",
            forge(|gather| std::mem::swap(&mut gather.source_input, &mut gather.index_input)),
        ),
        (
            "the owned address read's local ordinal",
            forge(|gather| gather.index_access = AccessOrdinal::new(2)),
        ),
        (
            "the source shape",
            forge(|gather| gather.source_shape = Shape::from_dims([5, 4])),
        ),
        (
            "the index shape",
            forge(|gather| gather.index_shape = Shape::from_dims([3])),
        ),
        (
            "the result shape",
            forge(|gather| gather.result_shape = Shape::from_dims([4, 5])),
        ),
        (
            "the claimed occurrence",
            forge(|gather| gather.member = SemanticMemberId(gather.member.0 + 1)),
        ),
        (
            "the source element count",
            forge(|gather| gather.source_elements += 1),
        ),
        (
            "the index element count",
            forge(|gather| gather.index_elements += 1),
        ),
        (
            "the result element count",
            forge(|gather| gather.result_elements += 1),
        ),
    ] {
        assert_ne!(base, forged, "{label} must move the subject");
    }
}

/// A gather takes its own output sub-tag, and no other arm's bytes move.
///
/// The sub-tag is what keeps `tiler.compiler.request-subject.v6` from stepping:
/// a gather is a subject the earlier vocabulary could not express at all, so
/// every previously encodable output still encodes to exactly what it did. The
/// second half of that claim is carried by the module's existing pinned
/// subjects, which this lane did not touch and which pass unchanged; this states
/// the first half.
#[test]
fn a_gather_output_subject_takes_its_own_sub_tag() {
    let program = gather_program_over([4, 2], [3], 0);
    let normalized = select_supported_strategy(&program, &laws_of(&program))
        .expect("the gather fixture is recognized");
    let [recognized] = normalized.outputs() else {
        panic!("the fixture declares one output");
    };
    let mut bytes = Vec::new();
    encode_output_subject(&mut bytes, &output_subject(recognized));
    let tag = b"gather-f32.v1";
    assert!(
        bytes
            .windows(tag.len())
            .any(|window| window == tag.as_slice()),
        "the gather arm writes its own framed sub-tag",
    );
    for other in [
        b"pointwise-f32.v4".as_slice(),
        b"contraction-f32.v1".as_slice(),
        b"serial-sum-f32.v3".as_slice(),
        b"epilogue-f32.v1".as_slice(),
        b"staged-family.v2".as_slice(),
    ] {
        assert!(
            !bytes.windows(other.len()).any(|window| window == other),
            "a gather subject must not carry another arm's sub-tag",
        );
    }
}

/// A gather's spelling follows its own occurrence's bounds evidence.
///
/// **The two halves are the same call over the same code path, separated only by
/// which record the occurrence's own lowering minted.** That is what makes this a
/// discrimination rather than a pair of unrelated assertions: before
/// `RegionVocabularyWall::GatherIndexBoundsUnproved` existed, the wall declined
/// every gather member set unconditionally, so a statically proved gather and one
/// owing invocation validation were indistinguishable above lowering. Each half
/// is now reachable, and neither would be if the seam carrying the proof into
/// `spell_region` were absent — the whole population would collapse back onto the
/// refusal, which is exactly the state a relaxation-shaped repair would leave.
///
/// - `TRANSPLANT_SOURCE_EXTENT` reaches `2^32`, so the closed deriver discharges
///   the obligation with `U32RangeContainedBySourceExtent` and the region is
///   spelled.
/// - `gather_program`'s `[4, 2]` source reaches neither closed argument over an
///   inhabited result domain, so the deriver mints a
///   `GatherIndexValidationRequirement` and the occurrence is declined **by
///   name**. It is refused rather than admitted with a weaker record: the receipt
///   that would discharge one is
///   [`admit-an-invocation-scoped-gather-index-validation-receipt`](../../tickets/admit-an-invocation-scoped-gather-index-validation-receipt.md)'s
///   vocabulary, and admitting it here would open a fail-closed boundary.
///
/// **What it would take for either half to say something else, and whether that
/// case is reachable.** The proved half stops holding if the lowering resolves no
/// single realized region, if that region's gather access stops carrying a static
/// proof, or if the arm stops consulting the lowering at all — all three
/// reachable, and the third is the relaxation this ticket forbids. The unproved
/// half stops holding if the arm admits a requirement as though it were a proof,
/// which is the same relaxation seen from the other side. The `unresolved_for_test`
/// case below is the third reachable cause: a caller that never resolved a
/// lowering cannot obtain the spelling by omission.
///
/// Watched failing under a deliberate subject perturbation: replacing
/// `gather_bounds_proof(lowering, normalized.member).is_some()` with `true` —
/// which is precisely "the check was relaxed" wearing the costume of "the
/// argument arrived" — leaves the proved half green and reddens the unresolved
/// assertion, which is the first of the two the relaxation reaches, with
/// `left: Ok(RegionSpelling { output: 0, kind: Gather })  right: Err(GatherIndexBoundsUnproved)`.
/// The unproved assertion below it carries the identical text. The same
/// perturbation additionally reddens
/// `a_governed_gather_refuses_at_dispatch_then_at_the_region_vocabulary` at
/// `crates/tiler-compiler/src/frontier.rs`'s `a gather spelling is decided before
/// the region is built`, because `gather_region` then has no proof to embed —
/// which is what makes that expectation a statement about ordering rather than a
/// hope.
#[test]
fn a_gathers_spelling_follows_its_own_occurrences_bounds_evidence() {
    let spell = |program: &SemanticProgram,
                 target: &VerifiedTargetRequest,
                 members: &[crate::region::SemanticStage]| {
        let lowering = crate::lowering::resolve_lowering(program, target)
            .expect("the governed gather capability lowers a recognized gather");
        crate::physical::spell_region(
            target,
            members,
            crate::physical::RegionWrite::ProgramOutput,
            &lowering,
        )
    };

    // The proved population: a gathered extent containing every U32 value.
    let (proved_program, proved_target) = transplant_gather_target();
    let [proved_output] = proved_target.normalized().outputs() else {
        panic!("the fixture declares one output");
    };
    let proved_members = proved_output.members();
    assert_eq!(
        proved_members.len(),
        1,
        "a gather claims exactly one occurrence"
    );
    let spelling = spell(&proved_program, &proved_target, &proved_members)
        .expect("a statically proved gather is spelled by the governed vocabulary");
    assert_eq!(spelling.kind(), crate::physical::RegionSpellingKind::Gather);
    assert_eq!(
        spelling.output(),
        0,
        "the spelling resolves the declared output whose partition it belongs to",
    );

    // An unresolved lowering resolves no member, so the same proved occurrence
    // is refused: the spelling is evidence read out of a lowering, never an
    // answer derived from the member set alone.
    assert_eq!(
        crate::physical::spell_region(
            &proved_target,
            &proved_members,
            crate::physical::RegionWrite::ProgramOutput,
            &crate::lowering::ResolvedLowering::unresolved_for_test(),
        ),
        Err(crate::physical::RegionVocabularyWall::GatherIndexBoundsUnproved),
    );

    // The unproved population: an inhabited result over a source extent neither
    // closed argument covers.
    let unproved_program = gather_program();
    let mut request = CompilationRequest::governed(&unproved_program);
    request.target_profiles = vec![TargetProfile::governed_with_gather_index_dispatch_for_test()];
    let planned = verify_planned_request(request).expect("the fixture admits a planned request");
    let unproved_target = planned
        .for_target(0)
        .expect("the U32-capable profile admits the fixture");
    let [unproved_output] = unproved_target.normalized().outputs() else {
        panic!("the fixture declares one output");
    };
    let unproved_members = unproved_output.members();
    assert_eq!(
        spell(&unproved_program, &unproved_target, &unproved_members),
        Err(crate::physical::RegionVocabularyWall::GatherIndexBoundsUnproved),
        "a gather owing invocation validation is declined by name, not admitted \
beside the proved one",
    );
    assert_eq!(
        crate::physical::RegionVocabularyWall::GatherIndexBoundsUnproved.reason(),
        "gather-index-bounds-unproved",
    );
    // And the refusal really is this occurrence's own evidence rather than a
    // missing lowering: the same program resolves one.
    let unproved_lowering = crate::lowering::resolve_lowering(&unproved_program, &unproved_target)
        .expect("a gather this build cannot prove statically still lowers");
    let [occurrence] = unproved_lowering.occurrences() else {
        panic!("the fixture declares one gather occurrence");
    };
    let crate::lowering::OccurrenceEvidence::Refined(refinement) = occurrence.evidence();
    assert!(
        refinement
            .single_region()
            .expect("a gather is realized by one region, not a chain")
            .accesses()
            .find_map(|access| match access.view() {
                tiler_ir::index::TensorAccessView::GatherRead(gather) =>
                    Some(gather.bounds_resolution()),
                tiler_ir::index::TensorAccessView::Direct(_) => None,
            })
            .expect("the realized region carries the gather access")
            .invocation_validation_required()
            .is_some(),
        "the declined half must owe invocation validation, or it is refused for \
some other reason and proves nothing about this wall",
    );

    // A member set that is not this occurrence's falls through to the caller's
    // own wall, which is what separates "this region cannot be built" from
    // "this cover names occurrences no output owns".
    let foreign = [crate::region::SemanticStage::first(
        crate::region::SemanticMemberId(proved_members[0].member().0 + 1),
    )];
    assert_eq!(
        spell(&proved_program, &proved_target, &foreign),
        Err(crate::physical::RegionVocabularyWall::PartialCoverage),
    );
}

/// The spelled gather region is the one the request-subject binding admits.
///
/// **The builder and the binding are two accounts of one region, and this is
/// what stops them drifting.** `gather_region` places the source read, the
/// address read it owns, and the write in the canonical order, derives the
/// address relation through `gather_index_read_map`, and embeds the proof the
/// occurrence's realization retained; `gather_accesses_match` re-derives every
/// one of those independently and compares. A builder that named the wrong
/// ordinal, proposed `LinearIdentity` for a replicating index, or embedded a
/// proof from another region would produce a region this call refuses — and
/// through `enumerate_frontier` such a region is a `MalformedProposal`, not a
/// graceful decline, so the agreement is load-bearing rather than cosmetic.
///
/// It is also the positive control the whole ticket rests on: the wall retired
/// because a region can now be built *and admitted*, not because the check
/// stopped running. Hard feasibility, intrinsic verification, and the
/// numerical-realization comparison all run here unchanged.
///
/// **The negative control transplants the proof out of the spelled region**, so
/// the admission above cannot be a binding that admits every gather. It is the
/// same region this vocabulary just built, with one field replaced by a proof
/// minted for a shape-compatible sibling — the occupancy conjunct
/// `a_gather_proof_minted_for_another_region_is_refused` introduced, asked here
/// of the *spelled* region rather than of a hand-built fixture, which is what
/// shows the threading did not weaken it.
///
/// Watched failing under a deliberate subject perturbation: swapping
/// `gather_region`'s two reads so the address read is placed at local access 0
/// reddens this with
/// `left: Some(Intrinsic { rule: "gather-address-read-not-later", region: RegionId(0) })  right: None`.
/// The rule is `tiler_ir::schedule`'s rather than this crate's
/// `request-binding`, and the difference is worth keeping: the intrinsic
/// verifier owns the canonical-order obligation and refuses the swap before the
/// request-subject binding is asked anything, so the two checks are not two
/// spellings of one rule.
#[test]
fn the_spelled_gather_region_binds_its_own_request_subject() {
    let (program, target) = transplant_gather_target();
    let [output] = target.normalized().outputs() else {
        panic!("the fixture declares one output");
    };
    let lowering = crate::lowering::resolve_lowering(&program, &target)
        .expect("the governed gather capability lowers a recognized gather");
    let (region, members) = crate::physical::gather_region(
        &target,
        output,
        crate::physical::RegionWrite::ProgramOutput,
        &lowering,
    )
    .expect("a statically proved gather has a governed region");
    assert_eq!(
        members,
        output.members(),
        "the built region claims exactly the occurrences the recognizer did",
    );
    let verified = crate::physical::verify_schedule_with_feasibility(
        region.clone(),
        members.clone(),
        &target,
        &lowering,
    );
    assert_eq!(
        verified.as_ref().err(),
        None,
        "the region this vocabulary spells must bind the subject it claims",
    );

    // The same spelled region carrying a proof minted for a different region.
    // Every fact the binding compared before the occupancy conjunct existed is
    // equal between the two proofs, so only that conjunct can refuse it.
    let mut transplanted = region;
    let crate::physical::BoundsProofKind::GatherSource { proof, .. } =
        &mut transplanted.index.bounds_proofs[0].kind
    else {
        panic!("the spelled region proves its source read as a gather");
    };
    let foreign = mint_gather_proof(true);
    assert_eq!(
        foreign.source_shape(),
        proof.source_shape(),
        "the transplant must agree on every pre-existing comparison, or it \
refuses for a reason that is not the occupancy check",
    );
    assert_ne!(foreign.region().as_bytes(), proof.region().as_bytes());
    **proof = foreign;
    assert_eq!(
        crate::physical::verify_schedule_with_feasibility(
            transplanted,
            members,
            &target,
            &lowering,
        )
        .as_ref()
        .err(),
        Some(&crate::physical::PhysicalError::Intrinsic {
            rule: "request-binding",
            region: tiler_ir::schedule::RegionId::new(0),
        }),
        "a proof minted for another region must not bind the spelled one",
    );
}

/// The gathered source extent of the transplant fixture, chosen to reach `2^32`.
///
/// Only a *statically proved* gather carries a `GatherIndexBoundsProof` at all,
/// and the two closed arguments are an empty result domain and a gathered extent
/// containing the whole U32 space. `gather_program()`'s `[4, 2]` source reaches
/// neither, so it mints a validation requirement rather than a proof and cannot
/// stage this subject. The inhabited argument is taken over the vacuous one for
/// the reason `tiler-ir`'s own gather fixture takes it: an empty domain makes
/// every downstream count zero and could hide an arithmetic defect.
const TRANSPLANT_SOURCE_EXTENT: u64 = 1 << 32;

/// The result domain is four points, and the size is load-bearing.
///
/// `governed_with_gather_index_dispatch_for_test` admits four grid threads, so a
/// wider fixture is refused for `grid-axis` hard feasibility *before* the
/// binding's verdict is observable — which would leave the transplant assertion
/// green for a reason that has nothing to do with the proof.
const TRANSPLANT_RESULT_ELEMENTS: u64 = 4;

fn transplant_source_shape() -> Shape {
    Shape::from_dims([TRANSPLANT_SOURCE_EXTENT, 2])
}

fn transplant_index_shape() -> Shape {
    Shape::from_dims([2])
}

fn transplant_result_shape() -> Shape {
    Shape::from_dims([2, 2])
}

/// Mints one real static gather proof through `tiler-ir`'s public index builder.
///
/// **Nothing here is a hand-built proof.** `GatherIndexBoundsProof` has no public
/// constructor and its fields are crate-private, so the only way to hold one is
/// to build a verified index region whose gather access the closed deriver
/// proved — which is exactly the route an out-of-crate provider has, and exactly
/// why the evidence cannot be withheld.
///
/// `index_tensor_first` selects between two regions that agree on every fact
/// [`crate::physical`]'s gather binding compares — source shape, result shape,
/// index shape, axis, and the owned address ordinal — and differ only in the
/// order the two input boundaries are declared. That is enough to move
/// `CanonicalIndexRegionIdentity`, which folds the tensor list, so the pair is a
/// *shape-compatible different region*: the subject the transplant needs.
fn mint_gather_proof(index_tensor_first: bool) -> tiler_ir::index::GatherIndexBoundsProof {
    use tiler_ir::index::{DomainRole, IndexRegionBuilder, TensorAccessView};

    let registry = tiler_ir::index::FrozenScalarRegistry::standard().expect("the profile composes");
    let mut builder = IndexRegionBuilder::new(registry).expect("a builder is admitted");
    let dimensions: Vec<_> = transplant_result_shape()
        .extents()
        .iter()
        .map(|extent| {
            builder
                .dimension(DomainRole::Parallel, *extent)
                .expect("a parallel dimension is admitted")
        })
        .collect();
    let declare_source = |builder: &mut IndexRegionBuilder| {
        builder
            .tensor(
                tiler_ir::index::TensorRole::Input,
                F32::resolved_type().clone(),
                transplant_source_shape(),
            )
            .expect("the source boundary is admitted")
    };
    let declare_index = |builder: &mut IndexRegionBuilder| {
        builder
            .tensor(
                tiler_ir::index::TensorRole::Input,
                gather_index_resolved_type(),
                transplant_index_shape(),
            )
            .expect("the index boundary is admitted")
    };
    let (source, index) = if index_tensor_first {
        let index = declare_index(&mut builder);
        (declare_source(&mut builder), index)
    } else {
        let source = declare_source(&mut builder);
        (source, declare_index(&mut builder))
    };
    let output = builder
        .tensor(
            tiler_ir::index::TensorRole::Output,
            F32::resolved_type().clone(),
            transplant_result_shape(),
        )
        .expect("the output boundary is admitted");
    // Result axes are [source before axis | index | source after axis]. The
    // gather is on axis 0, so the index run leads and the one remaining source
    // axis trails: the source coordinate is result dimension 1.
    let source_coordinates = vec![
        builder
            .dimension_expr(dimensions[1])
            .expect("a dimension coordinate is admitted"),
    ];
    let index_coordinates = vec![
        builder
            .dimension_expr(dimensions[0])
            .expect("a dimension coordinate is admitted"),
    ];
    let value = builder
        .gather_read(
            source,
            index,
            &dimensions,
            &source_coordinates,
            &index_coordinates,
            Axis::new(0),
        )
        .expect("the gather is admitted");
    let write_coordinates: Vec<_> = dimensions
        .iter()
        .map(|dimension| {
            builder
                .dimension_expr(*dimension)
                .expect("a dimension coordinate is admitted")
        })
        .collect();
    let write = builder
        .write(output, &dimensions, &write_coordinates)
        .expect("the write is admitted");
    builder
        .output(write, value)
        .expect("the output is admitted");
    let region = builder.build().expect("the index region verifies");
    let proof = region
        .accesses()
        .find_map(|access| match access.view() {
            TensorAccessView::GatherRead(gather) => gather.bounds_resolution().statically_proved(),
            TensorAccessView::Direct(_) => None,
        })
        .expect("a gathered extent containing every U32 value discharges the obligation")
        .clone();
    assert_eq!(
        proof.kind(),
        tiler_ir::index::GatherIndexBoundsProofKind::U32RangeContainedBySourceExtent,
        "the fixture must rest on the inhabited argument, not on vacuity",
    );
    proof
}

/// Builds the one scheduled region a recognized gather occurrence would bind.
///
/// Canonical order for the one-gather occurrence the accepted surface admits:
/// the `f32` source read at local access 0, the address-only U32 read it owns at
/// access 1, and the owning write at access 2.
fn transplant_gather_region(
    target: &VerifiedTargetRequest,
    proof: tiler_ir::index::GatherIndexBoundsProof,
) -> crate::physical::ScheduledRegion {
    use crate::physical::{
        Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, BoundsWitnessId,
        ExecutionBinding, KernelSchedule, LaunchPlan, LogicalAccess, OwnershipProof,
        OwnershipProofKind, OwnershipWitnessId, ReductionTopology, RegionId, RegionProgram,
        ScalarProgram, TailPolicy, TensorRole,
    };

    let relation = LogicalAccess::GatherSource {
        source_shape: transplant_source_shape(),
        result_shape: transplant_result_shape(),
        axis: Axis::new(0),
        index_access: AccessOrdinal::new(1),
        index_shape: transplant_index_shape(),
    };
    let address = tiler_ir::schedule::gather_index_read_map(
        &transplant_source_shape(),
        Axis::new(0),
        &transplant_index_shape(),
    )
    .expect("the fixture is a well-formed gather");
    let read = |map: LogicalAccess, bounds: u32| Access {
        tensor: TensorRole::Input,
        component_role: None,
        mode: AccessMode::Read,
        map,
        bounds: BoundsWitnessId::new(bounds),
        ownership: None,
    };
    let accesses = vec![
        read(relation, 0),
        read(address.clone(), 1),
        Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(2),
            ownership: Some(OwnershipWitnessId::new(0)),
        },
    ];
    let bounds_proofs = vec![
        BoundsProof {
            id: BoundsWitnessId::new(0),
            tensor: TensorRole::Input,
            component_role: None,
            kind: BoundsProofKind::GatherSource {
                source_shape: transplant_source_shape(),
                result_shape: transplant_result_shape(),
                axis: Axis::new(0),
                index_access: AccessOrdinal::new(1),
                index_shape: transplant_index_shape(),
                proof: Box::new(proof),
            },
        },
        BoundsProof {
            id: BoundsWitnessId::new(1),
            tensor: TensorRole::Input,
            component_role: None,
            // Derived from the address read's own relation rather than restated:
            // `gather_index_read_map` answers a replicating relation for this
            // fixture, whose bounded population is the index operand's two
            // elements and not the result domain's six.
            kind: BoundsProofKind::LinearRange {
                element_count: match &address {
                    LogicalAccess::BroadcastReplication { operand_shape, .. } => {
                        tiler_ir::schedule::element_count(operand_shape)
                            .expect("a bounded index operand")
                    }
                    _ => TRANSPLANT_RESULT_ELEMENTS,
                },
            },
        },
        BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: TRANSPLANT_RESULT_ELEMENTS,
            },
        },
    ];
    let mut expression = tiler_ir::schedule::PointwiseF32ExpressionBuilder::new();
    let root = expression
        .input(AccessOrdinal::FIRST)
        .expect("the one f32 leaf is admitted");
    let expression = expression.build(root).expect("the identity composes");
    crate::physical::ScheduledRegion {
        index: crate::physical::IndexRegion {
            id: RegionId::new(0),
            iteration_shape: transplant_result_shape(),
            accesses,
            bounds_proofs,
            ownership_proof: OwnershipProof {
                id: OwnershipWitnessId::new(0),
                tensor: TensorRole::Output,
                kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                    output_count: TRANSPLANT_RESULT_ELEMENTS,
                },
            },
            program: RegionProgram::Numerical {
                scalar: ScalarProgram::PointwiseF32(expression),
                // The region must implement *this request's* resolved
                // realization; a value re-derived here would be a second
                // account of the contract free to disagree with it.
                numerical: target.numerical_contract().realization(),
            },
        },
        schedule: KernelSchedule {
            binding: ExecutionBinding::GlobalLinearInvocation,
            work_items: TRANSPLANT_RESULT_ELEMENTS,
            threads_per_workgroup: 1,
            tail: TailPolicy::Exact,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::None,
            launch: LaunchPlan {
                grid_threads: TRANSPLANT_RESULT_ELEMENTS,
                threads_per_workgroup: 1,
                zero_work_skips_dispatch: true,
            },
        },
    }
}

/// Builds the transplant fixture's program and the verified target request its
/// occurrence belongs to.
///
/// The program is returned beside the request because resolving the
/// occurrence's real lowering needs both, and re-deriving it at the call site
/// would be a second fixture free to drift from the one the request verified.
fn transplant_gather_target() -> (SemanticProgram, VerifiedTargetRequest) {
    let program = gather_program_over([TRANSPLANT_SOURCE_EXTENT, 2], [2], 0);
    let mut request = CompilationRequest::governed(&program);
    request.target_profiles = vec![TargetProfile::governed_with_gather_index_dispatch_for_test()];
    let planned = verify_planned_request(request).expect("the fixture admits a planned request");
    let target = planned
        .for_target(0)
        .expect("the U32-capable profile admits the fixture");
    (program, target)
}

/// Reads the statically proved gather bounds proof out of the region one
/// occurrence's own resolved lowering realized.
///
/// **Nothing here fabricates a proof or an identity.** The proof is the one the
/// index layer's closed deriver minted while the governed provider's region was
/// built, so its `CanonicalIndexRegionIdentity` is by construction the realized
/// region's — which is exactly the conjunct
/// [`a_gather_proof_minted_for_another_region_is_refused`] shows a transplant
/// cannot satisfy.
fn own_gather_proof(
    lowering: &crate::lowering::ResolvedLowering,
) -> tiler_ir::index::GatherIndexBoundsProof {
    use tiler_ir::index::TensorAccessView;

    let [occurrence] = lowering.occurrences() else {
        panic!("the fixture declares one gather occurrence");
    };
    let crate::lowering::OccurrenceEvidence::Refined(refinement) = occurrence.evidence();
    refinement
        .single_region()
        .expect("a gather is realized by one region, not a chain")
        .accesses()
        .find_map(|access| match access.view() {
            TensorAccessView::GatherRead(gather) => gather.bounds_resolution().statically_proved(),
            TensorAccessView::Direct(_) => None,
        })
        .expect("the realized region's gather access discharges its obligation statically")
        .clone()
}

/// A retained gather proof minted for another region is refused by the binding.
///
/// **The two proofs are shape-compatible and region-distinct, and both halves
/// are asserted rather than assumed.** Every fact the request binding compared
/// before this change — source shape, result shape, index shape, axis, and the
/// owned address ordinal — is equal between them, so none of those comparisons
/// can separate the pair; their `CanonicalIndexRegionIdentity` differs, so the
/// new one can. That is the exact state the finding is about: a proof minted for
/// region A, attached to a shape-compatible occurrence B, staying *bounds*-sound
/// while corrupting identity, because both closed proof kinds are functions of
/// the source shape, the index shape, and the axis and the schedule folds the
/// proof identity — which folds A's region — into its own.
///
/// **What it would take for this to say something else, and whether that case is
/// reachable.** The refusal has two independent causes and only one of them is
/// the transplant. `gather_accesses_match` admits a proof when the occurrence's
/// lowering realized a single region *and* that region's identity is the
/// proof's. Both conjuncts are now reachable: the governed profile carries a
/// `tiler::gather-f32@1` index-access capability, so `resolve_lowering` answers
/// `Ok` for this fixture and holds a real realized region —
/// [`a_gather_occurrence_resolves_a_governed_lowering_and_refines`] pins that —
/// and the second half of this test is therefore the **positive control** the
/// absence used to preclude, run against the same scheduled region, the same
/// members, and the same target, with only the proof and the resolved lowering
/// changed.
///
/// The perturbation that shows the new comparison is what refuses the
/// transplant is deleting the `realized == Some(proof.region())` conjunct, which
/// was run: the transplanted region is then **admitted whole** — intrinsic
/// verification, the request-subject binding, and hard feasibility all pass —
/// and the first assertion reddens with
/// `left: None  right: Some(Intrinsic { rule: "request-binding", region: RegionId(0) })`.
#[test]
fn a_gather_proof_minted_for_another_region_is_refused() {
    let own = mint_gather_proof(false);
    let transplant = mint_gather_proof(true);

    // The pre-existing comparisons cannot separate the pair.
    assert_eq!(own.source_shape(), transplant.source_shape());
    assert_eq!(own.result_shape(), transplant.result_shape());
    assert_eq!(own.index_shape(), transplant.index_shape());
    assert_eq!(own.axis(), transplant.axis());
    assert_eq!(own.kind(), transplant.kind());
    // The new one can.
    assert_ne!(
        own.region().as_bytes(),
        transplant.region().as_bytes(),
        "the fixture must offer two genuinely different regions, or the \
         transplant is a proof swapped for an equal one and corrupts nothing",
    );

    let (program, target) = transplant_gather_target();
    let [output] = target.normalized().outputs() else {
        panic!("the fixture declares one output");
    };
    let members = output.members();
    assert_eq!(members.len(), 1, "a gather claims exactly one occurrence");
    let resolved = crate::lowering::resolve_lowering(&program, &target)
        .expect("the governed gather capability lowers this occurrence");

    let refusal = crate::physical::verify_schedule_with_feasibility(
        transplant_gather_region(&target, transplant),
        members.clone(),
        &target,
        &resolved,
    );
    // Projected to the error alone so an admission prints `None` rather than a
    // whole verified region; the discrimination is unchanged, because `None` is
    // reachable only from `Ok`.
    assert_eq!(
        refusal.as_ref().err(),
        Some(&crate::physical::PhysicalError::Intrinsic {
            rule: "request-binding",
            region: tiler_ir::schedule::RegionId::new(0),
        }),
        "a proof minted for another region must not bind this occurrence",
    );

    // The positive control: the occurrence's *own* proof, read out of the very
    // region its resolved lowering realized, admits the identical scheduled
    // region. Without it the refusal above would be indistinguishable from a
    // binding that refuses every gather.
    let admitted = crate::physical::verify_schedule_with_feasibility(
        transplant_gather_region(&target, own_gather_proof(&resolved)),
        members.clone(),
        &target,
        &resolved,
    );
    assert_eq!(
        admitted.as_ref().err(),
        None,
        "the occurrence's own realized region's proof must bind it",
    );

    // And the unresolved lowering still refuses, so a caller that never
    // resolved one cannot obtain the admission above by omission.
    let unresolved = crate::physical::verify_schedule_with_feasibility(
        transplant_gather_region(&target, own_gather_proof(&resolved)),
        members.clone(),
        &target,
        &crate::lowering::ResolvedLowering::unresolved_for_test(),
    );
    assert_eq!(
        unresolved.as_ref().err(),
        Some(&crate::physical::PhysicalError::Intrinsic {
            rule: "request-binding",
            region: tiler_ir::schedule::RegionId::new(0),
        }),
        "an unresolved lowering resolves no member, so it binds nothing",
    );
}

/// A recognized gather resolves the governed capability and refines to a region
/// carrying its own statically proved bounds obligation.
///
/// **This assertion is the inverse of the one it replaces, and the inversion is
/// this lane's landing.** It previously required `resolve_lowering` to answer
/// `Err` with `missing-capability`, because no installed capability lowered a
/// gather at all. The governed profile now registers `tiler::gather-f32@1`, so
/// the same fixture through the same call resolves, and leaving the old
/// expectation would have left the suite asserting the opposite of the tree.
///
/// **Each of the four facts is read from the resolved value rather than
/// inferred from the one before it**, because they are four different claims:
/// that a capability resolved at all; that its emitted realization was proved
/// to *realize the occurrence* rather than merely to verify; that the
/// realization is one region rather than a chain, which is what makes it
/// evaluable and what a schedule can name; and that the region's gather access
/// carries a `GatherIndexBoundsProof` rather than an invocation-validation
/// requirement. The last is the one the schedule layer needs and the reason
/// this fixture gathers along an extent containing every U32 value.
///
/// The proof's region is compared against the realized region's own canonical
/// identity, which is exactly the conjunct
/// [`a_gather_proof_minted_for_another_region_is_refused`] shows a transplanted
/// proof fails.
///
/// **The negative control perturbs the installed authority, not the
/// assertion.** The same fixture, the same target, and the same call against a
/// registry carrying every governed index-access family *except* this one
/// refuses under `missing-capability`. Without it a harness that resolved
/// nothing would read as a landed row, and a `resolve_lowering` that could no
/// longer say `Err` would make every refusal elsewhere in this file vacuous.
#[test]
fn a_gather_occurrence_resolves_a_governed_lowering_and_refines() {
    use tiler_ir::index::TensorAccessView;

    let gather = gather_program_over([TRANSPLANT_SOURCE_EXTENT, 2], [2], 0);
    let mut request = CompilationRequest::governed(&gather);
    request.target_profiles = vec![TargetProfile::governed_with_gather_index_dispatch_for_test()];
    let planned = verify_planned_request(request).expect("the fixture admits a planned request");
    let target = planned
        .for_target(0)
        .expect("the U32-capable profile admits the fixture");
    let lowering = crate::lowering::resolve_lowering(&gather, &target)
        .expect("the governed gather capability lowers a recognized gather");
    let [occurrence] = lowering.occurrences() else {
        panic!("the fixture declares one gather occurrence");
    };
    let crate::lowering::OccurrenceEvidence::Refined(refinement) = occurrence.evidence();
    let region = refinement
        .single_region()
        .expect("a gather is realized by one region, not a chain");
    let proof = region
        .accesses()
        .find_map(|access| match access.view() {
            TensorAccessView::GatherRead(gather) => gather.bounds_resolution().statically_proved(),
            TensorAccessView::Direct(_) => None,
        })
        .expect("the realized region's gather access discharges its obligation statically");
    assert_eq!(
        proof.kind(),
        tiler_ir::index::GatherIndexBoundsProofKind::U32RangeContainedBySourceExtent,
        "the fixture must rest on the inhabited argument, not on vacuity",
    );
    assert_eq!(
        proof.region().as_bytes(),
        region.canonical_identity().as_bytes(),
        "the proof must bind the region this occurrence's lowering realized",
    );

    // The negative control, and it perturbs the installed authority rather than
    // the assertion: `install_governed_index_access` composes the shipped rows
    // minus the substituted family, which is the affordance an external
    // provider replaces one family through.
    let scalars = crate::governed::governed_scalars().expect("the governed scalars compose");
    let mut builder = crate::capability::LoweringCapabilityRegistryBuilder::new(
        scalars.semantic_authority().clone(),
        scalars.clone(),
    )
    .expect("the governed scalar registry retains its exact semantic authority");
    crate::capability::install_governed_index_access(
        &mut builder,
        &[tiler_ir::semantic::gather_f32_op()],
    )
    .expect("the governed rows install onto a fresh builder");
    let mut substituted = CompilationRequest::governed(&gather);
    substituted.target_profiles =
        vec![TargetProfile::governed_with_gather_index_dispatch_for_test()];
    substituted.capabilities = CompilerCapabilitySnapshot::new(builder.freeze(), scalars);
    let planned =
        verify_planned_request(substituted).expect("the fixture admits a planned request");
    let target = planned
        .for_target(0)
        .expect("the U32-capable profile admits the fixture");
    let error = crate::lowering::resolve_lowering(&gather, &target)
        .expect_err("a registry without the gather row cannot lower a gather");
    assert_eq!(error.reason(), "missing-capability");
}

/// A statically proved gather compiles, with its address operand materialized at
/// its own `u32` carrier rather than at the program's arithmetic.
///
/// **The vacuous closed bounds argument is what this fixture uniquely carries
/// down the whole compiler path.** A `[4, 0]` source has an empty result domain,
/// so the index layer discharges the gather's bounds obligation *vacuously* — no
/// `2^32` gathered extent required. Its sibling
/// `a_gathers_spelling_follows_its_own_occurrences_bounds_evidence` asserts its
/// own proof kind is `U32RangeContainedBySourceExtent` and says in its own words
/// that the fixture "must rest on the inhabited argument, not on vacuity", so
/// this is the only compiler-level witness that the other closed argument is
/// spelled and admitted too.
///
/// **This test has pinned two successive walls and now pins their absence.** It
/// asserted `("kernel-lowering", "gather-kernel-body")` until
/// [`lower-the-indirect-gather-read-through-the-structured-kernel-body`](../../tickets/lower-the-indirect-gather-read-through-the-structured-kernel-body.md)
/// emitted the indirect body, and then
/// `InvalidCompilerOutput(Program(CoreConstruction(StageElementType { position: 1, expected: U32, actual: F32 })))`
/// until
/// [`route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type`](../../tickets/route-a-program-inputs-storage-carrier-from-its-own-resolved-value-type.md)
/// routed a declared input's carrier from the input. Both refusals were the same
/// fact seen from two layers: `crate::program`'s `BoundedCarrier::of` answered
/// for the *program's* arithmetic and reached every declared value, so the
/// `tiler::u32@1` index was declared `f32` while the emitted body read it at the
/// exact-width `KernelType::U32`.
///
/// **A successful compile is a weaker assertion than a pinned refusal, so the
/// carriers are read out rather than left implied.** `expect` on the compile
/// says only that nothing refused; the assertions below say *which* carrier each
/// boundary got. Which of them can independently say no is worth stating,
/// because two cannot: `KernelProgramBuilder::build` proves the storage scalar
/// and the element type are one carrier's pair (`StorageAccessType`) and proves
/// the byte count against the declared view (`StageElementCount`,
/// `AccessibleBytesDisagreement`), so `storage_scalar` and `required_bytes` are
/// readouts of what those proofs already force once `element_type` is `U32`. The
/// element type is the one this test's own assertion carries. `alignment` is
/// deliberately not asserted at all: `AlignmentRequirement::natural_for` answers
/// four bytes for `U32` and for `F32` alike, so an assertion on it here could
/// not fail for the cause this test exists to catch.
///
/// Watched failing under four deliberate subject perturbations, each on the code
/// the property is about rather than on the assertion:
///
/// - **The address operand's own carrier.** Answering `StorageScalar::F32` /
///   `KernelType::F32` in `BoundedCarrier::of_input`'s
///   `gather_index_resolved_type` arm restores the old wall and reports
///   `the governed gather compiles once its index carries its own type, got Program(CoreConstruction(StageElementType { position: 1, expected: U32, actual: F32 }))`.
/// - **The two halves of that carrier are one fact.** Answering
///   `StorageScalar::F32` beside `KernelType::U32` — a pair no carrier names —
///   reports
///   `got Program(CoreConstruction(StorageAccessType { scalar: F32, encoding: Unpacked, expected: F32, actual: U32 }))`,
///   which is the proof that makes the `storage_scalar` assertion a readout.
/// - **The width the accessible range is scaled by.** Adding one to
///   `input.carrier.element_bytes()` in `declare_host_abi` reports
///   `got Program(CoreConstruction(AccessibleBytesDisagreement { position: 1, expected: 8, actual: 10 }))`,
///   so the ABI formula really is checked against the declared view rather than
///   merely declared.
/// - **The vacuous proof reaches the spelling path.** Inverting the proof gate
///   in `physical.rs` to `gather_bounds_proof(lowering, normalized.member).is_none()`
///   reports `a vacuously proved gather is spelled by the governed vocabulary: GatherIndexBoundsUnproved`
///   from the earlier `expect`, so the premise is load-bearing rather than
///   incidental setup for the compile below.
#[test]
fn a_statically_proved_gather_compiles_with_its_index_at_its_own_carrier() {
    // Empty result domain: the vacuous closed argument, so the obligation is
    // discharged without the `2^32` extent the inhabited argument needs.
    let program = gather_program_over([4, 0], [2], 0);
    let mut request = CompilationRequest::governed(&program);
    request.target_profiles = vec![TargetProfile::governed_with_gather_index_dispatch_for_test()];
    let planned = verify_planned_request(request).expect("the fixture admits a planned request");
    let target = planned
        .for_target(0)
        .expect("the U32-capable profile admits the fixture");
    let lowering = crate::lowering::resolve_lowering(&program, &target)
        .expect("the governed gather capability lowers a recognized gather");
    let [output] = target.normalized().outputs() else {
        panic!("the fixture declares one output");
    };
    // The premise: this occurrence really is statically proved, so what the
    // compile below exercises is the program's carrier rather than the bounds
    // obligation, which would stop one whole layer earlier.
    let spelling = crate::physical::spell_region(
        &target,
        &output.members(),
        crate::physical::RegionWrite::ProgramOutput,
        &lowering,
    )
    .expect("a vacuously proved gather is spelled by the governed vocabulary");
    assert_eq!(spelling.kind(), crate::physical::RegionSpellingKind::Gather);

    let mut request = CompilationRequest::governed(&program);
    request.target_profiles = vec![TargetProfile::governed_with_gather_index_dispatch_for_test()];
    let compiled = crate::pipeline::compile(request).unwrap_or_else(|error| {
        // Projected to the innermost typed payload, for the reason
        // `a_governed_gather_refuses_at_dispatch_then_at_the_region_vocabulary`
        // projects its own: a `CompileError` reaching this layer carries a whole
        // explain trace, so printing it whole would bury the one difference this
        // test is about under fifteen megabytes of canonical bytes — and its
        // `Display` peels to a bare rule name that names no operand.
        match compiler_output_refusal(&error) {
            Some(output) => panic!(
                "the governed gather compiles once its index carries its own type, got {output:?}"
            ),
            None => panic!(
                "the governed gather compiles once its index carries its own type, got {error}"
            ),
        }
    });
    let values: Vec<_> = packaged_program(&compiled).core().values().collect();
    let boundary = |name: &str| {
        values
            .iter()
            .find(|value| {
                matches!(
                    value.origin(),
                    tiler_ir::program::MaterializedOrigin::ProgramInput { key } if key.as_str() == name,
                )
            })
            .unwrap_or_else(|| panic!("the packaged program declares the `{name}` input"))
    };

    let source = boundary("source");
    assert_eq!(
        source.storage_scalar(),
        tiler_ir::program::StorageScalar::F32
    );
    assert_eq!(source.element_type(), tiler_ir::kernel::KernelType::F32);

    let index = boundary("index");
    assert_eq!(
        index.storage_scalar(),
        tiler_ir::program::StorageScalar::U32,
        "the address operand must be materialized at the carrier its own resolved type names",
    );
    assert_eq!(
        index.element_type(),
        tiler_ir::kernel::KernelType::U32,
        "the address operand must be read at its exact width",
    );
    // Two elements at the `U32` carrier's four bytes. The byte count derives
    // from the same answer the element type does, which is what keeps a buffer
    // from being sized by one width and read at another.
    assert_eq!(index.required_bytes(), 8);
}

/// The innermost invalid-compiler-output payload one refusal carries.
///
/// Recursive over `Explained` for the same reason `planning_capability_rule` is:
/// the explain wrapper is added at the target boundary and says nothing about
/// which authority refused.
fn compiler_output_refusal(
    error: &crate::pipeline::CompileError,
) -> Option<&crate::pipeline::CompilerOutputError> {
    match error {
        crate::pipeline::CompileError::InvalidCompilerOutput(output) => Some(output),
        crate::pipeline::CompileError::Explained { source, .. } => compiler_output_refusal(source),
        _ => None,
    }
}

use super::super::{
    Axis, BoundaryRead, ContractionIndex, ContractionIndexStructure, DeclaredInputOrdinal, F32,
    InputKey, LogicalAccess, NormalizedOutput, NormalizedStaged, OutputKey, SemanticProgram,
    SemanticStage, SerialSumContributor, Shape, UNREAD_DECLARED_INPUT_TAG,
    encode_elementwise_reads, encode_explain_shape, encode_output_subject, output_subject,
    push_len, push_slice, select_supported_strategy,
};
use super::support::{
    contraction_fed_normalization, contraction_program, laws_of, normalization_program, recognize,
    recognize_outputs,
};
use tiler_ir::semantic::{
    F32Add, F32Constant, F32Multiply, SemanticProgramBuilder, StrictSerialF32Sum,
};

/// The staged subject separates an edge-fed operand from a declared one, and
/// separates a carried producer from an absent one.
///
/// **Two claims, each isolated, because either alone would pass on the
/// other's evidence.** The occurrence's own operand run and the producer are
/// two facts the `staged-family.v2` arm writes, and a forgery that moved both
/// at once would be separated by whichever the encoder still carried — the
/// exact way a check stops exercising its shape while staying green.
///
/// Each forgery therefore moves exactly one field of the *same* recognized
/// value, leaving every operand shape, element count, key, member ordinal and
/// published shape identical. Neither forgery is a value the recognizer
/// produces; that is what makes them drivable at all, and it is the same
/// device the request-subject mutation tests above use.
///
/// Watched failing under two deliberate perturbations, one per claim:
/// dropping the role tag from `encode_output_subject`'s staged arm makes the
/// first pair equal, and dropping its producer run makes the second pair
/// equal.
#[test]
fn a_staged_subject_separates_an_edge_fed_operand_from_a_declared_one() {
    let program = contraction_fed_normalization(false, false);
    let normalized = select_supported_strategy(&program, &laws_of(&program)).unwrap();
    let [recognized] = normalized.outputs() else {
        panic!("the fixture declares one output");
    };
    let encoded = |output: &NormalizedOutput| {
        let mut bytes = Vec::new();
        encode_output_subject(&mut bytes, &output_subject(output));
        bytes
    };
    let forge = |edit: fn(&mut NormalizedStaged)| {
        let mut forged = recognized.clone();
        let NormalizedOutput::Staged(staged) = &mut forged else {
            panic!("a normalization output recognizes as a staged family")
        };
        edit(staged);
        encoded(&forged)
    };
    assert_ne!(
        encoded(recognized),
        forge(|staged| {
            staged.operand_reads[0] = BoundaryRead::Input(DeclaredInputOrdinal::new(0));
        }),
        "the operand's boundary role is part of what the occurrence reads",
    );
    assert_ne!(
        encoded(recognized),
        forge(|staged| staged.producer = None),
        "the shape writing the edge is part of what this partition computes",
    );
}

/// Two occurrences differing only in `eps` bind different request subjects.
///
/// The attribute record is what separates them: both programs declare the
/// same keys, the same shapes, the same operand map, the same member, and
/// the same element counts, so a staged subject arm that omitted the record
/// would give two different normalizations one identity. Watched failing
/// under a deliberate perturbation: dropping the attribute run from
/// `encode_output_subject`'s staged arm makes the two subjects equal.
#[test]
fn a_staged_subject_separates_two_occurrences_differing_only_in_eps() {
    let subject_bytes = |eps_bits: u32| {
        let program = normalization_program(false, eps_bits);
        let normalized = select_supported_strategy(&program, &laws_of(&program)).unwrap();
        let mut bytes = Vec::new();
        for output in normalized.outputs() {
            encode_output_subject(&mut bytes, &output_subject(output));
        }
        bytes
    };
    let first = subject_bytes(1.0e-6_f32.to_bits());
    let second = subject_bytes(1.0e-5_f32.to_bits());
    assert_ne!(
        first, second,
        "the occurrence's eps payload is part of what the staged stage computes"
    );
}

/// The read run separates two subsets and leaves a complete one empty.
///
/// **Both halves of the sub-tag determination, driven at the encoder.** The
/// complete read list writes the framed zero it has always written, which is
/// the "no already-encodable subject's bytes move" half; the three
/// two-element subsets of three declared inputs write three different runs,
/// which is the injectivity half the marker exists for. Without the marker
/// all three would be that same framed zero, and one arm would encode three
/// programs.
#[test]
fn the_read_run_marks_unread_declared_inputs_and_leaves_a_complete_list_empty() {
    let dense = |ordinal| {
        (
            DeclaredInputOrdinal::new(ordinal),
            LogicalAccess::LinearIdentity,
        )
    };
    let run = |reads: &[(DeclaredInputOrdinal, LogicalAccess)]| {
        let mut bytes = Vec::new();
        encode_elementwise_reads(&mut bytes, 3, reads);
        bytes
    };
    // The framed zero every already-encodable subject wrote, byte for byte.
    assert_eq!(run(&[dense(0), dense(1), dense(2)]), vec![0_u8; 8]);
    // One marker, naming the ordinal no leaf read.
    let mut expected = vec![0_u8; 7];
    expected.push(1);
    expected.extend_from_slice(&1_u32.to_be_bytes());
    expected.push(UNREAD_DECLARED_INPUT_TAG);
    assert_eq!(run(&[dense(0), dense(2)]), expected);
    // The three subsets of the same size are three distinct runs, which is
    // the collision the marker closes.
    let subsets = [
        run(&[dense(0), dense(1)]),
        run(&[dense(0), dense(2)]),
        run(&[dense(1), dense(2)]),
    ];
    for (position, first) in subsets.iter().enumerate() {
        for second in &subsets[position + 1..] {
            assert_ne!(first, second);
        }
    }
}

/// A contraction over one of the three two-input subsets of one declaration.
///
/// The independent output retains the skipped input without entering the
/// contraction walk. All input shapes and occurrence positions are equal
/// across fixtures, so the read ordinals are the only contraction-subject
/// field that changes.
fn contraction_subset_program(pair: [usize; 2]) -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let shape = Shape::from_dims([2, 2]);
    let inputs: Vec<_> = ["a", "b", "c"]
        .into_iter()
        .map(|key| {
            builder
                .input::<F32>(InputKey::new(key).unwrap(), shape.clone())
                .unwrap()
        })
        .collect();
    let structure = ContractionIndexStructure::new(
        [
            vec![ContractionIndex::new(0), ContractionIndex::new(1)],
            vec![ContractionIndex::new(1), ContractionIndex::new(2)],
        ],
        [ContractionIndex::new(0), ContractionIndex::new(2)],
    )
    .expect("ab,bc->ac is an admitted structure");
    let product = tiler_ir::semantic::F32TensorContraction::apply(
        &mut builder,
        &structure,
        inputs[pair[0]],
        inputs[pair[1]],
    )
    .unwrap();
    let skipped = (0..3)
        .find(|ordinal| !pair.contains(ordinal))
        .expect("two of three inputs leave one skipped");
    let retained = F32Add::apply(&mut builder, inputs[skipped], inputs[skipped]).unwrap();
    builder
        .output(OutputKey::new("product").unwrap(), product)
        .unwrap();
    builder
        .output(OutputKey::new("retained").unwrap(), retained)
        .unwrap();
    builder.build().unwrap()
}

/// The three subsets are distinguished by the contraction arm itself.
///
/// This drives [`encode_output_subject`] directly, excluding the enclosing
/// semantic graph identity that would distinguish separately built programs
/// whatever this arm encoded. It also pins both read predicates for the
/// skipped ordinal, so restoring dense indexing or a declaration-length
/// predicate makes the first non-prefix subset fail independently.
#[test]
fn contraction_subjects_separate_all_two_input_subsets_of_three_declarations() {
    let pairs = [[0_u32, 1_u32], [0, 2], [1, 2]];
    let mut subjects = Vec::new();
    for pair in pairs {
        let program = contraction_subset_program([
            usize::try_from(pair[0]).unwrap(),
            usize::try_from(pair[1]).unwrap(),
        ]);
        let recognized = recognize_outputs(&program).expect("both outputs are recognized");
        let NormalizedOutput::Contraction(contraction) = &recognized.outputs()[0] else {
            panic!("the first output is the contraction");
        };
        assert_eq!(contraction.input_keys.len(), 3);
        assert_eq!(
            contraction
                .reads
                .iter()
                .map(|read| read.input_ordinal)
                .collect::<Vec<_>>(),
            pair,
        );
        let skipped = (0..3).find(|ordinal| !pair.contains(ordinal)).unwrap();
        for ordinal in pair {
            assert!(
                recognized.outputs()[0].reads_declared_input(DeclaredInputOrdinal::new(ordinal))
            );
            assert_eq!(
                recognized.outputs()[0].input_elements_at(DeclaredInputOrdinal::new(ordinal)),
                Some(4),
            );
        }
        assert!(!recognized.outputs()[0].reads_declared_input(DeclaredInputOrdinal::new(skipped)));
        assert_eq!(
            recognized.outputs()[0].input_elements_at(DeclaredInputOrdinal::new(skipped)),
            None,
        );

        let mut bytes = Vec::new();
        encode_output_subject(&mut bytes, &output_subject(&recognized.outputs()[0]));
        subjects.push(bytes);
    }
    for (position, first) in subjects.iter().enumerate() {
        for second in &subjects[position + 1..] {
            assert!(first != second, "two declared-input subsets collided");
        }
    }
}

/// The conditional ordinal run does not move an old contraction subject.
///
/// The helper is the exact pre-widening `contraction-f32.v1` arm, projected
/// through the new read records. Equality therefore checks every byte of an
/// already-admitted two-declaration subject, not merely its tag or digest.
#[test]
fn a_two_declaration_contraction_keeps_its_v1_subject_bytes() {
    let program = contraction_program(false);
    let recognized = recognize(&program).expect("the contraction is recognized");
    let NormalizedOutput::Contraction(normalized) = &recognized else {
        panic!("the output is a contraction");
    };
    assert_eq!(
        normalized
            .reads
            .iter()
            .map(|read| read.input_ordinal)
            .collect::<Vec<_>>(),
        [0, 1],
    );

    let mut legacy = Vec::new();
    push_slice(&mut legacy, b"contraction-f32.v1");
    push_len(&mut legacy, normalized.input_keys.len());
    for key in &normalized.input_keys {
        push_slice(&mut legacy, key.as_str().as_bytes());
    }
    push_slice(&mut legacy, normalized.output_key.as_str().as_bytes());
    for read in &normalized.reads {
        encode_explain_shape(&mut legacy, &read.shape);
    }
    encode_explain_shape(&mut legacy, &normalized.output_shape);
    encode_explain_shape(&mut legacy, &normalized.contracted_shape);
    push_slice(
        &mut legacy,
        normalized.structure.canonical_encoding().as_bytes(),
    );
    for read in &normalized.reads {
        push_len(&mut legacy, read.operand_position);
    }
    push_len(&mut legacy, normalized.members.len());
    for atom in &normalized.members {
        legacy.extend_from_slice(&atom.member().0.to_be_bytes());
    }
    for read in &normalized.reads {
        legacy.extend_from_slice(&read.elements.to_be_bytes());
    }
    legacy.extend_from_slice(&normalized.output_elements.to_be_bytes());
    legacy.extend_from_slice(&normalized.contracted_elements.to_be_bytes());

    let mut current = Vec::new();
    encode_output_subject(&mut current, &output_subject(&recognized));
    assert_eq!(current, legacy, "an existing v1 subject moved bytes");
}

/// `sum(input * 2.0, [cols])` — the pointwise-prologue neighbour of every
/// produced fold below, and the shape the `tiler-build` Metal goldens qualify.
fn pointwise_prologue_fold() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 4]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let scaled = F32Multiply::apply(&mut builder, input, scale).unwrap();
    let folded = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), folded)
        .unwrap();
    builder.build().unwrap()
}

/// `sum(input, [cols])` — the declared-input neighbour.
fn declared_input_fold() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 4]))
        .unwrap();
    let folded = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), folded)
        .unwrap();
    builder.build().unwrap()
}

/// `sum(sum(input, [cols]) * 2.0, [rows])` — a produced fold with a
/// continuation, over the same declaration as the two neighbours above.
fn produced_fold_with_continuation() -> SemanticProgram {
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let input = builder
        .input::<F32>(InputKey::new("input").unwrap(), Shape::from_dims([2, 4]))
        .unwrap();
    let scale = F32Constant::apply(&mut builder, 2.0_f32.to_bits()).unwrap();
    let inner = StrictSerialF32Sum::apply(&mut builder, input, [Axis::new(1)]).unwrap();
    let scaled = F32Multiply::apply(&mut builder, inner, scale).unwrap();
    let outer = StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(0)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), outer)
        .unwrap();
    builder.build().unwrap()
}

/// Renders one encoded subject as lowercase hex, for a pinned comparison.
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    bytes.iter().fold(String::new(), |mut rendered, byte| {
        let _ = write!(rendered, "{byte:02x}");
        rendered
    })
}

/// Encodes one program's sole recognized output subject.
fn encoded_subject(program: &SemanticProgram) -> Vec<u8> {
    let recognized = recognize(program).expect("the fixture is recognized");
    let mut bytes = Vec::new();
    encode_output_subject(&mut bytes, &output_subject(&recognized));
    bytes
}

/// The two neighbour arms' `serial-sum-f32.v3` bytes did not move.
///
/// **Pinned to exact bytes rather than compared structurally**, because what the
/// accepted carrier promised is that these two subjects encode to *what they
/// already did* — so the enclosing `tiler.compiler.request-subject.v6` domain
/// does not step, the `domains.rs` pin row stays where it is, and every governed
/// compilation keeps its request qualifier. A structural assertion would pass
/// through exactly the change that breaks that promise.
///
/// The two values below were captured at base `441f3215` — before the
/// contributor source existed — by running this test in a detached worktree at
/// that commit. They are recorded rather than derived for the reason
/// `tiler-build`'s standard Metal pins are: the point is that they do **not**
/// move. A change here is either a deliberate identity revision, which must
/// step the sub-tag and restate every pin in the commit that states why, or the
/// defect this test exists to catch.
#[test]
fn the_declared_input_and_pointwise_prologue_arms_keep_their_exact_bytes() {
    const DECLARED_INPUT: &str = "000000000000001173657269616c2d73756d2d6633322e763300000000000000010000000000000005696e7075740000000000000006726573756c740000000000000002000000000000000200000000000000040000000000000001000000000000000200000000000000010000000100000000000000000000000000000000000000000000000100000000000000000000000800000000000000020000000000000000";
    const POINTWISE_PROLOGUE: &str = "000000000000001173657269616c2d73756d2d6633322e763300000000000000010000000000000005696e7075740000000000000006726573756c74000000000000000200000000000000020000000000000004000000000000000100000000000000020000000000000001000000010000000000000003010000000002400000000400000000000000010000000200000000000000020000000000000001000000000000000100000002000000000000000800000000000000020000000000000000";

    assert_eq!(
        hex(&encoded_subject(&declared_input_fold())),
        DECLARED_INPUT
    );
    assert_eq!(
        hex(&encoded_subject(&pointwise_prologue_fold())),
        POINTWISE_PROLOGUE,
    );
}

/// A produced fold takes its own framed sub-tag, and the *tag* is what separates
/// it — not the unread-declared-input marker run.
///
/// **The forgery is built as bytes rather than inferred.** The subject that a
/// forger would have to write to claim `serial-sum-f32.v3` for a produced fold is
/// constructed here by perturbing only the recognized contributor source and
/// re-encoding, so both byte strings are real encoder output over the same fold.
/// The assertion is then about where they diverge: the framed tag, at position
/// zero, before any payload is read.
///
/// **That distinction is the whole point.** A dropped-producer forgery pushed
/// through the old grammar emits an `encode_elementwise_reads` run of *only*
/// unread markers, which no legal old subject produces — so the two strings would
/// differ even with no tag split, and a test resting on that would pass while the
/// separation rested on an accident of one program's declaration count. Resting
/// identity on an unstated invariant is what `encode_elementwise_reads`'s own
/// comment forbids, so what is asserted is the structural control.
#[test]
fn a_produced_fold_cannot_encode_under_the_old_serial_sum_tag() {
    let framed = |tag: &[u8]| {
        let mut bytes = Vec::new();
        push_slice(&mut bytes, tag);
        bytes
    };
    let produced_tag = framed(b"serial-sum-produced-f32.v1");
    let retained_tag = framed(b"serial-sum-f32.v3");

    let recognized = recognize(&produced_fold_with_continuation()).expect("the produced fold");
    let mut produced = Vec::new();
    encode_output_subject(&mut produced, &output_subject(&recognized));
    assert!(
        produced.starts_with(&produced_tag),
        "the materialized arm must open with its own framed tag",
    );

    // The forgery: the same fold, claiming the neighbour's grammar. Only the
    // contributor source moves, so every other fact the arm writes is the
    // produced fold's own.
    let mut forged = recognized.clone();
    forged.serial_sum_mut().contributor =
        SerialSumContributor::DeclaredInput(DeclaredInputOrdinal::new(0));
    let mut forged_bytes = Vec::new();
    encode_output_subject(&mut forged_bytes, &output_subject(&forged));
    assert!(
        forged_bytes.starts_with(&retained_tag),
        "the forgery must be real encoder output under the tag it claims",
    );
    assert_ne!(produced, forged_bytes);
    // The separation is *inside the framed tag*, so it holds for every produced
    // fold rather than only for the ones whose declaration count happens to emit
    // a marker run. The two tag strings differ in length, so the divergence
    // lands in the eight-byte length prefix — the first field either arm writes,
    // and one no payload can reach past.
    let shared = produced
        .iter()
        .zip(&forged_bytes)
        .take_while(|(left, right)| left == right)
        .count();
    assert!(
        shared < retained_tag.len(),
        "the two arms must diverge inside the framed tag, before any payload; \
         they share {shared} of the retained tag's {} bytes",
        retained_tag.len(),
    );

    // And the producer itself is bound: two produced folds differing only in
    // what writes their contributors are different subjects. Un-repaired, an
    // encoder that dropped the producer would collide them.
    let mut other_producer = recognized.clone();
    let SerialSumContributor::Materialized(materialized) =
        &mut other_producer.serial_sum_mut().contributor
    else {
        panic!("the fixture folds a materialized contributor");
    };
    let NormalizedOutput::SerialSum(inner_fold) = &mut materialized.producer else {
        panic!("the fixture's producer is a fold");
    };
    inner_fold.reduction_axes = vec![Axis::new(0)];
    let mut other_bytes = Vec::new();
    encode_output_subject(&mut other_bytes, &output_subject(&other_producer));
    assert_ne!(
        produced, other_bytes,
        "the producer is written through the recursion, so a different producer is a different subject",
    );
}

/// The continuation's presence is written, so omitting it is a different
/// subject rather than the same one.
///
/// A produced fold whose contributor *is* the produced value carries no
/// continuation; one with an expression between the two carries a presence byte
/// and the epilogue read vocabulary. The two must not share a byte string, or a
/// forgery could drop the continuation and keep the fold's identity.
#[test]
fn a_produced_folds_continuation_presence_is_bound() {
    let with_continuation =
        recognize(&produced_fold_with_continuation()).expect("the produced fold");
    let mut present = Vec::new();
    encode_output_subject(&mut present, &output_subject(&with_continuation));

    let mut without = with_continuation.clone();
    let SerialSumContributor::Materialized(materialized) =
        &mut without.serial_sum_mut().contributor
    else {
        panic!("the fixture folds a materialized contributor");
    };
    materialized.continuation = None;
    let mut absent = Vec::new();
    encode_output_subject(&mut absent, &output_subject(&without));

    assert_ne!(present, absent);
    assert!(
        absent.len() < present.len(),
        "the absent continuation writes its presence byte and nothing else",
    );
    // The presence byte is the last byte of the shorter encoding, and it is
    // `0x00`; the longer one carries `0x01` at that position and the framed
    // expression after it. A forgery that truncated the payload but kept the
    // byte would still be a different subject.
    assert_eq!(absent.last().copied(), Some(0x00));
    assert_eq!(present.get(absent.len() - 1).copied(), Some(0x01));
}

/// A produced fold's `sum(rms_norm(x, w))` shape retains a staged producer and
/// no synthesized continuation, and a produced sum's fold stays resolvable.
#[test]
fn a_produced_folds_partition_claims_the_producer_and_the_continuation() {
    let recognized = recognize(&produced_fold_with_continuation()).expect("the produced fold");
    let NormalizedOutput::SerialSum(fold) = &recognized else {
        panic!("a produced fold recognizes as a serial sum");
    };
    let SerialSumContributor::Materialized(materialized) = &fold.contributor else {
        panic!("the fixture folds a materialized contributor");
    };

    // Every occurrence is claimed exactly once, which is what
    // `check_output_cover` requires: the inner fold, the constant, the multiply,
    // and the outer fold.
    assert_eq!(
        recognized.members().len(),
        produced_fold_with_continuation().operation_count(),
    );

    // The three parts are disjoint and none of them is the pointwise prologue's.
    assert_eq!(fold.prologue_members(), None);
    let continuation = fold
        .continuation_members()
        .expect("the `* 2` is a continuation region");
    assert!(
        !fold
            .members
            .pointwise()
            .iter()
            .any(|atom| continuation.contains(atom)),
        "a continuation member must never enter the declared-input prologue part",
    );
    assert!(
        !fold
            .members
            .all()
            .iter()
            .any(|atom| continuation.contains(atom)),
        "the fused affine candidate is prologue-union-fold, so it must not claim the continuation",
    );
    for part in [
        fold.members.reduction(),
        continuation,
        &materialized.producer.members(),
    ] {
        assert!(
            recognized.owns_region_members(part),
            "each part of the fold's partition must resolve to this output",
        );
    }
    // The continuation union the fold is *not* a part: no scheduled region
    // computes an expression over a staged value and a fold of its result.
    let mut grouped: Vec<SemanticStage> = continuation.to_vec();
    grouped.extend_from_slice(fold.members.reduction());
    grouped.sort_unstable();
    grouped.dedup();
    assert!(
        !recognized.owns_region_members(&grouped),
        "grouping the continuation with the fold is declined, not flattened",
    );
}

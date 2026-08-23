use super::super::{
    Axis, BTreeSet, DeclaredInputOrdinal, Extent, F32, InputKey, NormalizedOutput, OutputKey, Shape,
};
use super::support::{contraction_program, normalization_program, recognize};
use tiler_ir::semantic::{F32Add, F32Multiply, SemanticProgramBuilder, StrictSerialF32Sum};

/// One fixture of [`every_arm_answers_the_declared_tensors_own_count`] and
/// everything asserted about it.
///
/// Named rather than a tuple so each column reads as the claim it is: the
/// rows carry six columns each, and in a positional literal an exchanged
/// pair of `u64`s looks like a passing row.
struct CountRow {
    label: &'static str,
    /// The arm the fixture must reach, so a row whose recognition moved
    /// stops standing for the arm it names.
    arm: &'static str,
    output: NormalizedOutput,
    /// The iteration domain the widening read is *not* answered at, or
    /// `None` where the row has no widening read — for the two arms that
    /// hold no elementwise read list, and for the bare fold whose one read
    /// is dense.
    domain: Option<u64>,
    /// The count each declared ordinal must resolve to, in declaration
    /// order. Its length is the declared arity.
    counts: &'static [Option<u64>],
    max: u64,
}

/// Every arm of [`NormalizedOutput::input_elements_at`] answers the declared
/// tensor's own element count, and none answers a reading region's domain.
///
/// **The two numbers coincide unless a read widens, so most rows carry a
/// widening one.** A `[2]` weight broadcast into a `[2, 2]` region iterates
/// four points and holds two elements; an arm answering `4` would scale an
/// opaque call by the iteration space rather than by the buffer whose exact
/// access projects to that declared ordinal, which is the confidently
/// wrong work count [`crate::call_declaration::WorkScaling`] exists to
/// prevent. Each row therefore states the domain beside the counts and
/// refuses to run if they are equal, so a row that had no widening to get
/// wrong cannot pass for one that did.
///
/// **The rows are counted against the arms.** "Every arm" is the claim, so
/// the population is asserted to reach all five rather than described as
/// doing so; a variant added without a row fails here rather than shipping
/// unexamined. [`NormalizedOutput::reads_declared_input`] is asserted beside
/// every count because the two are separate statements of which ordinals a
/// walk reached, and
/// [`NormalizedProgram::agreed_input_elements_at`] refuses when they drift.
///
/// **Watched failing once each, every perturbation on the subject rather
/// than on an assertion, and each quoted by the row that caught it:**
///
/// - Restoring [`NormalizedOutput::input_elements_at`]'s pointwise arm to
///   `normalized.elements`, the reading region's domain it answered before:
///   *a sole widened pointwise read: ordinal 0 is not the declared tensor's
///   own count — left `Some(4)`, right `Some(2)`*.
/// - Restoring [`NormalizedOutput::max_input_elements`]'s pointwise arm to
///   the same domain, perturbed alone so the count rows still pass: *a sole
///   widened pointwise read: the largest declared input count this output
///   reads — left `4`, right `2`*. The two arms are perturbed separately
///   because together the first fires and hides the second.
/// - Restoring the serial-sum arm to `normalized.input_elements`: *a widened
///   read in a fold's prologue: ordinal 0 — left `Some(4)`, right
///   `Some(2)`*.
/// - Restoring the epilogue arm's consumed half to `chain.elements`: *a
///   widened read in a chain's epilogue: ordinal 1 — left `Some(4)`, right
///   `Some(2)`*.
/// - Dropping the serial-sum arm's `contributor_input` term, which is the
///   one read no read list describes: *a prologue-less fold's own
///   contributor read: ordinal 0 — left `None`, right `Some(6)`*.
/// - Answering [`read_tensor_elements`]'s structural arms with
///   `domain_elements` instead of the operand shape, which is the single
///   statement the three widening rows share: the first of them fires, *a
///   sole widened pointwise read: ordinal 0 — left `Some(4)`, right
///   `Some(2)`*, and each later row fires in turn once its predecessor is
///   admitted.
#[test]
fn every_arm_answers_the_declared_tensors_own_count() {
    // A `[2]` operand replicated across a leading axis into `[2, 2]`: the
    // read addresses two elements over a domain of four, which is the whole
    // difference these rows are about.
    let widen = |builder: &mut SemanticProgramBuilder, operand: tiler_ir::semantic::Value<F32>| {
        let mapping = tiler_ir::semantic::BroadcastAxisMapping::new(
            [Extent::new(2), Extent::new(2)],
            [
                tiler_ir::semantic::BroadcastAxisSource::Replicate,
                tiler_ir::semantic::BroadcastAxisSource::FromOperand(Axis::new(0)),
            ],
        )
        .expect("one replicated axis over a rank-one operand is an admitted relation");
        tiler_ir::semantic::F32Broadcast::apply(builder, &mapping, operand)
            .expect("the standard registry admits the broadcast family")
    };
    let weight = |builder: &mut SemanticProgramBuilder| {
        builder
            .input::<F32>(InputKey::new("w").unwrap(), Shape::from_dims([2]))
            .unwrap()
    };

    // `w + w` over the widened read alone: one declared input, read only
    // through the relation, so this is the row where the maximum moves too.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let w = weight(&mut builder);
    let widened = widen(&mut builder, w);
    let root = F32Add::apply(&mut builder, widened, widened).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    let sole_widened_read = builder.build().unwrap();

    // `a * broadcast(w)`: the widened read beside a dense one, so the two
    // ordinals must answer different counts from one region.
    let mixed_program = |folded: bool| {
        let mut builder = SemanticProgramBuilder::try_standard().unwrap();
        let w = weight(&mut builder);
        let a = builder
            .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2]))
            .unwrap();
        let widened = widen(&mut builder, w);
        let scaled = F32Multiply::apply(&mut builder, a, widened).unwrap();
        let root = if folded {
            StrictSerialF32Sum::apply(&mut builder, scaled, [Axis::new(1)]).unwrap()
        } else {
            scaled
        };
        builder
            .output(OutputKey::new("result").unwrap(), root)
            .unwrap();
        builder.build().unwrap()
    };

    // `sum(a, axis 1)`: no prologue, so the fold's own contributor read is
    // the one access no read list describes. Nothing widens here, and the
    // row is what keeps that term live.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 3]))
        .unwrap();
    let root = StrictSerialF32Sum::apply(&mut builder, a, [Axis::new(1)]).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    let bare_fold = builder.build().unwrap();

    // `sum(a, axis 2) * broadcast(w)`: the producer folds ordinal `0` at its
    // own twelve-element shape and the epilogue widens ordinal `1` over a
    // four-point domain, so one chain carries both halves.
    let mut builder = SemanticProgramBuilder::try_standard().unwrap();
    let a = builder
        .input::<F32>(InputKey::new("a").unwrap(), Shape::from_dims([2, 2, 3]))
        .unwrap();
    let w = weight(&mut builder);
    let reduced = StrictSerialF32Sum::apply(&mut builder, a, [Axis::new(2)]).unwrap();
    let widened = widen(&mut builder, w);
    let root = F32Multiply::apply(&mut builder, reduced, widened).unwrap();
    builder
        .output(OutputKey::new("result").unwrap(), root)
        .unwrap();
    let widened_epilogue = builder.build().unwrap();

    let arm = |output: &NormalizedOutput| match output {
        NormalizedOutput::SerialSum(_) => "serial-sum",
        NormalizedOutput::Pointwise(_) => "pointwise",
        NormalizedOutput::Contraction(_) => "contraction",
        NormalizedOutput::Epilogue(_) => "epilogue",
        NormalizedOutput::Staged(_) => "staged",
        NormalizedOutput::Gather(_) => "gather",
    };
    let rows: [CountRow; 7] = [
        CountRow {
            label: "a sole widened pointwise read",
            arm: "pointwise",
            output: recognize(&sole_widened_read).expect("a widened read is an elementwise region"),
            domain: Some(4),
            counts: &[Some(2)],
            max: 2,
        },
        CountRow {
            label: "a widened pointwise read beside a dense one",
            arm: "pointwise",
            output: recognize(&mixed_program(false))
                .expect("a widened read is an elementwise region"),
            domain: Some(4),
            counts: &[Some(2), Some(4)],
            max: 4,
        },
        CountRow {
            label: "a widened read in a fold's prologue",
            arm: "serial-sum",
            output: recognize(&mixed_program(true)).expect("a widened prologue read is recognized"),
            domain: Some(4),
            counts: &[Some(2), Some(4)],
            max: 4,
        },
        CountRow {
            label: "a prologue-less fold's own contributor read",
            arm: "serial-sum",
            output: recognize(&bare_fold).expect("a fold over a declared input is recognized"),
            domain: None,
            counts: &[Some(6)],
            max: 6,
        },
        CountRow {
            label: "a widened read in a chain's epilogue",
            arm: "epilogue",
            output: recognize(&widened_epilogue).expect("a widened epilogue read is recognized"),
            domain: Some(4),
            counts: &[Some(12), Some(2)],
            max: 12,
        },
        CountRow {
            label: "a contraction's two operands",
            arm: "contraction",
            output: recognize(&contraction_program(false))
                .expect("a binary contraction is recognized"),
            domain: None,
            counts: &[Some(6), Some(12)],
            max: 12,
        },
        CountRow {
            label: "a staged family's operand run",
            arm: "staged",
            output: recognize(&normalization_program(false, 1.0e-6_f32.to_bits()))
                .expect("a normalization is recognized"),
            domain: None,
            counts: &[Some(4), Some(4)],
            max: 4,
        },
    ];
    let reached: BTreeSet<&str> = rows.iter().map(|row| row.arm).collect();
    assert_eq!(
        reached.len(),
        5,
        "the rows reach {reached:?}, which is not every arm of the accessor",
    );

    for CountRow {
        label,
        arm: expected_arm,
        output,
        domain,
        counts,
        max,
    } in rows
    {
        assert_eq!(
            arm(&output),
            expected_arm,
            "{label}: the fixture recognized as another arm, so the row proves nothing about \
             the one it names",
        );
        if let Some(domain) = domain {
            assert!(
                counts.iter().any(|count| *count != Some(domain)),
                "{label}: every count equals the domain of {domain}, so this row cannot \
                 observe the difference it exists for",
            );
        }
        for (ordinal, expected) in counts.iter().enumerate() {
            let ordinal = u32::try_from(ordinal).expect("the fixtures declare few inputs");
            assert_eq!(
                output.input_elements_at(DeclaredInputOrdinal::new(ordinal)),
                *expected,
                "{label}: ordinal {ordinal} is not the declared tensor's own count",
            );
            assert_eq!(
                output.reads_declared_input(DeclaredInputOrdinal::new(ordinal)),
                expected.is_some(),
                "{label}: ordinal {ordinal} — the predicate and the count disagree about what \
                 this walk reads",
            );
        }
        let past = u32::try_from(counts.len()).expect("the fixtures declare few inputs");
        assert_eq!(
            output.input_elements_at(DeclaredInputOrdinal::new(past)),
            None,
            "{label}: an ordinal past the declaration produced a count",
        );
        assert_eq!(
            output.max_input_elements(),
            max,
            "{label}: the largest declared input count this output reads",
        );
    }
}

/// The recognized-output vocabulary is sized from its own enum.
///
/// A hand-written six would be satisfied by an enumeration that had stopped
/// covering the type. `variant_count` makes a widened vocabulary a build error
/// at this line instead, which is the property every consumer's exhaustive match
/// already has and which this states for the population as a whole.
#[test]
fn the_recognized_output_vocabulary_is_sized_from_its_type() {
    assert_eq!(
        std::mem::variant_count::<NormalizedOutput>(),
        6,
        "the recognized output vocabulary changed size; every dependent claim \
about it needs re-reading",
    );
}

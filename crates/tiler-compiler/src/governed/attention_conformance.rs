//! Bit-level conformance of the two attention contractions' `direct` realization.
//!
//! # What is compared, and against what
//!
//! Two independent computations of each attention contraction:
//!
//! 1. the **emitted index region** — the region `refine_index_region` actually
//!    returned for the occurrence, executed by `tiler-reference`'s independent
//!    index-region oracle; and
//! 2. the **registered reference evaluator** for
//!    `tiler::tensor-contraction-f32@1`, which decodes the family's numerical
//!    signature rather than restating it.
//!
//! # Why there is no third comparison here, unlike the projection structure
//!
//! [`super::contraction_conformance`] compares a third quantity: the retained
//! `result_sha256` of the L3 realization probe's `direct` kernel. **No such
//! record exists for either attention structure.** The L3 probe swept the
//! projection structure alone, and
//! [`realize-the-attention-contractions-on-metal`](../../../../tickets/realize-the-attention-contractions-on-metal.md)
//! states that no cell of either structure has been timed or digested at any
//! shape. Inventing a digest here, or reusing the projection structure's, would
//! state as measured something never measured. The comparison is therefore
//! between the emitted region and the reference, and this module says so rather
//! than implying a device baseline it does not have.
//!
//! **Measurement boundary: a host comparison is not a dispatched one.** What
//! these tests establish is that the region the governed lowering emitted
//! computes the same bits as the registered reference at the stated extents.
//! Whether a Metal kernel built from that region computes them on a device is
//! [`integrate-the-contraction-vertical-into-the-runtime`](../../../../tickets/integrate-the-contraction-vertical-into-the-runtime.md)'s
//! subject, and the projection structure is the only one it has carried there.
//!
//! # Which extents are reached, and why not the prefill B1 rows
//!
//! `MAX_EVALUATION_STEPS` caps one region evaluation at 16,777,216 steps. A
//! *fold* step is the natural unit for the contraction — output population times
//! contracted extent — and the rows are:
//!
//! | Row | fold steps, either structure | reached |
//! | --- | --- | --- |
//! | C1 prefill, `T = S = 10` | 204,800 | yes |
//! | B1-a decode, `T = 1`, `S = 256` | 524,288 | yes |
//! | B1-a prefill, `T = S = 128` | 33,554,432 | **no** |
//!
//! **The oracle's budget is not counted in fold steps, and this module does not
//! claim it is.** `MAX_EVALUATION_STEPS` counts scalar applications and
//! index-expression evaluations, of which one fold step performs several, so a
//! fold-step count above the cap is *sufficient* for refusal but the cap is not
//! a fold-step bound. Measured here rather than assumed: raising the constant
//! from 16Mi to 64Mi left the B1-a prefill refusal firing unchanged, so the true
//! cost of that row is more than four times its fold-step count. An earlier
//! revision of this comment said the row was refused "at twice the cap", which
//! that observation refutes.
//!
//! **The decode rows are the cheap ones precisely because `T = 1`**: one query
//! position against a grown context is a small output with a long fold in
//! structure 3, and a small output with a fixed fold in structure 2. So "at
//! least one B1 extent" is met by a decode row rather than by relaxing a bound
//! that protects the host, and [`the_prefill_b1_row_exceeds_the_region_oracle_budget`]
//! asserts the refusal rather than routing around it.
//!
//! **What it would take for that check to say *no*, and that the case is
//! reachable.** It says no if the oracle stops refusing the prefill row. Both
//! branches are exercised in this file rather than assumed: the same
//! `emitted_region_evaluation` returns `Ok` for the two reached rows in
//! [`both_attention_contractions_agree_with_the_reference_at_c1_and_b1`] and
//! `Err` for the prefill row here, so the refusal discriminates rows rather than
//! refusing everything.

use tiler_ir::index::{IndexRefinementSubject, NumericalContractIdentity};
use tiler_ir::semantic::{
    ContractionIndex, ContractionIndexStructure, F32, F32TensorContraction, InputKey, OutputKey,
    SemanticProgram, SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;
use tiler_reference::{
    FloatBitOrder, FrozenReferenceRegistry, FrozenScalarReferenceRegistry, IndexRegionAuthority,
    IndexRegionEvaluationError, IndexRegionEvaluator, IndexRegionInput, InputBinding,
    ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

use super::{governed_lowering_capabilities, governed_scalars};
use crate::capability::LoweringSignature;
use crate::legality::refine_index_region;

/// Frontend labels for the two attention structures, deliberately not dense.
///
/// Spelled with the same arbitrary labels `tiler-ir`'s own structure tests use,
/// so the renaming-invariant canonicalization is exercised rather than assumed:
/// a lowering that only worked for densely-numbered indices would pass a
/// dense fixture and fail the workload.
const G: ContractionIndex = ContractionIndex::new(70);
const R: ContractionIndex = ContractionIndex::new(71);
const TQ: ContractionIndex = ContractionIndex::new(72);
const SK: ContractionIndex = ContractionIndex::new(73);
const DH: ContractionIndex = ContractionIndex::new(74);

/// Key/value groups, shared by the C1 and B1 rows.
const GROUPS: u64 = 8;
/// Grouped-query repetition, so `GROUPS * REPEATS` is the 16 query heads.
const REPEATS: u64 = 2;
/// Head lane width.
const HEAD_DIM: u64 = 128;

/// The score structure, `grtd,gsd->grts`.
///
/// `r` is in the query operand and the result and never in the key operand,
/// which is what makes the 8-to-16 repetition free rather than a broadcast.
fn score_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new([vec![G, R, TQ, DH], vec![G, SK, DH]], [G, R, TQ, SK])
        .expect("grtd,gsd->grts is admitted")
}

/// The value structure, `grts,gsd->grtd`.
fn value_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new([vec![G, R, TQ, SK], vec![G, SK, DH]], [G, R, TQ, DH])
        .expect("grts,gsd->grtd is admitted")
}

fn left_key() -> InputKey {
    InputKey::new("left").expect("a governed key")
}

fn right_key() -> InputKey {
    InputKey::new("right").expect("a governed key")
}

/// One attention contraction at one row, as the shapes its structure implies.
///
/// `groups` and `head_dim` are carried rather than fixed so a fixture can state
/// a *small* cell without ceasing to be the same five-index structure. The
/// workload rows below all use the pinned `GROUPS` and `HEAD_DIM`; only the
/// perturbation fixture departs, and it says why.
#[derive(Clone, Copy)]
struct Cell {
    id: &'static str,
    /// Key/value groups.
    groups: u64,
    /// Query positions.
    t: u64,
    /// Key positions.
    s: u64,
    /// Head lane width, which is structure 2's contracted extent.
    head_dim: u64,
}

/// The C1 prefill row: `S = T = 10`, the conformance track's own extents.
const C1_PREFILL: Cell = Cell {
    id: "c1_prefill",
    groups: GROUPS,
    t: 10,
    s: 10,
    head_dim: HEAD_DIM,
};

/// The B1-a decode row at its last step: one query position, 256 of context.
const B1A_DECODE: Cell = Cell {
    id: "b1a_decode",
    groups: GROUPS,
    t: 1,
    s: 256,
    head_dim: HEAD_DIM,
};

/// The B1-a prefill row, reached by neither oracle evaluation.
const B1A_PREFILL: Cell = Cell {
    id: "b1a_prefill",
    groups: GROUPS,
    t: 128,
    s: 128,
    head_dim: HEAD_DIM,
};

/// A small cell carrying all five indices, used for the perturbation below.
///
/// Small so the fixture is cheap, and still five indices so what it perturbs is
/// the structure this module is about rather than a rank-2 stand-in.
const PERTURBATION_CELL: Cell = Cell {
    id: "perturbation",
    groups: 2,
    t: 2,
    s: 3,
    head_dim: 4,
};

/// How far the perturbation below moves its operand element, and why not less.
///
/// **One unit in the last place is the wrong instrument here, and the reason is
/// arithmetic rather than a defect.** The projection harness perturbs by a
/// single representable step and calls it "the smallest change the fold can
/// observe at all". That phrasing does not transfer, and assuming it did cost
/// this module two failing runs before the cause was derived:
///
/// A perturbation of one ULP of an operand near `0.5` is about `3e-8`. It enters
/// the result multiplied by the *other* operand, whose magnitude is below `1`,
/// so its contribution to the sum is *smaller* still — about `7.5e-9` here. The
/// sum it must move has magnitude of order the operand scale, whose own `f32`
/// ULP is about `6e-8`. The contribution is therefore below the accumulator's
/// resolution and is rounded away. **Shrinking the contracted extent does not
/// fix this**, which is the part worth recording: the fold's magnitude does not
/// fall proportionally with its length, so the ratio barely moves. Observed
/// directly — at `head_dim = 128` all 96 outputs were bit-identical after the
/// one-ULP change, and at a contracted extent of four all 24 still were.
///
/// That the projection harness's `k = 4` cell happens to flip a bit is luck of
/// its particular operands, not a property the instrument has. So this module
/// perturbs by a whole `1.0`: large enough that the changed product moves the
/// sum by roughly the operand scale, which is seven orders of magnitude above
/// the accumulator's ULP. It still discriminates exactly what the test exists
/// for — an ignored contributor, or a comparison that never read the operands —
/// and it discriminates them deterministically instead of by luck.
const PERTURBATION_OFFSET: f32 = 1.0;

/// Which of the two structures a fixture is built over.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Structure {
    /// `grtd,gsd->grts`, contracting the static head lane.
    Score,
    /// `grts,gsd->grtd`, contracting the growing key position.
    Value,
}

impl Structure {
    fn structure(self) -> ContractionIndexStructure {
        match self {
            Self::Score => score_structure(),
            Self::Value => value_structure(),
        }
    }

    /// The left operand's shape at one cell.
    fn left_shape(self, cell: Cell) -> Shape {
        match self {
            Self::Score => Shape::from_dims([cell.groups, REPEATS, cell.t, cell.head_dim]),
            Self::Value => Shape::from_dims([cell.groups, REPEATS, cell.t, cell.s]),
        }
    }

    /// The right operand is `[g, s, d]` for both structures.
    fn right_shape(self, cell: Cell) -> Shape {
        let _ = self;
        Shape::from_dims([cell.groups, cell.s, cell.head_dim])
    }

    /// The result's shape at one cell.
    fn output_shape(self, cell: Cell) -> Shape {
        match self {
            Self::Score => Shape::from_dims([cell.groups, REPEATS, cell.t, cell.s]),
            Self::Value => Shape::from_dims([cell.groups, REPEATS, cell.t, cell.head_dim]),
        }
    }

    /// The contracted extent: the static head lane, or the growing context.
    fn contracted_extent(self, cell: Cell) -> u64 {
        match self {
            Self::Score => cell.head_dim,
            Self::Value => cell.s,
        }
    }

    /// Fold steps, which is the quantity both oracles are budgeted in.
    fn fold_steps(self, cell: Cell) -> u64 {
        self.output_shape(cell)
            .element_count()
            .expect("a conformance cell is bounded") as u64
            * self.contracted_extent(cell)
    }
}

/// The workload seed, distinct per operand so the two are not the same tensor.
const LEFT_SEED: u64 = 0x4154_5445_4E00;
const RIGHT_SEED_MASK: u64 = 0xA5A5_A5A5_A5A5_A5A5;

fn splitmix64(x: u64) -> u64 {
    let x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let z = x;
    let z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    let z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// An operand value at one index: `m * 2^-24` with `m` an integer in
/// `[-2^23, 2^23)`.
///
/// Every such value is exactly representable in binary32, so the operands
/// themselves introduce no rounding and any difference a comparison reports is a
/// difference in how the contraction was evaluated. This is the projection
/// probe's own generator, reused so the two conformance modules discriminate
/// the same way.
fn prng_value(seed: u64, index: u64) -> f32 {
    let bits = splitmix64(seed.wrapping_add(index.wrapping_mul(0x2545_F491_4F6C_DD1D)));
    let field =
        i64::from(u32::try_from((bits >> 40) & 0xFF_FFFF).expect("a 24-bit field fits in u32"));
    #[expect(
        clippy::cast_precision_loss,
        reason = "an integer in [-2^23, 2^23) is exactly representable in binary32"
    )]
    let magnitude = (field - 8_388_608) as f32;
    magnitude * (1.0 / 16_777_216.0)
}

fn prng_tensor(shape: Shape, seed: u64) -> Tensor {
    let count = shape
        .element_count()
        .expect("a conformance cell is bounded");
    let elements = (0..count)
        .map(|index| {
            ReferenceElement::from_float_bits(
                prng_value(seed, index as u64).to_bits().to_be_bytes(),
                FloatBitOrder::MostSignificantByteFirst,
            )
            .expect("a generated operand is a valid f32 pattern")
        })
        .collect();
    Tensor::dense(F32::resolved_type(), shape, elements).expect("the operand tensor is well formed")
}

/// Builds `left x right -> result` for one structure at one cell.
fn attention_program(structure: Structure, cell: Cell) -> SemanticProgram {
    let mut builder =
        SemanticProgramBuilder::try_standard().expect("the standard registry composes");
    let left = builder
        .input::<F32>(left_key(), structure.left_shape(cell))
        .expect("the first operand is declared");
    let right = builder
        .input::<F32>(right_key(), structure.right_shape(cell))
        .expect("the second operand is declared");
    let result = F32TensorContraction::apply(&mut builder, &structure.structure(), left, right)
        .expect("the occurrence is well formed");
    builder
        .output(OutputKey::new("result").expect("a governed key"), result)
        .expect("the result is named");
    builder.build().expect("the program verifies")
}

/// Returns the exact `f32` bit patterns of a dense reference tensor.
fn result_bits(tensor: &Tensor) -> Vec<u32> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("a contraction result is a dense f32 tensor")
    };
    elements
        .iter()
        .map(|value| {
            u32::from_be_bytes(
                <[u8; 4]>::try_from(value.as_bytes()).expect("an f32 element is four bytes"),
            )
        })
        .collect()
}

/// Evaluates one attention contraction through the registered reference.
fn reference_result(program: &SemanticProgram, left: &Tensor, right: &Tensor) -> Vec<u32> {
    let evaluator = ReferenceEvaluator::new(
        FrozenReferenceRegistry::standard().expect("the governed value profile composes"),
    );
    let outputs = evaluator
        .evaluate(
            program,
            &[
                InputBinding::new(&left_key(), left),
                InputBinding::new(&right_key(), right),
            ],
        )
        .expect("the governed reference evaluates the attention cell");
    result_bits(&outputs[0])
}

/// Evaluates the region the governed lowering emitted for the same occurrence.
fn emitted_region_result(
    structure: Structure,
    cell: Cell,
    left: &Tensor,
    right: &Tensor,
) -> Vec<u32> {
    emitted_region_evaluation(structure, cell, left, right)
        .expect("the emitted region executes on the oracle")
}

fn emitted_region_evaluation(
    structure: Structure,
    cell: Cell,
    left: &Tensor,
    right: &Tensor,
) -> Result<Vec<u32>, IndexRegionEvaluationError> {
    let scalars = governed_scalars().expect("the governed scalar authority composes");
    let registry =
        governed_lowering_capabilities(&scalars).expect("the governed capabilities compose");
    let realizations = super::governed_realization_laws(&scalars);
    let program = attention_program(structure, cell);
    let occurrence = IndexRefinementSubject::derive(
        &program,
        program
            .operations()
            .next()
            .expect("the program carries its contraction")
            .id(),
        NumericalContractIdentity::try_from_key(
            crate::request::StrictF32NumericalContract::governed().key,
        )
        .expect("the governed contract key resolves"),
    )
    .expect("the verified contraction yields a refinement subject");
    let signature = LoweringSignature::new(
        occurrence.signature().operands().iter().cloned(),
        occurrence.signature().results().iter().cloned(),
    )
    .expect("the occurrence's signature is bounded");
    let resolved = registry
        .resolve_index_access(occurrence.operation(), &signature)
        .expect("the governed registry covers the contraction");
    let refinement = refine_index_region(&resolved, &occurrence, &realizations, &scalars)
        .expect("the governed attention contraction lowering refines")
        .into_refined()
        .expect("the lowering discharges every index-domain predicate");
    let evaluator = IndexRegionEvaluator::new(
        FrozenReferenceRegistry::standard().expect("the governed value profile composes"),
        FrozenScalarReferenceRegistry::standard().expect("the governed scalar oracle composes"),
    );
    let evaluation = evaluator.evaluate(
        refinement
            .single_region()
            .expect("every governed family realizes its occurrence in one region"),
        IndexRegionAuthority::new(&scalars),
        &[
            IndexRegionInput::new(refinement.operand_bindings()[0].input_tensor(), left),
            IndexRegionInput::new(refinement.operand_bindings()[1].input_tensor(), right),
        ],
    )?;
    Ok(result_bits(&evaluation.outputs()[0]))
}

/// Builds the operand pair for one structure at one cell.
fn operands(structure: Structure, cell: Cell) -> (Tensor, Tensor) {
    (
        prng_tensor(structure.left_shape(cell), LEFT_SEED),
        prng_tensor(structure.right_shape(cell), LEFT_SEED ^ RIGHT_SEED_MASK),
    )
}

/// Both attention contractions' emitted regions agree with the reference, bit
/// for bit, at the C1 prefill row and at a B1 decode row.
///
/// Comparison is on exact bit patterns rather than `f32` equality: `-0.0 == 0.0`
/// holds and a NaN equals nothing, so float comparison would silently accept
/// exactly the results a numerical contract exists to pin.
#[test]
fn both_attention_contractions_agree_with_the_reference_at_c1_and_b1() {
    for structure in [Structure::Score, Structure::Value] {
        for cell in [C1_PREFILL, B1A_DECODE] {
            let (left, right) = operands(structure, cell);
            let program = attention_program(structure, cell);
            let reference = reference_result(&program, &left, &right);
            let expected_outputs = structure
                .output_shape(cell)
                .element_count()
                .expect("a conformance cell is bounded");
            assert_eq!(
                reference.len(),
                expected_outputs,
                "{:?}/{}: every output element is produced",
                structure,
                cell.id
            );
            let emitted = emitted_region_result(structure, cell, &left, &right);
            assert_eq!(
                emitted, reference,
                "{:?}/{}: the emitted region disagrees with the reference evaluator",
                structure, cell.id
            );
        }
    }
}

/// One perturbed contributing element changes the compared result.
///
/// Without this the agreement above would be consistent with a comparison that
/// never looked at the operands. Perturbing the *last* contributed element is
/// deliberate: a fold that stopped early, or one seeded at `+0.0` and therefore
/// ignoring its first contributor, would still be caught by a first-element
/// perturbation, so the last one is the position that discriminates the fold's
/// completeness.
#[test]
fn a_single_perturbed_contributor_breaks_both_attention_comparisons() {
    let cell = PERTURBATION_CELL;
    for structure in [Structure::Score, Structure::Value] {
        let (left, right) = operands(structure, cell);
        let program = attention_program(structure, cell);
        let baseline = reference_result(&program, &left, &right);
        assert_eq!(
            emitted_region_result(structure, cell, &left, &right),
            baseline,
            "{structure:?}: the unperturbed pair agrees, so the perturbation below is the variable"
        );

        let TensorPayloadView::Dense(elements) = left.payload() else {
            panic!("a dense operand")
        };
        let mut perturbed: Vec<ReferenceElement> = elements.to_vec();
        let last = perturbed.len() - 1;
        let bits = u32::from_be_bytes(
            <[u8; 4]>::try_from(perturbed[last].as_bytes()).expect("an f32 element is four bytes"),
        );
        // See `PERTURBATION_OFFSET` for why this is a whole unit rather than one
        // unit in the last place.
        let moved = f32::from_bits(bits) + PERTURBATION_OFFSET;
        perturbed[last] = ReferenceElement::from_float_bits(
            moved.to_bits().to_be_bytes(),
            FloatBitOrder::MostSignificantByteFirst,
        )
        .expect("a perturbed operand is a valid f32 pattern");
        let left = Tensor::dense(F32::resolved_type(), structure.left_shape(cell), perturbed)
            .expect("the perturbed operand tensor is well formed");

        assert_ne!(
            reference_result(&program, &left, &right),
            baseline,
            "{structure:?}: the reference ignored a contributing element"
        );
        assert_ne!(
            emitted_region_result(structure, cell, &left, &right),
            baseline,
            "{structure:?}: the emitted region ignored a contributing element"
        );
    }
}

/// The prefill B1 row exceeds the region oracle's step budget, for both
/// structures.
///
/// This is the other half of the reach statement in this module's
/// documentation. The bound is a property of `tiler-reference`'s oracle, not of
/// the emitted region, and a budget that had been raised — or a refusal reported
/// under a different resource — would silently change which rows "the oracle can
/// evaluate" names. Raising it belongs to `tiler-reference`, which this work
/// does not own.
#[test]
fn the_prefill_b1_row_exceeds_the_region_oracle_budget() {
    /// The oracle's own bound, restated so the arithmetic below is checked
    /// against a number rather than against a hope.
    ///
    /// Compared against *fold* steps, which the module documentation explains is
    /// a sufficient condition for refusal rather than the oracle's own unit: one
    /// fold step costs the oracle several. So `fold_steps > REGION_STEP_LIMIT`
    /// implies the row is refused, while `fold_steps <= REGION_STEP_LIMIT` does
    /// not by itself imply the row is reached — that direction is established by
    /// actually evaluating those rows in the agreement test, not here.
    const REGION_STEP_LIMIT: u64 = 16 * 1024 * 1024;

    for structure in [Structure::Score, Structure::Value] {
        assert!(
            structure.fold_steps(B1A_PREFILL) > REGION_STEP_LIMIT,
            "{structure:?}: the recomputed fold must exceed the bound this refusal names",
        );
        let (left, right) = operands(structure, B1A_PREFILL);
        let error = emitted_region_evaluation(structure, B1A_PREFILL, &left, &right)
            .expect_err("this row exceeds the oracle's evaluation budget");
        assert!(
            format!("{error}").contains("EvaluationSteps"),
            "{structure:?}: the refusal must name the oracle's step budget: {error}"
        );
    }

    // And the two rows that *are* reached stay under it, so the boundary above
    // is a statement about these extents rather than about the structures.
    for structure in [Structure::Score, Structure::Value] {
        for cell in [C1_PREFILL, B1A_DECODE] {
            assert!(
                structure.fold_steps(cell) <= REGION_STEP_LIMIT,
                "{:?}/{}: a reached row must be inside the budget",
                structure,
                cell.id
            );
        }
    }
}

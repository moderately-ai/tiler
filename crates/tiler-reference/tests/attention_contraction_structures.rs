//! The two attention index structures, reference-evaluated at the C1 prefill
//! shapes: `grtd,gsd->grts` and `grts,gsd->grtd`.
//!
//! # What these structures are
//!
//! They are structure *values* under the one governed key
//! `tiler::tensor-contraction-f32@1`, not new keys — ADR 0087 accepts a
//! single family whose node carries its index structure as an attribute. Nothing
//! in `tiler-ir` or `tiler-reference` was widened to admit them: the five
//! structural rules already admit every well-formed binary structure, and the
//! evaluator already folds an arbitrary admitted one. This file is the corpus
//! that says so, at four-wide tuples and five indices rather than at structure
//! 1's two-wide tuples and three.
//!
//! No workload-named constructor is added to the neutral core, following the
//! grouped-query head-layout profile's precedent: `grtd` is a consumer's reading
//! of an index tuple, and the compiler's semantic model does not learn it. The
//! constructors below are local to this test.
//!
//! # The oracle, and what makes it independent
//!
//! The primary comparison is **repeat-then-matmul**, which is the pinned
//! reference's own composition: materialize the eight key heads into sixteen by
//! repeat-interleave, then contract each query head's `[T, 128]` slab against its
//! key head's `[S, 128]` slab under the *already-validated* structure 1
//! `td,od->to`. That path states which contributor pairs with which through a
//! different structure, a different program, and an explicit materialization, so
//! an error in structure 2's or 3's access relation cannot cancel against it.
//!
//! `the_declared_order_removes_the_disagreement_the_probe_measured` records the
//! consequence and is the reason the order contract sits on the structure. The
//! retained [attention-block probe] measures `torch`'s einsum kernel against
//! `torch`'s matmul kernel and finds **943 of 1,600** F32 elements differing at a
//! maximum absolute gap of 1.72e-5, and **0 of 1,600** when both are evaluated in
//! float64 and rounded once. Two spellings of one structure, no permission
//! distinguishing them, different bits — because neither `torch` path declares a
//! contributor order. Under this family's declared order both spellings fold the
//! identical contributor sequence, so the count here is **0 of 1,600 in F32**.
//! That is the contract doing the work the probe showed was missing; it is not a
//! reproduction of the 943, which is a property of `torch`'s two kernels and of
//! the probe's seed and is not reproducible from a declared order at all.
//!
//! A secondary comparison folds each structure by hand, indexing the operands
//! straight from the einsum letters. Its independence is narrower and stated
//! rather than implied: it restates the *access relation* independently and runs
//! the same binary32 multiply and add in the same declared order, so it
//! discriminates a wrong index binding and says nothing about the arithmetic.
//!
//! # Boundary
//!
//! Every extent is static and every operand is `tiler::f32@1`. A semantic value
//! fact carries a static extent, so the growing context length `S` is exercised
//! at the static values the C1 row takes — ten at prefill, and up to eighteen
//! across its decode — and never as a symbol; the unresolved third outcome of the
//! extent-agreement path stays unreachable, exactly as the projection profile's
//! landing recorded. Operands are deterministic synthetic data, not the
//! checkpoint, and the asserted counts are the data-robust ones: zero differing
//! elements, and the fourteen query heads whose source differs between the two
//! grouped-query readings.
//!
//! This is evidence about the semantic contract and the host reference
//! evaluator. It establishes nothing about a schedule, a lowering, a kernel, a
//! device, or a model-level tolerance; none is exercised.
//!
//! [attention-block probe]: ../../../spikes/program-planning/attention-block-reference/README.md

use tiler_ir::semantic::{
    ContractionIndex, ContractionIndexStructure, F32, F32TensorContraction, InputKey, OutputKey,
    SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

/// The C1 prefill row's extents, from the retained attention-block probe.
const GROUPS: usize = 8;
const REPEATS: usize = 2;
const QUERY_HEADS: usize = GROUPS * REPEATS;
const POSITIONS: usize = 10;
const HEAD_DIM: usize = 128;

// --- the two structures ------------------------------------------------------

/// Frontend labels, deliberately neither dense nor ascending.
const G: u32 = 70;
const R: u32 = 71;
const T: u32 = 72;
const S: u32 = 73;
const D: u32 = 74;

fn index(label: u32) -> ContractionIndex {
    ContractionIndex::new(label)
}

/// The score structure, `grtd,gsd->grts`.
///
/// `r` — the grouped-query repetition — is in the query operand and the result
/// and in neither the key operand nor the contracted set. That is what makes the
/// eight-to-sixteen repetition free: no `[16, S, 128]` key is materialized.
fn score_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [
            vec![index(G), index(R), index(T), index(D)],
            vec![index(G), index(S), index(D)],
        ],
        [index(G), index(R), index(T), index(S)],
    )
    .expect("grtd,gsd->grts is admitted")
}

/// The value structure, `grts,gsd->grtd`, which contracts over the growing `S`.
fn value_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [
            vec![index(G), index(R), index(T), index(S)],
            vec![index(G), index(S), index(D)],
        ],
        [index(G), index(R), index(T), index(D)],
    )
    .expect("grts,gsd->grtd is admitted")
}

/// Structure 1, `td,od->to` — the projection structure, used here as the oracle.
fn projection_structure() -> ContractionIndexStructure {
    ContractionIndexStructure::new(
        [vec![index(0), index(1)], vec![index(2), index(1)]],
        [index(0), index(2)],
    )
    .expect("td,od->to is admitted")
}

// --- fixtures ----------------------------------------------------------------

/// Deterministic synthetic operands in `[-2, 2)`.
///
/// A local generator rather than a transcribed vector: the retained probe's own
/// operands come from a `torch` seed this crate cannot reproduce, and the counts
/// asserted below are the ones the probe itself states generalize past its seed.
fn samples(count: usize, seed: u64) -> Vec<f32> {
    let mut state = seed | 1;
    (0..count)
        .map(|_| {
            // SplitMix64, so consecutive draws do not correlate across the
            // strides the contractions read them at.
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            // Assembled from bits rather than cast from an integer: the mantissa
            // lands in `[1, 2)` exactly, the scale by four is exact, and the
            // subtraction is exact by Sterbenz over `[4, 8)`. So the generator
            // introduces no rounding the corpus would then be asserting about.
            // Written as separate operations rather than `mul_add`, which is the
            // fusion this family declares forbidden.
            let mantissa = ((z ^ (z >> 31)) >> 40) as u32 & 0x007f_ffff;
            f32::from_bits(0x3f80_0000 | mantissa) * 4.0 - 6.0
        })
        .collect()
}

fn tensor<const N: usize>(dims: [u64; N], values: &[f32]) -> Tensor {
    let shape = Shape::from_dims(dims);
    assert_eq!(
        shape.element_count(),
        Some(values.len()),
        "a fixture states every element"
    );
    Tensor::dense(
        F32::resolved_type(),
        shape,
        values
            .iter()
            .map(|value| {
                ReferenceElement::from_float_bits(
                    value.to_bits().to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .expect("an f32 payload is four bytes")
            })
            .collect(),
    )
    .expect("the fixture is well formed")
}

fn payload_bits(tensor: &Tensor) -> Vec<u32> {
    let TensorPayloadView::Dense(elements) = tensor.payload() else {
        panic!("a covered result is a dense f32 tensor");
    };
    elements
        .iter()
        .map(|element| {
            u32::from_be_bytes(
                <[u8; 4]>::try_from(element.as_bytes()).expect("an f32 element is four bytes"),
            )
        })
        .collect()
}

/// Evaluates one contraction through the public builder and reference evaluator.
fn contract(structure: &ContractionIndexStructure, left: &Tensor, right: &Tensor) -> Vec<u32> {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let left_key = InputKey::new("left").expect("a valid key");
    let right_key = InputKey::new("right").expect("a valid key");
    let left_value = builder
        .input::<F32>(left_key.clone(), left.shape().clone())
        .expect("an F32 input");
    let right_value = builder
        .input::<F32>(right_key.clone(), right.shape().clone())
        .expect("an F32 input");
    let result = F32TensorContraction::apply(&mut builder, structure, left_value, right_value)
        .expect("an admitted structure is accepted");
    builder
        .output(OutputKey::new("result").expect("a valid key"), result)
        .expect("an output");
    let program = builder.build().expect("the program is complete");
    let outputs = ReferenceEvaluator::standard()
        .expect("the standard evaluator opens")
        .evaluate(
            &program,
            &[
                InputBinding::new(&left_key, left),
                InputBinding::new(&right_key, right),
            ],
        )
        .expect("a covered program evaluates");
    let [output] = outputs.as_slice() else {
        panic!("a covered program has one output");
    };
    payload_bits(output)
}

fn differing(left: &[u32], right: &[u32]) -> usize {
    assert_eq!(left.len(), right.len(), "a comparison is element-wise");
    left.iter().zip(right).filter(|(a, b)| a != b).count()
}

/// Which key head query head `h` reads.
///
/// `repeat_kv` is repeat-interleave, so the group is `h / REPEATS`. Repeat-tile,
/// `h % GROUPS`, produces an identically shaped tensor and is the reading the
/// retained probe measures wrong for fourteen of the sixteen heads; it is the
/// perturbation below.
fn interleaved_group(head: usize) -> usize {
    head / REPEATS
}

fn tiled_group(head: usize) -> usize {
    head % GROUPS
}

// --- the score structure -----------------------------------------------------

/// `grtd,gsd->grts` denotes repeat-then-matmul, bit for bit under the declared
/// order — and the declared order is what makes that true in F32.
///
/// The probe measures `torch`'s two spellings differing at 943 of 1,600 F32
/// elements and agreeing at 0 of 1,600 in float64: structurally one computation,
/// two undeclared reduction orders. This family declares the order, so the two
/// spellings fold the identical contributor sequence and the F32 count is 0.
#[test]
fn the_declared_order_removes_the_disagreement_the_probe_measured() {
    let query = samples(GROUPS * REPEATS * POSITIONS * HEAD_DIM, 0x005c_07e5);
    let key = samples(GROUPS * POSITIONS * HEAD_DIM, 0x0000_4e77);

    let structured = contract(
        &score_structure(),
        &tensor(
            [
                GROUPS as u64,
                REPEATS as u64,
                POSITIONS as u64,
                HEAD_DIM as u64,
            ],
            &query,
        ),
        &tensor([GROUPS as u64, POSITIONS as u64, HEAD_DIM as u64], &key),
    );
    assert_eq!(structured.len(), 1_600, "the C1 score element count");

    let repeated = repeat_then_matmul_scores(&query, &key, interleaved_group);
    assert_eq!(
        differing(&structured, &repeated),
        0,
        "under a declared contributor order the einsum spelling and \
         repeat-then-matmul are the same bits, not merely the same values"
    );

    // The perturbation, which is the probe's own grouped-query finding: reading
    // the key head as `h % 8` instead of `h / 2` produces an identically shaped
    // result and pairs fourteen of the sixteen query heads with a different key
    // head. A comparison that could not fail would report agreement here too.
    let tiled = repeat_then_matmul_scores(&query, &key, tiled_group);
    assert_ne!(differing(&structured, &tiled), 0);
    let per_head = POSITIONS * POSITIONS;
    let differing_heads = (0..QUERY_HEADS)
        .filter(|head| {
            let range = head * per_head..(head + 1) * per_head;
            structured[range.clone()] != tiled[range]
        })
        .count();
    assert_eq!(
        differing_heads, 14,
        "repeat-interleave and repeat-tile agree only at heads 0 and 15"
    );
}

/// Contracts each query head's slab against its key head's slab under structure 1.
///
/// The result is laid out `(g, r, t, s)`, which is the score structure's output
/// tuple, so the comparison above is element-wise without a permutation.
fn repeat_then_matmul_scores(
    query: &[f32],
    key: &[f32],
    group_of: impl Fn(usize) -> usize,
) -> Vec<u32> {
    let mut scores = Vec::with_capacity(QUERY_HEADS * POSITIONS * POSITIONS);
    for head in 0..QUERY_HEADS {
        let query_slab = &query[head * POSITIONS * HEAD_DIM..(head + 1) * POSITIONS * HEAD_DIM];
        let group = group_of(head);
        let key_slab = &key[group * POSITIONS * HEAD_DIM..(group + 1) * POSITIONS * HEAD_DIM];
        scores.extend(contract(
            &projection_structure(),
            &tensor([POSITIONS as u64, HEAD_DIM as u64], query_slab),
            &tensor([POSITIONS as u64, HEAD_DIM as u64], key_slab),
        ));
    }
    scores
}

// --- the value structure -----------------------------------------------------

/// `grts,gsd->grtd` denotes repeat-then-matmul too, over the growing extent.
///
/// The oracle transposes each value head to `[128, S]` so the already-validated
/// `td,od->to` applies, which puts the contracted key position last in both
/// operands and folds it in the same ascending order the value structure does.
#[test]
fn the_value_structure_denotes_repeat_then_matmul_bit_for_bit() {
    let probabilities = samples(GROUPS * REPEATS * POSITIONS * POSITIONS, 0x0000_9a1e);
    let values = samples(GROUPS * POSITIONS * HEAD_DIM, 0x0000_7c3d);

    let structured = contract(
        &value_structure(),
        &tensor(
            [
                GROUPS as u64,
                REPEATS as u64,
                POSITIONS as u64,
                POSITIONS as u64,
            ],
            &probabilities,
        ),
        &tensor([GROUPS as u64, POSITIONS as u64, HEAD_DIM as u64], &values),
    );
    assert_eq!(
        structured.len(),
        20_480,
        "the C1 attention-output element count"
    );

    let repeated = repeat_then_matmul_values(&probabilities, &values, interleaved_group);
    assert_eq!(differing(&structured, &repeated), 0);

    let tiled = repeat_then_matmul_values(&probabilities, &values, tiled_group);
    assert_ne!(
        differing(&structured, &tiled),
        0,
        "the value contraction reads a repeated value head, so the same \
         grouped-query reading is load-bearing here"
    );
}

fn repeat_then_matmul_values(
    probabilities: &[f32],
    values: &[f32],
    group_of: impl Fn(usize) -> usize,
) -> Vec<u32> {
    let mut outputs = Vec::with_capacity(QUERY_HEADS * POSITIONS * HEAD_DIM);
    for head in 0..QUERY_HEADS {
        let probability_slab =
            &probabilities[head * POSITIONS * POSITIONS..(head + 1) * POSITIONS * POSITIONS];
        let group = group_of(head);
        let value_slab = &values[group * POSITIONS * HEAD_DIM..(group + 1) * POSITIONS * HEAD_DIM];
        // `[S, 128] -> [128, S]`, so the contracted key position is the last axis
        // of both operands and `td,od->to` applies.
        let mut transposed = vec![0.0_f32; HEAD_DIM * POSITIONS];
        for position in 0..POSITIONS {
            for lane in 0..HEAD_DIM {
                transposed[lane * POSITIONS + position] = value_slab[position * HEAD_DIM + lane];
            }
        }
        outputs.extend(contract(
            &projection_structure(),
            &tensor([POSITIONS as u64, POSITIONS as u64], probability_slab),
            &tensor([HEAD_DIM as u64, POSITIONS as u64], &transposed),
        ));
    }
    outputs
}

// --- the strict fold, stated independently from the einsum letters -----------

/// Both structures reproduce a fold written straight from the index letters.
///
/// Independence boundary, stated rather than implied: this restates the *access
/// relation* — which operand element pairs with which, at which output
/// coordinate — and runs the same binary32 multiply and add in the same declared
/// ascending order. It discriminates a wrong index binding and is silent about
/// the arithmetic, which the transcribed exceptional-value corpus in
/// `contraction_conformance.rs` covers instead.
#[test]
fn both_structures_reproduce_a_fold_written_from_the_index_letters() {
    // A small shape, so the loops below are readable as the spelling they state.
    let (groups, repeats, positions, head_dim) = (2_usize, 2_usize, 3_usize, 4_usize);
    let query = samples(groups * repeats * positions * head_dim, 0x0000_11a7);
    let key = samples(groups * positions * head_dim, 0x0000_22b8);

    // grtd,gsd->grts
    let mut expected = Vec::new();
    for g in 0..groups {
        for r in 0..repeats {
            for t in 0..positions {
                for s in 0..positions {
                    let mut accumulator: Option<f32> = None;
                    for d in 0..head_dim {
                        let left = query[((g * repeats + r) * positions + t) * head_dim + d];
                        let right = key[(g * positions + s) * head_dim + d];
                        let product = left * right;
                        accumulator = Some(accumulator.map_or(product, |value| value + product));
                    }
                    expected.push(accumulator.expect("a nonempty fold").to_bits());
                }
            }
        }
    }
    let scores = contract(
        &score_structure(),
        &tensor(
            [
                groups as u64,
                repeats as u64,
                positions as u64,
                head_dim as u64,
            ],
            &query,
        ),
        &tensor([groups as u64, positions as u64, head_dim as u64], &key),
    );
    assert_eq!(differing(&scores, &expected), 0);

    // grts,gsd->grtd, over the same operands reshaped to the value structure's
    // tuples: the probability tensor is `[g, r, t, s]` and the value tensor
    // `[g, s, d]`.
    let probabilities = samples(groups * repeats * positions * positions, 0x0000_33c9);
    let values = samples(groups * positions * head_dim, 0x0000_44da);
    let mut expected = Vec::new();
    for g in 0..groups {
        for r in 0..repeats {
            for t in 0..positions {
                for d in 0..head_dim {
                    let mut accumulator: Option<f32> = None;
                    for s in 0..positions {
                        let left =
                            probabilities[((g * repeats + r) * positions + t) * positions + s];
                        let right = values[(g * positions + s) * head_dim + d];
                        let product = left * right;
                        accumulator = Some(accumulator.map_or(product, |value| value + product));
                    }
                    expected.push(accumulator.expect("a nonempty fold").to_bits());
                }
            }
        }
    }
    let outputs = contract(
        &value_structure(),
        &tensor(
            [
                groups as u64,
                repeats as u64,
                positions as u64,
                positions as u64,
            ],
            &probabilities,
        ),
        &tensor([groups as u64, positions as u64, head_dim as u64], &values),
    );
    assert_eq!(differing(&outputs, &expected), 0);
}

// --- the signed zero, reachable from ordinary data ---------------------------

/// A masked position contributes a signed zero to the value contraction.
///
/// This is the case structure 3 reaches from ordinary data rather than from a
/// designed exceptional vector. Query position 0 attends to position 0 alone, so
/// its probability row is `1.0` followed by exact `+0.0` entries; each of those
/// contributes `+0.0 * v`, which is `-0.0` wherever `v` is negative. The
/// accumulator starts at the *first product* rather than at `+0.0`, so a first
/// product of `-0.0` survives every subsequent `-0.0` and is rewritten to `+0.0`
/// by the first `+0.0` — the mask changing a result sign inside a contraction
/// that never sees the mask.
///
/// The retained probe observes exactly this at the C1 row: first product
/// `0x80000000`, masked contributor signs `0x00000000 0x80000000 0x80000000`, and
/// a strict ascending fold of `0x00000000`. The operands below are designed to
/// reproduce the mechanism, because the probe's own values come from a `torch`
/// seed; both outcomes are asserted, so the sign is discriminated rather than
/// observed once.
#[test]
fn a_masked_position_contributes_a_signed_zero_to_the_value_contraction() {
    let (groups, repeats, positions, head_dim) = (1_usize, 1_usize, 1_usize, 1_usize);
    // Query position 0 attends to key position 0 alone.
    let probabilities = vec![1.0_f32, 0.0, 0.0, 0.0];
    let attended = probabilities.len();

    // `v[key 0, lane 0]` is exactly negative zero, so the first product is `-0.0`.
    // Every later contributor is `+0.0 * v`, whose sign follows `v`.
    let all_negative = vec![-0.0_f32, -1.0, -1.0, -1.0];
    let bits = contract(
        &value_structure(),
        &tensor(
            [
                groups as u64,
                repeats as u64,
                positions as u64,
                attended as u64,
            ],
            &probabilities,
        ),
        &tensor(
            [groups as u64, attended as u64, head_dim as u64],
            &all_negative,
        ),
    );
    assert_eq!(
        bits,
        vec![0x8000_0000],
        "every contributor is -0.0, so the unseeded fold returns -0.0 where a \
         +0.0-seeded one would return +0.0"
    );

    // One positive value at a masked position is enough to rewrite the sign,
    // which is the probe's C1 observation.
    let one_positive = vec![-0.0_f32, 1.0, -1.0, -1.0];
    let bits = contract(
        &value_structure(),
        &tensor(
            [
                groups as u64,
                repeats as u64,
                positions as u64,
                attended as u64,
            ],
            &probabilities,
        ),
        &tensor(
            [groups as u64, attended as u64, head_dim as u64],
            &one_positive,
        ),
    );
    assert_eq!(
        bits,
        vec![0x0000_0000],
        "fl(-0.0 + +0.0) is +0.0, so a single attended-sign contributor at a \
         masked position flips the result's sign"
    );
}

// --- the growing extent, at the static values C1 takes -----------------------

/// The value structure evaluates at every context length the C1 row reaches.
///
/// `S` is the workload's only growing extent, and a semantic value fact carries a
/// static extent, so it is exercised at ten, sixteen, and eighteen rather than as
/// a symbol. Each is compared against repeat-then-matmul at its own width, so a
/// structure whose access relation happened to be right only at `S == T` would
/// fail here.
#[test]
fn the_value_structure_folds_every_static_context_length_c1_reaches() {
    for context in [10_usize, 16, 18] {
        let probabilities = samples(GROUPS * REPEATS * POSITIONS * context, 0x0000_51e0);
        let values = samples(GROUPS * context * HEAD_DIM, 0x0000_62f1);
        let structured = contract(
            &value_structure(),
            &tensor(
                [
                    GROUPS as u64,
                    REPEATS as u64,
                    POSITIONS as u64,
                    context as u64,
                ],
                &probabilities,
            ),
            &tensor([GROUPS as u64, context as u64, HEAD_DIM as u64], &values),
        );
        assert_eq!(
            structured.len(),
            GROUPS * REPEATS * POSITIONS * HEAD_DIM,
            "the contracted extent leaves the result shape"
        );

        let mut expected = Vec::with_capacity(structured.len());
        for head in 0..QUERY_HEADS {
            let probability_slab =
                &probabilities[head * POSITIONS * context..(head + 1) * POSITIONS * context];
            let group = interleaved_group(head);
            let value_slab = &values[group * context * HEAD_DIM..(group + 1) * context * HEAD_DIM];
            let mut transposed = vec![0.0_f32; HEAD_DIM * context];
            for position in 0..context {
                for lane in 0..HEAD_DIM {
                    transposed[lane * context + position] = value_slab[position * HEAD_DIM + lane];
                }
            }
            expected.extend(contract(
                &projection_structure(),
                &tensor([POSITIONS as u64, context as u64], probability_slab),
                &tensor([HEAD_DIM as u64, context as u64], &transposed),
            ));
        }
        assert_eq!(
            differing(&structured, &expected),
            0,
            "S = {context} agrees with repeat-then-matmul"
        );
    }
}

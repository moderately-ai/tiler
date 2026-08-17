//! Deterministic oracle evidence for the two fixed contraction memberships.
//!
//! This module does not execute a device and makes no Apple support claim. It
//! transcribes the retained eight-case operands and expected `+ftz` results,
//! asks `tiler-reference` to form the separately-rounded products, and then
//! asks its partitioned reduction oracle about each declared membership. The
//! lane-strided case is synthetic because the standard live Apple profile
//! forbids permutation.

use tiler_ir::schedule::{ContributorMembership, FlushedZeroSign, SubnormalMode};
use tiler_ir::semantic::{F32, F32Multiply, InputKey, OutputKey, SemanticProgramBuilder};
use tiler_ir::shape::{Axis, Shape};
use tiler_reference::{
    FloatBitOrder, FrozenReferenceRegistry, InputBinding, ReferenceElement, ReferenceEvaluator,
    ReferenceNumericalConformance, Tensor, TensorPayloadView, strict_partitioned_sum_under,
};

const K: usize = 16;
const CANONICAL_NAN: u32 = 0x7fc0_0000;

#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    left: [u32; K],
    right: [u32; K],
    expected: [u32; 2],
}

const fn pad<const N: usize>(prefix: [u32; N]) -> [u32; K] {
    let mut result = [0_u32; K];
    let mut index = 0;
    while index < N {
        result[index] = prefix[index];
        index += 1;
    }
    result
}

const CASES: [Case; 8] = [
    Case {
        name: "witness",
        left: pad([0x4000_0000]),
        right: pad([0x4040_0000]),
        expected: [0x40c0_0000; 2],
    },
    Case {
        name: "order_absorption",
        left: pad([
            0x4b80_0000,
            0x3fc0_0000,
            0x3fc0_0000,
            0x3fc0_0000,
            0x3fc0_0000,
            0x3fc0_0000,
            0x3fc0_0000,
            0x3fc0_0000,
        ]),
        right: pad([0x3f80_0000; 8]),
        expected: [0x4b80_0006; 2],
    },
    Case {
        name: "contraction_pair",
        left: pad([0x3f80_0000, 0x3eb9_7ef9]),
        right: pad([0x3f80_0000, 0x3fc0_0000]),
        expected: [0x3fc5_8f9e; 2],
    },
    Case {
        name: "split_topology",
        left: [
            0xae8f_cc10,
            0xbef8_ce2c,
            0x3119_8e79,
            0xc009_21ca,
            0x3d9d_1929,
            0x4400_7fca,
            0x41b6_7583,
            0xbb63_d3c2,
            0x4328_602b,
            0x3cb3_5a07,
            0x2d94_1111,
            0xbab1_dbdb,
            0x44ab_a077,
            0x394a_4e66,
            0x3123_ac20,
            0xb603_7546,
        ],
        right: [
            0xb5d8_9c01,
            0xb26d_6247,
            0x3929_72f3,
            0x3a8b_ee84,
            0x30a8_8108,
            0x348c_ef8b,
            0x35b6_5100,
            0x440b_2b3f,
            0xb42c_ea8a,
            0xaf12_8d46,
            0xc613_5504,
            0xc4b2_44a6,
            0xb43b_1f42,
            0x360d_d62a,
            0xc5b5_87e5,
            0xb2f0_4f93,
        ],
        expected: [0xbb1d_0683, 0xbb1d_0672],
    },
    Case {
        name: "negative_zero_seed",
        left: [0xbf80_0000; K],
        right: [0x0000_0000; K],
        expected: [0x8000_0000; 2],
    },
    Case {
        name: "nan_payload",
        left: pad([0x7fc0_dead, 0x3f80_0000]),
        right: pad([0x3f80_0000, 0x3f80_0000]),
        expected: [CANONICAL_NAN; 2],
    },
    Case {
        name: "infinity_times_zero",
        left: pad([0x7f80_0000, 0x3f80_0000]),
        right: pad([0x0000_0000, 0x3f80_0000]),
        expected: [CANONICAL_NAN; 2],
    },
    Case {
        name: "subnormal_product",
        left: pad([0x0080_0000]),
        right: pad([0x3f00_0000]),
        expected: [0x0000_0000; 2],
    },
];

fn tensor(bits: &[u32; K]) -> Tensor {
    Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([K as u64]),
        bits.iter()
            .map(|bits| {
                ReferenceElement::from_float_bits(
                    bits.to_be_bytes(),
                    FloatBitOrder::MostSignificantByteFirst,
                )
                .expect("an f32 payload is four bytes")
            })
            .collect(),
    )
    .expect("the corpus tensor is dense")
}

fn ftz() -> ReferenceNumericalConformance {
    let mode = SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    };
    ReferenceNumericalConformance::new(mode, mode)
}

fn products(case: Case) -> Tensor {
    let mut builder = SemanticProgramBuilder::try_standard().expect("the registry opens");
    let left_key = InputKey::new("left").unwrap();
    let right_key = InputKey::new("right").unwrap();
    let left = builder
        .input::<F32>(left_key.clone(), Shape::from_dims([K as u64]))
        .unwrap();
    let right = builder
        .input::<F32>(right_key.clone(), Shape::from_dims([K as u64]))
        .unwrap();
    let product = F32Multiply::apply(&mut builder, left, right).unwrap();
    builder
        .output(OutputKey::new("products").unwrap(), product)
        .unwrap();
    let program = builder.build().unwrap();
    let left_tensor = tensor(&case.left);
    let right_tensor = tensor(&case.right);
    let outputs = ReferenceEvaluator::under(
        FrozenReferenceRegistry::standard().expect("the reference profile composes"),
        ftz(),
    )
    .evaluate(
        &program,
        &[
            InputBinding::new(&left_key, &left_tensor),
            InputBinding::new(&right_key, &right_tensor),
        ],
    )
    .unwrap();
    outputs.into_iter().next().expect("one product tensor")
}

fn lane_strided(products: &Tensor) -> Tensor {
    let TensorPayloadView::Dense(elements) = products.payload() else {
        panic!("the product tensor is dense")
    };
    let reordered = (0..4)
        .flat_map(|lane| (lane..K).step_by(4))
        .map(|ordinal| elements[ordinal].clone())
        .collect();
    Tensor::dense(
        F32::resolved_type(),
        Shape::from_dims([K as u64]),
        reordered,
    )
    .unwrap()
}

fn bits(value: &Tensor) -> u32 {
    let TensorPayloadView::Dense(elements) = value.payload() else {
        panic!("the oracle result is dense")
    };
    let [value] = elements else {
        panic!("one reduced result is expected")
    };
    u32::from_be_bytes(value.as_bytes().try_into().unwrap())
}

fn split_result(products: &Tensor) -> u32 {
    let reduced = strict_partitioned_sum_under(products, &[Axis::new(0)], 4, 4, ftz()).unwrap();
    bits(&reduced)
}

#[test]
fn the_retained_eight_case_oracle_matches_both_fixed_memberships() {
    const MEMBERSHIPS: [ContributorMembership;
        core::mem::variant_count::<ContributorMembership>()] = [
        ContributorMembership::Contiguous,
        ContributorMembership::LaneStrided,
    ];
    assert_eq!(CASES.len(), 8);
    for case in CASES {
        let products = products(case);
        let actual = MEMBERSHIPS.map(|membership| match membership {
            ContributorMembership::Contiguous => split_result(&products),
            ContributorMembership::LaneStrided => split_result(&lane_strided(&products)),
        });
        assert_eq!(actual, case.expected, "{}", case.name);
    }
}

#![feature(variant_count)]

//! The exceptional-value corpus for `tiler::strict-tensor-contraction-f32@1`,
//! through the public semantic and reference boundary.
//!
//! # Where the numbers come from
//!
//! Every operand vector and every expectation is transcribed from the retained
//! Metal contraction spike, not re-derived here:
//!
//! - operands: `spikes/scheduling/metal_contraction_vertical/contraction_probe.py`,
//!   `semantic_cases()`;
//! - expectations: the `strict_fold` rows of
//!   `spikes/scheduling/metal_contraction_vertical/results/2026-07-31-correctness-apple9-f32-msl4-macos26-m4max-metal32023.883/semantics-candidates.tsv`.
//!
//! Transcribing rather than recomputing is the point. A helper that re-derived
//! the expectation from the same fold the evaluator runs would agree with it for
//! reasons that say nothing about either being right — the independence rule the
//! two oracles in this crate are built on. The spike's values were produced by an
//! exact rational model in a different language and retained as evidence, so
//! agreeing with them is a claim about the *contract*, reproduced across two
//! independent implementations.
//!
//! The probe drives `16 x 16 x 16` so that every realization's structural
//! precondition is satisfied by one dispatch, and only `C[0, 0]` carries the
//! designed dot — every other operand entry is `+0.0`. The cases below drive
//! `[1, 16] x [1, 16] -> [1, 1]`, which is that same cell's products and that
//! same fold with the unpopulated rows dropped.
//!
//! # Which conformance level these results claim
//!
//! Exactly one, stated so a reader knows what a pass buys. The evaluator computes
//! under [`tiler_reference::ReferenceNumericalConformance::strict`]: both
//! subnormal dimensions preserved, separately rounded multiply and add, a strict
//! left fold in the declared contributor order, and bit-preserving signed zeros.
//! Its results are therefore *the* value the declared contract names, not a
//! member of an admitted set — which is only meaningful because the declared
//! signature forbids fusion, reassociation, and permutation. Under a contract
//! permitting any of those the answer would be a result set and no single value
//! would be the reference; the reference refuses such a declaration rather than
//! evaluating one reading of it.
//!
//! **The `+ftz` column of the spike's TSV is not this contract.** Flushing
//! subnormals is a property of the qualified Apple9/F32 target row, not of the
//! operation, which is why `subnormal_product` below expects `0x00400000` and not
//! the `0x00000000` that row produces. A device comparison against this oracle is
//! a comparison against the strict reading, and the flushing dimension has to be
//! declared on the comparison rather than absorbed here.
//!
//! **What a pass here is not.** It is evidence about the semantic contract and
//! the host reference evaluator. It is not evidence about any schedule, any
//! lowering, any compiled kernel, any device, or any model-level tolerance; no
//! such thing is exercised. Nor is it a universal claim: this is an exhaustive
//! pass over eight named exceptional cases, not a proof over the binary32 domain.
//!
//! # Coverage against the general reduction checklist
//!
//! [`REDUCTION_CONTRACT_LEDGER`] maps every subject under `Required adversarial
//! tests` in the governing reduction research record into exactly one current
//! classification. The decomposition is intentionally finer than its prose:
//! for example, the retained qNaN case exercises one payload in one contributor
//! position, so it does not discharge “qNaN in every contributor position” or
//! “several NaN payloads.” The ledger keeps both rows admitted and uncovered.
//!
//! This ledger is target-independent. The compiler's host-side comparison of
//! selected workload cells, the six retained `direct` digests, and the Apple
//! live-device envelope remain separate realization evidence. None can turn a
//! target-independent uncovered row below into a covered one.

use tiler_ir::semantic::{
    ContractionIndex, ContractionIndexStructure, F32, F32TensorContraction, InputKey, OutputKey,
    SemanticProgramBuilder,
};
use tiler_ir::shape::Shape;
use tiler_reference::{
    FloatBitOrder, InputBinding, ReferenceElement, ReferenceEvaluator, Tensor, TensorPayloadView,
};

/// The extent of the probe's contracted axis.
const K: usize = 16;

/// `td,od->to` over `[M, K] x [N, K] -> [M, N]` — the admitted index structure.
///
/// The contracted index is the *last* axis of both operands, because the pinned
/// workload's checkpoint stores every projection weight `[out, in]`. The ordinary
/// `[M, K] x [K, N]` spelling is a different structure.
fn td_od_to() -> ContractionIndexStructure {
    let index = ContractionIndex::new;
    ContractionIndexStructure::new(
        [vec![index(0), index(1)], vec![index(2), index(1)]],
        [index(0), index(2)],
    )
    .expect("`td,od->to` is admitted")
}

fn tensor(dims: [u64; 2], bits: &[u32]) -> Tensor {
    let shape = Shape::from_dims(dims);
    assert_eq!(
        shape.element_count(),
        Some(bits.len()),
        "a fixture states every element"
    );
    Tensor::dense(
        F32::resolved_type(),
        shape,
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
fn contract(left: &Tensor, right: &Tensor) -> Vec<u32> {
    let structure = td_od_to();
    let mut builder = SemanticProgramBuilder::try_standard().expect("the standard builder opens");
    let left_key = InputKey::new("left").expect("a valid key");
    let right_key = InputKey::new("right").expect("a valid key");
    let left_value = builder
        .input::<F32>(left_key.clone(), left.shape().clone())
        .expect("an F32 input");
    let right_value = builder
        .input::<F32>(right_key.clone(), right.shape().clone())
        .expect("an F32 input");
    let result = F32TensorContraction::apply(&mut builder, &structure, left_value, right_value)
        .expect("a covered occurrence is admitted");
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

/// Evaluates one `K`-wide dot as the single cell of a `[1, 1]` result.
fn dot(left: &[u32; K], right: &[u32; K]) -> u32 {
    let bits = contract(&tensor([1, K as u64], left), &tensor([1, K as u64], right));
    let [value] = bits.as_slice() else {
        panic!("a `[1, 1]` result has one element");
    };
    *value
}

/// Pads a designed prefix with the `+0.0` the probe's `pad` supplies.
fn pad(prefix: &[u32]) -> [u32; K] {
    let mut padded = [0x0000_0000_u32; K];
    padded[..prefix.len()].copy_from_slice(prefix);
    padded
}

/// One atomic subject derived from the governing `Required adversarial tests`.
///
/// Broad phrases are split wherever this admitted profile gives their members
/// different answers. In particular, rank zero is outside a contraction (which
/// must contract at least one index) while the admitted positive ranks remain a
/// wider unexercised population; the F16/BF16 and verifier clauses are split for
/// the same reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReductionContractSubject {
    SupportedCellPositiveAndNegative,
    RankZero,
    EveryAdmittedPositiveRank,
    FirstContractedAxis,
    MiddleContractedAxis,
    LastContractedAxis,
    MultipleContractedAxes,
    AllAxesContracted,
    DuplicateAxes,
    OutOfRangeAxes,
    DynamicallyBoundAxes,
    ZeroReducedExtent,
    ZeroSurvivingExtent,
    NoSeed,
    NeutralSeed,
    NonNeutralSeed,
    RuntimeSeed,
    SeedConversionHalfway,
    SeedConversionOverflow,
    SingletonPositiveZero,
    SingletonNegativeZero,
    PositiveThenNegativeZero,
    NegativeThenPositiveZero,
    Subnormals,
    Infinities,
    QuietNanEveryContributorPosition,
    SignallingNanEveryContributorPosition,
    SeveralNanPayloads,
    ThreeElementReassociationWitness,
    ThreeElementPermutationWitness,
    SerialTree,
    BalancedTree,
    SkewedTree,
    SimdTree,
    ThreadgroupTree,
    ContiguousMultiPassTree,
    NoncontiguousLaneTree,
    AtomicArrivalTree,
    MaskedEmptyPartials,
    HasValueEmptyPartials,
    InvalidReplicatedEmptyValues,
    InvalidReplicatedSeeds,
    IntegerWrappingBoundary,
    IntegerSaturatingBoundary,
    IntegerCheckedBoundary,
    IntegerWideningBoundary,
    F16InputF32AccumulatorSameResult,
    F16InputF32AccumulatorNarrowerResult,
    Bf16InputF32AccumulatorSameResult,
    Bf16InputF32AccumulatorNarrowerResult,
    ScratchNormalRoundTrip,
    ScratchSubnormalRoundTrip,
    ScratchNanCanonicalization,
    RepeatedPlanIdentityExecution,
    MissingPermissionRejection,
    MissingAlgebraicCapabilityRejection,
    MissingTargetCapabilityRejection,
    MissingNonemptyProofRejection,
    MissingLosslessScratchRejection,
}

impl ReductionContractSubject {
    /// The complete typed population, sized from the type rather than by hand.
    const ALL: [Self; core::mem::variant_count::<Self>()] = [
        Self::SupportedCellPositiveAndNegative,
        Self::RankZero,
        Self::EveryAdmittedPositiveRank,
        Self::FirstContractedAxis,
        Self::MiddleContractedAxis,
        Self::LastContractedAxis,
        Self::MultipleContractedAxes,
        Self::AllAxesContracted,
        Self::DuplicateAxes,
        Self::OutOfRangeAxes,
        Self::DynamicallyBoundAxes,
        Self::ZeroReducedExtent,
        Self::ZeroSurvivingExtent,
        Self::NoSeed,
        Self::NeutralSeed,
        Self::NonNeutralSeed,
        Self::RuntimeSeed,
        Self::SeedConversionHalfway,
        Self::SeedConversionOverflow,
        Self::SingletonPositiveZero,
        Self::SingletonNegativeZero,
        Self::PositiveThenNegativeZero,
        Self::NegativeThenPositiveZero,
        Self::Subnormals,
        Self::Infinities,
        Self::QuietNanEveryContributorPosition,
        Self::SignallingNanEveryContributorPosition,
        Self::SeveralNanPayloads,
        Self::ThreeElementReassociationWitness,
        Self::ThreeElementPermutationWitness,
        Self::SerialTree,
        Self::BalancedTree,
        Self::SkewedTree,
        Self::SimdTree,
        Self::ThreadgroupTree,
        Self::ContiguousMultiPassTree,
        Self::NoncontiguousLaneTree,
        Self::AtomicArrivalTree,
        Self::MaskedEmptyPartials,
        Self::HasValueEmptyPartials,
        Self::InvalidReplicatedEmptyValues,
        Self::InvalidReplicatedSeeds,
        Self::IntegerWrappingBoundary,
        Self::IntegerSaturatingBoundary,
        Self::IntegerCheckedBoundary,
        Self::IntegerWideningBoundary,
        Self::F16InputF32AccumulatorSameResult,
        Self::F16InputF32AccumulatorNarrowerResult,
        Self::Bf16InputF32AccumulatorSameResult,
        Self::Bf16InputF32AccumulatorNarrowerResult,
        Self::ScratchNormalRoundTrip,
        Self::ScratchSubnormalRoundTrip,
        Self::ScratchNanCanonicalization,
        Self::RepeatedPlanIdentityExecution,
        Self::MissingPermissionRejection,
        Self::MissingAlgebraicCapabilityRejection,
        Self::MissingTargetCapabilityRejection,
        Self::MissingNonemptyProofRejection,
        Self::MissingLosslessScratchRejection,
    ];

    /// The retained test relationship independently expected for covered rows.
    const fn expected_exact_test(self) -> Option<&'static str> {
        match self {
            Self::LastContractedAxis => Some("the_execution_witness_is_exactly_six"),
            Self::NoSeed => {
                Some("the_accumulator_is_seeded_from_the_first_product_and_not_from_positive_zero")
            }
            Self::Subnormals => Some("a_subnormal_product_is_preserved_rather_than_flushed"),
            Self::SerialTree => Some("the_searched_split_separator_reproduces_the_strict_value"),
            _ => None,
        }
    }
}

/// A named ordinary test outside the eight retained exact-bit cases.
#[derive(Clone, Copy)]
struct OrdinaryTest {
    source: OrdinaryTestSource,
    name: &'static str,
}

/// The target-independent source files the ledger may cite.
///
/// A device or digest source is deliberately unrepresentable here. That keeps
/// target realization evidence from silently satisfying a reference row.
#[derive(Clone, Copy)]
enum OrdinaryTestSource {
    ReferenceContractionUnit,
    SemanticContraction,
    AttentionContractionStructures,
}

impl OrdinaryTestSource {
    const fn path(self) -> &'static str {
        match self {
            Self::ReferenceContractionUnit => "crates/tiler-reference/src/contraction/tests.rs",
            Self::SemanticContraction => "crates/tiler-ir/src/semantic/contraction/tests.rs",
            Self::AttentionContractionStructures => {
                "crates/tiler-reference/tests/attention_contraction_structures.rs"
            }
        }
    }

    const fn source(self) -> &'static str {
        match self {
            Self::ReferenceContractionUnit => include_str!("../src/contraction/tests.rs"),
            Self::SemanticContraction => {
                include_str!("../../tiler-ir/src/semantic/contraction/tests.rs")
            }
            Self::AttentionContractionStructures => {
                include_str!("attention_contraction_structures.rs")
            }
        }
    }
}

/// Exactly one evidence classification for one checklist subject.
enum Coverage {
    /// Exercised by one of this file's eight exact-bit retained cases.
    ExactBit {
        test_name: &'static str,
        run: fn(),
        scope: &'static str,
    },
    /// Exercised by another named target-independent ordinary test.
    Ordinary(&'static [OrdinaryTest]),
    /// Not a constructible member of the admitted strict F32 contraction profile.
    Outside(&'static str),
    /// Constructible or required by the admitted profile, but not fully covered.
    AdmittedUncovered(&'static str),
}

struct LedgerEntry {
    subject: ReductionContractSubject,
    coverage: Coverage,
}

const fn exact_bit(
    subject: ReductionContractSubject,
    test_name: &'static str,
    run: fn(),
    scope: &'static str,
) -> LedgerEntry {
    LedgerEntry {
        subject,
        coverage: Coverage::ExactBit {
            test_name,
            run,
            scope,
        },
    }
}

const REFERENCE_UNIT: OrdinaryTestSource = OrdinaryTestSource::ReferenceContractionUnit;
const SEMANTIC: OrdinaryTestSource = OrdinaryTestSource::SemanticContraction;
const ATTENTION_STRUCTURES: OrdinaryTestSource = OrdinaryTestSource::AttentionContractionStructures;

/// Current coverage of every atomic subject in `Required adversarial tests`.
///
/// This is a slice so deleting one row reaches the runtime census and reports
/// the missing typed subject by name. [`ReductionContractSubject::ALL`] is the
/// independently type-sized population against which it is checked.
const REDUCTION_CONTRACT_LEDGER: &[LedgerEntry] = &[
    LedgerEntry {
        subject: ReductionContractSubject::SupportedCellPositiveAndNegative,
        coverage: Coverage::Ordinary(&[
            OrdinaryTest {
                source: REFERENCE_UNIT,
                name: "the_governed_signature_decodes_to_the_unseeded_binary32_fold",
            },
            OrdinaryTest {
                source: REFERENCE_UNIT,
                name: "a_declaration_this_reference_does_not_compute_is_refused_by_field",
            },
        ]),
    },
    LedgerEntry {
        subject: ReductionContractSubject::RankZero,
        coverage: Coverage::Outside(
            "a contraction must name at least one contracted index appearing in both operands",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::EveryAdmittedPositiveRank,
        coverage: Coverage::AdmittedUncovered(
            "ordinary tests exercise selected ranks, not every positive rank through the structural bound",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::FirstContractedAxis,
        coverage: Coverage::AdmittedUncovered(
            "the retained structure contracts the last axis of both operands",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::MiddleContractedAxis,
        coverage: Coverage::Ordinary(&[OrdinaryTest {
            source: ATTENTION_STRUCTURES,
            name: "the_value_structure_denotes_repeat_then_matmul_bit_for_bit",
        }]),
    },
    exact_bit(
        ReductionContractSubject::LastContractedAxis,
        "the_execution_witness_is_exactly_six",
        the_execution_witness_is_exactly_six,
        "the retained td,od->to structure contracts the last axis of both operands",
    ),
    LedgerEntry {
        subject: ReductionContractSubject::MultipleContractedAxes,
        coverage: Coverage::AdmittedUncovered(
            "all retained exact-bit cases have one contracted index",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::AllAxesContracted,
        coverage: Coverage::AdmittedUncovered(
            "all retained exact-bit cases preserve one free axis from each operand",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::DuplicateAxes,
        coverage: Coverage::Outside(
            "contraction has typed index tuples rather than a reduction-axis list",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::OutOfRangeAxes,
        coverage: Coverage::Outside(
            "contraction indices are labels bound by tuples, not numeric axis selectors",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::DynamicallyBoundAxes,
        coverage: Coverage::Outside(
            "this contraction family accepts only static operand shapes and index bindings",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::ZeroReducedExtent,
        coverage: Coverage::Ordinary(&[OrdinaryTest {
            source: SEMANTIC,
            name: "an_empty_contracted_domain_is_refused_because_the_fold_is_unseeded",
        }]),
    },
    LedgerEntry {
        subject: ReductionContractSubject::ZeroSurvivingExtent,
        coverage: Coverage::Ordinary(&[OrdinaryTest {
            source: SEMANTIC,
            name: "an_empty_contracted_domain_is_refused_because_the_fold_is_unseeded",
        }]),
    },
    exact_bit(
        ReductionContractSubject::NoSeed,
        "the_accumulator_is_seeded_from_the_first_product_and_not_from_positive_zero",
        the_accumulator_is_seeded_from_the_first_product_and_not_from_positive_zero,
        "one sixteen-contributor negative-zero vector distinguishes first-product from positive-zero seeding",
    ),
    LedgerEntry {
        subject: ReductionContractSubject::NeutralSeed,
        coverage: Coverage::Outside("the registered contraction declares no explicit seed"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::NonNeutralSeed,
        coverage: Coverage::Outside("the registered contraction declares no explicit seed"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::RuntimeSeed,
        coverage: Coverage::Outside("the registered contraction declares no explicit seed"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::SeedConversionHalfway,
        coverage: Coverage::Outside(
            "there is no seed and every admitted arithmetic role is already F32",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::SeedConversionOverflow,
        coverage: Coverage::Outside(
            "there is no seed and every admitted arithmetic role is already F32",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::SingletonPositiveZero,
        coverage: Coverage::AdmittedUncovered(
            "the signed-zero retained case folds sixteen negative-zero products, not one positive zero",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::SingletonNegativeZero,
        coverage: Coverage::AdmittedUncovered(
            "the signed-zero retained case folds sixteen products rather than a singleton",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::PositiveThenNegativeZero,
        coverage: Coverage::AdmittedUncovered(
            "no retained case folds the two zero signs in this order",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::NegativeThenPositiveZero,
        coverage: Coverage::Ordinary(&[OrdinaryTest {
            source: ATTENTION_STRUCTURES,
            name: "a_masked_position_contributes_a_signed_zero_to_the_value_contraction",
        }]),
    },
    exact_bit(
        ReductionContractSubject::Subnormals,
        "a_subnormal_product_is_preserved_rather_than_flushed",
        a_subnormal_product_is_preserved_rather_than_flushed,
        "one positive subnormal product is preserved; no other sign, boundary, or position is claimed",
    ),
    LedgerEntry {
        subject: ReductionContractSubject::Infinities,
        coverage: Coverage::AdmittedUncovered(
            "the retained infinity operand is multiplied by zero and forms a NaN product before the fold; no infinity contributor is exercised",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::QuietNanEveryContributorPosition,
        coverage: Coverage::AdmittedUncovered(
            "one retained qNaN payload appears in the first position only",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::SignallingNanEveryContributorPosition,
        coverage: Coverage::AdmittedUncovered("no retained case supplies an sNaN contributor"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::SeveralNanPayloads,
        coverage: Coverage::AdmittedUncovered(
            "one noncanonical input payload does not satisfy several payloads",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::ThreeElementReassociationWitness,
        coverage: Coverage::AdmittedUncovered(
            "the retained order discriminators use longer padded sequences, not three elements",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::ThreeElementPermutationWitness,
        coverage: Coverage::AdmittedUncovered(
            "the retained reversed-order checks use longer padded sequences, not three elements",
        ),
    },
    exact_bit(
        ReductionContractSubject::SerialTree,
        "the_searched_split_separator_reproduces_the_strict_value",
        the_searched_split_separator_reproduces_the_strict_value,
        "the retained separator distinguishes the strict serial left fold from both two-part split topologies",
    ),
    LedgerEntry {
        subject: ReductionContractSubject::BalancedTree,
        coverage: Coverage::Outside("the operation permits no reassociation"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::SkewedTree,
        coverage: Coverage::AdmittedUncovered(
            "the canonical left fold is maximally skewed, but the governing list names serial and skewed separately and no distinct skewed-family test is identified",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::SimdTree,
        coverage: Coverage::Outside("the operation permits no reassociation or permutation"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::ThreadgroupTree,
        coverage: Coverage::Outside("the operation permits no reassociation or permutation"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::ContiguousMultiPassTree,
        coverage: Coverage::Outside("the operation permits no reassociation"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::NoncontiguousLaneTree,
        coverage: Coverage::Outside("the operation permits neither reassociation nor permutation"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::AtomicArrivalTree,
        coverage: Coverage::Outside(
            "timing-dependent arrival is outside strict order and plan determinism",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::MaskedEmptyPartials,
        coverage: Coverage::Outside("the admitted serial fold forms no parallel partials or masks"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::HasValueEmptyPartials,
        coverage: Coverage::Outside("the admitted serial fold forms no parallel partial state"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::InvalidReplicatedEmptyValues,
        coverage: Coverage::Outside("the admitted serial fold injects no empty value"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::InvalidReplicatedSeeds,
        coverage: Coverage::Outside("the admitted fold is unseeded and forms no partials"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::IntegerWrappingBoundary,
        coverage: Coverage::Outside("the registered contraction is F32 throughout"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::IntegerSaturatingBoundary,
        coverage: Coverage::Outside("the registered contraction is F32 throughout"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::IntegerCheckedBoundary,
        coverage: Coverage::Outside("the registered contraction is F32 throughout"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::IntegerWideningBoundary,
        coverage: Coverage::Outside("the registered contraction is F32 throughout"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::F16InputF32AccumulatorSameResult,
        coverage: Coverage::Outside("the registered contraction accepts only F32 operands"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::F16InputF32AccumulatorNarrowerResult,
        coverage: Coverage::Outside("the registered contraction accepts and returns only F32"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::Bf16InputF32AccumulatorSameResult,
        coverage: Coverage::Outside("the registered contraction accepts only F32 operands"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::Bf16InputF32AccumulatorNarrowerResult,
        coverage: Coverage::Outside("the registered contraction accepts and returns only F32"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::ScratchNormalRoundTrip,
        coverage: Coverage::Outside("the admitted serial reference uses no scratch boundary"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::ScratchSubnormalRoundTrip,
        coverage: Coverage::Outside("the admitted serial reference uses no scratch boundary"),
    },
    LedgerEntry {
        subject: ReductionContractSubject::ScratchNanCanonicalization,
        coverage: Coverage::Outside(
            "the retained NaN cases exercise arithmetic canonicalization, not a scratch round trip",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::RepeatedPlanIdentityExecution,
        coverage: Coverage::AdmittedUncovered(
            "the operation declares plan determinism, but this target-independent corpus has no artifact, variant, or target identity",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::MissingPermissionRejection,
        coverage: Coverage::AdmittedUncovered(
            "no target-independent contraction test proposes a tree requiring a forbidden permission",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::MissingAlgebraicCapabilityRejection,
        coverage: Coverage::AdmittedUncovered(
            "the operation declares no algebraic capability, but this corpus does not drive a consuming rule's refusal",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::MissingTargetCapabilityRejection,
        coverage: Coverage::AdmittedUncovered(
            "target capability is outside the reference oracle and no target-independent relationship is claimed here",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::MissingNonemptyProofRejection,
        coverage: Coverage::AdmittedUncovered(
            "semantic construction refuses an empty contracted extent, but no verifier test names a missing nonempty proof",
        ),
    },
    LedgerEntry {
        subject: ReductionContractSubject::MissingLosslessScratchRejection,
        coverage: Coverage::Outside("the admitted serial reference has no scratch contract"),
    },
];

// --- the eight cases --------------------------------------------------------

/// Execution witness: every topology agrees, so a failure here rules out the
/// whole column rather than discriminating within it.
#[test]
fn the_execution_witness_is_exactly_six() {
    assert_eq!(dot(&pad(&[0x4000_0000]), &pad(&[0x4040_0000])), 0x40c0_0000);
}

/// Order absorption: a `2^24` leading contributor absorbs `1.5` increments under
/// a left fold and does not under a wider or regrouped accumulation.
///
/// The retained candidates separate here: `strict_fold` and `zero_seed_fold` give
/// `0x4b800007`, the splits give `0x4b800006`, and the exact-then-round and
/// reversed topologies give `0x4b800005`. Reproducing the first is what says the
/// evaluator folds left in ascending order rather than accumulating widely.
#[test]
fn a_leading_large_contributor_absorbs_under_the_ascending_left_fold() {
    let left = pad(&[
        0x4b80_0000,
        0x3fc0_0000,
        0x3fc0_0000,
        0x3fc0_0000,
        0x3fc0_0000,
        0x3fc0_0000,
        0x3fc0_0000,
        0x3fc0_0000,
    ]);
    let right = pad(&[0x3f80_0000; 8]);
    assert_eq!(dot(&left, &right), 0x4b80_0007);

    // The perturbation, and a second retained candidate value: over the reversed
    // contributor sequence the same products fold to the spike's `reversed_fold`
    // value. Reversal is a permutation, which this family forbids, so a
    // permuting evaluator would return this instead — and it does not.
    let mut reversed_left = left;
    reversed_left.reverse();
    let mut reversed_right = right;
    reversed_right.reverse();
    assert_eq!(dot(&reversed_left, &reversed_right), 0x4b80_0005);
}

/// The separately-rounded-against-fused discriminator, in ADR 0015's sense of
/// contraction: the accumulator already holds `1.0` when an inexact product
/// arrives, which is the step a fused multiply-add would collapse to one
/// rounding.
///
/// `strict_fold` gives `0x3fc58f9e` and every fused candidate gives `0x3fc58f9d`.
/// With the inexact product first the two coincide and the case would report a
/// fusion-free evaluator whatever it did, so the order is load-bearing.
#[test]
fn a_separately_rounded_product_and_add_is_not_the_fused_value() {
    assert_eq!(
        dot(
            &pad(&[0x3f80_0000, 0x3eb9_7ef9]),
            &pad(&[0x3f80_0000, 0x3fc0_0000])
        ),
        0x3fc5_8f9e
    );
}

/// The searched vector separating strict, contiguous-split, strided-split, fused,
/// and infinitely wide accumulation — every one of those pairwise distinct.
///
/// This is the case that separates the contiguous split (`0xbb1d0683`) from the
/// strided one (`0xbb1d0672`); `order_absorption` does not.
#[test]
fn the_searched_split_separator_reproduces_the_strict_value() {
    assert_eq!(dot(&SPLIT_LEFT, &SPLIT_RIGHT), 0xbb1d_0482);

    // The reversed sequence, which is the spike's `reversed_fold` value here.
    let mut reversed_left = SPLIT_LEFT;
    reversed_left.reverse();
    let mut reversed_right = SPLIT_RIGHT;
    reversed_right.reverse();
    assert_eq!(dot(&reversed_left, &reversed_right), 0xbb1d_0494);
}

/// The signed-zero accumulator seed, where the seed is the only difference.
///
/// Every product is `-0.0`. A fold seeded from the first product returns
/// `0x80000000`; one seeded at `+0.0` returns `0x00000000`, because
/// `fl(+0.0 + -0.0)` is `+0.0`. The idiomatic accumulator-starts-at-zero loop
/// computes the second value, and this is the only case in the corpus that
/// notices.
#[test]
fn the_accumulator_is_seeded_from_the_first_product_and_not_from_positive_zero() {
    assert_eq!(
        dot(&[0xbf80_0000; K], &[0x0000_0000; K]),
        0x8000_0000,
        "a `+0.0`-seeded fold returns 0x00000000 here"
    );
}

/// A non-canonical quiet NaN payload entering as a contributor.
///
/// Only the payload-propagating topology returns `0x7fc0dead`; every
/// canonicalizing one returns the governed `0x7fc00000`. The operand itself is
/// *read* unchanged — canonicalization applies to a produced value — so the
/// canonical result is evidence that the multiply's result was canonicalized
/// rather than that the operand was rewritten.
#[test]
fn a_non_canonical_nan_payload_is_replaced_by_the_governed_one() {
    assert_eq!(
        dot(
            &pad(&[0x7fc0_dead, 0x3f80_0000]),
            &pad(&[0x3f80_0000, 0x3f80_0000])
        ),
        0x7fc0_0000
    );
}

/// `inf * 0` formed *inside* the reduction rather than handed to it.
///
/// The distinction matters: an evaluator that only canonicalized NaN operands
/// would pass the payload case above and fail this one, because here no operand
/// is a NaN and the NaN is produced by the multiply.
#[test]
fn a_nan_the_reduction_forms_itself_is_canonicalized_too() {
    assert_eq!(
        dot(
            &pad(&[0x7f80_0000, 0x3f80_0000]),
            &pad(&[0x0000_0000, 0x3f80_0000])
        ),
        0x7fc0_0000
    );
}

/// A subnormal product, `2^-126 * 0.5 = 2^-127`.
///
/// Preserved, because the reference computes the strict reading. The measured
/// Apple9/F32 row flushes it to `0x00000000` under every math mode; that is a
/// target realization and is deliberately not this operation's contract.
#[test]
fn a_subnormal_product_is_preserved_rather_than_flushed() {
    assert_eq!(dot(&pad(&[0x0080_0000]), &pad(&[0x3f00_0000])), 0x0040_0000);
}

// --- the structure, beyond one cell -----------------------------------------

/// The fold is per output coordinate, over the structure's own index bindings.
///
/// Every case above reads `C[0, 0]`, which an evaluator that ignored the output
/// coordinate entirely would still pass. This drives `[2, 16] x [2, 16] -> [2, 2]`
/// with the witness in row zero and the split separator in row one, so the two
/// diagonal cells must reproduce two different retained values and the two
/// off-diagonal cells must equal neither.
#[test]
fn each_output_cell_folds_its_own_contributor_pair() {
    let mut left = [0x0000_0000_u32; 2 * K];
    let mut right = [0x0000_0000_u32; 2 * K];
    left[..K].copy_from_slice(&pad(&[0x4000_0000]));
    right[..K].copy_from_slice(&pad(&[0x4040_0000]));
    left[K..].copy_from_slice(&SPLIT_LEFT);
    right[K..].copy_from_slice(&SPLIT_RIGHT);

    let result = contract(
        &tensor([2, K as u64], &left),
        &tensor([2, K as u64], &right),
    );
    let [zero_zero, zero_one, one_zero, one_one] = result.as_slice() else {
        panic!("a `[2, 2]` result has four elements");
    };
    assert_eq!(*zero_zero, 0x40c0_0000, "row 0 against column 0 is 2 * 3");
    assert_eq!(
        *one_one, 0xbb1d_0482,
        "row 1 against column 1 is the split separator"
    );
    for cross in [zero_one, one_zero] {
        assert_ne!(*cross, 0x40c0_0000);
        assert_ne!(*cross, 0xbb1d_0482);
    }
}

/// Every governing checklist subject has exactly one current classification.
///
/// This check reaches three independently removable things:
///
/// - a ledger row, through the typed full-population census;
/// - an exact-bit relationship, through both its independently expected name and
///   its function pointer, which executes the named test body; and
/// - an ordinary-test relationship, through the named `#[test] fn` in the
///   included target-independent source.
///
/// The first failure names the missing typed subject. The latter two name both
/// subject and relationship, so deleting evidence cannot degrade into an
/// unexplained count mismatch.
#[test]
fn every_reduction_contract_subject_has_one_live_evidence_classification() {
    for subject in ReductionContractSubject::ALL {
        let mut entries = REDUCTION_CONTRACT_LEDGER
            .iter()
            .filter(|entry| entry.subject == subject);
        let Some(entry) = entries.next() else {
            panic!("reduction-contract subject {subject:?} has no ledger entry");
        };
        assert!(
            entries.next().is_none(),
            "reduction-contract subject {subject:?} has more than one ledger entry"
        );

        match &entry.coverage {
            Coverage::ExactBit {
                test_name,
                run,
                scope,
            } => {
                assert_eq!(
                    subject.expected_exact_test(),
                    Some(*test_name),
                    "reduction-contract subject {subject:?} names the wrong exact-bit test relationship"
                );
                assert!(
                    !scope.is_empty(),
                    "reduction-contract subject {subject:?} has an unbounded exact-bit classification"
                );
                run();
            }
            Coverage::Ordinary(relationships) => {
                assert!(
                    subject.expected_exact_test().is_none(),
                    "reduction-contract subject {subject:?} lost its required exact-bit relationship"
                );
                assert!(
                    !relationships.is_empty(),
                    "reduction-contract subject {subject:?} names no ordinary test relationship"
                );
                for relationship in *relationships {
                    let function = format!("fn {}(", relationship.name);
                    let source = relationship.source.source();
                    let Some(position) = source.find(&function) else {
                        panic!(
                            "reduction-contract subject {subject:?} names missing ordinary test {} in {}",
                            relationship.name,
                            relationship.source.path(),
                        );
                    };
                    assert!(
                        source[..position].trim_end().ends_with("#[test]"),
                        "reduction-contract subject {subject:?} relationship {} in {} is not an ordinary #[test]",
                        relationship.name,
                        relationship.source.path(),
                    );
                }
            }
            Coverage::Outside(reason) | Coverage::AdmittedUncovered(reason) => {
                assert!(
                    subject.expected_exact_test().is_none(),
                    "reduction-contract subject {subject:?} lost its required exact-bit relationship"
                );
                assert!(
                    !reason.is_empty(),
                    "reduction-contract subject {subject:?} has an unexplained classification"
                );
            }
        }
    }
}

/// The left operand of the searched split separator, exactly as the probe states it.
const SPLIT_LEFT: [u32; K] = [
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
];

/// The right operand of the searched split separator.
const SPLIT_RIGHT: [u32; K] = [
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
];

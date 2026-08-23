use super::super::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind, ContractionAxisSource,
    KernelSchedule, LogicalAccess, OwnershipProof, OwnershipProofKind, ReductionTopology, RegionId,
    RegionProgram, ScalarProgram, ScheduledRegionBuilder, ScheduledRegionDiagnostic, TailPolicy,
    TensorRole, cooperative_tile, element_count,
};
use super::support::{
    contraction_builder, linear_schedule, reassociating_numerical, strict_numerical,
};
use super::support_contraction::{
    CONTRACTED_EXTENT, CONTRACTED_TILE, OUTPUT_BLOCK, OUTPUT_EXTENT, OUTPUT_POSITIONS,
    TILE_PARTICIPANTS, admitted_operand_tile, operand_contraction_builder, operand_tile_fixture,
};
use crate::schedule::handles::{BoundsWitnessId, OwnershipWitnessId};
use crate::schedule::model::{ContributorOrder, LaunchPlan};
use crate::schedule::numerics::ArithmeticType;
use crate::shape::{Axis, Shape};
use std::fmt::Write as _;

/// Contraction inputs are distinguished by exact access position, not role payload.
#[test]
fn contraction_inputs_are_distinguished_by_access_position() {
    let verified = contraction_builder().build().unwrap();
    assert_eq!(
        verified.region().index.accesses[0].tensor,
        TensorRole::Input
    );
    assert_eq!(
        verified.region().index.accesses[1].tensor,
        TensorRole::Input
    );
}

fn live_contraction_builder(
    live_access: u32,
    live_axis: u32,
    output: [u64; 2],
) -> ScheduledRegionBuilder {
    let left_shape = Shape::from_dims([output[0]]);
    let right_shape = Shape::from_dims([output[1]]);
    let output_shape = Shape::from_dims(output);
    let contracted = Shape::from_dims([]);
    let left = TensorRole::Input;
    let right = TensorRole::Input;
    let owner = OwnershipWitnessId::new(0);
    let output_elements = element_count(&output_shape).unwrap_or(0);
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(42));
    builder.iteration_shape(output_shape.clone()).unwrap();
    for (witness, tensor, operand, free) in
        [(0, left, left_shape, 0_u32), (1, right, right_shape, 1)]
    {
        builder
            .push_access(Access {
                tensor,
                component_role: None,
                mode: AccessMode::Read,
                map: LogicalAccess::ContractionOperand {
                    operand_shape: operand,
                    output_shape: output_shape.clone(),
                    contracted_shape: contracted.clone(),
                    sources: vec![ContractionAxisSource::Output { position: free }],
                    order: ContributorOrder::OriginalAxisLexicographic,
                },
                bounds: BoundsWitnessId::new(witness),
                ownership: None,
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor,
                component_role: None,
                kind: BoundsProofKind::LiveExtentReach,
            })
            .unwrap();
    }
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(2),
            ownership: Some(owner),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: output_elements,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: owner,
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: output_elements,
            },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted,
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
            },
            numerical: strict_numerical(),
        })
        .unwrap();
    builder
        .schedule(KernelSchedule {
            reduction: ReductionTopology::LiveContraction {
                live_access: AccessOrdinal::new(live_access),
                live_axis: Axis::new(live_axis),
                order: ContributorOrder::OriginalAxisLexicographic,
                permits_reassociation: false,
                permits_permutation: false,
            },
            ..linear_schedule(output_elements, owner)
        })
        .unwrap();
    builder
}

/// A well-formed live contraction is schedule-verified; a swapped live
/// axis is not.
#[test]
fn a_live_contraction_admits_the_named_inner_axis_and_refuses_a_swapped_symbol() {
    let verified = live_contraction_builder(0, 1, [2, 3])
        .build()
        .expect("the named inner axis of input 0 is the live contracted bound");
    assert!(matches!(
        verified.region().schedule.reduction,
        ReductionTopology::LiveContraction {
            live_axis,
            ..
        } if live_axis == Axis::new(1)
    ));

    let swapped = live_contraction_builder(0, 0, [2, 3])
        .build()
        .expect_err("naming the free axis as the live bound must fail");
    assert_eq!(
        swapped.diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "swapped-symbol live axis: {swapped}"
    );
    assert_eq!(
        swapped.diagnostics()[0].rule(),
        "numerical-or-access-refinement"
    );
}

/// A live contraction operand may not spell its obligation as a zero range.
///
/// The narrowing that `BoundsProofKind::LiveExtentReach` bought. The retired
/// spelling made the live pairing satisfiable by a count that happened to be
/// zero rather than by anything checked about the access, and the guard on the
/// *static* `ContractionOperand` arm is what stops the demoted proof falling
/// through to a concrete operand-product comparison instead — which would bake
/// the live extent it exists to keep symbolic.
#[test]
fn a_live_contraction_operand_refuses_a_zero_linear_range() {
    let mut builder = live_contraction_builder(0, 1, [2, 3]);
    builder.bounds_proofs[0].kind = BoundsProofKind::LinearRange { element_count: 0 };
    let error = builder
        .build()
        .expect_err("a live operand's obligation is not a zero range");
    assert_eq!(
        error.diagnostics(),
        [ScheduledRegionDiagnostic::BoundsProof],
        "zero-range live contraction operand: {error}"
    );
    assert_eq!(error.diagnostics()[0].rule(), "bounds-proof");
}

/// The same refusal for a count that matches the operand's own product.
///
/// The negative control on the arm above: it is the *relation under this
/// topology* that may not carry a `LinearRange`, not merely the value zero, so
/// the static comparison a non-live operand passes must not rescue this one.
#[test]
fn a_live_contraction_operand_refuses_its_own_static_product() {
    let mut builder = live_contraction_builder(0, 1, [2, 3]);
    builder.bounds_proofs[0].kind = BoundsProofKind::LinearRange { element_count: 2 };
    let error = builder
        .build()
        .expect_err("a live operand may not be sized by its static product");
    assert_eq!(
        error.diagnostics(),
        [ScheduledRegionDiagnostic::BoundsProof],
        "static-product live contraction operand: {error}"
    );
}

/// A static contraction operand may not wear the live variant either.
#[test]
fn a_static_contraction_operand_refuses_the_live_extent_reach() {
    let mut builder = contraction_builder();
    builder.bounds_proofs[0].kind = BoundsProofKind::LiveExtentReach;
    let error = builder
        .build()
        .expect_err("a static operand's reach is a quantity it must state");
    assert_eq!(
        error.diagnostics(),
        [ScheduledRegionDiagnostic::BoundsProof],
        "live reach on a static contraction operand: {error}"
    );
}

/// An axis the named input does not have is refused at schedule verification.
#[test]
fn a_live_contraction_refuses_a_wrong_live_axis() {
    let error = live_contraction_builder(0, 5, [2, 3])
        .build()
        .expect_err("axis 5 is outside the live input's rank");
    assert_eq!(
        error.diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement],
        "wrong-axis live contraction: {error}"
    );
    assert_eq!(
        error.diagnostics()[0].rule(),
        "numerical-or-access-refinement"
    );
}

/// An overflowing static output product is refused by name.
#[test]
fn a_live_contraction_refuses_an_overflowing_output_product() {
    let error = live_contraction_builder(0, 1, [u64::MAX, 2])
        .build()
        .expect_err("a [u64::MAX, 2] output product must overflow");
    assert_eq!(
        error.diagnostics(),
        [ScheduledRegionDiagnostic::ShapeProductOverflow],
        "overflowing live contraction: {error}"
    );
    assert_eq!(error.diagnostics()[0].rule(), "shape-product-overflow");
}

/// An exactly tiled output domain verifies under the blocked binding.
#[test]
fn an_exact_cooperative_contraction_verifies_under_the_blocked_binding() {
    let admitted = admitted_operand_tile();
    assert_eq!(admitted.rounds, 1);
    let verified = operand_contraction_builder(&admitted, operand_tile_fixture())
        .build()
        .expect("the exact-divisible operand-sharing tile verifies");
    assert_eq!(verified.region().schedule.work_items, OUTPUT_POSITIONS);
    assert_eq!(
        verified.region().index.ownership_proof.kind,
        OwnershipProofKind::OneGlobalInvocationPerOutput {
            output_count: OUTPUT_POSITIONS,
        }
    );
    let tile = cooperative_tile(&verified.region().schedule.reduction)
        .expect("the topology carries its tile");
    assert_eq!(tile.commit.count, TILE_PARTICIPANTS);
    assert_eq!(verified.requirements().threads_per_workgroup, 256);
}

/// Preflight refuses a non-divisible output block by name.
#[test]
fn a_non_divisible_output_block_is_refused_in_preflight() {
    let refusal = crate::schedule::admit_exact_cooperative_contraction(
        &Shape::from_dims([33, OUTPUT_EXTENT]),
        &Shape::from_dims([OUTPUT_BLOCK, OUTPUT_BLOCK]),
        &Shape::from_dims([CONTRACTED_EXTENT]),
        &Shape::from_dims([CONTRACTED_TILE]),
    )
    .expect_err("33 is not divisible by 16");
    assert_eq!(
        refusal,
        crate::schedule::CooperativeContractionAdmission::OutputBlockNotDivisible {
            axis: 0,
            output: 33,
            block: OUTPUT_BLOCK,
        }
    );
    assert_eq!(
        refusal.rule(),
        "cooperative-contraction-output-block-not-divisible"
    );
}

/// Preflight refuses a non-divisible contracted tile by name.
#[test]
fn a_non_divisible_contracted_tile_is_refused_in_preflight() {
    let refusal = crate::schedule::admit_exact_cooperative_contraction(
        &Shape::from_dims([OUTPUT_EXTENT, OUTPUT_EXTENT]),
        &Shape::from_dims([OUTPUT_BLOCK, OUTPUT_BLOCK]),
        &Shape::from_dims([17]),
        &Shape::from_dims([CONTRACTED_TILE]),
    )
    .expect_err("17 is not divisible by 16");
    assert_eq!(
        refusal,
        crate::schedule::CooperativeContractionAdmission::ContractedTileNotDivisible {
            axis: 0,
            contracted: 17,
            tile: CONTRACTED_TILE,
        }
    );
    assert_eq!(
        refusal.rule(),
        "cooperative-contraction-contracted-tile-not-divisible"
    );
}

fn predicated_operand_builder(
    output_m: u64,
    output_n: u64,
    contracted: u64,
) -> (
    crate::schedule::PredicatedCooperativeContraction,
    ScheduledRegionBuilder,
) {
    let admitted = crate::schedule::admit_predicated_cooperative_contraction(
        &Shape::from_dims([output_m, output_n]),
        &Shape::from_dims([OUTPUT_BLOCK, OUTPUT_BLOCK]),
        &Shape::from_dims([contracted]),
        &Shape::from_dims([CONTRACTED_TILE]),
    )
    .expect("the predicated launch is representable");
    let output = Shape::from_dims([output_m, output_n]);
    let contracted_shape = Shape::from_dims([contracted]);
    let left = Shape::from_dims([output_m, contracted]);
    let right = Shape::from_dims([output_n, contracted]);
    let work_items = output_m.checked_mul(output_n).expect("M×N fits");
    let operand_map = |free_position, operand: Shape| LogicalAccess::ContractionOperand {
        operand_shape: operand,
        output_shape: output.clone(),
        contracted_shape: contracted_shape.clone(),
        sources: vec![
            ContractionAxisSource::Output {
                position: free_position,
            },
            ContractionAxisSource::Contracted { position: 0 },
        ],
        order: ContributorOrder::OriginalAxisLexicographic,
    };
    let mut tile = operand_tile_fixture();
    tile.rounds = admitted.rounds;
    let mut builder = ScheduledRegionBuilder::new(RegionId::new(7));
    builder.iteration_shape(output.clone()).unwrap();
    for (witness, ordinal, map) in [
        (0, 0, operand_map(0, left.clone())),
        (1, 1, operand_map(1, right.clone())),
    ] {
        builder
            .push_access(Access {
                tensor: TensorRole::Input,
                component_role: None,
                mode: AccessMode::Read,
                map,
                bounds: BoundsWitnessId::new(witness),
                ownership: None,
            })
            .unwrap();
        builder
            .push_bounds_proof(BoundsProof {
                id: BoundsWitnessId::new(witness),
                tensor: TensorRole::Input,
                component_role: None,
                kind: BoundsProofKind::LinearRange {
                    element_count: match ordinal {
                        0 => output_m.checked_mul(contracted).expect("MK fits"),
                        _ => output_n.checked_mul(contracted).expect("NK fits"),
                    },
                },
            })
            .unwrap();
    }
    builder
        .push_access(Access {
            tensor: TensorRole::Output,
            component_role: None,
            mode: AccessMode::Write,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(2),
            ownership: Some(OwnershipWitnessId::new(0)),
        })
        .unwrap();
    builder
        .push_bounds_proof(BoundsProof {
            id: BoundsWitnessId::new(2),
            tensor: TensorRole::Output,
            component_role: None,
            kind: BoundsProofKind::LinearRange {
                element_count: work_items,
            },
        })
        .unwrap();
    builder
        .ownership_proof(OwnershipProof {
            id: OwnershipWitnessId::new(0),
            tensor: TensorRole::Output,
            kind: OwnershipProofKind::OneGlobalInvocationPerOutput {
                output_count: work_items,
            },
        })
        .unwrap();
    builder
        .program(RegionProgram::Numerical {
            scalar: ScalarProgram::StrictTensorContraction {
                contracted_shape: contracted_shape.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                canonical_nan_bits: 0x7fc0_0000,
            },
            numerical: reassociating_numerical(),
        })
        .unwrap();
    let threads = u32::try_from(TILE_PARTICIPANTS).expect("256 fits u32");
    builder
        .schedule(KernelSchedule {
            binding: admitted.binding.clone(),
            work_items: admitted.work_items,
            threads_per_workgroup: threads,
            tail: TailPolicy::Predicated,
            output_owner: OwnershipWitnessId::new(0),
            reduction: ReductionTopology::CooperativeContraction {
                tile,
                contracted_shape,
                contracted_tile: admitted.contracted_tile.clone(),
                order: ContributorOrder::OriginalAxisLexicographic,
                accumulation: ArithmeticType::F32,
                permits_reassociation: true,
                permits_permutation: false,
            },
            launch: LaunchPlan {
                grid_threads: admitted.grid_threads,
                threads_per_workgroup: threads,
                zero_work_skips_dispatch: true,
            },
        })
        .unwrap();
    (admitted, builder)
}

/// Exact and Predicated [32, 32] blocks under the same binding stay distinct.
#[test]
fn exact_and_predicated_neighbours_keep_distinct_identities() {
    let exact = operand_contraction_builder(&admitted_operand_tile(), operand_tile_fixture())
        .build()
        .expect("the exact neighbour verifies");
    let (_, predicated_builder) =
        predicated_operand_builder(OUTPUT_EXTENT, OUTPUT_EXTENT, CONTRACTED_EXTENT);
    let predicated = predicated_builder
        .build()
        .expect("the predicated neighbour verifies");
    assert_eq!(
        exact.region().schedule.work_items,
        predicated.region().schedule.work_items
    );
    assert_eq!(
        exact.region().schedule.launch.grid_threads,
        predicated.region().schedule.launch.grid_threads
    );
    assert_ne!(
        exact.canonical_identity().as_bytes(),
        predicated.canonical_identity().as_bytes()
    );
    let mut exact_hex = String::new();
    let mut predicated_hex = String::new();
    for byte in exact.canonical_identity().as_bytes() {
        write!(&mut exact_hex, "{byte:02x}").unwrap();
    }
    for byte in predicated.canonical_identity().as_bytes() {
        write!(&mut predicated_hex, "{byte:02x}").unwrap();
    }
    assert!(exact_hex.contains("01"), "Exact keeps tail tag 0x01");
    assert!(
        predicated_hex.contains("02"),
        "Predicated appends tail tag 0x02"
    );
}

/// Partial free extents, exact neighbours, zero work, overflow, and nondivisible K.
#[test]
fn predicated_admission_covers_the_required_shapes() {
    let cases = [
        (1, OUTPUT_EXTENT, CONTRACTED_EXTENT, true),
        (10, OUTPUT_EXTENT, CONTRACTED_EXTENT, true),
        (OUTPUT_EXTENT, 10, CONTRACTED_EXTENT, true),
        (10, 10, CONTRACTED_EXTENT, true),
        (OUTPUT_EXTENT, OUTPUT_EXTENT, CONTRACTED_EXTENT, true),
        (0, OUTPUT_EXTENT, CONTRACTED_EXTENT, true),
    ];
    for (m, n, k, ok) in cases {
        let admitted = crate::schedule::admit_predicated_cooperative_contraction(
            &Shape::from_dims([m, n]),
            &Shape::from_dims([OUTPUT_BLOCK, OUTPUT_BLOCK]),
            &Shape::from_dims([k]),
            &Shape::from_dims([CONTRACTED_TILE]),
        );
        assert_eq!(admitted.is_ok(), ok, "M={m} N={n} K={k}");
        if let Ok(admitted) = admitted {
            if m == 0 || n == 0 {
                assert_eq!(admitted.work_items, 0);
                assert_eq!(admitted.grid_threads, 0);
            } else {
                assert_eq!(admitted.work_items, m * n);
                assert!(admitted.grid_threads >= admitted.work_items);
                assert_eq!(admitted.grid_threads % TILE_PARTICIPANTS, 0);
            }
            let (_, builder) = predicated_operand_builder(m, n, k);
            builder
                .build()
                .unwrap_or_else(|error| panic!("M={m} N={n} K={k} refused: {error:?}"));
        }
    }
    let overflow = crate::schedule::admit_predicated_cooperative_contraction(
        &Shape::from_dims([u64::MAX, u64::MAX]),
        &Shape::from_dims([OUTPUT_BLOCK, OUTPUT_BLOCK]),
        &Shape::from_dims([CONTRACTED_EXTENT]),
        &Shape::from_dims([CONTRACTED_TILE]),
    );
    assert_eq!(
        overflow,
        Err(crate::schedule::CooperativeContractionAdmission::ShapeProductOverflow)
    );
    let nondivisible_k = crate::schedule::admit_predicated_cooperative_contraction(
        &Shape::from_dims([10, 10]),
        &Shape::from_dims([OUTPUT_BLOCK, OUTPUT_BLOCK]),
        &Shape::from_dims([17]),
        &Shape::from_dims([CONTRACTED_TILE]),
    );
    assert_eq!(
        nondivisible_k
            .expect_err("17 is not divisible by 16")
            .rule(),
        "cooperative-contraction-contracted-tile-not-divisible"
    );
}

/// Predicated never rewrites itself to Exact when the block happens to divide.
#[test]
fn a_divisible_predicated_proposal_does_not_normalize_to_exact() {
    let (admitted, builder) =
        predicated_operand_builder(OUTPUT_EXTENT, OUTPUT_EXTENT, CONTRACTED_EXTENT);
    let verified = builder
        .build()
        .expect("divisible Predicated still verifies");
    assert_eq!(verified.region().schedule.tail, TailPolicy::Predicated);
    assert_eq!(admitted.grid_threads, OUTPUT_POSITIONS);
    assert_ne!(
        format!("{:?}", verified.region().schedule.tail),
        format!("{:?}", TailPolicy::Exact)
    );
}

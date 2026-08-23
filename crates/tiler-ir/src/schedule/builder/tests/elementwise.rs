use super::super::{
    Access, AccessMode, AccessOrdinal, BoundsProof, BoundsProofKind,
    CanonicalScheduledRegionIdentity, LogicalAccess, NumericalRealization, RegionId, RegionProgram,
    ScalarProgram, ScheduledRegionDiagnostic, TensorRole, encode_identity,
};
use super::support::{
    STRICT_F32_REGION_IDENTITY_HEX, float_rows, pointwise_builder, set_numerical, set_scalar,
    three_input_builder,
};
use crate::schedule::PointwiseF32ExpressionBuilder;
use crate::schedule::handles::BoundsWitnessId;
use crate::schedule::numerics::{
    ApproximationEnvelope, ExceptionalValueAssumption, FlushedZeroSign, NumericalPermission,
    SubnormalMode, ValueDomainProvenance,
};
use crate::shape::Shape;
use std::fmt::Write as _;

/// The same region's identity under `tiler.schedule.v6`.
///
/// Retained rather than deleted, because it is what makes the `v7` step's
/// blast radius a measured fact instead of an assurance: the separator
/// moves *and* the payload moves by exactly the two inserted
/// elementary-dimension bytes, so the retained comparison shows the step
/// changed precisely what its grammar argument claims and nothing else.
///
/// **Rebaselined from the `v5` value at the `v7` step, and the rebaseline is
/// the point rather than housekeeping.** Carried forward unchanged this
/// constant would have made the retained comparison a `v7`-against-`v5` one
/// — a claim about two separator steps combined, which is strictly weaker
/// than a claim about either: a payload change at one step exactly undone at
/// the next satisfies it. Moving it to the `v6` value keeps the comparison
/// proving exactly one step. That discards the `v5` datum deliberately; its
/// whole content was the earlier step's claim, which the commit that made it
/// already carries.
const STRICT_F32_REGION_IDENTITY_HEX_V6: &str = "74696c65722e7363686564756c652e763600000000000000000200000000000000020000000000000003000000000000000201000101000000000002000201000000010100000000000000000000000200000000010011000000000000000600000001020011000000000000000600000000020000000000000006240000000000000005000000000000001500000000000000010100000000000000040000000000000000000000150000000000000001020000000000000004400000000000000000000021000000000000000104000000000000000400000000000000000000000400000001000000000000001500000000000000010200000000000000043f8000000000000000000021000000000000000103000000000000000400000002000000000000000400000003000000000000000400000004000000000000001574696c65722e746573742e7374726963742d6633327fc0000001010101010101010100000000000000060000000101000000003100000000000000060000000101";

#[test]
fn valid_pointwise_region_verifies_and_derives_requirements() {
    let verified = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
        .build()
        .unwrap();
    assert_eq!(verified.region().schedule.work_items, 6);
    assert_eq!(verified.requirements().buffer_bindings, 2);
    assert!(verified.requirements().requires_device_memory);
    // The realization reaches the requirements record per dimension rather
    // than as a predicate, so a feasibility authority can name the exact
    // dimension a target failed to honour (ADR 0076 item 3).
    let requirements = verified.requirements();
    assert_eq!(
        float_rows(&requirements).input_subnormals,
        SubnormalMode::Preserve
    );
    assert_eq!(
        float_rows(&requirements).result_subnormals,
        SubnormalMode::Preserve
    );
    assert_eq!(
        float_rows(&requirements).contraction,
        NumericalPermission::Forbidden
    );
    assert_eq!(
        float_rows(&requirements).reassociation,
        NumericalPermission::Forbidden
    );
    assert_eq!(
        float_rows(&requirements).permutation,
        NumericalPermission::Forbidden
    );
    assert_eq!(
        float_rows(&requirements).signed_zero,
        NumericalPermission::Forbidden
    );
    assert_eq!(
        float_rows(&requirements).nan_assumptions,
        ExceptionalValueAssumption::MakeNoAssumption
    );
    assert_eq!(
        float_rows(&requirements).infinity_assumptions,
        ExceptionalValueAssumption::MakeNoAssumption
    );
}

/// A contract that permits both transforms still carries its subnormal
/// obligation into the requirements record.
///
/// The `requires_strict_f32` predicate this replaced read contraction and
/// reassociation only, so exactly this realization derived `false` and
/// would have been admitted on a target declaring no strict-`f32` support
/// while still demanding preserved subnormals.
#[test]
fn a_relaxed_transform_contract_still_carries_its_subnormal_obligation() {
    let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
    set_numerical(
        &mut builder,
        NumericalRealization::new(
            "tiler.test.relaxed-transforms-preserved-subnormals",
            0x7fc0_0000,
            SubnormalMode::Preserve,
            SubnormalMode::Preserve,
            NumericalPermission::Permitted,
            NumericalPermission::Permitted,
            NumericalPermission::Forbidden,
            NumericalPermission::Forbidden,
            NumericalPermission::Forbidden,
            ApproximationEnvelope::Forbidden,
            ExceptionalValueAssumption::MakeNoAssumption,
            ExceptionalValueAssumption::MakeNoAssumption,
        ),
    );
    let carried = builder.build().unwrap().requirements();
    assert_eq!(
        float_rows(&carried).contraction,
        NumericalPermission::Permitted
    );
    assert_eq!(
        float_rows(&carried).reassociation,
        NumericalPermission::Permitted
    );
    assert_eq!(
        float_rows(&carried).input_subnormals,
        SubnormalMode::Preserve
    );
    assert_eq!(
        float_rows(&carried).result_subnormals,
        SubnormalMode::Preserve
    );
}

#[test]
fn pointwise_f32_admits_output_and_rejects_other_destination_roles() {
    let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
    builder.accesses[1].tensor = TensorRole::Output;
    builder.bounds_proofs[1].tensor = TensorRole::Output;
    builder.ownership_proof.as_mut().unwrap().tensor = TensorRole::Output;
    assert!(builder.build().is_ok());

    let mut rejected = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
    rejected.accesses[1].tensor = TensorRole::Input;
    rejected.bounds_proofs[1].tensor = TensorRole::Input;
    rejected.ownership_proof.as_mut().unwrap().tensor = TensorRole::Input;
    assert_eq!(
        rejected.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
    );
}

#[test]
fn a_three_input_pointwise_region_verifies_and_binds_one_buffer_per_read() {
    let verified = three_input_builder(4).build().unwrap();
    assert_eq!(verified.requirements().buffer_bindings, 4);
    assert_eq!(verified.region().index.accesses.len(), 4);
}

/// Input roles do not carry program-interface association.
///
/// Reordering otherwise identical input accesses remains intrinsically
/// well-formed. The compiler owns the checked association between each
/// exact local access and the declared program input it serves.
#[test]
fn input_access_roles_are_fieldless_and_positioned_by_the_access_list() {
    let mut permuted = three_input_builder(4);
    permuted.accesses.swap(0, 1);
    permuted.bounds_proofs.swap(0, 1);
    assert!(permuted.build().is_ok());
    let verified = three_input_builder(4).build().unwrap();
    assert!(
        verified.region().index.accesses[..3]
            .iter()
            .all(|access| access.tensor == TensorRole::Input)
    );
}

/// A rank-one reindex over the whole extent, mirrored or not.
///
/// A single decode spanning the domain tiles it, so both spellings are
/// bijections a pointwise region admits; neither is `LinearIdentity`, and the
/// two are different relations. Those are the only properties the
/// repeated-read cases below need.
fn whole_extent_reindex(elements: u64, mirrored: bool) -> LogicalAccess {
    let shape = crate::shape::Shape::from_dims([elements]);
    LogicalAccess::ReindexBijection {
        operand_shape: shape.clone(),
        result_shape: shape,
        axes: vec![crate::schedule::AxisDecode {
            divisor: 1,
            modulus: elements,
            mirrored,
        }],
    }
}

/// One input may be read twice when the two reads address it differently.
///
/// This is the region behind `a * permute(a)`: two expression leaves mean
/// two different tensors derived from one declared input, so they need two
/// reads with two relations. Binding one access to both leaves is what made
/// that program compile as `permute(a) * permute(a)` and return a wrong
/// tensor, so the admission and its bound are the same rule.
///
/// Local access order is identity-bearing and the compiler binds that order
/// to its checked request subject. A repeated intermediate remains refused:
/// the role carries no ordinal, so the attribution that makes the input pair
/// unambiguous is exactly what it lacks.
#[test]
fn one_declared_input_may_be_read_densely_and_through_a_relation() {
    let control = three_input_builder(4).build().unwrap();

    let mut paired = three_input_builder(4);
    paired.accesses[1].tensor = TensorRole::Input;
    paired.accesses[1].map = whole_extent_reindex(4, true);
    paired.bounds_proofs[1].tensor = TensorRole::Input;
    let verified = paired.build().unwrap();
    // Three reads and a write still bind four buffers: a second read of one
    // declared input is a second binding, not a shared one.
    assert_eq!(verified.requirements().buffer_bindings, 4);
    // The pair reaches the encoding, so the region that reads input `0`
    // twice is a different region from the one that reads inputs `0` and
    // `1` — not one region with two spellings.
    assert_ne!(
        verified.canonical_identity().as_bytes(),
        control.canonical_identity().as_bytes()
    );

    let mut reversed = three_input_builder(4);
    reversed.accesses[0].map = whole_extent_reindex(4, true);
    reversed.accesses[1].tensor = TensorRole::Input;
    reversed.bounds_proofs[1].tensor = TensorRole::Input;
    let reversed = reversed.build().unwrap();
    assert_ne!(reversed.canonical_identity(), verified.canonical_identity());

    let mut two_relations = three_input_builder(4);
    two_relations.accesses[0].map = whole_extent_reindex(4, false);
    two_relations.accesses[1].tensor = TensorRole::Input;
    two_relations.accesses[1].map = whole_extent_reindex(4, true);
    two_relations.bounds_proofs[1].tensor = TensorRole::Input;
    assert!(two_relations.build().is_ok());

    let mut two_intermediates = three_input_builder(4);
    for position in 0..2 {
        two_intermediates.accesses[position].tensor = TensorRole::Intermediate;
        two_intermediates.bounds_proofs[position].tensor = TensorRole::Intermediate;
    }
    two_intermediates.accesses[1].map = whole_extent_reindex(4, true);
    assert_eq!(
        two_intermediates.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
    );
}

/// An elementwise region may read one materialized intermediate, and only
/// one.
///
/// This is the consumer half of a `producer -> intermediate -> epilogue`
/// chain: the region carries the epilogue's own expression and binds one of
/// its leaves to a tensor an earlier region wrote. Every other obligation is
/// discharged exactly as the input-reading control's is — the same bounds
/// proof, the same ownership proof, the same map — so a widening rather than
/// a relaxation.
///
/// The two refusals are what the widening must not lose. A second
/// intermediate read is ambiguous rather than merely unsupported:
/// `TensorRole::Intermediate` carries no ordinal, so nothing says which
/// materialization edge each read binds. A read of the program output is
/// refused for a different reason — a region does not consume what it
/// publishes — and both report the access-refinement rule.
#[test]
fn an_elementwise_region_may_read_one_materialized_intermediate() {
    let control = three_input_builder(4).build().unwrap();

    let mut epilogue = three_input_builder(4);
    epilogue.accesses[0].tensor = TensorRole::Intermediate;
    epilogue.bounds_proofs[0].tensor = TensorRole::Intermediate;
    let verified = epilogue.build().unwrap();
    assert_eq!(verified.requirements().buffer_bindings, 4);
    // The read's boundary role reaches the encoding, so the epilogue and its
    // input-reading control are distinct regions rather than one region with
    // two spellings.
    assert_ne!(
        verified.canonical_identity().as_bytes(),
        control.canonical_identity().as_bytes()
    );

    let mut two_intermediates = three_input_builder(4);
    for position in 0..2 {
        two_intermediates.accesses[position].tensor = TensorRole::Intermediate;
        two_intermediates.bounds_proofs[position].tensor = TensorRole::Intermediate;
    }
    assert_eq!(
        two_intermediates.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
    );

    let mut reads_output = three_input_builder(4);
    reads_output.accesses[0].tensor = TensorRole::Output;
    reads_output.bounds_proofs[0].tensor = TensorRole::Output;
    assert_eq!(
        reads_output.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::NumericalOrAccessRefinement]
    );
}

/// An expression leaf with no read access behind it is refused by count.
///
/// Without this the kernel lowering would look up input `2` among two
/// loaded values, and the region would have promised a buffer its signature
/// never declares.
#[test]
fn a_pointwise_region_reads_exactly_one_tensor_per_expression_leaf() {
    let mut short = three_input_builder(4);
    short.accesses.remove(2);
    short.bounds_proofs.remove(2);
    assert_eq!(
        short.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::AccessCount]
    );

    // The converse: an access no leaf reads is refused by the same rule.
    let mut long = three_input_builder(4);
    long.accesses.insert(
        3,
        Access {
            tensor: TensorRole::Input,
            component_role: None,
            mode: AccessMode::Read,
            map: LogicalAccess::LinearIdentity,
            bounds: BoundsWitnessId::new(4),
            ownership: None,
        },
    );
    long.bounds_proofs.insert(
        3,
        BoundsProof {
            id: BoundsWitnessId::new(4),
            tensor: TensorRole::Input,
            component_role: None,
            kind: BoundsProofKind::LinearRange { element_count: 4 },
        },
    );
    assert_eq!(
        long.build().unwrap_err().diagnostics(),
        [ScheduledRegionDiagnostic::AccessCount]
    );
}

/// Two regions differing only in which input a leaf reads differ in identity.
///
/// `(a * b) + c` and `(a * a) + c` compute different things, and before the
/// ordinal reached the encoding neither the role nor the leaf could say so.
#[test]
fn input_ordinals_separate_canonical_scheduled_region_identity() {
    let three = three_input_builder(4).build().unwrap();

    let mut expression = PointwiseF32ExpressionBuilder::new();
    let a = expression.input(AccessOrdinal::new(0)).unwrap();
    let b = expression.input(AccessOrdinal::new(1)).unwrap();
    // The same shape of program, but the product squares its first input.
    let product = expression.multiply(a.clone(), a).unwrap();
    let root = expression.add(product, b).unwrap();
    let squared = expression.build(root).unwrap();

    let mut builder = three_input_builder(4);
    builder.accesses.remove(2);
    builder.bounds_proofs.remove(2);
    builder.accesses[2].bounds = BoundsWitnessId::new(3);
    set_scalar(&mut builder, ScalarProgram::PointwiseF32(squared));
    let two = builder.build().unwrap();

    assert_ne!(
        three.canonical_identity().as_bytes(),
        two.canonical_identity().as_bytes()
    );
}

fn identity_with_pointwise_expression(
    expression: super::super::super::PointwiseF32Expression,
) -> Vec<u8> {
    let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
    set_scalar(&mut builder, ScalarProgram::PointwiseF32(expression));
    builder
        .build()
        .unwrap()
        .canonical_identity()
        .as_bytes()
        .to_vec()
}

/// The reciprocal square root is a distinct node from the exponential.
///
/// Both are one-argument elementary functions over one input, so nothing but
/// the node tag distinguishes their expressions. An appended tag that had
/// collided with `Exp`'s would make these two identities equal, which is the
/// concrete form of "the schedule domain did not step": the new tag
/// separates, and every tag below it keeps its meaning.
#[test]
fn the_reciprocal_square_root_node_separates_identity_from_the_exponential() {
    fn elementary(reciprocal_square_root: bool) -> super::super::super::PointwiseF32Expression {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let input = builder.input(AccessOrdinal::FIRST).unwrap();
        let root = if reciprocal_square_root {
            builder.rsqrt(input).unwrap()
        } else {
            builder.exp(input).unwrap()
        };
        builder.build(root).unwrap()
    }
    assert_ne!(
        identity_with_pointwise_expression(elementary(true)),
        identity_with_pointwise_expression(elementary(false))
    );
}

#[test]
fn pointwise_identity_canonicalizes_ready_order_and_separates_semantics() {
    fn ready_order(reverse: bool) -> super::super::super::PointwiseF32Expression {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let input = builder.input(AccessOrdinal::FIRST).unwrap();
        let (two, three) = if reverse {
            let three = builder.constant(3.0_f32.to_bits()).unwrap();
            let two = builder.constant(2.0_f32.to_bits()).unwrap();
            (two, three)
        } else {
            let two = builder.constant(2.0_f32.to_bits()).unwrap();
            let three = builder.constant(3.0_f32.to_bits()).unwrap();
            (two, three)
        };
        let (add, product) = if reverse {
            let product = builder.multiply(input.clone(), three).unwrap();
            let add = builder.add(input, two).unwrap();
            (add, product)
        } else {
            let add = builder.add(input.clone(), two).unwrap();
            let product = builder.multiply(input, three).unwrap();
            (add, product)
        };
        let root = builder.add(add, product).unwrap();
        builder.build(root).unwrap()
    }

    let canonical = identity_with_pointwise_expression(ready_order(false));
    assert_eq!(
        canonical,
        identity_with_pointwise_expression(ready_order(true))
    );

    let association = {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let input = builder.input(AccessOrdinal::FIRST).unwrap();
        let two = builder.constant(2.0_f32.to_bits()).unwrap();
        let three = builder.constant(3.0_f32.to_bits()).unwrap();
        let inner = builder.add(two, three).unwrap();
        let root = builder.add(input, inner).unwrap();
        identity_with_pointwise_expression(builder.build(root).unwrap())
    };
    assert_ne!(canonical, association);

    let operand_order = {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let input = builder.input(AccessOrdinal::FIRST).unwrap();
        let two = builder.constant(2.0_f32.to_bits()).unwrap();
        let three = builder.constant(3.0_f32.to_bits()).unwrap();
        let add = builder.add(two, input.clone()).unwrap();
        let product = builder.multiply(three, input).unwrap();
        let root = builder.add(add, product).unwrap();
        identity_with_pointwise_expression(builder.build(root).unwrap())
    };
    assert_ne!(canonical, operand_order);

    let constant_bits = {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let input = builder.input(AccessOrdinal::FIRST).unwrap();
        let two = builder.constant((-2.0_f32).to_bits()).unwrap();
        let three = builder.constant(3.0_f32.to_bits()).unwrap();
        let add = builder.add(input.clone(), two).unwrap();
        let product = builder.multiply(input, three).unwrap();
        let root = builder.add(add, product).unwrap();
        identity_with_pointwise_expression(builder.build(root).unwrap())
    };
    assert_ne!(canonical, constant_bits);
}

#[test]
fn pointwise_identity_separates_signed_zero_and_nan_payload_bits() {
    fn literal_identity(bits: u32) -> Vec<u8> {
        let mut builder = PointwiseF32ExpressionBuilder::new();
        let input = builder.input(AccessOrdinal::FIRST).unwrap();
        let constant = builder.constant(bits).unwrap();
        let root = builder.add(input, constant).unwrap();
        identity_with_pointwise_expression(builder.build(root).unwrap())
    }

    assert_ne!(
        literal_identity(0.0_f32.to_bits()),
        literal_identity((-0.0_f32).to_bits())
    );
    assert_ne!(literal_identity(0x7fc0_0001), literal_identity(0x7fc0_0002));
}

/// Every numerical dimension separates canonical scheduled-region identity.
///
/// The encoder previously wrote `profile_key`, the NaN bits, and two
/// derived permission booleans, so two regions differing only in a
/// subnormal dimension collided. Each realization below holds `profile_key`
/// fixed precisely so the key cannot stand in for the field values it names
/// (ADR 0076 item 6). The subject is `encode_identity` rather than the
/// builder because the schedule verifier separately constrains the scalar
/// program to agree with the contraction permission, and varying both would
/// stop isolating the numerical field.
#[test]
fn every_numerical_dimension_separates_scheduled_region_identity() {
    let region = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
        .build()
        .unwrap()
        .region()
        .clone();
    let baseline = NumericalRealization::new(
        "tiler.test.identity-probe",
        0x7fc0_0000,
        SubnormalMode::Preserve,
        SubnormalMode::Preserve,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        NumericalPermission::Forbidden,
        ApproximationEnvelope::Forbidden,
        ExceptionalValueAssumption::MakeNoAssumption,
        ExceptionalValueAssumption::MakeNoAssumption,
    );
    let preserving_sign = SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::PreservesSign,
    };
    let always_positive = SubnormalMode::FlushToZero {
        zero_sign: FlushedZeroSign::AlwaysPositive,
    };
    let realizations = [
        baseline,
        NumericalRealization {
            input_subnormals: preserving_sign,
            ..baseline
        },
        NumericalRealization {
            result_subnormals: preserving_sign,
            ..baseline
        },
        // The flushed zero's sign is part of the behaviour, so two flushes
        // producing different zeros are different realizations.
        NumericalRealization {
            input_subnormals: always_positive,
            ..baseline
        },
        NumericalRealization {
            contraction: NumericalPermission::Permitted,
            ..baseline
        },
        NumericalRealization {
            reassociation: NumericalPermission::Permitted,
            ..baseline
        },
        NumericalRealization {
            permutation: NumericalPermission::Permitted,
            ..baseline
        },
        NumericalRealization {
            signed_zero: NumericalPermission::Permitted,
            ..baseline
        },
        NumericalRealization {
            nan_assumptions: ExceptionalValueAssumption::AssumeAbsent {
                provenance: ValueDomainProvenance::CompilerProven,
            },
            ..baseline
        },
        NumericalRealization {
            infinity_assumptions: ExceptionalValueAssumption::AssumeAbsent {
                provenance: ValueDomainProvenance::RuntimeValidated,
            },
            ..baseline
        },
        NumericalRealization {
            nan_assumptions: ExceptionalValueAssumption::AssumeAbsent {
                provenance: ValueDomainProvenance::CallerDeclaredUnvalidated,
            },
            ..baseline
        },
    ];

    let mut seen: Vec<CanonicalScheduledRegionIdentity> = Vec::new();
    for realization in realizations {
        let mut candidate = region.clone();
        let RegionProgram::Numerical { numerical, .. } = &mut candidate.index.program else {
            panic!("the fixture region is arithmetic");
        };
        *numerical = realization;
        let identity = encode_identity(&candidate);
        assert!(
            !seen.contains(&identity),
            "{realization:?} collided with an earlier realization"
        );
        seen.push(identity);
    }
}

/// The exact canonical identity of the governed strict-`f32` test region.
///
/// Completing the encoding over both subnormal dimensions and re-encoding
/// each permission as a tagged value changed these bytes. Pinning them
/// keeps a later reordering or omission from slipping past the distinctness
/// test above, which only proves that its eleven realizations differ from
/// each other.
#[test]
fn the_strict_f32_region_has_its_recorded_canonical_identity() {
    let verified = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
        .build()
        .unwrap();
    let hex =
        verified
            .canonical_identity()
            .as_bytes()
            .iter()
            .fold(String::new(), |mut hex, byte| {
                let _ = write!(hex, "{byte:02x}");
                hex
            });
    assert_eq!(hex, STRICT_F32_REGION_IDENTITY_HEX);
}

#[test]
fn equivalent_regions_with_different_ids_share_identity() {
    let first = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
        .build()
        .unwrap();
    let second = pointwise_builder(RegionId::new(7), Shape::from_dims([2, 3]), 6)
        .build()
        .unwrap();
    assert_ne!(first.region().index.id, second.region().index.id);
    assert_eq!(
        first.canonical_identity().as_bytes(),
        second.canonical_identity().as_bytes()
    );
}

#[test]
fn distinct_content_has_distinct_identity() {
    let first = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
        .build()
        .unwrap();
    let second = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 4]), 8)
        .build()
        .unwrap();
    assert_ne!(
        first.canonical_identity().as_bytes(),
        second.canonical_identity().as_bytes()
    );
}

#[test]
fn zero_domain_pointwise_region_verifies() {
    let verified = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 0]), 0)
        .build()
        .unwrap();
    assert_eq!(verified.region().schedule.work_items, 0);
}

#[test]
fn write_without_ownership_is_rejected_by_the_access_contract() {
    let mut builder = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6);
    builder.accesses[1].ownership = None;
    let error = builder.build().unwrap_err();
    assert_eq!(
        error.diagnostics(),
        [ScheduledRegionDiagnostic::AccessContract]
    );
}

/// The `v7` step moves the domain and exactly the elementary payload bytes.
///
/// Compared against the retained `v6` identity structurally rather than by
/// bare inequality: the payload delta must be precisely the two inserted
/// one-byte rows — the reciprocal-transform permission and the
/// approximate-intrinsic envelope — between the signed-zero permission and
/// the NaN assumption, so a step that moved anything else fails here
/// instead of hiding inside "the bytes differ".
#[test]
fn the_elementary_dimension_step_moves_domain_and_payload() {
    // Eighteen bytes of `tiler.schedule.vN\0`, so thirty-six hex digits.
    const SEPARATOR: usize = 36;

    let verified = pointwise_builder(RegionId::new(0), Shape::from_dims([2, 3]), 6)
        .build()
        .unwrap();
    let mut hex = String::new();
    for byte in verified.canonical_identity().as_bytes() {
        write!(&mut hex, "{byte:02x}").unwrap();
    }
    assert_eq!(hex, STRICT_F32_REGION_IDENTITY_HEX);
    assert_ne!(
        STRICT_F32_REGION_IDENTITY_HEX[..SEPARATOR],
        STRICT_F32_REGION_IDENTITY_HEX_V6[..SEPARATOR]
    );
    // The two spellings differ by exactly four hex digits — the two
    // inserted permission/envelope tag bytes — at one position inside the
    // numerical record. Locate the insertion by the longest common prefix
    // and check the suffixes re-align after it.
    let new = &STRICT_F32_REGION_IDENTITY_HEX[SEPARATOR..];
    let old = &STRICT_F32_REGION_IDENTITY_HEX_V6[SEPARATOR..];
    assert_eq!(new.len(), old.len() + 4, "two one-byte rows were inserted");
    let prefix = new
        .as_bytes()
        .iter()
        .zip(old.as_bytes())
        .take_while(|(new, old)| new == old)
        .count();
    assert_eq!(
        &new[prefix + 4..],
        &old[prefix..],
        "every byte after the two inserted rows is carried unchanged"
    );
}

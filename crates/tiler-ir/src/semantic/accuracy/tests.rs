//! Every rule of the accuracy vocabulary, watched refusing.
//!
//! A check that has never been seen to say no is indistinguishable from one that
//! cannot. Each test below either exhibits the exact perturbation that turns an
//! accepted contract into a refusal, or pins a value whose derivation is a claim
//! about a specification rather than about this code.

use super::rational::{ReductionPathCounts, take_reduction_path_counts};
use super::*;
use crate::semantic::{
    F32, NormativeDefinitionRef, OpKey, ResolvedValueType, TypeKey,
    builtin_scalar_value_type_facts, builtin_scalar_value_types,
};

// --- fixtures -------------------------------------------------------------

fn f32_facts() -> crate::semantic::CanonicalValue {
    builtin_scalar_value_type_facts(&F32::resolved_type()).expect("f32 is a governed built-in")
}

fn f32_format() -> UlpFormat {
    UlpFormat::from_value_type_facts(&f32_facts()).expect("f32 carries the metric")
}

fn reference() -> NormativeDefinitionRef {
    NormativeDefinitionRef::new("the exponential function on the reals")
        .expect("a governed reference is bounded")
}

fn proof() -> NormativeDefinitionRef {
    NormativeDefinitionRef::new("exp is strictly positive on the reals")
        .expect("a governed proof reference is bounded")
}

fn exceptional() -> ExceptionalValueContract {
    ExceptionalValueContract::new(
        NanReferenceRule::CanonicalNan,
        InfiniteReferenceRule::SignedInfinity,
        DomainErrorRule::CanonicalNan,
        FiniteOverflowRule::SignedInfinity,
    )
}

fn whole_domain_clause(
    predicate: AccuracyPredicate,
) -> Result<AccuracyDomainClause, AccuracyContractError> {
    AccuracyDomainClause::new(
        [(OperandOrdinal::new(0), DomainInterval::unbounded())],
        ReferenceResultConstraint::new([ReferenceResultClass::Positive], None, Some(proof()))?,
        predicate,
    )
}

fn bounded_contract(form: AccuracyContractForm) -> AccuracyContract {
    AccuracyContract::new(
        OpKey::new("test", "exp-f32", 1).expect("a test operation key is canonical"),
        vec![F32::resolved_type()],
        F32::resolved_type(),
        reference(),
        form,
        exceptional(),
    )
}

fn ulp_contract(tolerance: ExactTolerance) -> AccuracyContract {
    let clause = whole_domain_clause(AccuracyPredicate::ulp(
        ulp_reference_gap_metric_key(),
        tolerance,
    ))
    .expect("the clause is well formed");
    bounded_contract(AccuracyContractForm::BoundedPiecewise(
        AccuracyDomain::new([DomainInterval::unbounded()], [clause]).expect("one clause covers"),
    ))
}

// --- exact rational arithmetic -------------------------------------------

/// A tolerance is an exact number, so a value no host float holds is exact here.
#[test]
fn exact_rationals_are_exact_where_host_floats_are_not() {
    let tenth = ExactRational::from_ratio(1, 10).expect("a nonzero denominator");
    let three_tenths = tenth.add(&tenth).add(&tenth);
    assert_eq!(
        three_tenths,
        ExactRational::from_ratio(3, 10).expect("valid")
    );
    // The same sum in binary floating point is not the same value, which is the
    // whole reason ADR 0042 forbids a host floating-point literal here. Compared
    // by bit pattern rather than by `!=`, so the assertion is about the two
    // values rather than about a tolerance.
    assert_ne!((0.1_f64 + 0.1 + 0.1).to_bits(), 0.3_f64.to_bits());
}

/// Lowest terms is the invariant that makes the encoding an identity.
#[test]
fn exact_rationals_normalize_to_one_spelling() {
    let halves = ExactRational::from_ratio(2, 4).expect("valid");
    let half = ExactRational::from_ratio(1, 2).expect("valid");
    assert_eq!(halves, half);
    let (sign, numerator, denominator) = halves.to_sign_magnitude_ratio();
    assert_eq!(
        (sign, numerator, denominator),
        (ExactSign::Positive, vec![1], vec![2])
    );
}

/// Reducing by a shift finds the divisor the general algorithm finds.
///
/// A power-of-two denominator is the shape a certified enclosure produces at
/// every outward rounding, and its reduction is a trailing-zero count rather than
/// a greatest-common-divisor loop. The whole population is enumerated and both
/// answers are counted, so a shift that reduced nothing could not look like
/// agreement. The oracle divides by `num_integer::Integer::gcd` directly, so it
/// is the general algorithm rather than a second copy of the shift.
#[test]
fn a_dyadic_denominator_reduces_by_the_general_divisor() {
    use num_bigint::BigUint;
    use num_integer::Integer;

    const MAGNITUDES: i128 = 192;
    const EXPONENTS: u32 = 12;

    let one = BigUint::from(1_u32);
    let mut checked = 0_usize;
    let mut shared_a_factor = 0_usize;
    for magnitude in 1..=MAGNITUDES {
        for exponent in 0..=EXPONENTS {
            for sign in [1_i128, -1] {
                let reduced = ExactRational::from_integer(sign * magnitude).scale_by_power_of_two(
                    -i32::try_from(exponent).expect("a bounded exponent fits i32"),
                );
                let numerator =
                    BigUint::from(u128::try_from(magnitude).expect("strictly positive"));
                let denominator = &one << exponent;
                let divisor = numerator.gcd(&denominator);
                let expected = ExactRational::from_sign_magnitude_ratio(
                    if sign < 0 {
                        ExactSign::Negative
                    } else {
                        ExactSign::Positive
                    },
                    &(&numerator / &divisor).to_bytes_be(),
                    &(&denominator / &divisor).to_bytes_be(),
                )
                .expect("dividing out the greatest common divisor leaves lowest terms");
                assert_eq!(reduced, expected, "{sign} * {magnitude} / 2^{exponent}");
                checked += 1;
                if divisor != one {
                    shared_a_factor += 1;
                }
            }
        }
    }
    assert_eq!(
        checked,
        usize::try_from(MAGNITUDES).expect("bounded") * (EXPONENTS as usize + 1) * 2
    );
    // A factor is shared exactly when the magnitude is even and the exponent is
    // at least one: 96 even magnitudes, 12 nonzero exponents, both signs.
    assert_eq!(shared_a_factor, 96 * 12 * 2);
}

/// A dyadic denominator must not silently fall back to the general gcd path.
///
/// The value assertion keeps the observed call attached to real normalization,
/// while the exact path census protects the value-preserving cost distinction.
/// Replacing the dyadic branch with the module's observed general reduction mechanism
/// leaves the value unchanged but increments the general-call count.
#[test]
fn a_dyadic_reduction_never_enters_the_general_gcd_path() {
    let _ = take_reduction_path_counts();

    let reduced = ExactRational::from_ratio(6, 8).expect("a nonzero denominator");

    assert_eq!(
        reduced.to_sign_magnitude_ratio(),
        (ExactSign::Positive, vec![3], vec![4])
    );
    assert_eq!(
        take_reduction_path_counts(),
        ReductionPathCounts {
            total: 1,
            general: 0,
        },
        "the observed dyadic normalization must avoid the general gcd path"
    );
}

/// A non-dyadic denominator still takes the general path and reduces correctly.
#[test]
fn a_non_dyadic_reduction_keeps_the_general_gcd_path() {
    let _ = take_reduction_path_counts();

    let reduced = ExactRational::from_ratio(6, 15).expect("a nonzero denominator");

    assert_eq!(
        reduced.to_sign_magnitude_ratio(),
        (ExactSign::Positive, vec![2], vec![5])
    );
    assert_eq!(
        take_reduction_path_counts(),
        ReductionPathCounts {
            total: 1,
            general: 1,
        },
        "the observed non-dyadic normalization must retain the general gcd path"
    );
}

/// The widest dyadic pair the decode boundary admits is decided, both ways.
///
/// [`MAX_EXACT_RATIONAL_MAGNITUDE_BYTES`] is what an outside caller may present,
/// so this is the boundary's own worst case rather than a convenient one. The odd
/// numerator is coprime with every power of two and is admitted; the same
/// magnitude made even shares the denominator's factor and is refused, which is
/// the invariant that keeps one number from acquiring two spellings.
#[test]
fn the_widest_dyadic_decode_is_decided_both_ways() {
    let _ = take_reduction_path_counts();
    let mut denominator = vec![0_u8; MAX_EXACT_RATIONAL_MAGNITUDE_BYTES];
    denominator[0] = 1;

    // Compared as a boolean rather than with `assert_eq!` throughout, because a
    // mismatch on four-kilobyte magnitudes would otherwise bury the reason under
    // the operands.
    let odd = vec![0xff_u8; MAX_EXACT_RATIONAL_MAGNITUDE_BYTES];
    let admitted =
        ExactRational::from_sign_magnitude_ratio(ExactSign::Positive, &odd, &denominator)
            .expect("an odd magnitude is coprime with a power of two");
    assert!(
        admitted.to_sign_magnitude_ratio()
            == (ExactSign::Positive, odd.clone(), denominator.clone()),
        "a pair already in lowest terms must decode unchanged"
    );

    let mut even = odd;
    let last = even.len() - 1;
    even[last] = 0xfe;
    match ExactRational::from_sign_magnitude_ratio(ExactSign::Positive, &even, &denominator) {
        Err(error) => assert_eq!(error, ExactRationalError::NotInLowestTerms),
        Ok(_) => panic!("an even magnitude shares the denominator's factor of two"),
    }
    assert_eq!(
        take_reduction_path_counts(),
        ReductionPathCounts {
            total: 2,
            general: 0,
        },
        "the widest admitted pair must retain bounded dyadic reduction"
    );
}

/// Zero magnitude keeps its separate dyadic answer and decoder refusal.
#[test]
fn zero_magnitude_keeps_its_dyadic_reduction_answer() {
    let _ = take_reduction_path_counts();
    // Zero is the one magnitude whose divisor is the denominator itself, which no
    // shift expresses, so the reduction answers it apart. Two rules refuse this
    // pair — that answer, and the decoder's own zero rule below it — and breaking
    // either alone still leaves it refused, so this holds the pair rather than
    // one site.
    assert_eq!(
        ExactRational::from_sign_magnitude_ratio(ExactSign::Zero, &[], &[2])
            .expect_err("zero over two is a second spelling of zero"),
        ExactRationalError::NotInLowestTerms
    );
    assert!(ExactRational::from_sign_magnitude_ratio(ExactSign::Zero, &[], &[1]).is_ok());
    assert_eq!(
        take_reduction_path_counts(),
        ReductionPathCounts {
            total: 2,
            general: 0,
        },
        "zero over a dyadic denominator must not enter the general gcd path"
    );
}

/// A second spelling of one number is refused on decode, not renormalized.
#[test]
fn a_non_lowest_terms_decode_is_refused() {
    let error = ExactRational::from_sign_magnitude_ratio(ExactSign::Positive, &[2], &[4])
        .expect_err("2/4 is not in lowest terms");
    assert_eq!(error, ExactRationalError::NotInLowestTerms);
    assert_eq!(
        error.diagnostic_code(),
        "accuracy.rational.not-in-lowest-terms"
    );
}

/// Every exact-number refusal is reachable and carries its own code.
#[test]
fn every_exact_number_refusal_can_say_no() {
    assert_eq!(
        ExactRational::from_ratio(1, 0).expect_err("a zero denominator"),
        ExactRationalError::ZeroDenominator
    );
    assert_eq!(
        ExactRational::zero()
            .reciprocal()
            .expect_err("zero has no reciprocal"),
        ExactRationalError::DivisionByZero
    );
    assert!(matches!(
        ExactRational::from_integer(-1)
            .sqrt_enclosure(8)
            .expect_err("a negative radicand"),
        ExactRationalError::NegativeSquareRoot { .. }
    ));
    assert!(matches!(
        ExactTolerance::try_from_rational(ExactRational::from_integer(-1))
            .expect_err("a negative tolerance"),
        ExactRationalError::NegativeTolerance { .. }
    ));
    assert!(matches!(
        ExactRational::from_sign_magnitude_ratio(ExactSign::Positive, &[0, 1], &[1])
            .expect_err("a leading zero"),
        ExactRationalError::NoncanonicalLeadingZero
    ));
    assert!(matches!(
        ExactRational::from_sign_magnitude_ratio(ExactSign::Zero, &[1], &[1])
            .expect_err("zero with a magnitude"),
        ExactRationalError::NonemptyZeroMagnitude
    ));
    assert!(matches!(
        ExactRational::from_sign_magnitude_ratio(ExactSign::Positive, &[], &[1])
            .expect_err("a nonzero sign with no magnitude"),
        ExactRationalError::EmptyNonzeroMagnitude
    ));
    assert!(matches!(
        ExactRational::from_sign_magnitude_ratio(
            ExactSign::Positive,
            &vec![1; MAX_EXACT_RATIONAL_MAGNITUDE_BYTES + 1],
            &[1]
        )
        .expect_err("an over-long magnitude"),
        ExactRationalError::MagnitudeTooLong { .. }
    ));
}

/// The square-root enclosure brackets rather than approximates, and narrows on request.
#[test]
fn the_square_root_enclosure_brackets_and_narrows() {
    let two = ExactRational::from_integer(2);
    let (coarse_lower, coarse_upper) = two.sqrt_enclosure(4).expect("nonnegative");
    let (fine_lower, fine_upper) = two.sqrt_enclosure(40).expect("nonnegative");
    for (lower, upper) in [(&coarse_lower, &coarse_upper), (&fine_lower, &fine_upper)] {
        assert!(
            lower.multiply(lower) <= two,
            "the lower endpoint must not exceed the root"
        );
        assert!(
            upper.multiply(upper) > two,
            "the upper endpoint must exceed the root"
        );
    }
    let coarse_width = coarse_upper.subtract(&coarse_lower);
    let fine_width = fine_upper.subtract(&fine_lower);
    assert!(
        fine_width < coarse_width,
        "more grid bits must narrow the enclosure"
    );
}

/// A binary32 bit pattern maps to the exact rational the format defines.
#[test]
fn binary32_values_map_to_their_exact_rational_value() {
    assert_eq!(
        ExactRational::from_f32(1.5).expect("finite"),
        ExactRational::from_ratio(3, 2).expect("valid")
    );
    // The least positive subnormal is 2^-149, exactly.
    assert_eq!(
        ExactRational::from_f32(f32::from_bits(1)).expect("finite"),
        ExactRational::power_of_two(-149)
    );
    assert_eq!(ExactRational::from_f32(f32::INFINITY), None);
    assert_eq!(ExactRational::from_f32(f32::NAN), None);
}

// --- the ULP metric -------------------------------------------------------

/// The binary32 parameters come from the catalog descriptor, not from a copy here.
#[test]
fn the_metric_derives_binary32_from_its_own_descriptor() {
    let format = f32_format();
    assert_eq!(format.precision(), 24);
    assert_eq!(format.max_exponent(), 127);
    assert_eq!(format.min_exponent(), -126);
    assert!(format.has_subnormals());
    assert_eq!(format.class(), "ieee-binary");
    assert_eq!(
        format.largest_finite(),
        ExactRational::from_f32(f32::MAX).expect("finite")
    );
    assert_eq!(
        format.smallest_positive_finite(),
        ExactRational::power_of_two(-149)
    );
}

/// At a power of two the smaller gap applies, and the scale rises immediately above it.
///
/// This is the clause of ADR 0042 that distinguishes `tiler::ulp-reference-gap@1`
/// from the definitions that leave the representable case unresolved, so it is
/// pinned at the exact boundary rather than in the middle of a binade.
#[test]
fn the_representable_case_uses_the_predecessor_gap() {
    let format = f32_format();
    let one = ExactRational::one();
    assert_eq!(
        format.ulp_scale(&one).expect("in range"),
        ExactRational::power_of_two(-24),
        "ulp(1) is the predecessor gap 2^-24, not the successor gap 2^-23"
    );
    let just_above = one.add(&ExactRational::power_of_two(-30));
    assert_eq!(
        format.ulp_scale(&just_above).expect("in range"),
        ExactRational::power_of_two(-23),
        "the scale rises immediately above a power of two"
    );
    let just_below = one.subtract(&ExactRational::power_of_two(-30));
    assert_eq!(
        format.ulp_scale(&just_below).expect("in range"),
        ExactRational::power_of_two(-24)
    );
}

/// At zero the scale is the least positive finite value, and it is the subnormal one.
#[test]
fn the_zero_rule_uses_the_least_positive_finite_value() {
    let format = f32_format();
    assert_eq!(
        format
            .ulp_scale(&ExactRational::zero())
            .expect("defined at zero"),
        ExactRational::power_of_two(-149)
    );
    // Throughout the gradual subnormal interval the scale stays the minimum
    // positive subnormal; flushing is a separate contract and does not move it.
    assert_eq!(
        format
            .ulp_scale(&ExactRational::power_of_two(-140))
            .expect("in range"),
        ExactRational::power_of_two(-149)
    );
}

/// A reference above the largest finite value leaves the metric's domain.
///
/// The perturbation: the largest finite value is inside, and a step above it is
/// outside. The `OpenCL` hypothetical-successor overflow allowance would accept the
/// second; `tiler::ulp-reference-gap@1` deliberately does not inherit it.
#[test]
fn the_metric_refuses_a_reference_past_the_finite_range() {
    let format = f32_format();
    let largest = format.largest_finite();
    assert!(format.ulp_scale(&largest).is_ok());
    let past = largest.add(&ExactRational::power_of_two(104));
    let error = format
        .ulp_scale(&past)
        .expect_err("outside the finite range");
    assert_eq!(
        error.diagnostic_code(),
        "accuracy.metric.reference-out-of-finite-range"
    );
}

/// Every dtype the catalog governs is classified, and only the float classes carry the metric.
///
/// The loop names its population and counts both answers, so "every dtype is
/// compatible" and "the check did not run" cannot look the same.
#[test]
fn the_metric_rejects_every_dtype_whose_adjacent_values_it_cannot_derive() {
    let mut compatible = Vec::new();
    let mut rejected = Vec::new();
    for value_type in builtin_scalar_value_types() {
        let facts = builtin_scalar_value_type_facts(&value_type).expect("a governed scalar");
        let name = value_type
            .nominal_key()
            .expect("the catalog's scalars are nominal")
            .name()
            .to_owned();
        match UlpFormat::from_value_type_facts(&facts) {
            Ok(_) => compatible.push(name),
            Err(error) => rejected.push((name, error)),
        }
    }
    compatible.sort();
    assert_eq!(compatible, vec!["bf16", "f128", "f16", "f32", "f64"]);
    assert_eq!(
        compatible.len() + rejected.len(),
        builtin_scalar_value_types().len(),
        "every governed scalar must land on exactly one side"
    );
    // Each rejected family is rejected for a reason a reader can act on.
    let reason = |name: &str| {
        rejected
            .iter()
            .find(|(candidate, _)| candidate == name)
            .map_or("not rejected", |(_, error)| error.diagnostic_code())
    };
    assert_eq!(reason("bool"), "accuracy.metric.incompatible-dtype");
    assert_eq!(reason("i32"), "accuracy.metric.incompatible-dtype");
    assert_eq!(reason("decimal64"), "accuracy.metric.incompatible-dtype");
    assert_eq!(reason("f8e4m3fn"), "accuracy.metric.incompatible-dtype");
    assert_eq!(reason("f8e8m0fnu"), "accuracy.metric.incompatible-dtype");
}

/// The interpretable classes are exactly the two rows, each with a stated basis.
#[test]
fn the_format_rule_table_names_its_population() {
    let rules: Vec<_> = ulp_metric_format_rules().collect();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].0, "ieee-binary");
    assert_eq!(rules[1].0, "bfloat");
    for (class, basis) in rules {
        assert!(!basis.is_empty(), "{class} states no basis");
    }
    assert!(!f32_format().normative_basis().is_empty());
}

/// A complex or foreign identity is not a governed scalar row and reports so.
#[test]
fn a_non_scalar_identity_has_no_scalar_descriptor() {
    let complex = crate::semantic::complex_value_type(&F32::resolved_type()).expect("admitted");
    assert!(builtin_scalar_value_type_facts(&complex).is_none());
    let foreign = ResolvedValueType::nominal(TypeKey::new("other", "f32", 1).expect("valid"));
    assert!(builtin_scalar_value_type_facts(&foreign).is_none());
}

// --- predicate normalization ---------------------------------------------

/// Same-kind nesting flattens, members sort and deduplicate, singletons collapse.
#[test]
fn boolean_predicates_normalize() {
    let a = AccuracyPredicate::absolute(ExactTolerance::from_integer(1));
    let b = AccuracyPredicate::absolute(ExactTolerance::from_integer(2));
    let nested = AccuracyPredicate::all_of([
        a.clone(),
        AccuracyPredicate::all_of([b.clone(), a.clone()]).expect("valid"),
    ])
    .expect("valid");
    let flat = AccuracyPredicate::all_of([b.clone(), a.clone()]).expect("valid");
    assert_eq!(
        nested, flat,
        "nesting and duplication normalize to one predicate"
    );
    let AccuracyPredicateView::Boolean { members, .. } = flat.view() else {
        panic!("a two-member conjunction stays Boolean")
    };
    assert_eq!(members.len(), 2);
    assert!(
        members[0] < members[1],
        "members are sorted by canonical encoding"
    );

    let singleton = AccuracyPredicate::all_of([a.clone()]).expect("valid");
    assert_eq!(singleton, a, "a singleton canonicalizes to its member");
    // A different Boolean kind does not flatten into this one.
    let mixed = AccuracyPredicate::all_of([
        a,
        AccuracyPredicate::any_of([b, AccuracyPredicate::relative(ExactTolerance::zero())])
            .expect("valid"),
    ])
    .expect("valid");
    let AccuracyPredicateView::Boolean { members, .. } = mixed.view() else {
        panic!("a mixed conjunction stays Boolean")
    };
    assert_eq!(members.len(), 2);
}

/// An empty collection is invalid, and both kinds say so under their own name.
#[test]
fn an_empty_boolean_collection_is_refused() {
    for (result, kind) in [
        (AccuracyPredicate::all_of([]), BooleanPredicateKind::AllOf),
        (AccuracyPredicate::any_of([]), BooleanPredicateKind::AnyOf),
    ] {
        let error = result.expect_err("an empty collection is invalid");
        assert_eq!(
            error,
            AccuracyContractError::EmptyPredicateCollection { kind }
        );
        assert_eq!(
            error.diagnostic_code(),
            "accuracy.predicate.empty-collection"
        );
    }
}

/// The member and nesting bounds each refuse.
#[test]
fn the_predicate_bounds_each_refuse() {
    let members: Vec<_> = (0..=MAX_ACCURACY_PREDICATE_MEMBERS)
        .map(|index| {
            AccuracyPredicate::absolute(ExactTolerance::from_integer(
                u64::try_from(index).expect("a bounded member count fits u64") + 1,
            ))
        })
        .collect();
    assert!(matches!(
        AccuracyPredicate::all_of(members).expect_err("over the member bound"),
        AccuracyContractError::TooManyPredicateMembers { .. }
    ));

    // Alternating kinds cannot flatten, so nesting accumulates depth.
    let mut deep = AccuracyPredicate::absolute(ExactTolerance::from_integer(1));
    let mut error = None;
    for level in 0..MAX_ACCURACY_PREDICATE_DEPTH + 4 {
        let sibling = AccuracyPredicate::relative(ExactTolerance::from_integer(
            u64::try_from(level).expect("a bounded level count fits u64") + 1,
        ));
        let built = if level % 2 == 0 {
            AccuracyPredicate::all_of([deep.clone(), sibling])
        } else {
            AccuracyPredicate::any_of([deep.clone(), sibling])
        };
        match built {
            Ok(next) => deep = next,
            Err(reported) => {
                error = Some(reported);
                break;
            }
        }
    }
    assert!(
        matches!(
            error,
            Some(
                AccuracyContractError::PredicateTooDeep { .. }
                    | AccuracyContractError::TooManyPredicateNodes { .. }
            )
        ),
        "unbounded nesting must be refused, got {error:?}"
    );
}

/// `AnyOf` does not hide an undefined relative predicate at a zero reference.
///
/// The recursion is the point: the disjunction's other member is satisfied at
/// zero, and the relative member is still undefined there.
#[test]
fn a_relative_predicate_under_any_of_still_requires_a_nonzero_reference() {
    let hidden = AccuracyPredicate::any_of([
        AccuracyPredicate::absolute(ExactTolerance::from_integer(1)),
        AccuracyPredicate::relative(ExactTolerance::from_integer(1)),
    ])
    .expect("valid");
    assert!(hidden.requires_nonzero_reference());
    let bare = AccuracyPredicate::absolute(ExactTolerance::from_integer(1));
    assert!(!bare.requires_nonzero_reference());
}

/// Every predicate round-trips through its canonical value.
#[test]
fn predicates_round_trip_through_canonical_values() {
    let predicates = [
        AccuracyPredicate::absolute(ExactTolerance::from_ratio(1, 8).expect("valid")),
        AccuracyPredicate::relative(ExactTolerance::from_integer(3)),
        AccuracyPredicate::absolute_relative(
            ExactTolerance::from_integer(1),
            ExactTolerance::from_ratio(1, 16).expect("valid"),
        ),
        AccuracyPredicate::ulp(
            ulp_reference_gap_metric_key(),
            ExactTolerance::from_integer(4),
        ),
        AccuracyPredicate::all_of([
            AccuracyPredicate::absolute(ExactTolerance::from_integer(1)),
            AccuracyPredicate::any_of([
                AccuracyPredicate::relative(ExactTolerance::from_integer(2)),
                AccuracyPredicate::ulp(
                    ulp_reference_gap_metric_key(),
                    ExactTolerance::from_integer(4),
                ),
            ])
            .expect("valid"),
        ])
        .expect("valid"),
    ];
    for predicate in predicates {
        let encoded = predicate.to_canonical_value().expect("bounded");
        let decoded = AccuracyPredicate::from_canonical_value(&encoded).expect("round trip");
        assert_eq!(decoded, predicate);
        assert_eq!(decoded.canonical_encoding(), predicate.canonical_encoding());
    }
}

/// Decode refuses every non-canonical Boolean spelling instead of renormalizing.
///
/// Five perturbations of one valid encoding, each producing a distinct code. If
/// decode renormalized instead, all five would silently succeed and one predicate
/// would have six identities.
#[test]
fn a_non_canonical_boolean_encoding_is_refused_rather_than_renormalized() {
    use crate::semantic::{AttributeFieldId, CanonicalField, CanonicalValue};

    let small = AccuracyPredicate::absolute(ExactTolerance::from_integer(1));
    let large = AccuracyPredicate::absolute(ExactTolerance::from_integer(2));
    let sorted = AccuracyPredicate::all_of([small.clone(), large.clone()]).expect("valid");
    assert!(
        AccuracyPredicate::from_canonical_value(&sorted.to_canonical_value().expect("bounded"))
            .is_ok()
    );

    let boolean = |kind: &str, members: Vec<CanonicalValue>| {
        CanonicalValue::record([
            CanonicalField::new(
                AttributeFieldId::new(1),
                CanonicalValue::utf8(kind).expect("bounded"),
            ),
            CanonicalField::new(
                AttributeFieldId::new(5),
                CanonicalValue::sequence(members).expect("bounded"),
            ),
        ])
        .expect("bounded")
    };
    let encode = |predicate: &AccuracyPredicate| predicate.to_canonical_value().expect("bounded");

    // Which of the two encodes first tells us which order is non-canonical.
    let (first, second) = if small < large {
        (&small, &large)
    } else {
        (&large, &small)
    };

    let unsorted = boolean("all-of", vec![encode(second), encode(first)]);
    assert_eq!(
        AccuracyPredicate::from_canonical_value(&unsorted)
            .expect_err("unsorted members")
            .diagnostic_code(),
        "accuracy.predicate.non-canonical-order"
    );

    let duplicated = boolean("all-of", vec![encode(first), encode(first)]);
    assert_eq!(
        AccuracyPredicate::from_canonical_value(&duplicated)
            .expect_err("a repeated member")
            .diagnostic_code(),
        "accuracy.predicate.duplicate-member"
    );

    let singleton = boolean("all-of", vec![encode(first)]);
    assert_eq!(
        AccuracyPredicate::from_canonical_value(&singleton)
            .expect_err("a singleton collection")
            .diagnostic_code(),
        "accuracy.predicate.non-canonical-singleton"
    );

    let empty = boolean("all-of", Vec::new());
    assert_eq!(
        AccuracyPredicate::from_canonical_value(&empty)
            .expect_err("an empty collection")
            .diagnostic_code(),
        "accuracy.predicate.empty-collection"
    );

    let nested = boolean(
        "all-of",
        vec![
            encode(&AccuracyPredicate::relative(ExactTolerance::zero())),
            encode(&sorted),
        ],
    );
    assert_eq!(
        AccuracyPredicate::from_canonical_value(&nested)
            .expect_err("unflattened same-kind nesting")
            .diagnostic_code(),
        "accuracy.predicate.non-canonical-nesting"
    );
}

// --- the accuracy domain --------------------------------------------------

/// An interval admitting no value is refused where it is written.
#[test]
fn an_empty_domain_interval_is_refused() {
    let operand = OperandOrdinal::new(0);
    let one = ExactRational::one();
    assert!(
        DomainInterval::new(
            operand,
            DomainBound::Closed(one.clone()),
            DomainBound::Closed(one.clone())
        )
        .is_ok(),
        "a single-point closed interval admits one value"
    );
    let error = DomainInterval::new(
        operand,
        DomainBound::Open(one.clone()),
        DomainBound::Closed(one),
    )
    .expect_err("an open lower bound at the closed upper bound admits nothing");
    assert_eq!(error.diagnostic_code(), "accuracy.domain.empty-interval");
}

/// A clause that asserts a reference-result class without a proof is refused.
#[test]
fn an_unjustified_reference_result_class_is_refused() {
    assert!(
        ReferenceResultConstraint::new([ReferenceResultClass::Nonzero], None, Some(proof()))
            .is_ok()
    );
    let error = ReferenceResultConstraint::new(
        [ReferenceResultClass::Nonzero],
        None,
        None::<NormativeDefinitionRef>,
    )
    .expect_err("no operation-specific proof");
    assert_eq!(
        error.diagnostic_code(),
        "accuracy.domain.unjustified-reference-result-class"
    );
    // The unconstrained constraint asserts nothing and needs no proof.
    assert!(!ReferenceResultConstraint::unconstrained().proves_nonzero());
}

/// A gap in the clauses is found and reported with a witness point.
///
/// The perturbation: two clauses that meet exactly cover the line, and the same
/// two with one endpoint opened leave a hole the check names.
#[test]
fn an_uncovered_domain_reports_the_point_it_missed() {
    let operand = OperandOrdinal::new(0);
    let zero = ExactRational::zero();
    let predicate = AccuracyPredicate::ulp(
        ulp_reference_gap_metric_key(),
        ExactTolerance::from_integer(4),
    );
    let clause = |lower: DomainBound, upper: DomainBound| {
        AccuracyDomainClause::new(
            [(
                operand,
                DomainInterval::new(operand, lower, upper).expect("nonempty"),
            )],
            ReferenceResultConstraint::unconstrained(),
            predicate.clone(),
        )
        .expect("well formed")
    };

    let covering = AccuracyDomain::new(
        [DomainInterval::unbounded()],
        [
            clause(DomainBound::Unbounded, DomainBound::Open(zero.clone())),
            clause(DomainBound::Closed(zero.clone()), DomainBound::Unbounded),
        ],
    )
    .expect("well formed");
    assert!(covering.verify_coverage().is_ok());

    let holed = AccuracyDomain::new(
        [DomainInterval::unbounded()],
        [
            clause(DomainBound::Unbounded, DomainBound::Open(zero.clone())),
            clause(DomainBound::Open(zero.clone()), DomainBound::Unbounded),
        ],
    )
    .expect("well formed");
    let error = holed.verify_coverage().expect_err("zero is uncovered");
    assert_eq!(
        error.diagnostic_code(),
        "accuracy.domain.incomplete-coverage"
    );
    assert_eq!(
        error,
        AccuracyContractError::IncompleteDomainCoverage {
            witness: vec![zero]
        }
    );
}

/// Overlapping clauses intersect: every matching clause applies, in no order.
#[test]
fn overlapping_clauses_intersect_rather_than_taking_priority() {
    let operand = OperandOrdinal::new(0);
    let clause = |tolerance: u64, lower: DomainBound, upper: DomainBound| {
        AccuracyDomainClause::new(
            [(
                operand,
                DomainInterval::new(operand, lower, upper).expect("nonempty"),
            )],
            ReferenceResultConstraint::unconstrained(),
            AccuracyPredicate::ulp(
                ulp_reference_gap_metric_key(),
                ExactTolerance::from_integer(tolerance),
            ),
        )
        .expect("well formed")
    };
    let domain = AccuracyDomain::new(
        [DomainInterval::unbounded()],
        [
            clause(4, DomainBound::Unbounded, DomainBound::Unbounded),
            clause(
                2,
                DomainBound::Closed(ExactRational::zero()),
                DomainBound::Unbounded,
            ),
        ],
    )
    .expect("well formed");
    let cells = domain.verify_coverage().expect("covered");
    let overlapped = cells
        .iter()
        .filter(|cell| cell.applicable().len() == 2)
        .count();
    assert!(
        overlapped > 0,
        "the two clauses overlap on the nonnegative side"
    );
    assert!(
        cells.iter().all(|cell| !cell.applicable().is_empty()),
        "coverage means no cell has an empty applicable set"
    );
}

/// A decomposition too large to decide is a refusal, not a truncated check.
#[test]
fn an_undecidable_coverage_budget_refuses() {
    let predicate = AccuracyPredicate::ulp(
        ulp_reference_gap_metric_key(),
        ExactTolerance::from_integer(4),
    );
    // Four operands, each split by every clause, exceeds the cell budget.
    let admitted: Vec<_> = (0..MAX_ACCURACY_DOMAIN_OPERANDS)
        .map(|_| DomainInterval::unbounded())
        .collect();
    let clauses: Vec<_> = (0..MAX_ACCURACY_DOMAIN_CLAUSES)
        .map(|index| {
            let bindings: Vec<_> = (0..MAX_ACCURACY_DOMAIN_OPERANDS)
                .map(|operand| {
                    let ordinal = OperandOrdinal::new(
                        u32::try_from(operand).expect("a bounded operand count fits u32"),
                    );
                    (
                        ordinal,
                        DomainInterval::new(
                            ordinal,
                            DomainBound::Closed(ExactRational::from_integer(
                                i128::try_from(index).expect("a bounded clause count fits i128"),
                            )),
                            DomainBound::Unbounded,
                        )
                        .expect("nonempty"),
                    )
                })
                .collect();
            AccuracyDomainClause::new(
                bindings,
                ReferenceResultConstraint::unconstrained(),
                predicate.clone(),
            )
            .expect("well formed")
        })
        .collect();
    let domain = AccuracyDomain::new(admitted, clauses).expect("well formed");
    let error = domain
        .verify_coverage()
        .expect_err("the decomposition exceeds the budget");
    assert_eq!(
        error.diagnostic_code(),
        "accuracy.domain.coverage-not-verifiable"
    );
}

/// A bounded contract with no clauses is refused.
#[test]
fn an_empty_clause_set_is_refused() {
    let error = AccuracyDomain::new([DomainInterval::unbounded()], [])
        .expect_err("a bounded contract needs a clause");
    assert_eq!(error.diagnostic_code(), "accuracy.domain.empty-clause-set");
}

// --- contract verification ------------------------------------------------

/// A four-ULP contract over the whole domain verifies, and states how.
#[test]
fn a_constant_ulp_contract_verifies_by_exhibiting_round_to_nearest() {
    let contract = ulp_contract(ExactTolerance::from_integer(4));
    let verified = contract.verify(&f32_facts()).expect("verifies");
    assert!(matches!(
        verified.establishment(),
        ResultSetEstablishment::RoundToNearestWitness { .. }
    ));
    assert_eq!(
        verified.contract().canonical_encoding(),
        contract.canonical_encoding()
    );
}

/// A ULP tolerance below one half admits no candidate and is refused.
///
/// The perturbation on the accepted contract above: one half verifies, one
/// quarter does not, because round-to-nearest attains exactly half an ULP at a
/// midpoint and it is the closest any representable value can be.
#[test]
fn a_ulp_tolerance_below_the_rounding_floor_is_refused() {
    assert!(
        ulp_contract(ExactTolerance::from_ratio(1, 2).expect("valid"))
            .verify(&f32_facts())
            .is_ok()
    );
    let error = ulp_contract(ExactTolerance::from_ratio(1, 4).expect("valid"))
        .verify(&f32_facts())
        .expect_err("a quarter ULP admits nothing");
    assert_eq!(
        error.diagnostic_code(),
        "accuracy.contract.empty-composed-result-set"
    );
    assert!(matches!(
        error,
        AccuracyContractError::EmptyComposedResultSet {
            reason: UnestablishedResultSet::UlpToleranceBelowRoundingFloor { .. }
        }
    ));
}

/// An absolute bound with no proved reference magnitude cannot be established.
#[test]
fn an_absolute_bound_without_a_reference_magnitude_is_refused() {
    let clause = whole_domain_clause(AccuracyPredicate::absolute(ExactTolerance::from_integer(1)))
        .expect("well formed");
    let contract = bounded_contract(AccuracyContractForm::BoundedPiecewise(
        AccuracyDomain::new([DomainInterval::unbounded()], [clause]).expect("well formed"),
    ));
    let error = contract
        .verify(&f32_facts())
        .expect_err("no magnitude bound");
    assert!(matches!(
        error,
        AccuracyContractError::EmptyComposedResultSet {
            reason: UnestablishedResultSet::AbsoluteBoundWithoutReferenceMagnitude { .. }
        }
    ));
}

/// With a proved magnitude bound the absolute clause is decided, both ways.
#[test]
fn an_absolute_bound_is_decided_against_the_proved_spacing() {
    let operand = OperandOrdinal::new(0);
    // A reference proved to lie in [1, 2] has spacing 2^-23 just above one, so
    // half a spacing is 2^-24 and any tolerance at or above it is satisfiable.
    let magnitude = DomainInterval::new(
        operand,
        DomainBound::Closed(ExactRational::one()),
        DomainBound::Closed(ExactRational::from_integer(2)),
    )
    .expect("nonempty");
    let contract = |tolerance: ExactTolerance| {
        let clause = AccuracyDomainClause::new(
            [(operand, DomainInterval::unbounded())],
            ReferenceResultConstraint::new(
                [ReferenceResultClass::Positive],
                Some(magnitude.clone()),
                Some(proof()),
            )
            .expect("justified"),
            AccuracyPredicate::absolute(tolerance),
        )
        .expect("well formed");
        bounded_contract(AccuracyContractForm::BoundedPiecewise(
            AccuracyDomain::new([DomainInterval::unbounded()], [clause]).expect("well formed"),
        ))
    };
    assert!(
        contract(
            ExactTolerance::try_from_rational(ExactRational::power_of_two(-23)).expect("valid")
        )
        .verify(&f32_facts())
        .is_ok()
    );
    let error = contract(
        ExactTolerance::try_from_rational(ExactRational::power_of_two(-40)).expect("valid"),
    )
    .verify(&f32_facts())
    .expect_err("below the spacing at the proved maximum");
    assert!(matches!(
        error,
        AccuracyContractError::EmptyComposedResultSet {
            reason: UnestablishedResultSet::AbsoluteBoundBelowSpacing { .. }
        }
    ));
}

/// A relative clause over a domain that admits a zero reference is refused.
///
/// The perturbation: the same clause with the reference proved nonzero verifies.
#[test]
fn a_relative_clause_at_a_possibly_zero_reference_is_refused() {
    let operand = OperandOrdinal::new(0);
    let build = |constraint: ReferenceResultConstraint| {
        let clause = AccuracyDomainClause::new(
            [(operand, DomainInterval::unbounded())],
            constraint,
            AccuracyPredicate::relative(ExactTolerance::from_integer(1)),
        )
        .expect("well formed");
        bounded_contract(AccuracyContractForm::BoundedPiecewise(
            AccuracyDomain::new([DomainInterval::unbounded()], [clause]).expect("well formed"),
        ))
    };

    let error = build(ReferenceResultConstraint::unconstrained())
        .verify(&f32_facts())
        .expect_err("a zero reference is admitted");
    assert_eq!(
        error.diagnostic_code(),
        "accuracy.predicate.undefined-relative-at-zero-reference"
    );

    // Proving the reference nonzero clears the definedness rule, and a proved
    // magnitude range then decides the bound itself.
    let magnitude = DomainInterval::new(
        operand,
        DomainBound::Closed(ExactRational::one()),
        DomainBound::Closed(ExactRational::from_integer(2)),
    )
    .expect("nonempty");
    assert!(
        build(
            ReferenceResultConstraint::new(
                [ReferenceResultClass::Nonzero],
                Some(magnitude),
                Some(proof()),
            )
            .expect("justified"),
        )
        .verify(&f32_facts())
        .is_ok()
    );
}

/// A dtype whose adjacent-value behaviour is not derivable refuses the contract.
#[test]
fn a_bounded_contract_over_an_incompatible_dtype_is_refused() {
    let facts = builtin_scalar_value_type_facts(&ResolvedValueType::nominal(
        TypeKey::new("tiler", "i32", 1).expect("valid"),
    ))
    .expect("a governed scalar");
    let error = ulp_contract(ExactTolerance::from_integer(4))
        .verify(&facts)
        .expect_err("an integer has no ULP");
    assert_eq!(
        error.diagnostic_code(),
        "accuracy.contract.incompatible-result-dtype"
    );
}

/// A clause measuring under an undefined metric is refused rather than assumed.
///
/// This is the shape the Metal evidence takes: a bound stated under a vendor's
/// own ULP definition. It is carried faithfully and it is not silently measured
/// under Tiler's metric.
#[test]
fn a_clause_under_a_foreign_metric_is_not_silently_adopted() {
    let foreign = AccuracyMetricKey::new("apple", "msl-ulp", 1).expect("valid");
    assert!(!foreign.is_ulp_reference_gap());
    let clause = whole_domain_clause(AccuracyPredicate::ulp(
        foreign,
        ExactTolerance::from_integer(4),
    ))
    .expect("well formed");
    let contract = bounded_contract(AccuracyContractForm::BoundedPiecewise(
        AccuracyDomain::new([DomainInterval::unbounded()], [clause]).expect("well formed"),
    ));
    let error = contract
        .verify(&f32_facts())
        .expect_err("an undefined metric");
    assert!(matches!(
        error,
        AccuracyContractError::EmptyComposedResultSet {
            reason: UnestablishedResultSet::UnregisteredMetric { .. }
        }
    ));
}

/// The five composition steps are in ADR 0042's order, and only one can empty the set.
#[test]
fn the_composition_states_its_order_and_where_it_can_fail() {
    let contract = ulp_contract(ExactTolerance::from_integer(4));
    assert_eq!(
        contract.composition_steps(),
        [
            CompositionStep::InputSubnormalContract,
            CompositionStep::ExactReferenceClassification,
            CompositionStep::AccuracyConformingCandidateSelection,
            CompositionStep::ResultSubnormalAndSignedZeroMapping,
            CompositionStep::NanCanonicalization,
        ]
    );
    let emptying: Vec<_> = CompositionStep::ORDER
        .into_iter()
        .filter(|step| step.can_empty_the_result_set())
        .collect();
    assert_eq!(
        emptying,
        vec![CompositionStep::AccuracyConformingCandidateSelection]
    );
}

/// The four forms are distinct identities, never equated by name.
#[test]
fn the_four_contract_forms_have_four_identities() {
    let digest = NamedElementaryDescriptorDigest::new([0xab, 0xcd]).expect("nonempty");
    let forms = [
        AccuracyContractForm::CorrectlyRounded {
            rounding: ReferenceRoundingRule::NearestTiesToEven,
        },
        AccuracyContractForm::Faithful,
        AccuracyContractForm::BoundedPiecewise(
            AccuracyDomain::new(
                [DomainInterval::unbounded()],
                [whole_domain_clause(correctly_rounded_ulp_bound()).expect("well formed")],
            )
            .expect("well formed"),
        ),
        AccuracyContractForm::NamedElementary {
            profile: NamedElementaryProfileKey::new("vendor", "fast-exp", 1).expect("valid"),
            descriptor_digest: digest,
            descriptor_basis: NormativeDefinitionRef::new("the vendor's fast-math table")
                .expect("bounded"),
        },
    ];
    let mut encodings: Vec<_> = forms
        .iter()
        .map(|form| bounded_contract(form.clone()).canonical_encoding())
        .collect();
    encodings.sort();
    encodings.dedup();
    assert_eq!(encodings.len(), 4, "the four forms must not collide");
}

/// An empty named-elementary descriptor digest pins nothing and is refused.
#[test]
fn an_empty_named_elementary_digest_is_refused() {
    assert!(NamedElementaryDescriptorDigest::new([1]).is_ok());
    let error = NamedElementaryDescriptorDigest::new([]).expect_err("an empty digest pins nothing");
    assert_eq!(
        error.diagnostic_code(),
        "accuracy.contract.malformed-attribute"
    );
}

/// Every contract form round-trips through its canonical value.
#[test]
fn contracts_round_trip_through_canonical_values() {
    let digest = NamedElementaryDescriptorDigest::new([1, 2, 3]).expect("nonempty");
    for form in [
        AccuracyContractForm::CorrectlyRounded {
            rounding: ReferenceRoundingRule::NearestTiesToEven,
        },
        AccuracyContractForm::Faithful,
        AccuracyContractForm::BoundedPiecewise(
            AccuracyDomain::new(
                [DomainInterval::unbounded()],
                [whole_domain_clause(AccuracyPredicate::ulp(
                    ulp_reference_gap_metric_key(),
                    ExactTolerance::from_integer(4),
                ))
                .expect("well formed")],
            )
            .expect("well formed"),
        ),
        AccuracyContractForm::NamedElementary {
            profile: NamedElementaryProfileKey::new("vendor", "fast-exp", 1).expect("valid"),
            descriptor_digest: digest,
            descriptor_basis: NormativeDefinitionRef::new("the vendor's fast-math table")
                .expect("bounded"),
        },
    ] {
        let contract = bounded_contract(form);
        let encoded = contract.to_canonical_value().expect("bounded");
        let decoded = AccuracyContract::from_canonical_value(&encoded).expect("round trip");
        assert_eq!(decoded, contract);
        assert_eq!(decoded.canonical_encoding(), contract.canonical_encoding());
    }
}

/// Every field ADR 0042 names for semantic identity moves the encoding.
#[test]
fn every_identity_field_moves_the_canonical_encoding() {
    let base = ulp_contract(ExactTolerance::from_integer(4));
    let baseline = base.canonical_encoding();
    let f64_type = ResolvedValueType::nominal(TypeKey::new("tiler", "f64", 1).expect("valid"));
    let perturbations = [
        AccuracyContract::new(
            OpKey::new("test", "exp-f32", 2).expect("valid"),
            base.operand_types().to_vec(),
            base.result_type().clone(),
            base.reference_semantics().clone(),
            base.form().clone(),
            base.exceptional(),
        ),
        AccuracyContract::new(
            base.operation().clone(),
            vec![f64_type.clone()],
            base.result_type().clone(),
            base.reference_semantics().clone(),
            base.form().clone(),
            base.exceptional(),
        ),
        AccuracyContract::new(
            base.operation().clone(),
            base.operand_types().to_vec(),
            f64_type,
            base.reference_semantics().clone(),
            base.form().clone(),
            base.exceptional(),
        ),
        AccuracyContract::new(
            base.operation().clone(),
            base.operand_types().to_vec(),
            base.result_type().clone(),
            NormativeDefinitionRef::new("a different reference").expect("bounded"),
            base.form().clone(),
            base.exceptional(),
        ),
        ulp_contract(ExactTolerance::from_integer(5)),
        AccuracyContract::new(
            base.operation().clone(),
            base.operand_types().to_vec(),
            base.result_type().clone(),
            base.reference_semantics().clone(),
            base.form().clone(),
            ExceptionalValueContract::new(
                NanReferenceRule::CanonicalNan,
                InfiniteReferenceRule::CanonicalNan,
                DomainErrorRule::CanonicalNan,
                FiniteOverflowRule::SignedInfinity,
            ),
        ),
    ];
    for (index, perturbed) in perturbations.iter().enumerate() {
        assert_ne!(
            perturbed.canonical_encoding(),
            baseline,
            "perturbation {index} did not move the identity"
        );
    }
}

// --- refinement -----------------------------------------------------------

/// A tighter bound of the same shape refines; a looser one does not.
#[test]
fn a_tighter_exact_bound_refines_and_a_looser_one_does_not() {
    let registry = RegisteredImplicationRegistry::standard().expect("the standard rows register");
    let tight = ulp_contract(ExactTolerance::from_integer(2));
    let loose = ulp_contract(ExactTolerance::from_integer(4));
    assert_eq!(
        refines(&tight, &loose, &registry),
        RefinementOutcome::Refines {
            basis: RefinementBasis::TighterExactBound
        }
    );
    let outcome = refines(&loose, &tight, &registry);
    assert!(!outcome.is_physically_feasible());
    assert_eq!(
        outcome,
        RefinementOutcome::Unknown {
            reason: RefinementUnknown::LooserExactBound
        }
    );
}

/// Correctly rounded, faithful, and one-ULP are three obligations, not one.
///
/// The registered rows go one way only. A faithful candidate against a correctly
/// rounded requirement is `Unknown`, and no tolerance makes it otherwise.
#[test]
fn the_three_named_forms_are_never_equated_by_name() {
    let registry = RegisteredImplicationRegistry::standard().expect("registers");
    let exact = bounded_contract(AccuracyContractForm::CorrectlyRounded {
        rounding: ReferenceRoundingRule::NearestTiesToEven,
    });
    let faithful = bounded_contract(AccuracyContractForm::Faithful);
    let one_ulp = ulp_contract(ExactTolerance::from_integer(1));

    assert!(refines(&exact, &faithful, &registry).is_physically_feasible());
    assert!(refines(&exact, &one_ulp, &registry).is_physically_feasible());
    assert!(refines(&faithful, &one_ulp, &registry).is_physically_feasible());

    for (candidate, required) in [
        (&faithful, &exact),
        (&one_ulp, &exact),
        (&one_ulp, &faithful),
    ] {
        let outcome = refines(candidate, required, &registry);
        assert!(
            !outcome.is_physically_feasible(),
            "{outcome:?} equated two distinct forms"
        );
    }

    // A half-ULP requirement is what correctly rounded reaches; faithful is not.
    let half_ulp = ulp_contract(ExactTolerance::from_ratio(1, 2).expect("valid"));
    assert!(refines(&exact, &half_ulp, &registry).is_physically_feasible());
    assert!(!refines(&faithful, &half_ulp, &registry).is_physically_feasible());
}

/// Without the registered rows, nothing but identity refines.
#[test]
fn an_empty_registry_makes_every_cross_form_refinement_unknown() {
    let empty = RegisteredImplicationRegistry::empty();
    let exact = bounded_contract(AccuracyContractForm::CorrectlyRounded {
        rounding: ReferenceRoundingRule::NearestTiesToEven,
    });
    let faithful = bounded_contract(AccuracyContractForm::Faithful);
    assert!(matches!(
        refines(&exact, &faithful, &empty),
        RefinementOutcome::Unknown {
            reason: RefinementUnknown::NoImplicationBetweenForms { .. }
        }
    ));
    // Identity still refines, because it needs no implication at all.
    assert!(refines(&exact, &exact, &empty).is_physically_feasible());
}

/// A cross-metric bound needs a registered implication, not a matching spelling.
///
/// This is the exact shape the Metal guarantee takes: `<= 4 ulp` under Apple's
/// own definition of `ulp`. It is `Unknown` until someone derives and registers
/// the relation, and registering it makes the same comparison decide.
#[test]
fn a_cross_metric_implication_must_be_registered_rather_than_name_matched() {
    let apple = AccuracyMetricKey::new("apple", "msl-ulp", 1).expect("valid");
    let candidate = {
        let clause = whole_domain_clause(AccuracyPredicate::ulp(
            apple.clone(),
            ExactTolerance::from_integer(4),
        ))
        .expect("well formed");
        bounded_contract(AccuracyContractForm::BoundedPiecewise(
            AccuracyDomain::new([DomainInterval::unbounded()], [clause]).expect("well formed"),
        ))
    };
    let required = ulp_contract(ExactTolerance::from_integer(4));

    let mut registry = RegisteredImplicationRegistry::standard().expect("registers");
    assert_eq!(
        refines(&candidate, &required, &registry),
        RefinementOutcome::Unknown {
            reason: RefinementUnknown::UnregisteredMetricImplication {
                from: apple.clone(),
                to: ulp_reference_gap_metric_key(),
            }
        },
        "the standard registry must carry no cross-metric row"
    );

    registry.register(
        RegisteredImplicationKey::new("test", "msl-ulp-agrees", 1).expect("valid"),
        RegisteredImplication::ScaledMetric {
            from: apple,
            to: ulp_reference_gap_metric_key(),
            factor: ExactTolerance::from_integer(1),
        },
        NormativeDefinitionRef::new("a derivation showing the two definitions agree here")
            .expect("bounded"),
    );
    assert!(refines(&candidate, &required, &registry).is_physically_feasible());

    // The factor is applied, so a scale that widens the bound past the
    // requirement is still refused.
    let mut widening = RegisteredImplicationRegistry::standard().expect("registers");
    widening.register(
        RegisteredImplicationKey::new("test", "msl-ulp-is-twice", 1).expect("valid"),
        RegisteredImplication::ScaledMetric {
            from: AccuracyMetricKey::new("apple", "msl-ulp", 1).expect("valid"),
            to: ulp_reference_gap_metric_key(),
            factor: ExactTolerance::from_integer(2),
        },
        NormativeDefinitionRef::new("a derivation showing one Apple ULP is two Tiler ULPs")
            .expect("bounded"),
    );
    assert!(!refines(&candidate, &required, &widening).is_physically_feasible());
}

/// A different signature, reference, or exceptional contract is never a refinement.
#[test]
fn refinement_requires_the_same_subject() {
    let registry = RegisteredImplicationRegistry::standard().expect("registers");
    let base = ulp_contract(ExactTolerance::from_integer(4));
    let other_operation = AccuracyContract::new(
        OpKey::new("test", "other-f32", 1).expect("valid"),
        base.operand_types().to_vec(),
        base.result_type().clone(),
        base.reference_semantics().clone(),
        base.form().clone(),
        base.exceptional(),
    );
    assert_eq!(
        refines(&base, &other_operation, &registry),
        RefinementOutcome::Unknown {
            reason: RefinementUnknown::DifferentSignature
        }
    );
    let other_reference = AccuracyContract::new(
        base.operation().clone(),
        base.operand_types().to_vec(),
        base.result_type().clone(),
        NormativeDefinitionRef::new("a different reference").expect("bounded"),
        base.form().clone(),
        base.exceptional(),
    );
    assert_eq!(
        refines(&base, &other_reference, &registry),
        RefinementOutcome::Unknown {
            reason: RefinementUnknown::DifferentReferenceSemantics
        }
    );
    let other_exceptional = AccuracyContract::new(
        base.operation().clone(),
        base.operand_types().to_vec(),
        base.result_type().clone(),
        base.reference_semantics().clone(),
        base.form().clone(),
        ExceptionalValueContract::new(
            NanReferenceRule::Refuse,
            InfiniteReferenceRule::SignedInfinity,
            DomainErrorRule::CanonicalNan,
            FiniteOverflowRule::SignedInfinity,
        ),
    );
    assert_eq!(
        refines(&base, &other_exceptional, &registry),
        RefinementOutcome::Unknown {
            reason: RefinementUnknown::DifferentExceptionalValueContract
        }
    );
}

/// Bounded clauses are matched by the region they constrain, not by position.
///
/// Two contracts stating the same clause set in opposite order are the same
/// contract, and two clauses that agree on operand zero and differ on operand one
/// are different regions. Matching by position gets the first wrong; matching by
/// operand zero alone gets the second wrong, and would carry a bound out of the
/// region where it was proved.
#[test]
fn bounded_clauses_are_matched_by_region_rather_than_by_position() {
    let registry = RegisteredImplicationRegistry::standard().expect("registers");
    let zero = OperandOrdinal::new(0);
    let one = OperandOrdinal::new(1);
    let below = |ordinal| {
        DomainInterval::new(
            ordinal,
            DomainBound::Unbounded,
            DomainBound::Open(ExactRational::zero()),
        )
        .expect("nonempty")
    };
    let above = |ordinal| {
        DomainInterval::new(
            ordinal,
            DomainBound::Closed(ExactRational::zero()),
            DomainBound::Unbounded,
        )
        .expect("nonempty")
    };
    let clause = |first: DomainInterval, second: DomainInterval, tolerance: u64| {
        AccuracyDomainClause::new(
            [(zero, first), (one, second)],
            ReferenceResultConstraint::unconstrained(),
            AccuracyPredicate::ulp(
                ulp_reference_gap_metric_key(),
                ExactTolerance::from_integer(tolerance),
            ),
        )
        .expect("well formed")
    };
    let contract = |clauses: [AccuracyDomainClause; 2]| {
        bounded_contract(AccuracyContractForm::BoundedPiecewise(
            AccuracyDomain::new(
                [DomainInterval::unbounded(), DomainInterval::unbounded()],
                clauses,
            )
            .expect("well formed"),
        ))
    };

    // The same two regions, written in opposite order, with tighter bounds.
    let required = contract([
        clause(below(zero), above(one), 4),
        clause(above(zero), above(one), 8),
    ]);
    let reordered = contract([
        clause(above(zero), above(one), 4),
        clause(below(zero), above(one), 2),
    ]);
    assert!(
        refines(&reordered, &required, &registry).is_physically_feasible(),
        "clause order must not change what a contract means"
    );

    // The same operand-zero regions, differing only on operand one: not the same
    // regions, so no refinement is established.
    let shifted = contract([
        clause(below(zero), below(one), 2),
        clause(above(zero), below(one), 4),
    ]);
    assert_eq!(
        refines(&shifted, &required, &registry),
        RefinementOutcome::Unknown {
            reason: RefinementUnknown::DifferentDomains
        }
    );
}

/// The closed Boolean algebra establishes what it can and refuses what it cannot.
#[test]
fn the_closed_boolean_algebra_establishes_only_what_it_proves() {
    let registry = RegisteredImplicationRegistry::standard().expect("registers");
    let build = |predicate: AccuracyPredicate| {
        let clause = whole_domain_clause(predicate).expect("well formed");
        bounded_contract(AccuracyContractForm::BoundedPiecewise(
            AccuracyDomain::new([DomainInterval::unbounded()], [clause]).expect("well formed"),
        ))
    };
    let tight = AccuracyPredicate::ulp(
        ulp_reference_gap_metric_key(),
        ExactTolerance::from_integer(1),
    );
    let loose = AccuracyPredicate::ulp(
        ulp_reference_gap_metric_key(),
        ExactTolerance::from_integer(4),
    );
    let absolute = AccuracyPredicate::absolute(ExactTolerance::from_integer(1));

    // A conjunction on the candidate side implies any member it contains.
    assert!(
        refines(
            &build(AccuracyPredicate::all_of([tight.clone(), absolute.clone()]).expect("valid")),
            &build(loose.clone()),
            &registry
        )
        .is_physically_feasible()
    );
    // A conjunction on the required side needs every member proved.
    assert!(
        !refines(
            &build(tight.clone()),
            &build(AccuracyPredicate::all_of([loose.clone(), absolute.clone()]).expect("valid")),
            &registry
        )
        .is_physically_feasible()
    );
    // A disjunction on the required side needs one member proved.
    assert!(
        refines(
            &build(tight),
            &build(AccuracyPredicate::any_of([loose, absolute]).expect("valid")),
            &registry
        )
        .is_physically_feasible()
    );
}

// --- conformance evidence -------------------------------------------------

/// Only the first three classes discharge a hard requirement, and the loop counts both sides.
#[test]
fn only_provable_evidence_discharges_a_hard_requirement() {
    let text = |value: &str| NormativeDefinitionRef::new(value).expect("bounded");
    let build = |class: ConformanceEvidenceClass| {
        let measures = matches!(
            class,
            ConformanceEvidenceClass::ExhaustiveFinite
                | ConformanceEvidenceClass::EmpiricalQualification
        );
        ConformanceEvidence::new(
            class,
            text("exp at f32 on the ordinary domain"),
            text("a target"),
            text("an implementation"),
            text("a toolchain"),
            Some(text("a device")),
            measures.then(|| text("an oracle")),
            measures.then(|| text("a corpus")),
            [0xaa],
        )
        .expect("a well formed record")
    };
    let mut discharged = Vec::new();
    let mut refused = Vec::new();
    for class in ConformanceEvidenceClass::ALL {
        let record = build(class);
        match record.discharge() {
            Ok(_) => discharged.push(class),
            Err(error) => {
                assert_eq!(
                    error.diagnostic_code(),
                    "accuracy.evidence.class-cannot-discharge"
                );
                refused.push(class);
            }
        }
    }
    assert_eq!(
        discharged,
        vec![
            ConformanceEvidenceClass::FormalProof,
            ConformanceEvidenceClass::ExhaustiveFinite,
            ConformanceEvidenceClass::NormativeGuarantee,
        ]
    );
    assert_eq!(
        refused,
        vec![
            ConformanceEvidenceClass::EmpiricalQualification,
            ConformanceEvidenceClass::Unknown,
        ]
    );
    assert_eq!(
        discharged.len() + refused.len(),
        ConformanceEvidenceClass::ALL.len()
    );
}

/// A measurement with no oracle, no corpus, or no digest is not a record.
#[test]
fn an_irreproducible_measurement_is_refused() {
    let text = |value: &str| NormativeDefinitionRef::new(value).expect("bounded");
    let build = |oracle: Option<NormativeDefinitionRef>,
                 corpus: Option<NormativeDefinitionRef>,
                 digest: &[u8]| {
        ConformanceEvidence::new(
            ConformanceEvidenceClass::EmpiricalQualification,
            text("a scope"),
            text("a target"),
            text("an implementation"),
            text("a toolchain"),
            None,
            oracle,
            corpus,
            digest,
        )
    };
    assert_eq!(
        build(None, Some(text("a corpus")), &[1])
            .expect_err("no oracle")
            .diagnostic_code(),
        "accuracy.evidence.missing-reference-oracle"
    );
    assert_eq!(
        build(Some(text("an oracle")), None, &[1])
            .expect_err("no corpus")
            .diagnostic_code(),
        "accuracy.evidence.missing-corpus"
    );
    assert_eq!(
        build(Some(text("an oracle")), Some(text("a corpus")), &[])
            .expect_err("no digest")
            .diagnostic_code(),
        "accuracy.evidence.malformed-digest"
    );
    assert!(build(Some(text("an oracle")), Some(text("a corpus")), &[1]).is_ok());
}

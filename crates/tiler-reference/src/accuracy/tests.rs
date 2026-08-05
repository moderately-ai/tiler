//! The enclosure's bracket, and the decision's ability to say no.
//!
//! The tests that matter here are the ones that perturb the enclosure. An
//! enclosure is only useful if it can be *wrong in the safe direction* — too wide
//! to decide — and if the decision reports that rather than resolving it. Every
//! degradation below starts from a precision that decides and lowers it until the
//! answer changes, so the change is attributable to the enclosure rather than to
//! the case.

use super::*;

use tiler_ir::semantic::accuracy::{
    AccuracyContract, AccuracyContractForm, AccuracyDomain, AccuracyDomainClause,
    AccuracyPredicate, DomainErrorRule, DomainInterval, ExactTolerance, ExceptionalValueContract,
    FiniteOverflowRule, InfiniteReferenceRule, NanReferenceRule, OperandOrdinal,
    ReferenceResultClass, ReferenceResultConstraint, ReferenceRoundingRule,
    ulp_reference_gap_metric_key,
};
use tiler_ir::semantic::{F32, NormativeDefinitionRef, OpKey, builtin_scalar_value_type_facts};

fn format() -> UlpFormat {
    UlpFormat::from_value_type_facts(
        &builtin_scalar_value_type_facts(&F32::resolved_type()).expect("a governed scalar"),
    )
    .expect("f32 carries the metric")
}

fn corpus_precision() -> EnclosurePrecision {
    EnclosurePrecision::binary32_corpus()
}

/// The bounded corpus the three L3′ verticals reach for the exponential.
///
/// The band boundaries are the L3′ record's measured ones: the F32 overflow
/// threshold near `+88.72`, the subnormal onset near `-87.34`, and the exact-zero
/// onset near `-103.97`. A corpus without them reported uniform agreement in that
/// record and hid a real disagreement, which is why they are here.
fn exponential_corpus() -> Vec<ExactRational> {
    [0_i128, 1, -1, 2, -2, 10, -10, 44, -44, 88, -88, 100, -100]
        .into_iter()
        .map(ExactRational::from_integer)
        .chain([
            ExactRational::from_ratio(1, 2).expect("valid"),
            ExactRational::from_ratio(-1, 2).expect("valid"),
            ExactRational::from_ratio(1, 1024).expect("valid"),
            ExactRational::from_ratio(-8873, 100).expect("valid"),
        ])
        .collect()
}

/// Every corpus enclosure brackets, and the bracket is self-consistent.
///
/// `exp(x) * exp(-x) = 1` is an identity the enclosure arithmetic must contain.
/// It is a genuine check on soundness rather than a restatement of the code: an
/// enclosure whose tail bound were too narrow would produce a product interval
/// that misses one, and this would catch it.
#[test]
fn every_corpus_enclosure_brackets_and_is_self_consistent() {
    let one = ExactRational::one();
    let mut checked = 0_usize;
    for argument in exponential_corpus() {
        let positive = exp_enclosure(&argument, corpus_precision()).expect("in range");
        let negative = exp_enclosure(&argument.negate(), corpus_precision()).expect("in range");
        assert!(positive.lower() <= positive.upper(), "{argument}: inverted");
        assert!(
            !positive.lower().is_negative() && !positive.lower().is_zero(),
            "{argument}: the exponential is strictly positive"
        );
        let product = positive.multiply(&negative);
        assert!(
            *product.lower() <= one && one <= *product.upper(),
            "{argument}: exp(x) * exp(-x) must bracket one, got [{}, {}]",
            product.lower(),
            product.upper()
        );
        checked += 1;
    }
    assert_eq!(
        checked,
        exponential_corpus().len(),
        "the loop must visit every corpus argument"
    );
}

/// The enclosure is monotone and pins the two values that are exactly known.
#[test]
fn the_enclosure_is_monotone_and_exact_where_the_value_is() {
    let precision = corpus_precision();
    assert_eq!(
        exp_enclosure(&ExactRational::zero(), precision).expect("in range"),
        CertifiedEnclosure::exact(ExactRational::one())
    );
    let mut previous: Option<CertifiedEnclosure> = None;
    for step in -8_i128..=8 {
        let enclosure =
            exp_enclosure(&ExactRational::from_integer(step), precision).expect("in range");
        if let Some(previous) = previous {
            assert!(
                previous.upper() < enclosure.lower(),
                "the exponential is strictly increasing at {step}"
            );
        }
        previous = Some(enclosure);
    }
}

/// The enclosure brackets `e` to more digits than a host double carries.
///
/// The bounds are the decimal expansion of Euler's number, which is a claim about
/// a mathematical constant rather than about this code: `e =
/// 2.718281828459045235360287471352662497757...`.
#[test]
fn the_enclosure_of_one_brackets_eulers_number() {
    let enclosure = exp_enclosure(&ExactRational::one(), corpus_precision()).expect("in range");
    let lower = ExactRational::from_ratio(2_718_281_828_459_045_235, 1_000_000_000_000_000_000)
        .expect("valid");
    let upper = ExactRational::from_ratio(2_718_281_828_459_045_236, 1_000_000_000_000_000_000)
        .expect("valid");
    assert!(
        *enclosure.lower() >= lower && *enclosure.upper() <= upper,
        "the enclosure [{}, {}] must sit inside the known decimal bracket",
        enclosure.lower(),
        enclosure.upper()
    );
}

/// The reciprocal square root brackets, and its square times the argument is one.
#[test]
fn the_reciprocal_square_root_brackets() {
    let one = ExactRational::one();
    for value in [1_i128, 2, 4, 9, 1024, 1_000_003] {
        let argument = ExactRational::from_integer(value);
        let enclosure = rsqrt_enclosure(&argument, corpus_precision()).expect("positive");
        let product = enclosure
            .multiply(&enclosure)
            .multiply(&CertifiedEnclosure::exact(argument.clone()));
        assert!(
            *product.lower() <= one && one <= *product.upper(),
            "{argument}: rsqrt(x)^2 * x must bracket one"
        );
    }
    assert_eq!(
        rsqrt_enclosure(&ExactRational::zero(), corpus_precision())
            .expect_err("zero is outside the domain")
            .diagnostic_code(),
        "reference.enclosure.outside-domain"
    );
    assert_eq!(
        rsqrt_enclosure(&ExactRational::from_integer(-1), corpus_precision())
            .expect_err("a negative radicand")
            .diagnostic_code(),
        "reference.enclosure.outside-domain"
    );
}

/// An argument whose exponential exceeds the governed magnitude is refused.
///
/// Asserted at the exact boundary rather than only far past it: the greatest
/// admitted argument must still bracket and its successor must refuse, so a change
/// that quietly narrowed the domain fails here rather than passing by refusing
/// more. `2^16` and `2^40` are past it on the same code, and are the reason the
/// bound moved — the superseded halving bound admitted every binade below 23, and
/// timed at this precision `2^16` cost 247× the greatest argument the oracle
/// itself presents while `2^22` did not finish in nine minutes.
///
/// Both signs are watched because the negative branch reaches the bound by
/// recursion rather than by its own check, and a refusal that only fired on the
/// positive side would leave the expensive half of the domain open.
#[test]
fn an_over_large_argument_is_refused() {
    let refused = [
        ExactRational::from_integer(MAX_ARGUMENT_MAGNITUDE + 1),
        ExactRational::power_of_two(16),
        ExactRational::power_of_two(40),
    ];
    let mut checked = 0_usize;
    for argument in &refused {
        for signed in [argument.clone(), argument.negate()] {
            // The outcome is read as a code rather than unwrapped, because a
            // regression here returns an enclosure whose endpoints are thousands
            // of digits wide and `expect_err` would render the whole of one.
            assert_eq!(
                exp_enclosure(&signed, corpus_precision())
                    .as_ref()
                    .err()
                    .map(EnclosureError::diagnostic_code),
                Some("reference.enclosure.argument-too-large"),
                "{signed} is past the governed result magnitude"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 2 * refused.len(), "both signs of every argument");

    let greatest_admitted = ExactRational::from_integer(MAX_ARGUMENT_MAGNITUDE);
    assert!(
        exp_enclosure(&greatest_admitted, corpus_precision()).is_ok(),
        "the greatest admitted argument must still bracket"
    );
    assert!(
        exp_enclosure(&greatest_admitted.negate(), corpus_precision()).is_ok(),
        "and so must its negation, which brackets through the reciprocal"
    );
}

/// The admitted argument bound is the result budget expressed in argument units.
///
/// Two constants that must move together, checked against a bracketing pair for
/// `log2 e = 1.44269504088896340735992...` rather than by restating the product
/// this module already asserts in prose. Raising either alone fails here: a wider
/// [`MAX_ARGUMENT_MAGNITUDE`] admits a result past the budget, and a wider
/// [`MAX_RESULT_MAGNITUDE_BITS`] leaves an argument bound that is no longer the
/// greatest the budget allows.
#[test]
fn the_argument_bound_is_the_result_budget_in_argument_units() {
    let lower = ExactRational::from_ratio(1_442_695_040_888_963_407, 1_000_000_000_000_000_000)
        .expect("valid");
    let upper = ExactRational::from_ratio(1_442_695_040_888_963_408, 1_000_000_000_000_000_000)
        .expect("valid");
    let budget = ExactRational::from_integer(i128::from(MAX_RESULT_MAGNITUDE_BITS));
    assert!(
        ExactRational::from_integer(MAX_ARGUMENT_MAGNITUDE).multiply(&upper) <= budget,
        "the greatest admitted argument's result must fit the budget"
    );
    assert!(
        ExactRational::from_integer(MAX_ARGUMENT_MAGNITUDE + 1).multiply(&lower) > budget,
        "and it must be the greatest such argument, not merely one of them"
    );
}

/// A coarse grid cannot bracket a tiny root away from zero, and says so.
///
/// The perturbation isolates the grid: the same argument brackets at the corpus
/// precision and refuses at a four-bit one, where the integer square root of a
/// value below `2^-8` floors to zero and the reciprocal has no bracket to invert.
/// A clamp here would invent a bound the arithmetic did not establish.
#[test]
fn a_grid_too_coarse_to_bracket_is_refused() {
    let tiny = ExactRational::power_of_two(-20);
    assert!(rsqrt_enclosure(&tiny, corpus_precision()).is_ok());
    let error = rsqrt_enclosure(&tiny, EnclosurePrecision::new(4).expect("a stateable grid"))
        .expect_err("the grid cannot bracket");
    assert_eq!(
        error.diagnostic_code(),
        "reference.enclosure.precision-too-coarse"
    );
}

/// A precision the series cannot reach within its term bound is refused.
///
/// The cap exists because a loop bounded only by its own convergence test cannot
/// report that it failed to converge. Asking for a grid far finer than the
/// truncated series can supply is what makes it say so.
///
/// **The grid is derived rather than picked, because the reduction depth moves
/// it.** The final term the cap admits is `T_512 = y^512 / 512!` at `y <= 2^-k`,
/// which is at or below `2^-(512 * k + log2(512!))` — about `2^-8483` at the
/// depth this module reduces to — so a grid below that is reached and one above
/// it is not. A depth that reached further would make this test pass by
/// succeeding rather than by refusing, which is why the number is stated with its
/// derivation instead of as a constant that once worked.
#[test]
fn a_precision_the_series_cannot_reach_is_refused() {
    let error = exp_enclosure(
        &ExactRational::one(),
        EnclosurePrecision::new(12_000).expect("a stateable grid"),
    )
    .expect_err("512 terms cannot reach a 12000-bit grid");
    assert_eq!(
        error.diagnostic_code(),
        "reference.enclosure.precision-unreachable"
    );

    // The reachable neighbour, so the refusal is attributable to the grid rather
    // than to the argument: the same argument at a grid the series does reach.
    assert!(
        exp_enclosure(
            &ExactRational::one(),
            EnclosurePrecision::new(4_000).expect("a stateable grid"),
        )
        .is_ok()
    );
}

/// A grid width the exponent arithmetic cannot express cannot be stated at all.
///
/// The first of the two layers, and the one that makes the second unreachable: a
/// width past the bound is refused where it is written, so no enclosure function
/// can be handed one. `u32::MAX` and `2_147_483_646` are the two widths that
/// aborted the process before this bound existed.
///
/// The admitted neighbour is checked here at construction — the stated width and
/// the finer grid derived from it are both exponents `i32` can carry, which is
/// exactly what the bound claims — and separately through a whole call, in
/// [`the_widest_admitted_grid_refuses_rather_than_aborting`].
#[test]
fn a_grid_width_the_arithmetic_cannot_express_is_refused() {
    let refused = [
        u32::MAX,
        2_147_483_646,
        MAX_GRID_FRACTION_BITS.saturating_add(1),
    ];
    let mut checked = 0_usize;
    for width in refused {
        assert_eq!(
            EnclosurePrecision::new(width)
                .as_ref()
                .err()
                .map(EnclosureError::diagnostic_code),
            Some("reference.enclosure.grid-width-unrepresentable"),
            "a {width}-bit grid is past what the exponent arithmetic can express"
        );
        checked += 1;
    }
    assert_eq!(checked, refused.len(), "every rejected width was checked");

    let widest =
        EnclosurePrecision::new(MAX_GRID_FRACTION_BITS).expect("the widest stateable grid");
    assert_eq!(widest.fraction_bits(), MAX_GRID_FRACTION_BITS);
    assert!(
        i32::try_from(widest.squaring_grid().fraction_bits()).is_ok(),
        "the grid derived from the widest admitted one must still be an exponent"
    );
}

/// The widest grid the type admits is refused end to end, not aborted.
///
/// The closing criterion as a call rather than as an argument. Every conversion
/// the old code asserted on is exercised at this width — the tail threshold's
/// negated exponent, and the squaring grid derived from it — and what comes back
/// is the series' own refusal, because `MAX_SERIES_TERMS` stops long before a
/// two-billion-bit grid. It is the one place the module's totality claim is
/// observed at its extreme instead of derived, which is worth the two seconds and
/// the quarter-gigabyte threshold it allocates to get there.
#[test]
fn the_widest_admitted_grid_refuses_rather_than_aborting() {
    let widest =
        EnclosurePrecision::new(MAX_GRID_FRACTION_BITS).expect("the widest stateable grid");
    assert_eq!(
        exp_enclosure(&ExactRational::one(), widest)
            .as_ref()
            .err()
            .map(EnclosureError::diagnostic_code),
        Some("reference.enclosure.precision-unreachable"),
        "the term bound is what stops the widest admitted grid, not an abort"
    );
}

/// The stated bound is `i32`'s limit less the margin the squaring chain needs.
///
/// Two constants that must move together, on the pattern
/// [`the_argument_bound_is_the_result_budget_in_argument_units`] uses for the
/// argument axis. Raising `MAX_GRID_FRACTION_BITS` alone admits a precision whose
/// squaring grid the arithmetic cannot express; raising `SQUARING_GRID_MARGIN`
/// alone leaves a bound that is no longer the widest such precision. The equality
/// is what makes the refusal a property of the arithmetic rather than a number
/// someone chose.
#[test]
fn the_grid_bound_is_the_exponent_limit_less_the_squaring_margin() {
    assert_eq!(
        MAX_GRID_FRACTION_BITS + SQUARING_GRID_MARGIN,
        i32::MAX.unsigned_abs(),
        "the bound must leave exactly the squaring margin below the exponent limit"
    );
    assert_eq!(
        SQUARING_GRID_MARGIN,
        REDUCED_ARGUMENT_BITS + 2,
        "the margin is the reduction depth plus the threshold's two bits"
    );
}

// --- the conformance decision --------------------------------------------

fn four_ulp() -> AccuracyPredicate {
    AccuracyPredicate::ulp(
        ulp_reference_gap_metric_key(),
        ExactTolerance::from_integer(4),
    )
}

/// A correctly rounded candidate conforms and a far-off one violates.
///
/// Both answers on the same reference, so a check that could only ever say one
/// thing would fail here.
#[test]
fn the_decision_says_both_yes_and_no() {
    let format = format();
    let reference = exp_enclosure(&ExactRational::one(), corpus_precision()).expect("in range");
    let rounded = format
        .round_to_nearest_ties_even(reference.lower())
        .expect("in range");
    assert_eq!(
        decide_predicate(&four_ulp(), &format, &reference, &rounded),
        ConformanceDecision::Conforms
    );

    // A candidate one binade away is not within four ULPs of anything.
    let far = rounded.scale_by_power_of_two(1);
    assert_eq!(
        decide_predicate(&four_ulp(), &format, &reference, &far),
        ConformanceDecision::Violates
    );

    // And the boundary is where the tolerance says it is: five ULPs away
    // violates a four-ULP bound and satisfies a six-ULP one.
    let spacing = format.ulp_scale(&rounded).expect("in range");
    let five_ulps_off = rounded.add(&spacing.multiply(&ExactRational::from_integer(5)));
    assert_eq!(
        decide_predicate(&four_ulp(), &format, &reference, &five_ulps_off),
        ConformanceDecision::Violates
    );
    let six_ulp = AccuracyPredicate::ulp(
        ulp_reference_gap_metric_key(),
        ExactTolerance::from_integer(6),
    );
    assert_eq!(
        decide_predicate(&six_ulp, &format, &reference, &five_ulps_off),
        ConformanceDecision::Conforms
    );
}

/// Degrading the enclosure turns a decided case undecided, not a silent pass.
///
/// **The failure proof for the enclosure itself.** The candidate is placed exactly
/// on the tolerance boundary, where the answer depends entirely on how narrow the
/// bracket is. At the corpus precision the decision is definite; with a grid two
/// bits wide the bracket straddles the tolerance and the decision is `Undecided`.
/// If the enclosure's tail bound were treated as tight — or if the comparison
/// resolved its own uncertainty — the coarse case would report `Conforms` and the
/// check would be one that cannot fail.
#[test]
fn a_degraded_enclosure_yields_undecided_rather_than_a_silent_pass() {
    let format = format();
    let precise = exp_enclosure(&ExactRational::one(), corpus_precision()).expect("in range");
    let rounded = format
        .round_to_nearest_ties_even(precise.lower())
        .expect("in range");
    let spacing = format.ulp_scale(&rounded).expect("in range");
    // Four ULPs above the reference's own lower bound, which is a hair inside the
    // four-ULP tolerance at the precise enclosure and ambiguous at a coarse one.
    let candidate = precise
        .lower()
        .add(&spacing.multiply(&ExactRational::from_integer(4)));

    let decided = decide_predicate(&four_ulp(), &format, &precise, &candidate);
    assert_ne!(
        decided,
        ConformanceDecision::Undecided {
            reason: UndecidedConformance::EnclosureTooWide
        },
        "the corpus precision must decide this case"
    );

    let coarse = exp_enclosure(
        &ExactRational::one(),
        EnclosurePrecision::new(2).expect("a stateable grid"),
    )
    .expect("still in range at a coarse grid");
    assert!(
        coarse.width() > precise.width(),
        "a coarser grid must widen the bracket"
    );
    let undecided = decide_predicate(&four_ulp(), &format, &coarse, &candidate);
    assert_eq!(
        undecided,
        ConformanceDecision::Undecided {
            reason: UndecidedConformance::EnclosureTooWide
        },
        "a bracket that straddles the tolerance must not resolve itself"
    );
    assert!(
        !undecided.conforms(),
        "an undecided comparison is not a pass"
    );
}

/// A relative predicate over a bracket that straddles zero is undecided, not divided.
#[test]
fn a_relative_predicate_at_an_unproven_zero_is_undecided() {
    let format = format();
    let straddling = CertifiedEnclosure::new(
        ExactRational::from_ratio(-1, 1024).expect("valid"),
        ExactRational::from_ratio(1, 1024).expect("valid"),
    );
    assert_eq!(
        decide_predicate(
            &AccuracyPredicate::relative(ExactTolerance::from_integer(1)),
            &format,
            &straddling,
            &ExactRational::zero(),
        ),
        ConformanceDecision::Undecided {
            reason: UndecidedConformance::ReferenceNotProvablyNonzero
        }
    );
}

/// A foreign metric is not evaluated under Tiler's, it is reported unsupported.
#[test]
fn a_foreign_metric_is_reported_rather_than_substituted() {
    use tiler_ir::semantic::accuracy::AccuracyMetricKey;
    let format = format();
    let reference = CertifiedEnclosure::exact(ExactRational::one());
    let foreign = AccuracyPredicate::ulp(
        AccuracyMetricKey::new("apple", "msl-ulp", 1).expect("valid"),
        ExactTolerance::from_integer(4),
    );
    assert_eq!(
        decide_predicate(&foreign, &format, &reference, &ExactRational::one()),
        ConformanceDecision::Undecided {
            reason: UndecidedConformance::UnsupportedMetric
        }
    );
}

/// A NaN or infinite candidate has no exact value and never reaches the comparison.
#[test]
fn a_non_finite_candidate_has_no_exact_value() {
    assert_eq!(exact_binary32_candidate(f32::NAN), None);
    assert_eq!(exact_binary32_candidate(f32::NEG_INFINITY), None);
    assert_eq!(
        exact_binary32_candidate(1.5),
        Some(ExactRational::from_ratio(3, 2).expect("valid"))
    );
}

// --- deciding a whole contract -------------------------------------------

fn contract(form: AccuracyContractForm) -> AccuracyContract {
    AccuracyContract::new(
        OpKey::new("test", "exp-f32", 1).expect("valid"),
        vec![F32::resolved_type()],
        F32::resolved_type(),
        NormativeDefinitionRef::new("the exponential function on the reals").expect("bounded"),
        form,
        ExceptionalValueContract::new(
            NanReferenceRule::CanonicalNan,
            InfiniteReferenceRule::SignedInfinity,
            DomainErrorRule::CanonicalNan,
            FiniteOverflowRule::SignedInfinity,
        ),
    )
}

fn bounded_contract(predicate: AccuracyPredicate) -> AccuracyContract {
    let clause = AccuracyDomainClause::new(
        [(OperandOrdinal::new(0), DomainInterval::unbounded())],
        ReferenceResultConstraint::new(
            [ReferenceResultClass::Positive],
            None,
            Some(NormativeDefinitionRef::new("exp is strictly positive").expect("bounded")),
        )
        .expect("justified"),
        predicate,
    )
    .expect("well formed");
    contract(AccuracyContractForm::BoundedPiecewise(
        AccuracyDomain::new([DomainInterval::unbounded()], [clause]).expect("well formed"),
    ))
}

/// The three interpretable forms decide differently on the same candidate.
///
/// The candidate is one ULP above the correctly rounded value. That satisfies the
/// four-ULP bound, satisfies faithful rounding only if it is a bracketing
/// neighbour, and never satisfies the correctly rounded contract — which is the
/// separation ADR 0042 forbids collapsing.
#[test]
fn the_forms_decide_the_same_candidate_differently() {
    let format = format();
    let inputs = [ExactRational::one()];
    let reference = exp_enclosure(&inputs[0], corpus_precision()).expect("in range");
    let rounded = format
        .round_to_nearest_ties_even(reference.lower())
        .expect("in range");
    let spacing = format.ulp_scale(&rounded).expect("in range");
    let one_ulp_high = rounded.add(&spacing);

    let exact_form = contract(AccuracyContractForm::CorrectlyRounded {
        rounding: ReferenceRoundingRule::NearestTiesToEven,
    });
    assert_eq!(
        decide_contract(&exact_form, &format, &inputs, &reference, &rounded),
        ConformanceDecision::Conforms
    );
    assert_eq!(
        decide_contract(&exact_form, &format, &inputs, &reference, &one_ulp_high),
        ConformanceDecision::Violates
    );

    let faithful = contract(AccuracyContractForm::Faithful);
    assert_eq!(
        decide_contract(&faithful, &format, &inputs, &reference, &rounded),
        ConformanceDecision::Conforms
    );

    let bounded = bounded_contract(four_ulp());
    assert_eq!(
        decide_contract(&bounded, &format, &inputs, &reference, &one_ulp_high),
        ConformanceDecision::Conforms,
        "one ULP high satisfies a four-ULP bound and not a correctly rounded contract"
    );
}

/// A named-elementary profile is reported uninterpretable rather than guessed.
#[test]
fn a_named_profile_is_not_interpreted_from_its_digest() {
    use tiler_ir::semantic::accuracy::{
        NamedElementaryDescriptorDigest, NamedElementaryProfileKey,
    };
    let named = contract(AccuracyContractForm::NamedElementary {
        profile: NamedElementaryProfileKey::new("vendor", "fast-exp", 1).expect("valid"),
        descriptor_digest: NamedElementaryDescriptorDigest::new([1, 2, 3]).expect("nonempty"),
        descriptor_basis: NormativeDefinitionRef::new("the vendor's fast-math table")
            .expect("bounded"),
    });
    assert_eq!(
        decide_contract(
            &named,
            &format(),
            &[ExactRational::one()],
            &CertifiedEnclosure::exact(ExactRational::one()),
            &ExactRational::one(),
        ),
        ConformanceDecision::Undecided {
            reason: UndecidedConformance::NamedProfileNotInterpretable
        }
    );
}

/// A point no clause reaches is reported rather than defaulted to conforming.
#[test]
fn an_input_no_clause_reaches_is_reported() {
    let clause = AccuracyDomainClause::new(
        [(
            OperandOrdinal::new(0),
            DomainInterval::new(
                OperandOrdinal::new(0),
                tiler_ir::semantic::accuracy::DomainBound::Closed(ExactRational::zero()),
                tiler_ir::semantic::accuracy::DomainBound::Unbounded,
            )
            .expect("nonempty"),
        )],
        ReferenceResultConstraint::unconstrained(),
        four_ulp(),
    )
    .expect("well formed");
    let partial = contract(AccuracyContractForm::BoundedPiecewise(
        AccuracyDomain::new([DomainInterval::unbounded()], [clause]).expect("well formed"),
    ));
    assert_eq!(
        decide_contract(
            &partial,
            &format(),
            &[ExactRational::from_integer(-1)],
            &CertifiedEnclosure::exact(ExactRational::one()),
            &ExactRational::one(),
        ),
        ConformanceDecision::Undecided {
            reason: UndecidedConformance::NoApplicableClause
        }
    );
}

/// The whole decision is exact integer arithmetic, so it cannot move with the profile.
///
/// Not a claim this test can prove on its own — it runs under one profile at a
/// time. What it pins is the value the release-profile run must also produce, so
/// the two runs compare against one recorded bracket rather than against each
/// other.
#[test]
fn the_corpus_bracket_is_pinned_and_profile_independent() {
    let enclosure =
        exp_enclosure(&ExactRational::from_integer(2), corpus_precision()).expect("in range");
    // exp(2) = 7.389056098930650227230427460575...
    let lower = ExactRational::from_ratio(7_389_056_098_930_650_227, 1_000_000_000_000_000_000)
        .expect("valid");
    let upper = ExactRational::from_ratio(7_389_056_098_930_650_228, 1_000_000_000_000_000_000)
        .expect("valid");
    assert!(*enclosure.lower() >= lower && *enclosure.upper() <= upper);
}

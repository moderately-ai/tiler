//! The eviction policy, decided over a stated environment and nothing else.
//!
//! Every test here supplies its own snapshot and its own gate. Nothing reads or
//! mutates the process environment, and no test's outcome depends on whether
//! another ran first — the amortization flag is per-process in production and
//! per-value here, which is the whole reason [`EvictionGate::new`] is public to
//! the crate.

use std::cell::RefCell;
use std::ffi::OsString;
use std::time::Duration;

use tiler_cache::expansion::{CollectionBound, MaxEntryAge, MaxEntryAgeRefusal};

use super::{
    DISABLE_VALUE, EvictionEnvironment, EvictionGate, EvictionPolicy, EvictionRefusal,
    EvictionSchedule, MAX_ENTRY_AGE_VARIABLE, UNITS, resolve, spelled,
};

/// A snapshot stating one value, or none.
fn snapshot(stated: Option<&str>) -> EvictionEnvironment {
    EvictionEnvironment::new(stated.map(OsString::from))
}

/// The bound one age states, spelled the way production spells it.
fn bounded(max_age: Duration) -> EvictionPolicy {
    EvictionPolicy::Bounded(CollectionBound {
        max_total_bytes: None,
        max_entries: None,
        max_entry_age: Some(MaxEntryAge::new(max_age).expect("a non-zero age is a bound")),
    })
}

/// A consumer who configures nothing gets the cache crate's own constant.
///
/// The assertion names [`MaxEntryAge::DEFAULT`] rather than thirty days, because
/// the constant is the authority: a frontend restating the number would be a
/// second one, and the day the cache's ground for it changes this test must
/// follow rather than fail.
#[test]
fn an_unconfigured_host_evicts_under_the_cache_crates_default_age() {
    assert_eq!(
        resolve(&snapshot(None)).expect("an absent statement is not a refusal"),
        bounded(MaxEntryAge::DEFAULT.as_duration()),
    );
}

/// The default states an age and neither aggregate ceiling.
///
/// The paired half of the test above, and not a restatement of it: a bound that
/// also carried a byte or entry ceiling would evict by publication recency,
/// which is a different policy from the age-based one Tom decided on, and the
/// equality above would still hold if the ceilings were filled in from a guess.
#[test]
fn the_automatic_policy_states_an_age_and_no_aggregate_ceiling() {
    let EvictionPolicy::Bounded(bound) = resolve(&snapshot(None)).expect("the default resolves")
    else {
        panic!("an unconfigured host must evict rather than opt out");
    };
    assert_eq!(bound.max_total_bytes, None);
    assert_eq!(bound.max_entries, None);
    assert_eq!(bound.max_entry_age, Some(MaxEntryAge::DEFAULT));
}

/// The documented default is itself a value a consumer could type.
///
/// What this protects is the diagnostics: each one offers the default as a
/// spelling to paste back, so a default the parser could not read would be
/// advice that does not work.
#[test]
fn the_default_spelling_round_trips_through_the_parser() {
    let spelling = spelled(MaxEntryAge::DEFAULT.as_duration());
    assert_eq!(spelling, "30d", "the constant is thirty days");
    assert_eq!(
        resolve(&snapshot(Some(&spelling))).expect("the default's own spelling parses"),
        bounded(MaxEntryAge::DEFAULT.as_duration()),
    );
}

/// The opt-out is exactly one value, and it is the root policy's own word.
#[test]
fn the_opt_out_disables_eviction_exactly() {
    assert_eq!(
        resolve(&snapshot(Some(DISABLE_VALUE))).expect("`off` is a statement, not a failure"),
        EvictionPolicy::Disabled,
    );
    assert_eq!(
        DISABLE_VALUE, "off",
        "the opt-out is the spelling ADR 0089 accepted for the sibling variable",
    );
}

/// A value that merely resembles the opt-out is a refusal, not an opt-out.
///
/// The paired negative of the test above: without it, "matched exactly" is also
/// what a case-folding or trimming comparison would report — and a consumer who
/// wrote `OFF` expecting no eviction would instead get one under a bound they
/// did not choose, which is the one outcome this module exists to prevent.
#[test]
fn a_value_resembling_the_opt_out_is_refused_rather_than_read_as_one() {
    for near in ["OFF", "Off", "off ", " off", "offf", "0ff"] {
        let refusal = resolve(&snapshot(Some(near)))
            .expect_err("only the exact opt-out disables the eviction");
        assert!(
            matches!(refusal, EvictionRefusal::Malformed { .. }),
            "`{near}` must be malformed rather than an opt-out or a bound: {refusal:?}",
        );
    }
}

/// Every accepted unit states the duration it names.
///
/// Parametrized over the whole table rather than over one member, because the
/// table is the claim: a unit added to it without a matching second of
/// arithmetic would leave a spelling that parses to the wrong age while this
/// test still passed.
#[test]
fn every_accepted_unit_states_the_duration_it_names() {
    assert_eq!(
        UNITS.len(),
        4,
        "the accepted units are counted, not sampled"
    );
    for (suffix, seconds_per_unit) in UNITS {
        let stated = format!("7{suffix}");
        assert_eq!(
            resolve(&snapshot(Some(&stated))).expect("an accepted unit parses"),
            bounded(Duration::from_secs(7 * seconds_per_unit)),
            "`{stated}` must state seven {suffix}",
        );
    }
}

/// Every unusable value refuses the eviction instead of guessing a bound.
///
/// The population is enumerated and counted, and each entry is a value someone
/// plausibly types: a bare count with no unit, a spelled-out unit, a plural, a
/// compound, a sign, a decimal, a separator, an unsupported unit, an uppercase
/// unit, a zero, and a count no duration can hold.
///
/// The assertion is two-sided. Each must refuse, *and* the refusal must be the
/// specific one the value earned — a check that only asked whether an error came
/// back would pass on an implementation that mapped everything to one variant
/// and told a consumer nothing about what to change.
#[test]
fn every_unusable_value_refuses_the_eviction_rather_than_guessing_a_bound() {
    let cases: [(&str, EvictionRefusal); 12] = [
        ("", EvictionRefusal::Empty),
        (
            "30",
            EvictionRefusal::Malformed {
                value: "30".to_owned(),
            },
        ),
        (
            "30 days",
            EvictionRefusal::Malformed {
                value: "30 days".to_owned(),
            },
        ),
        (
            "30days",
            EvictionRefusal::Malformed {
                value: "30days".to_owned(),
            },
        ),
        (
            "1d12h",
            EvictionRefusal::Malformed {
                value: "1d12h".to_owned(),
            },
        ),
        (
            "-1d",
            EvictionRefusal::Malformed {
                value: "-1d".to_owned(),
            },
        ),
        (
            "1.5d",
            EvictionRefusal::Malformed {
                value: "1.5d".to_owned(),
            },
        ),
        (
            "1_000d",
            EvictionRefusal::Malformed {
                value: "1_000d".to_owned(),
            },
        ),
        (
            "4w",
            EvictionRefusal::Malformed {
                value: "4w".to_owned(),
            },
        ),
        (
            "30D",
            EvictionRefusal::Malformed {
                value: "30D".to_owned(),
            },
        ),
        (
            "0d",
            EvictionRefusal::NotABound {
                value: "0d".to_owned(),
                source: MaxEntryAgeRefusal::Zero,
            },
        ),
        (
            "18446744073709551615d",
            EvictionRefusal::TooLarge {
                value: "18446744073709551615d".to_owned(),
            },
        ),
    ];
    assert_eq!(
        cases.len(),
        12,
        "the population this test covers is enumerated and counted",
    );
    for (stated, expected) in cases {
        assert_eq!(
            resolve(&snapshot(Some(stated))).expect_err("an unusable value states no bound"),
            expected,
            "`{stated}` must refuse with the reason it earned",
        );
    }
}

/// A one-second age is accepted, so the refusals above are about the values and
/// not about a floor.
///
/// The cache crate refuses zero and nothing above it, deliberately: a floor
/// would be exactly the guessed number the design record declines to choose.
/// Without this, "every short age refuses" would satisfy the test above too.
#[test]
fn no_floor_sits_above_the_one_refused_age() {
    assert_eq!(
        resolve(&snapshot(Some("1s"))).expect("one second is a legitimate policy"),
        bounded(Duration::from_secs(1)),
    );
}

/// A value the host cannot report as text refuses rather than being read
/// lossily.
#[test]
fn a_value_that_is_not_text_refuses_rather_than_being_read_lossily() {
    use std::os::unix::ffi::OsStringExt as _;

    let value = OsString::from_vec(vec![0x33, 0x30, 0xff, 0x64]);
    assert_eq!(
        resolve(&EvictionEnvironment::new(Some(value.clone())))
            .expect_err("a non-UTF-8 statement is not a duration"),
        EvictionRefusal::NotText { value },
    );
}

/// Every refusal names the variable, the opt-out, and the default, and says
/// nothing was removed.
///
/// A consumer reads exactly one of these lines and gets no document with it, so
/// each has to carry both remedies and the reassurance that the fault cost them
/// no cache. The population is every refusal shape, counted.
#[test]
fn every_refusal_states_the_variable_both_remedies_and_the_consequence() {
    let refusals = [
        EvictionRefusal::Empty,
        EvictionRefusal::NotText {
            value: OsString::from("30x"),
        },
        EvictionRefusal::Malformed {
            value: "30 days".to_owned(),
        },
        EvictionRefusal::NotABound {
            value: "0s".to_owned(),
            source: MaxEntryAgeRefusal::Zero,
        },
        EvictionRefusal::TooLarge {
            value: "99999999999999999999d".to_owned(),
        },
    ];
    assert_eq!(
        refusals.len(),
        5,
        "the population this test covers is every refusal shape, counted",
    );
    for refusal in refusals {
        let rendered = refusal.to_string();
        for required in [
            MAX_ENTRY_AGE_VARIABLE,
            DISABLE_VALUE,
            &spelled(MaxEntryAge::DEFAULT.as_duration()),
            "Nothing was removed",
        ] {
            assert!(
                rendered.contains(required),
                "a refusal must name `{required}`: {rendered}",
            );
        }
    }
}

/// Observation reads exactly the one name the policy is defined over.
///
/// The sibling of `cache_root`'s own check, and it exists for the same reason: a
/// second name is how one cache would come to be trimmed under two policies, and
/// nothing else here could notice one appearing. Both presence cases are
/// covered, because a second read is most naturally written as a *fallback* —
/// read this, and if it is missing read something else — which a present-only
/// case would leave undetected.
#[test]
fn observation_reads_exactly_the_policy_variable() {
    let cases = [None, Some("30d")];
    assert_eq!(
        cases.len(),
        2,
        "the population is every presence combination of one variable, counted",
    );
    for stated in cases {
        let seen = RefCell::new(Vec::new());
        let environment = EvictionEnvironment::observe(|name| {
            seen.borrow_mut().push(name.to_owned());
            assert_eq!(
                name, MAX_ENTRY_AGE_VARIABLE,
                "the policy read an unexpected variable `{name}`",
            );
            stated.map(OsString::from)
        });
        assert_eq!(
            seen.into_inner(),
            vec![MAX_ENTRY_AGE_VARIABLE.to_owned()],
            "with statement {stated:?}",
        );
        assert_eq!(environment, snapshot(stated));
    }
}

/// The amortization rule: one process, one pass, however many publications.
///
/// This is the claim the long-lived analyzer server depends on. It is asserted
/// on the gate rather than through a cache, because what is being stated is that
/// the *permission* is issued once — a test that watched removals would also
/// pass on a gate that admitted every caller against an already-empty cache.
#[test]
fn a_gate_admits_one_sweep_and_one_report_per_process() {
    let gate = EvictionGate::new();
    assert!(gate.claim_sweep(), "the first publication sweeps");
    for attempt in 0..8 {
        assert!(
            !gate.claim_sweep(),
            "publication {attempt} after the first must sweep nothing",
        );
    }
    assert!(gate.claim_report(), "the first refusal reports");
    assert!(
        !gate.claim_report(),
        "a second refusal in one process must stay quiet",
    );

    // A fresh gate is a fresh process, which is what keeps one build's
    // amortization from silencing the next one's.
    assert!(EvictionGate::new().claim_sweep());
}

/// An unusable statement is reported once, names what to change, and yields no
/// bound.
///
/// Written against a stream the test owns rather than the process's standard
/// error, so what a consumer reads is a value that can be asserted on. The
/// second call is the amortization: a rust-analyzer session expanding all
/// afternoon must not repeat one misconfiguration into its log forever.
#[test]
fn an_unusable_statement_reports_once_and_evicts_nothing() {
    let gate = EvictionGate::new();
    let schedule = EvictionSchedule::stated(snapshot(Some("30 days")), &gate);
    let mut written = Vec::new();

    assert_eq!(
        schedule.bound_reported_to(&mut written),
        None,
        "a refusal must state no bound, rather than a guessed one",
    );
    let first = String::from_utf8(written.clone()).expect("the message is text");
    assert!(first.contains(MAX_ENTRY_AGE_VARIABLE), "{first}");
    assert!(first.contains("30 days"), "{first}");
    assert_eq!(first.lines().count(), 1, "one line per process: {first}");

    assert_eq!(schedule.bound_reported_to(&mut written), None);
    assert_eq!(
        String::from_utf8(written).expect("the message is text"),
        first,
        "the second expansion in one process must add nothing",
    );
}

/// A usable statement and the opt-out both stay silent.
///
/// The paired negative of the test above. Without it, "a refusal is reported"
/// would also be what an implementation that narrated every expansion produced,
/// and background hygiene that announces itself on every build is the noise the
/// report disposition was decided against.
#[test]
fn a_usable_statement_and_the_opt_out_report_nothing() {
    for (stated, expected) in [
        (None, Some(MaxEntryAge::DEFAULT.as_duration())),
        (Some("12h"), Some(Duration::from_hours(12))),
        (Some(DISABLE_VALUE), None),
    ] {
        let gate = EvictionGate::new();
        let schedule = EvictionSchedule::stated(snapshot(stated), &gate);
        let mut written = Vec::new();
        let bound = schedule.bound_reported_to(&mut written);
        assert_eq!(
            bound
                .and_then(|bound| bound.max_entry_age)
                .map(MaxEntryAge::as_duration),
            expected,
            "statement {stated:?} must resolve to {expected:?}",
        );
        assert!(
            written.is_empty(),
            "nothing usable may write to a consumer's build output: {}",
            String::from_utf8_lossy(&written),
        );
    }
}

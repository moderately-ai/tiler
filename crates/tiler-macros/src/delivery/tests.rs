//! The stated policy, the named profiles, and the tokens a plan delivers.
//!
//! Four kinds of evidence live here. Exact-text assertions pin what the emitter
//! writes, because the generated code is the product. Evaluation against
//! `rustc --print cfg` decides what that text *means* on a consumer target
//! without needing that target's standard library, which is what lets it cover
//! fifteen of them. The fixture comparisons bind both to files the facade
//! actually compiles, so the compile evidence is evidence about this emitter
//! rather than about text someone wrote on the same day. And
//! [`every_emitted_shape_compiles_as_the_five_target_matrix_says`] compiles the
//! emitter's own output *for* the five targets
//! `docs/correctness-and-testing.md` names, which is the only one of the four in
//! which the compiler that will gate a consumer's build is the thing that
//! decided the answer.
//!
//! # The five-target matrix under real cross-target compilation
//!
//! **Measurement — `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, 2026-08-01,
//! `rustc --edition 2024 --crate-type lib --emit=metadata --target <triple>`
//! over the items [`DeliveryPlan::items_source`] emits, one fixture per shape
//! and target.** Fifteen compilations, and every one agreed with the matrix
//! `the_emitted_arms_select_exactly_one_payload_per_consumer_target` and
//! `a_retained_diagnostic_fires_only_on_the_family_it_names` derive from
//! `rustc --print cfg`:
//!
//! | emitted shape | `aarch64-apple-darwin` | `aarch64-apple-ios` | `aarch64-apple-ios-sim` | `aarch64-apple-ios-macabi` | `x86_64-unknown-linux-gnu` |
//! | --- | --- | --- | --- | --- | --- |
//! | macOS and iOS device both built | payload 1 | payload 0 | fallback | fallback | fallback |
//! | iOS device built, simulator retained | fallback | payload 0 | build fails on the retained diagnostic | fallback | fallback |
//! | macOS retained, nothing built | build fails on the retained diagnostic | compiles, no item survives `#[cfg]` | compiles, no item survives `#[cfg]` | compiles, no item survives `#[cfg]` | compiles, no item survives `#[cfg]` |
//!
//! This replaces an inference with a compilation. The `rustc --print cfg`
//! evidence establishes which predicate holds where; it cannot establish that
//! the emitted *items* are well-formed on a target, that exactly one selector
//! arm survives `#[cfg]` there, or that a byte-string literal of all 256 byte
//! values lexes the same for a non-Apple target. Compiling for the target
//! decides all three at once, because a second surviving arm is a duplicate
//! definition and a gap is an undefined name.
//!
//! **Boundary — check level, no SDK, no link.** `--emit=metadata` is exactly
//! what `cargo check` runs, so `#[cfg]` selection, selector totality, the
//! byte-string literal, and the `const` assertions are decided by rustc for the
//! named target. No linker runs, no Apple SDK is consulted, and nothing is
//! linked or executed. It says the delivered *source* is correct for each
//! target; it says nothing about whether a `metallib` carried in it would load
//! there, which `docs/research/apple-targets/artifact-compatibility.md` owns.

use core::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use tiler_metal_aot::family::{
    ArtifactDeliveryPolicy, ArtifactFamilySelection, FamilyRequirement, FamilySelectionError,
    SelectedFamily,
};
use tiler_metal_aot::input::{
    ApplePlatform, DeploymentMinimum, MetalTarget, MetalTargetError, MslVersion,
};

use super::{
    ARTIFACT_BINDING, DeliveredFamily, DeliveryPlan, DeliveryRefusal, FamilyDelivery, NamedProfile,
    PROFILE_MSL_VERSION, PlanRefusal, SELECTED_PAYLOAD_BINDING, StatementRefusal,
    byte_string_literal, fallback_plan, stated_delivery, stated_policy,
};
use crate::family_cfg::consumer_cfg;
use crate::family_cfg::tests::{evaluate, target_cfg};
use crate::grammar::{
    DeliverySyntax, DeploymentMinimumSyntax, FamilyMinimumSyntax, Name, StatedDelivery,
};

/// A span a test can construct and assert on, as in `crate::grammar::tests`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct At(u32);

/// The `deliver` keyword's span in every statement built below.
const KEYWORD: At = At(1);

fn spelled(text: &str, at: u32) -> Name<At> {
    Name {
        text: text.to_owned(),
        span: At(at),
    }
}

/// `deliver <profile>;`, with the profile name at span `at`.
fn profile_statement(name: &str, at: u32) -> DeliverySyntax<At> {
    DeliverySyntax {
        keyword: KEYWORD,
        stated: StatedDelivery::Profile(spelled(name, at)),
    }
}

/// `deliver <family> <major>.<minor>, …;`, each entry naming its own spans.
fn family_statement(entries: &[(&str, u32, u16, u16, u32)]) -> DeliverySyntax<At> {
    DeliverySyntax {
        keyword: KEYWORD,
        stated: StatedDelivery::Families(
            entries
                .iter()
                .map(
                    |&(name, name_at, major, minor, minimum_at)| FamilyMinimumSyntax {
                        name: spelled(name, name_at),
                        minimum: DeploymentMinimumSyntax {
                            major,
                            minor,
                            span: At(minimum_at),
                        },
                    },
                )
                .collect(),
        ),
    }
}

/// The families one stated policy names, in the driver's canonical order.
fn resolved(statement: &DeliverySyntax<At>) -> Vec<&'static str> {
    let policy = stated_policy(Some(statement)).expect("the statement resolves");
    ArtifactFamilySelection::new(policy)
        .expect("a stated policy is a valid selection")
        .families()
        .iter()
        .map(|selected| selected.family.as_str())
        .collect()
}

/// The five consumer targets `docs/correctness-and-testing.md` names.
const NORMATIVE_TARGETS: [&str; 5] = [
    "aarch64-apple-darwin",
    "aarch64-apple-ios",
    "aarch64-apple-ios-sim",
    "aarch64-apple-ios-macabi",
    "x86_64-unknown-linux-gnu",
];

fn selected(family: ApplePlatform, major: u16, minor: u16) -> SelectedFamily {
    SelectedFamily {
        family,
        deployment_minimum: DeploymentMinimum::new(major, minor),
        msl_version: PROFILE_MSL_VERSION,
    }
}

fn policy(families: Vec<SelectedFamily>) -> ArtifactDeliveryPolicy {
    ArtifactDeliveryPolicy::SelectedFamilies {
        families,
        requirement: FamilyRequirement::RequiredWhenTargetMatches,
    }
}

fn selection(families: Vec<SelectedFamily>) -> ArtifactFamilySelection {
    ArtifactFamilySelection::new(policy(families)).expect("the selection is valid")
}

fn plan(
    families: Vec<SelectedFamily>,
    artifact: &[u8],
    deliveries: Vec<FamilyDelivery>,
) -> DeliveryPlan {
    DeliveryPlan::new(selection(families), artifact.to_vec(), deliveries)
        .expect("the plan covers its selection")
}

fn lines(source: &[&str]) -> String {
    let mut joined = source.join("\n");
    joined.push('\n');
    joined
}

/// One `#[cfg]`-gated item the emitter produced.
#[derive(Debug, Eq, PartialEq)]
enum GatedItem {
    /// A selector arm naming a built family's payload position.
    Payload { predicate: String, position: usize },
    /// The selector's catch-all arm, which is the semantic fallback.
    Fallback { predicate: String },
    /// A retained toolchain diagnostic.
    Diagnostic { predicate: String },
}

impl GatedItem {
    fn predicate(&self) -> &str {
        match self {
            Self::Payload { predicate, .. }
            | Self::Fallback { predicate }
            | Self::Diagnostic { predicate } => predicate,
        }
    }

    /// Whether this arm is a selector arm rather than a diagnostic.
    const fn is_selector(&self) -> bool {
        matches!(self, Self::Payload { .. } | Self::Fallback { .. })
    }
}

/// Reads every gated item out of one emitter output.
///
/// Total over the emitter's own shapes and panicking on anything else, and it
/// accounts for every line: an item the emitter learned to write but this parser
/// did not learn to read would otherwise be dropped, and a matrix test that
/// silently ignored an arm would report a clean partition over arms it never saw.
fn gated_items(source: &str) -> Vec<GatedItem> {
    let mut items = Vec::new();
    let mut pending: Option<String> = None;
    for line in source.lines() {
        if let Some(predicate) = line
            .strip_prefix("#[cfg(")
            .and_then(|rest| rest.strip_suffix(")]"))
        {
            assert!(
                pending.is_none(),
                "two `#[cfg]` attributes in a row; the emitter's shape changed",
            );
            pending = Some(predicate.to_owned());
            continue;
        }

        let Some(predicate) = pending.take() else {
            assert!(
                line.starts_with("const __TILER_ARTIFACT: &[u8] = b\""),
                "ungated emitted line `{line}` is not the artifact this parser knows",
            );
            continue;
        };

        if line.starts_with("::tiler::__private::__tiler_compile_error!(") {
            items.push(GatedItem::Diagnostic { predicate });
        } else if let Some(rest) = line
            .split_once("::core::option::Option::Some(")
            .map(|split| split.1)
        {
            let position = rest
                .strip_suffix("usize);")
                .expect("a payload arm names its position")
                .parse()
                .expect("a payload position is a number");
            items.push(GatedItem::Payload {
                predicate,
                position,
            });
        } else {
            assert!(
                line.ends_with("::core::option::Option::None;"),
                "gated emitted line `{line}` is not a shape this parser knows",
            );
            items.push(GatedItem::Fallback { predicate });
        }
    }
    assert!(
        pending.is_none(),
        "a trailing `#[cfg]` attribute gates nothing"
    );
    items
}

/// A region stating no `deliver` statement states `FallbackOnly`, and that is
/// deliverable.
///
/// This is the policy `tensor!` routes on for every region written without the
/// statement, so the absence is a *stated* no-AOT decision rather than an
/// unstated one — and it is what keeps such a region expanding to exactly the
/// tokens it expanded to before the statement existed.
#[test]
fn a_region_stating_no_delivery_states_a_deliverable_fallback_only_policy() {
    let policy = stated_policy::<At>(None).expect("absence resolves");
    assert_eq!(policy, ArtifactDeliveryPolicy::FallbackOnly);
    let selection = stated_delivery(policy).expect("FallbackOnly is deliverable");
    assert!(!selection.invokes_backend_compiler());
    assert!(selection.families().is_empty());
    assert_eq!(
        selection.policy(),
        &ArtifactDeliveryPolicy::FallbackOnly,
        "the frontend must state FallbackOnly rather than an empty family list",
    );
}

/// The production expansion plans no delivery, so its tokens are unchanged.
///
/// The delivery half is complete, and this is what says it is also inert: an
/// expansion that delivers `FallbackOnly` contributes no item, so `FallbackOnly`
/// still performs no backend compiler work and still expands to exactly the
/// block it expanded to before any of this existed. Both spellings of it are
/// checked, because `deliver fallback-only;` is a stated policy a consumer can
/// write and must be inert in exactly the same way as writing nothing.
#[test]
fn the_production_expansion_plans_no_delivery_items() {
    let inert: [Option<DeliverySyntax<At>>; 2] =
        [None, Some(profile_statement("fallback-only", 2))];
    assert_eq!(
        inert.len(),
        2,
        "the population this test covers is every spelling of no delivery, counted",
    );
    for stated in inert {
        let policy = stated_policy(stated.as_ref()).expect("no-delivery resolves");
        assert_eq!(policy, ArtifactDeliveryPolicy::FallbackOnly);
        let selection = stated_delivery(policy).expect("FallbackOnly is deliverable");
        let plan = fallback_plan(selection).expect("FallbackOnly plans nothing");
        assert_eq!(plan.items_source(), "");
        assert!(gated_items(&plan.items_source()).is_empty());
    }
}

/// Every profile a consumer states resolves to that profile's own families.
///
/// The statement is the consumer-visible half of
/// `every_named_profile_expands_to_a_canonical_selection` below: that one checks
/// what a profile *means*, and this one checks that stating its accepted name in
/// a region reaches it.
#[test]
fn every_profile_name_a_region_states_resolves_to_its_families() {
    let expected: [(&str, &[&str]); 4] = [
        ("fallback-only", &[]),
        ("macos", &["macos"]),
        ("ios", &["ios-device", "ios-simulator"]),
        ("macos-and-ios", &["ios-device", "ios-simulator", "macos"]),
    ];
    assert_eq!(
        expected.len(),
        NamedProfile::ALL.len(),
        "the population this test covers is every accepted profile name, counted",
    );
    for (spelling, families) in expected {
        assert_eq!(
            resolved(&profile_statement(spelling, 2)),
            families.to_vec(),
            "`deliver {spelling};` must resolve to these families",
        );
    }
}

/// A family list resolves to the families it names, at the floors it states.
///
/// The escape hatch's whole purpose: the same families a profile names, at a
/// deployment minimum the consumer chose. `ios` expands to two driver families
/// from one stated entry, because a developer building for iOS builds for the
/// simulator too.
#[test]
fn a_family_list_resolves_to_its_families_at_the_floors_it_states() {
    let statement = family_statement(&[("macos", 2, 27, 4, 3), ("ios", 4, 28, 0, 5)]);
    let policy = stated_policy(Some(&statement)).expect("the statement resolves");
    let ArtifactDeliveryPolicy::SelectedFamilies {
        families,
        requirement,
    } = ArtifactFamilySelection::new(policy)
        .expect("the list is a valid selection")
        .policy()
        .clone()
    else {
        panic!("a family list states selected families");
    };
    assert_eq!(requirement, FamilyRequirement::RequiredWhenTargetMatches);
    assert_eq!(
        families,
        vec![
            selected(ApplePlatform::IOsDevice, 28, 0),
            selected(ApplePlatform::IOsSimulator, 28, 0),
            selected(ApplePlatform::MacOs, 27, 4),
        ],
        "one stated `ios` entry is two driver families, and each carries the stated floor",
    );
}

/// A stated floor at the governed minimum is the profile's own selection.
///
/// The two productions are one vocabulary rather than two: writing the floors a
/// profile would have chosen must reach the identical selection, or `ios` would
/// mean one thing as a profile and another as a family.
#[test]
fn a_family_list_on_the_governed_floors_equals_the_profile_that_names_them() {
    let listed = stated_policy(Some(&family_statement(&[
        ("macos", 2, 26, 0, 3),
        ("ios", 4, 26, 0, 5),
    ])))
    .expect("the statement resolves");
    assert_eq!(
        ArtifactFamilySelection::new(listed).expect("valid"),
        ArtifactFamilySelection::new(NamedProfile::MacOsAndIOs.policy()).expect("valid"),
    );
}

/// A profile a region states but this frontend does not name is refused at the
/// name.
///
/// The near misses matter more than the hit: a profile decides which families a
/// consumer's build compiles for, so a name that is nearly one must refuse
/// rather than pick. `fallback_only` is included because it is the spelling a
/// consumer reaches for when the hyphen looks like an identifier problem.
#[test]
fn an_unknown_profile_is_refused_at_the_name() {
    for unknown in ["fallback_only", "macOS", "mac", "apple", "ios-device"] {
        let refusal = stated_policy(Some(&profile_statement(unknown, 7)))
            .expect_err("an unknown profile is refused");
        assert_eq!(
            refusal,
            StatementRefusal::UnknownProfile {
                name: unknown.to_owned(),
                span: At(7),
            },
        );
        let rendered = refusal.to_string();
        for offered in NamedProfile::ALL {
            assert!(
                rendered.contains(offered.as_str()),
                "the refusal must offer `{}`: {rendered}",
                offered.as_str(),
            );
        }
    }
    // The accepting neighbour differs only in the name.
    assert_eq!(resolved(&profile_statement("macos", 7)), vec!["macos"]);
}

/// A family list naming a family this frontend does not publish is refused at
/// the name.
///
/// `ios-device` and `mac-catalyst` are the driver's own identifiers, and their
/// refusal is the boundary: the consumer surface names `macos` and `ios`, and a
/// driver identifier leaking into a region would publish a vocabulary Tom did
/// not accept.
#[test]
fn an_unknown_family_is_refused_at_the_name() {
    for unknown in [
        "ios-device",
        "ios-simulator",
        "mac-catalyst",
        "fallback-only",
    ] {
        assert_eq!(
            stated_policy(Some(&family_statement(&[(unknown, 9, 26, 0, 10)])))
                .expect_err("an unknown family is refused"),
            StatementRefusal::UnknownFamily {
                name: unknown.to_owned(),
                span: At(9),
            },
        );
    }
    // The accepting neighbours differ only in the name.
    for family in DeliveredFamily::ALL {
        let minimum = family.governed_minimum();
        let statement =
            family_statement(&[(family.as_str(), 9, minimum.major(), minimum.minor(), 10)]);
        assert!(
            !resolved(&statement).is_empty(),
            "`{}` is a family a list may name",
            family.as_str(),
        );
    }
}

/// One family stated twice is refused at the repetition.
#[test]
fn a_repeated_family_is_refused_at_the_second_spelling() {
    assert_eq!(
        stated_policy(Some(&family_statement(&[
            ("macos", 2, 26, 0, 3),
            ("macos", 4, 27, 0, 5),
        ])))
        .expect_err("one family is stated twice"),
        StatementRefusal::RepeatedFamily {
            name: "macos",
            span: At(4),
        },
    );
    // The accepting neighbour differs only in the second family's name.
    assert_eq!(
        resolved(&family_statement(&[
            ("macos", 2, 26, 0, 3),
            ("ios", 4, 26, 0, 5),
        ])),
        vec!["ios-device", "ios-simulator", "macos"],
    );
}

/// A stated floor below the governed one is refused at the version that stated
/// it, carrying the driver's own reason.
///
/// The frontend forwards `MetalTarget::new`'s rejection rather than restating a
/// floor, and it runs that check per entry so the refusal lands on the version
/// token rather than on the statement. `ios 16.0` is checked as well as
/// `macos 13.0` because one stated `ios` entry is two driver families, and the
/// first of them in canonical order is what must be named.
#[test]
fn a_floor_below_the_governed_minimum_is_refused_at_the_version() {
    /// One entry stated below its floor, and the first driver family it covers.
    struct BelowFloor {
        /// The governed entry stated first, so the refusal cannot be the
        /// statement's.
        governed: (&'static str, u32, u16, u16, u32),
        /// The family stated second, and the floor it states.
        family: &'static str,
        major: u16,
        minor: u16,
        /// The driver family the refusal must name, in canonical order.
        platform: ApplePlatform,
        /// The minimum the governed table required.
        required: DeploymentMinimum,
    }

    let cases = [
        BelowFloor {
            governed: ("ios", 2, 26, 0, 3),
            family: "macos",
            major: 13,
            minor: 0,
            platform: ApplePlatform::MacOs,
            required: DeploymentMinimum::new(26, 0),
        },
        BelowFloor {
            governed: ("macos", 2, 26, 0, 3),
            family: "ios",
            major: 16,
            minor: 0,
            platform: ApplePlatform::IOsDevice,
            required: DeploymentMinimum::new(26, 0),
        },
    ];
    for case in cases {
        // Stated second, after a governed entry, so a refusal at the *statement*
        // would be indistinguishable from one at the entry that is wrong.
        let statement =
            family_statement(&[case.governed, (case.family, 4, case.major, case.minor, 5)]);
        assert_eq!(
            stated_policy(Some(&statement)).expect_err("the floor is below the governed minimum"),
            StatementRefusal::UngovernedTarget {
                source: MetalTargetError::DeploymentMinimumTooLow {
                    platform: case.platform,
                    language: PROFILE_MSL_VERSION,
                    requested: DeploymentMinimum::new(case.major, case.minor),
                    required: case.required,
                },
                span: At(5),
            },
        );
    }

    // The accepting neighbours differ only in the minor version: exactly the
    // governed floor is admitted, so the refusal is a floor and not a gap.
    assert_eq!(
        resolved(&family_statement(&[
            ("macos", 2, 26, 0, 3),
            ("ios", 4, 26, 0, 5),
        ])),
        vec!["ios-device", "ios-simulator", "macos"],
    );
}

/// A region stating a selected family resolves to a selection that invokes the
/// backend compiler, rather than being refused here.
///
/// This layer used to refuse it, because nothing compiled one. `crate::aot`
/// does now, so refusing here would refuse the thing the statement asks for;
/// what this layer still owes is that both accepted productions reach a
/// selection naming the same families, and that the selection *says* it needs
/// the backend compiler — which is the flag `crate::expand` branches on to
/// decide whether any toolchain work happens at all.
#[test]
fn a_stated_selected_family_resolves_to_a_backend_compilation() {
    let stated: [(&str, DeliverySyntax<At>, Vec<&str>); 2] = [
        (
            "deliver macos;",
            profile_statement("macos", 2),
            vec!["macos"],
        ),
        (
            "deliver macos 26.0, ios 26.0;",
            family_statement(&[("macos", 2, 26, 0, 3), ("ios", 4, 26, 0, 5)]),
            vec!["ios-device", "ios-simulator", "macos"],
        ),
    ];
    for (spelling, statement, families) in stated {
        let policy = stated_policy(Some(&statement)).expect("the statement resolves");
        let selection = stated_delivery(policy).expect("a stated family is a valid selection");
        assert!(
            selection.invokes_backend_compiler(),
            "`{spelling}` must state a selection that needs the offline driver",
        );
        assert_eq!(
            selection
                .families()
                .iter()
                .map(|selected| selected.family.as_str())
                .collect::<Vec<_>>(),
            families,
            "`{spelling}` must name its families in canonical order",
        );
    }
}

/// `FallbackOnly` states a selection that invokes no backend compiler.
///
/// The paired negative of the test above, and the executable form of the
/// property ADR 0053 defines `FallbackOnly` by: without it, "a selected family
/// needs the driver" would also be what a flag that was always true reported.
#[test]
fn fallback_only_states_a_selection_that_invokes_no_backend_compiler() {
    let selection = stated_delivery(ArtifactDeliveryPolicy::FallbackOnly)
        .expect("`fallback-only` is a valid selection");
    assert!(!selection.invokes_backend_compiler());
    assert!(selection.families().is_empty());
    assert!(
        fallback_plan(selection)
            .expect("an empty selection needs no artifact")
            .items_source()
            .is_empty(),
        "a region delivering nothing must be token-for-token what it always was",
    );
}

/// The frontend gets the driver's empty-selection rejection, not its own.
#[test]
fn an_empty_family_list_is_refused_as_an_invalid_selection() {
    assert_eq!(
        stated_delivery(policy(Vec::new())).expect_err("an empty selection is invalid"),
        DeliveryRefusal::InvalidSelection(FamilySelectionError::EmptySelection),
    );
}

/// A repeated family is refused, and the refusal names it.
#[test]
fn a_repeated_family_is_refused_as_an_invalid_selection() {
    assert_eq!(
        stated_delivery(policy(vec![
            selected(ApplePlatform::MacOs, 26, 0),
            selected(ApplePlatform::MacOs, 27, 0),
        ]))
        .expect_err("a duplicate family is invalid"),
        DeliveryRefusal::InvalidSelection(FamilySelectionError::DuplicateFamily {
            family: ApplePlatform::MacOs,
        }),
    );
}

/// A deployment minimum below its language floor is refused with the
/// target-level reason intact.
///
/// The frontend forwards the driver's version check rather than restating a
/// floor of its own, which is the point of there being one owner.
#[test]
fn a_deployment_minimum_below_its_language_floor_is_refused() {
    assert_eq!(
        stated_delivery(policy(vec![selected(ApplePlatform::MacOs, 13, 0)]))
            .expect_err("MSL 4.0 requires macOS 26.0"),
        DeliveryRefusal::InvalidSelection(FamilySelectionError::InvalidTarget {
            source: MetalTargetError::DeploymentMinimumTooLow {
                platform: ApplePlatform::MacOs,
                language: PROFILE_MSL_VERSION,
                requested: DeploymentMinimum::new(13, 0),
                required: DeploymentMinimum::new(26, 0),
            },
        }),
    );
}

/// The frontend reads one canonical value: declaration order is
/// presentation, and the identity bytes are the driver's.
///
/// Stating the same two families in either order has to yield one subject,
/// or two invocations meaning the same thing would be two artifacts.
#[test]
fn declaration_order_does_not_change_what_the_frontend_states() {
    let forward = selection(vec![
        selected(ApplePlatform::MacOs, 26, 0),
        selected(ApplePlatform::IOsDevice, 26, 0),
    ]);
    let reversed = selection(vec![
        selected(ApplePlatform::IOsDevice, 26, 0),
        selected(ApplePlatform::MacOs, 26, 0),
    ]);
    assert_eq!(forward, reversed);
    assert_eq!(forward.canonical_bytes(), reversed.canonical_bytes());
    assert_eq!(
        forward
            .compile_targets()
            .expect("both families resolve")
            .len(),
        2,
        "two families remain two compilations after canonicalization",
    );
}

/// Every named profile expands to a canonical selection, and to the families it
/// names.
///
/// This was Q-ART-008's close condition at the type level, and it stays checked
/// after the question closed: a profile is a spelling that resolves through
/// `ArtifactFamilySelection::new`, never a second encoder that could disagree
/// with it about ordering or validity.
#[test]
fn every_named_profile_expands_to_a_canonical_selection() {
    assert_eq!(
        NamedProfile::ALL.len(),
        4,
        "the population this test covers is every named profile, counted",
    );

    let expected: [(NamedProfile, &[&str]); 4] = [
        (NamedProfile::FallbackOnly, &[]),
        (NamedProfile::MacOs, &["macos"]),
        (NamedProfile::IOs, &["ios-device", "ios-simulator"]),
        (
            NamedProfile::MacOsAndIOs,
            &["ios-device", "ios-simulator", "macos"],
        ),
    ];
    for (profile, families) in expected {
        let selection = ArtifactFamilySelection::new(profile.policy())
            .expect("every profile is a valid selection");
        let named: Vec<&str> = selection
            .families()
            .iter()
            .map(|selected| selected.family.as_str())
            .collect();
        assert_eq!(
            named,
            families.to_vec(),
            "profile `{}` must expand to these families in canonical order",
            profile.as_str(),
        );
        assert_eq!(
            selection.invokes_backend_compiler(),
            !families.is_empty(),
            "profile `{}` must invoke the backend compiler exactly when it names a family",
            profile.as_str(),
        );
    }
}

/// Every profile name resolves to itself, and an unknown name resolves to
/// nothing.
///
/// The negative half is the one that matters: a profile decides which families a
/// consumer's build compiles for, so a near-miss must refuse rather than pick.
#[test]
fn every_profile_name_round_trips_and_a_near_miss_refuses() {
    for profile in NamedProfile::ALL {
        assert_eq!(NamedProfile::parse(profile.as_str()), Some(profile));
    }
    for unknown in [
        "",
        "MacOS",
        "macos ",
        "mac",
        "macos-and-ios-and-tvos",
        "apple",
    ] {
        assert_eq!(
            NamedProfile::parse(unknown),
            None,
            "`{unknown}` must not resolve to a profile",
        );
    }
}

/// Every profile family sits exactly on its governed language floor.
///
/// The deployment minimums are derived from the driver's table rather than
/// chosen here, and this is what says so: one minor version lower is refused by
/// the driver, so no profile excludes an OS version it could have included.
#[test]
fn every_profile_family_sits_on_its_governed_language_floor() {
    let mut checked = 0_usize;
    for profile in NamedProfile::ALL {
        for family in ArtifactFamilySelection::new(profile.policy())
            .expect("valid")
            .families()
        {
            checked += 1;
            let minimum = family.deployment_minimum;
            assert!(
                MetalTarget::new(family.family, minimum, family.msl_version).is_ok(),
                "the profile's own minimum must be a governed target",
            );
            let below = if minimum.minor() > 0 {
                DeploymentMinimum::new(minimum.major(), minimum.minor() - 1)
            } else {
                DeploymentMinimum::new(minimum.major() - 1, 0)
            };
            assert_eq!(
                MetalTarget::new(family.family, below, family.msl_version)
                    .expect_err("a minimum below the governed floor is not a target",),
                MetalTargetError::DeploymentMinimumTooLow {
                    platform: family.family,
                    language: family.msl_version,
                    requested: below,
                    required: minimum,
                },
                "{} is not on its floor",
                family.family.as_str(),
            );
        }
    }
    assert_eq!(
        checked, 6,
        "the population this test covers is every family of every profile, counted",
    );
}

/// No profile names Mac Catalyst, and the driver's own table is the reason.
///
/// `docs/correctness-and-testing.md` requires a Catalyst consumer to be covered,
/// and the ticket states the shape that coverage takes: Catalyst matches *no*
/// selected family and takes the fallback, never an iOS-device or macOS payload
/// relabelled as Catalyst-compatible. This pins why. Catalyst is representable —
/// `ApplePlatform::MacCatalyst` exists and has its own predicate — but the
/// governed table admits it only at MSL 4.0, so a profile at MSL 3.1 cannot name
/// it and one that named it would raise every other family in it to 4.0.
#[test]
fn no_profile_names_mac_catalyst_and_the_governed_table_is_why() {
    for profile in NamedProfile::ALL {
        for family in ArtifactFamilySelection::new(profile.policy())
            .expect("valid")
            .families()
        {
            assert_ne!(
                family.family,
                ApplePlatform::MacCatalyst,
                "profile `{}` names Mac Catalyst",
                profile.as_str(),
            );
        }
    }
    // Nor can the escape hatch reach it: the family list publishes the same two
    // names, so no spelling a consumer writes selects Catalyst.
    assert_eq!(DeliveredFamily::parse("mac-catalyst"), None);
    assert_eq!(DeliveredFamily::parse("catalyst"), None);
    assert_eq!(
        MetalTarget::new(
            ApplePlatform::MacCatalyst,
            DeploymentMinimum::new(26, 0),
            MslVersion::Metal3_1,
        )
        .expect_err("the governed table has no Catalyst row at MSL 3.1"),
        MetalTargetError::LanguageUnavailable {
            platform: ApplePlatform::MacCatalyst,
            language: MslVersion::Metal3_1,
        },
    );
    // At the standard the profiles actually select, Catalyst *is* a governed
    // target — so its absence from this frontend is a vocabulary decision
    // (`Q-ART-012`, deferred) rather than the floor it once was, and the two
    // assertions above and below are what keep that distinction legible.
    assert!(
        MetalTarget::new(
            ApplePlatform::MacCatalyst,
            DeploymentMinimum::new(26, 0),
            PROFILE_MSL_VERSION,
        )
        .is_ok(),
        "Catalyst is representable at the profile standard, so its absence is a decision",
    );
}

/// A plan must cover every selected family exactly once.
#[test]
fn a_plan_must_cover_every_selected_family() {
    let two = selection(vec![
        selected(ApplePlatform::MacOs, 26, 0),
        selected(ApplePlatform::IOsDevice, 26, 0),
    ]);
    assert_eq!(
        DeliveryPlan::new(
            two.clone(),
            b"bytes".to_vec(),
            vec![FamilyDelivery::Payload]
        )
        .expect_err("one outcome does not cover two families"),
        PlanRefusal::OutcomeCountMismatch {
            selected: 2,
            supplied: 1,
        },
    );
    assert_eq!(
        DeliveryPlan::new(
            two,
            b"bytes".to_vec(),
            vec![
                FamilyDelivery::Payload,
                FamilyDelivery::Payload,
                FamilyDelivery::Payload,
            ],
        )
        .expect_err("three outcomes do not cover two families"),
        PlanRefusal::OutcomeCountMismatch {
            selected: 2,
            supplied: 3,
        },
    );
}

/// A family that built without an artifact carrying it is refused.
#[test]
fn a_built_family_without_an_artifact_is_refused() {
    assert_eq!(
        DeliveryPlan::new(
            selection(vec![selected(ApplePlatform::MacOs, 26, 0)]),
            Vec::new(),
            vec![FamilyDelivery::Payload],
        )
        .expect_err("a built family needs bytes to select within"),
        PlanRefusal::ArtifactMissing { built: 1 },
    );
}

/// An artifact no consumer target could select is refused.
#[test]
fn an_artifact_with_no_built_family_is_refused() {
    assert_eq!(
        DeliveryPlan::new(
            selection(vec![selected(ApplePlatform::MacOs, 26, 0)]),
            b"orphan".to_vec(),
            vec![FamilyDelivery::Retained("no toolchain".to_owned())],
        )
        .expect_err("bytes with no reachable payload are refused"),
        PlanRefusal::ArtifactUnused { bytes: 6 },
    );
}

/// Every plan refusal reads as a defect in `tiler-macros` rather than in the
/// invocation, because that is what one is.
#[test]
fn a_malformed_plan_reads_as_a_frontend_defect() {
    let refusal = DeliveryRefusal::MalformedPlan(PlanRefusal::ArtifactMissing { built: 2 });
    let rendered = refusal.to_string();
    assert!(rendered.contains("defect in `tiler-macros`"), "{rendered}");
    assert!(
        rendered.contains("one payload per built family"),
        "{rendered}"
    );
}

/// One built family emits its gated selector and a total catch-all.
#[test]
fn one_built_family_emits_its_gated_selector_and_a_total_catch_all() {
    let plan = plan(
        vec![selected(ApplePlatform::MacOs, 26, 0)],
        b"tiler",
        vec![FamilyDelivery::Payload],
    );
    assert_eq!(
        plan.items_source(),
        lines(&[
            r#"const __TILER_ARTIFACT: &[u8] = b"tiler";"#,
            r#"#[cfg(all(target_os = "macos", target_abi = ""))]"#,
            "const __TILER_SELECTED_PAYLOAD: ::core::option::Option<usize> = \
             ::core::option::Option::Some(0usize);",
            r#"#[cfg(not(any(all(target_os = "macos", target_abi = ""))))]"#,
            "const __TILER_SELECTED_PAYLOAD: ::core::option::Option<usize> = \
             ::core::option::Option::None;",
        ]),
    );
}

/// A retained family emits its diagnostic under its own `#[cfg]` and nothing
/// else.
///
/// No artifact and no selector: nothing built, so there is no payload to select
/// and no bytes to carry. The `compile_error!` is the whole delivery, and it is
/// gated so an unrelated consumer target still compiles the fallback.
#[test]
fn a_retained_family_emits_only_its_gated_diagnostic() {
    let plan = plan(
        vec![selected(ApplePlatform::MacOs, 26, 0)],
        b"",
        vec![FamilyDelivery::Retained(
            "xcrun: error: unable to find utility \"metal\"".to_owned(),
        )],
    );
    assert_eq!(
        plan.items_source(),
        lines(&[
            r#"#[cfg(all(target_os = "macos", target_abi = ""))]"#,
            r#"::tiler::__private::__tiler_compile_error!("xcrun: error: unable to find utility \"metal\"");"#,
        ]),
        "the retained diagnostic is escaped so it cannot terminate its own literal",
    );
}

/// A mixed plan gates the built family and leaves the retained one to the
/// catch-all.
///
/// The retained family's predicate is deliberately absent from `any(…)`, so its
/// consumer target gets one actionable `compile_error!` and a well-formed
/// selector rather than that error plus an undefined name.
#[test]
fn a_mixed_plan_gates_the_built_family_and_leaves_the_retained_one_to_the_catch_all() {
    let plan = plan(
        vec![
            selected(ApplePlatform::IOsDevice, 26, 0),
            selected(ApplePlatform::IOsSimulator, 26, 0),
        ],
        b"tiler",
        vec![
            FamilyDelivery::Payload,
            FamilyDelivery::Retained("the iOS simulator SDK is not installed".to_owned()),
        ],
    );
    assert_eq!(
        plan.items_source(),
        lines(&[
            r#"#[cfg(all(target_os = "ios", target_abi = "sim"))]"#,
            r#"::tiler::__private::__tiler_compile_error!("the iOS simulator SDK is not installed");"#,
            r#"const __TILER_ARTIFACT: &[u8] = b"tiler";"#,
            r#"#[cfg(all(target_os = "ios", target_abi = ""))]"#,
            "const __TILER_SELECTED_PAYLOAD: ::core::option::Option<usize> = \
             ::core::option::Option::Some(0usize);",
            r#"#[cfg(not(any(all(target_os = "ios", target_abi = ""))))]"#,
            "const __TILER_SELECTED_PAYLOAD: ::core::option::Option<usize> = \
             ::core::option::Option::None;",
        ]),
    );
}

/// Payload positions follow canonical family order and skip retained families.
///
/// The position is what a consumer's `#[cfg]` selects within the one envelope,
/// so it has to count *built* families rather than selected ones — otherwise a
/// family that failed to build would leave a hole and every family after it
/// would select its neighbour's payload.
#[test]
fn payload_positions_count_built_families_in_canonical_order() {
    let plan = plan(
        vec![
            selected(ApplePlatform::IOsDevice, 26, 0),
            selected(ApplePlatform::IOsSimulator, 26, 0),
            selected(ApplePlatform::MacOs, 26, 0),
        ],
        b"tiler",
        vec![
            FamilyDelivery::Retained("no iOS SDK".to_owned()),
            FamilyDelivery::Payload,
            FamilyDelivery::Payload,
        ],
    );
    let positions: Vec<(String, usize)> = gated_items(&plan.items_source())
        .into_iter()
        .filter_map(|item| match item {
            GatedItem::Payload {
                predicate,
                position,
            } => Some((predicate, position)),
            _ => None,
        })
        .collect();
    assert_eq!(
        positions,
        vec![
            (
                consumer_cfg(ApplePlatform::IOsSimulator).predicate(),
                0_usize,
            ),
            (consumer_cfg(ApplePlatform::MacOs).predicate(), 1_usize),
        ],
        "the retained iOS-device family takes no position, and the two built families take 0 and 1",
    );
}

/// The emitted arms select exactly one payload per governed consumer target.
///
/// This is the matrix `docs/correctness-and-testing.md` states, evaluated
/// against `rustc`'s own `cfg` answer for each target rather than against a
/// second reading of the map: macOS and iOS device each select their own
/// payload, and the iOS simulator, Mac Catalyst, and an unrelated non-Apple
/// target each match no selected family and take the semantic fallback. Catalyst
/// is the case the corpus is most explicit about — it must never receive the
/// iOS-device payload, and `target_os = "ios"` alone would have given it one.
#[test]
fn the_emitted_arms_select_exactly_one_payload_per_consumer_target() {
    let plan = plan(
        vec![
            selected(ApplePlatform::MacOs, 26, 0),
            selected(ApplePlatform::IOsDevice, 26, 0),
        ],
        b"tiler",
        vec![FamilyDelivery::Payload, FamilyDelivery::Payload],
    );
    let items = gated_items(&plan.items_source());
    assert_eq!(
        items.iter().filter(|item| item.is_selector()).count(),
        3,
        "two built families and one catch-all",
    );
    assert!(
        !items
            .iter()
            .any(|item| matches!(item, GatedItem::Diagnostic { .. })),
        "no family was retained, so no diagnostic may be emitted",
    );

    // Canonical family order puts `ios-device` before `macos`.
    let expected: [(&str, Option<usize>); 5] = [
        ("aarch64-apple-darwin", Some(1)),
        ("aarch64-apple-ios", Some(0)),
        ("aarch64-apple-ios-sim", None),
        ("aarch64-apple-ios-macabi", None),
        ("x86_64-unknown-linux-gnu", None),
    ];
    assert_eq!(
        expected.map(|(triple, _)| triple),
        NORMATIVE_TARGETS,
        "the matrix must cover exactly the normative targets",
    );

    for (triple, selects) in expected {
        let target = target_cfg(triple);
        let active: Vec<&GatedItem> = items
            .iter()
            .filter(|item| item.is_selector() && evaluate(item.predicate(), &target))
            .collect();
        assert_eq!(
            active.len(),
            1,
            "{triple} must activate exactly one selector arm, not {active:?}",
        );
        match selects {
            Some(position) => assert_eq!(
                active[0],
                &GatedItem::Payload {
                    predicate: active[0].predicate().to_owned(),
                    position,
                },
                "{triple} must select payload {position}",
            ),
            None => assert!(
                matches!(active[0], GatedItem::Fallback { .. }),
                "{triple} matches no selected family and must take the fallback, not {:?}",
                active[0],
            ),
        }
    }
}

/// A retained family's diagnostic fires on its own target and nowhere else.
///
/// The other half of the matrix: `docs/integration/frontends.md` requires a
/// selected family's failure to be fatal on the matching target while leaving
/// "an unrelated fallback-only target" building.
#[test]
fn a_retained_diagnostic_fires_only_on_the_family_it_names() {
    let plan = plan(
        vec![
            selected(ApplePlatform::MacOs, 26, 0),
            selected(ApplePlatform::IOsDevice, 26, 0),
        ],
        b"tiler",
        // Positional against canonical family order, which puts `ios-device`
        // first: the iOS device built and macOS did not.
        vec![
            FamilyDelivery::Payload,
            FamilyDelivery::Retained("no macOS toolchain".to_owned()),
        ],
    );
    let items = gated_items(&plan.items_source());

    for triple in NORMATIVE_TARGETS {
        let target = target_cfg(triple);
        let fatal = items
            .iter()
            .filter(|item| {
                matches!(item, GatedItem::Diagnostic { .. }) && evaluate(item.predicate(), &target)
            })
            .count();
        assert_eq!(
            fatal,
            usize::from(triple == "aarch64-apple-darwin"),
            "{triple} must see the retained macOS diagnostic only if it is macOS",
        );
        assert_eq!(
            items
                .iter()
                .filter(|item| item.is_selector() && evaluate(item.predicate(), &target))
                .count(),
            1,
            "{triple} must still define the selector exactly once",
        );
    }
}

/// Every byte round-trips through the emitted literal, including the two that
/// would otherwise terminate or escape it.
///
/// Checked as text here and compiled as a literal by the facade's
/// `family_cfg_matching_family_embeds_its_payload` fixture, which is what makes
/// this a claim about Rust rather than about this function's own idea of Rust.
#[test]
fn every_byte_renders_as_a_literal_rust_accepts() {
    let all: Vec<u8> = (0..=u8::MAX).collect();
    let literal = byte_string_literal(&all);
    assert!(literal.starts_with("b\""), "{literal}");
    assert!(literal.ends_with('"'), "{literal}");
    assert!(literal.contains("\\x00"), "a NUL must be escaped");
    assert!(literal.contains("\\\""), "a quote must be escaped");
    assert!(literal.contains("\\\\"), "a backslash must be escaped");
    assert!(literal.contains("\\xff"), "a high byte must be escaped");
    assert!(
        literal.contains("ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
        "printable ASCII must pass through so a text payload stays readable: {literal}",
    );
    assert_eq!(byte_string_literal(b""), "b\"\"");
    assert_eq!(byte_string_literal(b"tiler"), "b\"tiler\"");
}

/// The emitted delivery items are what the facade's compile-pass fixture
/// compiles.
///
/// Without this the two ends would be related only by having been written on the
/// same day: the fixture would prove that *some* gated delivery compiles, and
/// the emitter would prove that *some* text is produced.
#[test]
fn the_nonmatching_fixture_compiles_what_this_emitter_produces() {
    let source = fixture("pass/family_cfg_nonmatching_targets_fall_back.rs");
    let plan = plan(
        vec![
            selected(ApplePlatform::IOsDevice, 26, 0),
            selected(ApplePlatform::IOsSimulator, 26, 0),
        ],
        b"tiler-artifact-envelope",
        vec![
            FamilyDelivery::Payload,
            FamilyDelivery::Retained("xcrun: error: unable to find utility \"metal\"".to_owned()),
        ],
    );
    let items = plan.items_source();
    assert!(
        source.contains(&items),
        "the fixture no longer contains the text this emitter produces.\n\nemitted:\n{items}",
    );
}

/// The matching-family fixture compiles what this emitter produces.
#[test]
fn the_matching_fixture_compiles_what_this_emitter_produces() {
    let source = fixture("pass/family_cfg_matching_family_embeds_its_payload.rs");
    let artifact: Vec<u8> = (0..=u8::MAX).collect();
    let plan = plan(
        vec![
            selected(ApplePlatform::MacOs, 26, 0),
            selected(ApplePlatform::IOsDevice, 26, 0),
        ],
        &artifact,
        vec![FamilyDelivery::Payload, FamilyDelivery::Payload],
    );
    let items = plan.items_source();
    assert!(
        source.contains(&items),
        "the fixture no longer contains the text this emitter produces.\n\nemitted:\n{items}",
    );
}

/// The compile-fail fixture compiles what this emitter produces, and fails.
///
/// The failing half is `trybuild`'s: this end only keeps the fixture's text
/// identical to the emitter's, so the retained diagnostic the consumer sees is
/// the one this module writes.
#[test]
fn the_retained_diagnostic_fixture_compiles_what_this_emitter_produces() {
    let source = fixture("fail/family_cfg_matching_family_retains_its_diagnostic.rs");
    let plan = plan(
        vec![selected(ApplePlatform::MacOs, 26, 0)],
        b"",
        vec![FamilyDelivery::Retained(
            "xcrun: error: unable to find utility \"metal\"".to_owned(),
        )],
    );
    let items = plan.items_source();
    assert!(
        source.contains(&items),
        "the fixture no longer contains the text this emitter produces.\n\nemitted:\n{items}",
    );
}

/// One real Apple `metal` refusal, captured whole.
///
/// **Measurement, not composition.** Produced on 2026-08-04 by
/// `crate::aot::tests::a_retained_msl_diagnostic_carries_the_emitted_source_position`
/// on macOS 27.0 / Apple M4 Max under Metal Toolchain 27A5228f, running the
/// host's real `metal` over that expansion's own emitted MSL. The two absolute
/// paths, the line, and the column are that run's; the framing, the stage, the
/// exit status, and the compiler's own bytes are what any `metal` refusal
/// retains. The fixture reading this text records the same provenance, and
/// neither may be edited into agreement with the other by hand — a capture that
/// was adjusted is no longer a capture.
///
/// It is kept separate from [`RETAINED_DIAGNOSTIC`] because the two prove
/// different things: that one is a single line, and a real compiler diagnostic
/// is several with a caret rule and quoted source.
const METAL_FRONT_END_DIAGNOSTIC: &str = concat!(
    "`tiler::tensor!` could not compile this region's artifact on this build host: ",
    "Metal AOT driver failed: offline metal failed ",
    "[/var/folders/7k/00gbj8p92d938w7bqf3k78040000gn/T/",
    "tiler-macros-aot-msl-position-7338-ThreadId(2)/metal] (exit code 1): ",
    "/var/folders/7k/00gbj8p92d938w7bqf3k78040000gn/T/",
    "tiler-metal-aot-7338-0-1785871605869051000/kernel.metal:63:39: ",
    "error: use of undeclared identifier 'tiler_no_such_identifier'\n",
    "kernel void tiler_injected_defect() { tiler_no_such_identifier(); }\n",
    "                                      ^\n",
    "1 error generated.",
);

/// A multi-line `metal` diagnostic reaches the consumer as one emitted item, and
/// the compile-fail fixture compiles exactly that.
///
/// The sibling above pins a one-line retained text, which cannot distinguish an
/// emitter that escapes newlines from one that writes them through: a raw
/// newline inside the `compile_error!` literal would close it and turn a
/// diagnostic into source the consumer's compiler tries to parse. That is why
/// the line count is asserted rather than only the fixture containment — the
/// containment check would still pass on a fixture that had been regenerated
/// from a broken emitter.
#[test]
fn the_metal_front_end_fixture_compiles_what_this_emitter_produces() {
    let plan = plan(
        vec![selected(ApplePlatform::MacOs, 26, 0)],
        b"",
        vec![FamilyDelivery::Retained(
            METAL_FRONT_END_DIAGNOSTIC.to_owned(),
        )],
    );
    let items = plan.items_source();
    assert_eq!(
        METAL_FRONT_END_DIAGNOSTIC.lines().count(),
        4,
        "the captured diagnostic must still be the multi-line one, or this test proves nothing",
    );
    assert_eq!(
        items.lines().count(),
        2,
        "a four-line diagnostic must emit as one gated item and one attribute: {items}",
    );

    let source = fixture("fail/family_cfg_matching_family_retains_a_metal_front_end_diagnostic.rs");
    assert!(
        source.contains(&items),
        "the fixture no longer contains the text this emitter produces.\n\nemitted:\n{items}",
    );
}

/// Reads one of the facade's `trybuild` fixtures.
fn fixture(relative: &str) -> String {
    let path = format!(
        "{}/../tiler/tests/facade/{relative}",
        env!("CARGO_MANIFEST_DIR"),
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("the facade fixture `{path}` is readable: {error}"))
}

/// The driver diagnostic every retained delivery below carries.
///
/// One string for both shapes and for the assertion that reads rustc's stderr,
/// so "the message the consumer sees" and "the message this test looks for"
/// cannot drift into two texts that are equal only by having been typed twice.
const RETAINED_DIAGNOSTIC: &str = "xcrun: error: unable to find utility \"metal\"";

/// What one consumer target's selector must resolve to for one emitted shape.
///
/// [`Self::NotEmitted`] is not "no payload": it is a plan in which *nothing*
/// built, so the emitter writes no artifact and no selector at all and there is
/// no name to assert on. Spelling it apart from [`Self::Fallback`] is what stops
/// a shape that silently stopped emitting its selector from reading as a target
/// that correctly took the fallback.
#[derive(Clone, Copy, Debug)]
enum SelectorOutcome {
    /// The plan emits no artifact and no selector.
    NotEmitted,
    /// The selector resolves to this payload position within the one envelope.
    Payload(usize),
    /// The selector takes its `not(any(…))` arm: the semantic fallback.
    Fallback,
}

/// One consumer target's row in one emitted shape's matrix.
struct CrossTargetRow {
    /// The Rust target triple the fixture is compiled for.
    triple: &'static str,
    /// What this target's selector must resolve to.
    selector: SelectorOutcome,
    /// The retained diagnostic this target's build must fail on, or nothing.
    fatal: Option<&'static str>,
}

/// One emitted delivery shape and what each normative target must do with it.
struct CrossTargetShape {
    /// What this shape is, so a failure names the plan rather than a row index.
    name: &'static str,
    /// The items the emitter produced, verbatim.
    items: String,
    /// How many bytes the one envelope carries, or zero when nothing built.
    artifact_bytes: usize,
    /// One row per target in [`NORMATIVE_TARGETS`], in that order.
    matrix: [CrossTargetRow; 5],
}

/// Compiles one fixture for one target at `cargo check` level.
///
/// `rustc --emit=metadata` is what `cargo check` runs. Cargo is skipped rather
/// than avoided: the fixture is dependency-free by construction, so a manifest
/// would contribute a target directory and a lockfile resolution and nothing
/// else to the verdict, and the toolchain is the same one either way because
/// `rust-toolchain.toml` resolves by directory ancestry — the same reason
/// [`crate::family_cfg::tests::target_cfg`] spells `rustc` directly.
///
/// The edition is stated because rustc's command-line default is 2015, where the
/// `::core::` paths the emitter writes do not resolve at all; 2024 is the
/// workspace edition, which is the edition a consumer compiles the expansion in.
fn check_for_target(directory: &Path, triple: &str, source: &str) -> Result<(), String> {
    let fixture = directory.join("fixture.rs");
    // Cross-target rust-std is installed without a cross-built facade rlib.
    // This exact local stand-in exercises the facade's builtin re-export shape;
    // the trybuild fixtures exercise the real `tiler::__private` owner.
    let source = format!(
        "extern crate self as tiler;\n\
         pub mod __private {{\n\
             pub use core::compile_error as __tiler_compile_error;\n\
         }}\n\
         {source}"
    );
    std::fs::write(&fixture, &source)
        .unwrap_or_else(|error| panic!("the fixture `{}` is writable: {error}", fixture.display()));

    let output = Command::new("rustc")
        .args([
            "--edition",
            "2024",
            "--crate-name",
            "tiler_family_cfg_fixture",
            "--crate-type",
            "lib",
            "--emit",
            "metadata",
            "--target",
            triple,
            "-o",
        ])
        .arg(directory.join("fixture.rmeta"))
        .arg(&fixture)
        .output()
        .unwrap_or_else(|error| panic!("`rustc --target {triple}` did not run: {error}"));

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).into_owned())
    }
}

/// Fails when a target cannot be compiled for at all.
///
/// Two of the shapes below *expect* a non-zero exit, and a missing `rust-std`
/// produces one too — so without this probe "the target is not installed" and
/// "the gated `compile_error!` fired" would be the same observation, and the one
/// that proves nothing would be counted as the one that proves the matrix. The
/// probe compiles a fixture that must succeed on every target there is, so its
/// failure is about the host and never about the emitter.
fn require_installed_target(directory: &Path, triple: &str) {
    if let Err(stderr) = check_for_target(
        directory,
        triple,
        "pub const PROBE: ::core::option::Option<usize> = ::core::option::Option::None;\n",
    ) {
        panic!(
            "`{triple}` cannot be compiled for, so no outcome for it would mean anything. Install \
             its standard library with `rustup target add {triple}`:\n{stderr}"
        );
    }
}

/// Appends the expectation one target must satisfy to the emitter's own items.
///
/// `const _: () = assert!(…)` is evaluated by `--emit=metadata`, so the verdict
/// comes from the compiler that will compile a consumer rather than from this
/// crate's reading of its own predicates. Both emitted names are referenced,
/// which is also what keeps the fixture free of unused-item lints without
/// passing rustc a flag that would suppress them.
fn with_expectation(shape: &CrossTargetShape, selector: SelectorOutcome) -> String {
    let mut source = shape.items.clone();
    let expected = match selector {
        SelectorOutcome::NotEmitted => return source,
        SelectorOutcome::Payload(position) => {
            format!("::core::option::Option::Some({position}usize)")
        }
        SelectorOutcome::Fallback => "::core::option::Option::None".to_owned(),
    };
    write!(
        source,
        "const _: () = assert!(matches!({SELECTED_PAYLOAD_BINDING}, {expected}));\n\
         const _: () = assert!({ARTIFACT_BINDING}.len() == {}usize);\n",
        shape.artifact_bytes,
    )
    .expect("writing to a `String` cannot fail");
    source
}

/// The three emitted shapes, in the order the module doc's table lists them.
///
/// They are the plans the three facade fixtures stand in for, built here through
/// the same emitter, so this evidence and the `trybuild` evidence are about one
/// text rather than two: `the_matching_fixture_compiles_what_this_emitter_produces`
/// and its two siblings are what tie each fixture to the plan repeated below.
fn cross_target_shapes() -> [CrossTargetShape; 3] {
    let envelope: Vec<u8> = (0..=u8::MAX).collect();
    [
        // Canonical family order puts `ios-device` before `macos`, so macOS is
        // payload 1 — a selector that ignored the family and took the first
        // payload would be indistinguishable from a correct one otherwise.
        CrossTargetShape {
            name: "macOS and iOS device both built",
            items: plan(
                vec![
                    selected(ApplePlatform::MacOs, 26, 0),
                    selected(ApplePlatform::IOsDevice, 26, 0),
                ],
                &envelope,
                vec![FamilyDelivery::Payload, FamilyDelivery::Payload],
            )
            .items_source(),
            artifact_bytes: envelope.len(),
            matrix: [
                CrossTargetRow {
                    triple: "aarch64-apple-darwin",
                    selector: SelectorOutcome::Payload(1),
                    fatal: None,
                },
                CrossTargetRow {
                    triple: "aarch64-apple-ios",
                    selector: SelectorOutcome::Payload(0),
                    fatal: None,
                },
                CrossTargetRow {
                    triple: "aarch64-apple-ios-sim",
                    selector: SelectorOutcome::Fallback,
                    fatal: None,
                },
                CrossTargetRow {
                    triple: "aarch64-apple-ios-macabi",
                    selector: SelectorOutcome::Fallback,
                    fatal: None,
                },
                CrossTargetRow {
                    triple: "x86_64-unknown-linux-gnu",
                    selector: SelectorOutcome::Fallback,
                    fatal: None,
                },
            ],
        },
        CrossTargetShape {
            name: "iOS device built, iOS simulator retained",
            items: plan(
                vec![
                    selected(ApplePlatform::IOsDevice, 26, 0),
                    selected(ApplePlatform::IOsSimulator, 26, 0),
                ],
                b"tiler-artifact-envelope",
                vec![
                    FamilyDelivery::Payload,
                    FamilyDelivery::Retained(RETAINED_DIAGNOSTIC.to_owned()),
                ],
            )
            .items_source(),
            artifact_bytes: b"tiler-artifact-envelope".len(),
            matrix: [
                CrossTargetRow {
                    triple: "aarch64-apple-darwin",
                    selector: SelectorOutcome::Fallback,
                    fatal: None,
                },
                CrossTargetRow {
                    triple: "aarch64-apple-ios",
                    selector: SelectorOutcome::Payload(0),
                    fatal: None,
                },
                // The retained family's own target, and the only one the
                // diagnostic may reach. It still takes the fallback arm rather
                // than an undefined name, which is why the build fails with one
                // actionable error instead of two.
                CrossTargetRow {
                    triple: "aarch64-apple-ios-sim",
                    selector: SelectorOutcome::Fallback,
                    fatal: Some(RETAINED_DIAGNOSTIC),
                },
                CrossTargetRow {
                    triple: "aarch64-apple-ios-macabi",
                    selector: SelectorOutcome::Fallback,
                    fatal: None,
                },
                CrossTargetRow {
                    triple: "x86_64-unknown-linux-gnu",
                    selector: SelectorOutcome::Fallback,
                    fatal: None,
                },
            ],
        },
        CrossTargetShape {
            name: "macOS retained, nothing built",
            items: plan(
                vec![selected(ApplePlatform::MacOs, 26, 0)],
                b"",
                vec![FamilyDelivery::Retained(RETAINED_DIAGNOSTIC.to_owned())],
            )
            .items_source(),
            artifact_bytes: 0,
            matrix: [
                CrossTargetRow {
                    triple: "aarch64-apple-darwin",
                    selector: SelectorOutcome::NotEmitted,
                    fatal: Some(RETAINED_DIAGNOSTIC),
                },
                CrossTargetRow {
                    triple: "aarch64-apple-ios",
                    selector: SelectorOutcome::NotEmitted,
                    fatal: None,
                },
                CrossTargetRow {
                    triple: "aarch64-apple-ios-sim",
                    selector: SelectorOutcome::NotEmitted,
                    fatal: None,
                },
                CrossTargetRow {
                    triple: "aarch64-apple-ios-macabi",
                    selector: SelectorOutcome::NotEmitted,
                    fatal: None,
                },
                CrossTargetRow {
                    triple: "x86_64-unknown-linux-gnu",
                    selector: SelectorOutcome::NotEmitted,
                    fatal: None,
                },
            ],
        },
    ]
}

/// Every emitted shape compiles for every normative target exactly as the matrix
/// says, decided by rustc for that target.
///
/// This is the claim `generate-cfg-gated-artifact-family-delivery` recorded as
/// out of reach while `aarch64-apple-darwin` was the only installed target: that
/// "a nonmatching target compiles the semantic fallback" rests on a build that
/// ran rather than on evaluating the predicates against `rustc --print cfg`. The
/// module doc records the resulting five-target matrix, its `cargo check`-level
/// boundary, and the toolchain that produced it.
///
/// It is `#[ignore]`d for a host reason and not a cost one — twenty check
/// compilations measure in seconds. Four of the five targets are `rustup`
/// `rust-std` components that `rust-toolchain.toml` does not declare and
/// `deps.sh` neither installs nor verifies, so making this gate-resident would
/// fail `make check` on a host bootstrapped exactly as this repository
/// documents. Declaring them is a host-toolchain policy change and is tracked by
/// `declare-the-cross-compilation-targets-in-the-toolchain-manifest`; a test
/// that instead skipped the targets it could not find would report a clean pass
/// over a population it never counted, which is the failure mode `AGENTS.md`
/// names outright.
///
/// Run it by hand from the repository root:
///
/// ```text
/// rustup target add aarch64-apple-ios aarch64-apple-ios-sim \
///     aarch64-apple-ios-macabi x86_64-unknown-linux-gnu
/// cargo nextest run -p tiler-macros --run-ignored all \
///     -E 'test(every_emitted_shape_compiles_as_the_five_target_matrix_says)'
/// ```
#[test]
#[ignore = "cross-compiles for four targets `deps.sh` neither installs nor verifies; run by hand"]
fn every_emitted_shape_compiles_as_the_five_target_matrix_says() {
    let directory = std::env::temp_dir().join(format!(
        "tiler-family-cfg-cross-target-{}",
        std::process::id(),
    ));
    // Removed first so a rerun cannot read a stale fixture, and reconstructed
    // entirely from this harness — nothing here is a checked-in artifact.
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap_or_else(|error| {
        panic!(
            "the fixture directory `{}` is creatable: {error}",
            directory.display()
        )
    });

    for triple in NORMATIVE_TARGETS {
        require_installed_target(&directory, triple);
    }

    let shapes = cross_target_shapes();
    let mut compiled = 0_usize;
    for shape in &shapes {
        assert_eq!(
            shape.matrix.each_ref().map(|row| row.triple),
            NORMATIVE_TARGETS,
            "`{}` must state a row for exactly the normative targets, in order",
            shape.name,
        );
        // A shape whose matrix expects no fatal target must carry no diagnostic
        // at all, and one that expects a fatal target must carry the diagnostic
        // its non-matching targets then compile *without*. Otherwise the four
        // targets that see an empty translation unit would pass by having been
        // handed nothing to compile.
        assert_eq!(
            shape
                .items
                .contains("::tiler::__private::__tiler_compile_error!"),
            shape.matrix.iter().any(|row| row.fatal.is_some()),
            "`{}` emits a gated `compile_error!` exactly when its matrix expects one to fire:\n{}",
            shape.name,
            shape.items,
        );

        for row in &shape.matrix {
            let source = with_expectation(shape, row.selector);
            let outcome = check_for_target(&directory, row.triple, &source);
            compiled += 1;

            match (row.fatal, outcome) {
                (None, Ok(())) => {}
                (None, Err(stderr)) => panic!(
                    "`{}` must compile for {}:\n{stderr}\nfixture:\n{source}",
                    shape.name, row.triple,
                ),
                (Some(diagnostic), Ok(())) => panic!(
                    "`{}` must fail for {} on the retained diagnostic `{diagnostic}`, and it \
                     compiled:\nfixture:\n{source}",
                    shape.name, row.triple,
                ),
                (Some(diagnostic), Err(stderr)) => {
                    assert!(
                        stderr.contains(diagnostic),
                        "`{}` must fail for {} on the driver's own retained text, not on \
                         something else:\n{stderr}",
                        shape.name,
                        row.triple,
                    );
                    // A failure that also carries a const-eval panic means the
                    // selector was wrong as well, and the retained text alone
                    // would have hidden it behind an expected non-zero exit.
                    assert!(
                        !stderr.contains("evaluation panicked"),
                        "`{}` for {} fails on the retained diagnostic *and* on its selector \
                         expectation:\n{stderr}",
                        shape.name,
                        row.triple,
                    );
                }
            }
        }
    }

    assert_eq!(
        compiled,
        shapes.len() * NORMATIVE_TARGETS.len(),
        "the population this test covers is every emitted shape on every normative target, counted",
    );
    assert_eq!(
        compiled, 15,
        "three emitted shapes and five consumer targets"
    );

    std::fs::remove_dir_all(&directory).unwrap_or_else(|error| {
        panic!(
            "the fixture directory `{}` is removable: {error}",
            directory.display()
        )
    });
}

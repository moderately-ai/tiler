//! The stated policy, the named profiles, and the tokens a plan delivers.
//!
//! Three kinds of evidence live here. Exact-text assertions pin what the emitter
//! writes, because the generated code is the product. Evaluation against
//! `rustc --print cfg` decides what that text *means* on a real consumer target,
//! which is the only way to check the five-target matrix
//! `docs/correctness-and-testing.md` states without installing five standard
//! libraries. And the fixture comparisons bind both to files the facade actually
//! compiles, so the compile evidence is evidence about this emitter rather than
//! about text someone wrote on the same day.

use tiler_metal_aot::family::{
    ArtifactDeliveryPolicy, ArtifactFamilySelection, FamilyRequirement, FamilySelectionError,
    SelectedFamily,
};
use tiler_metal_aot::input::{
    ApplePlatform, DeploymentMinimum, MetalTarget, MetalTargetError, MslVersion,
};

use super::{
    DeliveryPlan, DeliveryRefusal, FamilyDelivery, NamedProfile, PlanRefusal, byte_string_literal,
    stated_delivery, stated_plan, stated_policy,
};
use crate::family_cfg::consumer_cfg;
use crate::family_cfg::tests::{evaluate, target_cfg};

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
        msl_version: MslVersion::Metal3_1,
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

        if line.starts_with("const _: () = { ::core::compile_error!(") {
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

/// The current expansion states `FallbackOnly`, and that is deliverable.
///
/// This is the policy `tensor!` actually routes on today, so the anchor it
/// emits is a *stated* no-AOT decision rather than an unstated one.
#[test]
fn the_current_expansion_states_a_deliverable_fallback_only_policy() {
    let selection = stated_delivery(stated_policy()).expect("FallbackOnly is deliverable");
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
/// expansion that states `FallbackOnly` contributes no item, so `FallbackOnly`
/// still performs no backend compiler work and still expands to exactly the
/// block it expanded to before any of this existed.
#[test]
fn the_production_expansion_plans_no_delivery_items() {
    let selection = stated_delivery(stated_policy()).expect("FallbackOnly is deliverable");
    let plan = stated_plan(selection).expect("FallbackOnly plans nothing");
    assert_eq!(plan.items_source(), "");
    assert!(gated_items(&plan.items_source()).is_empty());
}

/// A valid selection naming families is refused rather than silently
/// downgraded to fallback.
///
/// The paired negative of the test above: without it, "`FallbackOnly` is
/// deliverable" would also be what a function that accepted everything
/// reported.
#[test]
fn a_selected_family_is_refused_rather_than_downgraded_to_fallback() {
    let refusal = stated_delivery(policy(vec![
        selected(ApplePlatform::IOsSimulator, 17, 0),
        selected(ApplePlatform::MacOs, 14, 0),
    ]))
    .expect_err("this expansion cannot build a selected family yet");
    assert_eq!(
        refusal,
        DeliveryRefusal::BackendCompilationUnavailable {
            families: vec!["ios-simulator", "macos"],
        },
        "the refusal names the families in canonical order, not declaration order",
    );
    let rendered = refusal.to_string();
    assert!(rendered.contains("ios-simulator"), "{rendered}");
    assert!(rendered.contains("macos"), "{rendered}");
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
            selected(ApplePlatform::MacOs, 14, 0),
            selected(ApplePlatform::MacOs, 15, 0),
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
            .expect_err("MSL 3.1 requires macOS 14.0"),
        DeliveryRefusal::InvalidSelection(FamilySelectionError::InvalidTarget {
            source: MetalTargetError::DeploymentMinimumTooLow {
                platform: ApplePlatform::MacOs,
                language: MslVersion::Metal3_1,
                requested: DeploymentMinimum::new(13, 0),
                required: DeploymentMinimum::new(14, 0),
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
        selected(ApplePlatform::MacOs, 14, 0),
        selected(ApplePlatform::IOsDevice, 17, 0),
    ]);
    let reversed = selection(vec![
        selected(ApplePlatform::IOsDevice, 17, 0),
        selected(ApplePlatform::MacOs, 14, 0),
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
/// This is Q-ART-008's close condition at the type level: a profile is a
/// spelling that resolves through `ArtifactFamilySelection::new`, never a second
/// encoder that could disagree with it about ordering or validity.
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
        let selection = profile
            .selection()
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
        for family in profile.selection().expect("valid").families() {
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
        for family in profile.selection().expect("valid").families() {
            assert_ne!(
                family.family,
                ApplePlatform::MacCatalyst,
                "profile `{}` names Mac Catalyst",
                profile.as_str(),
            );
        }
    }
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
    assert!(
        MetalTarget::new(
            ApplePlatform::MacCatalyst,
            DeploymentMinimum::new(26, 0),
            MslVersion::Metal4_0,
        )
        .is_ok(),
        "Catalyst is representable at MSL 4.0, so its absence is a floor and not a gap",
    );
}

/// A plan must cover every selected family exactly once.
#[test]
fn a_plan_must_cover_every_selected_family() {
    let two = selection(vec![
        selected(ApplePlatform::MacOs, 14, 0),
        selected(ApplePlatform::IOsDevice, 17, 0),
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
            selection(vec![selected(ApplePlatform::MacOs, 14, 0)]),
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
            selection(vec![selected(ApplePlatform::MacOs, 14, 0)]),
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
        vec![selected(ApplePlatform::MacOs, 14, 0)],
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
        vec![selected(ApplePlatform::MacOs, 14, 0)],
        b"",
        vec![FamilyDelivery::Retained(
            "xcrun: error: unable to find utility \"metal\"".to_owned(),
        )],
    );
    assert_eq!(
        plan.items_source(),
        lines(&[
            r#"#[cfg(all(target_os = "macos", target_abi = ""))]"#,
            r#"const _: () = { ::core::compile_error!("xcrun: error: unable to find utility \"metal\""); };"#,
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
            selected(ApplePlatform::IOsDevice, 17, 0),
            selected(ApplePlatform::IOsSimulator, 17, 0),
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
            r#"const _: () = { ::core::compile_error!("the iOS simulator SDK is not installed"); };"#,
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
            selected(ApplePlatform::IOsDevice, 17, 0),
            selected(ApplePlatform::IOsSimulator, 17, 0),
            selected(ApplePlatform::MacOs, 14, 0),
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
            selected(ApplePlatform::MacOs, 14, 0),
            selected(ApplePlatform::IOsDevice, 17, 0),
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
            selected(ApplePlatform::MacOs, 14, 0),
            selected(ApplePlatform::IOsDevice, 17, 0),
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
            selected(ApplePlatform::IOsDevice, 17, 0),
            selected(ApplePlatform::IOsSimulator, 17, 0),
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
            selected(ApplePlatform::MacOs, 14, 0),
            selected(ApplePlatform::IOsDevice, 17, 0),
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
        vec![selected(ApplePlatform::MacOs, 14, 0)],
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

/// Reads one of the facade's `trybuild` fixtures.
fn fixture(relative: &str) -> String {
    let path = format!(
        "{}/../tiler/tests/facade/{relative}",
        env!("CARGO_MANIFEST_DIR"),
    );
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("the facade fixture `{path}` is readable: {error}"))
}

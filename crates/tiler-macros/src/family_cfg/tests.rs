//! The versioned map, checked against its pinned rows and against `rustc`.
//!
//! Two kinds of evidence live here and they answer different questions. The
//! pinned table answers "did this change?", which is what makes a widening
//! without a version bump visible. The `rustc --print cfg` comparison answers
//! "is it right?", by evaluating the predicate this module actually emits
//! against the compiler's own answer for a real target — including the four
//! Apple targets and the non-Apple target `docs/correctness-and-testing.md`
//! names, none of whose standard libraries are installed on this host and none
//! of which therefore have to be.

use std::collections::BTreeSet;
use std::process::Command;

use tiler_metal_aot::input::ApplePlatform;

use super::{MAP_VERSION, consumer_cfg};

/// The exact table [`MAP_VERSION`] names, one row per governed family.
///
/// Spelled out rather than derived, because a table derived from the source it
/// checks reports only that the source equals itself. Changing any row, or
/// adding one, must change this literal *and* [`MAP_VERSION`]: generated code
/// already embedded in a built consumer was gated by the old row.
const PINNED_MAP: [(&str, &str); ApplePlatform::COUNT] = [
    ("macos", r#"all(target_os = "macos", target_abi = "")"#),
    ("ios-device", r#"all(target_os = "ios", target_abi = "")"#),
    (
        "ios-simulator",
        r#"all(target_os = "ios", target_abi = "sim")"#,
    ),
    (
        "mac-catalyst",
        r#"all(target_os = "ios", target_abi = "macabi")"#,
    ),
    ("tvos-device", r#"all(target_os = "tvos", target_abi = "")"#),
    (
        "tvos-simulator",
        r#"all(target_os = "tvos", target_abi = "sim")"#,
    ),
    (
        "visionos-device",
        r#"all(target_os = "visionos", target_abi = "")"#,
    ),
    (
        "visionos-simulator",
        r#"all(target_os = "visionos", target_abi = "sim")"#,
    ),
    (
        "watchos-device",
        r#"all(target_os = "watchos", target_abi = "")"#,
    ),
    (
        "watchos-simulator",
        r#"all(target_os = "watchos", target_abi = "sim")"#,
    ),
];

/// Every Rust target this map is checked against, and the family it selects.
///
/// The first five rows are the population `docs/correctness-and-testing.md`
/// states normatively — macOS, iOS device, iOS simulator, Catalyst, and an
/// unrelated non-Apple target. The rest exist so the check covers every family
/// the map claims to cover rather than only the interesting ones, and
/// `x86_64-apple-ios` is here because it is the one Apple triple whose family is
/// not guessable from its name: it is the *simulator*.
const CHECKED_TARGETS: [(&str, Option<ApplePlatform>); 15] = [
    ("aarch64-apple-darwin", Some(ApplePlatform::MacOs)),
    ("aarch64-apple-ios", Some(ApplePlatform::IOsDevice)),
    ("aarch64-apple-ios-sim", Some(ApplePlatform::IOsSimulator)),
    ("aarch64-apple-ios-macabi", Some(ApplePlatform::MacCatalyst)),
    ("x86_64-unknown-linux-gnu", None),
    ("x86_64-apple-darwin", Some(ApplePlatform::MacOs)),
    ("x86_64-apple-ios", Some(ApplePlatform::IOsSimulator)),
    ("x86_64-apple-ios-macabi", Some(ApplePlatform::MacCatalyst)),
    ("aarch64-apple-tvos", Some(ApplePlatform::TvOsDevice)),
    ("aarch64-apple-tvos-sim", Some(ApplePlatform::TvOsSimulator)),
    (
        "aarch64-apple-visionos",
        Some(ApplePlatform::VisionOsDevice),
    ),
    (
        "aarch64-apple-visionos-sim",
        Some(ApplePlatform::VisionOsSimulator),
    ),
    ("aarch64-apple-watchos", Some(ApplePlatform::WatchOsDevice)),
    (
        "aarch64-apple-watchos-sim",
        Some(ApplePlatform::WatchOsSimulator),
    ),
    ("x86_64-pc-windows-msvc", None),
];

/// The `key = "value"` pairs `rustc` reports for one target.
///
/// Fails rather than returns empty when `rustc` cannot answer. An empty set
/// would make every predicate evaluate to false, which is exactly what a
/// correct map for a non-Apple target looks like — so a silent failure here
/// would read as a pass for the cases that matter least and hide the ones that
/// matter most.
pub(crate) fn target_cfg(triple: &str) -> BTreeSet<(String, String)> {
    let output = Command::new("rustc")
        .args(["--print", "cfg", "--target", triple])
        .output()
        .unwrap_or_else(|error| {
            panic!("`rustc --print cfg --target {triple}` did not run: {error}")
        });
    assert!(
        output.status.success(),
        "`rustc --print cfg --target {triple}` failed: {}",
        String::from_utf8_lossy(&output.stderr),
    );

    let printed = String::from_utf8(output.stdout).expect("`rustc --print cfg` prints UTF-8");
    let pairs: BTreeSet<(String, String)> = printed
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once('=')?;
            Some((
                key.trim().to_owned(),
                value.trim().trim_matches('"').to_owned(),
            ))
        })
        .collect();

    // `target_os` is present for every target rustc knows, so its absence means
    // the parse walked nothing rather than that the target has no OS.
    assert!(
        pairs.iter().any(|(key, _)| key == "target_os"),
        "no `target_os` in the parsed cfg for {triple}; the parse is wrong, not the target",
    );
    pairs
}

/// Splits one comma-separated predicate list, respecting nesting.
///
/// A naive `split(", ")` would cut `any(all(a, b), all(c, d))` into four pieces,
/// three of which are not predicates — so the catch-all arm the emitter produces
/// would be silently misread rather than refused.
fn split_operands(list: &str) -> Vec<&str> {
    let mut operands = Vec::new();
    let mut depth = 0_usize;
    let mut start = 0_usize;
    for (position, character) in list.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => depth = depth.checked_sub(1).expect("the predicate is balanced"),
            ',' if depth == 0 => {
                operands.push(list[start..position].trim());
                start = position + 1;
            }
            _ => {}
        }
    }
    assert_eq!(depth, 0, "`{list}` has unbalanced parentheses");
    operands.push(list[start..].trim());
    operands
}

/// Evaluates one rendered predicate against a target's `cfg` set.
///
/// Deliberately total over exactly the grammar Tiler emits — `all(k = "v", …)`,
/// `any(p, …)`, `not(p)`, and a quoted key/value term — and panics on anything
/// else. A permissive parser would let a widened predicate evaluate under a model
/// that no longer describes it, which is the failure this file exists to catch.
///
/// `all` is what a family predicate is; `any` and `not` are what the delivery
/// selector's catch-all arm is built from, so one evaluator covers the map and
/// the generated code that embeds it.
pub(crate) fn evaluate(predicate: &str, target: &BTreeSet<(String, String)>) -> bool {
    if let Some(inner) = strip_call(predicate, "all") {
        let operands = split_operands(inner);
        assert_eq!(
            operands.len(),
            2,
            "`{predicate}` has {} terms; a family predicate names the two governed keys",
            operands.len(),
        );
        // The arity check above runs on the collected operands, so short
        // circuiting here cannot skip it.
        return operands.into_iter().all(|term| term_holds(term, target));
    }
    if let Some(inner) = strip_call(predicate, "any") {
        let operands = split_operands(inner);
        assert!(!operands.is_empty(), "`{predicate}` names no alternative");
        return operands
            .into_iter()
            .any(|operand| evaluate(operand, target));
    }
    if let Some(inner) = strip_call(predicate, "not") {
        return !evaluate(inner, target);
    }
    panic!("`{predicate}` is not a shape this evaluator models")
}

/// Strips `name( … )`, or nothing.
fn strip_call<'a>(predicate: &'a str, name: &str) -> Option<&'a str> {
    predicate
        .strip_prefix(name)?
        .strip_prefix('(')?
        .strip_suffix(')')
}

/// Decides one `key = "value"` term against a target's `cfg` set.
fn term_holds(term: &str, target: &BTreeSet<(String, String)>) -> bool {
    let (key, value) = term
        .split_once(" = ")
        .unwrap_or_else(|| panic!("`{term}` is not the `key = \"value\"` shape"));
    let value = value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or_else(|| panic!("`{term}` does not carry a quoted value"));
    target.contains(&(key.to_owned(), value.to_owned()))
}

/// The pinned table is exactly what the map produces, row for row.
#[test]
fn the_versioned_map_is_pinned_row_by_row() {
    assert_eq!(
        MAP_VERSION, "tiler.frontend.family-consumer-cfg.v1",
        "the map version changed; check that the rows below changed with it",
    );
    assert_eq!(
        PINNED_MAP.len(),
        ApplePlatform::COUNT,
        "the pinned table must cover every governed family",
    );

    let rendered: Vec<(&str, String)> = ApplePlatform::ALL
        .into_iter()
        .map(|family| (family.as_str(), consumer_cfg(family).predicate()))
        .collect();
    let pinned: Vec<(&str, String)> = PINNED_MAP
        .into_iter()
        .map(|(family, predicate)| (family, predicate.to_owned()))
        .collect();

    assert_eq!(
        rendered, pinned,
        "the family-to-consumer-`cfg` map changed. It is versioned Tiler data: generated code in \
         an already-built consumer was gated by the old rows, so bump `MAP_VERSION` in the same \
         change that edits this table",
    );
}

/// No two families share a predicate.
///
/// Two families that did would be indistinguishable at the consumer, and the
/// generated selector would define one name twice — so the delivery would either
/// fail to compile or hand one family's payload to the other. This is the
/// property the whole map exists to have.
#[test]
fn no_two_families_share_a_consumer_predicate() {
    let predicates: BTreeSet<String> = ApplePlatform::ALL
        .into_iter()
        .map(|family| consumer_cfg(family).predicate())
        .collect();
    assert_eq!(
        predicates.len(),
        ApplePlatform::COUNT,
        "two governed families render one predicate: {predicates:?}",
    );
}

/// Every family's predicate is true on exactly its own target and false on every
/// other checked target.
///
/// This is the check that makes the map evidence rather than an assertion: the
/// predicate this module emits is evaluated against `rustc`'s own `cfg` answer
/// for fifteen real targets, so the table is compared with the compiler that
/// will evaluate it rather than with a second reading of the same research note.
#[test]
fn each_family_predicate_matches_exactly_its_own_rust_target() {
    assert_eq!(
        CHECKED_TARGETS.len(),
        15,
        "the population this test covers is fifteen named targets, counted",
    );

    for (triple, expected) in CHECKED_TARGETS {
        let target = target_cfg(triple);
        let matched: Vec<ApplePlatform> = ApplePlatform::ALL
            .into_iter()
            .filter(|family| evaluate(&consumer_cfg(*family).predicate(), &target))
            .collect();

        match expected {
            Some(family) => assert_eq!(
                matched,
                vec![family],
                "{triple} must select exactly the {} family",
                family.as_str(),
            ),
            None => assert!(
                matched.is_empty(),
                "{triple} is not a governed artifact family, but it selected {matched:?}",
            ),
        }
    }
}

/// The three families sharing `target_os = "ios"` stay apart.
///
/// Named separately from the sweep above because this is the specific hazard the
/// second key exists for. `docs/backends/metal.md` forbids relabelling bytes
/// across these families, and
/// `docs/research/apple-targets/numerical-behaviour.md` records that a
/// wrong-family `metallib` loads and dispatches without error — so a predicate
/// that conflated them would produce no symptom at all.
#[test]
fn the_three_ios_families_are_kept_apart() {
    let device = consumer_cfg(ApplePlatform::IOsDevice).predicate();
    let simulator = consumer_cfg(ApplePlatform::IOsSimulator).predicate();
    let catalyst = consumer_cfg(ApplePlatform::MacCatalyst).predicate();
    assert_ne!(device, simulator);
    assert_ne!(device, catalyst);
    assert_ne!(simulator, catalyst);

    for (triple, family) in [
        ("aarch64-apple-ios", ApplePlatform::IOsDevice),
        ("aarch64-apple-ios-sim", ApplePlatform::IOsSimulator),
        ("aarch64-apple-ios-macabi", ApplePlatform::MacCatalyst),
    ] {
        let target = target_cfg(triple);
        assert!(
            target.contains(&("target_os".to_owned(), "ios".to_owned())),
            "{triple} must report `target_os = \"ios\"`, or this test proves nothing",
        );
        for other in [
            ApplePlatform::IOsDevice,
            ApplePlatform::IOsSimulator,
            ApplePlatform::MacCatalyst,
        ] {
            assert_eq!(
                evaluate(&consumer_cfg(other).predicate(), &target),
                other == family,
                "{triple} against the {} predicate",
                other.as_str(),
            );
        }
    }
}

/// The evaluator refuses a predicate shape it does not model.
///
/// Without this, `each_family_predicate_matches_exactly_its_own_rust_target`
/// would be trusted on the strength of a parser nobody had watched fail — and a
/// widened predicate would be evaluated by a model that no longer describes it.
#[test]
#[should_panic(expected = "is not a shape this evaluator models")]
fn the_evaluator_refuses_a_shape_it_does_not_model() {
    let target = target_cfg("aarch64-apple-darwin");
    evaluate(r#"cfg(target_os = "macos")"#, &target);
}

/// The evaluator refuses a family predicate carrying a third key.
#[test]
#[should_panic(expected = "names the two governed keys")]
fn the_evaluator_refuses_a_third_key() {
    let target = target_cfg("aarch64-apple-darwin");
    evaluate(
        r#"all(target_os = "macos", target_abi = "", target_vendor = "apple")"#,
        &target,
    );
}

/// `any` and `not` mean what the generated selector's catch-all needs them to.
///
/// The catch-all arm is the one thing standing between a target that matches no
/// selected family and an undefined name, so the evaluator's reading of it is
/// checked directly rather than only through the arms built from it.
#[test]
fn the_evaluator_reads_the_catch_all_shape() {
    let macos = target_cfg("aarch64-apple-darwin");
    let device = consumer_cfg(ApplePlatform::IOsDevice).predicate();
    let simulator = consumer_cfg(ApplePlatform::IOsSimulator).predicate();
    let catch_all = format!("not(any({device}, {simulator}))");

    assert!(
        evaluate(&catch_all, &macos),
        "macOS matches neither iOS family, so the catch-all must hold",
    );
    assert!(
        !evaluate(
            &format!(
                "not(any({}))",
                consumer_cfg(ApplePlatform::MacOs).predicate()
            ),
            &macos,
        ),
        "macOS matches the macOS family, so its catch-all must not hold",
    );
    assert!(evaluate(
        &format!("any({device}, {simulator})"),
        &target_cfg("aarch64-apple-ios")
    ));
}

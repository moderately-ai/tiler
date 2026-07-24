//! The checked correspondence between the two Apple target vocabularies.
//!
//! `tiler-metal` and `tiler-metal-aot` each own an MSL language version, an
//! Apple artifact family, and a deployment minimum. [`crate::target`] and
//! `tiler_metal_aot::input` record why that duplication is owned rather than
//! removed; this module is the mechanism that keeps it honest.
//!
//! # Why the check lives here, and can only live here
//!
//! `tiler-metal-aot` is dependency-free by design and cannot see this crate.
//! This crate's development dependency on the driver is therefore the only edge
//! in the workspace over which both vocabularies are visible at once. The same
//! fact bounds what this module can be: a test, never a production conversion.
//! A `MetalTargetFacts` → `MetalTarget` translation needs a normal dependency in
//! one direction or the other, so it belongs to whichever component eventually
//! orchestrates emission and compilation together, and that component inherits
//! the obligation these tests state.
//!
//! # Why it is total rather than pointwise
//!
//! `crate::golden_compilation` already asserts that the target the goldens
//! declare is the target the driver compiles them for. That check is
//! *pointwise*: it compares one macOS 13.0 MSL 3.1 target in both spellings, and
//! it stays green if either crate gains a language standard or an artifact
//! family the other does not have — which is exactly the divergence the two
//! vocabularies are exposed to. It proves the fixtures are compiled for the
//! target they claim, and nothing about the vocabularies.
//!
//! The maps below are total instead. Every arm must produce the counterpart its
//! own variant determines, and no wildcard can invent one, so:
//!
//! - a variant added to [`MslLanguageVersion`] or [`MetalPlatform`] fails to
//!   compile at the emitter index below, in the crate that defines them, where
//!   `#[non_exhaustive]` does not apply; and
//! - a variant added to `MslVersion` or `ApplePlatform` fails to compile at the
//!   driver index below, which is an out-of-crate match and is why ADR 0074
//!   convention 5b keeps those two driver enums exhaustive.
//!
//! Each pair table is declared with the matching count, so a widened index
//! function that is not accompanied by a new pair is an array-length error
//! rather than a silently short table.

use tiler_metal_aot::input::{ApplePlatform, DeploymentMinimum, MslVersion};

use crate::target::{MetalDeploymentMinimum, MetalPlatform, MslLanguageVersion};

/// The number of Apple artifact families each vocabulary names.
const FAMILY_COUNT: usize = 3;

/// The number of MSL standards each vocabulary names.
const LANGUAGE_COUNT: usize = 2;

/// Assigns each emitter family a dense index.
///
/// The match is exhaustive over [`MetalPlatform`], which this crate defines, so
/// the enum's `#[non_exhaustive]` attribute does not reach it and a new family
/// stops the build here until it is indexed.
const fn emitter_family_index(family: MetalPlatform) -> usize {
    match family {
        MetalPlatform::MacOs => 0,
        MetalPlatform::IOsDevice => 1,
        MetalPlatform::IOsSimulator => 2,
    }
}

/// Assigns each driver family a dense index.
///
/// The match is exhaustive over `ApplePlatform` from outside the crate that
/// defines it. That compiles only because the driver deliberately leaves the
/// enum exhaustive, and it is the guard that makes a family added there a build
/// failure here rather than a silent divergence.
const fn driver_family_index(family: ApplePlatform) -> usize {
    match family {
        ApplePlatform::MacOs => 0,
        ApplePlatform::IOsDevice => 1,
        ApplePlatform::IOsSimulator => 2,
    }
}

/// Assigns each emitter language standard a dense index.
///
/// Exhaustive over [`MslLanguageVersion`] within its defining crate, for the
/// same reason [`emitter_family_index`] is.
const fn emitter_language_index(language: MslLanguageVersion) -> usize {
    match language {
        MslLanguageVersion::Metal3_0 => 0,
        MslLanguageVersion::Metal3_1 => 1,
    }
}

/// Assigns each driver language standard a dense index.
///
/// Exhaustive over `MslVersion` from outside its defining crate, for the same
/// reason [`driver_family_index`] is.
const fn driver_language_index(language: MslVersion) -> usize {
    match language {
        MslVersion::Metal3_0 => 0,
        MslVersion::Metal3_1 => 1,
    }
}

/// Every Apple artifact family, in both spellings.
const FAMILIES: [(MetalPlatform, ApplePlatform); FAMILY_COUNT] = [
    (MetalPlatform::MacOs, ApplePlatform::MacOs),
    (MetalPlatform::IOsDevice, ApplePlatform::IOsDevice),
    (MetalPlatform::IOsSimulator, ApplePlatform::IOsSimulator),
];

/// Every MSL standard, in both spellings.
const LANGUAGES: [(MslLanguageVersion, MslVersion); LANGUAGE_COUNT] = [
    (MslLanguageVersion::Metal3_0, MslVersion::Metal3_0),
    (MslLanguageVersion::Metal3_1, MslVersion::Metal3_1),
];

/// Deployment minimums exercised across both vocabularies.
///
/// A deployment minimum is a `(major, minor)` pair rather than a variant set, so
/// there is nothing to enumerate exhaustively. What can diverge is the component
/// width, the accessors, and the rendering — one side reaches an emitted header,
/// the other an `air64-apple-*` triple — so those are what the test compares.
/// The macOS 13.0 row is the governed golden target; the others exercise
/// nonzero and two-digit minor components across the macOS and iOS ranges.
const DEPLOYMENT_MINIMUMS: [(u16, u16); 5] = [(13, 0), (14, 2), (16, 0), (17, 4), (26, 10)];

/// Neither vocabulary may name an artifact family the other does not.
///
/// Failure here means the emitter can describe a target the driver cannot
/// compile for, or the driver can compile for a family no emitted source can
/// declare. Both produce an artifact whose provenance header and whose
/// compilation disagree about what it is.
#[test]
fn the_family_table_covers_every_variant_of_both_vocabularies_exactly_once() {
    let mut emitter_seen = [false; FAMILY_COUNT];
    let mut driver_seen = [false; FAMILY_COUNT];
    for (emitter, driver) in FAMILIES {
        let emitter_index = emitter_family_index(emitter);
        let driver_index = driver_family_index(driver);
        assert!(
            emitter_index < FAMILY_COUNT,
            "FAMILY_COUNT does not cover the emitter family {emitter}"
        );
        assert!(
            driver_index < FAMILY_COUNT,
            "FAMILY_COUNT does not cover the driver family {}",
            driver.as_str()
        );
        assert!(
            !emitter_seen[emitter_index],
            "{emitter} is paired more than once"
        );
        assert!(
            !driver_seen[driver_index],
            "{} is paired more than once",
            driver.as_str()
        );
        emitter_seen[emitter_index] = true;
        driver_seen[driver_index] = true;
    }
    assert!(
        emitter_seen.into_iter().all(|seen| seen),
        "an emitter artifact family has no driver counterpart"
    );
    assert!(
        driver_seen.into_iter().all(|seen| seen),
        "a driver artifact family has no emitter counterpart"
    );
}

/// Paired families must carry the same stable identifier.
///
/// The emitter writes its spelling into the provenance header and the driver
/// records its own in `tiler_metal_aot::record::ArtifactProvenance`. Two
/// spellings for one family would make those two records incomparable.
#[test]
fn both_vocabularies_spell_every_artifact_family_identically() {
    for (emitter, driver) in FAMILIES {
        assert_eq!(
            emitter.as_str(),
            driver.as_str(),
            "the two vocabularies disagree on {emitter}"
        );
    }
}

/// Neither vocabulary may name an MSL standard the other does not.
///
/// Failure here means source can be emitted against a standard the driver
/// cannot select, or compiled under one no emitted header can declare.
#[test]
fn the_language_table_covers_every_variant_of_both_vocabularies_exactly_once() {
    let mut emitter_seen = [false; LANGUAGE_COUNT];
    let mut driver_seen = [false; LANGUAGE_COUNT];
    for (emitter, driver) in LANGUAGES {
        let emitter_index = emitter_language_index(emitter);
        let driver_index = driver_language_index(driver);
        assert!(
            emitter_index < LANGUAGE_COUNT,
            "LANGUAGE_COUNT does not cover the emitter standard {emitter}"
        );
        assert!(
            driver_index < LANGUAGE_COUNT,
            "LANGUAGE_COUNT does not cover the driver standard {}",
            driver.std_token()
        );
        assert!(
            !emitter_seen[emitter_index],
            "{emitter} is paired more than once"
        );
        assert!(
            !driver_seen[driver_index],
            "{} is paired more than once",
            driver.std_token()
        );
        emitter_seen[emitter_index] = true;
        driver_seen[driver_index] = true;
    }
    assert!(
        emitter_seen.into_iter().all(|seen| seen),
        "an emitter MSL standard has no driver counterpart"
    );
    assert!(
        driver_seen.into_iter().all(|seen| seen),
        "a driver MSL standard has no emitter counterpart"
    );
}

/// Paired standards must produce the same `-std` token.
///
/// The emitter writes the token into the provenance header and the driver passes
/// it to `metal` as `-std=<token>`. A divergence would put a standard in the
/// header that the compilation did not use.
#[test]
fn both_vocabularies_spell_every_language_standard_identically() {
    for (emitter, driver) in LANGUAGES {
        assert_eq!(
            emitter.std_token(),
            driver.std_token(),
            "the two vocabularies disagree on {emitter}"
        );
    }
}

/// Both deployment minimums must carry and render the same version.
///
/// A component that truncated on one side, or a rendering that differed, would
/// put one version in the emitted header and another in the target triple.
#[test]
fn both_vocabularies_carry_and_render_a_deployment_minimum_identically() {
    for (major, minor) in DEPLOYMENT_MINIMUMS {
        let emitter = MetalDeploymentMinimum::new(major, minor);
        let driver = DeploymentMinimum::new(major, minor);
        assert_eq!(emitter.major(), driver.major(), "{major}.{minor}");
        assert_eq!(emitter.minor(), driver.minor(), "{major}.{minor}");
        assert_eq!(
            emitter.to_string(),
            driver.to_string(),
            "the two vocabularies render {major}.{minor} differently"
        );
    }
}
